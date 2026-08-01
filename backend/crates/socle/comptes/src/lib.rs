//! `socle/comptes` — comptes, rôles, permissions, appareils enrôlés.
//!
//! **Coquille du cycle 001.** Ce crate existe, compile et occupe sa place dans la hiérarchie
//! du principe II ; il ne porte aucune logique. Son contenu vient du cycle CPT.
//!
//! Le créer vide maintenant n'est pas décoratif : c'est ce qui rend la porte **P-03** capable
//! de constater dès aujourd'hui qu'aucune arête interdite n'existe dans le graphe de
//! dépendances, au lieu de l'apprendre au premier crate réellement écrit.
//!
//! **Le cycle 003 (CPT) le remplit.** Premier apport : la constatation que les deux chaînes
//! cryptographiques du cycle se construisent pour `linux/amd64` — voir
//! [`preuve_cryptographique`], échafaudage retiré dès que les vrais chemins les exercent.

#![forbid(unsafe_code)]

pub mod authentification;
pub mod preuve_cryptographique;
