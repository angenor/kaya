//! **Immuabilité du registre des actions — les DEUX versants.**
//!
//! # Pourquoi deux versants, et pas seulement le refus
//!
//! Le versant **négatif** — « aucune écriture destructrice n'aboutit » — est celui auquel on
//! pense. Seul, il a un défaut qui ne se voit pas : **il passe au vert si la table disparaît.**
//! Un `DELETE` qui échoue et un `DELETE` sur une table inexistante produisent tous deux une
//! erreur, et un test qui se contente d'`is_err()` ne les distingue pas.
//!
//! Le versant **positif** ferme ce trou : une entrée s'écrit réellement, se relit, et porte les
//! valeurs qu'on y a mises. C'est l'exigence du § « Couverture des portes » de la constitution —
//! *un test négatif prouve qu'une porte sait échouer, il ne prouve pas qu'elle regarde quelque
//! chose*.
//!
//! # Ce que ce fichier n'inspecte PAS
//!
//! La **présence de chemins de purge dans le code** : c'est le contrôle statique
//! `scripts/ci/outbox-sans-purge.sh`, qui balaie `backend`, `scripts` et `infra`. Les deux se
//! complètent — le script voit le code qui n'a jamais tourné, ce fichier voit ce que la base fait
//! réellement.
//!
//! # Le registre d'audit n'a PAS de déclencheur, contrairement au grand livre
//!
//! `evenement_outbox` en porte un, parce que `kaya_owner` conserve des privilèges d'écriture sur
//! elle pour la marquer publiée : il fallait donc arrêter aussi la maintenance lancée sous le
//! propriétaire. `journal_audit` n'a **aucun** chemin d'écriture légitime après l'insertion — pas
//! même un marquage — et `kaya_app` n'y détient que `SELECT, INSERT`. L'absence de privilège suffit
//! et se lit d'un coup d'œil dans la migration ; un déclencheur en plus donnerait deux endroits où
//! chercher la règle.

mod commun;

use serde_json::json;
use sqlx::Row;
use uuid::Uuid;

/// Crée une personne, un compte et rend l'identifiant du compte — l'auteur des entrées d'audit.
///
/// Sous le rôle **propriétaire** : le jeu d'essai n'a pas à passer par les chemins applicatifs,
/// qui n'existent pas encore à cette tâche.
async fn compte_auteur(pool: &sqlx::PgPool, tenant_id: Uuid) -> Uuid {
    let personne_id = Uuid::now_v7();
    let compte_id = Uuid::now_v7();

    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, tenant_id)
        .await
        .expect("pose du tenant");

    sqlx::query!(
        "INSERT INTO comptes.personne (id, tenant_id, nom) VALUES ($1, $2, $3)",
        personne_id,
        tenant_id,
        "Auteur de test"
    )
    .execute(&mut *tx)
    .await
    .expect("insertion de la personne");

    sqlx::query!(
        r#"
        INSERT INTO comptes.compte
            (id, tenant_id, personne_id, identifiant_email, condensat_mot_de_passe)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        compte_id,
        tenant_id,
        personne_id,
        format!("auteur-{compte_id}@test.local"),
        "$argon2id$v=19$m=19456,t=2,p=1$c2VsZGV0ZXN0$Y29uZGVuc2F0ZGV0ZXN0"
    )
    .execute(&mut *tx)
    .await
    .expect("insertion du compte");

    tx.commit().await.expect("commit");
    compte_id
}

/// Écrit une entrée d'audit et rend son identifiant, sous le rôle **applicatif**.
async fn ecrire_entree(pool: &sqlx::PgPool, tenant_id: Uuid, auteur: Uuid, cible: &str) -> Uuid {
    let id = Uuid::now_v7();

    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, tenant_id)
        .await
        .expect("pose du tenant");

    sqlx::query(
        r#"
        INSERT INTO comptes.journal_audit
            (id, tenant_id, type_action, auteur_compte_id, cible_type, contexte)
        VALUES ($1, $2, 'changement_role', $3, $4, $5)
        "#,
    )
    .bind(id)
    .bind(tenant_id)
    .bind(auteur)
    .bind(cible)
    .bind(json!({ "sens": "attribution", "role_code": "caissier" }))
    .execute(&mut *tx)
    .await
    .expect("insertion de l'entrée d'audit");

    tx.commit().await.expect("commit");
    id
}

// =================================================================================================
//  VERSANT POSITIF — sans lui, supprimer la table suffirait à passer au vert
// =================================================================================================

/// **Une entrée s'écrit et se relit, avec ses valeurs.**
#[tokio::test]
async fn une_entree_s_ecrit_et_se_relit() {
    let pool_owner = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool_owner, "audit — versant positif").await;
    let auteur = compte_auteur(&pool_owner, jeu.tenant_id).await;

    let pool = commun::pool_app().await;
    let id = ecrire_entree(&pool, jeu.tenant_id, auteur, "compte").await;

    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, jeu.tenant_id)
        .await
        .expect("pose du tenant");

    let ligne = sqlx::query(
        r#"
        SELECT type_action, auteur_compte_id, cible_type, contexte, cree_le, horodatage_client
        FROM comptes.journal_audit
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(&mut *tx)
    .await
    .expect("lecture")
    .expect("l'entrée doit exister — sans quoi le versant négatif ne prouverait rien");

    assert_eq!(ligne.get::<String, _>("type_action"), "changement_role");
    assert_eq!(ligne.get::<Uuid, _>("auteur_compte_id"), auteur);
    assert_eq!(ligne.get::<String, _>("cible_type"), "compte");
    assert_eq!(
        ligne.get::<serde_json::Value, _>("contexte")["role_code"],
        json!("caissier")
    );

    // `cree_le` fait autorité, `horodatage_client` est resté nul — aucune règle ne s'y appuie.
    assert!(
        ligne
            .try_get::<Option<time::OffsetDateTime>, _>("horodatage_client")
            .expect("colonne présente")
            .is_none()
    );

    tx.rollback().await.expect("rollback");
}

// =================================================================================================
//  VERSANT NÉGATIF — les privilèges, puis leur effet
// =================================================================================================

/// **`kaya_app` détient `SELECT, INSERT` et rien d'autre.**
///
/// C'est l'immuabilité elle-même : elle ne repose ni sur une convention de rédaction, ni sur
/// l'absence de chemin de code, mais sur un privilège absent.
#[tokio::test]
async fn kaya_app_ne_detient_que_select_et_insert_sur_le_registre() {
    let pool = commun::pool_owner().await;

    for (privilege, attendu) in [
        ("SELECT", true),
        ("INSERT", true),
        ("UPDATE", false),
        ("DELETE", false),
        ("TRUNCATE", false),
    ] {
        let detenu: bool = sqlx::query_scalar(
            "SELECT has_table_privilege('kaya_app', 'comptes.journal_audit', $1)",
        )
        .bind(privilege)
        .fetch_one(&pool)
        .await
        .expect("lecture du privilège");

        assert_eq!(
            detenu, attendu,
            "privilège {privilege} sur comptes.journal_audit : {detenu}, attendu {attendu}.\n\
             Le registre des actions est de classe A — append-only. Un `UPDATE` accordé casserait \
             la commutativité que le test de désordre vérifie, et rendrait faux le classement A \
             sans que rien ne le signale. Un `DELETE` accordé ferait du registre que le \
             propriétaire achète une liste qu'on peut vider."
        );
    }
}

/// **Un `UPDATE` sous le rôle applicatif est refusé — effet réel, pas seulement privilège.**
#[tokio::test]
async fn un_update_sous_le_role_applicatif_est_refuse() {
    let pool_owner = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool_owner, "audit — refus d'UPDATE").await;
    let auteur = compte_auteur(&pool_owner, jeu.tenant_id).await;

    let pool = commun::pool_app().await;
    let id = ecrire_entree(&pool, jeu.tenant_id, auteur, "compte").await;

    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, jeu.tenant_id)
        .await
        .expect("pose du tenant");

    let resultat = sqlx::query("UPDATE comptes.journal_audit SET cible_type = 'falsifie' WHERE id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await;

    let erreur = resultat.expect_err(
        "l'UPDATE a réussi — le registre des actions est modifiable, et tout ce qu'il prouve \
         cesse d'être opposable",
    );
    assert!(
        erreur.to_string().to_lowercase().contains("permission"),
        "refus attendu pour défaut de privilège, obtenu : {erreur}"
    );

    tx.rollback().await.expect("rollback");
}

/// **Un `DELETE` sous le rôle applicatif est refusé.**
#[tokio::test]
async fn un_delete_sous_le_role_applicatif_est_refuse() {
    let pool_owner = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool_owner, "audit — refus de DELETE").await;
    let auteur = compte_auteur(&pool_owner, jeu.tenant_id).await;

    let pool = commun::pool_app().await;
    let id = ecrire_entree(&pool, jeu.tenant_id, auteur, "compte").await;

    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, jeu.tenant_id)
        .await
        .expect("pose du tenant");

    let erreur = sqlx::query("DELETE FROM comptes.journal_audit WHERE id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await
        .expect_err("le DELETE a réussi — le registre peut être vidé");

    assert!(
        erreur.to_string().to_lowercase().contains("permission"),
        "refus attendu pour défaut de privilège, obtenu : {erreur}"
    );

    tx.rollback().await.expect("rollback");

    // Et l'entrée est toujours là — le refus n'a rien emporté au passage.
    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, jeu.tenant_id)
        .await
        .expect("pose du tenant");
    let subsiste: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM comptes.journal_audit WHERE id = $1)")
            .bind(id)
            .fetch_one(&mut *tx)
            .await
            .expect("lecture");
    assert!(subsiste, "l'entrée a disparu après un DELETE pourtant refusé");
    tx.rollback().await.expect("rollback");
}

/// **FR-014 est structurel** : un compte désigné par une entrée d'audit ne peut pas être supprimé.
///
/// Le mécanisme est la clé étrangère `journal_audit.auteur_compte_id → compte.id`. Ce test
/// l'exerce sous le rôle **propriétaire**, qui détient pourtant `DELETE` sur `compte` : c'est la
/// contrainte qui refuse, pas le privilège. Sous `kaya_app`, le `DELETE` échouerait pour la
/// mauvaise raison et ne prouverait pas ce qu'on cherche.
#[tokio::test]
async fn un_compte_designe_par_une_entree_ne_peut_pas_etre_supprime() {
    let pool = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool, "audit — FR-014 structurel").await;
    let auteur = compte_auteur(&pool, jeu.tenant_id).await;

    let pool_app = commun::pool_app().await;
    ecrire_entree(&pool_app, jeu.tenant_id, auteur, "compte").await;

    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, jeu.tenant_id)
        .await
        .expect("pose du tenant");

    let erreur = sqlx::query("DELETE FROM comptes.compte WHERE id = $1")
        .bind(auteur)
        .execute(&mut *tx)
        .await
        .expect_err(
            "le compte a été supprimé alors qu'une entrée d'audit le désigne. FR-014 cesse d'être \
             structurel : le registre désignerait un identifiant sans nom.",
        );

    assert!(
        erreur.to_string().to_lowercase().contains("foreign key")
            || erreur.to_string().to_lowercase().contains("violates"),
        "refus attendu par violation de clé étrangère, obtenu : {erreur}"
    );

    tx.rollback().await.expect("rollback");
}

/// **Le rejeu d'une entrée est inoffensif et silencieux.**
///
/// Trois soumissions du même identifiant → **une** ligne. C'est le comportement d'une entité de
/// classe A, et c'est ce qui rend une trace écrite hors ligne rejouable sans doublon. Le test
/// complet de classe A — rejeu **et** désordre — vit dans `audit_classe_a.rs` ; celui-ci vérifie
/// seulement que l'immuabilité n'entre pas en conflit avec l'idempotence.
#[tokio::test]
async fn trois_soumissions_du_meme_identifiant_produisent_une_ligne() {
    let pool_owner = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool_owner, "audit — rejeu inoffensif").await;
    let auteur = compte_auteur(&pool_owner, jeu.tenant_id).await;

    let pool = commun::pool_app().await;
    let id = Uuid::now_v7();

    for tentative in 1..=3 {
        let mut tx = pool.begin().await.expect("transaction");
        kaya_etablissements::tenant_context::poser_tenant(&mut tx, jeu.tenant_id)
            .await
            .expect("pose du tenant");

        sqlx::query(
            r#"
            INSERT INTO comptes.journal_audit
                (id, tenant_id, type_action, auteur_compte_id, cible_type, contexte)
            VALUES ($1, $2, 'changement_role', $3, 'compte', '{}'::jsonb)
            ON CONFLICT (id) DO NOTHING
            "#,
        )
        .bind(id)
        .bind(jeu.tenant_id)
        .bind(auteur)
        .execute(&mut *tx)
        .await
        .unwrap_or_else(|e| panic!("tentative {tentative} : {e}"));

        tx.commit().await.expect("commit");
    }

    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, jeu.tenant_id)
        .await
        .expect("pose du tenant");
    let lignes: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM comptes.journal_audit WHERE id = $1")
            .bind(id)
            .fetch_one(&mut *tx)
            .await
            .expect("comptage");
    tx.rollback().await.expect("rollback");

    assert_eq!(
        lignes, 1,
        "trois soumissions du même identifiant ont produit {lignes} ligne(s). Un terminal qui \
         vide sa file après une coupure créerait des doublons dans le registre — découverts trois \
         mois plus tard, en cherchant pourquoi une remise apparaît trois fois."
    );
}
