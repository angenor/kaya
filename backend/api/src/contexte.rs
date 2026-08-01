//! Contexte d'appel — **extrait du jeton vérifié**, et la dérogation du cycle 001 est levée.
//!
//! # Ce que ce fichier remplace
//!
//! Jusqu'à CPT-01, le tenant et le compte se lisaient dans deux en-têtes non authentifiés,
//! `x-kaya-tenant` et `x-kaya-compte`, sous une dérogation nommée `CONTEXTE_PAR_EN_TETES`. Elle
//! n'était pas un raccourci commode : sans elle, la porte P-08 n'aurait eu aucun moyen de se
//! présenter comme deux tenants différents, et trois cycles auraient écrit des endpoints sans
//! qu'elle les voie jamais.
//!
//! | Élément | Valeur |
//! |---|---|
//! | Nom | `CONTEXTE_PAR_EN_TETES` |
//! | Ouverte le | 2026-07-31, cycle 001 |
//! | **Levée le** | **2026-08-01, cycle 003, tâche T030** |
//! | Condition de levée | *« CPT-01 — le contexte vient du jeton vérifié, ces en-têtes disparaissent »* |
//!
//! Les deux en-têtes, la variable `KAYA_CONTEXTE_PAR_EN_TETES` et la fonction
//! `verifier_derogation()` n'existent plus. **C'est la seule façon de lever une dérogation** : en
//! retirant le code, pas en cessant de l'employer.
//!
//! # Deux vérifications, et la seconde est celle qui coûte
//!
//! 1. **La signature** — locale, quelques microsecondes, sans aucun accès réseau. Elle établit que
//!    le jeton vient bien de ce serveur et qu'il n'a pas expiré.
//! 2. **La liste de révocation** — un aller-retour Redis, **à chaque requête authentifiée**.
//!
//! La seconde est le prix de la « coupure immédiate au départ d'un employé » (cadrage §12.2). Un
//! jeton signé reste mathématiquement valide jusqu'à son expiration, quoi qu'il arrive : sans
//! cette consultation, révoquer une session laisserait l'accès ouvert jusqu'à 60 minutes — et le
//! jeton de rafraîchissement, 90 jours. C'est le seul recours contre un téléphone volé avant
//! l'enrôlement d'appareil de CPT-05.
//!
//! **Redis cesse donc d'être optionnel** : sans lui, aucune requête authentifiée n'aboutit. Le
//! choix est écrit ici et dans `secrets.rs` plutôt que découvert à la première panne.
//!
//! # Les permissions viennent du jeton, pas d'une lecture
//!
//! Elles y ont été mises à la connexion (`session/jeton.rs`). Les relire en base à chaque requête
//! coûterait une requête SQL sur le chemin le plus chaud du produit ; le prix de l'autre choix est
//! qu'un rôle retiré prend effet au **rafraîchissement suivant**, soit au plus la durée du jeton
//! d'accès. C'est l'hypothèse 5 de la spec, et c'est un arbitrage, pas un oubli.

use std::future::Future;
use std::pin::Pin;

use actix_web::{FromRequest, HttpRequest, dev::Payload};
use uuid::Uuid;

use kaya_comptes::session::jeton;

use crate::application::EtatApplication;

/// Qui appelle, pour quel tenant, avec quels droits.
///
/// **Toutes les données viennent du jeton vérifié.** Aucun champ n'est lu d'un en-tête, d'un
/// paramètre de requête ni d'un corps : c'est ce qui distingue ce fichier de celui qu'il remplace.
#[derive(Debug, Clone)]
pub struct ContexteAppel {
    pub tenant_id: Uuid,
    pub compte_id: Uuid,
    /// La session — c'est **elle** que la liste de révocation désigne, et elle que l'utilisateur
    /// coupe depuis « Appareils connectés ».
    pub session_id: Uuid,
    /// L'établissement actif. `None` pour un compte de portée éditeur.
    pub etablissement_actif: Option<Uuid>,
    /// **L'union** des permissions des rôles portés (FR-017).
    pub permissions: Vec<String>,
}

impl ContexteAppel {
    /// L'appelant détient-il cette permission ?
    ///
    /// **Aucune méthode ne rend ni n'accepte un rôle**, ici comme dans `AccessController` : un
    /// consommateur qui brancherait sur un rôle recréerait la hiérarchie que le principe VII
    /// interdit.
    pub fn detient(&self, permission: &str) -> bool {
        self.permissions.iter().any(|p| p == permission)
    }
}

/// Le schéma d'authentification, écrit une fois.
const PREFIXE_BEARER: &str = "Bearer ";

impl FromRequest for ContexteAppel {
    type Error = actix_web::Error;
    // Un `Future` boîté plutôt qu'un `Ready` : la consultation de la liste de révocation est un
    // aller-retour Redis, donc asynchrone. C'est le seul changement de forme que la levée de la
    // dérogation impose à l'extracteur — et il est structurant, puisqu'il rend la consultation
    // impossible à oublier : elle est dans le chemin que **tout** handler authentifié traverse.
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let etat = req.app_data::<actix_web::web::Data<EtatApplication>>().cloned();

        let jeton_presente = req
            .headers()
            .get(actix_web::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix(PREFIXE_BEARER))
            .map(str::to_owned);

        Box::pin(async move {
            let Some(etat) = etat else {
                // Ne peut arriver que si l'état n'a pas été injecté au montage — donc jamais en
                // production, et à la première requête d'un test mal monté. Le message le dit.
                return Err(actix_web::error::ErrorInternalServerError(
                    "état applicatif absent : l'application n'a pas été montée avec EtatApplication",
                ));
            };

            let Some(jeton_presente) = jeton_presente else {
                return Err(non_authentifie());
            };

            let claims = jeton::verifier_acces(&etat.cle_jwt, &jeton_presente)
                .map_err(|_| non_authentifie())?;

            // **La consultation de la liste de révocation.** Elle vient après la signature et
            // avant tout le reste : vérifier d'abord la signature évite un aller-retour Redis pour
            // un jeton forgé, que n'importe qui peut produire en quantité.
            let revoquee = etat
                .entrepot
                .est_revoquee(claims.sid)
                .await
                .map_err(|erreur| {
                    tracing::error!(erreur = %erreur, "liste de révocation injoignable");
                    // **Redis injoignable refuse la requête**, il ne la laisse pas passer. C'est
                    // le choix le plus coûteux en disponibilité et le seul défendable : laisser
                    // passer reviendrait à désactiver la révocation à chaque panne de Redis,
                    // c'est-à-dire exactement au moment où personne ne le remarquerait.
                    actix_web::error::ErrorServiceUnavailable(
                        "vérification de session indisponible",
                    )
                })?;

            if revoquee {
                return Err(non_authentifie());
            }

            Ok(ContexteAppel {
                tenant_id: claims.tenant,
                compte_id: claims.sub,
                session_id: claims.sid,
                etablissement_actif: claims.etablissement,
                permissions: claims.perms,
            })
        })
    }
}

/// `401`, jamais `400` — et le refus ne dit **pas** pourquoi.
///
/// Jeton absent, mal formé, signé d'une autre clé, expiré ou révoqué : un seul refus. Distinguer
/// « expiré » de « révoqué » apprendrait à qui détient un jeton volé que sa cible s'en est aperçue,
/// donc qu'il faut se dépêcher.
fn non_authentifie() -> actix_web::Error {
    actix_web::error::ErrorUnauthorized("non authentifié")
}
