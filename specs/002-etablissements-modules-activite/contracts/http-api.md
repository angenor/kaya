# Contrat HTTP — Cycle 002 · Établissements, modules d'activité et configuration héritée

**Phase 1 du plan** · 2026-07-31 · [plan.md](../plan.md) · [traits-exposes.md](traits-exposes.md)

> **Ce document décrit ce que le code doit produire ; il n'est pas la source de vérité.**
> Le contrat OpenAPI est **généré par utoipa depuis les annotations du code** et le client
> TypeScript depuis ce contrat (principe I·a, porte P-01). Un écart entre ce fichier et le code se
> tranche en faveur du code — et se corrige ici.

Vingt et une opérations, montées par `service(...)` exclusivement : `utoipa-actix-web` ne collecte
les chemins que de cette façon, et un point d'entrée monté par `route(...)` serait servi **sans
figurer au contrat** — donc absent du client généré et invisible pour la porte P-08.

---

## Conventions du cycle

**Le chemin et le verbe ne sont jamais répétés dans l'annotation.** Ils viennent de l'attribut de
routage d'Actix (feature `actix_extras`). Les écrire deux fois laisserait le contrat annoncer une
adresse que le serveur ne sert pas.

```rust
#[utoipa::path(tag = "etablissements", responses(...), security(("bearer" = [])))]
#[post("")]
pub async fn creer(...) -> Result<HttpResponse, actix_web::Error>
```

**`200` sur rejeu, jamais `409`.** Toute écriture porte un `id` UUID v7 fourni par le client ; trois
envois produisent un enregistrement, `201` puis `200`, `200`. Le corps rendu est la ligne **telle
qu'elle est en base** : le serveur fait foi en conflit (principe VI). Cela vaut aussi pour les
entités de classe C — un double-clic sur « Créer » ne doit pas créer deux établissements
([research.md R-11](../research.md)).

**Aucun détail interne ne franchit la frontière.** Ni message PostgreSQL, ni nom de table, ni
trace. Le détail part dans les journaux, corrélé par identifiant de requête.

### Corps d'erreur structuré — nouveauté de ce cycle

Le cycle 001 rendait des messages en clair. Ce cycle doit produire des refus **que l'interface
traduit** (porte P-16 : aucune chaîne utilisateur en dur), tout en **nommant la valeur refusée**
(FR-032, FR-033). D'où un corps stable :

```json
{
  "code": "capacite_non_implementee",
  "valeur": "LIVRAISON",
  "message": "capacité LIVRAISON déclarée au référentiel mais non implémentée au MVP"
}
```

- `code` — identifiant stable, jamais traduit, sur lequel le client branche sa clé i18n ;
- `valeur` — ce qui a été refusé, pour composer un message qui nomme la chose ;
- `message` — diagnostic pour les journaux et le développeur. **Jamais affiché tel quel.**

Codes d'erreur du cycle : `capacite_non_implementee`, `profil_non_implemente`,
`module_non_implemente`, `module_non_actif`, `cle_hors_catalogue`, `portee_interdite`,
`devise_figee`, `desactivation_bloquee`, `classement_incoherent`.

---

## 1 · Établissements — ETB-01

| # | Opération | Chemin | Codes |
|---|---|---|---|
| 1 | `GET` | `/api/v1/etablissements` | 200, 401 |
| 2 | `POST` | `/api/v1/etablissements` | **201**, **200** (rejeu), 400, 401, 403 |
| 3 | `GET` | `/api/v1/etablissements/{id}` | 200, 401, 403, 404 |
| 4 | `PATCH` | `/api/v1/etablissements/{id}` | 200, 400, 401, 403, 404, **422** |

**Corps de création** — `id` (UUID v7 client), `nom`, `juridiction`, `classement`, `etoiles?`,
`commune`, `fuseau_horaire`, `devise`, `adresse?`, `ncc?`.

**`422` sur modification**, deux cas nommés :

- `devise_figee` — la devise ne se modifie plus après la première opération financière. **Le
  contrôle est posé à vide à ce cycle** : la fonction qui compte les opérations rend zéro tant
  qu'aucune n'existe, et le cycle CAI la branche.
- `classement_incoherent` — un nombre d'étoiles sans classement par étoiles, ou l'inverse.

**Modification du fuseau horaire** : acceptée, mais la réponse porte un
`avertissement: "fuseau_change"` que l'interface **doit** présenter avant de confirmer. L'événement
`etablissement.fuseau_change` enregistre l'avertissement présenté — non pas qu'il ait été affiché,
mais qu'il faisait partie de l'opération.

---

## 2 · Référentiels — ETB-02, ETB-02b

| # | Opération | Chemin | Codes |
|---|---|---|---|
| 5 | `GET` | `/api/v1/referentiels/modules-activite` | 200, 401 |
| 6 | `GET` | `/api/v1/referentiels/capacites` | 200, 401 |
| 7 | `GET` | `/api/v1/referentiels/profils-stock` | 200, 401 |

**Lecture seule. Aucun verbe d'écriture n'est exposé** — l'enrichissement du référentiel relève de
l'éditeur (ETB-08, provision), et un point d'entrée d'écriture existant « pour plus tard » serait
une surface que rien ne garde.

Chaque élément porte `code`, `implementee`, `libelle_cle` et, pour les profils, `motif_refus_cle`.

**`implementee` est rendu, et c'est délibéré.** L'interface n'affiche jamais une valeur non
implémentée (FR-036) ; le drapeau existe pour que la **console éditeur** puisse un jour piloter le
référentiel, et pour que le client sache distinguer « valeur inconnue » de « valeur connue non
implémentée » dans un message d'erreur. Le filtrage est une règle d'affichage, testée sur la
fonction de sélection ([research.md R-13](../research.md)).

---

## 3 · Services d'un établissement — ETB-02

| # | Opération | Chemin | Codes |
|---|---|---|---|
| 8 | `GET` | `/api/v1/etablissements/{id}/services` | 200, 401, 403, 404 |
| 9 | `PUT` | `/api/v1/etablissements/{id}/services/{code}` | 200, **201**, 400, 401, 403, 404, **422** |

**`GET` ne rend que les services actifs.** Un paramètre `?inclure_inactifs=true` **n'existe pas** :
ce que l'interface ne doit pas montrer, l'interface ne doit pas le recevoir (principe VII, et
`RegistreModules` applique la même règle).

**`PUT` est idempotent et porte les deux sens** — corps `{ "id": "<uuid v7>", "actif": true|false }`.
`201` à la première activation, `200` ensuite. Le même point d'entrée active et désactive : deux
points d'entrée distincts laisseraient deux chemins pour un état, et un jour deux comportements.

**`422` :**

- `module_non_implemente` — le code existe au référentiel avec `implementee = false` ;
- `desactivation_bloquee` — un obstacle enregistré s'y oppose. Le corps porte la liste des
  obstacles, chacun avec son `motif_cle` et son `nombre`. **Aucun obstacle n'est enregistré à ce
  cycle** ; le chemin est exercé par un obstacle factice en test
  ([traits-exposes.md §6](traits-exposes.md)).

**La désactivation ne supprime rien** et la réactivation restitue l'état antérieur — y compris les
déclarations de capacité et les surcharges de configuration, qui deviennent inertes sans être
touchées.

---

## 4 · Capacités déclarées par un service — ETB-02b

| # | Opération | Chemin | Codes |
|---|---|---|---|
| 10 | `GET` | `/api/v1/etablissements/{id}/services/{code}/capacites` | 200, 401, 403, 404 |
| 11 | `POST` | `/api/v1/etablissements/{id}/services/{code}/capacites` | **201**, **200**, 400, 401, 403, 404, **422** |

**Corps** — `{ "id": "<uuid v7>", "capacite_code": "STOCK", "profil_code": "SIMPLE" }`.

**Les neuf refus de ce cycle**, tous en `422`, tous nommant la valeur, **aucune ligne écrite** :

| Valeur | `code` | Message |
|---|---|---|
| `LIVRAISON`, `PRODUCTION`, `COMMERCE_EN_LIGNE`, `FIDELITE`, `DEVIS`, `COMPTES_CLIENTS` | `capacite_non_implementee` | Nomme la capacité, indique qu'elle n'est pas implémentée au MVP |
| `VALORISE`, `DETAILLE` | `profil_non_implemente` | Nomme le profil |
| `AUCUN` | `profil_non_implemente` | **Message distinct** : une capacité non consommée **ne se déclare pas** — c'est le seul refus qui enseigne quelque chose plutôt que de constater une absence |

Le refus est tenu à trois couches ([research.md R-02](../research.md)) : clé étrangère composite et
`CHECK` en base, variante d'erreur au service, absence pure à l'interface. Le `422` est la forme
que prend la deuxième — il ne remplace ni la première ni la troisième.

---

## 5 · Points de vente — ETB-03

| # | Opération | Chemin | Codes |
|---|---|---|---|
| 12 | `GET` | `/api/v1/etablissements/{id}/points-de-vente` | 200, 401, 403, 404 |
| 13 | `POST` | `/api/v1/etablissements/{id}/points-de-vente` | **201**, **200**, 400, 401, 403, 404, **422** |
| 14 | `PATCH` | `/api/v1/points-de-vente/{id}` | 200, 400, 401, 403, 404 |
| 15 | `PUT` | `/api/v1/points-de-vente/{id}/tables` | 200, 400, 401, 403, 404 |

**Corps de création** — `{ id, module_code, nom, caisse_id? }`. `422` `module_non_actif` si le
service n'est pas activé sur l'établissement : la clé étrangère vers `etablissement_module` le rend
structurellement impossible, le `422` donne le message.

**Un point de vente sans tables est un comptoir** — la réponse porte `tables: []`, sans drapeau.
`PUT .../tables` remplace l'ensemble des tables : une liste vide fait du point de vente un
comptoir, ce qui est une transition légitime et non une suppression accidentelle.

**`caisse_id` n'est pas vérifié à ce cycle** — `socle/caisse` n'a pas de table
([research.md R-12](../research.md)). La vérification arrive au cycle CAI, par trait.

---

## 6 · Configuration héritée — ETB-04

| # | Opération | Chemin | Codes |
|---|---|---|---|
| 16 | `GET` | `/api/v1/configuration` | 200, 400, 401, 403 |
| 17 | `PUT` | `/api/v1/configuration` | 200, **201**, 400, 401, 403, 404, **422** |

**`GET`** — paramètres de requête `cle?`, `etablissement_id?`, `module_code?`,
`point_de_vente_id?`. Sans `cle`, rend **toutes** les valeurs applicables à la cible, en une
descente.

Chaque valeur rendue porte **son origine** :

```json
{ "cle": "politique_impression", "valeur": "…", "origine": "TENANT" }
```

**Une clé sans valeur à aucun niveau est absente de la réponse** — jamais rendue à `null`, jamais
accompagnée d'un défaut. `null` serait indistinguable d'une valeur nulle légitimement posée, et un
défaut serait un paramètre en dur (principe I·c).

**`PUT`** — corps `{ id, cle, valeur, portee, portee_id? }`. `422` :

- `cle_hors_catalogue` — la clé n'est pas au catalogue. La clé étrangère l'impose déjà en base ;
- `portee_interdite` — la portée demandée est plus basse que la `portee_la_plus_basse` déclarée au
  catalogue pour cette clé.

L'événement `parametre_configuration.ecrit` porte l'ancienne valeur quand il s'agit d'une surcharge
— sans quoi le grand livre dirait qu'une valeur a changé sans dire depuis quoi.

---

## 7 · Identité visuelle — ETB-05

| # | Opération | Chemin | Codes |
|---|---|---|---|
| 18 | `GET` | `/api/v1/branding` | 200, 401, 403 |
| 19 | `PUT` | `/api/v1/branding` | 200, 400, 401, 403, 404 |
| 20 | `POST` | `/api/v1/branding/logo` | 201, 400, 401, 403, 413 |
| 21 | `POST` | `/api/v1/branding/apercu` | 200, 400, 401, 403 |

**`GET`** — paramètre `etablissement_id?`. Sans lui, l'identité du tenant. Avec lui, **l'identité
résolue** : champ par champ, la première valeur non nulle en descendant du tenant vers
l'établissement. La réponse porte l'origine de chaque champ, comme la configuration.

**`POST /logo`** — corps multipart. Rend la clé d'objet, jamais une URL de stockage : l'accès passe
par une URL signée de courte durée, et l'objet vit dans le stockage S3, jamais en base. `413` si la
taille dépasse le plafond.

**Le plafond est une constante technique nommée, pas un paramètre d'établissement.** Un exploitant
n'a aucune raison de régler la taille maximale d'un logo, et l'inscrire au catalogue de paramètres
ferait entrer au récapitulatif du principe I·c une valeur qui ne relève pas de l'exploitation. Elle
est donc déclarée dans le code, avec sa justification, et **son dépassement produit un message qui
donne la limite** — jamais un refus muet.

**`POST /apercu`** — rend le document de test **sans rien enregistrer**. Corps : l'identité visuelle
à prévisualiser, telle qu'elle est à l'écran, y compris non enregistrée (FR-057).

**Le document rendu porte obligatoirement la mention « Document non fiscal — ne tient pas lieu de
facture »** (principe V, FR-058). Un test vérifie sa présence dans la sortie ; sans lui, le premier
aperçu ressemblant à une facture serait imprimé et présenté à un client.

---

## Isolation — porte P-08

**Les vingt et une opérations sont soumises au test d'isolation**, sans exception. Le tenant A ne
lit ni n'écrit aucune ligne du tenant B, y compris par identifiant direct.

Deux surfaces méritent une attention particulière, parce que leur forme invite à l'erreur :

- **`GET /api/v1/configuration`** descend quatre niveaux. L'isolation doit tenir **à chaque
  niveau**, pas seulement au premier : un `point_de_vente_id` appartenant à un autre tenant ne doit
  rien rendre, pas même la valeur héritée du tenant appelant.
- **Les trois référentiels** sont globaux et rendent les mêmes lignes à tout le monde. C'est
  correct et **doit être écrit dans le test**, sans quoi un futur relecteur les prendra pour une
  fuite. Le test vérifie l'identité des réponses entre deux tenants, ce qui transforme une
  exception en assertion.

Le cycle 001 a montré qu'une porte paramétrée sur le mauvais contrat passe au vert avec des points
d'entrée servis. P-08 consomme `application::contrat_complet()` — les chemins ne sont collectés
qu'au montage des routes.

---

## Génération du client

Après toute modification de handler :

```sh
scripts/ci/generer-client.sh    # puis commit du client
```

La porte P-01 fait échouer le build sur tout écart. Le client TypeScript n'est **jamais** édité à
la main.
