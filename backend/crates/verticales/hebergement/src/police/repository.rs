//! Accès aux données de la fiche de police et de sa numérotation.
//!
//! # ★ L'incrément est un `UPDATE … RETURNING`, et c'est ce qui sérialise
//!
//! Le verrou de ligne posé par l'`UPDATE` est **la définition même de la classe B** : deux
//! arrivées simultanées sur le même établissement s'attendent, et aucune ne reçoit le numéro de
//! l'autre.
//!
//! Une `SEQUENCE` aurait deux propriétés fatales — globale au schéma, et laissant des trous, car
//! `nextval` consomme même sur transaction annulée. Une numérotation de document opérationnel doit
//! être **continue par établissement** ; c'est ce que la gendarmerie attend, et c'est le défaut
//! corrigé par `0012` au cycle 002.

use uuid::Uuid;

use super::super::sejour::modele::FichePolice;
use crate::erreurs::ErreurSejour;

/// Prend le numéro suivant pour un établissement, **dans la transaction fournie**.
///
/// `INSERT … ON CONFLICT DO UPDATE` crée le compteur au premier appel et l'incrémente ensuite, en
/// **une** requête. Deux requêtes — lire puis écrire — rouvriraient exactement la fenêtre que le
/// verrou de ligne ferme.
pub async fn numero_suivant(
    tx: &mut sqlx::PgTransaction<'_>,
    tenant_id: Uuid,
    etablissement_id: Uuid,
) -> Result<i64, ErreurSejour> {
    let numero = sqlx::query_scalar!(
        r#"
        INSERT INTO hebergement.numerotation_fiche_police
            (tenant_id, etablissement_id, dernier_numero)
        VALUES ($1, $2, 1)
        ON CONFLICT (tenant_id, etablissement_id)
        DO UPDATE SET dernier_numero = hebergement.numerotation_fiche_police.dernier_numero + 1
        RETURNING dernier_numero
        "#,
        tenant_id,
        etablissement_id,
    )
    .fetch_one(&mut **tx)
    .await?;

    Ok(numero)
}

/// Écrit la fiche de police d'un séjour.
///
/// **`complete = false` quand aucun client n'est rattaché** (FR-047) : la fiche existe et est
/// numérotée, et **aucun champ de remplissage n'y figure**. Ni fabriquée, ni silencieusement
/// omise. Terme utilisateur : « Identité à compléter ».
pub async fn ecrire(
    tx: &mut sqlx::PgTransaction<'_>,
    tenant_id: Uuid,
    id: Uuid,
    etablissement_id: Uuid,
    sejour_id: Uuid,
    numero: i64,
    complete: bool,
) -> Result<bool, ErreurSejour> {
    let insere = sqlx::query_scalar!(
        r#"
        INSERT INTO hebergement.fiche_police
            (id, tenant_id, etablissement_id, sejour_id, numero, complete, completee_le)
        VALUES ($1, $2, $3, $4, $5, $6, CASE WHEN $6 THEN now() ELSE NULL END)
        ON CONFLICT (sejour_id) DO NOTHING
        RETURNING id
        "#,
        id,
        tenant_id,
        etablissement_id,
        sejour_id,
        numero,
        complete,
    )
    .fetch_optional(&mut **tx)
    .await?;

    Ok(insere.is_some())
}

/// Passe une fiche à « complète » quand l'identité est saisie **après** la clé.
///
/// C'est le parcours normal du passage (FR-023, FR-028). Le rattachement **ne rouvre pas le
/// séjour et ne remet pas en cause l'attribution**.
///
/// Rend `false` sur une fiche déjà complète : c'est un rejeu, pas une erreur.
pub async fn completer(
    tx: &mut sqlx::PgTransaction<'_>,
    sejour_id: Uuid,
) -> Result<bool, ErreurSejour> {
    let touchee = sqlx::query_scalar!(
        r#"
        UPDATE hebergement.fiche_police
        SET complete = true, completee_le = now()
        WHERE sejour_id = $1 AND complete = false
        RETURNING id
        "#,
        sejour_id
    )
    .fetch_optional(&mut **tx)
    .await?;

    Ok(touchee.is_some())
}

/// Lit la fiche de police d'un séjour.
pub async fn lire_par_sejour(
    tx: &mut sqlx::PgTransaction<'_>,
    sejour_id: Uuid,
) -> Result<Option<FichePolice>, ErreurSejour> {
    let ligne = sqlx::query!(
        r#"
        SELECT id, sejour_id, numero, complete, generee_le, completee_le
        FROM hebergement.fiche_police
        WHERE sejour_id = $1
        "#,
        sejour_id
    )
    .fetch_optional(&mut **tx)
    .await?;

    Ok(ligne.map(|l| FichePolice {
        id: l.id,
        sejour_id: l.sejour_id,
        numero: l.numero,
        complete: l.complete,
        generee_le: l.generee_le,
        completee_le: l.completee_le,
    }))
}
