# Feature Specification: Tenants, établissements, modules d'activité et configuration héritée

**Feature Branch**: `002-etablissements-modules-activite` (aucune branche git dédiée créée — travail sur `main`, comme au cycle 001)

**Created**: 2026-07-31

**Status**: Draft

**Input**: User description: "Tenants, établissements, modules d'activité et configuration héritée — ETB-01, ETB-02, ETB-02b, ETB-02c, ETB-03, ETB-04, ETB-05. Deux référentiels distincts en table (module d'activité et capacité) ; seule STOCK au profil SIMPLE implémentée, toute autre valeur refusée explicitement ; l'interface ne montre jamais un module ni une capacité inactifs ; trois tests structurels obligatoires écrits avant l'implémentation et exécutés en intégration continue pour toujours ; seeds Deloria et Résidence Test. Hors périmètre : ETB-06, ETB-07, ETB-08."

## Contexte et traçabilité

Deuxième cycle du projet, tranche T1 (`docs/user-stories-v1.md` §0.5). Le cycle 001 a livré la
colonne vertébrale technique : monorepo, isolation multi-tenant, journal d'événements, module
doré, portes de conformité. Il a créé `tenant` et `etablissement` en **forme minimale et
assumée** — nom, fuseau horaire, devise — en écrivant dans la migration qui les porte que
« **ETB-01 les enrichira par migration additive, jamais en modifiant ce fichier** ». Ce cycle
honore cette dette et livre la **première fonctionnalité visible par un exploitant**.

C'est aussi le cycle qui décide si le produit reste extensible. Tout le reste de Kaya — séjours,
commandes, caisse, fiscalité — se posera sur les référentiels et la résolution de configuration
définis ici. Une spécialisation hôtelière introduite maintenant dans le socle ne se verrait pas
avant trois tranches.

**Sources de vérité consultées** (ordre de préséance de la constitution) :

| Source | Sections utilisées |
|---|---|
| `.specify/memory/constitution.md` v1.2.0 | Préambule, principes I à XII, portes P-01 à P-20 (P-05b incluse), § Couverture des portes, Definition of Done |
| `docs/cadrage-v1.md` | §2.1 (pilote), **§4 (modèle d'entité universel)**, §11.3 (classement de référence), §13.2 (frontière), **§14 (provisions)** |
| `docs/user-stories-v1.md` | **Module ETB (ETB-01 à ETB-05, ETB-02b, ETB-02c)**, §0.3 (personas), §0.4 (DoD), §0.5 (tranches), §0.7 (tests hors-ligne), Récapitulatif des paramètres d'établissement |
| `docs/registre-classes-offline.md` v1.0.1 | §5.1 (`socle/etablissements` — quatorze entités classées), §11 (tests obligatoires par classe) |
| `docs/design/derivation.md` v1.0.0 | Ligne `G1` Établissement et modules → hérite de `G2` |
| `docs/design/lexique.md` v1.0.0 | Vocabulaire utilisateur opposable — « Votre établissement », « Vos services » |
| `docs/design/html/G2-offre-hebergement.html` | Référence normative du motif de configuration |
| `specs/001-socle-technique-monorepo/` | Dette d'écran (FR-024), forme minimale de `tenant` et `etablissement`, mécanique de seeds |

**Périmètre du cycle** : ETB-01, ETB-02, ETB-02b, ETB-02c, ETB-03, ETB-04, ETB-05 — critères
d'acceptation repris **tels quels**, sans exigence ajoutée ni retranchée.

**Hors périmètre** : ETB-06 (sélecteur de contexte, P1), ETB-07 et ETB-08 (PROVISIONS —
**tables seulement, aucune UI, aucune logique**). Voir § Out of Scope.

**Personas** :

- **Admin éditeur** — console web depuis Abidjan. C'est lui qui provisionne un tenant, crée le
  premier établissement, active les services et pose l'identité visuelle. Sur ce cycle, il est le
  seul à écrire.
- **M. Koffi (propriétaire)** — possède Deloria et une résidence meublée. **Ne saisit jamais
  rien.** Il consulte depuis son téléphone. Sa présence au périmètre impose deux choses : ses deux
  établissements ne se ressemblent pas (cinq services contre un seul), et ce qu'il voit ne doit
  jamais mentionner un service qu'il n'a pas.

## Clarifications

### Session 2026-07-31

Résolus par les documents de référence, sans sollicitation :

- Classe hors-ligne des quatorze entités du cycle → `docs/registre-classes-offline.md` §5.1 :
  toutes en **C** (référentiel), sauf `note_etablissement` (A, livrée au cycle 001) et la
  sélection d'établissement actif (A, relevant d'ETB-06, hors périmètre).
- Référence visuelle de l'écran de configuration → `docs/design/derivation.md` : `G1`
  « Établissement et modules » **hérite de `G2`**. La dette d'écran du cycle 001 est donc levée
  ici : un motif hérité existe, l'écran se code.
- Vocabulaire utilisateur de « module d'activité » → `docs/design/lexique.md` : « **Vos
  services** ». Celui de « tenant » : il **n'existe pas** pour l'utilisateur, qui lit « Votre
  établissement ».
- Valeurs des référentiels de modules et de capacités, et leur extensibilité sans migration →
  `docs/cadrage-v1.md` §14.3 et §14.4.
- Paramètres d'établissement déjà arrêtés (classement, commune, fuseau, devise, modules actifs)
  → « Récapitulatif des paramètres d'établissement », `docs/user-stories-v1.md`.

Questions posées :

- **Q: Les trois tests structurels exigent « commande, encaissement, document fiscal, clôture » —
  or rien de tout cela n'existe avant T2 et T3. Comment les tenir dès maintenant ?**
  → **A: harnais progressif à étapes dues.** Les trois tests sont écrits et **verts dès ce cycle**
  sur les étapes livrables ; les étapes non encore livrées sont **déclarées dues** avec le cycle
  qui les doit ; le harnais **échoue** si un cycle livre l'étape sans la brancher. Même logique
  que TRX-05b, la tâche de recollement de fin de tranche. Aucun test n'est jamais rouge ni ignoré.
  Voir FR-021 à FR-028.
- **Q: Le cycle 001 a reporté « la couche écran » au cycle ETB. Quels écrans ce cycle livre-t-il ?**
  → **A: `G1` seul, toutes sections.** Un écran unique — identité de l'établissement, services
  actifs, points de vente, identité visuelle avec aperçu — couvrant ETB-01, ETB-02, ETB-02b,
  ETB-03 et ETB-05. Aucun écran hors matrice de dérivation. L'accueil `R1` reste au cycle CPT :
  son filtrage par permission dépend de rôles qui n'existent pas encore, et le livrer à moitié
  imposerait de le rouvrir.

Tranchés par défaut raisonnable, consignés en § Assumptions et révisables en `/speckit-plan` :
portée du refus des profils de stock (2), statut du service fictif (3), rattachement de caisse
d'un point de vente (5), politique d'impression comme paramètre (6), stockage du logo (7),
services déclarant `STOCK` chez Deloria (9).

## User Scenarios & Testing *(mandatory)*

> Les priorités `P1`/`P2`/`P3` ci-dessous sont les **priorités d'implémentation du modèle Spec
> Kit** — l'ordre dans lequel les tranches se construisent. Elles ne remplacent pas les priorités
> produit `P0`/`P1`/`P2`/`PROVISION` de `docs/user-stories-v1.md`, qui restent la référence de
> périmètre : **ETB-01 à ETB-05 sont toutes P0**.

### User Story 1 - Un établissement à service unique fonctionne de bout en bout (Priority: P1)

Trois établissements témoins gardent le produit en permanence : un **maquis** qui ne fait que de
la restauration, une **résidence meublée** qui ne fait que de l'hébergement, et un établissement
portant un **service fictif minimal qui ne consomme aucune capacité**. Chacun est créé, exploité
et clôturé de bout en bout sans qu'aucune ligne de code ne suppose l'existence d'un hébergement,
d'un point de vente ou d'un stock.

**Why this priority**: c'est la seule story qui, si elle tombe, invalide tout le reste. ETB-02c la
nomme « le garde-fou de toute extension future du produit » — s'il tombe un jour, le socle s'est
spécialisé sans qu'on le voie. Elle est écrite **avant** l'implémentation, précisément pour
qu'elle contraigne la conception au lieu de la constater.

**Independent Test**: exécuter les trois parcours en intégration continue sur une base vierge.
Chacun crée son établissement, active son seul service, résout sa configuration et déroule les
étapes disponibles à ce jour. Aucun n'active `HEBERGEMENT` ni ne crée de point de vente, sauf le
maquis pour son propre service. Le troisième n'active **aucune** capacité.

**Acceptance Scenarios**:

1. **Given** une base vierge, **When** on crée un établissement dont le seul service actif est
   `RESTAURATION`, **Then** l'établissement est exploitable : sa configuration se résout, ses
   points de vente se créent, et aucune opération ne réclame ni unité louable, ni séjour, ni
   formule.
2. **Given** une base vierge, **When** on crée un établissement dont le seul service actif est
   `HEBERGEMENT`, **Then** l'établissement est exploitable **sans qu'aucun point de vente
   n'existe**, et aucune opération ne réclame de catalogue ni de table.
3. **Given** un service d'activité **fictif minimal**, créé pour le seul besoin du test et ne
   déclarant **aucune** capacité, **When** on crée un établissement qui ne porte que lui,
   **Then** les étapes livrées à ce jour — création, activation, résolution de configuration —
   réussissent, et les étapes dues aux cycles suivants (vente comptoir, encaissement, document
   fiscal, clôture journalière) sont **déclarées, nommées et attribuées à leur cycle**.
4. **Given** le harnais des trois parcours, **When** un cycle ultérieur livre l'une des étapes
   dues sans la brancher aux trois parcours, **Then** l'intégration continue **échoue** en
   nommant l'étape non branchée et le parcours concerné.
5. **Given** les trois parcours verts, **When** un crate du socle acquiert une dépendance vers
   une verticale, **Then** l'intégration continue échoue (porte P-03, en place depuis le
   cycle 001).
6. **Given** le service fictif minimal, **When** on inspecte les jeux de données de démonstration
   et de production, **Then** il n'y figure **jamais** — il n'existe que dans le harnais de test.

---

### User Story 2 - L'exploitant décrit son établissement et choisit ses services (Priority: P1)

L'Admin éditeur ouvre l'écran de configuration d'un établissement. Il renseigne ce qui identifie
l'établissement — juridiction, classement, commune, fuseau horaire, devise, adresse, numéro de
compte contribuable — puis active les services que l'établissement rend. Un service qu'il n'a pas
activé n'apparaît nulle part dans le produit : pas grisé, pas annoncé, **absent**.

**Why this priority**: ETB-01 et ETB-02, toutes deux P0. C'est la première chose qu'un exploitant
voit du produit, et la règle « absent, pas grisé » est un invariant de la constitution
(principe VII) qu'aucun écran ultérieur ne pourra rattraper s'il est manqué ici.

**Independent Test**: créer deux établissements aux services différents — cinq services pour le
premier, un seul pour le second — et vérifier sur l'interface rendue de chacun qu'aucun service
non activé n'y apparaît sous quelque forme que ce soit.

**Acceptance Scenarios**:

1. **Given** un tenant existant, **When** l'Admin éditeur crée un établissement en renseignant
   juridiction, classement, commune, fuseau horaire, devise, adresse et numéro de compte
   contribuable, **Then** l'établissement est créé et un événement de journal en porte la trace
   complète.
2. **Given** un établissement, **When** l'Admin éditeur active un service, **Then** le service
   devient présent dans l'interface, et un événement de journal enregistre l'activation dans la
   même transaction.
3. **Given** un établissement dont le service `PRESSING` n'est pas activé, **When** on inspecte
   l'interface rendue, **Then** le mot « pressing » **n'y figure sous aucune forme** — ni entrée
   désactivée, ni mention « disponible dans votre offre », ni marqueur masqué.
4. **Given** un établissement à cinq services actifs, **When** l'Admin éditeur en désactive un,
   **Then** le service disparaît de l'interface, **aucune donnée n'est supprimée**, et la
   réactivation restitue l'état antérieur.
5. **Given** un tenant, **When** on lui rattache un second, puis un troisième établissement,
   **Then** aucune limite ni aucun effet tarifaire n'apparaît — le nombre d'établissements est
   sans conséquence sur la facturation.
6. **Given** un établissement **sans aucun service actif**, **When** on l'ouvre, **Then** il est
   valide et son interface est vide de tout service, sans erreur ni écran cassé.

---

### User Story 3 - Une capacité non implémentée est refusée, jamais ignorée (Priority: P1)

Les capacités sont le second référentiel : ce dont un service a besoin pour fonctionner, par
opposition à ce que l'établissement fait. Un service **déclare** les capacités qu'il consomme. Au
MVP, une seule est implémentée — la gestion de stock, dans son profil le plus simple. Toute autre
valeur est refusée avec un message qui nomme la valeur et le motif du refus. Jamais acceptée en
silence, jamais ignorée.

**Why this priority**: ETB-02b, P0, et **porte P-06 de la constitution**. Une capacité acceptée en
silence produirait un établissement configuré pour un comportement qui n'existe pas — l'écart ne
se découvrirait qu'à l'usage, chez le client.

**Independent Test**: tenter d'écrire chacune des six capacités non implémentées, puis chacun des
trois profils de stock non implémentés, par tous les chemins d'écriture existants. Vérifier que
chaque tentative est refusée et que le refus nomme la valeur.

**Acceptance Scenarios**:

1. **Given** un établissement au service `RESTAURATION`, **When** on déclare qu'il consomme la
   capacité `STOCK` au profil `SIMPLE`, **Then** la déclaration est acceptée.
2. **Given** le même établissement, **When** on déclare qu'il consomme `LIVRAISON`, `PRODUCTION`,
   `COMMERCE_EN_LIGNE`, `FIDELITE`, `DEVIS` ou `COMPTES_CLIENTS`, **Then** l'écriture est
   **refusée** avec un message qui nomme la capacité et indique qu'elle n'est pas implémentée au
   MVP — et **aucune ligne n'est écrite**.
3. **Given** une déclaration de `STOCK`, **When** on lui donne le profil `VALORISE`, `DETAILLE`
   ou `AUCUN`, **Then** l'écriture est refusée avec un message qui nomme le profil ; le message
   du profil `AUCUN` indique qu'une capacité non consommée **ne se déclare pas** plutôt que de se
   déclarer à zéro.
4. **Given** les six capacités non implémentées, **When** on ouvre l'écran de configuration,
   **Then** **aucune n'est proposée** — le refus explicite protège les autres chemins d'écriture
   (interface de programmation, import, jeu de données), il ne remplace pas la règle « absent, pas
   grisé ».
5. **Given** le référentiel de capacités, **When** on l'inspecte, **Then** les sept valeurs y
   figurent en **table** — l'ajout d'une huitième est une écriture de configuration, jamais une
   migration.
6. **Given** un service consommant `STOCK`, **When** ce service est désactivé, **Then** la
   déclaration de consommation devient inerte sans être supprimée, et la réactivation la restitue.

---

### User Story 4 - La configuration se résout par héritage, avec surcharge à chaque niveau (Priority: P1)

Tout paramètre du produit se résout en descendant une chaîne à quatre niveaux : tenant, puis
établissement, puis service, puis point de vente. Chaque niveau peut surcharger le précédent. La
valeur qui s'applique est celle du niveau le plus proche qui en définit une, et l'origine de cette
valeur est toujours connue.

**Why this priority**: ETB-04, P0, et **c'est le composant le plus réutilisé du produit**. Tous
les modules suivants s'en servent : temps de remise en état (HEB), heures d'arrivée et de départ
(HEB), barème de passage (HEB), taux de TVA et taxe de nuitée (FIS), seuil d'écart de caisse
(CAI), politique d'impression (IMP), seuil d'alerte de stock (STK). Une erreur de résolution ici
se manifesterait comme une erreur fiscale trois cycles plus tard.

**Independent Test**: sur une matrice de cas couvrant chaque combinaison de niveaux définis et
absents, vérifier la valeur résolue **et** le niveau dont elle provient. Y compris le cas où aucun
niveau ne définit la valeur, et celui d'un établissement qui n'a ni service ni point de vente.

**Acceptance Scenarios**:

1. **Given** un paramètre défini au seul niveau tenant, **When** on le résout pour un point de
   vente, **Then** la valeur du tenant s'applique et son origine est signalée comme héritée du
   tenant.
2. **Given** un paramètre défini au tenant **et** surchargé au point de vente, **When** on le
   résout pour ce point de vente, **Then** la valeur du point de vente s'applique ; résolu pour un
   autre point de vente du même établissement, la valeur du tenant s'applique.
3. **Given** un paramètre défini au tenant et au point de vente mais **ni** à l'établissement
   **ni** au service — une surcharge partielle, **When** on le résout, **Then** la valeur du point
   de vente s'applique, et les deux niveaux absents ne provoquent ni erreur ni valeur intermédiaire
   inventée.
4. **Given** un paramètre défini à **aucun** niveau, **When** on le résout, **Then** le résultat
   est une **absence explicite** — jamais une valeur par défaut codée en dur, jamais une valeur
   vide ambiguë. L'appelant décide quoi en faire.
5. **Given** un établissement sans aucun service ni point de vente — la résidence meublée à ses
   débuts, **When** on résout un paramètre à son niveau, **Then** la chaîne se réduit à deux
   niveaux et fonctionne.
6. **Given** un paramètre surchargé au niveau d'un service, **When** ce service est désactivé,
   **Then** la surcharge devient inerte sans être supprimée : la résolution remonte au niveau
   supérieur, et la réactivation restitue la surcharge.
7. **Given** deux tenants, **When** l'un résout un paramètre, **Then** aucune valeur de l'autre
   tenant n'entre dans la résolution, quel que soit le niveau interrogé.
8. **Given** un paramètre qualifié de « paramétrable » dans une story, **When** on cherche sa
   valeur dans le code, **Then** elle n'y est pas : elle vit dans la configuration, et le
   récapitulatif des paramètres de `docs/user-stories-v1.md` la recense.

---

### User Story 5 - Les points de vente sont déclarés, un comptoir en est un (Priority: P2)

Un service peut porter plusieurs points de vente — un restaurant et sa terrasse, un bar de salle
et un bar de piscine. Un point de vente sans tables est un comptoir : c'est la forme normale, pas
un cas dégradé. Un établissement peut n'en avoir aucun.

**Why this priority**: ETB-03, P0, mais placée après les quatre précédentes parce que les points
de vente sont le quatrième niveau de la chaîne de configuration : ils supposent que les trois
premiers tiennent. Leur exploitation réelle — catalogue, commandes, additions — relève de PDV,
tranche T2.

**Independent Test**: créer deux points de vente sur un même service, l'un avec des tables et
l'autre sans, et vérifier que la résolution de configuration descend jusqu'à chacun.

**Acceptance Scenarios**:

1. **Given** un établissement dont le service `RESTAURATION` est actif, **When** on y crée deux
   points de vente, **Then** les deux existent, rattachés au même service, et se distinguent par
   leur nom.
2. **Given** un point de vente **sans aucune table**, **When** on l'ouvre, **Then** c'est un
   **comptoir** — un point de vente valide et complet, non un point de vente incomplet.
3. **Given** un point de vente rattaché à un service, **When** ce service est désactivé, **Then**
   le point de vente disparaît de l'interface avec son service et **aucune donnée n'est supprimée**.
4. **Given** un établissement dont aucun service ne porte de point de vente — la résidence
   meublée, **When** on l'exploite, **Then** aucune opération ne réclame de point de vente.
5. **Given** un point de vente, **When** on tente de le rattacher à un service **non activé** sur
   son établissement, **Then** l'écriture est refusée avec un message qui nomme le service.

---

### User Story 6 - L'identité visuelle est posée et vérifiée avant d'être imprimée (Priority: P2)

Le tenant pose son logo, sa couleur primaire, l'en-tête et le pied de ses documents, ses mentions
légales et ses coordonnées. Un établissement peut les surcharger. Un aperçu immédiat, sur un
document de test, montre le résultat avant qu'un seul document ne soit imprimé.

**Why this priority**: ETB-05, P0. Placée en P2 d'implémentation parce qu'elle ne conditionne
aucune autre story de ce cycle, alors que les quatre premières se conditionnent entre elles.

**Independent Test**: poser une identité visuelle au niveau tenant, la surcharger sur un seul de
ses deux établissements, et constater sur l'aperçu de chacun que le bon jeu s'applique.

**Acceptance Scenarios**:

1. **Given** un tenant, **When** l'Admin éditeur pose logo, couleur primaire, en-tête, pied,
   mentions légales et coordonnées, **Then** l'ensemble s'applique à **tous** ses établissements.
2. **Given** cette identité de tenant, **When** un établissement en surcharge une partie — son
   logo seul, par exemple, **Then** cet établissement affiche son logo et **hérite du reste**.
3. **Given** une identité posée, **When** l'Admin éditeur demande l'aperçu, **Then** un document
   de test s'affiche **immédiatement** avec l'identité résolue, sans enregistrement préalable.
4. **Given** l'aperçu affiché, **When** on le lit, **Then** il porte la mention « **Document non
   fiscal — ne tient pas lieu de facture** » et **ne ressemble en rien** à une facture normalisée.
5. **Given** un aperçu, **When** on le compare en mode clair et en mode sombre, **Then** les deux
   sont vérifiés — l'aperçu d'un document destiné au papier reste lisible dans les deux thèmes de
   l'application.

---

### User Story 7 - Les deux tenants de démonstration portent la configuration réelle (Priority: P3)

Le jeu de données de démonstration reflète deux établissements qui ne se ressemblent pas : Deloria
à Abengourou, cinq services actifs et une gestion de stock simple ; « Résidence Test », un seul
service, aucune capacité. Un rechargement en une commande restitue exactement le même état.

**Why this priority**: ce cycle ajoute ses seeds à la mécanique livrée par TRX-05a. Sans ces deux
tenants, ni le test d'isolation ni la démonstration de fin de tranche n'ont de matière — mais la
mécanique existe déjà, donc l'effort est incrémental.

**Independent Test**: exécuter la commande de rechargement trois fois de suite sur une base non
vierge et constater un état final identique et deux tenants correctement configurés.

**Acceptance Scenarios**:

1. **Given** la mécanique de seeds du cycle 001, **When** on la recharge, **Then** l'établissement
   d'Abengourou porte : classement non classé, commune d'Abengourou, fuseau `Africa/Abidjan`,
   devise `XOF`, ses cinq services actifs et la capacité `STOCK` au profil `SIMPLE`.
2. **Given** la même commande, **When** on la recharge, **Then** « Résidence Test » porte le seul
   service `HEBERGEMENT` et **aucune** capacité.
3. **Given** une base déjà seedée, **When** on recharge une seconde puis une troisième fois,
   **Then** l'état final est identique — aucune ligne dupliquée, aucun troisième établissement.
4. **Given** les seeds, **When** on cherche le service fictif minimal d'ETB-02c, **Then** il n'y
   est pas.

---

### Edge Cases

Cas limites arrêtés ici plutôt que découverts en implémentation. Chacun a une réponse, et cette
réponse est testable.

- **Désactiver un service qui porte des opérations en cours.** La désactivation **ne supprime
  jamais de données**. Un service portant des opérations ouvertes — séjours en cours, additions
  non réglées — ne peut pas être désactivé. Ce contrôle ne peut pas être écrit ici : il exigerait
  qu'un crate du socle interroge une verticale, ce que la hiérarchie des crates interdit. Le
  **point de contrôle est donc posé à vide dans ce cycle**, et chaque verticale s'y branche au
  cycle où elle crée des opérations. Tant qu'aucune verticale n'est branchée, la désactivation est
  libre — ce qui est exact aujourd'hui, puisque rien n'existe encore.
- **Établissement sans aucun service.** Valide. Le cadrage §4.1 écrit « modules d'activité
  activés (0..n) ». L'interface est alors vide de services, sans erreur.
- **Modifier le fuseau horaire après des opérations horodatées.** Autorisé, tracé, et **précédé
  d'un avertissement explicite** : les horodatages enregistrés sont absolus et ne changent pas,
  mais tout regroupement par journée locale — clôtures passées, états de reversement — s'en trouve
  réinterprété.
- **Modifier la devise après une opération financière.** Refusé. Les montants sont des entiers
  d'unité mineure d'une devise donnée ; en changer réécrirait le sens de chaque montant. Le
  contrôle est posé à vide dans ce cycle et branché par le cycle qui crée la première opération
  financière (CAI, tranche T2).
- **Modifier le classement de l'établissement.** Autorisé et tracé — le classement détermine le
  barème de la taxe communale de nuitée (`docs/cadrage-v1.md` §9.6). Le pilote est « non classé
  **à confirmer** » : la modification doit rester possible et laisser une trace datée.
- **Deux établissements d'un même tenant dans des fuseaux différents.** Valide. Le fuseau
  appartient à l'établissement, jamais au serveur.
- **Résoudre un paramètre pour un point de vente d'un autre tenant.** La résolution ne voit rien
  et ne renvoie rien — l'isolation s'applique à chaque niveau de la chaîne, pas seulement au
  premier.
- **Numéro de compte contribuable en double.** Accepté : un même contribuable peut exploiter
  plusieurs établissements. Le format est vérifié, l'unicité n'est pas imposée.
- **Service fictif minimal présent en production.** Impossible : il n'existe que dans le harnais
  de test, et une vérification échoue s'il apparaît dans un jeu de données de démonstration ou de
  production.
- **Écriture d'un référentiel par un tenant.** Refusée : les deux référentiels sont globaux à la
  plateforme et en lecture seule pour les tenants. Leur enrichissement relève de l'éditeur
  (ETB-08, provision).
- **Lecture de la configuration hors connexion.** L'écriture d'un référentiel est de classe C, sa
  lecture en cache est de classe A — avec fraîcheur affichée. Confondre les deux rendrait le
  produit inutilisable hors ligne, ou ouvrirait une écriture de référentiel sur un terminal. **Ce
  cycle pose cette classification au registre ; le cache lui-même et son témoin de fraîcheur sont
  construits par SYN-01/02 et ETB-06** (voir § Out of Scope).

## Requirements *(mandatory)*

### Functional Requirements

#### A. Tenant et établissement — ETB-01

- **FR-001**: Le système DOIT porter un modèle `tenant → etablissement (1..n)` ; un tenant sans
  établissement est valide, un établissement sans tenant ne l'est pas.
- **FR-002**: Un établissement DOIT porter : juridiction, classement, commune, fuseau horaire,
  devise, adresse et numéro de compte contribuable, en plus du nom, du fuseau et de la devise déjà
  livrés au cycle 001.
- **FR-003**: Le classement DOIT accepter les valeurs « étoiles » (avec leur nombre), « non
  classé » et « résidence meublée », et être **modifiable et tracé** — le pilote est déclaré « non
  classé à confirmer ».
- **FR-004**: L'enrichissement de `tenant` et `etablissement` DOIT se faire **exclusivement par
  migration additive**. Aucune migration déjà appliquée n'est modifiée (porte P-02, principe I·b).
- **FR-005**: La devise DOIT être un code ISO 4217 porté par l'établissement, et les montants
  DOIVENT être exprimés en entiers d'unité mineure de cette devise (porte P-10).
- **FR-006**: Le fuseau horaire DOIT être porté par l'établissement, jamais par le serveur, et
  DOIT être **modifiable avec avertissement explicite** sur la réinterprétation des regroupements
  par journée locale.
- **FR-007**: La devise d'un établissement NE DOIT PAS être modifiable après sa première opération
  financière. Le contrôle est **posé à vide dans ce cycle** et branché par le cycle qui crée la
  première opération financière.
- **FR-008**: Le nombre d'établissements d'un tenant NE DOIT avoir **aucun effet tarifaire** ;
  aucun compteur d'établissements à visée de facturation n'est introduit (ETB-01, ADM-03).
- **FR-009**: Le modèle NE DOIT PAS empêcher qu'un utilisateur soit rattaché à **plusieurs
  établissements avec des rôles différents sur chacun**. La table de rattachement et les rôles
  relèvent du cycle CPT ; ce cycle garantit seulement qu'aucune contrainte ne les interdit.
- **FR-010**: Le format du numéro de compte contribuable DOIT être vérifié syntaxiquement ; son
  **unicité NE DOIT PAS être imposée**, un contribuable pouvant exploiter plusieurs établissements.

#### B. Référentiel des modules d'activité — ETB-02

- **FR-011**: Le référentiel des modules d'activité DOIT être une **table**, jamais une énumération
  figée dans le code, et contenir `HEBERGEMENT`, `RESTAURATION`, `BAR`, `PRESSING`,
  `SALLE_REUNION`.
- **FR-012**: L'ajout d'une valeur au référentiel DOIT être une **écriture de configuration, pas
  une migration** (`docs/cadrage-v1.md` §14.3 ; ETB-08, provision).
- **FR-013**: Le référentiel DOIT être **global à la plateforme** et **en lecture seule pour les
  tenants** : son enrichissement relève de l'éditeur. Cette absence d'identifiant de tenant est une
  **exception nommée** à la règle « chaque table porte `tenant_id` », déclarée explicitement pour
  que la porte P-07 ne la rencontre pas en silence — comme l'a été la table `tenant` au cycle 001.
- **FR-014**: Un établissement DOIT pouvoir activer et désactiver chaque service indépendamment,
  depuis l'écran de configuration.
- **FR-015**: La désactivation d'un service NE DOIT **jamais** supprimer de données ; la
  réactivation DOIT restituer l'état antérieur.
- **FR-016**: Un service portant des opérations en cours NE DOIT PAS être désactivable. Le point de
  contrôle DOIT être **posé à vide dans ce cycle** et exposé de sorte que chaque verticale s'y
  branche au cycle où elle crée des opérations — **sans qu'aucun crate du socle ne dépende d'une
  verticale** (porte P-03).
- **FR-017**: L'interface NE DOIT **jamais** montrer un service inactif : ni entrée désactivée, ni
  mention « disponible dans votre offre », ni marqueur masqué. **Absent** (principe VII).
- **FR-018**: Un établissement **sans aucun service actif** DOIT être valide et son interface DOIT
  s'afficher sans erreur.
- **FR-019**: `SALLE_REUNION` DOIT rester une **spécialisation d'hébergement** et NE DOIT créer
  aucune entité nouvelle (`docs/cadrage-v1.md` §4.1, règle 5).
- **FR-020**: Toute activation ou désactivation de service DOIT émettre un événement de journal
  **dans la même transaction**, à charge utile complète et dénormalisée (porte P-05).

#### C. Tests structurels permanents — ETB-02 et ETB-02c

- **FR-021**: Trois parcours structurels DOIVENT être écrits **avant l'implémentation** et exécutés
  en intégration continue **de façon permanente** : (a) établissement au seul service
  `RESTAURATION` — un maquis ; (b) établissement au seul service `HEBERGEMENT` — une résidence
  meublée ; (c) établissement portant un **service fictif minimal ne consommant aucune capacité**.
- **FR-022**: Le parcours (c) DOIT être la **preuve formelle** que le socle ne suppose ni
  hébergement, ni point de vente, ni stock, ni aucune spécificité de verticale.
- **FR-023**: Chaque parcours DOIT déclarer la **liste complète et ordonnée** de ses étapes —
  création, vente comptoir, encaissement, document fiscal, clôture journalière — et, pour chacune,
  si elle est **livrée** ou **due**, avec le cycle qui la doit.
- **FR-024**: Les parcours DOIVENT être **verts dès ce cycle** sur leurs étapes livrées. Aucune
  étape n'est marquée « ignorée » ni laissée en échec.
- **FR-025**: L'intégration continue DOIT **échouer** lorsqu'une étape déclarée due devient
  réalisable sans avoir été branchée aux trois parcours. La détection DOIT s'appuyer sur un
  **marqueur observable** de la disponibilité de l'étape — présence de la table ou du point d'entrée
  qui la porte — et non sur une revue humaine.
- **FR-026**: Conformément au § « Couverture des portes » de la constitution, le harnais DOIT
  **déclarer en tête ce qu'il inspecte et ce qu'il n'inspecte pas**, **compter les étapes réellement
  exercées** et les comparer au total déclaré, et **ne jamais modifier** ce qu'il inspecte.
- **FR-027**: Le service fictif minimal NE DOIT exister que dans le harnais de test. Une
  vérification DOIT échouer s'il apparaît dans un jeu de données de démonstration ou de production.
- **FR-028**: Aucun des trois parcours NE DOIT dépendre d'un autre : chacun crée son établissement,
  l'exploite et s'achève indépendamment.

#### D. Référentiel des capacités — ETB-02b

- **FR-029**: Le référentiel des capacités DOIT être une **table distincte de celle des modules**,
  contenant `STOCK`, `LIVRAISON`, `PRODUCTION`, `COMMERCE_EN_LIGNE`, `FIDELITE`, `DEVIS`,
  `COMPTES_CLIENTS`.
- **FR-030**: Les deux référentiels NE DOIVENT **jamais** être fusionnés ni dérivés l'un de
  l'autre : le module est la **verticale** (ce que fait l'établissement), la capacité est le
  **transverse** (ce dont il a besoin pour le faire).
- **FR-031**: Un module DOIT **déclarer les capacités qu'il consomme**. La déclaration lie un
  service activé sur un établissement à une capacité, et porte son profil.
- **FR-032**: Seule la capacité `STOCK` DOIT être acceptée à l'écriture. Les six autres DOIVENT
  être **refusées explicitement**, avec un message nommant la capacité et indiquant qu'elle n'est
  pas implémentée au MVP. **Aucune ligne n'est écrite** en cas de refus (porte P-06).
- **FR-033**: Le profil de stock DOIT accepter la seule valeur `SIMPLE`. `VALORISE`, `DETAILLE` et
  `AUCUN` DOIVENT être refusés explicitement, avec un message nommant le profil ; le message du
  profil `AUCUN` DOIT indiquer qu'une capacité non consommée **ne se déclare pas**.
- **FR-034**: Le profil DOIT être une propriété de la **déclaration de consommation par un
  service**, jamais du produit : un même tenant pourra un jour exploiter un hôtel en `SIMPLE` et une
  quincaillerie en `DETAILLE`.
- **FR-035**: Un refus de capacité ou de profil NE DOIT **jamais** être silencieux, ni dégradé en
  valeur par défaut, ni consigné en journal sans être signalé à l'appelant.
- **FR-036**: L'interface NE DOIT proposer **aucune** capacité non implémentée. Le refus explicite
  protège les autres chemins d'écriture — interface de programmation, import, jeu de données — et
  **ne remplace pas** la règle « absent, pas grisé ».
- **FR-037**: La désactivation d'un service DOIT rendre ses déclarations de consommation **inertes
  sans les supprimer** ; la réactivation les restitue.

#### E. Points de vente — ETB-03

- **FR-038**: Un point de vente DOIT porter : son établissement, le **service auquel il se
  rattache**, un nom, un ensemble de tables éventuellement vide, une politique d'impression et un
  rattachement de caisse.
- **FR-039**: Un service DOIT pouvoir porter **plusieurs** points de vente — un restaurant et sa
  terrasse.
- **FR-040**: Un point de vente **sans tables** DOIT être un **comptoir** : une forme normale et
  complète, non un cas dégradé ni un état transitoire.
- **FR-041**: Un point de vente NE DOIT PAS pouvoir se rattacher à un service **non activé** sur
  son établissement ; le refus nomme le service.
- **FR-042**: Un établissement DOIT pouvoir n'avoir **aucun** point de vente, et aucune opération
  du socle NE DOIT en réclamer un.
- **FR-043**: La désactivation d'un service DOIT faire disparaître ses points de vente de
  l'interface **sans supprimer aucune donnée**.
- **FR-044**: Le référentiel de tables d'un point de vente DOIT exister comme entité propre, de
  classe hors-ligne C. Son exploitation — plan de salle, ouverture, transfert, division — relève du
  cycle PDV, tranche T2.

#### F. Configuration héritée — ETB-04

- **FR-045**: Le système DOIT résoudre tout paramètre le long de la chaîne **tenant →
  établissement → service → point de vente**, chaque niveau pouvant surcharger le précédent.
- **FR-046**: La résolution DOIT être exposée comme un **trait propre du crate `etablissements`**,
  consommé par tous les modules suivants — jamais réimplémenté ailleurs, jamais atteint par une
  jointure entre schémas de modules (porte P-04).
- **FR-047**: La résolution DOIT renvoyer, avec la valeur, le **niveau dont elle provient**, afin
  que l'interface puisse distinguer une valeur héritée d'une valeur surchargée.
- **FR-048**: Lorsqu'aucun niveau ne définit la valeur, la résolution DOIT renvoyer une **absence
  explicite** — jamais une valeur par défaut codée en dur, jamais une valeur vide ambiguë.
- **FR-049**: Les **surcharges partielles** DOIVENT fonctionner : un paramètre défini au tenant et
  au point de vente, absent de l'établissement et du service, se résout à la valeur du point de
  vente.
- **FR-050**: La résolution DOIT fonctionner sur une **chaîne écourtée** — établissement sans
  service, service sans point de vente — sans erreur ni niveau inventé.
- **FR-051**: Une surcharge portée par un service désactivé DOIT devenir **inerte sans être
  supprimée** ; la résolution remonte alors au niveau supérieur.
- **FR-052**: La résolution DOIT être couverte par une **matrice de tests exhaustive** croisant,
  pour chaque niveau, les états « défini » et « absent », et vérifiant à la fois la valeur et son
  origine.
- **FR-053**: **Aucune valeur d'un autre tenant** NE DOIT entrer dans une résolution, à quelque
  niveau de la chaîne que ce soit (porte P-08).
- **FR-054**: Tout paramètre qualifié de « paramétrable » dans une story DOIT vivre dans cette
  configuration, **jamais en dur dans le code**, et être inscrit au récapitulatif des paramètres de
  `docs/user-stories-v1.md` **dans le même changement** que son implémentation (principe I·c, DoD
  point 9).

#### G. Identité visuelle — ETB-05

- **FR-055**: L'identité visuelle DOIT porter : logo, couleur primaire, en-tête et pied des
  documents imprimés, mentions légales et coordonnées.
- **FR-056**: Elle DOIT être définie **par tenant**, avec **surcharge par établissement**, y
  compris partielle — surcharger le seul logo laisse hériter tout le reste.
- **FR-057**: Un **aperçu immédiat** sur un document de test DOIT être disponible, sans
  enregistrement préalable.
- **FR-058**: Le document de test DOIT porter la mention « **Document non fiscal — ne tient pas
  lieu de facture** » et NE DOIT en aucun cas ressembler à une facture normalisée (principe V).
- **FR-059**: La couleur primaire de l'identité visuelle NE DOIT PAS contourner les jetons de
  design : elle s'applique aux documents produits, **jamais à l'interface de l'application**, dont
  les couleurs restent celles de `docs/design/tokens.md` (porte P-17).

#### H. Écran de configuration — G1

- **FR-060**: Le cycle DOIT livrer l'écran **`G1` « Établissement et modules »**, dérivé du motif
  maquetté `G2` selon `docs/design/derivation.md`, et **aucun autre écran**.
- **FR-061**: `G1` DOIT porter quatre sections : identité de l'établissement (ETB-01), services
  actifs et capacités déclarées (ETB-02, ETB-02b), points de vente (ETB-03), identité visuelle avec
  aperçu (ETB-05).
- **FR-062**: `G1` DOIT être vérifié **en mode clair et en mode sombre** (DoD point 8).
- **FR-063**: Toute chaîne visible DOIT être externalisée en clés **fr et en**, fr par défaut, avec
  parité vérifiée (porte P-16).
- **FR-064**: Les termes utilisateur nouveaux — capacité, point de vente, comptoir, classement,
  numéro de compte contribuable, valeur héritée, valeur surchargée — DOIVENT entrer dans
  `docs/design/lexique.md` **avant** d'être écrits en dur, avec leur formulation française et
  anglaise.
- **FR-065**: L'interface DOIT nommer un module d'activité « **Vos services** » et NE DOIT
  **jamais** employer le mot « tenant » (`docs/design/lexique.md`).
- **FR-066**: Aucun fichier de `docs/design/html/` NE DOIT être copié sous `app/` ; seul
  `docs/design/theme.css` l'est, déjà en place depuis le cycle 001 (porte P-19).

#### I. Conformité transverse

- **FR-067**: Toute table créée par ce cycle DOIT avoir la sécurité au niveau ligne **activée et
  forcée**, avec une politique en lecture **et** en écriture, et un test d'isolation entre deux
  tenants sur chaque point d'entrée (portes P-07 et P-08).
- **FR-068**: Les deux référentiels globaux — modules et capacités — DOIVENT porter une politique
  de **lecture universelle et aucun droit d'écriture pour le rôle applicatif**, cette exception
  étant nommée dans la migration qui les crée.
- **FR-069**: Toutes les entités de ce cycle DOIVENT être déclarées de classe hors-ligne **C** dans
  `docs/registre-classes-offline.md` §5.1, et un test DOIT **échouer si l'une de leurs écritures
  est atteignable depuis un chemin de code exécutable hors ligne** (porte P-13, §0.7).
- **FR-070**: La distinction entre **l'écriture d'un référentiel ou d'un paramètre (classe C)** et
  sa **lecture en cache (classe A)** DOIT être écrite au registre des classes hors-ligne, et aucun
  chemin d'écriture de ce cycle NE DOIT être atteignable hors connexion. **La mise en cache
  elle-même et l'affichage de fraîcheur relèvent de SYN-01/02 et d'ETB-06 — hors périmètre.** Ce
  cycle pose la classification que ces cycles consommeront, jamais leur mécanisme : le promettre
  ici en ferait une exigence que rien n'implémente.
- **FR-071**: Toute transition d'état de ce cycle — création d'établissement, activation ou
  désactivation de service, déclaration de capacité, création ou modification de point de vente,
  écriture d'un paramètre, changement d'identité visuelle — DOIT émettre un événement de journal
  **dans la même transaction** (porte P-05).
- **FR-072**: Le changement de **classement** et le changement de **fuseau horaire** DOIVENT être
  tracés de façon **durable et immuable** par un type d'événement propre : le premier détermine le
  barème de la taxe communale de nuitée, le second réinterprète tout regroupement par journée
  locale. **La consultation de cette trace relève du journal d'audit (CPT-04), hors périmètre** —
  l'événement s'écrit ici, l'écran qui le donne à lire vient au cycle CPT.
- **FR-073**: Les annotations du contrat d'API DOIVENT être à jour et le client TypeScript
  régénéré **sans diff manuel** (porte P-01, DoD point 2).
- **FR-074**: Les requêtes DOIVENT être vérifiées à la compilation et le cache de requêtes
  complet — **toutes** les requêtes du dépôt, pas un sous-ensemble (porte P-18, leçon du cycle 1).

#### J. Jeux de données de démonstration

- **FR-075**: Le cycle DOIT ajouter ses seeds à la mécanique rejouable livrée par TRX-05a, dans la
  même tâche que ses migrations, sans en modifier la mécanique.
- **FR-076**: L'établissement d'Abengourou DOIT porter : classement non classé, commune
  d'Abengourou, fuseau `Africa/Abidjan`, devise `XOF`, ses **cinq services actifs** et la capacité
  `STOCK` au profil `SIMPLE`.
- **FR-077**: L'établissement « Résidence Test » DOIT porter le **seul service `HEBERGEMENT`** et
  **aucune capacité**. Ses quatre unités relèvent du cycle HEB et de la tâche de recollement
  TRX-05b.
- **FR-078**: Trois rechargements successifs DOIVENT produire un **état final identique**.

#### K. Refus de périmètre

- **FR-079**: Le cycle NE DOIT livrer **aucune** interface ni **aucune** logique pour ETB-07
  (partenaires externes) et ETB-08 (modules et capacités additionnels). Ces provisions ne sont
  **pas non plus** créées en table par ce cycle — voir § Out of Scope.
- **FR-080**: Le cycle NE DOIT livrer **aucun** sélecteur de contexte (ETB-06, P1), ni bascule
  d'établissement, ni indicateur de synchronisation permanent.

### Key Entities

- **Tenant** — le client de la plateforme : un propriétaire ou un groupe. Porte la configuration
  fiscale, l'identité visuelle, l'abonnement et les utilisateurs. Existe en forme minimale depuis
  le cycle 001. Classe **C**. Seule table du produit dont la colonne d'isolation est sa propre clé.
- **Établissement** — **l'entité centrale du produit, jamais « hôtel »**. Porte juridiction,
  classement, commune, fuseau horaire, devise, adresse et numéro de compte contribuable. Un tenant
  en a de un à n ; leur nombre est sans effet tarifaire. Classe **C**.
- **Module d'activité** — référentiel **global** de la **verticale** : ce que l'établissement fait.
  `HEBERGEMENT`, `RESTAURATION`, `BAR`, `PRESSING`, `SALLE_REUNION`. En table, extensible par
  configuration. Lecture seule pour les tenants. Classe **C**.
- **Activation de service (`etablissement_module`)** — lien entre un établissement et un module,
  avec son état. Réversible, jamais destructif. Classe **C**.
- **Capacité** — référentiel **global et distinct** du **transverse** : ce dont un service a
  besoin. `STOCK`, `LIVRAISON`, `PRODUCTION`, `COMMERCE_EN_LIGNE`, `FIDELITE`, `DEVIS`,
  `COMPTES_CLIENTS`. Une seule implémentée. Classe **C**.
- **Déclaration de consommation (`module_capacite`)** — lien entre un service activé et une
  capacité, portant le **profil**. Seul le couple `STOCK` / `SIMPLE` est accepté. Classe **C**.
- **Point de vente** — rattaché à un établissement **et à un service**. Porte un nom, des tables
  éventuellement absentes, une politique d'impression et un rattachement de caisse. Sans tables,
  c'est un **comptoir**. Classe **C**.
- **Table de point de vente (`table_pdv`)** — référentiel des tables d'un point de vente. Créée
  ici, exploitée au cycle PDV. Classe **C**.
- **Paramètre de configuration** — valeur portée à l'un des quatre niveaux de la chaîne d'héritage.
  Sa résolution est le composant le plus réutilisé du produit. Classe **C** en écriture, **A** en
  lecture de cache — classification posée ici, cache construit au cycle SYN.
- **Identité visuelle (`branding`)** — logo, couleur primaire, en-tête et pied de documents,
  mentions légales, coordonnées. Par tenant, surchargeable par établissement. Classe **C**.
- **Service fictif minimal** — module d'activité **de test uniquement**, ne consommant aucune
  capacité. Support de la preuve d'agnosticité du socle. **N'existe jamais hors du harnais de
  test.**

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Les **trois** parcours structurels sont verts en intégration continue à la clôture du
  cycle, et le restent à chaque exécution ultérieure. Aucun n'est ignoré, aucun n'est conditionnel.
- **SC-002**: **100 % des étapes** de chacun des trois parcours sont déclarées, comptées et
  attribuées — livrées, ou dues avec leur cycle. Le nombre d'étapes exercées est comparé au total
  déclaré à chaque exécution, et un écart fait échouer la vérification.
- **SC-003**: Une étape déclarée due, rendue réalisable sans être branchée aux trois parcours, fait
  **échouer l'intégration continue** — vérifié par injection volontaire.
- **SC-004**: **100 % des valeurs de capacité et de profil non implémentées** — six capacités et
  trois profils, soit neuf cas — sont refusées, par **tous** les chemins d'écriture existants, et
  chaque refus nomme la valeur refusée. **Zéro écriture** en base sur un refus.
- **SC-005**: **Zéro occurrence** d'un service ou d'une capacité inactif dans l'interface rendue
  d'un établissement, vérifié sur les deux établissements de démonstration — dont l'un n'a qu'un
  seul service sur cinq.
- **SC-006**: **100 % des combinaisons** de la matrice de résolution de configuration — quatre
  niveaux, chacun défini ou absent, chaînes écourtées comprises — sont couvertes par des tests, et
  chaque cas vérifie **la valeur et son niveau d'origine**.
- **SC-007**: **Aucune valeur d'un tenant n'est lisible depuis un autre**, sur **100 % des points
  d'entrée** livrés par ce cycle, y compris par identifiant direct et à chaque niveau de la chaîne
  de configuration.
- **SC-008**: Un exploitant active ou désactive un service et en constate l'effet dans l'interface
  **en moins de 30 secondes**, sans reconnexion et sans rechargement manuel.
- **SC-009**: L'aperçu d'un document de test s'affiche **en moins de 2 secondes** après une
  modification d'identité visuelle, sans enregistrement préalable, et porte la mention de document
  non fiscal.
- **SC-010**: Le rechargement des jeux de démonstration produit un **état final identique** sur
  trois exécutions successives, avec les deux établissements correctement configurés — cinq services
  et une capacité pour l'un, un service et aucune capacité pour l'autre.
- **SC-011**: **Zéro terme utilisateur** livré sans entrée préalable au lexique, et **parité de
  100 %** entre les clés françaises et anglaises.
- **SC-012**: **Zéro entité** de ce cycle absente du registre des classes hors-ligne, et **zéro
  écriture** de classe C atteignable depuis un chemin de code exécutable hors ligne.
- **SC-013**: **100 % des transitions d'état** de ce cycle émettent leur événement de journal dans
  la même transaction, vérifié par la porte existante.
- **SC-014**: L'écran `G1` est vérifié **en mode clair et en mode sombre**, et **aucune couleur ni
  aucun espacement littéral** n'y figure hors des jetons de design.

## Assumptions

Hypothèses retenues faute de précision dans les documents de référence. Chacune est un défaut
raisonnable, révisable en `/speckit-clarify` ou `/speckit-plan`.

1. **Enrichissement additif de `tenant` et `etablissement`** — la migration du cycle 001 l'écrit
   noir sur blanc et la porte P-02 l'impose. Les colonnes d'ETB-01 s'ajoutent par nouvelle
   migration ; le fichier existant n'est pas touché. Les établissements déjà seedés reçoivent des
   valeurs explicites, jamais nulles sur un champ que la fiscalité lira.
2. **Portée du refus des profils de stock** — l'énoncé « seule `STOCK` au profil `SIMPLE` est
   implémentée ; toute autre valeur est refusée » est appliqué **littéralement** : `AUCUN` est
   refusé au même titre que `VALORISE` et `DETAILLE`. Une capacité non consommée **ne se déclare
   pas** ; admettre `AUCUN` créerait deux représentations du même état, dont l'une finirait par
   diverger. Le message de refus d'`AUCUN` est distinct et le dit.
3. **Statut du service fictif minimal** — il n'est **jamais** un enregistrement permanent du
   référentiel. Le harnais de test le crée dans son propre contexte et une vérification échoue s'il
   apparaît dans un jeu de données de démonstration ou de production. Un module fictif permanent
   finirait tôt ou tard proposé à l'activation dans la console éditeur.
4. **Référentiels globaux sans identifiant de tenant** — les deux référentiels sont partagés par
   tous les tenants et enrichis par l'éditeur seul (ETB-08). L'alternative — un référentiel dupliqué
   par tenant — multiplierait les valeurs par le nombre de clients et rendrait impossible l'ajout
   d'une valeur « par configuration, sans migration ». L'exception à la règle « chaque table porte
   `tenant_id` » est **nommée dans la migration**, comme l'a été `tenant`.
5. **Rattachement de caisse d'un point de vente** — la caisse relève de `socle/caisse`, dont aucune
   table n'existe avant le cycle CAI (tranche T2). Le rattachement est donc porté **sans contrainte
   référentielle entre schémas de modules** et résolu par trait exposé, conformément au principe II.
   La contrainte de cohérence est posée au cycle CAI.
6. **Politique d'impression** — enregistrée comme **paramètre de la chaîne de configuration** au
   niveau du point de vente, et non comme colonne. C'est l'esprit exact du principe I·c. Son jeu de
   valeurs est défini par le cycle IMP (tranche T2) ; ce cycle n'en fixe aucune et inscrit le
   paramètre au récapitulatif de `docs/user-stories-v1.md`.
7. **Stockage du logo** — le fichier va au stockage d'objets, consommé par son interface S3
   uniquement (principe II). Seule sa référence est en base. Format et taille maximale sont arrêtés
   en `/speckit-plan`.
8. **Juridiction** — un seul adaptateur existe au MVP (`CoteDIvoire`). Le champ est posé et sa
   valeur unique, afin qu'un second adaptateur soit une écriture de configuration et non une
   migration (`docs/cadrage-v1.md` §14.1).
9. **Services déclarant `STOCK` chez Deloria** — `RESTAURATION` et `BAR`, les deux services qui
   vendent des articles stockés. Le pressing et la salle de réunion ne déclarent rien. Sans
   conséquence observable avant le cycle STK (tranche T5), donc révisable sans coût.
10. **Point 10 de la Definition of Done** — « tout document imprimé vérifié sur imprimante thermique
    réelle » est **sans objet** sur ce cycle : l'aperçu d'ETB-05 est un rendu à l'écran. La
    vérification sur imprimante relève du cycle IMP (tranche T2), où le premier document est
    réellement imprimé. Consigné explicitement plutôt que passé sous silence.
11. **Accueil `R1` et rôles** — le filtrage des tuiles d'accueil par permission dépend du cycle CPT.
    Ce cycle livre la donnée qui permettra le filtrage **par service actif** ; l'écran lui-même ne
    l'est pas.
12. **Traçabilité des changements sensibles** — les changements de classement et de fuseau horaire
    sont tracés par le **journal d'événements** livré au cycle 001, immuable et à rétention
    illimitée. Le journal d'audit consultable par le propriétaire (CPT-04) est un écran distinct, dû
    au cycle CPT ; l'événement, lui, est écrit dès maintenant.

## Dependencies

- **Cycle 001 (`specs/001-socle-technique-monorepo/`)** — livré et fusionné. Fournit : `tenant` et
  `etablissement` en forme minimale, l'isolation multi-tenant, le journal d'événements, la mécanique
  de seeds rejouable, la porte du registre des classes hors-ligne et les portes P-01 à P-20. Ce
  cycle **honore sa dette d'écran** (FR-024 de la spec 001) en livrant `G1`.
- **`docs/registre-classes-offline.md`** — les quatorze entités de `socle/etablissements` y sont déjà
  classées (§5.1). Ce cycle **ajoute la distinction écriture C / lecture de cache A** pour les
  paramètres et les référentiels, et consigne l'ajout au journal des modifications du registre.
- **`docs/design/derivation.md`** — `G1` hérite de `G2`. La maquette `G2-offre-hebergement.html` est
  ouverte au moment de coder l'écran, et respectée. **Jamais copiée.**
- **`docs/design/lexique.md`** — sept termes utilisateur nouveaux y entrent avant d'être codés
  (FR-064). Sans cette entrée, la porte P-16 et la Definition of Done ne sont pas satisfaites.
- **`docs/user-stories-v1.md`, récapitulatif des paramètres** — mis à jour dans le même changement
  pour tout paramètre introduit, la politique d'impression comprise.
- **`docs/versions-gelees.md`** v1.0.2 (2026-07-30) — aucune brique nouvelle n'est introduite par ce
  cycle. Prochaine revue groupée : 2026-08-31.
- **Cycle CPT (suivant)** — porte le rattachement d'un utilisateur à plusieurs établissements avec
  des rôles distincts (FR-009). Ce cycle garantit seulement qu'aucune contrainte ne l'interdit.
- **Cycles PDV, CAI et FIS** — doivent brancher leurs étapes aux trois parcours structurels
  (FR-025). C'est une dépendance **sortante** : ce cycle installe la contrainte, les suivants la
  subissent.

## Out of Scope

Explicitement exclu de ce cycle.

| Exclu | Référence | Raison |
|---|---|---|
| Sélecteur de contexte — barre permanente, bascule d'établissement en deux taps, indicateur de synchronisation | ETB-06, P1 | Suppose des rôles et un poste actif, qui relèvent du cycle CPT |
| Partenaires externes — `partenaire` à `tenant_id` nullable, `demande_partenaire`, comptes et mouvements de compensation | ETB-07, PROVISION | **Tables seulement** au MVP, et **non créées ici** : leur place est le cycle qui livre la première relation externe. Aucune UI, aucune logique |
| Modules et capacités additionnels — `SPA`, `BOULANGERIE`, `SUPERETTE`, `QUINCAILLERIE`, `EXCURSION` ; capacités au-delà de `STOCK` | ETB-08, PROVISION | Les deux référentiels sont **en table**, donc l'ajout est déjà possible par configuration. **Aucune valeur additionnelle n'est écrite, aucun code ne les traite** |
| Profils de stock supérieurs — `VALORISE`, `DETAILLE` | Cadrage §14.5 | Refusés explicitement (FR-033) |
| Exploitation des points de vente — catalogue, tables ouvertes, commandes, additions | PDV, tranche T2 | Ce cycle crée le référentiel, pas son exploitation |
| Exploitation du stock — mouvements, seuils, inventaire | STK, tranche T5 | La capacité se **déclare** ici ; elle ne fait rien avant STK |
| Accueil à tuiles filtrées par permission | `R1`, cycle CPT | Le filtrage par permission suppose des rôles ; livrer l'écran à moitié imposerait de le rouvrir |
| **Mise en cache locale des référentiels et affichage de fraîcheur** | SYN-01/02 ; indicateur permanent en ETB-06 | Ce cycle **classe** l'écriture en C et la lecture en A (FR-070) ; le mécanisme de cache et son témoin de fraîcheur appartiennent au cycle SYN |
| **Consultation du journal des changements sensibles** | CPT-04 | L'événement de changement de classement ou de fuseau est **écrit** ici (FR-072) ; l'écran qui le donne à lire vient au cycle CPT |
| Écran de note interne | Dette du cycle 001 | Cet écran n'hérite d'aucun motif de `docs/design/derivation.md` ; il se maquette avant de se coder |
| Impression réelle d'un document | IMP, tranche T2 | ETB-05 livre un **aperçu à l'écran** ; l'imprimante thermique est vérifiée au cycle IMP |
| Convention inter-établissements | Cadrage §4.3, §14.9 | Provision, tables uniquement, hors de ce cycle |
| Calcul de palier d'abonnement et métrique `unite_facturable` | ADM-03, tranche T5 | Ce cycle garantit seulement l'absence d'effet tarifaire du nombre d'établissements (FR-008) |
