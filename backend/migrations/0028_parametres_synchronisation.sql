-- 0028 — SYN : les deux paramètres d'établissement du cycle.
--
-- **Migration additive**, dans le schéma `etablissements` et non `synchronisation` : le catalogue
-- des clés de configuration est un référentiel unique du produit (`0008`), pas un objet par
-- module. Chaque cycle y ajoute les siennes — `0019` pour CPT, `0023` pour HEB.
--
-- L'`INSERT` est possible **après** l'activation de la sécurité au niveau ligne grâce à la
-- politique `administration_editeur` posée en `0008`. Sans elle, il n'écrirait rien **et ne se
-- plaindrait pas**.
--
-- Les deux clés figurent au « Récapitulatif des paramètres d'établissement » de
-- `docs/user-stories-v1.md`, avec leur clé technique entre accents graves :
-- `backend/tests/parametres_catalogue.rs` compare catalogue → récapitulatif et fait échouer le
-- build sur toute clé absente du second (principe I·c).

INSERT INTO etablissements.parametre_catalogue
    (cle, type_valeur, portee_la_plus_basse, story, libelle_cle, description_cle)
VALUES
    -- **300 secondes, et c'est la valeur du cadrage §11.4 devenue un DÉFAUT, pas une constante.**
    --
    -- Le cadrage écrit « alerte au-delà de 5 minutes de dérive ». Le principe I(c) interdit
    -- d'inscrire une valeur métier en dur : un établissement dont le parc de terminaux est mauvais
    -- doit pouvoir resserrer ce seuil sans livraison, et un autre dont le personnel change l'heure
    -- de l'affichage doit pouvoir l'élargir.
    --
    -- **`ENTIER` en secondes, et non `DUREE_MINUTES`** — l'écart mesuré est une différence
    -- d'instants, exprimée en secondes par le serveur comme par le terminal. La convertir en
    -- minutes pour la ranger, puis la reconvertir pour la comparer, introduirait un arrondi dans
    -- une détection dont tout l'objet est de mesurer un écart. Le nom de la clé porte l'unité.
    --
    -- La détection porte sur la **valeur absolue** de l'écart (SYN-04) : une horloge en avance est
    -- aussi fausse qu'une horloge en retard, et le lexique donne les deux formulations.
    ('sync.derive_horloge_seuil_secondes', 'ENTIER', 'ETABLISSEMENT', 'SYN-04',
     'parametres.sync_derive_horloge_seuil_secondes.libelle',
     'parametres.sync_derive_horloge_seuil_secondes.description'),

    -- **3 000 millisecondes — ce qui rend l'état « connexion faible » TESTABLE.**
    --
    -- `navigator.onLine` dit qu'une interface réseau est active, pas que le serveur répond. À
    -- Abengourou, une 3G qui affiche « en ligne » sans porter la moindre requête est le cas
    -- courant : sans troisième état, le témoin mentirait exactement au moment où il compte.
    --
    -- Sans seuil **nommé**, cet état ne serait pas testable et aucune porte ne pourrait le
    -- distinguer de « connecté ». Le rendre paramétrable coûte une ligne de plus dans cette
    -- migration, et évite qu'un `const 3000` finisse dans un composant.
    --
    -- Le mot « dégradé » n'atteint jamais l'écran : l'utilisateur lit « Connexion faible »
    -- (`docs/design/lexique.md`).
    ('sync.latence_degradee_seuil_ms', 'ENTIER', 'ETABLISSEMENT', 'SYN-02',
     'parametres.sync_latence_degradee_seuil_ms.libelle',
     'parametres.sync_latence_degradee_seuil_ms.description');

-- ---------------------------------------------------------------------------------------------
--  AUCUNE VALEUR PAR DÉFAUT N'EST POSÉE ICI, et c'est le principe I·c
-- ---------------------------------------------------------------------------------------------
--
-- Le catalogue déclare qu'une clé **existe**, son type et jusqu'où elle se surcharge. Les valeurs
-- vivent dans `parametre_configuration`, écrites par la configuration d'un tenant — jamais par une
-- migration, qui n'a pas de tenant courant et n'écrirait rien en silence sur une table en
-- `FORCE ROW LEVEL SECURITY`. Les valeurs Deloria sont posées par les **seeds**.
--
-- ---------------------------------------------------------------------------------------------
--  CE QUI NE VA PAS AU CATALOGUE, et pourquoi
-- ---------------------------------------------------------------------------------------------
--
-- Trois réglages de ce cycle restent **hors** du catalogue, et l'omission est délibérée :
--
--   * l'**intervalle croissant de réessai** de la file — il n'est pas un réglage d'exploitation
--     mais une propriété du protocole d'envoi. Un exploitant qui le doublerait ne réglerait rien
--     de visible ; il ferait seulement attendre ses écritures plus longtemps ;
--   * la **taille maximale de la file** — il n'y en a pas, et c'est une décision : une file qui
--     refuserait une écriture parce qu'elle est pleine ferait perdre le travail qu'elle existe
--     pour sauver ;
--   * les **codes de réponse qui mettent en quarantaine** — la frontière `4xx` / `5xx` est une
--     sémantique HTTP, pas un réglage d'établissement. La rendre paramétrable permettrait de
--     configurer un rejeu infini sur un refus définitif.
