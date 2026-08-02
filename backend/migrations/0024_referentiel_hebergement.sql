-- 0024 — HEB-01, HEB-03, HEB-04, HEB-05 : le référentiel de l'offre. Six tables, **classe C**.
--
-- Ce que l'exploitant règle une fois puis ajuste à la marge : ses types de chambre, ses chambres,
-- ce qu'il vend dessus, à quel prix, et sur quelles plages. Rien de ce fichier n'est atteignable
-- hors ligne — c'est ce que la classe C dit, et les privilèges le confirment (module doré, « les
-- privilèges disent la classe »).
--
-- **Aucune clé étrangère ne sort du schéma `hebergement`.** `etablissement_id` et `tenant_id` sont
-- des UUID sans `REFERENCES` : une clé vers `etablissements.etablissement` serait une clé
-- inter-schémas, interdite par le principe II et la porte P-04. La cohérence passe par
-- `EstablishmentDirectory`, ce qui rend un `404` intelligible au lieu d'une violation de
-- contrainte.

-- =============================================================================================
--  1. categorie — un groupe d'unités homogènes
-- =============================================================================================
CREATE TABLE hebergement.categorie (
    id               UUID        PRIMARY KEY,
    tenant_id        UUID        NOT NULL,
    etablissement_id UUID        NOT NULL,

    nom              TEXT        NOT NULL,

    -- **`capacite_accueil`, jamais `capacite`.** Le lexique réserve « capacité » au transverse
    -- (stock, livraison, fidélité) et écrit qu'il « n'apparaît jamais » à l'utilisateur. Nommer
    -- cette colonne `capacite` créerait deux sens pour un mot déjà chargé, dans deux schémas
    -- voisins — et le jour où quelqu'un chercherait « la capacité d'un établissement », il
    -- trouverait des lits.
    capacite_accueil SMALLINT    NOT NULL
        CONSTRAINT categorie_capacite_positive CHECK (capacite_accueil > 0),

    cree_le          TIMESTAMPTZ NOT NULL DEFAULT now(),
    modifie_le       TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT categorie_nom_unique UNIQUE (etablissement_id, nom)
);

COMMENT ON TABLE hebergement.categorie IS
    'Un groupe d''unités homogènes. Terme utilisateur : « type de chambre » (lexique). Classe hors-ligne C.';

-- =============================================================================================
--  2. temps_remise_en_etat — une TABLE, pas une colonne
-- =============================================================================================
--
-- HEB-01 écrit `categorie {…, temps_remise_en_etat_par_formule}` : le battement varie par
-- catégorie **ET** par famille de formule. Une suite qui se rangerait en colonne perdrait l'un
-- des deux axes.
CREATE TABLE hebergement.temps_remise_en_etat (
    categorie_id    UUID     NOT NULL REFERENCES hebergement.categorie (id) ON DELETE CASCADE,

    famille_formule TEXT     NOT NULL
        CONSTRAINT temps_remise_famille_connue
            CHECK (famille_formule IN ('NUITEE', 'PASSAGE', 'DEMI_JOURNEE', 'MENSUEL')),

    -- **`>= 0` et non `> 0`.** Une catégorie peut légitimement n'avoir aucun battement — une
    -- salle de réunion qu'on n'aère pas entre deux réunions. Zéro est une valeur, pas une absence.
    duree_minutes   INTEGER  NOT NULL
        CONSTRAINT temps_remise_duree_non_negative CHECK (duree_minutes >= 0),

    -- Porté par la table fille bien qu'il soit dérivable du parent : une politique de sécurité
    -- qui devrait joindre `categorie` pour trouver le tenant serait plus lente et plus fragile.
    tenant_id       UUID     NOT NULL,

    PRIMARY KEY (categorie_id, famille_formule)
);

COMMENT ON TABLE hebergement.temps_remise_en_etat IS
    'Battement entre deux occupations, par catégorie ET par famille de formule. Classe hors-ligne C, sur le régime de sa catégorie.';

-- =============================================================================================
--  3. unite — une chambre, un logement, une salle
-- =============================================================================================
CREATE TABLE hebergement.unite (
    id               UUID        PRIMARY KEY,
    tenant_id        UUID        NOT NULL,
    etablissement_id UUID        NOT NULL,
    categorie_id     UUID        NOT NULL REFERENCES hebergement.categorie (id),

    code             TEXT        NOT NULL,

    -- Nul pour une salle en rez-de-chaussée non numéroté. `NULL` dit « pas d'étage », `0` dirait
    -- « rez-de-chaussée » : deux faits différents, deux valeurs.
    etage            SMALLINT    NULL,

    -- **Colonne seule — aucun endpoint ne l'écrit à ce cycle.** Le sous-statut de ménage est de
    -- classe A (dernier-écrit-gagne, seul cas du produit) et relève de HEB-06. La colonne existe
    -- parce que la disponibilité la lit ; l'écriture viendra avec son écran (principe X).
    statut_menage    TEXT        NOT NULL DEFAULT 'propre'
        CONSTRAINT unite_statut_menage_connu
            CHECK (statut_menage IN ('a_nettoyer', 'propre', 'maintenance')),

    cree_le          TIMESTAMPTZ NOT NULL DEFAULT now(),
    modifie_le       TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT unite_code_unique UNIQUE (etablissement_id, code)
);

COMMENT ON TABLE hebergement.unite IS
    'Une chambre, un logement, une salle. AUCUNE colonne statut_occupation : il est DÉRIVÉ des occupations. Classe hors-ligne C.';
COMMENT ON COLUMN hebergement.unite.statut_menage IS
    'Classe A — dernier-écrit-gagne, seul cas du produit. Aucun endpoint ne l''écrit avant HEB-06.';

-- ---------------------------------------------------------------------------------------------
--  AUCUNE COLONNE `statut_occupation`, et c'est le point le plus important de cette table
-- ---------------------------------------------------------------------------------------------
--
-- Le statut d'occupation (libre / occupée / réservée) est **dérivé** des occupations
-- (cadrage §11.4, registre §7.2). L'inscrire en table rendrait possible de le poser à la main —
-- ce que le cadrage désigne nommément comme la cause des doubles attributions.
--
-- Une relecture ultérieure qui le chercherait doit trouver ce paragraphe plutôt que le vide.

-- =============================================================================================
--  4. formule — ce qu'on vend sur une catégorie
-- =============================================================================================
CREATE TABLE hebergement.formule (
    id                               UUID        PRIMARY KEY,
    tenant_id                        UUID        NOT NULL,
    etablissement_id                 UUID        NOT NULL,

    -- **La formule est attachée à la CATÉGORIE, jamais au type d'établissement** (FR-017,
    -- FR-019). C'est ce qui permet à une résidence de vendre du passage si elle le veut, et à un
    -- hôtel de vendre au mois. L'offre suit l'établissement, elle n'est pas un gabarit à remplir.
    categorie_id                     UUID        NOT NULL REFERENCES hebergement.categorie (id),

    famille                          TEXT        NOT NULL
        CONSTRAINT formule_famille_connue
            CHECK (famille IN ('NUITEE', 'PASSAGE', 'DEMI_JOURNEE', 'MENSUEL')),

    -- **Entier d'unité mineure** (principe V, porte P-10). Prix d'appel : la nuit, le mois, la
    -- plage. Pour `PASSAGE`, c'est le premier palier — la table de barème fait foi.
    prix_mineur                      BIGINT      NOT NULL
        CONSTRAINT formule_prix_non_negatif CHECK (prix_mineur >= 0),

    duree_min_minutes                INTEGER     NULL
        CONSTRAINT formule_duree_min_positive CHECK (duree_min_minutes > 0),
    duree_max_minutes                INTEGER     NULL
        CONSTRAINT formule_duree_max_positive CHECK (duree_max_minutes > 0),

    -- Heures murales locales — 14 h et 12 h pour la nuitée de Deloria. La conversion en instant
    -- se fait au serveur, avec le fuseau de l'établissement lu par `EstablishmentDirectory`.
    heure_arrivee_standard           TIME        NULL,
    heure_depart_standard            TIME        NULL,

    -- 1 à 7, nul = tous les jours. Une salle de réunion qui ne se loue pas le dimanche.
    jours_autorises                  SMALLINT[]  NULL,

    -- ═══ LES DEUX CHAMPS FISCAUX — ET LA FRONTIÈRE DU PRINCIPE V ═══
    --
    -- Ce sont des **paramètres**. Ce crate les stocke et ne les interprète JAMAIS : la règle qui
    -- les consommera vivra dans `JurisdictionAdapter` (`socle/fiscalite`), en T3, et la porte
    -- P-12 fait échouer le build sur une règle fiscale trouvée ailleurs.
    --
    -- `assujettie_taxe_nuitee` est **éditable** : c'est le « moyen facultatif d'ajouter la taxe »
    -- quand une commune l'impose. Le cadrage §9.6 écrit « hors Abidjan variable selon la
    -- collectivité » — le paramètre doit exister quoi qu'il arrive, et B-02 décidera de sa valeur
    -- par défaut légale, jamais de son existence.
    assujettie_taxe_nuitee           BOOLEAN     NOT NULL,

    regle_conversion_taxe            TEXT        NULL
        CONSTRAINT formule_regle_conversion_connue
            CHECK (regle_conversion_taxe IN ('aucune', 'une_nuitee_par_occupation',
                                             'au_prorata', 'seuil_horaire')),

    -- Renseigné pour `PASSAGE` seul — toute heure entamée au-delà du dernier palier est due.
    prix_heure_supplementaire_mineur BIGINT      NULL
        CONSTRAINT formule_heure_sup_non_negative
            CHECK (prix_heure_supplementaire_mineur >= 0),

    cree_le                          TIMESTAMPTZ NOT NULL DEFAULT now(),
    modifie_le                       TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- FR-021 : une catégorie ne porte pas deux formules de la même famille. Deux « Nuitée » sur
    -- le même type de chambre, ce sont deux prix pour la même chose — et le choix se ferait par
    -- ordre d'insertion.
    CONSTRAINT formule_famille_unique UNIQUE (categorie_id, famille),

    -- FR-020 : une durée maximale inférieure à la minimale est inexploitable.
    CONSTRAINT formule_durees_coherentes
        CHECK (duree_min_minutes IS NULL OR duree_max_minutes IS NULL
               OR duree_max_minutes >= duree_min_minutes),

    -- Le prix d'heure supplémentaire n'a de sens que sur le passage.
    CONSTRAINT formule_heure_sup_reservee_au_passage
        CHECK (prix_heure_supplementaire_mineur IS NULL OR famille = 'PASSAGE'),

    -- **La contrainte qui supprime le besoin d'un troisième état d'écran.** Une formule
    -- assujettie SANS règle de conversion est une incohérence, pas un état d'attente : la rendre
    -- impossible à enregistrer évite d'avoir à dessiner « paramétrage fiscal en attente », qui
    -- n'existe ni à la maquette `G2` ni au lexique.
    CONSTRAINT formule_regle_fiscale_coherente
        CHECK (NOT assujettie_taxe_nuitee OR regle_conversion_taxe IS NOT NULL)
);

COMMENT ON TABLE hebergement.formule IS
    'Ce qu''on vend sur une catégorie. Porte DEUX PARAMÈTRES fiscaux qu''elle n''interprète jamais — la règle vit dans JurisdictionAdapter (P-12). Classe hors-ligne C.';

-- ---------------------------------------------------------------------------------------------
--  CE QUE LA BASE NE PEUT PAS GARANTIR, et où c'est tenu
-- ---------------------------------------------------------------------------------------------
--
-- Qu'une formule `PASSAGE` porte au moins un palier, et qu'une `DEMI_JOURNEE` porte au moins une
-- plage (FR-025, FR-033), ne s'exprime pas en contrainte de table : la dépendance va de l'enfant
-- au parent, et la ligne parente existe avant ses enfants.
--
-- C'est le **service** qui le valide, dans la transaction de création, et
-- `backend/tests/hebergement_referentiel.rs` qui le vérifie. Écrit ici pour qu'on ne cherche pas
-- une contrainte absente.

-- =============================================================================================
--  5. bareme_palier — les paliers du passage
-- =============================================================================================
CREATE TABLE hebergement.bareme_palier (
    formule_id    UUID   NOT NULL REFERENCES hebergement.formule (id) ON DELETE CASCADE,

    -- **`> 0`** — FR-025 refuse un palier de durée nulle : il serait toujours le premier atteint,
    -- et tout passage vaudrait son prix.
    duree_minutes INTEGER NOT NULL
        CONSTRAINT bareme_palier_duree_positive CHECK (duree_minutes > 0),

    prix_mineur   BIGINT NOT NULL
        CONSTRAINT bareme_palier_prix_non_negatif CHECK (prix_mineur >= 0),

    tenant_id     UUID   NOT NULL,

    -- **L'unicité de la durée EST la clé primaire**, pas une contrainte ajoutée. C'est ce qui rend
    -- « un barème aux paliers désordonnés » impossible à constituer plutôt qu'à corriger : deux
    -- paliers de même durée ne peuvent pas coexister, donc l'ordre est total. La lecture trie par
    -- `duree_minutes`.
    PRIMARY KEY (formule_id, duree_minutes)
);

COMMENT ON TABLE hebergement.bareme_palier IS
    'Paliers du passage. La clé primaire (formule_id, duree_minutes) rend l''ordre TOTAL. Classe hors-ligne C.';

-- =============================================================================================
--  6. plage_demi_journee — des heures murales, jamais des instants
-- =============================================================================================
CREATE TABLE hebergement.plage_demi_journee (
    id          UUID PRIMARY KEY,
    formule_id  UUID NOT NULL REFERENCES hebergement.formule (id) ON DELETE CASCADE,

    -- **`TIME` et non `TIMESTAMPTZ`.** « 8 h – 12 h » est une règle d'exploitation qui vaut tous
    -- les jours, y compris ceux qui n'existent pas encore. La stocker en instant imposerait une
    -- ligne par jour. La conversion se fait au serveur, avec le fuseau de l'établissement.
    heure_debut TIME NOT NULL,
    heure_fin   TIME NOT NULL,

    -- Clé i18n — « matin », « après-midi ». Jamais une phrase : elle traverserait l'API jusqu'à
    -- l'écran sans passer par le catalogue de traductions.
    libelle_cle TEXT NOT NULL,

    tenant_id   UUID NOT NULL,

    -- Interdit une plage qui traverse minuit. **Assumé** : une demi-journée qui franchit minuit
    -- n'est pas une demi-journée. Une formule de nuit se modélise en `PASSAGE` ou en `NUITEE`.
    CONSTRAINT plage_bornes CHECK (heure_fin > heure_debut),
    CONSTRAINT plage_unique UNIQUE (formule_id, heure_debut, heure_fin)
);

COMMENT ON TABLE hebergement.plage_demi_journee IS
    'Plages fixes de la demi-journée, en heures MURALES locales. Classe hors-ligne C.';

-- =============================================================================================
--  Sécurité au niveau ligne — le patron du module doré, appliqué aux six tables
-- =============================================================================================
--
-- Trois éléments, aucun optionnel :
--   * `FORCE` — sans lui, le propriétaire des tables reste hors politique ;
--   * `WITH CHECK` — sans lui, un tenant peut INSÉRER chez un autre : la fuite qui n'apparaît
--     dans aucune lecture ;
--   * le second argument `true` de `current_setting` — sans lui, une transaction sans contexte
--     lève une erreur au lieu de ne rien voir.

ALTER TABLE hebergement.categorie            ENABLE ROW LEVEL SECURITY;
ALTER TABLE hebergement.categorie            FORCE  ROW LEVEL SECURITY;
ALTER TABLE hebergement.temps_remise_en_etat ENABLE ROW LEVEL SECURITY;
ALTER TABLE hebergement.temps_remise_en_etat FORCE  ROW LEVEL SECURITY;
ALTER TABLE hebergement.unite                ENABLE ROW LEVEL SECURITY;
ALTER TABLE hebergement.unite                FORCE  ROW LEVEL SECURITY;
ALTER TABLE hebergement.formule              ENABLE ROW LEVEL SECURITY;
ALTER TABLE hebergement.formule              FORCE  ROW LEVEL SECURITY;
ALTER TABLE hebergement.bareme_palier        ENABLE ROW LEVEL SECURITY;
ALTER TABLE hebergement.bareme_palier        FORCE  ROW LEVEL SECURITY;
ALTER TABLE hebergement.plage_demi_journee   ENABLE ROW LEVEL SECURITY;
ALTER TABLE hebergement.plage_demi_journee   FORCE  ROW LEVEL SECURITY;

CREATE POLICY isolation_tenant ON hebergement.categorie
    USING      (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

CREATE POLICY isolation_tenant ON hebergement.temps_remise_en_etat
    USING      (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

CREATE POLICY isolation_tenant ON hebergement.unite
    USING      (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

CREATE POLICY isolation_tenant ON hebergement.formule
    USING      (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

CREATE POLICY isolation_tenant ON hebergement.bareme_palier
    USING      (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

CREATE POLICY isolation_tenant ON hebergement.plage_demi_journee
    USING      (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

-- =============================================================================================
--  Privilèges — ILS DISENT LA CLASSE
-- =============================================================================================
--
-- Classe **C** : référentiel éditable en ligne, jamais hors ligne. Les quatre verbes sont
-- accordés parce que l'exploitant crée, corrige et retire son offre. Ceux d'`occupation`
-- diffèrent, et c'est là que la différence de classe se lit — voir `0025`.

GRANT SELECT, INSERT, UPDATE, DELETE ON hebergement.categorie            TO kaya_app;
GRANT SELECT, INSERT, UPDATE, DELETE ON hebergement.temps_remise_en_etat TO kaya_app;
GRANT SELECT, INSERT, UPDATE, DELETE ON hebergement.unite                TO kaya_app;
GRANT SELECT, INSERT, UPDATE, DELETE ON hebergement.formule              TO kaya_app;
GRANT SELECT, INSERT, UPDATE, DELETE ON hebergement.bareme_palier        TO kaya_app;
GRANT SELECT, INSERT, UPDATE, DELETE ON hebergement.plage_demi_journee   TO kaya_app;
