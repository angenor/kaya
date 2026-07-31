//! Handlers des établissements — **ETB-01**, opérations 1 à 4 du contrat.
//!
//! `specs/002-etablissements-modules-activite/contracts/http-api.md` §1 décrit ce que ce fichier
//! doit produire. **La source de vérité reste ce code** : le contrat OpenAPI est généré depuis les
//! annotations `#[utoipa::path]` ci-dessous, et le client TypeScript depuis ce contrat
//! (principe I·a, porte P-01).
//!
//! Le chemin et le verbe **ne sont jamais répétés** dans l'annotation utoipa : ils viennent de
//! l'attribut de routage d'Actix. Les écrire deux fois laisserait le contrat annoncer une adresse
//! que le serveur ne sert pas.

use actix_web::{HttpResponse, get, patch, post, web};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use kaya_etablissements::Issue;
use kaya_etablissements::etablissement::{
    CreerEtablissement, ErreurEtablissement, EtablissementVue, ModifierEtablissement,
    classement_depuis_requete,
};

use crate::application::EtatApplication;
use crate::contexte::ContexteAppel;
use crate::routes::erreurs::{CorpsErreur, interne};

/// Corps de création d'un établissement.
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreerEtablissementRequete {
    /// UUID v7 **généré par le client**. C'est lui qui rend le rejeu inoffensif : un double-clic
    /// sur « Créer » ne produit pas deux établissements.
    pub id: Uuid,
    pub nom: String,
    /// Sélectionne le `JurisdictionAdapter`. `CI` au MVP — **n'encode aucune règle fiscale**.
    pub juridiction: String,
    /// `ETOILES` | `NON_CLASSE` | `RESIDENCE_MEUBLEE`.
    pub classement: String,
    /// Obligatoire **si et seulement si** `classement = "ETOILES"`.
    pub etoiles: Option<u8>,
    /// Commune de rattachement — assiette du reversement communal.
    pub commune: String,
    pub fuseau_horaire: String,
    /// ISO 4217.
    pub devise: String,
    pub adresse: Option<String>,
    /// Numéro de compte contribuable.
    pub ncc: Option<String>,
}

/// Corps de modification — **tout champ absent est laissé tel quel**.
#[derive(Debug, Deserialize, ToSchema)]
pub struct ModifierEtablissementRequete {
    pub nom: Option<String>,
    pub classement: Option<String>,
    pub etoiles: Option<u8>,
    pub commune: Option<String>,
    pub fuseau_horaire: Option<String>,
    pub devise: Option<String>,
    pub adresse: Option<String>,
    pub ncc: Option<String>,
}

/// Réponse de modification — la vue à jour et, le cas échéant, un avertissement à présenter.
#[derive(Debug, Serialize, ToSchema)]
pub struct ModificationReponse {
    #[serde(flatten)]
    pub etablissement: EtablissementVue,
    /// `fuseau_change` quand le fuseau horaire a été modifié.
    ///
    /// **Un code, pas une phrase** : l'interface doit le présenter avant de confirmer, dans la
    /// langue de l'utilisateur. Changer de fuseau réinterprète tout regroupement par journée
    /// locale — une clôture déjà produite ne couvre plus la même période.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avertissement: Option<String>,
}

/// Liste les établissements du tenant.
#[utoipa::path(
    tag = "etablissements",
    responses(
        (status = 200, description = "Établissements du tenant", body = Vec<EtablissementVue>),
        (status = 401, description = "Non authentifié"),
    ),
    security(("bearer" = []))
)]
#[get("")]
pub async fn lister(
    etat: web::Data<EtatApplication>,
    contexte: ContexteAppel,
) -> Result<HttpResponse, actix_web::Error> {
    let liste = etat
        .service_etablissement()
        .lister(contexte.tenant_id)
        .await
        .map_err(en_reponse)?;

    Ok(HttpResponse::Ok().json(liste))
}

/// Crée un établissement.
///
/// **`200` sur rejeu, pas `409`.** Le corps rendu est la ligne telle qu'elle est en base — le
/// serveur fait foi en conflit (principe VI).
#[utoipa::path(
    tag = "etablissements",
    request_body = CreerEtablissementRequete,
    responses(
        (status = 201, description = "Établissement créé", body = EtablissementVue),
        (status = 200, description = "Déjà créé (rejeu idempotent)", body = EtablissementVue),
        (status = 400, description = "Requête invalide", body = CorpsErreur),
        (status = 401, description = "Non authentifié"),
        (status = 403, description = "Permission absente"),
        (status = 422, description = "Règle métier — classement incohérent", body = CorpsErreur),
    ),
    security(("bearer" = []))
)]
#[post("")]
pub async fn creer(
    etat: web::Data<EtatApplication>,
    contexte: ContexteAppel,
    corps: web::Json<CreerEtablissementRequete>,
) -> Result<HttpResponse, actix_web::Error> {
    let corps = corps.into_inner();
    let classement = classement_depuis_requete(&corps.classement, corps.etoiles)
        .map_err(en_reponse)?;

    let (vue, issue) = etat
        .service_etablissement()
        .creer(
            contexte.tenant_id,
            CreerEtablissement {
                id: corps.id,
                nom: corps.nom,
                juridiction: corps.juridiction,
                classement,
                commune: corps.commune,
                fuseau_horaire: corps.fuseau_horaire,
                devise: corps.devise,
                adresse: corps.adresse,
                ncc: corps.ncc,
            },
        )
        .await
        .map_err(en_reponse)?;

    Ok(match issue {
        Issue::Creee => HttpResponse::Created().json(vue),
        Issue::DejaPresente => HttpResponse::Ok().json(vue),
    })
}

/// Lit un établissement.
#[utoipa::path(
    tag = "etablissements",
    params(("etablissement_id" = Uuid, Path, description = "Identifiant de l'établissement")),
    responses(
        (status = 200, description = "Établissement", body = EtablissementVue),
        (status = 401, description = "Non authentifié"),
        (status = 403, description = "Permission absente"),
        (status = 404, description = "Établissement inconnu", body = CorpsErreur),
    ),
    security(("bearer" = []))
)]
#[get("/{etablissement_id}")]
pub async fn lire(
    etat: web::Data<EtatApplication>,
    contexte: ContexteAppel,
    chemin: web::Path<Uuid>,
) -> Result<HttpResponse, actix_web::Error> {
    let vue = etat
        .service_etablissement()
        .lire(contexte.tenant_id, chemin.into_inner())
        .await
        .map_err(en_reponse)?;

    Ok(HttpResponse::Ok().json(vue))
}

/// Modifie un établissement.
///
/// **Deux refus `422` nommés** : `classement_incoherent` — un nombre d'étoiles sans classement
/// étoilé, ou l'inverse ; `devise_figee` — la devise ne se modifie plus après la première
/// opération financière. Le second est **posé à vide à ce cycle** : la fonction qui compte les
/// opérations rend zéro tant qu'aucune n'existe, et le cycle CAI la branche.
#[utoipa::path(
    tag = "etablissements",
    params(("etablissement_id" = Uuid, Path, description = "Identifiant de l'établissement")),
    request_body = ModifierEtablissementRequete,
    responses(
        (status = 200, description = "Établissement modifié", body = ModificationReponse),
        (status = 400, description = "Requête invalide", body = CorpsErreur),
        (status = 401, description = "Non authentifié"),
        (status = 403, description = "Permission absente"),
        (status = 404, description = "Établissement inconnu", body = CorpsErreur),
        (status = 422, description = "Règle métier — classement incohérent, devise figée", body = CorpsErreur),
    ),
    security(("bearer" = []))
)]
#[patch("/{etablissement_id}")]
pub async fn modifier(
    etat: web::Data<EtatApplication>,
    contexte: ContexteAppel,
    chemin: web::Path<Uuid>,
    corps: web::Json<ModifierEtablissementRequete>,
) -> Result<HttpResponse, actix_web::Error> {
    let id = chemin.into_inner();
    let corps = corps.into_inner();

    // Le classement ne se modifie que **d'un bloc** : le code et le nombre d'étoiles sont une
    // seule décision. Accepter `etoiles` seul obligerait à deviner le code, et la première
    // supposition erronée passerait le `CHECK` de la base en produisant un barème de nuitée faux.
    let classement = match (&corps.classement, corps.etoiles) {
        (Some(code), etoiles) => Some(classement_depuis_requete(code, etoiles).map_err(en_reponse)?),
        (None, None) => None,
        (None, Some(_)) => {
            return Err(CorpsErreur::nouveau(
                "classement_incoherent",
                Some("etoiles sans classement".to_owned()),
                "le nombre d'étoiles ne se modifie pas seul : classement et étoiles sont une seule \
                 décision"
                    .to_owned(),
            )
            .en_422());
        }
    };

    let resultat = etat
        .service_etablissement()
        .modifier(
            contexte.tenant_id,
            id,
            ModifierEtablissement {
                nom: corps.nom,
                classement,
                commune: corps.commune,
                fuseau_horaire: corps.fuseau_horaire,
                devise: corps.devise,
                adresse: corps.adresse,
                ncc: corps.ncc,
            },
        )
        .await
        .map_err(en_reponse)?;

    Ok(HttpResponse::Ok().json(ModificationReponse {
        etablissement: resultat.etablissement,
        avertissement: resultat.avertissement.map(str::to_owned),
    }))
}

/// Traduit une erreur de domaine en réponse HTTP structurée.
pub fn en_reponse(erreur: ErreurEtablissement) -> actix_web::Error {
    let corps = CorpsErreur::nouveau(erreur.code(), erreur.valeur(), erreur.to_string());

    match erreur {
        ErreurEtablissement::NomInvalide
        | ErreurEtablissement::CommuneInvalide
        | ErreurEtablissement::FuseauInconnu(_)
        | ErreurEtablissement::DeviseInvalide(_)
        | ErreurEtablissement::NccInvalide => corps.en_400(),

        ErreurEtablissement::ClassementIncoherent(_) | ErreurEtablissement::DeviseFigee => {
            corps.en_422()
        }

        ErreurEtablissement::Inconnu => corps.en_404(),

        autre => interne("service des établissements", autre),
    }
}
