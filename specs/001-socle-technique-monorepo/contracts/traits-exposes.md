# Traits exposés entre crates — cycle 001

**Règle de nommage** : les entités et colonnes sont en **français sans accent** ; les **traits
d'abstraction sont en anglais** (`JurisdictionAdapter`, `PlatformAdapter`, `FneGateway`…). Cette
distinction vient des documents de référence et n'est pas une préférence.

**Règle d'architecture** : un crate expose son API interne sous forme de trait ; les dépendances
sont injectées (cadrage §13.2). Aucune requête ne joint deux schémas de modules — **toute lecture
inter-modules passe par un trait de cette liste**.

---

## 1. `OutboxWriter` — `socle/synchronisation`

Le trait le plus important du cycle : c'est lui qui rend la porte P-05 tenable.

```rust
pub trait OutboxWriter: Send + Sync {
    /// Écrit un événement DANS la transaction fournie. Jamais en dehors.
    async fn ecrire<'t>(
        &self,
        tx: &mut sqlx::PgTransaction<'t>,
        evenement: EvenementAEcrire,
    ) -> Result<(), ErreurOutbox>;
}

pub struct EvenementAEcrire {
    pub id: Uuid,                      // UUID v7
    pub tenant_id: Uuid,
    pub etablissement_id: Option<Uuid>,
    pub type_evenement: String,        // 'note_etablissement.creee'
    pub agregat: String,
    pub agregat_id: Uuid,
    pub version_schema: i16,
    pub payload: serde_json::Value,    // COMPLET et DÉNORMALISÉ
}
```

**La signature est le mécanisme, pas une convention de style.** `ecrire` prend une transaction en
paramètre et n'ouvre jamais la sienne : il devient **impossible d'écrire un événement hors de la
transaction métier**. Un trait qui prendrait un pool laisserait le développeur libre de créer une
seconde transaction — et la garantie « même transaction SQL » de TRX-02 reposerait sur sa
discipline. Ici elle repose sur le compilateur.

`survenu_le` et `sequence_etablissement` ne sont **pas** dans la structure : ils sont posés par
l'implémentation côté serveur. L'appelant ne peut donc pas fournir un horodatage de terminal ni
casser la monotonie de la séquence.

---

## 2. `EventConsumer` — `socle/synchronisation`

```rust
pub trait EventConsumer: Send + Sync {
    fn nom(&self) -> &'static str;
    /// DOIT être idempotent : trois présentations = effet d'une seule.
    async fn consommer(&self, evenement: &EvenementPublie) -> Result<(), ErreurConsommation>;
}
```

Le worker in-process (R-08) parcourt les consommateurs enregistrés. Un consommateur qui échoue
laisse l'événement **non marqué publié** — donc republié au prochain tour, indéfiniment. Aucun
événement n'est jamais abandonné ni supprimé.

---

## 3. `EstablishmentDirectory` — `socle/etablissements`

```rust
pub trait EstablishmentDirectory: Send + Sync {
    async fn etablissement(&self, id: Uuid) -> Result<Option<Etablissement>, ErreurLecture>;
    async fn appartient_au_tenant(&self, etablissement_id: Uuid, tenant_id: Uuid)
        -> Result<bool, ErreurLecture>;
}
```

C'est par ce trait — et jamais par un `JOIN` — que les crates de `capacites/` et `verticales/`
liront un établissement. Poser le trait dès maintenant, alors qu'aucun crate ne le consomme
encore, est ce qui empêchera le premier `JOIN` inter-schémas d'être écrit « juste cette fois » au
cycle HEB.

---

## 4. `JurisdictionAdapter` — `socle/fiscalite`

**Déclaré seulement. Aucune implémentation à ce cycle**, pas même `CoteDIvoire` : les règles
fiscales sont FIS-01 à FIS-07, tranche T3. Le déclarer maintenant est ce qui garantit qu'aucune
règle fiscale ne pourra naître ailleurs (principe V, porte P-12).

```rust
pub trait JurisdictionAdapter: Send + Sync {
    fn compute_taxes(&self, base: &BaseImposable) -> Result<VentilationTaxes, ErreurFiscale>;
    fn required_document_fields(&self, type_doc: TypeDocument) -> Vec<ChampObligatoire>;
    fn emission_channel(&self) -> EmissionChannel;
    async fn certify(&self, document: &DocumentAcertifier) -> Result<Certification, ErreurFiscale>;
    fn remittance_reports(&self, periode: Periode) -> Vec<EtatDeReversement>;
}

pub enum EmissionChannel { FneApi, Terne }   // provision §14.5 — `Terne` jamais implémenté au MVP
```

Les cinq méthodes viennent littéralement du cadrage §14.1. Les types associés
(`BaseImposable`, `VentilationTaxes`…) sont déclarés dans le crate `domain` et **restent vides ou
minimaux** à ce cycle — les remplir serait implémenter la fiscalité, ce que le principe X
interdit.

---

## 5. `PlatformAdapter` — `app/core` (TypeScript)

```ts
export interface PlatformAdapter {
  imprimer(doc: DocumentImprimable): Promise<ResultatCapacite>
  scanner(): Promise<ResultatCapacite<string>>
  ocrPieceIdentite(image: Blob): Promise<ResultatCapacite<ChampsPieceIdentite>>
  stockageSecurise: StockageSecurise
  notifier(notification: Notification): Promise<ResultatCapacite>
  geolocaliser(): Promise<ResultatCapacite<Position>>
  etatReseau(): EtatReseau
}

export type ResultatCapacite<T = void> =
  | { disponible: true;  valeur: T }
  | { disponible: false; raison: CapaciteIndisponible }   // JAMAIS un throw silencieux
```

**Le type de retour est le contrat, pas l'interface.** `ResultatCapacite` force chaque appelant à
traiter le cas « cette plateforme ne sait pas faire ça » — la constitution (principe VII) exige
qu'une capacité absente **le dise explicitement à l'utilisateur**. Une méthode qui renverrait
`Promise<void>` et lèverait une exception laisserait le choix de l'ignorer, et l'interface
grise-sans-explication réapparaîtrait au premier écran mobile.

Quatre implémentations à créer, **vides mais conformes** : `desktop`, `android`, `ios`, `web`.
Chacune renvoie `{ disponible: false }` pour tout ce qu'elle ne sait pas encore faire. Aucun
composant n'importe `@tauri-apps/api` — c'est ce que vérifie la porte P-15.

---

## 6. Traits déclarés par la stack mais **hors périmètre de ce cycle**

Nommés ici pour que leur absence soit un choix visible, pas un oubli : `FneGateway`
(`Partenaire` | `Direct`, FIS-02), `PaymentProvider` (CinetPay, CAI-03), `AccessController`
(provision §14.21). Aucun n'est déclaré à ce cycle — leur emplacement est
`socle/fiscalite` pour les deux premiers, à trancher pour le troisième.
