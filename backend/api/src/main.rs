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
    // Le fichier `.env` n'existe qu'en développement ; son absence n'est pas une erreur.
    if dotenvy::from_path("backend/.env").is_err() {
        let _ = dotenvy::dotenv();
    }

    observabilite::initialiser_journaux();

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

    let port: u16 = std::env::var("KAYA_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8080);

    tracing::info!(port, "ouverture du port d'écoute");
    application::servir(pool, port).await
}
