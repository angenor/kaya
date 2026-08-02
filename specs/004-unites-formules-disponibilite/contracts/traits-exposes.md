# Traits exposés — cycle 004 (HEB)

Ce que le crate `kaya-hebergement` expose aux autres crates, et ce qu'il consomme d'eux.

**La règle qui commande tout** : aucune requête ne joint deux schémas de modules (principe II,
porte P-04). Les lectures inter-modules passent par un trait. Ce document dit lesquels.

---

## 1. Ce que `hebergement` CONSOMME du socle

Aucun de ces traits n'est nouveau — les trois existent depuis les cycles 002 et 003.

| Trait | Crate | Ce que HEB en tire |
|---|---|---|
| `EstablishmentDirectory` | `socle/etablissements` | **Le fuseau horaire** de l'établissement — indispensable pour convertir les plages de demi-journée en instants (R-13), et la **devise**, pour formater les prix |
| `RegistreModules` | `socle/etablissements` | Le module `HEBERGEMENT` est-il actif ? Tout endpoint du cycle le vérifie avant d'agir, et rend le refus normalisé au cycle 002 |
| `ResolveurConfiguration` | `socle/etablissements` | Les trois paramètres du catalogue : heures d'arrivée et de départ standard, seuil de bascule en nuitée |
| `OutboxWriter` | `socle/synchronisation` | Les cinq événements du cycle, **dans la transaction** de leur écriture |
| Trait d'audit | `socle/comptes` | La trace de rebascule de palier au registre des actions (CPT-04) |
| `AccessController` | `socle/comptes` | La garde des cinq permissions |

**Le sens de la dépendance est le bon** : `verticales/` peut dépendre de `socle/`, jamais
l'inverse. La porte **P-03** et `backend/tests/architecture.rs` le vérifient sur le graphe de
dépendances.

---

## 2. Ce que `hebergement` EXPOSE

**Trois traits, tous destinés à des consommateurs qui n'existent pas encore.** Le principe X
(« prêt ≠ construit ») commande de justifier chacun : un trait sans consommateur est une
abstraction spéculative.

### 2.1 `MoteurDisponibilite` — consommé par SEJ-02, cycle suivant

```rust
#[async_trait]
pub trait MoteurDisponibilite: Send + Sync {
    /// Les unités attribuables d'une catégorie sur un intervalle.
    ///
    /// **Cette réponse ne garantit rien** : entre la lecture et l'attribution, une autre
    /// transaction peut prendre l'unité. La garantie est la contrainte d'exclusion.
    async fn unites_disponibles(
        &self,
        etablissement_id: Uuid,
        categorie_id: Uuid,
        periode: PgRange<OffsetDateTime>,
    ) -> Result<Vec<UniteDisponible>, ErreurDisponibilite>;

    /// Attribue une unité. **Prend la transaction** — le check-in de SEJ-02 attribuera l'unité
    /// et ouvrira la note dans la même transaction.
    async fn attribuer(
        &self,
        tx: &mut sqlx::PgTransaction<'_>,
        demande: DemandeAttribution,
    ) -> Result<Occupation, ErreurAttribution>;
}
```

**Pourquoi ce trait existe dès maintenant, alors que SEJ-02 n'est pas écrit.** `attribuer` prend
la transaction, exactement comme `OutboxWriter::ecrire` : c'est ce qui rendra possible au check-in
d'attribuer l'unité **et** d'ouvrir la note dans une seule transaction. Un trait qui prendrait un
pool obligerait SEJ-02 à deux transactions — donc à une saga pour une opération qui n'en demande
pas.

L'endpoint d'attribution de ce cycle (§2.2 du contrat HTTP) est le **premier** consommateur : le
trait n'est pas spéculatif, il a un implémenteur et un appelant dès sa création.

### 2.2 `MoteurTarification` — consommé par SEJ-03 (T2) et FIS-03 (T3)

```rust
#[async_trait]
pub trait MoteurTarification: Send + Sync {
    /// Le montant dû pour une occupation, à l'instant d'autorité serveur.
    ///
    /// **Calcule, ne facture pas** : aucune ligne de note n'est écrite. SEJ-03 consommera
    /// cette décision.
    async fn calculer(
        &self,
        occupation_id: Uuid,
    ) -> Result<DecisionTarification, ErreurTarification>;
}

pub struct DecisionTarification {
    pub duree_reelle_minutes: i64,
    pub formule_appliquee: FamilleFormule,
    pub palier_retenu_minutes: Option<i32>,
    pub heures_supplementaires: i32,
    /// **Entier d'unité mineure** (principe V, porte P-10).
    pub montant_du_mineur: i64,
    pub devise: String,
    pub rebascule: Option<Rebascule>,
    pub instant_autorite: OffsetDateTime,
}
```

### 2.3 `ParametrageFiscalHebergement` — consommé par `JurisdictionAdapter` en T3

```rust
#[async_trait]
pub trait ParametrageFiscalHebergement: Send + Sync {
    /// Le paramétrage fiscal d'une formule — **jamais un calcul**.
    async fn parametrage(
        &self,
        formule_id: Uuid,
    ) -> Result<ParametrageFiscal, ErreurParametrage>;
}

pub struct ParametrageFiscal {
    pub assujettie_taxe_nuitee: bool,
    /// **`None` = formule NON assujettie.** La contrainte `formule_regle_fiscale_coherente`
    /// rend impossible une formule assujettie sans règle — ce n'est donc jamais un état
    /// d'attente, et il n'y a rien à refuser.
    ///
    /// ⛔ **L'axe « par client » n'est pas résolu.** `UneNuiteeParOccupation` réduit trois
    /// nuits à une ; elle ne dit RIEN de trois personnes, alors que la taxe est due « par
    /// nuitée et par client » (cadrage §9.6) et que les accompagnants comptent (SEJ-02).
    /// Le consommateur — FIS-03 — devra trancher cet axe explicitement, jamais par défaut.
    pub regle_conversion: Option<RegleConversionTaxe>,
}
```

> **C'est la frontière du principe V, et elle est ici.** Ce trait rend un **paramètre**, jamais un
> montant de taxe. Toute règle fiscale vit dans `JurisdictionAdapter` (`socle/fiscalite`), et la
> porte **P-12** fait échouer le build sur une règle fiscale trouvée ailleurs. `hebergement`
> stocke `assujettie_taxe_nuitee` et `regle_conversion_taxe` **sans jamais les interpréter**.
>
> Écrit explicitement parce que c'est la confusion la plus tentante du cycle : le crate qui
> détient le paramètre semble être celui qui doit l'appliquer. Il ne l'est pas.

---

## 3. Le type de refus qui traverse toute la verticale

```rust
#[derive(Debug, thiserror::Error)]
pub enum ErreurAttribution {
    /// **Violation de la contrainte d'exclusion** — la seule qui vient de la base.
    #[error("unite_deja_occupee")]
    UniteDejaOccupee,
    #[error("formule_hors_categorie")]
    FormuleHorsCategorie,
    #[error("plage_non_fractionnable")]
    PlageNonFractionnable,
    #[error("intervalle_invalide")]
    IntervalleInvalide,
    #[error("duree_hors_contrainte")]
    DureeHorsContrainte,
    #[error("service_inactif")]
    ServiceInactif,
    #[error(transparent)]
    Base(#[from] sqlx::Error),
}
```

### La traduction de la violation d'exclusion — écrite **une fois**

```rust
/// **`ErrorKind::ExclusionViolation` existe en sqlx 0.9 ; l'accesseur symétrique, non.**
///
/// Le trait `DatabaseError` porte `is_unique_violation()`, `is_foreign_key_violation()` et
/// `is_check_violation()` — mais **pas** `is_exclusion_violation()`. Vérifié dans les sources
/// de `sqlx-core` 0.9.0. Écrire la forme symétrique par analogie ne compilerait pas.
///
/// `ErrorKind` est `#[non_exhaustive]` : `matches!` est la forme correcte, un `match` exhaustif
/// ne compilerait pas davantage.
pub fn est_violation_exclusion(erreur: &sqlx::Error, contrainte: &str) -> bool {
    matches!(erreur, sqlx::Error::Database(e)
        if matches!(e.kind(), sqlx::error::ErrorKind::ExclusionViolation)
            && e.constraint() == Some(contrainte))
}
```

**Le nom de contrainte est vérifié, pas seulement le genre d'erreur.** Une table qui gagnerait une
seconde contrainte d'exclusion ferait autrement passer ses violations pour des doubles
attributions. Aujourd'hui `hebergement.occupation` n'en a qu'une ; le test le vérifie, ce qui rend
l'ajout d'une seconde visible plutôt que silencieux.

---

## 4. Ce que le crate n'expose PAS

| Absent | Motif |
|---|---|
| Un trait de statut d'unité | HEB-06 — et le statut d'occupation est dérivé, pas exposé comme un état |
| Un trait de calendrier tarifaire | HEB-07, P1 |
| Un trait de prestation incluse | HEB-09 — table seule, aucune logique |
| Un trait `ResourceReservable` générique | Abstraction spéculative à un seul implémenteur (research R-09). À rouvrir quand RSV apparaîtra, avec deux implémenteurs |
| Tout accès direct à `hebergement.*` depuis un autre crate | Interdit par P-04 — passe par les traits ci-dessus |
