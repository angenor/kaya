-- 0014 — CPT : le schéma `comptes`.
--
-- **Une migration appliquée n'est jamais modifiée** (principe I(b), porte P-02). C'est toute la
-- raison de ce fichier : les trois schémas du produit naissent en `0001`, et le réflexe naturel —
-- le seul qui vienne à l'esprit — est d'ajouter le quatrième là où sont les trois autres.
--
-- `0001` est **appliquée**. La porte P-02 compare son empreinte à celle du dépôt et fait échouer
-- le build sur tout écart. Un `CREATE SCHEMA comptes` glissé dans `0001` produirait donc, dans
-- l'ordre : une porte rouge, une tentation de « juste régénérer l'empreinte », et la fin de la
-- garantie que les migrations déjà passées disent la vérité sur les bases déjà déployées
-- (research.md R-11).
--
-- Le schéma naît donc ici, seul, dans la migration la plus courte du produit.

-- =============================================================================================
--  Le quatrième schéma de module
-- =============================================================================================
--
-- Aucune requête ne joint deux schémas de modules (principe II, porte P-04). Ce cycle en porte
-- **trois tentations**, toutes trois refusées et vérifiées : `compte_role.etablissement_id`,
-- `journal_audit.etablissement_id` et `permission.module_code` désignent des objets du schéma
-- `etablissements` sans aucune clé étrangère vers lui. L'existence est vérifiée par les traits
-- `EstablishmentDirectory` et `RegistreModules`, ce qui donne un `404` intelligible au lieu d'une
-- violation de contrainte.
CREATE SCHEMA IF NOT EXISTS comptes;

COMMENT ON SCHEMA comptes IS
    'Module socle/comptes — personnes, comptes, rôles cumulables, journal d''audit. Aucune session : elles vivent en Redis (éphémère reconstructible).';

-- `USAGE` seulement : le droit d'entrer dans le schéma, jamais celui d'y créer. Les objets sont
-- créés par les migrations, sous `kaya_owner`, et par rien d'autre (principe I(b)).
GRANT USAGE ON SCHEMA comptes TO kaya_app;

-- `kaya_ledger_reader` n'obtient RIEN ici, et c'est le point du rôle : la reconstitution
-- autonome doit se faire depuis le seul grand livre. Lui ouvrir `comptes` laisserait le test
-- lire les noms des auteurs au lieu de les retrouver dans la charge utile dénormalisée des
-- événements — et la garantie de TRX-02 cesserait d'être démontrée.
--
-- `kaya_worker` non plus : il ne lit que `synchronisation.evenement_outbox` (migration 0005).
