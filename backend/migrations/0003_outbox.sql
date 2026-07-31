-- 0003 — `evenement_outbox`, le GRAND LIVRE PERMANENT.
--
-- **Ce n'est pas une file de messages** (principe II). Rétention illimitée, charge utile
-- financière complète et dénormalisée, immuable. Une correction est un nouvel événement, jamais
-- une modification de l'ancien.
--
-- Ordre imposé : ce fichier précède `0004_note_etablissement.sql`. Le module doré écrit son
-- événement dans la même transaction que sa ligne métier — la table cible doit exister avant.

-- =============================================================================================
--  1. La table
-- =============================================================================================
CREATE TABLE synchronisation.evenement_outbox (
    id                      UUID        PRIMARY KEY,
    tenant_id               UUID        NOT NULL,
    -- NULL pour un événement de niveau tenant (création d'établissement, par exemple).
    etablissement_id        UUID            NULL,
    -- Monotone **par établissement** (R-07). Une séquence globale ne conviendrait pas : deux
    -- établissements se partageraient les numéros, et un trou dans la suite de l'un serait
    -- indistinguable d'un événement écrit par l'autre. C'est cette monotonie qui permettra à un
    -- nœud de site (mode C, incrément 3) de détecter qu'il lui manque un événement.
    sequence_etablissement  BIGINT      NOT NULL,
    type_evenement          TEXT        NOT NULL,
    agregat                 TEXT        NOT NULL,
    agregat_id              UUID        NOT NULL,
    -- Version du FORMAT de `payload` (R-06). Coûte un entier aujourd'hui et vaut toute la
    -- provision §14.7 : en phase 2, la génération SYSCOHADA rétroactive relira des événements
    -- écrits par des versions du code qui n'existeront plus. Sans numéro de version, il faudrait
    -- deviner ; avec, on écrit un décodeur par génération de format.
    version_schema          SMALLINT    NOT NULL,
    -- COMPLET et DÉNORMALISÉ. Critère opposable en revue : un lecteur qui n'a que cette ligne et
    -- le numéro de version doit pouvoir dire ce qui s'est passé, pour quel montant, avec quelles
    -- taxes et sur quel document. Jamais un identifiant renvoyant à une autre table.
    payload                 JSONB       NOT NULL,
    -- Horodatage d'AUTORITÉ SERVEUR (principe IV). Jamais l'horloge d'un terminal.
    survenu_le              TIMESTAMPTZ NOT NULL,
    -- NULL = non publié. **Jamais de suppression, jamais de retour à NULL.**
    publie_le               TIMESTAMPTZ     NULL,

    -- `NULLS NOT DISTINCT` étend l'unicité aux événements de niveau tenant. Sans lui, deux
    -- lignes à `etablissement_id` NULL portant la même séquence seraient acceptées, PostgreSQL
    -- considérant par défaut que deux NULL ne sont pas égaux — la moitié des événements
    -- échapperait à la contrainte censée les ordonner.
    CONSTRAINT evenement_outbox_sequence_unique
        UNIQUE NULLS NOT DISTINCT (etablissement_id, sequence_etablissement)
);

COMMENT ON TABLE synchronisation.evenement_outbox IS
    'Grand livre permanent. Rétention ILLIMITÉE, immuable. Classe hors-ligne A.';

-- **L'index le plus important du produit sur le long terme.**
-- Partiel sur `publie_le IS NULL`, donc il ne contient que les événements en attente : c'est le
-- seul index de cette table qui reste petit indéfiniment. Sans lui, la publication ralentirait
-- proportionnellement à l'historique, et la première réaction serait de purger — exactement ce
-- que TRX-02 interdit.
CREATE INDEX evenement_outbox_en_attente_idx
    ON synchronisation.evenement_outbox (id)
    WHERE publie_le IS NULL;

CREATE INDEX evenement_outbox_tenant_temps_idx
    ON synchronisation.evenement_outbox (tenant_id, survenu_le);

CREATE INDEX evenement_outbox_agregat_idx
    ON synchronisation.evenement_outbox (agregat, agregat_id);

-- =============================================================================================
--  2. Séquence monotone par établissement
-- =============================================================================================
--
-- Une séquence PostgreSQL par établissement, créée à la demande. La fonction est
-- `SECURITY DEFINER` parce que la créer exige `CREATE` sur le schéma — un droit que `kaya_app`
-- ne doit jamais avoir : il ouvrirait la porte à la création d'une table hors migration, donc
-- hors du principe I(b) et hors de la porte P-07.
--
-- **Les séquences ne sont pas transactionnelles : un rollback laisse un trou.** C'est accepté et
-- voulu (R-07). La séquence garantit l'ORDRE et la DÉTECTION DE MANQUE, pas l'absence de trou.
-- L'inverse imposerait un verrou par établissement sur le chemin d'écriture le plus chaud du
-- produit. Personne ne doit « corriger » plus tard un trou qui n'est pas un bug.

CREATE FUNCTION synchronisation.prochaine_sequence(p_cle UUID)
RETURNS BIGINT
LANGUAGE plpgsql
SECURITY DEFINER
-- `search_path` figé : sans lui, un appelant pourrait interposer ses propres objets devant ceux
-- du schéma et détourner une fonction qui s'exécute avec les droits du propriétaire.
SET search_path = synchronisation, pg_catalog
AS $$
DECLARE
    nom_sequence TEXT := 'seq_' || replace(p_cle::text, '-', '_');
    valeur       BIGINT;
BEGIN
    EXECUTE format('CREATE SEQUENCE IF NOT EXISTS synchronisation.%I', nom_sequence);
    EXECUTE format('SELECT nextval(''synchronisation.%I'')', nom_sequence) INTO valeur;
    RETURN valeur;
END;
$$;

REVOKE ALL ON FUNCTION synchronisation.prochaine_sequence(UUID) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION synchronisation.prochaine_sequence(UUID) TO kaya_app;

-- =============================================================================================
--  3. Immuabilité — TROIS couches, pour trois fautes différentes (R-05)
-- =============================================================================================
--
--   Couche 1, les privilèges : arrête le bug applicatif.
--   Couche 2, le déclencheur : arrête la migration ou le script de maintenance lancé sous
--             `kaya_owner` — le cas réel du développeur solo qui se connecte en production à
--             23 h pour « corriger une ligne ».
--   Couche 3, la porte de CI `scripts/ci/outbox-sans-purge.sh` : arrête le code qui aurait été
--             écrit pour purger.
--
-- Aucune des trois ne suffit seule.

-- --- Couche 1 -------------------------------------------------------------------------------
REVOKE UPDATE, DELETE, TRUNCATE ON synchronisation.evenement_outbox FROM kaya_app;
GRANT  SELECT, INSERT           ON synchronisation.evenement_outbox TO   kaya_app;
-- Seule mutation concédée, et sur une seule colonne. `SELECT ... FOR UPDATE SKIP LOCKED` du
-- worker (R-08) s'appuie sur ce privilège de colonne.
GRANT  UPDATE (publie_le)       ON synchronisation.evenement_outbox TO   kaya_app;

-- Le rôle du test de reconstitution autonome : lecture, et **uniquement** sur cette table.
-- C'est l'absence de tout autre droit qui fait la démonstration (R-11).
GRANT  SELECT                   ON synchronisation.evenement_outbox TO   kaya_ledger_reader;

-- --- Couche 2 -------------------------------------------------------------------------------
CREATE FUNCTION synchronisation.refuser_mutation_evenement()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        RAISE EXCEPTION
            'evenement_outbox est un grand livre permanent : suppression interdite (ligne %)',
            OLD.id
            USING ERRCODE = 'restrict_violation';
    END IF;

    -- Le marquage « publié » est formellement une mutation. C'est la seule tolérée, et elle est
    -- **monotone et non réversible** : NULL vers une valeur, une seule fois, jamais l'inverse.
    -- Sans cette exception il faudrait une seconde table de marquage, donc une jointure de plus
    -- sur le chemin du grand livre — précisément ce que TRX-02 cherche à éviter.
    IF OLD.publie_le IS NOT NULL THEN
        RAISE EXCEPTION
            'evenement_outbox : la publication est définitive, elle ne se rejoue pas (ligne %)',
            OLD.id
            USING ERRCODE = 'restrict_violation';
    END IF;

    IF NEW.publie_le IS NULL THEN
        RAISE EXCEPTION
            'evenement_outbox : publie_le ne revient jamais à NULL (ligne %)', OLD.id
            USING ERRCODE = 'restrict_violation';
    END IF;

    -- Toute autre différence entre l'ancienne et la nouvelle ligne est refusée. La comparaison
    -- se fait par `to_jsonb`, ce qui évite d'ajouter l'extension `hstore` pour un unique test
    -- (data-model §4.1, note d'implémentation).
    IF (to_jsonb(NEW) - 'publie_le') IS DISTINCT FROM (to_jsonb(OLD) - 'publie_le') THEN
        RAISE EXCEPTION
            'evenement_outbox est immuable : seul le marquage de publication est permis (ligne %)',
            OLD.id
            USING ERRCODE = 'restrict_violation';
    END IF;

    RETURN NEW;
END;
$$;

-- Un déclencheur s'applique à TOUS les rôles, y compris le propriétaire des tables et un
-- superutilisateur. C'est ce qui le rend complémentaire des privilèges, et non redondant.
CREATE TRIGGER evenement_outbox_immuable
    BEFORE UPDATE OR DELETE ON synchronisation.evenement_outbox
    FOR EACH ROW EXECUTE FUNCTION synchronisation.refuser_mutation_evenement();

-- =============================================================================================
--  4. Isolation multi-tenant
-- =============================================================================================
ALTER TABLE synchronisation.evenement_outbox ENABLE ROW LEVEL SECURITY;
ALTER TABLE synchronisation.evenement_outbox FORCE  ROW LEVEL SECURITY;

CREATE POLICY isolation_tenant ON synchronisation.evenement_outbox
    USING      (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);
