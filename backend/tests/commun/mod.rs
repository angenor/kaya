//! Support commun des tests d'intégration.
//!
//! **Les tests de ce répertoire s'exécutent sur une base PostgreSQL réelle, jamais sur un
//! simulacre.** La constitution (principe VIII) exige des tests d'intégration sur les transitions
//! d'état, et l'essentiel de ce que ce cycle garantit — sécurité au niveau ligne, immuabilité par
//! déclencheur, contraintes d'exclusion, privilèges par rôle — n'existe que dans la base. Un
//! simulacre validerait le code en laissant la garantie non testée.

#![allow(dead_code)]

use std::str::FromStr;

use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};
use uuid::Uuid;

/// `application_name` porté par **toutes** les connexions des tests.
///
/// C'est ce qui rend une session de test distinguable d'une session d'un autre processus, et
/// c'est la seule chose qui le permette : `pg_stat_activity` masque `application_name` et `state`
/// des backends appartenant à un **autre rôle**, mais les expose pour ceux du **même rôle**. Le
/// contrôle de `exiger_grand_livre_sans_consommateur_concurrent` s'appuie exactement là-dessus.
pub const NOM_APPLICATION_TESTS: &str = "kaya-tests";

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

/// Chaîne de connexion du rôle **worker de publication** — lecture tous tenants sur le seul
/// grand livre, marquage `publie_le` seul (migration 0005).
pub fn url_worker() -> String {
    variable("DATABASE_URL_WORKER")
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

pub async fn pool_worker() -> PgPool {
    connecter(&url_worker()).await
}

async fn connecter(url: &str) -> PgPool {
    let options = PgConnectOptions::from_str(url)
        .expect("URL de connexion illisible")
        .application_name(NOM_APPLICATION_TESTS);

    PgPoolOptions::new()
        .max_connections(4)
        .connect_with(options)
        .await
        .expect("connexion à la base de test impossible")
}

/// Refuse de dérouler un test dont les décomptes exigent d'être **seul consommateur du grand
/// livre**, si un autre processus en consomme au même instant.
///
/// # Le défaut que ce contrôle remplace
///
/// `worker_redemarrage.rs` et `outbox_immuabilite.rs` échouaient environ deux fois sur cinq en
/// `cargo test --workspace`, et jamais en isolation ni en intégration continue. La cause n'était
/// ni dans le worker ni dans la rédaction des tests : **le binaire de développement était en
/// train de tourner**. Son worker in-process (`api/src/main.rs`) démarre avec
/// `ConfigurationWorker::default()`, donc `tenant: None` — **tous les tenants** — et boucle toutes
/// les 500 ms sur la même base que les tests. Il publiait les événements d'un test entre leur
/// création et le premier `traiter_un_lot()`, et le test comptait alors moins d'événements que
/// prévu. En intégration continue le défaut est hors d'atteinte : chaque job monte un PostgreSQL
/// neuf, sans binaire qui tourne.
///
/// **Le worker faisait son travail, correctement.** Le filtre par tenant est posé au moment de la
/// SÉLECTION du lot, dans le même `WHERE` que `publie_le IS NULL` et donc avant
/// `FOR UPDATE SKIP LOCKED` : un worker restreint à un tenant ne prélève jamais la ligne d'un
/// autre. Ce qui était faux, c'est l'hypothèse implicite des tests — *je suis le seul consommateur
/// de mes événements* — que rien n'écrivait ni ne vérifiait.
///
/// # Pourquoi ce contrôle plutôt qu'une sérialisation
///
/// Sérialiser les tests masquerait le défaut sans l'expliquer, et surtout ne le corrigerait pas :
/// le worker concurrent n'est pas un test, aucun `--test-threads=1` ne l'arrête. Aucune rédaction
/// des tests ne peut d'ailleurs les rendre déterministes face à un consommateur non contrôlé —
/// ils observent un **partage de travail**, et un tiers change le partage. Le déterminisme exige
/// d'écarter le tiers ; il ne reste qu'à le constater plutôt qu'à le subir.
///
/// # Périmètre inspecté, et ce qu'il ne couvre pas
///
/// Sont comptées les sessions du rôle `kaya_worker` sur la base courante dont
/// l'`application_name` n'est pas [`NOM_APPLICATION_TESTS`]. Deux limites, assumées :
///
///   * un worker démarré **après** ce contrôle passe au travers — la fenêtre est celle du test,
///     pas celle du processus ;
///   * l'identification se fait par `application_name`, pas par la configuration réelle du tiers :
///     rien ne permet de lire le `tenant:` d'un autre processus. Un worker étranger correctement
///     restreint à un autre tenant serait donc refusé alors qu'il serait inoffensif. Le faux
///     positif coûte un message ; le faux négatif coûtait deux jours.
///
/// Le contrôle ne concerne **pas** les autres fichiers de test : leurs décomptes sont filtrés par
/// tenant et un consommateur concurrent ne les change pas.
pub async fn exiger_grand_livre_sans_consommateur_concurrent() {
    // La sonde passe par le pool `kaya_worker` et non par le propriétaire : `pg_stat_activity`
    // masque `application_name` des backends d'un autre rôle. Interrogé sous `kaya_owner`, le
    // contrôle ne verrait que des noms vides et ne distinguerait plus ses propres sessions.
    let pool = pool_worker().await;

    let etrangers: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) AS "c!"
        FROM pg_stat_activity
        WHERE datname = current_database()
          AND usename = 'kaya_worker'
          AND coalesce(application_name, '') <> $1
        "#,
        NOM_APPLICATION_TESTS
    )
    .fetch_one(&pool)
    .await
    .expect("lecture de pg_stat_activity");

    assert_eq!(
        etrangers, 0,
        "{etrangers} session(s) du rôle `kaya_worker` étrangères aux tests sont ouvertes sur cette \
         base. Un worker de publication tourne donc en dehors de `cargo test`, presque toujours \
         le binaire de développement (`cargo run -p kaya-api`), dont le worker publie TOUS les \
         tenants toutes les 500 ms. Il consommerait les événements de ce test avant lui et les \
         décomptes seraient faux, au hasard de l'ordonnancement.\n\
         \n\
         Remède : arrêter le binaire de développement le temps des tests, ou faire pointer les \
         variables DATABASE_URL* sur une base dédiée aux tests.\n\
         \n\
         Ce n'est pas un défaut du worker : son filtre par tenant est posé à la sélection du lot, \
         et deux workers concurrents se partagent le grand livre sans perte ni doublon. C'est ce \
         test qui exige d'être seul consommateur de ses propres événements."
    );
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

/// URL de l'entrepôt des sessions.
pub fn url_redis() -> String {
    variable("REDIS_URL")
}

/// Clé de signature des jetons, telle que le serveur la lit.
pub fn cle_jwt() -> Vec<u8> {
    variable("KAYA_JWT_CLE").into_bytes()
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

    // `commune` est `NOT NULL` sans défaut depuis la migration 0007 : le défaut n'y servait qu'à
    // remplir les lignes existantes et a été retiré aussitôt, pour qu'aucune création ultérieure
    // ne puisse l'omettre en silence. Le classement reste `NON_CLASSE`, valeur par défaut : les
    // tests qui ont besoin d'un classement étoilé le posent eux-mêmes.
    sqlx::query!(
        r#"
        INSERT INTO etablissements.etablissement
            (id, tenant_id, nom, fuseau_horaire, devise, commune)
        VALUES ($1, $2, $3, 'Africa/Abidjan', 'XOF', 'Abengourou')
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

/// Une personne, un compte, et les rôles qu'on lui donne — **par le chemin réel**.
///
/// Le condensat est produit par `kaya_comptes::authentification::hacher`, jamais écrit à la main :
/// un condensat figé dans le code cesserait de correspondre le jour où les paramètres montent, et
/// les tests échoueraient sans dire pourquoi.
#[derive(Debug, Clone, Copy)]
pub struct JeuCompte {
    pub personne_id: Uuid,
    pub compte_id: Uuid,
}

/// Crée un compte utilisable pour se connecter, avec ses rôles.
///
/// `roles` porte des couples `(role_code, etablissement_id)` — `None` pour un rôle d'éditeur.
pub async fn creer_compte(
    pool: &PgPool,
    tenant_id: Uuid,
    nom: &str,
    identifiant: &str,
    mot_de_passe: &str,
    roles: &[(&str, Option<Uuid>)],
) -> JeuCompte {
    let personne_id = Uuid::now_v7();
    let compte_id = Uuid::now_v7();
    let condensat = kaya_comptes::authentification::hacher(mot_de_passe).expect("hachage");

    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, tenant_id)
        .await
        .expect("pose du tenant");

    sqlx::query!(
        "INSERT INTO comptes.personne (id, tenant_id, nom) VALUES ($1, $2, $3)",
        personne_id,
        tenant_id,
        nom
    )
    .execute(&mut *tx)
    .await
    .expect("insertion de la personne");

    // L'identifiant part dans la colonne qui correspond à sa forme : un « @ » en fait un courriel,
    // sans quoi la résolution par téléphone ne le trouverait pas — et le test échouerait sur un
    // détail de jeu d'essai plutôt que sur ce qu'il vérifie.
    let (telephone, email) = if identifiant.contains('@') {
        (None, Some(identifiant))
    } else {
        (Some(identifiant), None)
    };

    sqlx::query!(
        r#"
        INSERT INTO comptes.compte
            (id, tenant_id, personne_id, identifiant_telephone, identifiant_email,
             condensat_mot_de_passe)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        compte_id,
        tenant_id,
        personne_id,
        telephone,
        email,
        condensat,
    )
    .execute(&mut *tx)
    .await
    .expect("insertion du compte");

    for (role_code, etablissement_id) in roles {
        sqlx::query!(
            r#"
            INSERT INTO comptes.compte_role
                (id, tenant_id, compte_id, role_code, etablissement_id, attribue_par_compte_id)
            VALUES ($1, $2, $3, $4, $5, $3)
            "#,
            Uuid::now_v7(),
            tenant_id,
            compte_id,
            role_code,
            *etablissement_id,
        )
        .execute(&mut *tx)
        .await
        .expect("attribution du rôle");
    }

    tx.commit().await.expect("commit");

    JeuCompte {
        personne_id,
        compte_id,
    }
}

/// Le service d'authentification, assemblé **comme le fait l'application réelle**.
pub fn service_authentification(
    pool: PgPool,
) -> kaya_comptes::authentification::ServiceAuthentification<
    kaya_synchronisation::outbox::PgOutboxWriter,
    kaya_comptes::audit::JournalAuditPostgres,
> {
    kaya_comptes::authentification::ServiceAuthentification::nouveau(
        pool,
        kaya_comptes::session::Entrepot::nouveau(&url_redis()).expect("entrepôt Redis"),
        kaya_comptes::session::LimiteTentatives::nouveau(&url_redis()).expect("limiteur Redis"),
        cle_jwt(),
        kaya_synchronisation::outbox::PgOutboxWriter::nouveau(),
        kaya_comptes::audit::JournalAuditPostgres,
    )
}
