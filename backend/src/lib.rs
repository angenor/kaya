//! `kaya-backend` — **harnais des tests d'intégration transverses**.
//!
//! Ce paquet ne porte aucune logique de production et n'est lié à aucun binaire. Il existe pour
//! une seule raison : héberger `backend/tests/`, les tests qui traversent plusieurs crates et ne
//! peuvent donc appartenir à aucun d'eux — catalogue RLS, isolation multi-tenant, immuabilité de
//! l'outbox, graphe de dépendances entre familles de crates.
//!
//! Les tester depuis un crate métier reviendrait à lui donner une dépendance vers tous les
//! autres, ce que la hiérarchie du principe II interdit.

#![forbid(unsafe_code)]
