//! Handler du registre des actions — **CPT-04**, opération 19 du contrat.
//!
//! # Une seule opération, et c'est une décision (research R-17)
//!
//! **Aucun point d'entrée d'écriture.** Au MVP en mode A, une entrée voyage toujours avec
//! l'opération qu'elle trace, dans sa transaction. En livrer un produirait deux choses : une
//! **cible vide**, puisque rien ne l'appellerait — et une porte P-08 verte sur un endpoint que
//! personne n'exerce —, et surtout **une surface par laquelle un terminal forgerait des entrées
//! dans le registre censé le surveiller**.
//!
//! **Ni export, ni alertes** : DIR-04, tranche T5 (FR-040). La frontière est écrite deux fois dans
//! la spec, et une troisième ici.
//!
//! # L'auteur est dénormalisé à la LECTURE, jamais à l'écriture
//!
//! `journal_audit` porte `auteur_compte_id` et rien d'autre. Y figer le nom au moment de
//! l'écriture donnerait un registre à rétention illimitée où le nom serait faux après un mariage ;
//! le joindre en SQL serait une jointure intra-schéma acceptable ici, mais ne le serait plus le
//! jour où l'auteur d'une remise viendra d'un autre module. Le trait `AnnuaireComptes` le résout
//! **en lot** : une requête pour une page de cent entrées, pas cent.
//!
//! # L'horodatage d'autorité est celui qui s'affiche
//!
//! `cree_le` est posé par la base. `horodatage_client` est rendu **à part**, jamais présenté comme
//! la date de l'action : un téléphone en avance de deux heures ferait mentir le registre qui sert
//! à prouver ce qui s'est passé (principe IV).

use actix_web::{HttpResponse, get, web};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use kaya_comptes::AnnuaireComptes;
use kaya_comptes::audit::{Curseur, ErreurAudit, FiltresAudit, LIMITE_DEFAUT, TypeActionAudit};

use crate::application::EtatApplication;
use crate::contexte::ContexteAppel;
use crate::routes::erreurs::{CorpsErreur, interne};
use crate::securite;

/// La permission qui ouvre le registre.
const PERM_CONSULTER: &str = "cpt.audit.consulter";

/// Les cinq filtres du contrat — **tous combinables, tous optionnels** (FR-037).
#[derive(Debug, Deserialize, IntoParams)]
pub struct FiltresJournal {
    /// Qui a agi.
    #[serde(default)]
    pub auteur_compte_id: Option<Uuid>,
    #[serde(default)]
    pub etablissement_id: Option<Uuid>,
    /// L'une des dix familles de la taxonomie.
    #[serde(default)]
    pub type_action: Option<String>,
    /// Borne **inclusive** de début, sur l'horodatage d'autorité.
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub depuis: Option<OffsetDateTime>,
    /// Borne **exclusive** de fin — une journée se demande `[J, J+1)`.
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub jusqu_a: Option<OffsetDateTime>,
    /// Curseur de page suivante — l'horodatage de la dernière entrée reçue.
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub apres_cree_le: Option<OffsetDateTime>,
    /// Curseur de page suivante — l'identifiant de la dernière entrée reçue.
    #[serde(default)]
    pub apres_id: Option<Uuid>,
    #[serde(default)]
    pub limite: Option<i64>,
}

/// L'auteur d'une entrée, tel que l'écran l'affiche.
///
/// **Ni identifiant de connexion, ni condensat.** `nom` vient de `personne` ; afficher un numéro de
/// téléphone dans un registre à rétention illimitée diffuserait un contact personnel.
#[derive(Debug, Serialize, ToSchema)]
pub struct AuteurVue {
    pub compte_id: Uuid,
    /// Absent si le compte n'est plus lisible — jamais un identifiant en repli.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nom: Option<String>,
}

/// Une entrée du registre, telle que l'API la rend.
#[derive(Debug, Serialize, ToSchema)]
pub struct EntreeJournalVue {
    pub id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub etablissement_id: Option<Uuid>,
    pub type_action: TypeActionAudit,
    pub auteur: AuteurVue,
    pub cible_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cible_id: Option<Uuid>,
    pub contexte: serde_json::Value,
    /// Indicatif — rendu **à part**, et jamais présenté comme la date de l'action.
    #[serde(skip_serializing_if = "Option::is_none", with = "time::serde::rfc3339::option")]
    pub horodatage_client: Option<OffsetDateTime>,
    /// Horodatage d'**autorité serveur**. C'est celui que l'écran `G4` affiche.
    #[serde(with = "time::serde::rfc3339")]
    pub cree_le: OffsetDateTime,
}

/// Une page du registre.
#[derive(Debug, Serialize, ToSchema)]
pub struct PageJournalVue {
    pub elements: Vec<EntreeJournalVue>,
    /// Curseur de la page suivante — **absent quand il n'y a pas de suite**.
    ///
    /// Deux champs plutôt qu'une chaîne opaque : un curseur encodé demanderait un décodeur, donc
    /// une surface de plus, pour cacher deux valeurs qui figurent déjà dans la page rendue.
    #[serde(skip_serializing_if = "Option::is_none", with = "time::serde::rfc3339::option")]
    pub suivant_cree_le: Option<OffsetDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suivant_id: Option<Uuid>,
}

/// Liste le registre des actions — **filtres combinables, pagination par curseur**.
#[utoipa::path(
    operation_id = "journal_audit_lister",
    tag = "audit",
    params(FiltresJournal),
    responses(
        (status = 200, description = "Une page du registre", body = PageJournalVue),
        (status = 400, description = "Type d'action inconnu", body = CorpsErreur),
        (status = 401, description = "Non authentifié"),
        (status = 403, description = "Permission absente", body = CorpsErreur),
    ),
    security(("bearer" = []))
)]
#[get("")]
pub async fn lister(
    etat: web::Data<EtatApplication>,
    contexte: ContexteAppel,
    filtres: web::Query<FiltresJournal>,
) -> Result<HttpResponse, actix_web::Error> {
    securite::exiger(&contexte, PERM_CONSULTER)?;
    let filtres = filtres.into_inner();

    // Un type d'action inconnu est un `400`, pas une page vide. Une page vide laisserait croire
    // qu'il ne s'est rien passé, ce qui est le pire message qu'un registre puisse donner.
    let type_action = match filtres.type_action.as_deref() {
        Some(code) => Some(TypeActionAudit::depuis_code(code).ok_or_else(|| {
            CorpsErreur::nouveau(
                "type_action_inconnu",
                Some(code.to_owned()),
                format!("« {code} » n'est pas une famille de la taxonomie d'audit"),
            )
            .en_400()
        })?),
        None => None,
    };

    // Le curseur n'est complet que si ses **deux** parties sont là : une moitié de curseur
    // paginerait sur `cree_le` seul, qui n'est pas un ordre total — deux entrées de la même
    // transaction partagent la microseconde, et l'une des deux serait sautée.
    let curseur = match (filtres.apres_cree_le, filtres.apres_id) {
        (Some(cree_le), Some(id)) => Some(Curseur { cree_le, id }),
        _ => None,
    };

    let page = etat
        .lire_journal_audit(
            contexte.tenant_id,
            &FiltresAudit {
                auteur_compte_id: filtres.auteur_compte_id,
                etablissement_id: filtres.etablissement_id,
                type_action,
                depuis: filtres.depuis,
                jusqu_a: filtres.jusqu_a,
            },
            curseur,
            filtres.limite.unwrap_or(LIMITE_DEFAUT),
        )
        .await
        .map_err(en_reponse)?;

    // **Résolution des auteurs EN LOT** — une requête pour la page, pas une par entrée.
    let auteurs = etat
        .annuaire_comptes()
        .comptes(
            contexte.tenant_id,
            &kaya_comptes::audit::repository::auteurs(&page),
        )
        .await
        .map_err(|erreur| interne("résolution des auteurs du registre", erreur))?;

    let elements = page
        .elements
        .into_iter()
        .map(|entree| EntreeJournalVue {
            auteur: AuteurVue {
                compte_id: entree.auteur_compte_id,
                // Un auteur illisible rend `None`, **jamais son identifiant en repli** : un UUID
                // affiché à la place d'un nom est un identifiant technique sous les yeux de
                // l'exploitant, et l'interface saura dire « compte supprimé » dans sa langue.
                nom: auteurs.get(&entree.auteur_compte_id).map(|c| c.nom_affichage.clone()),
            },
            id: entree.id,
            etablissement_id: entree.etablissement_id,
            type_action: entree.type_action,
            cible_type: entree.cible_type,
            cible_id: entree.cible_id,
            contexte: entree.contexte,
            horodatage_client: entree.horodatage_client,
            cree_le: entree.cree_le,
        })
        .collect();

    Ok(HttpResponse::Ok().json(PageJournalVue {
        elements,
        suivant_cree_le: page.suivant.map(|c| c.cree_le),
        suivant_id: page.suivant.map(|c| c.id),
    }))
}

fn en_reponse(erreur: ErreurAudit) -> actix_web::Error {
    interne("lecture du registre des actions", erreur)
}
