-- 0023 — HEB : les trois paramètres d'établissement du cycle.
--
-- **Migration additive**, dans le schéma `etablissements` et non `hebergement` : le catalogue des
-- clés de configuration est un référentiel unique du produit (`0008`), pas un objet par module.
-- Chaque cycle y ajoute les siennes — c'est ce que `0019` a fait pour CPT.
--
-- L'`INSERT` est possible **après** l'activation de la sécurité au niveau ligne grâce à la
-- politique `administration_editeur` posée en `0008`. Sans elle, il n'écrirait rien **et ne se
-- plaindrait pas**.
--
-- Les trois clés figurent au « Récapitulatif des paramètres d'établissement » de
-- `docs/user-stories-v1.md`, avec leur clé technique entre accents graves :
-- `backend/tests/parametres_catalogue.rs` compare catalogue → récapitulatif et fait échouer le
-- build sur toute clé absente du second (principe I·c).

INSERT INTO etablissements.parametre_catalogue
    (cle, type_valeur, portee_la_plus_basse, story, libelle_cle, description_cle)
VALUES
    -- **`HEURE_LOCALE`, et non `TEXTE`.** Le plan écrivait `TEXTE` ; le type fermé de `0008`
    -- porte `HEURE_LOCALE` et son en-tête dit pourquoi la liste est fermée : « un type de valeur
    -- n'est pas une fonctionnalité produit ». Employer `TEXTE` ici laisserait la validation
    -- accepter « demain matin », et la seule chose qui distinguerait une heure d'une phrase
    -- serait la vigilance de l'appelant. L'écart est réel, il se justifie ici et ne se résorbe
    -- pas en ajustant le plan.
    --
    -- 14 h à Deloria. La valeur elle-même n'est PAS posée ici — voir la note finale.
    ('heure_arrivee_standard', 'HEURE_LOCALE', 'ETABLISSEMENT', 'HEB-03',
     'parametres.heure_arrivee_standard.libelle',
     'parametres.heure_arrivee_standard.description'),

    -- 12 h à Deloria. Deux clés distinctes plutôt qu'une paire : l'exploitant règle l'une sans
    -- l'autre — un hôtel qui recule son départ à 11 h ne touche pas son heure d'arrivée.
    ('heure_depart_standard', 'HEURE_LOCALE', 'ETABLISSEMENT', 'HEB-03',
     'parametres.heure_depart_standard.libelle',
     'parametres.heure_depart_standard.description'),

    -- **`DUREE_MINUTES`, et non `ENTIER`** — même raisonnement que ci-dessus : le nom de la clé
    -- porte l'unité, le type la confirme, et un `ENTIER` nu se serait un jour lu en heures.
    --
    -- 480 min (8 h) à Deloria. Au-delà de ce seuil, un passage **change de formule** : ce n'est
    -- pas un palier majoré, c'est une nuitée (HEB-04). La valeur est un paramètre parce que la
    -- pratique varie d'un établissement à l'autre, jamais une constante du barème.
    ('seuil_bascule_nuitee_minutes', 'DUREE_MINUTES', 'ETABLISSEMENT', 'HEB-04',
     'parametres.seuil_bascule_nuitee_minutes.libelle',
     'parametres.seuil_bascule_nuitee_minutes.description');

-- ---------------------------------------------------------------------------------------------
--  CE QUI NE VA PAS AU CATALOGUE, et pourquoi (research R-16)
-- ---------------------------------------------------------------------------------------------
--
-- Trois valeurs HEB du récapitulatif restent **hors** du catalogue, et l'omission est délibérée :
--
--   * le **temps de remise en état** — il varie par catégorie *ET* par formule. Ce n'est pas un
--     scalaire d'établissement : « 30 min » n'a de sens que rapporté à une catégorie et à une
--     famille de formule. Il vit dans `hebergement.temps_remise_en_etat` ;
--   * les **plages de demi-journée** — le registre §7.1 les classe comme référentiel. Elles
--     vivent dans `hebergement.plage_demi_journee` ;
--   * le **barème de passage** — même motif, table `hebergement.bareme_palier`, dont la clé
--     primaire `(formule_id, duree_minutes)` rend un barème désordonné impossible à constituer.
--
-- Le type `BAREME` existe au catalogue depuis `0008` ; l'employer ici mettrait une suite de
-- couples dans un `JSONB` que rien ne contraint, là où une table porte l'ordre et l'unicité.
--
-- ---------------------------------------------------------------------------------------------
--  AUCUNE VALEUR PAR DÉFAUT N'EST POSÉE ICI, et c'est le principe I·c
-- ---------------------------------------------------------------------------------------------
--
-- Le catalogue déclare qu'une clé **existe**, son type et jusqu'où elle se surcharge. Les valeurs
-- vivent dans `parametre_configuration`, écrites par la configuration d'un tenant — jamais par
-- une migration, qui n'a pas de tenant courant et n'écrirait rien en silence sur une table en
-- `FORCE ROW LEVEL SECURITY`. Les valeurs Deloria sont posées par les **seeds**.
