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
///
/// # Trois champs, et deux sont nouveaux depuis CPT-01
///
/// L'entrepôt des sessions et la clé de signature ne sont pas là par commodité : ils sont sur le
/// chemin de **chaque requête authentifiée**, puisque `ContexteAppel` vérifie la signature du
/// jeton et consulte la liste de révocation avant de laisser passer un handler. Les construire à
/// la demande, comme les services, coûterait un client Redis par requête.
#[derive(Clone)]
pub struct EtatApplication {
    /// Pool **applicatif**, sous `kaya_app`, soumis à la sécurité au niveau ligne.
    pub pool: PgPool,
    /// Entrepôt des sessions — **la liste de révocation est consultée à chaque requête**.
    pub entrepot: kaya_comptes::session::Entrepot,
    /// Compteur de tentatives de connexion — deux clés, l'identifiant et l'origine.
    pub limite: kaya_comptes::session::LimiteTentatives,
    /// Clé de signature des jetons, lue de l'environnement au démarrage (principe IX).
    pub cle_jwt: Vec<u8>,
}

impl EtatApplication {
    /// Assemble l'état depuis l'environnement — **la seule fabrique du produit**.
    ///
    /// Les tests d'intégration l'appellent aussi : un état monté autrement en test prouverait
    /// quelque chose sur lui-même et rien sur le service servi, exactement comme une route
    /// déclarée hors de `routes::configurer`.
    pub fn depuis_environnement(pool: PgPool) -> Result<Self, String> {
        let url_redis = std::env::var("REDIS_URL").map_err(|_| {
            "REDIS_URL est absente. Depuis CPT-01, Redis n'est plus optionnel : la liste de \
             révocation est consultée à chaque requête authentifiée, et sans elle aucune session \
             ne peut être coupée avant son expiration — jusqu'à 90 jours pour un jeton de \
             rafraîchissement."
                .to_owned()
        })?;

        Ok(Self {
            pool,
            entrepot: kaya_comptes::session::Entrepot::nouveau(&url_redis)
                .map_err(|e| format!("entrepôt des sessions injoignable : {e}"))?,
            limite: kaya_comptes::session::LimiteTentatives::nouveau(&url_redis)
                .map_err(|e| format!("compteur de tentatives injoignable : {e}"))?,
            cle_jwt: crate::secrets::cle_jwt().map_err(|e| e.to_string())?,
        })
    }

    /// Service d'authentification et de session — CPT-01.
    pub fn service_authentification(
        &self,
    ) -> kaya_comptes::authentification::ServiceAuthentification<
        kaya_synchronisation::outbox::PgOutboxWriter,
        kaya_comptes::audit::JournalAuditPostgres,
    > {
        kaya_comptes::authentification::ServiceAuthentification::nouveau(
            self.pool.clone(),
            self.entrepot.clone(),
            self.limite.clone(),
            self.cle_jwt.clone(),
            kaya_synchronisation::outbox::PgOutboxWriter::nouveau(),
            kaya_comptes::audit::JournalAuditPostgres,
        )
    }

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

    /// Service des comptes — CPT-01, opérations 10 à 14.
    ///
    /// L'entrepôt des sessions y entre pour **une seule raison** : le changement de mot de passe
    /// coupe les autres sessions, immédiatement. Sans lui, le geste qu'on fait quand on soupçonne
    /// que quelqu'un a son mot de passe laisserait les sessions ouvertes jusqu'à quatre-vingt-dix
    /// jours.
    pub fn service_comptes(
        &self,
    ) -> kaya_comptes::compte::ServiceComptes<
        kaya_synchronisation::outbox::PgOutboxWriter,
        kaya_comptes::audit::JournalAuditPostgres,
    > {
        kaya_comptes::compte::ServiceComptes::nouveau(
            self.pool.clone(),
            kaya_synchronisation::outbox::PgOutboxWriter::nouveau(),
            kaya_comptes::audit::JournalAuditPostgres,
            self.entrepot.clone(),
        )
    }

    /// Service des rôles — CPT-02, opérations 15 et 16.
    ///
    /// # Pourquoi celui-ci prend un `tenant_id` et les autres non
    ///
    /// Il détient un `EstablishmentDirectory`, par lequel l'attribution vérifie qu'un
    /// établissement existe **sans jointure inter-schémas** (principe II, porte P-04). Ce trait
    /// porte son tenant dans l'instance et non dans ses signatures — décision du cycle 002, prise
    /// pour qu'un consommateur qui demande « l'établissement 42 » n'ait pas à connaître le
    /// contexte d'authentification. Le prix est ici : le service se construit par requête.
    pub fn service_roles(
        &self,
        tenant_id: uuid::Uuid,
    ) -> kaya_comptes::roles::ServiceRoles<
        kaya_synchronisation::outbox::PgOutboxWriter,
        kaya_comptes::audit::JournalAuditPostgres,
        kaya_etablissements::etablissement::PgEstablishmentDirectory,
    > {
        kaya_comptes::roles::ServiceRoles::nouveau(
            self.pool.clone(),
            kaya_synchronisation::outbox::PgOutboxWriter::nouveau(),
            kaya_comptes::audit::JournalAuditPostgres,
            kaya_etablissements::etablissement::PgEstablishmentDirectory::nouveau(
                self.pool.clone(),
                tenant_id,
            ),
        )
    }

    /// Le même service, pour les **deux référentiels globaux**.
    ///
    /// Les référentiels des rôles et des permissions sont sans `tenant_id` : ils rendent la même
    /// chose à tout le monde. Un tenant nul y est donc exact, et non un contournement — mais il
    /// mérite d'être nommé, sans quoi le prochain lecteur croirait à une faute.
    pub fn service_roles_lecture(
        &self,
    ) -> kaya_comptes::roles::ServiceRoles<
        kaya_synchronisation::outbox::PgOutboxWriter,
        kaya_comptes::audit::JournalAuditPostgres,
        kaya_etablissements::etablissement::PgEstablishmentDirectory,
    > {
        self.service_roles(uuid::Uuid::nil())
    }

    /// Lit une page du registre des actions — CPT-04, opération 19.
    ///
    /// La lecture ne passe **par aucun service** : elle ne porte aucune règle métier, aucune
    /// transition d'état, aucun événement. Y interposer une couche vide donnerait l'illusion qu'il
    /// s'y passe quelque chose — même raisonnement que les trois référentiels du cycle 002.
    ///
    /// La transaction est **annulée** : c'est une lecture, et un `commit` sur une transaction sans
    /// écriture ne dirait rien de plus tout en laissant croire le contraire.
    pub async fn lire_journal_audit(
        &self,
        tenant_id: uuid::Uuid,
        filtres: &kaya_comptes::audit::FiltresAudit,
        curseur: Option<kaya_comptes::audit::Curseur>,
        limite: i64,
    ) -> Result<kaya_comptes::audit::PageAudit, kaya_comptes::audit::ErreurAudit> {
        let mut tx = self.pool.begin().await?;
        kaya_etablissements::tenant_context::poser_tenant(&mut tx, tenant_id)
            .await
            .map_err(|e| match e {
                kaya_etablissements::tenant_context::ErreurContexteTenant::Base(e) => {
                    kaya_comptes::audit::ErreurAudit::Base(e)
                }
            })?;
        let page =
            kaya_comptes::audit::repository::lister(&mut tx, filtres, curseur, limite).await?;
        tx.rollback().await?;
        Ok(page)
    }

    /// Contrôle d'accès et annuaire des comptes — les deux traits de `socle/comptes`.
    ///
    /// Employé par la lecture du registre des actions, qui résout ses auteurs **en lot**.
    pub fn annuaire_comptes(&self) -> kaya_comptes::ControleAccesPostgres {
        kaya_comptes::ControleAccesPostgres::nouveau(self.pool.clone())
    }

    /// Service de l'identité civile — CPT-00.
    pub fn service_personne(
        &self,
    ) -> kaya_comptes::personne::ServicePersonne<kaya_synchronisation::outbox::PgOutboxWriter> {
        kaya_comptes::personne::ServicePersonne::nouveau(
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

/// Origines autorisées à appeler l'API depuis un navigateur.
///
/// # Pourquoi CORS existe ici, et pourquoi jamais `*`
///
/// L'application est une **SPA** servie depuis une autre origine que l'API : `localhost:3000` en
/// développement, `tauri://localhost` sous Tauri (principe VII, mode SPA). Sans en-têtes CORS, le
/// navigateur bloque chaque appel et **aucun écran ne fonctionne** — le cycle 001 ne pouvait pas
/// le rencontrer, n'ayant aucun écran.
///
/// `Access-Control-Allow-Origin: *` réglerait le symptôme et ouvrirait l'API à toute page web que
/// l'utilisateur visite. La liste est donc **explicite et configurable**, et son défaut ne contient
/// que des origines locales : une installation de production qui n'aurait pas réglé
/// `KAYA_ORIGINES_AUTORISEES` refuse les navigateurs plutôt que de les accepter tous.
fn origines_autorisees() -> Vec<String> {
    std::env::var("KAYA_ORIGINES_AUTORISEES")
        .map(|v| {
            v.split(',')
                .map(str::trim)
                .filter(|o| !o.is_empty())
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_else(|_| {
            vec![
                "http://localhost:3000".to_owned(),
                "http://localhost:3100".to_owned(),
                // Origine de l'application empaquetée par Tauri. Elle diffère par plateforme —
                // `tauri://localhost` sur macOS et iOS, `http://tauri.localhost` sur Windows et
                // Android — d'où les deux formes.
                "tauri://localhost".to_owned(),
                "http://tauri.localhost".to_owned(),
            ]
        })
}

/// Construit la politique CORS.
///
/// Extraite pour être **testable sans lever de serveur** : une politique qui n'autorise pas les
/// en-têtes de contexte laisserait chaque appel échouer au préflight, et le symptôme — « l'écran
/// ne charge rien » — ne dirait pas d'où il vient.
pub fn politique_cors() -> actix_cors::Cors {
    let mut cors = actix_cors::Cors::default()
        .allowed_methods(vec!["GET", "POST", "PUT", "PATCH", "DELETE", "OPTIONS"])
        .allowed_headers(vec![
            actix_web::http::header::CONTENT_TYPE,
            actix_web::http::header::AUTHORIZATION,
        ])
        // Les deux en-têtes du provisoire `CONTEXTE_PAR_EN_TETES` ont disparu avec lui (T030) :
        // le contexte vient désormais du jeton, porté par `Authorization`, déjà déclaré ci-dessus.
        // Une heure de cache de préflight : au-delà, on rejoue un aller-retour par requête sur un
        // réseau que la persona Aminata n'a pas.
        .max_age(3600);

    for origine in origines_autorisees() {
        cors = cors.allowed_origin(&origine);
    }
    cors
}

/// Démarre le serveur HTTP.
pub async fn servir(etat: EtatApplication, port: u16) -> std::io::Result<()> {
    let monter_swagger = swagger_ui_activee();

    if monter_swagger {
        tracing::warn!(
            "Swagger UI est montée — attendu en développement, jamais en production (FR-032)"
        );
    }

    HttpServer::new(move || {
        let (app, api) = App::new()
            .wrap(politique_cors())
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
