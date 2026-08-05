//! **Ce que `hebergement` expose aux autres crates.**
//!
//! La règle qui commande tout : aucune requête ne joint deux schémas de modules (principe II,
//! porte P-04). Les lectures inter-modules passent par un trait — ce fichier dit lesquels.
//!
//! # Trois traits, tous destinés à des consommateurs qui n'existent pas encore
//!
//! Le principe X (« prêt ≠ construit ») commande de justifier chacun : **un trait sans
//! consommateur est une abstraction spéculative**. Les trois justifications sont écrites à leur
//! définition, et elles ne se valent pas — [`MoteurDisponibilite`] a un implémenteur **et** un
//! appelant dès sa création ; les deux autres ont une raison de forme, écrite chez elles.
//!
//! # Pourquoi `#[async_trait::async_trait]`
//!
//! Rust sait écrire `async fn` dans un trait depuis 1.75, mais un tel trait **n'est pas
//! dyn-compatible**. L'injection de dépendances du cadrage §13.2 suppose `Arc<dyn Trait>` :
//! l'annotation est un choix contraint, pas une habitude reprise d'un exemple.

use sqlx::postgres::types::PgRange;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::Issue;
use crate::occupation::{DemandeAttribution, ErreurAttribution, OccupationVue, UniteDisponible};
use crate::referentiel::{ErreurReferentiel, RegleConversionTaxe};

// =================================================================================================
//  1. MoteurDisponibilite — consommé par l'endpoint de ce cycle, puis par SEJ-02
// =================================================================================================

/// La disponibilité et l'attribution d'une unité.
///
/// # `attribuer` PREND la transaction, et c'est toute la raison du trait
///
/// C'est ce qui rendra possible au check-in de SEJ-02 d'attribuer l'unité **et** d'ouvrir la note
/// dans une seule transaction. Un trait qui prendrait un pool obligerait SEJ-02 à deux
/// transactions — donc à une saga avec compensation explicite, pour une opération qui n'en demande
/// pas.
///
/// **Le trait n'est pas spéculatif** : l'endpoint d'attribution de ce cycle en est le premier
/// consommateur, et il a un implémenteur dès sa création.
#[async_trait::async_trait]
pub trait MoteurDisponibilite: Send + Sync {
    /// Les unités attribuables d'une catégorie sur un intervalle.
    ///
    /// **Cette réponse ne garantit rien.** Entre la lecture et l'attribution, une autre
    /// transaction peut prendre l'unité. La garantie est la contrainte d'exclusion, jamais cette
    /// liste (FR-013) — un consommateur qui la traiterait comme une réservation reproduirait le
    /// verrou applicatif que le principe IV refuse.
    async fn unites_disponibles(
        &self,
        etablissement_id: Uuid,
        categorie_id: Uuid,
        periode: PgRange<OffsetDateTime>,
    ) -> Result<Vec<UniteDisponible>, ErreurAttribution>;

    /// Attribue une unité, **dans la transaction fournie**.
    ///
    /// Rend l'[`Issue`] pour que l'appelant distingue une création d'un rejeu — un terminal qui
    /// vide sa file ne doit pas voir d'erreur pour une écriture déjà acceptée.
    async fn attribuer(
        &self,
        tx: &mut sqlx::PgTransaction<'_>,
        demande: DemandeAttribution,
    ) -> Result<(OccupationVue, Issue), ErreurAttribution>;
}

// =================================================================================================
//  2. MoteurTarification — consommé par SEJ-03 (T2) et FIS-03 (T3)
// =================================================================================================

/// La rebascule d'un passage : le palier vendu, et celui qui s'applique en réalité.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct Rebascule {
    pub palier_vendu_minutes: i32,
    /// **Entier d'unité mineure** (P-10).
    pub montant_vendu_mineur: i64,
    /// Ce qui reste dû. **Peut être négatif** — un départ anticipé existe.
    pub difference_mineur: i64,
}

/// Ce que le moteur décide. **Il calcule, il ne facture pas.**
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct DecisionTarification {
    pub duree_reelle_minutes: i64,
    pub formule_appliquee: crate::referentiel::FamilleFormule,
    /// **Absent quand la durée a fait basculer en nuitée** : ce n'est pas un palier majoré, c'est
    /// un changement de formule.
    pub palier_retenu_minutes: Option<i32>,
    pub heures_supplementaires: i32,
    /// **Entier d'unité mineure** (principe V, porte P-10).
    pub montant_du_mineur: i64,
    /// ISO 4217, **au même niveau que le montant**, toujours.
    pub devise: String,
    pub rebascule: Option<Rebascule>,
    /// **Horodatage d'autorité serveur** — jamais l'horloge d'un terminal.
    #[serde(with = "time::serde::rfc3339")]
    pub instant_autorite: OffsetDateTime,
}

/// Le montant dû pour une occupation, à l'instant d'autorité serveur.
///
/// **Calcule, ne facture pas** : aucune ligne de note n'est écrite — la note est SEJ-03, tranche
/// T2. Ce que ce trait produit est une **décision de tarification** que SEJ-03 consommera.
#[async_trait::async_trait]
pub trait MoteurTarification: Send + Sync {
    async fn calculer(&self, occupation_id: Uuid)
    -> Result<DecisionTarification, ErreurAttribution>;
}

// =================================================================================================
//  3. ParametrageFiscalHebergement — LA FRONTIÈRE DU PRINCIPE V
// =================================================================================================

/// Le paramétrage fiscal d'une formule — **jamais un montant**.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParametrageFiscal {
    pub assujettie_taxe_nuitee: bool,
    /// **`None` = formule NON assujettie.** La contrainte `formule_regle_fiscale_coherente` rend
    /// impossible une formule assujettie sans règle : ce n'est donc **jamais un état d'attente**,
    /// et il n'y a rien à refuser.
    ///
    /// ⛔ **L'axe « par client » n'est pas résolu.** `UneNuiteeParOccupation` réduit trois nuits à
    /// une ; elle ne dit **rien** de trois personnes, alors que la taxe est due « par nuitée **et
    /// par client** » (cadrage §9.6) et que les accompagnants comptent (SEJ-02). Le consommateur —
    /// FIS-03 — devra trancher cet axe **explicitement, jamais par défaut** : un multiplicateur
    /// posé à l'aveugle se retrouverait sur des factures et dans un état de reversement communal.
    pub regle_conversion: Option<RegleConversionTaxe>,
}

/// Rend le **paramétrage** fiscal d'une formule.
///
/// > **C'est la frontière du principe V, et elle est ici.**
///
/// Ce trait rend un paramètre, jamais un montant de taxe. Toute règle fiscale vit dans
/// `JurisdictionAdapter` (`socle/fiscalite`), et la porte **P-12** fait échouer le build sur une
/// règle fiscale trouvée ailleurs. `hebergement` stocke `assujettie_taxe_nuitee` et
/// `regle_conversion_taxe` **sans jamais les interpréter**.
///
/// Écrit explicitement parce que c'est la confusion la plus tentante du cycle : le crate qui
/// détient le paramètre semble être celui qui doit l'appliquer. **Il ne l'est pas.**
///
/// # Pourquoi ce trait existe avant son consommateur
///
/// Contrairement à [`MoteurDisponibilite`], il n'a pas d'appelant à ce cycle. Sa raison est de
/// **forme** : sans lui, FIS-03 lirait `hebergement.formule` par une jointure inter-schémas — la
/// voie facile, celle que P-04 attrape, mais après coup. Une alternative qui existe se prend ; une
/// alternative à construire se contourne (précédent d'`EstablishmentDirectory`, posé à vide au
/// cycle 001 pour la même raison).
#[async_trait::async_trait]
pub trait ParametrageFiscalHebergement: Send + Sync {
    async fn parametrage(&self, formule_id: Uuid) -> Result<ParametrageFiscal, ErreurReferentiel>;
}

// =================================================================================================
//  4. LecteurSejour — exposé, consommé par SEJ-03 (T2) et FIS-03 (T3)
// =================================================================================================

/// Ce qu'un séjour est, **pour un consommateur qui n'est pas la verticale hébergement**.
#[derive(Debug, Clone)]
pub struct SejourResume {
    pub id: Uuid,
    pub etablissement_id: Uuid,
    pub client_id: Option<Uuid>,
    pub statut: crate::sejour::StatutSejour,
    pub note_id: Uuid,
    pub devise: String,
    /// **Entier d'unité mineure** (principe V, porte P-10). **Somme des lignes**, jamais une
    /// colonne totalisatrice — une colonne se désynchronise en silence.
    pub total_mineur: i64,
}

/// Le constat de taxe **figé**, tel que `JurisdictionAdapter` le lira en T3.
///
/// ═══════════════════════════════════════════════════════════════════════════════════════════════
///  ★ C'EST LA FRONTIÈRE DU PRINCIPE V, ET ELLE EST ICI
/// ═══════════════════════════════════════════════════════════════════════════════════════════════
///
/// Cette structure porte des **faits** et un **paramétrage recopié**. Elle ne porte **aucun montant
/// de taxe** : `nuitees_assujetties` et `montant_mineur` sont posés au schéma et **jamais
/// alimentés** par ce cycle.
///
/// Décider quelles nuits sont assujetties est une **règle fiscale** — `une_nuitee_par_occupation`
/// réduit trois nuits à une —, et la porte **P-12** fait échouer le build sur une règle fiscale
/// trouvée hors de `JurisdictionAdapter`.
///
/// Écrit explicitement parce que c'est la confusion la plus tentante du cycle : **le crate qui
/// détient le constat semble être celui qui doit en tirer le montant. Il ne l'est pas** —
/// exactement comme [`ParametrageFiscalHebergement`].
///
/// > ⚠️ **Le risque de ce trait est qu'il tente FIS de recalculer.** `constat_taxe` rend un
/// > paramétrage, pas un montant. Le jour où FIS-03 sera écrit, la tentation sera de rappeler la
/// > formule **vivante** plutôt que la copie **figée** — ce qui ferait bouger un séjour clos. Le
/// > constat est la seule source légitime, et son immuabilité est portée par le **privilège**
/// > (`GRANT SELECT, INSERT` seuls), pas par cette phrase.
#[derive(Debug, Clone)]
pub struct ConstatTaxeSejour {
    pub sejour_id: Uuid,
    /// **Arithmétique** : le nombre de nuits calendaires. **Trois pour trois nuits**, jamais un.
    pub nuits_constatees: i32,
    /// **Indicatif** depuis la décision B-10 (2026-08-03) : la taxe est due par nuitée et par
    /// **séjour**, jamais par personne. Documente le séjour, n'entre dans aucun calcul.
    pub nombre_personnes: i32,
    pub assujettie_taxe_nuitee: bool,
    pub regle_conversion_taxe: Option<RegleConversionTaxe>,
    pub classement_etablissement: String,
    pub commune: String,
    /// Horodatage d'**autorité** du figeage.
    pub fige_le: OffsetDateTime,
}

/// Ce que `hebergement` expose du séjour aux **autres crates**.
///
/// # Pourquoi il existe AVANT son consommateur
///
/// Contrairement à [`MoteurDisponibilite`], il n'a **aucun appelant à ce cycle**. Sa raison est de
/// **forme**, et elle est écrite ici plutôt que supposée :
///
/// | Consommateur | Quand | Ce qu'il ferait sans ce trait |
/// |---|---|---|
/// | **SEJ-03** | T2 | Rattacher une consommation de bar à un séjour → `restauration` ou `bar` lirait `hebergement.sejour` |
/// | **FIS-03** | T3 | Lire le constat figé pour produire le montant → `fiscalite` lirait `hebergement.taxe_sejour_constat` |
///
/// Les deux seraient des **jointures inter-schémas** (porte P-04), la seconde sur la donnée la
/// plus sensible du produit.
///
/// C'est le raisonnement de [`ParametrageFiscalHebergement`] au cycle 004, mot pour mot : *une
/// alternative qui existe se prend ; une alternative à construire se contourne*. Deux
/// consommateurs sont **nommés et datés** — ce n'est pas une abstraction à un implémenteur
/// imaginaire.
#[async_trait::async_trait]
pub trait LecteurSejour: Send + Sync {
    async fn resume(
        &self,
        sejour_id: Uuid,
    ) -> Result<Option<SejourResume>, crate::erreurs::ErreurSejour>;

    /// Les séjours **ouverts** d'un établissement — ce dont SEJ-03 aura besoin pour proposer
    /// « porter cette consommation sur une chambre ».
    async fn ouverts(
        &self,
        etablissement_id: Uuid,
    ) -> Result<Vec<SejourResume>, crate::erreurs::ErreurSejour>;

    /// Le constat **figé** d'un séjour clos. `None` si le séjour est encore ouvert.
    async fn constat_taxe(
        &self,
        sejour_id: Uuid,
    ) -> Result<Option<ConstatTaxeSejour>, crate::erreurs::ErreurSejour>;
}
