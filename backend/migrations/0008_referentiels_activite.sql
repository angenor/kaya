-- 0008 — ETB-02 / ETB-02b : les quatre référentiels globaux.
--
-- `module_activite` (la verticale), `capacite` (le transverse), `profil_stock` (le profil de la
-- seule capacité implémentée) et `parametre_catalogue` (les clés de configuration connues).
--
-- **Module d'activité ≠ capacité.** Deux référentiels distincts, jamais fusionnés ni dérivés
-- l'un de l'autre (FR-030) : le module est ce que l'établissement FAIT, la capacité est ce dont
-- il a BESOIN pour le faire. Un maquis fait de la restauration et a besoin de suivre son stock ;
-- une résidence meublée fait de l'hébergement et n'a besoin de rien de tel.
--
-- =============================================================================================
--  EXCEPTION NOMMÉE — ces quatre tables n'ont PAS de `tenant_id`
-- =============================================================================================
--
-- Elles sont **le même référentiel pour tous les clients**. C'est la seconde exception du
-- produit, après `tenant` au cycle 001 (« seule table dont la colonne comparée est sa propre
-- clé »), et elle est écrite ici pour la même raison : une exception écrite est relisible, une
-- exception silencieuse devient un précédent.
--
-- Le régime n'est donc pas une dispense de la porte P-07, c'est un **régime nommé** — deux
-- politiques et un jeu de privilèges asymétrique (research.md R-01) :
--
--   `lecture_universelle`     le référentiel est le même pour tous : `FOR SELECT USING (true)`.
--   `administration_editeur`  l'écriture appartient à l'éditeur (ETB-08), donc au propriétaire.
--   `GRANT SELECT` seul       `kaya_app` est refusé DEUX FOIS : aucun privilège d'écriture, et
--                             aucune politique qui l'autoriserait. Un `GRANT` accordé par erreur
--                             plus tard ne suffirait donc pas à ouvrir la table.
--
-- **ORDRE IMPÉRATIF : `CREATE TABLE` → `INSERT` → `ENABLE`/`FORCE` → `CREATE POLICY`.**
-- `FORCE` applique les politiques au propriétaire lui-même. Insérer après l'avoir activé, mais
-- avant d'avoir créé `administration_editeur`, échouerait — le propriétaire n'aurait alors aucune
-- politique d'écriture. Les valeurs entrent donc quand la table n'est encore gardée par rien.
--
-- =============================================================================================
--  `libelle_cle` porte une CLÉ i18n, jamais un libellé
-- =============================================================================================
--
-- Une chaîne utilisateur stockée en base échapperait à la porte P-16 — elle n'aurait ni parité
-- fr/en, ni relecture de vocabulaire, et le premier écran l'afficherait telle quelle. Ce que ces
-- colonnes portent est un **identifiant de traduction** ; le texte vit dans
-- `app/core/i18n/{fr,en}.json`, sous le contrôle du lexique.

-- =============================================================================================
--  module_activite — LA VERTICALE
-- =============================================================================================
CREATE TABLE etablissements.module_activite (
    code        TEXT     PRIMARY KEY,

    -- Support de la clé étrangère composite de `etablissement_module` (migration 0009). C'est
    -- cette colonne, recopiée par la table qui la référence, qui rend le refus **structurel**.
    implementee BOOLEAN  NOT NULL,

    libelle_cle TEXT     NOT NULL,

    -- Ordre d'affichage stable, indépendant de l'alphabet et de la locale. Trier sur le libellé
    -- traduit ferait changer l'ordre de l'écran en passant du français à l'anglais.
    ordre       SMALLINT NOT NULL,

    UNIQUE (code, implementee)
);

COMMENT ON TABLE etablissements.module_activite IS
    'Référentiel GLOBAL des modules d''activité (la verticale). Sans tenant_id — exception nommée. Classe hors-ligne C en écriture, A en lecture cachée.';

-- Les cinq modules du MVP, tous implémentés. L'ajout de `SPA` ou `QUINCAILLERIE` (ETB-08,
-- provision) sera un `INSERT` avec `implementee = false` : la valeur existe au référentiel et
-- **reste inactivable** tant que le drapeau n'est pas levé — une écriture de configuration, pas
-- une migration (cadrage §14.3).
INSERT INTO etablissements.module_activite (code, implementee, libelle_cle, ordre) VALUES
    ('HEBERGEMENT',   true, 'services.modules.HEBERGEMENT',   10),
    ('RESTAURATION',  true, 'services.modules.RESTAURATION',  20),
    ('BAR',           true, 'services.modules.BAR',           30),
    ('PRESSING',      true, 'services.modules.PRESSING',      40),
    ('SALLE_REUNION', true, 'services.modules.SALLE_REUNION', 50);

-- =============================================================================================
--  capacite — LE TRANSVERSE
-- =============================================================================================
CREATE TABLE etablissements.capacite (
    code        TEXT    PRIMARY KEY,
    implementee BOOLEAN NOT NULL,
    libelle_cle TEXT    NOT NULL,
    ordre       SMALLINT NOT NULL,

    UNIQUE (code, implementee)
);

COMMENT ON TABLE etablissements.capacite IS
    'Référentiel GLOBAL des capacités transverses. Seule STOCK est implémentée au MVP. Classe hors-ligne C.';

-- **Une seule capacité implémentée, six déclarées et refusées.** Les six autres ne sont pas
-- absentes du référentiel : elles y figurent avec `implementee = false`, ce qui permet au refus
-- de distinguer « connu mais non implémenté » de « inconnu » — distinction qu'un `CHECK ... IN`
-- littéral ne saurait pas faire, et qui change le message rendu à l'exploitant.
INSERT INTO etablissements.capacite (code, implementee, libelle_cle, ordre) VALUES
    ('STOCK',             true,  'services.capacites.STOCK',             10),
    ('LIVRAISON',         false, 'services.capacites.LIVRAISON',          20),
    ('PRODUCTION',        false, 'services.capacites.PRODUCTION',         30),
    ('COMMERCE_EN_LIGNE', false, 'services.capacites.COMMERCE_EN_LIGNE',  40),
    ('FIDELITE',          false, 'services.capacites.FIDELITE',           50),
    ('DEVIS',             false, 'services.capacites.DEVIS',              60),
    ('COMPTES_CLIENTS',   false, 'services.capacites.COMPTES_CLIENTS',    70);

-- =============================================================================================
--  profil_stock — le profil de la seule capacité implémentée
-- =============================================================================================
--
-- Une quatrième table plutôt qu'un `CHECK ... IN ('AUCUN','SIMPLE','VALORISE','DETAILLE')`
-- (research.md R-03) : ouvrir un profil doit être une écriture de configuration, pas une
-- migration (cadrage §14.5).
CREATE TABLE etablissements.profil_stock (
    code            TEXT     PRIMARY KEY,
    implementee     BOOLEAN  NOT NULL,
    libelle_cle     TEXT     NOT NULL,

    -- Clé i18n du message expliquant le refus. `NULL` pour un profil implémenté : il n'y a rien
    -- à expliquer.
    motif_refus_cle TEXT     NULL,
    ordre           SMALLINT NOT NULL,

    UNIQUE (code, implementee),

    CONSTRAINT profil_stock_motif_si_refuse
        CHECK (implementee = (motif_refus_cle IS NULL))
);

COMMENT ON TABLE etablissements.profil_stock IS
    'Référentiel GLOBAL des profils de la capacité STOCK. Seul SIMPLE est implémenté. Classe hors-ligne C.';

-- **Le motif d'`AUCUN` est distinct des deux autres, et c'est le seul refus qui enseigne quelque
-- chose.** `VALORISE` et `DETAILLE` sont des fonctionnalités absentes du MVP — on annonce une
-- absence. `AUCUN` n'est pas une fonctionnalité manquante : c'est une demande qui n'a pas de
-- sens, puisqu'une capacité qu'on ne consomme pas ne se déclare simplement pas. Répondre
-- « profil non implémenté » à cette demande enverrait attendre une version future une personne
-- qui doit juste ne rien faire.
INSERT INTO etablissements.profil_stock (code, implementee, libelle_cle, motif_refus_cle, ordre) VALUES
    ('SIMPLE',    true,  'services.profils.SIMPLE',    NULL,                                   10),
    ('AUCUN',     false, 'services.profils.AUCUN',     'services.refus.profil.AUCUN',          20),
    ('VALORISE',  false, 'services.profils.VALORISE',  'services.refus.profil.VALORISE',       30),
    ('DETAILLE',  false, 'services.profils.DETAILLE',  'services.refus.profil.DETAILLE',       40);

-- =============================================================================================
--  parametre_catalogue — les clés de configuration connues
-- =============================================================================================
--
-- Sans catalogue, `parametre_configuration` serait un `JSONB` sans validation, sans type et sans
-- découvrabilité : rien n'empêcherait une clé mal orthographiée ni un montant en flottant, et le
-- « récapitulatif des paramètres fait foi » du principe I(c) resterait décoratif. Le catalogue est
-- ce qui rend ce principe **vérifiable par un test**
-- (`backend/tests/parametres_catalogue.rs`).
--
-- **La table est créée ici, avec les autres référentiels ; elle est PEUPLÉE par 0011**, avec la
-- chaîne d'héritage qui la consomme. La politique `administration_editeur` posée ci-dessous rend
-- cet `INSERT` ultérieur possible sous le propriétaire.
CREATE TABLE etablissements.parametre_catalogue (
    cle                 TEXT PRIMARY KEY,

    -- **Liste fermée assumée.** Contrairement aux capacités, un type de valeur n'est pas une
    -- fonctionnalité produit : son ajout touche le code de validation, donc il mérite une
    -- migration. L'ouvrir par référentiel donnerait l'illusion qu'un type se déclare sans écrire
    -- la validation qui va avec.
    type_valeur         TEXT NOT NULL
        CONSTRAINT parametre_catalogue_type_connu
            CHECK (type_valeur IN ('ENTIER', 'TEXTE', 'BOOLEEN', 'DUREE_MINUTES',
                                   'MONTANT_MINEUR', 'HEURE_LOCALE', 'BAREME')),

    -- Jusqu'où la surcharge peut descendre. Le tenant est toujours autorisé comme racine.
    portee_la_plus_basse TEXT NOT NULL
        CONSTRAINT parametre_catalogue_portee_connue
            CHECK (portee_la_plus_basse IN ('TENANT', 'ETABLISSEMENT', 'MODULE', 'POINT_DE_VENTE')),

    -- Traçabilité vers le « Récapitulatif des paramètres d'établissement » de
    -- docs/user-stories-v1.md. C'est cette colonne que la porte de cohérence documentaire lit.
    story               TEXT NOT NULL,

    libelle_cle         TEXT NOT NULL,
    description_cle     TEXT NOT NULL
);

COMMENT ON TABLE etablissements.parametre_catalogue IS
    'Référentiel GLOBAL des clés de configuration connues. Peuplé par 0011. Classe hors-ligne C.';
COMMENT ON COLUMN etablissements.parametre_catalogue.type_valeur IS
    'MONTANT_MINEUR impose une valeur JSONB ENTIÈRE — extension de la porte P-10 au JSONB.';

-- =============================================================================================
--  Sécurité au niveau ligne — le régime nommé des référentiels globaux
-- =============================================================================================
--
-- Les INSERT ci-dessus sont déjà passés : la table n'était encore gardée par rien. C'est l'ordre
-- décrit en tête, et il n'est pas interchangeable.

ALTER TABLE etablissements.module_activite      ENABLE ROW LEVEL SECURITY;
ALTER TABLE etablissements.module_activite      FORCE  ROW LEVEL SECURITY;
ALTER TABLE etablissements.capacite             ENABLE ROW LEVEL SECURITY;
ALTER TABLE etablissements.capacite             FORCE  ROW LEVEL SECURITY;
ALTER TABLE etablissements.profil_stock         ENABLE ROW LEVEL SECURITY;
ALTER TABLE etablissements.profil_stock         FORCE  ROW LEVEL SECURITY;
ALTER TABLE etablissements.parametre_catalogue  ENABLE ROW LEVEL SECURITY;
ALTER TABLE etablissements.parametre_catalogue  FORCE  ROW LEVEL SECURITY;

CREATE POLICY lecture_universelle ON etablissements.module_activite
    FOR SELECT USING (true);
CREATE POLICY administration_editeur ON etablissements.module_activite
    FOR ALL TO kaya_owner USING (true) WITH CHECK (true);

CREATE POLICY lecture_universelle ON etablissements.capacite
    FOR SELECT USING (true);
CREATE POLICY administration_editeur ON etablissements.capacite
    FOR ALL TO kaya_owner USING (true) WITH CHECK (true);

CREATE POLICY lecture_universelle ON etablissements.profil_stock
    FOR SELECT USING (true);
CREATE POLICY administration_editeur ON etablissements.profil_stock
    FOR ALL TO kaya_owner USING (true) WITH CHECK (true);

CREATE POLICY lecture_universelle ON etablissements.parametre_catalogue
    FOR SELECT USING (true);
CREATE POLICY administration_editeur ON etablissements.parametre_catalogue
    FOR ALL TO kaya_owner USING (true) WITH CHECK (true);

-- `SELECT` **et rien d'autre**. Le privilège dit la règle : l'enrichissement du référentiel
-- relève de l'éditeur (ETB-08), aucun tenant n'y écrit.
GRANT SELECT ON etablissements.module_activite     TO kaya_app;
GRANT SELECT ON etablissements.capacite            TO kaya_app;
GRANT SELECT ON etablissements.profil_stock        TO kaya_app;
GRANT SELECT ON etablissements.parametre_catalogue TO kaya_app;
