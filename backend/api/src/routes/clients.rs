//! Handlers de la fiche client — **SEJ-01**, opérations 1 à 4 et 6 du contrat du cycle 006.
//!
//! # L'opération 5 n'est PAS ici, et c'est structurel
//!
//! `GET /api/v1/clients/{client_id}/sejours` — l'historique des séjours — **paraît** appartenir à
//! ce fichier. Elle est servie depuis `routes/sejours.rs`, sur le crate `hebergement`.
//!
//! Si `socle/comptes` lisait `hebergement.sejour`, ce serait **deux violations d'un coup** :
//! jointure inter-schémas (**P-04**) *et* arête `socle/ → verticales/` (**P-03**). Le chemin HTTP
//! cache ce découpage à l'appelant, et c'est normal : **le contrat est une façade, pas une carte
//! des crates.**
//!
//! # Aucun `etablissement_id` dans ces chemins
//!
//! La fiche est du **tenant**, pas d'un établissement (FR-002). Un client de Deloria enregistré à
//! l'accueil est le même client au restaurant, et ses préférences le suivent. C'est aussi ce qui
//! rend les deux permissions transversales (`module_code = NULL`, migration `0030`) : un maquis
//! sans hébergement en aura besoin dès SEJ-05.

use actix_web::{HttpResponse, get, patch, post, web};
use serde::Deserialize;
use time::{Date, OffsetDateTime};
use utoipa::ToSchema;
use uuid::Uuid;

use kaya_comptes::client::{
    CreerClient, ErreurClient, FicheClient, ModifierClient, Preference, ResultatRecherche,
};
use kaya_etablissements::Issue;
use kaya_etablissements::traits::Cible;

use crate::application::EtatApplication;
use crate::contexte::ContexteAppel;
use crate::routes::erreurs::{CorpsErreur, interne};
use crate::securite;

/// Permissions du contrat, nommées une fois.
///
/// **Transversales** : `module_code = NULL` en base. Un maquis ou un bar seul en aura besoin dès
/// SEJ-05, sans module hébergement (research R-13).
const PERM_LIRE: &str = "sej.client.lire";
const PERM_GERER: &str = "sej.client.gerer";

/// Clé de configuration héritée portant l'indicatif téléphonique par défaut.
///
/// **Jamais `+225` en dur** (principe I·c, porte P-12) : une constante ivoirienne ferait échouer
/// le premier établissement togolais, et le ferait échouer **silencieusement** — en rendant
/// introuvables des fiches pourtant créées.
const CLE_INDICATIF: &str = "indicatif_telephonique_defaut";

/// Repli employé quand aucun indicatif n'est configuré.
///
/// **Une chaîne vide, jamais un indicatif de pays.** Sans configuration, le numéro est replié tel
/// qu'il a été saisi : la recherche par suffixe le retrouve, et aucun pays n'est supposé. Poser
/// `+225` ici serait exactement le paramètre en dur déguisé en commodité que le principe I·c
/// interdit.
const INDICATIF_ABSENT: &str = "";

// =================================================================================================
//  Corps de réponse
// =================================================================================================

/// La fiche **et ses préférences**, rendues ensemble.
///
/// ★ **`#[serde(flatten)]`, et c'est ce qui garde le contrat compatible.** Les champs de la fiche
/// restent au premier niveau — `nom`, `telephone`, `numero_piece` — exactement là où ils étaient
/// avant que les préférences ne s'y ajoutent. Les envelopper sous une clé `fiche` aurait été plus
/// « propre » et aurait cassé chaque appelant existant pour un gain nul.
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct FicheClientDetail {
    #[serde(flatten)]
    #[schema(inline)]
    pub fiche: FicheClient,
    /// De la plus récente à la plus ancienne. **Append-only** : une préférence ne se modifie ni ne
    /// s'efface — « allergique aux arachides » raturé et réécrit ne laisse aucune trace de qui a
    /// raturé.
    pub preferences: Vec<Preference>,
}

// =================================================================================================
//  Corps de requête
// =================================================================================================

/// Paramètres de la recherche.
#[derive(Debug, Deserialize, ToSchema, utoipa::IntoParams)]
pub struct RechercheRequete {
    /// La saisie brute de l'opérateur. **Une seule entrée pour trois formes** : le serveur déduit
    /// s'il s'agit d'un nom, d'un téléphone ou d'un numéro de pièce. Au comptoir, l'opérateur ne
    /// choisit pas un mode.
    #[serde(default)]
    pub recherche: String,
    /// Nombre maximal de résultats. Ramené au plafond du service quand il le dépasse.
    #[serde(default)]
    pub limite: Option<i64>,
}

/// Corps de création d'une fiche.
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreerClientRequete {
    /// UUID v7 **généré par le client** (FR-086) : c'est lui qui rend le rejeu inoffensif —
    /// `201`, puis `200`, `200`.
    pub id: Uuid,
    pub nom: String,
    #[serde(default)]
    pub prenoms: Option<String>,
    #[serde(default)]
    pub date_naissance: Option<Date>,
    #[serde(default)]
    pub nationalite: Option<String>,
    /// E.164, ou national — l'indicatif de l'établissement complète la saisie.
    #[serde(default)]
    pub telephone: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub type_piece: Option<String>,
    /// ⚠️ **Chiffré au repos dès réception** (FR-012). Il n'est **jamais** rendu par cette
    /// opération ni par la recherche, et n'entre **jamais** dans l'outbox.
    #[serde(default)]
    pub numero_piece: Option<String>,
    /// Indicatif — **jamais employé par une règle métier** (porte P-23).
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub horodatage_client: Option<OffsetDateTime>,
}

/// Corps de modification — **remplacement complet des champs modifiables**.
///
/// Le verbe est `PATCH` au contrat, la sémantique est celle d'un remplacement : une fusion champ
/// par champ rendrait impossible d'effacer un numéro de téléphone, l'absence du champ et sa mise à
/// `null` étant indistinguables après désérialisation.
#[derive(Debug, Deserialize, ToSchema)]
pub struct ModifierClientRequete {
    pub nom: String,
    #[serde(default)]
    pub prenoms: Option<String>,
    #[serde(default)]
    pub date_naissance: Option<Date>,
    #[serde(default)]
    pub nationalite: Option<String>,
    #[serde(default)]
    pub telephone: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub type_piece: Option<String>,
    #[serde(default)]
    pub numero_piece: Option<String>,
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub horodatage_client: Option<OffsetDateTime>,
}

/// Corps d'enregistrement d'une préférence — **classe A, append-only**.
#[derive(Debug, Deserialize, ToSchema)]
pub struct PreferenceRequete {
    /// UUID v7 généré par le client. Rejeu triple → un enregistrement, **et aucun second
    /// événement outbox**.
    pub id: Uuid,
    pub texte: String,
    /// Accepté et **indicatif** ; ne porte **aucune règle** (porte P-23). L'ordre des préférences
    /// vient de `cree_le`, l'horodatage d'autorité.
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub horodatage_client: Option<OffsetDateTime>,
}

// =================================================================================================
//  1 · GET /api/v1/clients — la recherche
// =================================================================================================

/// Cherche des fiches clientes — **trois formes, une entrée**.
///
/// La forme est déduite de la saisie : que des chiffres → téléphone ; alphanumérique compact avec
/// au moins un chiffre → numéro de pièce ; sinon → nom. Une saisie ambiguë interroge **les trois**
/// et fusionne.
///
/// **Seules des personnes qualifiées clientes sont rendues** : le personnel n'y apparaît jamais
/// (FR-004).
///
/// `tronque` dit qu'il y avait plus de résultats que la limite — une liste silencieusement coupée
/// est un mensonge sur un écran de comptoir : Yao conclurait que la fiche n'existe pas et en
/// créerait une seconde.
#[utoipa::path(
    operation_id = "client_rechercher",
    tag = "clients",
    params(RechercheRequete),
    responses(
        (status = 200, description = "Les fiches trouvées, et l'indication de troncature", body = ResultatRecherche),
        (status = 401, description = "Non authentifié"),
        (status = 403, description = "Permission absente", body = CorpsErreur),
    ),
    security(("bearer" = []))
)]
#[get("")]
pub async fn rechercher(
    etat: web::Data<EtatApplication>,
    contexte: ContexteAppel,
    requete: web::Query<RechercheRequete>,
) -> Result<HttpResponse, actix_web::Error> {
    securite::exiger(&contexte, PERM_LIRE)?;
    let requete = requete.into_inner();

    let indicatif = indicatif_du_tenant(&etat, &contexte).await;

    let resultat = etat
        .service_client()
        .rechercher(
            contexte.tenant_id,
            &indicatif,
            &requete.recherche,
            requete.limite,
        )
        .await
        .map_err(en_reponse)?;

    Ok(HttpResponse::Ok().json(resultat))
}

// =================================================================================================
//  2 · POST /api/v1/clients — créer une fiche
// =================================================================================================

/// Crée une fiche client.
///
/// **`200` sur rejeu, jamais `409`** : un terminal qui vide sa file après une coupure ne doit pas
/// voir d'erreur pour une écriture déjà acceptée. Le corps rendu est la ligne **telle qu'elle est
/// en base** — le serveur fait foi en conflit.
///
/// ⚠️ **Classe C** : refusée immédiatement et explicitement hors ligne, jamais mise en file
/// (porte P-13). Décision O-01, option (a), tranchée le 2026-08-03 — un client jamais vu exige le
/// réseau (FR-011).
///
/// ⚠️ **`numero_piece` n'est pas rendu par cette opération**, bien qu'il vienne d'être fourni :
/// l'appelant le connaît déjà. Le rendre produirait une entrée au registre des actions pour une
/// consultation qui n'en est pas une, et un registre bruyant n'est plus lu.
#[utoipa::path(
    operation_id = "client_creer",
    tag = "clients",
    request_body = CreerClientRequete,
    responses(
        (status = 201, description = "Fiche créée", body = FicheClient),
        (status = 200, description = "Fiche déjà créée (rejeu idempotent)", body = FicheClient),
        (status = 401, description = "Non authentifié"),
        (status = 403, description = "Permission absente", body = CorpsErreur),
        (status = 422, description = "nom_vide, telephone_invalide, nationalite_invalide", body = CorpsErreur),
    ),
    security(("bearer" = []))
)]
#[post("")]
pub async fn creer(
    etat: web::Data<EtatApplication>,
    contexte: ContexteAppel,
    corps: web::Json<CreerClientRequete>,
) -> Result<HttpResponse, actix_web::Error> {
    securite::exiger(&contexte, PERM_GERER)?;
    let corps = corps.into_inner();
    let indicatif = indicatif_du_tenant(&etat, &contexte).await;

    let (fiche, issue) = etat
        .service_client()
        .creer(
            contexte.tenant_id,
            &indicatif,
            CreerClient {
                id: corps.id,
                nom: corps.nom,
                prenoms: corps.prenoms,
                telephone: corps.telephone,
                email: corps.email,
                date_naissance: corps.date_naissance,
                nationalite: corps.nationalite,
                type_piece: corps.type_piece,
                numero_piece: corps.numero_piece,
                horodatage_client: corps.horodatage_client,
            },
        )
        .await
        .map_err(en_reponse)?;

    Ok(match issue {
        Issue::Creee => HttpResponse::Created().json(fiche),
        Issue::DejaPresente => HttpResponse::Ok().json(fiche),
    })
}

// =================================================================================================
//  3 · GET /api/v1/clients/{client_id} — ★ le seul chemin qui déchiffre
// =================================================================================================

/// Lit une fiche complète.
///
/// ★ **C'est le seul point d'entrée du produit qui rend un numéro de pièce d'identité en clair**,
/// et il **journalise la consultation** au registre des actions (FR-012, principe IX) —
/// famille `consultation_piece_identite`, dans la **même transaction** que la lecture.
///
/// La trace n'est écrite que si un numéro est réellement déchiffré : lire une fiche sans pièce
/// n'est pas un accès à une pièce, et tracer toutes les lectures noierait les vraies consultations
/// sous des entrées vides.
///
/// # ★ Les préférences voyagent AVEC la fiche, et non par une dix-huitième opération
///
/// L'écran `R5` affiche l'identité, les coordonnées **et** les préférences dans le même volet ;
/// les demander séparément afficherait un instant une fiche sans ses préférences, et coûterait un
/// aller-retour de plus sur un réseau qui les fait payer.
///
/// ⚠️ **`ServiceClient::preferences` existait, testé, et n'était appelé de nulle part.** C'est
/// exactement le défaut que le cycle 003 a payé cher — *« une unité écrite n'est ni testée ni
/// branchée par défaut »* : `initialiserTheme()` a vécu deux cycles exportée, documentée « à
/// appeler au démarrage », et appelée nulle part. Ce chemin-ci est le sien.
#[utoipa::path(
    operation_id = "client_lire",
    tag = "clients",
    params(("client_id" = Uuid, Path, description = "Identifiant de la fiche")),
    responses(
        (status = 200, description = "La fiche et ses préférences, numéro de pièce compris — consultation journalisée", body = FicheClientDetail),
        (status = 401, description = "Non authentifié"),
        (status = 403, description = "Permission absente", body = CorpsErreur),
        (status = 404, description = "Fiche inconnue", body = CorpsErreur),
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
    let client_id = chemin.into_inner();
    let service = etat.service_client();

    let fiche = service
        .lire(contexte.tenant_id, contexte.compte_id, client_id)
        .await
        .map_err(en_reponse)?;

    // La lecture des préférences suit celle de la fiche, jamais l'inverse : sur une fiche
    // inconnue, le `404` doit venir de la fiche — une liste de préférences vide serait un `200`
    // sur un client qui n'existe pas.
    let preferences = service
        .preferences(contexte.tenant_id, client_id)
        .await
        .map_err(en_reponse)?;

    Ok(HttpResponse::Ok().json(FicheClientDetail { fiche, preferences }))
}

// =================================================================================================
//  4 · PATCH /api/v1/clients/{client_id}
// =================================================================================================

/// Modifie une fiche — **remplacement complet des champs modifiables**.
#[utoipa::path(
    operation_id = "client_modifier",
    tag = "clients",
    params(("client_id" = Uuid, Path, description = "Identifiant de la fiche")),
    request_body = ModifierClientRequete,
    responses(
        (status = 200, description = "Fiche modifiée", body = FicheClient),
        (status = 401, description = "Non authentifié"),
        (status = 403, description = "Permission absente", body = CorpsErreur),
        (status = 404, description = "Fiche inconnue", body = CorpsErreur),
        (status = 422, description = "nom_vide, telephone_invalide, nationalite_invalide", body = CorpsErreur),
    ),
    security(("bearer" = []))
)]
#[patch("")]
pub async fn modifier(
    etat: web::Data<EtatApplication>,
    contexte: ContexteAppel,
    chemin: web::Path<Uuid>,
    corps: web::Json<ModifierClientRequete>,
) -> Result<HttpResponse, actix_web::Error> {
    securite::exiger(&contexte, PERM_GERER)?;
    let corps = corps.into_inner();
    let indicatif = indicatif_du_tenant(&etat, &contexte).await;

    let fiche = etat
        .service_client()
        .modifier(
            contexte.tenant_id,
            &indicatif,
            chemin.into_inner(),
            ModifierClient {
                nom: corps.nom,
                prenoms: corps.prenoms,
                telephone: corps.telephone,
                email: corps.email,
                date_naissance: corps.date_naissance,
                nationalite: corps.nationalite,
                type_piece: corps.type_piece,
                numero_piece: corps.numero_piece,
                horodatage_client: corps.horodatage_client,
            },
        )
        .await
        .map_err(en_reponse)?;

    Ok(HttpResponse::Ok().json(fiche))
}

// =================================================================================================
//  6 · POST /api/v1/clients/{client_id}/preferences — classe A
// =================================================================================================

/// Enregistre une préférence — **append-only**.
///
/// La préférence courante est **la ligne la plus récente**, jamais une colonne mise à jour. Une
/// correction est une ligne nouvelle : c'est ce qui rend le rejeu inoffensif et le désordre
/// commutatif, les deux propriétés que `tester_classe_a!` vérifie.
///
/// **Rejeu triple → un enregistrement, et aucun second événement outbox.**
#[utoipa::path(
    operation_id = "client_preference_enregistrer",
    tag = "clients",
    params(("client_id" = Uuid, Path, description = "Identifiant de la fiche")),
    request_body = PreferenceRequete,
    responses(
        (status = 201, description = "Préférence enregistrée", body = Preference),
        (status = 200, description = "Préférence déjà enregistrée (rejeu idempotent)", body = Preference),
        (status = 401, description = "Non authentifié"),
        (status = 403, description = "Permission absente", body = CorpsErreur),
        (status = 404, description = "Fiche inconnue", body = CorpsErreur),
        (status = 422, description = "preference_invalide", body = CorpsErreur),
    ),
    security(("bearer" = []))
)]
#[post("")]
pub async fn enregistrer_preference(
    etat: web::Data<EtatApplication>,
    contexte: ContexteAppel,
    chemin: web::Path<Uuid>,
    corps: web::Json<PreferenceRequete>,
) -> Result<HttpResponse, actix_web::Error> {
    securite::exiger(&contexte, PERM_GERER)?;
    let corps = corps.into_inner();

    let (preference, issue) = etat
        .service_client()
        .enregistrer_preference(
            contexte.tenant_id,
            chemin.into_inner(),
            corps.id,
            &corps.texte,
            corps.horodatage_client,
        )
        .await
        .map_err(en_reponse)?;

    Ok(match issue {
        Issue::Creee => HttpResponse::Created().json(preference),
        Issue::DejaPresente => HttpResponse::Ok().json(preference),
    })
}

// =================================================================================================
//  Fonctions internes
// =================================================================================================

/// Résout l'indicatif téléphonique par défaut du tenant.
///
/// **La cible n'a pas d'établissement**, et c'est cohérent avec FR-002 : la fiche est du tenant.
/// La descente de chaîne s'arrête donc au niveau tenant, ce que `Cible` sait faire sans fabriquer
/// de niveau fictif (FR-050).
///
/// Une résolution en échec — Redis, base — **ne fait pas échouer la requête** : elle rend
/// l'indicatif vide, et le numéro est replié tel qu'il a été saisi. Refuser une création de fiche
/// parce qu'un paramètre de confort n'a pas pu être lu serait disproportionné au comptoir.
async fn indicatif_du_tenant(etat: &EtatApplication, contexte: &ContexteAppel) -> String {
    let cible = Cible {
        tenant_id: contexte.tenant_id,
        etablissement_id: None,
        module_code: None,
        point_de_vente_id: None,
    };

    match etat.service_configuration().resoudre(&cible, CLE_INDICATIF).await {
        Ok(Some(resolue)) => resolue
            .valeur
            .as_str()
            .map(str::to_owned)
            .unwrap_or_else(|| INDICATIF_ABSENT.to_owned()),
        _ => INDICATIF_ABSENT.to_owned(),
    }
}

/// Traduit une erreur de domaine en réponse HTTP.
///
/// **Aucun détail interne ne franchit la frontière** : ni message PostgreSQL, ni nom de table, ni
/// — surtout — la valeur qu'un déchiffrement n'a pas su lire. L'interface branche sa clé i18n sur
/// le `code`, jamais sur le `message`.
fn en_reponse(erreur: ErreurClient) -> actix_web::Error {
    match erreur {
        ErreurClient::NomVide => CorpsErreur::nouveau(
            erreur_code(&ErreurClient::NomVide),
            None,
            "le nom doit compter entre 1 et 200 caractères après nettoyage".to_owned(),
        )
        .en_422(),
        ErreurClient::TelephoneInvalide => CorpsErreur::nouveau(
            "telephone_invalide",
            None,
            "le numéro de téléphone ne comporte pas assez de chiffres".to_owned(),
        )
        .en_422(),
        ErreurClient::NationaliteInvalide => CorpsErreur::nouveau(
            "nationalite_invalide",
            None,
            "la nationalité doit compter entre 2 et 80 caractères".to_owned(),
        )
        .en_422(),
        ErreurClient::PreferenceInvalide => CorpsErreur::nouveau(
            "preference_invalide",
            None,
            "le texte doit compter entre 1 et 2000 caractères".to_owned(),
        )
        .en_422(),
        ErreurClient::Inconnu => CorpsErreur::nouveau(
            "client_inconnu",
            None,
            "aucune fiche de cet identifiant dans ce tenant".to_owned(),
        )
        .en_404(),
        autre => interne("service de la fiche client", autre),
    }
}

/// Le code stable, lu du type d'erreur plutôt que recopié.
fn erreur_code(erreur: &ErreurClient) -> &'static str {
    erreur.code()
}
