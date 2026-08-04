//! Données de démonstration — **rejouables**.
//!
//! ```sh
//! cargo run -p kaya-api --bin seeds
//! cargo run -p kaya-api --bin seeds     # même état final
//! ```
//!
//! # Trois propriétés, et ce qui les tient
//!
//! **Rejouable.** Trois exécutions successives produisent le même état final. Ce n'est pas obtenu
//! par un `DELETE` préalable — qui détruirait les données de travail du pilote à chaque
//! démonstration — mais par des **identifiants fixes**. Chaque ligne seedée a un UUID écrit en dur
//! ci-dessous.
//!
//! # `DO UPDATE` sur l'identité, `DO NOTHING` sur le reste — la distinction compte
//!
//! Les **colonnes d'identité** des deux établissements de démonstration sont réappliquées à chaque
//! exécution. Ce sont des valeurs de référence, pas des données de travail : « recharger la
//! démonstration » doit restituer **exactement** l'état décrit ici.
//!
//! Le cycle 002 a montré pourquoi. La migration `0007` a ajouté `commune` avec un `DEFAULT ''`
//! retiré aussitôt ; les lignes existantes ont donc reçu une chaîne vide, et un `DO NOTHING` les y
//! aurait laissées **pour toujours** — l'écran `G1` affichait une commune vide sur le tenant du
//! pilote, alors que le seed déclarait « Abengourou » deux lignes plus haut. Un seed qui n'applique
//! pas les valeurs qu'il déclare donne un état faux, et personne ne pense à le soupçonner.
//!
//! Les **activations, capacités, points de vente et tables** gardent `DO NOTHING` : leur unicité
//! porte sur le couple qui les définit, et les réécrire à l'identique ne changerait rien.
//!
//! **Séparé des migrations** (principe I(b)). Une migration décrit le schéma et n'est jamais
//! rejouée ; un seed décrit un jeu de données et l'est constamment. Les mêler rendrait impossible
//! de recharger une démonstration sans toucher au schéma.
//!
//! **Sous le rôle applicatif.** Les seeds passent par `kaya_app`, soumis à la sécurité au niveau
//! ligne, et posent le contexte de tenant comme le ferait l'application. Les écrire sous
//! `kaya_owner` contournerait ce que le reste du cycle cherche à garantir — et un jeu de données
//! seedé hors politique serait invisible depuis l'application.
//!
//! # Portée réduite, assumée
//!
//! Ce cycle livre **la mécanique et les deux tenants**. Les 17 unités, les 30 articles et les
//! 5 comptes de test de FR-062 peuplent des tables qui n'existent pas encore — elles viennent des
//! cycles HEB, PDV et CPT. Ce qu'ils devront contenir est écrit dans
//! `backend/migrations/seeds/README.md`, pour que chaque cycle sache ce qu'il doit y ajouter.

use kaya_api::db;
use kaya_etablissements::tenant_context;
use sqlx::PgPool;
use time::Time;
use time::macros::time;
use uuid::{Uuid, uuid};

// =================================================================================================
//  Identifiants FIXES — c'est eux qui rendent le seed rejouable
// =================================================================================================
//
// Écrits en dur, jamais tirés au hasard. Un `Uuid::now_v7()` produirait un nouveau jeu à chaque
// exécution : la base grossirait sans fin et « recharger la démonstration » créerait un troisième
// établissement au lieu de retrouver le premier.

/// Tenant du pilote — Résidence Hôtel Deloria, Abengourou (cadrage §2.1).
const TENANT_DELORIA: Uuid = uuid!("0198c4a0-0000-7000-8000-000000000001");
const ETABLISSEMENT_DELORIA: Uuid = uuid!("0198c4a0-0000-7000-8000-000000000002");

/// Second tenant — **module hébergement seul, aucun point de vente**.
const TENANT_RESIDENCE_TEST: Uuid = uuid!("0198c4a0-0000-7000-8000-000000000011");
const ETABLISSEMENT_RESIDENCE_TEST: Uuid = uuid!("0198c4a0-0000-7000-8000-000000000012");

/// Les cinq services de Deloria, identifiants fixes.
const SERVICES_DELORIA: [(Uuid, &str); 5] = [
    (uuid!("0198c4a0-0000-7000-8000-000000000021"), "HEBERGEMENT"),
    (uuid!("0198c4a0-0000-7000-8000-000000000022"), "RESTAURATION"),
    (uuid!("0198c4a0-0000-7000-8000-000000000023"), "BAR"),
    (uuid!("0198c4a0-0000-7000-8000-000000000024"), "PRESSING"),
    (uuid!("0198c4a0-0000-7000-8000-000000000025"), "SALLE_REUNION"),
];

/// **`STOCK` au profil `SIMPLE`, déclarée par RESTAURATION et BAR seulement.**
///
/// Ce sont les deux services qui vendent des articles stockés — hypothèse 9 de la spécification,
/// révisable sans coût avant le cycle STK. `HEBERGEMENT`, `PRESSING` et `SALLE_REUNION` n'en
/// déclarent aucune, et **c'est ce qui rend le jeu de données représentatif** : un seed où tout
/// est activé partout ne prouverait rien du refus ni de l'absence.
const CAPACITES_DELORIA: [(Uuid, &str); 2] = [
    (uuid!("0198c4a0-0000-7000-8000-000000000031"), "RESTAURATION"),
    (uuid!("0198c4a0-0000-7000-8000-000000000032"), "BAR"),
];

/// Les deux points de vente de Deloria. Le second n'a **aucune table** : c'est un comptoir.
const POINTS_DE_VENTE_DELORIA: [(Uuid, &str, &str); 2] = [
    (
        uuid!("0198c4a0-0000-7000-8000-000000000041"),
        "RESTAURATION",
        "Salle du restaurant",
    ),
    (
        uuid!("0198c4a0-0000-7000-8000-000000000042"),
        "BAR",
        "Comptoir du bar",
    ),
];

/// Les tables de la salle du restaurant. Le comptoir du bar n'en a aucune.
const TABLES_DELORIA: [(Uuid, &str); 3] = [
    (uuid!("0198c4a0-0000-7000-8000-000000000051"), "1"),
    (uuid!("0198c4a0-0000-7000-8000-000000000052"), "2"),
    (uuid!("0198c4a0-0000-7000-8000-000000000053"), "Terrasse"),
];

/// Le service HEBERGEMENT de Résidence Test — **le seul**, et sans capacité.
const SERVICE_RESIDENCE_TEST: Uuid = uuid!("0198c4a0-0000-7000-8000-000000000061");

// -------------------------------------------------------------------------------------------
//  HEB — le parc de Deloria : 17 unités en 5 catégories, plus la salle de réunion
// -------------------------------------------------------------------------------------------
//
// Répartition du cadrage §2.1, à la ligne près. La salle de réunion est une **catégorie**, pas une
// entité nouvelle : c'est la décision de HEB-01, et le jeu de données doit la tenir, sans quoi le
// premier lecteur croira qu'une table manque.
//
// # Ce que le cadrage donne, et ce qu'il ne donne pas
//
// Il donne les cinq tarifs de nuitée et le tarif de la salle. Il ne donne **ni la capacité
// d'accueil par catégorie, ni le plan d'étage**. Les capacités sont donc uniformes et les étages
// nuls : une valeur uniforme signale qu'elle n'est pas relevée, là où une variation inventée se
// lirait comme un fait constaté. L'atelier terrain (**B-07**) les relèvera avec les barèmes.

/// `(id, nom, capacité d'accueil, prix de la nuitée, unités)`.
///
/// 3 + 5 + 4 + 2 + 3 = **17 unités**, le décompte du cadrage. Chaque unité porte son identifiant
/// **littéral** — la règle 1 du README des seeds : un identifiant calculé serait stable lui aussi,
/// mais c'est en le lisant qu'on vérifie qu'il n'a pas bougé d'un cycle à l'autre.
const CATEGORIES_CHAMBRES: [(Uuid, &str, i16, i64, &[(Uuid, &str)]); 5] = [
    (
        uuid!("0198c4a0-0000-7000-8000-000000000101"),
        "Standard",
        2,
        12_500,
        &[
            (uuid!("0198c4a0-0000-7000-8000-000000000201"), "A1"),
            (uuid!("0198c4a0-0000-7000-8000-000000000202"), "A2"),
            (uuid!("0198c4a0-0000-7000-8000-000000000203"), "A3"),
        ],
    ),
    (
        uuid!("0198c4a0-0000-7000-8000-000000000102"),
        "Classique",
        2,
        15_500,
        &[
            (uuid!("0198c4a0-0000-7000-8000-000000000211"), "B1"),
            (uuid!("0198c4a0-0000-7000-8000-000000000212"), "B2"),
            (uuid!("0198c4a0-0000-7000-8000-000000000213"), "B3"),
            (uuid!("0198c4a0-0000-7000-8000-000000000214"), "B4"),
            (uuid!("0198c4a0-0000-7000-8000-000000000215"), "B5"),
        ],
    ),
    (
        uuid!("0198c4a0-0000-7000-8000-000000000103"),
        "Classique supérieure",
        2,
        17_500,
        &[
            (uuid!("0198c4a0-0000-7000-8000-000000000221"), "C1"),
            (uuid!("0198c4a0-0000-7000-8000-000000000222"), "C2"),
            (uuid!("0198c4a0-0000-7000-8000-000000000223"), "C3"),
            (uuid!("0198c4a0-0000-7000-8000-000000000224"), "C4"),
        ],
    ),
    (
        uuid!("0198c4a0-0000-7000-8000-000000000104"),
        "Supérieure A",
        2,
        20_500,
        &[
            (uuid!("0198c4a0-0000-7000-8000-000000000231"), "D1"),
            (uuid!("0198c4a0-0000-7000-8000-000000000232"), "D2"),
        ],
    ),
    (
        uuid!("0198c4a0-0000-7000-8000-000000000105"),
        "Supérieure B",
        2,
        25_500,
        &[
            (uuid!("0198c4a0-0000-7000-8000-000000000241"), "E1"),
            (uuid!("0198c4a0-0000-7000-8000-000000000242"), "E2"),
            (uuid!("0198c4a0-0000-7000-8000-000000000243"), "E3"),
        ],
    ),
];

/// L'unique unité de la salle de réunion.
const UNITE_SALLE: (Uuid, &str) = (uuid!("0198c4a0-0000-7000-8000-000000000251"), "SR1");

/// La salle de réunion — **une sixième catégorie, à une unité**.
const CATEGORIE_SALLE: Uuid = uuid!("0198c4a0-0000-7000-8000-000000000106");

/// **Le tarif que le cadrage porte est JOURNALIER ; le produit vend la salle par PLAGE.**
///
/// Le §2.1 écrit « Salle de réunion 50 500/jour ». Deux plages sont déclarées — 8 h – 12 h et
/// 13 h – 16 h —, et le cadrage ne relève **pas** le prix de l'une d'elles. La valeur seedée
/// reprend donc le nombre du cadrage **sans le transformer** : diviser par deux poserait une règle
/// de tarification que personne n'a énoncée, et un nombre inventé dans un seed finit par être lu
/// comme un fait constaté.
///
/// **Provisoire jusqu'à B-07**, au même titre que le barème de passage — l'atelier terrain relève
/// les formules et barèmes réellement pratiqués.
const PRIX_DEMI_JOURNEE_MINEUR: i64 = 50_500;

/// Identifiants fixes des formules — `(id, id de catégorie)`, dans l'ordre de `CATEGORIES_CHAMBRES`.
const FORMULES_NUITEE: [Uuid; 5] = [
    uuid!("0198c4a0-0000-7000-8000-000000000111"),
    uuid!("0198c4a0-0000-7000-8000-000000000112"),
    uuid!("0198c4a0-0000-7000-8000-000000000113"),
    uuid!("0198c4a0-0000-7000-8000-000000000114"),
    uuid!("0198c4a0-0000-7000-8000-000000000115"),
];

const FORMULES_PASSAGE: [Uuid; 5] = [
    uuid!("0198c4a0-0000-7000-8000-000000000121"),
    uuid!("0198c4a0-0000-7000-8000-000000000122"),
    uuid!("0198c4a0-0000-7000-8000-000000000123"),
    uuid!("0198c4a0-0000-7000-8000-000000000124"),
    uuid!("0198c4a0-0000-7000-8000-000000000125"),
];

const FORMULE_DEMI_JOURNEE: Uuid = uuid!("0198c4a0-0000-7000-8000-000000000131");

/// **Le barème de passage du cadrage §5.3 — `(durée en minutes, prix)`.**
///
/// ⚠️ **Provisoire.** La décision **B-07** — « barèmes de passage réels du pilote » — n'est pas
/// prise ; le cadrage lui-même écrit que ces valeurs « sont à confirmer à l'atelier initial ». Les
/// traiter comme définitives les figerait dans les tests, et B-07 deviendrait un changement de
/// tests plutôt qu'un paramètre.
///
/// Le prix d'heure supplémentaire (+1 200) vit sur la formule, pas ici : il s'applique **au-delà**
/// du dernier palier et n'est donc pas un palier.
const BAREME_PASSAGE: [(i32, i64); 4] = [(60, 1_500), (120, 2_800), (180, 4_000), (240, 5_000)];

const PRIX_HEURE_SUPPLEMENTAIRE_MINEUR: i64 = 1_200;

/// Les deux plages de la salle — `(id, heure de début, heure de fin, clé de libellé)`.
///
/// Les libellés sont des **clés i18n**, jamais des phrases : une phrase seedée traverserait l'API
/// jusqu'à l'écran sans passer par le catalogue de traductions.
const PLAGES_SALLE: [(Uuid, Time, Time, &str); 2] = [
    (
        uuid!("0198c4a0-0000-7000-8000-000000000141"),
        time!(08:00),
        time!(12:00),
        "hebergement.plage.matin",
    ),
    (
        uuid!("0198c4a0-0000-7000-8000-000000000142"),
        time!(13:00),
        time!(16:00),
        "hebergement.plage.apres_midi",
    ),
];

/// **Les heures murales de la nuitée** — 14 h et 12 h (cadrage §5.2).
///
/// Elles vivent en **deux endroits**, et ce n'est pas une duplication : sur la formule, ce sont les
/// heures de la nuitée vendue ; au catalogue de configuration, ce sont les heures **par défaut de
/// l'établissement**, dont HEB-03 se sert quand une formule ne les porte pas. Les seeder aux mêmes
/// valeurs est un choix du pilote, pas une contrainte du modèle.
const HEURE_ARRIVEE_STANDARD: Time = time!(14:00);
const HEURE_DEPART_STANDARD: Time = time!(12:00);

/// **Les temps de remise en état du cadrage §5.4** — passage 30 min, nuitée 2 h, demi-journée 1 h.
///
/// Ils ne vont pas au catalogue de paramètres : « 30 min » n'a de sens que rapporté à une catégorie
/// **et** à une famille de formule, et un scalaire d'établissement perdrait l'un des deux axes.
const REMISE_EN_ETAT_CHAMBRE: [(&str, i32); 2] = [("NUITEE", 120), ("PASSAGE", 30)];
const REMISE_EN_ETAT_SALLE_MINUTES: i32 = 60;

// -------------------------------------------------------------------------------------------
//  HEB — Résidence Test : quatre meublés, MOIS et NUITÉE seulement
// -------------------------------------------------------------------------------------------
//
// **Ce tenant n'est pas un décor de démonstration, c'est une épreuve.** Il éprouve deux promesses
// que Deloria seule ne peut pas éprouver :
//
//   * « Aucune formule n'est réservée à un type d'établissement » (cadrage §5.2). Une résidence
//     meublée vend au mois **et** à la nuitée ; l'hôtel vend à la nuitée **et** au passage. Si le
//     code avait quelque part supposé qu'un mensuel appartient aux résidences, ce jeu de données ne
//     le montrerait pas — mais le premier hôtel qui vend au mois s'en apercevrait en production.
//   * **Aucun code ne suppose l'existence du passage.** Résidence Test n'a ni formule `PASSAGE`, ni
//     barème, ni plage. Un chemin de code qui exigerait un barème pour lire une offre échouerait
//     ici, et nulle part ailleurs.
//
// # Les valeurs sont des valeurs de TEST, et la distinction avec Deloria compte
//
// Les tarifs de Deloria sont relevés au cadrage §2.1 ; ceux-ci ne le sont pas et ne prétendent pas
// l'être. Aucun atelier ne viendra les confirmer : ce tenant n'a pas de client.

const CATEGORIE_MEUBLE: Uuid = uuid!("0198c4a0-0000-7000-8000-000000000301");

const UNITES_MEUBLE: [(Uuid, &str); 4] = [
    (uuid!("0198c4a0-0000-7000-8000-000000000311"), "M1"),
    (uuid!("0198c4a0-0000-7000-8000-000000000312"), "M2"),
    (uuid!("0198c4a0-0000-7000-8000-000000000313"), "M3"),
    (uuid!("0198c4a0-0000-7000-8000-000000000314"), "M4"),
];

const FORMULE_MEUBLE_NUITEE: Uuid = uuid!("0198c4a0-0000-7000-8000-000000000321");
const FORMULE_MEUBLE_MENSUEL: Uuid = uuid!("0198c4a0-0000-7000-8000-000000000322");

/// Les **cinq** valeurs de configuration Deloria, promises par les migrations `0023` et `0028`.
///
/// Le catalogue déclare qu'une clé existe ; **les valeurs viennent d'ici** — une migration n'a pas
/// de tenant courant et n'écrirait rien, en silence, sur une table en `FORCE ROW LEVEL SECURITY`.
///
/// `(id, clé, valeur JSON)`. Le type de la valeur est vérifié par le catalogue : `HEURE_LOCALE`
/// exige une chaîne, `DUREE_MINUTES` et `ENTIER` un entier.
const PARAMETRES_DELORIA: [(Uuid, &str, &str); 5] = [
    (
        uuid!("0198c4a0-0000-7000-8000-000000000151"),
        "heure_arrivee_standard",
        r#""14:00""#,
    ),
    (
        uuid!("0198c4a0-0000-7000-8000-000000000152"),
        "heure_depart_standard",
        r#""12:00""#,
    ),
    (
        uuid!("0198c4a0-0000-7000-8000-000000000153"),
        "seuil_bascule_nuitee_minutes",
        "480",
    ),
    // ── SYN, migration `0028` ────────────────────────────────────────────────────────────
    //
    // **300 secondes** — la valeur du cadrage §11.4 (« alerte au-delà de 5 minutes de dérive »),
    // posée en secondes parce que l'écart mesuré est une différence d'instants. Un terminal de
    // Deloria dont l'horloge s'écarte de plus de cinq minutes fait écrire une entrée au registre
    // des actions, **une par épisode**, et l'écriture n'est jamais refusée.
    (
        uuid!("0198c4a0-0000-7000-8000-000000000154"),
        "sync.derive_horloge_seuil_secondes",
        "300",
    ),
    // **3 000 millisecondes** — au-delà, le témoin affiche « Connexion faible » plutôt que
    // « Enregistré ». Sans cette valeur, le pilote verrait un état que rien ne produit : à
    // Abengourou, une 3G qui répond en quatre secondes est le cas courant, et c'est exactement
    // celui que le troisième état existe pour dire.
    (
        uuid!("0198c4a0-0000-7000-8000-000000000155"),
        "sync.latence_degradee_seuil_ms",
        "3000",
    ),
];

// -------------------------------------------------------------------------------------------
//  CPT — les trois personnes du pilote, et le cumul de rôles d'Adjoua
// -------------------------------------------------------------------------------------------
//
// **Adjoua porte les trois rôles, et c'est tout le point du cycle.** Un jeu de données où chacun
// n'aurait qu'un rôle ne démontrerait rien de l'union des permissions : c'est exactement la
// situation que le cadrage décrit — dans un établissement de cette taille, la même personne tient
// la réception le matin, la caisse le soir et gère l'équipe entre les deux.
//
// Yao n'a qu'un rôle, et M. Koffi est propriétaire : les trois ensemble donnent trois accueils
// différents sur la même application, ce que l'écran `R1` doit montrer.

/// M. Koffi — propriétaire de Deloria (cadrage §2.1).
const PERSONNE_KOFFI: Uuid = uuid!("0198c4a0-0000-7000-8000-000000000071");
const COMPTE_KOFFI: Uuid = uuid!("0198c4a0-0000-7000-8000-000000000072");

/// Adjoua — **gérante, caissière ET réceptionniste**.
const PERSONNE_ADJOUA: Uuid = uuid!("0198c4a0-0000-7000-8000-000000000073");
const COMPTE_ADJOUA: Uuid = uuid!("0198c4a0-0000-7000-8000-000000000074");

/// Yao — réceptionniste.
const PERSONNE_YAO: Uuid = uuid!("0198c4a0-0000-7000-8000-000000000075");
const COMPTE_YAO: Uuid = uuid!("0198c4a0-0000-7000-8000-000000000076");

/// Les attributions de rôles, identifiants fixes — `(id, compte, rôle)`.
///
/// L'établissement est toujours celui de Deloria : les huit rôles sauf `admin_editeur` sont de
/// portée `ETABLISSEMENT` et en exigent un.
const ROLES_DELORIA: [(Uuid, Uuid, &str); 5] = [
    (
        uuid!("0198c4a0-0000-7000-8000-000000000081"),
        COMPTE_KOFFI,
        "proprietaire",
    ),
    (
        uuid!("0198c4a0-0000-7000-8000-000000000082"),
        COMPTE_ADJOUA,
        "gerant",
    ),
    (
        uuid!("0198c4a0-0000-7000-8000-000000000083"),
        COMPTE_ADJOUA,
        "caissier",
    ),
    (
        uuid!("0198c4a0-0000-7000-8000-000000000084"),
        COMPTE_ADJOUA,
        "receptionniste",
    ),
    (
        uuid!("0198c4a0-0000-7000-8000-000000000085"),
        COMPTE_YAO,
        "receptionniste",
    ),
];

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if dotenvy::from_path("backend/.env").is_err() {
        let _ = dotenvy::dotenv();
    }
    kaya_api::observabilite::initialiser_journaux();

    // **Refus d'exécution en production**, avant toute connexion (T005). La garde vit dans le
    // binaire et non dans le script d'appel : un script se contourne d'une ligne de commande, et
    // c'est bien le binaire qu'on lance à la main un soir d'incident en cherchant à « juste
    // remettre les données de démonstration ».
    let mot_de_passe = kaya_api::secrets::mot_de_passe_seeds()?;

    let pool = db::pool_application().await?;

    seeder_deloria(&pool).await?;
    seeder_residence_test(&pool).await?;
    seeder_comptes_deloria(&pool, &mot_de_passe).await?;
    seeder_hebergement_deloria(&pool).await?;
    seeder_parametres_deloria(&pool).await?;
    seeder_hebergement_residence_test(&pool).await?;
    // ── Cycle 006 — les fiches clientes et les séjours de démonstration ─────────────────────
    seeder_clients(&pool).await?;
    seeder_sejours(&pool).await?;

    println!("Seeds appliqués. Deux tenants :");
    println!("  Deloria         {TENANT_DELORIA}  (établissement {ETABLISSEMENT_DELORIA})");
    println!(
        "  Résidence Test  {TENANT_RESIDENCE_TEST}  (établissement {ETABLISSEMENT_RESIDENCE_TEST})"
    );
    println!();
    println!(
        "Parc de Deloria : 17 unités en 5 catégories, plus la salle de réunion (SR1, 6e catégorie)."
    );
    println!(
        "Résidence Test  : 4 meublés, au mois et à la nuitée — aucun passage, aucune plage."
    );
    println!();
    println!("Trois comptes sur Deloria — le mot de passe vient de KAYA_SEEDS_MOT_DE_PASSE :");
    println!("  koffi@deloria.test    propriétaire");
    println!("  adjoua@deloria.test   gérante + caissière + réceptionniste  ← le cumul");
    println!("  yao@deloria.test      réceptionniste");
    println!();
    println!();
    println!("Fiches clientes : 12 sur Deloria, 2 sur Résidence Test — aucune pièce d'identité.");
    println!("Séjours         : 4 — nuitée en cours (2 nuits, 2 accompagnants), passage en cours");
    println!("                  sans fiche, séjour CLOS avec son constat de taxe FIGÉ, et un");
    println!("                  meublé en cours sur Résidence Test.");
    println!("                  Le constat porte nuitees_assujetties et montant_mineur à NULL :");
    println!("                  ce cycle fige des FAITS, le calcul appartient à FIS-03.");
    println!();
    println!("Rejouable : une seconde exécution laisse exactement le même état.");

    Ok(())
}

/// Tenant du pilote — **identité complète depuis ETB-01**.
///
/// Le cycle 001 ne pouvait seeder que le nom, le fuseau et la devise : `etablissement` était en
/// forme minimale. La migration `0007_etablissement_identite.sql` a livré les sept colonnes
/// d'identité, et `commune` est `NOT NULL` **sans défaut** — une création qui l'omettrait est
/// désormais refusée, ce qui est exactement le but.
async fn seeder_deloria(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let mut tx = pool.begin().await?;
    tenant_context::poser_tenant(&mut tx, TENANT_DELORIA).await?;

    sqlx::query!(
        r#"
        INSERT INTO etablissements.tenant (id, nom)
        VALUES ($1, 'Deloria')
        ON CONFLICT (id) DO NOTHING
        "#,
        TENANT_DELORIA
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!(
        r#"
        INSERT INTO etablissements.etablissement
            (id, tenant_id, nom, fuseau_horaire, devise,
             juridiction, classement, etoiles, commune, adresse)
        VALUES ($1, $2, 'Résidence Hôtel Deloria — Abengourou', 'Africa/Abidjan', 'XOF',
                'CI', 'NON_CLASSE', NULL, 'Abengourou', NULL)
        ON CONFLICT (id) DO UPDATE
        SET nom = EXCLUDED.nom,
            fuseau_horaire = EXCLUDED.fuseau_horaire,
            devise = EXCLUDED.devise,
            juridiction = EXCLUDED.juridiction,
            classement = EXCLUDED.classement,
            etoiles = EXCLUDED.etoiles,
            commune = EXCLUDED.commune,
            adresse = EXCLUDED.adresse
        "#,
        ETABLISSEMENT_DELORIA,
        TENANT_DELORIA
    )
    .execute(&mut *tx)
    .await?;

    // ── Cinq services actifs ────────────────────────────────────────────────────────────────
    //
    // `ON CONFLICT DO NOTHING` sur l'identifiant **et** sur le couple (établissement, module) :
    // la seconde contrainte est celle qui compte, un module ne s'activant qu'une fois par
    // établissement.
    for (id, code) in SERVICES_DELORIA {
        sqlx::query!(
            r#"
            INSERT INTO etablissements.etablissement_module
                (id, tenant_id, etablissement_id, module_code, module_implemente)
            VALUES ($1, $2, $3, $4, true)
            ON CONFLICT (etablissement_id, module_code) DO NOTHING
            "#,
            id,
            TENANT_DELORIA,
            ETABLISSEMENT_DELORIA,
            code,
        )
        .execute(&mut *tx)
        .await?;
    }

    // ── STOCK/SIMPLE sur RESTAURATION et BAR ────────────────────────────────────────────────
    for (id, module_code) in CAPACITES_DELORIA {
        let service_id = SERVICES_DELORIA
            .iter()
            .find(|(_, code)| *code == module_code)
            .map(|(id, _)| *id)
            .expect("le service qui déclare la capacité doit figurer dans SERVICES_DELORIA");

        sqlx::query!(
            r#"
            INSERT INTO etablissements.module_capacite
                (id, tenant_id, etablissement_module_id,
                 capacite_code, capacite_implementee, profil_code, profil_implemente)
            VALUES ($1, $2, $3, 'STOCK', true, 'SIMPLE', true)
            ON CONFLICT (etablissement_module_id, capacite_code) DO NOTHING
            "#,
            id,
            TENANT_DELORIA,
            service_id,
        )
        .execute(&mut *tx)
        .await?;
    }

    // ── Deux points de vente, dont un COMPTOIR ──────────────────────────────────────────────
    for (id, module_code, nom) in POINTS_DE_VENTE_DELORIA {
        let service_id = SERVICES_DELORIA
            .iter()
            .find(|(_, code)| *code == module_code)
            .map(|(id, _)| *id)
            .expect("le service du point de vente doit figurer dans SERVICES_DELORIA");

        sqlx::query!(
            r#"
            INSERT INTO etablissements.point_de_vente
                (id, tenant_id, etablissement_id, etablissement_module_id, nom)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (id) DO NOTHING
            "#,
            id,
            TENANT_DELORIA,
            ETABLISSEMENT_DELORIA,
            service_id,
            nom,
        )
        .execute(&mut *tx)
        .await?;
    }

    // Les tables de la salle. **Le comptoir du bar n'en reçoit aucune** — c'est ce qui en fait un
    // comptoir, et le jeu de données porte donc les deux formes.
    let salle = POINTS_DE_VENTE_DELORIA[0].0;
    for (id, libelle) in TABLES_DELORIA {
        sqlx::query!(
            r#"
            INSERT INTO etablissements.table_pdv (id, tenant_id, point_de_vente_id, libelle)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (point_de_vente_id, libelle) DO NOTHING
            "#,
            id,
            TENANT_DELORIA,
            salle,
            libelle,
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    tracing::info!(tenant = %TENANT_DELORIA, "tenant Deloria seedé");
    Ok(())
}

/// Second tenant — **la raison d'être de ce seed n'est pas la démonstration**.
///
/// « Résidence Test » porte le **module hébergement seul, sans aucun point de vente**. C'est ce
/// qui rend vérifiable la promesse la plus structurante du produit :
///
/// > Aucun crate partagé ne suppose qu'un établissement possède de l'hébergement, ni qu'il
/// > possède un point de vente (constitution, préambule).
///
/// Un jeu de données à un seul tenant complet laisserait cette promesse invérifiable jusqu'au
/// premier client maquis — c'est-à-dire jusqu'au moment où la corriger coûterait une refonte.
/// Il sert aussi de second tenant aux tests d'isolation.
async fn seeder_residence_test(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let mut tx = pool.begin().await?;
    tenant_context::poser_tenant(&mut tx, TENANT_RESIDENCE_TEST).await?;

    sqlx::query!(
        r#"
        INSERT INTO etablissements.tenant (id, nom)
        VALUES ($1, 'Résidence Test')
        ON CONFLICT (id) DO NOTHING
        "#,
        TENANT_RESIDENCE_TEST
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!(
        r#"
        INSERT INTO etablissements.etablissement
            (id, tenant_id, nom, fuseau_horaire, devise,
             juridiction, classement, etoiles, commune, adresse)
        VALUES ($1, $2, 'Résidence Test — hébergement seul', 'Africa/Abidjan', 'XOF',
                'CI', 'RESIDENCE_MEUBLEE', NULL, 'Abidjan', NULL)
        ON CONFLICT (id) DO UPDATE
        SET nom = EXCLUDED.nom,
            fuseau_horaire = EXCLUDED.fuseau_horaire,
            devise = EXCLUDED.devise,
            juridiction = EXCLUDED.juridiction,
            classement = EXCLUDED.classement,
            etoiles = EXCLUDED.etoiles,
            commune = EXCLUDED.commune,
            adresse = EXCLUDED.adresse
        "#,
        ETABLISSEMENT_RESIDENCE_TEST,
        TENANT_RESIDENCE_TEST
    )
    .execute(&mut *tx)
    .await?;

    // **HEBERGEMENT seul, AUCUNE capacité, AUCUN point de vente.**
    //
    // C'est la moitié la plus structurante du jeu de données : un établissement qui ne porte
    // qu'un service et rien d'autre doit être pleinement exploitable. Ajouter ici un point de
    // vente « pour faire complet » détruirait la seule preuve que le socle n'en suppose aucun.
    sqlx::query!(
        r#"
        INSERT INTO etablissements.etablissement_module
            (id, tenant_id, etablissement_id, module_code, module_implemente)
        VALUES ($1, $2, $3, 'HEBERGEMENT', true)
        ON CONFLICT (etablissement_id, module_code) DO NOTHING
        "#,
        SERVICE_RESIDENCE_TEST,
        TENANT_RESIDENCE_TEST,
        ETABLISSEMENT_RESIDENCE_TEST,
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    tracing::info!(tenant = %TENANT_RESIDENCE_TEST, "tenant Résidence Test seedé");
    Ok(())
}

/// **Les trois comptes du pilote** — CPT-00, CPT-01, CPT-02.
///
/// # Le mot de passe vient de l'environnement, jamais du code
///
/// `KAYA_SEEDS_MOT_DE_PASSE`. Un mot de passe littéral ici vivrait dans le dépôt, dans l'image et
/// dans les archives de tous les postes ayant cloné le projet — et il finirait employé sur un
/// serveur de démonstration joignable depuis internet.
///
/// # ★ Le condensat est VÉRIFIÉ, et réécrit seulement s'il ne concorde plus
///
/// Argon2 tire un sel aléatoire : deux exécutions produisent deux condensats différents pour le
/// même mot de passe. Le seed **ne compare donc pas des condensats** — il **vérifie** le condensat
/// existant contre `KAYA_SEEDS_MOT_DE_PASSE`. S'il concorde, rien n'est écrit, et l'état final est
/// identique à la troisième exécution comme à la première.
///
/// **S'il ne concorde pas, il est réécrit** — cycle 006, et ce n'est pas une préférence de style.
/// L'ancienne rédaction posait `DO NOTHING` au motif qu'un condensat est « une donnée de travail
/// dont la valeur exacte n'a pas d'importance ». C'est vrai de sa *valeur*, faux de ce qu'il
/// **ouvre** : sur une base où les comptes existaient d'une exécution antérieure, changer
/// `KAYA_SEEDS_MOT_DE_PASSE` ne changeait rien, et la connexion échouait sur « Identifiant ou mot
/// de passe incorrect » — message **indiscernable d'une faute de frappe** (FR-012). Le fichier de
/// seed déclarait pourtant le bon mot de passe, deux lignes plus haut.
///
/// C'est le raisonnement de `le_seed_applique_reellement_l_identite_qu_il_declare`, appliqué au
/// mot de passe : **un seed qui n'applique pas ce qu'il déclare donne un état faux**, et personne
/// ne pense à le soupçonner parce que le fichier dit la bonne chose.
///
/// # `DO NOTHING` sur les rôles, et pourquoi c'est le couple qui compte
///
/// L'unicité de `compte_role` porte sur `(compte_id, role_code, etablissement_id)` avec
/// `NULLS NOT DISTINCT`. Le conflit se résout donc sur ce couple, pas sur l'identifiant : un rôle
/// réattribué à l'identique ne crée pas de seconde ligne, même si l'on changeait son UUID.
async fn seeder_comptes_deloria(
    pool: &PgPool,
    mot_de_passe: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut tx = pool.begin().await?;
    tenant_context::poser_tenant(&mut tx, TENANT_DELORIA).await?;

    // `(personne, compte, nom, prénoms, identifiant)`
    let gens = [
        (
            PERSONNE_KOFFI,
            COMPTE_KOFFI,
            "Koffi",
            Some("Yao Bernard"),
            "koffi@deloria.test",
        ),
        (
            PERSONNE_ADJOUA,
            COMPTE_ADJOUA,
            "N'Guessan",
            Some("Adjoua"),
            "adjoua@deloria.test",
        ),
        (PERSONNE_YAO, COMPTE_YAO, "Kouassi", Some("Yao"), "yao@deloria.test"),
    ];

    for (personne_id, compte_id, nom, prenoms, identifiant) in gens {
        sqlx::query!(
            r#"
            INSERT INTO comptes.personne (id, tenant_id, nom, prenoms)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (id) DO UPDATE
            SET nom = EXCLUDED.nom,
                prenoms = EXCLUDED.prenoms,
                modifie_le = now()
            "#,
            personne_id,
            TENANT_DELORIA,
            nom,
            prenoms,
        )
        .execute(&mut *tx)
        .await?;

        // ★ **Le seed APPLIQUE le mot de passe qu'il déclare — mais seulement s'il ne l'est pas
        //    déjà.**
        //
        // Le condensat existant est **vérifié** contre `KAYA_SEEDS_MOT_DE_PASSE`. S'il concorde,
        // rien n'est écrit : le seed reste rejouable au sens strict, et l'objection d'origine —
        // *« recalculer un hachage à chaque exécution pour le jeter aussitôt serait du travail
        // pur »* — est respectée, une vérification coûtant ce qu'aurait coûté le hachage évité.
        //
        // ⚠️ **S'il ne concorde pas, il est RÉÉCRIT.** C'est le défaut trouvé au cycle 006, et il
        // a coûté une session : les comptes existaient d'une exécution antérieure, la variable
        // avait changé depuis, et la connexion échouait sur « Identifiant ou mot de passe
        // incorrect » — message **indiscernable d'une faute de frappe** (FR-012), sur une base
        // dont le fichier de seed déclarait pourtant le bon mot de passe deux lignes plus haut.
        // C'est exactement le raisonnement de `le_seed_applique_reellement_l_identite_qu_il_
        // declare` : un seed qui n'applique pas ce qu'il déclare donne un état faux, et personne
        // ne pense à le soupçonner parce que le fichier dit la bonne chose.
        let condensat_actuel: Option<String> = sqlx::query_scalar!(
            "SELECT condensat_mot_de_passe FROM comptes.compte WHERE id = $1",
            compte_id
        )
        .fetch_optional(&mut *tx)
        .await?;

        let a_reecrire = match &condensat_actuel {
            None => true,
            Some(condensat) => !kaya_comptes::authentification::verifier(condensat, mot_de_passe)
                .map(|v| v.valide)
                // Un condensat illisible est réécrit : le refuser laisserait un compte de
                // démonstration définitivement inaccessible.
                .unwrap_or(false),
        };

        if a_reecrire {
            let condensat = kaya_comptes::authentification::hacher(mot_de_passe)?;

            sqlx::query!(
                r#"
                INSERT INTO comptes.compte
                    (id, tenant_id, personne_id, identifiant_email, condensat_mot_de_passe)
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT (id) DO UPDATE
                SET condensat_mot_de_passe = EXCLUDED.condensat_mot_de_passe,
                    modifie_le = now()
                "#,
                compte_id,
                TENANT_DELORIA,
                personne_id,
                identifiant,
                condensat,
            )
            .execute(&mut *tx)
            .await?;

            if condensat_actuel.is_some() {
                tracing::warn!(
                    identifiant,
                    "le compte existait avec un AUTRE mot de passe — condensat réécrit depuis \
                     KAYA_SEEDS_MOT_DE_PASSE"
                );
            }
        }
    }

    // ── Le cumul ────────────────────────────────────────────────────────────────────────────
    //
    // `attribue_par_compte_id` désigne M. Koffi, y compris pour son propre rôle de propriétaire.
    // C'est une convention de seed, pas une règle : dans le produit, le premier propriétaire est
    // provisionné par l'éditeur (ETB-08). L'écrire ainsi évite une colonne nullable qui
    // signifierait « attribué par personne » et qu'il faudrait traiter partout.
    for (id, compte_id, role_code) in ROLES_DELORIA {
        sqlx::query!(
            r#"
            INSERT INTO comptes.compte_role
                (id, tenant_id, compte_id, role_code, etablissement_id, attribue_par_compte_id)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (compte_id, role_code, etablissement_id) DO NOTHING
            "#,
            id,
            TENANT_DELORIA,
            compte_id,
            role_code,
            ETABLISSEMENT_DELORIA,
            COMPTE_KOFFI,
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    tracing::info!(
        tenant = %TENANT_DELORIA,
        comptes = 3,
        roles = ROLES_DELORIA.len(),
        "comptes et rôles du pilote seedés — Adjoua en porte trois"
    );
    Ok(())
}

/// **Le parc de Deloria** — 17 unités, 6 catégories, 11 formules.
///
/// # Ce que ce seed pose, et l'ordre qui s'impose
///
/// `formule` référence `categorie`, `bareme_palier` et `plage_demi_journee` référencent `formule`,
/// et `temps_remise_en_etat` référence `categorie`. L'ordre d'écriture suit donc les clés
/// étrangères ; il n'y a aucun choix à faire, et une inversion échouerait bruyamment — ce qui est
/// préférable à un seed qui n'écrirait rien en silence.
///
/// # Les deux paramètres fiscaux, et ce qu'ils NE décident pas
///
/// La **nuitée est assujettie**, avec `une_nuitee_par_occupation` : un séjour de trois nuits vaut
/// 500 F, pas 3 × 500. Le **passage et la demi-journée ne le sont pas** — c'est un constat
/// d'exploitation du pilote, et le paramètre reste activable pour la commune qui l'imposerait.
///
/// **Ce n'est pas une décision fiscale.** `assujettie_taxe_nuitee` est un paramètre que ce crate
/// stocke et n'interprète jamais ; la règle vivra dans `JurisdictionAdapter` (P-12), et **B-02**
/// tranchera la valeur par défaut légale — jamais l'existence du paramètre.
///
/// **L'axe des personnes reste ouvert** : `une_nuitee_par_occupation` réduit trois nuits à une, et
/// ne dit rien de trois personnes. Une occupation de 3 nuits à 2 clients vaut 500 F ou 1 000 F, et
/// aucune source ne le dit — c'est **B-10**, à trancher avant le cycle SEJ. Le seed n'anticipe pas :
/// un multiplicateur posé à l'aveugle se retrouverait sur des factures et dans un état de
/// reversement communal.
async fn seeder_hebergement_deloria(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let mut tx = pool.begin().await?;
    tenant_context::poser_tenant(&mut tx, TENANT_DELORIA).await?;

    // ── Les cinq catégories de chambres, leurs unités, leurs deux formules ──────────────────
    for (rang, (categorie_id, nom, capacite, prix_nuitee, unites)) in
        CATEGORIES_CHAMBRES.iter().enumerate()
    {
        inserer_categorie(&mut tx, TENANT_DELORIA, ETABLISSEMENT_DELORIA, *categorie_id, nom, *capacite).await?;

        for (famille, duree) in REMISE_EN_ETAT_CHAMBRE {
            inserer_temps_remise_en_etat(&mut tx, TENANT_DELORIA, *categorie_id, famille, duree).await?;
        }

        for (unite_id, code) in *unites {
            inserer_unite(&mut tx, TENANT_DELORIA, ETABLISSEMENT_DELORIA, *unite_id, *categorie_id, code).await?;
        }

        // La nuitée — **le seul assujettissement du parc**.
        sqlx::query!(
            r#"
            INSERT INTO hebergement.formule
                (id, tenant_id, etablissement_id, categorie_id, famille, prix_mineur,
                 heure_arrivee_standard, heure_depart_standard,
                 assujettie_taxe_nuitee, regle_conversion_taxe)
            VALUES ($1, $2, $3, $4, 'NUITEE', $5, $6, $7, true, 'une_nuitee_par_occupation')
            ON CONFLICT (categorie_id, famille) DO NOTHING
            "#,
            FORMULES_NUITEE[rang],
            TENANT_DELORIA,
            ETABLISSEMENT_DELORIA,
            categorie_id,
            prix_nuitee,
            HEURE_ARRIVEE_STANDARD,
            HEURE_DEPART_STANDARD,
        )
        .execute(&mut *tx)
        .await?;

        // Le passage. `prix_mineur` est le **premier palier** — la table de barème fait foi, et
        // c'est elle que le moteur lit.
        //
        // `duree_max_minutes` reste **nul**, délibérément : la durée au-delà de laquelle un passage
        // change de formule est le paramètre `seuil_bascule_nuitee_minutes`, et le franchir doit
        // **basculer en nuitée**, pas refuser l'attribution. Recopier 480 ici produirait un refus
        // là où le produit doit facturer autrement.
        sqlx::query!(
            r#"
            INSERT INTO hebergement.formule
                (id, tenant_id, etablissement_id, categorie_id, famille, prix_mineur,
                 duree_min_minutes, prix_heure_supplementaire_mineur, assujettie_taxe_nuitee)
            VALUES ($1, $2, $3, $4, 'PASSAGE', $5, 60, $6, false)
            ON CONFLICT (categorie_id, famille) DO NOTHING
            "#,
            FORMULES_PASSAGE[rang],
            TENANT_DELORIA,
            ETABLISSEMENT_DELORIA,
            categorie_id,
            BAREME_PASSAGE[0].1,
            PRIX_HEURE_SUPPLEMENTAIRE_MINEUR,
        )
        .execute(&mut *tx)
        .await?;

        for (duree_minutes, prix_mineur) in BAREME_PASSAGE {
            sqlx::query!(
                r#"
                INSERT INTO hebergement.bareme_palier
                    (formule_id, duree_minutes, prix_mineur, tenant_id)
                VALUES ($1, $2, $3, $4)
                ON CONFLICT (formule_id, duree_minutes) DO NOTHING
                "#,
                FORMULES_PASSAGE[rang],
                duree_minutes,
                prix_mineur,
                TENANT_DELORIA,
            )
            .execute(&mut *tx)
            .await?;
        }
    }

    // ── La salle de réunion — une CATÉGORIE, pas une entité nouvelle ────────────────────────
    inserer_categorie(
        &mut tx,
        TENANT_DELORIA,
        ETABLISSEMENT_DELORIA,
        CATEGORIE_SALLE,
        "Salle de réunion",
        20,
    )
    .await?;
    inserer_temps_remise_en_etat(
        &mut tx,
        TENANT_DELORIA,
        CATEGORIE_SALLE,
        "DEMI_JOURNEE",
        REMISE_EN_ETAT_SALLE_MINUTES,
    )
    .await?;
    inserer_unite(
        &mut tx,
        TENANT_DELORIA,
        ETABLISSEMENT_DELORIA,
        UNITE_SALLE.0,
        CATEGORIE_SALLE,
        UNITE_SALLE.1,
    ).await?;

    sqlx::query!(
        r#"
        INSERT INTO hebergement.formule
            (id, tenant_id, etablissement_id, categorie_id, famille, prix_mineur,
             assujettie_taxe_nuitee)
        VALUES ($1, $2, $3, $4, 'DEMI_JOURNEE', $5, false)
        ON CONFLICT (categorie_id, famille) DO NOTHING
        "#,
        FORMULE_DEMI_JOURNEE,
        TENANT_DELORIA,
        ETABLISSEMENT_DELORIA,
        CATEGORIE_SALLE,
        PRIX_DEMI_JOURNEE_MINEUR,
    )
    .execute(&mut *tx)
    .await?;

    for (id, heure_debut, heure_fin, libelle_cle) in PLAGES_SALLE {
        sqlx::query!(
            r#"
            INSERT INTO hebergement.plage_demi_journee
                (id, formule_id, heure_debut, heure_fin, libelle_cle, tenant_id)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (formule_id, heure_debut, heure_fin) DO NOTHING
            "#,
            id,
            FORMULE_DEMI_JOURNEE,
            heure_debut,
            heure_fin,
            libelle_cle,
            TENANT_DELORIA,
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    let unites: usize = CATEGORIES_CHAMBRES.iter().map(|(_, _, _, _, u)| u.len()).sum();
    tracing::info!(
        tenant = %TENANT_DELORIA,
        unites = unites + 1,
        categories = CATEGORIES_CHAMBRES.len() + 1,
        "parc de Deloria seedé — 17 chambres, plus la salle de réunion"
    );
    Ok(())
}

/// Une catégorie. `DO NOTHING` sur l'identifiant **et** sur le nom, dont l'unicité est la
/// contrainte réelle.
async fn inserer_categorie(
    tx: &mut sqlx::PgTransaction<'_>,
    tenant_id: Uuid,
    etablissement_id: Uuid,
    id: Uuid,
    nom: &str,
    capacite_accueil: i16,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO hebergement.categorie
            (id, tenant_id, etablissement_id, nom, capacite_accueil)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (etablissement_id, nom) DO NOTHING
        "#,
        id,
        tenant_id,
        etablissement_id,
        nom,
        capacite_accueil,
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

/// Une unité. **`etage` est nul** : le plan d'étage de Deloria n'est pas relevé au cadrage, et
/// `NULL` dit « pas d'étage » là où `0` dirait « rez-de-chaussée » — deux faits différents.
async fn inserer_unite(
    tx: &mut sqlx::PgTransaction<'_>,
    tenant_id: Uuid,
    etablissement_id: Uuid,
    id: Uuid,
    categorie_id: Uuid,
    code: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO hebergement.unite
            (id, tenant_id, etablissement_id, categorie_id, code, etage)
        VALUES ($1, $2, $3, $4, $5, NULL)
        ON CONFLICT (etablissement_id, code) DO NOTHING
        "#,
        id,
        tenant_id,
        etablissement_id,
        categorie_id,
        code,
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn inserer_temps_remise_en_etat(
    tx: &mut sqlx::PgTransaction<'_>,
    tenant_id: Uuid,
    categorie_id: Uuid,
    famille_formule: &str,
    duree_minutes: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO hebergement.temps_remise_en_etat
            (categorie_id, famille_formule, duree_minutes, tenant_id)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (categorie_id, famille_formule) DO NOTHING
        "#,
        categorie_id,
        famille_formule,
        duree_minutes,
        tenant_id,
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

/// **Résidence Test — quatre meublés, au mois et à la nuitée.**
///
/// # Ce que ce jeu de données éprouve, et que Deloria ne peut pas éprouver
///
/// 1. **Aucune formule n'est réservée à un type d'établissement** (cadrage §5.2). Une résidence
///    meublée vend au mois **et** à la nuitée ; l'hôtel vend à la nuitée **et** au passage. Les deux
///    tenants portent donc des offres disjointes sur les mêmes tables, et un code qui aurait
///    supposé « le mensuel, c'est pour les résidences » n'a nulle part où se cacher.
/// 2. **Aucun code ne suppose l'existence du passage.** Ce tenant n'a ni formule `PASSAGE`, ni
///    barème, ni plage. Un chemin qui exigerait un barème pour lire une offre échouerait ici, et
///    nulle part ailleurs — ce qui est précisément la définition d'une porte à cible vide qu'on
///    vient de remplir.
///
/// # Deux valeurs absentes, et pourquoi elles le restent
///
/// **Le temps de remise en état du mensuel n'est pas déclaré.** Le cadrage §5.4 donne trois défauts
/// — passage 30 min, nuitée 2 h, demi-journée 1 h — et **aucun pour le mois**. `battement_minutes`
/// rend `0` quand aucune ligne n'existe, ce qui est visible et corrigible ; une valeur inventée ici
/// se lirait comme un constat, et le premier meublé loué au mois serait rendu disponible sans
/// ménage.
///
/// **La taxe de nuitée sur un séjour au mois n'est pas tranchée.** `assujettie_taxe_nuitee` est à
/// `false`, et ce n'est pas une décision fiscale : **B-02** vise le passage et la demi-journée, pas
/// le mensuel — la question n'est posée nulle part. Ce tenant n'émet aucun document, donc la valeur
/// n'a aucun effet ; elle est signalée ici pour qu'on ne la prenne pas pour un arbitrage rendu.
async fn seeder_hebergement_residence_test(
    pool: &PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut tx = pool.begin().await?;
    tenant_context::poser_tenant(&mut tx, TENANT_RESIDENCE_TEST).await?;

    inserer_categorie(
        &mut tx,
        TENANT_RESIDENCE_TEST,
        ETABLISSEMENT_RESIDENCE_TEST,
        CATEGORIE_MEUBLE,
        "Meublé deux pièces",
        4,
    )
    .await?;

    inserer_temps_remise_en_etat(&mut tx, TENANT_RESIDENCE_TEST, CATEGORIE_MEUBLE, "NUITEE", 120)
        .await?;

    for (unite_id, code) in UNITES_MEUBLE {
        inserer_unite(
            &mut tx,
            TENANT_RESIDENCE_TEST,
            ETABLISSEMENT_RESIDENCE_TEST,
            unite_id,
            CATEGORIE_MEUBLE,
            code,
        )
        .await?;
    }

    // La nuitée — **assujettie comme celle de Deloria**. La formule est la même chose des deux
    // côtés ; c'est l'établissement qui diffère, jamais la règle.
    sqlx::query!(
        r#"
        INSERT INTO hebergement.formule
            (id, tenant_id, etablissement_id, categorie_id, famille, prix_mineur,
             heure_arrivee_standard, heure_depart_standard,
             assujettie_taxe_nuitee, regle_conversion_taxe)
        VALUES ($1, $2, $3, $4, 'NUITEE', 18000, $5, $6, true, 'une_nuitee_par_occupation')
        ON CONFLICT (categorie_id, famille) DO NOTHING
        "#,
        FORMULE_MEUBLE_NUITEE,
        TENANT_RESIDENCE_TEST,
        ETABLISSEMENT_RESIDENCE_TEST,
        CATEGORIE_MEUBLE,
        HEURE_ARRIVEE_STANDARD,
        HEURE_DEPART_STANDARD,
    )
    .execute(&mut *tx)
    .await?;

    // Le mensuel. `duree_min_minutes` reste **nul** : « un mois » ne se convertit pas en minutes
    // sans choisir sa longueur, et 28, 30 ou 31 jours ne sont pas la même contrainte. Le poser à
    // 43 200 refuserait un mois de février.
    sqlx::query!(
        r#"
        INSERT INTO hebergement.formule
            (id, tenant_id, etablissement_id, categorie_id, famille, prix_mineur,
             assujettie_taxe_nuitee)
        VALUES ($1, $2, $3, $4, 'MENSUEL', 250000, false)
        ON CONFLICT (categorie_id, famille) DO NOTHING
        "#,
        FORMULE_MEUBLE_MENSUEL,
        TENANT_RESIDENCE_TEST,
        ETABLISSEMENT_RESIDENCE_TEST,
        CATEGORIE_MEUBLE,
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    tracing::info!(
        tenant = %TENANT_RESIDENCE_TEST,
        unites = UNITES_MEUBLE.len(),
        "Résidence Test seedée — quatre meublés, mois et nuitée, AUCUN passage"
    );
    Ok(())
}

/// **Les valeurs de configuration que les migrations `0023` et `0028` ont promises aux seeds.**
///
/// Le catalogue déclare qu'une clé existe, son type et jusqu'où elle se surcharge ; il ne pose
/// aucune valeur par défaut (principe I·c). Sans ce seed, les cinq clés — trois de HEB, deux de
/// SYN — existeraient sans qu'aucun établissement ne les renseigne, et la configuration
/// d'établissement montrerait cinq lignes vides sur le tenant du pilote.
///
/// La portée est l'**établissement** : `etablissement_module_id` et `point_de_vente_id` restent
/// nuls, et la contrainte `parametre_configuration_une_seule_portee` garantit qu'on ne peut pas en
/// renseigner deux.
async fn seeder_parametres_deloria(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let mut tx = pool.begin().await?;
    tenant_context::poser_tenant(&mut tx, TENANT_DELORIA).await?;

    for (id, cle, valeur) in PARAMETRES_DELORIA {
        let valeur: serde_json::Value = serde_json::from_str(valeur)?;

        sqlx::query!(
            r#"
            INSERT INTO etablissements.parametre_configuration
                (id, tenant_id, etablissement_id, cle, valeur)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (tenant_id, etablissement_id, etablissement_module_id,
                         point_de_vente_id, cle)
            DO NOTHING
            "#,
            id,
            TENANT_DELORIA,
            ETABLISSEMENT_DELORIA,
            cle,
            valeur,
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    tracing::info!(
        tenant = %TENANT_DELORIA,
        parametres = PARAMETRES_DELORIA.len(),
        "valeurs de configuration posées — HEB : 14 h, 12 h, bascule 480 min ; SYN : dérive 300 s, latence 3 000 ms"
    );
    Ok(())
}


// ===============================================================================================
//  SEJ — les fiches clientes et les séjours de démonstration (cycle 006)
// ===============================================================================================
//
// # ⚠️ JAMAIS PAR MIGRATION, et c'est le piège de tout ce fichier
//
// Une table en `FORCE ROW LEVEL SECURITY` accepte un `INSERT` de migration **en n'écrivant rien**,
// sans erreur : une migration n'a pas de tenant courant, sa politique d'isolation ne trouve donc
// aucune ligne à laisser passer, et `INSERT 0` ne lève pas. Le seed pose `app.current_tenant`
// avant chaque transaction, et c'est ce qui le rend possible.
//
// # Ce que ce jeu de données rend VISIBLE, et qu'aucune autre donnée ne montrerait
//
// Le séjour clos porte son **constat de taxe figé** — avec `nuitees_assujetties` et
// `montant_mineur` à **`NULL`**. C'est ce qui rend lisible, en base et sans lire une ligne de
// code, que **ce cycle a laissé le calcul à FIS-03** : il fige des **faits** et un paramétrage
// recopié, il ne dérive aucune assiette. Un jeu de données qui aurait « rempli » ces deux colonnes
// pour faire joli aurait fait croire à un calcul qui n'existe pas.
//
// # Les instants sont RELATIFS À `now()`, et le seed reste rejouable
//
// « Un séjour en cours » n'a de sens que par rapport à maintenant : des dates figées auraient été
// « en cours » le jour de leur écriture et closes le lendemain. La rejouabilité tient à autre
// chose : **tout est en `ON CONFLICT DO NOTHING` sur des identifiants littéraux**. Une seconde
// exécution n'écrit rien — pas même une occupation, dont la contrainte d'exclusion est couverte
// par un `ON CONFLICT` **sans cible**, seule forme qui attrape aussi les violations d'exclusion.

/// `(personne_id, client_id_est_le_meme, nom, prénoms, téléphone)`.
///
/// **Douze fiches**, assez pour que la recherche par nom rende plusieurs résultats et que la
/// troncature se voie. Les diacritiques et l'apostrophe sont **volontaires** : `nom_repli` les
/// replie, et c'est ce qui fait que « Kone » retrouve « Koné » et que les deux apostrophes
/// retrouvent la même fiche.
///
/// ⚠️ **Aucune pièce d'identité.** Le numéro est chiffré par `CoffreTenant`, dont la clé vit hors
/// de la base (`KAYA_COFFRE_CLE`) : en écrire un ici ferait dépendre le seed d'un secret, et un
/// seed qui échoue faute de secret est un seed que personne ne lance. Le parcours de capture de
/// pièce s'éprouve par les tests d'intégration, qui ont le coffre.
const CLIENTS_DELORIA: [(Uuid, &str, Option<&str>, Option<&str>); 12] = [
    (uuid!("0198c4a0-0000-7000-8000-000000000401"), "Bakayoko", Some("Adama"), Some("+2250707123456")),
    (uuid!("0198c4a0-0000-7000-8000-000000000402"), "Koné", Some("Aminata"), Some("+2250101223344")),
    (uuid!("0198c4a0-0000-7000-8000-000000000403"), "Kouassi", Some("Yao"), Some("+2250505667788")),
    (uuid!("0198c4a0-0000-7000-8000-000000000404"), "N'Guessan", Some("Affoué"), Some("+2250747889900")),
    (uuid!("0198c4a0-0000-7000-8000-000000000405"), "Traoré", Some("Ibrahim"), Some("+2250102030405")),
    (uuid!("0198c4a0-0000-7000-8000-000000000406"), "Diabaté", Some("Fatoumata"), Some("+2250708091011")),
    (uuid!("0198c4a0-0000-7000-8000-000000000407"), "Yapi", Some("Serge"), Some("+2250544332211")),
    (uuid!("0198c4a0-0000-7000-8000-000000000408"), "Ouattara", Some("Salif"), Some("+2250766554433")),
    (uuid!("0198c4a0-0000-7000-8000-000000000409"), "Gnahoré", Some("Marie"), None),
    (uuid!("0198c4a0-0000-7000-8000-00000000040a"), "Konan", Some("Brou"), Some("+2250122334455")),
    (uuid!("0198c4a0-0000-7000-8000-00000000040b"), "Assamoi", None, Some("+2250799887766")),
    (uuid!("0198c4a0-0000-7000-8000-00000000040c"), "Zadi", Some("Nadège"), Some("+2250155667788")),
];

/// Deux fiches sur Résidence Test — **assez pour prouver l'isolation**, pas plus.
///
/// Un test d'isolation qui n'aurait qu'un tenant peuplé ne distinguerait pas « la politique RLS
/// filtre » de « l'autre tenant est vide ».
const CLIENTS_RESIDENCE_TEST: [(Uuid, &str, Option<&str>, Option<&str>); 2] = [
    (uuid!("0198c4a0-0000-7000-8000-000000000451"), "Sanogo", Some("Mariam"), Some("+2250611223344")),
    (uuid!("0198c4a0-0000-7000-8000-000000000452"), "Coulibaly", Some("Drissa"), Some("+2250688776655")),
];

// Les trois séjours de Deloria — chacun montre une chose qu'aucun autre ne montre.
const SEJOUR_NUITEE: Uuid = uuid!("0198c4a0-0000-7000-8000-000000000411");
const SEJOUR_PASSAGE: Uuid = uuid!("0198c4a0-0000-7000-8000-000000000412");
const SEJOUR_CLOS: Uuid = uuid!("0198c4a0-0000-7000-8000-000000000413");
const SEJOUR_RESIDENCE: Uuid = uuid!("0198c4a0-0000-7000-8000-000000000461");

/// Une prestation à seeder : tout ce qu'un séjour porte, en une structure.
struct SejourSeed {
    sejour_id: Uuid,
    tenant_id: Uuid,
    etablissement_id: Uuid,
    client_id: Option<Uuid>,
    unite_id: Uuid,
    formule_id: Uuid,
    /// Décalage du début par rapport à `now()`, en minutes. **Négatif = dans le passé.**
    debut_minutes: i64,
    fin_minutes: i64,
    /// Battement de remise en état, en minutes — il entre dans `periode`, jamais dans les bornes
    /// commerciales. Le client ne paie pas le ménage.
    remise_en_etat_minutes: i64,
    /// Numéro de fiche de police, **continu par établissement**.
    numero_fiche: i64,
    quantite: &'static str,
    prix_unitaire_mineur: i64,
    montant_mineur: i64,
    devise: &'static str,
    /// `true` pour le séjour clos : note arrêtée, occupation libérée, constat figé.
    clos: bool,
}

/// Les fiches clientes de Deloria et de Résidence Test.
async fn seeder_clients(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    for (tenant_id, fiches) in [
        (TENANT_DELORIA, &CLIENTS_DELORIA[..]),
        (TENANT_RESIDENCE_TEST, &CLIENTS_RESIDENCE_TEST[..]),
    ] {
        let mut tx = pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, tenant_id).await?;

        for (id, nom, prenoms, telephone) in fiches {
            // ★ **`nom_repli` vient de la MÊME fonction que le produit**, jamais d'un
            // `lower()` écrit ici. Deux replis divergents rendraient introuvables des fiches
            // pourtant créées — et la divergence ne se verrait qu'à la recherche.
            let nom_repli = kaya_comptes::client::repli(nom);
            let telephone_repli = telephone.map(kaya_comptes::client::repli_telephone);

            sqlx::query!(
                r#"
                INSERT INTO comptes.personne
                    (id, tenant_id, nom, prenoms, telephone, nom_repli, telephone_repli)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                ON CONFLICT (id) DO NOTHING
                "#,
                id,
                tenant_id,
                nom,
                prenoms.as_deref(),
                telephone.as_deref(),
                nom_repli,
                telephone_repli,
            )
            .execute(&mut *tx)
            .await?;

            // La qualification cliente — **une fiche client EST une personne qualifiée**, jamais
            // une seconde identité. Le personnel n'apparaît donc jamais dans la recherche.
            sqlx::query!(
                r#"
                INSERT INTO comptes.client (personne_id, tenant_id)
                VALUES ($1, $2)
                ON CONFLICT (personne_id) DO NOTHING
                "#,
                id,
                tenant_id,
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
    }

    // Une préférence sur la première fiche — **append-only**, et c'est ce que `R5` affiche.
    let mut tx = pool.begin().await?;
    tenant_context::poser_tenant(&mut tx, TENANT_DELORIA).await?;
    sqlx::query!(
        r#"
        INSERT INTO comptes.preference_personne (id, tenant_id, personne_id, texte)
        VALUES ($1, $2, $3, 'Chambre calme, étage bas.')
        ON CONFLICT (id) DO NOTHING
        "#,
        uuid!("0198c4a0-0000-7000-8000-000000000431"),
        TENANT_DELORIA,
        CLIENTS_DELORIA[0].0,
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    tracing::info!(
        deloria = CLIENTS_DELORIA.len(),
        residence_test = CLIENTS_RESIDENCE_TEST.len(),
        "fiches clientes seedées"
    );
    Ok(())
}

/// Les quatre séjours de démonstration.
async fn seeder_sejours(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let seeds = [
        // 1 · **Nuitée en cours** — deux nuits, deux accompagnants. Chambre B3.
        SejourSeed {
            sejour_id: SEJOUR_NUITEE,
            tenant_id: TENANT_DELORIA,
            etablissement_id: ETABLISSEMENT_DELORIA,
            client_id: Some(CLIENTS_DELORIA[0].0),
            unite_id: uuid!("0198c4a0-0000-7000-8000-000000000213"),
            formule_id: FORMULES_NUITEE[1],
            debut_minutes: -24 * 60,
            fin_minutes: 24 * 60,
            remise_en_etat_minutes: 120,
            numero_fiche: 1,
            quantite: "2",
            prix_unitaire_mineur: 15_500,
            montant_mineur: 31_000,
            devise: "XOF",
            clos: false,
        },
        // 2 · **Passage en cours** — deux heures, SANS fiche cliente. La pièce vient après la
        //     clé (FR-023) : la fiche de police est numérotée et déclarée incomplète.
        SejourSeed {
            sejour_id: SEJOUR_PASSAGE,
            tenant_id: TENANT_DELORIA,
            etablissement_id: ETABLISSEMENT_DELORIA,
            client_id: None,
            unite_id: uuid!("0198c4a0-0000-7000-8000-000000000201"),
            formule_id: FORMULES_PASSAGE[0],
            debut_minutes: -30,
            fin_minutes: 90,
            remise_en_etat_minutes: 30,
            numero_fiche: 2,
            quantite: "1",
            prix_unitaire_mineur: 2_800,
            montant_mineur: 2_800,
            devise: "XOF",
            clos: false,
        },
        // 3 · **Séjour CLOS avec son constat figé** — le seul qui montre ce que le cycle a laissé
        //     à FIS-03.
        SejourSeed {
            sejour_id: SEJOUR_CLOS,
            tenant_id: TENANT_DELORIA,
            etablissement_id: ETABLISSEMENT_DELORIA,
            client_id: Some(CLIENTS_DELORIA[4].0),
            unite_id: uuid!("0198c4a0-0000-7000-8000-000000000241"),
            formule_id: FORMULES_NUITEE[4],
            debut_minutes: -5 * 24 * 60,
            fin_minutes: -3 * 24 * 60,
            remise_en_etat_minutes: 120,
            numero_fiche: 3,
            quantite: "2",
            prix_unitaire_mineur: 25_500,
            montant_mineur: 51_000,
            devise: "XOF",
            clos: true,
        },
        // 4 · Résidence Test — **un meublé à la nuitée**, pour que l'isolation ait deux côtés.
        SejourSeed {
            sejour_id: SEJOUR_RESIDENCE,
            tenant_id: TENANT_RESIDENCE_TEST,
            etablissement_id: ETABLISSEMENT_RESIDENCE_TEST,
            client_id: Some(CLIENTS_RESIDENCE_TEST[0].0),
            unite_id: UNITES_MEUBLE[0].0,
            formule_id: FORMULE_MEUBLE_NUITEE,
            debut_minutes: -12 * 60,
            fin_minutes: 36 * 60,
            remise_en_etat_minutes: 0,
            numero_fiche: 1,
            quantite: "2",
            prix_unitaire_mineur: 18_000,
            montant_mineur: 36_000,
            devise: "XOF",
            clos: false,
        },
    ];

    for seed in seeds {
        seeder_un_sejour(pool, &seed).await?;
    }

    // Les deux accompagnants du séjour de nuitée — **un nom suffit** (FR-015).
    let mut tx = pool.begin().await?;
    tenant_context::poser_tenant(&mut tx, TENANT_DELORIA).await?;
    for (id, nom) in [
        (uuid!("0198c4a0-0000-7000-8000-000000000421"), "Aya"),
        (uuid!("0198c4a0-0000-7000-8000-000000000422"), "Konan"),
    ] {
        sqlx::query!(
            r#"
            INSERT INTO hebergement.accompagnant (id, tenant_id, sejour_id, nom)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (id) DO NOTHING
            "#,
            id,
            TENANT_DELORIA,
            SEJOUR_NUITEE,
            nom,
        )
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;

    tracing::info!("quatre séjours de démonstration seedés — dont un clos, avec son constat figé");
    Ok(())
}

/// Un séjour, son occupation, sa note, sa ligne, sa fiche de police — **et son constat s'il est
/// clos**.
///
/// Une transaction par séjour, comme le produit : cinq écritures groupées.
async fn seeder_un_sejour(
    pool: &PgPool,
    seed: &SejourSeed,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut tx = pool.begin().await?;
    tenant_context::poser_tenant(&mut tx, seed.tenant_id).await?;

    sqlx::query!(
        r#"
        INSERT INTO hebergement.sejour
            (id, tenant_id, etablissement_id, client_id, statut, ouvert_le, clos_le)
        VALUES ($1, $2, $3, $4,
                CASE WHEN $5 THEN 'clos' ELSE 'en_cours' END,
                now() + make_interval(mins => $6::INT),
                CASE WHEN $5 THEN now() + make_interval(mins => $7::INT) ELSE NULL END)
        ON CONFLICT DO NOTHING
        "#,
        seed.sejour_id,
        seed.tenant_id,
        seed.etablissement_id,
        seed.client_id,
        seed.clos,
        seed.debut_minutes as i32,
        seed.fin_minutes as i32,
    )
    .execute(&mut *tx)
    .await?;

    // L'occupation. **`periode` inclut la remise en état, les bornes commerciales non** — le
    // client ne paie pas le ménage, mais la chambre n'est pas attribuable pendant.
    let occupation_id = deriver_seed(seed.sejour_id, 0x01);
    sqlx::query!(
        r#"
        INSERT INTO hebergement.occupation
            (id, tenant_id, etablissement_id, unite_id, formule_id, sejour_id,
             periode, debut_client, fin_client, statut, libere_le)
        VALUES ($1, $2, $3, $4, $5, $6,
                tstzrange(now() + make_interval(mins => $7::INT),
                          now() + make_interval(mins => $8::INT), '[)'),
                now() + make_interval(mins => $7::INT),
                now() + make_interval(mins => $9::INT),
                CASE WHEN $10 THEN 'liberee' ELSE 'active' END,
                CASE WHEN $10 THEN now() + make_interval(mins => $9::INT) ELSE NULL END)
        ON CONFLICT DO NOTHING
        "#,
        occupation_id,
        seed.tenant_id,
        seed.etablissement_id,
        seed.unite_id,
        seed.formule_id,
        seed.sejour_id,
        seed.debut_minutes as i32,
        (seed.fin_minutes + seed.remise_en_etat_minutes) as i32,
        seed.fin_minutes as i32,
        seed.clos,
    )
    .execute(&mut *tx)
    .await?;

    // La note. **Aucune colonne de total** : le total est la somme des lignes, calculée à la
    // lecture. Une colonne se désynchroniserait au premier ajustement.
    let note_id = deriver_seed(seed.sejour_id, 0x02);
    sqlx::query!(
        r#"
        INSERT INTO hebergement.note_sejour (id, tenant_id, sejour_id, devise, statut, arretee_le)
        VALUES ($1, $2, $3, $4,
                CASE WHEN $5 THEN 'arretee' ELSE 'ouverte' END,
                CASE WHEN $5 THEN now() + make_interval(mins => $6::INT) ELSE NULL END)
        ON CONFLICT DO NOTHING
        "#,
        note_id,
        seed.tenant_id,
        seed.sejour_id,
        seed.devise,
        seed.clos,
        seed.fin_minutes as i32,
    )
    .execute(&mut *tx)
    .await?;

    // La ligne d'hébergement. **`quantite` est un `NUMERIC`** (porte P-10) : une bière se vend à
    // l'unité, du fer au mètre. Le libellé est une **clé i18n**, jamais une chaîne rendue.
    sqlx::query!(
        r#"
        INSERT INTO hebergement.ligne_sejour
            (id, tenant_id, note_id, occupation_id, nature, libelle_cle,
             quantite, prix_unitaire_mineur, montant_mineur, devise,
             periode_debut, periode_fin)
        VALUES ($1, $2, $3, $4, 'hebergement', 'hebergement.note.ligne.hebergement',
                $5::TEXT::NUMERIC, $6, $7, $8,
                now() + make_interval(mins => $9::INT),
                now() + make_interval(mins => $10::INT))
        ON CONFLICT DO NOTHING
        "#,
        deriver_seed(seed.sejour_id, 0x03),
        seed.tenant_id,
        note_id,
        occupation_id,
        seed.quantite,
        seed.prix_unitaire_mineur,
        seed.montant_mineur,
        seed.devise,
        seed.debut_minutes as i32,
        seed.fin_minutes as i32,
    )
    .execute(&mut *tx)
    .await?;

    // Le compteur de fiches — **par établissement, et jamais une `SEQUENCE`** : une séquence est
    // globale au schéma et laisse des trous, deux propriétés fatales à une numérotation continue.
    sqlx::query!(
        r#"
        INSERT INTO hebergement.numerotation_fiche_police
            (tenant_id, etablissement_id, dernier_numero)
        VALUES ($1, $2, $3)
        ON CONFLICT (tenant_id, etablissement_id) DO UPDATE
        SET dernier_numero = GREATEST(hebergement.numerotation_fiche_police.dernier_numero,
                                      EXCLUDED.dernier_numero)
        "#,
        seed.tenant_id,
        seed.etablissement_id,
        seed.numero_fiche,
    )
    .execute(&mut *tx)
    .await?;

    // La fiche de police. **Elle ne recopie AUCUNE identité** : les noms viennent du client et des
    // accompagnants. Sans client rattaché, elle est **déclarée incomplète** — jamais fabriquée
    // avec « M. X », qui serait un document légal faux.
    sqlx::query!(
        r#"
        INSERT INTO hebergement.fiche_police
            (id, tenant_id, etablissement_id, sejour_id, numero, complete, completee_le)
        VALUES ($1, $2, $3, $4, $5, $6,
                CASE WHEN $6 THEN now() + make_interval(mins => $7::INT) ELSE NULL END)
        ON CONFLICT DO NOTHING
        "#,
        deriver_seed(seed.sejour_id, 0x04),
        seed.tenant_id,
        seed.etablissement_id,
        seed.sejour_id,
        seed.numero_fiche,
        seed.client_id.is_some(),
        seed.debut_minutes as i32,
    )
    .execute(&mut *tx)
    .await?;

    // ★ Le constat de taxe — **seulement sur le séjour clos**, et `nuitees_assujetties` /
    // `montant_mineur` restent à `NULL`. C'est ce qui rend visible, en base, que ce cycle fige des
    // FAITS et un paramétrage recopié, sans dériver d'assiette : la règle fiscale vit dans
    // `JurisdictionAdapter`, et son test doré appartient à FIS-03.
    if seed.clos {
        sqlx::query!(
            r#"
            INSERT INTO hebergement.taxe_sejour_constat
                (id, tenant_id, etablissement_id, sejour_id,
                 nuits_constatees, nombre_personnes, periode_debut, periode_fin,
                 formule_id, famille_formule, assujettie_taxe_nuitee, regle_conversion_taxe,
                 classement_etablissement, commune)
            SELECT $1, $2, $3, $4, $5, $6,
                   now() + make_interval(mins => $7::INT),
                   now() + make_interval(mins => $8::INT),
                   $9, 'NUITEE', true, 'une_nuitee_par_occupation',
                   -- ★ **RECOPIÉ, jamais référencé.** Le constat doit rester vrai le jour où
                   -- l'établissement change de classement : une clé étrangère rendrait le passé
                   -- réécrivable, et un contrôle fiscal porte sur ce qui était vrai ce jour-là.
                   e.classement, e.commune
            FROM etablissements.etablissement e
            WHERE e.id = $3
            ON CONFLICT DO NOTHING
            "#,
            deriver_seed(seed.sejour_id, 0x06),
            seed.tenant_id,
            seed.etablissement_id,
            seed.sejour_id,
            2,
            1,
            seed.debut_minutes as i32,
            seed.fin_minutes as i32,
            seed.formule_id,
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

/// Dérive un identifiant fille depuis celui du séjour — **la même règle que le produit**.
///
/// Trois octets de discriminant XORés en fin d'UUID. Le seed la reproduit pour que les
/// identifiants de la note, de la ligne et de la fiche soient **exactement** ceux qu'un rejeu du
/// produit produirait : un seed qui tirerait des identifiants neufs rendrait la démonstration
/// impossible à recouper avec ce que l'application écrit.
fn deriver_seed(source: Uuid, discriminant: u8) -> Uuid {
    let mut octets = *source.as_bytes();
    octets[13] ^= discriminant;
    octets[14] ^= discriminant.wrapping_mul(3);
    octets[15] ^= discriminant.wrapping_mul(7);
    Uuid::from_bytes(octets)
}
