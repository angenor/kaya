-- 0009 — ETB-02 / ETB-02b : activation d'un service, déclaration de ce qu'il consomme.
--
-- Deux tables, et **une seule idée** : ce qu'un établissement ne fait pas ne doit exister nulle
-- part chez lui. Un module s'active, se désactive, et sa désactivation ne supprime rien.
--
-- =============================================================================================
--  LA CLÉ ÉTRANGÈRE COMPOSITE — ce qui rend le refus STRUCTUREL et non applicatif
-- =============================================================================================
--
-- Le référentiel porte `implementee` ; la table qui le référence **recopie** la colonne et exige
-- par `CHECK` qu'elle soit vraie. Activer un module non implémenté devient alors impossible : la
-- seule ligne de référentiel portant son code a `implementee = false`, et le `CHECK` refuse.
--
-- La dénormalisation est **assumée** (plan.md, Complexity Tracking). Les alternatives ont été
-- écartées pour des raisons précises :
--
--   déclencheur `BEFORE INSERT`  du code caché dans la base, invisible en lecture de schéma, à
--                                maintenir en parallèle du référentiel ;
--   `CHECK (code = 'STOCK')`     ferait de l'ouverture d'une capacité une MIGRATION, alors que le
--                                cadrage §14.4 en fait une écriture de configuration ;
--   validation applicative seule contournée par tout import, seed ou script de reprise.
--
-- Le jour où une capacité est implémentée, un `UPDATE` du référentiel l'ouvre — sans migration.

-- =============================================================================================
--  etablissement_module — l'activation
-- =============================================================================================
CREATE TABLE etablissements.etablissement_module (
    -- UUID v7 **fourni par le client** (cadrage §11.5.1) : c'est ce qui rend le rejeu inoffensif.
    id                UUID        PRIMARY KEY,

    tenant_id         UUID        NOT NULL REFERENCES etablissements.tenant (id),
    etablissement_id  UUID        NOT NULL REFERENCES etablissements.etablissement (id),

    module_code       TEXT        NOT NULL,
    -- Recopie du référentiel — support de la clé étrangère composite ci-dessous.
    module_implemente BOOLEAN     NOT NULL,

    -- **La désactivation ne supprime rien.** Elle bascule ce drapeau, ce qui rend l'état antérieur
    -- restituable à la réactivation (FR-015) : déclarations de capacité et surcharges de
    -- configuration redeviennent actives sans avoir jamais été touchées.
    actif             BOOLEAN     NOT NULL DEFAULT true,

    -- Horodatages d'**autorité serveur** (principe IV). Jamais l'horloge d'un terminal.
    active_le         TIMESTAMPTZ NOT NULL DEFAULT now(),
    desactive_le      TIMESTAMPTZ     NULL,

    cree_le           TIMESTAMPTZ NOT NULL DEFAULT now(),
    modifie_le        TIMESTAMPTZ NOT NULL DEFAULT now(),

    FOREIGN KEY (module_code, module_implemente)
        REFERENCES etablissements.module_activite (code, implementee),

    CONSTRAINT etablissement_module_implemente
        CHECK (module_implemente),

    -- **Un module s'active une fois par établissement.** Une réactivation est un
    -- `UPDATE actif = true`, jamais une seconde ligne — c'est ce qui fait que l'état antérieur est
    -- restitué plutôt que recréé.
    UNIQUE (etablissement_id, module_code)
);

CREATE INDEX etablissement_module_lecture_idx
    ON etablissements.etablissement_module (tenant_id, etablissement_id, actif);

COMMENT ON TABLE etablissements.etablissement_module IS
    'Activation d''un module d''activité sur un établissement. Classe hors-ligne C. La désactivation ne supprime rien.';
COMMENT ON COLUMN etablissements.etablissement_module.module_implemente IS
    'Recopie de module_activite.implementee — support de la clé étrangère composite. Dénormalisation assumée : c''est elle qui rend le refus structurel.';

-- =============================================================================================
--  module_capacite — ce que le service consomme
-- =============================================================================================
--
-- **La déclaration appartient au SERVICE, pas à l'établissement.** C'est le module qui déclare ce
-- dont il a besoin : chez Deloria, `RESTAURATION` et `BAR` suivent leur stock, `HEBERGEMENT` non.
-- Rattacher la déclaration à l'établissement obligerait à inventer une règle pour dire quels
-- services y ont droit.
CREATE TABLE etablissements.module_capacite (
    id                      UUID        PRIMARY KEY,
    tenant_id               UUID        NOT NULL REFERENCES etablissements.tenant (id),

    etablissement_module_id UUID        NOT NULL
        REFERENCES etablissements.etablissement_module (id),

    capacite_code           TEXT        NOT NULL,
    capacite_implementee    BOOLEAN     NOT NULL,

    -- **`NOT NULL` au MVP** : seule `STOCK` est déclarable, et elle exige un profil. Le jour où
    -- une capacité sans profil sera implémentée, une migration additive rendra la colonne
    -- nullable avec la règle correspondante. Poser aujourd'hui un
    -- `CHECK ((capacite_code = 'STOCK') = (profil_code IS NOT NULL))` réintroduirait en base la
    -- valeur en dur que le référentiel existe précisément pour éviter.
    profil_code             TEXT        NOT NULL,
    profil_implemente       BOOLEAN     NOT NULL,

    declaree_le             TIMESTAMPTZ NOT NULL DEFAULT now(),

    FOREIGN KEY (capacite_code, capacite_implementee)
        REFERENCES etablissements.capacite (code, implementee),
    FOREIGN KEY (profil_code, profil_implemente)
        REFERENCES etablissements.profil_stock (code, implementee),

    CONSTRAINT module_capacite_capacite_implementee CHECK (capacite_implementee),
    CONSTRAINT module_capacite_profil_implemente    CHECK (profil_implemente),

    UNIQUE (etablissement_module_id, capacite_code)
);

CREATE INDEX module_capacite_lecture_idx
    ON etablissements.module_capacite (tenant_id, etablissement_module_id);

COMMENT ON TABLE etablissements.module_capacite IS
    'Capacité transverse déclarée par un service. Classe hors-ligne C. Aucune colonne d''état : la désactivation du service la rend inerte sans la toucher.';

-- **Aucune colonne d'état sur cette table** (FR-037). La désactivation d'un service rend ses
-- déclarations inertes en les lisant à travers `etablissement_module.actif` — donc aucune
-- écriture à la désactivation, donc aucune perte à la réactivation.

-- =============================================================================================
--  Sécurité au niveau ligne — le patron unique
-- =============================================================================================
ALTER TABLE etablissements.etablissement_module ENABLE ROW LEVEL SECURITY;
ALTER TABLE etablissements.etablissement_module FORCE  ROW LEVEL SECURITY;
ALTER TABLE etablissements.module_capacite      ENABLE ROW LEVEL SECURITY;
ALTER TABLE etablissements.module_capacite      FORCE  ROW LEVEL SECURITY;

CREATE POLICY isolation_tenant ON etablissements.etablissement_module
    USING      (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

CREATE POLICY isolation_tenant ON etablissements.module_capacite
    USING      (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

-- **Pas de `DELETE` — le privilège dit la règle mieux qu'un commentaire.** On ne supprime pas
-- l'activation d'un service : on la désactive, et l'historique du journal d'événements garde la
-- trace des deux transitions.
GRANT SELECT, INSERT, UPDATE ON etablissements.etablissement_module TO kaya_app;
GRANT SELECT, INSERT, UPDATE ON etablissements.module_capacite      TO kaya_app;
