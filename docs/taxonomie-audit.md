# Kaya — Taxonomie du registre des actions

*Source de vérité des types d'action tracés au journal d'audit (`comptes.journal_audit`).
Créé par le cycle 003 (CPT), story **CPT-04**.*

**Version 1.0.0** — 2026-08-01. Dix familles, **0 branchée, 10 dues**.

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

## Les dix familles

| # | Code | Ce que ça trace | État | Story qui la doit |
|---|---|---|---|---|
| 1 | `remise` | Une remise accordée sur une ligne ou une note | **dû** | PDV-03 / SEJ-03 — tranche T2 |
| 2 | `annulation_ligne_envoyee` | L'annulation d'une ligne **déjà partie en cuisine ou au bar** | **dû** | PDV-03 — tranche T2 |
| 3 | `avoir` | L'émission d'un avoir sur une facture certifiée | **dû** | FIS-06 — tranche T3 |
| 4 | `ouverture_tiroir` | Une ouverture de tiroir-caisse hors encaissement | **dû** | IMP-01 — tranche T2 |
| 5 | `modification_tarif` | Le changement du prix d'un article vendable | **dû** | PDV-01 — tranche T2 |
| 6 | `suppression` | La mise hors service de ce qui ne se supprime jamais | **dû** | **CPT-01 — ce cycle** |
| 7 | `changement_role` | Une attribution ou un retrait de rôle | **dû** | **CPT-02 — ce cycle** |
| 8 | `ecart_caisse` | Un écart constaté au comptage de fin de shift | **dû** | CAI-04 — tranche T2 |
| 9 | `rebascule_palier_passage` | Le passage automatique au palier tarifaire supérieur | **dû** | HEB-04 — tranche T1 |
| 10 | `forcage_disponibilite` | L'attribution d'une unité que le système déclarait indisponible | **dû** | HEB — tranche T1 |

**Deux d'entre elles sont dues par ce cycle même**, et c'est délibéré : ce document est écrit
**avant** la première migration, donc avant que `changement_role` et `suppression` aient un chemin.
Leur passage à `branché` se fait dans le changement qui les branche, pas avant.

### Ce que `suppression` recouvre — et pourquoi le mot est faux mais gardé

**Rien ne se supprime jamais dans Kaya** (FR-014, principe VI). Un compte se désactive, un service
se retire, une ligne s'annule par une contre-ligne. Le type garde pourtant le nom `suppression`
parce que c'est **le geste que l'utilisateur croit faire**, et que le registre est lu par un
propriétaire qui cherche « qui a supprimé ça ». Le lexique traduit ; la taxonomie nomme l'intention.

Au cycle 003, `suppression` trace la **désactivation d'un compte** (`compte_changer_etat`, opération
13 du contrat). Les cycles suivants y logeront le retrait d'un article du catalogue et la mise hors
service d'une unité.

---

## Ce que la taxonomie n'est pas

**Ce n'est pas la liste des événements outbox.** Les deux registres sont distincts (research R-08) :

| | Journal d'audit | Grand livre (outbox) |
|---|---|---|
| Public | Le **propriétaire**, dans l'interface | Les **projections**, en interne |
| Contenu | Ce qu'une personne a fait | Une transition d'état |
| Classe | **A** — l'entrée s'écrit hors ligne avec l'action qu'elle trace | Celle de l'opération tracée |
| Granularité | Dix familles, stables sur la vie du produit | Un type par transition, vingt et un à ce cycle |

Une attribution de rôle produit **les deux** : l'événement `role.attribue` et l'entrée d'audit
`changement_role`, dans la même transaction. Ce n'est pas une redondance — l'un alimente les
projections, l'autre est un produit que M. Koffi achète.

**Ce n'est pas non plus la liste des actions journalisées techniquement.** Une connexion, un
rafraîchissement de session et un échec d'authentification vont aux **journaux applicatifs**, jamais
ici (research R-15). Le registre est permanent et à rétention illimitée : y écrire les connexions y
écrirait la liste horodatée des présences du personnel.

---

## Ajouter une famille

Une onzième famille se justifie par une story, pas par une intuition. Le cas normal est l'inverse :
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
