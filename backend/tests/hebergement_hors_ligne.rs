//! **PORTE P-13 — les treize opérations de l'hébergement, et leur classe hors-ligne.**
//!
//! > Aucune opération de classe B, C ou D n'est atteignable depuis un chemin de code exécutable
//! > hors ligne (principe VI).
//!
//! # Ce que ce fichier peut prouver, et ce qu'il ne peut pas
//!
//! Il n'existe **aucune file d'attente côté serveur** : le rejeu hors ligne est une affaire de
//! terminal, et le terminal n'est pas ici. Ce que le serveur peut établir, et qui suffit à l'objet
//! de la porte, tient en deux propositions :
//!
//! 1. **chacune des treize exige un jeton** — donc une session, donc une connexion, donc le
//!    réseau. Une opération atteignable sans jeton le serait depuis n'importe quel chemin, y
//!    compris un terminal qui vide une file locale ;
//! 2. **les treize aboutissent en ligne**. Sans ce versant, une opération retirée du produit
//!    satisferait encore la moitié négative — c'est la leçon du cycle 003, et elle vaut ici mot
//!    pour mot.
//!
//! # Le périmètre est FERMÉ, pas énuméré
//!
//! [`OPERATIONS`] liste les treize, et le premier test compare cette liste aux chemins
//! `/hebergement/` **réellement présents au contrat**. Une quatorzième opération ajoutée sans être
//! inscrite ici fait échouer la porte au lieu d'échapper au balayage — c'est la différence entre
//! une liste et une porte, et c'est exactement le trou trouvé sur le schéma `comptes` au cycle 003.
//!
//! # Les deux propriétés que ce fichier NE refait PAS, et où elles vivent
//!
//! Le registre §11 exige, pour ce module, un test d'isolation multi-tenant (classe C) et un test de
//! concurrence (classe B). Les deux existent, et les dupliquer ici les ferait diverger :
//!
//! | Propriété | Où elle est tenue |
//! |---|---|
//! | Isolation multi-tenant du référentiel et de la disponibilité | `isolation_tenant.rs` — `p08_cycle_004_appels_croises_sur_le_referentiel_d_hebergement` et `…_sur_la_disponibilite` |
//! | Concurrence de la classe B | `hebergement_disponibilite.rs` — `deux_attributions_concurrentes_une_seule_reussit` |
//!
//! Ce qui est vérifié **ici** en revanche, parce que personne d'autre ne le fait : que la classe
//! déclarée au registre et les **privilèges réellement accordés** disent la même chose.

mod commun;

use actix_web::http::{Method, StatusCode};
use serde_json::json;
use time::OffsetDateTime;
use uuid::Uuid;

const AUTORISATION: &str = "Authorization";

/// Une opération du module, telle que le contrat l'expose.
struct Operation {
    /// Le numéro de l'opération dans `0022_permissions_hebergement.sql`, pour qu'on retrouve sa
    /// permission sans chercher.
    numero: u8,
    nom: &'static str,
    methode: Method,
    /// Chemin **du contrat**, avec ses accolades.
    chemin: &'static str,
}

/// **Les treize, nommées une par une.** Le décompte est ce qui rend la liste opposable.
const OPERATIONS: &[Operation] = &[
    Operation {
        numero: 1,
        nom: "lister les catégories",
        methode: Method::GET,
        chemin: "/api/v1/etablissements/{etablissement_id}/hebergement/categories",
    },
    Operation {
        numero: 2,
        nom: "créer une catégorie",
        methode: Method::POST,
        chemin: "/api/v1/etablissements/{etablissement_id}/hebergement/categories",
    },
    Operation {
        numero: 3,
        nom: "modifier une catégorie",
        methode: Method::PUT,
        chemin: "/api/v1/etablissements/{etablissement_id}/hebergement/categories/{categorie_id}",
    },
    Operation {
        numero: 4,
        nom: "lister les unités",
        methode: Method::GET,
        chemin: "/api/v1/etablissements/{etablissement_id}/hebergement/unites",
    },
    Operation {
        numero: 5,
        nom: "créer une unité",
        methode: Method::POST,
        chemin: "/api/v1/etablissements/{etablissement_id}/hebergement/unites",
    },
    Operation {
        numero: 6,
        nom: "modifier une unité",
        methode: Method::PUT,
        chemin: "/api/v1/etablissements/{etablissement_id}/hebergement/unites/{unite_id}",
    },
    Operation {
        numero: 7,
        nom: "lister les formules",
        methode: Method::GET,
        chemin: "/api/v1/etablissements/{etablissement_id}/hebergement/formules",
    },
    Operation {
        numero: 8,
        nom: "créer une formule",
        methode: Method::POST,
        chemin: "/api/v1/etablissements/{etablissement_id}/hebergement/formules",
    },
    Operation {
        numero: 9,
        nom: "modifier une formule",
        methode: Method::PUT,
        chemin: "/api/v1/etablissements/{etablissement_id}/hebergement/formules/{formule_id}",
    },
    Operation {
        numero: 10,
        nom: "consulter la disponibilité",
        methode: Method::GET,
        chemin: "/api/v1/etablissements/{etablissement_id}/hebergement/disponibilite",
    },
    Operation {
        numero: 11,
        nom: "attribuer une unité",
        methode: Method::POST,
        chemin: "/api/v1/etablissements/{etablissement_id}/hebergement/occupations",
    },
    Operation {
        numero: 12,
        nom: "libérer une occupation",
        methode: Method::POST,
        chemin: "/api/v1/etablissements/{etablissement_id}/hebergement/occupations/{occupation_id}/liberation",
    },
    Operation {
        numero: 13,
        nom: "calculer le tarif",
        methode: Method::POST,
        chemin: "/api/v1/etablissements/{etablissement_id}/hebergement/occupations/{occupation_id}/tarif",
    },
];

const TREIZE: usize = 13;

// =================================================================================================
//  VERSANT NÉGATIF — aucune des treize n'est atteignable sans jeton
// =================================================================================================

/// Chacune des treize porte une exigence d'authentification, **et il n'y en a pas une quatorzième**.
///
/// Les deux moitiés comptent. La première refuse une opération publique ; la seconde refuse une
/// opération qui aurait échappé à la liste — auquel cas la porte inspecterait treize chemins en
/// croyant couvrir le module.
#[test]
fn les_treize_operations_exigent_un_jeton_et_aucune_autre_n_existe() {
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
            Method::DELETE => item.delete.as_ref(),
            _ => None,
        }
        .unwrap_or_else(|| {
            panic!(
                "opération {} — « {} » : le contrat ne sert pas {} sur {}",
                operation.numero, operation.nom, operation.methode, operation.chemin
            )
        });

        inspectees += 1;

        // `security` absent OU vide = opération publique. Les deux seules du produit sont
        // `session_ouvrir` et `session_rafraichir`.
        if !op
            .security
            .as_ref()
            .is_some_and(|exigences| !exigences.is_empty())
        {
            sans_securite.push(format!("{} — {}", operation.numero, operation.nom));
        }
    }

    assert_eq!(
        inspectees, TREIZE,
        "{inspectees} opération(s) inspectée(s) au lieu de {TREIZE}. Une porte dont la cible \
         rétrécit passe au vert sans rien vérifier."
    );
    assert!(
        sans_securite.is_empty(),
        "ces opérations d'hébergement ne portent aucune exigence d'authentification :\n  {}\n\n\
         Le référentiel est de classe C et l'occupation de classe B : ni l'une ni l'autre n'est \
         atteignable hors ligne. Une opération sans jeton l'est depuis n'importe quel chemin — y \
         compris un terminal qui vide une file locale. Le principe VI l'interdit.",
        sans_securite.join("\n  ")
    );

    // ── La liste est FERMÉE — aucune quatorzième au contrat ────────────────────────────────────
    let au_contrat: usize = contrat
        .paths
        .paths
        .iter()
        .filter(|(chemin, _)| chemin.contains("/hebergement/"))
        .map(|(_, item)| {
            [
                item.get.is_some(),
                item.post.is_some(),
                item.put.is_some(),
                item.delete.is_some(),
                item.patch.is_some(),
            ]
            .iter()
            .filter(|servie| **servie)
            .count()
        })
        .sum();

    assert_eq!(
        au_contrat, TREIZE,
        "le contrat sert {au_contrat} opération(s) sous `/hebergement/` alors que ce fichier en \
         déclare {TREIZE}.\n\
         Une opération ajoutée sans être inscrite ici échapperait entièrement au balayage, et la \
         porte resterait verte — c'est exactement le trou trouvé sur le schéma `comptes` au \
         cycle 003. Inscrire l'opération ci-dessus, dans le MÊME changement que sa route."
    );
}

// =================================================================================================
//  LA CLASSE DÉCLARÉE ET LES PRIVILÈGES RÉELS DISENT-ILS LA MÊME CHOSE ?
// =================================================================================================

/// **Les privilèges disent la classe** — module doré, couche 4.
///
/// Le registre déclare le référentiel en **C** et l'occupation en **B**. Ce test constate que la
/// base est réglée en conséquence, et surtout **que les deux régimes diffèrent** : une occupation
/// ne se supprime pas.
///
/// Sans l'assertion de différence, un `GRANT ALL` massif sur tout le schéma satisferait chaque
/// vérification prise isolément.
#[tokio::test]
async fn les_privileges_disent_la_classe_de_chaque_table() {
    let pool = commun::pool_owner().await;

    let privileges: Vec<(String, String)> = sqlx::query_as(
        r#"
        SELECT table_name, string_agg(privilege_type, ',' ORDER BY privilege_type)
        FROM information_schema.role_table_grants
        WHERE grantee = 'kaya_app' AND table_schema = 'hebergement'
        GROUP BY table_name
        ORDER BY table_name
        "#,
    )
    .fetch_all(&pool)
    .await
    .expect("lecture des privilèges");

    assert!(
        !privileges.is_empty(),
        "aucun privilège lu sur le schéma `hebergement` — le test n'inspecte rien. Base non migrée ?"
    );

    let verbes = |table: &str| -> String {
        privileges
            .iter()
            .find(|(t, _)| t == table)
            .map(|(_, v)| v.clone())
            .unwrap_or_default()
    };

    // Classe **C** — l'exploitant crée, corrige et retire son offre. Les quatre verbes.
    for table in [
        "categorie",
        "temps_remise_en_etat",
        "unite",
        "formule",
        "bareme_palier",
        "plage_demi_journee",
    ] {
        assert_eq!(
            verbes(table),
            "DELETE,INSERT,SELECT,UPDATE",
            "`hebergement.{table}` est de classe C : le référentiel s'édite en ligne, et les \
             quatre verbes sont accordés. Obtenu : « {} »",
            verbes(table)
        );
    }

    // Classe **B** — l'occupation ne se supprime PAS. Libérer est un `UPDATE`, jamais un `DELETE` :
    // une occupation effacée emporterait la trace de qui a occupé quoi, et l'historique d'un
    // établissement se réécrirait ligne à ligne sans que rien n'en garde mémoire.
    assert_eq!(
        verbes("occupation"),
        "INSERT,SELECT,UPDATE",
        "`hebergement.occupation` doit porter INSERT, SELECT et UPDATE — et surtout **pas** \
         DELETE. Libérer une chambre est un `UPDATE` qui raccourcit la période et pose un statut ; \
         un `DELETE` effacerait la trace de l'occupation elle-même. Obtenu : « {} »",
        verbes("occupation")
    );

    // Et la provision HEB-09 n'a **rien** — vérifié aussi par `provisions_sans_logique.rs`, gardé
    // ici parce que c'est la troisième valeur de la même échelle : quatre verbes, trois, zéro.
    assert_eq!(
        verbes("prestation_incluse"),
        "",
        "`hebergement.prestation_incluse` est une PROVISION : aucun privilège, pas même SELECT. \
         Obtenu : « {} »",
        verbes("prestation_incluse")
    );
}

// =================================================================================================
//  VERSANT POSITIF — les treize aboutissent EN LIGNE
// =================================================================================================

/// **Le parcours complet d'Adjoua**, des treize opérations dans l'ordre où elles se déroulent.
///
/// *Une porte qui refuse sans vérifier ce qu'elle autorise passe au vert en n'ayant rien à
/// inspecter.* Le test précédent constate que les treize exigent un jeton ; celui-ci constate
/// qu'avec le jeton, les treize **aboutissent**.
///
/// L'ordre n'est pas décoratif : on ne peut pas créer une unité avant sa catégorie, ni attribuer
/// avant d'avoir une formule, ni calculer un tarif avant d'avoir attribué. Le parcours est celui
/// d'Adjoua réglant son offre le matin et attribuant une chambre l'après-midi.
#[actix_web::test]
async fn les_treize_operations_aboutissent_en_ligne() {
    let pool_owner = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool_owner, "P-13 hébergement en ligne").await;
    let etb = jeu.etablissement_id;

    // Le module doit être actif : un service inactif rend `service_inactif`, ce qui ferait échouer
    // le parcours pour une raison qui n'est pas celle qu'on teste.
    let mut tx = pool_owner.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, jeu.tenant_id)
        .await
        .expect("pose du tenant");
    sqlx::query(
        r#"
        INSERT INTO etablissements.etablissement_module
            (id, tenant_id, etablissement_id, module_code, module_implemente)
        VALUES ($1, $2, $3, 'HEBERGEMENT', true)
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(Uuid::now_v7())
    .bind(jeu.tenant_id)
    .bind(etb)
    .execute(&mut *tx)
    .await
    .expect("activation du module");
    tx.commit().await.expect("commit");

    // **Gérante**, parce qu'elle porte les cinq permissions du module. Une réceptionniste n'a pas
    // `heb.offre.gerer` : le parcours s'arrêterait à la création de catégorie, et pour une bonne
    // raison — ce n'est pas Yao qui fixe les tarifs.
    let adjoua =
        commun::compte_connecte(&pool_owner, jeu, "Adjoua P-13", &[("gerant", Some(etb))]).await;

    let app = monter_application!(commun::pool_app().await);
    let base = format!("/api/v1/etablissements/{etb}/hebergement");

    let categorie_id = Uuid::now_v7();
    let unite_id = Uuid::now_v7();
    let formule_id = Uuid::now_v7();
    let occupation_id = Uuid::now_v7();

    let mut reussies = 0usize;

    macro_rules! exiger_succes {
        ($numero:expr, $nom:expr, $requete:expr) => {{
            let reponse = actix_web::test::call_service(&app, $requete).await;
            let statut = reponse.status();
            assert!(
                statut.is_success(),
                "opération {} — « {} » a rendu {statut} EN LIGNE.\n\
                 Le versant négatif de cette porte resterait vert sur une opération retirée du \
                 produit ou cassée : c'est ce que cette moitié-ci empêche.\n\
                 Corps : {}",
                $numero,
                $nom,
                String::from_utf8_lossy(
                    &actix_web::body::to_bytes(reponse.into_body()).await.unwrap_or_default()
                )
            );
            reussies += 1;
            statut
        }};
    }

    // ── L'offre : catégorie, unité, formule — créer, lister, modifier ─────────────────────────

    exiger_succes!(
        2,
        "créer une catégorie",
        actix_web::test::TestRequest::post()
            .uri(&format!("{base}/categories"))
            .insert_header((AUTORISATION, adjoua.bearer.clone()))
            .set_json(json!({
                "id": categorie_id,
                "nom": "Standard",
                "capacite_accueil": 2,
                "temps_remise_en_etat": [
                    { "famille_formule": "NUITEE", "duree_minutes": 120 },
                    { "famille_formule": "PASSAGE", "duree_minutes": 30 },
                ],
            }))
            .to_request()
    );

    exiger_succes!(
        1,
        "lister les catégories",
        actix_web::test::TestRequest::get()
            .uri(&format!("{base}/categories"))
            .insert_header((AUTORISATION, adjoua.bearer.clone()))
            .to_request()
    );

    exiger_succes!(
        3,
        "modifier une catégorie",
        actix_web::test::TestRequest::put()
            .uri(&format!("{base}/categories/{categorie_id}"))
            .insert_header((AUTORISATION, adjoua.bearer.clone()))
            .set_json(json!({ "nom": "Standard rénovée", "capacite_accueil": 3 }))
            .to_request()
    );

    exiger_succes!(
        5,
        "créer une unité",
        actix_web::test::TestRequest::post()
            .uri(&format!("{base}/unites"))
            .insert_header((AUTORISATION, adjoua.bearer.clone()))
            .set_json(json!({ "id": unite_id, "categorie_id": categorie_id, "code": "A1" }))
            .to_request()
    );

    exiger_succes!(
        4,
        "lister les unités",
        actix_web::test::TestRequest::get()
            .uri(&format!("{base}/unites"))
            .insert_header((AUTORISATION, adjoua.bearer.clone()))
            .to_request()
    );

    exiger_succes!(
        6,
        "modifier une unité",
        actix_web::test::TestRequest::put()
            .uri(&format!("{base}/unites/{unite_id}"))
            .insert_header((AUTORISATION, adjoua.bearer.clone()))
            .set_json(json!({ "code": "A1", "etage": 1 }))
            .to_request()
    );

    exiger_succes!(
        8,
        "créer une formule",
        actix_web::test::TestRequest::post()
            .uri(&format!("{base}/formules"))
            .insert_header((AUTORISATION, adjoua.bearer.clone()))
            .set_json(json!({
                "id": formule_id,
                "categorie_id": categorie_id,
                "famille": "NUITEE",
                "prix_mineur": 12_500,
                "assujettie_taxe_nuitee": true,
                "regle_conversion_taxe": "une_nuitee_par_occupation",
            }))
            .to_request()
    );

    exiger_succes!(
        7,
        "lister les formules",
        actix_web::test::TestRequest::get()
            .uri(&format!("{base}/formules"))
            .insert_header((AUTORISATION, adjoua.bearer.clone()))
            .to_request()
    );

    exiger_succes!(
        9,
        "modifier une formule",
        actix_web::test::TestRequest::put()
            .uri(&format!("{base}/formules/{formule_id}"))
            .insert_header((AUTORISATION, adjoua.bearer.clone()))
            .set_json(json!({
                "prix_mineur": 13_000,
                "assujettie_taxe_nuitee": true,
                "regle_conversion_taxe": "une_nuitee_par_occupation",
            }))
            .to_request()
    );

    // ── L'exploitation : disponibilité, attribution, tarif, libération ────────────────────────
    //
    // Les instants sont **futurs et fixes**. Un `now()` côté test rendrait le parcours dépendant du
    // moment de son exécution, et la nuitée attribuée entrerait en conflit avec celle de la
    // veille au prochain passage.
    let debut = OffsetDateTime::now_utc() + time::Duration::days(400);
    let fin = debut + time::Duration::days(2);
    let format = &time::format_description::well_known::Rfc3339;
    let debut_txt = debut.format(format).expect("formatage");
    let fin_txt = fin.format(format).expect("formatage");

    exiger_succes!(
        10,
        "consulter la disponibilité",
        actix_web::test::TestRequest::get()
            .uri(&format!(
                "{base}/disponibilite?categorie_id={categorie_id}&debut={}&fin={}",
                urlencoding(&debut_txt),
                urlencoding(&fin_txt)
            ))
            .insert_header((AUTORISATION, adjoua.bearer.clone()))
            .to_request()
    );

    exiger_succes!(
        11,
        "attribuer une unité",
        actix_web::test::TestRequest::post()
            .uri(&format!("{base}/occupations"))
            .insert_header((AUTORISATION, adjoua.bearer.clone()))
            .set_json(json!({
                "id": occupation_id,
                "unite_id": unite_id,
                "formule_id": formule_id,
                "debut_client": debut_txt,
                "fin_client": fin_txt,
            }))
            .to_request()
    );

    exiger_succes!(
        13,
        "calculer le tarif",
        actix_web::test::TestRequest::post()
            .uri(&format!("{base}/occupations/{occupation_id}/tarif"))
            .insert_header((AUTORISATION, adjoua.bearer.clone()))
            .to_request()
    );

    let statut = exiger_succes!(
        12,
        "libérer une occupation",
        actix_web::test::TestRequest::post()
            .uri(&format!("{base}/occupations/{occupation_id}/liberation"))
            .insert_header((AUTORISATION, adjoua.bearer.clone()))
            .set_json(json!({ "id": occupation_id }))
            .to_request()
    );
    assert!(statut.is_success());

    assert_eq!(
        reussies, TREIZE,
        "{reussies} opération(s) exercée(s) en ligne au lieu de {TREIZE}"
    );

    // **Et sans jeton, la première d'entre elles est refusée.** Le versant négatif est vérifié sur
    // le contrat ; celui-ci le constate sur le serveur réellement monté — un `security` déclaré
    // qu'aucun intergiciel n'applique produirait un contrat conforme et une API ouverte.
    let sans_jeton = actix_web::test::call_service(
        &app,
        actix_web::test::TestRequest::get()
            .uri(&format!("{base}/categories"))
            .to_request(),
    )
    .await;
    assert_eq!(
        sans_jeton.status(),
        StatusCode::UNAUTHORIZED,
        "l'offre se lit sans jeton : le `security` du contrat n'est pas appliqué par le serveur, \
         et le référentiel de classe C serait atteignable depuis n'importe quel chemin"
    );
}

/// Encode un instant RFC 3339 pour une chaîne de requête.
///
/// `now_utc()` produit un `Z`, donc aucun `+` aujourd'hui — l'encodage est **défensif** : le jour
/// où ce test emploierait un instant à décalage explicite, `+00:00` vaudrait espace dans la chaîne
/// de requête, la date serait refusée, et l'échec ne dirait rien de ce qu'on vérifie ici.
fn urlencoding(valeur: &str) -> String {
    valeur.replace('+', "%2B").replace(':', "%3A")
}
