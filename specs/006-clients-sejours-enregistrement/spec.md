# Feature Specification: Fiches clients, arrivée, départ et prolongation

**Feature Branch**: `006-clients-sejours-enregistrement` (aucune branche git dédiée créée — travail sur la branche courante, comme aux cycles 001 à 005)

**Created**: 2026-08-03

**Status**: Draft

**Input**: User description: "Fiches clients, check-in, check-out et prolongation. Périmètre : SEJ-01, SEJ-02, SEJ-04 — critères tels quels. Fiche client rattachée au TENANT et partagée entre ses établissements. Recherche par nom, téléphone ou numéro de pièce en moins de 300 ms sur 10 000 fiches. Check-in (SEJ-02) : sélection de formule et de période, proposition automatique d'une unité disponible, attribution (classe B), enregistrement des accompagnants (ils IMPACTENT le calcul de la taxe de nuitée), génération de la fiche de police, ouverture de la note. Client connu → pré-remplissage complet, AUCUNE ressaisie. OBJECTIF MESURÉ ET TESTÉ : moins de 60 s pour un client connu, MOINS DE 30 s POUR UN PASSAGE. Un passage qui dépasse 90 s sera contourné par le personnel et le produit aura échoué — traite cette contrainte comme un critère d'acceptation, pas comme un souhait. Check-out (SEJ-04) : calcul final, taxe de nuitée FIGÉE à cet instant et jamais recalculée dynamiquement, prolongation avec vérification de disponibilité sur l'intervalle étendu et signalement explicite du conflit avec la réservation suivante, départ anticipé avec régularisation tracée, changement d'unité en cours de séjour créant deux intervalles avec historique conservé. Hors périmètre : SEJ-03 (note temps réel, tranche T2), SEJ-05 (clients extérieurs, tranche T2), SEJ-06 (OCR, P1, tranche T4). La génération du document fiscal au check-out est du ressort du cycle FIS — expose le point d'ancrage, n'implémente pas la certification. Personas : Yao, Adjoua. Points d'attention : dépendances = etablissements, comptes, hebergement. Le check-in d'un passage doit être un parcours DISTINCT et ultra-court, pas le parcours de nuitée avec des champs en plus."

## Contexte et traçabilité

Sixième cycle du projet, dernier de la tranche T1 (`docs/user-stories-v1.md` §0.5, ordre « TRX,
ETB, CPT, HEB, **SEJ**, SYN » — SYN a été livré en avance au cycle 005). Les cinq cycles
précédents ont livré le socle technique, le référentiel d'établissements, les comptes, le
référentiel d'hébergement avec son moteur de disponibilité et son moteur de tarification, puis la
file hors-ligne. **Ce cycle est celui qui les fait servir à quelque chose** : c'est le premier où
un client réel entre dans une chambre réelle, et le premier dont l'échec est mesurable au
chronomètre.

**La démo de fin de tranche T1 est exactement le périmètre de ce cycle**
(`docs/user-stories-v1.md` §0.5) : *« Yao enregistre un client en chambre B3 pour 2 nuits, puis un
passage de 4 h en A1 — la disponibilité empêche tout chevauchement, tout est tracé. »* Le moteur
de disponibilité existe depuis le cycle 004 ; ce qui manque est **la personne, le séjour et le
geste**.

**Ce cycle porte la seule exigence du corpus formulée comme une condition d'échec du produit.**
Le cadrage §5.6 est explicite : *« le module de passage doit être irréprochable en rapidité (moins
de 30 secondes pour enregistrer un passage) sinon il sera contourné »*. Ce n'est pas un objectif de
confort. Le passage est aujourd'hui encaissé en espèces et non tracé ; le personnel n'a **aucun
intérêt** à le saisir. Un parcours lent ne sera pas critiqué, il sera **évité**, et le produit
perdra en même temps la donnée qui fait son argument de vente auprès du propriétaire. La cible de
30 secondes est donc traitée ici comme un **critère d'acceptation opposable**, avec un protocole de
mesure, un budget de gestes gardé en intégration continue et un seuil d'échec déclaré.

**Deux décisions ouvertes du corpus arrivent à échéance à ce cycle, nommément :**

1. **O-01** (`docs/registre-classes-offline.md` §14) — *« `client` / `personne` en C rend le
   check-in d'un client inconnu impossible hors ligne, y compris en mode C. »* Échéance :
   **« Avant SEJ-02 (tranche T1) »**. C'est ce cycle.
2. **B-10** (`docs/cadrage-v1.md` annexe B) — *« Exonération de taxe de nuitée par personne […]
   **La fenêtre où l'ajout coûte zéro est le cycle SEJ, qui crée `accompagnant`** — pas HEB. »*
   Échéance : **« avant le cycle SEJ, à l'atelier terrain »**. C'est ce cycle.

**Les deux ont été tranchées à l'atelier du 2026-08-03** — voir § Clarifications. B-10 l'a été
**contre trois écrits du corpus**, dont les amendements sont listés en § Suites documentaires dues
et sont dus avant `/speckit-plan`.

**Sources de vérité consultées** (ordre de préséance de la constitution) :

| Source | Sections utilisées |
|---|---|
| `.specify/memory/constitution.md` v1.8.0 | Principe **IV** (intervalle horodaté, horodatage d'autorité, statut dérivé), **V** (montants entiers, prix verrouillés à la ligne, **taxe de nuitée figée au check-out**, aucune règle fiscale hors de `JurisdictionAdapter`, documents opérationnels ≠ fiscaux), **VI** (classes A/B/C/D, refus immédiat hors ligne, écriture orpheline), **VII** (application unique, rôles cumulés, module inactif **absent**), **VIII** (i18n fr/en, mode sombre), **IX** (données d'identité), **X** (prêt ≠ construit), **XII** (référence visuelle) ; portes **P-04**, **P-05**, **P-07**, **P-08**, **P-09**, **P-10**, **P-12**, **P-13**, **P-14**, **P-16**, **P-17**, **P-19**, **P-22**, **P-23** ; § Couverture des portes (les six exigences) ; Definition of Done |
| `docs/cadrage-v1.md` | **§5 entier** (5.1 intervalles, 5.2 formules, 5.3 barème dégressif, 5.4 remise en état, 5.5 fiscalité infra-journalière, **5.6 vigilance opérationnelle — la contrainte des 30 s**), **§9.6** (taxe communale de nuitée : ligne distincte, **figée au check-out** — sa mention « par nuitée **et par client** » est **amendée** par la décision B-10 du 2026-08-03, voir § Suites documentaires dues), §9.7 (fiche de police, classement étoiles), §11.1 à §11.5 (classes hors-ligne, écriture orpheline, horloges), §12 (identité des clients), §14 (provisions), **annexe B — décisions B-02 et B-10** |
| `docs/user-stories-v1.md` | **Module SEJ (SEJ-01, SEJ-02, SEJ-04)** — critères repris **tels quels** ; SEJ-03, SEJ-05, SEJ-06 pour la frontière ; §0.3 (personas), §0.4 (DoD), §0.5 (tranches et **démo de fin de T1**), §0.7 (tests hors-ligne obligatoires, **scénario orphelin**), TRX-05b (recollement des seeds), TRX-06 (données sensibles), FIS-02/FIS-03 (point d'ancrage fiscal), HEB-02/03/04/05 (ce dont le séjour hérite), récapitulatif des paramètres d'établissement |
| `docs/registre-classes-offline.md` v1.3.0 | **§8 séjours** — `client` **C**, `client.preferences` **A**, `sejour` check-in **B**, `accompagnant` **A**, `fiche_police` **B**, `ligne_sejour` **B**, `sejour` check-out **B** ; §7.2 (`occupation` **B**, statut **dérivé**) ; §11 (tests obligatoires par classe, outillage instancié) ; §12 (cas pièges — l'horloge, le passage) ; **§14 décision O-01** |
| `docs/design/derivation.md` v1.2.0 | **`R3` Check-in nuitée hérite de `R4`** · **`R5` Fiche client et recherche hérite de `R7`** — les deux sont **inscrits**, donc codables. `R6` Note temps réel (hérite de `R7`, « sans l'action finale ») est **hors périmètre** (SEJ-03) |
| `docs/design/html/R4-passage*.html` (5 états) | **Référence normative du parcours de passage** — zone vitesse. Le parcours entier est **deux gestes** : la durée, puis la chambre. La mention **« Pièce d'identité : après la clé, pas avant »** est la décision de conception qui rend les 30 secondes atteignables. L'état « enregistré » affiche **« enregistré en 9 s »** |
| `docs/design/html/R7-note-depart*.html` (3 états) | **Référence normative du départ** — zone charme. Sections de note, sous-totaux, **taxe de séjour en ligne distincte**, mention « Document non fiscal », bloc « Déjà fait », libération de la chambre |
| `docs/design/lexique.md` v1.5.1 | Vocabulaire opposable. Aucune entrée n'existe encore pour **séjour, client, accompagnant, fiche de police, arrivée, départ, prolongation, changement de chambre** : elles sont dues **avant** le code (précédent v1.4.0, « le mot est inscrit avant d'être codé ») |
| `docs/module-dore.md` | Patron de tranche verticale (sqlx 0.9) ; **« La septième couche »** (patron d'écriture front) ; **« La huitième couche »** (cycle de vie de l'application — obligatoire avant toute page nouvelle) |
| `specs/004-`, `specs/005-` | Moteur de disponibilité et de tarification consommés ici ; file hors-ligne et témoin de synchronisation déjà montés dans la coquille ; harnais de portes à étapes dues |

**Périmètre du cycle** : **SEJ-01**, **SEJ-02**, **SEJ-04** — critères d'acceptation repris **tels
quels**, sans exigence ajoutée ni retranchée. Les trois sont **P0**.

**Hors périmètre** : SEJ-03 (note de séjour temps réel, tranche T2), SEJ-05 (clients extérieurs,
tranche T2), SEJ-06 (enregistrement accéléré par OCR, P1, tranche T4), la certification fiscale
(FIS, tranche T3) et l'encaissement (CAI, tranche T2). Voir § Out of Scope, qui dit pour chacun ce
qui est exposé et ce qui ne l'est pas.

**Personas** :

- **Yao (réceptionniste)** — **l'utilisateur qui décide du succès de ce cycle**. Il enregistre les
  arrivées, gère les passages, encaisse. Debout, pressé, un client en face. Sa mesure du produit
  n'est pas une liste de fonctionnalités, c'est le temps entre la demande du client et la remise de
  la clé. Le corpus lui donne deux budgets et un seuil d'échec : 30 s pour un passage, 60 s pour un
  client connu, **90 s = contournement**.
- **Adjoua (gérante de site)** — cumule gérante, caissière et réceptionniste. Elle fait les
  départs : la note, le calcul final, la régularisation d'un départ anticipé, la prolongation. Zone
  de charme — le client attend son papier, pas sa clé. C'est aussi elle qui tranche un conflit de
  prolongation.
- **M. Koffi (propriétaire)** — ne saisit rien, il lit. Ce cycle le concerne par deux points : la
  **part des passages réellement saisis** (indicateur du §18 du cadrage, et l'argument de vente du
  produit), et le fait que **toute régularisation, tout changement de chambre et toute prolongation
  soient retrouvables au registre des actions** — l'écart que le cahier papier ne lui montrait pas.

**Ce que ce cycle ne fait PAS et qui pourrait être supposé** : il n'encaisse pas, il ne certifie
pas, il n'imprime pas de facture, il ne porte pas les consommations des points de vente sur la
note, il ne gère pas le client sans hébergement, il ne lit pas de pièce d'identité à la caméra, et
il ne pose pas de réservation. Il livre **la personne, le séjour, l'arrivée, le départ et leurs
traces**.

## Clarifications

### Session 2026-08-03 — points tranchés par le corpus

Les cinq points suivants avaient plusieurs lectures plausibles ; l'ordre de préséance des sources
les tranche sans arbitrage humain. Ils sont consignés pour qu'une relecture n'y voie pas un oubli.

- **Q : La maquette `R4-passage-hors-ligne` montre l'écran du passage utilisable sans réseau. Le
  passage est-il donc enregistrable hors ligne ?**
  **R : Non — et la maquette ne dit pas le contraire.** `docs/registre-classes-offline.md` §8
  classe le check-in en **B**, et le cadrage §11.1 réserve l'écriture B au **mode nœud de site**,
  qui n'existe pas au MVP. Ce que la maquette montre hors ligne est **la lecture** : les durées,
  les prix (« Le prix est sur le bouton. Rien ici n'a besoin du réseau. ») et l'état des chambres
  avec sa fraîcheur affichée — toutes de classe A. **La remise de la clé, elle, est refusée
  immédiatement et explicitement**, jamais grisée, jamais mise en file (principe VI, porte P-13).
  Le registre prime sur la maquette (préséance 4 contre 7).

- **Q : Le check-out doit-il générer le document fiscal, puisque SEJ-04 le dit ?**
  **R : Non — il expose le point d'ancrage, il ne certifie pas.** SEJ-04 renvoie explicitement à
  **FIS-02**, qui est de tranche **T3**. Le principe V et la porte **P-12** interdisent par
  ailleurs toute règle fiscale hors du trait `JurisdictionAdapter`. Ce cycle **fige l'assiette** de
  la taxe de séjour et le fait de la clôture ; le montant et la facture viennent avec FIS. Voir
  FR-062 à FR-066 et § Out of Scope.

- **Q : Le check-out encaisse-t-il ?**
  **R : Non.** L'encaissement est **CAI-03**, tranche T2. La maquette `R7` affiche « L'argent est
  encaissé, en espèces » — c'est l'état de l'écran **une fois T2 et T3 livrés**, pas une exigence
  de ce cycle. Le séjour se clôt sur une note **arrêtée et non réglée**, et l'écran le dit en toutes
  lettres plutôt que de laisser croire à un paiement.

- **Q : Le mot « check-in » atteint-il l'interface ou une URL ?**
  **R : Non — les mots visibles sont « arrivée » et « départ ».** La leçon de `S1` au cycle 005 est
  explicite : *« le nom du fichier de page décide de la route, et une URL est visible »* — c'est ce
  qui a fait renommer `/synchronisation` en `/mes-envois`. « Check-in » est du jargon anglais absent
  du lexique ; les maquettes disent « Arrivée », « Départ », « Le passage ». Les routes suivent.

- **Q : Le séjour porte-t-il déjà des lignes de note, alors que SEJ-03 est en T2 ?**
  **R : Oui, les lignes d'hébergement seulement.** SEJ-02 exige « l'ouverture de la note » et
  SEJ-04 exige un « calcul final » — un calcul final sur une note vide n'a pas de sens. Ce cycle
  écrit donc la **ligne d'hébergement** et ses **lignes d'ajustement** (dépassement, départ
  anticipé, changement d'unité). SEJ-03 ajoutera les consommations des points de vente, les
  transferts de charges et les remises. La ligne du registre (`ligne_sejour`, classe **B**) est
  **honorée, pas réécrite** — le registre déclare d'avance, c'est son usage établi.

### Session 2026-08-03 — décisions tranchées au terrain

Deux décisions du corpus arrivaient à échéance **à ce cycle** ; une troisième portait sur un
livrable réglementaire non cartographié. **Les trois sont tranchées.** Aucun marqueur
`[NEEDS CLARIFICATION]` ne subsiste.

- **Q1 — O-01 : un client jamais vu, en coupure réseau. → Option (a) : `client` reste en classe C.**
  Une fiche nouvelle exige le réseau. **Le modèle ne change pas**, et c'est ce qui rend la décision
  peu coûteuse : au MVP, la question est de toute façon sans effet visible, puisque l'arrivée
  elle-même est de classe **B** et donc déjà refusée hors ligne — ce que O-01 décrivait mordra
  seulement quand le **nœud de site** arrivera (incrément 3), où l'arrivée deviendra possible hors
  ligne alors qu'une **fiche nouvelle** restera impossible. Cette friction résiduelle est **acceptée
  et nommée** : au comptoir, en coupure, un client inconnu s'enregistre sans fiche, et sa fiche se
  crée au retour du réseau — ce que le parcours de passage rend naturel, la pièce d'identité se
  saisissant déjà **après la clé** (`R4`). Voir FR-011.

- **Q2 — B-10 : l'axe « par personne » de la taxe de séjour. → La taxe est due PAR SÉJOUR, pas par
  personne.** Deux personnes une nuit à 500 F/nuit paient **500 F**, pas 1 000 F. **L'axe des
  personnes n'existe pas** : il n'y a donc **ni multiplication par le nombre d'occupants, ni
  exonération par personne, ni colonne de motif** — et cette absence est une **décision consignée,
  pas un oubli** (annexe B). Voir FR-018, FR-020, FR-062, FR-064.

  ⚠️ **Cette décision contredit trois écrits du corpus** — `docs/cadrage-v1.md` §9.6 (« par nuitée
  **et par client** »), FIS-03 (« par nuitée et par client, **accompagnants inclus** ») et FIS-08
  (« nuitées assujetties, **nombre de clients**, montant dû »). L'arbitrage terrain prime, et les
  trois amendements sont **dus avant `/speckit-plan`** : voir § Suites documentaires dues. Sans eux,
  le cycle FIS re-dériverait la règle inverse depuis une source de rang supérieur à la présente
  spécification.

  **Ce que la décision ne touche pas** : l'**axe des nuits** reste le paramètre existant
  `regle_conversion_taxe`, posé au cycle 004 et inchangé — Deloria est semé
  `une_nuitee_par_occupation`, soit « **Une seule taxe pour tout le séjour** » (lexique, formulation
  validée au terrain le 2026-08-02). L'exemple donné à l'arbitrage porte sur une seule nuit et ne
  distingue donc pas cet axe ; si la pratique de Deloria est en réalité **une taxe par nuit**, c'est
  le **seed** de la règle de conversion qu'il faut changer, pas ce cycle — la valeur est éditable
  formule par formule et aucune règle n'est en dur.

  **Le nombre de personnes reste enregistré et figé au départ** bien qu'il n'entre pas dans
  l'assiette. Il est dû à la fiche de police, il documente l'occupation réelle de l'unité, et il
  coûte zéro : si une commune facturait un jour par personne, la donnée historique existe.

- **Q3 — le contenu de la fiche de police. → Option (a) : registre minimal, format officiel
  différé.** La fiche porte le titulaire et ses accompagnants avec leur identité, la période,
  l'unité, l'établissement et son numéro de document. Le **gabarit officiel** du pilote reste à
  cartographier (cadrage §9.7) : c'est un **rendu**, pas une donnée, et il s'ajoutera sans migration
  quand le formulaire réel sera fourni. Voir FR-045 à FR-049 et § Out of Scope.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Le passage en deux gestes (Priority: P1)

Yao est au comptoir. Un client se présente et demande une chambre pour deux heures. Yao ouvre
l'écran du passage : les durées sont là avec leur prix et leur heure de fin, les chambres sont là
avec leur état. Il touche « 2 h », il touche une chambre libre, il donne la clé. **C'est tout.** La
pièce d'identité se saisit après, pendant que le client monte ; le téléphone aussi, s'il le donne.
L'écran confirme « C'est fait — chambre 103 donnée pour 2 h », rappelle l'heure de fin à redire au
client, et propose « Client suivant ».

**Why this priority** : c'est la story dont l'échec condamne le produit. Le cadrage §5.6 dit qu'un
passage lent sera **contourné**, pas critiqué — et un passage contourné est une chambre vendue en
espèces hors du système, donc l'argument de vente du produit perdu en même temps que la donnée. Le
parcours est aussi **distinct** de celui de la nuitée : ce n'est pas le même écran avec moins de
champs, c'est un écran dont la grammaire entière est le tap.

**Independent Test** : entièrement testable seule. Un établissement, une catégorie, un barème de
passage, trois chambres libres — livré par le cycle 004. On chronomètre le parcours du premier
geste à l'écran de confirmation, on compte les gestes obligatoires, et on vérifie qu'aucun champ de
saisie libre n'est exigé avant la remise de la clé.

**Acceptance Scenarios** :

1. **Given** un établissement avec des chambres libres et un barème de passage, **When** Yao touche
   une durée puis une chambre, **Then** le séjour est ouvert, l'unité attribuée sur l'intervalle
   `[maintenant, maintenant + durée)` augmenté du temps de remise en état, la note ouverte avec sa
   ligne d'hébergement au prix du palier, et l'écran de confirmation affiché — **en deux gestes,
   sans aucun champ de saisie obligatoire**.
2. **Given** le parcours de passage, **When** on compte les interactions obligatoires du premier
   geste à la confirmation, **Then** il y en a **exactement deux**, et le nombre d'appels réseau
   bloquants est **au plus un**.
3. **Given** un passage enregistré, **When** Yao saisit ensuite le téléphone ou le numéro de pièce
   du client, **Then** la fiche client est créée ou rattachée **sans rouvrir le séjour** et sans
   remettre en cause l'attribution déjà faite.
4. **Given** un client déjà connu dont le téléphone est saisi ou reconnu, **When** l'écran du
   passage s'ouvre, **Then** il affiche « M. Bakayoko — 7ᵉ passage » avec une sortie explicite
   (« Ce n'est pas lui »), et **la pièce déjà enregistrée n'est pas redemandée**.
5. **Given** toutes les chambres occupées, **When** Yao ouvre l'écran, **Then** l'état vide illustré
   annonce combien de chambres sont prises et **liste ce qui se libère avec l'heure**, la plus
   proche en tête.
6. **Given** l'appareil hors ligne, **When** Yao ouvre l'écran du passage, **Then** les durées, les
   prix et l'état des chambres restent lisibles avec leur fraîcheur affichée, **et l'attribution est
   refusée immédiatement et explicitement** — jamais grisée en silence, jamais mise en file.
7. **Given** deux réceptionnistes touchant la même chambre au même instant, **When** les deux
   attributions partent, **Then** **exactement une** réussit, et le refus de l'autre vient de la
   contrainte d'exclusion de la base, pas d'une vérification applicative.

---

### User Story 2 - La fiche client, trouvée en un souffle (Priority: P1)

Yao tape les trois premières lettres d'un nom, ou les quatre derniers chiffres d'un téléphone, ou
un numéro de pièce. La liste se réduit pendant qu'il tape. Il ouvre une fiche : identité,
coordonnées, préférences, et **l'historique de ses séjours** dans tous les établissements du
tenant. La fiche est celle du tenant, pas celle de l'établissement : un client connu à Abengourou
est connu à Yopougon.

**Why this priority** : sans elle, « client connu → pré-remplissage complet, aucune ressaisie »
n'existe pas, et la cible de 60 secondes de la nuitée est inatteignable. C'est aussi la seule
exigence du module portant un chiffre de performance explicite : **moins de 300 ms sur
10 000 fiches**.

**Independent Test** : testable seule, sans aucun séjour. On charge un jeu de mesure de 10 000
fiches, on lance les trois formes de recherche, on mesure. L'historique se teste avec des séjours
fabriqués.

**Acceptance Scenarios** :

1. **Given** 10 000 fiches clients dans un tenant, **When** une recherche par nom, par téléphone ou
   par numéro de pièce est lancée, **Then** les résultats reviennent en **moins de 300 ms au 95ᵉ
   centile**, mesuré côté serveur sur le jeu de mesure.
2. **Given** un client saisi « KOUAMÉ », **When** on cherche « kouame », **Then** il est trouvé —
   la recherche par nom ignore la casse et les signes diacritiques.
3. **Given** un téléphone enregistré `+225 07 12 34 56 78`, **When** on cherche `07123456`,
   `0712345678` ou `+22507123456 78`, **Then** la fiche est trouvée : les numéros sont comparés sous
   forme normalisée, avec l'indicatif par défaut de l'établissement (`indicatif_telephonique_defaut`,
   CPT-01).
4. **Given** deux établissements d'un même tenant, **When** une fiche est créée depuis l'un,
   **Then** elle est trouvable depuis l'autre — et **jamais depuis un autre tenant**.
5. **Given** une fiche client ouverte, **When** son historique est consulté, **Then** il liste les
   séjours de tous les établissements du tenant, du plus récent au plus ancien, avec établissement,
   unité, période et formule.
6. **Given** un compte sans la permission de gestion des fiches clients, **When** il ouvre la
   recherche, **Then** l'action de création est **absente**, pas grisée.
7. **Given** l'appareil hors ligne, **When** une création ou une modification de fiche est tentée,
   **Then** elle est refusée immédiatement et explicitement (classe C).

---

### User Story 3 - L'arrivée d'un client connu en moins de soixante secondes (Priority: P1)

Un client attendu se présente pour deux nuits. Yao le retrouve d'un mot, l'écran se remplit tout
seul : nom, prénoms, téléphone, pièce, préférences. Il choisit la formule et la période — les heures
d'arrivée et de départ standard de l'établissement sont déjà posées. Le système **propose une unité
disponible** de la catégorie voulue ; Yao l'accepte ou en choisit une autre. Il ajoute deux
accompagnants — un nom suffit. La chambre est attribuée, la fiche de police est produite, la note
est ouverte. **Aucune information déjà connue n'a été retapée.**

**Why this priority** : c'est le parcours de référence de l'hôtellerie classique et la moitié de la
démo de fin de tranche T1. Sa cible — 60 secondes pour un client connu — est portée par SEJ-02 et
n'est tenable que si le pré-remplissage est **total**.

**Independent Test** : testable seule dès que les fiches clients existent. On compte les champs
pré-remplis face aux champs saisis, on chronomètre, on vérifie que la proposition d'unité tombe sur
une unité réellement libre sur l'intervalle demandé.

**Acceptance Scenarios** :

1. **Given** un client connu et une catégorie ayant au moins une unité libre, **When** Yao
   sélectionne la formule et la période, **Then** une unité disponible est **proposée
   automatiquement**, et la proposition est vérifiée libre sur l'intervalle demandé, temps de remise
   en état inclus.
2. **Given** un client connu, **When** l'écran d'arrivée s'ouvre sur lui, **Then** **aucun** champ
   déjà renseigné sur sa fiche n'est vide ni à retaper, et le nombre de champs saisis pour conclure
   l'arrivée est **zéro** hors formule, période et accompagnants.
3. **Given** la formule nuitée d'un établissement, **When** la période est ouverte, **Then**
   l'heure d'arrivée et l'heure de départ standard (`heure_arrivee_standard`,
   `heure_depart_standard`) sont déjà appliquées et modifiables.
4. **Given** deux accompagnants ajoutés, **When** le séjour est ouvert, **Then** le nombre de
   personnes du séjour est de trois, cette valeur figure sur la **fiche de police** et sera **figée
   au départ** — et elle **ne multiplie pas** la taxe de séjour, due par séjour et non par personne.
5. **Given** une catégorie dont toutes les unités sont prises sur l'intervalle, **When** la
   proposition est demandée, **Then** le refus est explicite et nomme la première disponibilité
   ultérieure — jamais une liste vide sans explication.
6. **Given** une arrivée conclue, **When** on inspecte l'état du système, **Then** il existe un
   séjour ouvert, une occupation active, une fiche de police produite, une note ouverte portant sa
   ligne d'hébergement, un événement outbox, et une entrée au registre des actions.
7. **Given** un compte sans la permission d'attribution, **When** il ouvre l'écran d'arrivée,
   **Then** l'action de confirmation est **absente**.

---

### User Story 4 - Le départ, et la taxe figée (Priority: P1)

Le client descend. Adjoua ouvre sa note : l'hébergement nuit par nuit, les sous-totaux, le nombre
de personnes. Elle arrête la note. À cet instant, et à cet instant seulement, **l'assiette de la
taxe de séjour est figée** : tant de nuitées assujetties, tant de personnes, la règle de conversion
de la formule appliquée. Le séjour est clos, plus rien ne peut s'y ajouter, la chambre est libérée
pour le ménage. La note porte la mention « Document non fiscal — ne tient pas lieu de facture ».

**Why this priority** : le départ est la seule opération du produit dont une erreur devient
irrattrapable — une taxe recalculée après coup impose un avoir certifié (cadrage §9.6), et un avoir
FNE se fait **par quantité, pas par montant** (§9.4). Figer au bon instant coûte une ligne ; ne pas
figer coûte une procédure.

**Independent Test** : testable seule sur un séjour fabriqué. On clôt, on modifie ensuite les
accompagnants ou le barème, on relit : l'assiette figée n'a pas bougé.

**Acceptance Scenarios** :

1. **Given** un séjour ouvert, **When** le départ est prononcé, **Then** la durée réelle est
   calculée **exclusivement sur l'horodatage d'autorité serveur**, le montant d'hébergement final
   est arrêté, et l'assiette de la taxe de séjour est **figée avec l'instant du figeage**.
2. **Given** un séjour clos, **When** un accompagnant est ajouté, le barème modifié ou la formule
   éditée, **Then** l'assiette figée **est inchangée**, et aucune relecture ne la recalcule.
3. **Given** un séjour clos, **When** une écriture est tentée sur sa note, **Then** elle est refusée
   — la note est arrêtée.
4. **Given** un passage dont la durée réelle dépasse le palier acheté, **When** le départ est
   prononcé, **Then** la **rebascule de palier** du cycle 004 s'applique, la différence est portée
   en **ligne d'ajustement distincte** — jamais par modification de la ligne initiale — et tracée au
   registre des actions avec la durée constatée et les deux paliers.
5. **Given** un séjour clos, **When** l'unité est consultée, **Then** son occupation est libérée à
   l'instant réel du départ, le temps de remise en état s'applique, et son statut d'occupation est
   **dérivé** — jamais posé à la main.
6. **Given** une formule non assujettie à la taxe de séjour, **When** le départ est prononcé,
   **Then** l'assiette figée porte zéro nuitée assujettie **et conserve la trace de la règle
   appliquée** — l'absence de taxe est un fait établi, pas un silence.
7. **Given** l'appareil hors ligne, **When** un départ est tenté, **Then** il est refusé
   immédiatement et explicitement (classe B).

---

### User Story 5 - La prolongation, et le conflit dit en face (Priority: P2)

Le client veut rester une nuit de plus. Adjoua demande la prolongation. Le système vérifie la
disponibilité **sur l'intervalle étendu**, temps de remise en état compris. Si la chambre est libre,
la prolongation est faite et la note s'allonge. Si la chambre est réservée derrière, **le conflit
est nommé** : qui vient, à quelle heure, et quelles sont les options — une autre chambre libre sur
la période, ou refuser.

**Why this priority** : la prolongation est fréquente et son échec silencieux produit exactement ce
que la contrainte d'exclusion existe pour empêcher. Elle est en P2 parce que le séjour doit exister
avant d'être prolongé, pas parce qu'elle est secondaire.

**Independent Test** : testable seule sur un séjour ouvert. Deux cas suffisent : intervalle étendu
libre, intervalle étendu occupé par une occupation suivante.

**Acceptance Scenarios** :

1. **Given** un séjour en cours et l'intervalle étendu libre, **When** la prolongation est demandée,
   **Then** l'occupation est étendue, la note reçoit ses lignes d'hébergement supplémentaires au
   tarif en vigueur, et l'opération est tracée.
2. **Given** un séjour en cours et une occupation suivante sur la même unité, **When** la
   prolongation est demandée, **Then** elle est **refusée avec le conflit nommé** — unité, instant
   de début de l'occupation suivante — et non par un message générique.
3. **Given** un conflit sur l'unité courante, **When** le refus est affiché, **Then** les unités de
   la même catégorie libres sur l'intervalle étendu sont proposées, ce qui ouvre sur le changement
   d'unité.
4. **Given** une prolongation acceptée, **When** on inspecte les occupations, **Then** la garantie
   de non-chevauchement reste portée par la contrainte d'exclusion de la base, y compris avec le
   temps de remise en état.
5. **Given** un séjour clos, **When** une prolongation est demandée, **Then** elle est refusée : on
   ne prolonge pas un séjour terminé.

---

### User Story 6 - Le départ anticipé, régularisé et tracé (Priority: P2)

Le client part la veille de ce qui était prévu. Adjoua prononce le départ. Le montant est recalculé
sur la durée réellement occupée, la différence apparaît comme une **régularisation** identifiée,
avec son motif, son auteur et son instant. Ce que le client paie et pourquoi est lisible sur la
note sans explication orale.

**Why this priority** : c'est le cas où l'argent bouge en faveur du client, donc celui que le
propriétaire veut pouvoir vérifier. Le cadrage §8.3 fait du journal d'audit « ce que le
propriétaire achète ».

**Independent Test** : testable seule. Un séjour de trois nuits clos après deux, et l'on vérifie la
note, la régularisation et sa trace.

**Acceptance Scenarios** :

1. **Given** un séjour prévu pour trois nuits, **When** le départ est prononcé après deux nuits,
   **Then** l'hébergement est arrêté sur la durée réelle, la différence est portée en **ligne de
   régularisation distincte**, et la ligne initiale n'est **pas** modifiée.
2. **Given** une régularisation, **When** le registre des actions est consulté, **Then** on y trouve
   son auteur, son instant d'autorité, son montant, le séjour concerné et le motif retenu.
3. **Given** un départ anticipé, **When** l'assiette de la taxe de séjour est figée, **Then** elle
   porte le nombre de nuitées **réellement** assujetties, pas le nombre prévu.
4. **Given** un départ anticipé, **When** l'occupation est libérée, **Then** l'unité redevient
   disponible à partir de l'instant réel du départ augmenté du temps de remise en état — la
   disponibilité rendue est immédiate, pas différée à l'heure initialement prévue.

---

### User Story 7 - Le changement de chambre, avec son histoire (Priority: P3)

La climatisation de la 204 tombe en panne la deuxième nuit. Adjoua déplace le client en 207. Le
séjour reste **un seul séjour** ; l'occupation de la 204 est close à l'instant du déplacement, une
occupation de la 207 s'ouvre à partir de là. La note garde les deux, nommées, avec leurs périodes.
L'historique du client montre le séjour et ses deux chambres.

**Why this priority** : sans elle, un changement se fait par « clore et rouvrir », ce qui casse
l'historique, la note et l'assiette de la taxe. En P3 parce qu'elle est moins fréquente que les
six autres, jamais parce qu'elle serait optionnelle : SEJ-04 l'exige nommément.

**Independent Test** : testable seule. Un séjour, un déplacement, et l'on vérifie qu'il y a deux
occupations, un séjour, et une note continue.

**Acceptance Scenarios** :

1. **Given** un séjour en cours en 204, **When** le client est déplacé en 207, **Then** il existe
   **deux occupations** rattachées au **même séjour**, contiguës sur l'instant du déplacement, et
   l'historique conserve les deux.
2. **Given** un changement d'unité, **When** l'unité cible n'est pas libre sur la période restante,
   **Then** le déplacement est refusé avec le conflit nommé — jamais un déplacement partiel.
3. **Given** un changement d'unité entre deux catégories de tarif différent, **When** la note est
   consultée, **Then** chaque période porte le tarif de son unité, et le changement est tracé au
   registre des actions avec les deux unités et l'instant.
4. **Given** un séjour à deux occupations, **When** le départ est prononcé, **Then** l'assiette de
   la taxe de séjour est figée **sur l'ensemble du séjour**, pas par occupation.
5. **Given** un changement d'unité, **When** l'unité d'origine est consultée, **Then** son
   occupation est close à l'instant du déplacement et son temps de remise en état s'applique.

---

### Edge Cases

- **Un passage dont personne ne ressort.** La durée achetée est écoulée, la chambre n'est pas
  rendue. L'unité reste occupée, la rebascule de palier s'applique au départ effectif, et l'écart
  est visible. Le système **ne clôt jamais un séjour tout seul** : une clôture automatique
  produirait une facturation sans témoin.
- **Un passage dont la pièce d'identité n'est jamais saisie.** Le client repart sans l'avoir
  donnée. Le séjour existe, la fiche de police est **incomplète et identifiée comme telle** ; elle
  n'est ni fabriquée, ni silencieusement omise.
- **Un client qui existe deux fois.** Deux fiches pour la même personne, créées dans deux
  établissements du tenant. La recherche les montre toutes deux ; **ce cycle ne fusionne pas** — il
  ne crée pas non plus de doublon silencieux à l'insu de l'opérateur (voir Q1, dont l'option (b) ou
  (c) rendrait la fusion obligatoire).
- **Le téléphone d'un client est celui d'un autre.** Deux fiches partagent un numéro (couple,
  entreprise). La reconnaissance au téléphone propose **le choix**, jamais une reconnaissance
  d'office.
- **La réservation suivante commence pendant le temps de remise en état.** Le conflit existe alors
  même que les intervalles clients ne se touchent pas. Il est signalé sur l'intervalle **réel**,
  remise en état comprise — c'est cet intervalle que la base protège.
- **Une horloge de terminal qui dérive d'une heure.** Le montant et la durée sont identiques : tout
  s'appuie sur l'horodatage d'autorité (porte P-23). Le terminal affiche l'alerte de dérive du
  lexique, avec sa seconde phrase obligatoire.
- **Un départ prononcé avant l'instant d'arrivée.** Impossible : l'intervalle serait vide, ce que
  la base refuse déjà.
- **Une prolongation qui fait basculer un passage en nuitée.** Le seuil `seuil_bascule_nuitee_minutes`
  du cycle 004 s'applique ; l'utilisateur en est prévenu **avant** de confirmer, avec le nouveau
  montant.
- **Un séjour clos qui reçoit une écriture tardive.** C'est l'écriture orpheline du cadrage §11.4 :
  elle part en **file de réconciliation à résolution humaine**, jamais de rejet silencieux, jamais
  d'ajout d'office. La table `reconciliation_orpheline` existe depuis le cycle 005 ; **sa résolution
  est SYN-03, tranche T3** — ce cycle alimente la file, il ne la vide pas.
- **Deux accompagnants du même nom.** Aucun blocage : un accompagnant n'a pas d'unicité, c'est une
  écriture de classe A.
- **Un accompagnant retiré après le départ.** Refusé : l'assiette est figée et le séjour est clos.
- **Le module hébergement est inactif sur l'établissement.** Les écrans d'arrivée, de passage et de
  départ sont **absents** — pas grisés (principe VII). La recherche de fiches clients, elle, reste
  disponible : elle ne dépend d'aucun module d'activité.

## Requirements *(mandatory)*

### Functional Requirements

#### A. Fiche client et recherche (SEJ-01)

- **FR-001** : Le système MUST porter une entité `client` rattachée au **tenant**, avec au minimum
  nom, prénoms, date de naissance, nationalité, pièce d'identité, téléphone, courriel et
  préférences — les huit attributs nommés par SEJ-01, sans en retrancher.
- **FR-002** : Une fiche client MUST être visible et utilisable depuis **tous les établissements du
  tenant**, et depuis aucun autre tenant.
- **FR-003** : Le système MUST offrir une recherche par **nom**, par **téléphone** et par **numéro
  de pièce d'identité**, les trois formes servies par la même entrée de recherche.
- **FR-004** : La recherche par nom MUST ignorer la casse et les signes diacritiques, et MUST porter
  sur le nom comme sur les prénoms.
- **FR-005** : La recherche par téléphone MUST comparer des numéros **normalisés**, en appliquant
  l'indicatif par défaut de l'établissement (`indicatif_telephonique_defaut`) aux numéros saisis
  sans indicatif.
- **FR-006** : La recherche MUST renvoyer ses résultats en **moins de 300 ms au 95ᵉ centile sur
  10 000 fiches**, mesuré côté serveur, sur un jeu de mesure reproductible.
- **FR-007** : Le jeu de mesure de 10 000 fiches MUST être généré par la commande de test et MUST
  NOT être chargé dans les tenants de démonstration.
- **FR-008** : Une fiche client MUST donner accès à **l'historique de ses séjours** sur tous les
  établissements du tenant, du plus récent au plus ancien, avec établissement, unité, période et
  formule.
- **FR-009** : Les **préférences** d'un client, ses notes internes et sa photo MUST être de classe
  **A** (registre §8) : écrivables hors ligne, rejeu idempotent, application commutative.
- **FR-010** : La **création et la modification** d'une fiche client MUST être de classe **C** :
  refusées immédiatement et explicitement hors ligne, jamais mises en file.
- **FR-011** : *(Décision O-01, option (a) — 2026-08-03.)* Un client **jamais vu** MUST exiger le
  réseau pour que sa fiche existe. Le système MUST NOT créer de fiche locale provisoire, MUST NOT
  produire de doublon à l'insu de l'opérateur, et MUST permettre d'enregistrer un séjour **sans
  fiche client**, la fiche étant créée ou rattachée ensuite (FR-028). La friction résiduelle — en
  mode nœud de site, une arrivée sera possible hors ligne alors qu'une fiche nouvelle ne le sera pas
  — est **acceptée et documentée**, non contournée.
- **FR-012** : Le numéro de pièce d'identité MUST être protégé au repos et son accès MUST être
  journalisé, **dès ce cycle** — la donnée naît ici, et TRX-06 (P1) apporte l'export, la suppression
  et la purge paramétrable, pas la protection.
- **FR-013** : Le système MUST enregistrer l'instant de capture de la pièce d'identité, afin que la
  rétention paramétrable de TRX-06 (90 jours par défaut) s'applique plus tard **sans migration**.
- **FR-014** : Les fiches clients MUST être lisibles et modifiables selon des permissions dédiées, et
  toute action non permise MUST être **absente** de l'interface, jamais grisée.

#### B. Accompagnants

- **FR-015** : Le système MUST permettre d'enregistrer des **accompagnants** sur un séjour, avec au
  minimum un nom ; les autres attributs d'identité sont facultatifs.
- **FR-016** : L'ajout d'un accompagnant MUST être de classe **A** (registre §8) : écriture hors
  ligne autorisée, rejeu triple produisant un seul enregistrement, désordre commutatif.
- **FR-017** : Le **nombre de personnes** d'un séjour MUST être dérivé du client titulaire et de ses
  accompagnants, jamais saisi en double.
- **FR-018** : *(Décision B-10 — 2026-08-03.)* Le nombre de personnes MUST NOT entrer dans
  l'assiette de la taxe de séjour : **la taxe est due par séjour, jamais par personne.** Deux
  personnes une nuit produisent **une** taxe, pas deux. Le nombre de personnes MUST malgré tout être
  **enregistré et figé au départ**, parce qu'il est dû à la fiche de police, qu'il documente
  l'occupation réelle de l'unité, et qu'il ne coûte rien à conserver.
- **FR-019** : Un accompagnant MUST NOT être ajoutable ni retirable sur un séjour clos.
- **FR-020** : *(Décision B-10 — 2026-08-03.)* Le système MUST NOT porter de **motif d'exonération
  par personne** : l'axe des personnes n'existant pas, une exonération par personne n'a pas d'objet.
  Cette absence MUST être consignée comme une **décision**, au registre des classes comme au dépôt,
  et non laissée à l'interprétation d'une relecture.

#### C. Le parcours de passage — distinct et ultra-court (SEJ-02)

- **FR-021** : Le passage MUST avoir **son propre parcours**, distinct de celui de la nuitée. Ce
  n'est pas le parcours de nuitée avec des champs masqués : c'est un écran dont la grammaire est le
  tap, conforme à la maquette normative `R4`.
- **FR-022** : Le parcours de passage MUST se conclure en **deux gestes obligatoires** — la durée,
  puis l'unité — de l'ouverture de l'écran à la confirmation.
- **FR-023** : Le parcours de passage MUST NOT exiger **aucun champ de saisie libre** avant la
  remise de la clé. La pièce d'identité et le téléphone se saisissent **après**, conformément à la
  mention normative de `R4` : « Pièce d'identité : après la clé, pas avant ».
- **FR-024** : Chaque durée proposée MUST afficher **son prix et son heure de fin**, calculés par le
  moteur de tarification du cycle 004, sans que l'utilisateur ait à les demander.
- **FR-025** : L'écran MUST afficher l'état de toutes les unités de l'établissement — libre,
  occupée avec son heure de fin, à nettoyer — et l'attribution MUST se faire d'un seul tap sur une
  unité libre.
- **FR-026** : Après l'attribution, l'écran de confirmation MUST rappeler **l'unité, la durée et
  l'heure de fin à redire au client**, et proposer l'enchaînement « client suivant ».
- **FR-027** : Quand toutes les unités sont prises, l'écran MUST afficher un état vide **utile** :
  le nombre d'unités prises et **la liste de ce qui se libère avec l'heure**, la plus proche en tête.
- **FR-028** : Le rattachement d'un client à un passage déjà enregistré MUST être possible **sans
  rouvrir ni modifier l'attribution**.
- **FR-029** : Quand un client est reconnu, l'écran MUST le nommer avec son rang de passage et MUST
  offrir une sortie explicite (« Ce n'est pas lui »), et MUST NOT redemander une pièce déjà
  enregistrée.
- **FR-030** : Hors ligne, l'écran de passage MUST rester **consultable** — durées, prix, état des
  unités avec fraîcheur affichée — et l'attribution MUST être refusée **immédiatement et
  explicitement**.
- **FR-031** : Le parcours de passage MUST NOT dépasser **un appel réseau bloquant** entre le
  premier geste et la confirmation.
- **FR-032** : L'attribution issue d'un passage MUST porter l'intervalle
  `[instant d'autorité, instant d'autorité + durée)` augmenté du **temps de remise en état** de la
  catégorie et de la formule.
- **FR-033** : L'instant de début du passage MUST provenir **exclusivement de l'horodatage
  d'autorité serveur**, jamais de l'horloge du terminal (principe IV, porte P-23, cadrage §11.4).

#### D. Le parcours d'arrivée — nuitée, demi-journée, mensuel (SEJ-02)

- **FR-034** : Le parcours d'arrivée MUST permettre de choisir la **formule**, la **période** et la
  **catégorie**, et MUST proposer **automatiquement une unité disponible** de cette catégorie sur
  l'intervalle demandé.
- **FR-035** : Pour un client connu, le parcours MUST **pré-remplir intégralement** ce que porte sa
  fiche. Le nombre de champs à ressaisir MUST être **zéro** hors formule, période et accompagnants.
- **FR-036** : Pour la formule nuitée, l'heure d'arrivée et l'heure de départ standard de
  l'établissement (`heure_arrivee_standard`, `heure_depart_standard`) MUST être appliquées d'office
  et rester modifiables.
- **FR-037** : L'unité proposée MUST être vérifiée libre sur l'intervalle demandé, **temps de remise
  en état inclus**, au moment de la proposition.
- **FR-038** : L'utilisateur MUST pouvoir refuser l'unité proposée et en choisir une autre parmi
  celles réellement libres sur l'intervalle.
- **FR-039** : Quand aucune unité de la catégorie n'est disponible, le refus MUST nommer **la
  première disponibilité ultérieure** plutôt que de renvoyer une liste vide.
- **FR-040** : L'attribution MUST être de classe **B** (registre §7.2 et §8) et sa garantie de
  non-chevauchement MUST provenir de la **contrainte d'exclusion de la base**, jamais d'une
  vérification applicative préalable.
- **FR-041** : L'attribution MUST être gardée par la permission d'attribution d'unité
  (`heb.unite.attribuer`, posée au cycle 004), et l'action MUST être **absente** sans elle.
- **FR-042** : Le système MUST refuser une arrivée sur un établissement dont le module hébergement
  n'est pas actif, et l'entrée correspondante MUST être **absente** de l'accueil.
- **FR-043** : L'ouverture d'un séjour MUST émettre un **événement outbox dans la même transaction**
  (principe II, porte P-05) et une entrée au **registre des actions**.
- **FR-044** : Une arrivée MUST être refusée **immédiatement et explicitement** hors ligne
  (classe B, porte P-13).

#### E. Fiche de police

- **FR-045** : Le système MUST produire une **fiche de police** à l'ouverture de chaque séjour
  (SEJ-02), de classe **B** et **numérotée** (registre §8).
- **FR-046** : La fiche de police MUST couvrir le client titulaire **et ses accompagnants**.
- **FR-047** : Une fiche de police dont l'identité n'est pas complète MUST être **identifiée comme
  incomplète** ; elle MUST NOT être fabriquée avec des valeurs de remplissage, ni silencieusement
  omise.
- **FR-048** : La fiche de police est un **document opérationnel**, jamais fiscal : elle porte la
  mention « Document non fiscal — ne tient pas lieu de facture » et suit une numérotation interne à
  l'établissement, distincte de toute numérotation fiscale (principe V, FIS-02).
- **FR-049** : *(Décision Q3, option (a) — 2026-08-03.)* La fiche de police MUST porter le **registre
  minimal** : pour le titulaire et pour chaque accompagnant, nom, prénoms, date de naissance,
  nationalité et numéro de pièce d'identité ; pour le séjour, l'établissement, l'unité, la période et
  le numéro de document. Le **gabarit officiel** du pilote MUST NOT être inventé : c'est un **rendu**
  qui s'ajoutera sans migration quand le formulaire réel sera fourni (cadrage §9.7, « à cartographier
  avec le pilote »).

#### F. Le séjour et sa note

- **FR-050** : Le système MUST porter une entité `sejour` liant un **client**, un **établissement**,
  une ou plusieurs **occupations** et une **note**, avec un cycle de vie `en_cours → clos`.
- **FR-051** : Un séjour MUST pouvoir porter **plusieurs occupations successives**, ce qui rend le
  changement d'unité possible sans casser l'historique.
- **FR-052** : L'ouverture d'un séjour MUST **ouvrir sa note** (SEJ-02) et y inscrire la **ligne
  d'hébergement** de la période prévue, au tarif issu du moteur de tarification du cycle 004.
- **FR-053** : Le prix d'une ligne MUST être **verrouillé à sa création** (principe V). Tout écart
  ultérieur — dépassement, rebascule de palier, départ anticipé, changement d'unité, prolongation —
  MUST produire une **ligne d'ajustement distincte**, jamais une modification de la ligne existante.
- **FR-054** : Les montants MUST être des **entiers d'unité mineure** et les quantités des valeurs
  **décimales**, jamais entières (principe V, porte P-10).
- **FR-055** : Une note ouverte MUST afficher son total provisoire, et une note arrêtée MUST refuser
  toute écriture.
- **FR-056** : Toute écriture arrivant sur un séjour **clos** MUST partir en **file de
  réconciliation à résolution humaine** (cadrage §11.4) : jamais de rejet silencieux, jamais d'ajout
  d'office.

#### G. Le départ (SEJ-04)

- **FR-057** : Le départ MUST produire le **calcul final** de la note sur la **durée réellement
  occupée**, calculée exclusivement sur l'horodatage d'autorité serveur.
- **FR-058** : Le départ MUST **clore** le séjour, **arrêter** la note et **libérer** l'occupation à
  l'instant réel du départ, temps de remise en état appliqué à partir de cet instant.
- **FR-059** : Le statut d'occupation de l'unité MUST rester **dérivé** des occupations, jamais posé
  à la main (principe IV).
- **FR-060** : Un passage dont la durée réelle dépasse le palier acheté MUST déclencher la
  **rebascule de palier** du cycle 004, avec la différence portée en ligne d'ajustement et tracée au
  registre des actions avec la durée constatée et les deux paliers.
- **FR-061** : Le départ MUST être de classe **B**, refusé immédiatement et explicitement hors
  ligne, et gardé par une permission dédiée.
- **FR-062** : Le départ MUST **figer l'assiette de la taxe de séjour** à cet instant : nombre de
  nuitées assujetties, formule et **règle de conversion appliquée** (`assujettie_taxe_nuitee`,
  `regle_conversion_taxe`), identification du barème applicable (classement de l'établissement,
  commune de rattachement) et **instant du figeage**. **L'assiette porte un séjour, jamais un nombre
  d'occupants** (décision B-10).
- **FR-063** : L'assiette figée MUST NOT être recalculée par aucune relecture ultérieure, quelle que
  soit l'évolution des accompagnants, des barèmes, des formules ou du paramétrage (cadrage §9.6,
  principe V).
- **FR-064** : L'état figé au départ MUST enregistrer le nombre de personnes **réellement** compté
  — titulaire et accompagnants — **à titre de constat, hors de l'assiette**. Il documente
  l'occupation réelle et sert la fiche de police ; il MUST NOT multiplier la taxe (FR-018).
- **FR-065** : Ce cycle MUST NOT écrire de **règle fiscale** : le montant de la taxe est une sortie
  du trait `JurisdictionAdapter` (principe V, porte P-12). Ce cycle fige **l'assiette** et expose le
  **point d'ancrage** ; le montant et la facture sont du ressort de FIS (tranche T3).
- **FR-066** : Une formule non assujettie MUST produire une assiette figée à **zéro nuitée
  assujettie** portant **la trace de la règle appliquée** — l'absence de taxe est un fait établi,
  jamais un silence.
- **FR-067** : Le départ MUST émettre un événement outbox dans sa transaction et une entrée au
  registre des actions.
- **FR-068** : Le système MUST NOT clore un séjour automatiquement à l'expiration de la période
  prévue.

#### H. Prolongation (SEJ-04)

- **FR-069** : La prolongation MUST vérifier la disponibilité **sur l'intervalle étendu**, temps de
  remise en état compris.
- **FR-070** : En cas de conflit, le refus MUST **nommer le conflit** : l'unité et l'instant de
  début de l'occupation suivante. Un message générique est un défaut.
- **FR-071** : En cas de conflit, le système MUST proposer les unités de la même catégorie libres
  sur l'intervalle étendu.
- **FR-072** : Une prolongation acceptée MUST étendre l'occupation et inscrire ses lignes
  d'hébergement supplémentaires au tarif en vigueur.
- **FR-073** : Une prolongation faisant franchir `seuil_bascule_nuitee_minutes` MUST prévenir
  l'utilisateur **avant confirmation**, avec le montant résultant.
- **FR-074** : Une prolongation MUST être refusée sur un séjour clos, et MUST être de classe **B**,
  tracée au registre des actions.

#### I. Départ anticipé (SEJ-04)

- **FR-075** : Un départ prononcé avant la fin prévue MUST arrêter l'hébergement sur la durée réelle
  et porter la différence en **ligne de régularisation distincte**, la ligne initiale restant
  inchangée.
- **FR-076** : Toute régularisation MUST être tracée au registre des actions avec son auteur, son
  instant d'autorité, son montant, le séjour concerné et son motif.
- **FR-077** : L'assiette figée MUST porter les nuitées **réellement** assujetties, pas les nuitées
  prévues.
- **FR-078** : La disponibilité rendue MUST partir de l'instant réel du départ augmenté du temps de
  remise en état, jamais de l'heure initialement prévue.

#### J. Changement d'unité en cours de séjour (SEJ-04)

- **FR-079** : Un changement d'unité MUST clore l'occupation d'origine à l'instant du déplacement et
  ouvrir une occupation sur l'unité cible à partir de ce même instant, **sur le même séjour**.
- **FR-080** : Le changement MUST être refusé, avec le conflit nommé, si l'unité cible n'est pas
  libre sur la période restante ; un déplacement partiel MUST NOT être produit.
- **FR-081** : L'historique du séjour MUST conserver **les deux occupations**, avec leurs unités et
  leurs périodes, et la note MUST porter le tarif propre à chaque période.
- **FR-082** : Le changement MUST être tracé au registre des actions avec les deux unités et
  l'instant, et MUST être de classe **B**.
- **FR-083** : L'assiette de la taxe de séjour MUST être figée **sur l'ensemble du séjour**, jamais
  par occupation.

#### K. Classes hors-ligne, traçabilité, isolation

- **FR-084** : Chaque entité nouvelle MUST déclarer sa classe hors-ligne au registre et MUST porter
  les tests correspondants, instanciés par l'outillage du cycle 005 (`tester_classe_a!`,
  `tester_classe_bcd!`), sous peine d'échec de `outillage_classes.rs`.
- **FR-085** : Toute entité rattachée à un séjour MUST porter le **test du scénario orphelin**
  (§0.7 des user stories).
- **FR-086** : Toute écriture MUST porter un **UUID v7 généré côté client**, et le serveur MUST
  dédupliquer — le rejeu est inoffensif.
- **FR-087** : Toute table nouvelle MUST porter la **RLS activée et forcée** avec au moins une
  politique, et l'isolation entre tenants MUST être vérifiée sur chaque point d'entrée (portes P-07,
  P-08).
- **FR-088** : Aucune requête MUST joindre deux schémas de modules : la fiche client étant partagée
  hors de la verticale hébergement, sa lecture depuis un séjour passe par un **trait exposé**
  (principe II, porte P-04).
- **FR-089** : Aucun calcul de durée, de montant, de taxe ni de clôture MUST s'appuyer sur
  `horodatage_client` (porte P-23).
- **FR-090** : Aucune opération de classe B, C ou D de ce cycle MUST être atteignable depuis un
  chemin de code exécutable hors ligne (porte P-13), et chaque refus MUST être annoncé
  **immédiatement**, jamais par un grisé silencieux ni une mise en file.

#### L. Permissions

- **FR-091** : Le système MUST poser des permissions dédiées pour la **lecture** et la **gestion**
  des fiches clients. Elles MUST être **transversales** (sans module d'activité) : un maquis ou un
  bar seul en aura besoin dès SEJ-05, sans module hébergement.
- **FR-092** : Le système MUST poser des permissions dédiées pour **ouvrir un séjour**, **clore un
  séjour**, **prolonger** et **changer d'unité**, rattachées au module **HEBERGEMENT**.
- **FR-093** : Toute permission posée MUST garder une **action réelle** de ce cycle : une permission
  sans contrepartie est refusée par la règle établie au cycle 003.
- **FR-094** : Les rôles existants MUST recevoir ces permissions selon leur métier, le
  réceptionniste couvrant l'intégralité du parcours d'arrivée et de départ.

#### M. Écrans, vocabulaire et parcours

- **FR-095** : Le cycle MUST livrer quatre écrans : **`R4` Le passage** et **`R7` La note et le
  départ** (tous deux **maquettés**, référence normative à respecter), **`R3` Arrivée** (hérite de
  `R4`, `derivation.md`) et **`R5` Fiche client et recherche** (hérite de `R7`).
- **FR-096** : `R7` MUST être livré avec **la seule section hébergement** de la note ; les sections
  restaurant, bar et autres frais apparaîtront avec les points de vente (T2). L'absence MUST être
  visible comme une absence, pas comme un vide inexpliqué.
- **FR-097** : Chaque page MUST avoir **une seule racine, et ce doit être un élément** — jamais un
  `v-if`/`v-else` de premier niveau (leçon du cycle 003, `CLAUDE.md`).
- **FR-098** : Aucun mot technique MUST atteindre l'interface ni une **route**. Les mots visibles
  sont « arrivée », « départ », « le passage », « la note » — jamais « check-in », « check-out »,
  « occupation », « intervalle » ni « séjour clos ».
- **FR-099** : Le **lexique** (`docs/design/lexique.md`) MUST être complété **avant** le code pour
  chacun des termes nouveaux de ce cycle — client, accompagnant, séjour, fiche de police, arrivée,
  départ, prolongation, changement de chambre, note arrêtée, assiette figée — avec leurs formulations
  **fr et en**, comme au cycle HEB (« le mot est inscrit avant d'être codé »).
- **FR-100** : Toute chaîne visible MUST être externalisée en clés i18n **fr et en**, et chaque
  écran MUST être vérifié en **mode clair et en mode sombre** (porte P-16, DoD 7 et 8).
- **FR-101** : Chaque route nouvelle MUST s'atteindre **en direct et par navigation**, sans erreur
  de console, sur les deux moteurs exercés par la porte **P-22**.
- **FR-102** : Tout montant affiché MUST passer par le formateur unique du projet, avec l'espace
  fine insécable ; les heures gardent l'espace ordinaire (`17 h 30`).
- **FR-103** : Tout champ de formulaire MUST passer par le composant de saisie canonique, et aucune
  couleur ni aucun espacement littéral MUST apparaître hors des jetons (porte P-17).

#### N. Contrat, données de démonstration et mesure

- **FR-104** : Les points d'entrée nouveaux MUST être annotés au contrat OpenAPI, le client
  TypeScript MUST être régénéré **sans diff manuel**, et les identifiants d'opération MUST rester
  uniques (portes P-01, P-01b).
- **FR-105** : Les seeds du tenant de démonstration MUST recevoir des fiches clients et au moins un
  séjour de chaque forme — nuitée en cours, passage en cours, séjour clos — rechargeables **en une
  seule commande**, autant de fois que voulu, avec le même résultat.
- **FR-106** : Le protocole de mesure des cibles de temps MUST être écrit et versionné dans le
  dépôt : matériel de référence, jeu de données, point de départ et point d'arrivée du chronomètre,
  et valeur relevée.
- **FR-107** : L'intégration continue MUST garder les cibles de temps par des critères
  **déterministes** — budget de gestes obligatoires, budget d'appels réseau bloquants, budget de
  temps machine du parcours scripté — et MUST NOT reposer sur une mesure de temps humain, qui
  rougirait au hasard et serait désactivée dans le mois (leçon SC-004 du cycle 004).

### Key Entities

- **`client`** — l'identité d'une personne accueillie, **rattachée au tenant** et partagée entre ses
  établissements. Nom, prénoms, date de naissance, nationalité, pièce d'identité, téléphone,
  courriel, préférences. Classe **C** en création et modification ; ses préférences, notes internes
  et photo sont de classe **A**. Distincte de `personne` (schéma des comptes), qui porte l'identité
  civile du **personnel** : le registre les déclare séparément.
- **`accompagnant`** — une personne présente sur un séjour sans en être le titulaire. Un nom suffit.
  Classe **A**. Il est dû à la **fiche de police** et documente l'occupation réelle de l'unité ; il
  **n'entre pas dans l'assiette** de la taxe de séjour (décision B-10 du 2026-08-03).
- **`sejour`** — l'agrégat central du cycle : un client, un établissement, une ou plusieurs
  occupations, une note, un nombre de personnes, un cycle de vie `en_cours → clos`. Check-in et
  check-out sont de classe **B**.
- **`fiche_police`** — document opérationnel numéroté produit à l'ouverture d'un séjour, couvrant le
  titulaire et ses accompagnants. Classe **B**. Contenu à cartographier (Q3).
- **`note_sejour`** et **`ligne_sejour`** — la note du séjour et ses lignes. Ce cycle n'écrit que
  les lignes d'**hébergement** et d'**ajustement** ; les consommations viennent avec SEJ-03. Classe
  **B**. Prix verrouillé à la création de la ligne.
- **`assiette_taxe_sejour_figee`** — l'état figé au départ : nuitées assujetties, formule et règle de
  conversion appliquées, barème identifié, instant du figeage, **et le nombre de personnes à titre de
  constat, hors assiette**. Une assiette porte **un séjour**, jamais un nombre d'occupants. **Jamais
  recalculée.** Le **montant** n'est pas de ce cycle (FIS).
- **`occupation`** *(existante, cycle 004)* — l'intervalle `[début, fin)` protégé par la contrainte
  d'exclusion. Ce cycle la **rattache à un séjour** et en crée plusieurs pour un même séjour lors
  d'un changement d'unité.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001** : **Un passage s'enregistre en deux gestes.** Du premier geste à la confirmation, le
  parcours compte **exactement deux interactions obligatoires** et **zéro champ de saisie libre
  obligatoire**. Ce critère est gardé en intégration continue et est **déterministe** — il ne dépend
  d'aucune horloge de machine.
- **SC-002** : **Un passage s'enregistre en moins de 30 secondes**, chronométré sur le matériel de
  référence de l'établissement, du premier geste à l'écran de confirmation, avec un opérateur formé
  et un client inconnu. La valeur relevée est consignée au dépôt. **Au-delà de 90 secondes, la story
  est en échec** — pas améliorable, en échec : le corpus dit que le parcours sera contourné.
- **SC-003** : **Une arrivée de client connu s'enregistre en moins de 60 secondes**, mesurée de la
  même façon, et **aucune information déjà portée par la fiche n'est retapée** — le nombre de champs
  ressaisis est **zéro**, ce qui est vérifié par test et non par observation.
- **SC-004** : La part machine du parcours de passage — attente réseau et rendu confondus — tient
  dans un budget déclaré, vérifié par un parcours scripté sur les deux moteurs de rendu exercés.
  *Le budget est fixé très au-dessus de la valeur observée : un seuil serré rougirait au hasard et
  serait désactivé dans le mois.*
- **SC-005** : Une recherche de fiche client par nom, par téléphone ou par numéro de pièce revient en
  **moins de 300 ms au 95ᵉ centile sur 10 000 fiches**, mesuré côté serveur sur un jeu de mesure
  reproductible.
- **SC-006** : **Aucune double attribution n'est possible depuis un parcours de séjour.** Deux
  arrivées concurrentes sur la même unité et des intervalles chevauchants aboutissent à **exactement
  une** écriture, et le refus de l'autre **provient de la contrainte d'exclusion de la base** — la
  garantie tient **vérification applicative préalable neutralisée**.
- **SC-007** : **L'assiette de la taxe de séjour est immuable après le départ.** Après clôture,
  aucune modification d'accompagnant, de barème, de formule ou de paramétrage ne change une seule
  valeur de l'assiette figée, et aucune relecture ne la recalcule.
- **SC-008** : **Cent pour cent des écarts d'argent sont retrouvables au registre des actions** —
  rebascule de palier, régularisation de départ anticipé, prolongation, changement d'unité — avec
  auteur, instant d'autorité, montant et motif.
- **SC-009** : Un séjour avec changement d'unité reste **un seul séjour** portant **deux
  occupations** et **une note continue** ; son historique montre les deux chambres.
- **SC-010** : **Aucune opération de classe B ou C de ce cycle n'est atteignable hors ligne**, et
  chaque indisponibilité est annoncée à l'utilisateur en **moins d'une seconde**, sans grisé
  silencieux ni mise en file.
- **SC-011** : Les montants et les durées sont **identiques quelle que soit la dérive de l'horloge
  du terminal**, jusqu'à une heure d'écart, dans les deux sens.
- **SC-012** : Les quatre écrans du cycle s'ouvrent **en mode clair et en mode sombre**, par
  navigation interne **et** par adresse directe, sans erreur de console, sur les deux moteurs
  exercés.
- **SC-013** : La **démo de fin de tranche T1** s'exécute de bout en bout sur les seules données de
  démonstration : *« Yao enregistre un client en chambre B3 pour 2 nuits, puis un passage de 4 h en
  A1 — la disponibilité empêche tout chevauchement, tout est tracé. »*
- **SC-014** : Un établissement **sans module hébergement** fonctionne sans qu'aucun code ne suppose
  l'existence d'un séjour, et la recherche de fiches clients y reste disponible.

## Assumptions

Ces choix ont été faits faute d'indication contraire du corpus ; ils sont consignés pour être
contredits explicitement si besoin.

- **La note s'ouvre avec sa ligne d'hébergement, pas vide.** SEJ-02 exige « l'ouverture de la note »
  et SEJ-04 un « calcul final » ; un calcul final sur une note vide n'aurait pas de sens. La ligne
  du registre `ligne_sejour` (classe **B**, rattachée à SEJ-03) est **honorée** pour son sous-ensemble
  hébergement, selon l'usage établi du registre qui déclare d'avance.
- **Le montant de la taxe de séjour n'est pas calculé à ce cycle.** L'assiette est figée, le montant
  est une sortie de l'adaptateur de juridiction (FIS, T3). Un séjour clos porte donc une assiette
  figée et un montant **non encore déterminé**, ce qui est dit explicitement plutôt que rendu par un
  zéro trompeur.
- **Le départ n'encaisse pas.** L'encaissement est CAI-03 (T2). L'écran de départ dit que la note est
  arrêtée et non réglée.
- **`client` et `personne` restent deux entités distinctes**, comme le registre les déclare. Les
  fusionner coupleraient les fiches clients au schéma des comptes.
- **La fiche client vit hors de la verticale hébergement.** Elle sert déjà SEJ-05 (clients extérieurs,
  sans hébergement) et servira CAI-07. Sa lecture depuis un séjour passe donc par un trait exposé,
  jamais par une jointure inter-schémas (porte P-04).
- **L'axe des nuits de la taxe de séjour reste le paramètre existant.** La décision B-10 ferme l'axe
  des **personnes** ; elle ne touche pas `regle_conversion_taxe`, semé `une_nuitee_par_occupation`
  pour Deloria au cycle 004 — soit « une seule taxe pour tout le séjour ». Si la pratique réelle est
  une taxe **par nuit**, c'est le **seed** qui change, pas ce cycle : la valeur est éditable formule
  par formule et aucune règle n'est en dur.
- **Le rang de passage affiché sur `R4`** (« 7ᵉ passage ») se calcule sur l'historique du client dans
  le tenant, cohérent avec le partage inter-établissements de la fiche.
- **Aucune fusion de fiches doublons n'est livrée.** Le cas est détecté et montré ; sa résolution
  n'est demandée par aucune story du périmètre.
- **La file de réconciliation est alimentée, pas vidée.** `reconciliation_orpheline` existe depuis le
  cycle 005 ; sa résolution humaine est SYN-03 (T3).
- **La proposition automatique d'unité choisit la première unité libre de la catégorie** selon un
  ordre stable et explicable — aucune stratégie d'optimisation de remplissage n'est demandée par
  SEJ-02, et en inventer une rendrait la proposition imprévisible pour l'opérateur.
- **Le protocole de mesure des cibles de temps est humain et consigné**, la garde d'intégration
  continue est déterministe. La leçon SC-004 du cycle 004 est reprise telle quelle : une assertion de
  temps humain en CI est instable et finit désactivée, donc elle ne garde rien.

## Out of Scope

Chaque ligne dit ce qui est **exposé** et ce qui ne l'est **pas**.

- **SEJ-03 — Note de séjour temps réel (P0, tranche T2).** La note existe et porte ses lignes
  d'hébergement et d'ajustement. **Ne sont pas livrés** : les consommations venues des points de
  vente, le transfert de charges entre séjours, les remises, et l'écran `R6` qui dérive de `R7`
  « sans l'action finale ».
- **SEJ-05 — Clients extérieurs (P0, tranche T2).** La fiche client est déjà transverse et ses
  permissions déjà sans module d'activité, ce qui rend SEJ-05 possible sans migration. **N'est pas
  livrée** : la vente à un client sans hébergement, ni son addition autonome.
- **SEJ-06 — Enregistrement accéléré par OCR (P1, tranche T4).** La saisie de la pièce d'identité
  est manuelle et **différée après la remise de la clé**, ce qui est précisément le point d'entrée de
  l'OCR. **Ne sont pas livrés** : la capture caméra, l'extraction sur l'appareil, l'écran `M5`.
- **FIS — Fiscalité (tranche T3).** L'assiette de la taxe de séjour est figée au départ, avec la
  règle appliquée et le barème identifié : c'est le **point d'ancrage**. **Ne sont pas livrés** : le
  calcul du montant, la décomposition HT/TVA/taxe, la ligne distincte sur facture, la certification
  FNE, l'avoir, l'état de reversement communal.
- **CAI — Caisse (tranche T2).** **Ne sont pas livrés** : l'encaissement, le fractionnement entre
  modes, le shift, la clôture journalière.
- **IMP — Impression (tranche T2).** La note et la fiche de police sont produites en tant que
  documents ; **leur impression sur imprimante thermique réelle n'est pas de ce cycle** — le point 10
  de la Definition of Done s'appliquera au cycle IMP.
- **TRX-06 — Conformité ARTCI (P1).** La pièce d'identité est protégée au repos, son accès
  journalisé et son instant de capture enregistré, afin que la rétention paramétrable s'ajoute
  **sans migration**. **Ne sont pas livrés** : l'export et la suppression des données d'une personne,
  le registre des traitements, et **le consentement tracé à l'enregistrement d'un client** — cette
  dernière absence est une **dette nommée**, pas un oubli : le corpus la place en P1, hors de la
  tranche ouverte.
- **RSV — Réservations (tranche T4).** Le conflit de prolongation est signalé face à **toute
  occupation suivante**, qu'elle vienne d'un séjour ou, plus tard, d'une réservation. **Ne sont pas
  livrés** : la création de réservation, le planning `V1`, la conversion réservation → séjour.
- **HEB-06 — Statut d'unité et sous-statut ménage (P1).** Le départ libère l'occupation et le temps
  de remise en état s'applique. **Ne sont pas livrés** : le sous-statut ménage librement modifiable
  et son écran.
- **Le gabarit officiel de la fiche de police.** Le registre minimal est livré (FR-049) ; **n'est pas
  livré** le formulaire à la forme du pilote, qui reste à cartographier (cadrage §9.7). C'est un
  rendu, pas une donnée : il s'ajoutera sans migration.
- **La création d'une fiche client hors ligne.** Décision O-01, option (a) : `client` reste en classe
  C. **N'est pas livré** le client provisoire local, ni la fusion au retour du réseau.
- **Fusion de fiches clients en doublon.** Détectée et montrée, non résolue.
- **`R2` Vue du jour.** Elle hérite de `R1` + composant 14 et servira les arrivées et départs du
  jour. Aucune story du périmètre ne l'appelle : elle **ne se construit pas** (principe X).

## Dependencies

- **Cycle 002 — établissements et modules d'activité** : l'établissement, son fuseau horaire, sa
  devise, sa commune, son classement, et l'activation du module `HEBERGEMENT` dont dépend l'existence
  même des écrans d'arrivée et de départ.
- **Cycle 003 — comptes, rôles et audit** : les comptes et les rôles cumulables, la garde de
  permission côté serveur et côté écran, le **registre des actions** qui reçoit toutes les traces de
  ce cycle, et `indicatif_telephonique_defaut` dont dépend la normalisation des téléphones.
- **Cycle 004 — hébergement** : les catégories, les unités, les formules, les barèmes, le **moteur de
  disponibilité** avec sa contrainte d'exclusion, le **moteur de tarification** avec la rebascule de
  palier, les temps de remise en état, `heure_arrivee_standard`, `heure_depart_standard`,
  `seuil_bascule_nuitee_minutes`, la permission `heb.unite.attribuer`, et l'entité `occupation` que
  ce cycle rattache à un séjour.
- **Cycle 005 — synchronisation** : la file hors-ligne chiffrée et ses quatre déclencheurs, le témoin
  de synchronisation monté dans la coquille, la table `reconciliation_orpheline`, l'outillage de test
  des classes hors-ligne (`tester_classe_a!`, `tester_classe_bcd!`), la porte **P-23** sur la
  provenance de l'instant.
- **Cycle 001 — socle** : le contrat OpenAPI et la génération du client, l'outbox, la RLS forcée, la
  mécanique de seeds rejouable.
- **Décisions bloquantes** : **O-01**, **B-10** et le registre minimal de la fiche de police sont
  **tranchés** (§ Clarifications, 2026-08-03). Reste dû avant `/speckit-plan` : les amendements
  documentaires ci-dessous.

## Suites documentaires dues

**La décision B-10 contredit trois écrits de rang supérieur à la présente spécification.** Tant
qu'ils ne sont pas amendés, le cycle **FIS** (tranche T3) re-dérivera la règle inverse depuis une
source qui prime — c'est exactement la situation que l'ordre de préséance existe pour empêcher. Les
amendements sont dus **avant `/speckit-plan`**, et aucun ne relève de ce cycle d'implémentation :

| Document | Passage | Ce qui doit y être écrit |
|---|---|---|
| `docs/cadrage-v1.md` §9.6 | « Par nuitée **et par client** : sans étoile 500 · 1★ 1 000 · … » | La taxe est due **par nuitée et par séjour**. L'axe des personnes est fermé par arbitrage terrain du 2026-08-03. |
| `docs/cadrage-v1.md` annexe B, **B-10** | Décision ouverte, échéance « avant le cycle SEJ » | **Tranchée** : pas d'axe par personne, donc **pas de motif d'exonération**. La ligne passe de « ouverte » à « close », avec sa date. |
| `docs/user-stories-v1.md` **FIS-03** | « La taxe de nuitée est **par nuitée et par client** (accompagnants inclus) » | Même correction. C'est la ligne la plus dangereuse : FIS-03 est le critère d'acceptation du moteur de taxes. |
| `docs/user-stories-v1.md` **FIS-08** | « nuitées assujetties, **nombre de clients**, montant dû » | L'état de reversement communal compte des **séjours assujettis**, et peut continuer de reporter le nombre de personnes **à titre indicatif**. |
| `docs/user-stories-v1.md`, récapitulatif des paramètres | « La dimension « par client » n'est **PAS** tranchée (B-02) » | Tranchée le 2026-08-03. *(La référence à B-02 y est par ailleurs erronée : la décision est **B-10**.)* |
| `docs/design/lexique.md` v1.5.1, entrées `regle_conversion_taxe` | « **Ces deux formulations ne disent rien des personnes** […] l'axe des personnes n'est pas tranché (B-10) » | La prudence qui les justifiait tombe. Les formulations restent **justes et employables** — elles n'ont jamais parlé des personnes — mais leur note explicative doit dire que l'axe est désormais clos. |
| `docs/registre-classes-offline.md` §14, **O-01** | Décision ouverte, échéance « avant SEJ-02 » | **Tranchée, option (a)** : `client` reste en **C**. La friction en mode nœud de site est acceptée et nommée. |

**Un point de vigilance sur la valeur du seed.** Le récapitulatif des paramètres décrit la règle de
Deloria comme « `une_nuitee_par_occupation` — **500 F pour un séjour de 3 nuits**, pratique
attestée », alors que l'arbitrage du 2026-08-03 raisonne sur « 500 F **par nuit** ». Les deux
énoncés portent sur l'**axe des nuits**, que la décision B-10 ne touche pas — mais ils ne disent pas
la même chose. **À confirmer au même atelier** : si la pratique est une taxe par nuit, le seed passe
à `au_prorata`, ce qui est un changement de **donnée**, pas de code.

**Enfin, l'énoncé d'entrée de ce cycle est contredit sur un point** et le fait est consigné pour
qu'une relecture ne le prenne pas pour une dérive : la demande écrivait « les accompagnants
**IMPACTENT** le calcul de la taxe de nuitée ». L'arbitrage terrain du 2026-08-03 dit l'inverse.
Les accompagnants restent enregistrés, figés au départ et portés à la fiche de police — ils ne
multiplient simplement plus la taxe.
