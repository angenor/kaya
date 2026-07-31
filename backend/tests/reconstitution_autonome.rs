//! **Le test central du cycle** — reconstituer une opération depuis la seule charge utile.
//!
//! # Ce qu'il prouve, et pourquoi la manière compte plus que le résultat
//!
//! TRX-02 exige que le grand livre porte une charge utile **complète et dénormalisée** : un
//! lecteur qui n'a qu'une ligne doit pouvoir dire ce qui s'est passé, pour quel montant, avec
//! quelles taxes et sur quel document.
//!
//! Un test qui « n'interroge pas les autres tables » par discipline de rédaction ne prouve rien.
//! Il suffit d'un `JOIN` ajouté six mois plus tard — pour enrichir un rapport, pour retrouver le
//! nom d'un client — et la garantie disparaît sans qu'aucune alerte ne se déclenche. Le jour où
//! l'on voudra régénérer la comptabilité de 2026, la charge utile sera incomplète et il sera trop
//! tard.
//!
//! Ce test se connecte donc avec **`kaya_ledger_reader`**, un rôle qui n'a le droit de lire
//! **que** `synchronisation.evenement_outbox` (migration 0001). Toute lecture d'une autre table
//! lève une erreur de permission PostgreSQL. Le test ne *promet* pas de rester dans le grand
//! livre : il en est *empêché*.
//!
//! C'est la différence entre une convention et une garantie — la seule qui tiendra sur dix ans de
//! cycles.

mod commun;

use serde_json::Value;
use uuid::Uuid;

/// Jeu de cas **figé**, versionné avec ce test.
const JEU_FIGE: &str = include_str!("fixtures/grand_livre_v1.json");

fn jeu() -> Value {
    serde_json::from_str(JEU_FIGE).expect("jeu de cas illisible")
}

/// Reconstitution d'un encaissement depuis la seule charge utile.
#[tokio::test]
async fn un_encaissement_se_reconstitue_depuis_la_seule_charge_utile() {
    let jeu = jeu();
    let pool_owner = commun::pool_owner().await;
    let tenant = commun::creer_tenant(&pool_owner, "reconstitution autonome").await;

    // --- Seed du jeu financier, sous le rôle propriétaire --------------------------------------
    let evenement_id = Uuid::now_v7();
    let agregat_id = Uuid::now_v7();
    semer_evenement(&pool_owner, &jeu, tenant, evenement_id, agregat_id).await;

    // --- Lecture, sous le rôle qui ne peut rien lire d'autre -----------------------------------
    let pool = commun::pool_ledger().await;
    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, tenant.tenant_id)
        .await
        .expect("pose du tenant");

    let ligne = sqlx::query!(
        r#"
        SELECT type_evenement, agregat, version_schema, payload
        FROM synchronisation.evenement_outbox
        WHERE id = $1
        "#,
        evenement_id
    )
    .fetch_one(&mut *tx)
    .await
    .expect("lecture du grand livre sous kaya_ledger_reader");

    // --- Reconstitution — aucune autre source que `payload` ------------------------------------
    let attendu = &jeu["attendu"];
    let charge = &ligne.payload;

    assert_eq!(
        ligne.version_schema,
        jeu["version_schema"].as_i64().unwrap() as i16,
        "la version du format doit être portée par la ligne : sans elle, un décodeur futur devrait \
         deviner le format au lieu d'être écrit pour lui"
    );
    assert_eq!(ligne.type_evenement, jeu["type_evenement"].as_str().unwrap());
    assert_eq!(ligne.agregat, jeu["agregat"].as_str().unwrap());

    assert_eq!(
        charge["montant_mineur"], attendu["montant_mineur"],
        "le montant doit être lisible depuis la seule charge utile"
    );
    assert_eq!(charge["devise"], attendu["devise"]);
    assert_eq!(charge["mode_reglement"], attendu["mode_reglement"]);
    assert_eq!(
        charge["contrepartie"]["libelle"], attendu["contrepartie_libelle"],
        "la contrepartie est DÉNORMALISÉE : un identifiant de client obligerait à lire une autre \
         table, ce que ce rôle ne peut pas faire"
    );
    assert_eq!(
        charge["document"]["numero"], attendu["document_numero"],
        "la référence de document doit être en clair, pas un identifiant technique"
    );

    // --- Ventilation de taxes : recalculée, pas relue -----------------------------------------
    let taxes = charge["taxes"].as_array().expect("taxes absentes");
    let total: i64 = taxes
        .iter()
        .map(|t| t["montant_mineur"].as_i64().expect("montant de taxe"))
        .sum();
    assert_eq!(
        total,
        attendu["total_taxes_mineur"].as_i64().unwrap(),
        "la ventilation de taxes doit se recalculer depuis la charge utile"
    );

    let codes: Vec<&str> = taxes
        .iter()
        .map(|t| t["code"].as_str().expect("code de taxe"))
        .collect();
    let codes_attendus: Vec<&str> = attendu["codes_taxes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c.as_str().unwrap())
        .collect();
    assert_eq!(codes, codes_attendus);

    // La taxe de nuitée est un **forfait** : ni assiette ni taux. Les deux `null` sont porteurs de
    // sens, et un décodeur qui les traiterait comme des zéros produirait une assiette fausse.
    let nuitee = taxes
        .iter()
        .find(|t| t["code"] == "TAXE_NUITEE")
        .expect("taxe de nuitée absente");
    assert!(
        nuitee["assiette_mineur"].is_null() && nuitee["taux_millieme"].is_null(),
        "la taxe de nuitée est un forfait : assiette et taux doivent rester nuls, jamais zéro"
    );

    // Tous les montants sont des **entiers**. Un flottant dans la charge utile rouvrirait le
    // risque d'arrondi que le principe V ferme sur les colonnes.
    for taxe in taxes {
        assert!(
            taxe["montant_mineur"].is_i64(),
            "montant de taxe non entier : {}",
            taxe["montant_mineur"]
        );
    }
    assert!(
        charge["montant_mineur"].is_i64(),
        "le montant de l'encaissement n'est pas un entier"
    );

    tx.rollback().await.expect("rollback");
}

/// **Test négatif** — le rôle du grand livre ne peut lire aucune autre table.
///
/// C'est ce test qui donne sa valeur au précédent. Sans lui, rien ne distinguerait un rôle
/// réellement restreint d'un rôle qu'on aurait cru restreindre.
#[tokio::test]
async fn le_lecteur_du_grand_livre_ne_peut_lire_aucune_autre_table() {
    let pool = commun::pool_ledger().await;

    for table in [
        "etablissements.tenant",
        "etablissements.etablissement",
        "etablissements.note_etablissement",
    ] {
        let mut tx = pool.begin().await.expect("transaction");

        // `AssertSqlSafe` est légitime ici : la requête est composée d'une constante du test,
        // sans aucune donnée d'utilisateur (R-03). Sur le chemin qui décide de la visibilité des
        // données, elle n'apparaît nulle part.
        let resultat = sqlx::raw_sql(sqlx::AssertSqlSafe(format!("SELECT 1 FROM {table} LIMIT 1")))
            .execute(&mut *tx)
            .await;

        assert!(
            resultat.is_err(),
            "kaya_ledger_reader a pu lire {table}. Le test de reconstitution ne prouve alors plus \
             rien : il pourrait s'appuyer sur cette table sans que personne ne s'en aperçoive."
        );

        let erreur = resultat.unwrap_err().to_string();
        assert!(
            erreur.contains("permission") || erreur.contains("droit"),
            "l'échec doit venir d'un refus de PERMISSION, pas d'autre chose \
             (table absente, syntaxe…) : {erreur}"
        );

        let _ = tx.rollback().await;
    }
}

async fn semer_evenement(
    pool: &sqlx::PgPool,
    jeu: &Value,
    tenant: commun::JeuTenant,
    evenement_id: Uuid,
    agregat_id: Uuid,
) {
    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, tenant.tenant_id)
        .await
        .expect("pose du tenant");

    let sequence: i64 = sqlx::query_scalar!(
        "SELECT synchronisation.prochaine_sequence($1)",
        tenant.etablissement_id
    )
    .fetch_one(&mut *tx)
    .await
    .expect("séquence")
    .unwrap_or_default();

    sqlx::query!(
        r#"
        INSERT INTO synchronisation.evenement_outbox (
            id, tenant_id, etablissement_id, sequence_etablissement,
            type_evenement, agregat, agregat_id, version_schema, payload, survenu_le
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, now())
        "#,
        evenement_id,
        tenant.tenant_id,
        tenant.etablissement_id,
        sequence,
        jeu["type_evenement"].as_str().unwrap(),
        jeu["agregat"].as_str().unwrap(),
        agregat_id,
        jeu["version_schema"].as_i64().unwrap() as i16,
        jeu["payload"],
    )
    .execute(&mut *tx)
    .await
    .expect("seed de l'événement financier");

    tx.commit().await.expect("commit");
}
