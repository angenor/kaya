//! Accès aux données des points de vente et de leurs tables.

use uuid::Uuid;

use super::modele::{CreerPointDeVente, ErreurPointDeVente, PointDeVenteVue, TableVue};
use crate::Issue;

struct Ligne {
    id: Uuid,
    etablissement_id: Uuid,
    module_code: String,
    nom: String,
    caisse_id: Option<Uuid>,
    actif: bool,
    cree_le: time::OffsetDateTime,
}

/// Insère un point de vente, ou constate qu'il existe déjà.
pub async fn inserer(
    tx: &mut sqlx::PgTransaction<'_>,
    tenant_id: Uuid,
    etablissement_id: Uuid,
    etablissement_module_id: Uuid,
    demande: &CreerPointDeVente,
) -> Result<(Uuid, Issue), ErreurPointDeVente> {
    let insere = sqlx::query_scalar!(
        r#"
        INSERT INTO etablissements.point_de_vente
            (id, tenant_id, etablissement_id, etablissement_module_id, nom, caisse_id)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (id) DO NOTHING
        RETURNING id
        "#,
        demande.id,
        tenant_id,
        etablissement_id,
        etablissement_module_id,
        demande.nom,
        demande.caisse_id,
    )
    .fetch_optional(&mut **tx)
    .await?;

    Ok(match insere {
        Some(id) => (id, Issue::Creee),
        None => (demande.id, Issue::DejaPresente),
    })
}

/// Lit un point de vente **avec ses tables**.
pub async fn lire(
    tx: &mut sqlx::PgTransaction<'_>,
    id: Uuid,
) -> Result<Option<PointDeVenteVue>, ErreurPointDeVente> {
    let ligne = sqlx::query_as!(
        Ligne,
        r#"
        SELECT pdv.id, pdv.etablissement_id, em.module_code, pdv.nom,
               pdv.caisse_id, pdv.actif, pdv.cree_le
        FROM etablissements.point_de_vente pdv
        JOIN etablissements.etablissement_module em ON em.id = pdv.etablissement_module_id
        WHERE pdv.id = $1
        "#,
        id
    )
    .fetch_optional(&mut **tx)
    .await?;

    let Some(ligne) = ligne else {
        return Ok(None);
    };

    let tables = tables_du_point_de_vente(tx, ligne.id).await?;
    Ok(Some(en_vue(ligne, tables)))
}

/// Les points de vente **actifs** d'un établissement, chacun avec ses tables.
pub async fn lister(
    tx: &mut sqlx::PgTransaction<'_>,
    etablissement_id: Uuid,
) -> Result<Vec<PointDeVenteVue>, ErreurPointDeVente> {
    let lignes = sqlx::query_as!(
        Ligne,
        r#"
        SELECT pdv.id, pdv.etablissement_id, em.module_code, pdv.nom,
               pdv.caisse_id, pdv.actif, pdv.cree_le
        FROM etablissements.point_de_vente pdv
        JOIN etablissements.etablissement_module em ON em.id = pdv.etablissement_module_id
        WHERE pdv.etablissement_id = $1 AND pdv.actif AND em.actif
        ORDER BY pdv.nom, pdv.id
        "#,
        etablissement_id
    )
    .fetch_all(&mut **tx)
    .await?;

    let mut vues = Vec::with_capacity(lignes.len());
    for ligne in lignes {
        let tables = tables_du_point_de_vente(tx, ligne.id).await?;
        vues.push(en_vue(ligne, tables));
    }
    Ok(vues)
}

fn en_vue(ligne: Ligne, tables: Vec<TableVue>) -> PointDeVenteVue {
    PointDeVenteVue {
        id: ligne.id,
        etablissement_id: ligne.etablissement_id,
        module_code: ligne.module_code,
        nom: ligne.nom,
        caisse_id: ligne.caisse_id,
        actif: ligne.actif,
        tables,
        cree_le: ligne.cree_le,
    }
}

/// Les tables **actives** d'un point de vente.
///
/// Une liste vide **est** le comptoir. Aucune méthode ne rend cette information autrement : une
/// seconde source pourrait contredire celle-ci.
pub async fn tables_du_point_de_vente(
    tx: &mut sqlx::PgTransaction<'_>,
    point_de_vente_id: Uuid,
) -> Result<Vec<TableVue>, ErreurPointDeVente> {
    let tables = sqlx::query_as!(
        TableVue,
        r#"
        SELECT id, libelle
        FROM etablissements.table_pdv
        WHERE point_de_vente_id = $1 AND actif
        ORDER BY libelle, id
        "#,
        point_de_vente_id
    )
    .fetch_all(&mut **tx)
    .await?;

    Ok(tables)
}

/// Applique une modification.
pub async fn modifier(
    tx: &mut sqlx::PgTransaction<'_>,
    id: Uuid,
    nom: &str,
    caisse_id: Option<Uuid>,
    actif: bool,
) -> Result<(), ErreurPointDeVente> {
    let touchees = sqlx::query!(
        r#"
        UPDATE etablissements.point_de_vente
        SET nom = $2, caisse_id = $3, actif = $4, modifie_le = now()
        WHERE id = $1
        "#,
        id,
        nom,
        caisse_id,
        actif,
    )
    .execute(&mut **tx)
    .await?
    .rows_affected();

    if touchees == 0 {
        return Err(ErreurPointDeVente::Inconnu);
    }
    Ok(())
}

/// Désactive toutes les tables d'un point de vente.
///
/// **Désactive, ne supprime pas.** Une table retirée du plan de salle reste référencée par les
/// commandes déjà passées ; les supprimer rendrait illisible l'historique d'un soir de service.
pub async fn desactiver_toutes_les_tables(
    tx: &mut sqlx::PgTransaction<'_>,
    point_de_vente_id: Uuid,
) -> Result<Vec<Uuid>, ErreurPointDeVente> {
    let desactivees = sqlx::query_scalar!(
        r#"
        UPDATE etablissements.table_pdv
        SET actif = false
        WHERE point_de_vente_id = $1 AND actif
        RETURNING id
        "#,
        point_de_vente_id
    )
    .fetch_all(&mut **tx)
    .await?;

    Ok(desactivees)
}

/// Pose une table, ou la réactive si elle existait sous le même libellé.
///
/// `ON CONFLICT (point_de_vente_id, libelle)` plutôt que sur `id` : le personnel qui repose la
/// « Terrasse 3 » après l'avoir retirée s'attend à retrouver **sa** table, pas une seconde ligne
/// portant le même nom — que l'unicité refuserait de toute façon.
pub async fn poser_table(
    tx: &mut sqlx::PgTransaction<'_>,
    tenant_id: Uuid,
    point_de_vente_id: Uuid,
    id: Uuid,
    libelle: &str,
) -> Result<Uuid, ErreurPointDeVente> {
    let pose = sqlx::query_scalar!(
        r#"
        INSERT INTO etablissements.table_pdv
            (id, tenant_id, point_de_vente_id, libelle, actif)
        VALUES ($1, $2, $3, $4, true)
        ON CONFLICT (point_de_vente_id, libelle) DO UPDATE SET actif = true
        RETURNING id
        "#,
        id,
        tenant_id,
        point_de_vente_id,
        libelle,
    )
    .fetch_one(&mut **tx)
    .await?;

    Ok(pose)
}

/// L'activation d'un service, si elle existe **et est active**.
pub async fn service_actif(
    tx: &mut sqlx::PgTransaction<'_>,
    etablissement_id: Uuid,
    module_code: &str,
) -> Result<Option<Uuid>, ErreurPointDeVente> {
    let id = sqlx::query_scalar!(
        r#"
        SELECT id
        FROM etablissements.etablissement_module
        WHERE etablissement_id = $1 AND module_code = $2 AND actif
        "#,
        etablissement_id,
        module_code,
    )
    .fetch_optional(&mut **tx)
    .await?;

    Ok(id)
}

pub async fn etablissement_existe(
    tx: &mut sqlx::PgTransaction<'_>,
    etablissement_id: Uuid,
) -> Result<bool, ErreurPointDeVente> {
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
