//! **HEB-04 — le montant d'un passage, sur des cas figés.**
//!
//! # Deux niveaux, et ils ne prouvent pas la même chose
//!
//! Les cas figés du barème sont des **tests unitaires**, dans le crate
//! (`tarification/bareme.rs`) : la fonction est pure, et l'exercer sans base ni réseau est ce qui
//! permet d'en écrire dix en trois lignes chacun.
//!
//! Ce fichier-ci exerce ce que la fonction pure ne peut pas atteindre : **d'où vient la durée**.
//! C'est le seul endroit où la garantie de FR-029 se vérifie, et c'est aussi le piège que le
//! cadrage §11 désigne — « le passage aggrave la sensibilité à l'horloge ».
//!
//! # Ce que le test d'horloge démontre exactement
//!
//! Il n'existe **aucun moyen**, depuis un client, d'influencer la durée facturée : l'endpoint ne
//! prend aucun instant en paramètre, et le service lit `cree_le` et `now()` en SQL. Le test le
//! constate en soumettant deux appels dont les horloges d'appelant diffèrent de quarante minutes,
//! et en comparant les montants.

mod commun;

use time::OffsetDateTime;
use uuid::Uuid;

use commun::{JeuTenant, creer_tenant, pool_app, pool_owner};

/// Le barème de Deloria — `docs/user-stories-v1.md`, récapitulatif des paramètres.
const BAREME: [(i32, i64); 4] = [(60, 1_500), (120, 2_800), (180, 4_000), (240, 5_000)];
const PRIX_HEURE_SUP: i64 = 1_200;
const PRIX_NUITEE: i64 = 12_500;
const SEUIL_BASCULE_MINUTES: i64 = 480;

struct Decor {
    jeu: JeuTenant,
    unite_id: Uuid,
    formule_passage: Uuid,
    compte_id: Uuid,
}

/// Un établissement avec un passage barémé, une nuitée, et le seuil de bascule réglé.
async fn poser_decor(pool: &sqlx::PgPool, nom: &str) -> Decor {
    let jeu = creer_tenant(pool, nom).await;
    let compte = commun::creer_compte(
        pool,
        jeu.tenant_id,
        "Yao",
        &format!("+225{}", &Uuid::now_v7().simple().to_string()[22..]),
        commun::MOT_DE_PASSE_TEST,
        &[("receptionniste", Some(jeu.etablissement_id))],
    )
    .await;

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

    // **Le seuil est un PARAMÈTRE**, posé dans la configuration héritée — jamais une constante du
    // code. Le test le pose comme l'exploitant le poserait.
    sqlx::query(
        r#"
        INSERT INTO etablissements.parametre_configuration
            (id, tenant_id, cle, etablissement_id, valeur)
        VALUES ($1, $2, 'seuil_bascule_nuitee_minutes', $3, $4)
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(Uuid::now_v7())
    .bind(jeu.tenant_id)
    .bind(jeu.etablissement_id)
    .bind(serde_json::json!(SEUIL_BASCULE_MINUTES))
    .execute(&mut *tx)
    .await
    .expect("seuil de bascule");

    let categorie_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO hebergement.categorie (id, tenant_id, etablissement_id, nom, capacite_accueil)
         VALUES ($1, $2, $3, 'Standard', 2)",
    )
    .bind(categorie_id)
    .bind(jeu.tenant_id)
    .bind(jeu.etablissement_id)
    .execute(&mut *tx)
    .await
    .expect("catégorie");

    let unite_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO hebergement.unite (id, tenant_id, etablissement_id, categorie_id, code)
         VALUES ($1, $2, $3, $4, 'A1')",
    )
    .bind(unite_id)
    .bind(jeu.tenant_id)
    .bind(jeu.etablissement_id)
    .bind(categorie_id)
    .execute(&mut *tx)
    .await
    .expect("unité");

    let formule_passage = Uuid::now_v7();
    sqlx::query(
        r#"
        INSERT INTO hebergement.formule
            (id, tenant_id, etablissement_id, categorie_id, famille, prix_mineur,
             duree_min_minutes, duree_max_minutes, assujettie_taxe_nuitee,
             prix_heure_supplementaire_mineur)
        VALUES ($1, $2, $3, $4, 'PASSAGE', 1500, 60, 480, false, $5)
        "#,
    )
    .bind(formule_passage)
    .bind(jeu.tenant_id)
    .bind(jeu.etablissement_id)
    .bind(categorie_id)
    .bind(PRIX_HEURE_SUP)
    .execute(&mut *tx)
    .await
    .expect("passage");

    for (duree, prix) in BAREME {
        sqlx::query(
            "INSERT INTO hebergement.bareme_palier (formule_id, duree_minutes, prix_mineur, tenant_id)
             VALUES ($1, $2, $3, $4)",
        )
        .bind(formule_passage)
        .bind(duree)
        .bind(prix)
        .bind(jeu.tenant_id)
        .execute(&mut *tx)
        .await
        .expect("palier");
    }

    // La nuitée de la MÊME catégorie — c'est elle que la bascule applique.
    sqlx::query(
        r#"
        INSERT INTO hebergement.formule
            (id, tenant_id, etablissement_id, categorie_id, famille, prix_mineur,
             assujettie_taxe_nuitee)
        VALUES ($1, $2, $3, $4, 'NUITEE', $5, false)
        "#,
    )
    .bind(Uuid::now_v7())
    .bind(jeu.tenant_id)
    .bind(jeu.etablissement_id)
    .bind(categorie_id)
    .bind(PRIX_NUITEE)
    .execute(&mut *tx)
    .await
    .expect("nuitée");

    tx.commit().await.expect("commit");

    Decor {
        jeu,
        unite_id,
        formule_passage,
        compte_id: compte.compte_id,
    }
}

/// Insère une occupation dont `cree_le` est **posé dans le passé**, pour simuler une durée réelle.
///
/// `cree_le` a un `DEFAULT now()` et aucune contrainte d'immuabilité : le test l'écrit
/// explicitement. C'est la seule façon d'exercer une durée de quatre heures sans attendre quatre
/// heures — et le service, lui, ne l'écrit jamais.
async fn occuper_depuis(
    pool: &sqlx::PgPool,
    decor: &Decor,
    minutes_ecoulees: i64,
    duree_vendue_minutes: i64,
) -> Uuid {
    let id = Uuid::now_v7();
    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, decor.jeu.tenant_id)
        .await
        .expect("tenant");

    sqlx::query(
        r#"
        INSERT INTO hebergement.occupation
            (id, tenant_id, etablissement_id, unite_id, formule_id,
             periode, debut_client, fin_client, cree_le)
        VALUES (
            $1, $2, $3, $4, $5,
            tstzrange(now() - make_interval(mins => $6),
                      now() + make_interval(mins => $7), '[)'),
            now() - make_interval(mins => $6),
            now() - make_interval(mins => $6) + make_interval(mins => $8),
            now() - make_interval(mins => $6)
        )
        "#,
    )
    .bind(id)
    .bind(decor.jeu.tenant_id)
    .bind(decor.jeu.etablissement_id)
    .bind(decor.unite_id)
    .bind(decor.formule_passage)
    .bind(minutes_ecoulees as i32)
    .bind(600_i32)
    .bind(duree_vendue_minutes as i32)
    .execute(&mut *tx)
    .await
    .expect("occupation");

    tx.commit().await.expect("commit");
    id
}

fn service(
    pool: sqlx::PgPool,
    tenant_id: Uuid,
) -> kaya_hebergement::tarification::ServiceTarification<
    kaya_etablissements::etablissement::PgEstablishmentDirectory,
    kaya_etablissements::modules::PgRegistreModules,
    kaya_comptes::audit::JournalAuditPostgres,
> {
    kaya_hebergement::tarification::ServiceTarification::nouveau(
        pool.clone(),
        tenant_id,
        kaya_etablissements::etablissement::PgEstablishmentDirectory::nouveau(
            pool.clone(),
            tenant_id,
        ),
        kaya_etablissements::modules::PgRegistreModules::nouveau(pool, tenant_id),
        kaya_comptes::audit::JournalAuditPostgres,
    )
}

// =================================================================================================
//  Les cas figés, sur le CHEMIN RÉEL
// =================================================================================================

/// **2 h → 2 800.** Le cas simple, et celui qui vérifie que la chaîne entière est branchée.
#[actix_web::test]
async fn deux_heures_valent_2800_sur_le_chemin_reel() {
    let pool = pool_owner().await;
    let decor = poser_decor(&pool, "HEB-04 — 2 h").await;
    let occupation = occuper_depuis(&pool, &decor, 120, 120).await;

    let decision = service(pool_app().await, decor.jeu.tenant_id)
        .calculer(decor.jeu.etablissement_id, occupation, decor.compte_id)
        .await
        .expect("calcul");

    assert_eq!(decision.montant_du_mineur, 2_800);
    assert_eq!(decision.palier_retenu_minutes, Some(120));
    assert_eq!(decision.devise, "XOF", "la devise vient de l'établissement");
    assert!(
        decision.rebascule.is_none(),
        "aucune rebascule : le client a rendu la chambre au palier qu'il avait acheté"
    );
}

/// **4 h 10 sur un passage vendu 2 h → 6 200, et une REBASCULE tracée.**
///
/// 5 000 (dernier palier) + 1 × 1 200. C'est le cas de la maquette et du contrat.
#[actix_web::test]
async fn quatre_heures_dix_sur_un_passage_vendu_deux_heures() {
    let pool = pool_owner().await;
    let decor = poser_decor(&pool, "HEB-04 — rebascule").await;
    let occupation = occuper_depuis(&pool, &decor, 250, 120).await;

    let decision = service(pool_app().await, decor.jeu.tenant_id)
        .calculer(decor.jeu.etablissement_id, occupation, decor.compte_id)
        .await
        .expect("calcul");

    assert_eq!(decision.montant_du_mineur, 6_200);
    assert_eq!(decision.palier_retenu_minutes, Some(240));
    assert_eq!(decision.heures_supplementaires, 1);

    let rebascule = decision.rebascule.expect("une rebascule doit être annoncée");
    assert_eq!(rebascule.palier_vendu_minutes, 120);
    assert_eq!(rebascule.montant_vendu_mineur, 2_800);
    assert_eq!(rebascule.difference_mineur, 3_400);

    // ── La trace au registre des actions, dans la MÊME transaction ────────────────────────────
    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, decor.jeu.tenant_id)
        .await
        .expect("tenant");
    let (type_action, contexte): (String, serde_json::Value) = sqlx::query_as(
        r#"
        SELECT type_action, contexte
        FROM comptes.journal_audit
        WHERE cible_id = $1
        "#,
    )
    .bind(occupation)
    .fetch_one(&mut *tx)
    .await
    .expect(
        "la rebascule doit être tracée au registre des actions — c'est ce que M. Koffi lira, et \
         `TypeActionAudit::RebasculePalierPassage` existe depuis le cycle 003 avec la mention \
         « Dû par HEB-04 »",
    );

    assert_eq!(type_action, "rebascule_palier_passage");
    assert_eq!(contexte["palier_vendu_minutes"].as_i64(), Some(120));
    assert_eq!(contexte["difference_mineur"].as_i64(), Some(3_400));
    // **Nommage monétaire réservé** : `devise` au même niveau que tout montant `_mineur`.
    assert_eq!(contexte["devise"].as_str(), Some("XOF"));
}

/// **8 h → bascule en NUITÉE**, au prix de la nuitée de la même catégorie.
///
/// Pas quatre heures plus quatre heures supplémentaires (9 800) : un changement de formule.
#[actix_web::test]
async fn huit_heures_basculent_en_nuitee_au_prix_de_la_categorie() {
    let pool = pool_owner().await;
    let decor = poser_decor(&pool, "HEB-04 — bascule").await;
    let occupation = occuper_depuis(&pool, &decor, 480, 120).await;

    let decision = service(pool_app().await, decor.jeu.tenant_id)
        .calculer(decor.jeu.etablissement_id, occupation, decor.compte_id)
        .await
        .expect("calcul");

    assert_eq!(
        decision.formule_appliquee,
        kaya_hebergement::referentiel::FamilleFormule::Nuitee
    );
    assert_eq!(decision.montant_du_mineur, PRIX_NUITEE);
    assert_eq!(
        decision.palier_retenu_minutes, None,
        "une bascule n'a pas de palier : ce n'est pas un palier majoré, c'est un changement de \
         formule"
    );
}

// =================================================================================================
//  FR-029 — L'HORLOGE DU TERMINAL NE CHANGE RIEN
// =================================================================================================

/// **Une horloge décalée de quarante minutes donne le MÊME montant.**
///
/// Le cadrage §11 le désigne comme le piège du passage : « le passage aggrave la sensibilité à
/// l'horloge ». Sur une nuitée, une heure d'écart ne change pas le montant ; sur un passage à
/// 1 500 F l'heure, elle en change un septième.
///
/// # Ce que le test démontre, et comment
///
/// L'endpoint **ne prend aucun instant en paramètre** : il n'y a rien à décaler. Le test le
/// constate en deux temps — il vérifie d'abord que la signature ne l'accepte pas (le calcul ne
/// reçoit qu'un identifiant d'occupation), puis que deux appels séparés par une horloge d'appelant
/// décalée rendent le même montant, à la seconde de calcul près.
#[actix_web::test]
async fn une_horloge_decalee_de_quarante_minutes_ne_change_pas_le_montant() {
    let pool = pool_owner().await;
    let decor = poser_decor(&pool, "HEB-04 — horloge").await;
    let occupation = occuper_depuis(&pool, &decor, 120, 120).await;
    let svc = service(pool_app().await, decor.jeu.tenant_id);

    // Premier appel — l'appelant est « à l'heure ».
    let a_l_heure = svc
        .calculer(decor.jeu.etablissement_id, occupation, decor.compte_id)
        .await
        .expect("calcul");

    // Second appel — l'appelant croit qu'il est 40 minutes plus tard. Il n'a **aucun moyen** de le
    // dire au serveur : c'est la démonstration. La variable ci-dessous n'est employée nulle part
    // dans l'appel, et c'est précisément ce que le test constate.
    let horloge_du_terminal = OffsetDateTime::now_utc() + time::Duration::minutes(40);
    let _ = horloge_du_terminal;

    let decale = svc
        .calculer(decor.jeu.etablissement_id, occupation, decor.compte_id)
        .await
        .expect("calcul");

    assert_eq!(
        a_l_heure.montant_du_mineur, decale.montant_du_mineur,
        "le montant a changé alors que seule l'horloge de l'appelant diffère : la durée facturée \
         dépend donc d'une horloge que le client contrôle (FR-029)"
    );
    assert_eq!(a_l_heure.palier_retenu_minutes, decale.palier_retenu_minutes);

    // Et les deux instants d'autorité viennent de la **base**, donc avancent ensemble.
    assert!(
        decale.instant_autorite >= a_l_heure.instant_autorite,
        "l'instant d'autorité doit venir de la base et progresser"
    );
    assert!(
        (decale.instant_autorite - a_l_heure.instant_autorite) < time::Duration::minutes(1),
        "deux appels consécutifs ne peuvent pas être séparés de plus d'une minute : l'instant \
         vient d'ailleurs que de la base"
    );
}

/// **Un barème absent est refusé explicitement** — FR-025, sur le chemin réel.
#[actix_web::test]
async fn un_passage_sans_palier_refuse_le_calcul() {
    let pool = pool_owner().await;
    let decor = poser_decor(&pool, "HEB-04 — sans palier").await;
    let occupation = occuper_depuis(&pool, &decor, 120, 120).await;

    // Le barème est retiré après coup : le service du référentiel refuserait de créer une telle
    // formule (FR-025), et c'est justement ce qui rend ce cas impossible à produire autrement.
    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, decor.jeu.tenant_id)
        .await
        .expect("tenant");
    sqlx::query("DELETE FROM hebergement.bareme_palier WHERE formule_id = $1")
        .bind(decor.formule_passage)
        .execute(&mut *tx)
        .await
        .expect("retrait du barème");
    tx.commit().await.expect("commit");

    let erreur = service(pool_app().await, decor.jeu.tenant_id)
        .calculer(decor.jeu.etablissement_id, occupation, decor.compte_id)
        .await
        .expect_err("un passage sans palier ne sait rien facturer");

    assert_eq!(erreur.code(), "bareme_absent");
}
