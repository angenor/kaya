//! Types du séjour.
//!
//! # Ce qui circule vers l'écran, et ce qui n'en sort jamais
//!
//! [`SejourVue`] porte le nom du client, **résolu par le trait `AnnuaireClients`** — jamais par
//! une jointure. Il ne porte **aucun numéro de pièce** : `ClientResume` n'en a pas, et c'est ce
//! qui empêche la donnée sensible de traverser du socle vers la verticale.

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use utoipa::ToSchema;
use uuid::Uuid;

/// L'état d'un séjour. **Deux valeurs, et pas une de plus.**
///
/// Un séjour est en cours ou terminé. Les états intermédiaires qu'on serait tenté d'ajouter —
/// « en attente », « à confirmer » — appartiennent à la **réservation** (RSV, tranche T4), qui a
/// son propre cycle de vie `provisoire → confirmee → honoree | annulee | no_show`. Les mélanger
/// rendrait impossible de dire, devant une chambre occupée, si quelqu'un y dort.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum StatutSejour {
    EnCours,
    Clos,
}

impl StatutSejour {
    pub fn code(self) -> &'static str {
        match self {
            StatutSejour::EnCours => "en_cours",
            StatutSejour::Clos => "clos",
        }
    }

    /// Reconstruit le statut depuis le code lu en base.
    ///
    /// Rend `None` sur un code inconnu plutôt que de paniquer : une ligne écrite par une version
    /// ultérieure du produit — cas réel en mode auto-hébergé, où les binaires ne sont pas tous à
    /// jour au même instant — ne doit pas faire tomber la lecture de la liste entière.
    pub fn depuis_code(code: &str) -> Option<Self> {
        match code {
            "en_cours" => Some(StatutSejour::EnCours),
            "clos" => Some(StatutSejour::Clos),
            _ => None,
        }
    }
}

/// Un séjour tel qu'il est en base.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Sejour {
    pub id: Uuid,
    pub etablissement_id: Uuid,
    /// **Absent pour un passage** : la pièce d'identité vient après la clé (FR-023).
    pub client_id: Option<Uuid>,
    pub statut: StatutSejour,
    /// Horodatage d'**autorité serveur**. C'est lui que le calcul de durée réelle lit au départ.
    #[serde(with = "time::serde::rfc3339")]
    pub ouvert_le: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339::option")]
    pub clos_le: Option<OffsetDateTime>,
}

/// Un accompagnant — **classe A**.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Accompagnant {
    pub id: Uuid,
    pub sejour_id: Uuid,
    pub nom: String,
    pub prenoms: Option<String>,
    /// ⚠️ **Le numéro de pièce n'est PAS rendu par ce type.** La colonne existe (migration
    /// `0031`) ; ce champ dit seulement qu'une pièce est enregistrée, comme `ClientResume` le fait
    /// pour le titulaire. Le rendre exposerait une seconde surface de fuite pour la donnée que
    /// FR-012 protège.
    pub piece_enregistree: bool,
    #[serde(with = "time::serde::rfc3339::option")]
    pub retire_le: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339")]
    pub cree_le: OffsetDateTime,
}

/// La fiche de police d'un séjour.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FichePolice {
    pub id: Uuid,
    pub sejour_id: Uuid,
    /// Continu **par établissement**, sans trou.
    pub numero: i64,
    /// **FR-047** — une fiche sans identité rattachée est identifiée comme telle. Terme
    /// utilisateur : « Identité à compléter », jamais « incomplète ».
    pub complete: bool,
    #[serde(with = "time::serde::rfc3339")]
    pub generee_le: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339::option")]
    pub completee_le: Option<OffsetDateTime>,
}

/// Une ligne de la note.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LigneNote {
    pub id: Uuid,
    pub nature: String,
    /// Renseigné **seulement** sur un ajustement.
    pub motif: Option<String>,
    /// **Clé i18n, jamais un libellé rendu** : la note s'affiche en `fr` et en `en` (P-16).
    pub libelle_cle: String,
    /// ⚠️ **`NUMERIC` en base**, rendu en chaîne décimale : un `f64` perdrait des chiffres sur une
    /// quantité au prorata, et le principe V l'interdit jusque dans le contrat.
    pub quantite: String,
    /// **Entier d'unité mineure** (P-10).
    pub prix_unitaire_mineur: i64,
    /// **Entier d'unité mineure. Peut être négatif** — un départ anticipé rembourse.
    pub montant_mineur: i64,
    pub devise: String,
    #[serde(with = "time::serde::rfc3339::option")]
    pub periode_debut: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub periode_fin: Option<OffsetDateTime>,
}

/// La note d'un séjour, **avec son total calculé**.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NoteVue {
    pub id: Uuid,
    pub sejour_id: Uuid,
    pub statut: String,
    pub devise: String,
    pub lignes: Vec<LigneNote>,
    /// ★ **La somme des lignes, calculée à la lecture — jamais une colonne.**
    ///
    /// Une colonne totalisatrice se désynchronise en silence, et le silence est exactement ce que
    /// le propriétaire achète en installant ce logiciel (cadrage §8.3).
    pub total_mineur: i64,
    #[serde(with = "time::serde::rfc3339::option")]
    pub arretee_le: Option<OffsetDateTime>,
}

/// Ce que l'ouverture d'un séjour rend — **tout ce que l'écran doit afficher, en un appel**.
///
/// C'est ce qui tient le budget de FR-031 : au plus **un** appel réseau bloquant entre le premier
/// geste et la confirmation. Rendre le séjour seul obligerait l'écran à trois appels de plus pour
/// afficher « C'est fait » avec l'heure de fin.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SejourOuvert {
    pub sejour: Sejour,
    pub occupation: crate::occupation::OccupationVue,
    pub note: NoteVue,
    pub fiche_police: FichePolice,
    /// Horodatage d'**autorité serveur** — il dit **quand** la réponse était vraie, et c'est lui
    /// que l'écran affiche, jamais l'horloge du terminal.
    #[serde(with = "time::serde::rfc3339")]
    pub instant_autorite: OffsetDateTime,
}

/// Un séjour dans la liste — **avec le nom de son client**, résolu par `AnnuaireClients`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SejourVue {
    pub sejour: Sejour,
    /// ⚠️ **Résolu par le trait, jamais par jointure** (P-04). `None` quand le séjour n'a pas de
    /// client rattaché — ou quand sa fiche a été purgée par TRX-06, deux cas que l'écran présente
    /// de la même façon : sans nom.
    pub client_nom: Option<String>,
    pub client_telephone: Option<String>,
    /// Nombre de personnes — **dérivé** du titulaire et des accompagnants non retirés, jamais
    /// saisi en double (FR-018).
    pub nombre_personnes: i32,
    pub unite_id: Option<Uuid>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub fin_prevue: Option<OffsetDateTime>,
    /// **Entier d'unité mineure** — la somme des lignes.
    pub total_mineur: i64,
    pub devise: String,
}

/// Demande d'ouverture d'un séjour.
#[derive(Debug, Clone)]
pub struct OuvrirSejour {
    /// UUID v7 **généré par le client** (FR-086) — le serveur déduplique, il n'engendre pas.
    pub id: Uuid,
    pub etablissement_id: Uuid,
    pub unite_id: Uuid,
    pub formule_id: Uuid,
    pub debut_client: OffsetDateTime,
    pub fin_client: OffsetDateTime,
    /// **Absent pour un passage.**
    pub client_id: Option<Uuid>,
    /// Ajoutés dans la **même** transaction que le séjour : un accompagnant déclaré à l'arrivée
    /// et perdu par un second appel manqué ferait une fiche de police fausse.
    pub accompagnants: Vec<NouvelAccompagnant>,
    pub horodatage_client: Option<OffsetDateTime>,
}

/// Un accompagnant à ajouter — **un nom suffit** (FR-015).
#[derive(Debug, Clone)]
pub struct NouvelAccompagnant {
    pub id: Uuid,
    pub nom: String,
    pub prenoms: Option<String>,
    pub date_naissance: Option<time::Date>,
    pub nationalite: Option<String>,
    pub type_piece: Option<String>,
    pub numero_piece: Option<String>,
    pub horodatage_client: Option<OffsetDateTime>,
}

/// Ce qu'un ajout d'accompagnant produit.
///
/// ★ **Trois issues, et la troisième est celle du principe VI.** Un ajout sur un séjour **clos**
/// n'est ni accepté (`201` serait un ajout d'office) ni rejeté (`409` serait un rejet silencieux) :
/// il part en **file de réconciliation** avec son motif.
#[derive(Debug, Clone)]
pub enum IssueAccompagnant {
    /// Ajouté à un séjour ouvert — `201`.
    Ajoute(Accompagnant),
    /// Rejeu du même identifiant — `200`, **et aucun second événement outbox**.
    Rejeu(Accompagnant),
    /// ★ Le séjour est **clos** — `202`, avec l'identifiant de la ligne de réconciliation.
    Orphelin { reconciliation_id: Uuid },
}
