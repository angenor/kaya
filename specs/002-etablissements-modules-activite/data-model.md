# Modèle de données — Cycle 002 · Établissements, modules d'activité et configuration héritée

**Phase 1 du plan** · 2026-07-31 · [plan.md](plan.md) · [research.md](research.md)

Six migrations additives : **dix tables créées, une table enrichie** — soit **onze entités** au
registre des classes hors-ligne, dont **deux y sont absentes** et doivent y être ajoutées dans le
même changement, sous peine de faire échouer `backend/tests/classes_offline.rs`.

> **Ce décompte est opposable.** Dix est le nombre de tables dont la porte P-07 doit vérifier la
> sécurité au niveau ligne, onze celui des entités que le registre doit déclarer. Confondre les
> deux fait inspecter un sous-ensemble en croyant tout couvrir — le défaut exact que la
> constitution a documenté après le cycle 001.

Tout vit dans le schéma `etablissements`. **Aucune clé étrangère ne franchit une frontière de
module** (principe II) : le rattachement de caisse d'un point de vente est une colonne sans
référence, comme `auteur_compte_id` du module doré.

---

## Vue d'ensemble

```
tenant ─┬─ etablissement ─┬─ etablissement_module ─┬─ module_capacite
        │   (enrichi)     │   (activation)         └─ point_de_vente ── table_pdv
        │                 │
        └─ branding ──────┘   (tenant, surchargé par établissement)

parametre_configuration ── portée dérivée de trois clés étrangères nullables
                           (tenant → établissement → module → point de vente)

RÉFÉRENTIELS GLOBAUX, sans tenant_id, en lecture seule pour les tenants :
  module_activite · capacite · profil_stock · parametre_catalogue
```

| Migration | Tables | Story |
|---|---|---|
| `0007_etablissement_identite.sql` | `etablissement` **enrichie** (7 colonnes) | ETB-01 |
| `0008_referentiels_activite.sql` | `module_activite`, `capacite`, `profil_stock` | ETB-02, ETB-02b |
| `0009_activation_modules.sql` | `etablissement_module`, `module_capacite` | ETB-02, ETB-02b |
| `0010_points_de_vente.sql` | `point_de_vente`, `table_pdv` | ETB-03 |
| `0011_configuration_heritee.sql` | `parametre_catalogue`, `parametre_configuration` | ETB-04 |
| `0012_branding.sql` | `branding` | ETB-05 |

---

## 0007 — `etablissement` enrichie

**Migration additive** : `0002_etablissements_socle.sql` n'est pas touchée (porte P-02). Elle
l'annonçait déjà — « ETB-01 les enrichira par migration additive, jamais en modifiant ce fichier ».

| Colonne | Type | Contrainte | Note |
|---|---|---|---|
| `juridiction` | `TEXT` | `NOT NULL DEFAULT 'CI'` | Un seul adaptateur au MVP (§14.1). La valeur sélectionne le `JurisdictionAdapter`, elle n'encode aucune règle |
| `classement` | `TEXT` | `NOT NULL DEFAULT 'NON_CLASSE'`, `CHECK IN ('ETOILES','NON_CLASSE','RESIDENCE_MEUBLEE')` | Détermine le barème de la taxe communale de nuitée (§9.6) |
| `etoiles` | `SMALLINT` | `NULL`, `CHECK ((classement = 'ETOILES') = (etoiles IS NOT NULL))`, `CHECK (etoiles IS NULL OR etoiles > 0)` | Le nombre n'existe que pour le classement par étoiles, et l'égalité de conditions l'impose dans les deux sens. **Aucun plafond en base** : le nombre maximal d'étoiles est fixé par la réglementation nationale, donc par le `JurisdictionAdapter` (principe V, porte P-12) — même traitement que le NCC ci-dessous, et pour la même raison |
| `commune` | `TEXT` | `NOT NULL DEFAULT ''` puis `DROP DEFAULT` | Commune de rattachement — assiette du reversement communal |
| `adresse` | `TEXT` | `NULL` | Absente au provisionnement, renseignée ensuite |
| `ncc` | `TEXT` | `NULL`, `CHECK (ncc IS NULL OR length(btrim(ncc)) > 0)` | **Le contrôle de forme est volontairement minimal** — la validité d'un numéro de compte contribuable est une règle de juridiction, et le principe V la confine au `JurisdictionAdapter` (porte P-12). Un motif d'expression régulière ici serait une règle fiscale en base |

### Le piège qui décide de cette migration

**`ADD COLUMN ... NOT NULL DEFAULT`, jamais `ADD COLUMN` puis `UPDATE`.**

`etablissement` est en `FORCE ROW LEVEL SECURITY` et la migration s'exécute sous `kaya_owner`. Un
`UPDATE` de migration est soumis à la politique, `current_setting('app.current_tenant', true)` vaut
`NULL` hors requête applicative, **aucune ligne n'est touchée — et aucune erreur n'est levée**. La
migration réussit en n'écrivant rien.

`ADD COLUMN ... DEFAULT` est du DDL : il remplit les lignes existantes sans passer par une
politique. Voir [research.md R-08](research.md), dont la règle générale est reportée dans
`docs/module-dore.md`.

### Traçabilité des changements sensibles

Aucune table d'historique. Le **journal d'événements** du cycle 001 porte la trace, immuable et à
rétention illimitée. Deux modifications reçoivent leur propre type d'événement plutôt que d'être
noyées dans `etablissement.modifie` :

- `etablissement.classement_change` — change le barème de la taxe de nuitée ;
- `etablissement.fuseau_change` — réinterprète tout regroupement par journée locale.

---

## 0008 — Les quatre référentiels globaux

**Aucun `tenant_id`.** Exception nommée dans la migration, comme l'a été `tenant` au cycle 001.
Motif identique sur les quatre tables ([research.md R-01](research.md)) :

```sql
ALTER TABLE etablissements.<ref> ENABLE ROW LEVEL SECURITY;
ALTER TABLE etablissements.<ref> FORCE  ROW LEVEL SECURITY;
CREATE POLICY lecture_universelle  ON etablissements.<ref> FOR SELECT USING (true);
CREATE POLICY administration_editeur ON etablissements.<ref>
    FOR ALL TO kaya_owner USING (true) WITH CHECK (true);
GRANT SELECT ON etablissements.<ref> TO kaya_app;      -- et rien d'autre
```

**Ordre impératif dans la migration** : `CREATE TABLE` → `INSERT` des valeurs → `ENABLE`/`FORCE` →
`CREATE POLICY`. Insérer après l'activation échouerait, le propriétaire n'ayant pas encore de
politique d'écriture.

### `module_activite`

| Colonne | Type | Note |
|---|---|---|
| `code` | `TEXT PRIMARY KEY` | `HEBERGEMENT`, `RESTAURATION`, `BAR`, `PRESSING`, `SALLE_REUNION` |
| `implementee` | `BOOLEAN NOT NULL` | `true` pour les cinq. Support de la clé étrangère composite |
| `libelle_cle` | `TEXT NOT NULL` | **Clé i18n, jamais un libellé.** Une chaîne utilisateur en base échapperait à la porte P-16 et n'aurait pas d'anglais |
| `ordre` | `SMALLINT NOT NULL` | Ordre d'affichage stable, indépendant de l'alphabet et de la locale |
| `UNIQUE (code, implementee)` | | Cible de la clé étrangère composite de `etablissement_module` |

L'ajout de `SPA` ou `QUINCAILLERIE` (ETB-08) est un `INSERT` avec `implementee = false` : la valeur
existe au référentiel et **reste inactivable** tant que le drapeau n'est pas levé.

### `capacite`

Sept lignes. `STOCK` seule avec `implementee = true` ; `LIVRAISON`, `PRODUCTION`,
`COMMERCE_EN_LIGNE`, `FIDELITE`, `DEVIS`, `COMPTES_CLIENTS` avec `false`. Mêmes colonnes que
ci-dessus.

### `profil_stock`

Quatre lignes. `SIMPLE` seule avec `implementee = true` ; `AUCUN`, `VALORISE`, `DETAILLE` avec
`false`. Une colonne supplémentaire, `motif_refus_cle` (`TEXT NULL`) : clé i18n du message qui
explique le refus. Celui d'`AUCUN` est distinct — il dit qu'une capacité non consommée **ne se
déclare pas**, au lieu d'annoncer une fonctionnalité manquante.

### `parametre_catalogue`

| Colonne | Type | Note |
|---|---|---|
| `cle` | `TEXT PRIMARY KEY` | Ex. `politique_impression` |
| `type_valeur` | `TEXT NOT NULL` | `CHECK IN ('ENTIER','TEXTE','BOOLEEN','DUREE_MINUTES','MONTANT_MINEUR','HEURE_LOCALE','BAREME')`. **Liste fermée assumée** : contrairement aux capacités, un type n'est pas une fonctionnalité produit et son ajout mérite une migration |
| `portee_la_plus_basse` | `TEXT NOT NULL` | `CHECK IN ('TENANT','ETABLISSEMENT','MODULE','POINT_DE_VENTE')`. Jusqu'où la surcharge peut descendre. Le tenant est toujours autorisé comme racine |
| `story` | `TEXT NOT NULL` | `ETB-03`, `HEB-02`… Traçabilité vers le récapitulatif des paramètres |
| `libelle_cle`, `description_cle` | `TEXT NOT NULL` | Clés i18n |

**Contenu à ce cycle : une seule clé** — `politique_impression`, portée la plus basse
`POINT_DE_VENTE`, story `ETB-03`, sans jeu de valeurs (défini par le cycle IMP). Un catalogue à une
entrée se justifie parce que le résolveur doit exister **avant** le cycle qui le consommera en
premier ; le concevoir au cycle HEB le teinterait d'hébergement.

**Porte de cohérence documentaire.** `backend/tests/parametres_catalogue.rs` vérifie que **toute
clé du catalogue figure au « Récapitulatif des paramètres d'établissement »** de
`docs/user-stories-v1.md`. Comparaison **catalogue → récapitulatif**, asymétrique comme celle de
`classes_offline.rs` : une ligne du récapitulatif sans clé est normale (le paramètre relève d'un
cycle futur), une clé sans ligne est l'erreur.

---

## 0009 — Activation et déclaration de consommation

### `etablissement_module`

| Colonne | Type | Note |
|---|---|---|
| `id` | `UUID PRIMARY KEY` | **UUID v7 fourni par le client** (§11.5.1, [R-11](research.md)) |
| `tenant_id` | `UUID NOT NULL` | |
| `etablissement_id` | `UUID NOT NULL REFERENCES etablissement(id)` | Même schéma, clé étrangère légitime |
| `module_code` | `TEXT NOT NULL` | |
| `module_implemente` | `BOOLEAN NOT NULL` | Recopie du référentiel |
| `actif` | `BOOLEAN NOT NULL DEFAULT true` | La désactivation **ne supprime rien** |
| `active_le`, `desactive_le` | `TIMESTAMPTZ` | Horodatage d'autorité serveur |

```sql
FOREIGN KEY (module_code, module_implemente)
    REFERENCES etablissements.module_activite (code, implementee),
CHECK (module_implemente),
UNIQUE (etablissement_id, module_code)
```

Activer un module non implémenté est **structurellement impossible** : la seule ligne de
référentiel portant son code a `implementee = false`, et le `CHECK` refuse.

`UNIQUE (etablissement_id, module_code)` : un module s'active une fois par établissement. Une
réactivation est un `UPDATE actif = true`, jamais une seconde ligne — c'est ce qui fait que l'état
antérieur est restitué (FR-015).

**Privilèges** : `GRANT SELECT, INSERT, UPDATE` à `kaya_app`. **Pas de `DELETE`** — le privilège
dit la règle mieux qu'un commentaire.

### `module_capacite`

| Colonne | Type | Note |
|---|---|---|
| `id` | `UUID PRIMARY KEY` | UUID v7 client |
| `tenant_id` | `UUID NOT NULL` | |
| `etablissement_module_id` | `UUID NOT NULL REFERENCES etablissement_module(id)` | **La déclaration appartient au service, pas à l'établissement** — c'est le module qui déclare ce qu'il consomme |
| `capacite_code`, `capacite_implementee` | `TEXT`, `BOOLEAN` | Clé étrangère composite + `CHECK (capacite_implementee)` |
| `profil_code`, `profil_implemente` | `TEXT`, `BOOLEAN` | Idem vers `profil_stock` |
| `UNIQUE (etablissement_module_id, capacite_code)` | | |

`profil_code` est **`NOT NULL` au MVP** : seule `STOCK` est déclarable, et elle exige un profil. Le
jour où une capacité sans profil sera implémentée, une migration additive rendra la colonne
nullable avec la règle correspondante. Poser aujourd'hui un `CHECK ((capacite_code = 'STOCK') = ...)`
réintroduirait en base la valeur en dur que [R-02](research.md) écarte.

**La désactivation d'un service rend ses déclarations inertes sans les toucher** (FR-037) : elles
sont lues à travers `etablissement_module.actif`. Aucune colonne d'état sur cette table, aucune
écriture à la désactivation, donc aucune perte à la réactivation.

---

## 0010 — Points de vente

### `point_de_vente`

| Colonne | Type | Note |
|---|---|---|
| `id` | `UUID PRIMARY KEY` | UUID v7 client |
| `tenant_id` | `UUID NOT NULL` | |
| `etablissement_id` | `UUID NOT NULL REFERENCES etablissement(id)` | Dénormalisé pour l'isolation et la résolution |
| `etablissement_module_id` | `UUID NOT NULL REFERENCES etablissement_module(id)` | **C'est cette clé étrangère qui tient FR-041** : un point de vente ne peut pas se rattacher à un service non activé, puisque la seule cible possible est une activation existante |
| `nom` | `TEXT NOT NULL`, `CHECK (length(btrim(nom)) > 0)` | |
| `caisse_id` | `UUID NULL` | **Aucune clé étrangère** : `socle/caisse` est un autre module (principe II, [R-12](research.md)). Le cycle CAI ajoutera la vérification par trait |
| `actif` | `BOOLEAN NOT NULL DEFAULT true` | |
| `UNIQUE (etablissement_id, nom)` | | Deux points de vente homonymes seraient indiscernables sur un ticket |

**La politique d'impression n'est pas une colonne** — c'est un paramètre de la chaîne d'héritage
au niveau point de vente ([R-04](research.md)), conformément au principe I·c.

### `table_pdv`

| Colonne | Type | Note |
|---|---|---|
| `id` | `UUID PRIMARY KEY` | UUID v7 client |
| `tenant_id` | `UUID NOT NULL` | |
| `point_de_vente_id` | `UUID NOT NULL REFERENCES point_de_vente(id)` | |
| `libelle` | `TEXT NOT NULL` | « 12 », « Terrasse 3 » |
| `actif` | `BOOLEAN NOT NULL DEFAULT true` | |
| `UNIQUE (point_de_vente_id, libelle)` | | |

**Un comptoir est un point de vente sans aucune ligne ici.** Pas de drapeau `est_comptoir` : un
drapeau pourrait contredire les données, une absence non. C'est la forme normale, pas un cas
dégradé (FR-040).

---

## 0011 — Configuration héritée

### `parametre_configuration`

| Colonne | Type | Note |
|---|---|---|
| `id` | `UUID PRIMARY KEY` | UUID v7 client |
| `tenant_id` | `UUID NOT NULL` | |
| `etablissement_id` | `UUID NULL REFERENCES etablissement(id)` | |
| `etablissement_module_id` | `UUID NULL REFERENCES etablissement_module(id)` | |
| `point_de_vente_id` | `UUID NULL REFERENCES point_de_vente(id)` | |
| `cle` | `TEXT NOT NULL REFERENCES parametre_catalogue(cle)` | Une clé hors catalogue est refusée par la base |
| `valeur` | `JSONB NOT NULL` | |
| `modifie_le` | `TIMESTAMPTZ NOT NULL DEFAULT now()` | |

```sql
CHECK (num_nonnulls(etablissement_id, etablissement_module_id, point_de_vente_id) <= 1),
UNIQUE NULLS NOT DISTINCT (tenant_id, etablissement_id, etablissement_module_id,
                           point_de_vente_id, cle)
```

**Deux décisions non évidentes.**

*La portée est **dérivée**, jamais déclarée.* Trois clés étrangères nullables dont au plus une est
renseignée ; zéro renseignée signifie « niveau tenant ». Une colonne `portee` accompagnée d'un
`portee_id` polymorphe serait plus courte à écrire et **ne permettrait aucune intégrité
référentielle** : rien n'empêcherait `portee = 'POINT_DE_VENTE'` avec l'identifiant d'un
établissement. Ici, la portée ne peut pas mentir.

*`NULLS NOT DISTINCT` n'est pas un détail.* Sans lui, `UNIQUE` traite chaque `NULL` comme distinct
et **deux surcharges de niveau tenant portant la même clé passeraient toutes les deux**. La
résolution en choisirait une au hasard. PostgreSQL 18.4 le prend en charge ; l'index unique partiel
qui servait de contournement historique est inutile ici.

### Requête de résolution

Une seule descente, du plus spécifique au plus général :

```sql
SELECT valeur, ...
FROM etablissements.parametre_configuration pc
JOIN etablissements.etablissement_module em ON ...      -- filtre em.actif
WHERE pc.cle = $1 AND (
      pc.point_de_vente_id = $2
   OR pc.etablissement_module_id = $3
   OR pc.etablissement_id = $4
   OR num_nonnulls(pc.etablissement_id, pc.etablissement_module_id, pc.point_de_vente_id) = 0)
ORDER BY <rang de portée décroissant>
LIMIT 1
```

Le rang de portée est calculé en SQL depuis les colonnes renseignées — pas lu dans une colonne, qui
pourrait diverger. Les niveaux absents de la cible ne produisent aucune branche : une chaîne
écourtée fonctionne sans niveau inventé (FR-050).

**Une surcharge portée par un service désactivé est ignorée sans être supprimée** (FR-051) : le
filtre porte sur `etablissement_module.actif`, dans le **même schéma** — la porte P-04 n'est pas
concernée.

**Index** : `(tenant_id, cle)` et `(point_de_vente_id)`, `(etablissement_module_id)`,
`(etablissement_id)` partiels sur non-`NULL`.

---

## 0012 — Identité visuelle

### `branding`

| Colonne | Type | Note |
|---|---|---|
| `id` | `UUID PRIMARY KEY` | UUID v7 client |
| `tenant_id` | `UUID NOT NULL` | |
| `etablissement_id` | `UUID NULL REFERENCES etablissement(id)` | `NULL` = niveau tenant |
| `logo_objet_cle` | `TEXT NULL` | Clé d'objet dans le stockage S3. **Jamais le binaire en base** |
| `couleur_primaire` | `TEXT NULL`, `CHECK (~ '^#[0-9A-Fa-f]{6}$')` | |
| `entete_document`, `pied_document`, `mentions_legales`, `coordonnees` | `TEXT NULL` | |
| `UNIQUE NULLS NOT DISTINCT (tenant_id, etablissement_id)` | | Une ligne par niveau, au plus |

**Toutes les colonnes de contenu sont nullables, et c'est le mécanisme de surcharge partielle**
(FR-056) : la résolution prend, champ par champ, la première valeur non nulle en descendant du
tenant vers l'établissement. Surcharger le seul logo laisse hériter tout le reste, sans qu'aucune
logique de fusion n'ait à être écrite.

**`couleur_primaire` ne touche jamais l'interface** (FR-059). Elle s'applique aux documents
produits. La porte P-17 interdit toute couleur littérale hors des jetons ; cette valeur est une
**donnée client**, pas un style d'application, et la distinction doit être écrite dans le composant
qui la consomme — sans quoi le premier développeur pressé l'appliquera à un bouton.

---

## Classes hors-ligne — toutes les entités du cycle

| Entité | Écriture | Lecture en cache | Registre |
|---|---|---|---|
| `etablissement` (colonnes ajoutées) | **C** | **A** — fraîcheur affichée | §5.1, déjà déclaré |
| `module_activite` | **C** | **A** | §5.1, déjà déclaré |
| `capacite` | **C** | **A** | §5.1, déjà déclaré |
| **`profil_stock`** | **C** | **A** | **§5.1, à AJOUTER** |
| `etablissement_module` | **C** | **A** | §5.1, déjà déclaré |
| `module_capacite` | **C** | **A** | §5.1, déjà déclaré |
| `point_de_vente` | **C** | **A** | §5.1, déjà déclaré |
| `table_pdv` | **C** | **A** | §5.1, déjà déclaré |
| **`parametre_catalogue`** | **C** | **A** | **§5.1, à AJOUTER** |
| `parametre_configuration` | **C** | **A** | §5.1, déjà déclaré |
| `branding` | **C** | **A** | §5.1, déjà déclaré |

**Deux tables sont absentes du registre.** `backend/tests/classes_offline.rs` compare les tables
réelles aux entités déclarées et **fait échouer le build** sur toute table absente. `profil_stock`
et `parametre_catalogue` doivent être ajoutées au §5.1 **dans le même changement que leur
migration**, avec une entrée au journal §13.

### La distinction écriture / lecture, à écrire au registre

Le registre classe des **opérations**, et il en manque une pour tout le référentiel : **la lecture
en cache**. L'écriture d'un référentiel est de classe C — jamais hors ligne. Sa **lecture** doit
rester possible hors connexion, avec fraîcheur affichée, sinon le produit devient inutilisable dès
la première coupure : un serveur qui ne peut pas lire la liste des services de son établissement ne
peut rien faire.

C'est la même dualité que `encaissement`, B en espèces et D en Mobile Money. Une ligne est ajoutée
au §5.1 pour la nommer explicitement, faute de quoi un cycle ultérieur tranchera dans un sens ou
dans l'autre sans que la décision soit visible.

---

## Événements du journal

Un événement par transition, **dans la transaction** (porte P-05). `version_schema = 1` partout.
Charge utile **complète et dénormalisée** : un lecteur qui n'a que l'événement doit pouvoir dire ce
qui s'est passé (règle R-11 du cycle 001).

| Type | Agrégat | Charge utile |
|---|---|---|
| `etablissement.cree` | `etablissement` | Identité complète : juridiction, classement, étoiles, commune, fuseau, devise, adresse, NCC |
| `etablissement.modifie` | `etablissement` | Valeurs **avant et après** des champs touchés |
| `etablissement.classement_change` | `etablissement` | Ancien et nouveau classement, étoiles — le barème de nuitée en dépend |
| `etablissement.fuseau_change` | `etablissement` | Ancien et nouveau fuseau, et l'avertissement présenté à l'opérateur |
| `etablissement_module.active` | `etablissement_module` | Établissement, code du module, horodatage d'autorité |
| `etablissement_module.desactive` | `etablissement_module` | Idem |
| `module_capacite.declaree` | `module_capacite` | Service, capacité, profil |
| `point_de_vente.cree` / `.modifie` | `point_de_vente` | Établissement, service, nom, présence de tables, caisse rattachée |
| `table_pdv.creee` / `.desactivee` | `table_pdv` | Point de vente, libellé |
| `parametre_configuration.ecrit` | `parametre_configuration` | Clé, valeur, **niveau de portée**, ancienne valeur si surcharge |
| `branding.modifie` | `branding` | Niveau, champs touchés — pas le binaire du logo, sa clé d'objet |

**Aucun événement sur rejeu.** Un `ON CONFLICT DO NOTHING` qui ne crée rien n'émet rien : le grand
livre enregistre les transitions d'état, pas les tentatives réseau. C'est le point 5 de l'ordre des
opérations du module doré, et celui qu'on écrit mal.

---

## Jeux de données de démonstration

Ajoutés à la mécanique de TRX-05a, identifiants fixes, `ON CONFLICT DO NOTHING`. Exécution sous le
rôle applicatif **avec pose du tenant courant** — jamais sous le propriétaire ([R-08](research.md)).

**Deloria, établissement d'Abengourou** — classement `NON_CLASSE`, commune `Abengourou`, fuseau
`Africa/Abidjan`, devise `XOF`, juridiction `CI`. Cinq services actifs : `HEBERGEMENT`,
`RESTAURATION`, `BAR`, `PRESSING`, `SALLE_REUNION`. Capacité `STOCK` au profil `SIMPLE` déclarée
par **`RESTAURATION` et `BAR`** — les deux services qui vendent des articles stockés (hypothèse 9
de la spécification, révisable sans coût avant le cycle STK).

**Résidence Test** — service `HEBERGEMENT` seul, **aucune capacité**, aucun point de vente. Ses
quatre unités relèvent du cycle HEB et de la tâche de recollement TRX-05b.

**Ni l'un ni l'autre ne contient `MODULE_FICTIF_TEST`** — vérifié par un test dédié (FR-027).

---

## Ce que ce modèle ne contient pas, volontairement

| Absent | Raison |
|---|---|
| `partenaire`, `demande_partenaire`, `compte_compensation` | ETB-07, provision. **Non créées par ce cycle** (FR-079) |
| Valeurs de module ou de capacité au-delà des cinq et des sept | ETB-08, provision. Le référentiel est en table : l'ajout est une écriture, pas une migration |
| Table d'historique des modifications | Le journal d'événements la remplace : immuable, rétention illimitée, déjà en place |
| Drapeau `est_comptoir` sur `point_de_vente` | Un drapeau peut contredire les données ; l'absence de tables, non |
| Colonne `portee` sur `parametre_configuration` | Dérivée des clés étrangères, elle ne peut pas mentir |
| Clé étrangère de `point_de_vente.caisse_id` | Frontière de module (principe II) |
| Rattachement compte ↔ établissement | Cycle CPT. Ce cycle vérifie seulement qu'aucune contrainte ne l'interdit (FR-009) |
