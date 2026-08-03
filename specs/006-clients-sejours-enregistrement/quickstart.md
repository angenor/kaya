# Quickstart — valider le cycle 006

**Guide de validation exécutable.** Il ne contient aucun code d'implémentation : les migrations sont
en [data-model.md](./data-model.md), le contrat en [contracts/http-api.md](./contracts/http-api.md),
les traits en [contracts/traits-exposes.md](./contracts/traits-exposes.md).

**Ce qui est prouvé ici, dans l'ordre d'importance :**

1. un passage s'enregistre en **deux gestes** et **un appel** ;
2. **exactement une** arrivée concurrente réussit, **par la contrainte de la base** ;
3. le constat de taxe est **impossible à modifier**, même en SQL direct ;
4. un accompagnant tardif part en **réconciliation**, ni rejeté ni ajouté ;
5. la recherche répond **sous 300 ms** sur 10 000 fiches ;
6. les quatre routes s'ouvrent **en direct et par navigation**, deux moteurs, deux thèmes.

---

## 0 · Prérequis

```sh
docker compose -f infra/compose.yml up -d          # Postgres 18.4, Redis 8.8.1, Garage 2.3.0
bash scripts/dev/preparer-base.sh                  # rôles, schémas, migrations 0001 → 0034
bash scripts/dev/preparer-stockage.sh
```

> ⚠️ **La suite backend et le e2e ne coexistent pas.**
> `exiger_grand_livre_sans_consommateur_concurrent` refuse de dérouler les tests d'outbox quand un
> worker de publication tourne hors de `cargo test` — c'est-à-dire quand l'API est allumée.
> **Séquencer**, et arrêter l'API **par port** : `lsof -ti:8080 | xargs kill`.
> **Jamais `pkill -f`** — la commande a déjà tué le serveur de développement d'un autre projet de
> ce poste.

> ⚠️ **Le limiteur de tentatives punit les exécutions rapprochées.** Dix connexions par identifiant
> sur une fenêtre **glissante** de cinq minutes, réussies comprises, et chaque essai la repousse.
> Le refus est **indiscernable** d'un mot de passe faux : ne pas chercher ailleurs, attendre.

---

## 1 · Le budget du passage — le critère qui décide du produit

**C'est la première chose à vérifier, et la seule dont l'échec condamne le cycle.**

```sh
pnpm --filter @kaya/app test app/tests/budget-gestes.spec.ts
```

**Attendu** — trois assertions, toutes **déterministes** (aucune horloge de machine) :

| Assertion | Valeur |
|---|---|
| Interactions obligatoires du premier geste à la confirmation | **exactement 2** — la durée, puis la chambre |
| Champs de saisie libre obligatoires avant la remise de la clé | **0** |
| Appels réseau bloquants dans la même fenêtre | **au plus 1** |

> **Pourquoi ce test et pas un chronomètre.** *« Une mesure de latence en intégration continue
> dépend de la machine : elle rougirait au hasard, et serait désactivée dans le mois »* — leçon
> SC-004 du cycle 004. Un test désactivé ne garde rien ; il est **pire** qu'un test absent, parce
> qu'il donne l'illusion d'une garde. Ce que ce test attrape est la seule régression qui menace
> vraiment la cible : **l'ajout d'un champ « juste un de plus » au parcours**.

Puis la part machine, sur les deux moteurs :

```sh
pnpm exec playwright test tests-e2e/passage.spec.ts
```

Le budget est déclaré dans le fichier et fixé **très au-dessus** de la valeur observée.

**Enfin, le chronométrage humain — mesuré au terrain, jamais en CI** (FR-106) : du premier geste à
l'écran « C'est fait », sur le matériel de l'établissement, opérateur formé, client inconnu.
**Cible 30 s. Au-delà de 90 s, la story est en échec** — pas améliorable, en échec.

---

## 2 · Une seule arrivée réussit, et c'est la base qui le décide

```sh
cd backend
cargo test --test sejour_arrivee
```

**Le test qui compte** : deux arrivées concurrentes sur la même unité et des intervalles
chevauchants, deux transactions distinctes, insertion sans commit, puis commit des deux.

| Attendu | Pourquoi ça ne suffit pas de compter |
|---|---|
| **Exactement une** réussit | Un `SELECT … FOR UPDATE`, `SERIALIZABLE` ou un verrou applicatif donneraient le même compte |
| Le refus est un **`ExclusionViolation`** sur `occupation_sans_chevauchement` | **C'est la cause du refus qui est assertée**, pas son existence. Ces trois mécanismes rendraient la double attribution *improbable* au lieu d'*impossible* |
| Le refus vient du **parcours de séjour**, pas de l'endpoint nu du cycle 004 | Prouve que la transaction du check-in **n'a pas contourné la garantie** par une lecture préalable qui paraîtrait prudente |

**Et la transaction est-elle vraiment unique ?**

```sh
cargo test --test sejour_arrivee -- panne_apres_attribution
```

Attendu : après une panne simulée entre l'attribution et l'écriture de la note, **aucun** séjour,
**aucune** note, **aucune** fiche de police — et surtout **aucune occupation orpheline**, qui
rendrait une chambre indisponible sans qu'aucun séjour ne l'explique.

**Vérification manuelle du numéro de fiche de police** — il doit être continu **par établissement** :

```sql
SELECT etablissement_id, count(*), min(numero), max(numero)
FROM hebergement.fiche_police GROUP BY etablissement_id;
-- attendu : max = count pour chaque établissement, AUCUN trou
```

---

## 3 · Le constat de taxe est impossible à modifier

```sh
cargo test --test sejour_depart
```

**Le test qui compte** : après clôture, on modifie tout ce qui pourrait faire bouger le montant —
un accompagnant, le barème de la formule, `assujettie_taxe_nuitee`, le classement de
l'établissement, la commune — puis on relit le constat.

Attendu : **aucune valeur n'a changé.**

Puis la garantie de second rang, en SQL direct sous le rôle applicatif :

```sql
SET LOCAL app.current_tenant = '<tenant>';
UPDATE hebergement.taxe_sejour_constat SET nuits_constatees = 99;
-- attendu : ERROR — permission denied for table taxe_sejour_constat
DELETE FROM hebergement.taxe_sejour_constat;
-- attendu : ERROR — permission denied
```

> **Le figeage est un privilège, pas une intention.** `GRANT SELECT, INSERT` seuls. Le rôle
> applicatif **ne peut pas** modifier un constat, quelle que soit la ligne de code écrite au-dessus.
> Le test l'asserte tout de même : une garantie de privilège se perd en une ligne de migration.

**Et le montant reste absent** — c'est la preuve visible que ce cycle n'a écrit aucune règle
fiscale :

```sql
SELECT nuitees_assujetties, montant_mineur FROM hebergement.taxe_sejour_constat;
-- attendu : NULL, NULL sur TOUTES les lignes
```

```sh
cargo test --test portes_a_vide -- p11    # DOIT RESTER VERT
cargo test --test provisions_sans_logique # 5 provisions, 2 colonnes gardées
```

---

## 4 · L'accompagnant tardif — la première écriture orpheline du produit

```sh
cargo test --test sejour_orphelin
```

**Quatre assertions**, dans cet ordre :

1. accompagnant émis hors ligne, vidé **avant** la clôture → `201`, ajout normal ;
2. le même, vidé **après** la clôture → **`202`**, ni `201` ni `409` ;
3. une ligne existe dans `synchronisation.reconciliation_orpheline`, avec le séjour, l'entité, la
   charge utile et le motif ;
4. **le séjour clos est inchangé** : ni accompagnant ajouté, ni constat modifié.

> **`201` serait un ajout d'office. `409` serait un rejet silencieux.** Le principe VI interdit les
> deux : *« jamais de rejet silencieux, jamais d'ajout d'office »*. Le gérant tranchera — mais c'est
> **SYN-03, tranche T3**. Ce cycle **alimente** la file, il ne la vide pas, et `UPDATE` n'est pas
> accordé sur `reconciliation_orpheline`.

---

## 5 · La recherche de fiche client

```sh
cargo test --test client_recherche -- --nocapture
```

| Cas | Attendu |
|---|---|
| `kouame` | trouve **KOUAMÉ** — repli des signes diacritiques |
| `nguessan` | trouve **N'Guessan** *et* **N’Guessan** — apostrophe droite **et** typographique |
| `07123456` · `0712345678` · `+225 07 12 34 56 78` | trouvent la **même** fiche |
| Numéro de pièce avec espaces et tirets | trouve la fiche |
| Une personne **non qualifiée cliente** (le personnel) | **n'apparaît pas** |
| 10 000 fiches, 100 recherches par forme | **95ᵉ centile < 300 ms**, mesuré côté serveur |

> Le jeu de mesure est **généré par le test** et **jamais chargé dans les tenants de
> démonstration** (FR-007). Un tenant de démonstration à dix mille fiches rendrait toute
> démonstration illisible et toute exécution de seeds interminable.

---

## 6 · Les classes hors-ligne — les tests du §0.7

```sh
cargo test --test sejour_hors_ligne      # P-13 — 15 opérations refusées, 2 nommées comme classe A
cargo test --test outillage_classes      # échoue en NOMMANT une entité non instanciée
cargo test --test classes_offline        # toute table est déclarée au registre
```

| Classe | Entités | Ce qui est exercé |
|---|---|---|
| **A** | `accompagnant`, `preference_personne` | Rejeu triple → **un** enregistrement **et aucun second événement outbox** · six ordres → même état final |
| **B** | `sejour`, `note_sejour`, `ligne_sejour`, `fiche_police`, `numerotation_fiche_police`, `taxe_sejour_constat` | Inatteignable hors ligne · deux exécutions simultanées, une seule réussit |
| **C** | `client` | Inatteignable hors ligne · isolation multi-tenant sur l'endpoint |
| **D** | — | Aucune. `tester_classe_d!` **reste à vide**, et `outillage_classes.rs` le dit |

---

## 7 · Le parcours réel — les quatre routes

```sh
# Exige l'API, la base et les seeds. Le script le vérifie et le dit.
pnpm porte:p22
pnpm porte:p22:negatif      # prouve que la porte sait échouer
```

Attendu : `/passage`, `/arrivee`, `/clients`, `/depart` s'ouvrent **en chargement direct** *et*
**par navigation interne**, sur **Chromium et WebKit**, dans les **deux thèmes**, sans erreur de
console.

> **Rappel qui coûte cher à réapprendre** : *une page a UNE SEULE racine, et c'est un élément* —
> jamais un `v-if`/`v-else` de premier niveau. Une racine multiple compile en fragment ; un fragment
> dont la branche active est un `defineAsyncComponent` non résolu a un `el` nul, et Vue lève
> `Cannot read properties of null (reading 'parentNode')` à la navigation **suivante**. L'écran ne
> se monte pas, l'ancien reste affiché, et l'adresse a pourtant changé.

Puis le balayage hors ligne :

```sh
pnpm exec playwright test tests-e2e/hors-ligne.spec.ts
```

> ⚠️ **Le contrôle qui empêche ce test de mentir tient en une ligne** : vérifier que l'URL n'est
> **pas** `/connexion`. Le jeton d'accès vit en mémoire ; un rechargement exige le réseau pour
> reprendre la session, et hors ligne **toutes les routes renvoyaient sur `/connexion`** — neuf cas
> verts, neuf fois le même écran (cycle 005).

---

## 8 · Les portes, une par une

```sh
pnpm porte:p01 · p02 · p04 · p05b · p10 · p15 · p19 · p20 · p21 · p21b
pnpm lint · pnpm test:i18n · pnpm lint:tokens
cd backend && cargo test --test couverture_portes
cargo test --test horodatage_autorite       # P-23 — périmètre DÉCOUVERT, les crates nouveaux y entrent
cargo test --test architecture              # P-03 — aucune arête socle/ → verticales/
```

**Décomptes attendus** :

| Porte | Avant | Après |
|---|---|---|
| P-01b / P-08 — opérations | 56 | **73** |
| P-05 — types d'événements | 27 | **36** |
| P-07 — tables | 29 | **38** |
| `PLANCHER_TABLES` | 35 | **44** |
| Provisions sans logique | 6 | **5** |

### ⚠️ `cargo sqlx prepare` — deux passes, puis deux contrôles

Ce cycle écrit des requêtes **dans les seeds** (binaire) **et dans les tests d'intégration**. Une
passe unique en perdrait la moitié — la commande ne collecte que les cibles que son `cargo check`
compile réellement, et le répertoire d'où on la lance décide de ce qu'elle voit.

```sh
cd backend
rm -rf /tmp/sqlx-a /tmp/sqlx-b && mkdir -p /tmp/sqlx-a /tmp/sqlx-b

cargo sqlx prepare --workspace -- --all-targets            # passe 1 — tests
git status --short .sqlx | grep '^??' | awk '{print $2}' | xargs -I{} cp {} /tmp/sqlx-a/
git checkout .sqlx

(cd api && cargo sqlx prepare --workspace -- --all-targets)  # passe 2 — binaires
git status --short .sqlx | grep '^??' | awk '{print $2}' | xargs -I{} cp {} /tmp/sqlx-b/
git checkout .sqlx

cp /tmp/sqlx-a/*.json /tmp/sqlx-b/*.json .sqlx/
```

Puis les **deux** contrôles, dans cet ordre — **le second seul ne suffit pas** :

```sh
git status --short backend/.sqlx    # AUCUNE suppression ; que des ajouts

# ⚠️ Le `touch` n'est pas décoratif : sans lui, le check affiche `Finished` en une seconde
# SANS consulter `.sqlx` — les macros ne sont pas réévaluées, donc un cache vide passerait.
grep -rl "sqlx::query" --include="*.rs" crates api tests | xargs touch
SQLX_OFFLINE=true DATABASE_URL= cargo check --workspace --all-targets --locked
```

---

## 9 · La démo de fin de tranche T1

**Le critère de clôture du cycle, et de la tranche.** `docs/user-stories-v1.md` §0.5 :

> *« Yao enregistre un client en chambre B3 pour 2 nuits, puis un passage de 4 h en A1 — la
> disponibilité empêche tout chevauchement, tout est tracé. »*

```sh
bash scripts/dev/charger-seeds.sh          # une commande, idempotente
pnpm --filter @kaya/app dev
```

| Étape | Écran | Ce qu'on vérifie |
|---|---|---|
| 1 | `/clients` | La fiche de M. Bakayoko se trouve en tapant `bakay` |
| 2 | `/arrivee` | Client connu → **zéro champ ressaisi** ; unité proposée automatiquement ; deux accompagnants ajoutés |
| 3 | `/passage` | **Deux taps**, chambre A1 pour 4 h, écran « C'est fait » avec l'heure de fin |
| 4 | `/passage` | Retenter A1 sur un intervalle chevauchant → refus, **et le refus dit pourquoi** |
| 5 | `/depart` | La note nuit par nuit, le total, le constat figé — **et le montant de taxe absent**, `null` et non zéro |
| 6 | `/journal-audit` | La rebascule, la régularisation, le changement d'unité, avec auteur, instant et montant |

**Chacun des six en clair et en sombre.** Le point 8 de la Definition of Done ne se vérifie pas au
jugé.

---

## 10 · Ce qui reste dû à la sortie du cycle

| Dû | Où |
|---|---|
| **Lexique v1.6.0** — le vocabulaire du cycle, `fr` et `en` | **AVANT le code** (séquencement 1) |
| **Amendements de la décision B-10** — cadrage §9.6 et B-10, FIS-03, FIS-08, récapitulatif | **AVANT les migrations** (séquencement 2) |
| **O-01 tranchée** au registre §14 | Avant les migrations |
| Registre des classes §8 — 4 lignes ajoutées, journal v1.4.0 | Fin de cycle |
| `derivation.md` — `R3` et `R5` d'« inscrit » à « codé » | Fin de cycle |
| `taxonomie-audit.md` — famille 10 **reste « due »**, et le dire | Fin de cycle |
| Rétention 90 jours du numéro de pièce, export et suppression d'une personne | **TRX-06**, P1 — **dette nommée**, pas un oubli |
| Impression de la note et de la fiche de police sur thermique réelle | **IMP**, tranche T2 — le point 10 de la DoD s'y applique |
