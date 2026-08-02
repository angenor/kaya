//! Le montant d'un passage, sa rebascule de palier et sa bascule en nuitée — **HEB-04**.
//!
//! > **Le moteur calcule, il ne facture pas.**
//!
//! Aucune ligne de note n'est écrite ici : la note est SEJ-03, tranche T2. Ce que ce module
//! produit est une **décision de tarification** que SEJ-03 consommera.
//!
//! Deux couches, et la séparation est le point : [`bareme`] est une **fonction pure** que l'on
//! teste sur des cas figés, sans base et sans réseau ; [`service`] va chercher la durée réelle
//! depuis l'horodatage d'autorité serveur et trace la rebascule au registre des actions.

pub mod bareme;
pub mod service;

pub use bareme::{Calcul, ErreurBareme, Palier, calculer};
pub use service::ServiceTarification;
