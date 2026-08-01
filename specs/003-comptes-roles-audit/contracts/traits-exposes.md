# Traits exposés par `socle/comptes`

**Trois traits.** Ils sont le **seul chemin** par lequel les autres crates atteignent ce que ce
cycle produit : aucune requête ne joint deux schémas de modules (principe II, porte **P-04**), et
aucun crate du socle ne dépend d'une verticale (porte **P-03**).

`#[async_trait::async_trait]` sur les trois, pour la même raison qu'au cycle 002 : un `async fn`
natif de trait n'est pas dyn-compatible, et l'injection de dépendances du cadrage §13.2 suppose
`Arc<dyn Trait>`.

---

## Une note de nommage, parce que ce cycle mélange les deux conventions

`CLAUDE.md` range les **traits d'abstraction** en anglais et nomme `AccessController` dans la
liste des traits canoniques du produit. Il est donc en anglais, sans discussion. Les deux autres
n'y figurent pas : ils suivent la convention effective du cycle 002 — `RegistreModules`,
`RepertoirePointsDeVente`, `ObstacleDesactivation` — et sont en français.

**La règle qui en découle** : un trait nommé par les documents de référence garde son nom ; un
trait nouveau suit le français des identifiants métier. Écrit ici pour que le cycle suivant n'ait
pas à en décider une troisième fois.

---

## 1 · `AccessController` — la seule autorité sur « a-t-il le droit »

**Le trait canonique du produit**, nommé au préambule de `CLAUDE.md`.

```rust
#[async_trait::async_trait]
pub trait AccessController: Send + Sync {
    /// Permissions effectives d'un compte sur un établissement — **l'UNION de ses rôles**.
    async fn permissions_effectives(
        &self,
        compte_id: Uuid,
        etablissement_id: Option<Uuid>,
    ) -> Result<BTreeSet<String>, ErreurAcces>;

    /// Le compte détient-il cette permission ? Convenance sur la précédente.
    async fn detient(
        &self,
        compte_id: Uuid,
        etablissement_id: Option<Uuid>,
        permission: &str,
    ) -> Result<bool, ErreurAcces>;
}
```

**Union, jamais priorité.** `BTreeSet` plutôt que `Vec` : le type dit l'unicité et l'ordre stable,
et rend structurellement impossible la faute de FR-017 — un « rôle principal » dont les
permissions primeraient. Aucune signature de ce trait n'accepte de rôle ; seul l'ensemble sort.

**Ce qu'il n'expose volontairement pas** :

| Absent | Raison |
|---|---|
| Les **rôles** d'un compte | Un consommateur qui branche sur un rôle plutôt que sur une permission recrée la hiérarchie que le principe VII interdit. Seul l'écran `G3` les affiche, et il passe par l'API, pas par ce trait |
| L'attribution ou le retrait d'un rôle | **Classe C**, écriture réservée au service. Un trait d'écriture des droits serait un chemin d'élévation offert à tout crate |
| Le condensat de mot de passe | Aucun autre module n'a de raison de le voir, ni même de savoir qu'il existe |

**Consommateurs attendus** : `api/` pour la garde des handlers ; tout cycle ultérieur qui protège
une action (`verticales/hebergement` pour `heb.unite.attribuer`, `socle/caisse` pour une remise).

---

## 2 · `AnnuaireComptes` — lire qui est l'auteur, sans jointure

```rust
#[async_trait::async_trait]
pub trait AnnuaireComptes: Send + Sync {
    async fn compte(&self, id: Uuid) -> Result<Option<CompteResume>, ErreurAcces>;

    /// Lecture en lot — le journal d'audit affiche cent auteurs par page.
    async fn comptes(&self, ids: &[Uuid]) -> Result<BTreeMap<Uuid, CompteResume>, ErreurAcces>;
}

pub struct CompteResume {
    pub id: Uuid,
    pub nom_affichage: String,   // depuis `personne`, jamais l'identifiant de connexion
    pub actif: bool,
}
```

**Pourquoi ce trait existe.** `note_etablissement.auteur_compte_id` porte, depuis le cycle 001, un
UUID **sans clé étrangère** — le module doré appelle cela « le point le plus contre-intuitif du
patron ». Ce trait est ce qui rend cet UUID lisible : il est la contrepartie promise de l'absence
de jointure, et sans lui la tentation du `JOIN comptes.compte` reviendrait au premier écran qui
affiche un auteur.

**`comptes(&[Uuid])` n'est pas une optimisation prématurée** : le journal d'audit affiche une page
d'entrées d'auteurs différents. Sans lecture en lot, l'écran `G4` ferait cent appels — c'est le
problème classique, et il est moins cher de le fermer à l'écriture du trait qu'à la première
lenteur en clientèle.

**`nom_affichage` vient de `personne`**, jamais de l'identifiant de connexion : afficher un numéro
de téléphone dans un journal consulté par le propriétaire diffuserait un contact personnel dans un
registre à rétention illimitée.

---

## 3 · `JournalAudit` — le trait que tous les cycles suivants appelleront

```rust
#[async_trait::async_trait]
pub trait JournalAudit: Send + Sync {
    /// Écrit une entrée **dans la transaction de l'opération tracée**.
    async fn tracer(
        &self,
        tx: &mut sqlx::PgTransaction<'_>,
        entree: EntreeAudit,
    ) -> Result<(), ErreurAudit>;
}

pub struct EntreeAudit {
    pub id: Uuid,                        // UUID v7 — rejeu inoffensif
    pub tenant_id: Uuid,
    pub etablissement_id: Option<Uuid>,
    pub type_action: TypeActionAudit,    // énumération fermée, research R-09
    pub auteur_compte_id: Uuid,
    pub cible_type: String,
    pub cible_id: Option<Uuid>,
    pub contexte: serde_json::Value,
    pub horodatage_client: Option<OffsetDateTime>,
}
```

**La signature prend la transaction et n'en ouvre jamais une** — exactement comme
`OutboxWriter::ecrire` au cycle 001, et pour la même raison : c'est la signature, pas la
discipline, qui garantit que la trace et l'opération tombent ou passent ensemble. Écrire l'entrée
ailleurs demanderait de fabriquer une seconde transaction et de la passer explicitement, ce qui se
voit en revue et ne s'écrit pas par distraction.

**`type_action` est une énumération fermée**, pas une chaîne. Les dix familles de CPT-04 y
figurent toutes ; huit n'ont encore aucun appelant (research R-09). Un `String` laisserait un
cycle inventer `remise_appliquee` à côté de `remise`, et le filtre de `G4` cesserait de trouver la
moitié des entrées sans que rien n'échoue.

**Ce que ce trait n'expose pas — et c'est délibéré** : aucune méthode de lecture. La consultation
du journal passe par l'API (`journal_audit_lister`), sous permission. Un trait de lecture
permettrait à n'importe quel crate de lire le registre du propriétaire sans passer par le contrôle
d'accès.

**Aucune méthode de suppression ni de correction.** Une correction est une nouvelle entrée
(FR-033). Le trait ne peut pas offrir ce que les privilèges de la table refusent — `GRANT SELECT,
INSERT` seulement.

---

## Où ces traits sont implémentés, et où ils sont injectés

| | Défini dans | Implémenté dans | Injecté par |
|---|---|---|---|
| `AccessController` | `socle/comptes` | `socle/comptes` | `api/` (garde des handlers) |
| `AnnuaireComptes` | `socle/comptes` | `socle/comptes` | `api/`, puis tout écran affichant un auteur |
| `JournalAudit` | `socle/comptes` | `socle/comptes` | `api/`, et **tout service qui trace** — `socle/caisse`, `verticales/*` à partir de T2 |

**Le sens de dépendance est celui du principe II** : `socle/comptes` définit et implémente ; les
verticales consomment. Aucune ne s'inverse — contrairement à `ObstacleDesactivation` du cycle 002,
qui est le seul trait du produit implémenté par les verticales. `backend/tests/architecture.rs`
vérifie le graphe.

---

## Ce que ce cycle NE fournit pas, et qui sera demandé

Écrit ici pour que le cycle suivant ne le suppose pas acquis :

| Manque | À figer par |
|---|---|
| **L'enrôlement d'appareil** — la table existe, aucun trait ne l'expose | CPT-05, tranche T4 |
| **Le sélecteur d'établissement actif** — la connexion en choisit un, sans bascule | ETB-06 |
| **La lecture du journal d'audit par un autre crate** — volontairement absente | Aucun besoin identifié ; à rouvrir s'il apparaît |
| **Le décompte d'entrées d'audit par période** — pour les alertes | DIR-04, tranche T5 |
