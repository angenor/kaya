//! Handlers des notes internes — **cinquième couche du module doré**.
//!
//! `specs/001-socle-technique-monorepo/contracts/http-api.md` §2 décrit ce que ce fichier doit
//! produire. La **source de vérité reste ce code** : le contrat OpenAPI est généré depuis les
//! annotations `#[utoipa::path]` ci-dessous, et le client TypeScript depuis ce contrat
//! (principe I(a), porte P-01).

use actix_web::{HttpResponse, get, post, web};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use kaya_etablissements::note::{CreerNote, ErreurNote, Issue, NoteEtablissement};

use crate::application::EtatApplication;
use crate::contexte::ContexteAppel;

/// Corps de création d'une note.
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreerNoteRequete {
    /// UUID v7 **généré par le client**. C'est lui qui rend le rejeu inoffensif : trois envois du
    /// même identifiant produisent un seul enregistrement.
    pub id: Uuid,
    /// Texte de la note — entre 1 et 2000 caractères après nettoyage.
    pub texte: String,
    /// Indicatif : ordre d'affichage local. **Jamais utilisé par une règle métier.**
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub horodatage_client: Option<OffsetDateTime>,
}

/// Pagination.
#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct PaginationParams {
    /// Nombre maximal d'éléments renvoyés. Défaut 50, plafond 200.
    pub limite: Option<i64>,
    /// Nombre d'éléments ignorés en tête.
    pub decalage: Option<i64>,
}

impl PaginationParams {
    // Le plafond n'est pas une politesse : sans lui, un appel unique pourrait demander l'entier
    // d'un historique qui grandit sans fin, et la mémoire du serveur avec.
    const LIMITE_DEFAUT: i64 = 50;
    const LIMITE_MAX: i64 = 200;

    fn limite(&self) -> i64 {
        self.limite
            .unwrap_or(Self::LIMITE_DEFAUT)
            .clamp(1, Self::LIMITE_MAX)
    }

    fn decalage(&self) -> i64 {
        self.decalage.unwrap_or(0).max(0)
    }
}

/// Une page de notes.
#[derive(Debug, Serialize, ToSchema)]
pub struct PageNotes {
    pub elements: Vec<NoteEtablissement>,
    pub total: i64,
    pub limite: i64,
    pub decalage: i64,
}

/// Liste les notes internes d'un établissement.
// Le chemin et le verbe ne sont PAS répétés dans l'annotation utoipa : ils sont déduits de
// l'attribut de routage d'Actix ci-dessous (feature `actix_extras`). Les écrire deux fois
// laisserait le contrat et la route diverger sans que rien ne le signale — le contrat annoncerait
// une adresse que le serveur ne sert pas.
#[utoipa::path(
    tag = "etablissements",
    params(
        ("etablissement_id" = Uuid, Path, description = "Identifiant de l'établissement"),
        PaginationParams,
    ),
    responses(
        (status = 200, description = "Notes de l'établissement", body = PageNotes),
        (status = 401, description = "Non authentifié"),
        (status = 403, description = "Permission absente"),
        (status = 404, description = "Établissement inconnu"),
    ),
    security(("bearer" = []))
)]
#[get("")]
pub async fn lister(
    etat: web::Data<EtatApplication>,
    contexte: ContexteAppel,
    chemin: web::Path<Uuid>,
    pagination: web::Query<PaginationParams>,
) -> Result<HttpResponse, actix_web::Error> {
    let etablissement_id = chemin.into_inner();
    let service = etat.service_note();

    let (elements, total) = service
        .lister(
            contexte.tenant_id,
            etablissement_id,
            pagination.limite(),
            pagination.decalage(),
        )
        .await
        .map_err(en_reponse)?;

    Ok(HttpResponse::Ok().json(PageNotes {
        elements,
        total,
        limite: pagination.limite(),
        decalage: pagination.decalage(),
    }))
}

/// Crée une note interne.
///
/// **`200` sur rejeu, pas `409`** — et c'est un choix, pas un raccourci. Un client hors ligne qui
/// vide sa file ne doit pas voir d'erreur pour une écriture que le serveur a déjà acceptée : le
/// principe VI exige que le rejeu soit idempotent, et un `409` obligerait chaque appelant à
/// traiter un cas d'erreur qui n'en est pas un. Le corps renvoyé est la note **telle qu'elle est
/// en base** — le serveur fait foi en conflit.
#[utoipa::path(
    tag = "etablissements",
    params(("etablissement_id" = Uuid, Path, description = "Identifiant de l'établissement")),
    request_body = CreerNoteRequete,
    responses(
        (status = 201, description = "Note créée", body = NoteEtablissement),
        (status = 200, description = "Note déjà créée (rejeu idempotent)", body = NoteEtablissement),
        (status = 400, description = "Requête invalide"),
        (status = 401, description = "Non authentifié"),
        (status = 403, description = "Permission absente"),
        (status = 404, description = "Établissement inconnu"),
    ),
    security(("bearer" = []))
)]
#[post("")]
pub async fn creer(
    etat: web::Data<EtatApplication>,
    contexte: ContexteAppel,
    chemin: web::Path<Uuid>,
    corps: web::Json<CreerNoteRequete>,
) -> Result<HttpResponse, actix_web::Error> {
    let etablissement_id = chemin.into_inner();
    let corps = corps.into_inner();
    let service = etat.service_note();

    let (note, issue) = service
        .creer(
            contexte.tenant_id,
            CreerNote {
                id: corps.id,
                etablissement_id,
                auteur_compte_id: contexte.compte_id,
                texte: corps.texte,
                horodatage_client: corps.horodatage_client,
            },
        )
        .await
        .map_err(en_reponse)?;

    Ok(match issue {
        Issue::Creee => HttpResponse::Created().json(note),
        Issue::DejaPresente => HttpResponse::Ok().json(note),
    })
}

/// Traduit une erreur de domaine en réponse HTTP.
///
/// **Aucun détail interne ne franchit la frontière** : ni message de PostgreSQL, ni nom de table,
/// ni chaîne de connexion. Le détail part dans les journaux, corrélé par l'identifiant de
/// requête, où le support peut le retrouver.
fn en_reponse(erreur: ErreurNote) -> actix_web::Error {
    match erreur {
        ErreurNote::TexteInvalide => actix_web::error::ErrorBadRequest(
            "texte invalide : entre 1 et 2000 caractères après nettoyage",
        ),
        ErreurNote::EtablissementInconnu => {
            actix_web::error::ErrorNotFound("établissement inconnu")
        }
        autre => {
            tracing::error!(erreur = %autre, "échec du service des notes");
            actix_web::error::ErrorInternalServerError("erreur interne")
        }
    }
}
