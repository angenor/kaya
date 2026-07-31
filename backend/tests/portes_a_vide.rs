//! **Portes P-06, P-09 et P-11 — installées À VIDE, avec assertion de non-régression** (R-15).
//!
//! # Pourquoi installer une porte qui n'a rien à vérifier
//!
//! Ces trois portes n'ont **aucune cible** au cycle 001 : il n'existe ni référentiel de capacités
//! (ETB-02b), ni occupation (HEB-02), ni calcul fiscal (T3).
//!
//! Une porte ajoutée « quand on en aura besoin » n'est jamais ajoutée — ou elle l'est après que
//! trois cycles ont écrit du code non conforme, et il faut alors choisir entre corriger
//! rétroactivement et renoncer à la règle. Une porte verte à vide coûte une poignée de lignes et
//! garantit qu'aucun cycle ultérieur ne pourra livrer sans la rencontrer.
//!
//! # Le piège de la porte verte à vide, et ce qui le neutralise
//!
//! Une porte qui ne trouve rien est indistinguable d'une porte qui n'a rien à trouver. Elle
//! resterait donc verte le jour où sa cible existerait mais où elle aurait cessé de la voir — un
//! nom de table changé, un répertoire déplacé.
//!
//! Chaque test porte donc une **assertion de non-régression** : il échoue si sa cible apparaît
//! sans que la porte ne soit activée. Le cycle qui crée la cible ne peut pas l'ignorer — c'est son
//! propre build qui le lui dit.

mod commun;

use sqlx::Row;

/// Une table existe-t-elle dans un schéma applicatif ?
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

/// **P-06 — LEVÉE au cycle 002.** La porte a désormais ses cibles, elle n'est plus à vide.
///
/// L'assertion de non-régression a fonctionné exactement comme prévu : la migration
/// `0008_referentiels_activite.sql` a créé `etablissements.capacite`, ce test a échoué, et la
/// porte a été activée **dans le même changement**. C'est le comportement qu'on attendait d'elle
/// sans jamais l'avoir vu — il est consigné ici plutôt que dans un message de commit, parce que
/// c'est la seule preuve qu'une porte à vide sait se réveiller.
///
/// Ce qui reste ici est le **chaînage** : P-06 vit maintenant dans
/// `backend/tests/capacites_refusees.rs`, et ce test échoue si ce fichier disparaît. Sans ce
/// relais, supprimer la porte réelle ne casserait plus rien — l'assertion à vide ayant été
/// retirée, plus personne ne réclamerait sa présence.
#[test]
fn p06_est_levee_et_vit_desormais_dans_son_propre_fichier() {
    use std::path::Path;

    assert!(
        Path::new("tests/capacites_refusees.rs").exists(),
        "P-06 : `backend/tests/capacites_refusees.rs` a disparu.\n\
         La porte a été levée de `portes_a_vide.rs` au cycle 002 quand `etablissements.capacite` \
         est apparue ; son contenu réel vit désormais dans ce fichier. Le supprimer laisserait \
         P-06 sans aucune cible ET sans assertion à vide — c'est-à-dire silencieusement absente."
    );
}

/// **P-09** — toute occupation est un `tstzrange` protégé par une contrainte d'exclusion GiST.
///
/// Cible attendue : la table `occupation` de **HEB-02**.
///
/// # Partiellement exercée dès ce cycle
///
/// `fiscalite.exercice_comptable` utilise `EXCLUDE USING gist` sur `daterange`. Ce n'est pas une
/// occupation, mais c'est **le même mécanisme** : le spike valide `btree_gist` et le mapping de
/// type sqlx 0.9 avant que la disponibilité des unités n'en dépende. Voir
/// `backend/tests/provisions_sans_logique.rs`.
#[tokio::test]
async fn p09_occupation_protegee_par_exclusion_gist() {
    let pool = commun::pool_owner().await;

    // Assertion de non-régression : le mécanisme lui-même doit rester exercé.
    let exercice = table_existe(&pool, "fiscalite", "exercice_comptable").await;
    assert!(
        exercice,
        "P-09 : `fiscalite.exercice_comptable` a disparu. C'était le SEUL usage d'EXCLUDE USING \
         gist du produit — sans lui, plus rien ne valide le mécanisme sur lequel HEB-02 s'appuiera \
         pour empêcher la double attribution de chambre."
    );

    let occupation = table_existe(&pool, "hebergement", "occupation").await;
    assert!(
        !occupation,
        "P-09 : la table `occupation` existe désormais, mais la porte est toujours installée à \
         vide.\n\
         HEB-02 doit, dans le MÊME changement, vérifier ici que :\n\
           1. la période est un `tstzrange`, JAMAIS une paire de dates — le marché pratique \
              massivement le passage horaire et la demi-journée ;\n\
           2. une contrainte `EXCLUDE USING gist (unite_id WITH =, periode WITH &&)` la protège ;\n\
           3. deux attributions concurrentes chevauchantes échouent — pas « improbablement », \
              jamais."
    );
}

/// **P-11** — tests dorés fiscaux verts sur jeux de cas figés.
///
/// Cible attendue : les jeux de cas de **FIS-01 à FIS-07**, tranche T3.
///
/// Le harnais est installé avec un jeu vide. L'assertion de non-régression porte sur le
/// répertoire : dès qu'un jeu fiscal y apparaît, ce test doit être remplacé par son exécution.
#[test]
fn p11_tests_dores_fiscaux() {
    use std::path::Path;

    let repertoire = Path::new("tests/fixtures/fiscal");
    // Les fichiers cachés — `.gitkeep` — ne sont pas des jeux de cas : ils existent pour que Git
    // conserve un répertoire vide.
    let jeux: Vec<_> = std::fs::read_dir(repertoire)
        .map(|entrees| {
            entrees
                .flatten()
                .map(|e| e.path())
                .filter(|c| {
                    !c.file_name()
                        .and_then(|n| n.to_str())
                        .is_some_and(|n| n.starts_with('.'))
                })
                .collect()
        })
        .unwrap_or_default();

    assert!(
        jeux.is_empty(),
        "P-11 : {} jeu(x) de cas fiscal présent(s) dans {}, mais la porte est toujours installée \
         à vide.\n\
         Le cycle qui les a ajoutés doit, dans le MÊME changement, écrire ici leur exécution.\n\
         Jeux trouvés : {:?}",
        jeux.len(),
        repertoire.display(),
        jeux
    );

    // Le crate `socle/fiscalite` doit exister : c'est là que les règles vivront, et la porte P-12
    // vérifie qu'elles ne vivent nulle part ailleurs.
    assert!(
        Path::new("crates/socle/fiscalite/src/lib.rs").exists(),
        "P-11 : le crate socle/fiscalite a disparu — les tests dorés n'auraient plus de cible."
    );
}

/// Les trois portes ci-dessus sont-elles **toutes** encore à vide ?
///
/// Récapitulatif exécutable, pour qu'un développeur sache d'un coup d'œil ce qui reste dû, sans
/// relire trois tests.
#[test]
fn recapitulatif_des_portes_installees_a_vide() {
    let a_vide = ["P-09 (HEB-02)", "P-11 (T3, FIS-01 à FIS-07)"];
    let levees = ["P-06 (ETB-02b) → backend/tests/capacites_refusees.rs"];

    println!("Portes encore installées à vide, avec assertion de non-régression :");
    for porte in a_vide {
        println!("  · {porte}");
    }
    println!();
    println!("Levées depuis le cycle 001 — leur cible existe et la porte est active :");
    for porte in levees {
        println!("  · {porte}");
    }
    println!();
    println!("Chaque porte à vide échouera dès que sa cible apparaîtra sans qu'elle soit activée.");

    // Trois au cycle 001, deux désormais : P-06 a été levée par le cycle 002. Le décompte est
    // asserté pour qu'une porte retirée sans être levée — donc sans remplaçante — se voie.
    assert_eq!(a_vide.len() + levees.len(), 3);
}
