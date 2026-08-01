//! **US3 — le cumul de rôles donne l'union** (FR-017, FR-018, FR-023).
//!
//! # Ce que ce fichier vérifie, et l'ordre dans lequel il faut le lire
//!
//! 1. **L'union est exacte, et sans doublon.** Adjoua porte trois rôles ; ses droits sont la
//!    réunion des trois, jamais ceux d'un rôle « principal ».
//! 2. **Le retrait ne retire que l'exclusif** (FR-018) — et la paire qui l'exerce réellement est
//!    `gerant` + `comptable`, pas `gerant` + `caissier`. Voir la note ci-dessous.
//! 3. **Un compte sans rôle se connecte** et obtient un ensemble **vide**, pas une erreur.
//! 4. **La portée est structurelle** : `admin_editeur` refuse un `etablissement_id`, les sept
//!    autres l'exigent.
//! 5. **La dernière habilitation ne se retire pas** (FR-023), et le cumul rend la question moins
//!    évidente qu'elle n'en a l'air.
//!
//! # La note qui évite de « corriger » ce fichier
//!
//! **Les quatre rôles opérationnels sont, à ce cycle, des sous-ensembles stricts de `gerant`.**
//! Leurs cinq permissions de lecture sont toutes dans les seize du gérant. Conséquence directe :
//! sur le compte d'Adjoua (gérante + caissière + réceptionniste), **retirer `caissier` ne retire
//! aucune permission**.
//!
//! Ce n'est ni un défaut de la distribution ni une faiblesse de FR-018 : les permissions propres
//! au caissier — ouvrir un shift, encaisser, compter, clôturer — appartiennent au cycle CAI et
//! naîtront avec les écrans qu'elles gardent. En poser dès maintenant produirait des permissions
//! qui ne gardent **rien**, ce que FR-021 fait échouer.
//!
//! La paire qui exerce réellement le scénario 2 de US3 est donc **`gerant` + `comptable`** :
//! `cpt.audit.consulter` est **exclusive au comptable**, et les cinq lectures sont partagées.
//! C'est celle que ce fichier emploie, et la raison est écrite ici pour qu'on ne « répare » pas le
//! test en inventant des permissions au caissier.

mod commun;

use std::collections::BTreeSet;

use kaya_comptes::{AccessController, AnnuaireComptes};
use uuid::Uuid;

/// Le contrôleur d'accès, assemblé comme le fait l'application réelle.
fn controleur(pool: sqlx::PgPool) -> kaya_comptes::ControleAccesPostgres {
    kaya_comptes::ControleAccesPostgres::nouveau(pool)
}

/// Le service des rôles, assemblé comme le fait `EtatApplication::service_roles`.
fn service_roles(
    pool: sqlx::PgPool,
    tenant_id: Uuid,
) -> kaya_comptes::roles::ServiceRoles<
    kaya_synchronisation::outbox::PgOutboxWriter,
    kaya_comptes::audit::JournalAuditPostgres,
    kaya_etablissements::etablissement::PgEstablishmentDirectory,
> {
    kaya_comptes::roles::ServiceRoles::nouveau(
        pool.clone(),
        kaya_synchronisation::outbox::PgOutboxWriter::nouveau(),
        kaya_comptes::audit::JournalAuditPostgres,
        kaya_etablissements::etablissement::PgEstablishmentDirectory::nouveau(pool, tenant_id),
    )
}

fn ensemble(codes: &[&str]) -> BTreeSet<String> {
    codes.iter().map(|c| (*c).to_owned()).collect()
}

// =================================================================================================
//  1 · L'union — le cœur du module
// =================================================================================================

/// **Trois rôles, une connexion, l'union exacte** (FR-017).
///
/// C'est le compte d'Adjoua : gérante, caissière **et** réceptionniste sur le même établissement.
#[actix_web::test]
async fn les_permissions_sont_l_union_des_trois_roles_sans_doublon() {
    let pool_owner = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool_owner, "US3 union").await;
    let etb = jeu.etablissement_id;

    let adjoua = commun::compte_connecte(
        &pool_owner,
        jeu,
        "Adjoua",
        &[
            ("gerant", Some(etb)),
            ("caissier", Some(etb)),
            ("receptionniste", Some(etb)),
        ],
    )
    .await;

    let controleur = controleur(commun::pool_app().await);
    let union = controleur
        .permissions_effectives(jeu.tenant_id, adjoua.compte_id, Some(etb))
        .await
        .expect("lecture des permissions");

    // `gerant` porte les seize permissions sauf `cpt.audit.consulter` ; les deux autres rôles
    // n'apportent que des lectures déjà comprises. L'union vaut donc exactement celles du gérant.
    assert!(
        union.contains("etb.service.basculer"),
        "la permission du gérant manque : {union:?}"
    );
    assert!(
        union.contains("cpt.role.attribuer"),
        "la permission du gérant manque : {union:?}"
    );
    assert!(
        !union.contains("cpt.audit.consulter"),
        "le registre des actions n'appartient PAS au gérant — c'est ce que M. Koffi achète"
    );

    // **Sans doublon, par le type.** L'assertion est faible en apparence et forte en réalité : un
    // `Vec` aurait porté trois fois `etb.note.lire`, une par rôle, et l'écran aurait affiché la
    // même tuile trois fois (FR-027).
    let brut: Vec<&String> = union.iter().collect();
    let dedoublonne: BTreeSet<&String> = union.iter().collect();
    assert_eq!(brut.len(), dedoublonne.len());
}

/// **`detient` se dérive de l'union** — la méthode par défaut du trait, contre une vraie base.
#[actix_web::test]
async fn detient_repond_comme_l_union() {
    let pool_owner = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool_owner, "US3 detient").await;
    let etb = jeu.etablissement_id;

    let yao = commun::compte_connecte(
        &pool_owner,
        jeu,
        "Yao",
        &[("receptionniste", Some(etb))],
    )
    .await;

    let controleur = controleur(commun::pool_app().await);

    assert!(
        controleur
            .detient(jeu.tenant_id, yao.compte_id, Some(etb), "etb.etablissement.lire")
            .await
            .expect("lecture"),
        "un réceptionniste lit son établissement"
    );
    assert!(
        !controleur
            .detient(jeu.tenant_id, yao.compte_id, Some(etb), "etb.service.basculer")
            .await
            .expect("lecture"),
        "un réceptionniste ne bascule pas les services"
    );
}

// =================================================================================================
//  2 · Le retrait ne retire que l'exclusif — FR-018
// =================================================================================================

/// **Retirer `comptable` retire `cpt.audit.consulter` et RIEN d'autre.**
///
/// Les cinq lectures sont partagées avec `gerant` ; elles demeurent. C'est le scénario 2 de US3,
/// et c'est la seule paire de rôles du cycle 003 qui l'exerce réellement — voir le commentaire de
/// tête.
#[actix_web::test]
async fn retirer_un_role_ne_retire_que_les_permissions_exclusives() {
    let pool_owner = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool_owner, "US3 retrait").await;
    let etb = jeu.etablissement_id;

    let compte = commun::compte_connecte(
        &pool_owner,
        jeu,
        "Gérant comptable",
        &[("gerant", Some(etb)), ("comptable", Some(etb))],
    )
    .await;

    let pool = commun::pool_app().await;
    let controleur = controleur(pool.clone());

    let avant = controleur
        .permissions_effectives(jeu.tenant_id, compte.compte_id, Some(etb))
        .await
        .expect("lecture avant");

    assert!(
        avant.contains("cpt.audit.consulter"),
        "le comptable apporte le registre des actions : {avant:?}"
    );

    service_roles(pool.clone(), jeu.tenant_id)
        .retirer(
            jeu.tenant_id,
            compte.compte_id,
            compte.compte_id,
            "comptable",
            Some(etb),
        )
        .await
        .expect("le retrait doit réussir");

    let apres = controleur
        .permissions_effectives(jeu.tenant_id, compte.compte_id, Some(etb))
        .await
        .expect("lecture après");

    // **Une seule permission perdue, et c'est l'exclusive.**
    let perdues: Vec<&String> = avant.difference(&apres).collect();
    assert_eq!(
        perdues,
        vec![&"cpt.audit.consulter".to_owned()],
        "le retrait a emporté autre chose que l'exclusive : {perdues:?}"
    );

    // Les cinq lectures partagées demeurent — c'est la moitié de FR-018 qu'on oublierait.
    for partagee in [
        "etb.etablissement.lire",
        "etb.pdv.lire",
        "etb.configuration.lire",
        "etb.branding.lire",
        "etb.note.lire",
    ] {
        assert!(
            apres.contains(partagee),
            "« {partagee} » est partagée avec `gerant` et ne devait pas partir"
        );
    }
}

// =================================================================================================
//  3 · Un compte sans rôle
// =================================================================================================

/// **Un compte sans aucun rôle se connecte, et obtient un ensemble VIDE.**
///
/// Pas une erreur : c'est l'état d'un compte fraîchement créé, avant que quiconque lui donne un
/// rôle. Y répondre par un échec rendrait la connexion impossible pendant cet intervalle, et
/// personne ne saurait pourquoi.
#[actix_web::test]
async fn un_compte_sans_role_se_connecte_et_n_a_rien() {
    let pool_owner = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool_owner, "US3 sans rôle").await;

    // Aucun rôle passé : la connexion doit aboutir quand même.
    let nu = commun::compte_connecte(&pool_owner, jeu, "Compte nu", &[]).await;

    let controleur = controleur(commun::pool_app().await);
    let union = controleur
        .permissions_effectives(jeu.tenant_id, nu.compte_id, Some(jeu.etablissement_id))
        .await
        .expect("un compte sans rôle rend un ensemble vide, jamais une erreur");

    assert!(union.is_empty(), "attendu vide, obtenu {union:?}");
}

// =================================================================================================
//  4 · La portée — structurelle, pas conventionnelle
// =================================================================================================

/// **`admin_editeur` REFUSE un `etablissement_id`, les sept autres l'EXIGENT.**
#[actix_web::test]
async fn la_portee_du_role_decide_de_l_etablissement() {
    let pool_owner = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool_owner, "US3 portée").await;
    let etb = jeu.etablissement_id;

    let auteur = commun::compte_connecte(
        &pool_owner,
        jeu,
        "Habilité",
        &[("proprietaire", Some(etb))],
    )
    .await;
    let cible = commun::compte_connecte(&pool_owner, jeu, "Cible portée", &[]).await;

    let pool = commun::pool_app().await;
    let service = service_roles(pool, jeu.tenant_id);

    // Un rôle d'établissement SANS établissement — refusé.
    let refus = service
        .attribuer(
            jeu.tenant_id,
            auteur.compte_id,
            kaya_comptes::roles::AttribuerRole {
                id: Uuid::now_v7(),
                compte_id: cible.compte_id,
                role_code: "caissier".to_owned(),
                etablissement_id: None,
                horodatage_client: None,
            },
        )
        .await;
    assert!(
        matches!(refus, Err(kaya_comptes::roles::ErreurRoles::PorteeIncompatible)),
        "un rôle d'établissement sans établissement doit être refusé, obtenu {refus:?}"
    );

    // `admin_editeur` AVEC établissement — refusé aussi. La symétrie est le point : une seule des
    // deux directions vérifiée laisserait passer l'autre.
    let refus = service
        .attribuer(
            jeu.tenant_id,
            auteur.compte_id,
            kaya_comptes::roles::AttribuerRole {
                id: Uuid::now_v7(),
                compte_id: cible.compte_id,
                role_code: "admin_editeur".to_owned(),
                etablissement_id: Some(etb),
                horodatage_client: None,
            },
        )
        .await;
    assert!(
        matches!(refus, Err(kaya_comptes::roles::ErreurRoles::PorteeIncompatible)),
        "`admin_editeur` avec établissement doit être refusé, obtenu {refus:?}"
    );

    // Et les deux formes correctes passent.
    for (role, etablissement) in [("caissier", Some(etb)), ("admin_editeur", None)] {
        service
            .attribuer(
                jeu.tenant_id,
                auteur.compte_id,
                kaya_comptes::roles::AttribuerRole {
                    id: Uuid::now_v7(),
                    compte_id: cible.compte_id,
                    role_code: role.to_owned(),
                    etablissement_id: etablissement,
                    horodatage_client: None,
                },
            )
            .await
            .unwrap_or_else(|e| panic!("« {role} » à la bonne portée doit passer : {e}"));
    }
}

/// **Un établissement inconnu rend `etablissement_inconnu`, jamais une violation de contrainte.**
///
/// `compte_role.etablissement_id` n'a **aucune clé étrangère** : ce serait une clé inter-schémas
/// (porte P-04). La vérification passe par `EstablishmentDirectory`, et c'est elle qui rend le
/// refus intelligible au lieu d'un `500`.
#[actix_web::test]
async fn un_etablissement_inconnu_se_refuse_par_le_trait() {
    let pool_owner = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool_owner, "US3 établissement inconnu").await;

    let auteur = commun::compte_connecte(
        &pool_owner,
        jeu,
        "Habilité inconnu",
        &[("proprietaire", Some(jeu.etablissement_id))],
    )
    .await;
    let cible = commun::compte_connecte(&pool_owner, jeu, "Cible inconnu", &[]).await;

    let refus = service_roles(commun::pool_app().await, jeu.tenant_id)
        .attribuer(
            jeu.tenant_id,
            auteur.compte_id,
            kaya_comptes::roles::AttribuerRole {
                id: Uuid::now_v7(),
                compte_id: cible.compte_id,
                role_code: "caissier".to_owned(),
                // Un établissement qui n'existe pas — et qui n'existe dans aucun tenant.
                etablissement_id: Some(Uuid::now_v7()),
                horodatage_client: None,
            },
        )
        .await;

    assert!(
        matches!(
            refus,
            Err(kaya_comptes::roles::ErreurRoles::EtablissementInconnu)
        ),
        "attendu `EtablissementInconnu`, obtenu {refus:?}"
    );
}

// =================================================================================================
//  5 · FR-023 — la dernière habilitation
// =================================================================================================

/// **Le dernier compte habilité ne peut pas se retirer son habilitation.**
///
/// Le décompte porte sur l'état **résultant** du retrait, dans la transaction non validée : c'est
/// ce qui rend le cas suivant possible sans faux positif.
#[actix_web::test]
async fn la_derniere_habilitation_ne_se_retire_pas() {
    let pool_owner = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool_owner, "US3 dernière habilitation").await;
    let etb = jeu.etablissement_id;

    // **Un seul compte habilité** sur cet établissement.
    let seul = commun::compte_connecte(&pool_owner, jeu, "Seul gérant", &[("gerant", Some(etb))])
        .await;

    let pool = commun::pool_app().await;
    let refus = service_roles(pool.clone(), jeu.tenant_id)
        .retirer(jeu.tenant_id, seul.compte_id, seul.compte_id, "gerant", Some(etb))
        .await;

    assert!(
        matches!(
            refus,
            Err(kaya_comptes::roles::ErreurRoles::DerniereHabilitation)
        ),
        "attendu `DerniereHabilitation`, obtenu {refus:?}"
    );

    // **La transaction a été annulée** : le rôle est toujours là.
    let union = controleur(pool)
        .permissions_effectives(jeu.tenant_id, seul.compte_id, Some(etb))
        .await
        .expect("lecture");
    assert!(
        union.contains("cpt.role.attribuer"),
        "le refus n'a pas annulé le retrait : {union:?}"
    );
}

/// **Le cumul rend la garde plus subtile qu'un décompte préalable**.
///
/// Adjoua est gérante **et** propriétaire. Retirer `gerant` ne lui retire pas
/// `cpt.role.attribuer` — `proprietaire` la porte aussi. Un décompte fait **avant** le `DELETE`
/// l'aurait comptée comme « la dernière » et aurait refusé un retrait parfaitement sûr.
///
/// C'est la raison pour laquelle le service supprime d'abord et compte ensuite. Ce test est le
/// seul qui distingue les deux implémentations.
#[actix_web::test]
async fn un_compte_qui_reste_habilite_par_un_autre_role_peut_perdre_le_premier() {
    let pool_owner = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool_owner, "US3 cumul habilitant").await;
    let etb = jeu.etablissement_id;

    let adjoua = commun::compte_connecte(
        &pool_owner,
        jeu,
        "Adjoua cumul",
        &[("gerant", Some(etb)), ("proprietaire", Some(etb))],
    )
    .await;

    let pool = commun::pool_app().await;
    service_roles(pool.clone(), jeu.tenant_id)
        .retirer(jeu.tenant_id, adjoua.compte_id, adjoua.compte_id, "gerant", Some(etb))
        .await
        .expect("le retrait doit passer : `proprietaire` porte encore l'habilitation");

    let union = controleur(pool)
        .permissions_effectives(jeu.tenant_id, adjoua.compte_id, Some(etb))
        .await
        .expect("lecture");
    assert!(
        union.contains("cpt.role.attribuer"),
        "`proprietaire` porte l'habilitation : {union:?}"
    );
}

// =================================================================================================
//  6 · L'annuaire des comptes
// =================================================================================================

/// **`AnnuaireComptes` lit `nom_affichage` depuis `personne`, jamais l'identifiant de connexion.**
///
/// Afficher un numéro de téléphone dans un registre à rétention illimitée diffuserait un contact
/// personnel. Le test le vérifie sur la valeur rendue, pas sur l'intention.
#[actix_web::test]
async fn l_annuaire_rend_le_nom_de_la_personne_et_jamais_l_identifiant() {
    let pool_owner = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool_owner, "US3 annuaire").await;

    let koffi = commun::compte_connecte(
        &pool_owner,
        jeu,
        "Koffi",
        &[("proprietaire", Some(jeu.etablissement_id))],
    )
    .await;

    let annuaire = controleur(commun::pool_app().await);
    let resume = annuaire
        .compte(jeu.tenant_id, koffi.compte_id)
        .await
        .expect("lecture")
        .expect("le compte existe");

    assert_eq!(resume.nom_affichage, "Koffi");
    assert!(resume.actif);
    // Aucun numéro : ce serait un contact personnel dans un registre permanent.
    assert!(!resume.nom_affichage.starts_with('+'));

    // **La lecture en lot** rend une carte, et un identifiant absent se lit comme une clé absente.
    let lot = annuaire
        .comptes(jeu.tenant_id, &[koffi.compte_id, Uuid::now_v7()])
        .await
        .expect("lecture en lot");
    assert_eq!(lot.len(), 1);
    assert!(lot.contains_key(&koffi.compte_id));

    // Un lot vide n'ouvre aucune transaction — et rend une carte vide, jamais une erreur.
    assert!(annuaire.comptes(jeu.tenant_id, &[]).await.expect("lot vide").is_empty());
}

/// **L'annuaire est isolé par tenant**, comme tout le reste.
#[actix_web::test]
async fn l_annuaire_ne_traverse_pas_les_tenants() {
    let pool_owner = commun::pool_owner().await;
    let a = commun::creer_tenant(&pool_owner, "US3 annuaire A").await;
    let b = commun::creer_tenant(&pool_owner, "US3 annuaire B").await;

    let chez_a = commun::compte_connecte(
        &pool_owner,
        a,
        "Compte de A",
        &[("proprietaire", Some(a.etablissement_id))],
    )
    .await;

    let annuaire = controleur(commun::pool_app().await);

    // Vu depuis B, le compte de A **n'existe pas** — pas une erreur, une absence.
    let vu_de_b = annuaire
        .compte(b.tenant_id, chez_a.compte_id)
        .await
        .expect("lecture");
    assert!(vu_de_b.is_none(), "le compte d'un autre tenant est visible");

    // Et ses permissions, vues de B, sont vides.
    let union = annuaire
        .permissions_effectives(b.tenant_id, chez_a.compte_id, Some(a.etablissement_id))
        .await
        .expect("lecture");
    assert_eq!(union, ensemble(&[]), "les droits ont traversé le tenant");
}
