//! Les seeds sont **rejouables** — SC-007.
//!
//! Trois exécutions successives produisent le même état final.
//!
//! # Pourquoi trois et pas deux
//!
//! Deux exécutions attrapent la duplication franche : le second passage crée un doublon, le
//! décompte double, le test échoue. Elles laissent passer un défaut plus discret — un seed qui
//! **met à jour** ce qu'il trouve au lieu de ne rien faire. Deux exécutions donneraient alors le
//! même décompte et un contenu modifié.
//!
//! Le test compare donc **le décompte et le contenu**, sur trois passages. C'est la forme du test
//! de rejeu de la classe A, appliquée aux seeds — et pour la même raison : recharger une
//! démonstration devant le pilote ne doit rien changer à ce qu'il voit.

mod commun;

use std::process::Command;

use uuid::{Uuid, uuid};

const TENANT_DELORIA: Uuid = uuid!("0198c4a0-0000-7000-8000-000000000001");
const TENANT_RESIDENCE_TEST: Uuid = uuid!("0198c4a0-0000-7000-8000-000000000011");
const ETABLISSEMENT_DELORIA: Uuid = uuid!("0198c4a0-0000-7000-8000-000000000002");
const ETABLISSEMENT_RESIDENCE_TEST: Uuid = uuid!("0198c4a0-0000-7000-8000-000000000012");

#[tokio::test]
async fn trois_executions_produisent_le_meme_etat_final() {
    let pool = commun::pool_owner().await;

    let mut etats = Vec::new();
    for passage in 1..=3 {
        executer_seeds(passage);
        etats.push(lire_etat(&pool).await);
    }

    assert_eq!(
        etats[0], etats[1],
        "le deuxième passage a changé l'état : les seeds ne sont pas rejouables"
    );
    assert_eq!(
        etats[1], etats[2],
        "le troisième passage a changé l'état — un seed qui met à jour au lieu de ne rien faire \
         passerait le test à deux exécutions"
    );

    // Les deux tenants attendus, et **seulement** eux dans le jeu seedé.
    assert_eq!(
        etats[0].len(),
        2,
        "les seeds doivent produire exactement deux tenants, obtenu : {:?}",
        etats[0]
    );

    let noms: Vec<&str> = etats[0].iter().map(|(_, nom, ..)| nom.as_str()).collect();
    assert!(noms.contains(&"Deloria"), "tenant Deloria absent");
    assert!(
        noms.contains(&"Résidence Test"),
        "tenant « Résidence Test » absent — c'est lui qui rend vérifiable que rien dans le socle \
         ne suppose l'existence d'un point de vente"
    );
}

/// L'établissement de « Résidence Test » n'a **aucun point de vente**, et c'est le sujet.
///
/// À ce cycle, la vérification est structurelle : aucune table de point de vente n'existe encore
/// (elles viennent d'ETB-03). Le test constate donc l'invariant réellement disponible — le second
/// tenant existe, avec son établissement, et le socle fonctionne pour lui exactement comme pour le
/// premier. ETB-03 durcira ce test en vérifiant l'absence de ligne de point de vente.
#[tokio::test]
async fn le_second_tenant_fonctionne_sans_point_de_vente() {
    executer_seeds(0);
    let pool = commun::pool_app().await;

    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, TENANT_RESIDENCE_TEST)
        .await
        .expect("pose du tenant");

    let etablissements: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "c!" FROM etablissements.etablissement"#
    )
    .fetch_one(&mut *tx)
    .await
    .expect("comptage");

    assert_eq!(
        etablissements, 1,
        "« Résidence Test » doit voir exactement son établissement, et rien d'autre"
    );

    // Et il ne voit pas celui de Deloria — l'isolation vaut aussi pour les données seedées.
    let deloria: Option<Uuid> = sqlx::query_scalar!(
        "SELECT id FROM etablissements.etablissement WHERE tenant_id = $1",
        TENANT_DELORIA
    )
    .fetch_optional(&mut *tx)
    .await
    .expect("lecture croisée");

    assert!(
        deloria.is_none(),
        "« Résidence Test » voit l'établissement de Deloria : les seeds ont contourné la sécurité \
         au niveau ligne — probablement écrits sous le rôle propriétaire"
    );

    tx.rollback().await.expect("rollback");
}

/// Exécute le binaire de seeds.
fn executer_seeds(passage: u32) {
    let sortie = Command::new(env!("CARGO"))
        .args(["run", "--quiet", "-p", "kaya-api", "--bin", "seeds"])
        .output()
        .expect("exécution du binaire de seeds");

    assert!(
        sortie.status.success(),
        "passage {passage} : les seeds ont échoué\n{}",
        String::from_utf8_lossy(&sortie.stderr)
    );
}

/// État seedé : les tenants, leur nombre d'établissements, **et leur parc**.
///
/// Le parc est entré dans l'état comparé au cycle 004. Sans lui, les trois exécutions auraient pu
/// dupliquer 17 unités sans que rien ne le dise : le décompte des établissements, lui, serait resté
/// à 1.
///
/// Le cycle 006 y a ajouté les **fiches clientes et les séjours**, pour la même raison : trois
/// exécutions auraient pu dupliquer douze fiches et quatre séjours sans que le décompte des unités
/// ne bouge d'une ligne.
///
/// Trié, pour que la comparaison ne dépende pas de l'ordre de lecture.
async fn lire_etat(pool: &sqlx::PgPool) -> Vec<(Uuid, String, i64, i64, i64, i64, i64)> {
    let mut etat = Vec::new();

    for tenant_id in [TENANT_DELORIA, TENANT_RESIDENCE_TEST] {
        let mut tx = pool.begin().await.expect("transaction");
        kaya_etablissements::tenant_context::poser_tenant(&mut tx, tenant_id)
            .await
            .expect("pose du tenant");

        let ligne = sqlx::query!(
            "SELECT nom FROM etablissements.tenant WHERE id = $1",
            tenant_id
        )
        .fetch_optional(&mut *tx)
        .await
        .expect("lecture du tenant");

        let Some(ligne) = ligne else {
            tx.rollback().await.expect("rollback");
            continue;
        };

        let etablissements: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "c!" FROM etablissements.etablissement WHERE tenant_id = $1"#,
            tenant_id
        )
        .fetch_one(&mut *tx)
        .await
        .expect("comptage des établissements");

        let unites: i64 = sqlx::query_scalar!(r#"SELECT COUNT(*) AS "c!" FROM hebergement.unite"#)
            .fetch_one(&mut *tx)
            .await
            .expect("comptage des unités");

        let formules: i64 =
            sqlx::query_scalar!(r#"SELECT COUNT(*) AS "c!" FROM hebergement.formule"#)
                .fetch_one(&mut *tx)
                .await
                .expect("comptage des formules");

        let clients: i64 = sqlx::query_scalar!(r#"SELECT COUNT(*) AS "c!" FROM comptes.client"#)
            .fetch_one(&mut *tx)
            .await
            .expect("comptage des fiches clientes");

        let sejours: i64 = sqlx::query_scalar!(r#"SELECT COUNT(*) AS "c!" FROM hebergement.sejour"#)
            .fetch_one(&mut *tx)
            .await
            .expect("comptage des séjours");

        etat.push((
            tenant_id,
            ligne.nom,
            etablissements,
            unites,
            formules,
            clients,
            sejours,
        ));
        tx.rollback().await.expect("rollback");
    }

    etat.sort();
    etat
}

// =================================================================================================
//  Cycle 002 — la configuration exacte des deux tenants
// =================================================================================================

/// **Deloria porte cinq services actifs, dont deux seulement suivent leur stock.**
///
/// Le décompte est asserté valeur par valeur plutôt que globalement : un seed où tout serait activé
/// partout ne prouverait rien du refus ni de l'absence, et c'est précisément ce que le jeu de
/// données doit démontrer.
#[tokio::test]
async fn deloria_porte_cinq_services_et_stock_sur_deux_seulement() {
    let pool = commun::pool_owner().await;
    executer_seeds(1);

    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, TENANT_DELORIA)
        .await
        .expect("pose du tenant");

    let services: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT em.module_code
        FROM etablissements.etablissement_module em
        WHERE em.etablissement_id = $1 AND em.actif
        ORDER BY em.module_code
        "#,
    )
    .bind(ETABLISSEMENT_DELORIA)
    .fetch_all(&mut *tx)
    .await
    .expect("lecture des services");

    assert_eq!(
        services,
        vec!["BAR", "HEBERGEMENT", "PRESSING", "RESTAURATION", "SALLE_REUNION"],
        "Deloria doit porter les CINQ services du référentiel"
    );

    let avec_stock: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT em.module_code
        FROM etablissements.module_capacite mc
        JOIN etablissements.etablissement_module em ON em.id = mc.etablissement_module_id
        WHERE em.etablissement_id = $1 AND mc.capacite_code = 'STOCK'
        ORDER BY em.module_code
        "#,
    )
    .bind(ETABLISSEMENT_DELORIA)
    .fetch_all(&mut *tx)
    .await
    .expect("lecture des capacités");

    assert_eq!(
        avec_stock,
        vec!["BAR", "RESTAURATION"],
        "seuls les deux services qui vendent des articles stockés déclarent STOCK — hypothèse 9 \
         de la spécification. Un seed qui l'activerait partout ne démontrerait plus rien."
    );

    // Deux points de vente, dont **un comptoir** : le jeu de données porte les deux formes.
    let tables_par_pdv: Vec<(String, i64)> = sqlx::query_as(
        r#"
        SELECT pdv.nom, COUNT(t.id)
        FROM etablissements.point_de_vente pdv
        LEFT JOIN etablissements.table_pdv t ON t.point_de_vente_id = pdv.id AND t.actif
        WHERE pdv.etablissement_id = $1
        GROUP BY pdv.nom
        ORDER BY pdv.nom
        "#,
    )
    .bind(ETABLISSEMENT_DELORIA)
    .fetch_all(&mut *tx)
    .await
    .expect("lecture des points de vente");

    assert_eq!(
        tables_par_pdv,
        vec![
            ("Comptoir du bar".to_owned(), 0),
            ("Salle du restaurant".to_owned(), 3),
        ],
        "le jeu de données doit porter un COMPTOIR (zéro table) et un point de vente à tables — \
         sans les deux formes, rien ne distingue l'un de l'autre à la démonstration"
    );

    tx.rollback().await.expect("rollback");
}

/// **Résidence Test : HEBERGEMENT seul, aucune capacité, aucun point de vente.**
///
/// C'est la moitié la plus structurante du jeu de données. Ajouter ici un point de vente « pour
/// faire complet » détruirait la seule preuve que le socle n'en suppose aucun.
#[tokio::test]
async fn residence_test_n_a_qu_un_service_et_rien_d_autre() {
    let pool = commun::pool_owner().await;
    executer_seeds(1);

    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, TENANT_RESIDENCE_TEST)
        .await
        .expect("pose du tenant");

    let services: Vec<String> = sqlx::query_scalar(
        "SELECT module_code FROM etablissements.etablissement_module WHERE etablissement_id = $1",
    )
    .bind(ETABLISSEMENT_RESIDENCE_TEST)
    .fetch_all(&mut *tx)
    .await
    .expect("lecture des services");

    assert_eq!(services, vec!["HEBERGEMENT"], "un seul service, et c'est HEBERGEMENT");

    for (table, requete) in [
        (
            "module_capacite",
            r#"SELECT COUNT(*) FROM etablissements.module_capacite mc
               JOIN etablissements.etablissement_module em ON em.id = mc.etablissement_module_id
               WHERE em.etablissement_id = $1"#,
        ),
        (
            "point_de_vente",
            "SELECT COUNT(*) FROM etablissements.point_de_vente WHERE etablissement_id = $1",
        ),
    ] {
        let compte: i64 = sqlx::query_scalar(requete)
            .bind(ETABLISSEMENT_RESIDENCE_TEST)
            .fetch_one(&mut *tx)
            .await
            .expect("comptage");
        assert_eq!(
            compte, 0,
            "Résidence Test porte {compte} ligne(s) dans `{table}` alors qu'elle ne doit en avoir \
             aucune. C'est ce jeu de données qui rend vérifiable qu'aucun crate partagé ne suppose \
             l'existence d'un point de vente ni d'un stock."
        );
    }

    // Identité : résidence meublée, à Abidjan.
    let (classement, commune): (String, String) = sqlx::query_as(
        "SELECT classement, commune FROM etablissements.etablissement WHERE id = $1",
    )
    .bind(ETABLISSEMENT_RESIDENCE_TEST)
    .fetch_one(&mut *tx)
    .await
    .expect("lecture de l'identité");
    assert_eq!(classement, "RESIDENCE_MEUBLEE");
    assert_eq!(commune, "Abidjan");

    tx.rollback().await.expect("rollback");
}

/// **L'identité déclarée par le seed est RÉELLEMENT appliquée.**
///
/// Le défaut que ce test attrape s'est produit : la migration `0007` a ajouté `commune` avec un
/// `DEFAULT ''` retiré aussitôt, les lignes existantes ont reçu une chaîne vide, et le
/// `ON CONFLICT DO NOTHING` du seed les y a laissées. L'écran `G1` affichait une commune vide sur
/// le tenant du pilote, alors que le seed déclarait « Abengourou » deux lignes plus haut.
///
/// **Un seed qui n'applique pas les valeurs qu'il déclare donne un état faux**, et personne ne
/// pense à le soupçonner : le fichier dit la bonne chose.
#[tokio::test]
async fn le_seed_applique_reellement_l_identite_qu_il_declare() {
    let pool = commun::pool_owner().await;

    // On dégrade volontairement la ligne, comme l'aurait fait une migration additive.
    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, TENANT_DELORIA)
        .await
        .expect("pose du tenant");
    sqlx::query("UPDATE etablissements.etablissement SET commune = '' WHERE id = $1")
        .bind(ETABLISSEMENT_DELORIA)
        .execute(&mut *tx)
        .await
        .expect("dégradation");
    tx.commit().await.expect("commit");

    executer_seeds(1);

    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, TENANT_DELORIA)
        .await
        .expect("pose du tenant");
    let commune: String =
        sqlx::query_scalar("SELECT commune FROM etablissements.etablissement WHERE id = $1")
            .bind(ETABLISSEMENT_DELORIA)
            .fetch_one(&mut *tx)
            .await
            .expect("relecture");
    tx.rollback().await.expect("rollback");

    assert_eq!(
        commune, "Abengourou",
        "le seed n'a pas réappliqué la commune qu'il déclare. Recharger la démonstration doit \
         restituer EXACTEMENT l'état décrit dans `seeds.rs` — sinon le fichier dit une chose et la \
         base en montre une autre."
    );
}

/// **Adjoua porte trois rôles, et c'est le point du cycle 003.**
///
/// Un jeu de données où chacun n'aurait qu'un rôle ne démontrerait rien de l'union des
/// permissions. Ce test fige la figure que le cadrage décrit : dans un établissement de cette
/// taille, la même personne tient la réception le matin, la caisse le soir, et gère l'équipe entre
/// les deux — sur **une seule connexion**, dans **une seule application**.
///
/// Il vérifie aussi ce qui n'est pas là : aucun compte de démonstration ne porte `admin_editeur`.
/// Ce rôle est de portée `EDITEUR` et n'appartient à aucun tenant ; le seeder par commodité
/// donnerait au pilote un accès qu'il ne doit pas avoir, et le premier test d'isolation écrit
/// dessus le figerait.
#[tokio::test]
async fn adjoua_porte_les_trois_roles_et_personne_n_est_admin_editeur() {
    let pool = commun::pool_owner().await;

    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, TENANT_DELORIA)
        .await
        .expect("pose du tenant");

    let roles: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT cr.role_code
        FROM comptes.compte_role cr
        JOIN comptes.compte c   ON c.id = cr.compte_id
        JOIN comptes.personne p ON p.id = c.personne_id
        WHERE p.prenoms = 'Adjoua'
        ORDER BY cr.role_code
        "#,
    )
    .fetch_all(&mut *tx)
    .await
    .expect("lecture des rôles d'Adjoua");

    assert_eq!(
        roles,
        vec![
            "caissier".to_owned(),
            "gerant".to_owned(),
            "receptionniste".to_owned()
        ],
        "Adjoua doit porter les trois rôles — c'est la figure que US3 démontre. Obtenu : {roles:?}"
    );

    let admins: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM comptes.compte_role WHERE role_code = 'admin_editeur'",
    )
    .fetch_one(&mut *tx)
    .await
    .expect("comptage des admin_editeur");

    assert_eq!(
        admins, 0,
        "un compte de démonstration porte `admin_editeur`. Ce rôle est de portée EDITEUR et \
         n'appartient à aucun tenant : le seeder par commodité donnerait au pilote un accès qu'il \
         ne doit pas avoir."
    );

    tx.rollback().await.expect("rollback");
}

// =================================================================================================
//  Cycle 004 — le parc des deux tenants, et l'offre disjointe qui l'éprouve
// =================================================================================================

/// **17 unités en 5 catégories, plus la salle de réunion — et la salle est une CATÉGORIE.**
///
/// Le décompte est asserté par catégorie et non globalement : `17` se retrouve de plusieurs
/// manières, et une répartition fausse passerait un total juste. La ligne qui compte le plus est la
/// dernière — la salle de réunion apparaît comme une sixième catégorie à une unité, jamais comme
/// une entité d'un autre genre.
#[tokio::test]
async fn deloria_porte_dix_sept_unites_et_la_salle_en_sixieme_categorie() {
    let pool = commun::pool_owner().await;
    executer_seeds(1);

    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, TENANT_DELORIA)
        .await
        .expect("pose du tenant");

    let parc: Vec<(String, i64)> = sqlx::query_as(
        r#"
        SELECT c.nom, COUNT(u.id)
        FROM hebergement.categorie c
        LEFT JOIN hebergement.unite u ON u.categorie_id = c.id
        WHERE c.etablissement_id = $1
        GROUP BY c.nom
        ORDER BY c.nom
        "#,
    )
    .bind(ETABLISSEMENT_DELORIA)
    .fetch_all(&mut *tx)
    .await
    .expect("lecture du parc");

    assert_eq!(
        parc,
        vec![
            ("Classique".to_owned(), 5),
            ("Classique supérieure".to_owned(), 4),
            ("Salle de réunion".to_owned(), 1),
            ("Standard".to_owned(), 3),
            ("Supérieure A".to_owned(), 2),
            ("Supérieure B".to_owned(), 3),
        ],
        "la répartition du cadrage §2.1 doit être seedée à la ligne près — et la salle de réunion \
         doit être une CATÉGORIE, jamais une entité nouvelle"
    );

    let chambres: i64 = parc
        .iter()
        .filter(|(nom, _)| nom != "Salle de réunion")
        .map(|(_, n)| n)
        .sum();
    assert_eq!(chambres, 17, "17 unités louables, plus la salle");

    tx.rollback().await.expect("rollback");
}

/// **La nuitée est assujettie, le passage et la demi-journée ne le sont pas.**
///
/// Ce n'est pas une règle fiscale — le crate stocke ces deux colonnes et ne les interprète jamais
/// (P-12) — c'est le **paramétrage du pilote**, et c'est ce qui doit être vérifiable : une nuitée
/// seedée non assujettie ferait disparaître la taxe communale du jeu de démonstration sans qu'aucun
/// test ne bronche.
///
/// La règle est `une_nuitee_par_occupation` : un séjour de trois nuits vaut 500 F, pas 3 × 500.
#[tokio::test]
async fn seule_la_nuitee_est_assujettie_a_la_taxe_de_nuitee() {
    let pool = commun::pool_owner().await;
    executer_seeds(1);

    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, TENANT_DELORIA)
        .await
        .expect("pose du tenant");

    let familles: Vec<(String, bool, Option<String>, i64)> = sqlx::query_as(
        r#"
        SELECT famille, bool_and(assujettie_taxe_nuitee), max(regle_conversion_taxe), COUNT(*)
        FROM hebergement.formule
        WHERE etablissement_id = $1
        GROUP BY famille
        ORDER BY famille
        "#,
    )
    .bind(ETABLISSEMENT_DELORIA)
    .fetch_all(&mut *tx)
    .await
    .expect("lecture des formules");

    assert_eq!(
        familles,
        vec![
            (
                "DEMI_JOURNEE".to_owned(),
                false,
                None,
                1
            ),
            (
                "NUITEE".to_owned(),
                true,
                Some("une_nuitee_par_occupation".to_owned()),
                5
            ),
            ("PASSAGE".to_owned(), false, None, 5),
        ],
        "seule la NUITÉE est assujettie, avec `une_nuitee_par_occupation` — trois nuits valent \
         500 F, pas 3 × 500. Le passage et la demi-journée ne le sont pas : constat d'exploitation \
         du pilote, et B-02 tranchera la valeur par défaut légale, jamais l'existence du paramètre."
    );

    // Le barème du cadrage §5.3, palier par palier, sur chacune des cinq catégories.
    let paliers: Vec<(i32, i64, i64)> = sqlx::query_as(
        r#"
        SELECT p.duree_minutes, p.prix_mineur, COUNT(*)
        FROM hebergement.bareme_palier p
        JOIN hebergement.formule f ON f.id = p.formule_id
        WHERE f.etablissement_id = $1
        GROUP BY p.duree_minutes, p.prix_mineur
        ORDER BY p.duree_minutes
        "#,
    )
    .bind(ETABLISSEMENT_DELORIA)
    .fetch_all(&mut *tx)
    .await
    .expect("lecture des paliers");

    assert_eq!(
        paliers,
        vec![
            (60, 1_500, 5),
            (120, 2_800, 5),
            (180, 4_000, 5),
            (240, 5_000, 5)
        ],
        "le barème du cadrage §5.3 — provisoire jusqu'à B-07 — doit être seedé sur les cinq \
         catégories de chambres"
    );

    tx.rollback().await.expect("rollback");
}

/// **Résidence Test : quatre meublés, MOIS et NUITÉE — et AUCUN passage.**
///
/// C'est le test qui donne sa raison d'être au second tenant. Deux promesses y sont éprouvées, et
/// aucune des deux ne l'est par Deloria seule :
///
/// * **aucune formule n'est réservée à un type d'établissement** — les deux tenants portent des
///   offres disjointes sur les mêmes tables ;
/// * **aucun code ne suppose l'existence du passage** — ni formule, ni barème, ni plage ici. Un
///   chemin qui exigerait un barème pour lire une offre échouerait à cet endroit, et nulle part
///   ailleurs.
#[tokio::test]
async fn residence_test_vend_au_mois_et_a_la_nuitee_sans_aucun_passage() {
    let pool = commun::pool_owner().await;
    executer_seeds(1);

    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, TENANT_RESIDENCE_TEST)
        .await
        .expect("pose du tenant");

    let unites: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM hebergement.unite WHERE etablissement_id = $1")
            .bind(ETABLISSEMENT_RESIDENCE_TEST)
            .fetch_one(&mut *tx)
            .await
            .expect("comptage des unités");
    assert_eq!(unites, 4, "Résidence Test porte quatre meublés");

    let familles: Vec<String> = sqlx::query_scalar(
        "SELECT famille FROM hebergement.formule WHERE etablissement_id = $1 ORDER BY famille",
    )
    .bind(ETABLISSEMENT_RESIDENCE_TEST)
    .fetch_all(&mut *tx)
    .await
    .expect("lecture des formules");

    assert_eq!(
        familles,
        vec!["MENSUEL".to_owned(), "NUITEE".to_owned()],
        "mois et nuitée SEULEMENT. Le mensuel sur une résidence et le passage sur un hôtel sont \
         deux offres disjointes portées par les mêmes tables : c'est ce qui rend vérifiable \
         qu'aucune formule n'est réservée à un type d'établissement (cadrage §5.2)."
    );

    for (table, requete) in [
        (
            "bareme_palier",
            r#"SELECT COUNT(*) FROM hebergement.bareme_palier p
               JOIN hebergement.formule f ON f.id = p.formule_id
               WHERE f.etablissement_id = $1"#,
        ),
        (
            "plage_demi_journee",
            r#"SELECT COUNT(*) FROM hebergement.plage_demi_journee pl
               JOIN hebergement.formule f ON f.id = pl.formule_id
               WHERE f.etablissement_id = $1"#,
        ),
    ] {
        let compte: i64 = sqlx::query_scalar(requete)
            .bind(ETABLISSEMENT_RESIDENCE_TEST)
            .fetch_one(&mut *tx)
            .await
            .expect("comptage");
        assert_eq!(
            compte, 0,
            "Résidence Test porte {compte} ligne(s) dans `{table}` alors qu'elle ne vend ni \
             passage ni demi-journée. C'est cette absence qui éprouve qu'aucun code ne les suppose."
        );
    }

    tx.rollback().await.expect("rollback");
}

/// **Le condensat n'est jamais celui du code.**
///
/// Trois choses vérifiées d'un coup : le mot de passe vient de l'environnement (le condensat est
/// au format PHC d'Argon2id, donc réellement haché), les paramètres sont ceux d'OWASP, et **deux
/// comptes au même mot de passe ont deux condensats** — sans quoi une simple comparaison de la
/// colonne dirait qui partage son mot de passe avec qui.
#[tokio::test]
async fn les_condensats_sont_argon2id_et_tous_differents() {
    let pool = commun::pool_owner().await;

    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, TENANT_DELORIA)
        .await
        .expect("pose du tenant");

    let condensats: Vec<String> =
        sqlx::query_scalar("SELECT condensat_mot_de_passe FROM comptes.compte ORDER BY id")
            .fetch_all(&mut *tx)
            .await
            .expect("lecture des condensats");

    assert_eq!(condensats.len(), 3, "trois comptes attendus sur Deloria");

    for condensat in &condensats {
        assert!(
            condensat.starts_with("$argon2id$"),
            "condensat qui n'est pas de l'Argon2id : {condensat}"
        );
        assert!(
            condensat.contains("m=19456,t=2,p=1"),
            "les paramètres OWASP doivent voyager avec le condensat : {condensat}"
        );
    }

    let distincts: std::collections::BTreeSet<&String> = condensats.iter().collect();
    assert_eq!(
        distincts.len(),
        condensats.len(),
        "deux comptes partagent le même condensat : le sel n'est pas aléatoire, et une \
         comparaison de la colonne dirait qui partage son mot de passe avec qui"
    );

    tx.rollback().await.expect("rollback");
}

// =================================================================================================
//  Cycle 006 — les fiches clientes et les séjours de démonstration
// =================================================================================================

/// **Douze fiches sur Deloria, deux sur Résidence Test — et le personnel n'en fait pas partie.**
///
/// La seconde moitié est celle qui compte. Une fiche client **est une personne qualifiée** par
/// `comptes.client`, jamais une seconde identité : si le seed qualifiait aussi les trois comptes du
/// pilote — ou si la recherche lisait `personne` sans passer par `client` — Koffi, Adjoua et Yao
/// apparaîtraient dans les résultats du comptoir. Le défaut ne se verrait qu'à la démonstration.
#[tokio::test]
async fn douze_fiches_clientes_sur_deloria_et_aucun_membre_du_personnel() {
    let pool = commun::pool_owner().await;
    executer_seeds(1);

    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, TENANT_DELORIA)
        .await
        .expect("pose du tenant");

    let fiches: i64 = sqlx::query_scalar!(r#"SELECT COUNT(*) AS "c!" FROM comptes.client"#)
        .fetch_one(&mut *tx)
        .await
        .expect("comptage des fiches");
    assert_eq!(
        fiches, 12,
        "douze fiches — assez pour que la recherche par nom rende plusieurs résultats et que la \
         troncature se voie"
    );

    // ★ Aucun compte n'est qualifié client.
    let personnel_qualifie: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) AS "c!"
        FROM comptes.client cl
        JOIN comptes.compte c ON c.personne_id = cl.personne_id
        "#
    )
    .fetch_one(&mut *tx)
    .await
    .expect("comptage croisé");

    assert_eq!(
        personnel_qualifie, 0,
        "un membre du personnel est qualifié client : il apparaîtra dans la recherche du comptoir. \
         Une fiche client EST une personne qualifiée, jamais une seconde identité — et c'est la \
         qualification, pas la table `personne`, qui décide de ce que la recherche rend."
    );

    // Le repli des diacritiques est celui du produit : « Kone » doit retrouver « Koné ».
    let replis: Vec<String> = sqlx::query_scalar!(
        // ⚠️ **Jointure sur `comptes.client`, et ce n'est pas une précaution de style.** Adjoua
        // N'Guessan est un membre du PERSONNEL, homonyme de la cliente Affoué N'Guessan : sa
        // ligne `personne` n'a pas de `nom_repli` — le repli sert la recherche cliente, et le
        // personnel n'y figure pas. Sans la jointure, le test lit sa colonne nulle et échoue sur
        // un message qui accuse le seed.
        r#"SELECT p.nom_repli AS "nom_repli!"
           FROM comptes.personne p
           JOIN comptes.client c ON c.personne_id = p.id
           WHERE p.nom IN ('Koné', 'N''Guessan')
           ORDER BY p.nom"#
    )
    .fetch_all(&mut *tx)
    .await
    .expect("lecture des replis");
    assert_eq!(
        replis,
        vec!["kone".to_owned(), "nguessan".to_owned()],
        "le seed doit employer la MÊME fonction de repli que le produit. Deux replis divergents \
         rendraient introuvables des fiches pourtant créées, et la divergence ne se verrait qu'à \
         la recherche."
    );

    tx.rollback().await.expect("rollback");
}

/// ★ **Le séjour clos porte son constat FIGÉ — et le montant y est `NULL`.**
///
/// C'est le seul endroit du jeu de données qui rend visible, **en base et sans lire une ligne de
/// code**, que ce cycle a laissé le calcul à FIS-03 : il fige des **faits** — nuits, personnes,
/// période — et un **paramétrage recopié**, sans dériver aucune assiette.
///
/// Un seed qui aurait « rempli » `nuitees_assujetties` et `montant_mineur` pour faire complet
/// aurait fait croire à un calcul qui n'existe pas, et le premier écran qui les lirait aurait
/// affiché un montant de taxe inventé.
#[tokio::test]
async fn le_sejour_clos_porte_un_constat_fige_dont_le_montant_est_nul() {
    let pool = commun::pool_owner().await;
    executer_seeds(1);

    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, TENANT_DELORIA)
        .await
        .expect("pose du tenant");

    let constats: Vec<(i32, i32, bool, Option<i32>, Option<i64>, String)> = sqlx::query_as(
        r#"
        SELECT nuits_constatees, nombre_personnes, assujettie_taxe_nuitee,
               nuitees_assujetties, montant_mineur, commune
        FROM hebergement.taxe_sejour_constat
        WHERE sejour_id = ANY($1)
        "#,
    )
    .bind(SEJOURS_SEEDES.as_slice())
    .fetch_all(&mut *tx)
    .await
    .expect("lecture des constats");

    assert_eq!(
        constats.len(),
        1,
        "un seul séjour est clos, donc un seul constat : le figeage a lieu AU DÉPART, jamais avant"
    );
    let (nuits, personnes, assujettie, nuitees, montant, commune) = &constats[0];
    assert_eq!(*nuits, 2, "deux nuits constatées");
    assert!(*personnes >= 1);
    assert!(*assujettie, "la nuitée de Deloria est assujettie");

    assert!(
        nuitees.is_none() && montant.is_none(),
        "le constat porte un montant ou un nombre de nuitées assujetties. Ce cycle fige des FAITS \
         et un paramétrage RECOPIÉ ; décider quelles nuits sont assujetties est une règle fiscale, \
         et elle ne vit que dans `JurisdictionAdapter` (principe V, porte P-12). Obtenu : \
         nuitees={nuitees:?}, montant={montant:?}"
    );

    assert_eq!(
        commune, "Abengourou",
        "la commune est RECOPIÉE dans le constat, jamais référencée : il doit rester vrai le jour \
         où l'établissement en change, parce qu'un contrôle fiscal porte sur ce qui était vrai ce \
         jour-là"
    );

    // Et la note du séjour clos est **arrêtée**, pas simplement fermée.
    let arretees: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "c!" FROM hebergement.note_sejour
           WHERE statut = 'arretee' AND sejour_id = ANY($1)"#,
        SEJOURS_SEEDES.as_slice(),
    )
    .fetch_one(&mut *tx)
    .await
    .expect("comptage des notes arrêtées");
    assert_eq!(arretees, 1, "la note du séjour clos doit être arrêtée");

    tx.rollback().await.expect("rollback");
}

/// Les quatre séjours **seedés**, littéralement — les mêmes identifiants que `seeds.rs`.
///
/// ⚠️ **Les assertions de ce fichier portent sur EUX, jamais sur « tous les séjours du tenant ».**
/// La base de développement est partagée avec le e2e, qui **crée des séjours** : une assertion sur
/// un décompte global rougirait après chaque exécution de P-22 ou de la démo, pour une raison sans
/// rapport avec les seeds. Un test des seeds doit parler des seeds.
const SEJOURS_SEEDES: [Uuid; 4] = [
    uuid!("0198c4a0-0000-7000-8000-000000000411"),
    uuid!("0198c4a0-0000-7000-8000-000000000412"),
    uuid!("0198c4a0-0000-7000-8000-000000000413"),
    uuid!("0198c4a0-0000-7000-8000-000000000461"),
];

/// **Trois séjours seedés sur Deloria, dont un passage SANS fiche cliente.**
///
/// Le passage sans fiche est ce qui rend la démonstration honnête : la pièce vient **après** la
/// clé (FR-023), et sa fiche de police est **numérotée et déclarée incomplète** — jamais fabriquée
/// avec « M. X », qui serait un document légal faux.
///
/// La numérotation est vérifiée **continue** : un trou ne se verrait qu'au contrôle.
#[tokio::test]
async fn trois_sejours_dont_un_passage_sans_fiche_et_une_numerotation_continue() {
    let pool = commun::pool_owner().await;
    executer_seeds(1);

    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, TENANT_DELORIA)
        .await
        .expect("pose du tenant");

    let statuts: Vec<(String, i64)> = sqlx::query_as(
        "SELECT statut, COUNT(*) FROM hebergement.sejour WHERE id = ANY($1) \
         GROUP BY statut ORDER BY statut",
    )
    .bind(SEJOURS_SEEDES.as_slice())
    .fetch_all(&mut *tx)
    .await
    .expect("lecture des séjours");
    assert_eq!(
        statuts,
        vec![("clos".to_owned(), 1), ("en_cours".to_owned(), 2)],
        "deux séjours en cours et un clos — sans le clos, rien ne montre le constat figé"
    );

    let sans_client: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "c!" FROM hebergement.sejour
           WHERE client_id IS NULL AND id = ANY($1)"#,
        SEJOURS_SEEDES.as_slice(),
    )
    .fetch_one(&mut *tx)
    .await
    .expect("comptage");
    assert_eq!(
        sans_client, 1,
        "un séjour SANS fiche cliente : la pièce vient après la clé (FR-023), et un jeu de données \
         où tout le monde aurait sa fiche cacherait le parcours réel du passage"
    );

    // La fiche de ce séjour existe, est numérotée, et est **déclarée** incomplète.
    let incompletes: Vec<i64> = sqlx::query_scalar!(
        r#"SELECT numero FROM hebergement.fiche_police
           WHERE NOT complete AND sejour_id = ANY($1) ORDER BY numero"#,
        SEJOURS_SEEDES.as_slice(),
    )
    .fetch_all(&mut *tx)
    .await
    .expect("lecture des fiches incomplètes");
    assert_eq!(
        incompletes.len(),
        1,
        "la fiche du passage sans client doit EXISTER, numérotée, et être déclarée incomplète — \
         ni omise, ni fabriquée"
    );

    // ★ Numérotation CONTINUE par établissement, sans trou.
    let numeros: Vec<i64> = sqlx::query_scalar!(
        r#"SELECT numero FROM hebergement.fiche_police
           WHERE sejour_id = ANY($1) ORDER BY numero"#,
        SEJOURS_SEEDES.as_slice(),
    )
    .fetch_all(&mut *tx)
    .await
    .expect("lecture des numéros");
    assert_eq!(
        numeros,
        vec![1, 2, 3],
        "la numérotation des fiches de police est CONTINUE par établissement, sans trou. Une \
         `SEQUENCE` en laisserait, et le trou ne se verrait qu'au contrôle."
    );

    // Deux accompagnants, donc trois personnes sur le séjour de nuitée.
    let accompagnants: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "c!" FROM hebergement.accompagnant
           WHERE retire_le IS NULL AND sejour_id = ANY($1)"#,
        SEJOURS_SEEDES.as_slice(),
    )
    .fetch_one(&mut *tx)
    .await
    .expect("comptage des accompagnants");
    assert_eq!(accompagnants, 2, "deux accompagnants sur le séjour de nuitée");

    tx.rollback().await.expect("rollback");
}

/// **Résidence Test porte son propre séjour, et ne voit pas ceux de Deloria.**
///
/// L'isolation des données seedées vaut aussi pour les séjours. Un test d'isolation qui n'aurait
/// qu'un tenant peuplé ne distinguerait pas « la politique RLS filtre » de « l'autre tenant est
/// vide » — c'est la raison d'être des deux fiches et de l'unique séjour de ce tenant.
#[tokio::test]
async fn residence_test_porte_son_sejour_et_ne_voit_pas_ceux_de_deloria() {
    let pool = commun::pool_app().await;
    executer_seeds(1);

    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, TENANT_RESIDENCE_TEST)
        .await
        .expect("pose du tenant");

    let sejours: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "c!" FROM hebergement.sejour WHERE id = ANY($1)"#,
        SEJOURS_SEEDES.as_slice(),
    )
    .fetch_one(&mut *tx)
    .await
    .expect("comptage");
    assert_eq!(
        sejours, 1,
        "Résidence Test doit voir exactement son séjour seedé, et rien des trois de Deloria — \
         l'isolation vaut aussi pour les données de démonstration"
    );

    let fiches: i64 = sqlx::query_scalar!(r#"SELECT COUNT(*) AS "c!" FROM comptes.client"#)
        .fetch_one(&mut *tx)
        .await
        .expect("comptage");
    assert_eq!(fiches, 2, "deux fiches clientes sur Résidence Test");

    tx.rollback().await.expect("rollback");
}
