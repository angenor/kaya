//! Accès aux données du **constat de taxe** — figé, et immuable par privilège.
//!
//! # ★ Il n'y a ni `modifier`, ni `supprimer`, et ce n'est pas un oubli
//!
//! `taxe_sejour_constat` reçoit `GRANT SELECT, INSERT` **seuls**. Écrire une fonction de
//! modification ici échouerait à l'exécution — ce qui est déjà tard. **L'absence dit la règle
//! avant.**
//!
//! C'est ce qui transforme SC-007 — « l'assiette est immuable après le départ » — d'une promesse
//! en une propriété de la base : une relecture ne peut pas la recalculer, le rôle applicatif n'en
//! a pas le droit.

use time::OffsetDateTime;
use uuid::Uuid;

use crate::erreurs::ErreurSejour;

/// Ce que le départ fige — **des faits et un paramétrage recopié, aucun montant**.
pub struct ConstatAEcrire {
    pub id: Uuid,
    pub etablissement_id: Uuid,
    pub sejour_id: Uuid,
    /// **Arithmétique** : nombre de nuits calendaires. **Zéro pour un passage**, et c'est juste.
    pub nuits_constatees: i32,
    /// **Indicatif** depuis B-10 : la taxe est due par séjour, jamais par personne.
    pub nombre_personnes: i32,
    pub periode_debut: OffsetDateTime,
    pub periode_fin: OffsetDateTime,
    /// Le paramétrage, **RECOPIÉ** — c'est ce qui rend le figeage vrai.
    pub formule_id: Uuid,
    pub famille_formule: String,
    pub assujettie_taxe_nuitee: bool,
    pub regle_conversion_taxe: Option<String>,
    pub classement_etablissement: String,
    pub commune: String,
}

/// Le constat tel qu'il est en base.
pub struct Constat {
    pub sejour_id: Uuid,
    pub nuits_constatees: i32,
    pub nombre_personnes: i32,
    pub assujettie_taxe_nuitee: bool,
    pub regle_conversion_taxe: Option<String>,
    pub classement_etablissement: String,
    pub commune: String,
    pub fige_le: OffsetDateTime,
    /// ⚠️ **Toujours `None` à ce cycle.** Le montant vient de FIS-03, tranche T3. Le rendre à
    /// `null` — et non à zéro, ni absent — dit ce qui est vrai : **il n'est pas encore
    /// déterminé**. Zéro laisserait croire que la taxe est nulle ; absent, qu'elle n'existe pas.
    pub nuitees_assujetties: Option<i32>,
    pub montant_mineur: Option<i64>,
    pub devise: Option<String>,
}

/// Écrit le constat. **`INSERT` seul** — le privilège rend toute modification impossible.
///
/// `ON CONFLICT (sejour_id) DO NOTHING` : un rejeu du départ ne réécrit pas le constat. C'est la
/// propriété qui compte — deux départs du même séjour, l'un rejoué après une coupure, doivent
/// laisser **un** constat, celui du premier.
pub async fn figer(
    tx: &mut sqlx::PgTransaction<'_>,
    tenant_id: Uuid,
    constat: &ConstatAEcrire,
) -> Result<bool, ErreurSejour> {
    let insere = sqlx::query_scalar!(
        r#"
        INSERT INTO hebergement.taxe_sejour_constat
            (id, tenant_id, etablissement_id, sejour_id,
             nuits_constatees, nombre_personnes, periode_debut, periode_fin,
             formule_id, famille_formule, assujettie_taxe_nuitee, regle_conversion_taxe,
             classement_etablissement, commune)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
        ON CONFLICT (sejour_id) DO NOTHING
        RETURNING id
        "#,
        constat.id,
        tenant_id,
        constat.etablissement_id,
        constat.sejour_id,
        constat.nuits_constatees,
        constat.nombre_personnes,
        constat.periode_debut,
        constat.periode_fin,
        constat.formule_id,
        constat.famille_formule,
        constat.assujettie_taxe_nuitee,
        constat.regle_conversion_taxe,
        constat.classement_etablissement,
        constat.commune,
    )
    .fetch_optional(&mut **tx)
    .await?;

    Ok(insere.is_some())
}

/// Lit le constat figé d'un séjour. `None` tant que le séjour est ouvert.
pub async fn lire(
    tx: &mut sqlx::PgTransaction<'_>,
    sejour_id: Uuid,
) -> Result<Option<Constat>, ErreurSejour> {
    let ligne = sqlx::query!(
        r#"
        SELECT sejour_id, nuits_constatees, nombre_personnes,
               assujettie_taxe_nuitee, regle_conversion_taxe,
               classement_etablissement, commune, fige_le,
               nuitees_assujetties, montant_mineur, devise
        FROM hebergement.taxe_sejour_constat
        WHERE sejour_id = $1
        "#,
        sejour_id
    )
    .fetch_optional(&mut **tx)
    .await?;

    Ok(ligne.map(|l| Constat {
        sejour_id: l.sejour_id,
        nuits_constatees: l.nuits_constatees,
        nombre_personnes: l.nombre_personnes,
        assujettie_taxe_nuitee: l.assujettie_taxe_nuitee,
        regle_conversion_taxe: l.regle_conversion_taxe,
        classement_etablissement: l.classement_etablissement,
        commune: l.commune,
        fige_le: l.fige_le,
        nuitees_assujetties: l.nuitees_assujetties,
        montant_mineur: l.montant_mineur,
        devise: l.devise,
    }))
}

/// Le nombre de **nuits calendaires** d'une période — **arithmétique pure**.
///
/// ═══════════════════════════════════════════════════════════════════════════════════════════════
///  ★ C'EST LA FRONTIÈRE DU PRINCIPE V, ET ELLE EST ICI
///
///  **Compter les nuits d'un intervalle est de l'arithmétique.** Décider **lesquelles sont
///  assujetties** est une règle fiscale : `une_nuitee_par_occupation` réduit trois nuits à une,
///  et cet arbitrage ne vit que dans `JurisdictionAdapter` (porte P-12).
///
///  Cette fonction rend **trois**. Elle ne rend jamais **un**.
/// ═══════════════════════════════════════════════════════════════════════════════════════════════
///
/// # La règle de comptage, et pourquoi elle est celle-là
///
/// Une nuit est comptée dès qu'une **frontière de jour calendaire** est franchie. Deux heures
/// l'après-midi n'en franchissent aucune : **zéro nuit**, et c'est juste — un passage n'est pas
/// une nuitée. Dix heures à cheval sur minuit en franchissent une : **une nuit**.
///
/// Le calcul porte sur des **jours calendaires et non sur des tranches de 24 h** : un client
/// arrivé à 22 h et parti à 8 h le lendemain a dormi une nuit, pas « zéro virgule quatre ».
pub fn nuits_calendaires(debut: OffsetDateTime, fin: OffsetDateTime) -> i32 {
    if fin <= debut {
        return 0;
    }
    // La différence de **dates**, pas de durées : c'est ce qui compte les frontières de jour.
    let jours = (fin.date() - debut.date()).whole_days();
    i32::try_from(jours.max(0)).unwrap_or(i32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::datetime;

    /// **Un passage de deux heures produit ZÉRO nuit**, et c'est juste.
    #[test]
    fn un_passage_ne_produit_aucune_nuit() {
        assert_eq!(
            nuits_calendaires(datetime!(2026-08-03 14:00 UTC), datetime!(2026-08-03 16:00 UTC)),
            0
        );
    }

    /// **Une nuit à cheval sur minuit en produit UNE**, même si elle ne dure que dix heures.
    ///
    /// Le comptage porte sur les frontières de jour calendaire, pas sur des tranches de 24 h.
    #[test]
    fn une_nuit_a_cheval_sur_minuit_en_produit_une() {
        assert_eq!(
            nuits_calendaires(datetime!(2026-08-03 22:00 UTC), datetime!(2026-08-04 08:00 UTC)),
            1
        );
    }

    /// ★ **Trois nuits en produisent TROIS. La fonction ne réduit rien.**
    ///
    /// C'est l'assertion qui garde la frontière du principe V : `une_nuitee_par_occupation`
    /// réduirait ces trois à une, et c'est un arbitrage **fiscal** que cette fonction ne connaît
    /// pas et ne doit jamais connaître.
    #[test]
    fn trois_nuits_en_produisent_trois_et_la_fonction_ne_reduit_rien() {
        assert_eq!(
            nuits_calendaires(datetime!(2026-08-03 14:00 UTC), datetime!(2026-08-06 11:00 UTC)),
            3,
            "cette fonction rend TROIS. Réduire à un serait appliquer `une_nuitee_par_occupation` \
             — une RÈGLE FISCALE, qui ne vit que dans JurisdictionAdapter (P-12)."
        );
    }

    /// Un intervalle vide ou inversé rend zéro plutôt que de paniquer.
    #[test]
    fn un_intervalle_vide_ou_inverse_rend_zero() {
        assert_eq!(
            nuits_calendaires(datetime!(2026-08-03 14:00 UTC), datetime!(2026-08-03 14:00 UTC)),
            0
        );
        assert_eq!(
            nuits_calendaires(datetime!(2026-08-06 14:00 UTC), datetime!(2026-08-03 14:00 UTC)),
            0
        );
    }
}
