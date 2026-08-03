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

mod commun;

use std::collections::{BTreeMap, BTreeSet};
use std::process::Command;

use serde_json::Value;

use commun::perimetre;

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
    // **Les racines viennent du module de périmètre, jamais d'un chemin écrit ici.** Elles s'y
    // composent depuis les `[workspace] members` — un répertoire de famille renommé se voit alors
    // au manifeste, et non trois cycles plus tard sur une porte devenue muette.
    let racine_de = |f: perimetre::Famille| format!("/{}/", f.racine());

    for (famille_perimetre, famille_locale) in [
        (perimetre::Famille::Socle, Famille::Socle),
        (perimetre::Famille::Capacites, Famille::Capacites),
        (perimetre::Famille::Verticales, Famille::Verticales),
        (perimetre::Famille::Domain, Famille::Domain),
    ] {
        if chemin_manifeste.contains(&racine_de(famille_perimetre)) {
            return famille_locale;
        }
    }
    Famille::Assemblage
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

/// **LA CIBLE DE P-03 N'EST PLUS VIDE — et c'est ce test qui le constate.**
///
/// # Trois cycles pendant lesquels la porte ne pouvait rien interdire
///
/// Jusqu'au cycle 004, `verticales/` ne contenait **aucun crate**. La porte parcourait le graphe,
/// ne trouvait aucune arête vers cette famille, et passait au vert — exactement comme elle
/// passerait au vert le jour où quelqu'un supprimerait la famille entière. Les deux situations
/// étaient indistinguables, et c'est la définition d'une porte à cible vide (constitution,
/// § « Couverture des portes »).
///
/// Ce test rend les deux distinguables, à trois niveaux qui ne se remplacent pas :
///
/// 1. **une famille `verticales/` peuplée** — sinon il n'y a rien à interdire ;
/// 2. **du code réel dedans** — un crate au `lib.rs` vide satisferait le point 1 sans qu'aucun
///    symbole ne puisse jamais remonter dans le socle ;
/// 3. **une arête autorisée réellement empruntée** — la verticale dépend du socle. Sans elle, la
///    hiérarchie serait respectée par absence de relation, pas par discipline : deux familles qui
///    ne se parlent pas ne prouvent rien de la règle qui dit dans quel sens elles peuvent le faire.
///
/// La référence compilée à `kaya_hebergement::MODULE_HEBERGEMENT` en fin de test est la garantie
/// la plus forte des trois : si le crate disparaissait, **ce fichier ne compilerait plus**, au lieu
/// de passer au vert en n'ayant rien vu.
#[test]
fn p03_la_cible_de_la_porte_n_est_pas_vide() {
    use std::fs;
    use std::path::Path;

    let graphe = lire_graphe();

    // ── 1 · la famille est peuplée ────────────────────────────────────────────────────────────
    let verticales: Vec<&String> = graphe
        .familles
        .iter()
        .filter(|(_, f)| **f == Famille::Verticales)
        .map(|(nom, _)| nom)
        .collect();

    assert!(
        !verticales.is_empty(),
        "aucun crate de `verticales/` dans le workspace.\n\
         P-03 parcourrait alors le graphe sans trouver une seule arête à interdire, et passerait \
         au vert — indistinguable d'une porte qui fonctionne. C'était l'état des cycles 001 à 003 ; \
         y revenir serait une régression, pas une simplification."
    );

    // ── 2 · il y a du code dedans ─────────────────────────────────────────────────────────────
    //
    // Compté sur les **sources**, pas sur le manifeste : un crate déclaré au workspace avec un
    // `lib.rs` vide satisferait le point 1 tout en n'exposant rien.
    fn compter_items_publics(racine: &Path) -> usize {
        let mut total = 0;
        let Ok(entrees) = fs::read_dir(racine) else {
            return 0;
        };
        for entree in entrees.flatten() {
            let chemin = entree.path();
            if chemin.is_dir() {
                total += compter_items_publics(&chemin);
            } else if chemin.extension().is_some_and(|e| e == "rs") {
                let Ok(source) = fs::read_to_string(&chemin) else {
                    continue;
                };
                total += source
                    .lines()
                    .filter(|l| {
                        let l = l.trim_start();
                        l.starts_with("pub fn ")
                            || l.starts_with("pub struct ")
                            || l.starts_with("pub enum ")
                            || l.starts_with("pub trait ")
                            || l.starts_with("pub const ")
                            || l.starts_with("pub async fn ")
                    })
                    .count();
            }
        }
        total
    }

    let racine_verticales = perimetre::Famille::Verticales.racine();
    let publics = compter_items_publics(Path::new(&racine_verticales));
    assert!(
        publics >= 20,
        "seulement {publics} item(s) public(s) dans `{racine_verticales}/`.\n\
         Un crate présent mais creux rendrait la porte P-03 aussi vide qu'une famille absente : \
         rien ne pourrait remonter dans le socle, donc rien ne pourrait être interdit."
    );

    // ── 3 · une arête autorisée est réellement empruntée ──────────────────────────────────────
    let branchees: Vec<&&String> = verticales
        .iter()
        .filter(|nom| {
            dependances_transitives(&graphe, nom)
                .iter()
                .any(|atteint| graphe.familles[atteint] == Famille::Socle)
        })
        .collect();

    assert!(
        !branchees.is_empty(),
        "aucun crate de `verticales/` ne dépend d'un crate de `socle/` : {verticales:?}\n\
         La hiérarchie serait alors respectée par absence de relation, pas par discipline. Deux \
         familles qui ne se parlent pas ne prouvent rien de la règle qui dit dans quel sens elles \
         peuvent le faire."
    );

    // ── Le garde-fou du compilateur ───────────────────────────────────────────────────────────
    //
    // Si `kaya_hebergement` disparaissait, ce fichier cesserait de compiler. Aucune assertion
    // lue à l'exécution n'offre cette garantie-là.
    assert_eq!(
        kaya_hebergement::MODULE_HEBERGEMENT,
        "HEBERGEMENT",
        "le code du module d'activité a changé : la constante est le nom du module au référentiel \
         d'ETB-02, et le lien entre la verticale et le socle passe par elle"
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

    // Assertion de non-régression (R-15) : la cible doit exister. Le chemin est **composé** par le
    // module de périmètre, qui panique si le crate a quitté le workspace — un chemin écrit à la
    // main aurait, lui, désigné un fichier absent et fait échouer la porte sur la mauvaise cause.
    let crate_domain = perimetre::chemin_domain();
    let crate_fiscalite = perimetre::chemin_crate(perimetre::Famille::Socle, "fiscalite");
    let chemin_fiscal_sans_ext = format!("{crate_domain}/src/fiscal");
    let chemin_fiscal = format!("{chemin_fiscal_sans_ext}.rs");
    let module_fiscal = Path::new(&chemin_fiscal);
    assert!(
        module_fiscal.exists(),
        "{chemin_fiscal} a disparu : la porte P-12 n'a plus de cible et cesserait de vérifier \
         quoi que ce soit sans que rien ne l'indique"
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

        // Les deux emplacements légitimes : le trait lui-même, et la déclaration des types. Les
        // deux chemins sont **composés** depuis le manifeste, ce qui garantit qu'une exemption ne
        // survit pas au crate qu'elle exempte.
        if chemin.contains(&format!("{crate_fiscalite}/")) || chemin.contains(&chemin_fiscal_sans_ext)
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
