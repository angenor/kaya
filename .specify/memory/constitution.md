<!--
SYNC IMPACT REPORT — 1.8.0 — 2026-08-02
========================================
Changement de version : 1.7.1 → 1.8.0 (MINOR)
Motif : une porte ajoutée (P-23). Jeu porté à 26 portes. Aucun principe ajouté,
        renommé ni supprimé.
Origine : plan du cycle 005 (SYN — file hors-ligne et horodatage d'autorité).

  P-23 — PROVENANCE DE L'INSTANT. Le principe IV exige déjà en toutes lettres que
  « toute logique métier, tout calcul fiscal, toute clôture et TOUT CALCUL DE DURÉE
  DE PASSAGE s'appuient exclusivement sur l'horodatage d'autorité serveur, jamais
  sur l'horloge d'un terminal ». AUCUNE des vingt-cinq portes ne le gardait : P-09
  vérifie que les occupations sont des intervalles protégés par une contrainte
  d'exclusion, pas la PROVENANCE d'un instant. La colonne `horodatage_client` existe
  sur quatre tables depuis les cycles précédents, et rien n'empêchait un calcul de
  s'y appuyer.

  Le moment est celui qui coûte le moins. SEJ et FIS écriront les premières règles de
  durée de passage et de taxe de nuitée — exactement les calculs que le principe IV
  vise. Poser la porte maintenant coûte un test ; la poser après coûte la revue de
  deux moteurs déjà écrits. Le cadrage §11.4 en donne la raison : « un téléphone
  d'entrée de gamme dérive et le personnel change l'heure », et « le passage aggrave
  la sensibilité à l'horloge » puisqu'il se facture à l'heure.

  Périmètre DÉCOUVERT, jamais énuméré — l'énumération à la main a produit un trou à
  chacun des quatre cycles précédents. Trois exemptions limitativement énumérées dans
  le script : ordre d'affichage local, détection de dérive, rendu de l'instant perçu.

SYNC IMPACT REPORT — 1.7.1 — 2026-08-02
========================================
Changement de version : 1.7.0 → 1.7.1 (PATCH)
Motif : réparation d'une écriture manquée, pas une décision nouvelle. L'intention
        était déjà versionnée en 1.7.0 ; seul son inscription au corps manquait.

  1. L'exigence 6 était ANNONCÉE par le rapport de la v1.7.0 et ABSENTE du corps :
     la section n'en listait que cinq. Le remplacement avait échoué en silence et la
     vérification comptait les occurrences dans tout le fichier au lieu de la seule
     section — le défaut même que la section reproche aux portes, commis sur elle.
     La règle était donc annoncée sans être opposable.

  2. L'intitulé « Trois exigences en découlent » était périmé depuis la v1.3.0, qui
     en avait ajouté deux. Remplacé par une formulation qui ne se périme plus.

  3. L'exigence 6 est REFORMULÉE PLUS DUREMENT que dans le rapport de la v1.7.0.
     Celui-ci affirmait qu'initialiserTheme() était « couvert par ses tests, et
     appelé nulle part ». C'était faux : theme-sombre.spec.ts n'importe pas
     core/theme — la fonction n'était NI testée NI branchée. La règle juste porte
     donc sur les deux versants, prouvés séparément. Sur onze fonctions d'amorçage,
     cinq n'étaient appelées nulle part.

SYNC IMPACT REPORT — 1.7.0 — 2026-08-01
========================================
Changement de version : 1.6.0 → 1.7.0 (MINOR)
Motif : une porte ajoutée (P-22) et une exigence de couverture (6). Jeu porté à
        25 portes. Aucun principe ajouté, renommé ni supprimé.
Origine : vérification EN NAVIGATEUR du cycle 003, après livraison de 64 tâches sur
        64, 24 portes vertes, 224 tests backend et 428 tests front.

  Constat : DEUX DES QUATRE ÉCRANS DU PRODUIT — G3 et G4 — sont inatteignables en
  navigateur. Trois défauts qu'aucune porte ni aucun test ne voyait :
    · une TypeError sur `parentNode` vide le <main> à la navigation vers une page
      paresseuse — reproduite sur /etablissement, donc antérieure au cycle 003 ;
    · un chargement direct d'adresse ne reprend jamais la session ;
    · la classe .dark n'est jamais appliquée.

  La cause est UNIQUE et architecturale : `app/app.vue` ne contient que <NuxtPage />,
  il n'existe ni `app/plugins/` ni `app/layouts/`, donc aucun point d'amorçage de
  l'application. `pages/index.vue` amorce pour lui seul ; les cinq autres pages n'ont
  rien. Le patron front documenté au cycle 002 couvrait l'écriture, l'appel typé, les
  erreurs et les permissions — pas le cycle de vie de l'application.

  Les 428 tests ne pouvaient pas le voir : ils montent les écrans avec
  @vue/test-utils, ce qui contourne le routeur, <Suspense>, les layouts et les
  plugins. P-22 rend enfin opposable le point 8 de la Definition of Done — « écran
  vérifié en mode clair et en mode sombre » — qui n'a été coché pour AUCUNE story
  depuis le début du projet, faute d'être vérifiable autrement qu'à la main.

SYNC IMPACT REPORT — 1.6.0 — 2026-08-01
========================================
Changement de version : 1.5.0 → 1.6.0 (MINOR)
Motif : deux portes existantes matériellement étendues — extension de guidance → MINOR.
        Le jeu reste à 24 portes ; aucun principe ajouté, renommé ni supprimé.
Origine : plan du cycle 003 (CPT — comptes, rôles, journal d'audit).

  1. P-05b — portée élargie de l'outbox à TOUT REGISTRE IMMUABLE. CPT-04 exige un
     « traçage immuable » du journal d'audit, et le registre des classes hors-ligne
     §5.2 pose que l'audit est un registre DISTINCT de l'outbox : deux registres,
     deux publics, une action tracée produit les deux. Le journal d'audit — « ce que
     le propriétaire achète » selon le cadrage §8.3 — n'était donc protégé par
     aucune porte contre la suppression ou la réécriture. La porte est reformulée
     sur la CATÉGORIE et non sur une table nommée, pour que le prochain registre
     soit couvert sans nouvel amendement.

  2. P-10 — étendue aux montants portés par du JSONB. `journal_audit.contexte` est
     en JSONB et accueillera des montants : écart de caisse, modification de tarif,
     remise. La porte n'inspectait que les colonnes SQL ; une valeur monétaire dans
     un document JSON échappait entièrement au principe V, qui impose des entiers en
     unités mineures. La garantie s'arrêtait donc à la frontière du JSONB, sur le
     registre même qui trace les écarts d'argent. Convention imposée et vérifiable :
     nommage réservé pour les clés monétaires, valeur entière, jamais un décimal ni
     une chaîne formatée.

SYNC IMPACT REPORT — 1.5.0 — 2026-07-31
========================================
Changement de version : 1.4.0 → 1.5.0 (MINOR)
Motif : deux portes existantes matériellement étendues — extension de guidance → MINOR.
        Le jeu reste à 24 portes ; aucun principe ajouté, renommé ni supprimé.
Origine : lot de consolidation du 2026-07-31 (polices, styleguide, licences, lint).

  1. P-15 — périmètre élargi de `app/` à `app/` ET `web/`. La porte était aveugle sur
     les surfaces publiques faute de configuration ESLint sur cet arbre. Or `web/qr` et
     `web/console` sont HORS Tauri : elles ne doivent jamais importer `@tauri-apps/api`,
     ce qui y rend la porte plus critique qu'ailleurs. Corrigé avant que les cycles QRC
     et ADM n'y créent leurs premiers composants — au coût de quelques lignes plutôt
     que d'une revue de composants.

  2. P-21b — cinquième contrôle : toute police embarquée porte sa licence et son avis
     de copyright, atteignables depuis le produit. Les quatre woff2 d'Archivo et Chivo
     Mono sont redistribués dans un binaire vendu par abonnement, et leur cmap est
     modifiée (ajout de U+202F, absent des polices amont). La clause 2 de l'OFL 1.1
     impose l'avis de copyright et la licence à toute redistribution. Vérifié : aucune
     des deux polices ne déclare de Reserved Font Name, donc conserver les noms de
     familles après modification est licite — c'est l'attribution qui manquait, pas le
     droit de modifier.
     Limite assumée et écrite dans la porte : elle vérifie la PRÉSENCE d'une licence,
     pas la conformité de son texte à l'amont. Le faire exigerait `node_modules` et lui
     ferait perdre son autonomie, donc sa capacité à tourner sur un changement de
     documentation seul.

Note de méthode : les amendements 1.1.0 à 1.4.0 ont été écrits directement dans le
fichier, alors que la clause « Amendement » impose de passer par /speckit-constitution.
Le rapport d'impact et le versionnement ont chaque fois été produits, donc l'esprit de la
règle a été tenu, mais pas sa lettre. Le présent amendement rétablit la procédure.

SYNC IMPACT REPORT — 1.4.0 — 2026-07-31
========================================
Changement de version : 1.3.0 → 1.4.0 (MINOR)
Motif : P-21b ajoutée et le corollaire « toute interdiction a un versant positif »
        inscrit en couverture des portes — extension matérielle → MINOR. 24 portes.
Origine : solde des dettes du cycle 002. P-21 interdisait les ressources externes
        mais ne vérifiait pas que le contenu local existe. Conséquence constatée
        deux fois : au cycle 002, retirer le CDN d'icônes laissait un écran sans
        icônes avec P-21 verte ; au volet suivant, Archivo et Chivo Mono ne sont
        toujours pas embarquées et P-21 passe — l'application tourne sur les polices
        système de repli. Or `docs/design/theme.css` prescrit explicitement de les
        servir en local (woff2, font-display: swap) « le produit tourne sur des
        liaisons lentes et doit s'afficher hors ligne », et `tokens.md` §2 fait de
        Chivo Mono tabulaire la condition de l'alignement des colonnes de montants :
        sans elle, un écran de caisse affiche des montants désalignés.
        Le 4e contrôle inventé pour les glyphes est la bonne idée ; P-21b la
        généralise à tout ce qui est déclaré.
Modifications :
  - Portes : P-21b ajoutée.
  - § Couverture des portes : corollaire du versant positif ajouté à l'exigence 4.
Aucun principe ajouté, renommé ni supprimé.

SYNC IMPACT REPORT — 1.3.0 — 2026-07-31
========================================
Changement de version : 1.2.1 → 1.3.0 (MINOR)
Motif : deux portes ajoutées (P-01b, P-21) et deux exigences de couverture (4, 5) —
        extension matérielle de la gouvernance → MINOR. Jeu porté à 23 portes.
Origine : livraison du cycle 002 (ETB). Trois défauts qu'aucune porte ne couvrait :
  1. Deux operationId homonymes produisaient un client TypeScript INVALIDE. P-01 ne
     compare que le généré au commité : un contrat cassé passe si le commit l'est
     aussi. → P-01b, unicité des operationId.
  2. La maquette charge sa police d'icônes depuis un CDN. Reprise telle quelle, elle
     rend l'écran dépendant du réseau — ce que le principe VI interdit. Le principe
     XII disait de réimplémenter le HTML, sans rien dire des ressources qu'il
     charge. → P-21, aucune ressource d'hôte externe.
  3. ESLint ne parsait aucun .vue typé : P-15 était verte car sa cible était vide,
     masquée par l'unique composant non annoté du cycle 001. → exigence 4, prouver
     qu'une porte a une cible non vide.
  Et le défaut de séquence de l'outbox (espace de numérotation partagé entre
  tenants, migration 0012), trouvé par le premier événement de portée tenant sur un
  second tenant — ni par relecture, ni par une porte. → exigence 5.
Modifications :
  - Portes : P-01b et P-21 ajoutées.
  - § Couverture des portes : exigences 4 et 5 ajoutées, constats du cycle 002.
Aucun principe ajouté, renommé ni supprimé.

SYNC IMPACT REPORT — 1.2.1 — 2026-07-31
========================================
Changement de version : 1.2.0 → 1.2.1 (PATCH)
Motif : le principe III exigeait « chaque table porte tenant_id » sans nommer
        l'exception des référentiels globaux (catalogues des modules d'activité et
        des capacités, communs à tous les tenants). La spécification du cycle 002 a
        dû trancher seule ; sans mention, chaque cycle la redécouvrira et la
        constitution restera en défaut. Clarification d'une règle existante → PATCH.
Modifications :
  - Principe III : exception des référentiels globaux nommée, avec l'obligation de
    la déclarer dans la migration pour que P-07 ne la rencontre pas en silence.
Aucun principe ajouté, renommé ni supprimé. Le jeu reste à 21 portes.

SYNC IMPACT REPORT — 1.2.0 — 2026-07-31
========================================
Changement de version : 1.1.0 → 1.2.0 (MINOR)
Motif : nouvelle sous-section « Couverture des portes », qui ajoute trois exigences
        applicables à toute porte — extension matérielle de la gouvernance → MINOR.
Origine : revue du cycle 1 (TRX) après implémentation. Quatre portes vertes se sont
        révélées défectueuses, AUCUNE trouvée par relecture : P-08 lisait un contrat
        vide, P-18 validait 43 requêtes sur 47, et le mode vérification de P-18
        écrasait le cache qu'il inspectait. Les portes savaient échouer ; elles ne
        regardaient pas tout.
Modifications :
  - § Portes de conformité : sous-section « Couverture des portes » ajoutée
    (déclarer le périmètre, vérifier la complétude, ne pas modifier l'inspecté).
Aucun principe ajouté, renommé ni supprimé. Le jeu reste à 21 portes.

SYNC IMPACT REPORT — 1.1.0 — 2026-07-30
========================================
Changement de version : 1.0.2 → 1.1.0 (MINOR)
Motif : ajout d'une porte de CI (P-05b) et précision du mécanisme d'isolation du
        principe III. L'ajout d'une porte est une extension matérielle de la
        gouvernance → MINOR, pas PATCH.
Origine : revue du plan du cycle 1 (TRX). Deux manques relevés par le plan :
  1. TRX-02 exige la rétention illimitée de l'outbox, mais AUCUNE porte ne
     l'imposait — P-01 à P-20 vérifiaient l'écriture de l'événement (P-05), jamais
     l'absence de chemin de suppression. Ajout de P-05b.
  2. Le principe III prescrivait littéralement « SET LOCAL app.current_tenant ».
     SET LOCAL est une commande utilitaire qui n'accepte aucun paramètre lié : la
     suivre imposerait d'interpoler l'UUID du tenant dans la chaîne SQL, donc
     d'employer AssertSqlSafe (sqlx 0.9) sur le chemin de code exact qui décide
     quelles lignes un client voit. Le principe impose désormais
     SELECT set_config('app.current_tenant', $1, true) — une fonction, donc un
     argument lié et une requête littérale vérifiable par query!.
Modifications :
  - Principe III : mécanisme d'isolation précisé et motivé.
  - Portes de CI : P-05b ajoutée. Le jeu compte désormais 21 portes.
Aucun principe ajouté, renommé ni supprimé.

SYNC IMPACT REPORT — 1.0.2 — 2026-07-30
========================================
Changement de version : 1.0.1 → 1.0.2 (PATCH)
Motif : les deux derniers TODO sont fermés sur constat factuel, et le gel des versions
        du principe XI est matérialisé. Aucune règle modifiée → PATCH.
Modifications :
  - TODO(DECOMPTE_MAQUETTES) fermé. Inventaire vérifié : docs/design/html/ contient
    11 codes d'écran (C4, F2, G2, M4, P2, Q1, R1, R4, R7, S2, V1) répartis en
    29 fichiers d'états. F2-registre-grave.html et S2-registre-grave.html ne sont PAS
    un doublon — ce sont deux écrans distincts partageant un suffixe de nom : F2 traite
    le document fiscal INDETERMINEE (FIS-05), S2 la consommation orpheline (SYN-03).
    L'observation de doublon portée en 1.0.0 était erronée.
  - TODO(PRECEDENCE_TOKENS) fermé. Comparaison exhaustive des 71 tokens tabulés de
    tokens.md avec les 104 déclarations du bloc @theme de theme.css : aucune divergence
    de valeur, aucun token manquant. Les seuls écarts sont de notation (« 14,5 px » vs
    « 14.5px », « / .85 » vs « / 0.85 »). La règle de préséance reste en vigueur comme
    filet, sans objet aujourd'hui.
  - Ajout de docs/versions-gelees.md au tableau des artefacts de gouvernance
    (principe XI matérialisé, gel 1.0.0 au 2026-07-30).
Aucun principe ajouté, renommé ni supprimé.

SYNC IMPACT REPORT — 1.0.1 — 2026-07-30
========================================
Changement de version : 1.0.0 → 1.0.1 (PATCH)
Motif : docs/registre-classes-offline.md a été créé. Le principe VI le désignait déjà
        comme source de vérité ; l'artefact existe désormais. Lever le TODO est une
        clarification sans effet sémantique sur aucun principe → PATCH.
Modifications :
  - § Éléments différés : TODO(REGISTRE_CLASSES_OFFLINE) levé, remplacé par un renvoi
    aux trois décisions ouvertes que le registre consigne (O-01 client/personne en C,
    O-02 classe de mouvement_stock, O-03 crate d'accueil de la surface QR).
  - Principe VI : aucune modification de règle ; le fichier cité existe maintenant.
Aucun principe ajouté, renommé ni supprimé.

SYNC IMPACT REPORT — 1.0.0 — 2026-07-30
========================================
Changement de version : (aucune, gabarit vierge) → 1.0.0
Motif : première ratification. Le fichier ne contenait que les jetons de gabarit
        (`[PROJECT_NAME]`, `[PRINCIPLE_N_NAME]`…). Passage à un contenu concret =
        MINOR/MAJOR sans objet ; on ouvre la série en 1.0.0.

Principes définis (12, aucun renommage puisqu'aucun n'existait) :
  I.    Sources de vérité
  II.   Architecture modulaire et hiérarchie des crates
  III.  Isolation multi-tenant
  IV.   Temps et disponibilité
  V.    Argent et fiscalité
  VI.   Hors-ligne et résilience réseau
  VII.  Application unique et rôles cumulés
  VIII. Qualité, i18n et observabilité
  IX.   Sécurité
  X.    Périmètre — « prêt ≠ construit »
  XI.   Versions épinglées
  XII.  Référence visuelle

Sections ajoutées :
  - « Contraintes techniques et documents de référence » (remplace [SECTION_2_NAME])
  - « Portes de conformité et flux de développement » (remplace [SECTION_3_NAME])
  - « Governance »

Sections supprimées : aucune.

TODO reportés : les deux TODO ouverts en 1.0.0 sont fermés en 1.0.2 (voir ci-dessus).
Restent trois décisions ouvertes portées par docs/registre-classes-offline.md (O-01,
O-02, O-03) et les décisions B-01 à B-09 de l'annexe B du cadrage.
-->

# Constitution Kaya

Kaya est une plateforme de gestion pour **établissements d'hébergement et de service** en
Afrique. Établissement pilote : Résidence Hôtel Deloria, Abengourou, Côte d'Ivoire.
Développement solo, monorepo unique.

**L'entité centrale est l'établissement, pas l'hôtel.** Un établissement active les modules
d'activité dont il a besoin (hébergement, restauration, bar, pressing, salle de réunion). Un
maquis seul, un bar seul, un pressing seul et une résidence meublée seule sont des
établissements valides. Aucun crate partagé NE DOIT supposer qu'un établissement possède de
l'hébergement, ni qu'il possède un point de vente.

Documents produit de référence : `docs/cadrage-v1.md` et `docs/user-stories-v1.md`. **En cas
de doute, ces documents priment sur toute supposition.**

## Core Principles

### I. Sources de vérité

Trois sources de vérité sont uniques, générées ou versionnées, et jamais dupliquées à la main.

- **(a) Contrat d'API.** Le contrat OpenAPI est **généré par utoipa depuis le code Actix** ; il
  n'est jamais écrit à la main. Le client TypeScript est **généré depuis ce contrat en CI**.
  Un diff de client non commité FAIT ÉCHOUER LE BUILD. Toute story qui touche l'API met
  d'abord à jour les annotations utoipa, puis régénère le client.
- **(b) Schéma de base.** Le schéma PostgreSQL n'est modifié QUE par **migrations sqlx
  versionnées**. Une migration appliquée N'EST JAMAIS modifiée — on en crée une nouvelle. Les
  seeds sont **rejouables** et vivent à part des migrations.
- **(c) Paramètres métier.** Tout paramètre qualifié de « paramétrable » vit dans la
  **configuration d'établissement**, avec héritage `tenant → établissement → module → point de
  vente` et surcharge à chaque niveau. Jamais en dur dans le code. Le **récapitulatif des
  paramètres en fin de `docs/user-stories-v1.md` fait foi** ; une nouvelle option paramétrable
  y est ajoutée dans le même changement que son implémentation.

*Rationale* : une seconde copie écrite à la main d'un contrat, d'un schéma ou d'un barème
dérive silencieusement. Chaque source de vérité a exactement un producteur.

### II. Architecture modulaire et hiérarchie des crates

Monolithe modulaire Rust, **microservices-ready** — ce qui signifie exactement ceci, et rien de
plus :

- **Un crate par domaine**, interfaces exposées **par traits**, dépendances injectées.
- **Un schéma PostgreSQL par module.** AUCUNE requête ne joint deux schémas de modules
  différents ; les lectures inter-modules passent par un trait exposé.
- **AUCUNE transaction SQL ne couvre deux modules.** Les opérations inter-modules sont des
  **sagas simples avec compensation explicite**.
- **Toute transition d'état écrit un événement outbox dans la même transaction.** L'outbox est
  un **grand livre permanent, pas une file de messages** : rétention illimitée, charge utile
  financière complète et dénormalisée, immuable (une correction est un nouvel événement).
- AUCUN service n'est extrait au MVP. AUCUNE file de messages externe n'est introduite —
  l'outbox est consommé par un **worker in-process**.

**Trois familles de crates, hiérarchie de dépendance stricte :**

| Famille | Contenu | Peut dépendre de |
|---|---|---|
| `socle/` | etablissements, comptes, caisse, fiscalite, documents, synchronisation, pilotage, editeur, metriques | `socle/` uniquement |
| `capacites/` | stocks — *(les autres capacités ne sont pas implémentées)* | `socle/` |
| `verticales/` | hebergement, restauration, bar, pressing | `socle/`, `capacites/` |

**UN TEST DE CI ÉCHOUE SI UN CRATE DE `socle/` DÉPEND D'UN CRATE DE `verticales/`.**

**LE SOCLE NE CONNAÎT NI « chambre », NI « unité louable », NI « séjour ».** Il connaît
`article_vendable` et `ressource_reservable`. Tout le spécifique hôtelier vit dans
`verticales/hebergement`. `SALLE_REUNION` est une spécialisation d'hébergement et ne crée
aucune entité nouvelle.

**MODULE D'ACTIVITÉ ≠ CAPACITÉ** — deux référentiels distincts, tous deux en table :

- **Module** (la verticale, ce que fait l'établissement) : `HEBERGEMENT`, `RESTAURATION`,
  `BAR`, `PRESSING`, `SALLE_REUNION`.
- **Capacité** (le transverse, ce dont il a besoin pour le faire) : `STOCK`, `LIVRAISON`,
  `PRODUCTION`, `COMMERCE_EN_LIGNE`, `FIDELITE`, `DEVIS`, `COMPTES_CLIENTS`.
- Un module **déclare les capacités qu'il consomme**. Seule `STOCK` au profil `SIMPLE` est
  implémentée ; **toute autre valeur est REFUSÉE EXPLICITEMENT, jamais ignorée**.

Le crate `domain` (moteur fiscal, barèmes, validation, types métier) est partagé entre l'API,
le nœud de site et la coquille Tauri : **UNE SEULE implémentation du calcul de la taxe de
nuitée**, pas trois.

**Rôles des dépôts de données**, sans exception : PostgreSQL est la **seule vérité durable** ;
Redis ne porte que de l'**éphémère reconstructible** (sessions, file FNE, verrous distribués,
limitation de débit, cache de catalogue) ; Garage est consommé **via l'API S3** uniquement.

*Rationale* : c'est cette hiérarchie qui garde le produit extensible à d'autres activités.
Sans elle, l'hôtellerie contamine le noyau en trois cycles et l'extension devient une
réécriture.

### III. Isolation multi-tenant

- Chaque table porte `tenant_id`. **Seule exception : les référentiels globaux** — le catalogue
  des modules d'activité et celui des capacités, communs à tous les tenants et en lecture seule
  pour eux. Toute table sans `tenant_id` DOIT être **nommée explicitement dans sa migration**
  comme exception, avec son motif ; la porte P-07 ne doit jamais en rencontrer une en silence.
  Une exception non déclarée est un défaut, pas une tolérance.
- **RLS `ENABLE` ET `FORCE`** sur toutes les tables, avec un **rôle applicatif distinct du
  propriétaire des tables**.
- Le tenant courant est posé **DANS CHAQUE TRANSACTION**, jamais à l'ouverture de connexion.
- **Le mécanisme est `SELECT set_config('app.current_tenant', $1, true)`, jamais
  `SET LOCAL app.current_tenant = ...`.** `SET LOCAL` est une commande utilitaire qui n'accepte
  aucun paramètre lié : l'employer imposerait d'interpoler l'identifiant du tenant dans la
  chaîne SQL, donc de recourir à `AssertSqlSafe`, donc de placer une concaténation SQL sur le
  chemin de code exact qui décide quelles lignes un client voit. `set_config` est une fonction :
  l'argument se lie, la requête reste littérale, et `query!` la vérifie à la compilation. Le
  troisième argument `true` donne la portée transactionnelle de `SET LOCAL`.
- **Un test de CI échoue si une table du schéma n'a aucune politique RLS.**
- **Un test d'isolation vérifie, sur chaque endpoint, que le tenant A ne lit ni n'écrit aucune
  ligne du tenant B.**

*Rationale* : avec un pool de connexions, `SET LOCAL` par transaction contre `SET` à
l'ouverture est la différence exacte entre l'isolation et la fuite de données entre clients.

### IV. Temps et disponibilité

- Une occupation est un **intervalle `[début, fin)` en timestamp avec fuseau horaire de
  l'établissement**, JAMAIS une paire de dates. Le marché pratique massivement le **passage
  horaire** et la **demi-journée**.
- La disponibilité est garantie par une **contrainte d'exclusion PostgreSQL**
  (`EXCLUDE USING gist` sur `unite_id` + `tstzrange`), **pas par un verrou applicatif**.
- Le **temps de remise en état est intégré à l'intervalle d'indisponibilité**, pas géré à part.
- Le statut d'unité « occupée » / « réservée » est **dérivé** des occupations, jamais posé à la
  main. Seul le sous-statut ménage est librement modifiable.
- Toute logique métier, tout calcul fiscal, toute clôture et **TOUT CALCUL DE DURÉE DE
  PASSAGE** s'appuient **exclusivement sur l'horodatage d'autorité serveur**, jamais sur
  l'horloge d'un terminal. L'horodatage client est indicatif (ordre d'affichage local) et une
  dérive au-delà du seuil paramétré déclenche une alerte.

*Rationale* : ce choix est structurant et irréversible. Modéliser en dates fermerait la porte
au passage et à la demi-journée, qui sont le différenciateur du produit. Une contrainte de base
rend la double attribution impossible, là où un verrou applicatif la rend seulement improbable.

### V. Argent et fiscalité

- **MONTANTS : entiers en unités mineures** + code **ISO 4217** porté par l'établissement
  (XOF, 0 décimale).
- **QUANTITÉS : `NUMERIC`, JAMAIS entier** — ligne de vente comme mouvement de stock. Un hôtel
  vend 1 bière ; une quincaillerie vendra 2,3 mètres de fer ; une boulangerie achètera 47,5 kg
  de farine. Passer d'entier à décimal après mise en production imposerait de migrer toutes les
  lignes.
- **Les prix sont verrouillés à la création de la ligne.**
- **AUCUNE règle fiscale ne vit hors du trait `JurisdictionAdapter`.** Un seul adaptateur au
  MVP (`CoteDIvoire`). TVA, taxe de nuitée et taxe de développement touristique sont des
  sorties de l'adaptateur, jamais des constantes.
- Chaque formule de location porte `assujettie_taxe_nuitee` et une **règle de conversion** : le
  traitement fiscal du **passage** et de la **demi-journée** est un **PARAMÈTRE**, jamais une
  constante.
- **Tout calcul fiscal a un test doré sur jeu de cas figés, exécuté en CI.**
- **Documents opérationnels et documents fiscaux sont deux agrégats étanches**, avec deux
  numérotations et deux cycles de vie. Tout document opérationnel porte la mention
  « **Document non fiscal — ne tient pas lieu de facture** ». Le mode dégradé ne produit jamais
  un document ressemblant à une facture normalisée.
- **L'API FNE n'expose AUCUNE clé d'idempotence.** L'état `INDETERMINEE` (timeout) N'EST JAMAIS
  rejoué automatiquement : **rapprochement manuel obligatoire**. Cycle imposé :
  `EN_ATTENTE → SOUMISE → CERTIFIEE`, avec `ECHEC` (erreur métier explicite, correction et
  resoumission) et `INDETERMINEE` (écran de rapprochement).
- **Les `id` d'items retournés par l'API de certification sont persistés** — sans eux, aucun
  avoir n'est possible. L'avoir se fait **par quantité, pas par montant**.
- Le montant de la taxe de nuitée est **figé au check-out**, jamais recalculé dynamiquement ;
  toute modification postérieure passe par un avoir.

*Rationale* : une erreur d'arrondi ou de type numérique répliquée chez plusieurs clients est
un risque fatal, et un rejeu naïf de certification produit une double facturation avec double
consommation de sticker.

### VI. Hors-ligne et résilience réseau

- **Chaque entité déclare sa classe A/B/C/D** dans `docs/registre-classes-offline.md`, selon
  l'arbre de décision du cadrage §11.2. En cas de doute, **classer plus strictement**.
- **Une opération B, C ou D atteignable depuis un chemin de code exécutable hors ligne FAIT
  ÉCHOUER LE BUILD.** Invariante vérifiée par test, pas par convention.
- **Toute écriture porte un UUID v7 généré côté client** ; le serveur déduplique ; **le rejeu
  est idempotent** ; **le serveur fait foi en conflit**.
- « Dernier écrit gagne » est autorisé **uniquement** sur les entités A sans conséquence.
- **La file se vide AU RETOUR AU PREMIER PLAN par défaut, sur toutes les plateformes** — iOS
  n'a pas de synchronisation en arrière-plan. `BGTaskScheduler` et `WorkManager` sont des
  **optimisations, jamais des hypothèses**.
- **AUCUNE donnée B, C ou D en cache d'écriture sur un terminal** : ces entités sont en lecture
  seule côté client. Purge du cache à la déconnexion ; chiffrement au repos sur mobile.
- **L'interface annonce immédiatement toute action indisponible faute de réseau** — jamais de
  grisé silencieux, jamais d'échec après coup, jamais de mise en file « au cas où ».
- Un **indicateur de synchronisation permanent** affiche connecté / dégradé / hors ligne et le
  nombre d'éléments en attente.
- Le **conflit d'écriture orpheline** (consommation hors ligne arrivant sur un séjour clos et
  facturé) va dans une **file de réconciliation à résolution humaine obligatoire** : jamais de
  rejet silencieux, jamais d'ajout d'office.

*Rationale* : une entité indûment classée A produit des incohérences silencieuses découvertes
trois mois plus tard en pleine clôture. Une entité indûment classée B produit une frustration
immédiate, visible et corrigeable.

### VII. Application unique et rôles cumulés

- **Une seule application Nuxt 4 + Tauri v2** pour tous les rôles métier — desktop, Android,
  iOS — en mode SPA. Surfaces web publiques séparées (page QR en SSR, console éditeur).
- **Les rôles sont CUMULABLES** : un utilisateur porte N rôles, ses permissions sont l'union.
  **C'est la norme, pas l'exception.** Un sélecteur de contexte permanent (établissement,
  poste) évite d'afficher tout simultanément.
- **L'accueil est un tableau de bord de tuiles filtrées par permission**, jamais un menu figé.
- **Chargement paresseux par module** : un serveur de salle ne télécharge pas le code du
  back-office.
- **L'interface ne montre JAMAIS un module d'activité ou une capacité inactifs** : pas de
  grisé, pas de « disponible dans votre offre ». **Absent.**
- **AUCUNE invocation directe de `window.__TAURI__` dans un composant.** Impression, scan, OCR,
  stockage sécurisé, notifications, géolocalisation et réseau passent par **`PlatformAdapter`**,
  avec implémentations `desktop`, `android`, `ios`, `web`. **Une capacité absente le DIT
  explicitement à l'utilisateur** ; elle n'échoue jamais en silence.

*Rationale* : un gérant qui est aussi caissier et réceptionniste installe une seule
application. L'adaptateur de plateforme est ce qui permet d'ajouter iOS sans rouvrir les
composants métier.

### VIII. Qualité, i18n et observabilité

- **Transitions d'état couvertes par des tests d'intégration** — pas seulement des tests
  unitaires.
- **Requêtes sqlx vérifiées à la compilation** (`cargo sqlx prepare` vert en CI).
- **AUCUNE chaîne utilisateur en dur** : clés i18n **fr ET en**, **fr par défaut**.
- **MODE SOMBRE dès le premier écran, jamais rétrofitté.** Chaque écran est vérifié en mode
  clair et en mode sombre avant d'être considéré comme terminé.
- **AUCUNE couleur ni espacement littéral** hors des tokens de `docs/design/tokens.md`.
- **Logs structurés avec corrélation** par requête ; **Sentry** ; sonde **`/health`** ;
  télémétrie de version pour le parc auto-hébergé.
- Tout document imprimé est vérifié sur **imprimante thermique réelle** avant clôture de la
  story.

*Rationale* : le support se fait à distance depuis Abidjan, à 220 km du pilote. Sans logs
corrélés et sans télémétrie, le diagnostic est impossible ; rétrofitter le mode sombre ou
l'i18n coûte plusieurs fois leur prix initial.

### IX. Sécurité

- **Le verrouillage par adresse MAC N'EST JAMAIS implémenté** : il est techniquement impossible
  (iOS 14 et Android 10 randomisent la MAC par réseau ; Android n'expose pas la MAC
  matérielle). À la place :
  - **enrôlement d'appareil** — le gérant approuve l'appareil une fois, une **paire de clés
    générée dans le Keystore/Keychain signe chaque requête** ;
  - **attestation d'intégrité** (Play Integrity, DeviceCheck + App Attest) ;
  - **liste blanche révocable** depuis le back-office.
- **Le géorepérage est SOUPLE** : 300 m par défaut, **alerte au gérant**, **JAMAIS bloquant sur
  une action critique**. Une position simulée détectée déclenche une alerte, pas un refus.
- **Coffre chiffré par tenant** pour les clés FNE et les secrets d'agrégateur de paiement.
- **AUCUN secret dans le binaire Tauri** (décompilable).
- **Journal d'audit immuable** sur : remise, annulation de ligne envoyée, avoir, ouverture de
  tiroir, modification de tarif, suppression, changement de rôle, écart de caisse et
  **rebascule de palier de passage**. C'est un **module de premier plan, pas un journal
  technique** — consultable par le propriétaire depuis n'importe quel terminal.
- Webhooks de paiement **validés par signature HMAC** ; jamais de confiance dans la
  redirection client seule ; idempotence sur le webhook.

*Rationale* : le journal d'audit est ce que le propriétaire achète réellement. Un caissier qui
ne peut pas encaisser parce que le GPS dérive est un client perdu — d'où le géorepérage non
bloquant.

### X. Périmètre — « prêt ≠ construit »

- Les **provisions du cadrage §14** (adaptateurs de juridiction supplémentaires, devises
  actives, modules et capacités additionnels, profils de stock supérieurs, canal `TERNE`,
  convention inter-établissements, partenaires, documents commerciaux, correspondance
  comptable, contrats et cautions, comptes clients entreprises, nœud de site, IoT et contrôle
  d'accès) sont des **choix de modèle de données et d'interfaces uniquement** : **aucune UI,
  aucune logique au MVP**.
- **Toute fonctionnalité qui ne contribue pas à (a) faire abandonner le papier au pilote ou
  (b) garantir la conformité fiscale est REFUSÉE.**
- **Les priorités P0 / P1 / P2 / PROVISION des user stories font foi.** Une demande du pilote
  reçoit systématiquement la réponse « paramètre ou phase suivante », consignée dans un
  registre des demandes.
- Règle de dérive : si l'incrément 1 dérape de plus de 3 semaines, on **sort du périmètre les
  stocks et le tableau de bord multi-sites de l'incrément 2** — on ne repousse jamais la
  livraison au pilote.

*Rationale* : le périmètre complet représente environ 70 semaines-homme pour un développeur
solo. Le sur-périmètre est le risque le plus probable du projet.

### XI. Versions épinglées

- **Dernières versions stables** de chaque brique : Rust, Actix Web, sqlx, utoipa, Nuxt 4,
  Tailwind 4, Tauri v2, PostgreSQL, Redis, Garage.
- **VÉRIFIÉES SUR LES REGISTRES OFFICIELS, avec l'URL citée** dans le changement qui les
  introduit ou les met à jour. Le gel en vigueur, ses URL et ses commandes de vérification
  vivent dans **`docs/versions-gelees.md`**, qui fait foi.
- **ÉPINGLÉES EXACTEMENT** — pas d'intervalle, pas de `^`, pas de `~` — et **figées par
  lockfiles** commités.
- **NE JAMAIS proposer un numéro de version de mémoire.** Une version non vérifiée est une
  version inconnue.
- **Aucune montée majeure pendant un incrément.** Revue de mise à jour **groupée, mensuelle**.
- Pour le parc auto-hébergé : **versions N et N-1 supportées, pas plus** ; migrations
  automatiques et idempotentes au démarrage.

*Rationale* : un intervalle de version transforme une reconstruction reproductible en pari.
Une montée majeure en cours d'incrément consomme le budget d'un incrément entier.

### XII. Référence visuelle

- **`docs/design/html/{code}-{nom}[-{etat}].html` est la RÉFÉRENCE NORMATIVE de chaque
  écran** : valeurs exactes et hiérarchie DOM, **un fichier par état**. Les fondations sont
  dans `docs/design/fondation/`, les prototypes animés dans `docs/design/proto/`, les documents
  imprimés dans `docs/design/documents/`.
- **`docs/design/tokens.md`** porte les **valeurs curées** (couleurs claires ET sombres,
  typographie, espacements, rayons, ombres, mouvement) consommées par le thème Tailwind 4, et
  **PRIME sur tout export en cas de divergence**. **`docs/design/mouvement.md`** porte les
  durées et courbes, extraites des prototypes de `docs/design/proto/`.
- **LE HTML DE MAQUETTE N'EST JAMAIS COPIÉ NI DÉPLACÉ VERS `app/`.** C'est une **cible, pas une
  source** : il est autonome, non sémantique, sans i18n, sans mode sombre câblé, sans RBAC. **On
  lit ses valeurs, on réimplémente.**
- **SEULE EXCEPTION : `docs/design/theme.css`**, le bloc `@theme` Tailwind 4, est **copié tel
  quel** dans `app/assets/css/` — c'est lui qui porte les tokens dans le code.
- **TAILWIND 4 D'ABORD, CSS EN DERNIER RECOURS** :
  - tout style s'exprime en **utilitaires du noyau référençant les tokens de `@theme`** ;
  - le mode sombre passe par la **variante `dark:`**, jamais par une seconde palette ;
  - **aucune classe personnalisée, aucun style en ligne** ;
  - le CSS explicite est réservé à ce que Tailwind n'exprime pas (`@keyframes`, impression
    thermique) et **reste regroupé** en un seul endroit.
- **Une seule identité visuelle** sur desktop, Android et iOS.

*Rationale* : copier la maquette dans l'application importe du HTML non sémantique, sans i18n
et sans RBAC, qu'il faudra défaire écran par écran. Réimplémenter depuis des valeurs exactes
coûte moins cher que corriger une copie.

## Contraintes techniques et documents de référence

### Hiérarchie documentaire

En cas de contradiction, l'ordre de préséance est le suivant :

1. **Cette constitution** — pour toute question de gouvernance, d'architecture ou d'invariant.
2. **`docs/cadrage-v1.md`** — périmètre, modèle d'entité, fiscalité, classes hors-ligne,
   déploiement, provisions §14.
3. **`docs/user-stories-v1.md`** — critères d'acceptation, priorités, Definition of Done,
   récapitulatif des paramètres d'établissement.
4. **`docs/design/tokens.md`** puis **`docs/design/mouvement.md`** — valeurs de design.
5. **`docs/design/html/`**, **`docs/design/fondation/`**, **`docs/design/proto/`**,
   **`docs/design/documents/`** — référence d'écran, de fondation, de mouvement et d'impression.
6. **`docs/Kaya_Vision_Plateforme.md`** — **fermé jusqu'au jalon J1** ; sans effet sur le MVP
   au-delà des provisions du cadrage §14.

### Pile technique imposée

| Couche | Choix | Contrainte |
|---|---|---|
| API | Actix Web (Rust) + utoipa + utoipa-swagger-ui | Spec sur `/api-docs/openapi.json` ; Swagger UI protégée hors production |
| Accès données | sqlx | Requêtes vérifiées à la compilation ; migrations versionnées |
| Base | PostgreSQL | Seule vérité durable ; RLS forcée ; contraintes d'exclusion GiST |
| Éphémère | Redis | Reconstructible uniquement ; jamais de donnée métier durable |
| Objets | Garage | **API S3 uniquement** ; une clé d'accès par usage |
| Application | Nuxt 4 + Tauri v2 + Tailwind 4 | SPA sous Tauri ; `PlatformAdapter` obligatoire |
| Fiscalité | Trait `JurisdictionAdapter` ; trait `FneGateway` (`Partenaire` \| `Direct`) | Bascule par configuration de tenant, sans toucher au métier |
| Paiement | Trait `PaymentProvider` ; CinetPay au MVP | Session créée côté serveur ; webhook signé HMAC |
| Développement | Docker + Compose | Kubernetes hors sujet |

Le serveur, le nœud de site et le paquet auto-hébergé sont **le même binaire Actix avec trois
configurations**. Jamais trois produits.

### Artefacts de gouvernance à maintenir

| Artefact | Rôle | Mis à jour |
|---|---|---|
| `docs/registre-classes-offline.md` | Classe A/B/C/D de chaque entité | À la création de toute entité |
| Récapitulatif des paramètres (`docs/user-stories-v1.md`) | Inventaire des paramètres d'établissement | À l'ajout de tout paramètre |
| Jeux de cas figés fiscaux | Tests dorés du moteur de taxes | À toute évolution fiscale |
| `docs/versions-gelees.md` | Versions vérifiées sur registres officiels, URL citées | Revue mensuelle groupée |
| Lockfiles (`Cargo.lock`, `pnpm-lock.yaml`, `rust-toolchain.toml`, `.nvmrc`) | Versions figées | Avec le gel ci-dessus |

## Portes de conformité et flux de développement

### Definition of Done

Une story n'est terminée que lorsque les dix points suivants sont vrais :

1. Critères d'acceptation couverts par des tests — unitaires **et** intégration sur les
   transitions d'état.
2. Annotations utoipa à jour ; client TypeScript régénéré **sans diff manuel**.
3. Migration sqlx versionnée ; `cargo sqlx prepare` vert ; seeds à jour.
4. **RLS activée et forcée** sur toute nouvelle table, avec test d'isolation multi-tenant.
5. **Classe hors-ligne déclarée** (A/B/C/D) pour toute nouvelle entité, avec ses tests.
6. **Événement outbox émis** pour tout changement d'état métier.
7. Clés **i18n fr et en** externalisées ; aucune chaîne en dur.
8. Écran vérifié **en mode clair et en mode sombre**.
9. Paramètres exposés dans la configuration d'établissement dès que la story dit
   « paramétrable ».
10. Tout document imprimé vérifié sur **imprimante thermique réelle**.

### Portes de CI bloquantes

Chacune fait échouer le build. Aucune n'est contournable par convention ou revue.

| Porte | Vérifie | Principe |
|---|---|---|
| P-01 | Le client TypeScript généré est identique au client commité | I |
| P-01b | **Tous les `operationId` du contrat OpenAPI sont uniques** — deux opérations homonymes produisent un client TypeScript invalide, que P-01 ne détecte pas puisqu'elle ne compare que le généré au commité | I |
| P-02 | Aucune migration déjà appliquée n'a été modifiée | I |
| P-03 | Aucun crate de `socle/` ne dépend d'un crate de `verticales/` | II |
| P-04 | Aucune requête ne joint deux schémas de modules différents | II |
| P-05 | Toute transition d'état émet un événement outbox dans sa transaction | II |
| P-05b | **Aucun chemin de code ne supprime ni ne modifie une ligne d'un REGISTRE IMMUABLE** — pas de `DELETE`, pas d'`UPDATE` hors marquage de publication, aucune purge, aucune rétention bornée. Sont des registres immuables l'outbox et le journal d'audit ; la porte porte sur la **catégorie**, pas sur une liste de tables, afin que le prochain registre soit couvert sans amendement | II, IX |
| P-06 | Toute valeur de capacité autre que `STOCK`/`SIMPLE` est refusée explicitement | II |
| P-07 | Toute table du schéma porte au moins une politique RLS, `ENABLE` et `FORCE` | III |
| P-08 | Le tenant A ne lit ni n'écrit aucune ligne du tenant B, sur chaque endpoint | III |
| P-09 | Toute occupation est un `tstzrange` protégé par une contrainte d'exclusion GiST | IV |
| P-10 | Aucun montant non entier ; aucune quantité non `NUMERIC`. **La garantie ne s'arrête pas à la frontière du JSONB** : toute clé monétaire d'un document JSON suit le nommage réservé et porte un **entier**, jamais un décimal ni une chaîne formatée — sans quoi le principe V cesse de tenir sur le registre même qui trace les écarts d'argent | V |
| P-11 | Tests dorés fiscaux verts sur jeux de cas figés | V |
| P-12 | Aucune règle fiscale hors du trait `JurisdictionAdapter` | V |
| P-13 | Aucune opération B, C ou D atteignable depuis un chemin exécutable hors ligne | VI |
| P-14 | Rejeu triple d'une écriture A produit un seul enregistrement ; désordre commutatif | VI |
| P-15 | Aucune invocation de `window.__TAURI__` hors de `PlatformAdapter`, **dans `app/` ET dans `web/`** — les surfaces publiques sont hors Tauri, elles ne doivent jamais importer `@tauri-apps/api` | VII |
| P-16 | Aucune chaîne utilisateur en dur ; parité des clés fr / en | VIII |
| P-17 | Aucune couleur ni espacement littéral hors tokens | VIII, XII |
| P-18 | `cargo sqlx prepare` vert | VIII |
| P-19 | Aucun fichier de `docs/design/html/` copié sous `app/` | XII |
| P-20 | Aucune dépendance déclarée en intervalle ; lockfiles commités et à jour | XI |
| P-21 | **Aucune ressource chargée depuis un hôte externe** — police, icône, script, feuille de style, image. Un CDN rend l'écran dépendant du réseau, ce que le mode hors-ligne interdit | VI, XII |
| P-21b | **Toute ressource déclarée est effectivement embarquée** — chaque famille de `--font-*` du bloc `@theme` est servie par un `@font-face` local, chaque glyphe employé figure dans la police sous-réglée, et **toute police embarquée est accompagnée de sa licence et de son avis de copyright, atteignables depuis le produit**. Retirer un CDN sans embarquer son contenu fait passer P-21 au vert **en n'affichant rien** ; embarquer une police sans son attribution redistribue une œuvre sous licence sans en respecter les termes | VI, IX, XII |
| P-22 | **PARCOURS RÉEL — l'application démarre et chaque route déclarée s'atteint**, sans erreur de console, de deux manières : par navigation interne **et** par chargement direct de l'adresse. Le thème déclaré s'applique effectivement. Un composant monté en test n'est pas un écran atteint : `@vue/test-utils` contourne le routeur, `<Suspense>`, les layouts et les plugins — tout ce qui fait qu'une page existe pour un utilisateur | VII, VIII |
| P-23 | **PROVENANCE DE L'INSTANT — aucun calcul métier, fiscal, de clôture ou de durée ne s'appuie sur `horodatage_client`.** Seul l'horodatage d'autorité serveur fait foi. Exemptions limitativement énumérées dans le script : ordre d'affichage local, détection de dérive d'horloge, rendu de l'instant tel que le terminal l'a perçu. Périmètre **découvert**, jamais énuméré à la main | IV |

### Couverture des portes — leçon du cycle 1

Une porte n'est acquise que lorsque son **périmètre** est établi, pas seulement sa capacité à
échouer. Le cycle 1 a produit quatre portes vertes défectueuses, dont **aucune n'a été trouvée par
relecture** : P-08 lisait un contrat vide alors que deux endpoints étaient servis ; P-18 validait
43 requêtes sur 47 ; et le mode vérification de P-18 écrasait le cache qu'il inspectait.

> **Un test négatif prouve qu'une porte sait échouer. Il ne prouve pas qu'elle regarde tout.**

Le cycle 002 l'a confirmé deux fois : le décompte de P-07 ne couvrait que 4 tables sur 10, et
**ESLint ne parsait aucun `.vue` typé** — la porte P-15 était verte parce qu'elle ne gardait rien,
masquée par l'unique composant non annoté du cycle précédent. Une porte dont la cible est vide est
indistinguable d'une porte qui passe.

Les exigences suivantes en découlent, applicables à toute porte nouvelle ou corrigée :

1. **Déclarer le périmètre inspecté** en commentaire de tête : ce que la porte lit, et ce qu'elle
   ne lit pas. Une limite assumée et écrite vaut mieux qu'une couverture supposée.
2. **Vérifier la complétude, pas seulement l'échec** : compter les cibles réellement examinées et
   les comparer au total attendu. Une porte qui inspecte un sous-ensemble sans le dire donne une
   fausse assurance, ce qui est pire que pas de porte.
3. **Ne jamais modifier l'artefact inspecté.** Un contrôle qui écrit dans ce qu'il vérifie peut
   masquer le défaut qu'il cherche.
4. **Prouver que la porte a une cible non vide.** Une porte qui n'inspecte rien passe toujours.
   Le test l'exerce sur au moins un cas réel, ou déclare explicitement qu'elle est installée à
   vide et le vérifie par une assertion de non-régression.
   *Corollaire — toute interdiction a un versant positif.* Une porte qui refuse une source
   externe doit vérifier que le contenu local existe ; une porte qui refuse une valeur en dur
   doit vérifier que le token est employé. Sans ce versant, supprimer la cible suffit à passer
   au vert : c'est ce qui a produit un écran sans icônes au cycle 002, puis une application sur
   polices système de repli au volet suivant.
5. **Exercer tout nouveau type d'événement sur les deux tenants de démonstration.** Le défaut de
   séquence de l'outbox — un espace de numérotation partagé entre tenants, corrigé par la
   migration 0012 — n'a été trouvé ni par relecture ni par une porte, mais par le premier
   événement de portée tenant appliqué à un second tenant. La couverture s'étend avec les
   fonctionnalités : elle doit être re-exercée, pas supposée acquise.
6. **Une unité écrite n'est ni testée ni branchée par défaut.** Pour toute fonction d'amorçage —
   thème, session, file hors-ligne, adaptateur de plateforme, i18n, télémétrie — **deux preuves
   distinctes sont dues** : un test qui l'exerce, et un test qui vérifie qu'elle est **appelée dans
   le parcours réel**. L'application en comptait onze, dont **cinq n'étaient appelées nulle part**
   et une n'était pas même importée par le fichier de test censé la couvrir. Le nom d'un fichier de
   test ne prouve pas ce qu'il teste.

### Flux de développement

- **Un module = un epic = un cycle Spec Kit** : `/speckit-specify` sur la section du module,
  `/speckit-plan` en pointant le cadrage §13 (pile) et §14 (provisions) comme contraintes, puis
  `/speckit-tasks` et implémentation.
- **Implémentation par tranches verticales**, jamais module par module de bout en bout. Ordre
  des tranches fixé au §0.5 de `docs/user-stories-v1.md`.
- **Module doré d'abord** : avant toute génération assistée, un module écrit à la main — entité,
  repository, service, handler, tests — sert de patron.
- **Pas de généricité prématurée** : du code concret se refactore, une abstraction prématurée
  se subit.

## Governance

**Autorité.** Cette constitution prime sur toute autre pratique, préférence ou habitude de
développement du projet. Un désaccord entre un choix d'implémentation et un principe se résout
en faveur du principe.

**Amendement.** Tout amendement requiert : (a) l'écriture du changement dans ce fichier via
`/speckit-constitution`, (b) un rapport d'impact en tête de fichier, et (c) lorsque
l'amendement invalide du code existant, un plan de migration explicite dans le même changement.
Un principe n'est jamais contourné en silence : il est amendé ou il est respecté.

**Versionnement.** Version sémantique `MAJOR.MINOR.PATCH` :

- **MAJOR** — suppression ou redéfinition incompatible d'un principe ou d'une règle de
  gouvernance.
- **MINOR** — ajout d'un principe ou d'une section, ou extension matérielle d'une règle
  existante.
- **PATCH** — clarification, reformulation, correction sans effet sémantique.

**Conformité.** Chaque plan de fonctionnalité passe un `Constitution Check` avant
implémentation. Les portes P-01 à P-23 (P-01b, P-05b et P-21b incluses) sont exécutées en intégration continue et leur échec
bloque la fusion. Toute complexité ajoutée doit être justifiée par écrit dans le plan ; à
justification absente, l'option la plus simple s'impose.

**Dérogation.** Une dérogation est temporaire, nommée, datée et accompagnée de sa condition de
lever. Une dérogation non levée à la revue mensuelle devient soit un amendement, soit une
tâche de mise en conformité.

**Revue.** Revue mensuelle groupée couvrant : versions des briques (principe XI), dérogations
ouvertes, décisions de l'annexe B du cadrage arrivées à échéance, et cohérence du registre des
classes hors-ligne avec le code.

### Éléments différés

- **Décisions ouvertes du registre des classes hors-ligne.** `docs/registre-classes-offline.md`
  existe depuis le 2026-07-30 et fait foi (principe VI). Il consigne trois décisions non
  tranchées, dont la classe la plus stricte s'applique en attendant : **O-01** — `client` /
  `personne` en C rend le check-in d'un client inconnu impossible hors ligne, y compris en
  mode C (à trancher avant SEJ-02) ; **O-02** — classe de `mouvement_stock`, décision B-05 du
  cadrage (à trancher avec le pilote) ; **O-03** — crate d'accueil de la surface QR, absente des
  quatre verticales du principe II (à trancher avant QRC-01).
- **Inventaire visuel — constaté le 2026-07-30, aucune action requise.** `docs/design/html/`
  contient **11 codes d'écran** — C4, F2, G2, M4, P2, Q1, R1, R4, R7, S2, V1 — répartis en
  **29 fichiers d'états**. `F2-registre-grave.html` et `S2-registre-grave.html` sont **deux
  écrans distincts** partageant un suffixe de nom : F2 traite le document fiscal `INDETERMINEE`
  (FIS-05), S2 la consommation orpheline arrivée après le départ (SYN-03). Le principe XII ne
  fige aucun décompte, afin qu'un état ajouté ne rende pas la constitution fausse.
- **Cohérence des tokens — vérifiée le 2026-07-30, aucune action requise.** Les 71 tokens
  tabulés de `docs/design/tokens.md` sont tous présents dans le bloc `@theme` de
  `docs/design/theme.css` (104 déclarations, les 33 supplémentaires étant les interlignes, durées,
  courbes et animations). **Aucune divergence de valeur.** Les seuls écarts sont de notation
  typographique — « 14,5 px » contre « 14.5px », « / .85 » contre « / 0.85 ». La règle de
  préséance du principe XII reste en vigueur comme filet, sans objet aujourd'hui.

**Version**: 1.8.0 | **Ratified**: 2026-07-30 | **Last Amended**: 2026-08-02
