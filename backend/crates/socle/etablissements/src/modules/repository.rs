//! Accès aux données des services et de leurs capacités.
//!
//! **Aucune requête ne joint deux schémas de modules** (porte P-04). Les jointures écrites ici
//! restent dans `etablissements` — `etablissement_module` avec `module_activite`,
//! `module_capacite` avec `capacite` — ce qui est conforme : la règle porte sur les schémas de
//! **modules différents**, pas sur les jointures en général.

use uuid::Uuid;

use super::modele::{CapaciteDuService, DeclarerCapacite, ErreurModules, ServiceActif};
use crate::Issue;

/// Une entrée du référentiel des modules.
pub struct ModuleReferentiel {
    pub code: String,
    pub implementee: bool,
    pub libelle_cle: String,
    pub ordre: i16,
}

/// Lit une entrée du référentiel des modules.
///
/// `None` signifie **inconnu**, ce qui n'est pas la même chose que « connu mais non implémenté ».
/// La distinction change le message rendu à l'exploitant, et c'est la raison d'être de la colonne
/// `implementee` (research.md R-02).
pub async fn module_du_referentiel(
    tx: &mut sqlx::PgTransaction<'_>,
    code: &str,
) -> Result<Option<ModuleReferentiel>, ErreurModules> {
    let ligne = sqlx::query_as!(
        ModuleReferentiel,
        r#"
        SELECT code, implementee, libelle_cle, ordre
        FROM etablissements.module_activite
        WHERE code = $1
        "#,
        code
    )
    .fetch_optional(&mut **tx)
    .await?;

    Ok(ligne)
}

/// Le référentiel complet des modules, ordre d'affichage.
pub async fn referentiel_modules(
    tx: &mut sqlx::PgTransaction<'_>,
) -> Result<Vec<ModuleReferentiel>, ErreurModules> {
    let lignes = sqlx::query_as!(
        ModuleReferentiel,
        r#"
        SELECT code, implementee, libelle_cle, ordre
        FROM etablissements.module_activite
        ORDER BY ordre, code
        "#
    )
    .fetch_all(&mut **tx)
    .await?;

    Ok(lignes)
}

/// Une entrée du référentiel des capacités.
pub struct CapaciteReferentiel {
    pub code: String,
    pub implementee: bool,
    pub libelle_cle: String,
    pub ordre: i16,
}

pub async fn capacite_du_referentiel(
    tx: &mut sqlx::PgTransaction<'_>,
    code: &str,
) -> Result<Option<CapaciteReferentiel>, ErreurModules> {
    let ligne = sqlx::query_as!(
        CapaciteReferentiel,
        r#"
        SELECT code, implementee, libelle_cle, ordre
        FROM etablissements.capacite
        WHERE code = $1
        "#,
        code
    )
    .fetch_optional(&mut **tx)
    .await?;

    Ok(ligne)
}

pub async fn referentiel_capacites(
    tx: &mut sqlx::PgTransaction<'_>,
) -> Result<Vec<CapaciteReferentiel>, ErreurModules> {
    let lignes = sqlx::query_as!(
        CapaciteReferentiel,
        r#"
        SELECT code, implementee, libelle_cle, ordre
        FROM etablissements.capacite
        ORDER BY ordre, code
        "#
    )
    .fetch_all(&mut **tx)
    .await?;

    Ok(lignes)
}

/// Une entrée du référentiel des profils de stock.
pub struct ProfilReferentiel {
    pub code: String,
    pub implementee: bool,
    pub libelle_cle: String,
    pub motif_refus_cle: Option<String>,
    pub ordre: i16,
}

pub async fn profil_du_referentiel(
    tx: &mut sqlx::PgTransaction<'_>,
    code: &str,
) -> Result<Option<ProfilReferentiel>, ErreurModules> {
    let ligne = sqlx::query_as!(
        ProfilReferentiel,
        r#"
        SELECT code, implementee, libelle_cle, motif_refus_cle, ordre
        FROM etablissements.profil_stock
        WHERE code = $1
        "#,
        code
    )
    .fetch_optional(&mut **tx)
    .await?;

    Ok(ligne)
}

pub async fn referentiel_profils(
    tx: &mut sqlx::PgTransaction<'_>,
) -> Result<Vec<ProfilReferentiel>, ErreurModules> {
    let lignes = sqlx::query_as!(
        ProfilReferentiel,
        r#"
        SELECT code, implementee, libelle_cle, motif_refus_cle, ordre
        FROM etablissements.profil_stock
        ORDER BY ordre, code
        "#
    )
    .fetch_all(&mut **tx)
    .await?;

    Ok(lignes)
}

/// Active un service — **première activation seulement**.
///
/// `ON CONFLICT (etablissement_id, module_code) DO NOTHING` : la seconde tentative ne crée rien,
/// et l'appelant bascule alors sur [`reactiver`]. La contrainte d'unicité porte sur le couple, pas
/// sur l'identifiant : deux clients qui activent `BAR` avec deux UUID différents doivent produire
/// **une** activation, pas deux.
pub async fn activer(
    tx: &mut sqlx::PgTransaction<'_>,
    tenant_id: Uuid,
    id: Uuid,
    etablissement_id: Uuid,
    module_code: &str,
) -> Result<Option<Uuid>, ErreurModules> {
    let insere = sqlx::query_scalar!(
        r#"
        INSERT INTO etablissements.etablissement_module
            (id, tenant_id, etablissement_id, module_code, module_implemente)
        VALUES ($1, $2, $3, $4, true)
        ON CONFLICT (etablissement_id, module_code) DO NOTHING
        RETURNING id
        "#,
        id,
        tenant_id,
        etablissement_id,
        module_code,
    )
    .fetch_optional(&mut **tx)
    .await?;

    Ok(insere)
}

/// État d'un service déjà présent en base.
pub struct EtatService {
    pub id: Uuid,
    pub actif: bool,
}

pub async fn etat(
    tx: &mut sqlx::PgTransaction<'_>,
    etablissement_id: Uuid,
    module_code: &str,
) -> Result<Option<EtatService>, ErreurModules> {
    let ligne = sqlx::query_as!(
        EtatService,
        r#"
        SELECT id, actif
        FROM etablissements.etablissement_module
        WHERE etablissement_id = $1 AND module_code = $2
        "#,
        etablissement_id,
        module_code,
    )
    .fetch_optional(&mut **tx)
    .await?;

    Ok(ligne)
}

/// Bascule `actif`. **Jamais un `DELETE`** — le privilège de la table l'interdit d'ailleurs.
///
/// Rend `true` si l'état a réellement changé. Une bascule vers l'état courant n'écrit rien et
/// n'émet donc aucun événement : le grand livre enregistre les transitions, pas les requêtes.
pub async fn basculer(
    tx: &mut sqlx::PgTransaction<'_>,
    id: Uuid,
    actif: bool,
) -> Result<bool, ErreurModules> {
    let touchees = sqlx::query!(
        r#"
        UPDATE etablissements.etablissement_module
        SET actif = $2,
            active_le    = CASE WHEN $2 THEN now() ELSE active_le END,
            desactive_le = CASE WHEN $2 THEN NULL  ELSE now()      END,
            modifie_le   = now()
        WHERE id = $1 AND actif <> $2
        "#,
        id,
        actif,
    )
    .execute(&mut **tx)
    .await?
    .rows_affected();

    Ok(touchees > 0)
}

/// Les services **actifs** d'un établissement, avec leurs capacités.
///
/// Deux requêtes plutôt qu'une jointure à plat : la seconde ne s'exécute que sur les services
/// trouvés, et l'assemblage en mémoire évite de dupliquer les colonnes du service par capacité.
pub async fn services_actifs(
    tx: &mut sqlx::PgTransaction<'_>,
    etablissement_id: Uuid,
) -> Result<Vec<ServiceActif>, ErreurModules> {
    let services = sqlx::query!(
        r#"
        SELECT em.id, em.module_code, ma.libelle_cle, ma.ordre, em.active_le
        FROM etablissements.etablissement_module em
        JOIN etablissements.module_activite ma ON ma.code = em.module_code
        WHERE em.etablissement_id = $1 AND em.actif
        ORDER BY ma.ordre, em.module_code
        "#,
        etablissement_id
    )
    .fetch_all(&mut **tx)
    .await?;

    let mut resultat = Vec::with_capacity(services.len());
    for service in services {
        let capacites = capacites_du_service(tx, service.id).await?;
        resultat.push(ServiceActif {
            id: service.id,
            module_code: service.module_code,
            libelle_cle: service.libelle_cle,
            ordre: service.ordre,
            active_le: service.active_le,
            capacites,
        });
    }

    Ok(resultat)
}

/// Les capacités déclarées par un service.
///
/// **Lues à travers `etablissement_module.actif`** : la désactivation d'un service rend ses
/// déclarations inertes sans les toucher, donc sans rien perdre à la réactivation (FR-037).
pub async fn capacites_du_service(
    tx: &mut sqlx::PgTransaction<'_>,
    etablissement_module_id: Uuid,
) -> Result<Vec<CapaciteDuService>, ErreurModules> {
    let lignes = sqlx::query!(
        r#"
        SELECT mc.id, mc.capacite_code, mc.profil_code, c.libelle_cle
        FROM etablissements.module_capacite mc
        JOIN etablissements.capacite c ON c.code = mc.capacite_code
        JOIN etablissements.etablissement_module em ON em.id = mc.etablissement_module_id
        WHERE mc.etablissement_module_id = $1 AND em.actif
        ORDER BY c.ordre, mc.capacite_code
        "#,
        etablissement_module_id
    )
    .fetch_all(&mut **tx)
    .await?;

    Ok(lignes
        .into_iter()
        .map(|l| CapaciteDuService {
            id: l.id,
            capacite_code: l.capacite_code,
            profil_code: l.profil_code,
            libelle_cle: l.libelle_cle,
        })
        .collect())
}

/// Déclare une capacité, idempotent sur l'identifiant client.
pub async fn declarer_capacite(
    tx: &mut sqlx::PgTransaction<'_>,
    tenant_id: Uuid,
    etablissement_module_id: Uuid,
    demande: &DeclarerCapacite,
) -> Result<Issue, ErreurModules> {
    let insere = sqlx::query_scalar!(
        r#"
        INSERT INTO etablissements.module_capacite
            (id, tenant_id, etablissement_module_id,
             capacite_code, capacite_implementee, profil_code, profil_implemente)
        VALUES ($1, $2, $3, $4, true, $5, true)
        ON CONFLICT (etablissement_module_id, capacite_code) DO NOTHING
        RETURNING id
        "#,
        demande.id,
        tenant_id,
        etablissement_module_id,
        demande.capacite_code,
        demande.profil_code,
    )
    .fetch_optional(&mut **tx)
    .await?;

    Ok(match insere {
        Some(_) => Issue::Creee,
        None => Issue::DejaPresente,
    })
}

/// Vérifie qu'un établissement existe **dans le tenant courant**.
pub async fn etablissement_existe(
    tx: &mut sqlx::PgTransaction<'_>,
    etablissement_id: Uuid,
) -> Result<bool, ErreurModules> {
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

/// Codes des modules **actifs** — la requête du trait [`crate::RegistreModules`].
pub async fn codes_actifs(
    tx: &mut sqlx::PgTransaction<'_>,
    etablissement_id: Uuid,
) -> Result<Vec<String>, ErreurModules> {
    let codes = sqlx::query_scalar!(
        r#"
        SELECT em.module_code
        FROM etablissements.etablissement_module em
        JOIN etablissements.module_activite ma ON ma.code = em.module_code
        WHERE em.etablissement_id = $1 AND em.actif
        ORDER BY ma.ordre, em.module_code
        "#,
        etablissement_id
    )
    .fetch_all(&mut **tx)
    .await?;

    Ok(codes)
}
