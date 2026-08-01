//! Types de l'identité civile.

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use utoipa::ToSchema;
use uuid::Uuid;

/// Une personne telle qu'elle est en base.
///
/// **Ni `type_piece`, ni `numero_piece`.** Les colonnes existent, ce type ne les porte pas — voir
/// le commentaire de tête du module. Un champ posé ici serait rempli par le premier handler qui
/// en aurait l'occasion.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Personne {
    pub id: Uuid,
    pub nom: String,
    pub prenoms: Option<String>,
    /// E.164. **Aucune contrainte de format national** : l'indicatif par défaut est un paramètre
    /// d'établissement (`indicatif_telephonique_defaut`), pas une règle de code.
    pub telephone: Option<String>,
    pub email: Option<String>,
    /// Indicatif — ordre d'affichage local. **Jamais un critère de tri ni de calcul.**
    #[serde(with = "time::serde::rfc3339::option")]
    pub horodatage_client: Option<OffsetDateTime>,
    /// Horodatage d'**autorité serveur**.
    #[serde(with = "time::serde::rfc3339")]
    pub cree_le: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub modifie_le: OffsetDateTime,
}

/// Demande de création.
///
/// `tenant_id` n'y figure pas : il vient du contexte d'authentification, jamais de l'appelant.
#[derive(Debug, Clone)]
pub struct CreerPersonne {
    /// UUID v7 **généré côté client** (principe VI) — c'est lui qui rend le rejeu inoffensif.
    pub id: Uuid,
    pub nom: String,
    pub prenoms: Option<String>,
    pub telephone: Option<String>,
    pub email: Option<String>,
    pub horodatage_client: Option<OffsetDateTime>,
}

/// Demande de modification.
///
/// **Remplacement complet, pas de fusion champ par champ.** Un `PUT` qui fusionnerait rendrait
/// impossible d'effacer un numéro de téléphone : l'absence du champ et sa mise à `null` seraient
/// indistinguables. Le contrat annonce un `PUT`, et c'en est un.
#[derive(Debug, Clone)]
pub struct ModifierPersonne {
    pub nom: String,
    pub prenoms: Option<String>,
    pub telephone: Option<String>,
    pub email: Option<String>,
    pub horodatage_client: Option<OffsetDateTime>,
}

/// Longueur maximale du nom — **alignée sur le `CHECK` de la migration `0015`**.
///
/// La validation applicative existe pour rendre un `400` intelligible, pas pour remplacer la
/// contrainte de base : un script de maintenance contournerait la première, jamais la seconde.
pub const NOM_MAX: usize = 200;

/// Échec du service des personnes.
#[derive(Debug, thiserror::Error)]
pub enum ErreurPersonne {
    #[error("nom invalide : entre 1 et {NOM_MAX} caractères après nettoyage")]
    NomInvalide,

    #[error("personne inconnue")]
    Inconnue,

    #[error("accès aux données : {0}")]
    Base(#[from] sqlx::Error),

    #[error("contexte de tenant : {0}")]
    ContexteTenant(#[from] kaya_etablissements::tenant_context::ErreurContexteTenant),

    #[error("grand livre : {0}")]
    Outbox(#[from] kaya_synchronisation::ErreurOutbox),
}
