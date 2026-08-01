//! `personne` — **l'identité civile, et rien d'autre** (CPT-00).
//!
//! Terme utilisateur : **« Personne »** / *Person* (`docs/design/lexique.md`).
//!
//! # Ce que ce sous-module ne porte pas, et pourquoi c'est le sujet
//!
//! Aucun élément d'authentification — ils vivent sur `compte`. Aucun élément de contrat de
//! travail — ils vivent sur `employe`, qui est une **provision** sans le moindre privilège pour
//! `kaya_app`. `backend/tests/personne_compte_employe.rs` refuse qu'une colonne de salaire, de
//! date d'embauche ou de numéro CNPS apparaisse ici (FR-004).
//!
//! # `type_piece` et `numero_piece` n'existent pas dans ce modèle
//!
//! Les **colonnes** existent (migration `0015`), le **type Rust** ne les porte pas. C'est la forme
//! la plus forte de la décision : leur alimentation relève de SEJ-01 et leur rétention de 90 jours
//! de TRX-06. Un champ posé dans la structure serait rempli par le premier handler qui en a
//! l'occasion, et le produit constituerait un fichier d'identités sans durée de conservation.
//!
//! # Trois couches, exactement celles du module doré
//!
//! | Couche | Fichier | Ce qu'elle décide |
//! |---|---|---|
//! | Modèle | [`modele`] | Ce qui circule, et ce qui ne circule pas |
//! | Repository | [`repository`] | Les requêtes, littérales, transaction en paramètre |
//! | Service | [`service`] | La transaction, et l'événement **dedans** |

pub mod modele;
pub mod repository;
pub mod service;

pub use modele::{CreerPersonne, ErreurPersonne, ModifierPersonne, Personne};
pub use service::{
    AGREGAT_PERSONNE, ServicePersonne, TYPE_PERSONNE_CREEE, TYPE_PERSONNE_MODIFIEE,
};
