-- 0007 — ETB-01 : l'identité de l'établissement.
--
-- **Migration additive.** `0002_etablissements_socle.sql` n'est pas touchée — elle l'annonçait
-- elle-même : « ETB-01 les enrichira par migration additive, jamais en modifiant ce fichier »
-- (principe I(b), porte P-02).
--
-- =============================================================================================
--  LE PIÈGE QUI DÉCIDE DE LA FORME DE CETTE MIGRATION
-- =============================================================================================
--
-- **`ADD COLUMN ... NOT NULL DEFAULT`, jamais `ADD COLUMN` puis `UPDATE`.**
--
-- `etablissement` est en `FORCE ROW LEVEL SECURITY` et cette migration s'exécute sous
-- `kaya_owner`. Un `UPDATE` de migration est donc soumis à la politique `isolation_tenant`, qui
-- compare `current_setting('app.current_tenant', true)` — NULL hors requête applicative. La
-- comparaison vaut NULL, **aucune ligne n'est touchée, et aucune erreur n'est levée** : la
-- migration réussit en n'écrivant rien, et le défaut se découvre au premier calcul de taxe.
--
-- `ADD COLUMN ... DEFAULT` est du **DDL** : il remplit les lignes existantes sans passer par
-- aucune politique. C'est la seule forme qui écrit réellement ici.
--
-- **Aucun `INSERT` ni `UPDATE` dans ce fichier**, pour la même raison. Ce qui doit être écrit
-- passe par le DDL ou par la mécanique de seeds, qui pose le tenant courant (research.md R-08,
-- règle générale reportée dans docs/module-dore.md).

-- =============================================================================================
--  Sept colonnes d'identité
-- =============================================================================================

-- Sélectionne le `JurisdictionAdapter` ; **n'encode aucune règle** (principe V). Un seul
-- adaptateur au MVP (cadrage §14.1), d'où le défaut permanent — il n'est pas retiré.
ALTER TABLE etablissements.etablissement
    ADD COLUMN juridiction TEXT NOT NULL DEFAULT 'CI';

-- Détermine le barème de la taxe communale de nuitée (cadrage §9.6). Vocabulaire fiscal
-- officiel conservé tel quel à l'écran (docs/design/lexique.md, règle 2).
ALTER TABLE etablissements.etablissement
    ADD COLUMN classement TEXT NOT NULL DEFAULT 'NON_CLASSE'
    CONSTRAINT etablissement_classement_connu
        CHECK (classement IN ('ETOILES', 'NON_CLASSE', 'RESIDENCE_MEUBLEE'));

-- Le nombre d'étoiles n'existe QUE pour le classement par étoiles, et l'égalité de conditions
-- l'impose **dans les deux sens** : ni étoiles sans classement étoilé, ni classement étoilé sans
-- étoiles. Un `CHECK` à sens unique laisserait passer l'une des deux incohérences.
ALTER TABLE etablissements.etablissement
    ADD COLUMN etoiles SMALLINT NULL;

ALTER TABLE etablissements.etablissement
    ADD CONSTRAINT etablissement_etoiles_coherentes
        CHECK ((classement = 'ETOILES') = (etoiles IS NOT NULL));

-- **Aucun plafond en base — porte P-12.** Le nombre maximal d'étoiles est fixé par la
-- réglementation nationale, donc par le `JurisdictionAdapter`. Un `BETWEEN 1 AND 5` serait une
-- règle de juridiction déguisée en contrainte d'intégrité : le jour où un pays en reconnaît six,
-- il faudrait une migration pour un changement qui n'en demande pas. Ne reste ici que ce qui
-- vaut sous toute juridiction — un nombre d'étoiles est strictement positif.
ALTER TABLE etablissements.etablissement
    ADD CONSTRAINT etablissement_etoiles_positives
        CHECK (etoiles IS NULL OR etoiles > 0);

-- Commune de rattachement — assiette du reversement communal. Le défaut n'a pas de sens
-- permanent : il ne sert qu'à remplir les lignes existantes, puis il est retiré pour qu'une
-- création ultérieure ne puisse pas l'omettre en silence.
ALTER TABLE etablissements.etablissement
    ADD COLUMN commune TEXT NOT NULL DEFAULT '';
ALTER TABLE etablissements.etablissement
    ALTER COLUMN commune DROP DEFAULT;

-- Absente au provisionnement, renseignée ensuite. Pas de défaut : `NULL` dit « pas encore
-- saisie », `''` dirait « saisie vide » — deux états différents qu'il ne faut pas confondre.
ALTER TABLE etablissements.etablissement
    ADD COLUMN adresse TEXT NULL;

-- Numéro de compte contribuable. **Le contrôle de forme est volontairement minimal** : la
-- validité d'un NCC est une règle de juridiction, que le principe V confine au
-- `JurisdictionAdapter` (porte P-12). Une expression régulière ici serait une règle fiscale en
-- base — et la première juridiction au format différent imposerait une migration.
ALTER TABLE etablissements.etablissement
    ADD COLUMN ncc TEXT NULL;

ALTER TABLE etablissements.etablissement
    ADD CONSTRAINT etablissement_ncc_non_vide
        CHECK (ncc IS NULL OR length(btrim(ncc)) > 0);

COMMENT ON COLUMN etablissements.etablissement.juridiction IS
    'Sélectionne le JurisdictionAdapter. N''encode aucune règle fiscale (principe V, porte P-12).';
COMMENT ON COLUMN etablissements.etablissement.classement IS
    'ETOILES | NON_CLASSE | RESIDENCE_MEUBLEE — détermine le barème de la taxe de nuitée.';
COMMENT ON COLUMN etablissements.etablissement.etoiles IS
    'Nombre d''étoiles. AUCUN plafond en base : le maximum est une règle de juridiction (P-12).';
COMMENT ON COLUMN etablissements.etablissement.ncc IS
    'Numéro de compte contribuable. Forme non vérifiée ici : règle de juridiction (P-12).';

-- Aucun changement de privilège ni de politique : la table les porte depuis 0002, et une
-- colonne ajoutée en hérite. Aucun `ENABLE`/`FORCE` à répéter — les relancer serait sans effet,
-- mais laisserait croire qu'une table enrichie doit être re-sécurisée.
