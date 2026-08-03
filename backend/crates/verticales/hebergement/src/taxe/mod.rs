//! `taxe` — le **constat** figé au départ (SEJ-04).
//!
//! # ★ Ce module ne calcule AUCUN montant, et c'est sa raison d'être
//!
//! Il enregistre des **faits** — nuits constatées, personnes, période — et un **paramétrage
//! recopié** — assujettissement, règle de conversion, classement, commune. Il ne les interprète
//! jamais.
//!
//! **Compter les nuits d'un intervalle est de l'arithmétique. Décider lesquelles sont assujetties
//! est une règle fiscale**, et elle ne vit que dans `JurisdictionAdapter` (`socle/fiscalite`,
//! porte P-12). `nuitees_assujetties` et `montant_mineur` sont **posées au schéma et jamais
//! alimentées** par ce cycle.
//!
//! Écrit explicitement parce que c'est la confusion la plus tentante du cycle : **le crate qui
//! détient le constat semble être celui qui doit en tirer le montant. Il ne l'est pas** —
//! exactement comme `ParametrageFiscalHebergement` au cycle 004.
//!
//! # Ce qui rend le figeage vérifiable plutôt que promis
//!
//! `GRANT SELECT, INSERT` **seuls** sur `taxe_sejour_constat`. Le rôle applicatif ne peut pas
//! modifier un constat, quelle que soit la ligne de code écrite au-dessus — et il n'y a ni
//! fonction de modification ni fonction de suppression dans [`repository`], parce que l'absence
//! dit la règle avant l'échec.

pub mod repository;

pub use repository::{Constat, ConstatAEcrire, nuits_calendaires};
