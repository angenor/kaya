---

description: "Tâches — Cycle 002 · Établissements, modules d'activité et configuration héritée"
---

# Tasks: Tenants, établissements, modules d'activité et configuration héritée

**Input**: Documents de conception de `specs/002-etablissements-modules-activite/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), [contracts/](contracts/), [quickstart.md](quickstart.md)

**Tests**: obligatoires, et non optionnels sur ce cycle — la spécification les exige nommément
(FR-021 à FR-028 : « écrits avant l'implémentation »), la Definition of Done les impose (§0.4,
points 1, 4, 5), et les portes P-06, P-07, P-08 et P-13 n'existent que sous forme de tests.

**Organisation**: par story, mais **ordonnées par dépendance de schéma** — instruction explicite du
cycle. Deux conséquences assumées :

- **US5 (points de vente) précède US4 (configuration héritée)** : le point de vente est le
  quatrième niveau de la chaîne d'héritage, et `parametre_configuration` porte une clé étrangère
  vers lui. Écrire la configuration d'abord imposerait une migration de rattrapage.
- **US1 (les trois parcours structurels) est scindée** : son harnais est écrit en **Phase 1, avant
  toute migration**, et chaque story suivante y branche ses étapes. Sa phase de clôture arrive en
  fin de cycle. Voir la note d'ordonnancement ci-dessous.

## Format: `[ID] [P?] [Story] Description`

- **[P]** : parallélisable — fichiers distincts, aucune dépendance sur une tâche inachevée
- **[Story]** : US1 à US7, telles que numérotées dans [spec.md](spec.md)
- Chaque tâche porte ses chemins de fichiers exacts

## Conventions de chemin

Monorepo du cycle 001 : `backend/migrations/`, `backend/crates/socle/etablissements/src/`,
`backend/api/src/routes/`, `backend/tests/`, `app/`, `docs/`.

---

## Note d'ordonnancement — pourquoi le harnais est écrit en premier

ETB-02c exige que les trois parcours soient écrits **avant l'implémentation**. Le mécanisme retenu
en fait une contrainte active plutôt qu'une intention :

Chaque étape déclare une **sentinelle** — la table ou le point d'entrée dont l'existence prouve que
l'étape est réalisable. Tant que la sentinelle est absente, l'étape est « due » et le harnais est
**vert**. Dès qu'une migration crée la table, la sentinelle apparaît et le harnais **échoue** tant
que l'étape n'est pas branchée.

Écrit en Phase 1, le harnais est donc vert à vide (0 exercée / 8 déclarées, 8 dues) et **oblige**
chaque tâche suivante à le brancher — le manquement fait échouer le build, pas une revue.

> **Le harnais n'est jamais commité rouge.** Une tâche qui crée une sentinelle et son branchement
> est **une seule tâche**, pas deux. C'est pourquoi les tâches de migration ci-dessous portent leur
> branchement dans leur propre périmètre.

**Aucune tâche P1 dans ce cycle** : ETB-01 à ETB-05 sont toutes **P0** et ETB-06 (P1) est hors
périmètre. La section de fin de liste habituellement réservée aux P1 est donc sans objet, et il
n'en est pas fabriqué une pour respecter la forme.

---

## Phase 1: Préalables — écrits avant toute ligne de schéma

**Objectif** : ce qui doit exister avant que quoi que ce soit d'autre soit écrit.

- [X] T001 [P] Ajouter les six entrées de vocabulaire à `docs/design/lexique.md`, avec formulation française et anglaise, **avant** toute clé i18n : `capacite` → *n'apparaît jamais ; seule la capacité concrète est nommée* — `STOCK` → « **Suivi du stock** » / *Stock tracking* ; `point_de_vente` → « **Point de vente** » / *Point of sale* ; point de vente sans tables → « **Comptoir** » / *Counter* ; valeur héritée → « **Vaut pour tous vos établissements** » / *Applies to all your establishments* ; valeur surchargée → « **Modifié ici** » / *Changed here*. `classement` et `numéro de compte contribuable (NCC)` sont conservés tels quels — vocabulaire fiscal officiel, règle 2 du lexique — et l'entrée le consigne explicitement.
- [X] T002 Écrire le harnais des trois parcours dans `backend/tests/agnosticite_socle.rs` : structure `Etape { nom, cycle_du, sentinelle, branchement }`, les **huit étapes déclarées** par parcours (création, activation, résolution de configuration, refus de capacité — livrées ; vente comptoir, encaissement, document fiscal, clôture journalière — dues à PDV, CAI, FIS), la lecture de `information_schema.tables` et de `application::contrat_complet()`, le **comptage exercées / déclarées** et le commentaire de tête déclarant le périmètre inspecté et ce qui ne l'est pas (§ « Couverture des portes »). **Vert à vide : 0/8, 8 dues.**
- [X] T003 [P] Vérifier l'environnement complet — `docker compose -f infra/compose.yml up -d`, `scripts/dev/preparer-base.sh`, `scripts/dev/preparer-stockage.sh` (requis par ETB-05, non exercé au cycle 001), et confirmer que `/health` rapporte les trois dépendances saines.
- [X] T004 Trancher l'ajout des deux paquets de test front (utilitaire de montage Vue, environnement DOM) : **vérifier leur version sur le registre npm officiel, citer l'URL**, épingler exactement dans `app/package.json`, commiter `pnpm-lock.yaml`, et inscrire la ligne correspondante au journal de `docs/versions-gelees.md` pour la revue du 2026-08-31. **En cas de refus, consigner dans `plan.md` que SC-005 se vérifie sur la seule fonction de sélection** — couverture moindre, écrite plutôt que supposée. **Dans le même passage, régulariser les six paquets déjà présents dans `app/package.json` et absents du gel §3.2** — `vitest`, `eslint`, `@eslint/js`, `eslint-plugin-vue`, `typescript-eslint`, `@tailwindcss/vite` : écart hérité du cycle 001, qui rend la procédure d'ajout incohérente tant qu'il subsiste. Versions **lues du dépôt**, jamais reproposées ; seule l'URL du registre est à ajouter.

**Point de contrôle** : le vocabulaire est arrêté, le harnais est vert à vide et surveille déjà.

---

## Phase 2: Fondations (prérequis bloquants)

**Objectif** : l'établissement enrichi et les quatre référentiels. **Aucune story ne démarre avant.**

**⚠️ Les deux migrations de cette phase créent les premières sentinelles.** Le harnais de T002
passe au rouge s'il n'est pas branché dans la même tâche.

- [X] T005 Migration `backend/migrations/0007_etablissement_identite.sql` — sept colonnes sur `etablissements.etablissement` (`juridiction`, `classement`, `etoiles`, `commune`, `adresse`, `ncc`), **exclusivement en `ADD COLUMN ... NOT NULL DEFAULT`** puis `DROP DEFAULT` où la valeur n'a pas de sens permanent. Contraintes `CHECK` : classement dans les trois valeurs, égalité de conditions `(classement = 'ETOILES') = (etoiles IS NOT NULL)`, `etoiles IS NULL OR etoiles > 0` — **aucun plafond en base, le nombre maximal d'étoiles est une règle de juridiction (porte P-12)**, même traitement que le `ncc`, réduit à « non vide si présent ». **Aucun `INSERT` ni `UPDATE` dans cette migration** — la table est en `FORCE ROW LEVEL SECURITY` et un DML n'y toucherait aucune ligne sans lever d'erreur ([research.md R-08](research.md)). `0002_etablissements_socle.sql` n'est pas modifiée (porte P-02). Vérifier après application que les deux établissements seedés portent bien les valeurs.
- [X] T006 Migration `backend/migrations/0008_referentiels_activite.sql` — quatre référentiels globaux `module_activite`, `capacite`, `profil_stock`, `parametre_catalogue`. **Ordre impératif** : `CREATE TABLE` → `INSERT` des valeurs → `ENABLE`/`FORCE ROW LEVEL SECURITY` → `CREATE POLICY`. Deux politiques par table — `lecture_universelle FOR SELECT USING (true)` et `administration_editeur FOR ALL TO kaya_owner`. `GRANT SELECT` seul à `kaya_app`. Colonnes `implementee` + `UNIQUE (code, implementee)` support des clés étrangères composites. `libelle_cle` porte une **clé i18n, jamais un libellé**. L'absence de `tenant_id` est **nommée en commentaire** comme l'a été `tenant` au cycle 001.
- [X] T007 [P] Déclarer les deux entités absentes du registre — `profil_stock` et `parametre_catalogue` — au §5.1 de `docs/registre-classes-offline.md`, **classe C**, avec entrée au journal §13. Ajouter la ligne qui distingue **l'écriture (C) de la lecture en cache (A, fraîcheur affichée)** pour l'ensemble des référentiels et des paramètres. Étendre `backend/tests/classes_offline.rs` et vérifier qu'il échoue sur une table non déclarée.
- [X] T008 Étendre `backend/tests/rls_catalogue.rs` (porte P-07) au régime des référentiels globaux : compter les quatre tables comme **conformes et les nommer**, en vérifiant qu'elles portent bien deux politiques et aucun droit d'écriture pour `kaya_app`. Ajouter le test négatif : une table de référentiel sans `administration_editeur` fait échouer la porte.
- [X] T009 Étendre `backend/crates/socle/etablissements/src/lib.rs` — struct `Etablissement` enrichie, type somme `Classement`, et créer `backend/crates/socle/etablissements/src/traits.rs` avec les **six traits** de [contracts/traits-exposes.md](contracts/traits-exposes.md), annotés `#[async_trait::async_trait]`. Aucune implémentation à ce stade : les signatures seules, qui contraignent les tâches suivantes.

**Point de contrôle** : le schéma porte l'identité et les référentiels ; les traits existent et
compilent à vide.

---

## Phase 3: US2 — L'exploitant décrit son établissement et choisit ses services (P0)

**Objectif** : créer un établissement, activer et désactiver ses services. Deux étapes du harnais
sont branchées ici.

**Test indépendant** : créer deux établissements aux services différents — cinq pour l'un, un seul
pour l'autre — et vérifier qu'aucun service non activé n'apparaît nulle part.

- [ ] T010 [US2] Migration `backend/migrations/0009_activation_modules.sql` — `etablissement_module` et `module_capacite`. Clés étrangères **composites** vers les référentiels plus `CHECK (module_implemente)` / `CHECK (capacite_implementee)` / `CHECK (profil_implemente)` : c'est la contrainte qui rend le refus structurel. `UNIQUE (etablissement_id, module_code)`. RLS `ENABLE` + `FORCE`, politique `isolation_tenant` avec `USING` **et** `WITH CHECK`. `GRANT SELECT, INSERT, UPDATE` — **pas de `DELETE`**, le privilège dit la règle. Déclarer les deux entités au registre (déjà présentes au §5.1 — vérifier, ne pas dupliquer).
- [ ] T011 [P] [US2] Repository de l'établissement dans `backend/crates/socle/etablissements/src/etablissement/{modele,repository}.rs` — macros `query!` **sur littéral uniquement**, transaction en paramètre jamais ouverte ici, `ON CONFLICT (id) DO NOTHING ... RETURNING` pour distinguer `201` de `200`. Suivre `docs/module-dore.md` couche 3, sans le réinventer.
- [ ] T012 [US2] Service de l'établissement dans `backend/crates/socle/etablissements/src/etablissement/service.rs` — ordre des opérations du module doré : valider, ouvrir la transaction, **poser le tenant courant**, vérifier, insérer, **émettre l'événement seulement si la ligne vient d'être créée**, commit. Types d'événements `etablissement.cree`, `.modifie`, `.classement_change`, `.fuseau_change`. Refus `devise_figee` posé **à vide** — la fonction qui compte les opérations financières rend zéro et sera branchée par CAI.
- [ ] T013 [P] [US2] Repository et service des modules dans `backend/crates/socle/etablissements/src/modules/{modele,repository,service}.rs` — activation et désactivation idempotentes, `UPDATE actif` jamais `DELETE`, événements `etablissement_module.active` / `.desactive`. Implémenter `RegistreModules` : **le trait ne rend jamais les modules inactifs**.
- [ ] T014 [US2] Câbler le point d'accrochage `ObstacleDesactivation` dans `ServiceModules` (`Vec<Arc<dyn ObstacleDesactivation>>`, **vide à ce cycle**) et écrire le test qui enregistre un **obstacle factice** et constate que la désactivation est refusée en le nommant, dans `backend/tests/desactivation_bloquee.rs`. Sans ce test, un point d'accrochage jamais exercé se casse sans que rien ne le signale.
- [ ] T015 [US2] Endpoints de l'établissement et des services dans `backend/api/src/routes/etablissements.rs` et `backend/api/src/routes/services.rs` — opérations 1 à 4 et 8 à 9 de [contracts/http-api.md](contracts/http-api.md). `#[utoipa::path]` **sans verbe ni chemin** (déduits de l'attribut Actix), montage par `service(...)` dans `backend/api/src/routes/mod.rs`. Corps d'erreur structuré `{ code, valeur, message }`. **`GET .../services` ne rend que les services actifs — aucun paramètre `inclure_inactifs`.** La réponse de modification du fuseau porte `avertissement: "fuseau_change"`, que l'interface doit présenter avant de confirmer. Terminer par la régénération du contrat et du client : `scripts/ci/generer-client.sh`, commit du diff, build vert (porte P-01).
- [ ] T016 [US2] Endpoints de lecture des référentiels dans `backend/api/src/routes/referentiels.rs` — opérations 5 à 7, **lecture seule, aucun verbe d'écriture exposé**. Terminer par régénération du contrat et du client, build vert.
- [ ] T017 [US2] **Brancher les étapes 1 et 2 du harnais** (`creation_etablissement`, `activation_module`) dans `backend/tests/agnosticite_socle.rs` pour les trois parcours. Le décompte doit passer à **2 exercées / 8 déclarées**. Vérifier à la main l'échec attendu en retirant un branchement, puis le rétablir.
- [ ] T018 [US2] Étendre `backend/tests/isolation_tenant.rs` (porte P-08) et `backend/tests/outbox_transactionnel.rs` (porte P-05) aux opérations et aux quatre types d'événements de cette phase. Ajouter **l'assertion explicite** que les trois référentiels rendent la même chose aux deux tenants — sans elle, un relecteur futur prendra le comportement pour une fuite.

**Point de contrôle** : un établissement se crée, ses services s'activent et se désactivent, et
rien ne fuit entre tenants.

---

## Phase 4: US3 — Une capacité non implémentée est refusée, jamais ignorée (P0)

**Objectif** : les neuf refus, tenus à trois couches. **C'est la porte P-06**, installée à vide au
cycle 001 et qui acquiert ici ses premières cibles.

**Test indépendant** : tenter les six capacités et les trois profils non implémentés par **tous**
les chemins d'écriture, et vérifier qu'aucune ligne n'est écrite.

- [ ] T019 [US3] Service de déclaration de capacité dans `backend/crates/socle/etablissements/src/modules/service.rs` — variantes d'erreur `CapaciteNonImplementee { code }` et `ProfilNonImplemente { code }`, **message distinct pour `AUCUN`** indiquant qu'une capacité non consommée ne se déclare pas. Événement `module_capacite.declaree`. Implémenter `RegistreCapacites`, qui rend `Option<CapaciteDeclaree>` et non un booléen.
- [ ] T020 [US3] Endpoints des capacités dans `backend/api/src/routes/services.rs` — opérations 10 et 11, `422` avec le corps structuré nommant la valeur refusée. Terminer par régénération du contrat et du client, build vert.
- [ ] T021 [US3] Écrire `backend/tests/capacites_refusees.rs` — **les neuf cas**, chacun vérifié à **deux niveaux** : `422` par l'API, **et** violation de contrainte sur `INSERT` direct sous le rôle applicatif. Vérifier après chaque tentative que **zéro ligne** a été écrite. Plus le cas nominal `STOCK`/`SIMPLE` : `201` puis `200` au rejeu.
- [ ] T022 [US3] **Brancher l'étape 3 du harnais** (`refus_capacite`) dans `backend/tests/agnosticite_socle.rs`. Pour le parcours d'agnosticité, l'étape vérifie que l'établissement au service fictif **ne déclare aucune capacité** et fonctionne malgré tout. Décompte : **3 exercées / 8**.

**Point de contrôle** : aucune capacité ni aucun profil non implémenté n'entre en base, par aucun
chemin.

---

## Phase 5: US5 — Les points de vente sont déclarés, un comptoir en est un (P0)

**Objectif** : le quatrième niveau de la chaîne de configuration. **Placée avant US4 par dépendance
de schéma** : `parametre_configuration` porte une clé étrangère vers `point_de_vente`.

**Test indépendant** : deux points de vente sur un même service, l'un avec tables et l'autre sans.

- [ ] T023 [US5] Migration `backend/migrations/0010_points_de_vente.sql` — `point_de_vente` et `table_pdv`. Clé étrangère vers `etablissement_module` : **c'est elle qui rend structurellement impossible** le rattachement à un service non activé. `caisse_id UUID NULL` **sans clé étrangère** — frontière de module, avec le commentaire qui dit pourquoi. `UNIQUE (etablissement_id, nom)`. RLS `ENABLE` + `FORCE`, politique `isolation_tenant`. **Aucun drapeau `est_comptoir`.** Vérifier la déclaration des deux entités au registre §5.1.
- [ ] T024 [P] [US5] Repository, service et trait dans `backend/crates/socle/etablissements/src/points_de_vente/{modele,repository,service}.rs` — implémenter `RepertoirePointsDeVente`, dont `tables` vide **est** le comptoir : aucune méthode `est_comptoir`. Événements `point_de_vente.cree` / `.modifie`, `table_pdv.creee` / `.desactivee`.
- [ ] T025 [US5] Endpoints dans `backend/api/src/routes/points_de_vente.rs` — opérations 12 à 15, `422 module_non_actif` nommant le service. `PUT .../tables` remplace l'ensemble : une liste vide fait du point de vente un comptoir, transition légitime. Terminer par régénération du contrat et du client, build vert.
- [ ] T026 [US5] Étendre `backend/tests/isolation_tenant.rs` et `backend/tests/outbox_transactionnel.rs` aux quatre opérations et aux quatre types d'événements. Ajouter au parcours « maquis » du harnais la création de son point de vente — le parcours « résidence » et le parcours d'agnosticité n'en créent **aucun**, et c'est ce qui est vérifié.

**Point de contrôle** : un maquis a ses points de vente, une résidence meublée n'en a aucun, et
aucune opération du socle n'en réclame.

---

## Phase 6: US4 — La configuration se résout par héritage, avec surcharge (P0)

**Objectif** : le composant le plus réutilisé du produit. Huit cycles le liront.

**Test indépendant** : la matrice complète — quatre niveaux, chacun défini ou absent, chaînes
écourtées comprises — en vérifiant **la valeur et son origine**.

- [ ] T027 [US4] Migration `backend/migrations/0011_configuration_heritee.sql` — `parametre_configuration` avec **trois clés étrangères nullables** (`etablissement_id`, `etablissement_module_id`, `point_de_vente_id`), `CHECK (num_nonnulls(...) <= 1)` et **`UNIQUE NULLS NOT DISTINCT`** : sans cette dernière, deux surcharges de niveau tenant portant la même clé passeraient toutes les deux et la résolution en choisirait une au hasard. `cle` en clé étrangère vers `parametre_catalogue`. Index sur `(tenant_id, cle)` et index partiels par niveau. RLS `ENABLE` + `FORCE`. Peupler le catalogue avec la clé `politique_impression`, portée la plus basse `POINT_DE_VENTE`, story `ETB-03`, **sans jeu de valeurs** — il est défini par le cycle IMP.
- [ ] T028 [US4] Repository et résolveur dans `backend/crates/socle/etablissements/src/configuration/{modele,repository,service}.rs` — implémenter `ResolveurConfiguration` : **une seule descente de chaîne**, rang de portée calculé en SQL depuis les colonnes renseignées, filtre sur `etablissement_module.actif` pour rendre inerte une surcharge de service désactivé **sans la supprimer**. `Option<ValeurResolue>` — jamais de valeur par défaut. `origine` obligatoire. `resoudre_tout` en un aller-retour. Événement `parametre_configuration.ecrit`, portant **l'ancienne valeur** en cas de surcharge.
- [ ] T029 [US4] Validation d'écriture d'un paramètre — clé au catalogue (déjà imposée par la clé étrangère), portée compatible avec `portee_la_plus_basse` (`422 portee_interdite`), et **type de valeur conforme au catalogue**. Le type `MONTANT_MINEUR` **refuse tout `JSONB` non entier** : c'est l'extension de la porte P-10 sans laquelle un montant en flottant entrerait par le `JSONB`. Étendre `scripts/ci/types-monetaires.sh` pour couvrir le catalogue.
- [ ] T030 [US4] Écrire `backend/tests/configuration_heritee.rs` — la **matrice exhaustive**, chaque cas vérifiant valeur **et** origine. Cinq cas nommés explicitement : tenant seul ; tenant + point de vente ; surcharge partielle (tenant et point de vente, ni établissement ni service) ; **défini nulle part → absence explicite, ni `null` ni défaut** ; surcharge sur service désactivé → remontée puis restitution à la réactivation. Plus l'isolation à **chaque niveau** : résoudre depuis le tenant A avec un `point_de_vente_id` du tenant B ne rend rien, pas même la valeur héritée du tenant A.
- [ ] T031 [P] [US4] Écrire `backend/tests/parametres_catalogue.rs` — **toute clé du catalogue figure au « Récapitulatif des paramètres d'établissement »** de `docs/user-stories-v1.md`. Comparaison asymétrique catalogue → récapitulatif, périmètre déclaré en tête. Ajouter `politique_impression` au récapitulatif dans le même changement (Definition of Done, point 9).
- [ ] T032 [US4] Endpoints de configuration dans `backend/api/src/routes/configuration.rs` — opérations 16 et 17. **Une clé sans valeur à aucun niveau est absente de la réponse**, jamais rendue à `null`. Chaque valeur porte son origine. Terminer par régénération du contrat et du client, build vert.
- [ ] T033 [US4] **Brancher l'étape 4 du harnais** (`resolution_configuration`) pour les trois parcours, y compris la **chaîne écourtée** du parcours d'agnosticité — un établissement sans point de vente résout sur trois niveaux. Décompte : **4 exercées / 8 déclarées, 4 dues**.

**Point de contrôle** : la résolution est complète et testée sur toute sa matrice ; les quatre
étapes livrables du harnais sont branchées.

---

## Phase 7: US6 — L'identité visuelle est posée et vérifiée avant d'être imprimée (P0)

**Objectif** : identité par tenant, surcharge partielle par établissement, aperçu immédiat.

**Test indépendant** : poser au tenant, surcharger sur un seul des deux établissements, constater
sur l'aperçu de chacun que le bon jeu s'applique.

- [ ] T034 [US6] Migration `backend/migrations/0012_branding.sql` — `branding`, `etablissement_id` nullable (`NULL` = niveau tenant), **toutes les colonnes de contenu nullables** : c'est le mécanisme de surcharge partielle, sans logique de fusion à écrire. `UNIQUE NULLS NOT DISTINCT (tenant_id, etablissement_id)`. `CHECK` de format hexadécimal sur `couleur_primaire`. `logo_objet_cle` porte une clé d'objet, **jamais le binaire**. RLS `ENABLE` + `FORCE`. Vérifier la déclaration au registre §5.1.
- [ ] T035 [US6] Repository, service et résolution champ par champ dans `backend/crates/socle/etablissements/src/branding/{modele,repository,service}.rs` — première valeur non nulle en descendant, origine rendue par champ. Téléversement du logo **via l'interface S3 uniquement** (`aws-sdk-s3`), une clé d'accès par usage. Plafond de taille : **constante technique nommée dans le code avec sa justification**, jamais une clé du catalogue de paramètres — un exploitant n'a aucune raison de la régler. Son dépassement produit un `413` **dont le message donne la limite**, jamais un refus muet. Événement `branding.modifie`, portant la clé d'objet et non le binaire.
- [ ] T036 [US6] Rendu du document de test et endpoints dans `backend/api/src/routes/branding.rs` — opérations 18 à 21. L'aperçu ne **rien enregistrer**. Test vérifiant la présence de la mention « **Document non fiscal — ne tient pas lieu de facture** » dans la sortie : sans lui, le premier aperçu ressemblant à une facture serait imprimé et présenté à un client. Terminer par régénération du contrat et du client, build vert.

**Point de contrôle** : l'identité visuelle se pose, se surcharge partiellement et s'aperçoit.

---

## Phase 8: US1 — Clôture des trois parcours structurels (P0) 🎯

**Objectif** : la preuve formelle que le socle ne suppose ni hébergement, ni point de vente, ni
stock. **Le garde-fou de toute extension future du produit.**

**Test indépendant** : les trois parcours verts en intégration continue, chacun indépendant des
deux autres, avec leur décompte affiché.

- [ ] T037 [US1] Compléter le parcours d'agnosticité dans `backend/tests/agnosticite_socle.rs` — création du service fictif `MODULE_FICTIF_TEST` sous `commun::pool_owner()`, **dans une transaction annulée**, ne déclarant **aucune** capacité. Vérifier que le parcours ne dépend d'aucun autre et peut s'exécuter en parallèle.
- [ ] T038 [US1] Écrire le test de non-fuite dans `backend/tests/agnosticite_socle.rs` — après exécution du jeu de seeds, **zéro occurrence** de `MODULE_FICTIF_TEST` dans quelque table que ce soit (FR-027).
- [ ] T039 [US1] Exercer le **test négatif du harnais** : créer à la main une table portant le nom d'une sentinelle d'étape due, constater l'échec nommant l'étape et le parcours, supprimer la table. Consigner la sortie observée dans un commentaire du fichier. **Sans avoir vu cet échec une fois, on ne sait pas si la porte regarde.**

**Point de contrôle** : les trois parcours sont verts, comptés, et l'un d'eux a été vu échouer pour
la bonne raison.

---

## Phase 9: Écran `G1` — Établissement et modules (P0)

**Objectif** : le premier écran du produit. Il solde la couche écran reportée par le cycle 001.

**Référence visuelle** — cas (b), **écran dérivé** : `docs/design/derivation.md` ligne
« `G1` Établissement et modules **hérite de `G2`** — Configuration ». Maquette dont il hérite, à
ouvrir et respecter : **`docs/design/html/G2-offre-hebergement.html`**, état de référence, plus
`G2-offre-hebergement-residence.html` pour la variante à service unique.

> **Le HTML de maquette n'est jamais copié ni déplacé vers `app/`** (porte P-19). On en lit les
> valeurs et la structure — sélecteur segmenté en tête, sections à `h2` `font-titre text-chiffre`,
> lignes-boutons `rounded-l-xs rounded-r-xl border-l-4`, bouton principal `h-13 rounded-xl bg-prim`
> — et on réimplémente en composants Nuxt avec i18n, mode sombre et chargement paresseux, que
> l'export ne contient pas.

- [ ] T040 Créer `app/pages/etablissement.vue` réduite à une coquille et `app/modules/etablissements/` portant le contenu métier, chargé par `defineAsyncComponent(() => import(...))` — **c'est ce qui rend le chargement paresseux par module effectif** et vérifiable sur la sortie de construction. Aucun appel natif : le choix de fichier du logo est un `<input type="file">` standard, donc **aucune extension de `PlatformAdapter`** et rien de nouveau pour la porte P-15.
- [ ] T041 Section « Identité » et section « Vos services » dans `app/modules/etablissements/` — d'après `G2`. Un service inactif est **absent** : ni entrée désactivée, ni mention « disponible dans votre offre », ni marqueur masqué. Le mot « capacité » **n'apparaît nulle part** : seul « Suivi du stock » est affiché sous le service qui le consomme (T001). Aucune capacité non implémentée n'est proposée. Clés i18n fr **et** en, jetons de design exclusivement.
- [ ] T042 Section « Points de vente » et section « Identité visuelle » avec aperçu — d'après `G2`. Un point de vente sans tables s'affiche « **Comptoir** ». Les valeurs de configuration portent « **Vaut pour tous vos établissements** » ou « **Modifié ici** » selon leur origine (T001). L'aperçu du document de test s'affiche sans enregistrement préalable.
- [ ] T043 [P] Écrire le test unitaire de la **fonction de sélection des services visibles** dans `app/tests/` — pure, sans DOM, sans nouvelle dépendance. Puis, si T004 a conclu à l'ajout, le **test de rendu** : monter `G1` avec un établissement à service unique et vérifier qu'aucun libellé ni code des quatre autres services n'apparaît dans le HTML produit.
- [ ] T044 Vérifier `G1` **en mode clair et en mode sombre**, section par section (Definition of Done, point 8 — sans objet au cycle 001, exigible ici). Exécuter `pnpm test:i18n` (parité fr/en, porte P-16), `pnpm lint:tokens` (porte P-17) **et `pnpm lint` (porte P-15)** — la règle `no-restricted-imports` qui interdit l'accès natif hors de `PlatformAdapter` n'a jamais été déclenchée par une tâche, et une règle jamais exécutée ne garde rien. **Étendre `app/scripts/lint-tokens.ts` pour exclure explicitement `branding.couleur_primaire`** : c'est une donnée client, pas un style d'application — sans exclusion nommée, la porte échoue à tort et le réflexe sera de la désactiver sur le fichier. **Ajouter en contrepartie l'assertion qui remplace le signal supprimé** : la couleur d'identité visuelle n'apparaît que dans le rendu de document de test, dans aucun composant de `G1` (FR-059).

**Point de contrôle** : le produit a son premier écran, vérifié dans les deux thèmes et dans les
deux langues.

---

## Phase 10: US7 — Les deux tenants de démonstration portent la configuration réelle (P0)

**Objectif** : un rechargement en une commande restitue exactement le même état.

**Test indépendant** : trois rechargements successifs sur une base non vierge, état final identique.

- [ ] T045 [US7] Étendre `backend/migrations/seeds/` et `backend/api/src/bin/seeds.rs` — identifiants fixes, `ON CONFLICT DO NOTHING`, exécution **sous le rôle applicatif avec pose du tenant courant**, jamais sous le propriétaire. Deloria : classement non classé, commune d'Abengourou, `Africa/Abidjan`, `XOF`, **cinq services actifs**, capacité `STOCK` au profil `SIMPLE` déclarée par `RESTAURATION` et `BAR`. Résidence Test : **`HEBERGEMENT` seul, aucune capacité, aucun point de vente**. Mettre à jour `backend/migrations/seeds/README.md`.
- [ ] T046 [US7] Étendre `backend/tests/seeds_rejouables.rs` — trois exécutions, état final identique, aucune ligne dupliquée, aucun troisième établissement. Vérifier la configuration exacte de chacun des deux tenants et l'**absence** de `MODULE_FICTIF_TEST`.

**Point de contrôle** : la démonstration se recharge en une commande et les deux établissements ne
se ressemblent pas.

---

## Phase 11: Portes, documentation et validation transverse

- [ ] T047 Exécuter `cd backend/api && cargo sqlx prepare --workspace -- --all-targets` puis `--check`, et **comparer le nombre de requêtes mises en cache au nombre réel de requêtes du dépôt** (porte P-18). Le cycle 001 en validait 43 sur 47 : le décompte se lit, il ne se suppose pas.
- [ ] T048 [P] Exécuter les portes de structure et corriger tout écart — `scripts/ci/migrations-figees.sh` (P-02), `cargo test --test architecture` (P-03, P-12), `scripts/ci/jointures-inter-schemas.sh` (P-04), `scripts/ci/outbox-sans-purge.sh` (P-05b), **`cargo test --test rls_catalogue` (P-07, ré-exécutée après la dernière migration — T008 ne portait que sur les quatre référentiels de la Phase 2)**, `scripts/ci/maquettes-non-copiees.sh` (P-19), `scripts/ci/versions-epinglees.sh` (P-20).
- [ ] T049 [P] Vérifier la porte P-13 des deux côtés — `backend/tests/classes_offline.rs` pour les onze entités de classe C, et `app/tests/file-classe-a.spec.ts` en confirmant que **`TYPES_CLASSE_A` de `app/core/sync/classes.ts` n'a reçu aucun type de ce cycle**. Les onze entités sont C : aucune n'est mise en file locale.
- [ ] T050 [P] Reporter dans `docs/module-dore.md` la règle générale issue de ce cycle : **aucune migration n'écrit de données par `INSERT` ou `UPDATE` sur une table en `FORCE ROW LEVEL SECURITY`** — ce qui doit être écrit passe par le DDL (`DEFAULT`) ou par la mécanique de seeds, qui pose le tenant courant. Ajouter la note sur l'ordre `INSERT` avant `ENABLE`/`FORCE` pour un référentiel global.
- [ ] T051 **Recollement des trois portes à décompte** — écrire `backend/tests/couverture_portes.rs`, qui compare pour chacune le nombre de cibles réellement inspectées au total attendu et **échoue sur tout écart** : **P-05** — types d'événements couverts par `outbox_transactionnel.rs` contre les **11** déclarés à [data-model.md](data-model.md) § Événements (`module_capacite.declaree`, `parametre_configuration.ecrit` et `branding.modifie` n'étaient couverts par aucune tâche) ; **P-07** — tables inspectées par `rls_catalogue.rs` contre les **10 tables créées**, lues du catalogue système et non d'un nombre écrit à la main ; **P-08** — chemins couverts par `isolation_tenant.rs` contre les chemins servis par `application::contrat_complet()`, soit **21** (les opérations 10–11, 16–17 et 18–21 n'étaient couvertes par aucune tâche). Déclarer en tête le périmètre inspecté et ce qui ne l'est pas. **Une porte qui s'étend sur plusieurs phases laisse un trou par construction ; ce test est ce qui le referme.**
- [ ] T052 [P] Ajouter à `backend/tests/portes_a_vide.rs` les trois assertions des exigences que rien ne vérifiait — **FR-008** : aucun compteur d'établissements à visée tarifaire dans le code du cycle ; **FR-009** : aucune contrainte d'unicité ni clé étrangère n'empêche un compte d'être rattaché à plusieurs établissements (vérifié sur le schéma, le rattachement lui-même relevant de CPT) ; **FR-019** : aucune table ni colonne propre à `SALLE_REUNION`, qui reste une spécialisation d'hébergement sans entité nouvelle. Satisfaites par construction, elles n'étaient tenues par personne.
- [ ] T053 Dérouler l'intégralité de [quickstart.md](quickstart.md) sur une base repartie de zéro, dans l'ordre, et **consigner tout écart entre l'attendu et l'observé** plutôt que de corriger le document en silence. **Mesurer et consigner les deux valeurs chiffrées de la spécification** : activation d'un service constatée à l'écran (SC-008, cible 30 s) et affichage de l'aperçu d'identité visuelle (SC-009, cible 2 s). Un critère chiffré que personne ne mesure n'est pas un critère.

---

## Phase 12: Revue Definition of Done

- [ ] T054 Revue des **dix points** de la Definition of Done (`docs/user-stories-v1.md` §0.4), point par point, avec la preuve de chacun : (1) critères couverts par tests unitaires **et** d'intégration sur les transitions ; (2) annotations utoipa à jour, client régénéré sans diff manuel ; (3) migrations versionnées, `cargo sqlx prepare` vert, seeds à jour ; (4) RLS activée **et** forcée sur les **dix tables créées** — décompte lu du catalogue système, jamais écrit à la main — avec test d'isolation sur les **21 opérations** ; (5) classe hors-ligne déclarée pour les **onze entités**, tests correspondants ; (6) événement outbox pour les **onze types de transition** ; (7) clés i18n fr et en, aucune chaîne en dur, six entrées ajoutées au lexique ; (8) `G1` vérifié en clair et en sombre ; (9) `politique_impression` exposée dans la configuration et inscrite au récapitulatif ; (10) **document imprimé sur imprimante thermique — SANS OBJET**, l'aperçu d'ETB-05 est un rendu à l'écran et la première impression réelle relève du cycle IMP. Consigner ce point 10 explicitement dans `plan.md`, **jamais le cocher en silence**. Vérifier enfin que **FR-079 et FR-080 tiennent** : aucune table de provision créée (ETB-07, ETB-08), aucun sélecteur de contexte (ETB-06).

---

## Dépendances et ordre d'exécution

### Dépendances de phase

- **Phase 1 (préalables)** — aucune dépendance. **T001 bloque toute clé i18n** ; **T002 bloque toute migration** (le harnais doit être vert à vide avant que la première sentinelle apparaisse).
- **Phase 2 (fondations)** — dépend de la Phase 1. **Bloque toutes les stories.**
- **Phase 3 (US2)** — dépend de la Phase 2. Bloque US3, US5 et l'écran.
- **Phase 4 (US3)** — dépend de US2 (`etablissement_module` doit exister).
- **Phase 5 (US5)** — dépend de US2. **Doit précéder US4** : clé étrangère de `parametre_configuration` vers `point_de_vente`.
- **Phase 6 (US4)** — dépend de US5.
- **Phase 7 (US6)** — dépend de la Phase 2 seulement. **Parallélisable avec US3, US5 et US4.**
- **Phase 8 (US1)** — dépend de US2, US3 et US4 : ses quatre étapes livrables y sont branchées.
- **Phase 9 (écran)** — dépend de US2, US3, US5, US6 pour ses quatre sections.
- **Phase 10 (US7)** — dépend de US2 et US3 ; complète après US5.
- **Phases 11 et 12** — dépendent de tout ce qui précède.

### Dépendance inverse, écrite pour ne pas surprendre

**US1 est prioritaire mais dépend des autres stories.** C'est structurel : les trois parcours
exercent ce que US2, US3 et US4 produisent. Le harnais est donc écrit en Phase 1 — il surveille dès
le premier jour — et sa phase de clôture arrive en fin de cycle. La priorité de US1 se lit dans le
**moment où son harnais est écrit**, pas dans le moment où sa phase se termine.

### Parallélisation

- **T001 et T003** en parallèle ; **T002 juste après T001**, avant toute migration.
- **T007 et T008** en parallèle après T006.
- **T011 et T013** en parallèle après T010.
- **US6 (T034 à T036)** en parallèle de US3, US5 et US4 — elle ne dépend que de la Phase 2.
- **T031, T043, T048, T049, T050, T052** en parallèle.
- **T051 ne se parallélise pas** : le recollement des trois portes suppose que toutes les
  migrations, tous les points d'entrée et tous les événements du cycle existent. Le lancer plus tôt
  compterait juste et couvrirait faux.

```bash
# Après la Phase 2, deux fronts indépendants :
Front A : T010 → T011/T013 → T012 → T015/T016 → T019 → T023 → T027 → T028
Front B : T034 → T035 → T036          # identité visuelle, indépendante
```

---

## Stratégie de livraison

### Cœur minimal démontrable

Phases 1 à 4 (T001 à T022) : un établissement se crée, ses services s'activent, et une capacité non
implémentée est refusée par la base elle-même. **C'est déjà démontrable au pilote** — sans écran,
par le contrat d'API.

### Incrément suivant

Phases 5 et 6 (T023 à T033) : les points de vente et la résolution de configuration. Le cycle HEB
ne peut pas démarrer sans cette dernière.

### Cycle complet

Phases 7 à 12 : identité visuelle, clôture des trois parcours, écran `G1`, jeux de données, portes,
revue. **La démonstration de fin de tranche T1 exige l'ensemble.**

### Séquencement en développement solo

Une tâche par demi-journée à une journée, dans l'ordre des identifiants — l'ordre est déjà celui
des dépendances. Les cinquante-quatre tâches représentent environ **six à huit semaines-homme**, ce
qui dépasse la fenêtre S4–S8 que le §0.5 alloue à toute la tranche T1. Ce n'est pas un défaut du
découpage : c'est une information à porter à la revue de tranche, où l'arbitrage se fait — pas ici.

---

## Notes

- `[P]` = fichiers distincts, aucune dépendance sur une tâche inachevée.
- **Toute tâche qui touche le schéma commence par sa migration**, politiques RLS incluses.
- **Toute tâche qui touche l'API se termine par** annotations utoipa à jour, client TypeScript
  régénéré, diff commité, build vert.
- **Toute tâche qui crée une entité** inclut sa déclaration de classe hors-ligne au §5.1 de
  `docs/registre-classes-offline.md` et le test correspondant.
- Le patron des six couches est `docs/module-dore.md`. **Il se suit, il ne se réinvente pas** : ce
  cycle est le premier à le consommer comme entrée.
- **« Service » a deux sens dans ce cycle, et ils ne se mélangent pas** : la **couche applicative**
  (`service.rs`, entre le repository et le handler) et le **module d'activité vu par
  l'utilisateur** (« Vos services », lexique). Le premier sens ne vaut que dans les chemins de code,
  le second que dans les libellés d'interface. Un fichier nommé `services/service.rs` signalerait
  que la distinction a été perdue.
- Commit après chaque tâche ou groupe logique. **Le harnais des trois parcours n'est jamais commité
  rouge.**
