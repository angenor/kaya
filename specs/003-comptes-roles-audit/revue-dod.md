# Revue de la Definition of Done — Cycle 003 · CPT

**T064** · 2026-08-01, **révisée le 2026-08-02** après le lot d'amorçage ·
`docs/user-stories-v1.md` §0.4

Les dix points, pour chacune des **six user stories**, **avec la preuve de chacun**. Un point coché
sans preuve est un point que personne n'a vérifié.

> **Le point 10 est SANS OBJET, et c'est écrit ici plutôt que coché en silence.** Ce cycle
> n'imprime rien : aucun document, aucune file d'impression, aucun pilote. Même règle qu'au
> cycle 001 pour le point 8, et qu'au cycle 002 pour le point 10.

> **Trois défauts trouvés en navigateur n'étaient couverts par AUCUN test, et deux rendaient `G3`
> et `G4` inatteignables.** Ils sont la raison d'être de cette revue : les 428 tests front montaient
> les composants directement, ce qui contourne le routeur, `<Suspense>`, les layouts et les plugins.
>
> **Ils sont soldés au 2026-08-02**, et leur cause était **unique** : l'application n'avait aucun
> point d'amorçage. La constitution a été amendée en **1.7.0** — porte **P-22**, parcours réel — et
> le § « Ce qui reste non conforme » porte l'état d'après. C'est ce qui fait passer le **point 8**
> de ✗ à ✓, pour la première fois depuis le premier cycle.

---

## Les chiffres du cycle, recomptés et non repris du plan

| Grandeur | Réel | Ce qu'annonçait le plan |
|---|---|---|
| Types d'événements outbox | **22** (13 + 9) | 21 |
| Opérations HTTP servies | **43** | 40 |
| `operationId` distincts | **43** | non compté |
| Tables, quatre schémas applicatifs | **26**, dont **10** créées ici | « 26 », mais comptées sur un seul schéma |
| Migrations du cycle | **7** (`0014` à `0020`) | **5** (`0014` à `0018`) |
| Familles d'audit | **10**, dont **2 branchées** | 10 |
| Tests backend | **224**, 0 échec | — |
| Tests front | **428** à la clôture, **440** après le lot d'amorçage — 0 échec, 0 erreur de type | — |
| Tests de parcours réel (P-22) | **19** sur **6** routes — la porte n'existait pas | — |
| Clés i18n | **183 fr / 183 en**, écart nul | — |

**Quatre écarts au plan, et non trois.** La première version de cette revue portait « 7 » dans les
deux colonnes de la ligne des migrations, ce qui faisait disparaître le quatrième derrière un
« pas d'écart » — dans la colonne même qui sert à mesurer les écarts. Le plan annonçait cinq
migrations, `0014` à `0018` ; `0019` (les cinq paramètres au catalogue) et `0020` (résolution d'un
identifiant **avant que le tenant soit connu**, ce que ni le plan ni le modèle de données n'avaient
prévu) sont nées en cours de cycle.

Aucun des quatre n'a été résorbé en corrigeant un chiffre : `couverture_portes.rs` relit le
catalogue système et le contrat.

---

## La matrice — dix points × six stories

`✓` conforme · `✗` non conforme · `—` sans objet pour cette story · `⊘` **sans objet pour le
cycle entier**

| | US1 | US2 | US3 | US4 | US5 | US6 |
|---|:---:|:---:|:---:|:---:|:---:|:---:|
| **1** Tests unitaires **et** d'intégration | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| **2** utoipa à jour, client TS sans diff | — | ✓ | ✓ | — | ✓ | — |
| **3** Migration versionnée, sqlx vert, seeds | ✓ | ✓ | ✓ | — | ✓ | — |
| **4** RLS `ENABLE`+`FORCE`, isolation multi-tenant | ✓ | ✓ | ✓ | — | ✓ | ✓ |
| **5** Classe hors-ligne déclarée + test | ✓ | ✓ | ✓ | — | ✓ | ✓ |
| **6** Événement outbox par transition | ✓ | ✓ | ✓ | — | — | — |
| **7** Clés i18n fr **et** en, rien en dur | — | ✓ | ✓ | ✓ | ✓ | ✓ |
| **8** Écran vérifié en clair **et** en sombre | — | ✓ | ✓ | ✓ | ✓ | — |
| **9** Paramètres au récapitulatif si « paramétrable » | — | ✓ | — | — | — | — |
| **10** Document imprimé sur thermique réelle | ⊘ | ⊘ | ⊘ | ⊘ | ⊘ | ⊘ |

Les `—` sont des sans-objet **de story**, chacun justifié au point correspondant :

- **US1** ne sert aucune opération HTTP (les trois tables sont un modèle, les points d'entrée de
  personnes relèvent de US2) et n'affiche aucun écran.
- **US4** ne crée ni table ni événement : c'est une **règle d'affichage** filtrant des tuiles, dont
  le versant serveur est la garde de permission de US3.
- **US6** ne crée ni table ni événement : elle vérifie que les opérations des autres stories ne
  sont pas atteignables hors ligne.
- **Le point 9** ne concerne que US2 : c'est la seule story qui dise « paramétrable » — cinq clés,
  toutes au récapitulatif.

**Le point 8 était en échec pour les quatre stories qui portent un écran** à la clôture du cycle —
`R0` compris : son thème sombre était juste, et aucun chemin du produit ne l'activait. **Soldé le
2026-08-02** par le lot d'amorçage, et rendu opposable par la porte **P-22**. C'est la première fois
que ce point est coché sur une preuve mécanique depuis le début du projet ; voir le point 8.

---

## 1 · Critères d'acceptation couverts par des tests unitaires **et** d'intégration

| Story | Tests backend | Tests front |
|---|---|---|
| **US1** — trois tables distinctes (CPT-00) | `personne_compte_employe.rs` (6), `provisions_sans_logique.rs` (8) | — |
| **US2** — connexion et révocation (CPT-01) | `authentification_indiscernable.rs` (7), `session_revocation.rs` (5), `politique_mot_de_passe.rs` (5) | `ecran-r0.spec.ts`, `auth-session.spec.ts`, `file-jeton-expire.spec.ts` |
| **US3** — cumul de rôles (CPT-02) | `roles_cumules.rs` (10), `isolation_tenant.rs` (8) | `ecran-g3.spec.ts` |
| **US4** — accueil filtré (CPT-03) | — *(la décision est côté client, gardée côté serveur par `securite.rs`)* | `permissions.spec.ts`, `ecran-r1.spec.ts` |
| **US5** — journal d'audit (CPT-04) | `audit_immuabilite.rs` (6), `audit_classe_a.rs` (5), `audit_taxonomie.rs` (6) | `ecran-g4.spec.ts` |
| **US6** — le hors-ligne ne fabrique pas de droit | `classes_offline.rs` (9) | `file-classe-a.spec.ts`, `file-jeton-expire.spec.ts` |

**224 tests backend sur 28 fichiers d'intégration, 440 tests front, 19 tests de parcours réel —
tous verts.** Les douze tests front et les dix-neuf de P-22 ajoutés après la clôture sont ceux qui
couvrent ce que cette revue avait trouvé à la main.

**Ce que ces tests ne couvrent pas, et il faut le dire ici** : ils montent les composants avec
`@vue/test-utils`, hors routeur et hors `<Suspense>`. Trois défauts d'intégration leur échappent
par construction — voir le dernier §.

## 2 · Annotations utoipa à jour, client TS régénéré sans diff

`pnpm porte:p01` — **verte**. 33 chemins, **43 opérations**, déterminisme d'octet vérifié par `cmp`,
ordre stable vérifié par ajout d'un endpoint (33 lignes changées sur 3620).

**P-01b n'existait pas, et elle existe maintenant.** La constitution la porte depuis le cycle 002 —
« deux opérations homonymes produisent un client TypeScript invalide, que P-01 ne détecte pas » — et
**aucun script ni test ne l'implémentait**. Ce cycle ajoute 19 `operationId` : 43 au total, tous
présents, tous distincts, vérifiés par `couverture_portes.rs` avec son test négatif (doublon **et**
chaîne vide, que `is_some()` laisserait passer).

> **Défaut trouvé et corrigé — la garantie de P-01 s'arrêtait au client.** T062 a exercé pour de
> vrai le critère écrit dans la tâche : renommer un champ côté serveur doit faire échouer la
> compilation du front. Un `#[serde(rename)]` sur `CompteVue.nom_affichage`, contrat régénéré — **le
> front a compilé sans une erreur**. Les quatre fichiers d'accès à l'API redéclaraient les types à
> la main et convertissaient par `as unknown as`, onze fois. Les types sont désormais des alias de
> `components['schemas'][…]` ; l'expérience refaite fait échouer `tsc`. Une divergence dormait déjà
> là : `ServiceActif.active_le` existait au contrat, pas dans la copie.

## 3 · Migrations versionnées, `cargo sqlx prepare` vert, seeds à jour

**Sept migrations** (`0014` à `0020`), `0002` non modifiée — **P-02 verte**.

**P-18 verte** : 142 requêtes générées, 142 au cache. Le cache exige **deux passes** — depuis
`backend/` (130 requêtes, perd les binaires) puis depuis `backend/api/` (les 12 des binaires
`seeds` et `contrat`) — et les deux contrôles dans l'ordre : `git status` **39 ajouts, 0
suppression**, puis `SQLX_OFFLINE=true DATABASE_URL= cargo check --workspace --all-targets
--locked` vert.

Seeds à jour et **rejoués** : deux tenants, trois comptes sur Deloria, Adjoua en porte trois
(`seeds_rejouables.rs`, 7 tests). Vérifié en exécution réelle pour la revue.

## 4 · RLS `ENABLE` **et** `FORCE` sur les dix tables créées, isolation sur les 43 opérations

**26 tables** dans les quatre schémas applicatifs, dont **10 créées par ce cycle**, toutes en
`ENABLE` + `FORCE` + au moins une politique. Décompte **lu du catalogue système**, jamais écrit à la
main.

> **Le décompte ne voyait qu'un schéma.** `TABLES_CREEES` portait des noms de table et la requête
> fixait `nspname = 'etablissements'` : les dix tables de `comptes` étaient hors du balayage. C'est
> le trou du cycle 002 — P-07 ne couvrant que 4 tables sur 10 — reformé un cran plus haut, sur les
> schémas au lieu des tables. La liste porte désormais le couple `(schéma, table)`, et les schémas
> s'en déduisent : plus de second endroit à tenir à jour.

**Huit référentiels globaux** au régime nommé — dont les quatre de ce cycle,
`methode_authentification`, `role`, `permission`, `role_permission` — comptés conformes, **jamais
exemptés** : `lecture_universelle` + `administration_editeur`, `SELECT` seul pour `kaya_app`.

**43 opérations servies**, toutes avec un régime d'isolation déclaré, **et le test obtient son jeton
par le vrai chemin de connexion** — pas par une forge à la clé de signature. Les deux opérations
publiques (`session_ouvrir`, `session_rafraichir`) sont nommées ; le test échoue si une troisième
s'y ajoute.

## 5 · Classe hors-ligne déclarée pour les dix entités, tests du §0.7

Les dix tables du cycle sont au registre, et le **schéma `comptes` a été ajouté à
`SCHEMAS_APPLICATIFS`** : il en était absent, donc les dix échappaient au balayage. `TABLES_ATTENDUES
= 26` distingue « tout est déclaré » de « il n'y avait rien à inspecter ».

**Sept opérations de classe C** — création de personne, création de compte, changement d'état,
changement de mot de passe, attribution de rôle, retrait de rôle, révocation de session — aucune
atteignable depuis un chemin exécutable hors ligne (**P-13**), le test déclarant le nombre inspecté.

`journal_audit` est de **classe A** et exercée comme telle : rejeu triple → un enregistrement,
six ordres → même état final (**P-14**).

> **Découverte de P-14 : `journal_audit.id` est une clé primaire GLOBALE, pas par tenant.** Le test
> de désordre partageait trois UUID entre six tenants ; la première permutation insérait, les cinq
> autres tombaient silencieusement sur `ON CONFLICT DO NOTHING`. Identifiants figés par permutation.

## 6 · Événement outbox pour chaque transition

**22 types**, tous couverts, chacun exercé **sur les deux tenants de démonstration** — exigence 5 du
§ « Couverture des portes », née du défaut de séquence que `0012` a corrigé.

`couverture_portes.rs` compare dans les **deux sens** : un type déclaré sans test, et un type émis
par le code sans être déclaré.

**Vérifié au-delà de la présence** : rollback provoqué par type — ni ligne métier ni événement ; et
**la connexion, le rafraîchissement et l'échec d'authentification n'émettent RIEN**. Ce ne sont pas
des transitions d'état métier, et les inscrire au grand livre y écrirait la liste horodatée des
présences du personnel — dans un registre à rétention illimitée.

> **`compte.modifie` est déclaré au modèle de données sans aucun émetteur, et c'est tranché.** Le
> contrat n'expose aucune opération de modification d'identifiant. Le déclarer ferait échouer la
> porte à chaque exécution ; l'inventer produirait une opération que personne n'a spécifiée — le
> principe X l'interdit dans les deux sens. `TYPES_SANS_EMETTEUR` le nomme, la ligne reste au modèle
> de données comme provision.

## 7 · Aucune chaîne en dur, clés fr **et** en, lexique

**183 clés en français, 183 en anglais**, écart nul. **P-16 verte** : 17 templates inspectés sans
littéral, **une exemption bornée** — `pages/styleguide.vue`, dont la contrepartie (route retirée du
routeur hors `KAYA_STYLEGUIDE`) est vérifiée par la porte elle-même.

Le vocabulaire a été posé au lexique **avant** toute clé : « Registre des actions » et jamais
« journal d'audit », « Ce que chacun peut faire » et jamais « rôles », « Appareil connecté » et
jamais « session ». **Les mots « rôle », « permission », « jeton » et « JWT » n'atteignent pas
l'interface** — vérifié par `ecran-g4.spec.ts` sur le HTML rendu.

Les deux échecs de connexion rendent **une seule phrase**, et le front traite le `401` **en bloc**
sans consulter le code : brancher sur le code marcherait aujourd'hui et fuirait au premier code
ajouté côté serveur.

## 8 · Les quatre écrans en mode clair **et** en mode sombre — **CONFORME au 2026-08-02**

*Non conforme à la clôture du cycle. Soldé par le lot d'amorçage, et **désormais opposable** — ce
qui n'avait jamais été le cas depuis le début du projet.*

**L'état à la clôture, conservé parce qu'il explique la porte P-22.** Le contrôle mécanique était
vert et la vérification en navigateur échouait : le produit n'appliquait **jamais** la classe
`.dark` — `initialiserTheme()` n'était appelé nulle part — et `G3` et `G4` ne s'affichaient pas, la
navigation vers une page paresseuse cassant l'application. Les captures sombres de la revue avaient
été obtenues en forçant la classe à la main. **Le point 8 n'avait été coché pour aucune story
depuis le premier cycle**, faute d'être vérifiable autrement qu'à l'œil.

**Ce qui est vérifié maintenant, et par quoi :**

| Preuve | Ce qu'elle établit |
|---|---|
| **P-22**, 19 tests sur **6 routes** | chaque route s'ouvre — en chargement **direct** et par navigation **interne** — sans erreur de console, avec un `<main>` unique, et la classe `.dark` **effectivement appliquée** dans les deux thèmes |
| Test négatif de P-22 | le layout cassé pour de vrai, la porte refuse, le fichier est remis par un `trap` |
| `theme-sombre.spec.ts` | chaque jeton de couleur employé porte une valeur sous `.dark` ; aucune classe `dark:` ne porte de couleur, donc pas de seconde palette |
| **P-17** (`lint:tokens`) | aucune couleur ni espacement littéral hors jetons |
| Styleguide, **18 sections** | les seize composants dans les deux thèmes, polices réellement embarquées, espace fine U+202F, colonnes de montants alignées au chiffre près |

**Ce que le point 8 ne couvre toujours pas, et c'est écrit ici plutôt que coché** : P-22 vérifie
qu'une page s'ouvre et qu'elle bascule de thème, **pas qu'elle est belle**. Aucune capture n'est
comparée. La conformité à la maquette reste humaine — même limite assumée que `classes_offline.rs`
pour la justesse des classes hors-ligne.

## 9 · Paramètres exposés dans la configuration d'établissement

**Cinq clés** posées au catalogue par la migration `0019`, **toutes les cinq inscrites au
« Récapitulatif des paramètres d'établissement »** de `docs/user-stories-v1.md` dans le même
changement :

| Clé | Défaut |
|---|---|
| `mot_de_passe_longueur_min` | 8, **aucune règle de composition**, refus des mots de passe compromis |
| `indicatif_telephonique_defaut` | +225 |
| `methode_authentification` | mot de passe (`OTP_SMS` déclarée non implémentée) |
| `jeton_acces_duree_min` | 60 |
| `jeton_rafraichissement_duree_jours` | 90, rotation à chaque usage |

`parametres_catalogue.rs` rend le principe I·c vérifiable : toute clé du catalogue doit figurer au
récapitulatif, comparaison asymétrique.

## 10 · Document imprimé sur imprimante thermique — **SANS OBJET**

*Consigné explicitement, jamais coché.*

Ce cycle ne produit aucun document. Il n'y a ni rendu imprimable, ni file d'impression, ni pilote,
ni mention « Document non fiscal » à porter — le registre des actions est un **écran de
consultation**, pas un document. La première impression réelle relève du cycle **IMP**.

Il n'y a donc, contrairement au cycle 002, aucune « seule chose vérifiable » à mettre à la place :
le point est vide, pas réduit.

---

## Les vingt-quatre portes, une par une

| Porte | Comment elle est exercée | État |
|---|---|---|
| P-01 | `pnpm porte:p01` | ✓ |
| **P-01b** | `couverture_portes.rs` — **ajoutée par ce cycle**, elle n'existait pas | ✓ |
| P-02 | `pnpm porte:p02` | ✓ |
| P-03 | `architecture.rs` | ✓ |
| P-04 | `pnpm porte:p04` | ✓ |
| P-05 | `outbox_transactionnel.rs` (15) + `couverture_portes.rs` | ✓ |
| P-05b | `pnpm porte:p05b` + `audit_immuabilite.rs`, `outbox_immuabilite.rs` | ✓ |
| P-06 | `capacites_refusees.rs` | ✓ |
| P-07 | `rls_catalogue.rs` + `couverture_portes.rs`, **deux schémas** | ✓ |
| P-08 | `isolation_tenant.rs` + `couverture_portes.rs`, 43 opérations | ✓ |
| P-09 | `portes_a_vide.rs` — **à vide**, avec assertion de non-régression | ✓ |
| P-10 | `pnpm porte:p10` — y compris les clés monétaires du `JSONB` d'audit | ✓ |
| P-11 | `portes_a_vide.rs` — **à vide**, avec assertion de non-régression | ✓ |
| P-12 | `architecture.rs` — aucune référence aux types fiscaux hors `socle/fiscalite` | ✓ |
| P-13 | `classes_offline.rs`, sept opérations de classe C | ✓ |
| P-14 | `audit_classe_a.rs`, `note_etablissement_classe_a.rs` | ✓ |
| P-15 | `pnpm porte:p15` — décompte des fichiers analysés par arbre | ✓ |
| P-16 | `pnpm test:i18n` + `pnpm lint` | ✓ |
| P-17 | `pnpm lint:tokens` | ✓ |
| P-18 | `scripts/ci/preparer-sqlx.sh --verifier` — 142/142 | ✓ |
| P-19 | `pnpm porte:p19` | ✓ |
| P-20 | `pnpm porte:p20` | ✓ |
| P-21 | `pnpm porte:p21` | ✓ |
| P-21b | `pnpm porte:p21b` | ✓ |

**Les vingt-quatre sont vertes.** Et c'est précisément ce que le dernier § relativise : elles le
sont sur un produit dont deux écrans ne s'affichent pas.

---

## Les treize vérifications du quickstart

| # | Vérification | État |
|---|---|---|
| 1 | Socle compile, requêtes au cache, les deux contrôles dans l'ordre | ✓ |
| 2 | Dix tables isolées et déclarées ; 26 inspectées | ✓ |
| 3 | Trois tables distinctes ; rien ne confond compte et employé | ✓ |
| 4 | Les deux échecs de connexion indiscernables — message, code **et médiane** | ✓ |
| 4b | Révocation immédiate, rotation, détection de réutilisation, coupure de 90 min | ✓ |
| 4c | Politique de mot de passe — longueur, liste embarquée, pas de composition | ✓ |
| 5 | Cumul de rôles = union exacte ; retrait partiel ; ensemble vide valide | ✓ |
| 6 | Aucune élévation de privilège hors ligne ; `TYPES_CLASSE_A` inchangé | ✓ |
| 7 | Journal immuable, relu, filtré ; rejeu, désordre, montants `JSONB` | ✓ |
| 8 | Taxonomie complète — **2 branchées, 8 dues**, chacune avec sa story | ✓ |
| 9 | Types d'événements émis sur les **deux** tenants — 22, non 21 | ✓ |
| 10 | Opérations isolées, contrat à jour — **43**, non 40 | ✓ |
| 11 | Les quatre écrans, en clair et en sombre | **✗** |

**Le quickstart portait les mêmes chiffres périmés que le plan** — « 21 types », « 40 opérations ».
Ils y étaient laissés tels quels, au motif que ce document est le recollement. **C'était une erreur
de jugement** : le quickstart est le seul document que quelqu'un *exécute*, et un lecteur qui aurait
comparé ses valeurs attendues aux sorties de `couverture_portes.rs` aurait conclu à une régression.
Les **39 occurrences** de quatre chiffres périmés, réparties sur onze fichiers, sont recalées au
2026-08-02 — y compris un quatrième écart que personne n'avait consigné : **sept migrations livrées
pour cinq annoncées**.

---

## Ce qui reste non conforme, ou hors du périmètre livré

*Écrit ici pour que la revue de tranche l'arbitre, pas pour être découvert plus tard.*

### Les trois défauts d'intégration — **SOLDÉS au 2026-08-02**

*Conservés parce qu'ils sont l'origine de la porte P-22 et de la huitième couche du module doré. Un
défaut effacé de la revue qui l'a trouvé est un défaut qu'on refera.*

Constatés le 2026-08-01 contre l'API réelle, sur données de démonstration, compte
`adjoua@deloria.test`. Les deux premiers étaient bloquants pour la démonstration de tranche.

**Leur cause était UNIQUE, et elle était architecturale** : l'application n'avait aucun point
d'amorçage — `app.vue` faisait 23 lignes avec `<NuxtPage />` et rien d'autre, il n'existait ni
`plugins/`, ni `layouts/`, ni `middleware/`. Les trois symptômes étaient la même absence vue sous
trois angles.

| # | Défaut | Preuve | Portée |
|---|---|---|---|
| **1** | **La navigation vers une page paresseuse casse l'application.** Cliquer une tuile de `R1` change l'URL, démonte la page courante et ne monte pas la suivante : `TypeError: Cannot read properties of null (reading 'parentNode')` dans `runtime-core`, puis `<main>` disparaît du DOM | Reproduit sur `/comptes` **et** sur `/etablissement` — l'écran du cycle 002 — par clic **et** par `history.pushState`. Le `<Suspense>` de Nuxt et `defineAsyncComponent` sont en cause ; le styleguide, qui n'est pas paresseux, rend parfaitement | **Antérieur à ce cycle** : `pages/etablissement.vue` emploie le même patron depuis ETB |
| **2** | **Un chargement direct de `/comptes` ou `/journal-audit` ne reprend jamais la session.** `reprendreSession()` n'est appelé que par `pages/index.vue` : l'écran affiche « Connectez-vous pour continuer » alors qu'un jeton de rafraîchissement valide est en stockage | `localStorage` porte `kaya.auth.rafraichissement` ; `/` reprend la session, `/comptes` ne la reprend pas | Ce cycle |
| **3** | **Le produit n'applique jamais la classe `.dark`.** `initialiserTheme()` est défini, testé, exporté — et **appelé nulle part**. `app.vue` est encore la coquille du cycle 001, sans bascule de thème | `grep -rl initialiserTheme app/` ne rend que sa propre définition ; en navigateur, `document.documentElement.className` reste vide | Ce cycle |

**Conséquence combinée à la clôture : `G3` et `G4` étaient inatteignables dans un navigateur.** Par
la tuile, le défaut 1 ; par l'adresse directe, le défaut 2. Leurs données, elles, se chargeaient —
les requêtes `GET /api/v1/comptes` et `GET /api/v1/referentiels/roles` rendaient `200`.

**Pourquoi 428 tests ne l'ont pas vu** : ils montent `EcranComptes.vue` et `EcranJournalAudit.vue`
directement avec `@vue/test-utils`, ce qui contourne le routeur, `<Suspense>`, les layouts et les
plugins. Les composants étaient justes ; c'est leur montage qui ne l'était pas.

#### Ce par quoi chacun est soldé

| # | Correctif | Ce qui l'oppose désormais |
|---|---|---|
| **1** | `layouts/default.vue` rend une **racine stable** et l'unique `<main>` ; les six pages ont une racine **élément**, plus aucun `v-if`/`v-else` de premier niveau. Le chargement paresseux est **intact** — principe VII | P-22, contrôle du `<main>` unique après navigation interne, sur les 6 routes |
| **2** | `middleware/01.session.global.ts` — la reprise avant **chaque** navigation, la première comprise | P-22, contrôle du chargement direct, sur les 6 routes |
| **3** | `plugins/01.theme.client.ts` + un script en ligne dans le `<head>` contre le scintillement | P-22, contrôle de la classe `.dark` effective |

**La cause du défaut 1 a été ÉTABLIE par expérience**, quatre pages sondes et une variable à la
fois : il faut **trois** conditions réunies — racine fragmentée, composant paresseux, bascule après
montage. Une racine unique suffit à l'éliminer. La table de vérité est dans `layouts/default.vue` et
dans la huitième couche du module doré.

#### Trois défauts de plus, trouvés par P-22 dès sa première exécution

| Défaut | Portée |
|---|---|
| **`/etablissement` rendait un `404`.** La tuile pointe la route sans paramètre, et la page lisait `config.public.etablissementId` — vide depuis que CPT-01 a retiré l'identité du `runtimeConfig` — **en sautant la session**. `G1` était inatteignable depuis l'accueil, comme `G3` et `G4`, pour une raison différente | Cycle 002, aggravé par CPT-01 |
| **`<dt>` et `<dd>` enfants d'un `<span>`** dans `SectionPointsDeVente.vue` — HTML invalide, signalé **à chaque construction** par le compilateur de Vue. Aucun test ne lit les diagnostics du compilateur | Cycle 002 |
| **Le limiteur de tentatives compte les connexions RÉUSSIES** — dix par identifiant, comptées avant vérification. C'est délibéré (un compteur qui ne compterait que les échecs rétablirait la fuite que FR-012 referme), mais un utilisateur légitime qui se connecte dix fois dans la fenêtre est refusé. **Non corrigé : c'est un arbitrage, pas un défaut** | À arbitrer au pilote |

#### Le quatrième défaut, et il n'est pas dans le produit

**Cinq points d'entrée d'amorçage sur onze n'étaient appelés nulle part** —
`initialiserTheme` n'était que le plus visible. La file hors-ligne entière est débranchée :
`FileLocale` jamais instanciée, `marquerClasseA` jamais appelée, `viderFile` sans crochet de retour
au premier plan, `operationRealisable` contournée par six gardes écrites à la main. Et
`fermerSession` — la purge du principe VI sur terminal partagé — qu'aucun bouton n'atteint : **il
n'existe aucune déconnexion dans le produit**, ni clé i18n, ni composant.

Pire que prévu sur un point : `initialiserTheme` **n'était pas même testée**.
`theme-sombre.spec.ts` n'importe pas `core/theme` — il lit les jetons de `theme.css`. La règle
juste est donc plus dure que « une unité testée n'est pas une unité branchée » : **une unité écrite
n'est ni testée ni branchée par défaut**, et il faut un contrôle pour chacune des deux propriétés.

`app/tests/amorcage.spec.ts` porte les onze points d'entrée à deux états — **6 branchés, 5 dus** —
et vérifie les **deux versants**.

### Le reste

| Point | État | Qui le doit |
|---|---|---|
| **DoD n° 10** | Sans objet — le cycle n'imprime rien | Cycle IMP |
| **`compte.modifie`** | Type déclaré au modèle de données sans opération qui le produise ; provision nommée, portée par `TYPES_SANS_EMETTEUR` | Le cycle qui exposera la modification d'identifiant |
| **O-01** — client inconnu hors ligne | Non tranchée. `client` / `personne` en classe C rend le check-in d'un client inconnu impossible hors ligne, même en mode nœud de site | **Avant SEJ-02** |
| **`OTP_SMS`** | Méthode d'authentification déclarée au référentiel, refusée explicitement par le service | Hors périmètre MVP |
| **CPT-05 / CPT-06** | `employe` et `appareil_enrole` provisionnées, isolées, **sans aucun chemin d'écriture ni point d'entrée** — vérifié par `provisions_sans_logique.rs` | Tranche T4 |
| **Huit familles d'audit dues** | `remise`, `annulation_ligne_envoyee`, `avoir`, `ouverture_tiroir`, `modification_tarif`, `ecart_caisse`, `rebascule_palier_passage`, `forcage_disponibilite` — chacune nomme sa story, et le harnais fait échouer le build le jour où l'une acquiert un chemin sans changer d'état | T1 à T3 |
| **Binaire fantôme sur le port 8080** | **Corrigé.** Le motif `pgrep -fl 'target/debug/kaya-api'` ne voyait pas un `target/debug/api` d'une compilation antérieure, resté en écoute toute une session et répondant `/health` **à la place** du bon. `lsof -nP -iTCP:8080 -sTCP:LISTEN` est désormais le contrôle qui fait foi — dans `REPRISE.md` et dans le script de P-22 —, parce qu'il ne dépend d'aucun nom | — |
| **Cinq points d'amorçage dus** | `FileLocale`, `marquerClasseA`, `viderFile`, `operationRealisable`, `fermerSession` — déclarés « dus » dans `amorcage.spec.ts`, qui fait échouer le build le jour où l'un acquiert un appelant sans changer d'état | SYN-01 pour la file, **ETB-06 pour la déconnexion** |
| **Aucune déconnexion dans le produit** | Ni bouton, ni clé i18n, alors que le principe VI pose que le terminal peut être partagé. `fermerSession` existe et attend | **ETB-06** |
| **Exigence 6 absente du corps de la constitution** | La v1.7.0 l'annonce dans son rapport d'impact — « une exigence de couverture (6) » — mais le § « Couverture des portes » n'en liste que cinq, sous un intitulé « Trois exigences en découlent » périmé depuis la v1.3.0. **Non corrigé à la main** : la clause d'amendement impose `/speckit-constitution` | Prochain amendement |
| **Barre de contexte et témoin de synchronisation** | Le layout ne les porte pas. Les deux en-têtes d'écran sont **différents** et les fondre est un changement d'écran, `derivation.md` étant opposable ; le témoin est le composant 10 | ETB-06 |
| **P-22 ne juge pas l'apparence** | Elle ouvre les pages et vérifie la bascule de thème ; aucune capture n'est comparée. La conformité à la maquette reste humaine | Revue mensuelle |
| **Mesures de performance** | Aucune prise ce cycle | Mesure sur le pilote |
