# Implementation Plan: Socle technique du monorepo Kaya

**Branch**: `001-socle-technique-monorepo` | **Date**: 2026-07-30 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/001-socle-technique-monorepo/spec.md`

**Artefacts de ce plan** : [research.md](./research.md) · [data-model.md](./data-model.md) ·
[contracts/http-api.md](./contracts/http-api.md) ·
[contracts/traits-exposes.md](./contracts/traits-exposes.md) · [quickstart.md](./quickstart.md)

---

## Summary

Ce cycle crée l'arborescence complète du monorepo, le grand livre d'événements, l'isolation
multi-tenant forcée, le contrat d'API généré, l'observabilité, les seeds, les provisions
comptables et les portes de CI qui rendent tout cela non contournable. Il ne livre **aucune
fonctionnalité métier**.

L'approche tient en trois décisions structurantes, toutes prises pour rendre une garantie
**mécanique plutôt que déclarative** :

1. **Le trait `OutboxWriter` prend une transaction en paramètre** et n'ouvre jamais la sienne.
   Écrire un événement hors de la transaction métier devient une erreur de compilation, pas un
   oubli de revue.
2. **Un troisième rôle PostgreSQL, `kaya_ledger_reader`**, n'a le droit de lire que la table
   d'événements. Le test de reconstitution autonome de TRX-02 ne *prétend* pas éviter les autres
   tables : il en est *empêché* par la base.
3. **Le tenant courant est posé par `set_config(..., true)` paramétré**, pas par `SET LOCAL`
   interpolé. La requête reste littérale, vérifiée à la compilation, et `AssertSqlSafe`
   n'apparaît jamais sur le chemin qui décide quelles lignes un client voit.

Le module doré — `note_etablissement`, **six couches**, écrit à la main — est produit **en
premier** et sert de patron à tous les cycles suivants. La septième couche, l'écran, est reportée
au cycle ETB : l'écran de notes n'hérite d'aucun motif de la matrice de dérivation
(`docs/Kaya_Design.md` §25), et un écran sans motif ne se code pas. **Ce cycle ne produit donc
aucun écran.**

---

## Technical Context

Toutes les versions viennent de `docs/versions-gelees.md` v1.0.2 (vérifié le 2026-07-30). **Aucune
n'est reproduite ici** : une copie dériverait à la première revue mensuelle. Ce tableau nomme les
briques et renvoie au gel.

**Language/Version** : Rust (toolchain épinglée par `rust-toolchain.toml`) · TypeScript sur
Node.js LTS (épinglé par `.nvmrc`)

**Primary Dependencies** : Actix Web · sqlx · utoipa + `utoipa-swagger-ui` + `utoipa-actix-web` ·
tokio · serde · `uuid` (feature `v7` — prérequis du principe VI) · `rust_decimal` · `aws-sdk-s3`
(Garage via API S3) · `sentry` · `tracing` + `tracing-subscriber` · Nuxt 4 · Tailwind 4 · Tauri v2
· `@nuxtjs/i18n`

**Storage** : PostgreSQL (seule vérité durable) · Redis (éphémère reconstructible **uniquement**)
· Garage (API S3 uniquement)

**Testing** : `cargo test` (unitaires + intégration sur base réelle) · Vitest côté application ·
scripts de porte en CI

**Target Platform** : production `linux/amd64` (Docker sur VPS Contabo, mode A du cadrage §10.1) ·
développement `darwin/arm64` · application desktop, Android, iOS

**Project Type** : monorepo — workspace Rust + application Nuxt/Tauri unique + deux surfaces web
séparées

**Performance Goals** : aucune cible chiffrée à ce cycle. La seule métrique suivie est le **temps
de compilation incrémentale**, mesuré dans le conteneur Linux (SC-010) — une mesure sur le poste
macOS ne prédit rien de la production.

**Constraints** : construction de production **dans Docker pour `linux/amd64`**, jamais par copie
d'un binaire local · toute dépendance native doit exister pour les deux architectures (voir
Complexity Tracking, écart 1) · sqlx 0.9 impose `AssertSqlSafe` sur toute requête non littérale et
change la sortie des macros `query!()` — **tout extrait trouvé en ligne vise 0.8.x et ne compilera
pas**

**Scale/Scope** : 2 tenants seedés · 6 tables · 3 schémas PostgreSQL · 3 endpoints · 1 type
d'événement · **0 écran** · 20 portes de CI

**NEEDS CLARIFICATION** : **aucun générateur de client OpenAPI → TypeScript ne figure au gel**
(`research.md` R-14). C'est le seul blocage dur du cycle : sans lui, la porte P-01 ne peut pas
être livrée. Le choix de l'outil et la vérification de sa version sur son registre officiel
relèvent de la **revue de gel**, pas de ce plan — conformément à la consigne « ne propose aucun
numéro de version ».

---

## Constitution Check

*GATE: à passer avant Phase 0, revérifié après Phase 1.*

### Conformité aux douze principes

| Principe | Statut | Comment ce cycle s'y conforme |
|---|---|---|
| I — Sources de vérité | ✅ | Contrat généré par utoipa ; client généré en CI ; schéma par migrations sqlx uniquement ; aucun paramètre en dur |
| II — Architecture et hiérarchie des crates | ✅ | 14 crates créés selon la hiérarchie ; un schéma par module ; `OutboxWriter` prend la transaction ; worker in-process, aucune file externe |
| III — Isolation multi-tenant | ✅ | `ENABLE` + `FORCE` sur les 6 tables ; 3 rôles distincts ; `set_config` par transaction ; deux tests de porte |
| IV — Temps et disponibilité | ⚠️ partiel | Horodatage d'autorité serveur posé et distinct de l'horodatage client. Aucune occupation à ce cycle — `EXCLUDE USING gist` est néanmoins exercé sur `exercice_comptable` (spike de HEB-02) |
| V — Argent et fiscalité | ⚠️ partiel | Montants en entiers d'unité mineure et taux en millièmes dans la charge utile. `JurisdictionAdapter` **déclaré, non implémenté** — les règles fiscales sont T3 |
| VI — Hors-ligne | ✅ | `note_etablissement` déclarée classe A ; UUID v7 client ; tests de rejeu et de désordre ; porte du registre |
| VII — Application unique, rôles cumulés | ✅ | Une application Nuxt + Tauri ; `PlatformAdapter` avec `ResultatCapacite` ; aucun `window.__TAURI__` hors adaptateur |
| VIII — Qualité, i18n, observabilité | ✅ | Tests d'intégration sur la transition d'état ; `sqlx prepare` ; clés fr/en ; mode sombre dès le premier écran ; `tracing` corrélé + Sentry + `/health` |
| IX — Sécurité | ⚠️ partiel | Trois rôles, journal d'événements immuable. Enrôlement d'appareil et journal d'audit sont CPT — hors périmètre |
| X — Périmètre « prêt ≠ construit » | ✅ | TRX-02b en tables seulement ; `JurisdictionAdapter` déclaré sans implémentation ; TRX-06/07/08 en emplacement seul |
| XI — Versions épinglées | ⚠️ **1 trou** | Gel repris tel quel, aucune version proposée. **Le générateur de client TS manque au gel** (R-14) |
| XII — Référence visuelle | ✅ | `theme.css` copié tel quel — seule exception ; **aucun écran produit** (aucun motif d'héritage, `docs/Kaya_Design.md` §25) ; aucun HTML de maquette copié sous `app/` |

Les trois ⚠️ « partiel » sont des principes dont ce cycle ne touche qu'une partie du domaine : ils
ne sont pas violés, ils ne sont pas encore pleinement exercés. Le ⚠️ du principe XI est un **trou
réel**, traité en Complexity Tracking.

### Les vingt portes — mécanisme de vérification de chacune

> Exigence de la consigne : *« une porte concernée sans mécanisme de vérification est un trou du
> plan »*. Les 20 portes sont donc traitées, y compris celles sans cible.

| Porte | Vérifie | Touchée | Mécanisme et test |
|---|---|---|---|
| **P-01** | Client TS identique au client commité | ✅ | Job CI : régénère depuis `/api-docs/openapi.json` puis `git diff --exit-code clients/ts`. **BLOQUÉ par R-14** |
| **P-02** | Aucune migration appliquée modifiée | ✅ | `scripts/ci/migrations-figees.sh` : empreinte de chaque fichier de migration comparée à la branche de base ; échec si un fichier existant diffère |
| **P-03** | `socle/` ne dépend pas de `verticales/` | ✅ | `backend/tests/architecture.rs` : lit `cargo metadata`, parcourt les arêtes du graphe, échoue sur toute arête interdite. Couvre aussi `capacites/ → verticales/` |
| **P-04** | Aucune requête ne joint deux schémas de modules | ✅ | `scripts/ci/jointures-inter-schemas.sh` : détecte deux préfixes de schéma distincts dans une même requête (`.sql` et macros `query!`). **Heuristique assumée** — complétée par la revue mensuelle |
| **P-05** | Toute transition d'état émet un événement dans sa transaction | ✅ | Deux niveaux : (a) la signature d'`OutboxWriter` rend l'écriture hors transaction impossible à compiler ; (b) `tests/outbox_transactionnel.rs` — après chaque mutation exposée, un événement existe ; après un rollback provoqué, **ni ligne métier ni événement** |
| **P-05b** *(ajoutée par ce plan)* | Aucun chemin de suppression d'événement | ✅ | `REVOKE DELETE` + déclencheur (data-model §4.1) + `scripts/ci/outbox-sans-purge.sh` qui échoue sur tout `DELETE`/`TRUNCATE` visant `evenement_outbox` |
| **P-06** | Capacité ≠ `STOCK`/`SIMPLE` refusée explicitement | ⬜ à vide | Aucun référentiel de capacités à ce cycle (ETB-02b). Test installé, assertion de non-régression : échoue si la porte cesse de trouver une cible **après** ETB-02b (R-15) |
| **P-07** | Toute table a une politique RLS, `ENABLE` et `FORCE` | ✅ | `backend/tests/rls_catalogue.rs` : interroge `pg_class.relrowsecurity`, `relforcerowsecurity` et `pg_policies`. **Lit le catalogue, pas les migrations.** Liste d'exclusion **nommée** (tables sqlx), jamais un motif |
| **P-08** | Tenant A ne lit ni n'écrit chez B, sur chaque endpoint | ✅ | `backend/tests/isolation_tenant.rs` : **paramétré sur la liste des routes de l'OpenAPI**, pour qu'un endpoint ajouté sans test fasse échouer la porte |
| **P-09** | Occupation en `tstzrange` protégé par exclusion GiST | ⬜ à vide | Aucune occupation (HEB-02). **Partiellement exercé** : `exercice_comptable` utilise `EXCLUDE USING gist` sur `daterange` — valide `btree_gist` et le mapping sqlx 0.9 avant que HEB-02 en dépende |
| **P-10** | Aucun montant non entier, aucune quantité non `NUMERIC` | ⬜ quasi à vide | `scripts/ci/types-monetaires.sh` : analyse les migrations, échoue sur `FLOAT`/`REAL`/`DOUBLE` pour un montant et sur tout entier nommé `quantite` |
| **P-11** | Tests dorés fiscaux verts | ⬜ à vide | Aucun calcul fiscal (T3). Harnais installé avec un jeu vide + assertion de non-régression |
| **P-12** | Aucune règle fiscale hors `JurisdictionAdapter` | ⬜ à vide | `backend/tests/architecture.rs` : aucun crate hors `socle/fiscalite` ne référence les types de taxe du crate `domain` |
| **P-13** | Aucune opération B/C/D atteignable hors ligne | ✅ | La file locale n'accepte que des types **marqués classe A** au niveau du type ; `app/core/sync` refuse à la compilation l'enregistrement d'un type non-A. Test : `app/tests/file-classe-a.spec.ts` |
| **P-14** | Rejeu triple = 1 enregistrement ; désordre commutatif | ✅ | `backend/tests/note_etablissement_classe_a.rs` — les deux tests du §0.7 sur `note_etablissement` |
| **P-15** | Aucun `window.__TAURI__` hors `PlatformAdapter` | ✅ | Règle ESLint `no-restricted-imports` sur `@tauri-apps/api`, avec dérogation nommée au seul répertoire `app/core/platform/` |
| **P-16** | Aucune chaîne en dur ; parité fr/en | ✅ | `pnpm test:i18n` : compare les jeux de clés `fr` et `en` (échec sur toute asymétrie) + analyse des littéraux affichés dans les templates |
| **P-17** | Aucune couleur ni espacement littéral hors jetons | ✅ | `pnpm lint:tokens` : échoue sur tout `#rrggbb`, `rgb(`, `px` hors des jetons de `theme.css` |
| **P-18** | `cargo sqlx prepare` vert | ✅ | CI : `cargo sqlx prepare --workspace --check` ; `.sqlx/` commité |
| **P-19** | Aucun fichier de `docs/design/html/` copié sous `app/` | ✅ | `scripts/ci/maquettes-non-copiees.sh` : empreintes de `docs/design/html/**` comparées à tout fichier sous `app/`. **`theme.css` explicitement exclu** — c'est la seule exception du principe XII |
| **P-20** | Aucun intervalle de version ; lockfiles à jour | ✅ | `scripts/ci/versions-epinglees.sh` : échoue sur tout `^`, `~`, `*` ou plage dans `Cargo.toml`/`package.json`, sur tout tag `latest` dans `compose.yml`, puis `cargo build --locked` et `pnpm install --frozen-lockfile` |

**Bilan** : 14 portes actives, 5 installées à vide avec assertion de non-régression, 1 ajoutée
(P-05b), 1 bloquée par un trou du gel (P-01).

---

## Project Structure

### Documentation (this feature)

```text
specs/001-socle-technique-monorepo/
├── plan.md               # Ce fichier
├── spec.md               # Spécification (avec Clarifications)
├── research.md           # Phase 0 — décisions techniques
├── data-model.md         # Phase 1 — migrations, RLS, classes hors-ligne
├── quickstart.md         # Phase 1 — guide de validation
├── contracts/
│   ├── http-api.md       # Endpoints et annotations utoipa
│   └── traits-exposes.md # Traits inter-crates
├── checklists/
│   └── requirements.md
└── tasks.md              # Phase 2 — produit par /speckit-tasks
```

### Source Code (repository root)

```text
backend/
├── Cargo.toml                    # workspace, [workspace.dependencies] en versions exactes
├── rust-toolchain.toml           # canal exact — jamais "stable"
├── sqlx.toml                     # multi-schémas + renommage de _sqlx_migrations
├── crates/
│   ├── domain/                   # types, règles, moteur fiscal — PARTAGÉ api/node/tauri
│   ├── socle/
│   │   ├── etablissements/       # tenant, etablissement, note_etablissement, EstablishmentDirectory
│   │   ├── comptes/              # vide, compile
│   │   ├── caisse/               # vide, compile
│   │   ├── fiscalite/            # JurisdictionAdapter (DÉCLARÉ) + provisions comptables
│   │   ├── documents/            # vide, compile
│   │   ├── synchronisation/      # evenement_outbox, OutboxWriter, EventConsumer, worker
│   │   ├── pilotage/             # vide, compile
│   │   ├── editeur/              # vide, compile
│   │   └── metriques/            # vide, compile
│   ├── capacites/
│   │   └── stocks/               # vide, compile
│   └── verticales/
│       ├── hebergement/          # vide, compile
│       ├── restauration/         # vide, compile
│       ├── bar/                  # vide, compile
│       └── pressing/             # vide, compile
├── api/                          # binaire Actix — migrations au démarrage, puis écoute
├── node/                         # binaire nœud de site — coquille, incrément 3
├── migrations/                   # 0001 → 0005
│   └── seeds/                    # rejouables, SÉPARÉS des migrations
└── tests/                        # architecture, rls_catalogue, isolation_tenant, outbox_*

app/                              # Nuxt 4 + Tauri v2 — APPLICATION UNIQUE, SPA
├── core/                         # auth · rbac · i18n(fr,en) · theme · sync · platform/
├── modules/
│   └── etablissements/           # coquille — l'écran de notes est reporté au cycle ETB
├── assets/css/theme.css          # COPIE EXACTE de docs/design/theme.css (seule exception XII)
└── src-tauri/                    # coquille Rust

web/
├── qr/                           # page publique de commande (SSR) — coquille
└── console/                      # console éditeur (ssr:false) — coquille

clients/ts/                       # client généré — JAMAIS édité à la main
infra/
├── compose.yml                   # tags d'image exacts — jamais "latest"
├── Dockerfile.api                # build multi-étapes pour linux/amd64, mold côté Linux
├── backup/                       # sauvegarder.sh, restaurer.sh
└── autoheberge/                  # EMPLACEMENT SEUL — paquet mode B livré avec TRX-07
docs/
├── module-dore.md                # PRODUIT par ce cycle
├── registre-classes-offline.md   # + note_etablissement
└── conformite/                   # EMPLACEMENT SEUL — registre ARTCI, TRX-06
scripts/ci/                       # scripts des portes P-02, P-04, P-05b, P-10, P-19, P-20
.github/workflows/                # CI filtrée par chemins
```

**Structure Decision** : l'arborescence est **imposée** par `docs/Kaya_Prompts_SpecKit.md` §0.1 et
la constitution (principe II). Elle n'est pas un choix de ce plan. Les seuls ajouts sont
`scripts/ci/`, `backend/tests/` et les deux répertoires d'emplacement (`infra/autoheberge/`,
`docs/conformite/`), qui matérialisent les portes et le hors-périmètre.

---

## Phases

### Phase 0 — Recherche ✅ terminée

Voir [research.md](./research.md). Seize décisions, dont deux qui changent le plan :

- **R-01** — `mold` n'existe pas sur macOS. L'exigence est scindée : Linux et CI seulement.
- **R-14** — aucun générateur de client TypeScript au gel. **Blocage dur de P-01.**

### Phase 1 — Conception ✅ terminée

[data-model.md](./data-model.md) · [contracts/](./contracts/) · [quickstart.md](./quickstart.md).

**Constitution Check post-conception** : réévalué ci-dessus après rédaction du modèle et des
contrats. Aucune violation nouvelle. Les deux écarts de périmètre identifiés à la conception sont
consignés en Complexity Tracking plutôt que résolus en silence.

### Phase 2 — Tâches ⬜ à produire par `/speckit-tasks`

**Ordre imposé, non parallélisable** :

1. Arborescence + workspace compilable + gel matérialisé (lockfiles, `rust-toolchain.toml`)
2. Migrations 0001 → 0003 (rôles, schémas, outbox)
3. **Module doré écrit à la main** — migration 0004, repository, service, handler, tests (six
   couches ; la couche écran est reportée au cycle ETB)
4. `docs/module-dore.md` + `note_etablissement` au registre
5. Portes de CI
6. Reste : observabilité, sauvegardes, seeds, provisions

L'étape 3 précède **tout** le reste du code (cadrage §13.1 : « avant toute génération assistée »).
`/speckit-tasks` ne doit pas la paralléliser.

---

## Complexity Tracking

| Écart | Pourquoi nécessaire | Alternative écartée parce que |
|---|---|---|
| **1. `mold` sur Linux uniquement** | Le poste de développement est macOS ; `mold` est un linker ELF et ne supporte pas Mach-O. La consigne « toute dépendance native doit exister pour les deux architectures » l'exclut donc du poste | L'imposer partout échoue au lien. `sold` est un produit commercial hors du gel. Renoncer perdrait le gain là où il compte — la CI et l'image de production, toutes deux Linux |
| **2. `tenant` et `etablissement` créés en forme minimale** | `note_etablissement` doit se rattacher à un établissement, `evenement_outbox` porte `etablissement_id`, et la RLS n'a rien à isoler sans tenant réel | Les créer complets empiéterait sur ETB-01. Ne pas les créer laisserait une table orpheline et une RLS non testable. ETB-01 les enrichira par **migration additive**, jamais par modification |
| **3. Provisions comptables logées dans `socle/fiscalite`** | La constitution fixe limitativement neuf crates de socle ; il n'existe pas de crate `comptabilite` et en créer un exigerait un amendement | `documents` traite la numérotation des pièces, pas leur traduction comptable. Un dixième crate demanderait `/speckit-constitution` pour une provision sans logique |
| **4. TRX-05 livré en portée réduite** | Les 17 unités, le catalogue et les comptes de test (FR-062) peuplent des tables qui **n'existent pas encore** — elles appartiennent à HEB, PDV et CPT | Créer ces tables ici reviendrait à faire trois cycles d'avance sans leurs règles métier. Ce cycle livre **la mécanique de seeds et les deux tenants** ; le contenu s'ajoute à chaque cycle. **Décision à confirmer** — voir ci-dessous |
| **5. P-04 vérifiée par heuristique** | Détecter de façon exacte qu'une requête joint deux schémas demanderait d'analyser le SQL, pas de le filtrer | Une analyse complète du SQL est disproportionnée à ce stade. L'heuristique attrape le cas courant ; la revue mensuelle couvre le reste. **La limite est écrite dans le script**, pas masquée |

### Deux points appelant une décision avant `/speckit-tasks`

**A — Le générateur de client TypeScript manque au gel (R-14).** C'est le seul blocage dur : sans
outil, pas de client généré, donc pas de porte P-01, donc US5 échoue. Aucun outil ni aucune
version n'est proposé ici, conformément à la consigne. Il faut ouvrir une revue de gel ponctuelle
pour l'ajouter, avec vérification sur registre officiel et URL citée. Les critères que l'outil
doit satisfaire sont arrêtés en R-14 : sortie déterministe, exécutable sous la LTS gelée,
disponible sur les deux architectures, sans dépendance à un runtime Java.

**B — La portée de TRX-05 (écart 4).** Ce cycle ne peut pas seeder 17 unités et 30 articles :
leurs tables viennent des cycles HEB et PDV. Le plan livre la mécanique et les deux tenants, et le
dit plutôt que de laisser FR-062 partiellement satisfait en silence. Si TRX-05 ne doit être
considérée close que lorsque les seeds sont complets, elle devient une story à cheval sur toute la
tranche T1.

Aucun de ces deux points n'empêche `/speckit-tasks` de produire les tâches ; le premier empêche
seulement `/speckit-implement` de clore US5.
