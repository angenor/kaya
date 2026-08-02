//! Traduction des erreurs de base en refus métier — **écrite une seule fois**.
//!
//! # Ce que sqlx 0.9 apporte, et ce qu'il n'apporte pas
//!
//! Le choix de sqlx `0.9.0` tient pour moitié à `#3918`, qui ajoute
//! [`sqlx::error::ErrorKind::ExclusionViolation`]. Le cycle 001 a écrit qu'il « reste à vérifier
//! avant HEB-02 » — la vérification est faite, et **l'apport est partiel** :
//!
//! | Ce qui existe en 0.9.0 | Ce qui n'existe pas |
//! |---|---|
//! | `ErrorKind::ExclusionViolation` | `DatabaseError::is_exclusion_violation()` |
//!
//! Le trait `DatabaseError` porte `is_unique_violation()`, `is_foreign_key_violation()` et
//! `is_check_violation()` — **et s'arrête là**. Vérifié dans les sources de `sqlx-core` 0.9.0,
//! `src/error.rs` : les trois accesseurs y sont, le quatrième n'y est pas. Écrire la forme
//! symétrique par analogie ne compile pas, et c'est le genre d'erreur qui coûte une demi-heure
//! parce qu'on y cherche une faute de frappe.
//!
//! `ErrorKind` est `#[non_exhaustive]` : `matches!` est la forme correcte. Un `match` exhaustif ne
//! compilerait pas davantage, et un `match` avec bras `_` compilerait en cessant de signaler
//! l'arrivée d'un genre nouveau.

/// Nom de la contrainte d'exclusion qui protège `hebergement.occupation`.
///
/// Écrit ici, une fois. Le recopier dans le service et dans les tests produirait, le jour où
/// quelqu'un renomme la contrainte, une traduction qui ne reconnaît plus rien — donc un `500` au
/// lieu d'un `409`, sur le chemin le plus important du produit.
pub const CONTRAINTE_SANS_CHEVAUCHEMENT: &str = "occupation_sans_chevauchement";

/// Cette erreur est-elle la violation de **cette** contrainte d'exclusion ?
///
/// # Le nom de contrainte est vérifié, pas seulement le genre d'erreur
///
/// Une table qui gagnerait une seconde contrainte d'exclusion ferait autrement passer ses
/// violations pour des doubles attributions — et l'écran afficherait « Cette chambre est déjà
/// prise sur cette période » pour un refus qui n'a rien à voir.
///
/// Aujourd'hui `hebergement.occupation` n'en porte qu'une, et
/// `backend/tests/hebergement_disponibilite.rs` le vérifie : l'ajout d'une seconde devient visible
/// plutôt que silencieux.
pub fn est_violation_exclusion(erreur: &sqlx::Error, contrainte: &str) -> bool {
    matches!(erreur, sqlx::Error::Database(e)
        if matches!(e.kind(), sqlx::error::ErrorKind::ExclusionViolation)
            && e.constraint() == Some(contrainte))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Une erreur de base **fabriquée**, pour exercer la traduction sans base.
    ///
    /// Le test d'intégration `deux_attributions_concurrentes_une_seule_reussit` exerce le vrai
    /// chemin, sur un vrai PostgreSQL. Celui-ci exerce la **fonction**, y compris ses cas de
    /// refus — un genre voisin, une autre contrainte —, que la base ne produit pas facilement à
    /// la demande.
    #[derive(Debug)]
    struct ErreurFictive {
        genre: sqlx::error::ErrorKind,
        contrainte: Option<String>,
    }

    impl std::fmt::Display for ErreurFictive {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "erreur fictive")
        }
    }

    impl std::error::Error for ErreurFictive {}

    impl sqlx::error::DatabaseError for ErreurFictive {
        fn message(&self) -> &str {
            "erreur fictive"
        }

        fn as_error(&self) -> &(dyn std::error::Error + Send + Sync + 'static) {
            self
        }

        fn as_error_mut(&mut self) -> &mut (dyn std::error::Error + Send + Sync + 'static) {
            self
        }

        fn into_error(self: Box<Self>) -> Box<dyn std::error::Error + Send + Sync + 'static> {
            self
        }

        fn constraint(&self) -> Option<&str> {
            self.contrainte.as_deref()
        }

        // `ErrorKind` n'est ni `Copy` ni `Clone` — il est `#[non_exhaustive]`, et le reconstruire
        // par `match` est la seule forme qui reste juste quand un genre s'y ajoute.
        fn kind(&self) -> sqlx::error::ErrorKind {
            match self.genre {
                sqlx::error::ErrorKind::ExclusionViolation => {
                    sqlx::error::ErrorKind::ExclusionViolation
                }
                sqlx::error::ErrorKind::UniqueViolation => sqlx::error::ErrorKind::UniqueViolation,
                sqlx::error::ErrorKind::CheckViolation => sqlx::error::ErrorKind::CheckViolation,
                _ => sqlx::error::ErrorKind::Other,
            }
        }
    }

    fn erreur(genre: sqlx::error::ErrorKind, contrainte: Option<&str>) -> sqlx::Error {
        sqlx::Error::Database(Box::new(ErreurFictive {
            genre,
            contrainte: contrainte.map(str::to_owned),
        }))
    }

    #[test]
    fn la_violation_de_la_bonne_contrainte_est_reconnue() {
        let e = erreur(
            sqlx::error::ErrorKind::ExclusionViolation,
            Some(CONTRAINTE_SANS_CHEVAUCHEMENT),
        );
        assert!(est_violation_exclusion(&e, CONTRAINTE_SANS_CHEVAUCHEMENT));
    }

    /// **Le test qui justifie la vérification du nom.** Une seconde contrainte d'exclusion sur la
    /// même table ne doit pas faire passer ses violations pour des doubles attributions.
    #[test]
    fn une_autre_contrainte_d_exclusion_n_est_pas_une_double_attribution() {
        let e = erreur(
            sqlx::error::ErrorKind::ExclusionViolation,
            Some("une_autre_contrainte"),
        );
        assert!(!est_violation_exclusion(&e, CONTRAINTE_SANS_CHEVAUCHEMENT));
    }

    /// Une violation d'unicité sur une contrainte **portant le même nom** — cas impossible en
    /// base, écrit pour que le genre soit vérifié et pas seulement le nom.
    #[test]
    fn un_autre_genre_d_erreur_n_est_pas_une_violation_d_exclusion() {
        let e = erreur(
            sqlx::error::ErrorKind::UniqueViolation,
            Some(CONTRAINTE_SANS_CHEVAUCHEMENT),
        );
        assert!(!est_violation_exclusion(&e, CONTRAINTE_SANS_CHEVAUCHEMENT));
    }

    #[test]
    fn une_erreur_sans_contrainte_nommee_n_en_est_pas_une() {
        let e = erreur(sqlx::error::ErrorKind::ExclusionViolation, None);
        assert!(!est_violation_exclusion(&e, CONTRAINTE_SANS_CHEVAUCHEMENT));
    }

    /// Une erreur qui n'est pas une erreur de base du tout — délai dépassé, connexion coupée.
    #[test]
    fn une_erreur_hors_base_n_en_est_pas_une() {
        assert!(!est_violation_exclusion(
            &sqlx::Error::PoolTimedOut,
            CONTRAINTE_SANS_CHEVAUCHEMENT
        ));
    }
}
