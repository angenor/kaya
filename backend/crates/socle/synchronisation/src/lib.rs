//! `socle/synchronisation` — le **grand livre d'événements**.
//!
//! Ce crate porte le schéma `synchronisation`, la table `evenement_outbox`, les traits
//! [`OutboxWriter`] et [`EventConsumer`], et le worker de publication in-process.
//!
//! **L'outbox n'est pas une file de messages** (principe II) : rétention illimitée, charge utile
//! financière complète et dénormalisée, immuable. Une correction est un nouvel événement, jamais
//! une modification de l'ancien.

#![forbid(unsafe_code)]

pub mod consommateurs;
pub mod outbox;
pub mod worker;

use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;

/// Événement à inscrire au grand livre.
///
/// **Deux champs manquent volontairement à cette structure** : `survenu_le` et
/// `sequence_etablissement`. Ils sont posés par l'implémentation, côté serveur. Les exposer
/// laisserait un appelant fournir l'horloge de son terminal — ce que le principe IV interdit —
/// ou casser la monotonie de la séquence.
#[derive(Debug, Clone)]
pub struct EvenementAEcrire {
    /// UUID v7 **généré côté client**. C'est lui qui rend le rejeu inoffensif.
    pub id: Uuid,
    pub tenant_id: Uuid,
    /// `None` pour un événement de niveau tenant.
    pub etablissement_id: Option<Uuid>,
    /// Par exemple `note_etablissement.creee`.
    pub type_evenement: String,
    pub agregat: String,
    pub agregat_id: Uuid,
    /// Version du **format** de `payload` (R-06).
    pub version_schema: i16,
    /// Charge utile **complète et dénormalisée**.
    pub payload: Value,
}

/// Événement lu depuis le grand livre et présenté à un consommateur.
#[derive(Debug, Clone)]
pub struct EvenementPublie {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub etablissement_id: Option<Uuid>,
    pub sequence_etablissement: i64,
    pub type_evenement: String,
    pub agregat: String,
    pub agregat_id: Uuid,
    pub version_schema: i16,
    pub payload: Value,
    pub survenu_le: OffsetDateTime,
}

/// Échec d'écriture au grand livre.
#[derive(Debug, thiserror::Error)]
pub enum ErreurOutbox {
    #[error("échec d'écriture de l'événement au grand livre : {0}")]
    Base(#[from] sqlx::Error),
}

/// Échec de consommation d'un événement publié.
///
/// Un consommateur qui échoue laisse l'événement **non marqué publié**, donc republié au tour
/// suivant, indéfiniment. Aucun événement n'est jamais abandonné ni supprimé.
#[derive(Debug, thiserror::Error)]
pub enum ErreurConsommation {
    #[error("le consommateur « {consommateur} » a échoué : {motif}")]
    Traitement {
        consommateur: &'static str,
        motif: String,
    },
}

/// Écriture d'un événement **dans la transaction fournie**.
///
/// # Ce que la signature garantit, et pourquoi elle est écrite ainsi
///
/// `ecrire` **prend une transaction en paramètre et n'ouvre jamais la sienne**. C'est le
/// mécanisme de la porte **P-05**, pas une convention de style : il devient impossible d'écrire
/// un événement hors de la transaction métier.
///
/// Un trait qui prendrait un pool de connexions laisserait le développeur libre d'ouvrir une
/// seconde transaction, et la garantie « ligne métier et événement dans la même transaction SQL »
/// de TRX-02 reposerait sur sa discipline. Ici elle repose sur le compilateur.
///
/// # Pourquoi `#[async_trait]` plutôt qu'un `async fn` natif
///
/// Rust sait écrire `async fn` dans un trait depuis 1.75, mais un tel trait n'est pas
/// dyn-compatible : `Arc<dyn OutboxWriter>` ne compilerait pas. Or l'injection de dépendances
/// (cadrage §13.2) suppose exactement cela. L'annotation est donc un choix contraint, pas une
/// habitude reprise d'un exemple.
#[async_trait::async_trait]
pub trait OutboxWriter: Send + Sync {
    async fn ecrire(
        &self,
        tx: &mut sqlx::PgTransaction<'_>,
        evenement: EvenementAEcrire,
    ) -> Result<(), ErreurOutbox>;
}

/// Consommateur d'événements publiés.
///
/// **L'idempotence est une obligation, pas une qualité** : trois présentations du même événement
/// doivent produire l'effet d'une seule. Un redémarrage brutal du worker republie ce qui n'a pas
/// été marqué (R-08) — sans idempotence, chaque redémarrage dupliquerait des effets.
#[async_trait::async_trait]
pub trait EventConsumer: Send + Sync {
    fn nom(&self) -> &'static str;

    async fn consommer(&self, evenement: &EvenementPublie) -> Result<(), ErreurConsommation>;
}
