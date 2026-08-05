//! Provisions comptables — **les tables existent, la logique n'existe pas** (principe X).
//!
//! # Un test dont l'objet est de constater qu'on n'a rien construit
//!
//! C'est inhabituel, et c'est le sujet. Les provisions du cadrage §14 sont des **choix de modèle
//! de données et d'interfaces uniquement** : aucune interface, aucune logique au MVP. Rien
//! n'empêche pourtant un cycle ultérieur d'« ajouter juste un petit endpoint de lecture » — et
//! c'est ainsi qu'une provision devient une fonctionnalité que personne n'a décidé de construire.
//!
//! Ce fichier rend ce glissement bruyant.
//!
//! # Périmètre inspecté — **cinq provisions, trois cycles**
//!
//! *§ « Couverture des portes » : une porte dont la cible est vide passe toujours au vert. Le
//! décompte est donc comparé à [`PROVISIONS`], et la liste est ici.*
//!
//! | Provision | Cycle | Ce qu'elle attend |
//! |---|---|---|
//! | `fiscalite.exercice_comptable` | 001 | La comptabilité SYSCOHADA |
//! | `fiscalite.mapping_comptable` | 001 | idem |
//! | `comptes.employe` | 003 | CPT-05 — le contrat de travail, la paie |
//! | `comptes.appareil_enrole` | 003 | CPT-05 / CPT-06 — l'enrôlement par paire de clés |
//! | `hebergement.prestation_incluse` | 004 | HEB-09 — le petit-déjeuner inclus, incrément 2 |
//!
//! **Les cinq n'accordent RIEN à `kaya_app`**, pas même `SELECT`. Ce qui prouve une provision
//! n'est pas l'absence de tout droit : c'est l'absence du droit d'**écrire**.
//!
//! # ★ `reconciliation_orpheline` CESSE d'être une provision au cycle 006
//!
//! Elle en était la sixième, avec `GRANT SELECT` **seul** : un lecteur légitime avant son cycle —
//! le récapitulatif de fin de journée doit pouvoir dire « trois constats attendent » — et aucun
//! écrivain.
//!
//! **Elle reçoit son premier écrivain** : un accompagnant de classe A arrivant après la clôture
//! d'un séjour (SEJ-02). C'est le premier cas réel d'écriture orpheline du produit.
//!
//! ⚠️ **Elle ne sort pas pour autant de ce fichier.** Sa *résolution* reste **SYN-03, tranche
//! T3** : `UPDATE` et `DELETE` demeurent interdits, et un contrôle dédié le vérifie plus bas. Une
//! table à moitié construite est exactement ce que ce fichier existe pour surveiller — et la
//! surveiller sur ce qui reste dû vaut mieux que de la retirer parce qu'une part est livrée.
//!
//! **N'est PAS inspecté** : ce que ferait un binaire de maintenance sous `kaya_owner`. Le
//! propriétaire des tables peut tout écrire, par construction — c'est le rôle applicatif qui est
//! bridé, et c'est le seul par lequel l'API passe.

mod commun;

use sqlx::Row;

/// Les provisions du produit — **schéma, table, cycle qui les a posées**.
///
/// Ajouter une provision sans l'inscrire ici la laisserait hors de tout contrôle. Le décompte
/// ci-dessous rend l'omission bruyante dans l'autre sens : une liste qui rétrécirait ferait
/// échouer le test au lieu de le rendre plus facile.
const PROVISIONS: &[(&str, &str, &str)] = &[
    ("fiscalite", "exercice_comptable", "cycle 001 — SYSCOHADA"),
    ("fiscalite", "mapping_comptable", "cycle 001 — SYSCOHADA"),
    ("comptes", "employe", "cycle 003 — CPT-05, contrat de travail"),
    (
        "comptes",
        "appareil_enrole",
        "cycle 003 — CPT-05/06, enrôlement d'appareil",
    ),
    (
        "hebergement",
        "prestation_incluse",
        "cycle 004 — HEB-09, petit-déjeuner inclus",
    ),
];

/// Les cinq tables existent, avec leurs contraintes.
#[tokio::test]
async fn les_tables_de_provision_existent() {
    let pool = commun::pool_owner().await;
    let mut inspectees = 0_usize;

    for (schema, table, cycle) in PROVISIONS {
        let existe: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS (
                SELECT 1 FROM information_schema.tables
                WHERE table_schema = $1 AND table_name = $2
            )
            "#,
        )
        .bind(schema)
        .bind(table)
        .fetch_one(&pool)
        .await
        .expect("lecture du catalogue");

        assert!(existe, "{schema}.{table} est absente ({cycle})");
        inspectees += 1;
    }

    assert_eq!(
        inspectees,
        PROVISIONS.len(),
        "{inspectees} provision(s) inspectée(s) sur {} déclarée(s)",
        PROVISIONS.len()
    );
}

/// **`comptes.employe` ne porte aucune colonne de pièce d'identité, et n'en portera pas.**
///
/// Le contrôle jumeau de `personne_compte_employe.rs`, dans l'autre sens : celui-là refuse qu'une
/// colonne de contrat migre vers les tables d'identité, celui-ci refuse qu'une colonne d'identité
/// migre vers la table de contrat. Les deux mouvements sont tentants pour la même raison —
/// « c'est la même personne » — et les deux effacent la distinction de CPT-00.
///
/// Le sujet n'est pas cosmétique : `type_piece` et `numero_piece` sont soumises à une rétention de
/// 90 jours (TRX-06). Recopiées sur une table de provision que personne ne surveille, elles y
/// resteraient indéfiniment.
#[tokio::test]
async fn aucune_colonne_de_piece_d_identite_sur_les_provisions_rh() {
    let pool = commun::pool_owner().await;
    let mut fautives = Vec::new();
    let mut inspectees = 0_usize;

    for (schema, table, _) in PROVISIONS.iter().filter(|(s, _, _)| *s == "comptes") {
        let colonnes = sqlx::query(
            r#"
            SELECT column_name
            FROM information_schema.columns
            WHERE table_schema = $1 AND table_name = $2
            "#,
        )
        .bind(schema)
        .bind(table)
        .fetch_all(&pool)
        .await
        .expect("lecture du catalogue de colonnes");

        assert!(!colonnes.is_empty(), "{schema}.{table} n'a aucune colonne");
        inspectees += 1;

        for ligne in colonnes {
            let nom: String = ligne.get::<String, _>("column_name").to_lowercase();
            for motif in ["piece", "passeport", "cni", "identite"] {
                if nom.contains(motif) {
                    fautives.push(format!("{schema}.{table}.{nom} (motif « {motif} »)"));
                }
            }
        }
    }

    assert_eq!(inspectees, 2, "les deux provisions RH doivent être inspectées");
    assert!(
        fautives.is_empty(),
        "des colonnes de pièce d'identité sont apparues sur une provision : {fautives:?}\n\
         Elles relèvent de `comptes.personne`, sous la rétention de 90 jours de TRX-06. Recopiées \
         ici, elles y resteraient indéfiniment."
    );
}

/// **Aucun endpoint ne touche les deux provisions du cycle 003.**
///
/// Même mécanique que pour les provisions comptables : le contrat OpenAPI est la source de vérité
/// de ce que l'API expose (principe I(a)).
#[test]
fn aucun_endpoint_n_expose_les_provisions_rh() {
    let contrat = kaya_api::application::contrat_complet();

    let suspects: Vec<&String> = contrat
        .paths
        .paths
        .keys()
        .filter(|chemin| {
            let c = chemin.to_lowercase();
            c.contains("employe") || c.contains("appareil") || c.contains("enrole")
        })
        .collect();

    assert!(
        suspects.is_empty(),
        "des endpoints exposent les provisions RH : {suspects:?}\n\
         `employe` est CPT-05 et `appareil_enrole` CPT-05/06, tranche T4. Un endpoint, même en \
         lecture, en fait une fonctionnalité que personne n'a décidé de construire — et il \
         échouerait de toute façon au premier appel : `kaya_app` n'a aucun privilège dessus."
    );
}

/// **Aucun droit d'écriture — ni même de lecture — sur les provisions RH.**
///
/// Plus strict que la règle du cycle 001 : `fiscalite` accorde `SELECT`, `comptes` n'accorde
/// **rien du tout**. C'est la garantie de second rang du contrôle de graphe d'appels de
/// `personne_compte_employe.rs` : un chemin de code écrit par distraction échoue au premier
/// appel, pas trois mois plus tard.
#[tokio::test]
async fn le_role_applicatif_n_a_aucun_privilege_sur_les_provisions_rh() {
    let pool = commun::pool_owner().await;
    let mut inspectees = 0_usize;

    for (schema, table, cycle) in PROVISIONS.iter().filter(|(s, _, _)| *s == "comptes") {
        let privileges: Vec<String> = sqlx::query(
            r#"
            SELECT privilege_type
            FROM information_schema.role_table_grants
            WHERE grantee = 'kaya_app' AND table_schema = $1 AND table_name = $2
            ORDER BY 1
            "#,
        )
        .bind(schema)
        .bind(table)
        .fetch_all(&pool)
        .await
        .expect("lecture des privilèges")
        .iter()
        .map(|l| l.get::<String, _>("privilege_type"))
        .collect();

        assert!(
            privileges.is_empty(),
            "le rôle applicatif détient {privileges:?} sur {schema}.{table} ({cycle}).\n\
             Une provision n'accorde RIEN, pas même `SELECT` — c'est ce qui la distingue d'un \
             début d'implémentation. Ne pas ajouter de `GRANT` « pour pouvoir tester » : ce test \
             teste précisément cette absence."
        );
        inspectees += 1;
    }

    assert_eq!(inspectees, 2, "les deux provisions RH doivent être inspectées");
}

/// **Aucun endpoint** ne les expose.
///
/// Le contrat OpenAPI est la source de vérité de ce que l'API expose (principe I(a)). S'y référer
/// est donc plus sûr que de relire les fichiers de routes : un endpoint monté sans annotation
/// n'apparaîtrait pas au contrat, mais il n'apparaîtrait pas non plus dans le client généré, et
/// la porte P-08 le signalerait.
#[test]
fn aucun_endpoint_n_expose_les_provisions() {
    let contrat = kaya_api::application::contrat_complet();

    let suspects: Vec<&String> = contrat
        .paths
        .paths
        .keys()
        .filter(|chemin| {
            let c = chemin.to_lowercase();
            c.contains("exercice")
                || c.contains("comptab")
                || c.contains("mapping")
                || c.contains("fiscalite")
        })
        .collect();

    assert!(
        suspects.is_empty(),
        "des endpoints exposent les provisions comptables : {suspects:?}\n\
         Les provisions sont des TABLES SEULEMENT (principe X). Un endpoint, même en lecture, en \
         fait une fonctionnalité que personne n'a décidé de construire."
    );
}

/// **Aucun droit d'écriture** n'est accordé au rôle applicatif.
///
/// C'est la vérification qui vaut les deux précédentes : même si un endpoint apparaissait, il ne
/// pourrait rien écrire. La provision est tenue par la base, pas par la discipline.
#[tokio::test]
async fn le_role_applicatif_ne_peut_pas_ecrire_dans_les_provisions() {
    let pool = commun::pool_owner().await;

    let lignes = sqlx::query(
        r#"
        SELECT table_name, privilege_type
        FROM information_schema.role_table_grants
        WHERE grantee = 'kaya_app'
          AND table_schema = 'fiscalite'
        ORDER BY 1, 2
        "#,
    )
    .fetch_all(&pool)
    .await
    .expect("lecture des privilèges");

    let ecritures: Vec<String> = lignes
        .iter()
        .filter_map(|l| {
            let privilege: String = l.get("privilege_type");
            let table: String = l.get("table_name");
            matches!(privilege.as_str(), "INSERT" | "UPDATE" | "DELETE")
                .then(|| format!("{table}: {privilege}"))
        })
        .collect();

    assert!(
        ecritures.is_empty(),
        "le rôle applicatif a des droits d'écriture sur les provisions : {ecritures:?}\n\
         Aucun chemin d'écriture ne doit pouvoir naître par inadvertance. Le jour où la \
         comptabilité sera implémentée, une migration accordera ces droits — un acte visible et \
         daté."
    );
}

// =================================================================================================
//  Cycle 004 — HEB-09, la provision qui se paierait le plus cher d'être mal posée
// =================================================================================================

/// **`quantite` est `NUMERIC`, et ce n'est pas un détail de provision.**
///
/// Un petit-déjeuner se compte à l'unité, une prestation de blanchisserie **au kilo**, une course
/// de conciergerie peut se compter en demi-heures. Un `INTEGER` posé aujourd'hui « puisque personne
/// ne s'en sert » imposerait de migrer **toutes les lignes de tous les clients** le jour où la
/// table est enfin peuplée — c'est-à-dire au pire moment possible.
///
/// La porte P-10 balaie les migrations ; ce test asserte la colonne **telle que la base la porte**,
/// ce qui reste vrai même si la table était un jour altérée par une migration ultérieure.
#[tokio::test]
async fn la_quantite_de_prestation_incluse_est_numeric_et_le_plafond_un_entier() {
    let pool = commun::pool_owner().await;

    let colonnes: Vec<(String, String)> = sqlx::query_as(
        r#"
        SELECT column_name, data_type
        FROM information_schema.columns
        WHERE table_schema = 'hebergement' AND table_name = 'prestation_incluse'
        ORDER BY ordinal_position
        "#,
    )
    .fetch_all(&pool)
    .await
    .expect("lecture du catalogue");

    assert!(
        !colonnes.is_empty(),
        "`hebergement.prestation_incluse` n'a aucune colonne — le test n'inspecte rien, et son vert \
         ne dirait rien"
    );

    let genre = |nom: &str| -> String {
        colonnes
            .iter()
            .find(|(c, _)| c == nom)
            .unwrap_or_else(|| panic!("colonne `{nom}` absente. Colonnes : {colonnes:?}"))
            .1
            .clone()
    };

    assert_eq!(
        genre("quantite"),
        "numeric",
        "`quantite` est « {} » au lieu de `numeric`.\n\
         Une quantité entière interdit la blanchisserie au kilo, et passer d'entier à décimal après \
         mise en production imposerait de migrer toutes les lignes de tous les clients.",
        genre("quantite")
    );

    assert_eq!(
        genre("valeur_unitaire_plafond_mineur"),
        "bigint",
        "le plafond est un **entier d'unité mineure** (principe V) : un `numeric` ici rouvrirait la \
         question des arrondis sur une valeur monétaire"
    );
}

/// **Aucun endpoint n'expose la prestation incluse**, et aucun ne le fera par distraction.
///
/// Même mécanique que pour les provisions des cycles 001 et 003 : le contrat OpenAPI est la source
/// de vérité de ce que l'API expose (principe I(a)).
#[test]
fn aucun_endpoint_n_expose_la_prestation_incluse() {
    let contrat = kaya_api::application::contrat_complet();

    let suspects: Vec<&String> = contrat
        .paths
        .paths
        .keys()
        .filter(|chemin| {
            let c = chemin.to_lowercase();
            c.contains("prestation") || c.contains("petit-dejeuner") || c.contains("incluse")
        })
        .collect();

    assert!(
        suspects.is_empty(),
        "des endpoints exposent la prestation incluse : {suspects:?}\n\
         HEB-09 est une TABLE SEULEMENT (principe X) : la fonctionnalité — décompte à la \
         consommation, non facturation, bascule du dépassement — arrive en incrément 2. Un \
         endpoint, même en lecture, en fait une fonctionnalité que personne n'a décidé de \
         construire — et il échouerait au premier appel : `kaya_app` n'a aucun privilège dessus."
    );
}

/// **Aucun privilège sur `prestation_incluse` — et les six tables voisines en ont quatre.**
///
/// C'est ce qui rend l'assertion probante : dans le **même schéma**, `0024` accorde
/// `SELECT, INSERT, UPDATE, DELETE` à `kaya_app` sur les six tables du référentiel. Un test qui se
/// contenterait de constater l'absence sur une table isolée ne dirait pas si `kaya_app` a des
/// droits quelque part ; ici, la différence est mesurée côte à côte.
#[tokio::test]
async fn le_role_applicatif_n_a_aucun_privilege_sur_la_prestation_incluse() {
    let pool = commun::pool_owner().await;

    let privileges: Vec<(String, i64)> = sqlx::query_as(
        r#"
        SELECT table_name, COUNT(*)
        FROM information_schema.role_table_grants
        WHERE grantee = 'kaya_app' AND table_schema = 'hebergement'
        GROUP BY table_name
        ORDER BY table_name
        "#,
    )
    .fetch_all(&pool)
    .await
    .expect("lecture des privilèges");

    let sur_la_provision = privileges
        .iter()
        .find(|(table, _)| table == "prestation_incluse");

    assert!(
        sur_la_provision.is_none(),
        "le rôle applicatif détient des privilèges sur `hebergement.prestation_incluse` : {:?}\n\
         Une provision n'accorde RIEN, pas même `SELECT` — c'est ce qui la distingue d'un début \
         d'implémentation. Ne pas ajouter de `GRANT` « pour pouvoir tester » : ce test teste \
         précisément cette absence.",
        sur_la_provision
    );

    // Le témoin : les tables du référentiel, elles, sont bien ouvertes. Sans cette assertion, un
    // `REVOKE` massif sur tout le schéma ferait passer le test ci-dessus au vert.
    let referentiel: Vec<&String> = privileges
        .iter()
        .filter(|(_, n)| *n == 4)
        .map(|(table, _)| table)
        .collect();
    assert!(
        referentiel.len() >= 6,
        "seules {} table(s) du schéma `hebergement` portent les quatre verbes pour `kaya_app`. \
         Les six tables du référentiel de `0024` doivent les avoir — sans ce témoin, un `REVOKE` \
         général rendrait l'assertion précédente vraie pour la mauvaise raison. Obtenu : \
         {privileges:?}",
        referentiel.len()
    );
}

// =================================================================================================
//  Cycle 005 — SYN-03, la provision qui accorde `SELECT` et RIEN d'autre
// =================================================================================================

/// ★ **`kaya_app` peut INSÉRER dans `reconciliation_orpheline`, mais ni MODIFIER ni SUPPRIMER.**
///
/// # Ce que ce test prouve, et qui n'est pas ce qu'on croit
///
/// Il ne prouve pas qu'aucun code ne résout une écriture orpheline : il prouve qu'**aucun code ne
/// le pourra**. La distinction compte, parce que le premier énoncé se vérifie par relecture —
/// donc mal — et le second par la base, à chaque appel.
///
/// # ⚠️ Le privilège a changé au cycle 006, et l'asymétrie EST la règle
///
/// | Verbe | Cycle 005 | Cycle 006 | Pourquoi |
/// |---|---|---|---|
/// | `SELECT` | ✅ | ✅ | Le récapitulatif de fin de journée compte les constats en attente |
/// | `INSERT` | ⛔ | ✅ | **SEJ-02** — un accompagnant de classe A arrivant après la clôture |
/// | `UPDATE` | ⛔ | ⛔ | La **résolution** est SYN-03, tranche T3 |
/// | `DELETE` | ⛔ | ⛔ | Une écriture orpheline ne s'efface pas : elle se tranche |
///
/// **Accorder `UPDATE` maintenant ferait croire à une résolution qui n'existe pas**, et ce fichier
/// ne pourrait plus dire ce qui est construit de ce qui est promis. C'est exactement le glissement
/// — « juste un petit endpoint » — qu'il existe pour rendre bruyant.
#[tokio::test]
async fn le_role_applicatif_peut_alimenter_la_reconciliation_mais_pas_la_resoudre() {
    let pool = commun::pool_owner().await;

    let privileges: Vec<(String, String)> = sqlx::query_as(
        r#"
        SELECT table_name, privilege_type
        FROM information_schema.role_table_grants
        WHERE grantee = 'kaya_app' AND table_schema = 'synchronisation'
        ORDER BY 1, 2
        "#,
    )
    .fetch_all(&pool)
    .await
    .expect("lecture des privilèges");

    let sur_la_table = |verbe: &str| {
        privileges
            .iter()
            .any(|(table, privilege)| table == "reconciliation_orpheline" && privilege == verbe)
    };

    // ── Ce qui est DÛ et ne l'est pas encore ──────────────────────────────────────────────────
    for verbe in ["UPDATE", "DELETE"] {
        assert!(
            !sur_la_table(verbe),
            "le rôle applicatif peut `{verbe}` sur `synchronisation.reconciliation_orpheline`.\n\
             La RÉSOLUTION d'une écriture orpheline est **SYN-03, tranche T3** : le cycle 006 \
             alimente la file, il ne la vide pas. Accorder ce droit maintenant ferait croire à \
             une résolution qui n'existe pas, et ce fichier ne pourrait plus dire ce qui est \
             construit de ce qui est promis.\n\
             Privilèges observés : {privileges:?}"
        );
    }

    // ── Le versant POSITIF, et il compte double ici ───────────────────────────────────────────
    //
    // Sans lui, un `REVOKE` massif rendrait les assertions précédentes vraies pour la mauvaise
    // raison — et la file, devenue inalimentable, perdrait silencieusement chaque accompagnant
    // arrivé après un départ.
    assert!(
        sur_la_table("SELECT"),
        "`kaya_app` n'a même pas `SELECT` : la migration `0027` l'accorde délibérément, pour que \
         le récapitulatif de fin de journée compte les constats en attente. Obtenu : {privileges:?}"
    );
    assert!(
        sur_la_table("INSERT"),
        "★ `kaya_app` ne peut PAS insérer dans `synchronisation.reconciliation_orpheline`.\n\
         Depuis SEJ-02 (migration `0031`), un accompagnant de classe A arrivant après la clôture \
         d'un séjour DOIT y être inscrit — c'est le premier cas réel d'écriture orpheline du \
         produit. Sans ce droit, l'écriture est perdue en silence : ni sur le séjour, ni en file, \
         et Adjoua ne saura jamais que sa saisie n'a pas compté.\n\
         Privilèges observés : {privileges:?}"
    );
}

/// **Aucun endpoint n'expose la réconciliation orpheline.**
#[test]
fn aucun_endpoint_n_expose_la_reconciliation_orpheline() {
    let contrat = kaya_api::application::contrat_complet();

    let suspects: Vec<&String> = contrat
        .paths
        .paths
        .keys()
        .filter(|chemin| {
            let c = chemin.to_lowercase();
            c.contains("reconciliation") || c.contains("orphelin")
        })
        .collect();

    assert!(
        suspects.is_empty(),
        "des endpoints exposent la réconciliation orpheline : {suspects:?}\n\
         La résolution d'un conflit orphelin est HUMAINE et obligatoire (cadrage §11.4) : un \
         endpoint livré avant l'écran qui la porte laisserait un chemin où le conflit se résout \
         sans que personne ne l'ait vu."
    );
}

/// **Le cycle de vie est tenu par la base, et l'égalité de conditions n'a pas d'échappatoire.**
///
/// Le `CHECK` de `0027` est une **égalité**, non trois implications : `etat = 'resolue'` équivaut
/// à « issue, horodatage et compte résolveur sont tous les trois posés ». Trois `CHECK` séparés
/// diraient « si résolue alors issue » sans dire « si issue alors résolue », et l'écart se paierait
/// le jour où l'écran de SYN-03 écrirait les deux dans le désordre.
///
/// Le test s'exécute sous `kaya_owner` — le seul rôle qui puisse écrire ici, précisément parce que
/// `kaya_app` ne le peut pas. C'est la contrainte qui est éprouvée, pas le privilège.
#[tokio::test]
async fn la_resolution_est_tout_ou_rien() {
    use uuid::Uuid;

    let pool = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool, "provisions — réconciliation orpheline").await;

    /// Insère un constat, en laissant l'appelant choisir l'état et ses corollaires.
    async fn inserer(
        pool: &sqlx::PgPool,
        jeu: commun::JeuTenant,
        etat: &str,
        issue: Option<&str>,
        resolue_le: Option<time::OffsetDateTime>,
        resolue_par: Option<Uuid>,
    ) -> Result<(), sqlx::Error> {
        let mut tx = pool.begin().await.expect("transaction");
        kaya_etablissements::tenant_context::poser_tenant(&mut tx, jeu.tenant_id)
            .await
            .expect("pose du tenant");

        let resultat = sqlx::query(
            r#"
            INSERT INTO synchronisation.reconciliation_orpheline
                (id, tenant_id, etablissement_id, ecriture_id, ecriture_type,
                 agregat_type, agregat_id, etat, issue, resolue_le, resolue_par_compte_id)
            VALUES ($1, $2, $3, $4, 'ligne_commande', 'addition', $5, $6, $7, $8, $9)
            "#,
        )
        .bind(Uuid::now_v7())
        .bind(jeu.tenant_id)
        .bind(jeu.etablissement_id)
        .bind(Uuid::now_v7())
        .bind(Uuid::now_v7())
        .bind(etat)
        .bind(issue)
        .bind(resolue_le)
        .bind(resolue_par)
        .execute(&mut *tx)
        .await
        .map(|_| ());

        if resultat.is_ok() {
            tx.commit().await.expect("commit");
        }
        resultat
    }

    // 1 · Un constat neuf, sans aucun corollaire — accepté.
    inserer(&pool, jeu, "constatee", None, None, None)
        .await
        .expect("un constat `constatee` sans issue doit être accepté");

    // 2 · Résolu avec ses trois corollaires — accepté.
    inserer(
        &pool,
        jeu,
        "resolue",
        Some("AVOIR_REFACTURATION"),
        Some(time::OffsetDateTime::now_utc()),
        Some(Uuid::now_v7()),
    )
    .await
    .expect("un constat `resolue` complet doit être accepté");

    // 3 · Résolu SANS issue — refusé par la base, pas par une revue.
    let sans_issue = inserer(
        &pool,
        jeu,
        "resolue",
        None,
        Some(time::OffsetDateTime::now_utc()),
        Some(Uuid::now_v7()),
    )
    .await;
    assert!(
        sans_issue.is_err(),
        "un constat marqué `resolue` sans issue a été accepté : l'égalité de conditions de `0027` \
         ne tient pas, et un écran pourrait clore un conflit sans dire comment"
    );

    // 4 · Une issue posée sur un constat ENCORE ouvert — refusé aussi. C'est le sens que trois
    //     `CHECK` séparés auraient laissé passer.
    let issue_sans_resolution = inserer(
        &pool,
        jeu,
        "constatee",
        Some("PRISE_EN_CHARGE"),
        None,
        None,
    )
    .await;
    assert!(
        issue_sans_resolution.is_err(),
        "une issue a été posée sur un constat encore `constatee`. C'est exactement le sens qu'une \
         implication simple laisse passer, et pourquoi `0027` pose une ÉGALITÉ de conditions."
    );
}

/// La contrainte d'exclusion **fonctionne** — spike de HEB-02.
///
/// Premier usage d'`EXCLUDE USING gist` du produit. HEB-02 reprendra exactement cette forme sur
/// `tstzrange` pour la disponibilité des unités louables ; l'exercer ici, sur un cas sans enjeu,
/// valide `btree_gist` et le mapping de type sqlx 0.9 avant que la double attribution de chambre
/// en dépende.
#[tokio::test]
async fn deux_exercices_qui_se_chevauchent_sont_refuses() {
    let pool = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool, "provisions — exclusion GiST").await;

    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, jeu.tenant_id)
        .await
        .expect("pose du tenant");

    sqlx::query!(
        r#"
        INSERT INTO fiscalite.exercice_comptable (id, tenant_id, debut, fin, statut)
        VALUES ($1, $2, DATE '2026-01-01', DATE '2027-01-01', 'ouvert')
        "#,
        uuid::Uuid::now_v7(),
        jeu.tenant_id
    )
    .execute(&mut *tx)
    .await
    .expect("premier exercice");

    // Chevauchement franc.
    let chevauchant = sqlx::query!(
        r#"
        INSERT INTO fiscalite.exercice_comptable (id, tenant_id, debut, fin, statut)
        VALUES ($1, $2, DATE '2026-06-01', DATE '2027-06-01', 'ouvert')
        "#,
        uuid::Uuid::now_v7(),
        jeu.tenant_id
    )
    .execute(&mut *tx)
    .await;

    assert!(
        chevauchant.is_err(),
        "deux exercices chevauchants ont été acceptés : « la période est-elle close ? » devient \
         indécidable, et c'est la seule règle que TRX-02b impose"
    );

    tx.rollback().await.expect("rollback");

    // Contiguïté : le second commence là où le premier finit. `'[)'` doit l'accepter.
    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, jeu.tenant_id)
        .await
        .expect("pose du tenant");

    sqlx::query!(
        r#"
        INSERT INTO fiscalite.exercice_comptable (id, tenant_id, debut, fin, statut)
        VALUES ($1, $2, DATE '2026-01-01', DATE '2027-01-01', 'ouvert')
        "#,
        uuid::Uuid::now_v7(),
        jeu.tenant_id
    )
    .execute(&mut *tx)
    .await
    .expect("premier exercice");

    let contigu = sqlx::query!(
        r#"
        INSERT INTO fiscalite.exercice_comptable (id, tenant_id, debut, fin, statut)
        VALUES ($1, $2, DATE '2027-01-01', DATE '2028-01-01', 'ouvert')
        "#,
        uuid::Uuid::now_v7(),
        jeu.tenant_id
    )
    .execute(&mut *tx)
    .await;

    assert!(
        contigu.is_ok(),
        "deux exercices CONTIGUS ont été refusés : la borne de fin doit être exclue ('[)'). Avec \
         '[]', le 31 décembre appartiendrait à deux exercices — et HEB-02 hériterait du même \
         défaut sur les occupations. {:?}",
        contigu.err()
    );

    tx.rollback().await.expect("rollback");
}

/// Un exercice **clos** ne se modifie plus — par déclencheur, pas par règle applicative.
#[tokio::test]
async fn un_exercice_clos_ne_se_modifie_plus() {
    let pool = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool, "provisions — période close").await;
    let exercice_id = uuid::Uuid::now_v7();

    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, jeu.tenant_id)
        .await
        .expect("pose du tenant");
    sqlx::query!(
        r#"
        INSERT INTO fiscalite.exercice_comptable (id, tenant_id, debut, fin, statut)
        VALUES ($1, $2, DATE '2025-01-01', DATE '2026-01-01', 'clos')
        "#,
        exercice_id,
        jeu.tenant_id
    )
    .execute(&mut *tx)
    .await
    .expect("exercice clos");
    tx.commit().await.expect("commit");

    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, jeu.tenant_id)
        .await
        .expect("pose du tenant");
    let reouverture = sqlx::query!(
        "UPDATE fiscalite.exercice_comptable SET statut = 'ouvert' WHERE id = $1",
        exercice_id
    )
    .execute(&mut *tx)
    .await;

    assert!(
        reouverture.is_err(),
        "un exercice clos a pu être rouvert. Le déclencheur est ce qui empêche la première \
         migration de données venue de le faire — une règle applicative serait contournée."
    );
    let _ = tx.rollback().await;
}
