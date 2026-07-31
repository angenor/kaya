-- 0012 — CORRECTION : l'unicité de séquence de l'outbox porte sur la CLÉ DE SÉQUENCE.
--
-- =============================================================================================
--  Le défaut, et pourquoi le cycle 001 ne pouvait pas le voir
-- =============================================================================================
--
-- `0003_outbox.sql` déclarait :
--
--     UNIQUE NULLS NOT DISTINCT (etablissement_id, sequence_etablissement)
--
-- avec ce commentaire, juste dans son intention : « `NULLS NOT DISTINCT` étend l'unicité aux
-- événements de niveau tenant. Sans lui, deux lignes à `etablissement_id` NULL portant la même
-- séquence seraient acceptées. »
--
-- Ce que le raisonnement manquait, c'est **d'où vient la séquence**. `PgOutboxWriter` la tire sur
-- `prochaine_sequence(etablissement_id ou tenant_id)` : pour un événement de niveau tenant, le
-- compteur est celui du **tenant**. Deux tenants distincts obtiennent donc chacun leur séquence 1,
-- et écrivent tous deux `(NULL, 1)`. `NULLS NOT DISTINCT` les rend égaux — collision.
--
-- **Le cycle 001 ne pouvait pas le rencontrer** : tous ses événements portaient un
-- `etablissement_id`. Le premier événement de niveau tenant du produit est
-- `parametre_configuration.ecrit` à la portée `TENANT`, livré par ETB-04 — et il a échoué au
-- deuxième tenant, dans les trois parcours structurels.
--
-- =============================================================================================
--  La correction
-- =============================================================================================
--
-- L'unicité doit porter sur **la même clé que la séquence** : `coalesce(etablissement_id,
-- tenant_id)`. C'est exactement ce que calcule `PgOutboxWriter`, et l'écrire ici met la contrainte
-- et le producteur en accord — au lieu de les laisser diverger sur un cas que rien n'exerçait.
--
-- Un index unique sur expression plutôt qu'une contrainte de table : PostgreSQL n'accepte pas
-- d'expression dans `UNIQUE (...)`. La garantie est identique.
--
-- **Aucune donnée n'est supprimée** (porte P-05b) : l'ancienne contrainte est remplacée, les
-- lignes restent. Le grand livre demeure à rétention illimitée et immuable.

ALTER TABLE synchronisation.evenement_outbox
    DROP CONSTRAINT evenement_outbox_sequence_unique;

CREATE UNIQUE INDEX evenement_outbox_sequence_unique
    ON synchronisation.evenement_outbox (coalesce(etablissement_id, tenant_id), sequence_etablissement);

COMMENT ON INDEX synchronisation.evenement_outbox_sequence_unique IS
    'Unicité de séquence sur la CLÉ DE SÉQUENCE — coalesce(etablissement_id, tenant_id), la même que celle de prochaine_sequence(). Corrige 0003, où NULLS NOT DISTINCT faisait partager un espace de numérotation à tous les événements de niveau tenant, tous tenants confondus.';
