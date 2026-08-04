# Revue Definition of Done — cycle 006 · Fiches clients, arrivée, départ et prolongation

**Date** : 2026-08-03, **révisée le 2026-08-04** · **Branche** : `006-clients-sejours-enregistrement`
**Référence** : `docs/user-stories-v1.md` §0.4, les dix points.

> **Ce document dit ce qui n'est pas satisfait avant de dire ce qui l'est.** Une revue qui
> commence par les réussites se lit en diagonale ; celle-ci commence par les manques, parce que ce
> sont eux qui décident si le cycle est livrable.

---

## ⛔ Ce qui n'est PAS conforme — **un** manque, nommé

> **Révision du 2026-08-04.** La première version de cette revue en listait **trois**. Deux sont
> soldés : les quatre écrans sont codés, et les quatre tâches de fin de cycle sont exécutées. Le
> texte d'origine est conservé plus bas, sous « Ce qui a été soldé », parce qu'un manque effacé se
> relit comme un manque qui n'a jamais existé.

### 1 · Une dépendance nouvelle, contre l'annonce du plan

`aes-gcm =0.11.0`. Le plan annonçait « aucune dépendance nouvelle » — **cette phrase est antérieure
à la tâche T018a**, ajoutée après la conception parce que le plan écrivait que le numéro de pièce
est « protégé au repos et son accès journalisé, **dès ce cycle** » et qu'**aucune tâche ne le
faisait**.

Version vérifiée sur `https://crates.io/api/v1/crates/aes-gcm` le 2026-08-03, jamais proposée de
mémoire. **À porter au gel §3.1 à la revue mensuelle du 2026-08-31** — c'est la seule entrée que
cette revue aura à trancher du fait de ce cycle. Deux voies écartées, avec leurs motifs, au
`Cargo.toml` du workspace.

### Et deux réserves qui ne sont pas des manques, mais qui se disent

| Réserve | Portée |
|---|---|
| **Le chronométrage humain de SC-002 et SC-003 n'est pas relevé** | Il exige un opérateur et le matériel de référence. Le **protocole** est écrit et versionné (FR-106), le tableau des valeurs reste **vide** — le remplir depuis un poste de développement produirait un chiffre flatteur et faux. Voir `mesures-terrain.md` |
| **L'étape 6 de la démo est partielle** | Le parcours de démonstration n'engendre aucune des trois entrées d'audit que le quickstart cite. Les fabriquer pour remplir l'écran ferait passer le cas au vert sur une démonstration que personne ne déroulera ainsi |

---

## ✅ Ce qui a été soldé, et ce que le solde a coûté

### Les quatre écrans sont codés

| Écran | Route | Nature |
|---|---|---|
| `R4` Le passage | `/passage` | **maquetté**, cinq états |
| `R3` Arrivée | `/arrivee` | **dérivé** de `R4` — parcours long, même grammaire |
| `R7` La note et le départ | `/depart` | **maquetté** |
| `R5` Fiche client et recherche | `/clients` | **dérivé** de `R7` — liste + fiche, sans le bloc de total |

`docs/design/derivation.md` porte « CODÉ » sur les quatre, **inscrit dans le même changement que
chaque fichier, jamais avant**.

### ★ Ce que la porte P-22 a trouvé, et qui vaut plus que les écrans

**`/passage` ne se montait pas en navigateur.** L'écran dont le cadrage §5.6 fait une *condition
d'existence du produit* portait :

```ts
import { useEtatReseau } from '~/core/platform'
// → The requested module does not provide an export named 'useEtatReseau'
```

La fonction vit dans `core/platform/reseau.ts` ; le baril ne la réexporte pas. **Aucun des 581
tests unitaires ne pouvait le voir, et c'est structurel** : ils doublaient `~/core/platform` et
**fournissaient** l'export manquant. *Le double rendait vrai ce que le baril rendait faux.*

`app/tests/imports-barils.spec.ts` rend désormais le même verdict en millisecondes, sur le seul
texte des fichiers — et il rougit quand on réintroduit le défaut, vérifié.

**Deux autres défauts sont tombés dans la même session :**

| Défaut | Ce qu'il produisait | Pourquoi aucun test unitaire ne le voyait |
|---|---|---|
| La grille du passage rendait **toutes** les unités de l'établissement alors que l'écran n'applique **qu'une** formule | `formule_hors_categorie` — un refus subi **après** le geste, devant le client, sur une chambre présentée comme libre | Les tests fournissent **une seule** catégorie ; il fallait le parc réel à six catégories des seeds |
| `passage.spec.ts` se connectait avec un identifiant **téléphonique** que les seeds ne posent pas | « Identifiant ou mot de passe incorrect » — **indiscernable d'un mot de passe faux** (FR-012) | Le fichier portait une **copie** du compte au lieu de la source unique de `routes.ts` |

### Et trois défauts que seule la suite COMPLÈTE a révélés

La suite entière — `cargo test --workspace`, doctests compris — n'avait pas tourné depuis un
moment ; les exécutions ciblées (`--test <fichier>`) la remplaçaient. Elle a rendu trois choses :

| Défaut | Ce qui le cachait |
|---|---|
| **Un test des seeds assertait sur *tous* les séjours du tenant** | Il passait tant qu'aucun e2e n'avait tourné. La base de développement est partagée : un test des seeds doit parler **des seeds**, et il compare désormais sur les quatre identifiants littéraux |
| **`un_passage_de_deux_heures_ne_constate_aucune_nuit` était vert de 10 h à minuit et rouge de minuit à 10 h** | `replace_hour(10)` plaçait le début **dans le futur** avant 10 h UTC ; le départ calculait alors une période dont la fin précède le début, et la contrainte rendait `500`. Trouvé à 1 h 39 du matin, pas autrement |
| **Un bloc indenté dans un commentaire de doc était compilé comme du Rust** | `cargo test --test <fichier>` n'exécute **aucun** doctest. Le bloc contenait `…` et ne compilait pas |

### Et deux défauts d'outillage, sans lesquels rien n'était exécutable

- **Les seeds n'appliquaient pas le mot de passe qu'ils déclaraient.** `ON CONFLICT DO NOTHING` :
  sur une base où les comptes existaient, changer `KAYA_SEEDS_MOT_DE_PASSE` ne changeait **rien**.
  Le condensat est maintenant **vérifié** et réécrit s'il ne concorde plus — le raisonnement exact
  de `le_seed_applique_reellement_l_identite_qu_il_declare`, appliqué au mot de passe.
- **`scripts/dev/charger-seeds.sh` était promis par le quickstart et n'existait pas.** Son option
  `--remettre-a-neuf` répare FR-105 : chaque exécution du e2e **consomme une chambre**, et
  recharger les seeds n'en libère aucune.

★ **La remise à neuf a d'abord été écrite dans le binaire, et la base l'a refusée** —
`permission denied for table ligne_sejour`. **Le modèle de privilèges avait raison** : une
correction sur une note est une **ligne d'ajustement**, jamais une suppression. Accorder le
`DELETE` pour faire tenir un script de développement aurait ouvert dans le produit un chemin que le
produit refuse. L'opération est donc ce qu'elle est : de l'**administration de base**, sous le rôle
propriétaire, hors de l'application.

### Les quatre tâches de fin de cycle sont exécutées

| Tâche | Résultat |
|---|---|
| **T077** — seeds | 12 fiches Deloria, 2 sur Résidence Test, **4 séjours** dont un clos avec son constat figé, `nuitees_assujetties` et `montant_mineur` à **`NULL`** |
| **T082** — balayage hors ligne | **28 cas verts**, 12 routes protégées dont les quatre du cycle, sur les deux moteurs |
| **T083** — mesure terrain | Protocole écrit et versionné ; le tableau des temps humains reste **vide**, et le document le dit en tête |
| **T084** — démo T1 | **Scriptée et déroulée** — `tests-e2e/demo-t1.spec.ts`, 24 cas verts, deux moteurs, clair **et** sombre |

**Mesures relevées** : part machine du passage **131 ms** (Chromium) et **223 ms** (WebKit), pour
un budget de 6 000 ms. P-22 : **123 cas verts**.

---

## Les dix points de la Definition of Done

### 1 · Critères d'acceptation couverts par des tests ✅

**472 tests backend verts**, dont dix fichiers écrits par ce cycle, et **581 tests front**. Les
critères **serveur** et les critères **d'écran** sont couverts ; s'y ajoutent 123 cas de P-22,
28 du balayage hors ligne et 24 de la démo scriptée, sur **deux moteurs**.

⚠️ **Ce décompte ne dit rien de ce que les tests unitaires ne peuvent pas voir.** Trois défauts de
ce cycle sont passés sous 581 tests verts et n'ont été trouvés qu'en navigateur — voir « Ce que la
porte P-22 a trouvé » ci-dessus. Un nombre de tests n'est pas une mesure de couverture.

| Fichier | Ce qu'il couvre |
|---|---|
| `client_recherche.rs` | Trois formes · repli sur noms ivoiriens · **deux apostrophes** · p95 **6-7 ms** sur 10 000 fiches (cible 300 ms) · le personnel n'apparaît pas · indépendance au module |
| `client_classes_offline.rs` | Classe A de `preference_personne`, classe C de `client` |
| `client_journal_acces.rs` | ★ Journal d'accès à la pièce · chiffrement au repos · **le contexte ne porte pas la valeur lue** |
| `sejour_arrivee.rs` | ★ Transaction unique · **concurrence par le parcours** · numérotation continue · fiche incomplète sans remplissage · P-09 ré-exercée |
| `sejour_orphelin.rs` | ★ Les quatre assertions du §0.7, **première cible en cinq cycles** |
| `sejour_depart.rs` | ★ Constat figé · **immuabilité par privilège** · dérive d'horloge · aucune clôture automatique |
| `sejour_prolongation.rs` | Conflit **nommé** avec alternatives · aucun déplacement partiel · P-09 par l'absurde |
| `sejour_hors_ligne.rs` | P-13 sur quinze opérations, les **trois** de classe A nommées |
| `accompagnant_classe_a.rs` | P-14, **troisième cible** |

Front : **532 tests verts**, dont `budget-gestes.spec.ts` — SC-001, déterministe.

### 2 · utoipa à jour, client TS régénéré sans diff ✅

**17 opérations**, 57 chemins au contrat. `pnpm porte:p01` ✓ — client identique au contrat.

### 3 · Migrations versionnées, `sqlx prepare` vert, seeds à jour ⚠️ **partiel**

Sept migrations (`0029` à `0035`), appliquées. `cargo sqlx prepare` à **deux passes** : 50 entrées
de la passe `backend/`, 43 de `backend/api/`, fusionnées hors de `.sqlx` entre les deux. Les **deux
contrôles** passent — aucune suppression, et le check hors ligne compile **après le `touch`**.

⛔ **Seeds non faits** (T077).

⚠️ **La numérotation dévie du plan d'un cran** : `0034` porte la charge utile de la file de
réconciliation, et le constat de taxe passe à `0035`. Motif au §« défauts trouvés ».

### 4 · RLS `ENABLE` + `FORCE` sur les 9 tables, isolation vérifiée ✅

`rls_catalogue.rs` : plancher relevé de 35 à **44**, atteint. `isolation_tenant.rs` : les
**14 chemins** du cycle déclarés `Regime::Isole`, avec leur motif.

### 5 · Classe hors-ligne déclarée pour les 9 entités, tests instanciés ✅

Registre v1.4.0 : cinq entités **honorées**, quatre lignes ajoutées. `outillage_classes.rs` ne
nomme plus aucune entité manquante. P-14 passe d'**une** cible à **trois**.

### 6 · Événements outbox pour chaque transition ✅

**Neuf types**, sur deux crates. `couverture_portes.rs` : 27 → **36**. Chacun a son test.

### 7 · Clés i18n `fr` ET `en`, aucune chaîne en dur ⚠️ **partiel**

`pnpm test:i18n` ✓ — catalogues à parité. Les clés de `R4` sont livrées ; celles des trois écrans
manquants ne le sont pas.

### 8 · Écrans vérifiés en clair et en sombre ⚠️ **partiel**

`R4` seulement. `theme-sombre.spec.ts` couvre ses trois composants, et **a attrapé une seconde
palette** que j'avais introduite.

### 9 · Aucun paramètre métier en dur ✅ **SANS OBJET, et c'est écrit**

Aucune story du périmètre ne dit « paramétrable » : **aucune clé nouvelle au catalogue**. Le point
est sans objet, et le dire vaut mieux que de le cocher.

Les deux seuils de la recherche — longueur du suffixe téléphonique, limite par défaut — sont des
**constantes nommées et commentées**, jamais des littéraux : ce ne sont pas des paramètres
d'établissement, mais leur révision doit être trouvable.

### 10 · Impression thermique ⛔ **NON SATISFAIT, et c'est nommé**

La note et la fiche de police sont **produites** et lisibles par l'API ; elles ne sont **pas
imprimées sur thermique réelle**. Cela relève d'**IMP, tranche T2**.

---

## Les portes de CI

| Porte | État | Note |
|---|---|---|
| P-01 · P-01b | ✅ | 73 `operationId` distincts |
| P-02 | ✅ | 35 migrations, aucune modifiée |
| P-03 | ✅ | Vert **alors que `socle/comptes` sert désormais une verticale** |
| P-04 | ✅ | **Paires sensibles** déclarées : `comptes × hebergement`, 70/94 requêtes |
| P-05 | ✅ | 36 types |
| P-05b | ✅ | **Trois** registres immuables — `taxe_sejour_constat` rejoint la catégorie |
| P-07 | ✅ | 38 tables |
| P-08 | ✅ | 73 opérations |
| P-09 | ✅ | **Ré-exercée par le parcours de séjour**, et par l'absurde au changement d'unité |
| P-10 | ✅ | Y compris dans le JSONB |
| P-11 | ✅ | **Verte à vide** — `fixtures/fiscal` ne contient que son `.gitkeep` |
| P-12 | ✅ | Aucune règle fiscale hors de l'adaptateur |
| P-13 | ✅ serveur / ⛔ écran | Le versant e2e ne couvre pas ce cycle (T082) |
| P-14 | ✅ | **Trois** cibles |
| P-15 · P-16 · P-17 | ✅ | |
| P-18 | ✅ | Double passe, deux contrôles |
| P-19 · P-20 · P-21 · P-21b | ✅ | |
| P-22 | ⚠️ **non exécutée** | Exige l'API, la base et les seeds — dépend de T077 |
| P-23 | ✅ | Périmètre découvert ; les quatre calculs lisent `now()` de la base |

---

## ★ Ce que ce cycle a TROUVÉ, et qui vaut plus que ce qu'il a construit

*Sur le modèle de `specs/005-.../revue-dod.md`. Six défauts, dont **quatre qu'aucune relecture
n'aurait vus**.*

### 1 · Deux arrivées concurrentes rendaient `500`, pas `409` ★

L'insertion spéculative (`ON CONFLICT`) combinée à la contrainte d'exclusion produit un
**interblocage PostgreSQL 40P01**. Au comptoir, Yao lisait « erreur interne » au lieu de « Cette
chambre est déjà prise ».

**Pourquoi le cycle 004 ne l'a pas vu** : son test de concurrence attribue par SQL direct, sans
`ON CONFLICT` — pas d'insertion spéculative, donc pas de jeton spéculatif à attendre. Le phénomène
n'existe que sur le chemin **idempotent**, celui que le parcours de séjour emploie. C'est P-09
ré-exercée **par le parcours réel** qui a payé.

**Et le nombre de réessais est mesuré, pas dérivé.** La première rédaction fixait deux, avec
l'argument qu'« après un réessai le conflit est établi ». La mesure l'a démentie : deux essais
laissaient passer un `500` sur quatre — le détecteur d'interblocage tourne **par processus**, les
deux transactions peuvent être abattues et se réessayer en même temps. Quatre tient dix fois sur
dix. *Un chiffre justifié par un raisonnement faux est plus dangereux qu'un chiffre mesuré, parce
qu'on ne le remesure jamais.*

### 2 · La file de réconciliation ne retenait rien de l'écriture perdue ★

Posée au cycle 005 comme provision, elle porte des identifiants et **aucune charge utile**. Or
quand un accompagnant arrive après la clôture, sa ligne **n'est pas écrite** : SYN-03 n'aurait eu
que des lignes vides à réconcilier, **deux cycles plus tard**.

Mode d'échec normal d'une table posée sans écrivain : rien ne pouvait le révéler avant qu'un
écrivain n'existe. Corrigé par la migration `0034`, qui prend le numéro que le plan réservait au
constat de taxe.

### 3 · Un accompagnant déclaré à l'arrivée n'émettait aucun événement ★

Le même fait — un accompagnant existe — émettait `sej.accompagnant.ajoute` par l'opération 11 et
**rien** par l'ouverture. Une projection bâtie dessus aurait manqué **la majorité** d'entre eux,
puisqu'on les déclare au comptoir.

*Le même fait produit le même événement, quel que soit le chemin qui l'a créé.*

### 4 · Les commentaires de `<template>` sont rendus dans le DOM ★

Ceux qui nommaient « Scanner la pièce », « Imprimer le reçu » et « encaissé en espèces »
atteignaient la page — lisibles par « afficher la source » — et faisaient échouer le contrôle
d'absence des éléments d'autres cycles, **pour une raison sans rapport avec ce qu'il mesure**.

### 5 · L'outillage de classe A fixait un rôle qui n'a plus tous les droits

`tester_classe_a!` employait `proprietaire`, « qui porte toutes les permissions ». Faux depuis la
migration `0030` : le propriétaire ne reçoit que les **lectures** de la fiche client. Le symptôme
aurait été un `403` accusant le handler alors que la cause est le rôle du harnais.

### 6 · La normalisation du téléphone retirait le zéro de tête

Par réflexe de **préfixe interurbain** — vrai en France, **faux en Côte d'Ivoire**, qui n'en a pas.
Le numéro national fait dix chiffres et s'écrit `+225 07 07 12 34 56`, zéro compris. Le retirer
produisait un chiffre de moins, et **une fiche introuvable dès qu'on la cherchait autrement qu'on
l'avait créée**. Attrapé par mon propre test unitaire.

---

## Cinq portes ont attrapé des ajouts, et chacune avait raison

| Porte | Ce qu'elle a attrapé |
|---|---|
| **P-17** | `dark:bg-ocre/10` — une **seconde palette**, ce que le principe XII interdit. Les jetons dédiés existaient |
| `permissions_par_module.rs` | Refusait qu'on ajuste sa constante **sans justifier** l'écart. Les trois sont désormais écrits |
| `theme-sombre.spec.ts` | `modules/sejours` n'était pas au périmètre inspecté |
| `audit_taxonomie.rs` | Le titre de section **porte le décompte en toutes lettres**, et le renommer a fait échouer cinq tests — exactement comme son commentaire l'annonce |
| `hebergement_hors_ligne.rs` | L'opération 17 est sous `/hebergement/` et échappait au balayage du cycle 004 |

---

## Décisions closes par ce cycle

| # | Décision | Arbitrage |
|---|---|---|
| **B-10** | Axe des personnes de la taxe de nuitée | **Par nuitée et par SÉJOUR**, jamais par personne. La question de l'exonération par personne **tombe avec elle** : aucune colonne de motif n'est due |
| **O-01** | Classe de `client` | **Option (a)** — classe C maintenue. Friction résiduelle **écrite** au §12 du registre plutôt que tue |

**B-02 reste ouverte** — elle porte l'axe des **nuits**, que B-10 ne touche pas. Les deux axes se
confondaient dans l'ancienne rédaction du §9.6, et c'est ce qui rendait B-10 illisible.
