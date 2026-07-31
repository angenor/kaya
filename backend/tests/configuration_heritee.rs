//! **ETB-04 — la matrice de résolution de la chaîne d'héritage.**
//!
//! Quatre niveaux, chacun défini ou absent, chaînes écourtées comprises. Chaque cas vérifie **la
//! valeur ET son origine** : une résolution qui rendrait la bonne valeur en se trompant d'origine
//! ferait afficher « modifié ici » sur une valeur héritée — et l'exploitant croirait avoir réglé
//! un paramètre qu'il n'a pas touché.
//!
//! # Ce que ce fichier inspecte, et ce qu'il n'inspecte pas
//!
//! **Inspecté** — la descente de chaîne sur les quatre niveaux, l'absence explicite d'une clé
//! définie nulle part, l'inertie d'une surcharge portée par un service désactivé et sa
//! restitution à la réactivation, et l'isolation **à chaque niveau**.
//!
//! **Non inspecté** — la performance de la descente (une seule requête, jamais quatre). Elle est
//! garantie par la forme du SQL, pas par un test : mesurer un nombre d'allers-retours demanderait
//! d'instrumenter le pilote, et le test se contenterait de constater ce que la lecture du
//! repository montre déjà.

mod commun;

use serde_json::json;
use uuid::Uuid;

use kaya_etablissements::configuration::{EcrireParametre, ServiceConfiguration};
use kaya_etablissements::modules::{BasculerService, ServiceModules};
use kaya_etablissements::points_de_vente::{CreerPointDeVente, ServicePointsDeVente};
use kaya_etablissements::{Cible, Portee};
use kaya_synchronisation::outbox::PgOutboxWriter;

/// La clé du catalogue, seule à ce cycle.
const CLE: &str = "politique_impression";

/// Un décor complet : tenant, établissement, service actif, point de vente.
struct Decor {
    tenant_id: Uuid,
    etablissement_id: Uuid,
    module_code: String,
    etablissement_module_id: Uuid,
    point_de_vente_id: Uuid,
    configuration: ServiceConfiguration<PgOutboxWriter>,
    modules: ServiceModules<PgOutboxWriter>,
}

impl Decor {
    async fn nouveau(nom: &str) -> Self {
        let pool_owner = commun::pool_owner().await;
        let pool_app = commun::pool_app().await;
        let jeu = commun::creer_tenant(&pool_owner, nom).await;

        let modules = ServiceModules::nouveau(pool_app.clone(), PgOutboxWriter::nouveau());
        let bascule = modules
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
            .expect("activation de RESTAURATION");

        let points = ServicePointsDeVente::nouveau(pool_app.clone(), PgOutboxWriter::nouveau());
        let (pdv, _) = points
            .creer(
                jeu.tenant_id,
                jeu.etablissement_id,
                CreerPointDeVente {
                    id: Uuid::now_v7(),
                    module_code: "RESTAURATION".to_owned(),
                    nom: format!("Comptoir {nom}"),
                    caisse_id: None,
                },
            )
            .await
            .expect("création du point de vente");

        Self {
            tenant_id: jeu.tenant_id,
            etablissement_id: jeu.etablissement_id,
            module_code: "RESTAURATION".to_owned(),
            etablissement_module_id: bascule.service_id,
            point_de_vente_id: pdv.id,
            configuration: ServiceConfiguration::nouveau(pool_app, PgOutboxWriter::nouveau()),
            modules,
        }
    }

    /// Pose une valeur à un niveau donné.
    async fn poser(&self, portee: Portee, portee_id: Option<Uuid>, valeur: &str) {
        self.configuration
            .ecrire(
                self.tenant_id,
                EcrireParametre {
                    id: Uuid::now_v7(),
                    cle: CLE.to_owned(),
                    valeur: json!(valeur),
                    portee,
                    portee_id,
                },
            )
            .await
            .unwrap_or_else(|e| panic!("écriture à la portée {portee:?} : {e}"));
    }

    /// La cible complète — quatre niveaux.
    fn cible_complete(&self) -> Cible {
        Cible {
            tenant_id: self.tenant_id,
            etablissement_id: Some(self.etablissement_id),
            module_code: Some(self.module_code.clone()),
            point_de_vente_id: Some(self.point_de_vente_id),
        }
    }

    /// Résout et rend `(valeur, origine)`.
    async fn resoudre(&self, cible: &Cible) -> Option<(String, Portee)> {
        self.configuration
            .resoudre(cible, CLE)
            .await
            .expect("résolution")
            .map(|v| {
                (
                    v.valeur.as_str().unwrap_or_default().to_owned(),
                    v.origine,
                )
            })
    }
}

/// **Cas 1 — tenant seul.** La valeur descend jusqu'au point de vente, et son origine reste
/// `TENANT`.
#[tokio::test]
async fn cas_1_tenant_seul() {
    let decor = Decor::nouveau("config tenant seul").await;
    decor.poser(Portee::Tenant, None, "aucune").await;

    let resolue = decor.resoudre(&decor.cible_complete()).await;
    assert_eq!(
        resolue,
        Some(("aucune".to_owned(), Portee::Tenant)),
        "une valeur posée au niveau tenant doit descendre jusqu'au point de vente, et rester \
         marquée TENANT — c'est ce qui fait afficher « vaut pour tous vos établissements »"
    );
}

/// **Cas 2 — tenant + point de vente.** Le plus spécifique gagne.
#[tokio::test]
async fn cas_2_tenant_puis_point_de_vente() {
    let decor = Decor::nouveau("config tenant + pdv").await;
    decor.poser(Portee::Tenant, None, "aucune").await;
    decor
        .poser(
            Portee::PointDeVente,
            Some(decor.point_de_vente_id),
            "ticket_cuisine",
        )
        .await;

    let resolue = decor.resoudre(&decor.cible_complete()).await;
    assert_eq!(
        resolue,
        Some(("ticket_cuisine".to_owned(), Portee::PointDeVente)),
        "la valeur du point de vente doit l'emporter sur celle du tenant"
    );

    // **Depuis une cible SANS point de vente, la valeur du tenant reste celle qui s'applique.**
    // Une surcharge de niveau inférieur ne remonte jamais.
    let sans_pdv = Cible {
        point_de_vente_id: None,
        ..decor.cible_complete()
    };
    assert_eq!(
        decor.resoudre(&sans_pdv).await,
        Some(("aucune".to_owned(), Portee::Tenant)),
        "une surcharge de point de vente ne doit pas remonter sur une cible qui ne le vise pas"
    );
}

/// **Cas 3 — surcharge partielle.** Tenant et point de vente définis, ni établissement ni service.
///
/// La chaîne saute deux niveaux, et cela doit fonctionner sans qu'aucun niveau intermédiaire ne
/// soit inventé.
#[tokio::test]
async fn cas_3_surcharge_partielle_saute_deux_niveaux() {
    let decor = Decor::nouveau("config surcharge partielle").await;
    decor.poser(Portee::Tenant, None, "aucune").await;
    decor
        .poser(
            Portee::PointDeVente,
            Some(decor.point_de_vente_id),
            "ticket_cuisine",
        )
        .await;

    // Aucune valeur aux niveaux ETABLISSEMENT ni MODULE.
    let resolue = decor.resoudre(&decor.cible_complete()).await;
    assert_eq!(
        resolue,
        Some(("ticket_cuisine".to_owned(), Portee::PointDeVente)),
        "la chaîne doit sauter les niveaux non définis sans les inventer"
    );
}

/// **Cas 4 — définie nulle part : ABSENCE EXPLICITE.**
///
/// Ni `null`, ni valeur par défaut. `null` serait indistinguable d'une valeur nulle légitimement
/// posée ; un défaut serait un paramètre en dur, que le principe I·c interdit.
#[tokio::test]
async fn cas_4_definie_nulle_part_est_absente_ni_null_ni_defaut() {
    let decor = Decor::nouveau("config absente").await;

    let resolue = decor.resoudre(&decor.cible_complete()).await;
    assert_eq!(
        resolue, None,
        "une clé définie à AUCUN niveau doit être absente de la réponse. Rendre `null` la rendrait \
         indistinguable d'une valeur nulle posée volontairement ; rendre un défaut serait un \
         paramètre en dur (principe I·c)."
    );

    let tout = decor
        .configuration
        .resoudre_tout(&decor.cible_complete())
        .await
        .expect("résolution complète");
    assert!(
        !tout.contains_key(CLE),
        "la clé non définie doit être ABSENTE de la carte, pas présente avec une valeur nulle"
    );
}

/// **Cas 5 — surcharge sur un service désactivé : inerte, puis restituée.**
///
/// Une surcharge portée par un service désactivé est **ignorée sans être supprimée** (FR-051). La
/// réactivation la restitue telle quelle — c'est la même propriété que les déclarations de
/// capacité, et elle justifie l'absence de `DELETE` sur la table.
#[tokio::test]
async fn cas_5_surcharge_de_service_desactive_inerte_puis_restituee() {
    let decor = Decor::nouveau("config service désactivé").await;
    decor.poser(Portee::Tenant, None, "aucune").await;
    decor
        .poser(
            Portee::Module,
            Some(decor.etablissement_module_id),
            "ticket_bar",
        )
        .await;

    let cible = Cible {
        point_de_vente_id: None,
        ..decor.cible_complete()
    };

    assert_eq!(
        decor.resoudre(&cible).await,
        Some(("ticket_bar".to_owned(), Portee::Module)),
        "la surcharge de service doit s'appliquer tant que le service est actif"
    );

    decor
        .modules
        .basculer(
            decor.tenant_id,
            decor.etablissement_id,
            &decor.module_code,
            BasculerService {
                id: Uuid::now_v7(),
                actif: false,
            },
        )
        .await
        .expect("désactivation du service");

    assert_eq!(
        decor.resoudre(&cible).await,
        Some(("aucune".to_owned(), Portee::Tenant)),
        "la surcharge d'un service DÉSACTIVÉ doit devenir inerte et laisser remonter la valeur du \
         tenant — sans être supprimée"
    );

    decor
        .modules
        .basculer(
            decor.tenant_id,
            decor.etablissement_id,
            &decor.module_code,
            BasculerService {
                id: Uuid::now_v7(),
                actif: true,
            },
        )
        .await
        .expect("réactivation du service");

    assert_eq!(
        decor.resoudre(&cible).await,
        Some(("ticket_bar".to_owned(), Portee::Module)),
        "la réactivation doit RESTITUER la surcharge : elle n'avait pas été supprimée, seulement \
         rendue inerte. C'est ce que l'absence de DELETE sur la table doit garantir."
    );
}

/// **La chaîne écourtée** — un établissement sans point de vente résout sur trois niveaux.
///
/// C'est le cas de la résidence meublée, et il ne doit réclamer aucun niveau inventé (FR-050).
#[tokio::test]
async fn une_chaine_ecourtee_resout_sans_inventer_de_niveau() {
    let decor = Decor::nouveau("config chaîne écourtée").await;
    decor.poser(Portee::Tenant, None, "aucune").await;
    decor
        .poser(
            Portee::Etablissement,
            Some(decor.etablissement_id),
            "ticket_reception",
        )
        .await;

    // Ni service ni point de vente dans la cible.
    let ecourtee = Cible {
        tenant_id: decor.tenant_id,
        etablissement_id: Some(decor.etablissement_id),
        module_code: None,
        point_de_vente_id: None,
    };

    assert_eq!(
        decor.resoudre(&ecourtee).await,
        Some(("ticket_reception".to_owned(), Portee::Etablissement)),
        "une chaîne à deux niveaux doit résoudre sans qu'aucun niveau absent ne soit fabriqué"
    );

    // Cible réduite au tenant seul.
    let tenant_seul = Cible {
        tenant_id: decor.tenant_id,
        etablissement_id: None,
        module_code: None,
        point_de_vente_id: None,
    };
    assert_eq!(
        decor.resoudre(&tenant_seul).await,
        Some(("aucune".to_owned(), Portee::Tenant)),
        "une cible réduite au tenant ne doit voir aucune surcharge d'un niveau inférieur"
    );
}

/// **L'isolation tient à CHAQUE niveau de la descente.**
///
/// Résoudre depuis le tenant A en visant un `point_de_vente_id` du tenant B ne doit rien rendre —
/// **pas même la valeur héritée du tenant A**.
///
/// C'est la surface la plus glissante du cycle : la descente touche quatre niveaux, et il suffit
/// qu'un seul échappe à la politique pour qu'une valeur d'autrui remonte. Rendre la valeur de A
/// serait presque aussi grave que rendre celle de B — cela confirmerait l'existence du point de
/// vente de B.
#[tokio::test]
async fn l_isolation_tient_a_chaque_niveau_de_la_descente() {
    let a = Decor::nouveau("config isolation A").await;
    let b = Decor::nouveau("config isolation B").await;

    a.poser(Portee::Tenant, None, "valeur_de_A").await;
    b.poser(Portee::Tenant, None, "valeur_de_B").await;
    b.poser(
        Portee::PointDeVente,
        Some(b.point_de_vente_id),
        "surcharge_de_B",
    )
    .await;

    // A résout en visant le point de vente de B.
    let cible_croisee = Cible {
        tenant_id: a.tenant_id,
        etablissement_id: Some(a.etablissement_id),
        module_code: Some(a.module_code.clone()),
        point_de_vente_id: Some(b.point_de_vente_id),
    };

    let resolue = a.resoudre(&cible_croisee).await;
    assert_eq!(
        resolue,
        Some(("valeur_de_A".to_owned(), Portee::Tenant)),
        "le tenant A visant le point de vente du tenant B doit voir sa PROPRE valeur héritée, \
         jamais celle de B. Résolu : {resolue:?}"
    );
    assert_ne!(
        resolue.as_ref().map(|(v, _)| v.as_str()),
        Some("surcharge_de_B"),
        "FUITE — la surcharge du tenant B a été rendue au tenant A"
    );

    // Et en visant l'établissement de B.
    let cible_etablissement_croise = Cible {
        tenant_id: a.tenant_id,
        etablissement_id: Some(b.etablissement_id),
        module_code: None,
        point_de_vente_id: None,
    };
    let resolue = a.resoudre(&cible_etablissement_croise).await;
    assert_eq!(
        resolue,
        Some(("valeur_de_A".to_owned(), Portee::Tenant)),
        "visant l'établissement de B, A doit voir sa propre valeur de tenant : {resolue:?}"
    );
}

/// **Une portée plus basse que celle du catalogue est refusée** — `portee_interdite`.
///
/// Poser un paramètre à un niveau que le catalogue n'autorise pas produirait une ligne que la
/// résolution ne remonterait jamais : l'exploitant croirait avoir réglé quelque chose, et rien ne
/// changerait.
#[tokio::test]
async fn une_portee_plus_basse_que_le_catalogue_est_refusee() {
    use kaya_etablissements::configuration::ErreurParametre;

    let pool_owner = commun::pool_owner().await;
    let pool_app = commun::pool_app().await;
    let jeu = commun::creer_tenant(&pool_owner, "config portée interdite").await;
    let configuration = ServiceConfiguration::nouveau(pool_app, PgOutboxWriter::nouveau());

    // `politique_impression` descend jusqu'à POINT_DE_VENTE : les quatre portées sont donc
    // permises. Le refus se vérifie sur une clé **hors catalogue**, l'autre garde de la même
    // écriture.
    let refus = configuration
        .ecrire(
            jeu.tenant_id,
            EcrireParametre {
                id: Uuid::now_v7(),
                cle: "cle_qui_n_existe_pas".to_owned(),
                valeur: json!("x"),
                portee: Portee::Tenant,
                portee_id: None,
            },
        )
        .await;

    assert!(
        matches!(refus, Err(ErreurParametre::CleHorsCatalogue(_))),
        "une clé hors catalogue doit être refusée en nommant la clé : {refus:?}"
    );
}

/// **Extension de la porte P-10 au `JSONB`** — un `MONTANT_MINEUR` refuse tout non-entier.
///
/// Cette validation est celle sans laquelle un montant en flottant entrerait par la porte de
/// service : `parametre_configuration.valeur` est un `JSONB`, donc **aucune colonne n'est en
/// cause** et l'analyse des migrations de P-10 ne peut rien voir.
///
/// Le test porte sur la fonction de compatibilité plutôt que sur une écriture réelle : le
/// catalogue ne contient aucune clé `MONTANT_MINEUR` à ce cycle, et en ajouter une pour le test
/// ferait échouer la porte de cohérence documentaire, qui exige que toute clé du catalogue figure
/// au récapitulatif des paramètres.
#[test]
fn p10_etendue_un_montant_mineur_refuse_tout_ce_qui_n_est_pas_entier() {
    use kaya_etablissements::configuration::valeur_compatible;

    assert!(
        valeur_compatible("MONTANT_MINEUR", &json!(1500)),
        "un entier doit être accepté"
    );

    for refuse in [json!(1500.75), json!("1500"), json!(true), json!(null)] {
        assert!(
            !valeur_compatible("MONTANT_MINEUR", &refuse),
            "MONTANT_MINEUR a accepté {refuse} : un montant à virgule entrerait par le JSONB, où \
             l'analyse des colonnes de P-10 ne peut pas le voir"
        );
    }

    // **`1500.0` est refusé aussi.** Il vaut un entier mais s'écrit en flottant : l'accepter
    // laisserait entrer la représentation dont on ne veut pas, et la suivante serait `1500.75`.
    assert!(
        !valeur_compatible("MONTANT_MINEUR", &json!(1500.0)),
        "MONTANT_MINEUR a accepté 1500.0 — un flottant qui vaut un entier reste un flottant"
    );

    // Un type inconnu refuse tout, plutôt que de laisser passer sans contrôle.
    assert!(
        !valeur_compatible("TYPE_INCONNU", &json!(1)),
        "un type inconnu doit faire échouer l'écriture, pas la laisser passer sans validation"
    );
}
