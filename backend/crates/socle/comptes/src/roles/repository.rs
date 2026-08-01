//! Lecture des droits — **l'union, calculée par la base**.
//!
//! # Pourquoi l'union se fait en SQL et non en Rust
//!
//! Un `SELECT DISTINCT` sur la jointure `compte_role → role_permission` **est** l'union, et il
//! n'existe aucun endroit où écrire une priorité. Ramener les rôles puis fusionner leurs
//! permissions côté applicatif produirait le même résultat aujourd'hui, et offrirait une ligne où
//! quelqu'un écrirait un jour `if role == "gerant" { … }` — la hiérarchie que le principe VII
//! interdit et que FR-017 nomme.
//!
//! # La portée, et le piège du `NULL`
//!
//! `compte_role.etablissement_id` vaut `NULL` pour `admin_editeur`. Une comparaison
//! `etablissement_id = $2` écarterait donc silencieusement les droits d'éditeur — en SQL, `NULL =
//! NULL` vaut `NULL`, jamais `true`. La condition est écrite pour retenir **les rôles de
//! l'établissement demandé ET les rôles sans établissement**, ce qui est la définition d'un rôle
//! d'éditeur : il vaut partout.

use std::collections::BTreeSet;

use uuid::Uuid;

use kaya_etablissements::Issue;

use super::modele::{AttribuerRole, ErreurRoles, PorteeRole};

/// La permission qui **habilite** à gérer les rôles d'un établissement.
///
/// Nommée ici plutôt qu'en littéral dans la requête : c'est elle que FR-023 protège, et une
/// chaîne recopiée à deux endroits finirait par en désigner deux.
pub const PERMISSION_HABILITANTE: &str = "cpt.role.attribuer";

/// Les permissions effectives d'un compte sur un établissement — **l'union de ses rôles**.
///
/// `BTreeSet` : le type dit l'unicité et l'ordre stable. Une tuile issue de trois rôles
/// n'apparaît qu'une fois (FR-027) sans que l'appelant ait à dédoublonner.
///
/// Un compte **sans aucun rôle** rend un ensemble **vide**, jamais une erreur : il se connecte,
/// et son accueil est vide. Une erreur ici rendrait la connexion impossible pour un compte
/// fraîchement créé, avant que qui que ce soit ait eu le temps de lui donner un rôle.
pub async fn permissions_effectives(
    tx: &mut sqlx::PgTransaction<'_>,
    compte_id: Uuid,
    etablissement_id: Option<Uuid>,
) -> Result<BTreeSet<String>, ErreurRoles> {
    let codes = sqlx::query_scalar!(
        r#"
        SELECT DISTINCT rp.permission_code AS "code!"
        FROM comptes.compte_role cr
        JOIN comptes.role_permission rp ON rp.role_code = cr.role_code
        WHERE cr.compte_id = $1
          AND (cr.etablissement_id IS NULL OR cr.etablissement_id = $2)
        ORDER BY 1
        "#,
        compte_id,
        etablissement_id,
    )
    .fetch_all(&mut **tx)
    .await?;

    Ok(codes.into_iter().collect())
}

/// Les établissements sur lesquels un compte porte au moins un rôle.
///
/// Sert à deux choses : choisir l'établissement actif quand la connexion n'en désigne aucun, et
/// remplir `etablissements[]` de la réponse de connexion. Le **sélecteur permanent** est ETB-06,
/// hors périmètre.
///
/// L'ordre est **stable**, et c'est ce qui rend « le premier accessible » une règle et non un
/// hasard : deux connexions successives sans `etablissement_id` doivent ouvrir le même
/// établissement, sans quoi l'utilisateur verrait son accueil changer d'une fois sur l'autre.
///
/// # L'ordre vient de `compte_role`, et jamais de `etablissements.etablissement`
///
/// La forme évidente — joindre la table des établissements pour trier sur **sa** date de création —
/// **joint deux schémas de modules**, ce que le principe II interdit et que la porte **P-04**
/// refuse. Elle a été écrite, et la porte l'a attrapée.
///
/// L'ordre retenu est donc celui de **l'attribution du rôle** : le premier établissement sur
/// lequel le compte a reçu un rôle est le premier proposé. C'est au moins aussi défendable — c'est
/// l'établissement de rattachement d'origine, pas le plus ancien du groupe — et cela ne suppose
/// rien de l'autre module.
///
/// **Aucune vérification d'existence ici.** Un `etablissement_id` de `compte_role` peut en théorie
/// désigner un établissement supprimé : rien ne se supprime dans Kaya (FR-014), et la vérification
/// à l'attribution passe par `EstablishmentDirectory` (T039). La faire ici la ferait passer par
/// une jointure, c'est-à-dire par la faute qu'on vient d'écarter.
pub async fn etablissements_accessibles(
    tx: &mut sqlx::PgTransaction<'_>,
    compte_id: Uuid,
) -> Result<Vec<Uuid>, ErreurRoles> {
    let ids = sqlx::query_scalar!(
        r#"
        SELECT DISTINCT ON (cr.etablissement_id) cr.etablissement_id AS "id!"
        FROM comptes.compte_role cr
        WHERE cr.compte_id = $1 AND cr.etablissement_id IS NOT NULL
        ORDER BY cr.etablissement_id, cr.cree_le
        "#,
        compte_id
    )
    .fetch_all(&mut **tx)
    .await?;

    Ok(ids)
}

/// Le compte porte-t-il au moins un rôle de portée `EDITEUR` ?
///
/// Un `admin_editeur` n'est rattaché à aucun établissement : sans cette distinction, la connexion
/// conclurait qu'il n'a accès à rien et lui rendrait un accueil vide.
pub async fn porte_un_role_editeur(
    tx: &mut sqlx::PgTransaction<'_>,
    compte_id: Uuid,
) -> Result<bool, ErreurRoles> {
    let porte = sqlx::query_scalar!(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM comptes.compte_role cr
            JOIN comptes.role r ON r.code = cr.role_code
            WHERE cr.compte_id = $1 AND r.portee = 'EDITEUR'
        ) AS "porte!"
        "#,
        compte_id
    )
    .fetch_one(&mut **tx)
    .await?;

    Ok(porte)
}

// =================================================================================================
//  Écritures — l'attribution et le retrait
// =================================================================================================

/// La portée déclarée d'un rôle, ou `None` si le code est inconnu.
///
/// Lue du référentiel plutôt que codée en dur : `role.portee` est une colonne, et un `match` sur
/// huit codes en Rust en ferait une seconde source qui dériverait au neuvième rôle.
pub async fn portee_du_role(
    tx: &mut sqlx::PgTransaction<'_>,
    role_code: &str,
) -> Result<Option<PorteeRole>, ErreurRoles> {
    let portee = sqlx::query_scalar!(
        r#"SELECT portee AS "portee!" FROM comptes.role WHERE code = $1"#,
        role_code
    )
    .fetch_optional(&mut **tx)
    .await?;

    // Un code présent en base dont la portée ne se décode pas est une incohérence de migration,
    // pas une entrée inconnue : `depuis_code` rendrait `None` et l'appelant lirait « rôle
    // inconnu », ce qui serait faux. La contrainte `role_portee_connue` ferme ce cas en base.
    Ok(portee.and_then(|p| PorteeRole::depuis_code(&p)))
}

/// Le compte existe-t-il **dans le tenant courant** ?
///
/// La politique de sécurité au niveau ligne fait le travail : hors du tenant posé, la ligne
/// n'existe pas. Il n'y a donc rien à comparer ici, et c'est voulu — une comparaison explicite de
/// `tenant_id` en Rust serait une seconde barrière qui masquerait l'absence de la première.
pub async fn compte_existe(
    tx: &mut sqlx::PgTransaction<'_>,
    compte_id: Uuid,
) -> Result<bool, ErreurRoles> {
    let existe = sqlx::query_scalar!(
        r#"SELECT EXISTS (SELECT 1 FROM comptes.compte WHERE id = $1) AS "existe!""#,
        compte_id
    )
    .fetch_one(&mut **tx)
    .await?;

    Ok(existe)
}

/// Attribue un rôle. **Idempotent** : un rejeu rend [`Issue::DejaPresente`].
///
/// L'unicité vient de `compte_role_unique (compte_id, role_code, etablissement_id)`, en
/// `NULLS NOT DISTINCT` — sans cette clause, `(compte, admin_editeur, NULL)` s'insérerait autant
/// de fois qu'on veut et le retrait n'en enlèverait qu'une occurrence sur N.
///
/// # Le conflit porte sur le TRIPLET, pas sur l'identifiant client
///
/// `ON CONFLICT (id)` serait la forme évidente, et elle serait fausse : deux terminaux qui
/// attribuent le même rôle génèrent deux UUID v7 différents, et la seconde insertion violerait
/// `compte_role_unique` au lieu d'être absorbée. Le conflit se déclare donc sur la contrainte
/// métier, et l'identifiant client ne sert qu'à nommer la ligne qui gagne.
pub async fn attribuer(
    tx: &mut sqlx::PgTransaction<'_>,
    tenant_id: Uuid,
    demande: &AttribuerRole,
    attribue_par_compte_id: Uuid,
) -> Result<Issue, ErreurRoles> {
    let insere = sqlx::query_scalar!(
        r#"
        INSERT INTO comptes.compte_role
            (id, tenant_id, compte_id, role_code, etablissement_id,
             attribue_par_compte_id, horodatage_client)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT ON CONSTRAINT compte_role_unique DO NOTHING
        RETURNING id
        "#,
        demande.id,
        tenant_id,
        demande.compte_id,
        demande.role_code,
        demande.etablissement_id,
        attribue_par_compte_id,
        demande.horodatage_client,
    )
    .fetch_optional(&mut **tx)
    .await?;

    Ok(if insere.is_some() {
        Issue::Creee
    } else {
        Issue::DejaPresente
    })
}

/// Retire un rôle. Rend `false` si le compte ne le portait pas.
///
/// `DELETE` et non `UPDATE` : `compte_role` n'a **pas** de privilège `UPDATE` (migration `0016`).
/// Changer un rôle, c'est en retirer un et en attribuer un autre — deux actes, deux entrées
/// d'audit. Une colonne `actif` aurait laissé un chemin où un seul acte en cache deux.
pub async fn retirer(
    tx: &mut sqlx::PgTransaction<'_>,
    compte_id: Uuid,
    role_code: &str,
    etablissement_id: Option<Uuid>,
) -> Result<bool, ErreurRoles> {
    // `IS NOT DISTINCT FROM` et non `=` : `etablissement_id` vaut `NULL` pour `admin_editeur`, et
    // `NULL = NULL` vaut `NULL` en SQL — jamais `true`. Un `=` ne retirerait donc **jamais** un
    // rôle d'éditeur, silencieusement.
    let retirees = sqlx::query!(
        r#"
        DELETE FROM comptes.compte_role
        WHERE compte_id = $1
          AND role_code = $2
          AND etablissement_id IS NOT DISTINCT FROM $3
        "#,
        compte_id,
        role_code,
        etablissement_id,
    )
    .execute(&mut **tx)
    .await?;

    Ok(retirees.rows_affected() > 0)
}

/// Combien de comptes **actifs** restent habilités à gérer les rôles d'un établissement ?
///
/// C'est la mesure de FR-023. Trois précisions qui décident du résultat :
///
///  * **`actif = true`** — un compte désactivé ne peut plus se connecter, donc il n'habilite
///    personne. Le compter laisserait retirer la dernière habilitation vivante.
///  * **Les rôles d'éditeur ne comptent pas** : `admin_editeur` n'est pas rattaché à
///    l'établissement, et son existence rendrait la garde inopérante partout.
///  * **La question porte sur la permission, pas sur le rôle.** Demander « combien de gérants »
///    serait faux le jour où un neuvième rôle porterait `cpt.role.attribuer`.
pub async fn comptes_habilites(
    tx: &mut sqlx::PgTransaction<'_>,
    etablissement_id: Uuid,
) -> Result<i64, ErreurRoles> {
    let nombre = sqlx::query_scalar!(
        r#"
        SELECT COUNT(DISTINCT cr.compte_id) AS "nombre!"
        FROM comptes.compte_role cr
        JOIN comptes.role_permission rp ON rp.role_code = cr.role_code
        JOIN comptes.compte c ON c.id = cr.compte_id
        WHERE cr.etablissement_id = $1
          AND rp.permission_code = $2
          AND c.actif
        "#,
        etablissement_id,
        PERMISSION_HABILITANTE,
    )
    .fetch_one(&mut **tx)
    .await?;

    Ok(nombre)
}

/// Ce rôle, sur cet établissement, porte-t-il la permission habilitante ?
///
/// Sert à ne poser la question coûteuse de [`comptes_habilites`] que lorsqu'elle a un sens :
/// retirer `serveur` ne peut pas priver un établissement de son habilitation.
pub async fn role_habilite(
    tx: &mut sqlx::PgTransaction<'_>,
    role_code: &str,
) -> Result<bool, ErreurRoles> {
    let habilite = sqlx::query_scalar!(
        r#"
        SELECT EXISTS (
            SELECT 1 FROM comptes.role_permission
            WHERE role_code = $1 AND permission_code = $2
        ) AS "habilite!"
        "#,
        role_code,
        PERMISSION_HABILITANTE,
    )
    .fetch_one(&mut **tx)
    .await?;

    Ok(habilite)
}

/// Le compte porte-t-il **encore** la permission habilitante sur cet établissement, une fois le
/// rôle retiré ?
///
/// Le cumul rend la question nécessaire : Adjoua est gérante **et** propriétaire, retirer
/// `gerant` ne lui retire pas `cpt.role.attribuer`. Compter les comptes habilités **après** le
/// `DELETE`, dans la même transaction, répond aux deux cas d'un coup — c'est ce que fait le
/// service, et cette fonction n'existe que pour le cas où l'on voudrait répondre avant.
pub async fn compte_reste_habilite(
    tx: &mut sqlx::PgTransaction<'_>,
    compte_id: Uuid,
    etablissement_id: Uuid,
) -> Result<bool, ErreurRoles> {
    let habilite = sqlx::query_scalar!(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM comptes.compte_role cr
            JOIN comptes.role_permission rp ON rp.role_code = cr.role_code
            WHERE cr.compte_id = $1
              AND cr.etablissement_id = $2
              AND rp.permission_code = $3
        ) AS "habilite!"
        "#,
        compte_id,
        etablissement_id,
        PERMISSION_HABILITANTE,
    )
    .fetch_one(&mut **tx)
    .await?;

    Ok(habilite)
}

// **Aucune fonction `roles_du_compte` ici**, et c'est délibéré : `compte::repository::roles_portes`
// la fait déjà, **en lot**, pour l'écran `G3`. En écrire une seconde version par compte
// produirait deux définitions de « les rôles d'un compte » — celle qui dérive étant toujours
// celle qu'on ne relit pas — et rouvrirait le problème des cent requêtes que le lot ferme.

/// Le référentiel des huit rôles — **le même pour les deux tenants**.
pub async fn referentiel_roles(
    tx: &mut sqlx::PgTransaction<'_>,
) -> Result<Vec<super::modele::EntreeReferentielRole>, ErreurRoles> {
    let lignes = sqlx::query!(
        r#"
        SELECT code AS "code!", libelle_cle AS "libelle_cle!",
               ordre AS "ordre!", portee AS "portee!"
        FROM comptes.role
        ORDER BY ordre
        "#
    )
    .fetch_all(&mut **tx)
    .await?;

    Ok(lignes
        .into_iter()
        .map(|l| super::modele::EntreeReferentielRole {
            code: l.code,
            libelle_cle: l.libelle_cle,
            ordre: l.ordre,
            portee: Some(l.portee),
            module_code: None,
        })
        .collect())
}

/// Le référentiel des dix-sept permissions — **le même pour les deux tenants**.
pub async fn referentiel_permissions(
    tx: &mut sqlx::PgTransaction<'_>,
) -> Result<Vec<super::modele::EntreeReferentielRole>, ErreurRoles> {
    let lignes = sqlx::query!(
        r#"
        SELECT code AS "code!", libelle_cle AS "libelle_cle!",
               ordre AS "ordre!", module_code
        FROM comptes.permission
        ORDER BY ordre
        "#
    )
    .fetch_all(&mut **tx)
    .await?;

    Ok(lignes
        .into_iter()
        .map(|l| super::modele::EntreeReferentielRole {
            code: l.code,
            libelle_cle: l.libelle_cle,
            ordre: l.ordre,
            portee: None,
            module_code: l.module_code,
        })
        .collect())
}
