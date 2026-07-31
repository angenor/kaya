//! **Le module doré** — `note_etablissement`, six couches écrites à la main.
//!
//! Ce module n'a aucune importance fonctionnelle : une note interne libre attachée à un
//! établissement. Sa valeur est d'être le **patron** que tous les cycles suivants recopient.
//!
//! Les six couches livrées :
//!
//!   1. **migration** — `backend/migrations/0004_note_etablissement.sql`
//!   2. **registre hors-ligne** — classe A déclarée au §5.1 de `docs/registre-classes-offline.md`
//!   3. **repository** — [`repository`], macros `query!` littérales contre sqlx 0.9
//!   4. **service** — [`service`], transaction unique portant la ligne **et** son événement
//!   5. **handler** — `backend/api/src/routes/notes.rs`, annoté pour utoipa
//!   6. **tests** — `backend/tests/note_etablissement_classe_a.rs`, rejeu et désordre
//!
//! **La septième couche — l'écran — est absente, et c'est une décision.** L'écran de notes
//! n'hérite d'aucun motif : il n'apparaît ni parmi les onze codes maquettés de
//! `docs/design/html/`, ni dans la matrice de dérivation `docs/design/derivation.md`. « Un écran
//! qui n'hérite d'aucun motif ne se code pas » (principe XII). La couche est reportée au cycle
//! ETB, qui dispose d'écrans réellement maquettés. Voir `docs/module-dore.md`.

pub mod modele;
pub mod repository;
pub mod service;

pub use modele::{CreerNote, ErreurNote, Issue, NoteEtablissement};
pub use service::ServiceNote;
