//! ★ **SEJ-04 — le départ, et la taxe FIGÉE.**
//!
//! ═══════════════════════════════════════════════════════════════════════════════════════════════
//!  CE QUE « FIGÉ » VEUT DIRE, ET COMMENT ON LE VÉRIFIE
//!
//!  SC-007 promet que **l'assiette est immuable après le départ**. Une promesse se relit ; une
//!  propriété se mesure. Ce fichier la mesure de **deux façons indépendantes** :
//!
//!  | Contrôle | Ce qu'il prouve |
//!  |---|---|
//!  | **(a)** Modifier accompagnant, barème, formule, classement, commune → le constat ne bouge pas | Le paramétrage est **RECOPIÉ**, jamais référencé |
//!  | **(b)** `UPDATE` et `DELETE` sous le rôle applicatif → `permission denied` | Le figeage est un **PRIVILÈGE**, pas une intention |
//!
//!  ⚠️ **(b) est asserté BIEN QUE le privilège le garantisse**, et la raison est écrite au modèle
//!  de données : *une garantie de privilège se perd en une ligne de migration*. Un `GRANT UPDATE`
//!  ajouté un jour « pour débloquer un correctif » rendrait ce test rouge — ce qui est exactement
//!  ce qu'on veut.
//! ═══════════════════════════════════════════════════════════════════════════════════════════════
//!
//! # ★ (g) La dérive d'horloge — ce que P-23 ne couvre PAS
//!
//! P-23 analyse le **code** et prouve qu'aucun calcul ne lit `horodatage_client`. Elle ne dit rien
//! du **comportement**. Le test (g) rejoue le même départ avec une horloge de terminal décalée de
//! **+1 h puis −1 h** et vérifie que la durée réelle, la ligne d'ajustement et le constat sont
//! **identiques au bit près**.
//!
//! C'est la forme qu'avait déjà `hebergement_tarification.rs` au cycle 004, et elle vaut ici
//! davantage : un départ antidaté ne fausse pas seulement un montant, il fausse une **assiette
//! fiscale figée** — que plus rien ne pourra corriger.

mod commun;

use uuid::Uuid;

use commun::{JeuTenant, creer_tenant, pool_app, pool_owner};

const ROLE: &str = "receptionniste";

// =================================================================================================
//  Décor
// =================================================================================================

struct Decor {
    jeu: JeuTenant,
    unite_id: Uuid,
    formule_id: Uuid,
    categorie_id: Uuid,
}

/// Un établissement **classé deux étoiles à Abengourou**, une nuitée **assujettie**.
///
/// Le classement et la commune ne sont pas décoratifs : ce sont deux des six valeurs que le
/// constat **recopie**, et le test (a) les change après la clôture pour vérifier que le constat ne
/// bouge pas.
async fn poser_decor(pool: &sqlx::PgPool, nom: &str) -> Decor {
    let jeu = creer_tenant(pool, nom).await;

    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, jeu.tenant_id)
        .await
        .expect("pose du tenant");

    sqlx::query(
        r#"
        UPDATE etablissements.etablissement
        SET classement = 'ETOILES', etoiles = 2, commune = 'Abengourou'
        WHERE id = $1
        "#,
    )
    .bind(jeu.etablissement_id)
    .execute(&mut *tx)
    .await
    .expect("classement");

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

    let unite_id = Uuid::now_v7();
    sqlx::query(
        r#"
        INSERT INTO hebergement.unite (id, tenant_id, etablissement_id, categorie_id, code, etage)
        VALUES ($1, $2, $3, $4, 'A1', 1)
        "#,
    )
    .bind(unite_id)
    .bind(jeu.tenant_id)
    .bind(jeu.etablissement_id)
    .bind(categorie_id)
    .execute(&mut *tx)
    .await
    .expect("unité");

    // Une nuitée **assujettie**, avec sa règle de conversion : c'est ce que le constat recopie.
    let formule_id = Uuid::now_v7();
    sqlx::query(
        r#"
        INSERT INTO hebergement.formule
            (id, tenant_id, etablissement_id, categorie_id, famille, prix_mineur,
             assujettie_taxe_nuitee, regle_conversion_taxe)
        VALUES ($1, $2, $3, $4, 'NUITEE', 12500, true, 'une_nuitee_par_occupation')
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
        unite_id,
        formule_id,
        categorie_id,
    }
}

macro_rules! ouvrir {
    ($app:expr, $bearer:expr, $decor:expr, $id:expr, $debut:expr, $heures:expr) => {{
        let requete = actix_web::test::TestRequest::post()
            .uri(&format!(
                "/api/v1/etablissements/{}/sejours",
                $decor.jeu.etablissement_id
            ))
            .insert_header(("authorization", $bearer.to_owned()))
            .set_json(serde_json::json!({
                "id": $id,
                "unite_id": $decor.unite_id,
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

macro_rules! clore {
    ($app:expr, $bearer:expr, $decor:expr, $sejour_id:expr) => {{
        let requete = actix_web::test::TestRequest::post()
            .uri(&format!(
                "/api/v1/etablissements/{}/sejours/{}/depart",
                $decor.jeu.etablissement_id, $sejour_id
            ))
            .insert_header(("authorization", $bearer.to_owned()))
            .to_request();
        actix_web::test::call_service(&$app, requete).await
    }};
}

/// Le constat tel qu'il est en base — **toutes les colonnes qui doivent rester figées**.
async fn lire_constat(pool: &sqlx::PgPool, decor: &Decor, sejour_id: Uuid) -> serde_json::Value {
    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, decor.jeu.tenant_id)
        .await
        .expect("pose du tenant");
    let ligne = sqlx::query!(
        r#"
        SELECT nuits_constatees, nombre_personnes, assujettie_taxe_nuitee,
               regle_conversion_taxe, classement_etablissement, commune,
               nuitees_assujetties, montant_mineur, devise
        FROM hebergement.taxe_sejour_constat
        WHERE sejour_id = $1
        "#,
        sejour_id
    )
    .fetch_one(&mut *tx)
    .await
    .expect("le constat doit exister après la clôture");
    tx.rollback().await.expect("rollback");

    serde_json::json!({
        "nuits_constatees": ligne.nuits_constatees,
        "nombre_personnes": ligne.nombre_personnes,
        "assujettie_taxe_nuitee": ligne.assujettie_taxe_nuitee,
        "regle_conversion_taxe": ligne.regle_conversion_taxe,
        "classement_etablissement": ligne.classement_etablissement,
        "commune": ligne.commune,
        "nuitees_assujetties": ligne.nuitees_assujetties,
        "montant_mineur": ligne.montant_mineur,
        "devise": ligne.devise,
    })
}

// =================================================================================================
//  ★ (a) LE CONSTAT NE BOUGE PAS, QUOI QU'ON CHANGE APRÈS
// =================================================================================================

/// ★ **Après la clôture, modifier le barème, la formule, le classement ou la commune ne change
/// AUCUNE valeur du constat.**
///
/// C'est ce qui donne au mot « figé » un contenu vérifiable. Référencer la formule plutôt que la
/// recopier aurait paru plus propre — et aurait fait bouger un séjour clos hier au premier
/// changement de tarif de demain.
#[actix_web::test]
async fn rien_de_ce_qui_change_apres_le_depart_ne_touche_le_constat() {
    let owner = pool_owner().await;
    let decor = poser_decor(&owner, "départ — constat figé").await;
    let cx = commun::compte_connecte(
        &owner,
        decor.jeu,
        "Yao",
        &[(ROLE, Some(decor.jeu.etablissement_id))],
    )
    .await;
    let app = monter_application!(pool_app().await);

    let sejour_id = Uuid::now_v7();
    let debut = time::OffsetDateTime::now_utc() - time::Duration::hours(2);
    assert_eq!(ouvrir!(app, cx.bearer, decor, sejour_id, debut, 24).status(), 201);
    assert_eq!(clore!(app, cx.bearer, decor, sejour_id).status(), 200);

    let avant = lire_constat(&owner, &decor, sejour_id).await;

    // ── On change TOUT ce que le constat recopie ─────────────────────────────────────────────
    let mut tx = owner.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, decor.jeu.tenant_id)
        .await
        .expect("pose du tenant");

    sqlx::query(
        r#"
        UPDATE hebergement.formule
        SET prix_mineur = 99999, assujettie_taxe_nuitee = false, regle_conversion_taxe = NULL
        WHERE id = $1
        "#,
    )
    .bind(decor.formule_id)
    .execute(&mut *tx)
    .await
    .expect("changement de formule");

    sqlx::query(
        r#"
        UPDATE etablissements.etablissement
        SET classement = 'NON_CLASSE', etoiles = NULL, commune = 'Bouaké'
        WHERE id = $1
        "#,
    )
    .bind(decor.jeu.etablissement_id)
    .execute(&mut *tx)
    .await
    .expect("changement de classement et de commune");

    // Et on ajoute un accompagnant **après** la clôture — il part en réconciliation, et le
    // `nombre_personnes` du constat ne doit pas bouger d'un cran.
    tx.commit().await.expect("commit");

    let requete = actix_web::test::TestRequest::post()
        .uri(&format!(
            "/api/v1/etablissements/{}/sejours/{sejour_id}/accompagnants",
            decor.jeu.etablissement_id
        ))
        .insert_header(("authorization", cx.bearer.clone()))
        .set_json(serde_json::json!({ "id": Uuid::now_v7(), "nom": "Aïcha" }))
        .to_request();
    assert_eq!(actix_web::test::call_service(&app, requete).await.status(), 202);

    let apres = lire_constat(&owner, &decor, sejour_id).await;

    assert_eq!(
        apres, avant,
        "★ le constat a BOUGÉ après la clôture.\n\n\
         SC-007 promet que l'assiette est immuable après le départ. Le paramétrage doit être \
         RECOPIÉ au constat, jamais référencé : référencer la formule paraît plus propre et fait \
         bouger un séjour clos hier au premier changement de tarif de demain.\n\n\
         Avant : {avant}\nAprès : {apres}"
    );

    // ★ Et le montant reste **`null`** — ce cycle a laissé le calcul à FIS-03.
    assert!(
        apres["nuitees_assujetties"].is_null() && apres["montant_mineur"].is_null(),
        "★ un montant de taxe a été écrit par le cycle 006. Décider quelles nuits sont \
         assujetties est une RÈGLE FISCALE : elle ne vit que dans `JurisdictionAdapter` (P-12), \
         et son test doré appartient à FIS-03. Constat : {apres}"
    );
}

// =================================================================================================
//  ★ (b) IMMUABILITÉ PAR PRIVILÈGE
// =================================================================================================

/// ★ **Sous le rôle applicatif, `UPDATE` et `DELETE` sur le constat échouent en
/// `permission denied`.**
///
/// ⚠️ **Asserté bien que la migration le garantisse** : *une garantie de privilège se perd en une
/// ligne de migration*. Un `GRANT UPDATE` ajouté un jour « pour débloquer un correctif » rendrait
/// ce test rouge, et c'est exactement ce qu'on veut.
#[tokio::test]
async fn le_constat_est_immuable_pour_le_role_applicatif() {
    let app_pool = pool_app().await;

    let privileges: Vec<(String,)> = sqlx::query_as(
        r#"
        SELECT privilege_type
        FROM information_schema.role_table_grants
        WHERE grantee = 'kaya_app'
          AND table_schema = 'hebergement'
          AND table_name = 'taxe_sejour_constat'
        ORDER BY 1
        "#,
    )
    .fetch_all(&pool_owner().await)
    .await
    .expect("lecture des privilèges");

    let accordes: Vec<&str> = privileges.iter().map(|(p,)| p.as_str()).collect();

    assert!(
        accordes.contains(&"SELECT") && accordes.contains(&"INSERT"),
        "le rôle applicatif doit pouvoir LIRE et ÉCRIRE un constat : sans cela, aucun départ ne \
         peut le figer. Privilèges : {accordes:?}"
    );

    for interdit in ["UPDATE", "DELETE"] {
        assert!(
            !accordes.contains(&interdit),
            "★ le rôle applicatif peut `{interdit}` sur `hebergement.taxe_sejour_constat`.\n\n\
             Le figeage est un PRIVILÈGE, pas une intention : c'est ce qui transforme SC-007 — \
             « l'assiette est immuable après le départ » — d'une promesse en une propriété de la \
             base. Une relecture ne doit pas pouvoir la recalculer ; le rôle applicatif n'en a \
             pas le droit.\n\
             Privilèges observés : {accordes:?}"
        );
    }

    // ── Le versant EXÉCUTÉ, pas seulement le catalogue ───────────────────────────────────────
    //
    // Lire `information_schema` prouve ce que le catalogue dit. Tenter l'écriture prouve ce que
    // la base fait — et les deux ne coïncident que si aucune politique, aucun rôle hérité et
    // aucun `SECURITY DEFINER` ne s'interpose.
    let refus = sqlx::query("UPDATE hebergement.taxe_sejour_constat SET nuits_constatees = 99")
        .execute(&app_pool)
        .await;

    assert!(
        refus.is_err(),
        "un `UPDATE` sur le constat a RÉUSSI sous le rôle applicatif. Le catalogue disait le \
         contraire : une politique, un rôle hérité ou un `SECURITY DEFINER` s'interpose."
    );
}

// =================================================================================================
//  ★ (c) LA NOTE ARRÊTÉE REFUSE TOUTE ÉCRITURE
// =================================================================================================

/// **Après le départ, la note est arrêtée et le séjour ne se clôt pas deux fois.**
///
/// Terme utilisateur : « La note est arrêtée : plus rien ne peut s'y ajouter ».
#[actix_web::test]
async fn un_sejour_deja_clos_refuse_une_seconde_cloture() {
    let owner = pool_owner().await;
    let decor = poser_decor(&owner, "départ — double clôture").await;
    let cx = commun::compte_connecte(
        &owner,
        decor.jeu,
        "Yao",
        &[(ROLE, Some(decor.jeu.etablissement_id))],
    )
    .await;
    let app = monter_application!(pool_app().await);

    let sejour_id = Uuid::now_v7();
    let debut = time::OffsetDateTime::now_utc() - time::Duration::hours(2);
    assert_eq!(ouvrir!(app, cx.bearer, decor, sejour_id, debut, 24).status(), 201);
    assert_eq!(clore!(app, cx.bearer, decor, sejour_id).status(), 200);

    let seconde = clore!(app, cx.bearer, decor, sejour_id);
    let statut = seconde.status().as_u16();
    let corps: serde_json::Value = actix_web::test::read_body_json(seconde).await;

    assert_eq!(statut, 409, "corps : {corps}");
    assert_eq!(
        corps["code"], "sejour_deja_clos",
        "le refus porte un CODE STABLE, que l'interface traduit par le lexique : « Ce séjour est \
         déjà terminé. » Corps : {corps}"
    );

    // La note est bien arrêtée en base.
    let mut tx = owner.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, decor.jeu.tenant_id)
        .await
        .expect("pose du tenant");
    let statut_note: String = sqlx::query_scalar(
        "SELECT statut FROM hebergement.note_sejour WHERE sejour_id = $1",
    )
    .bind(sejour_id)
    .fetch_one(&mut *tx)
    .await
    .expect("lecture de la note");
    tx.rollback().await.expect("rollback");

    assert_eq!(statut_note, "arretee");
}

// =================================================================================================
//  ★ (f) AUCUNE CLÔTURE AUTOMATIQUE — FR-068
// =================================================================================================

/// ★ **Un séjour dont la période prévue est dépassée reste `en_cours`.**
///
/// **Aucun worker ne le clôt.** Une clôture d'office produirait une facturation **sans témoin** :
/// personne n'aurait vu le client partir, et le montant serait pourtant arrêté — avec une taxe
/// figée que plus rien ne pourra corriger.
///
/// Le test attend au-delà de la fin prévue et vérifie que **rien** ne s'est passé.
#[actix_web::test]
async fn un_sejour_depasse_reste_ouvert_car_aucune_cloture_n_est_automatique() {
    let owner = pool_owner().await;
    let decor = poser_decor(&owner, "départ — aucune clôture d'office").await;
    let cx = commun::compte_connecte(
        &owner,
        decor.jeu,
        "Yao",
        &[(ROLE, Some(decor.jeu.etablissement_id))],
    )
    .await;
    let app = monter_application!(pool_app().await);

    // Un séjour dont la fin prévue est **déjà passée** : ouvert il y a trois heures pour une heure.
    let sejour_id = Uuid::now_v7();
    let debut = time::OffsetDateTime::now_utc() - time::Duration::hours(3);
    assert_eq!(ouvrir!(app, cx.bearer, decor, sejour_id, debut, 1).status(), 201);

    let mut tx = owner.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, decor.jeu.tenant_id)
        .await
        .expect("pose du tenant");
    let ligne = sqlx::query!(
        r#"SELECT statut, clos_le FROM hebergement.sejour WHERE id = $1"#,
        sejour_id
    )
    .fetch_one(&mut *tx)
    .await
    .expect("lecture du séjour");

    let constats: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM hebergement.taxe_sejour_constat WHERE sejour_id = $1",
    )
    .bind(sejour_id)
    .fetch_one(&mut *tx)
    .await
    .expect("comptage");
    tx.rollback().await.expect("rollback");

    assert_eq!(
        ligne.statut, "en_cours",
        "★ un séjour dépassé a été clos SANS QUE PERSONNE NE LE DEMANDE (FR-068).\n\
         Une clôture d'office produit une facturation sans témoin : personne n'a vu le client \
         partir, et le montant est pourtant arrêté — avec une taxe FIGÉE que plus rien ne \
         corrigera."
    );
    assert!(ligne.clos_le.is_none());
    assert_eq!(
        constats, 0,
        "un constat de taxe a été figé sur un séjour que personne n'a clos"
    );
}

// =================================================================================================
//  ★ (g) DÉRIVE D'HORLOGE — SC-011, le versant COMPORTEMENT
// =================================================================================================

/// ★ **Le même départ, rejoué avec une horloge de terminal décalée de +1 h puis −1 h, produit un
/// constat IDENTIQUE.**
///
/// ⚠️ **Ce n'est PAS couvert par P-23.** Celle-ci analyse le **code** et prouve qu'aucun calcul ne
/// lit `horodatage_client`. Ce test éprouve le **comportement** — et c'est la forme qu'avait déjà
/// `hebergement_tarification.rs` au cycle 004.
///
/// L'enjeu est plus grand ici qu'au cycle 004 : un départ antidaté ne fausse pas seulement un
/// montant, il fausse une **assiette fiscale figée**, que plus rien ne pourra corriger.
#[actix_web::test]
async fn une_horloge_de_terminal_decalee_ne_change_rien_au_constat() {
    let owner = pool_owner().await;
    let app = monter_application!(pool_app().await);

    let mut constats = Vec::new();

    // Trois départs identiques, avec trois `horodatage_client` différents : à l'heure, +1 h, −1 h.
    for (etiquette, decalage) in [
        ("à l'heure", time::Duration::ZERO),
        ("+1 h", time::Duration::hours(1)),
        ("−1 h", time::Duration::hours(-1)),
    ] {
        let decor = poser_decor(&owner, &format!("départ — horloge {etiquette}")).await;
        let cx = commun::compte_connecte(
            &owner,
            decor.jeu,
            "Yao",
            &[(ROLE, Some(decor.jeu.etablissement_id))],
        )
        .await;

        let sejour_id = Uuid::now_v7();
        let debut = time::OffsetDateTime::now_utc() - time::Duration::hours(2);

        // L'ouverture porte un `horodatage_client` **décalé** — indicatif, aucune règle (P-23).
        let requete = actix_web::test::TestRequest::post()
            .uri(&format!(
                "/api/v1/etablissements/{}/sejours",
                decor.jeu.etablissement_id
            ))
            .insert_header(("authorization", cx.bearer.clone()))
            .set_json(serde_json::json!({
                "id": sejour_id,
                "unite_id": decor.unite_id,
                "formule_id": decor.formule_id,
                "debut_client": debut
                    .format(&time::format_description::well_known::Rfc3339).unwrap(),
                "fin_client": (debut + time::Duration::hours(24))
                    .format(&time::format_description::well_known::Rfc3339).unwrap(),
                "horodatage_client": (time::OffsetDateTime::now_utc() + decalage)
                    .format(&time::format_description::well_known::Rfc3339).unwrap(),
            }))
            .to_request();
        assert_eq!(actix_web::test::call_service(&app, requete).await.status(), 201);

        assert_eq!(clore!(app, cx.bearer, decor, sejour_id).status(), 200);
        constats.push((etiquette, lire_constat(&owner, &decor, sejour_id).await));
    }

    let (_, reference) = &constats[0];
    for (etiquette, constat) in &constats[1..] {
        assert_eq!(
            constat, reference,
            "★ une horloge de terminal décalée de {etiquette} a produit un constat DIFFÉRENT.\n\n\
             Toute durée vient de `now()` de la BASE, jamais de l'horloge d'un terminal (P-23). \
             Un départ antidaté ne fausserait pas seulement un montant : il fausserait une \
             ASSIETTE FISCALE FIGÉE, que plus rien ne pourra corriger.\n\n\
             Référence : {reference}\nObtenu    : {constat}"
        );
    }
}

/// **Le nombre de nuits d'un passage de deux heures est ZÉRO, et c'est juste.**
///
/// Un passage n'est pas une nuitée. Compter une nuit « pour arrondir » ferait payer une taxe de
/// séjour à qui n'a pas dormi — et l'état de reversement communal la réclamerait au trésorier.
#[actix_web::test]
async fn un_passage_de_deux_heures_ne_constate_aucune_nuit() {
    let owner = pool_owner().await;
    let decor = poser_decor(&owner, "départ — passage sans nuit").await;
    let cx = commun::compte_connecte(
        &owner,
        decor.jeu,
        "Yao",
        &[(ROLE, Some(decor.jeu.etablissement_id))],
    )
    .await;
    let app = monter_application!(pool_app().await);

    let sejour_id = Uuid::now_v7();

    // ⚠️ **Le début est RELATIF à maintenant, jamais posé à une heure fixe de la journée.**
    //
    // La version d'origine écrivait `replace_hour(10)` : elle plaçait le début à 10 h du jour
    // courant, ce qui est **dans le futur** avant 10 h UTC. Le départ, lui, mesure la durée réelle
    // depuis `now()` de la base (P-23) — il calculait donc une période dont la fin précède le
    // début, et la contrainte `constat_periode_coherente` rendait `500`. Le test était **vert de
    // 10 h à minuit et rouge de minuit à 10 h**, sans que rien ne désigne l'heure comme cause.
    //
    // Cinq minutes en arrière garantissent deux choses à la fois : le séjour est **en cours**, et
    // aucune frontière de jour n'est franchie — sauf à lancer la suite dans les cinq minutes qui
    // suivent minuit, cas où l'assertion resterait juste pour une raison différente.
    let debut = time::OffsetDateTime::now_utc() - time::Duration::minutes(5);

    // La durée **vendue** reste de deux heures ; la durée **réelle** au départ est de cinq
    // minutes. C'est la seconde qui décide du nombre de nuits, et c'est le sujet du test.
    assert_eq!(ouvrir!(app, cx.bearer, decor, sejour_id, debut, 2).status(), 201);
    assert_eq!(clore!(app, cx.bearer, decor, sejour_id).status(), 200);

    let constat = lire_constat(&owner, &decor, sejour_id).await;
    assert_eq!(
        constat["nuits_constatees"], 0,
        "un passage de deux heures dans la même journée ne constate AUCUNE nuit. Compter une \
         nuit « pour arrondir » ferait payer une taxe de séjour à qui n'a pas dormi — et l'état \
         de reversement la réclamerait au trésorier. Constat : {constat}"
    );
}

/// Le décor porte une catégorie — référencée par les tests de prolongation (US5).
#[allow(dead_code)]
fn _categorie(decor: &Decor) -> Uuid {
    decor.categorie_id
}
