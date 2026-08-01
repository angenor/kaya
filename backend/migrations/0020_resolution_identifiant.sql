-- 0020 — CPT-01 : résoudre un identifiant de connexion **avant que le tenant soit connu**.
--
-- ═════════════════════════════════════════════════════════════════════════════════════════════
--  LE PROBLÈME, QUE NI LE PLAN NI LE MODÈLE DE DONNÉES N'AVAIENT VU
-- ═════════════════════════════════════════════════════════════════════════════════════════════
--
-- `comptes.compte` porte `FORCE ROW LEVEL SECURITY` et une politique qui compare `tenant_id` à
-- `current_setting('app.current_tenant', true)`. Hors requête applicative, ce réglage vaut `NULL`,
-- la comparaison vaut `NULL`, **aucune ligne n'est visible**.
--
-- C'est exactement ce qu'on veut partout ailleurs. Mais la connexion part d'un identifiant et
-- **de rien d'autre** : le contrat annonce `{ identifiant, mot_de_passe, etablissement_id? }`,
-- sans tenant — le tenant est précisément ce qu'elle doit **découvrir**. Sans dérogation,
-- `session_ouvrir` ne trouve jamais aucun compte et le produit n'a pas d'écran d'entrée.
--
-- ═════════════════════════════════════════════════════════════════════════════════════════════
--  QUATRE SOLUTIONS, ET POURQUOI LES TROIS AUTRES SONT ÉCARTÉES
-- ═════════════════════════════════════════════════════════════════════════════════════════════
--
--   1. **Demander le tenant au client** — sous-domaine, ou un champ de plus au formulaire.
--      Adjoua devrait connaître l'identifiant technique de son employeur pour ouvrir sa caisse.
--      Le contrat a tranché autrement, et il a raison.
--
--   2. **Un rôle `BYPASSRLS`.** Il verrait TOUTES les lignes de TOUTES les tables de tous les
--      clients. On échangerait un trou d'un tuple contre un trou de la taille de la base. En
--      outre `kaya_owner` est explicitement `NOBYPASSRLS` depuis `0001`, et l'attribut exige un
--      superutilisateur : le poser demanderait de renoncer à la propriété que `0001` défend.
--
--   3. **`SECURITY DEFINER` détenue par `kaya_owner`.** *C'est la solution qu'on écrit d'abord,
--      et elle ne marche pas.* `SECURITY DEFINER` change le rôle exécutant ; il ne désactive pas
--      la sécurité au niveau ligne. Et `FORCE ROW LEVEL SECURITY` — posé par `0015` précisément
--      pour que le propriétaire ne soit pas hors politique — s'applique donc aussi à
--      `kaya_owner`. La fonction s'exécute, ne rend **aucune ligne**, et ne lève **aucune
--      erreur** : la connexion échoue sur « identifiants invalides » pour tout le monde, y
--      compris avec le bon mot de passe. Consigné ici parce que le symptôme ne désigne pas sa
--      cause.
--
--   4. **Celle-ci** : un rôle dédié, sans connexion, propriétaire de la fonction, et une
--      politique qui ne lui ouvre **qu'une table, qu'en lecture**. Le trou existe — il est
--      nécessaire — et il a exactement la forme du besoin.
--
-- ═════════════════════════════════════════════════════════════════════════════════════════════
--  LE PÉRIMÈTRE DE LA DÉROGATION, ÉNONCÉ
-- ═════════════════════════════════════════════════════════════════════════════════════════════
--
-- | Élément | Portée |
-- |---|---|
-- | Rôle `kaya_auth` | `NOLOGIN` — **personne ne s'y connecte**, il n'existe que pour posséder la fonction |
-- | Privilèges | `SELECT` sur `comptes.compte`, et **rien d'autre**, sur aucune autre table |
-- | Politique `resolution_identifiant` | `FOR SELECT TO kaya_auth`, sur `comptes.compte` seule |
-- | Fonction | Un identifiant en entrée, **six colonnes** en sortie, une ligne au plus |
--
-- Ce que la fonction **ne rend pas**, et chaque absence est une décision :
--
-- | Absent | Raison |
-- |---|---|
-- | Nom, prénoms, identité civile | La connexion n'affiche rien avant d'avoir réussi. Rendre un nom laisserait énumérer les identités par tentatives |
-- | Rôles, permissions | Ils se lisent **après**, sous le tenant posé, par le chemin ordinaire |
-- | Une liste, un `LIKE`, un préfixe | **Aucune recherche.** Égalité stricte, une ligne au plus. Un `LIKE` en ferait un annuaire d'identifiants |
--
-- **`actif` est rendu, et c'est nécessaire** : un compte désactivé doit être refusé, mais d'un
-- refus **indiscernable** de celui d'un identifiant inconnu (FR-012). C'est le service qui tient
-- l'indiscernabilité ; la base lui donne de quoi décider.
--
-- ═════════════════════════════════════════════════════════════════════════════════════════════
--  LES TROIS PRÉCAUTIONS D'UNE FONCTION `SECURITY DEFINER`
-- ═════════════════════════════════════════════════════════════════════════════════════════════
--
--   1. **`SET search_path`** — sans lui, l'appelant crée un schéma temporaire portant sa propre
--      table `compte`, le place en tête de son `search_path`, et la fonction lit **sa** table avec
--      les privilèges du propriétaire. C'est l'attaque classique sur `SECURITY DEFINER`, et elle
--      est réelle : `kaya_app` a le droit de créer des objets temporaires.
--   2. **`REVOKE ... FROM PUBLIC`** avant tout `GRANT` — PostgreSQL accorde `EXECUTE` à `PUBLIC`
--      par défaut. Sans ce `REVOKE`, `kaya_ledger_reader` l'appellerait, lui qui n'a par ailleurs
--      que `SELECT` sur le grand livre.
--   3. **`STABLE`, jamais `VOLATILE`** — elle ne modifie rien. Le déclarer est aussi une
--      affirmation lisible : une fonction d'authentification qui écrirait serait un mécanisme de
--      journalisation caché.

-- =============================================================================================
--  1. Le rôle porteur de la dérogation
-- =============================================================================================
--
-- `NOLOGIN` : aucune chaîne de connexion ne le nomme, aucun mot de passe ne lui est attribué, et
-- `backend/tests/isolation_tenant.rs` peut donc affirmer qu'il n'est joignable par personne.
-- `NOBYPASSRLS` et `NOSUPERUSER` sont écrits explicitement, comme dans `0001` : ce sont les deux
-- attributs qui transformeraient une dérogation d'une table en dérogation générale.
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'kaya_auth') THEN
        CREATE ROLE kaya_auth NOLOGIN NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE;
    END IF;
END
$$;

-- **PostgreSQL 16 a changé ce qu'un `CREATEROLE` obtient sur les rôles qu'il crée**, et le
-- symptôme est déroutant : `ALTER FUNCTION … OWNER TO kaya_auth` échoue sur
-- « must be able to SET ROLE "kaya_auth" » alors que `kaya_owner` vient de créer ce rôle.
--
-- Depuis PG 16, la création accorde au créateur `ADMIN OPTION` mais **`SET FALSE`** : il peut
-- administrer le rôle, pas l'endosser — et changer le propriétaire d'un objet exige de pouvoir
-- l'endosser. Le `GRANT` ci-dessous ajoute donc `SET`, en s'appuyant sur l'`ADMIN OPTION` acquis.
--
-- **`INHERIT FALSE` est le point qui compte.** Sans lui, `kaya_owner` hériterait
-- automatiquement des privilèges de `kaya_auth` — donc de la politique qui ouvre `comptes.compte`
-- en lecture tous tenants — et la dérogation cesserait d'être limitée à la fonction. Avec
-- `INHERIT FALSE`, il faut un `SET ROLE` explicite : un acte délibéré, visible, que personne
-- n'écrit par distraction.
GRANT kaya_auth TO kaya_owner WITH SET TRUE, INHERIT FALSE;

COMMENT ON ROLE kaya_auth IS
    'CPT-01 — porte la SEULE dérogation à l''isolation par tenant. NOLOGIN : n''existe que pour posséder comptes.resoudre_identifiant. SELECT sur comptes.compte, rien d''autre.';

GRANT USAGE ON SCHEMA comptes TO kaya_auth;
GRANT SELECT ON comptes.compte TO kaya_auth;

-- La politique qui ouvre la table à ce rôle — **et à lui seul**.
--
-- Elle vient s'ajouter à `isolation_tenant`, elle ne la remplace pas : les politiques permissives
-- s'additionnent par `OR`, et `isolation_tenant` continue de s'appliquer intégralement à
-- `kaya_app` comme à `kaya_owner`, qui ne sont pas `kaya_auth`.
CREATE POLICY resolution_identifiant ON comptes.compte
    FOR SELECT TO kaya_auth
    USING (true);

-- =============================================================================================
--  2. La fonction
-- =============================================================================================

CREATE FUNCTION comptes.resoudre_identifiant(p_identifiant TEXT)
RETURNS TABLE (
    compte_id              UUID,
    tenant_id              UUID,
    condensat_mot_de_passe TEXT,
    methode_code           TEXT,
    actif                  BOOLEAN,
    personne_id            UUID
)
LANGUAGE sql
STABLE
SECURITY DEFINER
SET search_path = pg_catalog, comptes
AS $$
    -- Égalité stricte sur l'une ou l'autre colonne d'identifiant. **Aucun `LIKE`, aucun
    -- `lower()`** : les colonnes portent déjà leur forme normalisée à l'écriture, et une seconde
    -- normalisation ici la dédoublerait — donc la ferait diverger le jour où l'une changerait.
    --
    -- `LIMIT 1` n'est pas une précaution contre des doublons accidentels : les contraintes
    -- d'unicité sont **par tenant**, donc un même numéro peut légitimement exister chez deux
    -- clients. C'est une décision fonctionnelle, écrite ici : le premier compte par ordre stable
    -- l'emporte. Le cas est rare — un employé travaillant pour deux clients de Kaya avec le même
    -- numéro — et sa vraie réponse est ETB-06, le sélecteur d'établissement.
    SELECT c.id, c.tenant_id, c.condensat_mot_de_passe, c.methode_code, c.actif, c.personne_id
      FROM comptes.compte c
     WHERE c.identifiant_telephone = p_identifiant
        OR c.identifiant_email     = p_identifiant
     ORDER BY c.cree_le, c.id
     LIMIT 1;
$$;


COMMENT ON FUNCTION comptes.resoudre_identifiant(TEXT) IS
    'CPT-01 — LA SEULE DÉROGATION À L''ISOLATION PAR TENANT DU PRODUIT. Résout un identifiant de connexion avant que le tenant soit connu. Détenue par kaya_auth (NOLOGIN, SELECT sur comptes.compte seulement). Périmètre = signature. Voir 0020 pour les trois solutions écartées.';

-- Précaution 2 : `PUBLIC` reçoit `EXECUTE` par défaut. On le retire avant d'accorder.
REVOKE ALL ON FUNCTION comptes.resoudre_identifiant(TEXT) FROM PUBLIC;

-- Seul le rôle applicatif l'appelle. Ni `kaya_worker`, ni `kaya_ledger_reader` n'authentifient
-- qui que ce soit.
GRANT EXECUTE ON FUNCTION comptes.resoudre_identifiant(TEXT) TO kaya_app;


-- =============================================================================================
--  3. Le changement de propriétaire — EN DERNIER, et l'ordre n'est pas indifférent
-- =============================================================================================
--
-- **C'est ce transfert qui fait tout.** Sans lui la fonction s'exécute sous `kaya_owner`, à qui
-- `FORCE ROW LEVEL SECURITY` s'applique, et elle ne rend jamais rien — en silence.
--
-- Il vient **après** le commentaire et les privilèges parce qu'ils exigent tous d'être posés par
-- le propriétaire : une fois la fonction passée à `kaya_auth`, `kaya_owner` ne peut plus la
-- commenter ni en modifier les droits — `INHERIT FALSE` lui interdit d'hériter de quoi que ce
-- soit de `kaya_auth`. L'ordre inverse échoue sur « must be owner of function », trois
-- instructions plus loin, sans que le message désigne la cause.
--
-- Changer le propriétaire d'un objet exige en outre que le nouveau propriétaire ait `CREATE` sur
-- le schéma. Le privilège est donc accordé, employé, **puis retiré** : `kaya_auth` ne conserve
-- aucun droit d'écriture sur `comptes` au-delà de cette transaction, et la propriété de la
-- fonction survit à ce retrait. Un `GRANT CREATE` laissé en place serait un droit permanent pour
-- une nécessité d'un instant.
GRANT CREATE ON SCHEMA comptes TO kaya_auth;
ALTER FUNCTION comptes.resoudre_identifiant(TEXT) OWNER TO kaya_auth;
REVOKE CREATE ON SCHEMA comptes FROM kaya_auth;

-- Enfin, `EXECUTE` pour `kaya_owner` — **et il ne peut se donner qu'ici, après le transfert**.
--
-- La raison est un détail d'ACL qui coûte une demi-heure à retrouver : un `GRANT` au propriétaire
-- se fond dans l'entrée d'ACL du propriétaire, et **le changement de propriétaire la remplace**.
-- Accordé avant le transfert, ce droit disparaît sans laisser de trace ; accordé après, il faut
-- endosser `kaya_auth`, seul détenteur du droit d'accorder.
--
-- Pourquoi ce droit existe : **les macros `query!` de sqlx préparent chaque requête contre la
-- vraie base**, sous `DATABASE_URL`, donc sous `kaya_owner` (porte P-18). Sans lui, la
-- vérification à la compilation échoue sur « permission denied for function » — dans un fichier
-- Rust, pour une cause qui est ici.
--
-- Il n'élargit rien : `kaya_owner` détient déjà `SET ROLE kaya_auth`, donc tout ce que la
-- fonction peut faire lui était déjà accessible. Ce qui reste vrai, et qui est le sujet, c'est
-- qu'il n'a **aucun autre moyen de lire `comptes.compte` tous tenants**.
SET LOCAL ROLE kaya_auth;
GRANT EXECUTE ON FUNCTION comptes.resoudre_identifiant(TEXT) TO kaya_owner;
RESET ROLE;
