# Phase 1 — Modèle de données

**Cycle 006** — Fiches clients, arrivée, départ et prolongation · 2026-08-03
**Plan** : [plan.md](./plan.md) · **Recherche** : [research.md](./research.md)

**Neuf tables neuves, deux tables altérées, six migrations, deux schémas.** Aucune extension
PostgreSQL nouvelle.

---

## Vue d'ensemble

| # | Table | Schéma | Classe | Privilèges `kaya_app` | Migration |
|---|---|---|---|---|---|
| 1 | `client` | `comptes` | **C** | `SELECT, INSERT, UPDATE` | `0029` |
| 2 | `preference_personne` | `comptes` | **A** | `SELECT, INSERT` | `0029` |
| 3 | `sejour` | `hebergement` | **B** | `SELECT, INSERT, UPDATE` | `0031` |
| 4 | `accompagnant` | `hebergement` | **A** | `SELECT, INSERT, UPDATE` | `0031` |
| 5 | `note_sejour` | `hebergement` | **B** | `SELECT, INSERT, UPDATE` | `0032` |
| 6 | `ligne_sejour` | `hebergement` | **B** | `SELECT, INSERT` | `0032` |
| 7 | `fiche_police` | `hebergement` | **B** | `SELECT, INSERT, UPDATE` | `0033` |
| 8 | `numerotation_fiche_police` | `hebergement` | **B** | `SELECT, INSERT, UPDATE` | `0033` |
| 9 | `taxe_sejour_constat` | `hebergement` | **B** | `SELECT, INSERT` | `0034` |

**Tables altérées** : `comptes.personne` (`0029`), `hebergement.occupation` (`0031`).
**Privilège élargi** : `synchronisation.reconciliation_orpheline` passe de `SELECT` à
`SELECT, INSERT` (`0031`) — elle cesse d'être une provision.

> **Trois jeux de privilèges disent une règle que le code ne peut pas contourner.**
> `ligne_sejour` et `taxe_sejour_constat` n'ont **pas d'`UPDATE`** : un prix verrouillé et un
> constat figé ne se modifient pas, quelle que soit la ligne de code écrite au-dessus.
> `preference_personne` n'en a pas non plus : c'est une entité **append-only** de classe A, sur le
> patron exact de `note_etablissement` (module doré, couche 1 — « les privilèges disent la classe »).
> **Aucune des neuf tables n'a `DELETE`.**

---

## Migration `0029` — la fiche client

### `comptes.personne` — quatre colonnes ajoutées

```sql
ALTER TABLE comptes.personne
    ADD COLUMN nom_repli           TEXT NULL,
    ADD COLUMN telephone_repli     TEXT NULL,
    ADD COLUMN numero_piece_repli  TEXT NULL,
    ADD COLUMN piece_capturee_le   TIMESTAMPTZ NULL;
```

| Colonne | Rôle |
|---|---|
| `nom_repli` | `nom` et `prenoms` concaténés, repliés (R-04) — minuscules, sans signes diacritiques, sans apostrophe |
| `telephone_repli` | Chiffres seuls, préfixés de l'indicatif de l'établissement quand la saisie n'en porte pas (R-06) |
| `numero_piece_repli` | Alphanumérique en majuscules, sans espace ni tiret |
| `piece_capturee_le` | **FR-013** — l'instant de capture, pour que la rétention paramétrable de TRX-06 s'applique plus tard **sans migration** |

```sql
CREATE INDEX personne_nom_repli_idx          ON comptes.personne (tenant_id, nom_repli text_pattern_ops);
CREATE INDEX personne_telephone_repli_idx    ON comptes.personne (tenant_id, telephone_repli);
CREATE INDEX personne_numero_piece_repli_idx ON comptes.personne (tenant_id, numero_piece_repli);

COMMENT ON COLUMN comptes.personne.type_piece IS
    'Alimentée depuis SEJ-01 (cycle 006). Rétention 90 jours TRX-06 — encore DUE, dette nommée.';
```

> ⚠️ **`0015` n'est pas modifiée**, et c'est P-02. Son commentaire décrit l'état du cycle 003 et
> reste vrai de ce cycle-là. La mise à jour passe par `COMMENT ON COLUMN` ici.

> **`text_pattern_ops` n'est pas décoratif** : sans lui, un `LIKE 'kouam%'` n'emploie pas l'index
> dès que la collation de la base n'est pas `C`. C'est le genre de détail qui se découvre en
> production, sur le seul écran dont la lenteur condamne le produit.

### `comptes.client` — la qualification

```sql
CREATE TABLE comptes.client (
    -- L'identifiant EST celui de la personne. Pas de clé technique séparée : une personne est
    -- cliente ou ne l'est pas, il n'y a pas deux fiches à réconcilier.
    personne_id    UUID        PRIMARY KEY REFERENCES comptes.personne (id),
    tenant_id      UUID        NOT NULL,

    -- Les deux attributs que CPT n'a aucune raison de connaître (R-01).
    date_naissance DATE        NULL,
    nationalite    TEXT        NULL CHECK (length(btrim(nationalite)) BETWEEN 2 AND 80),

    horodatage_client TIMESTAMPTZ NULL,
    cree_le        TIMESTAMPTZ NOT NULL DEFAULT now(),
    modifie_le     TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

**La clé étrangère vers `comptes.personne` est légale** : même schéma. C'est le seul endroit du
cycle où une clé étrangère traverse un agrégat, et elle ne traverse aucun schéma.

**Ce que cette table fait, et qui n'est pas cosmétique** : elle **qualifie**. Sans elle, chercher
« Kouamé » à la réception ferait apparaître la femme de ménage — `comptes.personne` porte le
personnel autant que les clients (CPT-00). La recherche joint `personne` et `client` **dans le même
schéma**, en une requête, ce qui est la condition de la cible des 300 ms.

### `comptes.preference_personne` — classe A, append-only

```sql
CREATE TABLE comptes.preference_personne (
    id                UUID        PRIMARY KEY,          -- UUID v7 généré côté client
    tenant_id         UUID        NOT NULL,
    personne_id       UUID        NOT NULL REFERENCES comptes.personne (id),

    texte             TEXT        NOT NULL CHECK (length(btrim(texte)) BETWEEN 1 AND 2000),

    horodatage_client TIMESTAMPTZ NULL,                 -- indicatif, AUCUNE règle (P-23)
    cree_le           TIMESTAMPTZ NOT NULL DEFAULT now()  -- AUTORITÉ
);
```

**Append-only, exactement comme `note_etablissement`.** La préférence courante est **la ligne la
plus récente**, jamais une colonne mise à jour. Une correction est une ligne nouvelle. C'est ce qui
rend le rejeu inoffensif et le désordre commutatif — les deux propriétés que `tester_classe_a!`
vérifie.

`GRANT SELECT, INSERT` — **ni `UPDATE`, ni `DELETE`.**

### RLS — le patron identique partout

```sql
ALTER TABLE comptes.client              ENABLE ROW LEVEL SECURITY;
ALTER TABLE comptes.client              FORCE  ROW LEVEL SECURITY;
CREATE POLICY isolation_tenant ON comptes.client
    USING      (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);
-- idem preference_personne
```

---

## Migration `0030` — sept permissions

Sur le patron de `0022` (cycle 004), la seule migration du projet à poser des permissions
rattachées à un module d'activité.

```sql
INSERT INTO comptes.permission (code, module_code, libelle_cle, ordre) VALUES
    ('sej.client.lire',        NULL,          'comptes.permissions.sej.client.lire',        230),
    ('sej.client.gerer',       NULL,          'comptes.permissions.sej.client.gerer',       240),
    ('heb.sejour.lire',        'HEBERGEMENT', 'comptes.permissions.heb.sejour.lire',        250),
    ('heb.sejour.ouvrir',      'HEBERGEMENT', 'comptes.permissions.heb.sejour.ouvrir',      260),
    ('heb.sejour.clore',       'HEBERGEMENT', 'comptes.permissions.heb.sejour.clore',       270),
    ('heb.sejour.prolonger',   'HEBERGEMENT', 'comptes.permissions.heb.sejour.prolonger',   280),
    ('heb.sejour.changer_unite','HEBERGEMENT','comptes.permissions.heb.sejour.changer_unite',290);
```

**Les deux permissions de client sont transversales** (`module_code = NULL`) : un maquis ou un bar
seul en aura besoin dès SEJ-05, sans module hébergement (R-13).

**Attribution** : `receptionniste` et `gerant` reçoivent les sept ; `proprietaire` reçoit
`sej.client.lire` et `heb.sejour.lire`. Chaque permission garde **une opération réellement servie
par ce cycle** — la règle du cycle 003 refuse une permission sans contrepartie.

---

## Migration `0031` — le séjour

### `hebergement.sejour`

```sql
CREATE TABLE hebergement.sejour (
    id                UUID        PRIMARY KEY,          -- UUID v7 généré côté client
    tenant_id         UUID        NOT NULL,
    etablissement_id  UUID        NOT NULL,

    -- ⚠️ AUCUN `REFERENCES` : ce serait une clé étrangère inter-schémas (principe II, P-04).
    -- La lecture passe par le trait `AnnuaireClients`. NULL est LÉGAL : un passage s'enregistre
    -- sans fiche, la pièce d'identité se saisissant APRÈS la clé (maquette R4, FR-023).
    client_id         UUID        NULL,

    statut            TEXT        NOT NULL DEFAULT 'en_cours'
        CONSTRAINT sejour_statut_connu CHECK (statut IN ('en_cours', 'clos')),

    -- Horodatage d'AUTORITÉ. Le calcul de durée le lit ; jamais l'horloge d'un terminal (P-23).
    ouvert_le         TIMESTAMPTZ NOT NULL DEFAULT now(),
    clos_le           TIMESTAMPTZ NULL,

    horodatage_client TIMESTAMPTZ NULL,
    cree_le           TIMESTAMPTZ NOT NULL DEFAULT now(),
    modifie_le        TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT sejour_cloture_coherente
        CHECK ((statut = 'clos') = (clos_le IS NOT NULL)),
    CONSTRAINT sejour_cloture_apres_ouverture
        CHECK (clos_le IS NULL OR clos_le >= ouvert_le)
);

CREATE INDEX sejour_en_cours_idx
    ON hebergement.sejour (tenant_id, etablissement_id, statut, ouvert_le DESC);
CREATE INDEX sejour_par_client_idx
    ON hebergement.sejour (tenant_id, client_id, ouvert_le DESC);
```

`sejour_par_client_idx` sert l'historique (`GET /clients/{id}/sejours`) — servi **depuis
`hebergement`**, jamais depuis `comptes`, sans quoi ce serait à la fois une jointure inter-schémas
et une arête `socle/ → verticales/`.

**Pas de `DELETE`.** Un séjour ne se supprime pas ; il se clôt. Accorder `DELETE` permettrait
d'effacer une nuit vendue, et le classement en B deviendrait faux **sans que rien ne le signale**.

### `hebergement.accompagnant` — classe A

```sql
CREATE TABLE hebergement.accompagnant (
    id                UUID        PRIMARY KEY,          -- UUID v7 généré côté client
    tenant_id         UUID        NOT NULL,
    sejour_id         UUID        NOT NULL REFERENCES hebergement.sejour (id),

    -- Un nom suffit (FR-015). Le reste est facultatif, et c'est ce qui rend l'ajout tenable
    -- au comptoir : demander une pièce par accompagnant coûterait la cible des 60 secondes.
    nom               TEXT        NOT NULL CHECK (length(btrim(nom)) BETWEEN 1 AND 200),
    prenoms           TEXT        NULL,
    date_naissance    DATE        NULL,
    nationalite       TEXT        NULL,
    type_piece        TEXT        NULL,
    numero_piece      TEXT        NULL,
    piece_capturee_le TIMESTAMPTZ NULL,

    -- Un accompagnant se retire tant que le séjour est ouvert : `retire_le` plutôt qu'un DELETE,
    -- sans quoi la fiche de police perdrait la trace d'une personne qui a bien été déclarée.
    retire_le         TIMESTAMPTZ NULL,

    horodatage_client TIMESTAMPTZ NULL,                 -- indicatif, AUCUNE règle (P-23)
    cree_le           TIMESTAMPTZ NOT NULL DEFAULT now()  -- AUTORITÉ
);
```

> ⚠️ **`accompagnant` porte `type_piece` et `numero_piece`.** `provisions_sans_logique.rs` échoue
> sur toute colonne de pièce d'identité apparaissant **sur une provision RH** — `employe`,
> `appareil_enrole`. Cette table n'en est pas une : c'est un porteur légitime, dû à la fiche de
> police (FR-046). Le contrôle doit donc être **relu et son périmètre confirmé**, non contourné :
> il porte sur les provisions, et `piece_capturee_le` est présent ici pour la même raison que sur
> `personne` — la rétention de TRX-06 s'appliquera **sans migration**.

### `hebergement.occupation` — une colonne ajoutée

```sql
ALTER TABLE hebergement.occupation
    ADD COLUMN sejour_id UUID NULL REFERENCES hebergement.sejour (id);

CREATE INDEX occupation_par_sejour_idx ON hebergement.occupation (sejour_id)
    WHERE sejour_id IS NOT NULL;
```

**`NULL` est nécessaire** : l'endpoint d'attribution nu du cycle 004 existe toujours et n'ouvre
aucun séjour. Le rendre obligatoire casserait une opération servie.

**Un séjour porte une à N occupations** — c'est ce qui rend le changement d'unité possible sans
casser l'historique (FR-079, FR-081).

> ⚠️ **La contrainte d'exclusion n'est pas touchée, et c'est vérifié plutôt que supposé.** P-09 est
> **ré-exercée** après cette migration : type `tstzrange`, contrainte présente avec ses deux
> opérateurs, et deux arrivées concurrentes chevauchantes **par le parcours de séjour** aboutissant
> à exactement une écriture, le refus venant de la contrainte nommée.

### `synchronisation.reconciliation_orpheline` — le privilège élargi

```sql
GRANT INSERT ON synchronisation.reconciliation_orpheline TO kaya_app;
```

**Elle cesse d'être une provision.** Posée au cycle 005 avec `GRANT SELECT` **seul** pour prouver
qu'elle n'avait aucune logique, elle reçoit son premier écrivain : un accompagnant de classe A
arrivant après la clôture (R-10). Sa **résolution** reste SYN-03, tranche T3 — l'`UPDATE` n'est
**pas** accordé. Le décompte de `provisions_sans_logique.rs` passe de **six à cinq**.

---

## Migration `0032` — la note et ses lignes

### `hebergement.note_sejour`

```sql
CREATE TABLE hebergement.note_sejour (
    id                UUID        PRIMARY KEY,
    tenant_id         UUID        NOT NULL,
    sejour_id         UUID        NOT NULL UNIQUE REFERENCES hebergement.sejour (id),

    -- ISO 4217, au même niveau que les montants, toujours (principe V).
    devise            TEXT        NOT NULL CHECK (length(devise) = 3),

    statut            TEXT        NOT NULL DEFAULT 'ouverte'
        CONSTRAINT note_statut_connu CHECK (statut IN ('ouverte', 'arretee')),
    arretee_le        TIMESTAMPTZ NULL,

    cree_le           TIMESTAMPTZ NOT NULL DEFAULT now(),
    modifie_le        TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT note_arret_coherent
        CHECK ((statut = 'arretee') = (arretee_le IS NOT NULL))
);
```

**Aucune colonne de total.** Le total est **la somme des lignes**, calculée à la lecture. Une
colonne totalisatrice se désynchronise en silence — et le silence est exactement ce que le
propriétaire achète en installant ce logiciel.

### `hebergement.ligne_sejour`

```sql
CREATE TABLE hebergement.ligne_sejour (
    id                UUID        PRIMARY KEY,
    tenant_id         UUID        NOT NULL,
    note_id           UUID        NOT NULL REFERENCES hebergement.note_sejour (id),

    -- L'occupation d'où vient la ligne. NULL pour un ajustement qui n'en relève d'aucune.
    occupation_id     UUID        NULL REFERENCES hebergement.occupation (id),

    nature            TEXT        NOT NULL
        CONSTRAINT ligne_nature_connue CHECK (nature IN ('hebergement', 'ajustement')),

    -- Renseigné SEULEMENT sur un ajustement, et jamais deviné.
    motif             TEXT        NULL
        CONSTRAINT ligne_motif_connu CHECK (
            motif IS NULL OR motif IN (
                'rebascule_palier', 'depart_anticipe', 'prolongation', 'changement_unite'
            )),

    libelle_cle       TEXT        NOT NULL,             -- clé i18n, JAMAIS une chaîne rendue

    -- ⚠️ QUANTITÉ EN NUMERIC, JAMAIS ENTIER (principe V, P-10). Une nuitée est 1, une
    -- demi-journée 0.5, et un mois au prorata sera fractionnaire.
    quantite          NUMERIC(14, 4) NOT NULL CHECK (quantite <> 0),

    -- ⚠️ ENTIERS D'UNITÉ MINEURE (principe V, P-10). `montant_mineur` PEUT ÊTRE NÉGATIF :
    -- un départ anticipé rembourse. Le type `Rebascule` du cycle 004 le dit déjà.
    prix_unitaire_mineur BIGINT   NOT NULL,
    montant_mineur       BIGINT   NOT NULL,
    devise               TEXT     NOT NULL CHECK (length(devise) = 3),

    -- Période couverte par la ligne — c'est ce qui rend la note lisible nuit par nuit (R7).
    periode_debut     TIMESTAMPTZ NULL,
    periode_fin       TIMESTAMPTZ NULL,

    cree_le           TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT ligne_ajustement_motive
        CHECK ((nature = 'ajustement') = (motif IS NOT NULL))
);

CREATE INDEX ligne_sejour_par_note_idx ON hebergement.ligne_sejour (note_id, cree_le);
```

**`GRANT SELECT, INSERT` — pas d'`UPDATE`.** Le principe V pose que « les prix sont verrouillés à la
création de la ligne » ; le privilège le rend **impossible à contourner**. Une correction est une
**ligne d'ajustement** portant son motif (R-09), jamais une modification.

`libelle_cle` et non un libellé : la note s'affiche en `fr` **et** en `en` (P-16). Écrire
« Nuit du lun. 24 au mar. 25 » en base rendrait la note monolingue à jamais.

---

## Migration `0033` — la fiche de police et sa numérotation

### `hebergement.numerotation_fiche_police`

```sql
CREATE TABLE hebergement.numerotation_fiche_police (
    tenant_id        UUID   NOT NULL,
    etablissement_id UUID   NOT NULL,
    dernier_numero   BIGINT NOT NULL DEFAULT 0 CHECK (dernier_numero >= 0),
    PRIMARY KEY (tenant_id, etablissement_id)
);
```

**Un compteur, pas une `SEQUENCE`.** Une séquence PostgreSQL est **globale au schéma** et **laisse
des trous** ; les deux propriétés sont fatales à une numérotation de document opérationnel, qui doit
être continue **par établissement**. C'est le défaut corrigé par `0012` au cycle 002 — un espace de
numérotation d'outbox partagé entre tenants, trouvé par le premier événement appliqué à un second
tenant.

L'incrément se fait par `UPDATE … RETURNING dernier_numero` **dans la transaction du check-in**. Le
verrou de ligne est ce qui sérialise, et c'est la définition même de la classe **B**.

### `hebergement.fiche_police`

```sql
CREATE TABLE hebergement.fiche_police (
    id                UUID        PRIMARY KEY,
    tenant_id         UUID        NOT NULL,
    etablissement_id  UUID        NOT NULL,
    sejour_id         UUID        NOT NULL UNIQUE REFERENCES hebergement.sejour (id),

    numero            BIGINT      NOT NULL,
    CONSTRAINT fiche_police_numero_unique UNIQUE (tenant_id, etablissement_id, numero),

    -- FR-047 : une fiche dont l'identité est incomplète est IDENTIFIÉE comme telle. Elle n'est
    -- ni fabriquée avec des valeurs de remplissage, ni silencieusement omise.
    complete          BOOLEAN     NOT NULL DEFAULT false,

    generee_le        TIMESTAMPTZ NOT NULL DEFAULT now(),
    completee_le      TIMESTAMPTZ NULL,

    CONSTRAINT fiche_police_completude_coherente
        CHECK (complete = (completee_le IS NOT NULL))
);
```

**Le contenu n'est pas dupliqué ici.** La fiche référence le séjour ; les identités viennent du
client (par `AnnuaireClients`) et des accompagnants. Recopier nom, prénoms et numéro de pièce
créerait une **troisième** surface de rétention pour la même donnée sensible — exactement ce que
`provisions_sans_logique.rs` refuse ailleurs.

**Le gabarit officiel n'est pas inventé** (décision Q3, option (a)) : le registre minimal est en
base, le formulaire du pilote est un **rendu** qui s'ajoutera sans migration.

**`UPDATE` est accordé** — uniquement pour passer `complete` à vrai quand l'identité est saisie
après la clé, ce que le parcours de passage impose (FR-023, FR-028).

---

## Migration `0034` — le constat de taxe, figé par privilège

```sql
CREATE TABLE hebergement.taxe_sejour_constat (
    id                       UUID        PRIMARY KEY,
    tenant_id                UUID        NOT NULL,
    etablissement_id         UUID        NOT NULL,
    sejour_id                UUID        NOT NULL UNIQUE REFERENCES hebergement.sejour (id),

    -- ═══ LES FAITS — arithmétique, aucune règle fiscale ═══
    nuits_constatees         INTEGER     NOT NULL CHECK (nuits_constatees >= 0),
    nombre_personnes         INTEGER     NOT NULL CHECK (nombre_personnes >= 1),
    periode_debut            TIMESTAMPTZ NOT NULL,
    periode_fin              TIMESTAMPTZ NOT NULL,

    -- ═══ LE PARAMÉTRAGE, RECOPIÉ — c'est ce qui rend le figeage vrai ═══
    --
    -- Recopié, jamais référencé : une formule éditée demain, un classement changé, une commune
    -- redécoupée ne doivent RIEN changer à un séjour clos hier (FR-063, SC-007).
    formule_id               UUID        NOT NULL,
    famille_formule          TEXT        NOT NULL,
    assujettie_taxe_nuitee   BOOLEAN     NOT NULL,
    regle_conversion_taxe    TEXT        NULL,
    classement_etablissement TEXT        NOT NULL,
    commune                  TEXT        NOT NULL,

    -- ═══ POSÉES, JAMAIS ALIMENTÉES PAR CE CYCLE (principe X) ═══
    --
    -- Décider quelles nuits sont assujetties est une RÈGLE FISCALE : elle ne vit que dans
    -- `JurisdictionAdapter` (principe V, porte P-12), et son test doré appartient à FIS-03.
    -- `provisions_sans_logique.rs` vérifie qu'aucun chemin de code de ce cycle ne les écrit et
    -- qu'aucune opération du contrat ne les expose.
    nuitees_assujetties      INTEGER     NULL,
    montant_mineur           BIGINT      NULL,
    devise                   TEXT        NULL CHECK (devise IS NULL OR length(devise) = 3),

    -- Horodatage d'AUTORITÉ du figeage.
    fige_le                  TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT constat_periode_coherente CHECK (periode_fin > periode_debut),
    -- Une formule assujettie sans règle serait un état d'attente ; la contrainte du cycle 004
    -- l'interdit déjà sur `formule`, celle-ci le tient sur la copie.
    CONSTRAINT constat_regle_coherente
        CHECK (assujettie_taxe_nuitee = false OR regle_conversion_taxe IS NOT NULL)
);
```

```sql
-- ★ LE FIGEAGE EST UN PRIVILÈGE, PAS UNE INTENTION.
GRANT SELECT, INSERT ON hebergement.taxe_sejour_constat TO kaya_app;
```

> **Ni `UPDATE`, ni `DELETE`.** Le rôle applicatif **ne peut pas** modifier un constat, quelle que
> soit la ligne de code écrite au-dessus. C'est ce qui transforme SC-007 — « l'assiette est immuable
> après le départ » — d'une promesse en une propriété de la base. Une garantie de privilège se perd
> en une ligne de migration : `sejour_depart.rs` l'asserte tout de même, en tentant l'`UPDATE`.

**Un constat par séjour, pas par occupation** (FR-083) : la contrainte `UNIQUE` sur `sejour_id` le
porte. Un séjour à deux chambres produit **un** constat.

---

## Événements outbox

| Type | Agrégat | `version_schema` | Charge utile |
|---|---|---|---|
| `sej.client.cree` | `comptes.client` | 1 | Identifiants, **jamais** le numéro de pièce |
| `sej.client.modifie` | `comptes.client` | 1 | Champs modifiés, **jamais** le numéro de pièce |
| `sej.preference.enregistree` | `comptes.preference_personne` | 1 | Personne, texte, instant d'autorité |
| `sej.accompagnant.ajoute` | `hebergement.accompagnant` | 1 | Séjour, nom, **jamais** le numéro de pièce |
| `heb.sejour.ouvert` | `hebergement.sejour` | 1 | Séjour, unité, formule, période, personnes, **ligne d'hébergement complète** (`montant_mineur`, `devise`) |
| `heb.sejour.prolonge` | `hebergement.sejour` | 1 | Période étendue, lignes ajoutées, montants |
| `heb.sejour.unite_changee` | `hebergement.sejour` | 1 | Les deux unités, l'instant, les deux périodes, les lignes |
| `heb.sejour.clos` | `hebergement.sejour` | 1 | **Total, toutes les lignes, les ajustements et le constat de taxe** — l'opération se reconstitue sans consulter aucune autre table (TRX-02) |
| `heb.fiche_police.generee` | `hebergement.fiche_police` | 1 | Séjour, numéro, complétude |

> ⚠️ **Aucun numéro de pièce d'identité ne part dans l'outbox**, et c'est une décision, pas un
> oubli. L'outbox est un **grand livre à rétention illimitée et immuable** (principe II, P-05b) :
> une donnée sensible qui y entre ne peut **jamais** en sortir, et la rétention de 90 jours de
> TRX-06 deviendrait inapplicable. Le contrôle est explicite dans `outbox_transactionnel.rs`.

> **P-10 franchit la frontière du JSONB.** Toute clé monétaire des charges utiles suit le nommage
> réservé `<nom>_mineur` et porte un **entier**, jamais un décimal ni une chaîne formatée.
> `scripts/ci/types-monetaires.sh` l'inspecte jusque dans le JSON.

---

## Registre des actions — ce qui y entre

| Geste | Famille | Contexte écrit |
|---|---|---|
| Rebascule de palier au départ | `rebascule_palier_passage` *(branchée au cycle 004)* | Durée constatée, les deux paliers, la différence |
| Régularisation de départ anticipé | `rebascule_palier_passage` | Durée prévue, durée réelle, différence, motif |
| Changement d'unité | *(à trancher — voir ci-dessous)* | Les deux unités, l'instant |
| **Consultation d'un numéro de pièce d'identité** | **famille NOUVELLE, à inscrire à `docs/taxonomie-audit.md`** | Personne consultée, compte auteur, instant d'autorité — **jamais la valeur lue** |

> **Pourquoi une famille nouvelle plutôt qu'une existante.** FR-012 exige un **journal d'accès** à la
> pièce d'identité (principe IX, cadrage §12.1). Aucune des onze familles ne couvre une
> **consultation** : `suppression` trace une mise hors service, `changement_role` une attribution —
> toutes tracent un **geste qui modifie**. Une lecture de donnée sensible n'en est pas un, et la
> ranger sous une famille existante rendrait le registre illisible au propriétaire, qui est son
> public. La famille naît « **branchée** » — elle a son chemin de code dès T018a.
>
> ⚠️ **Le contexte écrit ne porte jamais la valeur lue.** Journaliser l'accès à un numéro de pièce
> en recopiant le numéro dans le registre des actions — qui est **immuable et à rétention
> illimitée** (P-05b) — créerait la fuite que le journal existe pour surveiller.

> ⚠️ **La famille `forcage_disponibilite` (n° 10) reste « due ».** La taxonomie la décrit comme
> *« l'attribution d'une unité que le système déclarait indisponible »*. **Ce cycle ne livre aucun
> forçage** : un changement d'unité vers une chambre occupée est **refusé**, avec le conflit nommé
> (FR-080). La ligne reste donc « due », et le dire vaut mieux que la laisser ambiguë — une famille
> déclarée « branchée » sans chemin de code ferait échouer le harnais de la taxonomie.

---

## Classes hors-ligne — ce que le registre doit dire

| Entité | Classe | Branche | Statut au registre |
|---|---|---|---|
| `client` | **C** | C2 — partagé entre établissements du tenant | **Déjà déclarée** (§8) — honorée |
| `preference_personne` | **A** | A4 — append-only, commutative, sans effet monétaire | **Ligne à ajouter** — le registre disait « `client.preferences` », sans nommer de table |
| `sejour` | **B** | B3 — ressource unique | **Déjà déclarée** — honorée |
| `accompagnant` | **A** | A4 | **Déjà déclarée** — honorée |
| `note_sejour` | **B** | B3 — effet monétaire | **Ligne à ajouter** — le registre nommait `ligne_sejour`, pas la note |
| `ligne_sejour` | **B** | B3 | **Déjà déclarée** — honorée pour son sous-ensemble hébergement |
| `fiche_police` | **B** | B3 — dérivée du check-in, numérotée | **Déjà déclarée** — honorée |
| `numerotation_fiche_police` | **B** | B3 — numérotation | **Ligne à ajouter** |
| `taxe_sejour_constat` | **B** | B3 — clôt la note | **Ligne à ajouter** — le registre parlait de « `sejour` — check-out, taxe figée » |

**Quatre lignes ajoutées, cinq honorées.** C'est le régime établi : le registre déclare des
entités, les cycles nomment les tables. Précédent exact du cycle 004 (`temps_remise_en_etat`,
`plage_demi_journee`) et du cycle 003 (`methode_authentification`, `role_permission`).

**Instanciations dues** dans `backend/tests/` :

```rust
tester_classe_a!(accompagnant,        schema = "hebergement", table = "accompagnant",        …);
tester_classe_a!(preference_personne, schema = "comptes",     table = "preference_personne", …);
tester_classe_bcd!(sejour,            classe = B, …);
tester_classe_bcd!(client,            classe = C, …);
// … et une par entité B restante
```

`outillage_classes.rs` **échoue en nommant** l'entité qui aurait une table sans instanciation.

---

## Seeds — ce qui est peuplé, et ce qui ne l'est pas

| Jeu | Contenu | Où |
|---|---|---|
| **Démonstration Deloria** | 12 fiches clients, 3 séjours : nuitée en cours (2 nuits, 2 accompagnants), passage en cours (2 h), séjour clos avec son constat figé | `migrations/seeds/` |
| **Résidence Test** | 2 fiches clients, 1 séjour en cours | `migrations/seeds/` |
| **Jeu de mesure** | **10 000 fiches**, tenant dédié | ⚠️ **Généré par `client_recherche.rs`**, jamais par les seeds (FR-007) |

**Les seeds ne passent jamais par une migration** : une table en `FORCE ROW LEVEL SECURITY` accepte
un `INSERT` de migration **en n'écrivant rien**, sans erreur — constaté au cycle 001, et le genre
de défaut qui ne se voit qu'à la démonstration.

**Le séjour clos des seeds est ce qui rend la démo de fin de T1 vérifiable** : il porte un constat
de taxe figé, donc `nuitees_assujetties` et `montant_mineur` à `NULL` — la preuve visible que ce
cycle a bien laissé le calcul à FIS.
