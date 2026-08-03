//! **Porte P-13** — aucune opération de classe B, C ou D n'est atteignable hors ligne.
//!
//! # Les DIX-SEPT opérations du cycle sont inspectées, et les DEUX de classe A sont NOMMÉES
//!
//! ⚠️ **Un test qui n'inspecterait que les opérations refusées ne prouverait pas qu'il les a
//! toutes vues.** C'est le mode d'échec que la section « Couverture des portes » de la
//! constitution désigne : une porte dont la cible est incomplète passe au vert en regardant moins
//! que ce qu'elle annonce.
//!
//! Les deux opérations de **classe A** — ajout et retrait d'accompagnant — sont donc **déclarées
//! comme telles** et vérifiées **positivement** : elles portent bien une exigence
//! d'authentification (aucune opération du produit n'est publique), mais leur classe autorise la
//! mise en file, et ce fichier le dit plutôt que de les omettre.
//!
//! # Ce que « inatteignable hors ligne » veut dire côté SERVEUR
//!
//! Il n'existe pas de file d'attente côté serveur : ce qu'on y vérifie est qu'une opération
//! **exige un jeton** — donc une session, donc une connexion, donc le réseau. Le versant *écran*
//! — l'annonce **avant la saisie**, jamais après un échec — est vérifié en direct par
//! `tests-e2e/hors-ligne.spec.ts`. **Les deux versants sont distincts et aucun ne remplace
//! l'autre.**

mod commun;

use actix_web::http::Method;

/// Les dix-sept opérations du cycle, avec leur **classe déclarée**.
///
/// La classe vient de `docs/registre-classes-offline.md` §7.3, jamais d'une intuition. Les
/// lectures ne portent pas de classe : le registre classe des **opérations d'écriture**, et une
/// lecture en cache est de classe A avec fraîcheur affichée (règle de portée générale, §1.0.2).
struct Operation {
    numero: u8,
    nom: &'static str,
    methode: Method,
    chemin: &'static str,
    /// `Some("A")` pour les deux opérations **atteignables** hors ligne, `None` pour une lecture.
    classe: Option<&'static str>,
}

const OPERATIONS: &[Operation] = &[
    // ── Fiches clients — SEJ-01 ────────────────────────────────────────────────────────────
    Operation {
        numero: 1,
        nom: "rechercher une fiche client",
        methode: Method::GET,
        chemin: "/api/v1/clients",
        classe: None,
    },
    Operation {
        numero: 2,
        nom: "créer une fiche client",
        methode: Method::POST,
        chemin: "/api/v1/clients",
        classe: Some("C"),
    },
    Operation {
        numero: 3,
        nom: "lire une fiche client",
        methode: Method::GET,
        chemin: "/api/v1/clients/{client_id}",
        classe: None,
    },
    Operation {
        numero: 4,
        nom: "modifier une fiche client",
        methode: Method::PATCH,
        chemin: "/api/v1/clients/{client_id}",
        classe: Some("C"),
    },
    Operation {
        numero: 5,
        nom: "historique des séjours d'un client",
        methode: Method::GET,
        chemin: "/api/v1/clients/{client_id}/sejours",
        classe: None,
    },
    Operation {
        numero: 6,
        nom: "enregistrer une préférence",
        methode: Method::POST,
        chemin: "/api/v1/clients/{client_id}/preferences",
        classe: Some("A"),
    },
    // ── Séjours — SEJ-02, SEJ-04 ───────────────────────────────────────────────────────────
    Operation {
        numero: 7,
        nom: "ouvrir un séjour",
        methode: Method::POST,
        chemin: "/api/v1/etablissements/{etablissement_id}/sejours",
        classe: Some("B"),
    },
    Operation {
        numero: 8,
        nom: "lister les séjours",
        methode: Method::GET,
        chemin: "/api/v1/etablissements/{etablissement_id}/sejours",
        classe: None,
    },
    Operation {
        numero: 9,
        nom: "lire un séjour",
        methode: Method::GET,
        chemin: "/api/v1/etablissements/{etablissement_id}/sejours/{sejour_id}",
        classe: None,
    },
    Operation {
        numero: 10,
        nom: "rattacher un client à un séjour",
        methode: Method::POST,
        chemin: "/api/v1/etablissements/{etablissement_id}/sejours/{sejour_id}/client",
        classe: Some("B"),
    },
    // ★ Les DEUX opérations de classe A — nommées, jamais omises.
    Operation {
        numero: 11,
        nom: "ajouter un accompagnant",
        methode: Method::POST,
        chemin: "/api/v1/etablissements/{etablissement_id}/sejours/{sejour_id}/accompagnants",
        classe: Some("A"),
    },
    Operation {
        numero: 12,
        nom: "retirer un accompagnant",
        methode: Method::DELETE,
        chemin:
            "/api/v1/etablissements/{etablissement_id}/sejours/{sejour_id}/accompagnants/{accompagnant_id}",
        classe: Some("A"),
    },
    Operation {
        numero: 15,
        nom: "clore un séjour",
        methode: Method::POST,
        chemin: "/api/v1/etablissements/{etablissement_id}/sejours/{sejour_id}/depart",
        classe: Some("B"),
    },
    Operation {
        numero: 16,
        nom: "lire la fiche de police",
        methode: Method::GET,
        chemin: "/api/v1/etablissements/{etablissement_id}/sejours/{sejour_id}/fiche-police",
        classe: None,
    },
    // ── État des unités — HEB, opération 17 ────────────────────────────────────────────────
    Operation {
        numero: 17,
        nom: "état des unités",
        methode: Method::GET,
        chemin: "/api/v1/etablissements/{etablissement_id}/hebergement/etat-des-unites",
        classe: None,
    },
];

/// Le décompte attendu — **figé**, et c'est ce qui rend la porte opposable.
///
/// ⚠️ **Quinze, pas dix-sept.** Les opérations 13 (prolongation) et 14 (changement d'unité)
/// arrivent avec leurs stories — US5 et US7. Le contrat du cycle en annonce dix-sept ; **quinze
/// sont servies à ce point du cycle**, et l'écart est écrit ici plutôt que laissé à la lecture
/// d'un décompte qui ne correspondrait pas.
///
/// Une constante qui suivrait silencieusement la longueur du tableau ne prouverait rien : elle
/// vaudrait toujours ce que le tableau vaut, y compris amputé.
const OPERATIONS_ATTENDUES: usize = 15;

// =================================================================================================
//  INSTANCIATION — les opérations d'écriture de classe B et C
// =================================================================================================
//
// La macro engendre les **deux** versants : aucune n'est atteignable sans jeton, **et** chacune
// aboutit avec. Sans le versant positif, une opération retirée du produit satisferait encore la
// moitié négative — et la porte resterait verte sur un produit amputé.
tester_classe_bcd!(
    ecritures_de_sejour,
    classe = "B",
    operations = &[
        (
            "ouvrir un séjour",
            actix_web::http::Method::POST,
            "/api/v1/etablissements/{etablissement_id}/sejours",
        ),
        (
            "rattacher un client à un séjour",
            actix_web::http::Method::POST,
            "/api/v1/etablissements/{etablissement_id}/sejours/{sejour_id}/client",
        ),
        (
            "clore un séjour",
            actix_web::http::Method::POST,
            "/api/v1/etablissements/{etablissement_id}/sejours/{sejour_id}/depart",
        ),
    ],
);

// =================================================================================================
//  Versant NÉGATIF — les quinze exigent un jeton, et il n'y en a pas une seizième
// =================================================================================================

/// **Chacune des quinze porte une exigence d'authentification, et aucune autre n'existe.**
///
/// Les deux moitiés comptent. La première refuse une opération publique ; la seconde refuse une
/// opération qui aurait **échappé à la liste** — auquel cas la porte inspecterait quinze chemins
/// en croyant couvrir le cycle.
#[test]
fn les_quinze_operations_du_cycle_exigent_un_jeton() {
    let contrat = kaya_api::application::contrat_complet();
    let mut inspectees = 0usize;
    let mut sans_securite = Vec::new();

    for operation in OPERATIONS {
        let item = contrat.paths.paths.get(operation.chemin).unwrap_or_else(|| {
            panic!(
                "opération {} — « {} » ({}) n'est pas au contrat : la liste a dérivé du produit, \
                 ou l'opération a été retirée sans que ce test suive.",
                operation.numero, operation.nom, operation.chemin
            )
        });

        let op = match operation.methode {
            Method::GET => item.get.as_ref(),
            Method::POST => item.post.as_ref(),
            Method::PUT => item.put.as_ref(),
            Method::PATCH => item.patch.as_ref(),
            Method::DELETE => item.delete.as_ref(),
            _ => None,
        }
        .unwrap_or_else(|| {
            panic!(
                "opération {} — le contrat ne sert pas {} sur {}",
                operation.numero, operation.methode, operation.chemin
            )
        });

        inspectees += 1;

        let gardee = op
            .security
            .as_ref()
            .is_some_and(|exigences| !exigences.is_empty());
        if !gardee {
            sans_securite.push(format!("{} — {}", operation.numero, operation.nom));
        }
    }

    assert_eq!(
        inspectees, OPERATIONS_ATTENDUES,
        "la porte a inspecté {inspectees} opération(s) pour {OPERATIONS_ATTENDUES} attendues. \
         Une porte dont la cible RÉTRÉCIT passe au vert sans rien vérifier."
    );

    assert!(
        sans_securite.is_empty(),
        "ces opérations sont PUBLIQUES — donc atteignables sans session, donc sans réseau, donc \
         depuis un terminal qui vide une file locale :\n  {}",
        sans_securite.join("\n  ")
    );
}

// =================================================================================================
//  ★ Les DEUX opérations de classe A — nommées, jamais omises
// =================================================================================================

/// ★ **Les deux écritures de séjour atteignables hors ligne sont NOMMÉES.**
///
/// ⚠️ **C'est ce test qui empêche la porte de mentir.** Un fichier qui n'aurait listé que les
/// opérations refusées serait vert, complet en apparence, et **muet sur ce qu'il n'a pas
/// regardé** : les deux opérations dont la classe autorise la mise en file. Le lecteur du journal
/// de CI en conclurait que le cycle n'en a aucune.
///
/// Elles exigent un jeton comme toutes les autres — aucune opération du produit n'est publique —
/// mais leur **classe** les rend rejouables au retour du réseau, avec les deux propriétés que
/// `tester_classe_a!` vérifie : rejeu inoffensif et désordre commutatif.
#[test]
fn les_deux_operations_de_classe_a_sont_nommees_et_non_omises() {
    let de_classe_a: Vec<&Operation> = OPERATIONS
        .iter()
        .filter(|o| o.classe == Some("A"))
        .collect();

    assert_eq!(
        de_classe_a.len(),
        3,
        "trois opérations de classe A sont attendues au cycle 006 — l'ajout et le retrait \
         d'accompagnant (SEJ-02) et l'enregistrement d'une préférence (SEJ-01). Trouvées : {:?}",
        de_classe_a.iter().map(|o| o.nom).collect::<Vec<_>>()
    );

    let noms: Vec<&str> = de_classe_a.iter().map(|o| o.nom).collect();
    for attendue in [
        "ajouter un accompagnant",
        "retirer un accompagnant",
        "enregistrer une préférence",
    ] {
        assert!(
            noms.contains(&attendue),
            "« {attendue} » est de classe A et doit être NOMMÉE dans ce fichier. Un test qui \
             n'inspecterait que les opérations refusées ne prouverait pas qu'il les a toutes vues."
        );
    }
}

/// **Les écritures de classe B et C sont bien refusées hors ligne, et les A ne le sont pas.**
///
/// Le contrôle porte sur la **cohérence entre le registre et le contrat** : une opération que le
/// registre classe en B doit être servie par un chemin d'écriture, et une opération de classe A
/// aussi. Ce qui les distingue n'est pas leur garde — identique — mais ce que l'**écran** en fait,
/// et c'est `tests-e2e/hors-ligne.spec.ts` qui l'éprouve.
#[test]
fn le_registre_et_le_contrat_s_accordent_sur_les_classes() {
    let ecritures: Vec<&Operation> = OPERATIONS.iter().filter(|o| o.classe.is_some()).collect();

    assert!(
        !ecritures.is_empty(),
        "aucune opération d'écriture déclarée : une porte dont la cible est vide passe toujours"
    );

    for operation in &ecritures {
        assert!(
            matches!(operation.methode, Method::POST | Method::PATCH | Method::DELETE),
            "l'opération {} « {} » porte une classe hors-ligne mais emploie {} : une classe se \
             déclare sur une ÉCRITURE, et une lecture en cache relève de la règle de portée \
             générale du registre (§1.0.2)",
            operation.numero,
            operation.nom,
            operation.methode
        );
    }

    let en_b_ou_c = ecritures
        .iter()
        .filter(|o| o.classe != Some("A"))
        .count();
    assert_eq!(
        en_b_ou_c, 5,
        "cinq écritures de classe B ou C sont attendues — créer et modifier une fiche (C), \
         ouvrir un séjour, rattacher un client et clore (B). Trouvées : {en_b_ou_c}"
    );
}
