# Kaya — Versions gelées

*Application du principe XI de `.specify/memory/constitution.md` : dernières versions stables,
**vérifiées sur les registres officiels avec l'URL citée**, puis **épinglées exactement** et
figées par lockfiles.*

**Version du gel : 1.0.14 — vérifié le 2026-08-04**
**Prochaine revue : 2026-08-31** (revue mensuelle groupée) — elle traite les **montées** de version
et les dix briques du §2. Depuis le gel 1.0.14, elle n'est plus le passage obligé d'un **ajout** :
voir §1, règle 3.

**Cible de déploiement retenue : Docker sur VPS Contabo** (mode A du cadrage §10.1, SaaS
mutualisé). Toutes les versions ci-dessous sont vérifiées disponibles pour cette cible (§4.2).

**Un seul point reste ouvert** : le choix de sqlx `0.9.0` doit être **confirmé par le spike
GiST/`tstzrange`** de la phase 0 (cadrage §16). Tout le reste est arrêté.

---

## 1. Règles d'usage de ce document

**AJOUTER EST LIBRE. MONTER NE L'EST PAS.** Les deux gestes ont été confondus jusqu'au gel
1.0.13, et cette confusion a produit la seule dérive que ce document ait connue — sept crates
épinglées dans les manifestes, absentes d'ici, en attente d'une revue mensuelle que la règle
elle-même leur imposait d'attendre.

**Le principe XI ne l'exigeait pas, il disait le contraire.** Ses termes exacts : les versions sont
vérifiées sur les registres officiels *« avec l'URL citée **dans le changement qui les introduit** ou
les met à jour »*, et ce qui est proscrit en cours d'incrément est *« aucune **montée majeure** »*.
Le principe prévoit donc explicitement qu'une version entre par un changement ; c'est le §1.4 de ce
document — *« jamais au fil de l'eau »* — qui avait durci au-delà de lui. La préséance tranche
(constitution avant document de référence), et le durcissement tombe.

1. **Aucun numéro de version n'est écrit de mémoire.** Chaque version porte l'URL du registre
   officiel interrogé et la date de vérification — dans ce fichier pour le §2, **en commentaire du
   manifeste** pour les §3.x. Une version non vérifiée est une version inconnue (principe XI).
2. **Épinglage exact obligatoire** : `= 4.14.0` ou `4.14.0`, jamais `^4.14`, `~4.14` ni `4.*`.
   La porte de CI **P-20** échoue sur tout intervalle et sur tout lockfile absent ou périmé.
   **Cette règle ne connaît aucune exception** — c'est elle qui rend une reconstruction
   reproductible, et rien de ce qui suit ne l'assouplit.
3. **AJOUTER une dépendance absente est libre, en cours de cycle, sans autorisation.** Trois
   obligations, aucune n'étant une permission à demander :
   - l'épinglage est exact et le lockfile est commité (règle 2) ;
   - le manifeste porte, **en commentaire au-dessus de la ligne**, le rôle, l'URL du registre
     interrogé et la date — ce que les cycles font déjà spontanément et bien ;
   - le commentaire dit **pourquoi les dépendances déjà présentes ne suffisent pas**. Pas pour
     obtenir un accord : pour que la question soit posée. L'arbitrage `aes-gcm` du cycle 006 —
     qui a examiné et écarté `ring`, déjà là transitivement — est le modèle.

   **L'inscription aux tableaux §3.x se fait DANS LE CHANGEMENT QUI AJOUTE**, jamais reportée à
   une revue. Une échéance sans porte est un rappel, pas un contrôle : c'est la leçon exacte des
   sept crates, et la règle qui les avait égarées est celle-ci, dans sa version d'avant.
4. **MONTER une version déjà gelée reste groupé et mensuel.** Aucune montée majeure pendant un
   incrément ; une faille de sécurité est la seule exception, consignée au §6. Motif : une montée
   touche du code qui marche, un ajout ne touche rien.
5. **Les dix briques du §2 ne se touchent qu'à la revue, y compris en mineur.** Rust, Actix Web,
   sqlx, utoipa, Nuxt, Tailwind, Tauri, PostgreSQL, Redis, Garage. Elles ne sont pas des
   dépendances parmi d'autres : monter `sqlx` de 0.9 à 1.0 réécrit les macros de chaque requête du
   dépôt. Le §2 reste tenu à la main et **opposable** — un écart y échoue, il ne se régularise pas.
6. **Deux crates de la même famille fonctionnelle ne cohabitent pas** — `time` et `chrono`,
   `thiserror` et `anyhow` en bibliothèque, deux clients HTTP, deux moteurs de chiffrement. La
   liberté d'ajouter ne vaut que pour ce qui manque, et la liste des familles exclusives est au
   §3.4.
7. Reproduire la vérification : les commandes exactes sont au §5.

> **Les tableaux §3.x sont destinés à être GÉNÉRÉS depuis les manifestes**, `docs/versions-gelees.md`
> devenant le miroir vérifié du code plutôt qu'un document parallèle qui en dévie. Le contrôle
> devient alors trivial et complet dans les deux sens : *le gel commité est-il celui que les
> manifestes engendrent ?* Tant que le générateur n'existe pas, les tableaux §3.x s'écrivent à la
> main **dans le même changement que le manifeste** — ce que la règle 3 impose déjà. Voir §4.3.

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
| `actix-cors` | **0.7.1** ⚠️ | CORS — l'application est une **SPA servie depuis une autre origine** que l'API | dépend d'`actix-web ^4` → **compatible**. `0.x` assumé : crate officiel de l'écosystème Actix, branche 0.7 stable depuis 2025-03-11 |
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
| `argon2` | **0.5.3** | Hachage de mot de passe (CPT-01) **et dérivation de la clé de tenant** | — |
| `aes-gcm` | **0.11.0** | **Chiffrement au repos des pièces d'identité clients** (SEJ-01, transfert ARTCI) | RustCrypto, **Rust pur** — aucune chaîne C, comme `argon2`. Vérifiée le 2026-08-03 |
| `serde_json` | **1.0.151** | Charges utiles JSONB — outbox, contexte d'audit | dernière `1.x` de `serde`, alignée sur `serde` 1.0.229 |
| `time` | **0.3.54** | `timestamptz` et `tstzrange` (principe IV, occupations) | features `serde` `macros` `formatting` `parsing` **vérifiées présentes** |
| `thiserror` | **2.0.19** | Types d'erreur de domaine | — |
| `async-trait` | **0.1.91** | Traits d'abstraction asynchrones (principe III) | — |
| `futures` | **0.3.33** | Combinateurs du worker outbox | — |
| `dotenvy` | **0.15.7** | Variables d'environnement en développement | jamais chargé en production — l'image lit l'environnement du conteneur |

> **Les sept dernières lignes régularisent l'existant, elles ne choisissent rien.** Ces versions
> étaient épinglées exactement dans `backend/Cargo.toml` — six depuis les cycles 001 à 005, une
> depuis le cycle 006 — avec leur date de vérification en commentaire, et **aucune n'était au
> gel**. Le §4.3 avait annoncé ce trou mot pour mot : *« une version du code absente du gel est
> aussi un défaut qu'une version du gel absente du code »*. C'est la **deuxième** fois qu'il
> s'ouvre — le gel 1.0.5 l'a comblé côté npm, personne ne l'a fait côté Rust.

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
| `@playwright/test` | **1.62.1** | Harnais **end-to-end** — porte **P-22** : l'application démarre et chaque route s'atteint. Runner, fixtures et `expect` à réessai inclus ; télécharge ses navigateurs sur le poste, **jamais dans le paquet livré** |
| `@types/node` | **24.13.3** ⚠️ | Types du runtime — **dernière `24.x`, alignée sur Node `24.18.1`**, pas la dernière stable |
| `@phosphor-icons/web` | **2.1.2** | **Source des glyphes d'icônes** — sous-réglée à la construction, jamais expédiée telle quelle (porte **P-21**) |
| `subset-font` | **2.5.0** | Sous-règle la police d'icônes ; **contrôle tiers** des polices de texte par harfbuzz — outil de génération, absent du paquet livré |
| `@fontsource-variable/archivo` | **5.3.0** | **Source d'Archivo** — texte et titres, embarquée en local (portes **P-21** et **P-21b**) |
| `@fontsource-variable/chivo-mono` | **5.3.0** | **Source de Chivo Mono** — montants, quantités, heures ; le tabulaire qui aligne les colonnes |

> **Un paquet ajouté au gel 1.0.11 — le harnais de la porte P-22.** `@playwright/test`
> **1.62.1** (Apache-2.0, publiée le 2026-07-30, **aucune `peerDependency`**), vérifiée sur
> `https://registry.npmjs.org/@playwright%2Ftest/latest` le **2026-08-01**.
>
> **Ce qu'il achète.** Le cycle 003 a été livré avec 24 portes vertes, 224 tests backend et
> 428 tests front — et deux des quatre écrans du produit étaient inatteignables en navigateur.
> Les tests front montent les composants avec `@vue/test-utils`, ce qui contourne le routeur,
> `<Suspense>`, les layouts et les plugins. Aucun outil du dépôt ne savait ouvrir une page.
>
> **Une seule ligne épingle toute la chaîne.** `@playwright/test` **1.62.1** dépend de
> `playwright` **1.62.1** en version **exacte**, qui dépend de `playwright-core` **1.62.1** en
> version exacte — aucun intervalle nulle part. Inscrire le second paquet serait redondant et
> créerait deux valeurs à tenir en phase. C'est `@playwright/test` et non `playwright` parce que
> le premier **embarque le runner** — `test`, les fixtures, `expect` à réessai, les reporters ;
> le second n'est que la bibliothèque d'automatisation, à adosser à un autre harnais.
>
> ⚠️ **La révision de navigateur n'est pas épinglable séparément, et le cache du poste ne
> servait pas.** `1.62.1` exige Chromium **1234** (Chrome 151.0.7922.34), figée dans le
> `browsers.json` de `playwright-core`. Le poste portait `chromium-1217` (Playwright 1.59.x) et
> `chromium-1228` (1.61.x) : ni l'un ni l'autre. Le téléchargement a bien eu lieu — 94,7 Mio,
> constaté, pas supposé. **En CI, le prévoir explicitement** (cache d'artefacts sur la clé
> `chromium-1234`, ou image `mcr.microsoft.com/playwright`), sinon chaque exécution repart du
> réseau.
>
> ⚠️ **DEUX navigateurs depuis le 2026-08-02, et le second pèse trois fois le premier.** P-22
> exerce `chromium` **et** `webkit` : la cible est Tauri, qui n'embarque aucun navigateur et
> emprunte celui du système — WebKit couvre **macOS, iOS et Linux**, Chromium couvre Windows et
> Android. Le même `browsers.json` de `playwright-core@1.62.1` impose WebKit **rev 2336**
> (version 26.5), relevé sur
> `https://cdn.jsdelivr.net/npm/playwright-core@1.62.1/browsers.json` le **2026-08-02**.
> Installation : `pnpm exec playwright install webkit`. **294 Mio sur le poste**, mesurés — pas
> 95. Une CI qui ne met en cache que `chromium-1234` retéléchargera WebKit à chaque exécution ;
> la clé de cache doit porter les deux révisions. La porte le vérifie elle-même : son étape 3/5
> compte les cas par projet et **refuse si un moteur n'en a aucun**, plutôt que de passer au vert
> sur un seul.
>
> **Hors périmètre de P-21, et ce n'est pas une dérogation.** Playwright télécharge des
> navigateurs, mais il ne s'exécute **jamais** dans le produit : c'est de l'outillage de
> développement, au même titre que `subset-font`. La porte **P-21** ne porte que sur ce que
> l'**application** charge à l'exécution, et un navigateur rangé dans
> `~/Library/Caches/ms-playwright` n'entre ni dans le paquet Nuxt, ni dans l'image
> `linux/amd64`. Même raison pour la licence : Apache-2.0 **n'entre pas** à
> `docs/conformite/licences-tierces.md`, qui inventorie les œuvres **embarquées** et dont
> l'attribution est due. Précédent identique : `subset-font`, BSD-3-Clause, déjà gelé et déjà
> absent de cet inventaire.
>
> **Déclaré à la RACINE, pas dans `app/`** — même motif qu'ESLint : la porte exerce les trois
> surfaces servies, `app/` aujourd'hui, `web/qr` et `web/console` dès qu'elles auront des écrans.
> Le placer dans `app/` le rendrait aveugle aux deux autres, ou imposerait de le déclarer trois
> fois.
>
> **À reporter à la revue du 2026-08-31** comme les paquets des gels 1.0.5, 1.0.6, 1.0.9 et
> 1.0.10.

> **Deux paquets ajoutés au gel 1.0.10 — les polices de texte embarquées, dernière dette du
> cycle 002.** `docs/design/theme.css`, section « POLICES », prescrit de les servir en local
> (`woff2`, `font-display: swap`) « parce que le produit tourne sur des liaisons lentes et doit
> s'afficher hors ligne ». Ce n'est pas cosmétique : `tokens.md` §2 fait de **Chivo Mono
> tabulaire** la condition de l'alignement des colonnes de montants — sur les polices système de
> repli, un écran de caisse ou de clôture affiche des montants désalignés.
>
> `@fontsource-variable/archivo` **5.3.0** et `@fontsource-variable/chivo-mono` **5.3.0**
> (OFL-1.1, publiées le 2026-07-19, **aucune dépendance et aucune `peerDependency`** pour l'une
> comme pour l'autre), vérifiées sur `https://registry.npmjs.org/@fontsource-variable%2Farchivo`
> et `https://registry.npmjs.org/@fontsource-variable%2Fchivo-mono` le **2026-07-31**.
>
> **Variable et non statique — mesuré, pas supposé.** Les quatre fichiers variables (deux
> familles × `latin` et `latin-ext`, axe `wght` 100→900) pèsent **114,0 ko**. L'équivalent
> statique demanderait **douze** fichiers pour **152,7 ko** : le produit emploie quatre graisses
> d'Archivo — 400 par défaut, 500 `font-medium`, 600 `font-semibold`, 700 `font-bold`, relevées
> dans `docs/design/` — et deux de Chivo Mono. Le variable pèse **75 %** du statique et absorbe
> une graisse de plus sans ajouter un fichier. Aucune italique n'est embarquée : les trois
> occurrences d'`italic` de `docs/design/` sont des `not-italic`.
>
> **Aucun sous-réglage de caractères, contrairement aux icônes.** Les glyphes d'icônes forment un
> ensemble fini et connu ; le texte est dynamique — noms de clients, communes, libellés saisis par
> l'exploitant. Un sous-réglage produirait un caractère manquant sur un nom propre ivoirien,
> constaté en production. On se limite aux **sous-ensembles de script** : `latin` **et**
> `latin-ext`, jamais `latin` seul, qui ne porte ni Ÿ ni les latines étendues. Le sous-ensemble
> `vietnamese` n'est pas embarqué — limite assumée, 23 ko pour un besoin qui n'existe pas.
>
> ⚠️ **U+202F n'existe ni dans Archivo ni dans Chivo Mono, et il a fallu l'ajouter.** `tokens.md`
> §2 impose l'espace fine insécable **U+202F** entre les groupes de milliers et avant le F
> (`12 500 F`). Vérification par lecture de la table `cmap` — pas par lecture de la
> `unicode-range` déclarée, qui annonce pourtant `U+2000-206F` : **le caractère est absent des
> `woff2` de Fontsource ET des `ttf` amont de Google Fonts** (sondés sur `Archivo_400Regular.ttf`
> et `ChivoMono_400Regular.ttf`). Sans correction, chaque montant du produit fait tomber son
> séparateur sur une police de repli, de chasse inconnue — donc les colonnes ne s'alignent plus,
> la propriété même que Chivo Mono doit garantir.
>
> `app/scripts/generer-polices.ts` ajoute donc à la `cmap` l'association **U+202F → dessin de
> U+2009** (THIN SPACE), présent dans les deux familles. Aucun glyphe n'est créé. Le choix est
> **mesuré** sur les `ttf` amont (unités de 1000) : en Archivo, U+2009 vaut 193 contre 209 pour
> l'espace mot — la fine attendue ; en Chivo Mono, U+2009 vaut 600 comme tout autre caractère —
> **la cellule pleine, donc l'alignement tabulaire tenu**. L'insécabilité vient du caractère
> U+202F lui-même, de catégorie Unicode `Zs` non sécable : on lui donne un dessin, on ne le
> substitue pas.
>
> **Déterminisme vérifié à l'octet**, condition du mode `--verifier` : Brotli est déterministe à
> paramètres fixés, et ils sont posés explicitement dans le script. **Validité vérifiée par un
> tiers** : `subset-font` — donc harfbuzz, le moteur de mise en forme des navigateurs — ouvre les
> quatre fichiers et retient U+202F en sous-réglant sur « 12 500 F ». Ce contrôle n'est pas
> décoratif : il a refusé une première version des fichiers, à laquelle il manquait le
> **complément d'alignement sur quatre octets** que le décodeur de référence exige. Le décodeur
> écrit pour ce script les relisait sans rien voir.
>
> **À reporter à la revue du 2026-08-31** comme les paquets des gels 1.0.5, 1.0.6 et 1.0.9.

> **Deux paquets ajoutés au gel 1.0.9 — la police d'icônes, dette ouverte du cycle 002.** La
> maquette charge Phosphor depuis `unpkg.com` ; reprise telle quelle, elle rend l'écran dépendant
> du réseau, ce que le principe VI interdit et ce que la porte **P-21** refuse désormais. Sans
> police embarquée, les icônes de `G1` ne s'affichaient pas du tout.
>
> `@phosphor-icons/web` **2.1.2** (MIT, publiée le 2025-03-31, aucune `peerDependency` ni
> dépendance) vérifiée sur `https://registry.npmjs.org/@phosphor-icons%2Fweb` le 2026-07-31.
> C'est la **même famille et la même version** que celles des maquettes — les glyphes du produit
> et ceux de la référence visuelle sont donc identiques au dessin près, ce qu'une montée de
> version briserait en silence.
>
> `subset-font` **2.5.0** (BSD-3-Clause, publiée le 2026-04-02, dépend de `harfbuzzjs`,
> `fontverter`, `lodash`, `p-limit`) vérifiée sur `https://registry.npmjs.org/subset-font` le
> 2026-07-31, à la dernière stable. **Outil de génération, jamais expédié** : il produit
> `app/assets/fonts/*.woff2` à la main, et les artefacts sont commités.
>
> **Pourquoi sous-régler plutôt qu'embarquer la police entière** : 279 ko de `woff2` pour les deux
> variantes, contre **9,4 ko** pour les 77 glyphes réellement employés. La persona Aminata
> travaille sur un Android d'entrée de gamme en réseau intermittent ; 270 ko d'icônes jamais
> affichées retardent le premier écran à chaque installation.
>
> **Déterminisme vérifié**, comme pour le générateur de client (§3.2, exigence 1) : deux
> exécutions successives produisent deux fichiers identiques à l'octet. Sans cette propriété, le
> mode `--verifier` de la porte échouerait au hasard et serait désactivé sous trois semaines.

> **`@types/node` — dette du cycle 001, réparée au gel 1.0.7.** `app/tsconfig.test.json` typait
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

### 3.4 Familles exclusives — le seul garde-fou de la liberté d'ajouter

La règle 3 du §1 rend l'ajout libre. Son unique limite : **une famille, une implémentation.** Deux
crates qui font la même chose ne cohabitent pas — c'est ce qui produit deux formats de date dans un
même dépôt, ou deux moteurs de chiffrement dont un seul est audité.

| Famille | Retenu | Écartés — ne pas introduire |
|---|---|---|
| Date et heure (Rust) | `time` | `chrono` |
| Erreurs de bibliothèque (Rust) | `thiserror` | `anyhow`, `eyre` — acceptables dans un **binaire**, jamais dans un crate de `crates/` |
| Chiffrement symétrique | `aes-gcm` (RustCrypto) | `ring`, `aws-lc-rs`, `openssl`, `pgcrypto` — motif au §6, gel 1.0.13 |
| Dérivation de clé et hachage | `argon2` | `bcrypt`, `scrypt`, `pbkdf2` |
| Décimales exactes | `rust_decimal` | `bigdecimal`, et **tout flottant** sur une quantité (principe V) |
| Client HTTP sortant | *aucun encore* | dès qu'un cycle en a besoin, **il tranche pour tout le dépôt** et l'inscrit ici |
| Sérialisation | `serde` / `serde_json` | — |
| Accès base | `sqlx` | `diesel`, `sea-orm` — brique du §2, hors ajout libre |

**Une famille absente de ce tableau n'est pas une famille libre : c'est une famille non encore
rencontrée.** Le cycle qui l'ouvre choisit pour tout le dépôt et inscrit sa ligne — c'est le
moment où le choix coûte le moins, et le seul où il est encore réversible.

**Ce tableau se tient à la main, et c'est assumé** : il porte des arbitrages, pas des versions.
Aucun générateur ne peut décider que `chrono` est écarté.

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
silencieuse en `5.9.3`, puis les sept crates du gel 1.0.13.

**Depuis le gel 1.0.14, le complément a deux régimes, parce que les deux moitiés du document ne
sont pas de même nature :**

| | §2 · dix briques, §3.3, §4.2 | §3.1 et §3.2 · dépendances directes |
|---|---|---|
| **Qui écrit** | la revue mensuelle, à la main | **le générateur**, depuis les manifestes |
| **Régime** | **opposable** — l'écart échoue | **miroir** — l'écart se régularise |
| **Ce que la porte fait** | compare et **refuse** | régénère et refuse **si le commité diffère** |
| **Motif** | monter une brique réécrit du code qui marche | ajouter est libre (§1, règle 3) ; le gel doit suivre, pas autoriser |

Le contrôle du régime miroir est **plus simple et plus complet** que la comparaison bilatérale
décrite ci-dessus : si le gel est engendré depuis les manifestes, il ne peut par construction ni
omettre une dépendance du code, ni en inventer une que le code n'a pas. Une seule question suffit —
*le gel commité est-il celui que les manifestes engendrent ?* — et un seul mode de réparation :
relancer le générateur.

Ce qui reste à vérifier dans les deux régimes :

- **les `peerDependencies` des paquets gelés**, ce qui aurait attrapé la contradiction
  `openapi-typescript ^5.x` ↔ `typescript 7.0.2` à la source ;
- **les familles exclusives du §3.4** — deux membres d'une même famille dans les manifestes est un
  échec, et c'est le seul garde-fou de la liberté d'ajouter ;
- **la présence du commentaire de justification** au-dessus de chaque ligne ajoutée (§1, règle 3).
  Un ajout sans motif écrit passe la porte de la forme et perd celui de l'intention.

Sans ce complément, le gel est un document que rien n'oppose au code. C'est l'illustration
exacte de la leçon du cycle 1 : *un test négatif prouve qu'une porte sait échouer, il ne prouve
pas qu'elle regarde tout.* Et l'illustration du gel 1.0.13 : *une échéance sans porte est un
rappel, pas un contrôle.*

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
         openapi-typescript openapi-fetch typescript \
         @phosphor-icons/web subset-font \
         @fontsource-variable/archivo @fontsource-variable/chivo-mono \
         @playwright/test; do
  printf "%-22s " "$p"
  curl -sS "https://registry.npmjs.org/$(echo $p | sed 's|/|%2F|')/latest" \
    | python3 -c "import sys,json;print(json.load(sys.stdin)['version'])"
done

# Playwright — la révision de navigateur, que le numéro npm ne donne PAS.
# `browsers.json` de playwright-core fait foi ; la révision n'est pas épinglable séparément, et
# un écart avec le cache du poste = un navigateur de ~95 Mio à retélécharger.
PW=$(curl -sS https://registry.npmjs.org/@playwright%2Ftest/latest \
  | python3 -c "import sys,json;print(json.load(sys.stdin)['version'])")
echo "@playwright/test $PW"
curl -sS "https://cdn.jsdelivr.net/npm/playwright-core@$PW/browsers.json" \
  | python3 -c "
import sys, json
# Les DEUX moteurs de P-22 : chromium couvre Windows et Android, webkit couvre macOS, iOS et
# Linux. N'en relever qu'un rendrait la vérification aveugle sur trois cibles du produit.
for e in json.load(sys.stdin)['browsers']:
    if e['name'] in ('chromium', 'webkit'):
        print(' ', e['name'], 'rev', e['revision'], '| version', e.get('browserVersion'))
"
ls -1 "${PLAYWRIGHT_BROWSERS_PATH:-$HOME/Library/Caches/ms-playwright}" 2>/dev/null \
  | grep -E '^(chromium|webkit)-' || echo "  (aucun navigateur en cache)"

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
| 1.0.14 | 2026-08-04 | **CHANGEMENT DE RÉGIME — ajouter une dépendance devient libre, monter reste groupé.** Le §1 est réécrit, le §3.4 (familles exclusives) est créé, le §4.3 distingue deux régimes de contrôle. **Aucun amendement de la constitution n'a été requis, et c'est le cœur de l'affaire** : le principe XI proscrit « aucune montée majeure pendant un incrément » et prescrit une « revue de mise à jour groupée, mensuelle » — il ne dit rien des **ajouts**. C'est le §1.4 de ce document, plus strict que le principe qu'il applique, qui étendait la règle des montées aux ajouts : *« La revue est mensuelle et groupée, jamais au fil de l'eau. »* **Cette règle est la cause directe de la dérive du gel 1.0.13** — elle ordonnait aux sept crates d'attendre une revue, et six l'ont attendue six semaines. Une règle qui produit la dette qu'elle prétend organiser doit changer, pas être mieux respectée. **Ce que le régime nouveau conserve intact** : épinglage exact sans exception, aucun numéro écrit de mémoire, URL et date de vérification, lockfiles commités — P-20 est inchangée, et rien de ce gel ne l'assouplit. **Ce qu'il déplace** : l'inscription aux tableaux §3.x se fait dans le changement qui ajoute, jamais reportée ; les dix briques du §2 restent hors ajout libre, y compris en mineur, parce que monter `sqlx` réécrit les macros de chaque requête du dépôt. **Ce qu'il ajoute** : le §3.4, seul garde-fou de la liberté — une famille, une implémentation, et une famille absente du tableau est une famille non encore rencontrée, pas une famille libre. **Le coût de l'ancien régime était déjà payé** : `backend/crates/socle/comptes/src/client/repli.rs` — 286 lignes, table de correspondances écrite à la main — motive son existence en citant *en premier* le fait qu'`unicode-normalization` « n'est pas au gel » et que l'ajouter « imposerait une décision de revue mensuelle ». Le second motif de ce fichier est **techniquement juste et le sauve** (NFD ne décompose ni `Ø` ni `Đ`, lettres à barre sans décomposition canonique, et le produit gagne à décider de ce qu'il replie) — mais l'ordre des arguments dit lequel a déclenché la réflexion. **Une contrainte de gouvernance des versions n'a pas à entrer dans un raisonnement de conception** ; c'est ce que ce gel corrige. Le fichier n'est pas modifié : sa décision reste défendable, et le prochain cycle qui touche à la recherche de noms est libre de la reprendre — ou de la garder pour son bon motif, désormais le seul. |
| 1.0.13 | 2026-08-04 | **Sept crates Rust inscrites au §3.1 — dont une seule est du cycle 006.** `aes-gcm` **0.11.0** (cycle 006), `serde_json` **1.0.151**, `time` **0.3.54**, `thiserror` **2.0.19**, `async-trait` **0.1.91**, `futures` **0.3.33**, `dotenvy` **0.15.7** (cycles 001 à 005). Toutes étaient **déjà épinglées exactement** dans `backend/Cargo.toml`, avec leur date de vérification de registre en commentaire — 2026-07-30 pour six, 2026-08-03 pour `aes-gcm`. **Aucune valeur n'est choisie ici** : le gel rattrape le code, ce que le §4.3 prescrit dans les deux sens. **Motif de l'inscription anticipée** : la revue mensuelle du 2026-08-31 était l'échéance consignée, et le rapport du cycle 006 n'annonçait qu'**une** entrée à trancher — juste pour son cycle, faux pour la revue, qui en aurait découvert sept le jour même. Le `Cargo.toml` portait la mention « à porter au gel §3.1 » **au-dessus des six autres**, écrite six semaines plus tôt et jamais exécutée : une échéance sans porte est un rappel, pas un contrôle. **Le seul arbitrage du lot** est celui d'`aes-gcm`, et il est écrit dans `backend/Cargo.toml` : `pgcrypto` écartée (la clé voyagerait dans `pg_stat_statements` et les journaux de la base — chiffrer au repos en publiant la clé n'est pas chiffrer), `ring` écartée **bien qu'à coût nul en chaîne d'approvisionnement**, déjà transitive de `rustls`, pour le motif qui avait écarté `aws_lc_rs` : chaîne `cmake`/`nasm`, et « une chaîne de construction C de plus est une panne de plus chez un client sans administrateur » (cadrage §10.1, mode B). **Ce que ce gel ne fait pas** : le complément du §4.3 reste à écrire, et c'est lui qui aurait trouvé les six — **deuxième occurrence du même trou après le gel 1.0.5**, côté Rust cette fois. Tant qu'il manque, l'écart se rouvrira au prochain cycle qui ajoute un crate. |
| 1.0.12 | 2026-08-02 | **WebKit ajouté aux navigateurs de la porte P-22 — aucun paquet nouveau, une cible de plus.** `@playwright/test` reste en **1.62.1** : ce qui change est le jeu de moteurs que la porte exerce, pas une version. **Motif** : la cible du produit est Tauri, qui n'embarque aucun navigateur et emprunte celui du système — **WKWebView** sur macOS et iOS, **WebKitGTK** sur Linux, **WebView2** (Chromium) sur Windows, **Android System WebView** (Chromium) sur Android. Trois cibles sur cinq sont WebKit, à commencer par le poste de développement : Chromium seul validait le moteur que le produit n'utilise pas sur la majorité de ses cibles. ⚠️ **La révision n'est pas épinglable séparément, et elle pèse trois fois Chromium** : `playwright-core@1.62.1` impose WebKit **rev 2336** (version 26.5), relevée sur `https://cdn.jsdelivr.net/npm/playwright-core@1.62.1/browsers.json` le 2026-08-02 — **294 Mio sur le poste, mesurés**, contre 94,7 pour Chromium. Une CI qui ne met en cache que `chromium-1234` retéléchargera 294 Mio à chaque exécution ; la clé doit porter les deux révisions. La commande de vérification du §5 relève désormais les deux, et l'étape 3/5 de la porte **compte les cas par projet et refuse si un moteur n'en a aucun** — un moteur absent retirerait trois cibles sans changer le verdict. **Limite écrite dans la porte** : le `webkit` de Playwright **n'est pas WKWebView**, seulement plus proche de la cible que Chromium ; le contrôle réel de macOS et d'iOS viendra avec la coquille Tauri. **Hors P-21** pour le même motif que Chromium : un navigateur de test n'entre ni dans le paquet Nuxt, ni dans l'image `linux/amd64`. |
| 1.0.11 | 2026-08-01 | **`@playwright/test` `1.62.1` inscrit — le harnais de la porte P-22.** Apache-2.0, publiée le 2026-07-30, aucune `peerDependency`, vérifiée sur `https://registry.npmjs.org/@playwright%2Ftest/latest` le 2026-08-01. **Motif** : le cycle 003 a été livré avec 24 portes vertes et 652 tests, et deux des quatre écrans du produit étaient inatteignables en navigateur — aucun outil du dépôt ne savait ouvrir une page. **Une ligne épingle la chaîne entière** : `@playwright/test` → `playwright` → `playwright-core`, toutes trois en `1.62.1` **exact**, aucun intervalle. Le paquet retenu est celui qui **embarque le runner**. ⚠️ **La révision de navigateur n'est pas épinglable séparément** : `1.62.1` impose Chromium **1234** (Chrome 151.0.7922.34) par son `browsers.json`, et le cache du poste (`1217`, `1228`) ne servait pas — 94,7 Mio téléchargés, constaté. **Hors P-21** : la porte ne vise que ce que l'application charge à l'exécution, et un navigateur de test n'entre ni dans le paquet ni dans l'image de production ; pour la même raison, Apache-2.0 n'entre pas à l'inventaire des licences tierces, réservé aux œuvres embarquées — précédent `subset-font`. Déclaré à la **racine** et non dans `app/`, comme ESLint et pour le même motif : la porte exerce `app/`, et `web/qr` et `web/console` dès qu'elles auront des écrans. |
| 1.0.10 | 2026-07-31 | **Archivo et Chivo Mono embarquées — dernière dette du cycle 002 soldée, portes P-21 et P-21b.** `@fontsource-variable/archivo` **5.3.0** et `@fontsource-variable/chivo-mono` **5.3.0**, OFL-1.1, publiées le 2026-07-19, aucune dépendance ni `peerDependency`, vérifiées sur `https://registry.npmjs.org/@fontsource-variable%2Farchivo` et `https://registry.npmjs.org/@fontsource-variable%2Fchivo-mono` le 2026-07-31. L'application tournait sur les polices système de repli, alors que `theme.css` prescrit le local et que `tokens.md` §2 confie l'alignement des colonnes de montants à Chivo Mono tabulaire. **Variable retenue sur mesure** : 4 fichiers / 114,0 ko contre 12 fichiers / 152,7 ko en statique, pour les quatre graisses d'Archivo et les deux de Chivo Mono réellement employées. **Aucun sous-réglage de caractères** — le texte est dynamique, contrairement aux icônes : `latin` **et** `latin-ext`, sous-ensembles de script entiers. ⚠️ **U+202F absent de la source, ajouté à la `cmap`** (associé au dessin de U+2009, chasse mesurée : 193 en Archivo, 600 en Chivo Mono donc cellule pleine) : le caractère n'existe ni dans les `woff2` de Fontsource ni dans les `ttf` amont de Google Fonts, ce que seule la lecture de la table révèle — la `unicode-range` déclarée annonce `U+2000-206F`. Déterminisme à l'octet vérifié ; validité confirmée par harfbuzz, qui a d'abord **refusé** les fichiers auxquels manquait le complément d'alignement sur quatre octets. |
| 1.0.9 | 2026-07-31 | **La police d'icônes embarquée — dette du cycle 002 soldée, porte P-21.** `@phosphor-icons/web` **2.1.2** (source des glyphes) et `subset-font` **2.5.0** (outil de sous-réglage), vérifiés sur `https://registry.npmjs.org/` le 2026-07-31, licences MIT et BSD-3-Clause, `peerDependencies` contrôlées — aucune pour le premier, aucune pour le second. La maquette charge Phosphor depuis `unpkg.com` ; l'application ne le fait **jamais**. Sous-ensemble de **77 glyphes sur ~1530**, soit **9,4 ko** au lieu de 279 ko. La version est **alignée sur celle des maquettes** : deux versions différentes donneraient deux dessins d'icône, écart qu'aucune porte ne verrait. Déterminisme à l'octet vérifié, condition du mode `--verifier`. **À reporter à la revue du 2026-08-31** comme les paquets des gels 1.0.5 et 1.0.6. |
| 1.0.8 | 2026-07-31 | **`actix-cors` `0.7.1` inscrit — manque révélé par le PREMIER ÉCRAN du produit.** L'application est une SPA servie depuis une autre origine que l'API : `localhost:3000` en développement, `tauri://localhost` sous Tauri. Sans en-têtes CORS, le navigateur bloque chaque appel et **aucun écran ne fonctionne** — le préflight `OPTIONS` rendait `404`. Le cycle 001 ne pouvait pas le rencontrer, n'ayant livré aucun écran ; `G1` l'a révélé au premier chargement réel. **`0.x` assumé, contrairement au motif qui a écarté `@hey-api/openapi-ts 0.99.0`** : c'est le crate officiel de l'écosystème Actix, maintenu par la même équipe, sa branche 0.7 est stable depuis 2025-03-11, et il déclare `actix-web ^4` — compatible avec le `4.14.0` gelé (vérifié sur `https://crates.io/api/v1/crates/actix-cors/0.7.1/dependencies` le 2026-07-31). L'alternative — écrire un CORS à la main sur un chemin de sécurité — était le mauvais échange. La politique est par **liste d'origines explicite**, jamais `*`, et son défaut ne contient que des origines locales. |
| 1.0.7 | 2026-07-31 | **`@types/node` `24.13.3` inscrit — dette du cycle 001.** `app/tsconfig.test.json` typait `scripts/**/*.ts` sans qu'aucun paquet ne fournisse les types de `node:fs` / `node:path` : `pnpm test` échouait en permanence sur six `TypeCheckError`, alors que ses dix-huit tests passaient. Les deux fichiers non typés portent les portes **P-16** et **P-17**. Version alignée sur la ligne majeure du runtime gelé (Node `24.18.1` LTS), donc dernière `24.x` et non `latest` (`26.1.2`) — même dérogation raisonnée que Node. Vérifiée sur `https://registry.npmjs.org/@types/node` le 2026-07-31. Suivi : toute montée de Node au §3.3 impose la même montée ici. |
| 1.0.6 | 2026-07-31 | **Trois paquets de test front inscrits — décision T004 du cycle 002, tranchée dans le sens de l'ajout.** `@vue/test-utils` **2.4.11**, `happy-dom` **20.11.1**, `@vitejs/plugin-vue` **6.0.8**, vérifiés sur `https://registry.npmjs.org/` le 2026-07-31, `peerDependencies` contrôlées une par une contre Vue 3 / Vite de Nuxt 4.5.1. `plan.md` laissait le choix ouvert entre ajouter et refuser, sans version : refuser aurait réduit SC-005 à un test de la fonction de sélection, c'est-à-dire à vérifier l'intention plutôt que le HTML produit — or « un service inactif est absent, jamais grisé » est une propriété de rendu. **Le plan n'en annonçait que deux** : le troisième est la dépendance technique qui compile un `.vue` hors du pipeline Nuxt, signalée ici plutôt que glissée dans le lot. À reporter à la revue du 2026-08-31 comme les six du gel 1.0.5. |
| 1.0.4 | 2026-07-31 | **TypeScript reculé de `7.0.2` à `5.9.3`** — corrige une erreur du gel 1.0.3, constatée à l'exécution au cycle 001. `openapi-typescript` 7.13.0 déclare `peerDependencies: { typescript: "^5.x" }` et TypeScript 7 a modifié l'API `ts.factory` : la génération du client échoue sur `TypeError: Cannot read properties of undefined (reading 'createKeywordTypeNode')`, donc la porte **P-01** ne peut pas s'exécuter. `5.9.3` vérifiée sur `https://registry.npmjs.org/typescript` le 2026-07-31 comme dernière `5.x`. Dérogation raisonnée au « dernière stable », de même nature que Node LTS. Condition de levée : `openapi-typescript` déclare `typescript ^7`. **Leçon de gouvernance** : le §3.1 vérifiait la compatibilité inter-crates, le §3.2 ne le faisait pas pour les paquets npm — l'écart est comblé. |
| 1.0.5 | 2026-07-31 | **Six paquets JS inscrits au gel** — `vitest` 4.1.10, `eslint` 10.8.0, `@eslint/js` 10.0.1, `eslint-plugin-vue` 10.10.0, `typescript-eslint` 8.65.0, `@tailwindcss/vite` 4.3.3. Ils vivaient dans `app/package.json` depuis le cycle 001 sans décision tracée. Vérifiés sur npm : tous à la dernière stable, valeurs du dépôt confirmées. Écart trouvé par l'analyse du cycle 002, **pas par P-20** — le complément du §4.3 reste à écrire. |
| 1.0.3 | 2026-07-30 | **Générateur de client TypeScript ajouté** — lacune du gel initial signalée par le plan du cycle 1 : la porte P-01 était inapplicable faute de générateur. Retenus : `openapi-typescript` **7.13.0** (types seulement) + `openapi-fetch` **0.17.0** (runtime écrit à la main) + `typescript` **7.0.2** (peerDependency). Critère de choix : minimiser la surface générée soumise à P-01. Écartés avec motif : `@hey-api/openapi-ts` 0.99.0 (`0.x`), `orval` 8.23.0, `oazapfts` 7.5.0. Deux exigences à valider au cycle 1 avant de clore US5 : déterminisme d'octet vérifié par `cmp`, et ordre de membres stable indépendant de l'ordre de découverte utoipa. |
| 1.0.2 | 2026-07-30 | **Cible de déploiement arrêtée : Docker sur VPS Contabo** (mode A). **PostgreSQL `18.4` confirmée, arbitrage fermé** — version maîtrisée en auto-géré, EOL 2030-11-14 retenu pour la conservation fiscale de 10 ans ; `17.10` reste l'option du paquet auto-hébergé (mode B). Ajout du §4.2 : les trois images Docker vérifiées disponibles en **amd64 et arm64**, donc un seul `compose.yml` pour le poste Apple Silicon et le VPS. Consigné : le binaire Rust n'est pas multi-architecture — construction de production **dans Docker pour `linux/amd64`**, jamais par copie locale. Consigné aussi : `dxflrs/garage` publie des tags de hash de commit qui masquent les tags sémantiques dans un tri par date — toujours interroger par nom. |
| 1.0.1 | 2026-07-30 | **Redis reculé de `8.10.0` à `8.8.1`** : `8.10.0` était en GA depuis un jour, avec neuf jours de RC, pour un nouvel encodage de hachage inutile à Kaya ; `8.8.1` porte les mêmes correctifs de sécurité du 2026-07-23 et deux mois de recul. **sqlx `0.9.0` confirmée** sur deux apports propres au projet (`#3918` erreur de violation d'exclusion pour HEB-02 ; `sqlx.toml` multi-schémas pour le principe II) et présence de `PgRange` vérifiée sur docs.rs. **PostgreSQL : arbitrage 18.4 / 17.10 ouvert**, rattaché à la décision B-01. Les neuf autres briques sont inchangées. |
| 1.0.0 | 2026-07-30 | Gel initial. 10 briques du principe XI + 14 crates + 3 paquets npm + Node LTS et pnpm. Compatibilité `utoipa-swagger-ui` / `utoipa-actix-web` avec `utoipa 5.5.0` vérifiée sur crates.io. Présence de la feature `uuid/v7` vérifiée. Deux points d'attention consignés : rupture d'API sqlx 0.8 → 0.9, et fraîcheur d'un jour de Redis 8.10.0. Dérogation raisonnée sur Node : LTS 24.18.1 retenue plutôt que la stable 26.5.1. |
