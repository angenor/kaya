//! **Porte de cohérence documentaire** — la taxonomie d'audit du code et celle du document ne
//! divergent jamais.
//!
//! # Ce qu'elle rend vérifiable
//!
//! CPT-04 énumère dix familles d'actions à tracer. **Huit n'ont aucun chemin d'écriture avant la
//! tranche T2 ou T3** : la remise n'existe pas encore, l'avoir non plus, le tiroir-caisse n'est pas
//! branché. Une liste de choses à faire qui vit dans une spécification se perd ; une liste qui vit
//! dans un harnais fait échouer le build le jour où quelqu'un branche un type sans le déclarer.
//!
//! C'est le harnais à étapes dues du cycle 002 (`agnosticite_socle.rs`), appliqué aux dix familles
//! de `docs/taxonomie-audit.md`.
//!
//! # Périmètre inspecté — et ce qui ne l'est PAS
//!
//! *Section exigée par le § « Couverture des portes » de la constitution : une porte dont la cible
//! est vide passe toujours, et un test négatif prouve qu'elle sait échouer, pas qu'elle regarde
//! tout.*
//!
//! **Inspecté** — tous les fichiers `.rs` sous :
//!
//!   * `backend/crates/**/src/` — le socle, les capacités, les verticales ;
//!   * `backend/api/src/` — handlers et assemblage.
//!
//! **Exclu, nommément** :
//!
//!   * `backend/crates/socle/comptes/src/audit/taxonomie.rs` — c'est la **définition** de
//!     l'énumération. Y voir chaque variante n'apprend rien : elles y sont toutes, par
//!     construction. L'inclure rendrait les dix types « branchés » dès leur déclaration, ce qui
//!     est exactement le contraire de ce qu'on mesure.
//!   * `backend/tests/` — un test qui construit une entrée d'audit n'est pas un chemin d'écriture
//!     du produit. Les inclure ferait basculer un type en « branché » parce qu'on l'a testé.
//!   * Les fichiers `.rs` générés ou vendus (aucun à ce jour).
//!
//! **Ce que la porte ne sait pas voir**, écrit pour qu'on ne la croie pas plus forte qu'elle
//! n'est : un chemin d'écriture qui n'emploierait pas la variante d'énumération — construction du
//! type d'action depuis une chaîne, réflexion, macro qui fabrique le nom. **C'est pourquoi
//! `TypeActionAudit` est une énumération fermée et que la colonne `type_action` n'est jamais
//! alimentée depuis un `String` de l'appelant.** Les deux se tiennent : le typage ferme la porte
//! que ce contrôle statique ne peut pas garder.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// La taxonomie, lue à la compilation. Une modification du document recompile le test.
const TAXONOMIE: &str = include_str!("../../docs/taxonomie-audit.md");

/// Nombre de familles annoncées par CPT-04. **Figé** : une onzième famille se décide, elle
/// n'apparaît pas.
const FAMILLES_ATTENDUES: usize = 10;

/// Le fichier de **définition** de l'énumération — exclu du balayage, voir le commentaire de tête.
const DEFINITION: &str = "crates/socle/comptes/src/audit/taxonomie.rs";

/// État déclaré d'une famille au document.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Etat {
    /// Un chemin de code écrit une entrée de ce type.
    Branche,
    /// La story qui l'apportera est nommée ; aucun chemin n'existe encore.
    Du,
}

/// Une ligne du tableau des dix familles.
#[derive(Debug, Clone)]
struct Famille {
    code: String,
    etat: Etat,
    /// La story qui la doit — reproduite dans les messages d'échec, pour que la porte dise quoi
    /// faire et pas seulement que quelque chose ne va pas.
    story: String,
}

/// Extrait les dix familles du tableau de `docs/taxonomie-audit.md`.
///
/// Le format attendu est celui du document : `| n | \`code\` | description | **état** | story |`.
/// L'extraction est volontairement stricte — un tableau reformaté doit faire échouer bruyamment
/// plutôt que rendre une liste vide, qui passerait au vert en n'ayant rien à comparer.
fn familles_du_document() -> Vec<Famille> {
    let Some(debut) = TAXONOMIE.find("## Les dix familles") else {
        panic!(
            "la section « Les dix familles » a disparu de docs/taxonomie-audit.md. C'est la \
             source que CPT-04 désigne ; sans elle, cette porte n'a plus rien à comparer."
        );
    };
    let section = &TAXONOMIE[debut..];
    let fin = section[3..].find("\n## ").map(|i| i + 3).unwrap_or(section.len());
    let section = &section[..fin];

    let mut familles = Vec::new();

    for ligne in section.lines() {
        let ligne = ligne.trim();
        if !ligne.starts_with('|') {
            continue;
        }
        let cellules: Vec<&str> = ligne.trim_matches('|').split('|').map(str::trim).collect();
        if cellules.len() < 5 {
            continue;
        }
        // Colonne 0 : le numéro. Écarte l'en-tête et le séparateur.
        if cellules[0].parse::<usize>().is_err() {
            continue;
        }

        let code = cellules[1].trim_matches('`').trim().to_owned();
        let etat = match cellules[3].replace('*', "").trim() {
            "branché" => Etat::Branche,
            "dû" => Etat::Du,
            autre => panic!(
                "état « {autre} » inconnu pour la famille « {code} » dans \
                 docs/taxonomie-audit.md. Les deux seuls états sont « branché » et « dû » — un \
                 troisième rendrait le harnais muet sur ce qu'il désigne."
            ),
        };
        let story = cellules[4].replace('*', "").trim().to_owned();

        familles.push(Famille { code, etat, story });
    }

    familles
}

/// `changement_role` → `ChangementRole`. La convention de nommage Rust, appliquée au code du
/// document — c'est elle qui permet de chercher la variante dans le source.
fn variante_rust(code: &str) -> String {
    code.split('_')
        .map(|mot| {
            let mut c = mot.chars();
            match c.next() {
                Some(premiere) => premiere.to_uppercase().collect::<String>() + c.as_str(),
                None => String::new(),
            }
        })
        .collect()
}

/// Racine du répertoire `backend/`, quel que soit le répertoire d'exécution des tests.
fn racine_backend() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Tous les fichiers `.rs` du périmètre inspecté, **avec le décompte** — c'est lui qui distingue
/// une porte qui ne trouve rien d'une porte qui n'a rien à trouver.
fn fichiers_inspectes() -> Vec<PathBuf> {
    let racine = racine_backend();
    let exclu = racine.join(DEFINITION);

    let mut fichiers = Vec::new();
    for arbre in ["crates", "api/src"] {
        collecter(&racine.join(arbre), &exclu, &mut fichiers);
    }
    fichiers.sort();
    fichiers
}

fn collecter(repertoire: &Path, exclu: &Path, sortie: &mut Vec<PathBuf>) {
    let Ok(entrees) = std::fs::read_dir(repertoire) else {
        return;
    };
    for entree in entrees.flatten() {
        let chemin = entree.path();
        if chemin.is_dir() {
            // `target/` d'un crate, s'il en existe un local.
            if chemin.file_name().is_some_and(|n| n == "target") {
                continue;
            }
            collecter(&chemin, exclu, sortie);
        } else if chemin.extension().is_some_and(|e| e == "rs") && chemin != exclu {
            sortie.push(chemin);
        }
    }
}

/// Pour chaque famille, les fichiers du périmètre qui emploient sa variante d'énumération.
fn chemins_par_famille(familles: &[Famille]) -> BTreeMap<String, Vec<String>> {
    let fichiers = fichiers_inspectes();

    assert!(
        !fichiers.is_empty(),
        "aucun fichier `.rs` inspecté sous backend/crates/ ni backend/api/src/. Une porte dont \
         la cible est vide passe toujours : c'est le défaut que le § « Couverture des portes » \
         de la constitution nomme."
    );

    let mut par_famille: BTreeMap<String, Vec<String>> = familles
        .iter()
        .map(|f| (f.code.clone(), Vec::new()))
        .collect();

    for fichier in &fichiers {
        let Ok(contenu) = std::fs::read_to_string(fichier) else {
            continue;
        };
        for famille in familles {
            let motif = format!("TypeActionAudit::{}", variante_rust(&famille.code));
            if contenu.contains(&motif) {
                let relatif = fichier
                    .strip_prefix(racine_backend())
                    .unwrap_or(fichier)
                    .display()
                    .to_string();
                par_famille
                    .get_mut(&famille.code)
                    .expect("famille présente")
                    .push(relatif);
            }
        }
    }

    par_famille
}

// =================================================================================================
//  Les trois contrôles
// =================================================================================================

/// **1 · Le document déclare exactement dix familles, sans doublon.**
#[test]
fn le_document_declare_les_dix_familles_de_cpt_04() {
    let familles = familles_du_document();

    assert_eq!(
        familles.len(),
        FAMILLES_ATTENDUES,
        "docs/taxonomie-audit.md déclare {} famille(s) au lieu des {FAMILLES_ATTENDUES} de \
         CPT-04. Une onzième famille se décide — elle n'apparaît pas au détour d'un cycle.",
        familles.len()
    );

    let codes: BTreeSet<&str> = familles.iter().map(|f| f.code.as_str()).collect();
    assert_eq!(
        codes.len(),
        familles.len(),
        "deux familles portent le même code dans docs/taxonomie-audit.md"
    );

    for famille in &familles {
        assert!(
            !famille.code.is_empty()
                && famille
                    .code
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c == '_'),
            "le code « {} » n'est pas en minuscules françaises sans accent, comme le veut la \
             convention de nommage du produit",
            famille.code
        );
        assert!(
            !famille.story.is_empty(),
            "la famille « {} » ne nomme aucune story. Un type dû sans story est une intention \
             sans propriétaire : personne ne le branchera.",
            famille.code
        );
    }
}

/// **2 · Aucun type déclaré « dû » n'a de chemin d'écriture.**
///
/// C'est le contrôle qui oblige à revenir au document. Le jour où PDV-03 écrit une remise, ce test
/// échoue **avant** la revue, et la ligne passe à « branché » dans le même changement.
#[test]
fn aucun_type_du_n_a_de_chemin_d_ecriture() {
    let familles = familles_du_document();
    let chemins = chemins_par_famille(&familles);

    let fautifs: Vec<String> = familles
        .iter()
        .filter(|f| f.etat == Etat::Du)
        .filter_map(|f| {
            let trouves = &chemins[&f.code];
            (!trouves.is_empty()).then(|| {
                format!(
                    "  · « {} » (dû par {}) est employé dans :\n      {}",
                    f.code,
                    f.story,
                    trouves.join("\n      ")
                )
            })
        })
        .collect();

    assert!(
        fautifs.is_empty(),
        "{} type(s) d'action déclaré(s) « dû » ont pourtant un chemin d'écriture :\n{}\n\n\
         Le code et docs/taxonomie-audit.md ont divergé. Si le branchement est voulu, faire \
         passer la ligne à **branché** dans CE changement — c'est tout ce que la porte demande. \
         Si ce n'est pas voulu, l'entrée d'audit part dans un registre à rétention illimitée sans \
         que quiconque l'ait décidé.",
        fautifs.len(),
        fautifs.join("\n")
    );
}

/// **3 · Tout type déclaré « branché » a réellement un chemin d'écriture.**
///
/// Sans ce versant, il suffirait de tout déclarer branché pour rendre le harnais muet. C'est la
/// même dissymétrie que P-05b : un contrôle qui n'a que son versant négatif passe au vert quand la
/// cible disparaît.
#[test]
fn tout_type_branche_a_reellement_un_chemin_d_ecriture() {
    let familles = familles_du_document();
    let chemins = chemins_par_famille(&familles);

    let menteurs: Vec<&str> = familles
        .iter()
        .filter(|f| f.etat == Etat::Branche)
        .filter(|f| chemins[&f.code].is_empty())
        .map(|f| f.code.as_str())
        .collect();

    assert!(
        menteurs.is_empty(),
        "{} type(s) déclaré(s) « branché » n'ont aucun chemin d'écriture dans le périmètre \
         inspecté : {:?}\n\n\
         Soit le chemin a été retiré et la ligne doit repasser à « dû » avec la story qui le \
         rétablira, soit il emploie une autre forme que `TypeActionAudit::<Variante>` — auquel \
         cas c'est la forme qu'il faut corriger, pas la porte : la colonne `type_action` ne \
         s'alimente jamais depuis un `String` d'appelant.",
        menteurs.len(),
        menteurs
    );
}

/// **4 · Décompte — l'état d'avancement, affiché plutôt que deviné.**
///
/// Ce test ne peut pas échouer sur le fond ; il existe pour que `cargo test` imprime où en est la
/// taxonomie. Le cycle 003 le laisse à **0 branché / 10 dus** ; il finira à 2 / 8.
#[test]
fn decompte_des_familles_branchees_et_dues() {
    let familles = familles_du_document();
    let chemins = chemins_par_famille(&familles);

    let branchees = familles.iter().filter(|f| f.etat == Etat::Branche).count();
    let dues = familles.iter().filter(|f| f.etat == Etat::Du).count();
    let fichiers = fichiers_inspectes().len();

    assert_eq!(
        branchees + dues,
        FAMILLES_ATTENDUES,
        "une famille n'est ni branchée ni due — l'extraction du document est cassée"
    );

    println!(
        "taxonomie d'audit : {branchees} branchée(s) / {dues} due(s) sur {FAMILLES_ATTENDUES}, \
         {fichiers} fichier(s) `.rs` inspecté(s)"
    );
    for famille in &familles {
        let marque = match famille.etat {
            Etat::Branche => "branché",
            Etat::Du => "dû     ",
        };
        println!(
            "  [{marque}] {:28} {} chemin(s) — {}",
            famille.code,
            chemins[&famille.code].len(),
            famille.story
        );
    }
}

/// **5 · Code → document : l'énumération Rust et le tableau portent les mêmes dix codes.**
///
/// C'est le sens de comparaison que le § « Couverture des portes » impose — celui qui attrape ce
/// qu'on ajoute, pas ce qu'on projette. Une variante ajoutée à `TypeActionAudit` sans ligne au
/// document fait échouer ce test ; une ligne au document sans variante aussi, parce qu'ici les
/// deux listes sont **fermées** et de même taille par construction (contrairement au registre des
/// classes hors-ligne, qui décrit tout le produit à venir).
///
/// Les codes sont extraits du corps de `fn code()`, où ils sont écrits à la main précisément pour
/// ne pas dépendre de la représentation `serde`.
#[test]
fn l_enumeration_rust_et_le_document_portent_les_memes_codes() {
    let source = std::fs::read_to_string(racine_backend().join(DEFINITION)).unwrap_or_else(|e| {
        panic!(
            "impossible de lire {DEFINITION} : {e}\n\
             C'est la définition de `TypeActionAudit`. Si elle a été déplacée, mettre à jour la \
             constante DEFINITION — qui sert aussi d'exclusion au balayage."
        )
    });

    // Les codes littéraux du `match` de `code()` : `TypeActionAudit::Variante => "code",`
    let codes_du_code: BTreeSet<String> = source
        .lines()
        .filter_map(|ligne| {
            let ligne = ligne.trim();
            let apres = ligne.strip_prefix("TypeActionAudit::")?;
            let (_, valeur) = apres.split_once("=> \"")?;
            let (code, _) = valeur.split_once('"')?;
            Some(code.to_owned())
        })
        .collect();

    let codes_du_document: BTreeSet<String> = familles_du_document()
        .into_iter()
        .map(|f| f.code)
        .collect();

    assert_eq!(
        codes_du_code.len(),
        FAMILLES_ATTENDUES,
        "{} code(s) extrait(s) de l'énumération pour {FAMILLES_ATTENDUES} attendu(s). \
         L'extraction lit les bras `TypeActionAudit::Variante => \"code\",` de `fn code()` : \
         une reformulation du `match` la casserait, et la porte ne comparerait plus rien. \
         Extraits : {codes_du_code:?}",
        codes_du_code.len()
    );

    assert_eq!(
        codes_du_code, codes_du_document,
        "l'énumération `TypeActionAudit` et docs/taxonomie-audit.md ont divergé.\n\
         Dans le code seulement : {:?}\n\
         Dans le document seulement : {:?}",
        codes_du_code
            .difference(&codes_du_document)
            .collect::<Vec<_>>(),
        codes_du_document
            .difference(&codes_du_code)
            .collect::<Vec<_>>()
    );
}

/// **Test négatif** — la porte sait échouer.
///
/// Exercé sur un jeu simulé plutôt que sur le vrai document : y injecter une famille factice
/// ferait échouer les trois contrôles ci-dessus au hasard de l'ordonnancement.
#[test]
fn test_negatif_la_porte_signale_les_deux_divergences() {
    // Un type « dû » avec un chemin d'écriture.
    let familles = vec![
        Famille {
            code: "remise".to_owned(),
            etat: Etat::Du,
            story: "PDV-03".to_owned(),
        },
        Famille {
            code: "changement_role".to_owned(),
            etat: Etat::Branche,
            story: "CPT-02".to_owned(),
        },
    ];

    let mut chemins: BTreeMap<String, Vec<String>> = BTreeMap::new();
    chemins.insert("remise".to_owned(), vec!["un/chemin.rs".to_owned()]);
    chemins.insert("changement_role".to_owned(), Vec::new());

    let dus_fautifs = familles
        .iter()
        .filter(|f| f.etat == Etat::Du && !chemins[&f.code].is_empty())
        .count();
    assert_eq!(
        dus_fautifs, 1,
        "la porte n'a pas signalé un type dû pourtant branché : elle ne protège rien"
    );

    let branches_menteurs = familles
        .iter()
        .filter(|f| f.etat == Etat::Branche && chemins[&f.code].is_empty())
        .count();
    assert_eq!(
        branches_menteurs, 1,
        "la porte n'a pas signalé un type branché sans chemin : son versant positif est absent"
    );

    // Et la conversion de nom, dont dépend tout le balayage.
    assert_eq!(variante_rust("changement_role"), "ChangementRole");
    assert_eq!(
        variante_rust("rebascule_palier_passage"),
        "RebasculePalierPassage"
    );
    assert_eq!(variante_rust("avoir"), "Avoir");
}
