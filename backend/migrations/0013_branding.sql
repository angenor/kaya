-- 0013 — ETB-05 : l'identité visuelle.
--
-- Posée au **tenant**, surchargée **partiellement** par établissement. M. Koffi a une identité
-- pour ses deux établissements, et une exception pour la résidence meublée : un autre logo, le
-- reste identique.
--
-- **Numérotée 0013 et non 0012** : la correction de la séquence d'outbox a pris ce numéro. Le
-- plan annonçait six migrations pour ce cycle ; il y en a sept, la septième étant la correction
-- d'un défaut du cycle 001 que ce cycle est le premier à rencontrer.

CREATE TABLE etablissements.branding (
    id                UUID        PRIMARY KEY,
    tenant_id         UUID        NOT NULL REFERENCES etablissements.tenant (id),

    -- **`NULL` = niveau tenant.** Même mécanique de portée dérivée que
    -- `parametre_configuration`, réduite à deux niveaux.
    etablissement_id  UUID            NULL REFERENCES etablissements.etablissement (id),

    -- =========================================================================================
    --  TOUTES LES COLONNES DE CONTENU SONT NULLABLES — c'est le MÉCANISME de surcharge partielle
    -- =========================================================================================
    --
    -- La résolution prend, **champ par champ**, la première valeur non nulle en descendant du
    -- tenant vers l'établissement. Surcharger le seul logo laisse hériter tout le reste, sans
    -- qu'aucune logique de fusion n'ait à être écrite (FR-056).
    --
    -- L'alternative — une ligne complète par niveau, tous champs obligatoires — obligerait à
    -- recopier l'en-tête, le pied et les mentions légales pour changer un logo. La première
    -- divergence entre les deux copies serait découverte sur un document imprimé.

    -- **Clé d'objet dans le stockage S3, JAMAIS le binaire.** Un logo en base gonfle chaque
    -- sauvegarde, chaque réplication et chaque restauration, pour un fichier qui ne change
    -- jamais. L'accès passe par une URL signée de courte durée (principe II).
    logo_objet_cle    TEXT            NULL,

    couleur_primaire  TEXT            NULL
                      CONSTRAINT branding_couleur_hexadecimale
                          CHECK (couleur_primaire IS NULL
                                 OR couleur_primaire ~ '^#[0-9A-Fa-f]{6}$'),

    entete_document   TEXT            NULL,
    pied_document     TEXT            NULL,
    mentions_legales  TEXT            NULL,
    coordonnees       TEXT            NULL,

    modifie_le        TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- **`NULLS NOT DISTINCT`**, pour la même raison qu'à `parametre_configuration` : sans lui,
    -- deux identités visuelles de niveau tenant coexisteraient, et la résolution en choisirait une
    -- au hasard. Une ligne par niveau, au plus.
    CONSTRAINT branding_unicite
        UNIQUE NULLS NOT DISTINCT (tenant_id, etablissement_id)
);

COMMENT ON TABLE etablissements.branding IS
    'Identité visuelle, niveau tenant (etablissement_id NULL) ou établissement. Classe hors-ligne C. Toutes les colonnes de contenu nullables — c''est le mécanisme de surcharge partielle.';

-- **`couleur_primaire` ne touche JAMAIS l'interface** (FR-059). Elle s'applique aux documents
-- produits, et à eux seuls. La porte P-17 interdit toute couleur littérale hors des jetons de
-- design ; cette valeur est une **donnée client**, pas un style d'application. La distinction est
-- écrite ici et dans le composant qui la consomme — sans quoi le premier développeur pressé
-- l'appliquerait à un bouton, et le produit prendrait la couleur de chaque client.
COMMENT ON COLUMN etablissements.branding.couleur_primaire IS
    'Couleur des DOCUMENTS produits. Jamais l''interface (FR-059) : c''est une donnée client, pas un jeton de design.';
COMMENT ON COLUMN etablissements.branding.logo_objet_cle IS
    'Clé d''objet S3. JAMAIS le binaire en base (principe II).';

ALTER TABLE etablissements.branding ENABLE ROW LEVEL SECURITY;
ALTER TABLE etablissements.branding FORCE  ROW LEVEL SECURITY;

CREATE POLICY isolation_tenant ON etablissements.branding
    USING      (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

GRANT SELECT, INSERT, UPDATE ON etablissements.branding TO kaya_app;
