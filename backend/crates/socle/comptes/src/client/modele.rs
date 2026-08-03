//! Types de la fiche client.
//!
//! # Ce qui circule, et ce qui ne circule jamais
//!
//! [`FicheClient`] porte le numéro de pièce d'identité ; [`ClientResume`] **ne le porte pas**.
//! Ce n'est pas une commodité de sérialisation : le résumé traverse vers `verticales/hebergement`
//! par le trait `AnnuaireClients`, et laisser le numéro traverser multiplierait les endroits où la
//! rétention de 90 jours de TRX-06 devra le purger. Le résumé porte
//! [`ClientResume::piece_enregistree`] — ce dont la fiche de police a besoin **sans lire la
//! pièce**.

use serde::{Deserialize, Serialize};
use time::{Date, OffsetDateTime};
use utoipa::ToSchema;
use uuid::Uuid;

/// La fiche complète d'un client — identité civile **et** qualification.
///
/// ⚠️ **`numero_piece` est en clair dans ce type et chiffré en base.** Le repository déchiffre à
/// la lecture et **journalise la consultation** au registre des actions (FR-012). Toute
/// construction de ce type depuis la base passe donc par un chemin qui trace ; c'est ce qui rend
/// le journal d'accès exhaustif sans discipline d'appelant.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FicheClient {
    /// L'identifiant **est** celui de la personne : une personne est cliente ou ne l'est pas.
    pub id: Uuid,
    pub nom: String,
    pub prenoms: Option<String>,
    /// E.164. **Aucune contrainte de format national** : l'indicatif par défaut est un paramètre
    /// d'établissement (`indicatif_telephonique_defaut`), jamais une règle de code.
    pub telephone: Option<String>,
    pub email: Option<String>,

    /// Les deux attributs que CPT n'a aucune raison de connaître — ils vivent sur `client`.
    pub date_naissance: Option<Date>,
    pub nationalite: Option<String>,

    pub type_piece: Option<String>,
    /// **Déchiffré à la lecture, et cette lecture est journalisée.** Absent quand aucune pièce
    /// n'est enregistrée — jamais une chaîne vide, qui laisserait croire à une pièce sans numéro.
    pub numero_piece: Option<String>,
    /// Instant de **capture** de la pièce (FR-013) — ce sur quoi la rétention de TRX-06
    /// s'appuiera, sans migration.
    #[serde(with = "time::serde::rfc3339::option")]
    pub piece_capturee_le: Option<OffsetDateTime>,

    /// Indicatif — ordre d'affichage local. **Jamais un critère de calcul** (porte P-23).
    #[serde(with = "time::serde::rfc3339::option")]
    pub horodatage_client: Option<OffsetDateTime>,
    /// Horodatage d'**autorité serveur**.
    #[serde(with = "time::serde::rfc3339")]
    pub cree_le: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub modifie_le: OffsetDateTime,
}

/// Ce qu'une liste de résultats montre — **et rien de plus**.
///
/// ⚠️ **Aucun numéro de pièce d'identité.** Voir le commentaire de tête. `piece_enregistree` dit
/// qu'une pièce est là ; il ne dit pas laquelle.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ClientResume {
    pub id: Uuid,
    pub nom: String,
    pub prenoms: Option<String>,
    pub telephone: Option<String>,
    /// Vrai quand une pièce est enregistrée — ce que la fiche de police doit savoir **sans lire
    /// la pièce elle-même** (FR-047).
    pub piece_enregistree: bool,
}

/// Le résultat d'une recherche.
///
/// `tronque` dit qu'il y avait **plus** de résultats que la limite. Une liste silencieusement
/// coupée est un mensonge sur un écran de comptoir : Yao conclurait que la fiche n'existe pas et
/// en créerait une seconde.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ResultatRecherche {
    pub clients: Vec<ClientResume>,
    pub tronque: bool,
}

/// Une préférence, telle qu'elle est en base.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Preference {
    pub id: Uuid,
    pub personne_id: Uuid,
    pub texte: String,
    #[serde(with = "time::serde::rfc3339::option")]
    pub horodatage_client: Option<OffsetDateTime>,
    /// **AUTORITÉ** — c'est lui qui ordonne, donc lui qui décide laquelle est courante.
    #[serde(with = "time::serde::rfc3339")]
    pub cree_le: OffsetDateTime,
}

/// Demande de création d'une fiche.
///
/// `tenant_id` n'y figure pas : il vient du contexte d'authentification, jamais de l'appelant.
#[derive(Debug, Clone)]
pub struct CreerClient {
    /// UUID v7 **généré côté client** (FR-086) — c'est lui, et non une clé engendrée côté
    /// serveur, qui rend le rejeu inoffensif.
    pub id: Uuid,
    pub nom: String,
    pub prenoms: Option<String>,
    pub telephone: Option<String>,
    pub email: Option<String>,
    pub date_naissance: Option<Date>,
    pub nationalite: Option<String>,
    pub type_piece: Option<String>,
    pub numero_piece: Option<String>,
    pub horodatage_client: Option<OffsetDateTime>,
}

/// Demande de modification.
///
/// **Remplacement complet, pas de fusion champ par champ** — même régime que `ModifierPersonne`.
/// Une fusion rendrait impossible d'effacer un numéro de téléphone : l'absence du champ et sa mise
/// à `null` seraient indistinguables.
#[derive(Debug, Clone)]
pub struct ModifierClient {
    pub nom: String,
    pub prenoms: Option<String>,
    pub telephone: Option<String>,
    pub email: Option<String>,
    pub date_naissance: Option<Date>,
    pub nationalite: Option<String>,
    pub type_piece: Option<String>,
    pub numero_piece: Option<String>,
    pub horodatage_client: Option<OffsetDateTime>,
}

/// La forme de recherche que le serveur **déduit** de la saisie.
///
/// L'opérateur ne choisit pas un mode : au comptoir, il tape ce qu'il a. La déduction est faite
/// une fois, ici, et [`FormeRecherche::Ambigue`] interroge **les trois** et fusionne plutôt que de
/// deviner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormeRecherche {
    /// Préfixe sur `nom_repli`.
    Nom,
    /// Suffixe sur `telephone_repli` — le numéro se retrouve avec ou sans indicatif.
    Telephone,
    /// Égalité sur `numero_piece_repli`.
    Piece,
    /// Les trois, fusionnées.
    Ambigue,
}

/// Longueur maximale du nom — **alignée sur le `CHECK` de la migration `0015`**.
pub const NOM_MAX: usize = 200;

/// Longueur maximale du texte d'une préférence — **alignée sur le `CHECK` de `0029`**.
pub const PREFERENCE_MAX: usize = 2000;

/// Échec du service de la fiche client.
///
/// Chaque variante porte un **code stable** que l'interface traduit par le lexique
/// (`docs/design/lexique.md`) — jamais le message de diagnostic, qui nomme des tables et parle
/// anglais technique.
#[derive(Debug, thiserror::Error)]
pub enum ErreurClient {
    #[error("nom invalide : entre 1 et {NOM_MAX} caractères après nettoyage")]
    NomVide,

    #[error("téléphone invalide")]
    TelephoneInvalide,

    #[error("nationalité invalide : entre 2 et 80 caractères")]
    NationaliteInvalide,

    #[error("texte de préférence invalide : entre 1 et {PREFERENCE_MAX} caractères")]
    PreferenceInvalide,

    #[error("client inconnu")]
    Inconnu,

    #[error("accès aux données : {0}")]
    Base(#[from] sqlx::Error),

    #[error("contexte de tenant : {0}")]
    ContexteTenant(#[from] kaya_etablissements::tenant_context::ErreurContexteTenant),

    #[error("grand livre : {0}")]
    Outbox(#[from] kaya_synchronisation::ErreurOutbox),

    #[error("registre des actions : {0}")]
    Audit(String),

    #[error("protection de la pièce d'identité : {0}")]
    Coffre(String),
}

impl ErreurClient {
    /// Le **code stable** rendu dans `CorpsErreur`, sur lequel l'interface branche sa clé i18n.
    ///
    /// Il ne change jamais, même si le message change : c'est le contrat avec l'écran.
    pub fn code(&self) -> &'static str {
        match self {
            ErreurClient::NomVide => "nom_vide",
            ErreurClient::TelephoneInvalide => "telephone_invalide",
            ErreurClient::NationaliteInvalide => "nationalite_invalide",
            ErreurClient::PreferenceInvalide => "preference_invalide",
            ErreurClient::Inconnu => "client_inconnu",
            ErreurClient::Base(_)
            | ErreurClient::ContexteTenant(_)
            | ErreurClient::Outbox(_)
            | ErreurClient::Audit(_)
            | ErreurClient::Coffre(_) => "erreur_interne",
        }
    }
}

/// Déduit la forme de recherche de la saisie.
///
/// **Les deux seuils de cette fonction sont des constantes nommées, jamais des littéraux dans la
/// requête** — ce ne sont pas des paramètres d'établissement (aucune story du périmètre ne dit
/// « paramétrable », principe I·c), mais ils décident du comportement au comptoir : anonymes,
/// leur révision serait introuvable.
pub fn deduire_forme(saisie: &str) -> FormeRecherche {
    let nettoye = saisie.trim();
    if nettoye.is_empty() {
        return FormeRecherche::Nom;
    }

    let que_des_chiffres = nettoye
        .chars()
        .all(|c| c.is_ascii_digit() || c == '+' || c == ' ' || c == '-' || c == '(' || c == ')');
    let chiffres = nettoye.chars().filter(char::is_ascii_digit).count();

    if que_des_chiffres && chiffres >= SUFFIXE_TELEPHONE_MIN {
        return FormeRecherche::Telephone;
    }

    // Alphanumérique sans espace, assez long, **et portant au moins un chiffre** : un numéro de
    // pièce. La condition sur le chiffre n'est pas décorative — sans elle, « Ouattara » serait
    // pris pour un numéro de pièce et la recherche par nom la plus banale tomberait dans la
    // mauvaise branche.
    let sans_espace = !nettoye.contains(char::is_whitespace);
    let alphanumerique = nettoye.chars().all(|c| c.is_ascii_alphanumeric() || c == '-');
    if sans_espace && alphanumerique && nettoye.len() > PIECE_LONGUEUR_MIN && chiffres > 0 {
        // Une saisie qui pourrait être un nom **et** un numéro reste ambiguë : mieux vaut trois
        // requêtes qu'un résultat vide inexplicable.
        return if nettoye.chars().all(|c| c.is_ascii_digit()) {
            FormeRecherche::Ambigue
        } else {
            FormeRecherche::Piece
        };
    }

    FormeRecherche::Nom
}

/// Longueur minimale du **suffixe** téléphonique interrogé.
///
/// Six chiffres, parce que c'est ce qu'un client dicte de tête quand il ne se souvient pas de son
/// indicatif, et parce qu'en dessous le suffixe cesse d'être discriminant : sur dix mille fiches,
/// quatre chiffres finaux rendraient une liste inexploitable.
///
/// **Ce n'est pas un paramètre d'établissement**, et c'est délibéré (principe I·c) : aucune story
/// du périmètre ne dit « paramétrable », et l'ouvrir au paramétrage créerait une clé de
/// configuration que personne ne saurait régler.
pub const SUFFIXE_TELEPHONE_MIN: usize = 6;

/// Longueur au-delà de laquelle une saisie alphanumérique compacte **portant un chiffre** est
/// prise pour un numéro de pièce.
///
/// Cinq, parce que les numéros de pièce ivoiriens en font davantage, et parce qu'en dessous on
/// entrerait dans la longueur des noms courts — « Yao », « Koffi ». Même régime que la constante
/// ci-dessus : nommée pour être révisable, non paramétrable pour n'être pas réglée à l'aveugle.
pub const PIECE_LONGUEUR_MIN: usize = 5;

/// Nombre de résultats rendus par défaut quand l'appelant n'en demande pas.
///
/// Vingt, parce qu'un écran de comptoir n'en affiche pas plus sans défiler, et que la troncature
/// est **dite** (`tronque`) plutôt que subie. Une limite plus haute ferait payer à chaque frappe
/// des lignes que personne ne lit.
pub const LIMITE_DEFAUT: i64 = 20;

/// Plafond de la limite demandable — au-delà, la requête est ramenée à cette valeur.
///
/// Sans plafond, un appelant pourrait demander dix mille fiches à chaque frappe et transformer la
/// recherche instantanée en export.
pub const LIMITE_MAX: i64 = 100;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn un_nom_reste_un_nom() {
        assert_eq!(deduire_forme("Kouamé"), FormeRecherche::Nom);
        assert_eq!(deduire_forme("Yao"), FormeRecherche::Nom);
        assert_eq!(deduire_forme("Marie Claire"), FormeRecherche::Nom);
        // Le cas qui ferait tomber une déduction naïve dans la branche « pièce ».
        assert_eq!(deduire_forme("Ouattara"), FormeRecherche::Nom);
    }

    #[test]
    fn un_numero_de_telephone_est_reconnu_avec_ou_sans_indicatif() {
        assert_eq!(deduire_forme("0707123456"), FormeRecherche::Telephone);
        assert_eq!(deduire_forme("+225 07 07 12 34 56"), FormeRecherche::Telephone);
        assert_eq!(deduire_forme("07-07-12-34-56"), FormeRecherche::Telephone);
    }

    #[test]
    fn un_numero_de_piece_alphanumerique_est_reconnu() {
        assert_eq!(deduire_forme("CI00123456"), FormeRecherche::Piece);
        assert_eq!(deduire_forme("A1234567"), FormeRecherche::Piece);
    }

    /// **Une saisie trop courte pour être un téléphone reste un nom**, pas un téléphone tronqué.
    #[test]
    fn une_saisie_numerique_trop_courte_reste_un_nom() {
        assert_eq!(deduire_forme("123"), FormeRecherche::Nom);
        assert_eq!(deduire_forme("12345"), FormeRecherche::Nom);
    }

    #[test]
    fn une_saisie_vide_ne_fait_pas_paniquer_la_deduction() {
        assert_eq!(deduire_forme(""), FormeRecherche::Nom);
        assert_eq!(deduire_forme("   "), FormeRecherche::Nom);
    }
}
