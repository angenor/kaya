---

description: "Tâches — Cycle 003 · Comptes, rôles cumulables et journal d'audit"
---

# Tasks: Comptes, rôles cumulables et journal d'audit

**Input**: Documents de conception de `specs/003-comptes-roles-audit/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), [contracts/](contracts/), [quickstart.md](quickstart.md)

**Tests**: **obligatoires, non optionnels.** La Definition of Done les impose (§0.4, points 1, 4,
5), les portes P-05b, P-07, P-08, P-10, P-13 et P-14 n'existent que sous forme de tests, et
trois exigences de ce cycle — l'indiscernabilité temporelle (FR-012), l'ordre de vidage de la file
(FR-011c) et l'immuabilité du journal (FR-033) — décrivent des défauts **qui ne se voient pas en
relecture**.

**Organisation**: par story, **réordonnées par dépendance**. Trois écarts assumés, chacun motivé :

- **Le journal d'audit est scindé.** Son **écriture** (table, trait, service, immuabilité) est en
  Phase 2 — FR-024 impose qu'une attribution de rôle écrive une entrée, donc US3 en dépend. Sa
  **consultation** (endpoint, filtres, écran `G4`) reste dans US5. La priorité P2 de US5 se lit
  dans le moment où son écran arrive, pas dans celui où sa table est créée.
- **US4 (accueil `R1`) suit US3**, sans quoi elle n'aurait rien à filtrer.
- **US6 (invariants hors ligne) ne dépend que de la Phase 2** et se parallélise avec tout le reste.

## Format: `[ID] [P?] [Story] Description`

- **[P]** : parallélisable — fichiers distincts, aucune dépendance sur une tâche inachevée
- **[Story]** : US1 à US6, telles que numérotées dans [spec.md](spec.md)
- Chaque tâche porte ses chemins de fichiers exacts

## Conventions de chemin

Monorepo des cycles 001 et 002 : `backend/migrations/`, `backend/crates/socle/comptes/src/`,
`backend/api/src/`, `backend/tests/`, `app/`, `scripts/ci/`, `docs/`.

---

## Note d'ordonnancement — les quatre verrous du cycle

**1 · `R0` n'existe dans aucun document, et quatre écrans en dépendent.** La règle opposable de
`docs/design/derivation.md` (porte **P-19**) est sans appel : « un écran absent des deux **ne se
code pas** ». T001 est une tâche documentaire de dix minutes qui débloque toute la couche écran.
Elle est en tête, et rien d'autre ne la remplace.

**2 · La levée de la dérogation `CONTEXTE_PAR_EN_TETES` est UNE SEULE TÂCHE.** T028 refond
`contexte.rs` **et** `isolation_tenant.rs` dans le même passage. Les séparer laisserait les
**21 opérations existantes** non testables entre les deux : un dépôt rouge un soir, et on ne sait
plus si l'échec vient de la refonte ou d'un vrai défaut.

**3 · La liste de révocation vient avec l'extracteur de contexte, pas après.** Elle est consultée
à chaque requête authentifiée : la brancher plus tard supposerait de rouvrir le chemin le plus
chaud du produit une fois qu'il est déjà testé. Elle est dans le périmètre de T028.

**4 · Le harnais de la taxonomie d'audit est écrit avant la première migration**, comme celui des
trois parcours au cycle 002. Vert à vide (0 branché / 10 déclarés, 10 dus), il **oblige** chaque
branchement ultérieur : un type qui acquiert un chemin d'écriture sans changer d'état fait échouer
le build, pas une revue.

> **Aucun harnais n'est jamais commité rouge.** Une tâche qui crée une sentinelle et son
> branchement est **une seule tâche**, pas deux.

**Aucune tâche P1 produit dans ce cycle** : CPT-00 à CPT-04 sont toutes **P0**, CPT-05 et CPT-06
(P1) sont hors périmètre. Les priorités `P1`/`P2`/`P3` ci-dessous sont celles du modèle Spec Kit.

---

## Phase 1 : Préalables (documents normatifs et environnement)

**Objectif** : poser ce qui bloque le reste. Aucune ligne de Rust ni de Vue.

- [X] T001 Amender `docs/design/derivation.md` — ajouter la ligne **`R0` Connexion | `G2` | Formulaire minimal ; états d'erreur et vides de `S3`** à la matrice des écrans dérivés, incrémenter le décompte (31 → 32 écrans dérivés, 42 → 43 au total) et porter la version du document. **Cette tâche bloque T032, T040, T045 et T050** : sans elle, quatre écrans sont non codables au titre de la porte P-19. Vérifier ensuite que `G3` (→ `G2`) et `G4` (→ `R5` + `F2`) y figurent déjà — c'est le cas, et le constater évite de le redécouvrir en phase écran.
- [X] T002 [P] Ajouter le vocabulaire de ce cycle à `docs/design/lexique.md`, **avant toute clé i18n** : `compte` → « **Compte** » / *Account* ; `personne` → « **Personne** » / *Person* ; `role`/`compte_role` → « **Ce que chacun peut faire** » (règle déjà posée pour RBAC) ; `journal_audit` → « **Registre des actions** » / *Activity log* — jamais « journal d'audit », qui est le nom technique ; session → « **Appareil connecté** » / *Connected device* ; révocation → « **Déconnecter cet appareil** » / *Disconnect this device* ; échec d'authentification → **une seule phrase**, « Identifiant ou mot de passe incorrect » / *Incorrect ID or password*, employée dans les deux cas (FR-012) ; refus hors ligne d'une opération de classe C → réemployer la formulation d'ETB-02. **Les mots « rôle », « permission », « jeton » et « JWT » n'atteignent jamais l'interface.**
- [X] T003 Créer `docs/taxonomie-audit.md` — les **dix familles** de CPT-04 (`remise`, `annulation_ligne_envoyee`, `avoir`, `ouverture_tiroir`, `modification_tarif`, `suppression`, `changement_role`, `ecart_caisse`, `rebascule_palier_passage`, `forcage_disponibilite`), chacune avec son **état** (`branché` | `dû`) et, pour les dues, **la story qui la doit** : remise → PDV-03/SEJ-03 (T2), annulation de ligne envoyée → PDV-03 (T2), avoir → FIS-06 (T3), ouverture de tiroir → IMP-01 (T2), modification de tarif → PDV-01 (T2), écart de caisse → CAI-04 (T2), rebascule de palier → HEB-04 (T1), forçage de disponibilité → HEB (T1). **Puis écrire le harnais** `backend/tests/audit_taxonomie.rs` : comparaison **code → document**, décompte branchés/déclarés, échec si un type dû acquiert un chemin d'écriture sans changer d'état, et commentaire de tête déclarant le périmètre inspecté **et ce qui ne l'est pas** (§ « Couverture des portes »). **Vert à vide : 0/10 branchés, 10 dus.**
- [X] T004 [P] **Constater** la construction pour les deux architectures avant tout code applicatif : activer `jsonwebtoken` et `argon2` dans `backend/crates/socle/comptes/Cargo.toml` (versions **lues du workspace**, jamais reproposées — elles sont déjà épinglées à `=11.0.0` et `=0.5.3`), écrire un appel minimal de chacune, puis `docker buildx build --platform linux/amd64 -f infra/Dockerfile.api`. La chaîne cryptographique de `jsonwebtoken` porte de l'assembleur par architecture : **le constater maintenant coûte une heure, à la fin du cycle il coûte une semaine** ([research.md R-16](research.md)).
- [X] T005 [P] Poser les deux variables d'environnement et leur refus au démarrage dans `backend/api/src/main.rs` — `KAYA_JWT_CLE` (32 octets minimum) et `KAYA_SEEDS_MOT_DE_PASSE`, sur le **modèle exact de `verifier_derogation()`** qu'elles remplaceront. Mettre à jour `infra/compose.yml` et la documentation d'environnement. **Redis n'est plus optionnel** : consigner que la liste de révocation le met sur le chemin de chaque requête authentifiée.
- [X] T006 [P] Embarquer la liste des mots de passe compromis dans `backend/crates/socle/comptes/src/authentification/` — **fichier de données commité, jamais un appel réseau** ([research.md R-03](research.md)). Documenter la source et la date d'extraction dans un en-tête, comme les versions gelées le sont. Ce n'est **pas une dépendance de paquet** : aucune ligne n'est à ajouter à `docs/versions-gelees.md`, et la porte P-20 n'est pas concernée — l'écrire dans le commentaire de tête évite qu'on le redemande.

**Point de contrôle** : les quatre écrans sont codables, le vocabulaire existe, le harnais surveille, la construction `amd64` est prouvée.

---

## Phase 2 : Fondations (migrations, traits, écriture d'audit)

**⚠️ Bloque toutes les stories.** Les migrations de cette phase créent les sentinelles que le
harnais de T003 surveille.

- [X] T007 Migration `backend/migrations/0014_schema_comptes.sql` — `CREATE SCHEMA comptes` et `GRANT USAGE ON SCHEMA comptes TO kaya_app`, sur le modèle des trois schémas de `0001`. **`0001_roles_et_schemas.sql` n'est PAS modifiée** : elle est appliquée, et la porte **P-02** compare son empreinte au dépôt ([research.md R-11](research.md)). Ajouter le schéma là où sont les autres est le réflexe naturel et le plus coûteux.
- [X] T008 Migration `backend/migrations/0015_personne_compte.sql` — `personne`, `methode_authentification`, `compte` selon [data-model.md](data-model.md) §1 à §3. `methode_authentification` est un **référentiel global** : ordre impératif `CREATE TABLE` → `INSERT` (`MOT_DE_PASSE` implémentée, **`OTP_SMS` avec `implementee = false`**) → `ENABLE`/`FORCE` → `CREATE POLICY`, deux politiques (`lecture_universelle`, `administration_editeur`), `GRANT SELECT` seul. `compte` recopie `methode_implementee` et la contraint par **clé étrangère composite** — le refus d'`OTP_SMS` est structurel, pas un `CHECK` relâchable. RLS `ENABLE` **et** `FORCE` sur `personne` et `compte` ; privilèges `SELECT, INSERT, UPDATE`, **jamais `DELETE`**. Unicité `(tenant_id, identifiant_telephone)` et `(tenant_id, identifiant_email)`.
- [X] T009 [P] Déclarer au registre `docs/registre-classes-offline.md` §5.2 la ligne **nouvelle** `methode_authentification` — **classe C**, branche C2 (référentiel) — et ajouter l'entrée au journal §13 constatant que les neuf entités déjà déclarées de `socle/comptes` sont désormais implémentées. Le §5.2 a été écrit d'avance : **ne pas réécrire ses lignes existantes**, les honorer.
- [X] T010 Migration `backend/migrations/0016_roles_permissions.sql` — `role`, `permission`, `role_permission` (**trois référentiels globaux**, même régime que `0008`) et `compte_role`. Les **huit rôles** avec leur `portee` (`admin_editeur` en `EDITEUR`, les sept autres en `ETABLISSEMENT`) et les **dix-sept permissions** de [data-model.md](data-model.md) §6, `module_code` à `NULL` — aucun module d'activité n'a encore d'écran. `permission.module_code` **n'a pas de clé étrangère** vers `etablissements.module_activite` : ce serait une clé inter-schémas (porte P-04). `compte_role` porte `UNIQUE NULLS NOT DISTINCT (compte_id, role_code, etablissement_id)` — sans quoi `(compte, admin_editeur, NULL)` s'insère autant de fois qu'on veut. Privilèges `SELECT, INSERT, DELETE` sur `compte_role`, **pas d'`UPDATE`** : changer un rôle, c'est en retirer un et en attribuer un autre, deux actes, deux entrées d'audit.
- [X] T011 Migration `backend/migrations/0017_journal_audit.sql` — `journal_audit` selon [data-model.md](data-model.md) §8, avec sa **clé étrangère vers `comptes.compte`** — c'est elle qui rend FR-014 structurel : un compte désigné par une entrée d'audit ne peut pas être supprimé. Les **trois index de filtre** `(tenant_id, etablissement_id, cree_le DESC)`, `(tenant_id, auteur_compte_id, cree_le DESC)`, `(tenant_id, type_action, cree_le DESC)`. **`GRANT SELECT, INSERT` seulement** — ni `UPDATE`, ni `DELETE` : c'est le patron de classe A du module doré, et ici c'est ce qui tient l'immuabilité.
- [X] T012 Migration `backend/migrations/0018_provisions_rh_appareils.sql` — `employe` et `appareil_enrole`, **provisions**. RLS `ENABLE` + `FORCE` et politique d'isolation quand même (la porte P-07 ne connaît pas d'exception), mais **aucun privilège pour `kaya_app`, pas même `SELECT`** : un chemin de code écrit par distraction échoue au premier appel, pas trois mois plus tard. `employe.salaire_mineur` en **`BIGINT` d'unité mineure dès maintenant** (porte P-10) — le poser en `NUMERIC` « puisque personne ne s'en sert » imposerait de migrer toutes les lignes le jour de la paie. Coordonnées de `appareil_enrole` en `NUMERIC`, jamais en flottant. **Aucune colonne d'adresse MAC, et l'absence est commentée** comme une décision (FR-042).
- [X] T013 Ajouter les **cinq paramètres** au catalogue `etablissements.parametre_catalogue` par migration additive — `indicatif_telephonique_defaut` (`+225`), `methode_authentification` (`MOT_DE_PASSE`), `mot_de_passe_longueur_min` (`8`), `jeton_acces_duree_min` (`60`), `jeton_rafraichissement_duree_jours` (`90`). Ils figurent **déjà** au « Récapitulatif des paramètres d'établissement » de `docs/user-stories-v1.md` : `backend/tests/parametres_catalogue.rs` compare catalogue → récapitulatif et **fait échouer le build** sur toute clé absente du second. Vérifier qu'il passe au vert **après** cette migration, pas avant.
- [X] T014 [P] Étendre `backend/tests/classes_offline.rs` aux **dix tables** de ce cycle — comparaison table réelle → registre, **avec décompte des tables inspectées** face au total attendu. Vérifier qu'il **échoue** sur une table non déclarée en en retirant temporairement une du registre.
- [X] T015 [P] Étendre `backend/tests/rls_catalogue.rs` — **26 tables** (16 existantes + 10), `relrowsecurity` **et** `relforcerowsecurity`, au moins une politique. Les **quatre référentiels globaux** (`methode_authentification`, `role`, `permission`, `role_permission`) sont comptés **conformes et nommés**, jamais exemptés. Décompte inspectées / attendues.
- [X] T016 Écrire le trait `JournalAudit` et son implémentation dans `backend/crates/socle/comptes/src/audit/` — signature `tracer(&self, tx: &mut sqlx::PgTransaction<'_>, entree: EntreeAudit)` qui **prend la transaction et n'en ouvre jamais une**, exactement comme `OutboxWriter::ecrire` du cycle 001 : c'est la signature, pas la discipline, qui garantit que la trace et l'opération tombent ou passent ensemble. `TypeActionAudit` est une **énumération fermée** exposée en `ToSchema` — un `String` laisserait un cycle inventer `remise_appliquee` à côté de `remise`, et le filtre de `G4` cesserait de trouver la moitié des entrées sans que rien n'échoue. Brancher le harnais de T003 sur les deux types livrables.
- [X] T017 Implémenter la **validation du nommage monétaire** dans le service d'audit — toute clé `*_mineur` du `contexte` porte un **entier** et exige une clé `devise` au même niveau ([research.md R-19](research.md)). Constitution **1.6.0**, porte **P-10** étendue. Test : `{"ecart_mineur": -12500, "devise": "XOF"}` accepté ; `-12500.5`, `"12 500 F"` et un `ecart_mineur` sans `devise` **refusés**.
- [X] T018 [P] Étendre `scripts/ci/types-monetaires.sh` au `JSONB` — recherche des clés `*_mineur` avec valeur non entière, **et** des montants nommés `montant`, `prix` ou `total` nus dans un document JSON. Déclarer le périmètre inspecté en tête et compter les cibles. **Le contrôle statique ne voit pas un document construit dynamiquement** : c'est pourquoi T017 le double à l'écriture, et l'écrire ici évite qu'on croie l'un suffisant.
- [X] T019 [P] Étendre `scripts/ci/outbox-sans-purge.sh` à la **catégorie « registre immuable »** — constitution **1.6.0**, porte **P-05b** reformulée : le script inspecte l'outbox **et** `journal_audit`, déclare son périmètre en tête, et compte les registres examinés. Écrire `backend/tests/audit_immuabilite.rs` avec **ses deux versants** : négatif — aucun chemin de `DELETE`/`UPDATE`, et `kaya_app` n'a que `SELECT, INSERT` ; **positif** — une entrée s'écrit et se relit. Sans le second, supprimer la table suffirait à passer au vert.
- [X] T020 Seeds `backend/migrations/seeds/` — les personnes et comptes du pilote : **M. Koffi** (`proprietaire`), **Adjoua** (`gerant` + `caissier` + `receptionniste` — **les trois, c'est le point du cycle**), **Yao** (`receptionniste`). Identifiants UUID v7 **figés** et `ON CONFLICT (id) DO NOTHING` : des identifiants tirés au hasard rendraient les seeds non rejouables, ce que TRX-05a interdit. Mot de passe lu dans `KAYA_SEEDS_MOT_DE_PASSE`, et **refus d'exécution si l'environnement se déclare production**. Vérifier `backend/tests/seeds_rejouables.rs` après trois exécutions successives.

**Point de contrôle** : le schéma existe, l'audit s'écrit, les seeds tournent. Les stories peuvent commencer.

---

## Phase 3 : US1 — Trois tables distinctes, jamais confondues (P1) 🎯 MVP structurel

**Objectif** : `personne`, `compte` et `employe` ne se confondent jamais. C'est la seule story dont
l'échec ne se voit sur aucun écran et ne se rattrape pas.

**Test indépendant** : les trois figures — employé sans compte, compte sans contrat, les deux —
sur une base vierge, plus un contrôle statique des colonnes.

- [X] T021 [US1] Écrire `backend/tests/personne_compte_employe.rs` **avant l'implémentation** — les trois figures de CPT-00, et le **contrôle statique** qui échoue si une colonne de contrat, de salaire, de date d'embauche ou de numéro CNPS apparaît sur `compte` ou `personne` (FR-004), ou si un chemin de code lit `employe` pour décider d'un droit (FR-005). Le contrôle lit `information_schema.columns` et le graphe d'appels, **déclare son périmètre**, et compte les tables inspectées.
- [X] T022 [US1] Sous-module `backend/crates/socle/comptes/src/personne/` — `modele.rs`, `repository.rs`, `service.rs`, sur le patron des trois couches du module doré. `id` **fourni par le client** (UUID v7), `ON CONFLICT (id) DO NOTHING RETURNING` pour distinguer `201` de `200` sans second aller-retour. Deux horodatages distincts, `cree_le` faisant autorité. Événements `personne.creee` et `personne.modifiee` **dans la transaction**, et **uniquement si la ligne vient d'être créée** — un rejeu ne produit aucun nouvel événement.
- [X] T023 [US1] Points d'entrée `backend/api/src/routes/personnes.rs` — les trois opérations de [contracts/http-api.md](contracts/http-api.md) §7-9. Verbe et chemin **déduits de l'attribut Actix**, jamais répétés dans `#[utoipa::path]`. Montage par `service(...)`, jamais `route(...)`. **`type_piece` et `numero_piece` ne sont ni acceptés ni rendus** : leur alimentation relève de SEJ-01 et leur rétention de 90 jours de TRX-06. **Aucune liste de personnes** — la recherche de fiches client est SEJ-01.
- [X] T024 [US1] Étendre `backend/tests/provisions_sans_logique.rs` à `employe` et `appareil_enrole` — aucun privilège d'écriture pour `kaya_app`, **aucun point d'entrée d'API** ne les touche, et aucune colonne de pièce d'identité n'est écrite par ce cycle. Décompte des provisions inspectées.

**Point de contrôle** : une femme de ménage existe sans compte, un comptable externe se connecte sans contrat, et rien ne les confond.

---

## Phase 4 : US2 — Connexion, sessions et révocation immédiate (P1)

**Objectif** : Adjoua se connecte sur deux appareils, une session se coupe à distance
immédiatement, et deux échecs de connexion sont indiscernables.

**Test indépendant** : connexion réussie, deux échecs indiscernables en message, code **et temps**,
puis deux sessions dont une révoquée.

- [X] T025 [US2] `backend/crates/socle/comptes/src/authentification/argon2.rs` — Argon2**id**, `m = 19456` KiB, `t = 2`, `p = 1`, sel de 16 octets, sortie de 32, **paramètres écrits avec leur source** (recommandation OWASP) et portés par le condensat au format PHC. **Rehachage après vérification réussie** si le condensat lu porte d'autres paramètres — sans quoi une montée ne protégerait que les comptes créés après elle. Le **condensat factice** de référence est calculé **au démarrage**, pas à chaque requête.
- [X] T026 [US2] Politique de mot de passe dans `authentification/politique.rs` + test `backend/tests/politique_mot_de_passe.rs` — **8 caractères, aucune règle de composition, refus des mots de passe compromis** contre la liste embarquée de T006. Le test vérifie les trois cas qui comptent : 7 caractères refusé ; **`12345678` refusé bien qu'il fasse huit** ; `chaise-tomate-abidjan` **accepté** sans majuscule ni chiffre ni symbole. Le contrôle porte sur la **création et le changement**, **jamais sur la connexion** — refuser à la connexion un mot de passe devenu compromis enfermerait dehors un utilisateur légitime.
- [X] T027 [US2] `backend/crates/socle/comptes/src/session/` — jetons JWT signés par `KAYA_JWT_CLE`, **trois clés Redis** ([research.md R-01](research.md)) : `session:{session_id}` (90 jours), `revoquees:{session_id}` (60 min), `famille:{famille_id}` (90 jours). **Rotation à chaque usage** ; un jeton présenté une seconde fois révoque **toute la famille**, pas seulement celui-là — révoquer le seul laisserait le voleur et la victime en course, et le premier des deux gagnerait. Durées **lues du catalogue de paramètres** (T013), jamais des constantes.
- [X] T028 [US2] Service d'authentification `authentification/service.rs` + test `backend/tests/authentification_indiscernable.rs` — sur identifiant inconnu, **exécuter quand même la vérification Argon2** contre le condensat factice, puis rendre le refus commun `identifiants_invalides`. Le test lance **100 tentatives de chaque type** et compare les **médianes**, échec si le rapport sort d'un facteur 2 — un seuil en valeur absolue serait inutilisable, la CI n'a pas de temps stable. *C'est la moitié de l'exigence que le message identique ne tient pas.*
- [X] T029 [US2] Limitation de débit Redis dans `session/limite.rs` — fenêtre glissante sur **deux clés distinctes**, l'identifiant présenté **et** l'origine. Compter par identifiant seul laisse un balayage de mille comptes à une tentative chacun ; par origine seule, une attaque distribuée sur un compte unique. **Le refus reste indiscernable** : un message « trop de tentatives » sur un identifiant existant rétablirait exactement la fuite que T028 ferme. **Aucun verrouillage définitif** — ce serait un déni de service offert à qui connaît le téléphone d'Adjoua.
- [X] T030 [US2] **UNE SEULE TÂCHE, et c'est délibéré.** Refondre `backend/api/src/contexte.rs` — `ContexteAppel` extrait du **jeton vérifié** (tenant, compte, établissement actif, permissions effectives) **et consultation de la liste de révocation Redis à chaque requête** —, supprimer `verifier_derogation()` et `KAYA_CONTEXTE_PAR_EN_TETES`, **et** refondre `backend/tests/isolation_tenant.rs` pour que ses requêtes obtiennent un **vrai jeton** par une fonction d'aide de `backend/tests/commun` appelant le **vrai chemin de connexion**. Forger le jeton avec la clé de test ferait passer les tests sans jamais exercer l'authentification. **Les séparer laisserait les 21 opérations existantes non testables entre les deux** : un dépôt rouge un soir, et on ne sait plus si l'échec vient de la refonte ou d'un vrai défaut. La dérogation du cycle 001 est **levée** par cette tâche.
- [X] T031 [US2] Points d'entrée `backend/api/src/routes/session.rs` — les **six** opérations de [contracts/http-api.md](contracts/http-api.md). `session_ouvrir` et `session_rafraichir` sont les **deux seules opérations publiques du produit** : la liste est nommée et fermée, et le test d'isolation la connaît. Ordre de montage du plus spécifique au plus général : `/session/actives/{session_id}` avant `/session/actives` avant `/session`. `operationId` uniques (porte **P-01b**).
- [X] T032 [US2] `backend/tests/session_revocation.rs` — une session révoquée **cesse d'être acceptée à la requête suivante**, sans attendre les 60 minutes ; les autres sessions du compte continuent ; un jeton de rafraîchissement présenté deux fois révoque toute la famille et émet `session.revoquee` avec son entrée d'audit ; un changement de mot de passe révoque les autres sessions immédiatement.
- [X] T033 [US2] `app/core/auth/` — remplacer la coquille du cycle 001 : ouverture de session, rafraîchissement, refus. Le **stockage du jeton de rafraîchissement passe entièrement par `PlatformAdapter`** (Keystore/Keychain sur mobile, stockage adapté sur web) — premier usage d'une capacité native par un écran, et **la porte P-15 y est plus critique qu'ailleurs**. **Le front ne décode jamais le jeton** : les permissions viennent de la réponse de connexion, en clair. Lever le provisoire de `app/modules/etablissements/donnees.ts` (« contexte d'appel — provisoire, levé par CPT-01 »).
- [X] T034 [US2] File hors ligne dans `app/core/sync/` + test `app/tests/file-jeton-expire.spec.ts` — les écritures de **classe A** entrent en file **sans jeton**, et le retour du réseau **rafraîchit avant de vider**, jamais l'inverse ([research.md R-18](research.md)). L'échec du rafraîchissement **ne vide pas la file**. Le test simule une coupure de 90 minutes — une fois et demie la durée du jeton — et **échoue si l'ordre s'inverse, y compris quand les deux réussissent**. *En développement la coupure dure trente secondes et le défaut ne se manifeste pas : il perd un service entier à Abengourou.*
- [X] T035 [US2] Écran `R0` dans `app/pages/connexion.vue` — dérivé de `G2` (ligne ajoutée par T001), champs par le composant **`ChampSaisie`**, **une seule phrase** pour les deux échecs (T002), refus **immédiat et explicite** hors ligne. Patron d'écriture front de `docs/module-dore.md`, « La septième couche » : appel par le client généré, squelette de chargement, erreur traduite du `code` et jamais du `message`, validation **au champ**. Clair **et** sombre, clés `fr` et `en`.
- [X] T036 [US2] `app/tests/ecran-r0.spec.ts` — les deux échecs rendent **la même phrase** ; hors ligne, le refus est annoncé **avant** toute tentative ; aucune couleur littérale ; aucun `window.__TAURI__` hors adaptateur.

**Point de contrôle** : Adjoua se connecte, travaille sur deux appareils, et une session volée se coupe à la requête suivante.

---

## Phase 5 : US3 — Le cumul de rôles donne l'union (P1)

**Objectif** : trois rôles, une seule connexion, permissions = union. C'est le cœur du module.

**Test indépendant** : union exacte, retrait sélectif, et aucune élévation hors ligne.

- [X] T037 [US3] Sous-module `backend/crates/socle/comptes/src/roles/` + trait **`AccessController`** dans `traits.rs` — `permissions_effectives(compte_id, etablissement_id) -> BTreeSet<String>`. **`BTreeSet`, pas `Vec`** : le type dit l'unicité et l'ordre stable, et rend structurellement impossible la faute de FR-017 — un « rôle principal » dont les permissions primeraient. **Aucune signature n'accepte ni ne rend de rôle** : un consommateur qui branche sur un rôle recrée la hiérarchie que le principe VII interdit. Trait en anglais parce que `CLAUDE.md` le nomme ; les deux autres suivent le français.
- [X] T038 [US3] Trait `AnnuaireComptes` et son implémentation — `compte(id)` et **`comptes(&[Uuid]) -> BTreeMap`**, lecture en lot. Ce n'est pas une optimisation prématurée : `G4` affiche une page d'entrées d'auteurs différents, et sans lot l'écran ferait cent appels. `nom_affichage` vient de **`personne`**, jamais de l'identifiant de connexion — afficher un numéro de téléphone dans un registre à rétention illimitée diffuserait un contact personnel.
- [X] T039 [US3] Service d'attribution et de retrait dans `roles/service.rs` — l'existence de l'établissement est vérifiée **par `EstablishmentDirectory`**, jamais par clé étrangère inter-schémas, ce qui donne un `404 etablissement_inconnu` au lieu d'une violation de contrainte. Refus `409 derniere_habilitation` si le retrait laisserait l'établissement sans aucun compte habilité (FR-023). `422 portee_incompatible` pour un `etablissement_id` sur `admin_editeur` ou son absence sur un rôle d'établissement. Émission de `role.attribue` / `role.retire` **et** de l'entrée d'audit `changement_role`, **dans la même transaction** (FR-024).
- [X] T040 [US3] `backend/api/src/securite.rs` — extracteur de permission pour les handlers, `403 permission_absente`. **L'interface ne devrait jamais le provoquer** : une action sans permission est *absente*, pas refusée. Le code existe pour l'appel direct, pas pour le parcours normal.
- [X] T041 [US3] Points d'entrée `backend/api/src/routes/comptes.rs` et `referentiels.rs` — les **neuf** opérations restantes de [contracts/http-api.md](contracts/http-api.md) (§10-18). Ordre de montage : `/comptes/{id}/roles/{role_code}` avant `/comptes/{id}/roles` avant `/comptes`. Le **condensat n'est rendu sur aucune réponse, sur aucun chemin**. Les deux référentiels rendent **la même chose aux deux tenants** — l'affirmer explicitement dans le test d'isolation, sans quoi un référentiel global et une fuite se ressemblent.
- [X] T042 [US3] `backend/tests/roles_cumules.rs` — sur le compte d'Adjoua : union exacte des trois ensembles sans doublon ; retirer `caissier` **conserve** les permissions partagées et ne retire que les exclusives ; un compte sans rôle se connecte et obtient un ensemble **vide**, pas une erreur ; `admin_editeur` refuse un `etablissement_id`, les sept autres l'exigent.
- [X] T043 [US3] Écran `G3` dans `app/modules/comptes/EcranComptes.vue` + `app/pages/comptes.vue` — dérivé de `G2`. Comptes, rôles portés avec leur établissement, attribution et retrait. **Classe C** : hors ligne, **l'action disparaît et un bandeau dit pourquoi**, la garde vivant dans le module d'appel et **non dans le composant** — un second appelant oublierait de la reposer. Jamais de grisé, jamais de mise en file « au cas où ». Chargement paresseux.
- [X] T044 [US3] `app/core/rbac/` — remplacer la coquille par l'union réelle, et **lever le provisoire nommé** de `app/modules/etablissements/bascule-service.ts` (`PERMISSION_BASCULER`, « levé par CPT-02 ») : la permission `etb.service.basculer` vient désormais du référentiel. Test `app/tests/ecran-g3.spec.ts` — action **absente du HTML rendu** sans permission, et c'est le HTML qui est vérifié, pas la valeur d'un booléen.

**Point de contrôle** : Adjoua porte trois rôles et voit l'union ; Yao n'a que les siens ; aucun rôle ne s'attribue hors ligne.

---

## Phase 6 : US4 — L'accueil ne montre que ce qu'on a le droit de faire (P2)

**Objectif** : quatre comptes, quatre accueils, sur la même application. C'est la dette explicite
du cycle 002.

**Test indépendant** : quatre connexions successives comparées aux quatre états maquettés de `R1`.

- [X] T045 [US4] `app/core/accueil/tuiles.ts` — catalogue des tuiles et de la **permission** qui les ouvre. Une tuile issue de plusieurs rôles n'apparaît **qu'une fois** (FR-027). Une tuile dont le module d'activité n'est pas activé dans l'établissement est **absente**, pas grisée — réemployer `services-visibles.ts` du cycle 002 plutôt que de le réécrire.
- [X] T046 [US4] Écran `R1` dans `app/pages/index.vue` — remplacer le placeholder du cycle 001. **Maquette lue, jamais copiée** (porte P-19) : `docs/design/html/R1-accueil.html` et ses trois états `-maquis`, `-proprietaire`, `-serveuse`. Tuiles filtrées, **chargement paresseux par module** via `defineAsyncComponent`, comme `pages/etablissement.vue` au cycle 002. Un compte sans aucun rôle obtient un **état vide explicite**, pas une erreur.
- [X] T047 [US4] `app/tests/ecran-r1.spec.ts` et `app/tests/permissions.spec.ts` — quatre comptes → quatre jeux de tuiles ; **aucune action interdite dans le HTML rendu** ; la tuile d'Adjoua issue de trois rôles présente une seule fois ; un compte de serveur ne charge **aucun** morceau de module dont il n'a pas la permission — le chargement paresseux **se constate**, il ne se déclare pas. Lever le provisoire de `app/pages/etablissement.vue` (« permissions — provisoire nommé, levé par CPT-02 »).

**Point de contrôle** : la dette du cycle 002 est soldée ; l'application a enfin un accueil.

---

## Phase 7 : US5 — Le journal d'audit se consulte (P2)

**Objectif** : M. Koffi retrouve qui a fait quoi, sur quoi et quand, depuis son téléphone.
*L'écriture est en Phase 2 — voir la note d'organisation.*

**Test indépendant** : écrire des entrées, les relire depuis un terminal distinct avec les quatre
filtres, et prouver qu'aucune ne se modifie.

- [X] T048 [US5] Lecture filtrée dans `backend/crates/socle/comptes/src/audit/repository.rs` — filtres **combinables** par auteur, établissement, type d'action et période, pagination par curseur sur `(cree_le DESC, id DESC)`. Jamais de tri sur `horodatage_client` : trier sur l'horloge d'un terminal ferait remonter en tête l'entrée d'un appareil mal réglé. Auteurs résolus **en lot** par `AnnuaireComptes`.
- [X] T049 [US5] Point d'entrée `backend/api/src/routes/journal_audit.rs` — l'opération `journal_audit_lister`, sous permission `cpt.audit.consulter`. **Aucun point d'entrée d'écriture** ([research.md R-17](research.md)) : au MVP en mode A, une entrée voyage toujours avec l'opération qu'elle trace. En livrer un produirait une cible vide **et** une surface par laquelle un terminal forgerait des entrées dans le registre censé le surveiller. **Ni export ni alertes** — DIR-04, tranche T5.
- [X] T050 [US5] `backend/tests/audit_classe_a.rs` — **rejeu** : trois soumissions du même identifiant → **un** enregistrement, et **un seul** événement ; **désordre** : trois entrées dans les **six** ordres → même état final, comparé comme **ensemble trié** sur des identifiants **figés par permutation**. Tirés au hasard à chaque envoi, le test comparerait des jeux différents et ne dirait rien. Porte **P-14**, seconde entité de classe A du produit.
- [X] T051 [US5] Écran `G4` dans `app/modules/audit/EcranJournalAudit.vue` + `app/pages/journal-audit.vue` — dérivé de **`R5` + `F2`** : liste filtrable, registre sobre. Les quatre filtres combinables, l'**horodatage d'autorité** affiché — jamais celui du terminal. Sans la permission, la tuile est absente et l'accès direct refusé. Test `app/tests/ecran-g4.spec.ts`, clair et sombre.

**Point de contrôle** : ce que le propriétaire achète existe et se lit depuis n'importe quel terminal.

---

## Phase 8 : US6 — Le hors-ligne ne fabrique jamais un droit (P3)

**Objectif** : prouver l'invariante. Cette story ne se construit pas, elle se prouve.
**Parallélisable avec les phases 5 à 7** — elle ne dépend que de la Phase 2.

- [X] T052 [P] [US6] Étendre `backend/tests/classes_offline.rs` aux **sept opérations de classe C** du module — création de personne, création de compte, changement d'état, changement de mot de passe, attribution de rôle, retrait de rôle, révocation de session. Le test **déclare le nombre d'opérations réellement inspectées** face au total attendu, et porte **son versant positif** : chacune fonctionne **en ligne**. Une porte qui refuse sans vérifier ce qu'elle autorise passe au vert en n'ayant rien à inspecter.
- [X] T053 [P] [US6] Étendre `app/tests/file-classe-a.spec.ts` — `TYPES_CLASSE_A` ne reçoit **aucun** type de ce cycle, et le typage **refuse la mise en file** d'une opération C. Vérifier qu'aucune donnée de classe C n'est en **cache d'écriture** sur le terminal, que les référentiels y sont en lecture seule, et que le cache est **purgé à la déconnexion** — ce sont des données d'identité.
- [X] T054 [P] [US6] Vérifier le refus hors ligne sur `G3` — l'indisponibilité est annoncée **immédiatement et explicitement, avant toute saisie**, et l'état `degrade` est traité **comme** hors ligne pour une opération de classe C. `navigator.onLine` dit qu'une interface réseau est active, **pas que le serveur répond** : la garde évite l'attente inutile, elle ne remplace pas le traitement d'erreur.

---

## Phase 9 : Ressources, i18n et thème

- [X] T055 Choisir les icônes des quatre écrans, puis **régénérer la police** : `pnpm --filter @kaya/app icones:generer`, commiter les `woff2`, puis `--verifier`. **Sans cette tâche, la porte P-21b échoue sur un glyphe employé mais absent** — c'est exactement ce qui a produit un écran sans icônes au cycle 002, puis une application sur polices système de repli au volet suivant. L'ordre compte : choisir, générer, vérifier.
- [X] T056 [P] Compléter `app/core/i18n/{fr,en}.json` — toutes les clés des quatre écrans et des **dix codes d'erreur métier**, à parité stricte. `pnpm --filter @kaya/app test:i18n`. Chaque phrase est passée par `docs/design/lexique.md` (T002) **avant** d'être codée, jamais après.
- [X] T057 [P] Vérifier les quatre écrans en mode **clair et sombre** — aucune classe `dark:` dans les composants : les noms de jetons sont identiques dans les deux thèmes, seules les valeurs changent sous `.dark`. `pnpm --filter @kaya/app lint:tokens` (porte P-17) et `app/tests/theme-sombre.spec.ts` étendu aux nouveaux modules.
- [X] T058 [P] `pnpm lint` **depuis la racine** — porte **P-15**, avec le décompte des fichiers réellement analysés par arbre. Le stockage sécurisé du jeton est le premier usage d'une capacité native par un écran : vérifier qu'il ne franchit `PlatformAdapter` nulle part ailleurs.

---

## Phase 10 : Recollement, contrat et revue

**⚠️ Ne se parallélise pas.** Ces tâches supposent que toutes les migrations, tous les points
d'entrée et tous les événements du cycle existent. Les lancer plus tôt compterait juste et
couvrirait faux.

- [X] T059 Étendre `backend/tests/outbox_transactionnel.rs` aux **dix types nouveaux** (total **21**) — rollback provoqué par type : ni ligne métier ni événement. **Et chaque type exercé sur les deux tenants de démonstration** : c'est l'exigence 5 du § « Couverture des portes », née du défaut de séquence que la migration `0012` a corrigé et qu'aucune relecture n'avait vu. Vérifier que la connexion, le rafraîchissement et l'échec d'authentification **n'émettent rien** — ce ne sont pas des transitions d'état métier, et les inscrire au grand livre y écrirait la liste horodatée des présences du personnel.
- [X] T060 Recollement des portes à décompte dans `backend/tests/couverture_portes.rs` — **21 types d'événements, 26 tables, 40 opérations HTTP**, plus l'unicité des `operationId` (porte **P-01b**) et la cohérence de la taxonomie d'audit. Échec sur tout écart entre cibles inspectées et total déclaré. → **Livré, et deux des trois chiffres de cette ligne étaient faux.** Le recollement a **recompté** plutôt qu'ajusté : **22** types d'événements (13 + 9 ; le tableau du modèle de données porte deux types par ligne à deux endroits, et `compte.modifie` y est déclaré sans aucune opération qui le produise — `TYPES_SANS_EMETTEUR` le nomme), **43** opérations HTTP (le plan comptait des chemins ; `/api/v1/comptes` et `/api/v1/session` en servent deux chacun), **26** tables au total dont **20** créées par les cycles 002 et 003. Le décompte de tables ne voyait que le schéma `etablissements` : `TABLES_CREEES` porte désormais le couple `(schéma, table)`, et les schémas se déduisent de la liste — il n'y a plus de second endroit à mettre à jour. **P-01b n'avait aucune implémentation** — ni script, ni test — alors que la constitution la porte depuis le cycle 002 : ajoutée, avec son test négatif (doublon **et** chaîne vide, que `is_some()` laisserait passer). Le récapitulatif confronte son total à celui de `classes_offline.rs`, lu de son source. 8 tests, tous verts.
- [X] T061 `cargo sqlx prepare --workspace -- --all-targets`, puis **les deux contrôles dans cet ordre** : `git status --short backend/.sqlx` — **aucune suppression, que des ajouts** — puis `SQLX_OFFLINE=true DATABASE_URL= cargo check --workspace --all-targets --locked`. Le second seul ne suffit pas : un cache amputé d'une requête inutilisée par le check passerait. En cas de suppression, `git checkout backend/.sqlx` restaure sans toucher aux fichiers non suivis. → **Livré.** Deux passes, la moisson conservée hors de `.sqlx` entre les deux puisque chaque `prepare` réécrit le répertoire entier. Passe 1 depuis `backend/` : 39 ajouts, **12 suppressions** — les requêtes des binaires, exactement ce que `CLAUDE.md` annonce. Passe 2 depuis `backend/api/` : 25 ajouts, 39 suppressions. Fusion des deux moissons par-dessus les entrées commitées restaurées. **Contrôle 1** : 39 ajouts, **0 suppression**. **Contrôle 2** : `SQLX_OFFLINE=true DATABASE_URL= cargo check --workspace --all-targets --locked` vert. Recoupé par `scripts/ci/preparer-sqlx.sh --verifier` — **142 requêtes générées, 142 au cache** (130 du workspace + 12 des binaires) : aucune entrée périmée, la fusion manuelle tombe sur le résultat du script au fichier près.
- [X] T062 Régénérer le client TypeScript — `pnpm generer:client`, commiter, puis `pnpm porte:p01`. Vérifier que les **40 opérations** figurent au contrat et que le front compile : renommer un champ côté serveur doit faire échouer la **compilation du front**, pas produire un `undefined` invisible jusqu'à la démonstration. → **Le client était déjà à jour et P-01 verte — mais la propriété que cette tâche demande de vérifier était FAUSSE.** Le renommage d'un champ serveur, exercé pour de vrai par `#[serde(rename)]` sur `CompteVue.nom_affichage` puis régénération, laissait le front compiler sans une erreur. Cause : les quatre fichiers d'accès à l'API **redéclaraient les types à la main** et convertissaient les réponses par `as unknown as` — onze occurrences, le seul mécanisme de TypeScript qui relie deux types sans rapport. Les types sont désormais des **alias** de `components['schemas'][…]`, les onze conversions ont disparu, et l'expérience refaite fait bien échouer `tsc`. **Une divergence dormait déjà là** : le contrat porte `ServiceActif.active_le`, la copie manuelle ne l'avait pas — deux fixtures de test l'omettaient donc, et rien ne le disait. S'ajoute le versant que `satisfies` ne peut pas couvrir : une famille d'audit **ajoutée** au contrat et absente de `TYPES_ACTION` s'afficherait en brut ; un contrôle de type dans `ecran-g4.spec.ts` la refuse. 43 opérations au contrat et 43 dans le client, sans doublon. 428 tests front, 0 échec ; `pnpm lint` propre.
- [X] T063 Mettre à jour `docs/module-dore.md` — solder **deux lignes** de « Ce que ce patron ne démontre PAS » : « **Le RBAC réel** — permissions en configuration, provisoire nommé | CPT-02 » et « **L'authentification** — contexte encore par deux en-têtes | CPT-01 ». Ajouter la mention de ce que ce cycle apporte au patron : la garde de permission, le stockage sécurisé par `PlatformAdapter`, et l'ordre **rafraîchir-avant-vider** de la file. → **Livré.** Les deux lignes quittent « ne démontre PAS » pour un tableau « soldé par », avec la preuve de chacune : `core/rbac` lit `sessionCourante()?.permissions` et `nuxt.config.ts` n'a plus les trois clés ; `enTetesAuth` rend le seul en-tête `Authorization`. Les trois apports deviennent les **points 9, 10 et 11** du patron. S'ajoute un encadré au point 1, que T062 a rendu nécessaire : la phrase « renommer un champ côté serveur fait échouer la compilation du front » **était fausse pendant deux cycles**, et P-01 restait verte — un type consommé s'écrit `components['schemas'][…]`, jamais une interface qui lui ressemble.
- [X] T064 Revue de la Definition of Done — les **dix points** pour chacune des six stories, avec **la preuve** de chacun, sur le modèle de `specs/002-etablissements-modules-activite/revue-dod.md`. Le **point 10 est SANS OBJET** — ce cycle n'imprime rien — et c'est **consigné** plutôt que coché à la légère. Exécuter les treize vérifications de [quickstart.md](quickstart.md) et les **24 portes** de bout en bout. → **Livré : [revue-dod.md](revue-dod.md).** Les **24 portes sont vertes** et **12 des 13 vérifications** du quickstart passent. La treizième — les quatre écrans en clair et en sombre — **échoue**, et c'est la vérification en navigateur réel qui l'a trouvée : trois défauts d'intégration qu'aucun des 428 tests front ne pouvait voir, parce qu'ils montent les composants directement et contournent le routeur, `<Suspense>` et les coquilles de page. (1) La navigation vers une page paresseuse casse l'application — `TypeError: Cannot read properties of null (reading 'parentNode')`, reproduit sur `/comptes` **et** sur `/etablissement`, donc antérieur à ce cycle. (2) Un chargement direct de `/comptes` ou `/journal-audit` ne reprend jamais la session : `reprendreSession()` n'est appelé que par `pages/index.vue`. (3) `initialiserTheme()` n'est appelé nulle part — le produit n'applique **jamais** la classe `.dark`. **Conséquence : `G3` et `G4` sont inatteignables en navigateur**, par la tuile comme par l'adresse. `R0` et `R1` ont été vus dans les deux thèmes sur données réelles ; le styleguide rend ses 18 sections en clair et en sombre. **Le point 10 de la DoD est consigné SANS OBJET, non coché.**

---

## Dépendances et ordre d'exécution

### Dépendances de phase

- **Phase 1** — aucune dépendance. **T001 bloque quatre écrans** (T035, T043, T046, T051) ; **T002 bloque toute clé i18n** ; **T003 doit précéder la première migration** — le harnais doit être vert à vide avant que sa première sentinelle apparaisse ; **T004 précède tout code Rust nouveau**.
- **Phase 2** — dépend de la Phase 1. **Bloque toutes les stories.** T013 (paramètres) doit précéder T027, qui lit les durées du catalogue.
- **Phase 3 (US1)** — dépend de la Phase 2.
- **Phase 4 (US2)** — dépend de la Phase 2 et de US1 (`personne` avant `compte`).
- **Phase 5 (US3)** — dépend de US2 (le jeton porte les permissions) et de T016 (l'audit s'écrit).
- **Phase 6 (US4)** — dépend de US3 : sans permissions, l'accueil n'a rien à filtrer.
- **Phase 7 (US5)** — dépend de la Phase 2 pour l'écriture, de US3 pour la permission de consultation, de T001 pour l'écran.
- **Phase 8 (US6)** — dépend de la **Phase 2 seulement**. Parallélisable avec les phases 5, 6 et 7.
- **Phase 9** — dépend des quatre écrans.
- **Phase 10** — dépend de tout ce qui précède.

### Dépendance inverse, écrite pour ne pas surprendre

**US5 est P2 mais sa moitié « écriture » est en Phase 2.** C'est structurel : FR-024 impose qu'une
attribution de rôle écrive une entrée d'audit, donc US3 ne peut pas attendre US5. La priorité P2
se lit dans le moment où l'**écran** `G4` arrive, pas dans celui où la table est créée — exactement
comme US1 au cycle 002, dont le harnais précédait la phase.

### Parallélisation

- **T002, T004, T005, T006** en parallèle après T001 ; **T003 juste après**, avant toute migration.
- **T009, T014, T015** en parallèle après leurs migrations respectives.
- **T018 et T019** en parallèle — deux scripts de porte distincts.
- **US6 (T052 à T054)** en parallèle des phases 5 à 7 — elle ne dépend que de la Phase 2.
- **T056, T057, T058** en parallèle après T055.
- **T060 ne se parallélise pas** : le recollement suppose que tout existe.

```bash
# Après la Phase 2, deux fronts :
Front A : T021 → T022 → T023 → T025 → T026 → T027 → T028 → T030 → T031 → T037 → …
Front B : T052 → T053 → T054          # invariants hors ligne, indépendants
```

---

## Stratégie de livraison

### Cœur minimal démontrable

Phases 1 à 4 (T001 à T036) : **Adjoua se connecte, sur deux appareils, et une session se coupe à
distance.** La dérogation du cycle 001 est levée, l'API n'est plus ouverte à qui choisit son
tenant. C'est déjà démontrable — et c'est le premier écran par lequel un utilisateur entre dans le
produit.

### Incrément suivant

Phase 5 (T037 à T044) : le cumul de rôles et l'écran `G3`. À partir de là, le RBAC est réel et les
provisoires du cycle 002 sont levés.

### Cycle complet

Phases 6 à 10 : l'accueil `R1` — dette du cycle 002 —, le journal d'audit consultable, les
invariants hors ligne et le recollement. **La démonstration de fin de tranche T1 exige
l'ensemble** : sans `R1`, il n'y a pas d'écran d'entrée ; sans `G4`, il manque ce que le
propriétaire achète.

### Séquencement en développement solo

Une tâche par demi-journée à une journée, dans l'ordre des identifiants — l'ordre est déjà celui
des dépendances. **T030 est la tâche la plus lourde du cycle** : elle touche 21 opérations
existantes et ne se commite pas à moitié ; lui réserver une journée entière plutôt que de la
commencer un vendredi.

Les soixante-quatre tâches représentent environ **sept à neuf semaines-homme**, ce qui dépasse
la fenêtre que le §0.5 alloue à toute la tranche T1. Ce n'est pas un défaut du découpage : c'est
une information à porter à la revue de tranche, où l'arbitrage se fait — pas ici. Le constat est
le même qu'au cycle 002, et sa répétition est en soi une donnée.

---

## Notes

- **Aucune version n'est proposée par ces tâches.** `jsonwebtoken` **11.0.0** et `argon2`
  **0.5.3** sont au gel §3.1 avec la mention « (CPT-01) » et déjà épinglés dans
  `backend/Cargo.toml` : T004 les **active**, il ne les choisit pas. Ce cycle n'ajoute aucune
  dépendance, ni Rust ni JavaScript.
- **La liste des mots de passe compromis n'est pas une dépendance de paquet** mais un fichier de
  données commité (T006). Elle ne figure donc pas au gel, et la porte P-20 ne la couvre pas —
  écrit ici pour qu'on ne le redemande pas.
- **Deux portes ont été amendées par la constitution 1.6.0 à l'origine de ce plan** — P-05b sur la
  catégorie « registre immuable », P-10 au-delà de la frontière du `JSONB`. Elles sont
  **implémentées ici** (T017, T018, T019), pas proposées.
- **La décision ouverte O-01** — `personne` en classe C et le check-in d'un client inconnu hors
  ligne — reste ouverte et se tranche **avant SEJ-02**. Ce cycle livre `personne` en **C**
  conformément au registre, sans la préempter.
