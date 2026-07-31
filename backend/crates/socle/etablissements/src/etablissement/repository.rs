//! Accès aux données de l'établissement — **patron du module doré, couche 3**.
//!
//! Les points que ce fichier reprend sans les réinventer (`docs/module-dore.md`) :
//!
//! - toutes les requêtes passent par les **macros `query!` sur littéral**, donc vérifiées à la
//!   compilation contre la vraie base (porte P-18) ; `AssertSqlSafe` n'apparaît nulle part ;
//! - le repository **prend la transaction, il ne l'ouvre pas** — c'est le service qui décide de
//!   la portée transactionnelle, parce que c'est lui qui doit y inclure l'événement outbox ;
//! - `ON CONFLICT (id) DO NOTHING ... RETURNING` distingue `201` de `200` sans second
//!   aller-retour dans le cas normal ;
//! - aucune jointure entre schémas de modules (porte P-04).

use uuid::Uuid;

use super::modele::{CreerEtablissement, ErreurEtablissement, EtablissementVue};
use crate::{Classement, Issue};

/// Ligne telle qu'elle est en base — `classement` et `etoiles` encore séparés.
struct Ligne {
    id: Uuid,
    nom: String,
    juridiction: String,
    classement: String,
    etoiles: Option<i16>,
    commune: String,
    fuseau_horaire: String,
    devise: String,
    adresse: Option<String>,
    ncc: Option<String>,
    cree_le: time::OffsetDateTime,
    modifie_le: time::OffsetDateTime,
}

impl From<Ligne> for EtablissementVue {
    fn from(l: Ligne) -> Self {
        EtablissementVue {
            id: l.id,
            nom: l.nom,
            juridiction: l.juridiction,
            classement: l.classement,
            // La base impose déjà `etoiles > 0` et l'égalité de conditions avec le classement ;
            // la conversion ne peut donc échouer que sur une ligne écrite hors de ces contraintes,
            // ce qu'aucun chemin n'autorise. `try_from` plutôt qu'un `as` : une troncature
            // silencieuse ferait d'un nombre aberrant un nombre plausible.
            etoiles: l.etoiles.and_then(|n| u8::try_from(n).ok()),
            commune: l.commune,
            fuseau_horaire: l.fuseau_horaire,
            devise: l.devise,
            adresse: l.adresse,
            ncc: l.ncc,
            cree_le: l.cree_le,
            modifie_le: l.modifie_le,
        }
    }
}

/// Insère un établissement, ou constate qu'il existe déjà.
pub async fn inserer(
    tx: &mut sqlx::PgTransaction<'_>,
    tenant_id: Uuid,
    demande: &CreerEtablissement,
) -> Result<(EtablissementVue, Issue), ErreurEtablissement> {
    let classement_code = demande.classement.code();
    let etoiles = demande.classement.etoiles().map(i16::from);

    let insere = sqlx::query_as!(
        Ligne,
        r#"
        INSERT INTO etablissements.etablissement
            (id, tenant_id, nom, fuseau_horaire, devise,
             juridiction, classement, etoiles, commune, adresse, ncc)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        ON CONFLICT (id) DO NOTHING
        RETURNING id, nom, juridiction, classement, etoiles, commune,
                  fuseau_horaire, devise, adresse, ncc, cree_le, modifie_le
        "#,
        demande.id,
        tenant_id,
        demande.nom,
        demande.fuseau_horaire,
        demande.devise,
        demande.juridiction,
        classement_code,
        etoiles,
        demande.commune,
        demande.adresse,
        demande.ncc,
    )
    .fetch_optional(&mut **tx)
    .await?;

    match insere {
        Some(ligne) => Ok((ligne.into(), Issue::Creee)),
        None => {
            // **Le serveur fait foi en conflit** (principe VI) : le corps rendu est la ligne telle
            // qu'elle est en base, pas celle que le client vient de proposer.
            let existant = lire(tx, demande.id)
                .await?
                .ok_or(ErreurEtablissement::Inconnu)?;
            Ok((existant, Issue::DejaPresente))
        }
    }
}

/// Lit un établissement par identifiant, **dans le tenant courant**.
pub async fn lire(
    tx: &mut sqlx::PgTransaction<'_>,
    id: Uuid,
) -> Result<Option<EtablissementVue>, ErreurEtablissement> {
    let ligne = sqlx::query_as!(
        Ligne,
        r#"
        SELECT id, nom, juridiction, classement, etoiles, commune,
               fuseau_horaire, devise, adresse, ncc, cree_le, modifie_le
        FROM etablissements.etablissement
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(&mut **tx)
    .await?;

    Ok(ligne.map(Into::into))
}

/// Liste les établissements du tenant courant.
///
/// Tri sur `nom` : c'est l'ordre qu'attend un propriétaire qui possède deux établissements, et il
/// ne dépend d'aucune horloge. La liste est bornée par le nombre d'établissements d'un tenant —
/// une poignée — donc sans pagination.
pub async fn lister(
    tx: &mut sqlx::PgTransaction<'_>,
) -> Result<Vec<EtablissementVue>, ErreurEtablissement> {
    let lignes = sqlx::query_as!(
        Ligne,
        r#"
        SELECT id, nom, juridiction, classement, etoiles, commune,
               fuseau_horaire, devise, adresse, ncc, cree_le, modifie_le
        FROM etablissements.etablissement
        ORDER BY nom, id
        "#
    )
    .fetch_all(&mut **tx)
    .await?;

    Ok(lignes.into_iter().map(Into::into).collect())
}

/// Applique une modification. **`modifie_le` vient de `now()`**, donc de l'horloge de la base.
///
/// Les paramètres sont les valeurs **finales**, déjà fusionnées par le service : le repository ne
/// décide pas de ce qui change, il écrit ce qu'on lui donne. Un `COALESCE($n, colonne)` en SQL
/// serait plus court et rendrait impossible de distinguer « non touché » de « remis à null » le
/// jour où un champ deviendra effaçable.
#[allow(clippy::too_many_arguments)]
pub async fn modifier(
    tx: &mut sqlx::PgTransaction<'_>,
    id: Uuid,
    nom: &str,
    classement: Classement,
    commune: &str,
    fuseau_horaire: &str,
    devise: &str,
    adresse: Option<&str>,
    ncc: Option<&str>,
) -> Result<EtablissementVue, ErreurEtablissement> {
    let classement_code = classement.code();
    let etoiles = classement.etoiles().map(i16::from);

    let ligne = sqlx::query_as!(
        Ligne,
        r#"
        UPDATE etablissements.etablissement
        SET nom = $2, classement = $3, etoiles = $4, commune = $5,
            fuseau_horaire = $6, devise = $7, adresse = $8, ncc = $9,
            modifie_le = now()
        WHERE id = $1
        RETURNING id, nom, juridiction, classement, etoiles, commune,
                  fuseau_horaire, devise, adresse, ncc, cree_le, modifie_le
        "#,
        id,
        nom,
        classement_code,
        etoiles,
        commune,
        fuseau_horaire,
        devise,
        adresse,
        ncc,
    )
    .fetch_optional(&mut **tx)
    .await?
    .ok_or(ErreurEtablissement::Inconnu)?;

    Ok(ligne.into())
}

/// Combien d'opérations financières cet établissement porte-t-il ?
///
/// # Posée à VIDE, et c'est écrit ici plutôt que deviné
///
/// Elle rend **zéro** tant qu'aucune table d'opération financière n'existe — ni encaissement
/// (CAI-02), ni document fiscal (FIS-02). Le refus `devise_figee` est donc inatteignable à ce
/// cycle, ce qui est **exact** : on ne fige pas une devise que rien n'a encore utilisée.
///
/// Le cycle CAI branche cette fonction sur `caisse.encaissement`. La poser maintenant, à vide,
/// évite que le contrôle soit ajouté après coup — c'est-à-dire après qu'un établissement aura
/// changé de devise en production, laissant des montants dont on ne sait plus dans quelle unité
/// ils sont libellés.
pub async fn compter_operations_financieres(
    _tx: &mut sqlx::PgTransaction<'_>,
    _etablissement_id: Uuid,
) -> Result<i64, ErreurEtablissement> {
    Ok(0)
}
