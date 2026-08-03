-- 0034 — SEJ-02 : la **charge utile** d'une écriture orpheline, et son motif.
--
-- ═══════════════════════════════════════════════════════════════════════════════════════════════
--  ★ CE DÉFAUT N'A ÉTÉ TROUVÉ QU'EN ÉCRIVANT LE PREMIER ÉCRIVAIN DE LA TABLE
--
--  `synchronisation.reconciliation_orpheline` est posée au cycle 005 avec `GRANT SELECT` seul,
--  comme provision. Ses colonnes disent **quoi** est arrivé en retard et **sur quel agrégat** —
--  `ecriture_id`, `ecriture_type`, `agregat_type`, `agregat_id` — et rien de plus.
--
--  **Elle ne porte AUCUNE charge utile.** Or, quand un accompagnant arrive après la clôture du
--  séjour, la ligne `hebergement.accompagnant` **n'est pas écrite** : le séjour est clos, l'ajout
--  est refusé comme ajout. Si la file ne retient que des identifiants, **le nom de la personne est
--  perdu** — et SYN-03 (tranche T3) n'aura littéralement rien à rattacher.
--
--  Le symptôme, le jour où SYN-03 sera écrit : un écran de réconciliation qui affiche des lignes
--  sans contenu, et une équipe qui conclut que la file « ne marche pas ». La cause serait à deux
--  cycles de distance.
--
--  La provision du cycle 005 était juste dans son intention et **incomplète dans sa forme** — ce
--  qui est le mode d'échec normal d'une table posée sans écrivain : rien ne pouvait le révéler
--  avant qu'un écrivain n'existe. C'est précisément ce que `provisions_sans_logique.rs` mesure, et
--  ce que cette migration corrige au moment où le premier écrivain arrive.
-- ═══════════════════════════════════════════════════════════════════════════════════════════════
--
-- ## Pourquoi une migration NOUVELLE plutôt qu'une correction de `0031`
--
-- La porte **P-02** compare les migrations à la branche de base : `0031` y étant un **ajout**, la
-- modifier ne la ferait pas échouer. Ce n'est pas la raison de s'en abstenir.
--
-- La raison est que **la colonne n'appartient pas au séjour**. Elle appartient au cas orphelin,
-- qui est un mécanisme de `socle/synchronisation` — et l'enfouir dans la migration des tables
-- d'hébergement rendrait invisible, à la relecture, le fait que le cycle 005 avait laissé la
-- provision incomplète.
--
-- ⚠️ **Le numéro dévie du plan d'un cran** : le plan réservait `0034` au constat de taxe, qui
-- passe à `0035`. `sqlx` refuse une version antérieure à une version déjà appliquée (constaté au
-- cycle 001) — l'ordre des numéros suit l'ordre d'écriture, pas l'ordre thématique.

-- =============================================================================================
--  1. La charge utile
-- =============================================================================================
ALTER TABLE synchronisation.reconciliation_orpheline
    -- **Du JSON OPAQUE pour le socle.** `kaya_synchronisation` ne doit connaître ni
    -- `Accompagnant`, ni `Sejour` : le typer ferait remonter un type de verticale dans une
    -- signature du socle, ce que la porte **P-03** refuse. C'est le piège concret que
    -- `contracts/traits-exposes.md` désigne nommément.
    --
    -- `NULL` autorisé : les lignes qu'un cycle ultérieur écrirait pour un agrégat dont l'écriture
    -- EST en base — une consommation de bar déjà enregistrée, cas du cadrage §11.4 — n'ont rien à
    -- recopier. La colonne porte ce qui serait **perdu autrement**, pas une duplication de
    -- principe.
    ADD COLUMN charge_utile JSONB NULL,

    -- **Le motif, en code stable.** L'écran de SYN-03 le traduira par le lexique — jamais un
    -- message de diagnostic (règle du cycle 002).
    --
    -- Un seul motif existe à ce cycle, et le `CHECK` le dit plutôt que de laisser le champ libre :
    -- une valeur inventée par un cycle ultérieur doit se décider, pas apparaître.
    ADD COLUMN motif TEXT NULL
        CONSTRAINT reconciliation_orpheline_motif_connu
            CHECK (motif IS NULL OR motif IN ('sejour_clos'));

COMMENT ON COLUMN synchronisation.reconciliation_orpheline.charge_utile IS
    'Ce que l''écriture portait, en JSON OPAQUE pour le socle (P-03 : kaya_synchronisation ne connaît ni Accompagnant ni Sejour). Sans elle, un accompagnant arrivé après la clôture perdrait son nom et SYN-03 n''aurait rien à rattacher.';
COMMENT ON COLUMN synchronisation.reconciliation_orpheline.motif IS
    'Code stable traduit par le lexique. Un seul motif au cycle 006 : sejour_clos.';

-- =============================================================================================
--  2. L'index de l'écran de SYN-03
-- =============================================================================================
--
-- L'écran listera les constats **non résolus** d'un établissement, du plus récent au plus ancien.
-- L'index est **partiel** : une file résolue n'est plus consultée, et indexer ses lignes ferait
-- grandir l'index avec l'historique pour des lignes que personne ne cherche.
CREATE INDEX reconciliation_orpheline_a_trancher_idx
    ON synchronisation.reconciliation_orpheline (tenant_id, etablissement_id, cree_le DESC)
    WHERE etat = 'constatee';
