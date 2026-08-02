//! **Le test que la migration `0016` promettait, et qui n'existait pas.**
//!
//! `0016_roles_permissions.sql` écrit, à propos de `permission.module_code` : « **Aucune clé
//! étrangère vers `etablissements.module_activite`** : ce serait une clé inter-schémas, interdite
//! par le principe II (porte P-04). La cohérence est tenue par un test qui lit le référentiel des
//! modules **à travers le trait `RegistreModules`** et échoue si une permission nomme un module
//! inconnu. »
//!
//! Ce test n'a jamais été écrit. Le cycle 003 pouvait s'en passer sans conséquence visible : ses
//! dix-sept permissions portaient toutes `module_code = NULL`, et **une porte dont la cible est
//! vide passe toujours**. Le cycle 004 apporte les cinq premières permissions rattachées à un
//! module — c'est donc le premier moment où l'absence coûterait quelque chose.
//!
//! # Deux requêtes, jamais une jointure
//!
//! La vérification compare deux ensembles qui vivent dans **deux schémas de modules différents** :
//! `comptes.permission` et `etablissements.module_activite`. Les joindre serait exactement ce que
//! le principe II interdit et ce que la porte P-04 attrape — **y compris dans un test**, puisque
//! son périmètre couvre `backend/tests/`.
//!
//! Les deux lectures sont donc distinctes, et la comparaison se fait en Rust. Le référentiel des
//! modules passe par `kaya_etablissements::modules::repository::referentiel_modules`, la fonction
//! que le trait et l'API consomment — pas par une requête réécrite ici, qui pourrait diverger.
//!
//! # Ce que la porte vérifie, et ce qu'elle refuse de vérifier
//!
//! Elle vérifie qu'un `module_code` non nul **désigne un module du référentiel**. Elle ne vérifie
//! pas qu'il désigne le *bon* : `heb.offre.lire` pourrait porter `RESTAURATION` et passer. Cette
//! justesse-là est métier, et la prétendre automatisée produirait une porte qui ment.

mod commun;

use std::collections::BTreeSet;

/// Le nombre de permissions rattachées à un module, **relu de la base** et comparé à ce que le
/// cycle 004 déclare livrer.
///
/// Sans ce décompte, retirer les cinq permissions du cycle laisserait le test vert : il n'aurait
/// plus rien à valider. C'est la différence entre « aucune permission ne nomme un module inconnu »
/// et « aucune permission ne nomme de module du tout ».
const PERMISSIONS_DE_MODULE_ATTENDUES: i64 = 5;

/// Les cinq permissions du cycle HEB, nommées.
///
/// Elles sont écrites ici **en plus** du décompte : un décompte seul passerait au vert si l'on
/// remplaçait `heb.unite.attribuer` par une sixième permission d'un autre module.
const PERMISSIONS_HEBERGEMENT: &[&str] = &[
    "heb.offre.lire",
    "heb.offre.gerer",
    "heb.disponibilite.consulter",
    "heb.unite.attribuer",
    "heb.unite.liberer",
];

/// Le module que ces cinq permissions servent.
const MODULE_ATTENDU: &str = "HEBERGEMENT";

#[actix_web::test]
async fn aucune_permission_ne_nomme_un_module_inconnu() {
    let pool = commun::pool_owner().await;

    // ── Lecture 1 : les permissions rattachées à un module ────────────────────────────────────
    let rattachees: Vec<(String, String)> = sqlx::query_as(
        r#"
        SELECT code, module_code
        FROM comptes.permission
        WHERE module_code IS NOT NULL
        ORDER BY ordre
        "#,
    )
    .fetch_all(&pool)
    .await
    .expect("lecture des permissions");

    // ── Lecture 2 : le référentiel des modules, par la fonction que l'API consomme ────────────
    let mut tx = pool.begin().await.expect("transaction");
    let modules = kaya_etablissements::modules::repository::referentiel_modules(&mut tx)
        .await
        .expect("lecture du référentiel des modules");
    tx.rollback().await.expect("rollback");

    let codes_connus: BTreeSet<String> = modules.into_iter().map(|m| m.code).collect();

    assert!(
        !codes_connus.is_empty(),
        "le référentiel des modules est vide : ce test n'a rien contre quoi comparer, et son vert \
         ne dirait rien. Vérifier que la migration 0008 est appliquée."
    );

    // ── La comparaison, en Rust, jamais en SQL ────────────────────────────────────────────────
    let inconnues: Vec<String> = rattachees
        .iter()
        .filter(|(_, module)| !codes_connus.contains(module))
        .map(|(code, module)| format!("{code} → « {module} »"))
        .collect();

    assert!(
        inconnues.is_empty(),
        "{} permission(s) nomment un module absent du référentiel :\n  {}\n\n\
         `permission.module_code` n'a AUCUNE clé étrangère vers `etablissements.module_activite` \
         — ce serait une clé inter-schémas (principe II, porte P-04). C'est ce test qui tient la \
         cohérence, et il vient de la constater rompue.\n\
         Modules connus : {:?}",
        inconnues.len(),
        inconnues.join("\n  "),
        codes_connus
    );
}

/// **La cible n'est pas vide** — et c'est la première fois qu'on peut l'affirmer.
///
/// La migration `0016` écrit que `module_code` « restera `NULL` jusqu'au cycle HEB, qui apportera
/// `heb.unite.attribuer` ». Ce test vérifie que la phrase a été honorée à la lettre.
#[actix_web::test]
async fn les_cinq_permissions_du_cycle_heb_sont_rattachees_au_module() {
    let pool = commun::pool_owner().await;

    let rattachees: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM comptes.permission WHERE module_code IS NOT NULL",
    )
    .fetch_one(&pool)
    .await
    .expect("décompte des permissions de module");

    assert_eq!(
        rattachees, PERMISSIONS_DE_MODULE_ATTENDUES,
        "{rattachees} permission(s) rattachées à un module, {PERMISSIONS_DE_MODULE_ATTENDUES} \
         attendues. Un écart n'est pas à résorber en ajustant cette constante : il se justifie à \
         l'endroit où il se constate."
    );

    for code in PERMISSIONS_HEBERGEMENT {
        let module: Option<String> = sqlx::query_scalar(
            "SELECT module_code FROM comptes.permission WHERE code = $1",
        )
        .bind(code)
        .fetch_optional(&pool)
        .await
        .expect("lecture de la permission")
        .flatten();

        assert_eq!(
            module.as_deref(),
            Some(MODULE_ATTENDU),
            "la permission « {code} » devrait porter module_code = « {MODULE_ATTENDU} »"
        );
    }
}

/// Le réceptionniste attribue des chambres, il ne fixe pas les tarifs.
///
/// Écrit comme un test et non comme un commentaire de migration parce que c'est la **seule**
/// exception de la distribution, et qu'une exception non testée se perd à la première reprise de
/// la table `role_permission`.
#[actix_web::test]
async fn le_receptionniste_a_tout_sauf_la_gestion_de_l_offre() {
    let pool = commun::pool_owner().await;

    let siennes: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT p.code
        FROM comptes.role_permission rp
        JOIN comptes.permission p ON p.code = rp.permission_code
        WHERE rp.role_code = 'receptionniste'
          AND p.module_code = $1
        ORDER BY p.ordre
        "#,
    )
    .bind(MODULE_ATTENDU)
    .fetch_all(&pool)
    .await
    .expect("lecture des permissions du réceptionniste");

    let siennes: BTreeSet<&str> = siennes.iter().map(String::as_str).collect();

    assert!(
        siennes.contains("heb.unite.attribuer") && siennes.contains("heb.unite.liberer"),
        "le réceptionniste doit pouvoir attribuer et libérer une unité — sinon la permission \
         annoncée par le cycle 003 ne garde rien (FR-021). Obtenu : {siennes:?}"
    );
    assert!(
        !siennes.contains("heb.offre.gerer"),
        "le réceptionniste ne fixe pas les tarifs : `heb.offre.gerer` ne doit pas lui être \
         attribuée. Obtenu : {siennes:?}"
    );
    assert_eq!(
        siennes.len(),
        PERMISSIONS_HEBERGEMENT.len() - 1,
        "quatre permissions sur cinq attendues pour le réceptionniste. Obtenu : {siennes:?}"
    );
}

/// **Propriétaire et gérant ont les cinq**, chacun.
#[actix_web::test]
async fn le_proprietaire_et_le_gerant_ont_les_cinq() {
    let pool = commun::pool_owner().await;

    for role in ["proprietaire", "gerant"] {
        let nombre: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM comptes.role_permission rp
            JOIN comptes.permission p ON p.code = rp.permission_code
            WHERE rp.role_code = $1 AND p.module_code = $2
            "#,
        )
        .bind(role)
        .bind(MODULE_ATTENDU)
        .fetch_one(&pool)
        .await
        .expect("décompte");

        assert_eq!(
            nombre,
            PERMISSIONS_DE_MODULE_ATTENDUES,
            "le rôle « {role} » devrait porter les {PERMISSIONS_DE_MODULE_ATTENDUES} permissions \
             d'hébergement"
        );
    }
}
