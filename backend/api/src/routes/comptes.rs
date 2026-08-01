//! Handlers des comptes et des rôles — **CPT-01, CPT-02**, opérations 10 à 16 du contrat.
//!
//! # Le condensat n'est rendu sur aucune réponse, sur aucun chemin
//!
//! Trois barrières, et aucune n'est une convention : [`CompteVue`] n'a pas de champ où le mettre,
//! `compte::repository::lire` ne sélectionne pas la colonne, et aucune structure de ce fichier
//! n'en porte. Une seule barrière se défait par distraction ; trois se défont par décision.
//!
//! # L'ordre de montage, et ce qu'il coûterait de l'inverser
//!
//! Actix essaie les scopes **dans l'ordre d'enregistrement**, et un scope qui accepte le préfixe
//! rend `404` sans laisser sa chance aux suivants. D'où, du plus spécifique au plus général :
//! `/comptes/{id}/roles/{role_code}` → `/comptes/{id}/roles` → `/comptes/{id}/etat` →
//! `/comptes/{id}/mot-de-passe` → `/comptes/{id}` → `/comptes`. Monter `/comptes` en premier
//! ferait disparaître les cinq autres — sans erreur de compilation, et avec un contrat OpenAPI
//! parfaitement exact.
//!
//! # Les gardes de permission sont ici, et nulle part ailleurs
//!
//! Aucun service de `socle/` ne consulte les permissions de l'appelant. Les mêler au service
//! obligerait à les passer partout, et le jour où l'un d'eux oublierait de les consulter, rien ne
//! le signalerait. Voir `api/src/securite.rs`.

use actix_web::{HttpResponse, delete, get, post, put, web};
use serde::Deserialize;
use time::OffsetDateTime;
use utoipa::ToSchema;
use uuid::Uuid;

use kaya_comptes::compte::{CompteVue, CreerCompte, ErreurCompte};
use kaya_comptes::roles::{AttribuerRole, ErreurRoles};
use kaya_etablissements::Issue;

use crate::application::EtatApplication;
use crate::contexte::ContexteAppel;
use crate::routes::erreurs::{CorpsErreur, interne};
use crate::securite;

/// Permissions du contrat, nommées une fois.
const PERM_LIRE: &str = "cpt.compte.lire";
const PERM_GERER: &str = "cpt.compte.gerer";
const PERM_ATTRIBUER: &str = "cpt.role.attribuer";

// =================================================================================================
//  Corps de requête
// =================================================================================================

/// Corps de création d'un compte.
///
/// **Le mot de passe entre ici et nulle part ailleurs.** Il n'est ni journalisé, ni rendu, ni
/// porté par la charge utile de l'événement.
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreerCompteRequete {
    /// UUID v7 **généré par le client** (principe VI) : c'est lui qui rend le rejeu inoffensif.
    pub id: Uuid,
    pub personne_id: Uuid,
    /// E.164. **Au moins un des deux identifiants** est obligatoire.
    #[serde(default)]
    pub identifiant_telephone: Option<String>,
    #[serde(default)]
    pub identifiant_email: Option<String>,
    pub mot_de_passe: String,
    /// Indicatif — **jamais employé par une règle métier**.
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub horodatage_client: Option<OffsetDateTime>,
}

/// Corps de changement d'état.
#[derive(Debug, Deserialize, ToSchema)]
pub struct ChangerEtatRequete {
    pub actif: bool,
}

/// Corps de changement de mot de passe.
#[derive(Debug, Deserialize, ToSchema)]
pub struct ChangerMotDePasseRequete {
    /// **Fourni par un compte qui agit sur lui-même**, absent quand un habilité agit sur un autre.
    ///
    /// Le demander à l'habilité rendrait l'opération impossible dans le seul cas où elle sert :
    /// quelqu'un a perdu son mot de passe.
    #[serde(default)]
    pub mot_de_passe_actuel: Option<String>,
    pub nouveau_mot_de_passe: String,
}

/// Corps d'attribution de rôle.
#[derive(Debug, Deserialize, ToSchema)]
pub struct AttribuerRoleRequete {
    /// UUID v7 **généré par le client**.
    pub id: Uuid,
    pub role_code: String,
    /// **Obligatoire** pour un rôle de portée `ETABLISSEMENT`, **interdit** pour `admin_editeur`.
    #[serde(default)]
    pub etablissement_id: Option<Uuid>,
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub horodatage_client: Option<OffsetDateTime>,
}

/// Filtres de la liste des comptes — **combinables**, chacun optionnel.
#[derive(Debug, Deserialize, ToSchema, utoipa::IntoParams)]
pub struct FiltresComptes {
    #[serde(default)]
    pub etablissement_id: Option<Uuid>,
    #[serde(default)]
    pub actif: Option<bool>,
    #[serde(default)]
    pub role_code: Option<String>,
}

/// L'établissement d'un retrait de rôle — en paramètre de requête, jamais dans le chemin.
///
/// Le mettre dans le chemin aurait produit `/comptes/{id}/roles/{role}/{etablissement}`, où
/// l'absence d'établissement — cas d'`admin_editeur` — n'aurait aucune écriture possible.
#[derive(Debug, Deserialize, ToSchema, utoipa::IntoParams)]
pub struct CibleRetrait {
    #[serde(default)]
    pub etablissement_id: Option<Uuid>,
}

// =================================================================================================
//  10 · Créer un compte
// =================================================================================================

/// Crée un compte.
///
/// **`200` sur rejeu, pas `409`** : un terminal qui vide sa file après une coupure ne doit pas
/// voir d'erreur pour une écriture déjà acceptée.
#[utoipa::path(
    operation_id = "compte_creer",
    tag = "comptes",
    request_body = CreerCompteRequete,
    responses(
        (status = 201, description = "Compte créé", body = CompteVue),
        (status = 200, description = "Compte déjà créé (rejeu idempotent)", body = CompteVue),
        (status = 401, description = "Non authentifié"),
        (status = 403, description = "Permission absente", body = CorpsErreur),
        (status = 422, description = "Identifiant absent ou refusé, mot de passe refusé", body = CorpsErreur),
    ),
    security(("bearer" = []))
)]
#[post("")]
pub async fn creer(
    etat: web::Data<EtatApplication>,
    contexte: ContexteAppel,
    corps: web::Json<CreerCompteRequete>,
) -> Result<HttpResponse, actix_web::Error> {
    securite::exiger(&contexte, PERM_GERER)?;
    let corps = corps.into_inner();

    let (compte, issue) = etat
        .service_comptes()
        .creer(
            contexte.tenant_id,
            CreerCompte {
                id: corps.id,
                personne_id: corps.personne_id,
                identifiant_telephone: corps.identifiant_telephone,
                identifiant_email: corps.identifiant_email,
                mot_de_passe: corps.mot_de_passe,
                horodatage_client: corps.horodatage_client,
            },
        )
        .await
        .map_err(en_reponse_compte)?;

    Ok(match issue {
        Issue::Creee => HttpResponse::Created().json(compte),
        Issue::DejaPresente => HttpResponse::Ok().json(compte),
    })
}

// =================================================================================================
//  11 · Lister les comptes
// =================================================================================================

/// Liste les comptes du tenant — écran `G3`.
///
/// Rend, par compte : identité, état, **et les rôles portés avec leur établissement**. Un même
/// compte peut être caissier ici et réceptionniste là ; une liste de codes sans établissement
/// serait fausse.
#[utoipa::path(
    operation_id = "compte_lister",
    tag = "comptes",
    params(FiltresComptes),
    responses(
        (status = 200, description = "Les comptes", body = Vec<CompteVue>),
        (status = 401, description = "Non authentifié"),
        (status = 403, description = "Permission absente", body = CorpsErreur),
    ),
    security(("bearer" = []))
)]
#[get("")]
pub async fn lister(
    etat: web::Data<EtatApplication>,
    contexte: ContexteAppel,
    filtres: web::Query<FiltresComptes>,
) -> Result<HttpResponse, actix_web::Error> {
    securite::exiger(&contexte, PERM_LIRE)?;
    let filtres = filtres.into_inner();

    let comptes = etat
        .service_comptes()
        .lister(
            contexte.tenant_id,
            filtres.etablissement_id,
            filtres.actif,
            filtres.role_code.as_deref(),
        )
        .await
        .map_err(en_reponse_compte)?;

    Ok(HttpResponse::Ok().json(comptes))
}

// =================================================================================================
//  12 · Lire un compte
// =================================================================================================

/// Lit un compte.
#[utoipa::path(
    operation_id = "compte_lire",
    tag = "comptes",
    params(("compte_id" = Uuid, Path, description = "Identifiant du compte")),
    responses(
        (status = 200, description = "Le compte", body = CompteVue),
        (status = 401, description = "Non authentifié"),
        (status = 403, description = "Permission absente", body = CorpsErreur),
        (status = 404, description = "Compte inconnu", body = CorpsErreur),
    ),
    security(("bearer" = []))
)]
#[get("")]
pub async fn lire(
    etat: web::Data<EtatApplication>,
    contexte: ContexteAppel,
    chemin: web::Path<Uuid>,
) -> Result<HttpResponse, actix_web::Error> {
    securite::exiger(&contexte, PERM_LIRE)?;

    let compte = etat
        .service_comptes()
        .lire(contexte.tenant_id, chemin.into_inner())
        .await
        .map_err(en_reponse_compte)?;

    Ok(HttpResponse::Ok().json(compte))
}

// =================================================================================================
//  13 · Changer l'état d'un compte
// =================================================================================================

/// Active ou désactive un compte.
///
/// **La désactivation EST la suppression** au sens de la taxonomie d'audit : rien ne se supprime
/// jamais dans Kaya (FR-014). Elle émet `compte.desactive` et une entrée `suppression` ; la
/// réactivation émet `compte.reactive` et **aucune entrée** — aucune des dix familles ne couvre le
/// rétablissement d'un droit.
#[utoipa::path(
    operation_id = "compte_changer_etat",
    tag = "comptes",
    params(("compte_id" = Uuid, Path, description = "Identifiant du compte")),
    request_body = ChangerEtatRequete,
    responses(
        (status = 200, description = "État changé", body = CompteVue),
        (status = 401, description = "Non authentifié"),
        (status = 403, description = "Permission absente", body = CorpsErreur),
        (status = 404, description = "Compte inconnu", body = CorpsErreur),
    ),
    security(("bearer" = []))
)]
#[put("")]
pub async fn changer_etat(
    etat: web::Data<EtatApplication>,
    contexte: ContexteAppel,
    chemin: web::Path<Uuid>,
    corps: web::Json<ChangerEtatRequete>,
) -> Result<HttpResponse, actix_web::Error> {
    securite::exiger(&contexte, PERM_GERER)?;

    let compte = etat
        .service_comptes()
        .changer_etat(
            contexte.tenant_id,
            contexte.compte_id,
            chemin.into_inner(),
            corps.actif,
        )
        .await
        .map_err(en_reponse_compte)?;

    Ok(HttpResponse::Ok().json(compte))
}

// =================================================================================================
//  14 · Changer le mot de passe
// =================================================================================================

/// Change le mot de passe — **et coupe les autres sessions, immédiatement**.
///
/// # C'est le handler qui décide lequel des deux régimes s'applique
///
/// Un compte agissant **sur lui-même** doit fournir son mot de passe actuel ; un habilité ne le
/// fournit pas. Le service reçoit la décision déjà prise, parce que lui seul ici sait qui appelle.
///
/// **Un compte qui agit sur lui-même sans fournir son mot de passe actuel est refusé**, même
/// habilité : sans cette règle, un jeton volé permettrait de changer le mot de passe de sa
/// victime, donc de la verrouiller dehors de son propre produit.
#[utoipa::path(
    operation_id = "compte_changer_mot_de_passe",
    tag = "comptes",
    params(("compte_id" = Uuid, Path, description = "Identifiant du compte")),
    request_body = ChangerMotDePasseRequete,
    responses(
        (status = 204, description = "Mot de passe changé — les autres sessions sont coupées"),
        (status = 401, description = "Non authentifié"),
        (status = 403, description = "Permission absente", body = CorpsErreur),
        (status = 404, description = "Compte inconnu", body = CorpsErreur),
        (status = 422, description = "Mot de passe refusé, ou mot de passe actuel invalide", body = CorpsErreur),
    ),
    security(("bearer" = []))
)]
#[put("")]
pub async fn changer_mot_de_passe(
    etat: web::Data<EtatApplication>,
    contexte: ContexteAppel,
    chemin: web::Path<Uuid>,
    corps: web::Json<ChangerMotDePasseRequete>,
) -> Result<HttpResponse, actix_web::Error> {
    let cible = chemin.into_inner();
    securite::exiger_ou_soi(&contexte, PERM_GERER, cible)?;

    let soi = contexte.compte_id == cible;
    let corps = corps.into_inner();

    if soi && corps.mot_de_passe_actuel.is_none() {
        return Err(CorpsErreur::nouveau(
            "mot_de_passe_actuel_requis",
            None,
            "un compte qui change son propre mot de passe fournit l'actuel".to_owned(),
        )
        .en_422());
    }

    etat.service_comptes()
        .changer_mot_de_passe(
            contexte.tenant_id,
            cible,
            // L'habilité n'en fournit pas, et s'il en fournissait un il serait ignoré : la
            // vérification n'a de sens que sur son propre compte.
            if soi { corps.mot_de_passe_actuel.as_deref() } else { None },
            &corps.nouveau_mot_de_passe,
            // « Les autres » : celui qui change son mot de passe ne se déconnecte pas lui-même.
            // Quand un habilité agit sur quelqu'un d'autre, `None` fait tomber **tout**, ce qui
            // est le comportement voulu.
            if soi { Some(contexte.session_id) } else { None },
            kaya_comptes::session::DureesSession::repli().acces_s,
        )
        .await
        .map_err(en_reponse_compte)?;

    Ok(HttpResponse::NoContent().finish())
}

// =================================================================================================
//  15 · Attribuer un rôle
// =================================================================================================

/// Attribue un rôle à un compte.
#[utoipa::path(
    operation_id = "compte_attribuer_role",
    tag = "comptes",
    params(("compte_id" = Uuid, Path, description = "Identifiant du compte")),
    request_body = AttribuerRoleRequete,
    responses(
        (status = 201, description = "Rôle attribué"),
        (status = 200, description = "Rôle déjà porté (rejeu idempotent)"),
        (status = 401, description = "Non authentifié"),
        (status = 403, description = "Permission absente", body = CorpsErreur),
        (status = 404, description = "Compte ou établissement inconnu", body = CorpsErreur),
        (status = 422, description = "Portée incompatible ou rôle inconnu", body = CorpsErreur),
    ),
    security(("bearer" = []))
)]
#[post("")]
pub async fn attribuer_role(
    etat: web::Data<EtatApplication>,
    contexte: ContexteAppel,
    chemin: web::Path<Uuid>,
    corps: web::Json<AttribuerRoleRequete>,
) -> Result<HttpResponse, actix_web::Error> {
    securite::exiger(&contexte, PERM_ATTRIBUER)?;
    let corps = corps.into_inner();

    let issue = etat
        .service_roles(contexte.tenant_id)
        .attribuer(
            contexte.tenant_id,
            contexte.compte_id,
            AttribuerRole {
                id: corps.id,
                compte_id: chemin.into_inner(),
                role_code: corps.role_code,
                etablissement_id: corps.etablissement_id,
                horodatage_client: corps.horodatage_client,
            },
        )
        .await
        .map_err(en_reponse_roles)?;

    Ok(match issue {
        Issue::Creee => HttpResponse::Created().finish(),
        Issue::DejaPresente => HttpResponse::Ok().finish(),
    })
}

// =================================================================================================
//  16 · Retirer un rôle
// =================================================================================================

/// Retire un rôle — **et refuse de retirer la dernière habilitation** (FR-023).
#[utoipa::path(
    operation_id = "compte_retirer_role",
    tag = "comptes",
    params(
        ("compte_id" = Uuid, Path, description = "Identifiant du compte"),
        ("role_code" = String, Path, description = "Code du rôle à retirer"),
        CibleRetrait,
    ),
    responses(
        (status = 204, description = "Rôle retiré"),
        (status = 401, description = "Non authentifié"),
        (status = 403, description = "Permission absente", body = CorpsErreur),
        (status = 409, description = "Dernière habilitation de l'établissement", body = CorpsErreur),
        (status = 422, description = "Portée incompatible ou rôle inconnu", body = CorpsErreur),
    ),
    security(("bearer" = []))
)]
#[delete("")]
pub async fn retirer_role(
    etat: web::Data<EtatApplication>,
    contexte: ContexteAppel,
    chemin: web::Path<(Uuid, String)>,
    cible: web::Query<CibleRetrait>,
) -> Result<HttpResponse, actix_web::Error> {
    securite::exiger(&contexte, PERM_ATTRIBUER)?;
    let (compte_id, role_code) = chemin.into_inner();

    etat.service_roles(contexte.tenant_id)
        .retirer(
            contexte.tenant_id,
            contexte.compte_id,
            compte_id,
            &role_code,
            cible.etablissement_id,
        )
        .await
        .map_err(en_reponse_roles)?;

    Ok(HttpResponse::NoContent().finish())
}

// =================================================================================================
//  Traduction des erreurs
// =================================================================================================

/// Traduit un échec du service des comptes en réponse HTTP.
///
/// **Aucun détail interne ne franchit la frontière** : ni message PostgreSQL, ni nom de table. Le
/// détail part dans les journaux, corrélé par identifiant de requête.
fn en_reponse_compte(erreur: ErreurCompte) -> actix_web::Error {
    match erreur {
        ErreurCompte::Inconnu => {
            CorpsErreur::nouveau("compte_inconnu", None, erreur.to_string()).en_404()
        }
        ErreurCompte::PersonneInconnue => {
            CorpsErreur::nouveau("personne_inconnue", None, erreur.to_string()).en_404()
        }
        ErreurCompte::IdentifiantAbsent => {
            CorpsErreur::nouveau("identifiant_absent", None, erreur.to_string()).en_422()
        }
        // **Le message ne dit pas que l'identifiant existe déjà.** Le dire apprendrait, à un
        // habilité d'un tenant, quels numéros sont clients de Kaya.
        ErreurCompte::IdentifiantRefuse => {
            CorpsErreur::nouveau("identifiant_refuse", None, erreur.to_string()).en_422()
        }
        // Le motif est **explicite** — trop court, trop long, compromis : l'utilisateur doit
        // savoir quoi corriger. C'est l'inverse exact du refus d'authentification, où le silence
        // protège.
        ErreurCompte::MotDePasseRefuse(ref refus) => CorpsErreur::nouveau(
            "mot_de_passe_refuse",
            Some(refus.code().to_owned()),
            erreur.to_string(),
        )
        .en_422(),
        ErreurCompte::MotDePasseActuelInvalide => {
            CorpsErreur::nouveau("mot_de_passe_actuel_invalide", None, erreur.to_string()).en_422()
        }
        autre => interne("écriture d'un compte", autre),
    }
}

/// Traduit un échec du service des rôles en réponse HTTP.
fn en_reponse_roles(erreur: ErreurRoles) -> actix_web::Error {
    match erreur {
        ErreurRoles::RoleInconnu(ref code) => {
            CorpsErreur::nouveau("role_inconnu", Some(code.clone()), erreur.to_string()).en_422()
        }
        ErreurRoles::CompteInconnu => {
            CorpsErreur::nouveau("compte_inconnu", None, erreur.to_string()).en_404()
        }
        // Vérifié **par trait**, jamais par clé étrangère : c'est ce qui donne un `404`
        // intelligible plutôt qu'une violation de contrainte remontée en `500`.
        ErreurRoles::EtablissementInconnu => {
            CorpsErreur::nouveau("etablissement_inconnu", None, erreur.to_string()).en_404()
        }
        ErreurRoles::PorteeIncompatible => {
            CorpsErreur::nouveau("portee_incompatible", None, erreur.to_string()).en_422()
        }
        // **Le seul refus métier du cycle**, et il est irréversible sans l'éditeur : d'où un code
        // propre plutôt qu'un `403`, qui aurait suggéré un problème de droits de l'appelant.
        ErreurRoles::DerniereHabilitation => {
            CorpsErreur::nouveau("derniere_habilitation", None, erreur.to_string()).en_409()
        }
        autre => interne("écriture d'un rôle", autre),
    }
}
