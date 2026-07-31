//! Connexions à la base — **deux configurations distinctes, jamais une seule élargie**.
//!
//! C'est la conséquence la plus facile à manquer de R-12 : les migrations s'exécutent au
//! démarrage sous `kaya_owner`, mais le pool de runtime tourne sous `kaya_app`.
//!
//! Un pool unique disposant des droits nécessaires aux migrations annulerait l'intérêt de
//! `FORCE ROW LEVEL SECURITY` : le propriétaire des tables est précisément le rôle que `FORCE`
//! sert à contraindre, et l'exécution ordinaire des requêtes passerait sous lui.

use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

/// Erreur de configuration ou d'ouverture d'une connexion.
#[derive(Debug, thiserror::Error)]
pub enum ErreurBase {
    #[error("variable d'environnement absente : {0}")]
    VariableAbsente(&'static str),

    #[error("connexion à la base impossible : {0}")]
    Connexion(#[from] sqlx::Error),

    #[error("migrations : {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),
}

fn variable(nom: &'static str) -> Result<String, ErreurBase> {
    std::env::var(nom).map_err(|_| ErreurBase::VariableAbsente(nom))
}

/// Pool du **runtime**, sous `kaya_app`.
///
/// Soumis à la sécurité au niveau ligne. C'est par lui que passe toute requête servie à un
/// client, sans exception.
pub async fn pool_application() -> Result<PgPool, ErreurBase> {
    let url = variable("DATABASE_URL_APP")?;
    Ok(PgPoolOptions::new()
        .max_connections(16)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&url)
        .await?)
}

/// Connexion des **migrations**, sous `kaya_owner`.
///
/// Ouverte au démarrage, utilisée une fois, refermée. Elle n'est jamais conservée dans l'état de
/// l'application : une connexion propriétaire qui traînerait finirait par servir une requête
/// ordinaire.
pub async fn pool_migrations() -> Result<PgPool, ErreurBase> {
    let url = variable("DATABASE_URL")?;
    Ok(PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_secs(10))
        .connect(&url)
        .await?)
}

/// Pool du **worker de publication**, sous `kaya_worker`.
///
/// Rôle distinct des trois autres, introduit par la migration 0005 : il lit `evenement_outbox`
/// tous tenants confondus — ce que la politique `isolation_tenant` interdit aux autres — et ne
/// peut écrire que la colonne `publie_le`.
pub async fn pool_worker() -> Result<PgPool, ErreurBase> {
    let url = variable("DATABASE_URL_WORKER")?;
    Ok(PgPoolOptions::new()
        .max_connections(2)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&url)
        .await?)
}

/// Applique les migrations, **avant l'ouverture du port d'écoute** (R-12).
///
/// Deux instances qui démarrent en même temps appliqueraient les migrations en concurrence ;
/// sqlx pose pour cela un verrou consultatif. Le comportement est **vérifié par un test de
/// démarrage concurrent**, pas supposé — c'est la différence entre une garantie et une lecture
/// de documentation.
#[tracing::instrument(skip(pool))]
pub async fn appliquer_migrations(pool: &PgPool) -> Result<(), ErreurBase> {
    tracing::info!("application des migrations sous le rôle propriétaire");
    migrateur().run(pool).await?;
    tracing::info!("migrations appliquées");
    Ok(())
}

/// Le migrateur, exposé pour que les tests s'exercent sur **celui du binaire**.
///
/// Un test qui construirait son propre migrateur ne dirait rien du démarrage réel : c'est
/// justement la configuration — nom et schéma de la table de suivi, lus dans `sqlx.toml` — qui a
/// déjà produit un échec silencieux, la macro et l'outil en ligne de commande tenant chacun leur
/// propre table.
pub fn migrateur() -> sqlx::migrate::Migrator {
    sqlx::migrate!("../migrations")
}
