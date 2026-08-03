---

description: "Tâches — Cycle 006 · Fiches clients, arrivée, départ et prolongation"
---

# Tasks: Fiches clients, arrivée, départ et prolongation

**Input**: Documents de conception de `specs/006-clients-sejours-enregistrement/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), [contracts/](contracts/), [quickstart.md](quickstart.md)

**Tests**: **obligatoires, non optionnels.** La Definition of Done les impose (§0.4, points 1, 4, 5),
les portes P-03, P-04, P-05, P-07, P-08, P-09, P-13, P-14, P-23 n'existent que sous forme de tests,
et **deux garanties de ce cycle décrivent des défauts qui ne se voient pas en relecture** : une
transaction de check-in qui aurait contourné la contrainte d'exclusion par une lecture préalable, et
un constat de taxe qu'un `UPDATE` pourrait recalculer.

**Organisation** : par story, réordonnées par dépendance. Trois écarts assumés, écrits ici plutôt
que subis :

- **US2 (fiche client) précède US1 (passage)** bien que US1 soit la story qui décide du produit :
  le pré-remplissage d'US3 en dépend, et la recherche est la seule story **entièrement testable sans
  aucun séjour**.
- **US1 porte le cœur du séjour** — `sejour`, `note_sejour`, `ligne_sejour`, `fiche_police` — que
  US3 et US4 réutilisent. Le passage est le parcours le plus court : c'est celui qui prouve que le
  cœur tient.
- **Les écrans suivent leur story**, sauf `R5` (fiche client) qui **hérite de `R7`** : il attend que
  le motif de `R7` soit implémenté, sans quoi il l'inventerait puis divergerait.

## Format: `[ID] [P?] [Story] Description`

- **[P]** : parallélisable — fichiers distincts, aucune dépendance sur une tâche inachevée
- **[Story]** : US1 à US7, telles que numérotées dans [spec.md](spec.md)
- Chaque tâche porte ses chemins de fichiers exacts

## Conventions de chemin

Monorepo des cycles 001 à 005 : `backend/migrations/`, `backend/crates/socle/comptes/src/`,
`backend/crates/verticales/hebergement/src/`, `backend/api/src/`, `backend/tests/`, `app/`,
`tests-e2e/`, `scripts/ci/`, `docs/`.

---

## ⚠️ Il n'y a AUCUNE tâche P1 dans ce cycle, et c'est un constat, pas un oubli

La consigne demande de placer les tâches **P1** en fin de liste. **Les trois stories du périmètre —
SEJ-01, SEJ-02, SEJ-04 — sont toutes P0** (`docs/user-stories-v1.md`, module SEJ). Les seules
stories P1 du module sont **SEJ-06** (OCR, tranche T4), hors périmètre.

Aucune section « P1 » n'est donc fabriquée. En créer une pour respecter la forme donnerait à croire
qu'une partie du cycle est différable, alors que **tout ce qui est ici est bloquant pour la démo de
fin de tranche T1**.

> **Ne pas confondre deux échelles.** `spec.md` numérote ses stories P1/P2/P3 selon le gabarit Spec
> Kit — *ordre d'importance interne au cycle*. `docs/user-stories-v1.md` §0.2 numérote P0/P1/P2 —
> *ce sans quoi on ne livre pas l'incrément*. Les stories P2 et P3 de `spec.md` (US5 à US7) sont
> **toutes P0 au sens du corpus** : SEJ-04 les exige nommément.

---

## Note d'ordonnancement — les six verrous du cycle

**1 · Le lexique avant le code, et les amendements avant les migrations.** Les routes du cycle
(`/passage`, `/arrivee`, `/clients`, `/depart`) sont **visibles** : la leçon `S1` du cycle 005 est
que « le nom du fichier de page décide de la route, et une URL est visible ». Et la migration `0034`
recopie un paramétrage fiscal : l'écrire pendant que trois sources de rang supérieur disent
l'inverse produirait le désaccord dans le même changement.

**2 · `0029 → 0030 → 0031 → 0032 → 0033 → 0034`, dans cet ordre.** `sqlx` **refuse une version
antérieure à une version déjà appliquée** — le cycle 001 s'y est heurté, et l'en-tête de
`0006_provisions_comptables.sql` le consigne. Les numéros suivent l'ordre des tâches, pas l'ordre
thématique.

**3 · P-09 est ré-exercée dans la MÊME tâche que l'`ALTER TABLE occupation`.** L'ajout de
`sejour_id` ne touche pas la contrainte d'exclusion — mais la constitution exige de re-exercer une
porte dont le périmètre s'étend, et une migration qui recréerait la table la perdrait sans que rien
ne le dise. Séparer les deux laisserait un soir où l'on ne saurait pas si le vert est mérité.

**4 · `classes_offline.rs` doit connaître les neuf tables DÈS la première migration.** Son
`PLANCHER_TABLES` passe de 35 à 44. Sans cet ajustement, les tables du cycle échappent au balayage —
exactement le trou trouvé sur `comptes` au cycle 003 et sur `hebergement` au cycle 004.

**5 · Le budget de gestes se mesure sur l'écran, pas sur l'API.** `app/tests/budget-gestes.spec.ts`
est écrit **avec** `EcranPassage.vue`, dans la même tâche. Écrit après, il constaterait le nombre de
gestes au lieu de le contraindre.

**6 · `couverture_portes.rs` en dernier.** Il suppose que toutes les migrations, toutes les
opérations et tous les événements existent ; le lancer plus tôt compterait juste et couvrirait faux.

---

## Note sur la référence visuelle — TROIS cas, et ce cycle en emploie DEUX

Le décompte se lit dans `docs/design/derivation.md`, jamais dans une consigne. **Quatre écrans**,
répartis en deux cas — **aucun écran composé, aucun écran hors des trois cas** :

| Écran | Cas | Référence exacte |
|---|---|---|
| `R4` Le passage | **(a) MAQUETTÉ** | `docs/design/html/R4-passage.html` + 4 états : `-connu`, `-enregistre`, `-hors-ligne`, `-complet` |
| `R7` La note et le départ | **(a) MAQUETTÉ** | `docs/design/html/R7-note-depart.html` — **état nominal SEUL**, voir ci-dessous |
| `R3` Arrivée | **(b) DÉRIVÉ** | `derivation.md` ligne `R3` : *« hérite de `R4` — parcours long : plus de champs, même grammaire »* |
| `R5` Fiche client et recherche | **(b) DÉRIVÉ** | `derivation.md` ligne `R5` : *« hérite de `R7` — liste + fiche, pas de total »* |

**Aucun écran de ce cycle n'est composé, et c'est vérifié plutôt que supposé.** `R4` et `R3` sont en
**zone de vitesse** — Yao est debout, pressé, un client en face — et `docs/Kaya_Design.md` §1 est
formel : *un écran de zone de vitesse ne se compose jamais*. `R4` porte précisément l'intention
dessinée qu'un assemblage ne retrouverait pas (les tailles des durées et de l'heure de fin). `R7` et
`R5` sont maquetté et dérivé : la question ne se pose pas.

### ⚠️ Les deux états de `R7` qui ne sont PAS de ce cycle

`R7-note-depart-envoi.html` (« Les impôts sont en train de valider ») et
`R7-note-depart-echec.html` (« Les impôts ont refusé la facture ») sont des états de **certification
FNE** — **FIS-05, tranche T3**. Ils ne se codent pas ici. **Seul l'état nominal** est la référence.

### ⚠️ Quatre éléments des maquettes appartiennent à d'autres cycles

Ils sont **absents** de l'implémentation, jamais grisés (principe VII), et la tâche d'écran le dit :

| Élément de maquette | Où | Cycle qui le doit |
|---|---|---|
| Bouton « **Scanner la pièce** » | `R4`, tous états | **SEJ-06** — OCR, P1, tranche T4. Seul « Saisir · téléphone » est livré |
| « **Garder la 101 pour ce client** » | `R4-passage-complet` | **RSV** — maintien d'unité, tranche T4 |
| « **Imprimer le reçu** », « encaissé en espèces » | `R4-passage-enregistre` | **IMP** (T2) et **CAI** (T2) |
| « Déjà versé 100 000 F », « Il resterait à payer », « compte final, encaissement, facture » | `R7` nominal | **CAI** (T2) et **FIS** (T3) |

**Le HTML de maquette n'est jamais copié ni déplacé vers `app/`** (porte P-19) : on en lit les
valeurs et la structure, on réimplémente en composants Nuxt avec i18n, mode sombre, RBAC et
chargement paresseux — que l'export ne contient pas. **Chaque écran est vérifié en clair ET en
sombre.**

---

## Phase 1 : Fondations documentaires (bloquantes)

**Objectif** : aucun mot ni aucune règle n'atteint le code avant d'être au bon document. Deux de ces
tâches sont bloquantes pour les migrations, pas seulement pour la revue.

- [X] T001 Porter `docs/design/lexique.md` à la **version 1.6.0** avec le vocabulaire du cycle, `fr` **et** `en`, section « Concept interne → ce qu'affiche l'interface ». Entrées dues : `sejour` → « **Séjour** » / *Stay* · check-in → « **Arrivée** » / *Arrival* · check-out → « **Départ** » / *Departure* · `client` → « **Client** » / *Guest* · `accompagnant` → « **Accompagnant** » / *Additional guest* · `fiche_police` → « **Fiche de police** » / *Police registration form* (le terme est celui de l'usage ivoirien, il reste) · note arrêtée → « **La note est arrêtée : plus rien ne peut s'y ajouter** » (chaîne exacte de `R7-note-depart-envoi.html`). **Mots écartés nommément** : « check-in », « check-out », « occupation », « constat », « assiette », « figeage » — aucun n'atteint l'interface ni une **route**. Les routes retenues sont `/passage`, `/arrivee`, `/clients`, `/depart` : « une URL est visible » (leçon `S1`, cycle 005).
- [X] T002 Inscrire à `docs/design/lexique.md` les **six refus** du cycle, en `fr` et `en` : `sejour_deja_clos` → « Ce séjour est déjà terminé » · `sejour_clos` (prolongation) → « On ne prolonge pas un séjour terminé » · `conflit_occupation_suivante` → « Cette chambre est réservée à partir de {heure} » · `unite_cible_occupee` → « Cette chambre n'est pas libre sur la période restante » · `bascule_formule_non_confirmee` → « Au-delà de {n} h, le tarif passe à la nuitée » · **écriture orpheline (`202`)** → « Cette information est arrivée après le départ du client. » suivie de « Le gérant décidera de la suite. ». ⚠️ **Les deux dernières sont à valider à l'atelier terrain** et portent la mention : la première annonce un changement de tarif avant confirmation, la seconde décrit une situation qu'aucun exploitant n'a encore vue.
- [X] T003 Amender `docs/cadrage-v1.md` **§9.6** et **annexe B, ligne B-10** : la taxe communale de nuitée est due **par nuitée et par séjour**, jamais par personne (arbitrage terrain du 2026-08-03). B-10 passe de « ouverte » à **close**, avec sa date. **Bloquant pour la migration `0034`** : celle-ci recopie un paramétrage fiscal, et l'écrire pendant que le cadrage dit l'inverse produirait le désaccord dans le même changement.
- [X] T004 Amender `docs/user-stories-v1.md` : **FIS-03** (« par nuitée et par client, accompagnants inclus » → par séjour), **FIS-08** (« nombre de clients » → **séjours assujettis**, le nombre de personnes restant indicatif), et le **récapitulatif des paramètres** — dont la référence « (B-02) » est **erronée** : la décision est **B-10**. Bloquant au même titre que T003.
- [X] T005 [P] Trancher **O-01** au `§14` de `docs/registre-classes-offline.md` : **option (a)**, `client` reste en classe **C**. Écrire la friction résiduelle plutôt que la taire — en mode nœud de site (incrément 3), une arrivée sera possible hors ligne alors qu'une **fiche nouvelle** ne le sera pas ; au MVP la question est sans effet visible, l'arrivée étant elle-même de classe B.
- [X] T006 [P] Déclarer au `§8` de `docs/registre-classes-offline.md` les **quatre tables que le registre ne nomme pas** : `preference_personne` (**A**, branche A4 — le registre écrit « `client.preferences` » sans nom de table), `note_sejour` (**B**, B3 — il nommait `ligne_sejour`, pas la note), `numerotation_fiche_police` (**B**, B3) et `taxe_sejour_constat` (**B**, B3 — il parlait de « `sejour` — check-out, taxe figée »). Les cinq entités déjà déclarées — `client`, `sejour`, `accompagnant`, `ligne_sejour`, `fiche_police` — sont **honorées, pas réécrites**. Entrée au **journal §13, version 1.4.0**, dans le même changement. ⚠️ **Le nom de table est `taxe_sejour_constat`, jamais `assiette_taxe_sejour_figee`** : la spécification emploie le second dans ses « Key Entities », le plan a retenu le premier (R-08 — ce cycle fige un **constat**, il ne dérive aucune assiette). `classes_offline.rs` compare des **noms de table** au registre : y déclarer l'ancien nom ferait échouer le build sans dire pourquoi.
- [X] T007 [P] Consigner dans `docs/user-stories-v1.md`, **TRX-06**, que la rétention de 90 jours du numéro de pièce portera sur **deux tables** — `comptes.personne` **et** `hebergement.accompagnant`. Découvert à la conception : un accompagnant n'a pas de fiche client, lui en créer une ferait entrer au fichier des personnes qui n'ont rien demandé. Sans cette ligne, la purge de TRX-06 en oubliera une.

**Point de contrôle** : le vocabulaire, les classes et les décisions sont écrits. Le code peut commencer.

---

## Phase 2 : Socle du cycle (bloquant pour toutes les stories)

- [X] T008 Créer `backend/migrations/0029_client_et_preferences.sql` : `ALTER TABLE comptes.personne` — colonnes `nom_repli`, `telephone_repli`, `numero_piece_repli`, `piece_capturee_le` ; les **trois index** dont `personne_nom_repli_idx (tenant_id, nom_repli text_pattern_ops)` — ⚠️ **`text_pattern_ops` n'est pas décoratif** : sans lui un `LIKE 'kouam%'` n'emploie pas l'index dès que la collation n'est pas `C`. Puis `CREATE TABLE comptes.client` (clé primaire = `personne_id`, clé étrangère **intra-schéma** légale) et `comptes.preference_personne` (append-only). **RLS `ENABLE` + `FORCE` + politique `isolation_tenant` sur les deux tables neuves.** Privilèges : `SELECT, INSERT, UPDATE` sur `client` ; **`SELECT, INSERT` seuls** sur `preference_personne` — classe A append-only, patron de `note_etablissement`. Mettre à jour le commentaire des colonnes de pièce par `COMMENT ON COLUMN`. ⚠️ **Ne pas toucher `0015`** (porte P-02) : son commentaire décrit l'état du cycle 003 et reste vrai de ce cycle-là.
- [X] T009 Créer `backend/migrations/0030_permissions_sejours.sql` : les **sept permissions** de [data-model.md](data-model.md) — `sej.client.lire` et `sej.client.gerer` **transversales** (`module_code = NULL`, car SEJ-05 en aura besoin sans module hébergement), les cinq `heb.sejour.*` sur `HEBERGEMENT`. Attribution : `receptionniste` et `gerant` reçoivent les sept, `proprietaire` les deux lectures. ⚠️ **Chaque permission doit garder une opération réellement servie par ce cycle** — la règle du cycle 003 refuse une permission sans contrepartie, et `couverture_portes.rs` la vérifie.
- [X] T010 Étendre `scripts/ci/jointures-inter-schemas.sh` à la paire **`comptes` × `hebergement`**, et vérifier que son **décompte de requêtes analysées par schéma** la reprend (porte **P-04**, exigence de périmètre déclaré). C'est le premier cycle où deux schémas se parlent sur le chemin chaud : sans cet ajout, la porte serait verte en ne regardant pas.
- [X] T011 Relever **les DEUX planchers**, pas un seul : `PLANCHER_TABLES` de **35 à 44** dans `backend/tests/classes_offline.rs` **et** dans `backend/tests/rls_catalogue.rs`. ⚠️ **Les deux fichiers portent une constante homonyme et indépendante.** Tous deux découvrent leurs schémas par `commun::perimetre::schemas_applicatifs()` — les neuf tables entrent donc automatiquement au balayage —, mais un plancher laissé à 35 rendrait **P-07 verte en inspectant moins de tables qu'attendu** : le mode d'échec exact que la section « Couverture des portes » nomme, une porte dont la complétude n'est pas vérifiée. Vérifier aussi que `REFERENTIELS_GLOBAUX` de `rls_catalogue.rs` **n'est pas touché** : les neuf tables portent toutes `tenant_id`, aucune n'est un référentiel global.
- [X] T012 [P] Écrire `backend/crates/socle/comptes/src/client/repli.rs` : la fonction `repli(&str) -> String` — minuscules, suppression des signes diacritiques latins **par table de correspondance écrite à la main**, suppression des apostrophes **droite (U+0027) et typographique (U+2019)**, réduction des espaces et traits d'union. **Aucune dépendance nouvelle** : `unicode-normalization` n'est pas au gel, et l'ajouter imposerait une décision de revue mensuelle pour ce que soixante correspondances couvrent. Test unitaire sur le jeu nommé : `Kouamé`, `N'Guessan`, `N’Guessan`, `Aïcha`, `Traoré`, `Koffi`, `Yao`, `Bakayoko`, `Adjoua`, `Éboué`, `Gbagbo`, `Ouattara`.
- [X] T013 [P] Créer la structure du module `backend/crates/socle/comptes/src/client/` — `modele.rs`, `repository.rs`, `service.rs`, `mod.rs` — et l'enregistrer dans `lib.rs`. Les types `FicheClient`, `ClientResume`, `ErreurClient` avec ses codes stables. **Aucune dépendance nouvelle au `Cargo.toml`.**
- [ ] T014 Créer la structure des modules `backend/crates/verticales/hebergement/src/{sejour,note,police,taxe}/` — `modele.rs`, `repository.rs`, `service.rs`, `mod.rs` chacun — et les enregistrer dans `lib.rs`. Étendre `erreurs.rs` : `ErreurSejour` avec ses codes stables (`sejour_deja_clos`, `conflit_occupation_suivante`, `unite_cible_occupee`, `bascule_formule_non_confirmee`, `sejour_inconnu`). **Réemployer `est_violation_exclusion` du cycle 004**, ne pas la réécrire.

**Point de contrôle** : schémas, permissions, portes et squelettes prêts. Les stories peuvent commencer.

---

## Phase 3 : US2 — La fiche client, trouvée en un souffle (P1)

**Objectif** : Yao trouve une fiche par nom, téléphone ou numéro de pièce, en moins de 300 ms sur
10 000 fiches, et le personnel n'y apparaît jamais.

**Test indépendant** : entièrement testable **sans aucun séjour**. On charge le jeu de mesure, on
lance les trois formes, on mesure.

- [X] T015 [US2] Écrire `backend/crates/socle/comptes/src/client/repository.rs` : création et modification d'une fiche — `INSERT` sur `comptes.personne` **et** `comptes.client` dans **la même transaction**, avec calcul des trois colonnes repliées et de `piece_capturee_le`. L'identifiant est l'**UUID v7 fourni par le client** (FR-086) : c'est lui, et non une clé engendrée côté serveur, qui rend le rejeu inoffensif. ⚠️ **sqlx 0.9** : `AssertSqlSafe` sur toute requête non littérale ; le patron est `docs/module-dore.md`, **pas un extrait trouvé en ligne** qui viserait 0.8. Insertion **idempotente** qui renseigne l'appelant (`Issue::Creee` / `Issue::Rejeu`).
- [X] T016 [US2] Écrire la recherche dans `repository.rs` : **une seule requête** joignant `personne` et `client` — jointure **intra-schéma**, donc légale. Trois formes : préfixe sur `nom_repli`, suffixe d'au moins six chiffres sur `telephone_repli`, égalité sur `numero_piece_repli`. La forme est déduite de la saisie ; une saisie ambiguë interroge **les trois** et fusionne. Rendre `tronque: bool` — une liste silencieusement coupée est un mensonge sur un écran de comptoir. ⚠️ **Les deux seuils de cette tâche — longueur minimale du suffixe téléphonique et `limite` par défaut — sont des constantes nommées et commentées dans le fichier, jamais des littéraux dans la requête.** Ce ne sont **pas** des paramètres d'établissement (aucune story du périmètre ne dit « paramétrable », principe I·c), mais ils décident du comportement au comptoir : anonymes, leur révision serait introuvable.
- [X] T017 [US2] Écrire `service.rs` : garde de permission, normalisation du téléphone avec `indicatif_telephonique_defaut` lu par la configuration héritée (CPT-01), validation au champ, et **émission des événements** `sej.client.cree` / `sej.client.modifie` **dans la transaction** (`OutboxWriter::ecrire(&mut tx, …)`). ⚠️ **Aucun numéro de pièce dans la charge utile** : l'outbox est un grand livre à rétention **illimitée** et immuable — une donnée sensible qui y entre ne peut jamais en sortir, et la rétention de 90 jours de TRX-06 deviendrait inapplicable.
- [X] T018 [US2] Ajouter à `backend/crates/socle/comptes/src/client/service.rs` le service de préférence (classe **A**, append-only) : `INSERT` seul, `horodatage_client` accepté et **indicatif** (P-23 — écrire la colonne n'est pas s'appuyer dessus), événement `sej.preference.enregistree`. La préférence courante est **la ligne la plus récente**, jamais une colonne mise à jour.
- [X] T018a [US2] ★ **Protéger le numéro de pièce d'identité au repos et journaliser ses lectures** (FR-012, principe IX, cadrage §12.1). ⚠️ **Cette tâche n'existait pas au premier découpage, et son absence était le seul défaut critique du cycle** : le plan écrit que la donnée est « protégée au repos et son accès journalisé, **dès ce cycle** — la donnée naît ici », et aucune tâche ne le faisait. Portée : `comptes.personne.numero_piece` **et** `hebergement.accompagnant.numero_piece` — **deux** tables, découvert à la conception (réévaluation de Phase 1, point a). Écrire dans `backend/crates/socle/comptes/src/client/` le chiffrement au repos par le coffre par tenant existant (cadrage §12.1), et la journalisation de **toute lecture** au registre des actions — famille `suppression` exclue, c'est une **consultation**, donc une entrée dédiée à inscrire à `docs/taxonomie-audit.md` dans le même changement. Test : une lecture de fiche laisse une trace ; la colonne n'est jamais lisible en clair par une requête directe sous le rôle applicatif. **Ne pas repousser à TRX-06** : TRX-06 (P1) apporte l'export, la suppression et la purge paramétrable, **pas la protection**.
- [X] T019 [US2] Créer `backend/api/src/routes/clients.rs` : les opérations **1 à 4 et 6** de [contracts/http-api.md](contracts/http-api.md) — `client_rechercher`, `client_creer`, `client_lire`, `client_modifier`, `client_preference_enregistrer`. ⚠️ **L'opération 5 (`client_historique_sejours`) n'est PAS ici** : elle lit `hebergement.sejour`, qui n'existe qu'en Phase 4, et elle se monte sur le crate `hebergement` — voir **T030a** et la note de fin de phase. Monter par `service(...)`, **jamais `route(...)`**, et **du plus spécifique au plus général** dans `routes/mod.rs`. `200` sur rejeu, jamais `409`. **Terminer par** : annotations `#[utoipa::path]` à jour, `pnpm generer:client`, **diff commité**, `cargo build` vert.
- [X] T020 [US2] Instancier les tests de classe dans `backend/tests/` : `tester_classe_bcd!(client, classe = C, …)` et `tester_classe_a!(preference_personne, schema = "comptes", table = "preference_personne", …)`. ⚠️ **`outillage_classes.rs` échoue en NOMMANT** l'entité qui aurait une table sans instanciation — ne pas recopier les tests à la main, la macro les engendre nommés.
- [X] T021 [US2] Écrire `backend/tests/client_recherche.rs` : les trois formes · le repli sur le jeu de noms de T012 · apostrophe droite **et** typographique · téléphone avec et sans indicatif · numéro de pièce avec espaces et tirets · **une personne non qualifiée cliente n'apparaît pas** · et la mesure — **10 000 fiches générées par le test**, cent recherches par forme, **95ᵉ centile < 300 ms côté serveur**. ⚠️ Le jeu de mesure vit dans un tenant dédié et **n'est jamais chargé dans les tenants de démonstration** (FR-007).
- [X] T022 [US2] Étendre `backend/tests/isolation_tenant.rs` aux six opérations de clients (porte **P-08**), et `backend/tests/rls_catalogue.rs` aux deux tables neuves (porte **P-07**).

**Point de contrôle** : la recherche et la fiche sont livrables et démontrables par API.

> ⚠️ **US2 se livre en DEUX temps, et c'est une dépendance, pas un choix.** Son scénario 5 —
> l'historique des séjours — ne peut pas exister avant les séjours : il est en **T030a**, fin de
> Phase 4. Son écran `R5` hérite de `R7` et vient en Phase 10. Écrire ici que « US2 est livrable »
> serait faux, et l'écrire faux ferait cocher une story à moitié faite.

---

## Phase 4 : US1 — Le passage en deux gestes (P1) 🎯 MVP

**Objectif** : Yao touche une durée, touche une chambre, donne la clé. Le séjour, la note et la
fiche de police existent, **en une transaction et un appel**.

**Test indépendant** : un établissement, une catégorie, un barème, trois chambres libres — tout
livré par le cycle 004. On compte les gestes, on chronomètre, on éprouve la concurrence.

- [ ] T023 [US1] Créer `backend/migrations/0031_sejour.sql` : `CREATE TABLE hebergement.sejour` (avec `client_id UUID NULL` **sans `REFERENCES`** — clé étrangère inter-schémas interdite, principe II ; `NULL` légal car un passage s'enregistre sans fiche) et `hebergement.accompagnant`. Puis `ALTER TABLE hebergement.occupation ADD COLUMN sejour_id UUID NULL REFERENCES hebergement.sejour (id)` + index partiel. Puis `GRANT INSERT ON synchronisation.reconciliation_orpheline TO kaya_app` — **`UPDATE` n'est PAS accordé**, la résolution est SYN-03 (T3). **RLS `ENABLE` + `FORCE` + politique sur les deux tables neuves**, aucun `DELETE` nulle part.
- [ ] T024 [US1] **Ré-exercer la porte P-09 dans le même changement que T023.** Étendre `backend/tests/hebergement_disponibilite.rs` : après la migration, (1) le type de `periode` est toujours `tstzrange`, (2) `occupation_sans_chevauchement` existe toujours avec ses deux opérateurs, (3) la contrainte se déclenche encore. ⚠️ *« La couverture s'étend avec les fonctionnalités : elle doit être re-exercée, pas supposée acquise »* — exigence 5 de la section « Couverture des portes ». Séparer T023 et T024 laisserait un soir où l'on ne saurait pas si le vert est mérité.
- [ ] T025 [US1] Créer `backend/migrations/0032_note_sejour.sql` : `note_sejour` (**aucune colonne de total** — le total est la somme des lignes ; une colonne totalisatrice se désynchronise en silence) et `ligne_sejour` — ⚠️ `quantite NUMERIC(14,4)`, **jamais entier** ; `prix_unitaire_mineur` et `montant_mineur` en `BIGINT`, **le second pouvant être négatif** (un départ anticipé rembourse) ; `libelle_cle` et non un libellé rendu, la note s'affichant en `fr` **et** `en`. Privilèges : **`SELECT, INSERT` seuls sur `ligne_sejour`** — le prix verrouillé à la ligne devient impossible à modifier. RLS sur les deux.
- [ ] T026 [US1] Créer `backend/migrations/0033_fiche_police.sql` : `numerotation_fiche_police (tenant_id, etablissement_id, dernier_numero)` — ⚠️ **un compteur, pas une `SEQUENCE`** : une séquence est globale au schéma et laisse des trous, deux propriétés fatales à une numérotation continue **par établissement** ; c'est le défaut corrigé par `0012` au cycle 002. Puis `fiche_police` avec `UNIQUE (tenant_id, etablissement_id, numero)` et le drapeau `complete`. **Aucune identité n'y est recopiée** : elle référence le séjour, les identités viennent du client et des accompagnants — recopier créerait une troisième surface de rétention. RLS sur les deux.
- [ ] T027 [US1] Écrire `backend/crates/socle/comptes/src/traits.rs` — le trait **`AnnuaireClients`** et son implémentation `PgAnnuaireClients`. ⚠️ **`resumes(&[Uuid])`, jamais `resume(Uuid)`** : une signature unitaire produirait N+1 requêtes sur la liste des séjours, et c'est le détail qui décide si l'écran de départ s'ouvre en 200 ms ou en deux secondes. `ClientResume` **ne porte aucun numéro de pièce** — il porte `piece_enregistree: bool`, ce dont la fiche de police a besoin sans lire la pièce. Voir [contracts/traits-exposes.md](contracts/traits-exposes.md).
- [X] T028 [US1] Écrire les repositories `sejour/repository.rs`, `note/repository.rs`, `police/repository.rs`. Chacun **prend la transaction, ne l'ouvre pas** (module doré, couche 3). La numérotation se fait par `UPDATE … RETURNING dernier_numero` **dans la transaction** : le verrou de ligne est ce qui sérialise, et c'est la définition même de la classe **B**.
- [X] T029 [US1] ★ Écrire `sejour/service.rs`, méthode `ouvrir` — **une transaction, cinq écritures, dans cet ordre** : (1) `MoteurDisponibilite::attribuer(&mut tx, …)` du cycle 004 — **tenter l'insertion et traduire la violation, jamais lire d'abord pour décider** ; (2) `INSERT sejour` et pose de `sejour_id` sur l'occupation ; (3) `INSERT note_sejour` + sa **ligne d'hébergement** au tarif du `MoteurTarification` ; (4) numérotation puis `INSERT fiche_police`, `complete = false` sans client rattaché ; (5) `OutboxWriter::ecrire(&mut tx, heb.sejour.ouvert)` et `heb.fiche_police.generee`. Garde préalable : établissement existant **et** module `HEBERGEMENT` actif. L'identifiant du séjour est l'**UUID v7 fourni par le client** (FR-086) — le serveur déduplique, il n'engendre pas.
- [ ] T030 [US1] Créer `backend/api/src/routes/sejours.rs` : les opérations **7, 8, 9, 10** — `sejour_ouvrir`, `sejour_lister`, `sejour_lire`, `sejour_rattacher_client`. `201` à la création, **`200` sur rejeu du même `id`**, `409 unite_deja_occupee` pour un conflit réel — ⚠️ **les deux `409` ne se confondent pas**, la distinction est celle du cycle 004. Monter dans `routes/mod.rs` **du plus spécifique au plus général**.
- [ ] T031 [US1] Ajouter l'opération **17** `hebergement_etat_des_unites` dans `backend/api/src/routes/hebergement_disponibilite.rs` : toutes les unités avec leur **état d'occupation dérivé** (`libre` · `occupee` avec `fin_prevue` · `remise_en_etat` avec `disponible_a`), `statut_menage` **en lecture seule**, et `instant_autorite`. ⚠️ **Ce n'est pas HEB-06** : le sous-statut ménage n'est pas modifiable ici, et l'état d'occupation est **dérivé**, jamais posé à la main (principe IV).
- [ ] T030a [US2] ★ Servir l'opération **5** `client_historique_sejours` — `GET /api/v1/clients/{client_id}/sejours`. ⚠️ **Elle est ici, et non en Phase 3, pour deux raisons opposables** : elle lit `hebergement.sejour`, créée en T023 ; et elle se monte **sur le crate `hebergement`**, jamais sur `socle/comptes`. Si `comptes` lisait `hebergement.sejour`, ce serait **deux violations d'un coup** — jointure inter-schémas (**P-04**) *et* arête `socle/ → verticales/` (**P-03**). Le chemin HTTP cache ce découpage à l'appelant, et c'est normal : le contrat est une façade, pas une carte des crates. Requête sur `sejour_par_client_idx`, du plus récent au plus ancien, avec établissement, unité, période et formule. Double garde de permission : `sej.client.lire` **et** `heb.sejour.lire`. **Terminer par** : utoipa, `pnpm generer:client`, diff commité, build vert. Étendre `backend/tests/isolation_tenant.rs` (**P-08**) à cette opération.
- [ ] T032 [US1] **Terminer la part API d'US1** : annotations `#[utoipa::path]` sur les cinq opérations, `pnpm generer:client`, **diff commité**, `cargo build` vert. Vérifier que les cinq `operationId` sont **uniques** (porte P-01b).
- [ ] T033 [US1] Instancier `tester_classe_bcd!` sur `sejour`, `note_sejour`, `ligne_sejour`, `fiche_police`, `numerotation_fiche_police` (classe **B**), et vérifier que `outillage_classes.rs` ne nomme plus aucune entité manquante.
- [ ] T034 [US1] ★ Écrire `backend/tests/sejour_arrivee.rs` : (a) **transaction unique** — une panne simulée après l'attribution ne laisse ni séjour, ni note, ni fiche de police, **ni occupation orpheline** ; (b) **concurrence** — deux arrivées chevauchantes **par le parcours de séjour**, exactement une réussit, et le refus est un `ExclusionViolation` **sur la contrainte nommée**. ⚠️ **C'est la cause du refus qui est assertée, pas son existence** : un `SELECT … FOR UPDATE`, `SERIALIZABLE` ou un verrou applicatif donneraient le même compte en rendant la double attribution *improbable* au lieu d'*impossible* ; (c) numérotation de fiche de police **continue par établissement**, sans trou ; (d) **passage sans client rattaché** → le séjour est valide, la fiche de police est **numérotée et déclarée incomplète** (`complete = false`), et **aucun champ de remplissage n'y figure** (FR-047) — ni fabriquée, ni silencieusement omise ; le rattachement ultérieur (opération 10) la passe à `complete = true` **sans rouvrir le séjour ni remettre en cause l'attribution** (FR-028).
- [ ] T035 [US1] Écrire `backend/tests/sejour_hors_ligne.rs` : porte **P-13** sur les **17** opérations du cycle. Les **deux** de classe A (ajout et retrait d'accompagnant) sont **nommées comme telles**, jamais omises — un test qui n'inspecterait que les opérations refusées ne prouverait pas qu'il les a toutes vues.
- [ ] T036 [US1] Étendre `backend/tests/outbox_transactionnel.rs` aux événements `heb.sejour.ouvert` et `heb.fiche_police.generee` : émis **dans** la transaction (porte P-05), charge utile **financière complète et dénormalisée** avec `montant_mineur` en **entier** (porte P-10, jusque dans le JSONB), et **aucun numéro de pièce**.

**Point de contrôle** : le cœur du séjour tient. `POST /sejours` est démontrable, la double attribution impossible.

---

## Phase 5 : Écran `R4` Le passage — MAQUETTÉ (sert US1)

**Référence** : `docs/design/html/R4-passage.html` et ses **quatre états** —
`R4-passage-connu.html`, `R4-passage-enregistre.html`, `R4-passage-hors-ligne.html`,
`R4-passage-complet.html`. **Zone de vitesse** : cet écran ne se compose jamais.

- [ ] T037 [US1] Créer `app/modules/sejours/donnees.ts` : les **lectures typées** depuis `clients/ts/types.gen.ts`, **jamais un `fetch` écrit à la main, jamais un type redéclaré, jamais un `as unknown as`** (septième couche, module doré — la phrase a été fausse pendant deux cycles avec P-01 verte). Les deux appels de montage : le barème de passage et l'état des unités.
- [ ] T038 [US1] [P] Écrire `app/modules/sejours/ChoixDuree.vue` d'après `R4-passage.html` : les paliers avec **leur prix et leur heure de fin** (« 2 h · 2 800 F · jusqu'à 17 h 30 »), calculés par le moteur de tarification, jamais en dur. ⚠️ **Lire les valeurs de la maquette, ne jamais copier son HTML** (porte P-19) : les tailles de la durée et de l'heure de fin sont l'intention dessinée de l'écran. Tout littéral de couleur ou d'espacement est refusé (P-17) — jetons seuls.
- [ ] T039 [US1] [P] Écrire `app/modules/sejours/GrilleUnites.vue` d'après `R4-passage.html` : la grille de toutes les chambres avec leur état — libre, « Occupée · 16 h 10 », « À nettoyer » — et **l'attribution d'un seul tap**. Les heures gardent l'**espace ordinaire** (`17 h 30`) et ne passent pas par le formateur de montant.
- [ ] T040 [US1] Écrire `app/modules/sejours/EcranPassage.vue` — les **cinq états** de la maquette : nominal · client reconnu (« M. Bakayoko — 7ᵉ passage » + sortie « Ce n'est pas lui ») · enregistré (« C'est fait », l'heure de fin **à redire au client**, « Client suivant ») · hors ligne (durées et prix lisibles avec fraîcheur affichée, **attribution refusée immédiatement et explicitement**, jamais grisée) · complet (nombre de chambres prises + **ce qui se libère avec l'heure**, la plus proche en tête). ⚠️ **Éléments ABSENTS, jamais grisés** : « Scanner la pièce » (SEJ-06, T4), « Garder la 101 » (RSV, T4), « Imprimer le reçu » et « encaissé en espèces » (IMP et CAI, T2).
- [ ] T041 [US1] Écrire `app/modules/sejours/ouvrir-sejour.ts` — l'écriture selon la **septième couche** : appel typé, squelette de chargement, refus métier **en langue utilisateur** via le lexique de T002, action **absente** sans `heb.sejour.ouvrir` (jamais grisée), refus immédiat hors ligne (classe B), rafraîchissement **avant** vidage.
- [ ] T042 [US1] Créer `app/pages/passage.vue` — route `/passage`. ⚠️ **UNE SEULE racine, et c'est un élément** : jamais un `v-if`/`v-else` de premier niveau. Une racine multiple compile en fragment ; un fragment dont la branche active est un `defineAsyncComponent` non résolu a un `el` nul, et Vue lève `Cannot read properties of null` à la navigation **suivante**. Clés i18n `fr` et `en` dans `app/core/i18n/`. **Vérifier en clair ET en sombre.**
- [ ] T043 [US1] ★ Écrire `app/tests/budget-gestes.spec.ts` **dans la même tâche que l'écran** : **exactement deux** interactions obligatoires du premier geste à la confirmation, **zéro** champ de saisie libre obligatoire, **au plus un** appel réseau bloquant. ⚠️ Écrit après l'écran, ce test **constaterait** le nombre de gestes au lieu de le **contraindre**. Puis `tests-e2e/passage.spec.ts` : part machine du parcours sous un budget déclaré, sur Chromium **et** WebKit, budget fixé **très au-dessus** de la valeur observée — un seuil serré rougirait au hasard et serait désactivé dans le mois.

**Point de contrôle** : 🎯 **MVP atteint.** Un passage s'enregistre en deux gestes, mesuré.

---

## Phase 6 : US3 — L'arrivée d'un client connu en moins de 60 s (P1)

**Objectif** : client retrouvé d'un mot, écran rempli tout seul, unité proposée, accompagnants
ajoutés, **aucune information déjà connue retapée**.

- [ ] T044 [US3] Écrire `backend/crates/verticales/hebergement/src/sejour/service.rs`, service d'accompagnant (classe **A**) : ajout et retrait (`retire_le`, jamais un `DELETE` — la fiche de police perdrait la trace d'une personne déclarée), `horodatage_client` **indicatif**, événement `sej.accompagnant.ajoute`. Le **nombre de personnes** du séjour est **dérivé** du titulaire et des accompagnants non retirés, jamais saisi en double.
- [ ] T045 [US3] ★ Traiter le **cas orphelin** dans le même `backend/crates/verticales/hebergement/src/sejour/service.rs` : un ajout sur un séjour **clos** rend **`202`**, écrit une ligne dans `synchronisation.reconciliation_orpheline` et **ne touche pas au séjour**. ⚠️ **`201` serait un ajout d'office, `409` un rejet silencieux — le principe VI interdit les deux.** La charge utile écrite dans la file est du **JSON opaque pour le socle** : `kaya_synchronisation` ne doit connaître ni `Accompagnant` ni `Sejour` (porte P-03).
- [ ] T046 [US3] Ajouter la **proposition automatique d'unité** à `backend/crates/verticales/hebergement/src/sejour/service.rs` : la première unité libre de la catégorie sur l'intervalle, **temps de remise en état inclus**, selon un ordre **stable et explicable** — aucune stratégie d'optimisation de remplissage n'est demandée, et en inventer une rendrait la proposition imprévisible pour l'opérateur. Quand aucune n'est libre, le refus **nomme la première disponibilité ultérieure**, jamais une liste vide.
- [ ] T047 [US3] Ajouter les opérations **11 et 12** à `backend/api/src/routes/sejours.rs` — `sejour_accompagnant_ajouter`, `sejour_accompagnant_retirer` — avec leurs trois codes de réponse (`201`, `200` rejeu, **`202` orphelin**). **Terminer par** : utoipa, `pnpm generer:client`, diff commité, build vert.
- [ ] T048 [US3] Instancier dans `backend/tests/accompagnant_classe_a.rs` la macro `tester_classe_a!(accompagnant, schema = "hebergement", table = "accompagnant", …)` — ⚠️ **P-14 gagne ici sa deuxième cible** ; elle n'en avait qu'une depuis le cycle 001. Le rejeu triple doit vérifier qu'un second envoi **n'émet aucun second événement outbox** : c'est le contrôle qui existait pour `note_etablissement` et qui a été **perdu à la réécriture** sur `occupation`.
- [ ] T049 [US3] ★ Écrire `backend/tests/sejour_orphelin.rs` — **la première cible du scénario orphelin du §0.7 en cinq cycles**, quatre assertions : (1) accompagnant vidé **avant** la clôture → `201` ; (2) vidé **après** → **`202`** ; (3) une ligne existe dans `reconciliation_orpheline` avec le séjour, l'entité, la charge utile et le motif ; (4) **le séjour clos est inchangé**. Mettre à jour `backend/tests/provisions_sans_logique.rs` : le décompte des provisions passe de **six à cinq** — `reconciliation_orpheline` cesse d'en être une.
- [ ] T050 [US3] Écrire `backend/tests/sejour_arrivee.rs`, seconde partie : client connu → **zéro champ ressaisi**, unité proposée **réellement libre** sur l'intervalle, trois personnes comptées après deux accompagnants, catégorie pleine → refus nommant la première disponibilité.

---

## Phase 7 : Écran `R3` Arrivée — DÉRIVÉ (sert US3)

**Référence** : `docs/design/derivation.md`, ligne `R3` — *« hérite de `R4` : parcours long, plus de
champs, même grammaire »*. **Ouvrir `R4-passage.html` et la respecter.**

- [ ] T051 [US3] [P] Écrire `app/modules/sejours/ListeAccompagnants.vue` : ajout d'un accompagnant **avec un nom seul** — demander une pièce par accompagnant coûterait la cible des 60 secondes. Composant **16** (champ de saisie canonique) pour tout champ, sans exception.
- [ ] T052 [US3] Écrire `app/modules/sejours/EcranArrivee.vue` : **la grammaire de `R4` conservée** — le tap reste le geste, les champs s'ajoutent sans devenir un formulaire. Client connu → **tout est pré-rempli** et rien n'est à retaper (FR-035). Heures d'arrivée et de départ standard appliquées d'office et modifiables. `docs/design/derivation.md` est mis à jour dans le même changement : `R3` passe d'« inscrit » à **« codé »**.
- [ ] T053 [US3] Créer `app/pages/arrivee.vue` — route `/arrivee`, racine unique, i18n `fr`/`en`, **clair ET sombre**, entrée **absente** de l'accueil sans module `HEBERGEMENT` (principe VII).

---

## Phase 8 : US4 — Le départ, et la taxe figée (P1)

**Objectif** : la note finale, la taxe **figée** à cet instant et jamais recalculée, la chambre
libérée.

- [ ] T054 [US4] Créer `backend/migrations/0034_taxe_sejour_constat.sql` : les **faits** (`nuits_constatees`, `nombre_personnes`, la période), le **paramétrage recopié** (`formule_id`, `famille_formule`, `assujettie_taxe_nuitee`, `regle_conversion_taxe`, `classement_etablissement`, `commune`), `fige_le`, et les colonnes **posées et jamais alimentées** `nuitees_assujetties`, `montant_mineur`, `devise`. ★ **`GRANT SELECT, INSERT` SEULS** — ni `UPDATE`, ni `DELETE` : le figeage est un **privilège**, pas une intention, et c'est ce qui transforme SC-007 d'une promesse en une propriété de la base. `UNIQUE (sejour_id)` — **un constat par séjour, jamais par occupation**. RLS `ENABLE` + `FORCE` + politique.
- [ ] T055 [US4] ★ Écrire `sejour/service.rs`, méthode `clore` — **une transaction** : (1) durée réelle depuis `now()` de la base et l'instant d'autorité d'ouverture, **jamais l'horloge d'un terminal** (P-23) ; (2) décision du `MoteurTarification` du cycle 004, **rebascule de palier comprise** — ne rien réimplémenter ; (3) `INSERT` d'une **ligne d'ajustement** si la durée réelle diffère, **jamais un `UPDATE`** de la ligne initiale (le privilège le rend d'ailleurs impossible) ; (4) `INSERT taxe_sejour_constat` — les faits **et** le paramétrage lu par `ParametrageFiscalHebergement`, **recopié, jamais interprété** ; (5) arrêt de la note, clôture du séjour, libération de l'occupation à l'instant réel ; (6) registre des actions et `heb.sejour.clos`.
- [ ] T056 [US4] ⚠️ **Vérifier la frontière du principe V pendant l'écriture de T055.** Ce cycle enregistre `nuits_constatees = 3` **et** la règle lue ; il n'écrit **jamais** `nuitees_assujetties = 1`. Compter les nuits d'un intervalle est de l'arithmétique ; décider lesquelles sont assujetties est une **règle fiscale** qui ne vit que dans `JurisdictionAdapter` (P-12). Vérifier que `backend/tests/fixtures/fiscal` **reste vide** et que `portes_a_vide.rs::p11_tests_dores_fiscaux` **reste vert** : sa rougeur signifierait qu'une règle fiscale a été écrite ici.
- [ ] T057 [US4] Étendre `backend/tests/provisions_sans_logique.rs` : `taxe_sejour_constat.montant_mineur` et `nuitees_assujetties` **existent** et **restent vides**, et **aucune opération du contrat ne les expose**. C'est le **versant positif** exigé par la constitution — sans lui, supprimer les colonnes suffirait à passer au vert.
- [ ] T058 [US4] Écrire le trait **`LecteurSejour`** dans `hebergement/src/traits.rs` — `resume`, `ouverts`, `constat_taxe`. **Sans consommateur à ce cycle**, et la justification est écrite à sa définition : SEJ-03 (T2) rattachera une consommation à un séjour, FIS-03 (T3) lira le constat figé ; sans le trait, les deux liraient `hebergement.*` par jointure inter-schémas. *Une alternative qui existe se prend ; une alternative à construire se contourne.* Voir [contracts/traits-exposes.md](contracts/traits-exposes.md).
- [ ] T059 [US4] Ajouter les opérations **15 et 16** à `sejours.rs` — `sejour_clore`, `sejour_fiche_police_lire`. ⚠️ **Le corps de réponse rend `nuitees_assujetties: null` et `montant_mineur: null`**, et c'est visible dans le contrat : zéro laisserait croire que la taxe est nulle, absent qu'elle n'existe pas ; `null` dit ce qui est vrai. **Terminer par** : utoipa, `pnpm generer:client`, diff commité, build vert.
- [ ] T060 [US4] ★ Écrire `backend/tests/sejour_depart.rs` : (a) après clôture, modifier accompagnant, barème, formule, `assujettie_taxe_nuitee`, classement et commune → **aucune valeur du constat ne change** ; (b) **immuabilité par privilège** — sous le rôle applicatif, `UPDATE` et `DELETE` sur `taxe_sejour_constat` échouent en `permission denied` ; (c) note arrêtée refusant toute écriture ; (d) rebascule de palier en **ligne d'ajustement distincte**, tracée au registre des actions avec la durée constatée et les deux paliers ; (e) unité libérée à l'instant réel, temps de remise en état appliqué **à partir de cet instant** ; (f) ★ **aucune clôture automatique** (FR-068) — un séjour dont la période prévue est dépassée reste `en_cours`, et **aucun worker ne le clôt** : une clôture d'office produirait une facturation sans témoin ; (g) ★ **dérive d'horloge** (SC-011) — le même départ, rejoué avec une horloge de terminal décalée de **+1 h puis −1 h**, produit une durée réelle, une ligne d'ajustement et un constat **identiques au bit près**. ⚠️ (b) est asserté **bien que le privilège le garantisse** : une garantie de privilège se perd en une ligne de migration. ⚠️ (g) n'est **pas** couvert par P-23 : celle-ci analyse le **code** et prouve qu'aucun calcul ne lit `horodatage_client` ; (g) éprouve le **comportement**, et c'est la forme qu'avait déjà `hebergement_tarification.rs` au cycle 004.
- [ ] T061 [US4] Instancier `tester_classe_bcd!(taxe_sejour_constat, classe = B, …)` et étendre `outbox_transactionnel.rs` à `heb.sejour.clos` — charge utile portant **le total, toutes les lignes, les ajustements et le constat**, de sorte que l'opération se reconstitue **sans consulter aucune autre table** (TRX-02).

---

## Phase 9 : Écran `R7` La note et le départ — MAQUETTÉ (sert US4)

**Référence** : `docs/design/html/R7-note-depart.html` — **état nominal SEUL**. Les états
`-envoi` et `-echec` sont de la **certification FNE (FIS-05, tranche T3)** et ne se codent pas ici.

- [ ] T062 [US4] [P] Écrire `app/modules/sejours/NoteSejour.vue` d'après `R7-note-depart.html` : la **section hébergement seule**, nuit par nuit avec sous-total, la mention obligatoire « **Document non fiscal — ne tient pas lieu de facture** », et le **total provisoire**. ⚠️ **L'absence des sections restaurant, bar et autres frais doit être visible comme une absence**, pas comme un vide inexpliqué — elles viennent avec les points de vente (T2). Tout montant passe par `app/core/format/montant.ts` avec l'**espace fine insécable** ; les heures gardent l'espace **ordinaire**.
- [ ] T063 [US4] Écrire `app/modules/sejours/EcranDepart.vue` : la liste des séjours en cours, la note du séjour choisi, et l'action finale. ⚠️ **Éléments ABSENTS de la maquette, jamais grisés** : « Déjà versé … par Wave », « Il resterait à payer aujourd'hui » et la mention « encaissement, facture » de l'action finale — **CAI (T2) et FIS (T3)**. Le séjour se clôt sur une note **arrêtée et non réglée**, et l'écran **le dit en toutes lettres** plutôt que de laisser croire à un paiement. Rendre aussi la **fiche de police** (opération 16), qui porte la même mention obligatoire « **Document non fiscal — ne tient pas lieu de facture** » que la note : c'est un **document opérationnel** au sens de FIS-02, et le principe V l'exige de tous (FR-048). Écrire `clore-sejour.ts` selon la septième couche.
- [ ] T064 [US4] Créer `app/pages/depart.vue` — route `/depart`, **sans segment dynamique** (le séjour choisi passe en paramètre de requête : une route sans paramètre est plus simple à couvrir par P-22). Racine unique, i18n `fr`/`en`, **clair ET sombre**.

---

## Phase 10 : Écran `R5` Fiche client et recherche — DÉRIVÉ (sert US2)

**Référence** : `docs/design/derivation.md`, ligne `R5` — *« hérite de `R7` : liste + fiche, pas de
total »*. **Ouvrir `R7-note-depart.html` et la respecter.** Cet écran suit `R7` parce qu'il en
hérite : livré avant, il inventerait le motif puis divergerait.

- [ ] T065 [US2] [P] Écrire `app/modules/sejours/FicheClient.vue` : identité, coordonnées, préférences, et **l'historique des séjours** de tous les établissements du tenant, du plus récent au plus ancien. Colonne de droite du motif `R7`, **sans le bloc de total**.
- [ ] T066 [US2] Écrire `app/modules/sejours/EcranClients.vue` : la recherche qui **réduit la liste pendant la frappe**, les trois formes servies par la même entrée, l'indication de troncature. Action de création **absente** sans `sej.client.gerer` — jamais grisée, la vérification portant sur le **HTML rendu** et non sur un attribut `disabled`. `derivation.md` est mis à jour : `R5` passe d'« inscrit » à **« codé »**.
- [ ] T067 [US2] Créer `app/pages/clients.vue` — route `/clients`, racine unique, i18n `fr`/`en`, **clair ET sombre**. Cette entrée reste **disponible sans module hébergement** : la fiche client ne dépend d'aucun module d'activité.

---

## Phase 11 : US5 — La prolongation, et le conflit dit en face (P2 interne · P0 au corpus)

- [ ] T068 [US5] Écrire `sejour/service.rs`, méthode `prolonger` : vérification de disponibilité **sur l'intervalle étendu**, temps de remise en état compris ; extension de l'occupation ; lignes d'hébergement supplémentaires au tarif en vigueur ; événement `heb.sejour.prolonge`. Refus sur séjour clos.
- [ ] T069 [US5] ★ Dans `backend/crates/verticales/hebergement/src/sejour/{service,modele}.rs`, le refus **nomme le conflit** : `409 conflit_occupation_suivante` portant l'unité, **l'instant de début de l'occupation suivante**, et les **unités alternatives** de la même catégorie libres sur l'intervalle étendu. ⚠️ Un message générique est un défaut (FR-070) : c'est la différence entre un refus qu'Adjoua peut expliquer au client et un refus qu'elle contournera. Traiter aussi `422 bascule_formule_non_confirmee` — le franchissement de `seuil_bascule_nuitee_minutes` est **annoncé avec son montant avant confirmation**, et la requête se rejoue avec `bascule_acceptee: true`.
- [ ] T070 [US5] Ajouter l'opération **13** `sejour_prolonger` à `sejours.rs`, l'action à `EcranDepart.vue`, et les libellés de refus du lexique (T002). **Terminer par** : utoipa, `pnpm generer:client`, diff commité, build vert.
- [ ] T071 [US5] Écrire `backend/tests/sejour_prolongation.rs` : intervalle étendu libre → prolongé ; occupation suivante → **conflit nommé** avec ses alternatives ; franchissement de seuil → annoncé avant confirmation ; séjour clos → refusé ; **la contrainte d'exclusion protège toujours**, temps de remise en état compris.

---

## Phase 12 : US6 — Le départ anticipé, régularisé et tracé (P2 interne · P0 au corpus)

- [ ] T072 [US6] Étendre la méthode `clore` de `backend/crates/verticales/hebergement/src/sejour/service.rs` : quand le départ est prononcé avant la fin prévue, l'hébergement est arrêté sur la **durée réelle** et la différence portée en **ligne de régularisation distincte** (`motif = 'depart_anticipe'`), la ligne initiale restant **inchangée** — ⚠️ `montant_mineur` **peut être négatif**, le type `Rebascule` du cycle 004 le dit déjà. La disponibilité rendue part de **l'instant réel du départ** augmenté du temps de remise en état, jamais de l'heure initialement prévue. Le constat porte les nuits **réellement** constatées.
- [ ] T073 [US6] Tracer la régularisation au **registre des actions** avec auteur, instant d'autorité, montant, séjour et motif — *« ce que le propriétaire achète »* (cadrage §8.3), et l'écart exact que le cahier papier ne lui montrait pas. Étendre `backend/tests/sejour_depart.rs` : trois nuits prévues, départ après deux → ligne de régularisation, ligne initiale intacte, constat à deux nuits, trace complète.

---

## Phase 13 : US7 — Le changement de chambre, avec son histoire (P3 interne · P0 au corpus)

- [ ] T074 [US7] Écrire `sejour/service.rs`, méthode `changer_unite` — **une transaction** : clôture de l'occupation d'origine à `now()`, ouverture d'une occupation sur l'unité cible **à partir du même instant**, **sur le même séjour**. Lignes portant le tarif **propre à chaque période**. Refus `409 unite_cible_occupee` avec le conflit nommé — ⚠️ **aucun déplacement partiel n'est jamais produit**, les deux occupations vivant dans la même transaction. Événement `heb.sejour.unite_changee` et trace au registre des actions avec les **deux** unités et l'instant.
- [ ] T075 [US7] Ajouter l'opération **14** `sejour_changer_unite` à `sejours.rs` et l'action à `EcranDepart.vue` (accessible depuis le conflit de prolongation, qui propose déjà les alternatives). **Terminer par** : utoipa, `pnpm generer:client`, diff commité, build vert.
- [ ] T076 [US7] Écrire `backend/tests/sejour_changement_unite.rs` : **deux occupations, un séjour** ; historique conservant les deux avec leurs unités et périodes ; tarif propre à chaque période ; refus sans déplacement partiel ; **constat figé sur l'ensemble du séjour**, jamais par occupation. ⚠️ **Assertion P-09 supplémentaire** : deux occupations contiguës sur deux unités **différentes** ne déclenchent pas la contrainte — et c'est justement pourquoi le test doit prouver qu'elle **se déclencherait** si les unités étaient les mêmes.

---

## Phase 14 : Seeds, recollement des portes et mesure

- [ ] T077 [P] Ajouter aux seeds `backend/migrations/seeds/` : 12 fiches clients Deloria, 3 séjours — nuitée en cours (2 nuits, 2 accompagnants), passage en cours (2 h), **séjour clos avec son constat figé** — et 2 fiches + 1 séjour pour « Résidence Test ». ⚠️ **Jamais par migration** : une table en `FORCE ROW LEVEL SECURITY` accepte un `INSERT` de migration **en n'écrivant rien**, sans erreur. Le séjour clos est ce qui rend visible que ce cycle a laissé le calcul à FIS : `nuitees_assujetties` et `montant_mineur` à `NULL`. Étendre `backend/tests/seeds_rejouables.rs`.
- [ ] T078 Mettre à jour `backend/tests/couverture_portes.rs` — **en dernier**, il suppose que tout existe : P-01b/P-08 de **56 à 73** opérations avec sa **ventilation par lot** (`cycle 006 — fiches clients`, 6 · `cycle 006 — séjours`, 10 · `cycle 006 — état des unités`, 1) ; P-05 de **27 à 36** types ; P-07 de **29 à 38** tables. ⚠️ **Le décompte est ventilé, jamais posé en un seul nombre** : un total unique se corrige en changeant un chiffre, une ventilation oblige à dire de quel lot vient l'écart — et c'est cette phrase-là qu'on ne peut pas écrire sans s'en apercevoir.
- [ ] T079 Vérifier que `backend/tests/horodatage_autorite.rs` (**P-23**) **voit les fichiers nouveaux** : son périmètre est **découvert**, pas énuméré, mais son décompte de fichiers inspectés doit avoir augmenté. Les quatre calculs du cycle — durée réelle, début de passage, instant de figeage, instant de changement d'unité — lisent tous `now()` de la base. `horodatage_client` **est écrit** sur `accompagnant` et `preference_personne`, et c'est permis : **écrire la colonne n'est pas s'appuyer dessus**.
- [ ] T080 Exécuter `cargo sqlx prepare` **à DEUX passes** selon la procédure de `CLAUDE.md` — ce cycle écrit des requêtes dans les **seeds** (binaire) **et** dans les tests d'intégration, et une passe unique en perdrait la moitié. Puis les **deux** contrôles, dans l'ordre : `git status --short backend/.sqlx` **sans aucune suppression**, puis — ⚠️ **précédé du `touch`** qui force la réévaluation des macros, sans quoi le check affiche `Finished` en une seconde **sans consulter `.sqlx`** — `SQLX_OFFLINE=true DATABASE_URL= cargo check --workspace --all-targets --locked`.
- [ ] T081 Passer les portes une par une : `pnpm porte:p01 p02 p04 p05b p10 p15 p19 p20 p21 p21b p22` · `pnpm porte:p22:negatif` · `pnpm lint` · `pnpm test:i18n` · `pnpm lint:tokens` · `cargo test --test architecture` (**P-03** — aucune arête `socle/ → verticales/`, alors que `socle/comptes` sert désormais le séjour). ⚠️ **P-22 exige l'API, la base et les seeds** ; **P-21b** réclame les glyphes nouveaux du module au sous-réglage d'icônes (`pnpm --filter @kaya/app icones:generer`) — les ajouter **avant** qu'elle ne les réclame. ★ **Deux ajouts que la seule exécution des portes ne donnerait pas** : (1) **P-05b** — vérifier que `scripts/ci/outbox-sans-purge.sh` **compte trois registres immuables** et non deux ; `taxe_sejour_constat` en est un, et la constitution exige de prouver qu'une cible n'est pas vide, pas seulement que la porte sait échouer. (2) exécuter **`cargo test --test agnosticite_socle`** (SC-014, ETB-02c) et le dire : c'est le **premier cycle où `socle/comptes` sert une verticale**, donc la première fois que ce test peut rougir pour une raison réelle. Vérifier au passage que la recherche de fiches clients reste disponible sur un établissement **sans module `HEBERGEMENT`**.
- [ ] T082 Exécuter le balayage hors ligne `pnpm exec playwright test tests-e2e/hors-ligne.spec.ts` en y ajoutant les **quatre routes** du cycle. ⚠️ **Le contrôle qui empêche ce test de mentir tient en une ligne** : vérifier que l'URL n'est **pas** `/connexion` — le jeton vit en mémoire, un rechargement exige le réseau, et hors ligne toutes les routes y renvoyaient (neuf cas verts, neuf fois le même écran, cycle 005). ⚠️ **Séquencer avec la suite backend** : `exiger_grand_livre_sans_consommateur_concurrent` refuse de tourner quand l'API est allumée. Arrêter **par port** : `lsof -ti:8080 | xargs kill` — **jamais `pkill -f`**.
- [ ] T083 **Mesurer et consigner** le chronométrage humain (FR-106) dans `specs/006-clients-sejours-enregistrement/mesures-terrain.md` : matériel de référence, jeu de données, point de départ et point d'arrivée du chronomètre, valeurs relevées pour un **passage client inconnu** et une **arrivée client connu**. ⚠️ **Cible 30 s et 60 s ; au-delà de 90 s pour un passage, la story est en échec** — pas améliorable, en échec. Le protocole est versionné au dépôt ; la mesure **n'est jamais assertée en CI**.
- [ ] T084 Dérouler la **démo de fin de tranche T1** de bout en bout sur les seules données de démonstration, selon `specs/006-clients-sejours-enregistrement/quickstart.md` §9, ses six étapes en clair **et** en sombre : *« Yao enregistre un client en chambre B3 pour 2 nuits, puis un passage de 4 h en A1 — la disponibilité empêche tout chevauchement, tout est tracé. »*
- [ ] T085 [P] Mettre à jour les documents de fin de cycle : `docs/design/derivation.md` (`R3` et `R5` d'« inscrit » à **« codé »**) · `docs/registre-classes-offline.md` journal **v1.4.0** · `docs/taxonomie-audit.md` — ⚠️ **deux mouvements, en sens inverses** : la famille 10 `forcage_disponibilite` **reste « due »** et il faut **le dire** — ce cycle ne livre **aucun** forçage, un changement d'unité vers une chambre occupée étant refusé ; et la **famille de consultation de pièce d'identité, ajoutée par T018a, naît « branchée »**, avec son chemin de code. Une famille déclarée « branchée » sans chemin de code ferait échouer le harnais, et une famille branchée non déclarée l'échouerait aussi. Porter le décompte de **onze à douze**.

---

## Phase 15 : Revue Definition of Done

- [ ] T086 Écrire `specs/006-clients-sejours-enregistrement/revue-dod.md` — les **dix points** de `docs/user-stories-v1.md` §0.4, chacun avec sa preuve exécutable, et ce qui **n'est pas** satisfait dit en toutes lettres : **(1)** critères couverts par tests unitaires **et** d'intégration sur les transitions d'état · **(2)** utoipa à jour, client TS régénéré **sans diff manuel** · **(3)** migration versionnée, `cargo sqlx prepare` vert **à deux passes**, seeds à jour · **(4)** RLS `ENABLE` **et** `FORCE` sur les 9 tables, avec test d'isolation · **(5)** classe hors-ligne déclarée pour les 9 entités, avec leurs tests **instanciés** · **(6)** outbox émis pour les 9 transitions · **(7)** clés i18n `fr` et `en`, aucune chaîne en dur · **(8)** les 4 écrans vérifiés en clair **et** en sombre · **(9)** ⚠️ **point SANS OBJET, et c'est écrit plutôt que coché** — aucune story du périmètre ne dit « paramétrable », aucune clé nouvelle au catalogue · **(10)** ⚠️ **point NON SATISFAIT, et c'est nommé** — la note et la fiche de police sont produites mais **non imprimées sur thermique réelle** : cela relève d'**IMP, tranche T2**. Consigner aussi les **défauts trouvés** pendant le cycle, sur le modèle de `specs/005-.../revue-dod.md` : ce qu'un cycle trouve vaut souvent plus que ce qu'il construit.

---

## Écrans non codés — signalés, pas inventés

| Écran | Pourquoi il n'est pas ici |
|---|---|
| `R6` Note temps réel | **SEJ-03, tranche T2.** Inscrit à `derivation.md` (hérite de `R7`, « sans l'action finale »), donc **codable** — mais aucune story du périmètre ne l'appelle : le principe X interdit de le bâtir |
| `R2` Vue du jour | Hérite de `R1` + composant 14. Servira les arrivées et départs du jour ; **aucune story du périmètre ne l'appelle** |
| `M5` Enregistrement OCR | **SEJ-06, P1, tranche T4.** Hérite de `R3`, qui existe à partir de ce cycle — la dérivation devient donc *possible*, elle ne devient pas *due* |
| États `-envoi` et `-echec` de `R7` | **FIS-05, tranche T3** — certification FNE |

**Aucun écran de ce cycle ne sort des trois cas.** Si un motif manquait à la bibliothèque pendant
l'implémentation, **la tâche s'arrête et l'écran part en maquettage** : un composant nouveau se
maquette, il ne s'improvise pas dans un écran.

---

## Décisions en attente

| # | Décision | Effet si elle tombe autrement | Échéance |
|---|---|---|---|
| 1 | **Formulation du message d'écriture orpheline** (T002) — « Cette information est arrivée après le départ du client. » | Une phrase seulement ; aucun effet de modèle | Atelier terrain, avant T045 |
| 2 | **Axe des nuits de la taxe** — le récapitulatif dit « 500 F pour un séjour de 3 nuits », l'arbitrage du 2026-08-03 raisonne sur « 500 F par nuit ». Les deux portent sur l'axe des **nuits**, que B-10 ne touche pas | Change le **seed** de `regle_conversion_taxe` (`une_nuitee_par_occupation` → `au_prorata`) — **une donnée, pas du code** | Même atelier que T003 |
| 3 | **Gabarit officiel de la fiche de police** | Un **rendu**, pas une donnée : s'ajoute sans migration | Cartographie avec le pilote |
| 4 | Généraliser `numerotation_fiche_police` en compteur de documents opérationnels du socle | Refactorisation à un consommateur près | **FIS-02**, tranche T3 |

---

## Dépendances entre stories

```
Phase 1 (documents)  ──►  Phase 2 (socle)
                              │
                              ├──►  US2 fiche client  ────────────────┐
                              │        (Ph. 3)                        │
                              │                                       ▼
                              └──►  US1 passage 🎯  ──►  US3 arrivée  ──►  US4 départ
                                       (Ph. 4-5)          (Ph. 6-7)        (Ph. 8-9)
                                                                              │
                                              ┌───────────────────────────────┤
                                              ▼               ▼               ▼
                                         US5 prolong.    US6 anticipé    US7 chgt unité
                                          (Ph. 11)        (Ph. 12)        (Ph. 13)
                                                              │
                                       Ph. 10 écran R5 ◄──────┘ (hérite de R7, Ph. 9)
                                                              │
                                                              ▼
                                                    Ph. 14 recollement ──► Ph. 15 DoD
```

**US2 est la seule story dont l'essentiel est livrable sans aucune autre** — recherche, fiche et
préférences en Phase 3. Son **historique** (T030a) attend que les séjours existent, et son écran
`R5` (Phase 10) attend le motif de `R7` : la story se solde donc en trois temps, et l'écrire évite
de la cocher à moitié faite. **US1 porte le cœur du séjour** dont US3, US4, US5, US6 et US7
dépendent toutes.

---

## Parallélisation

| Lot | Tâches | Condition |
|---|---|---|
| Documents | T005, T006, T007 | Après T001–T004 |
| Socle | T012, T013 | Après T008 |
| Sécurité de l'identité | T018a | Après T015 — **avant** T019, qui expose la fiche |
| Composants `R4` | T038, T039 | Après T037 |
| Écran `R7` / `R5` | T062 · T065 | Fichiers distincts |
| Fin de cycle | T077, T085 | Après T076 |

**Ne jamais paralléliser** : T023 avec T024 (P-09 se lève dans le même changement), T040 avec T043
(le budget de gestes se contraint, il ne se constate pas), T078 avec quoi que ce soit (il suppose
que tout existe).

---

## Stratégie de livraison

1. **Phases 1–2** — rien n'est démontrable, tout est bloquant. Une demi-journée à une journée par
   tâche.
2. **Phase 3 (US2)** — première story livrable, démontrable par API et par sa mesure.
3. **Phases 4–5 (US1)** — 🎯 **le MVP du cycle.** À la fin de la phase 5, un passage s'enregistre en
   deux gestes sur un vrai navigateur, et le budget est **mesuré**, pas espéré. **Si le budget n'est
   pas tenu ici, on s'arrête et on reprend l'écran** : tout ce qui suit s'appuie sur un parcours dont
   le corpus dit qu'il sera contourné s'il est lent.
4. **Phases 6–9** — l'arrivée longue et le départ. Le produit devient exploitable de bout en bout.
5. **Phases 11–13** — les trois cas du départ. Toutes **P0 au sens du corpus**, aucune n'est
   optionnelle.
6. **Phases 14–15** — recollement, mesure et revue. **T078 en dernier**, toujours.
