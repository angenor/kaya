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

use super::modele::ErreurRoles;

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
