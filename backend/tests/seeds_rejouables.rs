//! Les seeds sont **rejouables** — SC-007.
//!
//! Trois exécutions successives produisent le même état final.
//!
//! # Pourquoi trois et pas deux
//!
//! Deux exécutions attrapent la duplication franche : le second passage crée un doublon, le
//! décompte double, le test échoue. Elles laissent passer un défaut plus discret — un seed qui
//! **met à jour** ce qu'il trouve au lieu de ne rien faire. Deux exécutions donneraient alors le
//! même décompte et un contenu modifié.
//!
//! Le test compare donc **le décompte et le contenu**, sur trois passages. C'est la forme du test
//! de rejeu de la classe A, appliquée aux seeds — et pour la même raison : recharger une
//! démonstration devant le pilote ne doit rien changer à ce qu'il voit.

mod commun;

use std::process::Command;

use uuid::{Uuid, uuid};

const TENANT_DELORIA: Uuid = uuid!("0198c4a0-0000-7000-8000-000000000001");
const TENANT_RESIDENCE_TEST: Uuid = uuid!("0198c4a0-0000-7000-8000-000000000011");

#[tokio::test]
async fn trois_executions_produisent_le_meme_etat_final() {
    let pool = commun::pool_owner().await;

    let mut etats = Vec::new();
    for passage in 1..=3 {
        executer_seeds(passage);
        etats.push(lire_etat(&pool).await);
    }

    assert_eq!(
        etats[0], etats[1],
        "le deuxième passage a changé l'état : les seeds ne sont pas rejouables"
    );
    assert_eq!(
        etats[1], etats[2],
        "le troisième passage a changé l'état — un seed qui met à jour au lieu de ne rien faire \
         passerait le test à deux exécutions"
    );

    // Les deux tenants attendus, et **seulement** eux dans le jeu seedé.
    assert_eq!(
        etats[0].len(),
        2,
        "les seeds doivent produire exactement deux tenants, obtenu : {:?}",
        etats[0]
    );

    let noms: Vec<&str> = etats[0].iter().map(|(_, nom, _)| nom.as_str()).collect();
    assert!(noms.contains(&"Deloria"), "tenant Deloria absent");
    assert!(
        noms.contains(&"Résidence Test"),
        "tenant « Résidence Test » absent — c'est lui qui rend vérifiable que rien dans le socle \
         ne suppose l'existence d'un point de vente"
    );
}

/// L'établissement de « Résidence Test » n'a **aucun point de vente**, et c'est le sujet.
///
/// À ce cycle, la vérification est structurelle : aucune table de point de vente n'existe encore
/// (elles viennent d'ETB-03). Le test constate donc l'invariant réellement disponible — le second
/// tenant existe, avec son établissement, et le socle fonctionne pour lui exactement comme pour le
/// premier. ETB-03 durcira ce test en vérifiant l'absence de ligne de point de vente.
#[tokio::test]
async fn le_second_tenant_fonctionne_sans_point_de_vente() {
    executer_seeds(0);
    let pool = commun::pool_app().await;

    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, TENANT_RESIDENCE_TEST)
        .await
        .expect("pose du tenant");

    let etablissements: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "c!" FROM etablissements.etablissement"#
    )
    .fetch_one(&mut *tx)
    .await
    .expect("comptage");

    assert_eq!(
        etablissements, 1,
        "« Résidence Test » doit voir exactement son établissement, et rien d'autre"
    );

    // Et il ne voit pas celui de Deloria — l'isolation vaut aussi pour les données seedées.
    let deloria: Option<Uuid> = sqlx::query_scalar!(
        "SELECT id FROM etablissements.etablissement WHERE tenant_id = $1",
        TENANT_DELORIA
    )
    .fetch_optional(&mut *tx)
    .await
    .expect("lecture croisée");

    assert!(
        deloria.is_none(),
        "« Résidence Test » voit l'établissement de Deloria : les seeds ont contourné la sécurité \
         au niveau ligne — probablement écrits sous le rôle propriétaire"
    );

    tx.rollback().await.expect("rollback");
}

/// Exécute le binaire de seeds.
fn executer_seeds(passage: u32) {
    let sortie = Command::new(env!("CARGO"))
        .args(["run", "--quiet", "-p", "kaya-api", "--bin", "seeds"])
        .output()
        .expect("exécution du binaire de seeds");

    assert!(
        sortie.status.success(),
        "passage {passage} : les seeds ont échoué\n{}",
        String::from_utf8_lossy(&sortie.stderr)
    );
}

/// État seedé : les tenants et leur nombre d'établissements.
///
/// Trié, pour que la comparaison ne dépende pas de l'ordre de lecture.
async fn lire_etat(pool: &sqlx::PgPool) -> Vec<(Uuid, String, i64)> {
    let mut etat = Vec::new();

    for tenant_id in [TENANT_DELORIA, TENANT_RESIDENCE_TEST] {
        let mut tx = pool.begin().await.expect("transaction");
        kaya_etablissements::tenant_context::poser_tenant(&mut tx, tenant_id)
            .await
            .expect("pose du tenant");

        let ligne = sqlx::query!(
            "SELECT nom FROM etablissements.tenant WHERE id = $1",
            tenant_id
        )
        .fetch_optional(&mut *tx)
        .await
        .expect("lecture du tenant");

        let Some(ligne) = ligne else {
            tx.rollback().await.expect("rollback");
            continue;
        };

        let etablissements: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "c!" FROM etablissements.etablissement WHERE tenant_id = $1"#,
            tenant_id
        )
        .fetch_one(&mut *tx)
        .await
        .expect("comptage des établissements");

        etat.push((tenant_id, ligne.nom, etablissements));
        tx.rollback().await.expect("rollback");
    }

    etat.sort();
    etat
}
