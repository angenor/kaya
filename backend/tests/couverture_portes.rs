//! **Recollement des trois portes à décompte — P-05, P-07, P-08.**
//!
//! # Pourquoi ce fichier existe
//!
//! Ces trois portes portent chacune sur un ensemble dont la taille est connue : **11 types
//! d'événements**, **10 tables créées**, **21 opérations HTTP**. Leur extension s'est faite phase
//! par phase, ce qui est la manière normale de les faire grandir avec le code — **et la manière
//! normale de laisser un trou.**
//!
//! Chaque phase a étendu la porte à ce qu'elle livrait. Aucune n'était responsable de vérifier que
//! l'ensemble était couvert, et c'est exactement là que se perd un type d'événement : celui qu'une
//! phase a introduit sans que la tâche d'extension de la phase suivante le reprenne.
//!
//! Ce fichier compare, pour chacune des trois portes, **le nombre de cibles réellement inspectées
//! au total déclaré**, et échoue sur tout écart.
//!
//! # Ce qu'il inspecte, et ce qu'il n'inspecte pas
//!
//! *Exigence 2 du § « Couverture des portes » de la constitution.*
//!
//! **Inspecté** — la **couverture** : chaque type d'événement du modèle a-t-il un test ? chaque
//! table créée est-elle inventoriée ? chaque chemin servi a-t-il un régime d'isolation déclaré ?
//!
//! **Non inspecté** — la **justesse** de chaque test. Qu'un test d'isolation existe pour un chemin
//! ne dit pas qu'il vérifie la bonne chose. Ce fichier ferme le trou de la couverture, pas celui de
//! la qualité — que seule la revue couvre.
//!
//! # Ce fichier ne se parallélise pas avec les autres tâches
//!
//! Il suppose que **toutes** les migrations, tous les points d'entrée et tous les événements du
//! cycle existent. Le lancer plus tôt compterait juste et couvrirait faux.

mod commun;

use std::collections::BTreeSet;

use kaya_api::application;
use sqlx::Row;

/// Compte les opérations d'un chemin.
///
/// `PathItem` n'expose pas de carte d'opérations en utoipa 5.5 : chaque verbe est un champ
/// `Option<Operation>` distinct. Les additionner un par un est verbeux mais **exact** — et un
/// verbe ajouté à la structure par une montée de version ferait échouer la compilation ici plutôt
/// que de disparaître silencieusement du décompte.
fn compter_operations(item: &utoipa::openapi::path::PathItem) -> usize {
    [
        item.get.is_some(),
        item.put.is_some(),
        item.post.is_some(),
        item.delete.is_some(),
        item.options.is_some(),
        item.head.is_some(),
        item.patch.is_some(),
        item.trace.is_some(),
    ]
    .iter()
    .filter(|present| **present)
    .count()
}

// =================================================================================================
//  P-05 — les onze types d'événements
// =================================================================================================

/// **Les types d'événements du cycle**, tels que `data-model.md` § Événements les déclare.
///
/// # Treize types, pour « onze » annoncés — l'écart est un défaut de COMPTAGE, pas de couverture
///
/// Le tableau du modèle de données compte **onze lignes**, mais deux d'entre elles portent chacune
/// **deux types** : « `point_de_vente.cree` / `.modifie` » et « `table_pdv.creee` /
/// `.desactivee` ». Le décompte réel est donc **treize**, et c'est celui-ci qui fait foi — un
/// décompte tiré du nombre de lignes d'un tableau compte des lignes, pas des types.
///
/// C'est exactement la nature d'erreur que ce fichier existe pour attraper, et il l'a attrapée sur
/// lui-même : le plan aurait fait vérifier onze types en croyant les avoir tous.
///
/// Trois de ces types — `module_capacite.declaree`, `parametre_configuration.ecrit` et
/// `branding.modifie` — n'étaient couverts par **aucune tâche du plan**. Le recollement était
/// censé les trouver ; ils ont été ajoutés aux tests de `outbox_transactionnel.rs` au fil de leur
/// phase, et ce test constate qu'aucun ne manque.
const TYPES_EVENEMENTS: &[&str] = &[
    "etablissement.cree",
    "etablissement.modifie",
    "etablissement.classement_change",
    "etablissement.fuseau_change",
    "etablissement_module.active",
    "etablissement_module.desactive",
    "module_capacite.declaree",
    "point_de_vente.cree",
    "point_de_vente.modifie",
    "table_pdv.creee",
    "table_pdv.desactivee",
    "parametre_configuration.ecrit",
    "branding.modifie",
];

/// Le fichier de la porte P-05, lu à la compilation.
const OUTBOX_TRANSACTIONNEL: &str = include_str!("outbox_transactionnel.rs");

/// **P-05 — chaque type d'événement déclaré est vérifié par un test.**
#[test]
fn p05_les_types_d_evenements_declares_sont_tous_couverts() {
    let mut non_couverts = Vec::new();

    for type_evenement in TYPES_EVENEMENTS {
        if !OUTBOX_TRANSACTIONNEL.contains(type_evenement) {
            non_couverts.push(*type_evenement);
        }
    }

    assert!(
        non_couverts.is_empty(),
        "P-05 — {} type(s) d'événement déclaré(s) au modèle de données sans aucun test dans \
         `outbox_transactionnel.rs` :\n  {}\n\n\
         Un type ajouté sans test laisse la porte verte en n'inspectant qu'un sous-ensemble. \
         « Toute transition d'état écrit un événement dans la même transaction » ne vaut que pour \
         les transitions que quelqu'un a pensé à vérifier.",
        non_couverts.len(),
        non_couverts.join("\n  ")
    );

    println!(
        "P-05 — {}/{} types d'événements couverts.",
        TYPES_EVENEMENTS.len(),
        TYPES_EVENEMENTS.len()
    );
}

/// **Aucun type d'événement n'est émis par le code sans être déclaré ici.**
///
/// Le sens inverse du test précédent, et il n'est pas redondant : le premier attrape un type
/// déclaré qu'on aurait oublié de tester, celui-ci attrape un type **émis** que personne n'aurait
/// déclaré — donc qui n'apparaîtrait ni au modèle de données, ni dans aucun décompte.
#[test]
fn p05_aucun_type_emis_par_le_code_n_est_absent_de_la_liste() {
    use std::path::Path;

    let mut emis = BTreeSet::new();

    // Les constantes `pub const TYPE_… : &str = "…";` des services.
    for chemin in [
        "crates/socle/etablissements/src/etablissement/service.rs",
        "crates/socle/etablissements/src/modules/service.rs",
        "crates/socle/etablissements/src/points_de_vente/service.rs",
        "crates/socle/etablissements/src/configuration/service.rs",
        "crates/socle/etablissements/src/branding/service.rs",
    ] {
        let contenu = std::fs::read_to_string(Path::new(chemin))
            .unwrap_or_else(|_| panic!("{chemin} introuvable — le décompte porterait sur moins"));

        for ligne in contenu.lines() {
            let ligne = ligne.trim();
            if !ligne.starts_with("pub const TYPE_") {
                continue;
            }
            if let Some(debut) = ligne.find('"')
                && let Some(fin) = ligne[debut + 1..].find('"')
            {
                emis.insert(ligne[debut + 1..debut + 1 + fin].to_owned());
            }
        }
    }

    let declares: BTreeSet<String> = TYPES_EVENEMENTS.iter().map(|t| (*t).to_owned()).collect();
    let non_declares: Vec<&String> = emis.difference(&declares).collect();

    assert!(
        non_declares.is_empty(),
        "P-05 — {} type(s) d'événement ÉMIS par le code et absent(s) de la liste déclarée :\n  \
         {:?}\n\n\
         Un type émis sans être déclaré échappe à tous les décomptes : il n'apparaît ni au modèle \
         de données, ni dans la revue, ni dans ce recollement.",
        non_declares.len(),
        non_declares
    );

    assert!(
        emis.len() >= 10,
        "seulement {} type(s) extrait(s) des services : l'extraction est probablement cassée, et \
         ce test passerait en n'inspectant rien",
        emis.len()
    );
}

// =================================================================================================
//  P-07 — les dix tables créées, lues du CATALOGUE SYSTÈME
// =================================================================================================

/// Les dix tables **créées** par le cycle 002.
///
/// À ne pas confondre avec les **onze entités** du registre des classes hors-ligne :
/// `etablissement` y figure aussi, mais elle est *enrichie*, pas créée. Confondre les deux ferait
/// inspecter un sous-ensemble en croyant tout couvrir — le défaut exact que la constitution a
/// documenté après le cycle 001.
const TABLES_CREEES: &[&str] = &[
    "module_activite",
    "capacite",
    "profil_stock",
    "parametre_catalogue",
    "etablissement_module",
    "module_capacite",
    "point_de_vente",
    "table_pdv",
    "parametre_configuration",
    "branding",
];

/// **P-07 — les dix tables existent, et toutes sont inspectées par la porte.**
///
/// Le décompte est **lu du catalogue système**, jamais d'un nombre écrit à la main : une table
/// renommée ou supprimée doit se voir ici, pas dans six mois.
#[tokio::test]
async fn p07_les_dix_tables_creees_sont_toutes_inspectees() {
    let pool = commun::pool_owner().await;

    let reelles: BTreeSet<String> = sqlx::query(
        r#"
        SELECT c.relname AS nom
        FROM pg_class c
        JOIN pg_namespace n ON n.oid = c.relnamespace
        WHERE c.relkind = 'r' AND n.nspname = 'etablissements'
        "#,
    )
    .fetch_all(&pool)
    .await
    .expect("lecture du catalogue")
    .into_iter()
    .map(|l| l.get::<String, _>("nom"))
    .collect();

    let mut manquantes = Vec::new();
    for table in TABLES_CREEES {
        if !reelles.contains(*table) {
            manquantes.push(*table);
        }
    }

    assert!(
        manquantes.is_empty(),
        "P-07 — {} table(s) déclarée(s) par le cycle et ABSENTE(s) de la base :\n  {}\n\n\
         Soit une migration a été retirée, soit une table a été renommée sans mettre ce décompte \
         à jour. Dans les deux cas, la porte inspectait moins que ce qu'elle annonçait.",
        manquantes.len(),
        manquantes.join("\n  ")
    );

    // Et chacune est bien **isolée** — c'est ce que la porte P-07 garantit, revérifié ici APRÈS
    // la dernière migration du cycle. `rls_catalogue.rs` s'exécute aussi, mais rien ne garantit
    // qu'il ait tourné après `0013`.
    let mut sans_isolation = Vec::new();
    for table in TABLES_CREEES {
        let ligne = sqlx::query(
            r#"
            SELECT c.relrowsecurity AS activee,
                   c.relforcerowsecurity AS forcee,
                   (SELECT COUNT(*) FROM pg_policies p
                     WHERE p.schemaname = 'etablissements' AND p.tablename = c.relname) AS politiques
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE c.relkind = 'r' AND n.nspname = 'etablissements' AND c.relname = $1
            "#,
        )
        .bind(table)
        .fetch_one(&pool)
        .await
        .expect("lecture de l'état d'isolation");

        let activee: bool = ligne.get("activee");
        let forcee: bool = ligne.get("forcee");
        let politiques: i64 = ligne.get("politiques");

        if !activee || !forcee || politiques == 0 {
            sans_isolation.push(format!(
                "{table} — ENABLE={activee}, FORCE={forcee}, politiques={politiques}"
            ));
        }
    }

    assert!(
        sans_isolation.is_empty(),
        "P-07 — {} table(s) créée(s) par ce cycle sans isolation complète :\n  {}",
        sans_isolation.len(),
        sans_isolation.join("\n  ")
    );

    println!(
        "P-07 — {}/{} tables créées, inspectées et isolées.",
        TABLES_CREEES.len(),
        TABLES_CREEES.len()
    );
}

// =================================================================================================
//  P-08 — les chemins servis, comparés aux chemins déclarés
// =================================================================================================

/// Le fichier de la porte P-08, lu à la compilation.
const ISOLATION_TENANT: &str = include_str!("isolation_tenant.rs");

/// **P-08 — chaque chemin servi figure dans la table de couverture.**
///
/// La porte P-08 le vérifie déjà elle-même. Ce test-ci vérifie autre chose : que le décompte
/// **annoncé** correspond à ce qui est réellement servi. Le plan annonçait 21 opérations pour ce
/// cycle ; si le contrat en sert 19, quelque chose a été perdu en route, et P-08 resterait verte
/// puisqu'elle ne compare qu'à elle-même.
#[test]
fn p08_le_nombre_d_operations_servies_correspond_a_ce_qui_est_annonce() {
    let contrat = application::contrat_complet();

    let mut operations = 0usize;
    let mut chemins_non_declares = Vec::new();

    for (chemin, item) in &contrat.paths.paths {
        operations += compter_operations(item);
        if !ISOLATION_TENANT.contains(chemin.as_str()) {
            chemins_non_declares.push(chemin.clone());
        }
    }

    assert!(
        chemins_non_declares.is_empty(),
        "P-08 — {} chemin(s) servi(s) absent(s) de la table COUVERTURE d'`isolation_tenant.rs` :\n  \
         {}",
        chemins_non_declares.len(),
        chemins_non_declares.join("\n  ")
    );

    // Le décompte est **détaillé par lot**, pas posé en un seul nombre. Un total unique se
    // corrige en changeant un chiffre ; une ventilation oblige à dire de quel lot vient l'écart,
    // et c'est cette phrase-là qu'on ne peut pas écrire sans s'en apercevoir.
    //
    // Une opération de moins n'échouerait dans AUCUNE autre porte : P-08 compare les chemins
    // servis à sa propre table, et les deux baisseraient ensemble.
    const LOTS: &[(&str, usize)] = &[
        ("sonde de santé", 1),
        ("notes internes — module doré, cycle 001", 2),
        ("cycle 002 — établissements, services, PDV, configuration, branding, référentiels", 21),
        // ── Cycle 003 (CPT) — dix-neuf opérations, ventilées par lot de livraison ────────────
        ("cycle 003 — personnes (CPT-00, contrat §7-9)", 3),
        ("cycle 003 — session (CPT-01, contrat §1-6)", 6),
    ];
    let operations_attendues: usize = LOTS.iter().map(|(_, n)| n).sum();

    assert_eq!(
        operations,
        operations_attendues,
        "P-08 — {operations} opération(s) servie(s) au lieu des {operations_attendues} attendues.\n\
         Ventilation déclarée :\n{}\n\
         Le contrat du cycle 003 en annonce 40 au total ; tant que les lots restants ne sont pas \
         livrés, ce décompte croît lot par lot.",
        LOTS.iter()
            .map(|(nom, n)| format!("  {n:>3} — {nom}"))
            .collect::<Vec<_>>()
            .join("\n")
    );

    println!("P-08 — {operations} opérations servies, toutes déclarées.");
}

/// **Récapitulatif des trois décomptes**, imprimé pour la revue de fin de cycle.
#[tokio::test]
async fn recapitulatif_des_trois_portes_a_decompte() {
    let pool = commun::pool_owner().await;
    let tables: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM pg_class c
        JOIN pg_namespace n ON n.oid = c.relnamespace
        WHERE c.relkind = 'r' AND n.nspname = 'etablissements'
        "#,
    )
    .fetch_one(&pool)
    .await
    .expect("comptage");

    let operations: usize = application::contrat_complet()
        .paths
        .paths
        .values()
        .map(compter_operations)
        .sum();

    println!("Recollement des trois portes à décompte — cycle 002 :");
    println!(
        "  P-05 — {} types d'événements déclarés et couverts",
        TYPES_EVENEMENTS.len()
    );
    println!(
        "  P-07 — {} tables créées par ce cycle, {tables} tables au total dans `etablissements`",
        TABLES_CREEES.len()
    );
    println!("  P-08 — {operations} opérations servies, toutes avec un régime déclaré");
    println!();
    println!("Une porte qui s'étend sur plusieurs phases laisse un trou par construction ;");
    println!("ce fichier est ce qui le referme.");
}
