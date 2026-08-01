//! Le **registre des actions** — écriture, taxonomie, lecture filtrée.
//!
//! Terme utilisateur : **« Registre des actions »** / *Activity log* (`docs/design/lexique.md`).
//! « Journal d'audit » est le nom technique et n'apparaît jamais à l'écran.
//!
//! # Écriture et consultation sont deux moments différents du cycle
//!
//! L'**écriture** est ici, en Phase 2 des tâches, parce que FR-024 impose qu'une attribution de
//! rôle écrive une entrée : US3 en dépend, elle ne peut pas attendre. La **consultation** —
//! repository de lecture, endpoint, écran `G4` — relève d'US5. La priorité P2 de cette story se
//! lit dans le moment où son écran arrive, pas dans celui où sa table est créée.
//!
//! # Aucun point d'entrée d'écriture, et c'est une décision (research R-17)
//!
//! Une entrée voyage **toujours** avec l'opération qu'elle trace, dans sa transaction. Livrer un
//! `POST /journal-audit` produirait deux choses : une cible vide, puisque rien ne l'appellerait,
//! et une surface par laquelle un terminal forgerait des entrées dans le registre censé le
//! surveiller.

pub mod modele;
pub mod repository;
pub mod service;
pub mod taxonomie;

pub use modele::{EntreeAudit, EntreeAuditEnregistree, ErreurAudit};
pub use repository::{Curseur, FiltresAudit, LIMITE_DEFAUT, LIMITE_MAX, PageAudit};
pub use service::{JournalAudit, JournalAuditPostgres};
pub use taxonomie::TypeActionAudit;
