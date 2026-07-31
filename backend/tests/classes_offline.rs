//! **Porte du registre des classes hors-ligne** — une table non déclarée fait échouer le build.
//!
//! # Le sens de la comparaison, et pourquoi il n'est pas symétrique
//!
//! La comparaison va **de la table vers le registre** (R-10) :
//!
//! - une **table sans déclaration** est une erreur — c'est ce qu'on attrape ;
//! - une **entité déclarée sans table** est normale — `docs/registre-classes-offline.md` décrit
//!   tout le produit, y compris ce que les cycles HEB, PDV, CAI et FIS construiront.
//!
//! Comparer dans les deux sens ferait échouer la porte sur les quelque cent entités du registre
//! qui n'ont pas encore de table, et elle serait désactivée dans la semaine.
//!
//! # La limite assumée, écrite ici plutôt qu'enfouie
//!
//! **Cette porte vérifie la PRÉSENCE d'une entité au registre, jamais la JUSTESSE de sa classe.**
//!
//! Le registre classe des **opérations**, pas seulement des tables : `encaissement` y figure deux
//! fois — en **B** pour les espèces, en **D** pour le Mobile Money — parce que le mode de
//! règlement change ce qui est possible hors ligne. Aucune lecture du schéma ne peut retrouver
//! cette distinction : elle est métier.
//!
//! Prétendre l'automatiser produirait une porte qui ment — elle passerait au vert sur un
//! classement faux, et ce vert donnerait l'assurance qui empêche la relecture humaine. La
//! justesse des classes reste donc **humaine, revue mensuellement** (constitution, § Revue).
//!
//! Ce que la porte garantit, et qui suffit à son objet : **aucune entité ne peut être créée sans
//! que quelqu'un ait ouvert le registre et écrit une classe.** C'est le moment où la question se
//! pose ; c'est celui qu'on manquait avant.

mod commun;

use std::collections::BTreeSet;

use sqlx::Row;

/// Le registre, lu à la compilation. Une modification du fichier recompile le test.
const REGISTRE: &str = include_str!("../../docs/registre-classes-offline.md");

/// Schémas soumis à la porte.
const SCHEMAS_APPLICATIFS: &[&str] = &["etablissements", "synchronisation", "fiscalite"];

/// Tables exclues, **nommées une par une**, jamais par motif.
///
/// Un motif — « tout ce qui commence par `_` » — laisserait passer toute table future qui s'y
/// conformerait par accident. Chaque exclusion doit s'écrire, donc se justifier.
const TABLES_EXCLUES: &[&str] = &[
    // Table de suivi des migrations de sqlx : elle ne porte aucune donnée métier, seulement des
    // numéros de version appliqués. `backend/api/sqlx.toml` la place hors des schémas
    // applicatifs, l'exclusion est donc redondante aujourd'hui — gardée pour le jour où la
    // configuration changerait.
    "_migrations_appliquees",
];

/// Extrait les entités déclarées au registre.
///
/// Le registre est un tableau Markdown dont la première colonne porte les noms d'entités entre
/// accents graves. L'extraction est mécanique — et c'est précisément pourquoi le format du
/// registre ne doit pas changer sans mettre ce test à jour.
fn entites_declarees() -> BTreeSet<String> {
    let mut entites = BTreeSet::new();

    for ligne in REGISTRE.lines() {
        let ligne = ligne.trim();
        if !ligne.starts_with('|') {
            continue;
        }

        // Première cellule du tableau.
        let Some(cellule) = ligne.split('|').nth(1) else {
            continue;
        };

        // Tous les fragments entre accents graves de la cellule : une ligne peut en porter
        // plusieurs — « `mapping_comptable`, `exercice_comptable` » est une seule ligne du §10.
        let mut reste = cellule;
        while let Some(debut) = reste.find('`') {
            let apres = &reste[debut + 1..];
            let Some(fin) = apres.find('`') else { break };
            let brut = &apres[..fin];
            reste = &apres[fin + 1..];

            // « `etablissement.classement` » déclare une colonne : l'entité est la partie avant
            // le point.
            let nom = brut.split('.').next().unwrap_or(brut).trim();
            if !nom.is_empty() && nom.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                entites.insert(nom.to_lowercase());
            }
        }
    }

    entites
}

async fn tables_reelles(pool: &sqlx::PgPool) -> BTreeSet<String> {
    sqlx::query(
        r#"
        SELECT table_name
        FROM information_schema.tables
        WHERE table_schema = ANY($1) AND table_type = 'BASE TABLE'
        "#,
    )
    .bind(SCHEMAS_APPLICATIFS)
    .fetch_all(pool)
    .await
    .expect("lecture du catalogue")
    .into_iter()
    .map(|l| l.get::<String, _>("table_name").to_lowercase())
    .filter(|t| !TABLES_EXCLUES.contains(&t.as_str()))
    .collect()
}

#[tokio::test]
async fn toute_table_est_declaree_au_registre_des_classes_hors_ligne() {
    let pool = commun::pool_owner().await;
    let tables = tables_reelles(&pool).await;
    let declarees = entites_declarees();

    assert!(
        !tables.is_empty(),
        "aucune table trouvée — la porte n'a rien vérifié. Base non migrée ?"
    );
    assert!(
        declarees.len() > 50,
        "seulement {} entités extraites du registre : l'extraction est probablement cassée, et la \
         porte échouerait sur tout. Le format du tableau a-t-il changé ?",
        declarees.len()
    );

    let non_declarees: Vec<&String> = tables.difference(&declarees).collect();

    assert!(
        non_declarees.is_empty(),
        "{} table(s) absente(s) de docs/registre-classes-offline.md :\n  {}\n\n\
         Toute entité déclare sa classe A/B/C/D (principe VI), dans le MÊME changement que sa \
         migration. En cas de doute, classer plus strictement : une entité indûment classée A \
         produit des incohérences silencieuses découvertes trois mois plus tard en pleine \
         clôture ; une entité indûment classée B produit une frustration immédiate, visible et \
         corrigeable.",
        non_declarees.len(),
        non_declarees
            .iter()
            .map(|t| t.as_str())
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}

/// **Test négatif** — une table non déclarée fait bien échouer la porte.
///
/// Sans lui, rien ne distinguerait une porte qui fonctionne d'une porte dont l'extraction est
/// cassée et qui déclarerait tout conforme.
#[tokio::test]
async fn test_negatif_une_table_non_declaree_est_signalee() {
    let pool = commun::pool_owner().await;

    // `AssertSqlSafe` est légitime : le SQL est un littéral du test, sans donnée d'utilisateur.
    sqlx::raw_sql(sqlx::AssertSqlSafe(
        "CREATE TABLE IF NOT EXISTS etablissements.zzz_table_non_declaree (id UUID PRIMARY KEY)",
    ))
    .execute(&pool)
    .await
    .expect("création de la table de test");

    let tables = tables_reelles(&pool).await;
    let declarees = entites_declarees();
    let non_declarees: Vec<&String> = tables.difference(&declarees).collect();

    sqlx::raw_sql(sqlx::AssertSqlSafe(
        "DROP TABLE IF EXISTS etablissements.zzz_table_non_declaree",
    ))
    .execute(&pool)
    .await
    .expect("suppression de la table de test");

    assert!(
        non_declarees
            .iter()
            .any(|t| t.as_str() == "zzz_table_non_declaree"),
        "la porte n'a pas signalé une table absente du registre : elle ne protège rien. \
         Trouvées : {non_declarees:?}"
    );
}

/// Les entités créées par ce cycle sont bien au registre, avec la classe attendue.
///
/// Vérification de **présence uniquement** — voir la limite assumée en tête de fichier. Le fait
/// que `note_etablissement` soit classée A plutôt que B reste une décision humaine ; ce test
/// constate qu'elle a été prise et écrite.
#[test]
fn les_entites_du_cycle_001_sont_declarees() {
    let declarees = entites_declarees();

    for entite in [
        "tenant",
        "etablissement",
        "note_etablissement",
        "evenement_outbox",
        "exercice_comptable",
        "mapping_comptable",
    ] {
        assert!(
            declarees.contains(entite),
            "« {entite} » n'est pas déclarée au registre alors que ce cycle la crée"
        );
    }
}
