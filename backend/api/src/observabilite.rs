//! Journaux structurés et corrélation par requête.
//!
//! **Le support se fait à distance depuis Abidjan, à 220 km du pilote** (principe VIII). C'est
//! toute la raison d'être de ce module : sans identifiant de corrélation, diagnostiquer un
//! incident revient à demander au gérant de raconter ce qu'il a fait, ce qui ne marche jamais.

use std::future::{Ready, ready};
use std::sync::OnceLock;

use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::http::header::{HeaderName, HeaderValue};
use actix_web::{Error, HttpMessage};
use tracing_subscriber::EnvFilter;
use uuid::Uuid;

/// En-tête portant l'identifiant de corrélation, à l'aller comme au retour.
pub const EN_TETE_CORRELATION: &str = "x-kaya-correlation";

static INITIALISE: OnceLock<()> = OnceLock::new();

/// Identifiant de corrélation d'une requête, déposé dans ses extensions.
#[derive(Debug, Clone, Copy)]
pub struct IdCorrelation(pub Uuid);

/// Installe le souscripteur de journaux.
///
/// Idempotent : un second appel ne fait rien. Les tests d'intégration démarrent plusieurs
/// applications dans le même processus, et un souscripteur global posé deux fois provoque une
/// panique — un échec de test qui n'apprendrait rien sur le code testé.
pub fn initialiser_journaux() {
    INITIALISE.get_or_init(|| {
        let filtre = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("info,sqlx=warn"));

        // Sortie JSON : les journaux sont lus par un agrégateur, pas par un humain devant un
        // terminal. Une trace lisible à l'œil mais non analysable coûte le diagnostic.
        let _ = tracing_subscriber::fmt()
            .json()
            .with_env_filter(filtre)
            .with_current_span(true)
            .with_span_list(false)
            .try_init();
    });
}

/// Intergiciel de corrélation.
///
/// Reprend l'identifiant fourni par le client s'il est valide, en génère un sinon, et le renvoie
/// dans la réponse. **Reprendre celui du client est ce qui rend la corrélation utile hors
/// ligne** : un terminal qui rejoue sa file après une coupure porte les identifiants de ses
/// tentatives d'origine, et l'on peut relier la réussite d'aujourd'hui à l'échec d'hier.
pub struct Correlation;

impl<S, B> Transform<S, ServiceRequest> for Correlation
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = CorrelationService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(CorrelationService { service }))
    }
}

pub struct CorrelationService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for CorrelationService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>>>,
    >;

    actix_web::dev::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let correlation = req
            .headers()
            .get(EN_TETE_CORRELATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| Uuid::parse_str(v).ok())
            .unwrap_or_else(Uuid::now_v7);

        req.extensions_mut().insert(IdCorrelation(correlation));

        let methode = req.method().clone();
        let chemin = req.path().to_owned();
        let futur = self.service.call(req);

        Box::pin(async move {
            let portee = tracing::info_span!(
                "requete",
                correlation = %correlation,
                methode = %methode,
                chemin = %chemin,
            );
            let _entree = portee.enter();

            let mut reponse = futur.await?;

            if let Ok(valeur) = HeaderValue::from_str(&correlation.to_string()) {
                reponse.headers_mut().insert(
                    HeaderName::from_static(EN_TETE_CORRELATION),
                    valeur,
                );
            }
            Ok(reponse)
        })
    }
}

// =================================================================================================
//  Sentry — remontée des erreurs avec leur contexte
// =================================================================================================

/// Initialise Sentry si un DSN est configuré.
///
/// Renvoie une garde qu'il faut **conserver vivante pour toute la durée du processus** : sa
/// destruction vide la file d'envoi et coupe la remontée. La lier à une variable `_` la
/// détruirait immédiatement — erreur classique, silencieuse, et qui ne se voit qu'à l'absence
/// d'alertes.
///
/// # Ce qui ne part jamais chez Sentry
///
/// `send_default_pii` reste à **`false`** : ni adresse IP, ni identifiant de compte, ni corps de
/// requête. Le produit traite des pièces d'identité de clients et des montants ; le registre des
/// traitements ARTCI (TRX-06) n'a pas à couvrir un service tiers d'observation d'erreurs.
///
/// Les chaînes de connexion et les secrets ne partent pas non plus : ils ne sont ni dans les
/// messages d'erreur exposés, ni dans les champs de `tracing` — c'est pourquoi `db.rs` ne
/// journalise jamais l'URL qu'il consomme.
pub fn initialiser_sentry() -> Option<sentry::ClientInitGuard> {
    let dsn = std::env::var("SENTRY_DSN").ok().filter(|d| !d.is_empty())?;

    let environnement = std::env::var("KAYA_ENVIRONNEMENT")
        .unwrap_or_else(|_| "developpement".to_owned());

    // `ClientOptions` est `#[non_exhaustive]` : elle se construit par mutation d'un défaut, pas
    // par expression de structure. Une version future peut y ajouter un champ sans casser ce
    // code — c'est le but de l'attribut, et la raison pour laquelle le compilateur refuse la
    // forme littérale.
    let mut options = sentry::ClientOptions::default();
    options.release = Some(env!("CARGO_PKG_VERSION").into());
    options.environment = Some(environnement.into());
    // **Jamais de données personnelles.** Voir ci-dessus.
    options.send_default_pii = false;
    options.attach_stacktrace = true;

    let garde = sentry::init((dsn, options));

    tracing::info!("remontée des erreurs activée");
    Some(garde)
}
