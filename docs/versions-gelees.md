# Kaya — Versions gelées

*Application du principe XI de `.specify/memory/constitution.md` : dernières versions stables,
**vérifiées sur les registres officiels avec l'URL citée**, puis **épinglées exactement** et
figées par lockfiles.*

**Version du gel : 1.0.5 — vérifié le 2026-07-31**
**Prochaine revue : 2026-08-31** (revue mensuelle groupée)

**Cible de déploiement retenue : Docker sur VPS Contabo** (mode A du cadrage §10.1, SaaS
mutualisé). Toutes les versions ci-dessous sont vérifiées disponibles pour cette cible (§4.2).

**Un seul point reste ouvert** : le choix de sqlx `0.9.0` doit être **confirmé par le spike
GiST/`tstzrange`** de la phase 0 (cadrage §16). Tout le reste est arrêté.

---

## 1. Règles d'usage de ce document

1. **Aucun numéro de version de ce tableau n'a été écrit de mémoire.** Chaque ligne porte
   l'URL du registre officiel interrogé et la date de vérification.
2. **Épinglage exact obligatoire** : `= 4.14.0` ou `4.14.0`, jamais `^4.14`, `~4.14` ni `4.*`.
   La porte de CI **P-20** échoue sur tout intervalle et sur tout lockfile absent ou périmé.
3. **Aucune montée majeure pendant un incrément.** Une faille de sécurité est la seule
   exception, et elle est consignée au §6.
4. **La revue est mensuelle et groupée**, jamais au fil de l'eau. Elle met à jour ce fichier,
   les manifestes et les lockfiles dans un seul changement.
5. Reproduire la vérification : les commandes exactes sont au §5.

---

## 2. Les dix briques du principe XI

| # | Brique | Version gelée | Publiée le | Registre officiel interrogé |
|---|---|---|---|---|
| 1 | **Rust** (toolchain stable) | **1.97.1** | 2026-07-14 | `https://static.rust-lang.org/dist/channel-rust-stable.toml` — section `[pkg.rust]` |
| 2 | **Actix Web** | **4.14.0** | 2026-06-21 | `https://crates.io/api/v1/crates/actix-web` |
| 3 | **sqlx** | **0.9.0** ⚠️ | 2026-05-21 | `https://crates.io/api/v1/crates/sqlx` |
| 4 | **utoipa** | **5.5.0** | 2026-05-04 | `https://crates.io/api/v1/crates/utoipa` |
| 5 | **Nuxt** | **4.5.1** | — | `https://registry.npmjs.org/nuxt/latest` |
| 6 | **Tailwind CSS** | **4.3.3** | — | `https://registry.npmjs.org/tailwindcss/latest` |
| 7 | **Tauri** (crate) | **2.11.5** | 2026-07-01 | `https://crates.io/api/v1/crates/tauri` |
| 8 | **PostgreSQL** | **18.4** | 2026-05-14 | `https://www.postgresql.org/versions.json` |
| 9 | **Redis** | **8.8.1** | 2026-07-23 | `https://api.github.com/repos/redis/redis/releases` |
| 10 | **Garage** | **2.3.0** | 2026-04-16 | `https://git.deuxfleurs.fr/api/v1/repos/Deuxfleurs/garage/releases` |

### Arbitrages du gel

Le critère retenu n'est pas l'âge d'une version, c'est **son coût en terrain non défriché pour
un développeur solo**. Une version fraîche est acceptée quand elle ne change rien au code ou
qu'elle apporte quelque chose de nécessaire ; elle est refusée quand elle expose sans gain.

#### sqlx 0.9.0 — retenue, pour deux apports propres au projet

La stable précédente était `0.8.6` (2025-05-19), soit un an d'écart. Deux changements de
`0.9.0` visent directement l'architecture Kaya :

- **`#3918` — type d'erreur dédié à la violation de contrainte d'exclusion.** C'est le cœur de
  HEB-02 : deux attributions concurrentes chevauchantes doivent produire « unité déjà occupée sur
  cet intervalle », pas une erreur SQL brute. En `0.8.6`, il faut inspecter le SQLSTATE `23P01`
  à la main.
- **`sqlx.toml` avec exemple officiel multi-tenant** : renommage de `_sqlx_migrations` et
  **plusieurs schémas** — exactement le « un schéma Postgres par module » du principe II, plus
  les surcharges de types pour les macros.

`PgRange<T>` est présent en `0.9.0` (vérifié sur `docs.rs/sqlx/0.9.0`), donc `tstzrange` reste
mappable.

**Coût assumé** : `#3723` impose `AssertSqlSafe` sur toute requête non littérale, et `#3541`
peut altérer la sortie des macros `query!()`. La documentation, les exemples et les réponses en
ligne visent encore `0.8.x` — **tout extrait trouvé en ligne ne compilera pas**. C'est ce que le
**module doré** (cadrage §13.1) neutralise : il doit être écrit contre `0.9.0` **avant** toute
génération assistée, sinon chaque cycle réintroduira des appels `0.8`.

*Note de gouvernance* : sqlx est passé à l'organisation GitHub `transact-rs` et ne suit plus son
`Cargo.lock`. Transition saine (propriété collective formalisée par les auteurs principaux),
mais liens et outils tiers mettront du temps à s'aligner.

#### Redis — reculé de 8.10.0 à 8.8.1

`8.10.0` est passée en GA le 2026-07-29 après des RC datées du **2026-07-20** : neuf jours de
release candidate pour une mineure qui introduit *compact hashes*, un nouvel encodage de
hachage. Kaya n'en a aucun besoin — sessions, file FNE, verrous, limitation de débit et cache
fonctionnent depuis Redis 6.

**Aucun sacrifice de sécurité** : la salve du 2026-07-23 était un correctif de sécurité sur
**toutes** les branches maintenues (6.2 à 8.8) — use-after-free via payload `RESTORE` de stream,
écriture hors limites dans RedisBloom/TDigest. `8.8.1` porte ces correctifs et sa branche a deux
mois de recul (`8.8.0` du 2026-05-25).

**Toute version retenue doit être ≥ à la salve du 2026-07-23.** Reculer davantage exposerait à
ces failles.

#### PostgreSQL 18.4 — retenue, arbitrage fermé

`18.4` n'est pas une version fraîche : PG 18 est *current* depuis septembre 2025 et un `.4`
signifie trois cycles de correctifs passés. Kaya n'utilise aucune fonctionnalité propre à PG 18
— RLS, `EXCLUDE USING gist`, `tstzrange` et `NUMERIC` existent depuis PG 10 — donc le seul
critère était **où la base tourne**.

**Réponse : Docker sur un VPS Contabo auto-géré.** La version de PostgreSQL est entièrement
maîtrisée, sans plafonnement d'offre managée. `18.4` est donc retenue pour son **EOL au
2030-11-14**, la plus longue durée de vie disponible — ce qui compte pour un produit dont les
documents fiscaux sont conservés 10 ans.

L'alternative `17.10` (22 mois de recul, EOL 2029-11-08) reste valable pour le **paquet
auto-hébergé** (mode B) si un client ne sait administrer que PG 17. Les deux tags existent en
multi-architecture, la bascule est un changement de tag.

---

## 3. Dépendances directes du socle

Épinglées au même titre. Ce ne sont pas des « briques » au sens du principe XI, mais elles
entrent dans le lockfile et la porte P-20 les couvre.

### 3.1 Écosystème Rust

| Crate | Version | Rôle | Contrainte vérifiée |
|---|---|---|---|
| `utoipa-swagger-ui` | **9.0.2** | Swagger UI, protégée hors production | dépend de `utoipa ^5` et `actix-web ^4` → **compatible** |
| `utoipa-actix-web` | **0.1.2** | Intégration utoipa ↔ Actix | dépend de `utoipa ^5`, `actix-web ^4` → **compatible** |
| `tauri-build` | **2.6.3** | Build de la coquille Tauri | aligné sur `tauri` 2.11.x |
| `tokio` | **1.53.1** | Runtime asynchrone | — |
| `serde` | **1.0.229** | Sérialisation | — |
| `uuid` | **1.24.0** | **UUID v7 côté client** (principe VI) | feature `v7` **présente et vérifiée** |
| `rust_decimal` | **1.42.1** | Quantités `NUMERIC` (principe V) | jamais de flottant sur une quantité |
| `redis` | **1.5.0** | Client Redis | — |
| `aws-sdk-s3` | **1.140.0** | Accès Garage **via API S3** (principe II) | — |
| `sentry` | **0.49.0** | Rapport d'erreurs (principe VIII) | — |
| `tracing` | **0.1.44** | Logs structurés corrélés (principe VIII) | — |
| `tracing-subscriber` | **0.3.23** | Souscripteur de logs | — |
| `jsonwebtoken` | **11.0.0** | JWT court + refresh révocable (CPT-01) | — |
| `argon2` | **0.5.3** | Hachage de mot de passe (CPT-01) | — |

> **`uuid` avec la feature `v7` est un prérequis du principe VI**, pas un détail : toute
> écriture porte un UUID v7 généré côté client. La présence de la feature dans `1.24.0` a été
> vérifiée sur l'API crates.io, pas supposée.

### 3.2 Écosystème JavaScript

| Paquet | Version | Rôle |
|---|---|---|
| `@tauri-apps/cli` | **2.11.4** | CLI Tauri (build Android/iOS/desktop) |
| `@tauri-apps/api` | **2.11.1** | Pont JS ↔ Rust — **consommé uniquement par `PlatformAdapter`** (principe VII) |
| `@nuxtjs/i18n` | **10.6.0** | i18n fr/en, fr par défaut (principe VIII) |
| `openapi-typescript` | **7.13.0** | **Génère les types TS depuis `openapi.json`** — le seul artefact généré (principe I·a, porte P-01) |
| `openapi-fetch` | **0.17.0** | Client fetch typé, ~6 kB — **écrit à la main, jamais généré** |
| `typescript` | **5.9.3** ⚠️ | `peerDependency` de `openapi-typescript` — **dernière 5.x, pas la dernière stable** |
| `vitest` | **4.1.10** | Tests de l'application et des surfaces web |
| `eslint` | **10.8.0** | Lint — porte **P-15** (`window.__TAURI__` hors PlatformAdapter) |
| `@eslint/js` | **10.0.1** | Configuration de base d'eslint |
| `eslint-plugin-vue` | **10.10.0** | Règles Vue |
| `typescript-eslint` | **8.65.0** | Règles TypeScript |
| `@tailwindcss/vite` | **4.3.3** | Greffon Vite de Tailwind 4 — aligné sur `tailwindcss` |
| `@vue/test-utils` | **2.4.11** | Montage de composants Vue en test — **SC-005** : aucun service inactif dans le HTML rendu |
| `happy-dom` | **20.11.1** | Environnement DOM de Vitest, requis par le montage ci-dessus |
| `@vitejs/plugin-vue` | **6.0.8** | Compile les composants monofichiers pour Vitest **hors Nuxt** — sans lui, `@vue/test-utils` ne peut monter aucun `.vue` |
| `@types/node` | **24.13.3** ⚠️ | Types du runtime — **dernière `24.x`, alignée sur Node `24.18.1`**, pas la dernière stable |

> **`@types/node` — dette du cycle 001, réparée au gel 1.0.6.** `app/tsconfig.test.json` typait
> déjà `scripts/**/*.ts`, mais aucun paquet ne fournissait les types de `node:fs` et `node:path` :
> `pnpm test` sortait en **échec permanent** sur six `TypeCheckError`, avec dix-huit tests pourtant
> verts. Un `pnpm test` rouge en permanence est un `pnpm test` que personne ne lit — et les deux
> fichiers non typés sont ceux des portes **P-16** et **P-17**.
>
> **`24.13.3`, pas `26.1.2`.** Les types du runtime suivent la ligne majeure du runtime : Node est
> gelé en `24.18.1` LTS (§3.3), donc la dernière `24.x` est la seule valeur cohérente. Même
> dérogation raisonnée au « dernière stable » que Node lui-même. Vérifiée sur
> `https://registry.npmjs.org/@types/node` le 2026-07-31 (`dist-tags.latest` = `26.1.2`, dernière
> `24.x` = `24.13.3`, publiée le 2026-07-08). **Condition de suivi** : toute montée de Node au gel
> §3.3 impose de remonter ce paquet à la même majeure.

> **Trois paquets ajoutés au gel 1.0.6 — la décision ouverte du cycle 002 (T004), tranchée.**
> `plan.md` la laissait ouverte entre *ajouter* et *refuser*, sans proposer de version
> (principe XI). **Ajout retenu** : SC-005 exige de constater qu'aucun libellé ni code de service
> inactif n'apparaît **dans le HTML rendu** de `G1`. Le vérifier sur la seule fonction de sélection
> testerait l'intention, pas le résultat — or « un service inactif est **absent**, jamais grisé »
> (principe VII) est une garantie de rendu, et c'est exactement le genre de propriété qu'un
> composant peut perdre sans que sa fonction de sélection change.
>
> Le troisième paquet n'était pas prévu par le plan, qui n'en annonçait que deux. `@vue/test-utils`
> monte un composant **déjà compilé** ; hors du pipeline Nuxt, rien ne compile un fichier `.vue`
> pour Vitest. `@vitejs/plugin-vue` est donc une dépendance technique du choix, pas un ajout de
> confort — signalé plutôt que glissé dans le lot.
>
> Vérifiés sur `https://registry.npmjs.org/` le 2026-07-31 : `@vue/test-utils` **2.4.11**
> (2026-06-04, `peerDependencies: { vue: "3.x" }` — satisfaite par le Vue 3 de Nuxt 4.5.1),
> `happy-dom` **20.11.1** (2026-07-22, aucune `peerDependency`), `@vitejs/plugin-vue` **6.0.8**
> (2026-07-14, `peerDependencies: { vue: "^3.2.25", vite: "^5 || ^6 || ^7 || ^8" }` — satisfaite
> par le Vite de Nuxt 4.5.1). Les trois sont à la dernière stable.

> **Six paquets ajoutés au gel 1.0.5.** Ils étaient déclarés dans `app/package.json` depuis le
> cycle 001 **sans figurer au gel** — donc épinglés dans la bonne forme, mais adossés à aucune
> décision tracée. Écart relevé par l'analyse du cycle 002 (T004), pas par la porte P-20 : c'est
> précisément le trou décrit au **§4.3**. Les six ont été vérifiés sur le registre npm au
> 2026-07-31 et sont chacun à la dernière stable — les valeurs du dépôt sont donc confirmées, non
> corrigées.

#### Génération du client TypeScript — ajoutée au gel 1.0.3

Le gel initial ne portait **aucun générateur**, ce qui rendait la porte **P-01** inapplicable :
sans générateur, pas de client régénéré, donc pas de diff à comparer. Lacune comblée.

**Le choix repose sur une séparation, pas sur un outil** : `openapi-typescript` produit
**uniquement un fichier de types**, dérivé mécaniquement du contrat ; `openapi-fetch` est une
bibliothèque runtime **installée, jamais générée**. L'unique artefact soumis à P-01 est donc un
fichier de types, sans code d'exécution — ce qui réduit la surface de diff au strict dérivé du
contrat. Un générateur de SDK complet (`@hey-api/openapi-ts`, `orval`) produirait des fichiers
de client à chaque exécution, multipliant les occasions de faux positif.

Écartés, avec le motif : `@hey-api/openapi-ts` **0.99.0** est encore en `0.x`, donc à API
instable par convention sémantique ; `orval` **8.23.0** génère des couches de requêtes dont le
projet n'a pas besoin ; `oazapfts` **7.5.0** est un générateur de SDK, même objection que
Hey-API. Tous sont MIT et viables — le critère retenu est la **taille de la sortie générée**.

> ⚠️ **TypeScript reste en `5.9.3`, pas en `7.0.2`.** Le gel 1.0.3 avait épinglé `7.0.2` parce
> que c'était `latest` sur npm — **erreur de vérification** : `openapi-typescript` 7.13.0 déclare
> `peerDependencies: { typescript: "^5.x" }`. La valeur `7.0.2` violait donc la contrainte du
> paquet gelé dans le même mouvement. `5.9.3` est la dernière `5.x` (2025-09-30, dix mois de
> recul) et la seule qui satisfasse `^5.x`.
>
> Au passage, `7.0.2` était de toute façon le mauvais choix au regard du critère du §2 : la
> branche 7 est la réécriture en Go, une refonte majeure sans aucun gain pour ce projet, et
> l'outillage autour — eslint, vitest, typescript-eslint — vise encore `5.x`. Nuxt 4.5.1 embarque
> `6.0.3` pour son usage interne, ce qui ne contraint pas le projet mais montre un écosystème
> éclaté entre trois branches majeures. **Leçon retenue : vérifier les `peerDependencies` d'un
> paquet gelé, pas seulement son numéro** — le même contrôle avait bien été fait côté Rust pour
> les satellites d'utoipa.

Deux exigences que l'outil doit satisfaire, **à valider au cycle 1 avant de clore US5** :

1. **Déterminisme d'octet.** Deux exécutions successives sur le même `openapi.json` DOIVENT
   produire deux fichiers identiques. À vérifier par `cmp`, pas par lecture. Sans cette
   propriété, P-01 échoue au hasard et sera désactivée sous trois semaines.
2. **Ordre stable des membres**, indépendant de l'ordre de découverte des routes par utoipa.
   À vérifier en ajoutant un endpoint en fin de fichier Rust et en constatant que le diff
   généré reste local.

`openapi-fetch` pèse ~6 kB, ce qui compte : la persona Aminata travaille sur un Android
d'entrée de gamme en réseau intermittent.

#### TypeScript reculé de 7.0.2 à 5.9.3 — corrigé au gel 1.0.4

Le gel 1.0.3 avait retenu `typescript` **7.0.2**, dernière stable au registre npm. La combinaison
**ne fonctionne pas** : `openapi-typescript` 7.13.0 déclare `peerDependencies: { "typescript":
"^5.x" }`, et TypeScript 7 — la réimplémentation native — a modifié l'API `ts.factory` sur
laquelle le générateur s'appuie. L'exécution échoue immédiatement :

```
TypeError: Cannot read properties of undefined (reading 'createKeywordTypeNode')
    at openapi-typescript/dist/lib/ts.mjs:11:28
```

**Ce que l'erreur du gel 1.0.3 apprend** : la règle « dernière version stable » du principe XI
suppose que les versions sont compatibles entre elles. Elle ne remplace pas la vérification de
compatibilité, que le §3.1 pratique déjà pour les crates Rust (colonne « Contrainte vérifiée »).
La même colonne manquait au §3.2 ; l'écart est comblé ici.

`5.9.3` est la dernière `5.x`, vérifiée sur `https://registry.npmjs.org/typescript` le
2026-07-31 (`dist-tags.latest` = `7.0.2`, dernière `5.x` = `5.9.3`). C'est une **dérogation
raisonnée** au « dernière stable », de même nature que celle de Node LTS : la contrainte d'un
outil prime sur la fraîcheur.

**Condition de levée** : `openapi-typescript` publie une version déclarant `typescript ^7`. À
vérifier à chaque revue mensuelle, sur `peerDependencies` — pas sur le numéro de version de
l'outil, qui peut monter sans changer sa contrainte.

> Le décalage `tauri` 2.11.5 (crate) / `@tauri-apps/cli` 2.11.4 est normal : les deux
> versionnements sont indépendants dans la même branche 2.11.x.

### 3.3 Environnement d'exécution

Absent de la liste du principe XI, mais Nuxt ne s'exécute pas sans. Ajouté au gel pour que la
reconstruction soit reproductible.

| Brique | Version | Publiée le | Registre |
|---|---|---|---|
| **Node.js** | **24.18.1** (LTS « Krypton ») | 2026-07-28 | `https://nodejs.org/dist/index.json` |
| **pnpm** | **11.18.0** | — | `https://registry.npmjs.org/pnpm/latest` |

> **LTS, pas la dernière stable.** Node 26.5.1 existe (2026-07-28) mais n'est pas LTS. Pour une
> chaîne de build qui doit tenir 15 mois sans surprise, la LTS est le bon choix ; c'est une
> dérogation raisonnée au « dernière stable » du principe XI, consignée ici comme telle.
> À figer par `.nvmrc` **et** par le champ `engines` du `package.json`.

---

## 4. Où l'épinglage est matérialisé

### 4.1 Fichiers du dépôt

| Fichier | Porte | Contenu attendu |
|---|---|---|
| `rust-toolchain.toml` | P-20 | `channel = "1.97.1"` — jamais `stable` |
| `Cargo.toml` (workspace) | P-20 | `[workspace.dependencies]` en versions exactes, héritées par tous les crates |
| `Cargo.lock` | P-20 | **Commité**, y compris pour les binaires |
| `package.json` | P-20 | Versions exactes, sans `^` ni `~` ; `engines.node` |
| `pnpm-lock.yaml` | P-20 | **Commité** |
| `.nvmrc` | P-20 | `24.18.1` |
| `compose.yml` | P-20 | Tags d'image exacts du §4.2 — **jamais `latest`** |

> **`Cargo.lock` est commité même pour un binaire** : c'est ce qui rend la reconstruction
> identique à six mois d'écart, condition du support à distance du parc auto-hébergé
> (cadrage §10.2).

### 4.2 Images Docker — disponibilité vérifiée le 2026-07-30

| Service | Tag exact | Publié le | Architectures |
|---|---|---|---|
| PostgreSQL | `postgres:18.4` | 2026-07-19 | 386, **amd64**, arm, **arm64**, ppc64le, riscv64, s390x |
| Redis | `redis:8.8.1` | 2026-07-25 | 386, **amd64**, arm, **arm64**, ppc64le, riscv64, s390x |
| Garage | `dxflrs/garage:v2.3.0` | 2026-04-16 | 386, **amd64**, arm, **arm64** |

Les trois images sont **multi-architecture**, donc le même `compose.yml` fonctionne en
développement sur poste Apple Silicon (`arm64`) et en production sur VPS Contabo (`amd64`).

> **Piège de `dxflrs/garage` à connaître** : le dépôt publie en continu des tags de **hash de
> commit**, qui noient les tags sémantiques dans tout listing trié par date. Toujours interroger
> par nom (`?name=v2.3.0`), jamais lire la première page de tags.

> **Le binaire Rust, lui, n'est pas multi-architecture.** Un `cargo build` sur poste Apple
> Silicon produit un binaire `aarch64-apple-darwin` non déployable sur le VPS. La construction de
> production se fait **dans Docker pour `linux/amd64`** (build multi-étapes), jamais par copie
> d'un binaire construit localement. Corollaire : les mesures de performance faites sur le poste
> de développement ne prédisent pas celles de la production.

---

### 4.3 Ce que la porte P-20 ne vérifie pas — complément à écrire

`scripts/ci/versions-epinglees.sh` vérifie la **forme** : aucun intervalle, aucun `latest`, des
lockfiles suffisants. Il le documente lui-même et le motive bien — comparer les **valeurs** aux
registres officiels ferait de la CI une dépendance réseau.

**Mais le gel est un fichier du dépôt.** Comparer les manifestes à `docs/versions-gelees.md` ne
demande aucun réseau, et comble le trou qui a laissé passer `typescript 7.0.2` puis sa correction
silencieuse en `5.9.3` :

- lire les tableaux §2, §3.1, §3.2, §3.3 et §4.2 de ce fichier comme source ;
- comparer aux `Cargo.toml`, `package.json`, `compose.yml`, `.nvmrc`, `rust-toolchain.toml` ;
- **échouer sur tout écart**, dans les deux sens — une version du code absente du gel est aussi
  un défaut qu'une version du gel absente du code ;
- **et vérifier les `peerDependencies` des paquets gelés**, ce qui aurait attrapé la
  contradiction `openapi-typescript ^5.x` ↔ `typescript 7.0.2` à la source.

Sans ce complément, le gel est un document que rien n'oppose au code. C'est l'illustration
exacte de la leçon du cycle 1 : *un test négatif prouve qu'une porte sait échouer, il ne prouve
pas qu'elle regarde tout.*

## 5. Reproduire la vérification

À rejouer à chaque revue mensuelle. Aucune de ces commandes ne dépend d'un cache.

```sh
# Rust — canal stable officiel
curl -sS https://static.rust-lang.org/dist/channel-rust-stable.toml \
  | grep -A2 '^\[pkg\.rust\]'

# Crates Rust — crates.io exige un User-Agent
UA="kaya-version-check (angenor99@gmail.com)"
for c in actix-web sqlx utoipa utoipa-swagger-ui utoipa-actix-web tauri tauri-build \
         tokio serde uuid redis aws-sdk-s3 sentry tracing tracing-subscriber \
         jsonwebtoken argon2 rust_decimal; do
  printf "%-22s " "$c"
  curl -sS -H "User-Agent: $UA" "https://crates.io/api/v1/crates/$c" \
    | python3 -c "import sys,json;print(json.load(sys.stdin)['crate']['max_stable_version'])"
done

# Paquets npm
for p in nuxt tailwindcss @tauri-apps/cli @tauri-apps/api @nuxtjs/i18n pnpm \
         openapi-typescript openapi-fetch typescript; do
  printf "%-22s " "$p"
  curl -sS "https://registry.npmjs.org/$(echo $p | sed 's|/|%2F|')/latest" \
    | python3 -c "import sys,json;print(json.load(sys.stdin)['version'])"
done

# PostgreSQL — la version « current » est celle à retenir
curl -sS https://www.postgresql.org/versions.json \
  | python3 -c "import sys,json;[print(v['major']+'.'+v['latestMinor'],'EOL',v['eolDate']) for v in json.load(sys.stdin) if v['current']]"

# Redis
curl -sS https://api.github.com/repos/redis/redis/releases \
  | python3 -c "import sys,json;[print(r['tag_name'],r['published_at'][:10]) for r in json.load(sys.stdin)[:3] if not r['prerelease']]"

# Garage
curl -sS "https://git.deuxfleurs.fr/api/v1/repos/Deuxfleurs/garage/releases?limit=3" \
  | python3 -c "import sys,json;[print(r['tag_name'],r['published_at'][:10]) for r in json.load(sys.stdin) if not r['prerelease']]"

# Node.js — LTS, pas la dernière stable
curl -sS https://nodejs.org/dist/index.json \
  | python3 -c "import sys,json;v=[x for x in json.load(sys.stdin) if x['lts']][0];print(v['version'],v['lts'],v['date'])"

# Images Docker — TOUJOURS interroger par nom, jamais lire la 1re page de tags
for pair in "library/postgres:18.4" "library/redis:8.8.1" "dxflrs/garage:v2.3.0"; do
  repo="${pair%:*}"; tag="${pair#*:}"
  curl -sS "https://registry.hub.docker.com/v2/repositories/$repo/tags?name=$tag&page_size=5" \
    | python3 -c "
import sys,json
for t in json.load(sys.stdin)['results']:
    if t['name']=='$tag':
        a=sorted({i['architecture'] for i in (t.get('images') or []) if i.get('architecture')})
        print('$repo:$tag', t['last_updated'][:10], '|', ','.join(a)); break
else: print('$repo:$tag INTROUVABLE')"
done
```

**Compatibilité inter-crates** — à rejouer si `utoipa` monte de version majeure :

```sh
UA="kaya-version-check (angenor99@gmail.com)"
curl -sS -H "User-Agent: $UA" \
  "https://crates.io/api/v1/crates/utoipa-actix-web/0.1.2/dependencies" \
  | python3 -c "import sys,json;[print(d['crate_id'],d['req']) for d in json.load(sys.stdin)['dependencies'] if d['kind']=='normal']"
```

---

## 6. Journal des gels

| Version | Date | Modification |
|---|---|---|
| 1.0.6 | 2026-07-31 | **`@types/node` `24.13.3` inscrit — dette du cycle 001.** `app/tsconfig.test.json` typait `scripts/**/*.ts` sans qu'aucun paquet ne fournisse les types de `node:fs` / `node:path` : `pnpm test` échouait en permanence sur six `TypeCheckError`, alors que ses dix-huit tests passaient. Les deux fichiers non typés portent les portes **P-16** et **P-17**. Version alignée sur la ligne majeure du runtime gelé (Node `24.18.1` LTS), donc dernière `24.x` et non `latest` (`26.1.2`) — même dérogation raisonnée que Node. Vérifiée sur `https://registry.npmjs.org/@types/node` le 2026-07-31. Suivi : toute montée de Node au §3.3 impose la même montée ici. |
| 1.0.6 | 2026-07-31 | **Trois paquets de test front inscrits — décision T004 du cycle 002, tranchée dans le sens de l'ajout.** `@vue/test-utils` **2.4.11**, `happy-dom` **20.11.1**, `@vitejs/plugin-vue` **6.0.8**, vérifiés sur `https://registry.npmjs.org/` le 2026-07-31, `peerDependencies` contrôlées une par une contre Vue 3 / Vite de Nuxt 4.5.1. `plan.md` laissait le choix ouvert entre ajouter et refuser, sans version : refuser aurait réduit SC-005 à un test de la fonction de sélection, c'est-à-dire à vérifier l'intention plutôt que le HTML produit — or « un service inactif est absent, jamais grisé » est une propriété de rendu. **Le plan n'en annonçait que deux** : le troisième est la dépendance technique qui compile un `.vue` hors du pipeline Nuxt, signalée ici plutôt que glissée dans le lot. À reporter à la revue du 2026-08-31 comme les six du gel 1.0.5. |
| 1.0.4 | 2026-07-31 | **TypeScript reculé de `7.0.2` à `5.9.3`** — corrige une erreur du gel 1.0.3, constatée à l'exécution au cycle 001. `openapi-typescript` 7.13.0 déclare `peerDependencies: { typescript: "^5.x" }` et TypeScript 7 a modifié l'API `ts.factory` : la génération du client échoue sur `TypeError: Cannot read properties of undefined (reading 'createKeywordTypeNode')`, donc la porte **P-01** ne peut pas s'exécuter. `5.9.3` vérifiée sur `https://registry.npmjs.org/typescript` le 2026-07-31 comme dernière `5.x`. Dérogation raisonnée au « dernière stable », de même nature que Node LTS. Condition de levée : `openapi-typescript` déclare `typescript ^7`. **Leçon de gouvernance** : le §3.1 vérifiait la compatibilité inter-crates, le §3.2 ne le faisait pas pour les paquets npm — l'écart est comblé. |
| 1.0.5 | 2026-07-31 | **Six paquets JS inscrits au gel** — `vitest` 4.1.10, `eslint` 10.8.0, `@eslint/js` 10.0.1, `eslint-plugin-vue` 10.10.0, `typescript-eslint` 8.65.0, `@tailwindcss/vite` 4.3.3. Ils vivaient dans `app/package.json` depuis le cycle 001 sans décision tracée. Vérifiés sur npm : tous à la dernière stable, valeurs du dépôt confirmées. Écart trouvé par l'analyse du cycle 002, **pas par P-20** — le complément du §4.3 reste à écrire. |
| 1.0.4 | 2026-07-31 | **`typescript` corrigé de `7.0.2` à `5.9.3`** — le gel se contredisait : `openapi-typescript` 7.13.0 exige `peerDependencies: { typescript: "^5.x" }`, que `7.0.2` violait. Divergence relevée par l'implémentation du cycle 1, qui avait déclaré `5.9.3` dans les quatre manifestes JS — **la déviation était juste**. La porte P-20 ne l'a pas signalée parce qu'elle vérifie la **forme** (numéro exact) et non les **valeurs** ; complément à écrire, cf. §4.3. |
| 1.0.3 | 2026-07-30 | **Générateur de client TypeScript ajouté** — lacune du gel initial signalée par le plan du cycle 1 : la porte P-01 était inapplicable faute de générateur. Retenus : `openapi-typescript` **7.13.0** (types seulement) + `openapi-fetch` **0.17.0** (runtime écrit à la main) + `typescript` **7.0.2** (peerDependency). Critère de choix : minimiser la surface générée soumise à P-01. Écartés avec motif : `@hey-api/openapi-ts` 0.99.0 (`0.x`), `orval` 8.23.0, `oazapfts` 7.5.0. Deux exigences à valider au cycle 1 avant de clore US5 : déterminisme d'octet vérifié par `cmp`, et ordre de membres stable indépendant de l'ordre de découverte utoipa. |
| 1.0.2 | 2026-07-30 | **Cible de déploiement arrêtée : Docker sur VPS Contabo** (mode A). **PostgreSQL `18.4` confirmée, arbitrage fermé** — version maîtrisée en auto-géré, EOL 2030-11-14 retenu pour la conservation fiscale de 10 ans ; `17.10` reste l'option du paquet auto-hébergé (mode B). Ajout du §4.2 : les trois images Docker vérifiées disponibles en **amd64 et arm64**, donc un seul `compose.yml` pour le poste Apple Silicon et le VPS. Consigné : le binaire Rust n'est pas multi-architecture — construction de production **dans Docker pour `linux/amd64`**, jamais par copie locale. Consigné aussi : `dxflrs/garage` publie des tags de hash de commit qui masquent les tags sémantiques dans un tri par date — toujours interroger par nom. |
| 1.0.1 | 2026-07-30 | **Redis reculé de `8.10.0` à `8.8.1`** : `8.10.0` était en GA depuis un jour, avec neuf jours de RC, pour un nouvel encodage de hachage inutile à Kaya ; `8.8.1` porte les mêmes correctifs de sécurité du 2026-07-23 et deux mois de recul. **sqlx `0.9.0` confirmée** sur deux apports propres au projet (`#3918` erreur de violation d'exclusion pour HEB-02 ; `sqlx.toml` multi-schémas pour le principe II) et présence de `PgRange` vérifiée sur docs.rs. **PostgreSQL : arbitrage 18.4 / 17.10 ouvert**, rattaché à la décision B-01. Les neuf autres briques sont inchangées. |
| 1.0.0 | 2026-07-30 | Gel initial. 10 briques du principe XI + 14 crates + 3 paquets npm + Node LTS et pnpm. Compatibilité `utoipa-swagger-ui` / `utoipa-actix-web` avec `utoipa 5.5.0` vérifiée sur crates.io. Présence de la feature `uuid/v7` vérifiée. Deux points d'attention consignés : rupture d'API sqlx 0.8 → 0.9, et fraîcheur d'un jour de Redis 8.10.0. Dérogation raisonnée sur Node : LTS 24.18.1 retenue plutôt que la stable 26.5.1. |
