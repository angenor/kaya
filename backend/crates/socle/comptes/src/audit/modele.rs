//! Types du registre des actions.
//!
//! Terme utilisateur : **« Registre des actions »** / *Activity log* (`docs/design/lexique.md`).
//! `journal_audit` reste le nom **technique** — table, permission, endpoint — et n'apparaît jamais
//! à l'écran.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;
use utoipa::ToSchema;
use uuid::Uuid;

pub use super::taxonomie::TypeActionAudit;

/// Une entrée à écrire au registre.
///
/// **Trois champs manquent volontairement**, et c'est la même décision qu'`EvenementAEcrire` du
/// cycle 001 :
///
///   * `tenant_id` — il vient du contexte d'authentification, jamais de l'appelant ;
///   * `cree_le` — posé par la base, autorité serveur (principe IV) ;
///   * `auteur_compte_id` **est** exigé, lui, parce qu'un service peut tracer une action faite
///     par un compte qui n'est pas l'appelant — c'est le cas du retrait de rôle.
#[derive(Debug, Clone)]
pub struct EntreeAudit {
    /// UUID v7 **généré côté client**. C'est lui qui rend le rejeu inoffensif : trois soumissions
    /// de la même entrée produisent un enregistrement.
    pub id: Uuid,
    /// `None` pour une action de portée tenant ou éditeur.
    pub etablissement_id: Option<Uuid>,
    pub type_action: TypeActionAudit,
    pub auteur_compte_id: Uuid,
    /// « compte », « unite », « ligne_vente »… Libre, parce que les cibles appartiennent à des
    /// modules qui n'existent pas encore.
    pub cible_type: String,
    pub cible_id: Option<Uuid>,
    /// Document JSON. **Toute clé monétaire y porte le suffixe `_mineur`, une valeur entière, et
    /// une clé `devise` au même niveau d'objet** — voir [`super::service::valider_contexte`].
    pub contexte: Value,
    /// Indicatif. **Jamais un critère de tri.**
    pub horodatage_client: Option<OffsetDateTime>,
}

/// Une entrée telle qu'elle est en base, prête à être rendue par l'API.
///
/// L'auteur y figure par son identifiant seul : son **nom** est résolu en lot par le trait
/// `AnnuaireComptes`, à la lecture. Le dénormaliser ici obligerait à choisir entre un nom figé au
/// moment de l'écriture — donc faux après un mariage — et une jointure inter-modules, que le
/// principe II interdit.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EntreeAuditEnregistree {
    pub id: Uuid,
    pub etablissement_id: Option<Uuid>,
    pub type_action: TypeActionAudit,
    pub auteur_compte_id: Uuid,
    pub cible_type: String,
    pub cible_id: Option<Uuid>,
    pub contexte: Value,
    /// Indicatif — rendu **à part**, et jamais présenté comme la date de l'action.
    #[serde(with = "time::serde::rfc3339::option")]
    pub horodatage_client: Option<OffsetDateTime>,
    /// Horodatage d'**autorité serveur**. C'est celui que l'écran `G4` affiche.
    #[serde(with = "time::serde::rfc3339")]
    pub cree_le: OffsetDateTime,
}

/// Échec d'écriture au registre.
#[derive(Debug, thiserror::Error)]
pub enum ErreurAudit {
    #[error("écriture au registre des actions impossible : {0}")]
    Base(#[from] sqlx::Error),

    /// Le document `contexte` viole la convention monétaire de la porte P-10 étendue.
    ///
    /// **Distincte d'une erreur de base, et volontairement bruyante** : elle signale du code
    /// fautif, pas une panne. Une entrée refusée ici n'atteint jamais le registre — donc
    /// l'opération tracée est annulée avec elle, ce qui est le comportement voulu. Un montant
    /// faux dans le registre qui sert à détecter les fraudes est pire qu'une opération refusée.
    #[error("contexte d'audit invalide : {0}")]
    ContexteInvalide(String),
}
