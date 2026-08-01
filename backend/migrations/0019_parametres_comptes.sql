-- 0019 — CPT-01 : les cinq paramètres d'établissement du cycle.
--
-- **Migration additive**, dans le schéma `etablissements` et non `comptes` : le catalogue des
-- clés de configuration est un référentiel unique du produit (`0008`), pas un objet par module.
-- Chaque cycle y ajoute les siennes.
--
-- L'`INSERT` est possible **après** l'activation de la sécurité au niveau ligne grâce à la
-- politique `administration_editeur` posée en `0008` — c'est exactement le cas prévu par le
-- module doré, « Alimenter un référentiel après son activation ». Sans elle, cet `INSERT`
-- n'écrirait rien **et ne se plaindrait pas**.
--
-- Les cinq clés figurent au « Récapitulatif des paramètres d'établissement » de
-- `docs/user-stories-v1.md`, avec leur clé technique entre accents graves :
-- `backend/tests/parametres_catalogue.rs` compare catalogue → récapitulatif et fait échouer le
-- build sur toute clé absente du second (principe I·c).

INSERT INTO etablissements.parametre_catalogue
    (cle, type_valeur, portee_la_plus_basse, story, libelle_cle, description_cle)
VALUES
    -- **Une donnée de JURIDICTION, donc un paramètre — jamais une constante ni un `CHECK`**
    -- (principe V, porte P-12). `+225` codé en dur ferait échouer le premier établissement
    -- togolais, et la validation E.164 qui l'accompagne est un format international, pas une
    -- règle nationale.
    ('indicatif_telephonique_defaut', 'TEXTE', 'ETABLISSEMENT', 'CPT-01',
     'parametres.indicatif_telephonique_defaut.libelle',
     'parametres.indicatif_telephonique_defaut.description'),

    -- Le code de `comptes.methode_authentification`. Le catalogue ne porte pas de clé étrangère
    -- vers ce référentiel : ce serait une clé inter-schémas (porte P-04). La cohérence est tenue
    -- par le service, qui refuse `422 methode_non_implementee` sur une méthode non servie.
    ('methode_authentification', 'TEXTE', 'ETABLISSEMENT', 'CPT-01',
     'parametres.methode_authentification.libelle',
     'parametres.methode_authentification.description'),

    -- **Huit, et aucune règle de composition.** Les règles de composition produisent un mot de
    -- passe sur un post-it au comptoir ; c'est le refus des mots de passe compromis qui fait le
    -- travail à cette longueur (`socle/comptes/src/authentification/`). Le paramètre existe pour
    -- qu'un établissement puisse **monter** ce plancher, jamais pour supprimer le refus qui
    -- l'accompagne.
    ('mot_de_passe_longueur_min', 'ENTIER', 'ETABLISSEMENT', 'CPT-01',
     'parametres.mot_de_passe_longueur_min.libelle',
     'parametres.mot_de_passe_longueur_min.description'),

    -- **60 minutes, et c'est meilleur que 30 PARCE QUE la révocation est portée par Redis.**
    -- Les deux décisions se tiennent : un jeton court est le seul recours quand la révocation
    -- n'est pas immédiate, et il coûte alors des aller-retours sur le pire réseau du produit.
    -- Ici, une session révoquée cesse d'être acceptée à la requête suivante ; la durée du jeton
    -- ne borne donc plus que le délai de prise d'effet d'un **changement de droits**.
    ('jeton_acces_duree_min', 'DUREE_MINUTES', 'ETABLISSEMENT', 'CPT-01',
     'parametres.jeton_acces_duree_min.libelle',
     'parametres.jeton_acces_duree_min.description'),

    -- **90 jours, et cette valeur vient de M. Diarra**, qui « vient une fois par mois ». Elle
    -- oblige à trois contreparties, toutes livrées par ce cycle : rotation à chaque usage,
    -- détection de réutilisation avec révocation de **toute la famille**, et déconnexion à
    -- distance opérationnelle. Sans elles, 90 jours serait un accès ouvert un trimestre.
    ('jeton_rafraichissement_duree_jours', 'ENTIER', 'ETABLISSEMENT', 'CPT-01',
     'parametres.jeton_rafraichissement_duree_jours.libelle',
     'parametres.jeton_rafraichissement_duree_jours.description');

-- ---------------------------------------------------------------------------------------------
--  AUCUNE VALEUR PAR DÉFAUT N'EST POSÉE ICI, et c'est le principe I·c
-- ---------------------------------------------------------------------------------------------
--
-- Le catalogue déclare qu'une clé **existe**, son type et jusqu'où elle se surcharge. Les valeurs
-- vivent dans `parametre_configuration`, écrites par la configuration d'un tenant — jamais par
-- une migration, qui n'a pas de tenant courant et n'écrirait rien en silence sur une table en
-- `FORCE ROW LEVEL SECURITY`.
--
-- `ResolveurConfiguration` rend `Option<ValeurResolue>`, **jamais un défaut** : l'appelant qui a
-- besoin d'un défaut le déclare chez lui, où on peut le voir.
