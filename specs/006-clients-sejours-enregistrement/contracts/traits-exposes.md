# Traits exposés et consommés — cycle 006

**Deux traits exposés, cinq consommés.** La règle qui commande tout : *aucune requête ne joint deux
schémas de modules* (principe II, porte **P-04**). Les lectures inter-modules passent par un trait —
ce fichier dit lesquels, et **justifie chacun**, le principe X interdisant l'abstraction
spéculative.

Ce cycle est **le premier où deux schémas se parlent sur le chemin chaud** : un séjour affiche
toujours le nom de son client. C'est la jointure que tout le monde écrirait, et c'est celle qui
n'existe pas.

---

## 1 · `AnnuaireClients` — exposé par `socle/comptes`, consommé par `verticales/hebergement`

### Le trait

```rust
/// Résumé d'un client, tel que la verticale hébergement le lit — **jamais par jointure**.
///
/// ⚠️ **Aucun numéro de pièce d'identité n'y figure**, et c'est une décision. Il est soumis à la
/// rétention de 90 jours de TRX-06 ; le laisser traverser vers une verticale multiplierait les
/// endroits où il faudra le purger. La fiche de police le lit **là où il vit**, dans `comptes`.
#[derive(Debug, Clone)]
pub struct ClientResume {
    pub personne_id: Uuid,
    pub nom: String,
    pub prenoms: Option<String>,
    pub telephone: Option<String>,
    /// Vrai quand la pièce d'identité est enregistrée — ce que la fiche de police doit savoir
    /// **sans lire la pièce elle-même** (FR-047).
    pub piece_enregistree: bool,
}

#[async_trait::async_trait]
pub trait AnnuaireClients: Send + Sync {
    /// Les résumés de plusieurs clients, **en une requête**.
    ///
    /// ⚠️ **`resumes(&[Uuid])`, jamais `resume(Uuid)`.** Une signature unitaire produirait N+1
    /// requêtes sur la liste des séjours en cours — et c'est le détail qui décide si l'écran de
    /// départ s'ouvre en 200 ms ou en deux secondes. La forme par lot n'est pas une optimisation
    /// prématurée : elle est **la seule qui ne se dégrade pas** quand la liste grandit.
    ///
    /// Les identifiants inconnus sont **absents** de la réponse, jamais rendus en `None` : un
    /// séjour dont le client a été purgé (TRX-06) reste lisible, sans nom.
    async fn resumes(&self, ids: &[Uuid]) -> Result<Vec<ClientResume>, ErreurAnnuaireClients>;

    /// Le client existe et appartient au tenant courant.
    ///
    /// Appelé par l'ouverture d'un séjour pour refuser un `client_id` inventé — la RLS empêcherait
    /// déjà la lecture d'un client d'un autre tenant, mais un refus explicite vaut mieux qu'une
    /// ligne orpheline qu'aucune contrainte ne peut interdire, la clé étrangère étant impossible.
    async fn existe(&self, id: Uuid) -> Result<bool, ErreurAnnuaireClients>;
}
```

### Pourquoi il existe

**Il n'est pas spéculatif : il a un implémenteur et un appelant dès sa création.** L'implémenteur
est `PgAnnuaireClients` dans `socle/comptes` ; l'appelant est le service de séjour, sur trois
chemins — la liste des séjours en cours (`R7`), la fiche d'un séjour, et la reconnaissance d'un
client au passage (`R4`, « M. Bakayoko — 7ᵉ passage »).

**Sans lui, la voie facile serait une jointure `hebergement.sejour × comptes.personne`.** P-04
l'attraperait — mais après coup, une fois l'écran écrit. *Une alternative qui existe se prend ; une
alternative à construire se contourne* : c'est le raisonnement d'`EstablishmentDirectory`, posé au
cycle 001, et il vaut ici avec un appelant réel en plus.

### Le sens interdit, et pourquoi il n'a aucun garde-fou naturel

**`socle/comptes` ne lit JAMAIS `hebergement.sejour`.** L'historique des séjours d'un client
(`GET /clients/{id}/sejours`, opération 5) paraît appartenir au client — il est servi **depuis le
crate `hebergement`**.

Si `comptes` lisait `hebergement.sejour`, ce serait **deux violations d'un coup** :

| Violation | Porte |
|---|---|
| Jointure inter-schémas | **P-04** |
| Arête `socle/ → verticales/` | **P-03** |

Le chemin HTTP `/api/v1/clients/{id}/sejours` cache ce découpage à l'appelant, et c'est normal :
**le contrat est une façade, pas une carte des crates.**

### Ce que ce trait ne fait pas

| Absence | Motif |
|---|---|
| Créer ou modifier un client | Ce serait donner à une verticale le droit d'écrire dans le socle. La création passe par les opérations 2 et 4, servies par `comptes` |
| Rendre le numéro de pièce | Rétention TRX-06 — voir le commentaire de `ClientResume` |
| Chercher un client | La recherche est une opération de `comptes`, servie directement. Une verticale n'a pas à chercher dans l'annuaire, elle a à **résoudre des identifiants** |

---

## 2 · `LecteurSejour` — exposé par `verticales/hebergement`, consommé par SEJ-03 et FIS

```rust
/// Ce qu'un séjour est, pour un consommateur qui n'est pas la verticale hébergement.
#[derive(Debug, Clone)]
pub struct SejourResume {
    pub id: Uuid,
    pub etablissement_id: Uuid,
    pub client_id: Option<Uuid>,
    pub statut: StatutSejour,
    pub note_id: Uuid,
    pub devise: String,
    /// **Entier d'unité mineure** (principe V, P-10). Somme des lignes, jamais une colonne
    /// totalisatrice — une colonne se désynchronise en silence.
    pub total_mineur: i64,
}

/// Le constat de taxe **figé**, tel que `JurisdictionAdapter` le lira en T3.
///
/// > **C'est la frontière du principe V, et elle est ici.**
///
/// Cette structure porte des **faits** et un **paramétrage recopié**. Elle ne porte **aucun
/// montant de taxe** : `nuitees_assujetties` et `montant_mineur` sont posés au schéma et
/// **jamais alimentés par ce cycle**. Décider quelles nuits sont assujetties est une règle
/// fiscale — `une_nuitee_par_occupation` réduit trois nuits à une —, et la porte **P-12** fait
/// échouer le build sur une règle fiscale trouvée hors de `JurisdictionAdapter`.
///
/// Écrit explicitement parce que c'est la confusion la plus tentante du cycle : le crate qui
/// détient le constat semble être celui qui doit en tirer le montant. **Il ne l'est pas** —
/// exactement comme `ParametrageFiscalHebergement` au cycle 004.
#[derive(Debug, Clone)]
pub struct ConstatTaxeSejour {
    pub sejour_id: Uuid,
    pub nuits_constatees: i32,
    pub nombre_personnes: i32,
    pub assujettie_taxe_nuitee: bool,
    pub regle_conversion_taxe: Option<RegleConversionTaxe>,
    pub classement_etablissement: String,
    pub commune: String,
    /// Horodatage d'**autorité** du figeage.
    pub fige_le: OffsetDateTime,
}

#[async_trait::async_trait]
pub trait LecteurSejour: Send + Sync {
    async fn resume(&self, sejour_id: Uuid) -> Result<Option<SejourResume>, ErreurSejour>;

    /// Les séjours **ouverts** d'un établissement — ce dont SEJ-03 aura besoin pour proposer
    /// « porter cette consommation sur une chambre ».
    async fn ouverts(&self, etablissement_id: Uuid) -> Result<Vec<SejourResume>, ErreurSejour>;

    /// Le constat figé d'un séjour clos. `None` si le séjour est encore ouvert.
    async fn constat_taxe(&self, sejour_id: Uuid)
        -> Result<Option<ConstatTaxeSejour>, ErreurSejour>;
}
```

### Pourquoi il existe **avant** son consommateur

Contrairement à `AnnuaireClients`, il n'a **aucun appelant à ce cycle**. Sa raison est de **forme**,
et elle est écrite ici plutôt que supposée :

- **SEJ-03** (T2) devra rattacher une consommation de bar à un séjour. Sans ce trait, `restauration`
  ou `bar` lirait `hebergement.sejour` — jointure inter-schémas.
- **FIS-03** (T3) devra lire le constat figé pour produire le montant. Sans ce trait, `fiscalite`
  lirait `hebergement.taxe_sejour_constat` — même violation, sur la donnée la plus sensible du
  produit.

C'est le raisonnement de `ParametrageFiscalHebergement` au cycle 004, mot pour mot : *une
alternative qui existe se prend ; une alternative à construire se contourne*. Deux consommateurs
sont **nommés et datés** — ce n'est pas une abstraction à un implémenteur imaginaire.

> ⚠️ **Le risque de ce trait est qu'il tente FIS de recalculer.** `constat_taxe` rend un
> paramétrage, pas un montant. Le jour où FIS-03 sera écrit, la tentation sera de rappeler la
> formule vivante plutôt que la copie figée — ce qui ferait bouger un séjour clos. **Le constat est
> la seule source légitime**, et son immuabilité est portée par le privilège
> (`GRANT SELECT, INSERT` seuls), pas par cette phrase.

---

## Traits consommés par ce cycle

| Trait | Exposé par | Ce que ce cycle en fait |
|---|---|---|
| **`MoteurDisponibilite`** | `verticales/hebergement` (cycle 004) | ★ `attribuer(&mut tx, …)` — **la promesse du cycle 004 se vérifie ici**. C'est ce qui permet d'attribuer l'unité **et** d'ouvrir la note dans une seule transaction, donc de tenir le budget d'un appel bloquant |
| **`MoteurTarification`** | `verticales/hebergement` (cycle 004) | `calculer(occupation_id)` au départ — **rebascule de palier comprise**. Le cycle 006 ne réimplémente aucun barème |
| **`ParametrageFiscalHebergement`** | `verticales/hebergement` (cycle 004) | Lu **une seule fois, au départ**, pour **recopier** `assujettie_taxe_nuitee` et `regle_conversion_taxe` dans le constat. **Recopier n'est pas interpréter** |
| **`OutboxWriter`** | `socle/synchronisation` | 9 événements, écrits **dans** la transaction — la signature l'impose, ce n'est pas une discipline |
| **`EstablishmentDirectory`** · **`RegistreModules`** | `socle/etablissements` | La garde : l'établissement existe, le module `HEBERGEMENT` y est actif. Sans elle, un maquis pourrait ouvrir un séjour |
| **`AccessController`** | `socle/comptes` | `exiger(&contexte, "heb.sejour.ouvrir")` et les six autres |

---

## Le graphe, et ce que P-03 vérifie

```
                        ┌──────────────────────────┐
                        │   verticales/hebergement │
                        │   sejour · note · police │
                        │   taxe · occupation      │
                        └─────────┬────────────────┘
                                  │ consomme
      ┌──────────────┬────────────┼─────────────┬──────────────┐
      ▼              ▼            ▼             ▼              ▼
 AnnuaireClients  Outbox   Establishment   RegistreModules  AccessController
 socle/comptes    socle/   Directory       socle/           socle/comptes
                  synchro  socle/etabl.    etablissements

           ▲
           │  LecteurSejour — exposé, consommé par PERSONNE à ce cycle
           │  (SEJ-03 en T2, FIS-03 en T3 — nommés et datés)
           └────────────────────────────────────────────────────────────
```

**Toutes les flèches vont de `verticales/` vers `socle/`. Aucune ne remonte.**
`backend/tests/architecture.rs` fait échouer le build sur l'arête inverse, et la porte **P-03**
prouve que sa cible n'est pas vide : `verticales/hebergement` porte des symboles publics, et en
gagne à ce cycle.

**Le piège concret à surveiller pendant l'implémentation** : faire remonter un type de séjour dans
une signature du socle. Le cas le plus tentant est la file de réconciliation — un accompagnant
orphelin s'écrit dans `synchronisation.reconciliation_orpheline`. **La charge utile est du JSON
opaque pour le socle** ; `kaya_synchronisation` ne doit connaître ni `Accompagnant`, ni `Sejour`.
