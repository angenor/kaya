-- Amorçage du rôle propriétaire — exécuté UNE SEULE FOIS, à la création du volume de données.
--
-- Pourquoi ce fichier existe alors que le principe I(b) réserve toute modification de schéma aux
-- migrations sqlx : un **rôle** n'est pas un objet de schéma. Il doit préexister à la première
-- migration, puisque c'est sous ce rôle que les migrations s'exécutent. Les schémas, les tables
-- et les politiques, eux, ne sont créés que par `backend/migrations/`, sans exception.
--
-- En déploiement auto-hébergé (mode B), l'administrateur exécute ces trois ordres à la main
-- avant le premier démarrage. C'est la seule intervention manuelle de la mise en service.

-- `NOSUPERUSER` est la ligne la plus importante du fichier.
--
-- PostgreSQL laisse tout superutilisateur contourner la sécurité au niveau ligne, `FORCE`
-- comprise. Un `kaya_owner` superutilisateur rendrait la politique `isolation_tenant`
-- décorative : le test P-07 constaterait `relforcerowsecurity = true` et passerait, pendant
-- qu'en pratique le rôle des migrations verrait toutes les lignes de tous les clients.
--
-- `CREATEROLE` lui est en revanche nécessaire : c'est la migration 0001 qui crée `kaya_app` et
-- `kaya_ledger_reader`.
CREATE ROLE kaya_owner
    LOGIN
    NOSUPERUSER
    NOBYPASSRLS
    CREATEROLE
    PASSWORD 'motdepasse_dev';

-- Propriétaire de la base : sans cela, `kaya_owner` ne peut pas créer de schéma.
ALTER DATABASE kaya OWNER TO kaya_owner;

-- Le schéma `public` reste propriété de `postgres` par défaut ; les migrations n'y créent rien,
-- mais la table de suivi des migrations doit pouvoir s'y poser si `sqlx.toml` ne la déplace pas.
GRANT ALL ON SCHEMA public TO kaya_owner;
