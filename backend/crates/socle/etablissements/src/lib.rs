//! `socle/etablissements` — tenants, établissements, modules d'activité et configuration héritée.
//!
//! **L'entité centrale du produit est l'établissement, pas l'hôtel** (constitution, préambule).
//! Ce crate ne suppose ni hébergement, ni point de vente : un maquis seul, un pressing seul et
//! une résidence meublée seule sont des établissements valides. `backend/tests/agnosticite_socle.rs`
//! en fait la preuve permanente, sur trois parcours et huit étapes chacun.
//!
//! C'est aussi ici que vit la pose du tenant courant — le chemin de code le plus sensible du
//! produit, celui qui décide quelles lignes un client voit.
//!
//! # Un sous-module par story, trois couches chacun
//!
//! | Sous-module | Story | Ce qu'il porte |
//! |---|---|---|
//! | [`etablissement`] | ETB-01 | Identité : juridiction, classement, commune, NCC |
//! | [`modules`] | ETB-02, ETB-02b | Activation des services, déclaration de capacité |
//! | [`points_de_vente`] | ETB-03 | Points de vente et tables — un comptoir est un PDV sans table |
//! | [`configuration`] | ETB-04 | La chaîne d'héritage à quatre niveaux |
//! | [`branding`] | ETB-05 | Identité visuelle, surcharge partielle par champ |
//! | [`note`] | TRX-01 | **Le module doré** — le patron que les cinq ci-dessus recopient |
//!
//! Chacun a ses trois couches `modele` / `repository` / `service`, exactement la forme de
//! `note/`. Un fichier unique de deux mille lignes serait plus court à écrire et impossible à
//! relire.
//!
//! # Le mot « service » a deux sens ici, et ils ne se mélangent pas
//!
//! - **couche applicative** — `service.rs`, entre le repository et le handler ;
//! - **module d'activité vu par l'utilisateur** — « Vos services » (`docs/design/lexique.md`).
//!
//! Le premier sens ne vaut que dans les chemins de code, le second que dans les libellés
//! d'interface. Un fichier nommé `services/service.rs` signalerait que la distinction a été
//! perdue.

#![forbid(unsafe_code)]

pub mod branding;
pub mod configuration;
pub mod etablissement;
pub mod modules;
pub mod note;
pub mod points_de_vente;
pub mod tenant_context;
pub mod traits;

pub use traits::{
    Cible, ErreurConfiguration, ErreurRegistre, EstablishmentDirectory, CapaciteDeclaree, Obstacle,
    ObstacleDesactivation, PointDeVente, Portee, RegistreCapacites, RegistreModules,
    RepertoirePointsDeVente, ResolveurConfiguration, TablePdv, ValeurResolue,
};

use uuid::Uuid;

/// Résultat d'une écriture idempotente — **la distinction que le contrat HTTP transforme en `201`
/// ou `200`**.
///
/// Un rejeu n'est pas une erreur : c'est le comportement normal d'un terminal qui vide sa file
/// après une coupure. Répondre `409` obligerait chaque appelant à traiter comme un échec une
/// écriture que le serveur a déjà acceptée (principe VI).
///
/// # Pourquoi ce type vit ici et non dans chaque sous-module
///
/// Le module doré l'avait défini dans `note/modele.rs`, seul endroit qui en avait besoin au cycle
/// 001. Cinq sous-modules le partagent désormais, et cinq énumérations identiques à deux variantes
/// laisseraient un lecteur se demander laquelle employer. `note::Issue` reste accessible sous son
/// ancien nom — le patron n'est pas altéré, seulement remonté d'un cran.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Issue {
    /// La ligne n'existait pas — `201 Created`.
    Creee,
    /// La ligne existait déjà — `200 OK`, corps = la ligne **telle qu'elle est en base**.
    DejaPresente,
}

/// Classement de l'établissement — **il décide du barème de la taxe communale de nuitée**
/// (cadrage §9.6).
///
/// # Un type somme, pas une paire `(texte, Option<u8>)`
///
/// Le nombre d'étoiles n'existe que pour une seule variante, et la base l'impose déjà par une
/// égalité de conditions — `CHECK ((classement = 'ETOILES') = (etoiles IS NOT NULL))`. Deux
/// représentations de la même règle, l'une en base et l'autre dans le type : c'est voulu. **La
/// première protège des scripts, la seconde des développeurs.**
///
/// # Aucun plafond sur le nombre d'étoiles, ici comme en base
///
/// Le maximum est fixé par la réglementation nationale, donc par le `JurisdictionAdapter`
/// (principe V, porte P-12). Un `1..=5` codé ici serait une règle de juridiction déguisée en
/// contrainte de type, et le premier pays qui en reconnaît six imposerait de rouvrir le socle.
///
/// Le mot « classement » et ses valeurs restent affichés tels quels : c'est du **vocabulaire
/// fiscal officiel**, que l'exploitant lit sur ses propres papiers (lexique, règle 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Classement {
    Etoiles(u8),
    NonClasse,
    ResidenceMeublee,
}

impl Classement {
    /// Code stocké en base, et rendu par l'API.
    pub fn code(self) -> &'static str {
        match self {
            Classement::Etoiles(_) => "ETOILES",
            Classement::NonClasse => "NON_CLASSE",
            Classement::ResidenceMeublee => "RESIDENCE_MEUBLEE",
        }
    }

    pub fn etoiles(self) -> Option<u8> {
        match self {
            Classement::Etoiles(n) => Some(n),
            _ => None,
        }
    }

    /// Reconstruit le type somme depuis les deux colonnes.
    ///
    /// **Le seul endroit où la paire `(code, étoiles)` existe encore**, et il est étroit : au-delà,
    /// seul le type somme circule. Un `code` inconnu ou une incohérence entre les deux colonnes
    /// produit `None` — la base les refuse déjà, mais une lecture ne doit pas inventer un
    /// classement à partir d'une ligne écrite par un chemin qui ne serait pas passé par elle.
    pub fn depuis_colonnes(code: &str, etoiles: Option<i16>) -> Option<Self> {
        match (code, etoiles) {
            ("ETOILES", Some(n)) if n > 0 => u8::try_from(n).ok().map(Classement::Etoiles),
            ("NON_CLASSE", None) => Some(Classement::NonClasse),
            ("RESIDENCE_MEUBLEE", None) => Some(Classement::ResidenceMeublee),
            _ => None,
        }
    }
}

/// Établissement, tel que les autres modules le lisent.
///
/// Enrichi par ETB-01 : le cycle 001 n'en portait que le nom, le fuseau et la devise.
#[derive(Debug, Clone)]
pub struct Etablissement {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub nom: String,
    /// **Le fuseau appartient à l'établissement, pas au serveur** : une clôture journalière et un
    /// calcul de nuitée se font dans le temps local de l'établissement (principe IV).
    pub fuseau_horaire: String,
    /// ISO 4217. **Figée après la première opération financière** — voir le refus `devise_figee`.
    pub devise: String,
    /// Sélectionne le `JurisdictionAdapter`. **N'encode aucune règle** (principe V).
    pub juridiction: String,
    pub classement: Classement,
    /// Commune de rattachement — assiette du reversement communal.
    pub commune: String,
    pub adresse: Option<String>,
    /// Numéro de compte contribuable. **Sa validité est une règle de juridiction** (porte P-12) :
    /// rien ici ne la vérifie au-delà du « non vide ».
    pub ncc: Option<String>,
}

/// Échec de lecture d'un établissement.
#[derive(Debug, thiserror::Error)]
pub enum ErreurLecture {
    #[error("lecture impossible : {0}")]
    Base(#[from] sqlx::Error),

    /// La ligne porte un classement que le type somme ne sait pas reconstruire. Distincte d'une
    /// erreur de base : elle signale une donnée incohérente, pas une panne.
    #[error("classement illisible en base pour l'établissement {id}")]
    ClassementIllisible { id: Uuid },
}
