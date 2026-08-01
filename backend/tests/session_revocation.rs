//! **CPT-01 — la coupure immédiate, et ce qu'elle coûterait de ne pas l'être.**
//!
//! # Le seul recours contre un téléphone volé
//!
//! Un jeton signé reste **mathématiquement valide** jusqu'à son expiration, quoi qu'il arrive. Sans
//! liste de révocation, couper l'accès d'un employé qui vient de partir avec son téléphone
//! supposerait d'attendre : jusqu'à 60 minutes pour le jeton d'accès, **90 jours** pour celui de
//! rafraîchissement. C'est-à-dire un trimestre d'accès à la caisse.
//!
//! Le cadrage §12.2 exige la « coupure immédiate au départ d'un employé ». Ce fichier vérifie
//! qu'elle l'est vraiment — au **prochain appel**, pas au prochain quart d'heure.
//!
//! # Quatre propriétés, et la troisième est celle qu'on écrirait mal
//!
//! | # | Propriété | Ce qui casse sans elle |
//! |---|---|---|
//! | 1 | Une session révoquée cesse d'être acceptée **à la requête suivante** | Le téléphone volé travaille encore un trimestre |
//! | 2 | Les **autres** sessions du compte continuent | Couper un appareil déconnecterait l'employé de tous |
//! | 3 | Un jeton de rafraîchissement présenté **deux fois** révoque **toute la famille** | Le voleur et la victime restent en course, et le premier des deux gagne |
//! | 4 | Un changement de mot de passe révoque les autres sessions, **immédiatement** | Changer son mot de passe ne reprendrait rien à qui l'avait |
//!
//! La troisième est contre-intuitive : la réaction naturelle à un jeton réutilisé est de refuser
//! **celui-là**. Mais on ne sait pas lequel des deux porteurs est le voleur — révoquer un seul
//! exemplaire laisse les deux en course, et **aucun des deux ne sait qu'il y a eu course**. En
//! révoquant la famille, les deux sont déconnectés : le voleur perd l'accès, et la victime apprend
//! qu'il s'est passé quelque chose au moment où elle doit se reconnecter.
//!
//! # Périmètre inspecté
//!
//! Le service réel, la base réelle, Redis réel. **Aucun simulacre** : la révocation *est* un état
//! partagé dans Redis, et un simulacre validerait le code en laissant la garantie non testée.

mod commun;

use kaya_comptes::session::modele::ErreurSession;
use uuid::Uuid;

/// Un identifiant unique à cette exécution.
///
/// ⚠️ La partie aléatoire d'un UUID v7 est **à la fin** : ses 48 premiers bits sont l'horodatage,
/// et deux UUID engendrés dans la même seconde partagent leurs douze premiers caractères
/// hexadécimaux. Tailler dans le préfixe collisionne entre tests parallèles.
fn identifiant_unique() -> String {
    let hexa = Uuid::now_v7().simple().to_string();
    format!("+225{}", &hexa[hexa.len() - 10..])
}

/// Une origine par test — le compteur de tentatives plafonne à soixante par origine.
fn origine(index: u8) -> String {
    format!("198.51.100.{index}")
}

// =================================================================================================
//  1 · La coupure est immédiate
// =================================================================================================

/// **Une session révoquée cesse d'être acceptée à la requête suivante.**
///
/// Le jeton n'a pas expiré, sa signature est parfaite, et il est refusé. C'est tout l'objet de la
/// liste de révocation : sans elle, le refus attendrait l'expiration.
#[tokio::test]
async fn une_session_revoquee_cesse_d_etre_acceptee_immediatement() {
    let pool_owner = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool_owner, "CPT-01 — coupure immédiate").await;
    let identifiant = identifiant_unique();

    commun::creer_compte(
        &pool_owner,
        jeu.tenant_id,
        "Volé",
        &identifiant,
        commun::MOT_DE_PASSE_TEST,
        &[("proprietaire", Some(jeu.etablissement_id))],
    )
    .await;

    let service = commun::service_authentification(commun::pool_app().await);
    let ouverte = service
        .ouvrir(
            &identifiant,
            commun::MOT_DE_PASSE_TEST,
            Some(jeu.etablissement_id),
            Some("téléphone perdu".to_owned()),
            &origine(1),
        )
        .await
        .expect("connexion");

    let session_id = ouverte.jetons.session_id;

    // Avant : la session n'est pas révoquée.
    assert!(
        !service.est_revoquee(session_id).await.expect("lecture"),
        "une session fraîchement ouverte est déjà marquée révoquée"
    );

    service
        .revoquer(
            ouverte.compte_id,
            ouverte.tenant_id,
            ouverte.compte_id,
            session_id,
            3600,
        )
        .await
        .expect("révocation");

    // Après : elle l'est, **sans qu'aucun délai ne se soit écoulé**.
    assert!(
        service.est_revoquee(session_id).await.expect("lecture"),
        "la session n'est pas marquée révoquée juste après l'avoir été. Le jeton d'accès en \
         circulation resterait accepté jusqu'à son expiration — jusqu'à 90 jours pour celui de \
         rafraîchissement, soit un trimestre d'accès à la caisse pour un téléphone volé."
    );

    // Et le rafraîchissement est refusé, ce qui ferme l'autre porte : sans cela, le voleur
    // obtiendrait un jeton d'accès neuf et la révocation ne servirait à rien.
    let refus = service
        .rafraichir(&ouverte.jetons.rafraichissement, None)
        .await;
    assert!(
        matches!(refus, Err(ErreurSession::SessionInvalide)),
        "une session révoquée s'est rafraîchie : le voleur obtient un jeton neuf et la coupure \
         n'a rien coupé"
    );
}

/// **Les autres sessions du compte continuent.**
///
/// Le versant qui manque toujours. Sans lui, une implémentation qui révoquerait tout passerait le
/// test précédent — et Adjoua, coupant le téléphone qu'elle a perdu, se déconnecterait aussi de la
/// caisse devant laquelle elle est en train de travailler.
#[tokio::test]
async fn revoquer_une_session_laisse_les_autres_intactes() {
    let pool_owner = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool_owner, "CPT-01 — deux appareils").await;
    let identifiant = identifiant_unique();

    commun::creer_compte(
        &pool_owner,
        jeu.tenant_id,
        "Adjoua",
        &identifiant,
        commun::MOT_DE_PASSE_TEST,
        &[("proprietaire", Some(jeu.etablissement_id))],
    )
    .await;

    let service = commun::service_authentification(commun::pool_app().await);

    // Deux appareils, deux connexions — c'est le scénario de la story : Adjoua travaille sur la
    // caisse et porte son téléphone.
    let caisse = service
        .ouvrir(
            &identifiant,
            commun::MOT_DE_PASSE_TEST,
            Some(jeu.etablissement_id),
            Some("caisse".to_owned()),
            &origine(2),
        )
        .await
        .expect("connexion caisse");
    let telephone = service
        .ouvrir(
            &identifiant,
            commun::MOT_DE_PASSE_TEST,
            Some(jeu.etablissement_id),
            Some("téléphone".to_owned()),
            &origine(3),
        )
        .await
        .expect("connexion téléphone");

    let actives = service
        .lister_actives(caisse.compte_id, caisse.jetons.session_id)
        .await
        .expect("liste");
    assert_eq!(
        actives.len(),
        2,
        "deux connexions doivent produire deux sessions actives — sinon la seconde a écrasé la \
         première, et « travailler sur deux appareils » est impossible"
    );
    assert_eq!(
        actives.iter().filter(|s| s.courante).count(),
        1,
        "exactement une session doit être marquée « courante » : sans ce drapeau, l'utilisateur \
         se déconnecterait lui-même en croyant couper le téléphone perdu"
    );

    service
        .revoquer(
            telephone.compte_id,
            telephone.tenant_id,
            telephone.compte_id,
            telephone.jetons.session_id,
            3600,
        )
        .await
        .expect("révocation du téléphone");

    assert!(
        service
            .est_revoquee(telephone.jetons.session_id)
            .await
            .expect("lecture"),
        "le téléphone n'a pas été coupé"
    );
    assert!(
        !service
            .est_revoquee(caisse.jetons.session_id)
            .await
            .expect("lecture"),
        "couper le téléphone a coupé la caisse : Adjoua se déconnecte elle-même en essayant de \
         protéger son compte"
    );

    // Et la caisse se rafraîchit toujours — la preuve fonctionnelle, pas seulement la marque.
    assert!(
        service
            .rafraichir(&caisse.jetons.rafraichissement, None)
            .await
            .is_ok(),
        "la caisse ne se rafraîchit plus après la révocation du téléphone"
    );
}

// =================================================================================================
//  2 · La réutilisation révoque la famille entière
// =================================================================================================

/// **Un jeton de rafraîchissement présenté deux fois révoque TOUTE la famille.**
///
/// C'est la propriété la plus contre-intuitive du module, et celle qui distingue une rotation de
/// jetons d'une simple expiration. Le premier usage réussit et **consomme** le jeton ; le second
/// prouve qu'une copie circule.
#[tokio::test]
async fn un_jeton_de_rafraichissement_reutilise_revoque_toute_la_famille() {
    let pool_owner = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool_owner, "CPT-01 — réutilisation").await;
    let identifiant = identifiant_unique();

    commun::creer_compte(
        &pool_owner,
        jeu.tenant_id,
        "Copie en circulation",
        &identifiant,
        commun::MOT_DE_PASSE_TEST,
        &[("proprietaire", Some(jeu.etablissement_id))],
    )
    .await;

    let service = commun::service_authentification(commun::pool_app().await);
    let ouverte = service
        .ouvrir(
            &identifiant,
            commun::MOT_DE_PASSE_TEST,
            Some(jeu.etablissement_id),
            None,
            &origine(4),
        )
        .await
        .expect("connexion");

    // Premier usage : il réussit **et consomme** le jeton. C'est la rotation.
    let renouvelee = service
        .rafraichir(&ouverte.jetons.rafraichissement, None)
        .await
        .expect("le premier rafraîchissement doit réussir");

    assert_ne!(
        renouvelee.jetons.rafraichissement, ouverte.jetons.rafraichissement,
        "le jeton de rafraîchissement n'a pas tourné : présenté deux fois, il serait accepté deux \
         fois, et la détection de réutilisation n'aurait rien à détecter"
    );

    // Second usage du **même** jeton : une copie circule.
    let refus = service
        .rafraichir(&ouverte.jetons.rafraichissement, None)
        .await;
    assert!(
        matches!(refus, Err(ErreurSession::SessionInvalide)),
        "un jeton de rafraîchissement déjà consommé a été accepté une seconde fois"
    );

    // **Et le jeton légitime tombe avec.** C'est le point : les deux porteurs sont déconnectés,
    // parce que rien ne permet de savoir lequel est le voleur.
    let refus_legitime = service
        .rafraichir(&renouvelee.jetons.rafraichissement, None)
        .await;
    assert!(
        matches!(refus_legitime, Err(ErreurSession::SessionInvalide)),
        "seul le jeton réutilisé a été révoqué : le voleur et la victime restent en course, et le \
         premier des deux gagne — sans qu'aucun des deux ne sache qu'il y a eu course"
    );
}

/// La réutilisation **écrit une entrée d'audit et émet un événement**.
///
/// Sans trace, l'incident serait invisible : la victime constaterait une déconnexion inexpliquée
/// et personne ne saurait qu'un jeton a circulé.
#[tokio::test]
async fn la_reutilisation_laisse_une_trace_au_registre_et_au_grand_livre() {
    let pool_owner = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool_owner, "CPT-01 — trace de réutilisation").await;
    let identifiant = identifiant_unique();

    commun::creer_compte(
        &pool_owner,
        jeu.tenant_id,
        "Tracé",
        &identifiant,
        commun::MOT_DE_PASSE_TEST,
        &[("proprietaire", Some(jeu.etablissement_id))],
    )
    .await;

    let service = commun::service_authentification(commun::pool_app().await);
    let ouverte = service
        .ouvrir(
            &identifiant,
            commun::MOT_DE_PASSE_TEST,
            Some(jeu.etablissement_id),
            None,
            &origine(5),
        )
        .await
        .expect("connexion");

    let _ = service
        .rafraichir(&ouverte.jetons.rafraichissement, None)
        .await
        .expect("premier rafraîchissement");
    let _ = service
        .rafraichir(&ouverte.jetons.rafraichissement, None)
        .await;

    // Le tenant est posé avant de compter : la politique d'isolation convertit
    // `current_setting('app.current_tenant', true)` en `uuid`, et hors transaction ayant posé le
    // réglage il vaut la chaîne vide — pas `NULL` —, ce qui échoue en `22P02`.
    let mut tx = pool_owner.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, jeu.tenant_id)
        .await
        .expect("pose du tenant");

    let entrees: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) AS "c!"
        FROM comptes.journal_audit
        WHERE tenant_id = $1
          AND type_action = 'suppression'
          AND cible_type = 'session'
          AND contexte ->> 'motif' = 'reutilisation_jeton_rafraichissement'
        "#,
        jeu.tenant_id
    )
    .fetch_one(&mut *tx)
    .await
    .expect("décompte des entrées d'audit");

    let evenements: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) AS "c!"
        FROM synchronisation.evenement_outbox
        WHERE tenant_id = $1 AND type_evenement = 'session.revoquee'
        "#,
        jeu.tenant_id
    )
    .fetch_one(&mut *tx)
    .await
    .expect("décompte des événements");

    tx.rollback().await.expect("rollback");

    assert_eq!(
        entrees, 1,
        "{entrees} entrée(s) d'audit pour une réutilisation de jeton. Sans trace, la victime \
         constate une déconnexion inexpliquée et personne ne sait qu'un jeton a circulé."
    );
    assert_eq!(
        evenements, 1,
        "{evenements} événement(s) `session.revoquee` au grand livre pour une réutilisation"
    );
}

// =================================================================================================
//  3 · Le changement de mot de passe coupe les autres
// =================================================================================================

/// **Changer son mot de passe révoque les autres sessions, immédiatement.**
///
/// « Les autres » et non « toutes » : celui qui change son mot de passe ne doit pas se déconnecter
/// lui-même en le faisant. Sans cette révocation, changer son mot de passe après un vol ne
/// reprendrait **rien** à celui qui détient déjà un jeton — la nouvelle valeur ne concerne que les
/// connexions à venir.
#[tokio::test]
async fn changer_de_mot_de_passe_revoque_les_autres_sessions() {
    let pool_owner = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool_owner, "CPT-01 — changement de secret").await;
    let identifiant = identifiant_unique();

    commun::creer_compte(
        &pool_owner,
        jeu.tenant_id,
        "Change son secret",
        &identifiant,
        commun::MOT_DE_PASSE_TEST,
        &[("proprietaire", Some(jeu.etablissement_id))],
    )
    .await;

    let service = commun::service_authentification(commun::pool_app().await);

    let courante = service
        .ouvrir(
            &identifiant,
            commun::MOT_DE_PASSE_TEST,
            Some(jeu.etablissement_id),
            Some("poste courant".to_owned()),
            &origine(6),
        )
        .await
        .expect("connexion courante");
    let autre = service
        .ouvrir(
            &identifiant,
            commun::MOT_DE_PASSE_TEST,
            Some(jeu.etablissement_id),
            Some("appareil oublié".to_owned()),
            &origine(7),
        )
        .await
        .expect("connexion autre");

    let revoquees = service
        .revoquer_les_autres(
            courante.compte_id,
            Some(courante.jetons.session_id),
            3600,
        )
        .await
        .expect("révocation des autres");

    assert_eq!(
        revoquees, 1,
        "{revoquees} session(s) révoquée(s) au lieu d'une : le changement de mot de passe doit \
         couper les autres appareils, et **seulement** les autres"
    );
    assert!(
        service
            .est_revoquee(autre.jetons.session_id)
            .await
            .expect("lecture"),
        "l'appareil oublié n'a pas été coupé : changer son mot de passe après un vol ne reprend \
         rien à celui qui détient déjà un jeton"
    );
    assert!(
        !service
            .est_revoquee(courante.jetons.session_id)
            .await
            .expect("lecture"),
        "l'appareil depuis lequel le mot de passe a été changé s'est déconnecté lui-même"
    );
}
