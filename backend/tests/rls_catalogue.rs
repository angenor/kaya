//! **Porte P-07** — toute table d'un schéma applicatif porte `ENABLE`, `FORCE` et au moins une
//! politique de sécurité au niveau ligne.
//!
//! # Ce qui distingue cette porte d'une relecture des migrations
//!
//! Elle interroge **le catalogue PostgreSQL**, c'est-à-dire l'état réel de la base après
//! migration, jamais le texte des fichiers de migration (R-09).
//!
//! La différence n'est pas théorique : un `ALTER TABLE ... DISABLE ROW LEVEL SECURITY` ajouté
//! dans une migration ultérieure passerait sans encombre une analyse de texte cherchant
//! « ENABLE ROW LEVEL SECURITY ». Le catalogue, lui, dit ce qui est vrai maintenant.
//!
//! # Trois conditions, trois messages
//!
//! Elles sont vérifiées séparément parce que leurs échecs veulent dire des choses différentes :
//!
//! - `ENABLE` sans `FORCE` — le propriétaire des tables reste **hors politique**. Toute tâche de
//!   maintenance devient une fuite potentielle.
//! - `ENABLE FORCE` sans politique — la table est **bloquée pour tout le monde** au lieu d'être
//!   isolée. Le service tombe, mais aucune donnée ne fuit : c'est un incident, pas une faille.
//!
//! Un message unique ferait chercher la mauvaise chose pendant une heure.

mod commun;

use sqlx::{PgPool, Row};

/// Schémas soumis à la porte. Un schéma applicatif ajouté sans être inscrit ici échapperait à la
/// vérification — c'est pourquoi la liste vit à côté du test qui l'utilise, et non dans un
/// fichier de configuration qu'on oublierait d'ouvrir.
const SCHEMAS_APPLICATIFS: &[&str] = &["etablissements", "synchronisation", "fiscalite"];

/// Liste d'exclusion **nommée**, jamais un motif de nom (R-09).
///
/// Un motif — « tout ce qui commence par `_` », « tout ce qui contient `migration` » — laisserait
/// passer toute table future qui s'y conformerait par accident. Ici, chaque exclusion est un nom
/// complet, et en ajouter un demande d'écrire pourquoi.
///
/// La table de suivi des migrations de sqlx est la seule exclusion à ce cycle. Elle ne porte
/// aucune donnée de client : elle liste des numéros de version appliqués. `backend/api/sqlx.toml`
/// la place d'ailleurs dans son propre schéma `kaya_migrations`, hors des schémas applicatifs —
/// l'exclusion est donc doublement inutile aujourd'hui, et gardée pour le jour où la
/// configuration changerait.
const TABLES_EXCLUES: &[&str] = &["kaya_migrations._migrations_appliquees"];

struct EtatTable {
    nom_complet: String,
    rls_activee: bool,
    rls_forcee: bool,
    nombre_politiques: i64,
}

async fn inventorier(pool: &PgPool) -> Vec<EtatTable> {
    let lignes = sqlx::query(
        r#"
        SELECT n.nspname                            AS schema,
               c.relname                            AS table_nom,
               c.relrowsecurity                     AS rls_activee,
               c.relforcerowsecurity                AS rls_forcee,
               (SELECT COUNT(*)
                  FROM pg_policies p
                 WHERE p.schemaname = n.nspname
                   AND p.tablename  = c.relname)    AS nombre_politiques
          FROM pg_class c
          JOIN pg_namespace n ON n.oid = c.relnamespace
         WHERE c.relkind = 'r'
           AND n.nspname = ANY($1)
         ORDER BY 1, 2
        "#,
    )
    .bind(SCHEMAS_APPLICATIFS)
    .fetch_all(pool)
    .await
    .expect("lecture du catalogue PostgreSQL");

    lignes
        .into_iter()
        .map(|ligne| EtatTable {
            nom_complet: format!(
                "{}.{}",
                ligne.get::<String, _>("schema"),
                ligne.get::<String, _>("table_nom")
            ),
            rls_activee: ligne.get("rls_activee"),
            rls_forcee: ligne.get("rls_forcee"),
            nombre_politiques: ligne.get("nombre_politiques"),
        })
        .filter(|t| !TABLES_EXCLUES.contains(&t.nom_complet.as_str()))
        .collect()
}

/// Applique les trois conditions et renvoie les manquements, un par ligne.
///
/// Extraite du test pour que le **test négatif** ci-dessous puisse l'exercer sur une table
/// délibérément non conforme. Une porte dont on n'a jamais constaté l'échec n'est pas une porte,
/// c'est une intention.
fn manquements(tables: &[EtatTable]) -> Vec<String> {
    let mut manquements = Vec::new();
    for table in tables {
        if !table.rls_activee {
            manquements.push(format!(
                "{} — sécurité au niveau ligne NON ACTIVÉE (ENABLE manquant) : la table est \
                 lisible par tous les tenants",
                table.nom_complet
            ));
        }
        if !table.rls_forcee {
            manquements.push(format!(
                "{} — sécurité au niveau ligne NON FORCÉE (FORCE manquant) : le propriétaire des \
                 tables contourne la politique, donc toute migration ou maintenance voit tous les \
                 clients",
                table.nom_complet
            ));
        }
        if table.nombre_politiques == 0 {
            manquements.push(format!(
                "{} — AUCUNE politique : la table est bloquée pour tout le monde au lieu d'être \
                 isolée",
                table.nom_complet
            ));
        }
    }
    manquements
}

#[tokio::test]
async fn p07_toute_table_applicative_est_isolee() {
    let pool = commun::pool_owner().await;
    let tables = inventorier(&pool).await;

    assert!(
        !tables.is_empty(),
        "aucune table trouvée dans les schémas applicatifs — la porte P-07 n'a rien vérifié. \
         Base non migrée, ou liste SCHEMAS_APPLICATIFS périmée."
    );

    let manquements = manquements(&tables);
    assert!(
        manquements.is_empty(),
        "P-07 ÉCHOUE — {} manquement(s) :\n  {}",
        manquements.len(),
        manquements.join("\n  ")
    );
}

/// **T047** — une transaction sans contexte de tenant ne voit **rien**.
///
/// C'est le point le plus glissant du mécanisme, et le seul dont l'échec serait silencieux.
///
/// La politique compare `current_setting('app.current_tenant', true)`. Le second argument `true`
/// fait que le paramètre absent vaut `NULL` ; la comparaison vaut alors `NULL`, et aucune ligne
/// ne passe. Sans ce second argument, la requête lèverait une **erreur** — ce qui paraît plus
/// sûr, mais ne l'est pas : un `catch` mal placé au-dessus la transformerait en accès ouvert,
/// alors qu'un résultat vide ne peut se dégrader qu'en résultat vide.
#[tokio::test]
async fn p07_sans_contexte_de_tenant_zero_ligne_jamais_une_erreur() {
    let pool_owner = commun::pool_owner().await;
    commun::creer_tenant(&pool_owner, "P-07 sans contexte").await;

    let pool = commun::pool_app().await;
    let mut tx = pool.begin().await.expect("transaction");

    // Aucun appel à `poser_tenant`. C'est tout le sujet du test.
    let compte: Option<i64> =
        sqlx::query_scalar("SELECT COUNT(*) FROM etablissements.etablissement")
            .fetch_one(&mut *tx)
            .await
            .expect(
                "la lecture sans contexte de tenant doit RENVOYER ZÉRO LIGNE, pas lever une \
                 erreur — voir la politique isolation_tenant",
            );

    assert_eq!(
        compte,
        Some(0),
        "une transaction sans contexte de tenant a vu {compte:?} ligne(s) : l'isolation est ouverte"
    );

    tx.rollback().await.expect("rollback");
}

/// **T048 — test négatif.** Une table créée sans politique fait échouer la porte, avec son nom
/// dans le message.
///
/// Le test crée délibérément une table non conforme dans un schéma applicatif, constate que la
/// porte la signale, puis la supprime. Sans cette vérification, rien ne distinguerait une porte
/// qui fonctionne d'une porte qui ne trouve jamais rien.
#[tokio::test]
async fn p07_test_negatif_une_table_sans_politique_fait_echouer_la_porte() {
    let pool = commun::pool_owner().await;

    // `AssertSqlSafe` est légitime **ici et seulement ici** : le SQL est un littéral du test,
    // sans la moindre donnée d'utilisateur (R-03). Sur le chemin qui décide de la visibilité des
    // données, il n'apparaît nulle part — c'est pourquoi `poser_tenant` passe par `set_config`.
    sqlx::raw_sql(sqlx::AssertSqlSafe(
        "CREATE TABLE IF NOT EXISTS etablissements.table_non_conforme_p07 (
             id UUID PRIMARY KEY, tenant_id UUID NOT NULL
         )",
    ))
    .execute(&pool)
    .await
    .expect("création de la table de test");

    let tables = inventorier(&pool).await;
    let manquements = manquements(&tables);

    // Nettoyage avant les assertions : un échec ne doit pas laisser la table derrière lui et
    // faire échouer tous les tests suivants pour une raison sans rapport.
    sqlx::raw_sql(sqlx::AssertSqlSafe(
        "DROP TABLE IF EXISTS etablissements.table_non_conforme_p07",
    ))
    .execute(&pool)
    .await
    .expect("suppression de la table de test");

    assert!(
        !manquements.is_empty(),
        "la porte P-07 n'a rien signalé alors qu'une table sans politique existait : elle ne \
         protège rien"
    );
    assert!(
        manquements
            .iter()
            .any(|m| m.contains("table_non_conforme_p07")),
        "la porte a signalé quelque chose, mais pas la table fautive. Le message doit la nommer, \
         sinon il faut la chercher à la main :\n  {}",
        manquements.join("\n  ")
    );
    assert!(
        manquements
            .iter()
            .any(|m| m.contains("table_non_conforme_p07") && m.contains("AUCUNE politique")),
        "le motif exact doit apparaître, pas seulement le nom de la table"
    );
}
