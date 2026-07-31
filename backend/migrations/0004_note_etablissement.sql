-- 0004 — `note_etablissement`, l'entité du MODULE DORÉ.
--
-- Note interne libre attachée à un établissement. **Classe hors-ligne A** : append-only,
-- commutative, sans contrainte d'unicité métier, sans effet monétaire (arbre de décision du
-- cadrage §11.2, branche A4).
--
-- Cette table n'a aucune importance fonctionnelle. Sa valeur est d'être le **patron** que tous
-- les cycles suivants recopieront — d'où le soin apporté aux trois points ci-dessous, qui ne se
-- devinent pas et que le premier cycle doit trancher une fois pour toutes.

CREATE TABLE etablissements.note_etablissement (
    -- **(1) L'identifiant est fourni par le client, jamais généré par la base.**
    --
    -- C'est ce qui rend le rejeu inoffensif (cadrage §11.5.1) : trois envois de la même écriture
    -- entrent en conflit de clé primaire et produisent un seul enregistrement. Une clé générée
    -- côté base produirait trois lignes, et le terminal hors ligne qui vide sa file après une
    -- coupure créerait des doublons silencieux. L'INSERT porte donc `ON CONFLICT (id) DO NOTHING`.
    id                UUID        PRIMARY KEY,

    tenant_id         UUID        NOT NULL REFERENCES etablissements.tenant (id),
    etablissement_id  UUID        NOT NULL REFERENCES etablissements.etablissement (id),

    -- **(3) Aucune clé étrangère vers `compte`** — le point le plus contre-intuitif du patron.
    --
    -- Le crate `socle/comptes` n'existe pas encore (CPT-01), mais ce n'est pas la raison : même
    -- quand il existera, une clé étrangère d'ici vers lui joindrait deux schémas de modules, ce
    -- que le principe II interdit. L'intégrité référentielle inter-modules passe par un trait
    -- exposé — `EstablishmentDirectory` et ses successeurs — jamais par la base.
    auteur_compte_id  UUID        NOT NULL,

    texte             TEXT        NOT NULL
                      CHECK (length(btrim(texte)) BETWEEN 1 AND 2000),

    -- **(2) Deux horodatages distincts, jamais fusionnés.**
    --
    -- `horodatage_client` vient du terminal ; il sert à l'ordre d'affichage local et **aucune
    -- règle ne s'y appuie**. `cree_le` est posé par le serveur et fait autorité (principe IV).
    -- Les réunir « pour simplifier » est exactement la faute que le cadrage §11.4 décrit sous le
    -- nom d'horloges non fiables : un terminal mal réglé décalerait des durées de passage, donc
    -- des montants.
    horodatage_client TIMESTAMPTZ     NULL,
    cree_le           TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Index de la lecture réelle : les notes d'un établissement, de la plus récente à la plus
-- ancienne. `tenant_id` en tête parce que la politique de sécurité filtre dessus à chaque accès.
CREATE INDEX note_etablissement_lecture_idx
    ON etablissements.note_etablissement (tenant_id, etablissement_id, cree_le DESC);

COMMENT ON TABLE etablissements.note_etablissement IS
    'Note interne d''établissement. Classe hors-ligne A (append-only, commutative). Entité du module doré.';

-- Sécurité au niveau ligne — patron identique à toutes les autres tables, sans exception.
ALTER TABLE etablissements.note_etablissement ENABLE ROW LEVEL SECURITY;
ALTER TABLE etablissements.note_etablissement FORCE  ROW LEVEL SECURITY;

CREATE POLICY isolation_tenant ON etablissements.note_etablissement
    USING      (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

-- Aucun `UPDATE`, aucun `DELETE` : une entité de classe A est **append-only**. Une correction est
-- une nouvelle note, jamais une réécriture — sans quoi la commutativité du rejeu tomberait, et
-- avec elle la garantie que le test de désordre vérifie.
GRANT SELECT, INSERT ON etablissements.note_etablissement TO kaya_app;
