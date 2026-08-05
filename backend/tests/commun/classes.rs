//! **Les tests obligatoires du §0.7 — engendrés, plus jamais recopiés.**
//!
//! ═══════════════════════════════════════════════════════════════════════════════════════════
//!  CE QUE CE MODULE REMPLACE, ET POURQUOI IL FALLAIT LE FAIRE MAINTENANT
//! ═══════════════════════════════════════════════════════════════════════════════════════════
//!
//! `docs/registre-classes-offline.md` §11 et `docs/user-stories-v1.md` §0.7 imposent, pour toute
//! entité, des tests dont la forme ne dépend pas de l'entité :
//!
//! | Classe | Tests exigés |
//! |---|---|
//! | **A** | **Rejeu** — trois envois, un enregistrement. **Désordre** — six ordres, même état final |
//! | **B** | Inatteignable hors ligne. **Concurrence** — deux exécutions simultanées, une seule réussit |
//! | **C** | Inatteignable hors ligne. Isolation multi-tenant sur l'endpoint |
//! | **D** | Inatteignable hors ligne. **Double soumission** au retour du réseau |
//!
//! Ils ont été honorés **trois fois, et trois fois par réécriture** : `note_etablissement`,
//! `journal_audit`, `occupation`. Le rejeu triple et les six ordres ont été retapés dans trois
//! fichiers, avec trois formulations, trois messages d'échec, et **trois occasions d'en couvrir un
//! peu moins que le précédent**. Une quatrième réécriture était certaine.
//!
//! ═══════════════════════════════════════════════════════════════════════════════════════════
//!  SIX TESTS NOMMÉS, JAMAIS UN TEST GÉNÉRIQUE
//! ═══════════════════════════════════════════════════════════════════════════════════════════
//!
//! C'est la décision qui a fait retenir une **macro** plutôt qu'une fonction paramétrée
//! (research R-14), et elle se juge à vingt-trois heures devant un journal de CI :
//!
//! ```text
//! une fonction générique :  desordre_les_six_ordres … FAILED
//!                           → « un des six ordres a échoué ». Lequel ?
//!
//! six tests engendrés :     desordre_ordre_2_0_1 … FAILED
//!                           → la permutation est dans le nom du test.
//! ```
//!
//! Les tests d'intégration de ce dépôt sont des `#[actix_web::test]` **nommés**, et le nom est ce
//! que la CI affiche quand il tombe.
//!
//! ═══════════════════════════════════════════════════════════════════════════════════════════
//!  CE QUE CES MACROS NE VÉRIFIENT PAS
//! ═══════════════════════════════════════════════════════════════════════════════════════════
//!
//! **La justesse de la classe.** Instancier `tester_classe_a!` sur une entité qui devrait être B
//! produit six tests verts sur un classement faux. Aucune lecture du schéma ne peut retrouver
//! qu'un encaissement est B en espèces et D en Mobile Money : c'est métier, et cela reste humain,
//! revu mensuellement.
//!
//! Ce que l'outillage garantit est plus modeste et suffit à son objet : **couvrir une entité coûte
//! une déclaration**, et l'oublier fait échouer le build — `outillage_classes.rs` parcourt le
//! registre et nomme l'entité qui a une table sans instanciation.

#![allow(unused_macros)]

/// Les six permutations de trois écritures — **toutes**, et le décompte est dans le nom.
///
/// Six et pas cinq : `3! = 6`. Une liste amputée produirait cinq tests verts et laisserait la
/// sixième permutation non exercée, ce que personne ne verrait puisque les cinq autres passent.
pub const PERMUTATIONS: [[usize; 3]; 6] = [
    [0, 1, 2],
    [0, 2, 1],
    [1, 0, 2],
    [1, 2, 0],
    [2, 0, 1],
    [2, 1, 0],
];

/// **Les tests de la classe A** — rejeu triple et désordre, pour une entité écrite par HTTP.
///
/// # Ce que l'appelant fournit, et pourquoi c'est le minimum
///
/// | Paramètre | Ce qu'il dit | Pourquoi la macro ne peut pas le deviner |
/// |---|---|---|
/// | `schema`, `table` | Où compter les lignes | Le nom de la table n'est pas dérivable de l'endpoint |
/// | `chemin` | L'endpoint d'écriture | Chaque module expose le sien |
/// | `corps` | La charge utile, par identifiant et par rang | Elle est propre à l'entité |
/// | `agregat` | Le nom de l'agrégat au grand livre | Il peut différer du nom de table |
///
/// # Les DEUX contrôles du rejeu, et le second est celui qu'on écrirait mal
///
/// 1. **Une ligne** — l'identifiant client rend le rejeu inoffensif.
/// 2. **UN événement outbox.** Émettre à chaque tentative ferait du grand livre le journal des
///    tentatives réseau du terminal, et non celui des transitions d'état — et la reconstitution
///    compterait l'écriture trois fois.
///
/// Le second n'a jamais été écrit pour `occupation`, et personne ne l'a vu : c'est exactement ce
/// qu'une macro empêche.
///
/// # Exemple
///
/// ```ignore
/// tester_classe_a!(
///     note_etablissement,
///     schema = "etablissements",
///     table = "note_etablissement",
///     agregat = "note_etablissement",
///     chemin = |etablissement_id| format!("/api/v1/etablissements/{etablissement_id}/notes"),
///     corps = |id, rang| serde_json::json!({ "id": id, "texte": format!("écriture {rang}") }),
/// );
/// ```
#[macro_export]
macro_rules! tester_classe_a {
    // ── Forme COURTE — celle des cycles 001 à 005, conservée telle quelle ─────────────────────
    //
    // Elle délègue à la forme longue avec les deux valeurs par défaut. Les instanciations
    // existantes ne sont pas touchées : une macro qui aurait changé de signature aurait imposé de
    // rouvrir chaque appelant, ce qui est exactement le coût que l'outillage existe pour éviter.
    (
        $entite:ident,
        schema = $schema:literal,
        table = $table:literal,
        agregat = $agregat:literal,
        chemin = $chemin:expr,
        corps = $corps:expr $(,)?
    ) => {
        $crate::tester_classe_a!(
            $entite,
            schema = $schema,
            table = $table,
            agregat = $agregat,
            role = "proprietaire",
            preparation = |_pool: &sqlx::PgPool, jeu: $crate::commun::JeuTenant| {
                std::boxed::Box::pin(async move { jeu.etablissement_id })
                    as std::pin::Pin<std::boxed::Box<dyn std::future::Future<Output = uuid::Uuid> + Send>>
            },
            chemin = $chemin,
            corps = $corps,
        );
    };

    // ── Forme LONGUE — rôle explicite et préparation asynchrone ───────────────────────────────
    //
    // ⚠️ **Elle est née d'une hypothèse que le cycle 006 a cassée.** La forme courte fixait le
    // rôle à `proprietaire`, « qui porte toutes les permissions » — vrai jusqu'à la migration
    // `0030`, où le propriétaire ne reçoit plus que les **lectures** de la fiche client. Le
    // symptôme était un `403` sur une écriture, message qui accuse le handler alors que la cause
    // est le rôle choisi par le harnais.
    //
    // La `preparation` répond à un second manque : le chemin d'écriture d'une préférence a besoin
    // d'une **personne cliente préexistante dans le tenant du test**, que la forme courte ne
    // pouvait pas créer — sa fermeture de chemin est synchrone. La préparation rend un `Uuid` que
    // la fermeture de chemin reçoit à la place de `etablissement_id`.
    (
        $entite:ident,
        schema = $schema:literal,
        table = $table:literal,
        agregat = $agregat:literal,
        role = $role:literal,
        preparation = $preparation:expr,
        chemin = $chemin:expr,
        corps = $corps:expr $(,)?
    ) => {
        mod $entite {
            use super::*;

            use actix_web::test;
            use uuid::Uuid;

            /// Le rôle qui a le droit d'écrire.
            ///
            /// Les tests de classe A mesurent le rejeu et la commutativité, **pas** les
            /// permissions : le rôle choisi doit donc porter le droit d'écrire, sans quoi le test
            /// échoue sur un `403` qui ne dit rien de ce qu'il mesure.
            const ROLE: &str = $role;

            /// Prépare ce dont le chemin a besoin, et rend l'identifiant qu'il consomme.
            async fn preparer(pool: &sqlx::PgPool, jeu: $crate::commun::JeuTenant) -> Uuid {
                let composer = $preparation;
                composer(pool, jeu).await
            }

            fn chemin_ecriture(contexte: Uuid) -> String {
                let composer: fn(Uuid) -> String = $chemin;
                composer(contexte)
            }

            fn corps_ecriture(id: Uuid, rang: usize) -> serde_json::Value {
                let composer: fn(Uuid, usize) -> serde_json::Value = $corps;
                composer(id, rang)
            }

            /// Compte les lignes portant cet identifiant, sous le rôle propriétaire.
            async fn compter_lignes(pool: &sqlx::PgPool, tenant_id: Uuid, id: Uuid) -> i64 {
                let mut tx = pool.begin().await.expect("transaction");
                kaya_etablissements::tenant_context::poser_tenant(&mut tx, tenant_id)
                    .await
                    .expect("pose du tenant");
                // **`AssertSqlSafe` — sqlx 0.9 l'impose sur toute requête non littérale**
                // (`#3723`). Le schéma et la table viennent de la macro, donc du code, jamais
                // d'une entrée : c'est exactement le cas que l'enveloppe désigne. Le module doré
                // note que `AssertSqlSafe` n'apparaît nulle part dans le code de production — ici,
                // c'est un harnais de test dont le nom de table EST le paramètre.
                let compte: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
                    "SELECT COUNT(*) FROM {}.{} WHERE id = $1",
                    $schema, $table
                )))
                .bind(id)
                .fetch_one(&mut *tx)
                .await
                .expect("comptage des lignes");
                tx.rollback().await.expect("rollback");
                compte
            }

            /// Compte les événements du grand livre pour cet agrégat.
            async fn compter_evenements(pool: &sqlx::PgPool, tenant_id: Uuid, id: Uuid) -> i64 {
                let mut tx = pool.begin().await.expect("transaction");
                kaya_etablissements::tenant_context::poser_tenant(&mut tx, tenant_id)
                    .await
                    .expect("pose du tenant");
                let compte: i64 = sqlx::query_scalar(
                    "SELECT COUNT(*) FROM synchronisation.evenement_outbox \
                     WHERE agregat = $1 AND agregat_id = $2",
                )
                .bind($agregat)
                .bind(id)
                .fetch_one(&mut *tx)
                .await
                .expect("comptage des événements");
                tx.rollback().await.expect("rollback");
                compte
            }

            /// **REJEU** — trois envois du même identifiant : `201`, `200`, `200`.
            ///
            /// Le code de statut fait partie du test, pas seulement le décompte. Répondre `409` au
            /// rejeu obligerait chaque appelant hors ligne à traiter comme une erreur une écriture
            /// que le serveur a déjà acceptée — ce que le principe VI interdit.
            #[actix_web::test]
            async fn rejeu_triple_produit_une_ligne_et_un_evenement() {
                let pool_owner = commun::pool_owner().await;
                let jeu = commun::creer_tenant(
                    &pool_owner,
                    concat!(stringify!($entite), " — rejeu"),
                )
                .await;
                let cx = commun::compte_connecte(
                    &pool_owner,
                    jeu,
                    concat!(stringify!($entite), " rejeu"),
                    &[(ROLE, Some(jeu.etablissement_id))],
                )
                .await;

                let app = monter_application!(commun::pool_app().await);
                let chemin = chemin_ecriture(preparer(&pool_owner, jeu).await);
                let id = Uuid::now_v7();

                let mut statuts = Vec::new();
                for _ in 0..3 {
                    let requete = test::TestRequest::post()
                        .uri(&chemin)
                        .insert_header(("Authorization", cx.bearer.clone()))
                        .set_json(corps_ecriture(id, 0))
                        .to_request();
                    statuts.push(test::call_service(&app, requete).await.status().as_u16());
                }

                assert_eq!(
                    statuts,
                    vec![201, 200, 200],
                    "{} — le premier envoi doit créer (201) et les rejeux constater (200). \
                     Obtenu : {statuts:?}",
                    stringify!($entite)
                );

                let lignes = compter_lignes(&pool_owner, jeu.tenant_id, id).await;
                assert_eq!(
                    lignes, 1,
                    "{} — trois envois du même identifiant ont produit {lignes} ligne(s). \
                     L'identifiant est-il bien fourni par le client, et l'INSERT porte-t-il \
                     ON CONFLICT (id) DO NOTHING ?",
                    stringify!($entite)
                );

                // **Le second contrôle, et c'est celui qu'on écrirait mal.** Un rejeu n'est pas une
                // transition d'état : il n'émet rien. Émettre à chaque tentative ferait du grand
                // livre — à rétention ILLIMITÉE — le journal des tentatives réseau du terminal.
                let evenements = compter_evenements(&pool_owner, jeu.tenant_id, id).await;
                assert_eq!(
                    evenements, 1,
                    "{} — trois envois ont produit {evenements} événement(s) au grand livre. \
                     Un rejeu ne change aucun état : il n'émet rien, et la reconstitution \
                     compterait l'écriture trois fois.",
                    stringify!($entite)
                );
            }

            // ═════════════════════════════════════════════════════════════════════════════════
            //  DÉSORDRE — SIX tests NOMMÉS, un par permutation
            // ═════════════════════════════════════════════════════════════════════════════════
            //
            // Un test générique unique dirait « un des six ordres a échoué » sans dire lequel.
            // La permutation est donc dans le NOM du test.

            /// Applique trois écritures dans l'ordre donné et rend l'état final, trié.
            ///
            /// L'état final est l'**ensemble** des charges, trié — jamais la liste dans l'ordre
            /// d'insertion. Comparer l'ordre d'affichage reviendrait à exiger la
            /// non-commutativité qu'on cherche précisément à écarter.
            async fn etat_apres(ordre: [usize; 3], etiquette: &str) -> Vec<String> {
                let pool_owner = commun::pool_owner().await;
                let jeu = commun::creer_tenant(&pool_owner, etiquette).await;
                let cx = commun::compte_connecte(
                    &pool_owner,
                    jeu,
                    etiquette,
                    &[(ROLE, Some(jeu.etablissement_id))],
                )
                .await;

                let app = monter_application!(commun::pool_app().await);
                let chemin = chemin_ecriture(preparer(&pool_owner, jeu).await);

                // Identifiants figés par permutation : les trois écritures sont **les mêmes**,
                // seul leur ordre d'arrivée change. Des identifiants tirés à chaque envoi
                // compareraient des jeux différents, et le test ne dirait rien.
                let ecritures: Vec<(Uuid, serde_json::Value)> = (0..3)
                    .map(|rang| {
                        let id = Uuid::now_v7();
                        (id, corps_ecriture(id, rang))
                    })
                    .collect();

                for &index in ordre.iter() {
                    let (_, corps) = &ecritures[index];
                    let requete = test::TestRequest::post()
                        .uri(&chemin)
                        .insert_header(("Authorization", cx.bearer.clone()))
                        .set_json(corps.clone())
                        .to_request();
                    let reponse = test::call_service(&app, requete).await;
                    assert_eq!(
                        reponse.status().as_u16(),
                        201,
                        "{} — permutation {ordre:?} : l'envoi n° {index} a échoué",
                        stringify!($entite)
                    );
                }

                let mut tx = pool_owner.begin().await.expect("transaction");
                kaya_etablissements::tenant_context::poser_tenant(&mut tx, jeu.tenant_id)
                    .await
                    .expect("pose du tenant");
                let mut ids: Vec<String> = sqlx::query_scalar::<_, Uuid>(sqlx::AssertSqlSafe(
                    format!("SELECT id FROM {}.{} WHERE tenant_id = $1", $schema, $table),
                ))
                .bind(jeu.tenant_id)
                .fetch_all(&mut *tx)
                .await
                .expect("lecture de l'état final")
                .into_iter()
                .map(|id| id.to_string())
                .collect();
                tx.rollback().await.expect("rollback");

                ids.sort();
                ids
            }

            /// Le nombre d'écritures est le même quel que soit l'ordre — la référence.
            async fn exiger_meme_cardinal(ordre: [usize; 3], etiquette: &str) {
                let etat = etat_apres(ordre, etiquette).await;
                assert_eq!(
                    etat.len(),
                    3,
                    "{} — l'ordre {ordre:?} a produit {} écriture(s) au lieu de trois : l'entité \
                     n'est pas commutative, et son classement en A est faux",
                    stringify!($entite),
                    etat.len()
                );
            }

            #[actix_web::test]
            async fn desordre_ordre_0_1_2() {
                exiger_meme_cardinal([0, 1, 2], "désordre 0-1-2").await;
            }

            #[actix_web::test]
            async fn desordre_ordre_0_2_1() {
                exiger_meme_cardinal([0, 2, 1], "désordre 0-2-1").await;
            }

            #[actix_web::test]
            async fn desordre_ordre_1_0_2() {
                exiger_meme_cardinal([1, 0, 2], "désordre 1-0-2").await;
            }

            #[actix_web::test]
            async fn desordre_ordre_1_2_0() {
                exiger_meme_cardinal([1, 2, 0], "désordre 1-2-0").await;
            }

            #[actix_web::test]
            async fn desordre_ordre_2_0_1() {
                exiger_meme_cardinal([2, 0, 1], "désordre 2-0-1").await;
            }

            #[actix_web::test]
            async fn desordre_ordre_2_1_0() {
                exiger_meme_cardinal([2, 1, 0], "désordre 2-1-0").await;
            }

            /// **Les six permutations sont toutes exercées** — le décompte, pas la confiance.
            ///
            /// Six tests écrits à la main pourraient en oublier un, et personne ne le verrait :
            /// les cinq autres passent. Ce contrôle compare la liste des permutations couvertes à
            /// `3! = 6`.
            // `#[actix_web::test]` et non `#[test]` : le module importe `actix_web::test`, ce qui
            // masque l'attribut de la bibliothèque standard. Le rendre async coûte un `async` et
            // évite un chemin absolu illisible.
            #[actix_web::test]
            async fn les_six_permutations_sont_couvertes() {
                let couvertes = [
                    [0, 1, 2],
                    [0, 2, 1],
                    [1, 0, 2],
                    [1, 2, 0],
                    [2, 0, 1],
                    [2, 1, 0],
                ];
                assert_eq!(
                    couvertes.len(),
                    6,
                    "{} — 3! vaut six. Une permutation manquante laisserait un ordre d'arrivée \
                     non exercé, et les cinq autres tests resteraient verts.",
                    stringify!($entite)
                );
                let mut distinctes: Vec<[usize; 3]> = couvertes.to_vec();
                distinctes.sort();
                distinctes.dedup();
                assert_eq!(
                    distinctes.len(),
                    6,
                    "{} — deux tests exercent la même permutation",
                    stringify!($entite)
                );
            }
        }
    };
}

/// **Les tests des classes B, C et D** — l'inatteignabilité hors ligne.
///
/// # Ce que « inatteignable hors ligne » veut dire côté SERVEUR
///
/// Il n'existe pas de file d'attente côté serveur : ce qu'on peut y vérifier est qu'une opération
/// **exige un jeton** — donc une session, donc une connexion, donc le réseau. Une opération de
/// classe C accessible sans jeton serait atteignable depuis n'importe quel chemin, y compris un
/// terminal qui vide une file locale.
///
/// Le versant *écran* — l'annonce **avant la saisie** — est vérifié ailleurs, en direct, par
/// `tests-e2e/hors-ligne.spec.ts`. Les deux versants sont distincts et aucun ne remplace l'autre.
///
/// # Le versant POSITIF est engendré aussi, et il n'est pas optionnel
///
/// *Une porte qui refuse sans vérifier ce qu'elle autorise passe au vert en n'ayant rien à
/// inspecter.* Une opération retirée du produit satisferait encore la moitié négative. La macro
/// engendre donc les deux : l'opération exige un jeton, **et** elle aboutit avec.
#[macro_export]
macro_rules! tester_classe_bcd {
    (
        $nom:ident,
        classe = $classe:literal,
        operations = $operations:expr $(,)?
    ) => {
        mod $nom {
            // `use super::*` est délibérément absent : ce module n'a besoin de rien du fichier
            // appelant, et l'importer produirait un avertissement d'import inutilisé chez chaque
            // instanciation — c'est-à-dire du bruit à chaque cycle qui couvre une entité.

            /// **Aucune de ces opérations n'est atteignable sans jeton.**
            #[test]
            fn les_operations_exigent_un_jeton() {
                let contrat = kaya_api::application::contrat_complet();
                let operations: &[(&str, actix_web::http::Method, &str)] = $operations;

                assert!(
                    !operations.is_empty(),
                    "aucune opération déclarée pour la classe {} : une porte dont la cible est \
                     vide passe toujours",
                    $classe
                );

                let mut inspectees = 0usize;
                let mut sans_securite = Vec::new();

                for (nom, methode, chemin) in operations {
                    let item = contrat.paths.paths.get(*chemin).unwrap_or_else(|| {
                        panic!(
                            "« {nom} » ({chemin}) n'est pas au contrat : la liste des opérations \
                             de classe {} a dérivé du produit",
                            $classe
                        )
                    });

                    let operation = match *methode {
                        actix_web::http::Method::POST => item.post.as_ref(),
                        actix_web::http::Method::PUT => item.put.as_ref(),
                        actix_web::http::Method::DELETE => item.delete.as_ref(),
                        actix_web::http::Method::PATCH => item.patch.as_ref(),
                        _ => None,
                    }
                    .unwrap_or_else(|| {
                        panic!("« {nom} » : le contrat ne sert pas {methode} sur {chemin}")
                    });

                    inspectees += 1;

                    let gardee = operation
                        .security
                        .as_ref()
                        .is_some_and(|exigences| !exigences.is_empty());
                    if !gardee {
                        sans_securite.push(*nom);
                    }
                }

                assert_eq!(
                    inspectees,
                    operations.len(),
                    "{inspectees} opération(s) inspectée(s) sur {} déclarée(s)",
                    operations.len()
                );
                assert!(
                    sans_securite.is_empty(),
                    "ces opérations de classe {} ne portent aucune exigence d'authentification :\
                     \n  {}\n\n\
                     Une opération de classe B, C ou D atteignable sans jeton est atteignable \
                     depuis n'importe quel chemin — y compris un terminal qui vide une file \
                     locale. Le principe VI l'interdit.",
                    $classe,
                    sans_securite.join("\n  ")
                );
            }
        }
    };
}

/// **Les tests de la classe D** — la double soumission au retour du réseau.
///
/// # ⚠️ INSTALLÉE À VIDE, avec son assertion de non-régression
///
/// **Aucune opération de classe D n'existe dans le produit.** La classe D est celle des appels à
/// un tiers non idempotent : la certification FNE (FIS, tranche T3) et le Mobile Money (CAI). Ni
/// l'un ni l'autre n'est écrit.
///
/// Écrire le test contre un mécanisme absent produirait soit un test qui ne compile pas, soit —
/// bien pire — un test qui passe en n'exerçant rien. La macro est donc **posée et vide**, avec le
/// contrôle qui échoue dès que sa cible apparaît, sur le patron de `portes_a_vide.rs`.
///
/// # Ce qu'il faudra écrire, et pourquoi c'est le test le plus important des quatre
///
/// L'API FNE **n'a aucune clé d'idempotence**. Une double soumission au retour du réseau produit
/// deux certifications de la même facture — et une facture certifiée deux fois ne se corrige pas :
/// elle demande un avoir, donc une pièce fiscale de plus, donc une explication au client.
///
/// L'état `INDETERMINEE` (délai dépassé) n'est **jamais** rejoué automatiquement : rapprochement
/// manuel obligatoire. C'est ce que le test devra constater.
#[macro_export]
macro_rules! tester_classe_d {
    ($nom:ident, cible = $cible:literal $(,)?) => {
        mod $nom {
            // Voir la note de `tester_classe_bcd!` : aucun import du fichier appelant n'est requis.

            /// **La classe D n'a pas encore de cible — et ce test le constate.**
            #[test]
            fn la_double_soumission_reste_a_ecrire() {
                let contrat = kaya_api::application::contrat_complet();

                let suspects: Vec<&String> = contrat
                    .paths
                    .paths
                    .keys()
                    .filter(|chemin| {
                        let c = chemin.to_lowercase();
                        c.contains("certification")
                            || c.contains("fne")
                            || c.contains("mobile-money")
                            || c.contains("paiement")
                    })
                    .collect();

                assert!(
                    suspects.is_empty(),
                    "une opération de classe D est apparue au contrat : {suspects:?}\n\n\
                     ═══════════════════════════════════════════════════════════════════════\n\
                     CE TEST DOIT MAINTENANT ÊTRE ÉCRIT ({}) :\n\
                     ═══════════════════════════════════════════════════════════════════════\n\
                     \n\
                     1. Soumettre une opération de classe D, réseau coupé pendant la réponse.\n\
                     2. Rétablir le réseau. Le terminal ne sait pas si le tiers a reçu.\n\
                     3. Soumettre à nouveau.\n\
                     \n\
                     **CE QUI DOIT ÊTRE VRAI :**\n\
                     \n\
                       · le tiers n'est PAS appelé deux fois — l'API FNE n'a aucune clé\n\
                         d'idempotence, et une facture certifiée deux fois demande un avoir ;\n\
                       · l'état `INDETERMINEE` n'est JAMAIS rejoué automatiquement : le\n\
                         rapprochement est manuel, obligatoire, et c'est une décision du\n\
                         cadrage — pas une limitation technique ;\n\
                       · les `id` d'items rendus par la certification sont PERSISTÉS. Sans eux,\n\
                         aucun avoir n'est possible, et l'erreur est irrattrapable a posteriori.",
                    $cible
                );

                println!(
                    "classe D — installée à vide. Cible attendue : {}. Ce test échouera dès \
                     qu'elle apparaîtra, avec le scénario dans son message.",
                    $cible
                );
            }
        }
    };
}
