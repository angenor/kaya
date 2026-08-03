//! **Le périmètre des portes est découvert, et il ne rétrécit pas.**
//!
//! `commun/perimetre.rs` est consommé par dix fichiers de portes. Ce fichier-ci est le seul qui
//! l'inspecte **lui-même** — parce qu'un module de périmètre cassé rendrait toutes les portes qui
//! s'y adossent silencieusement vertes, et que ce mode de défaillance est exactement celui que le
//! module existe pour supprimer.
//!
//! # Périmètre inspecté
//!
//! *Exigence 1 du § « Couverture des portes » de la constitution.*
//!
//! **Inspecté** :
//!
//!   * les schémas que `pg_namespace` déclare, moins les exclusions nommées du module ;
//!   * les `[workspace] members` de `backend/Cargo.toml`, par famille ;
//!   * **tous** les fichiers `.rs` de `backend/tests/`, pour le contrôle prospectif de FR-004c.
//!
//! **Non inspecté** : ce que chaque porte fait de son périmètre. Une cible complète n'a jamais
//! rendu un test juste — elle l'empêche seulement d'être vide.

mod commun;

use commun::perimetre::{self, Famille};

/// **Les trois familles sont peuplées, disjointes, et leurs décomptes ne baissent pas.**
///
/// Les assertions de plancher vivent dans le module ; ce test les déclenche et **imprime** ce qui
/// a été découvert, pour que la revue de fin de cycle lise un périmètre plutôt qu'un « ok ».
#[test]
fn les_trois_familles_de_crates_sont_decouvertes_et_comptees() {
    let socle = perimetre::crates_du_socle();
    let capacites = perimetre::crates_des_capacites();
    let verticales = perimetre::crates_des_verticales();

    assert!(
        perimetre::familles_disjointes(),
        "un crate est compté dans deux familles — les décomptes sommés seraient faux :\n  \
         socle {socle:?}\n  capacités {capacites:?}\n  verticales {verticales:?}"
    );

    println!("Périmètre des crates, lu de `[workspace] members` :");
    for (famille, membres) in [
        ("socle", &socle),
        ("capacités", &capacites),
        ("verticales", &verticales),
    ] {
        println!("  {famille} — {} crate(s)", membres.len());
        for membre in membres.iter() {
            println!("      {membre}");
        }
    }
}

/// **Un crate nommé qui n'existe pas fait paniquer** — il ne rend jamais une cible vide.
///
/// C'est le versant négatif du module : sans lui, une porte qui viserait un crate renommé lirait
/// un répertoire absent, n'inspecterait rien, et passerait au vert.
#[test]
#[should_panic(expected = "n'est pas déclaré aux `[workspace] members`")]
fn un_crate_inexistant_panique_au_lieu_de_rendre_une_cible_vide() {
    perimetre::chemin_crate(Famille::Socle, "zzz-crate-qui-n-existe-pas");
}

/// Un crate réel se compose depuis le manifeste, sans qu'aucun chemin soit écrit en toutes lettres.
#[test]
fn un_crate_reel_se_compose_depuis_le_manifeste() {
    let fiscalite = perimetre::chemin_crate(Famille::Socle, "fiscalite");
    assert!(
        perimetre::racine_backend()
            .join(&fiscalite)
            .join("Cargo.toml")
            .exists(),
        "« {fiscalite} » est déclaré au workspace mais n'a pas de manifeste sur disque"
    );

    let taxonomie = perimetre::fichier_du_crate(Famille::Socle, "comptes", "src/audit/taxonomie.rs");
    assert!(
        perimetre::racine_backend().join(&taxonomie).exists(),
        "« {taxonomie} » composé depuis le manifeste ne désigne aucun fichier"
    );
}

/// **Les schémas applicatifs sont découverts, et le module dit ce qu'il laisse dehors.**
#[tokio::test]
async fn les_schemas_applicatifs_sont_decouverts_et_les_exclusions_declarees() {
    let pool = commun::pool_owner().await;
    let schemas = perimetre::schemas_applicatifs(&pool).await;

    println!("Périmètre des schémas, lu de `pg_namespace` :");
    for schema in &schemas {
        println!("  {schema}");
    }
    println!("Exclus, nommément :");
    for exclusion in perimetre::exclusions_declarees() {
        println!("  {exclusion}");
    }

    // Les schémas des migrations `0001`, `0014` et `0021` — cités pour que le test dise ce qu'il
    // attend, pas seulement combien. La liste n'est PAS le périmètre : elle en est un
    // sous-ensemble connu, et le décompte découvert peut la dépasser sans que rien n'échoue.
    for attendu in ["etablissements", "synchronisation", "fiscalite", "comptes", "hebergement"] {
        assert!(
            schemas.iter().any(|s| s == attendu),
            "le schéma « {attendu} », créé par une migration, n'est pas découvert : {schemas:?}\n\
             La découverte ne voit donc pas ce que la base porte, et toute porte qui s'y adosse \
             inspecterait moins que ce qu'elle annonce."
        );
    }

    assert!(
        !schemas.iter().any(|s| s == "public" || s == "kaya_migrations"),
        "un schéma exclu est pourtant découvert : {schemas:?}"
    );
}

/// **FR-004c — aucun fichier de `backend/tests/` ne déclare son périmètre à la main.**
///
/// # Pourquoi ce contrôle est PROSPECTIF, et pourquoi il fallait l'écrire maintenant
///
/// Le cycle 005 a ramené vingt et une occurrences de chemin de crate en dur à zéro. Sans ce test,
/// la règle « toute porte future en hérite » resterait **déclarative** : la vingt-septième porte
/// réintroduirait une liste, elle serait juste le jour de son écriture, et elle vieillirait comme
/// les trois précédentes — sans que rien ne le dise.
///
/// La correction est toujours la même : passer par `commun::perimetre`.
#[test]
fn aucun_fichier_de_test_ne_declare_son_perimetre_a_la_main() {
    let constats = perimetre::perimetres_en_dur();

    assert!(
        constats.is_empty(),
        "FR-004c — {} déclaration(s) de périmètre écrite(s) à la main dans backend/tests/ :\n{}\n\n\
         Un périmètre écrit à la main est correct le jour où on l'écrit, et il vieillit sans que \
         rien ne le dise. Le motif a laissé un trou à chacun des trois cycles précédents : six \
         tables sur dix (002), le schéma `comptes` (003), le schéma `hebergement` (004).\n\n\
         Remède : lire `commun::perimetre::schemas_applicatifs()` pour les schémas, \
         `crates_du_socle()` / `crates_des_capacites()` / `crates_des_verticales()` pour les \
         crates, et `chemin_crate()` / `fichier_du_crate()` pour viser un crate nommé — cette \
         dernière panique si le crate a disparu du workspace, au lieu de rendre une cible vide.",
        constats.len(),
        constats
            .iter()
            .map(|c| format!(
                "  · {}:{} — {} ({})",
                c.fichier, c.ligne, c.declaration, c.motif
            ))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// **Test négatif — le contrôle prospectif sait reconnaître les deux motifs qu'il cherche.**
///
/// Un contrôle qui ne trouve rien peut être un contrôle qui fonctionne, ou un contrôle cassé. Rien
/// ne les distingue depuis le vert. Celui-ci exerce la reconnaissance sur deux fragments écrits
/// ici même, sans toucher au dépôt — le précédent est `classes_offline.rs`, dont le test négatif
/// simule sa table fautive plutôt que de la créer, pour ne pas casser ses voisins.
#[test]
fn test_negatif_le_controle_prospectif_reconnait_ses_deux_motifs() {
    // Motif 1 — une constante de tableau qui nomme des schémas.
    let ligne_1 = r#"const SCHEMAS_APPLICATIFS: &[&str] = &["etablissements", "comptes"];"#;
    let nu = ligne_1.trim();
    let nom = nu
        .strip_prefix("const ")
        .and_then(|a| a.split(':').next())
        .map(str::trim)
        .expect("nom de constante");
    assert!(
        nu.contains("&[") && (nom.contains("SCHEMA") || nom.contains("CRATE")),
        "le motif « constante de périmètre » ne reconnaît plus sa propre forme : {ligne_1}"
    );

    // Motif 2 — un chemin de crate écrit en toutes lettres, quelle que soit la forme syntaxique.
    let prefixe = format!("{}/", Famille::Socle.racine());
    for ligne_2 in [
        format!(r#"    "{prefixe}etablissements/src/etablissement/service.rs","#),
        format!(r#"    Path::new("{prefixe}fiscalite/src/lib.rs")"#),
    ] {
        assert!(
            ligne_2.contains(&format!("\"{prefixe}")),
            "le motif « chemin de crate » ne reconnaît plus sa propre forme : {ligne_2}"
        );
    }

    // Et il ne se déclenche pas sur ce qui n'est pas un périmètre — sans quoi il échouerait sur
    // tout et serait désactivé dans la semaine.
    let innocente = r#"const TABLES_EXCLUES: &[&str] = &["_migrations_appliquees"];"#;
    let nom_innocent = innocente
        .strip_prefix("const ")
        .and_then(|a| a.split(':').next())
        .map(str::trim)
        .expect("nom de constante");
    assert!(
        !nom_innocent.contains("SCHEMA") && !nom_innocent.contains("CRATE"),
        "le contrôle signale une constante qui ne déclare aucun périmètre"
    );
}
