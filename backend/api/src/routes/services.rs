//! Handlers des services d'un établissement — **ETB-02, ETB-02b**, opérations 8 à 11.
//!
//! Terme utilisateur : **« Vos services »**. Le mot « capacité » n'apparaît nulle part à l'écran ;
//! seule la capacité concrète est nommée (`docs/design/lexique.md`).
//!
//! # `GET` ne rend que les services ACTIFS — et il n'existe aucun moyen de demander les autres
//!
//! Un paramètre `?inclure_inactifs=true` **n'existe pas**. Ce que l'interface ne doit pas montrer,
//! elle ne doit pas le recevoir (principe VII) : offrir le paramètre, c'est garantir qu'un jour
//! quelqu'un l'utilisera « juste pour la console d'administration », puis que la liste grisée
//! remontera dans l'écran principal.

use actix_web::{HttpResponse, get, post, put, web};
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;

use kaya_etablissements::Issue;
use kaya_etablissements::modules::{
    BasculerService, CapaciteDuService, DeclarerCapacite, ErreurModules, ServiceActif,
};

use crate::application::EtatApplication;
use crate::contexte::ContexteAppel;
use crate::routes::erreurs::{CorpsErreur, ObstacleVue, interne};

/// Corps d'activation ou de désactivation — **le même point d'entrée porte les deux sens**.
#[derive(Debug, Deserialize, ToSchema)]
pub struct BasculerServiceRequete {
    /// UUID v7 client, utilisé à la **première** activation. Une réactivation est un `UPDATE` de
    /// la ligne existante, jamais une seconde ligne : c'est ce qui restitue l'état antérieur.
    pub id: Uuid,
    pub actif: bool,
}

/// Corps de déclaration de capacité.
#[derive(Debug, Deserialize, ToSchema)]
pub struct DeclarerCapaciteRequete {
    pub id: Uuid,
    /// `STOCK` seule est implémentée au MVP.
    pub capacite_code: String,
    /// `SIMPLE` seul est implémenté au MVP.
    pub profil_code: String,
}

/// Chemin `{etablissement_id}/services/{module_code}`.
#[derive(Debug, Deserialize)]
pub struct CheminService {
    pub etablissement_id: Uuid,
    pub module_code: String,
}

/// Liste les services **actifs** d'un établissement.
#[utoipa::path(
    tag = "services",
    params(("etablissement_id" = Uuid, Path, description = "Identifiant de l'établissement")),
    responses(
        (status = 200, description = "Services actifs, avec leurs capacités", body = Vec<ServiceActif>),
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
    let services = etat
        .service_modules()
        .services_actifs(contexte.tenant_id, chemin.into_inner())
        .await
        .map_err(en_reponse)?;

    Ok(HttpResponse::Ok().json(services))
}

/// Active ou désactive un service.
///
/// **Idempotent, et il porte les deux sens.** `201` à la première activation, `200` ensuite. Deux
/// points d'entrée distincts laisseraient deux chemins pour un état, et un jour deux
/// comportements.
///
/// **La désactivation ne supprime rien** : déclarations de capacité et surcharges de configuration
/// deviennent inertes sans être touchées, et la réactivation les restitue.
#[utoipa::path(
    tag = "services",
    params(
        ("etablissement_id" = Uuid, Path, description = "Identifiant de l'établissement"),
        ("module_code" = String, Path, description = "Code du module d'activité"),
    ),
    request_body = BasculerServiceRequete,
    responses(
        (status = 201, description = "Service activé pour la première fois", body = ServiceActif),
        (status = 200, description = "État atteint (rejeu ou bascule)", body = ServiceActif),
        (status = 400, description = "Requête invalide", body = CorpsErreur),
        (status = 401, description = "Non authentifié"),
        (status = 403, description = "Permission absente"),
        (status = 404, description = "Établissement inconnu", body = CorpsErreur),
        (status = 422, description = "Module non implémenté, ou désactivation bloquée", body = CorpsErreur),
    ),
    security(("bearer" = []))
)]
#[put("")]
pub async fn basculer(
    etat: web::Data<EtatApplication>,
    contexte: ContexteAppel,
    chemin: web::Path<CheminService>,
    corps: web::Json<BasculerServiceRequete>,
) -> Result<HttpResponse, actix_web::Error> {
    let chemin = chemin.into_inner();
    let corps = corps.into_inner();
    let service = etat.service_modules();

    let issue = service
        .basculer(
            contexte.tenant_id,
            chemin.etablissement_id,
            &chemin.module_code,
            BasculerService {
                id: corps.id,
                actif: corps.actif,
            },
        )
        .await
        .map_err(en_reponse)?;

    // Le corps rendu est l'état **tel qu'il est en base** après l'opération. Sur une
    // désactivation, le service ne figure plus dans la liste des actifs — c'est exact, et le
    // client doit le constater plutôt que de recevoir une ligne qui n'existe plus.
    let actifs = service
        .services_actifs(contexte.tenant_id, chemin.etablissement_id)
        .await
        .map_err(en_reponse)?;
    let vue = actifs
        .into_iter()
        .find(|s| s.module_code == chemin.module_code);

    Ok(match (issue.issue, vue) {
        (Issue::Creee, Some(vue)) => HttpResponse::Created().json(vue),
        (_, Some(vue)) => HttpResponse::Ok().json(vue),
        (_, None) => HttpResponse::Ok().json(serde_json::json!({
            "module_code": chemin.module_code,
            "actif": false,
        })),
    })
}

/// Liste les capacités déclarées par un service.
#[utoipa::path(
    tag = "services",
    params(
        ("etablissement_id" = Uuid, Path, description = "Identifiant de l'établissement"),
        ("module_code" = String, Path, description = "Code du module d'activité"),
    ),
    responses(
        (status = 200, description = "Capacités déclarées", body = Vec<CapaciteDuService>),
        (status = 401, description = "Non authentifié"),
        (status = 403, description = "Permission absente"),
        (status = 404, description = "Établissement inconnu", body = CorpsErreur),
        (status = 422, description = "Service non actif", body = CorpsErreur),
    ),
    security(("bearer" = []))
)]
#[get("")]
pub async fn lister_capacites(
    etat: web::Data<EtatApplication>,
    contexte: ContexteAppel,
    chemin: web::Path<CheminService>,
) -> Result<HttpResponse, actix_web::Error> {
    let chemin = chemin.into_inner();
    let capacites = etat
        .service_modules()
        .capacites_du_service(
            contexte.tenant_id,
            chemin.etablissement_id,
            &chemin.module_code,
        )
        .await
        .map_err(en_reponse)?;

    Ok(HttpResponse::Ok().json(capacites))
}

/// Déclare une capacité consommée par un service — **la porte P-06 vue de l'API**.
///
/// **Les neuf refus du cycle**, tous en `422`, tous nommant la valeur, **aucune ligne écrite** :
/// six capacités (`LIVRAISON`, `PRODUCTION`, `COMMERCE_EN_LIGNE`, `FIDELITE`, `DEVIS`,
/// `COMPTES_CLIENTS`) et trois profils (`AUCUN`, `VALORISE`, `DETAILLE`).
///
/// Le `422` est la **deuxième** des trois couches du refus. La première — clé étrangère composite
/// et `CHECK` en base — le tient même pour un import ou un script de reprise ; la troisième est
/// l'absence pure à l'interface.
#[utoipa::path(
    tag = "services",
    params(
        ("etablissement_id" = Uuid, Path, description = "Identifiant de l'établissement"),
        ("module_code" = String, Path, description = "Code du module d'activité"),
    ),
    request_body = DeclarerCapaciteRequete,
    responses(
        (status = 201, description = "Capacité déclarée", body = Vec<CapaciteDuService>),
        (status = 200, description = "Déjà déclarée (rejeu idempotent)", body = Vec<CapaciteDuService>),
        (status = 400, description = "Requête invalide", body = CorpsErreur),
        (status = 401, description = "Non authentifié"),
        (status = 403, description = "Permission absente"),
        (status = 404, description = "Établissement inconnu", body = CorpsErreur),
        (status = 422, description = "Capacité ou profil non implémenté, ou service non actif", body = CorpsErreur),
    ),
    security(("bearer" = []))
)]
#[post("")]
pub async fn declarer_capacite(
    etat: web::Data<EtatApplication>,
    contexte: ContexteAppel,
    chemin: web::Path<CheminService>,
    corps: web::Json<DeclarerCapaciteRequete>,
) -> Result<HttpResponse, actix_web::Error> {
    let chemin = chemin.into_inner();
    let corps = corps.into_inner();
    let service = etat.service_modules();

    let issue = service
        .declarer_capacite(
            contexte.tenant_id,
            chemin.etablissement_id,
            &chemin.module_code,
            DeclarerCapacite {
                id: corps.id,
                capacite_code: corps.capacite_code,
                profil_code: corps.profil_code,
            },
        )
        .await
        .map_err(en_reponse)?;

    let capacites = service
        .capacites_du_service(
            contexte.tenant_id,
            chemin.etablissement_id,
            &chemin.module_code,
        )
        .await
        .map_err(en_reponse)?;

    Ok(match issue {
        Issue::Creee => HttpResponse::Created().json(capacites),
        Issue::DejaPresente => HttpResponse::Ok().json(capacites),
    })
}

/// Traduit une erreur du domaine des services en réponse structurée.
pub fn en_reponse(erreur: ErreurModules) -> actix_web::Error {
    let corps = CorpsErreur::nouveau(erreur.code(), erreur.valeur(), erreur.to_string())
        .avec_motif(erreur.motif_cle())
        .avec_obstacles(
            erreur
                .obstacles()
                .iter()
                .map(|o| ObstacleVue {
                    module_code: o.module_code.clone(),
                    motif_cle: o.motif_cle.clone(),
                    nombre: o.nombre,
                })
                .collect(),
        );

    match erreur {
        ErreurModules::EtablissementInconnu => corps.en_404(),

        // `422` et non `404` pour un code inconnu : la ressource visée — l'établissement —
        // existe. C'est la **valeur soumise** qui est refusée, et le client doit la corriger, pas
        // conclure que l'établissement a disparu.
        ErreurModules::ModuleInconnu(_)
        | ErreurModules::ModuleNonImplemente(_)
        | ErreurModules::ModuleNonActif(_)
        | ErreurModules::CapaciteInconnue(_)
        | ErreurModules::CapaciteNonImplementee(_)
        | ErreurModules::ProfilInconnu(_)
        | ErreurModules::ProfilNonImplemente { .. }
        | ErreurModules::DesactivationBloquee(_) => corps.en_422(),

        autre => interne("service des modules", autre),
    }
}
