//! L'attribution d'une unité sur un intervalle — **classe B, le cœur du cycle**.
//!
//! > **Deux clients ne peuvent jamais recevoir la même unité au même moment.**
//!
//! La garantie n'est pas dans ce module : elle est dans la contrainte `EXCLUDE USING gist` de la
//! migration `0025`. Ce module la **traduit** en refus métier, et calcule ce que le client ne doit
//! pas pouvoir influencer — la borne haute de la période d'indisponibilité.

pub mod modele;
pub mod repository;
pub mod service;

pub use modele::{
    DemandeAttribution, ErreurAttribution, OccupationVue, StatutOccupation, UniteDisponible,
};
pub use service::{
    AGREGAT_OCCUPATION, ServiceOccupation, TYPE_OCCUPATION_ATTRIBUEE, TYPE_OCCUPATION_LIBEREE,
    VERSION_SCHEMA_OCCUPATION, est_active,
};
