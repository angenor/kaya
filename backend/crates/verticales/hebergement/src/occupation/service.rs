//! Service de l'occupation — **le cœur du cycle**.
//!
//! # La règle qui commande tout ce fichier
//!
//! > **Tenter l'insertion et traduire la violation. Ne JAMAIS lire d'abord pour décider.**
//!
//! Une lecture préalable — « cette unité est-elle libre ? » puis « alors je l'attribue » — serait
//! exactement le verrou applicatif que le principe IV refuse. Entre les deux instructions, une
//! autre transaction peut prendre l'unité : le code paraîtrait correct, passerait en revue, et
//! double-attribuerait sous charge sans que rien ne le signale.
//!
//! La garantie est dans `occupation_sans_chevauchement`. Ce service la **traduit**, il ne la
//! remplace pas.
//!
//! # Ce que le serveur calcule et que le client ne peut pas influencer
//!
//! La **borne haute de `periode`** — `fin_client` + le battement de remise en état de la catégorie
//! pour la famille de la formule. Si le client l'envoyait, il pourrait la mettre à zéro et
//! supprimer le ménage ; la chambre suivante serait attribuée sur une chambre sale.

use serde_json::json;
use sqlx::PgPool;
use sqlx::postgres::types::PgRange;
use time::OffsetDateTime;
use uuid::Uuid;

use kaya_etablissements::tenant_context;
use kaya_etablissements::{EstablishmentDirectory, RegistreModules};
use kaya_synchronisation::{EvenementAEcrire, OutboxWriter};

use super::modele::{
    DemandeAttribution, ErreurAttribution, OccupationVue, StatutOccupation, UniteDisponible,
};
use super::repository;
use crate::erreurs::{CONTRAINTE_SANS_CHEVAUCHEMENT, est_violation_exclusion};
use crate::referentiel::{FamilleFormule, repository as referentiel_repo};
use crate::{Issue, MODULE_HEBERGEMENT};

pub const TYPE_OCCUPATION_ATTRIBUEE: &str = "heb.occupation.attribuee";
pub const TYPE_OCCUPATION_LIBEREE: &str = "heb.occupation.liberee";
pub const AGREGAT_OCCUPATION: &str = "hebergement.occupation";

/// Version du format des charges utiles d'occupation (R-06).
pub const VERSION_SCHEMA_OCCUPATION: i16 = 1;

/// Service de la disponibilité et de l'attribution.
///
/// # Pourquoi le tenant est porté par l'instance
///
/// Le trait [`crate::traits::MoteurDisponibilite`] est consommé par SEJ-02, qui attribuera une
/// unité **dans sa propre transaction**. Ajouter `tenant_id` à chaque signature du trait le ferait
/// remonter dans tous les appelants, où il finirait par être fourni par le corps d'une requête.
/// Le contexte est donc lié à l'instance, construite par la couche d'assemblage — même décision
/// qu'à `PgEstablishmentDirectory` au cycle 002.
pub struct ServiceOccupation<E, A, R>
where
    E: OutboxWriter,
    A: EstablishmentDirectory,
    R: RegistreModules,
{
    pool: PgPool,
    tenant_id: Uuid,
    outbox: E,
    annuaire: A,
    modules: R,
}

impl<E, A, R> ServiceOccupation<E, A, R>
where
    E: OutboxWriter,
    A: EstablishmentDirectory,
    R: RegistreModules,
{
    pub fn nouveau(pool: PgPool, tenant_id: Uuid, outbox: E, annuaire: A, modules: R) -> Self {
        Self {
            pool,
            tenant_id,
            outbox,
            annuaire,
            modules,
        }
    }

    /// L'établissement existe, et l'hébergement y est actif.
    async fn garde(&self, etablissement_id: Uuid) -> Result<(), ErreurAttribution> {
        self.annuaire
            .etablissement(etablissement_id)
            .await?
            .ok_or(ErreurAttribution::EtablissementInconnu)?;

        if !self
            .modules
            .module_actif(etablissement_id, MODULE_HEBERGEMENT)
            .await?
        {
            return Err(ErreurAttribution::ServiceInactif);
        }
        Ok(())
    }

    // =============================================================================================
    //  Consulter la disponibilité — une lecture qui ne garantit rien
    // =============================================================================================

    /// Les unités attribuables d'une catégorie sur un intervalle, **et l'instant d'autorité**.
    ///
    /// L'instant est rendu parce que le client en a besoin pour situer sa lecture, et **parce
    /// qu'il ne doit surtout pas employer le sien** : la durée facturée d'un passage se calcule
    /// depuis l'horloge du serveur.
    pub async fn unites_disponibles_avec_instant(
        &self,
        etablissement_id: Uuid,
        categorie_id: Uuid,
        debut: OffsetDateTime,
        fin: OffsetDateTime,
    ) -> Result<(Vec<UniteDisponible>, OffsetDateTime), ErreurAttribution> {
        if fin <= debut {
            return Err(ErreurAttribution::IntervalleInvalide);
        }
        self.garde(etablissement_id).await?;

        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, self.tenant_id).await?;

        let periode = PgRange {
            start: std::ops::Bound::Included(debut),
            end: std::ops::Bound::Excluded(fin),
        };
        let unites =
            repository::unites_disponibles(&mut tx, etablissement_id, categorie_id, periode)
                .await?;
        let instant = repository::maintenant(&mut tx).await?;

        // Lecture : la transaction est annulée.
        tx.rollback().await?;
        Ok((unites, instant))
    }

    // =============================================================================================
    //  Attribuer — l'opération que la contrainte protège
    // =============================================================================================

    /// Attribue une unité **dans la transaction fournie**.
    ///
    /// C'est la forme que SEJ-02 consommera : attribuer l'unité et ouvrir la note dans une seule
    /// transaction. Un trait qui prendrait un pool obligerait à deux transactions, donc à une saga
    /// pour une opération qui n'en demande pas.
    ///
    /// # L'ordre des opérations, et les deux points qu'on écrirait mal
    ///
    /// 1. valider l'intervalle — inutile d'aller en base pour une fin avant le début ;
    /// 2. vérifier que la formule appartient à la catégorie de l'unité ;
    /// 3. vérifier la durée contre les bornes de la formule ;
    /// 4. pour une demi-journée, vérifier la coïncidence avec une plage déclarée ;
    /// 5. calculer la borne haute — **le serveur, jamais le client** ;
    /// 6. **tenter l'insertion et traduire la violation** — jamais lire d'abord pour décider ;
    /// 7. émettre l'événement **uniquement si la ligne vient d'être créée**.
    ///
    /// Le point 6 est la garantie du produit. Le point 7 est celui qu'on écrirait mal : un rejeu
    /// ne produit aucun nouvel événement, sans quoi le grand livre deviendrait le journal des
    /// tentatives réseau du terminal.
    pub async fn attribuer_dans(
        &self,
        tx: &mut sqlx::PgTransaction<'_>,
        demande: DemandeAttribution,
    ) -> Result<(OccupationVue, Issue), ErreurAttribution> {
        if demande.fin_client <= demande.debut_client {
            return Err(ErreurAttribution::IntervalleInvalide);
        }

        // ── 2 · la formule s'applique-t-elle à cette chambre ? ────────────────────────────────
        if !referentiel_repo::formule_appartient_a_l_unite(tx, demande.formule_id, demande.unite_id)
            .await?
        {
            return Err(ErreurAttribution::FormuleHorsCategorie);
        }

        let (famille, duree_min, duree_max) =
            repository::contraintes_de_formule(tx, demande.formule_id)
                .await?
                .ok_or(ErreurAttribution::FormuleInconnue)?;
        let famille = FamilleFormule::depuis_code(&famille)?;

        // ── 3 · la durée demandée tient-elle dans les bornes de la formule ? ──────────────────
        let duree = demande.fin_client - demande.debut_client;
        let duree_minutes = duree.whole_minutes();
        if let Some(min) = duree_min
            && duree_minutes < i64::from(min)
        {
            return Err(ErreurAttribution::DureeHorsContrainte);
        }
        if let Some(max) = duree_max
            && duree_minutes > i64::from(max)
        {
            return Err(ErreurAttribution::DureeHorsContrainte);
        }

        // ── 4 · une demi-journée se loue en entier ────────────────────────────────────────────
        if famille == FamilleFormule::DemiJournee {
            self.verifier_plage(tx, &demande).await?;
        }

        // ── 5 · la borne haute vient du SERVEUR ───────────────────────────────────────────────
        let battement =
            repository::battement_minutes(tx, demande.unite_id, demande.formule_id).await?;
        let fin_periode = demande.fin_client + time::Duration::minutes(i64::from(battement));

        // ── 6 · tenter, puis traduire ─────────────────────────────────────────────────────────
        let creee = match repository::inserer(tx, self.tenant_id, &demande, fin_periode).await {
            Ok(creee) => creee,
            Err(erreur) if est_violation_exclusion(&erreur, CONTRAINTE_SANS_CHEVAUCHEMENT) => {
                return Err(ErreurAttribution::UniteDejaOccupee);
            }
            Err(erreur) => return Err(ErreurAttribution::Base(erreur)),
        };

        let vue = repository::lire(tx, demande.id)
            .await?
            .ok_or(ErreurAttribution::OccupationInconnue)?;

        // ── 7 · l'événement, uniquement à la création ─────────────────────────────────────────
        if creee {
            self.emettre(
                tx,
                demande.etablissement_id,
                TYPE_OCCUPATION_ATTRIBUEE,
                vue.id,
                json!({
                    "occupation_id": vue.id,
                    "unite_id": vue.unite_id,
                    "formule_id": vue.formule_id,
                    "debut_client": vue.debut_client.to_string(),
                    "fin_client": vue.fin_client.to_string(),
                    "indisponible_jusqu_a": vue.indisponible_jusqu_a.to_string(),
                    "battement_minutes": battement,
                }),
            )
            .await?;
        }

        Ok((
            vue,
            if creee {
                Issue::Creee
            } else {
                Issue::DejaPresente
            },
        ))
    }

    /// La même opération, **avec sa propre transaction** — ce que l'endpoint appelle.
    pub async fn attribuer(
        &self,
        demande: DemandeAttribution,
    ) -> Result<(OccupationVue, Issue), ErreurAttribution> {
        self.garde(demande.etablissement_id).await?;

        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, self.tenant_id).await?;
        let resultat = self.attribuer_dans(&mut tx, demande).await;
        match resultat {
            Ok(v) => {
                tx.commit().await?;
                Ok(v)
            }
            Err(e) => {
                // Une violation de contrainte empoisonne la transaction : le `rollback` est
                // obligatoire, et son échec ne doit pas masquer l'erreur métier.
                let _ = tx.rollback().await;
                Err(e)
            }
        }
    }

    /// **Une demi-journée se loue en entier** (FR-033).
    ///
    /// La comparaison se fait **après conversion en instants**, avec le fuseau de l'établissement :
    /// comparer des heures murales échouerait au passage de minuit, et surtout ne dirait rien de
    /// la date. « 8 h » désigne 8 h à Abengourou, quelle que soit l'horloge du terminal ou celle
    /// du serveur.
    async fn verifier_plage(
        &self,
        tx: &mut sqlx::PgTransaction<'_>,
        demande: &DemandeAttribution,
    ) -> Result<(), ErreurAttribution> {
        let etablissement = self
            .annuaire
            .etablissement(demande.etablissement_id)
            .await?
            .ok_or(ErreurAttribution::EtablissementInconnu)?;

        // **La conversion se fait en SQL.** PostgreSQL porte la base de fuseaux ; le crate `time`
        // ne porte que des décalages fixes, et aucune dépendance nouvelle n'est permise à ce
        // cycle (principe XI). Ce n'est pas un contournement : le fuseau appartient à
        // l'établissement, la base est unique, et c'est elle qui doit trancher « 8 h à Abengourou
        // le 3 août ».
        let plages = repository::plages_en_instants(
            tx,
            demande.formule_id,
            demande.debut_client,
            &etablissement.fuseau_horaire,
        )
        .await?;

        if plages.is_empty() {
            // Une formule de demi-journée sans plage n'aurait pas dû être créée (FR-033, validé au
            // service du référentiel). La rencontrer ici est un défaut de données, pas une saisie.
            return Err(ErreurAttribution::PlageNonFractionnable);
        }

        let coincide = plages
            .iter()
            .any(|(debut, fin)| *debut == demande.debut_client && *fin == demande.fin_client);

        if coincide {
            Ok(())
        } else {
            Err(ErreurAttribution::PlageNonFractionnable)
        }
    }

    // =============================================================================================
    //  Libérer — un UPDATE, jamais un DELETE
    // =============================================================================================

    /// Libère une occupation : la période est **raccourcie**, le statut passe à `liberee`.
    ///
    /// **Un rejeu rend `200`, pas une erreur.** Une occupation déjà libérée par la même opération
    /// est le cas normal d'un terminal qui vide sa file ; l'appelant reçoit la ligne telle qu'elle
    /// est en base, et **aucun second événement n'est émis**.
    pub async fn liberer(
        &self,
        etablissement_id: Uuid,
        occupation_id: Uuid,
    ) -> Result<(OccupationVue, Issue), ErreurAttribution> {
        self.garde(etablissement_id).await?;

        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, self.tenant_id).await?;

        let (unite_id, formule_id, _) = repository::famille_de_l_occupation(&mut tx, occupation_id)
            .await?
            .ok_or(ErreurAttribution::OccupationInconnue)?;

        let battement = repository::battement_minutes(&mut tx, unite_id, formule_id).await?;
        let liberee = repository::liberer(&mut tx, occupation_id, battement).await?;

        let vue = repository::lire(&mut tx, occupation_id)
            .await?
            .ok_or(ErreurAttribution::OccupationInconnue)?;

        if liberee {
            self.emettre(
                &mut tx,
                etablissement_id,
                TYPE_OCCUPATION_LIBEREE,
                vue.id,
                json!({
                    "occupation_id": vue.id,
                    "unite_id": vue.unite_id,
                    "libere_le": vue.libere_le.map(|d| d.to_string()),
                    "indisponible_jusqu_a": vue.indisponible_jusqu_a.to_string(),
                    "battement_minutes": battement,
                }),
            )
            .await?;
        }

        tx.commit().await?;

        Ok((
            vue,
            if liberee {
                Issue::Creee
            } else {
                Issue::DejaPresente
            },
        ))
    }

    /// L'état de **toutes** les unités d'un établissement — opération 17, cycle 006.
    ///
    /// Rend aussi l'instant d'autorité, parce que la réponse est vraie **à un instant** : le
    /// client en a besoin pour situer sa lecture, et **il ne doit surtout pas employer le sien**.
    pub async fn etat_des_unites(
        &self,
        etablissement_id: Uuid,
    ) -> Result<(Vec<repository::EtatUnite>, OffsetDateTime), ErreurAttribution> {
        self.garde(etablissement_id).await?;

        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, self.tenant_id).await?;
        let unites = repository::etat_des_unites(&mut tx, etablissement_id).await?;
        let instant = repository::maintenant(&mut tx).await?;
        // Lecture : la transaction est annulée.
        tx.rollback().await?;

        Ok((unites, instant))
    }

    /// Lit une occupation — employée par la tarification et les tests.
    pub async fn lire(
        &self,
        occupation_id: Uuid,
    ) -> Result<Option<OccupationVue>, ErreurAttribution> {
        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, self.tenant_id).await?;
        let vue = repository::lire(&mut tx, occupation_id).await?;
        tx.rollback().await?;
        Ok(vue)
    }

    async fn emettre(
        &self,
        tx: &mut sqlx::PgTransaction<'_>,
        etablissement_id: Uuid,
        type_evenement: &str,
        agregat_id: Uuid,
        payload: serde_json::Value,
    ) -> Result<(), ErreurAttribution> {
        self.outbox
            .ecrire(
                tx,
                EvenementAEcrire {
                    id: Uuid::now_v7(),
                    tenant_id: self.tenant_id,
                    etablissement_id: Some(etablissement_id),
                    type_evenement: type_evenement.to_owned(),
                    agregat: AGREGAT_OCCUPATION.to_owned(),
                    agregat_id,
                    version_schema: VERSION_SCHEMA_OCCUPATION,
                    payload,
                },
            )
            .await?;
        Ok(())
    }
}

/// L'état d'une occupation, pour les appelants qui n'ont pas besoin du reste.
pub fn est_active(vue: &OccupationVue) -> bool {
    vue.statut == StatutOccupation::Active
}

// =================================================================================================
//  L'implémentation du trait exposé
// =================================================================================================

/// **`attribuer` prend la transaction** — c'est toute la raison du trait, et l'implémentation le
/// tient : elle délègue à `attribuer_dans`, qui ne commite ni n'ouvre rien.
///
/// La garde d'établissement et de module n'est **pas** refaite ici : l'appelant qui fournit une
/// transaction l'a ouverte pour une opération plus large — le check-in de SEJ-02 — et a déjà
/// vérifié l'établissement. La refaire dans une transaction empruntée mélangerait deux niveaux de
/// responsabilité, et surtout ferait deux lectures pour un fait déjà établi.
#[async_trait::async_trait]
impl<E, A, R> crate::traits::MoteurDisponibilite for ServiceOccupation<E, A, R>
where
    E: OutboxWriter + Send + Sync,
    A: EstablishmentDirectory + Send + Sync,
    R: RegistreModules + Send + Sync,
{
    async fn unites_disponibles(
        &self,
        etablissement_id: Uuid,
        categorie_id: Uuid,
        periode: PgRange<OffsetDateTime>,
    ) -> Result<Vec<UniteDisponible>, ErreurAttribution> {
        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, self.tenant_id).await?;
        let unites =
            repository::unites_disponibles(&mut tx, etablissement_id, categorie_id, periode)
                .await?;
        tx.rollback().await?;
        Ok(unites)
    }

    async fn attribuer(
        &self,
        tx: &mut sqlx::PgTransaction<'_>,
        demande: DemandeAttribution,
    ) -> Result<(OccupationVue, Issue), ErreurAttribution> {
        self.attribuer_dans(tx, demande).await
    }
}
