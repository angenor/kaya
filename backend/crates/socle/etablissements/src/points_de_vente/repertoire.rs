//! Implémentation PostgreSQL de [`RepertoirePointsDeVente`].
//!
//! Le trait par lequel `verticales/restauration`, `verticales/bar` et `verticales/pressing` liront
//! leurs points de vente au cycle PDV — **jamais par jointure inter-schémas** (porte P-04).

use sqlx::PgPool;
use uuid::Uuid;

use super::repository;
use crate::{
    ErreurRegistre, PointDeVente, RepertoirePointsDeVente, TablePdv, tenant_context,
};

/// Lecture des points de vente, adossée à PostgreSQL.
#[derive(Debug, Clone)]
pub struct PgRepertoirePointsDeVente {
    pool: PgPool,
    tenant_id: Uuid,
}

impl PgRepertoirePointsDeVente {
    pub fn nouveau(pool: PgPool, tenant_id: Uuid) -> Self {
        Self { pool, tenant_id }
    }
}

fn en_erreur_registre(e: super::modele::ErreurPointDeVente) -> ErreurRegistre {
    match e {
        super::modele::ErreurPointDeVente::Base(e) => ErreurRegistre::Base(e),
        super::modele::ErreurPointDeVente::ContexteTenant(e) => ErreurRegistre::ContexteTenant(e),
        autre => unreachable!("lecture de points de vente : variante inattendue {autre:?}"),
    }
}

fn en_point_de_vente(vue: super::modele::PointDeVenteVue) -> PointDeVente {
    PointDeVente {
        id: vue.id,
        etablissement_id: vue.etablissement_id,
        module_code: vue.module_code,
        nom: vue.nom,
        caisse_id: vue.caisse_id,
        tables: vue
            .tables
            .into_iter()
            .map(|t| TablePdv {
                id: t.id,
                libelle: t.libelle,
            })
            .collect(),
    }
}

#[async_trait::async_trait]
impl RepertoirePointsDeVente for PgRepertoirePointsDeVente {
    async fn points_de_vente(
        &self,
        etablissement_id: Uuid,
    ) -> Result<Vec<PointDeVente>, ErreurRegistre> {
        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, self.tenant_id).await?;
        let liste = repository::lister(&mut tx, etablissement_id)
            .await
            .map_err(en_erreur_registre)?;
        tx.rollback().await?;
        Ok(liste.into_iter().map(en_point_de_vente).collect())
    }

    async fn point_de_vente(&self, id: Uuid) -> Result<Option<PointDeVente>, ErreurRegistre> {
        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, self.tenant_id).await?;
        let vue = repository::lire(&mut tx, id)
            .await
            .map_err(en_erreur_registre)?;
        tx.rollback().await?;
        Ok(vue.map(en_point_de_vente))
    }
}
