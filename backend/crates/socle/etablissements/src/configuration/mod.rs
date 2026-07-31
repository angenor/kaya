//! **ETB-04** — la chaîne d'héritage de configuration.
//!
//! Quatre niveaux : tenant → établissement → service → point de vente. Le plus spécifique gagne,
//! et **une seule descente** suffit à le trouver.
//!
//! C'est le composant que huit cycles liront. Écrit au cycle HEB, il aurait été teinté
//! d'hébergement ; écrit ici, avant son premier consommateur, il ne suppose rien de ce qu'on
//! configurera.

pub mod modele;
pub mod repository;
pub mod service;

pub use modele::{EcrireParametre, EntreeCatalogue, ErreurParametre, ValeurVue, valeur_compatible};
pub use service::{PgResolveurConfiguration, ServiceConfiguration, portee_depuis_code};
