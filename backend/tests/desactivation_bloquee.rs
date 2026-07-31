//! **FR-016 / T014** — un service portant des opérations en cours ne se désactive pas.
//!
//! # Ce test existe parce que le point d'accrochage est VIDE
//!
//! `ObstacleDesactivation` est **défini** dans `socle/etablissements`, **implémenté** par les
//! verticales, **injecté** à l'assemblage. À ce cycle aucune verticale ne crée d'opération : la
//! liste est vide et la désactivation est libre. C'est exact, et c'est le problème — un point
//! d'accrochage jamais exercé est indistinguable d'un point d'accrochage cassé.
//!
//! Le cycle 001 l'a montré sur les portes : *une porte qui ne trouve jamais rien est
//! indistinguable d'une porte qui n'a rien à trouver.* Un remaniement qui supprimerait l'appel aux
//! obstacles — ou qui l'appellerait après la bascule — passerait toutes les autres suites de tests
//! sans qu'aucune ne bouge.
//!
//! Ce fichier enregistre donc un **obstacle factice** et constate trois choses : le refus, le fait
//! qu'il **nomme** l'obstacle, et le fait qu'**aucune bascule n'a eu lieu** en base.

mod commun;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use uuid::Uuid;

use kaya_etablissements::modules::{BasculerService, ErreurModules, ServiceModules};
use kaya_etablissements::{ErreurRegistre, Obstacle, ObstacleDesactivation};
use kaya_synchronisation::outbox::PgOutboxWriter;

/// Une verticale imaginaire qui déclare toujours trois séjours en cours.
///
/// Elle compte ses appels : un obstacle enregistré mais **jamais interrogé** produirait exactement
/// le même résultat qu'aucun obstacle, et le test passerait pour la mauvaise raison le jour où
/// l'appel disparaîtrait du service.
struct SejoursEnCours {
    appels: AtomicUsize,
}

#[async_trait::async_trait]
impl ObstacleDesactivation for SejoursEnCours {
    async fn obstacles(
        &self,
        _etablissement_id: Uuid,
        module_code: &str,
    ) -> Result<Vec<Obstacle>, ErreurRegistre> {
        self.appels.fetch_add(1, Ordering::SeqCst);
        Ok(vec![Obstacle {
            module_code: module_code.to_owned(),
            // **Clé i18n, jamais une phrase** : le texte vit dans le catalogue de traductions.
            motif_cle: "services.obstacle.sejours_en_cours".to_owned(),
            nombre: 3,
        }])
    }
}

/// Une verticale qui ne s'oppose à rien — le cas de toutes celles de ce cycle.
struct AucunObstacle;

#[async_trait::async_trait]
impl ObstacleDesactivation for AucunObstacle {
    async fn obstacles(
        &self,
        _etablissement_id: Uuid,
        _module_code: &str,
    ) -> Result<Vec<Obstacle>, ErreurRegistre> {
        Ok(Vec::new())
    }
}

/// Un obstacle enregistré **refuse** la désactivation, et le refus le **nomme**.
#[tokio::test]
async fn un_obstacle_enregistre_refuse_la_desactivation_en_le_nommant() {
    let pool_owner = commun::pool_owner().await;
    let pool_app = commun::pool_app().await;
    let jeu = commun::creer_tenant(&pool_owner, "obstacle à la désactivation").await;

    let obstacle = Arc::new(SejoursEnCours {
        appels: AtomicUsize::new(0),
    });

    // Activation par un service SANS obstacle : c'est la désactivation qui est le sujet.
    let service = ServiceModules::nouveau(pool_app.clone(), PgOutboxWriter::nouveau());
    service
        .basculer(
            jeu.tenant_id,
            jeu.etablissement_id,
            "HEBERGEMENT",
            BasculerService {
                id: Uuid::now_v7(),
                actif: true,
            },
        )
        .await
        .expect("activation de HEBERGEMENT");

    let service_garde = ServiceModules::nouveau(pool_app.clone(), PgOutboxWriter::nouveau())
        .avec_obstacle(obstacle.clone());

    let refus = service_garde
        .basculer(
            jeu.tenant_id,
            jeu.etablissement_id,
            "HEBERGEMENT",
            BasculerService {
                id: Uuid::now_v7(),
                actif: false,
            },
        )
        .await;

    let Err(ErreurModules::DesactivationBloquee(obstacles)) = refus else {
        panic!(
            "la désactivation a été ACCEPTÉE malgré un obstacle enregistré. FR-016 exige qu'un \
             service portant des opérations en cours ne puisse pas être désactivé — et le point \
             d'accrochage vient d'être contourné sans que rien d'autre ne le signale."
        );
    };

    assert_eq!(
        obstacle.appels.load(Ordering::SeqCst),
        1,
        "l'obstacle n'a pas été interrogé exactement une fois : un obstacle enregistré mais jamais \
         appelé produit le même résultat qu'aucun obstacle"
    );

    assert_eq!(obstacles.len(), 1, "un seul obstacle était enregistré");
    assert_eq!(obstacles[0].module_code, "HEBERGEMENT");
    assert_eq!(
        obstacles[0].motif_cle, "services.obstacle.sejours_en_cours",
        "le refus doit porter la CLÉ i18n du motif — une phrase traverserait l'API jusqu'à l'écran \
         sans passer par le catalogue de traductions, donc sans anglais"
    );
    assert_eq!(
        obstacles[0].nombre, 3,
        "le nombre est séparé du motif pour que la phrase se compose dans la langue de \
         l'utilisateur, où le pluriel ne s'accorde pas partout de la même façon"
    );

    // **Aucune bascule n'a eu lieu.** Un refus qui laisserait la ligne modifiée serait pire qu'un
    // refus absent : le service serait désactivé ET l'appelant croirait l'inverse.
    let actifs = service_garde
        .services_actifs(jeu.tenant_id, jeu.etablissement_id)
        .await
        .expect("lecture des services actifs");
    assert!(
        actifs.iter().any(|s| s.module_code == "HEBERGEMENT"),
        "le service a été désactivé alors que l'opération a été refusée"
    );
}

/// **Le cas de ce cycle** — aucun obstacle enregistré, la désactivation passe.
///
/// Sans ce test, la seule garantie serait « un obstacle bloque », ce qui serait aussi vrai d'un
/// service qui refuserait *toujours* de désactiver. La porte doit savoir dire non **et** oui.
#[tokio::test]
async fn sans_obstacle_la_desactivation_passe() {
    let pool_owner = commun::pool_owner().await;
    let pool_app = commun::pool_app().await;
    let jeu = commun::creer_tenant(&pool_owner, "désactivation libre").await;

    let service = ServiceModules::nouveau(pool_app.clone(), PgOutboxWriter::nouveau())
        .avec_obstacle(Arc::new(AucunObstacle));

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
        .expect("activation de BAR");

    service
        .basculer(
            jeu.tenant_id,
            jeu.etablissement_id,
            "BAR",
            BasculerService {
                id: Uuid::now_v7(),
                actif: false,
            },
        )
        .await
        .expect("la désactivation doit passer quand aucun obstacle ne s'y oppose");

    let actifs = service
        .services_actifs(jeu.tenant_id, jeu.etablissement_id)
        .await
        .expect("lecture des services actifs");
    assert!(
        !actifs.iter().any(|s| s.module_code == "BAR"),
        "BAR figure encore parmi les services actifs après désactivation"
    );
}

/// **La désactivation ne supprime rien, et la réactivation restitue l'état antérieur** (FR-015).
///
/// C'est la propriété qui justifie l'absence de `DELETE` dans les privilèges de la table. Une
/// désactivation qui supprimerait la ligne perdrait les déclarations de capacité et les surcharges
/// de configuration qui s'y rattachent — et l'exploitant qui réactive un service découvrirait
/// qu'il doit tout ressaisir.
#[tokio::test]
async fn la_reactivation_restitue_les_capacites_declarees() {
    use kaya_etablissements::modules::DeclarerCapacite;

    let pool_owner = commun::pool_owner().await;
    let pool_app = commun::pool_app().await;
    let jeu = commun::creer_tenant(&pool_owner, "réactivation").await;
    let service = ServiceModules::nouveau(pool_app.clone(), PgOutboxWriter::nouveau());

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
        .expect("activation");

    service
        .declarer_capacite(
            jeu.tenant_id,
            jeu.etablissement_id,
            "RESTAURATION",
            DeclarerCapacite {
                id: Uuid::now_v7(),
                capacite_code: "STOCK".to_owned(),
                profil_code: "SIMPLE".to_owned(),
            },
        )
        .await
        .expect("déclaration de STOCK/SIMPLE");

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

    // Service désactivé : sa déclaration est **inerte**, pas supprimée. Le trait ne la rend plus.
    let inerte = service
        .capacites_du_service(jeu.tenant_id, jeu.etablissement_id, "RESTAURATION")
        .await;
    assert!(
        matches!(inerte, Err(ErreurModules::ModuleNonActif(_))),
        "un service désactivé ne doit rendre aucune capacité : elles sont inertes"
    );

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
        .expect("réactivation");

    let restituees = service
        .capacites_du_service(jeu.tenant_id, jeu.etablissement_id, "RESTAURATION")
        .await
        .expect("lecture après réactivation");

    assert_eq!(
        restituees.len(),
        1,
        "la réactivation n'a pas restitué la déclaration de capacité : l'exploitant devrait tout \
         ressaisir. C'est exactement ce que l'absence de DELETE sur la table doit empêcher."
    );
    assert_eq!(restituees[0].capacite_code, "STOCK");
    assert_eq!(restituees[0].profil_code, "SIMPLE");
}
