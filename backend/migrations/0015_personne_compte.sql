-- 0015 — CPT-00 / CPT-01 : `personne`, `methode_authentification`, `compte`.
--
-- **Les trois tables que CPT-00 interdit de confondre.** L'identité civile (`personne`), ce avec
-- quoi on se connecte (`compte`) et le contrat de travail (`employe`, migration 0018) sont trois
-- choses distinctes. Une femme de ménage a une fiche et aucun compte ; un comptable externe a un
-- compte et aucun contrat ; Adjoua a les deux. Les fusionner « puisque c'est la même personne »
-- est la faute que cette migration rend structurellement impossible : aucune colonne de contrat
-- n'existe ici, et `backend/tests/personne_compte_employe.rs` refuse qu'il en apparaisse une.

-- =============================================================================================
--  1. personne — l'identité civile
-- =============================================================================================
CREATE TABLE comptes.personne (
    -- UUID v7 **généré côté client** (principe VI) : c'est ce qui rend le rejeu inoffensif.
    id                UUID        PRIMARY KEY,

    -- Pas de `REFERENCES etablissements.tenant` : ce serait une clé étrangère inter-schémas,
    -- interdite par le principe II. La colonne porte le tenant, la politique RLS le filtre.
    tenant_id         UUID        NOT NULL,

    nom               TEXT        NOT NULL
                      CHECK (length(btrim(nom)) BETWEEN 1 AND 200),
    prenoms           TEXT        NULL,

    -- E.164. **Aucun `CHECK` de format national ici** : l'indicatif par défaut `+225` est un
    -- paramètre d'établissement (porte P-12), pas une contrainte de base. Une contrainte
    -- ivoirienne posée en dur ferait échouer le premier établissement togolais.
    telephone         TEXT        NULL,
    email             TEXT        NULL,

    -- ⚠️ **POSÉES ET NON ALIMENTÉES PAR CE CYCLE.**
    --
    -- Ce sont des données d'identité de client : leur alimentation relève de **SEJ-01** (fiche
    -- client) et leur **rétention de 90 jours** de **TRX-06**. Poser la colonne sans la politique
    -- de rétention qui va avec serait le moyen le plus simple de constituer un fichier
    -- d'identités sans durée de conservation — ce qui est exactement ce que l'ARTCI interdit.
    --
    -- `backend/tests/provisions_sans_logique.rs` vérifie qu'aucun point d'entrée de ce cycle ne
    -- les écrit.
    type_piece        TEXT        NULL,
    numero_piece      TEXT        NULL,

    -- Deux horodatages distincts, jamais fusionnés (module doré, couche 1).
    horodatage_client TIMESTAMPTZ NULL,
    cree_le           TIMESTAMPTZ NOT NULL DEFAULT now(),
    modifie_le        TIMESTAMPTZ NOT NULL DEFAULT now()
);

COMMENT ON TABLE comptes.personne IS
    'Identité civile. Classe hors-ligne C. Ne porte AUCUN élément d''authentification ni de contrat de travail (FR-004).';
COMMENT ON COLUMN comptes.personne.type_piece IS
    'POSÉE, NON ALIMENTÉE par CPT. Alimentation SEJ-01, rétention 90 jours TRX-06.';

CREATE INDEX personne_lecture_idx ON comptes.personne (tenant_id, nom);

-- =============================================================================================
--  2. methode_authentification — RÉFÉRENTIEL GLOBAL, et le refus structurel d'OTP_SMS
-- =============================================================================================
--
-- Sans `tenant_id` : c'est le même référentiel pour tous les clients. Régime nommé de `0008`,
-- repris à la lettre — deux politiques et un jeu de privilèges asymétrique.
--
-- **ORDRE IMPÉRATIF : `CREATE TABLE` → `INSERT` → `ENABLE`/`FORCE` → `CREATE POLICY`.**
-- `FORCE` applique les politiques au propriétaire lui-même. Insérer après l'avoir activé, mais
-- avant d'avoir créé `administration_editeur`, échouerait — et **en silence** : la comparaison
-- avec `current_setting('app.current_tenant', true)` vaut `NULL` hors requête applicative, aucune
-- ligne n'est touchée, aucune erreur n'est levée.
CREATE TABLE comptes.methode_authentification (
    code        TEXT     PRIMARY KEY,

    -- Support de la clé étrangère composite de `compte`. C'est cette colonne, recopiée par la
    -- table qui la référence, qui rend le refus **structurel**.
    implementee BOOLEAN  NOT NULL,

    -- Clé i18n, jamais un libellé : une chaîne stockée en base échapperait à la porte P-16.
    libelle_cle TEXT     NOT NULL,
    ordre       SMALLINT NOT NULL,

    UNIQUE (code, implementee)
);

COMMENT ON TABLE comptes.methode_authentification IS
    'Référentiel GLOBAL des méthodes d''authentification. Sans tenant_id — régime nommé de 0008. Classe hors-ligne C, branche C2.';

-- **Deux méthodes, une seule implémentée.** `OTP_SMS` figure au référentiel avec
-- `implementee = false`, ce qui permet au refus de distinguer « connu mais non servi » de
-- « inconnu » — distinction qu'un `CHECK ... IN` littéral ne saurait pas faire, et qui change le
-- message rendu à l'exploitant (patron d'ETB-02b).
INSERT INTO comptes.methode_authentification (code, implementee, libelle_cle, ordre) VALUES
    ('MOT_DE_PASSE', true,  'comptes.methodes.MOT_DE_PASSE', 10),
    ('OTP_SMS',      false, 'comptes.methodes.OTP_SMS',      20);

-- =============================================================================================
--  3. compte — l'identité d'authentification
-- =============================================================================================
CREATE TABLE comptes.compte (
    id                     UUID        PRIMARY KEY,
    tenant_id              UUID        NOT NULL,

    -- Clé étrangère **intra-schéma** : `personne` est dans le même module. C'est la seule forme
    -- de clé étrangère que le principe II autorise.
    personne_id            UUID        NOT NULL REFERENCES comptes.personne (id),

    identifiant_telephone  TEXT        NULL,
    identifiant_email      TEXT        NULL,

    -- Format PHC — les paramètres du hachage voyagent AVEC le condensat. C'est ce qui rend le
    -- rehachage possible à la vérification suivante quand la recommandation évolue : sans eux,
    -- une montée de paramètres ne protégerait que les comptes créés après elle.
    --
    -- **Jamais lu par un SELECT de liste.** Le repository expose deux chemins distincts : celui
    -- de l'authentification, qui le lit, et celui de l'affichage, qui ne le sélectionne pas. Une
    -- structure unique le ferait traverser toutes les couches, jusqu'au risque de le sérialiser
    -- un jour dans une réponse.
    condensat_mot_de_passe TEXT        NOT NULL,

    methode_code           TEXT        NOT NULL DEFAULT 'MOT_DE_PASSE',
    methode_implementee    BOOLEAN     NOT NULL DEFAULT true,

    -- Désactivation, jamais suppression (FR-014). La clé étrangère de `journal_audit` vers cette
    -- table rend d'ailleurs la suppression impossible tant qu'une entrée d'audit la désigne.
    actif                  BOOLEAN     NOT NULL DEFAULT true,

    horodatage_client      TIMESTAMPTZ NULL,
    cree_le                TIMESTAMPTZ NOT NULL DEFAULT now(),
    modifie_le             TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT compte_au_moins_un_identifiant
        CHECK (identifiant_telephone IS NOT NULL OR identifiant_email IS NOT NULL),

    -- Ceinture. La clé étrangère composite ci-dessous est la bretelle, et c'est elle qui tient :
    -- un `CHECK` seul se relâcherait à la première correction de bogue.
    CONSTRAINT compte_methode_implementee_seulement
        CHECK (methode_implementee),

    -- **Le refus d'`OTP_SMS` est structurel.** `(methode_code, methode_implementee)` ne peut
    -- désigner qu'une ligne du référentiel, et `methode_implementee` est contrainte à `true` :
    -- il n'existe aucune combinaison acceptable pointant vers une méthode non servie. Patron
    -- d'ETB-02b, repris à la lettre.
    CONSTRAINT compte_methode_servie
        FOREIGN KEY (methode_code, methode_implementee)
        REFERENCES comptes.methode_authentification (code, implementee),

    -- **L'unicité est PAR TENANT** (hypothèse 3 de la spec), cohérente avec l'isolation : deux
    -- clients distincts peuvent employer un même numéro. Une unicité globale ferait fuir
    -- l'information « ce numéro est déjà client de Kaya » à qui essaie d'en créer un compte.
    CONSTRAINT compte_identifiant_telephone_unique UNIQUE (tenant_id, identifiant_telephone),
    CONSTRAINT compte_identifiant_email_unique     UNIQUE (tenant_id, identifiant_email)
);

COMMENT ON TABLE comptes.compte IS
    'Identité d''authentification. Classe hors-ligne C. Ne porte AUCUN élément de contrat de travail (FR-004).';
COMMENT ON COLUMN comptes.compte.condensat_mot_de_passe IS
    'Format PHC — paramètres inclus, ce qui rend le rehachage possible. JAMAIS rendu par une réponse d''API, sur aucun chemin.';

-- L'index de la lecture réelle : les comptes d'un tenant. La recherche par identifiant à la
-- connexion passe par les deux contraintes d'unicité, qui portent déjà leurs index.
CREATE INDEX compte_lecture_idx ON comptes.compte (tenant_id, actif);
CREATE INDEX compte_personne_idx ON comptes.compte (tenant_id, personne_id);

-- =============================================================================================
--  Sécurité au niveau ligne
-- =============================================================================================
--
-- Les deux tables portant un `tenant_id` suivent le patron d'isolation ; le référentiel global
-- suit le régime nommé de `0008`. **Aucune exemption, jamais** — la porte P-07 ne connaît pas
-- d'exception, et un référentiel global y est compté conforme et nommé, pas dispensé.

ALTER TABLE comptes.personne                  ENABLE ROW LEVEL SECURITY;
ALTER TABLE comptes.personne                  FORCE  ROW LEVEL SECURITY;
ALTER TABLE comptes.compte                    ENABLE ROW LEVEL SECURITY;
ALTER TABLE comptes.compte                    FORCE  ROW LEVEL SECURITY;
ALTER TABLE comptes.methode_authentification  ENABLE ROW LEVEL SECURITY;
ALTER TABLE comptes.methode_authentification  FORCE  ROW LEVEL SECURITY;

CREATE POLICY isolation_tenant ON comptes.personne
    USING      (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

CREATE POLICY isolation_tenant ON comptes.compte
    USING      (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

CREATE POLICY lecture_universelle ON comptes.methode_authentification
    FOR SELECT USING (true);
CREATE POLICY administration_editeur ON comptes.methode_authentification
    FOR ALL TO kaya_owner USING (true) WITH CHECK (true);

-- =============================================================================================
--  Privilèges — ils disent la règle
-- =============================================================================================
--
-- **Aucun `DELETE`, nulle part.** Rien ne se supprime dans Kaya (FR-014) : une personne se
-- corrige, un compte se désactive. Accorder `DELETE` ouvrirait un chemin par lequel un compte
-- désigné par une entrée d'audit disparaîtrait — et le registre que le propriétaire achète
-- désignerait un identifiant sans nom.
GRANT SELECT, INSERT, UPDATE ON comptes.personne TO kaya_app;
GRANT SELECT, INSERT, UPDATE ON comptes.compte   TO kaya_app;

-- `SELECT` **et rien d'autre** sur le référentiel : `kaya_app` est refusé deux fois — aucun
-- privilège d'écriture, et aucune politique qui l'autoriserait. Un `GRANT` accordé par erreur
-- plus tard ne suffirait donc pas à ouvrir la table.
GRANT SELECT ON comptes.methode_authentification TO kaya_app;
