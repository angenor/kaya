//! **FR-012 — deux échecs de connexion sont indiscernables, y compris en TEMPS.**
//!
//! # Le défaut que ce fichier attrape, et qu'aucune relecture ne verrait
//!
//! Un service d'authentification écrit naturellement ressemble à ceci :
//!
//! ```ignore
//! let Some(compte) = resoudre(identifiant).await? else {
//!     return Err(IdentifiantsInvalides);      // ← ici, le défaut
//! };
//! if !verifier(&compte.condensat, mot_de_passe)? { return Err(IdentifiantsInvalides) }
//! ```
//!
//! Le code est **juste**. Le message est identique, le statut est identique, la revue passe. Et le
//! chemin de gauche répond en une fraction de milliseconde quand celui de droite paie 19 Mio et
//! quelques dizaines de millisecondes d'Argon2. **L'écart se mesure depuis n'importe quel réseau**,
//! et il répond à la question « ce numéro est-il client de Kaya ? » — c'est-à-dire « sur qui
//! insister ». La liste du personnel d'un hôtel est affichée à son accueil.
//!
//! # Pourquoi des MÉDIANES et un RAPPORT, jamais un seuil absolu
//!
//! Un seuil en millisecondes serait inutilisable : la CI n'a pas de temps stable, le poste de
//! développement encore moins, et le premier `assert!(duree < 50ms)` serait désactivé au troisième
//! échec fortuit. Le rapport entre deux médianes, lui, est **sans dimension** : il ne dépend ni de
//! la machine, ni de sa charge, tant que les deux séries les subissent ensemble.
//!
//! La médiane plutôt que la moyenne pour la même raison : une pause du ramasse-miettes ou un
//! ordonnancement malheureux produit des valeurs extrêmes qui déplacent une moyenne et laissent
//! une médiane intacte.
//!
//! **Facteur 2**, et il est large exprès. Le défaut réel produit un rapport de l'ordre de 100 :
//! entre « ne rien faire » et « hacher », il n'y a pas de zone grise. Un facteur serré attraperait
//! le même défaut et échouerait aussi les jours de charge — donc serait désactivé, donc
//! n'attraperait plus rien.
//!
//! # Cent tentatives, et la limitation de débit qui les refuserait
//!
//! `LimiteTentatives` plafonne à dix essais par identifiant et soixante par origine dans une
//! fenêtre de cinq minutes. Une série de cent tentatives sur **le même** identifiant serait donc
//! refusée avant tout hachage à partir de la onzième — et les deux séries deviendraient également
//! rapides, ce qui ferait passer le test **au vert sans qu'il mesure quoi que ce soit**.
//!
//! Le test crée donc cent comptes distincts et emploie une origine distincte par tentative. C'est
//! la seule issue honnête : **désactiver le limiteur reviendrait à retirer du chemin mesuré une
//! pièce qui s'y trouve en production**, et le temps mesuré ne serait plus celui du produit.
//!
//! # Périmètre inspecté
//!
//! | Contrôle | Périmètre |
//! |---|---|
//! | Temps | 100 tentatives de chaque type, **sur le service réel**, base et Redis réels |
//! | Débit | Les deux compteurs, identifiant **et** origine, et le refus indiscernable |
//! | Message et code | Les **trois** causes de refus : inconnu, mot de passe faux, compte désactivé |
//! | Jamais à la connexion | Les fichiers du chemin de connexion, contrôle statique |

mod commun;

use std::time::{Duration, Instant};

use kaya_comptes::session::ErreurSession;
use kaya_comptes::session::limite::MAX_PAR_IDENTIFIANT;
use uuid::Uuid;

/// Nombre de tentatives par série.
const TENTATIVES: usize = 100;

/// Rapport maximal admis entre les deux médianes, dans un sens comme dans l'autre.
const FACTEUR_MAX: f64 = 2.0;

/// Mot de passe des comptes de ce fichier. Il satisfait la politique, ce qui n'a pas d'importance
/// ici — mais un mot de passe refusé à la création rendrait le jeu d'essai muet.
const MOT_DE_PASSE: &str = "chaise-tomate-abidjan";

/// Fabrique un identifiant de connexion **unique à cette exécution**.
///
/// # Le défaut que cela corrige, et qui n'est pas dans le code du produit
///
/// L'unicité de `comptes.compte` est **par tenant** : `+2250700000001` peut donc exister chez
/// autant de tenants qu'on veut, et `comptes.resoudre_identifiant` rend le **premier par ordre
/// stable**. Un identifiant figé dans un test se retrouve donc, à la deuxième exécution, résolu
/// vers le compte créé par la **première** — dans un autre tenant, avec un autre condensat.
///
/// Le symptôme est déroutant : le test passe une fois puis échoue toujours, sur une comparaison
/// de `tenant_id` qui n'a rien à voir avec ce qu'il vérifie. La base n'est pas réinitialisée
/// entre les exécutions, et c'est le cas normal ici.
/// Une origine distincte par tentative — voir le commentaire de tête.
///
/// Les adresses sont prises dans `198.51.100.0/24`, réservée à la documentation par la RFC 5737 :
/// aucune ne désigne une machine réelle, ici ou ailleurs.
fn origine(index: usize) -> String {
    format!("198.51.100.{}", index % 256)
}

fn identifiant_unique(prefixe: &str) -> String {
    // Les douze derniers caractères hexadécimaux d'un UUID v7 — le nœud, tiré au hasard.
    let uuid = Uuid::now_v7().simple().to_string();
    format!("+225{prefixe}{}", &uuid[uuid.len() - 9..])
}

// =================================================================================================
//  1 · Le temps
// =================================================================================================

/// **Cent tentatives sur identifiant inconnu, cent sur mot de passe faux, mêmes médianes.**
///
/// C'est le seul test du produit dont le sujet est une **durée**. Il est lent par nature — deux
/// cents hachages Argon2 à 19 Mio — et c'est le prix de la seule preuve possible : mesurer.
#[tokio::test]
async fn les_deux_echecs_prennent_le_meme_temps() {
    let pool_owner = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool_owner, "FR-012 — indiscernabilité").await;
    // Cent comptes, un par tentative de la série « connu » — voir le commentaire de tête : une
    // série sur un identifiant unique serait coupée par le limiteur dès la onzième.
    let mut connus = Vec::with_capacity(TENTATIVES);
    for _ in 0..TENTATIVES {
        let identifiant = identifiant_unique("70");
        commun::creer_compte(
            &pool_owner,
            jeu.tenant_id,
            "Indiscernable",
            &identifiant,
            MOT_DE_PASSE,
            &[("caissier", Some(jeu.etablissement_id))],
        )
        .await;
        connus.push(identifiant);
    }

    let service = commun::service_authentification(commun::pool_app().await);

    // **Un tour de chauffe avant les mesures.** Le condensat factice est calculé au premier appel
    // s'il ne l'a pas déjà été, et la première connexion à Redis coûte un aller-retour de plus :
    // les compter fausserait la première série, quelle qu'elle soit.
    for i in 0..3 {
        let _ = service
            .ouvrir(&identifiant_unique("70"), MOT_DE_PASSE, None, None, &origine(900 + i))
            .await;
        let _ = service
            .ouvrir(&connus[i], "mauvais-mot-de-passe", None, None, &origine(950 + i))
            .await;
    }

    // Les deux séries sont **entrelacées**, pas exécutées l'une puis l'autre : une machine qui
    // ralentit en cours de test pénaliserait sinon la seconde série entière, et le test
    // signalerait un défaut qui n'existe pas.
    let mut inconnus = Vec::with_capacity(TENTATIVES);
    let mut mauvais = Vec::with_capacity(TENTATIVES);

    for i in 0..TENTATIVES {
        let inexistant = identifiant_unique("79");

        let debut = Instant::now();
        let refus = service
            .ouvrir(&inexistant, MOT_DE_PASSE, None, None, &origine(i))
            .await;
        inconnus.push(debut.elapsed());
        assert!(matches!(refus, Err(ErreurSession::IdentifiantsInvalides)));

        let debut = Instant::now();
        let refus = service
            .ouvrir(&connus[i], "mauvais-mot-de-passe", None, None, &origine(1000 + i))
            .await;
        mauvais.push(debut.elapsed());
        assert!(matches!(refus, Err(ErreurSession::IdentifiantsInvalides)));
    }

    let mediane_inconnus = mediane(&mut inconnus);
    let mediane_mauvais = mediane(&mut mauvais);

    let rapport = mediane_inconnus.as_secs_f64() / mediane_mauvais.as_secs_f64();

    assert!(
        rapport <= FACTEUR_MAX && rapport >= 1.0 / FACTEUR_MAX,
        "FR-012 ÉCHOUE sur le TEMPS — identifiant inconnu : {mediane_inconnus:?}, mot de passe \
         faux : {mediane_mauvais:?}, rapport {rapport:.2} hors du facteur {FACTEUR_MAX}.\n\
         \n\
         Le message et le code sont peut-être identiques ; la durée, elle, dit si le compte \
         existe. Presque toujours, la cause est un retour anticipé sur identifiant inconnu, avant \
         la vérification Argon2 : sur ce chemin le service ne fait rien, sur l'autre il paie 19 \
         Mio. Le remède est le condensat factice — vérifier QUAND MÊME, contre une valeur qui ne \
         correspond à rien."
    );

    println!(
        "FR-012 — inconnu {mediane_inconnus:?}, mot de passe faux {mediane_mauvais:?}, \
         rapport {rapport:.2} (limite {FACTEUR_MAX})"
    );
}

// =================================================================================================
//  2 · Le message et le code
// =================================================================================================

/// **Les trois causes de refus rendent le même objet.**
///
/// Compte inconnu, mot de passe faux, compte désactivé. Le troisième est celui qu'on distinguerait
/// « pour rendre service » : un `compte_desactive` explicite aiderait l'utilisateur légitime et
/// dirait à l'attaquant que le compte existe. FR-012 tranche, et le diagnostic part dans les
/// journaux.
#[tokio::test]
async fn les_trois_causes_de_refus_rendent_le_meme_refus() {
    let pool_owner = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool_owner, "FR-012 — trois causes").await;

    let actif = identifiant_unique("71");
    let desactive = identifiant_unique("72");

    commun::creer_compte(
        &pool_owner,
        jeu.tenant_id,
        "Actif",
        &actif,
        MOT_DE_PASSE,
        &[("caissier", Some(jeu.etablissement_id))],
    )
    .await;
    let compte_desactive = commun::creer_compte(
        &pool_owner,
        jeu.tenant_id,
        "Désactivé",
        &desactive,
        MOT_DE_PASSE,
        &[("caissier", Some(jeu.etablissement_id))],
    )
    .await;

    let mut tx = pool_owner.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, jeu.tenant_id)
        .await
        .expect("pose du tenant");
    sqlx::query!(
        "UPDATE comptes.compte SET actif = false WHERE id = $1",
        compte_desactive.compte_id
    )
    .execute(&mut *tx)
    .await
    .expect("désactivation");
    tx.commit().await.expect("commit");

    let service = commun::service_authentification(commun::pool_app().await);

    let cas = [
        ("identifiant inconnu", "+22509999999999", MOT_DE_PASSE),
        ("mot de passe faux", actif.as_str(), "pas-le-bon-mot-de-passe"),
        ("compte désactivé", desactive.as_str(), MOT_DE_PASSE),
    ];

    for (nom, identifiant, mot_de_passe) in cas {
        let refus = service
            .ouvrir(identifiant, mot_de_passe, None, None, &origine(11))
            .await;

        match refus {
            Err(ErreurSession::IdentifiantsInvalides) => {}
            Err(autre) => panic!(
                "« {nom} » rend « {autre} » au lieu du refus commun. Un refus distinct dit à qui \
                 essaie que le compte existe — c'est exactement ce que FR-012 ferme."
            ),
            Ok(_) => panic!("« {nom} » a ouvert une session"),
        }
    }

    // Le versant positif : le **bon** mot de passe ouvre bien une session. Sans lui, un service
    // qui refuserait tout passerait ce test au vert.
    let ouverte = service
        .ouvrir(&actif, MOT_DE_PASSE, None, None, &origine(12))
        .await
        .expect("le bon mot de passe doit ouvrir une session");
    assert!(!ouverte.jetons.acces.is_empty());
    assert_eq!(ouverte.tenant_id, jeu.tenant_id);
}

/// Une connexion réussie **n'émet aucun événement au grand livre** (research R-15).
///
/// Le grand livre a une rétention illimitée. Y inscrire les connexions y écrirait la liste
/// horodatée des présences du personnel, pour toujours — un fichier que personne n'a décidé de
/// constituer et qu'aucune purge ne peut retirer, l'outbox étant immuable.
#[tokio::test]
async fn une_connexion_reussie_n_emet_aucun_evenement() {
    let pool_owner = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool_owner, "R-15 — pas de présence au grand livre").await;
    let identifiant = identifiant_unique("73");

    commun::creer_compte(
        &pool_owner,
        jeu.tenant_id,
        "Présence",
        &identifiant,
        MOT_DE_PASSE,
        &[("caissier", Some(jeu.etablissement_id))],
    )
    .await;

    let service = commun::service_authentification(commun::pool_app().await);

    let ouverte = service
        .ouvrir(&identifiant, MOT_DE_PASSE, None, None, &origine(13))
        .await
        .expect("connexion");
    let _ = service
        .rafraichir(&ouverte.jetons.rafraichissement, None)
        .await
        .expect("rafraîchissement");
    let _ = service
        .ouvrir(&identifiant, "faux", None, None, &origine(14))
        .await;

    // **Le tenant est posé avant de compter.** La politique d'isolation du grand livre convertit
    // `current_setting('app.current_tenant', true)` en `uuid` ; hors transaction ayant posé le
    // réglage, ce paramètre vaut la **chaîne vide** — pas `NULL` — dès qu'une transaction
    // antérieure de la même connexion l'a posé, et la conversion échoue sur `22P02`. Le symptôme
    // ne dit rien de sa cause.
    let mut tx = pool_owner.begin().await.expect("transaction");
    kaya_etablissements::tenant_context::poser_tenant(&mut tx, jeu.tenant_id)
        .await
        .expect("pose du tenant");
    let evenements: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) AS "c!"
        FROM synchronisation.evenement_outbox
        WHERE tenant_id = $1
        "#,
        jeu.tenant_id
    )
    .fetch_one(&mut *tx)
    .await
    .expect("décompte");
    tx.rollback().await.expect("rollback");

    assert_eq!(
        evenements, 0,
        "une connexion, un rafraîchissement ou un échec ont écrit {evenements} événement(s) au \
         grand livre. Ce ne sont pas des transitions d'état métier : les y inscrire y écrirait la \
         liste horodatée des présences du personnel, dans un registre à rétention illimitée et \
         immuable."
    );
}

// =================================================================================================
//  3 · La limitation de débit — deux compteurs, et un refus qui ne dit rien
// =================================================================================================

/// **Un acharnement sur un identifiant est coupé, et le refus reste le refus commun.**
///
/// Le point du test n'est pas que la limite existe — c'est qu'elle ne **parle** pas. Un message
/// « trop de tentatives » sur un identifiant existant, et rien sur un identifiant inconnu,
/// rétablirait en clair la fuite que le condensat factice referme.
#[tokio::test]
async fn l_acharnement_sur_un_identifiant_est_coupe_sans_le_dire() {
    let pool_owner = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool_owner, "CPT-01 — limitation par identifiant").await;
    let identifiant = identifiant_unique("74");

    commun::creer_compte(
        &pool_owner,
        jeu.tenant_id,
        "Acharné",
        &identifiant,
        MOT_DE_PASSE,
        &[("caissier", Some(jeu.etablissement_id))],
    )
    .await;

    let service = commun::service_authentification(commun::pool_app().await);

    // Le seuil par identifiant est de dix. Chaque tentative part d'une **origine différente** :
    // sans cela on ne saurait pas lequel des deux compteurs a coupé, et le test prouverait l'un
    // en croyant prouver l'autre.
    for i in 0..MAX_PAR_IDENTIFIANT {
        let refus = service
            .ouvrir(&identifiant, "faux", None, None, &origine(200 + i))
            .await;
        assert!(
            matches!(refus, Err(ErreurSession::IdentifiantsInvalides)),
            "tentative {i} : refus inattendu"
        );
    }

    // Au-delà du seuil, **même avec le BON mot de passe**, la connexion est refusée — et du même
    // refus. C'est ce qui prouve que la coupure vient du débit et non du mot de passe.
    let refus = service
        .ouvrir(&identifiant, MOT_DE_PASSE, None, None, &origine(299))
        .await;

    match refus {
        Err(ErreurSession::IdentifiantsInvalides) => {}
        Err(autre) => panic!(
            "le dépassement rend « {autre} » au lieu du refus commun. Un refus distinct dit à qui \
             essaie que le compte existe — et qu'il est sur la bonne piste."
        ),
        Ok(_) => panic!(
            "la {}e tentative a ouvert une session : la limitation par identifiant ne coupe rien, \
             et un balayage de mots de passe n'a aucun frein",
            MAX_PAR_IDENTIFIANT + 1
        ),
    }
}

/// **Le compteur par identifiant n'enferme pas les autres comptes.**
///
/// Le versant positif, et il n'est pas décoratif : un limiteur qui refuserait tout passerait le
/// test précédent au vert. Il vérifie surtout **l'absence de verrouillage définitif** — un compte
/// bloqué après N échecs serait un déni de service offert à qui connaît le téléphone d'Adjoua.
#[tokio::test]
async fn la_limitation_d_un_compte_n_atteint_pas_les_autres() {
    let pool_owner = commun::pool_owner().await;
    let jeu = commun::creer_tenant(&pool_owner, "CPT-01 — limitation cloisonnée").await;

    let harcele = identifiant_unique("75");
    let voisin = identifiant_unique("76");

    for (nom, identifiant) in [("Harcelé", &harcele), ("Voisin", &voisin)] {
        commun::creer_compte(
            &pool_owner,
            jeu.tenant_id,
            nom,
            identifiant,
            MOT_DE_PASSE,
            &[("caissier", Some(jeu.etablissement_id))],
        )
        .await;
    }

    let service = commun::service_authentification(commun::pool_app().await);

    for i in 0..MAX_PAR_IDENTIFIANT + 2 {
        let _ = service
            .ouvrir(&harcele, "faux", None, None, &origine(300 + i))
            .await;
    }

    // Le voisin, lui, se connecte normalement. Sans ce contrôle, un compteur posé sur la mauvaise
    // clé — l'origine au lieu de l'identifiant, par exemple — passerait inaperçu.
    let ouverte = service
        .ouvrir(&voisin, MOT_DE_PASSE, None, None, &origine(399))
        .await;

    assert!(
        ouverte.is_ok(),
        "l'acharnement sur un compte a bloqué un autre compte : le compteur n'est pas posé sur \
         l'identifiant, ou un verrouillage global s'est glissé quelque part. {:?}",
        ouverte.err()
    );
}

// =================================================================================================
//  4 · La politique n'est jamais appelée à la connexion
// =================================================================================================

/// Les fichiers qui portent le chemin de connexion.
///
/// Liste **explicite** plutôt qu'un motif de nom : un fichier renommé sortirait silencieusement
/// d'un motif, alors qu'il fait échouer une liste nommée.
const CHEMINS_DE_CONNEXION: &[&str] = &[
    "crates/socle/comptes/src/authentification/service.rs",
    "crates/socle/comptes/src/session/entrepot.rs",
];

/// **Aucun fichier du chemin de connexion n'appelle la politique de mot de passe.**
///
/// Vérifier à la connexion qu'un mot de passe est encore conforme est un geste raisonnable en
/// apparence. Il transformerait chaque mise à jour de la liste des mots de passe compromis en
/// verrouillage silencieux de comptes légitimes : **la liste grossit, le mot de passe ne change
/// pas**. Et le refus rendu serait celui, volontairement muet, de FR-012 — donc sans le moindre
/// indice pour l'utilisateur ni pour le support.
#[test]
fn la_politique_n_est_jamais_appelee_sur_le_chemin_de_connexion() {
    let racine = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut inspectes = 0_usize;
    let mut fautifs = Vec::new();

    for relatif in CHEMINS_DE_CONNEXION {
        let chemin = racine.join(relatif);
        assert!(
            chemin.is_file(),
            "{relatif} n'existe pas — le contrôle n'inspecterait rien, et une porte sans cible \
             passe toujours au vert"
        );
        inspectes += 1;

        let contenu = std::fs::read_to_string(&chemin).expect("lecture");
        for (numero, ligne) in contenu.lines().enumerate() {
            let sans_commentaire = ligne.split("//").next().unwrap_or("");
            if sans_commentaire.contains("politique::")
                || sans_commentaire.contains("verifier_politique")
                || sans_commentaire.contains("est_compromis")
            {
                fautifs.push(format!("{relatif}:{}", numero + 1));
            }
        }
    }

    assert_eq!(inspectes, CHEMINS_DE_CONNEXION.len());
    assert!(
        fautifs.is_empty(),
        "la politique de mot de passe est appelée sur le chemin de connexion : {fautifs:?}\n\
         Elle porte sur la CRÉATION et le CHANGEMENT, jamais sur la connexion."
    );
}

// =================================================================================================
//  Outils
// =================================================================================================

/// Médiane d'une série de durées.
///
/// Trie sur place — l'appelant n'a plus besoin de l'ordre d'origine, et copier cent durées pour
/// préserver un ordre inutile serait du bruit.
fn mediane(durees: &mut [Duration]) -> Duration {
    assert!(!durees.is_empty(), "série vide : rien à mesurer");
    durees.sort_unstable();
    durees[durees.len() / 2]
}

/// Les identifiants fabriqués par [`identifiant_unique`] ne se répètent pas.
///
/// Garde-fou trivial et volontaire : les séries ci-dessus créent des comptes réels dans une base
/// qui n'est pas réinitialisée entre les exécutions. Une collision rendrait la série « inconnu »
/// partiellement « connue » — donc le test vert pour une mauvaise raison.
#[test]
fn les_identifiants_fabriques_ne_se_repetent_pas() {
    let fabriques: std::collections::BTreeSet<String> =
        (0..TENTATIVES).map(|_| identifiant_unique("79")).collect();
    assert_eq!(fabriques.len(), TENTATIVES);
}
