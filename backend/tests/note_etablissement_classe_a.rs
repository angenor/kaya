//! **Porte P-14** — les deux tests obligatoires de la classe A.
//!
//! `docs/user-stories-v1.md` §0.7 et `docs/registre-classes-offline.md` §11 les exigent pour
//! toute entité de classe A, dans la story qui l'introduit — jamais dans un lot de rattrapage.
//!
//! | Test | Ce qu'il vérifie |
//! |---|---|
//! | **Rejeu** | La même écriture envoyée trois fois produit **un seul** enregistrement |
//! | **Désordre** | Trois écritures dans les **six** ordres possibles donnent le même état final |
//!
//! # Pourquoi ces deux-là et pas d'autres
//!
//! Une classe A est écrite depuis un terminal qui peut être hors ligne. Deux choses lui arrivent
//! en pratique, et ce sont les seules qui comptent :
//!
//! - il **rejoue** sa file après une coupure, sans savoir ce que le serveur a déjà reçu ;
//! - ses écritures arrivent **dans le désordre**, parce que plusieurs terminaux se resynchronisent
//!   en même temps.
//!
//! Une entité indûment classée A produit des incohérences silencieuses découvertes trois mois
//! plus tard en pleine clôture. Ces deux tests sont ce qui rend le classement opposable.

mod commun;

use actix_web::test;
use serde_json::json;
use uuid::Uuid;


/// **Test de rejeu.** Trois envois du même identifiant → un enregistrement, `201` puis `200`.
///
/// Le code de statut fait partie du test, pas seulement le décompte. Répondre `409` au rejeu
/// obligerait chaque appelant hors ligne à traiter comme une erreur une écriture que le serveur a
/// déjà acceptée — ce que le principe VI interdit.
#[actix_web::test]
async fn rejeu_triple_produit_un_seul_enregistrement() {
    let pool_owner = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool_owner, "classe A — rejeu").await;
    let pool = commun::pool_app().await;
    let app = monter_application!(pool.clone());

    let note_id = Uuid::now_v7();
    // Depuis T030, le contexte vient d'un **jeton réel** obtenu par le vrai chemin de connexion.
    // Les deux en-têtes du provisoire `CONTEXTE_PAR_EN_TETES` n'existent plus.
    let cx = commun::compte_connecte(
        &pool_owner,
        jeu,
        "classe A — rejeu",
        &[("proprietaire", Some(jeu.etablissement_id))],
    )
    .await;
    let chemin = format!("/api/v1/etablissements/{}/notes", jeu.etablissement_id);

    let mut statuts = Vec::new();
    for _ in 0..3 {
        let requete = test::TestRequest::post()
            .uri(&chemin)
            .insert_header(("Authorization", cx.bearer.clone()))
            .set_json(json!({
                "id": note_id,
                "texte": "Le groupe électrogène a démarré à 19 h 40.",
            }))
            .to_request();

        statuts.push(test::call_service(&app, requete).await.status().as_u16());
    }

    assert_eq!(
        statuts,
        vec![201, 200, 200],
        "le premier envoi doit créer (201) et les rejeux constater (200). Obtenu : {statuts:?}"
    );

    let compte = compter_notes(&pool_owner, jeu.tenant_id, note_id).await;
    assert_eq!(
        compte, 1,
        "trois envois du même identifiant ont produit {compte} enregistrement(s). \
         L'identifiant est-il bien fourni par le client, et l'INSERT porte-t-il \
         ON CONFLICT (id) DO NOTHING ?"
    );

    // Un rejeu n'est PAS une transition d'état : il ne doit produire aucun second événement.
    // Sinon le grand livre devient le journal des tentatives réseau du terminal au lieu de celui
    // des changements d'état — et la reconstitution compterait la note trois fois.
    let evenements = compter_evenements(&pool_owner, jeu.tenant_id, note_id).await;
    assert_eq!(
        evenements, 1,
        "trois envois ont produit {evenements} événement(s) au grand livre. Un rejeu ne change \
         aucun état : il n'émet rien."
    );
}

/// **Test de désordre.** Trois notes appliquées dans les six ordres → même état final.
///
/// Six permutations, six jeux de données indépendants. Si l'état final dépendait de l'ordre
/// d'arrivée, l'entité ne serait pas commutative et son classement en A serait faux.
#[actix_web::test]
async fn desordre_les_six_ordres_donnent_le_meme_etat_final() {
    let pool_owner = commun::pool_owner().await;
    let pool = commun::pool_app().await;
    let app = monter_application!(pool.clone());

    const PERMUTATIONS: [[usize; 3]; 6] = [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ];

    let mut etats_finaux = Vec::new();

    for (rang, ordre) in PERMUTATIONS.iter().enumerate() {
        let jeu = commun::creer_tenant(&pool_owner, &format!("classe A — désordre {rang}")).await;
        let cx = commun::compte_connecte(
            &pool_owner,
            jeu,
            "classe A — désordre",
            &[("proprietaire", Some(jeu.etablissement_id))],
        )
        .await;
        let chemin = format!("/api/v1/etablissements/{}/notes", jeu.etablissement_id);

        // Identifiants figés par permutation : les trois notes sont *les mêmes* écritures, seul
        // leur ordre d'arrivée change. Des identifiants tirés au hasard à chaque envoi
        // compareraient des jeux différents et le test ne dirait rien.
        let notes = [
            (Uuid::now_v7(), "Réception : clé 12 rendue."),
            (Uuid::now_v7(), "Ménage : chambre 4 terminée."),
            (Uuid::now_v7(), "Bar : bouteille de gaz à remplacer."),
        ];

        for &index in ordre {
            let (id, texte) = notes[index];
            let requete = test::TestRequest::post()
                .uri(&chemin)
                .insert_header(("Authorization", cx.bearer.clone()))
                .set_json(json!({ "id": id, "texte": texte }))
                .to_request();

            let reponse = test::call_service(&app, requete).await;
            assert_eq!(
                reponse.status().as_u16(),
                201,
                "permutation {ordre:?} : l'envoi de la note {index} a échoué"
            );
        }

        // L'état final est l'ensemble {identifiant, texte}, trié — pas la liste dans l'ordre
        // d'insertion. Comparer l'ordre d'affichage reviendrait à exiger la non-commutativité
        // qu'on cherche justement à écarter.
        let mut etat: Vec<(Uuid, String)> = sqlx::query!(
            r#"
            SELECT id, texte
            FROM etablissements.note_etablissement
            WHERE etablissement_id = $1
            "#,
            jeu.etablissement_id
        )
        .fetch_all(&mut *transaction_tenant(&pool_owner, jeu.tenant_id).await)
        .await
        .expect("lecture de l'état final")
        .into_iter()
        .map(|l| (l.id, l.texte))
        .collect();
        etat.sort();

        let textes: Vec<String> = etat.iter().map(|(_, t)| t.clone()).collect();
        etats_finaux.push((ordre, textes));
    }

    let (premier_ordre, reference) = &etats_finaux[0];
    for (ordre, etat) in &etats_finaux[1..] {
        assert_eq!(
            etat, reference,
            "l'ordre {ordre:?} produit un état final différent de l'ordre {premier_ordre:?} : \
             l'entité n'est pas commutative et son classement en A est faux"
        );
    }
    assert_eq!(reference.len(), 3, "les trois notes doivent être présentes");
}

/// Ouvre une transaction avec le tenant courant posé.
async fn transaction_tenant(
    pool: &sqlx::PgPool,
    tenant_id: Uuid,
) -> sqlx::PgTransaction<'static> {
    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, tenant_id)
        .await
        .expect("pose du tenant");
    tx
}

async fn compter_notes(pool: &sqlx::PgPool, tenant_id: Uuid, note_id: Uuid) -> i64 {
    let mut tx = transaction_tenant(pool, tenant_id).await;
    sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "compte!" FROM etablissements.note_etablissement WHERE id = $1"#,
        note_id
    )
    .fetch_one(&mut *tx)
    .await
    .expect("comptage des notes")
}

async fn compter_evenements(pool: &sqlx::PgPool, tenant_id: Uuid, agregat_id: Uuid) -> i64 {
    let mut tx = transaction_tenant(pool, tenant_id).await;
    sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) AS "compte!"
        FROM synchronisation.evenement_outbox
        WHERE agregat = 'note_etablissement' AND agregat_id = $1
        "#,
        agregat_id
    )
    .fetch_one(&mut *tx)
    .await
    .expect("comptage des événements")
}
