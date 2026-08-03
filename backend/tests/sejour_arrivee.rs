//! ★ **SEJ-02** — l'ouverture d'un séjour : une transaction, la concurrence, et P-09 ré-exercée.
//!
//! # Les quatre garanties que ce fichier éprouve, et qui ne se voient PAS en relecture
//!
//! | # | Ce qui est vérifié | Ce qu'une relecture manquerait |
//! |---|---|---|
//! | **a** | Une panne après l'attribution ne laisse **rien** — ni séjour, ni note, ni fiche, **ni occupation orpheline** | Cinq écritures qui « ont l'air » dans une transaction |
//! | **b** | Deux arrivées chevauchantes : **exactement une** réussit, et le refus est un `ExclusionViolation` **sur la contrainte nommée** | Un `SELECT … FOR UPDATE` donnerait le même COMPTE en rendant la double attribution *improbable* au lieu d'*impossible* |
//! | **c** | La numérotation de fiche de police est **continue par établissement**, sans trou | Une `SEQUENCE` passerait le test sur un seul établissement |
//! | **d** | Un passage **sans client** produit un séjour valide et une fiche **numérotée et déclarée incomplète**, sans champ de remplissage | Une fiche fabriquée avec « M. X » passerait toute revue |
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
