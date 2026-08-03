-- 0031 — SEJ-02 : le séjour, ses accompagnants, et le lien vers l'occupation du cycle 004.
--
-- ═══════════════════════════════════════════════════════════════════════════════════════════════
--  CE QUE CETTE MIGRATION N'AJOUTE PAS, ET C'EST LE POINT LE PLUS IMPORTANT
--
--      Elle NE TOUCHE PAS `occupation_sans_chevauchement`.
--
--  La contrainte d'exclusion de `0025` reste la garantie du produit : deux clients ne peuvent
--  jamais recevoir la même unité au même moment. Cette migration lui ajoute une colonne voisine
--  et rien d'autre. **La porte P-09 est ré-exercée dans le même changement** — une migration qui
--  recréerait la table perdrait la contrainte sans que rien ne le dise, et
--  `backend/tests/hebergement_disponibilite.rs` vérifie après coup que le type de `periode` est
--  toujours `tstzrange`, que la contrainte existe avec ses deux opérateurs, et qu'elle se
--  déclenche encore — cette fois **par le parcours de séjour**, pas par l'endpoint nu.
-- ═══════════════════════════════════════════════════════════════════════════════════════════════

-- =============================================================================================
--  1. hebergement.sejour
-- =============================================================================================
CREATE TABLE hebergement.sejour (
    -- UUID v7 **généré côté client** (FR-086) : c'est lui, et non une clé engendrée côté serveur,
    -- qui rend le rejeu inoffensif. Le serveur déduplique, il n'engendre pas.
    id                UUID        PRIMARY KEY,
    tenant_id         UUID        NOT NULL,
    etablissement_id  UUID        NOT NULL,

    -- ⚠️ **AUCUN `REFERENCES`** : ce serait une clé étrangère inter-schémas (principe II,
    -- porte P-04). Même régime que `comptes.permission.module_code`, dont le commentaire de `0016`
    -- dit exactement pourquoi. La lecture passe par le trait `AnnuaireClients`, et le refus d'un
    -- `client_id` inventé est explicite dans le service — la base ne peut pas le tenir.
    --
    -- **`NULL` est LÉGAL, et c'est le parcours normal du passage** : la pièce d'identité se
    -- saisit APRÈS la clé (maquette `R4`, FR-023). Un séjour sans fiche est un séjour valide,
    -- dont la fiche de police naît `complete = false`.
    client_id         UUID        NULL,

    statut            TEXT        NOT NULL DEFAULT 'en_cours'
        CONSTRAINT sejour_statut_connu CHECK (statut IN ('en_cours', 'clos')),

    -- Horodatage d'**AUTORITÉ**. Le calcul de durée le lit ; **jamais l'horloge d'un terminal**
    -- (porte P-23). C'est ce qui rend impossible d'antidater une nuit depuis un appareil mal réglé.
    ouvert_le         TIMESTAMPTZ NOT NULL DEFAULT now(),
    clos_le           TIMESTAMPTZ NULL,

    -- Indicatif, **aucune règle** (porte P-23).
    horodatage_client TIMESTAMPTZ NULL,
    cree_le           TIMESTAMPTZ NOT NULL DEFAULT now(),
    modifie_le        TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- L'égalité de deux booléens plutôt que deux `CHECK` séparés : elle interdit les **deux**
    -- incohérences, dont celle qu'on oublie — un `clos_le` posé sur un séjour encore en cours.
    -- Patron de `occupation_liberation_coherente` (`0025`).
    CONSTRAINT sejour_cloture_coherente
        CHECK ((statut = 'clos') = (clos_le IS NOT NULL)),
    CONSTRAINT sejour_cloture_apres_ouverture
        CHECK (clos_le IS NULL OR clos_le >= ouvert_le)
);

COMMENT ON TABLE hebergement.sejour IS
    'Le passage d''un client dans l''établissement, de l''arrivée au départ. Classe hors-ligne B. client_id est un UUID SANS clé étrangère : ce serait une clé inter-schémas (P-04) ; la lecture passe par le trait AnnuaireClients.';
COMMENT ON COLUMN hebergement.sejour.ouvert_le IS
    'Horodatage d''AUTORITÉ SERVEUR. Le calcul de durée réelle au départ le lit ; jamais l''horloge d''un terminal (P-23).';
COMMENT ON COLUMN hebergement.sejour.client_id IS
    'NULL est LÉGAL : un passage s''enregistre sans fiche, la pièce venant APRÈS la clé (FR-023).';

-- La liste des séjours en cours d'un établissement — l'écran de départ l'ouvre à chaque montage.
CREATE INDEX sejour_en_cours_idx
    ON hebergement.sejour (tenant_id, etablissement_id, statut, ouvert_le DESC);

-- L'historique des séjours d'un client (`GET /clients/{id}/sejours`), servi **depuis
-- `hebergement`**, jamais depuis `comptes`.
CREATE INDEX sejour_par_client_idx
    ON hebergement.sejour (tenant_id, client_id, ouvert_le DESC);

-- =============================================================================================
--  2. hebergement.accompagnant — classe A
-- =============================================================================================
CREATE TABLE hebergement.accompagnant (
    id                UUID        PRIMARY KEY,
    tenant_id         UUID        NOT NULL,
    -- Clé étrangère **intra-schéma** : légale (principe II).
    sejour_id         UUID        NOT NULL REFERENCES hebergement.sejour (id),

    -- **Un nom suffit** (FR-015). Le reste est facultatif, et c'est ce qui rend l'ajout tenable au
    -- comptoir : demander une pièce par accompagnant coûterait la cible des 60 secondes de
    -- l'arrivée.
    nom               TEXT        NOT NULL CHECK (length(btrim(nom)) BETWEEN 1 AND 200),
    prenoms           TEXT        NULL,
    date_naissance    DATE        NULL,
    nationalite       TEXT        NULL,

    -- ⚠️ **SECONDE SURFACE DE RÉTENTION DU PRODUIT, et elle est ASSUMÉE.**
    --
    -- La fiche de police couvre le titulaire **et ses accompagnants** (FR-046). Un accompagnant
    -- n'a **pas** de fiche client — lui en créer une pour porter sa pièce ferait entrer au fichier
    -- des personnes qui n'ont rien demandé, ce qui est pire que la colonne.
    --
    -- Conséquence écrite à `docs/user-stories-v1.md`, TRX-06 : la purge de 90 jours portera sur
    -- **DEUX** tables, `comptes.personne` et celle-ci. `piece_capturee_le` est présent pour la
    -- même raison qu'ailleurs — la rétention s'appliquera **sans migration**.
    --
    -- `provisions_sans_logique.rs` n'est **pas contourné** : son contrôle porte sur les provisions
    -- RH (`employe`, `appareil_enrole`), et cette table n'en est pas une. Son périmètre est
    -- confirmé, pas élargi.
    type_piece        TEXT        NULL,
    numero_piece      TEXT        NULL,
    piece_capturee_le TIMESTAMPTZ NULL,

    -- **`retire_le` plutôt qu'un `DELETE`.** Sans cela, la fiche de police perdrait la trace d'une
    -- personne qui a bien été déclarée — et une fiche de police qui perd une déclaration est un
    -- document faux devant la gendarmerie.
    retire_le         TIMESTAMPTZ NULL,

    -- Indicatif, **aucune règle** (porte P-23) — mais **écrit**, comme sur toute classe A.
    -- Écrire la colonne n'est pas s'appuyer dessus.
    horodatage_client TIMESTAMPTZ NULL,
    -- **AUTORITÉ** — c'est lui qui ordonne.
    cree_le           TIMESTAMPTZ NOT NULL DEFAULT now()
);

COMMENT ON TABLE hebergement.accompagnant IS
    'Une personne qui séjourne AVEC le client, sans fiche à elle. Classe hors-ligne A, branche A4. Un nom suffit (FR-015). Retrait par retire_le, jamais par DELETE : la fiche de police perdrait la trace d''une personne déclarée.';
COMMENT ON COLUMN hebergement.accompagnant.numero_piece IS
    'SECONDE surface de rétention du numéro de pièce, assumée : un accompagnant n''a pas de fiche client. La purge de TRX-06 portera sur DEUX tables.';

CREATE INDEX accompagnant_par_sejour_idx
    ON hebergement.accompagnant (sejour_id, cree_le)
    WHERE retire_le IS NULL;

-- =============================================================================================
--  3. hebergement.occupation — une colonne, et pas une de plus
-- =============================================================================================
--
-- **`NULL` est nécessaire**, et ce n'est pas une facilité : l'endpoint d'attribution nu du
-- cycle 004 existe toujours et n'ouvre aucun séjour. Le rendre obligatoire casserait une opération
-- servie — et le casserait à la première attribution faite depuis l'écran de disponibilité.
--
-- **Un séjour porte une à N occupations** : c'est ce qui rend le changement d'unité possible sans
-- casser l'historique (FR-079, FR-081). Le lien est donc porté par l'occupation, pas par le séjour.
ALTER TABLE hebergement.occupation
    ADD COLUMN sejour_id UUID NULL REFERENCES hebergement.sejour (id);

COMMENT ON COLUMN hebergement.occupation.sejour_id IS
    'NULL pour une attribution nue (endpoint HEB-02, cycle 004). Un séjour porte une à N occupations — c''est ce qui rend le changement d''unité possible sans casser l''historique.';

-- Index **partiel** : la très grande majorité des occupations d'un exploitant qui n'emploie que
-- l'écran de disponibilité n'ont pas de séjour. Indexer les `NULL` coûterait de l'espace pour des
-- lignes qu'aucune requête ne cherche.
CREATE INDEX occupation_par_sejour_idx
    ON hebergement.occupation (sejour_id)
    WHERE sejour_id IS NOT NULL;

-- =============================================================================================
--  4. synchronisation.reconciliation_orpheline — le privilège élargi
-- =============================================================================================
--
-- **Elle cesse d'être une provision.** Posée au cycle 005 avec `GRANT SELECT` **seul** pour
-- prouver qu'elle n'avait aucune logique, elle reçoit son premier écrivain : un accompagnant de
-- classe A arrivant **après** la clôture du séjour.
--
-- C'est le **premier cas réel d'écriture orpheline du produit**. Le cadrage §11.4 le décrit avec
-- une consommation de bar sur un séjour facturé (T2) ; l'accompagnant hors ligne le produit dès ce
-- cycle, et il est plus simple à éprouver.
--
-- ⚠️ **`UPDATE` n'est PAS accordé.** La *résolution* d'une écriture orpheline est **SYN-03**,
-- tranche T3. Ce cycle alimente la file ; il ne la vide pas. Accorder `UPDATE` maintenant ferait
-- croire à une résolution qui n'existe pas, et `provisions_sans_logique.rs` ne pourrait plus dire
-- ce qui est construit de ce qui est promis.
--
-- Le décompte de `provisions_sans_logique.rs` passe de **six à cinq**.
GRANT INSERT ON synchronisation.reconciliation_orpheline TO kaya_app;

-- =============================================================================================
--  5. Sécurité au niveau ligne
-- =============================================================================================

ALTER TABLE hebergement.sejour ENABLE ROW LEVEL SECURITY;
ALTER TABLE hebergement.sejour FORCE  ROW LEVEL SECURITY;
CREATE POLICY isolation_tenant ON hebergement.sejour
    USING      (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

ALTER TABLE hebergement.accompagnant ENABLE ROW LEVEL SECURITY;
ALTER TABLE hebergement.accompagnant FORCE  ROW LEVEL SECURITY;
CREATE POLICY isolation_tenant ON hebergement.accompagnant
    USING      (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

-- =============================================================================================
--  6. Privilèges — AUCUN `DELETE`, nulle part
-- =============================================================================================
--
-- **Un séjour ne se supprime pas ; il se clôt.** Accorder `DELETE` permettrait d'effacer une nuit
-- vendue, et le classement en B deviendrait faux **sans que rien ne le signale**. Les privilèges
-- disent la classe (module doré, couche 1).
--
-- **`accompagnant` reçoit `UPDATE`** — uniquement pour poser `retire_le`. C'est un retrait, pas
-- une suppression : la fiche de police garde la trace de la personne déclarée.
GRANT SELECT, INSERT, UPDATE ON hebergement.sejour       TO kaya_app;
GRANT SELECT, INSERT, UPDATE ON hebergement.accompagnant TO kaya_app;
