//! **Porte P-05** — toute transition d'état émet un événement **dans sa transaction**.
//!
//! La porte a deux niveaux, et ce fichier vérifie le second :
//!
//!   (a) la **signature** d'`OutboxWriter::ecrire` rend l'écriture hors transaction impossible à
//!       compiler — garantie statique, rien à tester ;
//!   (b) **après chaque mutation exposée, un événement existe ; après un rollback provoqué, ni
//!       ligne métier ni événement** — c'est ce qui suit.
//!
//! Le point (b) est le seul qui puisse encore échouer une fois (a) acquis : rien n'empêche
//! d'oublier l'appel. Ce qui ne peut pas arriver, en revanche, c'est que la ligne soit écrite et
//! l'événement perdu — et c'est précisément ce que le test de rollback constate.

mod commun;

use uuid::Uuid;

use kaya_etablissements::note::{CreerNote, ServiceNote};
use kaya_synchronisation::outbox::PgOutboxWriter;
use kaya_synchronisation::{EvenementAEcrire, OutboxWriter};

/// Après une création, l'événement existe — même agrégat, même identifiant.
#[tokio::test]
async fn p05_toute_creation_laisse_un_evenement() {
    let pool_owner = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool_owner, "P-05 création").await;

    let service = ServiceNote::nouveau(commun::pool_app().await, PgOutboxWriter::nouveau());
    let note_id = Uuid::now_v7();

    service
        .creer(
            jeu.tenant_id,
            CreerNote {
                id: note_id,
                etablissement_id: jeu.etablissement_id,
                auteur_compte_id: Uuid::now_v7(),
                texte: "Le climatiseur de la 7 fuit.".to_owned(),
                horodatage_client: None,
            },
        )
        .await
        .expect("création de la note");

    let mut tx = pool_owner.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, jeu.tenant_id)
        .await
        .expect("pose du tenant");

    let evenement = sqlx::query!(
        r#"
        SELECT type_evenement, agregat, version_schema, payload
        FROM synchronisation.evenement_outbox
        WHERE agregat_id = $1
        "#,
        note_id
    )
    .fetch_optional(&mut *tx)
    .await
    .expect("lecture du grand livre");

    let evenement = evenement.expect(
        "aucun événement pour une note créée : la transition d'état n'est pas tracée, et le grand \
         livre ne permettra pas de reconstituer ce qui s'est passé",
    );

    assert_eq!(evenement.type_evenement, "note_etablissement.creee");
    assert_eq!(evenement.agregat, "note_etablissement");
    assert_eq!(evenement.version_schema, 1);
    assert_eq!(
        evenement.payload["texte"], "Le climatiseur de la 7 fuit.",
        "la charge utile doit être dénormalisée : le texte en clair, pas un renvoi vers la table"
    );

    tx.rollback().await.expect("rollback");
}

/// **Après un rollback provoqué : ni ligne métier, ni événement.**
///
/// Le test écrit les deux dans une transaction, puis annule. C'est le scénario qu'une
/// implémentation naïve raterait — celle qui ouvrirait une seconde transaction pour l'événement
/// « pour ne pas alourdir la première ». La ligne disparaîtrait, l'événement resterait, et le
/// grand livre affirmerait une transition qui n'a jamais eu lieu.
#[tokio::test]
async fn p05_apres_un_rollback_ni_ligne_ni_evenement() {
    let pool_owner = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool_owner, "P-05 rollback").await;

    let pool = commun::pool_app().await;
    let note_id = Uuid::now_v7();
    let evenement_id = Uuid::now_v7();

    // Transaction menée à la main : le service commit toujours, or c'est justement l'absence de
    // commit qu'on veut éprouver.
    {
        let mut tx = pool.begin().await.expect("transaction");
        kaya_etablissements::tenant_context::poser_tenant(&mut tx, jeu.tenant_id)
            .await
            .expect("pose du tenant");

        sqlx::query!(
            r#"
            INSERT INTO etablissements.note_etablissement
                (id, tenant_id, etablissement_id, auteur_compte_id, texte)
            VALUES ($1, $2, $3, $4, 'note annulée par rollback')
            "#,
            note_id,
            jeu.tenant_id,
            jeu.etablissement_id,
            Uuid::now_v7(),
        )
        .execute(&mut *tx)
        .await
        .expect("insertion de la note");

        PgOutboxWriter::nouveau()
            .ecrire(
                &mut tx,
                EvenementAEcrire {
                    id: evenement_id,
                    tenant_id: jeu.tenant_id,
                    etablissement_id: Some(jeu.etablissement_id),
                    type_evenement: "note_etablissement.creee".to_owned(),
                    agregat: "note_etablissement".to_owned(),
                    agregat_id: note_id,
                    version_schema: 1,
                    payload: serde_json::json!({ "texte": "note annulée par rollback" }),
                },
            )
            .await
            .expect("écriture de l'événement");

        // Rollback — la transaction entière disparaît.
        tx.rollback().await.expect("rollback");
    }

    let mut tx = pool_owner.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, jeu.tenant_id)
        .await
        .expect("pose du tenant");

    let notes: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "c!" FROM etablissements.note_etablissement WHERE id = $1"#,
        note_id
    )
    .fetch_one(&mut *tx)
    .await
    .expect("comptage des notes");

    let evenements: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "c!" FROM synchronisation.evenement_outbox WHERE id = $1"#,
        evenement_id
    )
    .fetch_one(&mut *tx)
    .await
    .expect("comptage des événements");

    assert_eq!(notes, 0, "la note a survécu au rollback");
    assert_eq!(
        evenements, 0,
        "l'événement a survécu au rollback alors que la ligne métier a disparu. Le grand livre \
         affirmerait une transition qui n'a jamais eu lieu — c'est exactement ce que la signature \
         d'OutboxWriter::ecrire est censée rendre impossible."
    );

    tx.rollback().await.expect("rollback");
}

/// La séquence est **monotone par établissement**, et les trous sont acceptés.
///
/// R-07 le dit explicitement : les séquences PostgreSQL ne sont pas transactionnelles, un
/// rollback laisse un trou, et **c'est voulu**. Le test vérifie la propriété réellement exigée —
/// la croissance stricte — et non la continuité, qui imposerait un verrou par établissement sur
/// le chemin d'écriture le plus chaud du produit.
#[tokio::test]
async fn la_sequence_est_strictement_croissante_par_etablissement() {
    let pool_owner = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool_owner, "P-05 séquence").await;

    let service = ServiceNote::nouveau(commun::pool_app().await, PgOutboxWriter::nouveau());
    for index in 0..5 {
        service
            .creer(
                jeu.tenant_id,
                CreerNote {
                    id: Uuid::now_v7(),
                    etablissement_id: jeu.etablissement_id,
                    auteur_compte_id: Uuid::now_v7(),
                    texte: format!("note {index}"),
                    horodatage_client: None,
                },
            )
            .await
            .expect("création");
    }

    let mut tx = pool_owner.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, jeu.tenant_id)
        .await
        .expect("pose du tenant");

    let sequences: Vec<i64> = sqlx::query_scalar!(
        r#"
        SELECT sequence_etablissement
        FROM synchronisation.evenement_outbox
        WHERE etablissement_id = $1
        ORDER BY id
        "#,
        jeu.etablissement_id
    )
    .fetch_all(&mut *tx)
    .await
    .expect("lecture des séquences");

    assert_eq!(sequences.len(), 5);
    for paire in sequences.windows(2) {
        assert!(
            paire[1] > paire[0],
            "la séquence n'est pas strictement croissante : {sequences:?}. C'est elle qui \
             permettra à un nœud de site de détecter qu'il lui manque un événement."
        );
    }

    tx.rollback().await.expect("rollback");
}

// =================================================================================================
//  Cycle 002 — un événement par transition, et AUCUN sur rejeu
// =================================================================================================

/// Lit les types d'événements écrits pour un agrégat donné, dans l'ordre de séquence.
async fn types_evenements(
    pool_owner: &sqlx::PgPool,
    tenant_id: Uuid,
    agregat_id: Uuid,
) -> Vec<String> {
    let mut tx = pool_owner.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, tenant_id)
        .await
        .expect("pose du tenant");

    let types = sqlx::query_scalar!(
        r#"
        SELECT type_evenement
        FROM synchronisation.evenement_outbox
        WHERE agregat_id = $1
        ORDER BY sequence_etablissement
        "#,
        agregat_id
    )
    .fetch_all(&mut *tx)
    .await
    .expect("lecture du grand livre");

    tx.rollback().await.expect("rollback");
    types
}

/// **`etablissement.cree` — un événement, et un seul, malgré trois envois.**
///
/// Un rejeu ne produit **aucun** nouvel événement. L'émettre à chaque tentative ferait du grand
/// livre le journal des tentatives réseau du terminal, et non celui des transitions d'état : la
/// reconstitution compterait trois fois un établissement créé une fois.
#[tokio::test]
async fn p05_creation_d_etablissement_un_seul_evenement_malgre_le_rejeu() {
    use kaya_etablissements::etablissement::{CreerEtablissement, ServiceEtablissement};

    let pool_owner = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool_owner, "P-05 création établissement").await;
    let service = ServiceEtablissement::nouveau(commun::pool_app().await, PgOutboxWriter::nouveau());

    let id = Uuid::now_v7();
    let demande = || CreerEtablissement {
        id,
        nom: "Établissement P-05".to_owned(),
        juridiction: "CI".to_owned(),
        classement: kaya_etablissements::Classement::NonClasse,
        commune: "Abengourou".to_owned(),
        fuseau_horaire: "Africa/Abidjan".to_owned(),
        devise: "XOF".to_owned(),
        adresse: None,
        ncc: None,
    };

    for passage in 1..=3 {
        service
            .creer(jeu.tenant_id, demande())
            .await
            .unwrap_or_else(|e| panic!("passage {passage} : {e}"));
    }

    let types = types_evenements(&pool_owner, jeu.tenant_id, id).await;
    assert_eq!(
        types,
        vec!["etablissement.cree"],
        "trois envois du même identifiant ont produit {} événement(s) au lieu d'un seul : le \
         grand livre enregistrerait les tentatives réseau et non les transitions d'état",
        types.len()
    );
}

/// **Les trois types de modification, chacun émis pour sa propre raison.**
///
/// `classement_change` et `fuseau_change` pourraient tenir dans `etablissement.modifie` — et c'est
/// justement le problème : ils y seraient noyés. Le classement décide du barème de la taxe de
/// nuitée, le fuseau réinterprète tout regroupement par journée locale. Les retrouver dans le
/// grand livre ne doit pas demander de relire toutes les modifications.
#[tokio::test]
async fn p05_les_changements_sensibles_ont_leur_propre_type() {
    use kaya_etablissements::etablissement::{
        CreerEtablissement, ModifierEtablissement, ServiceEtablissement,
    };

    let pool_owner = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool_owner, "P-05 modification").await;
    let service = ServiceEtablissement::nouveau(commun::pool_app().await, PgOutboxWriter::nouveau());

    let id = Uuid::now_v7();
    service
        .creer(
            jeu.tenant_id,
            CreerEtablissement {
                id,
                nom: "Avant".to_owned(),
                juridiction: "CI".to_owned(),
                classement: kaya_etablissements::Classement::NonClasse,
                commune: "Abengourou".to_owned(),
                fuseau_horaire: "Africa/Abidjan".to_owned(),
                devise: "XOF".to_owned(),
                adresse: None,
                ncc: None,
            },
        )
        .await
        .expect("création");

    // Une seule requête touche les trois : nom (modifie), classement, fuseau.
    let resultat = service
        .modifier(
            jeu.tenant_id,
            id,
            ModifierEtablissement {
                nom: Some("Après".to_owned()),
                classement: Some(kaya_etablissements::Classement::Etoiles(3)),
                fuseau_horaire: Some("Africa/Accra".to_owned()),
                ..Default::default()
            },
        )
        .await
        .expect("modification");

    assert_eq!(
        resultat.avertissement,
        Some("fuseau_change"),
        "la modification du fuseau doit rendre un avertissement que l'interface présente avant de \
         confirmer : une clôture déjà produite ne couvre plus la même période"
    );

    let types = types_evenements(&pool_owner, jeu.tenant_id, id).await;
    for attendu in [
        "etablissement.cree",
        "etablissement.modifie",
        "etablissement.classement_change",
        "etablissement.fuseau_change",
    ] {
        assert!(
            types.iter().any(|t| t == attendu),
            "l'événement « {attendu} » n'a pas été émis. Types trouvés : {types:?}"
        );
    }

    // **Une modification qui ne change rien n'émet aucun événement.** Même principe que le rejeu :
    // le grand livre enregistre les transitions, pas les requêtes reçues.
    let avant = types.len();
    service
        .modifier(
            jeu.tenant_id,
            id,
            ModifierEtablissement {
                nom: Some("Après".to_owned()),
                ..Default::default()
            },
        )
        .await
        .expect("modification sans changement");
    let apres = types_evenements(&pool_owner, jeu.tenant_id, id).await;
    assert_eq!(
        apres.len(),
        avant,
        "une modification qui ne change rien a émis {} événement(s) supplémentaire(s)",
        apres.len() - avant
    );
}

/// **`etablissement_module.active` / `.desactive` et `module_capacite.declaree`.**
///
/// Une bascule vers l'état courant n'émet rien : réactiver un service déjà actif n'est pas une
/// transition.
#[tokio::test]
async fn p05_activation_desactivation_et_declaration_de_capacite() {
    use kaya_etablissements::modules::{BasculerService, DeclarerCapacite, ServiceModules};

    let pool_owner = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool_owner, "P-05 services").await;
    let service = ServiceModules::nouveau(commun::pool_app().await, PgOutboxWriter::nouveau());

    let activation = service
        .basculer(
            jeu.tenant_id,
            jeu.etablissement_id,
            "RESTAURATION",
            BasculerService {
                id: Uuid::now_v7(),
                actif: true,
            },
        )
        .await
        .expect("activation");
    let service_id = activation.service_id;

    // Rejeu de l'activation : aucune transition, donc aucun événement.
    service
        .basculer(
            jeu.tenant_id,
            jeu.etablissement_id,
            "RESTAURATION",
            BasculerService {
                id: Uuid::now_v7(),
                actif: true,
            },
        )
        .await
        .expect("rejeu de l'activation");

    let apres_activation = types_evenements(&pool_owner, jeu.tenant_id, service_id).await;
    assert_eq!(
        apres_activation,
        vec!["etablissement_module.active"],
        "l'activation puis son rejeu ont produit {apres_activation:?} : une bascule vers l'état \
         courant n'est pas une transition"
    );

    let capacite_id = Uuid::now_v7();
    service
        .declarer_capacite(
            jeu.tenant_id,
            jeu.etablissement_id,
            "RESTAURATION",
            DeclarerCapacite {
                id: capacite_id,
                capacite_code: "STOCK".to_owned(),
                profil_code: "SIMPLE".to_owned(),
            },
        )
        .await
        .expect("déclaration de capacité");

    let capacite = types_evenements(&pool_owner, jeu.tenant_id, capacite_id).await;
    assert_eq!(
        capacite,
        vec!["module_capacite.declaree"],
        "la déclaration de capacité doit émettre exactement un événement"
    );

    service
        .basculer(
            jeu.tenant_id,
            jeu.etablissement_id,
            "RESTAURATION",
            BasculerService {
                id: Uuid::now_v7(),
                actif: false,
            },
        )
        .await
        .expect("désactivation");

    let final_ = types_evenements(&pool_owner, jeu.tenant_id, service_id).await;
    assert_eq!(
        final_,
        vec![
            "etablissement_module.active",
            "etablissement_module.desactive"
        ],
        "la désactivation doit émettre son propre événement, à la suite de l'activation"
    );
}

/// **Un refus n'écrit ni ligne ni événement** — la garantie transactionnelle vue de l'échec.
///
/// Le test de rollback provoqué plus haut vérifie qu'une transaction annulée ne laisse rien. Ce
/// test-ci vérifie le cas réel qui l'exerce : une capacité refusée. Sans lui, on saurait que le
/// rollback fonctionne, sans savoir qu'il est bien déclenché par le refus.
#[tokio::test]
async fn p05_un_refus_de_capacite_ne_laisse_ni_ligne_ni_evenement() {
    use kaya_etablissements::modules::{BasculerService, DeclarerCapacite, ServiceModules};

    let pool_owner = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool_owner, "P-05 refus").await;
    let service = ServiceModules::nouveau(commun::pool_app().await, PgOutboxWriter::nouveau());

    service
        .basculer(
            jeu.tenant_id,
            jeu.etablissement_id,
            "BAR",
            BasculerService {
                id: Uuid::now_v7(),
                actif: true,
            },
        )
        .await
        .expect("activation");

    let capacite_id = Uuid::now_v7();
    let refus = service
        .declarer_capacite(
            jeu.tenant_id,
            jeu.etablissement_id,
            "BAR",
            DeclarerCapacite {
                id: capacite_id,
                capacite_code: "LIVRAISON".to_owned(),
                profil_code: "SIMPLE".to_owned(),
            },
        )
        .await;

    assert!(refus.is_err(), "LIVRAISON doit être refusée");

    let evenements = types_evenements(&pool_owner, jeu.tenant_id, capacite_id).await;
    assert!(
        evenements.is_empty(),
        "un refus a laissé {evenements:?} au grand livre : le journal enregistrerait des \
         transitions qui n'ont pas eu lieu"
    );
}

/// **`point_de_vente.cree` / `.modifie` et `table_pdv.creee` / `.desactivee`.**
///
/// Le point délicat est le **remplacement d'ensemble** : enregistrer un plan de salle inchangé ne
/// doit produire aucun événement. Sans filtrage, chaque enregistrement produirait douze
/// désactivations et douze créations, et le grand livre deviendrait illisible sur la seule table
/// qu'un exploitant touche souvent.
#[tokio::test]
async fn p05_points_de_vente_et_tables() {
    use kaya_etablissements::modules::{BasculerService, ServiceModules};
    use kaya_etablissements::points_de_vente::{
        CreerPointDeVente, ModifierPointDeVente, ServicePointsDeVente, TableDemandee,
    };

    let pool_owner = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool_owner, "P-05 points de vente").await;
    let pool_app = commun::pool_app().await;

    ServiceModules::nouveau(pool_app.clone(), PgOutboxWriter::nouveau())
        .basculer(
            jeu.tenant_id,
            jeu.etablissement_id,
            "RESTAURATION",
            BasculerService {
                id: Uuid::now_v7(),
                actif: true,
            },
        )
        .await
        .expect("activation");

    let service = ServicePointsDeVente::nouveau(pool_app.clone(), PgOutboxWriter::nouveau());
    let pdv_id = Uuid::now_v7();

    // Création — et rejeu, qui ne doit rien émettre de plus.
    for _ in 0..2 {
        service
            .creer(
                jeu.tenant_id,
                jeu.etablissement_id,
                CreerPointDeVente {
                    id: pdv_id,
                    module_code: "RESTAURATION".to_owned(),
                    nom: "Salle".to_owned(),
                    caisse_id: None,
                },
            )
            .await
            .expect("création du point de vente");
    }

    let apres_creation = types_evenements(&pool_owner, jeu.tenant_id, pdv_id).await;
    assert_eq!(
        apres_creation,
        vec!["point_de_vente.cree"],
        "création + rejeu ont produit {apres_creation:?}"
    );

    service
        .modifier(
            jeu.tenant_id,
            pdv_id,
            ModifierPointDeVente {
                nom: Some("Grande salle".to_owned()),
                ..Default::default()
            },
        )
        .await
        .expect("modification");

    let apres_modification = types_evenements(&pool_owner, jeu.tenant_id, pdv_id).await;
    assert_eq!(
        apres_modification,
        vec!["point_de_vente.cree", "point_de_vente.modifie"]
    );

    // Deux tables posées.
    let table_1 = Uuid::now_v7();
    let table_2 = Uuid::now_v7();
    let tables = || {
        vec![
            TableDemandee {
                id: table_1,
                libelle: "12".to_owned(),
            },
            TableDemandee {
                id: table_2,
                libelle: "Terrasse 3".to_owned(),
            },
        ]
    };
    service
        .remplacer_tables(jeu.tenant_id, pdv_id, tables())
        .await
        .expect("pose des tables");

    for id in [table_1, table_2] {
        let types = types_evenements(&pool_owner, jeu.tenant_id, id).await;
        assert_eq!(types, vec!["table_pdv.creee"], "table {id}");
    }

    // **Le même plan de salle, réenregistré : AUCUN événement supplémentaire.**
    service
        .remplacer_tables(jeu.tenant_id, pdv_id, tables())
        .await
        .expect("réenregistrement du même plan de salle");

    for id in [table_1, table_2] {
        let types = types_evenements(&pool_owner, jeu.tenant_id, id).await;
        assert_eq!(
            types,
            vec!["table_pdv.creee"],
            "réenregistrer un plan de salle inchangé a produit {types:?} pour la table {id} : le \
             grand livre deviendrait illisible sur la table qu'un exploitant touche le plus souvent"
        );
    }

    // **Liste vide ⇒ comptoir.** Transition légitime, et les deux tables sont désactivées.
    let comptoir = service
        .remplacer_tables(jeu.tenant_id, pdv_id, Vec::new())
        .await
        .expect("passage en comptoir");
    assert!(
        comptoir.tables.is_empty(),
        "le point de vente doit être devenu un comptoir"
    );

    for id in [table_1, table_2] {
        let types = types_evenements(&pool_owner, jeu.tenant_id, id).await;
        assert_eq!(
            types,
            vec!["table_pdv.creee", "table_pdv.desactivee"],
            "table {id} : le retrait doit émettre sa désactivation"
        );
    }
}

/// **`parametre_configuration.ecrit` — et l'ANCIENNE valeur dans la charge utile.**
///
/// Sans elle, le grand livre dirait qu'une valeur a changé sans dire depuis quoi : une
/// reconstitution ne pourrait pas remonter le fil d'un barème modifié trois fois, et c'est
/// exactement ce qu'on demandera au journal le jour d'un contrôle fiscal.
#[tokio::test]
async fn p05_ecriture_de_parametre_porte_l_ancienne_valeur() {
    use kaya_etablissements::Portee;
    use kaya_etablissements::configuration::{EcrireParametre, ServiceConfiguration};

    let pool_owner = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool_owner, "P-05 configuration").await;
    let service = ServiceConfiguration::nouveau(commun::pool_app().await, PgOutboxWriter::nouveau());

    let premier = Uuid::now_v7();
    service
        .ecrire(
            jeu.tenant_id,
            EcrireParametre {
                id: premier,
                cle: "politique_impression".to_owned(),
                valeur: serde_json::json!("aucune"),
                portee: Portee::Tenant,
                portee_id: None,
            },
        )
        .await
        .expect("première écriture");

    let types = types_evenements(&pool_owner, jeu.tenant_id, premier).await;
    assert_eq!(types, vec!["parametre_configuration.ecrit"]);

    // Seconde écriture, valeur différente : l'ancienne doit figurer à la charge utile.
    let second = Uuid::now_v7();
    service
        .ecrire(
            jeu.tenant_id,
            EcrireParametre {
                id: second,
                cle: "politique_impression".to_owned(),
                valeur: serde_json::json!("ticket_cuisine"),
                portee: Portee::Tenant,
                portee_id: None,
            },
        )
        .await
        .expect("seconde écriture");

    let mut tx = pool_owner.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, jeu.tenant_id)
        .await
        .expect("pose du tenant");
    let payload: serde_json::Value = sqlx::query_scalar!(
        r#"
        SELECT payload AS "payload!"
        FROM synchronisation.evenement_outbox
        WHERE agregat_id = $1
        "#,
        second
    )
    .fetch_one(&mut *tx)
    .await
    .expect("lecture de l'événement");
    tx.rollback().await.expect("rollback");

    assert_eq!(
        payload["ancienne_valeur"],
        serde_json::json!("aucune"),
        "l'événement de surcharge doit porter l'ANCIENNE valeur. Charge utile : {payload}"
    );
    assert_eq!(payload["valeur"], serde_json::json!("ticket_cuisine"));
    assert_eq!(payload["niveau"], serde_json::json!("TENANT"));

    // Réécrire la même valeur n'émet rien — même principe que le rejeu.
    let troisieme = Uuid::now_v7();
    service
        .ecrire(
            jeu.tenant_id,
            EcrireParametre {
                id: troisieme,
                cle: "politique_impression".to_owned(),
                valeur: serde_json::json!("ticket_cuisine"),
                portee: Portee::Tenant,
                portee_id: None,
            },
        )
        .await
        .expect("réécriture à l'identique");

    let types = types_evenements(&pool_owner, jeu.tenant_id, troisieme).await;
    assert!(
        types.is_empty(),
        "réécrire une valeur identique a émis {types:?} : le grand livre enregistrerait une \
         transition qui n'a pas eu lieu"
    );
}

/// **`branding.modifie` — et la clé d'objet, jamais le binaire.**
///
/// Le grand livre est à rétention illimitée : y écrire des logos le ferait grossir sans fin pour
/// une information que le stockage objet porte déjà.
#[tokio::test]
async fn p05_modification_d_identite_visuelle() {
    use kaya_etablissements::branding::{BrandingNiveau, EcrireBranding, ServiceBranding};

    let pool_owner = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool_owner, "P-05 branding").await;
    let service = ServiceBranding::nouveau(commun::pool_app().await, PgOutboxWriter::nouveau());

    let id = Uuid::now_v7();
    service
        .ecrire(
            jeu.tenant_id,
            EcrireBranding {
                id,
                etablissement_id: None,
                contenu: BrandingNiveau {
                    logo_objet_cle: Some("branding/x/tenant/logo".to_owned()),
                    couleur_primaire: Some("#0A7B5F".to_owned()),
                    ..Default::default()
                },
            },
        )
        .await
        .expect("écriture de l'identité visuelle");

    let types = types_evenements(&pool_owner, jeu.tenant_id, id).await;
    assert_eq!(types, vec!["branding.modifie"]);

    let mut tx = pool_owner.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, jeu.tenant_id)
        .await
        .expect("pose du tenant");
    let payload: serde_json::Value = sqlx::query_scalar!(
        r#"SELECT payload AS "payload!" FROM synchronisation.evenement_outbox WHERE agregat_id = $1"#,
        id
    )
    .fetch_one(&mut *tx)
    .await
    .expect("lecture de l'événement");
    tx.rollback().await.expect("rollback");

    assert_eq!(payload["niveau"], serde_json::json!("TENANT"));
    assert_eq!(
        payload["logo_objet_cle"],
        serde_json::json!("branding/x/tenant/logo"),
        "l'événement doit porter la CLÉ d'objet, jamais le binaire — le grand livre est à rétention \
         illimitée. Charge utile : {payload}"
    );
    let champs = payload["champs_touches"]
        .as_array()
        .expect("champs_touches doit être un tableau");
    assert!(
        champs.iter().any(|c| c == "logo_objet_cle") && champs.iter().any(|c| c == "couleur_primaire"),
        "les champs réellement touchés doivent être nommés : {champs:?}"
    );

    // Réécrire à l'identique n'émet rien.
    let second = Uuid::now_v7();
    service
        .ecrire(
            jeu.tenant_id,
            EcrireBranding {
                id: second,
                etablissement_id: None,
                contenu: BrandingNiveau {
                    logo_objet_cle: Some("branding/x/tenant/logo".to_owned()),
                    couleur_primaire: Some("#0A7B5F".to_owned()),
                    ..Default::default()
                },
            },
        )
        .await
        .expect("réécriture identique");
    let types = types_evenements(&pool_owner, jeu.tenant_id, second).await;
    assert!(
        types.is_empty(),
        "réécrire une identité identique a émis {types:?}"
    );
}

// =================================================================================================
//  Cycle 003 — les neuf types du module des comptes, et les deux tenants
// =================================================================================================
//
// **Exigence 5 du § « Couverture des portes » : chaque type est exercé sur les DEUX tenants.**
// C'est né du défaut de séquence que la migration `0012` a corrigé et qu'aucune relecture n'avait
// vu — une numérotation d'événements correcte pour un tenant et fausse pour le second. Un test à
// un seul tenant serait vert sur ce défaut-là.

/// Le service des comptes, assemblé comme le fait l'application réelle.
fn service_comptes(
    pool: sqlx::PgPool,
) -> kaya_comptes::compte::ServiceComptes<PgOutboxWriter, kaya_comptes::audit::JournalAuditPostgres>
{
    kaya_comptes::compte::ServiceComptes::nouveau(
        pool,
        PgOutboxWriter::nouveau(),
        kaya_comptes::audit::JournalAuditPostgres,
        kaya_comptes::session::Entrepot::nouveau(&commun::url_redis()).expect("entrepôt Redis"),
    )
}

/// Le service des rôles, assemblé comme le fait l'application réelle.
fn service_roles(
    pool: sqlx::PgPool,
    tenant_id: Uuid,
) -> kaya_comptes::roles::ServiceRoles<
    PgOutboxWriter,
    kaya_comptes::audit::JournalAuditPostgres,
    kaya_etablissements::etablissement::PgEstablishmentDirectory,
> {
    kaya_comptes::roles::ServiceRoles::nouveau(
        pool.clone(),
        PgOutboxWriter::nouveau(),
        kaya_comptes::audit::JournalAuditPostgres,
        kaya_etablissements::etablissement::PgEstablishmentDirectory::nouveau(pool, tenant_id),
    )
}

/// **`personne.creee` et `personne.modifiee`, sur les DEUX tenants.**
///
/// Le rejeu de la création n'émet rien de plus ; la modification, elle, émet **à chaque fois** —
/// un second `PUT` identique est une seconde transition d'état, même si l'état final est le même,
/// et le registre doit pouvoir dire qui a touché la fiche et quand.
#[tokio::test]
async fn p05_personne_creee_et_modifiee_sur_les_deux_tenants() {
    use kaya_comptes::personne::{CreerPersonne, ModifierPersonne, ServicePersonne};

    let pool_owner = commun::pool_owner().await;
    let service = ServicePersonne::nouveau(commun::pool_app().await, PgOutboxWriter::nouveau());

    for rang in ['A', 'B'] {
        let jeu = commun::creer_tenant(&pool_owner, &format!("P-05 personne {rang}")).await;
        let id = Uuid::now_v7();

        // Deux envois du même identifiant : un seul événement.
        for _ in 0..2 {
            service
                .creer(
                    jeu.tenant_id,
                    CreerPersonne {
                        id,
                        nom: format!("Personne {rang}"),
                        prenoms: None,
                        telephone: None,
                        email: None,
                        horodatage_client: None,
                    },
                )
                .await
                .expect("création");
        }

        service
            .modifier(
                jeu.tenant_id,
                id,
                ModifierPersonne {
                    nom: format!("Personne {rang} modifiée"),
                    prenoms: None,
                    telephone: None,
                    email: None,
                    horodatage_client: None,
                },
            )
            .await
            .expect("modification");

        let types = types_evenements(&pool_owner, jeu.tenant_id, id).await;
        assert_eq!(
            types,
            vec!["personne.creee", "personne.modifiee"],
            "tenant {rang} : {types:?}"
        );
    }
}

/// **`compte.cree`, `compte.desactive`, `compte.reactive` et `compte.mot_de_passe_change`.**
///
/// Les quatre sur le même agrégat, sur les deux tenants. Deux propriétés s'y vérifient au passage :
///
///  * **une désactivation sans transition n'émet rien** — désactiver un compte déjà inactif est un
///    rejeu, et le traiter comme un acte ferait du grand livre le journal des reprises réseau ;
///  * **aucune charge utile ne porte le secret ni son condensat**, ni même sa longueur — la
///    longueur seule réduirait déjà l'espace de recherche d'une attaque hors ligne.
#[tokio::test]
async fn p05_les_quatre_types_de_compte_sur_les_deux_tenants() {
    use kaya_comptes::compte::CreerCompte;
    use kaya_comptes::personne::{CreerPersonne, ServicePersonne};

    let pool_owner = commun::pool_owner().await;
    let pool = commun::pool_app().await;
    let personnes = ServicePersonne::nouveau(pool.clone(), PgOutboxWriter::nouveau());
    let comptes = service_comptes(pool.clone());

    for rang in ['A', 'B'] {
        let jeu = commun::creer_tenant(&pool_owner, &format!("P-05 compte {rang}")).await;

        let personne_id = Uuid::now_v7();
        personnes
            .creer(
                jeu.tenant_id,
                CreerPersonne {
                    id: personne_id,
                    nom: format!("Titulaire {rang}"),
                    prenoms: None,
                    telephone: None,
                    email: None,
                    horodatage_client: None,
                },
            )
            .await
            .expect("personne");

        let compte_id = Uuid::now_v7();
        let hexa = Uuid::now_v7().simple().to_string();
        comptes
            .creer(
                jeu.tenant_id,
                CreerCompte {
                    id: compte_id,
                    personne_id,
                    identifiant_telephone: Some(format!("+225{}", &hexa[hexa.len() - 10..])),
                    identifiant_email: None,
                    mot_de_passe: commun::MOT_DE_PASSE_TEST.to_owned(),
                    horodatage_client: None,
                },
            )
            .await
            .expect("compte");

        comptes
            .changer_etat(jeu.tenant_id, compte_id, compte_id, false)
            .await
            .expect("désactivation");
        // **Sans transition, rien n'est émis** — la seconde désactivation est un rejeu.
        comptes
            .changer_etat(jeu.tenant_id, compte_id, compte_id, false)
            .await
            .expect("désactivation rejouée");
        comptes
            .changer_etat(jeu.tenant_id, compte_id, compte_id, true)
            .await
            .expect("réactivation");

        comptes
            .changer_mot_de_passe(
                jeu.tenant_id,
                compte_id,
                None,
                "abidjan-tomate-chaise",
                None,
                3600,
            )
            .await
            .expect("changement de mot de passe");

        let types = types_evenements(&pool_owner, jeu.tenant_id, compte_id).await;
        assert_eq!(
            types,
            vec![
                "compte.cree",
                "compte.desactive",
                "compte.reactive",
                "compte.mot_de_passe_change",
            ],
            "tenant {rang} : {types:?}"
        );

        // **Aucun secret dans le grand livre**, qui est permanent.
        let charges = charges_utiles(&pool_owner, jeu.tenant_id, compte_id).await;
        let brut = serde_json::to_string(&charges).expect("sérialisation");
        assert!(!brut.contains(commun::MOT_DE_PASSE_TEST), "le mot de passe est au grand livre");
        assert!(!brut.contains("abidjan-tomate-chaise"), "le nouveau mot de passe est au grand livre");
        assert!(!brut.contains("$argon2"), "un condensat est au grand livre");
    }
}

/// **`role.attribue` et `role.retire`, sur les deux tenants.**
///
/// Deux actes, deux événements. Il n'existe **aucun** type « rôle modifié » : `compte_role` n'a pas
/// de privilège `UPDATE`, et une opération unique cacherait l'un des deux au grand livre.
#[tokio::test]
async fn p05_role_attribue_et_retire_sur_les_deux_tenants() {
    let pool_owner = commun::pool_owner().await;
    let pool = commun::pool_app().await;

    for rang in ['A', 'B'] {
        let jeu = commun::creer_tenant(&pool_owner, &format!("P-05 rôle {rang}")).await;
        let etb = jeu.etablissement_id;

        let auteur = commun::compte_connecte(
            &pool_owner,
            jeu,
            &format!("Auteur rôle {rang}"),
            &[("proprietaire", Some(etb))],
        )
        .await;
        let cible = commun::compte_connecte(&pool_owner, jeu, &format!("Cible rôle {rang}"), &[]).await;

        let service = service_roles(pool.clone(), jeu.tenant_id);

        service
            .attribuer(
                jeu.tenant_id,
                auteur.compte_id,
                kaya_comptes::roles::AttribuerRole {
                    id: Uuid::now_v7(),
                    compte_id: cible.compte_id,
                    role_code: "caissier".to_owned(),
                    etablissement_id: Some(etb),
                    horodatage_client: None,
                },
            )
            .await
            .expect("attribution");

        service
            .retirer(jeu.tenant_id, auteur.compte_id, cible.compte_id, "caissier", Some(etb))
            .await
            .expect("retrait");

        let types = types_evenements(&pool_owner, jeu.tenant_id, cible.compte_id).await;
        assert_eq!(types, vec!["role.attribue", "role.retire"], "tenant {rang} : {types:?}");
    }
}

/// **`session.revoquee`, sur les deux tenants.**
#[tokio::test]
async fn p05_session_revoquee_sur_les_deux_tenants() {
    let pool_owner = commun::pool_owner().await;

    for rang in ['A', 'B'] {
        let jeu = commun::creer_tenant(&pool_owner, &format!("P-05 session {rang}")).await;
        let compte = commun::compte_connecte(
            &pool_owner,
            jeu,
            &format!("Session {rang}"),
            &[("proprietaire", Some(jeu.etablissement_id))],
        )
        .await;

        let service = commun::service_authentification(commun::pool_app().await);
        // `session_courante` ne sert qu'à marquer laquelle est « cet appareil-ci » dans la vue ;
        // un UUID neuf suffit ici, aucune des deux n'étant la nôtre.
        let sessions = service
            .lister_actives(compte.compte_id, Uuid::now_v7())
            .await
            .expect("liste des sessions");
        let session_id = sessions.first().expect("une session ouverte").id;

        service
            .revoquer(compte.compte_id, jeu.tenant_id, compte.compte_id, session_id, 3600)
            .await
            .expect("révocation");

        let types = types_evenements(&pool_owner, jeu.tenant_id, compte.compte_id).await;
        assert_eq!(types, vec!["session.revoquee"], "tenant {rang} : {types:?}");
    }
}

/// **La connexion, le rafraîchissement et l'échec d'authentification N'ÉMETTENT RIEN.**
///
/// Research R-15, et c'est une décision, pas une omission : ce ne sont **pas des transitions
/// d'état métier**. Le grand livre est permanent et à rétention illimitée — y inscrire les
/// connexions y écrirait **la liste horodatée des présences du personnel**, pour toujours, sans
/// que personne l'ait décidé.
///
/// Les échecs d'authentification vont aux journaux applicatifs, où ils ont une rétention bornée et
/// un public d'exploitation.
#[tokio::test]
async fn p05_la_connexion_et_ses_echecs_n_emettent_aucun_evenement() {
    let pool_owner = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool_owner, "P-05 connexion muette").await;

    // `compte_connecte` passe par le VRAI chemin de connexion — c'est là tout l'intérêt : le test
    // observe ce que fait `ServiceAuthentification::ouvrir`, pas ce qu'on croit qu'il fait.
    let compte = commun::compte_connecte(
        &pool_owner,
        jeu,
        "Connexion muette",
        &[("proprietaire", Some(jeu.etablissement_id))],
    )
    .await;

    let service = commun::service_authentification(commun::pool_app().await);

    // Un échec d'authentification, sur un identifiant qui n'existe pas.
    let echec = service
        .ouvrir("+22500000000000", "peu-importe-le-mot-de-passe", None, None, "203.0.113.250")
        .await;
    assert!(echec.is_err(), "l'échec doit rester un échec");

    let types = types_evenements(&pool_owner, jeu.tenant_id, compte.compte_id).await;
    assert!(
        types.is_empty(),
        "la connexion a émis {types:?} au grand livre.\n\
         Le grand livre est permanent : y inscrire les connexions y écrirait la liste horodatée \
         des présences du personnel, pour toujours (research R-15)."
    );
}

/// Les charges utiles des événements d'un agrégat — pour vérifier ce qu'elles ne portent PAS.
async fn charges_utiles(
    pool_owner: &sqlx::PgPool,
    tenant_id: Uuid,
    agregat_id: Uuid,
) -> Vec<serde_json::Value> {
    let mut tx = pool_owner.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, tenant_id)
        .await
        .expect("pose du tenant");

    let charges = sqlx::query_scalar!(
        r#"
        SELECT payload AS "payload!"
        FROM synchronisation.evenement_outbox
        WHERE agregat_id = $1
        ORDER BY sequence_etablissement
        "#,
        agregat_id
    )
    .fetch_all(&mut *tx)
    .await
    .expect("lecture des charges utiles");

    tx.rollback().await.expect("rollback");
    charges
}

// =================================================================================================
//  Cycle 004 (HEB) — les cinq types d'événements de la première verticale
// =================================================================================================
//
// | Type | Émis par | Ce que la charge utile porte |
// |---|---|---|
// | `heb.formule.creee` | `ServiceReferentiel::creer_formule` | famille, `prix_mineur` + `devise` |
// | `heb.formule.modifiee` | `ServiceReferentiel::modifier_formule` | l'état après |
// | `heb.categorie.tarif_modifie` | idem, **si le prix a bougé** | l'avant ET l'après |
// | `heb.occupation.attribuee` | `ServiceOccupation::attribuer` | bornes client et borne d'indisponibilité |
// | `heb.occupation.liberee` | `ServiceOccupation::liberer` | horodatage et nouvelle borne |
//
// **Nommage monétaire réservé (P-10)** : `prix_mineur` entier et `devise` au même niveau. Jamais
// `prix`, `montant` ni `total` nus — le contrôle statique les refuse.

/// Assemble les deux services du cycle 004 **comme l'application réelle le fait**.
fn services_hebergement(
    pool: sqlx::PgPool,
    tenant_id: Uuid,
) -> (
    kaya_hebergement::referentiel::ServiceReferentiel<
        kaya_synchronisation::outbox::PgOutboxWriter,
        kaya_etablissements::etablissement::PgEstablishmentDirectory,
        kaya_etablissements::modules::PgRegistreModules,
    >,
    kaya_hebergement::occupation::ServiceOccupation<
        kaya_synchronisation::outbox::PgOutboxWriter,
        kaya_etablissements::etablissement::PgEstablishmentDirectory,
        kaya_etablissements::modules::PgRegistreModules,
    >,
) {
    let annuaire = || {
        kaya_etablissements::etablissement::PgEstablishmentDirectory::nouveau(
            pool.clone(),
            tenant_id,
        )
    };
    let modules =
        || kaya_etablissements::modules::PgRegistreModules::nouveau(pool.clone(), tenant_id);

    (
        kaya_hebergement::referentiel::ServiceReferentiel::nouveau(
            pool.clone(),
            kaya_synchronisation::outbox::PgOutboxWriter::nouveau(),
            annuaire(),
            modules(),
        ),
        kaya_hebergement::occupation::ServiceOccupation::nouveau(
            pool.clone(),
            tenant_id,
            kaya_synchronisation::outbox::PgOutboxWriter::nouveau(),
            annuaire(),
            modules(),
        ),
    )
}

async fn activer_hebergement_pour(pool: &sqlx::PgPool, jeu: commun::JeuTenant) {
    kaya_etablissements::modules::ServiceModules::nouveau(
        pool.clone(),
        kaya_synchronisation::outbox::PgOutboxWriter::nouveau(),
    )
    .basculer(
        jeu.tenant_id,
        jeu.etablissement_id,
        "HEBERGEMENT",
        kaya_etablissements::modules::BasculerService {
            id: Uuid::now_v7(),
            actif: true,
        },
    )
    .await
    .expect("activation de l'hébergement");
}

/// La charge utile d'un événement, pour vérifier son **nommage monétaire**.
async fn charge_utile(
    pool_owner: &sqlx::PgPool,
    tenant_id: Uuid,
    agregat_id: Uuid,
    type_evenement: &str,
) -> serde_json::Value {
    let mut tx = pool_owner.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, tenant_id)
        .await
        .expect("pose du tenant");
    let payload: serde_json::Value = sqlx::query_scalar(
        r#"
        SELECT payload
        FROM synchronisation.evenement_outbox
        WHERE agregat_id = $1 AND type_evenement = $2
        ORDER BY sequence_etablissement DESC
        LIMIT 1
        "#,
    )
    .bind(agregat_id)
    .bind(type_evenement)
    .fetch_one(&mut *tx)
    .await
    .unwrap_or_else(|e| panic!("aucun événement « {type_evenement} » pour {agregat_id} : {e}"));
    tx.rollback().await.expect("rollback");
    payload
}

/// **`heb.formule.creee` — un événement, et un seul malgré trois envois.**
#[tokio::test]
async fn p05_heb_creation_de_formule_un_seul_evenement_malgre_le_rejeu() {
    let pool_owner = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool_owner, "P-05 HEB formule").await;
    let pool = commun::pool_app().await;
    activer_hebergement_pour(&pool, jeu).await;

    let (referentiel, _) = services_hebergement(pool.clone(), jeu.tenant_id);

    let categorie_id = Uuid::now_v7();
    referentiel
        .creer_categorie(
            jeu.tenant_id,
            kaya_hebergement::referentiel::CreerCategorie {
                id: categorie_id,
                etablissement_id: jeu.etablissement_id,
                nom: "Standard".to_owned(),
                capacite_accueil: 2,
                temps_remise_en_etat: Vec::new(),
            },
        )
        .await
        .expect("catégorie");

    // **Une catégorie n'émet AUCUN événement** — elle n'a d'effet ni monétaire, ni fiscal, ni sur
    // la disponibilité. Le vérifier ici évite qu'on en ajoute un « par symétrie ».
    assert!(
        types_evenements(&pool_owner, jeu.tenant_id, categorie_id)
            .await
            .is_empty(),
        "la création d'un type de chambre ne doit émettre aucun événement"
    );

    let formule_id = Uuid::now_v7();
    let demande = || kaya_hebergement::referentiel::CreerFormule {
        id: formule_id,
        etablissement_id: jeu.etablissement_id,
        categorie_id,
        famille: kaya_hebergement::referentiel::FamilleFormule::Nuitee,
        prix_mineur: 12_500,
        duree_min_minutes: None,
        duree_max_minutes: None,
        heure_arrivee_standard: Some("14:00".to_owned()),
        heure_depart_standard: Some("12:00".to_owned()),
        jours_autorises: None,
        assujettie_taxe_nuitee: true,
        regle_conversion_taxe: Some(
            kaya_hebergement::referentiel::RegleConversionTaxe::UneNuiteeParOccupation,
        ),
        prix_heure_supplementaire_mineur: None,
        paliers: Vec::new(),
        plages: Vec::new(),
    };

    for _ in 0..3 {
        referentiel
            .creer_formule(jeu.tenant_id, demande())
            .await
            .expect("création ou rejeu");
    }

    let types = types_evenements(&pool_owner, jeu.tenant_id, formule_id).await;
    assert_eq!(
        types,
        vec!["heb.formule.creee"],
        "trois envois du même identifiant doivent laisser UN événement : le grand livre porte les \
         transitions d'état, pas les tentatives réseau du terminal"
    );

    // **P-10 — nommage monétaire réservé** : `prix_mineur` entier, `devise` au même niveau.
    let payload = charge_utile(&pool_owner, jeu.tenant_id, formule_id, "heb.formule.creee").await;
    assert!(
        payload.get("prix_mineur").and_then(|v| v.as_i64()).is_some(),
        "la charge utile doit porter `prix_mineur` ENTIER : {payload}"
    );
    assert!(
        payload.get("devise").and_then(|v| v.as_str()).is_some(),
        "la charge utile doit porter `devise` au MÊME NIVEAU que le montant : {payload}"
    );
    assert!(
        payload.get("prix").is_none()
            && payload.get("montant").is_none()
            && payload.get("total").is_none(),
        "aucune clé monétaire nue : {payload}"
    );
}

/// **`heb.formule.modifiee` toujours, `heb.categorie.tarif_modifie` SEULEMENT si le prix bouge.**
///
/// Les deux disent deux choses différentes. Noyer le second dans le premier obligerait un lecteur
/// du grand livre à comparer deux charges utiles pour savoir si un tarif a changé — et c'est
/// exactement la question que la reconstitution financière pose.
#[tokio::test]
async fn p05_heb_le_tarif_modifie_n_est_emis_que_si_le_prix_bouge() {
    let pool_owner = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool_owner, "P-05 HEB tarif").await;
    let pool = commun::pool_app().await;
    activer_hebergement_pour(&pool, jeu).await;

    let (referentiel, _) = services_hebergement(pool.clone(), jeu.tenant_id);

    let categorie_id = Uuid::now_v7();
    referentiel
        .creer_categorie(
            jeu.tenant_id,
            kaya_hebergement::referentiel::CreerCategorie {
                id: categorie_id,
                etablissement_id: jeu.etablissement_id,
                nom: "Classique".to_owned(),
                capacite_accueil: 2,
                temps_remise_en_etat: Vec::new(),
            },
        )
        .await
        .expect("catégorie");

    let formule_id = Uuid::now_v7();
    referentiel
        .creer_formule(
            jeu.tenant_id,
            kaya_hebergement::referentiel::CreerFormule {
                id: formule_id,
                etablissement_id: jeu.etablissement_id,
                categorie_id,
                famille: kaya_hebergement::referentiel::FamilleFormule::Nuitee,
                prix_mineur: 15_500,
                duree_min_minutes: None,
                duree_max_minutes: None,
                heure_arrivee_standard: None,
                heure_depart_standard: None,
                jours_autorises: None,
                assujettie_taxe_nuitee: false,
                regle_conversion_taxe: None,
                prix_heure_supplementaire_mineur: None,
                paliers: Vec::new(),
                plages: Vec::new(),
            },
        )
        .await
        .expect("formule");

    let changements = |prix: i64, assujettie: bool| kaya_hebergement::referentiel::ModifierFormule {
        prix_mineur: prix,
        duree_min_minutes: None,
        duree_max_minutes: None,
        heure_arrivee_standard: None,
        heure_depart_standard: None,
        jours_autorises: None,
        assujettie_taxe_nuitee: assujettie,
        regle_conversion_taxe: if assujettie {
            Some(kaya_hebergement::referentiel::RegleConversionTaxe::UneNuiteeParOccupation)
        } else {
            None
        },
        prix_heure_supplementaire_mineur: None,
        paliers: Vec::new(),
        plages: Vec::new(),
    };

    // 1 · le prix ne bouge pas — l'exploitant active seulement la taxe.
    referentiel
        .modifier_formule(
            jeu.tenant_id,
            jeu.etablissement_id,
            formule_id,
            changements(15_500, true),
        )
        .await
        .expect("activation de la taxe");

    let types = types_evenements(&pool_owner, jeu.tenant_id, formule_id).await;
    assert_eq!(
        types,
        vec!["heb.formule.creee", "heb.formule.modifiee"],
        "activer la taxe sans toucher au prix ne doit PAS émettre `heb.categorie.tarif_modifie`"
    );

    // 2 · le prix bouge.
    referentiel
        .modifier_formule(
            jeu.tenant_id,
            jeu.etablissement_id,
            formule_id,
            changements(17_500, true),
        )
        .await
        .expect("changement de prix");

    let sur_categorie = types_evenements(&pool_owner, jeu.tenant_id, categorie_id).await;
    assert_eq!(
        sur_categorie,
        vec!["heb.categorie.tarif_modifie"],
        "un changement de prix doit émettre `heb.categorie.tarif_modifie`, sur l'agrégat CATÉGORIE"
    );

    // La charge utile porte **l'avant ET l'après** — sans quoi la reconstitution financière
    // devrait rejouer tout l'historique pour connaître le prix précédent.
    let payload = charge_utile(
        &pool_owner,
        jeu.tenant_id,
        categorie_id,
        "heb.categorie.tarif_modifie",
    )
    .await;
    assert_eq!(payload["prix_avant_mineur"].as_i64(), Some(15_500));
    assert_eq!(payload["prix_apres_mineur"].as_i64(), Some(17_500));
    assert!(payload.get("devise").is_some(), "devise au même niveau : {payload}");
}

/// **`heb.occupation.attribuee` et `heb.occupation.liberee`** — une transition, un événement.
#[tokio::test]
async fn p05_heb_attribution_et_liberation_emettent_chacune_leur_evenement() {
    let pool_owner = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool_owner, "P-05 HEB occupation").await;
    let pool = commun::pool_app().await;
    activer_hebergement_pour(&pool, jeu).await;

    let (referentiel, occupation) = services_hebergement(pool.clone(), jeu.tenant_id);

    let categorie_id = Uuid::now_v7();
    referentiel
        .creer_categorie(
            jeu.tenant_id,
            kaya_hebergement::referentiel::CreerCategorie {
                id: categorie_id,
                etablissement_id: jeu.etablissement_id,
                nom: "Standard".to_owned(),
                capacite_accueil: 2,
                temps_remise_en_etat: vec![kaya_hebergement::referentiel::TempsRemiseEnEtat {
                    famille_formule: kaya_hebergement::referentiel::FamilleFormule::Nuitee,
                    duree_minutes: 120,
                }],
            },
        )
        .await
        .expect("catégorie");

    let unite_id = Uuid::now_v7();
    referentiel
        .creer_unite(
            jeu.tenant_id,
            kaya_hebergement::referentiel::CreerUnite {
                id: unite_id,
                etablissement_id: jeu.etablissement_id,
                categorie_id,
                code: "A1".to_owned(),
                etage: Some(1),
            },
        )
        .await
        .expect("unité");

    let formule_id = Uuid::now_v7();
    referentiel
        .creer_formule(
            jeu.tenant_id,
            kaya_hebergement::referentiel::CreerFormule {
                id: formule_id,
                etablissement_id: jeu.etablissement_id,
                categorie_id,
                famille: kaya_hebergement::referentiel::FamilleFormule::Nuitee,
                prix_mineur: 12_500,
                duree_min_minutes: None,
                duree_max_minutes: None,
                heure_arrivee_standard: None,
                heure_depart_standard: None,
                jours_autorises: None,
                assujettie_taxe_nuitee: false,
                regle_conversion_taxe: None,
                prix_heure_supplementaire_mineur: None,
                paliers: Vec::new(),
                plages: Vec::new(),
            },
        )
        .await
        .expect("formule");

    let occupation_id = Uuid::now_v7();
    let demande = || kaya_hebergement::occupation::DemandeAttribution {
        id: occupation_id,
        etablissement_id: jeu.etablissement_id,
        unite_id,
        formule_id,
        debut_client: time::macros::datetime!(2027-05-01 14:00 UTC),
        fin_client: time::macros::datetime!(2027-05-03 12:00 UTC),
    };

    // Trois envois — **un seul événement**.
    for _ in 0..3 {
        occupation
            .attribuer(demande())
            .await
            .expect("attribution ou rejeu");
    }

    assert_eq!(
        types_evenements(&pool_owner, jeu.tenant_id, occupation_id).await,
        vec!["heb.occupation.attribuee"],
        "trois attributions du même identifiant doivent laisser UN événement"
    );

    // La charge utile porte la borne d'indisponibilité — le ménage compris.
    let payload = charge_utile(
        &pool_owner,
        jeu.tenant_id,
        occupation_id,
        "heb.occupation.attribuee",
    )
    .await;
    assert_eq!(payload["battement_minutes"].as_i64(), Some(120));
    assert!(
        payload.get("indisponible_jusqu_a").is_some(),
        "un lecteur qui n'a que cette ligne doit savoir jusqu'à quand la chambre est prise : \
         {payload}"
    );

    // Libération, puis rejeu — **un seul second événement**.
    for _ in 0..2 {
        occupation
            .liberer(jeu.etablissement_id, occupation_id)
            .await
            .expect("libération ou rejeu");
    }

    assert_eq!(
        types_evenements(&pool_owner, jeu.tenant_id, occupation_id).await,
        vec!["heb.occupation.attribuee", "heb.occupation.liberee"],
        "une libération rejouée ne doit pas produire un second `heb.occupation.liberee`"
    );
}

/// **Un refus d'attribution ne laisse ni ligne ni événement.**
///
/// La violation d'exclusion empoisonne la transaction ; le service la rejette, et le grand livre
/// n'en garde aucune trace. Un événement écrit avant l'échec ferait compter au grand livre une
/// attribution qui n'a jamais eu lieu.
#[tokio::test]
async fn p05_heb_une_attribution_refusee_ne_laisse_ni_ligne_ni_evenement() {
    let pool_owner = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool_owner, "P-05 HEB refus").await;
    let pool = commun::pool_app().await;
    activer_hebergement_pour(&pool, jeu).await;

    let (referentiel, occupation) = services_hebergement(pool.clone(), jeu.tenant_id);

    let categorie_id = Uuid::now_v7();
    referentiel
        .creer_categorie(
            jeu.tenant_id,
            kaya_hebergement::referentiel::CreerCategorie {
                id: categorie_id,
                etablissement_id: jeu.etablissement_id,
                nom: "Standard".to_owned(),
                capacite_accueil: 2,
                temps_remise_en_etat: Vec::new(),
            },
        )
        .await
        .expect("catégorie");
    let unite_id = Uuid::now_v7();
    referentiel
        .creer_unite(
            jeu.tenant_id,
            kaya_hebergement::referentiel::CreerUnite {
                id: unite_id,
                etablissement_id: jeu.etablissement_id,
                categorie_id,
                code: "A1".to_owned(),
                etage: None,
            },
        )
        .await
        .expect("unité");
    let formule_id = Uuid::now_v7();
    referentiel
        .creer_formule(
            jeu.tenant_id,
            kaya_hebergement::referentiel::CreerFormule {
                id: formule_id,
                etablissement_id: jeu.etablissement_id,
                categorie_id,
                famille: kaya_hebergement::referentiel::FamilleFormule::Nuitee,
                prix_mineur: 12_500,
                duree_min_minutes: None,
                duree_max_minutes: None,
                heure_arrivee_standard: None,
                heure_depart_standard: None,
                jours_autorises: None,
                assujettie_taxe_nuitee: false,
                regle_conversion_taxe: None,
                prix_heure_supplementaire_mineur: None,
                paliers: Vec::new(),
                plages: Vec::new(),
            },
        )
        .await
        .expect("formule");

    occupation
        .attribuer(kaya_hebergement::occupation::DemandeAttribution {
            id: Uuid::now_v7(),
            etablissement_id: jeu.etablissement_id,
            unite_id,
            formule_id,
            debut_client: time::macros::datetime!(2027-06-01 14:00 UTC),
            fin_client: time::macros::datetime!(2027-06-05 12:00 UTC),
        })
        .await
        .expect("première attribution");

    let refusee = Uuid::now_v7();
    let erreur = occupation
        .attribuer(kaya_hebergement::occupation::DemandeAttribution {
            id: refusee,
            etablissement_id: jeu.etablissement_id,
            unite_id,
            formule_id,
            debut_client: time::macros::datetime!(2027-06-02 10:00 UTC),
            fin_client: time::macros::datetime!(2027-06-03 10:00 UTC),
        })
        .await
        .expect_err("l'attribution chevauchante doit être refusée");
    assert_eq!(erreur.code(), "unite_deja_occupee");

    assert!(
        types_evenements(&pool_owner, jeu.tenant_id, refusee)
            .await
            .is_empty(),
        "un refus a laissé un événement au grand livre : la reconstitution compterait une \
         attribution qui n'a jamais eu lieu"
    );

    let mut tx = pool_owner.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, jeu.tenant_id)
        .await
        .expect("tenant");
    let total: i64 =
        sqlx::query_scalar(r#"SELECT COUNT(*) FROM hebergement.occupation WHERE unite_id = $1"#)
            .bind(unite_id)
            .fetch_one(&mut *tx)
            .await
            .expect("comptage");
    assert_eq!(total, 1, "le refus a laissé une ligne en base");
}

// =================================================================================================
//  ★ Cycle 006 — les événements du séjour
// =================================================================================================

/// ★ **`heb.sejour.ouvert` porte de quoi reconstituer l'opération SANS consulter aucune table.**
///
/// C'est TRX-02 : *« la charge utile financière est dénormalisée »*. Une charge utile qui ne
/// porterait que des identifiants obligerait la projection à relire `note_sejour`, `ligne_sejour`
/// et `formule` — trois tables dont le contenu aura changé quand la relecture aura lieu, puisque
/// le grand livre est **rétroactif** et sa rétention **illimitée**.
///
/// # Les trois contrôles, et le troisième est celui qui ne se devine pas
///
/// 1. l'événement est **émis** ;
/// 2. tout montant y est un **entier d'unité mineure** sous le nommage `<nom>_mineur`, avec sa
///    devise au même niveau (porte **P-10**, jusque dans le JSONB) ;
/// 3. ★ **aucun numéro de pièce d'identité n'y figure** — le grand livre est immuable et à
///    rétention illimitée : une donnée sensible qui y entre ne peut **jamais** en sortir, et la
///    rétention de 90 jours de TRX-06 deviendrait inapplicable sur la copie.
#[actix_web::test]
async fn p05_cycle_006_l_ouverture_d_un_sejour_emet_deux_evenements_reconstituables() {
    let pool_owner = commun::pool_owner().await;
    let decor = decor_sejour(&pool_owner, "P-05 séjour").await;
    let cx = commun::compte_connecte(
        &pool_owner,
        decor.jeu,
        "Yao",
        &[("receptionniste", Some(decor.jeu.etablissement_id))],
    )
    .await;
    let app = monter_application!(commun::pool_app().await);

    let sejour_id = Uuid::now_v7();
    let debut = time::OffsetDateTime::now_utc() + time::Duration::hours(1);
    let requete = actix_web::test::TestRequest::post()
        .uri(&format!(
            "/api/v1/etablissements/{}/sejours",
            decor.jeu.etablissement_id
        ))
        .insert_header(("authorization", cx.bearer.clone()))
        .set_json(serde_json::json!({
            "id": sejour_id,
            "unite_id": decor.unite_id,
            "formule_id": decor.formule_id,
            "debut_client": debut.format(&time::format_description::well_known::Rfc3339).unwrap(),
            "fin_client": (debut + time::Duration::hours(24))
                .format(&time::format_description::well_known::Rfc3339).unwrap(),
            "accompagnants": [{
                "id": Uuid::now_v7(),
                "nom": "Adjoua",
                // ⚠️ Une pièce est fournie : c'est ce qui rend le contrôle 3 non trivial.
                "type_piece": "CNI",
                "numero_piece": "CI00777888",
            }],
        }))
        .to_request();
    let reponse = actix_web::test::call_service(&app, requete).await;
    assert_eq!(reponse.status(), 201, "l'ouverture doit réussir");

    // ── 1 · les DEUX événements sont émis ─────────────────────────────────────────────────────
    let mut tx = pool_owner.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, decor.jeu.tenant_id)
        .await
        .expect("tenant");

    let charges: Vec<(String, serde_json::Value)> = sqlx::query_as(
        r#"
        SELECT type_evenement, payload
        FROM synchronisation.evenement_outbox
        WHERE tenant_id = $1
        ORDER BY sequence_etablissement
        "#,
    )
    .bind(decor.jeu.tenant_id)
    .fetch_all(&mut *tx)
    .await
    .expect("lecture du grand livre");
    tx.rollback().await.expect("rollback");

    for attendu in [
        "heb.sejour.ouvert",
        "heb.fiche_police.generee",
        "sej.accompagnant.ajoute",
    ] {
        assert!(
            charges.iter().any(|(t, _)| t == attendu),
            "l'événement « {attendu} » n'a pas été émis. Types trouvés : {:?}",
            charges.iter().map(|(t, _)| t.as_str()).collect::<Vec<_>>()
        );
    }

    let ouverture = charges
        .iter()
        .find(|(t, _)| t == "heb.sejour.ouvert")
        .map(|(_, p)| p)
        .expect("heb.sejour.ouvert");

    // ── 2 · la charge utile RECONSTITUE l'opération ───────────────────────────────────────────
    //
    // Sans ces champs, une projection devrait relire trois tables dont le contenu aura changé —
    // le grand livre est rétroactif, sa rétention illimitée.
    for cle in ["sejour_id", "unite_id", "formule_id", "total_mineur", "devise", "lignes"] {
        assert!(
            !ouverture[cle].is_null(),
            "la charge utile de `heb.sejour.ouvert` ne porte pas « {cle} » : l'opération ne se \
             reconstitue pas sans consulter une autre table (TRX-02). Charge : {ouverture}"
        );
    }

    assert!(
        ouverture["total_mineur"].is_i64(),
        "`total_mineur` doit être un ENTIER d'unité mineure (P-10, jusque dans le JSONB), \
         jamais un décimal ni une chaîne formatée. Valeur : {}",
        ouverture["total_mineur"]
    );
    let lignes = ouverture["lignes"].as_array().expect("lignes est un tableau");
    assert!(!lignes.is_empty(), "la note doit porter sa ligne d'hébergement");
    for ligne in lignes {
        assert!(
            ligne["montant_mineur"].is_i64() && ligne["prix_unitaire_mineur"].is_i64(),
            "tout montant de ligne est un entier d'unité mineure : {ligne}"
        );
        assert!(
            ligne["devise"].is_string(),
            "la devise voyage AU MÊME NIVEAU que les montants, toujours (principe V) : {ligne}"
        );
    }

    // ── 3 · ★ AUCUN numéro de pièce, dans AUCUNE charge utile ─────────────────────────────────
    for (type_evenement, charge) in &charges {
        let brut = charge.to_string();
        assert!(
            !brut.contains("CI00777888"),
            "★ un numéro de pièce d'identité est entré dans le grand livre, par « \
             {type_evenement} ». Le grand livre est IMMUABLE et à rétention ILLIMITÉE : la donnée \
             ne peut jamais en sortir, et la rétention de 90 jours de TRX-06 devient inapplicable \
             sur la copie. Charge : {brut}"
        );
        for suspect in ["numero_piece", "numeroPiece"] {
            assert!(
                !brut.contains(suspect),
                "la charge utile de « {type_evenement} » porte une clé « {suspect} » : même vide, \
                 elle invite le prochain cycle à la remplir. Charge : {brut}"
            );
        }
    }
}

/// **Un refus d'ouverture ne laisse aucun événement.**
///
/// La reconstitution compterait un séjour qui n'a jamais eu lieu — et le grand livre étant
/// permanent, l'erreur ne se corrige pas : elle se compense, ce qui suppose de l'avoir vue.
#[actix_web::test]
async fn p05_cycle_006_un_refus_d_ouverture_ne_laisse_aucun_evenement() {
    let pool_owner = commun::pool_owner().await;
    let decor = decor_sejour(&pool_owner, "P-05 séjour refusé").await;
    let cx = commun::compte_connecte(
        &pool_owner,
        decor.jeu,
        "Yao",
        &[("receptionniste", Some(decor.jeu.etablissement_id))],
    )
    .await;
    let app = monter_application!(commun::pool_app().await);
    let debut = time::OffsetDateTime::now_utc() + time::Duration::hours(1);

    let corps = |id: Uuid| {
        serde_json::json!({
            "id": id,
            "unite_id": decor.unite_id,
            "formule_id": decor.formule_id,
            "debut_client": debut.format(&time::format_description::well_known::Rfc3339).unwrap(),
            "fin_client": (debut + time::Duration::hours(24))
                .format(&time::format_description::well_known::Rfc3339).unwrap(),
        })
    };
    let envoyer = |corps: serde_json::Value| {
        let app = &app;
        let bearer = cx.bearer.clone();
        let etablissement = decor.jeu.etablissement_id;
        async move {
            let requete = actix_web::test::TestRequest::post()
                .uri(&format!("/api/v1/etablissements/{etablissement}/sejours"))
                .insert_header(("authorization", bearer))
                .set_json(&corps)
                .to_request();
            actix_web::test::call_service(app, requete).await.status().as_u16()
        }
    };

    assert_eq!(envoyer(corps(Uuid::now_v7())).await, 201);

    let refuse = Uuid::now_v7();
    assert_eq!(
        envoyer(corps(refuse)).await,
        409,
        "la seconde ouverture sur la même période doit être refusée"
    );

    assert!(
        types_evenements(&pool_owner, decor.jeu.tenant_id, refuse)
            .await
            .is_empty(),
        "un refus a laissé un événement : la reconstitution compterait un séjour qui n'a jamais \
         eu lieu, et le grand livre étant permanent, l'erreur ne se corrige pas — elle se \
         compense, ce qui suppose de l'avoir vue"
    );
}

/// Décor minimal pour les tests de séjour de ce fichier.
struct DecorSejour {
    jeu: commun::JeuTenant,
    unite_id: Uuid,
    formule_id: Uuid,
}

async fn decor_sejour(pool: &sqlx::PgPool, nom: &str) -> DecorSejour {
    let jeu = commun::creer_tenant(pool, nom).await;

    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, jeu.tenant_id)
        .await
        .expect("tenant");

    sqlx::query(
        r#"
        INSERT INTO etablissements.etablissement_module
            (id, tenant_id, etablissement_id, module_code, module_implemente)
        VALUES ($1, $2, $3, 'HEBERGEMENT', true) ON CONFLICT DO NOTHING
        "#,
    )
    .bind(Uuid::now_v7())
    .bind(jeu.tenant_id)
    .bind(jeu.etablissement_id)
    .execute(&mut *tx)
    .await
    .expect("module");

    let categorie_id = Uuid::now_v7();
    sqlx::query(
        r#"
        INSERT INTO hebergement.categorie (id, tenant_id, etablissement_id, nom, capacite_accueil)
        VALUES ($1, $2, $3, 'Standard', 2)
        "#,
    )
    .bind(categorie_id)
    .bind(jeu.tenant_id)
    .bind(jeu.etablissement_id)
    .execute(&mut *tx)
    .await
    .expect("catégorie");

    let unite_id = Uuid::now_v7();
    sqlx::query(
        r#"
        INSERT INTO hebergement.unite (id, tenant_id, etablissement_id, categorie_id, code, etage)
        VALUES ($1, $2, $3, $4, 'A1', 1)
        "#,
    )
    .bind(unite_id)
    .bind(jeu.tenant_id)
    .bind(jeu.etablissement_id)
    .bind(categorie_id)
    .execute(&mut *tx)
    .await
    .expect("unité");

    let formule_id = Uuid::now_v7();
    sqlx::query(
        r#"
        INSERT INTO hebergement.formule
            (id, tenant_id, etablissement_id, categorie_id, famille, prix_mineur,
             assujettie_taxe_nuitee)
        VALUES ($1, $2, $3, $4, 'NUITEE', 12500, false)
        "#,
    )
    .bind(formule_id)
    .bind(jeu.tenant_id)
    .bind(jeu.etablissement_id)
    .bind(categorie_id)
    .execute(&mut *tx)
    .await
    .expect("formule");

    tx.commit().await.expect("commit");

    DecorSejour {
        jeu,
        unite_id,
        formule_id,
    }
}

/// ★ **`heb.sejour.clos` porte le total, TOUTES les lignes, les ajustements ET le constat.**
///
/// TRX-02 : *l'opération se reconstitue **sans consulter aucune autre table**.* Le grand livre est
/// rétroactif et sa rétention illimitée : une projection qui relirait `note_sejour`,
/// `ligne_sejour` et `taxe_sejour_constat` lirait des tables dont le contenu aura changé — et sur
/// le constat, elle lirait une valeur que FIS-03 aura entre-temps alimentée.
#[actix_web::test]
async fn p05_cycle_006_la_cloture_emet_un_evenement_reconstituable() {
    let pool_owner = commun::pool_owner().await;
    let decor = decor_sejour(&pool_owner, "P-05 clôture").await;
    let cx = commun::compte_connecte(
        &pool_owner,
        decor.jeu,
        "Yao",
        &[("receptionniste", Some(decor.jeu.etablissement_id))],
    )
    .await;
    let app = monter_application!(commun::pool_app().await);

    let sejour_id = Uuid::now_v7();
    let debut = time::OffsetDateTime::now_utc() - time::Duration::hours(2);
    let requete = actix_web::test::TestRequest::post()
        .uri(&format!(
            "/api/v1/etablissements/{}/sejours",
            decor.jeu.etablissement_id
        ))
        .insert_header(("authorization", cx.bearer.clone()))
        .set_json(serde_json::json!({
            "id": sejour_id,
            "unite_id": decor.unite_id,
            "formule_id": decor.formule_id,
            "debut_client": debut.format(&time::format_description::well_known::Rfc3339).unwrap(),
            "fin_client": (debut + time::Duration::hours(24))
                .format(&time::format_description::well_known::Rfc3339).unwrap(),
        }))
        .to_request();
    assert_eq!(actix_web::test::call_service(&app, requete).await.status(), 201);

    let requete = actix_web::test::TestRequest::post()
        .uri(&format!(
            "/api/v1/etablissements/{}/sejours/{sejour_id}/depart",
            decor.jeu.etablissement_id
        ))
        .insert_header(("authorization", cx.bearer.clone()))
        .to_request();
    assert_eq!(actix_web::test::call_service(&app, requete).await.status(), 200);

    let mut tx = pool_owner.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, decor.jeu.tenant_id)
        .await
        .expect("tenant");
    let charge: serde_json::Value = sqlx::query_scalar(
        r#"
        SELECT payload FROM synchronisation.evenement_outbox
        WHERE tenant_id = $1 AND type_evenement = 'heb.sejour.clos'
        "#,
    )
    .bind(decor.jeu.tenant_id)
    .fetch_one(&mut *tx)
    .await
    .expect("l'événement `heb.sejour.clos` doit être émis");
    tx.rollback().await.expect("rollback");

    for cle in ["sejour_id", "clos_le", "duree_reelle_minutes", "total_mineur", "devise", "lignes"] {
        assert!(
            !charge[cle].is_null(),
            "la charge utile de `heb.sejour.clos` ne porte pas « {cle} » : l'opération ne se \
             reconstitue pas sans consulter une autre table (TRX-02). Charge : {charge}"
        );
    }

    assert!(
        charge["total_mineur"].is_i64(),
        "`total_mineur` doit être un ENTIER d'unité mineure (P-10, jusque dans le JSONB) : {charge}"
    );

    // ★ **Le constat voyage AVEC l'événement**, et son montant y est `null`.
    let constat = &charge["constat_taxe"];
    assert!(
        !constat.is_null(),
        "★ le constat de taxe doit voyager avec l'événement de clôture. Sans lui, FIS-03 devrait \
         relire `taxe_sejour_constat` — dont il aura entre-temps alimenté le montant, ce qui rend \
         la relecture circulaire. Charge : {charge}"
    );
    assert!(
        constat["nuitees_assujetties"].is_null() && constat["montant_mineur"].is_null(),
        "★ un montant de taxe est parti dans le grand livre. Décider quelles nuits sont \
         assujetties est une RÈGLE FISCALE (P-12), et le grand livre est IMMUABLE : une valeur \
         fausse qui y entre ne peut jamais en sortir. Constat : {constat}"
    );
    for cle in ["nuits_constatees", "assujettie_taxe_nuitee", "classement_etablissement", "commune"] {
        assert!(
            !constat[cle].is_null(),
            "le constat doit porter « {cle} » — c'est le paramétrage RECOPIÉ qui rend le figeage \
             vrai. Constat : {constat}"
        );
    }
}

// =================================================================================================
//  Cycle 006 — les cinq types que le recollement réclamait nommément
// =================================================================================================

/// **La fiche client émet `sej.client.cree` puis `sej.client.modifie`, et JAMAIS le numéro.**
///
/// ⚠️ **Ces deux charges utiles sont les plus sensibles du produit.** L'outbox est un grand livre
/// à rétention **illimitée** et **immuable** : un numéro de pièce qui y entre ne peut **jamais**
/// en sortir, et la rétention de 90 jours de TRX-06 deviendrait inapplicable sur la copie.
#[actix_web::test]
async fn p05_cycle_006_la_fiche_client_emet_sans_jamais_le_numero_de_piece() {
    let pool_owner = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool_owner, "P-05 fiche client").await;
    let cx = commun::compte_connecte(
        &pool_owner,
        jeu,
        "Yao",
        &[("receptionniste", Some(jeu.etablissement_id))],
    )
    .await;
    let app = monter_application!(commun::pool_app().await);

    const NUMERO: &str = "CI00246813";
    let client_id = Uuid::now_v7();

    let requete = actix_web::test::TestRequest::post()
        .uri("/api/v1/clients")
        .insert_header(("authorization", cx.bearer.clone()))
        .set_json(serde_json::json!({
            "id": client_id,
            "nom": "Traoré",
            "type_piece": "CNI",
            "numero_piece": NUMERO,
        }))
        .to_request();
    assert_eq!(actix_web::test::call_service(&app, requete).await.status(), 201);

    let requete = actix_web::test::TestRequest::patch()
        .uri(&format!("/api/v1/clients/{client_id}"))
        .insert_header(("authorization", cx.bearer.clone()))
        .set_json(serde_json::json!({ "nom": "Traoré Konan", "numero_piece": NUMERO }))
        .to_request();
    assert_eq!(actix_web::test::call_service(&app, requete).await.status(), 200);

    // Une préférence — classe A, append-only.
    let requete = actix_web::test::TestRequest::post()
        .uri(&format!("/api/v1/clients/{client_id}/preferences"))
        .insert_header(("authorization", cx.bearer.clone()))
        .set_json(serde_json::json!({ "id": Uuid::now_v7(), "texte": "Chambre calme" }))
        .to_request();
    assert_eq!(actix_web::test::call_service(&app, requete).await.status(), 201);

    let mut tx = pool_owner.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, jeu.tenant_id)
        .await
        .expect("tenant");
    let charges: Vec<(String, serde_json::Value)> = sqlx::query_as(
        r#"
        SELECT type_evenement, payload FROM synchronisation.evenement_outbox
        WHERE tenant_id = $1 ORDER BY sequence_etablissement
        "#,
    )
    .bind(jeu.tenant_id)
    .fetch_all(&mut *tx)
    .await
    .expect("lecture du grand livre");
    tx.rollback().await.expect("rollback");

    for attendu in ["sej.client.cree", "sej.client.modifie", "sej.preference.enregistree"] {
        assert!(
            charges.iter().any(|(t, _)| t == attendu),
            "l'événement « {attendu} » n'a pas été émis. Types : {:?}",
            charges.iter().map(|(t, _)| t.as_str()).collect::<Vec<_>>()
        );
    }

    for (type_evenement, charge) in &charges {
        let brut = charge.to_string();
        assert!(
            !brut.contains(NUMERO),
            "★ un numéro de pièce d'identité est entré dans le grand livre par « \
             {type_evenement} ». Le grand livre est IMMUABLE et à rétention ILLIMITÉE : la donnée \
             ne peut jamais en sortir, et la rétention de 90 jours de TRX-06 devient inapplicable \
             sur la copie. Charge : {brut}"
        );
    }
}

/// **La prolongation et le changement d'unité émettent leur événement, montants en ENTIERS.**
#[actix_web::test]
async fn p05_cycle_006_la_prolongation_et_le_changement_d_unite_emettent() {
    let pool_owner = commun::pool_owner().await;
    let decor = decor_sejour(&pool_owner, "P-05 prolongation").await;

    // Une seconde chambre, pour le changement d'unité.
    let unite_bis = Uuid::now_v7();
    {
        let mut tx = pool_owner.begin().await.expect("transaction");
        kaya_etablissements::tenant_context::poser_tenant(&mut tx, decor.jeu.tenant_id)
            .await
            .expect("tenant");
        let categorie: Uuid = sqlx::query_scalar(
            "SELECT categorie_id FROM hebergement.unite WHERE id = $1",
        )
        .bind(decor.unite_id)
        .fetch_one(&mut *tx)
        .await
        .expect("catégorie");
        sqlx::query(
            r#"
            INSERT INTO hebergement.unite
                (id, tenant_id, etablissement_id, categorie_id, code, etage)
            VALUES ($1, $2, $3, $4, 'A2', 1)
            "#,
        )
        .bind(unite_bis)
        .bind(decor.jeu.tenant_id)
        .bind(decor.jeu.etablissement_id)
        .bind(categorie)
        .execute(&mut *tx)
        .await
        .expect("seconde unité");
        tx.commit().await.expect("commit");
    }

    let cx = commun::compte_connecte(
        &pool_owner,
        decor.jeu,
        "Adjoua",
        &[("receptionniste", Some(decor.jeu.etablissement_id))],
    )
    .await;
    let app = monter_application!(commun::pool_app().await);

    let sejour_id = Uuid::now_v7();
    let debut = time::OffsetDateTime::now_utc() - time::Duration::hours(1);
    let requete = actix_web::test::TestRequest::post()
        .uri(&format!(
            "/api/v1/etablissements/{}/sejours",
            decor.jeu.etablissement_id
        ))
        .insert_header(("authorization", cx.bearer.clone()))
        .set_json(serde_json::json!({
            "id": sejour_id,
            "unite_id": decor.unite_id,
            "formule_id": decor.formule_id,
            "debut_client": debut.format(&time::format_description::well_known::Rfc3339).unwrap(),
            "fin_client": (debut + time::Duration::hours(24))
                .format(&time::format_description::well_known::Rfc3339).unwrap(),
        }))
        .to_request();
    assert_eq!(actix_web::test::call_service(&app, requete).await.status(), 201);

    let requete = actix_web::test::TestRequest::post()
        .uri(&format!(
            "/api/v1/etablissements/{}/sejours/{sejour_id}/prolongation",
            decor.jeu.etablissement_id
        ))
        .insert_header(("authorization", cx.bearer.clone()))
        .set_json(serde_json::json!({
            "id": Uuid::now_v7(),
            "nouvelle_fin_client": (debut + time::Duration::hours(48))
                .format(&time::format_description::well_known::Rfc3339).unwrap(),
        }))
        .to_request();
    assert_eq!(actix_web::test::call_service(&app, requete).await.status(), 200);

    let requete = actix_web::test::TestRequest::post()
        .uri(&format!(
            "/api/v1/etablissements/{}/sejours/{sejour_id}/changement-unite",
            decor.jeu.etablissement_id
        ))
        .insert_header(("authorization", cx.bearer.clone()))
        .set_json(serde_json::json!({ "id": Uuid::now_v7(), "unite_cible_id": unite_bis }))
        .to_request();
    assert_eq!(actix_web::test::call_service(&app, requete).await.status(), 200);

    let types = types_evenements(&pool_owner, decor.jeu.tenant_id, sejour_id).await;
    for attendu in ["heb.sejour.prolonge", "heb.sejour.unite_changee"] {
        assert!(
            types.iter().any(|t| t == attendu),
            "l'événement « {attendu} » n'a pas été émis. Types : {types:?}"
        );
    }

    // Les montants sont des **entiers d'unité mineure**, jusque dans le JSONB (P-10).
    let mut tx = pool_owner.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, decor.jeu.tenant_id)
        .await
        .expect("tenant");
    let charges: Vec<(serde_json::Value,)> = sqlx::query_as(
        r#"
        SELECT payload FROM synchronisation.evenement_outbox
        WHERE agregat_id = $1
          AND type_evenement IN ('heb.sejour.prolonge', 'heb.sejour.unite_changee')
        "#,
    )
    .bind(sejour_id)
    .fetch_all(&mut *tx)
    .await
    .expect("lecture");
    tx.rollback().await.expect("rollback");

    for (charge,) in &charges {
        assert!(
            charge["total_mineur"].is_i64() && charge["montant_ajoute_mineur"].is_i64(),
            "tout montant est un ENTIER d'unité mineure, jusque dans le JSONB (P-10) : {charge}"
        );
        assert!(
            charge["devise"].is_string(),
            "la devise voyage AU MÊME NIVEAU que les montants, toujours : {charge}"
        );
    }
}
