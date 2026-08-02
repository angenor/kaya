//! Types du référentiel de l'offre.
//!
//! # Les trois énumérations sont des types, pas des chaînes
//!
//! `famille`, `regle_conversion_taxe` et `statut_menage` sont stockés en `TEXT` avec un `CHECK`.
//! Ils traversent pourtant le code sous forme de **types sommes** : une chaîne libre finit par
//! porter `"Nuitee"` à un endroit et `"NUITEE"` à un autre, et la comparaison échoue là où
//! personne ne regarde.
//!
//! La conversion depuis la base est **explicite et faillible** ([`FamilleFormule::depuis_code`]) :
//! une valeur inconnue produit un refus nommé, jamais un `unwrap_or` silencieux qui rangerait une
//! demi-journée dans la nuitée.
//!
//! # Ce que ce module ne fait PAS
//!
//! Il ne calcule aucune taxe. [`RegleConversionTaxe`] est un **paramètre** que le crate stocke et
//! expose ; la règle qui le consommera vit dans `JurisdictionAdapter` (`socle/fiscalite`), en T3.
//! La porte P-12 fait échouer le build sur une règle fiscale trouvée ailleurs.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

// =================================================================================================
//  1. Les familles de formule
// =================================================================================================

/// Les **quatre** façons de louer une unité. Toute autre valeur est refusée explicitement
/// (FR-022), jamais ignorée.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FamilleFormule {
    /// La nuit, avec heure d'arrivée et de départ standard.
    Nuitee,
    /// À l'heure, avec un barème de paliers et des heures supplémentaires.
    Passage,
    /// Des plages fixes, non fractionnables.
    DemiJournee,
    /// Au mois — une résidence meublée.
    Mensuel,
}

impl FamilleFormule {
    /// Les quatre, dans l'ordre d'affichage de la maquette `G2`.
    pub const TOUTES: [FamilleFormule; 4] = [
        FamilleFormule::Nuitee,
        FamilleFormule::Passage,
        FamilleFormule::DemiJournee,
        FamilleFormule::Mensuel,
    ];

    /// Le code stocké en base — celui que le `CHECK` de `0024` accepte.
    pub fn code(self) -> &'static str {
        match self {
            FamilleFormule::Nuitee => "NUITEE",
            FamilleFormule::Passage => "PASSAGE",
            FamilleFormule::DemiJournee => "DEMI_JOURNEE",
            FamilleFormule::Mensuel => "MENSUEL",
        }
    }

    /// **Refus explicite d'une famille inconnue** (FR-022, patron du cycle 002).
    ///
    /// Le `unwrap_or(Nuitee)` qu'on écrirait par commodité rangerait une demi-journée dans la
    /// nuitée : le montant serait faux, et rien ne l'indiquerait.
    pub fn depuis_code(code: &str) -> Result<Self, ErreurReferentiel> {
        FamilleFormule::TOUTES
            .into_iter()
            .find(|f| f.code() == code)
            .ok_or_else(|| ErreurReferentiel::FamilleInconnue(code.to_owned()))
    }
}

// =================================================================================================
//  2. La règle de conversion de la taxe — UN PARAMÈTRE, jamais un calcul
// =================================================================================================

/// Comment la taxe de nuitée se compte sur une occupation de plusieurs nuits.
///
/// ⛔ **L'axe « par client » n'est pas résolu.** `UneNuiteeParOccupation` réduit trois nuits à une ;
/// elle ne dit **rien** de trois personnes, alors que la taxe est due « par nuitée **et par
/// client** » (cadrage §9.6) et que les accompagnants comptent (SEJ-02). Le consommateur — FIS-03,
/// en T3 — devra trancher cet axe explicitement, jamais par défaut : un multiplicateur posé à
/// l'aveugle se retrouverait sur des factures et dans un état de reversement communal.
///
/// C'est aussi ce qui rend les deux libellés du lexique employables aujourd'hui : « Une seule taxe
/// pour tout le séjour » et « Une taxe par nuit » ne disent rien des personnes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum RegleConversionTaxe {
    /// La formule est assujettie, mais aucune conversion ne s'applique.
    Aucune,
    /// **Une seule taxe pour tout le séjour** — 500 F pour trois nuits. Pratique attestée à
    /// Deloria.
    UneNuiteeParOccupation,
    /// **Une taxe par nuit** — 500 F × 3.
    AuProrata,
    /// Au-delà d'un seuil de durée, une nuitée est due. Réservé au passage ; **aucun seuil n'est
    /// codé ici** — il viendra du paramétrage (B-02).
    SeuilHoraire,
}

impl RegleConversionTaxe {
    pub const TOUTES: [RegleConversionTaxe; 4] = [
        RegleConversionTaxe::Aucune,
        RegleConversionTaxe::UneNuiteeParOccupation,
        RegleConversionTaxe::AuProrata,
        RegleConversionTaxe::SeuilHoraire,
    ];

    pub fn code(self) -> &'static str {
        match self {
            RegleConversionTaxe::Aucune => "aucune",
            RegleConversionTaxe::UneNuiteeParOccupation => "une_nuitee_par_occupation",
            RegleConversionTaxe::AuProrata => "au_prorata",
            RegleConversionTaxe::SeuilHoraire => "seuil_horaire",
        }
    }

    pub fn depuis_code(code: &str) -> Result<Self, ErreurReferentiel> {
        RegleConversionTaxe::TOUTES
            .into_iter()
            .find(|r| r.code() == code)
            .ok_or_else(|| ErreurReferentiel::RegleConversionInconnue(code.to_owned()))
    }
}

// =================================================================================================
//  3. Le sous-statut de ménage — lu, jamais écrit à ce cycle
// =================================================================================================

/// L'état de propreté d'une unité. **Classe A** — dernier-écrit-gagne, seul cas du produit.
///
/// Aucun endpoint de ce cycle ne l'écrit : c'est HEB-06 (P1, hors périmètre). Le type existe
/// parce que la consultation de disponibilité le rend, et qu'un `String` y laisserait passer
/// n'importe quoi.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum StatutMenage {
    ANettoyer,
    Propre,
    Maintenance,
}

impl StatutMenage {
    pub const TOUS: [StatutMenage; 3] = [
        StatutMenage::ANettoyer,
        StatutMenage::Propre,
        StatutMenage::Maintenance,
    ];

    pub fn code(self) -> &'static str {
        match self {
            StatutMenage::ANettoyer => "a_nettoyer",
            StatutMenage::Propre => "propre",
            StatutMenage::Maintenance => "maintenance",
        }
    }

    pub fn depuis_code(code: &str) -> Result<Self, ErreurReferentiel> {
        StatutMenage::TOUS
            .into_iter()
            .find(|s| s.code() == code)
            .ok_or_else(|| ErreurReferentiel::StatutMenageInconnu(code.to_owned()))
    }
}

// =================================================================================================
//  4. Ce que l'API rend
// =================================================================================================

/// Un battement de remise en état, pour une famille de formule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct TempsRemiseEnEtat {
    pub famille_formule: FamilleFormule,
    /// **Zéro est une valeur, pas une absence** — une salle qu'on n'aère pas entre deux réunions.
    pub duree_minutes: i32,
}

/// Un type de chambre, tel que l'API le rend.
///
/// Terme utilisateur : **« type de chambre »** — jamais « catégorie d'unité », qui colle deux mots
/// techniques dont l'un est déjà écarté du lexique.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CategorieVue {
    pub id: Uuid,
    pub etablissement_id: Uuid,
    pub nom: String,
    pub capacite_accueil: i16,
    /// Les battements déclarés, par famille. Vide tant qu'aucun n'a été réglé — auquel cas le
    /// service applique **zéro**, et le dit.
    pub temps_remise_en_etat: Vec<TempsRemiseEnEtat>,
}

/// Une chambre, un logement, une salle.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UniteVue {
    pub id: Uuid,
    pub categorie_id: Uuid,
    pub code: String,
    pub etage: Option<i16>,
    /// **Classe A, non modifiable à ce cycle** (HEB-06). Rendu parce que la consultation de
    /// disponibilité en a besoin.
    pub statut_menage: StatutMenage,
}

/// Un palier du barème de passage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct PalierVue {
    pub duree_minutes: i32,
    /// **Entier d'unité mineure** (P-10). La devise est portée par la formule, au même niveau.
    pub prix_mineur: i64,
}

/// Une plage fixe de demi-journée.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct PlageVue {
    pub id: Uuid,
    /// Heure **murale locale**, au format `HH:MM`. La conversion en instant se fait au serveur,
    /// avec le fuseau de l'établissement — jamais côté client.
    pub heure_debut: String,
    pub heure_fin: String,
    /// **Clé i18n, jamais une phrase.**
    pub libelle_cle: String,
}

/// Une formule, telle que l'écran `G2` la lit.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FormuleVue {
    pub id: Uuid,
    pub categorie_id: Uuid,
    pub famille: FamilleFormule,
    /// **Entier d'unité mineure** (P-10). Pour `PASSAGE`, c'est le premier palier — « à partir de
    /// 1 500 F l'heure ».
    pub prix_mineur: i64,
    /// ISO 4217, **au même niveau que le montant**, toujours. Lue de l'établissement par
    /// `EstablishmentDirectory`, jamais d'une constante : le produit sert deux devises.
    pub devise: String,
    pub duree_min_minutes: Option<i32>,
    pub duree_max_minutes: Option<i32>,
    /// Format `HH:MM`, heure murale locale.
    pub heure_arrivee_standard: Option<String>,
    pub heure_depart_standard: Option<String>,
    /// 1 à 7 ; **absent = tous les jours**.
    pub jours_autorises: Option<Vec<i16>>,
    pub assujettie_taxe_nuitee: bool,
    /// **Toujours présent dans la réponse**, `null` sur une formule non assujettie — le type
    /// TypeScript généré doit être `string | null`, jamais `string | undefined`.
    ///
    /// La contrainte `formule_regle_fiscale_coherente` rend impossible « assujettie sans règle » :
    /// ce n'est donc jamais un état d'attente, et il n'y a rien à refuser à l'écran.
    pub regle_conversion_taxe: Option<RegleConversionTaxe>,
    pub prix_heure_supplementaire_mineur: Option<i64>,
    /// Les paliers, **triés par durée croissante**. Vide hors `PASSAGE`.
    pub paliers: Vec<PalierVue>,
    /// Les plages, **triées par heure de début**. Vide hors `DEMI_JOURNEE`.
    pub plages: Vec<PlageVue>,
}

// =================================================================================================
//  5. Ce que l'API reçoit
// =================================================================================================

/// Création d'un type de chambre.
///
/// `tenant_id` n'y figure pas : il vient du contexte d'authentification, jamais du corps de la
/// requête. Une défense en profondeur commence par ne pas poser la question.
#[derive(Debug, Clone)]
pub struct CreerCategorie {
    /// UUID v7 **généré par le client** — c'est lui qui rend le rejeu inoffensif.
    pub id: Uuid,
    pub etablissement_id: Uuid,
    pub nom: String,
    pub capacite_accueil: i16,
    pub temps_remise_en_etat: Vec<TempsRemiseEnEtat>,
}

/// Modification d'un type de chambre — **remplacement complet**, jamais un correctif partiel.
///
/// Une modification partielle obligerait chaque champ à distinguer « absent » de « mis à nul », et
/// la distinction se perdrait au premier client qui envoie un objet construit à la main.
#[derive(Debug, Clone)]
pub struct ModifierCategorie {
    pub nom: String,
    pub capacite_accueil: i16,
    pub temps_remise_en_etat: Vec<TempsRemiseEnEtat>,
}

/// Création d'une unité.
#[derive(Debug, Clone)]
pub struct CreerUnite {
    pub id: Uuid,
    pub etablissement_id: Uuid,
    pub categorie_id: Uuid,
    pub code: String,
    pub etage: Option<i16>,
}

/// **Correction d'une unité — deux champs, et pas un de plus.**
///
/// Le registre §7.1 classe littéralement « `unite` — code, étage » en classe C : ces deux champs
/// sont déjà couverts en écriture, création **comme** correction. Les trois autres sont classés
/// ailleurs, et le service les **refuse** explicitement :
///
/// | Champ | Pourquoi il n'est pas ici |
/// |---|---|
/// | `categorie_id` | Change les formules applicables, **donc les tarifs**. Effet fiscal que le registre ne classe nulle part : ça se spécifie, ça ne se glisse pas dans un `PUT` de correction |
/// | `statut_menage` | **Classe A** — HEB-06 |
/// | Mise hors service | **Classe B** — c'est une opération de disponibilité, pas de référentiel |
#[derive(Debug, Clone)]
pub struct ModifierUnite {
    pub code: String,
    pub etage: Option<i16>,
}

/// Création d'une formule, **avec ses enfants dans la même transaction**.
///
/// Les paliers et les plages ne sont pas des ressources séparées : une formule `PASSAGE` sans
/// palier est inexploitable, et la créer en deux appels laisserait un état intermédiaire que
/// l'écran devrait savoir afficher.
#[derive(Debug, Clone)]
pub struct CreerFormule {
    pub id: Uuid,
    pub etablissement_id: Uuid,
    pub categorie_id: Uuid,
    pub famille: FamilleFormule,
    pub prix_mineur: i64,
    pub duree_min_minutes: Option<i32>,
    pub duree_max_minutes: Option<i32>,
    pub heure_arrivee_standard: Option<String>,
    pub heure_depart_standard: Option<String>,
    pub jours_autorises: Option<Vec<i16>>,
    pub assujettie_taxe_nuitee: bool,
    pub regle_conversion_taxe: Option<RegleConversionTaxe>,
    pub prix_heure_supplementaire_mineur: Option<i64>,
    pub paliers: Vec<PalierVue>,
    pub plages: Vec<PlageDemandee>,
}

/// Une plage demandée à la création — sans identifiant, que le serveur pose.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlageDemandee {
    pub heure_debut: String,
    pub heure_fin: String,
    pub libelle_cle: String,
}

/// Modification d'une formule — **c'est là que l'exploitant règle la taxe**.
#[derive(Debug, Clone)]
pub struct ModifierFormule {
    pub prix_mineur: i64,
    pub duree_min_minutes: Option<i32>,
    pub duree_max_minutes: Option<i32>,
    pub heure_arrivee_standard: Option<String>,
    pub heure_depart_standard: Option<String>,
    pub jours_autorises: Option<Vec<i16>>,
    /// Le drapeau que l'exploitant active quand sa commune impose la taxe.
    pub assujettie_taxe_nuitee: bool,
    /// « Une seule taxe pour tout le séjour » ou « Une taxe par nuit » (lexique).
    pub regle_conversion_taxe: Option<RegleConversionTaxe>,
    pub prix_heure_supplementaire_mineur: Option<i64>,
    pub paliers: Vec<PalierVue>,
    pub plages: Vec<PlageDemandee>,
}

// =================================================================================================
//  6. Les refus
// =================================================================================================

/// Échec du référentiel d'hébergement.
///
/// Chaque variante porte le **code** que le contrat HTTP rend et sur lequel l'interface branche sa
/// clé i18n. Le `message` de `thiserror` est un diagnostic pour les journaux — il n'est jamais
/// affiché.
#[derive(Debug, thiserror::Error)]
pub enum ErreurReferentiel {
    /// FR-022 — hors des quatre familles. **Refus explicite, jamais une valeur par défaut.**
    #[error("famille_inconnue: {0}")]
    FamilleInconnue(String),

    #[error("regle_conversion_inconnue: {0}")]
    RegleConversionInconnue(String),

    #[error("statut_menage_inconnu: {0}")]
    StatutMenageInconnu(String),

    #[error("etablissement_inconnu")]
    EtablissementInconnu,

    #[error("categorie_inconnue")]
    CategorieInconnue,

    #[error("unite_inconnue")]
    UniteInconnue,

    #[error("formule_inconnue")]
    FormuleInconnue,

    /// FR-025 — une formule `PASSAGE` sans palier ne sait rien facturer. La base ne peut pas
    /// l'exprimer : la dépendance va de l'enfant au parent.
    #[error("bareme_absent")]
    BaremeAbsent,

    /// FR-033 — une `DEMI_JOURNEE` sans plage n'a rien à vendre.
    #[error("plages_absentes")]
    PlagesAbsentes,

    /// Le refus **nomme ce qui occupe** : « 5 chambres », jamais « suppression impossible ».
    #[error("categorie_occupee: {unites} unité(s)")]
    CategorieOccupee { unites: i64 },

    /// Un corps de correction d'unité portant `categorie_id`, `statut_menage` ou une mise hors
    /// service. **Refusé, jamais ignoré silencieusement.**
    #[error("champ_non_modifiable: {0}")]
    ChampNonModifiable(String),

    /// L'heure murale n'est pas au format `HH:MM`.
    #[error("heure_invalide: {0}")]
    HeureInvalide(String),

    /// Le module `HEBERGEMENT` n'est pas actif dans cet établissement. Patron normalisé au
    /// cycle 002.
    #[error("service_inactif")]
    ServiceInactif,

    #[error("erreur de base : {0}")]
    Base(#[from] sqlx::Error),

    #[error("écriture au grand livre : {0}")]
    Outbox(#[from] kaya_synchronisation::ErreurOutbox),

    #[error("contexte de tenant : {0}")]
    ContexteTenant(#[from] kaya_etablissements::tenant_context::ErreurContexteTenant),

    #[error("lecture de l'établissement : {0}")]
    Annuaire(#[from] kaya_etablissements::ErreurLecture),

    #[error("registre des modules : {0}")]
    Registre(#[from] kaya_etablissements::ErreurRegistre),
}

impl ErreurReferentiel {
    /// Le code stable que le contrat HTTP rend — **c'est lui que l'interface traduit**, jamais le
    /// message.
    pub fn code(&self) -> &'static str {
        match self {
            ErreurReferentiel::FamilleInconnue(_) => "famille_inconnue",
            ErreurReferentiel::RegleConversionInconnue(_) => "regle_conversion_inconnue",
            ErreurReferentiel::StatutMenageInconnu(_) => "statut_menage_inconnu",
            ErreurReferentiel::EtablissementInconnu => "etablissement_inconnu",
            ErreurReferentiel::CategorieInconnue => "categorie_inconnue",
            ErreurReferentiel::UniteInconnue => "unite_inconnue",
            ErreurReferentiel::FormuleInconnue => "formule_inconnue",
            ErreurReferentiel::BaremeAbsent => "bareme_absent",
            ErreurReferentiel::PlagesAbsentes => "plages_absentes",
            ErreurReferentiel::CategorieOccupee { .. } => "categorie_occupee",
            ErreurReferentiel::ChampNonModifiable(_) => "champ_non_modifiable",
            ErreurReferentiel::HeureInvalide(_) => "heure_invalide",
            ErreurReferentiel::ServiceInactif => "service_inactif",
            ErreurReferentiel::Base(_)
            | ErreurReferentiel::Outbox(_)
            | ErreurReferentiel::ContexteTenant(_)
            | ErreurReferentiel::Annuaire(_)
            | ErreurReferentiel::Registre(_) => "erreur_interne",
        }
    }
}

// =================================================================================================
//  7. Heures murales — une conversion, écrite une fois
// =================================================================================================

/// Analyse une heure murale `HH:MM`.
///
/// Écrite ici plutôt que dans chaque appelant : trois endroits la font (formule, plage, seeds), et
/// la troisième copie accepterait `25:00` sans que rien ne le dise.
pub fn heure_depuis_texte(texte: &str) -> Result<time::Time, ErreurReferentiel> {
    let mut morceaux = texte.split(':');
    let heure: u8 = morceaux
        .next()
        .and_then(|h| h.parse().ok())
        .ok_or_else(|| ErreurReferentiel::HeureInvalide(texte.to_owned()))?;
    let minute: u8 = morceaux
        .next()
        .and_then(|m| m.parse().ok())
        .ok_or_else(|| ErreurReferentiel::HeureInvalide(texte.to_owned()))?;
    if morceaux.next().is_some() {
        return Err(ErreurReferentiel::HeureInvalide(texte.to_owned()));
    }
    time::Time::from_hms(heure, minute, 0)
        .map_err(|_| ErreurReferentiel::HeureInvalide(texte.to_owned()))
}

/// Rend une heure murale au format `HH:MM` — celui que le contrat annonce.
pub fn heure_en_texte(heure: time::Time) -> String {
    format!("{:02}:{:02}", heure.hour(), heure.minute())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn les_quatre_familles_font_l_aller_retour() {
        for famille in FamilleFormule::TOUTES {
            assert_eq!(
                FamilleFormule::depuis_code(famille.code()).unwrap(),
                famille
            );
        }
    }

    /// **FR-022 — le refus est explicite.** Un `unwrap_or(Nuitee)` rangerait une demi-journée dans
    /// la nuitée : le montant serait faux, et rien ne l'indiquerait.
    #[test]
    fn une_famille_inconnue_est_refusee_et_nommee() {
        let erreur = FamilleFormule::depuis_code("SEMAINE").unwrap_err();
        assert_eq!(erreur.code(), "famille_inconnue");
        assert!(erreur.to_string().contains("SEMAINE"));
    }

    #[test]
    fn les_quatre_regles_de_conversion_font_l_aller_retour() {
        for regle in RegleConversionTaxe::TOUTES {
            assert_eq!(
                RegleConversionTaxe::depuis_code(regle.code()).unwrap(),
                regle
            );
        }
    }

    /// Les codes stockés en base sont exactement ceux que le `CHECK` de `0024` accepte. Ce test
    /// échoue si quelqu'un renomme une variante sans toucher la migration.
    #[test]
    fn les_codes_sont_ceux_de_la_migration() {
        assert_eq!(FamilleFormule::DemiJournee.code(), "DEMI_JOURNEE");
        assert_eq!(
            RegleConversionTaxe::UneNuiteeParOccupation.code(),
            "une_nuitee_par_occupation"
        );
        assert_eq!(StatutMenage::ANettoyer.code(), "a_nettoyer");
    }

    #[test]
    fn une_heure_murale_fait_l_aller_retour() {
        let heure = heure_depuis_texte("08:30").unwrap();
        assert_eq!(heure_en_texte(heure), "08:30");
        assert_eq!(heure_en_texte(heure_depuis_texte("14:00").unwrap()), "14:00");
    }

    #[test]
    fn une_heure_hors_bornes_est_refusee() {
        for texte in ["25:00", "12:61", "midi", "12", "12:00:00", ""] {
            assert!(
                heure_depuis_texte(texte).is_err(),
                "« {texte} » ne devrait pas être accepté comme heure murale"
            );
        }
    }
}
