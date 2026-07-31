//! `socle/etablissements` — tenants, établissements et contexte multi-tenant.
//!
//! **L'entité centrale du produit est l'établissement, pas l'hôtel** (constitution, préambule).
//! Ce crate ne suppose ni hébergement, ni point de vente : un maquis seul, un pressing seul et
//! une résidence meublée seule sont des établissements valides.
//!
//! C'est aussi ici que vit la pose du tenant courant — le chemin de code le plus sensible du
//! produit, celui qui décide quelles lignes un client voit.

#![forbid(unsafe_code)]

pub mod note;
pub mod tenant_context;

use uuid::Uuid;

/// Établissement, tel que les autres modules le lisent.
///
/// Forme minimale, alignée sur la table du cycle 001. ETB-01 l'enrichira.
#[derive(Debug, Clone)]
pub struct Etablissement {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub nom: String,
    pub fuseau_horaire: String,
    pub devise: String,
}

/// Échec de lecture d'un établissement.
#[derive(Debug, thiserror::Error)]
pub enum ErreurLecture {
    #[error("lecture impossible : {0}")]
    Base(#[from] sqlx::Error),
}

/// **Le trait par lequel les autres modules lisent un établissement — jamais par jointure.**
///
/// # Pourquoi il est posé maintenant, alors qu'aucun crate ne le consomme
///
/// Aucune requête ne joint deux schémas de modules (principe II, porte P-04). Les crates de
/// `capacites/` et `verticales/` liront donc un établissement **par ce trait**.
///
/// Le poser aujourd'hui, à vide, est précisément ce qui empêchera le premier `JOIN` inter-schémas
/// d'être écrit « juste cette fois » au cycle HEB. Quand la question se posera, l'alternative
/// existera déjà — et une alternative qui existe se prend, là où une alternative à construire se
/// contourne.
#[async_trait::async_trait]
pub trait EstablishmentDirectory: Send + Sync {
    async fn etablissement(&self, id: Uuid) -> Result<Option<Etablissement>, ErreurLecture>;

    async fn appartient_au_tenant(
        &self,
        etablissement_id: Uuid,
        tenant_id: Uuid,
    ) -> Result<bool, ErreurLecture>;
}
