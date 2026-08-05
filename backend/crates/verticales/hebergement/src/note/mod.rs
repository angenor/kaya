//! `note` — la note d'un séjour et ses lignes (SEJ-02, SEJ-04).
//!
//! Terme utilisateur : la note **arrêtée** s'annonce « La note est arrêtée : plus rien ne peut s'y
//! ajouter » (lexique v1.6.0). Jamais « clôturée », « figée » ni « verrouillée ».
//!
//! # Ce que ce module ne porte PAS, et qui viendra ailleurs
//!
//! **Les consommations des points de vente, les transferts de charges et les remises sont
//! SEJ-03**, tranche T2. Ce cycle honore `ligne_sejour` pour son **sous-ensemble hébergement** :
//! la ligne de la période prévue, et les lignes d'ajustement.
//!
//! `provisions_sans_logique.rs` vérifie qu'aucun point d'entrée ne les expose — sans quoi le
//! contrat annoncerait une note complète que le produit ne sait pas remplir.

pub mod repository;

pub use repository::NouvelleLigne;
