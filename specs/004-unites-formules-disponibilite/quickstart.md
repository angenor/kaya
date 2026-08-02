# Quickstart — valider le cycle 004 (HEB)

**Guide de validation exécutable.** Il dit comment prouver que le cycle fonctionne, pas comment
l'écrire — l'implémentation est décrite par [data-model.md](./data-model.md) et
[contracts/](./contracts/), les tâches par `tasks.md`.

**La question à laquelle ce document répond** : *la double attribution est-elle impossible, et
peut-on le montrer ?*

---

## 0. Prérequis

```sh
# Services — Postgres 18.4, Redis 8.8.1, Garage 2.3.0
docker compose -f infra/compose.yml up -d

# Rôles, schémas, migrations (les 26, dont les 6 du cycle)
bash scripts/dev/preparer-base.sh

# Amorçage des buckets
bash scripts/dev/preparer-stockage.sh
```

`btree_gist` est installée par la migration `0001` — rien à faire, mais on peut le constater :

```sh
psql "$DATABASE_URL" -c "SELECT extname FROM pg_extension WHERE extname = 'btree_gist';"
# → btree_gist
```

---

## 1. La vérification qui compte — le chevauchement est impossible

### 1.1 Constater que la contrainte existe et porte sur le bon type

```sh
psql "$DATABASE_URL" -c "
  SELECT conname, pg_get_constraintdef(oid)
  FROM pg_constraint
  WHERE conrelid = 'hebergement.occupation'::regclass AND contype = 'x';"
```

Attendu — la contrainte, avec ses deux opérateurs :

```
occupation_sans_chevauchement | EXCLUDE USING gist (unite_id WITH =, periode WITH &&)
```

Et le type de la colonne, qui ne doit **jamais** être une paire de dates :

```sh
psql "$DATABASE_URL" -c "
  SELECT column_name, udt_name FROM information_schema.columns
  WHERE table_schema = 'hebergement' AND table_name = 'occupation'
    AND column_name = 'periode';"
# → periode | tstzrange
```

### 1.2 Le chevauchement, à la main

Deux occupations qui se chevauchent d'une heure sur la même unité :

```sql
-- Passe.
INSERT INTO hebergement.occupation (…, unite_id, periode, …)
VALUES (…, '<B3>', tstzrange('2026-08-03 14:00+00', '2026-08-05 14:00+00', '[)'), …);

-- ÉCHOUE — 23P01, exclusion_violation.
INSERT INTO hebergement.occupation (…, unite_id, periode, …)
VALUES (…, '<B3>', tstzrange('2026-08-05 13:00+00', '2026-08-07 14:00+00', '[)'), …);
```

```
ERROR:  conflicting key value violates exclusion constraint "occupation_sans_chevauchement"
```

**Contiguë, en revanche, passe** — la borne de fin est exclue :

```sql
-- Passe : `[)` fait que 14:00 n'appartient pas à la première occupation.
VALUES (…, '<B3>', tstzrange('2026-08-05 14:00+00', '2026-08-07 14:00+00', '[)'), …);
```

### 1.3 Le seul contournement possible, et qu'il est fermé

```sql
-- ÉCHOUE — CHECK occupation_periode_non_vide.
-- Sans ce CHECK, la ligne passerait : `&&` est FAUX dès qu'un intervalle est vide.
-- On aurait une occupation qui occupe une unité SANS la bloquer.
VALUES (…, '<B3>', tstzrange('2026-08-03 14:00+00', '2026-08-03 14:00+00', '[)'), …);
```

---

## 2. La suite de tests

```sh
cd backend

# Tout le cycle
cargo test --workspace

# ★ La porte P-09 levée, et la classe B
cargo test --test hebergement_disponibilite

# Le référentiel — classe C, refus explicites
cargo test --test hebergement_referentiel

# Le barème — cas figés
cargo test --test hebergement_tarification

# P-13 — rien de ce cycle n'est atteignable hors ligne
cargo test --test hebergement_hors_ligne
```

### Ce que `hebergement_disponibilite` doit prouver, et dans quel ordre

| Test | Ce qu'il établit |
|---|---|
| `periode_est_un_tstzrange` | Assertion 1 de P-09 — jamais une paire de dates |
| `contrainte_exclusion_gist_presente` | Assertion 2 de P-09 |
| **`deux_attributions_concurrentes_une_seule_reussit`** | **Assertion 3** — deux transactions distinctes, exactement une réussit, et l'échec est un `ErrorKind::ExclusionViolation` sur la contrainte nommée |
| `intervalle_vide_refuse` | Le seul contournement, fermé |
| `occupations_contigues_coexistent` | La borne de fin est exclue |
| `remise_en_etat_bloque_la_suivante` | 12 h + 2 h de ménage → 13 h refusé, 14 h accepté |
| `intervalle_traversant_minuit` | 22 h → 6 h n'est pas un cas spécial |
| `liberation_raccourcit_la_periode` | Pas de `DELETE` ; `statut = 'liberee'` |

> **Le test de concurrence est le seul qui distingue une garantie d'une coïncidence.** Il ouvre
> **deux transactions PostgreSQL réelles**, insère dans chacune sans commiter, puis commite les
> deux. Il asserte la **cause** du refus, pas seulement son existence : un test qui se contenterait
> de « une seule a réussi » passerait au vert sur un `SELECT … FOR UPDATE`, sur `SERIALIZABLE` ou
> sur un verrou applicatif — trois mécanismes qui rendent la double attribution *improbable* au
> lieu d'*impossible*, et qui se dégradent sous charge sans rien signaler.

---

## 3. Les portes, une par une

```sh
# Depuis la racine
pnpm porte:p01    # 13 opérations nouvelles → client TS régénéré sans diff
pnpm porte:p02    # les 20 migrations antérieures intactes
pnpm porte:p04    # aucune jointure hebergement × autre schéma
pnpm porte:p10    # prix_mineur entier, quantite NUMERIC
pnpm porte:p15    # pas de window.__TAURI__ hors PlatformAdapter
pnpm porte:p19    # G2-offre-hebergement.html lu, jamais copié
pnpm porte:p20    # aucune dépendance nouvelle — le lockfile ne doit PAS bouger
pnpm porte:p21    # l'écran ne charge rien d'un hôte externe
pnpm porte:p22    # /hebergement ET /chambres s'ouvrent en direct ET par navigation, 2 moteurs, 2 thèmes
```

```sh
cd backend
# P-18 — LA DOUBLE PASSE. Une seule passe perd les requêtes de l'autre cible.
bash ../scripts/ci/preparer-sqlx.sh

# Les DEUX contrôles, dans cet ordre — le second seul ne suffit pas
git status --short .sqlx    # AUCUNE suppression ; que des ajouts
SQLX_OFFLINE=true DATABASE_URL= cargo check --workspace --all-targets --locked
```

### Le test négatif de P-09 — prouver qu'elle sait échouer

Sur le modèle de `pnpm porte:p22:negatif` : retirer la contrainte d'exclusion sur une base de
test, constater que les trois assertions échouent, remettre.

```sh
psql "$DATABASE_URL_TEST" -c \
  "ALTER TABLE hebergement.occupation DROP CONSTRAINT occupation_sans_chevauchement;"
cargo test --test hebergement_disponibilite     # DOIT échouer
psql "$DATABASE_URL_TEST" < backend/migrations/0025_occupation.sql   # remettre
cargo test --test hebergement_disponibilite     # DOIT repasser
```

**Une porte qui n'a jamais échoué n'est pas une porte.** C'est la leçon des quatre portes vertes
défectueuses du cycle 001.

---

## 4. L'écran

```sh
pnpm --filter @kaya/app dev
# puis http://localhost:3000/hebergement
```

| À vérifier | Attendu |
|---|---|
| Mode clair **et** mode sombre | Les deux (DoD point 8) |
| Chargement direct de l'adresse | La page se monte, aucune erreur de console |
| Navigation depuis l'accueil | Idem — P-22 exige **les deux** |
| Connecté comme réceptionniste | L'action de modifier un tarif est **absente**, pas grisée |
| Réseau coupé (outils de développement) | Indisponibilité annoncée **immédiatement**, en langue utilisateur, sans file d'attente |
| Montants | `12 500 F` avec l'espace fine insécable U+202F, colonnes alignées en Chivo Mono |
| Vocabulaire | « chambre », « Taxe de séjour comprise dans le prix ». **Jamais** « unité louable », « occupation », « palier », « classe hors-ligne » |

**Les deux états maquettés** doivent tous deux fonctionner : l'hôtel à quatre formules, et la
résidence à deux — cette dernière portant l'affordance « Ajouter le passage ici », qui est la
preuve visuelle qu'aucune formule n'est réservée à un type d'établissement.

### 4.1 Le second écran — `G5`, chambres et catégories

```sh
# puis http://localhost:3000/chambres
```

`G5` est un **écran composé** : aucune maquette, aucun motif hérité, assemblé uniquement à partir
des composants canoniques (troisième cas de `docs/Kaya_Design.md` §2). Mêmes vérifications que
ci-dessus, plus :

| À vérifier | Attendu |
|---|---|
| Choix de la catégorie dans le formulaire d'unité | Un **choix fermé** (`<select>`, composant 16), **jamais un sélecteur segmenté** — six catégories ne tiennent pas en segments sur 372 px |
| Correction d'un code de chambre | Possible — c'est ce que le registre §7.1 classe (« `unite` — code, étage ») |
| Changement de catégorie · statut de ménage · mise hors service | **Absents de l'écran.** Effet fiscal non classé · classe A HEB-06 · classe B HEB-06 |
| Ligne de liste | Composant 08 — code en mono, colonne de largeur fixe, actions de bord au survol seulement |
| Catégorie sans unité | État vide illustré (composant 11), jamais une liste blanche |

---

## 5. Les seeds

```sh
# Rejouable, idempotent, en une commande — autant de fois que voulu
cargo run --bin seeds
cargo run --bin seeds    # le second passage donne le même état
```

```sh
psql "$DATABASE_URL" -c "
  SELECT c.nom, count(u.id) AS unites
  FROM hebergement.categorie c LEFT JOIN hebergement.unite u ON u.categorie_id = c.id
  GROUP BY c.nom ORDER BY c.nom;"
```

Attendu pour Deloria — **17 unités en 5 catégories, plus la salle de réunion** :

| Catégorie | Unités | Tarif nuitée |
|---|---|---|
| Standard | A1–A3 (3) | 12 500 |
| Classique | B1–B5 (5) | 15 500 |
| Classique supérieure | C1–C4 (4) | 17 500 |
| Supérieure A | D1–D2 (2) | 20 500 |
| Supérieure B | E1–E3 (3) | 25 500 |
| Salle de réunion | 1 | — (demi-journée) |

Le barème de passage : 1 h → 1 500 · 2 h → 2 800 · 3 h → 4 000 · 4 h → 5 000 · h. suppl. +1 200.
Les plages de demi-journée : 8 h – 12 h et 13 h – 16 h. Les temps de remise en état : passage
30 min, nuitée 2 h, demi-journée 1 h.

```sh
psql "$DATABASE_URL" -c "
  SELECT famille, assujettie_taxe_nuitee, regle_conversion_taxe
  FROM hebergement.formule ORDER BY famille;"
# NUITEE       | t | une_nuitee_par_occupation   ← 500 F pour un séjour de 3 nuits, pas 3 × 500
# PASSAGE      | f | aucune                      ← constat d'exploitation
# DEMI_JOURNEE | f | aucune                      ← idem
# MENSUEL      | f | aucune
```

> **Le passage et la demi-journée ne sont pas assujettis, et c'est un constat, pas un oubli.**
> Ce que le produit doit offrir, et qui se vérifie ici, c'est le **moyen facultatif de l'activer** :

```sh
# Activer la taxe sur le passage doit EXIGER une règle de conversion.
# Sans elle → refus par formule_regle_fiscale_coherente, pas une ligne à moitié écrite.
psql "$DATABASE_URL" -c "
  UPDATE hebergement.formule SET assujettie_taxe_nuitee = true
  WHERE famille = 'PASSAGE';"
# ERROR:  new row violates check constraint "formule_regle_fiscale_coherente"
```

> **C'est cette contrainte qui supprime le troisième état d'écran.** Une formule assujettie sans
> règle aurait imposé d'afficher « paramétrage fiscal en attente » — mention absente de la
> maquette `G2` **et** du lexique. Elle est impossible à constituer, donc les deux mentions
> maquettées suffisent.

---

## 6. Le second tenant — l'agnosticité, enfin mesurable

```sh
cargo test --test agnosticite_socle
```

**Ce test existe depuis le cycle 002 et prend son sens maintenant.** Jusqu'ici il prouvait que le
socle n'exigeait rien d'une verticale — mais aucune verticale n'existait pour le contredire. Ce
cycle en crée une. S'il passe encore, « aucun crate partagé ne suppose l'existence d'un
hébergement » cesse d'être une intention pour devenir un fait mesuré.

Le tenant « Résidence Test » (module hébergement seul, quatre unités) éprouve en outre qu'un
établissement qui ne propose que le mois et la nuitée fonctionne de bout en bout, sans qu'aucun
code ne suppose l'existence du passage.

---

## 7. Ce qui n'est PAS validable à ce cycle, et pourquoi

| Attendu absent | Motif |
|---|---|
| Enregistrer un client en chambre B3 | Check-in = SEJ-02, cycle suivant. **La démo de fin de T1 n'est pas exécutable ici** — ce cycle en livre la moitié basse |
| Voir une ligne de note | SEJ-03, tranche T2. Le moteur calcule, il ne facture pas |
| Voir un montant de taxe de séjour | FIS-03, tranche T3. Le paramètre est porté, jamais interprété |
| Changer le statut ménage d'une chambre | HEB-06, P1 — la colonne existe, l'endpoint non |
| Voir un planning | RSV, tranche T4 |
| Lire ou écrire une prestation incluse | HEB-09 — table vide, aucun privilège accordé |
