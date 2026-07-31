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
//! démonstration — mais par des **identifiants fixes** et des `ON CONFLICT DO NOTHING`. Chaque
//! ligne seedée a un UUID écrit en dur ci-dessous ; rejouer ne fait donc rien.
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

/// Tenant du pilote.
///
/// Les colonnes `classement`, `commune`, `ncc` et `adresse` du cadrage §2.1 **n'existent pas
/// encore** : `etablissement` est en forme minimale à ce cycle et ETB-01 l'enrichira par migration
/// additive. Ce qui peut être seedé aujourd'hui l'est ; le reste est décrit dans le README des
/// seeds plutôt qu'inventé ici.
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
            (id, tenant_id, nom, fuseau_horaire, devise)
        VALUES ($1, $2, 'Résidence Hôtel Deloria — Abengourou', 'Africa/Abidjan', 'XOF')
        ON CONFLICT (id) DO NOTHING
        "#,
        ETABLISSEMENT_DELORIA,
        TENANT_DELORIA
    )
    .execute(&mut *tx)
    .await?;

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
            (id, tenant_id, nom, fuseau_horaire, devise)
        VALUES ($1, $2, 'Résidence Test — hébergement seul', 'Africa/Abidjan', 'XOF')
        ON CONFLICT (id) DO NOTHING
        "#,
        ETABLISSEMENT_RESIDENCE_TEST,
        TENANT_RESIDENCE_TEST
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    tracing::info!(tenant = %TENANT_RESIDENCE_TEST, "tenant Résidence Test seedé");
    Ok(())
}
