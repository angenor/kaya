//! Corps d'erreur structuré — **nouveauté de ce cycle**.
//!
//! Le cycle 001 rendait des messages en clair. Ce cycle doit produire des refus que l'interface
//! **traduit** (porte P-16 : aucune chaîne utilisateur en dur), tout en **nommant la valeur
//! refusée** (FR-032, FR-033). Les deux exigences sont contradictoires si le serveur rend une
//! phrase : une phrase traduite ne peut pas nommer une valeur que seul le serveur connaît, et une
//! phrase française en dur ne se traduit pas.
//!
//! D'où trois champs, chacun pour un lecteur différent :
//!
//! | Champ | Lecteur | Règle |
//! |---|---|---|
//! | `code` | le **client**, qui branche sa clé i18n dessus | identifiant stable, **jamais traduit** |
//! | `valeur` | le **message composé**, qui nomme la chose refusée | la valeur telle qu'elle a été soumise |
//! | `message` | le **développeur** et les journaux | diagnostic. **Jamais affiché tel quel** |
//!
//! **Aucun détail interne ne franchit la frontière** : ni message PostgreSQL, ni nom de table, ni
//! trace. Le détail part dans les journaux, corrélé par identifiant de requête.

use actix_web::HttpResponse;
use serde::Serialize;
use utoipa::ToSchema;

/// Corps rendu par tout refus métier de ce cycle.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CorpsErreur {
    /// Identifiant stable — `capacite_non_implementee`, `module_non_actif`, `portee_interdite`…
    /// **Jamais traduit** : c'est sur lui que le client branche sa clé i18n.
    pub code: String,
    /// Ce qui a été refusé, quand il y a quelque chose à nommer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valeur: Option<String>,
    /// Diagnostic pour les journaux et le développeur. **Jamais affiché tel quel.**
    pub message: String,
    /// Clé i18n d'un motif explicatif, quand le référentiel en fournit un.
    ///
    /// C'est ce qui permet à `AUCUN` de dire « une capacité non consommée ne se déclare pas » là
    /// où `VALORISE` dit « pas encore implémenté » — deux refus dont l'un enseigne et l'autre
    /// constate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub motif_cle: Option<String>,
    /// Obstacles à une désactivation, chacun avec son motif et son nombre.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub obstacles: Vec<ObstacleVue>,
}

/// Un obstacle à la désactivation d'un service, tel que l'API le rend.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ObstacleVue {
    pub module_code: String,
    /// **Clé i18n — jamais une phrase.**
    pub motif_cle: String,
    /// Séparé du motif pour que la phrase se compose dans la langue de l'utilisateur, où le
    /// pluriel ne s'accorde pas partout de la même façon.
    pub nombre: u32,
}

impl CorpsErreur {
    pub fn nouveau(code: &str, valeur: Option<String>, message: String) -> Self {
        Self {
            code: code.to_owned(),
            valeur,
            message,
            motif_cle: None,
            obstacles: Vec::new(),
        }
    }

    pub fn avec_motif(mut self, motif_cle: Option<&str>) -> Self {
        self.motif_cle = motif_cle.filter(|m| !m.is_empty()).map(str::to_owned);
        self
    }

    pub fn avec_obstacles(mut self, obstacles: Vec<ObstacleVue>) -> Self {
        self.obstacles = obstacles;
        self
    }

    /// `422 Unprocessable Entity` — la requête est bien formée, la règle métier la refuse.
    pub fn en_422(self) -> actix_web::Error {
        actix_web::error::InternalError::from_response(
            self.message.clone(),
            HttpResponse::UnprocessableEntity().json(self),
        )
        .into()
    }

    /// `400 Bad Request` — la requête est mal formée.
    pub fn en_400(self) -> actix_web::Error {
        actix_web::error::InternalError::from_response(
            self.message.clone(),
            HttpResponse::BadRequest().json(self),
        )
        .into()
    }

    /// `401 Unauthorized` — l'appelant n'est pas identifié.
    ///
    /// **`401`, jamais `400`** : le client doit réessayer après authentification, pas corriger sa
    /// requête. La distinction décide de ce que fait le front — rediriger vers l'écran de
    /// connexion, ou afficher une erreur de saisie.
    pub fn en_401(self) -> actix_web::Error {
        actix_web::error::InternalError::from_response(
            self.message.clone(),
            HttpResponse::Unauthorized().json(self),
        )
        .into()
    }

    /// `403 Forbidden` — l'appelant est identifié et n'a pas la permission.
    ///
    /// **L'interface ne devrait jamais le provoquer** (FR-026) : une action sans permission est
    /// *absente*, pas refusée. Ce code existe pour l'appel direct, pas pour le parcours normal.
    pub fn en_403(self) -> actix_web::Error {
        actix_web::error::InternalError::from_response(
            self.message.clone(),
            HttpResponse::Forbidden().json(self),
        )
        .into()
    }

    /// `409 Conflict` — la règle métier refuse, et l'état actuel est la raison.
    pub fn en_409(self) -> actix_web::Error {
        actix_web::error::InternalError::from_response(
            self.message.clone(),
            HttpResponse::Conflict().json(self),
        )
        .into()
    }

    /// `404 Not Found`.
    pub fn en_404(self) -> actix_web::Error {
        actix_web::error::InternalError::from_response(
            self.message.clone(),
            HttpResponse::NotFound().json(self),
        )
        .into()
    }

    /// `413 Payload Too Large` — **le message donne la limite**, jamais un refus muet.
    pub fn en_413(self) -> actix_web::Error {
        actix_web::error::InternalError::from_response(
            self.message.clone(),
            HttpResponse::PayloadTooLarge().json(self),
        )
        .into()
    }
}

/// Erreur interne — **le détail part dans les journaux, pas dans la réponse**.
pub fn interne(contexte: &str, detail: impl std::fmt::Display) -> actix_web::Error {
    tracing::error!(erreur = %detail, contexte, "échec interne");
    CorpsErreur::nouveau(
        "erreur_interne",
        None,
        "erreur interne — voir les journaux, corrélés par identifiant de requête".to_owned(),
    )
    .en_500()
}

impl CorpsErreur {
    fn en_500(self) -> actix_web::Error {
        actix_web::error::InternalError::from_response(
            self.message.clone(),
            HttpResponse::InternalServerError().json(self),
        )
        .into()
    }
}
