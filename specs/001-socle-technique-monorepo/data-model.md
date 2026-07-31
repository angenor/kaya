# Phase 1 — Modèle de données et migrations

**Cycle** : 001 — Socle technique du monorepo Kaya
**Date** : 2026-07-30
**Dépend de** : [research.md](./research.md) · [spec.md](./spec.md)

> **Conventions non négociables appliquées ici** : identifiants métier en **français sans
> accent** ; montants en **entiers d'unité mineure** ; quantités en **`NUMERIC`** ; toute table
> porte `tenant_id` ; **un schéma PostgreSQL par module** ; aucune requête ne joint deux schémas
> de modules.

---

## 1. Schémas PostgreSQL créés par ce cycle

| Schéma | Crate propriétaire | Contenu de ce cycle |
|---|---|---|
| `etablissements` | `socle/etablissements` | `tenant`, `etablissement`, `note_etablissement` |
| `synchronisation` | `socle/synchronisation` | `evenement_outbox` |
| `fiscalite` | `socle/fiscalite` | `mapping_comptable`, `exercice_comptable` *(provisions)* |

**Pourquoi les provisions comptables vont dans `fiscalite`** : la constitution (principe II) fixe
limitativement les neuf crates de `socle/` — il n'y a pas de crate `comptabilite` et en créer un
demanderait un amendement. Parmi les neuf, `fiscalite` est le seul dont le domaine est la
production d'obligations réglementaires à partir d'événements métier ; `mapping_comptable`
associe un type d'événement à un compte de débit, un compte de crédit et un journal, ce qui est
exactement cela. `documents` a été écarté : il traite la numérotation et le cycle de vie des
pièces, pas leur traduction comptable.

---

## 2. `etablissements.tenant` et `etablissements.etablissement` — forme minimale

**Écart de périmètre assumé, à lire avant d'implémenter.** Ces deux tables appartiennent à
**ETB-01**, cycle suivant. Ce cycle en crée la **forme minimale strictement nécessaire** :
`note_etablissement` doit se rattacher à un établissement, `evenement_outbox` porte
`etablissement_id`, et la RLS n'a pas de sens sans un tenant réel à isoler. Les créer ici évite
une table orpheline ; les créer **complètes** empiéterait sur ETB-01.

ETB-01 les enrichira par **migration additive** — juridiction, classement, commune, NCC, adresse,
branding — conformément au principe I(b) : une migration appliquée n'est jamais modifiée.

```sql
CREATE TABLE etablissements.tenant (
    id              UUID PRIMARY KEY,                  -- UUID v7 généré côté client
    nom             TEXT        NOT NULL,
    cree_le         TIMESTAMPTZ NOT NULL DEFAULT now(),
    modifie_le      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE etablissements.etablissement (
    id              UUID PRIMARY KEY,                  -- UUID v7 généré côté client
    tenant_id       UUID        NOT NULL REFERENCES etablissements.tenant(id),
    nom             TEXT        NOT NULL,
    fuseau_horaire  TEXT        NOT NULL,              -- ex. 'Africa/Abidjan'
    devise          CHAR(3)     NOT NULL,              -- ISO 4217 ; XOF, 0 décimale
    cree_le         TIMESTAMPTZ NOT NULL DEFAULT now(),
    modifie_le      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX ON etablissements.etablissement (tenant_id);
```

`tenant` porte `tenant_id` sous la forme de sa propre clé primaire : sa politique RLS compare
`id`, pas une colonne séparée. C'est le seul cas du produit où les deux coïncident, et il est
signalé pour que la porte P-07 ne le traite pas comme une exception silencieuse.

**Classe hors-ligne** : `tenant` et `etablissement` sont **C** (`docs/registre-classes-offline.md`
§5.1) — référentiel, jamais écrit hors ligne.

---

## 3. `etablissements.note_etablissement` — entité du module doré

Note interne libre attachée à un établissement. **Classe A** : append-only, commutative, sans
contrainte d'unicité, sans effet monétaire (arbre de décision du cadrage §11.2, branche A4).

```sql
CREATE TABLE etablissements.note_etablissement (
    id                UUID        PRIMARY KEY,          -- UUID v7 généré côté client (dédup.)
    tenant_id         UUID        NOT NULL REFERENCES etablissements.tenant(id),
    etablissement_id  UUID        NOT NULL REFERENCES etablissements.etablissement(id),
    auteur_compte_id  UUID        NOT NULL,             -- pas de FK : socle/comptes viendra en CPT-01
    texte             TEXT        NOT NULL CHECK (length(btrim(texte)) BETWEEN 1 AND 2000),
    horodatage_client TIMESTAMPTZ     NULL,             -- indicatif, jamais de logique métier
    cree_le           TIMESTAMPTZ NOT NULL DEFAULT now()  -- horodatage d'AUTORITÉ serveur
);
CREATE INDEX ON etablissements.note_etablissement (tenant_id, etablissement_id, cree_le DESC);
```

**Trois points qui font de cette table un patron et non une table jetable** :

- **`id` fourni par le client, pas généré par la base.** C'est ce qui rend le rejeu inoffensif
  (cadrage §11.5.1) : trois envois de la même écriture entrent en conflit de clé primaire et
  produisent un seul enregistrement. Une clé générée côté base produirait trois lignes. Le
  `INSERT` porte donc `ON CONFLICT (id) DO NOTHING`, et c'est ce que le test de rejeu vérifie.
- **`horodatage_client` et `cree_le` sont deux colonnes distinctes.** Le premier vient du
  terminal et n'est jamais utilisé par une règle ; le second est posé par le serveur et fait
  autorité (constitution, principe IV). Les fusionner serait l'erreur que le cadrage §11.4
  décrit comme celle des horloges non fiables.
- **Aucune clé étrangère vers `compte`.** Le crate `socle/comptes` n'existe pas encore ; une FK
  inter-schémas de modules serait de toute façon interdite (principe II — aucune requête ne joint
  deux schémas de modules). L'intégrité référentielle inter-modules passe par le trait exposé,
  pas par la base. **C'est le point du patron le plus contre-intuitif** et il est documenté comme
  tel dans `docs/module-dore.md`.

**Tests de classe A obligatoires** (`docs/user-stories-v1.md` §0.7) :

| Test | Ce qu'il vérifie |
|---|---|
| Rejeu | Trois envois du même `id` → **un seul** enregistrement |
| Désordre | Trois notes appliquées dans les **six** ordres possibles → même état final |

---

## 4. `synchronisation.evenement_outbox` — le grand livre permanent

**L'entité centrale du cycle.** Classe **A** (`docs/registre-classes-offline.md` §5.6, branche A4
— append-only, immuable, rétention illimitée), tant pour l'écriture que pour le marquage
« publié ».

```sql
CREATE TABLE synchronisation.evenement_outbox (
    id                      UUID        PRIMARY KEY,     -- UUID v7 généré côté client
    tenant_id               UUID        NOT NULL,
    etablissement_id        UUID            NULL,        -- nul pour un événement de niveau tenant
    sequence_etablissement  BIGINT      NOT NULL,        -- monotone par établissement (R-07)
    type_evenement          TEXT        NOT NULL,        -- ex. 'note_etablissement.creee'
    agregat                 TEXT        NOT NULL,        -- ex. 'note_etablissement'
    agregat_id              UUID        NOT NULL,
    version_schema          SMALLINT    NOT NULL,        -- version du format de `payload` (R-06)
    payload                 JSONB       NOT NULL,        -- COMPLET et DÉNORMALISÉ
    survenu_le              TIMESTAMPTZ NOT NULL,        -- horodatage d'AUTORITÉ serveur
    publie_le               TIMESTAMPTZ     NULL,        -- NULL = non publié. JAMAIS de suppression
    CONSTRAINT evenement_outbox_sequence_unique
        UNIQUE (etablissement_id, sequence_etablissement)
);

CREATE INDEX ON synchronisation.evenement_outbox (publie_le) WHERE publie_le IS NULL;
CREATE INDEX ON synchronisation.evenement_outbox (tenant_id, survenu_le);
CREATE INDEX ON synchronisation.evenement_outbox (agregat, agregat_id);
```

**L'index partiel sur `publie_le IS NULL` est le seul qui reste petit indéfiniment.** C'est ce qui
permet à la rétention illimitée de ne pas dégrader le worker : la table croît sans fin, mais
l'index de travail ne contient que les événements en attente. Sans lui, la publication
ralentirait proportionnellement à l'historique — et la première réaction serait de purger, ce que
TRX-02 interdit.

### 4.1 Immuabilité — les trois couches (R-05)

```sql
-- Couche 1 : le rôle de runtime ne peut physiquement pas modifier.
REVOKE UPDATE, DELETE ON synchronisation.evenement_outbox FROM kaya_app;
GRANT  SELECT, INSERT  ON synchronisation.evenement_outbox TO   kaya_app;
GRANT  UPDATE (publie_le) ON synchronisation.evenement_outbox TO kaya_app;

-- Couche 2 : le déclencheur s'applique AUSSI au propriétaire des tables.
CREATE FUNCTION synchronisation.refuser_mutation_evenement() RETURNS trigger AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        RAISE EXCEPTION 'evenement_outbox est un grand livre permanent : suppression interdite';
    END IF;
    -- Seule mutation tolérée : marquage de publication, NULL -> valeur, jamais l'inverse.
    IF OLD.publie_le IS NOT NULL OR NEW.publie_le IS NULL
       OR NEW.* IS DISTINCT FROM (OLD.*::synchronisation.evenement_outbox
                                  #= hstore('publie_le', NEW.publie_le::text)) THEN
        RAISE EXCEPTION 'evenement_outbox est immuable : seul le marquage de publication est permis';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER evenement_outbox_immuable
    BEFORE UPDATE OR DELETE ON synchronisation.evenement_outbox
    FOR EACH ROW EXECUTE FUNCTION synchronisation.refuser_mutation_evenement();
```

> **Note d'implémentation** : la comparaison ligne à ligne ci-dessus est indicative. La forme
> exacte (`hstore`, comparaison colonne par colonne, ou `to_jsonb(OLD) - 'publie_le'`) est arrêtée
> à l'écriture du module doré, en privilégiant celle qui **n'ajoute aucune extension PostgreSQL**.
> `to_jsonb(NEW) - 'publie_le' = to_jsonb(OLD) - 'publie_le'` fait le travail sans `hstore` et est
> le candidat retenu sauf contre-indication.

### 4.2 Ce que `payload` doit contenir — et pourquoi c'est le cœur du cycle

Pour un encaissement, la charge utile porte **montant, mode de règlement, contrepartie,
ventilation de taxes et référence de document** — jamais un identifiant renvoyant à une autre
table. Le critère de conception, opposable en revue :

> Un lecteur qui n'a **que cette ligne** et le numéro de `version_schema` doit pouvoir dire ce qui
> s'est passé, pour quel montant, avec quelles taxes et sur quel document.

C'est ce que vérifie le test de reconstitution autonome (R-11), exécuté sous
`kaya_ledger_reader`, un rôle qui n'a le droit de lire aucune autre table.

**Format de la charge utile, version 1** (exemple de référence figé avec le test) :

```jsonc
{
  "montant_mineur": 15500,          // entier d'unité mineure, JAMAIS de flottant
  "devise": "XOF",
  "mode_reglement": "ESPECES",
  "contrepartie": { "type": "CLIENT", "libelle": "Kouassi Adjoua" },
  "taxes": [
    { "code": "TVA",             "assiette_mineur": 12712, "taux_millieme": 180, "montant_mineur": 2288 },
    { "code": "TAXE_NUITEE",     "assiette_mineur": null,  "taux_millieme": null, "montant_mineur":  500 }
  ],
  "document": { "type": "FACTURE", "numero": "F-2026-000123" }
}
```

Les taux sont en **millièmes entiers** (`180` = 18 %) : un taux en flottant rouvrirait par la
petite porte le risque d'arrondi que le principe V ferme sur les montants.

### 4.3 Événements émis par ce cycle

| Type | Agrégat | Émis quand | `version_schema` |
|---|---|---|---|
| `note_etablissement.creee` | `note_etablissement` | Création d'une note, **dans la transaction d'insertion** | 1 |

Un seul type, et c'est voulu : il n'existe qu'une transition d'état métier dans ce cycle. Sa
valeur est d'être le **cas d'usage du patron** — c'est lui que tous les cycles suivants
recopieront. La charge utile de cet événement est non financière ; le format financier ci-dessus
est exercé par le jeu de cas figé du test de reconstitution, seedé indépendamment.

---

## 5. Provisions comptables — `fiscalite` (TRX-02b)

**Tables seulement. Aucun endpoint, aucun écran, aucune règle métier.** Classe **C**
(`docs/registre-classes-offline.md` §10).

```sql
CREATE TABLE fiscalite.exercice_comptable (
    id          UUID        PRIMARY KEY,
    tenant_id   UUID        NOT NULL REFERENCES etablissements.tenant(id),
    debut       DATE        NOT NULL,
    fin         DATE        NOT NULL,
    statut      TEXT        NOT NULL CHECK (statut IN ('ouvert', 'clos')),
    CONSTRAINT exercice_comptable_bornes CHECK (fin > debut),
    CONSTRAINT exercice_comptable_sans_chevauchement
        EXCLUDE USING gist (
            tenant_id WITH =,
            daterange(debut, fin, '[)') WITH &&
        )
);

CREATE TABLE fiscalite.mapping_comptable (
    id              UUID    PRIMARY KEY,
    tenant_id       UUID    NOT NULL REFERENCES etablissements.tenant(id),
    type_evenement  TEXT    NOT NULL,
    compte_debit    TEXT    NOT NULL,
    compte_credit   TEXT    NOT NULL,
    journal         TEXT    NOT NULL,
    CONSTRAINT mapping_comptable_unique UNIQUE (tenant_id, type_evenement)
);
```

**La contrainte d'exclusion sur `exercice_comptable` n'est pas décorative** : deux exercices qui
se chevauchent rendraient « la période est-elle close ? » indécidable, et cette question est la
seule règle que TRX-02b impose (FR-046). Elle est posée maintenant parce qu'une contrainte
d'exclusion ajoutée sur une table déjà peuplée échoue sur les données existantes.

C'est aussi le **premier usage d'`EXCLUDE USING gist` du produit** — celui que HEB-02 reprendra
sur `tstzrange` pour la disponibilité des unités. À ce titre il vaut spike : il vérifie
l'extension `btree_gist` et le mapping de type sqlx 0.9 sur un cas sans enjeu, avant que HEB-02
en dépende (cf. point ouvert du gel, `docs/versions-gelees.md` en-tête).

**Le refus d'écriture sur période close** est implémenté comme un **déclencheur**, pas comme une
règle applicative — une règle applicative serait contournée par la première migration de données
venue, et la provision perdrait son sens.

---

## 6. Politiques RLS — patron unique appliqué à toutes les tables

Le même patron pour chaque table, sans exception. Toute divergence est un signal d'erreur.

```sql
ALTER TABLE <schema>.<table> ENABLE  ROW LEVEL SECURITY;
ALTER TABLE <schema>.<table> FORCE   ROW LEVEL SECURITY;   -- s'applique AUSSI au propriétaire

CREATE POLICY isolation_tenant ON <schema>.<table>
    USING       (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK  (tenant_id = current_setting('app.current_tenant', true)::uuid);
```

**Trois détails qui décident si l'isolation tient ou non** :

- **`USING` *et* `WITH CHECK`.** `USING` filtre la lecture ; `WITH CHECK` empêche l'écriture d'une
  ligne portant le tenant d'autrui. Sans `WITH CHECK`, un tenant peut **insérer** chez un autre —
  la fuite la moins visible et la plus grave.
- **`current_setting(..., true)`** — le second argument évite l'erreur quand le paramètre est
  absent ; l'expression vaut alors `NULL`, la comparaison est `NULL`, et **aucune ligne ne
  passe**. Une transaction sans contexte de tenant ne voit donc rien, ce qui est la propriété
  exigée par le scénario US4.6. Avec `current_setting(...)` sans le second argument, la requête
  lèverait une erreur — moins sûr, car un `catch` mal placé pourrait la transformer en accès
  ouvert.
- **`FORCE`** — sans lui, `kaya_owner` ignore la politique et toute maintenance devient une fuite
  potentielle.

`etablissements.tenant` est le seul cas particulier : sa politique compare `id`.

**Application par table de ce cycle** :

| Table | `ENABLE` | `FORCE` | Colonne comparée |
|---|---|---|---|
| `etablissements.tenant` | ✅ | ✅ | `id` |
| `etablissements.etablissement` | ✅ | ✅ | `tenant_id` |
| `etablissements.note_etablissement` | ✅ | ✅ | `tenant_id` |
| `synchronisation.evenement_outbox` | ✅ | ✅ | `tenant_id` |
| `fiscalite.exercice_comptable` | ✅ | ✅ | `tenant_id` |
| `fiscalite.mapping_comptable` | ✅ | ✅ | `tenant_id` |

---

## 7. Migrations à créer

Numérotation sqlx, jamais modifiée après application (principe I(b)).

| # | Fichier | Contenu |
|---|---|---|
| 0001 | `0001_roles_et_schemas.sql` | Rôles `kaya_owner`, `kaya_app`, `kaya_ledger_reader` ; schémas `etablissements`, `synchronisation`, `fiscalite` ; extension `btree_gist` |
| 0002 | `0002_etablissements_socle.sql` | `tenant`, `etablissement` + RLS |
| 0003 | `0003_outbox.sql` | `evenement_outbox`, séquence par établissement, index partiel, RLS, `REVOKE`, déclencheur d'immuabilité |
| 0004 | `0004_note_etablissement.sql` | `note_etablissement` + RLS (module doré) |
| 0005 | `0005_provisions_comptables.sql` | `exercice_comptable`, `mapping_comptable`, RLS, déclencheur de période close |

**Ordre imposé** : 0003 avant 0004. Le module doré émet un événement dans sa transaction — sa
table cible doit exister avant.

---

## 8. Classe hors-ligne de chaque entité touchée

À reporter dans `docs/registre-classes-offline.md` **dans le même changement** que la migration
(constitution, artefacts de gouvernance).

| Entité / opération | Classe | Branche | Déjà au registre ? |
|---|---|---|---|
| `tenant` — création, modification | **C** | C2 | ✅ §5.1 |
| `etablissement` — création, modification | **C** | C2 | ✅ §5.1 |
| `note_etablissement` — création | **A** | A4 | ❌ **à ajouter au §5.1** |
| `evenement_outbox` — écriture | **A** | A4 | ✅ §5.6 |
| `evenement_outbox` — marquage publié | **A** | A4 | ✅ §5.6 |
| `mapping_comptable`, `exercice_comptable` | **C** | C2 | ✅ §10 |

Une seule ligne à ajouter — `note_etablissement` — plus une entrée au §13 (journal des
modifications) du registre.

---

## 9. Ce que ce cycle NE crée pas, et qui pourrait manquer

Signalé pour qu'aucune tâche ne l'invente en cours de route :

- **Aucune table de `verticales/`** — ni unité louable, ni séjour, ni article. Les crates existent
  et compilent à vide.
- **Aucune table de `socle/comptes`** — l'authentification est CPT-01. Le module doré manipule un
  `auteur_compte_id` opaque, non contraint.
- **Aucune valeur monétaire persistée en dehors du jeu de cas figé du test de reconstitution.**
  La porte P-10 (montants entiers, quantités `NUMERIC`) n'a donc presque rien à vérifier ; elle
  est installée à vide (R-15).
- **Aucune table du référentiel de modules ni de capacités** — c'est ETB-02/ETB-02b. Le refus
  explicite de toute capacité autre que `STOCK`/`SIMPLE` (porte P-06) n'a pas de cible et est
  installé à vide.
