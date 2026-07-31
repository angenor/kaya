//! Immuabilité du grand livre — **les trois couches, vérifiées séparément** (R-05).
//!
//! Chaque couche arrête une faute différente, et aucune ne suffit seule :
//!
//! | Couche | Ce qu'elle arrête | Vérifiée par |
//! |---|---|---|
//! | Privilèges (`REVOKE`) | Le bug applicatif | [`sous_le_role_applicatif_update_et_delete_sont_refuses`] |
//! | Déclencheur | **La migration ou le script lancé sous le propriétaire** | [`sous_le_role_proprietaire_update_et_delete_sont_encore_refuses`] |
//! | Porte de CI | Le code écrit pour purger | `scripts/ci/outbox-sans-purge.sh` |
//!
//! **Le second test est celui qui compte.** Le premier ne fait que constater un `REVOKE`. Le
//! second reproduit le scénario réel : un développeur solo connecté en production à 23 h pour
//! « corriger une ligne », sous le rôle qui possède les tables. C'est là que la rétention
//! illimitée de TRX-02 se perd ou se tient.

mod commun;

use uuid::Uuid;

/// Couche 1 — le rôle du runtime n'a physiquement pas le droit de modifier.
#[tokio::test]
async fn sous_le_role_applicatif_update_et_delete_sont_refuses() {
    let pool_owner = commun::pool_owner().await;
    let (tenant, evenement_id) = semer(&pool_owner, "immuabilité — rôle applicatif").await;

    let pool = commun::pool_app().await;

    // --- UPDATE d'une colonne autre que `publie_le` -------------------------------------------
    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, tenant.tenant_id)
        .await
        .expect("pose du tenant");
    let resultat = sqlx::query!(
        "UPDATE synchronisation.evenement_outbox SET payload = '{}'::jsonb WHERE id = $1",
        evenement_id
    )
    .execute(&mut *tx)
    .await;
    assert!(
        resultat.is_err(),
        "kaya_app a pu réécrire la charge utile d'un événement : le grand livre n'est pas immuable"
    );
    let _ = tx.rollback().await;

    // --- DELETE --------------------------------------------------------------------------------
    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, tenant.tenant_id)
        .await
        .expect("pose du tenant");
    let resultat = sqlx::query!(
        "DELETE FROM synchronisation.evenement_outbox WHERE id = $1",
        evenement_id
    )
    .execute(&mut *tx)
    .await;
    assert!(
        resultat.is_err(),
        "kaya_app a pu supprimer un événement : la rétention illimitée n'est pas garantie"
    );
    let _ = tx.rollback().await;
}

/// Couche 2 — **le déclencheur s'applique aussi au propriétaire des tables**.
///
/// C'est le second essai qui compte : le premier ne teste qu'un `REVOKE`, celui-ci teste ce qui
/// protège de la maintenance.
#[tokio::test]
async fn sous_le_role_proprietaire_update_et_delete_sont_encore_refuses() {
    let pool = commun::pool_owner().await;
    let (tenant, evenement_id) = semer(&pool, "immuabilité — rôle propriétaire").await;

    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, tenant.tenant_id)
        .await
        .expect("pose du tenant");
    let resultat = sqlx::query!(
        "UPDATE synchronisation.evenement_outbox SET payload = '{}'::jsonb WHERE id = $1",
        evenement_id
    )
    .execute(&mut *tx)
    .await;
    assert!(
        resultat.is_err(),
        "le PROPRIÉTAIRE des tables a pu réécrire un événement. Les privilèges ne le contraignent \
         pas — seul le déclencheur le peut, et il ne joue pas son rôle."
    );
    let _ = tx.rollback().await;

    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, tenant.tenant_id)
        .await
        .expect("pose du tenant");
    let resultat = sqlx::query!(
        "DELETE FROM synchronisation.evenement_outbox WHERE id = $1",
        evenement_id
    )
    .execute(&mut *tx)
    .await;
    assert!(
        resultat.is_err(),
        "le PROPRIÉTAIRE des tables a pu supprimer un événement — le scénario du développeur solo \
         connecté en production à 23 h"
    );
    let _ = tx.rollback().await;
}

/// La **seule** mutation tolérée : `publie_le` de `NULL` vers une valeur, **une seule fois**.
#[tokio::test]
async fn le_marquage_de_publication_passe_une_fois_et_une_seule() {
    let pool_owner = commun::pool_owner().await;
    let (tenant, evenement_id) = semer(&pool_owner, "immuabilité — marquage").await;

    let pool = commun::pool_app().await;

    // --- Premier marquage : accepté ------------------------------------------------------------
    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, tenant.tenant_id)
        .await
        .expect("pose du tenant");
    sqlx::query!(
        "UPDATE synchronisation.evenement_outbox SET publie_le = now() WHERE id = $1",
        evenement_id
    )
    .execute(&mut *tx)
    .await
    .expect("le marquage de publication doit être accepté : sans lui, il faudrait une seconde \
             table de marquage, donc une jointure de plus sur le chemin du grand livre");
    tx.commit().await.expect("commit");

    // --- Second marquage : refusé --------------------------------------------------------------
    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, tenant.tenant_id)
        .await
        .expect("pose du tenant");
    let resultat = sqlx::query!(
        "UPDATE synchronisation.evenement_outbox SET publie_le = now() WHERE id = $1",
        evenement_id
    )
    .execute(&mut *tx)
    .await;
    assert!(
        resultat.is_err(),
        "la publication a pu être rejouée. Elle est MONOTONE : NULL vers une valeur, une fois."
    );
    let _ = tx.rollback().await;

    // --- Retour à NULL : refusé ----------------------------------------------------------------
    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, tenant.tenant_id)
        .await
        .expect("pose du tenant");
    let resultat = sqlx::query!(
        "UPDATE synchronisation.evenement_outbox SET publie_le = NULL WHERE id = $1",
        evenement_id
    )
    .execute(&mut *tx)
    .await;
    assert!(
        resultat.is_err(),
        "publie_le a pu revenir à NULL : un événement publié pourrait être republié \
         indéfiniment, et les consommateurs verraient des effets déjà produits"
    );
    let _ = tx.rollback().await;
}

/// Le worker non plus ne peut ni supprimer, ni réécrire.
///
/// Le rôle `kaya_worker` (migration 0005) lit tous les tenants : c'est le rôle le plus large du
/// produit sur cette table. Vérifier l'immuabilité sous lui n'est donc pas redondant — c'est le
/// cas où elle serait le plus facilement perdue.
#[tokio::test]
async fn sous_le_role_worker_seul_le_marquage_passe() {
    let pool_owner = commun::pool_owner().await;
    let (_tenant, evenement_id) = semer(&pool_owner, "immuabilité — worker").await;

    let pool = commun::pool_worker().await;

    let resultat = sqlx::query!(
        "DELETE FROM synchronisation.evenement_outbox WHERE id = $1",
        evenement_id
    )
    .execute(&pool)
    .await;
    assert!(resultat.is_err(), "kaya_worker a pu supprimer un événement");

    let resultat = sqlx::query!(
        "UPDATE synchronisation.evenement_outbox SET payload = '{}'::jsonb WHERE id = $1",
        evenement_id
    )
    .execute(&pool)
    .await;
    assert!(
        resultat.is_err(),
        "kaya_worker a pu réécrire la charge utile — il ne doit toucher que publie_le"
    );

    let marquage = sqlx::query!(
        "UPDATE synchronisation.evenement_outbox SET publie_le = now() WHERE id = $1",
        evenement_id
    )
    .execute(&pool)
    .await;
    assert!(
        marquage.is_ok(),
        "kaya_worker doit pouvoir marquer la publication, c'est sa seule raison d'être : {:?}",
        marquage.err()
    );
}

/// Sème un événement et renvoie le tenant et l'identifiant.
async fn semer(pool: &sqlx::PgPool, nom: &str) -> (commun::JeuTenant, Uuid) {
    let tenant = commun::creer_tenant(pool, nom).await;
    let evenement_id = Uuid::now_v7();

    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, tenant.tenant_id)
        .await
        .expect("pose du tenant");

    let sequence: i64 = sqlx::query_scalar!(
        "SELECT synchronisation.prochaine_sequence($1)",
        tenant.etablissement_id
    )
    .fetch_one(&mut *tx)
    .await
    .expect("séquence")
    .unwrap_or_default();

    sqlx::query!(
        r#"
        INSERT INTO synchronisation.evenement_outbox (
            id, tenant_id, etablissement_id, sequence_etablissement,
            type_evenement, agregat, agregat_id, version_schema, payload, survenu_le
        )
        VALUES ($1, $2, $3, $4, 'test.immuabilite', 'test', $5, 1, '{"a": 1}'::jsonb, now())
        "#,
        evenement_id,
        tenant.tenant_id,
        tenant.etablissement_id,
        sequence,
        Uuid::now_v7(),
    )
    .execute(&mut *tx)
    .await
    .expect("seed de l'événement");

    tx.commit().await.expect("commit");
    (tenant, evenement_id)
}
