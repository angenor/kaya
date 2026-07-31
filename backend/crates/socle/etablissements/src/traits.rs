//! **Les six traits exposés par `socle/etablissements`.**
//!
//! Ils sont le **seul chemin** par lequel les autres crates lisent ce que ce cycle produit :
//! aucune requête ne joint deux schémas de modules (principe II, porte P-04), et aucun crate du
//! socle ne dépend d'une verticale (porte P-03).
//!
//! Cinq sont des traits de **lecture**, implémentés ici et consommés ailleurs. Le sixième —
//! [`ObstacleDesactivation`] — inverse le sens, et c'est le seul point de conception délicat du
//! cycle.
//!
//! # Pourquoi `#[async_trait::async_trait]` partout
//!
//! Rust sait écrire `async fn` dans un trait depuis 1.75, mais un tel trait **n'est pas
//! dyn-compatible**. L'injection de dépendances du cadrage §13.2 suppose `Arc<dyn Trait>` :
//! l'annotation est un choix contraint, pas une habitude reprise d'un exemple. Même contrainte
//! qu'au cycle 001 sur `OutboxWriter`.
//!
//! # Ce que ces traits n'exposent volontairement pas
//!
//! | Absent | Raison |
//! |---|---|
//! | Écriture au référentiel des modules ou des capacités | Réservée à l'éditeur (ETB-08). Aucun tenant n'y écrit, donc aucun trait ne l'offre |
//! | Liste des modules **inactifs** | Ce que l'interface ne doit pas montrer, elle ne doit pas recevoir (principe VII) |
//! | Liste des capacités non implémentées | Idem. Le refus explicite protège les chemins d'écriture, il n'alimente pas l'interface |
//! | Valeur par défaut d'un paramètre | Serait un paramètre en dur (principe I·c) |
//! | Accès au binaire du logo | Le trait rend une clé d'objet ; le contenu passe par l'interface S3 (principe II) |

use std::collections::BTreeMap;

use uuid::Uuid;

use crate::{ErreurLecture, Etablissement};

// =================================================================================================
//  Erreurs communes
// =================================================================================================

/// Échec de lecture d'un registre — modules, capacités, points de vente.
#[derive(Debug, thiserror::Error)]
pub enum ErreurRegistre {
    #[error("lecture du registre impossible : {0}")]
    Base(#[from] sqlx::Error),

    #[error("contexte de tenant : {0}")]
    ContexteTenant(#[from] crate::tenant_context::ErreurContexteTenant),
}

/// Échec de résolution de configuration.
#[derive(Debug, thiserror::Error)]
pub enum ErreurConfiguration {
    #[error("lecture de la configuration impossible : {0}")]
    Base(#[from] sqlx::Error),

    #[error("contexte de tenant : {0}")]
    ContexteTenant(#[from] crate::tenant_context::ErreurContexteTenant),

    /// La valeur stockée ne se décode pas selon le type déclaré au catalogue. Distincte d'une
    /// erreur de base : elle signale une donnée écrite avant une évolution du catalogue, pas une
    /// panne.
    #[error("valeur illisible pour la clé « {cle} » : {detail}")]
    ValeurIllisible { cle: String, detail: String },
}

// =================================================================================================
//  1. EstablishmentDirectory — étendu par ETB-01
// =================================================================================================

/// **Le trait par lequel les autres modules lisent un établissement — jamais par jointure.**
///
/// Posé **à vide** au cycle 001 pour que le premier `JOIN` inter-schémas ne soit pas écrit « juste
/// cette fois » au cycle HEB. Ce cycle lui donne son contenu réel : [`Etablissement`] porte
/// désormais la juridiction, le classement, la commune, l'adresse et le NCC.
///
/// **Consommateurs** : tout crate qui a besoin d'un fuseau, d'une devise ou d'un classement —
/// `socle/fiscalite` (barème de nuitée), `socle/caisse` (clôture en temps local),
/// `verticales/hebergement` (calcul de durée).
#[async_trait::async_trait]
pub trait EstablishmentDirectory: Send + Sync {
    async fn etablissement(&self, id: Uuid) -> Result<Option<Etablissement>, ErreurLecture>;

    async fn appartient_au_tenant(
        &self,
        etablissement_id: Uuid,
        tenant_id: Uuid,
    ) -> Result<bool, ErreurLecture>;
}

// =================================================================================================
//  2. RegistreModules — quels services un établissement rend
// =================================================================================================

/// Les services **actifs** d'un établissement.
///
/// # Le trait ne rend jamais les modules inactifs
///
/// Une méthode `tous_les_modules_avec_etat` serait la porte d'entrée du grisé que le principe VII
/// interdit : donnée à l'interface, elle produirait une liste où figurent les services que
/// l'établissement n'a pas — et un jour quelqu'un les afficherait « pour information ».
///
/// **Ce que l'interface ne doit pas montrer, elle ne doit pas non plus recevoir.**
///
/// **Consommateurs** : l'accueil à tuiles (cycle CPT), chaque verticale au démarrage d'une
/// opération, la console éditeur.
#[async_trait::async_trait]
pub trait RegistreModules: Send + Sync {
    /// Codes des modules **actifs** de l'établissement, dans l'ordre d'affichage du référentiel.
    async fn modules_actifs(&self, etablissement_id: Uuid) -> Result<Vec<String>, ErreurRegistre>;

    /// Ce module est-il actif ici ? Réponse binaire, **sans exception si le module n'existe pas** :
    /// un code inconnu et un code non activé sont la même chose pour l'appelant.
    async fn module_actif(
        &self,
        etablissement_id: Uuid,
        code: &str,
    ) -> Result<bool, ErreurRegistre>;
}

// =================================================================================================
//  3. RegistreCapacites — ce qu'un service consomme
// =================================================================================================

/// Une capacité déclarée par un service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapaciteDeclaree {
    /// `STOCK` seule au MVP.
    pub capacite_code: String,
    /// `SIMPLE` seul au MVP.
    pub profil_code: String,
}

/// Ce qu'un service consomme comme capacité transverse.
///
/// **Consommateur unique au MVP** : `capacites/stocks`, qui n'agit que si `STOCK` est déclarée au
/// profil `SIMPLE`.
#[async_trait::async_trait]
pub trait RegistreCapacites: Send + Sync {
    async fn capacites_du_module(
        &self,
        etablissement_id: Uuid,
        module_code: &str,
    ) -> Result<Vec<CapaciteDeclaree>, ErreurRegistre>;

    /// Ce service consomme-t-il cette capacité, et **à quel profil** ?
    ///
    /// Rend `Option<CapaciteDeclaree>` plutôt qu'un `bool` : c'est le profil qui décide du
    /// comportement, et un booléen obligerait à un second appel — donc à deux vérités possibles
    /// entre les deux.
    async fn consomme(
        &self,
        etablissement_id: Uuid,
        module_code: &str,
        capacite_code: &str,
    ) -> Result<Option<CapaciteDeclaree>, ErreurRegistre>;
}

// =================================================================================================
//  4. ResolveurConfiguration — le composant le plus réutilisé du produit
// =================================================================================================

/// Niveau de la chaîne d'héritage, du plus général au plus spécifique.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Portee {
    Tenant,
    Etablissement,
    Module,
    PointDeVente,
}

impl Portee {
    /// Rang de spécificité — le plus élevé gagne à la résolution.
    ///
    /// Il existe ici pour l'appelant qui compare deux origines ; **la résolution, elle, le calcule
    /// en SQL** depuis les colonnes renseignées, jamais depuis une colonne stockée qui pourrait
    /// diverger de la réalité des clés étrangères.
    pub fn rang(self) -> u8 {
        match self {
            Portee::Tenant => 0,
            Portee::Etablissement => 1,
            Portee::Module => 2,
            Portee::PointDeVente => 3,
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Portee::Tenant => "TENANT",
            Portee::Etablissement => "ETABLISSEMENT",
            Portee::Module => "MODULE",
            Portee::PointDeVente => "POINT_DE_VENTE",
        }
    }
}

/// D'où l'on résout.
///
/// **Les `Option` absents raccourcissent la chaîne sans l'inventer** : un établissement sans point
/// de vente résout sur trois niveaux, et aucun niveau fictif n'est fabriqué pour compléter la
/// descente (FR-050).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cible {
    pub tenant_id: Uuid,
    pub etablissement_id: Option<Uuid>,
    pub module_code: Option<String>,
    pub point_de_vente_id: Option<Uuid>,
}

/// Une valeur résolue, **avec son origine**.
#[derive(Debug, Clone, PartialEq)]
pub struct ValeurResolue {
    pub valeur: serde_json::Value,
    /// **OBLIGATOIRE.** L'écran distingue « vaut pour tous vos établissements » de « modifié ici »
    /// (docs/design/lexique.md). Un champ optionnel serait ignoré par le premier appelant pressé,
    /// et la distinction disparaîtrait de l'interface sans que personne ne l'ait retirée.
    pub origine: Portee,
}

/// La chaîne d'héritage tenant → établissement → service → point de vente.
///
/// Trois choix, chacun contre une faute précise :
///
/// - **`Option<ValeurResolue>`, jamais un défaut.** Un défaut rendu par le résolveur serait un
///   paramètre en dur déguisé en commodité, et le principe I·c l'interdit. L'appelant qui a besoin
///   d'un défaut le déclare chez lui, où on peut le voir.
/// - **`origine` non optionnelle** — voir [`ValeurResolue`].
/// - **`resoudre_tout`** — l'écran `G1` affichera une trentaine de paramètres à terme ; trente
///   appels feraient trente descentes de chaîne.
///
/// **Consommateurs** : tous les cycles suivants. HEB (temps de remise en état, heures standard,
/// barème de passage), FIS (taux, taxes), CAI (seuil d'écart), IMP (politique d'impression),
/// STK (seuil d'alerte), RSV (expiration), QRC (paniers max), CPT (rayon de géorepérage),
/// SYN (dérive d'horloge).
#[async_trait::async_trait]
pub trait ResolveurConfiguration: Send + Sync {
    async fn resoudre(
        &self,
        cible: &Cible,
        cle: &str,
    ) -> Result<Option<ValeurResolue>, ErreurConfiguration>;

    /// Toutes les valeurs applicables à la cible, **en une descente**.
    ///
    /// Une clé sans valeur à aucun niveau est **absente de la carte**, jamais présente avec une
    /// valeur nulle : `null` serait indistinguable d'une valeur nulle légitimement posée.
    async fn resoudre_tout(
        &self,
        cible: &Cible,
    ) -> Result<BTreeMap<String, ValeurResolue>, ErreurConfiguration>;
}

// =================================================================================================
//  5. RepertoirePointsDeVente
// =================================================================================================

/// Une table d'un point de vente.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TablePdv {
    pub id: Uuid,
    pub libelle: String,
}

/// Un point de vente, **avec ses tables**.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PointDeVente {
    pub id: Uuid,
    pub etablissement_id: Uuid,
    pub module_code: String,
    pub nom: String,
    /// Rattachement de caisse — **sans clé étrangère en base** : `socle/caisse` est un autre
    /// module (principe II). Le cycle CAI ajoutera la vérification par trait.
    pub caisse_id: Option<Uuid>,
    /// **Vide ⇒ comptoir.** Ce n'est pas un cas dégradé, c'est la forme normale d'un maquis.
    pub tables: Vec<TablePdv>,
}

/// Lecture des points de vente d'un établissement.
///
/// **Aucune méthode `est_comptoir`.** `tables.is_empty()` dit la même chose sans qu'une seconde
/// source puisse la contredire. Une méthode dédiée finirait par lire un drapeau, et un drapeau
/// finit par mentir.
///
/// **Consommateurs** : `verticales/restauration`, `verticales/bar`, `verticales/pressing` (cycle
/// PDV), `socle/caisse` pour le rattachement.
#[async_trait::async_trait]
pub trait RepertoirePointsDeVente: Send + Sync {
    async fn points_de_vente(
        &self,
        etablissement_id: Uuid,
    ) -> Result<Vec<PointDeVente>, ErreurRegistre>;

    async fn point_de_vente(&self, id: Uuid) -> Result<Option<PointDeVente>, ErreurRegistre>;
}

// =================================================================================================
//  6. ObstacleDesactivation — le trait dont le sens est inversé
// =================================================================================================

/// Une raison de refuser la désactivation d'un service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Obstacle {
    pub module_code: String,
    /// **Clé i18n — jamais une phrase.** Une chaîne ici traverserait l'API jusqu'à l'écran sans
    /// passer par le catalogue de traductions, donc sans anglais et sans relecture de vocabulaire.
    pub motif_cle: String,
    /// « 3 séjours en cours ». Le nombre est séparé du motif pour que la phrase se compose dans la
    /// langue de l'utilisateur, où le pluriel ne s'accorde pas partout de la même façon.
    pub nombre: u32,
}

/// **Le seul trait implémenté ailleurs qu'ici.**
///
/// FR-016 exige qu'un service portant des opérations en cours ne puisse pas être désactivé — un
/// séjour ouvert, une addition non réglée. Or cette information vit dans les **verticales**, et un
/// crate du socle ne peut pas en dépendre (porte P-03).
///
/// # Inversion de dépendance
///
/// Le trait est **défini** dans `socle/etablissements`, **implémenté** par chaque verticale, et
/// **injecté** à l'assemblage — dans `backend/api/`, seul endroit du produit qui a le droit de
/// connaître tout le monde.
///
/// # À ce cycle, la liste est vide, et ce n'est pas un trou
///
/// Aucune verticale ne crée encore d'opération : la désactivation est donc libre, et c'est exact.
/// Ce que le cycle livre est le **point d'accrochage**, posé maintenant pour la même raison
/// qu'`EstablishmentDirectory` l'a été à vide au cycle 001 — quand la question se posera au cycle
/// SEJ, l'alternative existera déjà. **Une alternative qui existe se prend ; une alternative à
/// construire se contourne.**
///
/// Un test enregistre un obstacle factice et constate que la désactivation est refusée en le
/// nommant (`backend/tests/desactivation_bloquee.rs`). Sans lui, un point d'accrochage jamais
/// exercé peut être cassé par un remaniement sans que rien ne le signale.
#[async_trait::async_trait]
pub trait ObstacleDesactivation: Send + Sync {
    /// Qu'est-ce qui empêche de désactiver ce service, **à cet instant** ?
    async fn obstacles(
        &self,
        etablissement_id: Uuid,
        module_code: &str,
    ) -> Result<Vec<Obstacle>, ErreurRegistre>;
}
