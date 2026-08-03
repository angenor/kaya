//! **Porte P-14** — les deux tests obligatoires de la classe A, sur `accompagnant`.
//!
//! # ★ P-14 gagne ici sa TROISIÈME cible, et la deuxième d'un même cycle
//!
//! Elle n'en avait qu'**une** depuis le cycle 001 : `note_etablissement`. `occupation` est en B,
//! `journal_audit` est exercé à part. Le cycle 006 en apporte deux — `preference_personne`
//! (`client_classes_offline.rs`) et `accompagnant`, ici.
//!
//! Une porte à cible unique ne prouve pas grand-chose de son outillage : elle prouve que la macro
//! marche **sur un cas**. Trois cibles, sur trois schémas et deux crates, prouvent qu'elle marche.
//!
//! # Le contrôle qui a été PERDU, et que la macro rétablit
//!
//! L'en-tête d'`outillage_classes.rs` le décrit sans détour : *« `occupation` a sa table, sa
//! classe déclarée, et son rejeu n'a jamais vérifié qu'un second envoi n'émet aucun événement
//! outbox : le contrôle existait pour `note_etablissement`, et il a été perdu à la réécriture »*.
//!
//! Ce n'est pas une faute d'inattention isolée — c'est ce qui arrive **à chaque réécriture d'un
//! test à la main**, et c'est pourquoi le cycle 005 a fait de ces tests un outillage instancié.
//!
//! # Pourquoi `accompagnant` est en classe A, et pas en B comme le séjour qui le porte
//!
//! Un accompagnant est **append-only et commutatif** : l'ajouter deux fois ne l'ajoute qu'une, et
//! l'ordre d'arrivée de trois accompagnants ne change pas la composition du groupe. Aucun effet
//! monétaire, aucune ressource unique à sérialiser — branche **A4** du registre.
//!
//! C'est ce qui le rend **écrivable hors ligne**, donc rejouable au retour du réseau, donc
//! **susceptible d'arriver après la clôture du séjour**. Ce dernier point est tout l'objet de
//! `sejour_orphelin.rs`.

mod commun;

use uuid::Uuid;

// =================================================================================================
//  L'INSTANCIATION — ce que coûte la couverture d'une entité de classe A
// =================================================================================================
//
// Une déclaration. Elle engendre le rejeu triple — **une ligne, UN événement** — et les six ordres
// du désordre, en six tests **nommés** : la permutation est dans le nom, et c'est ce qu'on lit en
// CI à vingt-trois heures.
//
// ⚠️ **`receptionniste`, et pas `proprietaire`.** Depuis la migration `0030`, le propriétaire ne
// reçoit que les **lectures** du séjour. Le symptôme de l'oubli est un `403` qui accuse le
// handler alors que la cause est le rôle choisi par le harnais.
tester_classe_a!(
    accompagnant,
    schema = "hebergement",
    table = "accompagnant",
    agregat = "hebergement.accompagnant",
    role = "receptionniste",
    // La préparation ouvre un **séjour réel** — par le repository, pas par l'endpoint : ce que la
    // macro mesure est le rejeu d'un **accompagnant**, pas l'ouverture d'un séjour, qui a ses
    // propres tests dans `sejour_arrivee.rs`. Passer par l'endpoint ferait dépendre six tests de
    // commutativité du bon fonctionnement d'une autre opération.
    preparation = |pool: &sqlx::PgPool, jeu: commun::JeuTenant| {
        let pool = pool.clone();
        std::boxed::Box::pin(async move { ouvrir_sejour_de_test(&pool, jeu).await })
            as std::pin::Pin<std::boxed::Box<dyn std::future::Future<Output = uuid::Uuid> + Send>>
    },
    chemin = |sejour_id| format!(
        "/api/v1/etablissements/{}/sejours/{sejour_id}/accompagnants",
        etablissement_du_sejour(sejour_id)
    ),
    corps = |id, rang| serde_json::json!({
        "id": id,
        // **Un nom suffit** (FR-015). Demander une pièce par accompagnant coûterait la cible des
        // 60 secondes de l'arrivée.
        "nom": format!("Accompagnant {rang}"),
    }),
);

// =================================================================================================
//  Préparation
// =================================================================================================

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// L'établissement de chaque séjour créé — le chemin en a besoin, la macro ne le passe pas.
///
/// La fermeture de chemin de `tester_classe_a!` reçoit **un** identifiant. Le chemin des
/// accompagnants en demande **deux** : l'établissement et le séjour. Cette table les relie.
///
/// Ce n'est pas élégant, et l'alternative l'était moins : ajouter un second paramètre à la macro
/// aurait rouvert **toutes** les instanciations existantes pour une seule entité — exactement le
/// coût que l'outillage du cycle 005 existe pour éviter.
fn registre() -> &'static Mutex<HashMap<Uuid, Uuid>> {
    static REGISTRE: OnceLock<Mutex<HashMap<Uuid, Uuid>>> = OnceLock::new();
    REGISTRE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn etablissement_du_sejour(sejour_id: Uuid) -> Uuid {
    *registre()
        .lock()
        .expect("registre des séjours de test")
        .get(&sejour_id)
        .expect("le séjour doit avoir été préparé avant que son chemin ne soit composé")
}

/// Ouvre un séjour minimal — établissement, module actif, catégorie, unité, formule, séjour.
async fn ouvrir_sejour_de_test(pool: &sqlx::PgPool, jeu: commun::JeuTenant) -> Uuid {
    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, jeu.tenant_id)
        .await
        .expect("pose du tenant");

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

    let sejour_id = Uuid::now_v7();
    sqlx::query(
        r#"
        INSERT INTO hebergement.sejour (id, tenant_id, etablissement_id)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(sejour_id)
    .bind(jeu.tenant_id)
    .bind(jeu.etablissement_id)
    .execute(&mut *tx)
    .await
    .expect("séjour");

    tx.commit().await.expect("commit");

    registre()
        .lock()
        .expect("registre des séjours de test")
        .insert(sejour_id, jeu.etablissement_id);

    sejour_id
}
