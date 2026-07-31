# Revue de fin de cycle — 001, socle technique du monorepo

**Date** : 2026-07-31 · **Branche** : `001-socle-technique-monorepo`
**Tâches** : 90 exécutées sur 92 · **Tests** : 37 tests d'intégration Rust + 18 tests
d'application, verts sur trois exécutions consécutives

Ce document porte **T090** (déroulé du guide de validation) et **T091** (revue de la Definition of
Done). Il consigne ce qui est fait, ce qui ne l'est pas, et pourquoi — jamais une case cochée en
silence.

---

## 1. T090 — Déroulé de `quickstart.md`, sections 1 à 9

| § | Objet | Résultat | Écart |
|---|---|---|---|
| 1 | Amorçage | ✅ | `/health` **200**, trois dépendances opérationnelles. 4 écarts de commande, voir 1.1 |
| 2 | Contrat et client généré | ✅ | — |
| 3 | Isolation multi-tenant | ✅ | 7 tests verts |
| 4 | Grand livre | ✅ | 8 tests verts |
| 5 | Module doré et classes hors-ligne | ✅ | 5 tests verts |
| 6 | Fondations d'interface | ✅ | — |
| 7 | Seeds | ✅ | 2 tests verts |
| 8 | Sauvegarde et restauration | ⚠️ **partiel** | Voir 1.2 |
| 9 | Construction de production | ⚠️ **partiel** | Voir 1.3 |

### 1.1 Écarts de commande — le guide dit vrai, mais pas tout à fait

Trois commandes du guide ne fonctionnent pas telles quelles. Aucune n'est un défaut de
l'implémentation ; ce sont des précisions que le guide devra reprendre.

| Guide | Réel | Pourquoi |
|---|---|---|
| `cargo build --workspace` depuis la racine | depuis **`backend/`** | Le workspace Rust vit dans `backend/`, comme le fixe `plan.md` § Project Structure |
| `cargo test -p kaya-api --test isolation_tenant` | `-p kaya-backend` | Les tests transverses appartiennent au paquet racine `kaya-backend`, pas à `kaya-api` — chemins de `tasks.md` T045/T046 |
| `docker compose up` puis `cargo run` | **plus** `scripts/dev/preparer-base.sh` | Les mots de passe des rôles ne sont pas dans les migrations : un secret dans une migration est un secret dans l'historique Git, en clair, pour toujours |
| — | **plus** `scripts/dev/preparer-stockage.sh` | Un conteneur Garage qui démarre est **sain sans être utilisable** : nœud sans rôle, aucune clé, aucun compartiment |

Les deux derniers sont des **étapes supplémentaires à l'amorçage**, donc un écart réel à SC-001
(« moins de 30 minutes sur un poste neuf »). Elles restent largement dans le budget — quelques
secondes chacune — mais elles doivent figurer au guide, et elles y figurent désormais.

Le second a été **trouvé par la sonde elle-même**. `docker compose ps` rapportait Garage
`healthy`, ce qui était exact : le service répondait. `/health`, qui tente un appel S3 réel,
rapportait `degrade`. C'est exactement la différence que la sonde sert à faire, et la meilleure
démonstration de la raison pour laquelle elle ne lit pas l'état d'un pool.

### 1.2 §8 — Sauvegarde et restauration : **exercice non réalisé**

Les deux scripts sont écrits, exécutables, et leurs garde-fous ont été **déclenchés** pour
vérifier qu'ils refusent bien : dump suspect sous 1 Kio, restauration par-dessus la base de
production, variable requise absente.

**L'exercice de restauration complet en environnement vierge (T053) n'a pas eu lieu.** Il lui
manque trois éléments qui ne relèvent pas du code :

1. un **fournisseur de stockage tiers** provisionné — R-13 arrête l'invariant (hôte distinct,
   verrouillage d'objet, rétention verrouillée) et laisse explicitement le nom du fournisseur à
   trancher ;
2. une **paire de clés `age`** de production, déposée au coffre en deux exemplaires ;
3. un **serveur vierge** pour restaurer.

**Conséquence** : FR-060 n'est pas satisfaite et SC-006 reste ouvert. Consigné dans
`infra/backup/README.md` §6.

### 1.3 §9 — Construction de production : Dockerfile écrit, image `amd64` non produite

`infra/Dockerfile.api` est écrit et ses deux images de base sont vérifiées sur le registre
(`rust:1.97.1-trixie`, `debian:trixie-slim`). Il construit **sans base de données**, grâce à
`backend/.sqlx` et `SQLX_OFFLINE=true`.

L'image `linux/amd64` n'a pas été produite : sur un poste Apple Silicon, la construction croisée
passe par l'émulation et la mesure obtenue ne dirait rien de la production. **SC-010 — le temps de
compilation incrémentale mesuré dans le conteneur Linux — reste donc non mesuré**, ce que R-01
annonçait déjà : « une mesure sur le poste macOS ne prédit rien de la CI ».

Une construction **native `arm64`** a été lancée pour valider le Dockerfile lui-même. Elle
dépassait **27 minutes** sans avoir abouti au moment de la revue — compilation `--release` de
l'ensemble du workspace, dans un conteneur, avec les entrées-sorties de Docker Desktop sur macOS.
**Le Dockerfile n'est donc pas validé de bout en bout**, seulement écrit et vérifié statiquement :
ses deux images de base existent au registre, et `SQLX_OFFLINE=true` avec `backend/.sqlx` lui
permet de compiler sans base de données.

**Ce qui reste dû** : une construction complète, en CI Linux où elle est native et mise en cache,
avant le premier déploiement.

---

## 2. T091 — Definition of Done, les dix points

`docs/user-stories-v1.md` §0.4.

| # | Point | État | Détail |
|---|---|---|---|
| 1 | Critères couverts par tests unitaires **et** d'intégration | ✅ | 37 tests d'intégration sur base réelle, 18 côté application |
| 2 | Annotations utoipa à jour, client régénéré sans diff | ✅ | Porte P-01 verte, déterminisme et ordre stable vérifiés |
| 3 | Migration versionnée, `cargo sqlx prepare` vert, seeds à jour | ✅ | 6 migrations, `.sqlx` commité, seeds rejouables |
| 4 | RLS activée et forcée, test d'isolation | ✅ | 6 tables, portes P-07 et P-08 |
| 5 | Classe hors-ligne déclarée avec ses tests | ✅ | `note_etablissement` en A, rejeu et désordre |
| 6 | Événement outbox pour tout changement d'état | ✅ | Porte P-05, test de rollback |
| 7 | Clés i18n fr et en, aucune chaîne en dur | ✅ | Porte P-16, parité vérifiée |
| 8 | Écran vérifié en clair **et** en sombre | ⬜ **sans objet** | **Ce cycle ne produit aucun écran** — voir 2.1 |
| 9 | Paramètres en configuration d'établissement | ✅ | Aucun paramètre métier introduit — voir 2.2 |
| 10 | Document imprimé vérifié sur imprimante thermique | ⬜ **sans objet** | Aucun document produit à ce cycle |

### 2.1 Point 8 — pourquoi « sans objet » est une décision, pas une facilité

L'écran de notes internes n'hérite d'aucun motif :

- absent des **onze codes** maquettés de `docs/design/html/` — `C4`, `F2`, `G2`, `M4`, `P2`, `Q1`,
  `R1`, `R4`, `R7`, `S2`, `V1` ;
- absent de la matrice de dérivation `docs/design/derivation.md`.

« Un écran qui n'hérite d'aucun motif ne se code pas » (principe XII). La couche écran du module
doré est donc reportée au cycle ETB, qui dispose d'écrans réellement maquettés.

**Ce que le patron ne démontre pas, et qui reste dû au cycle ETB** : i18n en situation, mode
sombre, RBAC, chargement paresseux. Les fondations des quatre sont livrées ici. Écrit dans
`docs/module-dore.md`.

### 2.2 Point 9 — aucun paramètre métier n'a été introduit

Vérifié plutôt que supposé. Les seules valeurs configurables du cycle sont des **paramètres
d'exploitation** — port, chaînes de connexion, DSN Sentry, montage de Swagger UI — et non des
paramètres métier au sens du principe I(c). Le récapitulatif de `docs/user-stories-v1.md` n'a donc
pas à être modifié.

Les barèmes et tarifs qui viendront sont décrits dans `backend/migrations/seeds/README.md`,
**marqués provisoires tant que B-07 n'est pas tranchée**.

---

## 3. Checklist de conformité demandée à l'implémentation

| # | Point | État |
|---|---|---|
| 1 | Critères d'acceptation couverts par des tests | ✅ |
| 2 | utoipa à jour ; client TS régénéré, aucun diff | ✅ |
| 3 | Migrations versionnées ; `sqlx prepare` vert ; seeds à jour | ✅ |
| 4 | RLS `ENABLE` + `FORCE` sur chaque table ; test d'isolation | ✅ |
| 5 | Classe offline déclarée + tests du §0.7 | ✅ |
| 6 | Événements outbox pour chaque transition d'état | ✅ |
| 7 | Aucune chaîne en dur ; clés fr **et** en | ✅ |
| 8 | Chaque écran vérifié en clair et en sombre | ⬜ **sans objet — aucun écran** |
| 9 | Aucun paramètre métier en dur | ✅ |
| 10 | Montants entiers + devise ; aucune règle fiscale hors `JurisdictionAdapter` | ✅ |
| 11 | Aucun `window.__TAURI__` hors `PlatformAdapter` | ✅ ⚠️ une limite, voir 4.3 |
| 12 | Aucune jointure SQL entre schémas de modules | ✅ ⚠️ heuristique, voir 4.1 |
| 13 | Chaque écran a sa référence | ⬜ **sans objet — aucun écran** |
| 14 | Aucun bloc de `docs/design/html/` copié ; valeurs conformes aux jetons | ✅ |
| 15 | Styles en utilitaires Tailwind du noyau | ✅ — aucun CSS explicite écrit |
| 16 | Mode sombre par la variante `dark:` | ✅ |
| 17 | Aucun terme technique sans entrée au lexique | ✅ — « Note interne » ajoutée |
| 18 | Rien construit au-delà du périmètre | ✅ — provisions en tables seulement, vérifié par test |
| 19 | Quantités en `NUMERIC`, jamais en entier | ✅ — porte P-10 ; aucune quantité persistée à ce cycle |
| 20 | Aucune dépendance en intervalle ; lockfiles commités | ✅ — porte P-20 |
| 21 | Chaque porte concernée vérifiée par un test qui échoue vraiment | ✅ — **14 portes cassées volontairement**, voir §5 |

---

## 4. Ce qui reste non conforme, incomplet ou limité

**Rien de ce qui suit n'est masqué par une case cochée.**

### 4.1 Portes dont la garantie est partielle — limites écrites dans le code

| Porte | Limite | Où elle est écrite |
|---|---|---|
| **P-04** | Heuristique : pas d'analyse syntaxique du SQL. Manque une jointure dynamique ou cachée derrière une vue ; peut signaler un `UNION` | En-tête de `scripts/ci/jointures-inter-schemas.sh`, en encadré |
| **Registre hors-ligne** | Vérifie la **présence** d'une entité, jamais la **justesse** de sa classe — le registre classe des opérations, `encaissement` y figure deux fois | En-tête de `backend/tests/classes_offline.rs` |
| **P-16** | Détection des littéraux heuristique | En-tête de `app/scripts/test-i18n.ts`, et rappelé à chaque exécution |
| **P-15** | `(window as any).__TAURI__` échappe à `no-restricted-properties` | Commentaire de la règle dans `app/eslint.config.js` |

Ces quatre limites sont couvertes par la **revue mensuelle** (constitution, § Revue).

### 4.2 Trois portes installées à vide

**P-06** (ETB-02b), **P-09** (HEB-02) et **P-11** (T3) n'ont aucune cible à ce cycle. Elles portent
chacune une **assertion de non-régression** : elles échouent dès que leur cible apparaît sans
qu'elles soient activées. `backend/tests/portes_a_vide.rs`.

P-09 est **partiellement exercée** : `fiscalite.exercice_comptable` utilise `EXCLUDE USING gist`
sur `daterange`, ce qui valide `btree_gist` et le mapping sqlx 0.9 avant que HEB-02 n'en dépende
sur `tstzrange`.

### 4.3 Une dérogation ouverte, nommée et datée

| Élément | Valeur |
|---|---|
| Nom | `CONTEXTE_PAR_EN_TETES` |
| Ouverte le | 2026-07-31 |
| Ce qu'elle permet | Le tenant et le compte se lisent dans les en-têtes `x-kaya-tenant` et `x-kaya-compte` |
| Risque | **Toute personne joignant l'API choisit son tenant.** Inacceptable en production |
| Garde-fou | Le binaire **refuse de démarrer** sans `KAYA_CONTEXTE_PAR_EN_TETES=1` |
| Condition de levée | **CPT-01** — le contexte vient du jeton vérifié, les en-têtes disparaissent |

Sans elle, la porte P-08 n'aurait aucun moyen de se présenter comme deux tenants différents, et
elle aurait attendu CPT-01 — trois cycles auraient écrit des endpoints sans qu'elle les voie.

### 4.4 Deux tâches non exécutées

| Tâche | Ce qui manque | Effet |
|---|---|---|
| **T053** — exercice de restauration | Stockage tiers provisionné, clés `age` de production, serveur vierge | **FR-060 non satisfaite, SC-006 ouvert** |
| **T054** — supervision externe | Nom de domaine et serveur de production | Spécifiée (cibles, seuils, cadence) mais non provisionnée. Aucune dépendance de code : `/health` est public et sans authentification |

### 4.5 Écarts de numérotation de migration

`data-model.md` §7 et `tasks.md` T064 attribuaient **0005** aux provisions comptables. La
migration `0005_role_worker_publication.sql` s'est intercalée : la Phase 4 précède la Phase 10, et
sqlx refuse une version antérieure à une version déjà appliquée. Les provisions portent donc
**0006**, contenu inchangé. Consigné en tête des deux fichiers.

### 4.6 Un quatrième rôle PostgreSQL, non prévu par R-04

`kaya_worker`. `FORCE ROW LEVEL SECURITY` empêche le worker de balayer les tenants : sous aucun
des trois rôles prévus il ne voit quoi que ce soit — **et un worker qui ne voit rien ne publie
rien, en silence**.

Écartés : `BYPASSRLS` (vaut pour toutes les tables, bien au-delà du besoin) et l'itération par
tenant (déplace le problème sur la lecture de `etablissements.tenant`). Retenu : un rôle de
service, une politique nommée, une seule table, aucun autre droit. L'immuabilité est vérifiée
**sous ce rôle aussi** — c'est celui qui voit le plus, donc celui où elle serait le plus facilement
perdue.

### 4.7 Extensions du gel introduites par ce cycle

Six crates Rust nécessaires ne figuraient pas au gel §3.1 : `serde_json` 1.0.151, `time` 0.3.54,
`thiserror` 2.0.19, `async-trait` 0.1.91, `futures` 0.3.33, `dotenvy` 0.15.7. Plus, côté
JavaScript : `@eslint/js` 10.0.1, `typescript-eslint` 8.65.0, `eslint-plugin-vue` 10.10.0,
`@tailwindcss/vite` 4.3.3, `vitest` 4.1.10, `eslint` 10.8.0.

Toutes épinglées exactement et vérifiées sur leur registre le 2026-07-30/31. **À porter au gel à
la revue mensuelle du 2026-08-31.**

### 4.8 Correction du gel : TypeScript reculé de 7.0.2 à 5.9.3

Le gel 1.0.3 épinglait la dernière stable. `openapi-typescript` 7.13.0 déclare
`peerDependencies: { typescript: "^5.x" }`, et TypeScript 7 a modifié l'API `ts.factory` : la
génération échoue immédiatement, donc **P-01 ne pouvait pas s'exécuter**.

Gel corrigé en **1.0.4**. La leçon est écrite au journal : « dernière version stable » suppose que
les versions sont compatibles entre elles. Le §3.1 vérifiait déjà la compatibilité pour les crates
Rust ; le §3.2 ne le faisait pas pour npm.

### 4.9 `sccache` activé par l'environnement, non en dur (écart à R-02)

R-02 retient `sccache` sur les deux plateformes. Il n'est **pas** écrit dans
`backend/.cargo/config.toml` : `rustc-wrapper` est résolu à chaque invocation de cargo, et sur un
poste neuf où `sccache` n'est pas installé, la ligne ferait échouer `cargo build` avant la première
compilation — le scénario exact que SC-001 doit garantir. Il s'active par `RUSTC_WRAPPER`, et
l'image de CI le positionne.

---

## 5. Les 21 portes — état et vérification

**Chaque porte active a été mise en échec volontairement au moins une fois, puis remise au vert.**
Une porte dont on n'a jamais constaté l'échec n'est pas une porte, c'est une intention.

| Porte | État | Cassée volontairement par |
|---|---|---|
| P-01 | ✅ active | Client non régénéré après ajout d'un endpoint |
| P-02 | ✅ active | Ligne ajoutée à `0001_roles_et_schemas.sql` après application |
| P-03 | ✅ active | Arête `socle/comptes → verticales/hebergement` |
| P-04 | ✅ active ⚠️ heuristique | Requête joignant `etablissements` et `synchronisation` |
| P-05 | ✅ active | Rollback provoqué — ni ligne ni événement |
| P-05b | ✅ active | `DELETE` sur `evenement_outbox` + mention d'une rétention |
| P-06 | ⬜ à vide | Assertion de non-régression sur `etablissements.capacite` |
| P-07 | ✅ active | Table sans politique — les 3 conditions séparément |
| P-08 | ✅ active | Route montée sans régime déclaré — **défaut réel trouvé** (voir §6) |
| P-09 | ⬜ à vide, partiellement exercée | `exercice_comptable` en `EXCLUDE USING gist` |
| P-10 | ✅ active | `quantite INTEGER` + `montant_mineur DOUBLE PRECISION` |
| P-11 | ⬜ à vide | Assertion sur `tests/fixtures/fiscal/` |
| P-12 | ✅ active | `VentilationTaxes` référencé depuis `socle/caisse` |
| P-13 | ✅ active | `@ts-expect-error` retiré — le typecheck échoue |
| P-14 | ✅ active | Rejeu triple et désordre sur les six permutations |
| P-15 | ✅ active ⚠️ une limite | Import de `@tauri-apps/api` + `window.__TAURI__` |
| P-16 | ✅ active ⚠️ heuristique | Clé retirée de `en.json` + chaîne en dur dans un template |
| P-17 | ✅ active | `#1a1a1a` et `14px` dans un composant |
| P-18 | ✅ active | `cargo sqlx prepare --check` |
| P-19 | ✅ active | `R1-accueil.html` copié sous `app/assets/` |
| P-20 | ✅ active | `^4.14.0`, `redis:latest`, `channel = "stable"` |

**Bilan** : 18 actives, 3 installées à vide avec assertion de non-régression.

---

## 6. Deux défauts que les portes ont trouvés dans le code de ce cycle

Ils méritent d'être nommés : ce sont les deux seuls cas où une porte a signalé un vrai problème
plutôt que de confirmer une conformité.

### 6.1 P-08 lisait le mauvais contrat, et passait au vert à tort

`openapi::contrat()` ne renvoie que le squelette du `#[derive(OpenApi)]` — titre, étiquettes,
schéma d'authentification. **Les chemins sont collectés au montage des routes**, donc seulement par
`split_for_parts()`.

La porte, paramétrée sur le squelette, constatait **zéro route** et passait au vert alors que deux
endpoints étaient servis. Elle consomme désormais `application::contrat_complet()`.

**Une porte qui ne trouve jamais rien est indistinguable d'une porte qui n'a rien à trouver.**
C'est pourquoi chaque porte de ce cycle porte un test négatif.

### 6.2 Deux comportements pour la même situation

`lister()` répondait `200 []` là où `creer()` répond `404`, pour le même établissement hors du
tenant courant. Aucune fuite de données — mais deux réponses différentes sur le même chemin, pour
la même situation. Trouvé par le test croisé de P-08, corrigé.

---

## 7. Décisions ouvertes — état

| Décision | État | Effet sur ce cycle |
|---|---|---|
| **O-01** — `client`/`personne` en classe C | Ouverte | Aucun. À trancher avant SEJ-02 |
| **O-02** — classe de `mouvement_stock` | Ouverte | Aucun. À trancher avec le pilote |
| **O-03** — crate d'accueil de la surface QR | Ouverte | Aucun. À trancher avant QRC-01 |
| **B-01** — localisation de l'hébergement | Tranchée de fait | Serveur en Europe, pilote en Côte d'Ivoire : **transfert transfrontalier engagé dès le premier check-in**. À encadrer par TRX-06, consigné dans `docs/conformite/README.md` |
| **B-02** — fiscalité du passage et de la demi-journée | Ouverte | Aucune valeur en dur : aucune règle fiscale n'est écrite |
| **B-07** — barèmes | Ouverte | Les seeds à venir les portent **marqués provisoires** |

---

## 7 bis. La sonde de santé, constatée dans ses trois états

Trois observations successives, toutes réelles — c'est ce qui rend la sonde crédible :

| Observation | Réponse | Ce qu'elle démontre |
|---|---|---|
| Ni cache ni stockage démarrés | `503` · base `operationnel`, deux dépendances `degrade` | La sonde ne se fie pas au démarrage du service applicatif |
| Redis démarré, Garage non initialisé | `503` · **cache `operationnel`**, stockage `degrade` | `docker compose ps` rapportait Garage `healthy` — la sonde, elle, tente un **appel S3 réel** |
| Les trois configurés | `200` · **les trois `operationnel`** | Amorçage de SC-001 complet |

Le deuxième cas est celui qui compte. Un conteneur peut être sain sans être utilisable, et
c'est exactement l'écart qu'une sonde adossée à l'état d'un pool en mémoire ne verrait jamais.

---

## 8. Ce que le cycle livre — récapitulatif

- **17 paquets Rust**, hiérarchie du principe II tenue par les arêtes réelles du graphe
- **6 migrations**, 6 tables, 3 schémas, toutes en RLS `ENABLE` + `FORCE` + politique
- **4 rôles PostgreSQL**, dont un lecteur de grand livre qui ne peut rien lire d'autre
- **3 endpoints** au contrat, client TypeScript généré, déterminisme d'octet vérifié
- **Le module doré**, six couches écrites à la main contre sqlx 0.9, documenté dans
  `docs/module-dore.md`
- **37 tests d'intégration** sur base réelle + **18 tests** d'application
- **21 portes de CI**, dont 18 actives et toutes vérifiées en échec
- **0 écran** — décision vérifiée contre les onze maquettes et la matrice de dérivation
- **Deux scripts d'amorçage** — `preparer-base.sh` et `preparer-stockage.sh` — sans lesquels
  l'environnement démarre sans être utilisable
