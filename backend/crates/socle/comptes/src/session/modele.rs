//! Types des sessions.
//!
//! Terme utilisateur : **« Appareil connecté »** / *Connected device* (`docs/design/lexique.md`).
//! Les mots « session », « jeton » et « JWT » n'atteignent jamais l'interface.

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use utoipa::ToSchema;
use uuid::Uuid;

/// Durées de repli, **égales aux défauts du catalogue** (migration `0019`).
///
/// Elles ne servent que si le paramètre est illisible. Un repli différent du défaut documenté ne
/// se manifesterait qu'en cas de panne de lecture de configuration — c'est-à-dire jamais en test,
/// et une fois en production.
pub const ACCES_DUREE_MIN_DEFAUT: i64 = 60;
pub const RAFRAICHISSEMENT_DUREE_JOURS_DEFAUT: i64 = 90;

/// Clés du catalogue portant ces durées.
pub const CLE_ACCES_DUREE: &str = "jeton_acces_duree_min";
pub const CLE_RAFRAICHISSEMENT_DUREE: &str = "jeton_rafraichissement_duree_jours";

/// Une session telle qu'elle vit en Redis.
///
/// **Aucune table.** Les sessions sont éphémères et reconstructibles : Redis vidé, tout le monde
/// se reconnecte et aucune donnée métier ne manque. Elles ne figurent donc ni au registre des
/// classes hors-ligne, ni dans les sauvegardes (data-model, vue d'ensemble).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub compte_id: Uuid,
    pub tenant_id: Uuid,
    /// L'établissement actif au moment de l'ouverture. Le sélecteur permanent est **ETB-06**.
    pub etablissement_id: Option<Uuid>,
    /// La famille de jetons de rafraîchissement — c'est elle qu'on révoque en bloc sur détection
    /// de réutilisation.
    pub famille_id: Uuid,
    /// Libellé d'appareil, fourni par le client. Purement indicatif : il sert à ce que
    /// l'utilisateur reconnaisse **son** téléphone dans la liste avant de couper l'autre.
    pub libelle_appareil: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub ouverte_le: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub derniere_activite_le: OffsetDateTime,
    /// Au-delà, la session est **absente** même si sa donnée traîne encore dans le hachage.
    #[serde(with = "time::serde::rfc3339")]
    pub expire_le: OffsetDateTime,
}

/// Une session telle que l'API la rend — **sans rien qui permette de la rejouer**.
///
/// Ni `famille_id`, ni jeton, ni condensat. Cette structure part dans une réponse HTTP ; y laisser
/// de quoi reconstruire un jeton reviendrait à publier la session qu'on donne à révoquer.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SessionVue {
    pub id: Uuid,
    pub libelle_appareil: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub ouverte_le: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub derniere_activite_le: OffsetDateTime,
    /// **Vrai pour la session qui fait l'appel.** Sans ce drapeau, l'écran ne saurait pas
    /// laquelle est « cet appareil-ci », et l'utilisateur se déconnecterait lui-même en croyant
    /// couper le téléphone perdu.
    pub courante: bool,
}

impl Session {
    pub fn en_vue(&self, session_courante: Uuid) -> SessionVue {
        SessionVue {
            id: self.id,
            libelle_appareil: self.libelle_appareil.clone(),
            ouverte_le: self.ouverte_le,
            derniere_activite_le: self.derniere_activite_le,
            courante: self.id == session_courante,
        }
    }
}

/// Le couple de jetons délivré par une ouverture ou un rafraîchissement.
#[derive(Debug, Clone)]
pub struct JetonsDelivres {
    pub acces: String,
    /// Durée de vie du jeton d'accès, en secondes — ce que le client met dans son minuteur.
    pub expire_dans_s: i64,
    pub rafraichissement: String,
    pub session_id: Uuid,
}

/// Échec du service des sessions.
#[derive(Debug, thiserror::Error)]
pub enum ErreurSession {
    /// **Le seul refus d'authentification du produit.**
    ///
    /// Il ne distingue jamais compte inconnu, mot de passe faux, compte désactivé ni dépassement
    /// de tentatives (FR-012). La distinction dirait à qui essaie si un numéro est client de
    /// Kaya — et la liste du personnel d'un hôtel est publique sur sa porte.
    #[error("identifiants invalides")]
    IdentifiantsInvalides,

    /// Jeton de rafraîchissement inconnu, révoqué ou **déjà consommé**.
    #[error("session invalide")]
    SessionInvalide,

    /// Le compte est réglé sur une méthode connue mais non servie — `OTP_SMS`.
    ///
    /// **Refus nommé, jamais un repli silencieux sur le mot de passe** (FR-008). Le repli
    /// donnerait accès par un moyen que l'exploitant croit avoir désactivé.
    #[error("méthode d'authentification non implémentée : {0}")]
    MethodeNonImplementee(String),

    #[error("accès aux données : {0}")]
    Base(#[from] sqlx::Error),

    #[error("contexte de tenant : {0}")]
    ContexteTenant(#[from] kaya_etablissements::tenant_context::ErreurContexteTenant),

    #[error("entrepôt des sessions : {0}")]
    Entrepot(#[from] redis::RedisError),

    #[error("signature des jetons : {0}")]
    Jeton(#[from] jsonwebtoken::errors::Error),

    #[error("hachage : {0}")]
    Hachage(#[from] crate::authentification::ErreurHachage),

    #[error("registre des actions : {0}")]
    Audit(#[from] crate::audit::ErreurAudit),

    #[error("grand livre : {0}")]
    Outbox(#[from] kaya_synchronisation::ErreurOutbox),
}
