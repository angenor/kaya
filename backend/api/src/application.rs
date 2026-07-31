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

    /// Service des établissements — ETB-01.
    pub fn service_etablissement(
        &self,
    ) -> kaya_etablissements::etablissement::ServiceEtablissement<
        kaya_synchronisation::outbox::PgOutboxWriter,
    > {
        kaya_etablissements::etablissement::ServiceEtablissement::nouveau(
            self.pool.clone(),
            kaya_synchronisation::outbox::PgOutboxWriter::nouveau(),
        )
    }

    /// Service des modules d'activité — ETB-02, ETB-02b.
    ///
    /// # Le point d'accrochage des obstacles est construit ICI, et il est vide
    ///
    /// `backend/api/` est la famille « assemblage » — **le seul endroit du produit qui a le droit
    /// de connaître à la fois le socle et les verticales** (principe II). C'est donc ici que
    /// chaque verticale enregistrera son implémentation d'`ObstacleDesactivation`, par
    /// `.avec_obstacle(...)`, au cycle où elle crée des opérations en cours.
    ///
    /// Aucune n'en crée encore : la liste est vide et la désactivation est libre. C'est exact, pas
    /// un trou — et le poser maintenant évite qu'au cycle SEJ la voie facile soit d'ajouter une
    /// dépendance de `socle/etablissements` vers `verticales/hebergement` « juste cette fois »,
    /// exactement la faute que la porte P-03 attrape.
    pub fn service_modules(
        &self,
    ) -> kaya_etablissements::modules::ServiceModules<kaya_synchronisation::outbox::PgOutboxWriter>
    {
        kaya_etablissements::modules::ServiceModules::nouveau(
            self.pool.clone(),
            kaya_synchronisation::outbox::PgOutboxWriter::nouveau(),
        )
    }

    /// Service des points de vente — ETB-03.
    pub fn service_points_de_vente(
        &self,
    ) -> kaya_etablissements::points_de_vente::ServicePointsDeVente<
        kaya_synchronisation::outbox::PgOutboxWriter,
    > {
        kaya_etablissements::points_de_vente::ServicePointsDeVente::nouveau(
            self.pool.clone(),
            kaya_synchronisation::outbox::PgOutboxWriter::nouveau(),
        )
    }

    /// Service de la configuration héritée — ETB-04.
    pub fn service_configuration(
        &self,
    ) -> kaya_etablissements::configuration::ServiceConfiguration<
        kaya_synchronisation::outbox::PgOutboxWriter,
    > {
        kaya_etablissements::configuration::ServiceConfiguration::nouveau(
            self.pool.clone(),
            kaya_synchronisation::outbox::PgOutboxWriter::nouveau(),
        )
    }

    /// Service de l'identité visuelle — ETB-05.
    pub fn service_branding(
        &self,
    ) -> kaya_etablissements::branding::ServiceBranding<
        kaya_synchronisation::outbox::PgOutboxWriter,
    > {
        kaya_etablissements::branding::ServiceBranding::nouveau(
            self.pool.clone(),
            kaya_synchronisation::outbox::PgOutboxWriter::nouveau(),
        )
    }

    /// Accès au stockage objet — **via l'API S3 uniquement** (principe II).
    ///
    /// Construit à la demande comme les services : le client S3 est clonable et sans état de
    /// session. Le construire ici, plutôt que de le conserver dans l'état, garde visible le fait
    /// que le seul objet stocké par ce cycle est le logo d'identité visuelle.
    pub fn stockage(&self) -> crate::stockage::Stockage {
        crate::stockage::Stockage::depuis_environnement()
            .expect("configuration du stockage objet absente — voir S3_* dans backend/.env")
    }

    /// Le référentiel des modules, tel que l'API le rend.
    ///
    /// Les trois lectures de référentiel ne passent par aucun service : elles ne portent aucune
    /// règle métier, aucune transition d'état, aucun événement. Y interposer une couche service
    /// vide donnerait l'illusion qu'il s'y passe quelque chose.
    pub async fn referentiel_modules(
        &self,
    ) -> Result<
        Vec<crate::routes::referentiels::EntreeReferentiel>,
        kaya_etablissements::modules::ErreurModules,
    > {
        use crate::routes::referentiels::EntreeReferentiel;
        let mut tx = self.pool.begin().await?;
        let entrees = kaya_etablissements::modules::repository::referentiel_modules(&mut tx).await?;
        tx.rollback().await?;
        Ok(entrees
            .into_iter()
            .map(|e| EntreeReferentiel {
                code: e.code,
                libelle_cle: e.libelle_cle,
                implementee: e.implementee,
                ordre: e.ordre,
                motif_refus_cle: None,
            })
            .collect())
    }

    pub async fn referentiel_capacites(
        &self,
    ) -> Result<
        Vec<crate::routes::referentiels::EntreeReferentiel>,
        kaya_etablissements::modules::ErreurModules,
    > {
        use crate::routes::referentiels::EntreeReferentiel;
        let mut tx = self.pool.begin().await?;
        let entrees =
            kaya_etablissements::modules::repository::referentiel_capacites(&mut tx).await?;
        tx.rollback().await?;
        Ok(entrees
            .into_iter()
            .map(|e| EntreeReferentiel {
                code: e.code,
                libelle_cle: e.libelle_cle,
                implementee: e.implementee,
                ordre: e.ordre,
                motif_refus_cle: None,
            })
            .collect())
    }

    pub async fn referentiel_profils(
        &self,
    ) -> Result<
        Vec<crate::routes::referentiels::EntreeReferentiel>,
        kaya_etablissements::modules::ErreurModules,
    > {
        use crate::routes::referentiels::EntreeReferentiel;
        let mut tx = self.pool.begin().await?;
        let entrees = kaya_etablissements::modules::repository::referentiel_profils(&mut tx).await?;
        tx.rollback().await?;
        Ok(entrees
            .into_iter()
            .map(|e| EntreeReferentiel {
                code: e.code,
                libelle_cle: e.libelle_cle,
                implementee: e.implementee,
                ordre: e.ordre,
                motif_refus_cle: e.motif_refus_cle,
            })
            .collect())
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
