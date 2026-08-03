//! `kaya-api` — binaire Actix du serveur, du nœud de site et du paquet auto-hébergé.
//!
//! **Un seul binaire, trois configurations** (constitution, § Pile technique imposée). Jamais
//! trois produits : ce qui diverge est un fichier de configuration, pas une base de code.
//!
//! Le crate est aussi une bibliothèque : les tests d'intégration de `backend/tests/` montent
//! **l'application réelle**. Un test qui reconstruirait ses propres routes ne prouverait rien de
//! celles qui sont servies.

#![forbid(unsafe_code)]

pub mod application;
pub mod contexte;
pub mod db;
/// **SYN-04** — le câblage du constat de dérive d'horloge sur le registre des actions.
pub mod derive;
pub mod observabilite;
pub mod openapi;
pub mod routes;
pub mod securite;
pub mod secrets;
pub mod stockage;
