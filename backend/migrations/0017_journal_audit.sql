-- 0017 — CPT-04 : le registre des actions.
--
-- **Ce que M. Koffi achète.** CPT-04 le qualifie de « module de premier plan, pas un journal
-- technique » : c'est l'écran qu'un propriétaire ouvre quand il cherche qui a accordé une remise,
-- annulé une ligne partie en cuisine ou ouvert le tiroir un dimanche.
--
-- Terme utilisateur : **« Registre des actions »** (`docs/design/lexique.md`). « Journal
-- d'audit » est le nom technique — table, permission, endpoint — et n'apparaît jamais à l'écran.
--
-- **Classe hors-ligne A**, seconde entité A du produit après `note_etablissement` : append-only,
-- commutative, sans contrainte d'unicité métier. L'entrée voyage avec l'opération qu'elle trace,
-- et garde sa propre classe : tracer une ouverture de tiroir hors ligne est A, même quand
-- l'opération tracée est B.
--
-- **Ce n'est PAS l'outbox** (research R-08). Deux registres, deux publics, deux classes : le
-- grand livre alimente les projections, celui-ci est lu par un humain dans une interface.

CREATE TABLE comptes.journal_audit (
    -- UUID v7 **client** — c'est ce qui rend le rejeu inoffensif. Trois soumissions de la même
    -- entrée entrent en conflit de clé primaire et produisent un enregistrement unique.
    id                UUID        PRIMARY KEY,

    tenant_id         UUID        NOT NULL,

    -- **Aucune clé étrangère** : `etablissements` est un autre module (principe II, porte P-04).
    -- `NULL` pour une action de portée tenant ou éditeur.
    etablissement_id  UUID        NULL,

    -- Taxonomie fermée de `docs/taxonomie-audit.md`, dix familles. La colonne est du `TEXT` sans
    -- `CHECK ... IN` : c'est l'**énumération Rust** `TypeActionAudit` qui la ferme, et le harnais
    -- `backend/tests/audit_taxonomie.rs` qui vérifie l'accord entre le code et le document. Un
    -- `CHECK` littéral ici imposerait une migration à chaque famille branchée, c'est-à-dire à
    -- chacun des sept cycles suivants.
    type_action       TEXT        NOT NULL,

    -- **C'est cette clé étrangère qui rend FR-014 structurel** : tant qu'une entrée d'audit
    -- désigne un compte, ce compte ne peut pas être supprimé. La désactivation, elle, est un
    -- `UPDATE` de `actif` — et se trace ici, sous le type `suppression`.
    auteur_compte_id  UUID        NOT NULL REFERENCES comptes.compte (id),

    -- Sur quoi l'action a porté. `cible_type` est libre — « compte », « unite », « ligne_vente » —
    -- parce que les cibles appartiennent à des modules qui n'existent pas encore.
    cible_type        TEXT        NOT NULL,
    cible_id          UUID        NULL,

    -- ⚠️ **LE POINT OÙ LE PRINCIPE V CESSAIT DE TENIR.**
    --
    -- Un document JSON accepte `12500.5` ou `"12 500 F"` là où le principe impose un entier
    -- d'unité mineure — et ce registre trace précisément les **écarts de caisse**, les
    -- **modifications de tarif** et les **remises**, c'est-à-dire les trois choses qu'on consulte
    -- pour détecter une fraude. Un écart stocké en flottant, et l'audit ment sur le montant qu'il
    -- est censé prouver.
    --
    -- La constitution **1.6.0** étend P-10 en conséquence. Convention imposée :
    --
    --     { "ecart_mineur": -12500, "devise": "XOF", "motif": "…" }
    --
    --   * toute clé monétaire porte le suffixe `_mineur` ;
    --   * sa valeur est un **entier**, jamais un décimal ni une chaîne formatée ;
    --   * une clé `devise` l'accompagne **au même niveau d'objet** — le nombre de décimales vient
    --     de la devise, jamais d'une constante.
    --
    -- Vérifié à **deux** niveaux, et les deux sont nécessaires : `scripts/ci/types-monetaires.sh`
    -- ne voit pas un document construit dynamiquement par un service, et la validation du service
    -- ne voit pas un littéral mal nommé dans du code qui ne s'exécute pas encore.
    --
    -- Aucun montant de ce cycle n'entre encore ici : **le contrôle est posé avant le premier**.
    contexte          JSONB       NOT NULL DEFAULT '{}'::jsonb,

    -- Indicatif — ordre d'affichage local sur un terminal. **Aucune règle ne s'y appuie**, et
    -- surtout aucun tri : trier sur l'horloge d'un terminal ferait remonter en tête l'entrée d'un
    -- appareil mal réglé, dans le registre même qui sert à établir une chronologie.
    horodatage_client TIMESTAMPTZ NULL,

    -- **AUTORITÉ SERVEUR.** C'est lui que l'écran `G4` affiche, et lui seul.
    cree_le           TIMESTAMPTZ NOT NULL DEFAULT now()
);

COMMENT ON TABLE comptes.journal_audit IS
    'Registre des actions (terme utilisateur). Classe hors-ligne A : append-only, immuable. GRANT SELECT, INSERT seulement — ni UPDATE ni DELETE.';
COMMENT ON COLUMN comptes.journal_audit.contexte IS
    'JSONB. Toute clé monétaire porte le suffixe `_mineur`, une valeur ENTIÈRE, et une clé `devise` au même niveau (P-10 étendue, constitution 1.6.0).';
COMMENT ON COLUMN comptes.journal_audit.horodatage_client IS
    'Indicatif. JAMAIS un critère de tri : le registre sert à établir une chronologie, et un terminal mal réglé la fausserait.';

-- =============================================================================================
--  Les trois index de filtre — un par filtre de FR-037
-- =============================================================================================
--
-- L'ordre de lecture est `cree_le DESC, id DESC` (module doré, couche 3). Le départage par UUID
-- v7 n'est pas décoratif : deux entrées écrites dans la même transaction partagent `now()`, et
-- sans lui la pagination par curseur sauterait ou répéterait des lignes. L'UUID v7 étant ordonné
-- dans le temps, il départage dans le bon sens.
--
-- `tenant_id` est en tête des trois : la politique de sécurité filtre dessus à chaque accès.

CREATE INDEX journal_audit_par_etablissement_idx
    ON comptes.journal_audit (tenant_id, etablissement_id, cree_le DESC);

CREATE INDEX journal_audit_par_auteur_idx
    ON comptes.journal_audit (tenant_id, auteur_compte_id, cree_le DESC);

CREATE INDEX journal_audit_par_type_idx
    ON comptes.journal_audit (tenant_id, type_action, cree_le DESC);

-- =============================================================================================
--  Sécurité au niveau ligne
-- =============================================================================================

ALTER TABLE comptes.journal_audit ENABLE ROW LEVEL SECURITY;
ALTER TABLE comptes.journal_audit FORCE  ROW LEVEL SECURITY;

CREATE POLICY isolation_tenant ON comptes.journal_audit
    USING      (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

-- =============================================================================================
--  Privilèges — ILS SONT L'IMMUABILITÉ
-- =============================================================================================
--
-- **`SELECT, INSERT` — ni `UPDATE`, ni `DELETE`.** C'est le patron de classe A du module doré, et
-- ici c'est aussi ce qui tient FR-033.
--
-- L'immuabilité ne repose donc ni sur une convention de rédaction, ni sur l'absence de chemin de
-- code : elle repose sur un privilège que `kaya_app` n'a pas. Un `UPDATE` écrit par distraction
-- échoue à l'exécution, pas en revue — et un script de maintenance lancé sous `kaya_app` échoue
-- aussi.
--
-- `scripts/ci/outbox-sans-purge.sh`, étendu par la constitution 1.6.0 à la **catégorie « registre
-- immuable »**, double le contrôle côté statique. Ses **deux versants** comptent :
-- `backend/tests/audit_immuabilite.rs` vérifie qu'aucun chemin de purge n'existe **et** qu'une
-- entrée s'écrit et se relit — sans le second, supprimer la table suffirait à passer au vert.
GRANT SELECT, INSERT ON comptes.journal_audit TO kaya_app;
