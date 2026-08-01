//! Types du cumul de rôles.

use uuid::Uuid;

/// Portée d'un rôle — **et la contrainte qu'elle impose à l'attribution**.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PorteeRole {
    /// Le rôle s'attribue **sur un établissement**, qui est obligatoire.
    Etablissement,
    /// Le rôle vaut pour l'éditeur : `etablissement_id` est **interdit**.
    Editeur,
}

impl PorteeRole {
    pub fn depuis_code(code: &str) -> Option<Self> {
        match code {
            "ETABLISSEMENT" => Some(PorteeRole::Etablissement),
            "EDITEUR" => Some(PorteeRole::Editeur),
            _ => None,
        }
    }

    /// La portée est-elle compatible avec l'établissement fourni ?
    ///
    /// C'est la règle du `422 portee_incompatible` du contrat, écrite **une fois** : un
    /// `etablissement_id` sur `admin_editeur`, ou son absence sur un rôle d'établissement.
    pub fn compatible(self, etablissement_id: Option<Uuid>) -> bool {
        match self {
            PorteeRole::Etablissement => etablissement_id.is_some(),
            PorteeRole::Editeur => etablissement_id.is_none(),
        }
    }
}

/// Une attribution de rôle demandée.
#[derive(Debug, Clone)]
pub struct AttribuerRole {
    /// UUID v7 **généré côté client**.
    pub id: Uuid,
    pub compte_id: Uuid,
    pub role_code: String,
    pub etablissement_id: Option<Uuid>,
    pub horodatage_client: Option<time::OffsetDateTime>,
}

/// Une entrée du référentiel des rôles ou des permissions, telle que l'API la rend.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct EntreeReferentielRole {
    pub code: String,
    /// **Clé i18n, jamais un libellé** : une chaîne stockée en base échapperait à la porte P-16.
    pub libelle_cle: String,
    pub ordre: i16,
    /// `ETABLISSEMENT` ou `EDITEUR` pour un rôle ; absent pour une permission.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub portee: Option<String>,
    /// Module d'activité d'une permission. `None` = transversale.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module_code: Option<String>,
}

/// Échec du service des rôles.
#[derive(Debug, thiserror::Error)]
pub enum ErreurRoles {
    #[error("rôle inconnu : {0}")]
    RoleInconnu(String),

    #[error("compte inconnu")]
    CompteInconnu,

    /// Vérifié **par trait**, jamais par clé étrangère inter-schémas.
    #[error("établissement inconnu")]
    EtablissementInconnu,

    /// `etablissement_id` fourni pour un rôle d'éditeur, ou absent pour un rôle d'établissement.
    #[error("portée incompatible")]
    PorteeIncompatible,

    /// **Le seul refus métier du cycle.**
    ///
    /// Le retrait laisserait l'établissement sans aucun compte habilité à attribuer les rôles
    /// (FR-023). Irréversible sans l'éditeur, d'où un code propre plutôt qu'un `403`.
    #[error("dernière habilitation de l'établissement")]
    DerniereHabilitation,

    #[error("accès aux données : {0}")]
    Base(#[from] sqlx::Error),

    #[error("contexte de tenant : {0}")]
    ContexteTenant(#[from] kaya_etablissements::tenant_context::ErreurContexteTenant),

    #[error("registre des actions : {0}")]
    Audit(#[from] crate::audit::ErreurAudit),

    #[error("grand livre : {0}")]
    Outbox(#[from] kaya_synchronisation::ErreurOutbox),
}
