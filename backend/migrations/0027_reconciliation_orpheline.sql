-- 0027 — SYN-03 : le constat d'écriture orpheline. **Une table, et rien d'autre.**
--
-- **Prêt ≠ construit** (principe X). La table existe avec ses colonnes, ses contraintes et sa
-- sécurité ; il n'y a **aucun endpoint, aucun service, aucun écran**, et `kaya_app` reçoit
-- `SELECT` **seul**. `backend/tests/provisions_sans_logique.rs` le vérifie, et son décompte passe
-- de cinq à six dans le même changement.
--
-- # Ce que cette table constate, et pourquoi elle est posée maintenant
--
-- Une écriture arrive sur un agrégat **déjà clos et facturé** : la serveuse a saisi une commande
-- hors ligne à 22 h 40, le client a réglé et est parti à 23 h, le réseau revient à 23 h 20. Le
-- cadrage §11.4 nomme ce conflit « le plus fréquent en exploitation réelle », et sa résolution est
-- **humaine et obligatoire** : jamais de rejet silencieux, jamais d'ajout d'office sur une facture
-- déjà remise.
--
-- Le cycle 005 livre la file qui rend ce cas atteignable. Poser la table maintenant met le constat
-- à l'endroit où SYN-03 l'écrira, avec sa classe déjà décidée au registre (**A** pour la création,
-- **B** pour la résolution, §5.6, décidées le 2026-07-30).
--
-- **Ce qui n'est pas ici** : l'écran de réconciliation, le service, les endpoints et la logique de
-- résolution. Ils sont de SYN-03, tranche T3, et dépendent des séjours et des documents fiscaux —
-- dont aucun n'existe.
--
-- Migration **séparée**, comme `0018` et `0026` : l'asymétrie des privilèges se voit d'un coup
-- d'œil au lieu d'être noyée parmi les `GRANT` d'autres tables.

CREATE TABLE synchronisation.reconciliation_orpheline (
    -- **UUID v7 fourni par le CLIENT** — patron du module doré, et ce qui rend le rejeu
    -- inoffensif (principe VI). Un constat renvoyé trois fois par un terminal qui vide sa file
    -- produit une ligne, pas trois.
    id                    UUID        PRIMARY KEY,

    tenant_id             UUID        NOT NULL
                          REFERENCES etablissements.tenant (id),

    -- Le constat est **local à un établissement** : c'est là qu'on le tranche, et le filtre de
    -- l'écran SYN-03 partira de là.
    etablissement_id      UUID        NOT NULL
                          REFERENCES etablissements.etablissement (id),

    -- ─────────────────────────────────────────────────────────────────────────────────────────
    --  L'écriture arrivée en retard, et l'agrégat qu'elle a manqué
    -- ─────────────────────────────────────────────────────────────────────────────────────────
    --
    -- **Aucune clé étrangère sur ces quatre colonnes, et c'est le principe II.** L'écriture vit
    -- dans un autre schéma de module — `restauration.ligne_commande`, `hebergement.sejour`,
    -- `fiscalite.document_fiscal` — et une clé étrangère inter-schémas est refusée par la porte
    -- P-04. Le type est **nommé** plutôt que deviné : c'est lui qui dira à SYN-03 quoi rattacher.
    ecriture_id           UUID        NOT NULL,
    ecriture_type         TEXT        NOT NULL
        CONSTRAINT reconciliation_orpheline_ecriture_type_non_vide
            CHECK (length(trim(ecriture_type)) > 0),

    -- `sejour`, `addition`, `bon_de_depot`. Texte libre **à ce stade** : la liste des agrégats
    -- clôturables n'est pas arrêtée — le pressing et la salle de réunion en ajouteront. Un
    -- `CHECK ... IN` sur trois valeurs devrait être migré au premier quatrième type, alors
    -- qu'aucun code ne lit encore cette colonne.
    agregat_type          TEXT        NOT NULL
        CONSTRAINT reconciliation_orpheline_agregat_type_non_vide
            CHECK (length(trim(agregat_type)) > 0),
    agregat_id            UUID        NOT NULL,

    -- ─────────────────────────────────────────────────────────────────────────────────────────
    --  Le cycle de vie — deux états, pas davantage
    -- ─────────────────────────────────────────────────────────────────────────────────────────
    etat                  TEXT        NOT NULL DEFAULT 'constatee'
        CONSTRAINT reconciliation_orpheline_etat_connu
            CHECK (etat IN ('constatee', 'resolue')),

    -- Les trois issues du cadrage §11.4, en **MAJUSCULES FRANÇAISES** comme toute valeur
    -- d'énumération du produit. Nulle tant que le constat n'est pas tranché.
    issue                 TEXT        NULL
        CONSTRAINT reconciliation_orpheline_issue_connue
            CHECK (issue IN ('AVOIR_REFACTURATION', 'PRISE_EN_CHARGE',
                             'RATTACHEMENT_SEJOUR_SUIVANT')),

    -- Qui a tranché. **Sans clé étrangère** : `comptes` est un autre schéma de module.
    resolue_par_compte_id UUID        NULL,

    -- ─────────────────────────────────────────────────────────────────────────────────────────
    --  Les horodatages — l'un fait autorité, l'autre est indicatif
    -- ─────────────────────────────────────────────────────────────────────────────────────────
    --
    -- ⚠️ **`horodatage_client` ne porte AUCUNE règle** (principe IV, porte P-23). Il sert l'ordre
    -- d'affichage local et le rendu de l'instant tel que le terminal l'a perçu — rien d'autre. La
    -- porte refuse tout calcul métier, fiscal, de clôture ou de durée qui s'y appuierait.
    horodatage_client     TIMESTAMPTZ NULL,

    -- **L'horodatage d'autorité**, celui qui fait foi. `now()` rend l'instant du DÉBUT de
    -- transaction : deux constats écrits dans la même transaction le partagent, et le départage
    -- se fait par l'UUID v7, ordonné dans le temps (module doré, couche 1).
    cree_le               TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- Horodatage d'autorité de la résolution.
    resolue_le            TIMESTAMPTZ NULL,

    -- ─────────────────────────────────────────────────────────────────────────────────────────
    --  UNE ÉGALITÉ DE CONDITIONS — sur CHAQUE corollaire, et non sur leur conjonction
    -- ─────────────────────────────────────────────────────────────────────────────────────────
    --
    -- Même patron que le classement d'établissement du module doré : l'état et ses trois
    -- corollaires ne peuvent pas diverger. Un `UPDATE` qui poserait `etat = 'resolue'` en oubliant
    -- l'issue est refusé **par la base**, pas par une revue.
    --
    -- ⚠️ **L'égalité porte sur chaque corollaire séparément, et le détail n'est pas cosmétique.**
    -- La forme évidente — et celle que le modèle de données de ce cycle avait écrite —
    --
    --     (etat = 'resolue') = (issue IS NOT NULL AND resolue_le IS NOT NULL AND … IS NOT NULL)
    --
    -- laisse passer un corollaire **isolé** : sur un constat `constatee`, poser l'issue sans les
    -- deux autres donne `false = (true AND false AND false)`, c'est-à-dire `false = false`, donc
    -- accepté. Un écran qui écrirait les trois champs dans le désordre — ou un `UPDATE` interrompu
    -- entre deux instructions — laisserait une issue sur un conflit non tranché, exactement l'état
    -- que cette contrainte existe pour rendre impossible.
    --
    -- Le trou a été trouvé par le test qui l'exerce (`la_resolution_est_tout_ou_rien`), avant que
    -- la migration ne soit figée. Une provision se pose juste du premier coup, sinon elle ne sert
    -- à rien.
    CONSTRAINT reconciliation_orpheline_resolution_complete CHECK (
        (etat = 'resolue') = (issue IS NOT NULL)
        AND (etat = 'resolue') = (resolue_le IS NOT NULL)
        AND (etat = 'resolue') = (resolue_par_compte_id IS NOT NULL)
    )
);

COMMENT ON TABLE synchronisation.reconciliation_orpheline IS
    'PROVISION SYN-03 — table seulement. Aucun endpoint, aucun service, aucun écran ; kaya_app n''a que SELECT. Le constat qu''une écriture est arrivée sur un agrégat déjà clos et facturé (cadrage §11.4). Classe hors-ligne A à la création, B à la résolution (registre §5.6).';
COMMENT ON COLUMN synchronisation.reconciliation_orpheline.id IS
    'UUID v7 fourni par le client (principe VI) : c''est lui qui rend le rejeu inoffensif.';
COMMENT ON COLUMN synchronisation.reconciliation_orpheline.ecriture_id IS
    'Sans clé étrangère : l''écriture vit dans un autre schéma de module (principe II, porte P-04).';
COMMENT ON COLUMN synchronisation.reconciliation_orpheline.horodatage_client IS
    'INDICATIF. Aucune règle ne s''y appuie — principe IV, porte P-23. L''autorité est cree_le.';
COMMENT ON COLUMN synchronisation.reconciliation_orpheline.cree_le IS
    'HORODATAGE D''AUTORITÉ. Toute règle de durée, de taxe et de clôture part d''ici.';

-- =============================================================================================
--  Index — PARTIEL, sur ce qui se lit réellement
-- =============================================================================================
--
-- La lecture de SYN-03 est « ce qui reste à trancher », jamais l'historique complet : un index
-- total porterait des lignes résolues qu'aucun écran n'interroge. `tenant_id` en tête parce que
-- la politique d'isolation filtre dessus à chaque accès, et le tri par `cree_le DESC` parce que
-- l'écran présente le plus récent d'abord.

CREATE INDEX reconciliation_orpheline_a_traiter_idx
    ON synchronisation.reconciliation_orpheline (tenant_id, etablissement_id, cree_le DESC)
    WHERE etat = 'constatee';

-- =============================================================================================
--  Sécurité au niveau ligne — POSÉE QUAND MÊME
-- =============================================================================================
--
-- La porte **P-07 ne connaît pas d'exception** : `ENABLE` + `FORCE` + au moins une politique sur
-- toute table. Et une table sans politique aujourd'hui est une table sans politique le jour où on
-- l'ouvre — le cycle qui l'implémentera pensera à son métier, pas à l'isolation.

ALTER TABLE synchronisation.reconciliation_orpheline ENABLE ROW LEVEL SECURITY;
ALTER TABLE synchronisation.reconciliation_orpheline FORCE  ROW LEVEL SECURITY;

CREATE POLICY isolation_tenant ON synchronisation.reconciliation_orpheline
    USING      (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

-- =============================================================================================
--  Privilèges — `SELECT` SEUL. Ni INSERT, ni UPDATE, ni DELETE.
-- =============================================================================================
--
-- **C'est ici que la provision se prouve.**
--
-- Le registre déclare pourtant la création en **A** et la résolution en **B** — et les deux
-- classes restent justes. Ce n'est pas la classe qui est différée, c'est l'implémentation.
--
-- Accorder l'`INSERT` dès maintenant serait exactement l'« ajout d'un petit endpoint » que
-- `provisions_sans_logique.rs` existe pour rendre bruyant : un chemin de code écrit par
-- distraction échouerait au premier appel plutôt que d'écrire dans un agrégat dont personne n'a
-- spécifié la résolution.
--
-- Pourquoi `SELECT` alors, quand `0026` n'accorde rien du tout ? Parce que les deux provisions ne
-- posent pas la même question. `prestation_incluse` n'a **aucun** lecteur légitime avant son
-- cycle. Celle-ci en a un, et il est déjà écrit : le récapitulatif de fin de journée devra
-- pouvoir dire « trois constats attendent d'être tranchés » **avant** que SYN-03 ne livre l'écran
-- qui les tranche. Une provision qui interdirait la lecture forcerait ce cycle-là à commencer par
-- une migration de privilège — c'est-à-dire au mauvais moment.

GRANT SELECT ON synchronisation.reconciliation_orpheline TO kaya_app;
