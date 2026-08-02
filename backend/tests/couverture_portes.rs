//! **Recollement des portes à décompte — P-05, P-07, P-08, P-01b, et la taxonomie d'audit.**
//!
//! # Pourquoi ce fichier existe
//!
//! Ces portes portent chacune sur un ensemble dont la taille est connue. Leur extension s'est faite
//! phase par phase, ce qui est la manière normale de les faire grandir avec le code — **et la
//! manière normale de laisser un trou.**
//!
//! Chaque phase a étendu la porte à ce qu'elle livrait. Aucune n'était responsable de vérifier que
//! l'ensemble était couvert, et c'est exactement là que se perd un type d'événement : celui qu'une
//! phase a introduit sans que la tâche d'extension de la phase suivante le reprenne.
//!
//! Ce fichier compare, pour chaque porte, **le nombre de cibles réellement inspectées au total
//! déclaré**, et échoue sur tout écart.
//!
//! # Les cinq décomptes, à la clôture du cycle 003
//!
//! | Porte | Ensemble | Total |
//! |---|---|---|
//! | P-05 | types d'événements outbox déclarés au modèle de données | **22** (13 + 9) |
//! | P-07 | tables créées par les cycles 002 et 003 | **20** (10 + 10) |
//! | — | tables des quatre schémas applicatifs, tous cycles | **26** |
//! | P-08 | opérations HTTP servies par le contrat | **43** |
//! | P-01b | `operationId` du contrat, tous distincts | **43** |
//! | — | familles de la taxonomie d'audit | **10**, dont 2 branchées |
//!
//! **Trois de ces nombres démentent le plan du cycle 003**, qui annonçait 21 types, 40 opérations
//! et ne comptait que le schéma `etablissements`. Les écarts sont réels, chacun est justifié à
//! l'endroit où il se constate, et **aucun n'a été résorbé en ajustant un chiffre** : le décompte
//! se relit du catalogue système et du contrat, jamais d'une constante recopiée.
//!
//! # Ce qu'il inspecte, et ce qu'il n'inspecte pas
//!
//! *Exigence 2 du § « Couverture des portes » de la constitution.*
//!
//! **Inspecté** — la **couverture** : chaque type d'événement du modèle a-t-il un test ? chaque
//! table créée est-elle inventoriée et isolée ? chaque chemin servi a-t-il un régime d'isolation
//! déclaré ? chaque opération a-t-elle un `operationId` distinct ? chaque famille d'audit déclarée
//! branchée est-elle exercée par un test ?
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

use std::collections::{BTreeMap, BTreeSet};

use kaya_api::application;
use sqlx::Row;

/// Les opérations d'un chemin, avec leur verbe.
///
/// `PathItem` n'expose pas de carte d'opérations en utoipa 5.5 : chaque verbe est un champ
/// `Option<Operation>` distinct. Les énumérer un par un est verbeux mais **exact** — et un verbe
/// ajouté à la structure par une montée de version ferait échouer la compilation ici plutôt que de
/// disparaître silencieusement du décompte.
fn operations_de(
    item: &utoipa::openapi::path::PathItem,
) -> Vec<(&'static str, &utoipa::openapi::path::Operation)> {
    [
        ("GET", item.get.as_ref()),
        ("PUT", item.put.as_ref()),
        ("POST", item.post.as_ref()),
        ("DELETE", item.delete.as_ref()),
        ("OPTIONS", item.options.as_ref()),
        ("HEAD", item.head.as_ref()),
        ("PATCH", item.patch.as_ref()),
        ("TRACE", item.trace.as_ref()),
    ]
    .into_iter()
    .filter_map(|(verbe, operation)| operation.map(|o| (verbe, o)))
    .collect()
}

/// Compte les opérations d'un chemin.
fn compter_operations(item: &utoipa::openapi::path::PathItem) -> usize {
    operations_de(item).len()
}

// =================================================================================================
//  P-05 — les vingt-deux types d'événements
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
    // ── Cycle 003 (CPT) — neuf types, et non dix. L'écart est TRANCHÉ ici ───────────────────
    //
    // `data-model.md` § Événements en déclare **dix**, dont `compte.modifie` — « modification
    // d'identifiant ». **Aucune opération du contrat ne la produit** : les §10 à 16 exposent
    // créer, lister, lire, changer l'état, changer le mot de passe, attribuer et retirer un rôle.
    // Pas de modification d'identifiant.
    //
    // Ce n'est pas un oubli d'implémentation, c'est un écart entre deux documents de conception
    // écrits en parallèle — le modèle de données a prévu un événement pour une opération que le
    // contrat n'a pas retenue. **Le déclarer ici sans émetteur ferait échouer la porte à chaque
    // exécution**, et l'inventer côté serveur produirait une opération que personne n'a
    // spécifiée : le principe X l'interdit dans les deux sens.
    //
    // La ligne reste au modèle de données comme **provision nommée** — le jour où changer un
    // numéro de téléphone de connexion deviendra une opération, son type est déjà décidé.
    // Total du produit : **22 types**, 13 + 9.
    "personne.creee",
    "personne.modifiee",
    "compte.cree",
    "compte.desactive",
    "compte.reactive",
    "compte.mot_de_passe_change",
    "role.attribue",
    "role.retire",
    "session.revoquee",
    // ── Cycle 004 (HEB) — cinq types, RELUS DU CODE ────────────────────────────────────────
    //
    // **Le décompte n'est pas le sujet ; la liste des fichiers balayés l'est.** Avant ce
    // recollement, `p05_aucun_type_emis_par_le_code_n_est_absent_de_la_liste` ne lisait aucun
    // fichier de `verticales/` : les cinq types ci-dessous étaient émis en production et
    // **invisibles à la porte**, qui restait verte. Ajouter les types sans ajouter les fichiers
    // aurait rendu le total juste et la porte toujours aveugle — c'est le trou trouvé sur
    // `comptes` au cycle 003, reformé un cran plus loin.
    //
    // Total du produit : **27 types**, 13 + 9 + 5.
    "heb.categorie.tarif_modifie",
    "heb.formule.creee",
    "heb.formule.modifiee",
    "heb.occupation.attribuee",
    "heb.occupation.liberee",
];

/// Les types déclarés au modèle de données **sans émetteur**, nommés un par un.
///
/// Une liste vide serait le cas normal ; une liste non vide doit se justifier **ici**, pas dans un
/// commentaire perdu. Voir la note de `TYPES_EVENEMENTS` ci-dessus.
const TYPES_SANS_EMETTEUR: &[&str] = &["compte.modifie"];

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
        // ── Cycle 003 ──────────────────────────────────────────────────────────────────────
        "crates/socle/comptes/src/personne/service.rs",
        "crates/socle/comptes/src/compte/service.rs",
        "crates/socle/comptes/src/roles/service.rs",
        "crates/socle/comptes/src/authentification/service.rs",
        // ── Cycle 004 — LA VERTICALE, qui échappait entièrement au balayage ─────────────────
        "crates/verticales/hebergement/src/referentiel/service.rs",
        "crates/verticales/hebergement/src/occupation/service.rs",
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
//  P-07 — les vingt tables créées, lues du CATALOGUE SYSTÈME, sur DEUX schémas
// =================================================================================================

/// Les tables **créées** par les cycles 002 et 003, `(schéma, table)`.
///
/// # Le schéma faisait partie du décompte sans que personne l'écrive
///
/// La liste ne portait que des noms de table, et la requête qui la vérifiait fixait
/// `nspname = 'etablissements'`. Tant que le produit n'avait qu'un schéma applicatif métier, les
/// deux disaient la même chose. **Le cycle 003 a créé dix tables dans `comptes`** : elles seraient
/// restées invisibles à cette porte, et l'ajouter ne se voit pas — la porte serait restée verte en
/// inspectant la moitié de ce qu'elle annonçait.
///
/// Le même trou avait déjà été trouvé au cycle 002, où le décompte de P-07 ne couvrait que
/// 4 tables sur 10 (constitution, § Couverture des portes). Il s'est reformé un cran plus haut :
/// non plus sur les tables d'un schéma, mais sur les schémas eux-mêmes. **Porter le schéma dans la
/// donnée** est ce qui empêche la troisième occurrence.
///
/// À ne pas confondre avec les entités du registre des classes hors-ligne : `etablissement` y
/// figure aussi, mais elle est *enrichie*, pas créée.
const TABLES_CREEES: &[(&str, &str)] = &[
    // ── Cycle 002 (ETB) — dix tables ────────────────────────────────────────────────────────
    ("etablissements", "module_activite"),
    ("etablissements", "capacite"),
    ("etablissements", "profil_stock"),
    ("etablissements", "parametre_catalogue"),
    ("etablissements", "etablissement_module"),
    ("etablissements", "module_capacite"),
    ("etablissements", "point_de_vente"),
    ("etablissements", "table_pdv"),
    ("etablissements", "parametre_configuration"),
    ("etablissements", "branding"),
    // ── Cycle 003 (CPT) — dix tables, dont deux de provision ────────────────────────────────
    //
    // `employe` et `appareil_enrole` sont des **provisions sans logique** (§14 du cadrage) :
    // la table existe, isolée comme les autres, aucun chemin de code ne l'écrit. C'est
    // `provisions_sans_logique.rs` qui garde cette seconde propriété. Les omettre ici les
    // sortirait du décompte d'isolation, qui, lui, les concerne autant que les autres.
    ("comptes", "personne"),
    ("comptes", "methode_authentification"),
    ("comptes", "compte"),
    ("comptes", "role"),
    ("comptes", "permission"),
    ("comptes", "role_permission"),
    ("comptes", "compte_role"),
    ("comptes", "journal_audit"),
    ("comptes", "employe"),
    ("comptes", "appareil_enrole"),
    // ── Cycle 004 (HEB) — huit tables, dont une de provision ────────────────────────────────
    //
    // Six pour le référentiel (`0024`), `occupation` (`0025`), et `prestation_incluse` (`0026`).
    // La provision figure ici pour la même raison qu'`employe` et `appareil_enrole` : l'isolation
    // la concerne autant que les autres, et l'omettre la sortirait du décompte.
    ("hebergement", "categorie"),
    ("hebergement", "temps_remise_en_etat"),
    ("hebergement", "unite"),
    ("hebergement", "formule"),
    ("hebergement", "bareme_palier"),
    ("hebergement", "plage_demi_journee"),
    ("hebergement", "occupation"),
    ("hebergement", "prestation_incluse"),
];

/// **P-07 — les vingt tables existent, et toutes sont inspectées par la porte.**
///
/// Le décompte est **lu du catalogue système**, jamais d'un nombre écrit à la main : une table
/// renommée ou supprimée doit se voir ici, pas dans six mois.
#[tokio::test]
async fn p07_les_tables_creees_sont_toutes_inspectees() {
    let pool = commun::pool_owner().await;

    // Les schémas viennent de la liste elle-même : en ajouter un se fait en ajoutant une table,
    // et il n'y a aucun second endroit à mettre à jour — c'est ce qui a manqué au cycle 002.
    let schemas: BTreeSet<&str> = TABLES_CREEES.iter().map(|(schema, _)| *schema).collect();
    let schemas: Vec<String> = schemas.into_iter().map(str::to_owned).collect();

    let reelles: BTreeSet<(String, String)> = sqlx::query(
        r#"
        SELECT n.nspname AS schema, c.relname AS nom
        FROM pg_class c
        JOIN pg_namespace n ON n.oid = c.relnamespace
        WHERE c.relkind = 'r' AND n.nspname = ANY($1)
        "#,
    )
    .bind(&schemas)
    .fetch_all(&pool)
    .await
    .expect("lecture du catalogue")
    .into_iter()
    .map(|l| (l.get::<String, _>("schema"), l.get::<String, _>("nom")))
    .collect();

    assert!(
        !reelles.is_empty(),
        "aucune table trouvée dans {schemas:?} — la porte n'a rien inspecté. Base non migrée ?"
    );

    let mut manquantes = Vec::new();
    for (schema, table) in TABLES_CREEES {
        if !reelles.contains(&((*schema).to_owned(), (*table).to_owned())) {
            manquantes.push(format!("{schema}.{table}"));
        }
    }

    assert!(
        manquantes.is_empty(),
        "P-07 — {} table(s) déclarée(s) par un cycle et ABSENTE(s) de la base :\n  {}\n\n\
         Soit une migration a été retirée, soit une table a été renommée sans mettre ce décompte \
         à jour. Dans les deux cas, la porte inspectait moins que ce qu'elle annonçait.",
        manquantes.len(),
        manquantes.join("\n  ")
    );

    // Et chacune est bien **isolée** — c'est ce que la porte P-07 garantit, revérifié ici APRÈS
    // la dernière migration du cycle. `rls_catalogue.rs` s'exécute aussi, mais rien ne garantit
    // qu'il ait tourné après `0020`.
    let mut sans_isolation = Vec::new();
    for (schema, table) in TABLES_CREEES {
        let ligne = sqlx::query(
            r#"
            SELECT c.relrowsecurity AS activee,
                   c.relforcerowsecurity AS forcee,
                   (SELECT COUNT(*) FROM pg_policies p
                     WHERE p.schemaname = n.nspname AND p.tablename = c.relname) AS politiques
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE c.relkind = 'r' AND n.nspname = $1 AND c.relname = $2
            "#,
        )
        .bind(schema)
        .bind(table)
        .fetch_one(&pool)
        .await
        .expect("lecture de l'état d'isolation");

        let activee: bool = ligne.get("activee");
        let forcee: bool = ligne.get("forcee");
        let politiques: i64 = ligne.get("politiques");

        if !activee || !forcee || politiques == 0 {
            sans_isolation.push(format!(
                "{schema}.{table} — ENABLE={activee}, FORCE={forcee}, politiques={politiques}"
            ));
        }
    }

    assert!(
        sans_isolation.is_empty(),
        "P-07 — {} table(s) créée(s) par un cycle sans isolation complète :\n  {}",
        sans_isolation.len(),
        sans_isolation.join("\n  ")
    );

    println!(
        "P-07 — {}/{} tables créées sur {} schéma(s), inspectées et isolées.",
        TABLES_CREEES.len(),
        TABLES_CREEES.len(),
        schemas.len()
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
        // Sept opérations de comptes (§10-16) et **deux** référentiels (§17-18). Le contrat en
        // numérote neuf, et neuf sont servies — `compte_lister` et `compte_creer` partagent le
        // chemin `/api/v1/comptes`, ce qui fait six chemins pour sept opérations.
        ("cycle 003 — comptes et rôles (CPT-02, contrat §10-18)", 9),
        // Une seule, et en lecture. Aucun point d'entrée d'écriture d'audit — research R-17.
        ("cycle 003 — registre des actions (CPT-04, contrat §19)", 1),
        // ── Cycle 004 (HEB) — treize opérations, la première verticale ───────────────────────
        //
        // Neuf pour le référentiel (catégories, unités, formules — lire, créer, modifier), une
        // pour la disponibilité, deux pour l'occupation (attribuer, libérer), une pour le tarif.
        // La ventilation suit les trois fichiers de routes, ce qui rend un écart localisable.
        ("cycle 004 — référentiel d'hébergement (HEB-01/03/04/05, opérations 1-9)", 9),
        ("cycle 004 — disponibilité et occupation (HEB-02, opérations 10-12)", 3),
        ("cycle 004 — tarification du passage (HEB-04, opération 13)", 1),
    ];
    let operations_attendues: usize = LOTS.iter().map(|(_, n)| n).sum();

    assert_eq!(
        operations,
        operations_attendues,
        "P-08 — {operations} opération(s) servie(s) au lieu des {operations_attendues} attendues.\n\
         Ventilation déclarée :\n{}\n\
         Le total à la clôture du cycle 004 est de **56** — 43 après le cycle 003, plus les \
         treize opérations de l'hébergement. Le 43 lui-même n'était pas le 40 du plan : celui-ci \
         comptait des chemins là où la porte compte des opérations (`/api/v1/comptes` en sert \
         deux, `/api/v1/session` aussi). Ces écarts sont des défauts de comptage des plans, \
         constatés au recollement et laissés tels quels : c'est la ventilation ci-dessus qui fait \
         foi, parce qu'elle oblige à dire de quel lot vient un écart au lieu de corriger un \
         total.",
        LOTS.iter()
            .map(|(nom, n)| format!("  {n:>3} — {nom}"))
            .collect::<Vec<_>>()
            .join("\n")
    );

    println!("P-08 — {operations} opérations servies, toutes déclarées.");
}

// =================================================================================================
//  P-01b — l'unicité des operationId, qui n'était vérifiée NULLE PART
// =================================================================================================

/// **P-01b — tout `operationId` du contrat est présent, et tous sont distincts.**
///
/// # Le trou, et pourquoi aucune autre porte ne le voit
///
/// La constitution porte P-01b depuis son amendement du cycle 002 : « deux opérations homonymes
/// produisent un client TypeScript invalide, **que P-01 ne détecte pas puisqu'elle ne compare que
/// le généré au commité** ». Un client invalide régénéré de la même façon deux fois de suite reste
/// identique à lui-même — le déterminisme d'octet que vérifie `generer-client.sh` est vrai d'un
/// contrat cassé comme d'un contrat sain.
///
/// La porte n'avait pourtant **aucune implémentation** : ni script sous `scripts/ci/`, ni test.
/// Le cycle 003 ajoute 19 `operationId`, ce que le plan désignait comme « risque réel » — c'est le
/// moment où l'absence coûte quelque chose.
///
/// # Périmètre inspecté
///
/// *Exigence 1 du § « Couverture des portes ».*
///
/// **Inspecté** — toutes les opérations de `application::contrat_complet()`, c'est-à-dire la
/// **source** dont le contrat et le client dérivent tous deux, pas le `openapi.json` d'un artefact
/// de build qui pourrait dater.
///
/// **Non inspecté** — la *qualité* du nom. Qu'un `operationId` soit unique ne dit pas qu'il soit
/// lisible, ni qu'il suive la convention `ressource_verbe`. Une porte sur le style des noms se
/// discuterait ; celle-ci ne porte que sur ce qui casse le client.
#[test]
fn p01b_les_operation_id_du_contrat_sont_tous_presents_et_distincts() {
    let contrat = application::contrat_complet();

    let mut par_identifiant: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut sans_identifiant = Vec::new();
    let mut operations = 0usize;

    for (chemin, item) in &contrat.paths.paths {
        for (verbe, operation) in operations_de(item) {
            operations += 1;
            let ou = format!("{verbe} {chemin}");
            match operation.operation_id.as_deref().map(str::trim) {
                Some(identifiant) if !identifiant.is_empty() => {
                    par_identifiant
                        .entry(identifiant.to_owned())
                        .or_default()
                        .push(ou);
                }
                _ => sans_identifiant.push(ou),
            }
        }
    }

    // Versant « cible non vide » — exigence 4. Un contrat vide passerait les deux assertions
    // suivantes sans rien vérifier, et c'est exactement le défaut que P-08 a présenté au cycle 001.
    assert!(
        operations >= 56,
        "P-01b — seulement {operations} opération(s) lue(s) du contrat, pour 56 attendues au \
         minimum à la clôture du cycle 004. La porte ne compare rien, ou le contrat a rétréci. \
         Le contrat est-il assemblé ? (`application::contrat_complet`)"
    );

    assert!(
        sans_identifiant.is_empty(),
        "P-01b — {} opération(s) sans `operationId` :\n  {}\n\n\
         `openapi-typescript` dérive le nom du membre de l'`operationId` ; sans lui, il retombe \
         sur une clé construite à partir du chemin, qui change dès qu'on renomme la route.",
        sans_identifiant.len(),
        sans_identifiant.join("\n  ")
    );

    let doublons: Vec<String> = par_identifiant
        .iter()
        .filter(|(_, ou)| ou.len() > 1)
        .map(|(identifiant, ou)| format!("  · « {identifiant} » — {}", ou.join(", ")))
        .collect();

    assert!(
        doublons.is_empty(),
        "P-01b — {} `operationId` porté(s) par plus d'une opération :\n{}\n\n\
         Le client TypeScript généré déclarerait deux membres homonymes. P-01 resterait VERTE : \
         elle ne compare que le généré au commité, et un client invalide se régénère à \
         l'identique.",
        doublons.len(),
        doublons.join("\n")
    );

    assert_eq!(
        par_identifiant.len(),
        operations,
        "P-01b — {} identifiant(s) distinct(s) pour {operations} opération(s) : le décompte ne \
         retombe pas, alors qu'aucun doublon n'a été signalé. L'extraction est cassée.",
        par_identifiant.len()
    );

    println!("P-01b — {operations} opérations, {operations} operationId distincts.");
}

/// **Test négatif de P-01b** — la porte sait échouer.
///
/// *Exigence 4 du § « Couverture des portes ».* Exercé sur un jeu simulé : injecter un doublon
/// dans le vrai contrat ferait échouer le test ci-dessus au hasard de l'ordonnancement.
#[test]
fn test_negatif_p01b_detecte_un_doublon_et_une_absence() {
    let simule: &[(&str, Option<&str>)] = &[
        ("GET /api/v1/comptes", Some("compte_lister")),
        ("POST /api/v1/comptes", Some("compte_creer")),
        // Le doublon : deux routes, un seul nom.
        ("GET /api/v1/personnes/{id}", Some("compte_lister")),
        // L'absence, et sa variante insidieuse — une chaîne vide n'est pas `None`.
        ("DELETE /api/v1/session", None),
        ("GET /api/v1/session/moi", Some("   ")),
    ];

    let mut par_identifiant: BTreeMap<&str, usize> = BTreeMap::new();
    let mut sans = 0usize;
    for (_, identifiant) in simule {
        match identifiant.map(str::trim) {
            Some(nom) if !nom.is_empty() => *par_identifiant.entry(nom).or_default() += 1,
            _ => sans += 1,
        }
    }

    assert_eq!(
        par_identifiant.values().filter(|n| **n > 1).count(),
        1,
        "la porte n'a pas vu le doublon : elle ne protège rien"
    );
    assert_eq!(
        sans, 2,
        "la porte n'a pas vu les deux identifiants absents — dont la chaîne vide, qui est le cas \
         qu'un simple `is_some()` laisserait passer"
    );
}

// =================================================================================================
//  Taxonomie d'audit — le recollement de couverture, distinct de celui de cohérence
// =================================================================================================

/// La taxonomie, lue à la compilation.
const TAXONOMIE: &str = include_str!("../../docs/taxonomie-audit.md");

/// Les tests d'audit de classe A, lus à la compilation — c'est là qu'une famille branchée
/// s'exerce.
const AUDIT_CLASSE_A: &str = include_str!("audit_classe_a.rs");
const HEBERGEMENT_TARIFICATION: &str = include_str!("hebergement_tarification.rs");

/// **Les fichiers de tests qui exercent le registre des actions, nommés un par un.**
///
/// # Pourquoi ce n'est plus un fichier unique (corrigé au cycle 004)
///
/// La porte ne lisait que `audit_classe_a.rs`. Or un test d'audit vit naturellement **près du
/// cycle qui branche sa famille** : la rebascule de palier est exercée par
/// `hebergement_tarification.rs`, qui crée un dépassement réel et relit le registre. La porte
/// signalait donc une famille « branchée sans test » alors que le test existait, à côté.
///
/// Un fichier fautif se serait corrigé de deux façons : déplacer le test là où la porte regarde —
/// c'est-à-dire l'éloigner de ce qu'il teste —, ou élargir le regard. La seconde est la bonne, à
/// une condition : que la liste reste **nommée**. Un balayage de `tests/*.rs` ferait passer la
/// porte au premier fichier qui mentionnerait une famille en commentaire.
///
/// `include_str!` est le garde-fou : un fichier retiré de la liste ne compile plus, au lieu de
/// rétrécir la cible en silence.
const TESTS_QUI_EXERCENT_L_AUDIT: &[(&str, &str)] = &[
    ("audit_classe_a.rs", AUDIT_CLASSE_A),
    ("hebergement_tarification.rs", HEBERGEMENT_TARIFICATION),
];

/// Nombre de familles au document, repris de `audit_taxonomie.rs`.
///
/// Dix au cycle 003, **onze depuis le cycle 005** — `derive_horloge_constatee` (SYN-04).
const FAMILLES_ATTENDUES: usize = 11;

/// Le titre de la section du document où vit le tableau des familles.
///
/// Il porte le décompte en toutes lettres ; `audit_taxonomie.rs` déclare la même constante de son
/// côté. **La duplication est délibérée** — les deux fichiers sont des binaires de test distincts,
/// et un module partagé ferait qu'une extraction cassée casserait les deux du même coup, donc
/// silencieusement.
const TITRE_SECTION_TAXONOMIE: &str = "## Les onze familles";

/// `changement_role` → `ChangementRole`.
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

/// Les familles du document, `(code, branchée)`.
///
/// L'extraction est celle de `audit_taxonomie.rs`, réduite à ce que ce recollement demande. La
/// dupliquer est délibéré : les deux fichiers sont des **binaires de test distincts**, et un module
/// partagé ferait qu'une extraction cassée casserait les deux du même coup — donc silencieusement,
/// puisque les deux tomberaient sur une liste vide. La longueur attendue est asserée des deux
/// côtés, ce qui est la protection réelle.
fn familles_du_document() -> Vec<(String, bool)> {
    let Some(debut) = TAXONOMIE.find(TITRE_SECTION_TAXONOMIE) else {
        panic!("la section « {TITRE_SECTION_TAXONOMIE} » a disparu de docs/taxonomie-audit.md");
    };
    let section = &TAXONOMIE[debut..];
    let fin = section[3..]
        .find("\n## ")
        .map(|i| i + 3)
        .unwrap_or(section.len());

    section[..fin]
        .lines()
        .filter_map(|ligne| {
            let cellules: Vec<&str> = ligne
                .trim()
                .strip_prefix('|')?
                .trim_end_matches('|')
                .split('|')
                .map(str::trim)
                .collect();
            if cellules.len() < 5 || cellules[0].parse::<usize>().is_err() {
                return None;
            }
            let code = cellules[1].trim_matches('`').trim().to_owned();
            let branchee = match cellules[3].replace('*', "").trim() {
                "branché" => true,
                "dû" => false,
                autre => panic!("état « {autre} » inconnu pour la famille « {code} »"),
            };
            Some((code, branchee))
        })
        .collect()
}

/// **Toute famille d'audit déclarée « branchée » est exercée par un test.**
///
/// # Ce que ce contrôle ajoute à `audit_taxonomie.rs`, qui n'est pas la même question
///
/// `audit_taxonomie.rs` compare le document au **code de production** : une famille branchée a-t-
/// elle un chemin d'écriture ? C'est la cohérence.
///
/// Ce test-ci pose la question de la **couverture**, et elle est indépendante : un chemin
/// d'écriture peut exister sans qu'aucun test ne l'emprunte. C'est le même couple que P-05, où
/// `p05_aucun_type_emis_par_le_code_n_est_absent_de_la_liste` regarde le code et
/// `p05_les_types_d_evenements_declares_sont_tous_couverts` regarde les tests.
///
/// Deux familles sont branchées à la clôture du cycle 003 — `suppression` (CPT-01) et
/// `changement_role` (CPT-02) — et huit restent dues aux tranches T2 et T3.
#[test]
fn toute_famille_d_audit_branchee_est_exercee_par_un_test() {
    let familles = familles_du_document();

    assert_eq!(
        familles.len(),
        FAMILLES_ATTENDUES,
        "{} famille(s) extraite(s) de docs/taxonomie-audit.md au lieu de {FAMILLES_ATTENDUES}. \
         Le tableau a-t-il été reformaté ? Une extraction vide passerait au vert sans rien \
         comparer.",
        familles.len()
    );

    let branchees: Vec<&String> = familles
        .iter()
        .filter(|(_, branchee)| *branchee)
        .map(|(code, _)| code)
        .collect();

    assert!(
        !branchees.is_empty(),
        "aucune famille branchée : le produit en compte trois à la fin du cycle 004. Une cible \
         vide passe toujours."
    );

    // Une famille est exercée si l'un des fichiers déclarés emploie sa **variante Rust** —
    // `TypeActionAudit::ChangementRole`, quand le test construit l'entrée — ou son **code
    // littéral** entre guillemets, quand le test relit la colonne `type_action` en base. Les deux
    // formes sont du code : le second motif ne peut pas être satisfait par une simple mention.
    let non_exercees: Vec<String> = branchees
        .iter()
        .filter(|code| {
            let variante = format!("TypeActionAudit::{}", variante_rust(code));
            let litteral = format!("\"{code}\"");
            !TESTS_QUI_EXERCENT_L_AUDIT
                .iter()
                .any(|(_, source)| source.contains(&variante) || source.contains(&litteral))
        })
        .map(|code| (*code).clone())
        .collect();

    assert!(
        non_exercees.is_empty(),
        "{} famille(s) d'audit déclarée(s) « branchée(s) » sans aucun test dans les {} fichier(s) \
         inspecté(s) :\n  {}\n\n\
         Fichiers inspectés : {}\n\n\
         Un chemin d'écriture existe — `audit_taxonomie.rs` le vérifie — mais rien ne l'emprunte. \
         Le registre d'audit est à rétention illimitée : ce qu'on y écrit sans l'avoir exercé, on \
         ne le corrige pas après coup.\n\
         Si le test existe dans un fichier absent de cette liste, l'y inscrire — la liste est \
         nommée exprès, et `include_str!` empêche qu'elle rétrécisse en silence.",
        non_exercees.len(),
        TESTS_QUI_EXERCENT_L_AUDIT.len(),
        non_exercees.join("\n  "),
        TESTS_QUI_EXERCENT_L_AUDIT
            .iter()
            .map(|(nom, _)| *nom)
            .collect::<Vec<_>>()
            .join(", ")
    );

    println!(
        "taxonomie d'audit — {}/{} familles branchées, toutes exercées ; {} due(s).",
        branchees.len(),
        FAMILLES_ATTENDUES,
        FAMILLES_ATTENDUES - branchees.len()
    );
}

/// Le fichier de la porte du registre des classes hors-ligne, lu à la compilation.
///
/// Il porte lui aussi un décompte de tables — `TABLES_ATTENDUES` — établi indépendamment de
/// celui-ci, sur les quatre schémas applicatifs. **Deux décomptes du même ensemble doivent
/// s'accorder** ; qu'ils ne s'accordent pas est le symptôme exact d'un schéma oublié par l'un des
/// deux, ce qui est arrivé à `comptes` dans les deux fichiers de ce cycle.
const CLASSES_OFFLINE: &str = include_str!("classes_offline.rs");

/// Le nombre déclaré par `classes_offline.rs`, extrait de son source.
///
/// Le lire plutôt que le recopier est ce qui fait de ce test un **recollement** : une constante
/// recopiée diverge en silence, une constante relue échoue au premier écart.
fn tables_attendues_par_classes_offline() -> usize {
    CLASSES_OFFLINE
        .lines()
        .find_map(|ligne| {
            let ligne = ligne.trim();
            let apres = ligne.strip_prefix("const TABLES_ATTENDUES: usize = ")?;
            apres.trim_end_matches(';').parse::<usize>().ok()
        })
        .expect(
            "`const TABLES_ATTENDUES: usize = …;` introuvable dans classes_offline.rs. La \
             déclaration a-t-elle été reformulée ? Sans elle, ce recollement ne compare plus rien.",
        )
}

/// **Récapitulatif des décomptes**, imprimé pour la revue de fin de cycle — et recollé.
///
/// # Ce test ne se contente pas d'imprimer
///
/// Sa version du cycle 002 était un simple affichage. Elle comptait `WHERE nspname =
/// 'etablissements'` : à la clôture du cycle 003, elle aurait annoncé **13 tables** pour un produit
/// qui en porte 26, sans qu'aucune assertion ne bronche. Un récapitulatif faux est pire qu'aucun —
/// c'est le chiffre qu'on recopie dans la revue.
///
/// Il compte donc désormais **les cinq schémas applicatifs**, et confronte son total à celui que
/// `classes_offline.rs` déclare de son côté.
///
/// **`hebergement` est le cinquième, ajouté au cycle 004.** Sans lui, le récapitulatif aurait
/// compté 26 tables là où `classes_offline.rs` en déclarait 34 — et c'est bien ce qu'il a fait au
/// premier passage de ce recollement. La confrontation des deux totaux est ce qui l'a montré : un
/// décompte seul serait resté plausible.
#[tokio::test]
async fn recapitulatif_des_portes_a_decompte() {
    let pool = commun::pool_owner().await;

    // Les cinq schémas applicatifs, dans l'ordre de leur apparition au produit. `public` en est
    // exclu : `sqlx.toml` y place la table de suivi des migrations, qui ne porte rien de métier.
    const SCHEMAS_APPLICATIFS: &[&str] = &[
        "etablissements",
        "synchronisation",
        "fiscalite",
        "comptes",
        "hebergement",
    ];

    let par_schema: Vec<(String, i64)> = sqlx::query(
        r#"
        SELECT n.nspname AS schema, COUNT(*) AS tables
        FROM pg_class c
        JOIN pg_namespace n ON n.oid = c.relnamespace
        WHERE c.relkind = 'r' AND n.nspname = ANY($1)
        GROUP BY n.nspname
        ORDER BY n.nspname
        "#,
    )
    .bind(SCHEMAS_APPLICATIFS)
    .fetch_all(&pool)
    .await
    .expect("comptage")
    .into_iter()
    .map(|l| (l.get::<String, _>("schema"), l.get::<i64, _>("tables")))
    .collect();

    let tables: i64 = par_schema.iter().map(|(_, n)| n).sum();

    assert_eq!(
        par_schema.len(),
        SCHEMAS_APPLICATIFS.len(),
        "{} schéma(s) applicatif(s) trouvé(s) sur {} attendu(s) : {:?}\n\n\
         Un schéma absent du catalogue est un schéma que ce récapitulatif ne compte pas — et son \
         total resterait plausible. C'est ainsi que `comptes` a échappé au balayage pendant tout \
         le cycle 003.",
        par_schema.len(),
        SCHEMAS_APPLICATIFS.len(),
        par_schema
    );

    let attendues = tables_attendues_par_classes_offline();
    assert_eq!(
        usize::try_from(tables).expect("décompte positif"),
        attendues,
        "{tables} table(s) dans les cinq schémas applicatifs, contre {attendues} déclarée(s) par \
         `classes_offline.rs`.\n\
         Ventilation : {par_schema:?}\n\n\
         Les deux fichiers comptent le même ensemble ; un écart signifie qu'une migration a été \
         ajoutée sans mettre à jour `TABLES_ATTENDUES`, ou qu'un schéma manque à l'un des deux \
         balayages."
    );

    let operations: usize = application::contrat_complet()
        .paths
        .paths
        .values()
        .map(compter_operations)
        .sum();

    let familles = familles_du_document();
    let branchees = familles.iter().filter(|(_, b)| *b).count();

    println!("Recollement des portes à décompte — clôture du cycle 003 (CPT) :");
    println!(
        "  P-05  — {} types d'événements déclarés et couverts, {} sans émetteur ({})",
        TYPES_EVENEMENTS.len(),
        TYPES_SANS_EMETTEUR.len(),
        TYPES_SANS_EMETTEUR.join(", ")
    );
    println!(
        "  P-07  — {} tables créées par les cycles 002 et 003, {tables} au total sur {} schémas",
        TABLES_CREEES.len(),
        par_schema.len()
    );
    for (schema, n) in &par_schema {
        println!("            {schema:>16} — {n} table(s)");
    }
    println!("  P-08  — {operations} opérations servies, toutes avec un régime déclaré");
    println!("  P-01b — {operations} operationId, tous présents et distincts");
    println!(
        "  audit — {branchees} famille(s) branchée(s) sur {}, toutes exercées",
        familles.len()
    );
    println!();
    println!("Une porte qui s'étend sur plusieurs phases laisse un trou par construction ;");
    println!("ce fichier est ce qui le referme.");
}
