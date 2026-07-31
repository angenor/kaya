//! **Porte P-08** — le tenant A ne lit ni n'écrit aucune ligne du tenant B, **sur chaque
//! endpoint**.
//!
//! # Le mécanisme, et pourquoi il n'est pas une liste de tests écrits à la main
//!
//! Le test est **paramétré sur la liste des routes du contrat OpenAPI**. Chaque route découverte
//! doit figurer dans [`COUVERTURE`] ; une route absente fait échouer la porte, en la nommant.
//!
//! Sans cela, la porte protégerait exactement les endpoints auxquels quelqu'un a pensé, et
//! l'endpoint ajouté un vendredi soir serait celui qui fuit. Ici, ajouter une route **sans
//! décider** de son régime d'isolation casse le build.
//!
//! Deux régimes seulement, et le second doit se justifier :
//!
//! - [`Regime::Isole`] — l'endpoint touche des données de tenant. Un appel croisé doit ne rien
//!   voir et ne rien écrire.
//! - [`Regime::SansTenant`] — l'endpoint ne touche **aucune table applicative**. La sonde de
//!   santé est le seul cas légitime : elle est publique et ne lit rien d'un client.

mod commun;

use std::collections::BTreeSet;

use kaya_api::application;

/// Régime d'isolation d'un endpoint.
// Les variantes ne sont pas encore construites : `COUVERTURE` est vide tant qu'aucune route
// n'est montée. Les déclarer maintenant est le sujet de la porte — le régime doit exister avant
// la première route, sinon la première route serait ajoutée sans que rien ne l'oblige à choisir.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Regime {
    /// Touche des données de tenant — l'appel croisé est vérifié.
    Isole,
    /// Ne touche aucune table applicative. À justifier au cas par cas, jamais par défaut.
    SansTenant,
}

/// Régime déclaré de chaque route du contrat.
///
/// **Cette table est la déclaration, le contrat OpenAPI est la vérité.** L'écart entre les deux
/// fait échouer la porte, dans les deux sens : une route non déclarée, comme une déclaration qui
/// ne correspond à aucune route.
const COUVERTURE: &[(&str, Regime)] = &[
    // Module doré. `GET` et `POST` partagent le chemin ; les deux sont vérifiés par
    // `p08_appel_croise_sur_endpoint_ne_voit_ni_n_ecrit_rien`.
    (
        "/api/v1/etablissements/{etablissement_id}/notes",
        Regime::Isole,
    ),
    // Sonde de santé — **seul** régime `SansTenant` légitime de ce cycle : publique, sans
    // contexte, elle ne touche aucune table applicative (`contracts/http-api.md` §1). Toute
    // autre route déclarée ainsi doit être justifiée par écrit, ici même.
    ("/health", Regime::SansTenant),
];

/// Les routes **réellement montées**, pas le squelette déclaratif.
///
/// `application::contrat_complet()` assemble l'application comme le fait `servir` et en extrait
/// le contrat. Lire `openapi::contrat()` à la place ne renverrait que titre et étiquettes : la
/// porte constaterait zéro route et passerait au vert avec des endpoints servis — le premier
/// état dans lequel cette porte s'est trouvée, et la raison pour laquelle la distinction est
/// écrite ici.
fn routes_du_contrat() -> BTreeSet<String> {
    application::contrat_complet()
        .paths
        .paths
        .keys()
        .cloned()
        .collect()
}

fn routes_declarees() -> BTreeSet<String> {
    COUVERTURE.iter().map(|(r, _)| (*r).to_owned()).collect()
}

#[test]
fn p08_toute_route_du_contrat_est_couverte() {
    let contrat = routes_du_contrat();
    let declarees = routes_declarees();

    let non_declarees: Vec<_> = contrat.difference(&declarees).cloned().collect();
    assert!(
        non_declarees.is_empty(),
        "P-08 ÉCHOUE — {} route(s) du contrat OpenAPI sans régime d'isolation déclaré :\n  {}\n\n\
         Ajouter chaque route à COUVERTURE dans ce fichier, avec son régime. Une route dont \
         personne n'a décidé du régime est une route dont personne n'a vérifié l'isolation.",
        non_declarees.len(),
        non_declarees.join("\n  ")
    );

    let fantomes: Vec<_> = declarees.difference(&contrat).cloned().collect();
    assert!(
        fantomes.is_empty(),
        "P-08 ÉCHOUE — {} route(s) déclarée(s) qui n'existent plus au contrat :\n  {}\n\n\
         Une déclaration périmée donne l'illusion d'une couverture. La retirer.",
        fantomes.len(),
        fantomes.join("\n  ")
    );
}

/// Isolation **au niveau de la base**, indépendamment de tout endpoint.
///
/// Ce test tient même quand aucune route n'est montée : c'est la garantie de fond sur laquelle
/// repose l'isolation par endpoint. Si celle-ci tombait, aucun test d'endpoint ne pourrait la
/// rattraper.
#[tokio::test]
async fn p08_un_tenant_ne_lit_jamais_les_lignes_d_un_autre() {
    let pool_owner = commun::pool_owner().await;
    let a = commun::creer_tenant(&pool_owner, "P-08 tenant A").await;
    let b = commun::creer_tenant(&pool_owner, "P-08 tenant B").await;

    let pool = commun::pool_app().await;
    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, a.tenant_id)
        .await
        .expect("pose du tenant A");

    // Lecture croisée : A demande explicitement l'établissement de B, par son identifiant.
    let vu: Option<uuid::Uuid> = sqlx::query_scalar!(
        "SELECT id FROM etablissements.etablissement WHERE id = $1",
        b.etablissement_id
    )
    .fetch_optional(&mut *tx)
    .await
    .expect("lecture croisée");

    assert!(
        vu.is_none(),
        "le tenant A a lu l'établissement du tenant B : l'isolation ne tient pas"
    );

    tx.rollback().await.expect("rollback");
}

/// **Écriture** croisée — le cas le moins visible et le plus grave.
///
/// `USING` seul filtrerait la lecture et laisserait passer l'insertion d'une ligne portant le
/// tenant d'autrui. C'est `WITH CHECK` qui la refuse, et c'est ce que ce test constate.
#[tokio::test]
async fn p08_un_tenant_ne_peut_pas_ecrire_chez_un_autre() {
    let pool_owner = commun::pool_owner().await;
    let a = commun::creer_tenant(&pool_owner, "P-08 écriture A").await;
    let b = commun::creer_tenant(&pool_owner, "P-08 écriture B").await;

    let pool = commun::pool_app().await;
    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, a.tenant_id)
        .await
        .expect("pose du tenant A");

    let resultat = sqlx::query!(
        r#"
        INSERT INTO etablissements.etablissement (id, tenant_id, nom, fuseau_horaire, devise)
        VALUES ($1, $2, 'intrusion', 'Africa/Abidjan', 'XOF')
        "#,
        uuid::Uuid::now_v7(),
        b.tenant_id
    )
    .execute(&mut *tx)
    .await;

    assert!(
        resultat.is_err(),
        "le tenant A a inséré une ligne au nom du tenant B : WITH CHECK est absent ou inopérant. \
         C'est la fuite la moins visible du produit — elle ne se voit dans aucune lecture."
    );

    let _ = tx.rollback().await;
}

/// Isolation **par endpoint** — le tenant A vise l'établissement du tenant B, par HTTP.
///
/// C'est le scénario exact de la porte P-08 : deux tenants seedés, chaque endpoint visé en
/// croisé. Un test au niveau de la base ne suffirait pas — il resterait possible qu'un handler
/// ouvre une transaction sans poser le tenant courant, et voie alors tout ou rien selon le
/// hasard du code.
#[actix_web::test]
async fn p08_appel_croise_sur_endpoint_ne_voit_ni_n_ecrit_rien() {
    let pool_owner = commun::pool_owner().await;
    let a = commun::creer_tenant(&pool_owner, "P-08 endpoint A").await;
    let b = commun::creer_tenant(&pool_owner, "P-08 endpoint B").await;

    let pool = commun::pool_app().await;
    let app = monter_application!(pool.clone());

    let compte_a = uuid::Uuid::now_v7();
    let chemin_de_b = format!("/api/v1/etablissements/{}/notes", b.etablissement_id);

    // --- Lecture croisée : A demande les notes de l'établissement de B --------------------
    let requete = actix_web::test::TestRequest::get()
        .uri(&chemin_de_b)
        .insert_header((kaya_api::contexte::EN_TETE_TENANT, a.tenant_id.to_string()))
        .insert_header((kaya_api::contexte::EN_TETE_COMPTE, compte_a.to_string()))
        .to_request();
    let reponse = actix_web::test::call_service(&app, requete).await;

    // `404` : du point de vue de A, l'établissement de B **n'existe pas**. Un `403` confirmerait
    // son existence — une fuite d'information ténue, mais réelle : elle permet d'énumérer les
    // établissements des autres clients.
    assert_eq!(
        reponse.status().as_u16(),
        404,
        "le tenant A a obtenu {} en lisant les notes de l'établissement du tenant B",
        reponse.status()
    );

    // --- Écriture croisée : A crée une note chez B ----------------------------------------
    let requete = actix_web::test::TestRequest::post()
        .uri(&chemin_de_b)
        .insert_header((kaya_api::contexte::EN_TETE_TENANT, a.tenant_id.to_string()))
        .insert_header((kaya_api::contexte::EN_TETE_COMPTE, compte_a.to_string()))
        .set_json(serde_json::json!({
            "id": uuid::Uuid::now_v7(),
            "texte": "intrusion",
        }))
        .to_request();
    let reponse = actix_web::test::call_service(&app, requete).await;

    assert_eq!(
        reponse.status().as_u16(),
        404,
        "le tenant A a pu écrire chez le tenant B : statut {}",
        reponse.status()
    );

    // Et rien n'a été écrit, quel que soit le statut renvoyé.
    let mut tx = pool_owner.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, b.tenant_id)
        .await
        .expect("pose du tenant B");
    let compte: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) AS "compte!"
        FROM etablissements.note_etablissement
        WHERE etablissement_id = $1
        "#,
        b.etablissement_id
    )
    .fetch_one(&mut *tx)
    .await
    .expect("comptage");

    assert_eq!(
        compte, 0,
        "{compte} note(s) écrite(s) chez le tenant B par le tenant A"
    );
}
