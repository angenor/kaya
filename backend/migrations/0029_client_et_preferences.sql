-- 0029 — SEJ-01 : la fiche client. **Deux tables neuves, et quatre colonnes sur une table du
-- cycle 003.**
--
-- ═══════════════════════════════════════════════════════════════════════════════════════════════
--  LA DÉCISION DE CETTE MIGRATION TIENT EN UNE PHRASE
--
--      La fiche client est `comptes.personne` QUALIFIÉE par `comptes.client`.
--      Ce n'est pas une table portant nom, prénoms, téléphone et pièce d'identité.
--
--  Le réflexe — créer une table `client` complète — produirait un **second fichier d'identités**,
--  avec sa propre durée de conservation à tenir, sa propre purge à écrire, et deux fiches à
--  réconcilier pour une seule personne. `comptes.personne` porte déjà `nom`, `prenoms`,
--  `telephone`, `email`, `type_piece` et `numero_piece` ; ces deux dernières sont annotées dans
--  `0015` d'un commentaire qui décide de ce cycle : « POSÉES ET NON ALIMENTÉES. Alimentation
--  SEJ-01, rétention 90 jours TRX-06. » **Ce cycle les alimente.**
-- ═══════════════════════════════════════════════════════════════════════════════════════════════
--
-- ## Ce que `comptes.client` fait, et qui n'est pas cosmétique — elle QUALIFIE
--
-- `comptes.personne` porte le **personnel autant que les clients** (CPT-00 : « une femme de
-- ménage a une fiche et aucun compte »). Sans la table de qualification, chercher « Kouamé » à la
-- réception ferait apparaître la femme de ménage. La recherche joint `personne` et `client`
-- **dans le même schéma** — clé étrangère intra-schéma, la seule forme que le principe II
-- autorise —, en **une** requête, ce qui est la condition de la cible des 300 ms sur 10 000
-- fiches (FR-006, SC-005). Une table `client` dans un schéma séparé aurait imposé soit une
-- jointure inter-schémas, que P-04 interdit, soit deux requêtes.
--
-- ## ⚠️ `0015` N'EST PAS MODIFIÉE, et c'est P-02
--
-- La tentation est réelle : maintenant que les colonnes de pièce d'identité sont alimentées, leur
-- commentaire « POSÉES, NON ALIMENTÉES » paraît faux. **Il ne l'est pas** — il décrit l'état au
-- cycle 003 et reste vrai de ce cycle-là. P-02 interdit de modifier une migration déjà appliquée,
-- pas de modifier une table : la mise à jour passe par `COMMENT ON COLUMN` ci-dessous.

-- =============================================================================================
--  1. comptes.personne — quatre colonnes, et trois index qui décident d'un écran
-- =============================================================================================
--
-- Les trois colonnes « repliées » portent la forme **cherchable** de trois attributs. Elles sont
-- calculées à l'écriture, jamais à la lecture : replier dix mille lignes à chaque frappe de Yao
-- coûterait exactement la cible qu'on essaie de tenir.
ALTER TABLE comptes.personne
    -- `nom` et `prenoms` concaténés puis repliés (research R-04) — minuscules, sans signes
    -- diacritiques, sans apostrophe droite ni typographique. Chercher « kouame » doit trouver
    -- « KOUAMÉ », et « nguessan » doit trouver « N'Guessan » comme « N’Guessan ».
    ADD COLUMN nom_repli           TEXT NULL,

    -- Chiffres seuls, préfixés de l'indicatif de l'établissement quand la saisie n'en porte pas
    -- (research R-06). C'est ce qui fait que « 0707123456 » et « +2250707123456 » se trouvent
    -- l'un l'autre — au comptoir, personne ne tape l'indicatif.
    ADD COLUMN telephone_repli     TEXT NULL,

    -- Alphanumérique en majuscules, sans espace ni tiret : le même numéro écrit
    -- « CI-0012 3456 » et « ci00123456 » est le même numéro.
    ADD COLUMN numero_piece_repli  TEXT NULL,

    -- FR-013 — l'instant de **capture** de la pièce, distinct de `cree_le` et de `modifie_le`.
    -- Sans lui, la rétention paramétrable de TRX-06 devrait deviner depuis quand la pièce est là,
    -- ou remettre le compteur à zéro à chaque modification de la fiche. Il est posé **maintenant**
    -- pour que la purge s'applique plus tard **sans migration**.
    ADD COLUMN piece_capturee_le   TIMESTAMPTZ NULL;

-- ---------------------------------------------------------------------------------------------
--  ⚠️ `text_pattern_ops` n'est PAS décoratif
-- ---------------------------------------------------------------------------------------------
--
-- Sans lui, un `LIKE 'kouam%'` **n'emploie pas l'index** dès que la collation de la base n'est pas
-- `C` — ce qui est le cas de toute base créée avec une locale. La classe d'opérateurs compare
-- octet à octet, ce qui est précisément ce dont un préfixe a besoin.
--
-- C'est le genre de détail qui se découvre en production, sur le seul écran dont la lenteur
-- condamne le produit : le cadrage §5.6 fait de la rapidité du comptoir une condition d'existence.
CREATE INDEX personne_nom_repli_idx
    ON comptes.personne (tenant_id, nom_repli text_pattern_ops);

-- Les deux autres formes sont des égalités ou des suffixes courts : un B-tree ordinaire suffit.
CREATE INDEX personne_telephone_repli_idx
    ON comptes.personne (tenant_id, telephone_repli);
CREATE INDEX personne_numero_piece_repli_idx
    ON comptes.personne (tenant_id, numero_piece_repli);

COMMENT ON COLUMN comptes.personne.type_piece IS
    'Alimentée depuis SEJ-01 (cycle 006). Chiffrée au repos, accès journalisé au registre des actions. Rétention 90 jours TRX-06 — encore DUE, dette nommée.';
COMMENT ON COLUMN comptes.personne.numero_piece IS
    'Alimentée depuis SEJ-01 (cycle 006). Chiffrée au repos, accès journalisé au registre des actions. Rétention 90 jours TRX-06 — encore DUE, dette nommée. La purge portera sur DEUX tables : celle-ci et hebergement.accompagnant.';
COMMENT ON COLUMN comptes.personne.piece_capturee_le IS
    'Instant de CAPTURE de la pièce — distinct de cree_le et modifie_le. Posé pour que la rétention paramétrable de TRX-06 s''applique sans migration.';
COMMENT ON COLUMN comptes.personne.nom_repli IS
    'Forme cherchable de nom + prenoms : minuscules, sans signes diacritiques, sans apostrophe. Calculée à l''ÉCRITURE — replier à la lecture coûterait la cible des 300 ms.';

-- =============================================================================================
--  2. comptes.client — la qualification
-- =============================================================================================
CREATE TABLE comptes.client (
    -- **L'identifiant EST celui de la personne.** Pas de clé technique séparée : une personne est
    -- cliente ou ne l'est pas, il n'y a pas deux fiches à réconcilier. C'est aussi ce qui rend la
    -- création idempotente sur l'UUID v7 fourni par le terminal, sans second espace d'identité.
    --
    -- Clé étrangère **intra-schéma** — la seule forme que le principe II autorise, et le seul
    -- endroit de ce cycle où une clé étrangère traverse un agrégat.
    personne_id       UUID        PRIMARY KEY REFERENCES comptes.personne (id),
    tenant_id         UUID        NOT NULL,

    -- Les deux attributs que CPT n'a **aucune raison** de connaître (research R-01). Les mettre
    -- sur `personne` chargerait l'identité civile du personnel de champs qui ne le concernent pas.
    date_naissance    DATE        NULL,
    nationalite       TEXT        NULL CHECK (length(btrim(nationalite)) BETWEEN 2 AND 80),

    horodatage_client TIMESTAMPTZ NULL,
    cree_le           TIMESTAMPTZ NOT NULL DEFAULT now(),
    modifie_le        TIMESTAMPTZ NOT NULL DEFAULT now()
);

COMMENT ON TABLE comptes.client IS
    'QUALIFIE une personne comme cliente. Classe hors-ligne C (décision O-01, option (a), tranchée le 2026-08-03). Sans elle, chercher « Kouamé » à la réception ferait apparaître la femme de ménage.';

-- =============================================================================================
--  3. comptes.preference_personne — classe A, append-only
-- =============================================================================================
--
-- **Le patron exact de `note_etablissement`** (module doré, couche 1 — « les privilèges disent la
-- classe »). La préférence courante est **la ligne la plus récente**, jamais une colonne mise à
-- jour. Une correction est une ligne nouvelle.
--
-- C'est ce qui rend le rejeu inoffensif et le désordre commutatif — les deux propriétés que
-- `tester_classe_a!` vérifie, et que `UPDATE` détruirait : deux terminaux vidant leur file dans un
-- ordre quelconque doivent aboutir au même état.
CREATE TABLE comptes.preference_personne (
    -- UUID v7 **généré côté client** (principe VI) : c'est ce qui rend le rejeu inoffensif.
    id                UUID        PRIMARY KEY,
    tenant_id         UUID        NOT NULL,
    personne_id       UUID        NOT NULL REFERENCES comptes.personne (id),

    texte             TEXT        NOT NULL CHECK (length(btrim(texte)) BETWEEN 1 AND 2000),

    -- **Indicatif, et ne porte AUCUNE règle** (porte P-23). Écrire la colonne n'est pas s'appuyer
    -- dessus : elle rend l'instant tel que le terminal l'a perçu, ce qui est l'une des trois
    -- exemptions limitativement énumérées.
    horodatage_client TIMESTAMPTZ NULL,

    -- **AUTORITÉ.** C'est lui qui ordonne les préférences, donc lui qui décide laquelle est
    -- courante.
    cree_le           TIMESTAMPTZ NOT NULL DEFAULT now()
);

COMMENT ON TABLE comptes.preference_personne IS
    'Préférences d''une personne. Classe hors-ligne A, branche A4 — append-only, commutative, sans effet monétaire. La préférence courante est la ligne la plus récente ; ni UPDATE ni DELETE ne sont accordés.';
COMMENT ON COLUMN comptes.preference_personne.horodatage_client IS
    'INDICATIF. Aucune règle ne s''y appuie (porte P-23) ; l''ordre est donné par cree_le.';

CREATE INDEX preference_personne_courante_idx
    ON comptes.preference_personne (tenant_id, personne_id, cree_le DESC);

-- =============================================================================================
--  4. Sécurité au niveau ligne — le patron identique partout
-- =============================================================================================

ALTER TABLE comptes.client ENABLE ROW LEVEL SECURITY;
ALTER TABLE comptes.client FORCE  ROW LEVEL SECURITY;
CREATE POLICY isolation_tenant ON comptes.client
    USING      (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

ALTER TABLE comptes.preference_personne ENABLE ROW LEVEL SECURITY;
ALTER TABLE comptes.preference_personne FORCE  ROW LEVEL SECURITY;
CREATE POLICY isolation_tenant ON comptes.preference_personne
    USING      (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

-- =============================================================================================
--  5. Privilèges — et ce que leur asymétrie dit
-- =============================================================================================
--
-- **`client` reçoit `UPDATE`** : une fiche se corrige — un nom mal orthographié, un téléphone qui
-- change. **`preference_personne` ne le reçoit pas** : elle est append-only, et le privilège
-- absent est ce qui rend la classe A **impossible à contourner** par une ligne de code écrite
-- au-dessus. Les privilèges disent la classe (module doré).
--
-- **Aucune des deux n'a `DELETE`.** La suppression d'une personne relève de TRX-06, qui apportera
-- l'export, la suppression et la purge paramétrable — et qui devra alors décider du privilège en
-- connaissance de cause.
GRANT SELECT, INSERT, UPDATE ON comptes.client              TO kaya_app;
GRANT SELECT, INSERT         ON comptes.preference_personne TO kaya_app;
