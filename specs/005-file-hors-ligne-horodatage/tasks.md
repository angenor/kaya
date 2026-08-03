# Tasks: Classification hors-ligne, file d'actions et horodatage d'autorité

**Input**: Design documents from `/specs/005-file-hors-ligne-horodatage/`

**Prerequisites**: [plan.md](./plan.md), [spec.md](./spec.md), [research.md](./research.md),
[data-model.md](./data-model.md), [contracts/](./contracts/)

**Tests** : ce cycle **est** un cycle de tests pour une large part — le §0.7 des user stories et
les portes P-13/P-14 sont son objet. Les tâches de test ne sont donc pas optionnelles ici : elles
sont le livrable.

## Format: `[ID] [P?] [Story] Description`

- **[P]** : parallélisable (fichiers distincts, aucune dépendance sur une tâche incomplète)
- **[Story]** : US1 · US2 · US3 · US4, d'après [spec.md](./spec.md)

---

## ✅ Trois blocages posés, trois blocages levés

Levés avant l'implémentation, dans les documents qui font foi. Consignés ici parce que les
décisions se perdent autrement, et que trois tâches en dépendaient.

| # | Question | Réponse | Où |
|---|---|---|---|
| **B-1** | FR-034 exige une porte qui n'existait pas | **P-23 « Provenance de l'instant » ratifiée** — 26 portes | `.specify/memory/constitution.md` **1.8.0** |
| **B-2** | `S1` dérivait du « composant 8 », qui est la ligne de liste | **Composant 10**, témoin de synchronisation — corrigé avant ce cycle | `docs/design/derivation.md` **1.2.1** |
| **B-3** | Quatre formulations manquaient, et l'i18n contredisait le lexique | **Quatre entrées ajoutées** ; les trois libellés du témoin corrigés, avec la mention que `app/core/i18n` avait dérivé | `docs/design/lexique.md` **1.5.0** |

> **Ce que B-3 a produit et qui vaut d'être relu avant T031** : « Connecté » décrivait le réseau,
> « **Enregistré** » dit ce qui compte pour Aminata. La ligne « Synchronisation » du lexique porte
> désormais la mention explicite de la dérive passée — pour que la correction ne se redéfasse pas
> au prochain cycle.

> **Deux conséquences que P-23 impose au code, et qu'on lirait mal.** Ses exemptions sont
> **limitativement énumérées** : *ordre d'affichage local · détection de dérive d'horloge · rendu de
> l'instant tel que le terminal l'a perçu*. La couche de persistance qui **écrit** la colonne n'y
> figure pas, et n'a pas à y figurer — écrire une valeur n'est pas s'appuyer dessus. Et le titre de
> `S1` étant « Mes envois », **le fichier de page se nomme en conséquence** : le nom décide de la
> route, et `/synchronisation` afficherait un mot proscrit dans la barre d'adresse.

---

## Phase 1 : Préalables documentaires

**Objet** : mettre les documents normatifs en état avant que le code ne s'y adosse. Aucune de ces
tâches ne touche `app/` ni `backend/crates/`.

- [x] T001 ~~Lever B-2~~ — **fait avant le cycle** : `docs/design/derivation.md` **1.2.1**, `S1` → composant 10
- [x] T002 ~~Lever B-3~~ — **fait avant le cycle** : `docs/design/lexique.md` **1.5.0**, quatre entrées ajoutées
- [x] T003 Compléter `docs/design/lexique.md` — **fait** : version **1.5.1**, la **seconde forme de la dérive** (« **avance** de {n} minutes »), FR-035 portant sur la **valeur absolue** de l'écart et l'edge case de la spec exigeant les deux sens ; et les **formulations anglaises** des cinq libellés nouveaux, que le lexique ne donne pas encore
- [x] T004 [P] Ajouter la famille `derive_horloge_constatee` à `docs/taxonomie-audit.md`, contexte `{ ecart_secondes, seuil_secondes, sens }` — **aucune clé monétaire** (P-10 sur le JSONB)
- [x] T005 [P] Inscrire les deux paramètres au récapitulatif de `docs/user-stories-v1.md` §708 : `sync.derive_horloge_seuil_secondes` (défaut 300) et `sync.latence_degradee_seuil_ms` (défaut 3000)
- [x] T006 Porter `docs/registre-classes-offline.md` en **1.3.0** : §5.6 déclaré effectif, §11 réécrit pour dire que ses tests s'**instancient** désormais, journal des modifications — **aucune ligne d'entité ajoutée**, les deux tables du cycle y figurent depuis le 2026-07-30

**Point de contrôle** : les documents normatifs disent ce que le code va faire. B-2 et B-3 sont levés.

---

## Phase 2 : Fondations — découverte du périmètre et schéma

**⚠️ BLOQUANT** : aucune story ne démarre avant la fin de cette phase. Le module d'énumération est
consommé par dix fichiers de portes, et les deux migrations conditionnent tout le reste.

### Le module d'énumération partagé (FR-004b à FR-004d)

- [x] T007 Créer `backend/tests/commun/perimetre.rs` — `schemas_applicatifs()` lu de `pg_namespace` avec **liste d'exclusion nommée et justifiée** (`pg_catalog`, `information_schema`, `pg_toast*`, `public`, schéma des migrations), et `crates_du_socle()` / `crates_des_capacites()` / `crates_des_verticales()` lus des `[workspace] members` de `backend/Cargo.toml`. Exposer depuis `commun/mod.rs`
- [x] T008 Ajouter à `perimetre.rs` le **contrôle de non-régression** : le décompte de schémas et de crates échoue s'il **baisse** — un schéma disparu est soit une migration destructrice, soit un filtre devenu trop large
- [x] T009 Porter `backend/tests/classes_offline.rs` sur `perimetre::schemas_applicatifs()` — supprimer `SCHEMAS_APPLICATIFS` et `TABLES_ATTENDUES` en dur, conserver le décompte comparé à la découverte
- [x] T010 [P] Porter `backend/tests/architecture.rs` et `backend/tests/portes_a_vide.rs` sur `perimetre::crates_*()` — 4 des 21 chemins en dur
- [x] T011 [P] Porter `backend/tests/couverture_portes.rs` sur `perimetre::crates_*()` — 9 chemins en dur, le fichier le plus touché
- [x] T012 [P] Porter `backend/tests/audit_taxonomie.rs`, `authentification_indiscernable.rs` et `personne_compte_employe.rs` — les 8 chemins restants
- [x] T013 Porter `backend/tests/rls_catalogue.rs` sur le périmètre découvert et vérifier que **P-07 compte toutes les tables**, pas un sous-ensemble — c'est le trou du cycle 002 (4 tables sur 10)
- [x] T014 Vérifier que les **21 occurrences** de chemin de crate en dur sont ramenées à **zéro** (`grep -ro "crates/socle" backend/tests`), et que chaque fichier porté déclare son périmètre en commentaire de tête (exigence 1)

- [x] T014b Écrire dans `backend/tests/commun/perimetre.rs` le **contrôle prospectif de FR-004c** : un test qui échoue si un fichier de `backend/tests/` déclare un `const … : &[&str]` de schémas ou de crates sans passer par `perimetre::`. Sans lui, la règle « toute porte future en hérite » reste déclarative, et la porte n° 27 réintroduira une liste sans que rien ne le dise

> **Ce que cette phase va faire tomber, et qui n'est pas une régression.** Élargir un périmètre
> découvre du code que la porte ne voyait pas. Chaque échec est un **défaut trouvé**, à documenter
> et corriger — pas un effet de bord du cycle.

### Le schéma (2 migrations)

- [x] T015 Écrire `backend/migrations/0027_reconciliation_orpheline.sql` — table `synchronisation.reconciliation_orpheline` avec `id` client en clé primaire, deux horodatages, `CHECK` d'**égalité de conditions** sur le cycle de vie, index partiel `WHERE etat = 'constatee'`, **RLS `ENABLE` + `FORCE` + politique `isolation_tenant`**, et `GRANT SELECT` **seul** à `kaya_app`
- [x] T016 Écrire `backend/migrations/0028_parametres_synchronisation.sql` — 2 clés au catalogue sur le patron de `0023`, avec libellés et descriptions **en langue utilisateur**
- [x] T017 Étendre `backend/tests/provisions_sans_logique.rs` de **5 à 6** provisions ; vérifier que `kaya_app` **ne peut ni insérer ni modifier** `reconciliation_orpheline` — c'est ce qui prouve la provision (principe X)
- [x] T018 Étendre `backend/tests/isolation_tenant.rs` et `rls_catalogue.rs` à la table nouvelle : le tenant A ne lit aucune ligne du tenant B
- [x] T019 **Double passe `cargo sqlx prepare`** selon la procédure du quickstart §1, puis les **deux** contrôles : `git status --short backend/.sqlx` (aucune suppression) **puis** `touch` des fichiers à `sqlx::query` avant `SQLX_OFFLINE=true cargo check --workspace --all-targets --locked`

**Point de contrôle** : périmètre découvert, schéma en place, cache sqlx vérifié par les deux
contrôles. Les stories peuvent démarrer.

---

## Phase 3 : US1 — Aminata sait si son travail est en sécurité (P1) 🎯 MVP

**Objectif** : la file locale devient réelle — persistante, chiffrée, vidée au premier plan — et le
témoin dit vrai en permanence.

**Test d'indépendance** : couper le réseau dans un navigateur piloté, effectuer quatre écritures,
constater l'état et le nombre exacts, recharger la page, rétablir le réseau, repasser au premier
plan — les quatre arrivent, sans erreur de console. Quickstart §2.

### La plateforme

- [x] T020 [P] [US1] Ajouter `surRetourPremierPlan(rappel): () => void` à `app/core/platform/index.ts` et l'implémenter dans les quatre adaptateurs (`web.ts` : `visibilitychange` **et** `focus` ; `desktop.ts` : focus de fenêtre Tauri ; `android.ts` / `ios.ts` : reprise d'activité). **Rendre la fonction de désabonnement, jamais `void`**
- [x] T021 [US1] Alimenter l'état `degrade` dans `app/core/platform/reseau.ts` depuis un observateur d'appels — dernière issue et dernière durée, seuil lu de `sync.latence_degradee_seuil_ms`. C'est la ligne que le commentaire de tête du fichier annonce depuis le cycle 001

### La file

- [x] T022 [US1] Créer `app/core/sync/persistance.ts` — chiffrement **WebCrypto AES-GCM**, clé engendrée sur l'appareil et rangée dans `PlatformAdapter.stockageSecurise`, cryptogramme dans le stockage persistant ordinaire. **Aucune dépendance nouvelle**
- [x] T023 [US1] Étendre `EntreeFile` dans `app/core/sync/classes.ts` de `contexte` (tenant et établissement **figés à la saisie**) et `tentatives`. **Toujours aucun champ de jeton** — l'absence est ce qui l'empêche
- [x] T024 [US1] Rendre `FileLocale` persistante dans `app/core/sync/index.ts` : `ouvrir(adaptateur)` asynchrone, `enAttente`, `enQuarantaine`. **Aucun chemin de sortie autre que `viderFile`** — c'est ce qui porte l'ordre rafraîchir-avant-vider
- [x] T025 [US1] Créer `app/core/sync/envoi.ts` — quatre déclencheurs (retour au premier plan, passage à `connecte`, après écriture réussie, réessai à intervalle croissant plafonné). **Aucune minuterie de scrutation** : la batterie doit tenir un service
- [x] T026 [US1] Créer `app/core/sync/quarantaine.ts` — frontière par code de réponse selon [research.md](./research.md) R-10 ; `200` retire de la file (**rejeu réussi, pas un conflit**), `4xx` métier met en quarantaine, `5xx`/`408`/`429`/réseau réessaie
- [x] T027 [US1] Créer `app/core/sync/etat.ts` — `useEtatSynchronisation()`, source unique du témoin et de `S1`
- [x] T028 [US1] Créer `app/plugins/02.sync.client.ts` — `brancherFile` au démarrage, abonnement au retour au premier plan

### Les écrans

> **Décompte des écrans — `docs/design/derivation.md` v1.3.0 fait foi : le produit en compte 44**
> (11 codes maquettés / 29 fichiers d'états · 32 dérivés · 1 composé). Le chiffre de 43 est
> antérieur au cycle 004, qui a ajouté `G5` et ouvert la catégorie des écrans composés. **Ce cycle
> ajoute un écran** — `S1` figure déjà parmi les 32 dérivés. Total après ce cycle : **45**.

- [x] T029 [US1] Créer `app/core/design-system/TemoinSynchronisation.vue` — **composant 10** de `docs/design/composants.md`, « le composant le plus important du produit ». Trois états, une forme **et** une phrase chacun ; pouls lent (2,4 s) ; passage hors ligne **instantané, sans transition** ; **jamais de pourcentage**. Second composant Vue réutilisable du produit, après `ChampSaisie`
- [x] T030 [US1] Monter le témoin dans `app/layouts/default.vue` — présent sur **toutes** les pages, c'est ce que « indicateur permanent » veut dire. Vérifié en clair **et** en sombre
- [x] T031 [US1] Corriger les libellés `reseau.*` de `app/core/i18n/fr.json` et `en.json` d'après le lexique 1.5.0 — `connecte` → « **Enregistré** », `hors_ligne` → « **Hors connexion** », `en_attente` → « **En attente d'envoi ({n})** », `degrade` → « **Connexion faible** » ; ajouter les clés de la quarantaine et le bloc `horloge.*` à **deux formes** (retard et avance) plus sa phrase de rassurance obligatoire
- [x] T032 [US1] Créer `app/pages/notes.vue` — **ÉCRAN COMPOSÉ**, cas (c). Référence : les seize composants canoniques de `docs/design/composants.md` — **08** ligne de liste · **16** champ de saisie · **01·02·03** actions · **11** état vide illustré · **13** squelette de chargement. Emploie `notes_lister` et `notes_creer`, qui existent. Vérifié en clair **et** en sombre
- [x] T033 [US1] Inscrire `app/pages/notes.vue` à `docs/design/derivation.md`, tableau « Les écrans composés », avec la mention « **composé** · **à valider à l'atelier terrain** » et la **vérification des quatre conditions écrite dans la ligne** — liste et formulaire suivant un motif posé ; conception entièrement issue de la bibliothèque, vérifiée composant par composant ; note interne consultée rarement par un utilisateur formé ; aucun doute sur son apparence. **Zone de charme** : ni client en face, ni argent en jeu. Porter le fichier en 1.4.0 et le décompte de 44 à 45
- [x] T034 [US1] Créer `app/pages/mes-envois.vue` — écran **`S1`**, titre « **Mes envois** » (lexique 1.5.0). **ÉCRAN DÉRIVÉ**, cas (b). Référence : sa ligne de `docs/design/derivation.md` 1.2.1 — *« `S1` Panneau de synchronisation | **Composant 10** — témoin de synchronisation | Développement du composant : le témoin dit l'état d'un coup d'œil, le panneau détaille ce qui attend et permet d'agir »*. **Le nom du fichier décide de la route** : `/synchronisation` afficherait dans la barre d'adresse un mot que le lexique proscrit. File en attente et quarantaine, motifs en langue utilisateur branchés sur le `code`, **jamais sur le `message`**. Le geste `relancerDepuisQuarantaine` du contrat y est exposé
- [x] T035 [US1] Faire basculer les **deux marqueurs du cycle 003 dans ce même changement** : `brancherFile` de « dû par SYN-01 » à « branchée » dans `app/tests/amorcage.spec.ts`, et l'assertion de `app/tests/deconnexion.spec.ts` de « aucune file n'est branchée » à « la file est branchée **et vide** ». Sans cela le second test passerait **pour la mauvaise raison**

### Les tests de la story

- [x] T036 [P] [US1] Écrire `app/tests/file-persistance.spec.ts` — survit au **rechargement** et à l'extinction ; la charge est **illisible sans la clé** dans le stockage
- [x] T037 [P] [US1] Écrire `app/tests/temoin-sync.spec.ts` — **3 états × 2 thèmes × 2 langues**, soit douze combinaisons ; jamais de pourcentage
- [x] T038 [P] [US1] Étendre `app/tests/file-jeton-expire.spec.ts` — l'échec de rafraîchissement laisse la file **intacte** ; l'ordre rafraîchir-avant-vider tient **même quand les deux réussissent**
- [x] T039 [US1] Vérifier `pnpm porte:p22` et `porte:p22:negatif` — les deux écrans atteints **en direct et par navigation**, sur **Chromium et WebKit**, sans erreur de console

**Point de contrôle US1** : la file est réelle et son passager aussi. Quickstart §2 et §3 passent.
**C'est le MVP** — livrable seul.

---

## Phase 4 : US2 — Une action impossible hors ligne le dit avant la saisie (P1)

**Objectif** : l'invariante du principe VI devient opposable sur **les deux versants** — le type
qui refuse à la compilation, et l'écran qui annonce avant la saisie.

**Test d'indépendance** : réseau coupé, parcourir chaque écran d'écriture livré à ce jour ; toute
action B/C/D annonce son indisponibilité avant la saisie, aucune n'est mise en file. Quickstart §4.

- [ ] T040 [US2] Écrire `tests-e2e/hors-ligne.spec.ts` — périmètre **croisé** entre trois sources déjà existantes, aucune écrite à la main : opérations non-`GET` du contrat OpenAPI × classe du registre × routes de `tests-e2e/routes.ts`
- [ ] T041 [US2] Faire **rapporter** à `tests-e2e/hors-ligne.spec.ts` le nombre d'opérations B/C/D couvertes face au total du contrat, et échouer en **nommant** l'opération non couverte (exigence 2)
- [ ] T042 [US2] Déclarer en tête de `hors-ligne.spec.ts` la **limite assumée** : la porte vérifie qu'une annonce apparaît, **jamais que sa formulation est la bonne** — la justesse du libellé relève du lexique et de P-16, et les confondre donnerait une porte qui ment
- [ ] T043 [US2] Vérifier dans `tests-e2e/hors-ligne.spec.ts` que le balayage **n'écrit rien** (exigence 3) : seuls les écrans sont ouverts ; le seul geste d'écriture du parcours est la note interne, sur un tenant de test
- [x] T044 [US2] Étendre `app/tests/file-classe-a.spec.ts` — les `@ts-expect-error` couvrent le contexte et les tentatives nouveaux ; un enfilement d'opération non marquée **ne compile toujours pas**

**Point de contrôle US2** : P-13 est vérifiée sur ses deux versants, avec une cible comptée.

---

## Phase 5 : US3 — Adjoua clôture au franc près malgré une horloge fausse (P2)

**Objectif** : l'horodatage d'autorité devient la seule base admise, et la dérive est signalée sans
jamais bloquer.

**Test d'indépendance** : soumettre des écritures à horodatage volontairement décalé ; l'état
persisté porte un horodatage serveur cohérent, l'ordre d'affichage local reste celui du terminal.
Quickstart §5.

- [x] T045 [US3] Créer `backend/crates/socle/synchronisation/src/derive.rs` — `constater_derive(client, autorite, seuil) -> Option<Derive>` sur la **valeur absolue** de l'écart, et le trait `SignalDerive`. **Aucune dépendance** : `synchronisation` est le crate le plus bas, et `JournalAudit` vit dans `comptes` qui dépend de lui
- [x] T046 [US3] Câbler `SignalDerive` sur `JournalAudit` dans `backend/api/src/application.rs` — c'est la couche API qui connaît tout le monde, jamais `synchronisation`. Vérifier que **P-03 reste verte**
- [x] T047 [US3] Débrayer le signalement **par épisode** dans `backend/crates/socle/synchronisation/src/derive.rs`, via une clé Redis à durée de vie `(tenant, compte, appareil)` — deux cents écritures ne produisent pas deux cents entrées d'audit. La clé est **éphémère reconstructible** : la perdre produit une entrée de plus, jamais une donnée manquante
- [x] T048 [US3] Câbler le constat dans `backend/crates/socle/etablissements/src/note/service.rs` ; l'écriture est **acceptée** malgré la dérive (FR-036), et **aucun champ de réponse nouveau** n'est ajouté
- [x] T049 [US3] Porter `FAMILLES_ATTENDUES` de **10 à 11** dans `backend/tests/couverture_portes.rs` et ajouter le fichier de test à `TESTS_QUI_EXERCENT_L_AUDIT` — sans quoi le contrôle « toute famille branchée est exercée » échoue, et c'est ce qu'on attend de lui
- [x] T050 [US3] Écrire `backend/tests/derive_horloge.rs` — détection dans les **deux sens** ; acceptation malgré la dérive ; dix rejeux → **une seule** entrée d'audit ; famille exercée sur **les deux tenants de démonstration** (exigence 5)
- [x] T051 [US3] Créer `app/core/sync/horloge.ts` et avertir l'utilisateur depuis l'horodatage d'autorité de la réponse, **sans que le mot « dérive » ni aucune valeur technique n'apparaisse**. **Les deux sens sont dus** — retard et avance —, et la phrase « les durées et les montants restent calculés sur l'heure du serveur » est **obligatoire** : un avertissement qui inquiète sur ce qui va bien est pire que pas d'avertissement (lexique 1.5.0)
- [x] T052 [US3] Écrire `backend/tests/horodatage_autorite.rs` — **porte P-23**, périmètre découvert par `perimetre::crates_*()`. Les **trois exemptions sont celles de la constitution, à la lettre** : ordre d'affichage local · détection de dérive d'horloge · rendu de l'instant tel que le terminal l'a perçu. La liste est **close** — la persistance qui écrit la colonne n'en fait pas partie et n'en a pas besoin : écrire une valeur n'est pas s'appuyer dessus
- [x] T053 [US3] Installer `backend/tests/journee_avec_coupure.rs` **à vide**, avec son assertion de non-régression : la clôture journalière est de la tranche T3 ; le cycle qui la livrera trouvera ce test rouge, et c'est le but (SC-009)
- [x] T054 [US3] Nommer l'horodatage d'autorité dans `docs/module-dore.md` — `cree_le` fait autorité, `horodatage_client` ne porte aucune règle, et P-23 le vérifie. **Aucune colonne renommée** : ce n'est pas le nom qui manquait

**Point de contrôle US3** : SYN-04 est tenue, et la règle est gardée par une porte.

---

## Phase 6 : US4 — Un cycle suivant instancie les tests sans les réinventer (P2)

**Objectif** : les quatre familles de tests du §0.7 deviennent un outillage qu'on **instancie** en
une déclaration.

**Test d'indépendance** : réécrire les trois instanciations manuelles avec l'outillage, à
comportement inchangé et **sans perte de couverture**. Quickstart §6.

- [ ] T055 [US4] **Relever le décompte d'assertions** de `note_etablissement_classe_a.rs`, `audit_classe_a.rs` et `hebergement_hors_ligne.rs` **avant** tout portage, et le consigner. C'est le garde-fou du risque principal du cycle : une macro qui couvre moins transforme une réécriture en régression silencieuse
- [ ] T056 [US4] Créer `backend/tests/commun/classes.rs` — macro `tester_classe_a!` engendrant le **rejeu triple** (une ligne, **un** événement outbox) et le **désordre sur les six ordres**, en **six tests nommés** et non un test générique
- [ ] T057 [US4] Ajouter `tester_classe_bcd!` à `backend/tests/commun/classes.rs` — le test d'inatteignabilité hors ligne, et pour B le test de concurrence (deux exécutions simultanées, une seule réussit)
- [ ] T058 [US4] Ajouter `tester_classe_d!` à `backend/tests/commun/classes.rs` — double soumission au retour du réseau, **installée à vide** avec son assertion de non-régression : la certification FNE est de la tranche T3
- [ ] T059 [US4] Porter `note_etablissement_classe_a.rs` sur les macros et **comparer le décompte d'assertions** à T055
- [ ] T060 [US4] [P] Porter `audit_classe_a.rs` sur les macros, même comparaison
- [ ] T061 [US4] [P] Porter `hebergement_hors_ligne.rs` sur les macros, même comparaison
- [ ] T062 [US4] Créer `backend/tests/outillage_classes.rs` — parcourt le registre, en extrait toute entité **ayant une table réelle**, et échoue si elle n'a **aucune** instanciation correspondant à sa classe. Pendant exact de `classes_offline.rs` : celui-là vérifie qu'une classe est **déclarée**, celui-ci qu'elle est **exercée**
- [x] T063 [US4] Créer `app/tests/commun/classes.ts` — utilitaires du versant application : marque de classe, refus d'enfilement, annonce avant saisie

**Point de contrôle US4** : couvrir une entité nouvelle coûte une déclaration, et l'oublier fait
échouer le build.

---

## Phase 7 : Consolidation et revue

- [ ] T064 [P] Exécuter les **26 portes** une par une selon `specs/005-file-hors-ligne-horodatage/quickstart.md` §7, et consigner tout échec du portage de périmètre comme **défaut trouvé**, avec sa correction
- [ ] T065 [P] `pnpm lint` (racine, couvre `app/` et `web/`), `lint:tokens`, `test:i18n` — parité fr/en sur les clés nouvelles et corrigées
- [ ] T066 Reprendre la **double passe `sqlx prepare`** et les deux contrôles après toutes les modifications de requêtes, `touch` compris
- [ ] T067 Régénérer le client TypeScript dans `app/core/api/` (`pnpm generer:client`) et vérifier que **le diff est vide** — le contrat ne change pas, et c'est ce contrôle qui le prouve
- [ ] T068 Vérifier dans `app/tests/amorcage.spec.ts` les **deux preuves dues** pour chaque fonction d'amorçage (exigence 6) : `brancherFile` et `surRetourPremierPlan` ont un test qui les exerce **et** un test qui vérifie qu'elles sont appelées dans le parcours réel
- [ ] T069 Construire l'image de production `docker buildx build --platform linux/amd64 -f infra/Dockerfile.api` — le poste est `arm64`, jamais de copie d'un binaire local
- [ ] T070 **Revue Definition of Done** (`docs/user-stories-v1.md` §0.4) — les dix points, un par un, avec la preuve de chacun. Écrire `specs/005-file-hors-ligne-horodatage/revue-dod.md` sur le modèle du cycle 004, **y compris les points pris en défaut**

---

## Sur les priorités P0 / P1

**Ce cycle n'a aucune tâche P1.** SYN-01, SYN-02 et SYN-04 sont les trois **P0** du module
(`docs/user-stories-v1.md`, module SYN). La consigne « les tâches P1 en fin de liste » est donc
sans objet ici — et le dire vaut mieux que fabriquer une section vide pour respecter la forme.

Les priorités **US1 à US4** de [spec.md](./spec.md) sont un ordre de livraison interne au cycle,
pas les priorités produit : US1 et US2 y sont P1, US3 et US4 P2.

---

## Dépendances

```text
Phase 1 (docs)  ──► Phase 2 (périmètre + schéma)  ──►  US1 ──► US2
                                                    └─►  US3
                                                    └─►  US4
                                                          ↓
                                                      Phase 7
```

- **US1 est le MVP** et se livre seul.
- **US2** dépend d'US1 : le balayage en direct suppose la file branchée pour vérifier qu'aucune
  opération B/C/D n'y entre.
- **US3** et **US4** sont indépendantes d'US1 et l'une de l'autre — parallélisables une fois la
  Phase 2 finie.
- **T003** précède T031 et T051 — les formulations anglaises et la seconde forme de la dérive.
- **Aucun blocage externe ne subsiste** : B-1, B-2 et B-3 sont levés.

## Parallélisme

| Lot | Tâches | Condition |
|---|---|---|
| Documents | T002 · T003 · T004 · T005 | Après T001 |
| Portage du périmètre | T010 · T011 · T012 | Après T007–T008 |
| Tests d'US1 | T036 · T037 · T038 | Après T024–T028 |
| Portage des macros | T060 · T061 | Après T056–T058 et T055 |
| Consolidation | T064 · T065 | Après toutes les stories |

## Stratégie de livraison

1. **Phases 1 et 2** — incompressibles. Le périmètre découvert va faire tomber des portes ailleurs :
   c'est le but, et il faut du temps pour traiter ce qu'il révèle.
2. **US1 seule = MVP livrable.** Aminata a sa file, son témoin et son écran.
3. **US2** ferme l'invariante, **US3** l'horodatage, **US4** outille les cycles suivants.
4. **Phase 7** avant toute annonce de fin.

**Total : 71 tâches** — 6 documentaires (dont **2 déjà faites**), 14 de fondation, 20 pour US1,
5 pour US2, 10 pour US3, 9 pour US4, 7 de consolidation.
