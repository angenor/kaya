//! Horodatages.
//!
//! **Le point qui décide de la justesse de tout calcul de durée, de taxe et de clôture** : il
//! existe deux horodatages, et les confondre est l'erreur que le principe IV interdit.
//!
//! - L'**horodatage d'autorité serveur** fait foi. Toute règle métier s'appuie sur lui, sans
//!   exception.
//! - L'**horodatage client** est indicatif : ordre d'affichage local. Jamais une règle.
//!
//! Les deux sont des `OffsetDateTime` et rien dans le type ne les distingue — c'est le nom de la
//! colonne et la discipline du patron (`docs/module-dore.md`) qui les tiennent séparés. Les
//! réunir en un seul champ « pour simplifier » est exactement la faute décrite au cadrage §11.4.

use time::OffsetDateTime;

/// Horodatage posé par le **serveur**, seul à faire autorité.
pub type HorodatageAutorite = OffsetDateTime;

/// Horodatage rapporté par un **terminal**. Indicatif, jamais opposable.
pub type HorodatageClient = OffsetDateTime;
