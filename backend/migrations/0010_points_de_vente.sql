-- 0010 — ETB-03 : points de vente et tables.
--
-- **Un comptoir est un point de vente SANS AUCUNE LIGNE dans `table_pdv`.**
--
-- Pas de drapeau `est_comptoir`. Un drapeau peut contredire les données — rien n'empêcherait
-- `est_comptoir = true` sur un point de vente portant douze tables, et il faudrait alors décider
-- lequel des deux ment. Une absence, non. C'est la **forme normale** d'un maquis, pas un cas
-- dégradé (FR-040).

-- =============================================================================================
--  point_de_vente
-- =============================================================================================
CREATE TABLE etablissements.point_de_vente (
    id                      UUID        PRIMARY KEY,
    tenant_id               UUID        NOT NULL REFERENCES etablissements.tenant (id),

    -- Dénormalisé depuis `etablissement_module` pour l'isolation et la résolution de
    -- configuration : sans lui, chaque descente de chaîne devrait remonter au service pour
    -- retrouver l'établissement.
    etablissement_id        UUID        NOT NULL REFERENCES etablissements.etablissement (id),

    -- **C'est cette clé étrangère qui tient FR-041.** Un point de vente ne peut pas se rattacher
    -- à un service non activé : la seule cible possible est une activation existante. Le `422`
    -- `module_non_actif` du contrat HTTP donne le message ; la contrainte, elle, rend le cas
    -- structurellement impossible — y compris pour un import.
    etablissement_module_id UUID        NOT NULL
        REFERENCES etablissements.etablissement_module (id),

    nom                     TEXT        NOT NULL
                            CONSTRAINT point_de_vente_nom_non_vide
                                CHECK (length(btrim(nom)) > 0),

    -- **AUCUNE clé étrangère.** `socle/caisse` est un autre module, et une clé étrangère d'ici
    -- vers lui joindrait deux schémas de modules — ce que le principe II interdit. Ce n'est pas
    -- parce que la table n'existe pas encore : même quand elle existera, l'intégrité
    -- référentielle inter-modules passera par un trait exposé (research.md R-12). Le cycle CAI
    -- ajoutera la vérification, côté service.
    --
    -- Même traitement que `auteur_compte_id` du module doré.
    caisse_id               UUID            NULL,

    actif                   BOOLEAN     NOT NULL DEFAULT true,
    cree_le                 TIMESTAMPTZ NOT NULL DEFAULT now(),
    modifie_le              TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- Deux points de vente homonymes seraient indiscernables sur un ticket — et c'est le ticket
    -- que le client réclame quand il conteste une addition.
    UNIQUE (etablissement_id, nom)
);

CREATE INDEX point_de_vente_lecture_idx
    ON etablissements.point_de_vente (tenant_id, etablissement_id, actif);

COMMENT ON TABLE etablissements.point_de_vente IS
    'Point de vente. Classe hors-ligne C. Sans ligne dans table_pdv, c''est un comptoir — aucun drapeau.';
COMMENT ON COLUMN etablissements.point_de_vente.caisse_id IS
    'Rattachement de caisse. AUCUNE clé étrangère : frontière de module (principe II). Vérification par trait au cycle CAI.';

-- **La politique d'impression n'est PAS une colonne ici.** C'est un paramètre de la chaîne
-- d'héritage, au niveau point de vente (research.md R-04), conformément au principe I(c) : tout
-- paramètre d'exploitation figure au récapitulatif et se résout par héritage. Une colonne
-- l'aurait rendu invisible au récapitulatif et non surchargeable.

-- =============================================================================================
--  table_pdv
-- =============================================================================================
CREATE TABLE etablissements.table_pdv (
    id                UUID        PRIMARY KEY,
    tenant_id         UUID        NOT NULL REFERENCES etablissements.tenant (id),
    point_de_vente_id UUID        NOT NULL REFERENCES etablissements.point_de_vente (id),

    -- « 12 », « Terrasse 3 » — libellé libre, tel que le personnel le dit.
    libelle           TEXT        NOT NULL
                      CONSTRAINT table_pdv_libelle_non_vide
                          CHECK (length(btrim(libelle)) > 0),

    actif             BOOLEAN     NOT NULL DEFAULT true,
    cree_le           TIMESTAMPTZ NOT NULL DEFAULT now(),

    UNIQUE (point_de_vente_id, libelle)
);

CREATE INDEX table_pdv_lecture_idx
    ON etablissements.table_pdv (tenant_id, point_de_vente_id, actif);

COMMENT ON TABLE etablissements.table_pdv IS
    'Table d''un point de vente. Classe hors-ligne C. Zéro ligne pour un point de vente = comptoir.';

-- =============================================================================================
--  Sécurité au niveau ligne
-- =============================================================================================
ALTER TABLE etablissements.point_de_vente ENABLE ROW LEVEL SECURITY;
ALTER TABLE etablissements.point_de_vente FORCE  ROW LEVEL SECURITY;
ALTER TABLE etablissements.table_pdv      ENABLE ROW LEVEL SECURITY;
ALTER TABLE etablissements.table_pdv      FORCE  ROW LEVEL SECURITY;

CREATE POLICY isolation_tenant ON etablissements.point_de_vente
    USING      (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

CREATE POLICY isolation_tenant ON etablissements.table_pdv
    USING      (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

-- Pas de `DELETE` : un point de vente se désactive. Une table retirée du plan de salle passe
-- `actif = false` — les commandes passées qui la référencent doivent rester lisibles.
GRANT SELECT, INSERT, UPDATE ON etablissements.point_de_vente TO kaya_app;
GRANT SELECT, INSERT, UPDATE ON etablissements.table_pdv      TO kaya_app;
