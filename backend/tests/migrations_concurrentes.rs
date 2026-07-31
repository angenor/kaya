//! Démarrage concurrent — deux instances qui migrent en même temps (R-12).
//!
//! **Pourquoi ce test existe.** Les migrations s'appliquent au démarrage du binaire, avant
//! l'ouverture du port. En production, deux conteneurs redémarrent souvent ensemble : après un
//! déploiement, après une coupure de courant, après un `docker compose up` qui relance tout. Les
//! deux appellent alors `migrate!()` sur la même base, au même instant.
//!
//! sqlx pose pour cela un **verrou consultatif**. C'est écrit dans sa documentation — et la
//! documentation d'une bibliothèque n'est pas une garantie sur *notre* configuration : la table
//! de suivi est renommée et déplacée dans un schéma dédié par `backend/api/sqlx.toml`, ce qui est
//! précisément la partie du mécanisme qui pourrait mal se comporter. Le comportement est donc
//! **vérifié**, pas supposé.

mod commun;

use futures::future::join_all;

#[tokio::test]
async fn quatre_migrations_simultanees_ne_se_marchent_pas_dessus() {
    let migrateur = kaya_api::db::migrateur();

    // Quatre pools distincts : quatre connexions, comme quatre processus. Un pool partagé
    // sérialiserait les appels et le test ne prouverait rien.
    let pools = join_all((0..4).map(|_| commun::pool_owner())).await;

    let migrateur = &migrateur;
    let resultats = join_all(
        pools
            .iter()
            .map(|pool| async move { migrateur.run(pool).await }),
    )
    .await;

    for (index, resultat) in resultats.iter().enumerate() {
        assert!(
            resultat.is_ok(),
            "l'instance {index} a échoué à migrer alors qu'une autre migrait : {:?}\n\
             Le verrou consultatif de sqlx n'a pas joué son rôle — deux conteneurs qui \
             redémarrent ensemble laisseraient la base dans un état partiel.",
            resultat.as_ref().err()
        );
    }

    // Chaque migration n'est inscrite qu'une fois, quel que soit le nombre d'instances.
    let pool = &pools[0];
    let doublons: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) AS "compte!"
        FROM (
            SELECT version
            FROM kaya_migrations._migrations_appliquees
            GROUP BY version
            HAVING COUNT(*) > 1
        ) AS doublons
        "#
    )
    .fetch_one(pool)
    .await
    .expect("lecture de la table de suivi");

    assert_eq!(
        doublons, 0,
        "une migration est inscrite plusieurs fois : le verrou n'a pas tenu"
    );
}
