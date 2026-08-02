# Contrat HTTP — cycle 004 (HEB)

**Le contrat OpenAPI est généré par utoipa depuis le code, jamais écrit à la main** (principe I·a).
Ce document décrit ce que le code doit produire ; `openapi.json` fait foi, et la porte **P-01**
fait échouer le build sur tout écart entre le client TypeScript généré et le client commité.

**Treize opérations nouvelles.** Le total du contrat passe de **43 à 56**, et
`backend/tests/couverture_portes.rs` compare ce nombre aux opérations réellement inspectées par
P-08 (isolation multi-tenant) et P-01b (`operationId` distincts).

---

## Rappels de forme, tenus par le module doré

- **Le chemin n'est écrit qu'une fois** : `#[utoipa::path(...)]` **sans** `path` ni verbe ; les
  deux sont déduits de l'attribut de routage Actix (feature `actix_extras`). Les écrire deux fois
  laisserait le contrat annoncer une adresse que le serveur ne sert pas.
- **Monter par `service(...)`, jamais par `route(...)`** : `utoipa-actix-web` ne collecte les
  chemins que depuis `service(...)`. Un endpoint monté par `route(...)` serait servi sans figurer
  au contrat — absent du client généré, et invisible pour P-08.
- **`operation_id` explicite sur chaque opération** (P-01b) : deux opérations homonymes produisent
  un client TypeScript invalide, que P-01 ne détecte pas puisqu'elle ne compare que le généré au
  commité.
- **Aucun détail interne ne franchit la frontière** : ni message PostgreSQL, ni nom de table, ni
  trace. Le détail part dans les journaux, corrélé par l'identifiant de requête.
- **L'interface branche sa clé i18n sur le `code`, jamais sur le `message`** (lexique) — le
  `message` nomme des tables et parle anglais technique.

---

## 1. Le référentiel — `/etablissements/{etablissement_id}/hebergement`

| # | `operation_id` | Verbe | Chemin | Permission |
|---|---|---|---|---|
| 1 | `hebergement_lister_categories` | `GET` | `/categories` | `heb.offre.lire` |
| 2 | `hebergement_creer_categorie` | `POST` | `/categories` | `heb.offre.gerer` |
| 3 | `hebergement_modifier_categorie` | `PUT` | `/categories/{categorie_id}` | `heb.offre.gerer` |
| 4 | `hebergement_lister_unites` | `GET` | `/unites` | `heb.offre.lire` |
| 5 | `hebergement_creer_unite` | `POST` | `/unites` | `heb.offre.gerer` |
| 5b | `hebergement_modifier_unite` | `PUT` | `/unites/{unite_id}` | `heb.offre.gerer` |
| 6 | `hebergement_lister_formules` | `GET` | `/formules` | `heb.offre.lire` |
| 7 | `hebergement_creer_formule` | `POST` | `/formules` | `heb.offre.gerer` |
| 8 | `hebergement_modifier_formule` | `PUT` | `/formules/{formule_id}` | `heb.offre.gerer` |

### Opération 5b — corriger une unité : **deux champs, et pas un de plus**

```jsonc
// PUT /unites/{unite_id} — le corps ne porte QUE ces deux champs
{ "code": "B3", "etage": 1 }
```

**Le registre des classes borne l'opération**, et c'est lui qui fait autorité :
`docs/registre-classes-offline.md` §7.1 classe littéralement « `unite` (spécialisation de
`ressource_reservable`) — **code, étage** » en classe C. Ces deux champs sont donc déjà couverts
en écriture, création **comme** correction. Servir cette opération n'étend pas le périmètre : elle
sert ce que le registre a classé.

**Sans elle, une unité mal nommée puis occupée deviendrait définitive** — la suppression est
impossible dès qu'une occupation la référence.

**Les trois autres champs sont classés ailleurs, et le corps les REFUSE explicitement** :

| Champ | Pourquoi il n'est pas ici |
|---|---|
| `categorie_id` | Changer la catégorie change les formules applicables, **donc les tarifs**. Ce n'est pas une correction mais une opération métier à effet fiscal, **que le registre ne classe nulle part**. Si le besoin existe — une chambre rénovée qui monte de gamme —, il **se spécifie**, il ne se glisse pas dans un `PUT` de correction |
| `statut_menage` | **Classe A**, dernier-écrit-gagne — HEB-06 (P1). Autre chemin, autre classe, autre cycle |
| Mise hors service | **Classe B** — HEB-06 (P1). Elle retire une ressource de la disponibilité : c'est une opération **de disponibilité**, pas de référentiel |

Un corps portant l'un de ces trois champs est **refusé**, jamais ignoré silencieusement.

**Opération 6 — la seule que l'écran `G2` consomme en lecture.** Elle rend les formules avec leur prix
d'appel, leur famille, leur assujettissement à la taxe et — pour le passage — le premier palier de
leur barème. C'est ce que `G2` affiche.

```jsonc
// GET /formules → 200
[
  {
    "id": "0198f2c1-...",
    "categorie_id": "0198f2b0-...",
    "famille": "NUITEE",
    "prix_mineur": 12500,          // ENTIER d'unité mineure (P-10)
    "devise": "XOF",               // au même niveau, toujours
    "assujettie_taxe_nuitee": true,
    "regle_conversion_taxe": "une_nuitee_par_occupation",
    "heure_arrivee_standard": "14:00",
    "heure_depart_standard": "12:00"
  },
  {
    "famille": "PASSAGE",
    "prix_mineur": 1500,           // premier palier — « à partir de 1 500 F l'heure »
    "devise": "XOF",
    "assujettie_taxe_nuitee": false, // constat d'exploitation — ACTIVABLE par l'opération 8
    "regle_conversion_taxe": null,   // permis SEULEMENT parce que non assujettie
    "paliers": [
      { "duree_minutes": 60,  "prix_mineur": 1500 },
      { "duree_minutes": 120, "prix_mineur": 2800 },
      { "duree_minutes": 180, "prix_mineur": 4000 },
      { "duree_minutes": 240, "prix_mineur": 5000 }
    ],
    "prix_heure_supplementaire_mineur": 1200
  }
]
```

> **`regle_conversion_taxe: null` n'apparaît que sur une formule non assujettie.** La contrainte
> `formule_regle_fiscale_coherente` rend l'autre combinaison impossible à enregistrer — ce qui
> **supprime le besoin d'un troisième état d'écran**. Les deux mentions maquettées suffisent :
> « Taxe de séjour comprise dans le prix » et « Pas de taxe de séjour sur cette formule ».
>
> Le type TypeScript généré doit néanmoins être `string | null`, pas `string | undefined` : le
> champ est toujours présent dans la réponse.
>
> **L'opération 8 porte les deux champs fiscaux**, gardée par `heb.offre.gerer`. C'est là que
> l'exploitant active la taxe quand sa commune l'impose, et qu'il choisit entre
> `une_nuitee_par_occupation` (500 F pour trois nuits) et `au_prorata` (500 F × 3).

---

## 2. La disponibilité et l'attribution — le cœur

| # | `operation_id` | Verbe | Chemin | Permission | Classe |
|---|---|---|---|---|---|
| 9 | `hebergement_consulter_disponibilite` | `GET` | `/disponibilite` | `heb.disponibilite.consulter` | lecture |
| 10 | `hebergement_attribuer_unite` | `POST` | `/occupations` | `heb.unite.attribuer` | **B** |
| 11 | `hebergement_liberer_occupation` | `POST` | `/occupations/{occupation_id}/liberation` | `heb.unite.liberer` | **B** |

### 2.1 Consulter la disponibilité — une lecture, jamais une garantie

```
GET /disponibilite?categorie_id=…&debut=2026-08-03T14:00:00Z&fin=2026-08-05T12:00:00Z
```

```jsonc
// 200
{
  "unites_disponibles": [
    { "id": "…", "code": "B3", "etage": 1, "statut_menage": "propre" }
  ],
  "instant_autorite": "2026-08-02T10:20:31Z"   // horodatage SERVEUR
}
```

> **Cette réponse ne garantit rien, et le contrat doit le dire.** Entre la lecture et
> l'attribution, une autre transaction peut prendre l'unité. La garantie est **la contrainte
> d'exclusion**, jamais cette lecture (FR-013). Un client qui traiterait ce résultat comme une
> réservation reproduirait le verrou applicatif que le principe IV refuse.

### 2.2 Attribuer — l'opération que la contrainte protège

```jsonc
// POST /occupations
{
  "id": "0198f3aa-...",           // UUID v7 posé par le client (principe VI)
  "unite_id": "…",
  "formule_id": "…",
  "debut_client": "2026-08-03T14:00:00Z",
  "fin_client":   "2026-08-05T12:00:00Z"
}
```

Le serveur **calcule lui-même** la borne haute de `periode` en ajoutant le temps de remise en état
de la catégorie pour la famille de la formule. Le client ne l'envoie pas et ne peut pas l'influencer
— sans quoi il pourrait la mettre à zéro et supprimer le battement.

```jsonc
// 201
{
  "id": "0198f3aa-...",
  "unite_id": "…",
  "debut_client": "2026-08-03T14:00:00Z",
  "fin_client":   "2026-08-05T12:00:00Z",
  "indisponible_jusqu_a": "2026-08-05T14:00:00Z",   // fin_client + 2 h de ménage
  "statut": "active",
  "cree_le": "2026-08-02T10:20:31Z"                 // horodatage d'autorité
}
```

**`200` sur rejeu, pas `409`** — même UUID v7 soumis deux fois : le corps renvoyé est la ligne
telle qu'elle est en base. Un client hors ligne qui vide sa file ne doit pas voir d'erreur pour
une écriture que le serveur a déjà acceptée (principe VI).

### 2.3 Les refus, et pourquoi ils sont distincts

| Statut | `code` | Cause | Clé i18n de l'interface |
|---|---|---|---|
| `409` | `unite_deja_occupee` | **Violation de la contrainte d'exclusion** | « Cette chambre est déjà prise sur cette période » |
| `422` | `formule_hors_categorie` | La formule n'appartient pas à la catégorie de l'unité | « Cette formule ne s'applique pas à cette chambre » |
| `422` | `plage_non_fractionnable` | Demi-journée : l'intervalle ne coïncide pas avec une plage | « Une demi-journée se loue en entier : 8 h – 12 h ou 13 h – 16 h » |
| `422` | `intervalle_invalide` | Fin ≤ début | « La fin doit être après le début » |
| `422` | `duree_hors_contrainte` | Durée hors des bornes de la formule | « Cette formule se loue de 1 h à 8 h » |
| `403` | `permission_absente` | — | L'action est **absente** de l'interface, jamais grisée |
| `409` | `service_inactif` | Module hébergement non actif | Patron normalisé au cycle 002 |

**`unite_deja_occupee` doit venir de la contrainte, pas d'une vérification préalable.** Le service
tente l'insertion et traduit `ErrorKind::ExclusionViolation` ; il ne lit pas d'abord pour décider.
Une lecture préalable serait exactement le verrou applicatif que le principe IV refuse, et le test
de concurrence assertera la **cause** du refus, pas seulement son existence.

### 2.4 Libérer

```jsonc
// POST /occupations/{id}/liberation
{ "id": "0198f3bb-..." }   // UUID v7 de l'opération, pour l'idempotence
```

La libération **raccourcit `periode`** à `now()` + temps de remise en état, et pose
`statut = 'liberee'`, `libere_le = now()`. Ce n'est jamais un `DELETE` : une chambre occupée
reste une chambre occupée dans l'histoire.

---

## 3. La tarification — le moteur, sans la note

| # | `operation_id` | Verbe | Chemin | Permission |
|---|---|---|---|---|
| 12 | `hebergement_calculer_tarif` | `POST` | `/occupations/{occupation_id}/tarif` | `heb.disponibilite.consulter` |

```jsonc
// 200 — départ constaté à 4 h 10 sur un passage vendu 2 h
{
  "duree_reelle_minutes": 250,
  "formule_appliquee": "PASSAGE",
  "palier_retenu_minutes": 240,
  "heures_supplementaires": 1,
  "montant_du_mineur": 6200,        // 5000 + 1 × 1200
  "devise": "XOF",
  "rebascule": {
    "palier_vendu_minutes": 120,
    "montant_vendu_mineur": 2800,
    "difference_mineur": 3400
  },
  "instant_autorite": "2026-08-02T14:30:12Z"
}
```

**Le moteur calcule, il ne facture pas** (R-12). Aucune ligne de note n'est écrite — la note est
SEJ-03, tranche T2. Ce que cette opération produit, c'est une **décision de tarification** que
SEJ-03 consommera.

**La rebascule est tracée au registre des actions dans la même transaction**, via le trait d'audit
de `socle/comptes`. C'est ce que M. Koffi lira : « Durée dépassée : passé au tarif 4 h ».

**Toute durée vient de l'horodatage d'autorité serveur** (FR-029). L'appel ne prend aucun instant
en paramètre : le serveur lit `cree_le` de l'occupation et `now()`. Un client ne peut pas influencer
la durée facturée, même avec une horloge décalée.

**Bascule en nuitée** : si la durée atteint le seuil paramétré (8 h à Deloria),
`formule_appliquee` vaut `NUITEE` et `palier_retenu_minutes` est absent — ce n'est pas un palier
majoré, c'est un changement de formule.

---

## 4. Ce que le contrat ne sert pas, et pourquoi

| Absent | Motif |
|---|---|
| Modification de `statut_menage` | HEB-06, P1 — la colonne existe, l'endpoint non (principe X) |
| Mise hors service d'une unité | HEB-06 |
| Forçage de disponibilité | CPT-04 / HEB-06 |
| Suppression d'une occupation | Une occupation se libère, jamais ne s'efface (`DELETE` non accordé) |
| Changement de catégorie d'une unité | Effet tarifaire et fiscal, **non classé au registre**. Se spécifie, ne se glisse pas dans un `PUT` de correction |
| Modification du sous-statut de ménage | Classe A — HEB-06 |
| Mise hors service d'une unité | Classe B — HEB-06 |
| `?inclure_inactifs=true` | Ce que l'interface ne doit pas montrer, elle ne doit pas le recevoir — précédent ETB-02 |
| Écriture de `prestation_incluse` | Provision HEB-09 — table seule |
| Tout endpoint de séjour, client, note | SEJ et FIS |

---

## 5. Décomptes à mettre à jour dans le même changement

`backend/tests/couverture_portes.rs` porte des totaux qui se relisent du catalogue système et du
contrat, **jamais d'une constante recopiée**. Ce cycle les fait bouger :

| Porte | Ensemble | Avant | Après |
|---|---|---|---|
| P-05 | types d'événements outbox déclarés | 22 | **27** |
| P-07 | tables des schémas applicatifs | 26 | **34** |
| P-08 | opérations HTTP servies | 43 | **56** |
| P-01b | `operationId` distincts | 43 | **56** |

**Un décompte non mis à jour fait échouer le build** — c'est l'objet de ce fichier de test, né du
constat que trois nombres du plan du cycle 003 démentaient la réalité. Les chiffres ci-dessus sont
ceux que le plan **attend** ; ceux qui feront foi seront relus du catalogue et du contrat, et tout
écart sera justifié à l'endroit où il se constate, jamais résorbé en ajustant le tableau.
