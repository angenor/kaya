//! **Portes P-03 et P-12** — la hiérarchie des crates et le confinement de la fiscalité.
//!
//! Les deux se vérifient sur le **graphe de dépendances réel**, lu par `cargo metadata`. Une
//! lecture des `Cargo.toml` à la main manquerait les dépendances transitives : `socle/a` peut
//! dépendre de `verticales/z` sans que son manifeste le dise, s'il passe par `socle/b`.
//!
//! | Porte | Vérifie | Principe |
//! |---|---|---|
//! | **P-03** | Aucun crate de `socle/` ne dépend d'un crate de `verticales/` | II |
//! | **P-12** | Aucune règle fiscale hors du trait `JurisdictionAdapter` | V |
//!
//! # Pourquoi P-03 est la porte la plus structurante du produit
//!
//! C'est cette hiérarchie qui garde Kaya extensible à d'autres activités. Sans elle, l'hôtellerie
//! contamine le noyau en trois cycles et l'extension devient une réécriture : un maquis, un bar,
//! un pressing ou une résidence meublée sont des établissements valides, et **aucun crate partagé
//! ne doit supposer l'existence d'un hébergement ni d'un point de vente**.
//!
//! L'erreur ne se commet jamais franchement. Elle se commet en ajoutant « juste une dépendance »
//! pour réutiliser un type qui se trouve du mauvais côté.

use std::collections::{BTreeMap, BTreeSet};
use std::process::Command;

use serde_json::Value;

/// Famille d'un crate, déduite du **chemin de son manifeste**.
///
/// Le chemin, pas le nom : `kaya-hebergement` pourrait être renommé, déplacé, ou un crate pourrait
/// porter un nom trompeur. L'arborescence, elle, est celle que la constitution fixe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Famille {
    Domain,
    Socle,
    Capacites,
    Verticales,
    /// Binaires et harnais de tests — hors hiérarchie, ils assemblent tout.
    Assemblage,
}

fn famille(chemin_manifeste: &str) -> Famille {
    if chemin_manifeste.contains("/crates/socle/") {
        Famille::Socle
    } else if chemin_manifeste.contains("/crates/capacites/") {
        Famille::Capacites
    } else if chemin_manifeste.contains("/crates/verticales/") {
        Famille::Verticales
    } else if chemin_manifeste.contains("/crates/domain/") {
        Famille::Domain
    } else {
        Famille::Assemblage
    }
}

/// Cette arête est-elle autorisée par le principe II ?
///
/// | Famille | Peut dépendre de |
/// |---|---|
/// | `socle/` | `socle/` et `domain` **uniquement** |
/// | `capacites/` | `socle/`, `domain` |
/// | `verticales/` | `socle/`, `capacites/`, `domain` |
fn arete_autorisee(de: Famille, vers: Famille) -> bool {
    use Famille::*;
    match (de, vers) {
        (Domain, Domain) => true,
        (Domain, _) => false,
        (Socle, Socle | Domain) => true,
        (Socle, _) => false,
        (Capacites, Socle | Domain | Capacites) => true,
        (Capacites, _) => false,
        (Verticales, _) => true,
        (Assemblage, _) => true,
    }
}

struct Graphe {
    familles: BTreeMap<String, Famille>,
    /// Dépendances **directes** de chaque paquet interne.
    aretes: BTreeMap<String, BTreeSet<String>>,
}

fn lire_graphe() -> Graphe {
    let sortie = Command::new(env!("CARGO"))
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .output()
        .expect("cargo metadata");
    assert!(
        sortie.status.success(),
        "cargo metadata a échoué : {}",
        String::from_utf8_lossy(&sortie.stderr)
    );

    let metadata: Value = serde_json::from_slice(&sortie.stdout).expect("metadata illisible");
    let paquets = metadata["packages"].as_array().expect("packages");

    let mut familles = BTreeMap::new();
    let mut aretes: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let noms_internes: BTreeSet<String> = paquets
        .iter()
        .map(|p| p["name"].as_str().unwrap().to_owned())
        .collect();

    for paquet in paquets {
        let nom = paquet["name"].as_str().unwrap().to_owned();
        let manifeste = paquet["manifest_path"].as_str().unwrap();
        familles.insert(nom.clone(), famille(manifeste));

        let mut dependances = BTreeSet::new();
        for dependance in paquet["dependencies"].as_array().unwrap() {
            let cible = dependance["name"].as_str().unwrap();
            // Les dépendances de développement assemblent tout — un test d'intégration a le droit
            // de voir l'ensemble du produit. La hiérarchie contraint le code de production.
            let genre = dependance["kind"].as_str().unwrap_or("normal");
            if noms_internes.contains(cible) && genre == "normal" {
                dependances.insert(cible.to_owned());
            }
        }
        aretes.insert(nom, dependances);
    }

    Graphe { familles, aretes }
}

/// Fermeture transitive des dépendances d'un paquet.
fn dependances_transitives(graphe: &Graphe, depart: &str) -> BTreeSet<String> {
    let mut vues = BTreeSet::new();
    let mut a_visiter = vec![depart.to_owned()];

    while let Some(courant) = a_visiter.pop() {
        for voisin in graphe.aretes.get(&courant).into_iter().flatten() {
            if vues.insert(voisin.clone()) {
                a_visiter.push(voisin.clone());
            }
        }
    }
    vues
}

#[test]
fn p03_la_hierarchie_des_crates_est_respectee() {
    let graphe = lire_graphe();

    assert!(
        graphe.familles.len() >= 15,
        "seulement {} paquets vus : cargo metadata n'a pas lu tout le workspace",
        graphe.familles.len()
    );

    let mut violations = Vec::new();

    for (paquet, famille_source) in &graphe.familles {
        // La fermeture transitive, pas les seules dépendances directes : `socle/a` peut atteindre
        // `verticales/z` en passant par un tiers, et son manifeste ne le dirait pas.
        for atteint in dependances_transitives(&graphe, paquet) {
            let famille_cible = graphe.familles[&atteint];
            if !arete_autorisee(*famille_source, famille_cible) {
                violations.push(format!(
                    "{paquet} ({famille_source:?}) atteint {atteint} ({famille_cible:?})"
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "P-03 ÉCHOUE — {} arête(s) interdite(s) dans le graphe de dépendances :\n  {}\n\n\
         C'est cette hiérarchie qui garde le produit extensible à d'autres activités. Sans elle, \
         l'hôtellerie contamine le noyau en trois cycles et l'extension devient une réécriture.\n\
         Un crate de socle/ qui a besoin d'un type d'une verticale a besoin d'un TRAIT exposé, \
         pas d'une dépendance.",
        violations.len(),
        violations.join("\n  ")
    );
}

/// **P-12** — aucun crate hors `socle/fiscalite` ne référence les types de taxe de `domain`.
///
/// # Porte installée avec une cible réelle, et une assertion de non-régression
///
/// Aucune règle fiscale n'est écrite à ce cycle (T3). La porte pourrait donc être verte à vide et
/// ne rien prouver. Deux mesures l'évitent :
///
/// - les types de taxe **existent** dans `domain::fiscal` et le trait `JurisdictionAdapter` les
///   consomme, donc la porte a une cible ;
/// - l'assertion de non-régression ci-dessous échoue si `domain::fiscal` disparaissait — auquel
///   cas la porte cesserait silencieusement de vérifier quoi que ce soit.
#[test]
fn p12_aucune_regle_fiscale_hors_de_l_adaptateur() {
    use std::fs;
    use std::path::Path;

    // Assertion de non-régression (R-15) : la cible doit exister.
    let module_fiscal = Path::new("crates/domain/src/fiscal.rs");
    assert!(
        module_fiscal.exists(),
        "crates/domain/src/fiscal.rs a disparu : la porte P-12 n'a plus de cible et cesserait de \
         vérifier quoi que ce soit sans que rien ne l'indique"
    );

    /// Types de `domain::fiscal` dont la référence hors de `socle/fiscalite` signale une règle
    /// fiscale égarée.
    const TYPES_FISCAUX: &[&str] = &[
        "BaseImposable",
        "VentilationTaxes",
        "LigneTaxe",
        "EmissionChannel",
        "EtatDeReversement",
        "DocumentAcertifier",
        "Certification",
    ];

    fn parcourir(racine: &Path, fichiers: &mut Vec<std::path::PathBuf>) {
        let Ok(entrees) = fs::read_dir(racine) else {
            return;
        };
        for entree in entrees.flatten() {
            let chemin = entree.path();
            if chemin.is_dir() {
                if chemin.file_name().is_some_and(|n| n == "target") {
                    continue;
                }
                parcourir(&chemin, fichiers);
            } else if chemin.extension().is_some_and(|e| e == "rs") {
                fichiers.push(chemin);
            }
        }
    }

    let mut fichiers = Vec::new();
    parcourir(Path::new("crates"), &mut fichiers);
    parcourir(Path::new("api"), &mut fichiers);
    parcourir(Path::new("node"), &mut fichiers);

    let mut violations = Vec::new();

    for fichier in fichiers {
        let chemin = fichier.to_string_lossy().replace('\\', "/");

        // Les deux emplacements légitimes : le trait lui-même, et la déclaration des types.
        if chemin.contains("crates/socle/fiscalite/") || chemin.contains("crates/domain/src/fiscal")
        {
            continue;
        }

        let Ok(contenu) = fs::read_to_string(&fichier) else {
            continue;
        };
        // Retirer les commentaires : ce fichier-ci nomme les types dans sa documentation, et un
        // crate qui les mentionne dans un commentaire n'implémente aucune règle.
        let code: String = contenu
            .lines()
            .filter(|l| {
                let t = l.trim_start();
                !t.starts_with("//") && !t.starts_with("*")
            })
            .collect::<Vec<_>>()
            .join("\n");

        for type_fiscal in TYPES_FISCAUX {
            if code.contains(type_fiscal) {
                violations.push(format!("{chemin} référence {type_fiscal}"));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "P-12 ÉCHOUE — {} référence(s) aux types fiscaux hors de socle/fiscalite :\n  {}\n\n\
         AUCUNE règle fiscale ne vit hors du trait JurisdictionAdapter (principe V). TVA, taxe de \
         nuitée et taxe de développement touristique sont des SORTIES de l'adaptateur, jamais des \
         constantes.",
        violations.len(),
        violations.join("\n  ")
    );
}
