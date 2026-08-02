# Phase 1 — Modèle de données : SYN-01, SYN-02, SYN-04

**Cycle** : 005 (SYN) · **Date** : 2026-08-02 · **Recherche** : [research.md](./research.md)

---

## Vue d'ensemble — ce que ce cycle ajoute en base, et ce qu'il n'ajoute pas

**Deux migrations, une seule table nouvelle, et elle est une provision.** C'est le trait le plus
inhabituel de ce cycle et il mérite d'être dit avant le détail : SYN ne crée presque rien en base.
Son objet est de rendre **opposable** ce qui existe déjà — les deux horodatages posés depuis le
cycle 001, les classes déclarées au registre, les identifiants fournis par le client — et de
livrer côté application le mécanisme qui les emploie.

| Migration | Contenu | Nature |
|---|---|---|
| `0027_reconciliation_orpheline.sql` | `synchronisation.reconciliation_orpheline` | **Provision** — table et contraintes, aucune logique |
| `0028_parametres_synchronisation.sql` | 2 clés au catalogue de paramètres | Référentiel |

**Aucune colonne d'horodatage n'est ajoutée** (R-03). L'horodatage d'autorité est `cree_le`, déjà
en place ; l'horodatage client est `horodatage_client`, déjà en place. Ce cycle ajoute
l'**interdiction vérifiée** d'employer le second dans une règle.

---

## 1. `synchronisation.reconciliation_orpheline` — provision SYN-03

### Objet

Le constat qu'une écriture est arrivée sur un agrégat déjà clos et facturé — le conflit que le
cadrage §11.4 nomme « le plus fréquent en exploitation réelle ». Sa **résolution est humaine et
obligatoire** : jamais de rejet silencieux, jamais d'ajout d'office.

**Ce cycle en crée la table et rien d'autre.** L'écran de réconciliation, le service, les
endpoints et la logique de résolution sont de SYN-03, tranche T3, et dépendent des séjours et des
documents fiscaux — dont aucun n'existe.

### Colonnes

| Colonne | Type | Contrainte | Motif |
|---|---|---|---|
| `id` | `UUID` | `PRIMARY KEY` | **Fourni par le client** — patron du module doré, ce qui rend le rejeu inoffensif |
| `tenant_id` | `UUID NOT NULL` | `REFERENCES etablissements.tenant (id)` | Principe III |
| `etablissement_id` | `UUID NOT NULL` | `REFERENCES etablissements.etablissement (id)` | Le constat est local à un établissement |
| `ecriture_id` | `UUID NOT NULL` | — | L'écriture arrivée en retard. **Aucune clé étrangère** : elle vit dans un autre schéma de module (principe II) |
| `ecriture_type` | `TEXT NOT NULL` | `CHECK` non vide | Le type d'opération, pour que le cycle SYN-03 sache quoi rattacher |
| `agregat_type` | `TEXT NOT NULL` | `CHECK` non vide | `sejour`, `addition`, `bon_de_depot` — nommé, jamais deviné |
| `agregat_id` | `UUID NOT NULL` | — | Idem, sans clé étrangère inter-schémas |
| `etat` | `TEXT NOT NULL` | `CHECK (etat IN ('constatee','resolue'))`, défaut `'constatee'` | Deux états, pas davantage |
| `issue` | `TEXT NULL` | `CHECK (issue IN ('AVOIR_REFACTURATION','PRISE_EN_CHARGE','RATTACHEMENT_SEJOUR_SUIVANT'))` | Les trois du cadrage §11.4, en **majuscules françaises** |
| `resolue_par_compte_id` | `UUID NULL` | — | Qui a tranché. Sans clé étrangère : `comptes` est un autre schéma |
| `horodatage_client` | `TIMESTAMPTZ NULL` | — | Indicatif. **Aucune règle ne s'y appuie** |
| `cree_le` | `TIMESTAMPTZ NOT NULL` | `DEFAULT now()` | **Horodatage d'autorité** |
| `resolue_le` | `TIMESTAMPTZ NULL` | — | Horodatage d'autorité de la résolution |

### La contrainte qui porte le cycle de vie

```sql
CHECK ((etat = 'resolue') = (issue IS NOT NULL AND resolue_le IS NOT NULL
                             AND resolue_par_compte_id IS NOT NULL))
```

**Une égalité de conditions, pas trois `CHECK` séparés.** Même patron que le classement
d'établissement du module doré : l'état et ses trois corollaires ne peuvent pas diverger. Un
`UPDATE` qui poserait `etat = 'resolue'` en oubliant l'issue serait refusé par la base, pas par
une revue.

### Sécurité au niveau ligne

Patron identique à toutes les tables du produit, sans exception :

```sql
ALTER TABLE synchronisation.reconciliation_orpheline ENABLE ROW LEVEL SECURITY;
ALTER TABLE synchronisation.reconciliation_orpheline FORCE  ROW LEVEL SECURITY;

CREATE POLICY isolation_tenant ON synchronisation.reconciliation_orpheline
    USING      (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);
```

### Privilèges — c'est ici que la provision se prouve

```sql
GRANT SELECT ON synchronisation.reconciliation_orpheline TO kaya_app;
-- Ni INSERT, ni UPDATE, ni DELETE. Voir ci-dessous.
```

**Le registre déclare deux classes pour cette entité — création en A, résolution en B — et les
deux restent justes.** Ce ne sont pas les classes qui sont différées, c'est l'implémentation.
Accorder l'`INSERT` dès maintenant serait exactement l'« ajout d'un petit endpoint » que
`backend/tests/provisions_sans_logique.rs` existe pour rendre bruyant : son décompte passe de
**cinq à six** provisions dans le même changement.

### Index

```sql
CREATE INDEX reconciliation_orpheline_a_traiter_idx
    ON synchronisation.reconciliation_orpheline (tenant_id, etablissement_id, cree_le DESC)
    WHERE etat = 'constatee';
```

Index **partiel** : la lecture réelle de SYN-03 est « ce qui reste à trancher », jamais
l'historique complet. `tenant_id` en tête parce que la politique filtre dessus à chaque accès.

---

## 2. Les deux paramètres au catalogue

Migration `0028`, sur le patron exact de `0023_parametres_hebergement.sql`.

| Clé | `type_valeur` | `portee_la_plus_basse` | Défaut | Story |
|---|---|---|---|---|
| `sync.derive_horloge_seuil_secondes` | `entier` | `etablissement` | `300` | SYN-04 |
| `sync.latence_degradee_seuil_ms` | `entier` | `etablissement` | `3000` | SYN-02 |

**Libellés et descriptions en langue utilisateur**, comme les trois clés du cycle 004 — le
catalogue est lu par l'écran de configuration, pas seulement par le code.

**Les deux sont ajoutées au récapitulatif de `docs/user-stories-v1.md` §708 dans le même
changement.** Le principe I·c en fait la condition, et le cycle 004 a montré que différer cet
ajout le fait oublier.

**Le premier défaut n'est pas une invention** : c'est la valeur du cadrage §11.4 (« alerte au-delà
de 5 minutes de dérive »), transformée en défaut de paramètre plutôt qu'en constante de code.

---

## 3. Ce qui existe déjà et que ce cycle emploie sans le modifier

| Élément | Où | Ce que ce cycle en fait |
|---|---|---|
| `etablissements.note_etablissement` | migration `0004` | Premier passager réel de la file. **Aucune modification** — la table a déjà l'`id` client, les deux horodatages, et `GRANT SELECT, INSERT` sans `UPDATE` ni `DELETE` |
| `notes_creer` / `notes_lister` | `backend/api/src/routes/notes.rs` | Employés tels quels. `CreerNoteRequete` porte déjà `id` et `horodatage_client` |
| `cree_le` sur les tables métier | cycles 001 → 004 | **Devient l'horodatage d'autorité nommé**, et l'usage de son pendant client devient une faute vérifiée |
| `synchronisation.evenement_outbox` | migration `0003` | Aucun changement. Le rejeu **n'y écrit rien** — c'est le point subtil du patron |
| `comptes.journal_audit` | migration `0017` | Reçoit une **famille nouvelle** : la dérive d'horloge |
| `etablissements.parametre_configuration` | migration `0011` | Accueille les deux surcharges par héritage |

---

## 4. Événements outbox

**Aucun type d'événement nouveau.**

C'est une décision, pas un oubli, et elle mérite son paragraphe. Ce cycle ne crée **aucune
transition d'état métier** : la file transporte des écritures dont les événements sont déjà émis
par leurs services respectifs, la table de réconciliation est une provision sans écriture, et la
dérive d'horloge n'est pas une transition d'état — c'est un **constat d'exploitation**, qui va au
registre des actions et non au grand livre.

Le total du produit reste donc à **27 types**, et `TYPES_EVENEMENTS` de
`backend/tests/couverture_portes.rs` est inchangé.

> **Le point qu'on écrirait mal, et qui est vérifié par un test dédié.** Un rejeu ne produit
> **aucun** nouvel événement. Trois envois de la même note laissent **une ligne et un événement**.
> L'émettre à chaque tentative ferait du grand livre le journal des tentatives réseau du terminal,
> et non celui des transitions d'état. Le service de note l'applique déjà ; ce cycle en fait un
> test de la macro `tester_classe_a!`, donc une garantie de **toutes** les entités de classe A à
> venir, pas seulement de celle-là.

---

## 5. Famille d'audit nouvelle — la dérive d'horloge

| Famille | Déclenchement | Contexte |
|---|---|---|
| `derive_horloge_constatee` | Écart absolu entre `horodatage_client` et `cree_le` supérieur au seuil, à l'ingestion d'une écriture | `{ ecart_secondes, seuil_secondes, sens }` — jamais de montant, donc aucune clé monétaire (P-10) |

**Débrayée par épisode, pas par écriture** (R-04) : une clé Redis à durée de vie portant
`(tenant, compte, appareil)` empêche deux cents entrées identiques pendant un service. La clé est
**éphémère reconstructible** au sens du principe II — la perdre produit une entrée d'audit de plus,
jamais une donnée manquante.

Conséquences mécaniques, dans le même changement :

- `docs/taxonomie-audit.md` reçoit la famille ;
- `FAMILLES_ATTENDUES` de `couverture_portes.rs` passe de **10 à 11** ;
- `TESTS_QUI_EXERCENT_L_AUDIT` reçoit le fichier de test qui l'exerce — le contrôle « toute
  famille branchée est exercée par un test » échouerait sinon, et c'est ce qu'on attend de lui.

---

## 6. Classes hors-ligne — le registre, version 1.3.0

`docs/registre-classes-offline.md` §5.6 déclare **déjà** les entités de ce cycle, depuis le
2026-07-30, et ses lignes sont **honorées, pas réécrites** — même règle qu'aux cycles 003 et 004 :

| Entité ou opération | Classe | Branche | Statut à ce cycle |
|---|---|---|---|
| `reconciliation_orpheline` — création de l'élément en file | **A** | A4 | Table créée, **écriture non accordée** (provision) |
| `reconciliation_orpheline` — résolution | **B** | B3 | Table créée, **écriture non accordée** (provision) |
| Horodatage d'autorité — attribution | **serveur uniquement** | — | **Devient opposable** : porte proposée P-23 |
| Horodatage client — enregistrement indicatif | **A** | A4 | Inchangé |
| `evenement_outbox` — écriture, marquage publié | **A** | A4 | Inchangé |

**Ce qui change dans le registre à ce cycle :**

1. **Le §5.6 devient effectif** — la note de version le consigne, comme le §5.2 au cycle 003 et le
   §7 au cycle 004.
2. **Aucune ligne ajoutée au §5.6.** Les deux tables créées y figurent déjà. C'est le cas le plus
   sain, et il est rare : il faut le dire, sans quoi une relecture y verrait un oubli.
3. **Le §11 « Tests obligatoires par classe » gagne un paragraphe** : les tests qu'il impose
   existent désormais sous forme d'outillage, et chaque entité les **instancie** au lieu de les
   réécrire. C'est le changement de fond de ce cycle sur ce document.
4. **`classes_offline.rs` cesse d'énumérer ses schémas** et lit `perimetre::schemas_applicatifs()`.
   `SCHEMAS_APPLICATIFS` et `TABLES_ATTENDUES` en dur disparaissent ; le décompte reste, comparé à
   ce que la découverte trouve.

---

## 7. Entités côté application — la file et son état

Aucune table, mais un modèle qui doit être écrit : c'est ce qui vit sur le terminal.

### `EntreeFile` — étendue

L'interface existe (`app/core/sync/classes.ts`) et gagne ce qui manque à une file persistante :

| Champ | Type | Motif |
|---|---|---|
| `id` | `string` | UUID v7 **client** — existant |
| `type` | `string` | Type d'opération déclaré classe A — existant |
| `horodatageClient` | `string` | Indicatif — existant |
| `charge` | `OperationClasseA<T>` | Marquée au niveau du type — existant |
| `contexte` | `{ tenantId, etablissementId }` | **Nouveau.** Le contexte **au moment de la saisie** : changer d'établissement pendant une coupure ne réattribue jamais une écriture déjà enfilée |
| `tentatives` | `number` | **Nouveau.** Alimente l'intervalle croissant, et le diagnostic de `S1` |

**Ce que `EntreeFile` ne porte toujours pas, et c'est délibéré : aucun jeton.** Un jeton mis en
file serait périmé au retour, et le ranger prolongerait la durée de vie d'un secret sur un
terminal qu'on peut perdre. L'absence de champ est ce qui l'empêche.

### `EntreeQuarantaine` — nouvelle

| Champ | Type | Motif |
|---|---|---|
| `entree` | `EntreeFile` | L'écriture telle qu'elle était |
| `code` | `string` | Le `code` d'erreur du serveur — **jamais le `message`**, qui nomme des tables et parle anglais technique |
| `refuseeLe` | `string` | Quand le refus est tombé |

L'interface branche sa clé i18n sur `code`. C'est la règle déjà posée par le lexique, et la
quarantaine n'y déroge pas.

### `EtatSynchronisation` — nouvelle, réactive

```text
{ reseau: 'connecte' | 'degrade' | 'hors_ligne', enAttente: number, enQuarantaine: number }
```

Source unique du témoin (composant 10) et du panneau `S1`. **Trois états, jamais un pourcentage** —
la règle du composant est explicite.

### « Le serveur fait foi en conflit » — sans objet à ce cycle, et il faut le dire

FR-020 reprend un MUST du principe VI : le serveur tranche, et « dernier écrit gagne » n'est
autorisé que sur les entités A **sans conséquence**. **Aucune tâche ne l'implémente, et c'est
correct** — écrire pourquoi vaut mieux que laisser l'exigence sans couverture apparente.

La file ne transporte que des opérations de **classe A**, dont les entités sont **append-only** :
`note_etablissement` n'accorde ni `UPDATE` ni `DELETE` à `kaya_app`, et `journal_audit` non plus.
Deux écritures concurrentes ne peuvent donc pas se recouvrir — il n'y a pas de « dernier écrit »,
seulement deux lignes distinctes, ou une seule si l'identifiant est le même.

**Le cas se rouvrira à deux endroits nommés**, et aucun n'est de ce cycle : `unite.statut_menage`
(HEB-06), **seul cas du produit où le registre autorise dernier-écrit-gagne**, et toute opération
de classe B empruntant la file en mode nœud de site (incrément 3).

### Stockage local

| Élément | Où | Garantie |
|---|---|---|
| Clé de chiffrement de la file | `PlatformAdapter.stockageSecurise` | `coffre_systeme` sur desktop, Android, iOS ; **`aucune` sur web**, et le type le dit |
| Cryptogramme de la file | Stockage persistant ordinaire de la plateforme | Illisible sans la clé |

**La garantie du web n'est pas maquillée.** `NiveauGarantieStockage` porte le niveau **dans le
type** précisément pour que l'appelant le lise avant d'y ranger quoi que ce soit. Le produit
déclare `aucune` sur web parce que c'est vrai, et la contrepartie est portée ailleurs — purge à la
déconnexion, rotation des jetons, coupure depuis « Appareils connectés ».
