//! **SEJ-01** — la recherche de fiches clients, ses trois formes, et sa mesure.
//!
//! # Le test qui décide du produit est le dernier de ce fichier
//!
//! Le cadrage §5.6 fait de la rapidité du comptoir une **condition d'existence** : un écran lent
//! est un écran contourné. FR-006 et SC-005 fixent la cible — **300 ms au 95ᵉ centile sur 10 000
//! fiches**, mesurée côté serveur.
//!
//! ⚠️ **Le jeu de mesure est engendré par ce test, jamais par les seeds** (FR-007). Dix mille
//! fiches d'identités fictives dans un tenant de démonstration seraient chargées à chaque
//! préparation de poste, ralentiraient chaque écran, et finiraient un jour dans une démonstration
//! client. Il vit dans un tenant dédié, créé et rempli ici.
//!
//! # Ce que ce fichier vérifie, et qui ne se voit pas en relecture
//!
//! - **Les deux apostrophes** — droite `U+0027` et typographique `U+2019` — retrouvent la même
//!   fiche. Les claviers logiciels produisent la seconde ; une fiche « N'Guessan » créée sur
//!   tablette serait introuvable depuis un poste fixe, et le symptôme (« la fiche a disparu »)
//!   n'oriente vers aucune cause.
//! - **Une personne non qualifiée cliente n'apparaît JAMAIS.** `comptes.personne` porte le
//!   personnel autant que les clients : sans la jointure de qualification, chercher « Kouamé » à
//!   la réception ferait apparaître la femme de ménage.
//! - **La troncature est dite.** Une liste silencieusement coupée est un mensonge sur un écran de
//!   comptoir : Yao conclurait que la fiche n'existe pas et en créerait une seconde.

mod commun;

use std::time::Instant;

use actix_web::test;
use serde_json::json;
use uuid::Uuid;

use commun::{JeuTenant, compte_connecte, pool_app, pool_owner};

/// Le rôle du comptoir — **`receptionniste`, et pas `proprietaire`**.
///
/// ⚠️ **Le réflexe des cycles précédents était `proprietaire`, « qui porte toutes les
/// permissions ».** Depuis la migration `0030`, c'est faux : le propriétaire ne reçoit que les
/// deux **lectures** (`sej.client.lire`, `heb.sejour.lire`). Il consulte — il veut savoir qui est
/// passé et ce qui a été facturé — mais il n'enregistre pas d'arrivée, et lui donner
/// `sej.client.gerer` « au cas où » rendrait le registre des actions moins lisible en y mêlant des
/// gestes qu'il ne fait pas.
///
/// Le symptôme de l'oubli est un `403` sur une création de fiche, message qui accuse le handler
/// alors que la cause est le rôle choisi par le test.
const ROLE: &str = "receptionniste";

/// Nombre de fiches du jeu de mesure — **la cible de SC-005**.
const FICHES_DE_MESURE: usize = 10_000;

/// Nombre de recherches par forme, pour le 95ᵉ centile.
///
/// Cent, parce qu'un centile sur dix mesures n'est pas un centile : la 95ᵉ valeur de cent mesures
/// est lisible, la « 9,5ᵉ » de dix ne l'est pas.
const MESURES_PAR_FORME: usize = 100;

/// La cible, en millisecondes (FR-006, SC-005).
const CIBLE_P95_MS: u128 = 300;

// =================================================================================================
//  Utilitaires
// =================================================================================================

/// Crée une fiche par le chemin réel — l'endpoint, jamais un `INSERT` direct.
///
/// Un `INSERT` direct contournerait le calcul des colonnes repliées, et le test mesurerait une
/// recherche sur des données qu'aucun chemin de production ne produit.
///
/// # Pourquoi une macro et non une fonction
///
/// Le service rendu par `monter_application!` a un type que seul `actix_http::Request` sait
/// nommer — un crate qui n'est **pas** une dépendance de ce harnais. L'ajouter imposerait une
/// entrée au gel (principe XI) pour une signature. La macro contourne le nommage : c'est le même
/// arbitrage que `monter_application!` lui-même, et le même que l'encodage d'URL ci-dessous.
macro_rules! creer_fiche {
    ($app:expr, $bearer:expr, $corps:expr) => {{
        let corps = $corps;
        let requete = test::TestRequest::post()
            .uri("/api/v1/clients")
            .insert_header(("authorization", $bearer.to_owned()))
            .set_json(&corps)
            .to_request();
        let reponse = test::call_service(&$app, requete).await;
        assert_eq!(
            reponse.status(),
            201,
            "la création d'une fiche doit rendre 201 — corps : {corps}"
        );
    }};
}

macro_rules! rechercher {
    ($app:expr, $bearer:expr, $saisie:expr) => {{
        let requete = test::TestRequest::get()
            .uri(&format!(
                "/api/v1/clients?recherche={}",
                urlencoding_minimal($saisie)
            ))
            .insert_header(("authorization", $bearer.to_owned()))
            .to_request();
        let reponse = test::call_service(&$app, requete).await;
        assert_eq!(reponse.status(), 200, "la recherche doit rendre 200");
        let corps: serde_json::Value = test::read_body_json(reponse).await;
        corps
    }};
}

/// Encodage d'URL minimal — **écrit ici plutôt qu'importé**.
///
/// Les seuls caractères problématiques des saisies de ce fichier sont l'espace, le `+` et
/// l'apostrophe typographique. Ajouter une dépendance d'encodage pour trois remplacements
/// imposerait une entrée au gel (principe XI) pour ce que quatre lignes couvrent.
fn urlencoding_minimal(saisie: &str) -> String {
    saisie
        .chars()
        .map(|c| match c {
            ' ' => "%20".to_owned(),
            '+' => "%2B".to_owned(),
            c if c.is_ascii_alphanumeric() || "-_.~".contains(c) => c.to_string(),
            c => c.encode_utf8(&mut [0u8; 4]).bytes().map(|b| format!("%{b:02X}")).collect(),
        })
        .collect()
}

fn noms_de_la_liste(resultat: &serde_json::Value) -> Vec<String> {
    resultat["clients"]
        .as_array()
        .expect("clients est un tableau")
        .iter()
        .map(|c| c["nom"].as_str().expect("nom").to_owned())
        .collect()
}

// =================================================================================================
//  Les trois formes
// =================================================================================================

/// **Le repli des signes diacritiques, sur le jeu de noms ivoiriens réels.**
///
/// Chacun de ces noms est courant à Abengourou. Un jeu synthétique aurait couvert la table sans
/// montrer que « N'Guessan » a deux apostrophes possibles.
#[actix_web::test]
async fn la_recherche_par_nom_replie_les_signes_diacritiques() {
    let owner = pool_owner().await;
    let jeu = commun::creer_tenant(&owner, "SEJ — repli").await;
    let connexion = compte_connecte(&owner, jeu, "Yao", &[(ROLE, Some(jeu.etablissement_id))]).await;
    let application = monter_application!(pool_app().await);

    let noms = [
        "Kouamé", "N'Guessan", "Aïcha", "Traoré", "Koffi", "Yao", "Bakayoko", "Adjoua", "Éboué",
        "Gbagbo", "Ouattara",
    ];
    for nom in noms {
        creer_fiche!(application, connexion.bearer, json!({ "id": Uuid::now_v7(), "nom": nom }));
    }

    // Ce que Yao tape réellement : sans accent, en minuscules.
    for (saisie, attendu) in [
        ("kouame", "Kouamé"),
        ("aicha", "Aïcha"),
        ("traore", "Traoré"),
        ("eboue", "Éboué"),
    ] {
        let resultat = rechercher!(application, connexion.bearer, saisie);
        let noms = noms_de_la_liste(&resultat);
        assert!(
            noms.iter().any(|n| n == attendu),
            "« {saisie} » doit retrouver « {attendu} » — trouvés : {noms:?}"
        );
    }
}

/// ★ **Les deux apostrophes retrouvent la MÊME fiche.**
///
/// C'est l'assertion qui compte, et pas seulement le fait que chacune se replie : une
/// implémentation qui retirerait `U+0027` et laisserait `U+2019` passerait le test précédent.
#[actix_web::test]
async fn les_deux_apostrophes_retrouvent_la_meme_fiche() {
    let owner = pool_owner().await;
    let jeu = commun::creer_tenant(&owner, "SEJ — apostrophes").await;
    let connexion = compte_connecte(&owner, jeu, "Yao", &[(ROLE, Some(jeu.etablissement_id))]).await;
    let application = monter_application!(pool_app().await);

    // La fiche est créée avec l'apostrophe TYPOGRAPHIQUE — celle que produit un clavier de
    // tablette par correction automatique.
    creer_fiche!(application, connexion.bearer, json!({ "id": Uuid::now_v7(), "nom": "N\u{2019}Guessan", "prenoms": "Marie" }));

    for saisie in ["nguessan", "N'Guessan", "N\u{2019}Guessan"] {
        let resultat = rechercher!(application, connexion.bearer, saisie);
        assert_eq!(
            noms_de_la_liste(&resultat).len(),
            1,
            "« {saisie} » doit retrouver la fiche créée avec l'apostrophe typographique — c'est \
             le cas de la tablette au comptoir contre le poste fixe à la réception"
        );
    }
}

/// **Le téléphone se retrouve avec ou sans indicatif.** Au comptoir, personne ne le tape.
#[actix_web::test]
async fn le_telephone_se_retrouve_avec_et_sans_indicatif() {
    let owner = pool_owner().await;
    let jeu = commun::creer_tenant(&owner, "SEJ — téléphone").await;
    let connexion = compte_connecte(&owner, jeu, "Yao", &[(ROLE, Some(jeu.etablissement_id))]).await;
    let application = monter_application!(pool_app().await);

    creer_fiche!(application, connexion.bearer, json!({ "id": Uuid::now_v7(), "nom": "Bakayoko", "telephone": "+2250707123456" }));

    for saisie in ["0707123456", "+2250707123456", "07 07 12 34 56"] {
        let resultat = rechercher!(application, connexion.bearer, saisie);
        assert_eq!(
            noms_de_la_liste(&resultat),
            vec!["Bakayoko".to_owned()],
            "« {saisie} » doit retrouver la fiche : la comparaison se fait par SUFFIXE"
        );
    }
}

/// **Le numéro de pièce se retrouve malgré espaces, tirets et casse.**
#[actix_web::test]
async fn le_numero_de_piece_se_retrouve_malgre_la_ponctuation() {
    let owner = pool_owner().await;
    let jeu = commun::creer_tenant(&owner, "SEJ — pièce").await;
    let connexion = compte_connecte(&owner, jeu, "Yao", &[(ROLE, Some(jeu.etablissement_id))]).await;
    let application = monter_application!(pool_app().await);

    creer_fiche!(application, connexion.bearer, json!({
            "id": Uuid::now_v7(),
            "nom": "Ouattara",
            "type_piece": "CNI",
            "numero_piece": "CI-0012 3456",
        }));

    for saisie in ["CI00123456", "ci00123456", "CI-0012-3456"] {
        let resultat = rechercher!(application, connexion.bearer, saisie);
        assert_eq!(
            noms_de_la_liste(&resultat),
            vec!["Ouattara".to_owned()],
            "« {saisie} » doit retrouver la fiche : le même numéro écrit autrement est le même \
             numéro"
        );
    }
}

/// ★ **Une personne non qualifiée cliente n'apparaît JAMAIS.**
///
/// `comptes.personne` porte le personnel autant que les clients (CPT-00 — « une femme de ménage a
/// une fiche et aucun compte »). Sans la jointure de qualification, chercher « Kouamé » à la
/// réception ferait apparaître la femme de ménage — et ce serait une fuite d'identité du personnel
/// vers le comptoir.
#[actix_web::test]
async fn le_personnel_n_apparait_pas_dans_les_resultats() {
    let owner = pool_owner().await;
    let jeu = commun::creer_tenant(&owner, "SEJ — personnel").await;
    let connexion = compte_connecte(&owner, jeu, "Yao", &[(ROLE, Some(jeu.etablissement_id))]).await;
    let application = monter_application!(pool_app().await);

    // Une personne du PERSONNEL — créée par le chemin de CPT-00, sans qualification cliente.
    commun::creer_compte(
        &owner,
        jeu.tenant_id,
        "Kouamé Femme-de-ménage",
        &format!("+225{}", &Uuid::now_v7().simple().to_string()[22..]),
        commun::MOT_DE_PASSE_TEST,
        &[],
    )
    .await;

    // Un CLIENT du même nom de famille.
    creer_fiche!(application, connexion.bearer, json!({ "id": Uuid::now_v7(), "nom": "Kouamé Client" }));

    let resultat = rechercher!(application, connexion.bearer, "kouame");
    let noms = noms_de_la_liste(&resultat);

    assert_eq!(
        noms,
        vec!["Kouamé Client".to_owned()],
        "seule la personne QUALIFIÉE cliente doit apparaître — trouvés : {noms:?}"
    );
}

/// **La troncature est dite, jamais subie.**
#[actix_web::test]
async fn une_liste_coupee_le_declare() {
    let owner = pool_owner().await;
    let jeu = commun::creer_tenant(&owner, "SEJ — troncature").await;
    let connexion = compte_connecte(&owner, jeu, "Yao", &[(ROLE, Some(jeu.etablissement_id))]).await;
    let application = monter_application!(pool_app().await);

    for rang in 0..5 {
        creer_fiche!(application, connexion.bearer, json!({ "id": Uuid::now_v7(), "nom": format!("Traoré {rang}") }));
    }

    let requete = test::TestRequest::get()
        .uri("/api/v1/clients?recherche=traore&limite=2")
        .insert_header(("authorization", connexion.bearer.clone()))
        .to_request();
    let reponse = test::call_service(&application, requete).await;
    let resultat: serde_json::Value = test::read_body_json(reponse).await;

    assert_eq!(noms_de_la_liste(&resultat).len(), 2, "la limite est respectée");
    assert_eq!(
        resultat["tronque"], true,
        "une liste coupée doit le DIRE : sans cela, Yao conclut que la fiche n'existe pas et en \
         crée une seconde"
    );
}

/// **La recherche ne rend jamais de numéro de pièce**, même quand il existe.
///
/// `ClientResume` porte `piece_enregistree`, un booléen. Sans cette propriété, chaque frappe de
/// Yao produirait vingt entrées au registre des actions et vingt déchiffrements.
#[actix_web::test]
async fn la_recherche_ne_rend_aucun_numero_de_piece() {
    let owner = pool_owner().await;
    let jeu = commun::creer_tenant(&owner, "SEJ — résumé").await;
    let connexion = compte_connecte(&owner, jeu, "Yao", &[(ROLE, Some(jeu.etablissement_id))]).await;
    let application = monter_application!(pool_app().await);

    creer_fiche!(application, connexion.bearer, json!({
            "id": Uuid::now_v7(),
            "nom": "Adjoua",
            "type_piece": "CNI",
            "numero_piece": "CI00987654",
        }));

    let resultat = rechercher!(application, connexion.bearer, "adjoua");
    let brut = resultat.to_string();

    assert!(
        !brut.contains("CI00987654"),
        "aucun numéro de pièce ne doit franchir la recherche — corps : {brut}"
    );
    assert_eq!(
        resultat["clients"][0]["piece_enregistree"], true,
        "le résumé dit qu'une pièce EXISTE sans dire laquelle — c'est ce dont la fiche de police \
         a besoin (FR-047)"
    );
}

// =================================================================================================
//  ★ LA MESURE — 300 ms au 95ᵉ centile sur 10 000 fiches
// =================================================================================================

/// ★ **SC-005** — dix mille fiches, cent recherches par forme, 95ᵉ centile sous 300 ms.
///
/// # Le jeu de mesure vit dans un tenant dédié et n'est jamais chargé ailleurs
///
/// FR-007 l'impose. Dix mille identités fictives dans un tenant de démonstration seraient
/// chargées à chaque préparation de poste et finiraient un jour dans une démonstration client.
///
/// # Le peuplement passe par un `INSERT` direct, et c'est la seule exception du fichier
///
/// Les autres tests créent par l'endpoint, pour ne pas mesurer une recherche sur des données
/// qu'aucun chemin de production ne produit. Ici, dix mille appels HTTP mesureraient la création,
/// pas la recherche — et prendraient plusieurs minutes. **Les colonnes repliées sont donc
/// calculées avec la MÊME fonction que la production**, `kaya_comptes::client::repli`, jamais
/// recopiée : c'est ce qui garde le jeu représentatif.
#[actix_web::test]
#[ignore = "mesure : dix mille fiches à peupler — lancer avec `cargo test -- --ignored`"]
async fn la_recherche_tient_300_ms_au_95e_centile_sur_dix_mille_fiches() {
    let owner = pool_owner().await;
    let jeu = commun::creer_tenant(&owner, "SEJ — jeu de mesure (JAMAIS en démonstration)").await;
    let connexion = compte_connecte(&owner, jeu, "Yao", &[(ROLE, Some(jeu.etablissement_id))]).await;
    let application = monter_application!(pool_app().await);

    peupler_le_jeu_de_mesure(&owner, jeu).await;

    let formes: [(&str, fn(usize) -> String); 3] = [
        ("nom", |i| format!("nom{:04}", i % 1000)),
        ("téléphone", |i| format!("{:06}", i % 1000000)),
        ("pièce", |i| format!("CI{:08}", i % FICHES_DE_MESURE)),
    ];

    let mut echecs = Vec::new();

    for (nom_forme, composer) in formes {
        let mut durees = Vec::with_capacity(MESURES_PAR_FORME);

        for i in 0..MESURES_PAR_FORME {
            let saisie = composer(i);
            let depart = Instant::now();
            let _ = rechercher!(application, connexion.bearer, &saisie);
            durees.push(depart.elapsed().as_millis());
        }

        durees.sort_unstable();
        // Le 95ᵉ centile de cent mesures est la 95ᵉ valeur triée, indice 94.
        let p95 = durees[(MESURES_PAR_FORME * 95 / 100) - 1];
        let median = durees[MESURES_PAR_FORME / 2];

        println!(
            "  recherche par {nom_forme:10} — médiane {median:4} ms · p95 {p95:4} ms · \
             max {:4} ms",
            durees.last().copied().unwrap_or_default()
        );

        if p95 > CIBLE_P95_MS {
            echecs.push(format!("{nom_forme} : p95 = {p95} ms"));
        }
    }

    assert!(
        echecs.is_empty(),
        "SC-005 non tenu sur {FICHES_DE_MESURE} fiches — cible {CIBLE_P95_MS} ms au 95ᵉ centile :\n  \
         {}\n\n\
         Le cadrage §5.6 fait de la rapidité du comptoir une CONDITION D'EXISTENCE du produit : un \
         écran lent est un écran contourné. Vérifier d'abord que `personne_nom_repli_idx` porte \
         bien `text_pattern_ops` — sans lui, un LIKE de préfixe n'emploie pas l'index dès que la \
         collation n'est pas `C`.",
        echecs.join("\n  ")
    );
}

/// Peuple le tenant de mesure — **avec la fonction de repli de production**.
async fn peupler_le_jeu_de_mesure(pool: &sqlx::PgPool, jeu: JeuTenant) {
    use kaya_comptes::client::{repli, repli_piece, repli_telephone};

    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, jeu.tenant_id)
        .await
        .expect("pose du tenant");

    // Un lot unique plutôt que dix mille allers-retours : le peuplement n'est pas ce qu'on mesure.
    let mut ids = Vec::with_capacity(FICHES_DE_MESURE);
    let mut noms = Vec::with_capacity(FICHES_DE_MESURE);
    let mut noms_replies = Vec::with_capacity(FICHES_DE_MESURE);
    let mut telephones = Vec::with_capacity(FICHES_DE_MESURE);
    let mut pieces = Vec::with_capacity(FICHES_DE_MESURE);

    for i in 0..FICHES_DE_MESURE {
        let nom = format!("Nom{:04} Prénom{:04}", i % 1000, i / 1000);
        let telephone = format!("+225{:010}", i);
        let piece = format!("CI{:08}", i);

        ids.push(Uuid::now_v7());
        noms_replies.push(repli(&nom));
        telephones.push(repli_telephone(&telephone));
        pieces.push(repli_piece(&piece));
        noms.push(nom);
    }

    sqlx::query!(
        r#"
        INSERT INTO comptes.personne
            (id, tenant_id, nom, nom_repli, telephone_repli, numero_piece_repli)
        SELECT * FROM UNNEST($1::UUID[], $2::UUID[], $3::TEXT[], $4::TEXT[], $5::TEXT[], $6::TEXT[])
        "#,
        &ids,
        &vec![jeu.tenant_id; FICHES_DE_MESURE],
        &noms,
        &noms_replies,
        &telephones,
        &pieces,
    )
    .execute(&mut *tx)
    .await
    .expect("peuplement des personnes");

    sqlx::query!(
        r#"
        INSERT INTO comptes.client (personne_id, tenant_id)
        SELECT * FROM UNNEST($1::UUID[], $2::UUID[])
        "#,
        &ids,
        &vec![jeu.tenant_id; FICHES_DE_MESURE],
    )
    .execute(&mut *tx)
    .await
    .expect("qualification des clients");

    tx.commit().await.expect("commit du jeu de mesure");

    // `ANALYZE` : sans statistiques à jour, l'optimiseur choisit un parcours séquentiel sur une
    // table qu'il croit petite, et la mesure porterait sur un plan qu'aucune exploitation réelle
    // n'aurait — la base analysant d'elle-même après quelques minutes.
    sqlx::query("ANALYZE comptes.personne")
        .execute(pool)
        .await
        .expect("analyse de la table");
}

// =================================================================================================
//  ★ SC-014 — la fiche client ne dépend d'AUCUN module d'activité
// =================================================================================================

/// ★ **Un établissement SANS module hébergement cherche et crée des fiches clientes.**
///
/// ⚠️ **Les sept tests ci-dessus n'activent aucun module — mais aucun ne le DIT.** Un test qui
/// se trouve ne pas activer de module prouve la même chose qu'un test qui l'asserte, jusqu'au jour
/// où quelqu'un ajoute une activation « pour faire comme les autres » : la propriété disparaît
/// alors sans que rien ne rougisse.
///
/// # Ce que cette propriété protège, et quand elle comptera
///
/// La fiche client est du **tenant**, pas d'un module (FR-002), et ses deux permissions sont
/// **transversales** (`module_code = NULL`, migration `0030`). Un **maquis** ou un **bar seul** en
/// aura besoin dès **SEJ-05** — la vente à un client extérieur, sans hébergement.
///
/// Si la fiche client acquérait une dépendance à `HEBERGEMENT`, ce jour-là il faudrait soit créer
/// une seconde permission de client, soit **activer un module d'hébergement dans un maquis** pour
/// lire une fiche. Les deux sont absurdes, et le second passerait probablement.
#[actix_web::test]
async fn la_fiche_client_fonctionne_sans_aucun_module_d_activite() {
    let owner = pool_owner().await;
    let jeu = commun::creer_tenant(&owner, "SEJ — maquis sans hébergement").await;

    // ── Aucun module n'est activé, et c'est ASSERTÉ ──────────────────────────────────────────
    let modules: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM etablissements.etablissement_module WHERE etablissement_id = $1",
    )
    .bind(jeu.etablissement_id)
    .fetch_one(&owner)
    .await
    .expect("comptage des modules");
    assert_eq!(
        modules, 0,
        "ce test EXIGE un établissement sans module : avec un module actif, il ne prouverait plus          l'indépendance qu'il mesure"
    );

    let connexion = compte_connecte(&owner, jeu, "Yao", &[(ROLE, Some(jeu.etablissement_id))]).await;
    let application = monter_application!(pool_app().await);

    creer_fiche!(
        application,
        connexion.bearer,
        json!({ "id": Uuid::now_v7(), "nom": "Gbagbo", "telephone": "+2250707998877" })
    );

    let resultat = rechercher!(application, connexion.bearer, "gbagbo");
    assert_eq!(
        noms_de_la_liste(&resultat),
        vec!["Gbagbo".to_owned()],
        "★ la fiche client doit fonctionner sans AUCUN module d'activité. Un maquis ou un bar seul \
         en aura besoin dès SEJ-05 : si elle acquérait une dépendance à HEBERGEMENT, il faudrait \
         activer un module d'hébergement dans un maquis pour lire une fiche."
    );
}
