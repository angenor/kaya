//! Couche d'accès aux données du référentiel — **écrite à la main contre sqlx 0.9.0**.
//!
//! Les points que le module doré fixe, et qui sont tenus ici :
//!
//! - toutes les requêtes passent par les **macros `query!` sur littéral**, donc vérifiées à la
//!   compilation contre la vraie base (porte P-18) ; `AssertSqlSafe` n'apparaît nulle part ;
//! - le repository **prend la transaction**, il ne l'ouvre jamais — c'est le service qui décide de
//!   la portée, parce que c'est lui qui doit y inclure l'événement outbox ;
//! - **aucune jointure entre schémas de modules** (porte P-04). Le fuseau et la devise viennent
//!   d'`EstablishmentDirectory`, jamais d'un `JOIN` vers `etablissements`.
//!
//! # Les enfants se remplacent, ils ne se corrigent pas
//!
//! Paliers, plages et temps de remise en état sont **remplacés en bloc** : `DELETE` puis `INSERT`,
//! dans la transaction du parent. Un correctif ligne à ligne demanderait au client d'envoyer des
//! identifiants de palier, donc de connaître un état intermédiaire — et deux clients qui règlent
//! le même barème produiraient un mélange des deux plutôt que le dernier des deux.

use uuid::Uuid;

use super::modele::{
    CategorieVue, CreerCategorie, CreerFormule, CreerUnite, ErreurReferentiel, FamilleFormule,
    FormuleVue, ModifierCategorie, ModifierFormule, ModifierUnite, PalierVue, PlageDemandee,
    PlageVue, RegleConversionTaxe, StatutMenage, TempsRemiseEnEtat, heure_depuis_texte,
    heure_en_texte,
};

// =================================================================================================
//  1. Catégories
// =================================================================================================

/// Les catégories d'un établissement, **avec leurs battements**, en deux requêtes.
///
/// Deux requêtes plutôt qu'une jointure agrégée : la seconde est petite, et l'assemblage en Rust
/// se relit. Une requête unique avec `json_agg` produirait une colonne dont sqlx ne sait rien dire
/// à la compilation — donc un type deviné, ce que la porte P-18 existe pour éviter.
pub async fn lister_categories(
    tx: &mut sqlx::PgTransaction<'_>,
    etablissement_id: Uuid,
) -> Result<Vec<CategorieVue>, ErreurReferentiel> {
    let lignes = sqlx::query!(
        r#"
        SELECT id, etablissement_id, nom, capacite_accueil
        FROM hebergement.categorie
        WHERE etablissement_id = $1
        ORDER BY nom
        "#,
        etablissement_id
    )
    .fetch_all(&mut **tx)
    .await?;

    let battements = sqlx::query!(
        r#"
        SELECT t.categorie_id, t.famille_formule, t.duree_minutes
        FROM hebergement.temps_remise_en_etat t
        JOIN hebergement.categorie c ON c.id = t.categorie_id
        WHERE c.etablissement_id = $1
        ORDER BY t.famille_formule
        "#,
        etablissement_id
    )
    .fetch_all(&mut **tx)
    .await?;

    let mut categories = Vec::with_capacity(lignes.len());
    for ligne in lignes {
        let mut temps = Vec::new();
        for b in battements.iter().filter(|b| b.categorie_id == ligne.id) {
            temps.push(TempsRemiseEnEtat {
                famille_formule: FamilleFormule::depuis_code(&b.famille_formule)?,
                duree_minutes: b.duree_minutes,
            });
        }
        categories.push(CategorieVue {
            id: ligne.id,
            etablissement_id: ligne.etablissement_id,
            nom: ligne.nom,
            capacite_accueil: ligne.capacite_accueil,
            temps_remise_en_etat: temps,
        });
    }
    Ok(categories)
}

/// Lit une catégorie **dans le tenant courant**, avec ses battements.
pub async fn lire_categorie(
    tx: &mut sqlx::PgTransaction<'_>,
    id: Uuid,
) -> Result<Option<CategorieVue>, ErreurReferentiel> {
    let Some(ligne) = sqlx::query!(
        r#"
        SELECT id, etablissement_id, nom, capacite_accueil
        FROM hebergement.categorie
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(&mut **tx)
    .await?
    else {
        return Ok(None);
    };

    let battements = sqlx::query!(
        r#"
        SELECT famille_formule, duree_minutes
        FROM hebergement.temps_remise_en_etat
        WHERE categorie_id = $1
        ORDER BY famille_formule
        "#,
        id
    )
    .fetch_all(&mut **tx)
    .await?;

    let mut temps = Vec::with_capacity(battements.len());
    for b in battements {
        temps.push(TempsRemiseEnEtat {
            famille_formule: FamilleFormule::depuis_code(&b.famille_formule)?,
            duree_minutes: b.duree_minutes,
        });
    }

    Ok(Some(CategorieVue {
        id: ligne.id,
        etablissement_id: ligne.etablissement_id,
        nom: ligne.nom,
        capacite_accueil: ligne.capacite_accueil,
        temps_remise_en_etat: temps,
    }))
}

/// Insère une catégorie, ou constate qu'elle existe déjà.
///
/// `ON CONFLICT (id) DO NOTHING ... RETURNING` renvoie une ligne quand l'insertion a eu lieu, et
/// **rien** en cas de conflit : exactement ce qu'il faut pour distinguer `201` de `200` sans
/// second aller-retour dans le cas normal.
pub async fn inserer_categorie(
    tx: &mut sqlx::PgTransaction<'_>,
    tenant_id: Uuid,
    demande: &CreerCategorie,
) -> Result<bool, ErreurReferentiel> {
    let insere = sqlx::query!(
        r#"
        INSERT INTO hebergement.categorie
            (id, tenant_id, etablissement_id, nom, capacite_accueil)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (id) DO NOTHING
        RETURNING id
        "#,
        demande.id,
        tenant_id,
        demande.etablissement_id,
        demande.nom,
        demande.capacite_accueil,
    )
    .fetch_optional(&mut **tx)
    .await?;

    Ok(insere.is_some())
}

/// Modifie une catégorie. `modifie_le` vient de la **base**, jamais du processus applicatif.
pub async fn modifier_categorie(
    tx: &mut sqlx::PgTransaction<'_>,
    id: Uuid,
    changements: &ModifierCategorie,
) -> Result<bool, ErreurReferentiel> {
    let touchee = sqlx::query!(
        r#"
        UPDATE hebergement.categorie
        SET nom = $2, capacite_accueil = $3, modifie_le = now()
        WHERE id = $1
        RETURNING id
        "#,
        id,
        changements.nom,
        changements.capacite_accueil,
    )
    .fetch_optional(&mut **tx)
    .await?;

    Ok(touchee.is_some())
}

/// Remplace **en bloc** les battements d'une catégorie.
pub async fn remplacer_temps_remise(
    tx: &mut sqlx::PgTransaction<'_>,
    tenant_id: Uuid,
    categorie_id: Uuid,
    battements: &[TempsRemiseEnEtat],
) -> Result<(), ErreurReferentiel> {
    sqlx::query!(
        "DELETE FROM hebergement.temps_remise_en_etat WHERE categorie_id = $1",
        categorie_id
    )
    .execute(&mut **tx)
    .await?;

    for battement in battements {
        sqlx::query!(
            r#"
            INSERT INTO hebergement.temps_remise_en_etat
                (categorie_id, famille_formule, duree_minutes, tenant_id)
            VALUES ($1, $2, $3, $4)
            "#,
            categorie_id,
            battement.famille_formule.code(),
            battement.duree_minutes,
            tenant_id,
        )
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

/// Combien d'unités une catégorie porte-t-elle ?
///
/// Le refus de suppression **nomme ce qui occupe** : « 5 chambres », jamais « suppression
/// impossible ».
pub async fn compter_unites_de_categorie(
    tx: &mut sqlx::PgTransaction<'_>,
    categorie_id: Uuid,
) -> Result<i64, ErreurReferentiel> {
    let total = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "compte!" FROM hebergement.unite WHERE categorie_id = $1"#,
        categorie_id
    )
    .fetch_one(&mut **tx)
    .await?;
    Ok(total)
}

// =================================================================================================
//  2. Unités
// =================================================================================================

/// Les unités d'un établissement, **triées par catégorie puis par code**.
///
/// Le tri par code est celui de l'écran `G5` : les chambres se lisent dans l'ordre du couloir.
pub async fn lister_unites(
    tx: &mut sqlx::PgTransaction<'_>,
    etablissement_id: Uuid,
) -> Result<Vec<super::modele::UniteVue>, ErreurReferentiel> {
    let lignes = sqlx::query!(
        r#"
        SELECT u.id, u.categorie_id, u.code, u.etage, u.statut_menage
        FROM hebergement.unite u
        JOIN hebergement.categorie c ON c.id = u.categorie_id
        WHERE u.etablissement_id = $1
        ORDER BY c.nom, u.code
        "#,
        etablissement_id
    )
    .fetch_all(&mut **tx)
    .await?;

    let mut unites = Vec::with_capacity(lignes.len());
    for ligne in lignes {
        unites.push(super::modele::UniteVue {
            id: ligne.id,
            categorie_id: ligne.categorie_id,
            code: ligne.code,
            etage: ligne.etage,
            statut_menage: StatutMenage::depuis_code(&ligne.statut_menage)?,
        });
    }
    Ok(unites)
}

pub async fn lire_unite(
    tx: &mut sqlx::PgTransaction<'_>,
    id: Uuid,
) -> Result<Option<super::modele::UniteVue>, ErreurReferentiel> {
    let ligne = sqlx::query!(
        r#"
        SELECT id, categorie_id, code, etage, statut_menage
        FROM hebergement.unite
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(&mut **tx)
    .await?;

    match ligne {
        None => Ok(None),
        Some(l) => Ok(Some(super::modele::UniteVue {
            id: l.id,
            categorie_id: l.categorie_id,
            code: l.code,
            etage: l.etage,
            statut_menage: StatutMenage::depuis_code(&l.statut_menage)?,
        })),
    }
}

pub async fn inserer_unite(
    tx: &mut sqlx::PgTransaction<'_>,
    tenant_id: Uuid,
    demande: &CreerUnite,
) -> Result<bool, ErreurReferentiel> {
    let insere = sqlx::query!(
        r#"
        INSERT INTO hebergement.unite
            (id, tenant_id, etablissement_id, categorie_id, code, etage)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (id) DO NOTHING
        RETURNING id
        "#,
        demande.id,
        tenant_id,
        demande.etablissement_id,
        demande.categorie_id,
        demande.code,
        demande.etage,
    )
    .fetch_optional(&mut **tx)
    .await?;

    Ok(insere.is_some())
}

/// **Corrige `code` et `etage`, et rien d'autre.**
///
/// La requête ne touche ni `categorie_id`, ni `statut_menage` : les trois champs exclus le sont
/// **par la forme de cette requête**, pas seulement par une validation en amont. Un handler qui
/// oublierait le refus ne pourrait toujours pas les écrire par ici.
pub async fn modifier_unite(
    tx: &mut sqlx::PgTransaction<'_>,
    id: Uuid,
    changements: &ModifierUnite,
) -> Result<bool, ErreurReferentiel> {
    let touchee = sqlx::query!(
        r#"
        UPDATE hebergement.unite
        SET code = $2, etage = $3, modifie_le = now()
        WHERE id = $1
        RETURNING id
        "#,
        id,
        changements.code,
        changements.etage,
    )
    .fetch_optional(&mut **tx)
    .await?;

    Ok(touchee.is_some())
}

// =================================================================================================
//  3. Formules, avec leurs paliers et leurs plages
// =================================================================================================

/// Les formules d'un établissement, **avec leurs enfants**, en trois requêtes.
///
/// `devise` est passée par l'appelant : elle vient d'`EstablishmentDirectory`, jamais d'un `JOIN`
/// vers `etablissements` — ce serait une jointure inter-schémas (P-04).
pub async fn lister_formules(
    tx: &mut sqlx::PgTransaction<'_>,
    etablissement_id: Uuid,
    devise: &str,
) -> Result<Vec<FormuleVue>, ErreurReferentiel> {
    let lignes = sqlx::query!(
        r#"
        SELECT f.id, f.categorie_id, f.famille, f.prix_mineur,
               f.duree_min_minutes, f.duree_max_minutes,
               f.heure_arrivee_standard, f.heure_depart_standard, f.jours_autorises,
               f.assujettie_taxe_nuitee, f.regle_conversion_taxe,
               f.prix_heure_supplementaire_mineur
        FROM hebergement.formule f
        JOIN hebergement.categorie c ON c.id = f.categorie_id
        WHERE f.etablissement_id = $1
        ORDER BY c.nom, f.famille
        "#,
        etablissement_id
    )
    .fetch_all(&mut **tx)
    .await?;

    let paliers = sqlx::query!(
        r#"
        SELECT b.formule_id, b.duree_minutes, b.prix_mineur
        FROM hebergement.bareme_palier b
        JOIN hebergement.formule f ON f.id = b.formule_id
        WHERE f.etablissement_id = $1
        ORDER BY b.duree_minutes
        "#,
        etablissement_id
    )
    .fetch_all(&mut **tx)
    .await?;

    let plages = sqlx::query!(
        r#"
        SELECT p.id, p.formule_id, p.heure_debut, p.heure_fin, p.libelle_cle
        FROM hebergement.plage_demi_journee p
        JOIN hebergement.formule f ON f.id = p.formule_id
        WHERE f.etablissement_id = $1
        ORDER BY p.heure_debut
        "#,
        etablissement_id
    )
    .fetch_all(&mut **tx)
    .await?;

    let mut formules = Vec::with_capacity(lignes.len());
    for ligne in lignes {
        let regle = match ligne.regle_conversion_taxe.as_deref() {
            None => None,
            Some(code) => Some(RegleConversionTaxe::depuis_code(code)?),
        };
        formules.push(FormuleVue {
            id: ligne.id,
            categorie_id: ligne.categorie_id,
            famille: FamilleFormule::depuis_code(&ligne.famille)?,
            prix_mineur: ligne.prix_mineur,
            devise: devise.to_owned(),
            duree_min_minutes: ligne.duree_min_minutes,
            duree_max_minutes: ligne.duree_max_minutes,
            heure_arrivee_standard: ligne.heure_arrivee_standard.map(heure_en_texte),
            heure_depart_standard: ligne.heure_depart_standard.map(heure_en_texte),
            jours_autorises: ligne.jours_autorises,
            assujettie_taxe_nuitee: ligne.assujettie_taxe_nuitee,
            regle_conversion_taxe: regle,
            prix_heure_supplementaire_mineur: ligne.prix_heure_supplementaire_mineur,
            paliers: paliers
                .iter()
                .filter(|p| p.formule_id == ligne.id)
                .map(|p| PalierVue {
                    duree_minutes: p.duree_minutes,
                    prix_mineur: p.prix_mineur,
                })
                .collect(),
            plages: plages
                .iter()
                .filter(|p| p.formule_id == ligne.id)
                .map(|p| PlageVue {
                    id: p.id,
                    heure_debut: heure_en_texte(p.heure_debut),
                    heure_fin: heure_en_texte(p.heure_fin),
                    libelle_cle: p.libelle_cle.clone(),
                })
                .collect(),
        });
    }
    Ok(formules)
}

/// Lit une formule et ses enfants.
pub async fn lire_formule(
    tx: &mut sqlx::PgTransaction<'_>,
    id: Uuid,
    devise: &str,
) -> Result<Option<FormuleVue>, ErreurReferentiel> {
    let Some(ligne) = sqlx::query!(
        r#"
        SELECT id, categorie_id, famille, prix_mineur,
               duree_min_minutes, duree_max_minutes,
               heure_arrivee_standard, heure_depart_standard, jours_autorises,
               assujettie_taxe_nuitee, regle_conversion_taxe,
               prix_heure_supplementaire_mineur
        FROM hebergement.formule
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(&mut **tx)
    .await?
    else {
        return Ok(None);
    };

    let paliers = sqlx::query!(
        r#"
        SELECT duree_minutes, prix_mineur
        FROM hebergement.bareme_palier
        WHERE formule_id = $1
        ORDER BY duree_minutes
        "#,
        id
    )
    .fetch_all(&mut **tx)
    .await?;

    let plages = sqlx::query!(
        r#"
        SELECT id, heure_debut, heure_fin, libelle_cle
        FROM hebergement.plage_demi_journee
        WHERE formule_id = $1
        ORDER BY heure_debut
        "#,
        id
    )
    .fetch_all(&mut **tx)
    .await?;

    let regle = match ligne.regle_conversion_taxe.as_deref() {
        None => None,
        Some(code) => Some(RegleConversionTaxe::depuis_code(code)?),
    };

    Ok(Some(FormuleVue {
        id: ligne.id,
        categorie_id: ligne.categorie_id,
        famille: FamilleFormule::depuis_code(&ligne.famille)?,
        prix_mineur: ligne.prix_mineur,
        devise: devise.to_owned(),
        duree_min_minutes: ligne.duree_min_minutes,
        duree_max_minutes: ligne.duree_max_minutes,
        heure_arrivee_standard: ligne.heure_arrivee_standard.map(heure_en_texte),
        heure_depart_standard: ligne.heure_depart_standard.map(heure_en_texte),
        jours_autorises: ligne.jours_autorises,
        assujettie_taxe_nuitee: ligne.assujettie_taxe_nuitee,
        regle_conversion_taxe: regle,
        prix_heure_supplementaire_mineur: ligne.prix_heure_supplementaire_mineur,
        paliers: paliers
            .into_iter()
            .map(|p| PalierVue {
                duree_minutes: p.duree_minutes,
                prix_mineur: p.prix_mineur,
            })
            .collect(),
        plages: plages
            .into_iter()
            .map(|p| PlageVue {
                id: p.id,
                heure_debut: heure_en_texte(p.heure_debut),
                heure_fin: heure_en_texte(p.heure_fin),
                libelle_cle: p.libelle_cle,
            })
            .collect(),
    }))
}

/// La formule applicable à une unité, pour une famille donnée — **sans jointure inter-schémas**.
///
/// Employée par l'attribution : elle vérifie que la formule demandée appartient bien à la
/// catégorie de l'unité, refus `formule_hors_categorie`.
pub async fn formule_appartient_a_l_unite(
    tx: &mut sqlx::PgTransaction<'_>,
    formule_id: Uuid,
    unite_id: Uuid,
) -> Result<bool, ErreurReferentiel> {
    let coherent = sqlx::query_scalar!(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM hebergement.formule f
            JOIN hebergement.unite u ON u.categorie_id = f.categorie_id
            WHERE f.id = $1 AND u.id = $2
        ) AS "coherent!"
        "#,
        formule_id,
        unite_id
    )
    .fetch_one(&mut **tx)
    .await?;
    Ok(coherent)
}

pub async fn inserer_formule(
    tx: &mut sqlx::PgTransaction<'_>,
    tenant_id: Uuid,
    demande: &CreerFormule,
) -> Result<bool, ErreurReferentiel> {
    let arrivee = demande
        .heure_arrivee_standard
        .as_deref()
        .map(heure_depuis_texte)
        .transpose()?;
    let depart = demande
        .heure_depart_standard
        .as_deref()
        .map(heure_depuis_texte)
        .transpose()?;

    let insere = sqlx::query!(
        r#"
        INSERT INTO hebergement.formule
            (id, tenant_id, etablissement_id, categorie_id, famille, prix_mineur,
             duree_min_minutes, duree_max_minutes, heure_arrivee_standard,
             heure_depart_standard, jours_autorises, assujettie_taxe_nuitee,
             regle_conversion_taxe, prix_heure_supplementaire_mineur)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
        ON CONFLICT (id) DO NOTHING
        RETURNING id
        "#,
        demande.id,
        tenant_id,
        demande.etablissement_id,
        demande.categorie_id,
        demande.famille.code(),
        demande.prix_mineur,
        demande.duree_min_minutes,
        demande.duree_max_minutes,
        arrivee,
        depart,
        demande.jours_autorises.as_deref(),
        demande.assujettie_taxe_nuitee,
        demande.regle_conversion_taxe.map(|r| r.code()),
        demande.prix_heure_supplementaire_mineur,
    )
    .fetch_optional(&mut **tx)
    .await?;

    Ok(insere.is_some())
}

/// Modifie une formule — **`famille` et `categorie_id` ne bougent jamais**.
///
/// Changer la famille d'une formule reviendrait à transformer une nuitée en passage en gardant son
/// identifiant : les occupations déjà attribuées désigneraient une formule dont le sens a changé,
/// et le montant dû sur un séjour en cours changerait sous les pieds de l'exploitant.
pub async fn modifier_formule(
    tx: &mut sqlx::PgTransaction<'_>,
    id: Uuid,
    changements: &ModifierFormule,
) -> Result<bool, ErreurReferentiel> {
    let arrivee = changements
        .heure_arrivee_standard
        .as_deref()
        .map(heure_depuis_texte)
        .transpose()?;
    let depart = changements
        .heure_depart_standard
        .as_deref()
        .map(heure_depuis_texte)
        .transpose()?;

    let touchee = sqlx::query!(
        r#"
        UPDATE hebergement.formule
        SET prix_mineur = $2,
            duree_min_minutes = $3,
            duree_max_minutes = $4,
            heure_arrivee_standard = $5,
            heure_depart_standard = $6,
            jours_autorises = $7,
            assujettie_taxe_nuitee = $8,
            regle_conversion_taxe = $9,
            prix_heure_supplementaire_mineur = $10,
            modifie_le = now()
        WHERE id = $1
        RETURNING id
        "#,
        id,
        changements.prix_mineur,
        changements.duree_min_minutes,
        changements.duree_max_minutes,
        arrivee,
        depart,
        changements.jours_autorises.as_deref(),
        changements.assujettie_taxe_nuitee,
        changements.regle_conversion_taxe.map(|r| r.code()),
        changements.prix_heure_supplementaire_mineur,
    )
    .fetch_optional(&mut **tx)
    .await?;

    Ok(touchee.is_some())
}

/// Remplace **en bloc** les paliers d'une formule.
pub async fn remplacer_paliers(
    tx: &mut sqlx::PgTransaction<'_>,
    tenant_id: Uuid,
    formule_id: Uuid,
    paliers: &[PalierVue],
) -> Result<(), ErreurReferentiel> {
    sqlx::query!(
        "DELETE FROM hebergement.bareme_palier WHERE formule_id = $1",
        formule_id
    )
    .execute(&mut **tx)
    .await?;

    for palier in paliers {
        sqlx::query!(
            r#"
            INSERT INTO hebergement.bareme_palier
                (formule_id, duree_minutes, prix_mineur, tenant_id)
            VALUES ($1, $2, $3, $4)
            "#,
            formule_id,
            palier.duree_minutes,
            palier.prix_mineur,
            tenant_id,
        )
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

/// Remplace **en bloc** les plages d'une formule.
pub async fn remplacer_plages(
    tx: &mut sqlx::PgTransaction<'_>,
    tenant_id: Uuid,
    formule_id: Uuid,
    plages: &[PlageDemandee],
) -> Result<(), ErreurReferentiel> {
    sqlx::query!(
        "DELETE FROM hebergement.plage_demi_journee WHERE formule_id = $1",
        formule_id
    )
    .execute(&mut **tx)
    .await?;

    for plage in plages {
        let debut = heure_depuis_texte(&plage.heure_debut)?;
        let fin = heure_depuis_texte(&plage.heure_fin)?;
        sqlx::query!(
            r#"
            INSERT INTO hebergement.plage_demi_journee
                (id, formule_id, heure_debut, heure_fin, libelle_cle, tenant_id)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            Uuid::now_v7(),
            formule_id,
            debut,
            fin,
            plage.libelle_cle,
            tenant_id,
        )
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

/// Les plages d'une formule, **triées**, telles que la validation de demi-journée les compare.
pub async fn plages_de_formule(
    tx: &mut sqlx::PgTransaction<'_>,
    formule_id: Uuid,
) -> Result<Vec<(time::Time, time::Time)>, ErreurReferentiel> {
    let lignes = sqlx::query!(
        r#"
        SELECT heure_debut, heure_fin
        FROM hebergement.plage_demi_journee
        WHERE formule_id = $1
        ORDER BY heure_debut
        "#,
        formule_id
    )
    .fetch_all(&mut **tx)
    .await?;

    Ok(lignes
        .into_iter()
        .map(|l| (l.heure_debut, l.heure_fin))
        .collect())
}
