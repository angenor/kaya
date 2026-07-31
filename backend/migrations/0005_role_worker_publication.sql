-- 0005 — `kaya_worker`, rôle du worker de publication.
--
-- ## Pourquoi un QUATRIÈME rôle, alors que R-04 en prévoyait trois
--
-- R-04 arrête trois rôles : `kaya_owner` (migrations), `kaya_app` (runtime, soumis à la RLS) et
-- `kaya_ledger_reader` (lecture du seul grand livre). La recherche n'avait pas anticipé une
-- conséquence de `FORCE ROW LEVEL SECURITY` sur le worker de publication (R-08).
--
-- Le worker balaie `evenement_outbox` **tous tenants confondus** : c'est sa fonction. Or la
-- politique `isolation_tenant` filtre sur `current_setting('app.current_tenant', true)`, et
-- `FORCE` s'applique **aussi au propriétaire des tables**. Sous n'importe lequel des trois rôles,
-- le worker ne verrait donc rien du tout — et un worker qui ne voit rien ne publie rien, en
-- silence.
--
-- Trois issues étaient possibles :
--
--   1. **`BYPASSRLS` sur un rôle existant** — écarté. L'attribut vaut pour toutes les tables ; il
--      ouvrirait le contournement bien au-delà de ce que le worker demande.
--   2. **Itérer tenant par tenant** — écarté. Obtenir la liste des tenants demande une lecture
--      non filtrée de `etablissements.tenant` : le problème se déplace au lieu d'être résolu.
--   3. **Un rôle de service, une politique nommée, une seule table** — retenu.
--
-- Ce que `kaya_worker` peut faire, et rien de plus :
--
--   * `SELECT` sur `synchronisation.evenement_outbox`, tous tenants ;
--   * `UPDATE` de la seule colonne `publie_le` ;
--   * **aucun droit sur aucune autre table**, aucun `INSERT`, aucun `DELETE`, pas de `BYPASSRLS`.
--
-- L'immuabilité tient toujours : le déclencheur `evenement_outbox_immuable` s'applique à tous les
-- rôles, et `backend/tests/outbox_immuabilite.rs` le vérifie sous celui-ci comme sous les autres.
--
-- **Écart de numérotation consigné** : `data-model.md` §7 et `tasks.md` T064 attribuaient 0005
-- aux provisions comptables. Cette migration s'intercale parce que la Phase 4 précède la Phase 10
-- et qu'une migration ne s'insère pas après coup — sqlx refuse une version antérieure à une
-- version déjà appliquée. Les provisions prennent donc 0006 ; leur contenu est inchangé.

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'kaya_worker') THEN
        CREATE ROLE kaya_worker LOGIN NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE;
    END IF;
END
$$;

GRANT USAGE ON SCHEMA synchronisation TO kaya_worker;

-- `SELECT ... FOR UPDATE SKIP LOCKED` exige le privilège `UPDATE` sur au moins une colonne : le
-- verrou de ligne est une écriture en puissance. La colonne accordée est celle du marquage, et
-- elle seule.
GRANT SELECT              ON synchronisation.evenement_outbox TO kaya_worker;
GRANT UPDATE (publie_le)  ON synchronisation.evenement_outbox TO kaya_worker;

-- Politique **nommée**, restreinte à un rôle et à une table.
--
-- ## Pourquoi `WITH CHECK (true)` et non `(false)`
--
-- `WITH CHECK (false)` semble le choix prudent — le worker ne crée aucune ligne, autant tout
-- refuser à l'écriture. Il rend en réalité le worker **inopérant** : PostgreSQL évalue
-- `WITH CHECK` sur la **ligne résultante d'un `UPDATE`**, pas seulement sur un `INSERT`. Le
-- marquage `publie_le = now()`, seule raison d'être du rôle, échoue alors sur
-- « new row violates row-level security policy ».
--
-- Ce que le worker ne peut pas faire est donc porté par deux mécanismes plus précis que cette
-- politique :
--
--   * **les privilèges** — aucun `INSERT`, aucun `DELETE`, et `UPDATE` sur la seule colonne
--     `publie_le` : il ne peut ni créer une ligne, ni changer son `tenant_id` ;
--   * **le déclencheur `evenement_outbox_immuable`** — il refuse toute différence entre
--     l'ancienne et la nouvelle ligne autre que le passage de `publie_le` de `NULL` à une
--     valeur, et s'applique à tous les rôles.
--
-- `backend/tests/outbox_immuabilite.rs` vérifie les deux **sous ce rôle précisément** : c'est
-- celui qui voit tous les tenants, donc celui où l'immuabilité serait le plus facilement perdue.
CREATE POLICY publication_worker ON synchronisation.evenement_outbox
    FOR ALL
    TO kaya_worker
    USING      (true)
    WITH CHECK (true);

COMMENT ON POLICY publication_worker ON synchronisation.evenement_outbox IS
    'Worker de publication : lecture tous tenants, marquage publie_le seul. Aucune écriture.';
