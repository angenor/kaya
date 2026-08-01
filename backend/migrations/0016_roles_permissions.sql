-- 0016 — CPT-02 : les huit rôles, les dix-sept permissions, et le cumul.
--
-- **Le cœur du cycle.** Un compte porte N rôles ; ses permissions sont **l'union** des leurs,
-- sans priorité ni hiérarchie (FR-017). Adjoua est gérante, caissière et réceptionniste — les
-- trois, sur la même connexion, dans la même application.
--
-- Trois référentiels globaux, même régime que `0008` : sans `tenant_id`, deux politiques,
-- `GRANT SELECT` seul. Et une table de cumul, celle-là bien par tenant.

-- =============================================================================================
--  1. role — les huit, et rien d'autre
-- =============================================================================================
CREATE TABLE comptes.role (
    code        TEXT     PRIMARY KEY,

    -- `ETABLISSEMENT` : le rôle s'exerce dans un établissement, et `compte_role` en exige un.
    -- `EDITEUR` : il s'exerce au-dessus des tenants, et `compte_role` en interdit un.
    portee      TEXT     NOT NULL
                CONSTRAINT role_portee_connue CHECK (portee IN ('ETABLISSEMENT', 'EDITEUR')),

    libelle_cle TEXT     NOT NULL,

    -- Ordre d'affichage stable, indépendant de la locale : trier sur le libellé traduit ferait
    -- changer l'écran en passant du français à l'anglais (même raison qu'en `0008`).
    --
    -- **Il ne porte AUCUNE hiérarchie de droits.** Les permissions sont l'union, sans priorité
    -- (FR-017). Un `ordre` lu comme un rang produirait exactement le « rôle principal » que le
    -- principe VII interdit.
    ordre       SMALLINT NOT NULL
);

COMMENT ON TABLE comptes.role IS
    'Référentiel GLOBAL des huit rôles. Sans tenant_id — régime nommé de 0008. `ordre` est un ordre d''AFFICHAGE, jamais une hiérarchie de droits.';

INSERT INTO comptes.role (code, portee, libelle_cle, ordre) VALUES
    ('proprietaire',   'ETABLISSEMENT', 'comptes.roles.proprietaire',   10),
    ('gerant',         'ETABLISSEMENT', 'comptes.roles.gerant',         20),
    ('receptionniste', 'ETABLISSEMENT', 'comptes.roles.receptionniste', 30),
    ('serveur',        'ETABLISSEMENT', 'comptes.roles.serveur',        40),
    ('caissier',       'ETABLISSEMENT', 'comptes.roles.caissier',       50),
    ('magasinier',     'ETABLISSEMENT', 'comptes.roles.magasinier',     60),
    ('comptable',      'ETABLISSEMENT', 'comptes.roles.comptable',      70),
    ('admin_editeur',  'EDITEUR',       'comptes.roles.admin_editeur',  80);

-- =============================================================================================
--  2. permission — les modules LIVRÉS seulement
-- =============================================================================================
CREATE TABLE comptes.permission (
    -- Nomenclature `<module>.<objet>.<action>`.
    code        TEXT     PRIMARY KEY,

    -- `NULL` = transversal. **Aucune clé étrangère vers `etablissements.module_activite`** : ce
    -- serait une clé inter-schémas, interdite par le principe II (porte P-04). La cohérence est
    -- tenue par un test qui lit le référentiel des modules **à travers le trait
    -- `RegistreModules`** et échoue si une permission nomme un module inconnu.
    module_code TEXT     NULL,

    libelle_cle TEXT     NOT NULL,
    ordre       SMALLINT NOT NULL
);

COMMENT ON TABLE comptes.permission IS
    'Référentiel GLOBAL des permissions. module_code SANS clé étrangère : ce serait une clé inter-schémas (P-04).';
COMMENT ON COLUMN comptes.permission.module_code IS
    'NULL = transversale. Toutes le sont au cycle 003 : aucun module d''activité n''a encore d''écran.';

-- **Dix-sept permissions, toutes transversales.** Le principe X interdit d'en poser pour un
-- module qui n'a pas encore d'écran : une permission qui ne garde aucune action est une promesse
-- sans contrepartie, et FR-021 fait échouer le build dessus. `module_code` restera donc `NULL`
-- jusqu'au cycle HEB, qui apportera `heb.unite.attribuer`.
INSERT INTO comptes.permission (code, module_code, libelle_cle, ordre) VALUES
    ('etb.etablissement.lire',     NULL, 'comptes.permissions.etb.etablissement.lire',     10),
    ('etb.etablissement.modifier', NULL, 'comptes.permissions.etb.etablissement.modifier', 20),
    ('etb.service.basculer',       NULL, 'comptes.permissions.etb.service.basculer',       30),
    ('etb.capacite.declarer',      NULL, 'comptes.permissions.etb.capacite.declarer',      40),
    ('etb.pdv.lire',               NULL, 'comptes.permissions.etb.pdv.lire',               50),
    ('etb.pdv.gerer',              NULL, 'comptes.permissions.etb.pdv.gerer',              60),
    ('etb.configuration.lire',     NULL, 'comptes.permissions.etb.configuration.lire',     70),
    ('etb.configuration.ecrire',   NULL, 'comptes.permissions.etb.configuration.ecrire',   80),
    ('etb.branding.lire',          NULL, 'comptes.permissions.etb.branding.lire',          90),
    ('etb.branding.ecrire',        NULL, 'comptes.permissions.etb.branding.ecrire',       100),
    ('etb.note.lire',              NULL, 'comptes.permissions.etb.note.lire',             110),
    ('etb.note.ecrire',            NULL, 'comptes.permissions.etb.note.ecrire',           120),
    ('cpt.compte.lire',            NULL, 'comptes.permissions.cpt.compte.lire',           130),
    ('cpt.compte.gerer',           NULL, 'comptes.permissions.cpt.compte.gerer',          140),
    ('cpt.role.attribuer',         NULL, 'comptes.permissions.cpt.role.attribuer',        150),
    ('cpt.session.revoquer',       NULL, 'comptes.permissions.cpt.session.revoquer',      160),
    ('cpt.audit.consulter',        NULL, 'comptes.permissions.cpt.audit.consulter',       170);

-- =============================================================================================
--  3. role_permission — ce que chaque rôle ouvre
-- =============================================================================================
CREATE TABLE comptes.role_permission (
    role_code       TEXT NOT NULL REFERENCES comptes.role (code),
    permission_code TEXT NOT NULL REFERENCES comptes.permission (code),
    PRIMARY KEY (role_code, permission_code)
);

COMMENT ON TABLE comptes.role_permission IS
    'Référentiel GLOBAL — ce que chaque rôle ouvre. L''union de plusieurs rôles se calcule ici, sans priorité.';

-- **`proprietaire` — tout, y compris le registre des actions.** C'est M. Koffi.
INSERT INTO comptes.role_permission (role_code, permission_code)
SELECT 'proprietaire', code FROM comptes.permission;

-- **`gerant` — tout sauf le registre des actions.** Il exploite l'établissement ; le registre
-- existe en partie pour que le propriétaire puisse relire ce qu'il a fait. Lui en donner la
-- consultation ne serait pas une faille — il ne peut de toute façon rien y modifier —, mais
-- CPT-04 le désigne comme « ce que M. Koffi achète », et la lecture par le surveillé change ce
-- que le registre est.
INSERT INTO comptes.role_permission (role_code, permission_code)
SELECT 'gerant', code FROM comptes.permission WHERE code <> 'cpt.audit.consulter';

-- **`receptionniste`, `serveur`, `caissier`, `magasinier` — la lecture de ce qui les entoure.**
-- Aucun n'a de permission d'écriture à ce cycle : leurs actions propres (check-in, prise de
-- commande, encaissement, mouvement de stock) appartiennent aux cycles SEJ, PDV, CAI et STK, et
-- leurs permissions naîtront avec elles. Poser dès maintenant `heb.unite.attribuer` serait une
-- permission qui ne garde rien (FR-021).
INSERT INTO comptes.role_permission (role_code, permission_code)
SELECT r.code, p.code
FROM comptes.role r
CROSS JOIN comptes.permission p
WHERE r.code IN ('receptionniste', 'serveur', 'caissier', 'magasinier')
  AND p.code IN ('etb.etablissement.lire', 'etb.pdv.lire', 'etb.configuration.lire',
                 'etb.branding.lire', 'etb.note.lire');

-- **`comptable` — la lecture, plus le registre des actions.** C'est le seul rôle non
-- propriétaire qui le consulte : sa mission est précisément de retrouver ce qui s'est passé.
INSERT INTO comptes.role_permission (role_code, permission_code)
SELECT 'comptable', code FROM comptes.permission
WHERE code IN ('etb.etablissement.lire', 'etb.pdv.lire', 'etb.configuration.lire',
               'etb.branding.lire', 'etb.note.lire', 'cpt.audit.consulter');

-- **`admin_editeur` — tout.** Portée `EDITEUR` : il agit au-dessus des tenants (ETB-08).
INSERT INTO comptes.role_permission (role_code, permission_code)
SELECT 'admin_editeur', code FROM comptes.permission;

-- ---------------------------------------------------------------------------------------------
--  CE QUE CETTE DISTRIBUTION IMPLIQUE AU CYCLE 003 — écrit pour que personne ne la « corrige »
-- ---------------------------------------------------------------------------------------------
--
-- Les quatre rôles opérationnels sont, à ce cycle, des **sous-ensembles stricts de `gerant`** :
-- leurs cinq permissions de lecture sont toutes dans les seize du gérant. Conséquence directe,
-- et qui surprendrait sans cette note : sur le compte d'Adjoua (gérante + caissière +
-- réceptionniste), **retirer `caissier` ne retire aucune permission**.
--
-- Ce n'est ni un défaut de la distribution ni une faiblesse de FR-018 : c'est l'état exact du
-- produit au cycle 003. Les permissions propres au caissier — ouvrir un shift, encaisser,
-- compter, clôturer — appartiennent au cycle **CAI** et naîtront avec les écrans qu'elles
-- gardent. En poser dès maintenant produirait des permissions qui ne gardent **rien**, ce que
-- FR-021 fait échouer, et le principe X interdit (« prêt ≠ construit »).
--
-- La paire qui exerce réellement FR-018 existe pourtant, et c'est celle que
-- `backend/tests/roles_cumules.rs` emploie : **`gerant` + `comptable`**. Leur intersection est
-- faite des cinq lectures ; `cpt.audit.consulter` est **exclusive au comptable**. Retirer
-- `comptable` fait perdre cette seule permission et conserve les cinq autres — la démonstration
-- que le scénario 2 de US3 demande.
--
-- **Ne pas « équilibrer » cette table en inventant des permissions pour les rôles pauvres.** Un
-- rôle dont les permissions arrivent au cycle de ses écrans est un rôle correct ; un rôle garni
-- de permissions qui n'ouvrent rien est une promesse que l'interface devra tenir.

-- =============================================================================================
--  4. compte_role — LE CUMUL
-- =============================================================================================
CREATE TABLE comptes.compte_role (
    id                     UUID        PRIMARY KEY,
    tenant_id              UUID        NOT NULL,
    compte_id              UUID        NOT NULL REFERENCES comptes.compte (id),
    role_code              TEXT        NOT NULL REFERENCES comptes.role (code),

    -- `NULL` pour `admin_editeur`. **Aucune clé étrangère** vers `etablissements.etablissement` :
    -- clé inter-schémas (P-04). L'existence est vérifiée par le service via
    -- `EstablishmentDirectory`, ce qui donne un `404 etablissement_inconnu` intelligible plutôt
    -- qu'une violation de contrainte.
    etablissement_id       UUID        NULL,

    -- Qui a attribué ce rôle. Clé étrangère **intra-schéma**, donc autorisée.
    attribue_par_compte_id UUID        NOT NULL REFERENCES comptes.compte (id),

    horodatage_client      TIMESTAMPTZ NULL,
    cree_le                TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- **`NULLS NOT DISTINCT` n'est pas décoratif.** En SQL standard, deux `NULL` ne sont pas
    -- égaux : sans cette clause, `(compte, admin_editeur, NULL)` s'insérerait autant de fois
    -- qu'on veut, et le retrait d'un rôle n'en retirerait qu'une occurrence sur N. Disponible
    -- depuis PostgreSQL 15 ; la cible est 18.4.
    CONSTRAINT compte_role_unique UNIQUE NULLS NOT DISTINCT (compte_id, role_code, etablissement_id)
);

COMMENT ON TABLE comptes.compte_role IS
    'LE CUMUL — N lignes par compte. Les permissions sont l''UNION, sans priorité (FR-017). Classe hors-ligne C.';

-- L'index de la lecture réelle : les rôles d'un compte dans un établissement — c'est le calcul
-- des permissions effectives, fait à chaque délivrance de jeton.
CREATE INDEX compte_role_lecture_idx
    ON comptes.compte_role (tenant_id, compte_id, etablissement_id);

-- L'index du refus `derniere_habilitation` (FR-023) : qui, dans cet établissement, peut encore
-- attribuer des rôles ?
CREATE INDEX compte_role_par_etablissement_idx
    ON comptes.compte_role (tenant_id, etablissement_id, role_code);

-- =============================================================================================
--  Sécurité au niveau ligne
-- =============================================================================================

ALTER TABLE comptes.role            ENABLE ROW LEVEL SECURITY;
ALTER TABLE comptes.role            FORCE  ROW LEVEL SECURITY;
ALTER TABLE comptes.permission      ENABLE ROW LEVEL SECURITY;
ALTER TABLE comptes.permission      FORCE  ROW LEVEL SECURITY;
ALTER TABLE comptes.role_permission ENABLE ROW LEVEL SECURITY;
ALTER TABLE comptes.role_permission FORCE  ROW LEVEL SECURITY;
ALTER TABLE comptes.compte_role     ENABLE ROW LEVEL SECURITY;
ALTER TABLE comptes.compte_role     FORCE  ROW LEVEL SECURITY;

CREATE POLICY lecture_universelle ON comptes.role
    FOR SELECT USING (true);
CREATE POLICY administration_editeur ON comptes.role
    FOR ALL TO kaya_owner USING (true) WITH CHECK (true);

CREATE POLICY lecture_universelle ON comptes.permission
    FOR SELECT USING (true);
CREATE POLICY administration_editeur ON comptes.permission
    FOR ALL TO kaya_owner USING (true) WITH CHECK (true);

CREATE POLICY lecture_universelle ON comptes.role_permission
    FOR SELECT USING (true);
CREATE POLICY administration_editeur ON comptes.role_permission
    FOR ALL TO kaya_owner USING (true) WITH CHECK (true);

CREATE POLICY isolation_tenant ON comptes.compte_role
    USING      (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

-- =============================================================================================
--  Privilèges
-- =============================================================================================

GRANT SELECT ON comptes.role            TO kaya_app;
GRANT SELECT ON comptes.permission      TO kaya_app;
GRANT SELECT ON comptes.role_permission TO kaya_app;

-- **`SELECT, INSERT, DELETE` — pas d'`UPDATE`, et c'est une décision.**
--
-- Changer un rôle, c'est en retirer un et en attribuer un autre : **deux actes, deux entrées
-- d'audit**. Un `UPDATE` en ferait un seul événement, dont le registre ne dirait ni ce qui a été
-- retiré ni par qui — et le propriétaire qui cherche « qui a donné la caisse à Yao » ne
-- trouverait qu'une ligne modifiée.
--
-- Le retrait est un vrai `DELETE`, pas un drapeau : l'historique vit au **journal d'audit**, qui
-- est fait pour ça et que rien ne peut réécrire. Une colonne `retire_le` créerait un second
-- historique, partiel et modifiable.
GRANT SELECT, INSERT, DELETE ON comptes.compte_role TO kaya_app;
