//! Routes HTTP.
//!
//! Une seule fonction les monte, et c'est elle que montent aussi les tests d'intégration : la
//! porte P-08 est **paramétrée sur la liste des routes du contrat OpenAPI**, de sorte qu'un
//! endpoint ajouté sans test d'isolation fasse échouer la porte au lieu de passer inaperçu.

use utoipa_actix_web::service_config::ServiceConfig;

pub fn configurer(config: &mut ServiceConfig) {
    let _ = config;
}
