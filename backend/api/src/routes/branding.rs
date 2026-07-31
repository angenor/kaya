//! Handlers de l'identité visuelle — **ETB-05**, opérations 18 à 21.
//!
//! # Le téléversement passe par l'interface S3, jamais par la base
//!
//! Le binaire du logo part au stockage objet ; la base ne porte qu'une **clé d'objet**
//! (principe II). Un logo en base gonflerait chaque sauvegarde et chaque réplication pour un
//! fichier qui ne change jamais.

use actix_web::{HttpResponse, get, post, put, web};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use kaya_etablissements::branding::{
    BrandingNiveau, BrandingResolu, EcrireBranding, ErreurBranding, LOGO_TAILLE_MAX,
    rendre_document_test,
};

use crate::application::EtatApplication;
use crate::contexte::ContexteAppel;
use crate::routes::erreurs::{CorpsErreur, interne};

/// Niveau visé — sans `etablissement_id`, c'est celui du tenant.
#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct NiveauParams {
    pub etablissement_id: Option<Uuid>,
}

/// Corps d'écriture de l'identité visuelle.
#[derive(Debug, Deserialize, ToSchema)]
pub struct EcrireBrandingRequete {
    pub id: Uuid,
    /// Absent = niveau tenant.
    pub etablissement_id: Option<Uuid>,
    pub logo_objet_cle: Option<String>,
    /// Hexadécimal `#RRGGBB`. **S'applique aux documents produits, jamais à l'interface**
    /// (FR-059).
    pub couleur_primaire: Option<String>,
    pub entete_document: Option<String>,
    pub pied_document: Option<String>,
    pub mentions_legales: Option<String>,
    pub coordonnees: Option<String>,
}

/// Corps d'aperçu — **l'identité telle qu'elle est à l'écran, y compris non enregistrée**
/// (FR-057).
#[derive(Debug, Deserialize, ToSchema)]
pub struct ApercuRequete {
    pub nom_etablissement: String,
    pub logo_objet_cle: Option<String>,
    pub couleur_primaire: Option<String>,
    pub entete_document: Option<String>,
    pub pied_document: Option<String>,
    pub mentions_legales: Option<String>,
    pub coordonnees: Option<String>,
}

/// Le document de test rendu.
#[derive(Debug, Serialize, ToSchema)]
pub struct ApercuReponse {
    /// Contenu textuel du document.
    pub document: String,
    /// **Toujours présente.** Reprise ici pour que le client puisse la mettre en évidence sans
    /// analyser le corps du document.
    pub mention_non_fiscale: String,
}

/// La clé d'objet d'un logo téléversé.
#[derive(Debug, Serialize, ToSchema)]
pub struct LogoReponse {
    /// **Une clé d'objet, jamais une URL de stockage** : l'accès passe par une URL signée de
    /// courte durée, produite à la demande.
    pub logo_objet_cle: String,
}

/// Lit l'identité visuelle **résolue**, champ par champ, avec l'origine de chacun.
#[utoipa::path(
    tag = "branding",
    params(NiveauParams),
    responses(
        (status = 200, description = "Identité visuelle résolue", body = BrandingResolu),
        (status = 401, description = "Non authentifié"),
        (status = 403, description = "Permission absente"),
    ),
    security(("bearer" = []))
)]
#[get("")]
pub async fn resoudre(
    etat: web::Data<EtatApplication>,
    contexte: ContexteAppel,
    params: web::Query<NiveauParams>,
) -> Result<HttpResponse, actix_web::Error> {
    let resolu = etat
        .service_branding()
        .resoudre(contexte.tenant_id, params.into_inner().etablissement_id)
        .await
        .map_err(en_reponse)?;

    Ok(HttpResponse::Ok().json(resolu))
}

/// Écrit l'identité visuelle d'un niveau.
///
/// **Le corps décrit ce qui est posé À CE NIVEAU**, pas le résultat de la fusion : un champ absent
/// reste hérité. Enregistrer la vue fusionnée figerait chez soi tout ce dont on héritait, et la
/// première modification au niveau tenant ne redescendrait plus.
#[utoipa::path(
    tag = "branding",
    request_body = EcrireBrandingRequete,
    responses(
        (status = 200, description = "Identité visuelle enregistrée", body = BrandingNiveau),
        (status = 400, description = "Couleur invalide", body = CorpsErreur),
        (status = 401, description = "Non authentifié"),
        (status = 403, description = "Permission absente"),
        (status = 404, description = "Établissement inconnu", body = CorpsErreur),
    ),
    security(("bearer" = []))
)]
#[put("")]
pub async fn ecrire(
    etat: web::Data<EtatApplication>,
    contexte: ContexteAppel,
    corps: web::Json<EcrireBrandingRequete>,
) -> Result<HttpResponse, actix_web::Error> {
    let corps = corps.into_inner();
    let niveau = etat
        .service_branding()
        .ecrire(
            contexte.tenant_id,
            EcrireBranding {
                id: corps.id,
                etablissement_id: corps.etablissement_id,
                contenu: BrandingNiveau {
                    logo_objet_cle: corps.logo_objet_cle,
                    couleur_primaire: corps.couleur_primaire,
                    entete_document: corps.entete_document,
                    pied_document: corps.pied_document,
                    mentions_legales: corps.mentions_legales,
                    coordonnees: corps.coordonnees,
                },
            },
        )
        .await
        .map_err(en_reponse)?;

    Ok(HttpResponse::Ok().json(niveau))
}

/// Téléverse un logo.
///
/// **`413` si la taille dépasse le plafond, et le message DONNE la limite** — jamais un refus
/// muet. Le plafond est une constante technique nommée dans le code, avec sa justification : un
/// exploitant n'a aucune raison de le régler, et l'inscrire au catalogue de paramètres ferait
/// entrer au récapitulatif du principe I·c une valeur qui ne relève pas de l'exploitation.
#[utoipa::path(
    tag = "branding",
    params(NiveauParams),
    request_body(content = Vec<u8>, description = "Binaire du logo", content_type = "application/octet-stream"),
    responses(
        (status = 201, description = "Logo téléversé — rend sa clé d'objet", body = LogoReponse),
        (status = 400, description = "Requête invalide", body = CorpsErreur),
        (status = 401, description = "Non authentifié"),
        (status = 403, description = "Permission absente"),
        (status = 413, description = "Logo trop volumineux — le message donne la limite", body = CorpsErreur),
    ),
    security(("bearer" = []))
)]
#[post("")]
pub async fn televerser_logo(
    etat: web::Data<EtatApplication>,
    contexte: ContexteAppel,
    params: web::Query<NiveauParams>,
    corps: web::Bytes,
) -> Result<HttpResponse, actix_web::Error> {
    if corps.len() > LOGO_TAILLE_MAX {
        return Err(CorpsErreur::nouveau(
            "logo_trop_volumineux",
            Some(corps.len().to_string()),
            format!(
                "logo de {} octets : la limite est de {LOGO_TAILLE_MAX} octets ({} kio)",
                corps.len(),
                LOGO_TAILLE_MAX / 1024
            ),
        )
        .en_413());
    }

    let etablissement_id = params.into_inner().etablissement_id;

    // Une clé d'accès **par usage** : le chemin porte le tenant, ce qui rend impossible de lire
    // l'objet d'un autre client même en devinant la clé.
    let objet_cle = format!(
        "branding/{}/{}/logo",
        contexte.tenant_id,
        etablissement_id
            .map(|id| id.to_string())
            .unwrap_or_else(|| "tenant".to_owned())
    );

    etat.stockage()
        .televerser(&objet_cle, corps.to_vec())
        .await
        .map_err(|e| {
            interne(
                "téléversement du logo",
                format!("stockage objet indisponible : {e}"),
            )
        })?;

    etat.service_branding()
        .poser_logo(contexte.tenant_id, etablissement_id, &objet_cle)
        .await
        .map_err(en_reponse)?;

    Ok(HttpResponse::Created().json(LogoReponse {
        logo_objet_cle: objet_cle,
    }))
}

/// Rend le document de test — **sans rien enregistrer** (FR-057).
///
/// Le corps porte l'identité **telle qu'elle est à l'écran**, y compris non enregistrée : c'est ce
/// qui permet à l'exploitant de voir avant de valider, plutôt que d'enregistrer pour voir.
///
/// **Le document porte obligatoirement la mention « Document non fiscal — ne tient pas lieu de
/// facture »** (principe V, FR-058). Un aperçu ressemble à une facture : mêmes en-tête, logo,
/// coordonnées et mentions légales. Sans cette phrase, le premier aperçu imprimé serait présenté à
/// un client comme un justificatif.
#[utoipa::path(
    tag = "branding",
    request_body = ApercuRequete,
    responses(
        (status = 200, description = "Document de test — porte la mention non fiscale", body = ApercuReponse),
        (status = 400, description = "Requête invalide", body = CorpsErreur),
        (status = 401, description = "Non authentifié"),
        (status = 403, description = "Permission absente"),
    ),
    security(("bearer" = []))
)]
#[post("")]
pub async fn apercu(
    _etat: web::Data<EtatApplication>,
    _contexte: ContexteAppel,
    corps: web::Json<ApercuRequete>,
) -> Result<HttpResponse, actix_web::Error> {
    use kaya_etablissements::branding::{ChampResolu, MENTION_NON_FISCALE};

    let corps = corps.into_inner();
    let champ = |v: Option<String>| -> Option<ChampResolu> {
        v.filter(|s| !s.is_empty()).map(|valeur| ChampResolu {
            valeur,
            // L'aperçu porte l'état de l'écran : l'origine n'y a pas de sens, rien n'ayant encore
            // été enregistré. `ECRAN` le dit, plutôt qu'un `TENANT` qui serait faux.
            origine: "ECRAN".to_owned(),
        })
    };

    let identite = BrandingResolu {
        logo_objet_cle: champ(corps.logo_objet_cle),
        couleur_primaire: champ(corps.couleur_primaire),
        entete_document: champ(corps.entete_document),
        pied_document: champ(corps.pied_document),
        mentions_legales: champ(corps.mentions_legales),
        coordonnees: champ(corps.coordonnees),
    };

    let document = rendre_document_test(&identite, &corps.nom_etablissement);

    Ok(HttpResponse::Ok().json(ApercuReponse {
        document,
        mention_non_fiscale: MENTION_NON_FISCALE.to_owned(),
    }))
}

fn en_reponse(erreur: ErreurBranding) -> actix_web::Error {
    let corps = CorpsErreur::nouveau(erreur.code(), erreur.valeur(), erreur.to_string());

    match erreur {
        ErreurBranding::CouleurInvalide(_) => corps.en_400(),
        ErreurBranding::LogoTropVolumineux { .. } => corps.en_413(),
        ErreurBranding::EtablissementInconnu => corps.en_404(),
        autre => interne("service d'identité visuelle", autre),
    }
}
