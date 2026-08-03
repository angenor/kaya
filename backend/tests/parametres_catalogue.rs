//! **Porte de cohérence documentaire** — toute clé du catalogue figure au « Récapitulatif des
//! paramètres d'établissement ».
//!
//! # Ce qu'elle rend vérifiable
//!
//! Le principe I·c pose que « le récapitulatif des paramètres fait foi ». Sans cette porte, c'est
//! une phrase : rien n'empêcherait d'ajouter une clé au catalogue sans l'y inscrire, et le
//! récapitulatif deviendrait faux **sans que personne ne s'en aperçoive** — jusqu'au jour où un
//! exploitant chercherait pourquoi un réglage documenté nulle part change le comportement de son
//! établissement.
//!
//! # Le sens de la comparaison n'est PAS symétrique
//!
//! **Catalogue → récapitulatif**, comme `classes_offline.rs` va de la table vers le registre :
//!
//! - une **clé du catalogue absente du récapitulatif** est une erreur — c'est ce qu'on attrape ;
//! - une **ligne du récapitulatif sans clé** est normale : le récapitulatif décrit tout le produit,
//!   y compris les trente-cinq paramètres que HEB, FIS, CAI, RSV, QRC, CPT, SYN, STK et ADM
//!   livreront.
//!
//! Comparer dans les deux sens ferait échouer la porte sur l'essentiel du récapitulatif, et elle
//! serait désactivée dans la semaine.
//!
//! # Ce que cette porte n'inspecte pas
//!
//! **La justesse de la valeur documentée.** Que `politique_impression` ait tel ou tel jeu de
//! valeurs relève du cycle qui la définit ; ce qui est vérifié ici est que la clé **a été
//! inscrite** — c'est-à-dire que quelqu'un a ouvert le récapitulatif au moment de créer la clé.

mod commun;

use std::collections::BTreeSet;

use sqlx::Row;

/// Le récapitulatif, lu à la compilation. Une modification du fichier recompile le test.
const USER_STORIES: &str = include_str!("../../docs/user-stories-v1.md");

/// Extrait les clés techniques citées au récapitulatif, entre accents graves.
///
/// La convention est la même qu'au registre des classes hors-ligne : le libellé est en français
/// pour le lecteur, la clé technique entre accents graves pour la porte. Une ligne sans clé reste
/// parfaitement valide — c'est le cas des trente-cinq paramètres à venir.
fn cles_du_recapitulatif() -> BTreeSet<String> {
    let mut cles = BTreeSet::new();

    let Some(debut) = USER_STORIES.find("## Récapitulatif des paramètres d'établissement") else {
        panic!(
            "la section « Récapitulatif des paramètres d'établissement » a disparu de \
             docs/user-stories-v1.md. C'est la source que le principe I·c désigne comme faisant \
             foi ; sans elle, cette porte n'a plus rien à comparer."
        );
    };

    // La section s'arrête au titre suivant de même niveau.
    let section = &USER_STORIES[debut..];
    let fin = section[3..].find("\n## ").map(|i| i + 3).unwrap_or(section.len());
    let section = &section[..fin];

    for ligne in section.lines() {
        let ligne = ligne.trim();
        if !ligne.starts_with('|') {
            continue;
        }
        let mut reste = ligne;
        while let Some(d) = reste.find('`') {
            let apres = &reste[d + 1..];
            let Some(f) = apres.find('`') else { break };
            let brut = apres[..f].trim();
            reste = &apres[f + 1..];
            // **Le point est admis depuis le cycle 005.** Les huit premières clés du produit
            // n'ont aucun préfixe — `heure_arrivee_standard`, `mot_de_passe_longueur_min` —
            // parce qu'aucune ne risquait de collision. SYN en introduit deux dont le nom est
            // générique par nature : un `latence_degradee_seuil_ms` nu serait revendiqué par le
            // premier autre module qui mesure une latence. Le catalogue est un référentiel
            // **unique du produit**, partagé par tous les modules ; c'est là que les préfixes
            // deviennent utiles, et l'extraction doit les voir sous peine de rendre la porte
            // muette sur toute clé préfixée.
            if !brut.is_empty()
                && brut
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
            {
                cles.insert(brut.to_lowercase());
            }
        }
    }

    cles
}

async fn cles_du_catalogue(pool: &sqlx::PgPool) -> BTreeSet<String> {
    sqlx::query("SELECT cle FROM etablissements.parametre_catalogue")
        .fetch_all(pool)
        .await
        .expect("lecture du catalogue")
        .into_iter()
        .map(|l| l.get::<String, _>("cle").to_lowercase())
        .collect()
}

/// **Toute clé du catalogue figure au récapitulatif.**
#[tokio::test]
async fn toute_cle_du_catalogue_figure_au_recapitulatif() {
    let pool = commun::pool_owner().await;
    let catalogue = cles_du_catalogue(&pool).await;
    let recapitulatif = cles_du_recapitulatif();

    assert!(
        !catalogue.is_empty(),
        "le catalogue est vide — la porte n'a rien vérifié. Base non migrée ?"
    );

    let absentes: Vec<&String> = catalogue.difference(&recapitulatif).collect();

    assert!(
        absentes.is_empty(),
        "{} clé(s) du catalogue absente(s) du « Récapitulatif des paramètres d'établissement » de \
         docs/user-stories-v1.md :\n  {}\n\n\
         Le principe I·c pose que le récapitulatif fait foi. Une clé qui n'y figure pas est un \
         paramètre dont l'existence n'est écrite nulle part — et personne ne saura qu'il faut le \
         régler. L'inscription se fait dans le MÊME changement que l'ajout au catalogue, avec la \
         clé technique entre accents graves pour que cette porte la retrouve.",
        absentes.len(),
        absentes
            .iter()
            .map(|c| c.as_str())
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}

/// **Test négatif** — une clé absente du récapitulatif est bien signalée.
///
/// Exercé sur un ensemble simulé : créer une vraie entrée de catalogue ferait échouer le test
/// ci-dessus au hasard de l'ordonnancement, les fichiers de test s'exécutant en parallèle sur une
/// base partagée.
#[test]
fn test_negatif_une_cle_absente_du_recapitulatif_est_signalee() {
    let recapitulatif = cles_du_recapitulatif();

    assert!(
        recapitulatif.contains("politique_impression"),
        "« politique_impression » n'est pas au récapitulatif alors que le cycle 002 l'ajoute au \
         catalogue. L'extraction est-elle cassée, ou la ligne a-t-elle disparu ? Clés \
         extraites : {recapitulatif:?}"
    );

    let mut catalogue_simule = BTreeSet::new();
    catalogue_simule.insert("politique_impression".to_owned()); // au récapitulatif
    catalogue_simule.insert("cle_jamais_documentee".to_owned()); // absente

    let absentes: Vec<&String> = catalogue_simule.difference(&recapitulatif).collect();

    assert!(
        absentes.iter().any(|c| c.as_str() == "cle_jamais_documentee"),
        "la porte n'a pas signalé une clé absente du récapitulatif : elle ne protège rien"
    );
    assert!(
        !absentes.iter().any(|c| c.as_str() == "politique_impression"),
        "la porte signale une clé pourtant documentée : elle échouerait sur tout"
    );
}

/// Le catalogue contient **exactement** les clés annoncées par les cycles livrés, avec leurs
/// attributs.
///
/// # Pourquoi figer le décompte plutôt que se contenter de vérifier les présences
///
/// Une clé de configuration engage le récapitulatif du principe I·c et **tous les cycles qui la
/// liront**. Une clé ajoutée sans décision doit donc se voir. Le cycle 002 figeait le total à 1 ;
/// le cycle 003 le porte à 6, et le prochain cycle qui en ajoutera une devra passer ici — c'est
/// exactement le moment où la question « cette clé est-elle vraiment un paramètre ? » se pose.
#[tokio::test]
async fn le_catalogue_contient_exactement_les_cles_des_cycles_livres() {
    let pool = commun::pool_owner().await;

    let attendues: BTreeSet<String> = [
        // ETB-03
        "politique_impression",
        // CPT-01 — les cinq du cycle 003
        "indicatif_telephonique_defaut",
        "methode_authentification",
        "mot_de_passe_longueur_min",
        "jeton_acces_duree_min",
        "jeton_rafraichissement_duree_jours",
        // HEB-03 / HEB-04 — les trois du cycle 004. Le cycle en ajoute trois et **pas sept** :
        // temps de remise en état, barème de passage et plages de demi-journée sont des
        // référentiels en table, pas des scalaires d'établissement. Le motif est écrit dans
        // `0023_parametres_hebergement.sql` et au récapitulatif de `user-stories-v1.md`.
        "heure_arrivee_standard",
        "heure_depart_standard",
        "seuil_bascule_nuitee_minutes",
        // SYN-02 / SYN-04 — les deux du cycle 005, et **les premières à porter un préfixe de
        // module**. Le catalogue est un référentiel unique du produit : `latence_degradee_seuil_ms`
        // sans préfixe serait revendiqué par le premier autre module qui mesure une latence.
        "sync.derive_horloge_seuil_secondes",
        "sync.latence_degradee_seuil_ms",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();

    let reelles = cles_du_catalogue(&pool).await;

    assert_eq!(
        reelles, attendues,
        "le catalogue diverge des clés annoncées par les cycles livrés. Une clé ajoutée sans \
         décision engage le récapitulatif du principe I·c et tous les cycles qui la liront ; une \
         clé disparue casse la configuration des bases déjà déployées."
    );
}

/// **La portée la plus basse dit où le paramètre se règle**, et ce n'est pas une donnée
/// décorative : c'est elle qui décide si la clé est une colonne de table ou une valeur de
/// configuration.
#[tokio::test]
async fn les_portees_des_parametres_livres_sont_celles_qui_ont_ete_decidees() {
    let pool = commun::pool_owner().await;

    // (clé, portée la plus basse, type, story)
    let attendus = [
        // ETB-03 — research.md R-04 : elle se règle jusqu'au point de vente, c'est la raison pour
        // laquelle elle n'est PAS une colonne de `point_de_vente`.
        ("politique_impression", "POINT_DE_VENTE", "TEXTE", "ETB-03"),
        // CPT-01 — les cinq se règlent au niveau de l'établissement, jamais plus bas : un point
        // de vente n'a pas sa propre politique de mot de passe.
        (
            "indicatif_telephonique_defaut",
            "ETABLISSEMENT",
            "TEXTE",
            "CPT-01",
        ),
        (
            "methode_authentification",
            "ETABLISSEMENT",
            "TEXTE",
            "CPT-01",
        ),
        (
            "mot_de_passe_longueur_min",
            "ETABLISSEMENT",
            "ENTIER",
            "CPT-01",
        ),
        (
            "jeton_acces_duree_min",
            "ETABLISSEMENT",
            "DUREE_MINUTES",
            "CPT-01",
        ),
        (
            "jeton_rafraichissement_duree_jours",
            "ETABLISSEMENT",
            "ENTIER",
            "CPT-01",
        ),
        // HEB-03 — **`HEURE_LOCALE`, et non `TEXTE`.** Le plan du cycle 004 écrivait `TEXTE` ; le
        // type fermé de `0008` porte `HEURE_LOCALE`, et l'employer est ce qui empêche la
        // validation d'accepter « demain matin ». L'écart au plan est consigné dans la migration
        // `0023`, à l'endroit où il se constate.
        (
            "heure_arrivee_standard",
            "ETABLISSEMENT",
            "HEURE_LOCALE",
            "HEB-03",
        ),
        (
            "heure_depart_standard",
            "ETABLISSEMENT",
            "HEURE_LOCALE",
            "HEB-03",
        ),
        // HEB-04 — **`DUREE_MINUTES`, et non `ENTIER`**, même raisonnement : le nom de la clé
        // porte l'unité, le type la confirme, et un `ENTIER` nu se serait un jour lu en heures.
        (
            "seuil_bascule_nuitee_minutes",
            "ETABLISSEMENT",
            "DUREE_MINUTES",
            "HEB-04",
        ),
    ];

    for (cle, portee, type_valeur, story) in attendus {
        let ligne = sqlx::query(
            r#"
            SELECT cle, type_valeur, portee_la_plus_basse, story
            FROM etablissements.parametre_catalogue
            WHERE cle = $1
            "#,
        )
        .bind(cle)
        .fetch_optional(&pool)
        .await
        .expect("lecture du catalogue")
        .unwrap_or_else(|| panic!("la clé « {cle} » doit exister au catalogue"));

        assert_eq!(
            ligne.get::<String, _>("portee_la_plus_basse"),
            portee,
            "portée inattendue pour « {cle} »"
        );
        assert_eq!(
            ligne.get::<String, _>("type_valeur"),
            type_valeur,
            "type inattendu pour « {cle} »"
        );
        assert_eq!(
            ligne.get::<String, _>("story"),
            story,
            "story inattendue pour « {cle} »"
        );
    }
}

/// **La durée d'un jeton n'est PAS un `MONTANT_MINEUR`, et sa minute n'est pas une constante.**
///
/// `jeton_acces_duree_min` porte le type `DUREE_MINUTES`, qui existe au catalogue depuis `0008`.
/// Le poser en `ENTIER` marcherait tout aussi bien et perdrait ce que le type dit : l'unité. Un
/// cycle ultérieur lisant `60` sans elle pourrait le prendre pour des secondes.
#[tokio::test]
async fn la_duree_du_jeton_d_acces_porte_son_unite_dans_son_type() {
    let pool = commun::pool_owner().await;

    let type_valeur: String = sqlx::query_scalar(
        "SELECT type_valeur FROM etablissements.parametre_catalogue WHERE cle = 'jeton_acces_duree_min'",
    )
    .fetch_one(&pool)
    .await
    .expect("lecture du catalogue");

    assert_eq!(
        type_valeur, "DUREE_MINUTES",
        "la durée du jeton d'accès doit porter son unité dans son type : `60` sans unité peut se \
         lire en secondes, et un jeton d'une minute déconnecterait tout le monde en boucle"
    );
}
