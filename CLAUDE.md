# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## État du dépôt — lire en premier

**La tranche T1 est COMPLÈTE** — cycles 001 (TRX), 002 (ETB), 003 (CPT), 004 (HEB), 005 (SYN) et
006 (SEJ) livrés. Le socle, les établissements, les comptes, l'hébergement, la synchronisation, et
désormais **les clients et les séjours** : arrivée, passage, note, départ, fiche client.

Les décomptes de tests et de tâches **ne sont pas tenus ici** — ils changent à chaque commit, et un
nombre recopié dans ce fichier est faux avant d'être lu. `git log` et les revues de fin de cycle
(`specs/*/revue-dod.md`) font foi.

**Ce que le cycle 005 change, et qu'il faut savoir avant de coder :**

- **La file hors-ligne est réelle.** Persistante, chiffrée par WebCrypto (AES-GCM, clé au coffre
  système, cryptogramme dans le stockage ordinaire), vidée par **quatre déclencheurs et aucune
  minuterie de scrutation**. Son premier passager est la note interne — `/notes`, écran composé.
  Le témoin de synchronisation (composant 10) est monté dans la **coquille**, donc sur toutes les
  pages.
- **Le périmètre des portes est DÉCOUVERT, jamais énuméré.** `backend/tests/commun/perimetre.rs`
  lit les schémas de `pg_namespace` et les crates des `[workspace] members`. Vingt et une
  occurrences de chemin en dur ont disparu, et `perimetre_decouvert.rs` **refuse qu'on en
  réintroduise**. Ne pas écrire `crates/socle/...` dans un test : composer par `chemin_crate()`.
- **Les tests du §0.7 s'instancient.** `tester_classe_a!`, `tester_classe_bcd!`,
  `tester_classe_d!` dans `backend/tests/commun/classes.rs`. Couvrir une entité coûte une
  déclaration, et `outillage_classes.rs` échoue en **nommant** celle qui l'aurait oubliée.
- **P-23 garde la provenance de l'instant.** `cree_le` fait autorité, `horodatage_client` ne porte
  aucune règle. Trois exemptions **limitativement** énumérées — écrire la colonne n'en fait pas
  partie : écrire une valeur n'est pas s'appuyer dessus.

**La leçon la plus chère du projet à ce jour** : le cycle 003 a été livré avec 24 portes vertes et
652 tests, et **deux de ces cinq écrans étaient inatteignables en navigateur**. L'application
n'avait aucun point d'amorçage — ni `plugins/`, ni `layouts/`, ni `middleware/` — et chaque page
amorçait pour elle seule ce qu'elle avait pensé à amorcer. C'est réparé, et la porte **P-22** ouvre
désormais chaque route pour de vrai, en direct et par navigation, dans les deux thèmes. **Deux
règles en sont sorties, et elles coûtent cher à réapprendre :**

- **Une page a UNE SEULE racine, et c'est un élément** — jamais un `v-if`/`v-else` de premier
  niveau. Une racine multiple compile en fragment ; un fragment dont la branche active est un
  `defineAsyncComponent` non résolu a un `el` nul, et Vue lève
  `Cannot read properties of null (reading 'parentNode')` à la navigation suivante. L'écran ne se
  monte pas, l'ancien reste affiché, l'adresse a pourtant changé.
- **Une unité écrite n'est ni testée ni branchée par défaut.** `initialiserTheme()` a vécu deux
  cycles exportée, documentée « à appeler au démarrage » — et appelée nulle part. Quatre autres
  points d'entrée étaient dans le même état. `app/tests/amorcage.spec.ts` les porte tous, à deux
  états, et vérifie **les deux versants**.

**Le patron de référence est `docs/module-dore.md`** (812 lignes, **huit** couches — la huitième
est le cycle de vie de l'application) : une tranche verticale écrite à la main contre sqlx 0.9.
**Le lire avant d'écrire du Rust** — tout extrait trouvé en ligne vise sqlx 0.8 et ne compilera
pas. **Et avant de créer une page** : la huitième couche dit où va le thème, où va la session, et
ce que le layout rend.

**Les trois dettes du cycle 002 sont soldées** — lire avant de coder du front :

1. **Le patron d'écriture front existe, sur UNE opération.** La bascule d'un service (ETB-02) est
   câblée de bout en bout et documentée dans **`docs/module-dore.md`, « La septième couche »** :
   appel typé, squelette de chargement, refus métier en langue utilisateur, validation au champ,
   action **absente** sans permission, refus immédiat hors ligne (classe C), rafraîchissement sans
   rechargement. Le cycle 003 l'a étendu de trois points — garde de permission, stockage sécurisé
   par `PlatformAdapter`, ordre **rafraîchir-avant-vider** — et l'a appliqué à quatre écrans.
   **Les opérations d'écriture restantes suivent ce patron, cycle par cycle** — le compte n'est plus
   tenu ici : le contrat en sert 43 et il grandit à chaque cycle. Le lire avant d'en brancher une,
   ne pas réinventer.
2. **La police d'icônes est embarquée et sous-réglée.** 77 glyphes sur ~1530, 9,4 ko au lieu de
   279. Régénération : `pnpm --filter @kaya/app icones:generer`. La porte **P-21** refuse toute
   ressource d'hôte externe, **P-21b** vérifie que le contenu local existe vraiment.
3. **Archivo et Chivo Mono sont embarquées.** Quatre `woff2` variables, 114 ko, `latin` et
   `latin-ext`, **sans sous-réglage de caractères** — le texte est dynamique, contrairement aux
   icônes. Régénération : `pnpm --filter @kaya/app polices:generer`.

**Le piège des polices, à connaître avant d'y toucher : U+202F n'existe ni dans Archivo ni dans
Chivo Mono.** `docs/design/tokens.md` §2 impose pourtant l'espace fine insécable **U+202F** entre
les groupes de milliers et avant le F (`12 500 F`), et en fait la condition de l'alignement des
colonnes de montants en Chivo Mono tabulaire. Le caractère est absent des `woff2` de Fontsource
**et** des `ttf` amont de Google Fonts — alors que la `unicode-range` déclarée annonce
`U+2000-206F`. **La plage annoncée n'est pas la couverture réelle : seule la table `cmap` fait
foi.** `app/scripts/generer-polices.ts` ajoute donc l'association `U+202F → dessin de U+2009`, et
la porte **P-21b** relit la table pour le vérifier. Deux corollaires qui se paient cher :

- **L'ordre des `@font-face` compte** : `latin-ext` AVANT `latin`. Les plages se recouvrent (`œ`
  est annoncé par les deux, dessiné par une seule) et **à recouvrement, le dernier déclaré gagne**.
- **Un woff2 réécrit doit être complété à quatre octets**, sinon le décodeur des navigateurs le
  refuse en bloc.

**Le composant de saisie canonique est `app/core/design-system/ChampSaisie.vue`** — n° 16 de
`docs/design/composants.md`, avec sa vignette au styleguide. Aucun écran n'en a de maquette : il
est composé depuis les tokens. **Tout champ de formulaire passe par lui.**

**Un montant s'écrit par `app/core/format/montant.ts`, et par rien d'autre.**
`formaterMontant(montantMineur, codeDevise)` — le montant est un **entier d'unité mineure**, le
nombre de décimales et le symbole viennent de la **devise** (principe V), jamais d'une constante.
Ne pas recopier le `money(n)` de `tokens.md` §2 : c'est du code de maquette, mono-devise et sans
unité mineure, et le reprendre imposerait de rouvrir chaque appel à la deuxième devise (principe X).
`Intl.NumberFormat` est écarté aussi — son séparateur dépend de l'ICU embarqué, U+202F ou U+00A0
selon la version. `app/tests/montant.spec.ts` refuse toute seconde implémentation dans `core/`,
`modules/` et `pages/`. **Les heures gardent l'espace ORDINAIRE (`17 h 30`)** et ne passent pas
par là.

**Le styleguide est servi par l'application** : `app/pages/styleguide.vue`, les seize composants
dans tous leurs états, en clair et en sombre, avec les polices **réellement embarquées** — ce que
`docs/design/styleguide.html` ne peut pas montrer, chargeant les siennes depuis Google Fonts.
Route **retirée du routeur** hors développement, comme la Swagger UI du cycle 001 :

```sh
KAYA_STYLEGUIDE=1 pnpm --filter @kaya/app dev    # puis /styleguide
```

C'est aussi le seul fichier `.vue` **exempté** du contrôle des littéraux de P-16 — exemption
nommée, dont la contrepartie (la page n'atteint pas la production) est vérifiée par la porte
elle-même.

**Le parcours est réparé et opposable.** `app/plugins/`, `app/middleware/` et
`app/layouts/default.vue` portent l'amorçage — thème avant rendu, reprise de session avant chaque
navigation, coquille unique. La porte **P-22** vérifie que chaque route se charge en direct **et**
par navigation, sur **Chromium et WebKit**, sans erreur de console.

**Tauri n'embarque pas Chromium** : WKWebView sur macOS et iOS, WebKitGTK sur Linux, WebView2
sur Windows. P-22 tourne donc sur **Chromium et WebKit** — mais le WebKit de Playwright **n'est
pas** WKWebView. Un vert dit « tourne sur un moteur WebKit », jamais « vérifié sur la cible ».
La vérification sur WKWebView viendra avec la coquille Tauri.

**Le libellé de la déconnexion est « Passer la main »**, pas « Se déconnecter » : sur un terminal
de comptoir, l'appareil ne bouge pas, c'est la personne qui change. À ne pas confondre avec
« Déconnecter cet appareil », qui coupe un autre appareil à distance. Les deux entrées sont au
lexique — **tout terme visible passe par lui avant d'être codé**.

État par tranche : **T1 LIVRÉE**. Suivante : **T2** — services et note (restauration, bar,
pressing, salle de réunion), §0.5 de `docs/user-stories-v1.md`.

**Ce que le cycle 006 a trouvé, et qui vaut plus que ses quatre écrans.** Six défauts, tous par des
portes, **aucun par relecture**. Le détail est dans
`specs/006-clients-sejours-enregistrement/revue-dod.md`. Trois à connaître avant de coder :

- **UN DOUBLE DE TEST PEUT RENDRE VRAI CE QUE LE CODE REND FAUX.** `/passage` — l'écran dont le
  cadrage fait une condition d'existence du produit — **ne se montait pas en navigateur** : il
  importait `useEtatReseau` d'un baril qui ne l'exporte pas. Les tests unitaires ne pouvaient pas
  le voir, car ils **doublaient ce baril en fournissant l'export manquant**. Le mock réparait le
  défaut qu'il était censé attraper. Corollaire général : **un test qui double un module de
  frontière ne prouve rien sur ce module** — seule une porte qui charge le vrai le prouve. C'est
  P-22 qui l'a trouvé, et un contrôle dédié rend désormais ce verdict en millisecondes.
- **Une grille de sélection peut proposer ce que le serveur refusera.** La grille du passage offrait
  des chambres d'autres catégories : refus **subi après le geste, devant le client**. Invisible en
  test unitaire, qui ne fournit qu'une catégorie. Toute liste de choix se teste avec **au moins
  deux** valeurs de la dimension qui filtre.
- **Le modèle de privilèges a enseigné une règle métier.** La base a refusé une remise à neuf des
  seeds — `permission denied for table ligne_sejour` — et elle avait raison :
  **une correction sur une note est une ligne d'ajustement, jamais une suppression.** Quand un
  `GRANT` manquant bloque, la première hypothèse est que le privilège a raison.

Deux défauts étaient dans l'outillage même, ce qui est le pire endroit : les seeds n'appliquaient
pas le mot de passe qu'ils déclaraient, et **un test était vert de 10 h à minuit, rouge de minuit à
10 h** — dépendance à l'horloge locale dans un test, le cycle même qui suit l'adoption de P-23.

**Le correctif de l'accueil a trouvé deux choses qui valent pour tout le dépôt.** L'accueil ne
menait qu'à deux des treize écrans livrés — six routes n'étaient dans aucun catalogue, et deux
tuiles légitimes étaient masquées :

- **UN COMMENTAIRE QUI JUSTIFIE UNE VALEUR EN DUR EMPÊCHE DE LA RELIRE.** `app/pages/index.vue`
  portait `const modulesActifs = computed(() => [])` suivi de « vide à ce cycle, et c'est exact ».
  C'était exact quand ce fut écrit. Le cycle 004 a ensuite donné `moduleRequis: 'HEBERGEMENT'` à
  deux tuiles, et le filtre par module n'a plus **jamais** rien laissé passer depuis qu'il servait à
  quelque chose. Le commentaire **rassurait la relecture au lieu de l'alerter** : un `TODO` aurait
  été vu, une justification ne l'est pas. Deloria a cinq services actifs, pas zéro.
- **P-21b avait un versant manquant : elle vérifie que le DÉCLARÉ est embarqué, jamais que le RENDU
  est déclaré.** `ph-list-magnifying-glass` manquait du `woff2` depuis le cycle 003 — la tuile du
  registre des actions s'affichait **sans icône** chez le propriétaire. Le générateur ne relevait
  que les attributs `class=` littéraux, et le catalogue de tuiles nomme ses glyphes **en donnée** ;
  trois des cinq étaient embarqués par coïncidence. C'est l'exigence 4 de la constitution — *toute
  interdiction a un versant positif* — appliquée à une porte qui n'avait que le sien.

Et un rappel de la même famille : `ecran-r1.spec.ts` décrivait un produit disparu — Yao y « ne
voyait que l'établissement » avec cinq permissions, il en a seize. **Un test qui décrit un état
ancien ne se contente pas d'être inutile : il rassure.**

**Ce que le cycle 005 a trouvé, et qui vaut plus que ce qu'il a construit.** Six défauts, dont
quatre qu'aucune relecture n'aurait vus — le détail est dans
`specs/005-file-hors-ligne-horodatage/revue-dod.md`. Deux méritent d'être connus ici :

- **Un test peut passer au vert en n'inspectant rien, et le balayage hors ligne l'a fait.** Sa
  première version ouvrait chaque écran par `page.goto` — or le jeton d'accès vit en mémoire, un
  rechargement exige le réseau pour reprendre la session, et hors ligne **toutes les routes
  renvoyaient sur `/connexion`**. Neuf cas verts, neuf fois le même écran de connexion. Le contrôle
  qui l'empêche désormais tient en une ligne : vérifier que l'URL n'est **pas** `/connexion`.
- **Un contrôle de type peut ne rien contrôler.** `const x: FamillesNonListees[] = []` compile
  quel que soit le type — un tableau vide est assignable à tout. La forme qui tient compare à
  `never` **sans distribution** : `[T] extends [never] ? true : false`.

**⚠️ N'ARRÊTE JAMAIS UN PROCESSUS PAR SON NOM DE COMMANDE.** `pkill -f "nuxt.mjs dev"` a tué le
serveur de développement d'un **autre projet** de ce poste, qui tournait depuis cinq heures.
Cible par **port** — `lsof -ti:3000 | xargs kill` — ou par répertoire de travail. Le projet
emploie déjà `lsof` sur le port dans `REPRISE.md` pour la détection ; utilise le même moyen pour
l'arrêt. Plus généralement : **rien hors du dépôt** — ni processus, ni fichier de configuration
du poste.

## Langue et conventions de nommage

Le projet est **entièrement en français** — documentation, échanges, et **identifiants métier**.
C'est la convention la moins évidente et la plus facile à casser :

| Catégorie | Langue | Exemples réels des docs |
|---|---|---|
| Crates, tables, colonnes, entités | **français sans accent** | `etablissement`, `unite_louable`, `sejour`, `point_de_vente`, `article_vendable`, `ressource_reservable`, `mouvement_stock`, `cout_unitaire`, `assujettie_taxe_nuitee`, `regle_conversion_taxe` |
| Traits d'abstraction | **anglais** | `JurisdictionAdapter`, `FneGateway`, `PaymentProvider`, `EmissionChannel`, `AccessController`, `PlatformAdapter` |
| Valeurs d'énumération | **MAJUSCULES françaises** | `HEBERGEMENT`, `SALLE_REUNION`, `NUITEE`, `PASSAGE`, `DEMI_JOURNEE`, `EN_ATTENTE`, `INDETERMINEE` |
| Statuts de cycle de vie | minuscules françaises | `depose → en_traitement → pret → retire` · `provisoire → confirmee → honoree \| annulee \| no_show` |

**Reprendre littéralement les noms des documents plutôt que de les traduire ou de les
normaliser.** Écrire `establishment` ou `booking` au lieu de `etablissement` ou `reservation`
introduit une divergence entre le code et les documents de référence.

Chaînes visibles par l'utilisateur : **jamais en dur**, clés i18n **fr et en**, fr par défaut.

## Sources de vérité — ordre de préséance

En cas de contradiction, trancher dans cet ordre :

1. `.specify/memory/constitution.md` — 12 principes non négociables, **26 portes de CI**
   bloquantes (P-01 à P-23, dont P-01b, P-05b et P-21b). **À lire avant toute décision d'architecture.**
   Sa section « Couverture des portes » est née de portes vertes défectueuses aux cycles 001 et
   002 : *un test négatif prouve qu'une porte sait échouer, il ne prouve pas qu'elle regarde
   tout* — et une porte dont la cible est vide passe toujours.
2. `docs/cadrage-v1.md` — périmètre, modèle d'entité, fiscalité, classes hors-ligne, déploiement,
   provisions §14.
3. `docs/user-stories-v1.md` — critères d'acceptation, priorités P0/P1/P2/PROVISION, Definition
   of Done (§0.4), ordre des tranches (§0.5), **récapitulatif des paramètres d'établissement**.
4. `docs/registre-classes-offline.md` — classe A/B/C/D de chaque opération. **Une entité absente
   de ce registre n'est pas implémentable.**
5. `docs/versions-gelees.md` — versions épinglées, URL des registres, commandes de vérification.
6. `docs/design/tokens.md`, puis `docs/design/mouvement.md` — valeurs de design.
7. `docs/design/html/`, `fondation/`, `proto/`, `documents/` — référence normative d'écran.
   **`docs/design/derivation.md`** dit de quel motif hérite chacun des 31 écrans non maquettés
   (42 écrans en tout) ; **`docs/design/lexique.md`** donne le vocabulaire utilisateur. Les deux
   sont opposables : un écran hors des deux ne se code pas, un terme technique hors du lexique
   n'atteint jamais l'interface.
8. `docs/Kaya_Vision_Plateforme.md` — **fermé jusqu'au jalon J1**, sans effet sur le MVP.

## L'architecture en une page

**L'entité centrale est l'établissement, pas l'hôtel.** Un établissement active les modules dont
il a besoin. Un maquis seul, un bar seul, un pressing seul, une résidence meublée seule sont des
établissements valides. **Aucun crate partagé ne suppose l'existence d'un hébergement ni d'un
point de vente.**

Monolithe modulaire Rust, microservices-ready. **Trois familles de crates, hiérarchie stricte :**

```
socle/       etablissements comptes caisse fiscalite documents
             synchronisation pilotage editeur metriques      → dépend de socle/ SEULEMENT
capacites/   stocks (les autres non implémentées)            → dépend de socle/
verticales/  hebergement restauration bar pressing          → dépend de socle/ et capacites/
```

**Le socle ne connaît ni « chambre », ni « unité louable », ni « séjour »** — il connaît
`article_vendable` et `ressource_reservable`. Tout le spécifique hôtelier vit dans
`verticales/hebergement`. Un test de CI échoue si un crate de `socle/` dépend de `verticales/`.
C'est ce qui garde le produit extensible ; sans cette règle, l'hôtellerie contamine le noyau.

**Module d'activité ≠ capacité** — deux référentiels distincts, tous deux en table. Le module est
la verticale (`HEBERGEMENT`, `RESTAURATION`, `BAR`, `PRESSING`, `SALLE_REUNION`), la capacité est
le transverse (`STOCK`, `LIVRAISON`, `PRODUCTION`, `COMMERCE_EN_LIGNE`, `FIDELITE`, `DEVIS`,
`COMPTES_CLIENTS`). Seule `STOCK` au profil `SIMPLE` est implémentée ; **toute autre valeur est
refusée explicitement, jamais ignorée.**

**Un schéma PostgreSQL par module.** Aucune requête ne joint deux schémas de modules ; les
lectures inter-modules passent par un trait exposé. Aucune transaction ne couvre deux modules —
les opérations inter-modules sont des sagas avec compensation explicite. **Toute transition d'état
écrit un événement outbox dans la même transaction** ; l'outbox est un grand livre permanent
(rétention illimitée, immuable, charge utile financière dénormalisée), consommé par un worker
in-process. Aucune file de messages externe au MVP.

Le crate `domain` (moteur fiscal, barèmes, validation, types) est partagé entre l'API, le nœud de
site et la coquille Tauri : **une seule implémentation du calcul de la taxe de nuitée.**

Côté application : **une seule** application Nuxt 4 + Tauri v2 pour tous les rôles, desktop /
Android / iOS. Rôles **cumulables** (permissions = union). Accueil = tuiles filtrées par
permission. Un module inactif est **absent**, jamais grisé. Aucun `window.__TAURI__` dans un
composant — tout passe par `PlatformAdapter`.

## Pièges spécifiques à ce projet

Ceux-ci coûtent une migration ou une refonte s'ils sont manqués. Ils ne se devinent pas.

- **Montants = entiers d'unité mineure. Quantités = `NUMERIC`, jamais entier.** Un hôtel vend
  1 bière, une quincaillerie 2,3 m de fer, une boulangerie 47,5 kg de farine. Passer d'entier à
  décimal après mise en production imposerait de migrer toutes les lignes.
- **Une occupation est un intervalle `[début, fin)` en timestamp avec fuseau, JAMAIS une paire de
  dates.** Le marché pratique massivement le passage horaire et la demi-journée. Disponibilité
  garantie par `EXCLUDE USING gist (unite_id WITH =, periode WITH &&)`, **pas par un verrou
  applicatif**.
- **Le statut d'occupation d'une unité est dérivé**, jamais posé à la main. Seul le sous-statut
  ménage est modifiable. Les confondre produit des doubles attributions.
- **Tout calcul de durée, de taxe et toute clôture s'appuient sur l'horodatage d'autorité
  serveur**, jamais sur l'horloge d'un terminal.
- **Les polices embarquées sont sous licence, et leur attribution est due.** Trois œuvres tierces
  partent dans le binaire : Archivo et Chivo Mono (OFL 1.1), Phosphor (MIT). Leurs textes vivent
  dans `app/assets/fonts/*-LICENCE.txt` — **copies exactes de l'amont, jamais retouchées** — et
  sont importés en clair par `app/core/licences/`, ce qui les fait entrer dans le paquet. Ce qui a
  été modifié est déclaré dans `app/assets/fonts/MODIFICATIONS.md`, l'inventaire dans
  `docs/conformite/licences-tierces.md`, et la porte **P-21b, contrôle 5**, refuse toute police
  sans licence ni avis de copyright. **Ni Archivo ni Chivo Mono ne déclarent de Reserved Font
  Name** : c'est ce qui permet de modifier leur `cmap` en gardant le nom de famille. Une police à
  nom réservé imposerait de renommer la famille — donc de toucher aux jetons `--font-*`.
- **L'API FNE n'a aucune clé d'idempotence.** L'état `INDETERMINEE` (timeout) n'est **jamais**
  rejoué automatiquement — rapprochement manuel obligatoire.
- **Les `id` d'items retournés par la certification FNE sont persistés.** Sans eux, aucun avoir
  n'est possible. Erreur irrattrapable a posteriori.
- **Documents opérationnels et fiscaux sont deux agrégats étanches** : deux numérotations, deux
  cycles de vie. Tout document opérationnel porte « Document non fiscal — ne tient pas lieu de
  facture ».
- **Le HTML de `docs/design/html/` n'est JAMAIS copié vers `app/`** — c'est une cible, pas une
  source : autonome, non sémantique, sans i18n, sans RBAC. On lit ses valeurs, on réimplémente.
  **Seule exception** : `docs/design/theme.css` est copié tel quel dans `app/assets/css/`.
- **Tailwind 4 d'abord, CSS en dernier recours.** Mode sombre par la variante `dark:`, jamais une
  seconde palette. Aucune classe personnalisée, aucun style en ligne.
- **Aucune opération de classe B, C ou D atteignable hors ligne.** Vérifier
  `docs/registre-classes-offline.md` avant d'écrire un chemin de code. L'interface annonce
  **immédiatement** une action indisponible — jamais de grisé silencieux, jamais de file « au cas
  où ».
- **Le verrouillage par adresse MAC n'est jamais implémenté** (iOS/Android randomisent la MAC).
  À la place : enrôlement d'appareil par paire de clés Keystore/Keychain.
- **Le géorepérage n'est jamais bloquant** sur une action critique — alerte seulement.

## Versions

`docs/versions-gelees.md` fait foi. Deux règles absolues :

- **Ne jamais proposer un numéro de version de mémoire.** Vérifier sur le registre officiel et
  citer l'URL. Les commandes de vérification sont au §5 du document.
- **Épinglage exact** — jamais `^`, `~` ou un intervalle. Lockfiles commités, `Cargo.lock`
  inclus même pour un binaire.

**AJOUTER UNE DÉPENDANCE EST LIBRE. IL N'Y A PAS DE PERMISSION À DEMANDER.** Depuis le gel 1.0.14,
un cycle qui a besoin d'une bibliothèque absente l'ajoute, **en cours de cycle**, et l'inscrit au
§3.1 ou §3.2 **dans le même changement** — jamais reportée à une revue. Trois obligations, aucune
n'étant une autorisation :

1. **épinglage exact et lockfile commité** — la règle ne connaît aucune exception ;
2. **un commentaire au-dessus de la ligne du manifeste** : le rôle, l'URL du registre interrogé, la
   date. Les cycles le font déjà spontanément et bien ;
3. **dire pourquoi ce qui est déjà là ne suffit pas.** Pas pour obtenir un accord — pour que la
   question soit posée. L'arbitrage `aes-gcm` du cycle 006, qui a examiné et écarté `ring` pourtant
   déjà présent transitivement, est le modèle.

**Ce qui n'est PAS libre**, et la distinction est nette :

- **monter une version déjà gelée** — groupé, mensuel, hors incrément (principe XI) ;
- **toucher aux dix briques du §2** (Rust, Actix, sqlx, utoipa, Nuxt, Tailwind, Tauri, PostgreSQL,
  Redis, Garage) — y compris en mineur : monter `sqlx` réécrit les macros de chaque requête ;
- **introduire un second membre d'une famille déjà pourvue** — `chrono` quand `time` est là,
  `anyhow` dans un crate de bibliothèque quand `thiserror` est là. Le tableau est au **§3.4**, et
  une famille qui n'y figure pas est une famille **non encore rencontrée** : le cycle qui l'ouvre
  tranche pour tout le dépôt et inscrit sa ligne.

**Pourquoi ce changement.** L'ancienne règle — *« la revue est mensuelle et groupée, jamais au fil
de l'eau »* — était plus stricte que le principe XI, qui ne parle que de **montées**. Elle a produit
deux dégâts : sept crates épinglées dans les manifestes et absentes du gel pendant six semaines
(gel 1.0.13), et une contrainte de gouvernance entrée dans un raisonnement de **conception** —
`client/repli.rs` cite en premier argument le fait qu'`unicode-normalization` « n'est pas au gel ».
Une règle qui produit la dette qu'elle prétend organiser se change ; elle ne se respecte pas mieux.

Deux points à connaître pour ne pas perdre une journée :

- **sqlx 0.9.0** impose `AssertSqlSafe` sur toute requête non littérale et modifie la sortie des
  macros `query!()`. **Tout extrait trouvé en ligne vise 0.8.x et ne compilera pas.** Le module
  doré, écrit à la main contre 0.9.0, est le patron de référence — l'écrire **avant** toute
  génération assistée.
- **Cible de production : Docker sur VPS Contabo (`linux/amd64`).** Le poste de développement est
  `arm64`. Les images Postgres/Redis/Garage sont multi-arch, mais **le binaire Rust ne l'est
  pas** : construction de production dans Docker pour `linux/amd64`, jamais par copie d'un binaire
  local.

- **`cargo sqlx prepare` DÉTRUIT le cache silencieusement s'il ne recompile rien.** Il ne collecte
  que les requêtes des unités **effectivement (re)compilées** : lancé sur un build à jour, il
  annonce « no queries found » et **vide `.sqlx`**. Sans `-- --all-targets`, il ignore en outre
  les binaires et les tests. Constaté le 2026-08-01 : la commande perdait les **9 requêtes du
  binaire `seeds`**, qu'aucun `cargo clean -p` ni `touch` n'a suffi à faire réémettre.
  **Après tout `prepare`, vérifier les deux, dans cet ordre** — le second seul ne suffit pas, un
  cache amputé d'une requête inutilisée par le check passerait :

  ```sh
  git status --short backend/.sqlx    # AUCUNE suppression ; que des ajouts
  SQLX_OFFLINE=true DATABASE_URL= cargo check --workspace --all-targets --locked
  ```

  En cas de suppression, `git checkout backend/.sqlx` restaure les entrées commitées **sans
  toucher** aux fichiers non suivis, donc sans perdre les requêtes nouvelles.

  **La cause a été trouvée au cycle 003, et il faut DEUX passes.** `cargo sqlx prepare` ne
  collecte que les requêtes des cibles que son `cargo check` compile réellement, et le répertoire
  d'où on le lance décide de ce qu'il voit :

  | Lancé depuis | Ce qu'il collecte | Ce qu'il PERD |
  |---|---|---|
  | `backend/` | le paquet racine et ses tests d'intégration | les **binaires** de `kaya-api` — `seeds`, `contrat` |
  | `backend/api/` | les binaires et la bibliothèque de `kaya-api` | les tests de `backend/tests/` |

  Aucun `cargo clean`, aucun `touch`, aucun `--all-targets` n'y change quoi que ce soit : ce n'est
  pas un problème de cache de compilation. La procédure qui marche conserve les deux moissons
  **hors** de `.sqlx` entre les passes, puisque chaque `prepare` réécrit le répertoire entier :

  ```sh
  cd backend
  rm -rf /tmp/sqlx-a /tmp/sqlx-b && mkdir -p /tmp/sqlx-a /tmp/sqlx-b

  cargo sqlx prepare --workspace -- --all-targets            # passe 1 — tests
  git status --short .sqlx | grep '^??' | awk '{print $2}' | xargs -I{} cp {} /tmp/sqlx-a/
  git checkout .sqlx

  (cd api && cargo sqlx prepare --workspace -- --all-targets)  # passe 2 — binaires
  git status --short .sqlx | grep '^??' | awk '{print $2}' | xargs -I{} cp {} /tmp/sqlx-b/
  git checkout .sqlx

  cp /tmp/sqlx-a/*.json /tmp/sqlx-b/*.json .sqlx/
  ```

  Puis les deux contrôles habituels. Le symptôme, si l'on se contente d'une passe : le check
  hors ligne échoue sur `no cached data for this query` **dans les cibles que l'autre passe
  couvrait**, alors que `prepare` vient d'annoncer avoir écrit le cache.

- **Le contrôle hors ligne ne prouve rien s'il ne recompile rien** — constaté au cycle 004.
  `SQLX_OFFLINE=true cargo check` lancé sur un build à jour affiche `Finished` en une seconde
  **sans consulter `.sqlx`** : les macros ne sont pas réévaluées, donc un cache vide passerait.
  Le contrôle 1 (`git status`) reste probant en toutes circonstances ; le second exige de forcer
  la réévaluation :

  ```sh
  grep -rl "sqlx::query" --include="*.rs" crates api tests | xargs touch
  SQLX_OFFLINE=true DATABASE_URL= cargo check --workspace --all-targets --locked
  ```

  Sans le `touch`, le cycle 004 a vu le contrôle 2 passer au vert sur un cache **réellement
  périmé** — les erreurs sont apparues au premier fichier modifié pour une autre raison.

## Flux de travail

Le dépôt utilise **Spec Kit** (skills `speckit-*` dans `.claude/skills/`).

- **Un module = un epic = un cycle** : `/speckit-specify` sur la section du module, puis
  `/speckit-plan` en pointant le cadrage §13 (pile) et §14 (provisions) comme contraintes, puis
  `/speckit-tasks`, puis `/speckit-implement`.
- **Implémenter par tranches verticales**, jamais module par module de bout en bout. Ordre fixé
  au §0.5 de `docs/user-stories-v1.md` : T1 colonne vertébrale → T2 services et note →
  T3 fiscalité et clôture → T4 mobile et QR → T5 pilotage.
- Amender la constitution **uniquement** via `/speckit-constitution` — jamais à la main.
- Toute story doit satisfaire les 10 points de la Definition of Done (`docs/user-stories-v1.md`
  §0.4) et les portes P-01 à P-20 de la constitution.

## Commandes

```sh
# Services de développement — Postgres 18.4, Redis 8.8.1, Garage 2.3.0
docker compose -f infra/compose.yml up -d        # services db, cache, objets
bash scripts/dev/preparer-base.sh                # rôles, schémas, migrations
bash scripts/dev/preparer-stockage.sh            # amorçage des buckets Garage

# Backend (depuis backend/)
cargo test --workspace                           # 15 tests d'intégration
cargo test --test isolation_tenant                # un seul fichier de test
cargo sqlx prepare --workspace -- --all-targets   # cache de requêtes — LIRE L'AVERTISSEMENT
SQLX_OFFLINE=true cargo check --workspace --all-targets --locked   # comme l'image

# Portes de CI, exécutables une par une depuis la racine
pnpm porte:p01   # client TS régénéré sans diff       pnpm porte:p15   # pont natif confiné
pnpm porte:p02   # migration appliquée non modifiée   pnpm porte:p19   # maquette non copiée
pnpm porte:p04   # pas de jointure inter-schémas      pnpm porte:p20   # versions épinglées
pnpm porte:p05b  # pas de purge de l'outbox           pnpm porte:p21   # rien d'un hôte externe
pnpm porte:p10   # entiers / NUMERIC                  pnpm porte:p21b  # déclaré = embarqué
pnpm porte:p22   # PARCOURS RÉEL — chaque route s'ouvre, en direct ET par navigation, deux thèmes
                 #   exige l'API, la base et les seeds ; le script le vérifie et le dit
pnpm porte:p22:negatif   # prouve que P-22 sait échouer (casse le layout, constate, remet)

# P-23 · PROVENANCE DE L'INSTANT — et le balayage hors ligne de FR-005b
cargo test --test horodatage_autorite     # aucun calcul ne s'appuie sur horodatage_client
cargo test --test outillage_classes       # toute entité implémentée est EXERCÉE, pas seulement déclarée
pnpm exec playwright test tests-e2e/hors-ligne.spec.ts   # exige l'API ; deux moteurs

# ⚠️ LA SUITE BACKEND ET LE E2E NE COEXISTENT PAS. `exiger_grand_livre_sans_consommateur_
# concurrent` refuse de dérouler les tests d'outbox quand un worker de publication tourne hors
# de `cargo test` — c'est-à-dire quand l'API est allumée, ce que le e2e exige. Séquencer, et
# arrêter l'API PAR PORT : `lsof -ti:8080 | xargs kill`.
#
# ⚠️ LE LIMITEUR DE TENTATIVES PUNIT LES EXÉCUTIONS RAPPROCHÉES. Dix connexions par identifiant
# sur une fenêtre GLISSANTE de cinq minutes, réussies comprises — et chaque essai la repousse.
# Le refus est INDISCERNABLE d'un mot de passe faux (FR-012) : ne pas chercher ailleurs, attendre.
pnpm generer:client                               # types TS depuis openapi.json

# ESLint vit à la RACINE et couvre app/ ET web/qr ET web/console — les deux surfaces publiques
# sont HORS Tauri, donc l'endroit où la porte P-15 compte le plus. `porte:p15` ajoute le décompte
# des fichiers réellement analysés par arbre : une cible vide passerait autrement.
pnpm lint                                         # eslint . depuis la racine

# Application (depuis app/)
pnpm dev · pnpm build · pnpm test · pnpm lint:tokens · pnpm test:i18n
pnpm --filter @kaya/app polices:generer   # + icones:generer — `--verifier` en CI

# Image de production — TOUJOURS pour linux/amd64, jamais un binaire local
docker buildx build --platform linux/amd64 -f infra/Dockerfile.api -t kaya-api:<tag> .
```

Les mesures de temps de compilation se font **dans le conteneur**, seul endroit où `mold` est
actif — il n'existe pas sur macOS.

## Décisions ouvertes qui bloqueraient si elles étaient ignorées

- ~~**O-01**~~ — **TRANCHÉE le 2026-08-03, option (a)** : `client` reste en **classe C**, le réseau
  est exigé pour créer une fiche nouvelle. La friction résiduelle est **écrite** au §12 du registre
  plutôt que tue — l'arrivée d'un client inconnu hors ligne reste impossible, et c'est assumé.
- **O-02** — classe de `mouvement_stock` (A ou B), décision B-05 du cadrage, à trancher avec le
  pilote.
- **O-03** — crate d'accueil de la surface QR, transverse à `restauration` et `bar`, absente des
  quatre verticales.
- **B-02** — traitement fiscal de la taxe de nuitée sur le passage et la demi-journée. Aucune
  valeur en dur en attendant : c'est un paramètre par formule.

Les autres décisions ouvertes sont à l'annexe B de `docs/cadrage-v1.md`. **B-01** (localisation
de l'hébergement) est tranchée de fait par le choix Contabo — serveur en Europe, ce qui soulève
le transfert transfrontalier ARTCI pour les pièces d'identité de clients.
