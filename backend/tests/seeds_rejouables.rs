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

    let noms: Vec<&str> = etats[0].iter().map(|(_, nom, _)| nom.as_str()).collect();
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

/// État seedé : les tenants et leur nombre d'établissements.
///
/// Trié, pour que la comparaison ne dépende pas de l'ordre de lecture.
async fn lire_etat(pool: &sqlx::PgPool) -> Vec<(Uuid, String, i64)> {
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

        etat.push((tenant_id, ligne.nom, etablissements));
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
