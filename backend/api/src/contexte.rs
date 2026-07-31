//! Contexte d'appel — quel tenant, quel compte.
//!
//! # Un provisoire nommé, daté et borné
//!
//! **L'authentification est CPT-01, hors du périmètre de ce cycle.** Le contrat HTTP annonce
//! pourtant `security(("bearer" = []))` sur les endpoints de notes, et il a raison : c'est bien
//! un jeton qui portera le tenant et le compte.
//!
//! En attendant, le contexte se lit dans deux en-têtes. Ce n'est pas un raccourci commode, c'est
//! le seul moyen de rendre la porte **P-08** réelle dès maintenant : un test d'isolation par
//! endpoint suppose de pouvoir se présenter comme deux tenants différents. Sans cela, la porte
//! attendrait CPT-01, et trois cycles auraient écrit des endpoints sans qu'elle les voie jamais.
//!
//! **Dérogation** (constitution, § Dérogation — temporaire, nommée, datée, avec sa condition de
//! levée) :
//!
//! | Élément | Valeur |
//! |---|---|
//! | Nom | `CONTEXTE_PAR_EN_TETES` |
//! | Ouverte le | 2026-07-31, cycle 001 |
//! | Condition de levée | CPT-01 — le contexte vient du jeton vérifié, ces en-têtes disparaissent |
//! | Effet si non levée | Toute personne joignant l'API choisit son tenant. **Inacceptable en production.** |
//!
//! Le garde-fou est un refus au démarrage : [`verifier_dérogation`] fait échouer le binaire si
//! `KAYA_CONTEXTE_PAR_EN_TETES` n'est pas explicitement activé. Une dérogation qu'on peut oublier
//! d'ouvrir se retrouve ouverte en production.

use actix_web::{FromRequest, HttpRequest, dev::Payload};
use std::future::{Ready, ready};
use uuid::Uuid;

pub const EN_TETE_TENANT: &str = "x-kaya-tenant";
pub const EN_TETE_COMPTE: &str = "x-kaya-compte";

/// Qui appelle, et pour quel tenant.
#[derive(Debug, Clone, Copy)]
pub struct ContexteAppel {
    pub tenant_id: Uuid,
    pub compte_id: Uuid,
}

/// La dérogation est-elle ouverte ?
pub fn contexte_par_en_tetes_actif() -> bool {
    matches!(
        std::env::var("KAYA_CONTEXTE_PAR_EN_TETES").as_deref(),
        Ok("1") | Ok("true")
    )
}

/// Refuse de démarrer si la dérogation n'est pas explicitement ouverte.
///
/// Appelée au démarrage, pas à chaque requête : une vérification par requête serait un coût
/// permanent pour une décision qui ne change jamais en cours d'exécution.
pub fn verifier_derogation() {
    if !contexte_par_en_tetes_actif() {
        panic!(
            "Aucun mécanisme d'authentification n'est disponible.\n\
             L'authentification par jeton est livrée par CPT-01. En attendant, le contexte de \
             tenant se lit dans les en-têtes {EN_TETE_TENANT} et {EN_TETE_COMPTE}, ce qui laisse \
             tout appelant choisir son tenant.\n\
             Pour l'accepter en développement ou en test : KAYA_CONTEXTE_PAR_EN_TETES=1.\n\
             **Jamais en production.**"
        );
    }
    tracing::warn!(
        derogation = "CONTEXTE_PAR_EN_TETES",
        "le contexte de tenant vient d'en-têtes non authentifiés — levée par CPT-01"
    );
}

impl FromRequest for ContexteAppel {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let lire = |nom: &str| -> Option<Uuid> {
            req.headers()
                .get(nom)
                .and_then(|v| v.to_str().ok())
                .and_then(|v| Uuid::parse_str(v).ok())
        };

        let resultat = match (lire(EN_TETE_TENANT), lire(EN_TETE_COMPTE)) {
            (Some(tenant_id), Some(compte_id)) => Ok(ContexteAppel {
                tenant_id,
                compte_id,
            }),
            // `401`, pas `400` : l'appelant n'est pas identifié. La distinction compte pour le
            // client, qui doit réessayer après authentification et non corriger sa requête.
            _ => Err(actix_web::error::ErrorUnauthorized(
                "contexte d'appel absent ou invalide",
            )),
        };

        ready(resultat)
    }
}
