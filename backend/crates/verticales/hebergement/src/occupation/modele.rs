//! Types de l'occupation — **classe B**.
//!
//! # La distinction que tout ce module organise
//!
//! Une occupation porte **deux intervalles**, et les confondre est la faute qui coûterait cher :
//!
//! | | Ce que c'est | Qui le voit |
//! |---|---|---|
//! | `debut_client` / `fin_client` | Les bornes **commerciales** | Le client. La note se calcule là-dessus |
//! | `periode` (`tstzrange`) | La période d'**indisponibilité**, remise en état comprise | La contrainte d'exclusion |
//!
//! Le client ne paie pas le ménage, et la chambre n'est pas attribuable pendant. Une seule paire
//! de bornes obligerait à choisir laquelle des deux vérités porter.

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::referentiel::{ErreurReferentiel, StatutMenage};

/// L'état d'une occupation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum StatutOccupation {
    Active,
    /// **Libérée, jamais supprimée.** Une chambre occupée reste une chambre occupée dans
    /// l'histoire — `DELETE` n'est pas accordé à `kaya_app`.
    Liberee,
}

impl StatutOccupation {
    pub fn code(self) -> &'static str {
        match self {
            StatutOccupation::Active => "active",
            StatutOccupation::Liberee => "liberee",
        }
    }

    pub fn depuis_code(code: &str) -> Result<Self, ErreurAttribution> {
        match code {
            "active" => Ok(StatutOccupation::Active),
            "liberee" => Ok(StatutOccupation::Liberee),
            autre => Err(ErreurAttribution::StatutInconnu(autre.to_owned())),
        }
    }
}

/// Une occupation, telle que l'API la rend.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OccupationVue {
    pub id: Uuid,
    pub unite_id: Uuid,
    pub formule_id: Uuid,
    /// Borne commerciale — ce que le client connaît.
    #[serde(with = "time::serde::rfc3339")]
    pub debut_client: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub fin_client: OffsetDateTime,
    /// **Borne haute de la période d'indisponibilité** — `fin_client` + le battement de remise en
    /// état. Le serveur la calcule ; le client ne l'envoie pas et ne peut pas l'influencer.
    #[serde(with = "time::serde::rfc3339")]
    pub indisponible_jusqu_a: OffsetDateTime,
    pub statut: StatutOccupation,
    /// **Horodatage d'autorité serveur.** C'est lui que le calcul de durée d'un passage lit.
    #[serde(with = "time::serde::rfc3339")]
    pub cree_le: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339::option")]
    pub libere_le: Option<OffsetDateTime>,
}

/// Une unité attribuable, telle que la consultation de disponibilité la rend.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UniteDisponible {
    pub id: Uuid,
    pub code: String,
    pub etage: Option<i16>,
    /// Rendu pour information : une chambre « à nettoyer » est attribuable, elle demande
    /// simplement qu'on passe avant. **Aucun endpoint de ce cycle ne l'écrit** (HEB-06).
    pub statut_menage: StatutMenage,
}

/// Demande d'attribution.
///
/// `tenant_id` n'y figure pas : il est porté par l'instance du service, construite par la couche
/// d'assemblage qui connaît l'appelant. Le laisser fournir par le corps d'une requête offrirait à
/// un client la possibilité d'écrire chez un autre.
///
/// **La borne haute de `periode` n'y figure pas non plus**, et c'est la décision centrale de cette
/// structure : le serveur la calcule en ajoutant le battement de la catégorie. Si le client
/// l'envoyait, il pourrait la mettre à zéro et supprimer le ménage.
#[derive(Debug, Clone)]
pub struct DemandeAttribution {
    /// UUID v7 **généré par le client** — c'est lui qui rend le rejeu inoffensif.
    pub id: Uuid,
    pub etablissement_id: Uuid,
    pub unite_id: Uuid,
    pub formule_id: Uuid,
    pub debut_client: OffsetDateTime,
    pub fin_client: OffsetDateTime,
}

/// Le type de refus qui traverse toute la verticale.
#[derive(Debug, thiserror::Error)]
pub enum ErreurAttribution {
    /// **Violation de la contrainte d'exclusion** — la seule qui vient de la base.
    ///
    /// Elle n'est jamais produite par une vérification préalable : le service tente l'insertion et
    /// traduit l'erreur. Une lecture préalable serait exactement le verrou applicatif que le
    /// principe IV refuse.
    #[error("unite_deja_occupee")]
    UniteDejaOccupee,

    /// La formule n'appartient pas à la catégorie de l'unité.
    #[error("formule_hors_categorie")]
    FormuleHorsCategorie,

    /// Demi-journée : l'intervalle demandé ne coïncide pas avec une plage déclarée.
    #[error("plage_non_fractionnable")]
    PlageNonFractionnable,

    /// Fin ≤ début.
    #[error("intervalle_invalide")]
    IntervalleInvalide,

    /// Durée hors des bornes de la formule.
    #[error("duree_hors_contrainte")]
    DureeHorsContrainte,

    #[error("unite_inconnue")]
    UniteInconnue,

    #[error("formule_inconnue")]
    FormuleInconnue,

    #[error("occupation_inconnue")]
    OccupationInconnue,

    /// Une occupation déjà libérée ne se libère pas deux fois — mais un **rejeu** de la même
    /// opération n'est pas une erreur. Voir `ServiceOccupation::liberer`.
    #[error("occupation_deja_liberee")]
    OccupationDejaLiberee,

    #[error("statut_inconnu: {0}")]
    StatutInconnu(String),

    #[error("service_inactif")]
    ServiceInactif,

    #[error("etablissement_inconnu")]
    EtablissementInconnu,

    #[error("erreur de base : {0}")]
    Base(#[from] sqlx::Error),

    #[error("écriture au grand livre : {0}")]
    Outbox(#[from] kaya_synchronisation::ErreurOutbox),

    #[error("contexte de tenant : {0}")]
    ContexteTenant(#[from] kaya_etablissements::tenant_context::ErreurContexteTenant),

    #[error("lecture de l'établissement : {0}")]
    Annuaire(#[from] kaya_etablissements::ErreurLecture),

    #[error("registre des modules : {0}")]
    Registre(#[from] kaya_etablissements::ErreurRegistre),

    #[error("référentiel : {0}")]
    Referentiel(#[from] ErreurReferentiel),
}

impl ErreurAttribution {
    /// Le code stable que le contrat HTTP rend — **c'est lui que l'interface traduit**.
    pub fn code(&self) -> &'static str {
        match self {
            ErreurAttribution::UniteDejaOccupee => "unite_deja_occupee",
            ErreurAttribution::FormuleHorsCategorie => "formule_hors_categorie",
            ErreurAttribution::PlageNonFractionnable => "plage_non_fractionnable",
            ErreurAttribution::IntervalleInvalide => "intervalle_invalide",
            ErreurAttribution::DureeHorsContrainte => "duree_hors_contrainte",
            ErreurAttribution::UniteInconnue => "unite_inconnue",
            ErreurAttribution::FormuleInconnue => "formule_inconnue",
            ErreurAttribution::OccupationInconnue => "occupation_inconnue",
            ErreurAttribution::OccupationDejaLiberee => "occupation_deja_liberee",
            ErreurAttribution::StatutInconnu(_) => "statut_inconnu",
            ErreurAttribution::ServiceInactif => "service_inactif",
            ErreurAttribution::EtablissementInconnu => "etablissement_inconnu",
            ErreurAttribution::Referentiel(e) => e.code(),
            ErreurAttribution::Base(_)
            | ErreurAttribution::Outbox(_)
            | ErreurAttribution::ContexteTenant(_)
            | ErreurAttribution::Annuaire(_)
            | ErreurAttribution::Registre(_) => "erreur_interne",
        }
    }
}
