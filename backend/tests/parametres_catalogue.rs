//! **Porte de cohérence documentaire** — toute clé du catalogue figure au « Récapitulatif des
//! paramètres d'établissement ».
//!
//! # Ce qu'elle rend vérifiable
//!
//! Le principe I·c pose que « le récapitulatif des paramètres fait foi ». Sans cette porte, c'est
//! une phrase : rien n'empêcherait d'ajouter une clé au catalogue sans l'y inscrire, et le
//! récapitulatif deviendrait faux **sans que personne ne s'en aperçoive** — jusqu'au jour où un
//! exploitant chercherait pourquoi un réglage documenté nulle part change le comportement de son
//! établissement.
//!
//! # Le sens de la comparaison n'est PAS symétrique
//!
//! **Catalogue → récapitulatif**, comme `classes_offline.rs` va de la table vers le registre :
//!
//! - une **clé du catalogue absente du récapitulatif** est une erreur — c'est ce qu'on attrape ;
//! - une **ligne du récapitulatif sans clé** est normale : le récapitulatif décrit tout le produit,
//!   y compris les trente-cinq paramètres que HEB, FIS, CAI, RSV, QRC, CPT, SYN, STK et ADM
//!   livreront.
//!
//! Comparer dans les deux sens ferait échouer la porte sur l'essentiel du récapitulatif, et elle
//! serait désactivée dans la semaine.
//!
//! # Ce que cette porte n'inspecte pas
//!
//! **La justesse de la valeur documentée.** Que `politique_impression` ait tel ou tel jeu de
//! valeurs relève du cycle qui la définit ; ce qui est vérifié ici est que la clé **a été
//! inscrite** — c'est-à-dire que quelqu'un a ouvert le récapitulatif au moment de créer la clé.

mod commun;

use std::collections::BTreeSet;

use sqlx::Row;

/// Le récapitulatif, lu à la compilation. Une modification du fichier recompile le test.
const USER_STORIES: &str = include_str!("../../docs/user-stories-v1.md");

/// Extrait les clés techniques citées au récapitulatif, entre accents graves.
///
/// La convention est la même qu'au registre des classes hors-ligne : le libellé est en français
/// pour le lecteur, la clé technique entre accents graves pour la porte. Une ligne sans clé reste
/// parfaitement valide — c'est le cas des trente-cinq paramètres à venir.
fn cles_du_recapitulatif() -> BTreeSet<String> {
    let mut cles = BTreeSet::new();

    let Some(debut) = USER_STORIES.find("## Récapitulatif des paramètres d'établissement") else {
        panic!(
            "la section « Récapitulatif des paramètres d'établissement » a disparu de \
             docs/user-stories-v1.md. C'est la source que le principe I·c désigne comme faisant \
             foi ; sans elle, cette porte n'a plus rien à comparer."
        );
    };

    // La section s'arrête au titre suivant de même niveau.
    let section = &USER_STORIES[debut..];
    let fin = section[3..].find("\n## ").map(|i| i + 3).unwrap_or(section.len());
    let section = &section[..fin];

    for ligne in section.lines() {
        let ligne = ligne.trim();
        if !ligne.starts_with('|') {
            continue;
        }
        let mut reste = ligne;
        while let Some(d) = reste.find('`') {
            let apres = &reste[d + 1..];
            let Some(f) = apres.find('`') else { break };
            let brut = apres[..f].trim();
            reste = &apres[f + 1..];
            if !brut.is_empty() && brut.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                cles.insert(brut.to_lowercase());
            }
        }
    }

    cles
}

async fn cles_du_catalogue(pool: &sqlx::PgPool) -> BTreeSet<String> {
    sqlx::query("SELECT cle FROM etablissements.parametre_catalogue")
        .fetch_all(pool)
        .await
        .expect("lecture du catalogue")
        .into_iter()
        .map(|l| l.get::<String, _>("cle").to_lowercase())
        .collect()
}

/// **Toute clé du catalogue figure au récapitulatif.**
#[tokio::test]
async fn toute_cle_du_catalogue_figure_au_recapitulatif() {
    let pool = commun::pool_owner().await;
    let catalogue = cles_du_catalogue(&pool).await;
    let recapitulatif = cles_du_recapitulatif();

    assert!(
        !catalogue.is_empty(),
        "le catalogue est vide — la porte n'a rien vérifié. Base non migrée ?"
    );

    let absentes: Vec<&String> = catalogue.difference(&recapitulatif).collect();

    assert!(
        absentes.is_empty(),
        "{} clé(s) du catalogue absente(s) du « Récapitulatif des paramètres d'établissement » de \
         docs/user-stories-v1.md :\n  {}\n\n\
         Le principe I·c pose que le récapitulatif fait foi. Une clé qui n'y figure pas est un \
         paramètre dont l'existence n'est écrite nulle part — et personne ne saura qu'il faut le \
         régler. L'inscription se fait dans le MÊME changement que l'ajout au catalogue, avec la \
         clé technique entre accents graves pour que cette porte la retrouve.",
        absentes.len(),
        absentes
            .iter()
            .map(|c| c.as_str())
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}

/// **Test négatif** — une clé absente du récapitulatif est bien signalée.
///
/// Exercé sur un ensemble simulé : créer une vraie entrée de catalogue ferait échouer le test
/// ci-dessus au hasard de l'ordonnancement, les fichiers de test s'exécutant en parallèle sur une
/// base partagée.
#[test]
fn test_negatif_une_cle_absente_du_recapitulatif_est_signalee() {
    let recapitulatif = cles_du_recapitulatif();

    assert!(
        recapitulatif.contains("politique_impression"),
        "« politique_impression » n'est pas au récapitulatif alors que le cycle 002 l'ajoute au \
         catalogue. L'extraction est-elle cassée, ou la ligne a-t-elle disparu ? Clés \
         extraites : {recapitulatif:?}"
    );

    let mut catalogue_simule = BTreeSet::new();
    catalogue_simule.insert("politique_impression".to_owned()); // au récapitulatif
    catalogue_simule.insert("cle_jamais_documentee".to_owned()); // absente

    let absentes: Vec<&String> = catalogue_simule.difference(&recapitulatif).collect();

    assert!(
        absentes.iter().any(|c| c.as_str() == "cle_jamais_documentee"),
        "la porte n'a pas signalé une clé absente du récapitulatif : elle ne protège rien"
    );
    assert!(
        !absentes.iter().any(|c| c.as_str() == "politique_impression"),
        "la porte signale une clé pourtant documentée : elle échouerait sur tout"
    );
}

/// Le catalogue de ce cycle contient **exactement** la clé annoncée, avec ses attributs.
///
/// Un catalogue à une entrée se justifie parce que le résolveur doit exister **avant** son premier
/// consommateur : le concevoir au cycle HEB le teinterait d'hébergement. Ce test fige ce contenu,
/// pour qu'une clé ajoutée sans décision se voie.
#[tokio::test]
async fn le_catalogue_du_cycle_002_contient_exactement_politique_impression() {
    let pool = commun::pool_owner().await;

    let ligne = sqlx::query(
        r#"
        SELECT cle, type_valeur, portee_la_plus_basse, story
        FROM etablissements.parametre_catalogue
        WHERE cle = 'politique_impression'
        "#,
    )
    .fetch_optional(&pool)
    .await
    .expect("lecture du catalogue")
    .expect("la clé politique_impression doit exister au catalogue");

    assert_eq!(
        ligne.get::<String, _>("portee_la_plus_basse"),
        "POINT_DE_VENTE",
        "la politique d'impression se règle jusqu'au point de vente : c'est la raison pour \
         laquelle elle n'est PAS une colonne de `point_de_vente` (research.md R-04)"
    );
    assert_eq!(ligne.get::<String, _>("story"), "ETB-03");

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM etablissements.parametre_catalogue")
        .fetch_one(&pool)
        .await
        .expect("comptage");
    assert_eq!(
        total, 1,
        "le catalogue porte {total} clé(s) au lieu de la seule annoncée par ce cycle. Une clé \
         ajoutée sans décision doit se voir : elle engage le récapitulatif du principe I·c et tous \
         les cycles qui la liront."
    );
}
