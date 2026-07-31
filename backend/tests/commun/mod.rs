//! Support commun des tests d'intégration.
//!
//! **Les tests de ce répertoire s'exécutent sur une base PostgreSQL réelle, jamais sur un
//! simulacre.** La constitution (principe VIII) exige des tests d'intégration sur les transitions
//! d'état, et l'essentiel de ce que ce cycle garantit — sécurité au niveau ligne, immuabilité par
//! déclencheur, contraintes d'exclusion, privilèges par rôle — n'existe que dans la base. Un
//! simulacre validerait le code en laissant la garantie non testée.

#![allow(dead_code)]

use sqlx::postgres::{PgPool, PgPoolOptions};
use uuid::Uuid;

/// Chaîne de connexion du rôle **propriétaire** — migrations et préparation des jeux d'essai.
pub fn url_owner() -> String {
    variable("DATABASE_URL")
}

/// Chaîne de connexion du rôle **applicatif** — soumis à la sécurité au niveau ligne.
pub fn url_app() -> String {
    variable("DATABASE_URL_APP")
}

/// Chaîne de connexion du rôle **lecteur du grand livre** — `SELECT` sur `evenement_outbox`, et
/// rien d'autre. C'est l'absence de tout autre droit qui fait la démonstration (R-11).
pub fn url_ledger() -> String {
    variable("DATABASE_URL_LEDGER")
}

fn variable(nom: &str) -> String {
    charger_env();
    std::env::var(nom).unwrap_or_else(|_| {
        panic!(
            "{nom} doit être définie. Lancer `docker compose -f infra/compose.yml up -d` puis \
             `scripts/dev/preparer-base.sh`."
        )
    })
}

fn charger_env() {
    use std::sync::Once;
    static UNE_FOIS: Once = Once::new();
    UNE_FOIS.call_once(|| {
        // Le fichier vit à la racine du workspace Rust ; les tests s'exécutent depuis le
        // répertoire du paquet, qui est le même ici.
        let _ = dotenvy::from_filename(".env");
    });
}

pub async fn pool_owner() -> PgPool {
    connecter(&url_owner()).await
}

pub async fn pool_app() -> PgPool {
    connecter(&url_app()).await
}

pub async fn pool_ledger() -> PgPool {
    connecter(&url_ledger()).await
}

async fn connecter(url: &str) -> PgPool {
    PgPoolOptions::new()
        .max_connections(4)
        .connect(url)
        .await
        .expect("connexion à la base de test impossible")
}

/// Monte **l'application réelle**, celle que sert le binaire.
///
/// Le montage passe par `kaya_api::routes::configurer`, la même fonction qu'appelle
/// `application::servir`. Un test qui déclarerait ses propres routes prouverait quelque chose sur
/// lui-même et rien sur le service : c'est exactement le trou que la porte P-08 cherche à fermer.
#[macro_export]
macro_rules! monter_application {
    ($pool:expr) => {{
        use actix_web::{App, web};
        use utoipa_actix_web::AppExt;

        let (app, _contrat) = App::new()
            .app_data(web::Data::new(kaya_api::application::EtatApplication {
                pool: $pool,
            }))
            .into_utoipa_app()
            .openapi(kaya_api::openapi::contrat())
            .configure(kaya_api::routes::configurer)
            .split_for_parts();

        actix_web::test::init_service(app).await
    }};
}

/// Un tenant et son établissement, créés pour un test.
#[derive(Debug, Clone, Copy)]
pub struct JeuTenant {
    pub tenant_id: Uuid,
    pub etablissement_id: Uuid,
}

/// Crée un tenant et un établissement isolés, sous le rôle propriétaire.
///
/// Les identifiants sont des **UUID v7 générés côté client** (principe VI) : c'est ce qui rend le
/// rejeu inoffensif partout ailleurs, et les tests n'ont pas de raison de faire autrement.
///
/// Le contexte de tenant est posé même sous le rôle propriétaire : `FORCE ROW LEVEL SECURITY`
/// s'applique aussi à lui, et sans contexte l'insertion échouerait sur `WITH CHECK` — ce qui est
/// exactement le comportement voulu.
pub async fn creer_tenant(pool: &PgPool, nom: &str) -> JeuTenant {
    let tenant_id = Uuid::now_v7();
    let etablissement_id = Uuid::now_v7();

    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, tenant_id)
        .await
        .expect("pose du tenant");

    sqlx::query!(
        "INSERT INTO etablissements.tenant (id, nom) VALUES ($1, $2)",
        tenant_id,
        nom
    )
    .execute(&mut *tx)
    .await
    .expect("insertion du tenant");

    sqlx::query!(
        r#"
        INSERT INTO etablissements.etablissement (id, tenant_id, nom, fuseau_horaire, devise)
        VALUES ($1, $2, $3, 'Africa/Abidjan', 'XOF')
        "#,
        etablissement_id,
        tenant_id,
        format!("{nom} — établissement")
    )
    .execute(&mut *tx)
    .await
    .expect("insertion de l'établissement");

    tx.commit().await.expect("commit");

    JeuTenant {
        tenant_id,
        etablissement_id,
    }
}
