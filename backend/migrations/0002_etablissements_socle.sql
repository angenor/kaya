-- 0002 — `tenant` et `etablissement`, en FORME MINIMALE.
--
-- **Écart de périmètre assumé, à lire avant de modifier ce fichier.** Ces deux tables
-- appartiennent à **ETB-01**, le cycle suivant. Ce cycle n'en crée que ce dont il ne peut pas se
-- passer : `note_etablissement` doit se rattacher à un établissement, `evenement_outbox` porte
-- `etablissement_id`, et une politique d'isolation n'a rien à isoler sans tenant réel.
--
-- ETB-01 les enrichira — juridiction, classement, commune, NCC, adresse, identité visuelle — par
-- **migration additive**, jamais en modifiant ce fichier (principe I(b), porte P-02).

-- =============================================================================================
--  tenant
-- =============================================================================================
CREATE TABLE etablissements.tenant (
    -- UUID v7 **généré côté client** (principe VI). Une clé générée par la base rendrait le
    -- rejeu destructeur : trois envois de la même écriture produiraient trois lignes.
    id          UUID        PRIMARY KEY,
    nom         TEXT        NOT NULL,
    cree_le     TIMESTAMPTZ NOT NULL DEFAULT now(),
    modifie_le  TIMESTAMPTZ NOT NULL DEFAULT now()
);

COMMENT ON TABLE etablissements.tenant IS
    'Client de la plateforme. Classe hors-ligne C — référentiel, jamais écrit hors ligne.';

-- =============================================================================================
--  etablissement
-- =============================================================================================
CREATE TABLE etablissements.etablissement (
    id              UUID        PRIMARY KEY,
    tenant_id       UUID        NOT NULL REFERENCES etablissements.tenant (id),
    nom             TEXT        NOT NULL,
    -- Le fuseau appartient à l'établissement, pas au serveur : une clôture journalière et un
    -- calcul de nuitée se font dans le temps local de l'établissement (principe IV).
    fuseau_horaire  TEXT        NOT NULL,
    -- ISO 4217. XOF a **zéro décimale** — d'où les montants en entiers d'unité mineure.
    devise          CHAR(3)     NOT NULL,
    cree_le         TIMESTAMPTZ NOT NULL DEFAULT now(),
    modifie_le      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX etablissement_tenant_idx ON etablissements.etablissement (tenant_id);

COMMENT ON TABLE etablissements.etablissement IS
    'Établissement — l''entité centrale du produit, jamais « hôtel ». Classe hors-ligne C.';

-- =============================================================================================
--  Sécurité au niveau ligne — le patron unique, appliqué sans exception
-- =============================================================================================
--
-- Trois détails décident si l'isolation tient ou non, et aucun n'est optionnel :
--
--   `ENABLE`      active la politique pour les rôles ordinaires.
--   `FORCE`       l'applique AUSSI au propriétaire des tables. Sans lui, la première tâche de
--                 maintenance lancée sous `kaya_owner` verrait tous les clients.
--   `WITH CHECK`  empêche d'ÉCRIRE une ligne portant le tenant d'autrui. `USING` seul filtre la
--                 lecture et laisse l'insertion croisée passer — la fuite la moins visible et la
--                 plus grave.
--
-- `current_setting('app.current_tenant', true)` : le second argument évite l'erreur quand le
-- paramètre est absent. L'expression vaut alors NULL, la comparaison vaut NULL, et **aucune
-- ligne ne passe**. Une transaction sans contexte de tenant ne voit donc rien — jamais une
-- erreur, qu'un `catch` mal placé pourrait retourner en accès ouvert.

ALTER TABLE etablissements.tenant         ENABLE ROW LEVEL SECURITY;
ALTER TABLE etablissements.tenant         FORCE  ROW LEVEL SECURITY;
ALTER TABLE etablissements.etablissement  ENABLE ROW LEVEL SECURITY;
ALTER TABLE etablissements.etablissement  FORCE  ROW LEVEL SECURITY;

-- `tenant` est le SEUL cas du produit où la colonne comparée est la clé primaire : un tenant
-- *est* son propre tenant. Signalé explicitement pour que la porte P-07 ne le rencontre pas
-- comme une exception silencieuse.
CREATE POLICY isolation_tenant ON etablissements.tenant
    USING      (id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (id = current_setting('app.current_tenant', true)::uuid);

CREATE POLICY isolation_tenant ON etablissements.etablissement
    USING      (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

-- Aucun `DELETE` : on ne supprime pas un établissement, on le désactive. La suppression viendra
-- avec le droit à l'effacement du registre ARTCI (TRX-06), qui a ses propres règles.
GRANT SELECT, INSERT, UPDATE ON etablissements.tenant        TO kaya_app;
GRANT SELECT, INSERT, UPDATE ON etablissements.etablissement TO kaya_app;
