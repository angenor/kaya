# Feature Specification: Unités louables, formules de location et moteur de disponibilité

**Feature Branch**: `004-unites-formules-disponibilite` (aucune branche git dédiée créée — travail sur la branche courante, comme aux cycles 001, 002 et 003)

**Created**: 2026-08-02

**Status**: Draft

**Input**: User description: "Unités louables, formules de location et moteur de disponibilité — HEB-01, HEB-02, HEB-03, HEB-04, HEB-05, critères tels quels. DÉCISION STRUCTURANTE ET IRRÉVERSIBLE (HEB-02) : une occupation est un intervalle [début, fin) en TIMESTAMP AVEC FUSEAU DE L'ÉTABLISSEMENT, JAMAIS une paire de dates. Implémentation par contrainte d'exclusion PostgreSQL (EXCLUDE USING gist (unite_id WITH =, periode WITH &&) sur tstzrange) : le chevauchement devient IMPOSSIBLE AU NIVEAU DE LA BASE, pas seulement dans le code applicatif. Test obligatoire : deux attributions concurrentes de la même unité sur des intervalles chevauchants — une seule réussit, PAR LA CONTRAINTE et non par un verrou applicatif. Le temps de remise en état est intégré à l'intervalle d'indisponibilité, jamais géré à part. Quatre familles de formules (HEB-03) : NUITEE, PASSAGE, DEMI_JOURNEE, MENSUEL. AUCUNE formule n'est réservée à un type d'établissement. La formule est attachée à la CATÉGORIE D'UNITÉ. Barème de passage (HEB-04) : table de paliers {duree, prix} + prix d'heure supplémentaire ; un dépassement rebascule AUTOMATIQUEMENT sur le palier supérieur, différence ajoutée à la note et TRACÉE au journal d'audit ; au-delà d'un seuil paramétrable, bascule en nuitée. Le calcul de durée s'appuie EXCLUSIVEMENT sur l'horodatage d'autorité serveur. Chaque formule porte assujettie_taxe_nuitee et une règle de conversion (aucune / une_nuitee_par_occupation / au_prorata / seuil_horaire) : le traitement fiscal du passage et de la demi-journée est un PARAMÈTRE, pas une constante — la valeur par défaut viendra du fiscaliste (décision B-02), ne la code pas en dur. Seeds Deloria : 17 unités en 5 catégories + salle de réunion ; tarifs de nuitée réels ; barème de passage 1 h : 1 500, 2 h : 2 800, 3 h : 4 000, 4 h : 5 000, h. suppl. +1 200 ; plages de demi-journée 8h–12h et 13h–16h ; temps de remise en état passage 30 min, nuitée 2 h, demi-journée 1 h. Hors périmètre : HEB-06, HEB-07, HEB-08, HEB-09 (crée la table prestation_incluse, AUCUNE logique). Personas : Yao, Adjoua. Le statut d'occupation est DÉRIVÉ des occupations, jamais posé à la main ; seul le sous-statut ménage est librement modifiable. La salle de réunion est une unité louable d'une catégorie dédiée, PAS une entité nouvelle."

## Contexte et traçabilité

Quatrième cycle du projet, tranche T1 (`docs/user-stories-v1.md` §0.5, ordre « TRX, ETB, CPT,
**HEB**, SEJ, SYN »). Les trois cycles précédents ont livré le socle technique, le référentiel
d'établissements et les comptes. Aucun n'a produit une seule ligne de la verticale
`hebergement` : son crate existe en **coquille assumée** depuis le cycle 001, et son en-tête
écrit que « son contenu vient du cycle HEB ». Ce cycle honore cette dette.

**C'est le cycle le plus structurant du projet, et le seul dont une erreur de modélisation ne se
rattrape pas.** Le principe IV de la constitution le dit sans nuance : modéliser la disponibilité
en dates fermerait la porte au passage horaire et à la demi-journée, qui sont le différenciateur
du produit sur le marché ivoirien. Le choix est irréversible parce qu'il traverse ensuite le
séjour, la note, la taxe de nuitée, la clôture et le planning de réservation — cinq modules qui
liront tous le même intervalle.

**Deux dettes annoncées nommément par les cycles précédents arrivent à échéance ici :**

1. Le cycle 003 a posé dix-sept permissions **toutes transversales** (`module_code = NULL`), avec
   ce commentaire dans la migration `0016_roles_permissions.sql` : « le principe X interdit d'en
   poser pour un module qui n'a pas encore d'écran […] `module_code` restera donc `NULL`
   **jusqu'au cycle HEB, qui apportera `heb.unite.attribuer`** ». Ce cycle est le premier à poser
   des permissions rattachées à un module d'activité, et le premier à éprouver que le filtrage
   par module fonctionne réellement.
2. La porte **P-09** (« toute occupation est un `tstzrange` protégé par une contrainte d'exclusion
   GiST ») est verte depuis le cycle 001 **parce qu'aucune table d'occupation n'existe**. La
   section « Couverture des portes » de la constitution nomme exactement ce cas : *une porte dont
   la cible est vide est indistinguable d'une porte qui passe*. Ce cycle lui donne sa première
   cible, et doit donc établir son périmètre autant que sa capacité à échouer.

**Sources de vérité consultées** (ordre de préséance de la constitution) :

| Source | Sections utilisées |
|---|---|
| `.specify/memory/constitution.md` v1.7.1 | Principe **IV (temps et disponibilité)** — le cœur de ce cycle ; **I·c** (paramètres métier), **II** (hiérarchie des crates, outbox, un schéma par module), **III** (RLS), **V** (argent, fiscalité hors constantes), **VI** (hors-ligne), **VIII** (i18n, mode sombre), **X** (prêt ≠ construit), **XII** (référence visuelle) ; portes **P-09** (première cible), P-03, P-04, P-05, P-07, P-08, P-10, P-12, P-13, P-16, P-17, P-19, P-22 ; § Couverture des portes ; Definition of Done |
| `docs/cadrage-v1.md` | **§5 entier** (5.1 intervalles horodatés, 5.2 quatre familles, 5.3 barème dégressif, 5.4 remise en état, 5.5 fiscalité infra-journalière, 5.6 vigilance opérationnelle), §4.1 règle 1 (le socle ignore la chambre), §11.3 à §11.5 (classes hors-ligne), §14 (provisions), annexe B (décisions B-02, B-07) |
| `docs/user-stories-v1.md` | **Module HEB (HEB-01 à HEB-05)** — critères repris tels quels ; HEB-06 à HEB-09 pour la frontière ; §0.3 (personas), §0.4 (DoD), §0.5 (tranches et démo de fin de T1), §0.7 (tests hors-ligne obligatoires), TRX-05a/05b (mécanique de seeds), **récapitulatif des paramètres d'établissement** (7 lignes HEB) |
| `docs/registre-classes-offline.md` v1.0.1 | **§7.1 référentiel** (6 lignes en classe C), **§7.2 occupation et disponibilité** (occupation et remise en état en classe **B**, statut d'occupation **dérivé**, `statut_menage` en **A**), §11 (tests obligatoires par classe), §12 (cas pièges — l'horloge et le passage) |
| `docs/design/derivation.md` v1.2.0 | **`G2` est maquetté** — « L'offre d'hébergement », deux états (hôtel, résidence) ; `R2` Vue du jour hérite de `R1` + composant 14 (hors périmètre, tranche SEJ) |
| `docs/design/lexique.md` v1.3.0 | « Unité louable » → « chambre » / « logement » / « salle » selon le contexte · « Rebascule de palier de passage » → « Durée dépassée : passé au tarif 4 h » · « Temps de remise en état » → « Chambre indisponible 30 min (ménage) » · « Taxe communale de nuitée » → « Taxe de séjour (mairie) » · « Classe hors-ligne » → « nécessite internet » |
| `docs/design/html/G2-offre-hebergement.html`, `G2-offre-hebergement-residence.html` | Référence normative de l'écran, deux états. Le second porte l'affordance « Ajouter le passage ici » — la preuve visuelle qu'aucune formule n'est réservée à un type d'établissement |
| `docs/module-dore.md` | Patron de tranche verticale (sqlx 0.9) ; **« La septième couche »** (patron d'écriture front, obligatoire) ; **« La huitième couche »** (cycle de vie de l'application, obligatoire avant toute page nouvelle) |
| `specs/001-`, `specs/002-`, `specs/003-` | Coquille `verticales/hebergement`, mécanique de seeds rejouable, patron de refus explicite d'une valeur non implémentée, harnais de portes à étapes dues |

**Périmètre du cycle** : HEB-01, HEB-02, HEB-03, HEB-04, HEB-05 — critères d'acceptation repris
**tels quels**, sans exigence ajoutée ni retranchée. Les cinq sont **P0**.

**Hors périmètre** : HEB-06 (statut d'unité, P1), HEB-07 (calendrier tarifaire, P1), HEB-08
(contrats et cautions, PROVISION), HEB-09 (prestations incluses — **table seule**, aucune
logique). Voir § Out of Scope, qui dit pour chacun ce qui est créé et ce qui ne l'est pas.

**Personas** :

- **Yao (réceptionniste)** — l'utilisateur de référence de ce cycle, et celui qui décide de son
  succès. Il gère les passages. Le cadrage §5.6 est explicite : *le passage est aujourd'hui
  massivement encaissé en espèces et non tracé ; le tracer donne au propriétaire une visibilité
  qu'il n'a pas — c'est un argument de vente puissant, et une source de résistance du personnel*.
  **Un module de passage lent sera contourné, pas adopté.** La cible tenue par SEJ-02 est de
  moins de 30 secondes pour un passage ; ce cycle ne livre pas la saisie, mais livre le moteur de
  tarification dont elle dépend, et ne doit pas lui coûter de temps.
- **Adjoua (gérante de site)** — cumule gérante, caissière et réceptionniste. C'est elle qui règle
  l'offre : les catégories, les tarifs, le barème du passage, les plages de demi-journée. C'est
  aussi elle qui décide si le logiciel remplace le cahier. L'écran de l'offre est son écran.
- **M. Koffi (propriétaire)** — ne saisit jamais rien. Il lit. Ce cycle le concerne par un seul
  point, mais il est central : **toute rebascule de palier est tracée au registre des actions**,
  et c'est précisément l'écart que le cahier papier ne lui montrait pas.

**Ce que ce cycle ne fait PAS et qui pourrait être supposé** : il ne livre ni le check-in
(SEJ-02), ni la note de séjour (SEJ-03), ni le calcul effectif de la taxe de nuitée (FIS-03), ni
la décomposition des tarifs en HT + TVA + taxe (FIS-03), ni le planning de réservation (RSV, `V1`),
ni la vue du jour (`R2`). Il livre **le référentiel, le moteur de disponibilité et le moteur de
tarification** que ces cinq stories consommeront.

## Clarifications

### Session 2026-08-02

- **Q : Le référentiel des catégories et des unités a-t-il un écran de gestion à ce cycle ?**
  **R : Oui — au titre du TROISIÈME cas, l'écran composé.** *(Révisé le 2026-08-02.)*

  La première réponse était « non », au motif qu'un écran doit être soit maquetté, soit inscrit à
  `docs/design/derivation.md`. **Cette règle à deux cas était incomplète** :
  `docs/Kaya_Design.md` §2 porte depuis l'origine un tableau à deux colonnes — « on maquette
  si… » / « **on code directement si…** » — dont la seconde énumère quatre conditions :
  motif déjà posé (liste, formulaire ou fiche) · conception entièrement issue de la bibliothèque
  de composants · consulté rarement par un utilisateur formé · aucun doute sur son aspect.

  L'écran de gestion des chambres et catégories (`G5`) **remplit les quatre**, il est en **zone de
  charme** — Adjoua n'est ni debout, ni pressée, sans client en face ni argent en jeu — et sa
  couverture par les seize composants canoniques a été vérifiée motif par motif, sans qu'aucun ne
  manque. Il se code donc, et s'inscrit à `derivation.md` avec les mentions **« composé »** et
  **« à valider à l'atelier terrain »**. Voir FR-041, FR-041a et § Out of Scope.

  **Un écran de comptoir se maquette toujours** : le troisième cas est fermé à la zone de vitesse.

- **Q : L'attribution d'une occupation est-elle exposée en API à ce cycle, alors que le check-in
  n'existe pas encore ?**
  **R : Oui, en API et sans écran.** Le test obligatoire de classe B (« deux exécutions
  simultanées, une seule réussit ») exige un chemin exécutable, et la permission
  `heb.unite.attribuer` — annoncée par la migration `0016` du cycle 003 — doit garder une action
  réelle sous peine de faire échouer la règle du cycle précédent qui refuse une permission sans
  contrepartie. L'attribution est donc servie, testée et permissionnée ; sa consommation par un
  parcours utilisateur vient avec SEJ-02.

- **Q : Quelle valeur de règle de conversion fiscale porter sur le passage et la demi-journée en
  attendant l'avis du fiscaliste (décision B-02) ?**
  **R : ni l'une ni l'autre n'est assujettie — c'est un constat d'exploitation, pas un défaut.**
  La taxe de séjour ne s'applique pas au passage ni à la demi-journée dans la pratique observée ;
  les seeds posent donc `assujettie_taxe_nuitee = false`. Ce qui reste dû, et que le produit doit
  offrir, c'est **un moyen facultatif de l'activer** là où une commune l'imposerait — le paramètre
  est éditable, formule par formule. Voir FR-030 à FR-032.

- **Q : La taxe de séjour se compte-t-elle par nuit ou par séjour ?**
  **R : c'est un paramètre, et les deux valeurs existent déjà.** Le constat d'exploitation est
  qu'une personne réservant trois nuits paie **une seule** taxe de 500 F, pas trois —
  `une_nuitee_par_occupation`. La lecture stricte de « par nuitée et par client » est
  `au_prorata`. Les deux figurent au cadrage §5.5 depuis l'origine ; ce cycle les rend
  **modifiables**, parce que cette exonération n'est pas praticable partout.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Adjoua règle l'offre de son établissement (Priority: P1)

Adjoua ouvre « Vos formules » depuis les réglages de son établissement. Elle y voit les quatre
familles que Deloria propose — la nuitée, le passage, la demi-journée, le mois — chacune avec son
prix d'appel et l'indication de savoir si la taxe de séjour est comprise. Elle corrige le prix
d'une nuitée, ajoute une formule, et constate immédiatement le résultat sans recharger l'écran.

Derrière cet écran, son offre existe enfin comme donnée : cinq catégories de chambres, dix-sept
unités, une salle de réunion, et pour chaque catégorie les formules qu'elle accepte.

**Why this priority** : rien du reste n'existe sans ce référentiel. Une disponibilité sans unité
n'a pas de sujet, un barème sans formule n'a pas de porteur. C'est aussi le seul écran du cycle,
donc le seul livrable visible par un utilisateur.

**Independent Test** : peut être entièrement testé en chargeant les seeds Deloria, en ouvrant
l'écran de l'offre, et en vérifiant que les formules et leurs prix s'y lisent
en mode clair et en mode sombre — sans qu'aucune occupation n'existe. Livre à Adjoua la vue de
son offre, qu'elle n'a aujourd'hui que sur un cahier.

**Acceptance Scenarios** :

1. **Given** l'établissement Deloria chargé par les seeds, **When** Adjoua ouvre l'écran de l'offre
   d'hébergement, **Then** les quatre formules s'affichent avec leur prix d'appel formaté selon la
   devise de l'établissement, et la mention de taxe de séjour propre à chacune.
2. **Given** une résidence meublée qui ne propose que le mois et la nuitée, **When** son gérant
   ouvre le même écran, **Then** seules ses deux formules s'affichent, et l'affordance d'ajouter
   le passage y est présente — **aucune formule n'est réservée à un type d'établissement**.
3. **Given** Adjoua sur l'écran de l'offre, **When** elle modifie le prix d'une formule et
   valide, **Then** la liste se rafraîchit sans rechargement de page et la valeur enregistrée est
   celle affichée.
4. **Given** Yao, réceptionniste, **When** il ouvre l'application, **Then** l'action de modifier
   l'offre est **absente** de son interface — pas grisée, absente.
5. **Given** un terminal hors connexion, **When** l'utilisateur atteint l'écran de l'offre,
   **Then** l'indisponibilité est annoncée **immédiatement** et en langue utilisateur, sans
   tentative d'écriture ni file d'attente.
6. **Given** la salle de réunion de Deloria, **When** on consulte le référentiel
   des unités, **Then** elle apparaît comme une **unité louable d'une catégorie dédiée**, au même
   titre qu'une chambre, et non comme une entité d'un autre genre.

---

### User Story 2 - Deux clients ne peuvent jamais recevoir la même unité (Priority: P1)

Yao attribue la chambre B3 à un client pour deux nuits. Au même instant, depuis un autre
terminal, Adjoua attribue la même chambre à un autre client sur une période qui chevauche la
première d'une heure. **Une seule des deux attributions aboutit.** L'autre est refusée, avec un
motif en langue utilisateur, et le refus vient de la base de données — pas d'un verrou posé par
le code.

La chambre reste ensuite indisponible deux heures après le départ, le temps du ménage. Cette
indisponibilité n'est pas une règle affichée quelque part : elle fait partie de l'intervalle.

**Why this priority** : c'est la décision structurante et irréversible du projet. Une double
attribution est la panne la plus visible et la plus coûteuse d'un logiciel hôtelier — deux
clients devant la même porte. Elle est aussi la première cible réelle de la porte P-09.

**Independent Test** : peut être entièrement testé par un test de concurrence sur l'API
d'attribution, sans aucun écran : deux transactions simultanées sur des intervalles chevauchants,
une seule réussit, et le test échoue si la seconde est repoussée par un verrou applicatif plutôt
que par la contrainte de base.

**Acceptance Scenarios** :

1. **Given** la chambre B3 libre, **When** deux attributions concurrentes portent sur des
   intervalles qui se chevauchent, **Then** exactement une réussit et l'autre est refusée par la
   contrainte d'exclusion de la base, avec un code de refus distinct.
2. **Given** la chambre B3 occupée jusqu'à 12 h et un temps de remise en état de 2 h pour la
   nuitée, **When** on demande son attribution à partir de 13 h, **Then** l'attribution est
   refusée — l'unité est indisponible jusqu'à 14 h.
3. **Given** la même chambre, **When** on demande son attribution à partir de 14 h, **Then**
   l'attribution est acceptée : l'intervalle de remise en état est **contigu**, pas chevauchant.
4. **Given** une occupation de 22 h à 6 h le lendemain, **When** on la crée, **Then** elle est
   acceptée sans traitement particulier : un intervalle qui traverse minuit n'est pas un cas
   spécial.
5. **Given** deux occupations dont la première finit exactement quand la seconde commence,
   **When** aucun temps de remise en état n'est configuré, **Then** les deux coexistent — la
   borne de fin est **exclue** de l'intervalle.
6. **Given** une occupation attribuée, **When** elle est libérée, **Then** l'unité redevient
   disponible après son temps de remise en état, et un événement d'état est émis dans la même
   transaction que la libération.
7. **Given** un terminal hors connexion, **When** l'utilisateur tente une attribution, **Then**
   elle est refusée immédiatement — aucune attribution n'est mise en file « au cas où ».

---

### User Story 3 - Yao chiffre un passage, et son dépassement (Priority: P2)

Un client prend une chambre pour deux heures. Yao l'enregistre. Le client repart quatre heures et
dix minutes plus tard. Le système ne facture pas deux heures : il constate la durée réelle, la
rebascule sur le palier de quatre heures, et ajoute la différence. Yao voit « Durée dépassée :
passé au tarif 4 h ». Le propriétaire, lui, verra l'écart au registre des actions.

Si le client était resté au-delà du seuil paramétré — huit heures à Deloria — le séjour aurait basculé en nuitée, pas en un empilement d'heures supplémentaires.

**Why this priority** : le passage est majoritaire en volume dans une partie du parc, et c'est le
revenu que le cahier papier ne trace pas. C'est l'argument commercial du produit. Il vient après
la disponibilité parce qu'un tarif sans unité attribuable n'a rien à chiffrer.

**Independent Test** : peut être entièrement testé par des cas figés donnant, pour une durée
réelle et un barème, un montant et une décision de rebascule — sans écran, sans occupation
persistée, sans réseau.

**Acceptance Scenarios** :

1. **Given** le barème Deloria (1 h = 1 500, 2 h = 2 800, 3 h = 4 000, 4 h = 5 000, heure
   supplémentaire +1 200), **When** la durée réelle est de 2 h, **Then** le
   montant est 2 800.
2. **Given** le même barème, **When** la durée réelle est de 4 h 10, **Then** le palier retenu est
   celui de 4 h porté d'une heure supplémentaire, soit 6 200 — **toute heure entamée est due**.
3. **Given** une occupation vendue au palier de 2 h, **When** le départ est constaté à 4 h 10,
   **Then** la différence entre le montant vendu et le montant dû est portée au débit du séjour, et
   l'opération est **tracée au registre des actions** avec la durée constatée et les deux paliers.
4. **Given** un seuil de bascule en nuitée à 8 h, **When** la durée réelle atteint ou dépasse 8 h,
   **Then** la formule appliquée devient la nuitée, et non le dernier palier majoré d'heures
   supplémentaires.
5. **Given** une horloge de terminal en avance de 40 minutes, **When** la durée est calculée,
   **Then** elle l'est **exclusivement** depuis l'horodatage d'autorité serveur — le résultat est
   identique à celui d'un terminal à l'heure.
6. **Given** un barème dont les paliers sont saisis dans le désordre, **When** on l'enregistre,
   **Then** il est refusé ou normalisé — un barème ne peut pas comporter deux paliers de même
   durée, ni un palier de durée nulle.

---

### User Story 4 - Adjoua vend une demi-journée en salle de réunion (Priority: P3)

Adjoua loue la salle de réunion pour la matinée : 8 h – 12 h, la plage définie par sa catégorie.
Elle ne peut pas la louer de 9 h à 11 h : la plage n'est pas fractionnable. Une seconde location
l'après-midi est possible — 13 h – 16 h — parce que l'heure de remise en état d'une demi-journée
tient entre les deux.

**Why this priority** : la demi-journée est la formule par défaut de la salle de réunion et des
résidences meublées. Elle est moins volumineuse que le passage, mais c'est elle qui prouve que
les plages fixes et la remise en état se composent correctement.

**Independent Test** : peut être entièrement testé en attribuant deux demi-journées consécutives
à la même unité et en vérifiant qu'un fractionnement de plage est refusé.

**Acceptance Scenarios** :

1. **Given** la catégorie « salle de réunion » avec les plages 8 h – 12 h et 13 h – 16 h,
   **When** on demande une occupation de 9 h à 11 h, **Then** elle est refusée : une plage de
   demi-journée n'est pas fractionnable.
2. **Given** la plage du matin déjà occupée et un temps de remise en état d'une heure, **When**
   on demande la plage de l'après-midi, **Then** elle est acceptée — l'heure de battement tient
   entre 12 h et 13 h.
3. **Given** une catégorie dont le temps de remise en état de demi-journée serait porté à 2 h,
   **When** on demande la plage de l'après-midi après celle du matin, **Then** elle est refusée
   par la même contrainte que toute autre occupation — **la remise en état n'est jamais une règle
   à part**.
4. **Given** les plages exprimées en heures locales de l'établissement, **When** l'établissement
   est dans le fuseau `Africa/Abidjan`, **Then** 8 h désigne 8 h à Abengourou, quelle que soit
   l'horloge du terminal ou du serveur.

---

### Edge Cases

- **Attribution sur un intervalle vide ou inversé** (fin antérieure ou égale au début) : refusée.
  Un intervalle vide échapperait à la contrainte de chevauchement — c'est le seul trou possible
  dans la garantie.
- **Intervalle traversant un changement d'heure légale** : sans objet dans le fuseau
  `Africa/Abidjan` (UTC+0 toute l'année), mais le modèle est en instant absolu et non en heure
  murale, donc le cas est couvert par construction pour un futur établissement dans un fuseau à
  changement d'heure.
- **Modification du temps de remise en état d'une catégorie alors que des occupations existent** :
  les intervalles déjà écrits ne bougent pas. La nouvelle valeur ne vaut que pour les occupations
  suivantes — sans quoi une modification de paramètre pourrait rendre invalide un état déjà
  accepté par la base.
- **Suppression d'une catégorie qui porte encore des unités**, ou d'une formule qui porte encore
  des occupations : refusée avec un motif nommant ce qui l'occupe — le patron de refus explicite
  du cycle 002.
- **Barème de passage sans aucun palier**, ou formule de demi-journée sans aucune plage : refusée
  à l'enregistrement. Une formule inexploitable est une panne différée jusqu'au comptoir.
- **Durée réelle inférieure au premier palier** (un client repart au bout de 20 minutes) : le
  premier palier est dû en entier. Il n'y a pas de tarification en dessous du premier palier.
- **Deux formules de même famille sur la même catégorie** (deux barèmes de passage concurrents) :
  refusée — la famille est unique par catégorie, sans quoi le moteur n'a pas de règle de choix.
- **Occupation demandée sur une unité d'une catégorie qui n'accepte pas la formule demandée** :
  refusée avec un motif distinct du chevauchement.
- **Établissement dont le module hébergement n'est pas actif** : l'ensemble des endpoints de ce
  cycle répond le refus déjà normalisé au cycle 002 pour un service non actif, jamais une réponse
  « introuvable » nue.
- **Rebascule de palier alors que la note est déjà close** : hors périmètre de ce cycle (la note
  arrive avec SEJ-03), mais le moteur de tarification doit rendre sa décision sans supposer qu'une
  note existe — il calcule, il ne facture pas.

## Requirements *(mandatory)*

### Functional Requirements

#### Référentiel — catégories, unités et salle de réunion (HEB-01)

- **FR-001** : Le système DOIT porter une entité `categorie` rattachée à un établissement,
  décrivant un nom, une capacité d'accueil et, **par formule**, un temps de remise en état.
- **FR-002** : Le système DOIT porter une entité `unite` rattachée à une catégorie, décrivant un
  code, un étage et un sous-statut de ménage.
- **FR-003** : Ces entités DOIVENT vivre dans la verticale `hebergement` et **jamais dans le
  socle**. Le socle ne connaît que `article_vendable` et `ressource_reservable` ; `unite` est une
  spécialisation de `ressource_reservable`.
- **FR-004** : La **salle de réunion** DOIT être une unité louable d'une catégorie dédiée. Le
  système NE DOIT créer aucune entité, aucune table et aucun endpoint qui lui soit propre.
- **FR-005** : Le référentiel DOIT être lisible par toute personne autorisée à consulter
  l'établissement, et modifiable seulement par les permissions dédiées (FR-036).
- **FR-006** : La colonne de sous-statut de ménage DOIT exister avec une valeur par défaut, **sans
  aucun endpoint ni écran de modification à ce cycle** — sa gestion relève de HEB-06.
- **FR-007** : Le système NE DOIT PAS porter de colonne de statut d'occupation sur l'unité. Ce
  statut est **dérivé** des occupations ; l'inscrire en table rendrait possible de le poser à la
  main, ce que le cadrage §11.4 désigne comme la cause des doubles attributions.

#### Disponibilité — le cœur irréversible (HEB-02)

- **FR-008** : Une occupation DOIT être un intervalle `[début, fin)` en **timestamp avec fuseau**,
  et JAMAIS une paire de dates. Aucune représentation en dates NE DOIT exister nulle part dans le
  modèle, y compris dans les charges utiles d'API.
- **FR-009** : L'impossibilité de chevauchement DOIT être garantie par une **contrainte
  d'exclusion de la base de données** portant sur l'unité et l'intervalle, et NON par un verrou
  applicatif, une transaction sérialisable ou une vérification préalable en lecture.
- **FR-010** : Le système DOIT refuser une occupation dont l'intervalle est vide ou inversé — la
  contrainte d'exclusion ne protège pas contre un intervalle vide.
- **FR-011** : Le **temps de remise en état** DOIT être intégré à l'intervalle d'indisponibilité
  écrit en base, et NON géré comme une règle appliquée à la lecture. Deux occupations dont la
  remise en état se chevauche DOIVENT être refusées **par la même contrainte** que deux
  occupations qui se chevauchent directement.
- **FR-012** : La borne de fin d'un intervalle DOIT être **exclue** : deux occupations contiguës
  sont valides.
- **FR-013** : Le système DOIT servir une interrogation de disponibilité prenant une catégorie ou
  une unité et un intervalle, et rendant les unités attribuables — cette interrogation étant une
  **lecture**, elle NE DOIT jamais être la garantie d'unicité, seulement un confort d'usage.
- **FR-014** : L'attribution et la libération d'une occupation DOIVENT chacune émettre un
  événement d'état dans **la même transaction** que l'écriture.
- **FR-015** : Un refus pour chevauchement DOIT porter un **code de refus distinct** de tout autre
  refus, afin que l'interface puisse le traduire sans lire un message technique.
- **FR-016** : Le système DOIT refuser une occupation dont la formule n'est pas acceptée par la
  catégorie de l'unité demandée, avec un code de refus distinct du chevauchement.

#### Formules de location (HEB-03)

- **FR-017** : Le système DOIT porter une entité `formule` rattachée à une **catégorie d'unité**,
  et jamais à un type d'établissement.
- **FR-018** : Une formule DOIT appartenir à l'une des quatre familles `NUITEE`, `PASSAGE`,
  `DEMI_JOURNEE`, `MENSUEL`.
- **FR-019** : **Aucune famille de formule NE DOIT être restreinte à un type d'établissement.** Un
  hôtel DOIT pouvoir proposer du mensuel ; une résidence meublée DOIT pouvoir proposer du passage.
- **FR-020** : Une formule DOIT porter ses contraintes : durée minimale et maximale, plages
  horaires autorisées, jours autorisés, heures d'arrivée et de départ standard.
- **FR-021** : Une catégorie NE DOIT PAS porter deux formules de la même famille.
- **FR-022** : Toute valeur de famille autre que les quatre nommées DOIT être **refusée
  explicitement**, jamais ignorée ni traitée par défaut — comme les capacités au cycle 002.
- **FR-023** : Le prix porté par une formule DOIT être un **entier d'unité mineure**, et le nombre
  de décimales comme le symbole DOIVENT venir de la devise de l'établissement.

#### Barème dégressif du passage (HEB-04)

- **FR-024** : Le barème de passage DOIT être une **table de paliers** `{durée, prix}` accompagnée
  d'un prix d'heure supplémentaire au-delà du dernier palier. Il NE DOIT PAS être une formule
  arithmétique ni une constante.
- **FR-025** : Le système DOIT refuser un barème sans palier, un barème comportant deux paliers de
  même durée, ou un palier de durée nulle.
- **FR-026** : Une durée réelle dépassant le palier vendu DOIT **rebasculer automatiquement** sur
  le palier immédiatement supérieur ; la différence DOIT être portée au débit du séjour et
  **tracée au registre des actions** avec la durée constatée et les deux paliers en cause.
- **FR-027** : Au-delà d'un **seuil paramétrable par établissement**, le dépassement DOIT basculer
  en nuitée plutôt que d'empiler des heures supplémentaires.
- **FR-028** : Toute heure entamée au-delà du dernier palier DOIT être due en entier.
- **FR-029** : Le calcul de durée DOIT s'appuyer **exclusivement sur l'horodatage d'autorité
  serveur**. Aucun chemin de code NE DOIT lire l'horloge du terminal pour déterminer une durée
  facturable.

#### Fiscalité paramétrée — une exigence produit, jamais une constante (cadrage §5.5 et §9.6)

- **FR-030** : Chaque formule DOIT porter un indicateur d'assujettissement à la taxe de nuitée et
  une **règle de conversion** parmi `aucune`, `une_nuitee_par_occupation`, `au_prorata`,
  `seuil_horaire`. Les deux DOIVENT être **modifiables par l'exploitant**, formule par formule :
  l'exonération constatée sur le pilote n'est pas praticable partout, et le produit ne peut pas la
  figer.
- **FR-031** : Une formule assujettie SANS règle de conversion DOIT être **impossible à
  enregistrer**. Une formule non assujettie PEUT n'en porter aucune — « pas de taxe » et « pas de
  règle » disent la même chose.
- **FR-031a** : Les seeds NE DOIVENT PAS assujettir le passage ni la demi-journée — c'est la
  pratique constatée sur le pilote. La nuitée DOIT porter `une_nuitee_par_occupation` : un séjour
  de trois nuits acquitte **une seule** taxe.
- **FR-032** : Aucune valeur fiscale NE DOIT être codée en dur dans le moteur de tarification. Le
  traitement fiscal reste une sortie de l'adaptateur de juridiction.
- **FR-032a** : Le paramétrage fiscal NE DOIT être traité nulle part — code, test ou commentaire —
  comme une **constante provisoire en attente d'arbitrage**. Les règles varient par collectivité
  (cadrage §9.6, « hors Abidjan variable selon la collectivité ») : le paramètre est une **exigence
  permanente du produit**. **B-02 fixera une valeur par défaut légale, jamais l'existence du
  paramètre.**

#### Demi-journée (HEB-05)

- **FR-033** : Une formule de demi-journée DOIT porter des **plages fixes définies par catégorie**,
  exprimées en heures locales de l'établissement.
- **FR-034** : Une plage de demi-journée NE DOIT PAS être fractionnable : une demande dont
  l'intervalle ne coïncide pas avec une plage déclarée DOIT être refusée.
- **FR-035** : Deux demi-journées consécutives sur la même unité DOIVENT respecter le temps de
  remise en état, **par la contrainte de base** et non par une règle propre à la demi-journée.

#### Permissions, hors-ligne et traçabilité

- **FR-036** : Le système DOIT poser les permissions du module hébergement, **rattachées au module
  d'activité** — les premières du produit dans ce cas. Chacune DOIT garder une action réellement
  servie ; une permission sans action gardée DOIT faire échouer le build.
- **FR-037** : Toute action de ce cycle DOIT être **indisponible hors ligne** : le référentiel est
  de classe C, l'occupation et la remise en état sont de classe B. L'interface DOIT l'annoncer
  **immédiatement** et en langue utilisateur, sans grisé silencieux et sans file d'attente.
- **FR-038** : Toute nouvelle table DOIT porter la sécurité au niveau ligne, activée **et forcée**,
  avec un test d'isolation entre tenants sur chaque endpoint.
- **FR-039** : Aucune requête de ce cycle NE DOIT joindre le schéma du module hébergement à celui
  d'un autre module. Les lectures inter-modules passent par un trait exposé.
- **FR-040** : Le crate de la verticale `hebergement` NE DOIT être référencé par aucun crate du
  socle.

#### Interface

- **FR-041** : Le système DOIT livrer **l'écran de l'offre d'hébergement tel qu'il est maquetté** —
  la liste des formules avec leur prix d'appel et leur mention de taxe.
- **FR-041a** : Le système DOIT livrer un **écran de gestion des chambres et catégories**, au titre
  du troisième cas de `docs/Kaya_Design.md` §2 (« on code directement si… »), **assemblé
  uniquement à partir des composants canoniques** et inscrit à `docs/design/derivation.md` avec
  les mentions « composé » et « à valider à l'atelier terrain ». Il DOIT servir la création et la
  **correction** du code et de l'étage d'une unité. Il NE DOIT PAS servir le changement de
  catégorie (effet tarifaire et fiscal, non classé au registre), le sous-statut de ménage
  (classe A, HEB-06) ni la mise hors service (classe B, HEB-06).
- **FR-041b** : Si un motif nécessaire à un écran composé **manque à la bibliothèque de
  composants**, l'écran NE DOIT PAS être codé : le composant se maquette d'abord.
- **FR-042** : Toute chaîne visible DOIT être externalisée en clés `fr` et `en`, et employer le
  vocabulaire du lexique : « chambre » / « logement » / « salle » selon le contexte, « Durée
  dépassée : passé au tarif 4 h », « Chambre indisponible 30 min (ménage) », « Taxe de séjour
  (mairie) ». Les mots « unité louable », « occupation », « intervalle », « palier » et le
  vocabulaire des classes hors-ligne NE DOIVENT jamais atteindre l'interface.
- **FR-043** : L'écran DOIT être vérifié en mode clair **et** en mode sombre, et sa route DOIT
  s'ouvrir par navigation interne **et** par chargement direct de l'adresse, sans erreur de
  console.
- **FR-044** : Tout montant affiché DOIT passer par le formateur de montants unique du produit,
  qui prend un entier d'unité mineure et une devise.
- **FR-045** : L'écriture sur cet écran DOIT suivre le patron d'écriture front déjà établi :
  squelette de chargement, refus métier en langue utilisateur, validation au champ, action
  **absente** sans permission, rafraîchissement sans rechargement.

#### Paramètres, seeds et provisions

- **FR-046** : Les paramètres déclarés paramétrables DOIVENT vivre dans la configuration
  d'établissement avec son héritage : temps de remise en état par formule, heures d'arrivée et de
  départ standard, seuil de bascule du passage en nuitée, plages de demi-journée.
- **FR-047** : Les seeds DOIVENT peupler Deloria conformément au jeu de référence :
  dix-sept unités en cinq catégories, la salle de réunion, les tarifs de nuitée réels, le barème
  de passage, les plages de demi-journée et les temps de remise en état. Ils DOIVENT rester
  **rejouables et idempotents**, chargés en une commande.
- **FR-048** : Les seeds DOIVENT peupler le second tenant « Résidence Test » à quatre unités, afin
  d'éprouver qu'aucune formule n'est réservée à un type d'établissement.
- **FR-049** : Le système DOIT créer la table de prestation incluse **sans aucune logique, aucun
  endpoint et aucun écran** — provision de HEB-09.
- **FR-050** : Le système NE DOIT créer ni table ni colonne pour les contrats et cautions (HEB-08),
  ni pour le calendrier tarifaire (HEB-07) : le cadrage les situe en incrément 3 et en P1, et le
  principe X interdit de bâtir ce qu'aucune story n'appelle.

### Key Entities

- **Catégorie d'unité** — un groupe d'unités homogènes d'un établissement : un nom, une capacité
  d'accueil, et pour chaque famille de formule un temps de remise en état. Porte les formules.
  Classe hors-ligne **C**.
- **Unité** — une chambre, un logement ou une salle : un code, un étage, un sous-statut de ménage,
  rattachée à une catégorie. Spécialisation d'une ressource réservable du socle. Son statut
  d'occupation n'est pas un attribut : il se dérive. Classe **C**.
- **Formule** — ce qu'on vend sur une catégorie : une famille parmi quatre, un prix, des
  contraintes de durée et d'horaire, un assujettissement à la taxe de nuitée et sa règle de
  conversion. Classe **C**.
- **Palier de barème** — un couple durée / prix appartenant à une formule de passage, accompagné
  du prix d'heure supplémentaire au-delà du dernier palier. Classe **C**.
- **Plage de demi-journée** — un intervalle horaire fixe, non fractionnable, défini par catégorie.
  Classe **C**.
- **Occupation** — l'attribution d'une unité sur un intervalle `[début, fin)` en instant absolu,
  remise en état comprise. La seule entité de ce cycle qui décrit un fait plutôt qu'un
  paramétrage, et la seule protégée par une contrainte d'exclusion. Classe **B**.
- **Prestation incluse** — provision de HEB-09 : la table existe, rien ne l'écrit ni ne la lit.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001** : **Aucune double attribution n'est possible.** Deux attributions concurrentes de la
  même unité sur des intervalles chevauchants aboutissent à **exactement une** écriture, et le refus
  de l'autre **provient de la base de données** — code d'erreur de violation de contrainte
  d'exclusion, sur la contrainte nommée. *Le critère porte sur la **cause** du refus, pas sur son
  volume : deux transactions concurrentes prouvent que la base rejette ; mille prouvent la même
  chose en occupant la CI.*
- **SC-002** : La garantie tient **sans verrou applicatif** : **vérification préalable neutralisée
  dans le code**, une attribution chevauchante échoue quand même. C'est le critère qui distingue
  une garantie d'une coïncidence, et il est vérifié par un test — le principe IV l'exige mot pour
  mot : « garantie par une contrainte d'exclusion PostgreSQL, **pas par un verrou applicatif** ».
- **SC-003** : *(Critère d'atelier terrain.)* Adjoua consulte et corrige l'offre de son
  établissement en moins de deux minutes, à partir de l'accueil, sans documentation.
- **SC-004** : *(Critère d'atelier terrain, non mesuré en CI.)* Yao voit le montant d'un passage —
  y compris avec dépassement et rebascule — **sans attente perceptible**, sur le matériel de
  l'établissement, afin que la cible des trente secondes de SEJ-02 reste tenable une fois la saisie
  ajoutée par-dessus. Une valeur de référence est mesurée une fois en développement et consignée.
  **Une mesure de latence en intégration continue dépend de la machine : elle rougirait au hasard,
  et serait désactivée dans le mois.**
- **SC-005** : Le calcul du montant d'un passage donne le même résultat quelle que soit la dérive
  de l'horloge du terminal, jusqu'à une heure d'écart.
- **SC-006** : Cent pour cent des rebascules de palier sont retrouvables au registre des actions,
  avec la durée constatée et les deux paliers.
- **SC-007** : Le jeu de données complet de Deloria se recharge en **une seule
  commande**, autant de fois que voulu, avec le même résultat.
- **SC-008** : Un établissement qui ne propose que le mois et la nuitée fonctionne de bout en bout
  sans qu'aucun code ne suppose l'existence du passage.
- **SC-009** : L'écran de l'offre s'ouvre en mode clair et en mode sombre, par navigation et par
  adresse directe, sans erreur de console, sur les deux moteurs de rendu exercés.
- **SC-010** : Aucune action de ce cycle n'est atteignable depuis un chemin exécutable hors ligne,
  et l'indisponibilité est annoncée à l'utilisateur en moins d'une seconde.

## Assumptions

- **Le fuseau de l'établissement est celui déjà porté par son référentiel** (`Africa/Abidjan` pour Deloria,
  ETB-01). Les intervalles sont stockés en instant absolu ; le fuseau sert
  à interpréter les heures murales — plages de demi-journée, heures d'arrivée et de départ
  standard.
- **L'horodatage d'autorité est celui du serveur d'API.** La mécanique complète de détection de
  dérive d'horloge relève de SYN-04 (tranche T3) ; ce cycle exige seulement que le calcul de durée
  ne lise jamais l'horloge du terminal.
- **Les tarifs de nuitée sont stockés en montant unique à ce cycle.** Leur décomposition en hors
  taxe, TVA et taxe de nuitée est explicitement renvoyée à FIS-03 (tranche T3) par HEB-03.
- **Le rattachement des cinq tarifs réels aux cinq catégories suit l'ordre croissant de gamme**
  (standard 12 500, classique 15 500, classique supérieure 17 500, supérieure A 20 500, supérieure
  B 25 500). Le document ne pose pas explicitement l'appariement ; c'est la seule lecture cohérente
  avec la progression des catégories.
- **Les valeurs de prix lues sur la maquette ne font pas foi** — elle affiche une demi-journée à
  « 6 000 F les 5 h » et un mois à 180 000 F, incompatibles avec les plages 8 h – 12 h et
  13 h – 16 h du récapitulatif des paramètres. La maquette est une cible visuelle ; les seeds
  documentés font foi pour les données.
- **Le barème du passage reste à confirmer à l'atelier terrain** (décision B-07). Les valeurs
  posées en seeds sont celles du cadrage ; elles sont des données, pas des constantes de code.
- **L'attribution d'une occupation est servie sans écran** à ce cycle. Le parcours qui la
  déclenchera est le check-in (SEJ-02, cycle suivant).
- **Le module hébergement doit être actif** sur l'établissement pour que les endpoints de ce cycle
  répondent. Le refus d'un service non actif suit le patron normalisé au cycle 002.
- **La démo de fin de tranche T1** — « Yao enregistre un client en chambre B3 pour 2 nuits, puis un
  passage de 4 h en A1 » — n'est pas exécutable à la fin de ce cycle : elle exige le check-in. Ce
  cycle en livre la moitié basse.

## Out of Scope

Ce que ce cycle **ne construit pas**, et ce qu'il en prépare :

| Story | Statut | Ce qui est fait ici | Ce qui ne l'est pas |
|---|---|---|---|
| **HEB-06** — Statut d'unité | P1, cycle ultérieur | La colonne de sous-statut de ménage existe avec un défaut ; le statut d'occupation est **dérivable** des occupations | Aucun endpoint de modification du ménage, aucune mise hors service, aucun forçage de disponibilité, aucun écran |
| **HEB-07** — Calendrier tarifaire | P1, cycle ultérieur | Rien | Aucune table, aucune colonne de date d'effet — le principe X interdit de la poser d'avance |
| **HEB-08** — Contrats et cautions | PROVISION, incrément 3 | Rien | Aucune des quatre tables annoncées |
| **HEB-09** — Prestations incluses | PROVISION au MVP | **La table est créée, vide** | Aucune logique, aucun endpoint, aucun écran, aucun décompte |
| **SEJ-02** — Check-in | Cycle suivant | L'attribution d'occupation est servie et permissionnée | Aucun parcours d'enregistrement, aucune fiche client, aucune note |
| **SEJ-03** — Note de séjour | Tranche T2 | Le moteur rend un montant | Rien n'écrit de ligne de note ; le moteur calcule, il ne facture pas |
| **FIS-03** — Taxe de nuitée | Tranche T3 | Le paramétrage fiscal est porté par la formule | Aucun calcul de taxe, aucune décomposition de tarif |
| **RSV / `V1`** — Planning de réservation | Tranche T4 | Rien | Aucun planning, aucune réservation provisoire |
| **`R2`** — Vue du jour | Tranche SEJ | Rien | Aucune grille d'unités |

## Dependencies

- **Cycle 002 (ETB)** — établissement, fuseau, devise, modules d'activité et leur activation ;
  configuration héritée ; patron de refus d'un service non actif.
- **Cycle 003 (CPT)** — comptes, rôles cumulables, permissions, registre des actions. Ce cycle
  écrit dans ce registre et pose les premières permissions rattachées à un module.
- **Cycle 001 (TRX)** — mécanique de seeds rejouable, outbox, sécurité au niveau ligne, coquille
  du crate `verticales/hebergement`.
- **Décision B-02 (fiscaliste)** — **ne conditionne pas ce cycle.** Le cadrage §9.6 écrivant « hors
  Abidjan variable selon la collectivité », le paramètre est une **exigence produit** quoi qu'il
  arrive ; B-02 en fixera la **valeur par défaut légale**, pas l'existence. Les valeurs du pilote
  sont attestées au terrain depuis le 2026-08-02.
- **Axe « par client » — NON TRANCHÉ**, et à ne pas trancher par défaut. La taxe est due « par
  nuitée **et par client** » (cadrage §9.6, FIS-03), les accompagnants comptent (SEJ-02), et
  `une_nuitee_par_occupation` ne dit rien du nombre de personnes. Le calcul est renvoyé à
  **FIS-03** (T3) — ce cycle ne calcule aucune taxe.
- **Décision B-10 (cadrage, annexe B)** — exonération de taxe par personne. **Échéance : avant le
  cycle SEJ**, pas ici : la colonne de motif irait sur `accompagnant`, table de SEJ, et c'est là
  que la fenêtre d'ajout à coût nul se situe. **Ce cycle ne la reprend pas.**
- **Décision B-07 (atelier terrain)** — non tranchée. Le barème posé en seeds est une donnée
  révisable, jamais une constante.
