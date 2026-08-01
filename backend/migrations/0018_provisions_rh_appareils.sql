-- 0018 — CPT-00 / CPT-05 / CPT-06 : les deux provisions.
--
-- **Prêt ≠ construit** (principe X). `employe` et `appareil_enrole` existent en tant que tables,
-- avec leurs colonnes et leur sécurité, et **rien d'autre** : aucun point d'entrée d'API, aucun
-- service, aucun écran. `backend/tests/provisions_sans_logique.rs` le vérifie.
--
-- Elles sont dans une migration **séparée** pour une raison de lecture : leur absence de
-- privilèges se voit d'un coup d'œil, au lieu d'être noyée parmi les `GRANT` de six autres
-- tables.

-- =============================================================================================
--  1. employe — le contrat de travail, jamais confondu avec le compte
-- =============================================================================================
--
-- **C'est la troisième table de CPT-00**, celle qui rend la distinction réelle. Une femme de
-- ménage a une `personne` et un `employe`, aucun `compte`. Un comptable externe a une `personne`
-- et un `compte`, aucun `employe`. Les fusionner « puisque c'est la même personne » est la faute
-- que FR-004 interdit et que `backend/tests/personne_compte_employe.rs` attrape.
CREATE TABLE comptes.employe (
    id               UUID        PRIMARY KEY,
    tenant_id        UUID        NOT NULL,

    -- Clé étrangère intra-schéma — autorisée. C'est le lien qui dit « cet employé est cette
    -- personne », sans que la personne devienne un employé.
    personne_id      UUID        NOT NULL REFERENCES comptes.personne (id),

    -- Pas de `REFERENCES` : autre module (porte P-04).
    etablissement_id UUID        NULL,

    date_embauche    DATE        NULL,
    numero_cnps      TEXT        NULL,

    -- ⚠️ **`BIGINT` D'UNITÉ MINEURE DÈS LA PROVISION** (principe V, porte P-10).
    --
    -- C'est la seule colonne monétaire du cycle. La poser en `NUMERIC` « puisque personne ne s'en
    -- sert » imposerait de migrer **toutes les lignes le jour de la paie** — c'est-à-dire le jour
    -- où la table est enfin peuplée, donc le pire moment possible. Une provision se pose juste du
    -- premier coup, sinon elle ne sert à rien.
    salaire_mineur   BIGINT      NULL,

    -- **Le nombre de décimales vient de la DEVISE, jamais d'une constante** (principe V). Un
    -- salaire de 250 000 XOF s'écrit `250000` avec zéro décimale ; le même montant en EUR
    -- s'écrirait en centimes. Sans cette colonne, il faudrait deviner.
    devise_code      TEXT        NULL,

    cree_le          TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT employe_devise_si_salaire
        CHECK ((salaire_mineur IS NULL) = (devise_code IS NULL))
);

COMMENT ON TABLE comptes.employe IS
    'PROVISION CPT-05 — contrat de travail. Aucune logique, aucun endpoint, AUCUN privilège pour kaya_app. Classe hors-ligne C (§10 du registre).';
COMMENT ON COLUMN comptes.employe.salaire_mineur IS
    'ENTIER d''unité mineure dès la provision (porte P-10). Le nombre de décimales vient de devise_code.';

-- =============================================================================================
--  2. appareil_enrole — et l'adresse MAC qui n'y sera JAMAIS
-- =============================================================================================
CREATE TABLE comptes.appareil_enrole (
    id                      UUID        PRIMARY KEY,
    tenant_id               UUID        NOT NULL,
    compte_id               UUID        NOT NULL REFERENCES comptes.compte (id),
    etablissement_id        UUID        NULL,

    libelle                 TEXT        NULL,

    -- CPT-05 — paire de clés générée dans le **Keystore Android / Keychain iOS**, qui signe
    -- chaque requête. Seule la clé publique voyage jusqu'ici ; la privée ne quitte jamais
    -- l'enclave du terminal.
    cle_publique            TEXT        NULL,
    enrole_le               TIMESTAMPTZ NULL,
    revoque_le              TIMESTAMPTZ NULL,

    -- CPT-06 — Play Integrity (Android), DeviceCheck + App Attest (iOS), vérifiés **côté
    -- serveur**. Un verdict rendu par le terminal sur lui-même ne vaut rien.
    attestation_verdict     TEXT        NULL,
    attestation_verifiee_le TIMESTAMPTZ NULL,

    -- Géorepérage **SOUPLE** : alerte au gérant, **jamais blocage** (CPT-06, cadrage §12.2). Un
    -- caissier qui ne peut pas encaisser parce que le GPS dérive est un client perdu.
    --
    -- `NUMERIC`, jamais un flottant — même règle que les quantités (principe V). Une latitude en
    -- `double precision` accumule une dérive de représentation qui, sur un rayon de 300 m, n'est
    -- pas anecdotique.
    derniere_latitude       NUMERIC     NULL,
    derniere_longitude      NUMERIC     NULL,
    derniere_position_le    TIMESTAMPTZ NULL,

    cree_le                 TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ---------------------------------------------------------------------------------------------
--  L'ABSENCE D'ADRESSE MAC EST UNE DÉCISION, PAS UN OUBLI (FR-042, principe IX, cadrage §12.2)
-- ---------------------------------------------------------------------------------------------
--
-- Le verrouillage par adresse MAC est **techniquement impossible** : iOS et Android randomisent
-- la MAC par réseau depuis iOS 14 et Android 10, et la valeur lue change d'un jour à l'autre sur
-- le même appareil. Une colonne `adresse_mac` produirait donc un verrouillage qui se déverrouille
-- tout seul — c'est-à-dire une sécurité qui ment.
--
-- Ce qui la remplace est l'enrôlement par **paire de clés Keystore/Keychain** ci-dessus :
-- l'appareil prouve qu'il détient une clé privée qu'il ne peut pas exporter, ce qu'aucune
-- randomisation ne défait.
--
-- Écrit ici parce que la demande reviendra : « pourquoi ne pas simplement bloquer par MAC ? » est
-- la première question posée sur ce sujet, à chaque fois.
COMMENT ON TABLE comptes.appareil_enrole IS
    'PROVISION CPT-05/CPT-06. AUCUNE colonne d''adresse MAC, et il n''y en aura jamais (FR-042) : iOS et Android la randomisent. Remplacée par une paire de clés Keystore/Keychain.';

-- Le rayon de géorepérage — 300 m par défaut — est un **paramètre de configuration** du catalogue
-- d'ETB-04, pas une colonne d'ici : c'est un réglage d'établissement, pas une propriété
-- d'appareil. Deux appareils du même établissement n'ont pas deux rayons.

-- =============================================================================================
--  Sécurité au niveau ligne — POSÉE QUAND MÊME
-- =============================================================================================
--
-- La porte **P-07 ne connaît pas d'exception** : `ENABLE` + `FORCE` + au moins une politique sur
-- toute table. Et surtout, **une table sans politique aujourd'hui est une table sans politique le
-- jour où on l'ouvre** — le cycle qui l'implémentera pensera à son métier, pas à l'isolation.

ALTER TABLE comptes.employe          ENABLE ROW LEVEL SECURITY;
ALTER TABLE comptes.employe          FORCE  ROW LEVEL SECURITY;
ALTER TABLE comptes.appareil_enrole  ENABLE ROW LEVEL SECURITY;
ALTER TABLE comptes.appareil_enrole  FORCE  ROW LEVEL SECURITY;

CREATE POLICY isolation_tenant ON comptes.employe
    USING      (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

CREATE POLICY isolation_tenant ON comptes.appareil_enrole
    USING      (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

-- =============================================================================================
--  Privilèges — AUCUN. PAS MÊME `SELECT`.
-- =============================================================================================
--
-- C'est ce qui distingue une provision d'un début d'implémentation. Un chemin de code écrit par
-- distraction — une lecture « juste pour afficher le nom de l'employé » — **échoue au premier
-- appel**, pas trois mois plus tard quand quelqu'un s'apercevra que la table n'a jamais été
-- pensée pour ça.
--
-- Le `GRANT` viendra avec le cycle qui construit l'écran, dans la même migration que le reste.
--
-- Aucune ligne `GRANT` ci-dessous, et c'est délibéré. Ne pas en ajouter « pour pouvoir tester » :
-- `backend/tests/provisions_sans_logique.rs` teste précisément cette absence, sous le rôle
-- `kaya_app`, et un `GRANT SELECT` ajouté par commodité le ferait échouer.
