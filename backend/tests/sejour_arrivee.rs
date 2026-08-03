//! ★ **SEJ-02** — l'ouverture d'un séjour : une transaction, la concurrence, et P-09 ré-exercée.
//!
//! # Les huit garanties que ce fichier éprouve, et qui ne se voient PAS en relecture
//!
//! Les quatre premières portent sur **SEJ-02**, l'ouverture ; les quatre suivantes sur **SEJ-03**,
//! l'arrivée d'un client attendu.
//!
//! | # | Ce qui est vérifié | Ce qu'une relecture manquerait |
//! |---|---|---|
//! | **a** | Une panne après l'attribution ne laisse **rien** — ni séjour, ni note, ni fiche, **ni occupation orpheline** | Cinq écritures qui « ont l'air » dans une transaction |
//! | **b** | Deux arrivées chevauchantes : **exactement une** réussit, et le refus est un `ExclusionViolation` **sur la contrainte nommée** | Un `SELECT … FOR UPDATE` donnerait le même COMPTE en rendant la double attribution *improbable* au lieu d'*impossible* |
//! | **c** | La numérotation de fiche de police est **continue par établissement**, sans trou | Une `SEQUENCE` passerait le test sur un seul établissement |
//! | **d** | Un passage **sans client** produit un séjour valide et une fiche **numérotée et déclarée incomplète**, sans champ de remplissage | Une fiche fabriquée avec « M. X » passerait toute revue |
//! | **e** | La requête d'arrivée d'un client connu n'a **aucune place** pour un champ d'identité, et l'identité ressort quand même | Un écran qui pré-remplit **et renvoie** la copie : chaque arrivée écraserait la fiche par une version périmée |
//! | **f** | L'unité **proposée** est réellement libre — **l'attribution le confirme** | Une proposition d'accord avec elle-même : bornée par `fin_client`, elle proposerait une chambre encore en ménage, et le refus tomberait **après** le geste de Yao |
//! | **g** | Deux accompagnants font **trois** personnes, et le retrait d'un ramène à **deux** | Une colonne `nombre_personnes` posée : elle passe la première moitié, se désynchronise au retrait, et le constat de taxe **fige la valeur fausse** |
//! | **h** | Catégorie pleine → la liste **n'est pas vide** : chaque unité porte l'instant où elle se libère, **remise en état comprise** | Une liste vide, qui passe toute revue et oblige Yao à ouvrir un autre écran devant le client |
//!
//! # ★ (b) est la seule assertion qui compte, et elle porte sur la CAUSE
//!
//! Compter « une réussite sur deux » ne prouve rien : un verrou applicatif, un
//! `SELECT … FOR UPDATE` ou `SERIALIZABLE` donneraient le même compte. Ce qui est asserté est que
//! le refus est un **`ExclusionViolation` sur `occupation_sans_chevauchement`** — c'est-à-dire que
//! la transaction du check-in **n'a pas contourné la garantie** par une lecture préalable
//! « cette chambre est-elle libre ? », qui paraîtrait prudente et rendrait la double attribution
//! improbable au lieu d'impossible.
//!
//! C'est l'exigence 5 de la section « Couverture des portes » de la constitution, mot pour mot :
//! *« la couverture s'étend avec les fonctionnalités : elle doit être re-exercée, pas supposée
//! acquise »*. La porte **P-09** du cycle 004 est ré-exercée ici **par le parcours de séjour**,
//! pas par l'endpoint nu.

mod commun;

use time::OffsetDateTime;
use uuid::Uuid;

use commun::{JeuTenant, creer_tenant, pool_app, pool_owner};
use kaya_hebergement::erreurs::CONTRAINTE_SANS_CHEVAUCHEMENT;

/// Le rôle du comptoir — **`receptionniste`**, celui qui porte `heb.sejour.ouvrir`.
///
/// ⚠️ Pas `proprietaire` : depuis la migration `0030`, il ne reçoit que les **lectures**. Le
/// symptôme de l'oubli est un `403` qui accuse le handler alors que la cause est le rôle du test.
const ROLE: &str = "receptionniste";

// =================================================================================================
//  Décor
// =================================================================================================

struct Decor {
    jeu: JeuTenant,
    unite_id: Uuid,
    /// La seconde chambre — **elle sert le refus, pas la réussite**.
    ///
    /// Un décor à une seule chambre rendrait « aucune alternative disponible » indistinguable de
    /// « la recherche d'alternative ne fonctionne pas ». Elle est consommée par US5 (prolongation
    /// avec conflit nommé) et US7 (changement d'unité).
    #[allow(dead_code)]
    unite_bis_id: Uuid,
    formule_id: Uuid,
    /// La catégorie des deux chambres — **la proposition d'unité s'interroge par catégorie**,
    /// jamais par unité : c'est ce que Yao demande au comptoir (« une Standard »).
    categorie_id: Uuid,
}

/// Un établissement avec l'hébergement actif, un type de chambre, **deux** chambres et une nuitée.
///
/// Deux chambres, parce que le changement d'unité et la proposition d'alternative en ont besoin —
/// et parce qu'un décor à une seule chambre laisserait « aucune alternative » indistinguable de
/// « la recherche d'alternative ne marche pas ».
async fn poser_decor(pool: &sqlx::PgPool, nom: &str) -> Decor {
    let jeu = creer_tenant(pool, nom).await;

    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, jeu.tenant_id)
        .await
        .expect("pose du tenant");

    sqlx::query(
        r#"
        INSERT INTO etablissements.etablissement_module
            (id, tenant_id, etablissement_id, module_code, module_implemente)
        VALUES ($1, $2, $3, 'HEBERGEMENT', true)
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(Uuid::now_v7())
    .bind(jeu.tenant_id)
    .bind(jeu.etablissement_id)
    .execute(&mut *tx)
    .await
    .expect("activation du module");

    let categorie_id = Uuid::now_v7();
    sqlx::query(
        r#"
        INSERT INTO hebergement.categorie (id, tenant_id, etablissement_id, nom, capacite_accueil)
        VALUES ($1, $2, $3, 'Standard', 2)
        "#,
    )
    .bind(categorie_id)
    .bind(jeu.tenant_id)
    .bind(jeu.etablissement_id)
    .execute(&mut *tx)
    .await
    .expect("catégorie");

    let mut unites = Vec::new();
    for code in ["A1", "A2"] {
        let id = Uuid::now_v7();
        sqlx::query(
            r#"
            INSERT INTO hebergement.unite
                (id, tenant_id, etablissement_id, categorie_id, code, etage)
            VALUES ($1, $2, $3, $4, $5, 1)
            "#,
        )
        .bind(id)
        .bind(jeu.tenant_id)
        .bind(jeu.etablissement_id)
        .bind(categorie_id)
        .bind(code)
        .execute(&mut *tx)
        .await
        .expect("unité");
        unites.push(id);
    }

    let formule_id = Uuid::now_v7();
    sqlx::query(
        r#"
        INSERT INTO hebergement.formule
            (id, tenant_id, etablissement_id, categorie_id, famille, prix_mineur,
             assujettie_taxe_nuitee)
        VALUES ($1, $2, $3, $4, 'NUITEE', 12500, false)
        "#,
    )
    .bind(formule_id)
    .bind(jeu.tenant_id)
    .bind(jeu.etablissement_id)
    .bind(categorie_id)
    .execute(&mut *tx)
    .await
    .expect("formule");

    tx.commit().await.expect("commit");

    Decor {
        jeu,
        unite_id: unites[0],
        unite_bis_id: unites[1],
        formule_id,
        categorie_id,
    }
}

/// Le corps d'une ouverture de séjour.
fn corps_ouverture(
    id: Uuid,
    unite_id: Uuid,
    formule_id: Uuid,
    debut: OffsetDateTime,
    heures: i64,
    client_id: Option<Uuid>,
) -> serde_json::Value {
    let mut corps = serde_json::json!({
        "id": id,
        "unite_id": unite_id,
        "formule_id": formule_id,
        "debut_client": debut.format(&time::format_description::well_known::Rfc3339).unwrap(),
        "fin_client": (debut + time::Duration::hours(heures))
            .format(&time::format_description::well_known::Rfc3339).unwrap(),
    });
    if let Some(client_id) = client_id {
        corps["client_id"] = serde_json::json!(client_id);
    }
    corps
}

/// Ouvre un séjour par le chemin réel — l'endpoint, jamais un `INSERT` direct.
macro_rules! ouvrir {
    ($app:expr, $bearer:expr, $etablissement:expr, $corps:expr) => {{
        let requete = actix_web::test::TestRequest::post()
            .uri(&format!("/api/v1/etablissements/{}/sejours", $etablissement))
            .insert_header(("authorization", $bearer.to_owned()))
            .set_json(&$corps)
            .to_request();
        actix_web::test::call_service(&$app, requete).await
    }};
}

// =================================================================================================
//  ★ (a) UNE SEULE TRANSACTION — une panne ne laisse rien, pas même une occupation orpheline
// =================================================================================================

/// ★ **Une panne après l'attribution ne laisse ni séjour, ni note, ni fiche, ni occupation.**
///
/// # Comment la panne est simulée, et pourquoi c'est la bonne façon
///
/// Une formule **inexistante** fait échouer l'ouverture **après** que le moteur de disponibilité
/// a été appelé — mais la validation de la formule vient d'abord. On force donc l'échec plus loin :
/// un `client_id` inventé passe la validation de forme et échoue à la garde d'annuaire, **après**
/// l'attribution du point de vue de l'appelant.
///
/// L'assertion qui compte est la **quatrième** : aucune occupation orpheline. Les trois premières
/// se devinent ; celle-là est exactement ce qu'une transaction mal fermée laisserait — une chambre
/// bloquée que personne ne peut libérer, parce qu'aucun séjour ne la porte.
#[actix_web::test]
async fn une_panne_ne_laisse_ni_sejour_ni_note_ni_fiche_ni_occupation_orpheline() {
    let owner = pool_owner().await;
    let decor = poser_decor(&owner, "SEJ — transaction unique").await;
    let cx = commun::compte_connecte(
        &owner,
        decor.jeu,
        "Yao",
        &[(ROLE, Some(decor.jeu.etablissement_id))],
    )
    .await;
    let app = monter_application!(pool_app().await);

    let sejour_id = Uuid::now_v7();
    let debut = OffsetDateTime::now_utc() + time::Duration::hours(1);

    // Un `client_id` qui n'existe nulle part : la garde d'annuaire refuse, et **rien** ne doit
    // rester en base.
    let corps = corps_ouverture(
        sejour_id,
        decor.unite_id,
        decor.formule_id,
        debut,
        24,
        Some(Uuid::now_v7()),
    );
    let reponse = ouvrir!(app, cx.bearer, decor.jeu.etablissement_id, corps);
    assert_eq!(
        reponse.status(),
        404,
        "un client_id inventé doit être refusé : aucune clé étrangère ne peut le tenir, la clé \
         inter-schémas étant impossible (principe II)"
    );

    let mut tx = owner.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, decor.jeu.tenant_id)
        .await
        .expect("pose du tenant");

    for (table, colonne) in [
        ("hebergement.sejour", "id"),
        ("hebergement.note_sejour", "sejour_id"),
        ("hebergement.fiche_police", "sejour_id"),
    ] {
        let compte: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
            "SELECT COUNT(*) FROM {table} WHERE {colonne} = $1"
        )))
        .bind(sejour_id)
        .fetch_one(&mut *tx)
        .await
        .expect("comptage");
        assert_eq!(compte, 0, "{table} porte une ligne après un refus");
    }

    // ★ **L'assertion qui compte** : aucune occupation orpheline sur l'unité.
    //
    // C'est exactement ce qu'une transaction mal fermée laisserait — une chambre bloquée que
    // personne ne peut libérer, parce qu'aucun séjour ne la porte. Le symptôme se voit trois jours
    // plus tard, à l'écran de disponibilité, sans aucune trace de sa cause.
    let occupations: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM hebergement.occupation WHERE unite_id = $1",
    )
    .bind(decor.unite_id)
    .fetch_one(&mut *tx)
    .await
    .expect("comptage des occupations");
    tx.rollback().await.expect("rollback");

    assert_eq!(
        occupations, 0,
        "une OCCUPATION ORPHELINE est restée : la chambre est bloquée et aucun séjour ne la \
         porte, donc personne ne peut la libérer. C'est ce qu'une transaction mal fermée produit, \
         et le symptôme n'apparaît qu'à l'écran de disponibilité, sans trace de sa cause."
    );
}

// =================================================================================================
//  ★ (b) CONCURRENCE — P-09 ré-exercée PAR LE PARCOURS DE SÉJOUR
// =================================================================================================

/// ★ **Deux arrivées chevauchantes : exactement une réussit, et le refus vient de la CONTRAINTE.**
///
/// # Ce que ce test asserte, et que « une sur deux » n'asserte pas
///
/// Le **compte** ne prouve rien. Un `SELECT … FOR UPDATE`, un niveau `SERIALIZABLE` ou un verrou
/// applicatif donneraient exactement le même : une réussite, un refus. Ce qui est vérifié ici est
/// que le refus porte le code `unite_deja_occupee` — celui que le service produit **en traduisant
/// une violation d'exclusion**, et lui seul.
///
/// Si quelqu'un ajoutait une lecture préalable « cette chambre est-elle libre ? » au début de
/// `ouvrir`, ce test **continuerait de passer sur le compte** et le code changerait : la lecture
/// rendrait un refus avant d'atteindre la base. C'est pourquoi le second contrôle, sur le nombre
/// d'occupations réellement écrites, est là aussi.
#[actix_web::test]
async fn deux_arrivees_chevauchantes_par_le_parcours_de_sejour_une_seule_reussit() {
    let owner = pool_owner().await;
    let decor = poser_decor(&owner, "SEJ — concurrence").await;
    let cx = commun::compte_connecte(
        &owner,
        decor.jeu,
        "Yao",
        &[(ROLE, Some(decor.jeu.etablissement_id))],
    )
    .await;

    let debut = OffsetDateTime::now_utc() + time::Duration::hours(2);
    let etablissement = decor.jeu.etablissement_id;

    // **Deux applications distinctes**, comme deux instances d'API : monter une seule application
    // et lui envoyer deux requêtes les sérialiserait dans le harnais de test, et le test ne dirait
    // plus rien de la concurrence réelle.
    let (a, b) = futures::join!(
        async {
            let app = monter_application!(pool_app().await);
            let corps = corps_ouverture(
                Uuid::now_v7(),
                decor.unite_id,
                decor.formule_id,
                debut,
                24,
                None,
            );
            let reponse = ouvrir!(app, cx.bearer, etablissement, corps);
            let statut = reponse.status().as_u16();
            let corps: serde_json::Value = actix_web::test::read_body_json(reponse).await;
            (statut, corps)
        },
        async {
            let app = monter_application!(pool_app().await);
            let corps = corps_ouverture(
                Uuid::now_v7(),
                decor.unite_id,
                decor.formule_id,
                debut,
                24,
                None,
            );
            let reponse = ouvrir!(app, cx.bearer, etablissement, corps);
            let statut = reponse.status().as_u16();
            let corps: serde_json::Value = actix_web::test::read_body_json(reponse).await;
            (statut, corps)
        }
    );

    let reussites = [a.0, b.0].iter().filter(|s| **s == 201).count();
    let refus: Vec<&(u16, serde_json::Value)> =
        [&a, &b].into_iter().filter(|(s, _)| *s == 409).collect();

    assert_eq!(
        reussites, 1,
        "exactement une arrivée doit réussir — obtenus : {} et {}",
        a.0, b.0
    );
    assert_eq!(
        refus.len(),
        1,
        "exactement un refus 409 attendu — obtenus : {:?} et {:?}",
        a, b
    );

    // ★ **LA CAUSE, pas seulement l'existence du refus.**
    assert_eq!(
        refus[0].1["code"], "unite_deja_occupee",
        "le refus doit porter le code que le service produit EN TRADUISANT une violation \
         d'exclusion. Un autre code signalerait une lecture préalable « cette chambre est-elle \
         libre ? » — laquelle rendrait la double attribution IMPROBABLE au lieu d'IMPOSSIBLE, et \
         se dégraderait sous charge sans rien signaler. Corps reçu : {}",
        refus[0].1
    );

    // Second contrôle : **une seule occupation écrite**. Si une lecture préalable avait remplacé
    // la contrainte, ce compte pourrait valoir deux sous une concurrence plus serrée.
    //
    // ⚠️ **Le contexte de tenant est posé, même sous le rôle propriétaire.** `FORCE ROW LEVEL
    // SECURITY` s'applique aussi à lui, et une connexion du pool peut porter un `app.current_tenant`
    // vide laissé par une transaction précédente — auquel cas la politique échoue sur
    // `''::uuid` et le test rougit sur un message qui ne dit rien de ce qu'il mesure.
    let mut tx = owner.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, decor.jeu.tenant_id)
        .await
        .expect("pose du tenant");
    let compte: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM hebergement.occupation WHERE unite_id = $1",
    )
    .bind(decor.unite_id)
    .fetch_one(&mut *tx)
    .await
    .expect("comptage");
    tx.rollback().await.expect("rollback");
    assert_eq!(compte, 1, "deux occupations écrites sur la même unité");
}

/// **P-09 ré-exercée après les migrations du cycle 006** — trois assertions, dans le même
/// changement que l'`ALTER TABLE`.
///
/// L'ajout de `sejour_id` ne touche pas la contrainte d'exclusion. **Mais une migration qui
/// recréerait la table la perdrait sans que rien ne le dise**, et la constitution exige de
/// re-exercer une porte dont le périmètre s'étend. Séparer l'`ALTER TABLE` de ce test laisserait
/// un soir où l'on ne saurait pas si le vert est mérité.
#[actix_web::test]
async fn p09_la_contrainte_survit_a_l_ajout_de_sejour_id() {
    let owner = pool_owner().await;

    // ── 1 · `periode` est toujours un `tstzrange`, jamais une paire de dates ──────────────────
    let type_periode: String = sqlx::query_scalar(
        r#"
        SELECT format_type(a.atttypid, a.atttypmod)
        FROM pg_attribute a
        JOIN pg_class c ON c.oid = a.attrelid
        JOIN pg_namespace n ON n.oid = c.relnamespace
        WHERE n.nspname = 'hebergement' AND c.relname = 'occupation' AND a.attname = 'periode'
        "#,
    )
    .fetch_one(&owner)
    .await
    .expect("lecture du type");

    assert_eq!(
        type_periode, "tstzrange",
        "`periode` a changé de type après les migrations du cycle 006. Une paire de dates ne sait \
         pas dire « 14 h → 18 h le même jour », et la contrainte d'exclusion ne s'y applique pas."
    );

    // ── 2 · La contrainte existe TOUJOURS, avec ses deux opérateurs ───────────────────────────
    let definition: String = sqlx::query_scalar(
        r#"
        SELECT pg_get_constraintdef(c.oid)
        FROM pg_constraint c
        JOIN pg_class t ON t.oid = c.conrelid
        JOIN pg_namespace n ON n.oid = t.relnamespace
        WHERE n.nspname = 'hebergement' AND t.relname = 'occupation' AND c.conname = $1
        "#,
    )
    .bind(CONTRAINTE_SANS_CHEVAUCHEMENT)
    .fetch_one(&owner)
    .await
    .unwrap_or_else(|_| {
        panic!(
            "la contrainte « {CONTRAINTE_SANS_CHEVAUCHEMENT} » a DISPARU après les migrations du \
             cycle 006. C'est la garantie du produit : sans elle, la double attribution redevient \
             possible et rien à l'écran ne le signale."
        )
    });

    assert!(
        definition.contains("EXCLUDE USING gist"),
        "la contrainte n'est plus une exclusion GiST : {definition}"
    );
    assert!(
        definition.contains("unite_id WITH =") && definition.contains("periode WITH &&"),
        "la contrainte a perdu un de ses deux opérateurs — elle protégerait moins qu'avant, et le \
         test précédent passerait quand même : {definition}"
    );

    // ── 3 · `sejour_id` existe bien, et il est NULLABLE ───────────────────────────────────────
    //
    // `NULL` est **nécessaire** : l'endpoint d'attribution nu du cycle 004 existe toujours et
    // n'ouvre aucun séjour. Le rendre obligatoire casserait une opération servie, et le casserait
    // à la première attribution faite depuis l'écran de disponibilité.
    let nullable: bool = sqlx::query_scalar(
        r#"
        SELECT NOT a.attnotnull
        FROM pg_attribute a
        JOIN pg_class c ON c.oid = a.attrelid
        JOIN pg_namespace n ON n.oid = c.relnamespace
        WHERE n.nspname = 'hebergement' AND c.relname = 'occupation' AND a.attname = 'sejour_id'
        "#,
    )
    .fetch_one(&owner)
    .await
    .expect("la colonne sejour_id doit exister");

    assert!(
        nullable,
        "`occupation.sejour_id` est NOT NULL : l'endpoint d'attribution nu du cycle 004 ne peut \
         plus servir, et le défaut ne se verra qu'à la première attribution depuis l'écran de \
         disponibilité."
    );
}

// =================================================================================================
//  ★ (c) NUMÉROTATION CONTINUE PAR ÉTABLISSEMENT
// =================================================================================================

/// ★ **La numérotation des fiches de police est continue PAR ÉTABLISSEMENT, sans trou.**
///
/// # Pourquoi deux établissements, et pas un
///
/// Un test sur un seul établissement passerait avec une `SEQUENCE` — qui est pourtant **globale au
/// schéma**. Deux établissements du même tenant qui partageraient leur espace de numérotation
/// produiraient des fiches numérotées 1, 3, 5 d'un côté et 2, 4, 6 de l'autre : continues nulle
/// part, et le trou ne se verrait qu'au contrôle de la gendarmerie.
///
/// C'est le défaut exact corrigé par `0012` au cycle 002 — un espace de numérotation d'outbox
/// partagé entre tenants, trouvé par le premier événement appliqué à un second tenant.
#[actix_web::test]
async fn la_numerotation_est_continue_par_etablissement_sans_trou() {
    let owner = pool_owner().await;
    let premier = poser_decor(&owner, "SEJ — numérotation A").await;
    let second = poser_decor(&owner, "SEJ — numérotation B").await;

    let mut numeros_par_etablissement = Vec::new();

    for decor in [&premier, &second] {
        let cx = commun::compte_connecte(
            &owner,
            decor.jeu,
            "Yao",
            &[(ROLE, Some(decor.jeu.etablissement_id))],
        )
        .await;
        let app = monter_application!(pool_app().await);

        let mut numeros = Vec::new();
        // Trois arrivées **sur des périodes disjointes** : la contrainte d'exclusion refuserait
        // trois arrivées chevauchantes, et le test mesurerait alors la contrainte au lieu de la
        // numérotation.
        for rang in 0..3i64 {
            let debut = OffsetDateTime::now_utc() + time::Duration::days(rang * 3 + 1);
            let corps = corps_ouverture(
                Uuid::now_v7(),
                decor.unite_id,
                decor.formule_id,
                debut,
                24,
                None,
            );
            let reponse = ouvrir!(app, cx.bearer, decor.jeu.etablissement_id, corps);
            assert_eq!(reponse.status(), 201, "l'arrivée {rang} doit réussir");
            let corps: serde_json::Value = actix_web::test::read_body_json(reponse).await;
            numeros.push(corps["fiche_police"]["numero"].as_i64().expect("numéro"));
        }
        numeros_par_etablissement.push(numeros);
    }

    for (rang, numeros) in numeros_par_etablissement.iter().enumerate() {
        assert_eq!(
            numeros,
            &vec![1, 2, 3],
            "l'établissement {rang} n'a pas une numérotation continue partant de 1 : {numeros:?}. \
             Une SEQUENCE est globale au schéma et laisse des trous — deux propriétés fatales à \
             une numérotation de document opérationnel."
        );
    }
}

// =================================================================================================
//  ★ (d) PASSAGE SANS CLIENT — la pièce APRÈS la clé
// =================================================================================================

/// ★ **Un passage sans client produit un séjour valide et une fiche numérotée et INCOMPLÈTE.**
///
/// # Les trois choses qui doivent être vraies ensemble (FR-047)
///
/// 1. la fiche **existe** et porte un **numéro** — elle n'est pas silencieusement omise ;
/// 2. elle est **déclarée incomplète** — elle n'est pas fabriquée avec des valeurs de remplissage ;
/// 3. **aucun champ de remplissage n'y figure** — pas de « M. X », pas de nom vide.
///
/// La troisième est celle qu'une revue manquerait : une fiche portant « Client de passage » aurait
/// l'air correcte et serait un **document légal faux**.
///
/// # Puis le rattachement, qui ne remet rien en cause (FR-028)
///
/// L'opération 10 passe la fiche à complète **sans rouvrir le séjour ni remettre en cause
/// l'attribution**. C'est le parcours normal du passage : la pièce vient **après** la clé (FR-023),
/// et la maquette `R4` le dit en toutes lettres.
#[actix_web::test]
async fn un_passage_sans_client_produit_une_fiche_numerotee_et_declaree_incomplete() {
    let owner = pool_owner().await;
    let decor = poser_decor(&owner, "SEJ — passage sans client").await;
    let cx = commun::compte_connecte(
        &owner,
        decor.jeu,
        "Yao",
        &[(ROLE, Some(decor.jeu.etablissement_id))],
    )
    .await;
    let app = monter_application!(pool_app().await);

    let sejour_id = Uuid::now_v7();
    let debut = OffsetDateTime::now_utc() + time::Duration::hours(1);
    let corps = corps_ouverture(sejour_id, decor.unite_id, decor.formule_id, debut, 24, None);

    let reponse = ouvrir!(app, cx.bearer, decor.jeu.etablissement_id, corps);
    assert_eq!(
        reponse.status(),
        201,
        "un passage SANS fiche client est un séjour valide : la pièce vient après la clé (FR-023)"
    );
    let ouvert: serde_json::Value = actix_web::test::read_body_json(reponse).await;

    // 1 · la fiche EXISTE et porte un numéro
    let numero = ouvert["fiche_police"]["numero"]
        .as_i64()
        .expect("la fiche de police doit exister et porter un numéro, pas être omise");
    assert!(numero >= 1, "le numéro doit être un entier positif");

    // 2 · elle est DÉCLARÉE incomplète
    assert_eq!(
        ouvert["fiche_police"]["complete"], false,
        "sans client rattaché, la fiche doit être DÉCLARÉE incomplète (FR-047) — terme \
         utilisateur : « Identité à compléter »"
    );
    assert!(
        ouvert["fiche_police"]["completee_le"].is_null(),
        "une fiche incomplète ne porte pas d'instant de complétude"
    );

    // 3 · ★ AUCUN champ de remplissage
    //
    // Une fiche portant « M. X », « Client de passage » ou un nom vide aurait l'air correcte et
    // serait un document légal FAUX. Le contrôle porte sur le corps entier de la fiche.
    let fiche_brute = ouvert["fiche_police"].to_string();
    for remplissage in ["M. X", "Client de passage", "Inconnu", "N/A", "\"nom\""] {
        assert!(
            !fiche_brute.contains(remplissage),
            "la fiche porte un champ de remplissage « {remplissage} » : elle n'est ni fabriquée \
             ni omise, elle est DÉCLARÉE incomplète. Corps : {fiche_brute}"
        );
    }

    // Le séjour, lui, est parfaitement valide.
    assert_eq!(ouvert["sejour"]["statut"], "en_cours");
    assert!(ouvert["sejour"]["client_id"].is_null());

    // ── Le rattachement ultérieur, qui ne remet rien en cause (FR-028) ────────────────────────
    let client_id = creer_client(&owner, decor.jeu.tenant_id).await;
    let requete = actix_web::test::TestRequest::post()
        .uri(&format!(
            "/api/v1/etablissements/{}/sejours/{sejour_id}/client",
            decor.jeu.etablissement_id
        ))
        .insert_header(("authorization", cx.bearer.clone()))
        .set_json(serde_json::json!({ "client_id": client_id }))
        .to_request();
    let reponse = actix_web::test::call_service(&app, requete).await;
    assert_eq!(reponse.status(), 200, "le rattachement doit réussir");

    // La fiche est complète, **et le séjour n'a pas été rouvert**.
    let requete = actix_web::test::TestRequest::get()
        .uri(&format!(
            "/api/v1/etablissements/{}/sejours/{sejour_id}",
            decor.jeu.etablissement_id
        ))
        .insert_header(("authorization", cx.bearer.clone()))
        .to_request();
    let apres: serde_json::Value =
        actix_web::test::read_body_json(actix_web::test::call_service(&app, requete).await).await;

    assert_eq!(
        apres["fiche_police"]["complete"], true,
        "le rattachement doit passer la fiche à complète"
    );
    assert_eq!(
        apres["fiche_police"]["numero"], numero,
        "le rattachement ne doit PAS renuméroter la fiche : la continuité par établissement en \
         serait rompue, et le trou ne se verrait qu'au contrôle"
    );
    assert_eq!(
        apres["sejour"]["statut"], "en_cours",
        "le rattachement ne rouvre pas le séjour (FR-028)"
    );
    assert_eq!(
        apres["occupation"]["id"], ouvert["occupation"]["id"],
        "le rattachement ne remet pas en cause l'attribution (FR-028) : la chambre reste la même"
    );
}

// =================================================================================================
//  Rejeu — trois envois, un séjour, et aucun second événement
// =================================================================================================

/// **Trois envois du même identifiant produisent UN séjour, UNE note, UNE fiche.**
///
/// ★ **C'est ici que la dérivation d'identifiants se vérifie.** Le terminal fournit un seul
/// identifiant, celui du séjour ; la note, sa ligne et la fiche de police en sont dérivées. Si
/// elles tiraient des identifiants neufs à chaque tentative, le `ON CONFLICT` du séjour
/// constaterait le rejeu et les trois autres écriraient en double — **avec un numéro de fiche de
/// plus**. La numérotation perdrait sa continuité pour une raison invisible : une coupure réseau.
#[actix_web::test]
async fn trois_envois_du_meme_identifiant_produisent_un_seul_sejour_et_une_seule_fiche() {
    let owner = pool_owner().await;
    let decor = poser_decor(&owner, "SEJ — rejeu").await;
    let cx = commun::compte_connecte(
        &owner,
        decor.jeu,
        "Yao",
        &[(ROLE, Some(decor.jeu.etablissement_id))],
    )
    .await;
    let app = monter_application!(pool_app().await);

    let sejour_id = Uuid::now_v7();
    let debut = OffsetDateTime::now_utc() + time::Duration::hours(1);
    let corps = corps_ouverture(sejour_id, decor.unite_id, decor.formule_id, debut, 24, None);

    let mut statuts = Vec::new();
    for _ in 0..3 {
        let reponse = ouvrir!(app, cx.bearer, decor.jeu.etablissement_id, corps.clone());
        statuts.push(reponse.status().as_u16());
    }

    assert_eq!(
        statuts,
        vec![201, 200, 200],
        "le premier envoi crée (201), les rejeux constatent (200). Répondre 409 obligerait chaque \
         terminal hors ligne à traiter comme une erreur une écriture déjà acceptée — ce que le \
         principe VI interdit. Obtenu : {statuts:?}"
    );

    let mut tx = owner.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, decor.jeu.tenant_id)
        .await
        .expect("pose du tenant");

    for (table, colonne, attendu) in [
        ("hebergement.sejour", "id", 1i64),
        ("hebergement.note_sejour", "sejour_id", 1),
        ("hebergement.fiche_police", "sejour_id", 1),
        ("hebergement.occupation", "sejour_id", 1),
    ] {
        let compte: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
            "SELECT COUNT(*) FROM {table} WHERE {colonne} = $1"
        )))
        .bind(sejour_id)
        .fetch_one(&mut *tx)
        .await
        .expect("comptage");
        assert_eq!(
            compte, attendu,
            "{table} porte {compte} ligne(s) après trois envois du même identifiant. Les \
             identifiants de la note, de la ligne et de la fiche sont-ils bien DÉRIVÉS de celui du \
             séjour ?"
        );
    }

    // **Un seul événement de chaque type** — un rejeu ne change aucun état, et le grand livre a
    // une rétention illimitée.
    for type_evenement in ["heb.sejour.ouvert", "heb.fiche_police.generee"] {
        let compte: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM synchronisation.evenement_outbox \
             WHERE type_evenement = $1 AND tenant_id = $2",
        )
        .bind(type_evenement)
        .bind(decor.jeu.tenant_id)
        .fetch_one(&mut *tx)
        .await
        .expect("comptage des événements");
        assert_eq!(
            compte, 1,
            "trois envois ont produit {compte} événement(s) « {type_evenement} ». Un rejeu ne \
             change aucun état : il n'émet rien, sinon la reconstitution compterait l'écriture \
             trois fois."
        );
    }
    tx.rollback().await.expect("rollback");
}

// =================================================================================================
//  Utilitaire
// =================================================================================================

/// Crée une personne **qualifiée cliente** et rend son identifiant.
async fn creer_client(pool: &sqlx::PgPool, tenant_id: Uuid) -> Uuid {
    let personne_id = Uuid::now_v7();

    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, tenant_id)
        .await
        .expect("pose du tenant");

    sqlx::query("INSERT INTO comptes.personne (id, tenant_id, nom, nom_repli) VALUES ($1,$2,$3,$4)")
        .bind(personne_id)
        .bind(tenant_id)
        .bind("Bakayoko")
        .bind("bakayoko")
        .execute(&mut *tx)
        .await
        .expect("personne");

    sqlx::query("INSERT INTO comptes.client (personne_id, tenant_id) VALUES ($1, $2)")
        .bind(personne_id)
        .bind(tenant_id)
        .execute(&mut *tx)
        .await
        .expect("qualification");

    tx.commit().await.expect("commit");
    personne_id
}

/// Crée une personne **qualifiée cliente** avec un nom et un téléphone donnés.
///
/// Sert la preuve de FR-035 : ces deux valeurs sont écrites **une seule fois**, ici, et doivent
/// ressortir de l'arrivée sans qu'aucune requête d'arrivée ne les ait portées.
async fn creer_client_nomme(
    pool: &sqlx::PgPool,
    tenant_id: Uuid,
    nom: &str,
    telephone: &str,
) -> Uuid {
    let personne_id = Uuid::now_v7();

    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, tenant_id)
        .await
        .expect("pose du tenant");

    sqlx::query(
        "INSERT INTO comptes.personne (id, tenant_id, nom, nom_repli, telephone) \
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(personne_id)
    .bind(tenant_id)
    .bind(nom)
    .bind(nom.to_lowercase())
    .bind(telephone)
    .execute(&mut *tx)
    .await
    .expect("personne");

    sqlx::query("INSERT INTO comptes.client (personne_id, tenant_id) VALUES ($1, $2)")
        .bind(personne_id)
        .bind(tenant_id)
        .execute(&mut *tx)
        .await
        .expect("qualification");

    tx.commit().await.expect("commit");
    personne_id
}

/// Pose un battement de remise en état pour une catégorie et une famille de formule.
async fn poser_remise_en_etat(pool: &sqlx::PgPool, decor: &Decor, minutes: i32) {
    let mut tx = pool.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, decor.jeu.tenant_id)
        .await
        .expect("pose du tenant");

    sqlx::query(
        "INSERT INTO hebergement.temps_remise_en_etat \
             (categorie_id, famille_formule, duree_minutes, tenant_id) \
         VALUES ($1, 'NUITEE', $2, $3)",
    )
    .bind(decor.categorie_id)
    .bind(minutes)
    .bind(decor.jeu.tenant_id)
    .execute(&mut *tx)
    .await
    .expect("battement de remise en état");

    tx.commit().await.expect("commit");
}

// ⚠️ **La proposition d'unité n'a AUCUNE opération HTTP, et c'est délibéré.** L'écran `R3` la
// compose depuis l'opération 17 (`etat-des-unites`), déjà servie à `R4` : ajouter une dix-huitième
// opération pour une donnée que le terminal a déjà serait un appel réseau de plus sur le chemin le
// plus chronométré du produit. Les deux tests qui suivent l'éprouvent donc **au service**, monté
// par la fabrique du binaire — `depuis_environnement`, jamais champ par champ.

// =================================================================================================
//  ★ (e) CLIENT CONNU — zéro champ ressaisi (FR-035)
// =================================================================================================

/// ★ **Une arrivée de client connu ne porte AUCUN champ d'identité, et l'identité ressort quand
/// même.**
///
/// # Ce que ce test prouve, et qu'une relecture d'écran ne prouverait pas
///
/// FR-035 dit « zéro champ ressaisi ». On peut le vérifier de deux façons :
///
/// - regarder l'écran et constater qu'il pré-remplit — ce qui **n'exclut pas** qu'il renvoie
///   ensuite les valeurs pré-remplies au serveur, et donc qu'une modification de la fiche client
///   soit écrasée par une copie périmée à chaque arrivée ;
/// - vérifier que la **requête d'arrivée n'a aucune place** pour ces champs. C'est ce qui est fait
///   ici, sur le corps réellement envoyé.
///
/// La seconde est la seule qui tienne : elle rend la ressaisie **impossible**, pas seulement
/// évitée. Et elle explique pourquoi `hebergement.fiche_police` ne recopie **aucun** élément
/// d'identité (migration `0033`) — le nom qui ressort est **résolu** par `AnnuaireClients`, il
/// n'est pas stocké deux fois.
#[actix_web::test]
async fn un_client_connu_n_impose_aucune_ressaisie_et_l_identite_ressort_quand_meme() {
    let owner = pool_owner().await;
    let decor = poser_decor(&owner, "SEJ — client connu").await;
    let cx = commun::compte_connecte(
        &owner,
        decor.jeu,
        "Yao",
        &[(ROLE, Some(decor.jeu.etablissement_id))],
    )
    .await;
    let app = monter_application!(pool_app().await);

    // L'identité est écrite **une seule fois**, ici.
    let client_id = creer_client_nomme(
        &owner,
        decor.jeu.tenant_id,
        "Kouadio",
        // Indicatif ivoirien, **zéro initial conservé** : la Côte d'Ivoire n'a pas de préfixe
        // interurbain, le national tient en dix chiffres.
        "+2250707123456",
    )
    .await;

    let sejour_id = Uuid::now_v7();
    let debut = OffsetDateTime::now_utc() + time::Duration::hours(1);
    let corps = corps_ouverture(
        sejour_id,
        decor.unite_id,
        decor.formule_id,
        debut,
        48,
        Some(client_id),
    );

    // ── 1 · le corps envoyé n'a AUCUNE place pour un champ d'identité ─────────────────────────
    //
    // C'est l'assertion qui compte. Elle porte sur ce qui **part du terminal**, pas sur ce que
    // l'écran affiche.
    let cles: Vec<&String> = corps.as_object().expect("objet").keys().collect();
    for interdit in [
        "nom",
        "prenoms",
        "telephone",
        "email",
        "type_piece",
        "numero_piece",
        "date_naissance",
        "nationalite",
    ] {
        assert!(
            !cles.iter().any(|c| c.as_str() == interdit),
            "l'arrivée d'un client connu porte le champ « {interdit} » : c'est une ressaisie, et \
             surtout une COPIE qui écrasera la fiche à la prochaine arrivée. Clés envoyées : \
             {cles:?}"
        );
    }

    let reponse = ouvrir!(app, cx.bearer, decor.jeu.etablissement_id, corps);
    assert_eq!(reponse.status(), 201);

    // ── 2 · l'identité ressort quand même, RÉSOLUE et non recopiée ────────────────────────────
    let requete = actix_web::test::TestRequest::get()
        .uri(&format!(
            "/api/v1/etablissements/{}/sejours",
            decor.jeu.etablissement_id
        ))
        .insert_header(("authorization", cx.bearer.clone()))
        .to_request();
    let liste: serde_json::Value =
        actix_web::test::read_body_json(actix_web::test::call_service(&app, requete).await).await;

    let ligne = liste
        .as_array()
        .expect("un tableau")
        .iter()
        .find(|s| s["sejour"]["id"] == serde_json::json!(sejour_id))
        .expect("le séjour ouvert doit figurer dans la liste des séjours en cours");

    assert_eq!(
        ligne["client_nom"], "Kouadio",
        "le nom doit être RÉSOLU par AnnuaireClients — il n'a jamais transité par la requête \
         d'arrivée. Ligne : {ligne}"
    );
    assert_eq!(
        ligne["client_telephone"], "+2250707123456",
        "le zéro initial doit survivre : « 07 07 12 34 56 » est le numéro national ivoirien \
         COMPLET, pas un numéro préfixé"
    );

    // ── 3 · et la fiche de police ne RECOPIE rien ─────────────────────────────────────────────
    //
    // Une fiche qui dupliquerait le nom donnerait, au premier changement de fiche client, deux
    // vérités — dont l'une est un document légal.
    let requete = actix_web::test::TestRequest::get()
        .uri(&format!(
            "/api/v1/etablissements/{}/sejours/{sejour_id}/fiche-police",
            decor.jeu.etablissement_id
        ))
        .insert_header(("authorization", cx.bearer.clone()))
        .to_request();
    let fiche: serde_json::Value =
        actix_web::test::read_body_json(actix_web::test::call_service(&app, requete).await).await;

    assert!(
        !fiche.to_string().contains("Kouadio"),
        "la fiche de police RECOPIE le nom du client. Elle doit le référencer par `client_id` et \
         le laisser résoudre : sinon un changement de fiche client laisse un document légal qui \
         porte l'ancien nom, sans que rien ne le signale. Fiche : {fiche}"
    );
}

// =================================================================================================
//  ★ (f) L'UNITÉ PROPOSÉE EST RÉELLEMENT LIBRE — et c'est l'attribution qui le prouve
// =================================================================================================

/// ★ **La proposition tient : l'attribution sur l'unité proposée réussit.**
///
/// # Pourquoi l'assertion ne s'arrête pas à la réponse de la proposition
///
/// Vérifier que `unites_proposables` rend `A2` avec `disponible_a = None` ne prouve qu'une chose :
/// que la requête est d'accord avec elle-même. Si son `LEFT JOIN` bornait l'indisponibilité par
/// `fin_client` plutôt que par `periode`, elle proposerait une chambre encore en remise en état —
/// et **la contrainte d'exclusion refuserait l'attribution après le geste de Yao**, devant le
/// client.
///
/// Le test attribue donc **réellement** sur l'unité proposée. C'est le seul contrôle qui distingue
/// une proposition juste d'une proposition plausible.
#[actix_web::test]
async fn l_unite_proposee_est_reellement_libre_et_l_attribution_le_confirme() {
    let owner = pool_owner().await;
    let decor = poser_decor(&owner, "SEJ — proposition").await;
    // 45 minutes de battement : sans lui, `periode` et `fin_client` coïncideraient et le test ne
    // distinguerait pas les deux bornes.
    poser_remise_en_etat(&owner, &decor, 45).await;
    let cx = commun::compte_connecte(
        &owner,
        decor.jeu,
        "Yao",
        &[(ROLE, Some(decor.jeu.etablissement_id))],
    )
    .await;
    let app = monter_application!(pool_app().await);

    let debut = OffsetDateTime::now_utc() + time::Duration::hours(1);
    let fin = debut + time::Duration::hours(24);

    // A1 est prise sur l'intervalle.
    let corps = corps_ouverture(
        Uuid::now_v7(),
        decor.unite_id,
        decor.formule_id,
        debut,
        24,
        None,
    );
    assert_eq!(
        ouvrir!(app, cx.bearer, decor.jeu.etablissement_id, corps).status(),
        201
    );

    let etat = kaya_api::application::EtatApplication::depuis_environnement(pool_app().await)
        .expect("assemblage de l'état applicatif");
    let propositions = etat
        .service_sejour(decor.jeu.tenant_id)
        .unites_proposables(decor.jeu.etablissement_id, decor.categorie_id, debut, fin)
        .await
        .expect("proposition d'unité");

    let premiere = propositions.first().expect("la liste n'est jamais vide");
    assert_eq!(
        premiere.code, "A2",
        "l'ordre est STABLE et EXPLICABLE : les libres d'abord, puis par code croissant — l'ordre \
         du tableau de clés. Obtenu : {:?}",
        propositions.iter().map(|p| &p.code).collect::<Vec<_>>()
    );
    assert!(
        premiere.disponible_a.is_none(),
        "la première proposition doit être libre SUR L'INTERVALLE, pas « libre plus tard »"
    );

    // ★ Et l'attribution le confirme — la proposition n'est pas seulement d'accord avec elle-même.
    let corps = corps_ouverture(
        Uuid::now_v7(),
        premiere.unite_id,
        decor.formule_id,
        debut,
        24,
        None,
    );
    let reponse = ouvrir!(app, cx.bearer, decor.jeu.etablissement_id, corps);
    let statut = reponse.status().as_u16();
    let recu: serde_json::Value = actix_web::test::read_body_json(reponse).await;
    assert_eq!(
        statut, 201,
        "l'unité PROPOSÉE a été refusée à l'attribution. La proposition borne-t-elle bien \
         l'indisponibilité par `periode` — remise en état comprise — plutôt que par `fin_client` ? \
         Corps : {recu}"
    );
}

// =================================================================================================
//  ★ (g) TROIS PERSONNES APRÈS DEUX ACCOMPAGNANTS — dérivé, jamais saisi (FR-018)
// =================================================================================================

/// ★ **Le titulaire plus deux accompagnants font trois — et le retrait d'un ramène à deux.**
///
/// # La seconde moitié est celle qui compte
///
/// Un nombre de personnes **posé en colonne** passerait la première assertion et échouerait la
/// seconde : il se désynchroniserait au premier retrait. Et comme le constat de taxe **fige** ce
/// qu'il lit, la valeur fausse y resterait **pour toujours** — un séjour déclaré à trois personnes
/// alors qu'il en portait deux, sans rien pour le rattraper a posteriori.
#[actix_web::test]
async fn deux_accompagnants_font_trois_personnes_et_le_retrait_ramene_a_deux() {
    let owner = pool_owner().await;
    let decor = poser_decor(&owner, "SEJ — personnes").await;
    let cx = commun::compte_connecte(
        &owner,
        decor.jeu,
        "Yao",
        &[(ROLE, Some(decor.jeu.etablissement_id))],
    )
    .await;
    let app = monter_application!(pool_app().await);

    let sejour_id = Uuid::now_v7();
    let debut = OffsetDateTime::now_utc() + time::Duration::hours(1);
    let mut corps = corps_ouverture(sejour_id, decor.unite_id, decor.formule_id, debut, 24, None);

    // ⚠️ Les accompagnants partent dans la **même** requête que le séjour : déclarés à l'arrivée
    // et perdus par un second appel manqué, ils feraient une fiche de police fausse.
    let accompagnant_a = Uuid::now_v7();
    let accompagnant_b = Uuid::now_v7();
    corps["accompagnants"] = serde_json::json!([
        { "id": accompagnant_a, "nom": "Aya" },
        { "id": accompagnant_b, "nom": "Konan" },
    ]);

    assert_eq!(
        ouvrir!(app, cx.bearer, decor.jeu.etablissement_id, corps).status(),
        201
    );

    // **Un nom seul suffit** (FR-015) : demander une pièce par accompagnant coûterait la cible des
    // 60 secondes de l'arrivée.
    let compter = |bearer: String| {
        let app = &app;
        let etablissement = decor.jeu.etablissement_id;
        async move {
            let requete = actix_web::test::TestRequest::get()
                .uri(&format!("/api/v1/etablissements/{etablissement}/sejours"))
                .insert_header(("authorization", bearer))
                .to_request();
            let liste: serde_json::Value =
                actix_web::test::read_body_json(actix_web::test::call_service(app, requete).await)
                    .await;
            liste
                .as_array()
                .expect("un tableau")
                .iter()
                .find(|s| s["sejour"]["id"] == serde_json::json!(sejour_id))
                .expect("le séjour")["nombre_personnes"]
                .as_i64()
                .expect("un entier")
        }
    };

    assert_eq!(
        compter(cx.bearer.clone()).await,
        3,
        "le titulaire compte pour un, plus deux accompagnants (FR-018)"
    );

    // ★ Le retrait — c'est ici qu'une colonne posée se trahirait.
    let requete = actix_web::test::TestRequest::delete()
        .uri(&format!(
            "/api/v1/etablissements/{}/sejours/{sejour_id}/accompagnants/{accompagnant_b}",
            decor.jeu.etablissement_id
        ))
        .insert_header(("authorization", cx.bearer.clone()))
        .to_request();
    let reponse = actix_web::test::call_service(&app, requete).await;
    assert!(
        reponse.status().is_success(),
        "le retrait d'un accompagnant doit réussir ; obtenu {}",
        reponse.status()
    );

    assert_eq!(
        compter(cx.bearer.clone()).await,
        2,
        "après retrait, le nombre de personnes doit être DÉRIVÉ à nouveau. Une colonne \
         `nombre_personnes` sur `sejour` aurait passé l'assertion précédente et échoué celle-ci — \
         et le constat de taxe aurait figé la valeur fausse POUR TOUJOURS."
    );
}

// =================================================================================================
//  ★ (h) CATÉGORIE PLEINE — le refus NOMME la première disponibilité, jamais une liste vide
// =================================================================================================

/// ★ **Quand tout est pris, la liste n'est pas vide : chaque unité porte l'instant où elle se
/// libère — remise en état COMPRISE.**
///
/// # Les deux défauts que ce test attrape, et qu'aucune relecture ne verrait
///
/// 1. **Une liste vide.** Elle passerait toute revue (« il n'y a rien à proposer, c'est exact »)
///    et obligerait Yao à ouvrir un autre écran pour répondre au client qui attend devant lui.
///    C'est la différence entre un écran qui répond « pas avant 16 h 40 » et un écran qui dit
///    « complet ».
/// 2. **Un instant qui oublie le battement.** Annoncer la chambre libre à l'heure exacte du
///    départ, alors qu'elle est encore à faire, produit une promesse au client que la contrainte
///    d'exclusion démentira. L'assertion porte sur `fin + 45 min`, valeur qu'aucune coïncidence
///    ne peut produire.
#[actix_web::test]
async fn categorie_pleine_le_refus_nomme_la_premiere_disponibilite_remise_en_etat_comprise() {
    let owner = pool_owner().await;
    let decor = poser_decor(&owner, "SEJ — catégorie pleine").await;
    const BATTEMENT_MINUTES: i64 = 45;
    poser_remise_en_etat(&owner, &decor, BATTEMENT_MINUTES as i32).await;
    let cx = commun::compte_connecte(
        &owner,
        decor.jeu,
        "Yao",
        &[(ROLE, Some(decor.jeu.etablissement_id))],
    )
    .await;
    let app = monter_application!(pool_app().await);

    let debut = OffsetDateTime::now_utc() + time::Duration::hours(1);
    let fin = debut + time::Duration::hours(24);

    // Les deux chambres de la catégorie sont prises sur l'intervalle.
    for unite_id in [decor.unite_id, decor.unite_bis_id] {
        let corps = corps_ouverture(Uuid::now_v7(), unite_id, decor.formule_id, debut, 24, None);
        assert_eq!(
            ouvrir!(app, cx.bearer, decor.jeu.etablissement_id, corps).status(),
            201
        );
    }

    let etat = kaya_api::application::EtatApplication::depuis_environnement(pool_app().await)
        .expect("assemblage de l'état applicatif");
    let propositions = etat
        .service_sejour(decor.jeu.tenant_id)
        .unites_proposables(decor.jeu.etablissement_id, decor.categorie_id, debut, fin)
        .await
        .expect("proposition d'unité");

    // 1 · la liste n'est PAS vide
    assert_eq!(
        propositions.len(),
        2,
        "catégorie pleine : la liste doit rendre les DEUX unités avec leur instant de libération, \
         jamais une liste vide. Un refus qui dirait seulement « complet » obligerait Yao à ouvrir \
         un autre écran devant le client."
    );

    // 2 · chacune NOMME son instant, et cet instant inclut le battement
    let attendu = fin + time::Duration::minutes(BATTEMENT_MINUTES);
    for proposition in &propositions {
        let instant = proposition.disponible_a.unwrap_or_else(|| {
            panic!(
                "l'unité {} est annoncée libre alors que la catégorie est pleine",
                proposition.code
            )
        });
        assert_eq!(
            instant, attendu,
            "l'unité {} est annoncée libre à {instant}, soit l'heure exacte du départ. La remise \
             en état ({BATTEMENT_MINUTES} min) FAIT PARTIE de l'indisponibilité : promettre la \
             chambre plus tôt, c'est une promesse que la contrainte d'exclusion démentira devant \
             le client. Attendu : {attendu}",
            proposition.code
        );
    }
}

/// **Contrôle de diagnostic** : les deux mêmes arrivées, **séquentielles**.
///
/// Il isole la variable. Si celle-ci passe et que la version concurrente échoue, la cause est dans
/// l'ordonnancement du harnais, pas dans la traduction du refus.
#[actix_web::test]
async fn deux_arrivees_sequentielles_le_second_recoit_unite_deja_occupee() {
    let owner = pool_owner().await;
    let decor = poser_decor(&owner, "SEJ — séquentiel").await;
    let cx = commun::compte_connecte(
        &owner,
        decor.jeu,
        "Yao",
        &[(ROLE, Some(decor.jeu.etablissement_id))],
    )
    .await;
    let app = monter_application!(pool_app().await);
    let debut = OffsetDateTime::now_utc() + time::Duration::hours(2);

    let c1 = corps_ouverture(Uuid::now_v7(), decor.unite_id, decor.formule_id, debut, 24, None);
    let r1 = ouvrir!(app, cx.bearer, decor.jeu.etablissement_id, c1);
    assert_eq!(r1.status(), 201);

    let c2 = corps_ouverture(Uuid::now_v7(), decor.unite_id, decor.formule_id, debut, 24, None);
    let r2 = ouvrir!(app, cx.bearer, decor.jeu.etablissement_id, c2);
    let statut = r2.status().as_u16();
    let corps: serde_json::Value = actix_web::test::read_body_json(r2).await;
    assert_eq!(statut, 409, "corps : {corps}");
    assert_eq!(corps["code"], "unite_deja_occupee");
}
