//! Handlers de la disponibilité et de l'attribution — **HEB-02**, opérations 9 à 11.
//!
//! # La consultation est une lecture, jamais une garantie
//!
//! `GET /disponibilite` rend les unités attribuables **à l'instant où il répond**. Entre cette
//! lecture et l'attribution, une autre transaction peut prendre l'unité — et c'est normal. La
//! garantie est la contrainte d'exclusion, jamais cette réponse (FR-013).
//!
//! Un client qui traiterait ce résultat comme une réservation reproduirait exactement le verrou
//! applicatif que le principe IV refuse. C'est pourquoi la réponse porte `instant_autorite` : elle
//! dit **quand** elle était vraie, ce qui est une information honnête, plutôt que de laisser
//! croire qu'elle le reste.
//!
//! # `200` sur rejeu, jamais `409`
//!
//! Un client hors ligne qui vide sa file ne doit pas voir d'erreur pour une écriture que le
//! serveur a déjà acceptée (principe VI). Le corps renvoyé est la ligne **telle qu'elle est en
//! base** : le serveur fait foi en conflit.
//!
//! Ne pas confondre avec le `409` d'`unite_deja_occupee`, qui est un refus réel — deux
//! identifiants différents sur des intervalles chevauchants.

use actix_web::{HttpResponse, get, post, web};
use serde::Deserialize;
use time::OffsetDateTime;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use kaya_hebergement::Issue;
use kaya_hebergement::occupation::{
    DemandeAttribution, ErreurAttribution, OccupationVue, UniteDisponible,
};

use crate::application::EtatApplication;
use crate::contexte::ContexteAppel;
use crate::routes::erreurs::{CorpsErreur, interne};
use crate::securite::exiger;

const CONSULTER: &str = "heb.disponibilite.consulter";
const ATTRIBUER: &str = "heb.unite.attribuer";
const LIBERER: &str = "heb.unite.liberer";

// =================================================================================================
//  9 · Consulter la disponibilité
// =================================================================================================

/// Bornes de la consultation.
#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct DisponibiliteParams {
    /// Le type de chambre interrogé.
    pub categorie_id: Uuid,
    /// Début de l'intervalle, RFC 3339.
    #[serde(with = "time::serde::rfc3339")]
    pub debut: OffsetDateTime,
    /// Fin de l'intervalle, RFC 3339. **Exclue** — une chambre libérée à midi est disponible à
    /// midi.
    #[serde(with = "time::serde::rfc3339")]
    pub fin: OffsetDateTime,
}

/// Ce que la consultation rend.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct DisponibiliteVue {
    pub unites_disponibles: Vec<UniteDisponible>,
    /// **Horodatage serveur.** Il dit *quand* cette réponse était vraie — elle ne l'est plus
    /// nécessairement au moment où le client la lit, et c'est une information honnête plutôt
    /// qu'une garantie qu'on ne peut pas tenir.
    #[serde(with = "time::serde::rfc3339")]
    pub instant_autorite: OffsetDateTime,
}

#[utoipa::path(
    operation_id = "hebergement_consulter_disponibilite",
    tag = "hebergement",
    params(
        ("etablissement_id" = Uuid, Path, description = "Identifiant de l'établissement"),
        DisponibiliteParams,
    ),
    responses(
        (status = 200, description = "Unités attribuables à l'instant d'autorité — AUCUNE garantie de réservation", body = DisponibiliteVue),
        (status = 401, description = "Non authentifié"),
        (status = 403, description = "Permission absente", body = CorpsErreur),
        (status = 404, description = "Établissement inconnu", body = CorpsErreur),
        (status = 409, description = "Service hébergement non actif", body = CorpsErreur),
        (status = 422, description = "Intervalle invalide", body = CorpsErreur),
    ),
    security(("bearer" = []))
)]
#[get("")]
pub async fn consulter(
    etat: web::Data<EtatApplication>,
    contexte: ContexteAppel,
    chemin: web::Path<Uuid>,
    params: web::Query<DisponibiliteParams>,
) -> Result<HttpResponse, actix_web::Error> {
    exiger(&contexte, CONSULTER)?;
    let etablissement_id = chemin.into_inner();

    let (unites_disponibles, instant_autorite) = etat
        .service_occupation(contexte.tenant_id)
        .unites_disponibles_avec_instant(
            etablissement_id,
            params.categorie_id,
            params.debut,
            params.fin,
        )
        .await
        .map_err(en_reponse)?;

    Ok(HttpResponse::Ok().json(DisponibiliteVue {
        unites_disponibles,
        instant_autorite,
    }))
}

// =================================================================================================
//  10 · Attribuer — l'opération que la contrainte protège
// =================================================================================================

/// Demande d'attribution.
///
/// **La borne haute de la période n'y figure pas**, et c'est délibéré : le serveur la calcule en
/// ajoutant le battement de remise en état de la catégorie. Si le client l'envoyait, il pourrait
/// la mettre à zéro et supprimer le ménage.
#[derive(Debug, Deserialize, ToSchema)]
pub struct AttribuerRequete {
    /// UUID v7 **généré par le client** — c'est lui qui rend le rejeu inoffensif.
    pub id: Uuid,
    pub unite_id: Uuid,
    pub formule_id: Uuid,
    #[serde(with = "time::serde::rfc3339")]
    pub debut_client: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub fin_client: OffsetDateTime,
}

#[utoipa::path(
    operation_id = "hebergement_attribuer_unite",
    tag = "hebergement",
    params(("etablissement_id" = Uuid, Path, description = "Identifiant de l'établissement")),
    request_body = AttribuerRequete,
    responses(
        (status = 201, description = "Unité attribuée", body = OccupationVue),
        (status = 200, description = "Déjà attribuée (rejeu idempotent)", body = OccupationVue),
        (status = 401, description = "Non authentifié"),
        (status = 403, description = "Permission absente", body = CorpsErreur),
        (status = 404, description = "Unité ou formule inconnue", body = CorpsErreur),
        (status = 409, description = "Chambre déjà prise sur cette période, ou service non actif", body = CorpsErreur),
        (status = 422, description = "Formule hors catégorie, intervalle invalide, durée hors contrainte, plage non fractionnable", body = CorpsErreur),
    ),
    security(("bearer" = []))
)]
#[post("")]
pub async fn attribuer(
    etat: web::Data<EtatApplication>,
    contexte: ContexteAppel,
    chemin: web::Path<Uuid>,
    corps: web::Json<AttribuerRequete>,
) -> Result<HttpResponse, actix_web::Error> {
    exiger(&contexte, ATTRIBUER)?;
    let etablissement_id = chemin.into_inner();
    let corps = corps.into_inner();

    let (vue, issue) = etat
        .service_occupation(contexte.tenant_id)
        .attribuer(DemandeAttribution {
            id: corps.id,
            etablissement_id,
            unite_id: corps.unite_id,
            formule_id: corps.formule_id,
            debut_client: corps.debut_client,
            fin_client: corps.fin_client,
        })
        .await
        .map_err(en_reponse)?;

    Ok(match issue {
        Issue::Creee => HttpResponse::Created().json(vue),
        Issue::DejaPresente => HttpResponse::Ok().json(vue),
    })
}

// =================================================================================================
//  11 · Libérer — un UPDATE, jamais un DELETE
// =================================================================================================

#[derive(Debug, Deserialize)]
pub struct CheminOccupation {
    pub etablissement_id: Uuid,
    pub occupation_id: Uuid,
}

/// Corps de libération.
#[derive(Debug, Deserialize, ToSchema)]
pub struct LibererRequete {
    /// UUID v7 de l'**opération**, pour l'idempotence côté client. La libération elle-même est
    /// idempotente par son effet : une occupation déjà libérée rend `200` sans second événement.
    pub id: Uuid,
}

/// Libère une occupation.
///
/// La période est **raccourcie** à `now()` + le battement de remise en état, et `statut` passe à
/// `liberee`. Ce n'est jamais un `DELETE` : une chambre occupée reste une chambre occupée dans
/// l'histoire, et `DELETE` n'est pas accordé à `kaya_app`.
#[utoipa::path(
    operation_id = "hebergement_liberer_occupation",
    tag = "hebergement",
    params(
        ("etablissement_id" = Uuid, Path, description = "Identifiant de l'établissement"),
        ("occupation_id" = Uuid, Path, description = "Identifiant de l'occupation"),
    ),
    request_body = LibererRequete,
    responses(
        (status = 200, description = "Occupation libérée, ou déjà libérée (rejeu)", body = OccupationVue),
        (status = 401, description = "Non authentifié"),
        (status = 403, description = "Permission absente", body = CorpsErreur),
        (status = 404, description = "Occupation inconnue", body = CorpsErreur),
        (status = 409, description = "Service hébergement non actif", body = CorpsErreur),
    ),
    security(("bearer" = []))
)]
#[post("")]
pub async fn liberer(
    etat: web::Data<EtatApplication>,
    contexte: ContexteAppel,
    chemin: web::Path<CheminOccupation>,
    _corps: web::Json<LibererRequete>,
) -> Result<HttpResponse, actix_web::Error> {
    exiger(&contexte, LIBERER)?;
    let chemin = chemin.into_inner();

    let (vue, _issue) = etat
        .service_occupation(contexte.tenant_id)
        .liberer(chemin.etablissement_id, chemin.occupation_id)
        .await
        .map_err(en_reponse)?;

    // **`200` dans les deux cas.** Une libération et son rejeu produisent le même état, et
    // l'appelant n'a rien à faire de différent : distinguer `201` de `200` n'aurait de sens que
    // si une ressource naissait, et il n'en naît pas.
    Ok(HttpResponse::Ok().json(vue))
}

// =================================================================================================
//  Traduction des refus
// =================================================================================================

/// **Les codes de refus sont distincts, et chacun se traduit par une phrase différente.**
///
/// | Statut | Code | Ce que l'écran dit (lexique) |
/// |---|---|---|
/// | `409` | `unite_deja_occupee` | « Cette chambre est déjà prise sur cette période » |
/// | `422` | `formule_hors_categorie` | « Cette formule ne s'applique pas à cette chambre » |
/// | `422` | `plage_non_fractionnable` | « Une demi-journée se loue en entier » |
/// | `422` | `intervalle_invalide` | « La fin doit être après le début » |
/// | `422` | `duree_hors_contrainte` | « Cette formule se loue de 1 h à 8 h » |
///
/// Les fondre en un seul « requête invalide » obligerait l'utilisateur à deviner ce qu'il doit
/// corriger — et sur un comptoir, avec un client en face, il ne devine pas : il appelle.
pub(crate) fn en_reponse(erreur: ErreurAttribution) -> actix_web::Error {
    let code = erreur.code();
    match erreur {
        // **Le refus qui vient de la contrainte d'exclusion.** `409` : la requête est bien formée,
        // c'est l'état du monde qui la refuse.
        ErreurAttribution::UniteDejaOccupee => {
            CorpsErreur::nouveau(code, None, erreur.to_string()).en_409()
        }
        ErreurAttribution::ServiceInactif => {
            CorpsErreur::nouveau(code, None, erreur.to_string()).en_409()
        }
        ErreurAttribution::EtablissementInconnu
        | ErreurAttribution::UniteInconnue
        | ErreurAttribution::FormuleInconnue
        | ErreurAttribution::OccupationInconnue => {
            CorpsErreur::nouveau(code, None, erreur.to_string()).en_404()
        }
        ErreurAttribution::FormuleHorsCategorie
        | ErreurAttribution::PlageNonFractionnable
        | ErreurAttribution::IntervalleInvalide
        | ErreurAttribution::DureeHorsContrainte
        | ErreurAttribution::OccupationDejaLiberee => {
            CorpsErreur::nouveau(code, None, erreur.to_string()).en_422()
        }
        ErreurAttribution::Referentiel(e) => super::hebergement_referentiel::en_reponse(e),
        autre => interne("disponibilité d'hébergement", autre),
    }
}
