//! Authentification — hachage, politique de mot de passe, indiscernabilité des échecs.
//!
//! **Ce sous-module n'a pas de `repository.rs`, et c'est délibéré** : il n'écrit dans aucune
//! table. Il fait du calcul — hacher, comparer, refuser — sur des données que d'autres couches
//! lisent et écrivent. Lui donner un repository par symétrie avec `personne/` ou `roles/`
//! obligerait à inventer une table pour la remplir.

pub mod argon2;
pub mod mots_de_passe_compromis;
pub mod politique;
pub mod service;

pub use argon2::{ErreurHachage, Verification, condensat_factice, hacher, prechauffer, verifier};
pub use mots_de_passe_compromis::est_compromis;
pub use politique::{RefusMotDePasse, verifier as verifier_politique};
pub use service::{ServiceAuthentification, SessionOuverte};
