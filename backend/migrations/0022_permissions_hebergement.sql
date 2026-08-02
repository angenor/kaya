-- 0022 — HEB : les cinq permissions, et les PREMIÈRES du produit rattachées à un module.
--
-- La migration `0016` du cycle 003 l'annonce nommément : « `module_code` restera donc `NULL`
-- **jusqu'au cycle HEB, qui apportera `heb.unite.attribuer`** ». Ce fichier honore cette phrase
-- à la lettre — et c'est la première fois que le test qui lit le référentiel des modules à
-- travers le trait `RegistreModules` vérifie autre chose que `NULL`. Jusqu'ici sa cible était
-- vide au sens de la constitution.
--
-- **Toujours aucune clé étrangère** vers `etablissements.module_activite` : ce serait une clé
-- inter-schémas (principe II, porte P-04). La cohérence est tenue par ce test, pas par la base.
--
-- L'`INSERT` fonctionne **après** l'activation de la sécurité au niveau ligne grâce à la
-- politique `administration_editeur ... FOR ALL TO kaya_owner` posée en `0016`. Sans elle,
-- l'insertion réussirait **en n'écrivant rien**, sans erreur — le piège du module doré.

-- =============================================================================================
--  1. Les cinq permissions
-- =============================================================================================
--
-- Elles gardent toutes une action réellement servie par ce cycle : c'est FR-021, et le principe
-- X. Une permission qui ne garde rien est une promesse sans contrepartie.
--
--   heb.offre.lire              → opérations 1, 4, 6 (lecture du référentiel), écrans G2 et G5
--   heb.offre.gerer             → opérations 2, 3, 5, 5b, 7, 8 (écriture du référentiel)
--   heb.disponibilite.consulter → opérations 9 et 12 (disponibilité, calcul de tarif)
--   heb.unite.attribuer         → opération 10
--   heb.unite.liberer           → opération 11
INSERT INTO comptes.permission (code, module_code, libelle_cle, ordre) VALUES
    ('heb.offre.lire',              'HEBERGEMENT', 'comptes.permissions.heb.offre.lire',              180),
    ('heb.offre.gerer',             'HEBERGEMENT', 'comptes.permissions.heb.offre.gerer',             190),
    ('heb.disponibilite.consulter', 'HEBERGEMENT', 'comptes.permissions.heb.disponibilite.consulter', 200),
    ('heb.unite.attribuer',         'HEBERGEMENT', 'comptes.permissions.heb.unite.attribuer',         210),
    ('heb.unite.liberer',           'HEBERGEMENT', 'comptes.permissions.heb.unite.liberer',           220);

-- =============================================================================================
--  2. Attribution aux rôles
-- =============================================================================================

-- **`proprietaire` et `gerant` — les cinq.** Ils règlent l'offre et exploitent l'établissement.
INSERT INTO comptes.role_permission (role_code, permission_code)
SELECT r.code, p.code
FROM comptes.role r
CROSS JOIN comptes.permission p
WHERE r.code IN ('proprietaire', 'gerant')
  AND p.module_code = 'HEBERGEMENT';

-- **`receptionniste` — quatre sur cinq : tout sauf `heb.offre.gerer`.**
--
-- Yao attribue des chambres et les libère ; il ne fixe pas les tarifs. C'est la première fois du
-- produit qu'un rôle opérationnel gagne une permission d'ÉCRITURE — le commentaire de `0016`
-- l'annonçait : « leurs actions propres appartiennent aux cycles SEJ, PDV, CAI et STK, et leurs
-- permissions naîtront avec elles ». L'attribution d'unité naît ici, donc la permission aussi.
INSERT INTO comptes.role_permission (role_code, permission_code)
SELECT 'receptionniste', code
FROM comptes.permission
WHERE module_code = 'HEBERGEMENT'
  AND code <> 'heb.offre.gerer';

-- **`admin_editeur` — les cinq.** Portée `EDITEUR`, il agit au-dessus des tenants (ETB-08). La
-- distribution de `0016` lui donnait « tout » par un `SELECT code FROM comptes.permission` ;
-- cette forme n'ajoute rien rétroactivement, d'où la reprise explicite ici.
INSERT INTO comptes.role_permission (role_code, permission_code)
SELECT 'admin_editeur', code
FROM comptes.permission
WHERE module_code = 'HEBERGEMENT';

-- ---------------------------------------------------------------------------------------------
--  Les quatre rôles qui n'obtiennent RIEN ici, et pourquoi
-- ---------------------------------------------------------------------------------------------
--
-- `serveur`, `caissier`, `magasinier` et `comptable` ne reçoivent aucune permission de ce cycle.
-- Ce n'est pas un oubli : aucun d'eux n'attribue de chambre ni ne règle de tarif. Le comptable
-- lit le registre des actions, où les rebascules de palier apparaîtront — il n'a pas besoin de
-- lire l'offre pour cela.
--
-- **Ne pas « équilibrer » cette distribution.** Un rôle dont les permissions arrivent au cycle de
-- ses écrans est un rôle correct (note de `0016`).
