//! Types de l'établissement — ETB-01.
//!
//! Terme utilisateur : **« Votre établissement »** (`docs/design/lexique.md`). Le mot « tenant »
//! n'existe pas pour l'utilisateur, et « établissement » n'est jamais qualifié : le lexique pose
//! que l'utilisateur est toujours dans le sien.

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::Classement;

/// Un établissement tel que l'API le rend.
///
/// # Pourquoi ce type existe à côté de [`crate::Etablissement`]
///
/// [`crate::Etablissement`] est le type **du domaine**, celui que les autres crates lisent par
/// `EstablishmentDirectory`. Il porte `Classement` en type somme, qui ne se sérialise pas
/// directement en JSON sans imposer une forme au contrat HTTP.
///
/// Ce type-ci est la **vue transportée** : `classement` et `etoiles` y sont deux champs, comme
/// dans le corps de requête et comme en base. La conversion se fait ici, en un seul endroit —
/// le reste du code ne manipule que le type somme.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EtablissementVue {
    pub id: Uuid,
    pub nom: String,
    /// Sélectionne le `JurisdictionAdapter`. **N'encode aucune règle fiscale** (principe V).
    pub juridiction: String,
    /// `ETOILES` | `NON_CLASSE` | `RESIDENCE_MEUBLEE`. Vocabulaire fiscal officiel, conservé tel
    /// quel à l'écran (lexique, règle 2).
    pub classement: String,
    /// Renseigné **si et seulement si** `classement = "ETOILES"`.
    pub etoiles: Option<u8>,
    pub commune: String,
    /// **Le fuseau appartient à l'établissement, pas au serveur** (principe IV).
    pub fuseau_horaire: String,
    /// ISO 4217.
    pub devise: String,
    pub adresse: Option<String>,
    /// Numéro de compte contribuable — vocabulaire fiscal officiel (lexique, règle 2).
    pub ncc: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub cree_le: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub modifie_le: OffsetDateTime,
}

impl EtablissementVue {
    pub fn classement_type(&self) -> Option<Classement> {
        Classement::depuis_colonnes(&self.classement, self.etoiles.map(i16::from))
    }
}

/// Demande de création.
///
/// `tenant_id` n'y figure pas : il vient du contexte d'authentification, jamais du corps de la
/// requête. Le laisser fournir par l'appelant offrirait à un client la possibilité d'écrire chez
/// un autre — que la politique `WITH CHECK` refuserait, mais une défense en profondeur commence
/// par ne pas poser la question.
#[derive(Debug, Clone)]
pub struct CreerEtablissement {
    /// UUID v7 **généré par le client** — c'est lui qui rend le rejeu inoffensif. Un double-clic
    /// sur « Créer » ne doit pas créer deux établissements.
    pub id: Uuid,
    pub nom: String,
    pub juridiction: String,
    pub classement: Classement,
    pub commune: String,
    pub fuseau_horaire: String,
    pub devise: String,
    pub adresse: Option<String>,
    pub ncc: Option<String>,
}

/// Demande de modification — **tout champ absent est laissé tel quel**.
///
/// # Pourquoi `Option` partout, et pourquoi ce n'est pas ambigu ici
///
/// Aucun des champs modifiables n'est « effaçable » : on ne repasse pas une commune à vide, on ne
/// retire pas une devise. `None` signifie donc « non touché », sans que la distinction avec
/// « mettre à `null` » ait à se poser. Le jour où un champ deviendra effaçable, il faudra un
/// `Option<Option<T>>` — et ce commentaire sera l'endroit où l'on comprendra pourquoi.
#[derive(Debug, Clone, Default)]
pub struct ModifierEtablissement {
    pub nom: Option<String>,
    pub classement: Option<Classement>,
    pub commune: Option<String>,
    pub fuseau_horaire: Option<String>,
    pub devise: Option<String>,
    pub adresse: Option<String>,
    pub ncc: Option<String>,
}

/// Ce qui a changé lors d'une modification — **charge utile de l'événement**.
///
/// Le journal doit porter les valeurs **avant et après** (`data-model.md` § Événements) : un
/// lecteur qui n'a que l'événement doit pouvoir dire ce qui s'est passé sans consulter la table.
#[derive(Debug, Clone, Default)]
pub struct Changements {
    pub champs: Vec<Changement>,
}

#[derive(Debug, Clone)]
pub struct Changement {
    pub champ: &'static str,
    pub avant: serde_json::Value,
    pub apres: serde_json::Value,
}

impl Changements {
    pub fn ajouter(
        &mut self,
        champ: &'static str,
        avant: impl Into<serde_json::Value>,
        apres: impl Into<serde_json::Value>,
    ) {
        self.champs.push(Changement {
            champ,
            avant: avant.into(),
            apres: apres.into(),
        });
    }

    pub fn contient(&self, champ: &str) -> bool {
        self.champs.iter().any(|c| c.champ == champ)
    }

    pub fn est_vide(&self) -> bool {
        self.champs.is_empty()
    }

    /// Forme JSON de la charge utile : `{ champ: { avant, apres } }`.
    pub fn en_json(&self) -> serde_json::Value {
        let mut carte = serde_json::Map::new();
        for changement in &self.champs {
            carte.insert(
                changement.champ.to_owned(),
                serde_json::json!({ "avant": changement.avant, "apres": changement.apres }),
            );
        }
        serde_json::Value::Object(carte)
    }
}

/// Erreur du domaine de l'établissement.
///
/// Chaque variante porte **le code stable** que le contrat HTTP expose, et la valeur refusée
/// quand il y en a une (FR-032) : l'interface branche sa clé i18n sur le code, et compose un
/// message qui nomme la chose.
#[derive(Debug, thiserror::Error)]
pub enum ErreurEtablissement {
    #[error("nom vide ou trop long : entre 1 et 200 caractères après nettoyage")]
    NomInvalide,

    #[error("commune vide : elle est l'assiette du reversement communal")]
    CommuneInvalide,

    #[error("fuseau horaire inconnu : « {0} »")]
    FuseauInconnu(String),

    #[error("code devise invalide : « {0} » — trois lettres ISO 4217 attendues")]
    DeviseInvalide(String),

    /// `classement_incoherent` — un nombre d'étoiles sans classement par étoiles, ou l'inverse.
    #[error("classement incohérent : « {0} »")]
    ClassementIncoherent(String),

    /// `devise_figee` — **posé à vide à ce cycle.** La fonction qui compte les opérations
    /// financières rend zéro tant qu'aucune n'existe ; le cycle CAI la branche.
    #[error("la devise ne se modifie plus après la première opération financière")]
    DeviseFigee,

    #[error("numéro de compte contribuable vide")]
    NccInvalide,

    #[error("établissement inconnu ou hors du tenant courant")]
    Inconnu,

    #[error("erreur de base : {0}")]
    Base(#[from] sqlx::Error),

    #[error("écriture au grand livre : {0}")]
    Outbox(#[from] kaya_synchronisation::ErreurOutbox),

    #[error("contexte de tenant : {0}")]
    ContexteTenant(#[from] crate::tenant_context::ErreurContexteTenant),
}

impl ErreurEtablissement {
    /// Code stable exposé par le contrat HTTP — **jamais traduit**, jamais affiché tel quel.
    pub fn code(&self) -> &'static str {
        match self {
            ErreurEtablissement::NomInvalide => "nom_invalide",
            ErreurEtablissement::CommuneInvalide => "commune_invalide",
            ErreurEtablissement::FuseauInconnu(_) => "fuseau_inconnu",
            ErreurEtablissement::DeviseInvalide(_) => "devise_invalide",
            ErreurEtablissement::ClassementIncoherent(_) => "classement_incoherent",
            ErreurEtablissement::DeviseFigee => "devise_figee",
            ErreurEtablissement::NccInvalide => "ncc_invalide",
            ErreurEtablissement::Inconnu => "etablissement_inconnu",
            _ => "erreur_interne",
        }
    }

    /// La valeur refusée, quand il y en a une à nommer.
    pub fn valeur(&self) -> Option<String> {
        match self {
            ErreurEtablissement::FuseauInconnu(v)
            | ErreurEtablissement::DeviseInvalide(v)
            | ErreurEtablissement::ClassementIncoherent(v) => Some(v.clone()),
            _ => None,
        }
    }
}
