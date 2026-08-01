//! Hachage des mots de passe — **Argon2id, à paramètres explicites et datés**.
//!
//! # Livré en deux temps, et l'ordre est dit
//!
//! Ce fichier porte d'abord le **hachage** (tâche T020), parce que les seeds en ont besoin pour
//! écrire les comptes de démonstration : un seed qui poserait un condensat factice produirait des
//! comptes sur lesquels personne ne peut se connecter, donc une démonstration qui ne démontre
//! rien. La **vérification**, le **rehachage** et le **condensat factice de l'indiscernabilité**
//! viennent avec T025, dans ce même fichier.
//!
//! L'ordonnancement des tâches plaçait T025 après T020 sans voir cette dépendance. Elle est
//! consignée ici plutôt que résolue en silence.
//!
//! # Les paramètres, et d'où ils viennent
//!
//! `m = 19456` KiB (19 Mio), `t = 2`, `p = 1`, sel de 16 octets, sortie de 32.
//!
//! **Source : OWASP Password Storage Cheat Sheet**, configuration Argon2id recommandée
//! « m=19456 (19 MiB), t=2, p=1 ». Ce n'est pas un réglage choisi au jugé : c'est le point où le
//! coût pour le serveur reste imperceptible et celui de l'attaquant par carte graphique devient
//! prohibitif. Les écrire sans leur source les rendrait « ajustables », et le premier profil de
//! performance les diviserait par deux.
//!
//! # Le format PHC porte les paramètres AVEC le condensat
//!
//! ```text
//! $argon2id$v=19$m=19456,t=2,p=1$<sel base64>$<condensat base64>
//! ```
//!
//! C'est ce qui rend une montée de paramètres possible : à la vérification, on lit ceux du
//! condensat, on vérifie avec eux, puis on **rehache** si la recommandation a changé. Sans eux, la
//! montée ne protégerait que les comptes créés après elle — c'est-à-dire aucun de ceux qui
//! existent déjà, qui sont précisément les plus anciens et les plus exposés.

use argon2::password_hash::{PasswordHasher, SaltString, rand_core::OsRng};
use argon2::{Algorithm, Argon2, Params, Version};

/// Coût mémoire, en kibioctets. **19 456 KiB = 19 MiB** — OWASP Password Storage Cheat Sheet.
pub const MEMOIRE_KIB: u32 = 19_456;

/// Nombre de passes. **2** — même source.
pub const ITERATIONS: u32 = 2;

/// Degré de parallélisme. **1** — même source.
pub const PARALLELISME: u32 = 1;

/// Longueur du condensat, en octets. **32** — la taille de sortie de BLAKE2b tronquée par Argon2,
/// et la même que celle de SHA-256 : au-delà, on paie sans gagner.
pub const LONGUEUR_SORTIE: usize = 32;

/// Échec de hachage ou de vérification.
#[derive(Debug, thiserror::Error)]
pub enum ErreurHachage {
    /// Les paramètres sont refusés par le crate. **Ne peut arriver qu'après une modification des
    /// constantes ci-dessus** : elles sont validées par un test.
    #[error("paramètres Argon2 invalides : {0}")]
    Parametres(String),

    #[error("hachage impossible : {0}")]
    Hachage(String),
}

/// L'instance Argon2id aux paramètres du produit.
///
/// Construite à chaque appel plutôt que mise en cache : la construction ne fait que valider trois
/// entiers, quand le hachage lui-même alloue 19 Mio et prend des dizaines de millisecondes. Un
/// `OnceLock` ici optimiserait 0,01 % du coût en ajoutant un état global.
fn argon() -> Result<Argon2<'static>, ErreurHachage> {
    let params = Params::new(
        MEMOIRE_KIB,
        ITERATIONS,
        PARALLELISME,
        Some(LONGUEUR_SORTIE),
    )
    .map_err(|e| ErreurHachage::Parametres(e.to_string()))?;

    Ok(Argon2::new(Algorithm::Argon2id, Version::V0x13, params))
}

/// Hache un mot de passe et rend son condensat au **format PHC**.
///
/// Le sel est tiré de l'entropie du système à chaque appel : deux comptes portant le même mot de
/// passe produisent donc deux condensats différents, ce qui interdit de reconnaître les mots de
/// passe partagés par simple comparaison de la table.
///
/// # Ce que cette fonction ne fait PAS
///
/// Elle ne valide **aucune politique** — ni longueur, ni refus des mots de passe compromis. Cette
/// séparation est délibérée : la politique dépend d'un **paramètre d'établissement**
/// (`mot_de_passe_longueur_min`) que ce module n'a aucune raison de connaître, et elle s'applique
/// à la création et au changement, jamais à la connexion. Les mêler ferait de `hacher` un point
/// de passage qui refuserait un jour un mot de passe légitime devenu compromis.
pub fn hacher(mot_de_passe: &str) -> Result<String, ErreurHachage> {
    let sel = SaltString::generate(&mut OsRng);

    argon()?
        .hash_password(mot_de_passe.as_bytes(), &sel)
        .map(|condensat| condensat.to_string())
        .map_err(|e| ErreurHachage::Hachage(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Les paramètres sont acceptés par le crate — sans quoi tout hachage échouerait à
    /// l'exécution, sur un chemin que rien d'autre ne couvre.
    #[test]
    fn les_parametres_owasp_sont_valides() {
        argon().expect("les paramètres OWASP doivent être acceptés par argon2");
    }

    /// **Le condensat porte ses paramètres**, ce qui rend le rehachage possible.
    #[test]
    fn le_condensat_est_au_format_phc_et_porte_ses_parametres() {
        let condensat = hacher("chaise-tomate-abidjan").expect("hachage");

        assert!(
            condensat.starts_with("$argon2id$"),
            "condensat inattendu : {condensat}"
        );
        assert!(
            condensat.contains("m=19456,t=2,p=1"),
            "les paramètres doivent voyager avec le condensat, sinon une montée ne protégerait \
             que les comptes créés après elle. Obtenu : {condensat}"
        );
    }

    /// **Deux comptes au même mot de passe ont deux condensats.**
    ///
    /// Sans sel aléatoire, une simple comparaison de la colonne dirait qui partage son mot de
    /// passe avec qui — et un condensat cassé les casserait tous d'un coup.
    #[test]
    fn deux_hachages_du_meme_mot_de_passe_different() {
        let a = hacher("chaise-tomate-abidjan").expect("hachage");
        let b = hacher("chaise-tomate-abidjan").expect("hachage");
        assert_ne!(a, b, "le sel n'est pas aléatoire");
    }

    /// Un mot de passe vide se hache sans erreur.
    ///
    /// **Ce n'est pas une permission** : la politique refuse les mots de passe courts, et c'est
    /// son rôle. Ce test fige la séparation des responsabilités — `hacher` ne juge pas, sinon la
    /// règle vivrait à deux endroits et divergerait.
    #[test]
    fn hacher_ne_juge_pas_le_mot_de_passe() {
        assert!(hacher("").is_ok());
    }
}
