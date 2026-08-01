# Implementation Plan: Comptes, rôles cumulables et journal d'audit

**Branch**: `003-comptes-roles-audit` | **Date**: 2026-08-01 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/003-comptes-roles-audit/spec.md`

---

## Summary

Troisième cycle, tranche T1. Il remplit la coquille `socle/comptes` du cycle 001 et solde **trois
dettes nommées** : la dérogation `CONTEXTE_PAR_EN_TETES`, dont la condition de levée est
littéralement « CPT-01 » ; les permissions en dur du cycle 002 (`bascule-service.ts`,
`etablissement.vue`), marquées « provisoire nommé, levé par CPT-02 » ; et l'accueil `R1`, que le
cycle 002 a explicitement reporté « au cycle CPT ».

**Dix tables** dans un schéma `comptes` nouveau, **cinq migrations**, **dix-neuf opérations HTTP**
(contrat porté à 40), **trois traits exposés**, **dix types d'événements outbox** (portés à 21),
**quatre écrans** — dont un, `R0` Connexion, qui **n'existe dans aucun document de référence** et
dont l'inscription à la matrice de dérivation est un préalable, pas une conséquence.

**L'approche technique en trois phrases.** Les sessions vivent en Redis parce que le registre les
classe « éphémère reconstructible » — aucune table, aucune sauvegarde, et une révocation qui prend
effet au rafraîchissement suivant. Les permissions effectives sont **l'union** des rôles, calculées
à la délivrance d'un jeton et portées par lui, jamais recalculées par requête. Le journal d'audit
est un **agrégat distinct de l'outbox** — classe A contre classe de l'opération tracée — et son
immuabilité tient par les privilèges de la table, pas par une convention.

---

## Technical Context

**Language/Version** : Rust **1.97.1** (toolchain gelée), TypeScript **5.9.3**

**Primary Dependencies** — toutes déjà gelées et **déjà déclarées** au workspace, aucune version
proposée ici : Actix Web **4.14.0**, sqlx **0.9.0**, utoipa **5.5.0**, `jsonwebtoken` **11.0.0**,
`argon2` **0.5.3**, `redis` **1.5.0**, Nuxt **4.5.1**, Tailwind **4.3.3**, Tauri **2.11.5**.

> **`jsonwebtoken` et `argon2` figurent au gel §3.1 avec la mention « (CPT-01) »** et sont déjà
> dans `backend/Cargo.toml` en épinglage exact. Ce cycle les **active** dans `socle/comptes` ; il
> n'ajoute **aucune dépendance nouvelle**, ni Rust ni JavaScript.

**Storage** : PostgreSQL **18.4**, schéma `comptes`, RLS `ENABLE` **et** `FORCE`. Redis **8.8.1**
pour les sessions et la limitation de débit — **éphémère reconstructible seulement**. Garage :
non touché par ce cycle.

**Testing** : `cargo test --workspace` (six fichiers d'intégration nouveaux, huit étendus),
`vitest` **4.1.10** côté application, portes de CI depuis la racine.

**Target Platform** : Docker sur VPS Contabo, **`linux/amd64`**. Poste de développement `arm64` —
`argon2` (RustCrypto) est du Rust pur, la chaîne cryptographique de `jsonwebtoken` porte de
l'assembleur par architecture : **les deux cibles sont annoncées supportées, et la première
tâche du cycle le constate par un `docker buildx build --platform linux/amd64`** plutôt que de le
citer (research R-16).

**Project Type** : monolithe modulaire Rust + application unique Nuxt 4 / Tauri v2.

**Performance Goals** : la connexion coûte un Argon2id (`m = 19456`, `t = 2`, `p = 1`), dimensionné
pour rester imperceptible sur la cible. Toute autre opération ne fait **aucune** lecture de
permissions : elles voyagent dans le jeton (research R-06).

**Constraints** : sept opérations de **classe C** — aucune atteignable hors ligne, jamais. Un
échec d'authentification est indiscernable **en temps** autant qu'en message. Aucun secret dans le
binaire ni dans le dépôt.

**Scale/Scope** : 10 tables · 5 migrations · 19 opérations HTTP · 3 traits · 10 types d'événements
· 4 écrans · 17 permissions · 8 rôles · 10 familles d'audit dont **2 branchées et 8 dues**.

---

## Constitution Check

*GATE : franchi avant la phase 0, re-vérifié après la phase 1. Aucune violation non justifiée.*

> **Note de périmètre.** Le prompt d'entrée parle de « vingt portes P-01 à P-20 ». La constitution
> du dépôt en compte **vingt-quatre** — P-01b, P-05b, P-21 et P-21b sont venues après. Les
> vingt-quatre sont traitées ci-dessous ; c'est la constitution du dépôt qui fait foi.

### Conformité aux douze principes

| # | Principe | Comment ce cycle le tient |
|---|---|---|
| I | Sources de vérité | Contrat produit par utoipa, client régénéré en CI. Cinq migrations **additives** — `0001` n'est pas touchée, le schéma `comptes` naît en `0014`. Durée de jeton et politique de mot de passe sont des **paramètres d'établissement**, portés au catalogue **et** au récapitulatif de `user-stories-v1.md` |
| II | Architecture et hiérarchie | Tout dans le schéma `comptes`. **Aucune clé étrangère inter-schémas** : `compte_role.etablissement_id`, `journal_audit.etablissement_id` et `permission.module_code` sont des colonnes nues, vérifiées par trait. Trois traits exposés, aucune jointure |
| III | Isolation multi-tenant | `ENABLE` + `FORCE` sur les **dix tables**. Les **quatre référentiels globaux** suivent le régime nommé de `0008` — deux politiques, `GRANT SELECT` seul — jamais une exemption |
| IV | Temps et disponibilité | `cree_le` d'autorité serveur partout ; `horodatage_client` indicatif, jamais employé pour trier ni pour dater une entrée d'audit. **Aucune occupation créée** |
| V | Argent et fiscalité | Une seule colonne monétaire — `employe.salaire_mineur`, **`BIGINT` d'unité mineure dès la provision**. `journal_audit.contexte` est du `JSONB` : tout montant qui y entrera sera un entier mineur, et **la porte est étendue avant le premier**, pas après |
| VI | Hors-ligne | Sept opérations **C**, une entité **A**. Les sessions ne sont pas classées — éphémère reconstructible (registre §9). UUID v7 client sur toute écriture, `200` sur rejeu. Refus **immédiat et explicite** hors ligne, jamais de grisé ni de file |
| VII | Application unique, rôles cumulés | **Le cœur du cycle.** Union des permissions, `BTreeSet` dans la signature du trait pour rendre la hiérarchie structurellement impossible. `R1` en tuiles filtrées, action interdite **absente**. Chargement paresseux par module |
| VIII | Qualité, i18n, observabilité | Six fichiers de tests d'intégration nouveaux. Quatre écrans en clair **et** sombre, clés fr/en. Les échecs d'authentification vont aux **journaux applicatifs**, jamais au grand livre |
| IX | Sécurité | **Le second cœur.** Argon2id à paramètres explicites, indiscernabilité **temporelle**, clé de signature hors binaire avec refus de démarrer, journal d'audit immuable par privilèges. **Aucune adresse MAC, nulle part** |
| X | Périmètre — prêt ≠ construit | CPT-05 et CPT-06 : **tables et colonnes, aucun privilège d'écriture, aucun endpoint**. Permissions des modules livrés seulement. Les huit types d'audit dus n'ont aucun chemin d'écriture |
| XI | Versions épinglées | Gel repris tel quel. **Aucune dépendance nouvelle** — les deux crates nécessaires y sont déjà, nommément « (CPT-01) » |
| XII | Référence visuelle | `R1` est **maquetté** en quatre états. `G3` et `G4` sont **dérivés**. **`R0` n'est ni l'un ni l'autre** — l'amendement de la matrice est une tâche, faite avant l'écran |

### Les vingt-quatre portes — mécanisme de vérification de chacune

**Dix-neuf portes sont touchées. Cinq restent vertes à vide.** Trois doivent être **étendues**,
faute de quoi elles laisseraient passer ce qu'elles sont censées attraper.

| Porte | Touchée | Mécanisme de vérification | Test |
|---|---|---|---|
| **P-01** client TS identique | ✅ 19 opérations | Génération puis `git diff --exit-code` | `scripts/ci/generer-client.sh --verifier` |
| **P-01b** `operationId` uniques | ✅ **risque réel** | 19 identifiants nouveaux, dont six préfixés `session_` et sept `compte_`. Unicité vérifiée sur le contrat complet | `backend/tests/couverture_portes.rs` |
| **P-02** migrations figées | ✅ 5 nouvelles | Empreinte comparée. `0001` **doit** rester intacte : le schéma naît en `0014` (research R-11) | `scripts/ci/migrations-figees.sh` |
| **P-03** socle ↛ verticales | ✅ | `socle/comptes` ne dépend d'aucune verticale. Les trois traits sont **définis et implémentés** dans le socle, consommés ailleurs | `backend/tests/architecture.rs` |
| **P-04** pas de jointure inter-schémas | ✅ **trois tentations** | `compte_role.etablissement_id`, `journal_audit.etablissement_id`, `permission.module_code` : trois colonnes qui « appellent » un `JOIN` vers `etablissements`. Aucune ne l'a. L'existence est vérifiée par `EstablishmentDirectory` et `RegistreModules` | `scripts/ci/jointures-inter-schemas.sh` |
| **P-05** événement dans la transaction | ✅ **10 types → 21** | Rollback provoqué par type : ni ligne métier ni événement. **Décompte comparé au total déclaré**, et **chaque type exercé sur les deux tenants** (exigence 5) | `backend/tests/outbox_transactionnel.rs` (étendu) + recollement |
| **P-05b** journaux sans purge | ⚠️ **à ÉTENDRE** | Le script ne lit aujourd'hui que l'outbox. Il gagne un **second contrôle** sur `journal_audit` — même recherche, périmètre déclaré en tête, **et son versant positif** : une entrée s'écrit et se relit (research R-10) | `scripts/ci/outbox-sans-purge.sh` **(étendu)** + `backend/tests/audit_immuabilite.rs` |
| **P-06** capacité ≠ `STOCK`/`SIMPLE` | ⬜ inchangée | Aucune capacité touchée. **Le patron du refus est réutilisé** pour `OTP_SMS` — clé étrangère composite sur `implementee` — mais c'est une porte distincte | `backend/tests/capacites_refusees.rs` |
| **P-07** RLS sur toute table | ✅ **10 tables → 26** | `relrowsecurity` **et** `relforcerowsecurity`, au moins une politique. Les quatre référentiels globaux comptés **conformes et nommés**. Ré-exécutée après la dernière migration, avec décompte | `backend/tests/rls_catalogue.rs` (étendu) + recollement |
| **P-08** isolation A/B par endpoint | ✅ **40 opérations** | Paramétré sur le contrat complet. **Les requêtes obtiennent un vrai jeton** — la refonte est le coût principal du cycle (research R-04). Les référentiels globaux rendent la même chose aux deux tenants, **affirmé explicitement**. Deux opérations publiques, liste nommée et fermée | `backend/tests/isolation_tenant.rs` (refondu) + recollement |
| **P-09** occupation GiST | ⬜ sans cible | Aucune occupation créée | `backend/tests/portes_a_vide.rs` |
| **P-10** montants entiers, quantités `NUMERIC` | ⚠️ **à ÉTENDRE** | `employe.salaire_mineur` en `BIGINT`. **Et surtout** `journal_audit.contexte` en `JSONB` : une remise, un écart de caisse, une rebascule y inscriront des montants. Le contrôle s'étend au champ **avant** le premier, comme il s'est étendu au catalogue au cycle 002 | `scripts/ci/types-monetaires.sh` **(étendu)** |
| **P-11** tests dorés fiscaux | ⬜ sans cible | Aucun calcul fiscal | `backend/tests/portes_a_vide.rs` |
| **P-12** fiscalité confinée | ✅ **une tentation** | L'indicatif `+225` par défaut est une donnée de **juridiction** : il est **paramètre d'établissement**, jamais une constante ni un `CHECK`. La validation E.164 est un format international, pas une règle nationale | `backend/tests/architecture.rs` |
| **P-13** aucune opération C hors ligne | ✅ **7 opérations** | Backend : aucun chemin d'écriture atteignable depuis la file locale, **avec décompte des opérations inspectées**. Front : `TYPES_CLASSE_A` ne reçoit aucun type du cycle, et le typage refuse la mise en file | `backend/tests/classes_offline.rs` + `app/tests/file-classe-a.spec.ts` |
| **P-14** rejeu triple d'une écriture A | ✅ **seconde entité A** | `journal_audit` : trois soumissions → **un** enregistrement ; six ordres → même état final, comparé en **ensemble trié** sur des identifiants **figés par permutation** | `backend/tests/audit_classe_a.rs` |
| **P-15** pas de `window.__TAURI__` hors adaptateur | ✅ **enjeu nouveau** | **Le jeton de rafraîchissement doit être stocké de façon sécurisée** — Keystore/Keychain sur mobile, stockage adapté sur web. Ce chemin passe **entièrement par `PlatformAdapter`** ; c'est le premier usage d'une capacité native par un écran. Règle ESLint exécutée par `pnpm lint` **depuis la racine** | `app/eslint.config.js` via `pnpm lint` |
| **P-16** i18n, parité fr/en | ✅ **4 écrans** | Parité des catalogues. **Chaque phrase passe par `docs/design/lexique.md` avant d'être codée** — ce cycle y ajoute le vocabulaire des comptes, des rôles et du journal | `app/scripts/test-i18n.ts` |
| **P-17** aucune couleur littérale | ✅ 4 écrans | Jetons seulement, variante `dark:` par personne — les noms de jetons sont identiques dans les deux thèmes, seules les valeurs changent | `app/scripts/lint-tokens.ts` |
| **P-18** `cargo sqlx prepare` vert | ✅ | `--check --workspace -- --all-targets`, **décompte des requêtes mises en cache comparé au total**, et vérification qu'aucune entrée n'a disparu | `scripts/ci/preparer-sqlx.sh` |
| **P-19** maquettes non copiées | ✅ **point critique** | Aucun fichier de `docs/design/html/` sous `app/`. **Et le versant amont** : `R0` doit figurer à la matrice de dérivation **avant** d'être codé — un écran absent des deux ne se code pas | `scripts/ci/maquettes-non-copiees.sh` + revue de tâche |
| **P-20** versions épinglées | ✅ **sans ajout** | Aucune dépendance nouvelle. Les deux crates nécessaires sont déjà au gel et au `Cargo.toml`, en épinglage exact | `scripts/ci/versions-epinglees.sh` |
| **P-21** aucune ressource externe | ✅ 4 écrans | Aucune police, icône, script ni image d'hôte externe sur les écrans nouveaux | `scripts/ci/aucune-ressource-externe.sh` |
| **P-21b** déclaré = embarqué | ⚠️ **à RÉGÉNÉRER** | La police d'icônes est **sous-réglée à 77 glyphes**. Les quatre écrans en emploieront de nouvelles. **Sans `pnpm icones:generer`, la porte échoue sur un glyphe employé mais absent** : c'est exactement ce qui a produit un écran sans icônes au cycle 002 | `scripts/ci/ressources-embarquees.sh` + `icones:generer --verifier` |

**Les trois portes à décompte** — P-05, P-07 et P-08 — passent de 11 types, 16 tables et
21 opérations à **21 types, 26 tables, 40 opérations**. La tâche de recollement de fin de cycle
compare, pour chacune, les cibles réellement inspectées au total déclaré. Elle ne se parallélise
pas : elle suppose que toutes les migrations, tous les points d'entrée et tous les événements
existent.

**Porte supplémentaire livrée par ce cycle** — cohérence de la taxonomie d'audit :
`backend/tests/audit_taxonomie.rs` compare l'énumération Rust à `docs/taxonomie-audit.md` dans le
sens **code → document**, et fait échouer le build si un type dû acquiert un chemin d'écriture
sans changer d'état. C'est le harnais à étapes dues du cycle 002, appliqué aux dix familles de
CPT-04.

### Tests hors-ligne obligatoires — §0.7 des user stories

| Classe | Entités | Test imposé | Où |
|---|---|---|---|
| **C** | `personne`, `compte`, `employe`, `compte_role`, `role`, `permission`, `role_permission`, `methode_authentification`, `appareil_enrole` | Échoue si l'opération est atteignable depuis un chemin exécutable hors ligne — **avec décompte** | `backend/tests/classes_offline.rs` · `app/tests/file-classe-a.spec.ts` |
| **A** | `journal_audit` | **Rejeu** — trois envois, un enregistrement · **Désordre** — six ordres, même état final | `backend/tests/audit_classe_a.rs` |
| **D** | aucune | Sans objet — consigné plutôt que passé sous silence | — |
| Scénario orphelin | aucune entité rattachée à un séjour | Sans objet — `sejour` n'existe pas avant SEJ | — |

**Isolation multi-tenant sur chaque endpoint** : les quarante opérations, sans exception.

### Écrans concernés

| Écran | Origine | Ce qu'il porte | Vérifications propres |
|---|---|---|---|
| **`R0` Connexion** | ⚠️ **inexistant** — à inscrire à `docs/design/derivation.md` : hérite de `G2`, états d'erreur de `S3` | Identifiant, mot de passe, une action | Les deux échecs affichent **la même phrase**. Refus immédiat hors ligne. Aucun stockage de jeton hors `PlatformAdapter` |
| **`R1` Accueil** | **Maquetté**, 4 états — `R1-accueil`, `-maquis`, `-proprietaire`, `-serveuse` | Tuiles filtrées par permission | Quatre comptes → quatre jeux. **Action interdite absente du HTML rendu**. Tuile issue de trois rôles présente **une fois**. Chargement paresseux constaté, pas déclaré |
| **`G3` Utilisateurs et rôles** | **Dérivé de `G2`** | Comptes, rôles portés, attribution et retrait | Classe C : hors ligne, l'action **disparaît** et un bandeau dit pourquoi. Validation **au champ** |
| **`G4` Journal d'audit** | **Dérivé de `R5` + `F2`** | Liste filtrable, registre sobre | Quatre filtres combinables. Horodatage **d'autorité** affiché |

**Aucun autre écran.** L'écran de note interne, dette du cycle 001, n'hérite toujours d'aucun
motif : il se maquette avant de se coder.

---

## Project Structure

### Documentation (this feature)

```text
specs/003-comptes-roles-audit/
├── plan.md                      # Ce fichier
├── spec.md                      # 6 stories · 48 exigences · 12 critères
├── research.md                  # Phase 0 — 17 décisions
├── data-model.md                # Phase 1 — 10 tables, 5 migrations, 10 événements
├── quickstart.md                # Phase 1 — 11 vérifications
├── contracts/
│   ├── http-api.md              # 19 opérations (contrat porté à 40)
│   └── traits-exposes.md        # 3 traits
├── checklists/requirements.md
└── tasks.md                     # Phase 2 — produit par /speckit-tasks
```

### Source Code (repository root)

```text
backend/
├── migrations/
│   ├── 0014_schema_comptes.sql              # CREATE SCHEMA + GRANT USAGE
│   ├── 0015_personne_compte.sql             # personne, methode_authentification, compte
│   ├── 0016_roles_permissions.sql           # role, permission, role_permission, compte_role
│   ├── 0017_journal_audit.sql               # journal_audit + 3 index de filtre
│   ├── 0018_provisions_rh_appareils.sql     # employe, appareil_enrole — SANS privilège d'écriture
│   └── seeds/                               # M. Koffi, Adjoua (3 rôles), Yao
├── crates/socle/comptes/src/
│   ├── lib.rs                               # remplace la coquille du cycle 001
│   ├── traits.rs                            # AccessController, AnnuaireComptes, JournalAudit
│   ├── personne/{modele,repository,service}.rs
│   ├── compte/{modele,repository,service}.rs
│   ├── roles/{modele,repository,service}.rs
│   ├── authentification/{argon2,service}.rs # hachage, indiscernabilité, rehachage
│   ├── session/{jeton,redis,service}.rs     # JWT + Redis — AUCUNE table
│   └── audit/{modele,repository,service,taxonomie}.rs
├── api/src/
│   ├── contexte.rs                          # REFONTE — le jeton remplace les deux en-têtes
│   ├── securite.rs                          # extracteur de permission, refus 403
│   └── routes/{session,personnes,comptes,referentiels,journal_audit}.rs
└── tests/
    ├── personne_compte_employe.rs           # NOUVEAU — CPT-00, le garde-fou
    ├── authentification_indiscernable.rs    # NOUVEAU — message, code ET temps
    ├── roles_cumules.rs                     # NOUVEAU — l'union
    ├── audit_immuabilite.rs                 # NOUVEAU — versants négatif ET positif
    ├── audit_classe_a.rs                    # NOUVEAU — rejeu, désordre
    ├── audit_taxonomie.rs                   # NOUVEAU — code → document, étapes dues
    └── (étendus) rls_catalogue · classes_offline · isolation_tenant ·
        outbox_transactionnel · couverture_portes · provisions_sans_logique ·
        portes_a_vide · architecture

app/
├── core/
│   ├── auth/                                # remplace la coquille : session, jetons, refus
│   ├── rbac/                                # remplace la coquille : union réelle
│   ├── platform/                            # stockage sécurisé du jeton — P-15
│   └── accueil/tuiles.ts                    # catalogue des tuiles et de leur permission
├── modules/
│   ├── comptes/{EcranComptes.vue,comptes.ts,roles.ts}      # G3
│   └── audit/{EcranJournalAudit.vue,journal.ts}            # G4
├── pages/
│   ├── connexion.vue                        # R0 — après amendement de derivation.md
│   ├── index.vue                            # R1 — remplace le placeholder du cycle 001
│   └── comptes.vue · journal-audit.vue      # chargement paresseux par module
└── tests/
    └── ecran-r0 · ecran-r1 · ecran-g3 · ecran-g4 · permissions · file-classe-a (étendu)

docs/                                        # documents normatifs modifiés par ce cycle
├── design/derivation.md                     # + ligne R0 — PRÉALABLE au codage
├── design/lexique.md                        # + vocabulaire comptes, rôles, journal
├── registre-classes-offline.md              # + methode_authentification, + journal §13
├── taxonomie-audit.md                       # NOUVEAU — 10 familles, 2 branchées, 8 dues
├── user-stories-v1.md                       # + 2 paramètres au récapitulatif
└── module-dore.md                           # 2 lignes de « ce que le patron ne démontre pas »
                                             #   soldées : RBAC réel, authentification
```

**Structure Decision** — aucune structure nouvelle. Le crate `socle/comptes` existe depuis le
cycle 001 et attend son contenu ; l'arborescence « un sous-module par story, trois couches
chacun » est celle de `socle/etablissements`, elle-même copiée du module doré. Deux sous-modules
s'en écartent, et c'est délibéré : `authentification/` et `session/` **n'ont pas de
`repository.rs`**, puisqu'ils n'écrivent dans aucune table — l'un fait du calcul, l'autre parle à
Redis.

---

## Phases

### Phase 0 — Recherche ✅ terminée

`research.md` — **dix-sept décisions**. Les cinq qui commandent le reste :

1. **R-01** — sessions en Redis, aucune table, révocation effective au rafraîchissement suivant.
2. **R-02** — l'indiscernabilité coûte un **hachage factice** ; sans lui, le temps de réponse
   publie la liste des comptes.
3. **R-04** — la dérogation `CONTEXTE_PAR_EN_TETES` est levée, et les tests des **21 opérations
   existantes** doivent obtenir un vrai jeton. C'est le coût principal du cycle.
4. **R-08** — le journal d'audit **n'est pas** un consommateur de l'outbox : deux registres, deux
   publics, deux classes.
5. **R-17** — **aucun point d'entrée d'écriture d'audit** : au MVP en mode A, une entrée voyage
   toujours avec l'opération qu'elle trace.

### Phase 1 — Conception ✅ terminée

| Artefact | Contenu |
|---|---|
| `data-model.md` | 10 tables, 5 migrations, politiques RLS, privilèges, 10 types d'événements, journal du registre |
| `contracts/http-api.md` | 19 opérations, `operationId`, permissions, 9 codes d'erreur métier |
| `contracts/traits-exposes.md` | `AccessController`, `AnnuaireComptes`, `JournalAudit` |
| `quickstart.md` | 11 vérifications + le parcours de démonstration en six gestes |

### Phase 2 — Tâches ⬜ à produire par `/speckit-tasks`

**Trois contraintes d'ordonnancement, sinon le cycle se bloque :**

1. **L'amendement de `docs/design/derivation.md` (`R0`) précède tout code d'écran.** Un écran
   absent de la matrice **ne se code pas** — la tâche s'arrête. C'est une tâche documentaire, elle
   coûte dix minutes, et son absence bloque quatre écrans.
2. **La refonte de `contexte.rs` et celle de `isolation_tenant.rs` vont d'un bloc.** Le jour où
   les en-têtes disparaissent, les 21 opérations existantes cessent d'être testables tant que la
   fonction d'aide de connexion n'existe pas. Les séparer laisse le dépôt rouge entre les deux.
3. **La régénération de la police d'icônes suit le choix des icônes et précède la porte.**
   `icones:generer` puis `--verifier` : un glyphe employé mais non embarqué fait échouer P-21b, et
   l'écran s'affiche sans icônes.

**Et une tâche de recollement en fin de cycle**, non parallélisable : décompte de P-05 (21 types),
P-07 (26 tables), P-08 (40 opérations), plus la cohérence de la taxonomie d'audit.

---

## Complexity Tracking

*Aucune violation de la constitution. Trois points de complexité réelle, justifiés ici pour qu'ils
ne soient pas « simplifiés » plus tard par quelqu'un qui n'aurait pas la raison sous les yeux.*

| Point | Pourquoi il est nécessaire | Alternative plus simple, et pourquoi elle est rejetée |
|---|---|---|
| **Deux registres — outbox et journal d'audit** | Classes différentes : l'audit est **A**, l'opération tracée garde la sienne. L'ouverture de tiroir se fait et se trace hors ligne ; l'outbox suit une transaction qui, elle, n'a pas eu lieu | *Dériver l'audit d'un consommateur outbox* : rendrait impossible de tracer une action de classe A faite hors ligne, et ferait dépendre le registre que le propriétaire achète du bon fonctionnement d'un worker |
| **Hachage factice sur compte inexistant** | FR-012 exige l'indiscernabilité **en temps**. Un `401` en 2 ms contre 90 ms publie la liste des comptes | *Se contenter du même message* : c'est la moitié de l'exigence, et c'est la moitié qui ne se voit pas en relecture |
| **Permissions dans le jeton plutôt qu'en base à chaque requête** | Deux lectures évitées sur **toute** opération du produit, et une seule source pour le filtrage des tuiles | *Relire à chaque requête* : coût permanent sur les chemins les plus chauds, et deux calculs de l'union qui divergeront. Contrepartie assumée : un rôle retiré prend effet au rafraîchissement suivant, ce que la révocation de session rattrape quand c'est urgent |

### Deux points appelant une décision avant `/speckit-tasks`

1. **La durée du jeton d'accès et la politique de mot de passe sont des paramètres
   d'établissement** (DoD 9) : ils doivent figurer au **catalogue** *et* au « Récapitulatif des
   paramètres d'établissement » de `docs/user-stories-v1.md`, sans quoi
   `backend/tests/parametres_catalogue.rs` — livré au cycle 002 — fait échouer le build. **Deux
   lignes à ajouter au récapitulatif**, pas une décision technique.
2. **Le texte de la porte P-05b ne mentionne que l'outbox.** Le contrôle sur `journal_audit` est
   livré par ce cycle ; l'extension du **texte** relève de `/speckit-constitution` et se fait
   séparément. Livrer le contrôle sans amender le texte est acceptable — l'inverse ne le serait
   pas.

### Definition of Done — le point 10 est SANS OBJET, et c'est écrit ici

Ce cycle n'imprime rien. Le point 10 — « tout document imprimé vérifié sur imprimante thermique
réelle » — est sans objet, consigné comme tel plutôt que coché à la légère. Les neuf autres
s'appliquent intégralement, y compris le point 8 (mode clair **et** sombre) sur les quatre écrans
et le point 9 (paramètres exposés) sur les deux paramètres nouveaux.
