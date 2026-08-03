-- 0033 — SEJ-02 : la fiche de police et sa numérotation.
--
-- Terme utilisateur : **« Fiche de police »** / *Police registration form*, conservé tel quel
-- (lexique v1.6.0). C'est le terme de l'usage ivoirien — ce que l'exploitant lit sur ses propres
-- registres et ce que la gendarmerie lui demande. Le reformuler le rendrait méconnaissable, même
-- raisonnement que « classement » et « NCC ».

-- =============================================================================================
--  1. hebergement.numerotation_fiche_police — un COMPTEUR, pas une SEQUENCE
-- =============================================================================================
--
-- ═══════════════════════════════════════════════════════════════════════════════════════════════
--  POURQUOI PAS UNE `SEQUENCE`, ET POURQUOI C'EST UNE ERREUR DÉJÀ COMMISE
--
--  Une `SEQUENCE` PostgreSQL a deux propriétés **fatales** à une numérotation de document
--  opérationnel :
--
--    · elle est **globale au schéma** — deux établissements du même tenant partageraient leur
--      espace de numérotation, et deux tenants aussi ;
--    · elle **laisse des trous** — `nextval` consomme même si la transaction est annulée, par
--      construction, puisqu'elle n'est pas transactionnelle.
--
--  Une numérotation continue **par établissement** est ce que la gendarmerie attend. C'est le
--  défaut exact corrigé par `0012` au cycle 002 — un espace de numérotation d'outbox partagé entre
--  tenants, trouvé par le premier événement appliqué à un second tenant.
--
--  L'incrément se fait donc par `UPDATE … RETURNING dernier_numero` **dans la transaction du
--  check-in**. Le verrou de ligne est ce qui sérialise, et c'est **la définition même de la
--  classe B** : deux arrivées simultanées sur le même établissement s'attendent, et aucune ne
--  reçoit le numéro de l'autre.
-- ═══════════════════════════════════════════════════════════════════════════════════════════════
CREATE TABLE hebergement.numerotation_fiche_police (
    tenant_id        UUID   NOT NULL,
    etablissement_id UUID   NOT NULL,
    dernier_numero   BIGINT NOT NULL DEFAULT 0 CHECK (dernier_numero >= 0),
    PRIMARY KEY (tenant_id, etablissement_id)
);

COMMENT ON TABLE hebergement.numerotation_fiche_police IS
    'Compteur de fiches de police PAR ÉTABLISSEMENT. Classe hors-ligne B. Un compteur et NON une SEQUENCE : une séquence est globale au schéma et laisse des trous, deux propriétés fatales à une numérotation continue. Défaut corrigé par 0012 au cycle 002.';

-- =============================================================================================
--  2. hebergement.fiche_police
-- =============================================================================================
CREATE TABLE hebergement.fiche_police (
    id               UUID        PRIMARY KEY,
    tenant_id        UUID        NOT NULL,
    etablissement_id UUID        NOT NULL,

    -- `UNIQUE` : **une fiche par séjour**.
    sejour_id        UUID        NOT NULL UNIQUE REFERENCES hebergement.sejour (id),

    numero           BIGINT      NOT NULL,

    -- La continuité est garantie **par établissement**, pas globalement.
    CONSTRAINT fiche_police_numero_unique UNIQUE (tenant_id, etablissement_id, numero),

    -- **FR-047** — une fiche dont l'identité est incomplète est **IDENTIFIÉE comme telle**.
    -- Elle n'est ni fabriquée avec des valeurs de remplissage, ni silencieusement omise.
    --
    -- C'est le parcours **normal** du passage : la pièce vient après la clé (FR-023). Le terme
    -- utilisateur n'est donc pas « incomplète », qui sonne comme un défaut de saisie, mais
    -- « **Identité à compléter** » (lexique v1.6.0).
    complete         BOOLEAN     NOT NULL DEFAULT false,

    generee_le       TIMESTAMPTZ NOT NULL DEFAULT now(),
    completee_le     TIMESTAMPTZ NULL,

    CONSTRAINT fiche_police_completude_coherente
        CHECK (complete = (completee_le IS NOT NULL))
);

COMMENT ON TABLE hebergement.fiche_police IS
    'Le registre légal d''un séjour. Classe hors-ligne B. AUCUNE identité n''y est recopiée : elle référence le séjour, les identités viennent du client (par AnnuaireClients) et des accompagnants — recopier créerait une TROISIÈME surface de rétention.';
COMMENT ON COLUMN hebergement.fiche_police.complete IS
    'FR-047 : une fiche sans identité rattachée est IDENTIFIÉE comme telle, jamais fabriquée ni omise. C''est le parcours normal du passage — la pièce vient après la clé.';

-- ---------------------------------------------------------------------------------------------
--  ⚠️ CE QUE CETTE TABLE NE PORTE PAS, ET C'EST LE SUJET
-- ---------------------------------------------------------------------------------------------
--
-- **Aucune identité n'y est recopiée** — ni nom, ni prénoms, ni numéro de pièce. La fiche
-- référence le séjour ; les identités viennent du client, lu par le trait `AnnuaireClients`, et
-- des accompagnants.
--
-- Recopier créerait une **troisième** surface de rétention pour la même donnée sensible, après
-- `comptes.personne` et `hebergement.accompagnant` — et la purge de 90 jours de TRX-06 devrait
-- alors la connaître, sans quoi le numéro survivrait ici à sa suppression ailleurs.
--
-- **Le gabarit officiel n'est pas inventé** (décision Q3, option (a)) : le registre minimal est en
-- base, le formulaire du pilote est un **rendu** qui s'ajoutera sans migration.

CREATE INDEX fiche_police_par_etablissement_idx
    ON hebergement.fiche_police (tenant_id, etablissement_id, numero DESC);

-- =============================================================================================
--  3. Sécurité au niveau ligne
-- =============================================================================================

ALTER TABLE hebergement.numerotation_fiche_police ENABLE ROW LEVEL SECURITY;
ALTER TABLE hebergement.numerotation_fiche_police FORCE  ROW LEVEL SECURITY;
CREATE POLICY isolation_tenant ON hebergement.numerotation_fiche_police
    USING      (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

ALTER TABLE hebergement.fiche_police ENABLE ROW LEVEL SECURITY;
ALTER TABLE hebergement.fiche_police FORCE  ROW LEVEL SECURITY;
CREATE POLICY isolation_tenant ON hebergement.fiche_police
    USING      (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

-- =============================================================================================
--  4. Privilèges
-- =============================================================================================
--
-- **`UPDATE` est accordé sur les deux, et pour des raisons différentes :**
--
--   · `numerotation_fiche_police` — l'incrément *est* un `UPDATE … RETURNING`, et c'est lui qui
--     sérialise ;
--   · `fiche_police` — **uniquement** pour passer `complete` à vrai quand l'identité est saisie
--     après la clé, ce que le parcours de passage impose (FR-023, FR-028). Le rattachement ne
--     rouvre pas le séjour et ne remet pas en cause l'attribution.
--
-- **Aucune `DELETE`.** Un registre légal ne s'efface pas.
GRANT SELECT, INSERT, UPDATE ON hebergement.numerotation_fiche_police TO kaya_app;
GRANT SELECT, INSERT, UPDATE ON hebergement.fiche_police              TO kaya_app;
