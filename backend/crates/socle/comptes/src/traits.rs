//! **Les traits exposés par `socle/comptes`.**
//!
//! Ils sont le **seul chemin** par lequel les autres crates atteignent ce que ce cycle produit :
//! aucune requête ne joint deux schémas de modules (principe II, porte P-04), et aucun crate du
//! socle ne dépend d'une verticale (porte P-03).
//!
//! Le troisième — [`crate::audit::JournalAudit`] — vit avec sa table, dans `audit/service.rs` :
//! c'est un trait d'**écriture**, et le séparer de l'implémentation qui garantit sa transaction
//! aurait éloigné la signature de sa raison d'être.
//!
//! # Une note de nommage, parce que ce cycle mélange les deux conventions
//!
//! `CLAUDE.md` range les traits d'abstraction en **anglais** et nomme `AccessController` parmi les
//! traits canoniques du produit. Il est donc en anglais, sans discussion. `AnnuaireComptes` n'y
//! figure pas : il suit la convention effective du cycle 002 — `RegistreModules`,
//! `RepertoirePointsDeVente` — et reste en français.
//!
//! **La règle qui en découle** : un trait nommé par les documents de référence garde son nom ; un
//! trait nouveau suit le français des identifiants métier. Écrit ici pour que le cycle suivant
//! n'ait pas à en décider une troisième fois.
//!
//! # Pourquoi `#[async_trait::async_trait]`
//!
//! Rust sait écrire `async fn` dans un trait depuis 1.75, mais un tel trait **n'est pas
//! dyn-compatible**. L'injection de dépendances du cadrage §13.2 suppose `Arc<dyn Trait>` :
//! l'annotation est un choix contraint, pas une habitude reprise d'un exemple.

use std::collections::{BTreeMap, BTreeSet};

use sqlx::PgPool;
use uuid::Uuid;

use crate::roles::repository as depot_roles;

// =================================================================================================
//  Erreur commune
// =================================================================================================

/// Échec de lecture des droits ou de l'annuaire.
///
/// **Aucune variante ne distingue « compte inconnu » de « aucune permission ».** Ce n'est pas un
/// oubli : les deux rendent la même chose — un ensemble vide, un `None` — et les distinguer
/// donnerait à un appelant de quoi savoir qu'un compte existe.
#[derive(Debug, thiserror::Error)]
pub enum ErreurAcces {
    #[error("lecture des droits impossible : {0}")]
    Base(#[from] sqlx::Error),

    #[error("contexte de tenant : {0}")]
    ContexteTenant(#[from] kaya_etablissements::tenant_context::ErreurContexteTenant),

    #[error("lecture des rôles : {0}")]
    Roles(String),
}

impl From<crate::roles::ErreurRoles> for ErreurAcces {
    fn from(erreur: crate::roles::ErreurRoles) -> Self {
        match erreur {
            crate::roles::ErreurRoles::Base(e) => ErreurAcces::Base(e),
            crate::roles::ErreurRoles::ContexteTenant(e) => ErreurAcces::ContexteTenant(e),
            autre => ErreurAcces::Roles(autre.to_string()),
        }
    }
}

// =================================================================================================
//  1. AccessController — la seule autorité sur « a-t-il le droit »
// =================================================================================================

/// **Le trait canonique du produit**, nommé au préambule de `CLAUDE.md`.
///
/// # Union, jamais priorité — et c'est le TYPE qui le tient
///
/// `BTreeSet<String>` plutôt que `Vec<String>` : le type dit l'unicité et l'ordre stable. Une
/// tuile issue de trois rôles n'apparaît qu'une fois (FR-027) sans que l'appelant ait à
/// dédoublonner, et deux appels rendent le même ordre — ce qui rend les tests comparables sans
/// tri préalable.
///
/// **Aucune signature de ce trait n'accepte ni ne rend un rôle.** C'est la faute de FR-017 rendue
/// structurellement impossible : un consommateur qui voudrait écrire `if role == "gerant"` n'a
/// rien à quoi se brancher. La hiérarchie que le principe VII interdit n'est pas empêchée par une
/// consigne, elle est empêchée par l'absence de prise.
///
/// # Ce qu'il n'expose volontairement pas
///
/// | Absent | Raison |
/// |---|---|
/// | Les **rôles** d'un compte | Voir ci-dessus. Seul l'écran `G3` les affiche, et il passe par l'API |
/// | L'attribution ou le retrait | **Classe C**, réservée au service. Un trait d'écriture des droits serait un chemin d'élévation offert à tout crate |
/// | Le condensat de mot de passe | Aucun autre module n'a de raison de le voir, ni de savoir qu'il existe |
///
/// **Consommateurs attendus** : `api/` pour la garde des handlers ; tout cycle ultérieur qui
/// protège une action — `verticales/hebergement` pour `heb.unite.attribuer`, `socle/caisse` pour
/// une remise.
#[async_trait::async_trait]
pub trait AccessController: Send + Sync {
    /// Permissions effectives d'un compte sur un établissement — **l'UNION de ses rôles**.
    ///
    /// Un compte **sans aucun rôle** rend un ensemble **vide**, jamais une erreur : il se
    /// connecte, et son accueil est vide. Une erreur rendrait la connexion impossible pour un
    /// compte fraîchement créé, avant que quiconque ait eu le temps de lui donner un rôle.
    async fn permissions_effectives(
        &self,
        tenant_id: Uuid,
        compte_id: Uuid,
        etablissement_id: Option<Uuid>,
    ) -> Result<BTreeSet<String>, ErreurAcces>;

    /// Le compte détient-il cette permission ? **Convenance sur la précédente.**
    ///
    /// Fournie par défaut plutôt que laissée à chaque implémentation : deux implémentations de la
    /// même question finiraient par répondre différemment, et celle qui dérive est toujours celle
    /// qu'on ne relit pas.
    async fn detient(
        &self,
        tenant_id: Uuid,
        compte_id: Uuid,
        etablissement_id: Option<Uuid>,
        permission: &str,
    ) -> Result<bool, ErreurAcces> {
        Ok(self
            .permissions_effectives(tenant_id, compte_id, etablissement_id)
            .await?
            .contains(permission))
    }
}

// =================================================================================================
//  2. AnnuaireComptes — lire qui est l'auteur, sans jointure
// =================================================================================================

/// Un compte réduit à ce qu'un écran affiche de son **auteur**.
///
/// Ni identifiant de connexion, ni condensat, ni rôles. Ce type part dans des réponses HTTP : y
/// laisser le téléphone reviendrait à diffuser un contact personnel dans un registre à rétention
/// illimitée.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct CompteResume {
    pub id: Uuid,
    /// Nom affichable, lu de **`personne`** — jamais de l'identifiant de connexion.
    pub nom_affichage: String,
    /// Un compte désactivé garde ses entrées d'audit : le drapeau dit qu'il ne se connecte plus,
    /// pas qu'il n'a rien fait.
    pub actif: bool,
}

/// **Ce qui rend lisible un `auteur_compte_id` sans clé étrangère.**
///
/// `note_etablissement.auteur_compte_id` porte, depuis le cycle 001, un UUID **sans clé
/// étrangère** — le module doré appelle cela « le point le plus contre-intuitif du patron ». Ce
/// trait est la contrepartie promise de cette absence : sans lui, la tentation du
/// `JOIN comptes.compte` reviendrait au premier écran qui affiche un auteur, et la porte P-04
/// l'attraperait après coup plutôt qu'avant.
#[async_trait::async_trait]
pub trait AnnuaireComptes: Send + Sync {
    async fn compte(
        &self,
        tenant_id: Uuid,
        id: Uuid,
    ) -> Result<Option<CompteResume>, ErreurAcces>;

    /// **Lecture en lot** — et ce n'est pas une optimisation prématurée.
    ///
    /// L'écran `G4` affiche une page d'entrées d'auteurs différents. Sans lot, il ferait cent
    /// appels : c'est le problème classique, et il est moins cher de le fermer à l'écriture du
    /// trait qu'à la première lenteur en clientèle.
    ///
    /// Rend une `BTreeMap` et non un `Vec` : l'appelant cherche par identifiant, et un identifiant
    /// absent — compte supprimé, entrée d'un autre tenant — se lit comme une absence de clé, sans
    /// qu'il ait à parcourir la liste pour s'en assurer.
    async fn comptes(
        &self,
        tenant_id: Uuid,
        ids: &[Uuid],
    ) -> Result<BTreeMap<Uuid, CompteResume>, ErreurAcces>;
}

// =================================================================================================
//  Implémentations PostgreSQL
// =================================================================================================

/// Implémentation des deux traits de lecture.
///
/// Elle détient un pool clonable et **aucun état**. Chaque méthode ouvre sa propre transaction,
/// y pose le tenant, lit, et **annule** : ce sont des lectures, et un `commit` sur une
/// transaction sans écriture ne dirait rien de plus tout en laissant croire qu'il s'y passe
/// quelque chose.
#[derive(Debug, Clone)]
pub struct ControleAccesPostgres {
    pool: PgPool,
}

impl ControleAccesPostgres {
    pub fn nouveau(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl AccessController for ControleAccesPostgres {
    async fn permissions_effectives(
        &self,
        tenant_id: Uuid,
        compte_id: Uuid,
        etablissement_id: Option<Uuid>,
    ) -> Result<BTreeSet<String>, ErreurAcces> {
        let mut tx = self.pool.begin().await?;
        // **Le tenant se pose avant toute lecture, même en lecture seule.** Une transaction sans
        // cet appel ne voit aucune ligne — pas une erreur, zéro ligne — et l'appelant conclurait
        // « aucune permission » là où il fallait lire « isolation mal posée ».
        kaya_etablissements::tenant_context::poser_tenant(&mut tx, tenant_id).await?;

        let permissions =
            depot_roles::permissions_effectives(&mut tx, compte_id, etablissement_id).await?;

        tx.rollback().await?;
        Ok(permissions)
    }
}

#[async_trait::async_trait]
impl AnnuaireComptes for ControleAccesPostgres {
    async fn compte(
        &self,
        tenant_id: Uuid,
        id: Uuid,
    ) -> Result<Option<CompteResume>, ErreurAcces> {
        // Passe par la lecture en lot plutôt que d'écrire une seconde requête : deux requêtes pour
        // la même question finiraient par diverger sur la définition de `nom_affichage`.
        Ok(self.comptes(tenant_id, &[id]).await?.remove(&id))
    }

    async fn comptes(
        &self,
        tenant_id: Uuid,
        ids: &[Uuid],
    ) -> Result<BTreeMap<Uuid, CompteResume>, ErreurAcces> {
        if ids.is_empty() {
            // Une requête `= ANY('{}')` serait correcte et rendrait zéro ligne ; s'en dispenser
            // évite un aller-retour sur le cas le plus fréquent d'une page d'audit vide.
            return Ok(BTreeMap::new());
        }

        let mut tx = self.pool.begin().await?;
        kaya_etablissements::tenant_context::poser_tenant(&mut tx, tenant_id).await?;

        // `nom_affichage` vient de `personne`, jamais de `compte.identifiant_*`. La jointure est
        // **intra-schéma** : `comptes.compte` et `comptes.personne` sont deux tables du même
        // module, ce que le principe II autorise et que la porte P-04 ne vise pas.
        let lignes = sqlx::query!(
            r#"
            SELECT c.id AS "id!",
                   TRIM(COALESCE(p.prenoms, '') || ' ' || p.nom) AS "nom_affichage!",
                   c.actif AS "actif!"
            FROM comptes.compte c
            JOIN comptes.personne p ON p.id = c.personne_id
            WHERE c.id = ANY($1)
            "#,
            ids
        )
        .fetch_all(&mut *tx)
        .await?;

        tx.rollback().await?;

        Ok(lignes
            .into_iter()
            .map(|l| {
                (
                    l.id,
                    CompteResume {
                        id: l.id,
                        nom_affichage: l.nom_affichage,
                        actif: l.actif,
                    },
                )
            })
            .collect())
    }
}

// =================================================================================================
//  AnnuaireClients — exposé à `verticales/hebergement`, cycle 006
// =================================================================================================

/// Échec de lecture de l'annuaire des clients.
///
/// **Aucune variante ne distingue « client inconnu » de « client d'un autre tenant »** : les deux
/// rendent une absence, et les distinguer donnerait de quoi savoir qu'un identifiant existe
/// ailleurs. Même raisonnement qu'[`ErreurAcces`].
#[derive(Debug, thiserror::Error)]
pub enum ErreurAnnuaireClients {
    #[error("lecture de l'annuaire des clients impossible : {0}")]
    Base(#[from] sqlx::Error),

    #[error("contexte de tenant : {0}")]
    ContexteTenant(#[from] kaya_etablissements::tenant_context::ErreurContexteTenant),
}

/// L'annuaire des clients, tel qu'une **verticale** le lit — **jamais par jointure**.
///
/// # Ce que ce trait empêche, et pourquoi il n'a aucun garde-fou naturel
///
/// Un séjour affiche toujours le nom de son client. C'est la jointure
/// `hebergement.sejour × comptes.personne` que tout le monde écrirait — et P-04 l'attraperait,
/// mais **après coup**, une fois l'écran écrit.
///
/// *Une alternative qui existe se prend ; une alternative à construire se contourne.* C'est le
/// raisonnement d'`EstablishmentDirectory`, posé au cycle 001, et il vaut ici avec un appelant
/// réel en plus — le service de séjour, sur trois chemins : la liste des séjours en cours (`R7`),
/// la fiche d'un séjour, et la reconnaissance d'un client au passage (`R4`, « M. Bakayoko —
/// 7ᵉ passage »).
///
/// # ⚠️ Le sens INVERSE est interdit, et il est plus dangereux
///
/// **`socle/comptes` ne lit JAMAIS `hebergement.sejour`.** L'historique des séjours d'un client
/// (`GET /clients/{id}/sejours`) paraît appartenir au client ; il est servi **depuis le crate
/// `hebergement`**. Autrement, ce serait deux violations d'un coup — jointure inter-schémas
/// (**P-04**) *et* arête `socle/ → verticales/` (**P-03**).
#[async_trait::async_trait]
pub trait AnnuaireClients: Send + Sync {
    /// Les résumés de plusieurs clients, **en une requête**.
    ///
    /// ⚠️ **`resumes(&[Uuid])`, jamais `resume(Uuid)`.** Une signature unitaire produirait N+1
    /// requêtes sur la liste des séjours en cours — et c'est le détail qui décide si l'écran de
    /// départ s'ouvre en 200 ms ou en deux secondes. La forme par lot n'est pas une optimisation
    /// prématurée : elle est **la seule qui ne se dégrade pas** quand la liste grandit.
    ///
    /// Les identifiants inconnus sont **absents** de la réponse, jamais rendus en `None` : un
    /// séjour dont le client a été purgé (TRX-06) reste lisible, **sans nom**. Rendre une entrée
    /// vide obligerait chaque appelant à distinguer « purgé » de « jamais rattaché », deux cas
    /// qu'aucun écran ne présente différemment.
    async fn resumes(
        &self,
        tenant_id: Uuid,
        ids: &[Uuid],
    ) -> Result<Vec<crate::client::ClientResume>, ErreurAnnuaireClients>;

    /// Le client existe et appartient au tenant courant.
    ///
    /// Appelé par l'ouverture d'un séjour pour refuser un `client_id` **inventé**. La politique de
    /// sécurité empêcherait déjà la lecture d'un client d'un autre tenant, mais un refus explicite
    /// vaut mieux qu'une ligne orpheline qu'aucune contrainte ne peut interdire — la clé étrangère
    /// étant impossible entre deux schémas (principe II).
    async fn existe(&self, tenant_id: Uuid, id: Uuid) -> Result<bool, ErreurAnnuaireClients>;
}

/// Implémentation PostgreSQL de [`AnnuaireClients`].
///
/// ⚠️ **Elle ne déchiffre aucun numéro de pièce et n'en journalise donc aucun accès.**
/// `ClientResume` porte `piece_enregistree`, un booléen — ce dont la fiche de police a besoin
/// **sans lire la pièce** (FR-047). Laisser le numéro traverser vers une verticale multiplierait
/// les endroits où la rétention de 90 jours de TRX-06 devra le purger.
#[derive(Debug, Clone)]
pub struct PgAnnuaireClients {
    pool: PgPool,
}

impl PgAnnuaireClients {
    pub fn nouveau(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl AnnuaireClients for PgAnnuaireClients {
    async fn resumes(
        &self,
        tenant_id: Uuid,
        ids: &[Uuid],
    ) -> Result<Vec<crate::client::ClientResume>, ErreurAnnuaireClients> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut tx = self.pool.begin().await?;
        kaya_etablissements::tenant_context::poser_tenant(&mut tx, tenant_id).await?;
        let liste = crate::client::repository::resumes(&mut tx, ids)
            .await
            .map_err(en_erreur_annuaire)?;
        tx.rollback().await?;
        Ok(liste)
    }

    async fn existe(&self, tenant_id: Uuid, id: Uuid) -> Result<bool, ErreurAnnuaireClients> {
        let mut tx = self.pool.begin().await?;
        kaya_etablissements::tenant_context::poser_tenant(&mut tx, tenant_id).await?;
        let trouve = crate::client::repository::existe(&mut tx, id)
            .await
            .map_err(en_erreur_annuaire)?;
        tx.rollback().await?;
        Ok(trouve)
    }
}

/// Réduit une [`crate::client::ErreurClient`] aux deux causes que l'annuaire peut rencontrer.
///
/// Les autres variantes — validation, coffre, registre — ne sont pas atteignables depuis une
/// lecture par lot : les faire remonter dans le type de l'annuaire donnerait à une verticale de
/// quoi connaître les refus de validation du socle.
fn en_erreur_annuaire(erreur: crate::client::ErreurClient) -> ErreurAnnuaireClients {
    match erreur {
        crate::client::ErreurClient::Base(e) => ErreurAnnuaireClients::Base(e),
        crate::client::ErreurClient::ContexteTenant(e) => ErreurAnnuaireClients::ContexteTenant(e),
        autre => ErreurAnnuaireClients::Base(sqlx::Error::Protocol(autre.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Le type porte l'unicité : trois rôles qui partagent une permission n'en donnent qu'une.
    ///
    /// # Pourquoi ce fichier n'a pas de test asynchrone
    ///
    /// Aucun crate du socle n'a de `dev-dependencies`, et donc pas de `tokio` de test : les tests
    /// asynchrones du produit vivent dans `backend/tests/`, contre une vraie base. Y mettre
    /// `detient` est d'ailleurs plus fort qu'un double d'essai — il exerce l'implémentation
    /// réelle, avec son isolation de tenant. Voir `backend/tests/roles_cumules.rs`.
    #[test]
    fn l_ensemble_dedoublonne_par_construction() {
        // Ce que `BTreeSet` rend structurellement impossible : la même permission deux fois.
        let mut cumul = BTreeSet::new();
        for role in [
            vec!["cpt.compte.lire", "etb.service.basculer"],
            vec!["cpt.compte.lire"],
            vec!["etb.service.basculer", "cpt.audit.consulter"],
        ] {
            cumul.extend(role.into_iter().map(str::to_owned));
        }

        assert_eq!(cumul.len(), 3);
        // Et l'ordre est stable, ce qui rend deux appels comparables sans tri préalable.
        assert_eq!(
            cumul.iter().map(String::as_str).collect::<Vec<_>>(),
            vec!["cpt.audit.consulter", "cpt.compte.lire", "etb.service.basculer"]
        );
    }
}
