//! Point d'entrée du serveur d'API.
//!
//! **Ordre imposé au démarrage** (R-12) :
//!
//!   1. journaux ;
//!   2. migrations, sous le rôle **propriétaire**, puis fermeture de cette connexion ;
//!   3. pool de runtime, sous le rôle **applicatif** ;
//!   4. **ensuite seulement**, ouverture du port d'écoute.
//!
//! Ouvrir le port avant la fin des migrations exposerait un service qui répond sur un schéma
//! partiellement migré — un état où les erreurs sont incompréhensibles et où le client n'a aucun
//! moyen de savoir qu'il doit réessayer.

use kaya_api::{application, contexte, db, observabilite};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Sonde de conteneur — `HEALTHCHECK` de `infra/Dockerfile.api`.
    //
    // Le drapeau évite d'installer `curl` ou `wget` dans l'image d'exécution : une image de
    // production ne porte que ce qui sert à servir. Le binaire connaît déjà son port ; lui faire
    // interroger sa propre sonde coûte quinze lignes et aucune dépendance.
    if std::env::args().any(|a| a == "--verifier-sante") {
        let port = std::env::var("KAYA_PORT").unwrap_or_else(|_| "8080".to_owned());
        let adresse = format!("127.0.0.1:{port}");
        return match std::net::TcpStream::connect(&adresse) {
            Ok(_) => Ok(()),
            Err(erreur) => {
                eprintln!("sonde : {adresse} injoignable — {erreur}");
                std::process::exit(1);
            }
        };
    }

    // Le fichier `.env` n'existe qu'en développement ; son absence n'est pas une erreur.
    if dotenvy::from_path("backend/.env").is_err() {
        let _ = dotenvy::dotenv();
    }

    observabilite::initialiser_journaux();

    // La garde doit vivre aussi longtemps que le processus : la lier à `_` la détruirait
    // immédiatement, ce qui coupe la remontée sans le moindre message.
    let _sentry = observabilite::initialiser_sentry();

    // Refus de démarrer si la dérogation d'authentification n'est pas ouverte
    // explicitement. Une dérogation qu'on peut oublier d'ouvrir se retrouve ouverte en
    // production sans que personne ne l'ait décidé.
    contexte::verifier_derogation();

    let pool_migrations = db::pool_migrations()
        .await
        .expect("connexion de migration impossible");
    db::appliquer_migrations(&pool_migrations)
        .await
        .expect("migrations en échec — le port ne sera pas ouvert");
    // Une connexion propriétaire conservée finirait tôt ou tard par servir une requête
    // ordinaire, et `FORCE ROW LEVEL SECURITY` perdrait tout intérêt.
    pool_migrations.close().await;

    let pool = db::pool_application()
        .await
        .expect("connexion applicative impossible");

    // Worker de publication — **in-process**, démarré avec le serveur (R-08). Aucune file
    // externe, aucun second processus à superviser : le paquet auto-hébergé (mode B) doit tenir
    // en un binaire et trois conteneurs.
    //
    // Son pool est celui de `kaya_worker`, jamais celui de l'application : la politique
    // `isolation_tenant` filtre sur un contexte que le worker ne pose pas, et il publierait
    // silencieusement zéro événement.
    match db::pool_worker().await {
        Ok(pool_worker) => {
            let worker = kaya_synchronisation::worker::WorkerPublication::nouveau(
                pool_worker,
                // Aucun consommateur à ce cycle : le grand livre est écrit et marqué publié, mais
                // rien n'en dérive encore. Les projections de pilotage viendront avec PIL.
                Vec::new(),
                kaya_synchronisation::worker::ConfigurationWorker::default(),
            );
            tokio::spawn(worker.boucler());
            tracing::info!("worker de publication démarré");
        }
        Err(erreur) => {
            // Un worker absent ne perd aucun événement : ils restent en attente, et un
            // redémarrage les reprendra. Refuser de servir pour autant priverait le pilote de
            // son outil de travail pour un défaut de configuration réparable à distance.
            tracing::error!(erreur = %erreur, "worker de publication NON démarré — les événements s'accumuleront en attente");
        }
    }

    let port: u16 = std::env::var("KAYA_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8080);

    tracing::info!(port, "ouverture du port d'écoute");
    application::servir(pool, port).await
}
