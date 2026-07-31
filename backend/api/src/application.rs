//! Construction de l'application Actix.
//!
//! Une seule fonction construit les routes, et **les tests d'intégration l'appellent**. Un test
//! qui déclarerait ses propres routes prouverait quelque chose sur lui-même, pas sur le service
//! réellement servi — et la porte P-08, qui vise chaque endpoint du contrat, deviendrait
//! contournable en oubliant d'y inscrire une route.

use actix_web::{App, HttpServer, web};
use sqlx::PgPool;
use utoipa_actix_web::AppExt;
use utoipa_swagger_ui::SwaggerUi;

use crate::observabilite::Correlation;

/// État partagé injecté dans chaque handler.
#[derive(Clone)]
pub struct EtatApplication {
    /// Pool **applicatif**, sous `kaya_app`, soumis à la sécurité au niveau ligne.
    pub pool: PgPool,
}

/// Swagger UI est-elle montée ?
///
/// **Décision de configuration au démarrage, jamais un test de variable dispersé dans les
/// handlers** (FR-032, `contracts/http-api.md` §3). Une route non montée ne peut pas fuir par
/// oubli de garde ; une route montée derrière un `if` finit toujours par être atteinte par un
/// chemin qu'on n'avait pas prévu.
pub fn swagger_ui_activee() -> bool {
    matches!(
        std::env::var("KAYA_SWAGGER_UI").as_deref(),
        Ok("1") | Ok("true")
    )
}

/// Démarre le serveur HTTP.
pub async fn servir(pool: PgPool, port: u16) -> std::io::Result<()> {
    let etat = EtatApplication { pool };
    let monter_swagger = swagger_ui_activee();

    if monter_swagger {
        tracing::warn!(
            "Swagger UI est montée — attendu en développement, jamais en production (FR-032)"
        );
    }

    HttpServer::new(move || {
        let (app, api) = App::new()
            .wrap(Correlation)
            .app_data(web::Data::new(etat.clone()))
            .into_utoipa_app()
            .openapi(crate::openapi::contrat())
            .configure(crate::routes::configurer)
            .split_for_parts();

        // Le contrat lui-même reste toujours exposé : c'est la source de vérité du client
        // généré (principe I(a)), et le publier n'ouvre aucun accès aux données.
        let app = app.route(
            "/api-docs/openapi.json",
            web::get().to({
                let api = api.clone();
                move || {
                    let api = api.clone();
                    async move { actix_web::HttpResponse::Ok().json(api) }
                }
            }),
        );

        if monter_swagger {
            app.service(SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", api))
        } else {
            app
        }
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
