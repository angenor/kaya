# Kaya — Taxonomie du registre des actions

*Source de vérité des types d'action tracés au journal d'audit (`comptes.journal_audit`).
Créé par le cycle 003 (CPT), story **CPT-04**.*

**Version 1.2.0** — 2026-08-02. **Onze** familles, **4 branchées**, 7 dues.

> **La douzième est `consultation_piece_identite`**, ajoutée par le cycle 006 (SEJ-01) : la
> première qui trace une **lecture** et non une modification. Voir sa section dédiée.
>
> **La onzième est `derive_horloge_constatee`**, ajoutée par le cycle 005 (SYN-04). Elle ne trace
> pas un geste d'utilisateur mais un **constat d'exploitation** : l'heure d'un terminal s'écarte de
> celle du serveur au-delà du seuil paramétré. C'est la première famille de cette nature, et le §
> « Ce que la taxonomie n'est pas » dit pourquoi elle a quand même sa place ici plutôt qu'au grand
> livre. Elle a été déclarée **due** le temps d'un commit, puis **branchée** dans celui qui l'écrit
> — `api/src/derive.rs`, câblé sur le service de note par la couche API. Les deux temps ne sont pas
> une formalité : c'est `audit_taxonomie.rs` qui les impose, en faisant échouer le build dès qu'un
> type déclaré « dû » acquiert un chemin d'écriture.

---

## À quoi sert ce document

CPT-04 énumère dix familles d'actions à tracer. **Huit d'entre elles n'ont aucun chemin d'écriture
au cycle 003** : la remise n'existe pas encore, l'avoir non plus, le tiroir-caisse n'est pas
branché. Les inscrire quand même a une raison précise, et c'est la même qu'au cycle 002 pour les
étapes dues du parcours d'agnosticité :

> Une liste de choses à faire qui vit dans une spécification se perd. Une liste qui vit dans un
> **harnais de test** ne se perd pas — elle fait échouer le build le jour où quelqu'un branche un
> type sans le déclarer.

Le harnais est `backend/tests/audit_taxonomie.rs`. Il lit **ce fichier** et le compare au code.

**Terme utilisateur : « Registre des actions »** (`docs/design/lexique.md`). « Journal d'audit » est
le nom technique — table, permission, endpoint — et n'apparaît jamais à l'écran.

---

## Les deux états, et ce qu'ils engagent

| État | Ce qu'il signifie | Ce que le harnais vérifie |
|---|---|---|
| **branché** | Un chemin de code écrit une entrée de ce type | Le chemin **existe** |
| **dû** | La story qui l'apportera est nommée, aucun chemin n'existe | Le chemin **n'existe pas** |

**Les deux sens comptent.** Un type déclaré `dû` qui acquiert un chemin d'écriture fait échouer le
build : c'est ce qui oblige à revenir ici. Un type déclaré `branché` sans chemin fait échouer le
build aussi : sans quoi il suffirait de tout déclarer branché pour rendre le harnais muet.

---

## Les douze familles

| # | Code | Ce que ça trace | État | Story qui la doit |
|---|---|---|---|---|
| 1 | `remise` | Une remise accordée sur une ligne ou une note | **dû** | PDV-03 / SEJ-03 — tranche T2 |
| 2 | `annulation_ligne_envoyee` | L'annulation d'une ligne **déjà partie en cuisine ou au bar** | **dû** | PDV-03 — tranche T2 |
| 3 | `avoir` | L'émission d'un avoir sur une facture certifiée | **dû** | FIS-06 — tranche T3 |
| 4 | `ouverture_tiroir` | Une ouverture de tiroir-caisse hors encaissement | **dû** | IMP-01 — tranche T2 |
| 5 | `modification_tarif` | Le changement du prix d'un article vendable | **dû** | PDV-01 — tranche T2 |
| 6 | `suppression` | La mise hors service de ce qui ne se supprime jamais | **branché** | **CPT-01 — ce cycle** |
| 7 | `changement_role` | Une attribution ou un retrait de rôle | **branché** | **CPT-02 — ce cycle** |
| 8 | `ecart_caisse` | Un écart constaté au comptage de fin de shift | **dû** | CAI-04 — tranche T2 |
| 9 | `rebascule_palier_passage` | Le passage automatique au palier tarifaire supérieur | **branché** | **HEB-04 — cycle 004** |
| 10 | `forcage_disponibilite` | L'attribution d'une unité que le système déclarait indisponible | **dû** | HEB — tranche T1 |
| 11 | `derive_horloge_constatee` | L'heure d'un terminal s'écarte de celle du serveur au-delà du seuil | **branché** | **SYN-04 — cycle 005** |
| 12 | `consultation_piece_identite` | ★ La **consultation** d'un numéro de pièce d'identité | **branché** | **SEJ-01 — ce cycle** |

**Deux d'entre elles sont dues par ce cycle même**, et c'est délibéré : ce document a été écrit
**avant** la première migration, donc avant que `changement_role` et `suppression` aient un chemin.
Leur passage à `branché` se fait dans le changement qui les branche, pas avant.

**`suppression` est passée à branché en T028**, avec le service d'authentification — et **pas là
où on l'attendait**. Le document annonçait la désactivation de compte (opération 13, T041) ; c'est
la **révocation de session** qui a branché le type la première. Les deux sont des mises hors
service, les deux sont dues au même cycle, et le harnais a signalé l'écart au moment exact où le
premier chemin d'écriture est apparu. C'est son travail, et c'est ce que valait de l'écrire vert à
vide.

**`changement_role` est passée à branché en T039**, là où le document l'annonçait : le service
d'attribution et de retrait de `socle/comptes/src/roles/service.rs` écrit une entrée dans la
**même transaction** que la ligne et que l'événement outbox (FR-024). Deux gestes, deux entrées —
`contexte.sens` vaut `attribution` ou `retrait`, et jamais autre chose. C'est aussi ce qui
explique que `compte_role` n'ait pas de privilège `UPDATE` : changer un rôle est un retrait suivi
d'une attribution, donc deux lignes au registre, pas une modification silencieuse.

**`rebascule_palier_passage` est passée à branché au cycle 004**, avec le moteur de tarification
du passage. Le dépassement constaté au départ fait changer de palier ; la différence est ajoutée à
la note, et l'entrée d'audit porte la durée constatée et les **deux paliers** — celui qui avait été
vendu et celui qui s'applique. Le passage est aujourd'hui massivement encaissé en espèces sans
trace : c'est précisément cette famille qui donne au propriétaire la visibilité que le cadrage §5.6
lui promet, et c'est aussi la raison pour laquelle elle rencontrera de la résistance.

> **Le harnais a signalé l'écart avec un cycle de retard, et il faut le dire.** Le chemin
> d'écriture est apparu en T042, mais `backend/tests/audit_taxonomie.rs` n'a été relancé qu'au
> recollement T049 : le test était **rouge dans l'arbre** entre les deux, sans que rien ne le
> montre, parce que les tâches intermédiaires lançaient des suites ciblées. La porte a fait son
> travail ; c'est la fréquence à laquelle on l'interroge qui a manqué. `cargo test --workspace`
> avant chaque commit de fin de phase est le remède, et non une porte de plus.

**Cinq sur douze sont branchées à la clôture du cycle 006** — `suppression`, `changement_role`,
`rebascule_palier_passage`, `derive_horloge_constatee` et `consultation_piece_identite` —, et les
sept autres restent dues aux
tranches T1 à T3 — c'est exactement ce que le document annonçait, et le harnais l'a vérifié à
chaque étape.

### ★ `consultation_piece_identite` — la première famille qui trace une LECTURE

**Elle naît « branchée », avec son chemin de code**, dans le cycle qui la crée. Une famille
déclarée « branchée » sans chemin de code ferait échouer le harnais ; une famille branchée non
déclarée l'échouerait aussi. Les deux sens sont vérifiés par `backend/tests/audit_taxonomie.rs`.

| | |
|---|---|
| Ce qui la déclenche | Toute lecture d'une fiche client dont le numéro de pièce est **déchiffré** — `client_lire`, et le rattachement à une fiche de police |
| Où | `backend/crates/socle/comptes/src/client/service.rs` |
| Cible | `personne` — la personne **consultée**, jamais l'auteur |
| Contexte écrit | `{ motif }` — **aucune clé monétaire**, et surtout **jamais la valeur lue** |
| Ce qu'elle ne fait **jamais** | Refuser la lecture. Le journal surveille, il n'autorise pas — l'autorisation est `sej.client.lire` |

> ⚠️ **Le contexte ne porte JAMAIS le numéro consulté.** Recopier le numéro dans un registre
> **immuable et à rétention illimitée** (P-05b) créerait exactement la fuite que ce journal existe
> pour surveiller, et rendrait inapplicable la rétention de 90 jours de TRX-06 sur la copie : la
> donnée serait purgée d'un côté et conservée pour toujours de l'autre.

> **Pourquoi une famille nouvelle plutôt qu'une existante.** Aucune des onze ne couvre une
> consultation : `suppression` trace une mise hors service, `changement_role` une attribution —
> **toutes tracent un geste qui modifie**. Une lecture n'en est pas un, et la ranger sous une
> famille existante rendrait le registre illisible au propriétaire, qui est son public.

### `forcage_disponibilite` — reste « due », et le dire vaut mieux que la laisser ambiguë

La famille n° 10 décrit *« l'attribution d'une unité que le système déclarait indisponible »*.
**Le cycle 006 ne livre aucun forçage** : un changement d'unité vers une chambre occupée est
**refusé**, avec le conflit nommé (FR-080), et la prolongation qui bute sur une occupation
suivante l'est aussi. La ligne reste donc « due ».

C'est écrit ici parce que le cycle 006 est celui où l'on s'attendrait à la voir passer : il touche
l'attribution d'unités de bout en bout. Ne rien dire laisserait croire à un oubli.

### `derive_horloge_constatee` — la première famille qui ne trace aucun geste

Les dix premières familles tracent ce qu'**une personne a fait**. La onzième trace ce qu'un
**appareil est** : son horloge s'écarte de celle du serveur. Elle a sa place ici et non au grand
livre parce que son public est le même que celui des dix autres — Adjoua et M. Koffi, qui doivent
pouvoir constater après coup quel terminal déviait pendant le service, et non une projection.

| Ce qui la déclenche | Écart **absolu** entre `horodatage_client` et l'horodatage d'autorité supérieur à `sync.derive_horloge_seuil_secondes` (défaut 300) |
|---|---|
| Contexte écrit | `{ ecart_secondes, seuil_secondes, sens }` — **aucune clé monétaire**, ce que la porte P-10 vérifie jusque dans le JSONB |
| Ce qu'elle ne fait **jamais** | Refuser l'écriture. La dérive est signalée, pas opposée (FR-036) |
| Fréquence | **Une entrée par épisode, pas une par écriture.** Deux cents saisies pendant un service produiraient deux cents entrées identiques et le registre deviendrait illisible, donc inutilisé. Le débrayage passe par une clé Redis à durée de vie portant `(tenant, compte, appareil)` — **éphémère reconstructible** au sens du principe II : la perdre produit une entrée de plus, jamais une donnée manquante |

**Le mot « dérive » n'atteint jamais l'écran.** L'utilisateur lit « L'heure de cet appareil retarde
de {n} minutes », ou « avance », suivi de la phrase qui le rassure sur ce qui va bien —
`docs/design/lexique.md` fait foi, et les **deux sens** sont dus puisque la détection porte sur la
valeur absolue.

### Ce que `suppression` recouvre — et pourquoi le mot est faux mais gardé

**Rien ne se supprime jamais dans Kaya** (FR-014, principe VI). Un compte se désactive, un service
se retire, une ligne s'annule par une contre-ligne. Le type garde pourtant le nom `suppression`
parce que c'est **le geste que l'utilisateur croit faire**, et que le registre est lu par un
propriétaire qui cherche « qui a supprimé ça ». Le lexique traduit ; la taxonomie nomme l'intention.

Au cycle 003, `suppression` trace **trois gestes**, tous des mises hors service :

| Geste | Où | Cible |
|---|---|---|
| Révoquer une session — « Déconnecter cet appareil » | `authentification/service.rs`, T028 | `session` |
| Révoquer une famille de jetons sur **réutilisation détectée** | idem | `session` |
| Désactiver un compte (`compte_changer_etat`, opération 13) | T041 | `compte` |

`cible_type` les distingue — `session` ou `compte` —, ce qui permet au filtre de `G4` de les
séparer sans multiplier les familles. **Une famille par geste ferait une taxonomie de trente
entrées dont personne ne connaîtrait la moitié**, et le filtre d'un registre ne vaut que si son
vocabulaire tient dans une liste déroulante.

Les cycles suivants y logeront le retrait d'un article du catalogue et la mise hors service d'une
unité.

---

## Ce que la taxonomie n'est pas

**Ce n'est pas la liste des événements outbox.** Les deux registres sont distincts (research R-08) :

| | Journal d'audit | Grand livre (outbox) |
|---|---|---|
| Public | Le **propriétaire**, dans l'interface | Les **projections**, en interne |
| Contenu | Ce qu'une personne a fait | Une transition d'état |
| Classe | **A** — l'entrée s'écrit hors ligne avec l'action qu'elle trace | Celle de l'opération tracée |
| Granularité | Onze familles, stables sur la vie du produit | Un type par transition, **vingt-deux** à ce cycle (13 + 9 ; `compte.modifie` est déclaré sans émetteur et n'en fait pas vingt-trois) |

Une attribution de rôle produit **les deux** : l'événement `role.attribue` et l'entrée d'audit
`changement_role`, dans la même transaction. Ce n'est pas une redondance — l'un alimente les
projections, l'autre est un produit que M. Koffi achète.

**Ce n'est pas non plus la liste des actions journalisées techniquement.** Une connexion, un
rafraîchissement de session et un échec d'authentification vont aux **journaux applicatifs**, jamais
ici (research R-15). Le registre est permanent et à rétention illimitée : y écrire les connexions y
écrirait la liste horodatée des présences du personnel.

---

## Ajouter une famille

Une **douzième** famille se justifie par une story, pas par une intuition — la onzième l'a été par
SYN-04, et le tableau la nomme. Le cas normal est l'inverse :
faire passer une famille de `dû` à `branché`, dans le **même changement** que le code qui l'écrit.

1. Le type est ajouté à l'énumération `TypeActionAudit`
   (`backend/crates/socle/comptes/src/audit/taxonomie.rs`) — **une énumération fermée**, jamais un
   `String` : un `String` laisserait un cycle inventer `remise_appliquee` à côté de `remise`, et le
   filtre de l'écran `G4` cesserait de trouver la moitié des entrées sans que rien n'échoue.
2. Sa ligne passe à **branché** ici.
3. `cargo test --test audit_taxonomie` constate l'accord.

## Voir aussi

- `specs/003-comptes-roles-audit/data-model.md` §8 — la table et ses trois index de filtre
- `docs/registre-classes-offline.md` — `journal_audit` en classe **A**
- `docs/design/lexique.md` — « Registre des actions », et les mots qui n'atteignent jamais l'écran
