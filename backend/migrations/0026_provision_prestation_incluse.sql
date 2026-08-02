-- 0026 — HEB-09 : la prestation incluse. **Une table, et rien d'autre.**
--
-- **Prêt ≠ construit** (principe X). La table existe avec ses colonnes, ses contraintes et sa
-- sécurité ; il n'y a **aucun endpoint, aucun service, aucun écran, et aucun privilège** pour
-- `kaya_app`. `backend/tests/provisions_sans_logique.rs` le vérifie, et un `GRANT` ajouté « pour
-- pouvoir tester » ferait échouer ce test — c'est exactement son objet.
--
-- # Pourquoi cette table est posée maintenant, alors que la fonctionnalité est en incrément 2
--
-- **Le petit-déjeuner inclus est une pratique répandue dans l'hôtellerie ivoirienne**, et il
-- n'apparaissait nulle part dans le périmètre initial (user stories, HEB-09). Que le pilote le
-- pratique ou non, le modèle doit savoir le porter : d'autres établissements le proposeront. Ce
-- n'est pas de l'ambition plateforme, c'est une lacune du produit hôtelier, comblée comme telle.
--
-- Ce que la fonctionnalité fera, en incrément 2 : la prestation s'affiche sur la note, se décompte
-- à la consommation, n'est pas facturée, et **le dépassement du quota bascule en facturation
-- normale** avec mention explicite. Rien de tout cela n'est ici.
--
-- Migration **séparée**, comme `0018` : l'absence de privilèges se voit d'un coup d'œil au lieu
-- d'être noyée parmi les `GRANT` d'autres tables.

CREATE TABLE hebergement.prestation_incluse (
    id                             UUID        PRIMARY KEY,
    tenant_id                      UUID        NOT NULL,

    -- Clé étrangère **intra-schéma** — autorisée (principe II, porte P-04). La prestation est
    -- attachée à la FORMULE, jamais à l'unité ni au séjour : c'est l'offre qui inclut le
    -- petit-déjeuner, pas la chambre.
    formule_id                     UUID        NOT NULL
                                   REFERENCES hebergement.formule (id) ON DELETE CASCADE,

    -- Texte libre **à ce stade**, et c'est délibéré : la liste des types de prestation n'est pas
    -- arrêtée — petit-déjeuner, blanchisserie, conciergerie sont cités, la liste ne l'est pas. Un
    -- `CHECK` posé aujourd'hui sur trois valeurs devrait être migré au premier quatrième type,
    -- alors qu'aucun code ne lit encore cette colonne.
    type_prestation                TEXT        NOT NULL,

    -- ⚠️ **`NUMERIC`, JAMAIS un entier** (principe V, porte P-10).
    --
    -- Un petit-déjeuner se compte à l'unité, une prestation de blanchisserie **au kilo**, une
    -- course de conciergerie peut se compter en demi-heures. Poser `INTEGER` ici « puisque
    -- personne ne s'en sert » imposerait de migrer **toutes les lignes de tous les clients** le
    -- jour où la table est enfin peuplée — c'est-à-dire au pire moment. Une provision se pose
    -- juste du premier coup, sinon elle ne sert à rien.
    quantite                       NUMERIC     NOT NULL
        CONSTRAINT prestation_incluse_quantite_positive CHECK (quantite > 0),

    -- **Entier d'unité mineure** (principe V, porte P-10). Le plafond de valeur unitaire au-delà
    -- duquel le dépassement bascule en facturation normale. `>= 0` et non `> 0` : un plafond à
    -- zéro dit « incluse sans limite de valeur », ce qui est un réglage, pas une absence.
    valeur_unitaire_plafond_mineur BIGINT      NOT NULL
        CONSTRAINT prestation_incluse_plafond_non_negatif
            CHECK (valeur_unitaire_plafond_mineur >= 0),

    cree_le                        TIMESTAMPTZ NOT NULL DEFAULT now()
);

COMMENT ON TABLE hebergement.prestation_incluse IS
    'PROVISION HEB-09 — table seulement. Aucun endpoint, aucune logique, aucun écran. La fonctionnalité arrive en incrément 2. Classe hors-ligne C (registre §7.1).';
COMMENT ON COLUMN hebergement.prestation_incluse.quantite IS
    'NUMERIC, jamais entier (porte P-10) : un petit-déjeuner se compte à l''unité, une blanchisserie au kilo.';
COMMENT ON COLUMN hebergement.prestation_incluse.valeur_unitaire_plafond_mineur IS
    'ENTIER d''unité mineure dès la provision (porte P-10). Le nombre de décimales vient de la devise de l''établissement.';

-- =============================================================================================
--  Sécurité au niveau ligne — POSÉE QUAND MÊME
-- =============================================================================================
--
-- La porte **P-07 ne connaît pas d'exception** : `ENABLE` + `FORCE` + au moins une politique sur
-- toute table. Et surtout, **une table sans politique aujourd'hui est une table sans politique le
-- jour où on l'ouvre** — le cycle qui l'implémentera pensera à son métier, pas à l'isolation.

ALTER TABLE hebergement.prestation_incluse ENABLE ROW LEVEL SECURITY;
ALTER TABLE hebergement.prestation_incluse FORCE  ROW LEVEL SECURITY;

CREATE POLICY isolation_tenant ON hebergement.prestation_incluse
    USING      (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);

-- =============================================================================================
--  Privilèges — AUCUN. PAS MÊME `SELECT`.
-- =============================================================================================
--
-- C'est ce qui distingue une provision d'un début d'implémentation. Un chemin de code écrit par
-- distraction — une lecture « juste pour afficher le petit-déjeuner inclus » — **échoue au premier
-- appel**, pas trois mois plus tard.
--
-- Les six tables de `0024` accordent les quatre verbes à `kaya_app` ; celle-ci n'en accorde aucun,
-- et la différence se lit d'un fichier à l'autre. Le `GRANT` viendra avec le cycle qui construit
-- l'écran, dans la même migration que le reste.
--
-- Aucune ligne `GRANT` ci-dessous, et c'est délibéré.
