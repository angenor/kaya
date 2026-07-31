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

/// **Les quatre référentiels globaux — conformes, et NOMMÉS.**
///
/// Ils n'ont pas de `tenant_id` : ils sont le même référentiel pour tous les clients. Ce n'est
/// **pas une dispense** de la porte P-07 mais un **régime nommé** (research.md R-01) — deux
/// politiques au lieu d'une, et un jeu de privilèges asymétrique :
///
/// - `lecture_universelle` — `FOR SELECT USING (true)` ;
/// - `administration_editeur` — `FOR ALL TO kaya_owner`, l'écriture appartient à l'éditeur ;
/// - `GRANT SELECT` seul à `kaya_app`, qui est donc refusé **deux fois**.
///
/// Les nommer ici plutôt que les exclure change ce que la porte garantit : une table de
/// référentiel ajoutée demain sans sa politique d'administration serait attrapée, alors qu'une
/// exclusion par motif l'aurait laissée passer.
const REFERENTIELS_GLOBAUX: &[&str] = &[
    "etablissements.module_activite",
    "etablissements.capacite",
    "etablissements.profil_stock",
    "etablissements.parametre_catalogue",
];

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

/// Applique les trois conditions du régime des référentiels globaux.
///
/// Extraite pour que le **test négatif** puisse l'exercer sur un référentiel délibérément non
/// conforme, sans créer la table fautive en base — les fichiers de test s'exécutent en parallèle
/// sur une base partagée, et une table apparue le temps d'un inventaire ferait échouer au hasard
/// les deux portes qui inventorient (leçon du cycle 001, plus bas dans ce fichier).
fn manquements_referentiel(
    nom_complet: &str,
    politiques: &[String],
    ecritures_kaya_app: &[String],
    a_tenant_id: bool,
) -> Vec<String> {
    let mut manquements = Vec::new();

    for attendue in ["lecture_universelle", "administration_editeur"] {
        if !politiques.iter().any(|p| p == attendue) {
            manquements.push(format!(
                "{nom_complet} — politique « {attendue} » absente. Le régime des référentiels \
                 globaux en exige DEUX : sans `lecture_universelle`, aucun tenant ne voit le \
                 référentiel ; sans `administration_editeur`, le propriétaire lui-même ne peut \
                 plus l'alimenter sous FORCE ROW LEVEL SECURITY — et la migration qui essaiera \
                 échouera sur une table pourtant vide."
            ));
        }
    }

    if !ecritures_kaya_app.is_empty() {
        manquements.push(format!(
            "{nom_complet} — `kaya_app` détient {ecritures_kaya_app:?} sur un RÉFÉRENTIEL. Il doit \
             être refusé deux fois : aucun privilège d'écriture, et aucune politique qui \
             l'autoriserait. Un GRANT accordé par erreur suffit à défaire la seconde barrière."
        ));
    }

    if a_tenant_id {
        manquements.push(format!(
            "{nom_complet} — porte une colonne `tenant_id` alors qu'il est déclaré référentiel \
             GLOBAL. Dupliquer le référentiel par client multiplie ses lignes par le nombre de \
             tenants et rend impossible l'ajout d'une valeur « par configuration » (cadrage \
             §14.3) : il faudrait l'écrire chez chacun."
        ));
    }

    manquements
}

/// **T008 — le régime des référentiels globaux, vérifié plutôt que supposé.**
///
/// Trois conditions, chacune contre une faute précise :
///
/// 1. **deux politiques nommées** — `lecture_universelle` et `administration_editeur`. Une table
///    de référentiel qui n'aurait que la première serait en lecture seule pour tout le monde, y
///    compris pour les migrations de l'éditeur ; qui n'aurait que la seconde serait invisible aux
///    tenants ;
/// 2. **aucun droit d'écriture pour `kaya_app`** — le privilège dit la règle mieux qu'un
///    commentaire : l'enrichissement du référentiel relève d'ETB-08, aucun tenant n'y écrit ;
/// 3. **aucune colonne `tenant_id`** — sa présence signifierait que quelqu'un a commencé à
///    dupliquer le référentiel par client, ce que le cadrage §14.3 exclut.
#[tokio::test]
async fn p07_les_referentiels_globaux_ont_leur_regime_nomme() {
    let pool = commun::pool_owner().await;
    let mut manquements = Vec::new();

    for nom_complet in REFERENTIELS_GLOBAUX {
        let (schema, table) = nom_complet.split_once('.').expect("nom qualifié");

        let politiques: Vec<String> = sqlx::query_scalar(
            "SELECT policyname FROM pg_policies WHERE schemaname = $1 AND tablename = $2",
        )
        .bind(schema)
        .bind(table)
        .fetch_all(&pool)
        .await
        .expect("lecture des politiques");

        let ecritures: Vec<String> = sqlx::query_scalar(
            r#"
            SELECT privilege_type
            FROM information_schema.role_table_grants
            WHERE table_schema = $1 AND table_name = $2 AND grantee = 'kaya_app'
              AND privilege_type IN ('INSERT', 'UPDATE', 'DELETE', 'TRUNCATE')
            "#,
        )
        .bind(schema)
        .bind(table)
        .fetch_all(&pool)
        .await
        .expect("lecture des privilèges");

        let a_tenant_id: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS (
                SELECT 1 FROM information_schema.columns
                WHERE table_schema = $1 AND table_name = $2 AND column_name = 'tenant_id'
            )
            "#,
        )
        .bind(schema)
        .bind(table)
        .fetch_one(&pool)
        .await
        .expect("lecture des colonnes");

        manquements.extend(manquements_referentiel(
            nom_complet,
            &politiques,
            &ecritures,
            a_tenant_id,
        ));
    }

    assert!(
        manquements.is_empty(),
        "P-07 (régime des référentiels globaux) ÉCHOUE — {} manquement(s) :\n  {}",
        manquements.len(),
        manquements.join("\n  ")
    );

    println!(
        "P-07 — {} référentiels globaux inspectés et conformes :",
        REFERENTIELS_GLOBAUX.len()
    );
    for nom in REFERENTIELS_GLOBAUX {
        println!("  · {nom} — lecture_universelle + administration_editeur, SELECT seul à kaya_app");
    }
}

/// **T008 — test négatif.** Un référentiel sans `administration_editeur` fait échouer la porte.
///
/// C'est le manquement le plus vraisemblable, et le plus trompeur : la table serait lisible par
/// tous, tous les tests de lecture passeraient, et l'échec ne surviendrait qu'à la **prochaine
/// migration** qui tenterait d'y insérer une valeur — sous `FORCE ROW LEVEL SECURITY`, le
/// propriétaire sans politique d'écriture n'écrit rien.
#[test]
fn p07_test_negatif_un_referentiel_sans_administration_editeur_est_signale() {
    let conforme = manquements_referentiel(
        "etablissements.module_activite",
        &[
            "lecture_universelle".to_owned(),
            "administration_editeur".to_owned(),
        ],
        &[],
        false,
    );
    assert!(
        conforme.is_empty(),
        "la porte signale un référentiel pourtant conforme : elle échouerait sur les quatre, et \
         serait désactivée dans la semaine :\n  {}",
        conforme.join("\n  ")
    );

    let sans_administration = manquements_referentiel(
        "etablissements.referentiel_non_conforme",
        &["lecture_universelle".to_owned()],
        &[],
        false,
    );
    assert!(
        sans_administration
            .iter()
            .any(|m| m.contains("referentiel_non_conforme") && m.contains("administration_editeur")),
        "la porte n'a pas signalé l'absence d'`administration_editeur` : elle laisserait passer le \
         manquement dont l'échec est le plus tardif et le plus obscur.\n  {}",
        sans_administration.join("\n  ")
    );

    // Les deux autres conditions, exercées séparément : leurs échecs veulent dire des choses
    // très différentes, et un message unique ferait chercher la mauvaise chose.
    let ecriture_ouverte = manquements_referentiel(
        "etablissements.referentiel_non_conforme",
        &[
            "lecture_universelle".to_owned(),
            "administration_editeur".to_owned(),
        ],
        &["INSERT".to_owned()],
        false,
    );
    assert!(
        ecriture_ouverte.iter().any(|m| m.contains("kaya_app")),
        "un privilège d'écriture accordé à kaya_app sur un référentiel n'est pas signalé"
    );

    let avec_tenant_id = manquements_referentiel(
        "etablissements.referentiel_non_conforme",
        &[
            "lecture_universelle".to_owned(),
            "administration_editeur".to_owned(),
        ],
        &[],
        true,
    );
    assert!(
        avec_tenant_id.iter().any(|m| m.contains("tenant_id")),
        "un `tenant_id` apparu sur un référentiel global n'est pas signalé"
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

/// **T048 — test négatif.** Une table sans politique fait échouer la porte, avec son nom dans le
/// message.
///
/// # Pourquoi la table fautive n'est PAS créée en base
///
/// Une première version la créait réellement, le temps de l'inventaire. Les fichiers de test
/// s'exécutent en parallèle sur une base partagée : la table apparaissait donc dans le catalogue
/// pendant que `p07_toute_table_applicative_est_isolee` et la porte du registre inventoriaient —
/// et l'un ou l'autre échouait, au hasard de l'ordonnancement.
///
/// Un test qui casse ses voisins est un test qu'on finit par ignorer. Les trois conditions sont
/// donc exercées sur un état **simulé**, ce qui vérifie exactement ce qui compte : la fonction
/// `manquements` signale bien chaque défaut, et le nomme.
#[test]
fn p07_test_negatif_une_table_sans_politique_fait_echouer_la_porte() {
    let tables = vec![
        // Conforme — ne doit rien produire.
        EtatTable {
            nom_complet: "etablissements.note_etablissement".to_owned(),
            rls_activee: true,
            rls_forcee: true,
            nombre_politiques: 1,
        },
        // Les trois défauts, sur une même table fictive.
        EtatTable {
            nom_complet: "etablissements.table_non_conforme_p07".to_owned(),
            rls_activee: false,
            rls_forcee: false,
            nombre_politiques: 0,
        },
    ];

    let manquements = manquements(&tables);

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
    // Les trois conditions sont vérifiées SÉPARÉMENT, avec des messages distincts : `ENABLE`
    // sans `FORCE` laisse le propriétaire hors politique, `ENABLE FORCE` sans politique bloque
    // tout au lieu d'isoler. Deux situations très différentes — un message unique ferait
    // chercher la mauvaise chose pendant une heure.
    for motif in ["NON ACTIVÉE", "NON FORCÉE", "AUCUNE politique"] {
        assert!(
            manquements
                .iter()
                .any(|m| m.contains("table_non_conforme_p07") && m.contains(motif)),
            "le motif « {motif} » doit apparaître dans les manquements :\n  {}",
            manquements.join("\n  ")
        );
    }

    assert!(
        !manquements
            .iter()
            .any(|m| m.contains("note_etablissement")),
        "la porte signale une table pourtant conforme : elle échouerait sur tout"
    );
}
