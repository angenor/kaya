//! Accès aux données de l'occupation — **écrit à la main contre sqlx 0.9.0**.
//!
//! # Le mapping de type validé par ce cycle
//!
//! `PgRange<time::OffsetDateTime>` ↔ `TSTZ_RANGE` est vérifié dans les sources de
//! `sqlx-postgres` 0.9.0 (`src/types/range.rs:213`), et **exercé ici sur du code réel** — c'est le
//! retour du spike que le cycle 001 avait promis avant HEB-02.
//!
//! # Rappel sqlx 0.9 qui coûte un quart d'heure
//!
//! `query!` sur un `SELECT` produit un `Map`, qui n'a **pas** de méthode `.execute()`. Employer
//! `.fetch_one(&mut **tx)` — avec le **déréférencement double**, forme attendue pour exécuter sur
//! une transaction empruntée.

use sqlx::postgres::types::PgRange;
use time::OffsetDateTime;
use uuid::Uuid;

use super::modele::{
    DemandeAttribution, ErreurAttribution, OccupationVue, StatutOccupation, UniteDisponible,
};
use crate::referentiel::StatutMenage;

/// Insère une occupation, ou constate qu'elle existe déjà.
///
/// `ON CONFLICT (id) DO NOTHING ... RETURNING` distingue `201` de `200` sans second aller-retour.
///
/// **La violation d'exclusion n'est PAS interceptée ici** : elle remonte telle quelle au service,
/// qui la traduit. Le repository ne décide d'aucun refus métier.
pub async fn inserer(
    tx: &mut sqlx::PgTransaction<'_>,
    tenant_id: Uuid,
    demande: &DemandeAttribution,
    fin_periode: OffsetDateTime,
) -> Result<bool, sqlx::Error> {
    // La borne haute vient du serveur : `fin_client` + le battement de la catégorie.
    let periode = PgRange {
        start: std::ops::Bound::Included(demande.debut_client),
        end: std::ops::Bound::Excluded(fin_periode),
    };

    let insere = sqlx::query!(
        r#"
        INSERT INTO hebergement.occupation
            (id, tenant_id, etablissement_id, unite_id, formule_id,
             periode, debut_client, fin_client)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (id) DO NOTHING
        RETURNING id
        "#,
        demande.id,
        tenant_id,
        demande.etablissement_id,
        demande.unite_id,
        demande.formule_id,
        periode as PgRange<OffsetDateTime>,
        demande.debut_client,
        demande.fin_client,
    )
    .fetch_optional(&mut **tx)
    .await?;

    Ok(insere.is_some())
}

/// Lit une occupation dans le tenant courant.
///
/// `upper(periode)` plutôt que la colonne entière : ce que l'appelant veut est la borne
/// d'indisponibilité, et l'extraire en SQL évite de reconstruire un `Bound` côté Rust pour n'en
/// lire qu'un côté.
pub async fn lire(
    tx: &mut sqlx::PgTransaction<'_>,
    id: Uuid,
) -> Result<Option<OccupationVue>, ErreurAttribution> {
    let ligne = sqlx::query!(
        r#"
        SELECT id, unite_id, formule_id, debut_client, fin_client,
               upper(periode) AS "indisponible_jusqu_a!", statut, cree_le, libere_le
        FROM hebergement.occupation
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(&mut **tx)
    .await?;

    match ligne {
        None => Ok(None),
        Some(l) => Ok(Some(OccupationVue {
            id: l.id,
            unite_id: l.unite_id,
            formule_id: l.formule_id,
            debut_client: l.debut_client,
            fin_client: l.fin_client,
            indisponible_jusqu_a: l.indisponible_jusqu_a,
            statut: StatutOccupation::depuis_code(&l.statut)?,
            cree_le: l.cree_le,
            libere_le: l.libere_le,
        })),
    }
}

/// Les unités d'une catégorie **attribuables** sur un intervalle.
///
/// # Cette réponse ne garantit rien, et la requête ne prétend pas le contraire
///
/// Entre cette lecture et l'attribution, une autre transaction peut prendre l'unité. La garantie
/// est la contrainte d'exclusion, jamais cette liste (FR-013). Un appelant qui la traiterait comme
/// une réservation reproduirait le verrou applicatif que le principe IV refuse.
///
/// `NOT EXISTS` plutôt qu'un `LEFT JOIN ... IS NULL` : l'index GiST de la contrainte sert
/// exactement cette forme, et c'est la seule requête de disponibilité du produit.
pub async fn unites_disponibles(
    tx: &mut sqlx::PgTransaction<'_>,
    etablissement_id: Uuid,
    categorie_id: Uuid,
    periode: PgRange<OffsetDateTime>,
) -> Result<Vec<UniteDisponible>, ErreurAttribution> {
    let lignes = sqlx::query!(
        r#"
        SELECT u.id, u.code, u.etage, u.statut_menage
        FROM hebergement.unite u
        WHERE u.etablissement_id = $1
          AND u.categorie_id = $2
          AND NOT EXISTS (
              SELECT 1
              FROM hebergement.occupation o
              WHERE o.unite_id = u.id
                AND o.periode && $3
          )
        ORDER BY u.code
        "#,
        etablissement_id,
        categorie_id,
        periode as PgRange<OffsetDateTime>,
    )
    .fetch_all(&mut **tx)
    .await?;

    let mut unites = Vec::with_capacity(lignes.len());
    for l in lignes {
        unites.push(UniteDisponible {
            id: l.id,
            code: l.code,
            etage: l.etage,
            statut_menage: StatutMenage::depuis_code(&l.statut_menage)?,
        });
    }
    Ok(unites)
}

/// **Libère une occupation — un `UPDATE`, jamais un `DELETE`.**
///
/// La période est **raccourcie** à `now()` + le battement de remise en état : la chambre
/// redevient attribuable après le ménage, pas immédiatement. `libere_le` et `statut` sont posés
/// dans le même `UPDATE`, ce que la contrainte `occupation_liberation_coherente` exige.
///
/// `WHERE statut = 'active'` : un rejeu ne touche aucune ligne, ce que l'appelant distingue.
pub async fn liberer(
    tx: &mut sqlx::PgTransaction<'_>,
    id: Uuid,
    battement_minutes: i32,
) -> Result<bool, ErreurAttribution> {
    // `now()` et le battement sont calculés **en SQL** : l'horodatage d'autorité est celui de la
    // base, unique, et non celui du processus applicatif, dont il existe plusieurs instances.
    let touchee = sqlx::query!(
        r#"
        UPDATE hebergement.occupation
        SET periode = tstzrange(
                lower(periode),
                greatest(lower(periode) + interval '1 second',
                         now() + make_interval(mins => $2)),
                '[)'
            ),
            fin_client = least(fin_client, greatest(debut_client + interval '1 second', now())),
            statut = 'liberee',
            libere_le = now()
        WHERE id = $1 AND statut = 'active'
        RETURNING id
        "#,
        id,
        battement_minutes,
    )
    .fetch_optional(&mut **tx)
    .await?;

    Ok(touchee.is_some())
}

/// Le battement de remise en état applicable à une occupation, **par la catégorie de l'unité et la
/// famille de la formule**.
///
/// Rend `0` quand aucune ligne n'est déclarée : zéro est une valeur légitime — une salle de
/// réunion qu'on n'aère pas entre deux réunions —, et un défaut posé ici serait un paramètre
/// métier en dur (principe I·c).
pub async fn battement_minutes(
    tx: &mut sqlx::PgTransaction<'_>,
    unite_id: Uuid,
    formule_id: Uuid,
) -> Result<i32, ErreurAttribution> {
    let duree = sqlx::query_scalar!(
        r#"
        SELECT t.duree_minutes
        FROM hebergement.unite u
        JOIN hebergement.formule f ON f.id = $2
        JOIN hebergement.temps_remise_en_etat t
          ON t.categorie_id = u.categorie_id AND t.famille_formule = f.famille
        WHERE u.id = $1
        "#,
        unite_id,
        formule_id,
    )
    .fetch_optional(&mut **tx)
    .await?;

    Ok(duree.unwrap_or(0))
}

/// La formule d'une occupation, et sa famille — pour la libération et la tarification.
pub async fn famille_de_l_occupation(
    tx: &mut sqlx::PgTransaction<'_>,
    occupation_id: Uuid,
) -> Result<Option<(Uuid, Uuid, String)>, ErreurAttribution> {
    let ligne = sqlx::query!(
        r#"
        SELECT o.unite_id, o.formule_id, f.famille
        FROM hebergement.occupation o
        JOIN hebergement.formule f ON f.id = o.formule_id
        WHERE o.id = $1
        "#,
        occupation_id
    )
    .fetch_optional(&mut **tx)
    .await?;

    Ok(ligne.map(|l| (l.unite_id, l.formule_id, l.famille)))
}

/// Les contraintes de durée d'une formule, et sa famille.
pub async fn contraintes_de_formule(
    tx: &mut sqlx::PgTransaction<'_>,
    formule_id: Uuid,
) -> Result<Option<(String, Option<i32>, Option<i32>)>, ErreurAttribution> {
    let ligne = sqlx::query!(
        r#"
        SELECT famille, duree_min_minutes, duree_max_minutes
        FROM hebergement.formule
        WHERE id = $1
        "#,
        formule_id
    )
    .fetch_optional(&mut **tx)
    .await?;

    Ok(ligne.map(|l| (l.famille, l.duree_min_minutes, l.duree_max_minutes)))
}

/// Horodatage d'**autorité serveur**, lu depuis la base.
///
/// L'horloge du processus applicatif n'est pas celle de la base, et deux instances d'API n'ont pas
/// la même — la base, elle, est unique (module doré, couche 4).
pub async fn maintenant(
    tx: &mut sqlx::PgTransaction<'_>,
) -> Result<OffsetDateTime, ErreurAttribution> {
    let maintenant = sqlx::query_scalar!(r#"SELECT now() AS "maintenant!""#)
        .fetch_one(&mut **tx)
        .await?;
    Ok(maintenant)
}

/// Les plages d'une demi-journée **converties en instants**, pour le jour de la demande.
///
/// # Pourquoi la conversion se fait ici, en SQL
///
/// Une plage est stockée en heure **murale** (`TIME`) : « 8 h – 12 h » vaut tous les jours, y
/// compris ceux qui n'existent pas encore. La convertir en instant demande la base de fuseaux —
/// que PostgreSQL porte et que le crate `time` ne porte pas : `time` ne connaît que des décalages
/// fixes, et aucune dépendance nouvelle n'est permise à ce cycle (principe XI).
///
/// Ce n'est pas un contournement, c'est le bon endroit : le fuseau appartient à l'établissement
/// (principe IV), et « 8 h à Abengourou le 3 août » est une question à laquelle seule une base de
/// fuseaux répond — y compris les jours de changement d'heure, là où le produit s'étendra.
///
/// `$3` est le nom de fuseau lu de l'établissement par `EstablishmentDirectory`, jamais une
/// constante.
pub async fn plages_en_instants(
    tx: &mut sqlx::PgTransaction<'_>,
    formule_id: Uuid,
    jour_de_reference: OffsetDateTime,
    fuseau: &str,
) -> Result<Vec<(OffsetDateTime, OffsetDateTime)>, ErreurAttribution> {
    let lignes = sqlx::query!(
        r#"
        SELECT
            ((date_trunc('day', $2 AT TIME ZONE $3) + p.heure_debut) AT TIME ZONE $3)
                AS "debut!",
            ((date_trunc('day', $2 AT TIME ZONE $3) + p.heure_fin) AT TIME ZONE $3)
                AS "fin!"
        FROM hebergement.plage_demi_journee p
        WHERE p.formule_id = $1
        ORDER BY p.heure_debut
        "#,
        formule_id,
        jour_de_reference,
        fuseau,
    )
    .fetch_all(&mut **tx)
    .await?;

    Ok(lignes.into_iter().map(|l| (l.debut, l.fin)).collect())
}
