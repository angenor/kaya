//! Hachage des mots de passe — **Argon2id, à paramètres explicites et datés**.
//!
//! # Livré en deux temps, et l'ordre est dit
//!
//! Ce fichier a porté d'abord le **hachage** (tâche T020), parce que les seeds en ont besoin pour
//! écrire les comptes de démonstration : un seed qui poserait un condensat factice produirait des
//! comptes sur lesquels personne ne peut se connecter, donc une démonstration qui ne démontre
//! rien. La **vérification**, le **rehachage** et le **condensat factice de l'indiscernabilité**
//! sont venus avec T025, dans ce même fichier — et sans réécrire le hachage.
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

use std::sync::OnceLock;

use argon2::password_hash::{
    PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng,
};
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

    /// Le condensat lu en base n'est pas au format PHC.
    ///
    /// **Traité comme une erreur, jamais comme un refus d'authentification.** Un condensat
    /// illisible signale une corruption de données ou une migration ratée ; le confondre avec un
    /// mauvais mot de passe ferait disparaître l'incident dans le flot normal des échecs de
    /// connexion, là où personne ne le regarde jamais.
    #[error("condensat illisible : {0}")]
    CondensatIllisible(String),
}

/// Ce que la vérification apprend, en une seule lecture du condensat.
///
/// Deux informations distinctes, et les séparer serait une seconde vérification Argon2 — c'est-à-dire
/// 19 Mio et quelques dizaines de millisecondes sur le chemin de connexion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verification {
    /// Le mot de passe correspond-il au condensat ?
    pub valide: bool,
    /// Le condensat a-t-il été produit avec d'**autres** paramètres que ceux d'aujourd'hui ?
    ///
    /// `true` n'est **jamais** un motif de refus : c'est une invitation à rehacher après une
    /// vérification réussie.
    pub rehachage_requis: bool,
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

/// Vérifie un mot de passe contre un condensat, **et dit au passage si le condensat a vieilli**.
///
/// # Le rehachage, et pourquoi il se décide ici
///
/// Le condensat porte ses propres paramètres (format PHC). Les comparer à ceux d'aujourd'hui est
/// la seule façon de savoir qu'un compte est encore protégé par les réglages de 2026 alors que la
/// recommandation a bougé. **Sans ce contrôle, une montée de paramètres ne protégerait que les
/// comptes créés après elle** — c'est-à-dire aucun des anciens, qui sont précisément les plus
/// exposés.
///
/// La décision est prise ici et **l'action est prise par l'appelant** : rehacher demande d'écrire
/// en base, et ce module ne connaît aucune table. Le service d'authentification lit
/// [`Verification::rehachage_requis`] après un succès, appelle [`hacher`] et met à jour la ligne.
///
/// # Ce qui n'est jamais un motif de refus
///
/// Un condensat aux anciens paramètres reste **valide**. Refuser la connexion pour cause de
/// paramètres périmés enfermerait dehors tout le personnel le jour d'une montée — au moment
/// exact où l'on veut que personne ne remarque rien.
pub fn verifier(condensat: &str, mot_de_passe: &str) -> Result<Verification, ErreurHachage> {
    let analyse =
        PasswordHash::new(condensat).map_err(|e| ErreurHachage::CondensatIllisible(e.to_string()))?;

    // `verify_password` échoue aussi bien sur un mot de passe faux que sur un condensat d'un
    // algorithme inconnu. Seule la première forme est un refus ordinaire ; la seconde a déjà été
    // écartée par `PasswordHash::new` ci-dessus.
    let valide = argon()?
        .verify_password(mot_de_passe.as_bytes(), &analyse)
        .is_ok();

    Ok(Verification {
        valide,
        rehachage_requis: parametres_perimes(&analyse),
    })
}

/// Les paramètres portés par ce condensat diffèrent-ils de ceux d'aujourd'hui ?
///
/// Un paramètre **absent** du condensat compte comme périmé : il ne peut venir que d'un format
/// plus ancien, et rehacher est alors exactement ce qu'il faut faire.
fn parametres_perimes(analyse: &PasswordHash<'_>) -> bool {
    let lire = |nom: &str| -> Option<u32> {
        analyse
            .params
            .get(nom)
            .and_then(|v| v.decimal().ok())
    };

    analyse.algorithm.as_str() != Algorithm::Argon2id.ident().as_str()
        || lire("m") != Some(MEMOIRE_KIB)
        || lire("t") != Some(ITERATIONS)
        || lire("p") != Some(PARALLELISME)
}

/// Le **condensat factice** de l'indiscernabilité temporelle — calculé **une fois, au démarrage**.
///
/// # Ce qu'il sert à faire
///
/// Sur identifiant inconnu, le service d'authentification vérifie quand même un mot de passe :
/// celui-ci, contre ce condensat-là. Sans cela, un identifiant inexistant répondrait en une
/// fraction de milliseconde et un identifiant existant en quelques dizaines — et **le temps de
/// réponse dirait qui est client de Kaya**, alors même que le message et le code seraient
/// rigoureusement identiques (FR-012). *C'est la moitié de l'exigence que le message identique ne
/// tient pas.*
///
/// # Pourquoi au démarrage et non à chaque requête
///
/// Le hachage coûte 19 Mio et des dizaines de millisecondes. Le calculer à chaque tentative
/// **doublerait** le coût du chemin de connexion — et, plus grave, le doublerait pour le seul cas
/// de l'identifiant inconnu, ce qui rétablirait précisément l'écart de temps qu'on cherche à
/// effacer. La valeur est donc figée dans un [`OnceLock`], et [`prechauffer`] force son calcul au
/// démarrage plutôt qu'à la première tentative — sans quoi la toute première requête de la vie du
/// processus resterait distinguable.
///
/// # Le mot de passe factice n'est un secret pour personne
///
/// Il ne protège rien : aucun compte ne le porte, et il ne peut donc pas être « trouvé ». Ce qui
/// compte est que le condensat soit **réel**, produit par les mêmes paramètres, de sorte que le
/// coût de sa vérification soit celui d'un vrai compte.
pub fn condensat_factice() -> &'static str {
    static FACTICE: OnceLock<String> = OnceLock::new();

    FACTICE.get_or_init(|| {
        hacher("condensat-factice-de-l-indiscernabilite-CPT-01")
            .expect("le hachage du condensat factice ne peut échouer qu'avec des paramètres invalides, que les tests refusent")
    })
}

/// Force le calcul du condensat factice **maintenant**, au démarrage du processus.
///
/// Appelée par `main.rs`. Sans elle, le `OnceLock` se remplirait à la **première** tentative de
/// connexion sur identifiant inconnu — qui serait alors plus lente que toutes les suivantes, donc
/// distinguable. Un défaut d'une seule requête, invisible en test et parfaitement réel.
pub fn prechauffer() {
    let _ = condensat_factice();
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

    // ═══════════════════════════════════════════════════════════════════════════════════════
    //  T025 — vérification, rehachage, condensat factice
    // ═══════════════════════════════════════════════════════════════════════════════════════

    #[test]
    fn le_bon_mot_de_passe_est_accepte_et_le_mauvais_refuse() {
        let condensat = hacher("chaise-tomate-abidjan").expect("hachage");

        assert!(verifier(&condensat, "chaise-tomate-abidjan").unwrap().valide);
        assert!(!verifier(&condensat, "chaise-tomate-abidjar").unwrap().valide);
        assert!(!verifier(&condensat, "").unwrap().valide);
    }

    /// **Un condensat frais n'a jamais besoin d'être rehaché.**
    ///
    /// Sans ce test, un `parametres_perimes` qui rendrait toujours `true` passerait inaperçu — et
    /// chaque connexion réussie rehacherait, ajoutant 19 Mio et des dizaines de millisecondes au
    /// chemin le plus chaud du produit.
    #[test]
    fn un_condensat_aux_parametres_du_jour_ne_demande_aucun_rehachage() {
        let condensat = hacher("chaise-tomate-abidjan").expect("hachage");
        let verification = verifier(&condensat, "chaise-tomate-abidjan").expect("vérification");

        assert!(verification.valide);
        assert!(
            !verification.rehachage_requis,
            "un condensat produit à l'instant réclame un rehachage : la comparaison de paramètres \
             est fausse, et chaque connexion en paierait le coût"
        );
    }

    /// **Un condensat aux ANCIENS paramètres reste valide, et demande un rehachage.**
    ///
    /// C'est le cas qui fait exister la fonction. Le condensat ci-dessous est produit avec
    /// `m=16, t=1, p=1` — des paramètres délibérément faibles, comme le serait un condensat écrit
    /// avant une montée. Le mot de passe doit continuer à ouvrir la session : refuser ici
    /// enfermerait dehors tout le personnel le jour de la montée.
    #[test]
    fn un_condensat_aux_anciens_parametres_reste_valide_et_demande_un_rehachage() {
        let anciens = Params::new(16, 1, 1, Some(LONGUEUR_SORTIE)).expect("anciens paramètres");
        let sel = SaltString::generate(&mut OsRng);
        let condensat = Argon2::new(Algorithm::Argon2id, Version::V0x13, anciens)
            .hash_password(b"chaise-tomate-abidjan", &sel)
            .expect("hachage aux anciens paramètres")
            .to_string();

        let verification = verifier(&condensat, "chaise-tomate-abidjan").expect("vérification");

        assert!(
            verification.valide,
            "un condensat aux anciens paramètres doit rester valide — sinon une montée de \
             paramètres est une panne de connexion générale"
        );
        assert!(
            verification.rehachage_requis,
            "le rehachage n'est pas demandé : la montée ne protégerait que les comptes créés \
             après elle, donc aucun des anciens"
        );
    }

    /// Un condensat illisible est une **erreur**, pas un refus d'authentification.
    ///
    /// Les confondre ferait disparaître une corruption de données dans le flot ordinaire des
    /// mauvais mots de passe, là où personne ne la regarderait jamais.
    #[test]
    fn un_condensat_illisible_est_une_erreur_et_non_un_refus() {
        let erreur = verifier("pas-un-condensat-phc", "peu importe").expect_err("doit échouer");
        assert!(
            matches!(erreur, ErreurHachage::CondensatIllisible(_)),
            "erreur inattendue : {erreur}"
        );
    }

    /// **Le condensat factice est réel, stable, et ne correspond à rien.**
    #[test]
    fn le_condensat_factice_est_un_vrai_condensat_stable() {
        let a = condensat_factice();
        let b = condensat_factice();

        assert_eq!(a, b, "le condensat factice doit être calculé UNE fois");
        assert!(a.starts_with("$argon2id$"), "condensat inattendu : {a}");
        assert!(
            a.contains("m=19456,t=2,p=1"),
            "le condensat factice doit porter les MÊMES paramètres qu'un vrai — sinon sa \
             vérification coûte un temps différent, et l'écart rétablit la fuite. Obtenu : {a}"
        );

        // Le vérifier coûte le même travail qu'un vrai compte, et rend `false` — ce qui est
        // exactement ce que le service d'authentification en attend.
        assert!(!verifier(a, "n'importe quoi").unwrap().valide);
    }
}
