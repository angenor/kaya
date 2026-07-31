//! Accès aux données de l'identité visuelle.

use uuid::Uuid;

use super::modele::{BrandingNiveau, ErreurBranding};

/// Lit l'identité visuelle **d'un niveau précis**.
///
/// `etablissement_id = None` lit le niveau tenant. `IS NOT DISTINCT FROM` plutôt que `=` : `NULL =
/// NULL` vaut `NULL` en SQL, et la comparaison ordinaire ne trouverait jamais la ligne du niveau
/// tenant.
pub async fn lire_niveau(
    tx: &mut sqlx::PgTransaction<'_>,
    etablissement_id: Option<Uuid>,
) -> Result<Option<BrandingNiveau>, ErreurBranding> {
    let ligne = sqlx::query_as!(
        BrandingNiveau,
        r#"
        SELECT logo_objet_cle, couleur_primaire, entete_document,
               pied_document, mentions_legales, coordonnees
        FROM etablissements.branding
        WHERE etablissement_id IS NOT DISTINCT FROM $1
        "#,
        etablissement_id
    )
    .fetch_optional(&mut **tx)
    .await?;

    Ok(ligne)
}

/// Écrit l'identité visuelle d'un niveau, ou la remplace.
///
/// Rend le contenu **avant** modification, pour la charge utile de l'événement.
pub async fn ecrire(
    tx: &mut sqlx::PgTransaction<'_>,
    tenant_id: Uuid,
    id: Uuid,
    etablissement_id: Option<Uuid>,
    contenu: &BrandingNiveau,
) -> Result<Option<BrandingNiveau>, ErreurBranding> {
    let avant = lire_niveau(tx, etablissement_id).await?;

    sqlx::query!(
        r#"
        INSERT INTO etablissements.branding
            (id, tenant_id, etablissement_id, logo_objet_cle, couleur_primaire,
             entete_document, pied_document, mentions_legales, coordonnees)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        ON CONFLICT ON CONSTRAINT branding_unicite DO UPDATE
        SET logo_objet_cle   = EXCLUDED.logo_objet_cle,
            couleur_primaire = EXCLUDED.couleur_primaire,
            entete_document  = EXCLUDED.entete_document,
            pied_document    = EXCLUDED.pied_document,
            mentions_legales = EXCLUDED.mentions_legales,
            coordonnees      = EXCLUDED.coordonnees,
            modifie_le       = now()
        "#,
        id,
        tenant_id,
        etablissement_id,
        contenu.logo_objet_cle,
        contenu.couleur_primaire,
        contenu.entete_document,
        contenu.pied_document,
        contenu.mentions_legales,
        contenu.coordonnees,
    )
    .execute(&mut **tx)
    .await?;

    Ok(avant)
}

/// Met à jour **le seul logo**, sans toucher aux autres champs.
///
/// Le téléversement est une opération à part : l'écran envoie le fichier puis enregistre le reste.
/// Passer par [`ecrire`] écraserait les champs non transmis avec des `NULL`.
pub async fn poser_logo(
    tx: &mut sqlx::PgTransaction<'_>,
    tenant_id: Uuid,
    id: Uuid,
    etablissement_id: Option<Uuid>,
    objet_cle: &str,
) -> Result<(), ErreurBranding> {
    sqlx::query!(
        r#"
        INSERT INTO etablissements.branding (id, tenant_id, etablissement_id, logo_objet_cle)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT ON CONSTRAINT branding_unicite DO UPDATE
        SET logo_objet_cle = EXCLUDED.logo_objet_cle, modifie_le = now()
        "#,
        id,
        tenant_id,
        etablissement_id,
        objet_cle,
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}

pub async fn etablissement_existe(
    tx: &mut sqlx::PgTransaction<'_>,
    etablissement_id: Uuid,
) -> Result<bool, ErreurBranding> {
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
