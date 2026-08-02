//! Handlers du référentiel d'hébergement — **HEB-01, HEB-03, HEB-04, HEB-05**, opérations 1 à 8
//! et 5b.
//!
//! `specs/004-unites-formules-disponibilite/contracts/http-api.md` §1 décrit ce que ce fichier
//! doit produire. La **source de vérité reste ce code** : le contrat OpenAPI est généré depuis les
//! annotations `#[utoipa::path]` ci-dessous, et le client TypeScript depuis ce contrat
//! (principe I·a, porte P-01).
//!
//! # Les trois rappels de forme, tenus ici
//!
//! - **Le chemin n'est écrit qu'une fois** : `#[utoipa::path(...)]` sans `path` ni verbe, tous
//!   deux déduits de l'attribut de routage Actix (feature `actix_extras`) ;
//! - **monté par `service(...)`, jamais `route(...)`** — `utoipa-actix-web` ne collecte que depuis
//!   `service(...)`, et un endpoint monté autrement serait servi sans figurer au contrat, donc
//!   invisible pour P-08 ;
//! - **`operation_id` explicite sur chacune** (P-01b) : deux opérations homonymes produisent un
//!   client TypeScript invalide, que P-01 ne détecte pas puisqu'elle ne compare que le généré au
//!   commité.
//!
//! # Ce que l'opération 5b refuse, et pourquoi elle le dit
//!
//! `PUT /unites/{unite_id}` ne porte que `code` et `etage`. Un corps qui nommerait `categorie_id`,
//! `statut_menage` ou une mise hors service est **refusé explicitement** — jamais ignoré en
//! silence, ce qui ferait croire à l'appelant que sa modification a été prise.

use std::collections::BTreeMap;

use actix_web::{HttpResponse, get, post, put, web};
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;

use kaya_hebergement::Issue;
use kaya_hebergement::referentiel::{
    CategorieVue, CreerCategorie, CreerFormule, CreerUnite, ErreurReferentiel, FamilleFormule,
    FormuleVue, ModifierCategorie, ModifierFormule, ModifierUnite, PalierVue, PlageDemandee,
    RegleConversionTaxe, TempsRemiseEnEtat, UniteVue,
};

use crate::application::EtatApplication;
use crate::contexte::ContexteAppel;
use crate::routes::erreurs::{CorpsErreur, interne};
use crate::securite::exiger;

/// Les deux permissions du référentiel, écrites une fois.
const LIRE: &str = "heb.offre.lire";
const GERER: &str = "heb.offre.gerer";

// =================================================================================================
//  Chemins
// =================================================================================================

#[derive(Debug, Deserialize)]
pub struct CheminCategorie {
    pub etablissement_id: Uuid,
    pub categorie_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct CheminUnite {
    pub etablissement_id: Uuid,
    pub unite_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct CheminFormule {
    pub etablissement_id: Uuid,
    pub formule_id: Uuid,
}

// =================================================================================================
//  Corps de requête
// =================================================================================================

/// Création d'un type de chambre.
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreerCategorieRequete {
    /// UUID v7 **généré par le client** — c'est lui qui rend le rejeu inoffensif.
    pub id: Uuid,
    pub nom: String,
    pub capacite_accueil: i16,
    /// Les battements de remise en état, par famille de formule. **Remplacés en bloc.**
    #[serde(default)]
    pub temps_remise_en_etat: Vec<TempsRemiseEnEtat>,
}

/// Modification d'un type de chambre — **remplacement complet**, jamais un correctif partiel.
#[derive(Debug, Deserialize, ToSchema)]
pub struct ModifierCategorieRequete {
    pub nom: String,
    pub capacite_accueil: i16,
    #[serde(default)]
    pub temps_remise_en_etat: Vec<TempsRemiseEnEtat>,
}

/// Création d'une chambre.
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreerUniteRequete {
    pub id: Uuid,
    pub categorie_id: Uuid,
    pub code: String,
    #[serde(default)]
    pub etage: Option<i16>,
}

/// **Correction d'une chambre — deux champs, et pas un de plus.**
///
/// Tout autre champ présent dans le corps est **capté** par `autres` et **refusé** avec son nom.
/// Le laisser passer en silence ferait croire à l'appelant que sa modification a été prise ; le
/// refuser sans le nommer l'obligerait à deviner lequel des siens pose problème.
#[derive(Debug, Deserialize, ToSchema)]
pub struct ModifierUniteRequete {
    pub code: String,
    #[serde(default)]
    pub etage: Option<i16>,
    /// Champs inattendus, captés pour être **nommés dans le refus**.
    ///
    /// Absent du schéma OpenAPI : le contrat ne doit pas suggérer qu'on peut envoyer autre chose.
    #[serde(flatten)]
    #[schema(ignore)]
    pub autres: BTreeMap<String, serde_json::Value>,
}

/// Un palier de barème, tel que le client l'envoie.
#[derive(Debug, Deserialize, ToSchema)]
pub struct PalierRequete {
    pub duree_minutes: i32,
    /// **Entier d'unité mineure** (P-10). La devise vient de l'établissement, jamais du corps.
    pub prix_mineur: i64,
}

/// Une plage de demi-journée, telle que le client l'envoie. Heures **murales locales** `HH:MM`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct PlageRequete {
    pub heure_debut: String,
    pub heure_fin: String,
    /// **Clé i18n, jamais une phrase.**
    pub libelle_cle: String,
}

/// Création d'une formule, **avec ses enfants**.
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreerFormuleRequete {
    pub id: Uuid,
    pub categorie_id: Uuid,
    pub famille: FamilleFormule,
    pub prix_mineur: i64,
    #[serde(default)]
    pub duree_min_minutes: Option<i32>,
    #[serde(default)]
    pub duree_max_minutes: Option<i32>,
    #[serde(default)]
    pub heure_arrivee_standard: Option<String>,
    #[serde(default)]
    pub heure_depart_standard: Option<String>,
    #[serde(default)]
    pub jours_autorises: Option<Vec<i16>>,
    pub assujettie_taxe_nuitee: bool,
    #[serde(default)]
    pub regle_conversion_taxe: Option<RegleConversionTaxe>,
    #[serde(default)]
    pub prix_heure_supplementaire_mineur: Option<i64>,
    #[serde(default)]
    pub paliers: Vec<PalierRequete>,
    #[serde(default)]
    pub plages: Vec<PlageRequete>,
}

/// Modification d'une formule — **c'est ici que l'exploitant règle la taxe**.
///
/// `famille` et `categorie_id` n'y figurent pas : changer la famille d'une formule reviendrait à
/// transformer une nuitée en passage en gardant son identifiant, et le montant dû sur un séjour en
/// cours changerait sous les pieds de l'exploitant.
#[derive(Debug, Deserialize, ToSchema)]
pub struct ModifierFormuleRequete {
    pub prix_mineur: i64,
    #[serde(default)]
    pub duree_min_minutes: Option<i32>,
    #[serde(default)]
    pub duree_max_minutes: Option<i32>,
    #[serde(default)]
    pub heure_arrivee_standard: Option<String>,
    #[serde(default)]
    pub heure_depart_standard: Option<String>,
    #[serde(default)]
    pub jours_autorises: Option<Vec<i16>>,
    /// Le drapeau que l'exploitant active quand sa commune impose la taxe de séjour.
    pub assujettie_taxe_nuitee: bool,
    /// « Une seule taxe pour tout le séjour » ou « Une taxe par nuit » (lexique). **`null` n'est
    /// permis que sur une formule non assujettie** — la base le garantit.
    #[serde(default)]
    pub regle_conversion_taxe: Option<RegleConversionTaxe>,
    #[serde(default)]
    pub prix_heure_supplementaire_mineur: Option<i64>,
    #[serde(default)]
    pub paliers: Vec<PalierRequete>,
    #[serde(default)]
    pub plages: Vec<PlageRequete>,
}

// =================================================================================================
//  1 · Lister les types de chambre
// =================================================================================================

#[utoipa::path(
    operation_id = "hebergement_lister_categories",
    tag = "hebergement",
    params(("etablissement_id" = Uuid, Path, description = "Identifiant de l'établissement")),
    responses(
        (status = 200, description = "Types de chambre de l'établissement", body = Vec<CategorieVue>),
        (status = 401, description = "Non authentifié"),
        (status = 403, description = "Permission absente", body = CorpsErreur),
        (status = 404, description = "Établissement inconnu", body = CorpsErreur),
        (status = 409, description = "Service hébergement non actif", body = CorpsErreur),
    ),
    security(("bearer" = []))
)]
#[get("")]
pub async fn lister_categories(
    etat: web::Data<EtatApplication>,
    contexte: ContexteAppel,
    chemin: web::Path<Uuid>,
) -> Result<HttpResponse, actix_web::Error> {
    exiger(&contexte, LIRE)?;
    let etablissement_id = chemin.into_inner();

    let categories = etat
        .service_hebergement(contexte.tenant_id)
        .lister_categories(contexte.tenant_id, etablissement_id)
        .await
        .map_err(en_reponse)?;

    Ok(HttpResponse::Ok().json(categories))
}

// =================================================================================================
//  2 · Créer un type de chambre
// =================================================================================================

#[utoipa::path(
    operation_id = "hebergement_creer_categorie",
    tag = "hebergement",
    params(("etablissement_id" = Uuid, Path, description = "Identifiant de l'établissement")),
    request_body = CreerCategorieRequete,
    responses(
        (status = 201, description = "Type de chambre créé", body = CategorieVue),
        (status = 200, description = "Déjà créé (rejeu idempotent)", body = CategorieVue),
        (status = 401, description = "Non authentifié"),
        (status = 403, description = "Permission absente", body = CorpsErreur),
        (status = 404, description = "Établissement inconnu", body = CorpsErreur),
        (status = 409, description = "Service hébergement non actif", body = CorpsErreur),
        (status = 422, description = "Refus métier", body = CorpsErreur),
    ),
    security(("bearer" = []))
)]
#[post("")]
pub async fn creer_categorie(
    etat: web::Data<EtatApplication>,
    contexte: ContexteAppel,
    chemin: web::Path<Uuid>,
    corps: web::Json<CreerCategorieRequete>,
) -> Result<HttpResponse, actix_web::Error> {
    exiger(&contexte, GERER)?;
    let etablissement_id = chemin.into_inner();
    let corps = corps.into_inner();

    let (vue, issue) = etat
        .service_hebergement(contexte.tenant_id)
        .creer_categorie(
            contexte.tenant_id,
            CreerCategorie {
                id: corps.id,
                etablissement_id,
                nom: corps.nom,
                capacite_accueil: corps.capacite_accueil,
                temps_remise_en_etat: corps.temps_remise_en_etat,
            },
        )
        .await
        .map_err(en_reponse)?;

    Ok(match issue {
        Issue::Creee => HttpResponse::Created().json(vue),
        Issue::DejaPresente => HttpResponse::Ok().json(vue),
    })
}

// =================================================================================================
//  3 · Modifier un type de chambre
// =================================================================================================

#[utoipa::path(
    operation_id = "hebergement_modifier_categorie",
    tag = "hebergement",
    params(
        ("etablissement_id" = Uuid, Path, description = "Identifiant de l'établissement"),
        ("categorie_id" = Uuid, Path, description = "Identifiant du type de chambre"),
    ),
    request_body = ModifierCategorieRequete,
    responses(
        (status = 200, description = "Type de chambre modifié", body = CategorieVue),
        (status = 401, description = "Non authentifié"),
        (status = 403, description = "Permission absente", body = CorpsErreur),
        (status = 404, description = "Type de chambre inconnu", body = CorpsErreur),
        (status = 409, description = "Service hébergement non actif", body = CorpsErreur),
        (status = 422, description = "Refus métier", body = CorpsErreur),
    ),
    security(("bearer" = []))
)]
#[put("")]
pub async fn modifier_categorie(
    etat: web::Data<EtatApplication>,
    contexte: ContexteAppel,
    chemin: web::Path<CheminCategorie>,
    corps: web::Json<ModifierCategorieRequete>,
) -> Result<HttpResponse, actix_web::Error> {
    exiger(&contexte, GERER)?;
    let chemin = chemin.into_inner();
    let corps = corps.into_inner();

    let vue = etat
        .service_hebergement(contexte.tenant_id)
        .modifier_categorie(
            contexte.tenant_id,
            chemin.etablissement_id,
            chemin.categorie_id,
            ModifierCategorie {
                nom: corps.nom,
                capacite_accueil: corps.capacite_accueil,
                temps_remise_en_etat: corps.temps_remise_en_etat,
            },
        )
        .await
        .map_err(en_reponse)?;

    Ok(HttpResponse::Ok().json(vue))
}

// =================================================================================================
//  4 · Lister les chambres
// =================================================================================================

#[utoipa::path(
    operation_id = "hebergement_lister_unites",
    tag = "hebergement",
    params(("etablissement_id" = Uuid, Path, description = "Identifiant de l'établissement")),
    responses(
        (status = 200, description = "Chambres de l'établissement", body = Vec<UniteVue>),
        (status = 401, description = "Non authentifié"),
        (status = 403, description = "Permission absente", body = CorpsErreur),
        (status = 404, description = "Établissement inconnu", body = CorpsErreur),
        (status = 409, description = "Service hébergement non actif", body = CorpsErreur),
    ),
    security(("bearer" = []))
)]
#[get("")]
pub async fn lister_unites(
    etat: web::Data<EtatApplication>,
    contexte: ContexteAppel,
    chemin: web::Path<Uuid>,
) -> Result<HttpResponse, actix_web::Error> {
    exiger(&contexte, LIRE)?;
    let etablissement_id = chemin.into_inner();

    let unites = etat
        .service_hebergement(contexte.tenant_id)
        .lister_unites(contexte.tenant_id, etablissement_id)
        .await
        .map_err(en_reponse)?;

    Ok(HttpResponse::Ok().json(unites))
}

// =================================================================================================
//  5 · Créer une chambre
// =================================================================================================

#[utoipa::path(
    operation_id = "hebergement_creer_unite",
    tag = "hebergement",
    params(("etablissement_id" = Uuid, Path, description = "Identifiant de l'établissement")),
    request_body = CreerUniteRequete,
    responses(
        (status = 201, description = "Chambre créée", body = UniteVue),
        (status = 200, description = "Déjà créée (rejeu idempotent)", body = UniteVue),
        (status = 401, description = "Non authentifié"),
        (status = 403, description = "Permission absente", body = CorpsErreur),
        (status = 404, description = "Type de chambre inconnu", body = CorpsErreur),
        (status = 409, description = "Service hébergement non actif", body = CorpsErreur),
        (status = 422, description = "Refus métier", body = CorpsErreur),
    ),
    security(("bearer" = []))
)]
#[post("")]
pub async fn creer_unite(
    etat: web::Data<EtatApplication>,
    contexte: ContexteAppel,
    chemin: web::Path<Uuid>,
    corps: web::Json<CreerUniteRequete>,
) -> Result<HttpResponse, actix_web::Error> {
    exiger(&contexte, GERER)?;
    let etablissement_id = chemin.into_inner();
    let corps = corps.into_inner();

    let (vue, issue) = etat
        .service_hebergement(contexte.tenant_id)
        .creer_unite(
            contexte.tenant_id,
            CreerUnite {
                id: corps.id,
                etablissement_id,
                categorie_id: corps.categorie_id,
                code: corps.code,
                etage: corps.etage,
            },
        )
        .await
        .map_err(en_reponse)?;

    Ok(match issue {
        Issue::Creee => HttpResponse::Created().json(vue),
        Issue::DejaPresente => HttpResponse::Ok().json(vue),
    })
}

// =================================================================================================
//  5b · Corriger une chambre — DEUX CHAMPS, et le refus nomme les autres
// =================================================================================================

/// Les trois champs classés ailleurs, refusés **nommément**.
///
/// | Champ | Où il est classé |
/// |---|---|
/// | `categorie_id` | Nulle part au registre — effet tarifaire et fiscal, **ça se spécifie** |
/// | `statut_menage` | Classe **A**, HEB-06 |
/// | `hors_service` | Classe **B**, HEB-06 — opération de disponibilité, pas de référentiel |
const CHAMPS_REFUSES: [&str; 3] = ["categorie_id", "statut_menage", "hors_service"];

#[utoipa::path(
    operation_id = "hebergement_modifier_unite",
    tag = "hebergement",
    params(
        ("etablissement_id" = Uuid, Path, description = "Identifiant de l'établissement"),
        ("unite_id" = Uuid, Path, description = "Identifiant de la chambre"),
    ),
    request_body = ModifierUniteRequete,
    responses(
        (status = 200, description = "Chambre corrigée", body = UniteVue),
        (status = 401, description = "Non authentifié"),
        (status = 403, description = "Permission absente", body = CorpsErreur),
        (status = 404, description = "Chambre inconnue", body = CorpsErreur),
        (status = 409, description = "Service hébergement non actif", body = CorpsErreur),
        (status = 422, description = "Champ non modifiable par cette opération", body = CorpsErreur),
    ),
    security(("bearer" = []))
)]
#[put("")]
pub async fn modifier_unite(
    etat: web::Data<EtatApplication>,
    contexte: ContexteAppel,
    chemin: web::Path<CheminUnite>,
    corps: web::Json<ModifierUniteRequete>,
) -> Result<HttpResponse, actix_web::Error> {
    exiger(&contexte, GERER)?;
    let chemin = chemin.into_inner();
    let corps = corps.into_inner();

    // **Le refus précède l'écriture, et il nomme le champ.** Ignorer silencieusement ferait croire
    // à l'appelant que sa modification a été prise en compte.
    if let Some(champ) = CHAMPS_REFUSES
        .iter()
        .find(|c| corps.autres.contains_key(**c))
    {
        return Err(CorpsErreur::nouveau(
            "champ_non_modifiable",
            Some((*champ).to_owned()),
            format!(
                "« {champ} » n'est pas modifiable par cette opération : elle ne sert que `code` \
                 et `etage`, les deux champs que le registre des classes hors-ligne classe en C"
            ),
        )
        .en_422());
    }

    let vue = etat
        .service_hebergement(contexte.tenant_id)
        .modifier_unite(
            contexte.tenant_id,
            chemin.etablissement_id,
            chemin.unite_id,
            ModifierUnite {
                code: corps.code,
                etage: corps.etage,
            },
        )
        .await
        .map_err(en_reponse)?;

    Ok(HttpResponse::Ok().json(vue))
}

// =================================================================================================
//  6 · Lister les formules — la seule opération que l'écran G2 consomme en lecture
// =================================================================================================

#[utoipa::path(
    operation_id = "hebergement_lister_formules",
    tag = "hebergement",
    params(("etablissement_id" = Uuid, Path, description = "Identifiant de l'établissement")),
    responses(
        (status = 200, description = "Formules, avec leurs paliers et leurs plages", body = Vec<FormuleVue>),
        (status = 401, description = "Non authentifié"),
        (status = 403, description = "Permission absente", body = CorpsErreur),
        (status = 404, description = "Établissement inconnu", body = CorpsErreur),
        (status = 409, description = "Service hébergement non actif", body = CorpsErreur),
    ),
    security(("bearer" = []))
)]
#[get("")]
pub async fn lister_formules(
    etat: web::Data<EtatApplication>,
    contexte: ContexteAppel,
    chemin: web::Path<Uuid>,
) -> Result<HttpResponse, actix_web::Error> {
    exiger(&contexte, LIRE)?;
    let etablissement_id = chemin.into_inner();

    let formules = etat
        .service_hebergement(contexte.tenant_id)
        .lister_formules(contexte.tenant_id, etablissement_id)
        .await
        .map_err(en_reponse)?;

    Ok(HttpResponse::Ok().json(formules))
}

// =================================================================================================
//  7 · Créer une formule
// =================================================================================================

#[utoipa::path(
    operation_id = "hebergement_creer_formule",
    tag = "hebergement",
    params(("etablissement_id" = Uuid, Path, description = "Identifiant de l'établissement")),
    request_body = CreerFormuleRequete,
    responses(
        (status = 201, description = "Formule créée", body = FormuleVue),
        (status = 200, description = "Déjà créée (rejeu idempotent)", body = FormuleVue),
        (status = 401, description = "Non authentifié"),
        (status = 403, description = "Permission absente", body = CorpsErreur),
        (status = 404, description = "Type de chambre inconnu", body = CorpsErreur),
        (status = 409, description = "Service hébergement non actif", body = CorpsErreur),
        (status = 422, description = "Barème ou plages absents, famille inconnue", body = CorpsErreur),
    ),
    security(("bearer" = []))
)]
#[post("")]
pub async fn creer_formule(
    etat: web::Data<EtatApplication>,
    contexte: ContexteAppel,
    chemin: web::Path<Uuid>,
    corps: web::Json<CreerFormuleRequete>,
) -> Result<HttpResponse, actix_web::Error> {
    exiger(&contexte, GERER)?;
    let etablissement_id = chemin.into_inner();
    let corps = corps.into_inner();

    let (vue, issue) = etat
        .service_hebergement(contexte.tenant_id)
        .creer_formule(
            contexte.tenant_id,
            CreerFormule {
                id: corps.id,
                etablissement_id,
                categorie_id: corps.categorie_id,
                famille: corps.famille,
                prix_mineur: corps.prix_mineur,
                duree_min_minutes: corps.duree_min_minutes,
                duree_max_minutes: corps.duree_max_minutes,
                heure_arrivee_standard: corps.heure_arrivee_standard,
                heure_depart_standard: corps.heure_depart_standard,
                jours_autorises: corps.jours_autorises,
                assujettie_taxe_nuitee: corps.assujettie_taxe_nuitee,
                regle_conversion_taxe: corps.regle_conversion_taxe,
                prix_heure_supplementaire_mineur: corps.prix_heure_supplementaire_mineur,
                paliers: corps.paliers.into_iter().map(en_palier).collect(),
                plages: corps.plages.into_iter().map(en_plage).collect(),
            },
        )
        .await
        .map_err(en_reponse)?;

    Ok(match issue {
        Issue::Creee => HttpResponse::Created().json(vue),
        Issue::DejaPresente => HttpResponse::Ok().json(vue),
    })
}

// =================================================================================================
//  8 · Modifier une formule — LES DEUX CHAMPS FISCAUX
// =================================================================================================

/// C'est là que l'exploitant active la taxe quand sa commune l'impose, et qu'il choisit entre
/// « Une seule taxe pour tout le séjour » et « Une taxe par nuit ».
///
/// **Aucune règle fiscale n'est appliquée ici** : le champ est stocké, jamais interprété. La règle
/// qui le consommera vivra dans `JurisdictionAdapter` (`socle/fiscalite`), en T3 — porte P-12.
#[utoipa::path(
    operation_id = "hebergement_modifier_formule",
    tag = "hebergement",
    params(
        ("etablissement_id" = Uuid, Path, description = "Identifiant de l'établissement"),
        ("formule_id" = Uuid, Path, description = "Identifiant de la formule"),
    ),
    request_body = ModifierFormuleRequete,
    responses(
        (status = 200, description = "Formule modifiée", body = FormuleVue),
        (status = 401, description = "Non authentifié"),
        (status = 403, description = "Permission absente", body = CorpsErreur),
        (status = 404, description = "Formule inconnue", body = CorpsErreur),
        (status = 409, description = "Service hébergement non actif", body = CorpsErreur),
        (status = 422, description = "Barème ou plages absents, règle fiscale incohérente", body = CorpsErreur),
    ),
    security(("bearer" = []))
)]
#[put("")]
pub async fn modifier_formule(
    etat: web::Data<EtatApplication>,
    contexte: ContexteAppel,
    chemin: web::Path<CheminFormule>,
    corps: web::Json<ModifierFormuleRequete>,
) -> Result<HttpResponse, actix_web::Error> {
    exiger(&contexte, GERER)?;
    let chemin = chemin.into_inner();
    let corps = corps.into_inner();

    let vue = etat
        .service_hebergement(contexte.tenant_id)
        .modifier_formule(
            contexte.tenant_id,
            chemin.etablissement_id,
            chemin.formule_id,
            ModifierFormule {
                prix_mineur: corps.prix_mineur,
                duree_min_minutes: corps.duree_min_minutes,
                duree_max_minutes: corps.duree_max_minutes,
                heure_arrivee_standard: corps.heure_arrivee_standard,
                heure_depart_standard: corps.heure_depart_standard,
                jours_autorises: corps.jours_autorises,
                assujettie_taxe_nuitee: corps.assujettie_taxe_nuitee,
                regle_conversion_taxe: corps.regle_conversion_taxe,
                prix_heure_supplementaire_mineur: corps.prix_heure_supplementaire_mineur,
                paliers: corps.paliers.into_iter().map(en_palier).collect(),
                plages: corps.plages.into_iter().map(en_plage).collect(),
            },
        )
        .await
        .map_err(en_reponse)?;

    Ok(HttpResponse::Ok().json(vue))
}

// =================================================================================================
//  Conversions et traduction des refus
// =================================================================================================

fn en_palier(p: PalierRequete) -> PalierVue {
    PalierVue {
        duree_minutes: p.duree_minutes,
        prix_mineur: p.prix_mineur,
    }
}

fn en_plage(p: PlageRequete) -> PlageDemandee {
    PlageDemandee {
        heure_debut: p.heure_debut,
        heure_fin: p.heure_fin,
        libelle_cle: p.libelle_cle,
    }
}

/// Traduit un refus du domaine en réponse HTTP.
///
/// **Aucun détail interne ne franchit la frontière** : ni message PostgreSQL, ni nom de table, ni
/// trace. Le détail part dans les journaux, corrélé par l'identifiant de requête.
///
/// L'interface branche sa clé i18n sur le `code`, jamais sur le `message` — qui nomme des tables
/// et parle anglais technique.
pub(crate) fn en_reponse(erreur: ErreurReferentiel) -> actix_web::Error {
    let code = erreur.code();
    match erreur {
        ErreurReferentiel::EtablissementInconnu
        | ErreurReferentiel::CategorieInconnue
        | ErreurReferentiel::UniteInconnue
        | ErreurReferentiel::FormuleInconnue => {
            CorpsErreur::nouveau(code, None, erreur.to_string()).en_404()
        }
        // `service_inactif` est un **409**, pas un 404 : l'établissement existe, il ne fait
        // simplement pas d'hébergement. L'interface doit proposer d'ajouter le service, pas
        // afficher « introuvable ».
        ErreurReferentiel::ServiceInactif => {
            CorpsErreur::nouveau(code, None, erreur.to_string()).en_409()
        }
        ErreurReferentiel::FamilleInconnue(ref v)
        | ErreurReferentiel::RegleConversionInconnue(ref v)
        | ErreurReferentiel::StatutMenageInconnu(ref v)
        | ErreurReferentiel::ChampNonModifiable(ref v)
        | ErreurReferentiel::HeureInvalide(ref v) => {
            let valeur = v.clone();
            CorpsErreur::nouveau(code, Some(valeur), erreur.to_string()).en_422()
        }
        ErreurReferentiel::BaremeAbsent
        | ErreurReferentiel::PlagesAbsentes
        | ErreurReferentiel::CategorieOccupee { .. } => {
            CorpsErreur::nouveau(code, None, erreur.to_string()).en_422()
        }
        autre => interne("référentiel d'hébergement", autre),
    }
}
