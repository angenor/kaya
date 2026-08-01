//! `compte` — **l'identité d'authentification** (CPT-01).
//!
//! Terme utilisateur : **« Compte »** / *Account*.
//!
//! Un compte n'est ni une personne, ni un employé (CPT-00). Il porte de quoi se connecter, et
//! **rien** de la vie civile ni du contrat de travail — `backend/tests/personne_compte_employe.rs`
//! le vérifie sur `information_schema`.
//!
//! Le sous-module n'a **pas** de `service.rs` à ce stade : les écritures de compte (création,
//! changement d'état, changement de mot de passe) arrivent avec T041, et l'authentification vit
//! dans `authentification/service.rs` parce qu'elle orchestre le hachage, les sessions et le
//! registre des actions — trois choses qu'un service de compte n'a pas à connaître.

pub mod modele;
pub mod repository;

pub use modele::{
    CompteAuthentification, CompteVue, CreerCompte, ErreurCompte, RolePorte,
};
