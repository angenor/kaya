//! Couche service — **la transaction, et l'événement dans la transaction**.
//!
//! C'est la couche du patron qui porte la garantie la plus importante du produit :
//!
//! > Toute transition d'état écrit un événement outbox **dans la même transaction**
//! > (principe II, porte P-05).
//!
//! Ce n'est pas une discipline de rédaction. `OutboxWriter::ecrire` prend la transaction en
//! paramètre et n'en ouvre jamais une : écrire l'événement ailleurs qu'ici demanderait de
//! fabriquer une seconde transaction et de la passer explicitement, ce qui se voit en revue et ne
//! s'écrit pas par distraction.
//!
//! La conséquence est vérifiable : après un rollback provoqué, **ni ligne métier ni événement**.
//! `backend/tests/outbox_transactionnel.rs` le constate.

use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use kaya_synchronisation::{EvenementAEcrire, OutboxWriter};

use super::modele::{CreerNote, ErreurNote, Issue, NoteEtablissement};
use super::repository;
use crate::tenant_context;

/// Version du format de la charge utile de `note_etablissement.creee`.
///
/// **Toute évolution du format incrémente ce numéro** (R-06). En phase 2, la génération SYSCOHADA
/// rétroactive relira des événements écrits par des versions du code qui n'existeront plus ; sans
/// numéro, il faudrait deviner le format au lieu d'écrire un décodeur par génération.
pub const VERSION_SCHEMA_NOTE_CREEE: i16 = 1;

/// Type de l'unique événement métier de ce cycle.
pub const TYPE_NOTE_CREEE: &str = "note_etablissement.creee";

/// Nom de l'agrégat.
pub const AGREGAT_NOTE: &str = "note_etablissement";

/// Longueur maximale du texte — **doit rester alignée sur le `CHECK` de la migration 0004**.
///
/// La validation applicative existe pour renvoyer un `400` intelligible, pas pour remplacer la
/// contrainte de base : une migration de données ou un script de maintenance contournerait la
/// première, jamais la seconde.
pub const TEXTE_MAX: usize = 2000;

/// Service des notes internes.
pub struct ServiceNote<E: OutboxWriter> {
    pool: PgPool,
    outbox: E,
}

impl<E: OutboxWriter> ServiceNote<E> {
    pub fn nouveau(pool: PgPool, outbox: E) -> Self {
        Self { pool, outbox }
    }

    /// Crée une note **et** son événement, dans une seule transaction.
    ///
    /// Ordre des opérations, chacun pour une raison :
    ///
    ///   1. validation du texte — inutile d'ouvrir une transaction pour un texte vide ;
    ///   2. transaction, puis **pose du tenant courant** : sans elle, la politique de sécurité
    ///      ne verrait rien et l'insertion échouerait sur `WITH CHECK` ;
    ///   3. vérification de l'établissement — pour un `404` plutôt qu'une violation de clé ;
    ///   4. insertion idempotente ;
    ///   5. **événement, uniquement si la note vient d'être créée** ;
    ///   6. commit.
    ///
    /// Le point 5 est celui qu'on écrirait mal. Un rejeu ne produit **aucun** nouvel événement :
    /// l'émettre à chaque tentative ferait du grand livre le journal des tentatives réseau du
    /// terminal, et non celui des transitions d'état. Trois envois de la même note doivent
    /// laisser une ligne et un événement.
    #[tracing::instrument(skip(self, note), fields(note.id = %note.id, tenant.id = %tenant_id))]
    pub async fn creer(
        &self,
        tenant_id: Uuid,
        note: CreerNote,
    ) -> Result<(NoteEtablissement, Issue), ErreurNote> {
        let texte = note.texte.trim();
        if texte.is_empty() || texte.chars().count() > TEXTE_MAX {
            return Err(ErreurNote::TexteInvalide);
        }
        let note = CreerNote {
            texte: texte.to_owned(),
            ..note
        };

        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, tenant_id).await?;

        if !repository::etablissement_existe(&mut tx, note.etablissement_id).await? {
            return Err(ErreurNote::EtablissementInconnu);
        }

        let (enregistree, issue) = repository::inserer(&mut tx, tenant_id, &note).await?;

        if issue == Issue::Creee {
            // Charge utile **complète et dénormalisée** : un lecteur qui n'a que cette ligne doit
            // pouvoir dire ce qui s'est passé, sans consulter aucune autre table (R-11). D'où le
            // texte en clair plutôt qu'un renvoi vers `note_etablissement`.
            let evenement = EvenementAEcrire {
                id: Uuid::now_v7(),
                tenant_id,
                etablissement_id: Some(enregistree.etablissement_id),
                type_evenement: TYPE_NOTE_CREEE.to_owned(),
                agregat: AGREGAT_NOTE.to_owned(),
                agregat_id: enregistree.id,
                version_schema: VERSION_SCHEMA_NOTE_CREEE,
                payload: json!({
                    "note_id": enregistree.id,
                    "etablissement_id": enregistree.etablissement_id,
                    "auteur_compte_id": enregistree.auteur_compte_id,
                    "texte": enregistree.texte,
                    "cree_le": enregistree.cree_le.to_string(),
                }),
            };
            self.outbox.ecrire(&mut tx, evenement).await?;
        }

        tx.commit().await?;
        Ok((enregistree, issue))
    }

    /// Liste les notes d'un établissement.
    pub async fn lister(
        &self,
        tenant_id: Uuid,
        etablissement_id: Uuid,
        limite: i64,
        decalage: i64,
    ) -> Result<(Vec<NoteEtablissement>, i64), ErreurNote> {
        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, tenant_id).await?;

        // La politique de sécurité suffirait à ne rien renvoyer, mais une liste vide et un
        // établissement inexistant sont deux réponses différentes pour l'appelant. Sans cette
        // vérification, la lecture répondrait `200 []` là où la création répond `404` — deux
        // comportements pour la même situation, sur le même chemin.
        if !repository::etablissement_existe(&mut tx, etablissement_id).await? {
            return Err(ErreurNote::EtablissementInconnu);
        }

        let notes = repository::lister(&mut tx, etablissement_id, limite, decalage).await?;
        let total = repository::compter(&mut tx, etablissement_id).await?;

        tx.rollback().await?;
        Ok((notes, total))
    }
}
