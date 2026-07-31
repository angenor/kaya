//! Worker de publication — **in-process, aucune file externe** (R-08).
//!
//! # Le mécanisme
//!
//! ```text
//! SELECT ... WHERE publie_le IS NULL ORDER BY id FOR UPDATE SKIP LOCKED LIMIT n
//!   → présentation à chaque consommateur
//!   → UPDATE publie_le = now()
//! ```
//!
//! `SKIP LOCKED` permet à plusieurs instances d'API de tourner sans se marcher dessus ni
//! introduire de verrou distribué — ce qui compte dès le second conteneur démarré.
//!
//! **Redis n'est pas utilisé ici, et ce n'est pas un oubli.** Il ne porte que de l'éphémère
//! reconstructible (principe II) ; une file de publication qui perdrait son état ferait perdre
//! des événements, ce que la rétention illimitée de TRX-02 exclut. La file *est* la table.
//!
//! # Ce qui arrive quand un consommateur échoue
//!
//! L'événement reste **non marqué publié**, donc republié au tour suivant, indéfiniment. Aucun
//! événement n'est jamais abandonné ni supprimé. C'est ce qui rend l'idempotence des
//! consommateurs obligatoire et non facultative.
//!
//! # Le marquage se fait dans la transaction du lot
//!
//! Une reprise brutale entre la présentation et le marquage republie le lot. Les consommateurs
//! voient donc l'effet d'une seule présentation grâce à leur propre idempotence, jamais grâce à
//! une garantie du worker — `backend/tests/worker_redemarrage.rs` le constate.

use std::sync::Arc;
use std::time::Duration;

use sqlx::PgPool;

use crate::{ErreurOutbox, EventConsumer, EvenementPublie};

/// Réglages du worker.
#[derive(Debug, Clone, Copy)]
pub struct ConfigurationWorker {
    /// Nombre d'événements traités par tour.
    pub taille_lot: i64,
    /// Attente entre deux tours quand il n'y a rien à publier.
    pub intervalle: Duration,
    /// Périmètre de publication — `None` pour **tous les tenants**.
    ///
    /// # Pourquoi ce réglage existe
    ///
    /// Le serveur mutualisé publie tous les tenants : c'est le cas par défaut, et `None` le dit.
    ///
    /// Deux situations demandent un périmètre restreint :
    ///
    ///   * **le nœud de site** (mode C, incrément 3) ne connaît qu'un établissement et n'a aucune
    ///     raison de balayer le grand livre des autres ;
    ///   * **les tests**, qui s'exécutent en parallèle sur une base partagée. Sans périmètre, un
    ///     test de worker publierait les événements produits par les autres tests au même
    ///     instant, et ses décomptes deviendraient dépendants de l'ordonnancement — un test
    ///     intermittent, donc un test qu'on finit par ignorer.
    ///
    /// Le filtre s'ajoute à la politique de sécurité, il ne la remplace pas.
    pub tenant: Option<uuid::Uuid>,
}

impl Default for ConfigurationWorker {
    fn default() -> Self {
        Self {
            taille_lot: 100,
            intervalle: Duration::from_millis(500),
            tenant: None,
        }
    }
}

/// Worker de publication.
pub struct WorkerPublication {
    pool: PgPool,
    consommateurs: Vec<Arc<dyn EventConsumer>>,
    configuration: ConfigurationWorker,
}

impl WorkerPublication {
    /// Construit le worker.
    ///
    /// Le pool attendu est celui du rôle **`kaya_worker`** : il lit tous les tenants par la
    /// politique `publication_worker` (migration 0005) et ne peut rien écrire d'autre que
    /// `publie_le`. Lui donner le pool applicatif ferait publier zéro événement, en silence — la
    /// politique `isolation_tenant` filtrant sur un contexte que le worker ne pose pas.
    pub fn nouveau(
        pool: PgPool,
        consommateurs: Vec<Arc<dyn EventConsumer>>,
        configuration: ConfigurationWorker,
    ) -> Self {
        Self {
            pool,
            consommateurs,
            configuration,
        }
    }

    /// Traite **un** lot et renvoie le nombre d'événements publiés.
    ///
    /// Exposée séparément de la boucle pour que les tests exercent un tour précis plutôt que
    /// d'attendre un intervalle : un test qui dort est un test qui devient intermittent.
    #[tracing::instrument(skip(self))]
    pub async fn traiter_un_lot(&self) -> Result<usize, ErreurOutbox> {
        let mut tx = self.pool.begin().await?;

        // `ORDER BY id` avec des UUID v7 donne l'ordre chronologique d'écriture : v7 préfixe
        // l'identifiant d'un horodatage en millisecondes. Trier sur `survenu_le` serait moins
        // sûr — deux événements de la même milliseconde n'auraient pas d'ordre stable.
        let lignes = sqlx::query!(
            r#"
            SELECT id, tenant_id, etablissement_id, sequence_etablissement,
                   type_evenement, agregat, agregat_id, version_schema, payload, survenu_le
            FROM synchronisation.evenement_outbox
            WHERE publie_le IS NULL
              AND ($2::uuid IS NULL OR tenant_id = $2)
            ORDER BY id
            FOR UPDATE SKIP LOCKED
            LIMIT $1
            "#,
            self.configuration.taille_lot,
            self.configuration.tenant,
        )
        .fetch_all(&mut *tx)
        .await?;

        if lignes.is_empty() {
            tx.rollback().await?;
            return Ok(0);
        }

        let mut publies = Vec::with_capacity(lignes.len());

        for ligne in lignes {
            let evenement = EvenementPublie {
                id: ligne.id,
                tenant_id: ligne.tenant_id,
                etablissement_id: ligne.etablissement_id,
                sequence_etablissement: ligne.sequence_etablissement,
                type_evenement: ligne.type_evenement,
                agregat: ligne.agregat,
                agregat_id: ligne.agregat_id,
                version_schema: ligne.version_schema,
                payload: ligne.payload,
                survenu_le: ligne.survenu_le,
            };

            let mut tous_ont_reussi = true;
            for consommateur in &self.consommateurs {
                if let Err(erreur) = consommateur.consommer(&evenement).await {
                    tracing::warn!(
                        consommateur = consommateur.nom(),
                        evenement.id = %evenement.id,
                        erreur = %erreur,
                        "consommation en échec — l'événement sera republié au tour suivant"
                    );
                    tous_ont_reussi = false;
                }
            }

            if tous_ont_reussi {
                publies.push(evenement.id);
            }
        }

        if !publies.is_empty() {
            sqlx::query!(
                r#"
                UPDATE synchronisation.evenement_outbox
                SET publie_le = now()
                WHERE id = ANY($1) AND publie_le IS NULL
                "#,
                &publies
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(publies.len())
    }

    /// Boucle de publication, jusqu'à l'arrêt du processus.
    pub async fn boucler(self) {
        loop {
            match self.traiter_un_lot().await {
                Ok(0) => tokio::time::sleep(self.configuration.intervalle).await,
                Ok(n) => tracing::debug!(publies = n, "lot publié"),
                Err(erreur) => {
                    // Une erreur de base ne doit pas tuer le worker : la base peut être
                    // momentanément indisponible, et un worker mort ne se relève pas tout seul.
                    // Les événements restent en attente, ce qui est le comportement voulu.
                    tracing::error!(erreur = %erreur, "tour de publication en échec");
                    tokio::time::sleep(self.configuration.intervalle).await;
                }
            }
        }
    }
}
