# Revue de la Definition of Done — Cycle 004 · HEB

**T057** · 2026-08-02 · `docs/user-stories-v1.md` §0.4

Les dix points, pour les **cinq user stories** du cycle (HEB-01 à HEB-05), **avec la preuve de
chacun**. Un point coché sans preuve est un point que personne n'a vérifié.

> **Le point 10 est SANS OBJET, et c'est écrit ici plutôt que coché en silence.** Ce cycle
> n'imprime rien : aucun document, aucune file d'impression, aucun pilote. Même règle qu'au
> cycle 001 pour le point 8, et qu'aux cycles 002 et 003 pour le point 10.

---

## Les chiffres du cycle, recomptés et non repris du plan

| Grandeur | Réel | Ce qu'annonçait le plan |
|---|---|---|
| Types d'événements outbox | **27** (13 + 9 + 5) | 27 |
| Opérations HTTP servies | **56** (43 + 13) | 56 |
| `operationId` distincts | **56** | 56 |
| Tables, **cinq** schémas applicatifs | **34**, dont **8** créées ici | 34 |
| Migrations du cycle | **6** (`0021` à `0026`) | 6 |
| Familles d'audit | **10**, dont **3 branchées** | 10, dont 2 |
| Tests backend | **321**, 0 échec | — |
| Tests front | **458**, 0 échec, 0 erreur de type | — |
| Tests de parcours réel (P-22) | **56** sur **8** routes, 2 moteurs | — |
| Clés i18n | **276 fr / 276 en**, écart nul | — |

**Un seul écart au plan, et il est en plus** : `rebascule_palier_passage` fait passer les familles
d'audit branchées de deux à trois. Le plan comptait deux parce qu'il reprenait l'état du cycle 003.

Les quatre premières lignes ont été **relues du catalogue système et du contrat** par
`couverture_portes.rs`, jamais recopiées : c'est ce qui a permis d'y trouver deux portes aveugles
(voir T049 plus bas).

---

## Les dix points

### 1 · Critères d'acceptation couverts par des tests — unitaires **et** d'intégration ✓

| Niveau | Preuve |
|---|---|
| **Unitaires**, dans le crate | 25 tests dans `kaya-hebergement` — le barème (`tarification/bareme.rs`), la traduction d'erreur (`erreurs.rs`), les familles de formules |
| **Intégration**, sur les transitions d'état | `hebergement_referentiel.rs`, `hebergement_disponibilite.rs` (16 cas), `hebergement_tarification.rs`, `hebergement_hors_ligne.rs` |
| **Transitions d'état** nommément | `liberer_raccourcit_la_periode_sans_effacer_l_occupation` — `active → liberee` est un `UPDATE`, jamais un `DELETE` ; `trois_attributions_du_meme_identifiant_produisent_une_occupation` — le rejeu ne rejoue pas |

**Le test qui vaut le cycle** : `deux_attributions_concurrentes_une_seule_reussit` asserte que le
refus est un `ExclusionViolation` **sur la contrainte nommée**. Un test qui se contenterait de
« une seule a réussi » serait vert sur un verrou applicatif — qui se dégrade sous charge sans rien
signaler.

### 2 · Annotations utoipa à jour, client TypeScript régénéré sans diff manuel ✓

`pnpm porte:p01` — « client identique au contrat ». Les treize opérations portent leurs
annotations ; `p01b` vérifie que les 56 `operationId` sont présents et distincts, ce que P-01 ne
peut pas voir (un client invalide se régénère à l'identique).

### 3 · Migration versionnée, `cargo sqlx prepare` vert, seeds à jour ✓

Six migrations, `0021` à `0026`, **aucune modifiée après application** (`porte:p02`, 26 migrations
comparées à `origin/main`).

`cargo sqlx prepare` — la **double passe** de `CLAUDE.md` : 45 requêtes récoltées depuis `backend/`,
52 depuis `backend/api/`, fusion à 198 entrées dont **56 nouvelles**. Les deux contrôles dans
l'ordre : aucune suppression dans `git status --short .sqlx`, puis le check hors ligne.

> ⚠️ **Le second contrôle ment s'il ne recompile rien**, et c'est ce cycle qui l'a constaté. Lancé
> sur un build à jour, `SQLX_OFFLINE=true cargo check` affiche `Finished` en une seconde sans
> consulter `.sqlx`. Le cache était réellement périmé et le contrôle venait de passer au vert.
> `CLAUDE.md` gagne le `touch` des 50 fichiers à requêtes avant le check.

**Seeds à jour** : le parc de Deloria (17 unités, 6 catégories, 11 formules, 20 paliers, 2 plages,
11 temps de remise en état, 3 valeurs de configuration) et Résidence Test (4 meublés, mois et
nuitée). `seeds_rejouables.rs` — trois exécutions, même état, **parc compris** depuis ce cycle.

### 4 · RLS activée **et forcée** sur les huit tables, avec test d'isolation ✓

Relevé du catalogue système :

| Table | `ENABLE` | `FORCE` | Politiques |
|---|---|---|---|
| `categorie`, `temps_remise_en_etat`, `unite`, `formule`, `bareme_palier`, `plage_demi_journee`, `occupation`, `prestation_incluse` | ✓ | ✓ | 1 chacune |

Test d'isolation multi-tenant : `isolation_tenant.rs` —
`p08_cycle_004_appels_croises_sur_le_referentiel_d_hebergement` et `…_sur_la_disponibilite`. Un
compte du tenant A appelle les routes du tenant B et **ne voit ni ne modifie rien** ; le test
vérifie ensuite en base que la catégorie de B porte toujours son nom, sa chambre son code, sa
formule son tarif.

### 5 · Classe hors-ligne déclarée pour chaque entité, avec son test ✓

`docs/registre-classes-offline.md` §7.1 et §10 — référentiel en **C**, occupation en **B**,
`prestation_incluse` en **C** au titre des provisions. Version 1.2.0 du registre.

`classes_offline.rs` compare **34 tables** aux entités déclarées, sur les cinq schémas. Le schéma
`hebergement` a été ajouté au balayage par ce cycle : **sans cet ajout, les huit tables y
échappaient entièrement** — exactement le trou trouvé sur `comptes` au cycle 003.

`hebergement_hors_ligne.rs` (**P-13**) : les treize opérations exigent un jeton, les treize
aboutissent en ligne, et **la liste est fermée** — le décompte des chemins `/hebergement/` servis
par le contrat est comparé aux treize déclarées. Un troisième test constate que les privilèges
disent la classe : quatre verbes pour le référentiel, trois pour l'occupation (**jamais `DELETE`**),
zéro pour la provision.

### 6 · Événement outbox pour tout changement d'état ✓

Cinq types : `heb.categorie.tarif_modifie`, `heb.formule.creee`, `heb.formule.modifiee`,
`heb.occupation.attribuee`, `heb.occupation.liberee`. Tous émis **dans la transaction** de
l'écriture ; `outbox_transactionnel.rs` le vérifie, et vérifie aussi qu'un **rejeu n'émet rien** —
sans quoi le grand livre deviendrait le journal des tentatives réseau des terminaux.

> **P-05 ne lisait aucun fichier de `verticales/` avant le recollement.** Les cinq types étaient
> émis en production et **invisibles à la porte**, qui restait verte. Ajouter les types sans ajouter
> les fichiers au balayage aurait rendu le total juste et la porte toujours aveugle.

### 7 · Clés i18n `fr` et `en`, aucune chaîne en dur ✓

**276 clés de chaque côté**, écart nul (`pnpm test:i18n`, porte **P-16**). 26 templates inspectés
sans littéral ; une exemption, `pages/styleguide.vue`, **bornée** — la porte vérifie elle-même que
la route est retirée du routeur hors développement.

> **Un défaut trouvé au recollement, et c'est P-22 qui l'a trouvé.** Les seeds ont posé les trois
> valeurs de configuration Deloria ; l'écran d'établissement les affiche donc pour la première fois,
> et leurs clés i18n n'existaient pas. Corrigé en fr et en.
>
> La cause est une **dette consignée dans `SectionPointsDeVente.vue`** : l'écran *fabrique* la clé
> `configuration.<clé>.libelle` alors que `parametre_catalogue.libelle_cle` en déclare une en base,
> et les deux conventions ont divergé. Tant qu'aucun établissement n'avait de valeur pour ces
> paramètres, rien ne s'affichait et l'écart restait invisible.

### 8 · Écrans vérifiés en clair **et** en sombre ✓

Deux écrans livrés : **G2** (l'offre d'hébergement) et **G5** (le parc d'unités, écran composé).

`pnpm porte:p22` — **56 cas verts** sur 8 routes, en chargement **direct** et par **navigation
interne**, dans les **deux thèmes**, sur **deux moteurs** (Chromium et WebKit). `/hebergement` et
`/chambres` y sont, découvertes du système de fichiers.

`pnpm porte:p22:negatif` — la porte refuse bien une coquille cassée (`<main>` remplacé par `<div>`).

> **Limite à connaître** : le WebKit de Playwright **n'est pas** WKWebView. Un vert dit « tourne sur
> un moteur WebKit », jamais « vérifié sur la cible ». La vérification sur WKWebView viendra avec la
> coquille Tauri.

### 9 · Paramètres exposés dans la configuration d'établissement ✓

Trois clés au catalogue (`0023`), **valorisées par les seeds** — la migration l'annonçait :
« les valeurs Deloria sont posées par les seeds ».

| Clé | Type | Valeur Deloria |
|---|---|---|
| `heure_arrivee_standard` | `HEURE_LOCALE` | `"14:00"` |
| `heure_depart_standard` | `HEURE_LOCALE` | `"12:00"` |
| `seuil_bascule_nuitee_minutes` | `DUREE_MINUTES` | `480` |

Trois valeurs restent **hors** du catalogue, délibérément : le temps de remise en état (il varie par
catégorie **et** par famille), les plages de demi-journée et le barème de passage — tous trois sont
des référentiels, et une table porte l'ordre et l'unicité qu'un `JSONB` ne contraindrait pas.

### 10 · Document imprimé vérifié sur imprimante thermique — **SANS OBJET**

Ce cycle n'imprime rien. Aucun document, aucune file d'impression, aucun pilote. Écrit plutôt que
coché.

---

## Ce que le recollement a trouvé, et qu'aucune tâche ne prévoyait

Quatre défauts, tous dans des **portes**, aucun dans le produit :

| # | Défaut | Pourquoi il était invisible |
|---|---|---|
| 1 | **P-05 ne balayait pas `verticales/`** | Les cinq types HEB étaient émis et hors de tout décompte |
| 2 | **Le récapitulatif comptait 4 schémas sur 5** | Il annonçait 26 tables quand `classes_offline` en déclarait 34. C'est la **confrontation des deux totaux** qui l'a montré — un décompte seul serait resté plausible |
| 3 | **`rebascule_palier_passage` déclarée « due » alors qu'elle est branchée** | `audit_taxonomie.rs` était **rouge dans l'arbre depuis T042** : les tâches intermédiaires lançaient des suites ciblées |
| 4 | **La porte « une famille branchée = un test » ne lisait qu'un fichier** | Le test existe, dans `hebergement_tarification.rs` — près de ce qu'il teste |

Deux portes ont par ailleurs été corrigées sur leur **cible**, pas sur leur contenu :

- **P-02** traitait `backend/migrations/seeds/README.md` comme une migration et exigeait « une
  migration nouvelle qui corrige » pour un fichier de documentation. Le principe I(b) sépare
  pourtant les deux. Le décompte final du script était d'ailleurs déjà limité à `-maxdepth 1` :
  les deux extrémités se contredisaient.
- **P-03** n'avait **aucune cible** depuis trois cycles : `verticales/` était vide, la porte
  parcourait le graphe sans trouver une arête à interdire, et passait au vert — indistinguable
  d'une porte dont on aurait supprimé la famille entière.

**Leçon opératoire, et elle est la plus chère de ce recollement** : lancer `cargo test --workspace`
avant chaque commit de fin de phase, et non des suites ciblées. Le défaut n° 3 a vécu un cycle
entier dans un arbre que tout le monde croyait vert.

---

## Ce qui reste ouvert — signalé, pas résolu

| # | Point | Échéance |
|---|---|---|
| **B-02** | Traitement fiscal de la taxe de nuitée sur le passage et la demi-journée. Aucune valeur en dur : `assujettie_taxe_nuitee` est un **paramètre par formule**, et B-02 décidera de sa **valeur par défaut légale**, jamais de son existence | Fiscaliste, S3 |
| **B-07** | Barèmes de passage réels du pilote, et **prix d'une plage de demi-journée**. Le cadrage donne 50 500 **par jour** quand le produit vend **par plage** ; la valeur seedée reprend le nombre sans le transformer | Atelier terrain |
| **B-10** | **L'axe des personnes.** `une_nuitee_par_occupation` réduit trois nuits à une, et ne dit rien de trois clients. 3 nuits × 2 personnes valent 500 F ou 1 000 F — aucune source ne le dit | Avant le cycle SEJ |
| Dette | L'écran fabrique la clé i18n d'un paramètre au lieu de lire `libelle_cle`. Correctif de fond : exposer la colonne au contrat | Cycle qui touchera la configuration |
| Gel | Le **point ouvert** de l'en-tête de `docs/versions-gelees.md` peut être retiré (sqlx confirmée), et la note du §2 gagnerait sa limite : `#3918` apporte `ErrorKind::ExclusionViolation` mais **pas** `is_exclusion_violation()` | Revue mensuelle du 2026-08-31 |

**Aucune de ces cinq lignes n'a été tranchée par défaut.** Un multiplicateur posé à l'aveugle sur
B-10 se retrouverait sur des factures et dans un état de reversement communal.
