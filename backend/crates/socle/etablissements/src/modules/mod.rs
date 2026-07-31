//! **ETB-02 / ETB-02b** — activation des services et déclaration de ce qu'ils consomment.
//!
//! Terme utilisateur : **« Vos services »**. « Module d'activité » est le nom technique et
//! n'apparaît jamais à l'écran ; « capacité » **n'apparaît nulle part**, seule la capacité
//! concrète est nommée (`docs/design/lexique.md`).
//!
//! Le sous-module porte aussi les implémentations de [`crate::RegistreModules`] et de
//! [`crate::RegistreCapacites`] — les deux traits par lesquels chaque verticale demande, au
//! démarrage d'une opération, si son service est rendu ici.

pub mod modele;
pub mod registres;
pub mod repository;
pub mod service;

pub use modele::{
    BasculerService, CapaciteDuService, DeclarerCapacite, ErreurModules, ServiceActif,
};
pub use registres::{PgRegistreCapacites, PgRegistreModules};
pub use service::{IssueBascule, ServiceModules};
