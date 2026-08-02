# Feature Specification: Classification hors-ligne, file d'actions et horodatage d'autorité

**Feature Branch**: `005-file-hors-ligne-horodatage` (aucune branche git dédiée créée — travail sur la branche courante, comme aux cycles 001 à 004)

**Created**: 2026-08-02

**Status**: Draft

**Input**: User description: "Classification hors-ligne, file d'actions et horodatage d'autorité — SYN-01, SYN-02, SYN-04, critères tels quels. SYN-01 : chaque entité déclare sa classe A/B/C/D dans docs/registre-classes-offline.md ; le test de CI ÉCHOUE si une opération B, C ou D est atteignable depuis un chemin de code exécutable hors ligne, et si une entité n'a pas de classe déclarée. Le classement de référence du cadrage §11.3 fait foi. SYN-02 : toute écriture porte un UUID v7 CLIENT + horodatage local ; file locale persistante ; envoi opportuniste ; rejeu idempotent ; le serveur fait foi en conflit. La file est conçue pour être vidée AU RETOUR AU PREMIER PLAN par défaut sur toutes les plateformes — iOS n'a pas de Background Sync ; BGTaskScheduler et WorkManager viendront en optimisation (MOB-06), jamais en hypothèse. Indicateur permanent dans l'interface : connecté / dégradé / hors ligne + nombre d'éléments en attente, lisible d'un coup d'œil. SYN-04 : horodatage client indicatif (ordre d'affichage local) + horodatage d'autorité à l'arrivée. TOUTE logique métier, TOUT calcul fiscal, TOUTE clôture et TOUT calcul de durée de passage s'appuient exclusivement sur l'horodatage d'autorité. Détection et signalement d'une dérive supérieure à 5 minutes. Hors périmètre : SYN-03 (réconciliation des écritures orphelines) — implémentée au cycle 12, elle dépend des séjours et des documents fiscaux. Prévois la table de file de réconciliation et son état, pas l'écran. Personas : Aminata, Adjoua. Points d'attention : écris DÈS CE CYCLE les tests génériques du §0.7 des user stories (rejeu, désordre, double soumission) sous forme de macros ou d'utilitaires de test réutilisables par tous les cycles suivants. Chaque module qui crée une entité les instanciera."

## Contexte et traçabilité

Cinquième cycle du projet, **tranche T1 — colonne vertébrale** (`docs/user-stories-v1.md` §0.5,
« TRX, ETB, CPT, **HEB**, SEJ, **SYN** »). Les quatre cycles précédents ont livré le socle
technique, le référentiel d'établissements, les comptes et les rôles, puis les unités louables et
le moteur de disponibilité. Ce cycle livre le **module SYN** — trois de ses quatre stories.

**Ce cycle ne part pas d'une page blanche, et c'est sa principale singularité.** Quatre cycles ont
déjà posé, chacun pour ses propres besoins, des fragments de ce que SYN doit maintenant rendre
complet et opposable :

| Déjà en place | Posé par | Ce que ce cycle en fait |
|---|---|---|
| `docs/registre-classes-offline.md` v1.2.0 — 5 schémas déclarés, ~200 opérations | cycles 001 → 004 | Le rend **exhaustif et auto-vérifié** : plus aucune liste de schémas écrite à la main |
| `backend/tests/classes_offline.rs` — table → registre, décompte | cycles 001, 003, 004 | Remplace la **liste en dur de schémas** par une découverte, la lacune trouvée **deux fois** |
| `app/core/sync/classes.ts` — marque de type classe A infalsifiable | cycle 001 | Devient le point d'entrée réel d'une file réelle |
| `app/core/sync/index.ts` — `FileLocale` en mémoire, refus d'enfilement | cycle 001 | Reçoit sa **persistance** et son **envoi opportuniste** |
| `app/core/sync/vidage.ts` — ordre **rafraîchir-avant-vider** | cycle 003 | Est branché sur le retour au premier plan |
| `app/core/sync/attente.ts` — `ecrituresEnAttente()` rendant `0` | cycle 003 | Rend enfin **le compte réel**, et alimente l'indicateur permanent |
| Colonne `horodatage_client TIMESTAMPTZ NULL` sur 4 tables | cycles 001 → 003 | Reçoit son pendant : l'**horodatage d'autorité**, et la règle qui l'impose |
| Crate `socle/synchronisation` — outbox, worker de publication | cycle 001 | Accueille la file de réconciliation (provision SYN-03) |
| Patron d'insertion idempotente — *créée* / *déjà présente*, jamais de conflit ; événement émis **à la seule création** | cycle 001, 5 sous-modules | **L'étend, ne le rouvre pas** (FR-018b, FR-018c) |
| `brancherFile` inscrite « **due par SYN-01** » à l'inventaire d'amorçage | cycle 003 | Bascule à « branchée » **dans le même changement** (FR-022b) |
| Test de la garde assertant qu'**aucune file n'est branchée** | cycle 003 | Bascule de « pas de file » à « file vide », sinon il passe pour la mauvaise raison |

**Le risque propre à ce cycle est donc l'inverse du risque habituel** : non pas construire trop
peu, mais croire que le travail est fait parce que les fichiers existent. La leçon du cycle 003 —
*« une unité écrite n'est ni testée ni branchée par défaut »*, `initialiserTheme()` exportée
pendant deux cycles et appelée nulle part — s'applique ici mot pour mot. `ecrituresEnAttente()`
rend `0` aujourd'hui parce qu'**aucune file n'est branchée**, pas parce qu'une file serait vide,
et son propre commentaire le dit. Ce cycle est celui où la distinction cesse d'être théorique.

**Sources de vérité consultées** (ordre de préséance de la constitution) :

1. `.specify/memory/constitution.md` — **principe VI** (hors-ligne et résilience réseau) dans son
   intégralité ; principes I (sources de vérité), II (outbox), IV (intervalles horodatés),
   VII (`PlatformAdapter`), VIII (i18n), X (« prêt ≠ construit ») ; portes **P-13** (aucune
   opération B/C/D atteignable hors ligne), **P-14** (rejeu triple, désordre commutatif), P-05,
   P-05b, P-07, P-08, P-10, P-16, P-17, P-21, P-22 ; section « Couverture des portes ».
2. `docs/cadrage-v1.md` **§11 entier** — §11.1 les quatre classes, §11.2 l'arbre de décision,
   §11.3 le classement de référence (**fait foi**), §11.4 les cas pièges, §11.5 les six règles
   d'implémentation.
3. `docs/user-stories-v1.md` — module **SYN** (SYN-01, SYN-02, SYN-04), **§0.7 tests hors-ligne
   obligatoires**, §0.4 Definition of Done, §0.3 personas, §0.5 ordre des tranches,
   récapitulatif des paramètres d'établissement.
4. `docs/registre-classes-offline.md` v1.2.0 — §5.6 `socle/synchronisation`, §10 provisions,
   **§11 tests obligatoires par classe**, §12 décisions ouvertes O-01/O-02/O-03.
5. `docs/versions-gelees.md` — aucune dépendance nouvelle attendue.
6. `docs/design/composants.md` **n° 10 — Témoin de synchronisation** (« le composant le plus
   important du produit ») ; `docs/design/derivation.md` — écran **`S1` Panneau de
   synchronisation**, dérivé du composant 10 ; `docs/design/lexique.md` — entrées
   « Synchronisation », « Idempotence, rejeu, file d'attente », « Classe hors-ligne A/B/C/D »,
   « Refus hors ligne d'une opération de classe C », « Refus de passer la main, file d'envoi non
   vide ».

**Périmètre du cycle** : SYN-01, SYN-02, SYN-04 — critères d'acceptation repris **tels quels**,
sans exigence ajoutée ni retranchée. Les trois sont **P0**.

**Hors périmètre** : SYN-03 (réconciliation des écritures orphelines, P0 mais tranche **T3**) —
sa **table et ses états sont créés ici**, aucune interface, aucune logique de résolution.
MOB-06 (`BGTaskScheduler`, `WorkManager`) — optimisations d'arrière-plan, tranche T4.

**Personas** :

- **Aminata (serveuse bar/restaurant)** — Android d'entrée de gamme, réseau intermittent, saisit
  debout. C'est elle qui vit la file. Elle ne doit jamais avoir à se demander si son travail est
  parti : le témoin le dit d'un coup d'œil, sans qu'elle ait à ouvrir quoi que ce soit.
- **Adjoua (gérante de site)** — clôture la journée. C'est elle qui subit une dérive d'horloge :
  une clôture qui ne tombe pas au franc près parce qu'un terminal était réglé de vingt minutes
  est un incident qu'elle ne peut ni voir ni corriger. C'est aussi elle qui passe la main en fin
  de service, et qui doit être empêchée de le faire sur une file non vide.

**Ce que ce cycle ne fait PAS et qui pourrait être supposé** : il ne livre ni le mode nœud de
site (mode C, incrément 3), ni la synchronisation en arrière-plan, ni l'écran de réconciliation,
ni la purge chiffrée du cache de lecture des référentiels (ETB-06). Il ne tranche aucune des trois
décisions ouvertes O-01, O-02, O-03 — jusqu'à leur arbitrage, la classe inscrite au registre
s'applique, et c'est toujours la plus stricte des options.

## Clarifications

### Session 2026-08-02

- Q : Quelle écriture réelle passe par la file à ce cycle, pour que le mécanisme ne soit pas livré
  sans passager ? → R : `note_etablissement.creee` (seule opération de classe A du produit dont
  l'écriture est atteignable aujourd'hui) **et** `journal_audit` par ricochet. L'écran minimal de
  la note interne est livré avec, faute de quoi le mécanisme serait à nouveau du code exporté et
  appelé nulle part.
- Q : Que signifie exactement l'état « dégradé » du témoin, pour qu'il soit testable ? →
  R : la plateforme déclare le réseau présent **mais** la dernière tentative d'envoi a échoué sans
  réponse du serveur, ou le dernier aller-retour a dépassé un seuil paramétrable (défaut 3 s).
  Trois états seulement, jamais de pourcentage (composant 10).
- Q : Que devient une écriture de classe A que le serveur refuse **définitivement** au rejeu
  (refus métier, pas panne réseau) ? → R : **mise en quarantaine visible**, jamais de rejet
  silencieux ni de réessai infini. Elle quitte la file d'envoi, reste consultable dans le panneau
  `S1` avec son motif en langue utilisateur, et n'empêche plus de passer la main.
- Q : Jusqu'où va la porte de l'invariante hors-ligne côté application — contrainte de type seule,
  ou balayage des écrans en direct ? → R : **les deux versants** (FR-005b, FR-005c). La marque de
  classe refuse à la compilation ; une porte de parcours ouvre chaque écran d'écriture réseau
  coupé et constate l'annonce **avant** la saisie. Le périmètre est **découvert**, jamais
  énuméré — une liste manuelle est ce qui a produit les deux trous de FR-004.
- Q : Le mécanisme de déduplication serveur est-il à concevoir ? → R : **non, il est déjà
  tranché** depuis le cycle 001 et ce cycle l'applique (FR-018b à FR-018d). UUID client en clé
  primaire de la ligne, jamais un registre d'écritures reçues ; rejeu → la ligne telle qu'elle est
  en base, jamais un conflit ; et **aucun nouvel événement au rejeu**. Sa limite est nommée, non
  traitée.
- Q : La correction du périmètre des portes se limite-t-elle à la porte du registre ? → R : **non**
  (FR-004b à FR-004d). Dix fichiers de portes énumèrent leur périmètre à la main, 21 chemins de
  crates sont écrits en dur sur six d'entre eux. Ce cycle pose le **module d'énumération partagé**
  dont les portes suivantes hériteront — généralisation d'un motif que la porte de parcours
  applique déjà aux routes.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Aminata sait, sans y penser, si son travail est en sécurité (Priority: P1)

Aminata travaille dans la salle du bar de Deloria, où le réseau tombe plusieurs fois par service.
Elle saisit une note interne sur son Android pendant une coupure. Rien ne l'interrompt : l'écriture
est acceptée immédiatement, et le témoin de synchronisation, présent en permanence dans la barre
d'en-tête, passe de « Enregistré » à « En attente d'envoi (1) ». Elle continue. Quatre écritures
plus tard, le réseau revient ; elle repasse l'application au premier plan et le témoin redescend
à « Enregistré » sans qu'elle ait rien fait. À aucun moment elle n'a lu le mot « file »,
« synchronisation » ou « idempotence ».

**Why this priority** : c'est la raison d'être du module. Un produit qui perd le travail d'une
serveuse pendant une coupure est abandonné au premier service — le cahier revient le lendemain.
Le témoin est décrit par `docs/design/composants.md` comme « le composant le plus important du
produit », et c'est la seule story dont l'absence rendrait les deux autres sans objet côté
utilisateur.

**Independent Test** : peut être entièrement testé en coupant le réseau dans un navigateur piloté,
en effectuant quatre écritures de classe A, en constatant que le témoin affiche l'état et le
nombre exacts, puis en rétablissant le réseau et en repassant au premier plan — les quatre
écritures arrivent, le témoin revient à l'état connecté, et l'application n'a produit aucune
erreur de console.

**Acceptance Scenarios** :

1. **Given** l'application connectée et la file vide, **When** Aminata enregistre une note
   interne, **Then** l'écriture est acceptée sans délai perceptible et le témoin reste à l'état
   connecté avec zéro élément en attente.
2. **Given** le réseau coupé, **When** Aminata enregistre une note interne, **Then** l'écriture
   est acceptée **localement**, le témoin passe à l'état hors ligne et affiche « 1 en attente »,
   et **aucun message d'erreur n'apparaît** — l'opération est de classe A, elle est légitimement
   différée.
3. **Given** quatre écritures en attente et le réseau coupé, **When** Aminata ferme puis rouvre
   l'application, **Then** les quatre écritures sont toujours là — la file a survécu à
   l'extinction du processus — et le témoin affiche « 4 en attente ».
4. **Given** quatre écritures en attente, **When** le réseau revient et l'application repasse au
   premier plan, **Then** la session est rafraîchie **avant** tout envoi, les quatre écritures
   partent, et le témoin redescend à zéro élément en attente.
5. **Given** une file non vide, **When** Adjoua tente de passer la main, **Then** le geste est
   **refusé immédiatement** avec « Des enregistrements ne sont pas encore partis. Attendez le
   retour du réseau avant de passer la main. » — et le stockage local n'est pas purgé.
6. **Given** le réseau déclaré présent par la plateforme mais le serveur injoignable, **When**
   Aminata regarde le témoin, **Then** il affiche l'état **dégradé** et le nombre en attente,
   distinct à la fois de l'état connecté et de l'état hors ligne.

---

### User Story 2 - Une action impossible hors ligne le dit avant la saisie, jamais après (Priority: P1)

Aminata, hors réseau, ouvre l'écran des services de son établissement pour ajouter un service.
L'action lui annonce **avant qu'elle ne saisisse quoi que ce soit** : « Cette action nécessite
internet. » Elle ne remplit pas un formulaire pour découvrir en fin de course que rien n'a été
enregistré, et son geste n'est pas silencieusement mis en attente d'un réseau qui pourrait ne
revenir qu'après son service.

**Why this priority** : c'est l'invariante du principe VI, et la seule dont la violation ne se
voit pas en développement — où la coupure dure trente secondes. Elle se manifeste à Abengourou,
sur un encaissement rejoué deux fois ou sur une réservation posée deux fois sur la même chambre.
La règle a un versant technique (aucune opération B, C ou D atteignable depuis un chemin
exécutable hors ligne, **le build échoue**) et un versant humain (l'interface le dit tout de
suite) : les deux sont dans cette story parce que l'un sans l'autre ne protège personne.

**Independent Test** : peut être entièrement testé en coupant le réseau et en parcourant, dans un
navigateur réel, **chaque** écran d'écriture livré à ce jour — toute action de classe B, C ou D
annonce son indisponibilité avant la saisie, aucune n'est mise en file — puis en vérifiant que la
porte de compilation refuse un enfilement d'opération non marquée classe A.

**Acceptance Scenarios** :

1. **Given** le réseau coupé, **When** Aminata ouvre l'action d'ajout d'un service (classe C),
   **Then** l'indisponibilité est annoncée **avant la saisie**, dans la langue de l'utilisateur,
   et l'action ne peut pas être déclenchée.
2. **Given** le réseau coupé, **When** une opération de classe B, C ou D est tentée par un appel
   direct contournant l'interface, **Then** elle est refusée localement — jamais mise en file
   « au cas où ».
3. **Given** le dépôt à jour, **When** une entité est ajoutée au schéma sans être déclarée au
   registre des classes hors-ligne, **Then** le build **échoue**, en nommant la table manquante.
4. **Given** le dépôt à jour, **When** un schéma applicatif entier est ajouté sans être ajouté au
   périmètre de la porte, **Then** le build **échoue** — la lacune trouvée deux fois (schéma
   `comptes` au cycle 003, schéma `hebergement` au cycle 004) ne peut plus se reproduire.
5. **Given** le code de l'application, **When** un développeur tente de mettre en file une
   opération dont la classe n'a pas été explicitement déclarée, **Then** cela **ne compile pas**.

---

### User Story 3 - Adjoua clôture au franc près malgré une horloge fausse (Priority: P2)

Le téléphone d'Aminata est réglé de vingt-cinq minutes en avance. Elle saisit des écritures toute
la soirée. Adjoua clôture la journée : le total tombe au franc près, parce qu'aucun calcul ne
s'est appuyé sur l'horloge du terminal. L'écart d'horloge lui est **signalé** — sur le terminal
concerné et dans le registre des actions — sans jamais bloquer Aminata en plein service.

**Why this priority** : le cadrage §11.4 en fait un cas piège explicite, et §11.5 en fait la
condition de tout calcul de durée de passage — le passage étant majoritaire en volume dans une
partie du parc. La colonne `horodatage_client` existe depuis le cycle 001 sur quatre tables,
**sans que son pendant d'autorité ait jamais été posé** : à ce jour, rien n'empêche un cycle
ultérieur de calculer une durée sur elle. La story ferme ce trou avant que SEJ et FIS n'écrivent
la moindre règle.

**Independent Test** : peut être entièrement testé en soumettant des écritures portant un
horodatage client volontairement décalé de plusieurs heures, en constatant que l'état persisté
porte un horodatage d'autorité serveur cohérent, que l'ordre d'affichage local reste celui du
terminal, et qu'une simulation de journée d'exploitation avec réseau coupé puis rétabli produit
une clôture identique à la même journée sans coupure.

**Acceptance Scenarios** :

1. **Given** une écriture portant un horodatage client de trois heures dans le futur, **When**
   elle arrive au serveur, **Then** elle reçoit un horodatage d'autorité serveur, l'horodatage
   client est conservé tel quel, et **les deux sont distincts et lisibles**.
2. **Given** une écriture arrivée avec plus de 5 minutes d'écart entre horodatage client et
   horodatage d'autorité, **When** le serveur la traite, **Then** la dérive est **détectée et
   signalée**, et l'écriture est **acceptée** — jamais rejetée pour ce motif.
3. **Given** une dérive signalée, **When** Aminata regarde son terminal, **Then** elle est avertie
   que l'heure de son appareil est fausse, en langue utilisateur, sans que le mot « dérive » ni
   aucune valeur technique n'apparaisse.
4. **Given** une journée d'exploitation simulée avec une coupure réseau en son milieu, **When**
   Adjoua clôture, **Then** le résultat est identique **au franc près** à la même journée sans
   coupure.
5. **Given** le code du produit, **When** un calcul de durée, de taxe ou de clôture s'appuie sur
   l'horodatage client, **Then** cela est **détecté et refusé**.

---

### User Story 4 - Un cycle suivant instancie les tests de classe sans les réinventer (Priority: P2)

Le cycle qui livrera les commandes de point de vente créera une dizaine d'entités. Pour chacune,
il déclare une classe au registre et écrit **une ligne** pour instancier les tests obligatoires de
cette classe — rejeu triple, désordre commutatif, inatteignabilité hors ligne, double soumission
au retour du réseau. Il ne recopie pas trois cents lignes du cycle précédent en les adaptant.

**Why this priority** : le §0.7 des user stories impose ces tests à **chaque** entité du produit ;
trois instanciations existent déjà (`note_etablissement`, `journal_audit`, hébergement), écrites à
la main, avec les divergences que cela suppose. Sans outillage, la douzaine de cycles restants
paiera le même prix, et le premier cycle pressé sautera l'étape — c'est exactement le mécanisme
qui produit une porte verte à cible vide. C'est P2 et non P1 parce que la valeur n'atteint
l'utilisateur qu'indirectement, mais elle est demandée nommément par le brief de ce cycle.

**Independent Test** : peut être entièrement testé en réécrivant les trois instanciations
existantes avec l'outillage — à comportement inchangé et sans perte de couverture — puis en
vérifiant qu'une entité de classe A déclarée sans ses tests fait échouer le build.

**Acceptance Scenarios** :

1. **Given** une entité de classe A nouvellement déclarée, **When** le cycle qui l'introduit
   instancie l'outillage de test, **Then** les tests de **rejeu triple** et de **désordre sur les
   six ordres** sont exécutés sans écrire leur logique.
2. **Given** une entité de classe B, C ou D nouvellement déclarée, **When** le cycle l'instancie,
   **Then** le test d'**inatteignabilité hors ligne** est exécuté, et pour la classe D le test de
   **double soumission au retour du réseau**.
3. **Given** les trois entités déjà couvertes à la main, **When** elles sont portées sur
   l'outillage, **Then** la couverture est **identique ou supérieure**, et le décompte de tables
   inspectées ne baisse pas.
4. **Given** une entité déclarée au registre avec une classe, **When** aucune instanciation de
   test n'existe pour elle, **Then** le build **échoue** en la nommant.

---

### Edge Cases

- **La file survit-elle à un redémarrage forcé du terminal ?** Oui — la persistance est la
  condition, pas une commodité. Une file en mémoire seule perdrait le service d'Aminata au
  premier redémarrage de son Android d'entrée de gamme.
- **Et à un simple rechargement de l'application ?** Oui, et c'est le cas qui compte le plus : il
  arrive plusieurs fois par service, sans que personne le remarque, là où le redémarrage complet
  est rare et visible. Une file qui ne survit qu'au second est une file qui ne survit pas.
- **Que se passe-t-il si le serveur reçoit un rejeu d'une écriture déjà enregistrée ?** Il rend la
  ligne **telle qu'elle est en base**, sans conflit, sans modification, et **sans nouvel
  événement** — trois envois laissent une ligne et un événement.
- **Que se passe-t-il si le rafraîchissement de session échoue au retour du réseau ?** La file
  **reste intacte** et l'utilisateur apprend qu'il doit se reconnecter. Vider sur un refus
  d'authentification détruirait exactement les écritures qu'on cherche à sauver.
- **Que se passe-t-il si la même écriture part deux fois — envoi opportuniste puis retour au
  premier plan ?** Le serveur déduplique par l'UUID v7 client : un seul enregistrement, et la
  seconde réponse est **identique** à la première, pas une erreur de conflit.
- **Que se passe-t-il si trois écritures arrivent dans le désordre ?** Elles produisent le même
  état final quel que soit l'ordre — c'est la définition de la classe A, et ce qui rend l'envoi
  opportuniste possible sans séquencement global.
- **Que se passe-t-il si le serveur refuse définitivement une écriture au rejeu ?** Elle est mise
  en **quarantaine visible** avec son motif en langue utilisateur : jamais de rejet silencieux,
  jamais de réessai infini qui bloquerait toute la file derrière elle.
- **Que se passe-t-il si l'utilisateur se déconnecte avec une file non vide ?** Le geste est
  refusé immédiatement, et le stockage n'est pas purgé — la garde existe depuis le cycle 003 et
  devient enfin effective.
- **Que se passe-t-il si l'horloge du terminal est en retard plutôt qu'en avance ?** La dérive est
  détectée dans les deux sens : c'est une **valeur absolue**, pas un dépassement.
- **Que se passe-t-il si la file atteint une taille déraisonnable après une coupure très longue ?**
  Le nombre reste affiché tel quel — jamais de pourcentage, jamais de troncature silencieuse — et
  aucune écriture n'est abandonnée pour cause de volume.
- **Que se passe-t-il pour une écriture de classe A dont le contexte a disparu — l'établissement
  actif a changé pendant la coupure ?** L'écriture porte son contexte au moment de la saisie ; le
  changement d'établissement local ne la réattribue jamais.
- **Que se passe-t-il si le réseau est présent mais l'application vient d'être ouverte, file
  vide ?** Le témoin affiche l'état connecté et **zéro** — il ne reste jamais dans un état
  indéterminé au démarrage.
- **Que voit un utilisateur dont l'appareil n'a jamais eu de réseau depuis l'installation ?** Rien
  d'exploitable : la première authentification exige le serveur. Ce n'est pas un état à traiter,
  c'est une précondition, et elle est dite.

## Requirements *(mandatory)*

### Functional Requirements

#### SYN-01 — Classification : le registre et son invariante

- **FR-001** : Chaque entité et chaque opération qui écrit en base MUST porter une classe A, B, C
  ou D déclarée dans `docs/registre-classes-offline.md`, avec le **code de branche** de l'arbre de
  décision (D1, C2, B3, A4) qui la justifie, et non seulement la lettre.
- **FR-002** : Le classement de référence de `docs/cadrage-v1.md` §11.3 **fait foi**. Toute
  divergence entre le registre et le cadrage MUST être corrigée en faveur du cadrage, dans le même
  changement qui la révèle.
- **FR-003** : Le build MUST échouer si une table d'un schéma applicatif n'a **aucune** entité
  correspondante déclarée au registre, en nommant la table.
- **FR-004** : Le périmètre inspecté MUST être **découvert**, jamais énuméré à la main : tout
  schéma applicatif existant est inspecté du seul fait d'exister. Cette exigence est écrite parce
  que l'énumération manuelle a produit **deux trous réels** — le schéma `comptes` invisible au
  cycle 003, le schéma `hebergement` invisible au cycle 004 — et qu'un troisième est certain si la
  liste reste manuelle.
- **FR-004b** : Le produit MUST fournir un **module d'énumération partagé** dont toute porte tire
  son périmètre : la liste des schémas applicatifs **lue du catalogue de la base**, la liste des
  crates **lue du manifeste de l'espace de travail**. Corriger la seule porte du registre
  traiterait le symptôme : **dix fichiers de portes énumèrent aujourd'hui chacun leur propre
  périmètre**, avec **21 occurrences** d'un chemin de crate écrit en dur réparties sur six
  d'entre eux. La porte du registre n'est que la plus visible.
- **FR-004c** : Toute porte écrite après ce cycle MUST tirer son périmètre de ce module, et
  hériter du périmètre juste sans y penser. Le précédent existe déjà et fonctionne : la porte de
  parcours lit ses routes du répertoire des pages, jamais d'une liste. Ce cycle **généralise un
  motif éprouvé**, il n'en invente pas un.
- **FR-004d** : Toute porte à périmètre découvert MUST **rapporter le nombre d'éléments
  effectivement inspectés**. Une porte qui découvre mal est indistinguable d'une porte qui n'a
  rien à trouver, et les deux sont vertes.
- **FR-005** : Le build MUST échouer si une opération de classe B, C ou D est atteignable depuis
  un chemin de code exécutable hors ligne. Cette vérification MUST être structurelle — refusée à
  la compilation ou par une porte automatisée — jamais une convention de revue.
- **FR-005b** : La vérification MUST porter sur **les deux versants**, parce qu'aucun des deux ne
  couvre l'autre. Versant structurel : la charge non marquée classe A est refusée à la
  compilation. Versant vécu : **chaque écran d'écriture livré est ouvert réseau coupé**, dans un
  navigateur réel, et l'annonce d'indisponibilité exigée par FR-007 est constatée **avant la
  saisie**. Un type qui compile ne prouve pas qu'une phrase s'affiche ; une phrase qui s'affiche
  sur un écran ne prouve rien des trente autres.
- **FR-005c** : Le balayage de FR-005b MUST être **exhaustif par construction** — il découvre les
  écrans d'écriture plutôt que d'en tenir la liste. Une porte dont la cible est énumérée à la main
  passe au vert le jour où un écran s'ajoute sans y être inscrit, et rien ne le dit ; c'est le
  mécanisme exact des deux trous de FR-004.
- **FR-006** : La file locale MUST refuser, au niveau du type, toute charge dont la classe n'a pas
  été explicitement déclarée classe A par un point de passage unique et infalsifiable.
- **FR-007** : L'interface MUST annoncer l'indisponibilité d'une action de classe B, C ou D
  **avant la saisie**, en langue utilisateur — jamais un grisé silencieux, jamais un échec après
  coup, jamais une mise en file « au cas où ». La formulation MUST être celle du lexique.
- **FR-008** : Aucune donnée de classe B, C ou D MUST être conservée en **cache d'écriture** sur
  un terminal. La lecture en cache d'un référentiel de classe C reste permise, avec sa **fraîcheur
  affichée**.
- **FR-009** : Le registre MUST être mis à jour dans le même changement que toute entité qu'il
  décrit, journal des modifications compris, et sa version incrémentée.

#### SYN-02 — La file d'actions locale

- **FR-010** : Toute écriture MUST porter un **UUID v7 généré côté client**, y compris les classes
  A et D. C'est cet identifiant qui rend le rejeu inoffensif.
- **FR-011** : Toute écriture MUST porter un **horodatage local** du terminal, conservé tel quel
  jusqu'à la persistance serveur.
- **FR-012** : La file locale MUST être **persistante** — elle survit à la fermeture de
  l'application, à l'extinction du terminal, **et à un simple rechargement de l'application**. Ce
  troisième cas est nommé séparément parce qu'il est le plus fréquent et le moins impressionnant :
  une file en mémoire perd un service entier au premier rafraîchissement de page, soit exactement
  ce que la tolérance à la coupure était censée empêcher.
- **FR-013** : La file MUST être **chiffrée au repos dès le premier octet**, via le stockage
  sécurisé de l'abstraction de plateforme — jamais par un accès direct à une API native depuis un
  composant.
- **FR-013b** : Le chiffrement MUST NOT être différé au motif que le contenu du jour ne le
  justifie pas. Il ne se justifie pas par la file d'aujourd'hui mais par celle du cycle suivant :
  **l'extraction OCR d'une pièce d'identité est de classe A**, donc éligible à la file, et elle
  produit nom, prénoms, date de naissance, numéro de pièce et nationalité. Le format de stockage
  local se choisit maintenant et se change douloureusement après déploiement.
- **FR-014** : L'envoi MUST être **opportuniste** : dès que le réseau est disponible, sans
  intervention de l'utilisateur.
- **FR-015** : La file MUST être vidée **au retour au premier plan**, sur toutes les plateformes.
  Aucune fonctionnalité MUST dépendre d'une exécution en arrière-plan — iOS n'en offre pas.
- **FR-016** : Le vidage MUST rafraîchir la session **avant** tout envoi, jamais l'inverse, et cet
  ordre MUST être porté par un point de sortie unique, non par la discipline des appelants.
- **FR-017** : L'échec du rafraîchissement de session MUST laisser la file **intacte** et informer
  l'utilisateur qu'il doit se reconnecter.
- **FR-018** : Le serveur MUST **dédupliquer** par l'UUID v7 client : trois envois de la même
  écriture produisent **un seul** enregistrement, et les trois réponses sont **identiques**.
- **FR-018b** : La déduplication MUST reposer sur l'**UUID client en clé primaire de la ligne
  créée**, et non sur un registre d'écritures reçues. Le patron est **déjà arrêté depuis le cycle
  001** et appliqué par cinq sous-modules : première écriture → *créée* ; rejeu → *déjà présente*,
  avec pour corps **la ligne telle qu'elle est en base**. Un rejeu MUST NOT être traité comme un
  conflit : ce n'est pas une erreur, c'est le comportement normal d'un terminal qui vide sa file
  après une coupure. Ce cycle **applique ce patron, il ne le rouvre pas.**
- **FR-018c** : Un rejeu MUST NOT produire de nouvel événement au journal d'événements. **Trois
  envois, une ligne, UN événement.** C'est le point qu'un registre d'écritures reçues aurait
  manqué : émettre à chaque tentative ferait du grand livre le journal des tentatives réseau du
  terminal, et non celui des transitions d'état.
- **FR-018d** : La **limite** de ce choix MUST être écrite là où elle se lit, et non traitée : le
  serveur ne se souvient pas d'avoir reçu une écriture, seulement de détenir la ligne. Une ligne
  supprimée serait donc **recréée** par un rejeu. Elle ne mord pas ici, et il faut dire pourquoi —
  la file ne transporte **que** des écritures de classe A, append-only par définition, et la
  suppression est déjà interdite sur les registres immuables. La question **se rouvrira** le jour
  où une opération de classe B empruntera la file en mode nœud de site.
- **FR-019** : Trois écritures de classe A appliquées dans les six ordres possibles MUST produire
  le **même état final**.
- **FR-020** : En cas de conflit, **le serveur fait foi**. « Dernier écrit gagne » MUST être
  réservé aux entités de classe A sans conséquence, et nulle part ailleurs.
- **FR-021** : Une écriture **définitivement refusée** par le serveur MUST être mise en
  quarantaine consultable, avec son motif en langue utilisateur — jamais rejetée silencieusement,
  jamais réessayée indéfiniment, et elle MUST cesser de bloquer les écritures suivantes.
- **FR-022** : Une file non vide MUST empêcher la purge du stockage local, y compris lors du geste
  « passer la main », avec un refus **immédiat** et la formulation du lexique.
- **FR-022b** : Les deux marqueurs posés par le cycle 003 en prévision de ce cycle MUST basculer
  **dans le même changement** que le branchement de la file : l'inventaire d'amorçage porte le
  branchement de la file à l'état **« dû par SYN-01 »**, et un test échoue si une fonction dite
  due a un appelant ; le test de la garde asserte aujourd'hui qu'**aucune file n'est branchée**.
  Sans cette bascule, le second continuerait de passer **pour la mauvaise raison** — il mesurerait
  « pas de file » là où il doit mesurer « file vide », et la garde ne garderait rien.

#### SYN-02 — L'indicateur permanent

- **FR-023** : L'application MUST afficher en permanence un témoin de synchronisation à **trois
  états seulement** : connecté, dégradé, hors ligne.
- **FR-024** : L'état **dégradé** MUST être défini de façon testable : le réseau est déclaré
  présent par la plateforme, mais la dernière tentative d'envoi a échoué sans réponse du serveur,
  ou le dernier aller-retour a dépassé un seuil **paramétrable** (défaut 3 s).
- **FR-025** : Le témoin MUST afficher le **nombre d'écritures en attente**, jamais un
  pourcentage, jamais une barre de progression.
- **FR-026** : Le passage à l'état hors ligne MUST être instantané, sans transition.
- **FR-027** : Le témoin MUST être lisible **d'un coup d'œil** : une forme et une phrase par état,
  sans qu'aucune action de l'utilisateur soit nécessaire pour connaître l'état.
- **FR-028** : Le témoin MUST rendre le compte **réel** de la file, et non une valeur constante.
  Les deux versants MUST être vérifiés : il rend zéro quand aucune file n'est branchée, et le
  compte exact dès qu'une l'est.
- **FR-029** : Aucun terme technique MUST atteindre l'interface — ni « file », ni « rejeu », ni
  « idempotence », ni « classe A/B/C/D », ni « synchronisation ». Le lexique fait foi.
- **FR-030** : Le témoin MUST être vérifié en mode clair **et** en mode sombre, dans ses trois
  états, et ses libellés MUST exister en français et en anglais.

#### SYN-04 — Horodatage d'autorité

- **FR-031** : Toute écriture persistée MUST porter **deux** horodatages : celui du client,
  indicatif, et celui d'**autorité**, attribué à l'arrivée par le serveur.
- **FR-032** : L'horodatage client MUST servir **exclusivement** à l'ordre d'affichage local.
- **FR-033** : Toute logique métier, tout calcul fiscal, toute clôture et tout calcul de durée de
  passage MUST s'appuyer **exclusivement** sur l'horodatage d'autorité.
- **FR-034** : L'usage de l'horodatage client dans un calcul métier MUST être **détecté et
  refusé** par une porte automatisée, non par la revue.
- **FR-035** : Un écart en **valeur absolue** supérieur à un seuil paramétrable (défaut
  **5 minutes**) entre horodatage client et horodatage d'autorité MUST être détecté et signalé —
  au terminal concerné et au registre des actions.
- **FR-036** : Une dérive détectée MUST **rester non bloquante** : l'écriture est acceptée, le
  service continue.
- **FR-037** : Le signalement de dérive MUST être formulé en langue utilisateur, sans exposer le
  mot « dérive » ni de valeur technique.
- **FR-038** : Une journée d'exploitation simulée avec coupure réseau puis rétablissement MUST
  produire une clôture identique **au franc près** à la même journée sans coupure.

#### Tests génériques du §0.7 — l'outillage réutilisable

- **FR-039** : Le produit MUST fournir un outillage de test réutilisable couvrant les quatre
  familles imposées par `docs/user-stories-v1.md` §0.7 : **rejeu triple** et **désordre
  commutatif** (classe A), **inatteignabilité hors ligne** (classes B, C, D), **double soumission
  au retour du réseau** (classe D).
- **FR-040** : L'instanciation de ces tests pour une nouvelle entité MUST tenir en une déclaration
  courte, sans réécrire la logique de test.
- **FR-041** : Les trois entités déjà couvertes à la main MUST être portées sur cet outillage
  **sans perte de couverture** — le décompte de tables inspectées et le nombre d'assertions ne
  baissent pas.
- **FR-042** : Le build MUST échouer si une entité déclarée au registre n'a **aucune**
  instanciation de test correspondant à sa classe.
- **FR-043** : L'outillage MUST exister côté serveur **et** côté application — la marque de classe
  et le refus d'enfilement sont vérifiables côté application, le rejeu et le désordre côté
  serveur.

#### Provision SYN-03 — la file de réconciliation

- **FR-044** : La **table** de file de réconciliation des écritures orphelines MUST être créée,
  avec ses états de cycle de vie et ses trois issues de résolution possibles (avoir et
  refacturation, prise en charge, rattachement au séjour suivant).
- **FR-045** : **Aucune interface, aucune logique de résolution** MUST être livrée à ce cycle —
  principe X, « prêt ≠ construit ».
- **FR-046** : Les deux classes de l'entité MUST être respectées telles que le registre les
  déclare déjà : **A** pour la création de l'élément en file (constat append-only), **B** pour sa
  résolution (effet monétaire, résolution humaine obligatoire).

#### Exigences transverses de la Definition of Done

- **FR-047** : Toute nouvelle table MUST porter la sécurité au niveau ligne activée **et** forcée,
  avec son test d'isolation multi-tenant.
- **FR-048** : Tout changement d'état métier MUST émettre son événement au journal d'événements
  dans la **même transaction**.
- **FR-049** : Tout paramètre qualifié de « paramétrable » ici — seuil de dérive, seuil de
  latence de l'état dégradé — MUST vivre dans la configuration d'établissement avec sa chaîne
  d'héritage, et MUST être ajouté au récapitulatif des paramètres dans le même changement.
- **FR-050** : Toute chaîne visible MUST exister en français et en anglais, aucune en dur.
- **FR-051** : Tout écran touché MUST être atteignable **en direct et par navigation**, dans les
  deux thèmes, sans erreur de console.

### Key Entities

- **Écriture en attente** — une opération de classe A saisie localement et pas encore confirmée
  par le serveur. Porte son UUID v7 client, son type d'opération, son horodatage local, sa charge
  utile, son contexte au moment de la saisie, et son nombre de tentatives. Ne porte **jamais** de
  jeton d'authentification : un jeton mis en file serait périmé au retour, et le ranger
  prolongerait la durée de vie d'un secret sur un terminal qu'on peut perdre.
- **Quarantaine** — une écriture définitivement refusée par le serveur, sortie de la file d'envoi,
  conservée avec son motif. Elle n'empêche plus de passer la main et ne bloque plus les suivantes.
- **État de synchronisation** — connecté, dégradé ou hors ligne, plus le nombre d'écritures en
  attente. Trois états, jamais davantage.
- **Horodatage d'autorité** — l'instant attribué par le serveur à l'arrivée d'une écriture. Seule
  base admise de tout calcul métier, fiscal, de clôture et de durée.
- **Horodatage client** — l'instant lu sur le terminal à la saisie. Indicatif, réservé à l'ordre
  d'affichage local.
- **Dérive d'horloge** — l'écart absolu entre les deux, signalé au-delà du seuil paramétrable.
- **Élément de réconciliation orpheline** *(provision)* — le constat qu'une écriture est arrivée
  sur un agrégat déjà clos. Porte l'écriture concernée, l'agrégat visé, la date du constat, son
  état, et l'issue retenue lorsqu'un humain aura tranché. Table et états seulement.
- **Registre des classes hors-ligne** — le document normatif qui associe une classe et un code de
  branche à chaque entité et chaque opération. Il n'est pas une donnée du produit ; il est la
  cible d'une porte, et c'est ce qui le rend opposable.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001** : Aminata enregistre une écriture pendant une coupure et **reprend son travail sans
  interruption** — aucune boîte de dialogue, aucun message d'erreur, aucune attente perceptible.
- **SC-002** : Après une coupure de quatre-vingt-dix minutes suivie d'un retour au premier plan,
  **100 %** des écritures saisies hors ligne sont enregistrées côté serveur, aucune en double,
  aucune perdue.
- **SC-003** : La même écriture soumise trois fois produit **un seul** enregistrement, **trois
  réponses identiques** et **un seul** événement au grand livre — jamais trois.
- **SC-004** : Trois écritures appliquées dans les **six** ordres possibles produisent **six**
  états finaux identiques.
- **SC-005** *(critère d'atelier terrain, non mécanisable)* : Une personne qui regarde l'écran sait
  en **moins de deux secondes**, sans cliquer, si son travail est parti et combien d'éléments
  attendent. **Aucun test automatisé ne l'établit** — la lisibilité d'un coup d'œil se constate sur
  quelqu'un qui n'a pas écrit l'écran. Il se valide à l'atelier, au même titre que la mention « à
  valider à l'atelier terrain » que porte tout écran composé. Le dire ici évite qu'un vert de CI
  passe pour sa vérification.
- **SC-006** : **Zéro** action indisponible faute de réseau n'échoue après coup : chacune est
  annoncée avant la saisie, sur l'intégralité des écrans d'écriture livrés, constaté **écran par
  écran dans un navigateur réel, réseau coupé** — et le nombre d'écrans effectivement parcourus
  est **rapporté**, faute de quoi une cible vide passerait au vert.
- **SC-007** : La file survit à un **rechargement de l'application** et à une **extinction complète
  du terminal**, sans perte dans les deux cas, et son contenu n'est **jamais lisible en clair** sur
  le stockage.
- **SC-007b** : Après ce cycle, **zéro** porte n'énumère son périmètre à la main : les 21
  occurrences de chemin de crate écrites en dur, réparties sur six fichiers, sont ramenées à
  **zéro**, et chaque porte rapporte le nombre d'éléments qu'elle a inspectés.
- **SC-008** : Une file non vide empêche la purge du stockage dans **100 %** des cas, y compris
  par le geste « passer la main ».
- **SC-009** : Une journée d'exploitation avec coupure produit une clôture identique **au franc
  près** à la même journée sans coupure.
- **SC-010** : Un écart d'horloge supérieur au seuil est signalé dans **100 %** des cas, dans les
  deux sens, sans jamais bloquer une écriture.
- **SC-011** : L'ajout d'une entité au schéma sans déclaration de classe fait échouer le build
  dans **100 %** des cas, y compris dans un schéma applicatif entièrement nouveau.
- **SC-012** : Instancier les quatre familles de tests obligatoires pour une nouvelle entité
  demande **une déclaration courte** et aucune logique de test réécrite.
- **SC-013** : Le portage des trois entités déjà couvertes ne fait **baisser aucun** décompte de
  couverture existant.
- **SC-014** : Le témoin de synchronisation est correct dans ses **trois** états, dans les **deux**
  thèmes, dans les **deux** langues — soit douze combinaisons vérifiées.
- **SC-015** : **Aucun** terme technique du lexique n'atteint l'interface.

## Assumptions

- **Le premier passager de la file est la note interne.** C'est aujourd'hui la seule opération de
  classe A dont l'écriture soit atteignable, et son écran n'existe pas. Il est livré ici, minimal,
  parce qu'un mécanisme sans passager réel est exactement le défaut que le cycle 003 a payé —
  `initialiserTheme()` exportée deux cycles durant et appelée nulle part.
- **Le seuil de dérive et le seuil de latence sont des paramètres**, avec pour défauts les valeurs
  du cadrage (5 minutes) et une valeur d'usage (3 s). Le projet n'inscrit aucune valeur métier en
  dur, et le récapitulatif des paramètres d'établissement fait foi.
- **La détection de dérive est faite côté serveur**, à l'arrivée de l'écriture : c'est le seul
  point où les deux horodatages coexistent, et le seul dont l'horloge est fiable.
- **La déduplication n'est pas une question ouverte de ce cycle.** Le patron — UUID client en clé
  primaire, *créée* / *déjà présente*, jamais de conflit — est arrêté depuis le cycle 001 et
  appliqué par cinq sous-modules. Ce cycle l'étend, il ne le rediscute pas. Sa limite est nommée
  en FR-018d et **n'est pas traitée ici** : elle ne mord que sur une classe B en mode nœud de
  site, qui n'existe pas.
- **La persistance de la file passe par l'abstraction de plateforme** déjà en place, **chiffrée
  dès le premier octet**. Aucun accès natif direct depuis un composant.
- **Une tension apparaîtra au cycle suivant et n'est PAS tranchée ici** : l'extraction OCR d'une
  pièce d'identité est de classe **A**, donc éligible à la file, mais la fiche client qu'elle
  alimente est de classe **C**. Une extraction faite hors ligne ne peut donc pas créer sa fiche.
  C'est la **décision ouverte O-01** du registre, dont l'échéance est le cycle des séjours. Elle
  est mentionnée ici pour que ce cycle-là la trouve, pas pour être résolue.
- **L'envoi est unitaire, pas par lots.** Un lot introduirait une sémantique d'échec partiel qui
  n'est pas demandée par SYN-02 et que le rejeu idempotent rend inutile. L'ingestion par lots
  existe ailleurs (métriques), avec ses propres garanties.
- **Le mode nœud de site (mode C) n'existe pas encore.** Les opérations de classe B restent donc
  indisponibles hors ligne sans exception à ce cycle — le mode C est de l'incrément 3.
- **Les trois décisions ouvertes O-01, O-02 et O-03 restent ouvertes.** Jusqu'à leur arbitrage, la
  classe inscrite au registre s'applique, et c'est toujours la plus stricte des options.
- **Aucune dépendance nouvelle n'est attendue** ; le cas échéant, elle est vérifiée sur le
  registre officiel, épinglée exactement et inscrite au document des versions gelées.
- **L'application est déjà authentifiée avant toute utilisation hors ligne.** Un terminal qui n'a
  jamais joint le serveur n'a rien à offrir — ce n'est pas un état dégradé, c'est une précondition.

## Out of Scope

- **SYN-03 — écran de réconciliation des écritures orphelines** (P0, tranche T3). Seules la table
  et ses états sont créés ici. La résolution dépend des séjours (SEJ) et des documents fiscaux
  (FIS), dont aucun n'existe.
- **MOB-06 — synchronisation en arrière-plan** (`BGTaskScheduler`, `WorkManager`, tranche T4).
  Optimisation ; le produit doit être complet sans elle.
- **Le mode nœud de site (mode C)** — incrément 3. Il change ce qui est possible hors ligne pour
  la classe B ; rien ici ne doit le supposer, et rien ne doit l'empêcher.
- **Le cache de lecture des référentiels et son témoin de fraîcheur** (ETB-06) — le registre en
  fixe la classe (A, lecture, fraîcheur affichée), ce cycle n'en construit pas le mécanisme.
- **La purge chiffrée du cache à la déconnexion au-delà de la file** — la garde de la file est
  livrée ; l'inventaire complet de ce qui est purgé relève des cycles qui créent ces caches.
- **Toute interface de diagnostic technique de la file** — le panneau `S1` livré ici est
  utilisateur, pas administrateur. Le diagnostic à distance est de la console éditeur (T5).

## Dependencies

- **Cycle 001 (TRX)** — journal d'événements, worker de publication, marque de type classe A,
  coquille de file locale, colonne d'horodatage client, et **le patron d'insertion idempotente**
  (*créée* / *déjà présente*, événement émis à la seule création) que ce cycle étend sans le
  rouvrir.
- **Cycle 002 (ETB)** — chaîne d'héritage des paramètres d'établissement, sans laquelle les deux
  seuils de ce cycle seraient des constantes.
- **Cycle 003 (CPT)** — session et rafraîchissement, registre des actions, geste « passer la
  main » et sa garde, stockage sécurisé par l'abstraction de plateforme.
- **Cycle 004 (HEB)** — première verticale, et première instanciation manuelle des tests de classe
  à porter sur l'outillage.
- **`docs/design/composants.md` n° 10** — le témoin de synchronisation, ses trois états et ses
  règles. **`docs/design/derivation.md`** — l'écran `S1`, qui en est le développement.
- **`docs/design/lexique.md`** — toutes les formulations visibles de ce cycle y sont déjà, sauf
  celles que ce cycle devra y ajouter dans le même changement.
