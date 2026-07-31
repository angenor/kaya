//! Montants et devises.
//!
//! **Un montant est un entier d'unité mineure, jamais un flottant ni un décimal** (principe V).
//! Le XOF a zéro décimale : 15 500 XOF se représente `15_500`. Un flottant rouvrirait le risque
//! d'arrondi que le principe V ferme, et la porte **P-10** échoue sur tout `FLOAT`, `REAL` ou
//! `DOUBLE` porteur d'un montant.
//!
//! Les **quantités**, elles, ne passent jamais par un entier : voir [`Quantite`].

use serde::{Deserialize, Serialize};

/// Montant en **unités mineures** de la devise portée par l'établissement.
///
/// Le type est volontairement un alias transparent plutôt qu'une structure : au cycle 001 aucun
/// calcul monétaire n'existe, et une abstraction posée avant son premier usage se subit
/// (constitution, § Flux de développement — « pas de généricité prématurée »).
pub type MontantMineur = i64;

/// Taux exprimé en **millièmes entiers** : `180` vaut 18 %.
///
/// Même raison que pour les montants : un taux en flottant ferait rentrer l'arrondi par la
/// petite porte. Le format de charge utile de l'outbox (`data-model.md` §4.2) l'impose.
pub type TauxMillieme = i32;

/// Quantité vendue, reçue ou mouvementée.
///
/// **`NUMERIC`, jamais un entier** (principe V). Un hôtel vend 1 bière ; une quincaillerie vendra
/// 2,3 mètres de fer ; une boulangerie achètera 47,5 kg de farine. Passer d'entier à décimal
/// après mise en production imposerait de migrer toutes les lignes de vente et tous les
/// mouvements de stock.
pub type Quantite = rust_decimal::Decimal;

/// Code de devise **ISO 4217**, porté par l'établissement.
///
/// Aucune devise n'est active hors `XOF` au MVP — les autres sont une provision du cadrage §14,
/// donc un choix de modèle de données, pas une fonctionnalité.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Devise(pub String);

impl Devise {
    /// Franc CFA d'Afrique de l'Ouest — **zéro décimale**.
    pub const XOF: &'static str = "XOF";
}

impl std::fmt::Display for Devise {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
