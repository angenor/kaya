//! Accès aux données de la note et de ses lignes.
//!
//! # ★ Le total ne se lit dans aucune colonne
//!
//! `note_sejour` n'en porte pas : le total est la **somme des lignes**, calculée à la lecture. Une
//! colonne totalisatrice se désynchronise en silence — et le silence est exactement ce que le
//! propriétaire achète en installant ce logiciel (cadrage §8.3). Un total faux ne se voit qu'au
//! moment où le client conteste.
//!
//! # ★ Aucune fonction de ce fichier ne modifie une ligne
//!
//! Il n'y a **pas** de `modifier_ligne`, et ce n'est pas un oubli : `ligne_sejour` n'a pas le
//! privilège `UPDATE`. Une correction est une **ligne d'ajustement** portant son motif. Écrire la
//! fonction ici échouerait à l'exécution, ce qui est déjà tard — mais l'absence dit la règle avant.

use rust_decimal::Decimal;
use time::OffsetDateTime;
use uuid::Uuid;

use super::super::sejour::modele::{LigneNote, NoteVue};
use crate::erreurs::ErreurSejour;

/// Une ligne à écrire.
pub struct NouvelleLigne {
    pub id: Uuid,
    pub occupation_id: Option<Uuid>,
    pub nature: &'static str,
    /// Renseigné **seulement** sur un ajustement, et jamais deviné.
    pub motif: Option<&'static str>,
    /// **Clé i18n**, jamais un libellé rendu (porte P-16).
    pub libelle_cle: String,
    /// ⚠️ **`NUMERIC`, jamais entier** (principe V, porte P-10).
    pub quantite: Decimal,
    /// **Entier d'unité mineure.**
    pub prix_unitaire_mineur: i64,
    /// **Entier d'unité mineure. Peut être négatif** — un départ anticipé rembourse.
    pub montant_mineur: i64,
    pub devise: String,
    pub periode_debut: Option<OffsetDateTime>,
    pub periode_fin: Option<OffsetDateTime>,
}

/// Ouvre la note d'un séjour.
pub async fn ouvrir(
    tx: &mut sqlx::PgTransaction<'_>,
    tenant_id: Uuid,
    id: Uuid,
    sejour_id: Uuid,
    devise: &str,
) -> Result<bool, ErreurSejour> {
    let insere = sqlx::query_scalar!(
        r#"
        INSERT INTO hebergement.note_sejour (id, tenant_id, sejour_id, devise)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (sejour_id) DO NOTHING
        RETURNING id
        "#,
        id,
        tenant_id,
        sejour_id,
        devise,
    )
    .fetch_optional(&mut **tx)
    .await?;

    Ok(insere.is_some())
}

/// Ajoute une ligne. **`INSERT` seul** — le privilège rend l'`UPDATE` impossible.
pub async fn ajouter_ligne(
    tx: &mut sqlx::PgTransaction<'_>,
    tenant_id: Uuid,
    note_id: Uuid,
    ligne: &NouvelleLigne,
) -> Result<(), ErreurSejour> {
    sqlx::query!(
        r#"
        INSERT INTO hebergement.ligne_sejour
            (id, tenant_id, note_id, occupation_id, nature, motif, libelle_cle,
             quantite, prix_unitaire_mineur, montant_mineur, devise,
             periode_debut, periode_fin)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
        ON CONFLICT (id) DO NOTHING
        "#,
        ligne.id,
        tenant_id,
        note_id,
        ligne.occupation_id,
        ligne.nature,
        ligne.motif,
        ligne.libelle_cle,
        ligne.quantite,
        ligne.prix_unitaire_mineur,
        ligne.montant_mineur,
        ligne.devise,
        ligne.periode_debut,
        ligne.periode_fin,
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

/// Lit la note d'un séjour **avec ses lignes et son total calculé**.
pub async fn lire_par_sejour(
    tx: &mut sqlx::PgTransaction<'_>,
    sejour_id: Uuid,
) -> Result<Option<NoteVue>, ErreurSejour> {
    let entete = sqlx::query!(
        r#"
        SELECT id, sejour_id, statut, devise, arretee_le
        FROM hebergement.note_sejour
        WHERE sejour_id = $1
        "#,
        sejour_id
    )
    .fetch_optional(&mut **tx)
    .await?;

    let Some(entete) = entete else {
        return Ok(None);
    };

    let lignes = sqlx::query!(
        r#"
        SELECT id, nature, motif, libelle_cle, quantite,
               prix_unitaire_mineur, montant_mineur, devise,
               periode_debut, periode_fin
        FROM hebergement.ligne_sejour
        WHERE note_id = $1
        ORDER BY cree_le, id
        "#,
        entete.id
    )
    .fetch_all(&mut **tx)
    .await?;

    let lignes: Vec<LigneNote> = lignes
        .into_iter()
        .map(|l| LigneNote {
            id: l.id,
            nature: l.nature,
            motif: l.motif,
            libelle_cle: l.libelle_cle,
            // ⚠️ **Rendu en chaîne décimale, jamais en `f64`.** Un flottant perdrait des chiffres
            // sur une quantité au prorata, et le principe V l'interdit jusque dans le contrat.
            quantite: l.quantite.to_string(),
            prix_unitaire_mineur: l.prix_unitaire_mineur,
            montant_mineur: l.montant_mineur,
            devise: l.devise,
            periode_debut: l.periode_debut,
            periode_fin: l.periode_fin,
        })
        .collect();

    // ★ **Le total est la somme des lignes**, calculée ici. `saturating_add` plutôt que `+` : un
    // dépassement d'entier sur un total de note produirait un montant NÉGATIF, ce qui est pire
    // qu'un plafond — et le plafond de `i64` en unité mineure représente neuf quintillions.
    let total_mineur = lignes
        .iter()
        .fold(0i64, |acc, l| acc.saturating_add(l.montant_mineur));

    Ok(Some(NoteVue {
        id: entete.id,
        sejour_id: entete.sejour_id,
        statut: entete.statut,
        devise: entete.devise,
        lignes,
        total_mineur,
        arretee_le: entete.arretee_le,
    }))
}

/// Arrête la note — **plus rien ne peut s'y ajouter**.
///
/// Terme utilisateur : « La note est arrêtée : plus rien ne peut s'y ajouter » (lexique v1.6.0).
/// Jamais « clôturée », « figée » ni « verrouillée », qui sont les mots de la ligne de code.
///
/// Rend `false` quand la note était **déjà arrêtée** — la condition en fait un refus de la base.
pub async fn arreter(
    tx: &mut sqlx::PgTransaction<'_>,
    note_id: Uuid,
) -> Result<bool, ErreurSejour> {
    let touchee = sqlx::query_scalar!(
        r#"
        UPDATE hebergement.note_sejour
        SET statut = 'arretee', arretee_le = now(), modifie_le = now()
        WHERE id = $1 AND statut = 'ouverte'
        RETURNING id
        "#,
        note_id
    )
    .fetch_optional(&mut **tx)
    .await?;

    Ok(touchee.is_some())
}
