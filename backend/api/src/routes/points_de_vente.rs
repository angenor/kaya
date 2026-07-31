//! Handlers des points de vente — **ETB-03**, opérations 12 à 15.
//!
//! Terme utilisateur : **« Point de vente »**, et **« Comptoir »** pour celui qui n'a aucune table
//! (`docs/design/lexique.md`).

use actix_web::{HttpResponse, get, patch, post, put, web};
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;

use kaya_etablissements::Issue;
use kaya_etablissements::points_de_vente::{
    CreerPointDeVente, ErreurPointDeVente, ModifierPointDeVente, PointDeVenteVue, TableDemandee,
};

use crate::application::EtatApplication;
use crate::contexte::ContexteAppel;
use crate::routes::erreurs::{CorpsErreur, interne};

/// Corps de création d'un point de vente.
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreerPointDeVenteRequete {
    pub id: Uuid,
    /// Le service doit être **activé** sur l'établissement — sinon `422 module_non_actif`.
    pub module_code: String,
    pub nom: String,
    /// Rattachement de caisse. **Non vérifié à ce cycle** : `socle/caisse` n'a pas de table, et la
    /// vérification arrivera au cycle CAI par trait (research.md R-12).
    pub caisse_id: Option<Uuid>,
}

/// Corps de modification.
#[derive(Debug, Deserialize, ToSchema)]
pub struct ModifierPointDeVenteRequete {
    pub nom: Option<String>,
    pub caisse_id: Option<Uuid>,
    pub actif: Option<bool>,
}

/// Corps de remplacement des tables — **une liste vide fait un comptoir**.
#[derive(Debug, Deserialize, ToSchema)]
pub struct RemplacerTablesRequete {
    pub tables: Vec<TableRequete>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct TableRequete {
    pub id: Uuid,
    /// « 12 », « Terrasse 3 ».
    pub libelle: String,
}

/// Liste les points de vente d'un établissement.
///
/// **Une résidence meublée n'en a aucun, et la liste vide est la bonne réponse** — pas une erreur,
/// pas un établissement mal configuré.
#[utoipa::path(
    operation_id = "points_de_vente_lister",
    tag = "points-de-vente",
    params(("etablissement_id" = Uuid, Path, description = "Identifiant de l'établissement")),
    responses(
        (status = 200, description = "Points de vente, chacun avec ses tables", body = Vec<PointDeVenteVue>),
        (status = 401, description = "Non authentifié"),
        (status = 403, description = "Permission absente"),
        (status = 404, description = "Établissement inconnu", body = CorpsErreur),
    ),
    security(("bearer" = []))
)]
#[get("")]
pub async fn lister(
    etat: web::Data<EtatApplication>,
    contexte: ContexteAppel,
    chemin: web::Path<Uuid>,
) -> Result<HttpResponse, actix_web::Error> {
    let liste = etat
        .service_points_de_vente()
        .lister(contexte.tenant_id, chemin.into_inner())
        .await
        .map_err(en_reponse)?;

    Ok(HttpResponse::Ok().json(liste))
}

/// Crée un point de vente.
///
/// **`422 module_non_actif`** si le service n'est pas activé sur l'établissement : la clé
/// étrangère vers `etablissement_module` rend le cas structurellement impossible, et le `422`
/// donne le message qui **nomme le service**.
///
/// Le point de vente naît **sans table** — donc comptoir. Les tables se posent ensuite, par
/// `PUT .../tables`.
#[utoipa::path(
    operation_id = "points_de_vente_creer",
    tag = "points-de-vente",
    params(("etablissement_id" = Uuid, Path, description = "Identifiant de l'établissement")),
    request_body = CreerPointDeVenteRequete,
    responses(
        (status = 201, description = "Point de vente créé", body = PointDeVenteVue),
        (status = 200, description = "Déjà créé (rejeu idempotent)", body = PointDeVenteVue),
        (status = 400, description = "Requête invalide", body = CorpsErreur),
        (status = 401, description = "Non authentifié"),
        (status = 403, description = "Permission absente"),
        (status = 404, description = "Établissement inconnu", body = CorpsErreur),
        (status = 422, description = "Service non activé sur cet établissement", body = CorpsErreur),
    ),
    security(("bearer" = []))
)]
#[post("")]
pub async fn creer(
    etat: web::Data<EtatApplication>,
    contexte: ContexteAppel,
    chemin: web::Path<Uuid>,
    corps: web::Json<CreerPointDeVenteRequete>,
) -> Result<HttpResponse, actix_web::Error> {
    let corps = corps.into_inner();
    let (vue, issue) = etat
        .service_points_de_vente()
        .creer(
            contexte.tenant_id,
            chemin.into_inner(),
            CreerPointDeVente {
                id: corps.id,
                module_code: corps.module_code,
                nom: corps.nom,
                caisse_id: corps.caisse_id,
            },
        )
        .await
        .map_err(en_reponse)?;

    Ok(match issue {
        Issue::Creee => HttpResponse::Created().json(vue),
        Issue::DejaPresente => HttpResponse::Ok().json(vue),
    })
}

/// Modifie un point de vente.
#[utoipa::path(
    operation_id = "points_de_vente_modifier",
    tag = "points-de-vente",
    params(("point_de_vente_id" = Uuid, Path, description = "Identifiant du point de vente")),
    request_body = ModifierPointDeVenteRequete,
    responses(
        (status = 200, description = "Point de vente modifié", body = PointDeVenteVue),
        (status = 400, description = "Requête invalide", body = CorpsErreur),
        (status = 401, description = "Non authentifié"),
        (status = 403, description = "Permission absente"),
        (status = 404, description = "Point de vente inconnu", body = CorpsErreur),
    ),
    security(("bearer" = []))
)]
#[patch("")]
pub async fn modifier(
    etat: web::Data<EtatApplication>,
    contexte: ContexteAppel,
    chemin: web::Path<Uuid>,
    corps: web::Json<ModifierPointDeVenteRequete>,
) -> Result<HttpResponse, actix_web::Error> {
    let corps = corps.into_inner();
    let vue = etat
        .service_points_de_vente()
        .modifier(
            contexte.tenant_id,
            chemin.into_inner(),
            ModifierPointDeVente {
                nom: corps.nom,
                caisse_id: corps.caisse_id,
                actif: corps.actif,
            },
        )
        .await
        .map_err(en_reponse)?;

    Ok(HttpResponse::Ok().json(vue))
}

/// **Remplace l'ensemble des tables** d'un point de vente.
///
/// Une liste vide fait du point de vente un **comptoir** — transition légitime, exactement ce
/// qu'un maquis fait quand il retire ses tables pour ne plus servir qu'au comptoir. Ce n'est pas
/// une suppression accidentelle, et rien ne demande de confirmation particulière.
///
/// Les tables retirées sont **désactivées, jamais supprimées** : les commandes déjà passées les
/// référencent.
#[utoipa::path(
    operation_id = "points_de_vente_tables_remplacer",
    tag = "points-de-vente",
    params(("point_de_vente_id" = Uuid, Path, description = "Identifiant du point de vente")),
    request_body = RemplacerTablesRequete,
    responses(
        (status = 200, description = "Tables remplacées — liste vide = comptoir", body = PointDeVenteVue),
        (status = 400, description = "Requête invalide", body = CorpsErreur),
        (status = 401, description = "Non authentifié"),
        (status = 403, description = "Permission absente"),
        (status = 404, description = "Point de vente inconnu", body = CorpsErreur),
    ),
    security(("bearer" = []))
)]
#[put("")]
pub async fn remplacer_tables(
    etat: web::Data<EtatApplication>,
    contexte: ContexteAppel,
    chemin: web::Path<Uuid>,
    corps: web::Json<RemplacerTablesRequete>,
) -> Result<HttpResponse, actix_web::Error> {
    let corps = corps.into_inner();
    let vue = etat
        .service_points_de_vente()
        .remplacer_tables(
            contexte.tenant_id,
            chemin.into_inner(),
            corps
                .tables
                .into_iter()
                .map(|t| TableDemandee {
                    id: t.id,
                    libelle: t.libelle,
                })
                .collect(),
        )
        .await
        .map_err(en_reponse)?;

    Ok(HttpResponse::Ok().json(vue))
}

fn en_reponse(erreur: ErreurPointDeVente) -> actix_web::Error {
    let corps = CorpsErreur::nouveau(erreur.code(), erreur.valeur(), erreur.to_string());

    match erreur {
        ErreurPointDeVente::NomInvalide | ErreurPointDeVente::LibelleInvalide => corps.en_400(),
        ErreurPointDeVente::ModuleNonActif(_) | ErreurPointDeVente::NomDejaPris(_) => corps.en_422(),
        ErreurPointDeVente::EtablissementInconnu | ErreurPointDeVente::Inconnu => corps.en_404(),
        autre => interne("service des points de vente", autre),
    }
}
