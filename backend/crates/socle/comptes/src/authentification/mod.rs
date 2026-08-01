//! Authentification — hachage, politique de mot de passe, indiscernabilité des échecs.
//!
//! **Ce sous-module n'a pas de `repository.rs`, et c'est délibéré** : il n'écrit dans aucune
//! table. Il fait du calcul — hacher, comparer, refuser — sur des données que d'autres couches
//! lisent et écrivent. Lui donner un repository par symétrie avec `personne/` ou `roles/`
//! obligerait à inventer une table pour la remplir.

pub mod mots_de_passe_compromis;
