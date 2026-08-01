# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## État du dépôt — lire en premier

**Le socle et les établissements sont en place** — cycles 001 (TRX) et 002 (ETB) livrés :
18 crates Rust, 13 migrations, 98 tests backend, 60 tests front, 8 portes scriptées, l'écran `G1`,
et une image de production construite et exercée.

**Le patron de référence est `docs/module-dore.md`** (451 lignes) : une tranche verticale écrite
à la main contre sqlx 0.9. **Le lire avant d'écrire du Rust** — tout extrait trouvé en ligne vise
sqlx 0.8 et ne compilera pas.

**Les deux dettes du cycle 002 sont soldées** — lire avant de coder du front :

1. **Le patron d'écriture front existe, sur UNE opération.** La bascule d'un service (ETB-02) est
   câblée de bout en bout et documentée dans **`docs/module-dore.md`, « La septième couche »** :
   appel typé, squelette de chargement, refus métier en langue utilisateur, validation au champ,
   action **absente** sans permission, refus immédiat hors ligne (classe C), rafraîchissement sans
   rechargement. **Les vingt autres opérations d'écriture suivent ce patron, cycle par cycle** —
   le lire avant d'en brancher une, ne pas réinventer.
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

État par tranche : **T1 en cours** (TRX et ETB faits ; restent CPT, HEB, SYN, SEJ-1).

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

1. `.specify/memory/constitution.md` — 12 principes non négociables, **24 portes de CI**
   bloquantes (P-01 à P-21, dont P-01b, P-05b et P-21b). **À lire avant toute décision d'architecture.**
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

- **O-01** — `client` / `personne` sont en classe C, ce qui rend le check-in d'un **client
  inconnu** impossible hors ligne, même en mode nœud de site. À trancher **avant SEJ-02**.
- **O-02** — classe de `mouvement_stock` (A ou B), décision B-05 du cadrage, à trancher avec le
  pilote.
- **O-03** — crate d'accueil de la surface QR, transverse à `restauration` et `bar`, absente des
  quatre verticales.
- **B-02** — traitement fiscal de la taxe de nuitée sur le passage et la demi-journée. Aucune
  valeur en dur en attendant : c'est un paramètre par formule.

Les autres décisions ouvertes sont à l'annexe B de `docs/cadrage-v1.md`. **B-01** (localisation
de l'hébergement) est tranchée de fait par le choix Contabo — serveur en Europe, ce qui soulève
le transfert transfrontalier ARTCI pour les pièces d'identité de clients.
