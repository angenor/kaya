-- 0001 — Rôles, schémas de modules et extensions.
--
-- **Une migration appliquée n'est jamais modifiée** (principe I(b), porte P-02). Toute
-- correction se fait par une migration nouvelle. Ce fichier est donc écrit pour être définitif.

-- =============================================================================================
--  1. Les TROIS rôles — et pourquoi ils sont trois, pas deux
-- =============================================================================================
--
-- La constitution (principe III) en exige deux : le propriétaire des tables et le rôle
-- applicatif. Le troisième, `kaya_ledger_reader`, est ce qui transforme le test de
-- reconstitution autonome (FR-042) d'une déclaration en une démonstration.
--
-- Un test qui « n'interroge pas les autres tables » par discipline de rédaction ne prouve rien :
-- il suffit d'un JOIN ajouté six mois plus tard pour que la garantie disparaisse sans qu'aucune
-- alerte ne se déclenche. Un rôle qui n'a **pas le droit** de lire les autres tables fait échouer
-- ce JOIN à la seconde où il est écrit.
--
-- Aucun mot de passe n'est posé ici. Un secret dans une migration est un secret dans l'historique
-- Git, en clair, pour toujours. Les mots de passe sont posés hors du dépôt :
-- `scripts/dev/preparer-base.sh` pour le poste de développement et la CI, un secret
-- d'exploitation en production.

DO $$
BEGIN
    -- `kaya_owner` préexiste normalement : c'est sous ce rôle que la migration s'exécute
    -- (R-12). Le bloc est là pour le déploiement auto-hébergé, où l'administrateur peut
    -- l'avoir créé autrement, et pour que rejouer la migration ne casse rien.
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'kaya_owner') THEN
        -- NOSUPERUSER et NOBYPASSRLS ne sont pas des précautions de style : un superutilisateur
        -- contourne TOUTE politique de sécurité au niveau ligne, `FORCE` comprise. Un
        -- `kaya_owner` superutilisateur rendrait `isolation_tenant` décorative, et la porte P-07
        -- passerait en constatant un drapeau vrai pendant que l'isolation serait ouverte.
        CREATE ROLE kaya_owner LOGIN NOSUPERUSER NOBYPASSRLS CREATEROLE;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'kaya_app') THEN
        CREATE ROLE kaya_app LOGIN NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'kaya_ledger_reader') THEN
        CREATE ROLE kaya_ledger_reader LOGIN NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE;
    END IF;
END
$$;

-- =============================================================================================
--  2. Un schéma PostgreSQL par module (principe II)
-- =============================================================================================
--
-- Aucune requête ne joint deux de ces schémas ; les lectures inter-modules passent par un trait
-- exposé (`contracts/traits-exposes.md`). La porte P-04 le vérifie sur les fichiers SQL et les
-- macros `query!`.
--
-- Trois schémas à ce cycle. Les autres naissent avec leur module — jamais d'avance : un schéma
-- vide invite à y poser une table « en attendant ».

CREATE SCHEMA IF NOT EXISTS etablissements;
CREATE SCHEMA IF NOT EXISTS synchronisation;
CREATE SCHEMA IF NOT EXISTS fiscalite;

COMMENT ON SCHEMA etablissements IS
    'Module socle/etablissements — tenants, établissements, notes internes.';
COMMENT ON SCHEMA synchronisation IS
    'Module socle/synchronisation — grand livre d''événements. Rétention illimitée, immuable.';
COMMENT ON SCHEMA fiscalite IS
    'Module socle/fiscalite — obligations réglementaires et provisions comptables (TRX-02b).';

-- `USAGE` seulement : le droit d'entrer dans le schéma, jamais celui d'y créer. Les objets sont
-- créés par les migrations, sous `kaya_owner`, et par rien d'autre (principe I(b)).
GRANT USAGE ON SCHEMA etablissements  TO kaya_app;
GRANT USAGE ON SCHEMA synchronisation TO kaya_app;
GRANT USAGE ON SCHEMA fiscalite       TO kaya_app;

-- `kaya_ledger_reader` n'obtient l'accès qu'au seul schéma du grand livre. C'est **l'absence**
-- des deux autres qui fait la valeur du rôle.
GRANT USAGE ON SCHEMA synchronisation TO kaya_ledger_reader;

-- =============================================================================================
--  3. btree_gist — prérequis des contraintes d'exclusion
-- =============================================================================================
--
-- `EXCLUDE USING gist (tenant_id WITH =, plage WITH &&)` mêle un opérateur d'égalité sur un type
-- scalaire et un opérateur de chevauchement sur un type d'intervalle. GiST ne sait pas indexer le
-- premier sans cette extension.
--
-- Elle est posée **au cycle 001**, alors qu'aucune occupation n'existe encore, parce que
-- `fiscalite.exercice_comptable` s'en sert (data-model §5) et que c'est là le premier usage du
-- produit — le spike de HEB-02, qui en dépendra sur `tstzrange`.
--
-- `btree_gist` fait partie des extensions « trusted » de PostgreSQL : `kaya_owner`, propriétaire
-- de la base mais non superutilisateur, peut donc l'installer.
CREATE EXTENSION IF NOT EXISTS btree_gist;
