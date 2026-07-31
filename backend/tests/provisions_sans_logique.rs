//! Provisions comptables — **les tables existent, la logique n'existe pas** (principe X).
//!
//! # Un test dont l'objet est de constater qu'on n'a rien construit
//!
//! C'est inhabituel, et c'est le sujet. Les provisions du cadrage §14 sont des **choix de modèle
//! de données et d'interfaces uniquement** : aucune interface, aucune logique au MVP. Rien
//! n'empêche pourtant un cycle ultérieur d'« ajouter juste un petit endpoint de lecture » — et
//! c'est ainsi qu'une provision devient une fonctionnalité que personne n'a décidé de construire.
//!
//! Ce fichier rend ce glissement bruyant.

mod commun;

use sqlx::Row;

/// Les deux tables existent, avec leurs contraintes.
#[tokio::test]
async fn les_deux_tables_de_provision_existent() {
    let pool = commun::pool_owner().await;

    for table in ["exercice_comptable", "mapping_comptable"] {
        let existe: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS (
                SELECT 1 FROM information_schema.tables
                WHERE table_schema = 'fiscalite' AND table_name = $1
            )
            "#,
        )
        .bind(table)
        .fetch_one(&pool)
        .await
        .expect("lecture du catalogue");

        assert!(existe, "fiscalite.{table} est absente");
    }
}

/// **Aucun endpoint** ne les expose.
///
/// Le contrat OpenAPI est la source de vérité de ce que l'API expose (principe I(a)). S'y référer
/// est donc plus sûr que de relire les fichiers de routes : un endpoint monté sans annotation
/// n'apparaîtrait pas au contrat, mais il n'apparaîtrait pas non plus dans le client généré, et
/// la porte P-08 le signalerait.
#[test]
fn aucun_endpoint_n_expose_les_provisions() {
    let contrat = kaya_api::application::contrat_complet();

    let suspects: Vec<&String> = contrat
        .paths
        .paths
        .keys()
        .filter(|chemin| {
            let c = chemin.to_lowercase();
            c.contains("exercice")
                || c.contains("comptab")
                || c.contains("mapping")
                || c.contains("fiscalite")
        })
        .collect();

    assert!(
        suspects.is_empty(),
        "des endpoints exposent les provisions comptables : {suspects:?}\n\
         Les provisions sont des TABLES SEULEMENT (principe X). Un endpoint, même en lecture, en \
         fait une fonctionnalité que personne n'a décidé de construire."
    );
}

/// **Aucun droit d'écriture** n'est accordé au rôle applicatif.
///
/// C'est la vérification qui vaut les deux précédentes : même si un endpoint apparaissait, il ne
/// pourrait rien écrire. La provision est tenue par la base, pas par la discipline.
#[tokio::test]
async fn le_role_applicatif_ne_peut_pas_ecrire_dans_les_provisions() {
    let pool = commun::pool_owner().await;

    let lignes = sqlx::query(
        r#"
        SELECT table_name, privilege_type
        FROM information_schema.role_table_grants
        WHERE grantee = 'kaya_app'
          AND table_schema = 'fiscalite'
        ORDER BY 1, 2
        "#,
    )
    .fetch_all(&pool)
    .await
    .expect("lecture des privilèges");

    let ecritures: Vec<String> = lignes
        .iter()
        .filter_map(|l| {
            let privilege: String = l.get("privilege_type");
            let table: String = l.get("table_name");
            matches!(privilege.as_str(), "INSERT" | "UPDATE" | "DELETE")
                .then(|| format!("{table}: {privilege}"))
        })
        .collect();

    assert!(
        ecritures.is_empty(),
        "le rôle applicatif a des droits d'écriture sur les provisions : {ecritures:?}\n\
         Aucun chemin d'écriture ne doit pouvoir naître par inadvertance. Le jour où la \
         comptabilité sera implémentée, une migration accordera ces droits — un acte visible et \
         daté."
    );
}

/// La contrainte d'exclusion **fonctionne** — spike de HEB-02.
///
/// Premier usage d'`EXCLUDE USING gist` du produit. HEB-02 reprendra exactement cette forme sur
/// `tstzrange` pour la disponibilité des unités louables ; l'exercer ici, sur un cas sans enjeu,
/// valide `btree_gist` et le mapping de type sqlx 0.9 avant que la double attribution de chambre
/// en dépende.
#[tokio::test]
async fn deux_exercices_qui_se_chevauchent_sont_refuses() {
    let pool = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool, "provisions — exclusion GiST").await;

    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, jeu.tenant_id)
        .await
        .expect("pose du tenant");

    sqlx::query!(
        r#"
        INSERT INTO fiscalite.exercice_comptable (id, tenant_id, debut, fin, statut)
        VALUES ($1, $2, DATE '2026-01-01', DATE '2027-01-01', 'ouvert')
        "#,
        uuid::Uuid::now_v7(),
        jeu.tenant_id
    )
    .execute(&mut *tx)
    .await
    .expect("premier exercice");

    // Chevauchement franc.
    let chevauchant = sqlx::query!(
        r#"
        INSERT INTO fiscalite.exercice_comptable (id, tenant_id, debut, fin, statut)
        VALUES ($1, $2, DATE '2026-06-01', DATE '2027-06-01', 'ouvert')
        "#,
        uuid::Uuid::now_v7(),
        jeu.tenant_id
    )
    .execute(&mut *tx)
    .await;

    assert!(
        chevauchant.is_err(),
        "deux exercices chevauchants ont été acceptés : « la période est-elle close ? » devient \
         indécidable, et c'est la seule règle que TRX-02b impose"
    );

    tx.rollback().await.expect("rollback");

    // Contiguïté : le second commence là où le premier finit. `'[)'` doit l'accepter.
    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, jeu.tenant_id)
        .await
        .expect("pose du tenant");

    sqlx::query!(
        r#"
        INSERT INTO fiscalite.exercice_comptable (id, tenant_id, debut, fin, statut)
        VALUES ($1, $2, DATE '2026-01-01', DATE '2027-01-01', 'ouvert')
        "#,
        uuid::Uuid::now_v7(),
        jeu.tenant_id
    )
    .execute(&mut *tx)
    .await
    .expect("premier exercice");

    let contigu = sqlx::query!(
        r#"
        INSERT INTO fiscalite.exercice_comptable (id, tenant_id, debut, fin, statut)
        VALUES ($1, $2, DATE '2027-01-01', DATE '2028-01-01', 'ouvert')
        "#,
        uuid::Uuid::now_v7(),
        jeu.tenant_id
    )
    .execute(&mut *tx)
    .await;

    assert!(
        contigu.is_ok(),
        "deux exercices CONTIGUS ont été refusés : la borne de fin doit être exclue ('[)'). Avec \
         '[]', le 31 décembre appartiendrait à deux exercices — et HEB-02 hériterait du même \
         défaut sur les occupations. {:?}",
        contigu.err()
    );

    tx.rollback().await.expect("rollback");
}

/// Un exercice **clos** ne se modifie plus — par déclencheur, pas par règle applicative.
#[tokio::test]
async fn un_exercice_clos_ne_se_modifie_plus() {
    let pool = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool, "provisions — période close").await;
    let exercice_id = uuid::Uuid::now_v7();

    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, jeu.tenant_id)
        .await
        .expect("pose du tenant");
    sqlx::query!(
        r#"
        INSERT INTO fiscalite.exercice_comptable (id, tenant_id, debut, fin, statut)
        VALUES ($1, $2, DATE '2025-01-01', DATE '2026-01-01', 'clos')
        "#,
        exercice_id,
        jeu.tenant_id
    )
    .execute(&mut *tx)
    .await
    .expect("exercice clos");
    tx.commit().await.expect("commit");

    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, jeu.tenant_id)
        .await
        .expect("pose du tenant");
    let reouverture = sqlx::query!(
        "UPDATE fiscalite.exercice_comptable SET statut = 'ouvert' WHERE id = $1",
        exercice_id
    )
    .execute(&mut *tx)
    .await;

    assert!(
        reouverture.is_err(),
        "un exercice clos a pu être rouvert. Le déclencheur est ce qui empêche la première \
         migration de données venue de le faire — une règle applicative serait contournée."
    );
    let _ = tx.rollback().await;
}
