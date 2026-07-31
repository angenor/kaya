//! Types de l'activation de service — ETB-02, ETB-02b.
//!
//! Terme utilisateur : **« Vos services »** (`docs/design/lexique.md`). « Module d'activité » est
//! le nom **technique** — table, trait, événement — et n'apparaît jamais à l'écran. Le mot
//! « capacité », lui, **n'apparaît nulle part** : seule la capacité concrète est nommée, « Suivi
//! du stock », sous le service qui la consomme.

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::Obstacle;

/// Un service **actif** d'un établissement, tel que l'API le rend.
///
/// # Aucun service inactif ne franchit cette frontière
///
/// Il n'y a pas de champ `actif` : ce type ne décrit que des services actifs. Un booléen ici
/// serait la porte d'entrée du grisé qu'interdit le principe VII — donné à l'interface, il
/// produirait une liste où figurent les services que l'établissement n'a pas, et quelqu'un
/// finirait par les afficher « pour information ».
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ServiceActif {
    pub id: Uuid,
    /// `HEBERGEMENT`, `RESTAURATION`, `BAR`, `PRESSING`, `SALLE_REUNION`.
    pub module_code: String,
    /// **Clé i18n, jamais un libellé.** Le texte vit dans `app/core/i18n/{fr,en}.json`.
    pub libelle_cle: String,
    /// Ordre d'affichage du référentiel — stable, indépendant de la locale.
    pub ordre: i16,
    #[serde(with = "time::serde::rfc3339")]
    pub active_le: OffsetDateTime,
    /// Capacités déclarées par ce service. Vide est la forme normale.
    pub capacites: Vec<CapaciteDuService>,
}

/// Une capacité déclarée par un service.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CapaciteDuService {
    pub id: Uuid,
    /// `STOCK` seule au MVP.
    pub capacite_code: String,
    /// `SIMPLE` seul au MVP.
    pub profil_code: String,
    /// Clé i18n du libellé de la capacité — « Suivi du stock ».
    pub libelle_cle: String,
}

/// Demande d'activation ou de désactivation — **le même point d'entrée porte les deux sens**.
///
/// Deux points d'entrée distincts laisseraient deux chemins pour un état, et un jour deux
/// comportements.
#[derive(Debug, Clone)]
pub struct BasculerService {
    /// UUID v7 client — utilisé à la **première** activation seulement. Une réactivation est un
    /// `UPDATE` de la ligne existante, jamais une seconde ligne (FR-015).
    pub id: Uuid,
    pub actif: bool,
}

/// Demande de déclaration de capacité.
#[derive(Debug, Clone)]
pub struct DeclarerCapacite {
    pub id: Uuid,
    pub capacite_code: String,
    pub profil_code: String,
}

/// Erreur du domaine des services.
#[derive(Debug, thiserror::Error)]
pub enum ErreurModules {
    #[error("module « {0} » inconnu du référentiel")]
    ModuleInconnu(String),

    /// Le code existe au référentiel avec `implementee = false`.
    #[error("module « {0} » déclaré au référentiel mais non implémenté au MVP")]
    ModuleNonImplemente(String),

    /// Le service n'est pas activé sur cet établissement.
    #[error("module « {0} » non actif sur cet établissement")]
    ModuleNonActif(String),

    #[error("capacité « {0} » déclarée au référentiel mais non implémentée au MVP")]
    CapaciteNonImplementee(String),

    #[error("capacité « {0} » inconnue du référentiel")]
    CapaciteInconnue(String),

    /// Porte le **motif du référentiel**, pas une phrase : `AUCUN` mérite un message distinct de
    /// `VALORISE` et `DETAILLE`. Voir [`Self::motif_cle`].
    #[error("profil « {code} » non implémenté au MVP")]
    ProfilNonImplemente { code: String, motif_cle: String },

    #[error("profil « {0} » inconnu du référentiel")]
    ProfilInconnu(String),

    /// Un ou plusieurs obstacles s'opposent à la désactivation (FR-016).
    #[error("désactivation refusée : {} obstacle(s)", .0.len())]
    DesactivationBloquee(Vec<Obstacle>),

    #[error("établissement inconnu ou hors du tenant courant")]
    EtablissementInconnu,

    #[error("erreur de base : {0}")]
    Base(#[from] sqlx::Error),

    #[error("écriture au grand livre : {0}")]
    Outbox(#[from] kaya_synchronisation::ErreurOutbox),

    #[error("contexte de tenant : {0}")]
    ContexteTenant(#[from] crate::tenant_context::ErreurContexteTenant),
}

impl ErreurModules {
    /// Code stable exposé par le contrat HTTP — **jamais traduit**.
    pub fn code(&self) -> &'static str {
        match self {
            ErreurModules::ModuleInconnu(_) => "module_inconnu",
            ErreurModules::ModuleNonImplemente(_) => "module_non_implemente",
            ErreurModules::ModuleNonActif(_) => "module_non_actif",
            ErreurModules::CapaciteNonImplementee(_) => "capacite_non_implementee",
            ErreurModules::CapaciteInconnue(_) => "capacite_inconnue",
            ErreurModules::ProfilNonImplemente { .. } => "profil_non_implemente",
            ErreurModules::ProfilInconnu(_) => "profil_inconnu",
            ErreurModules::DesactivationBloquee(_) => "desactivation_bloquee",
            ErreurModules::EtablissementInconnu => "etablissement_inconnu",
            _ => "erreur_interne",
        }
    }

    /// La valeur refusée, pour composer un message qui **nomme la chose** (FR-032, FR-033).
    pub fn valeur(&self) -> Option<String> {
        match self {
            ErreurModules::ModuleInconnu(v)
            | ErreurModules::ModuleNonImplemente(v)
            | ErreurModules::ModuleNonActif(v)
            | ErreurModules::CapaciteNonImplementee(v)
            | ErreurModules::CapaciteInconnue(v)
            | ErreurModules::ProfilInconnu(v) => Some(v.clone()),
            ErreurModules::ProfilNonImplemente { code, .. } => Some(code.clone()),
            _ => None,
        }
    }

    /// Clé i18n du motif de refus, quand le référentiel en fournit une.
    ///
    /// **C'est ce qui distingue `AUCUN` des deux autres profils.** `VALORISE` et `DETAILLE`
    /// annoncent une absence — on attend une version future. `AUCUN` dit qu'une capacité non
    /// consommée **ne se déclare pas** : le refus enseigne au lieu de constater, et envoyer le
    /// même message ferait attendre une version future à quelqu'un qui doit juste ne rien faire.
    pub fn motif_cle(&self) -> Option<&str> {
        match self {
            ErreurModules::ProfilNonImplemente { motif_cle, .. } => Some(motif_cle),
            _ => None,
        }
    }

    /// Les obstacles, quand la désactivation est refusée.
    pub fn obstacles(&self) -> &[Obstacle] {
        match self {
            ErreurModules::DesactivationBloquee(o) => o,
            _ => &[],
        }
    }
}
