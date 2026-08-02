//! **PORTE P-09 — levée au cycle 004.** La garantie centrale du produit, et sa démonstration.
//!
//! > **Deux clients ne peuvent jamais recevoir la même unité au même moment.**
//!
//! Le cycle 001 a installé cette porte **à vide**, avec une assertion de non-régression qui
//! dictait, mot pour mot, ce que le cycle qui créerait la table `occupation` devrait vérifier ici.
//! Ce fichier est la réponse à ce message. Les trois assertions, dans son ordre :
//!
//! | # | Assertion | Moyen |
//! |---|---|---|
//! | 1 | La période est un `tstzrange`, jamais une paire de dates | Lecture d'`information_schema` |
//! | 2 | Une contrainte `EXCLUDE USING gist (unite_id WITH =, periode WITH &&)` la protège | Lecture de `pg_constraint` |
//! | 3 | Deux attributions concurrentes chevauchantes échouent — pas « improbablement », jamais | **Deux transactions réelles** |
//!
//! # Pourquoi l'assertion 3 asserte la CAUSE et pas seulement le refus
//!
//! Un test qui se contenterait de « une seule a réussi » passerait au vert sur un
//! `SELECT … FOR UPDATE`, sur `SERIALIZABLE`, ou sur un verrou applicatif posé dans le service.
//! Ces trois mécanismes rendent la double attribution **improbable** ; ils se dégradent sous
//! charge, en cas de reprise après incident, ou dès qu'un second processus écrit — et rien ne le
//! signale.
//!
//! Le test asserte donc que l'échec est un `ErrorKind::ExclusionViolation` **sur la contrainte
//! nommée**. C'est la seule formulation qui distingue une garantie d'une coïncidence.
//!
//! # Deux transactions suffisent, et mille ne prouveraient rien de plus
//!
//! SC-001 parle de mille attributions concurrentes. Deux transactions prouvent que **la base**
//! rejette ; mille prouveraient la même chose en occupant la CI pendant des minutes. Ce qui
//! compte n'est pas le nombre, c'est que le refus vienne de la contrainte.

mod commun;

use std::time::Duration;

use sqlx::Row;
use time::OffsetDateTime;
use time::macros::datetime;
use uuid::Uuid;

use commun::{JeuTenant, creer_tenant, pool_app, pool_owner};
use kaya_hebergement::erreurs::{CONTRAINTE_SANS_CHEVAUCHEMENT, est_violation_exclusion};

// =================================================================================================
//  Fabriques
// =================================================================================================

/// Un établissement avec l'hébergement actif, un type de chambre, une chambre et une nuitée.
struct Decor {
    jeu: JeuTenant,
    unite_id: Uuid,
    formule_id: Uuid,
    categorie_id: Uuid,
}

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
        VALUES ($1, $2, $3, 'HEBERGEMENT', true)
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(Uuid::now_v7())
    .bind(jeu.tenant_id)
    .bind(jeu.etablissement_id)
    .execute(&mut *tx)
    .await
    .expect("activation du module");

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
        INSERT INTO hebergement.unite
            (id, tenant_id, etablissement_id, categorie_id, code, etage)
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
        unite_id,
        formule_id,
        categorie_id,
    }
}

/// Insère une occupation par **SQL direct**, sur une transaction fournie.
///
/// Direct, et c'est le sujet : ces tests vérifient ce que la **base** garantit, pas ce que le
/// service organise. Passer par le service laisserait ouverte la question de savoir lequel des
/// deux refuse.
async fn inserer_occupation(
    tx: &mut sqlx::PgTransaction<'_>,
    decor: &Decor,
    debut: OffsetDateTime,
    fin: OffsetDateTime,
    fin_periode: OffsetDateTime,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO hebergement.occupation
            (id, tenant_id, etablissement_id, unite_id, formule_id,
             periode, debut_client, fin_client)
        VALUES ($1, $2, $3, $4, $5, tstzrange($6, $7, '[)'), $6, $8)
        "#,
    )
    .bind(Uuid::now_v7())
    .bind(decor.jeu.tenant_id)
    .bind(decor.jeu.etablissement_id)
    .bind(decor.unite_id)
    .bind(decor.formule_id)
    .bind(debut)
    .bind(fin_periode)
    .bind(fin)
    .execute(&mut **tx)
    .await
    .map(|_| ())
}

// =================================================================================================
//  ASSERTION 1 — un intervalle, jamais une paire de dates
// =================================================================================================

/// Le marché pratique massivement le **passage horaire** et la **demi-journée**.
///
/// Une paire `(date_arrivee, date_depart)` ne sait pas dire « 14 h → 18 h le même jour ». Le
/// premier code qui essaierait ajouterait une colonne d'heure à côté — deux sources pour un même
/// fait, et le jour où elles divergent, la chambre est double-attribuée.
///
/// Le test vérifie **les deux versants** : `periode` est bien un `tstzrange`, **et** aucune
/// colonne de type `date` n'est apparue à côté.
#[actix_web::test]
async fn p09_assertion_1_la_periode_est_un_tstzrange_et_aucune_paire_de_dates_n_existe() {
    let pool = pool_owner().await;

    let colonnes: Vec<(String, String)> = sqlx::query_as(
        r#"
        SELECT column_name, data_type
        FROM information_schema.columns
        WHERE table_schema = 'hebergement' AND table_name = 'occupation'
        ORDER BY ordinal_position
        "#,
    )
    .fetch_all(&pool)
    .await
    .expect("lecture du catalogue système");

    assert!(
        !colonnes.is_empty(),
        "P-09 : `hebergement.occupation` n'a aucune colonne — la porte n'inspecte rien, et son \
         vert ne dirait rien"
    );

    let periode = colonnes
        .iter()
        .find(|(nom, _)| nom == "periode")
        .unwrap_or_else(|| panic!("P-09 : aucune colonne `periode`. Colonnes : {colonnes:?}"));

    assert_eq!(
        periode.1, "tstzrange",
        "P-09 : `periode` est de type « {} » au lieu de `tstzrange`.\n\
         Une occupation est un intervalle `[début, fin)` en timestamp AVEC FUSEAU, jamais une \
         paire de dates : le marché pratique massivement le passage horaire et la demi-journée.",
        periode.1
    );

    let dates: Vec<&(String, String)> = colonnes
        .iter()
        .filter(|(_, genre)| genre == "date")
        .collect();
    assert!(
        dates.is_empty(),
        "P-09 : {} colonne(s) de type `date` sur `occupation` : {dates:?}.\n\
         Une paire de dates à côté de l'intervalle serait une seconde source pour le même fait — \
         et le jour où les deux divergent, la chambre est double-attribuée.",
        dates.len()
    );
}

// =================================================================================================
//  ASSERTION 2 — la contrainte existe, et c'est bien celle-là
// =================================================================================================

/// `contype = 'x'` désigne une contrainte d'exclusion. Le test lit sa **définition** et y cherche
/// les deux opérateurs : l'égalité sur l'unité, le chevauchement sur la période.
///
/// Vérifier seulement l'existence d'une contrainte d'exclusion laisserait passer
/// `EXCLUDE USING gist (etablissement_id WITH =, periode WITH &&)` — qui interdirait deux
/// occupations simultanées **dans tout l'établissement**. L'hôtel ne pourrait louer qu'une chambre
/// à la fois, et le test serait vert.
#[actix_web::test]
async fn p09_assertion_2_une_contrainte_d_exclusion_gist_protege_la_periode() {
    let pool = pool_owner().await;

    let contraintes: Vec<(String, String)> = sqlx::query_as(
        r#"
        SELECT c.conname, pg_get_constraintdef(c.oid)
        FROM pg_constraint c
        JOIN pg_class t ON t.oid = c.conrelid
        JOIN pg_namespace n ON n.oid = t.relnamespace
        WHERE n.nspname = 'hebergement' AND t.relname = 'occupation' AND c.contype = 'x'
        "#,
    )
    .fetch_all(&pool)
    .await
    .expect("lecture de pg_constraint");

    assert_eq!(
        contraintes.len(),
        1,
        "P-09 : {} contrainte(s) d'exclusion sur `occupation`, une seule attendue : {contraintes:?}\n\
         `est_violation_exclusion` vérifie le NOM de la contrainte précisément pour qu'une seconde \
         ne fasse pas passer ses violations pour des doubles attributions. Une seconde contrainte \
         doit donc être une décision, pas une surprise.",
        contraintes.len()
    );

    let (nom, definition) = &contraintes[0];
    assert_eq!(
        nom, CONTRAINTE_SANS_CHEVAUCHEMENT,
        "P-09 : la contrainte d'exclusion s'appelle « {nom} ». Le crate traduit la violation en \
         cherchant « {CONTRAINTE_SANS_CHEVAUCHEMENT} » : un nom différent produirait un `500` au \
         lieu d'un `409`, sur le chemin le plus important du produit."
    );

    let d = definition.to_lowercase();
    assert!(d.contains("gist"), "P-09 : la contrainte n'emploie pas GiST : {definition}");
    assert!(
        d.contains("unite_id with ="),
        "P-09 : la contrainte ne porte pas `unite_id WITH =`.\n\
         Sans lui, elle interdirait deux occupations simultanées dans TOUT l'établissement — \
         l'hôtel ne pourrait louer qu'une chambre à la fois. Définition : {definition}"
    );
    assert!(
        d.contains("periode with &&"),
        "P-09 : la contrainte ne porte pas `periode WITH &&`. Définition : {definition}"
    );
}

// =================================================================================================
//  ASSERTION 3 — LE TEST QUI DISTINGUE UNE GARANTIE D'UNE COÏNCIDENCE
// =================================================================================================

/// **Deux transactions PostgreSQL distinctes**, insertion dans chacune sans commit, puis commit
/// des deux. Exactement une réussit, et l'échec est un `ExclusionViolation` sur la contrainte
/// nommée.
///
/// # Ce que le test refuse de se contenter de vérifier
///
/// « Une seule a réussi » est vrai aussi avec un `SELECT … FOR UPDATE`, avec `SERIALIZABLE`, ou
/// avec un verrou applicatif dans le service. Ces trois mécanismes se dégradent sous charge sans
/// rien signaler. La **cause** du refus est donc assertée, et c'est ce qui fait de ce test une
/// démonstration plutôt qu'une observation.
///
/// # L'ordonnancement, et pourquoi il n'a pas besoin d'être parfait
///
/// La seconde transaction est lancée pendant que la première tient sa ligne non validée. Deux
/// issues, et **les deux sont concluantes** : soit elle atteint l'insertion et se bloque sur le
/// verrou de la contrainte jusqu'au commit de la première, puis échoue ; soit elle arrive après
/// le commit et échoue sur la ligne validée. Dans les deux cas, l'échec vient de la contrainte.
#[actix_web::test]
async fn deux_attributions_concurrentes_une_seule_reussit() {
    let pool = pool_app().await;
    let decor = poser_decor(&pool_owner().await, "P-09 concurrence").await;

    // Deux intervalles **chevauchants** sur la même unité — 14 h → 12 h le surlendemain, et
    // 10 h → 12 h le lendemain, qui tombe au milieu du premier.
    let debut_1 = datetime!(2026-08-03 14:00 UTC);
    let fin_1 = datetime!(2026-08-05 12:00 UTC);
    let debut_2 = datetime!(2026-08-04 10:00 UTC);
    let fin_2 = datetime!(2026-08-04 12:00 UTC);

    let mut tx1 = pool.begin().await.expect("transaction 1");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx1, decor.jeu.tenant_id)
        .await
        .expect("tenant 1");
    inserer_occupation(&mut tx1, &decor, debut_1, fin_1, fin_1)
        .await
        .expect("la première attribution doit réussir");

    // La seconde, dans une transaction **distincte** — donc sur une autre connexion.
    let pool2 = pool.clone();
    let jeu = decor.jeu;
    let unite_id = decor.unite_id;
    let formule_id = decor.formule_id;
    let categorie_id = decor.categorie_id;
    let seconde = tokio::spawn(async move {
        let decor = Decor {
            jeu,
            unite_id,
            formule_id,
            categorie_id,
        };
        let mut tx2 = pool2.begin().await.expect("transaction 2");
        kaya_etablissements::tenant_context::poser_tenant(&mut tx2, decor.jeu.tenant_id)
            .await
            .expect("tenant 2");
        let resultat = inserer_occupation(&mut tx2, &decor, debut_2, fin_2, fin_2).await;
        match resultat {
            Ok(()) => {
                tx2.commit().await.expect("commit 2");
                Ok(())
            }
            Err(e) => {
                let _ = tx2.rollback().await;
                Err(e)
            }
        }
    });

    // Laisser la seconde atteindre son insertion — voir la note d'ordonnancement ci-dessus : si
    // elle n'y est pas encore, le test reste concluant.
    tokio::time::sleep(Duration::from_millis(250)).await;
    tx1.commit().await.expect("commit 1");

    let resultat = seconde.await.expect("la tâche concurrente ne doit pas paniquer");

    let Err(erreur) = resultat else {
        panic!(
            "LES DEUX attributions ont réussi sur la même unité et des intervalles chevauchants.\n\
             La contrainte d'exclusion est absente, désactivée, ou ne porte pas sur ce qu'elle \
             devrait. C'est la garantie centrale du produit : deux clients viennent de recevoir la \
             même chambre."
        );
    };

    // ═══ La CAUSE, pas seulement l'existence du refus ═══
    assert!(
        est_violation_exclusion(&erreur, CONTRAINTE_SANS_CHEVAUCHEMENT),
        "L'attribution concurrente a bien échoué, mais **pas par la contrainte d'exclusion**.\n\
         Erreur obtenue : {erreur}\n\n\
         Un refus qui vient d'ailleurs — verrou applicatif, SELECT … FOR UPDATE, SERIALIZABLE — \
         rend la double attribution IMPROBABLE, pas impossible. Les trois se dégradent sous \
         charge sans rien signaler. Le principe IV l'écrit mot pour mot : « garantie par une \
         contrainte d'exclusion PostgreSQL, PAS par un verrou applicatif »."
    );

    // Et il ne reste **qu'une** occupation.
    let pool_o = pool_owner().await;
    let mut tx = pool_o.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, decor.jeu.tenant_id)
        .await
        .expect("tenant");
    let total: i64 = sqlx::query(
        "SELECT COUNT(*) AS c FROM hebergement.occupation WHERE unite_id = $1",
    )
    .bind(decor.unite_id)
    .fetch_one(&mut *tx)
    .await
    .expect("comptage")
    .get("c");

    assert_eq!(
        total, 1,
        "{total} occupation(s) sur la même unité — exactement une devait survivre"
    );
}
