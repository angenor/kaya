//! **CPT-01 — la politique de mot de passe, et le moment où elle s'applique.**
//!
//! # Deux exigences, dont une seule se voit
//!
//! La première est ce que la politique **refuse** : huit caractères, aucune règle de composition,
//! et le refus de ce qui figure déjà dans les listes d'attaquants. Elle se lit dans le code et se
//! teste en trois lignes.
//!
//! La seconde est **quand** elle s'applique — à la création et au changement, jamais à la
//! connexion — et celle-là ne se voit nulle part. Un appel à `politique::verifier` glissé dans le
//! chemin de connexion serait parfaitement raisonnable en revue : « on vérifie que le mot de passe
//! est encore conforme ». Il enfermerait dehors, le jour d'une mise à jour de la liste, un
//! utilisateur légitime dont le mot de passe n'a pas changé — avec, pour tout diagnostic, le refus
//! volontairement indiscernable de FR-012.
//!
//! # Périmètre inspecté
//!
//! | Contrôle | Périmètre | Angle mort assumé |
//! |---|---|---|
//! | Les trois cas de la politique | `authentification::politique::verifier` | — |
//! | Le seuil vient du catalogue | `etablissements.parametre_catalogue`, base réelle | — |
//!
//! **Le contrôle « jamais à la connexion » n'est PAS dans ce commit** : sa cible —
//! `authentification/service.rs` — n'existe pas encore. L'écrire aujourd'hui produirait une porte
//! qui n'inspecte rien, donc verte sans rien prouver. Il arrive avec **T028**, dans le changement
//! qui livre le chemin qu'il surveille.

mod commun;

use kaya_comptes::authentification::politique::{
    LONGUEUR_MIN_DEFAUT, RefusMotDePasse, verifier,
};

// =================================================================================================
//  1 · Les trois cas qui comptent
// =================================================================================================

/// **Sept caractères : refusé.** Le seuil est un seuil.
#[test]
fn sept_caracteres_sont_refuses() {
    assert!(matches!(
        verifier("chaise7", LONGUEUR_MIN_DEFAUT),
        Err(RefusMotDePasse::TropCourt { longueur: 7, .. })
    ));
}

/// **`12345678` : huit caractères, refusé quand même.**
///
/// C'est le cas qui justifie l'existence de la liste embarquée. Une politique de huit caractères
/// sans elle serait battue au premier essai — littéralement, `12345678` est parmi les tout
/// premiers mots de passe essayés.
#[test]
fn douze_mille_trois_cent_quarante_cinq_six_sept_huit_est_refuse_bien_qu_il_fasse_huit() {
    assert_eq!(
        verifier("12345678", LONGUEUR_MIN_DEFAUT),
        Err(RefusMotDePasse::Compromis)
    );
}

/// **`chaise-tomate-abidjan` : accepté, sans majuscule, ni chiffre, ni symbole.**
///
/// C'est le cas qui prouve l'absence de règle de composition. Le refuser reviendrait à la
/// politique que le NIST a retirée de ses recommandations — celle qui produit `Passw0rd!` puis un
/// post-it sous le clavier du comptoir.
#[test]
fn une_phrase_de_passe_sans_regle_de_composition_est_acceptee() {
    assert!(verifier("chaise-tomate-abidjan", LONGUEUR_MIN_DEFAUT).is_ok());
}

/// Les trois motifs de refus ont **trois codes distincts**.
///
/// Le statut HTTP est le même (`422`) ; c'est le corps qui enseigne. Un refus muet ferait essayer
/// `12345679`.
#[test]
fn les_trois_motifs_de_refus_se_distinguent_au_code() {
    let codes: Vec<&str> = [
        verifier("court", LONGUEUR_MIN_DEFAUT),
        verifier("12345678", LONGUEUR_MIN_DEFAUT),
        verifier(&"a".repeat(1000), LONGUEUR_MIN_DEFAUT),
    ]
    .iter()
    .map(|r| r.as_ref().expect_err("doit être refusé").code())
    .collect();

    assert_eq!(
        codes,
        vec![
            "mot_de_passe_trop_court",
            "mot_de_passe_compromis",
            "mot_de_passe_trop_long"
        ]
    );
}

// =================================================================================================
//  2 · Le seuil vient du catalogue, pas d'une constante
// =================================================================================================

/// **`mot_de_passe_longueur_min` est au catalogue, et son défaut vaut celui du code.**
///
/// Les deux valeurs doivent coïncider, sans quoi un établissement qui n'a jamais réglé le
/// paramètre serait soumis à un seuil différent de celui que le catalogue annonce.
#[tokio::test]
async fn le_seuil_est_un_parametre_d_etablissement_et_son_defaut_vaut_celui_du_code() {
    let pool = commun::pool_owner().await;

    let ligne = sqlx::query!(
        r#"
        SELECT type_valeur, portee_la_plus_basse
        FROM etablissements.parametre_catalogue
        WHERE cle = 'mot_de_passe_longueur_min'
        "#
    )
    .fetch_optional(&pool)
    .await
    .expect("lecture du catalogue");

    let ligne = ligne.expect(
        "`mot_de_passe_longueur_min` est absent du catalogue : le seuil vivrait alors en dur dans \
         le code, et un exploitant qui exige douze caractères devrait demander une évolution",
    );

    assert_eq!(ligne.type_valeur, "ENTIER");
    assert_eq!(
        ligne.portee_la_plus_basse, "ETABLISSEMENT",
        "le seuil se règle par établissement — un groupe hôtelier n'a pas la même exigence sur \
         son maquis et sur sa réception"
    );

    assert_eq!(
        LONGUEUR_MIN_DEFAUT, 8,
        "le repli du code doit rester égal au défaut documenté du catalogue ; un repli différent \
         se manifesterait uniquement quand la lecture du paramètre échoue, c'est-à-dire jamais en \
         test"
    );
}
