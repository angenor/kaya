//! Données de démonstration — **rejouables**.
//!
//! ```sh
//! cargo run -p kaya-api --bin seeds
//! cargo run -p kaya-api --bin seeds     # même état final
//! ```
//!
//! # Trois propriétés, et ce qui les tient
//!
//! **Rejouable.** Trois exécutions successives produisent le même état final. Ce n'est pas obtenu
//! par un `DELETE` préalable — qui détruirait les données de travail du pilote à chaque
//! démonstration — mais par des **identifiants fixes**. Chaque ligne seedée a un UUID écrit en dur
//! ci-dessous.
//!
//! # `DO UPDATE` sur l'identité, `DO NOTHING` sur le reste — la distinction compte
//!
//! Les **colonnes d'identité** des deux établissements de démonstration sont réappliquées à chaque
//! exécution. Ce sont des valeurs de référence, pas des données de travail : « recharger la
//! démonstration » doit restituer **exactement** l'état décrit ici.
//!
//! Le cycle 002 a montré pourquoi. La migration `0007` a ajouté `commune` avec un `DEFAULT ''`
//! retiré aussitôt ; les lignes existantes ont donc reçu une chaîne vide, et un `DO NOTHING` les y
//! aurait laissées **pour toujours** — l'écran `G1` affichait une commune vide sur le tenant du
//! pilote, alors que le seed déclarait « Abengourou » deux lignes plus haut. Un seed qui n'applique
//! pas les valeurs qu'il déclare donne un état faux, et personne ne pense à le soupçonner.
//!
//! Les **activations, capacités, points de vente et tables** gardent `DO NOTHING` : leur unicité
//! porte sur le couple qui les définit, et les réécrire à l'identique ne changerait rien.
//!
//! **Séparé des migrations** (principe I(b)). Une migration décrit le schéma et n'est jamais
//! rejouée ; un seed décrit un jeu de données et l'est constamment. Les mêler rendrait impossible
//! de recharger une démonstration sans toucher au schéma.
//!
//! **Sous le rôle applicatif.** Les seeds passent par `kaya_app`, soumis à la sécurité au niveau
//! ligne, et posent le contexte de tenant comme le ferait l'application. Les écrire sous
//! `kaya_owner` contournerait ce que le reste du cycle cherche à garantir — et un jeu de données
//! seedé hors politique serait invisible depuis l'application.
//!
//! # Portée réduite, assumée
//!
//! Ce cycle livre **la mécanique et les deux tenants**. Les 17 unités, les 30 articles et les
//! 5 comptes de test de FR-062 peuplent des tables qui n'existent pas encore — elles viennent des
//! cycles HEB, PDV et CPT. Ce qu'ils devront contenir est écrit dans
//! `backend/migrations/seeds/README.md`, pour que chaque cycle sache ce qu'il doit y ajouter.

use kaya_api::db;
use kaya_etablissements::tenant_context;
use sqlx::PgPool;
use uuid::{Uuid, uuid};

// =================================================================================================
//  Identifiants FIXES — c'est eux qui rendent le seed rejouable
// =================================================================================================
//
// Écrits en dur, jamais tirés au hasard. Un `Uuid::now_v7()` produirait un nouveau jeu à chaque
// exécution : la base grossirait sans fin et « recharger la démonstration » créerait un troisième
// établissement au lieu de retrouver le premier.

/// Tenant du pilote — Résidence Hôtel Deloria, Abengourou (cadrage §2.1).
const TENANT_DELORIA: Uuid = uuid!("0198c4a0-0000-7000-8000-000000000001");
const ETABLISSEMENT_DELORIA: Uuid = uuid!("0198c4a0-0000-7000-8000-000000000002");

/// Second tenant — **module hébergement seul, aucun point de vente**.
const TENANT_RESIDENCE_TEST: Uuid = uuid!("0198c4a0-0000-7000-8000-000000000011");
const ETABLISSEMENT_RESIDENCE_TEST: Uuid = uuid!("0198c4a0-0000-7000-8000-000000000012");

/// Les cinq services de Deloria, identifiants fixes.
const SERVICES_DELORIA: [(Uuid, &str); 5] = [
    (uuid!("0198c4a0-0000-7000-8000-000000000021"), "HEBERGEMENT"),
    (uuid!("0198c4a0-0000-7000-8000-000000000022"), "RESTAURATION"),
    (uuid!("0198c4a0-0000-7000-8000-000000000023"), "BAR"),
    (uuid!("0198c4a0-0000-7000-8000-000000000024"), "PRESSING"),
    (uuid!("0198c4a0-0000-7000-8000-000000000025"), "SALLE_REUNION"),
];

/// **`STOCK` au profil `SIMPLE`, déclarée par RESTAURATION et BAR seulement.**
///
/// Ce sont les deux services qui vendent des articles stockés — hypothèse 9 de la spécification,
/// révisable sans coût avant le cycle STK. `HEBERGEMENT`, `PRESSING` et `SALLE_REUNION` n'en
/// déclarent aucune, et **c'est ce qui rend le jeu de données représentatif** : un seed où tout
/// est activé partout ne prouverait rien du refus ni de l'absence.
const CAPACITES_DELORIA: [(Uuid, &str); 2] = [
    (uuid!("0198c4a0-0000-7000-8000-000000000031"), "RESTAURATION"),
    (uuid!("0198c4a0-0000-7000-8000-000000000032"), "BAR"),
];

/// Les deux points de vente de Deloria. Le second n'a **aucune table** : c'est un comptoir.
const POINTS_DE_VENTE_DELORIA: [(Uuid, &str, &str); 2] = [
    (
        uuid!("0198c4a0-0000-7000-8000-000000000041"),
        "RESTAURATION",
        "Salle du restaurant",
    ),
    (
        uuid!("0198c4a0-0000-7000-8000-000000000042"),
        "BAR",
        "Comptoir du bar",
    ),
];

/// Les tables de la salle du restaurant. Le comptoir du bar n'en a aucune.
const TABLES_DELORIA: [(Uuid, &str); 3] = [
    (uuid!("0198c4a0-0000-7000-8000-000000000051"), "1"),
    (uuid!("0198c4a0-0000-7000-8000-000000000052"), "2"),
    (uuid!("0198c4a0-0000-7000-8000-000000000053"), "Terrasse"),
];

/// Le service HEBERGEMENT de Résidence Test — **le seul**, et sans capacité.
const SERVICE_RESIDENCE_TEST: Uuid = uuid!("0198c4a0-0000-7000-8000-000000000061");

// -------------------------------------------------------------------------------------------
//  CPT — les trois personnes du pilote, et le cumul de rôles d'Adjoua
// -------------------------------------------------------------------------------------------
//
// **Adjoua porte les trois rôles, et c'est tout le point du cycle.** Un jeu de données où chacun
// n'aurait qu'un rôle ne démontrerait rien de l'union des permissions : c'est exactement la
// situation que le cadrage décrit — dans un établissement de cette taille, la même personne tient
// la réception le matin, la caisse le soir et gère l'équipe entre les deux.
//
// Yao n'a qu'un rôle, et M. Koffi est propriétaire : les trois ensemble donnent trois accueils
// différents sur la même application, ce que l'écran `R1` doit montrer.

/// M. Koffi — propriétaire de Deloria (cadrage §2.1).
const PERSONNE_KOFFI: Uuid = uuid!("0198c4a0-0000-7000-8000-000000000071");
const COMPTE_KOFFI: Uuid = uuid!("0198c4a0-0000-7000-8000-000000000072");

/// Adjoua — **gérante, caissière ET réceptionniste**.
const PERSONNE_ADJOUA: Uuid = uuid!("0198c4a0-0000-7000-8000-000000000073");
const COMPTE_ADJOUA: Uuid = uuid!("0198c4a0-0000-7000-8000-000000000074");

/// Yao — réceptionniste.
const PERSONNE_YAO: Uuid = uuid!("0198c4a0-0000-7000-8000-000000000075");
const COMPTE_YAO: Uuid = uuid!("0198c4a0-0000-7000-8000-000000000076");

/// Les attributions de rôles, identifiants fixes — `(id, compte, rôle)`.
///
/// L'établissement est toujours celui de Deloria : les huit rôles sauf `admin_editeur` sont de
/// portée `ETABLISSEMENT` et en exigent un.
const ROLES_DELORIA: [(Uuid, Uuid, &str); 5] = [
    (
        uuid!("0198c4a0-0000-7000-8000-000000000081"),
        COMPTE_KOFFI,
        "proprietaire",
    ),
    (
        uuid!("0198c4a0-0000-7000-8000-000000000082"),
        COMPTE_ADJOUA,
        "gerant",
    ),
    (
        uuid!("0198c4a0-0000-7000-8000-000000000083"),
        COMPTE_ADJOUA,
        "caissier",
    ),
    (
        uuid!("0198c4a0-0000-7000-8000-000000000084"),
        COMPTE_ADJOUA,
        "receptionniste",
    ),
    (
        uuid!("0198c4a0-0000-7000-8000-000000000085"),
        COMPTE_YAO,
        "receptionniste",
    ),
];

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if dotenvy::from_path("backend/.env").is_err() {
        let _ = dotenvy::dotenv();
    }
    kaya_api::observabilite::initialiser_journaux();

    // **Refus d'exécution en production**, avant toute connexion (T005). La garde vit dans le
    // binaire et non dans le script d'appel : un script se contourne d'une ligne de commande, et
    // c'est bien le binaire qu'on lance à la main un soir d'incident en cherchant à « juste
    // remettre les données de démonstration ».
    let mot_de_passe = kaya_api::secrets::mot_de_passe_seeds()?;

    let pool = db::pool_application().await?;

    seeder_deloria(&pool).await?;
    seeder_residence_test(&pool).await?;
    seeder_comptes_deloria(&pool, &mot_de_passe).await?;

    println!("Seeds appliqués. Deux tenants :");
    println!("  Deloria         {TENANT_DELORIA}  (établissement {ETABLISSEMENT_DELORIA})");
    println!(
        "  Résidence Test  {TENANT_RESIDENCE_TEST}  (établissement {ETABLISSEMENT_RESIDENCE_TEST})"
    );
    println!();
    println!("Trois comptes sur Deloria — le mot de passe vient de KAYA_SEEDS_MOT_DE_PASSE :");
    println!("  koffi@deloria.test    propriétaire");
    println!("  adjoua@deloria.test   gérante + caissière + réceptionniste  ← le cumul");
    println!("  yao@deloria.test      réceptionniste");
    println!();
    println!("Rejouable : une seconde exécution laisse exactement le même état.");

    Ok(())
}

/// Tenant du pilote — **identité complète depuis ETB-01**.
///
/// Le cycle 001 ne pouvait seeder que le nom, le fuseau et la devise : `etablissement` était en
/// forme minimale. La migration `0007_etablissement_identite.sql` a livré les sept colonnes
/// d'identité, et `commune` est `NOT NULL` **sans défaut** — une création qui l'omettrait est
/// désormais refusée, ce qui est exactement le but.
async fn seeder_deloria(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let mut tx = pool.begin().await?;
    tenant_context::poser_tenant(&mut tx, TENANT_DELORIA).await?;

    sqlx::query!(
        r#"
        INSERT INTO etablissements.tenant (id, nom)
        VALUES ($1, 'Deloria')
        ON CONFLICT (id) DO NOTHING
        "#,
        TENANT_DELORIA
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!(
        r#"
        INSERT INTO etablissements.etablissement
            (id, tenant_id, nom, fuseau_horaire, devise,
             juridiction, classement, etoiles, commune, adresse)
        VALUES ($1, $2, 'Résidence Hôtel Deloria — Abengourou', 'Africa/Abidjan', 'XOF',
                'CI', 'NON_CLASSE', NULL, 'Abengourou', NULL)
        ON CONFLICT (id) DO UPDATE
        SET nom = EXCLUDED.nom,
            fuseau_horaire = EXCLUDED.fuseau_horaire,
            devise = EXCLUDED.devise,
            juridiction = EXCLUDED.juridiction,
            classement = EXCLUDED.classement,
            etoiles = EXCLUDED.etoiles,
            commune = EXCLUDED.commune,
            adresse = EXCLUDED.adresse
        "#,
        ETABLISSEMENT_DELORIA,
        TENANT_DELORIA
    )
    .execute(&mut *tx)
    .await?;

    // ── Cinq services actifs ────────────────────────────────────────────────────────────────
    //
    // `ON CONFLICT DO NOTHING` sur l'identifiant **et** sur le couple (établissement, module) :
    // la seconde contrainte est celle qui compte, un module ne s'activant qu'une fois par
    // établissement.
    for (id, code) in SERVICES_DELORIA {
        sqlx::query!(
            r#"
            INSERT INTO etablissements.etablissement_module
                (id, tenant_id, etablissement_id, module_code, module_implemente)
            VALUES ($1, $2, $3, $4, true)
            ON CONFLICT (etablissement_id, module_code) DO NOTHING
            "#,
            id,
            TENANT_DELORIA,
            ETABLISSEMENT_DELORIA,
            code,
        )
        .execute(&mut *tx)
        .await?;
    }

    // ── STOCK/SIMPLE sur RESTAURATION et BAR ────────────────────────────────────────────────
    for (id, module_code) in CAPACITES_DELORIA {
        let service_id = SERVICES_DELORIA
            .iter()
            .find(|(_, code)| *code == module_code)
            .map(|(id, _)| *id)
            .expect("le service qui déclare la capacité doit figurer dans SERVICES_DELORIA");

        sqlx::query!(
            r#"
            INSERT INTO etablissements.module_capacite
                (id, tenant_id, etablissement_module_id,
                 capacite_code, capacite_implementee, profil_code, profil_implemente)
            VALUES ($1, $2, $3, 'STOCK', true, 'SIMPLE', true)
            ON CONFLICT (etablissement_module_id, capacite_code) DO NOTHING
            "#,
            id,
            TENANT_DELORIA,
            service_id,
        )
        .execute(&mut *tx)
        .await?;
    }

    // ── Deux points de vente, dont un COMPTOIR ──────────────────────────────────────────────
    for (id, module_code, nom) in POINTS_DE_VENTE_DELORIA {
        let service_id = SERVICES_DELORIA
            .iter()
            .find(|(_, code)| *code == module_code)
            .map(|(id, _)| *id)
            .expect("le service du point de vente doit figurer dans SERVICES_DELORIA");

        sqlx::query!(
            r#"
            INSERT INTO etablissements.point_de_vente
                (id, tenant_id, etablissement_id, etablissement_module_id, nom)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (id) DO NOTHING
            "#,
            id,
            TENANT_DELORIA,
            ETABLISSEMENT_DELORIA,
            service_id,
            nom,
        )
        .execute(&mut *tx)
        .await?;
    }

    // Les tables de la salle. **Le comptoir du bar n'en reçoit aucune** — c'est ce qui en fait un
    // comptoir, et le jeu de données porte donc les deux formes.
    let salle = POINTS_DE_VENTE_DELORIA[0].0;
    for (id, libelle) in TABLES_DELORIA {
        sqlx::query!(
            r#"
            INSERT INTO etablissements.table_pdv (id, tenant_id, point_de_vente_id, libelle)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (point_de_vente_id, libelle) DO NOTHING
            "#,
            id,
            TENANT_DELORIA,
            salle,
            libelle,
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    tracing::info!(tenant = %TENANT_DELORIA, "tenant Deloria seedé");
    Ok(())
}

/// Second tenant — **la raison d'être de ce seed n'est pas la démonstration**.
///
/// « Résidence Test » porte le **module hébergement seul, sans aucun point de vente**. C'est ce
/// qui rend vérifiable la promesse la plus structurante du produit :
///
/// > Aucun crate partagé ne suppose qu'un établissement possède de l'hébergement, ni qu'il
/// > possède un point de vente (constitution, préambule).
///
/// Un jeu de données à un seul tenant complet laisserait cette promesse invérifiable jusqu'au
/// premier client maquis — c'est-à-dire jusqu'au moment où la corriger coûterait une refonte.
/// Il sert aussi de second tenant aux tests d'isolation.
async fn seeder_residence_test(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let mut tx = pool.begin().await?;
    tenant_context::poser_tenant(&mut tx, TENANT_RESIDENCE_TEST).await?;

    sqlx::query!(
        r#"
        INSERT INTO etablissements.tenant (id, nom)
        VALUES ($1, 'Résidence Test')
        ON CONFLICT (id) DO NOTHING
        "#,
        TENANT_RESIDENCE_TEST
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!(
        r#"
        INSERT INTO etablissements.etablissement
            (id, tenant_id, nom, fuseau_horaire, devise,
             juridiction, classement, etoiles, commune, adresse)
        VALUES ($1, $2, 'Résidence Test — hébergement seul', 'Africa/Abidjan', 'XOF',
                'CI', 'RESIDENCE_MEUBLEE', NULL, 'Abidjan', NULL)
        ON CONFLICT (id) DO UPDATE
        SET nom = EXCLUDED.nom,
            fuseau_horaire = EXCLUDED.fuseau_horaire,
            devise = EXCLUDED.devise,
            juridiction = EXCLUDED.juridiction,
            classement = EXCLUDED.classement,
            etoiles = EXCLUDED.etoiles,
            commune = EXCLUDED.commune,
            adresse = EXCLUDED.adresse
        "#,
        ETABLISSEMENT_RESIDENCE_TEST,
        TENANT_RESIDENCE_TEST
    )
    .execute(&mut *tx)
    .await?;

    // **HEBERGEMENT seul, AUCUNE capacité, AUCUN point de vente.**
    //
    // C'est la moitié la plus structurante du jeu de données : un établissement qui ne porte
    // qu'un service et rien d'autre doit être pleinement exploitable. Ajouter ici un point de
    // vente « pour faire complet » détruirait la seule preuve que le socle n'en suppose aucun.
    sqlx::query!(
        r#"
        INSERT INTO etablissements.etablissement_module
            (id, tenant_id, etablissement_id, module_code, module_implemente)
        VALUES ($1, $2, $3, 'HEBERGEMENT', true)
        ON CONFLICT (etablissement_id, module_code) DO NOTHING
        "#,
        SERVICE_RESIDENCE_TEST,
        TENANT_RESIDENCE_TEST,
        ETABLISSEMENT_RESIDENCE_TEST,
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    tracing::info!(tenant = %TENANT_RESIDENCE_TEST, "tenant Résidence Test seedé");
    Ok(())
}

/// **Les trois comptes du pilote** — CPT-00, CPT-01, CPT-02.
///
/// # Le mot de passe vient de l'environnement, jamais du code
///
/// `KAYA_SEEDS_MOT_DE_PASSE`. Un mot de passe littéral ici vivrait dans le dépôt, dans l'image et
/// dans les archives de tous les postes ayant cloné le projet — et il finirait employé sur un
/// serveur de démonstration joignable depuis internet.
///
/// # Le condensat est recalculé à chaque exécution, et ce n'est PAS une non-idempotence
///
/// Argon2 tire un sel aléatoire : deux exécutions produisent deux condensats différents pour le
/// même mot de passe. C'est exactement ce qu'on veut, et c'est pourquoi l'`INSERT` porte
/// `ON CONFLICT (id) DO NOTHING` **et non `DO UPDATE`** : la ligne existante n'est pas réécrite,
/// donc l'état final est identique à la troisième exécution comme à la première.
///
/// La distinction avec l'identité des établissements — qui, elle, est réappliquée par `DO UPDATE`
/// — tient en une phrase : une commune est une **valeur de référence** que le seed déclare, un
/// condensat est une **donnée de travail** dont la valeur exacte n'a pas d'importance.
///
/// # `DO NOTHING` sur les rôles, et pourquoi c'est le couple qui compte
///
/// L'unicité de `compte_role` porte sur `(compte_id, role_code, etablissement_id)` avec
/// `NULLS NOT DISTINCT`. Le conflit se résout donc sur ce couple, pas sur l'identifiant : un rôle
/// réattribué à l'identique ne crée pas de seconde ligne, même si l'on changeait son UUID.
async fn seeder_comptes_deloria(
    pool: &PgPool,
    mot_de_passe: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut tx = pool.begin().await?;
    tenant_context::poser_tenant(&mut tx, TENANT_DELORIA).await?;

    // `(personne, compte, nom, prénoms, identifiant)`
    let gens = [
        (
            PERSONNE_KOFFI,
            COMPTE_KOFFI,
            "Koffi",
            Some("Yao Bernard"),
            "koffi@deloria.test",
        ),
        (
            PERSONNE_ADJOUA,
            COMPTE_ADJOUA,
            "N'Guessan",
            Some("Adjoua"),
            "adjoua@deloria.test",
        ),
        (PERSONNE_YAO, COMPTE_YAO, "Kouassi", Some("Yao"), "yao@deloria.test"),
    ];

    for (personne_id, compte_id, nom, prenoms, identifiant) in gens {
        sqlx::query!(
            r#"
            INSERT INTO comptes.personne (id, tenant_id, nom, prenoms)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (id) DO UPDATE
            SET nom = EXCLUDED.nom,
                prenoms = EXCLUDED.prenoms,
                modifie_le = now()
            "#,
            personne_id,
            TENANT_DELORIA,
            nom,
            prenoms,
        )
        .execute(&mut *tx)
        .await?;

        // Le condensat n'est calculé que si le compte n'existe pas — un hachage Argon2 coûte
        // 19 Mio et des dizaines de millisecondes, et le recalculer à chaque exécution pour le
        // jeter aussitôt serait du travail pur.
        let deja_present: bool = sqlx::query_scalar!(
            r#"SELECT EXISTS (SELECT 1 FROM comptes.compte WHERE id = $1) AS "existe!""#,
            compte_id
        )
        .fetch_one(&mut *tx)
        .await?;

        if !deja_present {
            let condensat = kaya_comptes::authentification::hacher(mot_de_passe)?;

            sqlx::query!(
                r#"
                INSERT INTO comptes.compte
                    (id, tenant_id, personne_id, identifiant_email, condensat_mot_de_passe)
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT (id) DO NOTHING
                "#,
                compte_id,
                TENANT_DELORIA,
                personne_id,
                identifiant,
                condensat,
            )
            .execute(&mut *tx)
            .await?;
        }
    }

    // ── Le cumul ────────────────────────────────────────────────────────────────────────────
    //
    // `attribue_par_compte_id` désigne M. Koffi, y compris pour son propre rôle de propriétaire.
    // C'est une convention de seed, pas une règle : dans le produit, le premier propriétaire est
    // provisionné par l'éditeur (ETB-08). L'écrire ainsi évite une colonne nullable qui
    // signifierait « attribué par personne » et qu'il faudrait traiter partout.
    for (id, compte_id, role_code) in ROLES_DELORIA {
        sqlx::query!(
            r#"
            INSERT INTO comptes.compte_role
                (id, tenant_id, compte_id, role_code, etablissement_id, attribue_par_compte_id)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (compte_id, role_code, etablissement_id) DO NOTHING
            "#,
            id,
            TENANT_DELORIA,
            compte_id,
            role_code,
            ETABLISSEMENT_DELORIA,
            COMPTE_KOFFI,
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    tracing::info!(
        tenant = %TENANT_DELORIA,
        comptes = 3,
        roles = ROLES_DELORIA.len(),
        "comptes et rôles du pilote seedés — Adjoua en porte trois"
    );
    Ok(())
}

