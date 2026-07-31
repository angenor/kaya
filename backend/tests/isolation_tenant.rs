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
    // ── Cycle 002 — établissements, services, capacités ────────────────────────────────────
    ("/api/v1/etablissements", Regime::Isole),
    ("/api/v1/etablissements/{etablissement_id}", Regime::Isole),
    (
        "/api/v1/etablissements/{etablissement_id}/services",
        Regime::Isole,
    ),
    (
        "/api/v1/etablissements/{etablissement_id}/services/{module_code}",
        Regime::Isole,
    ),
    (
        "/api/v1/etablissements/{etablissement_id}/services/{module_code}/capacites",
        Regime::Isole,
    ),
    (
        "/api/v1/etablissements/{etablissement_id}/points-de-vente",
        Regime::Isole,
    ),
    ("/api/v1/points-de-vente/{point_de_vente_id}", Regime::Isole),
    (
        "/api/v1/points-de-vente/{point_de_vente_id}/tables",
        Regime::Isole,
    ),
    // Identité visuelle — trois chemins, tous isolés. Le téléversement écrit au stockage objet
    // sous une clé qui PORTE LE TENANT : lire l'objet d'un autre client est impossible même en
    // devinant le reste du chemin.
    ("/api/v1/branding", Regime::Isole),
    ("/api/v1/branding/logo", Regime::Isole),
    // L'aperçu n'enregistre rien et ne lit aucune table : il rend le corps reçu, mis en forme.
    // C'est le seul régime `SansTenant` du cycle 002, et il est justifié ici.
    ("/api/v1/branding/apercu", Regime::SansTenant),
    // Configuration héritée — la cible vient des paramètres de requête. L'isolation à CHAQUE
    // niveau de la descente est vérifiée par `backend/tests/configuration_heritee.rs`.
    ("/api/v1/configuration", Regime::Isole),
    // ── Les trois référentiels GLOBAUX ─────────────────────────────────────────────────────
    //
    // Ils rendent **les mêmes lignes à tous les tenants**, et c'est correct : ce sont des
    // référentiels partagés, sans `tenant_id`. Ils sont déclarés `SansTenant` parce qu'ils ne
    // touchent aucune donnée de client.
    //
    // **Cette exception est transformée en assertion** par
    // `p08_les_referentiels_rendent_la_meme_chose_aux_deux_tenants` : sans elle, un futur
    // relecteur verrait trois routes `SansTenant` et conclurait à une fuite — ou pire,
    // « corrigerait » en y ajoutant un filtrage par tenant, ce qui multiplierait le référentiel
    // par le nombre de clients (cadrage §14.3).
    ("/api/v1/referentiels/modules-activite", Regime::SansTenant),
    ("/api/v1/referentiels/capacites", Regime::SansTenant),
    ("/api/v1/referentiels/profils-stock", Regime::SansTenant),
    // Sonde de santé — publique, sans contexte, elle ne touche aucune table applicative
    // (`contracts/http-api.md` §1). Toute autre route déclarée ainsi doit être justifiée par
    // écrit, ici même.
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

    // **Toutes les colonnes obligatoires sont renseignées**, `commune` comprise. Sans elle, la
    // migration 0007 ferait échouer l'insertion sur une violation de `NOT NULL` — et le test
    // passerait au vert **sans jamais exercer `WITH CHECK`**. Une porte qui échoue pour la
    // mauvaise raison est indistinguable d'une porte qui fonctionne.
    let resultat = sqlx::query!(
        r#"
        INSERT INTO etablissements.etablissement
            (id, tenant_id, nom, fuseau_horaire, devise, commune)
        VALUES ($1, $2, 'intrusion', 'Africa/Abidjan', 'XOF', 'Abengourou')
        "#,
        uuid::Uuid::now_v7(),
        b.tenant_id
    )
    .execute(&mut *tx)
    .await;

    let Err(erreur) = resultat else {
        panic!(
            "le tenant A a inséré une ligne au nom du tenant B : WITH CHECK est absent ou \
             inopérant. C'est la fuite la moins visible du produit — elle ne se voit dans aucune \
             lecture."
        );
    };

    // Et l'échec vient bien de la **politique de sécurité**, pas d'une contrainte d'intégrité qui
    // se trouverait passer par là. PostgreSQL rend `42501` (insufficient_privilege) sur une
    // violation de `WITH CHECK`.
    let code = erreur
        .as_database_error()
        .and_then(|e| e.code())
        .map(|c| c.to_string())
        .unwrap_or_default();
    assert_eq!(
        code, "42501",
        "l'insertion croisée a échoué avec le code {code} au lieu de 42501 \
         (insufficient_privilege).\n\
         Ce n'est donc PAS la politique WITH CHECK qui l'a refusée, mais une contrainte \
         d'intégrité — un NOT NULL, une clé étrangère. La porte passerait au vert sans avoir \
         exercé l'isolation en écriture. Erreur complète : {erreur}"
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

// =================================================================================================
//  Cycle 002 — isolation des établissements, des services et des capacités
// =================================================================================================

/// Construit un en-tête de contexte pour un tenant donné.
fn en_tetes(tenant_id: uuid::Uuid) -> [(&'static str, String); 2] {
    [
        (kaya_api::contexte::EN_TETE_TENANT, tenant_id.to_string()),
        (
            kaya_api::contexte::EN_TETE_COMPTE,
            uuid::Uuid::now_v7().to_string(),
        ),
    ]
}

/// **Isolation par endpoint sur les cinq chemins du cycle 002.**
///
/// Le tenant A vise l'établissement du tenant B par identifiant direct, sur chaque verbe. Deux
/// vérifications par appel : le statut refuse, **et** rien n'a été écrit chez B.
///
/// `404` plutôt que `403` : du point de vue de A, l'établissement de B **n'existe pas**. Un `403`
/// confirmerait son existence — une fuite ténue mais réelle, qui permet d'énumérer les
/// établissements des autres clients.
#[actix_web::test]
async fn p08_cycle_002_appels_croises_ne_voient_ni_n_ecrivent_rien() {
    let pool_owner = commun::pool_owner().await;
    let a = commun::creer_tenant(&pool_owner, "P-08 002 tenant A").await;
    let b = commun::creer_tenant(&pool_owner, "P-08 002 tenant B").await;

    let pool = commun::pool_app().await;
    let app = monter_application!(pool.clone());
    let [tenant_a, compte_a] = en_tetes(a.tenant_id);

    // B active un service, pour que A ait quelque chose à essayer de voir.
    let service_b = kaya_etablissements::modules::ServiceModules::nouveau(
        pool.clone(),
        kaya_synchronisation::outbox::PgOutboxWriter::nouveau(),
    );
    service_b
        .basculer(
            b.tenant_id,
            b.etablissement_id,
            "RESTAURATION",
            kaya_etablissements::modules::BasculerService {
                id: uuid::Uuid::now_v7(),
                actif: true,
            },
        )
        .await
        .expect("activation chez B");

    let etb_b = b.etablissement_id;
    let lectures = [
        format!("/api/v1/etablissements/{etb_b}"),
        format!("/api/v1/etablissements/{etb_b}/services"),
        format!("/api/v1/etablissements/{etb_b}/services/RESTAURATION/capacites"),
    ];

    for chemin in &lectures {
        let requete = actix_web::test::TestRequest::get()
            .uri(chemin)
            .insert_header((tenant_a.0, tenant_a.1.clone()))
            .insert_header((compte_a.0, compte_a.1.clone()))
            .to_request();
        let reponse = actix_web::test::call_service(&app, requete).await;
        assert_eq!(
            reponse.status().as_u16(),
            404,
            "le tenant A a obtenu {} sur {chemin} — l'établissement du tenant B doit lui être \
             indiscernable d'un établissement inexistant",
            reponse.status()
        );
    }

    // --- Écriture croisée : A active un service chez B ------------------------------------
    let requete = actix_web::test::TestRequest::put()
        .uri(&format!("/api/v1/etablissements/{etb_b}/services/BAR"))
        .insert_header((tenant_a.0, tenant_a.1.clone()))
        .insert_header((compte_a.0, compte_a.1.clone()))
        .set_json(serde_json::json!({ "id": uuid::Uuid::now_v7(), "actif": true }))
        .to_request();
    let reponse = actix_web::test::call_service(&app, requete).await;
    assert_eq!(
        reponse.status().as_u16(),
        404,
        "le tenant A a pu activer un service chez le tenant B : statut {}",
        reponse.status()
    );

    // --- Écriture croisée : A déclare une capacité chez B ----------------------------------
    let requete = actix_web::test::TestRequest::post()
        .uri(&format!(
            "/api/v1/etablissements/{etb_b}/services/RESTAURATION/capacites"
        ))
        .insert_header((tenant_a.0, tenant_a.1.clone()))
        .insert_header((compte_a.0, compte_a.1.clone()))
        .set_json(serde_json::json!({
            "id": uuid::Uuid::now_v7(),
            "capacite_code": "STOCK",
            "profil_code": "SIMPLE",
        }))
        .to_request();
    let reponse = actix_web::test::call_service(&app, requete).await;
    assert_eq!(
        reponse.status().as_u16(),
        404,
        "le tenant A a pu déclarer une capacité chez le tenant B"
    );

    // --- Et rien n'a été écrit chez B, quels que soient les statuts rendus -----------------
    let mut tx = pool_owner.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, b.tenant_id)
        .await
        .expect("pose du tenant B");

    let services: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) AS "compte!"
        FROM etablissements.etablissement_module
        WHERE etablissement_id = $1
        "#,
        b.etablissement_id
    )
    .fetch_one(&mut *tx)
    .await
    .expect("comptage des services");
    assert_eq!(
        services, 1,
        "le tenant B porte {services} service(s) au lieu du seul qu'il a activé : le tenant A a \
         écrit chez lui"
    );

    let capacites: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) AS "compte!"
        FROM etablissements.module_capacite mc
        JOIN etablissements.etablissement_module em ON em.id = mc.etablissement_module_id
        WHERE em.etablissement_id = $1
        "#,
        b.etablissement_id
    )
    .fetch_one(&mut *tx)
    .await
    .expect("comptage des capacités");
    assert_eq!(
        capacites, 0,
        "{capacites} capacité(s) déclarée(s) chez le tenant B par le tenant A"
    );

    tx.rollback().await.expect("rollback");
}

/// **La liste des établissements ne rend QUE ceux du tenant appelant.**
///
/// Le cas le plus facile à casser : un handler qui oublierait de poser le tenant courant verrait
/// zéro ligne — donc passerait les tests d'appel croisé ci-dessus, qui attendent justement de ne
/// rien voir. Seule une lecture **nominale**, où l'on attend un résultat non vide, distingue
/// « isolé » de « cassé ».
#[actix_web::test]
async fn p08_la_liste_des_etablissements_est_bornee_au_tenant() {
    let pool_owner = commun::pool_owner().await;
    let a = commun::creer_tenant(&pool_owner, "P-08 liste A").await;
    let b = commun::creer_tenant(&pool_owner, "P-08 liste B").await;

    let pool = commun::pool_app().await;
    let app = monter_application!(pool.clone());
    let [tenant_a, compte_a] = en_tetes(a.tenant_id);

    let requete = actix_web::test::TestRequest::get()
        .uri("/api/v1/etablissements")
        .insert_header((tenant_a.0, tenant_a.1.clone()))
        .insert_header((compte_a.0, compte_a.1.clone()))
        .to_request();
    let corps: serde_json::Value = actix_web::test::call_and_read_body_json(&app, requete).await;

    let ids: Vec<String> = corps
        .as_array()
        .expect("la liste doit être un tableau")
        .iter()
        .map(|e| e["id"].as_str().unwrap_or_default().to_owned())
        .collect();

    assert!(
        ids.contains(&a.etablissement_id.to_string()),
        "le tenant A ne voit pas son PROPRE établissement : le handler ne pose pas le tenant \
         courant, et tous les tests d'appel croisé passeraient pour la mauvaise raison"
    );
    assert!(
        !ids.contains(&b.etablissement_id.to_string()),
        "le tenant A voit l'établissement du tenant B dans sa liste"
    );
}

/// **Les trois référentiels rendent la MÊME chose aux deux tenants** — et c'est correct.
///
/// Sans cette assertion, trois routes déclarées `SansTenant` dans [`COUVERTURE`] ressembleraient à
/// une exception non vérifiée. Un futur relecteur conclurait à une fuite, ou « corrigerait » en y
/// ajoutant un filtrage par tenant — ce qui multiplierait cinq lignes de référentiel par le nombre
/// de clients et rendrait impossible l'ajout d'une valeur par configuration (cadrage §14.3).
///
/// **Ce test transforme une exception en assertion.**
#[actix_web::test]
async fn p08_les_referentiels_rendent_la_meme_chose_aux_deux_tenants() {
    let pool_owner = commun::pool_owner().await;
    let a = commun::creer_tenant(&pool_owner, "P-08 référentiel A").await;
    let b = commun::creer_tenant(&pool_owner, "P-08 référentiel B").await;

    let pool = commun::pool_app().await;
    let app = monter_application!(pool.clone());

    for chemin in [
        "/api/v1/referentiels/modules-activite",
        "/api/v1/referentiels/capacites",
        "/api/v1/referentiels/profils-stock",
    ] {
        let mut reponses = Vec::new();
        for tenant in [a.tenant_id, b.tenant_id] {
            let [t, c] = en_tetes(tenant);
            let requete = actix_web::test::TestRequest::get()
                .uri(chemin)
                .insert_header((t.0, t.1))
                .insert_header((c.0, c.1))
                .to_request();
            let corps: serde_json::Value =
                actix_web::test::call_and_read_body_json(&app, requete).await;
            reponses.push(corps);
        }

        assert!(
            !reponses[0].as_array().is_some_and(|a| a.is_empty()),
            "{chemin} rend une liste vide : le référentiel est inaccessible, et l'égalité \
             ci-dessous serait vraie sans rien prouver"
        );
        assert_eq!(
            reponses[0], reponses[1],
            "{chemin} rend des lignes DIFFÉRENTES aux deux tenants.\n\
             Les référentiels sont GLOBAUX et partagés : c'est voulu (research.md R-01). Une \
             divergence signifie qu'un filtrage par tenant a été introduit — ce qui obligerait à \
             écrire chaque valeur chez chaque client, contre le cadrage §14.3."
        );
    }
}

/// **Isolation des points de vente** — y compris par identifiant direct, hors du chemin de
/// l'établissement.
///
/// `PATCH /points-de-vente/{id}` et `PUT /points-de-vente/{id}/tables` ne portent **pas**
/// l'identifiant de l'établissement dans leur chemin : rien dans l'URL ne rattache la ressource à
/// un tenant. C'est exactement la forme où l'isolation repose entièrement sur la politique de
/// sécurité — et donc celle qu'il faut vérifier en premier.
#[actix_web::test]
async fn p08_les_points_de_vente_sont_isoles_meme_par_identifiant_direct() {
    let pool_owner = commun::pool_owner().await;
    let a = commun::creer_tenant(&pool_owner, "P-08 PDV A").await;
    let b = commun::creer_tenant(&pool_owner, "P-08 PDV B").await;

    let pool = commun::pool_app().await;
    let app = monter_application!(pool.clone());

    // B active RESTAURATION et crée son point de vente.
    let modules_b = kaya_etablissements::modules::ServiceModules::nouveau(
        pool.clone(),
        kaya_synchronisation::outbox::PgOutboxWriter::nouveau(),
    );
    modules_b
        .basculer(
            b.tenant_id,
            b.etablissement_id,
            "RESTAURATION",
            kaya_etablissements::modules::BasculerService {
                id: uuid::Uuid::now_v7(),
                actif: true,
            },
        )
        .await
        .expect("activation chez B");

    let pdv_b = kaya_etablissements::points_de_vente::ServicePointsDeVente::nouveau(
        pool.clone(),
        kaya_synchronisation::outbox::PgOutboxWriter::nouveau(),
    );
    let (point_b, _) = pdv_b
        .creer(
            b.tenant_id,
            b.etablissement_id,
            kaya_etablissements::points_de_vente::CreerPointDeVente {
                id: uuid::Uuid::now_v7(),
                module_code: "RESTAURATION".to_owned(),
                nom: "Terrasse de B".to_owned(),
                caisse_id: None,
            },
        )
        .await
        .expect("création du point de vente de B");

    let [tenant_a, compte_a] = en_tetes(a.tenant_id);

    // A modifie le point de vente de B, par identifiant direct.
    let requete = actix_web::test::TestRequest::patch()
        .uri(&format!("/api/v1/points-de-vente/{}", point_b.id))
        .insert_header((tenant_a.0, tenant_a.1.clone()))
        .insert_header((compte_a.0, compte_a.1.clone()))
        .set_json(serde_json::json!({ "nom": "détourné" }))
        .to_request();
    let reponse = actix_web::test::call_service(&app, requete).await;
    assert_eq!(
        reponse.status().as_u16(),
        404,
        "le tenant A a modifié le point de vente du tenant B par identifiant direct : statut {}",
        reponse.status()
    );

    // A pose des tables sur le point de vente de B.
    let requete = actix_web::test::TestRequest::put()
        .uri(&format!("/api/v1/points-de-vente/{}/tables", point_b.id))
        .insert_header((tenant_a.0, tenant_a.1.clone()))
        .insert_header((compte_a.0, compte_a.1.clone()))
        .set_json(serde_json::json!({
            "tables": [{ "id": uuid::Uuid::now_v7(), "libelle": "intrusion" }]
        }))
        .to_request();
    let reponse = actix_web::test::call_service(&app, requete).await;
    assert_eq!(
        reponse.status().as_u16(),
        404,
        "le tenant A a posé une table chez le tenant B"
    );

    // Et le point de vente de B est intact.
    let apres = pdv_b
        .lister(b.tenant_id, b.etablissement_id)
        .await
        .expect("relecture chez B");
    assert_eq!(apres.len(), 1);
    assert_eq!(apres[0].nom, "Terrasse de B", "le nom a été modifié par A");
    assert!(
        apres[0].tables.is_empty(),
        "des tables ont été posées chez B par A : {:?}",
        apres[0].tables
    );
}
