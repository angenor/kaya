# Mesures de terrain — SC-002, SC-003 et FR-106

**Cycle 006 — SEJ · Fiches clients, arrivée, départ et prolongation**
Version 1.0.0 · 2026-08-03

---

## ⚠️ À lire en premier : ce document contient un protocole, et une mesure MANQUANTE

**Le chronométrage humain n'a pas été relevé.** Il exige un opérateur devant un comptoir et le
matériel de référence ; ni l'un ni l'autre n'existent à ce jour dans ce dépôt. Ce fichier livre
donc :

| Ce qui est ici | État |
|---|---|
| Le **protocole** — matériel, jeu de données, points de départ et d'arrivée du chronomètre | ✅ écrit et versionné (FR-106) |
| Les **budgets déterministes** tenus par l'intégration continue | ✅ mesurés, verts |
| La **part machine** du parcours, chronométrée sur deux moteurs | ✅ mesurée |
| Le **temps humain** de SC-002 et SC-003 | ❌ **non relevé** |

**Ne pas conclure de ce document que les cibles de 30 s et de 60 s sont atteintes.** Ce qui est
établi, c'est que **rien de ce qui dépend du produit ne les empêche** : le parcours tient en deux
gestes, aucune saisie libre n'est obligatoire, un seul appel réseau bloque, et la part machine est
deux ordres de grandeur sous la cible. Le reste est la main de l'opérateur, et il se mesure au
terrain.

C'est aussi ce que **FR-107** impose : l'intégration continue ne garde **jamais** ces cibles par une
mesure de temps humain, qui rougirait au hasard et serait désactivée dans le mois — leçon SC-004 du
cycle 004. Elle les garde par des **budgets déterministes**, et le chronomètre humain reste un
constat de terrain, consigné ici, jamais asserté.

---

## 1 · Ce que chaque cible mesure, exactement

| Cible | Ce qui est chronométré | Départ du chronomètre | Arrêt du chronomètre |
|---|---|---|---|
| **SC-002 — passage, < 30 s** | Un client de passage, **inconnu**, sans fiche | L'opérateur touche l'écran `/passage` pour la première fois | L'écran affiche « C'est fait » **et** l'heure de fin |
| **SC-003 — arrivée, < 60 s** | Un client **connu**, retrouvé par la recherche | L'opérateur touche le champ de recherche de `/arrivee` | L'écran affiche « C'est fait » **et** le départ prévu |

**Seuil d'échec, et il n'est pas négociable : au-delà de 90 secondes pour un passage, SC-002 est en
échec.** Pas « améliorable » — en échec. Le cadrage §5.6 en fait une condition d'existence du
produit : *« le module de passage doit être irréprochable en rapidité (moins de 30 secondes) sinon
il sera contourné »*. Un écran de comptoir contourné, c'est un cahier papier qui revient, et tout
le reste du produit avec lui.

### Ce qui est DANS le chronomètre, et ce qui n'y est pas

**Dedans** : la lecture de l'écran, la décision, les gestes, l'attente du réseau, la lecture de la
confirmation.

**Dehors** — et chacun pour une raison écrite :

- **Le montage de l'écran.** Les appels de chargement (`etat-des-unites`, catégories, formules)
  partent **avant le premier geste**. Les précharger est précisément ce qui permet à l'attribution
  de n'être qu'un tap ; les compter reviendrait à pénaliser la décision qui rend la cible
  atteignable.
- **La saisie de la pièce d'identité.** La maquette `R4` l'écrit en toutes lettres : *« Pièce
  d'identité : après la clé, pas avant »* (FR-023). Le séjour est ouvert, la clé est donnée, la
  fiche reste à compléter. La compléter est un second parcours, avec son propre temps.
- **La conversation avec le client.** Elle n'est pas du produit, et elle domine la variance.

---

## 2 · Matériel de référence

Le matériel n'est pas un détail : une cible tenue sur un ordinateur de développement et manquée sur
le terminal du comptoir ne prouve rien.

| Élément | Référence retenue | Pourquoi celle-là |
|---|---|---|
| Terminal | Ordinateur portable d'entrée de gamme, écran 1366 × 768, 4 Gio de mémoire | C'est ce que le cadrage décrit au comptoir du pilote |
| Écran | **En plein soleil**, luminosité au maximum | Le 13,5 px de `--text-corps` a été choisi pour ce cas ; mesurer à l'ombre changerait le temps de lecture |
| Saisie | **Écran tactile si présent, sinon souris** | Le budget de gestes est le même ; le temps par geste ne l'est pas |
| Réseau | Connexion du pilote, **telle quelle** | Un réseau de bureau masquerait la latence qui décide de la cible |
| Application | Build de production (`pnpm build`), jamais le serveur de développement | Le mode développement recompile à la volée et fausse le premier rendu |
| Données | Les seeds, **rechargés juste avant** (`cargo run -p kaya-api --bin seeds`) | Voir §3 |

---

## 3 · Jeu de données

**Les seeds, et rien d'autre.** `backend/api/src/bin/seeds.rs`, rechargés avant chaque série :

- Deloria — **17 chambres en 5 catégories**, plus la salle de réunion ;
- **12 fiches clientes**, dont « Bakayoko Adama » et « Koné Aminata » — deux noms qui rendent
  plusieurs résultats à la recherche par préfixe ;
- **3 séjours** : une nuitée en cours (2 nuits, 2 accompagnants), un passage en cours **sans fiche**,
  et un séjour **clos** avec son constat de taxe figé.

⚠️ **Recharger les seeds entre deux séries, jamais entre deux mesures d'une même série.** Le parc se
remplit à mesure qu'on l'exerce, et une chambre libre de moins change la lecture de la grille — donc
le temps. Mesurer sur un parc qui se vide progressivement est **plus honnête** que remettre l'écran
à neuf à chaque coup : c'est ce qui se passe au comptoir un soir de forte affluence.

**Trois mesures par cible, et c'est la médiane qui est consignée.** Une seule mesure attrape le
premier essai, où l'opérateur cherche encore ses repères ; une moyenne se laisse tirer par un
incident réseau isolé.

---

## 4 · Protocole, pas à pas

### SC-002 — passage, client inconnu

1. Recharger les seeds. Ouvrir l'application, se connecter, **rester sur l'accueil**.
2. Déclencher le chronomètre **au moment où la main touche la tuile « Passage »**.
3. Toucher une durée. Toucher une chambre libre.
4. Arrêter le chronomètre **quand l'heure de fin est lisible à l'écran**.
5. Noter la valeur. Répéter deux fois, sur deux chambres différentes.

### SC-003 — arrivée, client connu

1. Recharger les seeds. Ouvrir `/arrivee`.
2. Déclencher le chronomètre **au moment où la main touche le champ de recherche**.
3. Taper « bak », toucher « Bakayoko » dans la liste.
4. Toucher le nombre de nuits. Ajouter un accompagnant au nom seul. Toucher une chambre libre.
5. Arrêter le chronomètre **quand le départ prévu est lisible à l'écran**.
6. Noter la valeur. Répéter deux fois.

---

## 5 · Relevés

### 5.1 · Temps humain — **NON RELEVÉ**

| Cible | Médiane | Valeurs | Date | Opérateur |
|---|---|---|---|---|
| SC-002 — passage, client inconnu | — | — | — | — |
| SC-003 — arrivée, client connu | — | — | — | — |

**Ce tableau reste vide jusqu'à une session de terrain.** Le remplir depuis un poste de
développement produirait un chiffre flatteur et faux : ni l'écran, ni le réseau, ni la main ne
seraient ceux du comptoir. Un chiffre faux dans ce tableau est pire qu'une case vide — il clôt une
question qui reste ouverte.

**Qui le remplit, et quand** : au premier déploiement chez le pilote, avant la démonstration de fin
de tranche T1. Le remplissage se fait dans ce fichier, par un commit qui **cite la date et le
matériel**.

### 5.2 · Ce qui EST mesuré, et qui est vert

| Contrainte | Cible | Relevé | Où c'est tenu |
|---|---|---|---|
| Interactions obligatoires, passage | **exactement 2** | **2** | `app/tests/budget-gestes.spec.ts` |
| Champs de saisie libre obligatoires, passage | **0** | **0** | idem |
| Appels réseau bloquants, passage | **≤ 1** | **1** | idem |
| Bouton de soumission sur l'arrivée | **aucun** | **aucun** | `app/tests/ecran-r3.spec.ts` |
| Part **machine** du passage, du premier geste à la confirmation | < 6 000 ms | voir §5.3 | `tests-e2e/passage.spec.ts`, Chromium **et** WebKit |
| Recherche de fiche sur 10 000 fiches, 95ᵉ centile | < 300 ms | test présent, **marqué `ignored`** | `backend/tests/client_recherche.rs` |

⚠️ **La mesure des 300 ms est écrite et n'est pas exécutée en continu** : peupler dix mille fiches
prend plus longtemps que toute la suite. Elle se lance à la main —
`cargo test --test client_recherche -- --ignored` — et sa valeur se consigne ici quand elle est
relevée. Une mesure qu'on prétend continue et qui ne tourne jamais est pire qu'une mesure déclarée
manuelle.

### 5.3 · Part machine du parcours de passage

Le budget est **fixé très au-dessus de la valeur observée, et c'est une décision** : ce test tourne
sur deux moteurs, sur des machines de CI dont la charge varie. Un budget serré rougirait au hasard,
serait désactivé sous trois semaines, et ne garderait alors plus rien. À 6 000 ms, il attrape une
**régression d'un ordre de grandeur** — un appel réseau ajouté, une attente introduite — et rien
d'autre. C'est exactement ce qu'on lui demande.

La valeur observée est imprimée par le test à chaque exécution (`console.log`), et **elle n'est pas
recopiée ici** : une valeur figée dans un document se périme au premier changement de machine, et
personne ne la corrige. Le journal de la porte fait foi.

---

## 6 · Ce que ce document ne prouve pas

- **Il ne prouve pas que la cible de 30 s est tenue.** Il prouve que le produit n'y met aucun
  obstacle mesurable.
- **Il ne prouve rien sur WKWebView.** P-22 et le chronométrage machine tournent sur le WebKit de
  Playwright, qui **n'est pas** WKWebView. La vérification sur la cible viendra avec la coquille
  Tauri.
- **Il ne dit rien du second parcours** — la complétion de la fiche d'identité après la clé. Elle a
  son propre temps, et aucune story du périmètre ne le borne.
