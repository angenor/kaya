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

/// Code SQLSTATE de l'interblocage — `40P01`.
///
/// Nommé plutôt que littéral : un code SQLSTATE écrit en dur dans une condition se retrouve un
/// jour comparé à `"40001"` (échec de sérialisation), qui est un **autre** phénomène.
pub const SQLSTATE_INTERBLOCAGE: &str = "40P01";

/// Cette erreur est-elle un **interblocage** ?
///
/// ═══════════════════════════════════════════════════════════════════════════════════════════════
///  ★ DÉFAUT TROUVÉ AU CYCLE 006, ET INVISIBLE AVANT LUI
///
///  Deux arrivées concurrentes sur la **même chambre** ne produisent pas toujours une violation
///  d'exclusion propre : elles peuvent **s'interbloquer**, et PostgreSQL en abat une avec `40P01`.
///
///  ```text
///  Process A waits for ShareLock on transaction B
///  Process B waits for ShareLock on speculative token of transaction A
///  … while checking exclusion constraint on relation "occupation"
///  ```
///
///  La cause est l'**insertion spéculative** — `INSERT … ON CONFLICT (id) DO NOTHING` — combinée à
///  une contrainte d'exclusion. Chaque transaction pose son tuple spéculatif, puis vérifie
///  l'exclusion contre celui de l'autre, et chacune attend l'autre.
/// ═══════════════════════════════════════════════════════════════════════════════════════════════
///
/// # Pourquoi le cycle 004 ne l'a pas vu
///
/// Son test de concurrence attribue par **SQL direct**, sans `ON CONFLICT` : il n'y a pas
/// d'insertion spéculative, donc pas de jeton spéculatif à attendre. Le phénomène n'apparaît que
/// sur le chemin **idempotent**, celui que le parcours de séjour emploie — et c'est exactement ce
/// que la constitution appelle re-exercer une porte par le parcours réel plutôt que par
/// l'endpoint nu.
///
/// # Ce qu'un interblocage veut dire, et ce qu'il ne veut pas dire
///
/// **Il est transitoire par définition.** PostgreSQL abat une transaction précisément pour que
/// l'autre puisse avancer ; au second essai, la gagnante a commité et l'exclusion rend un refus
/// **propre**. C'est pourquoi la réponse est un **réessai**, et non une traduction directe en
/// `unite_deja_occupee` : traduire ferait passer pour une chambre occupée tout interblocage, y
/// compris ceux qui n'ont rien à voir avec la disponibilité.
pub fn est_interblocage(erreur: &sqlx::Error) -> bool {
    matches!(erreur, sqlx::Error::Database(e) if e.code().as_deref() == Some(SQLSTATE_INTERBLOCAGE))
}

// =================================================================================================
//  Les refus du séjour — cycle 006
// =================================================================================================

/// Le type de refus du parcours de séjour.
///
/// # Chaque variante porte un CODE STABLE, et c'est le contrat avec l'écran
///
/// L'interface branche sa clé i18n sur le **code**, jamais sur le message de diagnostic — qui
/// nomme des tables et parle anglais technique (règle du cycle 002). Les six refus du cycle sont
/// au lexique v1.6.0 **avant** d'être codés.
///
/// # ⚠️ Deux `409` qui ne se confondent pas
///
/// - [`ErreurSejour::UniteDejaOccupee`] — une **période demandée** chevauche une occupation. Vient
///   de la contrainte d'exclusion, jamais d'une vérification préalable.
/// - [`ErreurSejour::UniteCibleOccupee`] — la **période restante** d'un séjour en cours ne peut
///   pas être servie par l'unité visée, au changement de chambre.
///
/// Les distinguer n'est pas cosmétique : Adjoua explique la première au client qui arrive, la
/// seconde au client déjà installé. Une phrase unique la ferait paraître incompétente dans un des
/// deux cas.
#[derive(Debug, thiserror::Error)]
pub enum ErreurSejour {
    /// Le séjour est déjà terminé. Terme utilisateur : « Ce séjour est déjà terminé. »
    #[error("sejour_deja_clos")]
    SejourDejaClos,

    /// Prolongation d'un séjour clos. **Distinct du précédent, et la phrase l'est aussi** :
    /// « On ne prolonge pas un séjour terminé. » — elle dit la RÈGLE, pas l'état, ce qui évite
    /// qu'Adjoua cherche comment « rouvrir » le séjour.
    #[error("sejour_clos")]
    SejourClos,

    /// ★ **Le refus qui NOMME son conflit** (FR-070).
    ///
    /// Il porte l'unité, l'instant de début de l'occupation suivante, et les unités alternatives
    /// libres sur l'intervalle étendu. Un message générique est un **défaut** : c'est la
    /// différence entre un refus qu'Adjoua peut expliquer au client et un refus qu'elle
    /// contournera.
    #[error("conflit_occupation_suivante")]
    ConflitOccupationSuivante,

    /// L'unité visée par un changement de chambre n'est pas libre sur la période restante.
    #[error("unite_cible_occupee")]
    UniteCibleOccupee,

    /// Le franchissement de `seuil_bascule_nuitee_minutes` **doit être confirmé avant** (FR-073).
    ///
    /// Le corps porte le montant résultant, et la requête se rejoue avec `bascule_acceptee: true`.
    /// Annoncer un changement de tarif **après** l'avoir appliqué serait le contraire de ce que le
    /// cadrage §8.3 vend au propriétaire.
    #[error("bascule_formule_non_confirmee")]
    BasculeFormuleNonConfirmee,

    #[error("sejour_inconnu")]
    SejourInconnu,

    #[error("note_inconnue")]
    NoteInconnue,

    #[error("accompagnant_inconnu")]
    AccompagnantInconnu,

    #[error("client_inconnu")]
    ClientInconnu,

    #[error("etablissement_inconnu")]
    EtablissementInconnu,

    #[error("service_inactif")]
    ServiceInactif,

    /// Un refus venu du moteur de disponibilité ou de tarification du cycle 004.
    ///
    /// **Réemployé, jamais réécrit.** `est_violation_exclusion` traduit la violation d'exclusion
    /// une seule fois dans le produit ; la recopier ici produirait, le jour d'un renommage de
    /// contrainte, un `500` au lieu d'un `409` sur le chemin le plus important du produit.
    #[error(transparent)]
    Attribution(#[from] crate::occupation::ErreurAttribution),

    #[error("erreur de base : {0}")]
    Base(#[from] sqlx::Error),

    #[error("écriture au grand livre : {0}")]
    Outbox(#[from] kaya_synchronisation::ErreurOutbox),

    #[error("contexte de tenant : {0}")]
    ContexteTenant(#[from] kaya_etablissements::tenant_context::ErreurContexteTenant),

    #[error("annuaire des clients : {0}")]
    Annuaire(String),

    #[error("registre des actions : {0}")]
    Audit(String),
}

impl ErreurSejour {
    /// Le **code stable** rendu dans `CorpsErreur`, sur lequel l'interface branche sa clé i18n.
    ///
    /// Il ne change jamais, même si le message change. Les six premiers sont au lexique v1.6.0.
    pub fn code(&self) -> &'static str {
        match self {
            ErreurSejour::SejourDejaClos => "sejour_deja_clos",
            ErreurSejour::SejourClos => "sejour_clos",
            ErreurSejour::ConflitOccupationSuivante => "conflit_occupation_suivante",
            ErreurSejour::UniteCibleOccupee => "unite_cible_occupee",
            ErreurSejour::BasculeFormuleNonConfirmee => "bascule_formule_non_confirmee",
            ErreurSejour::SejourInconnu => "sejour_inconnu",
            ErreurSejour::NoteInconnue => "note_inconnue",
            ErreurSejour::AccompagnantInconnu => "accompagnant_inconnu",
            ErreurSejour::ClientInconnu => "client_inconnu",
            ErreurSejour::EtablissementInconnu => "etablissement_inconnu",
            ErreurSejour::ServiceInactif => "service_inactif",
            // Le refus du cycle 004 garde **son** code : le retraduire ici en donnerait deux pour
            // le même fait, et l'écran en connaîtrait un des deux.
            ErreurSejour::Attribution(e) => match e {
                crate::occupation::ErreurAttribution::UniteDejaOccupee => "unite_deja_occupee",
                crate::occupation::ErreurAttribution::FormuleHorsCategorie => {
                    "formule_hors_categorie"
                }
                crate::occupation::ErreurAttribution::PlageNonFractionnable => {
                    "plage_non_fractionnable"
                }
                crate::occupation::ErreurAttribution::IntervalleInvalide => "intervalle_invalide",
                crate::occupation::ErreurAttribution::DureeHorsContrainte => {
                    "duree_hors_contrainte"
                }
                crate::occupation::ErreurAttribution::UniteInconnue => "unite_inconnue",
                crate::occupation::ErreurAttribution::FormuleInconnue => "formule_inconnue",
                crate::occupation::ErreurAttribution::OccupationInconnue => "occupation_inconnue",
                crate::occupation::ErreurAttribution::ServiceInactif => "service_inactif",
                crate::occupation::ErreurAttribution::EtablissementInconnu => {
                    "etablissement_inconnu"
                }
                _ => "erreur_interne",
            },
            ErreurSejour::Base(_)
            | ErreurSejour::Outbox(_)
            | ErreurSejour::ContexteTenant(_)
            | ErreurSejour::Annuaire(_)
            | ErreurSejour::Audit(_) => "erreur_interne",
        }
    }
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
