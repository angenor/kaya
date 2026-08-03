//! **Le pendant exact de `classes_offline.rs`** — celui-là vérifie qu'une classe est *déclarée*,
//! celui-ci qu'elle est *exercée*.
//!
//! ═══════════════════════════════════════════════════════════════════════════════════════════
//!  LE TROU QUE CE FICHIER FERME
//! ═══════════════════════════════════════════════════════════════════════════════════════════
//!
//! `classes_offline.rs` fait échouer le build sur toute table absente du registre. C'est ce qui
//! garantit qu'aucune entité n'est créée sans que quelqu'un ait ouvert le registre et écrit une
//! classe.
//!
//! **Il ne garantit rien de plus**, et l'écart est exactement celui-ci : une entité peut être
//! déclarée de classe A au registre, avoir sa table, et n'avoir **jamais** subi le rejeu triple ni
//! le désordre. La déclaration serait alors une intention, pas une propriété — et le §0.7 des user
//! stories exige la seconde.
//!
//! Le cas n'est pas théorique. `occupation` (cycle 004) a sa table, sa classe déclarée, et son
//! rejeu n'a **jamais** vérifié qu'un second envoi n'émet aucun événement outbox : le contrôle
//! existait pour `note_etablissement`, et il a été perdu à la réécriture. C'est précisément ce que
//! l'outillage engendré empêche, et ce fichier-ci fait que l'oubli d'instancier se voie.
//!
//! ═══════════════════════════════════════════════════════════════════════════════════════════
//!  PÉRIMÈTRE INSPECTÉ — exigence 1
//! ═══════════════════════════════════════════════════════════════════════════════════════════
//!
//! **Inspecté** : toute entité du registre **qui a une table réelle**, sur les schémas que
//! `commun::perimetre` découvre. La jonction des deux sources est le point : le registre décrit
//! tout le produit, y compris ce que six cycles construiront ; seules les entités **implémentées**
//! peuvent être exercées.
//!
//! **Non inspecté**, et il faut le lire avant de conclure d'un vert :
//!
//! - **La justesse de la classe.** Instancier `tester_classe_a!` sur une entité qui devrait être B
//!   produit six tests verts sur un classement faux. Aucune lecture du schéma ne retrouve qu'un
//!   encaissement est B en espèces et D en Mobile Money : c'est métier, revu mensuellement.
//! - **La qualité de l'instanciation.** Que `tester_classe_a!` soit appelée ne dit pas que ses
//!   paramètres sont justes — un `chemin` pointant sur le mauvais endpoint produirait des tests
//!   qui passent en n'exerçant pas l'entité annoncée.
//! - **Les provisions.** Une table sans aucun privilège d'écriture pour `kaya_app` ne peut être
//!   exercée par aucun test d'écriture, et c'est le but : `provisions_sans_logique.rs` la garde.

mod commun;

use std::collections::{BTreeMap, BTreeSet};

use commun::perimetre;
use sqlx::Row;

/// Le registre, lu à la compilation.
const REGISTRE: &str = include_str!("../../docs/registre-classes-offline.md");

/// Les fichiers de tests **du produit**, lus à la compilation.
///
/// `include_str!` plutôt qu'une lecture au démarrage : un fichier retiré de cette liste ne compile
/// plus, au lieu de rétrécir la cible en silence. C'est le même garde-fou que
/// `TESTS_QUI_EXERCENT_L_AUDIT` de `couverture_portes.rs`.
const SOURCES_DE_TESTS: &[(&str, &str)] = &[
    (
        "note_etablissement_classe_a.rs",
        include_str!("note_etablissement_classe_a.rs"),
    ),
    ("audit_classe_a.rs", include_str!("audit_classe_a.rs")),
    (
        "hebergement_hors_ligne.rs",
        include_str!("hebergement_hors_ligne.rs"),
    ),
    (
        "hebergement_disponibilite.rs",
        include_str!("hebergement_disponibilite.rs"),
    ),
    (
        "hebergement_referentiel.rs",
        include_str!("hebergement_referentiel.rs"),
    ),
    ("classes_offline.rs", include_str!("classes_offline.rs")),
    ("isolation_tenant.rs", include_str!("isolation_tenant.rs")),
    (
        "outbox_transactionnel.rs",
        include_str!("outbox_transactionnel.rs"),
    ),
    (
        "provisions_sans_logique.rs",
        include_str!("provisions_sans_logique.rs"),
    ),
    ("derive_horloge.rs", include_str!("derive_horloge.rs")),
    (
        "personne_compte_employe.rs",
        include_str!("personne_compte_employe.rs"),
    ),
    ("roles_cumules.rs", include_str!("roles_cumules.rs")),
    (
        "session_revocation.rs",
        include_str!("session_revocation.rs"),
    ),
    (
        "configuration_heritee.rs",
        include_str!("configuration_heritee.rs"),
    ),
    (
        "parametres_catalogue.rs",
        include_str!("parametres_catalogue.rs"),
    ),
    (
        "capacites_refusees.rs",
        include_str!("capacites_refusees.rs"),
    ),
    (
        "branding_identite_visuelle.rs",
        include_str!("branding_identite_visuelle.rs"),
    ),
    (
        "desactivation_bloquee.rs",
        include_str!("desactivation_bloquee.rs"),
    ),
    (
        "hebergement_tarification.rs",
        include_str!("hebergement_tarification.rs"),
    ),
    ("seeds_rejouables.rs", include_str!("seeds_rejouables.rs")),
    ("audit_immuabilite.rs", include_str!("audit_immuabilite.rs")),
    // ── Cycle 006 (SEJ) ────────────────────────────────────────────────────────────────────
    (
        "client_classes_offline.rs",
        include_str!("client_classes_offline.rs"),
    ),
    ("client_recherche.rs", include_str!("client_recherche.rs")),
    ("sejour_arrivee.rs", include_str!("sejour_arrivee.rs")),
    ("sejour_hors_ligne.rs", include_str!("sejour_hors_ligne.rs")),
    (
        "accompagnant_classe_a.rs",
        include_str!("accompagnant_classe_a.rs"),
    ),
    (
        "outbox_immuabilite.rs",
        include_str!("outbox_immuabilite.rs"),
    ),
    ("rls_catalogue.rs", include_str!("rls_catalogue.rs")),
    (
        "permissions_par_module.rs",
        include_str!("permissions_par_module.rs"),
    ),
    (
        "politique_mot_de_passe.rs",
        include_str!("politique_mot_de_passe.rs"),
    ),
    (
        "authentification_indiscernable.rs",
        include_str!("authentification_indiscernable.rs"),
    ),
    (
        "reconstitution_autonome.rs",
        include_str!("reconstitution_autonome.rs"),
    ),
    (
        "agnosticite_socle.rs",
        include_str!("agnosticite_socle.rs"),
    ),
    (
        "worker_redemarrage.rs",
        include_str!("worker_redemarrage.rs"),
    ),
    (
        "migrations_concurrentes.rs",
        include_str!("migrations_concurrentes.rs"),
    ),
];

/// Les tables **exemptées** d'exercice, nommées une par une avec leur motif.
///
/// La liste est courte et doit le rester : chaque entrée est une entité dont personne ne vérifie la
/// classe, donc une occasion future de se tromper sans que rien ne le dise.
const EXEMPTIONS: &[(&str, &str)] = &[
    // Les référentiels globaux de l'éditeur : `kaya_app` n'a que `SELECT`, l'écriture appartient à
    // `kaya_owner` par migration. Aucun rejeu, aucun désordre à exercer — il n'y a pas d'écriture
    // applicative à rejouer.
    ("module_activite", "référentiel global — écriture par migration seulement"),
    ("capacite", "référentiel global — écriture par migration seulement"),
    ("profil_stock", "référentiel global — écriture par migration seulement"),
    ("parametre_catalogue", "référentiel global — écriture par migration seulement"),
    ("methode_authentification", "référentiel global — écriture par migration seulement"),
    ("role", "référentiel global — écriture par migration seulement"),
    ("permission", "référentiel global — écriture par migration seulement"),
    ("role_permission", "jointure de référentiel — aucun cycle de vie propre"),
    // Les provisions : `provisions_sans_logique.rs` garde qu'elles n'ont AUCUN chemin d'écriture.
    // Les exercer demanderait d'en créer un, c'est-à-dire de cesser d'être une provision.
    ("employe", "provision — aucun privilège d'écriture (principe X)"),
    ("appareil_enrole", "provision — aucun privilège d'écriture (principe X)"),
    ("prestation_incluse", "provision — aucun privilège d'écriture (principe X)"),
    ("reconciliation_orpheline", "provision — `SELECT` seul (principe X)"),
    ("exercice_comptable", "provision comptable — aucun chemin d'écriture applicatif"),
    ("mapping_comptable", "provision comptable — aucun chemin d'écriture applicatif"),
    // La racine du modèle : un tenant se crée par l'éditeur, hors de toute file.
    ("tenant", "racine du modèle — création par l'éditeur, jamais depuis un terminal"),
];

/// Une entité du registre qui a une table réelle.
#[derive(Debug, Clone)]
struct EntiteImplementee {
    nom: String,
    schema: String,
}

/// Les tables réelles, par nom, sur les schémas **découverts**.
async fn tables_reelles(pool: &sqlx::PgPool) -> BTreeMap<String, String> {
    let schemas = perimetre::schemas_applicatifs(pool).await;

    sqlx::query(
        r#"
        SELECT table_schema, table_name
        FROM information_schema.tables
        WHERE table_schema = ANY($1) AND table_type = 'BASE TABLE'
        "#,
    )
    .bind(&schemas)
    .fetch_all(pool)
    .await
    .expect("lecture du catalogue")
    .into_iter()
    .map(|l| {
        (
            l.get::<String, _>("table_name").to_lowercase(),
            l.get::<String, _>("table_schema"),
        )
    })
    .collect()
}

/// Les entités déclarées au registre — même extraction que `classes_offline.rs`.
fn entites_du_registre() -> BTreeSet<String> {
    let mut entites = BTreeSet::new();
    for ligne in REGISTRE.lines() {
        let ligne = ligne.trim();
        if !ligne.starts_with('|') {
            continue;
        }
        let Some(cellule) = ligne.split('|').nth(1) else {
            continue;
        };
        let mut reste = cellule;
        while let Some(debut) = reste.find('`') {
            let apres = &reste[debut + 1..];
            let Some(fin) = apres.find('`') else { break };
            let brut = &apres[..fin];
            reste = &apres[fin + 1..];
            let nom = brut.split('.').next().unwrap_or(brut).trim();
            if !nom.is_empty() && nom.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                entites.insert(nom.to_lowercase());
            }
        }
    }
    entites
}

/// Cette entité est-elle **exercée** par un test du produit ?
///
/// # Ce que « exercée » veut dire, et pourquoi c'est plus lâche qu'on voudrait
///
/// Un test exerce une entité s'il la **nomme** — nom de table, nom d'agrégat, instanciation de
/// macro. La reconnaissance est textuelle, et c'est une limite assumée : rien ne distingue, dans le
/// texte, un test qui exerce d'un test qui mentionne.
///
/// L'alternative aurait été d'exiger une instanciation de macro pour **toutes** les entités, ce qui
/// aurait fait échouer la porte sur les huit référentiels globaux et les six provisions — donc de
/// l'exempter partout, donc de ne rien garder. Ce qui est gardé ici est plus modeste et suffit :
/// **une entité implémentée sans aucun test qui la nomme est certainement non exercée.**
fn est_exercee(entite: &str) -> Option<&'static str> {
    SOURCES_DE_TESTS
        .iter()
        .find(|(_, source)| source.contains(entite))
        .map(|(nom, _)| *nom)
}

// =================================================================================================
//  Les contrôles
// =================================================================================================

/// **Toute entité implémentée est exercée par au moins un test.**
#[tokio::test]
async fn toute_entite_implementee_est_exercee_par_un_test() {
    let pool = commun::pool_owner().await;
    let tables = tables_reelles(&pool).await;
    let registre = entites_du_registre();

    assert!(
        tables.len() >= 35,
        "seulement {} table(s) découverte(s) : la cible est vide ou la base n'est pas migrée. \
         Une porte dont la cible rétrécit passe au vert sans rien vérifier.",
        tables.len()
    );
    assert!(
        registre.len() > 50,
        "seulement {} entité(s) extraite(s) du registre : l'extraction est cassée",
        registre.len()
    );

    let implementees: Vec<EntiteImplementee> = tables
        .iter()
        .filter(|(nom, _)| registre.contains(*nom))
        .filter(|(nom, _)| !EXEMPTIONS.iter().any(|(exempte, _)| exempte == nom))
        .map(|(nom, schema)| EntiteImplementee {
            nom: nom.clone(),
            schema: schema.clone(),
        })
        .collect();

    assert!(
        !implementees.is_empty(),
        "aucune entité implémentée non exemptée : la porte n'a rien à vérifier, et les exemptions \
         couvrent tout. C'est le mode de défaillance qu'une liste d'exemptions produit quand elle \
         grandit sans qu'on la relise."
    );

    let mut non_exercees = Vec::new();
    let mut exercees = 0usize;

    for entite in &implementees {
        match est_exercee(&entite.nom) {
            Some(_) => exercees += 1,
            None => non_exercees.push(format!("{}.{}", entite.schema, entite.nom)),
        }
    }

    assert!(
        non_exercees.is_empty(),
        "{} entité(s) déclarée(s) au registre, dotée(s) d'une table, et exercée(s) par AUCUN \
         test :\n  {}\n\n\
         Le registre dit qu'elles ont une classe ; rien ne dit qu'elles la respectent. Une entité \
         déclarée de classe A qui n'a jamais subi le rejeu triple ni le désordre porte une \
         intention, pas une propriété — et le §0.7 des user stories exige la seconde.\n\n\
         Le remède coûte une déclaration :\n  \
         tester_classe_a!(<entité>, schema = \"…\", table = \"…\", agregat = \"…\", chemin = …, corps = …);\n\
         ou `tester_classe_bcd!` pour une opération de classe B, C ou D.\n\n\
         Si l'entité ne PEUT pas être exercée — référentiel écrit par migration, provision sans \
         privilège —, l'inscrire aux EXEMPTIONS de ce fichier **avec son motif**. Une exemption \
         sans motif est une entité dont personne ne vérifiera jamais la classe.",
        non_exercees.len(),
        non_exercees.join("\n  ")
    );

    println!(
        "outillage §0.7 — {exercees} entité(s) implémentée(s) exercée(s), {} exemptée(s) avec \
         motif, sur {} table(s) découverte(s) :",
        EXEMPTIONS.len(),
        tables.len()
    );
    for (exempte, motif) in EXEMPTIONS {
        if tables.contains_key(*exempte) {
            println!("    {exempte} — {motif}");
        }
    }
}

/// **Les macros du §0.7 sont réellement instanciées quelque part.**
///
/// Un outillage que personne n'appelle est du code exporté et appelé nulle part — le défaut exact
/// d'`initialiserTheme()`, qui a vécu deux cycles. Écrire trois macros et n'en instancier aucune
/// laisserait ce fichier vert et l'outillage mort.
#[test]
fn les_macros_du_paragraphe_07_ont_au_moins_une_instanciation() {
    let mut instanciations: BTreeMap<&str, Vec<&str>> = BTreeMap::new();

    for macro_nom in ["tester_classe_a!", "tester_classe_bcd!", "tester_classe_d!"] {
        for (fichier, source) in SOURCES_DE_TESTS {
            // **L'appel, pas la mention** : `tester_classe_a!(` avec son point d'exclamation ET
            // sa parenthèse. Chercher le nom seul ferait passer ce fichier-ci pour un appelant,
            // puisqu'il les nomme toutes les trois.
            if source.contains(&format!("{macro_nom}(")) {
                instanciations.entry(macro_nom).or_default().push(fichier);
            }
        }
    }

    // `tester_classe_d!` est installée À VIDE — aucune opération de classe D n'existe dans le
    // produit (certification FNE et Mobile Money sont de la tranche T3). Elle est donc attendue
    // **sans instanciation**, et c'est écrit ici plutôt que découvert.
    for attendue in ["tester_classe_a!", "tester_classe_bcd!"] {
        assert!(
            instanciations.contains_key(attendue),
            "la macro `{attendue}` n'est instanciée nulle part. Un outillage que personne n'appelle \
             est du code mort — et il donne l'illusion d'une couverture qui n'existe pas."
        );
    }

    println!("outillage §0.7 — instanciations trouvées :");
    for (macro_nom, fichiers) in &instanciations {
        println!("    {macro_nom} → {}", fichiers.join(", "));
    }
    println!(
        "    tester_classe_d! → aucune, et c'est attendu : aucune opération de classe D n'existe \
         (certification FNE, Mobile Money — tranche T3)"
    );
}

/// **Test négatif — la porte sait signaler une entité non exercée.**
///
/// Exercé sur un ensemble simulé : introduire une vraie table non exercée ferait échouer les
/// voisins, qui inventorient la même base en parallèle (leçon de `classes_offline.rs`).
#[test]
fn test_negatif_une_entite_non_exercee_est_signalee() {
    assert!(
        est_exercee("zzz_entite_jamais_testee").is_none(),
        "la porte prétend qu'une entité inventée est exercée : la reconnaissance est cassée, et \
         elle déclarerait tout conforme"
    );
    assert!(
        est_exercee("note_etablissement").is_some(),
        "la porte ne reconnaît plus une entité pourtant exercée par trois fichiers : elle \
         échouerait sur tout et serait désactivée dans la semaine"
    );
}
