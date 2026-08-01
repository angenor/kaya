# Reprise du cycle 003 (CPT) — état au 2026-08-01, troisième session

*T001 à T059 livrées. Reste **T060 à T064** — le recollement, le cache sqlx, et la revue.*

---

## Où en est le cycle

**59 tâches sur 64.** Phases 1 à 9 complètes ; **phase 10 entamée** — T059 est livrée, et la
moitié « types d'événements » de T060 aussi.

**226 tests backend**, **426 tests front**. Portes vérifiées vertes cette session : P-01, P-04,
P-05b, P-15, P-16, P-17, P-19, P-21, P-21b.

### Ce que la troisième session a livré

| Tâche | Livré |
|---|---|
| T033 | `core/auth/` — quatre opérations, stockage web qui **déclare** sa garantie dans le type |
| T034 | `core/sync/vidage.ts` — **rafraîchir avant de vider**, inversion exercée |
| T035–T036 | Écran `R0` + son test ; le composant 16 gagne son type `mot_de_passe` |
| T037–T038 | `traits.rs` — `AccessController` et `AnnuaireComptes`, `ControleAccesPostgres` |
| T039 | `roles/service.rs` ; `changement_role` passe à « branché » au même changement |
| T040 | `api/src/securite.rs` — `exiger` et `exiger_ou_soi` |
| T041 | Les **neuf** opérations de comptes, rôles et référentiels ; `compte/service.rs` |
| T042 | `roles_cumules.rs` — 10 propriétés, dont celle qui discrimine les deux FR-023 |
| T043–T044 | Écran `G3`, `core/rbac` réel, les deux provisoires du cycle 002 levés |
| T045–T047 | `core/accueil/tuiles.ts`, écran `R1`, `permissions.spec.ts` |
| T048–T050 | Lecture filtrée du registre, opération 19, `audit_classe_a.rs` |
| T051 | Écran `G4` — registre sobre, horodatage d'autorité |
| T052–T054 | Les sept opérations de classe C, versant positif compris ; purge à la déconnexion |
| T055–T058 | Police d'icônes régénérée (77 glyphes), `core/erreurs/codes.ts`, thème, lint |
| T059 | `outbox_transactionnel.rs` — 15 tests, chaque type sur **deux** tenants |

### Six écarts et découvertes, tous consignés dans le code

1. **`compte.modifie` n'a aucun émetteur, et c'est tranché.** `data-model.md` déclare dix types
   pour ce cycle ; le contrat n'expose aucune opération de modification d'identifiant. Écart entre
   deux documents écrits en parallèle. `TYPES_SANS_EMETTEUR` dans `couverture_portes.rs` le nomme.
   **Total réel : 22 types d'événements** (13 + 9), pas 21.
2. **Le contrat sert 43 opérations, pas 40.** 1 sonde + 2 notes + 21 (cycle 002) + 3 personnes +
   6 session + 9 comptes/rôles/référentiels + 1 audit. Le plan en annonçait 40 ; la ventilation par
   lot de `couverture_portes.rs` est à jour et verte. **C'est le chiffre du plan qui était faux**,
   et T060 doit le constater plutôt que d'y toucher.
3. **`comptes` manquait à `SCHEMAS_APPLICATIFS`** de `classes_offline.rs` : les dix tables du cycle
   échappaient au balayage. Corrigé, avec décompte — **26 tables attendues**.
4. **`journal_audit.id` est une clé primaire GLOBALE**, pas par tenant. Découvert par le test de
   désordre de P-14, qui partageait trois UUID entre six tenants : la première permutation insère,
   les cinq autres tombent silencieusement sur `ON CONFLICT DO NOTHING`.
5. **`transition-[transform,border-color]` faisait capturer un faux jeton « color »** au contrôle
   du mode sombre. Les valeurs arbitraires de Tailwind sont retirées avant extraction.
6. **Les trois handlers de personnes n'avaient aucune garde de permission** — `securite.rs`
   n'existait pas à T023. Posées en T041.

### Trois portes ont attrapé du vrai avant la revue

- **P-08** a refusé les huit chemins nouveaux, en les nommant, puis a refusé le total.
- **P-21b** a listé sept icônes employées et non embarquées — une icône absente ne s'affiche pas,
  et rien d'autre ne le dit.
- **Le harnais de taxonomie** aurait fait échouer le build si `changement_role` n'était pas passée
  à « branché » dans le même changement que le code qui l'écrit.

---

## Ce qui reste — T060 à T064

### T060 · Recollement de `couverture_portes.rs`

**Sa moitié « types d'événements » est déjà faite** : les 9 types nouveaux sont déclarés, les
4 fichiers de service du cycle sont dans le balayage « émis par le code », et `TYPES_SANS_EMETTEUR`
porte `compte.modifie`.

**Ce qui reste** :

- `TABLES_CREEES` et le `recapitulatif_des_trois_portes_a_decompte` ne comptent que le schéma
  `etablissements` — la ligne `WHERE n.nspname = 'etablissement'` du récapitulatif est du cycle 002
  et ne voit pas `comptes` ;
- l'unicité des `operationId` (porte **P-01b**) n'est vérifiée nulle part : c'est le point que le
  plan désigne comme « risque réel », 19 identifiants nouveaux ;
- la cohérence de la taxonomie d'audit à recoller au récapitulatif ;
- **recompter plutôt qu'ajuster** : 22 types, 26 tables, 43 opérations. Les trois écarts au plan
  sont réels et documentés ci-dessus.

### T061 · `cargo sqlx prepare` — **DEUX passes, et ce n'est TOUJOURS pas fait**

`.sqlx` est en retard de toutes les requêtes des cycles CPT-00 à CPT-04. La procédure exacte est
dans `CLAUDE.md`, section « Versions » — la reproduire ici en ferait une seconde copie qui
dériverait. Deux points à ne pas redécouvrir :

- lancé depuis `backend/`, il perd les **binaires** `seeds` et `contrat` ;
- lancé depuis `backend/api/`, il perd les **tests d'intégration**.

Les deux moissons se conservent **hors** de `.sqlx` entre les passes, puisque chaque `prepare`
réécrit le répertoire entier. Puis les **deux** contrôles, dans cet ordre : `git status --short
backend/.sqlx` (aucune suppression), puis le check hors ligne.

### T062 · Client TypeScript

**Il est déjà à jour et P-01 est verte** — régénéré deux fois cette session, à T041 et à T051,
parce que les écrans `G3` et `G4` s'écrivent contre ces types. T062 doit donc **vérifier** plutôt
que régénérer : `pnpm porte:p01`, et constater que les 43 opérations figurent au contrat.

### T063 · `docs/module-dore.md`

Deux lignes à solder dans « Ce que ce patron ne démontre PAS » :

| Ligne | Ce qui la solde |
|---|---|
| « **Le RBAC réel** — permissions en configuration, provisoire nommé \| CPT-02 » | `core/rbac` lit `sessionCourante()?.permissions` ; `runtimeConfig` n'a plus ni `permissions`, ni `tenantId`, ni `compteId` |
| « **L'authentification** — contexte encore par deux en-têtes \| CPT-01 » | `ContexteAppel` porte un jeton ; `enTetesAuth` rend un seul en-tête |

Et **trois apports** à ajouter au patron : la garde de permission (`securite.rs` + absence dans le
HTML rendu), le stockage sécurisé par `PlatformAdapter` avec sa garantie déclarée dans le type, et
l'ordre **rafraîchir-avant-vider** de la file.

### T064 · Revue de la Definition of Done

Sur le modèle de `specs/002-etablissements-modules-activite/revue-dod.md`. Les dix points pour
chacune des six stories, **avec la preuve de chacun**. Le **point 10 est SANS OBJET** — ce cycle
n'imprime rien — et c'est à **consigner**, pas à cocher. Exécuter les treize vérifications de
`quickstart.md` et les 24 portes de bout en bout.

---

## Trois choses à savoir avant de relancer les tests

**1 · Un binaire `kaya-api` tournait pendant la session, et il fait échouer deux tests.**
`outbox_immuabilite.rs` échoue sur `sous_le_role_worker_seul_le_marquage_passe` et
`le_marquage_de_publication_passe_une_fois_et_une_seule` — **le message du test nomme lui-même la
cause** : un worker de développement consomme le grand livre toutes les 500 ms. Ce n'est pas une
régression du cycle. Vérifier avant de chercher ailleurs :

```sh
pgrep -fl 'target/debug/kaya-api'
```

**2 · Ne jamais lancer deux `cargo test --workspace` en parallèle.** Ils se partagent la base et
les sessions `kaya_worker`, et produisent exactement le même symptôme.

**3 · `SQLX_OFFLINE=true cargo check` échoue, et c'est attendu** jusqu'à T061.

---

## Vérifications d'entrée

```sh
docker compose -f infra/compose.yml up -d
pgrep -fl 'target/debug/kaya-api'    # doit ne rien rendre — sinon, arrêter le binaire
cd backend && cargo test --workspace              # 226 tests attendus, 0 échec
cd ../app && pnpm test                            # 426 tests attendus, 0 échec
cd .. && pnpm lint && pnpm porte:p01 && pnpm porte:p15 && pnpm porte:p21b
```

---

## Le prompt à donner en session neuve

```text
/speckit-implement Termine le cycle 003 (CPT) — T060 à T064. Les tâches T001 à T059 sont
livrées et commitées ; lis specs/003-comptes-roles-audit/REPRISE.md d'abord, il est à jour
au commit près.

Implémente T060 à T064 dans l'ordre. Après chaque tâche : compile, teste, commite avec un
message conventionnel référençant la story.

Cinq choses à ne pas redécouvrir :

- AVANT TOUT : `pgrep -fl 'target/debug/kaya-api'`. Si un binaire de développement tourne,
  son worker consomme le grand livre et DEUX tests de outbox_immuabilite.rs échouent — le
  message du test nomme la cause. Ce n'est pas une régression. Et ne jamais lancer deux
  `cargo test --workspace` en parallèle : même symptôme.
- `cargo sqlx prepare` exige DEUX passes, une depuis backend/ et une depuis backend/api/ —
  procédure exacte dans CLAUDE.md, section « Versions ». Ce n'est TOUJOURS PAS fait pour ce
  cycle, et le check hors ligne échouera tant que T061 n'est pas passée.
- Le client TypeScript est DÉJÀ à jour et P-01 est verte : il a été régénéré à T041 et à
  T051 parce que les écrans G3 et G4 s'écrivent contre ses types. T062 vérifie, il ne
  régénère pas.
- Trois décomptes du plan sont faux, et les écarts sont réels et documentés dans REPRISE.md :
  22 types d'événements et non 21, 43 opérations et non 40, 26 tables. `compte.modifie` est
  déclaré au modèle de données sans aucune opération qui le produise — c'est tranché en T059,
  `TYPES_SANS_EMETTEUR` le porte. RECOMPTER au recollement plutôt qu'ajuster un chiffre.
- La moitié « types d'événements » de T060 est déjà faite. Ce qui reste : le récapitulatif
  qui ne compte que le schéma `etablissements`, l'unicité des operationId (P-01b, jamais
  vérifiée nulle part), et la cohérence de la taxonomie.

À la fin, déroule la checklist de la Definition of Done — les dix points pour chacune des six
stories, avec la preuve de chacun — et liste ce qui resterait non conforme. Le point 10 est
SANS OBJET (ce cycle n'imprime rien) : le consigner, ne pas le cocher.
```
