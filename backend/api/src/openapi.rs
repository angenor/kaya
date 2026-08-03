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
        (name = "comptes", description = "Personnes, comptes et rôles cumulables — les permissions sont l'UNION des rôles portés."),
        (name = "session", description = "Connexion, rafraîchissement et déconnexion à distance. Les deux seules opérations publiques du produit sont ici."),
        (name = "journal-audit", description = "Registre des actions — lecture filtrée seulement. Aucun point d'entrée d'écriture, par décision."),
        (name = "clients", description = "Fiches clients — du TENANT, jamais d'un établissement (FR-002). La recherche sert trois formes par une seule entrée : nom, téléphone, numéro de pièce. Le numéro de pièce d'identité est chiffré au repos et sa consultation journalisée au registre des actions."),
        (name = "sejours", description = "Séjours — arrivée, départ, prolongation, changement de chambre. L'ouverture est UN appel et UNE transaction : attribution, séjour, note, fiche de police et événement. La double attribution est impossible par la contrainte d'exclusion, jamais par un verrou applicatif."),
        (name = "hebergement", description = "Types de chambre, chambres, formules de location et moteur de disponibilité. La double attribution est rendue impossible par une contrainte d'exclusion PostgreSQL, jamais par un verrou applicatif."),
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
