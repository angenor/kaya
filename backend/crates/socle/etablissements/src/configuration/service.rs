//! Couche service de la configuration héritée — ETB-04.

use std::collections::BTreeMap;

use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use kaya_synchronisation::{EvenementAEcrire, OutboxWriter};

use super::modele::{EcrireParametre, EntreeCatalogue, ErreurParametre, valeur_compatible};
use super::repository;
use crate::{Cible, Portee, ValeurResolue, tenant_context};

pub const VERSION_SCHEMA: i16 = 1;
pub const AGREGAT: &str = "parametre_configuration";
pub const TYPE_ECRIT: &str = "parametre_configuration.ecrit";

/// Service de la configuration.
pub struct ServiceConfiguration<E: OutboxWriter> {
    pool: PgPool,
    outbox: E,
}

impl<E: OutboxWriter> ServiceConfiguration<E> {
    pub fn nouveau(pool: PgPool, outbox: E) -> Self {
        Self { pool, outbox }
    }

    /// Résout une clé sur une cible.
    pub async fn resoudre(
        &self,
        cible: &Cible,
        cle: &str,
    ) -> Result<Option<ValeurResolue>, ErreurParametre> {
        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, cible.tenant_id).await?;
        let resolue = repository::resoudre(
            &mut tx,
            cible.etablissement_id,
            cible.module_code.as_deref(),
            cible.point_de_vente_id,
            cle,
        )
        .await?;
        tx.rollback().await?;
        Ok(resolue)
    }

    /// Résout toutes les clés applicables, en une descente.
    pub async fn resoudre_tout(
        &self,
        cible: &Cible,
    ) -> Result<BTreeMap<String, ValeurResolue>, ErreurParametre> {
        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, cible.tenant_id).await?;
        let carte = repository::resoudre_tout(
            &mut tx,
            cible.etablissement_id,
            cible.module_code.as_deref(),
            cible.point_de_vente_id,
        )
        .await?;
        tx.rollback().await?;
        Ok(carte)
    }

    /// Écrit une valeur à un niveau donné.
    ///
    /// Trois validations, dans cet ordre :
    ///
    ///   1. **la clé est au catalogue** — la clé étrangère l'impose déjà en base ; ce contrôle-ci
    ///      donne un message qui nomme la clé plutôt qu'une violation de contrainte ;
    ///   2. **la portée est compatible** avec `portee_la_plus_basse` — poser un paramètre de
    ///      niveau tenant sur un point de vente n'a pas de sens, et le laisser passer produirait
    ///      une valeur que la résolution ne remonterait jamais ;
    ///   3. **le type de valeur est conforme** — l'extension de la porte P-10 au `JSONB`, sans
    ///      laquelle un montant en flottant entrerait par la porte de service.
    #[tracing::instrument(skip(self, demande), fields(cle = %demande.cle, portee = ?demande.portee))]
    pub async fn ecrire(
        &self,
        tenant_id: Uuid,
        demande: EcrireParametre,
    ) -> Result<(), ErreurParametre> {
        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, tenant_id).await?;

        let entree = repository::entree_catalogue(&mut tx, &demande.cle)
            .await?
            .ok_or_else(|| ErreurParametre::CleHorsCatalogue(demande.cle.clone()))?;

        // La portée demandée doit être **au plus aussi basse** que celle du catalogue. Le tenant
        // est toujours autorisé comme racine, quel que soit le paramètre.
        let plus_basse = portee_depuis_code(&entree.portee_la_plus_basse);
        if demande.portee.rang() > plus_basse.rang() {
            return Err(ErreurParametre::PorteeInterdite {
                cle: demande.cle,
                portee: demande.portee.code().to_owned(),
                plus_basse: entree.portee_la_plus_basse,
            });
        }

        if !valeur_compatible(&entree.type_valeur, &demande.valeur) {
            return Err(ErreurParametre::TypeIncompatible {
                cle: demande.cle,
                type_attendu: entree.type_valeur,
                recu: demande.valeur.to_string(),
            });
        }

        // Répartition de l'identifiant de niveau dans la bonne colonne. C'est ici que la portée
        // devient **dérivée** : au-delà, plus rien ne la déclare.
        let (etablissement_id, module_id, pdv_id) = match demande.portee {
            Portee::Tenant => (None, None, None),
            Portee::Etablissement => {
                let id = demande
                    .portee_id
                    .ok_or_else(|| ErreurParametre::PorteeIdManquant("ETABLISSEMENT".to_owned()))?;
                if !repository::etablissement_du_tenant(&mut tx, id).await? {
                    return Err(ErreurParametre::NiveauInconnu);
                }
                (Some(id), None, None)
            }
            Portee::Module => {
                let id = demande
                    .portee_id
                    .ok_or_else(|| ErreurParametre::PorteeIdManquant("MODULE".to_owned()))?;
                if !repository::module_du_tenant(&mut tx, id).await? {
                    return Err(ErreurParametre::NiveauInconnu);
                }
                (None, Some(id), None)
            }
            Portee::PointDeVente => {
                let id = demande
                    .portee_id
                    .ok_or_else(|| ErreurParametre::PorteeIdManquant("POINT_DE_VENTE".to_owned()))?;
                if !repository::point_de_vente_du_tenant(&mut tx, id).await? {
                    return Err(ErreurParametre::NiveauInconnu);
                }
                (None, None, Some(id))
            }
        };

        let ancienne = repository::ecrire(
            &mut tx,
            tenant_id,
            demande.id,
            etablissement_id,
            module_id,
            pdv_id,
            &demande.cle,
            &demande.valeur,
        )
        .await?;

        // Une écriture qui ne change rien n'émet aucun événement — même principe que le rejeu.
        if ancienne.as_ref() != Some(&demande.valeur) {
            self.outbox
                .ecrire(
                    &mut tx,
                    EvenementAEcrire {
                        id: Uuid::now_v7(),
                        tenant_id,
                        etablissement_id,
                        type_evenement: TYPE_ECRIT.to_owned(),
                        agregat: AGREGAT.to_owned(),
                        agregat_id: demande.id,
                        version_schema: VERSION_SCHEMA,
                        payload: json!({
                            "cle": demande.cle,
                            "valeur": demande.valeur,
                            "niveau": demande.portee.code(),
                            "portee_id": demande.portee_id,
                            // **L'ancienne valeur**, sans quoi le grand livre dirait qu'une valeur
                            // a changé sans dire depuis quoi — et une reconstitution ne pourrait
                            // pas remonter le fil d'un barème modifié trois fois.
                            "ancienne_valeur": ancienne,
                        }),
                    },
                )
                .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    /// Le catalogue complet des clés connues.
    pub async fn catalogue(
        &self,
        tenant_id: Uuid,
    ) -> Result<Vec<EntreeCatalogue>, ErreurParametre> {
        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, tenant_id).await?;
        let catalogue = repository::catalogue(&mut tx).await?;
        tx.rollback().await?;
        Ok(catalogue)
    }
}

pub fn portee_depuis_code(code: &str) -> Portee {
    match code {
        "ETABLISSEMENT" => Portee::Etablissement,
        "MODULE" => Portee::Module,
        "POINT_DE_VENTE" => Portee::PointDeVente,
        _ => Portee::Tenant,
    }
}

/// Implémentation du trait [`crate::ResolveurConfiguration`].
///
/// **Le composant le plus réutilisé du produit** : HEB, FIS, CAI, IMP, STK, RSV, QRC, CPT et SYN
/// le liront tous.
pub struct PgResolveurConfiguration {
    pool: PgPool,
}

impl PgResolveurConfiguration {
    pub fn nouveau(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl crate::ResolveurConfiguration for PgResolveurConfiguration {
    async fn resoudre(
        &self,
        cible: &Cible,
        cle: &str,
    ) -> Result<Option<ValeurResolue>, crate::ErreurConfiguration> {
        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, cible.tenant_id).await?;
        let resolue = repository::resoudre(
            &mut tx,
            cible.etablissement_id,
            cible.module_code.as_deref(),
            cible.point_de_vente_id,
            cle,
        )
        .await
        .map_err(en_erreur_configuration)?;
        tx.rollback().await?;
        Ok(resolue)
    }

    async fn resoudre_tout(
        &self,
        cible: &Cible,
    ) -> Result<BTreeMap<String, ValeurResolue>, crate::ErreurConfiguration> {
        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, cible.tenant_id).await?;
        let carte = repository::resoudre_tout(
            &mut tx,
            cible.etablissement_id,
            cible.module_code.as_deref(),
            cible.point_de_vente_id,
        )
        .await
        .map_err(en_erreur_configuration)?;
        tx.rollback().await?;
        Ok(carte)
    }
}

fn en_erreur_configuration(e: ErreurParametre) -> crate::ErreurConfiguration {
    match e {
        ErreurParametre::Base(e) => crate::ErreurConfiguration::Base(e),
        ErreurParametre::ContexteTenant(e) => crate::ErreurConfiguration::ContexteTenant(e),
        autre => unreachable!("résolution de configuration : variante inattendue {autre:?}"),
    }
}
