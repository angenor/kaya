//! **Porte P-14 — le registre des actions est de classe A.** Seconde entité A du produit.
//!
//! Classe A (cadrage §11.2, branche A4) : *append-only, commutative, sans contrainte d'unicité
//! métier, sans effet monétaire*. Deux propriétés en découlent, et ce fichier les prouve toutes
//! les deux — c'est le §0.7 des user stories qui les impose.
//!
//! | Propriété | Ce qu'elle signifie | Le test |
//! |---|---|---|
//! | **Rejeu** | Trois soumissions du même identifiant → **un** enregistrement | `trois_soumissions_du_meme_identifiant_font_un_enregistrement` |
//! | **Désordre** | Trois entrées dans les **six** ordres → **même état final** | `les_six_ordres_produisent_le_meme_etat_final` |
//!
//! # Les identifiants sont FIGÉS par permutation, jamais tirés au hasard
//!
//! C'est le point qu'on écrirait mal. Un test de désordre qui engendrerait de nouveaux UUID **à
//! chaque envoi** comparerait des jeux différents : les six états finaux seraient tous distincts,
//! et l'assertion « même état final » ne pourrait porter que sur le nombre de lignes — c'est-à-dire
//! sur rien. Chaque permutation tire donc ses trois identifiants **une fois**, et ses trois envois
//! permutent les mêmes.
//!
//! # Et ils sont figés PAR permutation, pas partagés entre les six — la nuance coûte cher
//!
//! La première rédaction partageait les trois mêmes UUID entre les six permutations, ce qui
//! paraissait plus rigoureux. Elle a échoué, et l'échec est instructif : **`journal_audit.id` est
//! une clé primaire GLOBALE, pas par tenant**. La première permutation insère ; les cinq autres
//! tombent sur `ON CONFLICT (id) DO NOTHING` — silencieusement, puisque c'est le comportement
//! voulu d'une entité de classe A — et leur registre reste vide sous leur propre tenant, que la
//! politique d'isolation leur rend seul visible.
//!
//! L'état comparé est donc le **contenu** des entrées (`contexte.rang`), pas leurs identifiants :
//! c'est lui qui dit « les trois mêmes actions ont été enregistrées », et c'est la seule chose que
//! la commutativité promet.
//!
//! # Pourquoi la classe A du registre ne contredit pas la classe C des opérations
//!
//! L'encadré du registre §5.2 le dit : « l'opération tracée garde sa propre classe ». Une ouverture
//! de tiroir se fait et se trace hors ligne — l'entrée d'audit part en file, l'opération aussi.
//! Une attribution de rôle, elle, est de classe C : elle ne se fait pas hors ligne, et sa trace
//! non plus, faute d'opération à tracer. **La classe de l'entrée ne relâche jamais celle de
//! l'opération**, elle dit seulement ce que l'entrée supporte quand l'opération, elle, est
//! permise.

mod commun;

use std::collections::BTreeSet;

// =================================================================================================
//  POURQUOI `tester_classe_a!` NE S'APPLIQUE PAS ICI — cycle 005
// =================================================================================================
//
// Ce n'est pas une lacune du portage, c'est un fait du produit, et il mérite d'être écrit plutôt
// que découvert par le prochain qui essaiera.
//
// La macro `tester_classe_a!` engendre ses tests **par HTTP** : elle envoie trois fois le même
// corps sur un endpoint d'écriture. **Le registre des actions n'en a aucun** — le contrat n'expose
// aucun point d'entrée d'écriture d'audit (research R-17 du cycle 003), et c'est délibéré : une
// entrée voyage toujours avec l'opération qu'elle trace, jamais seule.
//
// Instancier la macro ici aurait donc demandé de lui inventer un endpoint, c'est-à-dire de créer
// une opération que personne n'a spécifiée — ce que le principe X interdit. Les tests ci-dessous
// écrivent par le trait `JournalAudit`, qui est le chemin réel.
//
// Le contrôle `aucun_endpoint_d_ecriture_d_audit_n_est_apparu` plus bas garde cette raison : le
// jour où un endpoint existerait, il échouerait, et le portage deviendrait possible **et** dû.

use kaya_comptes::audit::{
    EntreeAudit, FiltresAudit, JournalAudit, JournalAuditPostgres, TypeActionAudit,
};
use serde_json::json;
use uuid::Uuid;

/// Écrit une entrée **dans sa propre transaction**, comme le ferait une opération tracée.
async fn tracer(
    pool: &sqlx::PgPool,
    tenant_id: Uuid,
    entree: EntreeAudit,
) -> Result<(), kaya_comptes::audit::ErreurAudit> {
    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, tenant_id)
        .await
        .expect("pose du tenant");

    JournalAuditPostgres.tracer(&mut tx, tenant_id, entree).await?;

    tx.commit().await.expect("commit");
    Ok(())
}

/// Une entrée d'essai, dont **seul l'identifiant varie**.
fn entree(id: Uuid, auteur: Uuid, cible: Uuid, rang: u8) -> EntreeAudit {
    EntreeAudit {
        id,
        etablissement_id: None,
        type_action: TypeActionAudit::ChangementRole,
        auteur_compte_id: auteur,
        cible_type: "compte".to_owned(),
        cible_id: Some(cible),
        // Le rang rend les trois entrées distinguables **à la relecture**, sans quoi l'assertion
        // d'ensemble ne verrait pas la différence entre « les trois » et « la même trois fois ».
        contexte: json!({ "role_code": "caissier", "sens": "attribution", "rang": rang }),
        horodatage_client: None,
    }
}

/// Relit toutes les entrées d'un auteur — l'**état final**, tel que `G4` le lira.
///
/// Rend les **rangs**, pas les identifiants : voir la note du commentaire de tête. Deux registres
/// portant les mêmes trois actions ont le même état final, quels que soient les UUID qui les
/// nomment.
async fn etat_final(pool: &sqlx::PgPool, tenant_id: Uuid, auteur: Uuid) -> Vec<i64> {
    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, tenant_id)
        .await
        .expect("pose du tenant");

    let page = kaya_comptes::audit::repository::lister(
        &mut tx,
        &FiltresAudit {
            auteur_compte_id: Some(auteur),
            ..Default::default()
        },
        None,
        100,
    )
    .await
    .expect("lecture");

    tx.rollback().await.expect("rollback");

    // **Comparé comme un ENSEMBLE TRIÉ**, pas comme une liste : l'ordre d'arrivée est indifférent
    // pour une entité commutative, et comparer des listes ferait échouer le test sur la propriété
    // même qu'il est censé démontrer.
    let mut etat: Vec<i64> = page
        .elements
        .iter()
        .map(|e| e.contexte["rang"].as_i64().unwrap_or(-1))
        .collect();
    etat.sort_unstable();
    etat
}

// =================================================================================================
//  1 · Rejeu — trois soumissions, un enregistrement
// =================================================================================================

/// **Trois soumissions du même identifiant produisent UN enregistrement.**
///
/// C'est ce qui rend le vidage d'une file locale inoffensif : un terminal qui perd son accusé de
/// réception réémet, et le registre ne double pas.
#[actix_web::test]
async fn trois_soumissions_du_meme_identifiant_font_un_enregistrement() {
    let pool_owner = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool_owner, "P-14 rejeu").await;
    let auteur = commun::compte_connecte(
        &pool_owner,
        jeu,
        "Auteur rejeu",
        &[("proprietaire", Some(jeu.etablissement_id))],
    )
    .await;

    let pool = commun::pool_app().await;
    let id = Uuid::now_v7();

    for _ in 0..3 {
        tracer(
            &pool,
            jeu.tenant_id,
            entree(id, auteur.compte_id, auteur.compte_id, 1),
        )
        .await
        .expect("un rejeu ne doit jamais échouer");
    }

    let etat = etat_final(&pool, jeu.tenant_id, auteur.compte_id).await;
    assert_eq!(etat.len(), 1, "trois soumissions ont produit {} lignes", etat.len());
    assert_eq!(etat, vec![1]);
}

/// **Un rejeu ne remplace pas la ligne d'origine.**
///
/// `ON CONFLICT DO NOTHING`, pas `DO UPDATE`. Un terminal qui réémettrait une entrée modifiée
/// réécrirait le registre — et un registre qu'on peut réécrire ne prouve rien. Le second contexte
/// est donc **ignoré**, silencieusement, ce qui est le comportement voulu d'une entité de classe A.
#[actix_web::test]
async fn un_rejeu_ne_reecrit_pas_l_entree_d_origine() {
    let pool_owner = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool_owner, "P-14 immuabilité au rejeu").await;
    let auteur = commun::compte_connecte(
        &pool_owner,
        jeu,
        "Auteur immuable",
        &[("proprietaire", Some(jeu.etablissement_id))],
    )
    .await;

    let pool = commun::pool_app().await;
    let id = Uuid::now_v7();

    tracer(&pool, jeu.tenant_id, entree(id, auteur.compte_id, auteur.compte_id, 1))
        .await
        .expect("première écriture");

    // Même identifiant, contexte différent — la tentative de réécriture.
    tracer(&pool, jeu.tenant_id, entree(id, auteur.compte_id, auteur.compte_id, 99))
        .await
        .expect("le rejeu est silencieux");

    let etat = etat_final(&pool, jeu.tenant_id, auteur.compte_id).await;
    assert_eq!(etat.len(), 1);
    assert_eq!(etat, vec![1], "le registre a été réécrit par un rejeu");
}

// =================================================================================================
//  2 · Désordre — six ordres, même état final
// =================================================================================================

/// **Les six ordres des trois mêmes entrées produisent le même état final.**
///
/// Une opération commutative n'a pas d'ordre d'arrivée à respecter : un terminal qui vide sa file
/// dans un ordre quelconque doit aboutir au même registre que celui qui la vide dans l'ordre.
#[actix_web::test]
async fn les_six_ordres_produisent_le_meme_etat_final() {
    let pool_owner = commun::pool_owner().await;
    let pool = commun::pool_app().await;

    const PERMUTATIONS: [[usize; 3]; 6] = [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ];

    let mut etats = Vec::new();

    for (rang_permutation, ordre) in PERMUTATIONS.iter().enumerate() {
        // Un tenant par permutation : les six états doivent être comparables entre eux, pas
        // s'additionner dans le même registre.
        // **Trois identifiants FIGÉS pour CETTE permutation**, tirés une fois avant ses trois
        // envois. Les tirer à chaque envoi ferait trois lignes distinctes au lieu de trois envois
        // des mêmes trois entrées ; les partager entre permutations les ferait entrer en conflit
        // sur la clé primaire globale (voir le commentaire de tête).
        let ids = [Uuid::now_v7(), Uuid::now_v7(), Uuid::now_v7()];

        let jeu = commun::creer_tenant(&pool_owner, &format!("P-14 désordre {rang_permutation}")).await;
        let auteur = commun::compte_connecte(
            &pool_owner,
            jeu,
            &format!("Auteur désordre {rang_permutation}"),
            &[("proprietaire", Some(jeu.etablissement_id))],
        )
        .await;

        for &position in ordre {
            tracer(
                &pool,
                jeu.tenant_id,
                entree(
                    ids[position],
                    auteur.compte_id,
                    auteur.compte_id,
                    position as u8 + 1,
                ),
            )
            .await
            .expect("écriture");
        }

        etats.push(etat_final(&pool, jeu.tenant_id, auteur.compte_id).await);
    }

    // **Comparaison d'ensembles triés** — l'ordre d'arrivée est ce qu'on teste, pas ce qu'on
    // compare.
    let distincts: BTreeSet<String> = etats.iter().map(|e| format!("{e:?}")).collect();

    assert_eq!(
        distincts.len(),
        1,
        "les six ordres ont produit {} états différents :\n{}",
        distincts.len(),
        distincts.into_iter().collect::<Vec<_>>().join("\n")
    );
    assert_eq!(etats[0].len(), 3, "les trois entrées doivent toutes être là");
}

// =================================================================================================
//  3 · La lecture filtrée — ce que `G4` demande
// =================================================================================================

/// **Les quatre filtres sont combinables**, et chacun réduit réellement.
#[actix_web::test]
async fn les_filtres_sont_combinables_et_reduisent_reellement() {
    let pool_owner = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool_owner, "P-14 filtres").await;
    let etb = jeu.etablissement_id;

    let koffi = commun::compte_connecte(&pool_owner, jeu, "Koffi filtres", &[("proprietaire", Some(etb))]).await;
    let adjoua = commun::compte_connecte(&pool_owner, jeu, "Adjoua filtres", &[("gerant", Some(etb))]).await;

    let pool = commun::pool_app().await;

    // Deux entrées de Koffi, une d'Adjoua — et deux types d'action différents.
    for (auteur, type_action) in [
        (koffi.compte_id, TypeActionAudit::ChangementRole),
        (koffi.compte_id, TypeActionAudit::Suppression),
        (adjoua.compte_id, TypeActionAudit::ChangementRole),
    ] {
        tracer(
            &pool,
            jeu.tenant_id,
            EntreeAudit {
                id: Uuid::now_v7(),
                etablissement_id: Some(etb),
                type_action,
                auteur_compte_id: auteur,
                cible_type: "compte".to_owned(),
                cible_id: Some(auteur),
                contexte: json!({}),
                horodatage_client: None,
            },
        )
        .await
        .expect("écriture");
    }

    let compter = |filtres: FiltresAudit| {
        let pool = pool.clone();
        async move {
            let mut tx = pool.begin().await.expect("transaction");
            kaya_etablissements::tenant_context::poser_tenant(&mut tx, jeu.tenant_id)
                .await
                .expect("pose du tenant");
            let page = kaya_comptes::audit::repository::lister(&mut tx, &filtres, None, 100)
                .await
                .expect("lecture");
            tx.rollback().await.expect("rollback");
            page.elements.len()
        }
    };

    // Sans filtre : les trois. C'est le **versant positif** — sans lui, les assertions suivantes
    // seraient vraies sur un registre vide.
    assert_eq!(compter(FiltresAudit { etablissement_id: Some(etb), ..Default::default() }).await, 3);

    // Par auteur : deux.
    assert_eq!(
        compter(FiltresAudit {
            etablissement_id: Some(etb),
            auteur_compte_id: Some(koffi.compte_id),
            ..Default::default()
        })
        .await,
        2
    );

    // Par type : deux.
    assert_eq!(
        compter(FiltresAudit {
            etablissement_id: Some(etb),
            type_action: Some(TypeActionAudit::ChangementRole),
            ..Default::default()
        })
        .await,
        2
    );

    // **Les deux combinés : une seule.** C'est la combinaison qui est le sujet de FR-037 — deux
    // filtres qui fonctionnent séparément peuvent très bien s'annuler quand on les cumule.
    assert_eq!(
        compter(FiltresAudit {
            etablissement_id: Some(etb),
            auteur_compte_id: Some(koffi.compte_id),
            type_action: Some(TypeActionAudit::ChangementRole),
            ..Default::default()
        })
        .await,
        1
    );

    // Une période qui n'englobe rien : zéro, sans erreur.
    let avant = time::OffsetDateTime::now_utc() - time::Duration::days(30);
    assert_eq!(
        compter(FiltresAudit {
            etablissement_id: Some(etb),
            jusqu_a: Some(avant),
            ..Default::default()
        })
        .await,
        0
    );
}

/// **La pagination par curseur ne saute ni ne répète aucune entrée.**
///
/// C'est ce qu'un `OFFSET` ne garantit pas : une entrée écrite entre deux pages décale tout, et la
/// dernière ligne de la page 1 réapparaît en tête de la page 2 — ou disparaît. Sur un registre
/// d'audit, une entrée sautée est exactement celle qu'on cherchait.
#[actix_web::test]
async fn le_curseur_parcourt_le_registre_sans_saut_ni_doublon() {
    let pool_owner = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool_owner, "P-14 curseur").await;
    let auteur = commun::compte_connecte(
        &pool_owner,
        jeu,
        "Auteur curseur",
        &[("proprietaire", Some(jeu.etablissement_id))],
    )
    .await;

    let pool = commun::pool_app().await;
    let mut ecrits = BTreeSet::new();

    for rang in 0..7u8 {
        let id = Uuid::now_v7();
        ecrits.insert(id);
        tracer(&pool, jeu.tenant_id, entree(id, auteur.compte_id, auteur.compte_id, rang))
            .await
            .expect("écriture");
    }

    // Parcours par pages de deux — trois pages pleines et une d'un seul élément.
    let filtres = FiltresAudit {
        auteur_compte_id: Some(auteur.compte_id),
        ..Default::default()
    };
    let mut lus: Vec<Uuid> = Vec::new();
    let mut curseur = None;

    loop {
        let mut tx = pool.begin().await.expect("transaction");
        kaya_etablissements::tenant_context::poser_tenant(&mut tx, jeu.tenant_id)
            .await
            .expect("pose du tenant");
        let page = kaya_comptes::audit::repository::lister(&mut tx, &filtres, curseur, 2)
            .await
            .expect("lecture");
        tx.rollback().await.expect("rollback");

        lus.extend(page.elements.iter().map(|e| e.id));
        match page.suivant {
            Some(suivant) => curseur = Some(suivant),
            None => break,
        }
    }

    assert_eq!(lus.len(), 7, "le parcours a rendu {} entrées sur 7", lus.len());
    let uniques: BTreeSet<Uuid> = lus.iter().copied().collect();
    assert_eq!(uniques.len(), 7, "une entrée a été rendue deux fois");
    assert_eq!(uniques, ecrits, "le parcours n'a pas rendu les entrées écrites");

    // Et l'ordre est bien **décroissant sur l'horodatage d'autorité** : le registre se lit du plus
    // récent au plus ancien, ce qui est ce qu'un propriétaire cherche.
    let mut decroissant = lus.clone();
    decroissant.sort_by(|a, b| b.cmp(a));
    assert_eq!(lus, decroissant, "l'ordre du parcours n'est pas celui du registre");
}

// =================================================================================================
//  Le contrôle qui garde la raison ci-dessus — cycle 005
// =================================================================================================

/// **Aucun endpoint d'écriture d'audit n'existe** — et c'est ce qui explique l'absence de macro.
///
/// Si un tel endpoint apparaissait, deux choses deviendraient vraies en même temps : `tester_classe_a!`
/// s'appliquerait à cette entité, et il faudrait l'instancier. Ce test échoue à ce moment-là,
/// c'est-à-dire au seul moment où quelqu'un peut y penser.
#[test]
fn aucun_endpoint_d_ecriture_d_audit_n_est_apparu() {
    let contrat = kaya_api::application::contrat_complet();

    let ecritures: Vec<String> = contrat
        .paths
        .paths
        .iter()
        .filter(|(chemin, _)| {
            let c = chemin.to_lowercase();
            c.contains("audit") || c.contains("registre")
        })
        .filter(|(_, item)| item.post.is_some() || item.put.is_some() || item.delete.is_some())
        .map(|(chemin, _)| chemin.clone())
        .collect();

    assert!(
        ecritures.is_empty(),
        "un endpoint d'ÉCRITURE d'audit est apparu : {ecritures:?}\n\n\
         Deux conséquences, dans cet ordre :\n\
         \n\
         1. **Vérifier que c'est voulu.** Le contrat n'en exposait aucun (research R-17) parce \
            qu'une entrée d'audit voyage avec l'opération qu'elle trace, jamais seule. Un endpoint \
            d'écriture permettrait d'écrire au registre SANS l'action correspondante — c'est-à-dire \
            de fabriquer une trace.\n\
         2. Si c'est voulu, **instancier `tester_classe_a!` sur cette entité** : la macro engendre \
            le rejeu triple et les six ordres par HTTP, ce qui n'était pas possible jusqu'ici."
    );
}
