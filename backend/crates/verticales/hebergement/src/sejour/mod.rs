//! `sejour` — **le cœur du cycle 006** : l'arrivée, le départ, la prolongation, le changement de
//! chambre.
//!
//! Terme utilisateur : **« Séjour »** / *Stay* (lexique v1.6.0). Un passage de deux heures est un
//! séjour autant que trois nuits.
//!
//! # La promesse du cycle 004 se vérifie ici
//!
//! `MoteurDisponibilite::attribuer(&mut tx, …)` a été écrit **pour ce moment**, et sa
//! documentation le dit mot pour mot : *« c'est ce qui rendra possible au check-in de SEJ-02
//! d'attribuer l'unité et d'ouvrir la note dans une seule transaction »*. Ce module est la
//! première vérification que cette promesse tient.
//!
//! # Le cœur n'est pas une table, c'est un budget
//!
//! Le cadrage §5.6 fait de la rapidité du passage une **condition d'existence** du produit :
//! *« le module de passage doit être irréprochable en rapidité (moins de 30 secondes) sinon il
//! sera contourné »*. La traduction en architecture est **une seule transaction, un seul appel
//! réseau bloquant** — cinq écritures groupées plutôt que cinq allers-retours.
//!
//! # ⚠️ Deux règles que ce module ne contourne jamais
//!
//! - **Tenter l'insertion et traduire la violation** — jamais lire d'abord pour décider. Une
//!   lecture préalable rendrait la double attribution *improbable* au lieu d'*impossible*.
//! - **Toute durée vient de `now()` de la base**, jamais de l'horloge d'un terminal (porte P-23).
//!   `horodatage_client` est écrit sur `accompagnant`, et c'est permis : **écrire la colonne n'est
//!   pas s'appuyer dessus**.

pub mod modele;
pub mod repository;
pub mod service;

pub use modele::{
    Accompagnant, FichePolice, IssueAccompagnant, LigneNote, NoteVue, NouvelAccompagnant,
    OuvrirSejour, Sejour, SejourOuvert, SejourVue, StatutSejour,
};
pub use service::{
    AGREGAT_SEJOUR, ServiceSejour, TYPE_ACCOMPAGNANT_AJOUTE, TYPE_FICHE_POLICE_GENEREE,
    TYPE_SEJOUR_OUVERT,
};
