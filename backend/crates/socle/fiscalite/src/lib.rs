//! `socle/fiscalite` — obligations réglementaires et **provisions comptables**.
//!
//! Ce crate porte le trait `JurisdictionAdapter` — **déclaré, jamais implémenté à ce cycle** —
//! et le schéma `fiscalite`, qui héberge les deux tables de provision comptable de TRX-02b.
//!
//! **Pourquoi les provisions comptables sont ici** : la constitution fixe limitativement les neuf
//! crates de `socle/` ; il n'existe pas de crate `comptabilite` et en créer un demanderait un
//! amendement. Parmi les neuf, `fiscalite` est le seul dont le domaine est la production
//! d'obligations réglementaires à partir d'événements métier — ce que `mapping_comptable` fait
//! exactement. `documents` a été écarté : il traite la numérotation des pièces, pas leur
//! traduction comptable.

#![forbid(unsafe_code)]
