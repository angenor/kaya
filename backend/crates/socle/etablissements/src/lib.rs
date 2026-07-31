//! `socle/etablissements` — tenants, établissements et contexte multi-tenant.
//!
//! **L'entité centrale du produit est l'établissement, pas l'hôtel** (constitution, préambule).
//! Ce crate ne suppose ni hébergement, ni point de vente : un maquis seul, un pressing seul et
//! une résidence meublée seule sont des établissements valides.
//!
//! C'est aussi ici que vit la pose du tenant courant, le chemin de code le plus sensible du
//! produit — celui qui décide quelles lignes un client voit.

#![forbid(unsafe_code)]
