//! Résolution de la chaîne d'héritage — **une seule descente, jamais quatre requêtes**.
//!
//! # Le rang de portée est calculé en SQL, depuis les colonnes renseignées
//!
//! Jamais lu dans une colonne `portee`, qui pourrait diverger des clés étrangères. Le `CHECK
//! (num_nonnulls(...) <= 1)` de la migration garantit qu'au plus une est renseignée ; le rang s'en
//! déduit sans ambiguïté.
//!
//! # Une chaîne écourtée fonctionne sans niveau inventé
//!
//! Les niveaux absents de la cible ne produisent aucune branche : un établissement sans point de
//! vente résout sur trois niveaux (FR-050). Rien n'est complété par un identifiant nul « pour
//! faire le compte » — ce qui produirait une correspondance fortuite avec les lignes de niveau
//! tenant.
//!
//! # Jointures — toutes dans le schéma `etablissements` (porte P-04)
//!
//! La résolution joint `parametre_configuration`, `etablissement_module` et `point_de_vente`.
//! **Même schéma** : la porte P-04 interdit les jointures entre schémas de **modules différents**,
//! pas les jointures en général.

use std::collections::BTreeMap;

use uuid::Uuid;

use super::modele::{EntreeCatalogue, ErreurParametre};
use crate::{Portee, ValeurResolue};

/// Une ligne résolue, avec son rang de portée.
struct LigneResolue {
    cle: String,
    valeur: serde_json::Value,
    rang: i32,
}

fn portee_depuis_rang(rang: i32) -> Portee {
    match rang {
        3 => Portee::PointDeVente,
        2 => Portee::Module,
        1 => Portee::Etablissement,
        _ => Portee::Tenant,
    }
}

/// Résout **une** clé sur la cible.
///
/// `None` signifie « définie à aucun niveau » — jamais une valeur par défaut. Un défaut rendu ici
/// serait un paramètre en dur déguisé en commodité, que le principe I·c interdit ; l'appelant qui
/// en a besoin le déclare chez lui, où on peut le voir.
pub async fn resoudre(
    tx: &mut sqlx::PgTransaction<'_>,
    etablissement_id: Option<Uuid>,
    module_code: Option<&str>,
    point_de_vente_id: Option<Uuid>,
    cle: &str,
) -> Result<Option<ValeurResolue>, ErreurParametre> {
    let ligne = sqlx::query_as!(
        LigneResolue,
        r#"
        WITH cible AS (
            SELECT $1::uuid AS etablissement_id,
                   (SELECT em.id
                      FROM etablissements.etablissement_module em
                     WHERE em.etablissement_id = $1
                       AND em.module_code = $2
                       AND em.actif) AS module_id,
                   $3::uuid AS pdv_id
        )
        SELECT pc.cle AS "cle!",
               pc.valeur AS "valeur!",
               (CASE
                    WHEN pc.point_de_vente_id       IS NOT NULL THEN 3
                    WHEN pc.etablissement_module_id IS NOT NULL THEN 2
                    WHEN pc.etablissement_id        IS NOT NULL THEN 1
                    ELSE 0
                END)::int AS "rang!"
          FROM etablissements.parametre_configuration pc
          CROSS JOIN cible
          LEFT JOIN etablissements.point_de_vente pdv
                 ON pdv.id = pc.point_de_vente_id
          LEFT JOIN etablissements.etablissement_module em_pdv
                 ON em_pdv.id = pdv.etablissement_module_id
         WHERE pc.cle = $4
           AND (
                   (pc.point_de_vente_id       = cible.pdv_id       AND em_pdv.actif)
                OR (pc.etablissement_module_id = cible.module_id)
                OR (pc.etablissement_id        = cible.etablissement_id)
                OR num_nonnulls(pc.etablissement_id,
                                pc.etablissement_module_id,
                                pc.point_de_vente_id) = 0
               )
         ORDER BY 3 DESC
         LIMIT 1
        "#,
        etablissement_id,
        module_code,
        point_de_vente_id,
        cle,
    )
    .fetch_optional(&mut **tx)
    .await?;

    Ok(ligne.map(|l| ValeurResolue {
        valeur: l.valeur,
        origine: portee_depuis_rang(l.rang),
    }))
}

/// Résout **toutes** les clés applicables à la cible, en un aller-retour.
///
/// `DISTINCT ON (cle) ... ORDER BY cle, rang DESC` garde, pour chaque clé, la ligne de portée la
/// plus spécifique. L'écran `G1` affichera une trentaine de paramètres à terme : trente appels à
/// [`resoudre`] feraient trente descentes de chaîne.
pub async fn resoudre_tout(
    tx: &mut sqlx::PgTransaction<'_>,
    etablissement_id: Option<Uuid>,
    module_code: Option<&str>,
    point_de_vente_id: Option<Uuid>,
) -> Result<BTreeMap<String, ValeurResolue>, ErreurParametre> {
    let lignes = sqlx::query_as!(
        LigneResolue,
        r#"
        WITH cible AS (
            SELECT $1::uuid AS etablissement_id,
                   (SELECT em.id
                      FROM etablissements.etablissement_module em
                     WHERE em.etablissement_id = $1
                       AND em.module_code = $2
                       AND em.actif) AS module_id,
                   $3::uuid AS pdv_id
        )
        SELECT DISTINCT ON (pc.cle)
               pc.cle AS "cle!",
               pc.valeur AS "valeur!",
               (CASE
                    WHEN pc.point_de_vente_id       IS NOT NULL THEN 3
                    WHEN pc.etablissement_module_id IS NOT NULL THEN 2
                    WHEN pc.etablissement_id        IS NOT NULL THEN 1
                    ELSE 0
                END)::int AS "rang!"
          FROM etablissements.parametre_configuration pc
          CROSS JOIN cible
          LEFT JOIN etablissements.point_de_vente pdv
                 ON pdv.id = pc.point_de_vente_id
          LEFT JOIN etablissements.etablissement_module em_pdv
                 ON em_pdv.id = pdv.etablissement_module_id
         WHERE (
                   (pc.point_de_vente_id       = cible.pdv_id       AND em_pdv.actif)
                OR (pc.etablissement_module_id = cible.module_id)
                OR (pc.etablissement_id        = cible.etablissement_id)
                OR num_nonnulls(pc.etablissement_id,
                                pc.etablissement_module_id,
                                pc.point_de_vente_id) = 0
               )
         ORDER BY pc.cle, 3 DESC
        "#,
        etablissement_id,
        module_code,
        point_de_vente_id,
    )
    .fetch_all(&mut **tx)
    .await?;

    Ok(lignes
        .into_iter()
        .map(|l| {
            (
                l.cle,
                ValeurResolue {
                    valeur: l.valeur,
                    origine: portee_depuis_rang(l.rang),
                },
            )
        })
        .collect())
}

/// Écrit une valeur à un niveau donné, ou la remplace.
///
/// Rend l'**ancienne valeur** quand il s'agissait d'une surcharge — sans elle, le grand livre
/// dirait qu'une valeur a changé sans dire depuis quoi.
#[allow(clippy::too_many_arguments)]
pub async fn ecrire(
    tx: &mut sqlx::PgTransaction<'_>,
    tenant_id: Uuid,
    id: Uuid,
    etablissement_id: Option<Uuid>,
    etablissement_module_id: Option<Uuid>,
    point_de_vente_id: Option<Uuid>,
    cle: &str,
    valeur: &serde_json::Value,
) -> Result<Option<serde_json::Value>, ErreurParametre> {
    let ancienne = sqlx::query_scalar!(
        r#"
        SELECT valeur
        FROM etablissements.parametre_configuration
        WHERE tenant_id = $1
          AND etablissement_id IS NOT DISTINCT FROM $2
          AND etablissement_module_id IS NOT DISTINCT FROM $3
          AND point_de_vente_id IS NOT DISTINCT FROM $4
          AND cle = $5
        "#,
        tenant_id,
        etablissement_id,
        etablissement_module_id,
        point_de_vente_id,
        cle,
    )
    .fetch_optional(&mut **tx)
    .await?;

    // `ON CONFLICT` sur la contrainte d'unicité — celle qui porte `NULLS NOT DISTINCT`. Sans elle,
    // deux surcharges de niveau tenant portant la même clé passeraient toutes les deux et la
    // résolution en choisirait une au hasard.
    sqlx::query!(
        r#"
        INSERT INTO etablissements.parametre_configuration
            (id, tenant_id, etablissement_id, etablissement_module_id, point_de_vente_id,
             cle, valeur)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT ON CONSTRAINT parametre_configuration_unicite
        DO UPDATE SET valeur = EXCLUDED.valeur, modifie_le = now()
        "#,
        id,
        tenant_id,
        etablissement_id,
        etablissement_module_id,
        point_de_vente_id,
        cle,
        valeur,
    )
    .execute(&mut **tx)
    .await?;

    Ok(ancienne)
}

/// Lit une entrée du catalogue.
pub async fn entree_catalogue(
    tx: &mut sqlx::PgTransaction<'_>,
    cle: &str,
) -> Result<Option<EntreeCatalogue>, ErreurParametre> {
    let entree = sqlx::query_as!(
        EntreeCatalogue,
        r#"
        SELECT cle, type_valeur, portee_la_plus_basse, story, libelle_cle, description_cle
        FROM etablissements.parametre_catalogue
        WHERE cle = $1
        "#,
        cle
    )
    .fetch_optional(&mut **tx)
    .await?;

    Ok(entree)
}

/// Le catalogue complet.
pub async fn catalogue(
    tx: &mut sqlx::PgTransaction<'_>,
) -> Result<Vec<EntreeCatalogue>, ErreurParametre> {
    let entrees = sqlx::query_as!(
        EntreeCatalogue,
        r#"
        SELECT cle, type_valeur, portee_la_plus_basse, story, libelle_cle, description_cle
        FROM etablissements.parametre_catalogue
        ORDER BY cle
        "#
    )
    .fetch_all(&mut **tx)
    .await?;

    Ok(entrees)
}

/// L'activation d'un service, si elle existe dans le tenant courant.
///
/// **Sans filtre sur `actif`** : on peut écrire une surcharge sur un service désactivé. Elle sera
/// inerte jusqu'à la réactivation, ce qui est exactement le comportement voulu (FR-051) — refuser
/// l'écriture obligerait à réactiver un service pour préparer sa configuration.
pub async fn module_du_tenant(
    tx: &mut sqlx::PgTransaction<'_>,
    etablissement_module_id: Uuid,
) -> Result<bool, ErreurParametre> {
    let existe = sqlx::query_scalar!(
        r#"
        SELECT EXISTS (
            SELECT 1 FROM etablissements.etablissement_module WHERE id = $1
        ) AS "existe!"
        "#,
        etablissement_module_id
    )
    .fetch_one(&mut **tx)
    .await?;
    Ok(existe)
}

pub async fn etablissement_du_tenant(
    tx: &mut sqlx::PgTransaction<'_>,
    etablissement_id: Uuid,
) -> Result<bool, ErreurParametre> {
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

pub async fn point_de_vente_du_tenant(
    tx: &mut sqlx::PgTransaction<'_>,
    point_de_vente_id: Uuid,
) -> Result<bool, ErreurParametre> {
    let existe = sqlx::query_scalar!(
        r#"
        SELECT EXISTS (
            SELECT 1 FROM etablissements.point_de_vente WHERE id = $1
        ) AS "existe!"
        "#,
        point_de_vente_id
    )
    .fetch_one(&mut **tx)
    .await?;
    Ok(existe)
}
