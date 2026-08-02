//! Handler de la tarification — **HEB-04**, opération 12.
//!
//! # L'appel ne prend AUCUN instant en paramètre
//!
//! C'est la décision de l'endpoint, et elle est structurelle : le serveur lit `cree_le` de
//! l'occupation et `now()`, tous deux en SQL. Un client ne peut donc pas influencer la durée
//! facturée, même avec une horloge décalée de quarante minutes.
//!
//! Le cadrage §11 désigne nommément le passage comme le cas où l'horloge coûte : sur une nuitée,
//! une heure d'écart ne change rien au montant ; sur un passage à 1 500 F l'heure, elle en change
//! un septième.
//!
//! # Le moteur calcule, il ne facture pas
//!
//! Aucune ligne de note n'est écrite — la note est SEJ-03, tranche T2. Ce que cette opération rend
//! est une **décision de tarification** que SEJ-03 consommera.

use actix_web::{HttpResponse, post, web};
use serde::Deserialize;
use uuid::Uuid;

use kaya_hebergement::occupation::ErreurAttribution;
use kaya_hebergement::traits::DecisionTarification;

use crate::application::EtatApplication;
use crate::contexte::ContexteAppel;
use crate::routes::erreurs::CorpsErreur;
use crate::securite::exiger;

/// La même permission que la consultation de disponibilité : c'est une **lecture chiffrée**, pas
/// une écriture. Yao la détient ; il en a besoin pour annoncer un montant au comptoir.
const CONSULTER: &str = "heb.disponibilite.consulter";

#[derive(Debug, Deserialize)]
pub struct CheminTarif {
    pub etablissement_id: Uuid,
    pub occupation_id: Uuid,
}

/// Calcule le montant dû pour une occupation.
#[utoipa::path(
    operation_id = "hebergement_calculer_tarif",
    tag = "hebergement",
    params(
        ("etablissement_id" = Uuid, Path, description = "Identifiant de l'établissement"),
        ("occupation_id" = Uuid, Path, description = "Identifiant de l'occupation"),
    ),
    responses(
        (status = 200, description = "Décision de tarification à l'instant d'autorité serveur", body = DecisionTarification),
        (status = 401, description = "Non authentifié"),
        (status = 403, description = "Permission absente", body = CorpsErreur),
        (status = 404, description = "Occupation inconnue", body = CorpsErreur),
        (status = 409, description = "Service hébergement non actif", body = CorpsErreur),
        (status = 422, description = "Barème absent sur la formule", body = CorpsErreur),
    ),
    security(("bearer" = []))
)]
#[post("")]
pub async fn calculer_tarif(
    etat: web::Data<EtatApplication>,
    contexte: ContexteAppel,
    chemin: web::Path<CheminTarif>,
) -> Result<HttpResponse, actix_web::Error> {
    exiger(&contexte, CONSULTER)?;
    let chemin = chemin.into_inner();

    let decision = etat
        .service_tarification(contexte.tenant_id)
        // `compte_id` de l'appelant : le registre des actions doit dire **qui** a constaté le
        // dépassement. Une entrée sans auteur ne répond pas à la question que le propriétaire pose.
        .calculer(chemin.etablissement_id, chemin.occupation_id, contexte.compte_id)
        .await
        .map_err(super::hebergement_disponibilite::en_reponse)?;

    Ok(HttpResponse::Ok().json(decision))
}

/// Réexport pour que le type entre au contrat OpenAPI — il vit dans le crate métier, où le
/// principe V l'a placé, et utoipa a besoin de le voir depuis ce module.
#[allow(unused_imports)]
pub use kaya_hebergement::traits::Rebascule;

/// Vérifie qu'une erreur d'attribution se traduit — le type est partagé avec la disponibilité,
/// et l'écrire ici garde la conversion visible depuis ce fichier.
#[allow(dead_code)]
fn traduire(erreur: ErreurAttribution) -> actix_web::Error {
    super::hebergement_disponibilite::en_reponse(erreur)
}
