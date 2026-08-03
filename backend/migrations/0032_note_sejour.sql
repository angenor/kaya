-- 0032 — SEJ-02 / SEJ-04 : la note du séjour et ses lignes.
--
-- ═══════════════════════════════════════════════════════════════════════════════════════════════
--  DEUX DÉCISIONS QUI NE SE RATTRAPENT PAS
--
--   1. **`note_sejour` n'a AUCUNE colonne de total.** Le total est la somme des lignes.
--   2. **`ligne_sejour` n'a PAS d'`UPDATE`.** Une correction est une ligne d'ajustement.
--
--  La première : une colonne totalisatrice se désynchronise **en silence**, et le silence est
--  exactement ce que le propriétaire achète en installant ce logiciel (cadrage §8.3). Un total
--  faux sur une note ne se voit qu'au moment où le client conteste.
--
--  La seconde : le principe V pose que « les prix sont verrouillés à la création de la ligne » ;
--  le privilège le rend **impossible à contourner**, quelle que soit la ligne de code écrite
--  au-dessus.
-- ═══════════════════════════════════════════════════════════════════════════════════════════════

-- =============================================================================================
--  1. hebergement.note_sejour
-- =============================================================================================
CREATE TABLE hebergement.note_sejour (
    id         UUID        PRIMARY KEY,
    tenant_id  UUID        NOT NULL,

    -- `UNIQUE` : **une note par séjour**. Deux notes sur un séjour produiraient deux totaux, et
    -- l'écran de départ n'aurait aucun moyen de choisir.
    sejour_id  UUID        NOT NULL UNIQUE REFERENCES hebergement.sejour (id),

    -- ISO 4217, **au même niveau que les montants, toujours** (principe V). Un montant sans sa
    -- devise est un nombre ; le produit sert deux devises dès le second pays (principe X).
    devise     TEXT        NOT NULL CHECK (length(devise) = 3),

    statut     TEXT        NOT NULL DEFAULT 'ouverte'
        CONSTRAINT note_statut_connu CHECK (statut IN ('ouverte', 'arretee')),
    arretee_le TIMESTAMPTZ NULL,

    cree_le    TIMESTAMPTZ NOT NULL DEFAULT now(),
    modifie_le TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT note_arret_coherent
        CHECK ((statut = 'arretee') = (arretee_le IS NOT NULL))
);

COMMENT ON TABLE hebergement.note_sejour IS
    'La note d''un séjour. Classe hors-ligne B. AUCUNE colonne de total : le total est la somme des lignes — une colonne totalisatrice se désynchronise en silence.';
COMMENT ON COLUMN hebergement.note_sejour.statut IS
    'Terme utilisateur d''« arretee » : « La note est arrêtée : plus rien ne peut s''y ajouter » (lexique v1.6.0). Jamais « clôturée », « figée » ni « verrouillée ».';

-- =============================================================================================
--  2. hebergement.ligne_sejour
-- =============================================================================================
CREATE TABLE hebergement.ligne_sejour (
    id            UUID        PRIMARY KEY,
    tenant_id     UUID        NOT NULL,
    note_id       UUID        NOT NULL REFERENCES hebergement.note_sejour (id),

    -- L'occupation d'où vient la ligne. `NULL` pour un ajustement qui n'en relève d'aucune.
    occupation_id UUID        NULL REFERENCES hebergement.occupation (id),

    nature        TEXT        NOT NULL
        CONSTRAINT ligne_nature_connue CHECK (nature IN ('hebergement', 'ajustement')),

    -- Renseigné **seulement** sur un ajustement, et jamais deviné. Un motif posé « par défaut »
    -- rendrait le registre des actions inexploitable : le propriétaire y cherche pourquoi un
    -- montant a bougé.
    motif         TEXT        NULL
        CONSTRAINT ligne_motif_connu CHECK (
            motif IS NULL OR motif IN (
                'rebascule_palier', 'depart_anticipe', 'prolongation', 'changement_unite'
            )),

    -- **Clé i18n, JAMAIS une chaîne rendue.** La note s'affiche en `fr` et en `en` (porte P-16) :
    -- écrire « Nuit du lun. 24 au mar. 25 » en base rendrait la note monolingue à jamais, et la
    -- chaîne échapperait entièrement au contrôle des littéraux.
    libelle_cle   TEXT        NOT NULL,

    -- ⚠️ **QUANTITÉ EN `NUMERIC`, JAMAIS ENTIER** (principe V, porte P-10). Une nuitée est 1, une
    -- demi-journée 0,5, et un mois au prorata sera fractionnaire. Passer d'entier à décimal après
    -- mise en production imposerait de migrer toutes les lignes.
    --
    -- `<> 0` : une ligne de quantité nulle serait une ligne qui n'existe pas, avec un montant qui,
    -- lui, existerait.
    quantite      NUMERIC(14, 4) NOT NULL CHECK (quantite <> 0),

    -- ⚠️ **ENTIERS D'UNITÉ MINEURE** (principe V, porte P-10). Le nombre de décimales vient de la
    -- **devise**, jamais d'une constante.
    prix_unitaire_mineur BIGINT NOT NULL,

    -- ⚠️ **PEUT ÊTRE NÉGATIF, et aucun `CHECK` ne l'interdit** : un départ anticipé rembourse.
    -- Le type `Rebascule` du cycle 004 le dit déjà — « ce qui reste dû, PEUT ÊTRE NÉGATIF ».
    -- Poser `>= 0` ici rendrait la régularisation de SEJ-06 impossible sans migration.
    montant_mineur       BIGINT NOT NULL,
    devise               TEXT   NOT NULL CHECK (length(devise) = 3),

    -- La période couverte par la ligne — c'est ce qui rend la note lisible **nuit par nuit** sur
    -- la maquette `R7`.
    periode_debut TIMESTAMPTZ NULL,
    periode_fin   TIMESTAMPTZ NULL,

    cree_le       TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- L'égalité de deux booléens, encore : elle interdit **les deux** incohérences — un ajustement
    -- sans motif, et un motif posé sur une ligne d'hébergement ordinaire.
    CONSTRAINT ligne_ajustement_motive
        CHECK ((nature = 'ajustement') = (motif IS NOT NULL))
);

COMMENT ON TABLE hebergement.ligne_sejour IS
    'Une ligne de la note. Classe hors-ligne B. quantite en NUMERIC (P-10) ; montant_mineur PEUT être négatif — un départ anticipé rembourse. Pas d''UPDATE : une correction est une ligne d''ajustement.';
COMMENT ON COLUMN hebergement.ligne_sejour.libelle_cle IS
    'Clé i18n, jamais un libellé rendu : la note s''affiche en fr ET en en (P-16).';
COMMENT ON COLUMN hebergement.ligne_sejour.quantite IS
    'NUMERIC, jamais entier (principe V, P-10) : une demi-journée est 0,5 et un mois au prorata sera fractionnaire.';

CREATE INDEX ligne_sejour_par_note_idx ON hebergement.ligne_sejour (note_id, cree_le);

-- =============================================================================================
--  3. Sécurité au niveau ligne
-- =============================================================================================

ALTER TABLE hebergement.note_sejour ENABLE ROW LEVEL SECURITY;
ALTER TABLE hebergement.note_sejour FORCE  ROW LEVEL SECURITY;
CREATE POLICY isolation_tenant ON hebergement.note_sejour
    USING      (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

ALTER TABLE hebergement.ligne_sejour ENABLE ROW LEVEL SECURITY;
ALTER TABLE hebergement.ligne_sejour FORCE  ROW LEVEL SECURITY;
CREATE POLICY isolation_tenant ON hebergement.ligne_sejour
    USING      (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

-- =============================================================================================
--  4. Privilèges — l'asymétrie EST la règle
-- =============================================================================================
--
-- **`note_sejour` reçoit `UPDATE`** : elle s'arrête au départ, ce qui est un changement d'état.
--
-- ★ **`ligne_sejour` ne le reçoit PAS.** Le prix verrouillé à la création devient impossible à
-- modifier — pas « interdit par convention », impossible. Une rebascule de palier, un départ
-- anticipé, une prolongation produisent une **ligne d'ajustement** portant son motif, jamais une
-- modification de la ligne initiale. C'est ce qui rend l'histoire de la note relisible : ce qui a
-- été vendu reste écrit, et ce qui a corrigé aussi.
--
-- **Aucune `DELETE` nulle part.**
GRANT SELECT, INSERT, UPDATE ON hebergement.note_sejour  TO kaya_app;
GRANT SELECT, INSERT         ON hebergement.ligne_sejour TO kaya_app;
