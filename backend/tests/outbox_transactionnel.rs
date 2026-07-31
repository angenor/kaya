//! **Porte P-05** — toute transition d'état émet un événement **dans sa transaction**.
//!
//! La porte a deux niveaux, et ce fichier vérifie le second :
//!
//!   (a) la **signature** d'`OutboxWriter::ecrire` rend l'écriture hors transaction impossible à
//!       compiler — garantie statique, rien à tester ;
//!   (b) **après chaque mutation exposée, un événement existe ; après un rollback provoqué, ni
//!       ligne métier ni événement** — c'est ce qui suit.
//!
//! Le point (b) est le seul qui puisse encore échouer une fois (a) acquis : rien n'empêche
//! d'oublier l'appel. Ce qui ne peut pas arriver, en revanche, c'est que la ligne soit écrite et
//! l'événement perdu — et c'est précisément ce que le test de rollback constate.

mod commun;

use uuid::Uuid;

use kaya_etablissements::note::{CreerNote, ServiceNote};
use kaya_synchronisation::outbox::PgOutboxWriter;
use kaya_synchronisation::{EvenementAEcrire, OutboxWriter};

/// Après une création, l'événement existe — même agrégat, même identifiant.
#[tokio::test]
async fn p05_toute_creation_laisse_un_evenement() {
    let pool_owner = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool_owner, "P-05 création").await;

    let service = ServiceNote::nouveau(commun::pool_app().await, PgOutboxWriter::nouveau());
    let note_id = Uuid::now_v7();

    service
        .creer(
            jeu.tenant_id,
            CreerNote {
                id: note_id,
                etablissement_id: jeu.etablissement_id,
                auteur_compte_id: Uuid::now_v7(),
                texte: "Le climatiseur de la 7 fuit.".to_owned(),
                horodatage_client: None,
            },
        )
        .await
        .expect("création de la note");

    let mut tx = pool_owner.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, jeu.tenant_id)
        .await
        .expect("pose du tenant");

    let evenement = sqlx::query!(
        r#"
        SELECT type_evenement, agregat, version_schema, payload
        FROM synchronisation.evenement_outbox
        WHERE agregat_id = $1
        "#,
        note_id
    )
    .fetch_optional(&mut *tx)
    .await
    .expect("lecture du grand livre");

    let evenement = evenement.expect(
        "aucun événement pour une note créée : la transition d'état n'est pas tracée, et le grand \
         livre ne permettra pas de reconstituer ce qui s'est passé",
    );

    assert_eq!(evenement.type_evenement, "note_etablissement.creee");
    assert_eq!(evenement.agregat, "note_etablissement");
    assert_eq!(evenement.version_schema, 1);
    assert_eq!(
        evenement.payload["texte"], "Le climatiseur de la 7 fuit.",
        "la charge utile doit être dénormalisée : le texte en clair, pas un renvoi vers la table"
    );

    tx.rollback().await.expect("rollback");
}

/// **Après un rollback provoqué : ni ligne métier, ni événement.**
///
/// Le test écrit les deux dans une transaction, puis annule. C'est le scénario qu'une
/// implémentation naïve raterait — celle qui ouvrirait une seconde transaction pour l'événement
/// « pour ne pas alourdir la première ». La ligne disparaîtrait, l'événement resterait, et le
/// grand livre affirmerait une transition qui n'a jamais eu lieu.
#[tokio::test]
async fn p05_apres_un_rollback_ni_ligne_ni_evenement() {
    let pool_owner = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool_owner, "P-05 rollback").await;

    let pool = commun::pool_app().await;
    let note_id = Uuid::now_v7();
    let evenement_id = Uuid::now_v7();

    // Transaction menée à la main : le service commit toujours, or c'est justement l'absence de
    // commit qu'on veut éprouver.
    {
        let mut tx = pool.begin().await.expect("transaction");
        kaya_etablissements::tenant_context::poser_tenant(&mut tx, jeu.tenant_id)
            .await
            .expect("pose du tenant");

        sqlx::query!(
            r#"
            INSERT INTO etablissements.note_etablissement
                (id, tenant_id, etablissement_id, auteur_compte_id, texte)
            VALUES ($1, $2, $3, $4, 'note annulée par rollback')
            "#,
            note_id,
            jeu.tenant_id,
            jeu.etablissement_id,
            Uuid::now_v7(),
        )
        .execute(&mut *tx)
        .await
        .expect("insertion de la note");

        PgOutboxWriter::nouveau()
            .ecrire(
                &mut tx,
                EvenementAEcrire {
                    id: evenement_id,
                    tenant_id: jeu.tenant_id,
                    etablissement_id: Some(jeu.etablissement_id),
                    type_evenement: "note_etablissement.creee".to_owned(),
                    agregat: "note_etablissement".to_owned(),
                    agregat_id: note_id,
                    version_schema: 1,
                    payload: serde_json::json!({ "texte": "note annulée par rollback" }),
                },
            )
            .await
            .expect("écriture de l'événement");

        // Rollback — la transaction entière disparaît.
        tx.rollback().await.expect("rollback");
    }

    let mut tx = pool_owner.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, jeu.tenant_id)
        .await
        .expect("pose du tenant");

    let notes: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "c!" FROM etablissements.note_etablissement WHERE id = $1"#,
        note_id
    )
    .fetch_one(&mut *tx)
    .await
    .expect("comptage des notes");

    let evenements: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "c!" FROM synchronisation.evenement_outbox WHERE id = $1"#,
        evenement_id
    )
    .fetch_one(&mut *tx)
    .await
    .expect("comptage des événements");

    assert_eq!(notes, 0, "la note a survécu au rollback");
    assert_eq!(
        evenements, 0,
        "l'événement a survécu au rollback alors que la ligne métier a disparu. Le grand livre \
         affirmerait une transition qui n'a jamais eu lieu — c'est exactement ce que la signature \
         d'OutboxWriter::ecrire est censée rendre impossible."
    );

    tx.rollback().await.expect("rollback");
}

/// La séquence est **monotone par établissement**, et les trous sont acceptés.
///
/// R-07 le dit explicitement : les séquences PostgreSQL ne sont pas transactionnelles, un
/// rollback laisse un trou, et **c'est voulu**. Le test vérifie la propriété réellement exigée —
/// la croissance stricte — et non la continuité, qui imposerait un verrou par établissement sur
/// le chemin d'écriture le plus chaud du produit.
#[tokio::test]
async fn la_sequence_est_strictement_croissante_par_etablissement() {
    let pool_owner = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool_owner, "P-05 séquence").await;

    let service = ServiceNote::nouveau(commun::pool_app().await, PgOutboxWriter::nouveau());
    for index in 0..5 {
        service
            .creer(
                jeu.tenant_id,
                CreerNote {
                    id: Uuid::now_v7(),
                    etablissement_id: jeu.etablissement_id,
                    auteur_compte_id: Uuid::now_v7(),
                    texte: format!("note {index}"),
                    horodatage_client: None,
                },
            )
            .await
            .expect("création");
    }

    let mut tx = pool_owner.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, jeu.tenant_id)
        .await
        .expect("pose du tenant");

    let sequences: Vec<i64> = sqlx::query_scalar!(
        r#"
        SELECT sequence_etablissement
        FROM synchronisation.evenement_outbox
        WHERE etablissement_id = $1
        ORDER BY id
        "#,
        jeu.etablissement_id
    )
    .fetch_all(&mut *tx)
    .await
    .expect("lecture des séquences");

    assert_eq!(sequences.len(), 5);
    for paire in sequences.windows(2) {
        assert!(
            paire[1] > paire[0],
            "la séquence n'est pas strictement croissante : {sequences:?}. C'est elle qui \
             permettra à un nœud de site de détecter qu'il lui manque un événement."
        );
    }

    tx.rollback().await.expect("rollback");
}
