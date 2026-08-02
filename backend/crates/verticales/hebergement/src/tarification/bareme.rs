//! Le barème du passage — **une fonction pure, en arithmétique entière**.
//!
//! # Aucun flottant, nulle part
//!
//! Les montants sont des **entiers d'unité mineure** (principe V, porte P-10). Un `f64` qui
//! traverserait ce calcul produirait `6199.999999999999` sur une addition parfaitement banale, et
//! l'arrondi se ferait à l'affichage — c'est-à-dire nulle part de vérifiable.
//!
//! # L'ordre des quatre étapes, et l'inversion qui coûterait cher
//!
//! 1. la durée réelle, en minutes ;
//! 2. **si la durée atteint le seuil → bascule en `NUITEE`, fin du calcul** ;
//! 3. le premier palier dont la durée est ≥ la durée réelle ;
//! 4. sinon, le dernier palier + `ceil((durée − durée du dernier palier) / 1 h)` heures
//!    supplémentaires — **toute heure entamée est due**.
//!
//! **Le point 2 précède le 3, et l'inverser produirait un empilement d'heures là où la nuitée
//! s'applique.** Un client resté neuf heures paierait quatre heures de palier plus cinq heures
//! supplémentaires, au lieu d'une nuit — un montant plus élevé que le tarif affiché, sur le
//! chemin le plus fréquent du produit.
//!
//! La bascule n'est **pas un palier majoré** : c'est un **changement de formule**. C'est pourquoi
//! la décision porte `palier_retenu_minutes: None` dans ce cas.

use crate::referentiel::FamilleFormule;

/// Un palier, réduit à ce que le calcul en consomme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palier {
    pub duree_minutes: i32,
    /// **Entier d'unité mineure** (P-10).
    pub prix_mineur: i64,
}

/// Ce que le barème décide — sans devise ni horodatage, que le service ajoute.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Calcul {
    pub formule_appliquee: FamilleFormule,
    /// `None` quand la durée a fait basculer en nuitée : il n'y a pas de palier.
    pub palier_retenu_minutes: Option<i32>,
    pub heures_supplementaires: i32,
    pub montant_du_mineur: i64,
}

/// Le barème est-il exploitable ?
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ErreurBareme {
    /// FR-025 — un passage sans palier ne sait rien facturer. Le service du référentiel le refuse
    /// à la création ; le rencontrer ici signale une donnée écrite hors du produit.
    #[error("bareme_absent")]
    BaremeAbsent,
}

/// Calcule le montant dû pour une durée réelle.
///
/// # Paramètres
///
/// - `duree_minutes` — la durée **constatée**, depuis l'horodatage d'autorité serveur. Jamais
///   l'horloge d'un terminal (FR-029) : le service la calcule, cette fonction ne fait que
///   l'appliquer, et c'est ce qui la rend testable sans base.
/// - `paliers` — **triés par durée croissante**. La clé primaire `(formule_id, duree_minutes)`
///   rend l'ordre total en base, et la lecture trie ; cette fonction retrie par sécurité, parce
///   qu'un appelant futur pourrait construire la liste autrement.
/// - `prix_heure_supplementaire_mineur` — `None` = pas d'heure supplémentaire possible, et la
///   durée est alors plafonnée au dernier palier.
/// - `seuil_bascule_minutes` — paramètre d'établissement (`seuil_bascule_nuitee_minutes`),
///   **jamais une constante**. `None` = aucune bascule.
/// - `prix_nuitee_mineur` — le prix de la nuitée de la même catégorie, pour le cas de bascule.
pub fn calculer(
    duree_minutes: i64,
    paliers: &[Palier],
    prix_heure_supplementaire_mineur: Option<i64>,
    seuil_bascule_minutes: Option<i32>,
    prix_nuitee_mineur: Option<i64>,
) -> Result<Calcul, ErreurBareme> {
    if paliers.is_empty() {
        return Err(ErreurBareme::BaremeAbsent);
    }

    // ── 2 · LA BASCULE, AVANT TOUT PALIER ─────────────────────────────────────────────────────
    //
    // Ce n'est pas un palier majoré : c'est un changement de formule. Le placer après la recherche
    // de palier ferait payer quatre heures plus cinq heures supplémentaires à qui reste neuf
    // heures — plus cher que la nuit affichée.
    if let (Some(seuil), Some(prix_nuitee)) = (seuil_bascule_minutes, prix_nuitee_mineur)
        && duree_minutes >= i64::from(seuil)
    {
        return Ok(Calcul {
            formule_appliquee: FamilleFormule::Nuitee,
            palier_retenu_minutes: None,
            heures_supplementaires: 0,
            montant_du_mineur: prix_nuitee,
        });
    }

    let mut tries = paliers.to_vec();
    tries.sort_by_key(|p| p.duree_minutes);

    // ── 3 · le premier palier qui couvre la durée ─────────────────────────────────────────────
    //
    // **Le premier palier est dû en entier.** Vingt minutes coûtent l'heure : il n'y a pas de
    // tarification en dessous, et l'exploitant ne loue pas la chambre à la minute.
    if let Some(palier) = tries.iter().find(|p| i64::from(p.duree_minutes) >= duree_minutes) {
        return Ok(Calcul {
            formule_appliquee: FamilleFormule::Passage,
            palier_retenu_minutes: Some(palier.duree_minutes),
            heures_supplementaires: 0,
            montant_du_mineur: palier.prix_mineur,
        });
    }

    // ── 4 · au-delà du dernier palier : les heures supplémentaires ────────────────────────────
    let dernier = *tries.last().expect("la liste est non vide, vérifié en tête");

    let Some(prix_heure) = prix_heure_supplementaire_mineur else {
        // Sans prix d'heure supplémentaire, le dernier palier plafonne. C'est une donnée
        // d'exploitation possible — « au-delà de 4 h, c'est 5 000 F quoi qu'il arrive » — et non
        // un cas d'erreur.
        return Ok(Calcul {
            formule_appliquee: FamilleFormule::Passage,
            palier_retenu_minutes: Some(dernier.duree_minutes),
            heures_supplementaires: 0,
            montant_du_mineur: dernier.prix_mineur,
        });
    };

    // **Toute heure entamée est due.** Le plafond entier — `(reste + 59) / 60` — plutôt qu'une
    // division flottante suivie de `ceil()` : le résultat est le même et il n'y a pas d'arrondi
    // à décider.
    let reste = duree_minutes - i64::from(dernier.duree_minutes);
    let heures = (reste + 59) / 60;

    Ok(Calcul {
        formule_appliquee: FamilleFormule::Passage,
        palier_retenu_minutes: Some(dernier.duree_minutes),
        heures_supplementaires: i32::try_from(heures).unwrap_or(i32::MAX),
        montant_du_mineur: dernier.prix_mineur + heures * prix_heure,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Le barème de Deloria — `docs/user-stories-v1.md`, récapitulatif des paramètres.
    fn deloria() -> Vec<Palier> {
        vec![
            Palier { duree_minutes: 60, prix_mineur: 1_500 },
            Palier { duree_minutes: 120, prix_mineur: 2_800 },
            Palier { duree_minutes: 180, prix_mineur: 4_000 },
            Palier { duree_minutes: 240, prix_mineur: 5_000 },
        ]
    }

    fn calculer_deloria(minutes: i64) -> Calcul {
        calculer(minutes, &deloria(), Some(1_200), Some(480), Some(12_500))
            .expect("le barème de Deloria est exploitable")
    }

    #[test]
    fn deux_heures_valent_2800() {
        let calcul = calculer_deloria(120);
        assert_eq!(calcul.montant_du_mineur, 2_800);
        assert_eq!(calcul.palier_retenu_minutes, Some(120));
        assert_eq!(calcul.heures_supplementaires, 0);
    }

    /// **4 h 10 → 6 200** = 5 000 (dernier palier) + 1 × 1 200. Dix minutes entamées coûtent
    /// l'heure : c'est la règle, et c'est ce que l'exploitant annonce au client.
    #[test]
    fn quatre_heures_dix_valent_6200() {
        let calcul = calculer_deloria(250);
        assert_eq!(calcul.montant_du_mineur, 6_200);
        assert_eq!(calcul.palier_retenu_minutes, Some(240));
        assert_eq!(calcul.heures_supplementaires, 1);
    }

    /// **20 min → 1 500.** Le premier palier est dû en entier ; il n'y a pas de tarification en
    /// dessous.
    #[test]
    fn vingt_minutes_valent_le_premier_palier() {
        let calcul = calculer_deloria(20);
        assert_eq!(calcul.montant_du_mineur, 1_500);
        assert_eq!(calcul.palier_retenu_minutes, Some(60));
    }

    /// **8 h → bascule en NUITÉE.** Pas quatre heures plus quatre heures supplémentaires : un
    /// changement de formule.
    #[test]
    fn huit_heures_basculent_en_nuitee() {
        let calcul = calculer_deloria(480);
        assert_eq!(calcul.formule_appliquee, FamilleFormule::Nuitee);
        assert_eq!(calcul.montant_du_mineur, 12_500);
        assert_eq!(
            calcul.palier_retenu_minutes, None,
            "une bascule n'a pas de palier : ce n'est pas un palier majoré"
        );
    }

    /// **La démonstration que l'ordre compte.** Sans la bascule, 8 h coûteraient 5 000 + 4 × 1 200
    /// = 9 800 — soit moins que la nuit ici, mais le rapport s'inverse dès neuf heures, et surtout
    /// le client paierait un passage là où il a dormi.
    #[test]
    fn sans_seuil_de_bascule_les_heures_s_empilent() {
        let calcul = calculer(480, &deloria(), Some(1_200), None, Some(12_500))
            .expect("barème exploitable");
        assert_eq!(calcul.formule_appliquee, FamilleFormule::Passage);
        assert_eq!(calcul.montant_du_mineur, 5_000 + 4 * 1_200);
    }

    /// Une minute de plus que le dernier palier coûte **une heure entière**.
    #[test]
    fn toute_heure_entamee_est_due() {
        assert_eq!(calculer_deloria(241).montant_du_mineur, 6_200);
        assert_eq!(calculer_deloria(300).montant_du_mineur, 6_200);
        assert_eq!(calculer_deloria(301).montant_du_mineur, 7_400);
    }

    /// Un barème **exactement** au palier ne déclenche aucune heure supplémentaire.
    #[test]
    fn la_borne_du_palier_est_incluse() {
        assert_eq!(calculer_deloria(240).montant_du_mineur, 5_000);
        assert_eq!(calculer_deloria(240).heures_supplementaires, 0);
    }

    /// **FR-025** — un passage sans palier ne sait rien facturer.
    #[test]
    fn un_bareme_sans_palier_est_refuse() {
        assert_eq!(
            calculer(120, &[], Some(1_200), Some(480), Some(12_500)),
            Err(ErreurBareme::BaremeAbsent)
        );
    }

    /// **Les paliers sont triés avant usage.** Un appelant qui les fournirait en désordre
    /// obtiendrait sinon le mauvais palier — la base garantit l'ordre, pas un appelant futur.
    #[test]
    fn des_paliers_en_desordre_donnent_le_meme_resultat() {
        let desordre = vec![
            Palier { duree_minutes: 240, prix_mineur: 5_000 },
            Palier { duree_minutes: 60, prix_mineur: 1_500 },
            Palier { duree_minutes: 180, prix_mineur: 4_000 },
            Palier { duree_minutes: 120, prix_mineur: 2_800 },
        ];
        let calcul = calculer(120, &desordre, Some(1_200), Some(480), Some(12_500)).unwrap();
        assert_eq!(calcul.montant_du_mineur, 2_800);
    }

    /// Sans prix d'heure supplémentaire, le **dernier palier plafonne** — donnée d'exploitation
    /// possible, pas un cas d'erreur.
    #[test]
    fn sans_heure_supplementaire_le_dernier_palier_plafonne() {
        let calcul = calculer(300, &deloria(), None, None, None).expect("barème exploitable");
        assert_eq!(calcul.montant_du_mineur, 5_000);
        assert_eq!(calcul.heures_supplementaires, 0);
    }

    /// **Aucun flottant** : le calcul reste exact sur des montants qui déborderaient la mantisse
    /// d'un `f64` si quelqu'un y passait un jour.
    #[test]
    fn l_arithmetique_reste_entiere_sur_de_grands_montants() {
        let gros = vec![Palier { duree_minutes: 60, prix_mineur: 9_007_199_254_740_993 }];
        let calcul = calculer(30, &gros, None, None, None).unwrap();
        assert_eq!(calcul.montant_du_mineur, 9_007_199_254_740_993);
    }
}
