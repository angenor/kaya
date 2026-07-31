//! Types de la fiscalité — **déclarés, jamais remplis à ce cycle**.
//!
//! Ils existent parce que la signature de `JurisdictionAdapter` (cadrage §14.1) en a besoin, et
//! pour une seconde raison qui compte davantage : la porte **P-12** vérifie qu'**aucun crate hors
//! `socle/fiscalite` ne référence ces types**. Une porte n'a de valeur que si elle a une cible ;
//! les déclarer maintenant lui en donne une avant que la première règle fiscale ne soit écrite.
//!
//! Les remplir serait implémenter la fiscalité — tranche T3, stories FIS-01 à FIS-07 — ce que le
//! principe X interdit à ce cycle.

use crate::monnaie::{Devise, MontantMineur, TauxMillieme};
use serde::{Deserialize, Serialize};

/// Assiette soumise au calcul de taxes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseImposable {
    pub montant_mineur: MontantMineur,
    pub devise: Devise,
}

/// Une ligne de taxe : son code, son assiette, son taux et son montant.
///
/// `assiette_mineur` et `taux_millieme` sont **optionnels** : la taxe de nuitée est un forfait
/// par nuit et par personne, sans assiette ni taux (cadrage §11.3). Les rendre obligatoires
/// forcerait à inventer une assiette fictive.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LigneTaxe {
    pub code: String,
    pub assiette_mineur: Option<MontantMineur>,
    pub taux_millieme: Option<TauxMillieme>,
    pub montant_mineur: MontantMineur,
}

/// Ventilation complète des taxes d'une opération.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VentilationTaxes {
    pub lignes: Vec<LigneTaxe>,
}

/// Nature du document produit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TypeDocument {
    /// Document **fiscal** — numérotation et cycle de vie propres.
    Facture,
    /// Document **opérationnel** — porte obligatoirement la mention
    /// « Document non fiscal — ne tient pas lieu de facture » (principe V).
    NoteProvisoire,
}

/// Champ qu'une juridiction rend obligatoire sur un type de document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChampObligatoire {
    pub nom: String,
}

/// Canal d'émission du document fiscal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmissionChannel {
    /// API FNE — le seul canal du MVP.
    FneApi,
    /// Provision du cadrage §14.5. **Jamais implémenté au MVP.**
    Terne,
}

/// Période de reversement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Periode {
    pub debut: time::Date,
    pub fin: time::Date,
}

/// État de reversement produit pour l'administration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EtatDeReversement {
    pub libelle: String,
}

/// Document présenté à la certification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentAcertifier {
    pub type_document: TypeDocument,
}

/// Retour de certification.
///
/// **Les `id` d'items renvoyés par l'API de certification sont persistés** (principe V) : sans
/// eux, aucun avoir n'est possible, et l'erreur est irrattrapable a posteriori.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Certification {
    pub reference: String,
    pub ids_items: Vec<String>,
}

/// Erreur du domaine fiscal.
#[derive(Debug, thiserror::Error)]
pub enum ErreurFiscale {
    /// Aucune règle n'est écrite à ce cycle : toute implémentation renverrait cette variante.
    #[error("aucune règle fiscale n'est implémentée avant la tranche T3 (FIS-01 à FIS-07)")]
    NonImplemente,
}
