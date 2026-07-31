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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if dotenvy::from_path("backend/.env").is_err() {
        let _ = dotenvy::dotenv();
    }
    kaya_api::observabilite::initialiser_journaux();

    let pool = db::pool_application().await?;

    seeder_deloria(&pool).await?;
    seeder_residence_test(&pool).await?;

    println!("Seeds appliqués. Deux tenants :");
    println!("  Deloria         {TENANT_DELORIA}  (établissement {ETABLISSEMENT_DELORIA})");
    println!(
        "  Résidence Test  {TENANT_RESIDENCE_TEST}  (établissement {ETABLISSEMENT_RESIDENCE_TEST})"
    );
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
