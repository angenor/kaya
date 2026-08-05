# Contrat HTTP — cycle 006

**Dix-sept opérations.** Le contrat OpenAPI est la source de vérité (principe I·a, TRX-01) : les
handlers portent `#[utoipa::path]`, le client TypeScript est régénéré et commité, et **P-01 échoue
sur tout diff**. Le total du produit passe de **56 à 73** opérations.

**Montage** : `service(...)`, jamais `route(...)` — `utoipa-actix-web` ne collecte que depuis
`service`. **Du plus spécifique au plus général**, sans quoi un préfixe court capture les chemins
longs et rend `404` **sans erreur de compilation et avec un contrat parfaitement exact** (piège
constaté au cycle 003).

**Codes de refus** : chaque opération d'écriture rend un `CorpsErreur` portant un **code stable**
que l'interface traduit par le lexique — jamais un message de diagnostic (règle du cycle 002).

---

## Fiches clients — opérations 1 à 6

Chemin de base : `/api/v1/clients`. **Aucun `etablissement_id`** : la fiche est du **tenant**
(FR-002).

| # | Méthode | Chemin | `operation_id` | Permission | Classe |
|---|---|---|---|---|---|
| 1 | `GET` | `/api/v1/clients` | `client_rechercher` | `sej.client.lire` | lecture |
| 2 | `POST` | `/api/v1/clients` | `client_creer` | `sej.client.gerer` | **C** |
| 3 | `GET` | `/api/v1/clients/{client_id}` | `client_lire` | `sej.client.lire` | lecture |
| 4 | `PATCH` | `/api/v1/clients/{client_id}` | `client_modifier` | `sej.client.gerer` | **C** |
| 5 | `GET` | `/api/v1/clients/{client_id}/sejours` | `client_historique_sejours` | `sej.client.lire` + `heb.sejour.lire` | lecture |
| 6 | `POST` | `/api/v1/clients/{client_id}/preferences` | `client_preference_enregistrer` | `sej.client.gerer` | **A** |

### 1 · `GET /api/v1/clients` — la recherche

```
?recherche=<texte>&limite=<n>
```

**Une seule entrée pour trois formes.** Le serveur décide de la forme d'après la saisie : que des
chiffres → téléphone ; alphanumérique sans espace de plus de cinq caractères → numéro de pièce ;
sinon → nom. Une saisie ambiguë interroge **les trois** et fusionne — c'est le comportement attendu
au comptoir, où l'opérateur ne choisit pas un mode.

| Réponse | Contenu |
|---|---|
| `200` | `{ clients: ClientResume[], tronque: bool }` |

`tronque` dit qu'il y avait plus de résultats que `limite` — une liste silencieusement coupée est
un mensonge sur un écran de comptoir.

**La recherche ne renvoie que des personnes qualifiées clientes** — jointure `personne × client`,
même schéma. Le personnel n'y apparaît jamais.

**Cible : 300 ms au 95ᵉ centile sur 10 000 fiches** (FR-006, SC-005), mesurée côté serveur.

### 2 · `POST /api/v1/clients` — créer une fiche

Corps : `{ id, nom, prenoms?, date_naissance?, nationalite?, telephone?, email?, type_piece?,
numero_piece? }`. `id` est un **UUID v7 généré par le client** — c'est lui qui rend le rejeu
inoffensif.

| Réponse | Cas |
|---|---|
| `201` | Créée |
| `200` | **Rejeu** — le même `id` renvoie la ligne telle qu'elle est en base. Un terminal qui vide sa file ne doit pas voir d'erreur pour une écriture déjà acceptée |
| `403` | Permission absente |
| `422` | `nom_vide`, `telephone_invalide`, `nationalite_invalide` |

**Classe C** : refusée **immédiatement et explicitement** hors ligne, jamais mise en file (P-13).
Décision O-01, option (a) — un client jamais vu exige le réseau (FR-011).

### 6 · `POST /api/v1/clients/{client_id}/preferences` — classe A

Corps : `{ id, texte, horodatage_client? }`. **Append-only** : la préférence courante est la
dernière ligne. Rejeu triple → un enregistrement **et aucun second événement outbox**.

`horodatage_client` est accepté, **indicatif**, et ne porte **aucune règle** (P-23).

---

## Séjours — opérations 7 à 16

Chemin de base : `/api/v1/etablissements/{etablissement_id}/sejours`.

| # | Méthode | Chemin | `operation_id` | Permission | Classe |
|---|---|---|---|---|---|
| 7 | `POST` | `.../sejours` | `sejour_ouvrir` | `heb.sejour.ouvrir` | **B** |
| 8 | `GET` | `.../sejours` | `sejour_lister` | `heb.sejour.lire` | lecture |
| 9 | `GET` | `.../sejours/{sejour_id}` | `sejour_lire` | `heb.sejour.lire` | lecture |
| 10 | `POST` | `.../sejours/{sejour_id}/client` | `sejour_rattacher_client` | `heb.sejour.ouvrir` | **B** |
| 11 | `POST` | `.../sejours/{sejour_id}/accompagnants` | `sejour_accompagnant_ajouter` | `heb.sejour.ouvrir` | **A** |
| 12 | `DELETE` | `.../sejours/{sejour_id}/accompagnants/{accompagnant_id}` | `sejour_accompagnant_retirer` | `heb.sejour.ouvrir` | **A** |
| 13 | `POST` | `.../sejours/{sejour_id}/prolongation` | `sejour_prolonger` | `heb.sejour.prolonger` | **B** |
| 14 | `POST` | `.../sejours/{sejour_id}/changement-unite` | `sejour_changer_unite` | `heb.sejour.changer_unite` | **B** |
| 15 | `POST` | `.../sejours/{sejour_id}/depart` | `sejour_clore` | `heb.sejour.clore` | **B** |
| 16 | `GET` | `.../sejours/{sejour_id}/fiche-police` | `sejour_fiche_police_lire` | `heb.sejour.lire` | lecture |

### ★ 7 · `POST .../sejours` — l'opération du cycle

**Un appel, une transaction, cinq écritures.** C'est ce qui tient le budget de FR-031 : au plus un
appel réseau bloquant entre le premier geste et la confirmation.

```
Corps : {
  id,                       // UUID v7 généré par le client
  unite_id,                 // choisie par l'opérateur (un tap sur R4)
  formule_id,
  debut_client, fin_client, // RFC 3339 — pour un passage, calculés depuis la durée touchée
  client_id?,               // ABSENT pour un passage : la pièce vient après la clé (R4)
  accompagnants?: [{ id, nom, prenoms? }]
}
```

**Ce que le serveur fait, dans l'ordre, dans une seule transaction :**

1. `MoteurDisponibilite::attribuer(&mut tx, demande)` — l'attribution, **par la contrainte
   d'exclusion**, jamais par une lecture préalable ;
2. `INSERT hebergement.sejour`, avec `sejour_id` posé sur l'occupation ;
3. `INSERT hebergement.note_sejour` + sa **ligne d'hébergement** au tarif du moteur de tarification ;
4. `UPDATE … RETURNING` sur le compteur de numérotation, puis `INSERT hebergement.fiche_police` —
   `complete = false` si aucun client n'est rattaché ;
5. `OutboxWriter::ecrire(&mut tx, heb.sejour.ouvert)`.

| Réponse | Cas |
|---|---|
| `201` | Séjour ouvert — corps : le séjour, l'occupation, la note et son total, la fiche de police |
| `200` | **Rejeu** du même `id` |
| `403` | Permission absente |
| `404` | Établissement, unité ou formule inconnus |
| `409` | `unite_deja_occupee` — **le refus vient de la contrainte**, jamais d'une vérification |
| `409` | `service_inactif` — module hébergement non actif sur l'établissement |
| `422` | `intervalle_invalide`, `duree_hors_contrainte`, `formule_hors_categorie`, `plage_non_fractionnable` |

> **Le `409` de rejeu et le `409` de conflit ne se confondent pas.** Même `id` → `200` avec la ligne
> en base. `id` différent sur un intervalle chevauchant → `409 unite_deja_occupee`. C'est la
> distinction posée au cycle 004, reprise telle quelle.

### 11 · `POST .../accompagnants` — et le cas orphelin

Corps : `{ id, nom, prenoms?, date_naissance?, nationalite?, type_piece?, numero_piece?,
horodatage_client? }`.

| Réponse | Cas |
|---|---|
| `201` | Ajouté à un séjour **ouvert** |
| `200` | Rejeu |
| **`202`** | ★ **Le séjour est CLOS.** L'écriture part en **file de réconciliation** — corps : `{ motif: "sejour_clos", reconciliation_id }`. **Ni `201`** (ajout d'office), **ni `409`** (rejet silencieux) : le principe VI interdit les deux |

**Classe A** : la seule écriture de séjour atteignable hors ligne, avec l'opération 12. Elle est
donc **nommée** dans `sejour_hors_ligne.rs`, jamais omise — un test qui n'inspecterait que les
opérations refusées ne prouverait pas qu'il les a toutes vues.

### ★ 15 · `POST .../depart` — le départ

Corps : `{ id }` — rien d'autre. **L'instant du départ est celui du serveur** (P-23) ; le laisser
fournir par le client permettrait d'antidater une nuit.

**Ce que le serveur fait, dans une seule transaction :**

1. calcule la durée réelle depuis `now()` et l'instant d'autorité d'ouverture ;
2. demande la décision au `MoteurTarification` du cycle 004 — **rebascule de palier comprise** ;
3. si la durée réelle diffère du prévu, `INSERT` d'une **ligne d'ajustement** portant son motif
   (`rebascule_palier` ou `depart_anticipe`) — **jamais un `UPDATE` de la ligne initiale**, ce que
   le privilège rend d'ailleurs impossible ;
4. `INSERT hebergement.taxe_sejour_constat` — les faits **et** le paramétrage recopié ;
5. arrête la note, clôt le séjour, libère l'occupation à l'instant réel ;
6. écrit au registre des actions et à l'outbox.

| Réponse | Cas |
|---|---|
| `200` | Séjour clos — corps : la note complète, les ajustements, le constat figé |
| `409` | `sejour_deja_clos` |
| `403` | Permission absente |

**Le corps de réponse ne porte aucun montant de taxe** : `nuitees_assujetties` et `montant_mineur`
du constat sont `null`, et c'est **visible dans le contrat**. Le rendre à zéro laisserait croire que
la taxe est nulle ; le rendre absent laisserait croire qu'elle n'existe pas. `null` dit ce qui est
vrai : **le montant n'est pas encore déterminé, il viendra de FIS**.

### 13 · `POST .../prolongation` — le conflit nommé

Corps : `{ id, nouvelle_fin_client }`.

| Réponse | Cas |
|---|---|
| `200` | Prolongé — corps : la période étendue et les lignes ajoutées |
| `409` | ★ `conflit_occupation_suivante` — corps : `{ unite_id, debut_occupation_suivante, unites_alternatives: UniteDisponible[] }`. **Le conflit est nommé**, pas générique (FR-070), et les alternatives de la même catégorie sont proposées (FR-071) |
| `409` | `sejour_clos` |
| `422` | `bascule_formule_non_confirmee` — le franchissement de `seuil_bascule_nuitee_minutes` **doit être confirmé avant** (FR-073). Le corps porte le montant résultant, et la requête se rejoue avec `bascule_acceptee: true` |

### 14 · `POST .../changement-unite`

Corps : `{ id, unite_cible_id }`. Clôt l'occupation courante à `now()`, en ouvre une sur l'unité
cible **sur le même séjour**. `409 unite_cible_occupee` avec le conflit nommé — **aucun déplacement
partiel n'est jamais produit** : les deux occupations vivent dans la même transaction.

---

## État des unités — opération 17

| # | Méthode | Chemin | `operation_id` | Permission |
|---|---|---|---|---|
| 17 | `GET` | `/api/v1/etablissements/{id}/hebergement/etat-des-unites` | `hebergement_etat_des_unites` | `heb.disponibilite.consulter` |

Rend **toutes** les unités de l'établissement avec :

- leur **état d'occupation dérivé** — `libre` · `occupee` avec `fin_prevue` · `remise_en_etat`
  avec `disponible_a` ;
- leur `statut_menage`, **en lecture seule** ;
- `instant_autorite`, qui dit **quand** la réponse était vraie.

**Ce n'est pas HEB-06** (hors périmètre) : le sous-statut ménage n'est pas modifiable ici, et
l'état d'occupation est **dérivé des occupations**, jamais posé à la main (principe IV).

**Cet appel se fait au montage de l'écran `R4`**, avant le premier geste : il ne compte pas dans le
budget d'un appel bloquant, qui court du premier geste à la confirmation.

---

## Ce que le contrat n'expose PAS, et pourquoi c'est écrit

| Absence | Motif |
|---|---|
| Toute écriture sur `nuitees_assujetties` ou `montant_mineur` | **Règle fiscale** → `JurisdictionAdapter`, FIS-03, tranche T3. `provisions_sans_logique.rs` vérifie qu'aucun chemin ne les touche |
| Consommations de points de vente sur la note | **SEJ-03**, tranche T2 |
| Transfert de charges, remise | **SEJ-03**, tranche T2 |
| Vente à un client sans hébergement | **SEJ-05**, tranche T2 |
| Capture de pièce par caméra | **SEJ-06**, P1, tranche T4 |
| Encaissement, document fiscal, certification | **CAI** (T2) et **FIS** (T3) |
| Fusion de deux fiches en doublon | Aucune story ne l'appelle |
| Suppression d'un séjour, d'une note, d'une ligne, d'un constat | **Aucune table n'a `DELETE`** — le privilège le dit avant le contrat |

---

## Décomptes que `couverture_portes.rs` doit porter

| Porte | Avant | Après | Ventilation à déclarer |
|---|---|---|---|
| **P-01b / P-08** | 56 | **73** | `("cycle 006 — fiches clients (SEJ-01)", 6)` · `("cycle 006 — séjours (SEJ-02/04)", 10)` · `("cycle 006 — état des unités", 1)` |
| **P-05** | 27 | **36** | Les neuf types du cycle, chacun avec son test dans `outbox_transactionnel.rs` |
| **P-07** | 29 | **38** | Les neuf tables nouvelles |
| `PLANCHER_TABLES` (`classes_offline.rs`) | 35 | **44** | Relevé du même écart |

> **Le décompte est ventilé par lot, jamais posé en un seul nombre.** Un total unique se corrige en
> changeant un chiffre ; une ventilation oblige à dire de quel lot vient l'écart, et c'est cette
> phrase-là qu'on ne peut pas écrire sans s'en apercevoir.
