-- 0035 — SEJ-04 : le **constat de taxe**, figé au départ.
--
-- ═══════════════════════════════════════════════════════════════════════════════════════════════
--  ★ LE POINT LE PLUS DÉLICAT DU CYCLE, ET IL TIENT EN UNE DISTINCTION
--
--      Ce cycle fige un CONSTAT. Il ne calcule AUCUN montant.
--
--  | Ce que cette table écrit | Nature | Qui l'interprète |
--  |---|---|---|
--  | `nuits_constatees` — nombre de nuits de la période | **arithmétique** | — |
--  | `nombre_personnes` — titulaire et accompagnants | **arithmétique** | — |
--  | `assujettie_taxe_nuitee`, `regle_conversion_taxe` — RECOPIÉS | **paramétrage** | FIS-03 |
--  | `classement_etablissement`, `commune` — RECOPIÉS | **paramétrage** | FIS-03 |
--  | `fige_le` — l'instant d'autorité du figeage | **fait** | — |
--  | `nuitees_assujetties`, `montant_mineur` | **POSÉES, JAMAIS ALIMENTÉES** | FIS-03 |
--
--  **Compter les nuits d'un intervalle est de l'arithmétique. Décider lesquelles sont assujetties
--  est une RÈGLE FISCALE.** `une_nuitee_par_occupation` réduit trois nuits à une : c'est un
--  arbitrage fiscal, il ne vit que dans `JurisdictionAdapter` (principe V, porte P-12). Ce cycle
--  enregistre **trois** et la règle lue, jamais **un**.
--
--  La porte **P-11** doit rester VERTE À VIDE. Si elle se réveillait — un jeu de cas apparaissant
--  dans `backend/tests/fixtures/fiscal` —, c'est qu'une règle fiscale aurait été écrite ici.
-- ═══════════════════════════════════════════════════════════════════════════════════════════════
--
-- ## Ce que « figé » veut dire, et pourquoi c'est vérifiable
--
-- **Tout ce qui pourrait changer après le départ est RECOPIÉ** : accompagnants comptés, barème,
-- formule, classement, commune. Un montant calculé plus tard depuis ce constat est donc **stable,
-- quelle que soit la date du calcul**.
--
-- Référencer la formule plutôt que la recopier aurait paru plus propre — et aurait fait bouger un
-- séjour clos hier au premier changement de tarif de demain. C'est exactement ce que SC-007
-- interdit.
--
-- ⚠️ **Le numéro dévie du plan d'un cran** : le plan réservait `0034` à ce fichier. La charge
-- utile de la file de réconciliation l'a pris (défaut trouvé en écrivant son premier écrivain), et
-- `sqlx` refuse une version antérieure à une version déjà appliquée.

CREATE TABLE hebergement.taxe_sejour_constat (
    id               UUID        PRIMARY KEY,
    tenant_id        UUID        NOT NULL,
    etablissement_id UUID        NOT NULL,

    -- **UN constat par SÉJOUR, jamais par occupation** (FR-083). Un séjour à deux chambres —
    -- changement d'unité en cours de route — produit **un** constat, pas deux. La contrainte
    -- `UNIQUE` le porte ; sans elle, un changement d'unité doublerait la taxe due.
    sejour_id        UUID        NOT NULL UNIQUE REFERENCES hebergement.sejour (id),

    -- ═════════════════════════════════════════════════════════════════════════════════════════
    --  LES FAITS — de l'arithmétique, aucune règle fiscale
    -- ═════════════════════════════════════════════════════════════════════════════════════════
    --
    -- `>= 0` et non `> 0` : un passage de deux heures produit **zéro** nuit constatée, et c'est
    -- un fait juste. Exiger au moins une nuit ferait mentir le constat d'un passage.
    nuits_constatees INTEGER     NOT NULL CHECK (nuits_constatees >= 0),

    -- Le titulaire compte pour un. `>= 1` : un séjour sans personne n'existe pas.
    --
    -- ⚠️ **Enregistré à titre INDICATIF depuis la décision B-10** (close le 2026-08-03) : la taxe
    -- est due **par nuitée et par séjour**, jamais par personne. Ce nombre documente le séjour ;
    -- il n'entre dans aucun calcul. Le garder n'est pas contradictoire — la fiche de police et
    -- l'état de reversement communal s'y réfèrent, et le retirer nous priverait d'un fait que
    -- rien d'autre ne porte après le départ.
    nombre_personnes INTEGER     NOT NULL CHECK (nombre_personnes >= 1),

    periode_debut    TIMESTAMPTZ NOT NULL,
    periode_fin      TIMESTAMPTZ NOT NULL,

    -- ═════════════════════════════════════════════════════════════════════════════════════════
    --  LE PARAMÉTRAGE, RECOPIÉ — c'est ce qui rend le figeage VRAI
    -- ═════════════════════════════════════════════════════════════════════════════════════════
    --
    -- **Recopié, jamais référencé.** Une formule éditée demain, un classement changé, une commune
    -- redécoupée ne doivent RIEN changer à un séjour clos hier (FR-063, SC-007).
    --
    -- `formule_id` est conservé **sans `REFERENCES`** : il sert à retrouver l'origine, pas à
    -- garantir l'intégrité. Une clé étrangère empêcherait de supprimer une formule obsolète alors
    -- que le constat, lui, porte déjà tout ce dont FIS-03 a besoin.
    formule_id               UUID    NOT NULL,
    famille_formule          TEXT    NOT NULL,
    assujettie_taxe_nuitee   BOOLEAN NOT NULL,
    regle_conversion_taxe    TEXT    NULL,
    classement_etablissement TEXT    NOT NULL,
    commune                  TEXT    NOT NULL,

    -- ═════════════════════════════════════════════════════════════════════════════════════════
    --  POSÉES, JAMAIS ALIMENTÉES PAR CE CYCLE (principe X)
    -- ═════════════════════════════════════════════════════════════════════════════════════════
    --
    -- Décider quelles nuits sont assujetties est une **règle fiscale** : elle ne vit que dans
    -- `JurisdictionAdapter` (principe V, porte P-12), et son test doré appartient à **FIS-03**.
    --
    -- `provisions_sans_logique.rs` vérifie **les deux versants** : qu'aucun chemin de code de ce
    -- cycle ne les écrit, **et** qu'elles existent bel et bien. Sans le second, supprimer les
    -- colonnes suffirait à passer au vert — et le jour où FIS-03 arriverait, il faudrait une
    -- migration au lieu d'un `UPDATE`.
    nuitees_assujetties INTEGER NULL,
    montant_mineur      BIGINT  NULL,
    devise              TEXT    NULL CHECK (devise IS NULL OR length(devise) = 3),

    -- Horodatage d'**AUTORITÉ** du figeage. `now()` de la base, jamais l'horloge d'un terminal.
    fige_le          TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT constat_periode_coherente CHECK (periode_fin > periode_debut),

    -- Une formule assujettie **sans règle** serait un état d'attente. La contrainte du cycle 004
    -- l'interdit déjà sur `formule` ; celle-ci le tient **sur la copie** — sans quoi une formule
    -- corrigée après coup pourrait laisser un constat incohérent derrière elle.
    CONSTRAINT constat_regle_coherente
        CHECK (assujettie_taxe_nuitee = false OR regle_conversion_taxe IS NOT NULL)
);

COMMENT ON TABLE hebergement.taxe_sejour_constat IS
    'Le CONSTAT de taxe figé au départ : des faits et un paramétrage RECOPIÉ. Classe hors-ligne B. Aucune règle fiscale — décider quelles nuits sont assujetties vit dans JurisdictionAdapter (P-12). IMMUABLE PAR PRIVILÈGE : SELECT et INSERT seuls.';
COMMENT ON COLUMN hebergement.taxe_sejour_constat.nuits_constatees IS
    'ARITHMÉTIQUE : le nombre de nuits calendaires de la période. Zéro pour un passage, et c''est juste.';
COMMENT ON COLUMN hebergement.taxe_sejour_constat.nuitees_assujetties IS
    'POSÉE, JAMAIS ALIMENTÉE par le cycle 006. Décider lesquelles sont assujetties est une RÈGLE FISCALE — FIS-03, tranche T3.';
COMMENT ON COLUMN hebergement.taxe_sejour_constat.nombre_personnes IS
    'INDICATIF depuis la décision B-10 (2026-08-03) : la taxe est due par nuitée et par SÉJOUR, jamais par personne. Documente le séjour, n''entre dans aucun calcul.';

CREATE INDEX taxe_sejour_constat_par_commune_idx
    ON hebergement.taxe_sejour_constat (tenant_id, commune, fige_le);

-- =============================================================================================
--  Sécurité au niveau ligne
-- =============================================================================================

ALTER TABLE hebergement.taxe_sejour_constat ENABLE ROW LEVEL SECURITY;
ALTER TABLE hebergement.taxe_sejour_constat FORCE  ROW LEVEL SECURITY;
CREATE POLICY isolation_tenant ON hebergement.taxe_sejour_constat
    USING      (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

-- =============================================================================================
--  ★ LE FIGEAGE EST UN PRIVILÈGE, PAS UNE INTENTION
-- =============================================================================================
--
-- **Ni `UPDATE`, ni `DELETE`.** Le rôle applicatif **ne peut pas** modifier un constat, quelle que
-- soit la ligne de code écrite au-dessus.
--
-- C'est ce qui transforme **SC-007** — « l'assiette est immuable après le départ » — d'une
-- promesse en une **propriété de la base**. Une relecture ne peut pas la recalculer ; elle n'en a
-- pas le droit.
--
-- ⚠️ **`sejour_depart.rs` l'asserte tout de même**, en tentant l'`UPDATE` sous le rôle applicatif.
-- Une garantie de privilège se perd en une ligne de migration, et le test est ce qui rend la perte
-- bruyante.
GRANT SELECT, INSERT ON hebergement.taxe_sejour_constat TO kaya_app;
