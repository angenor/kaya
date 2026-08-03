//! ★ **LE SCÉNARIO ORPHELIN — sa première cible en cinq cycles.**
//!
//! ═══════════════════════════════════════════════════════════════════════════════════════════════
//!  CE QUE LE §0.7 EXIGE DEPUIS LE CYCLE 001, ET QU'AUCUN CYCLE NE POUVAIT SATISFAIRE
//!
//!      « toute entité rattachée à un séjour : test du scénario orphelin (SYN-03) »
//!
//!  Aucune entité n'était rattachée à un séjour jusqu'ici, faute de séjour. Trois le sont
//!  désormais, et **une seule peut réellement produire un orphelin à ce cycle** :
//!
//!  | Entité | Peut arriver après la clôture ? | Pourquoi |
//!  |---|---|---|
//!  | `accompagnant` | **OUI** | Classe **A** — écrit hors ligne, mis en file, vidé au retour du réseau |
//!  | `preference_personne` | Non | Rattachée au **client**, pas au séjour. Un client n'est jamais clos |
//!  | `ligne_sejour` | Pas encore | Classe **B** — jamais écrite hors ligne au MVP. Le cas du cadrage §11.4 arrive avec **PDV, tranche T2** |
//! ═══════════════════════════════════════════════════════════════════════════════════════════════
//!
//! # ★ Pourquoi `202`, et pourquoi ni `201` ni `409`
//!
//! Le principe VI interdit **les deux** réponses évidentes :
//!
//! | Réponse | Ce qu'elle ferait | Pourquoi c'est refusé |
//! |---|---|---|
//! | `201` | Ajouter l'accompagnant au séjour clos | **Ajout d'office** — la note est arrêtée, la fiche de police émise, le constat de taxe figé. Ajouter une personne après coup rendrait faux un document légal déjà produit |
//! | `409` | Rejeter l'écriture | **Rejet silencieux** — Adjoua a bien saisi cette personne, hors ligne, avant le départ. Perdre sa saisie sans trace est ce que le cahier papier ne faisait pas |
//!
//! `202` dit ce qui est vrai : **l'écriture est conservée, elle n'est pas sur le séjour, et
//! quelqu'un devra trancher.** Terme utilisateur : « Cette information est arrivée après le départ
//! du client. » suivie de « Le gérant décidera de la suite. » — la seconde phrase est obligatoire,
//! sans elle Adjoua ne sait pas si son geste a compté.
//!
//! # ⚠️ La résolution n'est PAS ici
//!
//! **SYN-03, tranche T3.** Ce cycle **alimente** la file ; il ne la vide pas. Le privilège le dit
//! avant le code : `synchronisation.reconciliation_orpheline` reçoit `INSERT`, jamais `UPDATE`.

mod commun;

use uuid::Uuid;

use commun::{JeuTenant, creer_tenant, pool_app, pool_owner};

/// Le rôle du comptoir — celui qui porte `heb.sejour.ouvrir`.
const ROLE: &str = "receptionniste";

// =================================================================================================
//  Décor
// =================================================================================================

struct Decor {
    jeu: JeuTenant,
    sejour_id: Uuid,
}

/// Un établissement, un séjour **ouvert**, et rien d'autre.
///
/// Le séjour est posé par SQL direct : ce fichier mesure le **cas orphelin**, pas l'ouverture, qui
/// a ses propres tests. Y passer par l'endpoint ferait dépendre quatre assertions du bon
/// fonctionnement d'une autre opération.
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

    let sejour_id = Uuid::now_v7();
    sqlx::query(
        r#"
        INSERT INTO hebergement.sejour (id, tenant_id, etablissement_id)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(sejour_id)
    .bind(jeu.tenant_id)
    .bind(jeu.etablissement_id)
    .execute(&mut *tx)
    .await
    .expect("séjour");

    tx.commit().await.expect("commit");

    Decor { jeu, sejour_id }
}

/// Clôt le séjour — par SQL direct : le départ complet est l'objet de `sejour_depart.rs`.
async fn clore(pool: &sqlx::PgPool, decor: &Decor) {
    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, decor.jeu.tenant_id)
        .await
        .expect("pose du tenant");
    sqlx::query(
        r#"UPDATE hebergement.sejour SET statut = 'clos', clos_le = now() WHERE id = $1"#,
    )
    .bind(decor.sejour_id)
    .execute(&mut *tx)
    .await
    .expect("clôture");
    tx.commit().await.expect("commit");
}

/// Ajoute un accompagnant **par le chemin réel**.
macro_rules! ajouter {
    ($app:expr, $bearer:expr, $decor:expr, $id:expr, $nom:expr) => {{
        let requete = actix_web::test::TestRequest::post()
            .uri(&format!(
                "/api/v1/etablissements/{}/sejours/{}/accompagnants",
                $decor.jeu.etablissement_id, $decor.sejour_id
            ))
            .insert_header(("authorization", $bearer.to_owned()))
            .set_json(serde_json::json!({ "id": $id, "nom": $nom }))
            .to_request();
        actix_web::test::call_service(&$app, requete).await
    }};
}

// =================================================================================================
//  Les quatre assertions
// =================================================================================================

/// **(1) Vidé AVANT la clôture → `201`, ajout normal.**
///
/// C'est le cas majoritaire, et il doit rester banal : une file qui part au bon moment n'a rien
/// d'exceptionnel. Sans cette assertion, les trois suivantes pourraient passer sur un produit qui
/// refuserait **tous** les ajouts.
#[actix_web::test]
async fn un_accompagnant_vide_avant_la_cloture_est_un_ajout_normal() {
    let owner = pool_owner().await;
    let decor = poser_decor(&owner, "orphelin — avant clôture").await;
    let cx = commun::compte_connecte(
        &owner,
        decor.jeu,
        "Adjoua",
        &[(ROLE, Some(decor.jeu.etablissement_id))],
    )
    .await;
    let app = monter_application!(pool_app().await);

    let reponse = ajouter!(app, cx.bearer, decor, Uuid::now_v7(), "Aïcha");

    assert_eq!(
        reponse.status(),
        201,
        "un accompagnant ajouté à un séjour OUVERT est un ajout normal — 201, sans détour par la \
         file de réconciliation"
    );
}

/// ★ **(2) Vidé APRÈS la clôture → `202`, ni `201` ni `409`.**
///
/// C'est l'assertion centrale du fichier. Les deux réponses évidentes sont **toutes deux
/// interdites** par le principe VI :
///
/// - `201` serait un **ajout d'office** sur un séjour dont la note est arrêtée, la fiche de police
///   émise et le constat de taxe figé ;
/// - `409` serait un **rejet silencieux** de la saisie d'Adjoua, qui a bien vu cette personne.
#[actix_web::test]
async fn un_accompagnant_vide_apres_la_cloture_part_en_reconciliation() {
    let owner = pool_owner().await;
    let decor = poser_decor(&owner, "orphelin — après clôture").await;
    let cx = commun::compte_connecte(
        &owner,
        decor.jeu,
        "Adjoua",
        &[(ROLE, Some(decor.jeu.etablissement_id))],
    )
    .await;
    let app = monter_application!(pool_app().await);

    clore(&owner, &decor).await;

    let reponse = ajouter!(app, cx.bearer, decor, Uuid::now_v7(), "Aïcha");
    let statut = reponse.status().as_u16();
    let corps: serde_json::Value = actix_web::test::read_body_json(reponse).await;

    assert_eq!(
        statut, 202,
        "★ un accompagnant arrivé APRÈS la clôture doit rendre 202.\n\n\
         · 201 serait un AJOUT D'OFFICE sur un séjour dont la note est arrêtée, la fiche de \
         police émise et le constat de taxe figé ;\n\
         · 409 serait un REJET SILENCIEUX de la saisie d'Adjoua, qui a bien vu cette personne.\n\n\
         Le principe VI interdit les deux. Corps reçu : {corps}"
    );

    assert_eq!(
        corps["motif"], "sejour_clos",
        "le motif est un CODE STABLE, traduit par le lexique — jamais un message de diagnostic"
    );
    assert!(
        corps["reconciliation_id"].is_string(),
        "le corps doit porter l'identifiant de la ligne de réconciliation : sans lui, Adjoua ne \
         peut rien montrer au gérant. Corps : {corps}"
    );
}

/// ★ **(3) La ligne existe, avec le séjour, l'entité, LA CHARGE UTILE et le motif.**
///
/// ⚠️ **La charge utile est ce qui fait toute la valeur de la file**, et c'est le défaut que la
/// migration `0034` a corrigé : le séjour étant clos, la ligne `hebergement.accompagnant` **n'est
/// pas écrite**. Sans charge utile, la file ne retiendrait que des identifiants et SYN-03
/// n'aurait **rien à rattacher** — un écran de réconciliation affichant des lignes vides, deux
/// cycles plus tard, et une équipe qui conclut que la file « ne marche pas ».
#[actix_web::test]
async fn la_ligne_de_reconciliation_retient_le_nom_de_la_personne() {
    let owner = pool_owner().await;
    let decor = poser_decor(&owner, "orphelin — charge utile").await;
    let cx = commun::compte_connecte(
        &owner,
        decor.jeu,
        "Adjoua",
        &[(ROLE, Some(decor.jeu.etablissement_id))],
    )
    .await;
    let app = monter_application!(pool_app().await);

    clore(&owner, &decor).await;

    let accompagnant_id = Uuid::now_v7();
    let reponse = ajouter!(app, cx.bearer, decor, accompagnant_id, "Aïcha N'Guessan");
    assert_eq!(reponse.status(), 202);

    let mut tx = owner.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, decor.jeu.tenant_id)
        .await
        .expect("pose du tenant");

    let ligne = sqlx::query!(
        r#"
        SELECT ecriture_id, ecriture_type, agregat_type, agregat_id, etat,
               charge_utile, motif
        FROM synchronisation.reconciliation_orpheline
        WHERE ecriture_id = $1
        "#,
        accompagnant_id
    )
    .fetch_optional(&mut *tx)
    .await
    .expect("lecture de la file");

    // ★ **Et l'accompagnant n'est PAS en base** — c'est ce qui rend la charge utile nécessaire.
    let ecrit: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM hebergement.accompagnant WHERE id = $1",
    )
    .bind(accompagnant_id)
    .fetch_one(&mut *tx)
    .await
    .expect("comptage");
    tx.rollback().await.expect("rollback");

    let ligne = ligne.expect("une ligne de réconciliation doit exister");

    assert_eq!(ligne.ecriture_type, "accompagnant");
    assert_eq!(ligne.agregat_type, "sejour");
    assert_eq!(ligne.agregat_id, decor.sejour_id);
    assert_eq!(ligne.motif.as_deref(), Some("sejour_clos"));
    assert_eq!(
        ligne.etat, "constatee",
        "la ligne naît « constatee » : sa RÉSOLUTION est SYN-03, tranche T3. Ce cycle alimente \
         la file, il ne la vide pas — et le privilège le dit avant le code, UPDATE n'étant pas \
         accordé."
    );

    assert_eq!(
        ecrit, 0,
        "l'accompagnant NE DOIT PAS être en base : le séjour est clos, l'ajout est refusé comme \
         ajout. C'est exactement pourquoi la charge utile est nécessaire."
    );

    let charge = ligne.charge_utile.expect(
        "★ la charge utile est ABSENTE. Le séjour étant clos, la ligne `accompagnant` n'est pas \
         écrite : sans charge utile, le NOM de la personne est perdu et SYN-03 n'aura rien à \
         rattacher. C'est le défaut que la migration 0034 corrige.",
    );
    assert_eq!(
        charge["nom"], "Aïcha N'Guessan",
        "la charge utile doit retenir le NOM — c'est tout ce que SYN-03 aura pour trancher. \
         Charge : {charge}"
    );
}

/// ★ **(4) Le séjour clos est INCHANGÉ.**
///
/// Ni accompagnant ajouté, ni statut modifié, ni date de clôture repoussée.
///
/// ⚠️ **C'est asserté bien que le privilège le garantisse en partie**, et la raison est écrite au
/// modèle de données : *une garantie de privilège se perd en une ligne de migration*. Un
/// `GRANT UPDATE` ajouté un jour « pour débloquer un correctif » rendrait ce test rouge — ce qui
/// est exactement ce qu'on veut.
#[actix_web::test]
async fn le_sejour_clos_reste_inchange() {
    let owner = pool_owner().await;
    let decor = poser_decor(&owner, "orphelin — séjour intact").await;
    let cx = commun::compte_connecte(
        &owner,
        decor.jeu,
        "Adjoua",
        &[(ROLE, Some(decor.jeu.etablissement_id))],
    )
    .await;
    let app = monter_application!(pool_app().await);

    clore(&owner, &decor).await;

    // L'état du séjour **avant** l'écriture orpheline.
    let avant = lire_sejour(&owner, &decor).await;

    assert_eq!(ajouter!(app, cx.bearer, decor, Uuid::now_v7(), "Aïcha").status(), 202);
    assert_eq!(ajouter!(app, cx.bearer, decor, Uuid::now_v7(), "Koffi").status(), 202);

    let apres = lire_sejour(&owner, &decor).await;

    assert_eq!(apres, avant, "le séjour clos a changé après une écriture orpheline");

    let mut tx = owner.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, decor.jeu.tenant_id)
        .await
        .expect("pose du tenant");
    let accompagnants: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM hebergement.accompagnant WHERE sejour_id = $1",
    )
    .bind(decor.sejour_id)
    .fetch_one(&mut *tx)
    .await
    .expect("comptage");
    tx.rollback().await.expect("rollback");

    assert_eq!(
        accompagnants, 0,
        "deux écritures orphelines ont ajouté {accompagnants} accompagnant(s) au séjour clos. \
         Le nombre de personnes du constat de taxe est FIGÉ : l'y ajouter après coup rendrait \
         faux un document déjà produit."
    );
}

/// L'état observable d'un séjour — statut et instant de clôture.
async fn lire_sejour(pool: &sqlx::PgPool, decor: &Decor) -> (String, Option<String>) {
    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, decor.jeu.tenant_id)
        .await
        .expect("pose du tenant");
    let ligne = sqlx::query!(
        r#"SELECT statut, clos_le FROM hebergement.sejour WHERE id = $1"#,
        decor.sejour_id
    )
    .fetch_one(&mut *tx)
    .await
    .expect("lecture du séjour");
    tx.rollback().await.expect("rollback");

    (ligne.statut, ligne.clos_le.map(|d| d.to_string()))
}

/// **Le rejeu d'une écriture orpheline ne crée qu'UNE ligne de réconciliation.**
///
/// Un terminal qui vide sa file après une coupure renvoie la même écriture. Sans idempotence, le
/// gérant verrait trois fois la même personne à trancher — et un écran de réconciliation qui
/// répète est un écran qu'on cesse de lire.
#[actix_web::test]
async fn le_rejeu_d_une_ecriture_orpheline_ne_cree_qu_une_ligne() {
    let owner = pool_owner().await;
    let decor = poser_decor(&owner, "orphelin — rejeu").await;
    let cx = commun::compte_connecte(
        &owner,
        decor.jeu,
        "Adjoua",
        &[(ROLE, Some(decor.jeu.etablissement_id))],
    )
    .await;
    let app = monter_application!(pool_app().await);

    clore(&owner, &decor).await;

    let accompagnant_id = Uuid::now_v7();
    for _ in 0..3 {
        assert_eq!(
            ajouter!(app, cx.bearer, decor, accompagnant_id, "Aïcha").status(),
            202,
            "chaque rejeu rend 202 : le terminal ne doit pas voir d'erreur pour une écriture déjà \
             prise en compte"
        );
    }

    let mut tx = owner.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, decor.jeu.tenant_id)
        .await
        .expect("pose du tenant");
    let lignes: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM synchronisation.reconciliation_orpheline WHERE ecriture_id = $1",
    )
    .bind(accompagnant_id)
    .fetch_one(&mut *tx)
    .await
    .expect("comptage");
    tx.rollback().await.expect("rollback");

    assert_eq!(
        lignes, 1,
        "trois envois ont produit {lignes} ligne(s) de réconciliation. Le gérant verrait trois \
         fois la même personne à trancher, et un écran qui répète est un écran qu'on cesse de lire."
    );
}
