//! Routes HTTP.
//!
//! Une seule fonction les monte, et c'est celle que montent aussi les tests d'intégration : la
//! porte P-08 est **paramétrée sur la liste des routes du contrat OpenAPI**, de sorte qu'un
//! endpoint ajouté sans régime d'isolation déclaré fasse échouer la porte au lieu de passer
//! inaperçu.
//!
//! `utoipa-actix-web` ne collecte les chemins que depuis les appels `service(...)` — jamais
//! depuis `route(...)`. Un endpoint monté par `route(...)` serait servi **sans figurer au
//! contrat**, donc absent du client généré et invisible pour P-08. C'est la raison pour laquelle
//! chaque handler porte son propre attribut de routage.

pub mod notes;
pub mod sante;

use utoipa_actix_web::service_config::ServiceConfig;

pub fn configurer(config: &mut ServiceConfig) {
    // Sonde de santé — publique, sans contexte de tenant, hors de tout préfixe de version : la
    // supervision externe doit pouvoir l'interroger sans rien savoir du produit.
    config.service(sante::sante);

    config.service(
        utoipa_actix_web::scope::scope("/api/v1/etablissements/{etablissement_id}/notes")
            .service(notes::lister)
            .service(notes::creer),
    );
}
