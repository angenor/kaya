//! La liste des mots de passe compromis, **embarquée dans le binaire**.
//!
//! # Pourquoi ce refus n'est pas optionnel
//!
//! La politique de CPT-01 est de **huit caractères, aucune règle de composition**. Les règles de
//! composition — une majuscule, un chiffre, un symbole — sont refusées parce qu'elles produisent
//! un mot de passe sur un post-it au comptoir : la seule chose qu'elles garantissent est que
//! l'utilisateur ne s'en souviendra pas.
//!
//! À huit caractères sans règle, **c'est ce refus-ci qui fait tout le travail**. Sans lui, la
//! politique accepterait `12345678`, `password`, `motdepasse` — c'est-à-dire les huit premiers
//! essais de qui attaque un compte. Ce n'est donc pas une couche de confort à ajouter plus tard.
//!
//! # Embarquée, jamais interrogée en réseau
//!
//! `include_str!`, pas un appel à Have I Been Pwned. Trois raisons, dans l'ordre :
//!
//!   1. la création de compte doit aboutir sur le réseau d'Abengourou, où un service tiers est
//!      exactement ce qui ne répond pas ;
//!   2. le paquet auto-hébergé (mode B) tourne chez un client sans garantie de sortie internet ;
//!   3. un appel réseau publierait, à chaque création de compte, le fait qu'un mot de passe est en
//!      cours de choix — et son préfixe, même dans le protocole à k-anonymat.
//!
//! Le coût est de 800 Ko dans un binaire qui en fait plusieurs dizaines de Mo.
//!
//! # La recherche est binaire, et c'est la raison du tri par octets
//!
//! Le fichier est trié par **octets** (`LC_ALL=C sort`), qui est l'ordre de comparaison de `str`
//! en Rust. Un tri par locale — celui de n'importe quel `sort` sans `LC_ALL=C` — casserait la
//! recherche **sans erreur visible** : elle rendrait « non compromis » de temps en temps, sur les
//! entrées mal placées. Le test `la_liste_est_triee_par_octets` refuse cette situation à chaque
//! exécution plutôt que de la faire découvrir par un compte compromis.
//!
//! # Ce contrôle porte sur la création et le changement — JAMAIS sur la connexion
//!
//! Vérifier à la connexion enfermerait dehors un utilisateur légitime dont le mot de passe serait
//! devenu compromis entre-temps : la liste grossit, le mot de passe ne change pas. On refuse d'en
//! choisir un mauvais ; on ne prend jamais en otage celui qui en a déjà un.

use std::sync::OnceLock;

/// Le fichier de données, avec son en-tête documentaire. Voir
/// `authentification/mots-de-passe-compromis.txt` pour la source, la date d'extraction et les
/// transformations appliquées.
const LISTE_BRUTE: &str = include_str!("mots-de-passe-compromis.txt");

/// Début du marqueur qui clôt l'en-tête.
///
/// # Pourquoi un marqueur plutôt que « les lignes en `#` sont des commentaires »
///
/// **Cinquante-neuf mots de passe de la liste commencent par un croisillon** — `#1angel`,
/// `#1baby`, … Traiter le croisillon comme un caractère de commentaire les aurait retirés
/// silencieusement, et ces cinquante-neuf mots de passe auraient été acceptés. Le défaut a été
/// attrapé par le test `aucune_entree_n_est_perdue_par_l_en_tete`, écrit **avant** d'y avoir
/// pensé, sur la seule intuition qu'un séparateur implicite est un séparateur qui trahit.
const FIN_EN_TETE: &str = "#--- FIN DE L'EN-TÊTE";

/// Les entrées, triées, sans l'en-tête. Construites une fois.
static ENTREES: OnceLock<Vec<&'static str>> = OnceLock::new();

fn entrees() -> &'static [&'static str] {
    ENTREES.get_or_init(|| {
        let mut lignes = LISTE_BRUTE.lines();

        // Consomme l'en-tête jusqu'au marqueur inclus. Son absence est une erreur de
        // construction du fichier, pas un cas dégradé : sans lui, l'en-tête entrerait dans la
        // liste et le tri par octets serait faux dès la première comparaison.
        let marqueur_trouve = lignes.by_ref().any(|ligne| ligne.starts_with(FIN_EN_TETE));
        assert!(
            marqueur_trouve,
            "le marqueur de fin d'en-tête est absent de mots-de-passe-compromis.txt. Le fichier \
             a été régénéré sans lui : voir l'en-tête pour la commande exacte."
        );

        lignes.filter(|ligne| !ligne.is_empty()).collect()
    })
}

/// Nombre d'entrées de la liste — exposé pour le décompte des portes et le diagnostic.
pub fn nombre_d_entrees() -> usize {
    entrees().len()
}

/// Ce mot de passe figure-t-il parmi les plus répandus ?
///
/// La comparaison est **insensible à la casse** et ignore les espaces de bord. Sans cela,
/// `Azerty123` passerait là où `azerty123` est refusé — une distinction qui n'apprend rien à
/// l'utilisateur et ne coûte rien à l'attaquant, qui essaie les deux.
///
/// **La normalisation s'arrête là.** Elle ne défait ni les substitutions de caractères
/// (`p@ssword`), ni les suffixes numériques : ces variantes figurent déjà dans la liste quand
/// elles sont réellement répandues, et les deviner ici reviendrait à réinventer un moteur de
/// règles — c'est-à-dire à refuser des mots de passe légitimes sans savoir lesquels.
pub fn est_compromis(mot_de_passe: &str) -> bool {
    let normalise = mot_de_passe.trim().to_lowercase();
    entrees().binary_search(&normalise.as_str()).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **La cible n'est pas vide.** Une liste qui ne se charge pas rendrait `est_compromis`
    /// toujours faux : le refus passerait au vert en n'ayant rien à refuser.
    #[test]
    fn la_liste_porte_ses_entrees() {
        let n = nombre_d_entrees();
        assert!(
            n > 90_000,
            "la liste ne porte que {n} entrée(s). Le fichier de données est-il tronqué, ou \
             l'en-tête a-t-il avalé le contenu ? Une liste vide rendrait `est_compromis` \
             toujours faux, sans que rien ne le signale."
        );
    }

    /// **Le tri par octets, dont dépend la recherche binaire.**
    ///
    /// Ce test existe parce que le défaut qu'il attrape est invisible : une liste mal triée ne
    /// lève aucune erreur, elle rend simplement « non compromis » sur les entrées mal placées.
    #[test]
    fn la_liste_est_triee_par_octets() {
        let entrees = entrees();
        for paire in entrees.windows(2) {
            assert!(
                paire[0] < paire[1],
                "la liste n'est pas triée par octets : « {} » précède « {} ». La recherche \
                 binaire rendrait « non compromis » sans erreur visible. Régénérer avec \
                 `LC_ALL=C sort -u`, jamais avec un `sort` dépendant de la locale.",
                paire[0],
                paire[1]
            );
        }
    }

    /// **L'en-tête ne mange aucune entrée** — le test qui a trouvé le défaut.
    ///
    /// La liste contient cinquante-neuf mots de passe commençant par un croisillon. Une première
    /// version filtrait « les lignes en `#` » comme des commentaires : elle les retirait tous, en
    /// silence, et ces cinquante-neuf mots de passe auraient été acceptés à la création de compte.
    #[test]
    fn aucune_entree_n_est_perdue_par_l_en_tete() {
        let apres_marqueur = LISTE_BRUTE
            .lines()
            .skip_while(|l| !l.starts_with(FIN_EN_TETE))
            .skip(1)
            .filter(|l| !l.is_empty())
            .count();

        assert_eq!(
            apres_marqueur,
            nombre_d_entrees(),
            "le chargement perd des lignes du corps"
        );

        let en_croisillon = entrees().iter().filter(|e| e.starts_with('#')).count();
        assert!(
            en_croisillon > 0,
            "aucune entrée ne commence par « # » : soit la liste a changé de source, soit \
             l'en-tête les a de nouveau avalées. Dans le second cas, ces mots de passe seraient \
             acceptés sans que rien ne le signale."
        );

        assert!(est_compromis("#1angel"), "« #1angel » est dans la liste");
    }

    /// Les trois cas qui comptent, ceux de FR-011 et du test `politique_mot_de_passe.rs`.
    #[test]
    fn les_mots_de_passe_les_plus_repandus_sont_refuses() {
        assert!(
            est_compromis("12345678"),
            "« 12345678 » fait bien huit caractères et doit pourtant être refusé — c'est \
             exactement le cas qui justifie cette liste"
        );
        assert!(est_compromis("password"));
        assert!(est_compromis("azerty123"));
    }

    /// Une phrase de passe ordinaire passe, **sans majuscule, sans chiffre, sans symbole**.
    #[test]
    fn une_phrase_de_passe_ordinaire_est_acceptee() {
        assert!(
            !est_compromis("chaise-tomate-abidjan"),
            "une phrase de passe sans règle de composition doit être acceptée : c'est le cœur de \
             la politique de CPT-01"
        );
    }

    #[test]
    fn la_comparaison_ignore_la_casse_et_les_espaces_de_bord() {
        assert!(est_compromis("PASSWORD"));
        assert!(est_compromis("Azerty123"));
        assert!(est_compromis("  password  "));
    }

    /// La normalisation **ne défait pas** les substitutions — écrit pour que personne ne croie
    /// qu'elle le fait.
    #[test]
    fn la_normalisation_ne_devine_pas_les_variantes() {
        // Présent dans la liste par lui-même, pas par déduction depuis « password ».
        let par_lui_meme = est_compromis("p@ssw0rd");
        let devine = est_compromis("pa55w0rd-que-personne-n-a-jamais-employe");
        assert!(
            !devine,
            "la liste ne doit refuser que ce qu'elle contient : deviner des variantes \
             reviendrait à refuser des mots de passe légitimes sans savoir lesquels"
        );
        // Sans assertion sur `par_lui_meme` : ce que la liste contient est une donnée de la
        // source, pas une décision du produit.
        let _ = par_lui_meme;
    }
}
