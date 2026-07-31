# Traits exposés — Cycle 002 · `socle/etablissements`

**Phase 1 du plan** · 2026-07-31 · [plan.md](../plan.md) · [data-model.md](../data-model.md)

Six traits. Ils sont **le seul chemin** par lequel les autres crates lisent ce que ce cycle
produit : aucune requête ne joint deux schémas de modules (principe II, porte P-04), et aucun crate
du socle ne dépend d'une verticale (porte P-03).

Cinq sont des traits de **lecture**, implémentés ici et consommés ailleurs. Le sixième inverse le
sens, et c'est le seul point délicat du cycle.

---

## 1. `EstablishmentDirectory` — étendu

Le trait existe depuis le cycle 001, posé **à vide** pour que le premier `JOIN` inter-schémas ne
soit pas écrit « juste cette fois » au cycle HEB. Ce cycle lui donne son contenu réel.

```rust
pub struct Etablissement {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub nom: String,
    pub fuseau_horaire: String,      // déjà présent
    pub devise: String,              // déjà présent — ISO 4217
    // ── ajoutés par ETB-01 ──
    pub juridiction: String,         // sélectionne le JurisdictionAdapter, n'encode aucune règle
    pub classement: Classement,
    pub commune: String,
    pub adresse: Option<String>,
    pub ncc: Option<String>,
}

pub enum Classement {
    Etoiles(u8),                     // 1..=5
    NonClasse,
    ResidenceMeublee,
}
```

`Classement` est un **type somme**, pas une paire `(texte, Option<u8>)` : le nombre d'étoiles
n'existe que pour une seule variante, et la base l'impose déjà par une égalité de conditions. Deux
représentations de la même règle, l'une en base et l'autre dans le type — c'est voulu : la première
protège des scripts, la seconde des développeurs.

**Consommateurs** : tous les crates qui ont besoin d'un fuseau, d'une devise ou d'un classement —
`socle/fiscalite` (barème de nuitée), `socle/caisse` (clôture en temps local),
`verticales/hebergement` (calcul de durée).

---

## 2. `RegistreModules` — quels services un établissement rend

```rust
#[async_trait::async_trait]
pub trait RegistreModules: Send + Sync {
    /// Codes des modules **actifs** de l'établissement.
    async fn modules_actifs(&self, etablissement_id: Uuid)
        -> Result<Vec<String>, ErreurRegistre>;

    /// Ce module est-il actif ici ? Réponse binaire, sans exception si le module n'existe pas.
    async fn module_actif(&self, etablissement_id: Uuid, code: &str)
        -> Result<bool, ErreurRegistre>;
}
```

**Le trait ne rend jamais les modules inactifs.** Une méthode `tous_les_modules_avec_etat` serait
la porte d'entrée du grisé que le principe VII interdit : donnée à l'interface, elle produirait une
liste où figurent les services que l'établissement n'a pas. Ce que l'interface ne doit pas montrer,
elle ne doit pas non plus recevoir.

**Consommateurs** : l'accueil à tuiles (cycle CPT), chaque verticale au démarrage d'une opération,
la console éditeur.

---

## 3. `RegistreCapacites` — ce qu'un service consomme

```rust
pub struct CapaciteDeclaree {
    pub capacite_code: String,       // `STOCK` seule au MVP
    pub profil_code: String,         // `SIMPLE` seul au MVP
}

#[async_trait::async_trait]
pub trait RegistreCapacites: Send + Sync {
    async fn capacites_du_module(&self, etablissement_id: Uuid, module_code: &str)
        -> Result<Vec<CapaciteDeclaree>, ErreurRegistre>;

    async fn consomme(&self, etablissement_id: Uuid, module_code: &str, capacite_code: &str)
        -> Result<Option<CapaciteDeclaree>, ErreurRegistre>;
}
```

**Consommateur unique au MVP** : `capacites/stocks`, qui n'agit que si `STOCK` est déclarée au
profil `SIMPLE`. Le trait rend `Option` plutôt que `bool` parce que le profil décide du
comportement, et qu'un `bool` obligerait à un second appel — donc à deux vérités possibles entre
les deux.

---

## 4. `ResolveurConfiguration` — le composant le plus réutilisé du produit

```rust
pub enum Portee { Tenant, Etablissement, Module, PointDeVente }

/// D'où l'on résout. Les `Option` absents raccourcissent la chaîne sans l'inventer.
pub struct Cible {
    pub tenant_id: Uuid,
    pub etablissement_id: Option<Uuid>,
    pub module_code: Option<String>,
    pub point_de_vente_id: Option<Uuid>,
}

pub struct ValeurResolue {
    pub valeur: serde_json::Value,
    pub origine: Portee,             // OBLIGATOIRE — l'écran distingue hérité de surchargé
}

#[async_trait::async_trait]
pub trait ResolveurConfiguration: Send + Sync {
    async fn resoudre(&self, cible: &Cible, cle: &str)
        -> Result<Option<ValeurResolue>, ErreurConfiguration>;

    /// Toutes les valeurs applicables à la cible, en une descente.
    async fn resoudre_tout(&self, cible: &Cible)
        -> Result<BTreeMap<String, ValeurResolue>, ErreurConfiguration>;
}
```

Trois choix, chacun contre une faute précise :

- **`Option<ValeurResolue>`, jamais un défaut.** Un défaut rendu par le résolveur serait un
  paramètre en dur déguisé en commodité, et le principe I·c l'interdit. L'appelant qui a besoin
  d'un défaut le déclare chez lui, où on peut le voir.
- **`origine` non optionnelle.** Un champ optionnel serait ignoré par le premier appelant pressé,
  et l'écran ne pourrait plus dire « hérité du tenant » — la fonctionnalité disparaîtrait sans que
  personne ne la retire.
- **`resoudre_tout`.** L'écran `G1` affiche une trentaine de paramètres à terme ; trente appels
  feraient trente descentes de chaîne.

**Consommateurs** : tous les cycles suivants. HEB (temps de remise en état, heures standard,
barème de passage), FIS (taux, taxes), CAI (seuil d'écart, terminal bloquant), IMP (politique
d'impression), STK (seuil d'alerte), RSV (expiration, délai d'annulation), QRC (paniers max),
CPT (rayon de géorepérage), SYN (dérive d'horloge).

---

## 5. `RepertoirePointsDeVente` — lecture d'un point de vente

```rust
pub struct PointDeVente {
    pub id: Uuid,
    pub etablissement_id: Uuid,
    pub module_code: String,
    pub nom: String,
    pub caisse_id: Option<Uuid>,
    pub tables: Vec<TablePdv>,       // vide ⇒ comptoir
}

#[async_trait::async_trait]
pub trait RepertoirePointsDeVente: Send + Sync {
    async fn points_de_vente(&self, etablissement_id: Uuid)
        -> Result<Vec<PointDeVente>, ErreurRegistre>;

    async fn point_de_vente(&self, id: Uuid)
        -> Result<Option<PointDeVente>, ErreurRegistre>;
}
```

**Aucune méthode `est_comptoir`.** `tables.is_empty()` dit la même chose sans qu'une seconde source
puisse la contredire. Une méthode dédiée finirait par lire un drapeau, et un drapeau finit par
mentir.

**Consommateurs** : `verticales/restauration`, `verticales/bar`, `verticales/pressing` (cycle PDV),
`socle/caisse` pour le rattachement.

---

## 6. `ObstacleDesactivation` — le trait dont le sens est inversé

C'est le seul point de conception délicat du cycle. FR-016 exige qu'un service portant des
opérations en cours ne puisse pas être désactivé — un séjour ouvert, une addition non réglée. Or
cette information vit dans les **verticales**, et un crate du socle ne peut pas en dépendre
(porte P-03).

**Inversion de dépendance.** Le trait est **défini** dans `socle/etablissements`, **implémenté** par
chaque verticale, et **injecté** à l'assemblage — dans `backend/api/`, famille « assemblage », seul
endroit du produit qui a le droit de connaître tout le monde.

```rust
/// Une raison de refuser la désactivation d'un service.
pub struct Obstacle {
    pub module_code: String,
    pub motif_cle: String,           // clé i18n — jamais une phrase
    pub nombre: u32,                 // « 3 séjours en cours »
}

#[async_trait::async_trait]
pub trait ObstacleDesactivation: Send + Sync {
    /// Qu'est-ce qui empêche de désactiver ce service, à cet instant ?
    async fn obstacles(&self, etablissement_id: Uuid, module_code: &str)
        -> Result<Vec<Obstacle>, ErreurRegistre>;
}
```

Le service de désactivation interroge **tous** les obstacles enregistrés et refuse s'il en reste un.

```rust
pub struct ServiceModules {
    obstacles: Vec<Arc<dyn ObstacleDesactivation>>,   // vide à ce cycle
}
```

**À ce cycle, la liste est vide et la désactivation est donc libre.** C'est exact, et non un trou :
aucune verticale ne crée encore d'opération. Ce que le cycle livre est le **point d'accrochage**,
posé maintenant pour la même raison qu'`EstablishmentDirectory` l'a été à vide au cycle 001 : quand
la question se posera au cycle SEJ, l'alternative existera déjà. Une alternative qui existe se
prend ; une alternative à construire se contourne.

**Vérification.** Un test enregistre un obstacle factice et constate que la désactivation est
refusée en le nommant. Sans ce test, un point d'accrochage jamais exercé peut être cassé par un
refactoring sans que rien ne le signale — le cycle 001 a montré qu'une porte qui ne trouve jamais
rien est indistinguable d'une porte qui n'a rien à trouver.

---

## Câblage — qui construit quoi

| Trait | Implémentation | Injectée dans |
|---|---|---|
| `EstablishmentDirectory` | `PgEstablishmentDirectory` | `EtatApplication` |
| `RegistreModules` | `PgRegistreModules` | `EtatApplication`, chaque verticale |
| `RegistreCapacites` | `PgRegistreCapacites` | `capacites/stocks` |
| `ResolveurConfiguration` | `PgResolveurConfiguration` | `EtatApplication`, tous les cycles suivants |
| `RepertoirePointsDeVente` | `PgRepertoirePointsDeVente` | `EtatApplication` |
| `ObstacleDesactivation` | *aucune à ce cycle* | `ServiceModules`, par `Vec<Arc<dyn …>>` |

Tous les traits sont annotés `#[async_trait::async_trait]` : la dyn-compatibilité est requise par
l'injection `Arc<dyn Trait>` du cadrage §13.2, et Rust ne la fournit pas nativement pour un
`async fn` de trait. Même choix contraint qu'au cycle 001 sur `OutboxWriter`.

---

## Ce que ces traits n'exposent pas

| Absent | Raison |
|---|---|
| Écriture au référentiel des modules ou des capacités | Réservée à l'éditeur (ETB-08). Aucun tenant n'y écrit, donc aucun trait ne l'offre |
| Liste des modules **inactifs** | Ce que l'interface ne doit pas montrer, elle ne doit pas recevoir (principe VII) |
| Liste des capacités non implémentées | Idem. Le refus explicite protège les autres chemins d'écriture, il n'alimente pas l'interface |
| Valeur par défaut d'un paramètre | Serait un paramètre en dur (principe I·c) |
| Accès au binaire du logo | Le trait rend une clé d'objet ; le contenu passe par l'interface S3 (principe II) |
