//! Les secrets d'exploitation, et le refus de démarrer sans eux.
//!
//! # Le patron : échouer au démarrage, jamais à la première requête
//!
//! C'est le modèle de `contexte::verifier_derogation()`, que ces deux variables **remplacent** :
//! une vérification faite une fois, bruyamment, à un moment où personne n'utilise encore le
//! produit. Vérifier à chaque requête coûterait en permanence pour une décision qui ne change
//! jamais en cours d'exécution ; vérifier paresseusement, au premier besoin, ferait découvrir la
//! configuration manquante par un caissier devant un client.
//!
//! **Aucun secret n'a de valeur par défaut** (principe IX). Un défaut est un secret publié : il
//! vit dans le dépôt, dans l'image, dans les archives de tous les postes qui ont cloné le projet.
//! Le prix d'un défaut absent est un message d'erreur au premier déploiement ; celui d'un défaut
//! présent est une signature de jetons forgeable par quiconque a lu le code.
//!
//! # Redis n'est plus optionnel — écrit ici plutôt que découvert à la première panne
//!
//! Jusqu'au cycle 002, Redis servait au cache et à la file FNE : perdu, le produit ralentissait.
//! Depuis CPT-01, **la liste de révocation est consultée à chaque requête authentifiée** — c'est
//! le prix de la « coupure immédiate au départ d'un employé » du cadrage §12.2. Son statut de
//! stockage *éphémère reconstructible* ne change pas (Redis vidé, tout le monde se reconnecte et
//! aucune donnée métier ne manque), mais **son exigence de disponibilité, si** : Redis absent,
//! aucune requête authentifiée n'aboutit.

/// Longueur minimale de la clé de signature, en octets.
///
/// HS256 signe avec HMAC-SHA256, dont le bloc interne fait 64 octets et la sortie 32. Une clé plus
/// courte que la sortie n'ajoute aucune sécurité au-delà de sa propre longueur : la RFC 2104 §3 le
/// dit, et la RFC 7518 §3.2 en fait une exigence — « a key of the same size as the hash output or
/// larger MUST be used ». **32 est donc un plancher, pas un réglage.**
pub const LONGUEUR_MINIMALE_CLE_JWT: usize = 32;

/// Nom de la variable portant la clé de signature des jetons de session.
pub const VAR_CLE_JWT: &str = "KAYA_JWT_CLE";

/// Nom de la variable portant le mot de passe des comptes de démonstration.
pub const VAR_MOT_DE_PASSE_SEEDS: &str = "KAYA_SEEDS_MOT_DE_PASSE";

/// Nom de la variable qui déclare l'environnement d'exécution.
pub const VAR_ENVIRONNEMENT: &str = "KAYA_ENVIRONNEMENT";

/// Ce qui manque, et pourquoi c'est bloquant.
#[derive(Debug, thiserror::Error)]
pub enum ErreurSecret {
    #[error("{variable} est absente de l'environnement")]
    Absente { variable: &'static str },

    #[error(
        "{variable} fait {longueur} octet(s), il en faut au moins {minimum} \
         (RFC 7518 §3.2 : la clé HMAC ne descend pas sous la taille de l'empreinte)"
    )]
    TropCourte {
        variable: &'static str,
        longueur: usize,
        minimum: usize,
    },

    #[error("les seeds ne s'exécutent pas en environnement « {environnement} »")]
    ProductionRefusee { environnement: String },
}

/// La clé de signature des jetons de session, lue de l'environnement.
///
/// Rendue en octets bruts : la variable porte le secret **tel quel**, jamais une forme encodée.
/// Un encodage imposé (base64, hex) ajouterait un mode d'échec silencieux — une clé mal décodée
/// signe et vérifie parfaitement, et rien ne distingue une clé de 32 octets d'une chaîne base64 de
/// 32 caractères qui n'en porte que 24.
pub fn cle_jwt() -> Result<Vec<u8>, ErreurSecret> {
    valider_cle_jwt(std::env::var(VAR_CLE_JWT).ok())
}

/// La règle, séparée de sa source.
///
/// **Ce n'est pas une commodité de test.** `std::env::set_var` est `unsafe` depuis l'édition 2024
/// — l'environnement est un état global du processus, que rien ne protège d'un autre thread qui
/// le lit — et le crate porte `#![forbid(unsafe_code)]`. Un test qui poserait la variable devrait
/// donc lever l'interdiction sur tout le crate pour vérifier trois comparaisons de longueur.
/// Séparer la règle de sa lecture coûte quatre lignes et rend la règle testable **sans** toucher
/// à l'état global.
fn valider_cle_jwt(valeur: Option<String>) -> Result<Vec<u8>, ErreurSecret> {
    let octets = valeur
        .ok_or(ErreurSecret::Absente {
            variable: VAR_CLE_JWT,
        })?
        .into_bytes();

    if octets.len() < LONGUEUR_MINIMALE_CLE_JWT {
        return Err(ErreurSecret::TropCourte {
            variable: VAR_CLE_JWT,
            longueur: octets.len(),
            minimum: LONGUEUR_MINIMALE_CLE_JWT,
        });
    }

    Ok(octets)
}

/// Le mot de passe des comptes de démonstration.
///
/// **Il n'est pas lu par le serveur** — seulement par le binaire `seeds`. Un serveur qui exigerait
/// cette variable refuserait de démarrer en production, où les seeds ne s'exécutent jamais.
pub fn mot_de_passe_seeds() -> Result<String, ErreurSecret> {
    refuser_si_production()?;

    std::env::var(VAR_MOT_DE_PASSE_SEEDS).map_err(|_| ErreurSecret::Absente {
        variable: VAR_MOT_DE_PASSE_SEEDS,
    })
}

/// Les seeds refusent de s'exécuter si l'environnement se déclare production.
///
/// **La garde est ici, pas dans le script d'appel.** Un script se contourne d'une ligne de
/// commande ; le binaire, non. Et c'est bien le binaire qu'on lance à la main un soir d'incident,
/// sur le serveur, en cherchant à « juste remettre les données de démonstration ».
///
/// L'absence de la variable vaut « pas la production » : le poste de développement ne la pose pas,
/// et exiger une déclaration explicite pour un cas non dangereux ferait échouer tous les postes du
/// jour où on l'ajoute.
pub fn refuser_si_production() -> Result<(), ErreurSecret> {
    valider_environnement(std::env::var(VAR_ENVIRONNEMENT).ok())
}

/// La règle, séparée de sa source — même raison que [`valider_cle_jwt`].
fn valider_environnement(environnement: Option<String>) -> Result<(), ErreurSecret> {
    let Some(environnement) = environnement else {
        return Ok(());
    };

    if matches!(
        environnement.trim().to_lowercase().as_str(),
        "production" | "prod"
    ) {
        return Err(ErreurSecret::ProductionRefusee { environnement });
    }

    Ok(())
}

/// Refuse de démarrer si la clé de signature est absente ou trop courte.
///
/// Appelée une fois, au démarrage — même patron que `contexte::verifier_derogation()`.
pub fn verifier_secrets_de_demarrage() {
    match cle_jwt() {
        Ok(cle) => {
            tracing::info!(
                octets = cle.len(),
                "clé de signature des sessions chargée depuis l'environnement"
            );
        }
        Err(erreur) => {
            panic!(
                "{erreur}\n\
                 \n\
                 Les jetons de session sont signés avec {VAR_CLE_JWT}, lue de l'environnement et \
                 jamais du binaire (principe IX). Sans elle, aucune session ne peut être délivrée \
                 ni vérifiée — le serveur ne servirait que des `401`.\n\
                 \n\
                 Produire une clé :\n\
                 \n\
                     openssl rand -base64 48\n\
                 \n\
                 **Une clé par environnement.** Partager celle de développement avec la \
                 production reviendrait à publier la production : le dépôt, les images et les \
                 archives des postes la portent tous.\n\
                 \n\
                 Changer cette clé invalide toutes les sessions en cours — c'est le recours de \
                 dernier ressort si elle fuit, et c'est pourquoi la révocation ordinaire passe par \
                 Redis et non par une rotation de clé."
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn une_cle_absente_est_refusee() {
        assert!(matches!(
            valider_cle_jwt(None),
            Err(ErreurSecret::Absente { .. })
        ));
    }

    #[test]
    fn une_cle_de_trente_et_un_octets_est_refusee() {
        let erreur = valider_cle_jwt(Some("a".repeat(31))).expect_err("31 octets sont refusés");
        assert!(
            matches!(erreur, ErreurSecret::TropCourte { longueur: 31, .. }),
            "erreur inattendue : {erreur}"
        );
    }

    #[test]
    fn une_cle_de_trente_deux_octets_est_acceptee() {
        assert_eq!(
            valider_cle_jwt(Some("a".repeat(32)))
                .expect("32 octets suffisent")
                .len(),
            32
        );
    }

    /// **Le refus porte sur les octets, pas sur les caractères.** Seize « é » font trente-deux
    /// octets en UTF-8 et huit caractères de moins que le plancher : la distinction est vérifiée
    /// pour qu'un remaniement ne remplace pas `len()` par `chars().count()`, ce qui refuserait une
    /// clé parfaitement valide.
    #[test]
    fn la_longueur_se_compte_en_octets() {
        let cle = valider_cle_jwt(Some("é".repeat(16))).expect("seize « é » font 32 octets");
        assert_eq!(cle.len(), 32);
        assert_eq!("é".repeat(16).chars().count(), 16);
    }

    #[test]
    fn les_seeds_refusent_la_production() {
        for declaration in ["production", "prod", "  PROD  ", "Production"] {
            assert!(
                matches!(
                    valider_environnement(Some(declaration.to_owned())),
                    Err(ErreurSecret::ProductionRefusee { .. })
                ),
                "« {declaration} » a franchi la garde : ni la casse ni les espaces ne doivent \
                 ouvrir la porte"
            );
        }
    }

    /// **Absente vaut « pas la production ».** Exiger une déclaration explicite pour un cas non
    /// dangereux ferait échouer tous les postes de développement le jour où on l'ajoute — et la
    /// variable finirait posée à `developpement` dans un fichier commité, où elle serait un jour
    /// copiée sur le serveur.
    #[test]
    fn l_environnement_non_declare_vaut_pas_la_production() {
        assert!(valider_environnement(None).is_ok());
        assert!(valider_environnement(Some("developpement".to_owned())).is_ok());
        assert!(valider_environnement(Some("recette".to_owned())).is_ok());
    }
}
