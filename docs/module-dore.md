# Kaya — Le module doré

*Patron de référence de tous les cycles. Six couches produites par le cycle 001 (TRX) sur
`note_etablissement` ; la **septième — le patron d'écriture front** — ajoutée après le cycle 002
(ETB) sur la bascule d'un service ; la **huitième — le cycle de vie de l'application** — après la
vérification en navigateur du cycle 003 (CPT), qui a trouvé deux écrans inatteignables alors que
24 portes et 652 tests étaient verts.*

**Version 1.2.0 — 2026-08-01**

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

## Les huit couches

| # | Couche | Fichier |
|---|---|---|
| 1 | Migration | `backend/migrations/0004_note_etablissement.sql` |
| 2 | Registre hors-ligne | `docs/registre-classes-offline.md` §5.1 |
| 3 | Repository | `backend/crates/socle/etablissements/src/note/repository.rs` |
| 4 | Service | `backend/crates/socle/etablissements/src/note/service.rs` |
| 5 | Handler | `backend/api/src/routes/notes.rs` |
| 6 | Tests | `backend/tests/note_etablissement_classe_a.rs` |

La septième — **l'écriture depuis un écran** — n'existait pas au cycle 001, et c'était une décision
vérifiée : l'écran de notes internes n'héritait d'aucun motif maquetté. Elle a été livrée après le
cycle 002, sur une entité qui en avait un. Voir « La septième couche — le patron d'écriture front ».

**Elle ne se lit pas comme les six autres.** Les couches 1 à 6 décrivent des décisions de structure
qui se recopient telles quelles ; la septième décrit des décisions d'**interface**, où la faute type
n'est pas une erreur de compilation mais un grisé posé par réflexe.

**La huitième non plus, et pour une raison de nature différente.** Les sept premières décrivent
comment on écrit **une tranche** ; la huitième décrit ce que l'**application** doit faire au
démarrage — thème, session, coquille — et ne se recopie donc pas par tranche : elle s'écrit une fois
et se vérifie ensuite. Son absence n'a produit aucune erreur de compilation et aucun test rouge :
elle a rendu deux écrans inatteignables. Voir « La huitième couche — le CYCLE DE VIE de
l'application ».

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

### ⚠️ **`cree_le` FAIT AUTORITÉ. `horodatage_client` ne porte AUCUNE règle.**

*Nommé ici au cycle 005, et gardé par la porte **P-23** depuis la constitution 1.8.0. Aucune
colonne n'a été renommée : ce n'est pas le nom qui manquait, c'est l'interdiction vérifiée
d'employer l'autre.*

| Colonne | Ce qu'elle est | Ce qui peut s'y appuyer |
|---|---|---|
| `cree_le` | L'**horodatage d'autorité serveur**, posé par `DEFAULT now()` | **Tout** — durées, taxes, clôtures, tri, pagination |
| `horodatage_client` | L'instant tel que le **terminal** l'a perçu. **Indicatif** | **Rien**, hors trois exemptions limitativement énumérées |

Les trois exemptions sont celles de la constitution, à la lettre : **ordre d'affichage local ·
détection de dérive d'horloge · rendu de l'instant tel que le terminal l'a perçu**. La liste est
**close**.

**Ce qu'on lirait mal, et qui a demandé d'être écrit** : la couche de persistance qui *écrit* la
colonne n'est pas une exemption, et n'a pas à en être une — *écrire une valeur n'est pas s'appuyer
dessus*. Un `INSERT … horodatage_client` ne calcule rien, ne compare rien, ne décide rien. La porte
distingue donc par ce que le code **fait** de la colonne, pas par l'endroit où il se trouve :
`backend/tests/horodatage_autorite.rs` cherche trois formes — arithmétique d'instants, tri ou
filtrage SQL, affectation vers un champ d'autorité — et laisse passer la lecture, l'écriture et la
sérialisation.

**Le motif est daté et chiffré.** Le cadrage §11.4 : « un téléphone d'entrée de gamme dérive et le
personnel change l'heure », et « le passage aggrave la sensibilité à l'horloge » puisqu'il se
facture à l'heure. Une durée de passage calculée sur l'horloge d'un terminal est une facture
fausse ; une clôture qui s'y appuie est fausse au franc près.

**Le pendant du refus est le constat.** La dérive est **signalée, jamais opposée** (FR-036) : une
serveuse dont le téléphone retarde de dix minutes doit pouvoir saisir, et elle ne peut rien y
faire. `socle/synchronisation/src/derive.rs` porte la fonction pure — sur la **valeur absolue** de
l'écart, une horloge en avance étant aussi fausse qu'une horloge en retard — et la couche API la
câble sur le registre des actions, débrayée **par épisode** : deux cents saisies pendant un service
ne produisent qu'une entrée.

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

## La septième couche — le patron d'écriture front

*Ajoutée après le cycle 002 (ETB), sur l'activation et la désactivation d'un service (ETB-02).*
*Étendue de trois points par le cycle 003 (CPT) — voir « Ce que le cycle 003 ajoute au patron ».*

**Une seule opération sur les vingt et une que l'API exposait au cycle 002** — elle en sert **43**
depuis le cycle 003. C'est la leçon de la section
suivante : le cycle 001 a manqué sa couche écran, et le cycle 002 a livré un écran qui **affiche
sans rien écrire** — vingt et une opérations d'écriture testées côté API, aucun bouton qui les
appelle. *Une opération, complète et documentée, vaut mieux que vingt et une approximatives.*

### Pourquoi celle-là

Elle exerce **tout** ce qu'il fallait établir, et rien de plus : formulaire minimal, permission
requise, cas d'erreur métier réel (`desactivation_bloquee`, spécifié et testé côté serveur), refus
hors-ligne d'une opération de **classe C**, et un effet **visible immédiatement** — un service
inactif étant *absent* de l'interface, la réussite se constate sans lire un message.

| Couche | Fichier |
|---|---|
| Appel et refus | `app/modules/etablissements/bascule-service.ts` |
| Écran | `app/modules/etablissements/SectionServices.vue` |
| Champ de saisie | `app/core/design-system/ChampSaisie.vue` — **composant 16**, `docs/design/composants.md` |
| Identifiant client | `app/core/sync/uuid-v7.ts` |
| État réseau | `app/core/platform/reseau.ts`, `courant.ts` |
| Tests | `app/tests/patron-ecriture.spec.ts`, `app/tests/ecran-g1.spec.ts` |

### Les huit points, et celui qu'on écrirait mal

**1 · L'appel passe par le client généré, jamais par un `fetch` écrit à la main.**

```ts
const reponse = await client.PUT(
  '/api/v1/etablissements/{etablissement_id}/services/{module_code}',
  { params: { path: { etablissement_id, module_code } }, body: { id: uuidV7(), actif } },
)
```

Le chemin, la forme du corps et celle de la réponse viennent de `clients/ts/types.gen.ts`, dérivé du
contrat (porte P-01). Renommer un champ côté serveur fait échouer la **compilation du front**, au
lieu de produire un `undefined` que personne ne verrait avant la démonstration.

> **Cette phrase a été fausse pendant deux cycles, et P-01 était verte.** Elle ne l'est que si les
> types consommés **sont** ceux du contrat. Les quatre fichiers d'accès à l'API les redéclaraient
> à la main, puis convertissaient les réponses par `as unknown as` — la seule construction de
> TypeScript qui relie deux types sans rapport. P-01 restait verte : elle compare le client généré
> au client commité, et les deux étaient à jour ; la rupture était un cran plus loin. Vérifié en
> T062 en renommant réellement `CompteVue.nom_affichage` côté serveur — le front compilait.
>
> **Un type consommé s'écrit `components['schemas'][…]`, jamais une interface qui lui ressemble.**
> Une copie fidèle le reste jusqu'au premier champ ajouté d'un côté : le contrat portait déjà
> `ServiceActif.active_le` que la copie n'avait pas, et deux fixtures de test l'omettaient sans
> que rien ne puisse le dire.

**L'identifiant est un UUID v7 généré côté client** (principe VI). `crypto.randomUUID()` produit un
v4 : il rendrait le rejeu idempotent mais casserait l'ordre temporel dont dépend le
`ORDER BY cree_le DESC, id DESC` du repository. Le générateur tient en quinze lignes et n'ajoute
aucune dépendance au gel.

**2 · Le chargement est un squelette, pas un indicateur générique.**

Composant 13 : « occuper la forme exacte de ce qui arrive, pour que rien ne saute ». L'état retenu
porte donc **le sens et la cible** de l'opération, pas un simple booléen :

```ts
const enCours = ref<{ sens: 'ajout' | 'retrait', moduleCode: string } | null>(null)
```

À l'ajout, un squelette de ligne apparaît **en fin de liste**, là où la ligne se posera ; au retrait,
c'est **la ligne concernée** qui devient un squelette. Une roue au milieu de l'écran ne dirait ni
l'un ni l'autre — et sur le réseau d'Abengourou, l'attente dure assez longtemps pour qu'on la
regarde.

**3 · L'erreur serveur est traduite du `code`, jamais du `message`.**

`CorpsErreur` porte trois choses distinctes, et les confondre est la faute :

| Champ | Ce que c'est | Ce qu'on en fait |
|---|---|---|
| `code` | Identifiant stable, jamais traduit | **La clé sur laquelle on branche l'i18n** |
| `message` | Diagnostic pour les journaux | **Jamais affiché** — anglais technique, il nomme des tables |
| `motif_cle` | Clé i18n fournie par le référentiel | **Prime sur le code** : elle enseigne là où le code constate |

La table de correspondance est **explicite et fermée** ; un code inconnu tombe sur une phrase
honnête et générique plutôt que sur une clé i18n affichée en brut. Le rendu est le **composant 07**,
bandeau d'alerte : contrefort de 4 px, fond `-soft`, texte `-fort`, une phrase au passé. **Jamais
deux bandeaux empilés** — d'où une seule variable d'état, pas une liste.

Chaque phrase visible est passée par `docs/design/lexique.md` **avant** d'être codée. Ce cycle y a
ajouté cinq entrées, dont deux qui étaient des pièges : « **Retirer** », jamais « désactiver » (mot
d'interrupteur) ni « supprimer » (ce serait **faux** — la désactivation ne supprime rien), et
« ce service est encore en cours d'utilisation », jamais « obstacle », qui est le nom du trait.

**4 · L'erreur de validation est au champ, pas au bandeau.**

```ts
if (!moduleChoisi.value) { erreurChamp.value = 'champ.erreur.obligatoire'; return }
```

Elle porte sur ce qui est saisi : le message doit être **à côté de l'endroit où l'on corrige**. Le
composant 16 la rend avec **trois signaux, jamais la couleur seule** — bordure `danger`, message,
et icône d'avertissement dans ce message.

**5 · Permission absente : l'action est ABSENTE, pas désactivée.**

```ts
const peutModifier = computed(() => detient(props.permissions, PERMISSION_BASCULER))
```

Aucun `disabled`, aucun `title` explicatif, **rien dans le HTML rendu** — et c'est ce que le test
vérifie, pas la valeur du booléen. Le grisé est le réflexe naturel, et c'est celui que le principe
VII interdit : il apprend à l'utilisateur, à chaque écran et tous les jours, qu'une partie du
produit lui est refusée.

**6 · CLASSE C : le refus précède l'appel, et il s'explique.** *C'est le point qu'on écrirait mal.*

`etablissement_module` est de classe C au registre. Deux fautes symétriques guettent, et la seconde
est la plus tentante :

- **griser le bouton** — l'utilisateur ne sait pas pourquoi, et l'apprend en cliquant dans le vide ;
- **mettre en file « au cas où »** — promettre un envoi qu'on ne sait pas rejouer. Une opération de
  classe C n'a **aucune** garantie de rejeu ; la file la rejouerait sans que rien ne le déduplique.

Le patron fait la troisième chose : hors ligne, **l'action disparaît et un bandeau dit pourquoi**,
en une phrase, immédiatement. La garde vit dans `basculerService`, **pas dans le composant** : un
second appelant oublierait de la reposer, et la faute ne se verrait qu'en clientèle.

Deux subtilités écrites une fois pour toutes :

- `navigator.onLine` dit qu'une **interface réseau est active**, pas que le serveur répond. À
  Abengourou, une 3G qui affiche « en ligne » sans porter la moindre requête est le cas courant.
  **La garde hors-ligne ne dispense donc pas du traitement d'erreur** — elle évite l'attente
  inutile, elle ne la remplace pas.
- L'état `degrade` est traité **comme** hors ligne pour une opération de classe C. Personne ne le
  produit encore ; le cycle SYN l'alimentera depuis les échecs réels de requête.

**7 · Le rafraîchissement relit le serveur, sans rechargement de page.**

```ts
emit('services-changes', await chargerServices(props.contexte, props.etablissementId))
```

Trois décisions dans cette ligne. **Une seule requête**, pas les cinq de `chargerEcran` : l'identité
et les points de vente n'ont pas bougé. **La liste vient du serveur**, jamais reconstruite à la main
côté client — le serveur fait foi en conflit (principe VI). Et **elle suit le succès**, elle ne
l'accompagne pas : relire avant que le serveur ait tranché afficherait l'état d'avant en donnant
l'impression qu'il s'agit de celui d'après.

Le corps rendu par le `PUT` n'est **pas** relu, et c'est délibéré : une désactivation le rend absent
de la liste des actifs. C'est exact, c'est même l'effet à montrer, mais seule la liste entière donne
ce que l'écran doit afficher **dans les deux sens**.

**8 · Clair et sombre, par la variante `dark:` uniquement — c'est-à-dire par personne ici.**

Aucun composant de ce patron ne porte de classe `dark:`. Les noms de jetons sont identiques dans les
deux thèmes et seules les valeurs changent sous `.dark` : `bg-danger-soft text-danger-fort` bascule
tout seul. `app/tests/theme-sombre.spec.ts` le vérifie mécaniquement — chaque jeton employé a bien
une valeur sous `.dark` — et **couvre désormais `core/design-system/`** en plus des sections, parce
que le composant 16 est la pièce dont un défaut se propagerait le plus loin.

### Ce que le cycle 003 (CPT) ajoute au patron

Trois points, qui ne se déduisaient d'aucun des huit précédents. Ils s'appliquent à toute opération
qui touche à l'identité, c'est-à-dire à la plupart de celles qui restent.

**9 · La garde de permission vit côté serveur, et l'action est ABSENTE côté client.**

Les deux, jamais l'un sans l'autre — et ce ne sont pas deux implémentations de la même règle :

```rust
// backend/api/src/securite.rs — la garde qui fait autorité.
exiger(&contexte, "comptes.attribuer_role")?;
exiger_ou_soi(&contexte, "comptes.changer_mot_de_passe", compte_id)?;
```

Le serveur refuse ; le client, lui, **ne rend pas le bouton**. « Un module inactif est absent,
jamais grisé » (principe VII) vaut aussi pour une action : un bouton grisé apprend à qui ne peut
pas agir ce qu'il pourrait faire ailleurs, et invite à chercher comment. `core/rbac` lit
`sessionCourante()?.permissions` — l'union rendue par `session_ouvrir`, **jamais le jeton décodé**
(research R-06) : deux sources pour la même information, et une seule fait autorité.

La vérification côté écran porte donc sur le **HTML rendu**, pas sur un attribut `disabled` :
`app/tests/permissions.spec.ts` cherche l'absence de la chaîne, ce qu'un test de `:disabled`
laisserait passer.

`exiger_ou_soi` est le cas qu'on écrirait mal : changer *son propre* mot de passe ne demande aucune
permission, changer celui d'un autre en demande une. Une seule garde porte les deux, sans quoi le
chemin « soi » finirait par se passer de garde.

**10 · Un secret durable passe par `PlatformAdapter`, et le type DÉCLARE la garantie.**

Le jeton de rafraîchissement va au Keystore/Keychain, jamais dans un stockage web ordinaire. Mais
toute plateforme n'en a pas, et le silence sur ce point est ce qui produit une session qu'on croit
persistante :

```ts
// Le stockage web l'annonce dans son type : il n'y a pas de contresens possible à l'appel.
const persistante = await rangerRafraichissement(vue.rafraichissement)
return { issue: 'succes', session, persistante }
```

`persistante: false` n'est pas une erreur — c'est un fait que l'écran **dit**, plutôt que de laisser
découvrir une déconnexion inexpliquée une heure plus tard. La garantie est portée par le type de
retour, donc impossible à ignorer par distraction.

**11 · Rafraîchir AVANT de vider la file, jamais l'inverse.**

`core/sync/vidage.ts`. L'ordre paraît indifférent et ne l'est pas : une file de classe A vidée avec
un jeton expiré part en `401` opération par opération, et chaque échec est indistinguable d'un refus
métier. L'inversion est **exercée** par un test, pas seulement commentée — c'est la seule façon de
la garder, un ordre correct par accident se rétablit à la première refonte.

### La huitième couche — le CYCLE DE VIE de l'application

*Ajoutée après la vérification en navigateur du cycle 003, qui a trouvé deux des quatre écrans du
produit inatteignables alors que 24 portes et 652 tests étaient verts.*

**Les sept premières couches décrivent comment on écrit une tranche. Aucune ne dit comment
l'application démarre** — et c'est ce trou qui a produit le défaut. `app/app.vue` faisait vingt-trois
lignes et ne contenait que `<NuxtPage />` ; il n'existait ni `app/plugins/` ni `app/layouts/`.
Chaque page amorçait pour elle-même ce qu'elle avait pensé à amorcer, et **cinq sur six avaient
oublié la reprise de session**.

Les trois symptômes observés n'en faisaient qu'un :

| Symptôme | Ce qui manquait |
|---|---|
| `TypeError: Cannot read properties of null (reading 'parentNode')` à la navigation | une **racine stable** — donc un layout |
| Un chargement direct d'adresse ne reprend jamais la session | un **middleware global** |
| La classe `.dark` n'est jamais appliquée | un **plugin** |

#### Le découpage, et pourquoi chaque pièce est à cet endroit-là

| Pièce | Fichier | Pourquoi pas ailleurs |
|---|---|---|
| **Thème** | `plugins/01.theme.client.ts` | Nuxt résout **tous** les plugins avant `vueApp.mount()`. Un `onMounted` dans une page n'amorcerait que cette page — c'est le défaut, pas le remède. Suffixe `.client` obligatoire : le module touche `document` et `localStorage`. |
| **Anti-scintillement** | script en ligne dans `app.head` de `nuxt.config.ts` | En SPA, la coquille est peinte avec `body { background-color }` **avant** que le moindre module ne s'exécute. Même en `enforce: 'pre'`, un plugin arrive après le premier pixel : l'utilisateur en mode sombre verrait un éclair blanc à chaque ouverture. En ligne, jamais un hôte externe — P-21 intacte. |
| **Reprise de session** | `middleware/01.session.global.ts` | Un middleware global s'exécute avant **chaque** navigation, la première comprise. C'est la seule place qui couvre les six routes sans être recopiée six fois — et la recopie était la faute. |
| **Coquille** | `layouts/default.vue` | Une racine stable, **un seul `<main>`**. Une page nouvelle en hérite sans rien écrire, et ne peut pas l'oublier. |
| **Sortie de session** | `layouts/default.vue`, pied de coquille | Même raison que la reprise : dans un écran, chaque écran suivant devrait s'en souvenir. C'est un **pied** et non un en-tête parce que `R1` et `G1` portent déjà deux `<header>` différents et que `G3`/`G4` n'en ont aucun — se poser au-dessus les rouvrirait tous les trois, ce que `derivation.md` refuse. |

**Trois pièges qui coûtent une heure chacun, écrits une fois pour toutes :**

- **`<NuxtLayout>` n'est jamais la racine.** Il rend son `<slot>` dans un `<Transition>` ; le poser
  en racine reproduit la famille de défauts qu'on vient de fermer. Le `<div>` d'`app.vue` l'enveloppe.
- **Le middleware s'exécute bien à la première navigation en SPA, mais par un détour.** Le
  `router.beforeEach` qui porte les middlewares est posé **après** `router.isReady()` : la première
  résolution lui échappe, et Nuxt rejoue la route initiale au hook `app:created` avec `force: true`.
  Corollaire utile : un middleware s'exécute toujours **après** tous les plugins.
- **`return navigateTo(...)`, jamais `abortNavigation()` ni `return false`.** Sur la navigation
  initiale d'une application sans rendu serveur, ces deux-là n'annulent pas : ils affichent
  « Page Not Found ». Un utilisateur dont la session a expiré verrait une 404 au lieu de `R0`.

#### La cause du `parentNode`, isolée par expérience

Elle a été **établie**, pas supposée — quatre pages sondes, une variable changée à la fois :

| Racine du template | Composant | Bascule après montage | Erreur |
|---|---|---|---|
| fragment | paresseux | non | — |
| fragment | **paresseux** | **oui** | **`parentNode`** |
| fragment | synchrone | oui | — |
| **élément unique** | paresseux | oui | — |

**Il faut les trois conditions réunies.** Une racine multiple compile en *fragment* ; un fragment
dont la branche active devient un `defineAsyncComponent` non encore résolu a un `el` **nul**, et au
rendu suivant Vue appelle `hostParentNode(prevTree.el)`.

**Une racine unique suffit à l'éliminer, et le chargement paresseux reste intact** — le principe VII
l'exige module par module : « un serveur de salle ne télécharge pas le code du back-office ». La
règle opposable qui en découle : **une page a une seule racine, et c'est un élément, jamais un
`v-if`/`v-else` de premier niveau.**

#### La règle qui vaut au-delà de ce défaut

> **Une unité écrite n'est ni testée ni branchée par défaut, et il faut un contrôle pour chacune
> des deux propriétés.**

`initialiserTheme()` a vécu deux cycles exportée, documentée « à appeler au démarrage » — et appelée
nulle part. Elle n'était pas même testée : `theme-sombre.spec.ts` lit les jetons de `theme.css`, il
n'importe pas `core/theme`. Le balayage qui l'a trouvée a trouvé **quatre autres** points d'entrée
sans appelant : `FileLocale`, `marquerClasseA`, `viderFile` et `operationRealisable` — la file
hors-ligne entière — plus `fermerSession`, qu'aucun bouton n'atteignait.

`app/tests/amorcage.spec.ts` porte désormais les **douze points d'entrée** à deux états, `branché`
et `dû`, et vérifie **les deux versants** : un `dû` qui acquiert un appelant fait échouer le build,
un `branché` qui le perd aussi. Sans le second versant, tout déclarer branché rendrait le harnais
muet.

**Le harnais a servi une première fois au lot suivant, et c'est ce qu'on lui demandait.**
`fermerSession` est passée de **due** à **branchée** — le bouton « passer la main » du pied de
coquille l'appelle — et le changement n'a pas pu se faire en silence : tant que sa ligne restait
« due », le versant négatif échouait. Deux entrées sont nées dans le même mouvement,
`ecrituresEnAttente` branchée et `brancherFile` due par SYN-01. **Une ligne qui bouge est une
décision qu'on relit ; c'est toute la valeur de la liste.**

#### Ce que la coquille porte, et ce qu'elle ne porte pas

Elle porte, **depuis le lot de déconnexion** : une racine stable, un `<main>` unique, et le pied qui
permet de **quitter son poste** — un bouton discret (composant 03), absent hors session, qui refuse
si des écritures ne sont pas parties, révoque côté serveur, purge le stockage et renvoie sur `R0`.
Le libellé passe par `docs/design/lexique.md` : « **passer la main** », jamais « se déconnecter » —
sur un terminal de comptoir, l'appareil ne bouge pas, c'est la personne qui change, et c'est ce que
le journal d'audit doit refléter.

Elle ne porte ni barre de contexte unifiée, ni témoin de synchronisation. Ce ne sont pas des oublis :

- `EcranAccueil` et `EcranEtablissement` portent chacun un `<header>` **différent**. Les fondre est
  un changement d'écran, et `docs/design/derivation.md` est opposable. **C'est la raison pour
  laquelle la sortie de session est un pied et non un en-tête** : elle se pose sous les trois
  formes d'écran sans en rouvrir aucune.
- Le témoin de synchronisation est le **composant 10**, toujours dû à ETB-06. Prêt n'est pas
  construit (principe X).

### Ce que ce patron ne démontre PAS

Écrit ici pour que le cycle suivant ne le suppose pas acquis :

| Manque | À figer par |
|---|---|
| **La fraîcheur affichée et le cache local** | SYN-01/02 |
| **Le témoin de synchronisation permanent** (composant 10) | ETB-06 |
| **L'état `degrade`** — personne ne le produit | SYN |
| **Le bandeau d'annulation** (composant 14) — aucune action de ce patron n'est destructrice | Premier cycle qui en a une |
| **La sélection réelle de plateforme** — `adaptateurCourant()` renvoie le web | Construction de la coquille Tauri |

Et les deux que le cycle 003 a soldés, gardés ici pour qu'on ne les recherche pas :

| Manque relevé au cycle 002 | Soldé par |
|---|---|
| **Le RBAC réel** — permissions en configuration, provisoire nommé | CPT-02 · `core/rbac` lit `sessionCourante()?.permissions` ; `runtimeConfig` n'a plus ni `permissions`, ni `tenantId`, ni `compteId` |
| **L'authentification** — contexte encore par deux en-têtes | CPT-01 · `ContexteAppel` porte un jeton, `enTetesAuth` rend **un seul** en-tête, et `x-kaya-tenant` / `x-kaya-compte` ne sont plus acceptés |

---

## La septième couche au cycle 001, et pourquoi elle manquait alors

**Le cycle 001 ne produit aucun écran. C'est une décision vérifiée, pas une omission.**

L'écran de notes internes n'hérite d'aucun motif :

- il n'apparaît pas parmi les onze codes maquettés de `docs/design/html/` — `C4`, `F2`, `G2`,
  `M4`, `P2`, `Q1`, `R1`, `R4`, `R7`, `S2`, `V1` ;
- la matrice de dérivation `docs/design/derivation.md` n'a aucune ligne pour lui.

« Un écran qui n'hérite d'aucun motif ne se code pas » (principe XII). La couche est reportée au
**cycle ETB**, qui dispose d'écrans réellement maquettés (`G2`, `M4`).

### Ce que le patron ne démontrait pas alors — et par quoi c'est soldé

| Manque relevé au cycle 001 | Soldé par |
|---|---|
| **i18n** — clés `fr` et `en`, `fr` par défaut | ETB, écran `G1` |
| **Mode sombre** — variante `dark:`, jamais une seconde palette | ETB, écran `G1` + `app/tests/theme-sombre.spec.ts` |
| **Chargement paresseux par module** | ETB, `pages/etablissement.vue` par `defineAsyncComponent` |
| **RBAC** — module inactif **absent** et non grisé | ETB pour l'affichage ; **la couche d'écriture** pour les *actions* — voir la section précédente, point 5 |
| **L'écriture depuis un écran** — non relevé alors, et c'est ce qui manquait le plus | La septième couche, ci-dessus |

Les fondations existaient déjà au cycle 001 : `app/assets/css/theme.css` (copie exacte), les
catalogues `app/core/i18n/{fr,en}.json` à parité, `app/core/theme/`, et `PlatformAdapter` avec ses
quatre implémentations.

**Conséquence sur la Definition of Done** : le point 8 (« écran vérifié en mode clair et en mode
sombre ») était **sans objet au cycle 001**, au même titre que le point 10 (document imprimé vérifié
sur imprimante thermique). Consigné explicitement, jamais coché en silence. Le point 8 est exigible
depuis `G1` ; le point 10 le reste jusqu'à IMP.

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

### ✅ Retour du cycle 004 — la vérification promise est faite

Le cycle 001 laissait un point ouvert : « le type d'erreur dédié à la violation d'exclusion
apporté par sqlx `#3918` — c'est l'une des deux raisons du choix de la version, et il n'a pas de
cible à ce cycle ». HEB-02 lui en a donné une, sur `tstzrange` et en concurrence réelle.

| Point | Constat du cycle 004 |
|---|---|
| `ErrorKind::ExclusionViolation` | **Existe** en 0.9.0. C'est bien l'apport de `#3918`, et il sert |
| `DatabaseError::is_exclusion_violation()` | **N'existe pas.** Le trait porte `is_unique_violation()`, `is_foreign_key_violation()` et `is_check_violation()`, et s'arrête là — vérifié dans `sqlx-core` 0.9.0, `src/error.rs` |
| `PgRange<OffsetDateTime>` ↔ `TSTZ_RANGE` | **Validé**, et exercé sur du code de production (`occupation/repository.rs`) |
| Concurrence réelle sur `tstzrange` | **Exercée** — deux transactions distinctes, une seule réussit, et le test asserte la **cause** du refus |

**L'apport est donc partiel, et c'est ce qu'il faut savoir avant d'écrire la ligne.** Écrire
`e.is_exclusion_violation()` par analogie avec les trois autres accesseurs **ne compile pas**, et
l'erreur se cherche une demi-heure parce qu'on y suspecte une faute de frappe. La forme correcte
est un `matches!` sur `e.kind()` — `ErrorKind` étant `#[non_exhaustive]`, un `match` exhaustif ne
compilerait pas davantage, et un `match` avec bras `_` cesserait de signaler l'arrivée d'un genre
nouveau. `crates/verticales/hebergement/src/erreurs.rs` porte la forme retenue et ses cinq tests.

**Le nom de la contrainte est vérifié en plus du genre.** Une table qui gagnerait une seconde
contrainte d'exclusion ferait autrement passer ses violations pour des doubles attributions, et
l'écran afficherait « Cette chambre est déjà prise sur cette période » pour un refus sans rapport.

**Ce que ce retour change pour `docs/versions-gelees.md`, sans que ce document y touche** : le
**point ouvert de son en-tête** — « le choix de sqlx `0.9.0` doit être confirmé par le spike » —
peut être retiré, la confirmation étant acquise. Et la note du §2 gagnerait sa **limite** : dire
que `#3918` apporte « une erreur de violation d'exclusion » est exact et incomplet, puisqu'il
n'apporte pas l'accesseur. C'est la **revue mensuelle du 2026-08-31** qui tranche ; rien n'est
modifié ici, un gel ne se corrige pas au fil de l'eau.

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

- `.specify/memory/constitution.md` — les douze principes, les **vingt-cinq** portes (P-01 à P-22,
  dont P-01b, P-05b et P-21b ; P-22 est née de la vérification en navigateur du cycle 003)
- `docs/registre-classes-offline.md` — classe de chaque entité
- `docs/versions-gelees.md` — versions épinglées et journal des gels
- `specs/001-socle-technique-monorepo/` — spécification, plan, recherche, modèle de données
