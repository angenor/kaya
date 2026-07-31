//! `socle/synchronisation` — le **grand livre d'événements**.
//!
//! Ce crate porte le schéma `synchronisation`, la table `evenement_outbox`, les traits
//! `OutboxWriter` et `EventConsumer`, et le worker de publication in-process.
//!
//! **L'outbox n'est pas une file de messages** (principe II) : rétention illimitée, charge utile
//! financière complète et dénormalisée, immuable. Une correction est un nouvel événement, jamais
//! une modification de l'ancien.

#![forbid(unsafe_code)]
