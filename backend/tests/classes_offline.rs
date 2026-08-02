//! **Porte du registre des classes hors-ligne** — une table non déclarée fait échouer le build.
//!
//! # Le sens de la comparaison, et pourquoi il n'est pas symétrique
//!
//! La comparaison va **de la table vers le registre** (R-10) :
//!
//! - une **table sans déclaration** est une erreur — c'est ce qu'on attrape ;
//! - une **entité déclarée sans table** est normale — `docs/registre-classes-offline.md` décrit
//!   tout le produit, y compris ce que les cycles HEB, PDV, CAI et FIS construiront.
//!
//! Comparer dans les deux sens ferait échouer la porte sur les quelque cent entités du registre
//! qui n'ont pas encore de table, et elle serait désactivée dans la semaine.
//!
//! # La limite assumée, écrite ici plutôt qu'enfouie
//!
//! **Cette porte vérifie la PRÉSENCE d'une entité au registre, jamais la JUSTESSE de sa classe.**
//!
//! Le registre classe des **opérations**, pas seulement des tables : `encaissement` y figure deux
//! fois — en **B** pour les espèces, en **D** pour le Mobile Money — parce que le mode de
//! règlement change ce qui est possible hors ligne. Aucune lecture du schéma ne peut retrouver
//! cette distinction : elle est métier.
//!
//! Prétendre l'automatiser produirait une porte qui ment — elle passerait au vert sur un
//! classement faux, et ce vert donnerait l'assurance qui empêche la relecture humaine. La
//! justesse des classes reste donc **humaine, revue mensuellement** (constitution, § Revue).
//!
//! Ce que la porte garantit, et qui suffit à son objet : **aucune entité ne peut être créée sans
//! que quelqu'un ait ouvert le registre et écrit une classe.** C'est le moment où la question se
//! pose ; c'est celui qu'on manquait avant.

mod commun;

use std::collections::BTreeSet;

use sqlx::Row;

/// Le registre, lu à la compilation. Une modification du fichier recompile le test.
const REGISTRE: &str = include_str!("../../docs/registre-classes-offline.md");

/// Schémas soumis à la porte.
///
/// **`comptes` manquait, et son absence était un trou de couverture réel** : les dix tables du
/// cycle 003 échappaient au balayage, et une onzième créée sans déclaration serait passée. Le
/// test `les_entites_du_cycle_003_sont_declarees` les nommait une par une — ce qui vérifie que
/// celles-là sont déclarées, jamais qu'aucune autre ne manque. C'est exactement la différence
/// entre une liste et une porte.
/// **`hebergement` est ajouté par le cycle 004, et son absence aurait été le même trou.** Les huit
/// tables de la première verticale échapperaient entièrement au balayage — exactement ce que le
/// cycle 003 a trouvé sur `comptes`. La liste est le périmètre de la porte : ce qui n'y est pas
/// n'est pas inspecté, et rien ne le dit.
const SCHEMAS_APPLICATIFS: &[&str] = &[
    "etablissements",
    "synchronisation",
    "fiscalite",
    "comptes",
    "hebergement",
];

/// Nombre de tables attendu dans les cinq schémas — **le décompte, pas seulement la liste**.
///
/// 16 au cycle 002, 26 après le cycle 003, **34** à la fin du cycle 004. Une porte dont la cible
/// se vide passe toujours au vert : ce nombre est ce qui distingue « tout est déclaré » de « il
/// n'y avait rien à inspecter ».
///
/// **Il suit les tables réellement créées, migration par migration**, et non la cible du cycle :
/// 26 + 6 du référentiel (`0024`) + `occupation` (`0025`) + `prestation_incluse` (`0026`) = 34.
/// L'écrire d'avance rendrait la porte rouge entre deux migrations pour une raison qui n'est pas
/// un défaut, et on prendrait l'habitude de la lire rouge.
const TABLES_ATTENDUES: usize = 34;

/// Tables exclues, **nommées une par une**, jamais par motif.
///
/// Un motif — « tout ce qui commence par `_` » — laisserait passer toute table future qui s'y
/// conformerait par accident. Chaque exclusion doit s'écrire, donc se justifier.
const TABLES_EXCLUES: &[&str] = &[
    // Table de suivi des migrations de sqlx : elle ne porte aucune donnée métier, seulement des
    // numéros de version appliqués. `backend/api/sqlx.toml` la place hors des schémas
    // applicatifs, l'exclusion est donc redondante aujourd'hui — gardée pour le jour où la
    // configuration changerait.
    "_migrations_appliquees",
];

/// Extrait les entités déclarées au registre.
///
/// Le registre est un tableau Markdown dont la première colonne porte les noms d'entités entre
/// accents graves. L'extraction est mécanique — et c'est précisément pourquoi le format du
/// registre ne doit pas changer sans mettre ce test à jour.
fn entites_declarees() -> BTreeSet<String> {
    let mut entites = BTreeSet::new();

    for ligne in REGISTRE.lines() {
        let ligne = ligne.trim();
        if !ligne.starts_with('|') {
            continue;
        }

        // Première cellule du tableau.
        let Some(cellule) = ligne.split('|').nth(1) else {
            continue;
        };

        // Tous les fragments entre accents graves de la cellule : une ligne peut en porter
        // plusieurs — « `mapping_comptable`, `exercice_comptable` » est une seule ligne du §10.
        let mut reste = cellule;
        while let Some(debut) = reste.find('`') {
            let apres = &reste[debut + 1..];
            let Some(fin) = apres.find('`') else { break };
            let brut = &apres[..fin];
            reste = &apres[fin + 1..];

            // « `etablissement.classement` » déclare une colonne : l'entité est la partie avant
            // le point.
            let nom = brut.split('.').next().unwrap_or(brut).trim();
            if !nom.is_empty() && nom.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                entites.insert(nom.to_lowercase());
            }
        }
    }

    entites
}

async fn tables_reelles(pool: &sqlx::PgPool) -> BTreeSet<String> {
    sqlx::query(
        r#"
        SELECT table_name
        FROM information_schema.tables
        WHERE table_schema = ANY($1) AND table_type = 'BASE TABLE'
        "#,
    )
    .bind(SCHEMAS_APPLICATIFS)
    .fetch_all(pool)
    .await
    .expect("lecture du catalogue")
    .into_iter()
    .map(|l| l.get::<String, _>("table_name").to_lowercase())
    .filter(|t| !TABLES_EXCLUES.contains(&t.as_str()))
    .collect()
}

#[tokio::test]
async fn toute_table_est_declaree_au_registre_des_classes_hors_ligne() {
    let pool = commun::pool_owner().await;
    let tables = tables_reelles(&pool).await;
    let declarees = entites_declarees();

    assert!(
        !tables.is_empty(),
        "aucune table trouvée — la porte n'a rien vérifié. Base non migrée ?"
    );
    assert_eq!(
        tables.len(),
        TABLES_ATTENDUES,
        "{} table(s) inspectée(s) au lieu des {TABLES_ATTENDUES} attendues :\n  {}\n\n\
         Une porte dont la cible rétrécit passe au vert sans rien vérifier. Si une table a été \
         ajoutée, incrémenter TABLES_ATTENDUES **dans le même changement** que la migration.",
        tables.len(),
        tables.iter().map(String::as_str).collect::<Vec<_>>().join("\n  ")
    );
    assert!(
        declarees.len() > 50,
        "seulement {} entités extraites du registre : l'extraction est probablement cassée, et la \
         porte échouerait sur tout. Le format du tableau a-t-il changé ?",
        declarees.len()
    );

    let non_declarees: Vec<&String> = tables.difference(&declarees).collect();

    assert!(
        non_declarees.is_empty(),
        "{} table(s) absente(s) de docs/registre-classes-offline.md :\n  {}\n\n\
         Toute entité déclare sa classe A/B/C/D (principe VI), dans le MÊME changement que sa \
         migration. En cas de doute, classer plus strictement : une entité indûment classée A \
         produit des incohérences silencieuses découvertes trois mois plus tard en pleine \
         clôture ; une entité indûment classée B produit une frustration immédiate, visible et \
         corrigeable.",
        non_declarees.len(),
        non_declarees
            .iter()
            .map(|t| t.as_str())
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}

/// **Test négatif** — une table non déclarée fait bien échouer la porte.
///
/// Sans lui, rien ne distinguerait une porte qui fonctionne d'une porte dont l'extraction est
/// cassée et qui déclarerait tout conforme.
///
/// # Pourquoi la table fautive n'est PAS créée en base
///
/// Une première version créait réellement `etablissements.zzz_table_non_declaree`, le temps de
/// l'inventaire. Les fichiers de test s'exécutent en parallèle sur une base partagée : cette
/// table apparaissait donc dans le catalogue pendant que `toute_table_est_declaree_au_registre`
/// et la porte P-07 inventoriaient — et l'un ou l'autre échouait, au hasard de l'ordonnancement.
///
/// Un test qui casse ses voisins est un test qu'on finit par ignorer. La comparaison est donc
/// exercée sur un ensemble **simulé**, ce qui vérifie exactement ce qui compte : la fonction de
/// différence signale bien une table absente du registre.
#[test]
fn test_negatif_une_table_non_declaree_est_signalee() {
    let declarees = entites_declarees();

    let mut tables_simulees = BTreeSet::new();
    tables_simulees.insert("note_etablissement".to_owned()); // déclarée
    tables_simulees.insert("zzz_table_non_declaree".to_owned()); // absente du registre

    let non_declarees: Vec<&String> = tables_simulees.difference(&declarees).collect();

    assert!(
        non_declarees
            .iter()
            .any(|t| t.as_str() == "zzz_table_non_declaree"),
        "la porte n'a pas signalé une table absente du registre : elle ne protège rien. \
         Trouvées : {non_declarees:?}"
    );
    assert!(
        !non_declarees
            .iter()
            .any(|t| t.as_str() == "note_etablissement"),
        "la porte signale une table pourtant déclarée : elle échouerait sur tout, et serait \
         désactivée dans la semaine"
    );
}

/// Les entités créées par ce cycle sont bien au registre, avec la classe attendue.
///
/// Vérification de **présence uniquement** — voir la limite assumée en tête de fichier. Le fait
/// que `note_etablissement` soit classée A plutôt que B reste une décision humaine ; ce test
/// constate qu'elle a été prise et écrite.
#[test]
fn les_entites_du_cycle_001_sont_declarees() {
    let declarees = entites_declarees();

    for entite in [
        "tenant",
        "etablissement",
        "note_etablissement",
        "evenement_outbox",
        "exercice_comptable",
        "mapping_comptable",
    ] {
        assert!(
            declarees.contains(entite),
            "« {entite} » n'est pas déclarée au registre alors que ce cycle la crée"
        );
    }
}

/// **Les onze entités du cycle 002 sont déclarées au registre.**
///
/// # Pourquoi onze, et pas dix
///
/// Le cycle crée **dix tables** et en enrichit une onzième — `etablissement`, à qui ETB-01 ajoute
/// sept colonnes d'identité. Le décompte de la porte P-07 est celui des tables *créées* (dix) ;
/// celui du registre est celui des *entités* (onze). Les confondre ferait inspecter un
/// sous-ensemble en croyant tout couvrir — le défaut exact que la constitution a documenté après
/// le cycle 001.
///
/// Deux d'entre elles — `profil_stock` et `parametre_catalogue` — étaient **absentes du registre**
/// avant ce cycle, et le test principal ci-dessus les aurait attrapées dès l'application de la
/// migration 0008. Ce test-ci les nomme explicitement, pour que leur ajout soit une décision
/// visible plutôt que la conséquence mécanique d'un build rouge.
#[test]
fn les_entites_du_cycle_002_sont_declarees() {
    let declarees = entites_declarees();

    for entite in [
        "etablissement",  // enrichie par ETB-01 — la onzième entité, sans table nouvelle
        "module_activite",
        "capacite",
        "profil_stock",   // AJOUTÉE par ce cycle
        "etablissement_module",
        "module_capacite",
        "point_de_vente",
        "table_pdv",
        "parametre_catalogue", // AJOUTÉE par ce cycle
        "parametre_configuration",
        "branding",
    ] {
        assert!(
            declarees.contains(entite),
            "« {entite} » n'est pas déclarée au §5.1 de docs/registre-classes-offline.md alors que \
             le cycle 002 la crée ou l'enrichit.\n\
             La déclaration se fait dans le MÊME changement que la migration (principe VI), avec \
             une entrée au journal §13."
        );
    }
}

/// Les dix entités du cycle 003 (CPT) sont déclarées.
///
/// # Le §5.2 avait été écrit d'AVANCE, et ce test le constate
///
/// Neuf de ces entités figuraient au registre depuis le 2026-07-30, avant qu'aucune table
/// n'existe. C'était tout l'objet de les écrire alors : le cycle qui les implémente **honore** un
/// classement décidé à froid, au lieu de le décider à chaud, au milieu d'une migration, en
/// cherchant surtout à ce que la porte passe.
///
/// `methode_authentification` et `role_permission` sont les deux ajouts réels du cycle.
#[test]
fn les_entites_du_cycle_003_sont_declarees() {
    let declarees = entites_declarees();

    for entite in [
        "personne",
        "compte",
        "methode_authentification", // AJOUTÉE par ce cycle
        "role",
        "permission",
        "role_permission", // AJOUTÉE par ce cycle
        "compte_role",
        "journal_audit",
        "employe",         // déclarée au §10 des provisions, pas au §5.2
        "appareil_enrole", // provision, déclarée au §5.2
    ] {
        assert!(
            declarees.contains(entite),
            "« {entite} » n'est pas déclarée à docs/registre-classes-offline.md alors que le \
             cycle 003 la crée.\n\
             La déclaration se fait dans le MÊME changement que la migration (principe VI), avec \
             une entrée au journal §13."
        );
    }
}

/// Les huit entités du cycle 004 (HEB) sont déclarées.
///
/// # Le §7 avait été écrit d'AVANCE, comme le §5.2
///
/// Six d'entre elles figuraient au registre depuis le 2026-07-30, avant qu'aucune table n'existe.
/// Le cycle qui les implémente **honore** un classement décidé à froid, au lieu de le décider à
/// chaud, au milieu d'une migration, en cherchant surtout à ce que la porte passe.
///
/// `temps_remise_en_etat` et `plage_demi_journee` sont les deux ajouts réels du cycle : le
/// registre mentionnait le premier comme *attribut* de `categorie`, et nommait le second sans
/// nom de table.
///
/// **`prestation_incluse` est vérifiée ici alors qu'elle est déclarée au §10**, pas au §7.1 : la
/// redéclarer lui donnerait deux entrées, donc un jour deux classes. Le test ne s'intéresse qu'à
/// sa présence quelque part au registre — c'est exactement ce que la porte principale vérifie.
#[test]
fn les_entites_du_cycle_004_sont_declarees() {
    let declarees = entites_declarees();

    for entite in [
        "categorie",
        "temps_remise_en_etat", // AJOUTÉE par ce cycle — était un attribut, devient une table
        "unite",
        "formule",
        "bareme_palier",
        "plage_demi_journee", // AJOUTÉE par ce cycle — la ligne existait sans nom de table
        "occupation",
        "prestation_incluse", // déclarée au §10 des provisions, pas au §7.1
    ] {
        assert!(
            declarees.contains(entite),
            "« {entite} » n'est pas déclarée à docs/registre-classes-offline.md alors que le \
             cycle 004 la crée.\n\
             La déclaration se fait dans le MÊME changement que la migration (principe VI), avec \
             une entrée au journal §13."
        );
    }
}

/// **Les sessions ne sont PAS au registre, et c'est vérifié.**
///
/// Elles vivent en Redis, éphémères reconstructibles (research R-01). Une ligne « `session` —
/// classe C » serait une erreur douce : elle laisserait croire qu'il existe une table, donc une
/// sauvegarde, donc une donnée à restaurer. Il n'y a rien à restaurer — Redis vidé, tout le monde
/// se reconnecte.
///
/// Le registre le dit désormais explicitement ; ce test refuse qu'on ajoute la ligne par
/// symétrie.
#[test]
fn les_sessions_ne_sont_pas_declarees_comme_une_entite_durable() {
    let declarees = entites_declarees();

    for absente in ["session", "session_active", "jeton_rafraichissement"] {
        assert!(
            !declarees.contains(absente),
            "« {absente} » a été déclarée au registre des classes hors-ligne. Les sessions vivent \
             en Redis et sont éphémères reconstructibles : leur donner une classe laisserait \
             croire qu'il existe une table, donc une sauvegarde, donc une donnée à restaurer."
        );
    }
}

/// **La lecture en cache d'un référentiel est déclarée de classe A** — la distinction qui manquait.
///
/// Le registre classe des **opérations**, pas des tables. L'écriture d'un référentiel est de
/// classe C ; sa **lecture** doit rester possible hors connexion, avec fraîcheur affichée. Sans
/// cette ligne au registre, un cycle ultérieur conclurait qu'une entité de classe C ne se lit pas
/// hors ligne — et le produit deviendrait inutilisable dès la première coupure, une serveuse ne
/// pouvant même pas afficher la liste des services de son établissement.
///
/// Le test porte sur le **texte du registre**, pas sur du code : le mécanisme de cache relève de
/// SYN-01/02 et d'ETB-06. Ce qui est vérifié ici est que la décision a été prise et écrite, au
/// même titre que la classe d'une entité.
#[test]
fn la_lecture_en_cache_des_referentiels_est_declaree() {
    assert!(
        REGISTRE.contains("**Lecture en cache** de tout référentiel"),
        "la ligne distinguant l'ÉCRITURE (C) de la LECTURE EN CACHE (A) a disparu du §5.1 de \
         docs/registre-classes-offline.md.\n\
         Sans elle, rien ne dit qu'un référentiel de classe C reste lisible hors ligne, et le \
         premier cycle qui écrira le cache tranchera dans le sens le plus simple — celui qui \
         rend le produit inutilisable dès la première coupure."
    );
}

// =================================================================================================
//  Les SEPT opérations de classe C du cycle 003 — et leur versant positif
// =================================================================================================

/// Une opération d'écriture de classe C, telle que le contrat l'expose.
struct OperationC {
    /// Le nom au registre, tel qu'on en parle.
    nom: &'static str,
    methode: actix_web::http::Method,
    /// Chemin du **contrat** — celui que la porte P-08 connaît, avec ses accolades.
    chemin_contrat: &'static str,
}

/// **Les sept, nommées une par une.** Le décompte est ce qui rend la liste opposable.
///
/// Une liste sans total attendu se vide par distraction : il suffit qu'une opération soit retirée
/// pour que la boucle en inspecte six et passe au vert. Le `assert_eq!` sur `SEPT` en fin de test
/// l'empêche.
const OPERATIONS_C: &[OperationC] = &[
    OperationC {
        nom: "création de personne",
        methode: actix_web::http::Method::POST,
        chemin_contrat: "/api/v1/personnes",
    },
    OperationC {
        nom: "création de compte",
        methode: actix_web::http::Method::POST,
        chemin_contrat: "/api/v1/comptes",
    },
    OperationC {
        nom: "changement d'état de compte",
        methode: actix_web::http::Method::PUT,
        chemin_contrat: "/api/v1/comptes/{compte_id}/etat",
    },
    OperationC {
        nom: "changement de mot de passe",
        methode: actix_web::http::Method::PUT,
        chemin_contrat: "/api/v1/comptes/{compte_id}/mot-de-passe",
    },
    OperationC {
        nom: "attribution de rôle",
        methode: actix_web::http::Method::POST,
        chemin_contrat: "/api/v1/comptes/{compte_id}/roles",
    },
    OperationC {
        nom: "retrait de rôle",
        methode: actix_web::http::Method::DELETE,
        chemin_contrat: "/api/v1/comptes/{compte_id}/roles/{role_code}",
    },
    OperationC {
        nom: "révocation de session",
        methode: actix_web::http::Method::DELETE,
        chemin_contrat: "/api/v1/session/actives/{session_id}",
    },
];

const SEPT: usize = 7;

/// **Aucune des sept n'est atteignable sans authentification.**
///
/// C'est le versant *négatif* de la porte P-13 côté serveur. Il n'existe pas de file d'attente
/// côté serveur : ce qu'on peut vérifier ici est que chacune exige un jeton — donc une session,
/// donc une connexion, donc le réseau. Une opération de classe C accessible sans jeton serait
/// atteignable depuis n'importe quel chemin, y compris un terminal qui vide une file.
///
/// **Le décompte est déclaré**, et la liste des deux opérations publiques est fermée : elles sont
/// nommées dans `routes/session.rs` et vérifiées par `isolation_tenant.rs`.
#[tokio::test]
async fn les_sept_operations_de_classe_c_exigent_un_jeton() {
    let contrat = kaya_api::application::contrat_complet();
    let mut inspectees = 0usize;
    let mut sans_securite = Vec::new();

    for operation in OPERATIONS_C {
        let item = contrat
            .paths
            .paths
            .get(operation.chemin_contrat)
            .unwrap_or_else(|| {
                panic!(
                    "« {} » ({}) n'est pas au contrat : la liste des opérations de classe C a \
                     dérivé du produit, ou l'opération a été retirée sans que ce test suive.",
                    operation.nom, operation.chemin_contrat
                )
            });

        let op = match operation.methode {
            actix_web::http::Method::POST => item.post.as_ref(),
            actix_web::http::Method::PUT => item.put.as_ref(),
            actix_web::http::Method::DELETE => item.delete.as_ref(),
            _ => None,
        }
        .unwrap_or_else(|| {
            panic!(
                "« {} » : le contrat ne sert pas {} sur {}",
                operation.nom, operation.methode, operation.chemin_contrat
            )
        });

        inspectees += 1;

        // `security` absent OU vide = opération publique. Les deux seules du produit sont
        // `session_ouvrir` et `session_rafraichir`, et aucune n'est de classe C.
        let gardee = op
            .security
            .as_ref()
            .is_some_and(|exigences| !exigences.is_empty());

        if !gardee {
            sans_securite.push(operation.nom);
        }
    }

    assert_eq!(
        inspectees, SEPT,
        "{inspectees} opération(s) de classe C inspectée(s) au lieu de {SEPT}. Une porte dont la \
         cible rétrécit passe au vert sans rien vérifier."
    );
    assert!(
        sans_securite.is_empty(),
        "Ces opérations de classe C ne portent aucune exigence d'authentification :\n  {}\n\n\
         Une opération de classe C atteignable sans jeton est atteignable depuis n'importe quel \
         chemin — y compris un terminal qui vide une file locale. Le principe VI l'interdit.",
        sans_securite.join("\n  ")
    );
}

/// **LE VERSANT POSITIF — chacune des sept fonctionne EN LIGNE.**
///
/// *Une porte qui refuse sans vérifier ce qu'elle autorise passe au vert en n'ayant rien à
/// inspecter.* Le test précédent constate que les sept exigent un jeton ; celui-ci constate qu'avec
/// le jeton, les sept **aboutissent**. Sans lui, une opération retirée du produit satisferait
/// encore la moitié négative de la porte.
///
/// Les sept sont exercées **dans l'ordre où elles se déroulent réellement** : on crée une
/// personne, puis son compte, on lui donne un rôle, on le lui retire, on change son état, son mot
/// de passe, et on coupe une session. C'est le parcours de M. Koffi créant le compte d'Adjoua.
#[actix_web::test]
async fn les_sept_operations_de_classe_c_fonctionnent_en_ligne() {
    use actix_web::http::StatusCode;
    use serde_json::json;
    use uuid::Uuid;

    const AUTORISATION: &str = "Authorization";

    let pool_owner = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool_owner, "P-13 versant positif").await;
    let etb = jeu.etablissement_id;

    let koffi = commun::compte_connecte(
        &pool_owner,
        jeu,
        "Koffi classe C",
        &[("proprietaire", Some(etb))],
    )
    .await;

    let pool = commun::pool_app().await;
    let app = monter_application!(pool.clone());

    let personne_id = Uuid::now_v7();
    let compte_id = Uuid::now_v7();
    let hexa = Uuid::now_v7().simple().to_string();
    let telephone = format!("+225{}", &hexa[hexa.len() - 10..]);

    let mut reussies = 0usize;

    /// Exécute une requête et exige un statut de succès.
    macro_rules! exiger_succes {
        ($nom:expr, $requete:expr) => {{
            let reponse = actix_web::test::call_service(&app, $requete).await;
            let statut = reponse.status();
            assert!(
                statut.is_success(),
                "« {} » a rendu {statut} en ligne. Le versant NÉGATIF de cette porte resterait \
                 vert sur une opération retirée du produit : c'est ce que cette moitié-ci empêche.",
                $nom
            );
            reussies += 1;
            statut
        }};
    }

    // 1 · Création de personne.
    exiger_succes!(
        "création de personne",
        actix_web::test::TestRequest::post()
            .uri("/api/v1/personnes")
            .insert_header((AUTORISATION, koffi.bearer.clone()))
            .set_json(json!({ "id": personne_id, "nom": "Adjoua", "prenoms": "Kouassi" }))
            .to_request()
    );

    // 2 · Création de compte.
    exiger_succes!(
        "création de compte",
        actix_web::test::TestRequest::post()
            .uri("/api/v1/comptes")
            .insert_header((AUTORISATION, koffi.bearer.clone()))
            .set_json(json!({
                "id": compte_id,
                "personne_id": personne_id,
                "identifiant_telephone": telephone,
                "mot_de_passe": commun::MOT_DE_PASSE_TEST,
            }))
            .to_request()
    );

    // 3 · Attribution de rôle.
    exiger_succes!(
        "attribution de rôle",
        actix_web::test::TestRequest::post()
            .uri(&format!("/api/v1/comptes/{compte_id}/roles"))
            .insert_header((AUTORISATION, koffi.bearer.clone()))
            .set_json(json!({
                "id": Uuid::now_v7(),
                "role_code": "caissier",
                "etablissement_id": etb,
            }))
            .to_request()
    );

    // 4 · Retrait de rôle. `caissier` ne porte pas `cpt.role.attribuer` : FR-023 ne s'y oppose pas.
    exiger_succes!(
        "retrait de rôle",
        actix_web::test::TestRequest::delete()
            .uri(&format!(
                "/api/v1/comptes/{compte_id}/roles/caissier?etablissement_id={etb}"
            ))
            .insert_header((AUTORISATION, koffi.bearer.clone()))
            .to_request()
    );

    // 5 · Changement de mot de passe — par un habilité, donc **sans** mot de passe actuel.
    exiger_succes!(
        "changement de mot de passe",
        actix_web::test::TestRequest::put()
            .uri(&format!("/api/v1/comptes/{compte_id}/mot-de-passe"))
            .insert_header((AUTORISATION, koffi.bearer.clone()))
            .set_json(json!({ "nouveau_mot_de_passe": "abidjan-tomate-chaise" }))
            .to_request()
    );

    // 6 · Changement d'état — la désactivation, qui trace une entrée `suppression`.
    exiger_succes!(
        "changement d'état de compte",
        actix_web::test::TestRequest::put()
            .uri(&format!("/api/v1/comptes/{compte_id}/etat"))
            .insert_header((AUTORISATION, koffi.bearer.clone()))
            .set_json(json!({ "actif": false }))
            .to_request()
    );

    // 7 · Révocation de session — Koffi coupe la sienne, ce qui ne demande aucune permission.
    let moi: serde_json::Value = actix_web::test::call_and_read_body_json(
        &app,
        actix_web::test::TestRequest::get()
            .uri("/api/v1/session/moi")
            .insert_header((AUTORISATION, koffi.bearer.clone()))
            .to_request(),
    )
    .await;
    let session_id = moi["session_id"].as_str().expect("session_id");

    let statut = exiger_succes!(
        "révocation de session",
        actix_web::test::TestRequest::delete()
            .uri(&format!("/api/v1/session/actives/{session_id}"))
            .insert_header((AUTORISATION, koffi.bearer.clone()))
            .to_request()
    );
    assert_eq!(statut, StatusCode::NO_CONTENT);

    assert_eq!(
        reussies, SEPT,
        "{reussies} opération(s) exercée(s) en ligne au lieu de {SEPT}"
    );

    // **Et la coupure est immédiate** : le jeton qui vient de servir n'est plus accepté.
    let apres = actix_web::test::call_service(
        &app,
        actix_web::test::TestRequest::get()
            .uri("/api/v1/session/moi")
            .insert_header((AUTORISATION, koffi.bearer.clone()))
            .to_request(),
    )
    .await;
    assert_eq!(
        apres.status(),
        StatusCode::UNAUTHORIZED,
        "la session révoquée est encore acceptée : la liste de révocation n'est pas consultée"
    );
}
