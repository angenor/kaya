//! Politique de mot de passe — **huit caractères, aucune règle de composition, refus des
//! compromis**.
//!
//! # Pourquoi aucune règle de composition
//!
//! « Une majuscule, un chiffre, un symbole » est la règle que tout le monde attend, et c'est
//! celle qui produit `Passw0rd!` — puis le post-it sous le clavier du comptoir. Le NIST l'a
//! retirée de ses recommandations (SP 800-63B) pour cette raison : elle réduit l'espace réel des
//! mots de passe choisis tout en donnant l'impression du contraire.
//!
//! Ce qui la remplace est **la longueur, et le refus de ce qui est déjà connu des attaquants**.
//! `chaise-tomate-abidjan` n'a ni majuscule, ni chiffre, ni symbole, et vaut mieux que `Passw0rd!`
//! par plusieurs ordres de grandeur.
//!
//! # À huit caractères, c'est le refus des compromis qui fait tout le travail
//!
//! Sans lui, la politique accepterait `12345678`, `password`, `motdepasse` — les huit premiers
//! essais de qui attaque un compte. Le refus des mots de passe compromis n'est donc pas une couche
//! de confort à ajouter plus tard : il est ce qui rend la longueur minimale défendable.
//!
//! # Le seuil vient du CATALOGUE, jamais d'une constante de ce fichier
//!
//! `mot_de_passe_longueur_min` est un **paramètre d'établissement** (migration `0019`), défaut
//! `8`. Un exploitant qui exige douze caractères le règle sans toucher au code ; une constante en
//! dur ferait de cette exigence une demande d'évolution. [`LONGUEUR_MIN_DEFAUT`] n'existe que pour
//! le cas où le paramètre serait illisible — et il vaut alors le défaut du catalogue, jamais zéro.
//!
//! # Ce contrôle porte sur la CRÉATION et le CHANGEMENT — jamais sur la connexion
//!
//! C'est la décision la moins évidente de ce fichier, et la seule qu'on écrirait mal. Vérifier à
//! la connexion enfermerait dehors un utilisateur légitime dont le mot de passe serait devenu
//! compromis entre-temps : **la liste grossit, le mot de passe ne change pas**. Le jour où une
//! fuite tierce ajoute son mot de passe à la liste, Adjoua ne pourrait plus ouvrir sa caisse — et
//! rien, dans le message d'erreur indiscernable de FR-012, ne lui dirait pourquoi.
//!
//! On refuse d'**en choisir** un mauvais ; on ne prend jamais en otage celui qui en a déjà un.

use super::mots_de_passe_compromis;

/// Longueur minimale de repli, **égale au défaut du catalogue**.
///
/// Employée seulement si le paramètre d'établissement est illisible. Un repli à zéro
/// transformerait une panne de lecture de configuration en absence totale de politique.
pub const LONGUEUR_MIN_DEFAUT: usize = 8;

/// Longueur maximale acceptée.
///
/// **Ce n'est pas une règle de sécurité, c'est une borne de coût.** Argon2 hache une entrée de
/// taille arbitraire ; sans plafond, un corps de requête de dix mégaoctets ferait travailler le
/// serveur pour rien. Le seuil est assez haut pour qu'aucune phrase de passe humaine ne le touche.
pub const LONGUEUR_MAX: usize = 256;

/// Pourquoi un mot de passe est refusé.
///
/// **Le motif est explicite, contrairement au refus d'authentification.** Les deux situations
/// n'ont rien à voir : à la connexion, distinguer les cas dirait à un attaquant si un compte
/// existe (FR-012) ; au choix d'un mot de passe, l'utilisateur est déjà authentifié ou habilité,
/// et il doit savoir **quoi corriger**. Un « mot de passe refusé » muet le ferait essayer
/// `12345679`.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RefusMotDePasse {
    #[error("mot de passe trop court : {longueur} caractère(s) pour un minimum de {minimum}")]
    TropCourt { longueur: usize, minimum: usize },

    #[error("mot de passe trop long : {longueur} caractère(s) pour un maximum de {LONGUEUR_MAX}")]
    TropLong { longueur: usize },

    #[error("ce mot de passe figure parmi les plus répandus et les plus essayés")]
    Compromis,
}

impl RefusMotDePasse {
    /// Code stable rendu au client, sur lequel l'interface branche sa clé i18n.
    ///
    /// **Le code distingue les trois cas**, alors que le statut HTTP est le même
    /// (`422 mot_de_passe_refuse`). C'est le corps qui enseigne, pas le statut.
    pub fn code(&self) -> &'static str {
        match self {
            RefusMotDePasse::TropCourt { .. } => "mot_de_passe_trop_court",
            RefusMotDePasse::TropLong { .. } => "mot_de_passe_trop_long",
            RefusMotDePasse::Compromis => "mot_de_passe_compromis",
        }
    }
}

/// Vérifie un mot de passe **au moment où on le choisit**.
///
/// `longueur_min` vient du catalogue de paramètres de l'établissement, jamais d'une constante de
/// l'appelant.
///
/// # L'ordre des contrôles n'est pas indifférent
///
/// La longueur d'abord, la liste ensuite. Un mot de passe de trois caractères n'a aucune raison
/// de coûter une recherche binaire dans 97 747 entrées, et surtout : dire « trop court » est plus
/// utile que dire « trop répandu » à quelqu'un qui a tapé `abc`.
///
/// # La longueur se compte en CARACTÈRES, pas en octets
///
/// `« Yaoundé2026 »` fait onze caractères et douze octets. Compter les octets accepterait un mot
/// de passe de six caractères accentués — et, pire, rendrait la politique dépendante de la langue
/// de l'utilisateur.
pub fn verifier(mot_de_passe: &str, longueur_min: usize) -> Result<(), RefusMotDePasse> {
    let longueur = mot_de_passe.chars().count();

    if longueur < longueur_min {
        return Err(RefusMotDePasse::TropCourt {
            longueur,
            minimum: longueur_min,
        });
    }

    if longueur > LONGUEUR_MAX {
        return Err(RefusMotDePasse::TropLong { longueur });
    }

    if mots_de_passe_compromis::est_compromis(mot_de_passe) {
        return Err(RefusMotDePasse::Compromis);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Sept caractères : refusé.**
    #[test]
    fn sept_caracteres_sont_refuses() {
        let refus = verifier("chaise7", LONGUEUR_MIN_DEFAUT).expect_err("doit être refusé");
        assert_eq!(
            refus,
            RefusMotDePasse::TropCourt {
                longueur: 7,
                minimum: 8
            }
        );
        assert_eq!(refus.code(), "mot_de_passe_trop_court");
    }

    /// **`12345678` : huit caractères, et refusé quand même.**
    ///
    /// C'est le test qui prouve que la longueur seule ne suffit pas. Une politique qui accepterait
    /// celui-là serait battue au premier essai.
    #[test]
    fn douze_mille_trois_cent_quarante_cinq_six_sept_huit_est_refuse_bien_qu_il_fasse_huit() {
        let refus = verifier("12345678", LONGUEUR_MIN_DEFAUT).expect_err("doit être refusé");
        assert_eq!(refus, RefusMotDePasse::Compromis);
        assert_eq!(refus.code(), "mot_de_passe_compromis");
    }

    /// **`chaise-tomate-abidjan` : accepté, sans majuscule ni chiffre ni symbole.**
    ///
    /// C'est le test qui prouve l'absence de règle de composition. Le refuser serait exactement la
    /// politique que le NIST a retirée de ses recommandations.
    #[test]
    fn une_phrase_de_passe_sans_majuscule_ni_chiffre_ni_symbole_est_acceptee() {
        assert!(verifier("chaise-tomate-abidjan", LONGUEUR_MIN_DEFAUT).is_ok());
        assert!(verifier("le maquis de tante affoue", LONGUEUR_MIN_DEFAUT).is_ok());
    }

    /// Le seuil vient du **paramètre**, pas d'une constante.
    #[test]
    fn le_seuil_est_celui_qu_on_lui_donne() {
        // Le même mot de passe passe à 8 et échoue à 12.
        assert!(verifier("chaise99", 8).is_ok());
        assert!(matches!(
            verifier("chaise99", 12),
            Err(RefusMotDePasse::TropCourt { minimum: 12, .. })
        ));
    }

    /// La longueur se compte en **caractères**, pas en octets.
    #[test]
    fn la_longueur_se_compte_en_caracteres() {
        // Six caractères, neuf octets en UTF-8. Compter les octets l'accepterait à tort.
        assert!(matches!(
            verifier("éàèùôç", 8),
            Err(RefusMotDePasse::TropCourt { longueur: 6, .. })
        ));
    }

    /// Le plafond est une **borne de coût**, et il ne gêne aucune phrase humaine.
    #[test]
    fn un_mot_de_passe_demesure_est_refuse_pour_le_cout_du_hachage() {
        let demesure = "a".repeat(LONGUEUR_MAX + 1);
        assert!(matches!(
            verifier(&demesure, LONGUEUR_MIN_DEFAUT),
            Err(RefusMotDePasse::TropLong { .. })
        ));
        assert!(verifier(&"a".repeat(LONGUEUR_MAX), LONGUEUR_MIN_DEFAUT).is_ok());
    }
}
