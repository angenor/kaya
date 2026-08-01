//! Attribution et retrait — **deux actes, deux entrées d'audit, une seule transaction chacun**.
//!
//! # Trois choses tombent ou passent ensemble
//!
//! Une attribution écrit **la ligne**, **l'événement outbox** et **l'entrée d'audit** dans la même
//! transaction (FR-024, porte P-05). Ce n'est pas une discipline mais deux signatures :
//! `OutboxWriter::ecrire` et `JournalAudit::tracer` prennent la transaction et n'en ouvrent
//! jamais une. Un rôle attribué dont la trace manque serait exactement le trou que CPT-04 doit
//! fermer — et il ne se verrait qu'au moment où quelqu'un chercherait qui a donné ce droit.
//!
//! # `role.attribue` ET `changement_role` — ce n'est pas une redondance
//!
//! Deux registres, deux publics, deux classes (research R-08). L'outbox alimente les projections
//! et se consomme ; le registre des actions est **un produit que le propriétaire achète**, et il
//! ne se consomme pas. Les fusionner obligerait à choisir entre une rétention illimitée pour tout
//! et une purge qui emporterait la trace.
//!
//! # L'établissement se vérifie par TRAIT, jamais par clé étrangère
//!
//! `compte_role.etablissement_id` n'a aucune clé étrangère : ce serait une clé inter-schémas, que
//! le principe II interdit et que la porte P-04 refuse. L'existence passe donc par
//! [`EstablishmentDirectory`], ce qui donne un `404 etablissement_inconnu` intelligible plutôt
//! qu'une violation de contrainte remontée en `500`.
//!
//! # Ce que ce service ne fait PAS
//!
//! Il ne vérifie **pas** que l'appelant a le droit d'attribuer. Cette garde est celle du handler
//! (`api/src/securite.rs`, T040) : la mêler ici obligerait à passer les permissions de l'appelant
//! à chaque service du produit, et le jour où un service oublierait de les consulter, rien ne le
//! signalerait. Une garde à un seul endroit se relit ; une garde dispersée se contourne.

use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use kaya_etablissements::tenant_context;
use kaya_etablissements::{EstablishmentDirectory, Issue};
use kaya_synchronisation::{EvenementAEcrire, OutboxWriter};

use crate::audit::{EntreeAudit, JournalAudit, TypeActionAudit};

use super::modele::{AttribuerRole, ErreurRoles};
use super::repository;

/// Nom de l'agrégat au grand livre.
pub const AGREGAT_COMPTE_ROLE: &str = "compte_role";

/// Types d'événements — nomenclature `agregat.action`.
pub const TYPE_ROLE_ATTRIBUE: &str = "role.attribue";
pub const TYPE_ROLE_RETIRE: &str = "role.retire";

/// Version du format des charges utiles.
///
/// **Toute évolution du format l'incrémente.** La génération SYSCOHADA rétroactive relira un jour
/// des événements écrits par des versions du code qui n'existeront plus.
pub const VERSION_SCHEMA_ROLE: i16 = 1;

/// La cible d'audit d'un changement de rôle.
///
/// C'est le **compte** dont les droits changent, pas le rôle : le propriétaire cherche « qu'est-il
/// arrivé au compte d'Adjoua », jamais « qu'est-il arrivé au rôle caissier ».
const CIBLE_COMPTE: &str = "compte";

/// Service d'attribution et de retrait de rôles.
///
/// Trois dépendances injectées, et chacune est une frontière : l'outbox (grand livre), le registre
/// des actions (audit), et l'annuaire des établissements (trait, jamais jointure).
pub struct ServiceRoles<E: OutboxWriter, J: JournalAudit, R: EstablishmentDirectory> {
    pool: PgPool,
    outbox: E,
    journal: J,
    etablissements: R,
}

impl<E: OutboxWriter, J: JournalAudit, R: EstablishmentDirectory> ServiceRoles<E, J, R> {
    pub fn nouveau(pool: PgPool, outbox: E, journal: J, etablissements: R) -> Self {
        Self {
            pool,
            outbox,
            journal,
            etablissements,
        }
    }

    /// Attribue un rôle à un compte.
    ///
    /// Ordre des opérations, chacun pour une raison :
    ///
    ///   1. **portée du rôle** — un `422` sur un rôle d'éditeur avec établissement ne demande
    ///      aucune transaction ;
    ///   2. **existence de l'établissement, par trait** — avant d'écrire, pour donner un `404`
    ///      plutôt qu'une violation de contrainte ;
    ///   3. transaction, pose du tenant ;
    ///   4. existence du compte — dans la transaction, donc sous la politique d'isolation ;
    ///   5. insertion idempotente ;
    ///   6. **événement et entrée d'audit uniquement si la ligne vient d'être créée** ;
    ///   7. commit.
    ///
    /// Le point 6 est celui qu'on écrirait mal. Un rejeu — même identifiant, même triplet — ne
    /// produit **ni événement ni trace** : l'émettre à chaque tentative ferait du registre le
    /// journal des reprises réseau du terminal, et le registre a une rétention illimitée.
    #[tracing::instrument(
        skip(self, demande),
        fields(compte.id = %demande.compte_id, role = %demande.role_code, tenant.id = %tenant_id)
    )]
    pub async fn attribuer(
        &self,
        tenant_id: Uuid,
        auteur_compte_id: Uuid,
        demande: AttribuerRole,
    ) -> Result<Issue, ErreurRoles> {
        // 1 · La portée décide de l'obligation ou de l'interdiction d'`etablissement_id`.
        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, tenant_id).await?;

        let portee = repository::portee_du_role(&mut tx, &demande.role_code)
            .await?
            .ok_or_else(|| ErreurRoles::RoleInconnu(demande.role_code.clone()))?;

        if !portee.compatible(demande.etablissement_id) {
            return Err(ErreurRoles::PorteeIncompatible);
        }

        if !repository::compte_existe(&mut tx, demande.compte_id).await? {
            return Err(ErreurRoles::CompteInconnu);
        }

        // 2 · L'établissement se vérifie **par trait**. L'appel sort de la transaction courante et
        // ouvre la sienne : c'est une lecture d'un autre module, et lui passer notre transaction
        // reviendrait à lui donner accès au schéma `comptes`.
        if let Some(etablissement_id) = demande.etablissement_id
            && !self
                .etablissements
                .appartient_au_tenant(etablissement_id, tenant_id)
                .await
                .map_err(|_| ErreurRoles::EtablissementInconnu)?
        {
            return Err(ErreurRoles::EtablissementInconnu);
        }

        let issue = repository::attribuer(&mut tx, tenant_id, &demande, auteur_compte_id).await?;

        if issue == Issue::Creee {
            let evenement = evenement(tenant_id, TYPE_ROLE_ATTRIBUE, &demande, "attribution");
            self.outbox.ecrire(&mut tx, evenement).await?;

            self.journal
                .tracer(
                    &mut tx,
                    tenant_id,
                    entree_audit(auteur_compte_id, &demande, "attribution"),
                )
                .await?;
        }

        tx.commit().await?;
        Ok(issue)
    }

    /// Retire un rôle — **et refuse de retirer la dernière habilitation**.
    ///
    /// # Pourquoi le décompte se fait APRÈS le `DELETE`, dans la même transaction
    ///
    /// La forme évidente compte les comptes habilités avant, et refuse s'il n'en reste qu'un. Elle
    /// est fausse à cause du **cumul** : Adjoua est gérante et propriétaire, et retirer `gerant`
    /// ne lui retire pas `cpt.role.attribuer`. Un décompte préalable la compterait comme « la
    /// dernière » et refuserait un retrait parfaitement sûr.
    ///
    /// Supprimer d'abord puis compter répond aux deux cas d'un coup : le décompte porte sur l'état
    /// **résultant**. Si l'établissement se retrouve sans habilitation, la transaction est
    /// annulée — rien n'a eu lieu, et le refus est `409 derniere_habilitation`.
    ///
    /// C'est le seul refus métier du cycle, et il est irréversible sans l'éditeur : d'où un code
    /// propre plutôt qu'un `403`, qui aurait suggéré un problème de droits de l'appelant.
    #[tracing::instrument(
        skip(self),
        fields(compte.id = %compte_id, role = %role_code, tenant.id = %tenant_id)
    )]
    pub async fn retirer(
        &self,
        tenant_id: Uuid,
        auteur_compte_id: Uuid,
        compte_id: Uuid,
        role_code: &str,
        etablissement_id: Option<Uuid>,
    ) -> Result<(), ErreurRoles> {
        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, tenant_id).await?;

        let portee = repository::portee_du_role(&mut tx, role_code)
            .await?
            .ok_or_else(|| ErreurRoles::RoleInconnu(role_code.to_owned()))?;

        if !portee.compatible(etablissement_id) {
            return Err(ErreurRoles::PorteeIncompatible);
        }

        let retire = repository::retirer(&mut tx, compte_id, role_code, etablissement_id).await?;

        if !retire {
            // Le compte ne portait pas ce rôle. **Rien à tracer, et pas d'erreur** : un retrait
            // qui n'avait rien à retirer a atteint son but. Rendre `404` ferait échouer un rejeu
            // que le principe VI rend inévitable sur un réseau intermittent.
            tx.rollback().await?;
            return Ok(());
        }

        // FR-023 — le décompte porte sur l'état résultant, dans la transaction non validée.
        if let Some(etablissement_id) = etablissement_id
            && repository::role_habilite(&mut tx, role_code).await?
            && repository::comptes_habilites(&mut tx, etablissement_id).await? == 0
        {
            tx.rollback().await?;
            return Err(ErreurRoles::DerniereHabilitation);
        }

        let demande = AttribuerRole {
            // L'identifiant de l'événement et celui de l'entrée d'audit sont distincts de celui de
            // la ligne retirée — laquelle n'existe plus. Un UUID v7 neuf porte l'instant du
            // retrait, ce qui est ce que le registre doit dater.
            id: Uuid::now_v7(),
            compte_id,
            role_code: role_code.to_owned(),
            etablissement_id,
            horodatage_client: None,
        };

        let evenement = evenement(tenant_id, TYPE_ROLE_RETIRE, &demande, "retrait");
        self.outbox.ecrire(&mut tx, evenement).await?;

        self.journal
            .tracer(
                &mut tx,
                tenant_id,
                entree_audit(auteur_compte_id, &demande, "retrait"),
            )
            .await?;

        tx.commit().await?;
        Ok(())
    }

    /// Le référentiel des rôles — **le même pour les deux tenants**.
    pub async fn referentiel_roles(
        &self,
    ) -> Result<Vec<super::modele::EntreeReferentielRole>, ErreurRoles> {
        let mut tx = self.pool.begin().await?;
        let entrees = repository::referentiel_roles(&mut tx).await?;
        tx.rollback().await?;
        Ok(entrees)
    }

    /// Le référentiel des permissions — **le même pour les deux tenants**.
    pub async fn referentiel_permissions(
        &self,
    ) -> Result<Vec<super::modele::EntreeReferentielRole>, ErreurRoles> {
        let mut tx = self.pool.begin().await?;
        let entrees = repository::referentiel_permissions(&mut tx).await?;
        tx.rollback().await?;
        Ok(entrees)
    }
}

/// Construit l'événement — **charge utile complète et dénormalisée** (TRX-02, règle 2).
///
/// Un lecteur qui n'a que cette ligne doit pouvoir dire ce qui s'est passé sans consulter aucune
/// autre table. D'où `sens` en clair : sans lui, `role.attribue` et `role.retire` se distingueraient
/// par leur seul `type_evenement`, et un consommateur qui filtrerait mal les confondrait.
fn evenement(
    tenant_id: Uuid,
    type_evenement: &str,
    demande: &AttribuerRole,
    sens: &str,
) -> EvenementAEcrire {
    EvenementAEcrire {
        id: Uuid::now_v7(),
        tenant_id,
        etablissement_id: demande.etablissement_id,
        type_evenement: type_evenement.to_owned(),
        agregat: AGREGAT_COMPTE_ROLE.to_owned(),
        // L'agrégat est le **compte**, pas la ligne de rôle : c'est lui qui a une histoire, et une
        // ligne supprimée n'a plus d'identifiant à désigner.
        agregat_id: demande.compte_id,
        version_schema: VERSION_SCHEMA_ROLE,
        payload: json!({
            "compte_id": demande.compte_id,
            "role_code": demande.role_code,
            "etablissement_id": demande.etablissement_id,
            "sens": sens,
        }),
    }
}

/// Construit l'entrée d'audit `changement_role`.
///
/// **Aucune clé monétaire** : rien de ce que ce cycle trace ne porte de montant. La validation de
/// la porte P-10 étendue s'applique quand même — c'est elle qui garantit que le premier montant
/// écrit par un cycle ultérieur portera son suffixe et sa devise.
fn entree_audit(auteur_compte_id: Uuid, demande: &AttribuerRole, sens: &str) -> EntreeAudit {
    EntreeAudit {
        id: Uuid::now_v7(),
        etablissement_id: demande.etablissement_id,
        type_action: TypeActionAudit::ChangementRole,
        auteur_compte_id,
        cible_type: CIBLE_COMPTE.to_owned(),
        cible_id: Some(demande.compte_id),
        contexte: json!({
            "role_code": demande.role_code,
            "sens": sens,
            "etablissement_id": demande.etablissement_id,
        }),
        horodatage_client: demande.horodatage_client,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Les deux types d'événements sont **stables** : ils partent dans un grand livre permanent.
    #[test]
    fn les_types_d_evenements_ne_bougent_pas() {
        assert_eq!(TYPE_ROLE_ATTRIBUE, "role.attribue");
        assert_eq!(TYPE_ROLE_RETIRE, "role.retire");
        assert_eq!(AGREGAT_COMPTE_ROLE, "compte_role");
    }

    /// L'entrée d'audit ne porte **aucune clé monétaire**, et son contexte passe la validation de
    /// la porte P-10 étendue.
    #[test]
    fn le_contexte_d_audit_passe_la_validation_monetaire() {
        let demande = AttribuerRole {
            id: Uuid::now_v7(),
            compte_id: Uuid::now_v7(),
            role_code: "caissier".to_owned(),
            etablissement_id: Some(Uuid::now_v7()),
            horodatage_client: None,
        };

        let entree = entree_audit(Uuid::now_v7(), &demande, "attribution");

        assert_eq!(entree.type_action, TypeActionAudit::ChangementRole);
        assert_eq!(entree.cible_type, CIBLE_COMPTE);
        assert!(crate::audit::service::valider_contexte(&entree.contexte).is_ok());
    }

    /// La charge utile de l'événement dit le **sens** en clair.
    ///
    /// Sans lui, `role.attribue` et `role.retire` se distingueraient par leur seul type, et un
    /// consommateur qui filtrerait mal les confondrait — dans un registre à rétention illimitée,
    /// où l'on ne repasse pas corriger.
    #[test]
    fn la_charge_utile_porte_le_sens_en_clair() {
        let demande = AttribuerRole {
            id: Uuid::now_v7(),
            compte_id: Uuid::now_v7(),
            role_code: "gerant".to_owned(),
            etablissement_id: None,
            horodatage_client: None,
        };

        let attribution = evenement(Uuid::now_v7(), TYPE_ROLE_ATTRIBUE, &demande, "attribution");
        let retrait = evenement(Uuid::now_v7(), TYPE_ROLE_RETIRE, &demande, "retrait");

        assert_eq!(attribution.payload["sens"], "attribution");
        assert_eq!(retrait.payload["sens"], "retrait");
        assert_eq!(attribution.agregat_id, demande.compte_id);
    }
}
