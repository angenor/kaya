---

description: "Tâches — Cycle 004 · Unités louables, formules de location et moteur de disponibilité"
---

# Tasks: Unités louables, formules de location et moteur de disponibilité

**Input**: Documents de conception de `specs/004-unites-formules-disponibilite/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), [contracts/](contracts/), [quickstart.md](quickstart.md)

**Tests**: **obligatoires, non optionnels.** La Definition of Done les impose (§0.4, points 1, 4, 5),
les portes P-03, P-05, P-07, P-08, **P-09**, P-13 n'existent que sous forme de tests, et la
garantie centrale du cycle — l'impossibilité du chevauchement — décrit un défaut **qui ne se voit
pas en relecture** : un verrou applicatif et une contrainte de base produisent le même résultat sur
un test séquentiel.

**Organisation** : par story, réordonnées par dépendance. Deux écarts assumés :

- **US2 (disponibilité) suit US1 (référentiel)** bien que les deux soient P1 : `occupation`
  référence `unite` et `formule`, et une contrainte d'exclusion se pose à la création de la table.
  L'ordre n'est pas un confort, c'est une contrainte irréversible.
- **US4 (demi-journée) suit US2** : ses plages fixes se vérifient contre la même contrainte
  d'exclusion. Livrée avant, elle n'aurait rien contre quoi s'éprouver.

## Format: `[ID] [P?] [Story] Description`

- **[P]** : parallélisable — fichiers distincts, aucune dépendance sur une tâche inachevée
- **[Story]** : US1 à US4, telles que numérotées dans [spec.md](spec.md)
- Chaque tâche porte ses chemins de fichiers exacts

## Conventions de chemin

Monorepo des cycles 001 à 003 : `backend/migrations/`,
`backend/crates/verticales/hebergement/src/`, `backend/api/src/`, `backend/tests/`, `app/`,
`scripts/ci/`, `docs/`.

---

## Note d'ordonnancement — les cinq verrous du cycle

**1 · La contrainte d'exclusion se pose À LA CRÉATION de la table, jamais après.** Ajoutée sur une
table déjà peuplée, elle échoue sur les données existantes — et il faudrait alors choisir entre
corriger l'historique et renoncer à la garantie. `0021 → 0022 → 0023 → 0024 → 0025` est appliqué **dans cet ordre**, et **aucun seed n'entre
avant `0025`**.

⚠️ **Les numéros suivent l'ordre des tâches, pas l'ordre thématique.** `sqlx` **refuse une
version antérieure à une version déjà appliquée** — le cycle 001 s'y est heurté, et l'en-tête de
`0006_provisions_comptables.sql` le consigne. Numéroter le référentiel avant les permissions
aurait bloqué la migration dès la première application.

**2 · P-09 fera échouer le build, et c'est prévu.** `backend/tests/portes_a_vide.rs` interroge
depuis le cycle 001 `table_existe(&pool, "hebergement", "occupation")` et **échoue dès que cette
table existe**. La migration `0025` et la levée de la porte sont donc **UNE SEULE TÂCHE** (T018).
Les séparer laisserait le dépôt rouge un soir sans qu'on sache si l'échec vient de l'assertion
attendue ou d'un vrai défaut.

**3 · `classes_offline.rs` doit gagner `hebergement` DÈS la première table.** Son tableau
`SCHEMAS_APPLICATIFS` ne le contient pas : sans cet ajout, les huit tables du cycle échappent
entièrement au balayage — exactement le trou trouvé sur le schéma `comptes` au cycle 003. C'est
dans T012, avec la première migration de tables.

**4 · Aucune migration n'écrit de données sur une table en `FORCE ROW LEVEL SECURITY`.** Les
`INSERT` **réussissent en n'écrivant rien**, sans erreur. Les seeds passent par la mécanique de
seeds, jamais par une migration. Seule exception : `comptes.permission`, référentiel global couvert
par la politique `administration_editeur … TO kaya_owner` posée au cycle 003.

**5 · `couverture_portes.rs` se met à jour EN DERNIER (T049).** Il suppose que toutes les
migrations, tous les points d'entrée et tous les événements existent. Le lancer plus tôt compterait
juste et couvrirait faux — son propre en-tête le dit.

---

## Note sur la référence visuelle — TROIS cas, et ce cycle en emploie deux

`docs/Kaya_Design.md` §2 porte depuis l'origine une doctrine à **trois** cas, dont le troisième
avait disparu de la consigne : on maquette, on dérive, **ou on compose**. Le tableau « on maquette
si… / on code directement si… » énumère les quatre conditions du troisième cas, et ce cycle en
rencontre un exemple exact.

| Cas | Référence exigée | Écran de ce cycle |
|---|---|---|
| **(a) Maquetté** | Le fichier d'état exact de `docs/design/html/` | **`G2`** — l'offre d'hébergement |
| **(b) Dérivé** | Sa ligne de `docs/design/derivation.md` | aucun |
| **(c) Composé** | Assemblé **uniquement** à partir des seize composants canoniques, **zone de charme seulement**, inscrit à `derivation.md` avec les mentions « composé » et « à valider à l'atelier terrain » | **`G5`** — chambres et catégories (Phase 5b) |

**Un écran de comptoir se maquette toujours** : le cas (c) est fermé à la zone de vitesse.

**Le décompte se lit de `derivation.md`, jamais d'un nombre recopié** — il en portait 42 au
cycle 002, 43 depuis l'ajout de `R0` et `A1` le 2026-08-01, et **44** après T035.

**L'écran `G2` relève du cas (a) — ÉCRAN MAQUETTÉ.** Références exactes :

| État | Fichier |
|---|---|
| Hôtel — quatre formules | `docs/design/html/G2-offre-hebergement.html` |
| Résidence — deux formules, affordance « Ajouter le passage ici » | `docs/design/html/G2-offre-hebergement-residence.html` |

**L'écran `G5` relève du cas (c) — ÉCRAN COMPOSÉ**, et sa couverture par les seize composants a été
vérifiée motif par motif : voir le tableau de la Phase 5b. **Aucun motif ne manque à la
bibliothèque**, donc rien ne part en maquettage.

**Le HTML de maquette n'est jamais copié ni déplacé vers `app/`** (porte P-19). On en lit les
valeurs et la structure ; on réimplémente en composants Nuxt avec i18n, mode sombre, RBAC et
chargement paresseux — que l'export ne contient pas.

---

## Phase 1 : Fondations documentaires (bloquantes)

**Objet** : ce qui doit être écrit avant la première ligne de code, sous peine de faire échouer une
porte ou d'inventer du vocabulaire.

- [X] T001 Ajouter à `docs/design/lexique.md` les **six entrées manquantes** du cycle, section « Concept interne → ce qu'affiche l'interface », et porter la version à 1.4.0 : `unite_deja_occupee` → « Cette chambre est déjà prise sur cette période » · `formule_hors_categorie` → « Cette formule ne s'applique pas à cette chambre » · `plage_non_fractionnable` → « Une demi-journée se loue en entier » · `intervalle_invalide` → « La fin doit être après le début » · `duree_hors_contrainte` → « Cette formule se loue de 1 h à 8 h » · `formule` → « **Formule** » (le mot est sur la maquette `G2` — « Vos formules », « Ajouter une formule » — mais absent du lexique). **Aucun de ces termes ne s'écrit dans le code avant d'être ici** : les mots « occupation », « intervalle », « palier », « exclusion » n'atteignent jamais l'interface.
- [X] T002 ✅ **Formulation validée au terrain le 2026-08-02** — inscrire à `docs/design/lexique.md` le choix entre `une_nuitee_par_occupation` et `au_prorata`, que l'exploitant doit pouvoir faire à l'écran (FR-030). Libellés retenus : « **Une seule taxe pour tout le séjour** » / « **Une taxe par nuit** ». Le mot « conversion », le mot « prorata » et le nom de l'énumération n'atteignent pas l'interface. **Ces deux formulations ne disent rien des personnes** — c'est ce qui les rend employables alors que l'axe « par client » n'est pas tranché (voir § Décisions en attente).
- [X] T003 [P] Déclarer au `§7.1` de `docs/registre-classes-offline.md` les **deux tables que le registre ne nomme pas encore** : `temps_remise_en_etat` (classe **C**, branche C2, sur le régime de sa catégorie — le registre la mentionne comme attribut, devenue table elle se déclare pour elle-même ; précédent exact : `profil_stock` au cycle 002) et `plage_demi_journee` (le registre écrit « Plages de demi-journée » sans nom de table — ligne honorée, nom précisé). Entrée au **journal §13, version 1.2.0**, dans le même changement.
- [X] T004 [P] Ajouter au récapitulatif des paramètres d'établissement de `docs/user-stories-v1.md` les **trois clés** du cycle : `heure_arrivee_standard`, `heure_depart_standard`, `seuil_bascule_nuitee_minutes`. Le principe I·c impose que le récapitulatif fasse foi et soit mis à jour dans le même changement que l'implémentation.

**Point de contrôle** : le vocabulaire et les classes sont écrits. Le code peut commencer.

---

## Phase 2 : Socle du cycle (bloquant pour toutes les stories)

**⚠️ CRITIQUE** : aucune story ne démarre avant la fin de cette phase.

- [X] T005 Créer `backend/migrations/0021_schema_hebergement.sql` : `CREATE SCHEMA IF NOT EXISTS hebergement` + `GRANT USAGE ON SCHEMA hebergement TO kaya_app`. **Une migration dédiée**, comme `0014_schema_comptes.sql` — un `CREATE SCHEMA` glissé dans une migration ancienne produirait un écart entre schémas déclarés et réels que P-04 fait échouer. **Ne pas réinstaller `btree_gist`** : `0001_roles_et_schemas.sql:93` l'a fait, elle est globale à la base.
- [X] T006 Étendre la liste des schémas inspectés par `scripts/ci/jointures-inter-schemas.sh` à `hebergement`, et vérifier que son décompte de requêtes analysées par schéma le reprend (porte **P-04**, exigence de périmètre déclaré).
- [X] T007 Remplacer la coquille `backend/crates/verticales/hebergement/src/lib.rs` du cycle 001 par la structure du crate : modules `referentiel`, `occupation`, `tarification`, `traits`, `erreurs`. Déclarer les dépendances déjà présentes au `Cargo.toml` (`kaya-domain`, `kaya-etablissements`, `kaya-stocks`) — **aucune dépendance nouvelle**.
- [X] T008 Écrire `backend/crates/verticales/hebergement/src/erreurs.rs` : le helper `est_violation_exclusion(&sqlx::Error, contrainte: &str) -> bool`, **écrit une seule fois**, et son test unitaire. ⚠️ `ErrorKind::ExclusionViolation` existe en sqlx 0.9 mais **le trait `DatabaseError` n'expose PAS `is_exclusion_violation()`** — il porte les trois autres et s'arrête là ; l'écrire par symétrie ne compile pas. `ErrorKind` est `#[non_exhaustive]` : employer `matches!`, jamais un `match` exhaustif. Vérifier aussi le **nom de contrainte** via `constraint()`, sans quoi une seconde contrainte d'exclusion ferait passer ses violations pour des doubles attributions. Voir [research.md R-03](research.md).
- [X] T009 Créer `backend/migrations/0022_permissions_hebergement.sql` : les **cinq permissions**, `module_code = 'HEBERGEMENT'` — les **premières du produit rattachées à un module d'activité**, ce que la migration `0016` du cycle 003 annonce nommément. Toujours **sans clé étrangère** vers `etablissements.module_activite` (ce serait une clé inter-schémas, P-04). Attribution : `proprietaire` et `gerant` → les cinq ; `receptionniste` → tout sauf `heb.offre.gerer`.
- [X] T010 Étendre dans `backend/tests/` le test existant qui lit le référentiel des modules **à travers le trait `RegistreModules`** pour couvrir les cinq permissions nouvelles. **C'est la première fois qu'il vérifie autre chose que `NULL`** — jusqu'ici sa cible était vide au sens de la constitution.
- [X] T011 [P] Créer `backend/migrations/0023_parametres_hebergement.sql` : les trois clés au catalogue `etablissements.parametre_catalogue`, portée la plus basse `ETABLISSEMENT`. **Ne PAS y mettre** le temps de remise en état (il varie par catégorie *et* par formule → table dédiée), les plages de demi-journée ni le barème (référentiels, §7.1 du registre).

**Point de contrôle** : le schéma existe, le crate compile, les permissions et paramètres sont posés.

---

## Phase 3 : US1 — Adjoua règle l'offre de son établissement (P1) 🎯 MVP

**Objectif** : le référentiel existe comme donnée et se voit à l'écran. Cinq catégories, dix-sept
unités, une salle de réunion, et pour chaque catégorie les formules qu'elle accepte.

**Test indépendant** : charger les seeds, ouvrir l'écran de l'offre, vérifier que les formules et
leurs prix s'y lisent en mode clair et en mode sombre — **sans qu'aucune occupation n'existe**.

### Migration et modèle

- [X] T012 [US1] Créer `backend/migrations/0024_referentiel_hebergement.sql` : les **six tables de classe C** — `categorie`, `temps_remise_en_etat`, `unite`, `formule`, `bareme_palier`, `plage_demi_journee` — avec pour chacune `ENABLE` + `FORCE ROW LEVEL SECURITY`, la politique `isolation_tenant` (`USING` **et** `WITH CHECK`, `current_setting('app.current_tenant', true)`), et les privilèges `SELECT, INSERT, UPDATE, DELETE` à `kaya_app`. Voir [data-model.md §3](data-model.md). **`tenant_id` est porté par les tables filles** bien que dérivable du parent : une politique RLS qui devrait joindre le parent serait plus lente et plus fragile. **Dans la MÊME tâche** : ajouter `"hebergement"` à `SCHEMAS_APPLICATIFS` de `backend/tests/classes_offline.rs` — sans quoi les tables du cycle échappent au balayage.
- [X] T013 [US1] Vérifier les contraintes du référentiel par test dans `backend/tests/hebergement_referentiel.rs` : `formule_famille_unique` (FR-021), `formule_durees_coherentes`, `formule_heure_sup_reservee_au_passage`, **`formule_regle_fiscale_coherente`** (une formule assujettie sans règle est impossible à enregistrer — c'est ce qui supprime le besoin d'un troisième état d'écran), `bareme_palier` clé primaire `(formule_id, duree_minutes)` qui rend un barème désordonné impossible à constituer, `plage_bornes`. Et l'**absence** de colonne `statut_occupation` sur `unite` — elle est dérivée, l'inscrire en table rendrait possible de la poser à la main.
- [X] T014 [US1] Écrire `backend/crates/verticales/hebergement/src/referentiel/{modele,repository}.rs` : types de domaine, macros `query!` **littérales** (sqlx 0.9 exige `AssertSqlSafe` sur toute requête non littérale), transaction **prise en paramètre** et jamais ouverte par le repository. Consommer le fuseau et la devise via `EstablishmentDirectory`, jamais par jointure inter-schémas.
- [X] T015 [US1] Écrire `backend/crates/verticales/hebergement/src/referentiel/service.rs` : validations que la base ne peut pas porter — une formule `PASSAGE` porte au moins un palier, une `DEMI_JOURNEE` au moins une plage (la dépendance va de l'enfant au parent, aucune contrainte de table ne l'exprime) ; refus explicite de toute famille hors des quatre (FR-022, patron du cycle 002) ; refus de suppression d'une catégorie qui porte des unités, avec motif nommant ce qui l'occupe. Émettre `heb.formule.creee`, `heb.formule.modifiee`, `heb.categorie.tarif_modifie` **dans la transaction**, via `OutboxWriter::ecrire(tx, …)` — et **jamais sur rejeu**.

### API

- [X] T016 [US1] Écrire `backend/api/src/routes/hebergement_referentiel.rs` : les **neuf opérations** 1 à 8 et **5b** de [contracts/http-api.md §1](contracts/http-api.md). `#[utoipa::path]` **sans** `path` ni verbe (déduits de l'attribut Actix, feature `actix_extras`), `operation_id` explicite sur chacune (P-01b), montage par `service(...)` et **jamais** `route(...)` — `utoipa-actix-web` ne collecte que depuis `service(...)`. Garde de permission `heb.offre.lire` / `heb.offre.gerer`. Vérification du module actif via `RegistreModules`, refus normalisé du cycle 002. **L'opération 8 porte les deux champs fiscaux** : c'est là que l'exploitant active la taxe et choisit sa règle. **L'opération 5b (`PUT /unites/{unite_id}`) ne porte QUE `code` et `etage`** — ce que le registre §7.1 classe littéralement (« `unite` — code, étage », classe C). Un corps portant `categorie_id`, `statut_menage` ou une mise hors service est **refusé explicitement**, jamais ignoré : ces trois-là sont classés ailleurs (effet fiscal non classé · classe A HEB-06 · classe B HEB-06). **Terminer par** : régénération du client TypeScript (`pnpm generer:client`), commit du diff, `cargo build` vert.
- [X] T017 [P] [US1] Étendre `backend/tests/isolation_tenant.rs` aux neuf opérations : le tenant A ne lit ni n'écrit aucune ligne du tenant B (porte **P-08**), avec régime d'isolation déclaré pour chacune.

**Point de contrôle US1 (backend)** : le référentiel est servi, isolé, permissionné.

---

## Phase 4 : US2 — Deux clients ne peuvent jamais recevoir la même unité (P1)

**Objectif** : la double attribution devient **impossible**, pas improbable.

**Test indépendant** : deux transactions concurrentes sur des intervalles chevauchants, une seule
réussit — **et l'échec provient de la contrainte**, pas d'un verrou applicatif.

- [X] T018 [US2] ⚠️ **TÂCHE INDIVISIBLE — migration + levée de P-09 dans le même changement.** Créer `backend/migrations/0025_occupation.sql` : colonne **`periode TSTZRANGE NOT NULL`** (remise en état comprise), `debut_client` / `fin_client` (bornes commerciales), `statut`, `libere_le` ; la contrainte **`occupation_sans_chevauchement EXCLUDE USING gist (unite_id WITH =, periode WITH &&)`** ; les `CHECK` `occupation_periode_non_vide` (le **seul** contournement possible : `&&` est faux dès qu'un intervalle est vide, une ligne `[14h, 14h)` occuperait sans bloquer), `occupation_periode_semi_ouverte`, `occupation_bornes_client_coherentes`, `occupation_liberation_coherente` ; RLS `ENABLE` + `FORCE` + politique ; privilèges **`SELECT, INSERT, UPDATE` — jamais `DELETE`** (une occupation se libère, elle ne s'efface pas ; accorder `DELETE` rendrait faux le classement en B sans que rien ne le signale). **Aucun index supplémentaire** : l'index GiST de la contrainte sert déjà la requête la plus fréquente du produit. **Dans le même changement** : lever P-09 de `backend/tests/portes_a_vide.rs` vers `backend/tests/hebergement_disponibilite.rs`, y laisser un **relais** qui échoue si ce fichier disparaît (précédent P-06 du cycle 002), et ramener le décompte des portes à vide de deux à **une** (P-11 seule).
- [X] T019 [US2] Écrire dans `backend/tests/hebergement_disponibilite.rs` les **trois assertions dictées par le message du cycle 001** : (1) `periode` est de type `tstzrange` **et** aucune colonne `debut`/`fin` de type `date` n'existe sur la table — lecture d'`information_schema` ; (2) une contrainte `contype = 'x'` la protège, dont la définition contient les deux opérateurs — lecture de `pg_constraint` ; (3) **le test de concurrence**.
- [X] T020 [US2] ⚠️ **Le test qui distingue une garantie d'une coïncidence.** Écrire `deux_attributions_concurrentes_une_seule_reussit` dans `backend/tests/hebergement_disponibilite.rs` : **deux transactions PostgreSQL distinctes**, insertion dans chacune **sans commit**, puis commit des deux. Asserter que exactement une réussit, que l'échec est un `ErrorKind::ExclusionViolation`, et que `constraint()` rend `occupation_sans_chevauchement`. **Asserter la CAUSE, pas seulement l'existence du refus** : un test qui se contenterait de « une seule a réussi » passerait au vert sur `SELECT … FOR UPDATE`, sur `SERIALIZABLE` ou sur un verrou applicatif — trois mécanismes qui se dégradent sous charge sans rien signaler. **Deux transactions suffisent** : elles prouvent que la base rejette, là où mille prouveraient la même chose en occupant la CI (**SC-001**). `futures 0.3.33` est déjà au dépôt, motif inscrit : « tests de concurrence ».
- [X] T020b [US2] ⚠️ **Le critère qui prouve que la garantie vient de la BASE et non du code (SC-002).** Ajouter à `backend/tests/hebergement_disponibilite.rs` un test qui **neutralise la vérification préalable applicative** — écriture directe par le repository, en contournant le service — et constate que l'attribution chevauchante **échoue quand même**. Sans lui, rien ne distingue une garantie d'une coïncidence : T028 retire la *contrainte* et prouve donc l'inverse. Le principe IV l'exige mot pour mot : « garantie par une contrainte d'exclusion PostgreSQL, **pas par un verrou applicatif** ».
- [X] T021 [US2] Écrire les tests de forme d'intervalle dans `backend/tests/hebergement_disponibilite.rs` : `intervalle_vide_refuse`, `occupations_contigues_coexistent` (la borne de fin est **exclue**), `remise_en_etat_bloque_la_suivante` (12 h + 2 h de ménage → 13 h refusé, 14 h accepté — **par la même contrainte** que tout chevauchement, jamais par une règle à part), `intervalle_traversant_minuit` (22 h → 6 h n'est pas un cas spécial).
- [X] T022 [US2] Écrire `backend/crates/verticales/hebergement/src/occupation/{modele,repository}.rs` : `PgRange<time::OffsetDateTime>` → `TSTZ_RANGE` (vérifié dans `sqlx-postgres-0.9.0/src/types/range.rs:213`). ⚠️ Rappel sqlx 0.9 : `query!` sur un `SELECT` produit un `Map` sans `.execute()` — employer `.fetch_one(&mut **tx)`, avec le **déréférencement double** attendu pour exécuter sur une transaction empruntée.
- [X] T023 [US2] Écrire `backend/crates/verticales/hebergement/src/occupation/service.rs` : le serveur **calcule lui-même** la borne haute de `periode` en ajoutant le temps de remise en état de la catégorie pour la famille de la formule — le client ne l'envoie pas et ne peut pas l'influencer, sans quoi il la mettrait à zéro. **Tenter l'insertion et traduire la violation ; ne JAMAIS lire d'abord pour décider** — une lecture préalable serait exactement le verrou applicatif que le principe IV refuse. Émettre `heb.occupation.attribuee` et `heb.occupation.liberee` dans la transaction. La libération **raccourcit** `periode` et pose `statut = 'liberee'` : jamais un `DELETE`.
- [X] T024 [US2] Écrire `backend/crates/verticales/hebergement/src/traits.rs` : `MoteurDisponibilite`, dont `attribuer` **prend la transaction** — c'est ce qui permettra au check-in de SEJ-02 d'attribuer l'unité et d'ouvrir la note dans une seule transaction, là où un trait prenant un pool imposerait une saga pour une opération qui n'en demande pas.
- [X] T025 [US2] Écrire `backend/api/src/routes/hebergement_disponibilite.rs` : les **opérations 9 à 11**. L'interrogation de disponibilité est **une lecture qui ne garantit rien** — le contrat le dit, et un client qui la traiterait comme une réservation reproduirait le verrou refusé. Codes de refus **distincts** : `unite_deja_occupee` (409), `formule_hors_categorie`, `intervalle_invalide`, `duree_hors_contrainte` (422). **`200` sur rejeu, pas `409`** : un client hors ligne qui vide sa file ne doit pas voir d'erreur pour une écriture déjà acceptée. **Terminer par** : annotations utoipa à jour, `pnpm generer:client`, commit du diff, build vert.
- [X] T026 [P] [US2] Étendre `backend/tests/isolation_tenant.rs` et `backend/tests/rls_catalogue.rs` aux opérations 9 à 11 et aux tables `occupation` (portes **P-08**, **P-07**).
- [X] T027 [P] [US2] Étendre `backend/tests/outbox_transactionnel.rs` aux cinq types d'événements du cycle : `heb.occupation.attribuee`, `heb.occupation.liberee`, `heb.formule.creee`, `heb.formule.modifiee`, `heb.categorie.tarif_modifie`. Vérifier l'émission **dans** la transaction et **l'absence** d'émission sur rejeu — sinon le grand livre devient le journal des tentatives réseau du terminal. Charges utiles au nommage monétaire réservé : `prix_mineur` entier + `devise` au même niveau (**P-10**).
- [X] T028 [US2] Écrire le **test négatif de P-09** dans `scripts/ci/` sur le modèle de `porte:p22:negatif` : retirer la contrainte d'exclusion sur une base de test, constater l'échec des trois assertions, remettre. **Une porte qui n'a jamais échoué n'est pas une porte** — leçon des quatre portes vertes défectueuses du cycle 001.

**Point de contrôle US2** : la double attribution est impossible, et le test le prouve par la cause.

---

## Phase 5 : Écran de l'offre — `G2` (sert US1)

**Référence visuelle — cas (a), ÉCRAN MAQUETTÉ** :
`docs/design/html/G2-offre-hebergement.html` (hôtel, quatre formules) et
`docs/design/html/G2-offre-hebergement-residence.html` (résidence, deux formules).
**Jamais copiés ni déplacés vers `app/`** (porte P-19) : on en lit les valeurs et la structure, on
réimplémente en composants Nuxt avec i18n, mode sombre, RBAC et chargement paresseux — que l'export
ne contient pas.

- [X] T029 [US1] Créer `app/pages/hebergement.vue` et le module `app/modules/hebergement/`. ⚠️ **UNE SEULE racine, et c'est un élément** — jamais un `v-if`/`v-else` de premier niveau : une racine multiple compile en fragment, un fragment dont la branche active est un `defineAsyncComponent` non résolu a un `el` nul, et Vue lève `Cannot read properties of null (reading 'parentNode')` à la navigation suivante ; l'écran ne se monte pas, l'ancien reste affiché, l'adresse a pourtant changé. **Relire la huitième couche de `docs/module-dore.md` avant** : elle dit où va le thème, où va la session, et ce que rend le layout. Chargement paresseux par module.
- [X] T030 [US1] Écrire `app/modules/hebergement/{EcranOffre,CarteFormule}.vue` et `donnees.ts` d'après les deux fichiers d'états. Tout montant par `app/core/format/montant.ts` (`formaterMontant(montantMineur, codeDevise)`) — **jamais** `Intl.NumberFormat`, dont le séparateur dépend de l'ICU embarqué, ni un `money()` recopié de `tokens.md`. Tout champ par `app/core/design-system/ChampSaisie.vue`. Aucune couleur ni espacement littéral (**P-17**), Tailwind 4 d'abord, mode sombre par la variante `dark:` et jamais une seconde palette.
- [X] T031 [US1] Écrire `app/modules/hebergement/modifier-formule.ts` selon **la septième couche** de `docs/module-dore.md` : appel typé, squelette de chargement, refus métier en langue utilisateur, validation au champ, action **absente** sans permission (pas grisée), **refus immédiat hors ligne** (tout ce cycle est en classe B ou C), rafraîchissement sans rechargement. Y exposer les deux champs fiscaux — activation de la taxe et choix de la règle, **avec les libellés validés en T002**.
- [X] T032 [US1] Ajouter les clés i18n **`fr` et `en`** dans `app/core/i18n/{fr,en}.json` **pour les deux écrans du cycle — `G2` et `G5` — en un seul passage**. Un cycle, un passage sur les clés : deux tâches écrivant le même JSON se marcheraient dessus, et le marqueur `[P]` serait un piège. Aucune chaîne en dur (**P-16**), parité des clés vérifiée par `pnpm test:i18n`. Vocabulaire du lexique **exclusivement** : « chambre » / « logement » / « salle » selon le contexte, « Taxe de séjour comprise dans le prix », « Durée dépassée : passé au tarif 4 h », « Chambre indisponible 30 min (ménage) », et pour `G5` « chambre » / « logement » / « salle » selon le contexte. Les mots « unité louable », « catégorie d'unité », « occupation », « intervalle », « palier » et le vocabulaire des classes hors-ligne **n'atteignent jamais l'interface**.
- [X] T033 [US1] Vérifier `app/pages/hebergement.vue` **en mode clair ET en mode sombre** (DoD §0.4 point 8), et **les deux états maquettés** : l'hôtel à quatre formules et la résidence à deux — cette dernière portant l'affordance « Ajouter le passage ici », qui est la preuve visuelle qu'aucune formule n'est réservée à un type d'établissement.
- [X] T034 [US1] Étendre `scripts/ci/parcours-reel.sh` (**P-22**) à la route `/hebergement` : elle doit s'ouvrir **par navigation interne ET par chargement direct de l'adresse**, sans erreur de console, sur **Chromium et WebKit**, dans les deux thèmes. ⚠️ Le WebKit de Playwright **n'est pas** WKWebView : un vert dit « tourne sur un moteur WebKit », jamais « vérifié sur la cible » — la vérification sur WKWebView viendra avec la coquille Tauri. Vérifier que le décompte par projet compte la route nouvelle : un moteur sans cas doit faire échouer.

**Point de contrôle US1 complet** : Adjoua voit et corrige son offre. **MVP livrable.**

---

## Phase 5b : Écran composé `G5` — Chambres et catégories (sert US1)

**Référence visuelle — cas (c), ÉCRAN COMPOSÉ.** `docs/Kaya_Design.md` §2 porte depuis l'origine
une colonne « on code directement si » à quatre conditions. Cet écran les coche toutes :

| Condition | Vérification |
|---|---|
| Liste, formulaire ou fiche suivant un motif déjà posé | Une liste et deux formulaires |
| Conception entièrement issue de la bibliothèque | **Vérifiée composant par composant** ci-dessous |
| Consulté rarement, par un utilisateur formé | Adjoua règle son parc à l'ouverture, puis y revient à la marge |
| Personne n'a de doute sur ce à quoi il ressemble | Une liste de chambres et un formulaire |

**Zone de charme**, au sens de la règle qui tranche (`Kaya_Design.md` §1) : Adjoua n'est ni debout,
ni pressée, sans client en face ni argent en jeu. **Un écran de comptoir se maquette toujours** —
celui-ci n'en est pas un.

**Couverture par les seize composants — vérifiée, aucun motif ne manque** :

| Besoin | Composant | Note |
|---|---|---|
| Liste des catégories et des unités | **08 · Ligne de liste** | Son rôle nomme littéralement « chambres » ; actions de bord au survol |
| Formulaires de création et d'édition | **16 · Champ de saisie** | Seul composant d'écriture du produit |
| Choix de la catégorie d'une unité | **16**, état **« choix fermé (`<select>`) »** | ⚠️ **Pas le composant 12** : sa règle est explicite — « au-delà de quatre options c'est une liste, pas un segment ». Deloria a **six** catégories, salle de réunion comprise. Un segmenté à six options ne tient pas sur 372 px |
| Actions | **01 · 02 · 03** | Principal, secondaire, discret |
| Aucune unité dans une catégorie | **11 · État vide illustré** | |
| Chargement | **13 · Squelette de chargement** | Même hauteur de ligne que le contenu réel |

- [ ] T035 [US1] ⚠️ **TÂCHE BLOQUANTE POUR T036–T039** — inscrire `G5` Chambres et catégories à `docs/design/derivation.md` : ligne portant la mention **« composé »** et **« à valider à l'atelier terrain »**, la liste des composants employés (08, 16, 01–03, 11, 13), et la mise à jour du décompte (43 → 44 écrans, avec la catégorie « composés » distincte des maquettés et des dérivés). Porter la version du document. **Sans cette ligne, l'écran n'est pas codable au titre de la porte P-19** — même mécanique que `R0` au cycle 003. Y consigner aussi le rétablissement du **troisième cas** de `Kaya_Design.md` §2, que la matrice ne reflétait pas.
- [X] T036 [US1] Créer `app/pages/chambres.vue` et `app/modules/hebergement/EcranChambres.vue`. ⚠️ **UNE SEULE racine, et c'est un élément** — jamais un `v-if`/`v-else` de premier niveau. Chargement paresseux par module. Route filtrée par `heb.offre.lire` ; l'écran est **absent** de l'accueil sans cette permission, jamais grisé.
- [X] T037 [US1] Écrire `app/modules/hebergement/ListeUnites.vue` sur le **composant 08** : les unités groupées par catégorie, code en mono en colonne de largeur fixe, ligne entière cliquable, actions de bord au survol seulement. État vide par le **composant 11** quand une catégorie n'a aucune unité, squelette par le **13** au chargement. **Aucun statut d'occupation affiché** : il est dérivé, et son écran est `R2` (tranche SEJ). **Aucune action sur le statut de ménage** : c'est HEB-06, hors périmètre.
- [X] T038 [US1] Écrire `app/modules/hebergement/{FormulaireCategorie,FormulaireUnite}.vue` sur le **composant 16** uniquement. ⚠️ **Le formulaire d'unité sert la création ET la correction de `code` et `etage`, et rien d'autre** — le registre §7.1 classe « `unite` — code, étage », et un écran de gestion qui ne saurait pas corriger un code de chambre n'est pas un écran de gestion : une unité mal nommée puis occupée deviendrait définitive, la suppression étant impossible dès qu'une occupation la référence. **Ni changement de catégorie** (effet tarifaire et fiscal, non classé au registre — se spécifie, ne se glisse pas dans un `PUT` de correction), **ni sous-statut de ménage** (classe A, HEB-06), **ni mise hors service** (classe B, HEB-06). Forme : étiquette toujours visible au-dessus, `h-11` (`h-12` jamais nécessaire ici — zone de charme), erreur à **trois signaux** (bordure `danger`, message, icône `ph-fill ph-warning-circle`), l'aide s'efface pendant l'erreur. Le choix de catégorie emploie l'état **« choix fermé »**, pas le composant 12. Écriture selon **la septième couche** de `docs/module-dore.md` : refus métier en langue utilisateur, validation au champ, action absente sans `heb.offre.gerer`, **refus immédiat hors ligne** (classe C), rafraîchissement sans rechargement.
- [X] T039 [US1] Vérifier `app/pages/chambres.vue` **en mode clair ET en mode sombre** (DoD §0.4 point 8) — ses clés i18n sont écrites en T032, en un seul passage. Vocabulaire du lexique **exclusivement** : « chambre » / « logement » / « salle » selon le contexte, jamais « unité louable » ni « catégorie d'unité ». Étendre `scripts/ci/parcours-reel.sh` à la route `/chambres` : ouverture **en direct ET par navigation**, sur **Chromium et WebKit**, dans les deux thèmes (**P-22**).

**Point de contrôle** : Adjoua crée et corrige ses chambres sans passer par les seeds.

---

## Phase 6 : US3 — Yao chiffre un passage, et son dépassement (P2)

**Objectif** : le montant d'un passage, sa rebascule de palier et sa bascule en nuitée.

**Test indépendant** : des cas figés donnant, pour une durée réelle et un barème, un montant et une
décision de rebascule — sans écran, sans occupation persistée, sans réseau.

- [X] T040 [US3] Écrire `backend/crates/verticales/hebergement/src/tarification/bareme.rs` : **fonction pure**, arithmétique **entière** sur des `i64` d'unité mineure (P-10), aucun flottant. Ordre imposé : (1) durée réelle en secondes ; (2) **si durée ≥ seuil → bascule en `NUITEE`, fin du calcul** — ce n'est pas un palier majoré, c'est un changement de formule ; (3) premier palier dont la durée ≥ durée réelle ; (4) sinon dernier palier + `ceil((durée − durée du dernier palier) / 1 h) × prix_heure_supplementaire` — **toute heure entamée est due**. Le point 2 précède le 3, et l'inverser produirait un empilement d'heures là où la nuitée s'applique.
- [X] T041 [US3] Écrire `backend/tests/hebergement_tarification.rs`, cas figés : 2 h → **2 800** · 4 h 10 → **6 200** (5 000 + 1 × 1 200) · 20 min → **1 500** (le premier palier est dû en entier, il n'y a pas de tarification en dessous) · 8 h → **bascule en nuitée** · barème sans palier refusé.
- [X] T042 [US3] Écrire `backend/crates/verticales/hebergement/src/tarification/service.rs` et le trait `MoteurTarification`. **Le moteur calcule, il ne facture pas** : aucune ligne de note n'est écrite — la note est SEJ-03, tranche T2. Toute durée depuis **l'horodatage d'autorité serveur** : lire `cree_le` de l'occupation et `now()` en SQL, **jamais** un instant reçu du client. Tracer toute rebascule au registre des actions (CPT-04) via le trait d'audit de `socle/comptes`, dans la même transaction, avec la durée constatée et les deux paliers.
- [X] T043 [US3] Écrire dans `backend/tests/hebergement_tarification.rs` le test qui prouve l'indépendance à l'horloge du terminal : une horloge décalée de 40 minutes donne **le même montant** qu'un terminal à l'heure. Le cadrage §11 le désigne comme le piège du passage : « le passage aggrave la sensibilité à l'horloge ».
- [X] T044 [US3] Écrire `backend/api/src/routes/hebergement_tarification.rs` : **opération 12**. L'appel ne prend **aucun instant en paramètre** — le serveur les lit lui-même, de sorte qu'un client ne puisse pas influencer la durée facturée. **Terminer par** : annotations utoipa, `pnpm generer:client`, commit du diff, build vert.

---

## Phase 7 : US4 — Adjoua vend une demi-journée en salle de réunion (P3)

**Objectif** : les plages fixes non fractionnables, et leur composition avec la remise en état.

**Test indépendant** : deux demi-journées consécutives sur la même unité, et un fractionnement
refusé.

- [X] T045 [US4] Écrire la validation des plages dans `backend/crates/verticales/hebergement/src/occupation/service.rs` : conversion des `TIME` en instants **au serveur**, avec le fuseau de l'établissement lu par `EstablishmentDirectory`. La comparaison se fait **après** conversion, sur des instants — comparer des heures murales échouerait au passage de minuit. Refus `plage_non_fractionnable` si l'intervalle demandé ne coïncide pas avec une plage déclarée.
- [X] T046 [US4] Écrire les tests de demi-journée dans `backend/tests/hebergement_disponibilite.rs` : 9 h – 11 h refusé (non fractionnable) · 8 h – 12 h puis 13 h – 16 h acceptées avec 1 h de battement · **la même paire refusée si le temps de remise en état passe à 2 h** — par la **même contrainte** d'exclusion que tout chevauchement, ce qui prouve que la remise en état n'est jamais une règle à part · 8 h désigne 8 h à Abengourou quelle que soit l'horloge du terminal ou du serveur.

---

## Phase 8 : Seeds, recollement et revue

- [X] T047 Écrire les seeds Deloria dans `backend/migrations/seeds/` : **17 unités en 5 catégories** (A1–A3 standard 12 500 · B1–B5 classique 15 500 · C1–C4 classique supérieure 17 500 · D1–D2 supérieure A 20 500 · E1–E3 supérieure B 25 500) **plus la salle de réunion**, catégorie dédiée — **pas une entité nouvelle** ; barème de passage 1 h 1 500 · 2 h 2 800 · 3 h 4 000 · 4 h 5 000 · h. suppl. +1 200 ; plages 8 h – 12 h et 13 h – 16 h ; remise en état passage 30 min, nuitée 2 h, demi-journée 1 h. **Fiscalité** : `NUITEE` assujettie avec `une_nuitee_par_occupation` (500 F pour un séjour de trois nuits, pas 3 × 500) ; `PASSAGE` et `DEMI_JOURNEE` **non assujetties** — constat d'exploitation, et le paramètre reste activable. ⚠️ **Jamais par migration** : `FORCE ROW LEVEL SECURITY` fait réussir un `INSERT` de migration **en n'écrivant rien**, sans erreur.
- [X] T048 [P] Peupler le second tenant « Résidence Test » à **quatre unités**, mois et nuitée seulement — il éprouve qu'aucune formule n'est réservée à un type d'établissement, et qu'un établissement fonctionne de bout en bout sans qu'aucun code ne suppose l'existence du passage. Étendre `backend/tests/seeds_rejouables.rs` : rechargement en une commande, idempotent, décompte des unités des deux tenants.
- [ ] T049 ⚠️ **En dernier, jamais en parallèle.** Mettre à jour les quatre décomptes de `backend/tests/couverture_portes.rs` : P-05 **22 → 27** types d'événements · P-07 **26 → 34** tables · P-08 **43 → 56** opérations · P-01b **43 → 56** `operationId`. **Les nombres se relisent du catalogue système et du contrat, jamais d'une constante recopiée** : tout écart avec les valeurs attendues du plan est réel, se justifie à l'endroit où il se constate, et **ne se résorbe pas en ajustant le tableau** — trois nombres du plan du cycle 003 démentaient la réalité.
- [X] T050 [P] Étendre `backend/tests/provisions_sans_logique.rs` à `prestation_incluse` (migration `0026_provision_prestation_incluse.sql`, à créer dans cette tâche) : table présente, RLS activée et forcée, **aucun privilège** accordé à `kaya_app`, aucun endpoint, aucun écran. ⚠️ `quantite` en **`NUMERIC`, jamais entier** — un petit-déjeuner se compte à l'unité, une blanchisserie au kilo ; passer d'entier à décimal après mise en production imposerait de migrer toutes les lignes de tous les clients.
- [X] T051 [P] Écrire `backend/tests/hebergement_hors_ligne.rs` (**P-13**) : aucune des **treize** opérations n'est atteignable depuis un chemin de code exécutable hors ligne — le référentiel est en classe C, l'occupation en classe B. Test d'isolation multi-tenant pour la classe C, test de concurrence pour la classe B (déjà en T020), tels que `docs/registre-classes-offline.md` §11 les impose.
- [X] T052 Vérifier **P-03** dans `backend/tests/architecture.rs`, avec sa cible enfin non vide : aucun crate de `socle/` ne dépend de `verticales/`, **et** au moins un crate de `verticales/` porte des symboles publics — sans cette seconde assertion la porte redeviendrait indistinguable d'une porte à cible vide. Piège à surveiller : faire remonter un type de `hebergement` dans une signature de `socle/`. Les charges utiles d'événements sont du **JSON opaque** pour le socle.
- [X] T053 Exécuter `backend/tests/agnosticite_socle.rs` — **c'est la première fois qu'il peut échouer.** Jusqu'ici il prouvait que le socle n'exigeait rien d'une verticale, mais aucune verticale n'existait pour le contredire. S'il passe avec `hebergement` livré, « aucun crate partagé ne suppose l'existence d'un hébergement » cesse d'être une intention pour devenir un fait mesuré.
- [ ] T054 ⚠️ **La double passe de `cargo sqlx prepare`, qui coûte une journée si elle est manquée.** Ce cycle ajoute des requêtes des **deux** côtés — seeds (binaire `seeds`) et tests d'intégration —, or lancé depuis `backend/` il perd les binaires, et depuis `backend/api/` il perd les tests. Suivre la procédure à deux passes de `CLAUDE.md`, puis les **deux** contrôles **dans cet ordre** : `git status --short backend/.sqlx` (aucune suppression, que des ajouts) puis `SQLX_OFFLINE=true DATABASE_URL= cargo check --workspace --all-targets --locked`. Le second seul ne suffit pas : un cache amputé d'une requête inutilisée par le check passerait.
- [ ] T055 [P] Reporter à `docs/module-dore.md` le **retour du spike, promis par le cycle 001** : l'apport de sqlx `#3918` est vérifié **et partiel** — `ErrorKind::ExclusionViolation` existe, mais `DatabaseError` n'expose pas l'accesseur symétrique. Consigner aussi que `PgRange<OffsetDateTime>` → `TSTZ_RANGE` est validé, et que la concurrence réelle sur `tstzrange` est exercée. Signaler, **sans la changer**, que la note du §2 de `docs/versions-gelees.md` gagnerait sa limite, et que **le point ouvert de son en-tête peut être retiré** — c'est la revue mensuelle du 2026-08-31 qui tranche.
- [ ] T056 Exécuter toutes les portes : `pnpm porte:p01 p02 p04 p05b p10 p15 p19 p20 p21 p21b p22` et `pnpm porte:p22:negatif`, plus le test négatif de P-09 (T028). ⚠️ **`porte:p20` ne doit constater aucun mouvement de lockfile** : le cycle n'ajoute **aucune dépendance**, et un lockfile modifié signalerait une dérive.
- [ ] T057 **Revue Definition of Done** (`docs/user-stories-v1.md` §0.4) — les dix points, un par un, avec la preuve de chacun : (1) critères couverts par tests unitaires **et** d'intégration sur les transitions d'état ; (2) annotations utoipa à jour, client TS régénéré **sans diff manuel** ; (3) migration versionnée, `cargo sqlx prepare` vert, seeds à jour ; (4) **RLS activée et forcée** sur les huit tables, avec test d'isolation ; (5) **classe hors-ligne déclarée** pour chaque entité, avec son test ; (6) événement outbox pour tout changement d'état ; (7) clés i18n `fr` et `en`, aucune chaîne en dur ; (8) écran vérifié en clair **et** en sombre ; (9) paramètres exposés dans la configuration d'établissement ; (10) **sans objet** — ce cycle n'imprime aucun document.

---

## Écrans non codés — signalés, pas inventés

| Écran envisageable | Statut | Décision |
|---|---|---|
| Gestion des catégories et des unités | **ÉCRAN COMPOSÉ** — les quatre conditions de `Kaya_Design.md` §2 sont remplies, zone de charme, couverture par les seize composants vérifiée | **Se code** — Phase 5b, `G5`. Inscrit à `derivation.md` avec les mentions « composé » et « à valider à l'atelier terrain » (T035) |
| Édition d'une formule (formulaire) | Couvert par le motif `G2` — « Configuration », dont `G1` hérite déjà au cycle 002 | Codé comme panneau du motif, **pas** comme écran nouveau |
| Statut d'unité, calendrier tarifaire | HEB-06, HEB-07 — P1, hors périmètre | Aucune tâche |

---

## Décisions en attente

| # | Question | État | Effet |
|---|---|---|---|
| **T002** | Libellés du choix de règle fiscale | ✅ **Tranché au terrain le 2026-08-02** — « Une seule taxe pour tout le séjour » / « Une taxe par nuit » | T031 et T038 sont débloqués |
| **Axe « par client »** | `une_nuitee_par_occupation` réduit trois nuits à une. **Que fait-elle de trois personnes ?** Le cadrage §9.6 et FIS-03 disent tous deux « par nuitée **et par client** (accompagnants inclus) », et SEJ-02 précise que l'enregistrement des accompagnants « impacte le calcul de la taxe ». Une occupation de 3 nuits à 2 personnes vaut 500 F ou 1 000 F — **aucune source ne le dit** | ⛔ **NON TRANCHÉ, et à ne pas trancher par défaut** | Calcul renvoyé à **FIS-03** (T3) ; l'exonération par personne est inscrite en **B-10** au cadrage, échéance **avant le cycle SEJ** — la colonne de motif irait sur `accompagnant`, table de SEJ. **Aucune tâche ici.** Le calcul devra porter la marque « axe des personnes non résolu » : un multiplicateur posé à l'aveugle se retrouverait sur des factures et dans un état de reversement communal |

> **Le paramètre fiscal n'est pas une incertitude en attente d'arbitrage, c'est une exigence
> produit.** Le cadrage §9.6 écrit « hors Abidjan **variable selon la collectivité** » : les règles
> varient par collectivité, donc le paramètre doit exister quoi qu'il arrive. **B-02 ne décidera
> pas s'il faut un paramètre — B-02 décidera de sa valeur par défaut légale.** Aucun code, aucun
> test et aucun commentaire ne doit traiter `regle_conversion_taxe` comme une constante provisoire.

> **Les deux libellés retenus ne disent rien des personnes**, et c'est précisément ce qui les rend
> employables aujourd'hui : ils tranchent l'axe des nuits sans préjuger de l'axe non résolu.

---

## Dépendances entre stories

```
Phase 1 (docs)  ──►  Phase 2 (socle)  ──►  US1 (référentiel)  ──►  US2 (disponibilité)
                                              │                        │
                                              ▼                        ├──►  US3 (passage)
                            Phase 5 (écran G2) + Phase 5b (écran G5)   └──►  US4 (demi-journée)
                                                                             │
                                                            Phase 8 (seeds, recollement, DoD)
```

**US2 ne peut pas précéder US1** : `occupation` référence `unite` et `formule`, et la contrainte
d'exclusion se pose à la création. **US3 et US4 sont parallélisables** entre elles une fois US2
livrée.

## Parallélisation

| Ensemble | Tâches |
|---|---|
| Fondations documentaires | T003, T004 |
| Tests transverses, après US2 | T026, T027 |
| Recollement, après T047 | T048, T050, T051, T055 |

**Jamais parallélisable** : T018 (indivisible), T049 (suppose tout le reste fini), T054 (deux
passes séquentielles), T057 (revue finale).

## Stratégie de livraison

| Incrément | Contenu | Ce qu'il vaut |
|---|---|---|
| **MVP** | Phases 1 à 3 + Phases 5 et 5b | Adjoua voit et corrige son offre, **et son parc de chambres**. Deux écrans réels, livrables |
| **Cœur** | + US2 | La double attribution devient impossible. **C'est le cycle** |
| **Complet** | + US3, US4, Phase 8 | Le moteur de tarification et la demi-journée |

**Note sur les priorités** : la consigne demande de placer les tâches **P1 en fin de liste**. Ce
cycle n'en produit **aucune** — HEB-01 à HEB-05 sont **toutes P0**, et HEB-06/HEB-07, les deux
seules stories P1 du module, sont hors périmètre. Aucune tâche P1 n'a donc été fabriquée pour
respecter la forme. Les priorités `P1`–`P3` des user stories ci-dessus sont celles du **template**
— un ordre de livraison —, à ne pas confondre avec les priorités `P0`/`P1` du projet.
