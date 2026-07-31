//! Contrat OpenAPI — **généré depuis le code, jamais écrit à la main** (principe I(a)).
//!
//! `specs/001-socle-technique-monorepo/contracts/http-api.md` décrit ce que ce code doit
//! produire ; il n'est jamais la référence. La référence, ce sont les annotations
//! `#[utoipa::path]` posées sur les handlers.

use utoipa::OpenApi;
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Kaya — API",
        description = "Plateforme de gestion pour établissements d'hébergement et de service.",
        version = "0.1.0",
    ),
    tags(
        (name = "systeme", description = "Sonde de santé et diagnostic."),
        (name = "etablissements", description = "Tenants, établissements et notes internes."),
        (name = "referentiels", description = "Référentiels globaux — lecture seule."),
        (name = "services", description = "Services d'un établissement et capacités déclarées."),
        (name = "branding", description = "Identité visuelle — surcharge partielle par établissement, aperçu sans enregistrement."),
        (name = "configuration", description = "Chaîne d'héritage de configuration — tenant, établissement, service, point de vente."),
        (name = "points-de-vente", description = "Points de vente et tables — un comptoir est un point de vente sans table."),
    ),
    modifiers(&SecuriteBearer),
)]
pub struct Contrat;

/// Déclare le schéma d'authentification une seule fois, au lieu de le répéter sur chaque route.
pub struct SecuriteBearer;

impl utoipa::Modify for SecuriteBearer {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(composants) = openapi.components.as_mut() {
            composants.add_security_scheme(
                "bearer",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}

/// Contrat de départ, enrichi par les routes montées.
pub fn contrat() -> utoipa::openapi::OpenApi {
    Contrat::openapi()
}
