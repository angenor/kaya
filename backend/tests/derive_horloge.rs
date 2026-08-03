//! **SYN-04 — la dérive d'horloge est constatée, jamais opposée.**
//!
//! # Ce que ce fichier garde
//!
//! Le cadrage §11.4 pose le fait : « un téléphone d'entrée de gamme dérive et le personnel change
//! l'heure ». Le principe IV en tire la règle — toute durée, toute taxe, toute clôture partent de
//! l'horodatage d'autorité serveur. Reste la question que la règle ne tranche pas : **que fait-on
//! quand on constate qu'une horloge est fausse ?**
//!
//! La réponse du produit est en trois temps, et chacun est vérifié ici :
//!
//! 1. **On accepte l'écriture** (FR-036). Refuser rendrait le produit inutilisable pour une
//!    serveuse dont le téléphone retarde — et elle ne peut rien y faire.
//! 2. **On consigne au registre des actions**, dans les **deux sens** : une horloge en avance est
//!    aussi fausse qu'une horloge en retard.
//! 3. **Une entrée par épisode, pas une par écriture.** Deux cents saisies pendant un service
//!    noieraient le registre, et un registre noyé n'est plus lu.
//!
//! # Périmètre inspecté — exigence 1 du § « Couverture des portes »
//!
//! **Inspecté** : le chemin réel d'écriture d'une note, par l'application montée — pas le service
//! isolé. La fonction pure `constater_derive` a ses propres tests unitaires dans son crate ; ce
//! fichier vérifie ce qu'aucun test unitaire ne peut voir : que le constat est **branché**, qu'il
//! écrit vraiment au registre, et qu'il ne refuse rien.
//!
//! **Non inspecté** : la formulation affichée à l'utilisateur. Le mot « dérive » ne doit jamais
//! atteindre l'écran, et c'est le lexique et P-16 qui le tiennent — `app/core/sync/horloge.ts`
//! porte le versant application.
//!
//! **Exercé sur LES DEUX tenants de démonstration** (exigence 5). C'est le défaut de séquence de
//! l'outbox qui a produit cette exigence, et il n'a été trouvé ni par relecture ni par une porte :
//! un mécanisme qui marche sur un tenant peut échouer sur le second, et un test mono-tenant ne le
//! dit pas.

mod commun;

use actix_web::http::StatusCode;
use serde_json::json;
use sqlx::Row;
use time::OffsetDateTime;
use uuid::Uuid;

const AUTORISATION: &str = "Authorization";

/// Les entrées `derive_horloge_constatee` d'un tenant, les plus récentes d'abord.
async fn entrees_de_derive(pool: &sqlx::PgPool, tenant_id: Uuid) -> Vec<serde_json::Value> {
    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, tenant_id)
        .await
        .expect("pose du tenant");

    let lignes = sqlx::query(
        r#"
        SELECT contexte
        FROM comptes.journal_audit
        WHERE type_action = 'derive_horloge_constatee'
        ORDER BY cree_le DESC, id DESC
        "#,
    )
    .fetch_all(&mut *tx)
    .await
    .expect("lecture du registre");

    tx.rollback().await.expect("rollback");
    lignes
        .into_iter()
        .map(|l| l.get::<serde_json::Value, _>("contexte"))
        .collect()
}

/// Vide le débrayage d'un compte — **sinon deux tests consécutifs se masquent l'un l'autre**.
///
/// La clé Redis dure quatre heures : sans ce nettoyage, le second test du fichier ne verrait
/// jamais son entrée, et il passerait au vert en croyant vérifier le débrayage alors qu'il
/// vérifierait un résidu du premier.
async fn oublier_episode(tenant_id: Uuid, compte_id: Uuid) {
    let client = redis::Client::open(commun::url_redis()).expect("client Redis");
    let mut connexion = client
        .get_multiplexed_async_connection()
        .await
        .expect("connexion Redis");
    let cle = kaya_synchronisation::derive::cle_debrayage(
        kaya_synchronisation::derive::OrigineDerive {
            tenant_id,
            compte_id,
            appareil_id: None,
        },
    );
    let _: Result<i64, redis::RedisError> = redis::cmd("DEL")
        .arg(cle)
        .query_async(&mut connexion)
        .await;
}

/// Crée une note avec un horodatage client **volontairement décalé**.
macro_rules! creer_note_decalee {
    ($app:expr, $connexion:expr, $decalage:expr) => {{
        let horodatage = OffsetDateTime::now_utc() + $decalage;
        actix_web::test::call_service(
            &$app,
            actix_web::test::TestRequest::post()
                .uri(&format!(
                    "/api/v1/etablissements/{}/notes",
                    $connexion.etablissement_id
                ))
                .insert_header((AUTORISATION, $connexion.bearer.clone()))
                .set_json(json!({
                    "id": Uuid::now_v7(),
                    "texte": "Saisie hors ligne, horloge décalée.",
                    "horodatage_client": horodatage
                        .format(&time::format_description::well_known::Rfc3339)
                        .expect("format RFC 3339"),
                }))
                .to_request(),
        )
        .await
    }};
}

// =================================================================================================
//  1 · Les deux sens — et le second est celui qu'un écart signé aurait laissé passer
// =================================================================================================

/// **Une horloge EN RETARD est constatée, et l'écriture est acceptée.**
#[actix_web::test]
async fn un_terminal_en_retard_est_consigne_et_son_ecriture_acceptee() {
    let pool_owner = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool_owner, "SYN-04 retard").await;
    let adjoua = commun::compte_connecte(
        &pool_owner,
        jeu,
        "Adjoua dérive retard",
        &[("proprietaire", Some(jeu.etablissement_id))],
    )
    .await;
    oublier_episode(jeu.tenant_id, adjoua.compte_id).await;

    let app = monter_application!(commun::pool_app().await);

    // Trois heures de retard — bien au-delà du seuil de 300 s.
    let reponse = creer_note_decalee!(app, adjoua, -time::Duration::hours(3));

    assert_eq!(
        reponse.status(),
        StatusCode::CREATED,
        "l'écriture a été REFUSÉE à cause de la dérive. FR-036 : la dérive est signalée, jamais \
         opposée — une serveuse dont le téléphone retarde de dix minutes doit pouvoir saisir, et \
         elle ne peut rien y faire."
    );

    let entrees = entrees_de_derive(&pool_owner, jeu.tenant_id).await;
    assert_eq!(
        entrees.len(),
        1,
        "aucune entrée `derive_horloge_constatee` au registre. Le constat n'est pas branché, et \
         l'exploitant ne pourra pas retrouver quel terminal déviait pendant le service."
    );

    // **Le type d'action est relu en base, pas supposé.** La famille est déclarée « branchée » au
    // document ; ce qui le prouve est la valeur écrite dans la colonne, pas le nom de la variante
    // qu'on a cru employer.
    let mut tx = pool_owner.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, jeu.tenant_id)
        .await
        .expect("tenant");
    let type_action: String = sqlx::query_scalar(
        "SELECT type_action FROM comptes.journal_audit ORDER BY cree_le DESC LIMIT 1",
    )
    .fetch_one(&mut *tx)
    .await
    .expect("lecture du type d'action");
    tx.rollback().await.expect("rollback");
    assert_eq!(
        type_action, "derive_horloge_constatee",
        "l'entrée n'a pas été écrite avec le code de la onzième famille"
    );

    let contexte = &entrees[0];
    assert_eq!(contexte["sens"], "retard");
    assert_eq!(contexte["seuil_secondes"], 300);
    assert!(
        contexte["ecart_secondes"].as_u64().expect("écart") > 10_000,
        "l'écart consigné ne correspond pas au décalage posé : {contexte}"
    );

    // **Aucune clé monétaire** — la porte P-10 inspecte le JSONB, et ce contexte décrit un temps.
    for cle in ["montant", "montant_mineur", "prix", "devise"] {
        assert!(
            contexte.get(cle).is_none(),
            "le contexte porte « {cle} » : un constat de dérive ne décrit aucun montant"
        );
    }
}

/// **Une horloge EN AVANCE est constatée AUSSI.**
///
/// C'est le cas du scénario de recette du quickstart — un horodatage client trois heures dans le
/// futur — et celui qu'une comparaison sur l'écart **signé** aurait silencieusement laissé passer.
/// La détection porte sur la **valeur absolue**, et le lexique donne bien deux formulations.
#[actix_web::test]
async fn un_terminal_en_avance_est_consigne_aussi() {
    let pool_owner = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool_owner, "SYN-04 avance").await;
    let adjoua = commun::compte_connecte(
        &pool_owner,
        jeu,
        "Adjoua dérive avance",
        &[("proprietaire", Some(jeu.etablissement_id))],
    )
    .await;
    oublier_episode(jeu.tenant_id, adjoua.compte_id).await;

    let app = monter_application!(commun::pool_app().await);
    let reponse = creer_note_decalee!(app, adjoua, time::Duration::hours(3));

    assert_eq!(reponse.status(), StatusCode::CREATED);

    let entrees = entrees_de_derive(&pool_owner, jeu.tenant_id).await;
    assert_eq!(entrees.len(), 1, "une horloge en AVANCE n'a rien consigné");
    assert_eq!(
        entrees[0]["sens"], "avance",
        "le sens consigné est faux : l'exploitant ne saurait pas dans quel sens régler l'appareil"
    );
}

/// **Sous le seuil, rien n'est consigné.**
///
/// Deux horloges ne sont jamais exactement d'accord. Consigner à chaque saisie rendrait le registre
/// illisible, donc inutilisé — et c'est la façon la plus sûre de neutraliser un registre.
#[actix_web::test]
async fn sous_le_seuil_le_registre_reste_vide() {
    let pool_owner = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool_owner, "SYN-04 sous le seuil").await;
    let adjoua = commun::compte_connecte(
        &pool_owner,
        jeu,
        "Adjoua sans dérive",
        &[("proprietaire", Some(jeu.etablissement_id))],
    )
    .await;
    oublier_episode(jeu.tenant_id, adjoua.compte_id).await;

    let app = monter_application!(commun::pool_app().await);
    let reponse = creer_note_decalee!(app, adjoua, -time::Duration::seconds(30));

    assert_eq!(reponse.status(), StatusCode::CREATED);
    assert!(
        entrees_de_derive(&pool_owner, jeu.tenant_id).await.is_empty(),
        "un écart de trente secondes a été consigné. Le seuil est de cinq minutes ; consigner \
         en deçà noierait le registre sous des constats sans objet."
    );
}

// =================================================================================================
//  2 · Le débrayage par épisode — dix saisies, UNE entrée
// =================================================================================================

/// **Dix écritures décalées produisent UNE seule entrée d'audit.**
///
/// Sans débrayage, un service entier sur un terminal mal réglé écrirait deux cents entrées
/// identiques. Le registre est à **rétention illimitée** : ce qu'on y noie, on ne le dénoie pas.
#[actix_web::test]
async fn dix_ecritures_decalees_ne_consignent_qu_une_entree() {
    let pool_owner = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool_owner, "SYN-04 débrayage").await;
    let adjoua = commun::compte_connecte(
        &pool_owner,
        jeu,
        "Adjoua dix saisies",
        &[("proprietaire", Some(jeu.etablissement_id))],
    )
    .await;
    oublier_episode(jeu.tenant_id, adjoua.compte_id).await;

    let app = monter_application!(commun::pool_app().await);

    for _ in 0..10 {
        let reponse = creer_note_decalee!(app, adjoua, -time::Duration::hours(2));
        assert_eq!(
            reponse.status(),
            StatusCode::CREATED,
            "une des dix écritures a été refusée : la dérive ne refuse jamais"
        );
    }

    let entrees = entrees_de_derive(&pool_owner, jeu.tenant_id).await;
    assert_eq!(
        entrees.len(),
        1,
        "{} entrée(s) pour dix saisies. Le débrayage par épisode ne fonctionne pas, et un service \
         sur un terminal mal réglé noierait le registre des actions — qui est à rétention \
         illimitée.",
        entrees.len()
    );
}

// =================================================================================================
//  3 · Exigence 5 — la famille est exercée sur LES DEUX tenants
// =================================================================================================

/// **Deux tenants, deux constats indépendants.**
///
/// # Pourquoi cette exigence existe
///
/// C'est le défaut de séquence de l'outbox qui l'a produite, et il n'avait été trouvé ni par
/// relecture ni par une porte : un mécanisme qui fonctionne sur un tenant peut échouer sur le
/// second, et un test mono-tenant ne le dit pas.
///
/// Ici, le risque est précis et réel : la clé de débrayage porte le tenant. Si elle ne le portait
/// pas — ou si le contexte de tenant fuyait entre deux écritures —, le premier terminal masquerait
/// le second, et un exploitant ne verrait jamais la dérive de son voisin de comptoir.
#[actix_web::test]
async fn la_famille_est_exercee_sur_les_deux_tenants() {
    let pool_owner = commun::pool_owner().await;

    let jeu_a = commun::creer_tenant(&pool_owner, "SYN-04 tenant A").await;
    let jeu_b = commun::creer_tenant(&pool_owner, "SYN-04 tenant B").await;

    let compte_a = commun::compte_connecte(
        &pool_owner,
        jeu_a,
        "Adjoua tenant A",
        &[("proprietaire", Some(jeu_a.etablissement_id))],
    )
    .await;
    let compte_b = commun::compte_connecte(
        &pool_owner,
        jeu_b,
        "Yao tenant B",
        &[("proprietaire", Some(jeu_b.etablissement_id))],
    )
    .await;
    oublier_episode(jeu_a.tenant_id, compte_a.compte_id).await;
    oublier_episode(jeu_b.tenant_id, compte_b.compte_id).await;

    let app = monter_application!(commun::pool_app().await);

    assert_eq!(
        creer_note_decalee!(app, compte_a, -time::Duration::hours(2)).status(),
        StatusCode::CREATED
    );
    assert_eq!(
        creer_note_decalee!(app, compte_b, time::Duration::hours(2)).status(),
        StatusCode::CREATED
    );

    let entrees_a = entrees_de_derive(&pool_owner, jeu_a.tenant_id).await;
    let entrees_b = entrees_de_derive(&pool_owner, jeu_b.tenant_id).await;

    assert_eq!(entrees_a.len(), 1, "le tenant A n'a rien consigné");
    assert_eq!(
        entrees_b.len(),
        1,
        "le tenant B n'a rien consigné : le débrayage du premier a masqué le second, et un \
         exploitant ne verrait jamais la dérive de son voisin"
    );

    // Chacun voit **son** sens, ce qui prouve que les deux constats sont distincts et non un
    // constat unique lu deux fois.
    assert_eq!(entrees_a[0]["sens"], "retard");
    assert_eq!(entrees_b[0]["sens"], "avance");
}

// =================================================================================================
//  4 · L'horodatage client est CONSERVÉ, et il ne décide de rien
// =================================================================================================

/// **`horodatage_client` est persisté tel quel ; `cree_le` reste l'instant serveur.**
///
/// C'est le fondement de la porte **P-23** : la colonne existe, elle se relit, elle s'affiche — et
/// aucune règle ne s'y appuie. Vérifier qu'elle est **conservée** est le versant positif : une
/// colonne qu'on écraserait par l'instant serveur satisferait P-23 en supprimant l'information que
/// l'ordre d'affichage local emploie.
#[actix_web::test]
async fn l_horodatage_client_est_conserve_et_cree_le_reste_celui_du_serveur() {
    let pool_owner = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool_owner, "SYN-04 conservation").await;
    let adjoua = commun::compte_connecte(
        &pool_owner,
        jeu,
        "Adjoua conservation",
        &[("proprietaire", Some(jeu.etablissement_id))],
    )
    .await;
    oublier_episode(jeu.tenant_id, adjoua.compte_id).await;

    let app = monter_application!(commun::pool_app().await);
    let note_id = Uuid::now_v7();
    let horodatage_client = OffsetDateTime::now_utc() - time::Duration::hours(3);

    let reponse = actix_web::test::call_service(
        &app,
        actix_web::test::TestRequest::post()
            .uri(&format!(
                "/api/v1/etablissements/{}/notes",
                jeu.etablissement_id
            ))
            .insert_header((AUTORISATION, adjoua.bearer.clone()))
            .set_json(json!({
                "id": note_id,
                "texte": "Écriture différée.",
                "horodatage_client": horodatage_client
                    .format(&time::format_description::well_known::Rfc3339)
                    .expect("format"),
            }))
            .to_request(),
    )
    .await;
    assert_eq!(reponse.status(), StatusCode::CREATED);

    let mut tx = pool_owner.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, jeu.tenant_id)
        .await
        .expect("tenant");
    let ligne = sqlx::query(
        "SELECT horodatage_client, cree_le FROM etablissements.note_etablissement WHERE id = $1",
    )
    .bind(note_id)
    .fetch_one(&mut *tx)
    .await
    .expect("lecture de la note");
    tx.rollback().await.expect("rollback");

    let persiste: OffsetDateTime = ligne.get("horodatage_client");
    let autorite: OffsetDateTime = ligne.get("cree_le");

    assert!(
        (persiste - horodatage_client).abs() < time::Duration::seconds(1),
        "l'horodatage client a été RÉÉCRIT. Il est indicatif, il sert l'ordre d'affichage local, \
         et l'écraser supprimerait l'information sans rien protéger."
    );
    assert!(
        (OffsetDateTime::now_utc() - autorite).abs() < time::Duration::minutes(1),
        "`cree_le` n'est pas l'instant serveur : l'horodatage d'AUTORITÉ a été pris du client, ce \
         que le principe IV interdit et que la porte P-23 refuse."
    );
    assert!(
        autorite - persiste > time::Duration::hours(2),
        "les deux horodatages sont confondus : l'un des deux n'est pas celui qu'on croit"
    );
}
