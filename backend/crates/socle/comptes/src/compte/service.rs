//! Couche service des comptes — **création, état, mot de passe**.
//!
//! # Ce que ce fichier ne fait jamais
//!
//! Il ne rend **aucun condensat**, sur aucun chemin. Ce n'est pas de la discipline : [`CompteVue`]
//! n'a pas de champ où en mettre un, et la lecture d'affichage
//! ([`super::repository::lire`]) ne sélectionne même pas la colonne. Deux barrières, deux
//! moments — le type empêche d'y penser, la requête empêche de l'avoir sous la main.
//!
//! Il ne vérifie **aucune permission** de l'appelant : cette garde est celle du handler
//! (`api/src/securite.rs`). Une garde à un seul endroit se relit ; une garde dispersée se
//! contourne.
//!
//! # La politique de mot de passe s'applique ICI, jamais à la connexion
//!
//! Création et changement, c'est tout. Refuser à la **connexion** un mot de passe devenu
//! compromis entre-temps enfermerait dehors un utilisateur parfaitement légitime, sans qu'il
//! puisse rien y faire — puisqu'il faut être connecté pour en changer.
//!
//! La longueur minimale vient du **catalogue de paramètres**, jamais d'une constante : un
//! exploitant qui veut douze caractères les règle. `LONGUEUR_MIN_DEFAUT` n'existe que comme repli
//! quand aucun niveau de la chaîne ne porte la clé.
//!
//! # Le changement de mot de passe révoque les autres sessions, immédiatement
//!
//! C'est le geste qu'on fait quand on soupçonne que quelqu'un a son mot de passe. Laisser les
//! sessions ouvertes jusqu'à leur expiration — quatre-vingt-dix jours pour un rafraîchissement —
//! viderait le geste de son sens. **« Les autres » et non « toutes »** : celui qui change son mot
//! de passe ne se déconnecte pas lui-même en le faisant.

use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use kaya_etablissements::configuration::repository as configuration;
use kaya_etablissements::tenant_context;
use kaya_etablissements::Issue;
use kaya_synchronisation::{EvenementAEcrire, OutboxWriter};

use crate::audit::{EntreeAudit, JournalAudit, TypeActionAudit};
use crate::authentification::{self, politique};
use crate::session::entrepot::Entrepot;

use super::modele::{CLE_LONGUEUR_MIN, CompteVue, CreerCompte, ErreurCompte};
use super::repository;

/// Nom de l'agrégat au grand livre.
pub const AGREGAT_COMPTE: &str = "compte";

/// Types d'événements — nomenclature `agregat.action`.
pub const TYPE_COMPTE_CREE: &str = "compte.cree";
pub const TYPE_COMPTE_DESACTIVE: &str = "compte.desactive";
pub const TYPE_COMPTE_REACTIVE: &str = "compte.reactive";
pub const TYPE_COMPTE_MOT_DE_PASSE_CHANGE: &str = "compte.mot_de_passe_change";

/// Version du format des charges utiles.
pub const VERSION_SCHEMA_COMPTE: i16 = 1;

/// La cible d'audit — le compte lui-même.
const CIBLE_COMPTE: &str = "compte";

/// Service des comptes.
pub struct ServiceComptes<E: OutboxWriter, J: JournalAudit> {
    pool: PgPool,
    outbox: E,
    journal: J,
    /// L'entrepôt des sessions — **il n'est là que pour la révocation au changement de secret**.
    entrepot: Entrepot,
}

impl<E: OutboxWriter, J: JournalAudit> ServiceComptes<E, J> {
    pub fn nouveau(pool: PgPool, outbox: E, journal: J, entrepot: Entrepot) -> Self {
        Self {
            pool,
            outbox,
            journal,
            entrepot,
        }
    }

    /// Crée un compte.
    ///
    /// Ordre des opérations :
    ///
    ///   1. **au moins un identifiant** — un compte sans téléphone ni courriel ne peut jamais se
    ///      connecter, et rien ne le signalerait avant la première tentative ;
    ///   2. transaction, pose du tenant ;
    ///   3. longueur minimale lue du catalogue, puis politique de mot de passe ;
    ///   4. hachage — **hors** de toute décision, il coûte cent millisecondes et ne juge rien ;
    ///   5. insertion idempotente ;
    ///   6. événement **uniquement si la ligne vient d'être créée** ;
    ///   7. commit.
    ///
    /// **Aucune entrée d'audit.** La taxonomie des dix familles n'en a pas pour la création
    /// (`docs/taxonomie-audit.md`) : le registre trace ce qui retire ou modifie un droit, un prix
    /// ou de l'argent. Une création laisse `compte.cree` au grand livre, qui est permanent.
    #[tracing::instrument(
        skip(self, demande),
        fields(compte.id = %demande.id, tenant.id = %tenant_id)
    )]
    pub async fn creer(
        &self,
        tenant_id: Uuid,
        demande: CreerCompte,
    ) -> Result<(CompteVue, Issue), ErreurCompte> {
        let telephone = normaliser(demande.identifiant_telephone.clone());
        let email = normaliser(demande.identifiant_email.clone());

        if telephone.is_none() && email.is_none() {
            return Err(ErreurCompte::IdentifiantAbsent);
        }

        let demande = CreerCompte {
            identifiant_telephone: telephone,
            identifiant_email: email,
            ..demande
        };

        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, tenant_id).await?;

        let longueur_min = longueur_min(&mut tx).await;
        politique::verifier(&demande.mot_de_passe, longueur_min)?;

        let condensat = authentification::hacher(&demande.mot_de_passe)?;

        let (id, issue) = repository::inserer(&mut tx, tenant_id, &demande, &condensat)
            .await
            // Une violation d'unicité sur `(tenant_id, identifiant_*)` est un **refus métier**,
            // pas une panne. Le message ne dit **pas** que l'identifiant existe déjà : le dire
            // apprendrait, à un habilité d'un tenant, quels numéros sont clients de Kaya.
            .map_err(|erreur| match erreur {
                ErreurCompte::Base(sqlx::Error::Database(e)) if e.is_unique_violation() => {
                    tracing::info!(
                        contrainte = e.constraint().unwrap_or("inconnue"),
                        "création de compte refusée — identifiant déjà employé"
                    );
                    ErreurCompte::IdentifiantRefuse
                }
                autre => autre,
            })?;

        if issue == Issue::Creee {
            let evenement = EvenementAEcrire {
                id: Uuid::now_v7(),
                tenant_id,
                etablissement_id: None,
                type_evenement: TYPE_COMPTE_CREE.to_owned(),
                agregat: AGREGAT_COMPTE.to_owned(),
                agregat_id: id,
                version_schema: VERSION_SCHEMA_COMPTE,
                // **Ni le mot de passe ni son condensat.** Le grand livre est permanent : un
                // secret qui y entre y reste pour toujours.
                payload: json!({
                    "compte_id": id,
                    "personne_id": demande.personne_id,
                    "identifiant_telephone": demande.identifiant_telephone,
                    "identifiant_email": demande.identifiant_email,
                }),
            };
            self.outbox.ecrire(&mut tx, evenement).await?;
        }

        let vue = repository::lire(&mut tx, id).await?.ok_or(ErreurCompte::Inconnu)?;

        tx.commit().await?;
        Ok((vue, issue))
    }

    /// Lit un compte.
    pub async fn lire(&self, tenant_id: Uuid, id: Uuid) -> Result<CompteVue, ErreurCompte> {
        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, tenant_id).await?;
        let vue = repository::lire(&mut tx, id).await?;
        tx.rollback().await?;
        vue.ok_or(ErreurCompte::Inconnu)
    }

    /// Liste les comptes, avec les trois filtres combinables du contrat.
    pub async fn lister(
        &self,
        tenant_id: Uuid,
        etablissement_id: Option<Uuid>,
        actif: Option<bool>,
        role_code: Option<&str>,
    ) -> Result<Vec<CompteVue>, ErreurCompte> {
        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, tenant_id).await?;
        let comptes = repository::lister(&mut tx, etablissement_id, actif, role_code).await?;
        tx.rollback().await?;
        Ok(comptes)
    }

    /// Active ou désactive un compte.
    ///
    /// # Une désactivation trace, une réactivation non — et c'est la taxonomie qui le dit
    ///
    /// `suppression` est la famille des **mises hors service** (`docs/taxonomie-audit.md`) : la
    /// désactivation d'un compte en est une, sa réactivation n'en est pas. Aucune des dix familles
    /// ne couvre le rétablissement d'un droit, et en inventer une onzième ici serait une décision
    /// de taxonomie prise dans un fichier de service.
    ///
    /// L'événement, lui, est émis dans les deux sens : le grand livre trace les transitions
    /// d'état, pas les gestes d'administration.
    ///
    /// **Sans transition, rien n'est émis.** Désactiver un compte déjà inactif est un rejeu, et le
    /// traiter comme un acte ferait du registre le journal des reprises réseau du terminal.
    #[tracing::instrument(skip(self), fields(compte.id = %compte_id, tenant.id = %tenant_id))]
    pub async fn changer_etat(
        &self,
        tenant_id: Uuid,
        auteur_compte_id: Uuid,
        compte_id: Uuid,
        actif: bool,
    ) -> Result<CompteVue, ErreurCompte> {
        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, tenant_id).await?;

        let ancien = repository::changer_etat(&mut tx, compte_id, actif)
            .await?
            .ok_or(ErreurCompte::Inconnu)?;

        if ancien != actif {
            let evenement = EvenementAEcrire {
                id: Uuid::now_v7(),
                tenant_id,
                etablissement_id: None,
                type_evenement: if actif {
                    TYPE_COMPTE_REACTIVE.to_owned()
                } else {
                    TYPE_COMPTE_DESACTIVE.to_owned()
                },
                agregat: AGREGAT_COMPTE.to_owned(),
                agregat_id: compte_id,
                version_schema: VERSION_SCHEMA_COMPTE,
                payload: json!({ "compte_id": compte_id, "actif": actif }),
            };
            self.outbox.ecrire(&mut tx, evenement).await?;

            if !actif {
                self.journal
                    .tracer(
                        &mut tx,
                        tenant_id,
                        EntreeAudit {
                            id: Uuid::now_v7(),
                            etablissement_id: None,
                            type_action: TypeActionAudit::Suppression,
                            auteur_compte_id,
                            cible_type: CIBLE_COMPTE.to_owned(),
                            cible_id: Some(compte_id),
                            contexte: json!({ "geste": "desactivation_compte" }),
                            horodatage_client: None,
                        },
                    )
                    .await?;
            }
        }

        let vue = repository::lire(&mut tx, compte_id)
            .await?
            .ok_or(ErreurCompte::Inconnu)?;

        tx.commit().await?;
        Ok(vue)
    }

    /// Change le mot de passe **et coupe les autres sessions, immédiatement**.
    ///
    /// # `mot_de_passe_actuel` distingue les deux appelants, et sa présence n'est pas un choix
    ///
    /// Un compte qui agit **sur lui-même** le fournit ; un habilité qui agit sur un autre ne le
    /// fournit pas — il ne le connaît pas, et le lui demander rendrait l'opération impossible dans
    /// le seul cas où elle sert : quelqu'un a perdu son mot de passe.
    ///
    /// C'est le **handler** qui décide lequel des deux cas s'applique, parce que lui seul sait qui
    /// appelle (`securite::exiger_ou_soi`). Ce service reçoit la décision déjà prise.
    ///
    /// # La révocation vient APRÈS le commit, et c'est délibéré
    ///
    /// Redis n'est pas transactionnel avec PostgreSQL. Révoquer d'abord puis échouer à écrire
    /// déconnecterait tout le monde sans changer le mot de passe — le pire des deux mondes.
    /// Écrire d'abord puis échouer à révoquer laisse un mot de passe changé et des sessions encore
    /// ouvertes, ce qui est le moindre mal : elles expireront, et l'utilisateur peut couper chaque
    /// appareil depuis « Appareils connectés ».
    #[tracing::instrument(
        skip(self, mot_de_passe_actuel, nouveau_mot_de_passe),
        fields(compte.id = %compte_id, tenant.id = %tenant_id)
    )]
    pub async fn changer_mot_de_passe(
        &self,
        tenant_id: Uuid,
        compte_id: Uuid,
        mot_de_passe_actuel: Option<&str>,
        nouveau_mot_de_passe: &str,
        session_courante: Option<Uuid>,
        duree_acces_s: i64,
    ) -> Result<(), ErreurCompte> {
        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, tenant_id).await?;

        let compte = repository::lire_pour_authentification(&mut tx, compte_id)
            .await?
            .ok_or(ErreurCompte::Inconnu)?;

        if let Some(actuel) = mot_de_passe_actuel {
            let verification = authentification::verifier(&compte.condensat_mot_de_passe, actuel)?;
            if !verification.valide {
                return Err(ErreurCompte::MotDePasseActuelInvalide);
            }
        }

        let longueur_min = longueur_min(&mut tx).await;
        politique::verifier(nouveau_mot_de_passe, longueur_min)?;

        let condensat = authentification::hacher(nouveau_mot_de_passe)?;
        repository::ecrire_condensat(&mut tx, compte_id, &condensat).await?;

        let evenement = EvenementAEcrire {
            id: Uuid::now_v7(),
            tenant_id,
            etablissement_id: None,
            type_evenement: TYPE_COMPTE_MOT_DE_PASSE_CHANGE.to_owned(),
            agregat: AGREGAT_COMPTE.to_owned(),
            agregat_id: compte_id,
            version_schema: VERSION_SCHEMA_COMPTE,
            // **Ni le secret, ni son condensat, ni sa longueur.** La longueur seule réduirait
            // déjà l'espace de recherche d'une attaque hors ligne.
            payload: json!({ "compte_id": compte_id }),
        };
        self.outbox.ecrire(&mut tx, evenement).await?;

        tx.commit().await?;

        // Après le commit — voir le commentaire de la méthode.
        let sessions = self.entrepot.lister(compte_id).await?;
        for session in sessions {
            if Some(session.id) == session_courante {
                continue;
            }
            self.entrepot
                .marquer_revoquee(session.id, duree_acces_s)
                .await?;
            self.entrepot.revoquer_famille(session.famille_id).await?;
            self.entrepot.oublier(compte_id, session.id).await?;
        }

        Ok(())
    }
}

/// La longueur minimale du mot de passe, lue du catalogue.
///
/// # Pourquoi elle ne remonte pas d'erreur
///
/// Une panne de lecture de configuration ne doit pas empêcher de créer un compte : le repli vaut
/// le défaut documenté du catalogue, et la valeur fautive reste visible à l'écran de
/// configuration. C'est le même arbitrage que `session/parametres.rs`, et pour la même raison —
/// mettre l'établissement hors service coûte plus cher que d'appliquer huit caractères.
///
/// La résolution se fait au **niveau tenant** : un compte n'appartient à aucun établissement, et
/// une politique de mot de passe qui varierait d'un établissement à l'autre au sein d'un même
/// tenant serait ingérable pour un compte qui en porte plusieurs.
async fn longueur_min(tx: &mut sqlx::PgTransaction<'_>) -> usize {
    match configuration::resoudre(tx, None, None, None, CLE_LONGUEUR_MIN).await {
        Ok(Some(valeur)) => valeur
            .valeur
            .as_i64()
            .filter(|v| *v > 0)
            .map(|v| v as usize)
            .unwrap_or(politique::LONGUEUR_MIN_DEFAUT),
        Ok(None) => politique::LONGUEUR_MIN_DEFAUT,
        Err(erreur) => {
            tracing::warn!(
                erreur = %erreur,
                cle = CLE_LONGUEUR_MIN,
                "lecture de la longueur minimale impossible — repli sur le défaut du catalogue"
            );
            politique::LONGUEUR_MIN_DEFAUT
        }
    }
}

/// Ramène une chaîne vide à `None`.
///
/// Sans cela, un formulaire dont le champ n'a pas été rempli enverrait `""`, et
/// `compte_identifiant_telephone_unique` refuserait le **deuxième** compte sans téléphone.
fn normaliser(valeur: Option<String>) -> Option<String> {
    valeur
        .map(|v| v.trim().to_owned())
        .filter(|v| !v.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Les quatre types d'événements sont **stables** : ils partent dans un grand livre permanent.
    #[test]
    fn les_types_d_evenements_ne_bougent_pas() {
        assert_eq!(TYPE_COMPTE_CREE, "compte.cree");
        assert_eq!(TYPE_COMPTE_DESACTIVE, "compte.desactive");
        assert_eq!(TYPE_COMPTE_REACTIVE, "compte.reactive");
        assert_eq!(TYPE_COMPTE_MOT_DE_PASSE_CHANGE, "compte.mot_de_passe_change");
    }

    #[test]
    fn une_chaine_vide_devient_absente() {
        assert_eq!(normaliser(Some(String::new())), None);
        assert_eq!(normaliser(Some("  ".to_owned())), None);
        assert_eq!(
            normaliser(Some(" +2250700000001 ".to_owned())),
            Some("+2250700000001".to_owned())
        );
    }
}
