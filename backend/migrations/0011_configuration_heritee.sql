-- 0011 — ETB-04 : la chaîne d'héritage de configuration.
--
-- **Le composant le plus réutilisé du produit.** Huit cycles le liront — HEB, FIS, CAI, IMP, STK,
-- RSV, QRC, CPT. Écrit au cycle HEB, il serait teinté d'hébergement ; écrit ici, il sert tout le
-- monde.
--
-- Quatre niveaux : tenant → établissement → service → point de vente. Le plus spécifique gagne.

-- =============================================================================================
--  parametre_configuration
-- =============================================================================================
CREATE TABLE etablissements.parametre_configuration (
    id                      UUID        PRIMARY KEY,
    tenant_id               UUID        NOT NULL REFERENCES etablissements.tenant (id),

    -- **LA PORTÉE EST DÉRIVÉE, JAMAIS DÉCLARÉE.**
    --
    -- Trois clés étrangères nullables dont **au plus une** est renseignée ; zéro renseignée
    -- signifie « niveau tenant ». Une colonne `portee` accompagnée d'un `portee_id` polymorphe
    -- serait plus courte à écrire et **ne permettrait aucune intégrité référentielle** : rien
    -- n'empêcherait `portee = 'POINT_DE_VENTE'` avec l'identifiant d'un établissement, et la
    -- résolution appliquerait alors un paramètre au mauvais niveau sans que rien ne le signale.
    --
    -- Ici, la portée **ne peut pas mentir** : elle se lit des colonnes renseignées, et chacune
    -- est garantie par sa clé étrangère.
    etablissement_id        UUID            NULL REFERENCES etablissements.etablissement (id),
    etablissement_module_id UUID            NULL REFERENCES etablissements.etablissement_module (id),
    point_de_vente_id       UUID            NULL REFERENCES etablissements.point_de_vente (id),

    -- Une clé hors catalogue est refusée par la base elle-même. Sans cette clé étrangère, une
    -- faute de frappe créerait un paramètre que personne ne lirait jamais — et qui resterait
    -- invisible jusqu'au jour où l'on chercherait pourquoi un réglage « ne prend pas ».
    cle                     TEXT        NOT NULL
                            REFERENCES etablissements.parametre_catalogue (cle),

    valeur                  JSONB       NOT NULL,
    modifie_le              TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT parametre_configuration_une_seule_portee
        CHECK (num_nonnulls(etablissement_id, etablissement_module_id, point_de_vente_id) <= 1),

    -- **`NULLS NOT DISTINCT` n'est pas un détail — c'est le piège de cette migration.**
    --
    -- Sans lui, `UNIQUE` traite chaque `NULL` comme distinct de tous les autres. Deux surcharges
    -- de **niveau tenant** portant la même clé — donc trois colonnes à `NULL` chacune — passeraient
    -- toutes les deux, et la résolution en choisirait une au hasard selon le plan d'exécution.
    -- Le défaut ne se verrait qu'à la première divergence de comportement entre deux lectures
    -- identiques.
    --
    -- PostgreSQL 18.4 le prend en charge nativement ; l'index unique partiel qui servait de
    -- contournement historique est inutile ici.
    CONSTRAINT parametre_configuration_unicite
        UNIQUE NULLS NOT DISTINCT
            (tenant_id, etablissement_id, etablissement_module_id, point_de_vente_id, cle)
);

-- Index de la descente de chaîne : la résolution filtre toujours sur `(tenant_id, cle)`.
CREATE INDEX parametre_configuration_cle_idx
    ON etablissements.parametre_configuration (tenant_id, cle);

-- Index partiels par niveau — chacun ne porte que les lignes de sa portée, donc reste petit même
-- quand le niveau tenant en compte des milliers.
CREATE INDEX parametre_configuration_etablissement_idx
    ON etablissements.parametre_configuration (etablissement_id)
    WHERE etablissement_id IS NOT NULL;

CREATE INDEX parametre_configuration_module_idx
    ON etablissements.parametre_configuration (etablissement_module_id)
    WHERE etablissement_module_id IS NOT NULL;

CREATE INDEX parametre_configuration_pdv_idx
    ON etablissements.parametre_configuration (point_de_vente_id)
    WHERE point_de_vente_id IS NOT NULL;

COMMENT ON TABLE etablissements.parametre_configuration IS
    'Valeur de paramètre à un niveau de la chaîne d''héritage. Classe hors-ligne C. La portée est DÉRIVÉE des trois clés étrangères nullables.';

-- =============================================================================================
--  Sécurité au niveau ligne
-- =============================================================================================
ALTER TABLE etablissements.parametre_configuration ENABLE ROW LEVEL SECURITY;
ALTER TABLE etablissements.parametre_configuration FORCE  ROW LEVEL SECURITY;

CREATE POLICY isolation_tenant ON etablissements.parametre_configuration
    USING      (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

-- Pas de `DELETE` : retirer une surcharge, c'est réécrire la valeur du niveau supérieur, pas
-- effacer une ligne. Le jour où l'effacement d'une surcharge sera nécessaire, il devra passer par
-- une opération nommée qui émet son propre événement.
GRANT SELECT, INSERT, UPDATE ON etablissements.parametre_configuration TO kaya_app;

-- =============================================================================================
--  Contenu du catalogue — une seule clé à ce cycle
-- =============================================================================================
--
-- **`politique_impression`, portée la plus basse `POINT_DE_VENTE`, story ETB-03, SANS jeu de
-- valeurs** — celui-ci est défini par le cycle IMP.
--
-- Un catalogue à une entrée se justifie parce que le résolveur doit exister **avant** le cycle qui
-- le consommera en premier : le concevoir au cycle HEB le teinterait d'hébergement.
--
-- L'`INSERT` passe ici alors que `parametre_catalogue` est en `FORCE ROW LEVEL SECURITY` : c'est
-- la politique `administration_editeur ... TO kaya_owner`, posée en 0008, qui l'autorise. Sans
-- elle, cette ligne échouerait silencieusement — sans erreur, sans écriture.
INSERT INTO etablissements.parametre_catalogue
    (cle, type_valeur, portee_la_plus_basse, story, libelle_cle, description_cle)
VALUES (
    'politique_impression',
    'TEXTE',
    'POINT_DE_VENTE',
    'ETB-03',
    'configuration.politique_impression.libelle',
    'configuration.politique_impression.description'
);
