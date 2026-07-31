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

impl EtatApplication {
    /// Construit le service des notes.
    ///
    /// Le service est **assemblé à la demande** plutôt que conservé dans l'état : il ne détient
    /// qu'un pool clonable et un écrivain sans état, donc le construire ne coûte rien, et
    /// l'injection de l'écrivain d'outbox reste visible ici — là où l'on cherchera un jour à
    /// savoir par où passent les événements.
    pub fn service_note(
        &self,
    ) -> kaya_etablissements::note::ServiceNote<kaya_synchronisation::outbox::PgOutboxWriter> {
        kaya_etablissements::note::ServiceNote::nouveau(
            self.pool.clone(),
            kaya_synchronisation::outbox::PgOutboxWriter::nouveau(),
        )
    }
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

/// Le contrat OpenAPI **assemblé**, routes montées comprises.
///
/// # Pourquoi cette fonction existe
///
/// `openapi::contrat()` ne renvoie que le squelette déclaré par la macro `#[derive(OpenApi)]` :
/// titre, étiquettes, schéma d'authentification. Les chemins, eux, sont collectés par
/// `utoipa-actix-web` **au montage des routes** — donc seulement dans `split_for_parts()`.
///
/// La distinction a failli rendre la porte P-08 muette : paramétrée sur le squelette, elle
/// constatait zéro route et passait au vert alors que deux endpoints étaient servis. Une porte
/// qui ne trouve jamais rien est indistinguable d'une porte qui n'a rien à trouver.
///
/// Le contrat est donc extrait en construisant une application jetable — le même montage que
/// `servir`, sans écoute. C'est ce que consomment la porte P-08 et la génération du client
/// TypeScript.
pub fn contrat_complet() -> utoipa::openapi::OpenApi {
    let (_, contrat) = App::new()
        .into_utoipa_app()
        .openapi(crate::openapi::contrat())
        .configure(crate::routes::configurer)
        .split_for_parts();
    contrat
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
