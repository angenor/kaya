# Implementation Plan: Unités louables, formules de location et moteur de disponibilité

**Branch**: `004-unites-formules-disponibilite` | **Date**: 2026-08-02 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/004-unites-formules-disponibilite/spec.md`

**Artefacts de phase** : [research.md](./research.md) · [data-model.md](./data-model.md) ·
[contracts/http-api.md](./contracts/http-api.md) · [contracts/traits-exposes.md](./contracts/traits-exposes.md) ·
[quickstart.md](./quickstart.md)

---

## Summary

Ce cycle livre la première verticale du produit. Huit tables dans un schéma `hebergement` neuf,
treize opérations HTTP, cinq types d'événements outbox, cinq permissions — les **premières
rattachées à un module d'activité** — et **deux écrans** : l'offre d'hébergement, qui est
**maquetté**, et la gestion des chambres, qui est **composée** depuis les seize composants
canoniques (troisième cas de `docs/Kaya_Design.md` §2).

**Le cœur tient en une contrainte de base de données :**

```sql
CONSTRAINT occupation_sans_chevauchement
    EXCLUDE USING gist (unite_id WITH =, periode WITH &&)
```

Tout le reste du cycle sert cette ligne ou en découle. Elle rend la double attribution
**impossible** là où un verrou applicatif la rendrait seulement improbable, et le principe IV la
qualifie de structurante et irréversible.

**Trois dettes annoncées par les cycles précédents se soldent ici :**

1. Le crate `verticales/hebergement`, coquille depuis le cycle 001, reçoit son contenu — et devient
   **la première verticale non vide**, donc la première cible réelle de la porte P-03.
2. La migration `0016` du cycle 003 écrit que `module_code` « restera `NULL` jusqu'au cycle HEB,
   qui apportera `heb.unite.attribuer` ». Cinq permissions honorent cette phrase.
3. La porte **P-09** est verte depuis le cycle 001 **parce qu'aucune table d'occupation n'existe**.
   `backend/tests/portes_a_vide.rs` interroge littéralement `table_existe(&pool, "hebergement",
   "occupation")` et **fera échouer le build** dès que la migration `0025` sera appliquée. La porte
   se lève dans le même changement.

**Et un point ouvert du gel se referme.** `docs/versions-gelees.md` écrit en tête : « Un seul point
reste ouvert : le choix de sqlx `0.9.0` doit être confirmé par le spike GiST/`tstzrange` de la
phase 0 ». Ce cycle **est** cette confirmation — sur `tstzrange`, en concurrence réelle, avec le
type d'erreur exercé.

---

## Technical Context

**Aucune version n'est proposée ici.** `docs/versions-gelees.md` (gel **1.0.12**, vérifié le
2026-08-02) fait foi, et ses valeurs sont reprises telles quelles.

**Language/Version** : Rust **1.97.1** (toolchain gelée) · TypeScript **5.9.3** · edition et
`rust-version` héritées du workspace

**Primary Dependencies** — toutes déjà au dépôt, **aucune nouvelle** :
`actix-web 4.14.0` · `sqlx =0.9.0` (features `uuid`, `time`, `rust_decimal`, `sqlx-toml` déjà
actives) · `utoipa =5.5.0` (`actix_extras`, `uuid`, `time`) · `uuid =1.24.0` (`v7`) ·
`time =0.3.54` · `rust_decimal =1.42.1` · `futures =0.3.33` (tests de concurrence) ·
`thiserror =2.0.19` · `async-trait =0.1.91`

**Storage** : PostgreSQL **18.4**, schéma `hebergement`. Extension `btree_gist` **déjà installée**
par `0001_roles_et_schemas.sql:93` — prérequis de `EXCLUDE USING gist (uuid WITH =, …)`, posée au
premier cycle précisément pour ce moment. **Redis n'est pas touché** : rien de ce cycle n'est
éphémère reconstructible. **Garage n'est pas touché** : aucun objet.

**Testing** : `cargo test --workspace` (tests d'intégration sur base réelle) · `vitest 4.1.10`
(front) · `@playwright/test 1.62.1` — Chromium **1234** et WebKit **rev 2336** pour P-22

**Target Platform** : API en Docker `linux/amd64` sur VPS Contabo (mode A du cadrage §10.1).
**Le poste de développement est arm64** : la construction de production se fait dans Docker pour
`linux/amd64`, jamais par copie d'un binaire local. **Ce cycle n'ajoute aucune dépendance native,
aucun plugin, aucun outil** — la question des deux architectures ne se pose donc pas, et c'est
vérifié plutôt que supposé (§ Portes, P-20).

**Project Type** : monolithe modulaire Rust + application Nuxt 4 (SSR désactivé) + Tauri v2

**Performance Goals** : montant d'un passage rendu en < 300 ms (SC-004) — la cible de 30 s de
SEJ-02 doit rester tenable une fois la saisie ajoutée par-dessus. Mille attributions concurrentes
sur la même unité : exactement une réussit (SC-001).

**Constraints** : hors-ligne **interdit** sur tout ce cycle — référentiel en classe **C**,
occupation en classe **B** (P-13). Montants en entiers d'unité mineure, quantités en `NUMERIC`
(P-10). Toute durée depuis l'horodatage d'autorité serveur, jamais l'horloge d'un terminal
(principe IV).

**Scale/Scope** : 8 tables · 6 migrations · **13 opérations HTTP** · 5 événements outbox ·
5 permissions · 3 traits exposés · **2 écrans** (`G2` maquetté à 2 états, `G5` composé) ·
18 unités aux seeds Deloria

---

## Constitution Check

*GATE — évalué avant Phase 0, réévalué après Phase 1.*

### Principes engagés

| Principe | Ce que ce cycle en fait | Verdict |
|---|---|---|
| **I·a** Contrat généré | 13 opérations utoipa ; client TS régénéré et commité | ✅ |
| **I·b** Schéma par migrations | 6 migrations versionnées ; seeds **à part**, rejouables | ✅ |
| **I·c** Paramètres métier | 3 clés au catalogue de configuration héritée ; récapitulatif de `user-stories-v1.md` mis à jour **dans le même changement** | ✅ |
| **II** Hiérarchie des crates | `verticales/hebergement` dépend de `socle/` et `capacites/`, jamais l'inverse. **Première verticale non vide** | ✅ |
| **II** Un schéma par module | Schéma `hebergement` ; aucune clé étrangère n'en sort | ✅ |
| **II** Outbox transactionnel | 5 événements, signature qui **prend** la transaction | ✅ |
| **III** RLS | `ENABLE` + `FORCE` + `USING`/`WITH CHECK` sur les 8 tables | ✅ |
| **IV** Temps et disponibilité | **Le cœur du cycle** — `tstzrange`, `EXCLUDE USING gist`, remise en état dans l'intervalle, statut dérivé, horodatage d'autorité | ✅ |
| **V** Argent et fiscalité | `prix_mineur` en `BIGINT`, `quantite` en `NUMERIC`. **Aucune règle fiscale ici** — le crate porte le paramètre, `JurisdictionAdapter` l'interprétera | ✅ |
| **VI** Hors-ligne | Tout le cycle est indisponible hors ligne (B et C), annoncé immédiatement | ✅ |
| **VII** Application unique | Un module front chargé paresseusement ; `PlatformAdapter` seul pont natif | ✅ |
| **VIII** i18n et mode sombre | Clés `fr` et `en` ; écran vérifié en clair et en sombre | ✅ |
| **IX** Registres immuables | Le registre des actions reçoit les rebascules ; **rien n'est purgé** | ✅ |
| **X** Prêt ≠ construit | Aucune table pour HEB-07 ni HEB-08 ; `prestation_incluse` **vide** ; pas de `ressource_reservable` | ✅ |
| **XI** Versions | **Aucune dépendance nouvelle** ; gel 1.0.12 repris tel quel | ✅ |
| **XII** Référence visuelle | `G2` est **maquetté** — la maquette fait foi, elle n'est pas copiée. `G5` est **composé** (3ᵉ cas de `Kaya_Design.md` §2), couverture par les seize composants vérifiée, inscrit à `derivation.md` | ✅ |

### Les trois points qui méritaient un examen, et non une case cochée

**1. `ressource_reservable` n'existe pas — le socle est-il en défaut ?**

Le principe II écrit que le socle « connaît `article_vendable` et `ressource_reservable` ».
Vérification faite : **ces deux noms n'apparaissent dans aucune migration ni aucun crate** — seul
`docs/cadrage-v1.md:122` les mentionne. Trois cycles ont livré le socle sans elles, et
`etablissements.table_pdv` — conceptuellement une ressource réservable — est passée sans cette
abstraction au cycle 002.

**Lecture retenue** : l'énoncé est une **frontière de vocabulaire**, pas un inventaire de tables.
Il dit ce que le socle ne doit pas connaître (chambre, unité louable, séjour) et par quels mots il
nommerait la chose s'il avait à la nommer. Créer une table parente maintenant serait une
abstraction spéculative à **un seul implémenteur**, que le principe X interdit.

**Ce qui reste vérifiable, et qui est vérifié** : aucun crate de `socle/` ne gagne la moindre
notion d'unité, de chambre ou de formule — porte P-03, test `backend/tests/architecture.rs`.
Décision consignée en [research.md R-09](./research.md), à rouvrir quand RSV apportera un second
implémenteur. **Non-conformité : aucune.** Point de revue : oui.

**2. Le crate porte des paramètres fiscaux — le principe V est-il entamé ?**

`formule.assujettie_taxe_nuitee` et `formule.regle_conversion_taxe` sont des données fiscales dans
une verticale. La porte **P-12** refuse toute règle fiscale hors `JurisdictionAdapter`.

**La frontière tient parce que ce cycle ne calcule aucune taxe.** Il stocke un paramètre et
l'expose par le trait `ParametrageFiscalHebergement`, qui rend un `Option<RegleConversionTaxe>` —
jamais un montant. C'est exactement la conception que le cadrage §5.5 impose : « chaque formule
porte un drapeau et une règle de conversion ; aucune valeur n'est codée en dur ». La règle qui
consommera ce paramètre vivra dans `socle/fiscalite` en T3.

C'est la confusion la plus tentante du cycle — le crate qui détient le paramètre semble être celui
qui doit l'appliquer — et elle est écrite en toutes lettres dans
[contracts/traits-exposes.md §2.3](./contracts/traits-exposes.md).

**3. Un endpoint d'attribution sans parcours utilisateur — est-ce du prêt-non-construit ?**

Le principe X interdit de bâtir ce qu'aucune story n'appelle. L'attribution n'a pas d'écran à ce
cycle : le check-in est SEJ-02.

**Elle n'est pas spéculative pour deux raisons opposables** : le test obligatoire de classe B
(« deux exécutions simultanées, une seule réussit », `docs/registre-classes-offline.md` §11) exige
un chemin exécutable — sans endpoint, la garantie centrale du cycle serait invérifiable ; et la
permission `heb.unite.attribuer`, annoncée nommément par le cycle 003, doit garder une action
réellement servie sous peine de faire échouer le build.

**Verdict global : aucune violation à justifier.** La section « Complexity Tracking » reste vide.

---

## Project Structure

### Documentation (this feature)

```text
specs/004-unites-formules-disponibilite/
├── plan.md                      # Ce fichier
├── spec.md
├── research.md                  # Phase 0 — 19 décisions
├── data-model.md                # Phase 1 — 8 tables, 6 migrations
├── quickstart.md                # Phase 1 — validation exécutable
├── contracts/
│   ├── http-api.md              # 13 opérations
│   └── traits-exposes.md        # 3 traits exposés, 6 consommés
├── checklists/
│   └── requirements.md
└── tasks.md                     # Phase 2 — produit par /speckit-tasks
```

### Source Code (repository root)

```text
backend/
├── migrations/
│   ├── 0021_schema_hebergement.sql              # schéma + GRANT USAGE
│   ├── 0024_referentiel_hebergement.sql         # 6 tables classe C + RLS
│   ├── 0025_occupation.sql                      # ★ EXCLUDE USING gist + RLS
│   ├── 0026_provision_prestation_incluse.sql    # HEB-09 — table vide
│   ├── 0022_permissions_hebergement.sql         # 5 permissions, module_code non nul
│   ├── 0023_parametres_hebergement.sql          # 3 clés au catalogue
│   └── seeds/                                   # 18 unités Deloria + Résidence Test
│
├── crates/verticales/hebergement/src/
│   ├── lib.rs                                   # remplace la coquille du cycle 001
│   ├── traits.rs                                # MoteurDisponibilite, MoteurTarification,
│   │                                            #   ParametrageFiscalHebergement
│   ├── erreurs.rs                               # est_violation_exclusion() — écrite UNE fois
│   ├── referentiel/{modele,repository,service,mod}.rs
│   ├── occupation/{modele,repository,service,mod}.rs      # ★ le cœur
│   └── tarification/{modele,bareme,service,mod}.rs        # fonction pure + service
│
├── api/src/routes/
│   ├── hebergement_referentiel.rs               # opérations 1 à 8
│   ├── hebergement_disponibilite.rs             # opérations 9 à 11
│   └── hebergement_tarification.rs              # opération 12
│
└── tests/
    ├── hebergement_disponibilite.rs             # ★ P-09 levée + classe B
    ├── hebergement_referentiel.rs               # classe C + validations de service
    ├── hebergement_tarification.rs              # cas figés du barème
    ├── hebergement_hors_ligne.rs                # P-13 — B et C injoignables hors ligne
    ├── portes_a_vide.rs                         # MODIFIÉ — P-09 levée, relais posé
    ├── classes_offline.rs                       # MODIFIÉ — schéma `hebergement` ajouté
    ├── couverture_portes.rs                     # MODIFIÉ — 4 décomptes
    ├── agnosticite_socle.rs                     # MODIFIÉ — le test prend son sens ici
    └── isolation_tenant.rs, rls_catalogue.rs,
        outbox_transactionnel.rs                 # MODIFIÉS — nouvelles cibles

app/
├── pages/hebergement.vue                        # route /hebergement — G2, MAQUETTÉ
├── pages/chambres.vue                           # route /chambres    — G5, COMPOSÉ
├── modules/hebergement/
│   ├── EcranOffre.vue                           # d'après G2-offre-hebergement.html
│   ├── CarteFormule.vue
│   ├── EcranChambres.vue                        # G5 — composants 08, 16, 01–03, 11, 13
│   ├── ListeUnites.vue
│   ├── FormulaireCategorie.vue
│   ├── FormulaireUnite.vue                      # code et étage SEULEMENT
│   ├── donnees.ts                               # appels typés
│   └── modifier-formule.ts                      # écriture — septième couche
└── core/i18n/{fr,en}.json                       # MODIFIÉS — clés du module

docs/
├── registre-classes-offline.md                  # MODIFIÉ — §7.1 + journal §13 v1.2.0
├── user-stories-v1.md                           # MODIFIÉ — récapitulatif des paramètres
└── module-dore.md                               # MODIFIÉ — retour du spike (§ R-03, R-19)
```

**Structure Decision** — la structure existante est reprise sans exception : un crate par domaine
avec `{modele, repository, service}` par agrégat, `traits.rs` à la racine du crate, handlers dans
`api/src/routes/`, tests d'intégration dans `backend/tests/`, module front dans `app/modules/`.
C'est le patron des cycles 002 et 003 ; s'en écarter coûterait la lisibilité sans rien apporter.

---

## Portes de CI — comment chacune est vérifiée, et par quel test

*Exigence de la section « Couverture des portes » de la constitution : chaque porte déclare son
périmètre inspecté, vérifie sa complétude, ne modifie pas ce qu'elle inspecte et prouve que sa
cible n'est pas vide.*

### Les portes que ce cycle touche

| Porte | Ce qu'elle vérifie ici | Mécanisme | Cible non vide prouvée par |
|---|---|---|---|
| **P-01** | Client TS identique au commité après 13 opérations nouvelles | `scripts/ci/generer-client.sh`, diff commité | Le client contient les 12 `operationId` |
| **P-01b** | 56 `operationId` distincts | `couverture_portes.rs` — décompte relu **du contrat**, jamais d'une constante | 43 → **56**, écart asserté |
| **P-02** | Aucune des 21 migrations antérieures modifiée | `scripts/ci/migrations-figees.sh` | 6 migrations nouvelles, 20 figées |
| **P-03** | **Première verticale non vide** — aucun crate `socle/` ne dépend de `verticales/` | `backend/tests/architecture.rs`, graphe de dépendances | ⚠️ **Voir ci-dessous** |
| **P-04** | Aucune jointure `hebergement` × autre schéma | `scripts/ci/jointures-inter-schemas.sh` — **liste des schémas à étendre à `hebergement`** | Décompte des requêtes analysées par schéma |
| **P-05** | 5 événements émis **dans** leur transaction | `outbox_transactionnel.rs` + `couverture_portes.rs` | 22 → **27** types, chacun avec son test |
| **P-05b** | Aucune purge du registre des actions ni de l'outbox | `scripts/ci/outbox-sans-purge.sh` — périmètre inchangé | Registres existants toujours inspectés |
| **P-07** | RLS `ENABLE` + `FORCE` + politique sur les **8** tables nouvelles | `rls_catalogue.rs` + `couverture_portes.rs` | 26 → **34** tables |
| **P-08** | Le tenant A ne lit ni n'écrit rien du tenant B sur les 13 endpoints | `isolation_tenant.rs` + `couverture_portes.rs` | 43 → **56** opérations, régime déclaré pour chacune |
| **P-09** | ★ **LEVÉE** — `occupation` est un `tstzrange` protégé par `EXCLUDE USING gist` | `hebergement_disponibilite.rs` (nouveau) | ⚠️ **Voir ci-dessous** |
| **P-10** | `prix_mineur` en `BIGINT`, `quantite` en `NUMERIC`, clés JSONB `<nom>_mineur` + `devise` | `scripts/ci/types-monetaires.sh` | La porte cesse d'être « presque à vide » : 4 colonnes monétaires et 5 charges utiles |
| **P-12** | Aucune règle fiscale hors `JurisdictionAdapter` | Contrôle existant + revue du crate | Le crate porte des **paramètres**, aucun calcul |
| **P-13** | Aucune opération B ou C atteignable hors ligne — **tout ce cycle** | `hebergement_hors_ligne.rs` (nouveau) | 13 endpoints, tous inspectés |
| **P-15** | Aucun `window.__TAURI__` hors `PlatformAdapter` dans le module front | `pnpm porte:p15` (`pont-natif-confine.sh`), qui ajoute le **décompte des fichiers réellement analysés par arbre** — une cible vide passerait autrement | Le module `hebergement` entre dans le décompte de l'arbre `app/` |
| **P-16** | Aucune chaîne en dur ; parité `fr`/`en` | `pnpm test:i18n` | Clés du module comptées des deux côtés |
| **P-17** | Aucune couleur ni espacement littéral | `pnpm lint:tokens` | Les `.vue` du module sont analysés |
| **P-18** | `cargo sqlx prepare` vert | `scripts/ci/preparer-sqlx.sh` — ⚠️ **double passe obligatoire** | Décompte des requêtes en cache |
| **P-19** | Aucun fichier de `docs/design/html/` copié sous `app/` ; **tout écran a une référence** — maquette, ligne de dérivation, ou inscription « composé » | `scripts/ci/maquettes-non-copiees.sh` | `G2-offre-hebergement.html` est lu, jamais copié ; `G5` est inscrit à `derivation.md` avant d'être codé |
| **P-20** | Aucune dépendance en intervalle ; lockfiles à jour | `scripts/ci/versions-epinglees.sh` | **Aucune dépendance nouvelle** — la porte reste sur son périmètre |
| **P-21 / P-21b** | Aucune ressource d'hôte externe ; tout déclaré est embarqué | `scripts/ci/aucune-ressource-externe.sh`, `ressources-embarquees.sh` | L'écran n'ajoute ni police, ni icône, ni image |
| **P-22** | Les routes `/hebergement` **et `/chambres`** s'ouvrent **en direct ET par navigation**, sur Chromium **et** WebKit, dans les **deux thèmes** | `scripts/ci/parcours-reel.sh` | Les **deux** routes nouvelles sont comptées par projet — un moteur sans cas fait échouer |

### ⚠️ P-09 — la porte du cycle, et la seule dont le mécanisme de levée est déjà écrit

`backend/tests/portes_a_vide.rs` porte, depuis le cycle 001, une assertion de non-régression :

```rust
let occupation = table_existe(&pool, "hebergement", "occupation").await;
assert!(!occupation, "P-09 : la table `occupation` existe désormais, mais la porte est
        toujours installée à vide. HEB-02 doit, dans le MÊME changement, vérifier ici que : …");
```

**Ce test échouera dès que la migration `0023` sera appliquée.** C'est voulu, et c'est le
comportement que la constitution attend d'une porte à vide. La levée suit exactement le précédent
de P-06 au cycle 002 :

1. le contenu réel de la porte part dans `backend/tests/hebergement_disponibilite.rs` ;
2. `portes_a_vide.rs` garde un **relais** — un test qui échoue si ce fichier disparaît, sans quoi
   supprimer la porte réelle ne casserait plus rien ;
3. le récapitulatif exécutable et son décompte passent de deux portes à vide à **une** (P-11 seule).

**Les trois assertions que la porte levée doit porter** (dictées par le message du cycle 001) :

| # | Assertion | Comment |
|---|---|---|
| 1 | La période est un `tstzrange`, jamais une paire de dates | Lecture d'`information_schema` : le type de `periode` est `tstzrange`, **et** aucune colonne `debut`/`fin` de type `date` n'existe sur la table |
| 2 | Une contrainte `EXCLUDE USING gist (unite_id WITH =, periode WITH &&)` la protège | Lecture de `pg_constraint` : `contype = 'x'`, et la définition contient les deux opérateurs |
| 3 | Deux attributions concurrentes chevauchantes échouent — pas « improbablement », jamais | **Deux transactions distinctes**, insertion sans commit, puis commit des deux. Exactement une réussit, et l'échec est un `ErrorKind::ExclusionViolation` sur la contrainte nommée |

**L'assertion 3 est celle qui distingue une garantie d'une coïncidence.** Un test qui se
contenterait de « une seule a réussi » passerait au vert sur un `SELECT … FOR UPDATE`, sur
`SERIALIZABLE` ou sur un verrou applicatif — trois mécanismes qui rendraient la double attribution
improbable au lieu d'impossible. La **cause** du refus est assertée, pas seulement son existence.

**Test négatif de la porte** : retirer la contrainte d'exclusion sur une base de test, constater
l'échec des trois assertions, remettre. Sur le modèle de `pnpm porte:p22:negatif`.

### ⚠️ P-03 — la porte n'avait jamais eu de cible réelle

`verticales/hebergement` était une coquille vide : le graphe de dépendances ne pouvait porter
aucune arête interdite, faute de code à en tirer. Son en-tête le dit — « le créer vide maintenant
n'est pas décoratif : c'est ce qui rend la porte P-03 capable de constater dès aujourd'hui
qu'aucune arête interdite n'existe ».

**Ce cycle est le premier où P-03 pourrait échouer.** Le test doit donc, en plus de sa vérification
d'arêtes, **prouver que sa cible n'est pas vide** : au moins un crate de `verticales/` porte des
symboles publics, sans quoi la porte redeviendrait indistinguable d'une porte à cible vide.

Le piège concret à surveiller pendant l'implémentation : faire remonter un type de `hebergement`
dans une signature de `socle/` — par exemple si `OutboxWriter` devait connaître le type d'un
événement `heb.*`. Il ne le doit pas : les charges utiles d'événements sont du JSON opaque pour le
socle.

### ⚠️ P-18 — la double passe, qui coûte une journée si elle est manquée

`cargo sqlx prepare` **détruit le cache silencieusement s'il ne recompile rien**, et ne collecte
que les requêtes des cibles que son `cargo check` compile réellement. Le répertoire d'où on le
lance décide de ce qu'il voit :

| Lancé depuis | Collecte | Perd |
|---|---|---|
| `backend/` | paquet racine et tests d'intégration | les **binaires** de `kaya-api` — `seeds`, `contrat` |
| `backend/api/` | binaires et bibliothèque de `kaya-api` | les tests de `backend/tests/` |

**Ce cycle ajoute des requêtes des deux côtés** — les seeds Deloria (binaire `seeds`) et les tests
d'intégration. La procédure à deux passes de `CLAUDE.md` est donc obligatoire, suivie des **deux**
contrôles dans l'ordre :

```sh
git status --short backend/.sqlx    # AUCUNE suppression ; que des ajouts
SQLX_OFFLINE=true DATABASE_URL= cargo check --workspace --all-targets --locked
```

### Les portes que ce cycle ne touche pas, et pourquoi c'est dit

| Porte | Motif |
|---|---|
| **P-06** | Aucune capacité déclarée ; `STOCK`/`SIMPLE` reste le seul couple accepté |
| **P-11** | **Aucun calcul fiscal.** Le cycle porte le paramètre `regle_conversion_taxe`, il ne l'interprète pas. La porte reste installée à vide, avec son assertion de non-régression — qui **doit rester verte** : si elle se réveillait, c'est qu'un jeu de cas fiscal serait apparu ici, donc qu'une règle fiscale aurait été écrite hors `JurisdictionAdapter` |
| **P-14** | Rejeu triple d'une écriture de classe A. `unite.statut_menage` est classé A, mais **aucun endpoint ne l'écrit** à ce cycle (HEB-06). La porte n'a donc pas de cible nouvelle, et il serait faux de lui en déclarer une |

---

## Tests d'intégration — dont les tests hors-ligne obligatoires du §0.7

### Par classe, tels que `docs/registre-classes-offline.md` §11 les impose

| Classe | Tests exigés | Fichier |
|---|---|---|
| **B** — `occupation` | Test qui **échoue si l'opération est atteignable depuis un chemin exécutable hors ligne** · **Test de concurrence : deux exécutions simultanées, une seule réussit** | `hebergement_hors_ligne.rs` · `hebergement_disponibilite.rs` |
| **C** — les 5 tables du référentiel | Test qui **échoue si l'opération est atteignable hors ligne** · Test d'**isolation multi-tenant sur l'endpoint** | `hebergement_hors_ligne.rs` · `isolation_tenant.rs` |
| **Entité rattachée à un séjour** | Scénario orphelin (SYN-03) | **Sans objet à ce cycle** — aucune entité n'est rattachée à un séjour, le séjour n'existe pas. Deviendra dû avec SEJ-02 |

### Les deux tests transverses permanents

**1. Réseau coupé puis rétabli** au milieu d'une journée d'exploitation simulée — la clôture tombe
au franc près (SYN-04). Ce cycle ne le fait pas régresser : rien de ce qu'il livre n'est
atteignable hors ligne, donc rien ne peut diverger.

**2. Agnosticité du socle (ETB-02c)** — un établissement portant un module fictif minimal, **sans
aucune capacité**, va de la création à la clôture journalière.

> **Ce test existe depuis le cycle 002 et prend son sens exact maintenant.** Jusqu'ici, il
> prouvait que le socle n'exigeait rien d'une verticale — mais aucune verticale n'existait pour le
> contredire. Ce cycle en crée une : **c'est la première fois que le test peut échouer.** S'il
> passe encore avec `hebergement` livré, alors « aucun crate partagé ne suppose l'existence d'un
> hébergement » cesse d'être une intention pour devenir un fait mesuré.
>
> Un second tenant est déjà prévu à cet effet — « Résidence Test », module hébergement seul —, et
> FR-048 le peuple à quatre unités pour éprouver qu'aucune formule n'est réservée à un type
> d'établissement.

### Tests propres au cycle

| Fichier | Ce qu'il couvre |
|---|---|
| `hebergement_disponibilite.rs` | ★ P-09 (3 assertions) · classe B · intervalle vide refusé · borne de fin exclue · remise en état contiguë · intervalle traversant minuit · libération |
| `hebergement_referentiel.rs` | Classe C · famille inconnue refusée explicitement · deux formules de même famille refusées · barème sans palier refusé · demi-journée sans plage refusée · suppression bloquée avec motif |
| `hebergement_tarification.rs` | Cas figés du barème : 2 h → 2 800 · 4 h 10 → 6 200 · 20 min → 1 500 · 8 h → bascule en nuitée · durée depuis l'horodatage d'autorité malgré une horloge décalée |
| `hebergement_hors_ligne.rs` | P-13 sur les 13 endpoints |
| `provisions_sans_logique.rs` *(modifié)* | `prestation_incluse` : table présente, **aucun privilège** `kaya_app`, aucun endpoint |
| `seeds_rejouables.rs` *(modifié)* | Rechargement en une commande, idempotent, 18 unités Deloria + 4 « Résidence Test » |

---

## Séquencement — ce qui ne peut pas être parallélisé

1. **`0021` → `0022` → `0023` → `0024` → `0025`** — appliqué dans cet ordre. **Les numéros suivent
   l'ordre des tâches, pas l'ordre thématique** : `sqlx` refuse une version antérieure à une version
   déjà appliquée (constaté au cycle 001, consigné dans l'en-tête de `0006_provisions_comptables.sql`).
   La contrainte d'exclusion se pose **à la création** de la table : ajoutée sur une table peuplée, elle échoue sur les données existantes,
   et il faudrait alors choisir entre corriger l'historique et renoncer à la garantie.
2. **La levée de P-09 accompagne `0025` dans le même changement.** Le build est rouge entre les
   deux — c'est l'assertion de non-régression qui fait son travail, pas une régression.
3. **Les seeds viennent après le référentiel complet**, et jamais par migration : une table en
   `FORCE ROW LEVEL SECURITY` accepte un `INSERT` de migration **en n'écrivant rien**, sans erreur.
4. **`couverture_portes.rs` se met à jour en dernier.** Il suppose que toutes les migrations, tous
   les points d'entrée et tous les événements existent ; le lancer plus tôt compterait juste et
   couvrirait faux — son propre en-tête le dit.
5. **Les écrans viennent après le contrat**, et le client TypeScript est régénéré entre les deux.
6. **`G5` ne se code qu'après son inscription à `derivation.md`** — sans cette ligne, P-19 le
   refuse. Même mécanique que `R0` au cycle 003.

---

## Complexity Tracking

*Aucune violation de la constitution à justifier.* La section est laissée vide intentionnellement —
les trois points examinés en § Constitution Check se résolvent par lecture des textes, pas par
dérogation.
