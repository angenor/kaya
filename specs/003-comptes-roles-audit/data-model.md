# Phase 1 — Modèle de données : comptes, rôles cumulables et journal d'audit

**Cycle 003 (CPT)** · schéma `comptes` · **dix tables**, **sept migrations** (`0014` à `0020`)

*Toutes les décisions de forme viennent de `docs/module-dore.md` couche 1. Ce document dit ce que
ce cycle ajoute et **ce qui s'en écarte, avec la raison**.*

---

## Vue d'ensemble

| # | Table | Rôle | Classe | Story |
|---|---|---|---|---|
| 1 | `comptes.personne` | Identité civile | **C** | CPT-00 |
| 2 | `comptes.compte` | Identité d'authentification | **C** | CPT-01 |
| 3 | `comptes.employe` | Contrat de travail — **PROVISION, vide** | **C** | CPT-00 |
| 4 | `comptes.methode_authentification` | Référentiel global — `MOT_DE_PASSE`, `OTP_SMS` | **C** | CPT-01 |
| 5 | `comptes.role` | Référentiel global — les huit rôles | **C** | CPT-02 |
| 6 | `comptes.permission` | Référentiel global — permissions granulaires | **C** | CPT-02 |
| 7 | `comptes.role_permission` | Référentiel global — ce que chaque rôle ouvre | **C** | CPT-02 |
| 8 | `comptes.compte_role` | **Le cumul** — N lignes par compte | **C** | CPT-02 |
| 9 | `comptes.journal_audit` | Trace immuable | **A** | CPT-04 |
| 10 | `comptes.appareil_enrole` | **PROVISION** — colonnes seulement | **C** | CPT-05/06 |

**Aucune session en base.** Elles vivent en Redis (research R-01), sont éphémères et
reconstructibles, et ne figurent donc ni ici, ni au registre des classes, ni dans les sauvegardes.
**Trois clés** : la session (90 jours), la marque de révocation consultée **à chaque requête
authentifiée** (60 min), et la famille de jetons de rafraîchissement pour la détection de
réutilisation (90 jours).

---

## Les sept migrations

| Migration | Contenu | Pourquoi séparée |
|---|---|---|
| `0014_schema_comptes.sql` | `CREATE SCHEMA comptes` + `GRANT USAGE` | `0001` est appliquée : la modifier ferait échouer **P-02** (research R-11) |
| `0015_personne_compte.sql` | `personne`, `methode_authentification`, `compte` | Le cœur de CPT-00 et CPT-01 |
| `0016_roles_permissions.sql` | `role`, `permission`, `role_permission`, `compte_role` | Trois référentiels globaux + la table du cumul |
| `0017_journal_audit.sql` | `journal_audit` + ses trois index de filtre | Agrégat distinct, régime de privilèges distinct |
| `0018_provisions_rh_appareils.sql` | `employe`, `appareil_enrole` | **Provisions**. Séparées pour que leur absence de privilèges soit lisible d'un coup d'œil |
| `0019_parametres_comptes.sql` | Les **cinq paramètres** d'établissement du cycle | **Écrite dans le schéma `etablissements`, pas `comptes`** : le catalogue des clés de configuration est un référentiel unique du produit (`0008`), pas un objet par module |
| `0020_resolution_identifiant.sql` | Rôle `kaya_auth`, politique `resolution_identifiant`, fonction `comptes.resoudre_identifiant` | **Ni ce document ni le plan n'avaient vu le problème.** `compte` porte `FORCE ROW LEVEL SECURITY` et compare `tenant_id` à `app.current_tenant` ; hors requête applicative ce réglage vaut `NULL`, donc **aucune ligne n'est visible**. Or la connexion part d'un identifiant et de rien d'autre — le tenant est ce qu'elle doit découvrir. Sans cette migration, `session_ouvrir` ne trouve jamais de compte et le produit n'a pas d'écran d'entrée |

---

## 1 · `comptes.personne` — l'identité civile

```sql
id                UUID        PRIMARY KEY,          -- UUID v7 généré côté client
tenant_id         UUID        NOT NULL,
nom               TEXT        NOT NULL,
prenoms           TEXT        NULL,
telephone         TEXT        NULL,                 -- E.164
email             TEXT        NULL,
type_piece        TEXT        NULL,                 -- non alimenté par ce cycle, voir ci-dessous
numero_piece      TEXT        NULL,                 -- idem
horodatage_client TIMESTAMPTZ NULL,                 -- indicatif
cree_le           TIMESTAMPTZ NOT NULL DEFAULT now(),
modifie_le        TIMESTAMPTZ NOT NULL DEFAULT now()
```

**Ce qu'elle ne porte pas** : aucun élément d'authentification, aucun élément de contrat.
FR-004 en fait un contrôle outillé.

⚠️ **`type_piece` et `numero_piece` sont posées et NON alimentées par ce cycle.** Ce sont des
données d'identité de client : leur alimentation relève de **SEJ-01** (fiche client) et leur
**rétention de 90 jours** de **TRX-06**. Poser la colonne sans la politique de rétention qui va
avec serait le moyen le plus simple de constituer un fichier d'identités sans durée de
conservation. Le test de provision vérifie qu'aucun point d'entrée de ce cycle ne les écrit.

**Privilèges** : `SELECT, INSERT, UPDATE` — pas de `DELETE`. Une personne ne se supprime pas tant
qu'un compte ou une entrée d'audit s'y rattache.

---

## 2 · `comptes.methode_authentification` — le refus structurel d'`OTP_SMS`

Référentiel **global**, sans `tenant_id`, sur le régime nommé de `0008` (research R-12).

```sql
code        TEXT    PRIMARY KEY,      -- 'MOT_DE_PASSE' | 'OTP_SMS'
implementee BOOLEAN NOT NULL,
libelle_cle TEXT    NOT NULL,         -- clé i18n, jamais un libellé
UNIQUE (code, implementee)            -- support de la clé étrangère composite
```

| Code | `implementee` |
|---|---|
| `MOT_DE_PASSE` | `true` |
| `OTP_SMS` | **`false`** |

**C'est le patron d'ETB-02b, repris à la lettre.** `compte` recopie la colonne `implementee` et
la contraint à `true` par clé étrangère composite : le refus d'`OTP_SMS` est **structurel**, pas
un `CHECK` que la première correction de bogue relâcherait. FR-008 exige en outre un refus
**explicite et nommé** au niveau de l'API — la base est le dernier filet, pas le premier.

---

## 3 · `comptes.compte` — l'identité d'authentification

```sql
id                        UUID        PRIMARY KEY,
tenant_id                 UUID        NOT NULL,
personne_id               UUID        NOT NULL REFERENCES comptes.personne(id),
identifiant_telephone     TEXT        NULL,
identifiant_email         TEXT        NULL,
condensat_mot_de_passe    TEXT        NOT NULL,       -- format PHC, paramètres inclus (R-03)
methode_code              TEXT        NOT NULL DEFAULT 'MOT_DE_PASSE',
methode_implementee       BOOLEAN     NOT NULL DEFAULT true,
actif                     BOOLEAN     NOT NULL DEFAULT true,
horodatage_client         TIMESTAMPTZ NULL,
cree_le                   TIMESTAMPTZ NOT NULL DEFAULT now(),
modifie_le                TIMESTAMPTZ NOT NULL DEFAULT now(),

CONSTRAINT au_moins_un_identifiant
    CHECK (identifiant_telephone IS NOT NULL OR identifiant_email IS NOT NULL),
CONSTRAINT methode_implementee_seulement
    CHECK (methode_implementee),
FOREIGN KEY (methode_code, methode_implementee)
    REFERENCES comptes.methode_authentification(code, implementee),

UNIQUE (tenant_id, identifiant_telephone),
UNIQUE (tenant_id, identifiant_email)
```

**L'unicité est par tenant** (hypothèse 3 de la spec), cohérente avec l'isolation : deux clients
distincts peuvent avoir un employé au même numéro.

**`condensat_mot_de_passe` n'est jamais lu par un `SELECT` de liste.** Le repository expose deux
chemins distincts : celui de l'authentification, qui le lit, et celui de l'affichage, qui ne le
sélectionne pas. Une structure unique le ferait traverser toutes les couches, jusqu'au risque de
le sérialiser un jour dans une réponse.

**Privilèges** : `SELECT, INSERT, UPDATE` — pas de `DELETE`. La suppression est refusée en base
par la clé étrangère de `journal_audit` (§9), ce qui **rend FR-014 structurel**.

---

## 4 · `comptes.employe` — PROVISION, et le privilège le dit

```sql
id             UUID        PRIMARY KEY,
tenant_id      UUID        NOT NULL,
personne_id    UUID        NOT NULL REFERENCES comptes.personne(id),
etablissement_id UUID      NULL,           -- pas de REFERENCES : autre module (P-04)
date_embauche  DATE        NULL,
numero_cnps    TEXT        NULL,
salaire_mineur BIGINT      NULL,           -- ENTIER d'unité mineure (principe V, porte P-10)
devise_code    TEXT        NULL,           -- le nombre de décimales vient de la DEVISE
cree_le        TIMESTAMPTZ NOT NULL DEFAULT now()
```

**Trois choses en font une provision réelle et non un début d'implémentation :**

1. **Aucun privilège d'écriture pour `kaya_app`** — pas même `SELECT`. Un chemin de code écrit par
   distraction échouerait au premier appel, pas trois mois plus tard.
2. **Aucun point d'entrée d'API**, vérifié par `backend/tests/provisions_sans_logique.rs`.
3. **`salaire_mineur` est un `BIGINT` d'unité mineure dès maintenant.** Le poser en `NUMERIC`
   « puisque personne ne s'en sert » imposerait une migration de toutes les lignes le jour de la
   paie. La porte **P-10** le vérifie ; c'est la seule colonne monétaire du cycle.

RLS `ENABLE` + `FORCE` et politique d'isolation quand même : la porte **P-07** ne connaît pas
d'exception, et une table sans politique aujourd'hui est une table sans politique le jour où on
l'ouvre.

---

## 5 · `comptes.role` — les huit, et rien d'autre

Référentiel **global**, régime nommé de `0008`.

```sql
code        TEXT     PRIMARY KEY,
portee      TEXT     NOT NULL,     -- 'ETABLISSEMENT' | 'EDITEUR'
libelle_cle TEXT     NOT NULL,
ordre       SMALLINT NOT NULL
```

| Code | Portée | Ordre |
|---|---|---|
| `proprietaire` | `ETABLISSEMENT` | 10 |
| `gerant` | `ETABLISSEMENT` | 20 |
| `receptionniste` | `ETABLISSEMENT` | 30 |
| `serveur` | `ETABLISSEMENT` | 40 |
| `caissier` | `ETABLISSEMENT` | 50 |
| `magasinier` | `ETABLISSEMENT` | 60 |
| `comptable` | `ETABLISSEMENT` | 70 |
| `admin_editeur` | **`EDITEUR`** | 80 |

`ordre` porte l'ordre d'affichage — trier sur le libellé traduit ferait changer l'écran en passant
du français à l'anglais (même raison qu'en `0008`). **Il ne porte aucune hiérarchie de droits** :
les permissions sont l'union, sans priorité (FR-017).

---

## 6 · `comptes.permission` et `comptes.role_permission`

```sql
-- permission
code        TEXT     PRIMARY KEY,     -- <module>.<objet>.<action>
module_code TEXT     NULL,            -- NULL = transversal. PAS de REFERENCES : autre schéma
libelle_cle TEXT     NOT NULL,
ordre       SMALLINT NOT NULL

-- role_permission
role_code       TEXT NOT NULL REFERENCES comptes.role(code),
permission_code TEXT NOT NULL REFERENCES comptes.permission(code),
PRIMARY KEY (role_code, permission_code)
```

**`module_code` n'a pas de clé étrangère vers `etablissements.module_activite`** — ce serait une
clé étrangère inter-schémas, interdite par le principe II. La cohérence est tenue par un test qui
lit le référentiel des modules **à travers le trait `RegistreModules`** déjà exposé par
`socle/etablissements`, et échoue si une permission nomme un module inconnu.

**Les permissions du MVP — modules livrés seulement** (décision Q3 de la spec, principe X) :

| Permission | Module | Ouvre |
|---|---|---|
| `etb.etablissement.lire` / `.modifier` | `NULL` | Identité de l'établissement |
| `etb.service.basculer` | `NULL` | Activation d'un service — **lève le provisoire de `bascule-service.ts`** |
| `etb.capacite.declarer` | `NULL` | Déclaration de capacité |
| `etb.pdv.lire` / `.gerer` | `NULL` | Points de vente et tables |
| `etb.configuration.lire` / `.ecrire` | `NULL` | Configuration héritée |
| `etb.branding.lire` / `.ecrire` | `NULL` | Identité visuelle |
| `etb.note.lire` / `.ecrire` | `NULL` | Notes internes |
| `cpt.compte.lire` / `.gerer` | `NULL` | Comptes et personnes |
| `cpt.role.attribuer` | `NULL` | Attribution et retrait de rôle |
| `cpt.session.revoquer` | `NULL` | Déconnexion à distance |
| `cpt.audit.consulter` | `NULL` | **Journal d'audit** |

Dix-sept permissions, **toutes transversales** : aucun module d'activité n'a encore d'écran.
`module_code` restera `NULL` jusqu'au cycle HEB, qui apportera les premières
(`heb.unite.attribuer`). FR-021 fait échouer le build sur toute permission qui ne garde aucune
action.

---

## 7 · `comptes.compte_role` — le cumul

```sql
id                     UUID        PRIMARY KEY,
tenant_id              UUID        NOT NULL,
compte_id              UUID        NOT NULL REFERENCES comptes.compte(id),
role_code              TEXT        NOT NULL REFERENCES comptes.role(code),
etablissement_id       UUID        NULL,        -- NULL pour admin_editeur. Pas de REFERENCES
attribue_par_compte_id UUID        NOT NULL REFERENCES comptes.compte(id),
horodatage_client      TIMESTAMPTZ NULL,
cree_le                TIMESTAMPTZ NOT NULL DEFAULT now(),

UNIQUE NULLS NOT DISTINCT (compte_id, role_code, etablissement_id)
```

**`NULLS NOT DISTINCT` n'est pas décoratif.** Sans lui, `(compte, admin_editeur, NULL)` peut être
inséré autant de fois qu'on veut : en SQL standard, deux `NULL` ne sont pas égaux, donc l'unicité
ne s'applique pas. C'est disponible depuis PostgreSQL 15 ; la cible est **18.4**.

**Le retrait d'un rôle est un `DELETE`**, pas un drapeau. L'historique n'est pas perdu : il vit au
**journal d'audit**, qui est fait pour ça et que rien ne peut réécrire. Une colonne `retire_le`
créerait un second historique, partiel et modifiable.

**Privilèges** : `SELECT, INSERT, DELETE` — pas d'`UPDATE`. Changer un rôle, c'est en retirer un
et en attribuer un autre : deux actes, deux entrées d'audit.

**L'existence de l'établissement n'est pas vérifiée par la base** (pas de clé étrangère
inter-schémas) mais **par le service**, via `EstablishmentDirectory` — ce qui donne un `404` au
lieu d'une violation de contrainte (module doré, ordre des opérations, point 3).

---

## 8 · `comptes.journal_audit` — l'immuable

```sql
id                UUID        PRIMARY KEY,        -- UUID v7 client → rejeu inoffensif
tenant_id         UUID        NOT NULL,
etablissement_id  UUID        NULL,               -- pas de REFERENCES : autre module
type_action       TEXT        NOT NULL,           -- taxonomie, research R-09
auteur_compte_id  UUID        NOT NULL REFERENCES comptes.compte(id),
cible_type        TEXT        NOT NULL,
cible_id          UUID        NULL,
contexte          JSONB       NOT NULL DEFAULT '{}'::jsonb,
horodatage_client TIMESTAMPTZ NULL,               -- indicatif, ordre d'affichage local
cree_le           TIMESTAMPTZ NOT NULL DEFAULT now()   -- AUTORITÉ SERVEUR
```

**`GRANT SELECT, INSERT` — ni `UPDATE`, ni `DELETE`.** C'est le patron de classe A du module doré,
et ici c'est aussi ce qui tient FR-033. Le contrôle statique jumeau de P-05b le double
(research R-10).

**La clé étrangère vers `compte` est le mécanisme de FR-014** : tant qu'une entrée d'audit
désigne un compte, ce compte ne peut pas être supprimé. La désactivation, elle, est un `UPDATE`
de `actif`.

⚠️ **`contexte` est du `JSONB`, et c'est là que le principe V cessait de tenir.** Un document JSON
accepte `12500.5` ou `"12 500 F"` là où le principe impose un entier d'unité mineure — et le
registre concerné trace les **écarts de caisse**, les **modifications de tarif** et les
**remises**, c'est-à-dire les trois choses que le propriétaire consulte pour détecter une fraude.
Un écart stocké en flottant, et l'audit ment sur le montant qu'il est censé prouver.

**La constitution 1.6.0 étend P-10 en conséquence.** Convention imposée, vérifiable et
implémentée par ce cycle (research R-19) :

```json
{ "ecart_mineur": -12500, "devise": "XOF", "motif": "…" }
```

| Règle | Vérifiée par |
|---|---|
| Toute clé monétaire porte le suffixe **`_mineur`** | `scripts/ci/types-monetaires.sh` — échoue aussi sur un montant nommé `montant`, `prix` ou `total` nu |
| Sa valeur est un **entier**, jamais un décimal ni une chaîne formatée | Contrôle statique **et** validation du service d'audit à l'écriture |
| Une clé **`devise`** l'accompagne au même niveau d'objet | Validation du service à l'écriture |

Les deux niveaux sont nécessaires : le contrôle statique ne voit pas un document construit
dynamiquement par un service, et la validation à l'écriture ne voit pas un littéral mal nommé
dans du code qui ne s'exécute pas encore.

Aucun montant de ce cycle n'entre encore dans `contexte` — **le contrôle est posé avant le
premier, pas après**. Son versant positif l'exerce quand même : une entrée portant un montant est
acceptée sous forme entière, refusée en flottant comme en chaîne.

**Trois index, un par filtre de FR-037** :

```sql
(tenant_id, etablissement_id, cree_le DESC)
(tenant_id, auteur_compte_id, cree_le DESC)
(tenant_id, type_action,      cree_le DESC)
```

L'ordre est `cree_le DESC, id DESC` — jamais `horodatage_client` (module doré, couche 3), et le
départage par UUID v7 évite les sauts de pagination sur deux entrées de la même transaction.

---

## 9 · `comptes.appareil_enrole` — PROVISION pour CPT-05 et CPT-06

```sql
id                     UUID        PRIMARY KEY,
tenant_id              UUID        NOT NULL,
compte_id              UUID        NOT NULL REFERENCES comptes.compte(id),
etablissement_id       UUID        NULL,
libelle                TEXT        NULL,
cle_publique           TEXT        NULL,      -- Keystore Android / Keychain iOS (CPT-05)
enrole_le              TIMESTAMPTZ NULL,
revoque_le             TIMESTAMPTZ NULL,
attestation_verdict    TEXT        NULL,      -- Play Integrity / App Attest (CPT-06)
attestation_verifiee_le TIMESTAMPTZ NULL,
derniere_latitude      NUMERIC     NULL,      -- géorepérage SOUPLE, alerte seulement
derniere_longitude     NUMERIC     NULL,
derniere_position_le   TIMESTAMPTZ NULL,
cree_le                TIMESTAMPTZ NOT NULL DEFAULT now()
```

**Aucune colonne d'adresse MAC, et il n'y en aura jamais** (FR-042, principe IX, cadrage §12.2).
Écrit ici pour que l'absence soit une décision lisible et non un oubli.

**Coordonnées en `NUMERIC`, jamais en flottant** — même règle que les quantités (principe V).

**Aucun privilège d'écriture pour `kaya_app`**, comme `employe`. Le rayon de géorepérage
(300 m, alerte seulement) est un **paramètre de configuration** ajouté au catalogue d'ETB-04, pas
une colonne : c'est un réglage d'établissement, pas une propriété d'appareil.

---

## Les dix types d'événements outbox — dont **neuf** émis

**Total du produit après ce cycle : 22 types, 13 + 9** — et non les 21 du plan. Le tableau ci-dessous
déclare dix lignes ; `compte.modifie` n'en est pas une de plus au décompte, faute d'opération qui la
produise.

Nomenclature `agregat.action`, comme les **treize** types existants. *Treize, et non les onze qu'on
lit ailleurs* : le tableau du cycle 002 compte onze **lignes**, mais deux d'entre elles portent deux
types chacune — « `point_de_vente.cree` / `.modifie` » et « `table_pdv.creee` / `.desactivee` ». Un
décompte tiré du nombre de lignes d'un tableau compte des lignes, pas des types.

| Type | Agrégat | Portée | Émis par |
|---|---|---|---|
| `personne.creee` | `personne` | tenant | Création d'une personne |
| `personne.modifiee` | `personne` | tenant | Modification |
| `compte.cree` | `compte` | tenant | Création d'un compte |
| `compte.modifie` | `compte` | tenant | ⚠️ **PROVISION — aucun émetteur.** Le contrat n'expose aucune modification d'identifiant (§10 à 16 : créer, lister, lire, changer l'état, changer le mot de passe, attribuer et retirer un rôle). Écart entre deux documents de conception écrits en parallèle. La ligne reste ici pour que le type soit **déjà décidé** le jour où l'opération existera ; `TYPES_SANS_EMETTEUR` de `couverture_portes.rs` la nomme, sans quoi la porte chercherait un dixième émetteur qui n'existe pas |
| `compte.desactive` | `compte` | tenant | Désactivation |
| `compte.reactive` | `compte` | tenant | Réactivation |
| `compte.mot_de_passe_change` | `compte` | tenant | Changement de secret — **le payload ne porte JAMAIS le secret ni son condensat** |
| `role.attribue` | `compte_role` | **établissement** | Attribution |
| `role.retire` | `compte_role` | **établissement** | Retrait |
| `session.revoquee` | `compte` | tenant | Déconnexion à distance |

**Ce qui n'émet pas, et pourquoi** (research R-15) : la connexion, le rafraîchissement et l'échec
d'authentification. Ce ne sont pas des transitions d'état métier ; les inscrire au grand livre —
permanent, à rétention illimitée — y écrirait la liste horodatée des présences du personnel.

**Charge utile dénormalisée** (TRX-02, règle 2) : `role.attribue` porte le code du rôle, le nom du
compte cible, celui de l'auteur et l'établissement — pas seulement des identifiants. Un an plus
tard, le rôle peut avoir changé de libellé.

**Les deux tenants de démonstration**, sans exception : exigence 5 du § « Couverture des portes ».

---

## Journal du registre des classes hors-ligne

`docs/registre-classes-offline.md` §5.2 **décrit déjà les neuf lignes de ce cycle** — il a été
écrit d'avance. Ce cycle **n'ajoute pas de ligne**, il ajoute une entrée au journal §13 constatant
que les entités déclarées sont désormais implémentées, plus **une ligne nouvelle** :

| Entité | Classe | Branche | Réf. |
|---|---|---|---|
| `methode_authentification` — référentiel | **C** | C2 — référentiel | CPT-01 |

`backend/tests/classes_offline.rs` compare **table réelle → registre** et fait échouer le build
sur toute table non déclarée : les dix tables de ce cycle doivent y figurer avant que la migration
ne soit considérée comme terminée.
