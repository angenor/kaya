# Phase 0 — Recherche et décisions techniques

**Cycle** : 001 — Socle technique du monorepo Kaya
**Date** : 2026-07-30
**Spec** : [spec.md](./spec.md)

> **Règle appliquée à tout ce document** : `docs/versions-gelees.md` v1.0.2 fait foi. Aucun numéro
> de version n'est proposé, ni de mémoire, ni par revérification. Les points où le gel est
> **incomplet** sont signalés en R-14 et remontés à la revue mensuelle — ils ne sont pas comblés
> ici.

---

## R-01 — `mold` n'existe pas sur macOS. L'exigence doit être scindée.

**Décision** : `mold` est configuré pour **les cibles Linux uniquement** — image Docker de
construction et intégration continue. Sur le poste de développement macOS Apple Silicon, on
conserve **le linker par défaut d'Apple** (`ld_prime`, celui livré depuis Xcode 15).

**Rationale** : le projet cible `linux/amd64` en production mais se développe sur `darwin/arm64`
(cf. environnement de la session). `mold` est un linker **ELF** : il ne connaît pas le format
Mach-O ni les options du linker Apple (`-dynamic`), et son auteur a explicitement fermé la
demande de support macOS. Le tenter sur le poste de développement produit un échec de lien, pas
un gain. La consigne « linker mold » du cycle reste donc **tenue là où elle a un sens** — la
chaîne Linux — et n'est pas simulée sur macOS.

Ce point satisfait aussi la contrainte du prompt de planification : *« toute dépendance native,
tout plugin et tout outil que ce module ajoute doit exister pour les DEUX architectures »*. `mold`
n'existe pas pour les deux : il est donc cantonné à celle où il existe, au lieu d'être imposé
partout.

**Alternatives écartées** :

| Option | Écartée parce que |
|---|---|
| `mold` partout | Impossible techniquement sur macOS — échec de lien, pas ralentissement |
| `sold` (portage Mach-O de mold) | Produit commercial séparé, hors du gel, une dépendance payante sur le chemin de build d'un développeur solo |
| `lld` sur macOS | Intégration Mach-O fragile ; le linker Apple depuis Xcode 15 est déjà nettement plus rapide que l'ancien `ld64` |
| Renoncer à l'exigence | La CI et l'image de production sont Linux : c'est là que le gain est capitalisable et mesurable |

**Sources** : [rui314/mold #1171 — « Is macos supported »](https://github.com/rui314/mold/issues/1171) ·
[Rust Project Primer — Linking](https://rustprojectprimer.com/building/linker.html)

**Conséquence sur SC-010** : la mesure avant/après des optimisations est faite **dans le
conteneur Linux**, pas sur le poste. Une mesure macOS ne prédit rien de la CI.

---

## R-02 — `sccache` et `line-tables-only` s'appliquent, eux, aux deux architectures

**Décision** : `sccache` activé sur les deux plateformes ; `debug = "line-tables-only"` dans le
profil `dev` du workspace.

**Rationale** : `sccache` est distribué pour `darwin/arm64` et `linux/amd64` — il satisfait la
contrainte bi-architecture là où `mold` échoue. `line-tables-only` réduit le volume d'informations
de débogage tout en conservant les numéros de ligne dans les traces de panique, ce qui est
exactement le compromis voulu pour un profil de développement.

**À noter pour `/speckit-tasks`** : `sccache` n'est pas dans `docs/versions-gelees.md`. C'est un
outil de poste, pas une dépendance compilée dans le binaire — il n'entre pas dans un lockfile et
la porte P-20 ne le couvre pas. Sa version est donc libre, mais l'image Docker de CI l'épingle
pour la reproductibilité.

---

## R-03 — Poser le tenant courant sans jamais toucher à `AssertSqlSafe`

**Décision** : le tenant courant est posé par une requête **paramétrée** en début de chaque
transaction :

```
SELECT set_config('app.current_tenant', $1, true)
```

Le troisième argument `true` donne exactement la sémantique de `SET LOCAL` : la valeur retombe à
la fin de la transaction.

**Rationale** : c'est le point où l'on se serait naturellement trompé. `SET LOCAL
app.current_tenant = ...` **n'accepte pas de paramètre lié** — il faudrait interpoler
l'identifiant dans la chaîne SQL, donc passer par `sqlx::raw_sql` et l'envelopper dans
`sqlx::AssertSqlSafe` (obligatoire en sqlx 0.9 sur toute requête non littérale, changement
`#3723`). On aurait alors une concaténation de chaîne SQL sur le chemin le plus sensible du
produit — celui qui décide quelles lignes un client voit. `set_config()` est une fonction : son
argument se lie normalement, la requête reste littérale, la macro `query!` la vérifie à la
compilation, et `AssertSqlSafe` devient inutile ici.

**Conséquence** : `AssertSqlSafe` n'apparaît dans ce cycle **qu'aux endroits où le SQL est
réellement dynamique et sans donnée utilisateur** — le test de balayage RLS (R-06) et la
construction des schémas de test. Chacun de ces usages est commenté sur place avec la raison pour
laquelle l'assertion est légitime.

**Source** : [docs.rs/sqlx/0.9.0 — `AssertSqlSafe`](https://docs.rs/sqlx/0.9.0/sqlx/struct.AssertSqlSafe.html)
(struct `sqlx::AssertSqlSafe`, requise pour `raw_sql()` et toute requête non littérale).

---

## R-04 — Trois rôles PostgreSQL distincts, pas deux

**Décision** :

| Rôle | Droits | Usage |
|---|---|---|
| `kaya_owner` | Propriétaire des tables, exécute les migrations | Migrations seulement |
| `kaya_app` | `SELECT/INSERT/UPDATE/DELETE` selon la table, **soumis à la RLS** | Runtime de l'API |
| `kaya_ledger_reader` | `SELECT` sur **`synchronisation.evenement_outbox` uniquement** | Test de reconstitution autonome (FR-042) |

**Rationale** : la constitution (principe III) exige deux rôles — propriétaire et applicatif. Le
troisième est ce qui rend le test de reconstitution autonome **démontrable au lieu d'être
déclaratif**. Un test qui « n'interroge pas les autres tables » par discipline de rédaction ne
prouve rien : il suffit d'un `JOIN` ajouté six mois plus tard pour que la garantie disparaisse
sans que rien n'échoue. Un rôle qui n'a pas le droit de lire les autres tables fait échouer ce
`JOIN` immédiatement.

`FORCE ROW LEVEL SECURITY` est indispensable : sans lui, `kaya_owner` contournerait toutes les
politiques, et le jour où une tâche de maintenance tournerait sous ce rôle, l'isolation
tomberait silencieusement.

---

## R-05 — L'immuabilité de l'outbox se pose à trois niveaux

**Décision** : défense en profondeur, dans cet ordre :

1. **`REVOKE UPDATE, DELETE ON synchronisation.evenement_outbox FROM kaya_app`** — le rôle du
   runtime n'a physiquement pas le droit de modifier.
2. **Déclencheur `BEFORE UPDATE OR DELETE`** qui lève une exception, sauf pour la seule colonne
   `publie_le` passant de `NULL` à une valeur. Il s'applique **y compris au propriétaire**.
3. **Aucun chemin de code** de suppression, vérifié par une porte de CI (P-05b, cf. plan.md).

**Rationale** : les trois couches couvrent trois fautes différentes. Le `REVOKE` arrête le bug
applicatif. Le déclencheur arrête la migration ou le script de maintenance lancé sous
`kaya_owner` — le cas réel, celui d'un développeur solo qui se connecte en production à 23 h pour
« corriger une ligne ». La porte de CI arrête le code qui aurait été écrit pour purger.

**Le point subtil** : le marquage « publié » est une écriture, donc formellement une mutation.
C'est la seule exception, et elle est **monotone et non réversible** (`NULL → timestamp`, jamais
l'inverse). Le déclencheur l'autorise explicitement et refuse toute autre différence entre
l'ancienne et la nouvelle ligne. Sans cette exception, il faudrait une seconde table de marquage
— une jointure de plus sur le chemin du grand livre, exactement ce que TRX-02 cherche à éviter.

---

## R-06 — Charge utile : `JSONB` versionné, avec colonnes de tête typées

**Décision** : `evenement_outbox` porte des colonnes typées pour tout ce qui sert à **retrouver**
un événement, et un `JSONB` pour tout ce qui sert à **le reconstituer**. Le `JSONB` porte un
champ obligatoire `version_schema`.

**Rationale** : c'est l'arbitrage central du cycle. Des colonnes typées pour toute la charge
utile financière seraient ingérables — chaque nouveau type d'événement ajouterait des colonnes
nullables, et la table finirait en centaines de colonnes vides. Du `JSONB` pur rendrait
impossible tout index de recherche et toute contrainte.

Le vrai risque du `JSONB` est ailleurs : **un document sans version est illisible dans dix ans**.
En phase 2, la génération SYSCOHADA rétroactive relira des événements écrits par des versions du
code qui n'existent plus. `version_schema` est ce qui permettra d'écrire un décodeur par
génération de format au lieu de deviner. Il coûte un entier aujourd'hui et vaut la totalité de la
provision §14.7.

**Alternatives écartées** : colonnes typées exhaustives (ingérable) · `JSONB` sans version
(illisible rétroactivement) · table par type d'événement (multiplie les schémas, contredit
« un schéma par module »).

---

## R-07 — Séquence monotone par établissement

**Décision** : `evenement_outbox` porte `sequence_etablissement BIGINT NOT NULL`, unique par
`(etablissement_id, sequence_etablissement)`, alimentée par une séquence dédiée.

**Rationale** : le cadrage §11.5.2 exige un « journal d'événements append-only par établissement,
à séquence monotone ». Une séquence globale ne le donne pas : deux établissements se
partageraient les numéros, et un trou dans la suite d'un établissement serait indistinguable d'un
événement écrit par l'autre. La monotonie par établissement est ce qui permettra à un nœud de
site (mode C, incrément 3) de détecter qu'il lui manque un événement.

**Attention identifiée** : les séquences PostgreSQL ne sont pas transactionnelles — un rollback
laisse un trou. C'est **acceptable et voulu** : la séquence garantit l'ordre et la détection de
manque, pas l'absence de trou. Le contraire imposerait un verrou par établissement sur le chemin
d'écriture le plus chaud du produit. Ce choix est documenté dans `docs/module-dore.md` pour que
personne ne « corrige » plus tard un trou qui n'est pas un bug.

---

## R-08 — Worker de publication : `FOR UPDATE SKIP LOCKED`, in-process

**Décision** : une tâche asynchrone in-process, réveillée périodiquement, qui lit un lot avec
`SELECT ... WHERE publie_le IS NULL ORDER BY id FOR UPDATE SKIP LOCKED LIMIT n`, remet la charge
aux consommateurs, puis marque `publie_le`.

**Rationale** : `SKIP LOCKED` permet à plusieurs instances d'API de tourner sans se marcher
dessus ni introduire de verrou distribué — ce qui compte dès qu'un second conteneur est démarré.
Aucune file externe n'est introduite (contrainte du cadrage §13.2). Redis n'est **pas** utilisé
ici : il ne porte que de l'éphémère reconstructible (principe II), et une file de publication qui
perdrait son état ferait perdre des événements.

**Idempotence des consommateurs** : chaque consommateur maintient sa propre trace de dernier
événement traité. Un événement republié après un redémarrage brutal produit donc l'effet d'une
seule présentation — c'est ce que vérifie le test de redémarrage (SC-004).

---

## R-09 — Détection des tables sans RLS : requête catalogue, pas convention

**Décision** : un test d'intégration interroge le catalogue PostgreSQL et échoue si une table
d'un schéma applicatif n'a pas `relrowsecurity` **et** `relforcerowsecurity` **et** au moins une
politique dans `pg_policies`.

**Rationale** : la porte P-07 doit constater l'état réel de la base après migration, pas relire
les fichiers de migration. Un `ALTER TABLE ... DISABLE ROW LEVEL SECURITY` ajouté dans une
migration ultérieure passerait une analyse de texte et serait attrapé par une lecture du
catalogue. Les trois conditions sont vérifiées séparément : `ENABLE` sans `FORCE` laisse le
propriétaire hors politique, et `ENABLE FORCE` sans politique bloque tout au lieu d'isoler —
deux échecs distincts, deux messages distincts.

**Liste d'exclusion explicite** : les tables de migration de sqlx. Elle est **nommée dans le
test**, jamais dérivée d'un motif de nom — un motif laisserait passer toute table future qui s'y
conformerait par accident.

---

## R-10 — Porte du registre des classes hors-ligne

**Décision** : un test compare l'ensemble des tables des schémas applicatifs
(`information_schema.tables`) à l'ensemble des entités déclarées dans
`docs/registre-classes-offline.md`, et échoue sur toute table absente du registre.

**Rationale** : le registre est un document Markdown tabulaire dont la colonne « Entité ou
opération » porte les noms d'entités en `code`. L'extraction est donc mécanique. Le sens de la
comparaison est **table → registre** : une entité déclarée mais pas encore implémentée est
normale (le registre décrit tout le produit, pas seulement ce qui est construit) ; une table sans
déclaration est l'erreur à attraper.

**Limite acceptée et consignée** : le registre classe des **opérations**, pas seulement des
tables — `encaissement` y figure deux fois, en B espèces et en D Mobile Money. La porte vérifie
donc la **présence** d'une entité, pas la justesse de sa classe. La justesse reste humaine, revue
mensuellement (constitution, § Revue). Prétendre l'automatiser produirait une porte qui ment.

---

## R-11 — Test de reconstitution autonome : le mécanisme exact

**Décision** : le test se connecte avec `kaya_ledger_reader` (R-04), lit les événements d'un jeu
financier seedé, reconstruit chaque opération depuis la seule charge utile, et compare au résultat
attendu. Toute tentative de lecture d'une autre table lève une erreur de permission PostgreSQL,
qui fait échouer le test.

**Rationale** : c'est la traduction exécutable de FR-042. Le test ne prouve pas que le code
*évite* les autres tables ; il rend l'accès *impossible*. La différence est la seule chose qui
tiendra sur dix ans de cycles.

**Jeu de cas** : un encaissement complet, avec montant en unités mineures, mode de règlement,
contrepartie, ventilation de taxes et référence de document. Ce jeu est figé et versionné avec le
test — il devient le cas doré du grand livre, au même titre que les jeux fiscaux du principe V.

---

## R-12 — Migrations au démarrage

**Décision** : le binaire d'API appelle `sqlx::migrate!()` au démarrage, sous le rôle
`kaya_owner`, avant d'ouvrir le port d'écoute.

**Rationale** : décision de l'utilisateur, confirmée en clarification, alignée sur le cadrage
§10.2 (« migrations automatiques et idempotentes au démarrage »). Rétrofitter cela sur un socle de
migrations déjà écrit coûte cher ; le poser au premier cycle ne coûte rien.

**Deux conséquences à ne pas manquer** :

- Le pool de runtime tourne sous `kaya_app`, **pas** sous le rôle de migration. Deux
  configurations de connexion distinctes dans le binaire, pas une seule avec des droits élargis —
  sinon `FORCE ROW LEVEL SECURITY` perd son intérêt.
- Deux instances démarrant simultanément appliqueraient les migrations en concurrence. sqlx pose
  un verrou consultatif pour cela ; le comportement est vérifié par un test de démarrage
  concurrent plutôt que supposé.

---

## R-13 — Sauvegardes : stockage tiers avec verrouillage d'objet

**Décision** : `pg_dump` quotidien, chiffré avant transfert, poussé vers un **stockage objet tiers
sur un hôte distinct du serveur de production**, avec verrouillage d'objet et rétention
verrouillée. Garage reçoit une copie de travail ; il ne porte **jamais** l'immutabilité.

**Rationale** : Garage tourne sur le même VPS que la base. Un attaquant qui obtient le serveur
obtient les deux. TRX-04 le dit explicitement et c'est la seule ligne du cycle qui protège contre
une compromission plutôt que contre une panne.

**Reste à arrêter en `/speckit-tasks`** : le fournisseur exact. L'invariant — hôte distinct +
verrouillage d'objet + rétention verrouillée — est arrêté ; le nom du fournisseur ne change aucune
structure de code puisque l'accès se fait par API S3, déjà abstraite pour Garage.

---

## R-14 — Trou identifié dans le gel : aucun générateur de client TypeScript n'y figure

**Constat, non résolu ici.** `docs/versions-gelees.md` §3.2 épingle trois paquets JavaScript —
`@tauri-apps/cli`, `@tauri-apps/api`, `@nuxtjs/i18n`. **Aucun générateur de client OpenAPI → TypeScript
n'y est listé.** Or TRX-01 et la porte P-01 en exigent un dès ce cycle : sans lui, il n'y a pas de
client à régénérer, donc pas de diff à détecter, donc pas de porte.

**Ce que le plan fait** : il nomme le besoin et **ne choisit ni outil ni version**. Conformément à
la consigne — *« ne propose aucun numéro de version, ni de mémoire ni par vérification »* — le
choix de l'outil et la vérification de sa version sur son registre officiel sont un **point
d'entrée de la revue de gel**, à traiter avant `/speckit-implement`. C'est un ajout au gel, pas
une décision de plan.

**Critères que l'outil devra satisfaire**, eux, sont arrêtés ici : sortie déterministe (même
contrat ⇒ mêmes octets, sans quoi P-01 produirait des faux positifs à chaque exécution),
exécutable sous Node 24 LTS, disponible sur les deux architectures, et pas de dépendance à un
runtime Java.

**Impact si non traité** : la porte P-01 ne peut pas être livrée, et US5 échoue. C'est le seul
blocage dur identifié dans ce cycle.

---

## R-15 — Ce que le cycle ne peut pas encore vérifier, et pourquoi c'est dit

Quatre portes constitutionnelles n'ont **aucune cible** au cycle 1 : P-09 (contrainte d'exclusion
GiST), P-10 (montants entiers, quantités `NUMERIC`), P-11 (tests dorés fiscaux), P-12 (règles
fiscales hors `JurisdictionAdapter`).

**Décision** : elles sont **installées et vertes à vide**, avec une assertion de non-régression
qui échoue si la porte cesse de trouver quoi que ce soit à vérifier **après** que le cycle
concerné l'a activée.

**Rationale** : une porte ajoutée « quand on en aura besoin » n'est jamais ajoutée — ou elle
l'est après que trois cycles ont écrit du code non conforme. Une porte verte à vide coûte une
poignée de lignes et garantit qu'aucun cycle ultérieur ne pourra livrer sans la rencontrer.

---

## R-16 — Le patron de référence cité par la consigne n'existe pas encore

**Constat** : la consigne de planification dit *« le patron de référence est `docs/module-dore.md`
— aligne-toi dessus plutôt que sur un exemple trouvé en ligne »*. Ce fichier **est un livrable de
ce cycle** (FR-027) ; il n'existe pas au moment du plan.

**Conséquence assumée** : ce cycle **produit** le patron au lieu de le suivre. C'est
l'inversion normale du premier cycle, et c'est précisément pourquoi le cadrage §13.1 exige le
module doré « avant toute génération assistée ». Les décisions R-03, R-05, R-07 et R-12 ci-dessus
sont les éléments du patron qui ne se devinent pas et qui, non écrits maintenant, seraient
réintroduits en version 0.8.x par chaque cycle suivant.

**Ordre imposé aux tâches** : le module doré est écrit **à la main et en premier**, avant toute
autre tranche de code du cycle. `/speckit-tasks` doit refléter cet ordre, pas le paralléliser.
