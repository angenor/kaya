//! Accès aux données de l'identité civile — **macros littérales, transaction en paramètre**.
//!
//! Les trois règles du module doré, couche 3, sans aménagement :
//!
//!   * toutes les requêtes passent par `query!` / `query_as!` sur **littéral**, donc vérifiées à
//!     la compilation contre la vraie base (porte P-18) ; `AssertSqlSafe` n'apparaît nulle part ;
//!   * le repository **prend** la transaction, il ne l'ouvre pas — c'est le service qui décide de
//!     la portée, parce que c'est lui qui doit y inclure l'événement outbox ;
//!   * aucune jointure entre schémas de modules (porte P-04).
//!
//! **`type_piece` et `numero_piece` n'apparaissent dans aucune requête de ce fichier** — ni en
//! écriture, ni en lecture. `backend/tests/provisions_sans_logique.rs` le vérifie.

use uuid::Uuid;

use super::modele::{CreerPersonne, ErreurPersonne, ModifierPersonne, Personne};
use kaya_etablissements::Issue;

/// Insère une personne, ou constate qu'elle existe déjà.
///
/// `ON CONFLICT (id) DO NOTHING ... RETURNING` renvoie une ligne quand l'insertion a eu lieu, et
/// **rien** en cas de conflit : c'est exactement ce qu'il faut pour distinguer `201` de `200`,
/// sans second aller-retour dans le cas normal.
pub async fn inserer(
    tx: &mut sqlx::PgTransaction<'_>,
    tenant_id: Uuid,
    personne: &CreerPersonne,
) -> Result<(Personne, Issue), ErreurPersonne> {
    let insere = sqlx::query_as!(
        Personne,
        r#"
        INSERT INTO comptes.personne
            (id, tenant_id, nom, prenoms, telephone, email, horodatage_client)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (id) DO NOTHING
        RETURNING id, nom, prenoms, telephone, email, horodatage_client, cree_le, modifie_le
        "#,
        personne.id,
        tenant_id,
        personne.nom,
        personne.prenoms,
        personne.telephone,
        personne.email,
        personne.horodatage_client,
    )
    .fetch_optional(&mut **tx)
    .await?;

    match insere {
        Some(personne) => Ok((personne, Issue::Creee)),
        None => {
            // La ligne ne peut manquer que si un **autre tenant** détient cet identifiant : la
            // politique de sécurité la masque alors, et `ON CONFLICT` l'a pourtant vue. Assez
            // improbable pour ne pas mériter un type d'erreur propre, assez grave pour ne pas
            // être traité comme un succès.
            let existante = lire(tx, personne.id).await?.ok_or(ErreurPersonne::Inconnue)?;
            Ok((existante, Issue::DejaPresente))
        }
    }
}

/// Lit une personne du tenant courant.
pub async fn lire(
    tx: &mut sqlx::PgTransaction<'_>,
    id: Uuid,
) -> Result<Option<Personne>, ErreurPersonne> {
    let personne = sqlx::query_as!(
        Personne,
        r#"
        SELECT id, nom, prenoms, telephone, email, horodatage_client, cree_le, modifie_le
        FROM comptes.personne
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(&mut **tx)
    .await?;

    Ok(personne)
}

/// Remplace les champs modifiables d'une personne.
///
/// `modifie_le` est posé par `now()` **en SQL**, jamais par l'horloge du processus : deux
/// instances d'API n'ont pas la même horloge, la base en a une seule (principe IV). `cree_le`
/// n'est pas touchée — c'est elle qui fait autorité sur la date de création.
///
/// Rend `None` quand aucune ligne n'a été touchée : personne inconnue **ou** appartenant à un
/// autre tenant, deux cas que la politique de sécurité rend volontairement indistinguables.
pub async fn modifier(
    tx: &mut sqlx::PgTransaction<'_>,
    id: Uuid,
    modification: &ModifierPersonne,
) -> Result<Option<Personne>, ErreurPersonne> {
    let personne = sqlx::query_as!(
        Personne,
        r#"
        UPDATE comptes.personne
        SET nom               = $2,
            prenoms           = $3,
            telephone         = $4,
            email             = $5,
            horodatage_client = $6,
            modifie_le        = now()
        WHERE id = $1
        RETURNING id, nom, prenoms, telephone, email, horodatage_client, cree_le, modifie_le
        "#,
        id,
        modification.nom,
        modification.prenoms,
        modification.telephone,
        modification.email,
        modification.horodatage_client,
    )
    .fetch_optional(&mut **tx)
    .await?;

    Ok(personne)
}
