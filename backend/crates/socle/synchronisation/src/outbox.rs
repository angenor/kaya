//! Implémentation PostgreSQL du grand livre.
//!
//! **Écrit à la main contre sqlx 0.9.0**, comme tout le module doré. Tout extrait trouvé en ligne
//! vise `0.8.x` et ne compilera pas : `#3723` impose `AssertSqlSafe` sur toute requête non
//! littérale et `#3541` modifie la sortie des macros `query!()`.

use crate::{ErreurOutbox, EvenementAEcrire, OutboxWriter};

/// Écrivain d'événements adossé à PostgreSQL.
///
/// Sans état : il ne détient ni pool ni connexion. La transaction lui est **toujours** fournie
/// par l'appelant — c'est ce qui rend impossible l'écriture d'un événement hors de la
/// transaction métier.
#[derive(Debug, Default, Clone, Copy)]
pub struct PgOutboxWriter;

impl PgOutboxWriter {
    pub const fn nouveau() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl OutboxWriter for PgOutboxWriter {
    #[tracing::instrument(
        skip(self, tx, evenement),
        fields(
            evenement.id = %evenement.id,
            evenement.type = %evenement.type_evenement,
            tenant.id = %evenement.tenant_id,
        )
    )]
    async fn ecrire(
        &self,
        tx: &mut sqlx::PgTransaction<'_>,
        evenement: EvenementAEcrire,
    ) -> Result<(), ErreurOutbox> {
        // La séquence est tirée **côté serveur**, jamais fournie par l'appelant. La clé est
        // l'établissement quand il y en a un, le tenant sinon — un événement de niveau tenant
        // doit lui aussi être ordonné, sans quoi la moitié du grand livre serait hors séquence.
        //
        // Rappel qui évite une « correction » erronée plus tard : les séquences PostgreSQL ne
        // sont pas transactionnelles. Un rollback laisse un trou, et **c'est voulu** (R-07). La
        // séquence garantit l'ordre et la détection de manque, pas la continuité. Garantir la
        // continuité imposerait un verrou par établissement sur le chemin d'écriture le plus
        // chaud du produit.
        let cle_sequence = evenement.etablissement_id.unwrap_or(evenement.tenant_id);

        let sequence: i64 = sqlx::query_scalar!(
            "SELECT synchronisation.prochaine_sequence($1)",
            cle_sequence
        )
        .fetch_one(&mut **tx)
        .await?
        .unwrap_or_default();

        // `survenu_le` vient de `now()`, donc de **l'horloge du serveur de base**, pas de celle
        // du processus applicatif ni d'un terminal (principe IV). Deux instances d'API ne
        // partagent pas forcément la même horloge ; la base, elle, est unique.
        sqlx::query!(
            r#"
            INSERT INTO synchronisation.evenement_outbox (
                id, tenant_id, etablissement_id, sequence_etablissement,
                type_evenement, agregat, agregat_id, version_schema,
                payload, survenu_le, publie_le
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, now(), NULL)
            "#,
            evenement.id,
            evenement.tenant_id,
            evenement.etablissement_id,
            sequence,
            evenement.type_evenement,
            evenement.agregat,
            evenement.agregat_id,
            evenement.version_schema,
            evenement.payload,
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }
}
