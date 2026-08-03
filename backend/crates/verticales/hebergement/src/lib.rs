//! `verticales/hebergement` — unités louables, formules de location, moteur de disponibilité.
//!
//! **La première verticale non vide du produit.** Le crate existait depuis le cycle 001 comme
//! coquille, et son en-tête disait pourquoi : « c'est ce qui rend la porte **P-03** capable de
//! constater dès aujourd'hui qu'aucune arête interdite n'existe ». Ce cycle lui donne son
//! contenu — donc, pour la première fois, une cible réelle à cette porte.
//!
//! # Ce que ce crate garantit, et par quel moyen
//!
//! > **Deux clients ne peuvent jamais recevoir la même unité au même moment.**
//!
//! La garantie n'est pas dans ce code. Elle est dans une ligne de la migration `0025` :
//!
//! ```sql
//! CONSTRAINT occupation_sans_chevauchement
//!     EXCLUDE USING gist (unite_id WITH =, periode WITH &&)
//! ```
//!
//! Le service **tente l'insertion et traduit la violation** ; il ne lit jamais d'abord pour
//! décider. Une lecture préalable serait exactement le verrou applicatif que le principe IV
//! refuse : elle rendrait la double attribution improbable au lieu d'impossible, et se dégraderait
//! sous charge sans rien signaler.
//!
//! # La frontière du principe V, écrite ici parce que c'est la confusion la plus tentante
//!
//! `formule` porte `assujettie_taxe_nuitee` et `regle_conversion_taxe`. Ce sont des **paramètres**
//! fiscaux, et ce crate ne les interprète **jamais** : il les stocke et les expose par le trait
//! [`traits::ParametrageFiscalHebergement`], qui rend un paramétrage, jamais un montant. Toute
//! règle fiscale vit dans `JurisdictionAdapter` (`socle/fiscalite`), et la porte **P-12** fait
//! échouer le build sur une règle trouvée ailleurs.
//!
//! Le crate qui détient le paramètre semble être celui qui doit l'appliquer. Il ne l'est pas.
//!
//! # Les cinq modules
//!
//! | Module | Ce qu'il porte |
//! |---|---|
//! | [`erreurs`] | La traduction de la violation d'exclusion, **écrite une seule fois** |
//! | [`referentiel`] | Catégories, unités, formules, barèmes, plages — **classe C** |
//! | [`occupation`] | L'attribution et la libération d'une unité — **classe B** |
//! | [`tarification`] | Le montant d'un passage et sa rebascule. **Calcule, ne facture pas** |
//! | [`traits`] | Ce que le crate expose à SEJ-02, SEJ-03 et FIS-03 |
//! | [`sejour`] | ★ **Le cœur du cycle 006** — arrivée, départ, prolongation, changement de chambre |
//! | [`note`] | La note du séjour et ses lignes — **sous-ensemble hébergement seul** |
//! | [`police`] | La fiche de police et sa numérotation continue **par établissement** |

#![forbid(unsafe_code)]

pub mod erreurs;
pub mod note;
pub mod occupation;
pub mod police;
pub mod sejour;
pub mod referentiel;
pub mod tarification;
pub mod traits;

/// Nom du module d'activité que ce crate sert.
///
/// Écrit **une fois** : chaque endpoint du cycle vérifie que le module est actif dans
/// l'établissement, et le recopier produirait, le jour où quelqu'un écrirait `Hebergement`, un
/// contrôle qui ne trouve jamais rien — donc un refus permanent, ou pire, une absence de refus.
pub const MODULE_HEBERGEMENT: &str = "HEBERGEMENT";

/// Résultat d'une écriture idempotente — **la distinction que le contrat HTTP transforme en
/// `201` ou `200`**.
///
/// Reprend l'intention de `kaya_etablissements::Issue`, mais **redéclarée ici** : importer un type
/// du socle pour l'exposer dans les signatures d'une verticale donnerait au socle une raison de
/// connaître la verticale le jour où l'on voudrait la réciproque. Deux énumérations à deux
/// variantes coûtent moins que cette arête.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Issue {
    /// La ligne vient d'être créée — `201`, et **l'événement outbox est émis**.
    Creee,
    /// La ligne existait déjà : rejeu d'un terminal qui vide sa file — `200`, et **aucun
    /// événement**. Le grand livre porte les transitions d'état, pas les tentatives réseau.
    DejaPresente,
}
