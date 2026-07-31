//! Sonde de santé — **FR-031**, `contracts/http-api.md` §1.
//!
//! # Le point qui décide si cette sonde sert à quelque chose
//!
//! Elle vérifie chaque dépendance par une **requête réelle et courte** — `SELECT 1` sur la base,
//! `PING` sur le cache, une opération S3 sur le stockage objet — **jamais l'état d'un pool en
//! mémoire**.
//!
//! Un pool peut se croire sain plusieurs minutes après la mort de la base : ses connexions sont
//! encore ouvertes, rien ne les a réveillées. C'est exactement l'intervalle pendant lequel
//! l'alerte des 2 minutes (FR-057) ne partirait pas, alors que plus aucune requête client
//! n'aboutit. Le support est à 220 km d'Abengourou ; cet intervalle-là est celui où le pilote
//! appelle et où personne ne sait quoi lui répondre.
//!
//! # Ce que la sonde ne renvoie jamais
//!
//! Ni nom d'hôte, ni chaîne de connexion, ni version de PostgreSQL, ni trace d'erreur.
//! **L'endpoint est public** — il doit l'être, une sonde qui exige un jeton ne peut pas être
//! interrogée par la supervision externe. Il ne dit donc que ce qui est nécessaire : quelle
//! dépendance ne répond pas.

use std::time::Duration;

use actix_web::{HttpResponse, get, web};
use serde::Serialize;
use utoipa::ToSchema;

use crate::application::EtatApplication;

/// Délai au-delà duquel une dépendance est déclarée muette.
///
/// Court **volontairement** : une sonde qui attend dix secondes est une sonde que la supervision
/// externe déclare en échec avant d'avoir sa réponse, et l'alerte porte alors sur la sonde plutôt
/// que sur la panne.
const DELAI_SONDE: Duration = Duration::from_secs(2);

/// État global du service.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum StatutSante {
    Operationnel,
    Degrade,
}

/// État d'une dépendance.
#[derive(Debug, Serialize, ToSchema)]
pub struct EtatDependance {
    /// `base`, `cache` ou `stockage_objet`.
    pub nom: &'static str,
    pub statut: StatutSante,
    /// Durée de la vérification, en millisecondes. Utile au diagnostic à distance : une base qui
    /// répond en 1 800 ms n'est pas encore en panne, mais le sera.
    pub duree_ms: u128,
}

/// Réponse de la sonde.
#[derive(Debug, Serialize, ToSchema)]
pub struct EtatSante {
    pub statut: StatutSante,
    /// Version du binaire — **télémétrie du parc auto-hébergé** (principe VIII), pas une fuite :
    /// c'est la version de Kaya, jamais celle d'une dépendance.
    pub version: &'static str,
    pub dependances: Vec<EtatDependance>,
}

/// Sonde de santé du service.
#[utoipa::path(
    tag = "systeme",
    responses(
        (status = 200, description = "Service opérationnel", body = EtatSante),
        (status = 503, description = "Service dégradé",      body = EtatSante),
    )
)]
#[get("/health")]
pub async fn sante(etat: web::Data<EtatApplication>) -> HttpResponse {
    let mut dependances = Vec::with_capacity(3);

    dependances.push(sonder("base", sonder_base(&etat)).await);
    dependances.push(sonder("cache", sonder_cache()).await);
    dependances.push(sonder("stockage_objet", sonder_stockage()).await);

    let degrade = dependances
        .iter()
        .any(|d| d.statut == StatutSante::Degrade);

    let corps = EtatSante {
        statut: if degrade {
            StatutSante::Degrade
        } else {
            StatutSante::Operationnel
        },
        version: env!("CARGO_PKG_VERSION"),
        dependances,
    };

    if degrade {
        // `503`, pas `200` avec un corps qui dit « dégradé » : la supervision externe et les
        // équilibreurs de charge lisent le code de statut, pas le corps.
        HttpResponse::ServiceUnavailable().json(corps)
    } else {
        HttpResponse::Ok().json(corps)
    }
}

/// Applique le délai maximal et chronomètre.
async fn sonder<F>(nom: &'static str, verification: F) -> EtatDependance
where
    F: std::future::Future<Output = Result<(), String>>,
{
    let debut = std::time::Instant::now();
    let resultat = tokio::time::timeout(DELAI_SONDE, verification).await;
    let duree_ms = debut.elapsed().as_millis();

    let statut = match resultat {
        Ok(Ok(())) => StatutSante::Operationnel,
        Ok(Err(motif)) => {
            // Le motif part dans les journaux, corrélé — jamais dans la réponse publique.
            tracing::warn!(dependance = nom, motif, "dépendance dégradée");
            StatutSante::Degrade
        }
        Err(_) => {
            tracing::warn!(dependance = nom, delai_ms = DELAI_SONDE.as_millis(), "dépendance muette");
            StatutSante::Degrade
        }
    };

    EtatDependance {
        nom,
        statut,
        duree_ms,
    }
}

/// `SELECT 1` — une requête réelle, pas l'état du pool.
async fn sonder_base(etat: &EtatApplication) -> Result<(), String> {
    sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&etat.pool)
        .await
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// `PING` sur le cache.
///
/// Redis ne porte que de l'éphémère reconstructible (principe II) : son indisponibilité dégrade
/// le service sans le rendre faux. Elle est donc signalée, et non tue.
async fn sonder_cache() -> Result<(), String> {
    let url = std::env::var("REDIS_URL").map_err(|_| "REDIS_URL absente".to_owned())?;
    let client = redis::Client::open(url).map_err(|e| e.to_string())?;
    let mut connexion = client
        .get_multiplexed_async_connection()
        .await
        .map_err(|e| e.to_string())?;
    redis::cmd("PING")
        .query_async::<String>(&mut connexion)
        .await
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// Vérification du stockage objet, **par l'API S3 uniquement** (principe II).
async fn sonder_stockage() -> Result<(), String> {
    let endpoint = std::env::var("S3_ENDPOINT").map_err(|_| "S3_ENDPOINT absente".to_owned())?;
    let region = std::env::var("S3_REGION").unwrap_or_else(|_| "garage".to_owned());
    let cle = std::env::var("S3_ACCESS_KEY").map_err(|_| "S3_ACCESS_KEY absente".to_owned())?;
    let secret = std::env::var("S3_SECRET_KEY").map_err(|_| "S3_SECRET_KEY absente".to_owned())?;

    let configuration = aws_sdk_s3::Config::builder()
        .behavior_version(aws_sdk_s3::config::BehaviorVersion::latest())
        .region(aws_sdk_s3::config::Region::new(region))
        .endpoint_url(endpoint)
        // Garage n'implémente pas l'adressage par sous-domaine de compartiment.
        .force_path_style(true)
        .credentials_provider(aws_sdk_s3::config::Credentials::new(
            cle, secret, None, None, "kaya",
        ))
        .build();

    aws_sdk_s3::Client::from_conf(configuration)
        .list_buckets()
        .send()
        .await
        .map(|_| ())
        .map_err(|e| e.to_string())
}
