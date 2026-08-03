//! **Le câblage du constat de dérive** — là où le socle ne peut pas le faire.
//!
//! # Pourquoi ce fichier est dans `api/` et pas dans `socle/synchronisation`
//!
//! C'est une contrainte de hiérarchie, pas une préférence. L'ordre réel des dépendances est :
//!
//! ```text
//! socle/synchronisation  ←  socle/etablissements  ←  socle/comptes
//!    (outbox, dérive)         (tenant_context)        (JournalAudit)
//! ```
//!
//! `JournalAudit` vit dans `comptes`, **qui dépend de `synchronisation`**. Faire écrire l'audit par
//! le crate de la dérive créerait un cycle — refusé par le compilateur, et par la porte **P-03**
//! avant lui.
//!
//! `synchronisation` expose donc la fonction pure `constater_derive()` et le trait `SignalDerive` ;
//! **la couche API, qui connaît tout le monde, câble l'un sur l'autre**. C'est le montage déjà
//! éprouvé d'`OutboxWriter` et d'`EstablishmentDirectory`.
//!
//! # Ce que cette implémentation ajoute au trait, et qui ne pouvait pas être ailleurs
//!
//! | Ce qu'elle apporte | Pourquoi ici |
//! |---|---|
//! | Le **seuil**, lu de la configuration d'établissement | Le résolveur vit dans `etablissements` |
//! | Le **débrayage par épisode**, en Redis | Le client Redis est dans l'état applicatif |
//! | L'**écriture au registre**, dans sa propre transaction | `JournalAudit` vit dans `comptes` |

use std::time::Duration;

use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use kaya_comptes::audit::{EntreeAudit, JournalAudit, TypeActionAudit};
use kaya_etablissements::ResolveurConfiguration;
use kaya_etablissements::tenant_context;
use kaya_synchronisation::derive::{
    cle_debrayage, Derive, OrigineDerive, SignalDerive, CLE_SEUIL_DERIVE, SEUIL_DERIVE_DEFAUT,
};

/// Combien de temps un épisode de dérive reste consigné avant qu'une entrée nouvelle soit écrite.
///
/// **Quatre heures** — l'ordre de grandeur d'un service. Deux cents saisies pendant un service ne
/// produisent donc qu'une entrée, et une horloge encore fausse le lendemain matin en produit une
/// autre : c'est ce qu'un exploitant veut voir, puisque cela dit que rien n'a été corrigé.
///
/// Ce n'est **pas** un paramètre d'établissement, et ce n'est pas un oubli : ce nombre ne règle
/// rien de métier, il règle le bruit d'un registre. Le rendre configurable inviterait à le mettre
/// à zéro pour « tout voir », ce qui rendrait le registre illisible — exactement le défaut que le
/// débrayage existe pour empêcher.
const DUREE_EPISODE: Duration = Duration::from_secs(4 * 3600);

/// Le signal de dérive du produit : configuration → débrayage → registre.
pub struct SignalDeriveApplicatif<J: JournalAudit, R: ResolveurConfiguration> {
    pool: PgPool,
    journal: J,
    configuration: R,
    redis: redis::Client,
}

impl<J: JournalAudit, R: ResolveurConfiguration> SignalDeriveApplicatif<J, R> {
    pub fn nouveau(pool: PgPool, journal: J, configuration: R, redis: redis::Client) -> Self {
        Self {
            pool,
            journal,
            configuration,
            redis,
        }
    }

    /// Pose la clé d'épisode **si elle n'existe pas**, et dit si c'est nous qui l'avons posée.
    ///
    /// `SET … NX EX` est atomique : deux terminaux qui signalent au même instant ne produisent
    /// qu'une entrée, sans verrou applicatif. C'est le même raisonnement que la contrainte
    /// d'exclusion des occupations — la garantie appartient au magasin, pas au code qui l'appelle.
    ///
    /// **Rend `true` en cas de doute.** Redis injoignable, réponse illisible : une entrée de trop
    /// est un bruit ; une entrée manquante est une information perdue dans un registre immuable,
    /// où rien ne se rattrape après coup. Le principe II autorise Redis ici précisément parce que
    /// la perte de la clé est sans conséquence — c'est de l'éphémère reconstructible.
    async fn premier_du_episode(&self, origine: OrigineDerive) -> bool {
        let Ok(mut connexion) = self.redis.get_multiplexed_async_connection().await else {
            return true;
        };

        let pose: Result<Option<String>, redis::RedisError> = redis::cmd("SET")
            .arg(cle_debrayage(origine))
            .arg("1")
            .arg("NX")
            .arg("EX")
            .arg(DUREE_EPISODE.as_secs())
            .query_async(&mut connexion)
            .await;

        match pose {
            // `SET … NX` rend `OK` si la clé a été posée, `nil` si elle existait déjà.
            Ok(Some(_)) => true,
            Ok(None) => false,
            Err(_) => true,
        }
    }
}

#[async_trait::async_trait]
impl<J: JournalAudit, R: ResolveurConfiguration> SignalDerive for SignalDeriveApplicatif<J, R> {
    async fn seuil(&self, tenant_id: Uuid, etablissement_id: Uuid) -> Duration {
        let cible = kaya_etablissements::Cible {
            tenant_id,
            etablissement_id: Some(etablissement_id),
            module_code: None,
            point_de_vente_id: None,
        };

        // **Le défaut du catalogue quand la valeur ne se lit pas.** Refuser de constater parce
        // qu'un paramètre manque désactiverait le signalement au moment où la base est le plus
        // perturbée — c'est-à-dire quand il sert le plus.
        match self.configuration.resoudre(&cible, CLE_SEUIL_DERIVE).await {
            Ok(Some(resolue)) => resolue
                .valeur
                .as_u64()
                .map(Duration::from_secs)
                .unwrap_or(SEUIL_DERIVE_DEFAUT),
            _ => SEUIL_DERIVE_DEFAUT,
        }
    }

    async fn consigner(&self, origine: OrigineDerive, derive: Derive) {
        if !self.premier_du_episode(origine).await {
            // L'épisode est déjà consigné. Deux cents saisies pendant un service ne doivent pas
            // produire deux cents entrées identiques : un registre noyé n'est plus lu.
            return;
        }

        let entree = EntreeAudit {
            id: Uuid::now_v7(),
            // **Portée tenant, pas établissement.** C'est un terminal qui dévie, et le même
            // terminal sert parfois deux établissements. Le rattacher à l'un des deux ferait
            // disparaître le constat du filtre de l'autre.
            etablissement_id: None,
            type_action: TypeActionAudit::DeriveHorlogeConstatee,
            auteur_compte_id: origine.compte_id,
            cible_type: "appareil".to_owned(),
            cible_id: origine.appareil_id,
            // **Aucune clé monétaire** : la porte P-10 inspecte le JSONB, et ce contexte décrit un
            // temps. Le seuil est reproduit pour que l'entrée se lise seule dans dix ans, quand la
            // configuration aura changé.
            contexte: json!({
                "ecart_secondes": derive.ecart_secondes,
                "seuil_secondes": derive.seuil_secondes,
                "sens": derive.sens.code(),
            }),
            // Le constat n'a pas d'horodatage client : c'est le serveur qui constate.
            horodatage_client: None,
        };

        // Sa propre transaction, courte. Un échec ici **ne remonte pas** : l'écriture qui a révélé
        // la dérive est déjà commitée, et le trait ne rend rien précisément pour qu'aucun appelant
        // ne soit tenté de l'`?`.
        let Ok(mut tx) = self.pool.begin().await else {
            tracing::warn!("dérive constatée mais non consignée : transaction indisponible");
            return;
        };
        if tenant_context::poser_tenant(&mut tx, origine.tenant_id)
            .await
            .is_err()
        {
            tracing::warn!("dérive constatée mais non consignée : contexte de tenant refusé");
            return;
        }
        if let Err(erreur) = self.journal.tracer(&mut tx, origine.tenant_id, entree).await {
            tracing::warn!(%erreur, "dérive constatée mais non consignée");
            return;
        }
        if tx.commit().await.is_err() {
            tracing::warn!("dérive constatée mais non consignée : commit refusé");
        }
    }
}
