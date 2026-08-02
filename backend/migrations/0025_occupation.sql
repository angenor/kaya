-- 0025 — HEB-02 : **la table du cycle**, et la seule migration du projet dont une erreur ne se
-- rattrape pas.
--
-- ═══════════════════════════════════════════════════════════════════════════════════════════════
--  LE CŒUR TIENT EN CINQ LIGNES
--
--      CONSTRAINT occupation_sans_chevauchement
--          EXCLUDE USING gist (
--              unite_id WITH =,
--              periode  WITH &&
--          )
--
--  Elle rend la double attribution **IMPOSSIBLE**, là où un verrou applicatif la rendrait
--  seulement improbable. Tout le reste de ce fichier sert cette ligne ou en découle.
-- ═══════════════════════════════════════════════════════════════════════════════════════════════
--
-- **La contrainte se pose À LA CRÉATION, jamais après.** Ajoutée sur une table déjà peuplée, elle
-- échoue sur les données existantes — et il faudrait alors choisir entre corriger l'historique et
-- renoncer à la garantie. C'est la raison pour laquelle aucun seed n'entre avant cette migration.
--
-- `unite_id WITH =` exige l'extension `btree_gist` : GiST ne sait pas indexer l'égalité sur un
-- UUID sans elle. Elle est installée par `0001_roles_et_schemas.sql:93`, au cycle 001, **pour ce
-- moment précis**.

CREATE TABLE hebergement.occupation (
    id               UUID        PRIMARY KEY,
    tenant_id        UUID        NOT NULL,
    etablissement_id UUID        NOT NULL,

    unite_id         UUID        NOT NULL REFERENCES hebergement.unite (id),
    formule_id       UUID        NOT NULL REFERENCES hebergement.formule (id),

    -- ═══ LA colonne que la contrainte protège ═══
    --
    -- **Un intervalle, jamais une paire de dates.** Le marché pratique massivement le passage
    -- horaire et la demi-journée : une paire `(date_arrivee, date_depart)` ne sait pas dire
    -- « 14 h → 18 h le même jour », et le premier code qui essaierait ajouterait une colonne
    -- d'heure à côté — deux sources pour un même fait.
    --
    -- **Remise en état COMPRISE.** La période d'indisponibilité va du début client à la fin
    -- client + le battement de ménage. C'est ce qui fait que la remise en état bloque la
    -- réservation suivante **par la même contrainte** que tout chevauchement, et non par une
    -- règle à part qu'il faudrait penser à appliquer partout.
    periode          TSTZRANGE   NOT NULL,

    -- Les bornes **commerciales** — ce que le client connaît, et sur quoi la note se calcule.
    -- Distinctes de `periode` : le client ne paie pas le ménage.
    debut_client     TIMESTAMPTZ NOT NULL,
    fin_client       TIMESTAMPTZ NOT NULL,

    statut           TEXT        NOT NULL DEFAULT 'active'
        CONSTRAINT occupation_statut_connu CHECK (statut IN ('active', 'liberee')),

    -- Horodatage d'**autorité** de la libération.
    libere_le        TIMESTAMPTZ NULL,

    -- **Horodatage d'autorité serveur** — jamais l'horloge d'un terminal. C'est lui que le calcul
    -- de durée d'un passage lit (HEB-04, FR-029).
    cree_le          TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- ═════════════════════════════════════════════════════════════════════════════════════════
    --  LA contrainte du cycle
    -- ═════════════════════════════════════════════════════════════════════════════════════════
    CONSTRAINT occupation_sans_chevauchement
        EXCLUDE USING gist (
            unite_id WITH =,
            periode  WITH &&
        ),

    -- Le **SEUL contournement possible** de la contrainte ci-dessus.
    --
    -- `&&` est FAUX dès qu'un intervalle est vide : une ligne `[14h, 14h)` passerait l'exclusion
    -- **et** n'empêcherait aucune autre occupation. Une ligne fantôme qui occupe sans bloquer —
    -- la chambre apparaîtrait prise dans la liste et libre à l'attribution.
    CONSTRAINT occupation_periode_non_vide
        CHECK (NOT isempty(periode)),

    -- Verrouille la forme `[)`. Avec `[]`, deux occupations contiguës deviendraient chevauchantes,
    -- et le comportement du produit changerait selon la forme employée par l'appelant — une
    -- chambre libérée à midi ne serait pas attribuable à midi.
    CONSTRAINT occupation_periode_semi_ouverte
        CHECK (lower_inc(periode) AND NOT upper_inc(periode)),

    -- Les bornes commerciales sont **dans** la période d'indisponibilité, jamais hors d'elle.
    -- La remise en état ALLONGE la période ; elle ne la déplace pas.
    CONSTRAINT occupation_bornes_client_coherentes
        CHECK (fin_client > debut_client
               AND lower(periode) <= debut_client
               AND upper(periode) >= fin_client),

    -- Une occupation libérée porte son horodatage, une active n'en porte pas. L'égalité de deux
    -- booléens plutôt que deux `CHECK` séparés : elle interdit les **deux** incohérences, dont
    -- celle qu'on oublie — un `libere_le` posé sur une occupation encore active.
    CONSTRAINT occupation_liberation_coherente
        CHECK ((statut = 'liberee') = (libere_le IS NOT NULL))
);

COMMENT ON TABLE hebergement.occupation IS
    'L''attribution d''une unité sur un intervalle. Classe hors-ligne B. La double attribution est IMPOSSIBLE — contrainte d''exclusion GiST, jamais un verrou applicatif (principe IV).';
COMMENT ON COLUMN hebergement.occupation.periode IS
    'Période d''INDISPONIBILITÉ — remise en état comprise. Distincte des bornes commerciales : le client ne paie pas le ménage.';
COMMENT ON COLUMN hebergement.occupation.cree_le IS
    'Horodatage d''AUTORITÉ SERVEUR. Le calcul de durée d''un passage le lit ; jamais l''horloge d''un terminal.';

-- =============================================================================================
--  Aucun index supplémentaire — et c'est une décision
-- =============================================================================================
--
-- La contrainte d'exclusion crée son propre index GiST sur `(unite_id, periode)`, qui sert
-- **exactement** la requête la plus fréquente du produit : chercher les occupations d'une unité
-- qui chevauchent un intervalle.
--
-- Un index B-tree sur `(unite_id, cree_le)` serait ajouté sans mesure ; le principe X l'interdit
-- tant qu'aucun besoin ne l'appelle. Écrit ici pour qu'une relecture ne prenne pas l'absence pour
-- un oubli.

-- =============================================================================================
--  Sécurité au niveau ligne
-- =============================================================================================

ALTER TABLE hebergement.occupation ENABLE ROW LEVEL SECURITY;
ALTER TABLE hebergement.occupation FORCE  ROW LEVEL SECURITY;

CREATE POLICY isolation_tenant ON hebergement.occupation
    USING      (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

-- =============================================================================================
--  Privilèges — PAS DE `DELETE`, et c'est ce qui dit la classe
-- =============================================================================================
--
-- Une occupation ne se supprime pas : elle se **libère**, ce qui est un `UPDATE` de sa période et
-- de son statut. Une chambre occupée reste une chambre occupée dans l'histoire.
--
-- Accorder `DELETE` permettrait d'effacer la trace d'une attribution — et le classement en B
-- deviendrait faux **sans que rien ne le signale**. Les privilèges disent la classe (module doré).
GRANT SELECT, INSERT, UPDATE ON hebergement.occupation TO kaya_app;
