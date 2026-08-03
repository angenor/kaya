//! Routes HTTP.
//!
//! Une seule fonction les monte, et c'est celle que montent aussi les tests d'intégration : la
//! porte P-08 est **paramétrée sur la liste des routes du contrat OpenAPI**, de sorte qu'un
//! endpoint ajouté sans régime d'isolation déclaré fasse échouer la porte au lieu de passer
//! inaperçu.
//!
//! `utoipa-actix-web` ne collecte les chemins que depuis les appels `service(...)` — jamais
//! depuis `route(...)`. Un endpoint monté par `route(...)` serait servi **sans figurer au
//! contrat**, donc absent du client généré et invisible pour P-08. C'est la raison pour laquelle
//! chaque handler porte son propre attribut de routage.

pub mod branding;
pub mod clients;
pub mod comptes;
pub mod configuration;
pub mod erreurs;
pub mod etablissements;
pub mod hebergement_disponibilite;
pub mod hebergement_referentiel;
pub mod hebergement_tarification;
pub mod journal_audit;
pub mod notes;
pub mod personnes;
pub mod points_de_vente;
pub mod referentiels;
pub mod sante;
pub mod services;
pub mod session;

use utoipa_actix_web::service_config::ServiceConfig;

/// Monte toutes les routes du produit.
///
/// # L'ordre de montage n'est pas décoratif
///
/// Actix essaie les services **dans l'ordre d'enregistrement**, et un scope qui accepte le
/// préfixe rend `404` sans laisser sa chance aux suivants. Les scopes les plus spécifiques sont
/// donc montés d'abord : `.../services/{module_code}/capacites` avant `.../services/{module_code}`,
/// avant `/api/v1/etablissements`. Monter le plus général en premier ferait disparaître les
/// autres — sans erreur de compilation, et avec un contrat OpenAPI parfaitement exact.
pub fn configurer(config: &mut ServiceConfig) {
    use utoipa_actix_web::scope::scope;

    // Sonde de santé — publique, sans contexte de tenant, hors de tout préfixe de version : la
    // supervision externe doit pouvoir l'interroger sans rien savoir du produit.
    config.service(sante::sante);

    // Identité visuelle — trois chemins distincts plutôt qu'un seul avec des sous-ressources :
    // `utoipa-actix-web` collecte un chemin par scope, et un aperçu qui n'enregistre rien n'a pas
    // le même régime qu'une écriture.
    config.service(scope("/api/v1/branding/logo").service(branding::televerser_logo));
    config.service(scope("/api/v1/branding/apercu").service(branding::apercu));
    config.service(
        scope("/api/v1/branding")
            .service(branding::resoudre)
            .service(branding::ecrire),
    );

    // Configuration héritée — la cible vient des paramètres de requête, jamais du chemin : une
    // même clé se résout depuis quatre niveaux différents, et quatre chemins distincts laisseraient
    // quatre implémentations diverger.
    config.service(
        scope("/api/v1/configuration")
            .service(configuration::resoudre)
            .service(configuration::ecrire),
    );

    // Référentiels — lecture seule, aucun verbe d'écriture exposé. Les cinq rendent **la même
    // chose aux deux tenants** : ce sont des référentiels globaux, et le test d'isolation
    // l'affirme explicitement plutôt que de laisser un référentiel global et une fuite se
    // ressembler.
    config.service(
        scope("/api/v1/referentiels")
            .service(referentiels::modules_activite)
            .service(referentiels::capacites)
            .service(referentiels::profils_stock)
            .service(referentiels::roles)
            .service(referentiels::permissions),
    );

    // Session — CPT-01. **Du plus spécifique au plus général**, sans quoi `/api/v1/session`
    // accepterait le préfixe et rendrait `404` pour les quatre autres — sans erreur de
    // compilation, et avec un contrat OpenAPI parfaitement exact.
    config.service(
        scope("/api/v1/session/actives/{session_id}").service(session::revoquer),
    );
    config.service(scope("/api/v1/session/actives").service(session::lister_actives));
    config.service(scope("/api/v1/session/rafraichir").service(session::rafraichir));
    config.service(scope("/api/v1/session/moi").service(session::moi));
    config.service(
        scope("/api/v1/session")
            .service(session::ouvrir)
            .service(session::fermer),
    );

    // Personnes — CPT-00. Le plus spécifique d'abord : `/personnes/{personne_id}` avant
    // `/personnes`. **Aucune liste** n'est montée, et c'est une décision (contrat §7-9) : un
    // annuaire d'identités civiles suppose la politique de rétention de TRX-06, qui n'existe pas.
    config.service(
        scope("/api/v1/personnes/{personne_id}")
            .service(personnes::lire)
            .service(personnes::modifier),
    );
    config.service(scope("/api/v1/personnes").service(personnes::creer));

    // Comptes et rôles — CPT-01, CPT-02. **Du plus spécifique au plus général**, sans quoi
    // `/api/v1/comptes` accepterait le préfixe et rendrait `404` pour les cinq autres — sans
    // erreur de compilation, et avec un contrat OpenAPI parfaitement exact.
    config.service(
        scope("/api/v1/comptes/{compte_id}/roles/{role_code}").service(comptes::retirer_role),
    );
    config.service(scope("/api/v1/comptes/{compte_id}/roles").service(comptes::attribuer_role));
    config.service(scope("/api/v1/comptes/{compte_id}/etat").service(comptes::changer_etat));
    config.service(
        scope("/api/v1/comptes/{compte_id}/mot-de-passe")
            .service(comptes::changer_mot_de_passe),
    );
    config.service(scope("/api/v1/comptes/{compte_id}").service(comptes::lire));
    config.service(
        scope("/api/v1/comptes")
            .service(comptes::creer)
            .service(comptes::lister),
    );

    // Fiches clients — SEJ-01. **Aucun `etablissement_id`** : la fiche est du TENANT (FR-002).
    // Du plus spécifique au plus général, sans quoi `/api/v1/clients` accepterait le préfixe et
    // rendrait `404` pour les préférences — sans erreur de compilation, et avec un contrat
    // OpenAPI parfaitement exact.
    //
    // ⚠️ **`/clients/{id}/sejours` n'est PAS ici** : l'historique est monté avec les séjours, sur
    // le crate `hebergement`. Si `socle/comptes` lisait `hebergement.sejour`, ce serait une
    // jointure inter-schémas (P-04) ET une arête `socle/ → verticales/` (P-03). Le chemin HTTP
    // cache ce découpage à l'appelant : le contrat est une façade, pas une carte des crates.
    config.service(
        scope("/api/v1/clients/{client_id}/preferences")
            .service(clients::enregistrer_preference),
    );
    config.service(
        scope("/api/v1/clients/{client_id}")
            .service(clients::lire)
            .service(clients::modifier),
    );
    config.service(
        scope("/api/v1/clients")
            .service(clients::rechercher)
            .service(clients::creer),
    );

    // Registre des actions — CPT-04. **Une seule opération, en lecture** : aucun point d'entrée
    // d'écriture (research R-17). Une entrée voyage toujours avec l'opération qu'elle trace.
    config.service(scope("/api/v1/journal-audit").service(journal_audit::lister));

    // Hébergement — HEB-01, HEB-03, HEB-04, HEB-05. **Du plus spécifique au plus général**,
    // sans quoi `.../hebergement/categories` accepterait le préfixe et rendrait `404` pour la
    // modification — sans erreur de compilation, et avec un contrat OpenAPI parfaitement exact.
    config.service(
        scope("/api/v1/etablissements/{etablissement_id}/hebergement/categories/{categorie_id}")
            .service(hebergement_referentiel::modifier_categorie),
    );
    config.service(
        scope("/api/v1/etablissements/{etablissement_id}/hebergement/categories")
            .service(hebergement_referentiel::lister_categories)
            .service(hebergement_referentiel::creer_categorie),
    );
    config.service(
        scope("/api/v1/etablissements/{etablissement_id}/hebergement/unites/{unite_id}")
            .service(hebergement_referentiel::modifier_unite),
    );
    config.service(
        scope("/api/v1/etablissements/{etablissement_id}/hebergement/unites")
            .service(hebergement_referentiel::lister_unites)
            .service(hebergement_referentiel::creer_unite),
    );
    config.service(
        scope("/api/v1/etablissements/{etablissement_id}/hebergement/formules/{formule_id}")
            .service(hebergement_referentiel::modifier_formule),
    );
    config.service(
        scope("/api/v1/etablissements/{etablissement_id}/hebergement/formules")
            .service(hebergement_referentiel::lister_formules)
            .service(hebergement_referentiel::creer_formule),
    );

    // Disponibilité et attribution — HEB-02, le cœur du cycle. Toujours du plus spécifique au
    // plus général : `.../occupations/{id}/liberation` avant `.../occupations`.
    config.service(
        scope("/api/v1/etablissements/{etablissement_id}/hebergement/occupations/{occupation_id}/liberation")
            .service(hebergement_disponibilite::liberer),
    );
    config.service(
        scope("/api/v1/etablissements/{etablissement_id}/hebergement/occupations/{occupation_id}/tarif")
            .service(hebergement_tarification::calculer_tarif),
    );
    config.service(
        scope("/api/v1/etablissements/{etablissement_id}/hebergement/occupations")
            .service(hebergement_disponibilite::attribuer),
    );
    config.service(
        scope("/api/v1/etablissements/{etablissement_id}/hebergement/disponibilite")
            .service(hebergement_disponibilite::consulter),
    );

    config.service(
        scope("/api/v1/etablissements/{etablissement_id}/notes")
            .service(notes::lister)
            .service(notes::creer),
    );

    config.service(
        scope("/api/v1/etablissements/{etablissement_id}/services/{module_code}/capacites")
            .service(services::lister_capacites)
            .service(services::declarer_capacite),
    );

    config.service(
        scope("/api/v1/etablissements/{etablissement_id}/services/{module_code}")
            .service(services::basculer),
    );

    config.service(
        scope("/api/v1/etablissements/{etablissement_id}/services").service(services::lister),
    );

    config.service(
        scope("/api/v1/etablissements/{etablissement_id}/points-de-vente")
            .service(points_de_vente::lister)
            .service(points_de_vente::creer),
    );

    config.service(
        scope("/api/v1/points-de-vente/{point_de_vente_id}/tables")
            .service(points_de_vente::remplacer_tables),
    );

    config.service(
        scope("/api/v1/points-de-vente/{point_de_vente_id}")
            .service(points_de_vente::modifier),
    );

    config.service(
        scope("/api/v1/etablissements")
            .service(etablissements::lister)
            .service(etablissements::creer)
            .service(etablissements::lire)
            .service(etablissements::modifier),
    );
}
