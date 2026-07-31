//! Types de la configuration héritée — ETB-04.
//!
//! Terme utilisateur : une valeur héritée s'affiche **« Vaut pour tous vos établissements »**, une
//! valeur surchargée **« Modifié ici »** (`docs/design/lexique.md`). Les mots « héritage »,
//! « surcharge », « portée » et « override » n'atteignent jamais l'interface.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::Portee;

/// Une valeur résolue, telle que l'API la rend.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ValeurVue {
    pub cle: String,
    pub valeur: serde_json::Value,
    /// **Obligatoire.** `TENANT` | `ETABLISSEMENT` | `MODULE` | `POINT_DE_VENTE`. C'est ce qui
    /// permet à l'écran de distinguer « vaut pour tous vos établissements » de « modifié ici ».
    pub origine: String,
}

/// Une entrée du catalogue, telle que l'API la rend.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EntreeCatalogue {
    pub cle: String,
    pub type_valeur: String,
    pub portee_la_plus_basse: String,
    pub story: String,
    pub libelle_cle: String,
    pub description_cle: String,
}

/// Demande d'écriture d'une valeur.
#[derive(Debug, Clone)]
pub struct EcrireParametre {
    pub id: Uuid,
    pub cle: String,
    pub valeur: serde_json::Value,
    pub portee: Portee,
    /// Identifiant du niveau visé. `None` pour la portée `TENANT`, qui n'en a pas.
    pub portee_id: Option<Uuid>,
}

/// Erreur du domaine de la configuration.
#[derive(Debug, thiserror::Error)]
pub enum ErreurParametre {
    /// La clé n'est pas au catalogue. La clé étrangère l'impose déjà en base ; ce contrôle-ci
    /// donne le message.
    #[error("clé « {0} » absente du catalogue des paramètres")]
    CleHorsCatalogue(String),

    /// La portée demandée est **plus basse** que la `portee_la_plus_basse` déclarée au catalogue.
    #[error("portée « {portee} » interdite pour la clé « {cle} » (plus basse autorisée : {plus_basse})")]
    PorteeInterdite {
        cle: String,
        portee: String,
        plus_basse: String,
    },

    /// **Extension de la porte P-10 au `JSONB`.** Un paramètre de type `MONTANT_MINEUR` dont la
    /// valeur n'est pas un entier ferait entrer un montant en flottant par la porte de service —
    /// exactement ce que P-10 interdit sur les colonnes.
    #[error("valeur de type « {type_attendu} » attendue pour la clé « {cle} », reçu : {recu}")]
    TypeIncompatible {
        cle: String,
        type_attendu: String,
        recu: String,
    },

    #[error("la portée « {0} » exige un identifiant de niveau")]
    PorteeIdManquant(String),

    #[error("niveau visé inconnu ou hors du tenant courant")]
    NiveauInconnu,

    #[error("erreur de base : {0}")]
    Base(#[from] sqlx::Error),

    #[error("écriture au grand livre : {0}")]
    Outbox(#[from] kaya_synchronisation::ErreurOutbox),

    #[error("contexte de tenant : {0}")]
    ContexteTenant(#[from] crate::tenant_context::ErreurContexteTenant),
}

impl ErreurParametre {
    pub fn code(&self) -> &'static str {
        match self {
            ErreurParametre::CleHorsCatalogue(_) => "cle_hors_catalogue",
            ErreurParametre::PorteeInterdite { .. } => "portee_interdite",
            ErreurParametre::TypeIncompatible { .. } => "type_incompatible",
            ErreurParametre::PorteeIdManquant(_) => "portee_id_manquant",
            ErreurParametre::NiveauInconnu => "niveau_inconnu",
            _ => "erreur_interne",
        }
    }

    pub fn valeur(&self) -> Option<String> {
        match self {
            ErreurParametre::CleHorsCatalogue(v) | ErreurParametre::PorteeIdManquant(v) => {
                Some(v.clone())
            }
            ErreurParametre::PorteeInterdite { portee, .. } => Some(portee.clone()),
            ErreurParametre::TypeIncompatible { recu, .. } => Some(recu.clone()),
            _ => None,
        }
    }
}

/// Les types de valeur du catalogue, et **ce qu'ils acceptent en `JSONB`**.
///
/// # Pourquoi cette validation existe — extension de la porte P-10
///
/// P-10 impose « montants en entiers d'unité mineure, quantités en `NUMERIC` » et l'analyse des
/// migrations la vérifie **sur les colonnes**. Or `parametre_configuration.valeur` est un `JSONB` :
/// un barème de nuitée écrit `1500.75` y entrerait sans qu'aucune colonne ne soit en cause, et le
/// premier calcul fiscal produirait un montant à virgule dans une devise à zéro décimale.
///
/// La validation est donc portée par le **type déclaré au catalogue**, seul endroit qui sache ce
/// que la clé signifie.
pub fn valeur_compatible(type_valeur: &str, valeur: &serde_json::Value) -> bool {
    use serde_json::Value;
    match type_valeur {
        // **Entier strict.** `is_i64()` refuse `1500.75` et `1500.0` — le second est un flottant
        // qui vaut un entier, et l'accepter laisserait entrer la représentation dont on ne veut
        // pas. Un montant d'unité mineure s'écrit sans point décimal, toujours.
        "ENTIER" | "DUREE_MINUTES" | "MONTANT_MINEUR" => valeur.is_i64(),
        "TEXTE" | "HEURE_LOCALE" => valeur.is_string(),
        "BOOLEEN" => valeur.is_boolean(),
        // Un barème est une structure — objet ou tableau. Sa forme interne relève du cycle qui le
        // définit ; ce qui est refusé ici, c'est un scalaire déguisé en barème.
        "BAREME" => matches!(valeur, Value::Object(_) | Value::Array(_)),
        // Type inconnu : le `CHECK` du catalogue l'aurait déjà refusé. Rendre `false` plutôt que
        // `true` fait échouer l'écriture au lieu de la laisser passer sans contrôle.
        _ => false,
    }
}
