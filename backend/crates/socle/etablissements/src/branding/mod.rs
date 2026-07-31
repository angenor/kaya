//! **ETB-05** — l'identité visuelle.
//!
//! Posée au tenant, **surchargée partiellement** par établissement : toutes les colonnes de
//! contenu sont nullables, et la résolution prend champ par champ la première valeur non nulle.
//! Surcharger le seul logo laisse hériter tout le reste, sans qu'aucune logique de fusion n'ait à
//! être écrite.
//!
//! Le binaire du logo **ne vit jamais en base** : la table porte une clé d'objet, le contenu part
//! au stockage S3 (principe II).

pub mod modele;
pub mod repository;
pub mod service;

pub use modele::{
    BrandingNiveau, BrandingResolu, ChampResolu, EcrireBranding, ErreurBranding, couleur_valide,
};
pub use service::{
    LOGO_TAILLE_MAX, MENTION_NON_FISCALE, ServiceBranding, rendre_document_test,
};
