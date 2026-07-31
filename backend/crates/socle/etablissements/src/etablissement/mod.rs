//! **ETB-01** — l'identité de l'établissement.
//!
//! Trois couches, exactement la forme de `note/` (`docs/module-dore.md`) : [`modele`],
//! [`repository`], [`service`].
//!
//! Le sous-module porte aussi l'implémentation de [`crate::EstablishmentDirectory`], le trait par
//! lequel les autres crates lisent un établissement — **jamais par jointure inter-schémas**.

pub mod modele;
pub mod repertoire;
pub mod repository;
pub mod service;

pub use modele::{
    Changements, CreerEtablissement, ErreurEtablissement, EtablissementVue, ModifierEtablissement,
};
pub use repertoire::PgEstablishmentDirectory;
pub use service::{IssueModification, ServiceEtablissement, classement_depuis_requete};
