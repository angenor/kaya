---

description: "Liste de tâches — cycle 001, socle technique du monorepo Kaya"
---

# Tasks: Socle technique du monorepo Kaya

**Input**: documents de conception de `/specs/001-socle-technique-monorepo/`

**Prerequisites**: [plan.md](./plan.md) · [spec.md](./spec.md) · [research.md](./research.md) ·
[data-model.md](./data-model.md) · [contracts/](./contracts/) · [quickstart.md](./quickstart.md)

**Tests** : **obligatoires**, pas optionnels. La Definition of Done (`docs/user-stories-v1.md`
§0.4) l'exige, et les tests hors-ligne du §0.7 sont des portes de CI.

**Organisation** : par user story. Chaque phase est un incrément testable indépendamment.

## Format: `[ID] [P?] [Story] Description`

- **[P]** : parallélisable — fichiers différents, aucune dépendance non satisfaite
- **[Story]** : US1 à US9, conformément à [spec.md](./spec.md)
- Chemin de fichier exact dans chaque description

---

## ⚠️ Un point à lire avant de commencer

**B-1 — RÉSOLU le 2026-07-30. Aucune tâche n'est bloquée.**
Le gel est passé en **1.0.3** et §3.2 porte désormais le générateur, vérifié sur le registre npm :
`openapi-typescript` **7.13.0** (génère **uniquement les types**), `openapi-fetch` **0.17.0**
(client runtime **écrit à la main, jamais généré**), `typescript` **7.0.2** (peerDependency).
Le choix minimise la surface soumise à P-01 : un seul fichier de types, sans code d'exécution.
**T041 à T044 sont exécutables.** Deux vérifications conditionnent la clôture d'US5 :
déterminisme d'octet constaté par `cmp` sur deux exécutions consécutives, et ordre de membres
stable quand un endpoint est ajouté en fin de fichier Rust (le diff doit rester local).

**B-2 — Ce cycle ne produit finalement aucun écran. Tranché.**
Vérification faite : l'écran de notes d'établissement n'est **ni maquetté (a)** — les 11 codes de
`docs/design/html/` sont `C4`, `F2`, `G2`, `M4`, `P2`, `Q1`, `R1`, `R4`, `R7`, `S2`, `V1` — **ni
dérivé (b)** : la matrice `docs/design/derivation.md` n'a aucune ligne pour lui.

**Décision** : le module doré est réduit à **six couches**, la couche écran étant reportée au
cycle ETB, qui dispose d'écrans maquettés. La consigne « TRX ne produit aucun écran » se trouve
donc vérifiée — après examen, pas par présomption.

**Conséquences à ne pas perdre de vue** : le patron ne démontrera ni i18n, ni mode sombre, ni
RBAC, ni chargement paresseux. Ces quatre points restent à figer au cycle ETB, et
`docs/module-dore.md` doit le dire (T031). Le point **8 de la Definition of Done** (« écran
vérifié en clair et en sombre ») devient **sans objet** à ce cycle, au même titre que le point 10
— à consigner explicitement en T091, jamais à cocher en silence.

**B-3 — Terme utilisateur : tranché.** `note_etablissement` s'affiche **« Note interne »** en
français, *Internal note* en anglais. À ajouter au lexique `docs/design/lexique.md` (T031b) ; le
mot « établissement » est superflu dans le libellé, le lexique §6 posant déjà que l'utilisateur est
toujours dans le sien.

---

## Phase 1: Setup — arborescence et gel (US1)

**Objectif** : le dépôt existe, compile, et toutes les versions sont épinglées exactement.

- [ ] T001 Créer l'arborescence complète du monorepo conformément à `plan.md` § Project Structure — `backend/`, `app/`, `web/`, `clients/ts/`, `infra/`, `scripts/ci/`, `.github/workflows/`, avec un `.gitkeep` dans chaque répertoire vide
- [ ] T002 Créer `backend/Cargo.toml` (workspace) avec `[workspace.dependencies]` en **versions exactes** reprises de `docs/versions-gelees.md` §2 et §3.1 — jamais `^`, `~` ni plage
- [ ] T003 Créer `backend/rust-toolchain.toml` avec le canal exact du gel — jamais `stable`
- [ ] T004 [P] Créer les 14 crates vides mais compilables sous `backend/crates/` — `domain/`, les 9 de `socle/`, `capacites/stocks/`, les 4 de `verticales/` — chacun avec son `Cargo.toml` héritant du workspace et un `lib.rs` vide
- [ ] T005 [P] Créer les binaires `backend/api/` (Actix) et `backend/node/` (coquille, incrément 3) avec leurs `Cargo.toml`
- [ ] T006 Déclarer les dépendances inter-crates dans les `Cargo.toml` en respectant la hiérarchie stricte — `socle/` ne dépend que de `socle/`, `capacites/` de `socle/`, `verticales/` de `socle/` et `capacites/` ; vérifier `cargo build --workspace` vert
- [ ] T007 [P] Configurer `backend/.cargo/config.toml` — `debug = "line-tables-only"` en profil dev, `sccache` comme wrapper, et **`mold` conditionné à la cible Linux uniquement** (`research.md` R-01 : mold ne supporte pas macOS)
- [ ] T008 [P] Créer `infra/compose.yml` avec les trois services aux **tags d'image exacts** de `docs/versions-gelees.md` §4.2 — jamais `latest`
- [ ] T009 [P] Initialiser `app/` (Nuxt 4, SPA — SSR désactivé) et `package.json` en versions exactes du gel §3.2 et §3.3, plus `.nvmrc`
- [ ] T010 [P] Initialiser les coquilles `web/qr/` (SSR activé) et `web/console/` (`ssr:false`)
- [ ] T011 [P] Copier `docs/design/theme.css` **tel quel** vers `app/assets/css/theme.css` — seule exception autorisée par le principe XII ; configurer Tailwind 4 pour le consommer
- [ ] T012 Commiter tous les lockfiles — `backend/Cargo.lock` (y compris pour les binaires), `pnpm-lock.yaml`
- [ ] T013 Créer `.github/workflows/ci.yml` avec **filtrage par chemins** — une modification de `docs/` seule ne déclenche pas la construction du backend

**Checkpoint** : `cargo build --workspace` et `docker compose up` verts sur un poste neuf.

---

## Phase 2: Foundational — base, rôles et outbox (BLOQUANT)

> ⚠️ **ORDONNANCEMENT DES PORTES — à respecter, sans renumérotation.**
> Trois portes sont écrites plus loin dans la liste que le code qu'elles protègent. Une porte
> posée après sa cible documente l'existant au lieu de le contraindre. **Exécuter :**
> - **T074 (porte P-20, épinglage exact)** dès la fin de la Phase 1 — c'est la phase qui *fait*
>   le gel ; sinon un `^` peut survivre soixante-dix tâches.
> - **T045 et T046 (portes P-07 RLS et P-08 isolation)** à la fin de cette Phase 2, **avant
>   toute autre table** — ce sont les portes du principe III, et les Phases 2 et 3 créent déjà
>   du schéma. Leurs tâches restent numérotées en Phase 6 ; seule leur exécution avance.
> - **T078 (porte P-18, `cargo sqlx prepare`)** avec la première requête sqlx de cette phase.
>
> Les autres portes peuvent rester en Phase 11 : P-06, P-15, P-16, P-17 et P-19 n'ont pas de
> cible à ce cycle et sont installées à vide.

**⚠️ Aucune user story ne peut démarrer avant la fin de cette phase.**

- [ ] T014 Créer la migration `backend/migrations/0001_roles_et_schemas.sql` — rôles `kaya_owner`, `kaya_app`, `kaya_ledger_reader` ; schémas `etablissements`, `synchronisation`, `fiscalite` ; extension `btree_gist`
- [ ] T015 Créer `backend/sqlx.toml` — configuration multi-schémas et renommage de `_sqlx_migrations` (`docs/versions-gelees.md` §2, apport de sqlx 0.9)
- [ ] T016 Créer la migration `backend/migrations/0002_etablissements_socle.sql` — tables `tenant` et `etablissement` en **forme minimale** (`data-model.md` §2), avec `ENABLE` **et** `FORCE ROW LEVEL SECURITY` et la politique `isolation_tenant` (`USING` **et** `WITH CHECK`)
- [ ] T017 Implémenter dans `backend/api/src/db.rs` **deux configurations de connexion distinctes** — `kaya_owner` pour les migrations, `kaya_app` pour le runtime ; un seul rôle élargi annulerait l'effet de `FORCE`
- [ ] T018 Implémenter la pose du tenant courant dans `backend/crates/socle/etablissements/src/tenant_context.rs` via `SELECT set_config('app.current_tenant', $1, true)` **paramétré** — jamais `SET LOCAL` interpolé (`research.md` R-03) ; requête littérale, donc vérifiée par `query!` et sans `AssertSqlSafe`
- [ ] T019 Câbler `sqlx::migrate!()` **au démarrage** de `backend/api/src/main.rs`, sous `kaya_owner`, avant l'ouverture du port ; ajouter un test de démarrage concurrent (verrou consultatif sqlx)
- [ ] T020 [P] Configurer `tracing` + `tracing-subscriber` dans `backend/api/src/observabilite.rs` — journaux structurés avec identifiant de corrélation par requête
- [ ] T021 Créer la migration `backend/migrations/0003_outbox.sql` — table `evenement_outbox` (`data-model.md` §4), séquence par établissement, **index partiel sur `publie_le IS NULL`**, RLS `ENABLE`+`FORCE`+politique
- [ ] T022 Compléter `0003_outbox.sql` par les trois couches d'immuabilité (`research.md` R-05) — `REVOKE UPDATE, DELETE` pour `kaya_app`, `GRANT UPDATE (publie_le)`, et le déclencheur `evenement_outbox_immuable` qui s'applique **aussi au propriétaire**
- [ ] T023 Définir les traits `OutboxWriter` et `EventConsumer` dans `backend/crates/socle/synchronisation/src/lib.rs` conformément à `contracts/traits-exposes.md` §1 et §2 — `ecrire` **prend une transaction en paramètre** et n'en ouvre jamais une
- [ ] T024 Implémenter `PgOutboxWriter` dans `backend/crates/socle/synchronisation/src/outbox.rs` — pose `survenu_le` (horodatage d'autorité serveur) et `sequence_etablissement` côté serveur, jamais depuis l'appelant

**Checkpoint** : la base est migrée, l'isolation est posée, l'outbox est écrivable dans une
transaction. Les user stories peuvent démarrer.

---

## Phase 3: User Story 2 — Module doré (Priority: P1) 🎯 MVP

**Objectif** : une tranche verticale **de six couches** écrite **à la main** sur
`note_etablissement`, qui devient le patron de tous les cycles suivants. La septième couche —
l'écran — est reportée au cycle ETB (voir B-2).

**Test indépendant** : un développeur reproduit une seconde tranche en ne lisant que
`docs/module-dore.md`.

> **Ordre impératif** : cette phase précède **tout** le reste du code (cadrage §13.1 — « avant
> toute génération assistée »). Ne pas la paralléliser avec les phases 4 et suivantes.

- [ ] T025 [US2] Créer la migration `backend/migrations/0004_note_etablissement.sql` — table `note_etablissement` (`data-model.md` §3) avec `id` **fourni par le client**, `horodatage_client` et `cree_le` **distincts**, `CHECK` sur la longueur du texte, index, RLS `ENABLE`+`FORCE`+politique
- [ ] T026 [US2] Déclarer `note_etablissement` en **classe A** au §5.1 de `docs/registre-classes-offline.md`, plus une entrée au §13 (journal des modifications)
- [ ] T027 [US2] Écrire le repository `backend/crates/socle/etablissements/src/note/repository.rs` **à la main contre sqlx 0.9** — `INSERT ... ON CONFLICT (id) DO NOTHING`, macros `query!` littérales ; aucun extrait 0.8.x
- [ ] T028 [US2] Écrire le service `backend/crates/socle/etablissements/src/note/service.rs` — ouvre la transaction, insère la note **et** appelle `OutboxWriter::ecrire` dans la **même** transaction, émettant `note_etablissement.creee` (`data-model.md` §4.3)
- [ ] T029 [US2] Écrire les handlers `backend/api/src/routes/notes.rs` avec leurs annotations `#[utoipa::path]` conformément à `contracts/http-api.md` §2 — `GET` liste, `POST` création renvoyant **`200` sur rejeu** et `201` sur création
- [ ] T030 [US2] Écrire les tests dans `backend/tests/note_etablissement_classe_a.rs` — **rejeu** (trois envois du même `id` → un seul enregistrement) et **désordre** (trois notes dans les six ordres → même état final), conformément à `docs/user-stories-v1.md` §0.7
- [ ] T031 [US2] **Aucun écran à ce cycle — décision prise.** Le module doré est réduit à **six couches** : la couche écran est reportée au cycle ETB, qui dispose d'écrans réellement maquettés (`G2`, `M4`). Motif : l'écran de notes n'est ni maquetté ni dérivé — absent des 11 codes de `docs/design/html/` et des 30 lignes de `docs/design/derivation.md` — et la règle « un écran qui n'hérite d'aucun motif ne se code pas » prime. **Consigner ce report dans `docs/module-dore.md`** comme un manque connu du patron : i18n, mode sombre et RBAC restent à démontrer au cycle ETB
- [ ] T031b [US2] Ajouter au lexique `docs/design/lexique.md` la ligne « `note_etablissement` → **« Note interne »** / *Internal note* » et créer les clés i18n correspondantes dans `app/core/i18n/fr.json` et `en.json`, prêtes pour le cycle ETB
- [ ] T032 [US2] Après T029 : régénérer le client TypeScript vers `clients/ts/`, vérifier l'absence de diff non commité, `cargo build --workspace` vert *(dépend de la levée de B-1)*
- [ ] T033 [US2] Rédiger `docs/module-dore.md` — les **six** couches, leur extrait de référence, leur raison d'être, le **report de la couche écran** comme manque connu (i18n, mode sombre, RBAC, chargement paresseux à figer au cycle ETB), et les pièges de sqlx 0.9 neutralisés : `AssertSqlSafe` évité par `set_config` paramétré (R-03), absence de clé étrangère inter-modules, `id` client, séquence non transactionnelle (R-07)

**Checkpoint** : le patron existe et est documenté. Tous les cycles suivants le recopient.

---

## Phase 4: User Story 3 — Grand livre d'événements (Priority: P1)

**Objectif** : rétention illimitée, charge utile dénormalisée, immuabilité — prouvées par test.

**Test indépendant** : le test de reconstitution autonome passe sous un rôle qui ne peut lire
aucune autre table.

- [ ] T034 [US3] Implémenter le worker de publication in-process dans `backend/crates/socle/synchronisation/src/worker.rs` — `SELECT ... WHERE publie_le IS NULL ORDER BY id FOR UPDATE SKIP LOCKED LIMIT n`, puis marquage `publie_le` (`research.md` R-08) ; aucune file externe, aucun usage de Redis
- [ ] T035 [P] [US3] Implémenter deux consommateurs idempotents de démonstration dans `backend/crates/socle/synchronisation/src/consommateurs/` — chacun garde la trace de son dernier événement traité
- [ ] T036 [US3] Créer le jeu de cas financier figé dans `backend/tests/fixtures/grand_livre_v1.json` — encaissement complet avec montant en unités mineures, mode, contrepartie, ventilation de taxes en **millièmes entiers** et référence de document (`data-model.md` §4.2)
- [ ] T037 [US3] Écrire `backend/tests/reconstitution_autonome.rs` — se connecte avec `kaya_ledger_reader`, reconstitue chaque opération depuis la **seule** charge utile ; toute lecture d'une autre table lève une erreur de permission (`research.md` R-11). **C'est le test central du cycle.**
- [ ] T038 [P] [US3] Écrire `backend/tests/outbox_immuabilite.rs` — `UPDATE` et `DELETE` refusés sous `kaya_app`, puis **refusés à nouveau sous `kaya_owner`** ; seul `publie_le` `NULL → valeur` passe, une seule fois
- [ ] T039 [P] [US3] Écrire `backend/tests/outbox_transactionnel.rs` — après chaque mutation exposée un événement existe ; après un rollback provoqué, **ni ligne métier ni événement** (porte P-05)
- [ ] T040 [US3] Écrire `backend/tests/worker_redemarrage.rs` — redémarrage brutal au milieu d'un lot : nombre d'événements identique avant et après, effet d'une seule présentation chez les consommateurs

**Checkpoint** : le grand livre est prouvé permanent, autonome et immuable.

---

## Phase 5: User Story 5 — Contrat OpenAPI et client généré (Priority: P2)

**Objectif** : le contrat est généré depuis le code, le client depuis le contrat, et un diff fait
échouer le build.

**Test indépendant** : modifier une signature sans régénérer → la CI échoue.

> ✅ **B-1 résolu** — générateur au gel 1.0.3 §3.2. Phase exécutable.

- [ ] T041 [US5] Exposer `/api-docs/openapi.json` dans `backend/api/src/openapi.rs` via utoipa, et documenter `GET /health` conformément à `contracts/http-api.md` §1 — la sonde teste les dépendances par **requête réelle**, jamais l'état d'un pool en mémoire
- [ ] T042 [US5] Monter l'interface Swagger UI **conditionnellement au démarrage** dans `backend/api/src/main.rs` — une route non montée en production ne peut pas fuir par oubli de garde
- [ ] T043 [US5] Créer le script de génération `scripts/ci/generer-client.sh` — `openapi-typescript` **7.13.0** sur `/api-docs/openapi.json`, sortie vers `clients/ts/`. **Vérifier le déterminisme** : exécuter deux fois de suite et comparer par `cmp` ; ajouter un endpoint en fin de fichier Rust et vérifier que le diff généré reste local. Sans ces deux propriétés, P-01 échouera au hasard et finira désactivée
- [ ] T044 [US5] Ajouter le job de porte **P-01** dans `.github/workflows/ci.yml` — régénère puis `git diff --exit-code clients/ts` ; ajouter un test négatif qui vérifie que la porte échoue bien sur un diff volontaire

**Checkpoint** : le contrat est source de vérité et le client ne peut plus diverger.

---

## Phase 6: User Story 4 — Isolation multi-tenant (Priority: P1)

**Objectif** : aucune donnée ne franchit la frontière d'un tenant, et une table sans RLS fait
échouer le build.

**Test indépendant** : deux tenants seedés, chaque endpoint visé en croisé → aucune ligne.

- [ ] T045 [US4] Écrire `backend/tests/rls_catalogue.rs` (porte **P-07**) — interroge `pg_class.relrowsecurity`, `relforcerowsecurity` et `pg_policies` ; les trois conditions vérifiées **séparément** avec des messages distincts ; liste d'exclusion **nommée** (tables sqlx), jamais un motif de nom
- [ ] T046 [US4] Écrire `backend/tests/isolation_tenant.rs` (porte **P-08**) — **paramétré sur la liste des routes de l'OpenAPI**, pour qu'un endpoint ajouté sans test fasse échouer la porte
- [ ] T047 [P] [US4] Ajouter à `backend/tests/rls_catalogue.rs` le cas « transaction sans contexte de tenant » — la lecture retourne **zéro ligne**, jamais une erreur ni un accès total
- [ ] T048 [P] [US4] Ajouter un test négatif documenté dans `backend/tests/rls_catalogue.rs` — une table créée sans politique fait échouer la porte, avec son nom dans le message

**Checkpoint** : l'isolation est opposable, pas déclarative.

---

## Phase 7: User Story 6 — Observabilité et sauvegardes (Priority: P2)

**Objectif** : diagnostic possible à 220 km, et sauvegardes réellement restaurables.

**Test indépendant** : restaurer une sauvegarde chiffrée dans un environnement vierge en suivant
la seule procédure écrite.

- [ ] T049 [P] [US6] Câbler Sentry dans `backend/api/src/observabilite.rs` — remontée des erreurs avec leur contexte, sans secret ni chaîne de connexion
- [ ] T050 [US6] Implémenter la sonde `/health` dans `backend/api/src/routes/sante.rs` — `SELECT 1` sur la base, `PING` sur le cache, vérification du stockage objet ; **ne renvoie jamais** hôte, version de base ni trace d'erreur
- [ ] T051 [US6] Écrire `infra/backup/sauvegarder.sh` — `pg_dump` quotidien, **chiffré avant transfert**, poussé vers le stockage objet **tiers sur hôte distinct** avec verrouillage d'objet (`research.md` R-13) ; copie de travail vers Garage, qui ne porte **jamais** l'immutabilité
- [ ] T052 [US6] Écrire `infra/backup/restaurer.sh` et sa procédure dans `infra/backup/README.md` — rédigée pour être suivie **par quelqu'un qui n'a pas écrit le système**
- [ ] T053 [US6] Exécuter et chronométrer un **premier exercice de restauration complet** en environnement vierge ; consigner la durée et les écarts dans `infra/backup/README.md`
- [ ] T054 [P] [US6] Configurer la supervision externe déclenchant une alerte au-delà de **2 minutes** d'indisponibilité — hébergée hors du serveur surveillé, faute de quoi elle ne prouve rien

**Checkpoint** : une panne est diagnosticable à distance et une sauvegarde est restaurable.

---

## Phase 8: User Story 7 — Seeds et démonstration (Priority: P2)

**Objectif** : deux tenants rechargeables en une commande.

**Test indépendant** : trois exécutions successives → état final identique.

> **Portée réduite assumée** (`plan.md` Complexity Tracking, écart 4) : les 17 unités, les 30
> articles et les 5 comptes de test (FR-062) peuplent des tables qui n'existent pas encore — elles
> viennent des cycles HEB, PDV et CPT. Ce cycle livre la **mécanique** et les **deux tenants**.

- [ ] T055 [US7] Écrire le binaire de seeds `backend/api/src/bin/seeds.rs` — **rejouable**, vivant dans `backend/migrations/seeds/`, séparé des migrations (principe I(b))
- [ ] T056 [US7] Seeder le tenant Deloria et son établissement d'Abengourou — non classé, commune d'Abengourou, fuseau `Africa/Abidjan`, devise `XOF` (`docs/cadrage-v1.md` §2.1)
- [ ] T057 [P] [US7] Seeder le second tenant « Résidence Test » — **module hébergement seul**, aucun point de vente, pour prouver que rien dans le socle ne suppose l'existence d'un point de vente
- [ ] T058 [US7] Consigner dans `backend/migrations/seeds/README.md` les valeurs à seeder par les cycles ultérieurs — les 5 catégories et leurs tarifs de `docs/cadrage-v1.md` §2.1, **décomposés en prix HT + TVA + taxe communale** (FR-062b), et les barèmes du récapitulatif des paramètres marqués **provisoires** tant que **B-07** n'est pas tranchée
- [ ] T059 [US7] Écrire `backend/tests/seeds_rejouables.rs` — trois exécutions successives produisent le même état final

**Checkpoint** : une base de démonstration se recharge en une commande.

---

## Phase 9: User Story 8 — Registre des classes hors-ligne opposable (Priority: P3)

**Objectif** : une entité absente du registre fait échouer le build.

**Test indépendant** : ajouter une table sans la déclarer → la CI échoue.

- [ ] T060 [US8] Écrire `backend/tests/classes_offline.rs` (porte du registre) — compare `information_schema.tables` aux entités déclarées dans `docs/registre-classes-offline.md` ; sens de comparaison **table → registre** (`research.md` R-10)
- [ ] T061 [US8] Documenter dans `backend/tests/classes_offline.rs` la **limite assumée** : la porte vérifie la *présence* d'une entité, pas la justesse de sa classe — le registre classe des opérations, pas seulement des tables ; la justesse reste humaine et revue mensuellement
- [ ] T062 [US8] Implémenter la porte **P-13** dans `app/core/sync/` — la file locale n'accepte que des types **marqués classe A au niveau du type** ; enregistrer un type non-A ne doit pas compiler
- [ ] T063 [P] [US8] Écrire `app/tests/file-classe-a.spec.ts` — vérifie qu'aucune opération B, C ou D n'est atteignable depuis un chemin exécutable hors ligne

**Checkpoint** : le registre est opposable, plus seulement documentaire.

---

## Phase 10: User Story 9 — Provisions comptables (Priority: P3)

**Objectif** : deux tables, aucune logique.

**Test indépendant** : inspecter le schéma ; constater l'absence totale d'endpoint et d'écran.

- [ ] T064 [US9] Créer la migration `backend/migrations/0005_provisions_comptables.sql` — `exercice_comptable` avec sa **contrainte d'exclusion GiST** sur `daterange` et `mapping_comptable` (`data-model.md` §5), toutes deux avec RLS `ENABLE`+`FORCE`+politique
- [ ] T065 [US9] Ajouter au même fichier le déclencheur refusant toute écriture sur une **période close** — déclencheur et non règle applicative, sinon la première migration de données le contournerait
- [ ] T066 [P] [US9] Écrire `backend/tests/provisions_sans_logique.rs` — vérifie l'existence des deux tables **et** l'absence de tout endpoint, écran ou service les consommant (principe X)
- [ ] T067 [US9] Consigner dans `docs/module-dore.md` le retour du **spike `EXCLUDE USING gist`** : `btree_gist` disponible, mapping de type sqlx 0.9 validé sur `daterange` — préalable à HEB-02, qui en dépendra sur `tstzrange`

**Checkpoint** : la provision est posée sans une ligne de logique.

---

## Phase 11: Portes de CI transverses

**Objectif** : les 20 portes existent, y compris celles sans cible.

- [ ] T068 [P] Écrire `backend/tests/architecture.rs` — portes **P-03** (aucune arête `socle/ → verticales/`, ni `capacites/ → verticales/`) et **P-12** (aucun crate hors `socle/fiscalite` ne référence les types de taxe de `domain`), par lecture de `cargo metadata`
- [ ] T069 [P] Écrire `scripts/ci/migrations-figees.sh` — porte **P-02** : empreinte de chaque migration comparée à la branche de base ; échec si un fichier existant diffère
- [ ] T070 [P] Écrire `scripts/ci/jointures-inter-schemas.sh` — porte **P-04** ; **écrire la limite de l'heuristique en tête du script**, pas dans un commentaire enfoui
- [ ] T071 [P] Écrire `scripts/ci/outbox-sans-purge.sh` — porte **P-05b** : échec sur tout `DELETE` ou `TRUNCATE` visant `evenement_outbox`
- [ ] T072 [P] Écrire `scripts/ci/types-monetaires.sh` — porte **P-10** : échec sur `FLOAT`/`REAL`/`DOUBLE` pour un montant et sur tout entier nommé `quantite`
- [ ] T073 [P] Écrire `scripts/ci/maquettes-non-copiees.sh` — porte **P-19** : empreintes de `docs/design/html/**` comparées à tout fichier sous `app/`, avec `theme.css` **explicitement exclu**
- [ ] T074 [P] Écrire `scripts/ci/versions-epinglees.sh` — porte **P-20** : échec sur tout intervalle, tout tag `latest`, puis `cargo build --locked` et `pnpm install --frozen-lockfile`
- [ ] T075 [P] Ajouter la règle ESLint `no-restricted-imports` sur `@tauri-apps/api` dans `app/eslint.config.js` — porte **P-15**, avec dérogation nommée au seul répertoire `app/core/platform/`
- [ ] T076 [P] Écrire `app/scripts/test-i18n.ts` — porte **P-16** : parité des jeux de clés `fr` et `en`, plus détection des littéraux affichés
- [ ] T077 [P] Écrire `app/scripts/lint-tokens.ts` — porte **P-17** : échec sur tout `#rrggbb`, `rgb(`, ou `px` hors des jetons de `theme.css`
- [ ] T078 Ajouter `cargo sqlx prepare --workspace --check` à la CI et commiter `backend/.sqlx/` — porte **P-18**
- [ ] T079 Installer **à vide** les portes **P-06**, **P-09**, **P-11** avec leur assertion de non-régression (`research.md` R-15) — chacune échoue si elle cesse de trouver une cible après le cycle qui l'active
- [ ] T080 Câbler les 20 portes dans `.github/workflows/ci.yml` et vérifier que **chacune échoue** sur un cas non conforme injecté volontairement (SC-002)

**Checkpoint** : aucune règle de la constitution n'est plus contournable par convention.

---

## Phase 12: Interfaces transverses de l'application

- [ ] T081 [P] Définir `PlatformAdapter` et le type `ResultatCapacite` dans `app/core/platform/index.ts` conformément à `contracts/traits-exposes.md` §5 — le type de retour force chaque appelant à traiter le cas « capacité absente »
- [ ] T082 [P] Créer les quatre implémentations **vides mais conformes** dans `app/core/platform/` — `desktop.ts`, `android.ts`, `ios.ts`, `web.ts` ; chacune renvoie `{ disponible: false }` pour ce qu'elle ne sait pas faire
- [ ] T083 [P] Poser `app/core/i18n/` avec les catalogues `fr.json` et `en.json`, **fr par défaut**, et `app/core/theme/` avec le mode sombre par la variante `dark:` — jamais une seconde palette
- [ ] T084 [P] Poser les coquilles `app/core/auth/`, `app/core/rbac/` (permissions cumulatives) et `app/core/sync/` — structure et types seulement, la logique relève de CPT et SYN
- [ ] T085 [P] Déclarer `JurisdictionAdapter` dans `backend/crates/socle/fiscalite/src/lib.rs` — les cinq méthodes du cadrage §14.1, **déclarées sans implémentation**, avec les types associés minimaux dans `domain`
- [ ] T086 [P] Déclarer `EstablishmentDirectory` dans `backend/crates/socle/etablissements/src/lib.rs` — le trait par lequel les futures verticales liront un établissement, **jamais par jointure**

---

## Phase 13: Emplacements des stories P1 hors périmètre

**Objectif** : réserver la place de TRX-06, TRX-07 et TRX-08 sans écrire une ligne de logique.

> Placées en fin de liste, conformément à la consigne : livrables après le cœur P0.

- [ ] T087 [P] Créer `docs/conformite/README.md` — emplacement du registre des traitements ARTCI (**TRX-06**, P1), avec le périmètre à venir : export et suppression des données d'une personne, rétention paramétrable, consentement tracé
- [ ] T088 [P] Créer `infra/autoheberge/README.md` — emplacement du paquet mode B (**TRX-07**, P1) ; noter que les migrations idempotentes au démarrage sont **déjà livrées** (T019) et que la règle N/N-1 du cadrage §10.2 reste due
- [ ] T089 [P] Créer `app/core/design-system/README.md` — emplacement des 12 composants canoniques (**TRX-08**, P1) ; noter que `theme.css` et le mode sombre sont **déjà en place** (T011, T083)

---

## Phase 14: Revue de la Definition of Done

- [ ] T090 Exécuter le guide complet de [quickstart.md](./quickstart.md), sections 1 à 9, et consigner les écarts
- [ ] T091 **Revue Definition of Done** — vérifier les dix points de `docs/user-stories-v1.md` §0.4 pour chaque story livrée : (1) critères couverts par tests unitaires **et** d'intégration ; (2) annotations utoipa à jour et client régénéré sans diff ; (3) migration versionnée, `cargo sqlx prepare` vert, seeds à jour ; (4) RLS activée et forcée sur toute nouvelle table avec test d'isolation ; (5) classe hors-ligne déclarée avec ses tests ; (6) événement outbox émis pour tout changement d'état ; (7) clés i18n fr et en, aucune chaîne en dur ; (8) écran vérifié en clair **et** sombre ; (9) paramètres exposés en configuration d'établissement ; (10) documents imprimés vérifiés sur imprimante thermique — **sans objet à ce cycle, à consigner explicitement comme tel**

---

## Dependencies & Execution Order

### Dépendances de phase

- **Phase 1 (Setup)** — aucune dépendance
- **Phase 2 (Foundational)** — dépend de la Phase 1. **Bloque toutes les user stories**
- **Phase 3 (US2, module doré)** — dépend de la Phase 2. **Bloque toutes les autres stories** :
  c'est le patron, et l'écrire après aurait pour effet que les autres phases n'en tiennent pas
  compte (cadrage §13.1)
- **Phases 4 à 10** — dépendent de la Phase 3, parallélisables entre elles
- **Phase 11 (portes)** — dépend des phases qui créent les cibles à vérifier
- **Phases 12 à 13** — parallélisables dès la Phase 1
- **Phase 14** — dépend de tout le reste

### Dépendances entre stories

| Story | Dépend de | Raison |
|---|---|---|
| US1 (Phase 1) | — | Fondation |
| US2 (Phase 3) | US1 + Foundational | Le patron a besoin de la base et de l'outbox |
| US3 (Phase 4) | US2 | Le worker publie ce que le module doré écrit |
| US4 (Phase 6) | US2 | L'isolation se teste sur des endpoints réels |
| US5 (Phase 5) | US2 + **B-1 levé** | Rien à générer sans endpoint ni outil |
| US6, US7, US9 | Foundational | Indépendantes entre elles |
| US8 (Phase 9) | US2 | La porte du registre a besoin d'une entité déclarée |

### Opportunités de parallélisation

- **Phase 1** : T004, T005, T007 à T011 en parallèle
- **Phase 11** : T068 à T077 sont dix scripts indépendants — parallélisation maximale
- **Phases 12 et 13** : entièrement parallélisables, dès le début du cycle
- **Phase 4** : T035, T038, T039 en parallèle une fois T034 posé

---

## Parallel Example: Phase 11 (portes de CI)

```bash
# Dix scripts de porte, fichiers distincts, aucune dépendance mutuelle :
Task: "backend/tests/architecture.rs — portes P-03 et P-12"
Task: "scripts/ci/migrations-figees.sh — porte P-02"
Task: "scripts/ci/jointures-inter-schemas.sh — porte P-04"
Task: "scripts/ci/outbox-sans-purge.sh — porte P-05b"
Task: "scripts/ci/types-monetaires.sh — porte P-10"
Task: "scripts/ci/maquettes-non-copiees.sh — porte P-19"
Task: "scripts/ci/versions-epinglees.sh — porte P-20"
Task: "app/eslint.config.js — porte P-15"
Task: "app/scripts/test-i18n.ts — porte P-16"
Task: "app/scripts/lint-tokens.ts — porte P-17"
```

---

## Implementation Strategy

### MVP réel — jusqu'à la fin de la Phase 3

Le MVP de ce cycle n'est pas la première user story de la spec : c'est le **module doré**. Tant
qu'il n'est pas écrit et documenté, chaque tâche suivante risque de réintroduire des tournures
sqlx 0.8 qu'il faudra défaire.

1. Phase 1 — Setup
2. Phase 2 — Foundational
3. Phase 3 — Module doré **+ `docs/module-dore.md`**
4. **ARRÊT ET VALIDATION** : un développeur reproduit une tranche en ne lisant que le patron

### Livraison incrémentale

Phases 4 → 6 → 7 → 8 → 9 → 10, chacune testable seule. La Phase 5 (US5) attend la levée de B-1.

### Notes

- `[P]` = fichiers différents, aucune dépendance
- Commiter après chaque tâche ou groupe logique
- **Chaque tâche touchant le schéma commence par sa migration**, politiques RLS incluses
- **Chaque tâche touchant l'API se termine par** annotations utoipa + client régénéré + build vert
- **Chaque tâche créant une entité inclut** sa déclaration au registre des classes hors-ligne et
  ses tests de classe
