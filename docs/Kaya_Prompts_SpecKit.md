# Kaya — Prompts Spec Kit (prêts à coller)

*Compagnon d'exécution du Cadrage v1 et des User Stories v1 — Développement solo avec Claude Code + GitHub Spec Kit*

---

## 0. Préparation (une seule fois)

```bash
uvx --from git+https://github.com/github/spec-kit.git specify init kaya --ai claude
cd kaya
```

**État attendu du dépôt avant le cycle 1** — la maquette est déjà déposée :

```
CLAUDE.md                      guide de session — à lire en premier
.specify/memory/constitution.md  **la version et le nombre de portes se lisent DANS le fichier**
docs/
├── cadrage-v1.md              source de vérité produit et technique
├── user-stories-v1.md         source de vérité fonctionnelle, priorités
├── registre-classes-offline.md v1.0.0 — 157 opérations classées A/B/C/D
├── versions-gelees.md         v1.0.4 — versions épinglées + URL des registres
├── Kaya_Prompts_SpecKit.md    ce fichier
├── Kaya_Design.md             ✅ présent — matrice de dérivation, lexique
├── Kaya_Vision_Plateforme.md  fermé jusqu'au jalon J1
├── module-dore.md             ✅ patron sqlx 0.9 — À LIRE avant tout Rust
└── design/
    ├── theme.css              ⚠️ copié vers app/assets/css/ AU CYCLE 1
    ├── tokens.md              valeurs curées — PRIME sur tout export
    ├── composants.md          les composants canoniques — composants.md fait foi sur le nombre
    ├── mouvement.md           durées, courbes, sept patrons
    ├── styleguide.html        tous les composants, tous états, clair + sombre
    ├── README.md              ce qui se copie, ce qui se lit, valeurs arbitraires
    ├── derivation.md          ✅ 30 écrans dérivés + leur motif — NORMATIF
    ├── lexique.md             ✅ vocabulaire utilisateur — NORMATIF
    ├── html/                  29 fichiers {code}-{nom}[-{etat}].html — 11 écrans — RÉFÉRENCE
    ├── fondation/             directions, mouvement, plaisir, difficiles, illustrations
    ├── documents/             D1-D5 tickets, D6 note provisoire, D7 facture
    ├── proto/                 proto-0 à proto-6, prototypes animés
    └── notes-terrain.md       ⚠️ rempli à l'atelier d'Abengourou
```

**État au 2026-07-30 — aucun blocage design.** Les deux références opposables existent
désormais **à leur chemin annoncé**, extraites de `Kaya_Design.md` :

- **`docs/design/derivation.md`** — les **30 écrans codés sans maquette**, chacun avec le motif
  dont il hérite et ce qui change. Avec les 11 écrans maquettés (29 fichiers d'états), le
  produit couvre **41 écrans**.
- **`docs/design/lexique.md`** — 15 concepts internes et leur formulation utilisateur
  (« certification FNE » → « envoi aux impôts »), plus la procédure d'ajout d'une entrée.

Ces deux tableaux ont été **déplacés, pas recopiés** : `Kaya_Design.md` §6 et partie V y
renvoient et ne les dupliquent pas. Une copie aurait divergé — principe I de la constitution.

**Cinq documents ont été ajoutés depuis la rédaction de ce fichier** et font foi :
`.specify/memory/constitution.md` (**v1.2.0**), `docs/registre-classes-offline.md` (v1.0.0),
`docs/versions-gelees.md` (**v1.0.4**), `docs/module-dore.md` (patron sqlx 0.9) et `CLAUDE.md`.

**Le cycle 001 (TRX) est fusionné** : 18 crates, 6 migrations, 15 tests, 9 portes scriptées, image
de production exercée. Le cycle 002 (ETB) est spécifié. L'arborescence du §0.1 **existe** — ce
n'est plus une cible.

Le paquet installé utilise le **tiret** comme séparateur (`/speckit-specify`), conformément à
`.specify/integration.json` → `invoke_separator: "-"`. Tous les prompts de ce fichier l'emploient.

### 0.1 Monorepo — arborescence de référence (créée au cycle 1)

```
kaya/
├── backend/                      # workspace Rust (Cargo workspace)
│   ├── crates/
│   │   ├── domain/               # types, règles, moteur fiscal, barèmes — PARTAGÉ
│   │   ├── socle/                # ← ne dépend QUE de socle/. Jamais de verticales/
│   │   │   ├── etablissements/   # tenants, établissements, modules, capacités, config
│   │   │   ├── comptes/          # comptes, rôles cumulables, appareils, audit
│   │   │   ├── caisse/           # shifts, encaissements, clôture
│   │   │   ├── fiscalite/        # adaptateurs de juridiction, FNE, taxes
│   │   │   ├── documents/        # opérationnels et fiscaux, numérotation
│   │   │   ├── synchronisation/  # classes offline, file, réconciliation
│   │   │   ├── pilotage/         # tableaux de bord, audit, rapports
│   │   │   ├── editeur/          # console SaaS, abonnements
│   │   │   └── metriques/
│   │   ├── capacites/            # ← dépend de socle/. Transverses aux verticales
│   │   │   └── stocks/           # (production, livraison, commerce : NON IMPLÉMENTÉES)
│   │   └── verticales/           # ← dépend de socle/ et capacites/
│   │       ├── hebergement/      # unités louables, formules, disponibilité, séjours,
│   │       │                     #   réservations — TOUT le spécifique hôtelier
│   │       ├── restauration/     # catalogue, tables, commandes, QR
│   │       ├── bar/
│   │       └── pressing/         # bons de dépôt, cycle de retrait
│   ├── api/                      # binaire Actix — assemble les crates, expose utoipa
│   ├── node/                     # binaire nœud de site (même domain) — incrément 3
│   └── migrations/               # migrations sqlx versionnées + seeds/
├── app/                          # Nuxt 4 + Tauri v2 — APPLICATION UNIQUE
│   ├── modules/                  # reception/ pdv/ caisse/ stocks/ direction/ config/
│   ├── core/                     # auth, rbac, i18n, thème, sync, PlatformAdapter
│   └── src-tauri/                # coquille Rust + plugins natifs Swift/Kotlin
├── web/                          # Nuxt 4 — surfaces publiques
│   ├── qr/                       # page de commande par QR (SSR)
│   └── console/                  # console éditeur (ssr:false)
├── clients/ts/                   # client API généré — JAMAIS édité à la main
├── infra/                        # docker-compose dev, paquet auto-hébergé, sauvegardes
├── docs/                         # cadrage-v1.md, user-stories-v1.md,
│   │                             # registre-classes-offline.md, taxonomie-evenements.md
│   └── design/                   # theme.css (SEUL fichier copié → app/assets/css/),
│                                 # html/ (11 écrans en 29 fichiers d'états, RÉFÉRENCE — jamais copiés dans app/),
│                                 # tokens.md (PRIME sur tout export), composants.md,
│                                 # mouvement.md, styleguide.html, README.md,
│                                 # fondation/, documents/, proto/,
├── specs/                        # généré par Spec Kit (un dossier par cycle)
└── .github/workflows/            # CI filtrée par chemins + génération du client (échec sur diff)
```

**Cycle par module** (un module = un cycle complet, ordre du §3) :

```
/speckit-constitution   (une seule fois, §1)
puis : /speckit-specify (§3) → /speckit-clarify (§2.1) → /speckit-plan (§2.2)
→ /speckit-tasks (§2.3) → /speckit-analyze (§2.4) → /speckit-implement (§2.5)
→ commit / merge
```

---

## 1. Constitution (à coller une seule fois)

> ✅ **CONSOMMÉ le 2026-07-30 — ne pas recoller.** La constitution est ratifiée et vit
> dans `.specify/memory/constitution.md` (12 principes ; **le nombre de portes se lit dans le fichier**, il a changé neuf fois
> P-01 à P-20 dont P-05b, section « Couverture des portes », gouvernance). **C'est elle qui fait foi, pas le
> prompt ci-dessous**, conservé comme archive de ce qui a été soumis.
>
> Trois écarts constatés à la ratification et corrigés dans le fichier ratifié :
> `docs/design/html/` contient **29 fichiers pour 11 écrans**, non 27 ; `F2-registre-grave`
> et `S2-registre-grave` sont **deux écrans distincts** (document fiscal `INDETERMINEE`
> vs consommation orpheline), non un doublon ; `tokens.md` et `theme.css` **concordent**
> — 71 tokens vérifiés, aucune divergence de valeur.
>
> Pour amender : `/speckit-constitution` avec la seule modification voulue, jamais une
> réécriture complète, jamais à la main.

```
/speckit-constitution

Projet : Kaya — plateforme de gestion pour établissements d'hébergement et de
service en Afrique. Pilote : Résidence Hôtel Deloria, Abengourou, Côte d'Ivoire.
Développeur solo. Monorepo unique.

L'ENTITÉ CENTRALE EST L'ÉTABLISSEMENT, PAS L'HÔTEL. Un établissement active les
MODULES D'ACTIVITÉ dont il a besoin (hébergement, restauration, bar, pressing,
salle de réunion). Un maquis seul, un bar seul, un pressing seul et une résidence
meublée seule sont des établissements valides. AUCUN crate partagé ne doit
supposer qu'un établissement possède de l'hébergement, ni qu'il possède un point
de vente.

Documents produit de référence : docs/cadrage-v1.md et docs/user-stories-v1.md —
en cas de doute, ces documents priment sur toute supposition.

Principes non négociables :

1. SOURCES DE VÉRITÉ. (a) Le contrat OpenAPI est généré par utoipa depuis le code
   Actix ; le client TypeScript est généré depuis ce contrat en CI, jamais écrit
   à la main ; un diff de client non commité fait échouer le build. (b) Le schéma
   PostgreSQL n'est modifié que par migrations sqlx versionnées ; une migration
   appliquée n'est jamais modifiée — on en crée une nouvelle ; les seeds sont
   rejouables à part. (c) Tout paramètre métier qualifié de « paramétrable » vit
   dans la configuration d'établissement (héritage tenant → établissement →
   module → point de vente, avec surcharge), jamais en dur dans le code. Le
   récapitulatif des paramètres en fin de docs/user-stories-v1.md fait foi.

2. ARCHITECTURE. Monolithe modulaire Rust, microservices-ready : un crate par
   domaine, interfaces par traits, UN SCHÉMA POSTGRES PAR MODULE. Aucune requête
   ne joint deux schémas de modules différents — les lectures inter-modules
   passent par un trait exposé. Aucune transaction SQL ne couvre deux modules ;
   les opérations inter-modules sont des sagas simples avec compensation
   explicite. Toute transition d'état écrit un événement outbox dans la même
   transaction. AUCUN service n'est extrait au MVP, AUCUNE file de messages
   externe n'est introduite — l'outbox est consommé par un worker in-process.
   Le crate domain (moteur fiscal, barèmes, validation, types) est partagé entre
   l'API, le nœud de site et la coquille Tauri : UNE SEULE implémentation du
   calcul de la taxe de nuitée. Redis ne porte que de l'éphémère reconstructible
   (sessions, file FNE, verrous, rate-limit, cache). Garage via API S3. Postgres
   est la seule vérité durable.
   TROIS FAMILLES DE CRATES, HIÉRARCHIE STRICTE : socle/ (etablissements, comptes,
   caisse, fiscalite, documents, synchronisation, pilotage, editeur, metriques) ne
   dépend QUE de socle/ ; capacites/ (stocks — les autres non implémentées) dépend
   de socle/ ; verticales/ (hebergement, restauration, bar, pressing) dépend de
   socle/ et capacites/. UN TEST DE CI ÉCHOUE SI UN CRATE DE socle/ DÉPEND D'UN
   CRATE DE verticales/. LE SOCLE NE CONNAÎT NI « chambre », NI « unité louable »,
   NI « séjour » : il connaît article_vendable et ressource_reservable. Tout le
   spécifique hôtelier vit dans verticales/hebergement. C'est ce qui garde le
   produit extensible à d'autres activités ; sans cette règle, l'hôtellerie
   contamine le noyau en trois cycles.
   MODULE D'ACTIVITÉ ≠ CAPACITÉ : deux référentiels distincts, tous deux en table.
   Le module est la verticale (HEBERGEMENT, RESTAURATION, BAR, PRESSING,
   SALLE_REUNION) ; la capacité est le transverse (STOCK, LIVRAISON, PRODUCTION,
   COMMERCE_EN_LIGNE, FIDELITE, DEVIS, COMPTES_CLIENTS). Un module déclare les
   capacités qu'il consomme. Seule STOCK au profil SIMPLE est implémentée ; toute
   autre valeur est REFUSÉE EXPLICITEMENT, jamais ignorée.

3. MULTI-TENANT. Chaque table porte tenant_id. RLS ENABLE **ET** FORCE sur toutes
   les tables, avec un rôle applicatif distinct du propriétaire des tables.
   SET LOCAL app.current_tenant posé DANS CHAQUE TRANSACTION, jamais à l'ouverture
   de connexion. Un test de CI échoue si une table du schéma n'a aucune politique
   RLS. Un test d'isolation vérifie que le tenant A ne lit ni n'écrit aucune ligne
   du tenant B, sur chaque endpoint.

4. TEMPS ET DISPONIBILITÉ. Une occupation est un intervalle [début, fin) en
   TIMESTAMP AVEC FUSEAU DE L'ÉTABLISSEMENT, JAMAIS une paire de dates — le marché
   pratique massivement le passage horaire et la demi-journée. La disponibilité
   est garantie par une contrainte d'exclusion PostgreSQL
   (EXCLUDE USING gist sur unite_id + tstzrange), pas par un verrou applicatif.
   Le temps de remise en état est intégré à l'intervalle d'indisponibilité.
   Toute logique métier, tout calcul fiscal, toute clôture et TOUT CALCUL DE DURÉE
   DE PASSAGE s'appuient exclusivement sur l'horodatage d'autorité serveur, jamais
   sur l'horloge d'un terminal.

5. ARGENT ET FISCALITÉ. Tous les MONTANTS sont des ENTIERS en unités mineures +
   code ISO 4217 porté par l'établissement (XOF, 0 décimale). En revanche toute
   QUANTITÉ — ligne de vente, mouvement de stock — est en NUMERIC, JAMAIS en
   entier : un hôtel vend 1 bière, une quincaillerie vendra 2,3 mètres de fer,
   une boulangerie achètera 47,5 kg de farine. Passer d'entier à décimal après
   mise en production imposerait de migrer toutes les lignes. Les prix sont
   verrouillés à la création de la ligne. AUCUNE règle fiscale ne vit hors du
   trait JurisdictionAdapter — un seul adaptateur au MVP (CoteDIvoire). Chaque
   formule de location porte assujettie_taxe_nuitee et une règle de conversion :
   le traitement fiscal du passage et de la demi-journée est un PARAMÈTRE, jamais
   une constante. Tout calcul fiscal a un test doré sur jeu de cas figés, exécuté
   en CI. Documents opérationnels et documents fiscaux sont deux agrégats
   étanches, avec deux numérotations et deux cycles de vie ; tout document
   opérationnel porte la mention « Document non fiscal — ne tient pas lieu de
   facture ». L'API FNE n'ayant AUCUNE clé d'idempotence, l'état INDETERMINEE
   (timeout) n'est JAMAIS rejoué automatiquement : rapprochement manuel
   obligatoire. Les id d'items retournés par l'API de certification sont
   persistés — sans eux aucun avoir n'est possible.

6. OFFLINE. Chaque entité déclare sa classe A/B/C/D dans
   docs/registre-classes-offline.md. Une opération B, C ou D atteignable depuis un
   chemin de code exécutable hors ligne FAIT ÉCHOUER LE BUILD. Toute écriture
   porte un UUID v7 client ; le serveur déduplique ; le rejeu est idempotent ; le
   serveur fait foi en conflit. La file se vide AU RETOUR AU PREMIER PLAN par
   défaut sur toutes les plateformes (iOS n'a pas de synchronisation en arrière-
   plan) ; BGTaskScheduler et WorkManager sont des optimisations, jamais des
   hypothèses. Aucune donnée B, C ou D en cache d'écriture sur un terminal.
   L'interface annonce immédiatement toute action indisponible faute de réseau —
   jamais de grisé silencieux, jamais d'échec après coup, jamais de mise en file
   « au cas où ».

7. APPLICATION UNIQUE. Une seule application Nuxt 4 + Tauri v2 pour tous les rôles
   métier (desktop, Android, iOS). Les rôles sont CUMULABLES : un utilisateur porte
   N rôles, ses permissions sont l'union — c'est la norme, pas l'exception.
   L'accueil est un tableau de bord de tuiles filtrées par permission, jamais un
   menu figé. Chargement paresseux par module. L'interface ne montre JAMAIS un
   module d'activité inactif : pas de grisé, absent. Aucune invocation directe de
   window.__TAURI__ dans un composant — tout passe par PlatformAdapter
   (impression, scan, OCR, stockage sécurisé, notifications, géolocalisation,
   réseau) avec implémentations desktop/android/ios/web ; une capacité absente le
   DIT explicitement à l'utilisateur.

8. QUALITÉ ET INTERFACE. Transitions d'état couvertes par des tests d'intégration ;
   requêtes sqlx vérifiées à la compilation (cargo sqlx prepare). AUCUNE chaîne
   utilisateur en dur : clés i18n fr ET en, fr par défaut. MODE SOMBRE dès le
   premier écran, jamais rétrofitté. Aucune couleur ni espacement littéral hors
   des tokens de docs/design/tokens.md. Logs structurés avec corrélation ;
   Sentry ; /health.

9. SÉCURITÉ. Le verrouillage par adresse MAC est techniquement impossible (iOS 14
   et Android 10 randomisent la MAC ; Android n'expose pas la MAC matérielle) —
   il n'est jamais implémenté. À la place : enrôlement d'appareil par paire de
   clés en Keystore/Keychain signant chaque requête, attestation d'intégrité,
   liste blanche révocable. Le géorepérage est SOUPLE : 300 m par défaut, alerte
   au gérant, JAMAIS bloquant sur une action critique. Coffre chiffré par tenant
   pour les clés FNE. Aucun secret dans le binaire Tauri. Journal d'audit
   immuable sur remise, annulation, avoir, ouverture de tiroir, modification de
   tarif, changement de rôle, écart de caisse et rebascule de palier de passage —
   module de premier plan, pas journal technique.

10. PÉRIMÈTRE. « Prêt ≠ construit » : les provisions du cadrage §14 (adaptateurs
    de juridiction supplémentaires, devises actives, modules additionnels, canal
    TERNE, convention inter-établissements, contrats et cautions, comptes
    entreprises, IoT) sont des choix de modèle de données uniquement — aucune UI,
    aucune logique au MVP. Toute fonctionnalité qui ne contribue pas à faire
    abandonner le papier au pilote ou à garantir la conformité fiscale est
    refusée. Les priorités P0/P1/P2/PROVISION des user stories font foi.

11. VERSIONS. Dernières versions stables de chaque brique (Rust, Actix, sqlx,
    utoipa, Nuxt 4, Tailwind 4, Tauri v2, Postgres, Redis, Garage), VÉRIFIÉES SUR
    LES REGISTRES OFFICIELS avec l'URL citée, puis ÉPINGLÉES EXACTEMENT (pas
    d'intervalle) et figées par lockfiles. Ne jamais proposer un numéro de version
    de mémoire. Aucune montée majeure pendant un incrément ; revue groupée
    mensuelle.

12. RÉFÉRENCE VISUELLE. docs/design/html/{code}-{nom}[-{etat}].html est la
    RÉFÉRENCE NORMATIVE de chaque écran : valeurs exactes et hiérarchie DOM, un
    fichier par état (27 fichiers). Les fondations sont dans
    docs/design/fondation/, les prototypes animés dans docs/design/proto/, les
    documents imprimés dans docs/design/documents/.
    docs/design/tokens.md contient les valeurs curées (couleurs clair ET sombre,
    typographie, espacements, rayons) consommées par le thème Tailwind 4 et PRIME
    sur tout export en cas de divergence. docs/design/mouvement.md contient les
    durées et courbes, extraites des prototypes de docs/design/proto/.
    LE HTML DE MAQUETTE N'EST JAMAIS COPIÉ NI DÉPLACÉ VERS app/ — c'est une cible,
    pas une source : il est autonome, non sémantique, sans i18n, sans mode sombre
    câblé, sans RBAC. On lit ses valeurs, on réimplémente. SEULE EXCEPTION :
    docs/design/theme.css, le bloc @theme Tailwind 4, est copié tel quel dans
    app/assets/css/ — c'est lui qui porte les tokens.
    TAILWIND 4 D'ABORD, CSS EN DERNIER RECOURS : tout style s'exprime en
    utilitaires du noyau référençant les tokens de @theme ; le mode sombre passe
    par la variante dark:, jamais par une seconde palette ; aucune classe
    personnalisée ni style en ligne ; le CSS explicite est réservé à ce que
    Tailwind n'exprime pas (@keyframes, impression thermique) et reste regroupé.
    Une seule identité visuelle sur desktop, Android et iOS.
```

---

## 2. Prompts communs (identiques à chaque cycle, à coller tels quels)

> **Règle de rédaction de cette section.** Ces cinq prompts sont **universels** : un seul
> jeu à maintenir, collé tel quel du cycle 1 au cycle 17. Ils ne contiennent donc que des
> **invariants** — jamais « crée », « première tâche », « avant tout le reste », ni aucune
> injonction qui n'a de sens qu'une fois. Tout ce qui est propre à un cycle appartient à son
> prompt `/speckit-specify` du §3.
>
> Corollaire assumé : un cycle purement backend (FIS, SYN) recevra les lignes sur Nuxt et
> les surfaces web sans en avoir besoin. C'est du bruit inoffensif, et c'est le prix d'un
> jeu unique plutôt que dix-sept variantes à faire dériver.

### 2.1 `/speckit-clarify`

```
/speckit-clarify

Avant de me poser une question, vérifie si la réponse est dans docs/cadrage-v1.md
ou docs/user-stories-v1.md et cite la section. Ne me pose que les questions dont
la réponse n'y figure pas. Toute ambiguïté sur un montant, un seuil, un délai ou
un barème se résout par le « Récapitulatif des paramètres d'établissement » en fin
de docs/user-stories-v1.md (valeur seed, éditable). Toute ambiguïté visuelle se
résout par docs/design/html/{code}-{nom}[-{etat}].html (RÉFÉRENCE : valeurs
exactes et hiérarchie DOM — par exemple R4-passage-hors-ligne.html,
C4-cloture-bloquee.html), docs/design/tokens.md (qui PRIME en cas de divergence),
docs/design/composants.md, docs/design/mouvement.md et la MATRICE DE DÉRIVATION
docs/design/derivation.md (30 écrans dérivés : de quel motif chacun hérite).
Toute ambiguïté de VOCABULAIRE UTILISATEUR se résout par le LEXIQUE docs/design/lexique.md —
si le terme n'y figure pas, propose-moi sa formulation avant de l'écrire en dur. Toute ambiguïté sur le
comportement hors ligne se résout par docs/cadrage-v1.md §11 et
docs/registre-classes-offline.md.
```

### 2.2 `/speckit-plan`

```
/speckit-plan

VERSIONS : docs/versions-gelees.md FAIT FOI. Ne propose aucun numéro de version, ni
de mémoire ni par vérification : le gel est fait, daté et sourcé. Reprends ses
valeurs telles quelles. Si une version te paraît devoir changer, dis-le sans la
changer — c'est la revue mensuelle qui tranche.

Stack imposée (cadrage v1 §13 + docs/versions-gelees.md — non négociable) :
- Backend : Rust, Actix Web, sqlx + PostgreSQL (migrations versionnées, un schéma
  par module, RLS ENABLE **et** FORCE), utoipa + utoipa-swagger-ui, Redis
  (éphémère reconstructible SEULEMENT), Garage (API S3 uniquement).
- Application : Nuxt 4 en SSR désactivé + Tailwind 4 + Tauri v2 — UNE SEULE
  application pour tous les rôles métier, desktop + Android + iOS, chargement
  paresseux par module, PlatformAdapter obligatoire, mode sombre et i18n fr/en
  dès le premier écran.
- Surfaces web séparées : page publique de commande par QR (Nuxt SSR), console
  éditeur (ssr:false).
- CIBLE DE DÉPLOIEMENT : Docker sur VPS Contabo, linux/amd64 (mode A du cadrage
  §10.1). Le poste de développement est arm64 : les images Postgres, Redis et
  Garage sont multi-architecture, mais LE BINAIRE RUST NE L'EST PAS — la
  construction de production se fait dans Docker pour linux/amd64, jamais par copie
  d'un binaire construit localement. Toute dépendance native, tout plugin et tout
  outil que ce module ajoute doit exister pour les DEUX architectures.
- sqlx est en 0.9 : AssertSqlSafe est exigé sur toute requête non littérale et la
  sortie des macros query!() a changé. Tout extrait visant 0.8.x ne compilera pas.
  Le patron de référence est docs/module-dore.md — aligne-toi dessus plutôt que sur
  un exemple trouvé en ligne. Au cycle 1 seulement, ce fichier est un LIVRABLE et non
  une entrée : ce cycle le produit au lieu de le suivre.

Respecte la constitution (.specify/memory/constitution.md — la version du dépôt fait foi,
ne cite pas un numéro de mémoire), en particulier :
l'établissement est l'entité centrale et aucun crate ne suppose qu'il a de
l'hébergement ou un point de vente ; le socle ne connaît que article_vendable et
ressource_reservable ; la disponibilité est un intervalle horodaté garanti par
contrainte d'exclusion GiST ; aucune règle fiscale hors JurisdictionAdapter ;
montants en entiers d'unité mineure ET quantités en NUMERIC ; chaque entité déclare
sa classe dans docs/registre-classes-offline.md ; les provisions sont des données
seulement.

PORTES DE CI : la constitution définit VINGT-SIX portes bloquantes P-01 à P-23 (dont
P-01b, P-05b, P-21b), et exige de chaque porte qu'elle déclare son périmètre inspecté,
vérifie sa complétude, ne modifie pas ce qu'elle inspecte et prouve que sa cible n'est pas
vide — section « Couverture des portes ». Le plan
doit dire, pour chaque porte que ce module touche, COMMENT elle est vérifiée et par
quel test. Une porte concernée sans mécanisme de vérification est un trou du plan.

Livrables attendus du plan : migrations à créer (avec les politiques RLS),
endpoints avec annotations utoipa, structures et traits exposés aux autres crates,
événements outbox émis, classe offline de chaque entité touchée, écrans concernés,
tests d'intégration — dont les tests offline obligatoires du §0.7 des user stories.
```

### 2.3 `/speckit-tasks`

```
/speckit-tasks

Découpe en tâches d'une demi-journée à une journée maximum, ordonnées par
dépendance. Chaque tâche qui touche le schéma COMMENCE par sa migration sqlx,
politiques RLS incluses. Chaque tâche qui touche l'API SE TERMINE par la mise à
jour des annotations utoipa + la régénération du client TypeScript + build vert.
Chaque tâche qui crée une entité inclut sa déclaration de classe offline dans
docs/registre-classes-offline.md et le test correspondant.

RÉFÉRENCE VISUELLE — SI ce cycle produit des écrans. Plusieurs cycles n'en produisent
aucun (TRX, SYN, MET, et la part backend de FIS) : dans ce cas, ignore ce paragraphe,
ne fabrique pas de tâche d'interface pour respecter la forme.
LE DÉCOMPTE D'ÉCRANS SE LIT DANS docs/design/derivation.md, JAMAIS ICI. Il a changé trois fois
en quatre jours ; tout nombre recopié dans ce prompt est faux avant d'être lu. Chaque tâche d'interface cite sa référence, qui est dans
l'un de ces deux cas — jamais un troisième :
(a) ÉCRAN MAQUETTÉ — les fichiers d'états de docs/design/html/, nommage
    {code}-{nom}[-{etat}].html. La référence est le fichier d'état exact.
(b) ÉCRAN DÉRIVÉ — la référence est sa ligne de la MATRICE DE DÉRIVATION,
    docs/design/derivation.md, qui dit de quel motif il hérite et ce qui change
    (ex. « R3 Check-in nuitée hérite de R4 : parcours long, plus de champs, même
    grammaire »). Ouvre la maquette dont il hérite et respecte-la.
(c) ÉCRAN COMPOSÉ — ni maquetté ni dérivé, mais assemblé UNIQUEMENT à partir des seize
    composants canoniques de docs/design/composants.md, visibles au styleguide de
    l'application. Autorisé par docs/Kaya_Design.md §2, colonne « on code directement si »,
    aux QUATRE conditions cumulatives qui y figurent :
      · c'est une liste, un formulaire ou une table ;
      · sa conception découle entièrement de la bibliothèque de composants ;
      · il est consulté rarement, par un utilisateur formé ;
      · personne n'a de doute sur ce à quoi il ressemble.
    Plus deux garde-fous :
      · ZONE DE CHARME UNIQUEMENT (Kaya_Design.md §1) — configuration, référentiels,
        réglages. UN ÉCRAN DE ZONE DE VITESSE NE SE COMPOSE JAMAIS : R4, P2 et C4 portent
        une intention dessinée — 46 px pour les durées, 88 px pour l'heure de fin — qu'un
        assemblage ne retrouvera pas. Debout, pressé, un client en face ou de l'argent en
        jeu : on maquette.
      · SI UN MOTIF MANQUE À LA BIBLIOTHÈQUE, ON S'ARRÊTE. C'est le sens du §2 : « on
        maquette un écran parce qu'il pose un motif que d'autres reprendront ». Un composant
        nouveau se maquette, il ne s'improvise pas dans un écran.
    Tout écran composé est INSCRIT À docs/design/derivation.md dans le même changement,
    avec la mention « composé », les composants employés, et la marque « à valider à
    l'atelier terrain » — personne ne l'a dessiné, c'est une proposition.

UN ÉCRAN QUI N'ENTRE DANS AUCUN DES TROIS CAS NE SE CODE PAS : la tâche s'arrête et l'écran
part en maquettage. Ne l'invente pas, ne le déduis pas — signale-le.

LE HTML DE MAQUETTE N'EST JAMAIS COPIÉ NI DÉPLACÉ VERS app/ : on en lit les valeurs et
la structure, on réimplémente en composants Nuxt avec i18n, mode sombre, RBAC et
chargement paresseux — que l'export ne contient pas. Chaque écran est vérifié en mode
clair ET sombre.

Tout terme technique visible par l'utilisateur passe par le LEXIQUE,
docs/design/lexique.md (« certification FNE » devient « envoi aux impôts », un état
INDETERMINEE devient « nous ne savons pas si les impôts ont reçu cette facture »). Si le
terme n'y figure pas, demande sa formulation avant de l'écrire.

Les tâches P1 sont placées en fin de liste pour être livrables après le cœur P0.
Termine la liste par une tâche « revue Definition of Done »
(docs/user-stories-v1.md §0.4).
```

### 2.4 `/speckit-analyze`

```
/speckit-analyze

Vérifie la cohérence spec ↔ plan ↔ tâches ↔ constitution. Signale :
- toute exigence des stories du périmètre de ce module non couverte par une tâche ;
- toute tâche qui déborde du périmètre (P2, PROVISION, hors-périmètre listés dans
  la spec) ;
- toute violation des principes 2 (schéma par module, pas de jointure inter-
  schémas, outbox), 3 (RLS forcée, tenant_id), 4 (intervalles horodatés, horodatage
  d'autorité), 5 (entiers, adaptateur de juridiction, idempotence FNE), 6 (classes
  offline) et 10 (périmètre) de la constitution ;
- toute table créée sans politique RLS ;
- toute entité créée sans classe offline déclarée ;
- tout paramètre métier codé en dur qui devrait vivre dans la configuration
  d'établissement ;
- toute porte P-01 à P-20 de la constitution concernée par ce module sans mécanisme
  de vérification dans le plan ou les tâches ;
- toute version proposée hors de docs/versions-gelees.md.
```

### 2.5 `/speckit-implement`

```
/speckit-implement

Implémente les tâches dans l'ordre. Après chaque tâche : compile, teste, commite
avec un message conventionnel référençant la story (ex. "feat(hebergement): HEB-04
barème dégressif du passage avec rebascule de palier"). Ne saute jamais la
régénération du client TypeScript. À la fin, déroule cette checklist et liste ce
qui resterait non conforme :
[ ] Critères d'acceptation des stories du périmètre couverts par des tests
[ ] Annotations utoipa à jour ; client TS régénéré, aucun diff
[ ] Migrations sqlx versionnées ; cargo sqlx prepare vert ; seeds à jour
[ ] RLS ENABLE + FORCE sur chaque nouvelle table ; test d'isolation multi-tenant vert
[ ] Classe offline déclarée pour chaque entité + tests du §0.7 des user stories
[ ] Événements outbox émis pour chaque transition d'état
[ ] Aucune chaîne en dur : clés i18n fr ET en présentes
[ ] Chaque écran vérifié en mode clair et en mode sombre
[ ] Aucun paramètre métier en dur (configuration d'établissement)
[ ] Montants en entiers + devise ; aucune règle fiscale hors JurisdictionAdapter
[ ] Aucun window.__TAURI__ hors PlatformAdapter
[ ] Aucune jointure SQL entre schémas de modules différents
[ ] Chaque écran a sa référence : fichier de docs/design/html/ OU ligne de la matrice
    de dérivation docs/design/derivation.md — aucun écran inventé
[ ] Aucun bloc de docs/design/html/ copié dans app/ ; valeurs conformes à tokens.md
[ ] Styles en utilitaires Tailwind du noyau ; CSS explicite justifié et regroupé
[ ] Mode sombre par la variante dark:, pas par une palette dupliquée
[ ] Aucun terme technique exposé sans entrée au lexique docs/design/lexique.md
[ ] Rien construit au-delà du périmètre (provisions = données seulement)
[ ] Quantités en NUMERIC, jamais en entier (ligne de vente, mouvement de stock)
[ ] Aucune dépendance en intervalle ; lockfiles commités ; versions conformes à
    docs/versions-gelees.md
[ ] Chaque porte P-01 à P-20 concernée par le module est vérifiée par un test qui
    échoue vraiment — testé en le cassant volontairement une fois
```

---

## 3. Les 17 prompts `/speckit-specify` (ordre d'exécution)

### Cycle 1 — TRX (bootstrappe le monorepo)

```
/speckit-specify

Lis docs/user-stories-v1.md, module TRX — Transverse & infrastructure, et
docs/cadrage-v1.md sections §13 et §14.

Fonctionnalité : socle technique du monorepo Kaya.
Périmètre : TRX-01, TRX-02, TRX-02b, TRX-03, TRX-04, TRX-05 — reprends leurs
critères d'acceptation tels quels, n'invente pas d'exigences supplémentaires.
ATTENTION PARTICULIÈRE À TRX-02 : l'outbox est un GRAND LIVRE PERMANENT, pas une
file de messages. Rétention illimitée, charge utile financière complète et
dénormalisée, immuabilité. Écris le test qui vérifie qu'un événement publié reste
lisible et suffit à reconstituer l'opération sans consulter aucune autre table.
C'est ce qui rendra SYSCOHADA générable rétroactivement en phase 2.
Ce cycle crée aussi l'arborescence complète (§0.1 des prompts) : backend/
workspace Rust avec un crate par domaine, vides mais compilables, dont le crate
domain et le trait JurisdictionAdapter déclaré dans le crate fiscalite ; app/
Nuxt 4 + Tauri v2 avec core/ (auth, rbac, i18n fr+en, thème clair/sombre, sync,
PlatformAdapter) ; web/qr et web/console ; clients/ts généré ; infra/
docker-compose Postgres + Redis + Garage ; .github/workflows CI filtrée par
chemins.
PREMIÈRE TÂCHE — DÉJÀ FAITE, NE PAS REFAIRE : le gel des versions existe dans
docs/versions-gelees.md (dix briques du principe XI + quatorze crates + npm + Node
LTS, chacune vérifiée sur son registre officiel avec l'URL citée, au 2026-07-30).
La tâche du cycle est de MATÉRIALISER ce gel dans rust-toolchain.toml, le Cargo.toml
de workspace, package.json, .nvmrc, compose.yml et les lockfiles commités — §4 du
document. Ne revérifie aucun numéro : le gel fait foi jusqu'à la revue mensuelle.
DEUXIÈME TÂCHE OBLIGATOIRE — LA PLUS IMPORTANTE DU CYCLE : écrire À LA MAIN un
MODULE DORÉ — une tranche verticale complète (entité triviale → migration + RLS →
repository sqlx → service → handler utoipa → tests unitaires et d'intégration →
écran Nuxt en clair et sombre) qui servira de patron à tous les cycles suivants.
Il doit être écrit CONTRE sqlx 0.9 (AssertSqlSafe sur toute requête non littérale) :
tout extrait trouvé en ligne vise 0.8.x et ne compile pas. Sans ce patron, chaque
cycle suivant réintroduira des appels 0.8. Documente-le dans docs/module-dore.md.
TROISIÈME TÂCHE — LE REGISTRE EXISTE DÉJÀ : docs/registre-classes-offline.md est
écrit (157 opérations classées, quatre classes, arbre de décision, tests par classe).
La tâche du cycle est d'écrire LA PORTE DE CI P-13 : un test qui échoue si une entité
du schéma n'y est pas déclarée, et si une opération B, C ou D est atteignable depuis
un chemin de code exécutable hors ligne. Ne réécris pas le registre ; complète-le
au fil des entités créées.
Hors périmètre : TRX-06 (ARTCI, P1), TRX-07 (mise à jour et télémétrie, P1),
TRX-08 (design system, P1) — prévois seulement leur emplacement.
Personas : Admin éditeur.
Points d'attention : premier cycle du projet. L'openapi.json doit exister dès ce
cycle avec au moins /health documenté, et la CI doit déjà échouer sur un diff de
client non commité ET sur une table sans politique RLS. Optimise le temps de
compilation dès maintenant : linker mold, sccache, debug = "line-tables-only" en
profil dev.
```

### Cycle 2 — ETB

```
/speckit-specify

Lis docs/user-stories-v1.md, module ETB — Établissements & modules d'activité, et
docs/cadrage-v1.md sections §4 et §14.

Fonctionnalité : tenants, établissements, modules d'activité et configuration
héritée.
Périmètre : ETB-01, ETB-02, ETB-02b, ETB-02c, ETB-03, ETB-04, ETB-05 — critères
tels quels.
DEUX RÉFÉRENTIELS DISTINCTS, tous deux en TABLE et non en énumération figée :
module_activite (la verticale : HEBERGEMENT, RESTAURATION, BAR, PRESSING,
SALLE_REUNION) et capacite (le transverse : STOCK, LIVRAISON, PRODUCTION,
COMMERCE_EN_LIGNE, FIDELITE, DEVIS, COMPTES_CLIENTS). Un module DÉCLARE les
capacités qu'il consomme. Seule STOCK au profil SIMPLE est implémentée ; toute
autre valeur de capacité ou de profil est REFUSÉE EXPLICITEMENT avec un message
clair, jamais ignorée silencieusement.
L'interface ne montre JAMAIS un module ni une capacité inactifs : absent, pas grisé.
TROIS TESTS STRUCTURELS OBLIGATOIRES, à écrire AVANT l'implémentation, et qui
tournent en CI pour toujours :
(a) un établissement avec le SEUL module RESTAURATION (un maquis) fonctionne de
    bout en bout — création, commande, encaissement, clôture — sans qu'aucune
    ligne de code ne suppose l'existence d'un hébergement ;
(b) le test symétrique avec le SEUL module HEBERGEMENT (une résidence meublée) ;
(c) LE TEST D'AGNOSTICITÉ DU SOCLE (ETB-02c) : un établissement portant un module
    d'activité FICTIF MINIMAL, ne consommant AUCUNE capacité, fonctionne de bout
    en bout — création, vente comptoir, encaissement, document fiscal, clôture
    journalière. C'est la preuve formelle que le socle ne suppose ni hébergement,
    ni point de vente, ni stock, ni aucune spécificité de verticale. Si ce test
    tombe un jour, le socle s'est spécialisé sans qu'on le voie.
Seeds : tenant Deloria (établissement Abengourou, non classé, commune
d'Abengourou, Africa/Abidjan, XOF, 5 modules actifs, capacité STOCK profil SIMPLE)
et tenant « Résidence Test » (module HEBERGEMENT seul, aucune capacité, 4 unités).
Hors périmètre : ETB-06 (sélecteur de contexte, P1), ETB-07 et ETB-08
(PROVISIONS : partenaires externes avec tenant_id nullable, modules et capacités
additionnels —
tables seulement, aucune UI, aucune logique).
Personas : Admin éditeur, M. Koffi.
Points d'attention : la résolution de configuration héritée (tenant →
établissement → module → point de vente, avec surcharge) est utilisée par TOUS les
modules suivants — expose-la comme un trait propre du crate etablissements, testé
exhaustivement y compris les surcharges partielles et les valeurs absentes à tous
les niveaux.
```

### Cycle 3 — CPT

```
/speckit-specify

Lis docs/user-stories-v1.md, module CPT — Comptes, rôles & appareils, et
docs/cadrage-v1.md section §12.

Fonctionnalité : comptes, rôles cumulables et journal d'audit.
Périmètre : CPT-00, CPT-01, CPT-02, CPT-03, CPT-04 — critères tels quels.
CPT-00 EN PREMIER : personne, compte et employe sont TROIS TABLES DISTINCTES. Au
MVP seules personne et compte portent de la logique ; employe est provisionnée et
vide. Aucun code ne suppose que compte = employé — c'est ce qui conditionne la
faisabilité du module RH en phase 2 sans refonte de l'authentification.
Rôles {proprietaire, gerant, receptionniste, serveur, caissier, magasinier,
comptable, admin_editeur} CUMULABLES sur un compte : les permissions sont l'union,
c'est la norme et non l'exception — Adjoua est gérante, caissière ET
réceptionniste. Permissions granulaires attachées aux modules d'activité. JWT
court + refresh révocable, multi-appareils, déconnexion à distance. Les messages
d'erreur ne révèlent jamais si un compte existe. L'accueil est un tableau de bord
de tuiles filtrées par permission, jamais un menu figé ; chargement paresseux par
module.
Le journal d'audit (CPT-04) est un MODULE DE PREMIER PLAN, pas un journal
technique : c'est ce que le propriétaire achète. Il trace remise, annulation de
ligne envoyée, avoir, ouverture de tiroir, modification de tarif, suppression,
changement de rôle, écart de caisse, rebascule de palier de passage et forçage de
disponibilité — immuable, filtrable, consultable depuis n'importe quel terminal.
Hors périmètre : CPT-05 (enrôlement d'appareil, P1, tranche T4) et CPT-06
(attestation et géorepérage, P1, tranche T4) — prévois les colonnes, pas la
logique. Le verrouillage par adresse MAC n'est JAMAIS implémenté : il est
techniquement impossible.
Personas : Adjoua, Yao, M. Koffi, Admin éditeur.
Points d'attention : l'attribution de rôle est de classe C — aucune élévation de
privilège hors ligne, jamais.
```

### Cycle 4 — HEB

```
/speckit-specify

Lis docs/user-stories-v1.md, module HEB — Hébergement : unités & formules, et
docs/cadrage-v1.md section §5.

Fonctionnalité : unités louables, formules de location et moteur de disponibilité.
Périmètre : HEB-01, HEB-02, HEB-03, HEB-04, HEB-05 — critères tels quels.
DÉCISION STRUCTURANTE ET IRRÉVERSIBLE (HEB-02) : une occupation est un intervalle
[début, fin) en TIMESTAMP AVEC FUSEAU DE L'ÉTABLISSEMENT, JAMAIS une paire de
dates. Implémentation par contrainte d'exclusion PostgreSQL
(EXCLUDE USING gist (unite_id WITH =, periode WITH &&) sur tstzrange) : le
chevauchement devient IMPOSSIBLE AU NIVEAU DE LA BASE, pas seulement dans le code
applicatif. Test obligatoire : deux attributions concurrentes de la même unité sur
des intervalles chevauchants — une seule réussit, PAR LA CONTRAINTE et non par un
verrou applicatif. Le temps de remise en état est intégré à l'intervalle
d'indisponibilité, jamais géré à part.
Quatre familles de formules (HEB-03) : NUITEE, PASSAGE (horaire à paliers
dégressifs), DEMI_JOURNEE (plages fixes non fractionnables), MENSUEL. AUCUNE
formule n'est réservée à un type d'établissement : un hôtel peut proposer du
mensuel, une résidence meublée peut proposer du passage. La formule est attachée
à la CATÉGORIE D'UNITÉ.
Barème de passage (HEB-04) : table de paliers {duree, prix} + prix d'heure
supplémentaire au-delà du dernier palier. Un dépassement rebascule
AUTOMATIQUEMENT sur le palier supérieur, différence ajoutée à la note et TRACÉE au
journal d'audit ; au-delà d'un seuil paramétrable, bascule en nuitée. Le calcul de
durée s'appuie EXCLUSIVEMENT sur l'horodatage d'autorité serveur.
Chaque formule porte assujettie_taxe_nuitee et une règle de conversion
(aucune / une_nuitee_par_occupation / au_prorata / seuil_horaire) : le traitement
fiscal du passage et de la demi-journée est un PARAMÈTRE, pas une constante — la
valeur par défaut viendra du fiscaliste (décision B-02), ne la code pas en dur.
Seeds Deloria : 17 unités en 5 catégories + salle de réunion ; tarifs de nuitée
réels ; barème de passage 1 h : 1 500, 2 h : 2 800, 3 h : 4 000, 4 h : 5 000,
h. suppl. +1 200 ; plages de demi-journée 8h–12h et 13h–16h ; temps de remise en
état passage 30 min, nuitée 2 h, demi-journée 1 h.
Hors périmètre : HEB-06 (statut d'unité, P1), HEB-07 (calendrier tarifaire, P1),
HEB-08 (contrats et cautions, PROVISION) et HEB-09 (prestations incluses,
PROVISION au MVP : crée la table prestation_incluse, AUCUNE logique — la
fonctionnalité arrive au cycle 16).
Personas : Yao, Adjoua.
Points d'attention : c'est le module le plus structurant du projet. Le statut
d'occupation est DÉRIVÉ des occupations, jamais posé à la main ; seul le
sous-statut ménage est librement modifiable. La salle de réunion est une unité
louable d'une catégorie dédiée, PAS une entité nouvelle.
```

### Cycle 5 — SYN(en cours)

```
/speckit-specify

Lis docs/user-stories-v1.md, module SYN — Synchronisation & hors-ligne, et
docs/cadrage-v1.md section §11.

Fonctionnalité : classification hors-ligne, file d'actions et horodatage
d'autorité.
Périmètre : SYN-01, SYN-02, SYN-04 — critères tels quels.
SYN-01 : chaque entité déclare sa classe A/B/C/D dans
docs/registre-classes-offline.md ; le test de CI ÉCHOUE si une opération B, C ou D
est atteignable depuis un chemin de code exécutable hors ligne, et si une entité
n'a pas de classe déclarée. Le classement de référence du cadrage §11.3 fait foi.
SYN-02 : toute écriture porte un UUID v7 CLIENT + horodatage local ; file locale
persistante ; envoi opportuniste ; rejeu idempotent ; le serveur fait foi en
conflit. La file est conçue pour être vidée AU RETOUR AU PREMIER PLAN par défaut
sur toutes les plateformes — iOS n'a pas de Background Sync ; BGTaskScheduler et
WorkManager viendront en optimisation (MOB-06), jamais en hypothèse. Indicateur
permanent dans l'interface : connecté / dégradé / hors ligne + nombre d'éléments
en attente, lisible d'un coup d'œil.
SYN-04 : horodatage client indicatif (ordre d'affichage local) + horodatage
d'autorité à l'arrivée. TOUTE logique métier, TOUT calcul fiscal, TOUTE clôture et
TOUT calcul de durée de passage s'appuient exclusivement sur l'horodatage
d'autorité. Détection et signalement d'une dérive supérieure à 5 minutes.
Hors périmètre : SYN-03 (réconciliation des écritures orphelines) — implémentée au
cycle 12, elle dépend des séjours et des documents fiscaux. Prévois la table de
file de réconciliation et son état, pas l'écran.
Personas : Aminata, Adjoua.
Points d'attention : écris DÈS CE CYCLE les tests génériques du §0.7 des user
stories (rejeu, désordre, double soumission) sous forme de macros ou d'utilitaires
de test réutilisables par tous les cycles suivants. Chaque module qui crée une
entité les instanciera.
```

### Cycle 6 — SEJ (partie 1 — T1)

```
/speckit-specify

Lis docs/user-stories-v1.md, module SEJ — Séjours & enregistrement, et
docs/cadrage-v1.md sections §5 et §9.6.

Fonctionnalité : fiches clients, check-in, check-out et prolongation.
Périmètre : SEJ-01, SEJ-02, SEJ-04 — critères tels quels.
Fiche client rattachée au TENANT et partagée entre ses établissements. Recherche
par nom, téléphone ou numéro de pièce en moins de 300 ms sur 10 000 fiches.
Check-in (SEJ-02) : sélection de formule et de période, proposition automatique
d'une unité disponible, attribution (classe B), enregistrement des accompagnants
(ils IMPACTENT le calcul de la taxe de nuitée), génération de la fiche de police,
ouverture de la note. Client connu → pré-remplissage complet, AUCUNE ressaisie.
OBJECTIF MESURÉ ET TESTÉ : moins de 60 s pour un client connu, MOINS DE 30 s POUR
UN PASSAGE. Un passage qui dépasse 90 s sera contourné par le personnel et le
produit aura échoué — traite cette contrainte comme un critère d'acceptation, pas
comme un souhait.
Check-out (SEJ-04) : calcul final, taxe de nuitée FIGÉE à cet instant et jamais
recalculée dynamiquement, prolongation avec vérification de disponibilité sur
l'intervalle étendu et signalement explicite du conflit avec la réservation
suivante, départ anticipé avec régularisation tracée, changement d'unité en cours
de séjour créant deux intervalles avec historique conservé.
Hors périmètre : SEJ-03 (note temps réel, tranche T2), SEJ-05 (clients extérieurs,
tranche T2), SEJ-06 (OCR, P1, tranche T4). La génération du document fiscal au
check-out est du ressort du cycle FIS — expose le point d'ancrage, n'implémente
pas la certification.
Personas : Yao, Adjoua.
Points d'attention : dépendances = etablissements, comptes, hebergement. Le
check-in d'un passage doit être un parcours DISTINCT et ultra-court, pas le
parcours de nuitée avec des champs en plus.
```

### Cycle 7 — PDV

```
/speckit-specify

Lis docs/user-stories-v1.md, module PDV — Points de vente, et docs/cadrage-v1.md
section §6.

Fonctionnalité : catalogue, tables, prise de commande et pressing.
Périmètre : PDV-01, PDV-02, PDV-03, PDV-04, PDV-05, PDV-06 — critères tels quels.
Prix VERROUILLÉ à la création de la ligne : une modification de tarif ultérieure ne
modifie aucune commande existante.
QUATRE POINTS DE MODÈLE À NE PAS OUBLIER SUR L'ARTICLE ET LA LIGNE, ils coûtent
quelques minutes aujourd'hui et une migration complète plus tard :
(1) quantite en NUMERIC, JAMAIS en entier — un hôtel vend 1 bière, une
    quincaillerie vendra 2,3 mètres de fer ;
(2) unite_mesure OBLIGATOIRE sur article, valeur par défaut « unite », plus une
    table de conversion multi-unités CRÉÉE MAIS NON EXPLOITÉE ;
(3) code_barre NULLABLE, non utilisé au MVP ;
(4) article_parent_id NULLABLE pour les variantes, non utilisé au MVP.
Ne construis AUCUNE fonctionnalité autour de (2), (3) et (4) : ce sont des
provisions de modèle, pas des fonctions.
Cible de facturation d'une commande : table, sejour, comptoir, emporter. La cible
« sejour » n'est proposée QUE si le module HEBERGEMENT est actif ET qu'un séjour
est en cours — un maquis seul n'en voit jamais la trace.
PDV-03 est le CŒUR DU BESOIN HORS-LIGNE : l'ajout de lignes est de classe A et
fonctionne intégralement sans réseau. La modification d'une ligne NON ENVOYÉE est
purement locale et n'est jamais synchronisée avant l'envoi. En revanche
l'annulation d'une ligne ENVOYÉE et l'application d'une remise sont de classe B,
avec motif obligatoire, permission requise et journal d'audit.
PDV-06 — le pressing n'est PAS une vente immédiate : bon de dépôt avec liste
d'articles, état constaté, date de retrait promise, numéro de retrait, et cycle
depose → en_traitement → pret → retire. Articles d'un client logé rattachés à son
séjour, d'un client extérieur à un bon autonome. Règlement à l'avance ou au
retrait, paramétrable.
Hors périmètre : PDV-07 (écran de préparation, P1), PDV-08 (salle de réunion, P1).
Personas : Aminata, Yao, Adjoua.
Points d'attention : instancie les tests offline génériques du cycle SYN sur les
lignes de commande — rejeu, désordre, et le scénario « le réseau tombe entre la
saisie et l'envoi en préparation ».
```

### Cycle 8 — SEJ (partie 2) + note temps réel

```
/speckit-specify

Lis docs/user-stories-v1.md, module SEJ — stories SEJ-03 et SEJ-05, et
docs/cadrage-v1.md sections §4.1 et §6.1.

Fonctionnalité : note de séjour temps réel et ventes aux clients extérieurs.
Périmètre : SEJ-03, SEJ-05 — critères tels quels.
SEJ-03 : la note agrège hébergement, consommations de TOUS les points de vente,
extras et remises. LE TOTAL PROVISOIRE EST VISIBLE INSTANTANÉMENT — c'est un des
cinq problèmes explicites du cahier des charges du pilote, traite-le comme un
critère d'acceptation mesuré. Transfert de charges entre séjours de classe B,
tracé.
SEJ-05 : vente à un client sans hébergement — addition autonome, encaissement
immédiat, reçu. FONCTIONNE SANS MODULE HEBERGEMENT ACTIF : c'est le mode NORMAL
d'un maquis, d'un bar ou d'un pressing seul, pas un cas dégradé. Fiche client
optionnelle.
Hors périmètre : l'encaissement lui-même est du ressort du cycle CAI — expose le
point d'ancrage.
Personas : Adjoua, Yao, Aminata.
Points d'attention : la note est portée par l'ÉTABLISSEMENT, pas par le module.
Une consommation au bar s'ajoute à la note d'un séjour si l'hébergement est actif,
à une addition de table sinon — la même ligne de code, deux cibles.
```

### Cycle 9 — CAI

```
/speckit-specify

Lis docs/user-stories-v1.md, module CAI — Caisse & encaissements, et
docs/cadrage-v1.md section §8.

Fonctionnalité : shifts, encaissements multi-modes et clôture.
Périmètre : CAI-01, CAI-02, CAI-03, CAI-04, CAI-05, CAI-06 — critères tels quels.
CAI-02 : modes espèces, Mobile Money, carte, virement, à crédit. LE RÈGLEMENT
FRACTIONNÉ ENTRE PLUSIEURS MODES SUR UNE MÊME NOTE EST LA NORME en hôtellerie —
ce n'est pas un cas limite. Espèces = classe B (irréversible) ; Mobile Money et
carte = classe D. Montants en ENTIERS de FCFA, jamais en flottant.
CAI-06 — la clôture journalière est ATOMIQUE et REFUSÉE tant que : la file de
synchronisation n'est pas vide, un document fiscal est en attente ou en échec de
certification, un terminal est déconnecté depuis plus de 15 min (paramétrable),
une addition de table est restée ouverte. LE REFUS AFFICHE PRÉCISÉMENT CE QUI
BLOQUE. Une clôture fausse est pire qu'une clôture tardive : elle est signée,
imprimée, et devient la référence.
OBJECTIF MESURÉ : clôture en moins de 15 minutes, contre environ une heure
aujourd'hui à Deloria. Critère d'acceptation, pas souhait.
Récapitulatif de clôture : recettes par service, par module d'activité et PAR
FORMULE D'HÉBERGEMENT — distinguer les recettes de passage des recettes de nuitée
est un besoin réel non couvert par le papier.
Hors périmètre : CAI-07 (comptes clients à crédit, P1) — prévois les tables.
L'encaissement Mobile Money réel dépend du cycle ADM (trait PaymentProvider) :
expose l'interface, implémente le mode espèces complètement.
Personas : Adjoua, Yao.
Points d'attention : l'écart de caisse au-delà d'un seuil paramétrable notifie le
propriétaire (journal d'audit CPT-04). Toute sortie de caisse exige un motif.
```

### Cycle 10 — FIS

```
/speckit-specify

Lis docs/user-stories-v1.md, module FIS — Fiscalité & documents, et
docs/cadrage-v1.md section §9.

Fonctionnalité : moteur de taxes, certification FNE et documents.
Périmètre : FIS-01 à FIS-08 — critères tels quels. C'EST LE CYCLE LE PLUS
SENSIBLE DU PROJET : une erreur ici expose nos clients à des sanctions et engage
notre responsabilité.
FIS-01 : trait JurisdictionAdapter, un seul adaptateur (CoteDIvoire). AUCUNE règle
fiscale ne vit ailleurs dans le code. Test doré sur jeu de cas figés exécuté en
CI — toute modification qui casse un cas doré échoue le build.
FIS-02 : documents opérationnels et documents fiscaux sont DEUX AGRÉGATS ÉTANCHES,
deux numérotations, deux cycles de vie. Tout document opérationnel porte la mention
« Document non fiscal — ne tient pas lieu de facture ». Les documents fiscaux ne
sont JAMAIS générables hors ligne.
FIS-03 : TVA 18 %, taxe communale de nuitée PAR NUITÉE ET PAR CLIENT
(accompagnants inclus) en LIGNE DISTINCTE OBLIGATOIRE séparée du HT et de la TVA,
taxe dév. touristique 2,5 %. Montant FIGÉ au check-out. Le traitement du passage et
de la demi-journée vient du drapeau et de la règle de conversion portés par la
formule (HEB-03) — ne code AUCUNE valeur par défaut en dur, elle sera fournie par
le fiscaliste (B-02).
TÂCHE DE MIGRATION EXPLICITE : les tarifs Deloria affichés (12 500, 15 500,
17 500, 20 500, 25 500) incluent aujourd'hui la taxe de nuitée. Écris la
décomposition en HT + TVA + taxe de nuitée et un contrôle de cohérence.
FIS-04 : trait FneGateway { certify, refund, status } avec implémentations
Partenaire (API du partenaire agréé) et Direct (API DGI), bascule PAR
CONFIGURATION DE TENANT. Coffre chiffré par tenant pour les clés API. Chaque
établissement client a SON PROPRE compte FNE et SA PROPRE clé — écran de saisie
avec procédure d'accompagnement, la clé n'étant visible que par le gestionnaire
principal du client dans son espace FNE.
FIS-05 : file à quatre états EN_ATTENTE → SOUMISE → CERTIFIEE, branches ECHEC et
INDETERMINEE. L'ÉTAT INDETERMINEE N'EST JAMAIS REJOUÉ AUTOMATIQUEMENT — l'API FNE
n'expose aucune clé d'idempotence, un rejeu produirait une double certification et
consommerait un sticker en double. Écran de rapprochement manuel obligatoire.
FIS-06 : L'AVOIR SE FAIT PAR QUANTITÉ, PAS PAR MONTANT. L'interface GUIDE
l'opérateur dans la manipulation « annuler la ligne entière puis refacturer au
tarif remisé » pour un geste commercial partiel. LES id D'ITEMS RETOURNÉS PAR
L'API DE CERTIFICATION SONT PERSISTÉS — sans eux aucun avoir n'est possible, et
c'est irrattrapable a posteriori.
FIS-08 : état de reversement communal mensuel (nuitées assujetties, nombre de
clients, montant dû, échéance au 15 du mois suivant), export PDF et tableur. Aucun
concurrent ne le produit.
Hors périmètre : FIS-09 (export comptable, P1), FIS-10 (canal TERNE, PROVISION :
trait EmissionChannel et colonne rne_ref nullable, AUCUNE implémentation Terne),
FIS-11 (documents commerciaux, PROVISION : tables devis et document_commercial,
aucune UI, aucune logique — le B2B et le commerce en dépendront, l'hôtellerie non).
Personas : Adjoua, M. Diarra, M. Koffi.
Points d'attention : l'environnement de test DGI est en HTTP nu sur IP publique
(http://54.247.95.108) — aucune donnée réelle ne doit y transiter, et le code doit
refuser de démarrer en production sur une URL non HTTPS. Surveillance des stickers
avec alerte J-7 et J-2 et blocage préventif de la clôture si le stock ne couvre pas
les documents en attente.
```

### Cycle 11 — IMP

```
/speckit-specify

Lis docs/user-stories-v1.md, module IMP — Impression & documents, et
docs/cadrage-v1.md sections §8 et §9.

Fonctionnalité : impression thermique et documents imprimés.
Périmètre : IMP-01, IMP-02, IMP-03 — critères tels quels.
IMP-01 : ESC/POS 80 mm, USB et réseau, depuis desktop ; ouverture du tiroir-caisse.
FILE D'IMPRESSION AVEC REPRISE : une imprimante hors ligne NE BLOQUE JAMAIS
l'encaissement. Modèles : ticket de commande, bon de préparation, bon de dépôt
pressing, reçu, rapport de shift.
IMP-02 : mention « Document non fiscal — ne tient pas lieu de facture »
OBLIGATOIRE sur tout document opérationnel, détail par service, total provisoire,
branding de l'établissement (ETB-05).
IMP-03 : facture fiscale A4 PDF conforme — mentions obligatoires, identification
complète du client, désignation détaillée des prestations, LIGNE DISTINCTE DE
TAXE DE NUITÉE, visuel FNE, QR code et sceau retournés par la DGI. Stockage
Garage, rétention 10 ans, réimprimable depuis la fiche de séjour.
Hors périmètre : IMP-04 (modèles éditables, P1) ; l'impression Bluetooth mobile
est du ressort de MOB-04.
Personas : Adjoua, Yao.
Points d'attention : CHAQUE MODÈLE DOIT ÊTRE VÉRIFIÉ SUR UNE IMPRIMANTE THERMIQUE
RÉELLE, pas seulement en aperçu — la largeur utile, les caractères accentués et la
coupe papier ne se testent pas à l'écran.
```

### Cycle 12 — SYN (partie 2) + réconciliation

```
/speckit-specify

Lis docs/user-stories-v1.md, story SYN-03, et docs/cadrage-v1.md §11.4.

Fonctionnalité : réconciliation des écritures orphelines.
Périmètre : SYN-03 — critères tels quels.
Scénario : une consommation saisie hors ligne par Aminata arrive sur un séjour
déjà clos ET FACTURÉ par Adjoua. C'EST LE CONFLIT LE PLUS FRÉQUENT EN EXPLOITATION
RÉELLE et il n'a aucune solution automatique.
RÉSOLUTION HUMAINE OBLIGATOIRE : jamais de rejet silencieux, jamais d'ajout
d'office. Écran de réconciliation où le gérant tranche entre (a) avoir et
refacturation, (b) prise en charge par l'établissement, (c) rattachement au séjour
suivant du même client.
Aggravé par le fait que l'avoir FNE se fait par quantité (FIS-06) : l'écran doit
guider la manipulation complète, pas se contenter de signaler le problème.
Hors périmètre : rien de plus.
Personas : Adjoua, Aminata.
Points d'attention : ce cycle est court mais critique. Écris d'abord le test
d'intégration qui reproduit le scénario de bout en bout — réseau coupé, saisie,
check-out, certification, retour du réseau — puis implémente l'écran. Ce test
tourne en CI pour toujours.
```

### Cycle 13 — DIR (partie 1 — T3)

```
/speckit-specify

Lis docs/user-stories-v1.md, module DIR — Direction & pilotage, story DIR-01.

Fonctionnalité : tableau de bord d'établissement.
Périmètre : DIR-01 — critères tels quels. Occupation du jour, arrivées et départs,
unités à nettoyer, recettes par service, encaissements par mode, documents en
attente de certification.
Hors périmètre : DIR-02 à DIR-05 (tranche T5).
Personas : Adjoua, M. Koffi.
Points d'attention : ce tableau de bord est le premier écran que voit Adjoua
chaque matin. Il doit charger en moins d'une seconde sur le poste de la réception
et se lire sans faire défiler.

=== FIN DE L'INCRÉMENT 1 ===
Avant de poursuivre : déploiement chez Deloria en DOUBLE EXPLOITATION AVEC LE
CAHIER PAPIER pendant 3 semaines, formation du personnel (2 jours), support
quotidien. Le jalon J1 est atteint quand Deloria abandonne le cahier de son propre
chef. Ne démarre pas le cycle 14 avant.
```

### Cycle 14 — MOB

```
/speckit-specify

Lis docs/user-stories-v1.md, module MOB — Mobile Tauri, et docs/cadrage-v1.md
section §13.4.

Fonctionnalité : cibles Android et iOS de l'application unique.
Périmètre : MOB-01, MOB-02, MOB-03, MOB-04, MOB-05 — critères tels quels.
MOB-01 : chaîne complète cargo tauri android et cargo tauri ios, signature,
distribution. Android : APK direct hors store possible, mécanisme d'installation à
écrire. iOS : AUCUNE installation hors App Store, toute mise à jour du binaire
passe par la revue Apple — le plugin updater de Tauri exclut explicitement les
cibles mobiles. La mise à jour des assets web dans le WebView est un CORRECTIF
D'URGENCE, jamais le canal de livraison normal.
MOB-02 : AUCUNE invocation directe de window.__TAURI__ dans un composant. Si le
code existant en contient, ce cycle les refactore tous derrière PlatformAdapter
(impression, scan, OCR, stockage sécurisé, notifications, géolocalisation, réseau)
avec implémentations desktop/android/ios/web. Une capacité absente sur une
plateforme LE DIT explicitement à l'utilisateur et propose l'alternative.
MOB-03 : plugin natif Swift (APNs) + Kotlin (FCM) — le plugin officiel Tauri ne
couvre que les notifications LOCALES. Canal haute importance pour : document en
échec de certification, stickers bas, écart de caisse, terminal déconnecté.
Budget annoncé 2 à 3 semaines : si tu dépasses de plus d'une semaine, signale-le.
MOB-04 : plugin natif CoreBluetooth + Android BT pour l'impression thermique
mobile. Budget 2 semaines.
MOB-05 : Keystore/Keychain pour les clés d'appareil, CHIFFREMENT AU REPOS du cache
local et PURGE À LA DÉCONNEXION — ce sont des données d'identité de clients
d'hôtel.
Hors périmètre : MOB-06 (synchronisation en arrière-plan, P1) — la file se vide au
premier plan par défaut, c'est une optimisation.
Personas : Aminata, Yao, Adjoua, M. Koffi.
Points d'attention : teste sur un Android d'ENTRÉE DE GAMME réel, pas sur un
émulateur ni sur un téléphone récent — c'est le matériel qu'aura Aminata. La
latence de saisie d'une ligne de commande doit rester sous 200 ms.
```

### Cycle 15 — CPT (partie 2) + QRC

```
/speckit-specify

Lis docs/user-stories-v1.md, modules CPT (stories CPT-05, CPT-06) et QRC —
Commande par QR, et docs/cadrage-v1.md sections §7 et §12.2.

Fonctionnalité : enrôlement d'appareil et commande client par QR.
Périmètre : CPT-05, CPT-06, QRC-01, QRC-02, QRC-03, QRC-04 — critères tels quels.
CPT-05 : le gérant approuve un appareil une fois ; une paire de clés est générée
dans le Keystore/Keychain et SIGNE CHAQUE REQUÊTE. Liste des appareils enrôlés
dans le back-office avec révocation immédiate. LE VERROUILLAGE PAR ADRESSE MAC
N'EST JAMAIS IMPLÉMENTÉ : iOS 14 et Android 10 randomisent la MAC par réseau et
Android n'expose pas la MAC matérielle.
CPT-06 : Play Integrity et DeviceCheck + App Attest vérifiés CÔTÉ SERVEUR.
Géorepérage SOUPLE : 300 m par défaut, position simulée détectée → alerte au
gérant, JAMAIS blocage. JAMAIS bloquant sur une action critique — un caissier qui
ne peut pas encaisser parce que le GPS dérive est un client perdu.
QRC-01 : QR encodant un jeton SIGNÉ HMAC, OPAQUE ET RÉVOCABLE côté serveur —
jamais un identifiant de table lisible ou devinable. Révocation sans changer la
plaque physique. PDF de plaque téléchargeable par table.
QRC-02 : page Nuxt SSR HORS application Tauri et hors authentification. AUCUNE
donnée personnelle demandée : pas de compte, pas de téléphone, pas d'email.
Performance sur connexion limitée.
QRC-03 : la commande arrive en état À_CONFIRMER (réception = classe A) ; LE
SERVEUR VALIDE D'UN TAP EN CONSTATANT LA PRÉSENCE PHYSIQUE (validation = classe B).
Rien ne part en préparation avant. C'EST LE SEUL MÉCANISME ANTI-FRAUDE DU MVP :
aucun géorepérage, aucun portail captif, aucun paiement préalable.
QRC-04 : 3 paniers en attente maximum par table (paramétrable), compteur Redis en
fenêtre glissante — sans quoi un plaisantin sature l'écran du serveur.
Hors périmètre : paiement en ligne par le client, personnalisation du menu public.
Personas : Aminata, Adjoua, et tout client qui scanne.
Points d'attention : la surface QR est publique et non authentifiée — c'est la
seule de tout le produit. Traite-la comme telle : rate-limiting, aucune donnée
personnelle, aucun endpoint d'écriture au-delà de la création d'un panier en
attente.
```

### Cycle 16 — RSV + SEJ (OCR)

```
/speckit-specify

Lis docs/user-stories-v1.md, module RSV — Réservations et story SEJ-06.

Fonctionnalité : réservations, planning, enregistrement accéléré par OCR et
prestations incluses.
Périmètre : RSV-01, RSV-02, RSV-03, RSV-04, SEJ-06, HEB-09 — critères tels quels.
HEB-09 : la table prestation_incluse existe depuis le cycle 4 ; ce cycle en
implémente la FONCTIONNALITÉ. Une formule peut inclure des prestations —
petit-déjeuner, blanchisserie, courses de conciergerie. La prestation incluse
s'affiche sur la note, se décompte à la consommation, n'est PAS facturée, et le
DÉPASSEMENT DU QUOTA bascule proprement en facturation normale avec mention
explicite sur la note. Le petit-déjeuner inclus est une pratique répandue dans
l'hôtellerie ivoirienne absente du périmètre initial : c'est une lacune produit
comblée, pas une ambition plateforme.
RSV-02 : le planning visuel doit avoir une GRANULARITÉ HORAIRE — les passages et
les demi-journées doivent y être lisibles, pas écrasés dans une case de journée.
C'est ce qui distingue ce planning de tous les PMS existants. Vue « aujourd'hui »
par défaut avec arrivées, départs et unités à nettoyer.
RSV-03 : arrhes encaissées à la réservation et imputées sur la note au check-in ;
politique d'annulation paramétrable (délai franc, montant retenu).
SEJ-06 : capture de la pièce d'identité par la caméra, extraction SUR L'APPAREIL
(Vision iOS / ML Kit Android) — jamais dans le cloud, pour la latence, le coût et
la protection des données. L'agent CORRIGE, ne saisit pas. ENTIÈREMENT
DÉGRADABLE : caméra indisponible, modèle en échec ou document illisible → saisie
manuelle immédiate sans blocage. C'est un accélérateur, JAMAIS un passage
obligé — teste explicitement le chemin dégradé.
Pièce d'identité = donnée sensible : chiffrement au repos, rétention paramétrable
(90 j par défaut), journal d'accès, consentement tracé.
Hors périmètre : RSV-05 (conversion en séjour en un tap, P1) ; channel manager,
OTA et moteur de réservation public sont hors périmètre du produit.
Personas : Yao, Adjoua.
Points d'attention : l'OCR ne doit PAS allonger le parcours d'enregistrement quand
il échoue. Mesure le parcours complet avec et sans OCR : l'objectif de 60 s pour un
client nouveau doit tenir dans les deux cas.
```

### Cycle 17 — STK, DIR (partie 2), ADM, MET

```
/speckit-specify

Lis docs/user-stories-v1.md, modules STK — Stocks, DIR (stories DIR-02 à DIR-05),
ADM — Console éditeur & abonnements, MET — Métriques, et docs/cadrage-v1.md
sections §13.6, §15 et §18.

Fonctionnalité : stocks, pilotage consolidé, console éditeur et métriques.
Périmètre : STK-01 à STK-04, DIR-02 à DIR-05, ADM-01 à ADM-06, MET-01, MET-02 —
critères tels quels.
STK : le stock est une CAPACITÉ (ETB-02b), pas un module d'activité. Son crate vit
dans capacites/stocks, jamais dans socle/ ni dans une verticale. Il porte un profil ;
SEUL le profil SIMPLE est implémenté, les autres sont refusés explicitement.
STK-02 : classe B par défaut. LA DÉCISION B-05 DOIT ÊTRE TRANCHÉE AVANT CE
CYCLE — si le stock sert à détecter le vol il reste B et sérialisé ; s'il sert
seulement à réapprovisionner il passe en A et tout devient plus simple. Ne tranche
pas toi-même : demande la décision. Consultation hors ligne toujours affichée comme
INDICATIVE.
STK-02, DEUX COLONNES À NE PAS OUBLIER : quantite en NUMERIC (jamais entier), et
cout_unitaire NULLABLE ET JAMAIS RENSEIGNÉ au MVP. Sans cette seconde colonne,
aucune valorisation rétroactive ne serait possible et le profil VALORISE exigerait
un jour de recréer tout l'historique. Le coût de l'ajouter aujourd'hui est nul.
DIR-02 : vue consolidée sur tous les établissements du tenant, en temps réel,
LISIBLE SUR TÉLÉPHONE — c'est la demande explicite du propriétaire.
DIR-03 : ventilation par module d'activité, par point de vente ET PAR FORMULE
D'HÉBERGEMENT. Distinguer les recettes de passage des recettes de nuitée est un
besoin réel non couvert par le papier.
ADM-03 : calcul sur le NOMBRE TOTAL D'unite_facturable DU TENANT, tous
établissements confondus ; LE NOMBRE D'ÉTABLISSEMENTS N'A AUCUN IMPACT sur le prix.
unite_facturable est une MÉTRIQUE ABSTRAITE DÉFINIE PAR LA VERTICALE, jamais
« chambre » codé en dur : la verticale expose un trait qui dit ce qu'on compte —
la chambre pour l'hébergement, le point de vente pour la restauration et le
commerce, le véhicule pour la livraison. Le moteur de tarification ne connaît qu'un
nombre. Au MVP la seule implémentation est « chambre » et le comportement
observable est strictement identique à une facturation à la chambre ; sans cette
abstraction, aucune verticale sans chambres ne serait facturable un jour.
Deux modes avec application automatique du MOINS-DISANT : forfait par palier
(≤ 25 unités : 20 000 ; 26–50 : 40 000 ; > 50 : 1 000/unité) et compteur
(1 000/unité à tout niveau). Tous les seuils, montants et paliers ÉDITABLES DEPUIS
LA CONSOLE sans déploiement. Périodes de gratuité, frais d'installation et remises
tracés.
ADM-04 : trait PaymentProvider { create_checkout, verify_webhook, refund },
implémentation CinetPay. Session créée CÔTÉ SERVEUR, webhook validé par SIGNATURE
HMAC, jamais de confiance dans la redirection client seule, idempotence sur le
webhook.
ADM-05 : sans cet écran de diagnostic à distance, le support d'Abengourou impose
un déplacement de 4 h. Traite-le comme une fonctionnalité de premier plan.
Hors périmètre : MET-03 (agrégats et KPI, P1), CAI-07 (comptes à crédit, P1),
FIS-09 (export comptable, P1). Valorisation de stock, commandes fournisseurs et BI
libre-service sont hors périmètre du produit.
Personas : M. Koffi, Adjoua, Admin éditeur.
Points d'attention : ce cycle est large — découpe-le en quatre lots livrables
séparément (STK, DIR, ADM, MET) et implémente ADM en premier, car sans
provisionnement ni facturation il n'y a pas de second client.
```

---

## 4. Ordre d'exécution par tranche

| Tranche | Cycles | Démo de sortie |
|---|---|---|
| **T1** (S4–S8) | 1 → 6 (TRX, ETB, CPT, HEB, SYN, SEJ-1) | Yao enregistre un client en B3 pour 2 nuits, puis un passage de 4 h en A1 ; la contrainte d'exclusion empêche tout chevauchement ; un maquis de test fonctionne sans hébergement |
| **T2** (S9–S12) | 7 → 9 (PDV, SEJ-2, CAI) | Aminata prend une commande hors réseau, elle s'ajoute à la note de B3 ; Adjoua encaisse en espèces + Mobile Money sur la même note et boucle son shift |
| **T3** (S13–S17) | 10 → 13 (FIS, IMP, SYN-2, DIR-1) | Une facture Deloria est certifiée FNE avec taxe de nuitée en ligne distincte ; la clôture tombe au franc près ; l'état de reversement communal est généré |
| **J1** | — | **Deloria abandonne le cahier papier.** Double exploitation 3 semaines |
| **T4** (S18–S25) | 14 → 16 (MOB, CPT-2 + QRC, RSV + OCR) | Aminata saisit sur son Android enrôlé ; un client scanne le QR, Aminata valide d'un tap ; une réservation apparaît au planning horaire |
| **T5** (S26–S31) | 17 (STK, DIR-2, ADM, MET) | M. Koffi voit ses deux établissements depuis son téléphone ; un tenant est provisionné et facturé depuis la console |

**Incrément 3 (S32–S45)** : iOS en production, contrats et cautions des résidences meublées, nœud de site LAN, paquet auto-hébergé durci, second adaptateur de juridiction, comptes clients entreprises.

Chaque cycle spécifie l'ensemble de son périmètre ; dans `/speckit-tasks`, les tâches P1 sont placées en fin de liste pour être livrables après le cœur P0.

---

## 5. Règles de conduite du dépôt

- **Un écran se code dans trois cas, et seulement trois.** (a) **Maquetté** — un fichier d'état de `docs/design/html/`. (b) **Dérivé** — une ligne de `docs/design/derivation.md` qui dit de quel motif il hérite. (c) **Composé** — assemblé uniquement à partir des seize composants canoniques, aux conditions de `docs/Kaya_Design.md` §2 : liste, formulaire ou table ; conception entièrement issue de la bibliothèque ; consulté rarement par un utilisateur formé ; aucun doute sur son aspect. **Zone de charme seulement** — un écran de comptoir se maquette — et **inscrit à la matrice dans le même changement**. Hors de ces trois cas, le cycle s'arrête et l'écran part en maquettage. Le risque à écarter n'est pas la laideur, c'est la **dérive** : trente écrans inventés un par un, chacun avec sa grammaire.
- **Aucun terme technique visible par l'utilisateur sans entrée au lexique `docs/design/lexique.md`.** « Certification FNE », « état indéterminé », « écriture orpheline » ne doivent jamais atteindre un bouton ou un message. Tout nouveau concept exposé entre au lexique **avant** d'être codé.
- **Une branche par cycle** (`feat/heb-formules`), merge quand la checklist du §2.5 passe.
- **Commits conventionnels référençant les stories** : `feat(hebergement): HEB-02 disponibilité par contrainte d'exclusion GiST`.
- **Si une décision produit change** : mettre à jour d'abord `docs/cadrage-v1.md` et `docs/user-stories-v1.md` (et `docs/design/` si visuel), puis relancer `/speckit-specify` du module concerné — jamais l'inverse.
- **Fin de chaque tranche** : tag Git (`t1-done`), démo réelle sur le matériel du pilote, restauration de sauvegarde testée.
- **Ne jamais fusionner du code qu'on ne saurait pas déboguer à 2 h du matin pendant une clôture de caisse à Abengourou.** C'est la règle qui prime sur toutes les autres en développement solo assisté par IA.

---

## 6. Décisions à trancher avant certains cycles

| Décision | Bloque | Échéance |
|---|---|---|
| **B-03** — source de revenus de transition | Rien techniquement, tout le reste humainement | **S2** |
| **O-01** — `client` / `personne` en classe C rend le check-in d'un **client inconnu** impossible hors ligne, même en mode nœud de site | **Cycle 6 (SEJ-1), en T1** — la plus proche | **avant SEJ-02** |
| **B-02** — taxe de nuitée sur passage et demi-journée | Valeurs par défaut du cycle 10 (FIS) | S3, fiscaliste |
| **B-07** — barèmes de passage réels du pilote | Seeds du cycle 4 (HEB) | S3, atelier terrain |
| **O-03** — crate d'accueil de la surface QR, transverse à `restauration` et `bar`, absente des quatre verticales | Cycle 15 (QRC) | avant QRC-01 |
| **B-05 / O-02** — classe offline du stock (A ou B) | Cycle 17 (STK) | S4, avec le pilote |
| **B-06** — nom définitif et marque | Renommage global, trivial tant qu'il est fait tôt | S6 |
| **B-04** — montant des frais d'installation | Cycle 17 (ADM) | S6 |
| ~~**B-01** — hébergement en CI ou en Europe~~ | ✅ **tranchée de fait** : Docker sur VPS Contabo, qui sert l'Afrique depuis son hub Europe. Reste à consigner le **transfert transfrontalier ARTCI** (pièces d'identité de clients ivoiriens hors CI) au registre des traitements de TRX-06 | fait |

Les décisions **O-xx** sont portées par `docs/registre-classes-offline.md` §12. Jusqu'à leur
arbitrage, **la classe la plus stricte du registre s'applique** — aucun contournement.
