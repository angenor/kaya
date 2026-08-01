# Revue de la Definition of Done — Cycle 003 · CPT

**T064** · 2026-08-01 · `docs/user-stories-v1.md` §0.4

Les dix points, pour chacune des **six user stories**, **avec la preuve de chacun**. Un point coché
sans preuve est un point que personne n'a vérifié.

> **Le point 10 est SANS OBJET, et c'est écrit ici plutôt que coché en silence.** Ce cycle
> n'imprime rien : aucun document, aucune file d'impression, aucun pilote. Même règle qu'au
> cycle 001 pour le point 8, et qu'au cycle 002 pour le point 10.

> **Trois défauts trouvés en navigateur ne sont couverts par AUCUN test, et deux rendent `G3` et
> `G4` inatteignables.** Ils sont au § « Ce qui reste non conforme », et ils sont la raison d'être
> de cette revue : les 428 tests front montent les composants directement, ce qui contourne le
> routeur, `<Suspense>` et les coquilles de page. **Le point 8 n'est donc coché pour aucune
> story.**

---

## Les chiffres du cycle, recomptés et non repris du plan

| Grandeur | Réel | Ce qu'annonçait le plan |
|---|---|---|
| Types d'événements outbox | **22** (13 + 9) | 21 |
| Opérations HTTP servies | **43** | 40 |
| `operationId` distincts | **43** | non compté |
| Tables, quatre schémas applicatifs | **26**, dont **10** créées ici | « 26 », mais comptées sur un seul schéma |
| Migrations du cycle | **7** (`0014` à `0020`) | 7 |
| Familles d'audit | **10**, dont **2 branchées** | 10 |
| Tests backend | **224**, 0 échec | — |
| Tests front | **428**, 0 échec, 0 erreur de type | — |
| Clés i18n | **183 fr / 183 en**, écart nul | — |

Les trois écarts sont réels et documentés à l'endroit où ils se constatent. **Aucun n'a été résorbé
en corrigeant un chiffre** : `couverture_portes.rs` relit le catalogue système et le contrat.

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

**224 tests backend sur 28 fichiers d'intégration, 428 tests front, tous verts.**

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

## 8 · Les quatre écrans en mode clair **et** en mode sombre — **NON CONFORME**

*Contrôle mécanique vert, vérification en navigateur **échouée**. Le détail est au dernier §.*

**Ce qui est vérifié et tient** : `theme-sombre.spec.ts` — chaque jeton de couleur employé porte une
valeur sous `.dark`, aucune classe `dark:` ne porte de couleur, pas de seconde palette. **P-17
verte** (`lint:tokens`). Et le styleguide servi par l'application rend ses **18 sections** dans les
deux thèmes, avec les polices réellement embarquées, l'espace fine U+202F et les colonnes de
montants alignées au chiffre près — capture à l'appui.

**Ce qui ne tient pas**, constaté en navigateur réel contre l'API réelle :

- **le produit n'applique JAMAIS la classe `.dark`** — `initialiserTheme()` n'est appelé nulle part,
  et aucun composant ne bascule le thème. La palette sombre est juste ; l'interrupteur n'existe pas.
  Les captures sombres de cette revue ont été obtenues en **forçant la classe à la main** ;
- **`G3` et `G4` ne s'affichent pas** — la navigation vers une page paresseuse casse l'application.

`R0` et `R1` ont bien été vus dans les deux thèmes, sur données réelles : `R1` montre **deux tuiles**
pour Adjoua, filtrées par ses permissions, icônes embarquées présentes.

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

**Le quickstart porte les mêmes chiffres périmés que le plan** — « 21 types », « 40 opérations ». Ils
sont laissés tels quels et corrigés ici : ce document est le recollement, pas le quickstart.

---

## Ce qui reste non conforme, ou hors du périmètre livré

*Écrit ici pour que la revue de tranche l'arbitre, pas pour être découvert plus tard.*

### Trois défauts d'intégration, trouvés en navigateur, couverts par aucun test

Constatés le 2026-08-01 contre l'API réelle, sur données de démonstration, compte
`adjoua@deloria.test`. **Les deux premiers sont bloquants pour la démonstration de tranche.**

| # | Défaut | Preuve | Portée |
|---|---|---|---|
| **1** | **La navigation vers une page paresseuse casse l'application.** Cliquer une tuile de `R1` change l'URL, démonte la page courante et ne monte pas la suivante : `TypeError: Cannot read properties of null (reading 'parentNode')` dans `runtime-core`, puis `<main>` disparaît du DOM | Reproduit sur `/comptes` **et** sur `/etablissement` — l'écran du cycle 002 — par clic **et** par `history.pushState`. Le `<Suspense>` de Nuxt et `defineAsyncComponent` sont en cause ; le styleguide, qui n'est pas paresseux, rend parfaitement | **Antérieur à ce cycle** : `pages/etablissement.vue` emploie le même patron depuis ETB |
| **2** | **Un chargement direct de `/comptes` ou `/journal-audit` ne reprend jamais la session.** `reprendreSession()` n'est appelé que par `pages/index.vue` : l'écran affiche « Connectez-vous pour continuer » alors qu'un jeton de rafraîchissement valide est en stockage | `localStorage` porte `kaya.auth.rafraichissement` ; `/` reprend la session, `/comptes` ne la reprend pas | Ce cycle |
| **3** | **Le produit n'applique jamais la classe `.dark`.** `initialiserTheme()` est défini, testé, exporté — et **appelé nulle part**. `app.vue` est encore la coquille du cycle 001, sans bascule de thème | `grep -rl initialiserTheme app/` ne rend que sa propre définition ; en navigateur, `document.documentElement.className` reste vide | Ce cycle |

**Conséquence combinée : `G3` et `G4` sont inatteignables dans un navigateur.** Par la tuile, le
défaut 1 ; par l'adresse directe, le défaut 2. Leurs données, elles, se chargent — les requêtes
`GET /api/v1/comptes` et `GET /api/v1/referentiels/roles` rendent `200`.

**Pourquoi 428 tests ne l'ont pas vu** : ils montent `EcranComptes.vue` et `EcranJournalAudit.vue`
directement avec `@vue/test-utils`, ce qui contourne le routeur, `<Suspense>` et la coquille de
page. Les composants sont justes ; c'est leur montage qui ne l'est pas. **Il manque un test qui
navigue.**

### Le reste

| Point | État | Qui le doit |
|---|---|---|
| **DoD n° 10** | Sans objet — le cycle n'imprime rien | Cycle IMP |
| **`compte.modifie`** | Type déclaré au modèle de données sans opération qui le produise ; provision nommée, portée par `TYPES_SANS_EMETTEUR` | Le cycle qui exposera la modification d'identifiant |
| **O-01** — client inconnu hors ligne | Non tranchée. `client` / `personne` en classe C rend le check-in d'un client inconnu impossible hors ligne, même en mode nœud de site | **Avant SEJ-02** |
| **`OTP_SMS`** | Méthode d'authentification déclarée au référentiel, refusée explicitement par le service | Hors périmètre MVP |
| **CPT-05 / CPT-06** | `employe` et `appareil_enrole` provisionnées, isolées, **sans aucun chemin d'écriture ni point d'entrée** — vérifié par `provisions_sans_logique.rs` | Tranche T4 |
| **Huit familles d'audit dues** | `remise`, `annulation_ligne_envoyee`, `avoir`, `ouverture_tiroir`, `modification_tarif`, `ecart_caisse`, `rebascule_palier_passage`, `forcage_disponibilite` — chacune nomme sa story, et le harnais fait échouer le build le jour où l'une acquiert un chemin sans changer d'état | T1 à T3 |
| **`pgrep` de la fiche de reprise** | Le motif `target/debug/kaya-api` ne voyait pas un second binaire nommé `target/debug/api`, resté en écoute sur le port 8080 pendant toute la session précédente | À élargir dans la prochaine fiche |
| **Mesures de performance** | Aucune prise ce cycle | Mesure sur le pilote |
