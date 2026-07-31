//! `socle/fiscalite` — obligations réglementaires et **provisions comptables**.
//!
//! Ce crate porte le trait [`JurisdictionAdapter`] — **déclaré, jamais implémenté à ce cycle** —
//! et le schéma `fiscalite`, qui héberge les deux tables de provision comptable de TRX-02b.
//!
//! **Pourquoi les provisions comptables sont ici** : la constitution fixe limitativement les neuf
//! crates de `socle/` ; il n'existe pas de crate `comptabilite` et en créer un demanderait un
//! amendement. Parmi les neuf, `fiscalite` est le seul dont le domaine est la production
//! d'obligations réglementaires à partir d'événements métier — ce que `mapping_comptable` fait
//! exactement. `documents` a été écarté : il traite la numérotation des pièces, pas leur
//! traduction comptable.

#![forbid(unsafe_code)]

use kaya_domain::fiscal::{
    BaseImposable, Certification, ChampObligatoire, DocumentAcertifier, EmissionChannel,
    ErreurFiscale, EtatDeReversement, Periode, TypeDocument, VentilationTaxes,
};

/// Règles fiscales d'une juridiction.
///
/// # Déclaré, **jamais implémenté à ce cycle** — pas même `CoteDIvoire`
///
/// Les règles fiscales sont FIS-01 à FIS-07, tranche T3. Le principe X l'interdit ici, et ce
/// n'est pas une contrainte de calendrier : le déclarer **maintenant** est ce qui garantit
/// qu'aucune règle fiscale ne pourra naître ailleurs.
///
/// La porte **P-12** le vérifie mécaniquement — aucun crate hors `socle/fiscalite` ne référence
/// les types de taxe de `domain`. Sans ce trait, la première règle serait écrite dans le service
/// qui en a besoin, la deuxième dans un autre, et le jour où la Côte d'Ivoire changerait un taux
/// il faudrait les retrouver toutes.
///
/// Les cinq méthodes viennent **littéralement** du cadrage §14.1. Les réordonner ou les fusionner
/// ferait diverger le code de la source qui fait foi.
///
/// # Ce que la bascule d'adaptateur doit rester
///
/// Un changement de **configuration de tenant**, sans toucher au métier (constitution, § Pile
/// technique imposée). Une méthode qui prendrait un paramètre « pays » au lieu d'être portée par
/// l'implémentation ferait ressortir la juridiction dans le code appelant — exactement ce que le
/// trait sert à éviter.
#[async_trait::async_trait]
pub trait JurisdictionAdapter: Send + Sync {
    /// Calcule la ventilation de taxes d'une assiette.
    ///
    /// **TVA, taxe de nuitée et taxe de développement touristique sont des SORTIES de cette
    /// méthode, jamais des constantes** (principe V).
    fn compute_taxes(&self, base: &BaseImposable) -> Result<VentilationTaxes, ErreurFiscale>;

    /// Champs qu'un type de document doit obligatoirement porter dans cette juridiction.
    fn required_document_fields(&self, type_doc: TypeDocument) -> Vec<ChampObligatoire>;

    /// Canal d'émission du document fiscal.
    fn emission_channel(&self) -> EmissionChannel;

    /// Certifie un document auprès de l'administration.
    ///
    /// **L'API FNE n'expose aucune clé d'idempotence** (principe V). L'état `INDETERMINEE`
    /// (timeout) n'est **jamais** rejoué automatiquement : rapprochement manuel obligatoire. Un
    /// rejeu naïf produirait une double facturation avec double consommation de sticker.
    async fn certify(&self, document: &DocumentAcertifier)
    -> Result<Certification, ErreurFiscale>;

    /// États de reversement à produire pour une période.
    fn remittance_reports(&self, periode: Periode) -> Vec<EtatDeReversement>;
}
