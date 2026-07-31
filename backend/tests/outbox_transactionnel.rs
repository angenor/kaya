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
