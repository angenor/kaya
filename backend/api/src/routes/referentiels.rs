//! Handlers des référentiels — **ETB-02, ETB-02b**, opérations 5 à 7.
//!
//! # Lecture seule — aucun verbe d'écriture n'est exposé
//!
//! L'enrichissement du référentiel relève de l'éditeur (ETB-08, provision). Un point d'entrée
//! d'écriture existant « pour plus tard » serait une **surface que rien ne garde** : ni permission
//! définie, ni test d'isolation pensé pour lui, ni règle métier. Le jour où la console éditeur
//! existera, elle apportera son propre point d'entrée et le régime qui va avec.
//!
//! # `implementee` est rendu, et c'est délibéré
//!
//! L'interface **n'affiche jamais** une valeur non implémentée (FR-036). Le drapeau existe pour
//! deux raisons : la console éditeur pilotera un jour le référentiel, et le client doit pouvoir
//! distinguer « valeur inconnue » de « valeur connue non implémentée » dans un message d'erreur.
//! Le filtrage est une **règle d'affichage**, testée sur la fonction de sélection côté
//! application — pas une amputation de la réponse.

use actix_web::{HttpResponse, get, web};
use serde::Serialize;
use utoipa::ToSchema;

use kaya_etablissements::modules::ErreurModules;

use crate::application::EtatApplication;
use crate::contexte::ContexteAppel;

/// Une entrée de référentiel, telle que l'API la rend.
#[derive(Debug, Serialize, ToSchema)]
pub struct EntreeReferentiel {
    pub code: String,
    /// **Clé i18n, jamais un libellé.** Une chaîne utilisateur en base échapperait à la porte
    /// P-16 : ni parité fr/en, ni relecture de vocabulaire.
    pub libelle_cle: String,
    /// Voir l'en-tête du module : rendu délibérément, filtré à l'affichage.
    pub implementee: bool,
    /// Ordre d'affichage stable, indépendant de l'alphabet et de la locale.
    pub ordre: i16,
    /// Clé i18n du motif de refus — profils seulement, et `null` pour un profil implémenté.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub motif_refus_cle: Option<String>,
}

/// Référentiel des modules d'activité — « Vos services ».
#[utoipa::path(
    operation_id = "referentiels_modules_activite",
    tag = "referentiels",
    responses(
        (status = 200, description = "Modules d'activité connus", body = Vec<EntreeReferentiel>),
        (status = 401, description = "Non authentifié"),
    ),
    security(("bearer" = []))
)]
#[get("/modules-activite")]
pub async fn modules_activite(
    etat: web::Data<EtatApplication>,
    _contexte: ContexteAppel,
) -> Result<HttpResponse, actix_web::Error> {
    let entrees = etat.referentiel_modules().await.map_err(en_reponse)?;
    Ok(HttpResponse::Ok().json(entrees))
}

/// Référentiel des capacités transverses.
#[utoipa::path(
    operation_id = "referentiels_capacites",
    tag = "referentiels",
    responses(
        (status = 200, description = "Capacités connues", body = Vec<EntreeReferentiel>),
        (status = 401, description = "Non authentifié"),
    ),
    security(("bearer" = []))
)]
#[get("/capacites")]
pub async fn capacites(
    etat: web::Data<EtatApplication>,
    _contexte: ContexteAppel,
) -> Result<HttpResponse, actix_web::Error> {
    let entrees = etat.referentiel_capacites().await.map_err(en_reponse)?;
    Ok(HttpResponse::Ok().json(entrees))
}

/// Référentiel des profils de la capacité `STOCK`.
#[utoipa::path(
    operation_id = "referentiels_profils_stock",
    tag = "referentiels",
    responses(
        (status = 200, description = "Profils de stock connus", body = Vec<EntreeReferentiel>),
        (status = 401, description = "Non authentifié"),
    ),
    security(("bearer" = []))
)]
#[get("/profils-stock")]
pub async fn profils_stock(
    etat: web::Data<EtatApplication>,
    _contexte: ContexteAppel,
) -> Result<HttpResponse, actix_web::Error> {
    let entrees = etat.referentiel_profils().await.map_err(en_reponse)?;
    Ok(HttpResponse::Ok().json(entrees))
}

fn en_reponse(erreur: ErreurModules) -> actix_web::Error {
    crate::routes::erreurs::interne("lecture d'un référentiel", erreur)
}
