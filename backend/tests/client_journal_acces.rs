//! ★ **FR-012 — la consultation d'un numéro de pièce d'identité est JOURNALISÉE.**
//!
//! ═══════════════════════════════════════════════════════════════════════════════════════════════
//!  LA DOUZIÈME FAMILLE DU REGISTRE, ET LA PREMIÈRE QUI TRACE UNE LECTURE
//!
//!  Aucune des onze autres ne couvrait une **consultation** : `suppression` trace une mise hors
//!  service, `changement_role` une attribution — **toutes tracent un geste qui MODIFIE**. Une
//!  lecture de donnée sensible n'en est pas un, et la ranger sous une famille existante rendrait
//!  le registre illisible au propriétaire, qui est son public.
//! ═══════════════════════════════════════════════════════════════════════════════════════════════
//!
//! # Les trois propriétés vérifiées, et la troisième est celle qui compte
//!
//! 1. **une lecture de fiche laisse une trace** — sinon le journal d'accès n'existe pas ;
//! 2. **le numéro n'est JAMAIS lisible en clair** par une requête directe sous le rôle applicatif ;
//! 3. ★ **le contexte de la trace ne porte PAS la valeur lue**.
//!
//! La troisième est celle qu'une revue manquerait. Journaliser l'accès à un numéro de pièce en
//! recopiant le numéro dans le registre des actions — qui est **immuable et à rétention
//! illimitée** (P-05b) — créerait **exactement la fuite que ce journal existe pour surveiller**,
//! et rendrait la rétention de 90 jours de TRX-06 inapplicable sur la copie.
//!
//! # Ce que ce fichier prouve à `couverture_portes.rs`
//!
//! Que la famille déclarée « branchée » est **réellement exercée**. `audit_taxonomie.rs` vérifie
//! qu'un chemin de code existe ; celui-ci vérifie que quelque chose l'emprunte. *Le registre est
//! à rétention illimitée : ce qu'on y écrit sans l'avoir exercé, on ne le corrige pas après coup.*

mod commun;

use uuid::Uuid;

use commun::{creer_tenant, pool_app, pool_owner};
use kaya_comptes::audit::TypeActionAudit;

/// Le rôle du comptoir — il porte `sej.client.lire` et `sej.client.gerer`.
const ROLE: &str = "receptionniste";

const NUMERO_DE_PIECE: &str = "CI00135791";

/// ★ **Lire une fiche portant une pièce laisse UNE trace, et la trace ne porte pas le numéro.**
#[actix_web::test]
async fn lire_une_fiche_avec_piece_laisse_une_trace_sans_le_numero() {
    let owner = pool_owner().await;
    let jeu = creer_tenant(&owner, "SEJ — journal d'accès").await;
    let cx = commun::compte_connecte(&owner, jeu, "Yao", &[(ROLE, Some(jeu.etablissement_id))])
        .await;
    let app = monter_application!(pool_app().await);

    let client_id = Uuid::now_v7();
    let requete = actix_web::test::TestRequest::post()
        .uri("/api/v1/clients")
        .insert_header(("authorization", cx.bearer.clone()))
        .set_json(serde_json::json!({
            "id": client_id,
            "nom": "Bakayoko",
            "type_piece": "CNI",
            "numero_piece": NUMERO_DE_PIECE,
        }))
        .to_request();
    assert_eq!(actix_web::test::call_service(&app, requete).await.status(), 201);

    // La création **ne trace rien** : l'appelant connaît déjà le numéro, il vient de l'envoyer.
    // Tracer ici noierait les vraies consultations sous des entrées inutiles, et un registre
    // bruyant n'est plus lu.
    assert_eq!(
        traces(&owner, jeu.tenant_id, client_id).await.len(),
        0,
        "la CRÉATION d'une fiche ne doit laisser aucune trace de consultation : l'appelant connaît \
         déjà le numéro, il vient de l'envoyer. Un registre bruyant n'est plus lu."
    );

    // ── ★ LA LECTURE — c'est elle qui trace ──────────────────────────────────────────────────
    let requete = actix_web::test::TestRequest::get()
        .uri(&format!("/api/v1/clients/{client_id}"))
        .insert_header(("authorization", cx.bearer.clone()))
        .to_request();
    let reponse = actix_web::test::call_service(&app, requete).await;
    assert_eq!(reponse.status(), 200);
    let fiche: serde_json::Value = actix_web::test::read_body_json(reponse).await;

    // Le numéro est bien **rendu** à qui a le droit de le lire — sans quoi la fiche de police
    // serait impossible à remplir.
    assert_eq!(
        fiche["numero_piece"], NUMERO_DE_PIECE,
        "la lecture doit rendre le numéro à qui en a le droit : sans lui, la fiche de police est \
         impossible à remplir. Fiche : {fiche}"
    );

    let entrees = traces(&owner, jeu.tenant_id, client_id).await;
    assert_eq!(
        entrees.len(),
        1,
        "★ une lecture de fiche portant une pièce doit laisser UNE trace au registre des actions \
         (FR-012, principe IX). Trouvées : {entrees:?}"
    );

    let (type_action, cible_type, contexte) = &entrees[0];
    assert_eq!(type_action, TypeActionAudit::ConsultationPieceIdentite.code());
    assert_eq!(
        cible_type, "personne",
        "la cible est la personne CONSULTÉE, jamais l'auteur"
    );

    // ── ★ LE CONTEXTE NE PORTE PAS LA VALEUR LUE ─────────────────────────────────────────────
    let brut = contexte.to_string();
    assert!(
        !brut.contains(NUMERO_DE_PIECE),
        "★ le numéro consulté est RECOPIÉ dans le registre des actions.\n\n\
         Le registre est IMMUABLE et à rétention ILLIMITÉE (P-05b) : y recopier le numéro crée \
         exactement la fuite que ce journal existe pour surveiller, et rend la rétention de \
         90 jours de TRX-06 inapplicable sur la copie — la donnée serait purgée d'un côté et \
         conservée pour toujours de l'autre.\n\n\
         Contexte : {brut}"
    );
    assert!(
        brut.contains("motif"),
        "le contexte doit dire POURQUOI la lecture a eu lieu : c'est ce que le propriétaire vient \
         chercher au registre. Contexte : {brut}"
    );
}

/// **Une fiche SANS pièce ne laisse aucune trace.**
///
/// Lire une fiche sans pièce n'est pas un accès à une pièce. Tracer toutes les lectures noierait
/// les vraies consultations sous des entrées vides — le même raisonnement que la fréquence de
/// `derive_horloge_constatee`, qui s'écrit une fois par épisode et non une fois par écriture.
#[actix_web::test]
async fn lire_une_fiche_sans_piece_ne_laisse_aucune_trace() {
    let owner = pool_owner().await;
    let jeu = creer_tenant(&owner, "SEJ — sans pièce").await;
    let cx = commun::compte_connecte(&owner, jeu, "Yao", &[(ROLE, Some(jeu.etablissement_id))])
        .await;
    let app = monter_application!(pool_app().await);

    let client_id = Uuid::now_v7();
    let requete = actix_web::test::TestRequest::post()
        .uri("/api/v1/clients")
        .insert_header(("authorization", cx.bearer.clone()))
        .set_json(serde_json::json!({ "id": client_id, "nom": "Koffi" }))
        .to_request();
    assert_eq!(actix_web::test::call_service(&app, requete).await.status(), 201);

    let requete = actix_web::test::TestRequest::get()
        .uri(&format!("/api/v1/clients/{client_id}"))
        .insert_header(("authorization", cx.bearer.clone()))
        .to_request();
    assert_eq!(actix_web::test::call_service(&app, requete).await.status(), 200);

    assert!(
        traces(&owner, jeu.tenant_id, client_id).await.is_empty(),
        "lire une fiche SANS pièce n'est pas un accès à une pièce. Tracer toutes les lectures \
         noierait les vraies consultations sous des entrées vides."
    );
}

/// ★ **Le numéro n'est JAMAIS lisible en clair par une requête directe.**
///
/// C'est le versant « au repos » de FR-012 : un vidage de la base, une sauvegarde égarée, un
/// accès direct au serveur ne doivent rendre aucun numéro. Le contrôle porte sur la **colonne**,
/// pas sur l'API — qui, elle, a le droit de le rendre à qui en a la permission.
#[actix_web::test]
async fn la_colonne_ne_contient_jamais_le_numero_en_clair() {
    let owner = pool_owner().await;
    let jeu = creer_tenant(&owner, "SEJ — chiffrement au repos").await;
    let cx = commun::compte_connecte(&owner, jeu, "Yao", &[(ROLE, Some(jeu.etablissement_id))])
        .await;
    let app = monter_application!(pool_app().await);

    let client_id = Uuid::now_v7();
    let requete = actix_web::test::TestRequest::post()
        .uri("/api/v1/clients")
        .insert_header(("authorization", cx.bearer.clone()))
        .set_json(serde_json::json!({
            "id": client_id,
            "nom": "Adjoua",
            "type_piece": "CNI",
            "numero_piece": NUMERO_DE_PIECE,
        }))
        .to_request();
    assert_eq!(actix_web::test::call_service(&app, requete).await.status(), 201);

    let mut tx = owner.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, jeu.tenant_id)
        .await
        .expect("pose du tenant");
    let stocke: Option<String> = sqlx::query_scalar(
        "SELECT numero_piece FROM comptes.personne WHERE id = $1",
    )
    .bind(client_id)
    .fetch_one(&mut *tx)
    .await
    .expect("lecture directe");
    tx.rollback().await.expect("rollback");

    let stocke = stocke.expect("la colonne doit porter une valeur");

    assert_ne!(
        stocke, NUMERO_DE_PIECE,
        "★ le numéro de pièce est stocké EN CLAIR. Un vidage de la base, une sauvegarde égarée ou \
         un accès direct au serveur le rendraient lisible — ce que FR-012 et le cadrage §12.1 \
         interdisent."
    );
    assert!(
        !stocke.contains(NUMERO_DE_PIECE),
        "la valeur stockée CONTIENT le numéro en clair : {stocke}"
    );
    assert!(
        kaya_comptes::client::CoffreTenant::est_un_cryptogramme(&stocke),
        "la valeur stockée n'a pas la forme d'un cryptogramme du coffre — elle porte donc autre \
         chose que ce que le chiffrement produit. Valeur : {stocke}"
    );
}

/// Les traces de consultation portant sur une personne donnée.
async fn traces(
    pool: &sqlx::PgPool,
    tenant_id: Uuid,
    cible_id: Uuid,
) -> Vec<(String, String, serde_json::Value)> {
    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, tenant_id)
        .await
        .expect("pose du tenant");
    let lignes: Vec<(String, String, serde_json::Value)> = sqlx::query_as(
        r#"
        SELECT type_action, cible_type, contexte
        FROM comptes.journal_audit
        WHERE cible_id = $1 AND type_action = 'consultation_piece_identite'
        ORDER BY cree_le
        "#,
    )
    .bind(cible_id)
    .fetch_all(&mut *tx)
    .await
    .expect("lecture du registre");
    tx.rollback().await.expect("rollback");
    lignes
}
