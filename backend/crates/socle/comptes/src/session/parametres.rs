//! Les durées des jetons, **lues du catalogue, jamais écrites en constante**.
//!
//! `jeton_acces_duree_min` et `jeton_rafraichissement_duree_jours` sont des paramètres
//! d'établissement (migration `0019`). Un exploitant qui veut des sessions de quinze minutes sur
//! sa caisse et de trente jours sur le poste du gérant les règle ; deux constantes Rust en
//! feraient deux demandes d'évolution.
//!
//! # Le repli n'est pas un défaut caché
//!
//! Quand aucun niveau de la chaîne d'héritage ne porte la clé, la lecture rend `None` — jamais une
//! valeur par défaut (`configuration/repository.rs` le dit explicitement : *un défaut rendu ici
//! serait un paramètre en dur déguisé en commodité*). Le repli est donc **ici**, chez l'appelant,
//! où on peut le voir — et il vaut exactement le défaut documenté du catalogue.

use kaya_etablissements::configuration::repository as configuration;
use uuid::Uuid;

use super::modele::{
    ACCES_DUREE_MIN_DEFAUT, CLE_ACCES_DUREE, CLE_RAFRAICHISSEMENT_DUREE, ErreurSession,
    RAFRAICHISSEMENT_DUREE_JOURS_DEFAUT,
};

/// Les deux durées applicables à une session, en secondes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DureesSession {
    pub acces_s: i64,
    pub rafraichissement_s: i64,
}

impl DureesSession {
    /// Les valeurs de repli, égales aux défauts du catalogue.
    pub fn repli() -> Self {
        Self {
            acces_s: ACCES_DUREE_MIN_DEFAUT * 60,
            rafraichissement_s: RAFRAICHISSEMENT_DUREE_JOURS_DEFAUT * 24 * 3600,
        }
    }
}

/// Résout les deux durées pour un établissement.
///
/// `etablissement_id` peut être `None` — c'est le cas d'`admin_editeur`, qui n'est rattaché à
/// aucun établissement. La descente se fait alors au seul niveau tenant.
///
/// **La transaction est prise en paramètre**, comme partout : c'est le service qui décide de la
/// portée, et l'ouverture de session lit ces durées dans la même transaction que le reste.
pub async fn resoudre(
    tx: &mut sqlx::PgTransaction<'_>,
    etablissement_id: Option<Uuid>,
) -> Result<DureesSession, ErreurSession> {
    let acces_min = lire_entier(tx, etablissement_id, CLE_ACCES_DUREE)
        .await?
        .unwrap_or(ACCES_DUREE_MIN_DEFAUT);

    let rafraichissement_jours = lire_entier(tx, etablissement_id, CLE_RAFRAICHISSEMENT_DUREE)
        .await?
        .unwrap_or(RAFRAICHISSEMENT_DUREE_JOURS_DEFAUT);

    Ok(DureesSession {
        // Un paramètre à zéro ou négatif produirait un jeton déjà expiré à la délivrance, donc une
        // boucle de rafraîchissement infinie côté client. Le plancher d'une minute est une borne
        // de sûreté, pas une politique : il n'existe que pour qu'une faute de saisie ne mette pas
        // l'établissement hors service.
        acces_s: acces_min.max(1) * 60,
        rafraichissement_s: rafraichissement_jours.max(1) * 24 * 3600,
    })
}

/// Lit une clé entière du catalogue.
///
/// Une valeur présente mais non entière est traitée comme **absente** : le catalogue la déclare
/// `ENTIER`, et une valeur d'un autre type ne peut venir que d'une écriture qui a contourné la
/// validation. Faire échouer la connexion pour cela mettrait l'établissement dehors ; le repli le
/// laisse travailler, et la valeur fautive reste visible à l'écran de configuration.
async fn lire_entier(
    tx: &mut sqlx::PgTransaction<'_>,
    etablissement_id: Option<Uuid>,
    cle: &str,
) -> Result<Option<i64>, ErreurSession> {
    let resolue = configuration::resoudre(tx, etablissement_id, None, None, cle)
        .await
        .map_err(|e| {
            // La lecture de configuration a ses propres modes d'échec ; les aplatir sur
            // `ErreurSession::Base` perdrait le contexte dans les journaux.
            tracing::warn!(erreur = %e, cle, "lecture du paramètre de durée de session impossible");
            sqlx::Error::Protocol(format!("lecture du paramètre « {cle} » : {e}"))
        })?;

    Ok(resolue.and_then(|v| v.valeur.as_i64()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Le repli du code **vaut les défauts documentés du catalogue**.
    ///
    /// Un repli différent ne se manifesterait qu'en cas de panne de lecture de configuration —
    /// c'est-à-dire jamais en test, et une fois en production.
    #[test]
    fn le_repli_vaut_les_defauts_du_catalogue() {
        let repli = DureesSession::repli();
        assert_eq!(repli.acces_s, 60 * 60, "60 minutes");
        assert_eq!(repli.rafraichissement_s, 90 * 24 * 3600, "90 jours");
    }
}
