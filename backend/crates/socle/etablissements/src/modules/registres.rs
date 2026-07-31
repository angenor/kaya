//! Implémentations PostgreSQL de [`RegistreModules`] et [`RegistreCapacites`].
//!
//! Ce sont les deux traits par lesquels **chaque verticale** demande, au démarrage d'une
//! opération, si le service est rendu ici et ce qu'il consomme. Aucune verticale n'interroge
//! `etablissement_module` directement : ce serait une jointure inter-schémas (porte P-04).

use sqlx::PgPool;
use uuid::Uuid;

use super::repository;
use crate::{
    CapaciteDeclaree, ErreurRegistre, RegistreCapacites, RegistreModules, tenant_context,
};

/// Registre des services actifs, adossé à PostgreSQL.
#[derive(Debug, Clone)]
pub struct PgRegistreModules {
    pool: PgPool,
    tenant_id: Uuid,
}

impl PgRegistreModules {
    pub fn nouveau(pool: PgPool, tenant_id: Uuid) -> Self {
        Self { pool, tenant_id }
    }
}

fn en_erreur_registre(e: super::modele::ErreurModules) -> ErreurRegistre {
    match e {
        super::modele::ErreurModules::Base(e) => ErreurRegistre::Base(e),
        super::modele::ErreurModules::ContexteTenant(e) => ErreurRegistre::ContexteTenant(e),
        // Les autres variantes ne peuvent pas naître d'une lecture : le repository ne valide rien,
        // il lit. Les mapper sur une erreur de lecture générique serait mentir sur la cause ; le
        // cas est impossible et le panique le dit plutôt que de le masquer.
        autre => unreachable!("lecture de registre : variante inattendue {autre:?}"),
    }
}

#[async_trait::async_trait]
impl RegistreModules for PgRegistreModules {
    async fn modules_actifs(&self, etablissement_id: Uuid) -> Result<Vec<String>, ErreurRegistre> {
        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, self.tenant_id).await?;
        let codes = repository::codes_actifs(&mut tx, etablissement_id)
            .await
            .map_err(en_erreur_registre)?;
        tx.rollback().await?;
        Ok(codes)
    }

    /// **Sans exception si le module n'existe pas.** Un code inconnu et un code non activé sont la
    /// même chose pour l'appelant : dans les deux cas, ce service n'est pas rendu ici.
    async fn module_actif(
        &self,
        etablissement_id: Uuid,
        code: &str,
    ) -> Result<bool, ErreurRegistre> {
        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, self.tenant_id).await?;
        let etat = repository::etat(&mut tx, etablissement_id, code)
            .await
            .map_err(en_erreur_registre)?;
        tx.rollback().await?;
        Ok(etat.is_some_and(|e| e.actif))
    }
}

/// Registre des capacités déclarées, adossé à PostgreSQL.
#[derive(Debug, Clone)]
pub struct PgRegistreCapacites {
    pool: PgPool,
    tenant_id: Uuid,
}

impl PgRegistreCapacites {
    pub fn nouveau(pool: PgPool, tenant_id: Uuid) -> Self {
        Self { pool, tenant_id }
    }
}

#[async_trait::async_trait]
impl RegistreCapacites for PgRegistreCapacites {
    async fn capacites_du_module(
        &self,
        etablissement_id: Uuid,
        module_code: &str,
    ) -> Result<Vec<CapaciteDeclaree>, ErreurRegistre> {
        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, self.tenant_id).await?;

        let etat = repository::etat(&mut tx, etablissement_id, module_code)
            .await
            .map_err(en_erreur_registre)?;

        // Un service inactif ne consomme rien : ses déclarations existent en base mais sont
        // **inertes** (FR-037). Elles ne sont pas supprimées — la réactivation les restitue.
        let Some(etat) = etat.filter(|e| e.actif) else {
            tx.rollback().await?;
            return Ok(Vec::new());
        };

        let capacites = repository::capacites_du_service(&mut tx, etat.id)
            .await
            .map_err(en_erreur_registre)?;
        tx.rollback().await?;

        Ok(capacites
            .into_iter()
            .map(|c| CapaciteDeclaree {
                capacite_code: c.capacite_code,
                profil_code: c.profil_code,
            })
            .collect())
    }

    /// Rend `Option` plutôt que `bool` : **c'est le profil qui décide du comportement**, et un
    /// booléen obligerait `capacites/stocks` à un second appel — donc à deux vérités possibles
    /// entre les deux.
    async fn consomme(
        &self,
        etablissement_id: Uuid,
        module_code: &str,
        capacite_code: &str,
    ) -> Result<Option<CapaciteDeclaree>, ErreurRegistre> {
        let capacites = self
            .capacites_du_module(etablissement_id, module_code)
            .await?;
        Ok(capacites
            .into_iter()
            .find(|c| c.capacite_code == capacite_code))
    }
}
