//! Handlers de l'identité civile — **CPT-00**, opérations 7 à 9 du contrat.
//!
//! # Ce que ces trois opérations ne font PAS, et c'est le sujet
//!
//! **Aucune liste de personnes.** Le contrat en compte trois : créer, lire une, modifier une.
//! Exposer `GET /personnes` donnerait au produit un annuaire d'identités civiles avant qu'il ait
//! la politique de rétention qui va avec. La recherche de fiches client est **SEJ-01**, et elle
//! arrivera avec la rétention de 90 jours de TRX-06.
//!
//! **`type_piece` et `numero_piece` ne sont ni acceptés ni rendus.** Les colonnes existent
//! (migration `0015`), le type Rust ne les porte pas, et ces handlers ne les nomment nulle part.
//! Trois barrières pour la même décision, parce qu'une seule se défait par distraction.

use actix_web::{HttpResponse, get, post, put, web};
use serde::Deserialize;
use time::OffsetDateTime;
use utoipa::ToSchema;
use uuid::Uuid;

use kaya_comptes::personne::{CreerPersonne, ErreurPersonne, ModifierPersonne, Personne};
use kaya_etablissements::Issue;

use crate::application::EtatApplication;
use crate::contexte::ContexteAppel;
use crate::routes::erreurs::{CorpsErreur, interne};
use crate::securite;

/// Permissions du contrat, nommées une fois.
///
/// **Elles n'étaient pas posées à l'écriture de ces trois handlers (T023)** : la garde de
/// permission n'existait pas encore, elle est arrivée avec T040. Les poser ici est la seconde
/// moitié de la même tâche — un endpoint annoncé sous permission au contrat et servi sans elle
/// est un contrat qui ment.
const PERM_LIRE: &str = "cpt.compte.lire";
const PERM_GERER: &str = "cpt.compte.gerer";

/// Corps de création d'une personne.
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreerPersonneRequete {
    /// UUID v7 **généré par le client** (principe VI) : c'est lui qui rend le rejeu inoffensif —
    /// `201`, puis `200`, `200`.
    pub id: Uuid,
    pub nom: String,
    #[serde(default)]
    pub prenoms: Option<String>,
    /// E.164.
    #[serde(default)]
    pub telephone: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    /// Indicatif — **jamais employé par une règle métier**.
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub horodatage_client: Option<OffsetDateTime>,
}

/// Corps de modification — **remplacement complet**.
#[derive(Debug, Deserialize, ToSchema)]
pub struct ModifierPersonneRequete {
    pub nom: String,
    #[serde(default)]
    pub prenoms: Option<String>,
    #[serde(default)]
    pub telephone: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub horodatage_client: Option<OffsetDateTime>,
}

/// Crée une personne.
///
/// **`200` sur rejeu, pas `409`** (module doré, couche 5) : un terminal qui vide sa file après une
/// coupure ne doit pas voir d'erreur pour une écriture déjà acceptée. Le corps rendu est la ligne
/// **telle qu'elle est en base** — le serveur fait foi en conflit.
// Le verbe et le chemin viennent de l'attribut Actix, jamais répétés ici.
#[utoipa::path(
    operation_id = "personne_creer",
    tag = "comptes",
    request_body = CreerPersonneRequete,
    responses(
        (status = 201, description = "Personne créée", body = Personne),
        (status = 200, description = "Personne déjà créée (rejeu idempotent)", body = Personne),
        (status = 400, description = "Requête invalide", body = CorpsErreur),
        (status = 401, description = "Non authentifié"),
        (status = 403, description = "Permission absente", body = CorpsErreur),
    ),
    security(("bearer" = []))
)]
#[post("")]
pub async fn creer(
    etat: web::Data<EtatApplication>,
    contexte: ContexteAppel,
    corps: web::Json<CreerPersonneRequete>,
) -> Result<HttpResponse, actix_web::Error> {
    securite::exiger(&contexte, PERM_GERER)?;
    let corps = corps.into_inner();

    let (personne, issue) = etat
        .service_personne()
        .creer(
            contexte.tenant_id,
            CreerPersonne {
                id: corps.id,
                nom: corps.nom,
                prenoms: corps.prenoms,
                telephone: corps.telephone,
                email: corps.email,
                horodatage_client: corps.horodatage_client,
            },
        )
        .await
        .map_err(en_reponse)?;

    Ok(match issue {
        Issue::Creee => HttpResponse::Created().json(personne),
        Issue::DejaPresente => HttpResponse::Ok().json(personne),
    })
}

/// Lit une personne.
#[utoipa::path(
    operation_id = "personne_lire",
    tag = "comptes",
    params(("personne_id" = Uuid, Path, description = "Identifiant de la personne")),
    responses(
        (status = 200, description = "La personne", body = Personne),
        (status = 401, description = "Non authentifié"),
        (status = 403, description = "Permission absente", body = CorpsErreur),
        (status = 404, description = "Personne inconnue", body = CorpsErreur),
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

    let personne = etat
        .service_personne()
        .lire(contexte.tenant_id, chemin.into_inner())
        .await
        .map_err(en_reponse)?;

    Ok(HttpResponse::Ok().json(personne))
}

/// Modifie une personne — **remplacement complet des champs modifiables**.
///
/// Un `PUT` qui fusionnerait champ par champ rendrait impossible d'effacer un numéro de
/// téléphone : l'absence du champ et sa mise à `null` seraient indistinguables.
#[utoipa::path(
    operation_id = "personne_modifier",
    tag = "comptes",
    params(("personne_id" = Uuid, Path, description = "Identifiant de la personne")),
    request_body = ModifierPersonneRequete,
    responses(
        (status = 200, description = "Personne modifiée", body = Personne),
        (status = 400, description = "Requête invalide", body = CorpsErreur),
        (status = 401, description = "Non authentifié"),
        (status = 403, description = "Permission absente", body = CorpsErreur),
        (status = 404, description = "Personne inconnue", body = CorpsErreur),
    ),
    security(("bearer" = []))
)]
#[put("")]
pub async fn modifier(
    etat: web::Data<EtatApplication>,
    contexte: ContexteAppel,
    chemin: web::Path<Uuid>,
    corps: web::Json<ModifierPersonneRequete>,
) -> Result<HttpResponse, actix_web::Error> {
    securite::exiger(&contexte, PERM_GERER)?;
    let corps = corps.into_inner();

    let personne = etat
        .service_personne()
        .modifier(
            contexte.tenant_id,
            chemin.into_inner(),
            ModifierPersonne {
                nom: corps.nom,
                prenoms: corps.prenoms,
                telephone: corps.telephone,
                email: corps.email,
                horodatage_client: corps.horodatage_client,
            },
        )
        .await
        .map_err(en_reponse)?;

    Ok(HttpResponse::Ok().json(personne))
}

/// Traduit une erreur de domaine en réponse HTTP.
///
/// **Aucun détail interne ne franchit la frontière** : ni message PostgreSQL, ni nom de table.
fn en_reponse(erreur: ErreurPersonne) -> actix_web::Error {
    match erreur {
        ErreurPersonne::NomInvalide => CorpsErreur::nouveau(
            "nom_invalide",
            None,
            "le nom doit compter entre 1 et 200 caractères après nettoyage".to_owned(),
        )
        .en_400(),
        ErreurPersonne::Inconnue => CorpsErreur::nouveau(
            "personne_inconnue",
            None,
            "aucune personne de cet identifiant dans ce tenant".to_owned(),
        )
        .en_404(),
        autre => interne("service des personnes", autre),
    }
}
