//! Couche service des points de vente — ETB-03.

use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use kaya_synchronisation::{EvenementAEcrire, OutboxWriter};

use super::modele::{
    CreerPointDeVente, ErreurPointDeVente, ModifierPointDeVente, PointDeVenteVue, TableDemandee,
};
use super::repository;
use crate::{Issue, tenant_context};

pub const VERSION_SCHEMA: i16 = 1;

pub const AGREGAT_PDV: &str = "point_de_vente";
pub const AGREGAT_TABLE: &str = "table_pdv";

pub const TYPE_PDV_CREE: &str = "point_de_vente.cree";
pub const TYPE_PDV_MODIFIE: &str = "point_de_vente.modifie";
pub const TYPE_TABLE_CREEE: &str = "table_pdv.creee";
pub const TYPE_TABLE_DESACTIVEE: &str = "table_pdv.desactivee";

/// Longueur maximale du nom d'un point de vente — **il tient sur un ticket**.
pub const NOM_MAX: usize = 120;

pub struct ServicePointsDeVente<E: OutboxWriter> {
    pool: PgPool,
    outbox: E,
}

impl<E: OutboxWriter> ServicePointsDeVente<E> {
    pub fn nouveau(pool: PgPool, outbox: E) -> Self {
        Self { pool, outbox }
    }

    /// Crée un point de vente rattaché à un service **actif**.
    #[tracing::instrument(skip(self, demande), fields(etablissement.id = %etablissement_id))]
    pub async fn creer(
        &self,
        tenant_id: Uuid,
        etablissement_id: Uuid,
        demande: CreerPointDeVente,
    ) -> Result<(PointDeVenteVue, Issue), ErreurPointDeVente> {
        let nom = demande.nom.trim().to_owned();
        if nom.is_empty() || nom.chars().count() > NOM_MAX {
            return Err(ErreurPointDeVente::NomInvalide);
        }
        let demande = CreerPointDeVente { nom, ..demande };

        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, tenant_id).await?;

        if !repository::etablissement_existe(&mut tx, etablissement_id).await? {
            return Err(ErreurPointDeVente::EtablissementInconnu);
        }

        // La clé étrangère rendrait le cas impossible de toute façon ; ce contrôle-ci donne le
        // message qui **nomme le service** au lieu d'une violation de contrainte (FR-041).
        let module_id = repository::service_actif(&mut tx, etablissement_id, &demande.module_code)
            .await?
            .ok_or_else(|| ErreurPointDeVente::ModuleNonActif(demande.module_code.clone()))?;

        let (id, issue) =
            repository::inserer(&mut tx, tenant_id, etablissement_id, module_id, &demande).await?;

        let vue = repository::lire(&mut tx, id)
            .await?
            .ok_or(ErreurPointDeVente::Inconnu)?;

        if issue == Issue::Creee {
            self.emettre(
                &mut tx,
                tenant_id,
                etablissement_id,
                id,
                AGREGAT_PDV,
                TYPE_PDV_CREE,
                json!({
                    "point_de_vente_id": id,
                    "etablissement_id": etablissement_id,
                    "module_code": demande.module_code,
                    "nom": vue.nom,
                    // **La présence de tables, pas leur liste** : à la création il n'y en a
                    // aucune, et le lecteur du grand livre doit pouvoir dire que ce point de vente
                    // est né comptoir.
                    "a_des_tables": !vue.tables.is_empty(),
                    "caisse_id": vue.caisse_id,
                }),
            )
            .await?;
        }

        tx.commit().await?;
        Ok((vue, issue))
    }

    /// Modifie un point de vente.
    pub async fn modifier(
        &self,
        tenant_id: Uuid,
        id: Uuid,
        demande: ModifierPointDeVente,
    ) -> Result<PointDeVenteVue, ErreurPointDeVente> {
        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, tenant_id).await?;

        let avant = repository::lire(&mut tx, id)
            .await?
            .ok_or(ErreurPointDeVente::Inconnu)?;

        let nom = demande.nom.unwrap_or_else(|| avant.nom.clone());
        let nom = nom.trim().to_owned();
        if nom.is_empty() || nom.chars().count() > NOM_MAX {
            return Err(ErreurPointDeVente::NomInvalide);
        }
        let caisse_id = demande.caisse_id.or(avant.caisse_id);
        let actif = demande.actif.unwrap_or(avant.actif);

        if nom == avant.nom && caisse_id == avant.caisse_id && actif == avant.actif {
            tx.rollback().await?;
            return Ok(avant);
        }

        repository::modifier(&mut tx, id, &nom, caisse_id, actif).await?;
        let apres = repository::lire(&mut tx, id)
            .await?
            .ok_or(ErreurPointDeVente::Inconnu)?;

        self.emettre(
            &mut tx,
            tenant_id,
            avant.etablissement_id,
            id,
            AGREGAT_PDV,
            TYPE_PDV_MODIFIE,
            json!({
                "point_de_vente_id": id,
                "etablissement_id": avant.etablissement_id,
                "module_code": apres.module_code,
                "avant": { "nom": avant.nom, "caisse_id": avant.caisse_id, "actif": avant.actif },
                "apres": { "nom": apres.nom, "caisse_id": apres.caisse_id, "actif": apres.actif },
                "a_des_tables": !apres.tables.is_empty(),
            }),
        )
        .await?;

        tx.commit().await?;
        Ok(apres)
    }

    /// **Remplace l'ensemble des tables.**
    ///
    /// Une liste vide fait du point de vente un **comptoir** — transition légitime, pas une
    /// suppression accidentelle : c'est exactement ce qui arrive quand un maquis retire ses tables
    /// pour ne plus servir qu'au comptoir.
    ///
    /// Les tables retirées sont **désactivées, jamais supprimées** : les commandes déjà passées
    /// les référencent, et l'historique d'un soir de service doit rester lisible.
    #[tracing::instrument(skip(self, tables), fields(point_de_vente.id = %id, tables = tables.len()))]
    pub async fn remplacer_tables(
        &self,
        tenant_id: Uuid,
        id: Uuid,
        tables: Vec<TableDemandee>,
    ) -> Result<PointDeVenteVue, ErreurPointDeVente> {
        for table in &tables {
            if table.libelle.trim().is_empty() {
                return Err(ErreurPointDeVente::LibelleInvalide);
            }
        }

        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, tenant_id).await?;

        let avant = repository::lire(&mut tx, id)
            .await?
            .ok_or(ErreurPointDeVente::Inconnu)?;

        let desactivees = repository::desactiver_toutes_les_tables(&mut tx, id).await?;

        let mut posees = Vec::with_capacity(tables.len());
        for table in &tables {
            let pose =
                repository::poser_table(&mut tx, tenant_id, id, table.id, table.libelle.trim())
                    .await?;
            posees.push((pose, table.libelle.trim().to_owned()));
        }

        // Une table désactivée puis reposée dans le même appel n'a pas changé d'état : elle ne
        // doit apparaître dans aucun des deux événements. Sans ce filtrage, un simple
        // enregistrement du plan de salle inchangé produirait douze désactivations et douze
        // créations au grand livre.
        let reposees: Vec<Uuid> = posees.iter().map(|(id, _)| *id).collect();
        let vraiment_desactivees: Vec<Uuid> = desactivees
            .into_iter()
            .filter(|d| !reposees.contains(d))
            .collect();
        let vraiment_creees: Vec<&(Uuid, String)> = posees
            .iter()
            .filter(|(id, _)| !avant.tables.iter().any(|t| t.id == *id))
            .collect();

        for (table_id, libelle) in &vraiment_creees {
            self.emettre(
                &mut tx,
                tenant_id,
                avant.etablissement_id,
                *table_id,
                AGREGAT_TABLE,
                TYPE_TABLE_CREEE,
                json!({
                    "table_id": table_id,
                    "point_de_vente_id": id,
                    "libelle": libelle,
                }),
            )
            .await?;
        }

        for table_id in &vraiment_desactivees {
            let libelle = avant
                .tables
                .iter()
                .find(|t| t.id == *table_id)
                .map(|t| t.libelle.clone());
            self.emettre(
                &mut tx,
                tenant_id,
                avant.etablissement_id,
                *table_id,
                AGREGAT_TABLE,
                TYPE_TABLE_DESACTIVEE,
                json!({
                    "table_id": table_id,
                    "point_de_vente_id": id,
                    "libelle": libelle,
                }),
            )
            .await?;
        }

        let apres = repository::lire(&mut tx, id)
            .await?
            .ok_or(ErreurPointDeVente::Inconnu)?;

        tx.commit().await?;
        Ok(apres)
    }

    /// Les points de vente actifs d'un établissement.
    pub async fn lister(
        &self,
        tenant_id: Uuid,
        etablissement_id: Uuid,
    ) -> Result<Vec<PointDeVenteVue>, ErreurPointDeVente> {
        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, tenant_id).await?;

        if !repository::etablissement_existe(&mut tx, etablissement_id).await? {
            return Err(ErreurPointDeVente::EtablissementInconnu);
        }

        let liste = repository::lister(&mut tx, etablissement_id).await?;
        tx.rollback().await?;
        Ok(liste)
    }

    #[allow(clippy::too_many_arguments)]
    async fn emettre(
        &self,
        tx: &mut sqlx::PgTransaction<'_>,
        tenant_id: Uuid,
        etablissement_id: Uuid,
        agregat_id: Uuid,
        agregat: &str,
        type_evenement: &str,
        payload: serde_json::Value,
    ) -> Result<(), ErreurPointDeVente> {
        self.outbox
            .ecrire(
                tx,
                EvenementAEcrire {
                    id: Uuid::now_v7(),
                    tenant_id,
                    etablissement_id: Some(etablissement_id),
                    type_evenement: type_evenement.to_owned(),
                    agregat: agregat.to_owned(),
                    agregat_id,
                    version_schema: VERSION_SCHEMA,
                    payload,
                },
            )
            .await?;
        Ok(())
    }
}
