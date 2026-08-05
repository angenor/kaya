//! **SEJ-01** — les tests de classe des deux entités de la fiche client.
//!
//! | Entité | Classe | Ce que la macro engendre |
//! |---|---|---|
//! | `preference_personne` | **A** | Rejeu triple — une ligne, **un** événement — et les six ordres du désordre |
//! | `client` | **C** | Inatteignable hors ligne — chaque écriture exige un jeton — **et** le versant positif |
//!
//! # `preference_personne` est la DEUXIÈME cible de la porte P-14
//!
//! Elle n'en avait qu'**une** depuis le cycle 001 : `note_etablissement`. `occupation` est en B,
//! `journal_audit` est exercé à part. Une porte à cible unique ne prouve pas grand-chose de son
//! outillage — et le contrôle qui manquait à `occupation` est précisément celui que la macro
//! rétablit : **un rejeu n'émet aucun second événement outbox**.
//!
//! # ★ Ce que ce fichier a trouvé dans l'outillage, et qui vaut d'être écrit
//!
//! `tester_classe_a!` fixait le rôle à `proprietaire`, avec en commentaire : *« il porte toutes
//! les permissions »*. **C'était vrai jusqu'à la migration `0030`**, où le propriétaire ne reçoit
//! plus que les **lectures** de la fiche client — il consulte, il n'enregistre pas d'arrivée.
//! Le symptôme aurait été un `403` sur une écriture, message qui accuse le handler alors que la
//! cause est le rôle choisi par le harnais.
//!
//! La macro a donc gagné deux paramètres — `role` et `preparation` — dans une **forme longue**,
//! la forme courte des cycles 001 à 005 étant conservée et déléguant avec les valeurs par défaut.
//! Rouvrir chaque instanciation existante aurait été exactement le coût que l'outillage existe
//! pour éviter.
//!
//! # Pourquoi `client` est en C et pas en B — décision O-01, close le 2026-08-03
//!
//! Option (a) retenue : la classe **C** est maintenue, le réseau est exigé pour créer une fiche
//! nouvelle. Les options (b) — descendre en B avec fusion au cloud — et (c) — un « client
//! provisoire » local promu à la synchronisation — achetaient une friction de comptoir au prix de
//! doublons inter-établissements ou d'un mécanisme de promotion.
//!
//! **Au MVP la décision est sans effet visible** : l'arrivée elle-même est de classe B, donc déjà
//! inatteignable hors ligne. La friction résiduelle n'apparaît qu'en mode nœud de site
//! (incrément 3), et elle est écrite au §12 du registre plutôt que tue.

mod commun;

// =================================================================================================
//  CLASSE A — `preference_personne`, deuxième cible de P-14
// =================================================================================================
//
// La préparation crée une **personne cliente dans le tenant du test** et rend son identifiant,
// que la fermeture de chemin consomme. Sans elle, le chemin `/clients/{id}/preferences`
// désignerait une fiche inexistante et les six tests échoueraient sur un `404` qui ne dirait rien
// de la commutativité.
tester_classe_a!(
    preference_personne,
    schema = "comptes",
    table = "preference_personne",
    agregat = "comptes.preference_personne",
    // ⚠️ **`receptionniste`, et pas `proprietaire`** — voir le commentaire de tête.
    role = "receptionniste",
    preparation = |pool: &sqlx::PgPool, jeu: commun::JeuTenant| {
        let pool = pool.clone();
        std::boxed::Box::pin(async move { creer_client_de_test(&pool, jeu.tenant_id).await })
            as std::pin::Pin<std::boxed::Box<dyn std::future::Future<Output = uuid::Uuid> + Send>>
    },
    chemin = |client_id| format!("/api/v1/clients/{client_id}/preferences"),
    corps = |id, rang| serde_json::json!({
        "id": id,
        "texte": format!("préférence {rang} — chambre calme, étage bas"),
    }),
);

// =================================================================================================
//  CLASSE C — `client`
// =================================================================================================
//
// Les trois écritures de la fiche client. La macro engendre le versant **négatif** — aucune n'est
// atteignable sans jeton, donc sans session, donc sans réseau — **et** le versant positif, sans
// lequel une opération retirée du produit satisferait encore la moitié négative.
tester_classe_bcd!(
    client,
    classe = "C",
    operations = &[
        (
            "créer une fiche client",
            actix_web::http::Method::POST,
            "/api/v1/clients",
        ),
        (
            "modifier une fiche client",
            actix_web::http::Method::PATCH,
            "/api/v1/clients/{client_id}",
        ),
        (
            "enregistrer une préférence",
            actix_web::http::Method::POST,
            "/api/v1/clients/{client_id}/preferences",
        ),
    ],
);

// =================================================================================================
//  Préparation
// =================================================================================================

/// Crée une personne **qualifiée cliente** dans le tenant fourni, et rend son identifiant.
///
/// L'écriture est directe et non par l'endpoint, à dessein : ce que la macro mesure est le rejeu
/// d'une **préférence**, pas la création d'une fiche — laquelle a ses propres tests dans
/// `client_recherche.rs`. Passer par l'endpoint ferait dépendre six tests de commutativité du bon
/// fonctionnement d'une autre opération, et un échec là-bas rendrait ceux-ci illisibles.
async fn creer_client_de_test(pool: &sqlx::PgPool, tenant_id: uuid::Uuid) -> uuid::Uuid {
    let personne_id = uuid::Uuid::now_v7();

    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, tenant_id)
        .await
        .expect("pose du tenant");

    sqlx::query!(
        "INSERT INTO comptes.personne (id, tenant_id, nom, nom_repli) VALUES ($1, $2, $3, $4)",
        personne_id,
        tenant_id,
        "Bakayoko",
        "bakayoko",
    )
    .execute(&mut *tx)
    .await
    .expect("insertion de la personne");

    sqlx::query!(
        "INSERT INTO comptes.client (personne_id, tenant_id) VALUES ($1, $2)",
        personne_id,
        tenant_id,
    )
    .execute(&mut *tx)
    .await
    .expect("qualification cliente");

    tx.commit().await.expect("commit");
    personne_id
}
