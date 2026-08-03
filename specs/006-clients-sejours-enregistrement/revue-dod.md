# Revue Definition of Done — cycle 006 · Fiches clients, arrivée, départ et prolongation

**Date** : 2026-08-03 · **Branche** : `006-clients-sejours-enregistrement`
**Référence** : `docs/user-stories-v1.md` §0.4, les dix points.

> **Ce document dit ce qui n'est pas satisfait avant de dire ce qui l'est.** Une revue qui
> commence par les réussites se lit en diagonale ; celle-ci commence par les manques, parce que ce
> sont eux qui décident si le cycle est livrable.

---

## ⛔ Ce qui n'est PAS conforme — trois manques, nommés

### 1 · Trois écrans sur quatre ne sont pas codés

| Écran | État | Ce qui existe déjà |
|---|---|---|
| `R4` Le passage | ✅ **codé**, cinq états, route `/passage` | Tout |
| `R3` Arrivée | ⛔ **non codé** | Son API : opérations 7 à 12, testées |
| `R7` La note et le départ | ⛔ **non codé** | Son API **complète** : départ, prolongation, changement d'unité, constat figé |
| `R5` Fiche client et recherche | ⛔ **non codé** | Son API : recherche trois formes, fiche, historique, préférences |

**Ce que cela veut dire concrètement** : Yao peut enregistrer un passage en deux gestes sur un vrai
navigateur, et **ne peut pas** faire partir un client autrement que par l'API. La démonstration de
fin de tranche T1 (T084) n'est donc **pas déroulable en l'état** — elle demande l'arrivée d'un
client en chambre pour deux nuits, puis un passage.

⚠️ **`docs/design/derivation.md` n'a PAS été mis à « codé »** pour ces trois écrans, et c'est
délibéré : ce document est **opposable** — la porte P-19 s'en sert pour autoriser un écran sans
maquette. Y inscrire « codé » sur un écran qui n'existe pas ferait mentir le seul document qui dise
ce qui a le droit d'être codé, et le mensonge serait invisible : rien ne relit ce tableau contre le
système de fichiers.

**Le cycle a livré le fond avant la forme**, ce qui est l'inverse de son ordre habituel. C'est un
état inhabituel et il vaut d'être nommé plutôt que découvert.

### 2 · Quatre tâches de fin de cycle ne sont pas exécutées

| Tâche | Ce qui manque | Conséquence |
|---|---|---|
| **T077** — seeds | 12 fiches, 3 séjours, dont un clos avec son constat figé | La démo n'a pas de données |
| **T082** — balayage hors ligne | Les quatre routes du cycle dans `tests-e2e/hors-ligne.spec.ts` | Le versant **écran** de P-13 ne couvre pas ce cycle |
| **T083** — mesure terrain | `mesures-terrain.md` — chronométrage humain de FR-106 | La cible des 30 s / 60 s n'est pas mesurée |
| **T084** — démo T1 | Les six étapes de `quickstart.md` §9 | Dépend des écrans manquants |

⚠️ **T082 est le plus coûteux des quatre à laisser en l'état.** Le versant *type* de P-13 est
couvert — `sejour_hors_ligne.rs` inspecte les quinze opérations servies — mais le versant *écran*,
celui qui vérifie que l'indisponibilité est annoncée **avant la saisie**, ne voit pas `/passage`.
Le cycle 005 a montré qu'un balayage hors ligne peut passer au vert en n'inspectant **rien** :
neuf cas verts, neuf fois le même écran de connexion.

### 3 · Une dépendance nouvelle, contre l'annonce du plan

`aes-gcm =0.11.0`. Le plan annonçait « aucune dépendance nouvelle » — **cette phrase est antérieure
à la tâche T018a**, ajoutée après la conception parce que le plan écrivait que le numéro de pièce
est « protégé au repos et son accès journalisé, **dès ce cycle** » et qu'**aucune tâche ne le
faisait**.

Version vérifiée sur `https://crates.io/api/v1/crates/aes-gcm` le 2026-08-03, jamais proposée de
mémoire. **À porter au gel §3.1 à la revue mensuelle du 2026-08-31** — c'est la seule entrée que
cette revue aura à trancher du fait de ce cycle. Deux voies écartées, avec leurs motifs, au
`Cargo.toml` du workspace.

---

## Les dix points de la Definition of Done

### 1 · Critères d'acceptation couverts par des tests ⚠️ **partiel**

**455 tests backend verts**, dont neuf fichiers écrits par ce cycle. Les critères **serveur** sont
couverts ; les critères **d'écran** des trois écrans manquants ne le sont pas.

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
