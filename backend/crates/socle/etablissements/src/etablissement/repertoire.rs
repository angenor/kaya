//! Implémentation PostgreSQL d'[`EstablishmentDirectory`].
//!
//! **Le trait par lequel les autres crates lisent un établissement.** Posé à vide au cycle 001
//! pour que le premier `JOIN` inter-schémas ne soit pas écrit « juste cette fois » au cycle HEB ;
//! ce cycle lui donne son contenu.
//!
//! Toute lecture pose le tenant courant : sans lui, la politique de sécurité ne rend **aucune
//! ligne** — pas une erreur, zéro ligne. Un appelant qui oublierait le contexte verrait donc un
//! établissement inexistant plutôt que celui d'un autre client.

use sqlx::PgPool;
use uuid::Uuid;

use crate::{Classement, ErreurLecture, EstablishmentDirectory, Etablissement, tenant_context};

/// Lecture d'un établissement, adossée à PostgreSQL.
#[derive(Debug, Clone)]
pub struct PgEstablishmentDirectory {
    pool: PgPool,
    /// # Pourquoi le tenant est porté par l'implémentation, pas par le trait
    ///
    /// Le trait est consommé par des crates qui n'ont pas de contexte de tenant à passer — une
    /// verticale demande « l'établissement 42 », pas « l'établissement 42 du tenant 7 ». Le
    /// contexte est donc **lié à l'instance**, construite par la couche d'assemblage qui, elle,
    /// connaît l'appelant. Ajouter `tenant_id` à chaque signature du trait le ferait remonter
    /// dans tous les appelants, où il finirait par être fourni par le corps d'une requête.
    tenant_id: Uuid,
}

impl PgEstablishmentDirectory {
    pub fn nouveau(pool: PgPool, tenant_id: Uuid) -> Self {
        Self { pool, tenant_id }
    }
}

#[async_trait::async_trait]
impl EstablishmentDirectory for PgEstablishmentDirectory {
    async fn etablissement(&self, id: Uuid) -> Result<Option<Etablissement>, ErreurLecture> {
        let mut tx = self.pool.begin().await?;
        // `poser_tenant` ne peut échouer que sur une erreur de base ; la convertir ici évite
        // d'ajouter une variante à `ErreurLecture`, qui est le type que lisent les autres crates.
        tenant_context::poser_tenant(&mut tx, self.tenant_id)
            .await
            .map_err(|e| match e {
                crate::tenant_context::ErreurContexteTenant::Base(e) => ErreurLecture::Base(e),
            })?;

        let ligne = sqlx::query!(
            r#"
            SELECT id, tenant_id, nom, fuseau_horaire, devise,
                   juridiction, classement, etoiles, commune, adresse, ncc
            FROM etablissements.etablissement
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&mut *tx)
        .await?;

        tx.rollback().await?;

        let Some(l) = ligne else {
            return Ok(None);
        };

        let classement = Classement::depuis_colonnes(&l.classement, l.etoiles)
            .ok_or(ErreurLecture::ClassementIllisible { id })?;

        Ok(Some(Etablissement {
            id: l.id,
            tenant_id: l.tenant_id,
            nom: l.nom,
            fuseau_horaire: l.fuseau_horaire,
            devise: l.devise,
            juridiction: l.juridiction,
            classement,
            commune: l.commune,
            adresse: l.adresse,
            ncc: l.ncc,
        }))
    }

    /// L'établissement appartient-il à ce tenant ?
    ///
    /// La question a l'air redondante avec la politique de sécurité, qui masque déjà les lignes
    /// d'autrui. Elle ne l'est pas : l'appelant a besoin de distinguer « n'existe pas » de
    /// « existe ailleurs » pour choisir entre `404` et `403`, et la politique rend les deux
    /// identiques. La réponse est calculée **dans le contexte du tenant demandé**, jamais en
    /// lisant `tenant_id` hors politique.
    async fn appartient_au_tenant(
        &self,
        etablissement_id: Uuid,
        tenant_id: Uuid,
    ) -> Result<bool, ErreurLecture> {
        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, tenant_id)
            .await
            .map_err(|e| match e {
                crate::tenant_context::ErreurContexteTenant::Base(e) => ErreurLecture::Base(e),
            })?;

        let existe = sqlx::query_scalar!(
            r#"
            SELECT EXISTS (
                SELECT 1 FROM etablissements.etablissement WHERE id = $1
            ) AS "existe!"
            "#,
            etablissement_id
        )
        .fetch_one(&mut *tx)
        .await?;

        tx.rollback().await?;
        Ok(existe)
    }
}
