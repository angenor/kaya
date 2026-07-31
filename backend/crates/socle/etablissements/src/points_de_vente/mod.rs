//! **ETB-03** — points de vente et tables.
//!
//! **Un comptoir est un point de vente sans aucune table.** Ni drapeau `est_comptoir` en base, ni
//! méthode `est_comptoir` au trait : `tables.is_empty()` dit la même chose sans qu'une seconde
//! source puisse la contredire. Une méthode dédiée finirait par lire un drapeau, et un drapeau
//! finit par mentir.

pub mod modele;
pub mod repertoire;
pub mod repository;
pub mod service;

pub use modele::{
    CreerPointDeVente, ErreurPointDeVente, ModifierPointDeVente, PointDeVenteVue, TableDemandee,
    TableVue,
};
pub use repertoire::PgRepertoirePointsDeVente;
pub use service::ServicePointsDeVente;
