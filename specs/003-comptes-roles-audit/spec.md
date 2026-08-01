# Feature Specification: Comptes, rôles cumulables et journal d'audit

**Feature Branch**: `003-comptes-roles-audit` (aucune branche git dédiée créée — travail sur la branche courante, comme aux cycles 001 et 002)

**Created**: 2026-08-01

**Status**: Draft

**Input**: User description: "Comptes, rôles cumulables et journal d'audit — CPT-00, CPT-01, CPT-02, CPT-03, CPT-04, critères tels quels. CPT-00 EN PREMIER : `personne`, `compte` et `employe` sont TROIS TABLES DISTINCTES ; au MVP seules `personne` et `compte` portent de la logique, `employe` est provisionnée et vide ; aucun code ne suppose que `compte` = employé. Rôles cumulables, permissions = union, granulaires et attachées aux modules d'activité. JWT court + refresh révocable, multi-appareils, déconnexion à distance. Les messages d'erreur ne révèlent jamais si un compte existe. L'accueil est un tableau de bord de tuiles filtrées par permission, jamais un menu figé ; chargement paresseux par module. Le journal d'audit est un MODULE DE PREMIER PLAN, immuable, filtrable, consultable depuis n'importe quel terminal. Hors périmètre : CPT-05 et CPT-06 — prévoir les colonnes, pas la logique. Le verrouillage par adresse MAC n'est JAMAIS implémenté. L'attribution de rôle est de classe C — aucune élévation de privilège hors ligne, jamais."

## Contexte et traçabilité

Troisième cycle du projet, tranche T1 (`docs/user-stories-v1.md` §0.5, ordre « TRX, ETB, CPT »).
Le cycle 001 a livré la colonne vertébrale technique et a créé le crate `socle/comptes` en
**coquille assumée** — il compile, il occupe sa place dans la hiérarchie du principe II, il ne
porte aucune logique, et son en-tête écrit que « son contenu vient du cycle CPT ». Ce cycle
honore cette dette.

Le cycle 002 en a laissé une seconde, nommément : « **L'accueil `R1` reste au cycle CPT** : son
filtrage par permission dépend de rôles qui n'existent pas encore, et le livrer à moitié
imposerait de le rouvrir. » C'est ici que `R1` se code, et il ne peut se coder qu'ici.

C'est aussi le cycle qui décide si le module RH de la phase 2 sera possible sans refonte. La
distinction `personne` / `compte` / `employe` ne se voit sur aucun écran du MVP ; elle ne se
rattrape pas non plus après coup. Écrire « le salaire de l'utilisateur » quelque part rendrait la
paie inaccessible sans rouvrir l'authentification de tous les rôles.

**Sources de vérité consultées** (ordre de préséance de la constitution) :

| Source | Sections utilisées |
|---|---|
| `.specify/memory/constitution.md` v1.3.0 | Principes **VI** (hors-ligne), **VII** (application unique et rôles cumulés), **VIII** (i18n, mode sombre), **IX** (sécurité, journal d'audit), **X** (prêt ≠ construit), **XII** (référence visuelle) ; portes P-01 à P-21b ; § Couverture des portes ; Definition of Done |
| `docs/cadrage-v1.md` | **§8.3 (journal d'audit)**, §11.1 à §11.5 (classes hors-ligne, règles d'implémentation), **§12 (sécurité, terminaux du personnel)**, §13.2 (frontière), §14 (provisions) |
| `docs/user-stories-v1.md` | **Module CPT (CPT-00 à CPT-04)**, §0.3 (personas), §0.4 (DoD), §0.5 (tranches), §0.7 (tests hors-ligne obligatoires), DIR-04 (frontière de consultation) |
| `docs/registre-classes-offline.md` v1.0.1 | **§5.2 `socle/comptes`** (neuf lignes classées), §9 (« ce qui n'est pas classé » — sessions, JWT, refresh), §11 (tests obligatoires par classe), §12 (décision ouverte O-01) |
| `docs/design/derivation.md` v1.1.0 | `R1` maquetté (4 états) · `G3` Utilisateurs et rôles → hérite de `G2` · `G4` Journal d'audit → hérite de `R5` + `F2` · **aucune ligne pour un écran de connexion** (voir Clarifications, Q1) |
| `docs/design/lexique.md` v1.0.0 | « Ce que chacun peut faire » (RBAC, permissions) · « Téléphones autorisés » (enrôlement, attestation) |
| `docs/design/html/R1-accueil*.html` | Référence normative de l'accueil, quatre états : générique, maquis, propriétaire, serveuse |
| `docs/module-dore.md` | Patron de tranche verticale (sqlx 0.9) et **« La septième couche »** — patron d'écriture front, obligatoire pour toute opération d'écriture |
| `specs/001-socle-technique-monorepo/`, `specs/002-etablissements-modules-activite/` | Coquille `socle/comptes`, mécanique de seeds, harnais progressif à étapes dues, patron de refus explicite d'une valeur non implémentée |

**Périmètre du cycle** : CPT-00, CPT-01, CPT-02, CPT-03, CPT-04 — critères d'acceptation repris
**tels quels**, sans exigence ajoutée ni retranchée. Les cinq sont **P0**.

**Hors périmètre** : CPT-05 (enrôlement d'appareil, P1, tranche T4) et CPT-06 (attestation et
géorepérage, P1, tranche T4) — **colonnes et table provisionnées, aucune logique, aucune UI**,
comme ETB-07 et ETB-08 au cycle précédent. Voir § Out of Scope.

**Personas** :

- **Adjoua (gérante de site)** — l'utilisatrice de référence de ce cycle. Elle **cumule gérante,
  caissière et réceptionniste**. Ce n'est pas un cas limite à couvrir : c'est le cas nominal
  contre lequel le modèle de permissions se conçoit. Un modèle qui traite le cumul comme une
  exception la contraindrait à trois connexions.
- **Yao (réceptionniste)** — un seul rôle. Son accueil ne montre pas ce qu'il ne peut pas faire.
  Rapidité avant tout : la connexion ne doit pas devenir le nouveau goulet du comptoir.
- **M. Koffi (propriétaire)** — **ne saisit jamais rien**. Il lit le journal d'audit depuis son
  téléphone, à distance, pour détecter les écarts sans se déplacer. C'est lui qui achète le
  produit, et le journal d'audit est ce qu'il achète.
- **Admin éditeur** — console web depuis Abidjan. Il provisionne le tenant et son **premier
  compte propriétaire** ; sans lui, personne ne peut se connecter au premier jour d'un
  établissement.

## Clarifications

### Session 2026-08-01

**Résolus par les documents de référence, sans sollicitation** :

- Classe hors-ligne des entités du cycle → `docs/registre-classes-offline.md` §5.2 : `personne`,
  `compte`, `compte_role`, `role`, `permission`, l'élévation de privilège et `appareil_enrole`
  sont en **C** ; `journal_audit` est en **A** (append-only, immuable, sans effet propre) ; le
  relevé de position est en **A**.
- Relation entre le journal d'audit et l'opération tracée → registre §5.2, encadré :
  « **`journal_audit` est A, l'opération qu'il trace garde sa propre classe.** Tracer une remise
  hors ligne est A ; appliquer la remise est B. **Les deux ne voyagent pas ensemble.** » Le
  journal d'audit n'est donc **pas** un consommateur du journal d'événements outbox : dériver
  l'audit de l'outbox rendrait impossible de tracer une ouverture de tiroir hors ligne, qui est
  précisément de classe A.
- Statut des sessions, JWT et jetons de rafraîchissement → registre §9 : « **éphémère Redis
  reconstructible** ». Ce ne sont pas des données durables : leur perte reconnecte les
  utilisateurs, elle ne perd aucune information métier. Aucune classe hors-ligne ne leur est
  attribuée, et aucune sauvegarde ne les couvre.
- Vocabulaire utilisateur → `docs/design/lexique.md` : « RBAC, permissions » se dit « **Ce que
  chacun peut faire** » ; « attestation d'intégrité, enrôlement » se dit « **Téléphones
  autorisés** ». Les mots « rôle », « permission », « JWT », « jeton » n'atteignent jamais
  l'interface sous cette forme.
- Écrans dérivés du cycle → `docs/design/derivation.md` : `G3` « Utilisateurs et rôles » hérite
  de `G2` (configuration) ; `G4` « Journal d'audit » hérite de `R5` + `F2` (liste filtrable,
  registre sobre). `R1` est **maquetté** en quatre états, dont un état propriétaire et un état
  serveuse — c'est-à-dire que le filtrage par permission est déjà dessiné.
- Frontière avec DIR-04 → `docs/user-stories-v1.md` : DIR-04 (T5) ajoute **l'export** et les
  **alertes configurables** (remise au-delà d'un seuil, écart de caisse, rebascule anormale).
  CPT-04 livre la consultation filtrable ; l'export et les alertes ne sont pas de ce cycle.
- Décision ouverte O-01 (`personne` en classe C, check-in d'un client inconnu hors ligne) → le
  registre la date « **avant SEJ-02** », deux cycles plus loin. Ce cycle livre `personne` en
  classe C conformément au registre et **ne préempte pas** O-01.

**Questions posées** — les trois réponses ci-dessous ont été **confirmées par l'utilisateur le
2026-08-01** :

- **Q1 : aucun écran de connexion n'existe — ni maquetté, ni dérivé.** Les 42 écrans du produit
  n'en comptent aucun, et la règle opposable de `derivation.md` (porte **P-19**) dit qu'un écran
  absent des deux **ne se code pas**. Or CPT-01 exige une authentification.
  → **A: ajouter la ligne à la matrice de dérivation.** `R0` « Connexion » **hérite de `G2`** —
  c'est un formulaire à deux champs et une action primaire, motif entièrement couvert par `G2` et
  par le composant n° 16 `ChampSaisie` ; ses états d'erreur et vides relèvent de `S3`. La matrice
  est faite pour cela et a déjà été amendée le 2026-08-01 pour `A1`. Aucun maquettage préalable.
  L'amendement de `docs/design/derivation.md` est **une tâche de ce cycle**, faite **avant** que
  l'écran ne soit codé.
- **Q2 : « mot de passe fort, ou OTP SMS selon la configuration du tenant » — les deux au MVP ?**
  L'OTP SMS suppose un fournisseur SMS : contrat, coût par message, agrégateur. Aucun n'est
  nommé au cadrage §13 ni gelé dans `docs/versions-gelees.md`.
  → **A: mot de passe fort seul au MVP.** Le paramètre de méthode d'authentification existe en
  table et accepte la valeur `MOT_DE_PASSE` ; la valeur `OTP_SMS` est **refusée explicitement**,
  jamais ignorée — exactement le patron du refus des capacités non implémentées (porte **P-06**,
  cycle 002). Le jour où un fournisseur est choisi, la valeur s'active sans migration.
- **Q3 : quel périmètre pour le catalogue de permissions ?** Les exemples de CPT-02
  (`pdv.remise.appliquer`, `heb.unite.attribuer`) désignent des modules qui n'existent pas
  encore.
  → **A: seules les permissions des modules livrés.** Principe X, « prêt ≠ construit » : une
  permission qui ne garde aucune action est une permission non testable. Le référentiel est en
  table, alimenté par seeds, **extensible sans migration** ; chaque cycle ajoute les siennes dans
  la même tâche que ses écrans. Un contrôle de complétude vérifie qu'aucune permission déclarée
  ne garde zéro action, et qu'aucune action sensible ne s'exécute sans permission.

**Tranchés par défaut raisonnable**, consignés en § Assumptions et révisables en
`/speckit-plan` : amorçage du premier compte (1), désactivation plutôt que suppression (2),
identifiant de connexion (3), portée d'un rôle (4), effet du retrait d'un rôle sur une session
active (5), périmètre des types d'action d'audit livrables dès ce cycle (6), rattachement de
`journal_audit` au crate `socle/comptes` (7).

## User Scenarios & Testing *(mandatory)*

> Les priorités `P1`/`P2`/`P3` ci-dessous sont les **priorités d'implémentation du modèle Spec
> Kit** — l'ordre dans lequel les tranches se construisent. Elles ne remplacent pas les priorités
> produit `P0`/`P1`/`P2`/`PROVISION` de `docs/user-stories-v1.md` : **CPT-00 à CPT-04 sont
> toutes P0**, et le volet `employe` de CPT-00 est une PROVISION.

### User Story 1 - Trois tables distinctes, jamais confondues (Priority: P1)

Le modèle sépare **l'identité civile** (`personne` — nom, pièce, contact), **l'identité
d'authentification** (`compte` — porteuse des rôles) et **le contrat de travail** (`employe` —
embauche, salaire, numéro CNPS). Une femme de ménage est un **employé sans compte**. Un
comptable externe est un **compte sans contrat**. Un propriétaire est souvent les deux sans être
salarié. Au MVP, `employe` est une table provisionnée et vide.

**Why this priority**: c'est la seule story dont l'échec ne se voit pas et ne se rattrape pas.
Aucun écran du MVP ne la montre. Elle conditionne la faisabilité du module RH en phase 2 sans
refonte de l'authentification, et elle doit contraindre la conception au lieu de la constater —
donc être écrite et vérifiée **avant** les quatre autres.

**Independent Test**: exécuter en intégration continue, sur une base vierge, les trois figures
imposées — un employé sans compte, un compte sans contrat, une personne portant les deux — et un
contrôle statique qui échoue si un attribut de contrat, de rémunération ou d'emploi apparaît sur
`compte` ou sur `personne`, ou si un chemin de code lit `employe` pour décider d'un droit.

**Acceptance Scenarios**:

1. **Given** une base vierge, **When** on enregistre une femme de ménage comme `personne` puis
   comme `employe` sans lui créer de `compte`, **Then** l'enregistrement réussit et cette
   personne ne peut pas se connecter.
2. **Given** un comptable externe, **When** on lui crée un `compte` rattaché à une `personne`
   sans aucune ligne `employe`, **Then** il se connecte et exerce ses permissions normalement.
3. **Given** le propriétaire M. Koffi, **When** il porte une `personne`, un `compte` et aucune
   ligne `employe`, **Then** rien dans le produit ne le traite comme un salarié.
4. **Given** le code du cycle, **When** le contrôle structurel s'exécute, **Then** il échoue si
   une colonne de contrat, de salaire, de date d'embauche ou de numéro CNPS existe ailleurs que
   sur `employe`.
5. **Given** la table `employe`, **When** le cycle est livré, **Then** elle existe, elle est
   vide, elle porte sa politique d'isolation, et **aucun point d'entrée de l'API ne l'écrit ni ne
   la lit**.

---

### User Story 2 - Adjoua se connecte, et sa session se révoque à distance (Priority: P1)

Adjoua se connecte avec son identifiant et son mot de passe. Elle travaille sur le poste de la
réception et sur sa tablette : **deux sessions simultanées, sans se déconnecter de l'une pour
ouvrir l'autre**. Quand un téléphone est perdu ou qu'un employé part, la session correspondante
est **coupée à distance** et ne se rouvre pas. Une tentative de connexion échouée dit la même
chose que le compte existe ou non.

**Why this priority**: sans elle, aucune des trois autres stories n'est atteignable. C'est aussi
la surface la plus exposée du produit : une différence de message ou de délai entre « compte
inconnu » et « mot de passe faux » livre la liste des comptes d'un établissement à qui la
cherche.

**Independent Test**: dérouler une connexion réussie, une connexion échouée sur compte
inexistant, une connexion échouée sur mot de passe faux — et vérifier que les deux échecs sont
**indiscernables** en message, en code de retour et en ordre de grandeur de temps de réponse.
Puis ouvrir deux sessions, en révoquer une, et vérifier que le rafraîchissement de celle-là est
refusé alors que l'autre continue.

**Acceptance Scenarios**:

1. **Given** un compte actif, **When** Adjoua se connecte avec les bons identifiants, **Then**
   elle reçoit un accès de courte durée et un moyen de rafraîchissement révocable.
2. **Given** un identifiant qui n'existe pas et un identifiant qui existe avec un mauvais mot de
   passe, **When** les deux tentatives échouent, **Then** le message affiché, le code de retour
   et l'ordre de grandeur du temps de réponse sont **identiques**.
3. **Given** Adjoua connectée sur deux appareils, **When** elle continue de travailler sur les
   deux, **Then** les deux sessions restent valides indépendamment.
4. **Given** une session listée dans les sessions actives, **When** un compte habilité la
   révoque, **Then** le rafraîchissement est refusé **immédiatement**, l'appareil concerné est
   ramené à l'écran de connexion, et les autres sessions ne sont pas affectées.
5. **Given** un accès de courte durée expiré, **When** le moyen de rafraîchissement est employé,
   **Then** un nouvel accès est délivré sans ressaisie du mot de passe, et le moyen de
   rafraîchissement consommé ne se réemploie pas.
6. **Given** un compte désactivé, **When** il tente de se connecter ou de rafraîchir, **Then**
   les deux échouent, avec le même message que toute autre tentative échouée.
7. **Given** un terminal hors ligne, **When** l'utilisateur tente de se connecter, **Then**
   l'interface le lui **dit immédiatement et explicitement** — pas de grisé silencieux, pas de
   file d'attente « au cas où ».

---

### User Story 3 - Le cumul de rôles est la norme, et les permissions sont l'union (Priority: P1)

Adjoua porte trois rôles — gérante, caissière, réceptionniste. Ses permissions sont **l'union**
des trois, sans qu'aucun ne prime, sans conflit à arbitrer, sans ordre de priorité. Les
permissions sont **granulaires et attachées aux modules d'activité** : une permission qui
concerne un service non activé dans l'établissement ne donne aucun droit. L'attribution et le
retrait de rôle sont de **classe C** : ils exigent le cloud, toujours.

**Why this priority**: c'est le cœur du module, et le principe VII en fait une norme et non une
exception. Un modèle qui traiterait le cumul comme un cas limite obligerait Adjoua à trois
comptes ou à trois connexions — et le produit serait contourné dès la première semaine.

**Independent Test**: attribuer les trois rôles à un compte de test, vérifier que l'ensemble
effectif de ses permissions est exactement l'union des trois ensembles, retirer un rôle et
vérifier que seules les permissions exclusives à ce rôle disparaissent. Puis vérifier qu'aucun
chemin de code exécutable hors ligne n'atteint l'attribution de rôle.

**Acceptance Scenarios**:

1. **Given** un compte portant les rôles gérant, caissier et réceptionniste, **When** on calcule
   ses permissions effectives, **Then** le résultat est exactement l'union des permissions des
   trois rôles, sans doublon et sans arbitrage.
2. **Given** un compte portant deux rôles qui partagent une permission, **When** on lui retire
   l'un des deux, **Then** il **conserve** la permission partagée et perd uniquement les
   permissions exclusives au rôle retiré.
3. **Given** un établissement où le service restauration n'est pas activé, **When** un compte
   porte une permission rattachée à ce service, **Then** cette permission ne lui ouvre aucune
   action, et l'interface ne mentionne le service **nulle part**.
4. **Given** un terminal hors ligne, **When** un chemin de code tente d'attribuer, de retirer ou
   d'élever un rôle, **Then** l'opération est refusée ; un test dédié **échoue** si une telle
   opération est seulement atteignable depuis un chemin exécutable hors ligne.
5. **Given** une attribution de rôle réussie, **When** elle est enregistrée, **Then** une entrée
   « changement de rôle » est écrite au journal d'audit avec l'auteur, la cible, le rôle et
   l'horodatage d'autorité serveur.
6. **Given** un compte sans aucun rôle, **When** il se connecte, **Then** la connexion réussit et
   son accueil ne propose **aucune** tuile d'action, avec un état vide explicite.

---

### User Story 4 - L'accueil ne montre que ce qu'on a le droit de faire (Priority: P2)

L'écran d'accueil est un **tableau de bord de tuiles filtrées par permission**, jamais un menu
figé. Yao le réceptionniste, Adjoua qui cumule trois rôles, Aminata la serveuse et M. Koffi qui
ne saisit rien voient quatre accueils différents — sur la même application, avec le même code.
Un module inactif ou une action interdite est **absent**, jamais grisé. Le code d'un module ne se
télécharge pas tant qu'il n'est pas ouvert.

**Why this priority**: c'est la dette explicitement reportée par le cycle 002, et la première
chose que tout utilisateur voit. Elle vient après US2 et US3 parce qu'elle n'a rien à filtrer
tant que les permissions n'existent pas.

**Independent Test**: se connecter successivement avec quatre comptes de rôles différents sur le
même établissement et comparer les tuiles affichées aux quatre états maquettés de `R1`. Vérifier
par inspection du chargement qu'un compte serveur ne charge aucun code de back-office.

**Acceptance Scenarios**:

1. **Given** Yao, réceptionniste, **When** il ouvre l'accueil, **Then** il voit les tuiles de ses
   permissions et **aucune** tuile d'une action qu'il ne peut pas faire — ni grisée, ni assortie
   d'un message d'indisponibilité.
2. **Given** Adjoua, qui cumule trois rôles, **When** elle ouvre l'accueil, **Then** elle voit
   l'union des tuiles des trois rôles, chacune **une seule fois**.
3. **Given** M. Koffi, propriétaire, **When** il ouvre l'accueil depuis son téléphone, **Then**
   il voit des tuiles de consultation et le journal d'audit, conformément à l'état
   `R1-accueil-proprietaire`.
4. **Given** un établissement dont le service restauration n'est pas activé, **When** n'importe
   quel compte ouvre l'accueil, **Then** aucune tuile ni aucun libellé ne mentionne ce service.
5. **Given** un compte de serveur de salle, **When** il utilise l'application, **Then** le code
   du back-office n'est **pas** chargé tant qu'il n'ouvre aucun écran de back-office.
6. **Given** n'importe quel accueil, **When** on le vérifie, **Then** il est conforme en mode
   clair **et** en mode sombre, et aucune chaîne visible n'est en dur (clés fr et en présentes).

---

### User Story 5 - M. Koffi lit le journal d'audit depuis son téléphone (Priority: P2)

Le journal d'audit trace, de façon **immuable**, dix familles d'actions sensibles : remise,
annulation de ligne envoyée, avoir, ouverture de tiroir, modification de tarif, suppression,
changement de rôle, écart de caisse, rebascule de palier de passage et forçage de disponibilité.
M. Koffi le consulte **depuis n'importe quel terminal**, et le filtre par utilisateur,
établissement, type d'action et période. Rien ne s'y modifie, rien ne s'y supprime — une
correction est une nouvelle entrée.

**Why this priority**: « **c'est ce que le propriétaire achète réellement** » (cadrage §8.3,
constitution principe IX). Le module se conçoit comme une fonctionnalité de premier plan, avec
son écran, ses filtres et sa lisibilité — pas comme un journal technique consulté en base.

**Independent Test**: écrire les types d'action livrables de ce cycle, les relire depuis un
terminal distinct avec les quatre filtres, et vérifier par contrôle outillé qu'aucun chemin de
code ne supprime ni ne modifie une entrée. Vérifier que la même entrée soumise trois fois n'en
produit qu'une, et que trois entrées appliquées dans les six ordres possibles donnent le même
état final.

**Acceptance Scenarios**:

1. **Given** un changement de rôle effectué par Adjoua sur le compte de Yao, **When** M. Koffi
   ouvre le journal depuis son téléphone, **Then** il voit qui, quoi, sur qui, quand — avec
   l'horodatage **d'autorité serveur**, jamais celui du terminal.
2. **Given** un journal contenant des entrées de plusieurs types, d'utilisateurs et de dates
   différentes, **When** M. Koffi filtre par utilisateur, par établissement, par type d'action et
   par période, **Then** chaque filtre restreint le résultat et les filtres se combinent.
3. **Given** une entrée écrite, **When** un chemin de code quelconque tente de la modifier ou de
   la supprimer, **Then** le contrôle outillé **échoue le build** — au même titre que
   l'interdiction de purge du journal d'événements (porte P-05b).
4. **Given** la même entrée d'audit soumise trois fois, **When** elle est traitée, **Then** un
   seul enregistrement existe (classe A, test de rejeu).
5. **Given** trois entrées d'audit, **When** elles sont appliquées dans les six ordres possibles,
   **Then** l'état final est identique (classe A, test de désordre).
6. **Given** les dix types d'action nommés par CPT-04, **When** le cycle est livré, **Then** la
   taxonomie est **complète et versionnée**, les types livrables sont branchés, les autres sont
   **déclarés dus** avec le cycle qui les doit, et le harnais **échoue** si un cycle livre
   l'action sans écrire son entrée d'audit.
7. **Given** un compte sans permission de consultation du journal, **When** il ouvre l'accueil,
   **Then** la tuile du journal est **absente** et l'accès direct est refusé.

---

### User Story 6 - Le hors-ligne ne fabrique jamais un droit (Priority: P3)

Aucune élévation de privilège n'est possible hors ligne. La création d'une `personne`, la
création d'un `compte`, l'attribution ou le retrait d'un rôle, la modification des référentiels
de rôles et de permissions sont **toutes de classe C** : elles exigent le cloud. En revanche,
l'écriture d'une entrée au journal d'audit est de **classe A** : tracer une ouverture de tiroir
faite hors ligne fonctionne hors ligne, alors que l'action tracée garde sa propre classe.

**Why this priority**: c'est une invariante, vérifiable par test dès que US2 et US3 existent.
Elle est en P3 parce qu'elle ne se construit pas — elle se prouve. Le point d'attention est
nommé au registre : « **Élévation de privilège — C2 — aucune élévation hors ligne, jamais.** »

**Independent Test**: exécuter le test de classe des sept opérations C du module — il échoue si
l'une d'elles est atteignable depuis un chemin de code exécutable hors ligne. Exécuter les tests
de rejeu et de désordre sur `journal_audit`, seule entité A du cycle.

**Acceptance Scenarios**:

1. **Given** les sept opérations de classe C du module, **When** le test de classe s'exécute,
   **Then** il échoue si l'une d'elles est atteignable hors ligne, et il **déclare le nombre
   d'opérations réellement inspectées** face au total attendu.
2. **Given** un terminal hors ligne, **When** l'utilisateur ouvre l'écran des utilisateurs et des
   rôles, **Then** l'indisponibilité est annoncée **immédiatement et explicitement**, avant toute
   saisie.
3. **Given** un terminal hors ligne, **When** une action de classe A traçable est effectuée,
   **Then** son entrée d'audit s'écrit localement et remonte au retour du réseau sans doublon.
4. **Given** un terminal quelconque, **When** on inspecte ce qu'il conserve, **Then** aucune
   donnée de classe C n'y est en cache d'**écriture** ; les référentiels de rôles et de
   permissions y sont en **lecture seule**, et le cache est purgé à la déconnexion.

---

### Edge Cases

- **Deux échecs de connexion doivent être indiscernables** — compte inexistant et mot de passe
  faux : même message, même code, même ordre de grandeur de temps de réponse. Un traitement plus
  rapide pour un compte inexistant révèle son inexistence aussi sûrement qu'un message explicite.
- **Le dernier propriétaire ne peut pas se retirer son propre rôle** — un établissement sans
  aucun compte habilité à attribuer les rôles est irrécupérable sans intervention de l'éditeur.
- **Un rôle retiré pendant une session active** — les permissions sont réévaluées au
  rafraîchissement suivant ; le retrait ne devient effectif qu'à ce moment, sauf révocation
  explicite de la session (voir Assumptions, 5).
- **Un identifiant déjà employé** — le refus ne dit pas si l'identifiant existe déjà ; il dit que
  la création est impossible, et l'opération est journalisée côté serveur.
- **Un compte sans aucun rôle** — la connexion réussit, l'accueil est un état vide explicite. Ce
  n'est pas une erreur : c'est l'état d'un compte fraîchement créé.
- **Une personne portant deux comptes** — modèle autorisé mais sans usage au MVP ; rien ne le
  suppose ni ne l'interdit.
- **Un compte à supprimer** — il se **désactive**, il ne se supprime pas : ses entrées d'audit
  doivent rester lisibles et attribuables des années plus tard.
- **La perte du magasin de sessions** — tout le monde se reconnecte, aucune donnée métier n'est
  perdue. Les sessions sont éphémères et reconstructibles par construction.
- **L'horloge d'un terminal qui dérive** — l'horodatage d'audit est celui du serveur ;
  l'horodatage du terminal est indicatif et sert à l'ordre d'affichage local uniquement.
- **Un établissement multiple** — un compte habilité sur deux établissements du même tenant voit
  les deux dans le journal d'audit, filtrables séparément.
- **Une tentative de connexion répétée** — les tentatives infructueuses sont limitées en débit
  sans révéler la raison du refus, et sans verrouiller définitivement un compte légitime.

## Requirements *(mandatory)*

### Functional Requirements

**Modèle — les trois tables distinctes (CPT-00)**

- **FR-001**: Le système DOIT porter trois entités distinctes et jamais confondues : `personne`
  (identité civile), `compte` (identité d'authentification, porteuse des rôles) et `employe`
  (contrat de travail). Aucune n'est un alias, une vue ni une extension d'une autre.
- **FR-002**: Le système DOIT autoriser une `personne` sans `compte`, un `compte` sans `employe`,
  et une `personne` portant `compte` et `employe` — les trois combinaisons sont valides et
  couvertes par des tests.
- **FR-003**: `employe` DOIT être livrée **provisionnée et vide** : table créée avec sa politique
  d'isolation, **aucune UI, aucun point d'entrée d'API, aucune logique**, conformément au patron
  de provision du cycle 002.
- **FR-004**: Le système NE DOIT PAS porter d'attribut de contrat, de rémunération, de date
  d'embauche ni de numéro CNPS ailleurs que sur `employe`. Un contrôle outillé échoue si un tel
  attribut apparaît sur `compte` ou `personne`.
- **FR-005**: Aucun chemin de code NE DOIT lire `employe` pour décider d'un droit,
  d'une permission ou d'une session.

**Authentification et sessions (CPT-01)**

- **FR-006**: Un `compte` DOIT s'identifier par un numéro de téléphone au format E.164 — indicatif
  par défaut hérité de la configuration de l'établissement, `+225` pour le pilote — **ou** par une
  adresse électronique.
- **FR-007**: Le système DOIT exiger un mot de passe fort, dont la politique est un paramètre
  d'établissement et non une constante du code.
- **FR-008**: Le système DOIT accepter la valeur `MOT_DE_PASSE` pour la méthode
  d'authentification et **refuser explicitement** la valeur `OTP_SMS`, avec un message nommant la
  raison — jamais l'ignorer silencieusement, jamais la traiter comme `MOT_DE_PASSE`.
- **FR-009**: Le système DOIT délivrer un accès de courte durée et un moyen de rafraîchissement
  **révocable**, permettant de rétablir l'accès sans ressaisie du mot de passe.
- **FR-010**: Un même compte DOIT pouvoir tenir plusieurs sessions simultanées sur des appareils
  distincts, chacune révocable indépendamment.
- **FR-011**: Le système DOIT permettre de lister les sessions actives d'un compte et d'en
  **révoquer une à distance** ; la révocation prend effet au refus du rafraîchissement suivant,
  sans délai supplémentaire.
- **FR-012**: Les messages d'erreur d'authentification NE DOIVENT JAMAIS révéler si un compte
  existe : message, code de retour et ordre de grandeur du temps de réponse sont identiques
  quelle que soit la cause de l'échec.
- **FR-013**: Le système DOIT limiter le débit des tentatives infructueuses sans révéler la
  raison du refus et sans verrouiller définitivement un compte légitime.
- **FR-014**: Le système DOIT permettre de **désactiver** un compte ; un compte désactivé ne se
  connecte plus et ne rafraîchit plus. La suppression physique d'un compte est interdite tant que
  des entrées d'audit lui sont attribuées.
- **FR-015**: Les sessions, accès et moyens de rafraîchissement SONT des données **éphémères et
  reconstructibles** : leur perte reconnecte les utilisateurs et ne perd aucune donnée métier.
  Aucune classe hors-ligne ne leur est attribuée, aucune sauvegarde ne les couvre.

**Rôles et permissions (CPT-02)**

- **FR-016**: Le référentiel de rôles DOIT porter exactement les huit valeurs : `proprietaire`,
  `gerant`, `receptionniste`, `serveur`, `caissier`, `magasinier`, `comptable`, `admin_editeur`.
- **FR-017**: Un `compte` DOIT pouvoir porter **N rôles**, et ses permissions effectives SONT
  **l'union** des permissions de ses rôles — sans priorité, sans arbitrage, sans ordre.
- **FR-018**: Retirer un rôle NE DOIT retirer que les permissions **exclusives** à ce rôle ;
  celles partagées avec un rôle conservé demeurent.
- **FR-019**: Les permissions DOIVENT être granulaires et **attachées aux modules d'activité**
  (forme `<module>.<objet>.<action>`). Une permission rattachée à un service non activé dans
  l'établissement n'ouvre aucune action et n'apparaît nulle part.
- **FR-020**: Le référentiel de permissions DOIT être en table, alimenté par seeds et
  **extensible sans migration** ; chaque cycle ultérieur y ajoute les permissions de ses écrans.
- **FR-021**: Un contrôle de complétude DOIT échouer si une permission déclarée ne garde aucune
  action, ou si une action sensible du périmètre livré s'exécute sans vérification de permission.
- **FR-022**: L'attribution, le retrait et l'élévation de rôle SONT de **classe C** : un test
  dédié échoue si l'une de ces opérations est atteignable depuis un chemin de code exécutable
  hors ligne.
- **FR-023**: Le système NE DOIT PAS permettre au dernier compte habilité à gérer les rôles d'un
  établissement de se retirer cette habilitation.
- **FR-024**: Toute attribution ou tout retrait de rôle DOIT écrire une entrée « changement de
  rôle » au journal d'audit, dans la même transaction que l'opération.

**Interface adaptée aux rôles (CPT-03)**

- **FR-025**: L'accueil DOIT être un tableau de bord de **tuiles filtrées par permission**,
  jamais un menu figé, conforme aux quatre états maquettés de `R1`.
- **FR-026**: Une action interdite, un module d'activité inactif ou une capacité inactive SONT
  **absents** de l'interface — jamais grisés, jamais accompagnés d'un message de disponibilité
  conditionnelle.
- **FR-027**: Une tuile issue de plusieurs rôles cumulés NE DOIT apparaître **qu'une seule fois**.
- **FR-028**: Le code d'un module NE DOIT PAS être chargé tant qu'aucun écran de ce module n'est
  ouvert (**chargement paresseux par module**), et un contrôle vérifie qu'un compte à rôle unique
  ne charge pas le code des autres modules.
- **FR-029**: Il n'y a **qu'une seule application** : le contrôle d'accès décide de ce qu'on peut
  faire, **jamais** de quelle application on lance.
- **FR-030**: Les écrans du cycle SONT `R0` Connexion (dérivé de `G2`, ligne à ajouter à la
  matrice **avant** de coder), `R1` Accueil (maquetté, quatre états), `G3` Utilisateurs et rôles
  (dérivé de `G2`) et `G4` Journal d'audit (dérivé de `R5` + `F2`). **Aucun autre écran ne se
  code dans ce cycle.**
- **FR-031**: Toute écriture de l'interface DOIT suivre le patron de `docs/module-dore.md`,
  « La septième couche » : appel typé, squelette de chargement, refus métier en langue
  utilisateur, validation au champ, action **absente** sans permission, refus immédiat hors ligne
  pour les opérations de classe C, rafraîchissement sans rechargement.
- **FR-032**: Tout champ de formulaire DOIT passer par le composant canonique `ChampSaisie` ;
  chaque écran est vérifié en mode clair **et** en mode sombre ; aucune chaîne visible n'est en
  dur, clés `fr` et `en` présentes.

**Journal d'audit (CPT-04)**

- **FR-033**: Le journal d'audit DOIT être **immuable** : aucune entrée ne se modifie ni ne se
  supprime ; une correction est une nouvelle entrée. Un contrôle outillé échoue le build s'il
  trouve un chemin de suppression ou de modification, sur le modèle de la porte P-05b.
- **FR-034**: Chaque entrée DOIT porter au minimum : l'auteur (`compte`), l'établissement, le
  type d'action, la cible, le contexte utile à la relecture, et l'**horodatage d'autorité
  serveur**. L'horodatage du terminal, s'il est conservé, est indicatif et ne sert qu'à l'ordre
  d'affichage local.
- **FR-035**: La taxonomie DOIT couvrir les **dix** familles nommées par CPT-04 : remise,
  annulation de ligne envoyée, avoir, ouverture de tiroir, modification de tarif, suppression,
  changement de rôle, écart de caisse, rebascule de palier de passage, forçage de disponibilité.
  Elle est versionnée dans le dépôt et non déduite du code.
- **FR-036**: Les types d'action **livrables dans ce cycle** SONT branchés de bout en bout ; les
  autres SONT **déclarés dus** avec le cycle qui les doit, et le harnais **échoue** si un cycle
  livre l'action sans écrire son entrée d'audit — même mécanique que le harnais progressif à
  étapes dues du cycle 002.
- **FR-037**: Le journal DOIT être consultable **depuis n'importe quel terminal** et filtrable
  par utilisateur, établissement, type d'action et période, les filtres étant combinables.
- **FR-038**: L'écriture d'une entrée d'audit EST de **classe A** : elle réussit hors ligne,
  supporte le rejeu (trois soumissions identiques → un enregistrement) et le désordre (six ordres
  → même état final). **L'action tracée garde sa propre classe** : le journal d'audit n'est pas
  un consommateur du journal d'événements outbox.
- **FR-039**: La consultation du journal EST protégée par permission ; sans elle, la tuile est
  absente et l'accès direct refusé.
- **FR-040**: Le journal d'audit NE DOIT PAS livrer d'export ni d'alertes configurables — ils
  relèvent de DIR-04, tranche T5.

**Provisions (CPT-05, CPT-06) et interdits permanents**

- **FR-041**: Le système DOIT provisionner la table `appareil_enrole` et les colonnes nécessaires
  à l'attestation d'intégrité et au relevé de position — **colonnes seulement, aucune logique,
  aucune UI, aucun point d'entrée d'API**.
- **FR-042**: Le système NE DOIT JAMAIS implémenter de verrouillage par adresse MAC : iOS et
  Android randomisent la MAC par réseau et une application Android ne peut pas lire la MAC
  matérielle. Aucune colonne, aucun champ, aucune trace de ce mécanisme n'existe.
- **FR-043**: Le paramètre de rayon de géorepérage DOIT être provisionné avec sa valeur par
  défaut de **300 m** et la mention explicite qu'il est **alerte seulement, jamais bloquant** —
  sa logique relève de CPT-06.

**Traçabilité, isolation et portes**

- **FR-044**: Toute nouvelle table DOIT porter `ENABLE` **et** `FORCE ROW LEVEL SECURITY` avec au
  moins une politique, et un test d'isolation multi-tenant sur chaque point d'entrée.
- **FR-045**: Tout changement d'état métier du cycle DOIT émettre un événement outbox **dans la
  même transaction**, et tout nouveau type d'événement DOIT être exercé **sur les deux tenants de
  démonstration** (leçon de la migration 0012).
- **FR-046**: Les seeds DOIVENT créer les comptes du pilote — le compte propriétaire de M. Koffi,
  le compte d'Adjoua **portant ses trois rôles**, le compte de Yao — de façon **idempotente et
  rejouable**, conformément à la mécanique de TRX-05a.
- **FR-047**: Chaque contrôle ou porte nouvelle DOIT **déclarer son périmètre inspecté**, compter
  les cibles réellement examinées face au total attendu, ne jamais modifier l'artefact inspecté,
  et prouver que sa cible n'est pas vide (§ Couverture des portes de la constitution).
- **FR-048**: Toute interdiction posée par ce cycle DOIT avoir son **versant positif** vérifié :
  le contrôle qui refuse une élévation hors ligne vérifie aussi que l'élévation en ligne
  fonctionne ; celui qui refuse la suppression d'une entrée d'audit vérifie aussi qu'une entrée
  s'écrit et se relit.

### Key Entities

- **`personne`** — identité civile : nom, pièce, contact. Partagée entre établissements d'un même
  tenant. **Classe C.** Ne porte aucun élément d'authentification ni de contrat.
- **`compte`** — identité d'authentification : identifiant (téléphone E.164 ou email), secret,
  état actif/désactivé, rattachement à une `personne`, méthode d'authentification. Porteur des
  rôles. **Classe C.**
- **`employe`** — contrat, salaire, date d'embauche, numéro CNPS. **PROVISION : table vide,
  aucune logique.** Jamais confondue avec `compte`. **Classe C.**
- **`role`** — référentiel des huit rôles. **Classe C.**
- **`permission`** — référentiel granulaire de forme `<module>.<objet>.<action>`, rattachée à un
  module d'activité. Extensible par seeds sans migration. **Classe C.**
- **`role_permission`** — ce que chaque rôle ouvre. **Classe C.**
- **`compte_role`** — le cumul : N lignes pour un compte, portée par établissement. Attribution
  et retrait tracés au journal d'audit. **Classe C.**
- **Session** — accès de courte durée + moyen de rafraîchissement révocable, par appareil.
  **Éphémère et reconstructible**, jamais classée, jamais sauvegardée.
- **`journal_audit`** — entrée immuable : auteur, établissement, type d'action, cible, contexte,
  horodatage d'autorité. Append-only, rétention illimitée. **Classe A.**
- **`appareil_enrole`** — **PROVISION** pour CPT-05 : colonnes d'enrôlement, d'attestation et de
  position. Aucune logique au MVP. **Classe C.**

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Adjoua se connecte **une seule fois** et accède à ses trois métiers — gérante,
  caissière, réceptionniste — sans jamais se reconnecter ni changer de compte.
- **SC-002**: Une tentative de connexion échouée est **indiscernable** selon que le compte existe
  ou non : message identique, code identique, temps de réponse du même ordre de grandeur sur
  100 tentatives de chaque type.
- **SC-003**: Une session perdue ou volée est coupée à distance et **ne se rouvre plus** dès la
  tentative de rafraîchissement suivante, sans affecter les autres sessions du même compte.
- **SC-004**: Quatre comptes de rôles différents ouvrent le même accueil et y voient quatre
  ensembles de tuiles différents, **sans qu'aucune action interdite n'apparaisse**, même grisée.
- **SC-005**: M. Koffi retrouve **qui a fait quoi, sur quoi et quand** pour n'importe quelle
  action tracée, depuis son téléphone, en moins de trois filtres.
- **SC-006**: **Zéro** entrée du journal d'audit modifiable ou supprimable par un chemin de code
  quelconque — prouvé par un contrôle outillé qui déclare le périmètre qu'il inspecte.
- **SC-007**: **Zéro** opération d'élévation de privilège atteignable hors ligne, sur les sept
  opérations de classe C du module, avec décompte des opérations inspectées face au total.
- **SC-008**: Les trois figures de CPT-00 — employé sans compte, compte sans contrat, les deux —
  fonctionnent de bout en bout, et **aucun** attribut de contrat n'existe hors de `employe`.
- **SC-009**: Un compte à rôle unique ne charge **aucun** code de module dont il n'a pas la
  permission.
- **SC-010**: Les dix familles d'actions de la taxonomie d'audit sont déclarées ; celles
  livrables sont branchées et testées, les autres portent le cycle qui les doit, et le harnais
  échoue si l'une est livrée sans sa trace.
- **SC-011**: Les quatre écrans du cycle sont conformes à leur référence visuelle, vérifiés en
  mode clair **et** sombre, sans chaîne en dur, avec parité des clés `fr` et `en`.
- **SC-012**: Les dix points de la Definition of Done sont vrais pour chacune des cinq stories,
  et les 24 portes de CI sont vertes.

## Assumptions

Défauts raisonnables retenus faute de précision dans les documents de référence. Ils sont
révisables en `/speckit-plan` ; chacun est signalé pour qu'aucun ne s'installe par omission.

1. **Amorçage du premier compte** — l'**admin éditeur** crée le premier compte propriétaire d'un
   tenant, via les seeds pour les tenants de démonstration et via la console pour un tenant réel.
   ADM-01 (T5) industrialisera ce provisionnement ; ce cycle livre le strict nécessaire pour
   qu'un établissement neuf ait au moins un compte capable de se connecter.
2. **Désactivation plutôt que suppression** — un compte se désactive, il ne se supprime pas, tant
   que des entrées d'audit lui sont attribuées. Supprimer un compte rendrait illisible le journal
   que le propriétaire achète.
3. **Identifiant de connexion** — téléphone E.164 **ou** email, l'un des deux suffit ; l'unicité
   de l'identifiant est **par tenant**, cohérente avec l'isolation multi-tenant.
4. **Portée d'un rôle** — un rôle est attribué **par établissement**, pas globalement au tenant :
   M. Koffi possède deux établissements aux besoins différents, et un gérant de l'un n'est pas
   gérant de l'autre. Le rôle `admin_editeur` fait exception et est de portée éditeur.
5. **Effet du retrait d'un rôle sur une session active** — les permissions sont réévaluées au
   **rafraîchissement suivant**. Couper instantanément toutes les sessions à chaque changement de
   rôle imposerait une vérification permanente ; révoquer explicitement la session reste
   possible et immédiat (FR-011).
6. **Types d'action livrables dès ce cycle** — « changement de rôle » (FR-024) et « suppression »
   (désactivation d'un compte, désactivation d'un service d'établissement). Les huit autres sont
   déclarés dus : remise → PDV-03 / SEJ-03 (T2), annulation de ligne envoyée → PDV-03 (T2),
   avoir → FIS-06 (T3), ouverture de tiroir → IMP-01 (T2), modification de tarif → PDV-01 (T2),
   écart de caisse → CAI-04 (T2), rebascule de palier de passage → HEB-04 (T1, cycle 4), forçage
   de disponibilité → HEB (T1).
7. **Rattachement du journal d'audit** — `journal_audit` vit dans `socle/comptes`, conformément à
   son classement au §5.2 du registre. Il est lu par les autres modules à travers un trait exposé,
   jamais par jointure inter-schémas (porte P-04).
8. **Politique de mot de passe** — paramètre d'établissement, valeurs par défaut conformes aux
   usages courants, jamais une constante du code (DoD point 9).
9. **Le pilote reste connecté pour l'administration** — la gestion des comptes et des rôles se
   fait au bureau, sur un poste relié. Sa classe C n'est donc pas une friction quotidienne :
   c'est une opération rare, faite en ligne.

## Out of Scope

Explicitement exclu de ce cycle. Chaque ligne dit **où** la chose est traitée, pour qu'aucune ne
se perde.

| Exclu | Où |
|---|---|
| **CPT-05** — enrôlement d'appareil, paire de clés Keystore/Keychain signant chaque requête, liste et révocation | P1, tranche T4, cycle 15. **Table `appareil_enrole` provisionnée ici** (FR-041) |
| **CPT-06** — Play Integrity, DeviceCheck + App Attest, géorepérage souple | P1, tranche T4, cycle 15. **Colonnes et paramètre de rayon provisionnés ici** (FR-041, FR-043) |
| **Verrouillage par adresse MAC** | **Jamais implémenté** — techniquement impossible (cadrage §12.2, principe IX). Aucune colonne, aucune trace |
| **OTP SMS** | Valeur `OTP_SMS` **refusée explicitement** (FR-008). Aucun fournisseur SMS n'est gelé ; à rouvrir quand il le sera |
| **Export et alertes du journal d'audit** | DIR-04, tranche T5 (FR-040) |
| **Provisionnement de tenant et facturation** | ADM-01 à ADM-06, tranche T5 |
| **Module RH** — contrat, paie, CNPS | Phase 2. **C'est précisément ce que CPT-00 rend possible sans refonte** |
| **Écrans hors des quatre nommés** | FR-030. Un écran absent de la matrice de dérivation ne se code pas (porte P-19) |
| **Décision ouverte O-01** — `personne` en C et le check-in d'un client inconnu hors ligne | À trancher **avant SEJ-02**. Ce cycle livre `personne` en C sans préempter la décision |
| **Fédération d'identité, authentification à deux facteurs, SSO** | Non demandés par CPT-01. Principe X : prêt ≠ construit |

## Dépendances

- **Cycle 001 (TRX)** — isolation multi-tenant et RLS forcée, journal d'événements outbox,
  contrat OpenAPI et génération du client, mécanique de seeds, crate `socle/comptes` en coquille.
- **Cycle 002 (ETB)** — référentiels de modules d'activité et de capacités : les permissions
  granulaires de FR-019 s'y rattachent, et le filtrage de l'accueil dépend des services
  réellement activés dans l'établissement.
- **`docs/design/derivation.md`** — l'ajout de la ligne `R0` Connexion (Q1) est un **préalable**
  au codage de l'écran de connexion, pas une conséquence.
- **`docs/module-dore.md`** — patron de tranche verticale (backend) et « La septième couche »
  (patron d'écriture front). Ce cycle branche **quatre** opérations d'écriture supplémentaires
  sur ce patron.
