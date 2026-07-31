//! Couche d'accès aux données — **écrite à la main contre sqlx 0.9.0**.
//!
//! # Pourquoi « à la main » est une consigne et pas une posture
//!
//! Le cadrage §13.1 exige que ce module soit écrit avant toute génération assistée. La raison est
//! précise : sqlx `0.9.0` a rompu avec `0.8.x` sur deux points (`#3723` impose `AssertSqlSafe`
//! sur toute requête non littérale, `#3541` modifie la sortie des macros `query!()`), et **la
//! totalité de la documentation, des exemples et des réponses en ligne vise encore `0.8`**. Tout
//! extrait repris ailleurs ne compilera pas — ou pire, compilera en réintroduisant une tournure
//! que le cycle suivant recopiera.
//!
//! Ce fichier est donc le patron. Les points qu'il fixe :
//!
//! - toutes les requêtes passent par les **macros `query!` / `query_as!` sur littéral**, donc
//!   vérifiées à la compilation (porte P-18) ; `AssertSqlSafe` n'apparaît nulle part ;
//! - le repository **prend la transaction**, il ne l'ouvre pas — c'est le service qui décide de
//!   la portée transactionnelle, parce que c'est lui qui doit y inclure l'événement outbox ;
//! - aucune jointure entre schémas de modules (porte P-04).

use time::OffsetDateTime;
use uuid::Uuid;

use super::modele::{CreerNote, ErreurNote, Issue, NoteEtablissement};

/// Insère une note, ou constate qu'elle existe déjà.
///
/// `ON CONFLICT (id) DO NOTHING ... RETURNING` renvoie une ligne quand l'insertion a eu lieu, et
/// **rien** quand la clé existait. C'est exactement l'information dont le contrat HTTP a besoin
/// pour distinguer `201` de `200`, et elle s'obtient sans second aller-retour dans le cas normal.
///
/// Le rejeu ne relit la ligne existante que dans le cas de conflit — celui du terminal qui vide
/// sa file après une coupure. **Le serveur fait foi en conflit** (principe VI) : le corps renvoyé
/// est la note telle qu'elle est en base, pas celle que le client vient de proposer.
pub async fn inserer(
    tx: &mut sqlx::PgTransaction<'_>,
    tenant_id: Uuid,
    note: &CreerNote,
) -> Result<(NoteEtablissement, Issue), ErreurNote> {
    let insere = sqlx::query_as!(
        NoteEtablissement,
        r#"
        INSERT INTO etablissements.note_etablissement
            (id, tenant_id, etablissement_id, auteur_compte_id, texte, horodatage_client)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (id) DO NOTHING
        RETURNING id,
                  etablissement_id,
                  auteur_compte_id,
                  texte,
                  horodatage_client,
                  cree_le
        "#,
        note.id,
        tenant_id,
        note.etablissement_id,
        note.auteur_compte_id,
        note.texte,
        note.horodatage_client,
    )
    .fetch_optional(&mut **tx)
    .await?;

    match insere {
        Some(note) => Ok((note, Issue::Creee)),
        None => {
            let existante = lire(tx, note.id)
                .await?
                // La ligne ne peut manquer que si un autre tenant détient cet identifiant : la
                // politique de sécurité la masque alors, et `ON CONFLICT` l'a pourtant vue. Le
                // cas est assez improbable pour ne pas mériter un type d'erreur propre, et assez
                // grave pour ne pas être traité comme un succès.
                .ok_or(ErreurNote::EtablissementInconnu)?;
            Ok((existante, Issue::DejaPresente))
        }
    }
}

/// Lit une note par son identifiant, dans le tenant courant.
pub async fn lire(
    tx: &mut sqlx::PgTransaction<'_>,
    id: Uuid,
) -> Result<Option<NoteEtablissement>, ErreurNote> {
    let note = sqlx::query_as!(
        NoteEtablissement,
        r#"
        SELECT id, etablissement_id, auteur_compte_id, texte, horodatage_client, cree_le
        FROM etablissements.note_etablissement
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(&mut **tx)
    .await?;

    Ok(note)
}

/// Liste les notes d'un établissement, de la plus récente à la plus ancienne.
///
/// Le tri se fait sur `cree_le` — l'horodatage d'**autorité serveur** — jamais sur
/// `horodatage_client`. Trier sur l'horloge d'un terminal ferait remonter en tête la note d'un
/// appareil mal réglé.
///
/// L'ordre secondaire sur `id` n'est pas décoratif : deux notes créées dans la même transaction
/// partagent `now()`, et sans départage la pagination sauterait ou répéterait des lignes. L'UUID
/// v7 étant lui-même ordonné dans le temps, il départage dans le bon sens.
pub async fn lister(
    tx: &mut sqlx::PgTransaction<'_>,
    etablissement_id: Uuid,
    limite: i64,
    decalage: i64,
) -> Result<Vec<NoteEtablissement>, ErreurNote> {
    let notes = sqlx::query_as!(
        NoteEtablissement,
        r#"
        SELECT id, etablissement_id, auteur_compte_id, texte, horodatage_client, cree_le
        FROM etablissements.note_etablissement
        WHERE etablissement_id = $1
        ORDER BY cree_le DESC, id DESC
        LIMIT $2 OFFSET $3
        "#,
        etablissement_id,
        limite,
        decalage,
    )
    .fetch_all(&mut **tx)
    .await?;

    Ok(notes)
}

/// Compte les notes d'un établissement — pour la pagination.
pub async fn compter(
    tx: &mut sqlx::PgTransaction<'_>,
    etablissement_id: Uuid,
) -> Result<i64, ErreurNote> {
    let total = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) AS "compte!"
        FROM etablissements.note_etablissement
        WHERE etablissement_id = $1
        "#,
        etablissement_id
    )
    .fetch_one(&mut **tx)
    .await?;

    Ok(total)
}

/// Vérifie qu'un établissement existe **dans le tenant courant**.
///
/// La politique de sécurité au niveau ligne suffirait à masquer un établissement d'autrui, mais
/// une clé étrangère violée produirait une erreur SQL brute là où l'appelant a besoin d'un `404`
/// intelligible.
pub async fn etablissement_existe(
    tx: &mut sqlx::PgTransaction<'_>,
    etablissement_id: Uuid,
) -> Result<bool, ErreurNote> {
    let existe = sqlx::query_scalar!(
        r#"
        SELECT EXISTS (
            SELECT 1 FROM etablissements.etablissement WHERE id = $1
        ) AS "existe!"
        "#,
        etablissement_id
    )
    .fetch_one(&mut **tx)
    .await?;

    Ok(existe)
}

/// Horodatage d'**autorité serveur**, lu depuis la base.
///
/// Exposé pour les tests et le diagnostic. L'horloge du processus applicatif n'est pas celle de
/// la base, et deux instances d'API n'ont pas la même — la base, elle, est unique.
pub async fn maintenant(tx: &mut sqlx::PgTransaction<'_>) -> Result<OffsetDateTime, ErreurNote> {
    let maintenant = sqlx::query_scalar!(r#"SELECT now() AS "maintenant!""#)
        .fetch_one(&mut **tx)
        .await?;
    Ok(maintenant)
}
