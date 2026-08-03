-- 0030 — SEJ : les sept permissions du cycle, et les **premières transversales du produit
-- attachées à une notion de client**.
--
-- Sur le patron de `0022` (cycle 004), avec une différence qui compte : **deux des sept portent
-- `module_code = NULL`**.
--
-- ## Pourquoi `sej.client.*` est transversale et `heb.sejour.*` ne l'est pas
--
-- La fiche client **ne dépend d'aucun module d'activité** (research R-13). Un maquis seul, un bar
-- seul en auront besoin dès **SEJ-05** — la vente à un client extérieur, sans hébergement. Les
-- rattacher à `HEBERGEMENT` obligerait, ce jour-là, soit à créer une seconde permission de client,
-- soit à activer un module d'hébergement dans un maquis pour lire une fiche.
--
-- Les cinq autres gardent des opérations qui **n'ont de sens qu'avec un hébergement** : ouvrir un
-- séjour suppose une chambre.
--
-- ## Toujours aucune clé étrangère vers `etablissements.module_activite`
--
-- Ce serait une clé inter-schémas (principe II, porte P-04). La cohérence est tenue par
-- `backend/tests/permissions_par_module.rs`, pas par la base — régime posé en `0022`.
--
-- L'`INSERT` fonctionne **après** l'activation de la sécurité au niveau ligne grâce à la politique
-- `administration_editeur ... FOR ALL TO kaya_owner` posée en `0016`. Sans elle, l'insertion
-- réussirait **en n'écrivant rien**, sans erreur — le piège du module doré.

-- =============================================================================================
--  1. Les sept permissions
-- =============================================================================================
--
-- **Chacune garde une opération réellement servie par ce cycle.** C'est la règle du cycle 003 —
-- une permission sans contrepartie est une promesse — et `couverture_portes.rs` la vérifie.
--
--   sej.client.lire          → opérations 1, 3, 5 (recherche, lecture, historique)
--   sej.client.gerer         → opérations 2, 4, 6 (création, modification, préférence)
--   heb.sejour.lire          → opérations 5, 8, 9, 16 (historique, liste, fiche, fiche de police)
--   heb.sejour.ouvrir        → opérations 7, 10, 11, 12 (ouverture, rattachement, accompagnants)
--   heb.sejour.clore         → opération 15
--   heb.sejour.prolonger     → opération 13
--   heb.sejour.changer_unite → opération 14
INSERT INTO comptes.permission (code, module_code, libelle_cle, ordre) VALUES
    ('sej.client.lire',          NULL,          'comptes.permissions.sej.client.lire',          230),
    ('sej.client.gerer',         NULL,          'comptes.permissions.sej.client.gerer',         240),
    ('heb.sejour.lire',          'HEBERGEMENT', 'comptes.permissions.heb.sejour.lire',          250),
    ('heb.sejour.ouvrir',        'HEBERGEMENT', 'comptes.permissions.heb.sejour.ouvrir',        260),
    ('heb.sejour.clore',         'HEBERGEMENT', 'comptes.permissions.heb.sejour.clore',         270),
    ('heb.sejour.prolonger',     'HEBERGEMENT', 'comptes.permissions.heb.sejour.prolonger',     280),
    ('heb.sejour.changer_unite', 'HEBERGEMENT', 'comptes.permissions.heb.sejour.changer_unite', 290);

-- =============================================================================================
--  2. Attribution aux rôles
-- =============================================================================================

-- **`receptionniste` et `gerant` — les sept.**
--
-- C'est Yao qui enregistre, prolonge, change de chambre et fait partir. Le cycle 004 lui avait
-- donné quatre permissions d'écriture sur cinq ; celui-ci lui donne **tout le parcours du
-- séjour**, ce qui est exactement son métier. Un réceptionniste qui ne pourrait pas clore un
-- séjour renverrait le client vers le gérant à chaque départ.
INSERT INTO comptes.role_permission (role_code, permission_code)
SELECT r.code, p.code
FROM comptes.role r
CROSS JOIN comptes.permission p
WHERE r.code IN ('receptionniste', 'gerant')
  AND p.code IN ('sej.client.lire', 'sej.client.gerer', 'heb.sejour.lire',
                 'heb.sejour.ouvrir', 'heb.sejour.clore', 'heb.sejour.prolonger',
                 'heb.sejour.changer_unite');

-- **`proprietaire` — les DEUX lectures seulement, et c'est délibéré.**
--
-- Le propriétaire consulte : il veut savoir qui est passé et ce qui a été facturé, ce que le
-- registre des actions et ces deux lectures lui donnent. Il n'enregistre pas d'arrivée — et lui
-- donner `heb.sejour.ouvrir` « au cas où » rendrait le registre des actions moins lisible en y
-- mêlant des gestes qu'il ne fait pas, alors que c'est **ce que le propriétaire achète**
-- (cadrage §8.3).
--
-- ⚠️ **Écart assumé avec `0022`**, où `proprietaire` recevait les cinq permissions d'hébergement,
-- y compris `heb.unite.attribuer`. Là, il s'agissait de **régler l'offre** — tarifs, chambres,
-- formules — qui est bien son geste. Ici il s'agit d'**exploiter le comptoir**, qui ne l'est pas.
INSERT INTO comptes.role_permission (role_code, permission_code)
SELECT 'proprietaire', code
FROM comptes.permission
WHERE code IN ('sej.client.lire', 'heb.sejour.lire');

-- **`admin_editeur` — les sept.** Portée `EDITEUR`, il agit au-dessus des tenants (ETB-08). La
-- distribution de `0016` lui donnait « tout » par un `SELECT code FROM comptes.permission` ; cette
-- forme n'ajoute rien rétroactivement, d'où la reprise explicite — même remarque qu'en `0022`.
INSERT INTO comptes.role_permission (role_code, permission_code)
SELECT 'admin_editeur', code
FROM comptes.permission
WHERE code IN ('sej.client.lire', 'sej.client.gerer', 'heb.sejour.lire',
               'heb.sejour.ouvrir', 'heb.sejour.clore', 'heb.sejour.prolonger',
               'heb.sejour.changer_unite');

-- ---------------------------------------------------------------------------------------------
--  Les quatre rôles qui n'obtiennent RIEN ici, et pourquoi
-- ---------------------------------------------------------------------------------------------
--
-- `serveur`, `caissier`, `magasinier` et `comptable` ne reçoivent aucune permission de ce cycle.
--
-- Le cas du **caissier** mérite d'être écrit, parce qu'il paraîtra faux : le départ produit une
-- note, et encaisser semble être son geste. **L'encaissement n'est pas dans ce cycle** — il est
-- dans CAI, tranche T2 —, et la note se clôt ici **arrêtée et non réglée**. Le jour où CAI
-- arrivera, le caissier recevra une permission d'encaissement, pas `heb.sejour.clore`.
--
-- Le **serveur** portera une consommation sur une chambre à SEJ-03 (T2) : il aura alors besoin de
-- lire les séjours ouverts, ce que le trait `LecteurSejour` sert déjà. La permission naîtra avec
-- l'opération, pas avant.
--
-- **Ne pas « équilibrer » cette distribution.** Un rôle dont les permissions arrivent au cycle de
-- ses écrans est un rôle correct (note de `0016`).
