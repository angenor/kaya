//! Redémarrage brutal du worker — **SC-004**.
//!
//! # Ce qui est vérifié, et pourquoi c'est la bonne chose à vérifier
//!
//! Un worker interrompu entre la présentation d'un événement à ses consommateurs et le marquage
//! `publie_le` **republiera** cet événement au redémarrage. C'est voulu : l'inverse — marquer
//! avant de présenter — perdrait des événements, ce que la rétention illimitée de TRX-02 exclut.
//!
//! Deux propriétés en découlent, et le test constate les deux :
//!
//!   1. **Le nombre d'événements en base est identique avant et après.** Un redémarrage ne
//!      supprime ni ne duplique aucune ligne du grand livre.
//!   2. **Les consommateurs voient l'effet d'une seule présentation.** Non parce que le worker le
//!      garantit — il ne le peut pas — mais parce que **leur propre idempotence** le garantit
//!      (R-08). C'est la seule façon d'y arriver, et c'est pourquoi `EventConsumer` porte cette
//!      obligation dans sa documentation plutôt que dans une convention.

mod commun;

use std::sync::Arc;

use kaya_etablissements::note::{CreerNote, ServiceNote};
use kaya_synchronisation::consommateurs::{CompteurIdempotent, JournaliseurEvenements};
use kaya_synchronisation::outbox::PgOutboxWriter;
use kaya_synchronisation::worker::{ConfigurationWorker, WorkerPublication};
use uuid::Uuid;

#[tokio::test]
async fn un_redemarrage_brutal_ne_perd_ni_ne_duplique_aucun_evenement() {
    // Les décomptes de lot ci-dessous n'ont de sens que si aucun autre worker ne publie sur cette
    // base. Le filtre par tenant écarte les autres tests, pas un worker `tenant: None` extérieur.
    commun::exiger_grand_livre_sans_consommateur_concurrent().await;

    let pool_owner = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool_owner, "worker — redémarrage").await;

    // --- Cinq événements produits par le chemin métier réel ------------------------------------
    let service = ServiceNote::nouveau(commun::pool_app().await, PgOutboxWriter::nouveau());
    let mut identifiants = Vec::new();
    for index in 0..5 {
        let id = Uuid::now_v7();
        service
            .creer(
                jeu.tenant_id,
                CreerNote {
                    id,
                    etablissement_id: jeu.etablissement_id,
                    auteur_compte_id: Uuid::now_v7(),
                    texte: format!("événement {index}"),
                    horodatage_client: None,
                },
            )
            .await
            .expect("création");
        identifiants.push(id);
    }

    let avant = compter_evenements(&pool_owner, jeu).await;
    assert_eq!(avant, 5, "cinq créations doivent produire cinq événements");

    // --- Premier worker : il consomme, puis « meurt » avant d'avoir tout marqué ---------------
    //
    // Le redémarrage brutal est simulé en construisant un **second** worker avec de **nouveaux**
    // consommateurs, ce qui reproduit fidèlement la perte d'état en mémoire d'un processus tué.
    let compteur_1 = Arc::new(CompteurIdempotent::nouveau("compteur"));
    let journal_1 = Arc::new(JournaliseurEvenements::nouveau());
    let worker_1 = WorkerPublication::nouveau(
        commun::pool_worker().await,
        vec![compteur_1.clone(), journal_1.clone()],
        ConfigurationWorker {
            // Deux événements par tour : la « mort » intervient donc au milieu du lot suivant,
            // et non à une frontière commode.
            taille_lot: 2,
            // Périmètre restreint au tenant du test. Les fichiers de test s'exécutent en
            // parallèle sur une base partagée ; sans ce filtre, ce worker publierait les
            // événements produits au même instant par les autres tests et ses décomptes
            // dépendraient de l'ordonnancement.
            tenant: Some(jeu.tenant_id),
            ..ConfigurationWorker::default()
        },
    );

    let publies_1 = worker_1.traiter_un_lot().await.expect("premier lot");
    assert_eq!(publies_1, 2, "le premier lot doit publier deux événements");
    assert_eq!(compteur_1.distincts(), 2);
    assert_eq!(journal_1.presentations(), 2);

    drop(worker_1); // « mort » du processus

    // --- Second worker : reprend là où le premier s'est arrêté --------------------------------
    let compteur_2 = Arc::new(CompteurIdempotent::nouveau("compteur"));
    let journal_2 = Arc::new(JournaliseurEvenements::nouveau());
    let worker_2 = WorkerPublication::nouveau(
        commun::pool_worker().await,
        vec![compteur_2.clone(), journal_2.clone()],
        ConfigurationWorker {
            taille_lot: 10,
            tenant: Some(jeu.tenant_id),
            ..ConfigurationWorker::default()
        },
    );

    let publies_2 = worker_2.traiter_un_lot().await.expect("second lot");
    assert_eq!(
        publies_2, 3,
        "le second worker doit reprendre les trois événements restants — ni plus (republication \
         de ce qui est déjà marqué), ni moins (événements perdus)"
    );

    // Plus rien à publier.
    assert_eq!(
        worker_2.traiter_un_lot().await.expect("troisième lot"),
        0,
        "un troisième tour ne doit rien trouver : les cinq événements sont marqués"
    );

    // --- Propriété 1 : le grand livre n'a ni perdu ni gagné de ligne --------------------------
    let apres = compter_evenements(&pool_owner, jeu).await;
    assert_eq!(
        avant, apres,
        "le nombre d'événements a changé après redémarrage ({avant} → {apres}) : le grand livre \
         n'est ni permanent ni immuable"
    );

    // --- Propriété 2 : chaque événement présenté une fois à chaque consommateur ---------------
    let total_distincts = compteur_1.distincts() + compteur_2.distincts();
    assert_eq!(
        total_distincts, 5,
        "les consommateurs ont vu {total_distincts} événements distincts au lieu de 5"
    );
    let total_presentations = journal_1.presentations() + journal_2.presentations();
    assert_eq!(
        total_presentations, 5,
        "il y a eu {total_presentations} présentations pour 5 événements : le second worker a \
         repris des événements déjà marqués publiés"
    );

    // --- Tous les événements sont marqués, aucun ne reste en attente ---------------------------
    let en_attente = compter_en_attente(&pool_owner, jeu).await;
    assert_eq!(
        en_attente, 0,
        "{en_attente} événement(s) restent non publiés après deux tours complets"
    );
}

/// Un consommateur qui échoue laisse l'événement **non marqué**, donc republié.
///
/// Aucun événement n'est jamais abandonné. C'est ce qui rend acceptable l'absence de file de
/// lettres mortes : la file *est* la table, et elle ne perd rien.
#[tokio::test]
async fn un_consommateur_en_echec_laisse_l_evenement_en_attente() {
    use kaya_synchronisation::{ErreurConsommation, EvenementPublie};

    struct ToujoursEnEchec;

    #[async_trait::async_trait]
    impl kaya_synchronisation::EventConsumer for ToujoursEnEchec {
        fn nom(&self) -> &'static str {
            "toujours-en-echec"
        }
        async fn consommer(&self, _: &EvenementPublie) -> Result<(), ErreurConsommation> {
            Err(ErreurConsommation::Traitement {
                consommateur: "toujours-en-echec",
                motif: "échec délibéré du test".to_owned(),
            })
        }
    }

    // Un worker extérieur publierait l'événement et le laisserait marqué : l'attente « il reste
    // non publié » deviendrait fausse sans que le consommateur en échec y soit pour rien.
    commun::exiger_grand_livre_sans_consommateur_concurrent().await;

    let pool_owner = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool_owner, "worker — consommateur en échec").await;

    let service = ServiceNote::nouveau(commun::pool_app().await, PgOutboxWriter::nouveau());
    service
        .creer(
            jeu.tenant_id,
            CreerNote {
                id: Uuid::now_v7(),
                etablissement_id: jeu.etablissement_id,
                auteur_compte_id: Uuid::now_v7(),
                texte: "événement que personne ne sait consommer".to_owned(),
                horodatage_client: None,
            },
        )
        .await
        .expect("création");

    let worker = WorkerPublication::nouveau(
        commun::pool_worker().await,
        vec![Arc::new(ToujoursEnEchec)],
        ConfigurationWorker {
            tenant: Some(jeu.tenant_id),
            ..ConfigurationWorker::default()
        },
    );

    let publies = worker.traiter_un_lot().await.expect("lot");
    assert_eq!(
        publies, 0,
        "un événement dont la consommation échoue ne doit PAS être marqué publié"
    );

    assert_eq!(
        compter_en_attente(&pool_owner, jeu).await,
        1,
        "l'événement doit rester en attente, donc republié au tour suivant, indéfiniment"
    );
}

async fn compter_evenements(pool: &sqlx::PgPool, jeu: commun::JeuTenant) -> i64 {
    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, jeu.tenant_id)
        .await
        .expect("pose du tenant");
    sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "c!" FROM synchronisation.evenement_outbox WHERE etablissement_id = $1"#,
        jeu.etablissement_id
    )
    .fetch_one(&mut *tx)
    .await
    .expect("comptage")
}

async fn compter_en_attente(pool: &sqlx::PgPool, jeu: commun::JeuTenant) -> i64 {
    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, jeu.tenant_id)
        .await
        .expect("pose du tenant");
    sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) AS "c!"
        FROM synchronisation.evenement_outbox
        WHERE etablissement_id = $1 AND publie_le IS NULL
        "#,
        jeu.etablissement_id
    )
    .fetch_one(&mut *tx)
    .await
    .expect("comptage")
}
