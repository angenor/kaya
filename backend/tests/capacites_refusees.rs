//! **Porte P-06** — toute capacité ou tout profil non implémenté est refusé **explicitement,
//! jamais ignoré**.
//!
//! La nuance est tout le sujet. Une valeur **ignorée** laisse croire que la capacité est active :
//! l'exploitant coche « suivi du stock », rien ne se passe, et le défaut se découvre au premier
//! inventaire faux — trois mois plus tard, quand il faut expliquer un écart. Une valeur
//! **refusée** produit un message immédiat qui nomme ce qui manque.
//!
//! # Ce que cette porte inspecte, et ce qu'elle n'inspecte pas
//!
//! *Exigence 1 du § « Couverture des portes » de la constitution.*
//!
//! **Inspecté** — les **neuf valeurs non implémentées** : six capacités (`LIVRAISON`,
//! `PRODUCTION`, `COMMERCE_EN_LIGNE`, `FIDELITE`, `DEVIS`, `COMPTES_CLIENTS`) et trois profils
//! (`AUCUN`, `VALORISE`, `DETAILLE`). Chacune à **deux niveaux** — le référentiel qui la déclare
//! non implémentée, et le refus d'écriture par la base sous le rôle applicatif.
//!
//! **Non inspecté** : la **troisième couche du refus**, l'absence pure à l'interface (principe VII).
//! Elle est vérifiée côté application, par le test de la fonction de sélection et le test de rendu
//! de `G1`. Le `422` de l'API est la deuxième couche ; il ne remplace ni la première ni la
//! troisième (research.md R-02).
//!
//! # Le refus est STRUCTUREL, pas applicatif
//!
//! Une validation applicative se contourne par un import, un script de reprise ou un jeu de
//! données. Le rempart est donc en base, **déclaratif et sans déclencheur** : le référentiel porte
//! `implementee`, la déclaration de consommation le **recopie**, une clé étrangère composite les
//! lie, et un `CHECK` exige que la recopie soit vraie. Déclarer `LIVRAISON` devient impossible —
//! la seule ligne de référentiel qui la porte a `implementee = false`.

mod commun;

use sqlx::Row;

/// Les six capacités déclarées au référentiel et **non implémentées au MVP**.
const CAPACITES_REFUSEES: &[&str] = &[
    "LIVRAISON",
    "PRODUCTION",
    "COMMERCE_EN_LIGNE",
    "FIDELITE",
    "DEVIS",
    "COMPTES_CLIENTS",
];

/// Les trois profils de stock non implémentés.
const PROFILS_REFUSES: &[&str] = &["AUCUN", "VALORISE", "DETAILLE"];

/// Total attendu — comparé au nombre de cas réellement exercés (§ « Couverture des portes »).
const REFUS_ATTENDUS: usize = 9;

async fn table_existe(pool: &sqlx::PgPool, schema: &str, table: &str) -> bool {
    sqlx::query(
        r#"
        SELECT EXISTS (
            SELECT 1 FROM information_schema.tables
            WHERE table_schema = $1 AND table_name = $2
        ) AS existe
        "#,
    )
    .bind(schema)
    .bind(table)
    .fetch_one(pool)
    .await
    .expect("lecture du catalogue")
    .get("existe")
}

/// **Premier niveau — le référentiel déclare la valeur CONNUE et NON IMPLÉMENTÉE.**
///
/// Les six capacités ne sont pas absentes du référentiel : elles y figurent avec
/// `implementee = false`. C'est ce qui permet au refus de distinguer « connu mais non implémenté »
/// de « inconnu » — distinction qu'un `CHECK ... IN ('STOCK')` littéral ne saurait pas faire, et
/// qui change le message rendu à l'exploitant. Elle décide aussi de l'ouverture future : lever le
/// drapeau est une **écriture de configuration**, pas une migration (cadrage §14.4).
#[tokio::test]
async fn p06_les_neuf_valeurs_sont_connues_et_declarees_non_implementees() {
    let pool = commun::pool_owner().await;
    let mut exerces = 0usize;
    let mut manquements = Vec::new();

    for code in CAPACITES_REFUSEES {
        let implementee: Option<bool> = sqlx::query_scalar(
            "SELECT implementee FROM etablissements.capacite WHERE code = $1",
        )
        .bind(code)
        .fetch_optional(&pool)
        .await
        .expect("lecture du référentiel des capacités");

        match implementee {
            None => manquements.push(format!(
                "capacité « {code} » ABSENTE du référentiel. Elle doit y figurer avec \
                 implementee = false : une valeur absente est « inconnue », une valeur présente et \
                 non implémentée est « pas encore » — deux messages différents pour l'exploitant."
            )),
            Some(true) => manquements.push(format!(
                "capacité « {code} » déclarée IMPLÉMENTÉE alors que seule STOCK l'est au MVP. \
                 Le CHECK de module_capacite l'accepterait désormais, et rien ne l'implémente."
            )),
            Some(false) => exerces += 1,
        }
    }

    for code in PROFILS_REFUSES {
        let ligne = sqlx::query(
            "SELECT implementee, motif_refus_cle FROM etablissements.profil_stock WHERE code = $1",
        )
        .bind(code)
        .fetch_optional(&pool)
        .await
        .expect("lecture du référentiel des profils");

        match ligne {
            None => manquements.push(format!("profil « {code} » ABSENT du référentiel")),
            Some(l) if l.get::<bool, _>("implementee") => manquements.push(format!(
                "profil « {code} » déclaré IMPLÉMENTÉ alors que seul SIMPLE l'est au MVP"
            )),
            Some(l) => {
                let motif: Option<String> = l.get("motif_refus_cle");
                match motif {
                    None => manquements.push(format!(
                        "profil « {code} » n'a aucun `motif_refus_cle`. Le refus doit EXPLIQUER, \
                         pas seulement constater — c'est la différence entre « refusé \
                         explicitement » et « ignoré »."
                    )),
                    Some(_) => exerces += 1,
                }
            }
        }
    }

    assert!(
        manquements.is_empty(),
        "P-06 ÉCHOUE — {} manquement(s) :\n  {}",
        manquements.len(),
        manquements.join("\n  ")
    );

    assert_eq!(
        exerces, REFUS_ATTENDUS,
        "P-06 : {exerces} valeur(s) vérifiée(s) au lieu des {REFUS_ATTENDUS} attendues. Une porte \
         qui inspecte un sous-ensemble en croyant tout couvrir est le défaut exact que la \
         constitution a documenté après le cycle 001."
    );
    println!("P-06 — niveau référentiel : {exerces}/{REFUS_ATTENDUS} valeurs refusées, nommées.");
}

/// **Le message d'`AUCUN` est DISTINCT des deux autres profils.**
///
/// `VALORISE` et `DETAILLE` sont des fonctionnalités absentes du MVP — on annonce une absence, et
/// l'exploitant attend une version future. `AUCUN` n'est pas une fonctionnalité manquante : c'est
/// une demande qui n'a pas de sens, puisqu'**une capacité qu'on ne consomme pas ne se déclare
/// simplement pas**.
///
/// Leur donner le même message enverrait attendre une version future une personne qui doit juste
/// ne rien faire. C'est le seul refus du cycle qui **enseigne** quelque chose plutôt que de
/// constater une absence, et c'est pour cela qu'il est testé à part.
#[tokio::test]
async fn p06_le_refus_d_aucun_enseigne_au_lieu_de_constater() {
    let pool = commun::pool_owner().await;

    let motifs: Vec<(String, Option<String>)> = sqlx::query(
        "SELECT code, motif_refus_cle FROM etablissements.profil_stock WHERE implementee = false",
    )
    .fetch_all(&pool)
    .await
    .expect("lecture des motifs de refus")
    .into_iter()
    .map(|l| (l.get("code"), l.get("motif_refus_cle")))
    .collect();

    let motif_aucun = motifs
        .iter()
        .find(|(code, _)| code == "AUCUN")
        .and_then(|(_, m)| m.clone())
        .expect("le profil AUCUN doit porter un motif de refus");

    for (code, motif) in &motifs {
        if code == "AUCUN" {
            continue;
        }
        assert_ne!(
            motif.as_deref(),
            Some(motif_aucun.as_str()),
            "le profil « {code} » partage le motif de refus d'AUCUN.\n\
             AUCUN dit « une capacité non consommée ne se déclare pas » ; {code} dit « pas encore \
             implémenté ». Les confondre fait attendre une version future à quelqu'un qui doit \
             juste ne rien faire."
        );
    }
}

/// Active un module sur un établissement neuf et rend l'identifiant de l'activation.
///
/// Passe par le **rôle applicatif**, pas par le propriétaire : c'est sous ce rôle que le refus
/// doit tenir, puisque c'est celui qu'un import ou un script de reprise emprunterait.
async fn activer_module(pool_app: &sqlx::PgPool, jeu: &commun::JeuTenant, code: &str) -> uuid::Uuid {
    let id = uuid::Uuid::now_v7();
    let mut tx = pool_app.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, jeu.tenant_id)
        .await
        .expect("pose du tenant");

    sqlx::query(
        r#"
        INSERT INTO etablissements.etablissement_module
            (id, tenant_id, etablissement_id, module_code, module_implemente)
        VALUES ($1, $2, $3, $4, true)
        "#,
    )
    .bind(id)
    .bind(jeu.tenant_id)
    .bind(jeu.etablissement_id)
    .bind(code)
    .execute(&mut *tx)
    .await
    .expect("activation du module");

    tx.commit().await.expect("commit");
    id
}

/// **Second niveau — le refus est tenu par la BASE, sous le rôle applicatif.**
///
/// Les neuf tentatives d'`INSERT` direct, sans passer par aucun service. C'est le chemin qu'un
/// import, un jeu de données ou un script de reprise emprunterait — et celui qu'une validation
/// applicative seule laisserait ouvert.
///
/// Chaque cas vérifie **deux choses** : que l'écriture échoue, et qu'**aucune ligne n'a été
/// écrite**. La seconde n'est pas redondante — une erreur levée après une insertion partiellement
/// validée existerait sans qu'on la voie, et c'est l'état résiduel qui compte, pas le message.
#[tokio::test]
async fn p06_second_niveau_les_neuf_refus_sont_tenus_par_la_base() {
    let pool_owner = commun::pool_owner().await;
    let pool_app = commun::pool_app().await;
    let jeu = commun::creer_tenant(&pool_owner, "P-06 second niveau").await;
    let module_id = activer_module(&pool_app, &jeu, "RESTAURATION").await;

    let mut exerces = 0usize;
    let mut manquements = Vec::new();

    // Six capacités non implémentées, chacune avec un profil VALIDE : c'est bien la capacité qui
    // est refusée, pas le profil. Sans cette précaution, un profil invalide masquerait le cas.
    for capacite in CAPACITES_REFUSEES {
        match tenter_declaration(&pool_app, &jeu, module_id, capacite, "SIMPLE").await {
            Ok(()) => manquements.push(format!(
                "capacité « {capacite} » ACCEPTÉE en base sous le rôle applicatif. Le refus ne \
                 tient plus qu'au service — donc se contourne par tout import ou script de reprise."
            )),
            Err(_) => exerces += 1,
        }
    }

    // Trois profils non implémentés, avec la capacité VALIDE `STOCK`.
    for profil in PROFILS_REFUSES {
        match tenter_declaration(&pool_app, &jeu, module_id, "STOCK", profil).await {
            Ok(()) => manquements.push(format!(
                "profil « {profil} » ACCEPTÉ en base sous le rôle applicatif"
            )),
            Err(_) => exerces += 1,
        }
    }

    // **Aucune ligne écrite** — l'état résiduel, pas seulement les messages d'erreur.
    let lignes = compter_declarations(&pool_app, &jeu, module_id).await;
    assert_eq!(
        lignes, 0,
        "P-06 : {lignes} déclaration(s) en base après {REFUS_ATTENDUS} tentatives toutes censées \
         échouer. Une erreur levée après une écriture validée laisserait un inventaire faux."
    );

    assert!(
        manquements.is_empty(),
        "P-06 (second niveau) ÉCHOUE — {} manquement(s) :\n  {}",
        manquements.len(),
        manquements.join("\n  ")
    );
    assert_eq!(
        exerces, REFUS_ATTENDUS,
        "P-06 : {exerces} refus exercé(s) au lieu des {REFUS_ATTENDUS} attendus"
    );
    println!("P-06 — niveau base : {exerces}/{REFUS_ATTENDUS} refus tenus, zéro ligne écrite.");
}

/// **Le cas nominal passe** — sans lui, la porte serait satisfaite par une table qui refuse tout.
///
/// C'est la faute symétrique de « la porte ne trouve jamais rien » : une contrainte trop large
/// rendrait les neuf refus verts en bloquant aussi la seule déclaration légitime, et personne ne
/// s'en apercevrait avant le premier établissement qui suit son stock.
///
/// Le **rejeu** est vérifié dans la foulée : la seconde tentative avec le même identifiant ne crée
/// pas de seconde ligne (idempotence des écritures de classe C, principe VI).
#[tokio::test]
async fn p06_le_cas_nominal_stock_simple_est_accepte_et_rejouable() {
    let pool_owner = commun::pool_owner().await;
    let pool_app = commun::pool_app().await;
    let jeu = commun::creer_tenant(&pool_owner, "P-06 cas nominal").await;
    let module_id = activer_module(&pool_app, &jeu, "RESTAURATION").await;

    let declaration_id = uuid::Uuid::now_v7();
    for passage in 1..=3 {
        declarer(&pool_app, &jeu, declaration_id, module_id, "STOCK", "SIMPLE")
            .await
            .unwrap_or_else(|e| {
                panic!(
                    "passage {passage} : STOCK/SIMPLE doit être ACCEPTÉ — une contrainte qui \
                     refuse tout rendrait les neuf refus verts sans rien garantir. Erreur : {e}"
                )
            });
    }

    let lignes = compter_declarations(&pool_app, &jeu, module_id).await;
    assert_eq!(
        lignes, 1,
        "trois envois du même identifiant ont produit {lignes} ligne(s) : le rejeu n'est pas \
         inoffensif, et un terminal qui vide sa file après une coupure créerait des doublons"
    );
}

/// Tente une déclaration avec un identifiant neuf. `Err` signifie « refusée par la base ».
async fn tenter_declaration(
    pool_app: &sqlx::PgPool,
    jeu: &commun::JeuTenant,
    module_id: uuid::Uuid,
    capacite: &str,
    profil: &str,
) -> Result<(), sqlx::Error> {
    declarer(
        pool_app,
        jeu,
        uuid::Uuid::now_v7(),
        module_id,
        capacite,
        profil,
    )
    .await
}

/// `INSERT` direct, **sans passer par aucun service** — le chemin qu'un import emprunterait.
///
/// `module_implemente` / `capacite_implementee` sont posés à `true` volontairement : c'est ce
/// qu'écrirait quelqu'un qui cherche à contourner. La clé étrangère composite ne trouve alors
/// aucune ligne de référentiel correspondante et refuse — sans le `CHECK`, une recopie à `false`
/// serait passée.
async fn declarer(
    pool_app: &sqlx::PgPool,
    jeu: &commun::JeuTenant,
    id: uuid::Uuid,
    module_id: uuid::Uuid,
    capacite: &str,
    profil: &str,
) -> Result<(), sqlx::Error> {
    let mut tx = pool_app.begin().await?;
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, jeu.tenant_id)
        .await
        .expect("pose du tenant");

    let resultat = sqlx::query(
        r#"
        INSERT INTO etablissements.module_capacite
            (id, tenant_id, etablissement_module_id,
             capacite_code, capacite_implementee, profil_code, profil_implemente)
        VALUES ($1, $2, $3, $4, true, $5, true)
        ON CONFLICT (id) DO NOTHING
        "#,
    )
    .bind(id)
    .bind(jeu.tenant_id)
    .bind(module_id)
    .bind(capacite)
    .bind(profil)
    .execute(&mut *tx)
    .await;

    match resultat {
        Ok(_) => {
            tx.commit().await?;
            Ok(())
        }
        Err(e) => {
            let _ = tx.rollback().await;
            Err(e)
        }
    }
}

async fn compter_declarations(
    pool_app: &sqlx::PgPool,
    jeu: &commun::JeuTenant,
    module_id: uuid::Uuid,
) -> i64 {
    let mut tx = pool_app.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, jeu.tenant_id)
        .await
        .expect("pose du tenant");

    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM etablissements.module_capacite WHERE etablissement_module_id = $1",
    )
    .bind(module_id)
    .fetch_one(&mut *tx)
    .await
    .expect("comptage");

    tx.rollback().await.expect("rollback");
    total
}

/// **La table `module_capacite` porte bien les deux `CHECK`** qui rendent le refus déclaratif.
///
/// Les neuf refus ci-dessus passeraient aussi si la seule clé étrangère composite les tenait. Ce
/// test vérifie le second verrou : sans `CHECK (capacite_implementee)`, une ligne recopiant
/// `implementee = false` trouverait sa cible au référentiel et **serait acceptée**.
#[tokio::test]
async fn p06_les_deux_verrous_declaratifs_sont_en_place() {
    let pool = commun::pool_owner().await;

    assert!(
        table_existe(&pool, "etablissements", "module_capacite").await,
        "P-06 : `module_capacite` a disparu — la porte n'a plus de cible d'écriture"
    );

    let contraintes: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT conname
        FROM pg_constraint
        WHERE conrelid = 'etablissements.module_capacite'::regclass AND contype = 'c'
        "#,
    )
    .fetch_all(&pool)
    .await
    .expect("lecture des contraintes");

    for attendue in [
        "module_capacite_capacite_implementee",
        "module_capacite_profil_implemente",
    ] {
        assert!(
            contraintes.iter().any(|c| c == attendue),
            "P-06 : la contrainte « {attendue} » est absente.\n\
             La clé étrangère composite seule ne suffit pas : une ligne recopiant \
             `implementee = false` trouverait sa cible au référentiel et serait ACCEPTÉE. C'est le \
             CHECK qui exige que la recopie soit vraie.\n\
             Contraintes trouvées : {contraintes:?}"
        );
    }
}
