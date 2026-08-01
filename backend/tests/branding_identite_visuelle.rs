//! **ETB-05 — l'identité visuelle : surcharge partielle, aperçu, mention non fiscale.**
//!
//! # Le test qui compte le plus est le plus court
//!
//! `l_apercu_porte_toujours_la_mention_non_fiscale`. Un aperçu d'identité visuelle porte le logo,
//! l'en-tête, les coordonnées et les mentions légales de l'exploitant : **il ressemble à une
//! facture**. Sans la mention « Document non fiscal — ne tient pas lieu de facture », le premier
//! aperçu imprimé serait présenté à un client comme un justificatif — et personne ne s'en
//! apercevrait avant un contrôle.

mod commun;

use uuid::Uuid;

use kaya_etablissements::branding::{
    BrandingNiveau, BrandingResolu, ChampResolu, EcrireBranding, ErreurBranding,
    MENTION_NON_FISCALE, ServiceBranding, couleur_valide, rendre_document_test,
};
use kaya_synchronisation::outbox::PgOutboxWriter;

/// **La mention non fiscale est présente, toujours, quelle que soit l'identité.**
#[test]
fn l_apercu_porte_toujours_la_mention_non_fiscale() {
    let champ = |v: &str| {
        Some(ChampResolu {
            valeur: v.to_owned(),
            origine: "ECRAN".to_owned(),
        })
    };

    // Une identité complète — celle qui ressemble le plus à une facture.
    let complete = BrandingResolu {
        logo_objet_cle: champ("branding/x/tenant/logo"),
        couleur_primaire: champ("#0A7B5F"),
        entete_document: champ("Résidence Hôtel Deloria"),
        pied_document: champ("Merci de votre visite"),
        mentions_legales: champ("RCCM CI-ABJ-2019-B-12345"),
        coordonnees: champ("Abengourou — +225 07 00 00 00 00"),
    };

    let document = rendre_document_test(&complete, "Résidence Hôtel Deloria");
    assert!(
        document.contains(MENTION_NON_FISCALE),
        "le document de test ne porte pas « {MENTION_NON_FISCALE} ».\n\
         Un aperçu porte le logo, l'en-tête et les mentions légales de l'exploitant : il ressemble \
         à une facture. Sans cette phrase, le premier aperçu imprimé serait présenté à un client \
         comme un justificatif.\n\
         Document rendu :\n{document}"
    );

    // Et sur une identité **vide** — le cas où l'on serait tenté de ne rien rendre du tout.
    let vide = BrandingResolu {
        logo_objet_cle: None,
        couleur_primaire: None,
        entete_document: None,
        pied_document: None,
        mentions_legales: None,
        coordonnees: None,
    };
    let document = rendre_document_test(&vide, "Établissement sans identité");
    assert!(
        document.contains(MENTION_NON_FISCALE),
        "une identité visuelle vide produit un document SANS la mention non fiscale — c'est le cas \
         le plus probable au premier essai d'un exploitant.\nDocument rendu :\n{document}"
    );
}

/// **La surcharge est PARTIELLE : surcharger le logo laisse hériter tout le reste.**
///
/// C'est le mécanisme que la nullabilité des colonnes porte, sans qu'aucune logique de fusion
/// n'ait à être écrite (FR-056).
#[tokio::test]
async fn la_surcharge_est_partielle_champ_par_champ() {
    let pool_owner = commun::pool_owner().await;
    let pool_app = commun::pool_app().await;
    let jeu = commun::creer_tenant(&pool_owner, "branding surcharge partielle").await;
    let service = ServiceBranding::nouveau(pool_app, PgOutboxWriter::nouveau());

    // Identité complète au niveau tenant.
    service
        .ecrire(
            jeu.tenant_id,
            EcrireBranding {
                id: Uuid::now_v7(),
                etablissement_id: None,
                contenu: BrandingNiveau {
                    logo_objet_cle: Some("logo_tenant".to_owned()),
                    couleur_primaire: Some("#0A7B5F".to_owned()),
                    entete_document: Some("Groupe Koffi".to_owned()),
                    pied_document: Some("Merci".to_owned()),
                    mentions_legales: Some("RCCM CI-ABJ-2019-B-12345".to_owned()),
                    coordonnees: Some("Abengourou".to_owned()),
                },
            },
        )
        .await
        .expect("identité du tenant");

    // **Le seul logo** est surchargé sur l'établissement.
    service
        .ecrire(
            jeu.tenant_id,
            EcrireBranding {
                id: Uuid::now_v7(),
                etablissement_id: Some(jeu.etablissement_id),
                contenu: BrandingNiveau {
                    logo_objet_cle: Some("logo_residence".to_owned()),
                    ..Default::default()
                },
            },
        )
        .await
        .expect("surcharge du logo");

    let resolu = service
        .resoudre(jeu.tenant_id, Some(jeu.etablissement_id))
        .await
        .expect("résolution");

    let logo = resolu.logo_objet_cle.expect("le logo doit être résolu");
    assert_eq!(logo.valeur, "logo_residence");
    assert_eq!(
        logo.origine, "ETABLISSEMENT",
        "le logo surchargé doit porter l'origine ETABLISSEMENT"
    );

    for (nom, champ, attendu) in [
        ("entete_document", resolu.entete_document, "Groupe Koffi"),
        ("pied_document", resolu.pied_document, "Merci"),
        (
            "mentions_legales",
            resolu.mentions_legales,
            "RCCM CI-ABJ-2019-B-12345",
        ),
        ("coordonnees", resolu.coordonnees, "Abengourou"),
    ] {
        let champ = champ.unwrap_or_else(|| panic!("{nom} doit rester hérité, pas disparaître"));
        assert_eq!(champ.valeur, attendu, "{nom} n'a pas été hérité du tenant");
        assert_eq!(
            champ.origine, "TENANT",
            "{nom} doit porter l'origine TENANT — sans quoi l'écran afficherait « modifié ici » \
             sur une valeur que personne n'a touchée"
        );
    }
}

/// **Le second établissement du même tenant ne voit PAS la surcharge du premier.**
///
/// C'est le scénario de M. Koffi : deux établissements, une identité commune, une exception sur un
/// seul. Sans cette vérification, une surcharge d'établissement mal résolue s'appliquerait aux
/// deux — et la résidence meublée sortirait des documents à l'en-tête de l'hôtel.
#[tokio::test]
async fn une_surcharge_d_etablissement_ne_deborde_pas_sur_l_autre() {
    let pool_owner = commun::pool_owner().await;
    let pool_app = commun::pool_app().await;
    let jeu = commun::creer_tenant(&pool_owner, "branding deux établissements").await;
    let service = ServiceBranding::nouveau(pool_app.clone(), PgOutboxWriter::nouveau());

    // Un second établissement chez le même tenant.
    let second = Uuid::now_v7();
    let mut tx = pool_app.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, jeu.tenant_id)
        .await
        .expect("pose du tenant");
    sqlx::query(
        r#"
        INSERT INTO etablissements.etablissement
            (id, tenant_id, nom, fuseau_horaire, devise, commune)
        VALUES ($1, $2, 'Second établissement', 'Africa/Abidjan', 'XOF', 'Abidjan')
        "#,
    )
    .bind(second)
    .bind(jeu.tenant_id)
    .execute(&mut *tx)
    .await
    .expect("second établissement");
    tx.commit().await.expect("commit");

    service
        .ecrire(
            jeu.tenant_id,
            EcrireBranding {
                id: Uuid::now_v7(),
                etablissement_id: None,
                contenu: BrandingNiveau {
                    logo_objet_cle: Some("logo_commun".to_owned()),
                    entete_document: Some("Groupe Koffi".to_owned()),
                    ..Default::default()
                },
            },
        )
        .await
        .expect("identité du tenant");

    service
        .ecrire(
            jeu.tenant_id,
            EcrireBranding {
                id: Uuid::now_v7(),
                etablissement_id: Some(jeu.etablissement_id),
                contenu: BrandingNiveau {
                    entete_document: Some("Résidence meublée".to_owned()),
                    ..Default::default()
                },
            },
        )
        .await
        .expect("surcharge du premier établissement");

    let premier = service
        .resoudre(jeu.tenant_id, Some(jeu.etablissement_id))
        .await
        .expect("résolution du premier");
    assert_eq!(
        premier.entete_document.as_ref().map(|c| c.valeur.as_str()),
        Some("Résidence meublée")
    );

    let autre = service
        .resoudre(jeu.tenant_id, Some(second))
        .await
        .expect("résolution du second");
    assert_eq!(
        autre.entete_document.as_ref().map(|c| c.valeur.as_str()),
        Some("Groupe Koffi"),
        "la surcharge du premier établissement a débordé sur le second : les deux sortiraient des \
         documents au même en-tête"
    );
    assert_eq!(
        autre.entete_document.as_ref().map(|c| c.origine.as_str()),
        Some("TENANT")
    );
}

/// **Une couleur mal formée est refusée**, aux deux niveaux — service et base.
#[tokio::test]
async fn une_couleur_mal_formee_est_refusee() {
    let pool_owner = commun::pool_owner().await;
    let pool_app = commun::pool_app().await;
    let jeu = commun::creer_tenant(&pool_owner, "branding couleur").await;
    let service = ServiceBranding::nouveau(pool_app.clone(), PgOutboxWriter::nouveau());

    for invalide in ["vert", "#GGG", "#0A7B5", "0A7B5F", "#0A7B5FF"] {
        assert!(
            !couleur_valide(invalide),
            "« {invalide} » ne doit pas être acceptée comme couleur"
        );

        let refus = service
            .ecrire(
                jeu.tenant_id,
                EcrireBranding {
                    id: Uuid::now_v7(),
                    etablissement_id: None,
                    contenu: BrandingNiveau {
                        couleur_primaire: Some(invalide.to_owned()),
                        ..Default::default()
                    },
                },
            )
            .await;
        assert!(
            matches!(refus, Err(ErreurBranding::CouleurInvalide(_))),
            "« {invalide} » a été acceptée par le service : {refus:?}"
        );
    }

    assert!(couleur_valide("#0A7B5F"));
    assert!(
        couleur_valide("#abcdef"),
        "les minuscules sont valides en hexadécimal"
    );

    // Et le `CHECK` de la base tient indépendamment du service — c'est le rempart qu'un import ou
    // un script de reprise rencontrerait.
    let mut tx = pool_app.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, jeu.tenant_id)
        .await
        .expect("pose du tenant");
    let direct = sqlx::query(
        r#"
        INSERT INTO etablissements.branding (id, tenant_id, etablissement_id, couleur_primaire)
        VALUES ($1, $2, NULL, 'pas-une-couleur')
        "#,
    )
    .bind(Uuid::now_v7())
    .bind(jeu.tenant_id)
    .execute(&mut *tx)
    .await;
    assert!(
        direct.is_err(),
        "la base a accepté une couleur mal formée par INSERT direct : le CHECK est absent, et le \
         refus ne tiendrait plus qu'au service"
    );
    let _ = tx.rollback().await;
}

/// **L'aperçu n'enregistre RIEN** (FR-057).
///
/// Il porte l'identité telle qu'elle est à l'écran, y compris non enregistrée : c'est ce qui
/// permet de voir avant de valider, plutôt que d'enregistrer pour voir.
#[actix_web::test]
async fn l_apercu_n_enregistre_rien() {
    let pool_owner = commun::pool_owner().await;
    let pool = commun::pool_app().await;
    let jeu = commun::creer_tenant(&pool_owner, "branding aperçu").await;
    let app = monter_application!(pool.clone());
    let cx = commun::compte_connecte(
        &pool_owner,
        jeu,
        "branding aperçu",
        &[("proprietaire", Some(jeu.etablissement_id))],
    )
    .await;

    let requete = actix_web::test::TestRequest::post()
        .uri("/api/v1/branding/apercu")
        .insert_header(("Authorization", cx.bearer.clone()))
        .set_json(serde_json::json!({
            "nom_etablissement": "Aperçu jamais enregistré",
            "entete_document": "En-tête d'essai",
            "couleur_primaire": "#0A7B5F",
        }))
        .to_request();

    let corps: serde_json::Value = actix_web::test::call_and_read_body_json(&app, requete).await;

    let document = corps["document"].as_str().unwrap_or_default();
    assert!(
        document.contains(MENTION_NON_FISCALE),
        "l'aperçu rendu par l'API ne porte pas la mention non fiscale :\n{document}"
    );
    assert!(
        document.contains("En-tête d'essai"),
        "l'aperçu doit rendre l'identité TELLE QU'ELLE EST À L'ÉCRAN, y compris non enregistrée"
    );
    assert_eq!(
        corps["mention_non_fiscale"].as_str(),
        Some(MENTION_NON_FISCALE),
        "la mention doit être reprise à part, pour que le client puisse la mettre en évidence sans \
         analyser le corps du document"
    );

    // **Rien n'a été écrit.**
    let mut tx = pool_owner.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, jeu.tenant_id)
        .await
        .expect("pose du tenant");
    let lignes: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM etablissements.branding")
        .fetch_one(&mut *tx)
        .await
        .expect("comptage");
    assert_eq!(
        lignes, 0,
        "l'aperçu a écrit {lignes} ligne(s) : il ne doit RIEN enregistrer (FR-057)"
    );
    tx.rollback().await.expect("rollback");
}

/// **La couleur d'identité visuelle n'atteint jamais l'interface** (FR-059).
///
/// Elle s'applique aux **documents produits**. Ce test vérifie le seul endroit du backend qui la
/// consomme : le rendu de document. Le pendant côté application est dans `app/scripts/lint-tokens.ts`,
/// qui exclut nommément `branding.couleur_primaire` de la porte P-17 **et** vérifie qu'elle
/// n'apparaît dans aucun composant de `G1`.
#[test]
fn la_couleur_d_identite_visuelle_ne_sert_qu_aux_documents() {
    let identite = BrandingResolu {
        logo_objet_cle: None,
        couleur_primaire: Some(ChampResolu {
            valeur: "#0A7B5F".to_owned(),
            origine: "TENANT".to_owned(),
        }),
        entete_document: Some(ChampResolu {
            valeur: "En-tête".to_owned(),
            origine: "TENANT".to_owned(),
        }),
        pied_document: None,
        mentions_legales: None,
        coordonnees: None,
    };

    let document = rendre_document_test(&identite, "Test");
    assert!(
        document.contains("En-tête"),
        "le document doit porter l'en-tête"
    );
    assert!(
        document.contains(MENTION_NON_FISCALE),
        "le document doit porter la mention non fiscale"
    );
}
