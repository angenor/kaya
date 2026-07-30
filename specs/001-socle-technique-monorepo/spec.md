# Feature Specification: Socle technique du monorepo Kaya

**Feature Branch**: `001-socle-technique-monorepo` (aucune branche git dédiée créée — travail sur `main`)

**Created**: 2026-07-30

**Status**: Draft

**Input**: User description: "Socle technique du monorepo Kaya — TRX-01, TRX-02, TRX-02b, TRX-03, TRX-04, TRX-05 ; arborescence complète du monorepo (§0.1 des prompts) ; versions vérifiées et épinglées ; module doré écrit à la main ; registre des classes hors-ligne et sa porte de CI. Hors périmètre : TRX-06, TRX-07, TRX-08 — emplacement seulement."

## Contexte et traçabilité

Premier cycle du projet. Le dépôt est en phase 0 : aucun code n'existe. Ce cycle produit la
colonne vertébrale technique sur laquelle tous les cycles suivants s'appuient — il ne livre
aucune fonctionnalité métier visible par un exploitant d'établissement.

**Sources de vérité consultées** (ordre de préséance de la constitution) :

| Source | Sections utilisées |
|---|---|
| `.specify/memory/constitution.md` v1.0.2 | Principes I à XII, portes P-01 à P-20, Definition of Done |
| `docs/cadrage-v1.md` | §11 (classes hors-ligne), §13 (stack), §14 (provisions) |
| `docs/user-stories-v1.md` | Module TRX (TRX-01 à TRX-05, TRX-02b), §0.4 (DoD), §0.5 (tranches), §0.7 (tests hors-ligne) |
| `docs/registre-classes-offline.md` | Classement de référence, §11 (tests obligatoires par classe) |
| `docs/versions-gelees.md` v1.0.2 (2026-07-30) | Gel des dix briques du principe XI |
| `docs/Kaya_Prompts_SpecKit.md` §0.1 | Arborescence de référence du monorepo |

**Périmètre du cycle** : TRX-01, TRX-02, TRX-02b, TRX-03, TRX-04, TRX-05 — critères
d'acceptation repris tels quels, sans exigence ajoutée. Plus les trois tâches obligatoires du
cycle 1 (versions vérifiées et épinglées, module doré, registre des classes hors-ligne) et
l'arborescence complète du monorepo.

**Hors périmètre** : TRX-06 (conformité ARTCI), TRX-07 (mise à jour et télémétrie du parc),
TRX-08 (design system et thème) — leur **emplacement** est prévu dans l'arborescence, rien de
plus. Aucune UI, aucune logique.

**Persona** : Admin éditeur — console web, provisionne les tenants, diagnostique à distance
depuis Abidjan (`docs/user-stories-v1.md` §0.3). Sur ce cycle d'infrastructure, il est le seul
utilisateur : c'est lui qui installe, démarre, seed, diagnostique et restaure.

## Clarifications

### Session 2026-07-30

Résolus par les documents de référence, sans sollicitation :

- Tarifs et catégories des seeds Deloria → `docs/cadrage-v1.md` §2.1 (17 unités, 5 catégories,
  montants réels, salle de réunion 50 500/jour).
- Décomposition obligatoire des tarifs en prix HT + TVA + taxe de nuitée → `docs/cadrage-v1.md`
  §2.1, point de conformité.
- Barèmes de passage et plages de demi-journée → « Récapitulatif des paramètres
  d'établissement », `docs/user-stories-v1.md` (HEB-04, HEB-05), décision **B-07** ouverte.
- Schéma d'accueil du journal d'événements et classe du marquage « publié » →
  `docs/registre-classes-offline.md` §5.6.
- Rôle du binaire de nœud de site au cycle 1 → `docs/cadrage-v1.md` §10.1 (mode C = incrément 3).

Questions posées :

- Q: Sur quelle entité le module doré doit-il être écrit ? → A: `note_etablissement` — note
  interne libre attachée à un établissement, classe A, dans `socle/etablissements`.
- Q: Quelle référence visuelle l'écran du module doré doit-il suivre ? → A: un écran nouveau et
  minimal composé **exclusivement** de composants déjà spécifiés dans `docs/design/composants.md`,
  consommant `docs/design/theme.css`. Aucune nouvelle maquette normative n'est produite.
- Q: Que doit contenir `infra/` concernant le paquet auto-hébergé (mode B) au cycle 1 ? → A:
  **emplacement seulement** — répertoire et note de périmètre. Le paquet est livré avec TRX-07.
  **Précision apportée** : le binaire d'API applique les migrations **au démarrage**
  (`sqlx::migrate!()`), automatiquement et idempotemment (`docs/cadrage-v1.md` §10.2). Cette
  contrainte est structurante et coûteuse à rétrofitter — elle est donc tenue dès ce cycle,
  indépendamment du paquet auto-hébergé.
- Q: Où les sauvegardes chiffrées sont-elles externalisées, et qui porte leur immutabilité ? → A:
  un **stockage objet tiers, sur un hôte distinct du serveur de production**, avec **verrouillage
  d'objet** et rétention verrouillée. Le fournisseur exact est un choix de `/speckit-plan` ;
  l'invariant arrêté ici est : hôte distinct + verrouillage d'objet.

Fichiers cités comme sources mais **absents du dépôt** au 2026-07-30 : `docs/design/lexique.md`
(vocabulaire utilisateur) et `docs/design/derivation.md`. En attendant, le glossaire de l'annexe C
de `docs/cadrage-v1.md` et `docs/design/composants.md` (15 composants canoniques) en tiennent
lieu. Tout terme utilisateur nouveau est soumis avant d'être écrit en dur.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Le monorepo existe, compile et démarre sur des versions vérifiées (Priority: P1)

L'Admin éditeur clone le dépôt sur un poste neuf, lance une commande unique documentée, et
obtient une pile complète en marche : base de données, cache éphémère, stockage objet, API,
application. Toutes les briques tournent sur des versions vérifiées sur registre officiel et
épinglées exactement. Chaque crate du backend existe, même vide, et l'ensemble compile.

**Why this priority**: rien d'autre n'est démontrable tant que l'arborescence n'existe pas et
que le workspace ne compile pas. C'est la condition de possibilité des huit autres stories.
Priorité produit d'origine : préalable des stories P0 du module TRX (`docs/cadrage-v1.md`
§13.1 « mesures de vélocité, obligatoires, cycle 1 »).

**Independent Test**: sur une machine sans état préalable, cloner, exécuter la commande
d'amorçage documentée, constater que la pile répond, que la construction du workspace est verte
et que le manifeste de chaque brique porte une version exacte identique au gel.

**Acceptance Scenarios**:

1. **Given** un poste de développement sans le dépôt, **When** l'Admin éditeur clone et exécute
   la commande d'amorçage documentée, **Then** base de données, cache et stockage objet
   répondent, l'API répond sur sa sonde de santé et l'application se charge.
2. **Given** le dépôt cloné, **When** la construction complète du backend est lancée, **Then**
   elle réussit — tous les crates de `socle/`, `capacites/`, `verticales/`, le crate partagé
   `domain`, le binaire d'API et le binaire de nœud de site existent et compilent, même sans
   logique métier.
3. **Given** l'arborescence créée, **When** on inspecte le crate `socle/fiscalite`, **Then** le
   trait `JurisdictionAdapter` y est déclaré avec ses cinq opérations (`compute_taxes`,
   `required_document_fields`, `emission_channel`, `certify`, `remittance_reports`) et aucune
   règle fiscale ne vit ailleurs.
4. **Given** l'arborescence créée, **When** on inspecte `app/core/`, **Then** les six
   préoccupations transverses y sont présentes — authentification, contrôle d'accès cumulatif,
   internationalisation français et anglais avec français par défaut, thème clair et sombre,
   synchronisation, adaptateur de plateforme — et aucun composant n'appelle directement la
   couche native.
5. **Given** un manifeste de dépendances quelconque du dépôt, **When** on cherche un intervalle
   de version (`^`, `~`, `*`, plage), **Then** on n'en trouve aucun, et le fichier de
   verrouillage correspondant est présent et à jour.
6. **Given** une brique dont la version diffère du gel en vigueur, **When** on veut la changer,
   **Then** la nouvelle version est d'abord vérifiée sur le registre officiel, son URL est citée
   dans le changement, et `docs/versions-gelees.md` est mis à jour dans le même changement.
7. **Given** une modification d'une seule ligne dans un crate, **When** la construction
   incrémentale est relancée, **Then** elle bénéficie des trois optimisations exigées — éditeur
   de liens rapide, cache de compilation partagé, informations de débogage réduites aux tables
   de lignes en profil de développement.

---

### User Story 2 - Le module doré sert de patron à tous les cycles suivants (Priority: P1)

Avant toute génération assistée, l'Admin éditeur écrit **à la main** une tranche verticale
complète sur `note_etablissement` — une note interne libre attachée à un établissement, de classe
A : migration versionnée avec politique de sécurité au niveau ligne, accès aux données, service,
point d'entrée d'API documenté, tests unitaires et d'intégration, écran vérifié en clair et en
sombre. Cette tranche est documentée couche par couche et devient la référence que tous les
cycles suivants recopient.

**Why this priority**: exigence explicite du cadrage §13.1 et de la constitution (« module doré
d'abord »). Sans elle, chaque cycle réintroduira des tournures d'une version d'outil obsolète —
la documentation et les exemples en ligne de la bibliothèque d'accès aux données visent encore
la version précédente et **ne compileront pas** contre la version gelée
(`docs/versions-gelees.md` §2, arbitrage sqlx).

**Independent Test**: un développeur qui n'a pas écrit le module doré reproduit une seconde
tranche verticale équivalente en ne consultant que `docs/module-dore.md`, sans lire d'autre
source ni chercher d'exemple en ligne.

**Acceptance Scenarios**:

1. **Given** le module doré livré, **When** on parcourt ses couches, **Then** les sept couches
   sont présentes et écrites à la main : entité, migration avec sécurité au niveau ligne, accès
   aux données, service, point d'entrée d'API documenté, tests unitaires et d'intégration, écran
   applicatif.
2. **Given** le module doré, **When** on l'évalue contre les dix points de la Definition of Done
   (`docs/user-stories-v1.md` §0.4), **Then** les dix sont satisfaits — le point 10 (impression
   thermique) étant sans objet, ce qui est consigné explicitement.
3. **Given** l'écran du module doré, **When** on le consulte en mode clair puis en mode sombre,
   **Then** il est correct dans les deux modes, aucune chaîne n'est en dur, et les clés
   françaises et anglaises sont à parité.
4. **Given** le module doré, **When** on cherche une couleur ou un espacement littéral hors des
   jetons de conception, **Then** on n'en trouve aucun.
5. **Given** l'écran du module doré, **When** on inventorie ses éléments d'interface, **Then**
   ils proviennent **exclusivement** de `docs/design/composants.md` — ligne de liste, bouton
   principal, état vide illustré, témoin de synchronisation — et aucune nouvelle maquette
   normative n'a été produite dans `docs/design/html/`.
6. **Given** `docs/module-dore.md`, **When** on le lit, **Then** chaque couche y figure avec son
   extrait de référence, sa raison d'être et les pièges de la version gelée qu'elle neutralise.
7. **Given** `note_etablissement`, **When** on consulte `docs/registre-classes-offline.md`,
   **Then** elle y est déclarée en **classe A** dans `socle/etablissements` et porte les deux
   tests de classe A — rejeu triple et désordre commutatif.

---

### User Story 3 - Le journal d'événements est un grand livre permanent (Priority: P1)

Toute transition d'état métier écrit un événement dans la même transaction que l'écriture
métier. Cet événement n'est jamais supprimé, jamais modifié, et porte une charge utile
financière complète et dénormalisée. Un worker in-process le publie vers des consommateurs
idempotents. Des années plus tard, en phase 2, les écritures comptables SYSCOHADA seront
générées rétroactivement depuis ce seul journal.

**Why this priority**: TRX-02, P0. C'est la provision la plus déterminante du cadrage (§14.7).
Une erreur de conception ici — purge après publication, charge utile réduite à des
identifiants, mise à jour en place — est **irrattrapable a posteriori** : l'histoire comptable
n'existera plus.

**Independent Test**: exécuter le test de reconstitution autonome — un jeu d'événements
financiers est relu avec un accès restreint à la seule table d'événements, toutes les autres
tables étant inaccessibles, et chaque opération est reconstituée intégralement.

**Acceptance Scenarios**:

1. **Given** une transition d'état métier, **When** elle est enregistrée, **Then** un événement
   portant type, agrégat, identifiant de tenant, identifiant d'établissement, charge utile et
   horodatage est inséré **dans la même transaction** — si l'un échoue, les deux sont annulés.
2. **Given** un événement inséré, **When** le worker de publication in-process le traite,
   **Then** il est **marqué publié** et **reste présent** en base ; aucune suppression n'a lieu.
3. **Given** un événement déjà publié, **When** le même événement est présenté une seconde puis
   une troisième fois à un consommateur, **Then** l'effet observé est identique à une seule
   présentation.
4. **Given** un événement écrit, **When** une tentative de modification ou de suppression est
   faite par le rôle applicatif, **Then** elle est **refusée par la base de données**, pas par
   une convention de code.
5. **Given** une opération à corriger, **When** la correction est enregistrée, **Then** elle
   prend la forme d'un **nouvel événement**, l'original restant intact et lisible.
6. **Given** un événement d'encaissement, **When** on lit sa charge utile, **Then** elle porte
   le montant, le mode de règlement, la contrepartie, la ventilation de taxes et la référence de
   document — pas seulement des identifiants de lignes d'autres tables.
7. **Given** un jeu d'événements financiers publiés, **When** on tente de les relire avec un
   accès limité à la seule table d'événements, **Then** chaque opération est reconstituable
   intégralement **sans consulter aucune autre table** (test obligatoire de TRX-02).
8. **Given** le dépôt entier, **When** on cherche une file de messages externe, **Then** on n'en
   trouve aucune — l'outbox est l'unique frontière de découplage.
9. **Given** le dépôt entier, **When** on cherche un chemin de code, une tâche planifiée ou une
   migration qui supprime une ligne d'événement, **Then** on n'en trouve aucun.

---

### User Story 4 - Aucune donnée ne franchit la frontière d'un tenant (Priority: P1)

Chaque table porte un identifiant de tenant. La sécurité au niveau ligne est activée **et
forcée** sur toutes les tables, avec un rôle applicatif distinct du propriétaire des tables. Le
tenant courant est posé **dans chaque transaction**. Un utilisateur du tenant A ne lit ni
n'écrit aucune ligne du tenant B, sur aucun point d'entrée.

**Why this priority**: TRX-03, P0. Une fuite entre clients sur un produit multi-établissements
est fatale commercialement. La différence entre isolation et fuite tient exactement au fait de
poser le tenant par transaction plutôt qu'à l'ouverture de connexion, avec un pool de
connexions.

**Independent Test**: créer deux tenants seedés, s'authentifier sur le premier, et vérifier sur
chaque point d'entrée exposé qu'aucune ligne du second n'est lisible ni modifiable, y compris
par identifiant direct.

**Acceptance Scenarios**:

1. **Given** une table quelconque du schéma, **When** on inspecte sa définition, **Then** elle
   porte un identifiant de tenant, la sécurité au niveau ligne y est **activée et forcée**, et
   au moins une politique s'y applique.
2. **Given** une table nouvellement créée sans politique de sécurité au niveau ligne, **When**
   l'intégration continue s'exécute, **Then** **le build échoue** (porte P-07).
3. **Given** l'application connectée à la base, **When** on inspecte le rôle utilisé, **Then**
   c'est un rôle applicatif **distinct du propriétaire des tables**.
4. **Given** une requête traitée par l'API, **When** on inspecte la pose du tenant courant,
   **Then** elle a lieu **dans la transaction**, jamais à l'ouverture de la connexion.
5. **Given** un utilisateur authentifié sur le tenant A, **When** il appelle chaque point
   d'entrée exposé en visant une ressource du tenant B, **Then** aucune ligne n'est lue et
   aucune n'est écrite (porte P-08).
6. **Given** une transaction sans tenant courant posé, **When** une lecture est tentée,
   **Then** elle ne retourne aucune ligne — l'absence de contexte n'ouvre jamais l'accès total.

---

### User Story 5 - Le contrat d'API est généré et le client ne s'écrit jamais à la main (Priority: P2)

Les points d'entrée sont annotés dans le code ; le contrat est généré depuis ces annotations et
exposé sur `/api-docs/openapi.json`. L'intégration continue régénère le client TypeScript depuis
ce contrat : si le client régénéré diffère du client commité, le build échoue. Dès ce cycle, le
contrat existe et documente au moins la sonde de santé.

**Why this priority**: TRX-01, P0. Placée après les trois précédentes parce que le module doré
(US2) matérialise déjà un point d'entrée annoté et le contrat exposé ; cette story généralise la
règle et pose la porte de CI qui la rend non contournable.

**Independent Test**: modifier une signature de point d'entrée sans régénérer le client, pousser,
et constater que l'intégration continue échoue ; régénérer, commiter, et constater qu'elle passe.

**Acceptance Scenarios**:

1. **Given** l'API démarrée, **When** on appelle `/api-docs/openapi.json`, **Then** le contrat
   est retourné et documente au minimum la sonde de santé `/health`.
2. **Given** un point d'entrée exposé, **When** on inspecte son code, **Then** il porte son
   annotation de chemin et ses types portent leurs annotations de schéma et de paramètres.
3. **Given** l'environnement de production, **When** on tente d'atteindre l'interface
   d'exploration du contrat, **Then** elle est protégée ; hors production, elle est accessible.
4. **Given** une modification du contrat sans régénération du client, **When** l'intégration
   continue s'exécute, **Then** **le build échoue** sur le diff non commité (porte P-01).
5. **Given** le client TypeScript, **When** on inspecte son historique, **Then** il n'a jamais
   été édité à la main — chaque changement provient de la génération.

---

### User Story 6 - L'exploitation est observable et les sauvegardes sont restaurables (Priority: P2)

L'Admin éditeur diagnostique à distance depuis Abidjan, à 220 km du pilote. Les journaux sont
structurés et corrélés par requête, les erreurs remontent, une sonde de santé existe, et une
indisponibilité de plus de deux minutes déclenche une alerte. Une sauvegarde quotidienne
chiffrée est externalisée, et sa restauration complète est **testée et documentée**.

**Why this priority**: TRX-04, P0. Le support est distant ; sans corrélation ni télémétrie, le
diagnostic est impossible. Une sauvegarde dont la restauration n'a jamais été exécutée n'est pas
une sauvegarde.

**Independent Test**: provoquer une erreur applicative et retrouver la trace complète corrélée
par identifiant de requête ; puis restaurer une sauvegarde chiffrée dans un environnement vierge
en suivant la procédure écrite, et retrouver les données attendues.

**Acceptance Scenarios**:

1. **Given** une requête traitée, **When** on consulte les journaux, **Then** ils sont structurés
   et tous les événements de cette requête partagent un identifiant de corrélation.
2. **Given** une erreur applicative, **When** elle se produit, **Then** elle remonte au service
   de suivi des erreurs avec son contexte.
3. **Given** l'API démarrée, **When** on appelle la sonde `/health`, **Then** elle répond et
   reflète l'état des dépendances.
4. **Given** l'API indisponible plus de deux minutes, **When** le seuil est franchi, **Then**
   une alerte est déclenchée.
5. **Given** une journée écoulée, **When** on inspecte le stockage de sauvegarde, **Then** une
   sauvegarde chiffrée du jour est présente, externalisée hors du serveur applicatif et
   synchronisée vers le stockage objet.
6. **Given** une sauvegarde chiffrée et un environnement vierge, **When** l'Admin éditeur suit la
   procédure documentée, **Then** la restauration complète aboutit, elle est chronométrée et le
   résultat est consigné. L'exercice est rejoué **avant la bascule du pilote**, puis chaque
   trimestre.
7. **Given** la politique de sauvegarde, **When** on cherche d'où vient l'immutabilité des
   sauvegardes, **Then** elle est portée par le **stockage externe** (verrouillage d'objet), et
   **jamais** par le stockage objet auto-hébergé.

---

### User Story 7 - Deux tenants de démonstration se rechargent en une commande (Priority: P2)

L'Admin éditeur recharge à volonté un jeu de démonstration : le tenant Deloria avec son
établissement d'Abengourou et ses cinq modules d'activité, ses 17 unités, ses barèmes réels, son
catalogue et ses comptes de test ; et un second tenant « Résidence Test » réduit à l'hébergement
seul, qui prouve que le produit ne suppose jamais l'existence d'un point de vente.

**Why this priority**: TRX-05, P0. Sans jeu de données réaliste, ni le test d'isolation
multi-tenant, ni les démonstrations de fin de tranche, ni la validation de l'universalité du
modèle ne sont possibles.

**Independent Test**: exécuter la commande de rechargement deux fois de suite sur une base non
vierge et constater un état final identique et complet à chaque fois.

**Acceptance Scenarios**:

1. **Given** une base vierge, **When** la commande de seeds est exécutée, **Then** le tenant
   Deloria existe avec son établissement d'Abengourou — non classé, commune d'Abengourou, fuseau
   `Africa/Abidjan` — et ses modules hébergement, restauration, bar, pressing et salle de réunion
   activés.
2. **Given** les seeds chargés, **When** on inspecte le tenant Deloria, **Then** il porte 17
   unités réparties en 5 catégories aux tarifs réels, une salle de réunion, les barèmes de
   passage et de demi-journée, 30 articles de catalogue répartis sur les points de vente, et 5
   comptes de test aux rôles cumulés.
3. **Given** les seeds chargés, **When** on inspecte le second tenant, **Then** « Résidence
   Test » existe avec le **module hébergement seul** et 4 unités — aucun point de vente, et rien
   dans le produit ne s'en trouve cassé.
4. **Given** une base déjà seedée, **When** la commande de rechargement est relancée, **Then**
   elle aboutit en **une seule commande** et produit un état final identique.
5. **Given** les seeds, **When** on cherche où ils vivent, **Then** ils sont **séparés des
   migrations** et rejouables.

---

### User Story 8 - Le registre des classes hors-ligne est opposable (Priority: P3)

Toute entité du produit déclare sa classe de tolérance au hors-ligne — A, B, C ou D — dans un
registre versionné. Une entité absente du registre fait échouer le build. Les opérations de
classe B, C ou D ne sont atteignables depuis aucun chemin de code exécutable hors ligne.

**Why this priority**: transverse à tous les cycles suivants, mais sans effet observable tant
qu'aucune entité métier n'existe. Le registre `docs/registre-classes-offline.md` est **déjà
créé** (2026-07-30) et fait foi ; ce cycle livre la **porte de CI** qui le rend opposable et les
tests par classe.

**Independent Test**: ajouter une table sans déclarer son entité dans le registre, exécuter
l'intégration continue, constater l'échec ; déclarer l'entité, constater le passage.

**Acceptance Scenarios**:

1. **Given** `docs/registre-classes-offline.md`, **When** on le consulte, **Then** il porte les
   quatre classes du cadrage §11.1, l'arbre de décision §11.2 et le classement de chaque entité
   par crate.
2. **Given** une table présente au schéma mais dont l'entité n'est pas déclarée au registre,
   **When** l'intégration continue s'exécute, **Then** **le build échoue**.
3. **Given** une entité de classe A, **When** ses tests s'exécutent, **Then** le rejeu triple de
   la même écriture produit **un seul** enregistrement, et trois écritures appliquées dans les
   six ordres possibles produisent le **même état final** (`docs/user-stories-v1.md` §0.7).
4. **Given** une entité de classe B, C ou D, **When** on cherche un chemin de code exécutable
   hors ligne qui l'atteint, **Then** on n'en trouve aucun — vérifié par test, pas par convention
   (porte P-13).
5. **Given** un doute sur la classe d'une entité, **When** la décision est prise, **Then** c'est
   la classe **la plus stricte** qui s'applique, et la décision ouverte est consignée au registre.

---

### User Story 9 - Les provisions comptables existent en table, sans une ligne de logique (Priority: P3)

Deux tables sont créées et ne servent à rien aujourd'hui : la correspondance comptable et
l'exercice comptable. Elles évitent une migration douloureuse quand SYSCOHADA arrivera en
phase 2.

**Why this priority**: TRX-02b, priorité PROVISION. Quasi gratuit aujourd'hui, coûteux plus tard.
Aucune valeur utilisateur immédiate — d'où la priorité de séquencement la plus basse.

**Independent Test**: inspecter le schéma, constater la présence des deux tables avec leurs
colonnes, et l'absence totale de point d'entrée, d'écran et de règle métier les concernant.

**Acceptance Scenarios**:

1. **Given** le schéma, **When** on l'inspecte, **Then** la table de correspondance comptable
   existe avec identifiant de tenant, type d'événement, compte de débit, compte de crédit et
   journal.
2. **Given** le schéma, **When** on l'inspecte, **Then** la table d'exercice comptable existe
   avec début, fin et statut.
3. **Given** un exercice au statut clos, **When** une écriture est tentée sur sa période,
   **Then** elle est refusée — contrainte **distincte** de la clôture journalière et de la
   certification fiscale.
4. **Given** le dépôt entier, **When** on cherche une interface, un point d'entrée ou une règle
   métier consommant ces deux tables, **Then** on n'en trouve aucun. **Tables seulement.**

---

### Edge Cases

- **Que se passe-t-il quand le worker de publication redémarre au milieu d'un lot ?** Les
  événements non marqués publiés sont republiés ; les consommateurs étant idempotents, l'effet
  observé est inchangé. Aucun événement n'est perdu, aucun n'est supprimé.
- **Que se passe-t-il quand un consommateur échoue durablement ?** L'événement reste en base,
  non marqué publié, et reste republiable indéfiniment. Un consommateur défaillant ne bloque
  jamais la transaction métier qui a produit l'événement.
- **Que se passe-t-il quand une transaction métier échoue après l'écriture de l'événement ?**
  Les deux sont annulées ensemble — l'événement n'existe que si l'écriture métier a réussi.
- **Que se passe-t-il si un événement doit être corrigé ?** Un nouvel événement de correction est
  écrit. L'original n'est jamais touché. Aucun mécanisme d'édition n'existe.
- **Que se passe-t-il quand une requête arrive sans contexte de tenant ?** Aucune ligne n'est
  visible. L'absence de contexte n'est jamais interprétée comme un accès global.
- **Que se passe-t-il quand la restauration d'une sauvegarde échoue à l'exercice trimestriel ?**
  L'échec est consigné et traité comme un incident bloquant : une sauvegarde non restaurable
  n'en est pas une.
- **Que se passe-t-il quand le stockage objet auto-hébergé est compromis ?** Les sauvegardes
  restent intègres, leur immutabilité étant portée par le stockage externe et jamais par lui.
- **Que se passe-t-il si une brique du gel sort une version de sécurité en cours d'incrément ?**
  C'est la seule exception à la règle « aucune montée pendant un incrément » ; elle est vérifiée
  sur registre officiel, l'URL est citée, et la montée est consignée dans
  `docs/versions-gelees.md`.
- **Que se passe-t-il quand une capacité de plateforme est absente (impression, scan, OCR) ?**
  L'adaptateur de plateforme le **dit explicitement** à l'utilisateur. Il n'échoue jamais en
  silence et ne grise jamais sans explication.
- **Que se passe-t-il quand le module doré est modifié par un cycle ultérieur ?**
  `docs/module-dore.md` est mis à jour dans le même changement — sinon les cycles suivants
  recopient un patron périmé.

## Requirements *(mandatory)*

### Functional Requirements

#### Arborescence du monorepo — `docs/Kaya_Prompts_SpecKit.md` §0.1

- **FR-001**: Le dépôt DOIT porter l'arborescence de référence §0.1 : `backend/` (workspace),
  `app/`, `web/`, `clients/ts/`, `infra/`, `docs/`, `specs/`, `.github/workflows/`.
- **FR-002**: `backend/crates/` DOIT contenir le crate partagé `domain`, les neuf crates de
  `socle/` (`etablissements`, `comptes`, `caisse`, `fiscalite`, `documents`, `synchronisation`,
  `pilotage`, `editeur`, `metriques`), le crate `capacites/stocks`, et les quatre crates de
  `verticales/` (`hebergement`, `restauration`, `bar`, `pressing`).
- **FR-003**: `backend/` DOIT contenir le binaire `api/`, le binaire `node/` (nœud de site,
  incrément 3 — coquille seule) et `migrations/` avec `seeds/` à part.
- **FR-004**: Chaque crate DOIT compiler, même sans logique métier. La construction du workspace
  complet DOIT être verte.
- **FR-005**: Le trait `JurisdictionAdapter` DOIT être déclaré dans le crate `socle/fiscalite`,
  avec `compute_taxes`, `required_document_fields`, `emission_channel`, `certify` et
  `remittance_reports` (cadrage §14.1). Aucune règle fiscale ne DOIT vivre hors de ce trait.
- **FR-006**: Le crate `domain` DOIT être consommable par le binaire d'API, le binaire de nœud de
  site et la coquille Tauri — une seule implémentation du calcul de la taxe de nuitée.
- **FR-007**: La hiérarchie de dépendance DOIT être respectée : `socle/` ne dépend que de
  `socle/` ; `capacites/` dépend de `socle/` ; `verticales/` dépend de `socle/` et `capacites/`.
- **FR-008**: `app/` DOIT porter `modules/`, `core/` et `src-tauri/`. `app/core/` DOIT fournir
  authentification, contrôle d'accès cumulatif, internationalisation français et anglais avec
  français par défaut, thème clair et sombre, synchronisation et `PlatformAdapter` avec ses
  implémentations `desktop`, `android`, `ios` et `web`.
- **FR-009**: L'application unique DOIT fonctionner en mode SPA ; `web/qr/` DOIT être en rendu
  serveur et `web/console/` sans rendu serveur.
- **FR-010**: `infra/` DOIT fournir une composition de conteneurs de développement avec base de
  données, cache éphémère et stockage objet, aux versions gelées.
- **FR-010b**: `infra/` DOIT prévoir l'**emplacement seul** du paquet auto-hébergé (mode B) —
  répertoire et note de périmètre. Le paquet lui-même est livré avec TRX-07, hors périmètre.
- **FR-010c**: Le binaire d'API DOIT appliquer les migrations **au démarrage**, automatiquement et
  **idempotemment** (`docs/cadrage-v1.md` §10.2), quelle que soit la topologie de déploiement.
- **FR-011**: `.github/workflows/` DOIT porter une intégration continue **filtrée par chemins** —
  une modification du seul dossier de documentation ne déclenche pas la construction du backend.
- **FR-012**: L'arborescence DOIT **prévoir l'emplacement** de TRX-06 (conformité ARTCI), TRX-07
  (mise à jour et télémétrie du parc) et TRX-08 (design system et thème) — emplacement seulement,
  aucune interface, aucune logique.

#### Versions vérifiées et épinglées — principe XI, `docs/versions-gelees.md`

- **FR-013**: Les dix briques du principe XI — langage, cadre web, accès aux données, génération
  de contrat, cadre applicatif, cadre de style, coquille native, base de données, cache éphémère,
  stockage objet — DOIVENT être épinglées **exactement**, sans intervalle, sans `^`, sans `~`.
- **FR-014**: Chaque version retenue DOIT correspondre au gel en vigueur de
  `docs/versions-gelees.md` (v1.0.2, vérifié le 2026-07-30). Toute divergence DOIT être résolue
  par une vérification sur le registre officiel, **avec l'URL citée dans le changement**, et une
  mise à jour du gel dans le même changement.
- **FR-015**: Aucun numéro de version ne DOIT être proposé de mémoire.
- **FR-016**: Les fichiers de verrouillage DOIVENT être commités et à jour : verrouillage du
  workspace backend (y compris pour les binaires), verrouillage des dépendances applicatives,
  fichier de version du langage, fichier de version du runtime applicatif.
- **FR-017**: L'intégration continue DOIT échouer sur toute dépendance déclarée en intervalle et
  sur tout fichier de verrouillage absent ou périmé (porte P-20).
- **FR-018**: Le poste de développement étant `arm64` et la cible de production `linux/amd64`, la
  construction de production DOIT se faire en conteneur pour la cible, jamais par copie d'un
  binaire local.

#### Temps de compilation — cadrage §13.1

- **FR-019**: Un éditeur de liens rapide DOIT être configuré pour la construction de
  développement.
- **FR-020**: Un cache de compilation partagé DOIT être configuré.
- **FR-021**: Le profil de développement DOIT réduire les informations de débogage aux tables de
  lignes.
- **FR-022**: Le découpage en crates DOIT limiter la recompilation — une modification dans une
  verticale ne DOIT pas recompiler le socle.

#### Module doré — cadrage §13.1, constitution « module doré d'abord »

- **FR-023**: Une tranche verticale complète DOIT être écrite **à la main** sur
  `note_etablissement` — note interne libre attachée à un établissement, **classe A**, dans
  `socle/etablissements` — avant toute génération assistée, et couvrir les sept couches : entité,
  migration avec sécurité au niveau ligne, accès aux données, service, point d'entrée d'API
  documenté, tests unitaires **et** d'intégration, écran applicatif.
- **FR-024**: L'écran du module doré DOIT être vérifié en mode clair **et** en mode sombre, sans
  aucune chaîne en dur, avec parité des clés françaises et anglaises.
- **FR-024b**: L'écran du module doré DOIT être composé **exclusivement** de composants déjà
  spécifiés dans `docs/design/composants.md` et consommer `docs/design/theme.css`. **Aucune
  nouvelle maquette normative** ne DOIT être produite dans `docs/design/html/` par ce cycle.
- **FR-025**: Le module doré DOIT satisfaire les dix points de la Definition of Done
  (`docs/user-stories-v1.md` §0.4) ; tout point sans objet DOIT être consigné comme tel.
- **FR-026**: Le module doré DOIT être écrit contre la version gelée de la bibliothèque d'accès
  aux données, et DOIT neutraliser explicitement ses ruptures — assertion de sûreté sur les
  requêtes non littérales, changement de sortie des macros de requête.
- **FR-027**: `docs/module-dore.md` DOIT documenter chaque couche avec son extrait de référence,
  sa raison d'être et les pièges de version qu'elle neutralise.
- **FR-028**: `note_etablissement` DOIT être déclarée dans `docs/registre-classes-offline.md` en
  **classe A** sous `socle/etablissements`, et porter les deux tests de classe A — rejeu triple et
  désordre commutatif (`docs/user-stories-v1.md` §0.7).

#### TRX-01 — Contrat OpenAPI et génération du client

- **FR-029**: Les points d'entrée DOIVENT porter leur annotation de chemin ; les types exposés
  DOIVENT porter leurs annotations de schéma et de paramètres.
- **FR-030**: Le contrat DOIT être exposé sur `/api-docs/openapi.json`.
- **FR-031**: Le contrat DOIT documenter au minimum la sonde `/health` **dès ce cycle**.
- **FR-032**: L'interface d'exploration du contrat DOIT être protégée hors production.
- **FR-033**: L'intégration continue DOIT générer le client TypeScript depuis le contrat ; **un
  diff non commité DOIT faire échouer le build** (porte P-01).
- **FR-034**: Le client TypeScript ne DOIT jamais être édité à la main.

#### TRX-02 — Journal d'événements métier (grand livre permanent)

- **FR-035**: Toute transition d'état DOIT insérer un événement portant `{type, agrégat,
  tenant_id, etablissement_id, payload, horodatage}` **dans la même transaction SQL** que
  l'écriture métier.
- **FR-036**: Un worker de publication **in-process** DOIT consommer les événements. Aucune file
  de messages externe ne DOIT être introduite au MVP.
- **FR-037**: Les consommateurs (notifications, métriques) DOIVENT être idempotents — trois
  présentations du même événement produisent l'effet d'une seule.
- **FR-038**: **Rétention illimitée** — un événement publié DOIT être **marqué publié, jamais
  supprimé**. Aucun chemin de code, aucune tâche planifiée et aucune migration ne DOIT supprimer
  une ligne d'événement.
- **FR-039**: **Charge utile financière complète et dénormalisée** — un événement d'encaissement
  DOIT porter le montant, le mode de règlement, la contrepartie, la ventilation de taxes et la
  référence de document, et non de simples identifiants.
- **FR-040**: **Immuabilité** — un événement écrit ne DOIT jamais être modifié. L'interdiction de
  modification et de suppression DOIT être **portée par la base de données** pour le rôle
  applicatif, pas par une convention de code.
- **FR-041**: Une correction DOIT prendre la forme d'un **nouvel événement**.
- **FR-042**: **Test obligatoire de reconstitution autonome** — après publication, un événement
  DOIT rester lisible et sa charge utile DOIT suffire à reconstituer l'opération **sans consulter
  aucune autre table**. Le test DOIT s'exécuter avec un accès restreint à la seule table
  d'événements, les autres tables étant inaccessibles.
- **FR-043**: L'intégration continue DOIT échouer si une transition d'état n'émet pas d'événement
  dans sa transaction (porte P-05).

#### TRX-02b — Provisions comptables (PROVISION, tables seulement)

- **FR-044**: La table de correspondance comptable DOIT exister avec `{tenant_id, type_evenement,
  compte_debit, compte_credit, journal}`.
- **FR-045**: La table d'exercice comptable DOIT exister avec `{debut, fin, statut}`.
- **FR-046**: Une période close ne DOIT plus accepter d'écriture — contrainte **distincte** de la
  clôture journalière et de la certification fiscale.
- **FR-047**: **Aucune interface, aucune logique** ne DOIT consommer ces deux tables au MVP.

#### TRX-03 — Multi-tenant et sécurité au niveau ligne forcée

- **FR-048**: Chaque table DOIT porter `tenant_id`.
- **FR-049**: La sécurité au niveau ligne DOIT être **activée ET forcée** sur toutes les tables.
- **FR-050**: Le rôle applicatif DOIT être **distinct du propriétaire des tables**.
- **FR-051**: Le tenant courant DOIT être posé **dans chaque transaction**, jamais à l'ouverture
  de connexion.
- **FR-052**: L'intégration continue DOIT échouer si une table du schéma n'a **aucune politique**
  de sécurité au niveau ligne (porte P-07).
- **FR-053**: Un test d'isolation DOIT vérifier, **sur chaque point d'entrée**, qu'un utilisateur
  du tenant A ne lit ni n'écrit aucune ligne du tenant B (porte P-08).

#### TRX-04 — Observabilité et sauvegardes

- **FR-054**: Les journaux DOIVENT être structurés et corrélés par requête.
- **FR-055**: Les erreurs applicatives DOIVENT remonter au service de suivi d'erreurs avec leur
  contexte.
- **FR-056**: Une sonde `/health` DOIT être exposée.
- **FR-057**: Une indisponibilité de plus de **2 minutes** DOIT déclencher une alerte.
- **FR-058**: Une sauvegarde **quotidienne chiffrée** de la base DOIT être externalisée et
  synchronisée vers le stockage objet.
- **FR-059**: La **restauration complète** DOIT être **testée et documentée avant la bascule du
  pilote**, puis rejouée **chaque trimestre**.
- **FR-060**: L'immutabilité des sauvegardes DOIT être portée par le **stockage externe**
  (verrouillage d'objet), **jamais** par le stockage objet auto-hébergé.
- **FR-060b**: La destination des sauvegardes DOIT être un **stockage objet tiers hébergé sur un
  hôte distinct du serveur de production**, avec **verrouillage d'objet** et rétention verrouillée.
  Le stockage objet du produit tournant sur le serveur de production, l'y sauvegarder ferait tomber
  base et sauvegardes ensemble à la première compromission du serveur.

#### TRX-05 — Seeds et démonstration

- **FR-061**: Le tenant Deloria DOIT être seedé avec son établissement d'Abengourou — non classé,
  commune d'Abengourou, fuseau `Africa/Abidjan` — et les modules hébergement, restauration, bar,
  pressing et salle de réunion.
- **FR-062**: Le tenant Deloria DOIT porter **17 unités** réparties en **5 catégories** aux tarifs
  réels de `docs/cadrage-v1.md` §2.1 — Standard `A1–A3` à 12 500, Classique `B1–B5` à 15 500,
  Classique supérieure `C1–C4` à 17 500, Supérieure A `D1–D2` à 20 500, Supérieure B `E1–E3` à
  25 500 — plus une **salle de réunion à 50 500/jour**, les **barèmes de passage et de
  demi-journée** du récapitulatif des paramètres, **30 articles** de catalogue répartis sur les
  points de vente et **5 comptes de test aux rôles cumulés**.
- **FR-062b**: Les tarifs seedés DOIVENT être **décomposés en prix hors taxe, TVA et taxe
  communale de nuitée**, jamais stockés au montant affiché brut. Les tarifs actuels du pilote
  incluent 500 FCFA de taxe communale par catégorie ; les intégrer au prix au lieu d'en faire une
  ligne distincte place l'établissement en infraction (`docs/cadrage-v1.md` §2.1, point de
  conformité).
- **FR-062c**: Les valeurs de barème de passage seedées DOIVENT être identifiables comme
  **provisoires** tant que la décision **B-07** (barèmes réels du pilote, atelier terrain) n'est
  pas prise.
- **FR-063**: Un second tenant **« Résidence Test »** DOIT être seedé avec le **module hébergement
  seul** et **4 unités**, pour valider l'universalité du modèle.
- **FR-064**: Les seeds DOIVENT être rechargeables **en une seule commande** et rejouables — deux
  exécutions successives produisent le même état final.
- **FR-065**: Les seeds DOIVENT vivre **à part des migrations**.

#### Registre des classes hors-ligne — cadrage §11, constitution principe VI

- **FR-066**: `docs/registre-classes-offline.md` DOIT porter les **quatre classes** du cadrage
  §11.1, l'**arbre de décision** §11.2 et le classement de chaque entité par crate. *(Le fichier
  existe depuis le 2026-07-30 et fait foi ; ce cycle en livre l'opposabilité.)*
- **FR-067**: L'intégration continue DOIT **échouer si une entité présente au schéma n'est pas
  déclarée** dans le registre.
- **FR-068**: Toute entité de classe A DOIT porter un **test de rejeu** (trois envois de la même
  écriture produisent un seul enregistrement) et un **test de désordre** (trois écritures dans
  les six ordres possibles produisent le même état final).
- **FR-069**: Aucune opération de classe B, C ou D ne DOIT être atteignable depuis un chemin de
  code exécutable hors ligne — vérifié par test (porte P-13).
- **FR-070**: Toute écriture DOIT porter un identifiant unique **généré côté client** ; le serveur
  DOIT dédupliquer ; le rejeu DOIT être idempotent ; **le serveur fait foi en conflit**.

#### Portes d'intégration continue actives dès ce cycle

- **FR-071**: L'intégration continue DOIT échouer si un crate de `socle/` dépend d'un crate de
  `verticales/` (porte P-03).
- **FR-072**: L'intégration continue DOIT échouer si une requête joint deux schémas de modules
  différents (porte P-04).
- **FR-073**: L'intégration continue DOIT échouer sur toute valeur de capacité autre que `STOCK`
  au profil `SIMPLE` qui ne serait pas **refusée explicitement** (porte P-06).
- **FR-074**: L'intégration continue DOIT échouer sur toute chaîne utilisateur en dur et sur toute
  rupture de parité des clés françaises et anglaises (porte P-16).
- **FR-075**: L'intégration continue DOIT échouer sur toute couleur ou tout espacement littéral
  hors des jetons de conception (porte P-17).
- **FR-076**: L'intégration continue DOIT échouer si la vérification des requêtes à la compilation
  n'est pas verte (porte P-18).
- **FR-077**: L'intégration continue DOIT échouer si un fichier de maquette de
  `docs/design/html/` est copié sous `app/` (porte P-19).
- **FR-078**: L'intégration continue DOIT échouer si une migration déjà appliquée a été modifiée
  (porte P-02).
- **FR-079**: L'intégration continue DOIT échouer sur toute invocation directe de la couche native
  hors de `PlatformAdapter` (porte P-15).

### Key Entities

- **Événement de journal (`evenement_outbox`)** — l'entité centrale de ce cycle. Porte le type
  d'événement, l'agrégat concerné, l'identifiant de tenant, l'identifiant d'établissement, une
  charge utile **complète et dénormalisée**, un horodatage d'autorité serveur et un marqueur de
  publication. **Rétention illimitée, immuable.** Classe hors-ligne **A** (append-only,
  rétention illimitée — `docs/registre-classes-offline.md` §5.6). Écrite dans la transaction
  métier ; lue par le worker in-process et, en phase 2, par la génération d'écritures comptables
  rétroactives.
- **Correspondance comptable (`mapping_comptable`)** — provision. Associe un type d'événement à
  un compte de débit, un compte de crédit et un journal, par tenant. Table seulement.
- **Exercice comptable (`exercice_comptable`)** — provision. Période comptable avec début, fin et
  statut ; une période close n'accepte plus d'écriture. Table seulement.
- **Note d'établissement (`note_etablissement`)** — entité du module doré. Note interne libre
  attachée à un établissement : identifiant de tenant, identifiant d'établissement, auteur, texte,
  horodatage d'autorité. **Classe A** (append-only, commutatif, sans unicité, sans effet
  monétaire — arbre de décision du cadrage §11.2), dans `socle/etablissements`. Ne suppose ni
  hébergement ni point de vente. Support du patron : elle traverse les sept couches et porte les
  tests de rejeu triple et de désordre commutatif.
- **Tenant** et **établissement** — présents ici uniquement comme porteurs de l'isolation et des
  seeds. Leur modèle complet relève du cycle ETB, hors périmètre.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Sur un poste neuf, l'Admin éditeur passe du dépôt cloné à une pile complète en
  marche en **moins de 30 minutes**, en suivant une procédure écrite et sans intervention non
  documentée.
- **SC-002**: **100 % des situations non conformes injectées volontairement sont refusées** par
  l'intégration continue : client non régénéré, table sans politique de sécurité au niveau ligne,
  entité absente du registre des classes hors-ligne, dépendance d'un crate de socle vers une
  verticale, dépendance déclarée en intervalle, migration appliquée modifiée. Aucune n'est
  contournable par revue ou convention.
- **SC-003**: **100 % des opérations d'un jeu d'essai financier sont reconstituées** à partir de
  la seule table d'événements, toutes les autres tables étant rendues inaccessibles.
- **SC-004**: **Aucun événement n'est perdu ni supprimé** sur un exercice de redémarrage brutal
  du worker de publication au milieu d'un lot ; le nombre d'événements en base avant et après est
  identique, et l'effet observé chez les consommateurs est celui d'une seule présentation.
- **SC-005**: **Aucune ligne du second tenant n'est lisible ni modifiable** depuis le premier, sur
  **100 % des points d'entrée exposés**, y compris par identifiant direct.
- **SC-006**: Une **restauration complète** depuis une sauvegarde chiffrée aboutit dans un
  environnement vierge, en suivant la seule procédure écrite, **par une personne qui n'a pas écrit
  le système**. Sa durée est mesurée et consignée ; l'exercice est rejoué avant la bascule du
  pilote puis chaque trimestre.
- **SC-007**: Le rechargement complet des jeux de démonstration s'exécute **en une seule
  commande** et produit un **état final identique** sur trois exécutions successives.
- **SC-008**: Un développeur reproduit une **seconde tranche verticale complète** en ne consultant
  que `docs/module-dore.md`, sans autre source ni recherche en ligne.
- **SC-009**: **100 % des écrans livrés** sont vérifiés en mode clair et en mode sombre, et la
  **parité des clés françaises et anglaises est de 100 %**.
- **SC-010**: Le temps de recompilation incrémentale après modification d'une ligne est **mesuré
  et consigné**, avant et après activation des trois optimisations, et la réduction est constatée.
- **SC-011**: **Zéro numéro de version proposé de mémoire** : chaque version épinglée du dépôt
  correspond à une ligne de `docs/versions-gelees.md` portant l'URL de son registre officiel et sa
  date de vérification.
- **SC-012**: Une erreur applicative provoquée volontairement est **retrouvée en moins de 5
  minutes** depuis les journaux, par corrélation d'identifiant de requête, sans accès au serveur.

## Assumptions

Hypothèses retenues faute de précision dans les documents de référence. Chacune est un défaut
raisonnable, révisable en `/speckit-clarify` ou `/speckit-plan`.

1. ~~Entité du module doré~~ — **tranché en clarification** : `note_etablissement`, classe A, dans
   `socle/etablissements`. Voir la section Clarifications.
2. **Gel des versions déjà en vigueur** — `docs/versions-gelees.md` v1.0.2 a été vérifié le
   2026-07-30, soit le jour de ce cycle. La « première tâche obligatoire » est donc **satisfaite
   par constat de fraîcheur** : le cycle épingle les versions du gel et ne revérifie que si le gel
   a plus d'un mois ou si une brique doit changer. Toute revérification cite l'URL du registre.
3. **Registre des classes hors-ligne déjà créé** — `docs/registre-classes-offline.md` existe
   depuis le 2026-07-30 et couvre les quatre classes, l'arbre de décision et le classement par
   crate. La « troisième tâche obligatoire » se réduit donc à livrer la **porte de CI** qui le rend
   opposable et les tests par classe. Les trois décisions ouvertes qu'il consigne (O-01, O-02,
   O-03) restent ouvertes après ce cycle — aucune ne bloque le socle technique.
4. **Schéma d'accueil du journal d'événements** — le crate `socle/synchronisation`, conformément
   au classement de `docs/registre-classes-offline.md` §5.6.
5. **Portée du test de reconstitution autonome** — il s'exécute contre un rôle de base de données
   dont les droits de lecture sont limités à la seule table d'événements. C'est la seule manière de
   prouver l'autonomie de la charge utile plutôt que de la supposer.
6. **Restauration testée « avant la bascule du pilote »** — ce cycle livre la procédure écrite et
   **un premier exercice de restauration exécuté et chronométré** en environnement de
   développement. L'exercice en conditions de production reste dû avant la bascule.
7. **Portes de CI sans objet à ce cycle** — certaines portes (P-09 disponibilité, P-10 montants et
   quantités, P-11 et P-12 fiscalité) n'ont aucune cible tant qu'aucune entité métier n'existe.
   Elles sont **installées et vertes à vide** dès ce cycle plutôt qu'ajoutées plus tard, pour
   qu'aucun cycle ultérieur ne puisse les livrer sans les activer.
8. **Alerte d'indisponibilité** — le seuil de 2 minutes de TRX-04 est mesuré depuis la sonde de
   santé, par une supervision externe au serveur applicatif ; une supervision hébergée sur la
   machine surveillée ne prouve rien.
9. ~~Tarifs et barèmes des seeds~~ — **résolu par les documents** : les 17 unités, les 5
   catégories et leurs tarifs sont au `docs/cadrage-v1.md` §2.1 ; les barèmes de passage et les
   plages de demi-journée au « Récapitulatif des paramètres d'établissement » de
   `docs/user-stories-v1.md`. Voir FR-062 à FR-062c. Seule la composition des **30 articles** de
   catalogue n'est pas documentée : des articles représentatifs du bar et de la restauration du
   pilote sont utilisés, leur liste exacte n'ayant aucun effet structurant.
10. **Point 10 de la Definition of Done** — « tout document imprimé vérifié sur imprimante
    thermique réelle » est **sans objet** sur ce cycle : aucun document n'est imprimé. Cette
    absence est consignée explicitement plutôt que passée sous silence.
11. **Vocabulaire utilisateur** — `docs/design/lexique.md` et `docs/design/derivation.md` sont
    cités comme sources normatives mais **n'existent pas encore**. Le glossaire de l'annexe C de
    `docs/cadrage-v1.md` et `docs/design/composants.md` en tiennent lieu. Les libellés utilisateur
    de l'écran du module doré — le terme retenu pour `note_etablissement` en français et en
    anglais — sont **soumis avant d'être écrits en dur**.

## Dependencies

- **`docs/versions-gelees.md`** v1.0.2 (2026-07-30) — fait foi pour les dix briques. Prochaine
  revue groupée : 2026-08-31.
- **`docs/registre-classes-offline.md`** (2026-07-30) — fait foi pour la classe de chaque entité.
- **`docs/design/theme.css`** — **seul** fichier de `docs/design/` copié tel quel vers
  `app/assets/css/` (constitution principe XII). Le reste de `docs/design/html/` est une cible
  normative, jamais une source.
- **Cible de déploiement** — Docker sur VPS Contabo, `linux/amd64` (`docs/versions-gelees.md`).
  Le poste de développement est `arm64` : les images de base sont multi-architecture, le binaire
  compilé ne l'est pas.
- **Stockage de sauvegarde tiers** — un compte de stockage objet chez un fournisseur distinct de
  celui du serveur de production, prenant en charge le verrouillage d'objet avec rétention
  verrouillée. Sans lui, TRX-04 n'est pas livrable et l'exercice de restauration exigé avant la
  bascule du pilote ne peut pas avoir lieu. Fournisseur à arrêter en `/speckit-plan`.
- **Confirmation attendue** — le choix de la version de la bibliothèque d'accès aux données doit
  être **confirmé par le spike sur les contraintes d'exclusion et les intervalles de temps**
  (cadrage §16, `docs/versions-gelees.md` en-tête). C'est le seul point du gel resté ouvert.

## Out of Scope

Explicitement exclu de ce cycle. Chaque élément a son emplacement prévu dans l'arborescence,
rien de plus.

| Exclu | Référence | Raison |
|---|---|---|
| Conformité ARTCI — export et suppression des données d'une personne, rétention paramétrable, consentement tracé | TRX-06, P1 | Emplacement prévu seulement |
| Mise à jour et télémétrie du parc — serveur de mise à jour desktop, bundle de diagnostic | TRX-07, P1 | Emplacement prévu seulement |
| Design system et thème — jetons, 12 composants canoniques, règle d'analyse | TRX-08, P1 | Emplacement prévu seulement ; le thème minimal clair/sombre de `app/core/` suffit au module doré |
| Toute entité métier — établissements, comptes, unités, séjours, caisse, fiscalité | Modules ETB, CPT, HEB, SEJ, CAI, FIS | Cycles suivants de la tranche T1 |
| Nœud de site fonctionnel | Incrément 3 | Le binaire existe en coquille, sans logique |
| Paquet auto-hébergé (mode B) | Cadrage §10.1–10.2, TRX-07 | Emplacement seul dans `infra/`. Les migrations idempotentes au démarrage sont **dans** le périmètre (FR-010c) |
| Nouvelle maquette normative dans `docs/design/html/` | TRX-08, P1 | L'écran du module doré se compose des composants existants (FR-024b) |
| Extraction d'un service, file de messages externe | Cadrage §13.2 | Aucun service extrait au MVP |
| Toute logique sur les provisions comptables | TRX-02b | **Tables seulement** |
