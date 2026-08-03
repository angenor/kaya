//! Handlers du séjour — **SEJ-02**, opérations 7 à 12 du contrat du cycle 006.
//!
//! # ★ L'opération 5 est ici, et pas dans `clients.rs`
//!
//! `GET /api/v1/clients/{client_id}/sejours` — l'historique des séjours — **paraît** appartenir au
//! client. Elle lit `hebergement.sejour` et se monte donc sur le crate `hebergement`.
//!
//! Si `socle/comptes` la servait, ce serait **deux violations d'un coup** : jointure inter-schémas
//! (**P-04**) *et* arête `socle/ → verticales/` (**P-03**). Le chemin HTTP cache ce découpage à
//! l'appelant, et c'est normal — **le contrat est une façade, pas une carte des crates.**
//!
//! # Les deux `409` ne se confondent pas
//!
//! - Même `id` sur une ouverture → **`200`**, avec la ligne en base. Un terminal qui vide sa file
//!   ne doit pas voir d'erreur pour une écriture déjà acceptée.
//! - `id` différent sur un intervalle chevauchant → **`409 unite_deja_occupee`**, et le refus vient
//!   de la **contrainte d'exclusion**, jamais d'une vérification préalable.
//!
//! C'est la distinction posée au cycle 004, reprise telle quelle.

use actix_web::{HttpResponse, delete, get, post, web};
use serde::Deserialize;
use time::{Date, OffsetDateTime};
use utoipa::ToSchema;
use uuid::Uuid;

use kaya_hebergement::Issue;
use kaya_hebergement::erreurs::ErreurSejour;
use kaya_hebergement::sejour::{
    Accompagnant, FichePolice, IssueAccompagnant, NouvelAccompagnant, OuvrirSejour, Sejour,
    SejourOuvert, SejourVue,
};

use crate::application::EtatApplication;
use crate::contexte::ContexteAppel;
use crate::routes::erreurs::{CorpsErreur, interne};
use crate::securite;

/// Permissions du contrat, nommées une fois.
const PERM_LIRE: &str = "heb.sejour.lire";
const PERM_OUVRIR: &str = "heb.sejour.ouvrir";
/// Permission **transversale** de la fiche client — l'historique en exige **deux**.
const PERM_CLIENT_LIRE: &str = "sej.client.lire";

/// Nombre maximal de séjours rendus par l'historique d'un client.
///
/// Cinquante, parce que la fiche `R5` en montre une liste défilante et qu'au-delà personne ne
/// remonte. **Nommée plutôt que littérale** : ce n'est pas un paramètre d'établissement (aucune
/// story du périmètre ne dit « paramétrable », principe I·c), mais sa révision doit être trouvable.
const HISTORIQUE_MAX: i64 = 50;

// =================================================================================================
//  Corps de requête
// =================================================================================================

/// Corps d'ouverture d'un séjour — **l'opération du cycle**.
#[derive(Debug, Deserialize, ToSchema)]
pub struct OuvrirSejourRequete {
    /// UUID v7 **généré par le client** (FR-086) : c'est lui qui rend le rejeu inoffensif. Le
    /// serveur déduplique, il n'engendre pas.
    pub id: Uuid,
    /// Choisie par l'opérateur — **un tap sur `R4`**.
    pub unite_id: Uuid,
    pub formule_id: Uuid,
    /// RFC 3339. Pour un passage, calculés depuis la durée touchée.
    #[serde(with = "time::serde::rfc3339")]
    pub debut_client: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub fin_client: OffsetDateTime,
    /// **ABSENT pour un passage** : la pièce d'identité vient après la clé (maquette `R4`,
    /// FR-023). Un séjour sans fiche est un séjour valide.
    #[serde(default)]
    pub client_id: Option<Uuid>,
    /// Ajoutés dans la **même transaction** que le séjour : un accompagnant déclaré à l'arrivée et
    /// perdu par un second appel manqué ferait une fiche de police fausse.
    #[serde(default)]
    pub accompagnants: Vec<AccompagnantRequete>,
    /// Indicatif — **jamais employé par une règle métier** (porte P-23).
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub horodatage_client: Option<OffsetDateTime>,
}

/// Un accompagnant à ajouter — **un nom suffit** (FR-015).
///
/// Demander une pièce par accompagnant coûterait la cible des 60 secondes de l'arrivée.
#[derive(Debug, Deserialize, ToSchema)]
pub struct AccompagnantRequete {
    pub id: Uuid,
    pub nom: String,
    #[serde(default)]
    pub prenoms: Option<String>,
    #[serde(default)]
    pub date_naissance: Option<Date>,
    #[serde(default)]
    pub nationalite: Option<String>,
    #[serde(default)]
    pub type_piece: Option<String>,
    /// ⚠️ **Seconde surface de rétention du produit, et elle est assumée** : un accompagnant n'a
    /// pas de fiche client. La purge de 90 jours de TRX-06 portera sur **deux** tables.
    #[serde(default)]
    pub numero_piece: Option<String>,
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub horodatage_client: Option<OffsetDateTime>,
}

/// Corps de rattachement d'un client à un séjour déjà ouvert.
#[derive(Debug, Deserialize, ToSchema)]
pub struct RattacherClientRequete {
    pub client_id: Uuid,
}

/// Filtre de la liste des séjours.
#[derive(Debug, Deserialize, ToSchema, utoipa::IntoParams)]
pub struct ListerSejoursRequete {
    /// `true` par défaut — l'écran de départ ne montre que ce qui est en cours.
    #[serde(default = "vrai")]
    pub en_cours: bool,
}

fn vrai() -> bool {
    true
}

/// Réponse d'une écriture partie en **file de réconciliation** — `202`.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct EcritureOrpheline {
    /// Code stable, traduit par le lexique : « Cette information est arrivée après le départ du
    /// client. » suivie de « Le gérant décidera de la suite. »
    pub motif: String,
    pub reconciliation_id: Uuid,
}

// =================================================================================================
//  ★ 7 · POST .../sejours — l'opération du cycle
// =================================================================================================

/// Ouvre un séjour — **un appel, une transaction, cinq écritures**.
///
/// C'est ce qui tient le budget de FR-031 : au plus **un** appel réseau bloquant entre le premier
/// geste et la confirmation.
///
/// ⚠️ **`409 unite_deja_occupee` vient de la contrainte d'exclusion**, jamais d'une vérification
/// préalable. Un `SELECT … FOR UPDATE` donnerait le même code en rendant la double attribution
/// *improbable* au lieu d'*impossible*.
#[utoipa::path(
    operation_id = "sejour_ouvrir",
    tag = "sejours",
    params(("etablissement_id" = Uuid, Path, description = "Établissement")),
    request_body = OuvrirSejourRequete,
    responses(
        (status = 201, description = "Séjour ouvert — le séjour, l'occupation, la note et son total, la fiche de police", body = SejourOuvert),
        (status = 200, description = "Rejeu du même id — la ligne telle qu'elle est en base", body = SejourOuvert),
        (status = 401, description = "Non authentifié"),
        (status = 403, description = "Permission absente", body = CorpsErreur),
        (status = 404, description = "Établissement, unité, formule ou client inconnus", body = CorpsErreur),
        (status = 409, description = "unite_deja_occupee — le refus vient de la CONTRAINTE · service_inactif", body = CorpsErreur),
        (status = 422, description = "intervalle_invalide, duree_hors_contrainte, formule_hors_categorie, plage_non_fractionnable", body = CorpsErreur),
    ),
    security(("bearer" = []))
)]
#[post("")]
pub async fn ouvrir(
    etat: web::Data<EtatApplication>,
    contexte: ContexteAppel,
    chemin: web::Path<Uuid>,
    corps: web::Json<OuvrirSejourRequete>,
) -> Result<HttpResponse, actix_web::Error> {
    securite::exiger(&contexte, PERM_OUVRIR)?;
    let etablissement_id = chemin.into_inner();
    let corps = corps.into_inner();

    let (ouvert, issue) = etat
        .service_sejour(contexte.tenant_id)
        .ouvrir(OuvrirSejour {
            id: corps.id,
            etablissement_id,
            unite_id: corps.unite_id,
            formule_id: corps.formule_id,
            debut_client: corps.debut_client,
            fin_client: corps.fin_client,
            client_id: corps.client_id,
            accompagnants: corps.accompagnants.into_iter().map(en_nouvel).collect(),
            horodatage_client: corps.horodatage_client,
        })
        .await
        .map_err(en_reponse)?;

    Ok(match issue {
        Issue::Creee => HttpResponse::Created().json(ouvert),
        Issue::DejaPresente => HttpResponse::Ok().json(ouvert),
    })
}

// =================================================================================================
//  8 · GET .../sejours — la liste
// =================================================================================================

/// Liste les séjours d'un établissement, **avec le nom de leur client**.
///
/// ★ Les noms sont résolus **par lot**, en un seul appel au trait `AnnuaireClients` — jamais par
/// jointure (porte P-04), et jamais un par un : une résolution unitaire produirait N+1 requêtes,
/// et c'est le détail qui décide si l'écran de départ s'ouvre en 200 ms ou en deux secondes.
#[utoipa::path(
    operation_id = "sejour_lister",
    tag = "sejours",
    params(("etablissement_id" = Uuid, Path, description = "Établissement"), ListerSejoursRequete),
    responses(
        (status = 200, description = "Les séjours, avec nom du client, personnes, unité et total", body = Vec<SejourVue>),
        (status = 401, description = "Non authentifié"),
        (status = 403, description = "Permission absente", body = CorpsErreur),
        (status = 404, description = "Établissement inconnu", body = CorpsErreur),
    ),
    security(("bearer" = []))
)]
#[get("")]
pub async fn lister(
    etat: web::Data<EtatApplication>,
    contexte: ContexteAppel,
    chemin: web::Path<Uuid>,
    filtre: web::Query<ListerSejoursRequete>,
) -> Result<HttpResponse, actix_web::Error> {
    securite::exiger(&contexte, PERM_LIRE)?;

    let liste = etat
        .service_sejour(contexte.tenant_id)
        .lister(chemin.into_inner(), filtre.en_cours)
        .await
        .map_err(en_reponse)?;

    Ok(HttpResponse::Ok().json(liste))
}

// =================================================================================================
//  9 · GET .../sejours/{sejour_id}
// =================================================================================================

/// Lit un séjour complet — le séjour, l'occupation, la note et son total, la fiche de police.
#[utoipa::path(
    operation_id = "sejour_lire",
    tag = "sejours",
    params(
        ("etablissement_id" = Uuid, Path, description = "Établissement"),
        ("sejour_id" = Uuid, Path, description = "Séjour"),
    ),
    responses(
        (status = 200, description = "Le séjour complet", body = SejourOuvert),
        (status = 401, description = "Non authentifié"),
        (status = 403, description = "Permission absente", body = CorpsErreur),
        (status = 404, description = "Séjour inconnu", body = CorpsErreur),
    ),
    security(("bearer" = []))
)]
#[get("")]
pub async fn lire(
    etat: web::Data<EtatApplication>,
    contexte: ContexteAppel,
    chemin: web::Path<(Uuid, Uuid)>,
) -> Result<HttpResponse, actix_web::Error> {
    securite::exiger(&contexte, PERM_LIRE)?;
    let (_etablissement_id, sejour_id) = chemin.into_inner();

    let sejour = etat
        .service_sejour(contexte.tenant_id)
        .lire(sejour_id)
        .await
        .map_err(en_reponse)?
        .ok_or_else(|| {
            CorpsErreur::nouveau(
                "sejour_inconnu",
                None,
                "aucun séjour de cet identifiant dans ce tenant".to_owned(),
            )
            .en_404()
        })?;

    Ok(HttpResponse::Ok().json(sejour))
}

// =================================================================================================
//  10 · POST .../sejours/{sejour_id}/client — la pièce APRÈS la clé
// =================================================================================================

/// Rattache une fiche client à un séjour déjà ouvert.
///
/// **Ne rouvre pas le séjour et ne remet pas en cause l'attribution** (FR-028). C'est le parcours
/// normal du passage : la pièce vient **après** la clé (FR-023). La fiche de police passe à
/// « complète » dans la même transaction.
#[utoipa::path(
    operation_id = "sejour_rattacher_client",
    tag = "sejours",
    params(
        ("etablissement_id" = Uuid, Path, description = "Établissement"),
        ("sejour_id" = Uuid, Path, description = "Séjour"),
    ),
    request_body = RattacherClientRequete,
    responses(
        (status = 200, description = "Client rattaché — la fiche de police devient complète", body = Sejour),
        (status = 401, description = "Non authentifié"),
        (status = 403, description = "Permission absente", body = CorpsErreur),
        (status = 404, description = "Séjour ou client inconnus", body = CorpsErreur),
    ),
    security(("bearer" = []))
)]
#[post("")]
pub async fn rattacher_client(
    etat: web::Data<EtatApplication>,
    contexte: ContexteAppel,
    chemin: web::Path<(Uuid, Uuid)>,
    corps: web::Json<RattacherClientRequete>,
) -> Result<HttpResponse, actix_web::Error> {
    securite::exiger(&contexte, PERM_OUVRIR)?;
    let (etablissement_id, sejour_id) = chemin.into_inner();

    let sejour = etat
        .service_sejour(contexte.tenant_id)
        .rattacher_client(etablissement_id, sejour_id, corps.into_inner().client_id)
        .await
        .map_err(en_reponse)?;

    Ok(HttpResponse::Ok().json(sejour))
}

// =================================================================================================
//  ★ 11 · POST .../accompagnants — et le cas orphelin
// =================================================================================================

/// Ajoute un accompagnant — **classe A**, la seule écriture de séjour atteignable hors ligne.
///
/// ★ **Trois codes, et le troisième est celui du principe VI.**
///
/// Sur un séjour **clos**, l'écriture n'est ni acceptée (`201` serait un ajout d'office) ni
/// rejetée (`409` serait un rejet silencieux) : elle part en **file de réconciliation** avec son
/// motif et sa charge utile, et rend **`202`**. C'est le premier cas réel d'écriture orpheline du
/// produit.
#[utoipa::path(
    operation_id = "sejour_accompagnant_ajouter",
    tag = "sejours",
    params(
        ("etablissement_id" = Uuid, Path, description = "Établissement"),
        ("sejour_id" = Uuid, Path, description = "Séjour"),
    ),
    request_body = AccompagnantRequete,
    responses(
        (status = 201, description = "Ajouté à un séjour ouvert", body = Accompagnant),
        (status = 200, description = "Rejeu du même id — aucun second événement outbox", body = Accompagnant),
        (status = 202, description = "★ Le séjour est CLOS : l'écriture part en file de réconciliation. Ni 201 (ajout d'office), ni 409 (rejet silencieux)", body = EcritureOrpheline),
        (status = 401, description = "Non authentifié"),
        (status = 403, description = "Permission absente", body = CorpsErreur),
        (status = 404, description = "Séjour inconnu", body = CorpsErreur),
    ),
    security(("bearer" = []))
)]
#[post("")]
pub async fn ajouter_accompagnant(
    etat: web::Data<EtatApplication>,
    contexte: ContexteAppel,
    chemin: web::Path<(Uuid, Uuid)>,
    corps: web::Json<AccompagnantRequete>,
) -> Result<HttpResponse, actix_web::Error> {
    securite::exiger(&contexte, PERM_OUVRIR)?;
    let (etablissement_id, sejour_id) = chemin.into_inner();

    let issue = etat
        .service_sejour(contexte.tenant_id)
        .ajouter_accompagnant(etablissement_id, sejour_id, en_nouvel(corps.into_inner()))
        .await
        .map_err(en_reponse)?;

    Ok(match issue {
        IssueAccompagnant::Ajoute(a) => HttpResponse::Created().json(a),
        IssueAccompagnant::Rejeu(a) => HttpResponse::Ok().json(a),
        IssueAccompagnant::Orphelin { reconciliation_id } => {
            HttpResponse::Accepted().json(EcritureOrpheline {
                motif: "sejour_clos".to_owned(),
                reconciliation_id,
            })
        }
    })
}

// =================================================================================================
//  12 · DELETE .../accompagnants/{accompagnant_id}
// =================================================================================================

/// Retire un accompagnant — **`retire_le`, jamais un `DELETE`**.
///
/// Sans cela, la fiche de police perdrait la trace d'une personne qui a bien été déclarée, et un
/// registre légal qui perd une déclaration est un document faux devant la gendarmerie. Le verbe
/// HTTP est `DELETE` parce que c'est le geste de l'utilisateur ; la base, elle, ne supprime rien.
#[utoipa::path(
    operation_id = "sejour_accompagnant_retirer",
    tag = "sejours",
    params(
        ("etablissement_id" = Uuid, Path, description = "Établissement"),
        ("sejour_id" = Uuid, Path, description = "Séjour"),
        ("accompagnant_id" = Uuid, Path, description = "Accompagnant"),
    ),
    responses(
        (status = 200, description = "Retiré — la ligne porte retire_le, elle n'est pas supprimée", body = Accompagnant),
        (status = 401, description = "Non authentifié"),
        (status = 403, description = "Permission absente", body = CorpsErreur),
        (status = 404, description = "Accompagnant inconnu", body = CorpsErreur),
    ),
    security(("bearer" = []))
)]
#[delete("")]
pub async fn retirer_accompagnant(
    etat: web::Data<EtatApplication>,
    contexte: ContexteAppel,
    chemin: web::Path<(Uuid, Uuid, Uuid)>,
) -> Result<HttpResponse, actix_web::Error> {
    securite::exiger(&contexte, PERM_OUVRIR)?;
    let (etablissement_id, _sejour_id, accompagnant_id) = chemin.into_inner();

    let accompagnant = etat
        .service_sejour(contexte.tenant_id)
        .retirer_accompagnant(etablissement_id, accompagnant_id)
        .await
        .map_err(en_reponse)?;

    Ok(HttpResponse::Ok().json(accompagnant))
}

// =================================================================================================
//  16 · GET .../sejours/{sejour_id}/fiche-police
// =================================================================================================

/// Lit la fiche de police d'un séjour.
///
/// Elle porte la mention obligatoire « **Document non fiscal — ne tient pas lieu de facture** » à
/// l'affichage : c'est un **document opérationnel** au sens de FIS-02, et le principe V l'exige de
/// tous (FR-048).
///
/// ⚠️ **Elle ne porte aucune identité recopiée** : les noms viennent du client et des
/// accompagnants. Recopier créerait une troisième surface de rétention pour la même donnée.
#[utoipa::path(
    operation_id = "sejour_fiche_police_lire",
    tag = "sejours",
    params(
        ("etablissement_id" = Uuid, Path, description = "Établissement"),
        ("sejour_id" = Uuid, Path, description = "Séjour"),
    ),
    responses(
        (status = 200, description = "La fiche de police, avec son numéro et sa complétude", body = FichePolice),
        (status = 401, description = "Non authentifié"),
        (status = 403, description = "Permission absente", body = CorpsErreur),
        (status = 404, description = "Séjour inconnu", body = CorpsErreur),
    ),
    security(("bearer" = []))
)]
#[get("")]
pub async fn lire_fiche_police(
    etat: web::Data<EtatApplication>,
    contexte: ContexteAppel,
    chemin: web::Path<(Uuid, Uuid)>,
) -> Result<HttpResponse, actix_web::Error> {
    securite::exiger(&contexte, PERM_LIRE)?;
    let (_etablissement_id, sejour_id) = chemin.into_inner();

    let sejour = etat
        .service_sejour(contexte.tenant_id)
        .lire(sejour_id)
        .await
        .map_err(en_reponse)?
        .ok_or_else(|| {
            CorpsErreur::nouveau(
                "sejour_inconnu",
                None,
                "aucun séjour de cet identifiant dans ce tenant".to_owned(),
            )
            .en_404()
        })?;

    Ok(HttpResponse::Ok().json(sejour.fiche_police))
}

// =================================================================================================
//  ★ 5 · GET /api/v1/clients/{client_id}/sejours — l'historique
// =================================================================================================

/// L'historique des séjours d'un client, **du plus récent au plus ancien**.
///
/// ★ **Elle est ici, et non dans `clients.rs`, pour deux raisons opposables** : elle lit
/// `hebergement.sejour`, et elle se monte sur le crate `hebergement`. Si `socle/comptes` la
/// servait, ce serait une jointure inter-schémas (**P-04**) *et* une arête `socle/ → verticales/`
/// (**P-03**) — deux violations d'un coup.
///
/// **Double garde de permission** : `sej.client.lire` **et** `heb.sejour.lire`. La première dit
/// qu'on a le droit de savoir qui est ce client, la seconde qu'on a le droit de savoir ce qu'il a
/// consommé. Un rôle qui n'aurait que la première verrait une fiche sans historique — ce qui est
/// exactement le comportement voulu pour un compte de portée restreinte.
#[utoipa::path(
    operation_id = "client_historique_sejours",
    tag = "sejours",
    params(("client_id" = Uuid, Path, description = "Fiche client")),
    responses(
        (status = 200, description = "Les séjours du client, tous établissements du tenant confondus", body = Vec<SejourVue>),
        (status = 401, description = "Non authentifié"),
        (status = 403, description = "Permission absente — les DEUX sont exigées", body = CorpsErreur),
    ),
    security(("bearer" = []))
)]
#[get("")]
pub async fn historique_du_client(
    etat: web::Data<EtatApplication>,
    contexte: ContexteAppel,
    chemin: web::Path<Uuid>,
) -> Result<HttpResponse, actix_web::Error> {
    securite::exiger(&contexte, PERM_CLIENT_LIRE)?;
    securite::exiger(&contexte, PERM_LIRE)?;

    let liste = etat
        .service_sejour(contexte.tenant_id)
        .historique_du_client(chemin.into_inner(), HISTORIQUE_MAX)
        .await
        .map_err(en_reponse)?;

    Ok(HttpResponse::Ok().json(liste))
}

// =================================================================================================
//  Fonctions internes
// =================================================================================================

fn en_nouvel(requete: AccompagnantRequete) -> NouvelAccompagnant {
    NouvelAccompagnant {
        id: requete.id,
        nom: requete.nom,
        prenoms: requete.prenoms,
        date_naissance: requete.date_naissance,
        nationalite: requete.nationalite,
        type_piece: requete.type_piece,
        numero_piece: requete.numero_piece,
        horodatage_client: requete.horodatage_client,
    }
}

/// Traduit un refus de domaine en réponse HTTP.
///
/// **Le code est stable et l'interface y branche sa clé i18n** — jamais sur le message, qui nomme
/// des tables et parle anglais technique (règle du cycle 002). Les six refus du cycle sont au
/// lexique v1.6.0.
fn en_reponse(erreur: ErreurSejour) -> actix_web::Error {
    let code = erreur.code();
    let corps = |message: &str| CorpsErreur::nouveau(code, None, message.to_owned());

    match erreur {
        ErreurSejour::SejourDejaClos => corps("ce séjour est déjà terminé").en_409(),
        ErreurSejour::SejourClos => corps("on ne prolonge pas un séjour terminé").en_409(),
        ErreurSejour::ConflitOccupationSuivante => {
            corps("cette chambre est réservée à partir d'un instant donné").en_409()
        }
        ErreurSejour::UniteCibleOccupee => {
            corps("cette chambre n'est pas libre sur la période restante").en_409()
        }
        ErreurSejour::BasculeFormuleNonConfirmee => {
            corps("le franchissement du seuil change le tarif : à confirmer").en_422()
        }
        ErreurSejour::SejourInconnu | ErreurSejour::NoteInconnue => {
            corps("aucun séjour de cet identifiant dans ce tenant").en_404()
        }
        ErreurSejour::AccompagnantInconnu => {
            corps("aucun accompagnant de cet identifiant").en_404()
        }
        ErreurSejour::ClientInconnu => {
            corps("aucune fiche client de cet identifiant dans ce tenant").en_404()
        }
        ErreurSejour::EtablissementInconnu => corps("établissement inconnu").en_404(),
        ErreurSejour::ServiceInactif => {
            corps("le service d'hébergement n'est pas actif sur cet établissement").en_409()
        }
        ErreurSejour::Attribution(ref e) => match e {
            kaya_hebergement::occupation::ErreurAttribution::UniteDejaOccupee => {
                corps("cette chambre est déjà prise sur cette période").en_409()
            }
            kaya_hebergement::occupation::ErreurAttribution::UniteInconnue
            | kaya_hebergement::occupation::ErreurAttribution::FormuleInconnue
            | kaya_hebergement::occupation::ErreurAttribution::OccupationInconnue => {
                corps("unité, formule ou occupation inconnue").en_404()
            }
            kaya_hebergement::occupation::ErreurAttribution::ServiceInactif => {
                corps("le service d'hébergement n'est pas actif sur cet établissement").en_409()
            }
            kaya_hebergement::occupation::ErreurAttribution::EtablissementInconnu => {
                corps("établissement inconnu").en_404()
            }
            kaya_hebergement::occupation::ErreurAttribution::FormuleHorsCategorie
            | kaya_hebergement::occupation::ErreurAttribution::PlageNonFractionnable
            | kaya_hebergement::occupation::ErreurAttribution::IntervalleInvalide
            | kaya_hebergement::occupation::ErreurAttribution::DureeHorsContrainte => {
                corps("la demande ne satisfait pas les contraintes de la formule").en_422()
            }
            _ => interne("service du séjour", erreur),
        },
        autre => interne("service du séjour", autre),
    }
}
