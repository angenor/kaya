# Implementation Plan: Classification hors-ligne, file d'actions et horodatage d'autorité

**Branch**: `005-file-hors-ligne-horodatage` | **Date**: 2026-08-02 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/005-file-hors-ligne-horodatage/spec.md`

---

## Summary

SYN-01, SYN-02 et SYN-04 — trois stories P0 de la tranche T1. Le cycle rend **opposable** ce que
quatre cycles ont posé par fragments : la file locale reçoit sa persistance chiffrée, son envoi
opportuniste et son témoin permanent ; le registre des classes hors-ligne cesse d'être vérifié par
une liste écrite à la main ; l'horodatage client se voit interdire toute règle métier, par une
porte plutôt que par une convention. Les tests obligatoires du §0.7 deviennent un **outillage
réutilisable** que chaque cycle suivant instancie en une déclaration.

**Approche technique en une phrase** : presque rien en base — une provision et deux paramètres —
et l'essentiel dans le cycle de vie de l'application, dans l'outillage de test, et dans deux
mécanismes de découverte qui remplacent des listes manuelles.

---

## Technical Context

**Language/Version** : Rust 1.97.1 · TypeScript 5.9.3 · Node 24.18.1 — `docs/versions-gelees.md`
fait foi, valeurs reprises telles quelles.

**Primary Dependencies** : Actix Web 4.14.0, sqlx 0.9.0, utoipa 5.5.0, `uuid` 1.24.0 (feature
`v7`), Nuxt 4.5.1, Tailwind 4.3.3, Tauri 2.11.5, `@playwright/test` 1.62.1, Vitest 4.1.10.

**⚠️ Aucune dépendance nouvelle.** Le chiffrement de la file emploie **WebCrypto**, API du moteur
présente sur les quatre cibles — pas une bibliothèque. `docs/versions-gelees.md` est **inchangé**
par ce cycle, et rien n'y est proposé.

**Storage** : PostgreSQL 18.4 (un schéma par module, RLS `ENABLE` **et** `FORCE`) · Redis 8.8.1
**pour l'éphémère reconstructible seulement** — ici, le débrayage du signalement de dérive.
Garage n'est pas touché.

**Testing** : `cargo test --workspace` (intégration, base réelle) · Vitest (application) ·
Playwright sur **Chromium et WebKit** (porte P-22).

**Target Platform** : API — Docker `linux/amd64` sur VPS Contabo. Application — desktop, Android,
iOS par Tauri, plus le navigateur en développement.

**⚠️ Le poste est `arm64`, la cible est `amd64`.** Ce cycle n'ajoute **aucune dépendance native**,
aucun greffon, aucun outil de construction : la contrainte des deux architectures est satisfaite
par construction, et c'est un effet du choix de WebCrypto (R-06).

**Project Type** : monolithe modulaire Rust + application unique Nuxt/Tauri.

**Performance Goals** : le témoin de synchronisation lisible **en moins de deux secondes sans
cliquer** (SC-005) ; aucune minuterie de scrutation réseau — la batterie d'un Android d'entrée de
gamme doit tenir un service entier (R-09).

**Constraints** : file survivant au rechargement **et** à l'extinction (FR-012) ; chiffrée dès le
premier octet (FR-013) ; vidée au retour au premier plan sur **toutes** les plateformes, iOS
n'ayant pas de synchronisation en arrière-plan ; aucune donnée B, C ou D en cache d'écriture.

**Scale/Scope** : 2 migrations · 0 endpoint nouveau · 1 table (provision) · 2 paramètres ·
1 famille d'audit · 2 écrans + 1 composant du système de design · 1 module d'énumération partagé ·
1 outillage de test à deux versants · **10 fichiers de portes à porter** sur le périmètre découvert.

---

## Constitution Check

*Passage avant Phase 0, repassage après Phase 1.*

| Principe | Exigence | Statut | Comment il est tenu |
|---|---|---|---|
| **I·a** Contrat généré | utoipa → OpenAPI → client TS | ✅ | **Aucun endpoint nouveau** ; P-01 exécutée et diff vide attendu |
| **I·b** Schéma par migrations | Migration versionnée, jamais modifiée | ✅ | `0027`, `0028` — additives |
| **I·c** Paramètres en configuration | Aucune valeur métier en dur | ✅ | Les 5 min du cadrage deviennent le **défaut** de `sync.derive_horloge_seuil_secondes` ; récapitulatif §708 mis à jour dans le même changement |
| **II** Hiérarchie des crates | `socle/` ne dépend jamais de `verticales/` | ✅ | La dérive vit dans `socle/synchronisation`, **le plus bas** ; l'audit est câblé par la couche API (R-05) |
| **II** Un schéma par module | Aucune jointure inter-schémas | ✅ | `reconciliation_orpheline` n'a **aucune clé étrangère** vers `sejour` ni `document_fiscal` |
| **II** Outbox transactionnel | Toute transition émet son événement | ✅ | **Aucune transition d'état nouvelle** — et le rejeu n'en émet aucun, ce qui est testé |
| **III** RLS forcée | `ENABLE` + `FORCE`, test d'isolation | ✅ | Patron identique sur la table nouvelle |
| **IV** Intervalles et horodatage | Tout calcul sur l'horodatage d'autorité | ✅ | **Tenu, et gardé par P-23** — ratifiée le 2026-08-02, constitution 1.8.0 |
| **V** Montants et quantités | Entiers d'unité mineure, `NUMERIC` | ✅ | Aucune valeur monétaire créée ; le contexte d'audit de la dérive **ne porte aucune clé monétaire** |
| **VI** Hors-ligne | Classes déclarées, invariante vérifiée | ✅ | C'est l'objet du cycle |
| **VII** Application unique | `PlatformAdapter` obligatoire | ✅ | `surRetourPremierPlan` ajoutée à l'adaptateur, pas à un composant |
| **VIII** Qualité, i18n | fr/en, aucune chaîne en dur | ✅ | Clés `reseau.*` présentes mais **divergentes du lexique** — corrigées par T031 ; ajouts pour `S1`, la quarantaine et l'horloge |
| **IX** Sécurité | Aucun secret dans le binaire | ✅ | La clé de chiffrement est **engendrée sur l'appareil** et vit au coffre système |
| **X** « prêt ≠ construit » | Provisions = données seulement | ✅ | `reconciliation_orpheline` : `GRANT SELECT` **seul**, décompte de provisions 5 → 6 |
| **XI** Versions épinglées | Aucun intervalle, lockfiles commités | ✅ | **Aucune dépendance ajoutée** |
| **XII** Référence visuelle | Tokens, jamais de littéral | ✅ | Témoin = composant 10 ; `S1` et l'écran de note inscrits à la matrice de dérivation |

### La porte P-23 est acquise — constitution 1.8.0

**FR-034 exigeait qu'un calcul métier appuyé sur l'horodatage client soit « détecté et refusé par
une porte automatisée, non par la revue ».** Aucune des vingt-cinq portes ne le gardait : P-09
vérifie que les occupations sont des intervalles protégés par une contrainte d'exclusion, pas la
**provenance** d'un instant.

La porte a été ratifiée le 2026-08-02. Le jeu compte désormais **26 portes** :

> **P-23 — PROVENANCE DE L'INSTANT.** Aucun calcul métier, fiscal, de clôture ou de durée ne
> s'appuie sur `horodatage_client`. Seul l'horodatage d'autorité serveur fait foi. **Exemptions
> limitativement énumérées dans le script** : ordre d'affichage local, détection de dérive
> d'horloge, rendu de l'instant tel que le terminal l'a perçu. Périmètre **découvert**, jamais
> énuméré à la main. Principe IV.

**Ce que « limitativement » impose au script, et qui n'est pas anodin** : la liste est close. La
couche de persistance qui **écrit** la colonne n'y figure pas — et n'a pas à y figurer : écrire une
valeur n'est pas s'appuyer dessus. Lui inventer une exemption élargirait la porte de sa propre
autorité, ce qui est exactement ce que le mot interdit.

### La référence de l'écran `S1` est tranchée — `derivation.md` 1.2.1

La matrice faisait dériver `S1` du « composant 8 », qui est la **ligne de liste**. La version 1.2.1
l'a corrigé : `S1` dérive du **composant 10**, le témoin de synchronisation, avec la précision de ce
que « développement du composant » veut dire — *le témoin dit l'état d'un coup d'œil, le panneau
détaille ce qui attend et permet d'agir*.

---

## Project Structure

### Documentation (this feature)

```text
specs/005-file-hors-ligne-horodatage/
├── plan.md              # ce fichier
├── research.md          # Phase 0 — 14 décisions
├── data-model.md        # Phase 1
├── quickstart.md        # Phase 1
├── contracts/
│   ├── README.md
│   ├── api-http.md
│   ├── platform-adapter.md
│   └── sync-interne.md
├── checklists/
│   └── requirements.md
└── tasks.md             # Phase 2 — /speckit-tasks, PAS créé ici
```

### Source Code (repository root)

```text
backend/
├── migrations/
│   ├── 0027_reconciliation_orpheline.sql        # provision SYN-03 — table seule
│   └── 0028_parametres_synchronisation.sql      # 2 clés au catalogue
├── crates/socle/synchronisation/src/
│   ├── derive.rs                                # NOUVEAU — constater_derive(), trait SignalDerive
│   └── lib.rs                                   # exporte derive
├── api/src/
│   ├── routes/notes.rs                          # inchangé — câble le constat de dérive
│   └── application.rs                           # câble SignalDerive sur JournalAudit
└── tests/
    ├── commun/
    │   ├── perimetre.rs                         # NOUVEAU — schémas et crates DÉCOUVERTS
    │   └── classes.rs                           # NOUVEAU — macros §0.7
    ├── classes_offline.rs                       # listes en dur → perimetre::
    ├── architecture.rs · couverture_portes.rs · … # 10 fichiers portés (21 chemins en dur → 0)
    ├── derive_horloge.rs                        # NOUVEAU — SYN-04 serveur
    ├── horodatage_autorite.rs                   # NOUVEAU — porte P-23
    ├── outillage_classes.rs                     # NOUVEAU — les macros ont-elles une cible ?
    └── provisions_sans_logique.rs               # 5 → 6 provisions

app/
├── core/sync/
│   ├── persistance.ts                           # NOUVEAU — WebCrypto + stockage
│   ├── etat.ts                                  # NOUVEAU — état réactif à 3 valeurs
│   ├── quarantaine.ts                           # NOUVEAU
│   ├── envoi.ts                                 # NOUVEAU — déclencheurs, intervalle croissant
│   ├── index.ts · classes.ts · attente.ts · vidage.ts   # étendus
├── core/platform/
│   ├── index.ts                                 # + surRetourPremierPlan
│   ├── web.ts · desktop.ts · android.ts · ios.ts
│   └── reseau.ts                                # 'degrade' enfin alimenté
├── core/design-system/
│   └── TemoinSynchronisation.vue                # NOUVEAU — composant 10
├── layouts/default.vue                          # accueille le témoin
├── plugins/02.sync.client.ts                    # NOUVEAU — brancherFile au démarrage
├── pages/
│   ├── notes.vue                                # NOUVEAU — écran COMPOSÉ, 1er passager
│   └── mes-envois.vue                           # NOUVEAU — écran S1 ; « synchronisation » est proscrit du visible, URL comprise
└── tests/
    ├── commun/classes.ts                        # NOUVEAU — utilitaires §0.7 côté app
    ├── amorcage.spec.ts                         # brancherFile : « dû » → « branché »
    ├── deconnexion.spec.ts                      # « pas de file » → « file vide »
    └── file-*.spec.ts · temoin-sync.spec.ts     # étendus / nouveaux

tests-e2e/
├── routes.ts                                    # inchangé — lit déjà app/pages/
└── hors-ligne.spec.ts                           # NOUVEAU — balayage réseau coupé (FR-005b)

docs/
├── registre-classes-offline.md                  # → 1.3.0, §5.6 effectif, §11 outillé
├── design/derivation.md                         # + notes.vue (composé), + S1 ; divergence n° 8/10
├── taxonomie-audit.md                           # + derive_horloge_constatee
├── user-stories-v1.md                           # §708 — 2 paramètres
└── module-dore.md                               # l'horodatage d'autorité nommé
```

**Structure Decision** : aucune structure nouvelle. Le cycle s'inscrit dans l'arborescence des
quatre cycles précédents. Le seul répertoire créé est `backend/tests/commun/` étendu et
`app/tests/commun/` — deux modules d'outillage partagé, à l'endroit où les tests les cherchent
déjà.

---

## Livrables détaillés

### 1. Migrations

| Migration | Contenu | RLS | Privilèges `kaya_app` |
|---|---|---|---|
| `0027_reconciliation_orpheline.sql` | Table, `CHECK` d'égalité de conditions sur le cycle de vie, index partiel `WHERE etat = 'constatee'` | `ENABLE` + `FORCE`, politique `isolation_tenant` | **`SELECT` seul** — ni `INSERT`, ni `UPDATE` : c'est ce qui prouve la provision |
| `0028_parametres_synchronisation.sql` | 2 `INSERT` au catalogue, patron de `0023` | *(catalogue existant)* | *(inchangés)* |

Détail des colonnes et des contraintes : [data-model.md](./data-model.md) §1.

### 2. Endpoints

**Aucun.** Décision motivée dans [contracts/README.md](./contracts/README.md) : pas d'endpoint de
lot (l'échec partiel est une sémantique que le rejeu idempotent rend inutile), pas d'endpoint
d'heure serveur (chaque réponse porte déjà l'horodatage d'autorité).

`notes_creer` et `notes_lister` sont employés **tels quels** : `CreerNoteRequete` porte déjà `id`,
`texte` et `horodatage_client`, posés par le module doré au cycle 001 exactement pour cela.

### 3. Structures et traits exposés

| Élément | Crate | Consommé par |
|---|---|---|
| `constater_derive(client, autorite, seuil) -> Option<Derive>` | `socle/synchronisation` | Tout service acceptant un horodatage client |
| `trait SignalDerive` | `socle/synchronisation` | Câblé sur `JournalAudit` **par la couche API** — `comptes` dépend de `synchronisation`, l'inverse serait un cycle (R-05) |
| `perimetre::schemas_applicatifs()` · `perimetre::crates_*()` | `backend/tests/commun` | Les 10 fichiers de portes |
| `tester_classe_a!` · `tester_classe_bcd!` · `tester_classe_d!` | `backend/tests/commun` | Tout cycle créant une entité |
| `surRetourPremierPlan(rappel)` | `PlatformAdapter` | `core/sync/envoi.ts` |
| `useEtatSynchronisation()` | `core/sync/etat.ts` | Témoin (composant 10) et écran `S1` |

### 4. Événements outbox

**Aucun type nouveau.** Le total du produit reste à **27**, et `TYPES_EVENEMENTS` est inchangé.
Ce cycle ne crée aucune transition d'état métier — voir [data-model.md](./data-model.md) §4.

Le contrôle qui compte : **un rejeu n'émet aucun événement**. Trois envois → une ligne, **un**
événement. Le service de note l'applique déjà ; la macro `tester_classe_a!` en fait une garantie
de toutes les entités A à venir.

### 5. Famille d'audit

`derive_horloge_constatee`, débrayée par épisode via une clé Redis à durée de vie. Contexte
`{ ecart_secondes, seuil_secondes, sens }` — **aucune clé monétaire**, donc P-10 est satisfaite sur
le JSONB. `FAMILLES_ATTENDUES` passe de **10 à 11**.

### 6. Classes hors-ligne

Registre en **1.3.0**, §5.6 déclaré effectif. **Aucune ligne ajoutée** — les deux tables du cycle y
figurent déjà depuis le 2026-07-30. Le §11 gagne le paragraphe qui dit que ses tests existent
désormais sous forme d'outillage instancié, non recopié.

### 7. Écrans

| Écran | Catégorie | Motif |
|---|---|---|
| `app/pages/notes.vue` | **Écran composé** — la catégorie ouverte au cycle 004 | Liste (comp. 08) + champ (comp. 16) + actions (01·02·03) + état vide (11) + squelette (13). Les quatre conditions vérifiées une par une, zone de charme, mention « à valider à l'atelier terrain » |
| `app/pages/mes-envois.vue` — `S1` | **Écran dérivé** | Du **composant 10** (`derivation.md` 1.2.1). Titre « **Mes envois** ». **Le nom du fichier décide de la route** : `/synchronisation` afficherait dans la barre d'adresse un mot que le lexique proscrit |
| `TemoinSynchronisation.vue` | **Composant 10** | Trois états, une forme et une phrase chacun, pouls lent (2,4 s), passage hors ligne instantané, **jamais de pourcentage** |

Le témoin est monté dans `layouts/default.vue`, donc présent sur **toutes** les pages — c'est ce
que « indicateur permanent » veut dire.

### 8. Tests d'intégration — dont les tests §0.7

| Fichier | Ce qu'il garde |
|---|---|
| `backend/tests/commun/classes.rs` | **Les macros** : rejeu triple, désordre sur les 6 ordres, inatteignabilité, double soumission |
| `backend/tests/outillage_classes.rs` | Toute entité du registre ayant une table **a** son instanciation, sinon échec en la nommant |
| `backend/tests/derive_horloge.rs` | Détection dans les **deux sens**, acceptation malgré la dérive, débrayage par épisode |
| `backend/tests/horodatage_autorite.rs` | **P-23** — aucun calcul ne s'appuie sur `horodatage_client` ; les **3 exemptions ratifiées**, ni plus ni moins |
| `backend/tests/classes_offline.rs` | Porté sur le périmètre **découvert** ; décompte conservé |
| `backend/tests/provisions_sans_logique.rs` | 6 provisions ; `reconciliation_orpheline` **non écrivable** par `kaya_app` |
| `app/tests/commun/classes.ts` | Utilitaires §0.7 côté application |
| `app/tests/file-persistance.spec.ts` | Survit au rechargement ; **jamais lisible en clair** |
| `app/tests/temoin-sync.spec.ts` | 3 états × 2 thèmes × 2 langues |
| `tests-e2e/hors-ligne.spec.ts` | **FR-005b** — chaque écran d'écriture, réseau coupé, annonce **avant** la saisie |
| `backend/tests/journee_avec_coupure.rs` | **SC-009** — clôture identique au franc près (installé à vide : la clôture est de T3, assertion de non-régression comprise) |

---

## Portes de CI — comment chacune est vérifiée, et par quel test

*Exigence de la constitution : « une porte concernée sans mécanisme de vérification est un trou du
plan ». Les 20 portes touchées, et les 5 qui ne le sont pas, sont toutes ici.*

| Porte | Effet de ce cycle | Vérifiée par | Cible non vide (exig. 4) |
|---|---|---|---|
| **P-01** | Contrat inchangé → **diff vide attendu** | `pnpm porte:p01` | Le contrat sert 40+ opérations |
| **P-01b** | Aucun `operationId` ajouté | `couverture_portes.rs::p01b_…` | Test négatif existant (doublon + absence) |
| **P-02** | 2 migrations additives, aucune modifiée | `pnpm porte:p02` | 26 migrations déjà appliquées |
| **P-03** | `synchronisation` reste **sans dépendance** vers le haut | `architecture.rs`, porté sur `perimetre::crates_*()` | Découverte comptée et comparée |
| **P-04** | `reconciliation_orpheline` **sans clé étrangère** inter-schémas | `pnpm porte:p04` | 6 schémas applicatifs |
| **P-05** | **Aucun type nouveau** ; le total reste 27 | `couverture_portes.rs::p05_…` (2 sens) | 27 types, 4 arbres de fichiers balayés |
| **P-05b** | Aucun chemin de suppression ajouté ; la file **n'est pas** un registre immuable et le plan le dit | `pnpm porte:p05b` | Outbox + journal d'audit |
| **P-06** | Non touchée — aucune capacité manipulée | `capacites_refusees.rs` | Cible existante |
| **P-07** | 1 table nouvelle, `ENABLE` + `FORCE` + politique | `rls_catalogue.rs`, décompte de tables **découvert** | Décompte comparé au catalogue |
| **P-08** | Décompte d'opérations **inchangé** — et c'est le contrôle qui le prouve | `isolation_tenant.rs` + `couverture_portes.rs::p08_…` | Toutes les opérations servies |
| **P-09** | Non touchée — aucune occupation créée | `hebergement_disponibilite.rs`, `porte:p09:negatif` | Occupations du cycle 004 |
| **P-10** | Le contexte d'audit de la dérive **ne porte aucune clé monétaire** | `pnpm porte:p10` (JSONB compris) | Toutes les colonnes + JSONB |
| **P-11** | Non touchée — aucun calcul fiscal | Tests dorés fiscaux | Installée à vide, non-régression |
| **P-12** | Non touchée — aucune règle de juridiction | `architecture.rs` | Cible existante |
| **P-13** | **Les deux versants** : type + balayage en direct réseau coupé | `app/tests/file-classe-a.spec.ts` (`@ts-expect-error`) · `tests-e2e/hors-ligne.spec.ts` · `hebergement_hors_ligne.rs` | **Opérations non-`GET` de classe B/C/D comptées** face au contrat (R-11) |
| **P-14** | Rejeu et désordre **engendrés par macro**, plus recopiés | `commun/classes.rs` + `outillage_classes.rs` | Décompte d'assertions relevé **avant** portage et comparé après |
| **P-15** | 1 capacité de plus dans l'adaptateur ; rien hors de lui | `pnpm porte:p15` | Décompte de fichiers analysés par arbre |
| **P-16** | Clés `reseau.*` employées pour la première fois ; ajouts `S1` et quarantaine | `pnpm test:i18n` | Parité fr/en sur 341+ clés |
| **P-17** | Témoin et `S1` sur jetons seuls | `pnpm lint:tokens` | Tous les `.vue` |
| **P-18** | 2 migrations → **double passe `sqlx prepare`** obligatoire | `git status --short backend/.sqlx` **puis** `touch` + check hors ligne | Le `touch` force la réévaluation — sans lui le check ne prouve rien |
| **P-19** | Aucun fichier de maquette copié | `pnpm porte:p19` | `docs/design/html/` |
| **P-20** | **Aucune dépendance ajoutée** ; lockfiles inchangés | `pnpm porte:p20` | Tous les manifestes |
| **P-21** | WebCrypto est une API du moteur, pas un hôte externe | `pnpm porte:p21` | Toutes les ressources |
| **P-21b** | Non touchée — aucune police ni icône | `pnpm porte:p21b` | 77 glyphes, 4 `woff2` |
| **P-22** | 2 écrans nouveaux, atteints **en direct et par navigation**, 2 thèmes, 2 moteurs | `pnpm porte:p22` · `porte:p22:negatif` | Routes lues de `app/pages/` |
| **P-23** | **Provenance de l'instant** — aucun calcul ne s'appuie sur `horodatage_client` | `horodatage_autorite.rs` | Crates **découverts** ; les **3 exemptions ratifiées**, ni plus ni moins |

### Les quatre exigences de couverture, appliquées aux portes que ce cycle touche

1. **Déclarer le périmètre.** Chaque fichier porté reçoit, en tête, ce qu'il lit **et ce qu'il ne
   lit pas**. Les 21 chemins en dur disparaissent ; le commentaire qui explique la découverte les
   remplace.
2. **Vérifier la complétude.** `perimetre.rs` **compte** et échoue si le total baisse (R-02).
   `tests-e2e/hors-ligne.spec.ts` rapporte le nombre d'opérations B/C/D couvertes face au contrat.
3. **Ne pas modifier l'inspecté.** Le balayage en direct ouvre les écrans en lecture ; **aucun cas
   n'écrit**. Le seul geste d'écriture du parcours est la note interne, sur un tenant de test.
4. **Prouver une cible non vide.** Chaque macro engendre au moins un test **nommé** ; les macros
   installées à vide (classe D — la certification FNE n'existe pas) portent leur assertion de
   non-régression, sur le patron de `portes_a_vide.rs`.
5. **Exercer sur les deux tenants de démonstration.** La famille d'audit nouvelle est exercée sur
   **les deux** — c'est le défaut de séquence de l'outbox qui a produit cette exigence, et il n'a
   été trouvé ni par relecture ni par une porte.
6. **Deux preuves pour toute fonction d'amorçage.** `brancherFile` et `surRetourPremierPlan` en
   reçoivent deux chacune : un test qui les exerce, **et** un test qui vérifie qu'elles sont
   appelées dans le parcours réel. C'est l'exigence née des cinq fonctions appelées nulle part.

---

## Risques, et ce qui les tient

| Risque | Pourquoi il est réel | Ce qui le tient |
|---|---|---|
| **Le portage des trois instanciations manuelles perd de la couverture** | C'est la faute la plus probable du cycle : une macro qui couvre moins que le code qu'elle remplace transforme une réécriture en régression silencieuse | Décompte d'assertions relevé **avant** et comparé **après** (FR-041, SC-013) |
| **Le module d'énumération élargit un périmètre et fait tomber des portes ailleurs** | C'est le comportement **attendu** — ces portes étaient aveugles | À traiter comme des défauts trouvés, pas comme des régressions du cycle. Chacun documenté |
| **`crypto.subtle` indisponible sur WKWebView** | Exige un contexte sécurisé ; le WebKit de Playwright **n'est pas** WKWebView | Vérifié sur les deux moteurs de P-22, et **déclaré non vérifié sur la cible** jusqu'à la coquille Tauri |
| **P-18 : le cache sqlx amputé** | Deux passes obligatoires, et le check hors ligne **ne prouve rien s'il ne recompile rien** | Procédure des deux passes + `touch` avant le check, tous deux dans le quickstart |

---

## Complexity Tracking

| Violation | Pourquoi elle est nécessaire | Alternative plus simple, et pourquoi elle est écartée |
|---|---|---|
| **Porte P-23** — le vingt-sixième contrôle, ratifié le 2026-08-02 | FR-034 exige une vérification automatisée que rien ne couvrait | *Revue humaine.* Écartée par le texte même de l'exigence, et par cinq précédents de portes vertes défectueuses qu'aucune relecture n'a trouvées |
| **Module d'énumération partagé** — une abstraction dans les tests | Le principe « pas de généricité prématurée » s'y oppose en apparence | *Corriger `classes_offline.rs` seul.* Écartée : neuf autres fichiers portent le même défaut, et il s'est reproduit à chaque cycle depuis le 002. Ce n'est pas de la généricité prématurée, c'est la troisième occurrence d'un même bogue |
| **Deux écrans pour un cycle d'infrastructure** | Un mécanisme sans passager réel est du code exporté et appelé nulle part — le défaut exact d'`initialiserTheme()` | *Livrer la file sans écran.* Écartée par arbitrage explicite de l'utilisateur, et par la sixième exigence de couverture |
