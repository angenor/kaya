//! **Les contraintes du référentiel d'hébergement — celles de la BASE.**
//!
//! Ce fichier n'exerce pas le service : il exerce ce que la table garantit, seule, y compris
//! contre une migration de données ou un script de maintenance qui contournerait le code.
//!
//! # Pourquoi ces contraintes-là méritent un test
//!
//! Une contrainte `CHECK` se lit dans la migration ; ce qui ne se lit pas, c'est **qu'elle est
//! effective sur la base réellement déployée**. Un `CHECK` mal écrit — comparant deux colonnes
//! nulles, par exemple — est syntaxiquement valide et ne refuse jamais rien.
//!
//! Chaque test ci-dessous fait donc l'insertion que la contrainte doit refuser, et constate le
//! refus. Un test qui se contenterait de lire `pg_constraint` prouverait que la contrainte est
//! *déclarée*, jamais qu'elle *mord*.

mod commun;

use uuid::Uuid;

use commun::{JeuTenant, creer_tenant, pool_owner};

// =================================================================================================
//  Fabriques — écrites une fois, employées partout
// =================================================================================================

/// Insère une catégorie, sous le tenant courant.
async fn creer_categorie(pool: &sqlx::PgPool, jeu: JeuTenant, nom: &str) -> Uuid {
    let id = Uuid::now_v7();
    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, jeu.tenant_id)
        .await
        .expect("pose du tenant");

    sqlx::query(
        r#"
        INSERT INTO hebergement.categorie (id, tenant_id, etablissement_id, nom, capacite_accueil)
        VALUES ($1, $2, $3, $4, 2)
        "#,
    )
    .bind(id)
    .bind(jeu.tenant_id)
    .bind(jeu.etablissement_id)
    .bind(nom)
    .execute(&mut *tx)
    .await
    .expect("insertion de la catégorie");

    tx.commit().await.expect("commit");
    id
}

/// Insère une formule et rend le résultat brut — les tests de contrainte veulent l'erreur.
#[allow(clippy::too_many_arguments)]
async fn inserer_formule(
    pool: &sqlx::PgPool,
    jeu: JeuTenant,
    categorie_id: Uuid,
    famille: &str,
    prix_mineur: i64,
    duree_min: Option<i32>,
    duree_max: Option<i32>,
    assujettie: bool,
    regle: Option<&str>,
    heure_sup: Option<i64>,
) -> Result<Uuid, sqlx::Error> {
    let id = Uuid::now_v7();
    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, jeu.tenant_id)
        .await
        .expect("pose du tenant");

    let resultat = sqlx::query(
        r#"
        INSERT INTO hebergement.formule
            (id, tenant_id, etablissement_id, categorie_id, famille, prix_mineur,
             duree_min_minutes, duree_max_minutes, assujettie_taxe_nuitee,
             regle_conversion_taxe, prix_heure_supplementaire_mineur)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        "#,
    )
    .bind(id)
    .bind(jeu.tenant_id)
    .bind(jeu.etablissement_id)
    .bind(categorie_id)
    .bind(famille)
    .bind(prix_mineur)
    .bind(duree_min)
    .bind(duree_max)
    .bind(assujettie)
    .bind(regle)
    .bind(heure_sup)
    .execute(&mut *tx)
    .await;

    match resultat {
        Ok(_) => {
            tx.commit().await.expect("commit");
            Ok(id)
        }
        Err(e) => {
            tx.rollback().await.ok();
            Err(e)
        }
    }
}

/// Une formule de nuitée valide — le cas normal, dont les tests dérivent.
async fn creer_nuitee(pool: &sqlx::PgPool, jeu: JeuTenant, categorie_id: Uuid) -> Uuid {
    inserer_formule(
        pool,
        jeu,
        categorie_id,
        "NUITEE",
        12_500,
        None,
        None,
        true,
        Some("une_nuitee_par_occupation"),
        None,
    )
    .await
    .expect("la nuitée de référence doit s'insérer")
}

fn nomme(erreur: &sqlx::Error, contrainte: &str) -> bool {
    matches!(erreur, sqlx::Error::Database(e) if e.constraint() == Some(contrainte))
}

// =================================================================================================
//  1. FR-021 — une catégorie ne porte pas deux formules de la même famille
// =================================================================================================

/// Deux « Nuitée » sur le même type de chambre, ce sont **deux prix pour la même chose**, et le
/// choix se ferait par ordre d'insertion. La base le rend impossible.
#[actix_web::test]
async fn deux_formules_de_meme_famille_sur_une_categorie_sont_refusees() {
    let pool = pool_owner().await;
    let jeu = creer_tenant(&pool, "HEB — famille unique").await;
    let categorie = creer_categorie(&pool, jeu, "Standard").await;

    creer_nuitee(&pool, jeu, categorie).await;

    let seconde = inserer_formule(
        &pool,
        jeu,
        categorie,
        "NUITEE",
        15_000,
        None,
        None,
        false,
        None,
        None,
    )
    .await;

    let erreur = seconde.expect_err("une seconde NUITEE sur la même catégorie doit être refusée");
    assert!(
        nomme(&erreur, "formule_famille_unique"),
        "le refus doit venir de `formule_famille_unique`, pas d'autre chose : {erreur}"
    );
}

/// **La contrainte porte sur la catégorie, pas sur l'établissement.** Deux catégories du même
/// établissement peuvent chacune avoir leur nuitée — c'est même le cas normal de Deloria, qui en
/// a cinq à cinq prix.
#[actix_web::test]
async fn deux_categories_ont_chacune_leur_nuitee() {
    let pool = pool_owner().await;
    let jeu = creer_tenant(&pool, "HEB — nuitée par catégorie").await;
    let standard = creer_categorie(&pool, jeu, "Standard").await;
    let superieure = creer_categorie(&pool, jeu, "Supérieure").await;

    creer_nuitee(&pool, jeu, standard).await;
    creer_nuitee(&pool, jeu, superieure).await;
}

// =================================================================================================
//  2. FR-020 — des durées cohérentes
// =================================================================================================

#[actix_web::test]
async fn une_duree_maximale_inferieure_a_la_minimale_est_refusee() {
    let pool = pool_owner().await;
    let jeu = creer_tenant(&pool, "HEB — durées").await;
    let categorie = creer_categorie(&pool, jeu, "Standard").await;

    let erreur = inserer_formule(
        &pool,
        jeu,
        categorie,
        "PASSAGE",
        1_500,
        Some(480),
        Some(60),
        false,
        None,
        Some(1_200),
    )
    .await
    .expect_err("une durée max inférieure à la min doit être refusée");

    assert!(
        nomme(&erreur, "formule_durees_coherentes"),
        "le refus doit venir de `formule_durees_coherentes` : {erreur}"
    );
}

/// **Une seule borne renseignée reste valide.** Une nuitée n'a ni durée minimale ni maximale ; un
/// passage peut n'avoir qu'un plancher. La contrainte ne doit pas transformer l'absence en faute.
#[actix_web::test]
async fn une_seule_borne_de_duree_reste_valide() {
    let pool = pool_owner().await;
    let jeu = creer_tenant(&pool, "HEB — borne seule").await;
    let categorie = creer_categorie(&pool, jeu, "Standard").await;

    inserer_formule(
        &pool,
        jeu,
        categorie,
        "PASSAGE",
        1_500,
        Some(60),
        None,
        false,
        None,
        Some(1_200),
    )
    .await
    .expect("un passage avec un plancher seul doit s'insérer");
}

// =================================================================================================
//  3. Le prix d'heure supplémentaire est réservé au passage
// =================================================================================================

#[actix_web::test]
async fn un_prix_d_heure_supplementaire_hors_passage_est_refuse() {
    let pool = pool_owner().await;
    let jeu = creer_tenant(&pool, "HEB — heure sup").await;
    let categorie = creer_categorie(&pool, jeu, "Standard").await;

    let erreur = inserer_formule(
        &pool,
        jeu,
        categorie,
        "NUITEE",
        12_500,
        None,
        None,
        false,
        None,
        Some(1_200),
    )
    .await
    .expect_err("une nuitée ne porte pas de prix d'heure supplémentaire");

    assert!(
        nomme(&erreur, "formule_heure_sup_reservee_au_passage"),
        "le refus doit venir de `formule_heure_sup_reservee_au_passage` : {erreur}"
    );
}

// =================================================================================================
//  4. LA contrainte qui supprime un état d'écran
// =================================================================================================

/// Une formule **assujettie sans règle de conversion** est une incohérence, pas un état d'attente.
///
/// C'est ce qui permet à `G2` de n'avoir que deux mentions — « Taxe de séjour comprise dans le
/// prix » et « Pas de taxe de séjour sur cette formule ». Un troisième état, « paramétrage fiscal
/// en attente », n'existe ni à la maquette ni au lexique **parce que la base le rend impossible**.
#[actix_web::test]
async fn une_formule_assujettie_sans_regle_est_impossible_a_enregistrer() {
    let pool = pool_owner().await;
    let jeu = creer_tenant(&pool, "HEB — règle fiscale").await;
    let categorie = creer_categorie(&pool, jeu, "Standard").await;

    let erreur = inserer_formule(
        &pool,
        jeu,
        categorie,
        "NUITEE",
        12_500,
        None,
        None,
        true,
        None,
        None,
    )
    .await
    .expect_err("une formule assujettie sans règle doit être refusée");

    assert!(
        nomme(&erreur, "formule_regle_fiscale_coherente"),
        "le refus doit venir de `formule_regle_fiscale_coherente` : {erreur}"
    );
}

/// **`regle_conversion_taxe = NULL` est permis sur une formule NON assujettie**, et c'est le cas
/// du passage à Deloria. La contrainte ne doit pas exiger une règle là où il n'y a pas de taxe.
#[actix_web::test]
async fn une_formule_non_assujettie_se_passe_de_regle() {
    let pool = pool_owner().await;
    let jeu = creer_tenant(&pool, "HEB — non assujettie").await;
    let categorie = creer_categorie(&pool, jeu, "Standard").await;

    inserer_formule(
        &pool,
        jeu,
        categorie,
        "PASSAGE",
        1_500,
        Some(60),
        Some(480),
        false,
        None,
        Some(1_200),
    )
    .await
    .expect("un passage non assujetti sans règle doit s'insérer");
}

/// Une valeur de règle hors des quatre connues est refusée — **explicitement, jamais ignorée**.
#[actix_web::test]
async fn une_regle_de_conversion_inconnue_est_refusee() {
    let pool = pool_owner().await;
    let jeu = creer_tenant(&pool, "HEB — règle inconnue").await;
    let categorie = creer_categorie(&pool, jeu, "Standard").await;

    let erreur = inserer_formule(
        &pool,
        jeu,
        categorie,
        "NUITEE",
        12_500,
        None,
        None,
        true,
        Some("a_la_tete_du_client"),
        None,
    )
    .await
    .expect_err("une règle de conversion inconnue doit être refusée");

    assert!(
        nomme(&erreur, "formule_regle_conversion_connue"),
        "le refus doit venir de `formule_regle_conversion_connue` : {erreur}"
    );
}

/// Une famille hors des quatre est refusée **par la base**. Le service la refuse aussi, avec un
/// message utilisable — les deux, et pas l'un à la place de l'autre.
#[actix_web::test]
async fn une_famille_inconnue_est_refusee_par_la_base() {
    let pool = pool_owner().await;
    let jeu = creer_tenant(&pool, "HEB — famille inconnue").await;
    let categorie = creer_categorie(&pool, jeu, "Standard").await;

    let erreur = inserer_formule(
        &pool,
        jeu,
        categorie,
        "SEMAINE",
        60_000,
        None,
        None,
        false,
        None,
        None,
    )
    .await
    .expect_err("la famille SEMAINE n'existe pas");

    assert!(
        nomme(&erreur, "formule_famille_connue"),
        "le refus doit venir de `formule_famille_connue` : {erreur}"
    );
}

// =================================================================================================
//  5. bareme_palier — l'ordre est TOTAL par construction
// =================================================================================================

/// **Deux paliers de même durée sont impossibles**, donc un barème désordonné ne se constitue pas.
/// La garantie vient de la clé primaire, pas d'un tri à la lecture.
#[actix_web::test]
async fn deux_paliers_de_meme_duree_sont_impossibles() {
    let pool = pool_owner().await;
    let jeu = creer_tenant(&pool, "HEB — paliers").await;
    let categorie = creer_categorie(&pool, jeu, "Standard").await;
    let formule = inserer_formule(
        &pool,
        jeu,
        categorie,
        "PASSAGE",
        1_500,
        Some(60),
        Some(480),
        false,
        None,
        Some(1_200),
    )
    .await
    .expect("passage");

    let inserer_palier = |duree: i32, prix: i64| {
        let pool = pool.clone();
        async move {
            let mut tx = pool.begin().await.expect("transaction");
            kaya_etablissements::tenant_context::poser_tenant(&mut tx, jeu.tenant_id)
                .await
                .expect("pose du tenant");
            let r = sqlx::query(
                r#"
                INSERT INTO hebergement.bareme_palier (formule_id, duree_minutes, prix_mineur, tenant_id)
                VALUES ($1, $2, $3, $4)
                "#,
            )
            .bind(formule)
            .bind(duree)
            .bind(prix)
            .bind(jeu.tenant_id)
            .execute(&mut *tx)
            .await;
            match r {
                Ok(_) => {
                    tx.commit().await.expect("commit");
                    Ok(())
                }
                Err(e) => {
                    tx.rollback().await.ok();
                    Err(e)
                }
            }
        }
    };

    inserer_palier(60, 1_500).await.expect("premier palier");
    let erreur = inserer_palier(60, 1_800)
        .await
        .expect_err("un second palier de 60 min doit être refusé");

    assert!(
        nomme(&erreur, "bareme_palier_pkey"),
        "le refus doit venir de la clé primaire du barème : {erreur}"
    );
}

/// Un palier de durée nulle serait toujours le premier atteint, et **tout passage vaudrait son
/// prix**. FR-025 le refuse.
#[actix_web::test]
async fn un_palier_de_duree_nulle_est_refuse() {
    let pool = pool_owner().await;
    let jeu = creer_tenant(&pool, "HEB — palier nul").await;
    let categorie = creer_categorie(&pool, jeu, "Standard").await;
    let formule = inserer_formule(
        &pool,
        jeu,
        categorie,
        "PASSAGE",
        1_500,
        Some(60),
        Some(480),
        false,
        None,
        Some(1_200),
    )
    .await
    .expect("passage");

    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, jeu.tenant_id)
        .await
        .expect("pose du tenant");
    let erreur = sqlx::query(
        r#"
        INSERT INTO hebergement.bareme_palier (formule_id, duree_minutes, prix_mineur, tenant_id)
        VALUES ($1, 0, 0, $2)
        "#,
    )
    .bind(formule)
    .bind(jeu.tenant_id)
    .execute(&mut *tx)
    .await
    .expect_err("un palier de durée nulle doit être refusé");

    assert!(
        nomme(&erreur, "bareme_palier_duree_positive"),
        "le refus doit venir de `bareme_palier_duree_positive` : {erreur}"
    );
}

// =================================================================================================
//  6. plage_demi_journee — pas de plage qui traverse minuit
// =================================================================================================

#[actix_web::test]
async fn une_plage_qui_traverse_minuit_est_refusee() {
    let pool = pool_owner().await;
    let jeu = creer_tenant(&pool, "HEB — plage minuit").await;
    let categorie = creer_categorie(&pool, jeu, "Salle de réunion").await;
    let formule = inserer_formule(
        &pool,
        jeu,
        categorie,
        "DEMI_JOURNEE",
        6_000,
        None,
        None,
        false,
        None,
        None,
    )
    .await
    .expect("demi-journée");

    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, jeu.tenant_id)
        .await
        .expect("pose du tenant");
    let erreur = sqlx::query(
        r#"
        INSERT INTO hebergement.plage_demi_journee
            (id, formule_id, heure_debut, heure_fin, libelle_cle, tenant_id)
        VALUES ($1, $2, '22:00', '06:00', 'hebergement.plages.nuit', $3)
        "#,
    )
    .bind(Uuid::now_v7())
    .bind(formule)
    .bind(jeu.tenant_id)
    .execute(&mut *tx)
    .await
    .expect_err("une plage 22 h → 6 h doit être refusée");

    assert!(
        nomme(&erreur, "plage_bornes"),
        "le refus doit venir de `plage_bornes` : {erreur}"
    );
}

// =================================================================================================
//  7. L'ABSENCE de `statut_occupation` — vérifiée, pas supposée
// =================================================================================================

/// **Le statut d'occupation est dérivé, et rien ne permet de le poser à la main.**
///
/// Ce test lit le catalogue système plutôt qu'une constante : il échoue le jour où quelqu'un
/// ajoute la colonne « pour aller plus vite », ce qui est exactement le moment où la question doit
/// se poser. Le cadrage §11.4 désigne la confusion entre statut d'occupation et statut de ménage
/// comme la cause des doubles attributions.
#[actix_web::test]
async fn unite_ne_porte_aucune_colonne_de_statut_d_occupation() {
    let pool = pool_owner().await;

    let colonnes: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT column_name
        FROM information_schema.columns
        WHERE table_schema = 'hebergement' AND table_name = 'unite'
        ORDER BY ordinal_position
        "#,
    )
    .fetch_all(&pool)
    .await
    .expect("lecture du catalogue système");

    assert!(
        !colonnes.is_empty(),
        "la table `hebergement.unite` n'existe pas : ce test n'inspecte rien"
    );
    assert!(
        !colonnes.iter().any(|c| c.contains("statut_occupation")),
        "`hebergement.unite` porte une colonne de statut d'occupation. Il est DÉRIVÉ des \
         occupations (cadrage §11.4, registre §7.2) : l'inscrire en table rend possible de le \
         poser à la main, et c'est la cause désignée des doubles attributions.\n\
         Colonnes trouvées : {colonnes:?}"
    );
    assert!(
        colonnes.iter().any(|c| c == "statut_menage"),
        "`statut_menage` doit exister — c'est l'autre statut, de classe A, et le seul qui se pose \
         à la main. Sa présence est ce qui rend l'absence du premier significative."
    );
}
