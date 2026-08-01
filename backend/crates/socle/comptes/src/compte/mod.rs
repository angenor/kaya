//! `compte` — **l'identité d'authentification** (CPT-01).
//!
//! Terme utilisateur : **« Compte »** / *Account*.
//!
//! Un compte n'est ni une personne, ni un employé (CPT-00). Il porte de quoi se connecter, et
//! **rien** de la vie civile ni du contrat de travail — `backend/tests/personne_compte_employe.rs`
//! le vérifie sur `information_schema`.
//!
//! `service.rs` porte les écritures — création, changement d'état, changement de mot de passe.
//! **L'authentification n'y est pas** : elle vit dans `authentification/service.rs`, parce
//! qu'elle orchestre le hachage, les sessions et le registre des actions — trois choses qu'un
//! service de compte n'a pas à connaître.

pub mod modele;
pub mod repository;
pub mod service;

pub use modele::{
    CLE_LONGUEUR_MIN, CompteAuthentification, CompteVue, CreerCompte, ErreurCompte, RolePorte,
};
pub use service::{
    ServiceComptes, TYPE_COMPTE_CREE, TYPE_COMPTE_DESACTIVE, TYPE_COMPTE_MOT_DE_PASSE_CHANGE,
    TYPE_COMPTE_REACTIVE,
};
