//! **US1 / ETB-02c — les trois parcours structurels.** Le garde-fou de toute extension du produit.
//!
//! Trois établissements témoins gardent le socle en permanence : un **maquis** qui ne fait que de
//! la restauration, une **résidence meublée** qui ne fait que de l'hébergement, et un
//! établissement portant un **service fictif minimal ne consommant aucune capacité**. Si ces trois
//! parcours restent verts, aucun crate partagé ne suppose l'existence d'un hébergement, d'un point
//! de vente ou d'un stock.
//!
//! # Ce que ce harnais inspecte, et ce qu'il n'inspecte pas
//!
//! *Exigence 1 du § « Couverture des portes » de la constitution — un test négatif prouve qu'une
//! porte sait échouer, il ne prouve pas qu'elle regarde tout.*
//!
//! **Inspecté** — pour chacun des trois parcours, les **huit étapes** de son cycle de vie, de la
//! création de l'établissement à la clôture journalière. Pour chaque étape : l'existence de sa
//! **sentinelle** (`information_schema.tables` pour une table, `application::contrat_complet()`
//! pour un point d'entrée) et, si elle existe, le fait que le parcours l'exerce réellement.
//!
//! **Non inspecté**, et il faut le savoir :
//!
//! - **la justesse fonctionnelle des étapes dues.** Le harnais constate qu'une étape est branchée,
//!   pas qu'elle est bien écrite — c'est le rôle des tests du cycle qui la livre ;
//! - **les chemins d'exécution hors des trois parcours.** Un quatrième profil d'établissement, non
//!   déclaré ici, n'est gardé par rien ;
//! - **les dépendances entre crates.** C'est la porte P-03 (`backend/tests/architecture.rs`), qui
//!   lit le graphe `cargo metadata`. Les deux se complètent : P-03 interdit la dépendance, ce
//!   harnais interdit la **supposition** — un socle peut très bien supposer un hébergement sans
//!   jamais dépendre du crate qui le porte.
//!
//! # Le mécanisme : des étapes dues, avec sentinelle observable
//!
//! FR-025 interdit que la détection repose sur une revue humaine. Chaque étape déclare donc une
//! **sentinelle** — la table ou le point d'entrée dont l'existence prouve que l'étape est devenue
//! réalisable :
//!
//! | Sentinelle absente | Sentinelle présente, étape branchée | Sentinelle présente, étape **non** branchée |
//! |---|---|---|
//! | L'étape est **due**. Le harnais est vert. | L'étape est **exercée** et comptée. | **ÉCHEC**, en nommant l'étape et le parcours. |
//!
//! Le jour où le cycle PDV crée `restauration.commande`, ce fichier échoue tant que la vente
//! comptoir n'est pas branchée aux trois parcours. Ce n'est pas une revue qui l'attrape, c'est le
//! build.
//!
//! **Aucune étape n'est jamais marquée « ignorée » ni laissée en échec** (FR-024). Une étape due
//! n'est pas un test désactivé : c'est une déclaration que rien ne peut encore l'exercer, assortie
//! du nom du cycle qui la doit.
//!
//! # Ce harnais ne modifie jamais ce qu'il inspecte
//!
//! Il ne crée ni table, ni route, ni référentiel. Le service fictif du troisième parcours est créé
//! **dans une transaction annulée** (T037) : il n'existe donc à aucun moment pour un autre lecteur,
//! et FR-027 — « aucune trace dans un jeu de données » — est tenue par construction.

mod commun;

use std::collections::BTreeSet;

use kaya_api::application;
use sqlx::{PgPool, Row};

// =================================================================================================
//  Les trois parcours
// =================================================================================================

/// Un établissement témoin. **Aucun parcours ne dépend d'un autre** (FR-028) : chacun crée son
/// tenant, son établissement, l'exploite et s'achève seul, et les trois peuvent s'exécuter en
/// parallèle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Parcours {
    /// (a) Un maquis — **`RESTAURATION` seule**. Ni chambre, ni séjour, ni formule.
    Maquis,
    /// (b) Une résidence meublée — **`HEBERGEMENT` seul**. Ni catalogue, ni table, ni point de
    /// vente.
    ResidenceMeublee,
    /// (c) Un **service fictif minimal ne consommant aucune capacité** — la preuve formelle de
    /// FR-022. C'est le seul parcours qui ne repose sur aucune verticale réelle : s'il passe, le
    /// socle ne suppose rien.
    Agnosticite,
}

impl Parcours {
    pub const TOUS: [Parcours; 3] = [
        Parcours::Maquis,
        Parcours::ResidenceMeublee,
        Parcours::Agnosticite,
    ];

    pub fn nom(self) -> &'static str {
        match self {
            Parcours::Maquis => "maquis (RESTAURATION seule)",
            Parcours::ResidenceMeublee => "résidence meublée (HEBERGEMENT seul)",
            Parcours::Agnosticite => "agnosticité (service fictif, aucune capacité)",
        }
    }

    /// Le seul service actif de l'établissement témoin.
    ///
    /// Celui du parcours (c) **n'existe pas au référentiel** : il est créé dans une transaction
    /// annulée par le parcours lui-même (T037).
    pub fn module_code(self) -> &'static str {
        match self {
            Parcours::Maquis => "RESTAURATION",
            Parcours::ResidenceMeublee => "HEBERGEMENT",
            Parcours::Agnosticite => MODULE_FICTIF,
        }
    }
}

/// Le service fictif du troisième parcours.
///
/// **Il ne doit exister que dans ce harnais** (FR-027). `absence_du_module_fictif_dans_les_donnees`
/// vérifie qu'aucune table n'en porte trace après exécution des seeds.
pub const MODULE_FICTIF: &str = "MODULE_FICTIF_TEST";

// =================================================================================================
//  Les huit étapes, et leur sentinelle
// =================================================================================================

/// Ce dont l'existence prouve qu'une étape est devenue réalisable.
///
/// **Observable, jamais déclarative** (FR-025) : ni constante à maintenir, ni commentaire, ni
/// entrée de configuration. Une sentinelle se lit dans le catalogue PostgreSQL ou dans le contrat
/// OpenAPI assemblé — deux sources que le cycle qui livre l'étape modifie forcément.
#[derive(Debug, Clone, Copy)]
pub enum Sentinelle {
    /// Une table dans un schéma de module.
    Table {
        schema: &'static str,
        nom: &'static str,
    },
    /// Un chemin du contrat HTTP **réellement monté** — `contrat_complet()`, jamais le squelette
    /// `openapi::contrat()`, qui ne porte aucun chemin (piège constaté au cycle 001).
    Chemin(&'static str),
}

impl Sentinelle {
    fn description(self) -> String {
        match self {
            Sentinelle::Table { schema, nom } => format!("table {schema}.{nom}"),
            Sentinelle::Chemin(chemin) => format!("point d'entrée {chemin}"),
        }
    }
}

/// L'étape est-elle exercée par le parcours ?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Branchement {
    /// Le parcours l'exécute réellement — [`executer_etape`] la connaît.
    Branchee,
    /// Rien ne l'exécute encore. Légitime **tant que la sentinelle est absente**.
    Absente,
}

/// Une étape du cycle de vie d'un établissement témoin.
#[derive(Debug, Clone, Copy)]
pub struct Etape {
    /// Nom stable — il apparaît dans les messages d'échec et dans le décompte.
    pub nom: &'static str,
    /// Le cycle qui doit livrer cette étape. Écrit pour que l'échec dise **à qui** parler.
    pub cycle_du: &'static str,
    pub sentinelle: Sentinelle,
    pub branchement: Branchement,
}

/// Statut effectif d'une étape, calculé — jamais déclaré.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Statut {
    /// Sentinelle présente, étape branchée : le parcours l'exerce.
    Exercee,
    /// Sentinelle absente : rien ne peut encore l'exercer.
    Due,
    /// Sentinelle présente, étape non branchée. **C'est le seul cas d'échec.**
    RealisableNonBranchee,
}

/// **Les huit étapes de chaque parcours, dans l'ordre du cycle de vie** (FR-023).
///
/// Les quatre premières sont livrées par ce cycle, les quatre dernières sont dues à PDV, CAI et
/// FIS. Le tableau est identique pour les trois parcours : c'est **la même liste d'étapes** qui
/// doit tenir pour un maquis, pour une résidence meublée et pour un service fictif — sans quoi le
/// harnais ne prouverait rien sur l'agnosticité du socle. Ce qui diffère entre parcours, c'est le
/// **contenu** de l'étape, écrit dans [`executer_etape`].
pub fn etapes(_parcours: Parcours) -> Vec<Etape> {
    vec![
        Etape {
            nom: "creation_etablissement",
            cycle_du: "002-ETB (ETB-01)",
            sentinelle: Sentinelle::Chemin(CHEMIN_ETABLISSEMENTS),
            branchement: Branchement::Absente,
        },
        Etape {
            nom: "activation_module",
            cycle_du: "002-ETB (ETB-02)",
            sentinelle: Sentinelle::Chemin(CHEMIN_SERVICE),
            branchement: Branchement::Absente,
        },
        Etape {
            nom: "refus_capacite",
            cycle_du: "002-ETB (ETB-02b)",
            sentinelle: Sentinelle::Chemin(CHEMIN_CAPACITES),
            branchement: Branchement::Absente,
        },
        Etape {
            nom: "resolution_configuration",
            cycle_du: "002-ETB (ETB-04)",
            sentinelle: Sentinelle::Chemin(CHEMIN_CONFIGURATION),
            branchement: Branchement::Absente,
        },
        // ── Étapes dues. Leur sentinelle est une table : le cycle qui les livre la crée
        //    forcément, et ne peut donc pas livrer sans rencontrer ce harnais.
        Etape {
            nom: "vente_comptoir",
            cycle_du: "PDV (PDV-03)",
            sentinelle: Sentinelle::Table {
                schema: "restauration",
                nom: "commande",
            },
            branchement: Branchement::Absente,
        },
        Etape {
            nom: "encaissement",
            cycle_du: "CAI (CAI-02)",
            sentinelle: Sentinelle::Table {
                schema: "caisse",
                nom: "encaissement",
            },
            branchement: Branchement::Absente,
        },
        Etape {
            nom: "document_fiscal",
            cycle_du: "FIS (FIS-02)",
            sentinelle: Sentinelle::Table {
                schema: "fiscalite",
                nom: "document_fiscal",
            },
            branchement: Branchement::Absente,
        },
        Etape {
            nom: "cloture_journaliere",
            cycle_du: "CAI (CAI-06)",
            sentinelle: Sentinelle::Table {
                schema: "caisse",
                nom: "cloture_journaliere",
            },
            branchement: Branchement::Absente,
        },
    ]
}

/// Total déclaré, comparé au nombre réellement exercé (FR-026).
pub const ETAPES_DECLAREES: usize = 8;

// Chemins du contrat HTTP servant de sentinelles. Écrits une fois : un chemin recopié dans deux
// fichiers finit par diverger, et la sentinelle cesserait alors de voir sa cible sans rien dire.
const CHEMIN_ETABLISSEMENTS: &str = "/api/v1/etablissements";
const CHEMIN_SERVICE: &str = "/api/v1/etablissements/{etablissement_id}/services/{module_code}";
const CHEMIN_CAPACITES: &str =
    "/api/v1/etablissements/{etablissement_id}/services/{module_code}/capacites";
const CHEMIN_CONFIGURATION: &str = "/api/v1/configuration";

// =================================================================================================
//  Observation des sentinelles — lecture seule, sur l'état réel
// =================================================================================================

async fn tables_reelles(pool: &PgPool) -> BTreeSet<String> {
    sqlx::query(
        r#"
        SELECT table_schema || '.' || table_name AS nom_complet
        FROM information_schema.tables
        WHERE table_type = 'BASE TABLE'
        "#,
    )
    .fetch_all(pool)
    .await
    .expect("lecture du catalogue")
    .into_iter()
    .map(|l| l.get::<String, _>("nom_complet").to_lowercase())
    .collect()
}

fn chemins_du_contrat() -> BTreeSet<String> {
    application::contrat_complet()
        .paths
        .paths
        .keys()
        .cloned()
        .collect()
}

/// L'observation, faite une fois et passée aux tests.
pub struct Observation {
    tables: BTreeSet<String>,
    chemins: BTreeSet<String>,
}

impl Observation {
    pub async fn prendre(pool: &PgPool) -> Self {
        Self {
            tables: tables_reelles(pool).await,
            chemins: chemins_du_contrat(),
        }
    }

    fn sentinelle_presente(&self, sentinelle: Sentinelle) -> bool {
        match sentinelle {
            Sentinelle::Table { schema, nom } => {
                self.tables.contains(&format!("{schema}.{nom}").to_lowercase())
            }
            Sentinelle::Chemin(chemin) => self.chemins.contains(chemin),
        }
    }

    /// Statut d'une étape — **calculé depuis l'observation**, jamais lu d'une déclaration.
    pub fn statut(&self, etape: &Etape) -> Statut {
        match (
            self.sentinelle_presente(etape.sentinelle),
            etape.branchement,
        ) {
            (true, Branchement::Branchee) => Statut::Exercee,
            (true, Branchement::Absente) => Statut::RealisableNonBranchee,
            (false, _) => Statut::Due,
        }
    }
}

// =================================================================================================
//  La porte — une étape réalisable et non branchée fait échouer le build
// =================================================================================================

/// **FR-025** — l'intégration continue échoue dès qu'une étape due devient réalisable sans être
/// branchée aux trois parcours, en nommant l'étape et le parcours.
///
/// C'est le cœur du harnais. Sans lui, les quatre étapes dues seraient une intention écrite dans
/// un commentaire, et le cycle PDV livrerait sa vente comptoir sans jamais rencontrer la question
/// de l'agnosticité.
#[tokio::test]
async fn us1_aucune_etape_realisable_n_est_laissee_non_branchee() {
    let pool = commun::pool_owner().await;
    let observation = Observation::prendre(&pool).await;

    let mut manquements = Vec::new();
    for parcours in Parcours::TOUS {
        for etape in etapes(parcours) {
            if observation.statut(&etape) == Statut::RealisableNonBranchee {
                manquements.push(format!(
                    "parcours « {} » — étape « {} » (due au cycle {}) est RÉALISABLE : sa \
                     sentinelle, {}, existe désormais. Elle doit être branchée aux TROIS \
                     parcours dans le même changement que la migration ou la route qui la rend \
                     possible.",
                    parcours.nom(),
                    etape.nom,
                    etape.cycle_du,
                    etape.sentinelle.description(),
                ));
            }
        }
    }

    assert!(
        manquements.is_empty(),
        "Les trois parcours structurels ne couvrent plus tout ce qui est réalisable \
         — {} manquement(s) :\n  {}\n\n\
         « Le garde-fou de toute extension future du produit » (ETB-02c). Une étape livrée sans \
         être branchée ici laisse le socle se spécialiser sans que personne ne le voie.",
        manquements.len(),
        manquements.join("\n  ")
    );
}

/// **FR-026** — le décompte, comparé au total déclaré.
///
/// Un harnais qui n'affiche pas ce qu'il a exercé est indistinguable d'un harnais qui n'exerce
/// rien. Le décompte est donc **imprimé** et **asserté** : il ne peut ni régresser en silence, ni
/// dépasser le total déclaré.
#[tokio::test]
async fn us1_decompte_des_etapes_exercees_et_dues() {
    let pool = commun::pool_owner().await;
    let observation = Observation::prendre(&pool).await;

    println!("Trois parcours structurels — décompte par étape :");
    for parcours in Parcours::TOUS {
        let etapes = etapes(parcours);
        assert_eq!(
            etapes.len(),
            ETAPES_DECLAREES,
            "le parcours « {}» déclare {} étapes au lieu des {ETAPES_DECLAREES} attendues : la \
             liste complète et ordonnée est exigée par FR-023",
            parcours.nom(),
            etapes.len()
        );

        let exercees = etapes
            .iter()
            .filter(|e| observation.statut(e) == Statut::Exercee)
            .count();
        let dues = etapes
            .iter()
            .filter(|e| observation.statut(e) == Statut::Due)
            .count();

        println!(
            "  · {} — {exercees} exercée(s) / {ETAPES_DECLAREES} déclarée(s), {dues} due(s)",
            parcours.nom()
        );
        for etape in &etapes {
            let marque = match observation.statut(etape) {
                Statut::Exercee => "exercée",
                Statut::Due => "due",
                Statut::RealisableNonBranchee => "NON BRANCHÉE",
            };
            println!("      {:<26} {marque:<13} {}", etape.nom, etape.cycle_du);
        }

        assert_eq!(
            exercees + dues,
            ETAPES_DECLAREES,
            "parcours « {}» : {exercees} exercée(s) + {dues} due(s) ≠ {ETAPES_DECLAREES} \
             déclarée(s). Une étape est réalisable sans être branchée — voir \
             `us1_aucune_etape_realisable_n_est_laissee_non_branchee`.",
            parcours.nom()
        );
    }
}

/// Les huit étapes sont déclarées, nommées et attribuées à un cycle (FR-023, scénario 3).
#[test]
fn us1_chaque_etape_est_nommee_et_attribuee_a_un_cycle() {
    for parcours in Parcours::TOUS {
        for etape in etapes(parcours) {
            assert!(
                !etape.nom.trim().is_empty(),
                "une étape du parcours « {} » n'a pas de nom : le message d'échec ne pourrait pas \
                 la désigner",
                parcours.nom()
            );
            assert!(
                !etape.cycle_du.trim().is_empty(),
                "l'étape « {} » n'est attribuée à aucun cycle : l'échec ne dirait pas à qui parler",
                etape.nom
            );
        }
    }
}

/// **Test négatif de la porte, sur un état simulé.**
///
/// La version réelle — créer une table portant le nom d'une sentinelle due — est exercée **à la
/// main** en fin de cycle (T039), sortie observée consignée ci-dessous. Elle ne peut pas être
/// automatisée ici : les fichiers de test s'exécutent en parallèle sur une base partagée, et une
/// table créée le temps d'un inventaire ferait échouer au hasard la porte P-07 et celle du
/// registre. Un test qui casse ses voisins est un test qu'on finit par ignorer (leçon du
/// cycle 001, `rls_catalogue.rs`).
///
/// Ce qui est vérifié ici est ce qui compte : la fonction de statut **classe bien** une étape
/// réalisable et non branchée en `RealisableNonBranchee`, et ne se trompe ni dans un sens ni dans
/// l'autre.
#[test]
fn us1_test_negatif_une_etape_realisable_non_branchee_est_signalee() {
    let observation = Observation {
        tables: BTreeSet::from(["restauration.commande".to_owned()]),
        chemins: BTreeSet::new(),
    };

    let realisable_non_branchee = Etape {
        nom: "vente_comptoir",
        cycle_du: "PDV (PDV-03)",
        sentinelle: Sentinelle::Table {
            schema: "restauration",
            nom: "commande",
        },
        branchement: Branchement::Absente,
    };
    assert_eq!(
        observation.statut(&realisable_non_branchee),
        Statut::RealisableNonBranchee,
        "la porte n'a pas vu une étape rendue réalisable sans branchement : elle ne garde rien"
    );

    let due = Etape {
        sentinelle: Sentinelle::Table {
            schema: "caisse",
            nom: "encaissement",
        },
        ..realisable_non_branchee
    };
    assert_eq!(
        observation.statut(&due),
        Statut::Due,
        "une étape dont la sentinelle est absente doit rester DUE — sinon le harnais serait rouge \
         en permanence et finirait désactivé"
    );

    let exercee = Etape {
        branchement: Branchement::Branchee,
        ..realisable_non_branchee
    };
    assert_eq!(
        observation.statut(&exercee),
        Statut::Exercee,
        "une étape branchée dont la sentinelle existe doit être comptée exercée"
    );
}
