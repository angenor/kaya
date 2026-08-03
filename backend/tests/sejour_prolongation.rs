//! ★ **US5 et US7 — la prolongation et le changement de chambre.**
//!
//! # Les deux refus que ce fichier garde, et ce qui les distingue
//!
//! | Refus | Quand | Ce qu'il porte |
//! |---|---|---|
//! | `conflit_occupation_suivante` | Prolongation qui bute sur une réservation | L'unité, **l'instant** du conflit, et les **alternatives** |
//! | `unite_cible_occupee` | Changement vers une chambre prise | Le conflit, **sans déplacement partiel** |
//!
//! ⚠️ **Un message générique est un DÉFAUT** (FR-070). C'est la différence entre un refus
//! qu'Adjoua peut expliquer au client — « cette chambre est réservée à partir de 16 h 40, mais la
//! 108 est libre » — et un refus qu'elle contournera en notant la prolongation sur un papier.
//!
//! # ★ Ce que le changement d'unité prouve sur P-09, et qui n'est PAS évident
//!
//! Deux occupations **contiguës sur deux unités différentes** ne déclenchent **pas** la contrainte
//! d'exclusion — elle porte sur `(unite_id, periode)`, et les unités diffèrent. C'est **justement
//! pourquoi** ce fichier doit prouver qu'elle **se déclencherait** si les unités étaient les
//! mêmes : sans cette preuve, un changement d'unité vert ne dirait rien de la garantie.

mod commun;

use uuid::Uuid;

use commun::{JeuTenant, creer_tenant, pool_app, pool_owner};

const ROLE: &str = "receptionniste";

struct Decor {
    jeu: JeuTenant,
    unite_id: Uuid,
    unite_bis_id: Uuid,
    formule_id: Uuid,
}

/// Un établissement, **deux** chambres de la même catégorie, une nuitée.
///
/// Deux chambres : sans la seconde, « aucune alternative disponible » serait indistinguable de
/// « la recherche d'alternative ne fonctionne pas ».
async fn poser_decor(pool: &sqlx::PgPool, nom: &str) -> Decor {
    let jeu = creer_tenant(pool, nom).await;

    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, jeu.tenant_id)
        .await
        .expect("pose du tenant");

    sqlx::query(
        r#"
        INSERT INTO etablissements.etablissement_module
            (id, tenant_id, etablissement_id, module_code, module_implemente)
        VALUES ($1, $2, $3, 'HEBERGEMENT', true) ON CONFLICT DO NOTHING
        "#,
    )
    .bind(Uuid::now_v7())
    .bind(jeu.tenant_id)
    .bind(jeu.etablissement_id)
    .execute(&mut *tx)
    .await
    .expect("module");

    let categorie_id = Uuid::now_v7();
    sqlx::query(
        r#"
        INSERT INTO hebergement.categorie (id, tenant_id, etablissement_id, nom, capacite_accueil)
        VALUES ($1, $2, $3, 'Standard', 2)
        "#,
    )
    .bind(categorie_id)
    .bind(jeu.tenant_id)
    .bind(jeu.etablissement_id)
    .execute(&mut *tx)
    .await
    .expect("catégorie");

    let mut unites = Vec::new();
    for code in ["A1", "A2"] {
        let id = Uuid::now_v7();
        sqlx::query(
            r#"
            INSERT INTO hebergement.unite
                (id, tenant_id, etablissement_id, categorie_id, code, etage)
            VALUES ($1, $2, $3, $4, $5, 1)
            "#,
        )
        .bind(id)
        .bind(jeu.tenant_id)
        .bind(jeu.etablissement_id)
        .bind(categorie_id)
        .bind(code)
        .execute(&mut *tx)
        .await
        .expect("unité");
        unites.push(id);
    }

    let formule_id = Uuid::now_v7();
    sqlx::query(
        r#"
        INSERT INTO hebergement.formule
            (id, tenant_id, etablissement_id, categorie_id, famille, prix_mineur,
             assujettie_taxe_nuitee)
        VALUES ($1, $2, $3, $4, 'NUITEE', 12500, false)
        "#,
    )
    .bind(formule_id)
    .bind(jeu.tenant_id)
    .bind(jeu.etablissement_id)
    .bind(categorie_id)
    .execute(&mut *tx)
    .await
    .expect("formule");

    tx.commit().await.expect("commit");

    Decor {
        jeu,
        unite_id: unites[0],
        unite_bis_id: unites[1],
        formule_id,
    }
}

macro_rules! ouvrir {
    ($app:expr, $bearer:expr, $decor:expr, $id:expr, $unite:expr, $debut:expr, $heures:expr) => {{
        let requete = actix_web::test::TestRequest::post()
            .uri(&format!(
                "/api/v1/etablissements/{}/sejours",
                $decor.jeu.etablissement_id
            ))
            .insert_header(("authorization", $bearer.to_owned()))
            .set_json(serde_json::json!({
                "id": $id,
                "unite_id": $unite,
                "formule_id": $decor.formule_id,
                "debut_client": $debut
                    .format(&time::format_description::well_known::Rfc3339).unwrap(),
                "fin_client": ($debut + time::Duration::hours($heures))
                    .format(&time::format_description::well_known::Rfc3339).unwrap(),
            }))
            .to_request();
        actix_web::test::call_service(&$app, requete).await
    }};
}

macro_rules! prolonger {
    ($app:expr, $bearer:expr, $decor:expr, $sejour:expr, $fin:expr) => {{
        let requete = actix_web::test::TestRequest::post()
            .uri(&format!(
                "/api/v1/etablissements/{}/sejours/{}/prolongation",
                $decor.jeu.etablissement_id, $sejour
            ))
            .insert_header(("authorization", $bearer.to_owned()))
            .set_json(serde_json::json!({
                "id": Uuid::now_v7(),
                "nouvelle_fin_client": $fin
                    .format(&time::format_description::well_known::Rfc3339).unwrap(),
            }))
            .to_request();
        actix_web::test::call_service(&$app, requete).await
    }};
}

macro_rules! changer_unite {
    ($app:expr, $bearer:expr, $decor:expr, $sejour:expr, $cible:expr) => {{
        let requete = actix_web::test::TestRequest::post()
            .uri(&format!(
                "/api/v1/etablissements/{}/sejours/{}/changement-unite",
                $decor.jeu.etablissement_id, $sejour
            ))
            .insert_header(("authorization", $bearer.to_owned()))
            .set_json(serde_json::json!({ "id": Uuid::now_v7(), "unite_cible_id": $cible }))
            .to_request();
        actix_web::test::call_service(&$app, requete).await
    }};
}

// =================================================================================================
//  US5 — la prolongation
// =================================================================================================

/// **Un intervalle étendu libre → le séjour est prolongé, avec sa ligne au tarif en vigueur.**
#[actix_web::test]
async fn un_intervalle_etendu_libre_prolonge_le_sejour() {
    let owner = pool_owner().await;
    let decor = poser_decor(&owner, "prolongation — libre").await;
    let cx = commun::compte_connecte(
        &owner,
        decor.jeu,
        "Adjoua",
        &[(ROLE, Some(decor.jeu.etablissement_id))],
    )
    .await;
    let app = monter_application!(pool_app().await);

    let sejour_id = Uuid::now_v7();
    let debut = time::OffsetDateTime::now_utc() + time::Duration::hours(1);
    assert_eq!(
        ouvrir!(app, cx.bearer, decor, sejour_id, decor.unite_id, debut, 24).status(),
        201
    );

    let avant: serde_json::Value = {
        let requete = actix_web::test::TestRequest::get()
            .uri(&format!(
                "/api/v1/etablissements/{}/sejours/{sejour_id}",
                decor.jeu.etablissement_id
            ))
            .insert_header(("authorization", cx.bearer.clone()))
            .to_request();
        actix_web::test::read_body_json(actix_web::test::call_service(&app, requete).await).await
    };

    let nouvelle_fin = debut + time::Duration::hours(48);
    let reponse = prolonger!(app, cx.bearer, decor, sejour_id, nouvelle_fin);
    let statut = reponse.status().as_u16();
    let corps: serde_json::Value = actix_web::test::read_body_json(reponse).await;

    assert_eq!(statut, 200, "corps : {corps}");
    assert!(
        corps["note"]["total_mineur"].as_i64().unwrap()
            > avant["note"]["total_mineur"].as_i64().unwrap(),
        "la prolongation doit ajouter une ligne au tarif en vigueur : le total ne bouge pas. \
         Avant : {avant}\nAprès : {corps}"
    );

    // La ligne porte son **motif**, et la ligne initiale est **intacte**.
    let lignes = corps["note"]["lignes"].as_array().expect("lignes");
    assert_eq!(lignes.len(), 2, "une ligne d'ajustement doit s'ajouter : {corps}");
    assert_eq!(lignes[0]["motif"], serde_json::Value::Null, "la ligne initiale est INTACTE");
    assert_eq!(lignes[1]["motif"], "prolongation");
}

/// ★ **Le refus NOMME son conflit, et propose des alternatives** (FR-070, FR-071).
///
/// Sans l'instant, Adjoua ne peut rien dire au client. Sans les alternatives, elle doit ouvrir un
/// autre écran pendant que le client attend devant elle.
#[actix_web::test]
async fn une_occupation_suivante_produit_un_conflit_nomme_avec_ses_alternatives() {
    let owner = pool_owner().await;
    let decor = poser_decor(&owner, "prolongation — conflit").await;
    let cx = commun::compte_connecte(
        &owner,
        decor.jeu,
        "Adjoua",
        &[(ROLE, Some(decor.jeu.etablissement_id))],
    )
    .await;
    let app = monter_application!(pool_app().await);

    let debut = time::OffsetDateTime::now_utc() + time::Duration::hours(1);

    // Le séjour à prolonger, sur A1.
    let sejour_id = Uuid::now_v7();
    assert_eq!(
        ouvrir!(app, cx.bearer, decor, sejour_id, decor.unite_id, debut, 24).status(),
        201
    );

    // ★ Une réservation **sur la même chambre**, juste après — c'est elle qui bloque.
    let suivant_debut = debut + time::Duration::hours(27);
    assert_eq!(
        ouvrir!(app, cx.bearer, decor, Uuid::now_v7(), decor.unite_id, suivant_debut, 24).status(),
        201
    );

    let reponse = prolonger!(app, cx.bearer, decor, sejour_id, debut + time::Duration::hours(48));
    let statut = reponse.status().as_u16();
    let conflit: serde_json::Value = actix_web::test::read_body_json(reponse).await;

    assert_eq!(statut, 409, "corps : {conflit}");
    assert_eq!(conflit["unite_id"], decor.unite_id.to_string());

    assert!(
        !conflit["debut_occupation_suivante"].is_null(),
        "★ le refus doit NOMMER l'instant du conflit (FR-070). Sans lui, Adjoua ne peut rien dire \
         au client — et un refus qu'on ne peut pas expliquer est un refus qu'on contourne. \
         Corps : {conflit}"
    );

    let alternatives = conflit["unites_alternatives"].as_array().expect("alternatives");
    assert!(
        alternatives.iter().any(|a| a["unite_id"] == decor.unite_bis_id.to_string()),
        "★ le refus doit proposer les chambres LIBRES de la même catégorie (FR-071). La A2 l'est. \
         Sans elles, Adjoua doit ouvrir un autre écran pendant que le client attend devant elle. \
         Corps : {conflit}"
    );
    assert!(
        alternatives.iter().all(|a| a["unite_id"] != decor.unite_id.to_string()),
        "la chambre en conflit ne peut pas être sa propre alternative : {conflit}"
    );
}

/// **On ne prolonge pas un séjour terminé** — et la phrase dit la RÈGLE, pas l'état.
#[actix_web::test]
async fn un_sejour_clos_ne_se_prolonge_pas() {
    let owner = pool_owner().await;
    let decor = poser_decor(&owner, "prolongation — séjour clos").await;
    let cx = commun::compte_connecte(
        &owner,
        decor.jeu,
        "Adjoua",
        &[(ROLE, Some(decor.jeu.etablissement_id))],
    )
    .await;
    let app = monter_application!(pool_app().await);

    let sejour_id = Uuid::now_v7();
    let debut = time::OffsetDateTime::now_utc() - time::Duration::hours(2);
    assert_eq!(
        ouvrir!(app, cx.bearer, decor, sejour_id, decor.unite_id, debut, 24).status(),
        201
    );

    let requete = actix_web::test::TestRequest::post()
        .uri(&format!(
            "/api/v1/etablissements/{}/sejours/{sejour_id}/depart",
            decor.jeu.etablissement_id
        ))
        .insert_header(("authorization", cx.bearer.clone()))
        .to_request();
    assert_eq!(actix_web::test::call_service(&app, requete).await.status(), 200);

    let reponse = prolonger!(app, cx.bearer, decor, sejour_id, debut + time::Duration::hours(48));
    let statut = reponse.status().as_u16();
    let corps: serde_json::Value = actix_web::test::read_body_json(reponse).await;

    assert_eq!(statut, 409, "corps : {corps}");
    assert_eq!(
        corps["code"], "sejour_clos",
        "le code est `sejour_clos` et non `sejour_deja_clos` : la phrase dit la RÈGLE — « on ne \
         prolonge pas un séjour terminé » —, pas l'état. C'est ce qui évite qu'Adjoua cherche \
         comment « rouvrir » le séjour. Corps : {corps}"
    );
}

// =================================================================================================
//  US7 — le changement de chambre
// =================================================================================================

/// ★ **Deux occupations, UN séjour** — et l'historique conserve les deux.
#[actix_web::test]
async fn un_changement_d_unite_produit_deux_occupations_sur_un_seul_sejour() {
    let owner = pool_owner().await;
    let decor = poser_decor(&owner, "changement — nominal").await;
    let cx = commun::compte_connecte(
        &owner,
        decor.jeu,
        "Adjoua",
        &[(ROLE, Some(decor.jeu.etablissement_id))],
    )
    .await;
    let app = monter_application!(pool_app().await);

    let sejour_id = Uuid::now_v7();
    let debut = time::OffsetDateTime::now_utc() - time::Duration::hours(1);
    assert_eq!(
        ouvrir!(app, cx.bearer, decor, sejour_id, decor.unite_id, debut, 48).status(),
        201
    );

    let reponse = changer_unite!(app, cx.bearer, decor, sejour_id, decor.unite_bis_id);
    let statut = reponse.status().as_u16();
    let corps: serde_json::Value = actix_web::test::read_body_json(reponse).await;
    assert_eq!(statut, 200, "corps : {corps}");

    let mut tx = owner.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, decor.jeu.tenant_id)
        .await
        .expect("pose du tenant");

    let occupations: Vec<(Uuid, String)> = sqlx::query_as(
        r#"
        SELECT unite_id, statut FROM hebergement.occupation
        WHERE sejour_id = $1 ORDER BY cree_le
        "#,
    )
    .bind(sejour_id)
    .fetch_all(&mut *tx)
    .await
    .expect("lecture des occupations");

    let sejours: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM hebergement.sejour WHERE id = $1")
        .bind(sejour_id)
        .fetch_one(&mut *tx)
        .await
        .expect("comptage");
    tx.rollback().await.expect("rollback");

    assert_eq!(
        occupations.len(),
        2,
        "★ deux occupations doivent exister — c'est ce qui garde l'HISTOIRE du séjour : quelle \
         chambre, sur quelle période. Obtenu : {occupations:?}"
    );
    assert_eq!(sejours, 1, "un SEUL séjour : le changement ne le duplique pas");
    assert_eq!(occupations[0].0, decor.unite_id);
    assert_eq!(occupations[1].0, decor.unite_bis_id);
    assert_eq!(
        occupations[0].1, "liberee",
        "l'occupation d'origine est LIBÉRÉE, pas supprimée : une chambre occupée reste une chambre \
         occupée dans l'histoire"
    );

    // Chaque période porte **son tarif propre** — une ligne d'ajustement l'atteste.
    let lignes = corps["note"]["lignes"].as_array().expect("lignes");
    assert_eq!(lignes.len(), 2);
    assert_eq!(lignes[1]["motif"], "changement_unite");
}

/// ★ **Une chambre cible occupée est refusée SANS déplacement partiel.**
///
/// C'est la propriété que la transaction unique achète : le client ne se retrouve jamais
/// « nulle part », avec une chambre libérée et aucune autre attribuée.
#[actix_web::test]
async fn une_unite_cible_occupee_est_refusee_sans_deplacement_partiel() {
    let owner = pool_owner().await;
    let decor = poser_decor(&owner, "changement — cible occupée").await;
    let cx = commun::compte_connecte(
        &owner,
        decor.jeu,
        "Adjoua",
        &[(ROLE, Some(decor.jeu.etablissement_id))],
    )
    .await;
    let app = monter_application!(pool_app().await);

    let debut = time::OffsetDateTime::now_utc() - time::Duration::hours(1);

    let sejour_id = Uuid::now_v7();
    assert_eq!(
        ouvrir!(app, cx.bearer, decor, sejour_id, decor.unite_id, debut, 48).status(),
        201
    );
    // La chambre cible est prise par quelqu'un d'autre.
    assert_eq!(
        ouvrir!(app, cx.bearer, decor, Uuid::now_v7(), decor.unite_bis_id, debut, 48).status(),
        201
    );

    let reponse = changer_unite!(app, cx.bearer, decor, sejour_id, decor.unite_bis_id);
    let statut = reponse.status().as_u16();
    let corps: serde_json::Value = actix_web::test::read_body_json(reponse).await;

    assert_eq!(statut, 409, "corps : {corps}");
    assert_eq!(
        corps["code"], "unite_cible_occupee",
        "le code distingue ce refus d'`unite_deja_occupee` : celui-ci porte sur la période \
         RESTANTE d'un séjour en cours, celui-là sur une période demandée. Adjoua explique le \
         premier au client installé, le second au client qui arrive. Corps : {corps}"
    );

    // ★ **AUCUN déplacement partiel** : l'occupation d'origine est toujours active.
    let mut tx = owner.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, decor.jeu.tenant_id)
        .await
        .expect("pose du tenant");
    let occupations: Vec<(Uuid, String)> = sqlx::query_as(
        r#"SELECT unite_id, statut FROM hebergement.occupation WHERE sejour_id = $1"#,
    )
    .bind(sejour_id)
    .fetch_all(&mut *tx)
    .await
    .expect("lecture");
    tx.rollback().await.expect("rollback");

    assert_eq!(
        occupations.len(),
        1,
        "★ un DÉPLACEMENT PARTIEL a été produit : le séjour porte {} occupation(s). La clôture de \
         l'origine et l'ouverture de la cible vivent dans la MÊME transaction — un échec sur la \
         seconde doit annuler la première, sans quoi le client se retrouve « nulle part ».",
        occupations.len()
    );
    assert_eq!(
        occupations[0].1, "active",
        "l'occupation d'origine doit rester ACTIVE après un refus : le client est toujours dans \
         sa chambre"
    );
}

/// ★ **P-09 : la contrainte SE DÉCLENCHERAIT si les unités étaient les mêmes.**
///
/// ⚠️ **C'est l'assertion que ce fichier doit à la porte.** Deux occupations contiguës sur deux
/// unités **différentes** ne déclenchent pas la contrainte — elle porte sur `(unite_id, periode)`.
/// Un changement d'unité vert ne dit donc **rien** de la garantie.
///
/// Ce test prouve le contraire par l'absurde : la **même** période sur la **même** unité est
/// refusée, et le refus vient de la contrainte nommée.
#[actix_web::test]
async fn p09_deux_occupations_sur_la_meme_unite_restent_impossibles() {
    let owner = pool_owner().await;
    let decor = poser_decor(&owner, "changement — P-09 ré-exercée").await;
    let cx = commun::compte_connecte(
        &owner,
        decor.jeu,
        "Adjoua",
        &[(ROLE, Some(decor.jeu.etablissement_id))],
    )
    .await;
    let app = monter_application!(pool_app().await);

    let debut = time::OffsetDateTime::now_utc() + time::Duration::hours(1);

    assert_eq!(
        ouvrir!(app, cx.bearer, decor, Uuid::now_v7(), decor.unite_id, debut, 24).status(),
        201
    );

    // ── (a) Deux unités DIFFÉRENTES, même période → aucune violation ─────────────────────────
    assert_eq!(
        ouvrir!(app, cx.bearer, decor, Uuid::now_v7(), decor.unite_bis_id, debut, 24).status(),
        201,
        "deux unités différentes sur la même période ne se gênent pas : la contrainte porte sur \
         (unite_id, periode). C'est pourquoi le test (b) est nécessaire."
    );

    // ── (b) La MÊME unité, période chevauchante → la contrainte se déclenche ────────────────
    let reponse = ouvrir!(
        app,
        cx.bearer,
        decor,
        Uuid::now_v7(),
        decor.unite_id,
        debut + time::Duration::hours(2),
        24
    );
    let statut = reponse.status().as_u16();
    let corps: serde_json::Value = actix_web::test::read_body_json(reponse).await;

    assert_eq!(statut, 409, "corps : {corps}");
    assert_eq!(
        corps["code"], "unite_deja_occupee",
        "★ la contrainte d'exclusion doit TOUJOURS se déclencher sur une même unité. Sans cette \
         assertion, un changement d'unité vert ne dirait rien de la garantie — deux occupations \
         contiguës sur deux unités différentes ne la déclenchent pas, et c'est normal. \
         Corps : {corps}"
    );
}
