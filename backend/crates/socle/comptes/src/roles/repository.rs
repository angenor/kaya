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
/// L'ordre est **stable** (`cree_le, id`), et c'est ce qui rend « le premier accessible » une
/// règle et non un hasard : deux connexions successives sans `etablissement_id` doivent ouvrir le
/// même établissement, sans quoi l'utilisateur verrait son accueil changer d'une fois sur l'autre.
pub async fn etablissements_accessibles(
    tx: &mut sqlx::PgTransaction<'_>,
    compte_id: Uuid,
) -> Result<Vec<Uuid>, ErreurRoles> {
    let ids = sqlx::query_scalar!(
        r#"
        SELECT e.id AS "id!"
        FROM etablissements.etablissement e
        WHERE e.id IN (
            SELECT cr.etablissement_id
            FROM comptes.compte_role cr
            WHERE cr.compte_id = $1 AND cr.etablissement_id IS NOT NULL
        )
        ORDER BY e.cree_le, e.id
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
