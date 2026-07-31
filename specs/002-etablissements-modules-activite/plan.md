# Implementation Plan: Tenants, établissements, modules d'activité et configuration héritée

**Branch**: `002-etablissements-modules-activite` (aucune branche git dédiée — travail sur `main`, comme au cycle 001) | **Date**: 2026-07-31 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/002-etablissements-modules-activite/spec.md`

---

## Summary

Le cycle livre l'entité centrale du produit et les deux référentiels qui décident de son
extensibilité. Six migrations additives créent **dix tables** dans le schéma `etablissements` et en
enrichissent une onzième, six traits les exposent aux cycles suivants, vingt et une opérations HTTP
les servent, et un écran — `G1`, dérivé de la maquette `G2` — les rend utilisables.

> **Trois décomptes, tenus distincts d'un bout à l'autre du cycle** : **10 tables créées** (cible de
> la porte P-07), **11 entités au registre** des classes hors-ligne (`etablissement` y figure déjà),
> **11 types d'événements** et **21 opérations HTTP** (cibles des portes P-05 et P-08). Les
> confondre ferait inspecter un sous-ensemble en croyant tout couvrir.

**Trois choses distinguent ce cycle d'un cycle de CRUD.**

*Le refus explicite est structurel, pas applicatif.* Une capacité non implémentée est refusée par
une **clé étrangère composite plus un `CHECK`** — le référentiel porte `implementee`, la
déclaration le recopie et exige qu'il soit vrai. Un import, un script de reprise ou un jeu de
données ne peut pas contourner ce que le service refuse. La porte P-06, installée à vide au cycle
001, acquiert ici ses premières cibles.

*Les trois parcours structurels sont un harnais à étapes dues.* Ils sont verts dès ce cycle sur
les quatre étapes livrables et déclarent quatre étapes dues à PDV, CAI et FIS, chacune avec une
**sentinelle observable**. Le jour où un cycle rend une étape réalisable sans la brancher,
l'intégration continue échoue en le nommant. Aucun test n'est jamais rouge ni ignoré.

*La résolution de configuration est conçue avant son premier consommateur.* Écrite au cycle HEB,
elle serait teintée d'hébergement ; écrite ici, elle sert les huit cycles qui la liront.

Trois pièges de PostgreSQL sous sécurité au niveau ligne forcée ont été identifiés en phase 0, et
chacun **réussit en silence** s'il n'est pas connu : un `INSERT` de migration dans un référentiel
sans politique d'écriture pour le propriétaire, un `UPDATE` de migration qui ne touche aucune ligne
sans lever d'erreur, et une unicité qui laisse passer deux surcharges de même clé parce que
`NULL ≠ NULL`.

---

## Technical Context

Toutes les valeurs viennent de **`docs/versions-gelees.md`, reprises telles quelles**. Aucune n'est
proposée de mémoire ni revérifiée : le gel est daté, sourcé, et sa prochaine revue groupée est le
2026-08-31 (principe XI).

**Language/Version**: Rust `1.97.1` (`rust-toolchain.toml`, jamais `stable`) · Node.js `24.18.1`
LTS · pnpm `11.18.0`

**Primary Dependencies**: Actix Web `4.14.0` · sqlx `0.9.0` ⚠️ · utoipa `5.5.0` ·
utoipa-swagger-ui `9.0.2` · utoipa-actix-web `0.1.2` · Nuxt `4.5.1` · Tailwind CSS `4.3.3` ·
Tauri `2.11.5` · `@nuxtjs/i18n` `10.6.0` · `openapi-typescript` `7.13.0` · `openapi-fetch` `0.17.0`
· `uuid` `1.24.0` (feature `v7`) · `aws-sdk-s3` `1.140.0` (ETB-05, via API S3 uniquement)

**Storage**: PostgreSQL `18.4` — seule vérité durable. Redis `8.8.1` — **non utilisé par ce
cycle** : rien ici n'est éphémère ni reconstructible. Garage `2.3.0` via l'API S3 — le seul objet
stocké est le logo d'identité visuelle.

**Testing**: `cargo test` sur **base PostgreSQL réelle**, jamais sur simulacre — l'essentiel de ce
que le cycle garantit (politiques de sécurité, clés étrangères composites, unicité
`NULLS NOT DISTINCT`) n'existe que dans la base. `vitest 4.1.10` côté application.

**Target Platform**: Docker sur VPS Contabo, **`linux/amd64`** (mode A du cadrage §10.1). Poste de
développement `arm64`. Les images `postgres:18.4`, `redis:8.8.1` et `dxflrs/garage:v2.3.0` sont
multi-architecture ; **le binaire Rust ne l'est pas** — la construction de production se fait dans
Docker pour `linux/amd64`, jamais par copie d'un binaire local. **Ce cycle n'ajoute aucune
dépendance native, aucun plugin Tauri et aucun outil de construction** : la contrainte des deux
architectures est satisfaite sans vérification supplémentaire.

**Project Type**: monolithe modulaire Rust + application unique Nuxt 4 / Tauri v2, monorepo.

**Performance Goals**: résolution d'un paramètre en **une seule descente de chaîne** — une requête,
jamais quatre ; `resoudre_tout` rend l'ensemble des paramètres d'une cible en un aller-retour
(l'écran `G1` en affichera une trentaine à terme). Aperçu d'identité visuelle sous 2 s (SC-009).

**Constraints**: sqlx `0.9.0` impose `AssertSqlSafe` sur toute requête non littérale et a modifié
la sortie des macros `query!()` — **tout extrait trouvé en ligne vise `0.8.x` et ne compilera
pas**. Le patron est `docs/module-dore.md`, produit par le cycle 001 : ce cycle le **suit**, il ne
le réécrit pas. Toutes les requêtes passent par les macros sur littéral ; `AssertSqlSafe`
n'apparaît nulle part.

**Scale/Scope**: 6 migrations · **10 tables créées** (dont 4 référentiels globaux) + 1 enrichie ·
11 entités au registre · 6 traits exposés · 21 opérations HTTP · 11 types d'événements ·
**1 écran** (`G1`) · 2 jeux de données de démonstration.

---

## Constitution Check

*GATE : franchi avant la phase 0, re-vérifié après la phase 1. Aucune violation non justifiée.*

### Conformité aux douze principes

| # | Principe | Comment ce cycle le tient |
|---|---|---|
| I | Sources de vérité | Contrat généré depuis utoipa, client régénéré en CI. Migrations versionnées, `0002` **non modifiée**. Paramètres dans la configuration + **porte de cohérence** catalogue → récapitulatif |
| II | Architecture et hiérarchie | Tout dans le schéma `etablissements`. Aucune clé étrangère inter-modules (`caisse_id` sans référence). `ObstacleDesactivation` **inverse la dépendance** au lieu de la créer. Un événement par transition, dans la transaction |
| III | Isolation multi-tenant | `ENABLE` + `FORCE` sur les **dix tables créées**. `set_config('app.current_tenant', $1, true)`, jamais `SET LOCAL`. Les quatre référentiels globaux ont un régime **nommé**, pas une exemption |
| IV | Temps et disponibilité | Horodatages d'autorité serveur (`now()` en SQL). Le fuseau appartient à l'établissement. **Aucune occupation créée** — P-09 sans cible |
| V | Argent et fiscalité | Devise ISO 4217 sur l'établissement, figée après la première opération financière. **Aucune règle fiscale en base** : le contrôle de forme du NCC est volontairement minimal, sa validité relève du `JurisdictionAdapter` |
| VI | Hors-ligne | Onze entités en classe **C**. UUID v7 client sur toute écriture, `200` sur rejeu. Distinction **écriture C / lecture en cache A** ajoutée au registre — le **mécanisme** de cache et le témoin de fraîcheur relèvent de SYN-01/02 et d'ETB-06, hors périmètre |
| VII | Application unique, rôles cumulés | Un seul écran, dans l'application unique. **Un service inactif est absent** — et les traits ne l'exposent même pas. Chargement paresseux par import dynamique |
| VIII | Qualité, i18n, observabilité | Tests d'intégration sur les transitions. `cargo sqlx prepare` **complet**. Clés fr et en, fr par défaut. `G1` vérifié en clair et en sombre — premier écran du produit |
| IX | Sécurité | Aucun secret nouveau. Les changements sensibles (classement, fuseau) ont leur type d'événement propre. Le journal d'audit consultable relève de CPT-04 |
| X | Périmètre — prêt ≠ construit | ETB-07 et ETB-08 : **aucune table, aucune ligne de code**. Les référentiels étant en table, ETB-08 est déjà satisfaite sans écrire une valeur |
| XI | Versions épinglées | Gel repris tel quel. Deux paquets de test front manquent au §3.2 : **signalés sans version**, vérifiés et épinglés au moment de l'ajout, portés au gel à la revue du 2026-08-31 |
| XII | Référence visuelle | `G1` **hérite de `G2`** (matrice de dérivation). La maquette est lue, jamais copiée. Tailwind d'abord, variante `dark:`, aucune classe personnalisée |

### Les vingt et une portes — mécanisme de vérification de chacune

**Dix-sept portes sont touchées par ce cycle. Quatre ne le sont pas et restent vertes à vide.**
Deux portes doivent être **étendues** par ce cycle, faute de quoi elles laisseraient passer ce
qu'elles sont censées attraper.

| Porte | Touchée | Mécanisme de vérification | Test |
|---|---|---|---|
| **P-01** client TS identique | ✅ 21 opérations | Génération puis `git diff --exit-code` sur `clients/ts` | `scripts/ci/generer-client.sh` + CI |
| **P-02** migrations figées | ✅ 6 nouvelles | Empreinte de chaque migration appliquée comparée au dépôt. `0002` **doit** rester intacte | `scripts/ci/migrations-figees.sh` |
| **P-03** socle ↛ verticales | ✅ **risque réel** | Graphe `cargo metadata`. `ObstacleDesactivation` est **défini** dans le socle, **implémenté** par les verticales, **injecté** dans `api/` | `backend/tests/architecture.rs` |
| **P-04** pas de jointure inter-schémas | ✅ | Analyse des requêtes. La résolution joint `parametre_configuration` et `etablissement_module` — **même schéma**, conforme | `scripts/ci/jointures-inter-schemas.sh` |
| **P-05** événement dans la transaction | ✅ 11 types | Rollback provoqué : ni ligne métier ni événement. Un cas par type d'événement, **et un décompte des types vérifiés comparé aux 11 déclarés** — un type ajouté sans test fait échouer la porte | `backend/tests/outbox_transactionnel.rs` (étendu) + tâche de recollement |
| **P-05b** outbox sans purge | ⬜ inchangée | Aucun chemin de suppression ajouté | `scripts/ci/outbox-sans-purge.sh` |
| **P-06** capacité ≠ `STOCK`/`SIMPLE` refusée | ✅ **porte centrale** | **Neuf cas** — 6 capacités, 3 profils. Vérifiés à deux niveaux : `422` par l'API **et** violation de contrainte sur `INSERT` direct sous le rôle applicatif | `backend/tests/capacites_refusees.rs` **(nouveau)** |
| **P-07** RLS sur toute table | ✅ **10 tables créées** | Catalogue système : `relrowsecurity` **et** `relforcerowsecurity`, au moins une politique. Les 4 référentiels sont comptés **conformes et nommés** — deux politiques, pas d'isolation par tenant. **Ré-exécutée après la dernière migration**, avec décompte des tables inspectées | `backend/tests/rls_catalogue.rs` (étendu) + tâche de recollement |
| **P-08** isolation A/B par endpoint | ✅ 21 opérations | Paramétré sur `application::contrat_complet()` — jamais sur le squelette. Assertion **explicite** que les référentiels rendent la même chose aux deux tenants. **Décompte des chemins couverts comparé aux chemins servis** : une opération ajoutée sans régime d'isolation fait échouer la porte | `backend/tests/isolation_tenant.rs` (étendu) + tâche de recollement |
| **P-09** occupation GiST | ⬜ sans cible | Aucune occupation créée. Reste verte à vide | `backend/tests/portes_a_vide.rs` |
| **P-10** montants entiers, quantités `NUMERIC` | ⚠️ **à ÉTENDRE** | Aucun montant en colonne, **mais** le catalogue déclare un type `MONTANT_MINEUR` dont la valeur vit en `JSONB`. Sans extension, un montant en flottant entrerait par là. La validation d'écriture doit refuser un `JSONB` non entier sur ce type | `scripts/ci/types-monetaires.sh` **(étendu)** + test de validation |
| **P-11** tests dorés fiscaux | ⬜ sans cible | Aucun calcul fiscal | `backend/tests/portes_a_vide.rs` |
| **P-12** fiscalité confinée | ✅ **tentation réelle, deux fois** | Le `CHECK` sur `ncc` se limite à « non vide » **et celui sur `etoiles` à « strictement positif »** : le format du numéro de contribuable comme le nombre maximal d'étoiles sont fixés par la réglementation nationale, donc par le `JurisdictionAdapter`. Un plafond `BETWEEN 1 AND 5` en base serait une règle de juridiction déguisée en contrainte d'intégrité | `backend/tests/architecture.rs` |
| **P-13** aucune opération C hors ligne | ✅ **11 entités C** | Backend : aucun chemin d'écriture ETB atteignable depuis la file locale. Front : `TYPES_CLASSE_A` **ne reçoit aucun type de ce cycle**, et le typage refuse la mise en file | `backend/tests/classes_offline.rs` + `app/tests/file-classe-a.spec.ts` |
| **P-14** rejeu triple d'une écriture A | ⬜ sans cible | Aucune entité de classe A. L'idempotence des écritures C est néanmoins testée (`201`/`200`/`200`) | `backend/tests/portes_a_vide.rs` |
| **P-15** pas de `window.__TAURI__` hors adaptateur | ✅ premier écran | Règle ESLint `no-restricted-imports`, **exécutée par `pnpm lint` dans la phase de validation** — une règle jamais déclenchée ne garde rien. **`G1` n'appelle aucune capacité native** : le choix de fichier du logo est un `<input type="file">` standard | `app/eslint.config.js` via `pnpm lint` |
| **P-16** i18n, parité fr/en | ✅ **premier écran** | Parité des catalogues + **sept termes ajoutés au lexique avant d'être codés**. Les libellés de référentiel sont des **clés** en base, jamais des textes | `app/scripts/test-i18n.ts` |
| **P-17** aucune couleur littérale | ⚠️ **à ÉTENDRE** | `G1` n'emploie que des jetons. **Mais** `branding.couleur_primaire` est une **donnée client** stockée en hexadécimal : la porte doit l'exclure explicitement, sinon elle échoue à tort — ou, pire, on la contourne en désactivant la règle sur le fichier | `app/scripts/lint-tokens.ts` **(étendu)** |
| **P-18** `cargo sqlx prepare` vert | ✅ | `--check --workspace -- --all-targets`, et **décompte des requêtes mises en cache comparé au total**. Le cycle 001 en validait 43 sur 47 | `scripts/ci/preparer-sqlx.sh` |
| **P-19** maquettes non copiées | ✅ `G1` dérive de `G2` | Aucun fichier de `docs/design/html/` sous `app/`. Seul `theme.css` est copié, déjà en place | `scripts/ci/maquettes-non-copiees.sh` |
| **P-20** versions épinglées | ✅ si ajout front | Aucun intervalle, lockfiles commités. Les deux paquets de test front, s'ils sont ajoutés, sont épinglés exactement avec l'URL de leur registre | `scripts/ci/versions-epinglees.sh` |

**Les trois portes à décompte.** P-05, P-07 et P-08 portent chacune sur un ensemble dont la taille
est connue — 11 types d'événements, 10 tables, 21 opérations. Leur extension est répartie sur
plusieurs phases, ce qui est la manière normale de les faire grandir avec le code, **et la manière
normale de laisser un trou**. Une **tâche de recollement** en fin de cycle compare, pour chacune, le
nombre de cibles réellement inspectées au total déclaré, et échoue sur tout écart. C'est
l'exigence 2 du § « Couverture des portes » appliquée aux trois portes qui s'étendent
progressivement.

**Porte supplémentaire livrée par ce cycle** — cohérence du catalogue de paramètres :
`backend/tests/parametres_catalogue.rs` vérifie que **toute clé du catalogue figure au
« Récapitulatif des paramètres d'établissement »** de `docs/user-stories-v1.md`. Comparaison
asymétrique, catalogue → récapitulatif, comme celle de `classes_offline.rs`. Elle rend le
principe I·c vérifiable au lieu de seulement écrit.

### Tests hors-ligne obligatoires — §0.7 des user stories

Les onze entités sont de **classe C**. Le §0.7 impose donc, pour chacune :

- **un test qui échoue si l'opération est atteignable depuis un chemin de code exécutable hors
  ligne** — porté par `classes_offline.rs` côté base et par le typage de `app/core/sync/classes.ts`
  côté application, dont la liste `TYPES_CLASSE_A` ne doit recevoir **aucun** type de ce cycle ;
- **un test d'isolation multi-tenant sur l'endpoint** — les vingt et une opérations, sans exception.

Les tests de rejeu et de désordre (classe A) et le scénario orphelin (entité rattachée à un séjour)
**sont sans objet** : aucune entité de classe A, aucun séjour. Consigné plutôt que passé sous
silence.

### Écran concerné

| Écran | Origine | Sections | Vérifications |
|---|---|---|---|
| **`G1` — Établissement et modules** | **Hérite de `G2`** (`docs/design/derivation.md`). Maquette lue : `docs/design/html/G2-offre-hebergement.html` | Identité (ETB-01) · Vos services et capacités (ETB-02, ETB-02b) · Points de vente (ETB-03) · Identité visuelle avec aperçu (ETB-05) | Mode clair **et** sombre · clés fr/en · aucun service inactif dans le HTML rendu · aucune couleur littérale |

**Aucun autre écran.** L'accueil `R1` reste au cycle CPT — son filtrage par permission dépend de
rôles qui n'existent pas. L'écran de note interne, dette du cycle 001, n'hérite toujours d'aucun
motif : il se maquette avant de se coder.

---

## Project Structure

### Documentation (this feature)

```text
specs/002-etablissements-modules-activite/
├── plan.md                      # Ce fichier
├── spec.md                      # 7 stories · 80 exigences · 14 critères
├── research.md                  # Phase 0 — 14 décisions
├── data-model.md                # Phase 1 — 11 tables, 6 migrations
├── quickstart.md                # Phase 1 — 8 vérifications
├── contracts/
│   ├── http-api.md              # 21 opérations
│   └── traits-exposes.md        # 6 traits
├── checklists/requirements.md
└── tasks.md                     # Phase 2 — produit par /speckit-tasks
```

### Source Code (repository root)

Seuls les fichiers **créés ou modifiés** par ce cycle figurent ci-dessous. L'arborescence complète
date du cycle 001.

```text
backend/
├── migrations/
│   ├── 0007_etablissement_identite.sql          NOUVEAU — 7 colonnes, ADD COLUMN ... DEFAULT
│   ├── 0008_referentiels_activite.sql           NOUVEAU — 4 référentiels globaux
│   ├── 0009_activation_modules.sql              NOUVEAU — activation + déclaration de capacité
│   ├── 0010_points_de_vente.sql                 NOUVEAU — points de vente + tables
│   ├── 0011_configuration_heritee.sql           NOUVEAU — catalogue + valeurs
│   ├── 0012_branding.sql                        NOUVEAU — identité visuelle
│   └── seeds/                                   ÉTENDU — 2 tenants configurés
│
├── crates/socle/etablissements/src/
│   ├── lib.rs                                   ÉTENDU — Etablissement enrichi, Classement
│   ├── etablissement/{modele,repository,service}.rs    NOUVEAU — ETB-01
│   ├── modules/{modele,repository,service}.rs          NOUVEAU — ETB-02, ETB-02b
│   ├── points_de_vente/{modele,repository,service}.rs  NOUVEAU — ETB-03
│   ├── configuration/{modele,repository,service}.rs    NOUVEAU — ETB-04, le trait le plus lu
│   ├── branding/{modele,repository,service}.rs         NOUVEAU — ETB-05
│   └── traits.rs                                NOUVEAU — les 6 traits exposés
│
├── api/src/routes/
│   ├── etablissements.rs                        NOUVEAU
│   ├── referentiels.rs                          NOUVEAU
│   ├── services.rs                              NOUVEAU
│   ├── points_de_vente.rs                       NOUVEAU
│   ├── configuration.rs                         NOUVEAU
│   ├── branding.rs                              NOUVEAU
│   └── mod.rs                                   ÉTENDU — montage par service(...)
│
└── tests/
    ├── agnosticite_socle.rs                     NOUVEAU — les 3 parcours, écrits EN PREMIER
    ├── capacites_refusees.rs                    NOUVEAU — P-06, 9 cas
    ├── configuration_heritee.rs                 NOUVEAU — matrice exhaustive
    ├── parametres_catalogue.rs                  NOUVEAU — cohérence documentaire
    └── {isolation_tenant,rls_catalogue,classes_offline,outbox_transactionnel}.rs   ÉTENDUS

app/
├── pages/etablissement.vue                      NOUVEAU — coquille, import dynamique
├── modules/etablissements/                      NOUVEAU — G1 et ses 4 sections
├── core/i18n/{fr,en}.json                       ÉTENDUS — parité obligatoire
└── tests/                                       ÉTENDUS — sélection des services visibles

docs/
├── registre-classes-offline.md                  ÉTENDU — 2 entités + lecture en cache + §13
├── design/lexique.md                            ÉTENDU — 7 termes AVANT de coder
├── user-stories-v1.md                           ÉTENDU — récapitulatif des paramètres
└── module-dore.md                               ÉTENDU — « aucun DML de migration sous FORCE »
```

**Structure Decision** — l'arborescence est celle qu'impose la constitution (principe II) et que le
cycle 001 a matérialisée. Ce cycle n'en crée aucune branche nouvelle. Deux points méritent d'être
dits :

- **Un sous-module par story dans le crate `etablissements`**, chacun avec ses trois couches
  (`modele`, `repository`, `service`) — exactement la forme de `note/` produite par le module doré.
  Un fichier unique de 2 000 lignes serait plus court à écrire et impossible à relire.
- **`app/modules/etablissements/`** porte le contenu métier de `G1` ; la page ne fait que l'importer
  dynamiquement. C'est ce qui rend le chargement paresseux par module effectif et vérifiable sur la
  sortie de construction ([research.md R-14](research.md)).

---

## Phases

### Phase 0 — Recherche ✅ terminée

[`research.md`](research.md) — quatorze décisions. Trois portent sur des pièges de PostgreSQL sous
sécurité au niveau ligne forcée qui **réussissent en silence** : R-01 (référentiel global), R-08
(migration additive sur table peuplée) et l'unicité `NULLS NOT DISTINCT` de R-04. Une seule
question reste ouverte et elle est bornée : les versions des deux paquets de test front, que le
principe XI interdit de proposer ici.

### Phase 1 — Conception ✅ terminée

- [`data-model.md`](data-model.md) — 11 tables, leurs contraintes, leurs privilèges, leurs classes
  hors-ligne, les 11 types d'événements. **Deux tables sont absentes du registre** et doivent y
  être ajoutées dans le même changement.
- [`contracts/http-api.md`](contracts/http-api.md) — 21 opérations, le corps d'erreur structuré,
  les neuf refus.
- [`contracts/traits-exposes.md`](contracts/traits-exposes.md) — 6 traits, dont l'inversion de
  dépendance qui tient FR-016 sans violer P-03.
- [`quickstart.md`](quickstart.md) — 8 vérifications, dont les trois qui réussissent en silence si
  on ne les regarde pas.

**Constitution Check après conception** : franchi. Aucune violation. Quatre points de complexité
sont justifiés ci-dessous ; deux portes doivent être étendues, et cette extension fait partie du
périmètre livrable — une porte concernée sans mécanisme est un trou, pas une dette.

### Phase 2 — Tâches ⬜ à produire par `/speckit-tasks`

Trois contraintes d'ordonnancement, non négociables :

1. **`backend/tests/agnosticite_socle.rs` est écrit AVANT toute implémentation.** ETB-02c l'exige
   (« à écrire avant l'implémentation ») et c'est ce qui fait la différence entre un test qui
   contraint la conception et un test qui la constate.
2. **Registre et lexique sont mis à jour dans le même changement que le code qu'ils décrivent**,
   jamais après. `classes_offline.rs` fait échouer le build sur une table non déclarée, et la règle
   du lexique est antérieure au code par construction.
3. **Le contrat et le client sont régénérés à chaque modification de handler**, jamais groupés en
   fin de cycle.

---

## Complexity Tracking

Quatre écarts à la solution la plus simple. Chacun est justifié ; à justification absente, l'option
la plus simple s'impose (constitution, § Conformité).

| Complexité ajoutée | Pourquoi elle est nécessaire | Alternative plus simple, et pourquoi elle est rejetée |
|---|---|---|
| **`parametre_catalogue`** — une table de plus, non nommée par le registre des classes hors-ligne | Sans catalogue, `parametre_configuration` est un `JSONB` sans validation, sans type, sans découvrabilité — et le « récapitulatif des paramètres fait foi » du principe I·c reste décoratif. Le catalogue est ce qui rend ce principe **vérifiable par un test** | *Table clé/valeur sans catalogue* : rien n'empêcherait une clé mal orthographiée, un montant en flottant, ou une surcharge à un niveau qui n'a pas de sens. Le premier défaut de ce genre se découvrirait sur un barème fiscal |
| **Colonnes recopiées** `module_implemente`, `capacite_implementee`, `profil_implemente` — dénormalisation assumée | C'est le support de la clé étrangère composite qui rend le refus **structurel et déclaratif**. Sans elles, aucune contrainte de base ne peut refuser une capacité non implémentée | *Déclencheur `BEFORE INSERT`* : du code caché dans la base, invisible en lecture de schéma, à maintenir en parallèle du référentiel. *Validation applicative seule* : contournée par tout import ou script de reprise |
| **`ObstacleDesactivation` posé à vide** — un trait sans aucune implémentation à ce cycle | FR-016 exige le refus de désactiver un service occupé. L'information vit dans les verticales, que le socle ne peut pas atteindre (P-03). Le point d'accrochage doit exister **avant** le cycle qui en a besoin : une alternative qui existe se prend, une alternative à construire se contourne — c'est l'argument exact qui a fait poser `EstablishmentDirectory` à vide au cycle 001 | *Attendre le cycle SEJ* : au moment où la question se posera, la voie facile sera d'ajouter une dépendance du socle vers `verticales/hebergement` « juste cette fois ». C'est précisément la faute que P-03 attrape, et elle ne se commet jamais franchement |
| **Quatre référentiels globaux** au lieu de contraintes de valeur | Le cadrage §14.3 et §14.4 exigent que l'ajout d'un module ou d'une capacité soit **une écriture de configuration, pas une migration**. Et le message de refus doit distinguer « connu mais non implémenté » de « inconnu » — ce qu'un `CHECK` littéral ne sait pas faire | *`CHECK ... IN (...)`* : ferait de l'ouverture d'une capacité une migration, contre §14.4. *Type énuméré PostgreSQL* : ajouter une valeur reste une migration, et l'ordre des valeurs devient significatif sans qu'on l'ait voulu |

### Deux points appelant une décision avant `/speckit-tasks`

**Les deux paquets de test de l'application ne sont pas au gel.** `docs/versions-gelees.md` §3.2 ne
contient ni utilitaire de montage de composants Vue ni environnement DOM. Le principe XI interdit
d'en proposer une version ici. Deux issues, et c'est une décision de gouvernance, pas de plan :

- les **ajouter** — versions vérifiées sur le registre officiel avec l'URL citée au moment de
  l'ajout, puis portées au gel à la revue du 2026-08-31, comme les six crates du cycle 001 ;
- les **refuser** — SC-005 se vérifie alors sur la fonction de sélection seule, ce qui est une
  couverture moindre et **doit être consigné** plutôt que laissé croire acquis.

**Aucune version n'est proposée dans ce plan, dans aucun des deux cas.**

> **Tranché à T004, dans le sens de l'ajout** (gel **1.0.6**, 2026-07-31). SC-005 porte sur le
> **HTML rendu** : le vérifier sur la seule fonction de sélection testerait l'intention et non le
> résultat, alors que « un service inactif est absent, jamais grisé » est une propriété que le
> composant peut perdre sans que sa fonction de sélection change. Retenus, vérifiés sur
> `https://registry.npmjs.org/` avec `peerDependencies` contrôlées : `@vue/test-utils` **2.4.11**,
> `happy-dom` **20.11.1**, et — **non prévu par ce plan, qui n'annonçait que deux paquets** —
> `@vitejs/plugin-vue` **6.0.8**, sans lequel rien ne compile un `.vue` pour Vitest hors du
> pipeline Nuxt.

### Definition of Done — le point 10 est SANS OBJET, et c'est écrit ici

*Consigné avant la revue de T054, jamais coché en silence.*

Le point 10 (« document imprimé vérifié sur imprimante thermique ») **n'a aucune cible dans ce
cycle**. L'aperçu d'ETB-05 est un **rendu à l'écran** : il n'est pas envoyé à une imprimante, ne
passe par aucune file d'impression et ne dépend d'aucun pilote. La première impression réelle du
produit relève du cycle **IMP**, avec la politique d'impression dont ce cycle ne pose que la clé de
catalogue.

Le point 8 (« écran vérifié en mode clair et en mode sombre »), lui, était sans objet au cycle 001
faute d'écran. Il **devient exigible ici** avec `G1` et il est tenu — c'est la dette que ce cycle
solde.

**Le gel a un mois le 2026-08-31.** Rien dans ce cycle n'appelle de montée de version, et aucune
n'est proposée. Un point mérite néanmoins d'être **signalé sans être changé** : sqlx `0.9.0`
apporte un type d'erreur dédié à la violation de contrainte d'exclusion (`#3918`), l'une des deux
raisons du choix de cette version, et il **n'a toujours aucune cible** — ni au cycle 001, ni ici.
Sa première utilisation réelle sera HEB-02. C'est la revue mensuelle qui tranche, pas ce plan.
