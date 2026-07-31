//! `domain` — types, règles et validation **partagés** entre l'API, le nœud de site et la
//! coquille Tauri.
//!
//! La raison d'être de ce crate tient en une phrase du principe II : **une seule implémentation
//! du calcul de la taxe de nuitée**, pas trois. Tout ce qui doit donner le même résultat sur le
//! serveur, sur un nœud de site et sur un terminal vit ici.
//!
//! **Ce que ce crate ne contient pas au cycle 001** : aucune règle fiscale. Les types de la
//! fiscalité y sont déclarés parce que `JurisdictionAdapter` en a besoin dans sa signature, mais
//! ils restent **minimaux et sans logique** (principe X). La porte **P-12** vérifie qu'aucun
//! crate hors `socle/fiscalite` ne les référence.

#![forbid(unsafe_code)]

pub mod fiscal;
pub mod monnaie;
pub mod temps;
