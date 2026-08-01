//! `roles` — **le cumul, et l'union sans priorité** (CPT-02).
//!
//! Terme utilisateur : **« Ce que chacun peut faire »**. Les mots « rôle » et « permission »
//! n'atteignent jamais l'interface (`docs/design/lexique.md`).
//!
//! # Le cœur du module tient en une phrase
//!
//! Un compte porte N rôles ; ses droits sont l'**union** de leurs permissions, sans priorité ni
//! rôle principal (FR-017). Adjoua est gérante, caissière et réceptionniste sur le même
//! établissement : elle se connecte une fois et voit tout ce que les trois ouvrent.
//!
//! La faute symétrique — un « rôle principal » dont les permissions primeraient — n'est pas
//! interdite par une consigne mais par un **type** : [`traits::AccessController`] rend un
//! `BTreeSet<String>` et **n'accepte ni ne rend jamais un rôle**. Un consommateur qui voudrait
//! brancher sur un rôle n'a rien à quoi se brancher.
//!
//! `service.rs` (attribution, retrait) arrive avec **T039**.

pub mod modele;
pub mod repository;

pub use modele::{AttribuerRole, EntreeReferentielRole, ErreurRoles, PorteeRole};
