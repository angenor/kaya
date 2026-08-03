//! **CPT-00 — les trois tables que rien ne confond jamais.**
//!
//! # La seule story du cycle dont l'échec ne se voit sur aucun écran
//!
//! Une identité civile (`personne`), ce avec quoi on se connecte (`compte`) et un contrat de
//! travail (`employe`) sont trois choses distinctes. Les fusionner « puisque c'est la même
//! personne » ne casse rien le jour où on le fait : l'application marche, les écrans s'affichent,
//! les tests métier passent. Le défaut se paie plus tard, et il ne se rattrape pas —
//!
//!   * une femme de ménage sans compte devient un compte inutile qu'il faut créer pour la payer ;
//!   * un comptable externe sans contrat devient un employé fictif dans les états du personnel ;
//!   * et le jour où la paie arrive, **le salaire vit sur la table qui sert à décider des droits**.
//!
//! FR-004 et FR-005 en font donc des contrôles outillés plutôt qu'une consigne de revue.
//!
//! # Périmètre inspecté — et ce qui ne l'est PAS
//!
//! *§ « Couverture des portes » de la constitution : un test négatif prouve qu'un contrôle sait
//! échouer, il ne prouve pas qu'il regarde tout. Ce que ce fichier regarde est donc écrit ici.*
//!
//! | Contrôle | Périmètre | Angle mort assumé |
//! |---|---|---|
//! | Les trois figures | `comptes.personne`, `comptes.compte`, `comptes.employe` sur base réelle | — |
//! | Colonnes de contrat | **les 2 tables** `personne` et `compte`, via `information_schema.columns` | une colonne de contrat nommée sans aucun des huit motifs (`salaire`, `embauche`, `cnps`, `contrat`, `remuneration`, `paie`, `licenciement`, `anciennete`) passerait |
//! | Graphe d'appels | **3 arbres** — les sources de `socle/comptes` et de `socle/etablissements`, plus `backend/api/src/`. Les deux chemins de crate sont **composés depuis les `[workspace] members`** (`commun::perimetre`), jamais écrits ici : un crate renommé fait paniquer la composition au lieu de désigner un répertoire absent, que le balayage traiterait comme vide — donc conforme | un accès construit dynamiquement (chaîne concaténée) passerait ; c'est pourquoi l'absence de privilège de `0018` double ce contrôle |
//!
//! Le troisième contrôle a une garantie de second rang qui vaut mieux que lui : `kaya_app` n'a
//! **aucun privilège** sur `comptes.employe`, pas même `SELECT` (migration `0018`). Un chemin de
//! code écrit par distraction échoue au premier appel. Le contrôle statique existe pour que
//! l'échec arrive **avant** l'exécution, pas à la place.

mod commun;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use sqlx::Row;
use uuid::Uuid;

/// Les tables dont on vérifie qu'elles ne portent **aucune** colonne de contrat de travail.
///
/// Déclarée en constante, et comparée à un décompte : une porte dont la cible est vide passe
/// toujours au vert. Si cette liste tombait à zéro entrée par un remaniement, le test le dirait.
const TABLES_SANS_CONTRAT: [&str; 2] = ["personne", "compte"];

/// Les motifs qui trahissent une colonne de contrat de travail.
///
/// Ce sont des **fragments**, pas des noms complets : `salaire_mineur`, `salaire_brut` et
/// `salaire` tombent tous sur `salaire`. Nommer les colonnes exactes laisserait passer la
/// première variante qu'un cycle ultérieur inventerait.
const MOTIFS_DE_CONTRAT: [&str; 8] = [
    "salaire",
    "embauche",
    "cnps",
    "contrat",
    "remuneration",
    "paie",
    "licenciement",
    "anciennete",
];

/// Les arbres de code inspectés par le contrôle de graphe d'appels.
///
/// Les deux chemins de crate sont **composés** depuis les `[workspace] members` : un crate
/// renommé fait paniquer `fichier_du_crate` au lieu de désigner un répertoire absent, que le
/// balayage traiterait comme vide — donc conforme.
fn arbres_inspectes() -> Vec<String> {
    use commun::perimetre::{self, Famille};
    vec![
        perimetre::fichier_du_crate(Famille::Socle, "comptes", "src"),
        "api/src".to_owned(),
        perimetre::fichier_du_crate(Famille::Socle, "etablissements", "src"),
    ]
}

// =================================================================================================
//  1 · Les trois figures de CPT-00, sur une base réelle
// =================================================================================================

/// **Figure 1 — la femme de ménage : une personne, un contrat, aucun compte.**
///
/// C'est la figure qui casse si l'on décide qu'« un employé est un compte ». Elle n'a rien à
/// faire dans l'application et doit pourtant exister au système, sans quoi il faudrait lui créer
/// un compte qu'elle n'utilisera jamais — donc un identifiant de connexion de plus, donc une
/// surface d'attaque de plus, pour quelqu'un qui ne se connecte pas.
#[tokio::test]
async fn une_personne_peut_avoir_un_contrat_sans_aucun_compte() {
    let pool = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool, "CPT-00 — employé sans compte").await;

    let personne_id = Uuid::now_v7();
    let employe_id = Uuid::now_v7();

    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, jeu.tenant_id)
        .await
        .expect("pose du tenant");

    sqlx::query!(
        "INSERT INTO comptes.personne (id, tenant_id, nom, prenoms) VALUES ($1, $2, $3, $4)",
        personne_id,
        jeu.tenant_id,
        "Kouassi",
        Some("Affoué"),
    )
    .execute(&mut *tx)
    .await
    .expect("la personne s'insère");

    // `employe` n'a aucun privilège pour `kaya_app` — cette insertion passe donc par le rôle
    // propriétaire, comme le fera la migration du cycle qui l'implémentera vraiment.
    sqlx::query!(
        r#"
        INSERT INTO comptes.employe
            (id, tenant_id, personne_id, etablissement_id, date_embauche, salaire_mineur, devise_code)
        VALUES ($1, $2, $3, $4, DATE '2026-03-01', 90000, 'XOF')
        "#,
        employe_id,
        jeu.tenant_id,
        personne_id,
        jeu.etablissement_id,
    )
    .execute(&mut *tx)
    .await
    .expect("l'employé s'insère");

    let comptes: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "c!" FROM comptes.compte WHERE personne_id = $1"#,
        personne_id
    )
    .fetch_one(&mut *tx)
    .await
    .expect("décompte des comptes");

    assert_eq!(
        comptes, 0,
        "la figure 1 exige zéro compte : une femme de ménage a une fiche et un contrat, elle ne \
         se connecte à rien"
    );

    tx.rollback().await.expect("rollback");
}

/// **Figure 2 — le comptable externe : une personne, un compte, aucun contrat.**
///
/// La figure symétrique, et celle qu'on oublie. Un cabinet comptable externe consulte le registre
/// des actions et les états ; il n'est employé de personne. Le faire figurer parmi les employés
/// fausserait les états du personnel — et le jour de la paie, la faute se verrait au virement.
#[tokio::test]
async fn une_personne_peut_avoir_un_compte_sans_aucun_contrat() {
    let pool = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool, "CPT-00 — compte sans contrat").await;

    let personne_id = Uuid::now_v7();
    let compte_id = Uuid::now_v7();

    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, jeu.tenant_id)
        .await
        .expect("pose du tenant");

    sqlx::query!(
        "INSERT INTO comptes.personne (id, tenant_id, nom) VALUES ($1, $2, $3)",
        personne_id,
        jeu.tenant_id,
        "Diarra",
    )
    .execute(&mut *tx)
    .await
    .expect("la personne s'insère");

    sqlx::query!(
        r#"
        INSERT INTO comptes.compte
            (id, tenant_id, personne_id, identifiant_email, condensat_mot_de_passe)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        compte_id,
        jeu.tenant_id,
        personne_id,
        "cabinet@example.test",
        // Condensat de forme PHC, sans valeur : ce test ne vérifie rien de l'authentification.
        "$argon2id$v=19$m=19456,t=2,p=1$c2VsLXNhbnMtdmFsZXVy$Y29uZGVuc2F0LXNhbnMtdmFsZXVy",
    )
    .execute(&mut *tx)
    .await
    .expect("le compte s'insère");

    let employes: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "c!" FROM comptes.employe WHERE personne_id = $1"#,
        personne_id
    )
    .fetch_one(&mut *tx)
    .await
    .expect("décompte des employés");

    assert_eq!(
        employes, 0,
        "la figure 2 exige zéro contrat : un comptable externe se connecte et n'est employé de \
         personne"
    );

    tx.rollback().await.expect("rollback");
}

/// **Figure 3 — Adjoua : les deux, et deux lignes distinctes.**
///
/// Le point n'est pas qu'elle ait les deux — c'est que ce soient **deux lignes**, sur deux tables,
/// reliées par `personne_id`. Une table unique porterait la même information et rendrait les deux
/// figures précédentes inexprimables.
#[tokio::test]
async fn une_personne_peut_avoir_les_deux_et_ce_sont_deux_lignes_distinctes() {
    let pool = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool, "CPT-00 — les deux").await;

    let personne_id = Uuid::now_v7();

    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, jeu.tenant_id)
        .await
        .expect("pose du tenant");

    sqlx::query!(
        "INSERT INTO comptes.personne (id, tenant_id, nom, prenoms) VALUES ($1, $2, $3, $4)",
        personne_id,
        jeu.tenant_id,
        "N'Guessan",
        Some("Adjoua"),
    )
    .execute(&mut *tx)
    .await
    .expect("la personne s'insère");

    sqlx::query!(
        r#"
        INSERT INTO comptes.compte
            (id, tenant_id, personne_id, identifiant_telephone, condensat_mot_de_passe)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        Uuid::now_v7(),
        jeu.tenant_id,
        personne_id,
        "+2250700000001",
        "$argon2id$v=19$m=19456,t=2,p=1$c2VsLXNhbnMtdmFsZXVy$Y29uZGVuc2F0LXNhbnMtdmFsZXVy",
    )
    .execute(&mut *tx)
    .await
    .expect("le compte s'insère");

    sqlx::query!(
        r#"
        INSERT INTO comptes.employe
            (id, tenant_id, personne_id, etablissement_id, date_embauche, salaire_mineur, devise_code)
        VALUES ($1, $2, $3, $4, DATE '2024-11-15', 250000, 'XOF')
        "#,
        Uuid::now_v7(),
        jeu.tenant_id,
        personne_id,
        jeu.etablissement_id,
    )
    .execute(&mut *tx)
    .await
    .expect("l'employé s'insère");

    let (comptes, employes) = (
        sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "c!" FROM comptes.compte WHERE personne_id = $1"#,
            personne_id
        )
        .fetch_one(&mut *tx)
        .await
        .expect("décompte des comptes"),
        sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "c!" FROM comptes.employe WHERE personne_id = $1"#,
            personne_id
        )
        .fetch_one(&mut *tx)
        .await
        .expect("décompte des employés"),
    );

    assert_eq!((comptes, employes), (1, 1), "la figure 3 exige un de chaque");

    tx.rollback().await.expect("rollback");
}

// =================================================================================================
//  2 · Le contrôle statique des colonnes — FR-004
// =================================================================================================

/// **Aucune colonne de contrat sur `personne` ni sur `compte`.**
///
/// C'est le contrôle qui rend FR-004 opposable. Ajouter `salaire_mineur` à `compte` « pour éviter
/// une jointure » est le geste exact que ce test refuse — et c'est un geste raisonnable en
/// apparence, ce qui est précisément pourquoi il faut une machine pour le refuser.
///
/// Le sens de la comparaison est **colonne réelle → motif interdit**, pas l'inverse : c'est la
/// base qui fait foi, pas une liste tenue à la main.
#[tokio::test]
async fn aucune_colonne_de_contrat_n_apparait_sur_personne_ni_sur_compte() {
    let pool = commun::pool_owner().await;

    let mut inspectees = 0_usize;
    let mut fautives: Vec<String> = Vec::new();

    for table in TABLES_SANS_CONTRAT {
        let colonnes = sqlx::query(
            r#"
            SELECT column_name
            FROM information_schema.columns
            WHERE table_schema = 'comptes' AND table_name = $1
            ORDER BY ordinal_position
            "#,
        )
        .bind(table)
        .fetch_all(&pool)
        .await
        .expect("lecture du catalogue de colonnes");

        assert!(
            !colonnes.is_empty(),
            "comptes.{table} n'a aucune colonne : ou la table n'existe pas, et le contrôle \
             n'inspecte rien. Une porte dont la cible est vide passe toujours au vert."
        );
        inspectees += 1;

        for ligne in colonnes {
            let nom: String = ligne.get("column_name");
            let nom_bas = nom.to_lowercase();
            for motif in MOTIFS_DE_CONTRAT {
                if nom_bas.contains(motif) {
                    fautives.push(format!("comptes.{table}.{nom} (motif « {motif} »)"));
                }
            }
        }
    }

    assert_eq!(
        inspectees,
        TABLES_SANS_CONTRAT.len(),
        "{inspectees} table(s) inspectée(s) sur {} déclarée(s)",
        TABLES_SANS_CONTRAT.len()
    );

    assert!(
        fautives.is_empty(),
        "des colonnes de contrat de travail sont apparues sur les tables d'identité : {fautives:?}\n\
         \n\
         FR-004 : `personne` porte l'identité civile, `compte` porte l'authentification, et le \
         contrat de travail vit sur `comptes.employe` — nulle part ailleurs. Une colonne de \
         salaire sur `compte` mettrait la rémunération sur la table qui sert à décider des droits."
    );
}

/// La table qui **doit** porter ces colonnes les porte bien.
///
/// Le versant positif du contrôle précédent. Sans lui, supprimer `comptes.employe` ferait passer
/// le test négatif au vert : plus aucune colonne de contrat nulle part, donc aucune fautive.
#[tokio::test]
async fn les_colonnes_de_contrat_existent_bien_sur_employe() {
    let pool = commun::pool_owner().await;

    let colonnes: BTreeSet<String> = sqlx::query(
        r#"
        SELECT column_name
        FROM information_schema.columns
        WHERE table_schema = 'comptes' AND table_name = 'employe'
        "#,
    )
    .fetch_all(&pool)
    .await
    .expect("lecture du catalogue de colonnes")
    .iter()
    .map(|l| l.get::<String, _>("column_name"))
    .collect();

    for attendue in ["date_embauche", "numero_cnps", "salaire_mineur", "devise_code"] {
        assert!(
            colonnes.contains(attendue),
            "comptes.employe.{attendue} est absente. Le contrôle négatif de FR-004 deviendrait \
             vert à vide : plus aucune colonne de contrat nulle part, donc plus rien à trouver."
        );
    }
}

// =================================================================================================
//  3 · Le contrôle du graphe d'appels — FR-005
// =================================================================================================

/// **Aucun chemin de code ne lit `employe` pour décider d'un droit.**
///
/// FR-005 : les droits viennent de `compte_role`, jamais du contrat de travail. Lire `employe`
/// pour savoir si quelqu'un « travaille encore ici » recréerait une seconde source de vérité des
/// droits — et une source qui n'a ni audit, ni événement, ni privilège d'écriture.
///
/// **Le contrôle est textuel, et c'est assumé.** Il ne voit pas un accès construit dynamiquement.
/// Sa garantie de second rang est ailleurs et vaut mieux que lui : `kaya_app` n'a aucun privilège
/// sur cette table, pas même `SELECT` (migration `0018`, vérifié par
/// `provisions_sans_logique.rs`). Ce contrôle-ci sert à faire échouer **la compilation d'une
/// intention**, pas à remplacer le refus de la base.
#[test]
fn aucun_chemin_de_code_ne_lit_employe() {
    let racine = racine_backend();

    let mut fichiers_inspectes = 0_usize;
    let mut fautifs: Vec<String> = Vec::new();

    for arbre in arbres_inspectes() {
        let chemin = racine.join(arbre);
        assert!(
            chemin.is_dir(),
            "l'arbre inspecté {} n'existe pas — le contrôle n'inspecterait rien",
            chemin.display()
        );

        for fichier in fichiers_rust(&chemin) {
            let contenu = std::fs::read_to_string(&fichier).expect("lecture du fichier source");
            fichiers_inspectes += 1;

            for (numero, ligne) in contenu.lines().enumerate() {
                // Les commentaires parlent d'`employe` en abondance — c'est même leur rôle ici.
                // Seul un accès SQL réel compte.
                let sans_commentaire = ligne.split("//").next().unwrap_or("");
                if sans_commentaire.contains("comptes.employe") {
                    fautifs.push(format!(
                        "{}:{}",
                        fichier
                            .strip_prefix(&racine)
                            .unwrap_or(&fichier)
                            .display(),
                        numero + 1
                    ));
                }
            }
        }
    }

    assert!(
        fichiers_inspectes >= 30,
        "seulement {fichiers_inspectes} fichier(s) Rust inspecté(s) sur trois arbres : le \
         contrôle regarde trop peu pour dire quoi que ce soit"
    );

    assert!(
        fautifs.is_empty(),
        "du code applicatif accède à `comptes.employe` : {fautifs:?}\n\
         \n\
         FR-005 : les droits viennent de `compte_role`, jamais du contrat de travail. Et \
         `comptes.employe` est une PROVISION — `kaya_app` n'a aucun privilège dessus, pas même \
         `SELECT`. Ce code échouerait au premier appel."
    );
}

/// Racine du workspace Rust, quel que soit le répertoire d'exécution des tests.
fn racine_backend() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Tous les `.rs` d'un arbre, récursivement.
fn fichiers_rust(racine: &Path) -> Vec<PathBuf> {
    let mut trouves = Vec::new();
    let mut a_visiter = vec![racine.to_path_buf()];

    while let Some(repertoire) = a_visiter.pop() {
        let entrees = match std::fs::read_dir(&repertoire) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entree in entrees.flatten() {
            let chemin = entree.path();
            if chemin.is_dir() {
                a_visiter.push(chemin);
            } else if chemin.extension().is_some_and(|e| e == "rs") {
                trouves.push(chemin);
            }
        }
    }

    trouves.sort();
    trouves
}
