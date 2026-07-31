-- 0006 — Provisions comptables (TRX-02b).
--
-- **Deux tables. Aucun endpoint, aucun écran, aucune règle métier.** C'est le principe X — « prêt
-- ≠ construit » — dans sa forme la plus littérale : les provisions du cadrage §14 sont des choix
-- de modèle de données et d'interfaces, jamais des fonctionnalités.
--
-- `backend/tests/provisions_sans_logique.rs` vérifie **l'absence** de tout endpoint et de tout
-- service les consommant. C'est un test dont l'objet est de constater qu'on n'a rien construit —
-- et c'est le seul moyen d'empêcher qu'un cycle ultérieur « ajoute juste un petit endpoint de
-- lecture ».
--
-- **Écart de numérotation consigné** : `data-model.md` §7 et `tasks.md` T064 attribuaient 0005 à
-- ce fichier. La migration `0005_role_worker_publication.sql` s'est intercalée — la Phase 4
-- précède la Phase 10, et sqlx refuse une version antérieure à une version déjà appliquée. Le
-- contenu ci-dessous est celui de `data-model.md` §5, inchangé.

-- =============================================================================================
--  exercice_comptable — PREMIER USAGE D'EXCLUDE USING gist DU PRODUIT
-- =============================================================================================
--
-- La contrainte d'exclusion n'est pas décorative : deux exercices qui se chevauchent rendraient
-- « la période est-elle close ? » indécidable, et c'est la seule règle que TRX-02b impose
-- (FR-046).
--
-- Elle est posée **maintenant**, alors qu'aucune ligne n'existe, parce qu'une contrainte
-- d'exclusion ajoutée sur une table déjà peuplée échoue sur les données existantes — et qu'à ce
-- moment-là, il faudrait choisir entre corriger l'historique et renoncer à la contrainte.
--
-- **C'est aussi le spike de HEB-02.** La disponibilité des unités louables reprendra exactement
-- cette forme sur `tstzrange`. L'exercer ici, sur un cas sans enjeu, valide `btree_gist` et le
-- mapping de type sqlx 0.9 avant que la disponibilité en dépende.

CREATE TABLE fiscalite.exercice_comptable (
    id          UUID        PRIMARY KEY,
    tenant_id   UUID        NOT NULL REFERENCES etablissements.tenant (id),
    debut       DATE        NOT NULL,
    fin         DATE        NOT NULL,
    statut      TEXT        NOT NULL CHECK (statut IN ('ouvert', 'clos')),

    CONSTRAINT exercice_comptable_bornes CHECK (fin > debut),

    -- `tenant_id WITH =` exige `btree_gist` : GiST ne sait pas indexer l'égalité sur un UUID sans
    -- l'extension. Elle est installée par la migration 0001.
    --
    -- `'[)'` — borne de début incluse, borne de fin exclue. Le même choix qu'imposera le principe
    -- IV aux occupations : deux exercices contigus ne se chevauchent pas, alors qu'avec `'[]'` le
    -- 31 décembre appartiendrait à deux exercices.
    CONSTRAINT exercice_comptable_sans_chevauchement
        EXCLUDE USING gist (
            tenant_id WITH =,
            daterange(debut, fin, '[)') WITH &&
        )
);

COMMENT ON TABLE fiscalite.exercice_comptable IS
    'Provision comptable (TRX-02b). Table seulement — aucun endpoint, aucune logique. Classe C.';

-- =============================================================================================
--  mapping_comptable
-- =============================================================================================
CREATE TABLE fiscalite.mapping_comptable (
    id              UUID    PRIMARY KEY,
    tenant_id       UUID    NOT NULL REFERENCES etablissements.tenant (id),
    -- Associe un type d'événement du grand livre à sa traduction comptable. C'est ce qui rendra
    -- possible la génération SYSCOHADA **rétroactive** de la provision §14.7 : les événements
    -- sont déjà écrits, la correspondance viendra après.
    type_evenement  TEXT    NOT NULL,
    compte_debit    TEXT    NOT NULL,
    compte_credit   TEXT    NOT NULL,
    journal         TEXT    NOT NULL,

    CONSTRAINT mapping_comptable_unique UNIQUE (tenant_id, type_evenement)
);

COMMENT ON TABLE fiscalite.mapping_comptable IS
    'Provision comptable (TRX-02b). Table seulement — aucun endpoint, aucune logique. Classe C.';

-- =============================================================================================
--  Refus d'écriture sur période close — DÉCLENCHEUR, jamais une règle applicative
-- =============================================================================================
--
-- Une règle applicative serait contournée par la première migration de données venue, et la
-- provision perdrait exactement le sens qu'on lui donne : une période close ne bouge plus.
--
-- Le déclencheur porte sur `exercice_comptable` lui-même — c'est la seule table qu'il peut
-- protéger aujourd'hui. Les écritures comptables n'existent pas encore ; quand elles viendront,
-- elles porteront le même déclencheur, adossé à la même fonction.

CREATE FUNCTION fiscalite.refuser_ecriture_periode_close()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
    -- Une période close ne se rouvre pas et ne se modifie pas. La seule transition permise est
    -- 'ouvert' vers 'clos'.
    IF TG_OP = 'UPDATE' AND OLD.statut = 'clos' THEN
        RAISE EXCEPTION
            'exercice comptable clos : aucune modification possible (exercice %, du % au %)',
            OLD.id, OLD.debut, OLD.fin
            USING ERRCODE = 'restrict_violation';
    END IF;

    IF TG_OP = 'DELETE' AND OLD.statut = 'clos' THEN
        RAISE EXCEPTION
            'exercice comptable clos : suppression interdite (exercice %)', OLD.id
            USING ERRCODE = 'restrict_violation';
    END IF;

    RETURN CASE TG_OP WHEN 'DELETE' THEN OLD ELSE NEW END;
END;
$$;

CREATE TRIGGER exercice_comptable_periode_close
    BEFORE UPDATE OR DELETE ON fiscalite.exercice_comptable
    FOR EACH ROW EXECUTE FUNCTION fiscalite.refuser_ecriture_periode_close();

-- =============================================================================================
--  Isolation multi-tenant — patron identique, sans exception
-- =============================================================================================
ALTER TABLE fiscalite.exercice_comptable ENABLE ROW LEVEL SECURITY;
ALTER TABLE fiscalite.exercice_comptable FORCE  ROW LEVEL SECURITY;
ALTER TABLE fiscalite.mapping_comptable  ENABLE ROW LEVEL SECURITY;
ALTER TABLE fiscalite.mapping_comptable  FORCE  ROW LEVEL SECURITY;

CREATE POLICY isolation_tenant ON fiscalite.exercice_comptable
    USING      (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

CREATE POLICY isolation_tenant ON fiscalite.mapping_comptable
    USING      (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

-- `SELECT` seulement pour le rôle applicatif : **aucun chemin d'écriture n'existe**, et il ne
-- doit pas pouvoir en naître un par inadvertance. Le jour où la comptabilité sera implémentée,
-- une migration accordera les droits qui manquent — un acte visible, daté, et revu.
GRANT SELECT ON fiscalite.exercice_comptable TO kaya_app;
GRANT SELECT ON fiscalite.mapping_comptable  TO kaya_app;
