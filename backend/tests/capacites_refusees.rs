//! **Porte P-06** — toute capacité ou tout profil non implémenté est refusé **explicitement,
//! jamais ignoré**.
//!
//! La nuance est tout le sujet. Une valeur **ignorée** laisse croire que la capacité est active :
//! l'exploitant coche « suivi du stock », rien ne se passe, et le défaut se découvre au premier
//! inventaire faux — trois mois plus tard, quand il faut expliquer un écart. Une valeur
//! **refusée** produit un message immédiat qui nomme ce qui manque.
//!
//! # Ce que cette porte inspecte, et ce qu'elle n'inspecte pas
//!
//! *Exigence 1 du § « Couverture des portes » de la constitution.*
//!
//! **Inspecté** — les **neuf valeurs non implémentées** : six capacités (`LIVRAISON`,
//! `PRODUCTION`, `COMMERCE_EN_LIGNE`, `FIDELITE`, `DEVIS`, `COMPTES_CLIENTS`) et trois profils
//! (`AUCUN`, `VALORISE`, `DETAILLE`). Chacune à **deux niveaux** — le référentiel qui la déclare
//! non implémentée, et le refus d'écriture par la base sous le rôle applicatif.
//!
//! **Non inspecté** : la **troisième couche du refus**, l'absence pure à l'interface (principe VII).
//! Elle est vérifiée côté application, par le test de la fonction de sélection et le test de rendu
//! de `G1`. Le `422` de l'API est la deuxième couche ; il ne remplace ni la première ni la
//! troisième (research.md R-02).
//!
//! # Le refus est STRUCTUREL, pas applicatif
//!
//! Une validation applicative se contourne par un import, un script de reprise ou un jeu de
//! données. Le rempart est donc en base, **déclaratif et sans déclencheur** : le référentiel porte
//! `implementee`, la déclaration de consommation le **recopie**, une clé étrangère composite les
//! lie, et un `CHECK` exige que la recopie soit vraie. Déclarer `LIVRAISON` devient impossible —
//! la seule ligne de référentiel qui la porte a `implementee = false`.

mod commun;

use sqlx::Row;

/// Les six capacités déclarées au référentiel et **non implémentées au MVP**.
const CAPACITES_REFUSEES: &[&str] = &[
    "LIVRAISON",
    "PRODUCTION",
    "COMMERCE_EN_LIGNE",
    "FIDELITE",
    "DEVIS",
    "COMPTES_CLIENTS",
];

/// Les trois profils de stock non implémentés.
const PROFILS_REFUSES: &[&str] = &["AUCUN", "VALORISE", "DETAILLE"];

/// Total attendu — comparé au nombre de cas réellement exercés (§ « Couverture des portes »).
const REFUS_ATTENDUS: usize = 9;

async fn table_existe(pool: &sqlx::PgPool, schema: &str, table: &str) -> bool {
    sqlx::query(
        r#"
        SELECT EXISTS (
            SELECT 1 FROM information_schema.tables
            WHERE table_schema = $1 AND table_name = $2
        ) AS existe
        "#,
    )
    .bind(schema)
    .bind(table)
    .fetch_one(pool)
    .await
    .expect("lecture du catalogue")
    .get("existe")
}

/// **Premier niveau — le référentiel déclare la valeur CONNUE et NON IMPLÉMENTÉE.**
///
/// Les six capacités ne sont pas absentes du référentiel : elles y figurent avec
/// `implementee = false`. C'est ce qui permet au refus de distinguer « connu mais non implémenté »
/// de « inconnu » — distinction qu'un `CHECK ... IN ('STOCK')` littéral ne saurait pas faire, et
/// qui change le message rendu à l'exploitant. Elle décide aussi de l'ouverture future : lever le
/// drapeau est une **écriture de configuration**, pas une migration (cadrage §14.4).
#[tokio::test]
async fn p06_les_neuf_valeurs_sont_connues_et_declarees_non_implementees() {
    let pool = commun::pool_owner().await;
    let mut exerces = 0usize;
    let mut manquements = Vec::new();

    for code in CAPACITES_REFUSEES {
        let implementee: Option<bool> = sqlx::query_scalar(
            "SELECT implementee FROM etablissements.capacite WHERE code = $1",
        )
        .bind(code)
        .fetch_optional(&pool)
        .await
        .expect("lecture du référentiel des capacités");

        match implementee {
            None => manquements.push(format!(
                "capacité « {code} » ABSENTE du référentiel. Elle doit y figurer avec \
                 implementee = false : une valeur absente est « inconnue », une valeur présente et \
                 non implémentée est « pas encore » — deux messages différents pour l'exploitant."
            )),
            Some(true) => manquements.push(format!(
                "capacité « {code} » déclarée IMPLÉMENTÉE alors que seule STOCK l'est au MVP. \
                 Le CHECK de module_capacite l'accepterait désormais, et rien ne l'implémente."
            )),
            Some(false) => exerces += 1,
        }
    }

    for code in PROFILS_REFUSES {
        let ligne = sqlx::query(
            "SELECT implementee, motif_refus_cle FROM etablissements.profil_stock WHERE code = $1",
        )
        .bind(code)
        .fetch_optional(&pool)
        .await
        .expect("lecture du référentiel des profils");

        match ligne {
            None => manquements.push(format!("profil « {code} » ABSENT du référentiel")),
            Some(l) if l.get::<bool, _>("implementee") => manquements.push(format!(
                "profil « {code} » déclaré IMPLÉMENTÉ alors que seul SIMPLE l'est au MVP"
            )),
            Some(l) => {
                let motif: Option<String> = l.get("motif_refus_cle");
                match motif {
                    None => manquements.push(format!(
                        "profil « {code} » n'a aucun `motif_refus_cle`. Le refus doit EXPLIQUER, \
                         pas seulement constater — c'est la différence entre « refusé \
                         explicitement » et « ignoré »."
                    )),
                    Some(_) => exerces += 1,
                }
            }
        }
    }

    assert!(
        manquements.is_empty(),
        "P-06 ÉCHOUE — {} manquement(s) :\n  {}",
        manquements.len(),
        manquements.join("\n  ")
    );

    assert_eq!(
        exerces, REFUS_ATTENDUS,
        "P-06 : {exerces} valeur(s) vérifiée(s) au lieu des {REFUS_ATTENDUS} attendues. Une porte \
         qui inspecte un sous-ensemble en croyant tout couvrir est le défaut exact que la \
         constitution a documenté après le cycle 001."
    );
    println!("P-06 — niveau référentiel : {exerces}/{REFUS_ATTENDUS} valeurs refusées, nommées.");
}

/// **Le message d'`AUCUN` est DISTINCT des deux autres profils.**
///
/// `VALORISE` et `DETAILLE` sont des fonctionnalités absentes du MVP — on annonce une absence, et
/// l'exploitant attend une version future. `AUCUN` n'est pas une fonctionnalité manquante : c'est
/// une demande qui n'a pas de sens, puisqu'**une capacité qu'on ne consomme pas ne se déclare
/// simplement pas**.
///
/// Leur donner le même message enverrait attendre une version future une personne qui doit juste
/// ne rien faire. C'est le seul refus du cycle qui **enseigne** quelque chose plutôt que de
/// constater une absence, et c'est pour cela qu'il est testé à part.
#[tokio::test]
async fn p06_le_refus_d_aucun_enseigne_au_lieu_de_constater() {
    let pool = commun::pool_owner().await;

    let motifs: Vec<(String, Option<String>)> = sqlx::query(
        "SELECT code, motif_refus_cle FROM etablissements.profil_stock WHERE implementee = false",
    )
    .fetch_all(&pool)
    .await
    .expect("lecture des motifs de refus")
    .into_iter()
    .map(|l| (l.get("code"), l.get("motif_refus_cle")))
    .collect();

    let motif_aucun = motifs
        .iter()
        .find(|(code, _)| code == "AUCUN")
        .and_then(|(_, m)| m.clone())
        .expect("le profil AUCUN doit porter un motif de refus");

    for (code, motif) in &motifs {
        if code == "AUCUN" {
            continue;
        }
        assert_ne!(
            motif.as_deref(),
            Some(motif_aucun.as_str()),
            "le profil « {code} » partage le motif de refus d'AUCUN.\n\
             AUCUN dit « une capacité non consommée ne se déclare pas » ; {code} dit « pas encore \
             implémenté ». Les confondre fait attendre une version future à quelqu'un qui doit \
             juste ne rien faire."
        );
    }
}

/// **Second niveau — le refus par la base, sous le rôle applicatif.**
///
/// Assertion de non-régression tant que `module_capacite` n'existe pas : dès que la table
/// apparaît, ce test doit être remplacé par les neuf tentatives d'`INSERT` direct, chacune
/// vérifiant qu'**aucune ligne n'est écrite**.
///
/// Le premier niveau seul ne suffit pas : un référentiel bien rempli n'empêche rien si la table
/// qui le consomme ne recopie pas `implementee` et ne porte pas son `CHECK`.
#[tokio::test]
async fn p06_second_niveau_le_refus_est_tenu_par_la_base() {
    let pool = commun::pool_owner().await;

    assert!(
        !table_existe(&pool, "etablissements", "module_capacite").await,
        "P-06 : `etablissements.module_capacite` existe désormais, mais le second niveau de la \
         porte est encore une assertion à vide.\n\
         La tâche qui a créé la table doit, dans le MÊME changement, écrire ici :\n\
           1. les NEUF tentatives d'INSERT direct sous le rôle applicatif — six capacités, trois \
              profils — chacune constatant une violation de contrainte ET zéro ligne écrite ;\n\
           2. le cas nominal STOCK/SIMPLE, qui doit passer.\n\
         Sans le second niveau, le refus repose sur le seul service — donc se contourne par tout \
         import, script de reprise ou jeu de données."
    );
}
