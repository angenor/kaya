# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## État du dépôt — lire en premier

**Il n'y a aucun code.** Ni `Cargo.toml`, ni `package.json`, ni `compose.yml`, ni un seul `.rs`
ou `.vue`. Le dépôt est en **phase 0** : documentation produit, gouvernance et gel des versions.
Ne cherchez pas de source, ne supposez pas d'arborescence existante.

Le premier code viendra du cycle Spec Kit sur le module **TRX** (tranche T1). Tout ce qui suit
décrit ce qui est **arrêté**, pas ce qui existe.

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

1. `.specify/memory/constitution.md` — 12 principes non négociables, 20 portes de CI bloquantes.
   **À lire avant toute décision d'architecture.**
2. `docs/cadrage-v1.md` — périmètre, modèle d'entité, fiscalité, classes hors-ligne, déploiement,
   provisions §14.
3. `docs/user-stories-v1.md` — critères d'acceptation, priorités P0/P1/P2/PROVISION, Definition
   of Done (§0.4), ordre des tranches (§0.5), **récapitulatif des paramètres d'établissement**.
4. `docs/registre-classes-offline.md` — classe A/B/C/D de chaque opération. **Une entité absente
   de ce registre n'est pas implémentable.**
5. `docs/versions-gelees.md` — versions épinglées, URL des registres, commandes de vérification.
6. `docs/design/tokens.md`, puis `docs/design/mouvement.md` — valeurs de design.
7. `docs/design/html/`, `fondation/`, `proto/`, `documents/` — référence normative d'écran.
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

Aucune commande de build n'existe encore. Celles-ci sont **arrêtées par la constitution et le
gel**, et seront créées par le cycle TRX :

```sh
cargo sqlx prepare          # requêtes vérifiées à la compilation — porte P-18
cargo tauri android|ios     # builds mobiles (MOB-01)
```

Aujourd'hui, seule la vérification des versions est exécutable — voir §5 de
`docs/versions-gelees.md` (interrogation de crates.io, npm, postgresql.org, Docker Hub).

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
