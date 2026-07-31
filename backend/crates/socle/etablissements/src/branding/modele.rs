//! Types de l'identité visuelle — ETB-05.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Identité visuelle **résolue**, champ par champ, avec l'origine de chacun.
///
/// # Pourquoi une origine PAR CHAMP et non une origine globale
///
/// La surcharge est **partielle** : un établissement peut porter son propre logo tout en héritant
/// de l'en-tête, du pied et des mentions légales. Une origine globale mentirait sur cinq champs
/// pour en décrire un — et l'écran afficherait « modifié ici » sur des valeurs que personne n'a
/// touchées.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BrandingResolu {
    pub logo_objet_cle: Option<ChampResolu>,
    /// **N'atteint jamais l'interface** (FR-059) : c'est la couleur des **documents produits**.
    pub couleur_primaire: Option<ChampResolu>,
    pub entete_document: Option<ChampResolu>,
    pub pied_document: Option<ChampResolu>,
    pub mentions_legales: Option<ChampResolu>,
    pub coordonnees: Option<ChampResolu>,
}

/// Un champ résolu, avec son origine.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ChampResolu {
    pub valeur: String,
    /// `TENANT` ou `ETABLISSEMENT`.
    pub origine: String,
}

/// Identité visuelle telle qu'elle est posée à **un** niveau.
#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct BrandingNiveau {
    pub logo_objet_cle: Option<String>,
    pub couleur_primaire: Option<String>,
    pub entete_document: Option<String>,
    pub pied_document: Option<String>,
    pub mentions_legales: Option<String>,
    pub coordonnees: Option<String>,
}

/// Demande d'écriture.
#[derive(Debug, Clone)]
pub struct EcrireBranding {
    pub id: Uuid,
    /// `None` = niveau tenant.
    pub etablissement_id: Option<Uuid>,
    pub contenu: BrandingNiveau,
}

/// Erreur du domaine de l'identité visuelle.
#[derive(Debug, thiserror::Error)]
pub enum ErreurBranding {
    #[error("couleur invalide : « {0} » — format hexadécimal #RRGGBB attendu")]
    CouleurInvalide(String),

    #[error("logo trop volumineux : {taille} octets, maximum {maximum} octets")]
    LogoTropVolumineux { taille: usize, maximum: usize },

    #[error("téléversement du logo impossible : {0}")]
    Stockage(String),

    #[error("établissement inconnu ou hors du tenant courant")]
    EtablissementInconnu,

    #[error("erreur de base : {0}")]
    Base(#[from] sqlx::Error),

    #[error("écriture au grand livre : {0}")]
    Outbox(#[from] kaya_synchronisation::ErreurOutbox),

    #[error("contexte de tenant : {0}")]
    ContexteTenant(#[from] crate::tenant_context::ErreurContexteTenant),
}

impl ErreurBranding {
    pub fn code(&self) -> &'static str {
        match self {
            ErreurBranding::CouleurInvalide(_) => "couleur_invalide",
            ErreurBranding::LogoTropVolumineux { .. } => "logo_trop_volumineux",
            ErreurBranding::Stockage(_) => "stockage_indisponible",
            ErreurBranding::EtablissementInconnu => "etablissement_inconnu",
            _ => "erreur_interne",
        }
    }

    pub fn valeur(&self) -> Option<String> {
        match self {
            ErreurBranding::CouleurInvalide(v) => Some(v.clone()),
            ErreurBranding::LogoTropVolumineux { taille, .. } => Some(taille.to_string()),
            _ => None,
        }
    }
}

/// La couleur est-elle un hexadécimal `#RRGGBB` ?
///
/// Même contrôle qu'en base — la validation applicative existe pour rendre un `400` intelligible,
/// pas pour remplacer le `CHECK` : un import ou un script de reprise contournerait la première,
/// jamais le second.
pub fn couleur_valide(couleur: &str) -> bool {
    couleur.len() == 7
        && couleur.starts_with('#')
        && couleur[1..].chars().all(|c| c.is_ascii_hexdigit())
}
