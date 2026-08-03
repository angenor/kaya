//! **Le périmètre des portes — DÉCOUVERT, jamais énuméré.**
//!
//! # Le défaut que ce module existe pour empêcher, et ses trois occurrences
//!
//! Dix fichiers de tests énumèrent aujourd'hui leur propre périmètre — les schémas qu'ils
//! inspectent, les crates qu'ils balaient — et le font par une **liste écrite à la main**. Le
//! motif a produit un trou réel à chaque cycle :
//!
//! | Cycle | Ce qui est resté invisible | Combien de temps |
//! |---|---|---|
//! | 002 | 6 tables sur 10 pour la porte P-07 | un cycle entier |
//! | 003 | le schéma `comptes` — **dix tables** échappaient au balayage du registre | un cycle entier |
//! | 004 | le schéma `hebergement` — **huit tables**, même mécanisme | trouvé en fin de cycle |
//!
//! Un quatrième était certain. Ce n'est pas de la distraction : une liste manuelle est correcte le
//! jour où on l'écrit, et elle vieillit sans que rien ne le dise. **Une porte dont la cible
//! rétrécit passe au vert sans rien vérifier**, et c'est le mode de défaillance le plus coûteux
//! qui soit — il donne l'assurance qui empêche la relecture humaine.
//!
//! # Deux sources d'autorité, et rien d'autre
//!
//! | Ce qu'on cherche | Où on le lit | Pourquoi cette source |
//! |---|---|---|
//! | Les schémas applicatifs | `pg_namespace`, **moins une liste d'exclusion nommée** | La base est l'autorité sur ce qui existe. Un schéma créé par migration y apparaît sans que personne y pense |
//! | Les crates du produit | Les `[workspace] members` de `backend/Cargo.toml` | Le manifeste est la source de vérité du principe I(b). Parcourir le système de fichiers verrait un répertoire abandonné non déclaré — donc non compilé — et le compterait comme couvert |
//!
//! **La liste est d'EXCLUSION, jamais d'inclusion.** Un schéma nouveau est inspecté par défaut ;
//! c'est l'inverse qui demande une justification, et chaque exclusion porte la sienne en
//! commentaire. Une liste d'inclusion aurait exactement le défaut qu'on corrige : il faudrait
//! penser à y ajouter le schéma du cycle suivant.
//!
//! # Le contrôle de non-régression, et pourquoi il compte plus que la découverte
//!
//! Découvrir ne suffit pas : une requête qui ne rendrait plus rien — base non migrée, filtre
//! devenu trop large, connexion sur la mauvaise base — rendrait une liste vide, et **toutes les
//! portes passeraient au vert en n'inspectant rien**. Chaque fonction de ce module compare donc
//! son décompte à un **plancher**, et échoue s'il baisse.
//!
//! Un plancher qui monte se met à jour dans le même changement que la migration ou le crate qui
//! l'a fait monter. C'est une ligne à écrire, et c'est le prix de la garantie.
//!
//! # Ce que ce module NE fait pas
//!
//! Il ne dit pas ce qu'une porte doit vérifier, ni comment. Il dit **sur quoi**. La justesse de
//! chaque contrôle reste l'affaire de son fichier — et la limite est la même que partout ailleurs
//! dans ce dépôt : une cible complète ne rend pas un test juste, elle l'empêche seulement d'être
//! vide.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use sqlx::{PgPool, Row};

// =================================================================================================
//  Les schémas — lus du catalogue de PostgreSQL
// =================================================================================================

/// Schémas exclus du périmètre applicatif, **nommés un par un, avec leur motif**.
///
/// Un motif — « tout ce qui commence par `pg_` » — laisserait passer un schéma métier qui s'y
/// conformerait par accident, et surtout n'obligerait personne à justifier l'exclusion. Les deux
/// familles de schémas temporaires de PostgreSQL sont, elles, traitées par préfixe : leur nombre
/// est variable et leur suffixe est un numéro de session, donc innommable.
const SCHEMAS_EXCLUS: &[(&str, &str)] = &[
    (
        "pg_catalog",
        "catalogue système de PostgreSQL — aucun objet du produit",
    ),
    (
        "information_schema",
        "vues normalisées SQL — lecture du catalogue, pas des données",
    ),
    (
        "public",
        "vidé par le produit : `backend/api/sqlx.toml` déplace la table de suivi des migrations \
         ailleurs, et aucune migration n'y crée d'objet métier",
    ),
    (
        "kaya_migrations",
        "table de suivi des migrations de sqlx — des numéros de version appliqués, rien de métier",
    ),
];

/// Préfixes des schémas **temporaires** de PostgreSQL, exclus par famille.
///
/// `pg_toast_*` et `pg_temp_*` portent un numéro de session en suffixe : leur nombre varie d'une
/// connexion à l'autre et aucun ne peut être nommé d'avance.
const PREFIXES_EXCLUS: &[&str] = &["pg_toast", "pg_temp"];

/// Plancher du nombre de schémas applicatifs — **il ne baisse jamais**.
///
/// Cinq à la fin du cycle 004 : `etablissements`, `synchronisation` et `fiscalite` (migration
/// `0001`), `comptes` (`0014`), `hebergement` (`0021`).
///
/// **Un décompte qui baisse est un défaut, jamais un ajustement.** Il signifie soit qu'une
/// migration a détruit un schéma — ce que le principe I(b) interdit —, soit que la liste
/// d'exclusion est devenue trop large, soit que la base inspectée n'est pas celle qu'on croit. Les
/// trois méritent un échec bruyant.
pub const PLANCHER_SCHEMAS: usize = 5;

/// Les schémas applicatifs, **découverts** dans le catalogue de la base.
///
/// # Pourquoi pas un filtrage par propriétaire
///
/// `nspowner = kaya_owner` a été envisagé et écarté : il lie le périmètre d'une porte à la
/// configuration d'un rôle, donc à `scripts/dev/preparer-base.sh`. Un changement de rôle
/// applicatif viderait silencieusement la cible de dix portes. L'exclusion nommée ne dépend que de
/// PostgreSQL lui-même.
pub async fn schemas_applicatifs(pool: &PgPool) -> Vec<String> {
    let exclus: Vec<String> = SCHEMAS_EXCLUS
        .iter()
        .map(|(nom, _)| (*nom).to_owned())
        .collect();

    let schemas: Vec<String> = sqlx::query(
        r#"
        SELECT nspname
        FROM pg_namespace
        WHERE nspname <> ALL($1)
          AND nspname NOT LIKE 'pg\_toast%'
          AND nspname NOT LIKE 'pg\_temp%'
        ORDER BY nspname
        "#,
    )
    .bind(&exclus)
    .fetch_all(pool)
    .await
    .expect("lecture de pg_namespace")
    .into_iter()
    .map(|l| l.get::<String, _>("nspname"))
    .collect();

    assert!(
        schemas.len() >= PLANCHER_SCHEMAS,
        "{} schéma(s) applicatif(s) découvert(s) pour un plancher de {PLANCHER_SCHEMAS} : {:?}\n\n\
         Un décompte qui BAISSE n'est jamais un ajustement. Trois causes possibles, toutes des \
         défauts :\n  \
         · une migration a détruit un schéma — le principe I(b) l'interdit ;\n  \
         · la liste d'exclusion de `commun/perimetre.rs` est devenue trop large ;\n  \
         · la base inspectée n'est pas celle qu'on croit (DATABASE_URL, base non migrée).\n\n\
         Exclus par construction : {}\n\
         Préfixes exclus : {PREFIXES_EXCLUS:?}",
        schemas.len(),
        schemas,
        SCHEMAS_EXCLUS
            .iter()
            .map(|(nom, motif)| format!("{nom} ({motif})"))
            .collect::<Vec<_>>()
            .join(" · ")
    );

    schemas
}

// =================================================================================================
//  Les crates — lus des `[workspace] members`
// =================================================================================================

/// Le manifeste du workspace, lu **à la compilation**.
///
/// `include_str!` plutôt qu'une lecture au démarrage : une modification du manifeste recompile
/// tous les tests qui en dépendent, et un manifeste devenu illisible échoue à la compilation
/// plutôt qu'à l'exécution, là où l'erreur serait confondue avec un défaut de base.
const MANIFESTE: &str = include_str!("../../Cargo.toml");

/// Les trois familles de la hiérarchie du principe II, plus le noyau de types partagés.
///
/// **Les noms sont ceux des répertoires, pas des chemins.** C'est délibéré : rien dans ce dépôt ne
/// doit écrire `crates/socle` en toutes lettres hors de ce module, et un nom de famille se
/// compose, il ne se cherche pas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Famille {
    /// `domain` — moteur fiscal, barèmes, types. Hors hiérarchie : tout le monde en dépend.
    Domain,
    /// Le noyau agnostique. Ne dépend que de lui-même et de `domain`.
    Socle,
    /// Les transverses — stock, livraison, production. Dépendent du socle.
    Capacites,
    /// Les métiers — hébergement, restauration, bar, pressing. Dépendent des deux autres.
    Verticales,
}

impl Famille {
    /// Le segment de répertoire de cette famille.
    pub fn segment(self) -> &'static str {
        match self {
            Famille::Domain => "domain",
            Famille::Socle => "socle",
            Famille::Capacites => "capacites",
            Famille::Verticales => "verticales",
        }
    }

    /// La racine de la famille, relative à `backend/` — par exemple `crates/socle`.
    pub fn racine(self) -> String {
        format!("{RACINE_CRATES}/{}", self.segment())
    }
}

/// Le répertoire qui accueille tous les crates du produit.
const RACINE_CRATES: &str = "crates";

/// Planchers par famille — **ils ne baissent jamais**, même raison qu'au plancher des schémas.
///
/// **Neuf** crates de socle — `etablissements`, `comptes`, `caisse`, `fiscalite`, `documents`,
/// `synchronisation`, `pilotage`, `editeur`, `metriques` —, une capacité (`stocks`), quatre
/// verticales à la fin du cycle 004. `domain` n'est d'aucune des trois familles : il est hors
/// hiérarchie, et tout le monde en dépend.
pub const PLANCHER_SOCLE: usize = 9;
/// Voir [`PLANCHER_SOCLE`].
pub const PLANCHER_CAPACITES: usize = 1;
/// Voir [`PLANCHER_SOCLE`].
pub const PLANCHER_VERTICALES: usize = 4;

/// Les membres du workspace, tels que `backend/Cargo.toml` les déclare.
///
/// L'extraction est mécanique et **stricte** : le bloc `members = [` du `[workspace]`, jusqu'au
/// `]` fermant. Un manifeste reformaté doit faire échouer bruyamment plutôt que rendre une liste
/// vide — qui passerait au vert en n'ayant rien à inspecter.
pub fn membres_du_workspace() -> Vec<String> {
    let Some(debut) = MANIFESTE.find("[workspace]") else {
        panic!(
            "`[workspace]` introuvable dans backend/Cargo.toml. C'est la source de vérité du \
             principe I(b) sur les crates du produit ; sans elle, dix portes ne savent plus sur \
             quoi elles portent."
        );
    };
    let apres = &MANIFESTE[debut..];

    let Some(debut_membres) = apres.find("members = [") else {
        panic!("`members = [` introuvable sous `[workspace]` dans backend/Cargo.toml");
    };
    let liste = &apres[debut_membres + "members = [".len()..];
    let Some(fin) = liste.find(']') else {
        panic!("le tableau `members` de backend/Cargo.toml n'est pas refermé");
    };

    let membres: Vec<String> = liste[..fin]
        .split(',')
        .filter_map(|brut| {
            let brut = brut.trim();
            let debut = brut.find('"')?;
            let reste = &brut[debut + 1..];
            let fin = reste.find('"')?;
            Some(reste[..fin].to_owned())
        })
        .collect();

    assert!(
        membres.len() >= PLANCHER_SOCLE + PLANCHER_CAPACITES + PLANCHER_VERTICALES,
        "{} membre(s) extrait(s) de backend/Cargo.toml : l'extraction est probablement cassée, et \
         toute porte qui s'y appuie passerait au vert sur une liste vide. Le tableau `members` \
         a-t-il été reformaté ?\n\
         Extraits : {membres:?}",
        membres.len()
    );

    membres
}

/// Les crates d'une famille, chemins **relatifs à `backend/`**, triés.
///
/// La famille se lit du **chemin déclaré au manifeste**, jamais du nom du crate : `kaya-hebergement`
/// pourrait être renommé, déplacé, ou porter un nom trompeur. L'arborescence, elle, est celle que
/// la constitution fixe.
pub fn crates_de(famille: Famille) -> Vec<String> {
    let prefixe = format!("{}/", famille.racine());
    let mut trouves: Vec<String> = membres_du_workspace()
        .into_iter()
        .filter(|membre| membre.starts_with(&prefixe))
        .collect();
    trouves.sort();
    trouves
}

/// Les dix crates du socle. **Découverts**, comptés, et le décompte ne baisse pas.
pub fn crates_du_socle() -> Vec<String> {
    exiger_plancher(Famille::Socle, PLANCHER_SOCLE)
}

/// Les crates de capacités — `stocks` seul à ce jour.
pub fn crates_des_capacites() -> Vec<String> {
    exiger_plancher(Famille::Capacites, PLANCHER_CAPACITES)
}

/// Les quatre verticales métier.
pub fn crates_des_verticales() -> Vec<String> {
    exiger_plancher(Famille::Verticales, PLANCHER_VERTICALES)
}

/// Les crates des **trois familles de la hiérarchie**, dans l'ordre socle → capacités →
/// verticales.
///
/// C'est le périmètre de toute porte qui balaie « le code du produit » : `domain` en est **exclu**
/// délibérément — il ne porte ni service, ni accès base, ni règle d'établissement, et l'y inclure
/// diluerait les décomptes sans rien couvrir de plus.
pub fn crates_metier() -> Vec<String> {
    let mut tous = crates_du_socle();
    tous.extend(crates_des_capacites());
    tous.extend(crates_des_verticales());
    tous
}

fn exiger_plancher(famille: Famille, plancher: usize) -> Vec<String> {
    let trouves = crates_de(famille);
    assert!(
        trouves.len() >= plancher,
        "{} crate(s) découvert(s) dans la famille « {} » pour un plancher de {plancher} : {:?}\n\n\
         Un décompte qui baisse est un défaut : soit un crate a été retiré du workspace — donc \
         n'est plus compilé, ni inspecté par aucune porte —, soit l'extraction du manifeste est \
         cassée. Une porte dont la cible rétrécit passe au vert sans rien vérifier.",
        trouves.len(),
        famille.segment(),
        trouves
    );
    trouves
}

/// Le chemin d'un crate **nommé**, relatif à `backend/`, vérifié contre le manifeste.
///
/// # Pourquoi passer par ici plutôt qu'écrire le chemin
///
/// Une porte a parfois besoin d'un crate précis — `socle/fiscalite` pour P-12, `socle/comptes`
/// pour la taxonomie d'audit. Écrire `"crates/socle/fiscalite"` marcherait, jusqu'au jour où le
/// crate serait renommé ou sorti du workspace : la lecture du fichier échouerait en silence, la
/// porte n'inspecterait rien, et rien ne le dirait.
///
/// Cette fonction **panique** si le crate n'est pas déclaré au workspace. Un chemin qui ne
/// correspond à rien devient une erreur immédiate, pas une cible vide.
pub fn chemin_crate(famille: Famille, nom: &str) -> String {
    let chemin = format!("{}/{nom}", famille.racine());
    assert!(
        membres_du_workspace().iter().any(|m| m == &chemin),
        "« {chemin} » n'est pas déclaré aux `[workspace] members` de backend/Cargo.toml. Le crate \
         a-t-il été renommé, déplacé, ou sorti du workspace ? Une porte qui viserait un chemin \
         inexistant n'inspecterait rien — et passerait au vert."
    );
    chemin
}

/// Le chemin du crate `domain`, relatif à `backend/`, vérifié contre le manifeste.
///
/// `domain` n'appartient à aucune des trois familles de la hiérarchie : il est **seul de sa
/// famille**, et [`chemin_crate`] lui composerait un sous-répertoire qui n'existe pas. Une
/// fonction séparée est plus honnête qu'un cas particulier caché dans la précédente.
pub fn chemin_domain() -> String {
    let chemin = Famille::Domain.racine();
    assert!(
        membres_du_workspace().iter().any(|m| m == &chemin),
        "« {chemin} » n'est pas déclaré aux `[workspace] members` de backend/Cargo.toml. Le crate \
         des types partagés a-t-il été renommé ou déplacé ? Les portes P-11 et P-12 y ont leur \
         cible ; sans lui elles n'inspecteraient rien."
    );
    chemin
}

/// Le chemin d'un fichier **dans** un crate nommé, relatif à `backend/`.
///
/// Raccourci de [`chemin_crate`] pour le cas courant : viser un fichier précis d'un crate précis,
/// en gardant la vérification que le crate existe.
pub fn fichier_du_crate(famille: Famille, nom: &str, relatif: &str) -> String {
    format!("{}/{relatif}", chemin_crate(famille, nom))
}

/// Racine du répertoire `backend/`, quel que soit le répertoire d'exécution des tests.
pub fn racine_backend() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Tous les fichiers `.rs` sous un répertoire, récursivement, triés.
///
/// `target/` est écarté : un crate qui en porterait un localement noierait le balayage sous des
/// sources engendrées.
pub fn sources_rust(racine: &Path) -> Vec<PathBuf> {
    let mut fichiers = Vec::new();
    collecter(racine, &mut fichiers);
    fichiers.sort();
    fichiers
}

fn collecter(repertoire: &Path, sortie: &mut Vec<PathBuf>) {
    let Ok(entrees) = std::fs::read_dir(repertoire) else {
        return;
    };
    for entree in entrees.flatten() {
        let chemin = entree.path();
        if chemin.is_dir() {
            if chemin.file_name().is_some_and(|n| n == "target") {
                continue;
            }
            collecter(&chemin, sortie);
        } else if chemin.extension().is_some_and(|e| e == "rs") {
            sortie.push(chemin);
        }
    }
}

/// Les fichiers `.rs` de **tous les crates métier**, chemins absolus.
///
/// C'est le balayage que la plupart des portes veulent : le code du produit, sans `domain`, sans
/// `api`, sans les tests. Chaque appelant filtre ensuite ce qui le concerne.
pub fn sources_des_crates_metier() -> Vec<PathBuf> {
    let racine = racine_backend();
    let mut fichiers = Vec::new();
    for membre in crates_metier() {
        fichiers.extend(sources_rust(&racine.join(&membre)));
    }
    fichiers.sort();

    assert!(
        !fichiers.is_empty(),
        "aucun fichier `.rs` sous les crates métier découverts. Une porte dont la cible est vide \
         passe toujours."
    );

    fichiers
}

// =================================================================================================
//  Le contrôle prospectif de FR-004c — « toute porte future en hérite »
// =================================================================================================

/// Une déclaration de périmètre écrite **à la main** dans un fichier de test.
#[derive(Debug, Clone)]
pub struct PerimetreEnDur {
    /// Le fichier fautif, relatif à `backend/`.
    pub fichier: String,
    /// La ligne, à partir de 1.
    pub ligne: usize,
    /// Le nom de la constante, ou la valeur littérale qui a déclenché le constat.
    pub declaration: String,
    /// Ce qui a été reconnu — pour que le message dise quoi corriger, pas seulement que ça cloche.
    pub motif: &'static str,
}

/// Fichiers de `backend/tests/` **exemptés** du contrôle prospectif, avec leur motif.
///
/// La liste est nommée, et elle est courte parce qu'elle doit le rester : chaque entrée est une
/// porte qui garde le droit d'écrire un périmètre à la main, donc une occasion future de laisser
/// un trou.
const FICHIERS_EXEMPTES: &[(&str, &str)] = &[(
    "commun/perimetre.rs",
    "c'est ce module — il EST la source, il ne peut pas s'y adosser",
)];

/// Les fichiers de `backend/tests/` qui déclarent un périmètre **en dur**.
///
/// # Ce que le contrôle reconnaît, et pourquoi ces deux motifs
///
/// | Motif | Ce qui le déclenche | Pourquoi c'est un périmètre |
/// |---|---|---|
/// | Nom de constante | `const …SCHEMA…` ou `const …CRATE…` de type tableau | Une constante qui nomme des schémas ou des crates **est** un périmètre, quelle que soit sa forme |
/// | Chemin de crate littéral | une valeur commençant par `crates/<famille>/` | C'est la forme exacte des vingt et une occurrences que ce cycle a supprimées |
///
/// **Le second motif est le plus utile** : il n'exige rien du nommage, et attrape la faute telle
/// qu'elle a réellement été commise quatre cycles de suite — un chemin de crate recopié dans un
/// `const`, un `Path::new`, un `include_str!`.
///
/// # Ce qu'il ne sait pas voir
///
/// Un périmètre construit à l'exécution — concaténation, `format!` sur des fragments, lecture
/// d'un fichier tiers. Le dire vaut mieux que laisser croire à une garantie : ce contrôle rend
/// **coûteux** de réintroduire une liste, il ne le rend pas impossible.
pub fn perimetres_en_dur() -> Vec<PerimetreEnDur> {
    let racine_tests = racine_backend().join("tests");
    let prefixes_de_crate: Vec<String> = [
        Famille::Domain,
        Famille::Socle,
        Famille::Capacites,
        Famille::Verticales,
    ]
    .iter()
    .map(|f| format!("{}/", f.racine()))
    .collect();

    let mut constats = Vec::new();

    for fichier in sources_rust(&racine_tests) {
        let relatif = fichier
            .strip_prefix(&racine_tests)
            .unwrap_or(&fichier)
            .display()
            .to_string()
            .replace('\\', "/");

        if FICHIERS_EXEMPTES.iter().any(|(nom, _)| *nom == relatif) {
            continue;
        }

        let Ok(contenu) = std::fs::read_to_string(&fichier) else {
            continue;
        };

        for (rang, ligne) in contenu.lines().enumerate() {
            let nu = ligne.trim();
            if nu.starts_with("//") || nu.starts_with('*') || nu.starts_with("//!") {
                continue;
            }

            // Motif 1 — une constante de type tableau dont le nom parle de schémas ou de crates.
            if let Some(apres) = nu.strip_prefix("const ")
                && let Some(nom) = apres.split(':').next()
            {
                let nom = nom.trim();
                let est_tableau = nu.contains("&[");
                let parle_de_perimetre = nom.contains("SCHEMA") || nom.contains("CRATE");
                if est_tableau && parle_de_perimetre {
                    constats.push(PerimetreEnDur {
                        fichier: relatif.clone(),
                        ligne: rang + 1,
                        declaration: nom.to_owned(),
                        motif: "une constante de tableau qui nomme des schémas ou des crates",
                    });
                }
            }

            // Motif 2 — un chemin de crate écrit en toutes lettres, sous quelque forme que ce soit.
            for prefixe in &prefixes_de_crate {
                if nu.contains(&format!("\"{prefixe}")) {
                    constats.push(PerimetreEnDur {
                        fichier: relatif.clone(),
                        ligne: rang + 1,
                        declaration: nu.chars().take(100).collect(),
                        motif: "un chemin de crate écrit en toutes lettres",
                    });
                    break;
                }
            }
        }
    }

    constats
}

/// Les schémas exclus, pour qu'une porte puisse **imprimer** ce qu'elle n'inspecte pas
/// (exigence 1 du § « Couverture des portes » : déclarer son périmètre, et ce qu'il laisse dehors).
pub fn exclusions_declarees() -> Vec<String> {
    SCHEMAS_EXCLUS
        .iter()
        .map(|(nom, motif)| format!("{nom} — {motif}"))
        .chain(
            PREFIXES_EXCLUS
                .iter()
                .map(|p| format!("{p}* — schémas temporaires de PostgreSQL, suffixe de session")),
        )
        .collect()
}

/// Deux crates ne sont jamais comptés dans deux familles — invariant employé par les portes qui
/// somment les décomptes.
pub fn familles_disjointes() -> bool {
    let socle = crates_du_socle();
    let capacites = crates_des_capacites();
    let verticales = crates_des_verticales();
    let total = socle.len() + capacites.len() + verticales.len();
    let distincts: BTreeSet<String> = socle
        .into_iter()
        .chain(capacites)
        .chain(verticales)
        .collect();
    distincts.len() == total
}
