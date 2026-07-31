//! Handlers de la configuration héritée — **ETB-04**, opérations 16 et 17.
//!
//! # Une clé sans valeur à aucun niveau est ABSENTE de la réponse
//!
//! Jamais rendue à `null`, jamais accompagnée d'un défaut.
//!
//! - `null` serait **indistinguable d'une valeur nulle légitimement posée** — et un paramètre
//!   qu'on a délibérément vidé n'est pas un paramètre qu'on n'a jamais réglé ;
//! - un défaut serait un **paramètre en dur** (principe I·c), qui n'apparaîtrait ni au
//!   récapitulatif, ni à l'écran, et qu'on découvrirait en cherchant pourquoi deux établissements
//!   se comportent différemment sans configuration visible.

use actix_web::{HttpResponse, get, put, web};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use kaya_etablissements::Cible;
use kaya_etablissements::configuration::{
    EcrireParametre, ErreurParametre, ValeurVue, portee_depuis_code,
};

use crate::application::EtatApplication;
use crate::contexte::ContexteAppel;
use crate::routes::erreurs::{CorpsErreur, interne};

/// Cible de résolution — les `Option` absents **raccourcissent la chaîne sans l'inventer**.
#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct CibleParams {
    /// Sans lui, la résolution s'arrête au niveau tenant.
    pub etablissement_id: Option<Uuid>,
    pub module_code: Option<String>,
    pub point_de_vente_id: Option<Uuid>,
    /// Une clé précise. **Absent, rend toutes les valeurs applicables** — en une descente.
    pub cle: Option<String>,
}

/// Corps d'écriture d'une valeur.
#[derive(Debug, Deserialize, ToSchema)]
pub struct EcrireParametreRequete {
    pub id: Uuid,
    pub cle: String,
    pub valeur: serde_json::Value,
    /// `TENANT` | `ETABLISSEMENT` | `MODULE` | `POINT_DE_VENTE`.
    pub portee: String,
    /// Identifiant du niveau visé. Absent pour `TENANT`, qui n'en a pas.
    pub portee_id: Option<Uuid>,
}

/// Résout la configuration applicable à une cible.
///
/// Chaque valeur porte **son origine** — c'est ce qui permet à l'écran de distinguer « vaut pour
/// tous vos établissements » de « modifié ici ».
#[utoipa::path(
    tag = "configuration",
    params(CibleParams),
    responses(
        (status = 200, description = "Valeurs applicables, chacune avec son origine", body = Vec<ValeurVue>),
        (status = 400, description = "Cible invalide", body = CorpsErreur),
        (status = 401, description = "Non authentifié"),
        (status = 403, description = "Permission absente"),
    ),
    security(("bearer" = []))
)]
#[get("")]
pub async fn resoudre(
    etat: web::Data<EtatApplication>,
    contexte: ContexteAppel,
    params: web::Query<CibleParams>,
) -> Result<HttpResponse, actix_web::Error> {
    let params = params.into_inner();
    let cible = Cible {
        tenant_id: contexte.tenant_id,
        etablissement_id: params.etablissement_id,
        module_code: params.module_code,
        point_de_vente_id: params.point_de_vente_id,
    };

    let service = etat.service_configuration();

    let valeurs = match params.cle {
        Some(cle) => {
            // Une clé sans valeur à aucun niveau produit une liste **vide**, jamais une entrée à
            // `null`.
            match service.resoudre(&cible, &cle).await.map_err(en_reponse)? {
                Some(v) => vec![ValeurVue {
                    cle,
                    valeur: v.valeur,
                    origine: v.origine.code().to_owned(),
                }],
                None => Vec::new(),
            }
        }
        None => service
            .resoudre_tout(&cible)
            .await
            .map_err(en_reponse)?
            .into_iter()
            .map(|(cle, v)| ValeurVue {
                cle,
                valeur: v.valeur,
                origine: v.origine.code().to_owned(),
            })
            .collect(),
    };

    Ok(HttpResponse::Ok().json(valeurs))
}

/// Écrit une valeur à un niveau de la chaîne.
///
/// **Deux refus `422`** : `cle_hors_catalogue` — la clé n'est pas au catalogue, ce que la clé
/// étrangère impose déjà en base ; `portee_interdite` — la portée demandée est plus basse que la
/// `portee_la_plus_basse` déclarée pour cette clé.
///
/// Un troisième, `type_incompatible`, est **l'extension de la porte P-10 au `JSONB`** : une clé de
/// type `MONTANT_MINEUR` refuse toute valeur qui ne soit pas un entier. Sans lui, un barème écrit
/// `1500.75` entrerait sans qu'aucune colonne ne soit en cause, et le premier calcul fiscal
/// produirait un montant à virgule dans une devise à zéro décimale.
#[utoipa::path(
    tag = "configuration",
    request_body = EcrireParametreRequete,
    responses(
        (status = 200, description = "Valeur écrite", body = Vec<ValeurVue>),
        (status = 400, description = "Requête invalide", body = CorpsErreur),
        (status = 401, description = "Non authentifié"),
        (status = 403, description = "Permission absente"),
        (status = 404, description = "Niveau visé inconnu", body = CorpsErreur),
        (status = 422, description = "Clé hors catalogue, portée interdite ou type incompatible", body = CorpsErreur),
    ),
    security(("bearer" = []))
)]
#[put("")]
pub async fn ecrire(
    etat: web::Data<EtatApplication>,
    contexte: ContexteAppel,
    corps: web::Json<EcrireParametreRequete>,
) -> Result<HttpResponse, actix_web::Error> {
    let corps = corps.into_inner();
    let cle = corps.cle.clone();
    let portee = portee_depuis_code(&corps.portee);

    etat.service_configuration()
        .ecrire(
            contexte.tenant_id,
            EcrireParametre {
                id: corps.id,
                cle: corps.cle,
                valeur: corps.valeur,
                portee,
                portee_id: corps.portee_id,
            },
        )
        .await
        .map_err(en_reponse)?;

    // La réponse rend la valeur **telle qu'elle se résout désormais** depuis le niveau écrit, avec
    // son origine. Renvoyer simplement le corps reçu laisserait croire à une surcharge là où une
    // valeur plus spécifique peut déjà la masquer.
    let cible = Cible {
        tenant_id: contexte.tenant_id,
        etablissement_id: None,
        module_code: None,
        point_de_vente_id: None,
    };
    let valeurs = match etat
        .service_configuration()
        .resoudre(&cible, &cle)
        .await
        .map_err(en_reponse)?
    {
        Some(v) => vec![ValeurVue {
            cle,
            valeur: v.valeur,
            origine: v.origine.code().to_owned(),
        }],
        None => Vec::new(),
    };

    Ok(HttpResponse::Ok().json(valeurs))
}

fn en_reponse(erreur: ErreurParametre) -> actix_web::Error {
    let corps = CorpsErreur::nouveau(erreur.code(), erreur.valeur(), erreur.to_string());

    match erreur {
        ErreurParametre::PorteeIdManquant(_) => corps.en_400(),
        ErreurParametre::CleHorsCatalogue(_)
        | ErreurParametre::PorteeInterdite { .. }
        | ErreurParametre::TypeIncompatible { .. } => corps.en_422(),
        ErreurParametre::NiveauInconnu => corps.en_404(),
        autre => interne("service de configuration", autre),
    }
}
