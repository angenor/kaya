//! Types de l'identité d'**authentification**.
//!
//! # Deux structures pour une table, et c'est le sujet
//!
//! `comptes.compte` porte `condensat_mot_de_passe`. Une structure unique le ferait traverser
//! toutes les couches — service, handler, sérialisation — jusqu'au jour où quelqu'un ajouterait
//! `#[derive(Serialize)]` sur ce qui le porte, et le condensat partirait dans une réponse.
//!
//! D'où deux types **sans champ commun sensible** :
//!
//! | Type | Qui le lit | Porte le condensat |
//! |---|---|---|
//! | [`CompteAuthentification`] | le service d'authentification, et lui seul | **oui** |
//! | [`CompteVue`] | tout le reste, y compris l'API | **non**, et il n'a pas le champ |
//!
//! Ce n'est pas de la discipline : `CompteVue` **n'a pas de champ où le mettre**.

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use utoipa::ToSchema;
use uuid::Uuid;

/// Ce que la connexion a besoin de savoir, et **rien de plus**.
///
/// Volontairement **non sérialisable** : aucune dérivation `Serialize`. Un type qui porte un
/// condensat et sait se sérialiser finit sérialisé.
#[derive(Debug, Clone)]
pub struct CompteAuthentification {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub condensat_mot_de_passe: String,
    pub methode_code: String,
    pub actif: bool,
    pub personne_id: Uuid,
}

/// Un compte tel que l'API le rend — **sans condensat, par construction**.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CompteVue {
    pub id: Uuid,
    pub personne_id: Uuid,
    /// Nom affichable, lu de `personne`. **Jamais l'identifiant de connexion** : afficher un
    /// numéro de téléphone dans une liste consultable diffuserait un contact personnel.
    pub nom_affichage: String,
    pub identifiant_telephone: Option<String>,
    pub identifiant_email: Option<String>,
    pub methode_code: String,
    pub actif: bool,
    /// Les rôles portés, **avec leur établissement**. Un même compte peut être caissier ici et
    /// réceptionniste là ; une liste de codes sans établissement serait fausse.
    #[serde(default)]
    pub roles: Vec<RolePorte>,
    #[serde(with = "time::serde::rfc3339")]
    pub cree_le: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub modifie_le: OffsetDateTime,
}

/// Un rôle porté par un compte sur un établissement.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RolePorte {
    pub role_code: String,
    /// `None` pour `admin_editeur`, dont la portée est l'éditeur.
    pub etablissement_id: Option<Uuid>,
}

/// Clé du catalogue portant la longueur minimale du mot de passe.
///
/// Nommée ici plutôt qu'en littéral dans le service : c'est un **paramètre d'établissement**
/// (migration `0019`), et une chaîne recopiée à deux endroits finirait par en désigner deux.
pub const CLE_LONGUEUR_MIN: &str = "mot_de_passe_longueur_min";

/// Demande de création d'un compte.
#[derive(Debug, Clone)]
pub struct CreerCompte {
    /// UUID v7 **généré côté client**.
    pub id: Uuid,
    pub personne_id: Uuid,
    pub identifiant_telephone: Option<String>,
    pub identifiant_email: Option<String>,
    /// Le mot de passe **en clair**, ici et nulle part ailleurs. Il est haché par le service et
    /// n'atteint jamais le repository.
    pub mot_de_passe: String,
    pub horodatage_client: Option<OffsetDateTime>,
}

/// Échec du service des comptes.
#[derive(Debug, thiserror::Error)]
pub enum ErreurCompte {
    #[error("compte inconnu")]
    Inconnu,

    #[error("personne inconnue")]
    PersonneInconnue,

    /// Ni téléphone ni email.
    #[error("aucun identifiant fourni")]
    IdentifiantAbsent,

    /// **Le message ne dit pas que l'identifiant existe déjà.**
    ///
    /// Dire « ce numéro est déjà pris » à qui crée un compte apprendrait, à un habilité d'un
    /// tenant, quels numéros sont clients de Kaya. La tentative part au journal applicatif, où le
    /// support la retrouve.
    #[error("identifiant refusé")]
    IdentifiantRefuse,

    #[error("mot de passe refusé : {0}")]
    MotDePasseRefuse(#[from] crate::authentification::RefusMotDePasse),

    /// Le mot de passe actuel fourni ne correspond pas — cas du compte agissant sur lui-même.
    #[error("mot de passe actuel invalide")]
    MotDePasseActuelInvalide,

    #[error("accès aux données : {0}")]
    Base(#[from] sqlx::Error),

    #[error("contexte de tenant : {0}")]
    ContexteTenant(#[from] kaya_etablissements::tenant_context::ErreurContexteTenant),

    #[error("hachage : {0}")]
    Hachage(#[from] crate::authentification::ErreurHachage),

    #[error("registre des actions : {0}")]
    Audit(#[from] crate::audit::ErreurAudit),

    #[error("grand livre : {0}")]
    Outbox(#[from] kaya_synchronisation::ErreurOutbox),

    #[error("entrepôt des sessions : {0}")]
    Entrepot(#[from] redis::RedisError),
}
