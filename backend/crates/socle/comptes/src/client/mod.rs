//! `client` — **la fiche client, et la recherche qui la trouve en un souffle** (SEJ-01).
//!
//! Terme utilisateur : **« Client »** / *Guest* (`docs/design/lexique.md` v1.6.0).
//!
//! # La décision de ce module tient en une phrase
//!
//! > **La fiche client est `comptes.personne` QUALIFIÉE par `comptes.client`.**
//!
//! Ce n'est pas une table portant nom, prénoms, téléphone et pièce d'identité. Le réflexe — créer
//! une table complète — produirait un **second fichier d'identités**, avec sa propre durée de
//! conservation à tenir et deux fiches à réconcilier pour une seule personne. La migration `0029`
//! porte le raisonnement complet.
//!
//! **Ce que la table de qualification apporte, et qui n'est pas cosmétique** : `comptes.personne`
//! porte le personnel autant que les clients (CPT-00 — « une femme de ménage a une fiche et aucun
//! compte »). Sans elle, chercher « Kouamé » à la réception ferait apparaître la femme de ménage.
//!
//! # Pourquoi ce module vit dans `socle/comptes` et pas dans une verticale
//!
//! La fiche client **ne dépend d'aucun module d'activité**. Un maquis seul, un bar seul en auront
//! besoin dès SEJ-05, sans hébergement. C'est aussi ce qui rend ses deux permissions
//! transversales (`module_code = NULL`, migration `0030`).
//!
//! **Le sens inverse est interdit et n'a aucun garde-fou naturel** : `socle/comptes` ne lit
//! **jamais** `hebergement.sejour`. L'historique des séjours d'un client
//! (`GET /clients/{id}/sejours`) paraît appartenir au client — il est servi **depuis le crate
//! `hebergement`**. Si `comptes` le lisait, ce serait deux violations d'un coup : jointure
//! inter-schémas (**P-04**) *et* arête `socle/ → verticales/` (**P-03**). Le chemin HTTP cache ce
//! découpage à l'appelant, et c'est normal — le contrat est une façade, pas une carte des crates.
//!
//! # Les quatre fichiers
//!
//! | Fichier | Ce qu'il décide |
//! |---|---|
//! | [`modele`] | Ce qui circule, et **ce qui ne circule pas** — le résumé ne porte aucun numéro de pièce |
//! | [`coffre`] | Le chiffrement au repos du numéro de pièce, **par tenant** (FR-012) |
//! | [`repli`] | La forme cherchable d'un nom, d'un téléphone, d'un numéro de pièce |
//! | [`repository`] | Les requêtes, littérales, transaction en paramètre |
//! | [`service`] | La transaction, l'événement **dedans**, et le journal d'accès à la pièce |

pub mod coffre;
pub mod modele;
pub mod repli;
pub mod repository;
pub mod service;

pub use modele::{
    ClientResume, CreerClient, ErreurClient, FicheClient, FormeRecherche, ModifierClient,
    Preference, ResultatRecherche, deduire_forme,
};
pub use coffre::{CoffreTenant, ErreurCoffre};
pub use repli::{repli, repli_piece, repli_telephone};
pub use service::{
    AGREGAT_CLIENT, AGREGAT_PREFERENCE, ServiceClient, TYPE_CLIENT_CREE, TYPE_CLIENT_MODIFIE,
    TYPE_PREFERENCE_ENREGISTREE,
};
