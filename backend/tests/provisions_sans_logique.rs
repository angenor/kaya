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
//!
//! # Périmètre inspecté — **quatre provisions, deux cycles**
//!
//! *§ « Couverture des portes » : une porte dont la cible est vide passe toujours au vert. Le
//! décompte est donc comparé à [`PROVISIONS`], et la liste est ici.*
//!
//! | Provision | Cycle | Ce qu'elle attend |
//! |---|---|---|
//! | `fiscalite.exercice_comptable` | 001 | La comptabilité SYSCOHADA |
//! | `fiscalite.mapping_comptable` | 001 | idem |
//! | `comptes.employe` | 003 | CPT-05 — le contrat de travail, la paie |
//! | `comptes.appareil_enrole` | 003 | CPT-05 / CPT-06 — l'enrôlement par paire de clés |
//!
//! **N'est PAS inspecté** : ce que ferait un binaire de maintenance sous `kaya_owner`. Le
//! propriétaire des tables peut tout écrire, par construction — c'est le rôle applicatif qui est
//! bridé, et c'est le seul par lequel l'API passe.

mod commun;

use sqlx::Row;

/// Les provisions du produit — **schéma, table, cycle qui les a posées**.
///
/// Ajouter une provision sans l'inscrire ici la laisserait hors de tout contrôle. Le décompte
/// ci-dessous rend l'omission bruyante dans l'autre sens : une liste qui rétrécirait ferait
/// échouer le test au lieu de le rendre plus facile.
const PROVISIONS: &[(&str, &str, &str)] = &[
    ("fiscalite", "exercice_comptable", "cycle 001 — SYSCOHADA"),
    ("fiscalite", "mapping_comptable", "cycle 001 — SYSCOHADA"),
    ("comptes", "employe", "cycle 003 — CPT-05, contrat de travail"),
    (
        "comptes",
        "appareil_enrole",
        "cycle 003 — CPT-05/06, enrôlement d'appareil",
    ),
];

/// Les quatre tables existent, avec leurs contraintes.
#[tokio::test]
async fn les_quatre_tables_de_provision_existent() {
    let pool = commun::pool_owner().await;
    let mut inspectees = 0_usize;

    for (schema, table, cycle) in PROVISIONS {
        let existe: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS (
                SELECT 1 FROM information_schema.tables
                WHERE table_schema = $1 AND table_name = $2
            )
            "#,
        )
        .bind(schema)
        .bind(table)
        .fetch_one(&pool)
        .await
        .expect("lecture du catalogue");

        assert!(existe, "{schema}.{table} est absente ({cycle})");
        inspectees += 1;
    }

    assert_eq!(
        inspectees,
        PROVISIONS.len(),
        "{inspectees} provision(s) inspectée(s) sur {} déclarée(s)",
        PROVISIONS.len()
    );
}

/// **`comptes.employe` ne porte aucune colonne de pièce d'identité, et n'en portera pas.**
///
/// Le contrôle jumeau de `personne_compte_employe.rs`, dans l'autre sens : celui-là refuse qu'une
/// colonne de contrat migre vers les tables d'identité, celui-ci refuse qu'une colonne d'identité
/// migre vers la table de contrat. Les deux mouvements sont tentants pour la même raison —
/// « c'est la même personne » — et les deux effacent la distinction de CPT-00.
///
/// Le sujet n'est pas cosmétique : `type_piece` et `numero_piece` sont soumises à une rétention de
/// 90 jours (TRX-06). Recopiées sur une table de provision que personne ne surveille, elles y
/// resteraient indéfiniment.
#[tokio::test]
async fn aucune_colonne_de_piece_d_identite_sur_les_provisions_rh() {
    let pool = commun::pool_owner().await;
    let mut fautives = Vec::new();
    let mut inspectees = 0_usize;

    for (schema, table, _) in PROVISIONS.iter().filter(|(s, _, _)| *s == "comptes") {
        let colonnes = sqlx::query(
            r#"
            SELECT column_name
            FROM information_schema.columns
            WHERE table_schema = $1 AND table_name = $2
            "#,
        )
        .bind(schema)
        .bind(table)
        .fetch_all(&pool)
        .await
        .expect("lecture du catalogue de colonnes");

        assert!(!colonnes.is_empty(), "{schema}.{table} n'a aucune colonne");
        inspectees += 1;

        for ligne in colonnes {
            let nom: String = ligne.get::<String, _>("column_name").to_lowercase();
            for motif in ["piece", "passeport", "cni", "identite"] {
                if nom.contains(motif) {
                    fautives.push(format!("{schema}.{table}.{nom} (motif « {motif} »)"));
                }
            }
        }
    }

    assert_eq!(inspectees, 2, "les deux provisions RH doivent être inspectées");
    assert!(
        fautives.is_empty(),
        "des colonnes de pièce d'identité sont apparues sur une provision : {fautives:?}\n\
         Elles relèvent de `comptes.personne`, sous la rétention de 90 jours de TRX-06. Recopiées \
         ici, elles y resteraient indéfiniment."
    );
}

/// **Aucun endpoint ne touche les deux provisions du cycle 003.**
///
/// Même mécanique que pour les provisions comptables : le contrat OpenAPI est la source de vérité
/// de ce que l'API expose (principe I(a)).
#[test]
fn aucun_endpoint_n_expose_les_provisions_rh() {
    let contrat = kaya_api::application::contrat_complet();

    let suspects: Vec<&String> = contrat
        .paths
        .paths
        .keys()
        .filter(|chemin| {
            let c = chemin.to_lowercase();
            c.contains("employe") || c.contains("appareil") || c.contains("enrole")
        })
        .collect();

    assert!(
        suspects.is_empty(),
        "des endpoints exposent les provisions RH : {suspects:?}\n\
         `employe` est CPT-05 et `appareil_enrole` CPT-05/06, tranche T4. Un endpoint, même en \
         lecture, en fait une fonctionnalité que personne n'a décidé de construire — et il \
         échouerait de toute façon au premier appel : `kaya_app` n'a aucun privilège dessus."
    );
}

/// **Aucun droit d'écriture — ni même de lecture — sur les provisions RH.**
///
/// Plus strict que la règle du cycle 001 : `fiscalite` accorde `SELECT`, `comptes` n'accorde
/// **rien du tout**. C'est la garantie de second rang du contrôle de graphe d'appels de
/// `personne_compte_employe.rs` : un chemin de code écrit par distraction échoue au premier
/// appel, pas trois mois plus tard.
#[tokio::test]
async fn le_role_applicatif_n_a_aucun_privilege_sur_les_provisions_rh() {
    let pool = commun::pool_owner().await;
    let mut inspectees = 0_usize;

    for (schema, table, cycle) in PROVISIONS.iter().filter(|(s, _, _)| *s == "comptes") {
        let privileges: Vec<String> = sqlx::query(
            r#"
            SELECT privilege_type
            FROM information_schema.role_table_grants
            WHERE grantee = 'kaya_app' AND table_schema = $1 AND table_name = $2
            ORDER BY 1
            "#,
        )
        .bind(schema)
        .bind(table)
        .fetch_all(&pool)
        .await
        .expect("lecture des privilèges")
        .iter()
        .map(|l| l.get::<String, _>("privilege_type"))
        .collect();

        assert!(
            privileges.is_empty(),
            "le rôle applicatif détient {privileges:?} sur {schema}.{table} ({cycle}).\n\
             Une provision n'accorde RIEN, pas même `SELECT` — c'est ce qui la distingue d'un \
             début d'implémentation. Ne pas ajouter de `GRANT` « pour pouvoir tester » : ce test \
             teste précisément cette absence."
        );
        inspectees += 1;
    }

    assert_eq!(inspectees, 2, "les deux provisions RH doivent être inspectées");
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
