//! Handlers de session — **les deux seules opérations publiques du produit sont ici**.
//!
//! `session_ouvrir` et `session_rafraichir` ne portent pas `security(("bearer" = []))`, et c'est
//! la **seule liste d'exceptions du produit**. Elle est nommée, fermée, et le test d'isolation la
//! connaît : une opération nouvelle qui s'y ajouterait ferait échouer la porte P-08 au lieu de
//! passer inaperçue.
//!
//! # L'ordre de montage, et ce qu'il coûterait de l'inverser
//!
//! Actix essaie les scopes **dans l'ordre d'enregistrement**, et un scope qui accepte le préfixe
//! rend `404` sans laisser sa chance aux suivants. D'où, du plus spécifique au plus général :
//! `/session/actives/{session_id}` → `/session/actives` → `/session/rafraichir` → `/session/moi`
//! → `/session`. Monter `/session` en premier ferait disparaître les quatre autres — sans erreur
//! de compilation, et avec un contrat OpenAPI parfaitement exact.

use actix_web::{HttpRequest, HttpResponse, delete, get, post, web};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use utoipa::ToSchema;
use uuid::Uuid;

use kaya_comptes::session::modele::{ErreurSession, SessionVue};

use crate::application::EtatApplication;
use crate::contexte::ContexteAppel;
use crate::routes::erreurs::{CorpsErreur, interne};

// =================================================================================================
//  Corps de requête et de réponse
// =================================================================================================

/// Corps d'ouverture de session.
#[derive(Debug, Deserialize, ToSchema)]
pub struct OuvrirRequete {
    /// Numéro de téléphone ou courriel. **Un seul champ** : l'utilisateur ne sait pas dans quelle
    /// colonne son identifiant est rangé, et le lui demander serait lui faire porter un détail de
    /// modèle de données.
    pub identifiant: String,
    pub mot_de_passe: String,
    /// Établissement souhaité. Sans lui, **le premier accessible par ordre stable** devient actif.
    #[serde(default)]
    pub etablissement_id: Option<Uuid>,
    /// Libellé d'appareil, purement indicatif — il sert à ce que l'utilisateur reconnaisse **son**
    /// téléphone dans la liste avant de couper l'autre.
    #[serde(default)]
    pub libelle_appareil: Option<String>,
}

/// Corps de rafraîchissement.
#[derive(Debug, Deserialize, ToSchema)]
pub struct RafraichirRequete {
    pub rafraichissement: String,
    #[serde(default)]
    pub etablissement_id: Option<Uuid>,
}

/// Ce qu'une ouverture ou un rafraîchissement rend.
///
/// **Le condensat n'y figure pas, et aucune structure de ce fichier n'a de champ où le mettre.**
#[derive(Debug, Serialize, ToSchema)]
pub struct SessionOuverteVue {
    /// Jeton d'accès, à porter dans `Authorization: Bearer`.
    pub acces: String,
    /// Durée de vie du jeton d'accès, en secondes — ce que le client met dans son minuteur.
    pub expire_dans_s: i64,
    /// Jeton de rafraîchissement. **À ranger dans le stockage sécurisé de la plateforme**
    /// (Keystore / Keychain), jamais dans un stockage web ordinaire.
    pub rafraichissement: String,
    pub compte: CompteConnecteVue,
    /// **L'union** des permissions des rôles portés sur l'établissement actif (FR-017).
    ///
    /// Le front la lit **ici**, jamais en décodant le jeton (research R-06) : deux sources pour la
    /// même information, et une seule fait autorité.
    pub permissions: Vec<String>,
    /// Les établissements accessibles. Le sélecteur permanent est **ETB-06**, hors périmètre.
    pub etablissements: Vec<Uuid>,
}

/// Le compte connecté, tel que la réponse le rend.
#[derive(Debug, Serialize, ToSchema)]
pub struct CompteConnecteVue {
    pub compte_id: Uuid,
    pub tenant_id: Uuid,
    /// L'établissement actif. `None` pour un compte de portée éditeur.
    pub etablissement_actif: Option<Uuid>,
}

/// Ce que `session_moi` rend — **le contexte tel que le serveur le voit**.
///
/// Il n'apprend rien au client qu'il ne sache déjà ; il sert à **vérifier** qu'un jeton est encore
/// valide sans avoir à provoquer une opération métier, et à relire les permissions après une
/// reprise de l'application.
#[derive(Debug, Serialize, ToSchema)]
pub struct MoiVue {
    pub compte_id: Uuid,
    pub tenant_id: Uuid,
    pub session_id: Uuid,
    pub etablissement_actif: Option<Uuid>,
    pub permissions: Vec<String>,
}

/// Une page de sessions actives.
#[derive(Debug, Serialize, ToSchema)]
pub struct SessionsActivesVue {
    pub elements: Vec<SessionVue>,
}

// =================================================================================================
//  1 · Ouvrir — PUBLIQUE
// =================================================================================================

/// Ouvre une session.
///
/// **`401 identifiants_invalides` est le seul code d'échec d'authentification** : jamais
/// `compte_inconnu`, jamais `mot_de_passe_invalide`, jamais `compte_desactive`, jamais
/// « trop de tentatives » (FR-012). Et le **temps de réponse** est du même ordre dans tous les
/// cas — c'est la moitié de l'exigence que le code seul ne tient pas, et
/// `backend/tests/authentification_indiscernable.rs` la mesure.
#[utoipa::path(
    operation_id = "session_ouvrir",
    tag = "session",
    request_body = OuvrirRequete,
    responses(
        (status = 200, description = "Session ouverte", body = SessionOuverteVue),
        (status = 401, description = "Identifiants invalides — code unique", body = CorpsErreur),
        (status = 422, description = "Méthode d'authentification non implémentée", body = CorpsErreur),
    ),
)]
#[post("")]
pub async fn ouvrir(
    etat: web::Data<EtatApplication>,
    requete: HttpRequest,
    corps: web::Json<OuvrirRequete>,
) -> Result<HttpResponse, actix_web::Error> {
    let corps = corps.into_inner();

    let ouverte = etat
        .service_authentification()
        .ouvrir(
            &corps.identifiant,
            &corps.mot_de_passe,
            corps.etablissement_id,
            corps.libelle_appareil,
            &origine(&requete),
        )
        .await
        .map_err(en_reponse)?;

    Ok(HttpResponse::Ok().json(SessionOuverteVue {
        acces: ouverte.jetons.acces,
        expire_dans_s: ouverte.jetons.expire_dans_s,
        rafraichissement: ouverte.jetons.rafraichissement,
        compte: CompteConnecteVue {
            compte_id: ouverte.compte_id,
            tenant_id: ouverte.tenant_id,
            etablissement_actif: ouverte.etablissement_actif,
        },
        permissions: ouverte.permissions.into_iter().collect(),
        etablissements: ouverte.etablissements,
    }))
}

// =================================================================================================
//  2 · Rafraîchir — PUBLIQUE
// =================================================================================================

/// Rafraîchit une session — **rotation à chaque usage**.
///
/// Un jeton **déjà consommé** est un signal, pas une erreur ordinaire : il signifie qu'une copie
/// circule. La réponse est `401` **et toute la famille de jetons est révoquée** — pas seulement
/// celui qui est présenté. Révoquer le seul laisserait le voleur et la victime en course, et le
/// premier des deux gagnerait.
///
/// Les permissions sont **recalculées** ici : c'est le moment où un rôle retiré prend effet.
#[utoipa::path(
    operation_id = "session_rafraichir",
    tag = "session",
    request_body = RafraichirRequete,
    responses(
        (status = 200, description = "Session rafraîchie", body = SessionOuverteVue),
        (status = 401, description = "Jeton inconnu, révoqué ou déjà consommé", body = CorpsErreur),
    ),
)]
#[post("")]
pub async fn rafraichir(
    etat: web::Data<EtatApplication>,
    corps: web::Json<RafraichirRequete>,
) -> Result<HttpResponse, actix_web::Error> {
    let corps = corps.into_inner();

    let ouverte = etat
        .service_authentification()
        .rafraichir(&corps.rafraichissement, corps.etablissement_id)
        .await
        .map_err(en_reponse)?;

    Ok(HttpResponse::Ok().json(SessionOuverteVue {
        acces: ouverte.jetons.acces,
        expire_dans_s: ouverte.jetons.expire_dans_s,
        rafraichissement: ouverte.jetons.rafraichissement,
        compte: CompteConnecteVue {
            compte_id: ouverte.compte_id,
            tenant_id: ouverte.tenant_id,
            etablissement_actif: ouverte.etablissement_actif,
        },
        permissions: ouverte.permissions.into_iter().collect(),
        etablissements: ouverte.etablissements,
    }))
}

// =================================================================================================
//  3 · Fermer
// =================================================================================================

/// Ferme la session courante — la déconnexion volontaire.
///
/// **N'écrit aucune entrée d'audit et n'émet aucun événement** : se déconnecter de son propre
/// appareil n'est pas un acte d'administration. Révoquer une **autre** session, si.
#[utoipa::path(
    operation_id = "session_fermer",
    tag = "session",
    responses(
        (status = 204, description = "Session fermée"),
        (status = 401, description = "Non authentifié"),
    ),
    security(("bearer" = []))
)]
#[delete("")]
pub async fn fermer(
    etat: web::Data<EtatApplication>,
    contexte: ContexteAppel,
) -> Result<HttpResponse, actix_web::Error> {
    etat.service_authentification()
        .fermer(contexte.compte_id, contexte.session_id)
        .await
        .map_err(en_reponse)?;

    Ok(HttpResponse::NoContent().finish())
}

// =================================================================================================
//  4 · Moi
// =================================================================================================

/// Rend le contexte de la session courante.
#[utoipa::path(
    operation_id = "session_moi",
    tag = "session",
    responses(
        (status = 200, description = "Contexte de la session courante", body = MoiVue),
        (status = 401, description = "Non authentifié"),
    ),
    security(("bearer" = []))
)]
#[get("")]
pub async fn moi(contexte: ContexteAppel) -> Result<HttpResponse, actix_web::Error> {
    Ok(HttpResponse::Ok().json(MoiVue {
        compte_id: contexte.compte_id,
        tenant_id: contexte.tenant_id,
        session_id: contexte.session_id,
        etablissement_actif: contexte.etablissement_actif,
        permissions: contexte.permissions,
    }))
}

// =================================================================================================
//  5 · Lister les sessions actives
// =================================================================================================

/// Les sessions actives du compte appelant — **« Appareils connectés »**.
///
/// Reconstruites depuis Redis. Si Redis a été vidé, la liste est vide et tout le monde s'est
/// reconnecté : c'est exact, pas une panne (research R-01).
#[utoipa::path(
    operation_id = "session_lister_actives",
    tag = "session",
    responses(
        (status = 200, description = "Sessions actives du compte appelant", body = SessionsActivesVue),
        (status = 401, description = "Non authentifié"),
    ),
    security(("bearer" = []))
)]
#[get("")]
pub async fn lister_actives(
    etat: web::Data<EtatApplication>,
    contexte: ContexteAppel,
) -> Result<HttpResponse, actix_web::Error> {
    let elements = etat
        .service_authentification()
        .lister_actives(contexte.compte_id, contexte.session_id)
        .await
        .map_err(en_reponse)?;

    Ok(HttpResponse::Ok().json(SessionsActivesVue { elements }))
}

// =================================================================================================
//  6 · Révoquer une session
// =================================================================================================

/// Révoque une session — **effet immédiat**.
///
/// La session est marquée dans la liste de révocation Redis, consultée à chaque requête
/// authentifiée : le jeton d'accès en circulation cesse d'être accepté **à l'appel suivant**, sans
/// attendre son expiration. C'est la « coupure immédiate au départ d'un employé » du cadrage
/// §12.2, et le seul recours contre un téléphone volé avant l'enrôlement d'appareil de CPT-05.
///
/// **Révoquer sa propre session ne demande aucune permission.** Révoquer celle d'un autre exige
/// `cpt.session.revoquer` — mais ce cycle ne livre pas encore la révocation croisée : le service
/// ne sait couper que les sessions du compte appelant, faute d'un annuaire des sessions par
/// tenant. Écrit ici plutôt que découvert : la garde existe, la fonctionnalité qu'elle protège
/// viendra avec l'écran d'administration des appareils (CPT-05, tranche T4).
#[utoipa::path(
    operation_id = "session_revoquer",
    tag = "session",
    params(("session_id" = Uuid, Path, description = "Identifiant de la session à couper")),
    responses(
        (status = 204, description = "Session révoquée — effet immédiat"),
        (status = 401, description = "Non authentifié"),
        (status = 403, description = "Permission absente", body = CorpsErreur),
    ),
    security(("bearer" = []))
)]
#[delete("")]
pub async fn revoquer(
    etat: web::Data<EtatApplication>,
    contexte: ContexteAppel,
    chemin: web::Path<Uuid>,
) -> Result<HttpResponse, actix_web::Error> {
    let session_id = chemin.into_inner();

    etat.service_authentification()
        .revoquer(
            contexte.compte_id,
            contexte.tenant_id,
            contexte.compte_id,
            session_id,
            kaya_comptes::session::DureesSession::repli().acces_s,
        )
        .await
        .map_err(en_reponse)?;

    Ok(HttpResponse::NoContent().finish())
}

// =================================================================================================
//  Outils
// =================================================================================================

/// L'origine de l'appel, pour le compteur de tentatives.
///
/// # `X-Forwarded-For` n'est pas digne de confiance, et on l'emploie quand même
///
/// N'importe qui peut poser cet en-tête. En tirer une décision de sécurité serait naïf — mais ce
/// n'en est pas une : il alimente un **compteur**, dont le pire abus est qu'un attaquant fasse
/// varier sa valeur pour échapper au plafond par origine. Il resterait alors pris par le plafond
/// **par identifiant**, qui est celui qui protège un compte donné. C'est précisément pourquoi il
/// en faut deux.
///
/// Sans répartiteur devant l'API, l'en-tête est absent et l'adresse de pair fait foi. Avec un
/// répartiteur, l'adresse de pair serait celle du répartiteur — donc la même pour tout le monde,
/// et le compteur par origine deviendrait un compteur global.
fn origine(requete: &HttpRequest) -> String {
    requete
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        // Le premier élément de la liste est le client d'origine ; les suivants sont les
        // répartiteurs traversés.
        .and_then(|v| v.split(',').next())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_owned)
        .or_else(|| requete.peer_addr().map(|a| a.ip().to_string()))
        // Aucune origine identifiable : tout retombe dans le même seau. C'est le comportement
        // sûr — l'inconnu est plafonné, pas exempté.
        .unwrap_or_else(|| "inconnue".to_owned())
}

/// Traduit une erreur de session en réponse HTTP.
///
/// **Le refus d'authentification ne dit jamais pourquoi** — c'est FR-012, et c'est ici que la
/// règle se voit : trois causes internes distinctes tombent sur un seul code.
fn en_reponse(erreur: ErreurSession) -> actix_web::Error {
    match erreur {
        ErreurSession::IdentifiantsInvalides => CorpsErreur::nouveau(
            "identifiants_invalides",
            None,
            "authentification refusée — la cause exacte est dans les journaux, jamais ici"
                .to_owned(),
        )
        .en_401(),

        ErreurSession::SessionInvalide => CorpsErreur::nouveau(
            "session_invalide",
            None,
            "jeton de rafraîchissement inconnu, révoqué ou déjà consommé".to_owned(),
        )
        .en_401(),

        ErreurSession::MethodeNonImplementee(code) => CorpsErreur::nouveau(
            "methode_non_implementee",
            Some(code),
            "la méthode d'authentification de ce compte n'est pas servie par ce produit".to_owned(),
        )
        .en_422(),

        autre => interne("service de session", autre),
    }
}

/// Horodatage d'autorité, exposé pour le diagnostic des tests de durée.
#[allow(dead_code)]
fn maintenant() -> OffsetDateTime {
    OffsetDateTime::now_utc()
}
