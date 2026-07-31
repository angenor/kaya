# Kaya — Le module doré

*Patron de référence de tous les cycles. Produit par le cycle 001 (TRX), sur `note_etablissement`.*

**Version 1.0.0 — 2026-07-31**

---

## À quoi sert ce document

Le cadrage §13.1 exige qu'un module soit écrit **à la main, avant toute génération assistée**, et
serve de patron. Ce document est ce patron.

Il ne décrit pas « comment on écrit du Rust ». Il décrit **les six ou sept décisions par couche
qui ne se devinent pas** et que chaque cycle réintroduirait de travers s'il partait d'un exemple
trouvé en ligne. La raison est datée et précise : le gel retient **sqlx 0.9.0**, et la totalité de
la documentation publique, des exemples et des réponses en ligne vise encore `0.8.x`. Deux
changements suffisent à les rendre inutilisables — `#3723` impose `AssertSqlSafe` sur toute
requête non littérale, `#3541` modifie la sortie des macros `query!()`.

**Test de ce document** : un développeur doit pouvoir reproduire une seconde tranche verticale en
ne lisant que ce fichier. S'il doit ouvrir le code du module doré, ce document est incomplet.

---

## L'entité, et pourquoi elle est sans importance

`note_etablissement` est une note interne libre attachée à un établissement. Aucune valeur
métier — c'est délibéré. Un patron construit sur une entité importante mélangerait ce qui relève
de la structure et ce qui relève du métier ; il faudrait ensuite deviner lequel des deux on
recopie.

Terme utilisateur : **« Note interne »** / *Internal note* (`docs/design/lexique.md`).
`note_etablissement` est le nom **technique** — table, type, événement — et n'apparaît jamais à
l'écran.

---

## Les six couches

| # | Couche | Fichier |
|---|---|---|
| 1 | Migration | `backend/migrations/0004_note_etablissement.sql` |
| 2 | Registre hors-ligne | `docs/registre-classes-offline.md` §5.1 |
| 3 | Repository | `backend/crates/socle/etablissements/src/note/repository.rs` |
| 4 | Service | `backend/crates/socle/etablissements/src/note/service.rs` |
| 5 | Handler | `backend/api/src/routes/notes.rs` |
| 6 | Tests | `backend/tests/note_etablissement_classe_a.rs` |

La septième — l'écran — est **absente**. Voir la dernière section : c'est une décision, pas un
oubli, et elle a des conséquences que le cycle ETB doit reprendre.

---

## Couche 1 — La migration

### Trois décisions qui rendent la table réutilisable comme patron

**L'identifiant est fourni par le client, jamais généré par la base.**

```sql
id UUID PRIMARY KEY,   -- UUID v7 généré côté client
```

C'est ce qui rend le rejeu inoffensif (cadrage §11.5.1). Trois envois de la même écriture entrent
en conflit de clé primaire et produisent un enregistrement unique. Une clé générée par la base en
produirait trois, et le terminal qui vide sa file après une coupure créerait des doublons
silencieux — découverts trois mois plus tard, en clôture.

**Deux horodatages distincts, jamais fusionnés.**

```sql
horodatage_client TIMESTAMPTZ     NULL,   -- indicatif, aucune règle ne s'y appuie
cree_le           TIMESTAMPTZ NOT NULL DEFAULT now()   -- AUTORITÉ SERVEUR
```

Les réunir « pour simplifier » est la faute décrite au cadrage §11.4. Un terminal mal réglé
décalerait des durées de passage, donc des montants.

**Aucune clé étrangère vers un autre module — le point le plus contre-intuitif du patron.**

```sql
auteur_compte_id UUID NOT NULL,   -- pas de REFERENCES : socle/comptes est un autre module
```

Ce n'est pas parce que `socle/comptes` n'existe pas encore. Même quand il existera, une clé
étrangère joindrait deux schémas de modules, ce que le principe II interdit. **L'intégrité
référentielle inter-modules passe par un trait exposé, jamais par la base.**

### Le patron RLS, identique partout

```sql
ALTER TABLE <schema>.<table> ENABLE ROW LEVEL SECURITY;
ALTER TABLE <schema>.<table> FORCE  ROW LEVEL SECURITY;

CREATE POLICY isolation_tenant ON <schema>.<table>
    USING      (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);
```

Trois éléments, aucun optionnel :

- **`FORCE`** — sans lui, le propriétaire des tables reste hors politique, et la première tâche de
  maintenance voit tous les clients.
- **`WITH CHECK`** — sans lui, un tenant peut **insérer** chez un autre. C'est la fuite la moins
  visible du produit : elle n'apparaît dans aucune lecture.
- **le second argument `true`** de `current_setting` — sans lui, une transaction sans contexte
  lève une erreur au lieu de ne rien voir. Un résultat vide ne peut se dégrader qu'en résultat
  vide ; une erreur peut être avalée par un `catch` mal placé et devenir un accès ouvert.

### Aucune migration n'écrit de données sur une table en `FORCE ROW LEVEL SECURITY`

*Règle générale issue du cycle 002, à appliquer à toute migration désormais.*

**`INSERT` et `UPDATE` de migration ne fonctionnent pas** sur une table protégée par `FORCE ROW
LEVEL SECURITY` — et le pire est qu'ils **ne se plaignent pas**.

Une migration s'exécute sous `kaya_owner`. `FORCE` applique les politiques au propriétaire lui-même,
et `current_setting('app.current_tenant', true)` vaut `NULL` hors requête applicative. La
comparaison vaut `NULL`, **aucune ligne n'est touchée, et aucune erreur n'est levée** : la migration
réussit en n'écrivant rien. Le défaut se découvre au premier calcul qui lit la colonne vide.

Trois formes, et laquelle employer :

| Ce qu'on veut écrire | La forme qui marche |
|---|---|
| Remplir une colonne ajoutée sur une table peuplée | **`ADD COLUMN ... NOT NULL DEFAULT`** — c'est du DDL, il ne passe par aucune politique. Puis `DROP DEFAULT` si la valeur n'a pas de sens permanent |
| Peupler un **référentiel global** | `CREATE TABLE` → **`INSERT`** → `ENABLE`/`FORCE` → `CREATE POLICY`. Les valeurs entrent quand la table n'est encore gardée par rien. L'ordre n'est pas interchangeable |
| Alimenter un référentiel **après** son activation | Une politique `administration_editeur ... FOR ALL TO kaya_owner`, posée à la création. C'est elle qui rend possible un `INSERT` de migration ultérieure |
| Écrire des données de client | **La mécanique de seeds**, qui pose le tenant courant — jamais une migration |

Le cycle 002 a rencontré les trois : `0007` remplit sept colonnes par `DEFAULT`, `0008` insère les
quatre référentiels avant d'activer, et `0011` peuple le catalogue **après** activation grâce à la
politique posée en `0008`.

### Les privilèges disent la classe hors-ligne

```sql
GRANT SELECT, INSERT ON etablissements.note_etablissement TO kaya_app;
```

Ni `UPDATE` ni `DELETE` : une entité de **classe A** est append-only. Une correction est une
nouvelle ligne. Accorder `UPDATE` casserait la commutativité que le test de désordre vérifie — et
le classement en A deviendrait faux sans que rien ne le signale.

---

## Couche 2 — Le registre des classes hors-ligne

L'entité est déclarée dans **le même changement** que sa migration, avec une entrée au journal
§13. Depuis ce cycle, `backend/tests/classes_offline.rs` compare les tables réelles aux entités
déclarées et **fait échouer le build** sur toute table absente.

Sens de la comparaison : **table → registre**. Une entité déclarée mais pas encore implémentée est
normale ; une table non déclarée est l'erreur.

---

## Couche 3 — Le repository

**Toutes les requêtes passent par les macros `query!` / `query_as!` / `query_scalar!` sur
littéral.** Elles sont vérifiées à la compilation contre la vraie base (porte P-18), et
`AssertSqlSafe` n'apparaît nulle part.

### Le repository prend la transaction, il ne l'ouvre pas

```rust
pub async fn inserer(
    tx: &mut sqlx::PgTransaction<'_>,
    tenant_id: Uuid,
    note: &CreerNote,
) -> Result<(NoteEtablissement, Issue), ErreurNote>
```

C'est le service qui décide de la portée transactionnelle, parce que c'est lui qui doit y inclure
l'événement outbox.

### L'insertion idempotente renseigne l'appelant

```sql
INSERT INTO ... VALUES (...)
ON CONFLICT (id) DO NOTHING
RETURNING ...
```

`RETURNING` renvoie une ligne quand l'insertion a eu lieu, et **rien** en cas de conflit. C'est
exactement ce qu'il faut pour distinguer `201` de `200`, sans second aller-retour dans le cas
normal.

### Deux pièges de sqlx 0.9 neutralisés ici

**`query!` sur un `SELECT` ne s'exécute pas avec `.execute()`.** Un `SELECT` produit un `Map`, qui
n'a pas cette méthode. `set_config` est une fonction, donc un `SELECT` :

```rust
sqlx::query_scalar!("SELECT set_config('app.current_tenant', $1, true)", tenant_id.to_string())
    .fetch_one(&mut **tx)    // et non .execute()
    .await?;
```

**`&mut **tx`** — le déréférencement double est la forme attendue par sqlx 0.9 pour exécuter sur
une transaction empruntée.

### Trier sur l'horodatage d'autorité, et départager

```sql
ORDER BY cree_le DESC, id DESC
```

Jamais sur `horodatage_client` : trier sur l'horloge d'un terminal ferait remonter en tête la note
d'un appareil mal réglé. L'ordre secondaire n'est pas décoratif — deux notes créées dans la même
transaction partagent `now()`, et sans départage la pagination sauterait ou répéterait des lignes.
L'UUID v7 étant ordonné dans le temps, il départage dans le bon sens.

---

## Couche 4 — Le service

### La règle centrale du produit

> Toute transition d'état écrit un événement outbox **dans la même transaction** (principe II,
> porte P-05).

Elle n'est pas tenue par la discipline mais par une signature :

```rust
async fn ecrire(&self, tx: &mut sqlx::PgTransaction<'_>, evenement: EvenementAEcrire)
    -> Result<(), ErreurOutbox>;
```

`OutboxWriter::ecrire` **prend la transaction et n'en ouvre jamais une**. Écrire l'événement
ailleurs demanderait de fabriquer une seconde transaction et de la passer explicitement — ce qui
se voit en revue et ne s'écrit pas par distraction. Un trait qui prendrait un pool laisserait la
garantie reposer sur l'attention du développeur.

### L'ordre des opérations, et le point qu'on écrirait mal

1. valider — inutile d'ouvrir une transaction pour un texte vide ;
2. ouvrir la transaction, puis **poser le tenant courant** ;
3. vérifier l'existence de l'agrégat parent — pour un `404` plutôt qu'une violation de clé ;
4. insérer, idempotent ;
5. **émettre l'événement uniquement si la ligne vient d'être créée** ;
6. commit.

**Le point 5 est celui qu'on écrirait mal.** Un rejeu ne produit aucun nouvel événement. L'émettre
à chaque tentative ferait du grand livre le journal des tentatives réseau du terminal, et non
celui des transitions d'état : la reconstitution compterait trois fois une note écrite une fois.

### `survenu_le` vient de la base, pas du processus

L'implémentation pose `now()` en SQL. Deux instances d'API n'ont pas la même horloge ; la base,
elle, est unique.

### La séquence par établissement laisse des trous, et c'est voulu

Les séquences PostgreSQL ne sont pas transactionnelles : un rollback laisse un trou. La séquence
garantit **l'ordre et la détection de manque**, pas la continuité. Garantir la continuité
imposerait un verrou par établissement sur le chemin d'écriture le plus chaud du produit.

**Écrit ici pour que personne ne « corrige » plus tard un trou qui n'est pas un bug.**

---

## Couche 5 — Le handler

### Le chemin n'est écrit qu'une fois

```rust
#[utoipa::path(tag = "etablissements", responses(...))]
#[post("")]
pub async fn creer(...)
```

Ni `post,` ni `path = "..."` dans l'annotation utoipa : le verbe et le chemin sont déduits de
l'attribut de routage d'Actix (feature `actix_extras`). Les écrire deux fois laisserait le contrat
annoncer une adresse que le serveur ne sert pas, sans que rien ne le signale.

### Monter par `service(...)`, jamais par `route(...)`

`utoipa-actix-web` ne collecte les chemins **que** depuis `service(...)`. Un endpoint monté par
`route(...)` serait servi sans figurer au contrat : absent du client généré, et invisible pour la
porte P-08.

### `200` sur rejeu, pas `409`

Un client hors ligne qui vide sa file ne doit pas voir d'erreur pour une écriture que le serveur a
déjà acceptée (principe VI). Le corps renvoyé est la ligne **telle qu'elle est en base** : le
serveur fait foi en conflit.

### Aucun détail interne ne franchit la frontière

Ni message PostgreSQL, ni nom de table, ni trace. Le détail part dans les journaux, corrélé par
l'identifiant de requête.

### Le contrat est un produit du code

Après toute modification de handler : `scripts/ci/generer-client.sh`, puis commit du client. La
porte **P-01** fait échouer le build sur tout écart.

---

## Couche 6 — Les tests

Une entité de classe A a **deux tests obligatoires** (`docs/user-stories-v1.md` §0.7), dans la
story qui l'introduit :

| Test | Ce qu'il vérifie |
|---|---|
| **Rejeu** | Trois envois du même identifiant → **un** enregistrement, `201` puis `200`, `200` — **et un seul événement** |
| **Désordre** | Trois écritures dans les **six** ordres → même état final |

Deux précisions qui font la différence entre un test utile et un test décoratif :

- Le **code de statut** fait partie du test de rejeu, pas seulement le décompte de lignes.
- Le test de désordre compare un **ensemble trié**, pas une liste ordonnée : comparer l'ordre
  d'affichage reviendrait à exiger la non-commutativité qu'on cherche à écarter. Et les
  identifiants sont **figés par permutation** — tirés au hasard à chaque envoi, le test
  comparerait des jeux différents et ne dirait rien.

Les tests montent **l'application réelle**, via `kaya_api::routes::configurer`. Un test qui
déclarerait ses propres routes ne prouverait rien du service servi.

---

## La septième couche, et pourquoi elle manque

**Ce cycle ne produit aucun écran. C'est une décision vérifiée, pas une omission.**

L'écran de notes internes n'hérite d'aucun motif :

- il n'apparaît pas parmi les onze codes maquettés de `docs/design/html/` — `C4`, `F2`, `G2`,
  `M4`, `P2`, `Q1`, `R1`, `R4`, `R7`, `S2`, `V1` ;
- la matrice de dérivation `docs/design/derivation.md` n'a aucune ligne pour lui.

« Un écran qui n'hérite d'aucun motif ne se code pas » (principe XII). La couche est reportée au
**cycle ETB**, qui dispose d'écrans réellement maquettés (`G2`, `M4`).

### Ce que le patron ne démontre donc pas — à figer au cycle ETB

| Manque | À figer par |
|---|---|
| **i18n** — clés `fr` et `en`, `fr` par défaut | ETB, premier écran |
| **Mode sombre** — variante `dark:`, jamais une seconde palette | ETB, premier écran |
| **RBAC** — tuiles filtrées par permission, module inactif **absent** et non grisé | ETB |
| **Chargement paresseux par module** | ETB |

Les fondations existent déjà et sont livrées par ce cycle : `app/assets/css/theme.css` (copie
exacte), les catalogues `app/core/i18n/{fr,en}.json` à parité, `app/core/theme/`, et
`PlatformAdapter` avec ses quatre implémentations.

**Conséquence sur la Definition of Done** : le point 8 (« écran vérifié en mode clair et en mode
sombre ») est **sans objet à ce cycle**, au même titre que le point 10 (document imprimé vérifié
sur imprimante thermique). Consigné explicitement, jamais coché en silence.

---

## Pièges de l'outillage, constatés au cycle 001

Ceux-ci ont réellement coûté du temps. Ils ne se devinent pas.

### `sqlx.toml` se résout depuis le crate, pas depuis le workspace

sqlx lit `sqlx.toml` dans `$CARGO_MANIFEST_DIR` — le répertoire du `Cargo.toml` du crate qui
appelle la macro. Posé à la racine du workspace, il est lu par `sqlx-cli` mais **ignoré par
`sqlx::migrate!()`**.

Symptôme trompeur : les deux outils tiennent **deux tables de suivi différentes**. Le CLI inscrit
dans `kaya_migrations._migrations_appliquees`, la macro cherche dans `public._sqlx_migrations`, n'y
trouve rien, et rejoue tout au démarrage — échec sur « relation "tenant" already exists », qui ne
dit rien de la cause.

Le fichier vit donc dans `backend/api/`, et le CLI s'exécute depuis là :

```sh
cd backend/api && cargo sqlx migrate run --source ../migrations
```

### PostgreSQL 18 monte `/var/lib/postgresql`

Plus `/var/lib/postgresql/data`. L'image place les données dans un sous-répertoire nommé d'après
la version majeure. Monter l'ancien chemin fait échouer le conteneur avec un message qui parle de
migration alors que le volume est vide.

### utoipa sans `preserve_order` ni `preserve_path_order`

Sans ces features, utoipa sérialise chemins et schémas **dans l'ordre trié**, donc indépendamment
de l'ordre de découverte des routes. C'est exactement l'exigence n° 2 du gel §3.2 pour la porte
P-01 ; les activer réintroduirait la dépendance à l'ordre de déclaration. Vérifié : un endpoint
ajouté change 33 lignes sur 204.

### Une porte peut mentir en lisant le mauvais contrat

`openapi::contrat()` ne renvoie que le squelette du `#[derive(OpenApi)]` : titre, étiquettes,
schéma d'authentification. **Les chemins sont collectés au montage des routes**, donc seulement
par `split_for_parts()`.

La porte P-08, paramétrée sur le squelette, constatait zéro route et passait au vert avec deux
endpoints servis. Elle consomme désormais `application::contrat_complet()`. **Une porte qui ne
trouve jamais rien est indistinguable d'une porte qui n'a rien à trouver** — d'où le test négatif
obligatoire sur chaque porte.

### Le gel peut être faux, et c'est à l'exécution qu'on l'apprend

Le gel 1.0.3 épinglait `typescript 7.0.2`, dernière stable. `openapi-typescript` 7.13.0 déclare
`peerDependencies: { typescript: "^5.x" }` et TypeScript 7 a modifié l'API `ts.factory` : la
génération échoue immédiatement. Gel corrigé en **1.0.4** avec `5.9.3`.

« Dernière version stable » suppose que les versions sont compatibles entre elles. Le §3.1
vérifiait déjà la compatibilité pour les crates Rust ; le §3.2 ne le faisait pas pour npm.

---

## Le spike `EXCLUDE USING gist` — retour, avant HEB-02

Le cycle 001 est le **premier usage de contrainte d'exclusion du produit**, sur
`fiscalite.exercice_comptable` (`daterange`). Il vaut spike pour HEB-02, qui en dépendra sur
`tstzrange` pour la disponibilité des unités.

| Point vérifié | Constat |
|---|---|
| Extension `btree_gist` | Disponible, et **« trusted »** : `kaya_owner`, propriétaire non superutilisateur, l'installe sans intervention d'un superutilisateur |
| `EXCLUDE USING gist (tenant_id WITH =, daterange(...) WITH &&)` | Accepté, contrainte effective |
| Mapping de type sqlx 0.9 | Validé sur `daterange` ; `PgRange<T>` est présent en 0.9.0 |
| Ordre de pose | Une contrainte d'exclusion ajoutée sur une table **déjà peuplée** échoue sur les données existantes. À poser à la création, comme ici |

**Reste à vérifier avant HEB-02** : le type d'erreur dédié à la violation d'exclusion apporté par
sqlx `#3918` — c'est l'une des deux raisons du choix de la version, et il n'a pas de cible à ce
cycle (aucune écriture concurrente sur `exercice_comptable`).

---

## Écarts au gel introduits par ce cycle

Six crates Rust nécessaires n'étaient pas au gel §3.1. Épinglées exactement, vérifiées sur
`crates.io` le 2026-07-30, **à porter au gel à la revue mensuelle du 2026-08-31** :

| Crate | Version | Pourquoi |
|---|---|---|
| `serde_json` | `1.0.151` | Charge utile `JSONB` de l'outbox |
| `time` | `0.3.54` | `OffsetDateTime`, nommé par le contrat HTTP |
| `thiserror` | `2.0.19` | Types d'erreur de domaine |
| `async-trait` | `0.1.91` | Dyn-compatibilité des traits — `Arc<dyn OutboxWriter>` |
| `futures` | `0.3.33` | Tests de concurrence |
| `dotenvy` | `0.15.7` | Configuration de développement |

**`async-trait` mérite une note.** Rust sait écrire `async fn` dans un trait depuis 1.75, mais un
tel trait n'est pas dyn-compatible. L'injection de dépendances du cadrage §13.2 suppose
`Arc<dyn Trait>` : l'annotation est un choix contraint, pas une habitude reprise d'un exemple.

---

## Reproduire une tranche — la liste

1. **Migration** — identifiant client, horodatages distincts, aucune clé étrangère inter-modules,
   RLS `ENABLE` + `FORCE` + politique `USING`/`WITH CHECK`, privilèges qui reflètent la classe.
2. **Registre** — classe A/B/C/D au §5, entrée au journal §13, **même changement**.
3. **Repository** — macros `query!` littérales, transaction en paramètre, `ON CONFLICT DO NOTHING
   ... RETURNING` pour une classe A.
4. **Service** — transaction ouverte ici, tenant posé, événement outbox **dans la transaction**,
   **jamais sur rejeu**.
5. **Handler** — `#[utoipa::path]` sans chemin ni verbe, attribut de routage Actix, monté par
   `service(...)`, `200` sur rejeu.
6. **Tests** — les deux tests de la classe, sur l'application réelle. Plus le test négatif de
   chaque porte touchée.
7. **Client** — `scripts/ci/generer-client.sh`, commit du diff.
8. **Écran** — seulement s'il hérite d'un motif de `docs/design/html/` ou d'une ligne de
   `docs/design/derivation.md`. Sinon, il ne se code pas.

---

## Voir aussi

- `.specify/memory/constitution.md` — les douze principes, les vingt et une portes
- `docs/registre-classes-offline.md` — classe de chaque entité
- `docs/versions-gelees.md` — versions épinglées et journal des gels
- `specs/001-socle-technique-monorepo/` — spécification, plan, recherche, modèle de données
