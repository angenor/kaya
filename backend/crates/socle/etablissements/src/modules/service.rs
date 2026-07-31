//! Couche service des modules d'activité — activation, désactivation, déclaration de capacité.
//!
//! # Le point d'accrochage `ObstacleDesactivation`, et pourquoi il est posé VIDE
//!
//! FR-016 exige qu'un service portant des opérations en cours ne puisse pas être désactivé. Cette
//! information vit dans les **verticales**, et un crate du socle ne peut pas en dépendre
//! (porte P-03). Le trait est donc **défini** dans le socle, **implémenté** par les verticales et
//! **injecté** ici, à l'assemblage.
//!
//! À ce cycle, la liste est vide et la désactivation est libre. C'est exact — aucune verticale ne
//! crée encore d'opération — et ce n'est pas un trou : ce qui est livré est le point
//! d'accrochage. Quand la question se posera au cycle SEJ, la voie facile sera d'ajouter une
//! dépendance du socle vers `verticales/hebergement` « juste cette fois ». **Une alternative qui
//! existe se prend ; une alternative à construire se contourne.**
//!
//! `backend/tests/desactivation_bloquee.rs` enregistre un obstacle factice et constate le refus :
//! sans lui, un point d'accrochage jamais exercé se casse au premier remaniement sans que rien ne
//! le signale.

use std::sync::Arc;

use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use kaya_synchronisation::{EvenementAEcrire, OutboxWriter};

use super::modele::{
    BasculerService, CapaciteDuService, DeclarerCapacite, ErreurModules, ServiceActif,
};
use super::repository;
use crate::{Issue, ObstacleDesactivation, tenant_context};

pub const VERSION_SCHEMA: i16 = 1;

pub const AGREGAT_MODULE: &str = "etablissement_module";
pub const AGREGAT_CAPACITE: &str = "module_capacite";

pub const TYPE_ACTIVE: &str = "etablissement_module.active";
pub const TYPE_DESACTIVE: &str = "etablissement_module.desactive";
pub const TYPE_CAPACITE_DECLAREE: &str = "module_capacite.declaree";

/// Service des modules d'activité.
pub struct ServiceModules<E: OutboxWriter> {
    pool: PgPool,
    outbox: E,
    /// **Vide à ce cycle.** Chaque verticale y ajoutera son implémentation au cycle où elle crée
    /// des opérations.
    obstacles: Vec<Arc<dyn ObstacleDesactivation>>,
}

/// Résultat d'une bascule de service.
pub struct IssueBascule {
    pub issue: Issue,
    /// L'état a-t-il réellement changé ? Une bascule vers l'état courant n'émet aucun événement.
    pub transition: bool,
    pub service_id: Uuid,
}

impl<E: OutboxWriter> ServiceModules<E> {
    pub fn nouveau(pool: PgPool, outbox: E) -> Self {
        Self {
            pool,
            outbox,
            obstacles: Vec::new(),
        }
    }

    /// Enregistre une source d'obstacles à la désactivation.
    ///
    /// Appelée **à l'assemblage** (`backend/api/`), seul endroit du produit qui a le droit de
    /// connaître à la fois le socle et les verticales.
    pub fn avec_obstacle(mut self, obstacle: Arc<dyn ObstacleDesactivation>) -> Self {
        self.obstacles.push(obstacle);
        self
    }

    /// Active ou désactive un service. **Le même point d'entrée porte les deux sens.**
    ///
    /// Ordre des opérations :
    ///
    ///   1. vérifier le référentiel — un module inconnu et un module non implémenté sont **deux
    ///      messages différents** ;
    ///   2. transaction, pose du tenant, existence de l'établissement ;
    ///   3. pour une désactivation : **interroger tous les obstacles enregistrés** ;
    ///   4. activer (idempotent) ou basculer ;
    ///   5. événement **seulement s'il y a eu transition** ;
    ///   6. commit.
    #[tracing::instrument(skip(self, demande), fields(etablissement.id = %etablissement_id, module = %module_code))]
    pub async fn basculer(
        &self,
        tenant_id: Uuid,
        etablissement_id: Uuid,
        module_code: &str,
        demande: BasculerService,
    ) -> Result<IssueBascule, ErreurModules> {
        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, tenant_id).await?;

        // Le référentiel distingue « inconnu » de « connu mais non implémenté ». Un `CHECK`
        // littéral en base ne saurait pas le faire, et l'exploitant lirait le même message dans
        // les deux cas — l'un se corrige en changeant de valeur, l'autre en attendant un cycle.
        let module = repository::module_du_referentiel(&mut tx, module_code)
            .await?
            .ok_or_else(|| ErreurModules::ModuleInconnu(module_code.to_owned()))?;
        if !module.implementee {
            return Err(ErreurModules::ModuleNonImplemente(module_code.to_owned()));
        }

        if !repository::etablissement_existe(&mut tx, etablissement_id).await? {
            return Err(ErreurModules::EtablissementInconnu);
        }

        let etat = repository::etat(&mut tx, etablissement_id, module_code).await?;

        if !demande.actif {
            let Some(etat) = etat else {
                // Désactiver un service jamais activé : rien à faire, et surtout pas une erreur.
                // Le résultat demandé — « ce service n'est pas rendu » — est déjà vrai.
                tx.rollback().await?;
                return Err(ErreurModules::ModuleNonActif(module_code.to_owned()));
            };

            let obstacles = self.obstacles(etablissement_id, module_code).await?;
            if !obstacles.is_empty() {
                tx.rollback().await?;
                return Err(ErreurModules::DesactivationBloquee(obstacles));
            }

            let transition = repository::basculer(&mut tx, etat.id, false).await?;
            if transition {
                self.emettre(
                    &mut tx,
                    tenant_id,
                    etablissement_id,
                    etat.id,
                    AGREGAT_MODULE,
                    TYPE_DESACTIVE,
                    json!({
                        "etablissement_id": etablissement_id,
                        "etablissement_module_id": etat.id,
                        "module_code": module_code,
                        // **Ce qui n'est PAS supprimé**, écrit dans la charge utile : un lecteur du
                        // grand livre doit savoir que les déclarations de capacité et les
                        // surcharges de configuration sont devenues inertes, pas effacées.
                        "conserve": ["module_capacite", "parametre_configuration"],
                    }),
                )
                .await?;
            }
            tx.commit().await?;
            return Ok(IssueBascule {
                issue: Issue::DejaPresente,
                transition,
                service_id: etat.id,
            });
        }

        // ── Activation ───────────────────────────────────────────────────────────────────────
        match etat {
            None => {
                let insere =
                    repository::activer(&mut tx, tenant_id, demande.id, etablissement_id, module_code)
                        .await?;
                let service_id = match insere {
                    Some(id) => id,
                    // Course perdue avec une transaction concurrente : la ligne existe désormais.
                    // Le serveur fait foi (principe VI), on relit plutôt que d'échouer.
                    None => {
                        repository::etat(&mut tx, etablissement_id, module_code)
                            .await?
                            .ok_or(ErreurModules::EtablissementInconnu)?
                            .id
                    }
                };

                self.emettre(
                    &mut tx,
                    tenant_id,
                    etablissement_id,
                    service_id,
                    AGREGAT_MODULE,
                    TYPE_ACTIVE,
                    json!({
                        "etablissement_id": etablissement_id,
                        "etablissement_module_id": service_id,
                        "module_code": module_code,
                        "premiere_activation": true,
                    }),
                )
                .await?;

                tx.commit().await?;
                Ok(IssueBascule {
                    issue: Issue::Creee,
                    transition: true,
                    service_id,
                })
            }
            Some(etat) => {
                // **Réactivation : un `UPDATE`, jamais une seconde ligne.** C'est ce qui restitue
                // l'état antérieur — déclarations de capacité et surcharges de configuration
                // redeviennent actives sans avoir été touchées (FR-015).
                let transition = repository::basculer(&mut tx, etat.id, true).await?;
                if transition {
                    self.emettre(
                        &mut tx,
                        tenant_id,
                        etablissement_id,
                        etat.id,
                        AGREGAT_MODULE,
                        TYPE_ACTIVE,
                        json!({
                            "etablissement_id": etablissement_id,
                            "etablissement_module_id": etat.id,
                            "module_code": module_code,
                            "premiere_activation": false,
                            "restitue": ["module_capacite", "parametre_configuration"],
                        }),
                    )
                    .await?;
                }
                tx.commit().await?;
                Ok(IssueBascule {
                    issue: Issue::DejaPresente,
                    transition,
                    service_id: etat.id,
                })
            }
        }
    }

    /// Déclare une capacité consommée par un service — **ETB-02b, la porte P-06 côté service**.
    ///
    /// Le refus est tenu à trois couches (research.md R-02) : clé étrangère composite et `CHECK`
    /// en base, variante d'erreur ici, absence pure à l'interface. Ce qui est écrit ci-dessous est
    /// la **deuxième** — elle ne remplace ni la première ni la troisième, elle donne le message.
    #[tracing::instrument(skip(self, demande), fields(etablissement.id = %etablissement_id, module = %module_code))]
    pub async fn declarer_capacite(
        &self,
        tenant_id: Uuid,
        etablissement_id: Uuid,
        module_code: &str,
        demande: DeclarerCapacite,
    ) -> Result<Issue, ErreurModules> {
        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, tenant_id).await?;

        if !repository::etablissement_existe(&mut tx, etablissement_id).await? {
            return Err(ErreurModules::EtablissementInconnu);
        }

        let etat = repository::etat(&mut tx, etablissement_id, module_code)
            .await?
            .filter(|e| e.actif)
            .ok_or_else(|| ErreurModules::ModuleNonActif(module_code.to_owned()))?;

        let capacite = repository::capacite_du_referentiel(&mut tx, &demande.capacite_code)
            .await?
            .ok_or_else(|| ErreurModules::CapaciteInconnue(demande.capacite_code.clone()))?;
        if !capacite.implementee {
            return Err(ErreurModules::CapaciteNonImplementee(
                demande.capacite_code.clone(),
            ));
        }

        let profil = repository::profil_du_referentiel(&mut tx, &demande.profil_code)
            .await?
            .ok_or_else(|| ErreurModules::ProfilInconnu(demande.profil_code.clone()))?;
        if !profil.implementee {
            // Le motif vient du **référentiel**, pas d'un `match` écrit ici : c'est lui qui sait
            // que `AUCUN` mérite un message distinct de `VALORISE` et `DETAILLE`. Le coder ici
            // dupliquerait une décision qui vit déjà en base, et les deux divergeraient.
            return Err(ErreurModules::ProfilNonImplemente {
                code: demande.profil_code.clone(),
                motif_cle: profil.motif_refus_cle.unwrap_or_default(),
            });
        }

        let issue = repository::declarer_capacite(&mut tx, tenant_id, etat.id, &demande).await?;

        if issue == Issue::Creee {
            self.emettre(
                &mut tx,
                tenant_id,
                etablissement_id,
                demande.id,
                AGREGAT_CAPACITE,
                TYPE_CAPACITE_DECLAREE,
                json!({
                    "etablissement_id": etablissement_id,
                    "etablissement_module_id": etat.id,
                    "module_code": module_code,
                    "capacite_code": demande.capacite_code,
                    "profil_code": demande.profil_code,
                }),
            )
            .await?;
        }

        tx.commit().await?;
        Ok(issue)
    }

    /// Les services **actifs** d'un établissement. Aucun service inactif n'est jamais rendu.
    pub async fn services_actifs(
        &self,
        tenant_id: Uuid,
        etablissement_id: Uuid,
    ) -> Result<Vec<ServiceActif>, ErreurModules> {
        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, tenant_id).await?;

        if !repository::etablissement_existe(&mut tx, etablissement_id).await? {
            return Err(ErreurModules::EtablissementInconnu);
        }

        let services = repository::services_actifs(&mut tx, etablissement_id).await?;
        tx.rollback().await?;
        Ok(services)
    }

    /// Les capacités déclarées par un service actif.
    pub async fn capacites_du_service(
        &self,
        tenant_id: Uuid,
        etablissement_id: Uuid,
        module_code: &str,
    ) -> Result<Vec<CapaciteDuService>, ErreurModules> {
        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, tenant_id).await?;

        if !repository::etablissement_existe(&mut tx, etablissement_id).await? {
            return Err(ErreurModules::EtablissementInconnu);
        }

        let etat = repository::etat(&mut tx, etablissement_id, module_code)
            .await?
            .filter(|e| e.actif)
            .ok_or_else(|| ErreurModules::ModuleNonActif(module_code.to_owned()))?;

        let capacites = repository::capacites_du_service(&mut tx, etat.id).await?;
        tx.rollback().await?;
        Ok(capacites)
    }

    /// Interroge **tous** les obstacles enregistrés et rassemble leurs motifs.
    ///
    /// Tous, pas le premier : un exploitant qui doit fermer trois séjours et régler deux additions
    /// veut la liste complète, pas une découverte au coup par coup.
    async fn obstacles(
        &self,
        etablissement_id: Uuid,
        module_code: &str,
    ) -> Result<Vec<crate::Obstacle>, ErreurModules> {
        let mut tous = Vec::new();
        for source in &self.obstacles {
            let trouves = source
                .obstacles(etablissement_id, module_code)
                .await
                .map_err(|e| match e {
                    crate::ErreurRegistre::Base(e) => ErreurModules::Base(e),
                    crate::ErreurRegistre::ContexteTenant(e) => ErreurModules::ContexteTenant(e),
                })?;
            tous.extend(trouves);
        }
        Ok(tous)
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
    ) -> Result<(), ErreurModules> {
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
