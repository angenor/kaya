-- 0021 — HEB : le schéma `hebergement`.
--
-- **Une migration dédiée**, exactement comme `0014_schema_comptes.sql` au cycle 003, et pour la
-- même raison : les trois premiers schémas naissent en `0001`, et le réflexe naturel est
-- d'ajouter le cinquième là où sont les autres. `0001` est **appliquée** ; la porte P-02 compare
-- son empreinte à celle du dépôt et fait échouer le build sur tout écart.
--
-- Le schéma naît donc ici, seul.
--
-- **`btree_gist` n'est PAS réinstallée.** `0001_roles_et_schemas.sql:93` l'a posée, elle est
-- globale à la base, et son en-tête dit pourquoi : elle a été installée au cycle 001 alors
-- qu'aucune occupation n'existait, précisément pour ce moment. Sans elle,
-- `EXCLUDE USING gist (unite_id WITH =, ...)` échouerait — GiST ne sait pas indexer l'égalité
-- sur un UUID sans l'extension.

-- =============================================================================================
--  Le cinquième schéma de module — et le premier d'une VERTICALE
-- =============================================================================================
--
-- Aucune requête ne joint deux schémas de modules (principe II, porte P-04). Ce cycle en porte
-- **trois tentations**, toutes trois refusées et vérifiées : `categorie.etablissement_id`,
-- `unite.tenant_id` et `formule.etablissement_id` désignent des objets du schéma `etablissements`
-- **sans aucune clé étrangère vers lui**. L'existence est vérifiée par le trait
-- `EstablishmentDirectory`, ce qui donne un `404` intelligible au lieu d'une violation de
-- contrainte — et le fuseau horaire, dont ce cycle a besoin pour convertir les plages de
-- demi-journée en instants, se lit par ce même trait.
CREATE SCHEMA IF NOT EXISTS hebergement;

COMMENT ON SCHEMA hebergement IS
    'Module verticales/hebergement — unités louables, formules de location, moteur de disponibilité. Le socle ne connaît ni « chambre », ni « unité louable », ni « séjour » : tout le spécifique hôtelier vit ici.';

-- `USAGE` seulement : le droit d'entrer dans le schéma, jamais celui d'y créer. Les objets sont
-- créés par les migrations, sous `kaya_owner`, et par rien d'autre (principe I(b)).
GRANT USAGE ON SCHEMA hebergement TO kaya_app;

-- `kaya_ledger_reader` n'obtient RIEN ici, et c'est le point du rôle : la reconstitution
-- autonome se fait depuis le seul grand livre. Lui ouvrir `hebergement` laisserait le test lire
-- les unités au lieu de les retrouver dans la charge utile dénormalisée des événements.
--
-- `kaya_worker` non plus : il ne lit que `synchronisation.evenement_outbox` (migration 0005).
