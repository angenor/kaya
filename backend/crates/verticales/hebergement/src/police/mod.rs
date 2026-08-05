//! `police` — la fiche de police et sa numérotation (SEJ-02).
//!
//! Terme utilisateur : **« Fiche de police »** / *Police registration form*, conservé tel quel
//! (lexique v1.6.0) — c'est le terme de l'usage ivoirien, celui que la gendarmerie emploie.
//!
//! # Ce module ne recopie AUCUNE identité
//!
//! La fiche référence le séjour ; les identités viennent du client — lu par `AnnuaireClients` — et
//! des accompagnants. Recopier nom, prénoms et numéro de pièce créerait une **troisième** surface
//! de rétention pour la même donnée sensible, après `comptes.personne` et
//! `hebergement.accompagnant`, et la purge de TRX-06 devrait alors la connaître.
//!
//! **Le gabarit officiel n'est pas inventé** : le registre minimal est en base, le formulaire du
//! pilote est un **rendu** qui s'ajoutera sans migration.

pub mod repository;
