//! Types du point de vente — ETB-03.
//!
//! Terme utilisateur : **« Point de vente »**, et **« Comptoir »** pour celui qui n'a aucune table
//! (`docs/design/lexique.md`). Jamais « point de vente sans tables », qui décrit un manque là où
//! il s'agit d'une forme normale.

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use utoipa::ToSchema;
use uuid::Uuid;

/// Un point de vente, tel que l'API le rend.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PointDeVenteVue {
    pub id: Uuid,
    pub etablissement_id: Uuid,
    /// Le service auquel il est rattaché — `RESTAURATION`, `BAR`, `PRESSING`…
    pub module_code: String,
    pub nom: String,
    /// **Aucune clé étrangère en base** : frontière de module (principe II).
    pub caisse_id: Option<Uuid>,
    pub actif: bool,
    /// **Vide ⇒ comptoir.** Aucun champ `est_comptoir` : un drapeau pourrait contredire cette
    /// liste, et il faudrait alors décider lequel des deux ment.
    pub tables: Vec<TableVue>,
    #[serde(with = "time::serde::rfc3339")]
    pub cree_le: OffsetDateTime,
}

/// Une table d'un point de vente.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TableVue {
    pub id: Uuid,
    /// « 12 », « Terrasse 3 » — tel que le personnel le dit.
    pub libelle: String,
}

/// Demande de création d'un point de vente.
#[derive(Debug, Clone)]
pub struct CreerPointDeVente {
    /// UUID v7 client.
    pub id: Uuid,
    /// Le service doit être **activé** sur l'établissement : la clé étrangère vers
    /// `etablissement_module` le rend structurellement impossible autrement.
    pub module_code: String,
    pub nom: String,
    pub caisse_id: Option<Uuid>,
}

/// Demande de modification — tout champ absent est laissé tel quel.
#[derive(Debug, Clone, Default)]
pub struct ModifierPointDeVente {
    pub nom: Option<String>,
    pub caisse_id: Option<Uuid>,
    pub actif: Option<bool>,
}

/// Une table à poser, dans un remplacement d'ensemble.
#[derive(Debug, Clone)]
pub struct TableDemandee {
    pub id: Uuid,
    pub libelle: String,
}

/// Erreur du domaine des points de vente.
#[derive(Debug, thiserror::Error)]
pub enum ErreurPointDeVente {
    #[error("nom vide ou trop long : entre 1 et 120 caractères après nettoyage")]
    NomInvalide,

    #[error("libellé de table vide")]
    LibelleInvalide,

    /// Le service n'est pas activé sur cet établissement (FR-041).
    #[error("module « {0} » non actif sur cet établissement")]
    ModuleNonActif(String),

    #[error("un point de vente porte déjà le nom « {0} » dans cet établissement")]
    NomDejaPris(String),

    #[error("établissement inconnu ou hors du tenant courant")]
    EtablissementInconnu,

    #[error("point de vente inconnu ou hors du tenant courant")]
    Inconnu,

    #[error("erreur de base : {0}")]
    Base(#[from] sqlx::Error),

    #[error("écriture au grand livre : {0}")]
    Outbox(#[from] kaya_synchronisation::ErreurOutbox),

    #[error("contexte de tenant : {0}")]
    ContexteTenant(#[from] crate::tenant_context::ErreurContexteTenant),
}

impl ErreurPointDeVente {
    pub fn code(&self) -> &'static str {
        match self {
            ErreurPointDeVente::NomInvalide => "nom_invalide",
            ErreurPointDeVente::LibelleInvalide => "libelle_invalide",
            ErreurPointDeVente::ModuleNonActif(_) => "module_non_actif",
            ErreurPointDeVente::NomDejaPris(_) => "nom_deja_pris",
            ErreurPointDeVente::EtablissementInconnu => "etablissement_inconnu",
            ErreurPointDeVente::Inconnu => "point_de_vente_inconnu",
            _ => "erreur_interne",
        }
    }

    pub fn valeur(&self) -> Option<String> {
        match self {
            ErreurPointDeVente::ModuleNonActif(v) | ErreurPointDeVente::NomDejaPris(v) => {
                Some(v.clone())
            }
            _ => None,
        }
    }
}
