//! Types de la note d'établissement.
//!
//! Terme utilisateur : **« Note interne »** en français, *Internal note* en anglais
//! (`docs/design/lexique.md`). Le mot « établissement » est superflu dans le libellé, le lexique
//! posant déjà que l'utilisateur est toujours dans le sien. `note_etablissement` reste le nom
//! **technique** — table, type, événement — et n'apparaît jamais à l'écran.

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use utoipa::ToSchema;
use uuid::Uuid;

/// Une note interne, telle qu'elle existe en base.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NoteEtablissement {
    pub id: Uuid,
    pub etablissement_id: Uuid,
    pub auteur_compte_id: Uuid,
    pub texte: String,
    /// Indicatif — ordre d'affichage local. **Jamais utilisé par une règle métier.**
    #[serde(with = "time::serde::rfc3339::option")]
    pub horodatage_client: Option<OffsetDateTime>,
    /// Horodatage d'**autorité serveur**. C'est lui qui fait foi.
    #[serde(with = "time::serde::rfc3339")]
    pub cree_le: OffsetDateTime,
}

/// Demande de création d'une note.
///
/// `tenant_id` n'y figure pas : il vient du contexte d'authentification, jamais du corps de la
/// requête. Le laisser fournir par l'appelant offrirait à un client la possibilité d'écrire chez
/// un autre — que la politique `WITH CHECK` refuserait, mais une défense en profondeur commence
/// par ne pas poser la question.
#[derive(Debug, Clone)]
pub struct CreerNote {
    /// UUID v7 **généré par le client**. C'est lui qui rend le rejeu inoffensif.
    pub id: Uuid,
    pub etablissement_id: Uuid,
    pub auteur_compte_id: Uuid,
    pub texte: String,
    pub horodatage_client: Option<OffsetDateTime>,
}

/// Résultat d'une création — **la distinction que le contrat HTTP transforme en 201 ou 200**.
///
/// Un rejeu n'est pas une erreur : c'est le comportement normal d'un terminal qui vide sa file
/// après une coupure. Répondre `409` obligerait chaque appelant à traiter comme un échec une
/// écriture que le serveur a déjà acceptée (principe VI).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Issue {
    /// La note n'existait pas — `201 Created`.
    Creee,
    /// La note existait déjà, à l'identique — `200 OK`, corps = la note en base.
    DejaPresente,
}

/// Erreur du domaine des notes.
#[derive(Debug, thiserror::Error)]
pub enum ErreurNote {
    #[error("texte vide ou trop long : entre 1 et 2000 caractères après nettoyage")]
    TexteInvalide,

    #[error("établissement inconnu ou hors du tenant courant")]
    EtablissementInconnu,

    #[error("erreur de base : {0}")]
    Base(#[from] sqlx::Error),

    #[error("écriture au grand livre : {0}")]
    Outbox(#[from] kaya_synchronisation::ErreurOutbox),

    #[error("contexte de tenant : {0}")]
    ContexteTenant(#[from] crate::tenant_context::ErreurContexteTenant),
}
