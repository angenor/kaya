//! Couche service — **la transaction, et l'événement dans la transaction**.
//!
//! > Toute transition d'état écrit un événement outbox **dans la même transaction**
//! > (principe II, porte P-05).
//!
//! La garantie tient à une signature, pas à une discipline : `OutboxWriter::ecrire` prend la
//! transaction et n'en ouvre jamais une.
//!
//! # Le point qu'on écrirait mal, et qui vaut d'être répété ici
//!
//! **L'événement n'est émis que si la ligne vient d'être créée.** Un rejeu n'en produit aucun.
//! L'émettre à chaque tentative ferait du grand livre le journal des tentatives réseau du
//! terminal — et le grand livre a une rétention illimitée.

use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use kaya_etablissements::tenant_context;
use kaya_etablissements::Issue;
use kaya_synchronisation::{EvenementAEcrire, OutboxWriter};

use super::modele::{CreerPersonne, ErreurPersonne, ModifierPersonne, NOM_MAX, Personne};
use super::repository;

/// Nom de l'agrégat au grand livre.
pub const AGREGAT_PERSONNE: &str = "personne";

/// Types d'événements — nomenclature `agregat.action`, comme les onze du cycle 001.
pub const TYPE_PERSONNE_CREEE: &str = "personne.creee";
pub const TYPE_PERSONNE_MODIFIEE: &str = "personne.modifiee";

/// Version du format des charges utiles ci-dessous.
///
/// **Toute évolution du format l'incrémente.** La génération SYSCOHADA rétroactive relira un jour
/// des événements écrits par des versions du code qui n'existeront plus ; sans numéro, il
/// faudrait deviner le format au lieu d'écrire un décodeur par génération.
pub const VERSION_SCHEMA_PERSONNE: i16 = 1;

/// Service de l'identité civile.
pub struct ServicePersonne<E: OutboxWriter> {
    pool: PgPool,
    outbox: E,
}

impl<E: OutboxWriter> ServicePersonne<E> {
    pub fn nouveau(pool: PgPool, outbox: E) -> Self {
        Self { pool, outbox }
    }

    /// Crée une personne **et** son événement, dans une seule transaction.
    ///
    /// Ordre des opérations, chacun pour une raison :
    ///
    ///   1. valider — inutile d'ouvrir une transaction pour un nom vide ;
    ///   2. transaction, puis **pose du tenant courant** ;
    ///   3. insertion idempotente ;
    ///   4. **événement uniquement si la ligne vient d'être créée** ;
    ///   5. commit.
    ///
    /// Il n'y a **pas d'étape « vérifier l'agrégat parent »** ici, contrairement au module doré :
    /// une personne n'a pas de parent. Elle appartient au tenant, que la politique de sécurité
    /// impose déjà.
    #[tracing::instrument(skip(self, demande), fields(personne.id = %demande.id, tenant.id = %tenant_id))]
    pub async fn creer(
        &self,
        tenant_id: Uuid,
        demande: CreerPersonne,
    ) -> Result<(Personne, Issue), ErreurPersonne> {
        let demande = CreerPersonne {
            nom: nom_valide(&demande.nom)?,
            prenoms: normaliser(demande.prenoms),
            telephone: normaliser(demande.telephone),
            email: normaliser(demande.email),
            ..demande
        };

        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, tenant_id).await?;

        let (enregistree, issue) = repository::inserer(&mut tx, tenant_id, &demande).await?;

        if issue == Issue::Creee {
            let evenement = self.evenement(tenant_id, TYPE_PERSONNE_CREEE, &enregistree);
            self.outbox.ecrire(&mut tx, evenement).await?;
        }

        tx.commit().await?;
        Ok((enregistree, issue))
    }

    /// Lit une personne.
    pub async fn lire(&self, tenant_id: Uuid, id: Uuid) -> Result<Personne, ErreurPersonne> {
        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, tenant_id).await?;

        let personne = repository::lire(&mut tx, id).await?;

        tx.rollback().await?;
        personne.ok_or(ErreurPersonne::Inconnue)
    }

    /// Modifie une personne **et** émet `personne.modifiee`.
    ///
    /// Contrairement à la création, **l'événement est toujours émis** quand la ligne existe : une
    /// modification n'a pas d'identifiant client propre, donc aucune idempotence à préserver. Un
    /// second `PUT` identique est une seconde transition d'état, même si l'état final est le
    /// même — et le registre des actions doit pouvoir dire qui a touché la fiche, et quand.
    #[tracing::instrument(skip(self, modification), fields(personne.id = %id, tenant.id = %tenant_id))]
    pub async fn modifier(
        &self,
        tenant_id: Uuid,
        id: Uuid,
        modification: ModifierPersonne,
    ) -> Result<Personne, ErreurPersonne> {
        let modification = ModifierPersonne {
            nom: nom_valide(&modification.nom)?,
            prenoms: normaliser(modification.prenoms),
            telephone: normaliser(modification.telephone),
            email: normaliser(modification.email),
            ..modification
        };

        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, tenant_id).await?;

        let modifiee = repository::modifier(&mut tx, id, &modification)
            .await?
            .ok_or(ErreurPersonne::Inconnue)?;

        let evenement = self.evenement(tenant_id, TYPE_PERSONNE_MODIFIEE, &modifiee);
        self.outbox.ecrire(&mut tx, evenement).await?;

        tx.commit().await?;
        Ok(modifiee)
    }

    /// Construit l'événement — **charge utile complète et dénormalisée** (TRX-02, règle 2).
    ///
    /// Un lecteur qui n'a que cette ligne doit pouvoir dire ce qui s'est passé, sans consulter
    /// aucune autre table. D'où le nom en clair plutôt qu'un renvoi vers `comptes.personne`.
    ///
    /// **Portée tenant** : `etablissement_id` vaut `None`. Une personne n'appartient pas à un
    /// établissement — c'est son `employe` qui en désignera un, et c'est une autre table.
    fn evenement(&self, tenant_id: Uuid, type_evenement: &str, personne: &Personne) -> EvenementAEcrire {
        EvenementAEcrire {
            id: Uuid::now_v7(),
            tenant_id,
            etablissement_id: None,
            type_evenement: type_evenement.to_owned(),
            agregat: AGREGAT_PERSONNE.to_owned(),
            agregat_id: personne.id,
            version_schema: VERSION_SCHEMA_PERSONNE,
            payload: json!({
                "personne_id": personne.id,
                "nom": personne.nom,
                "prenoms": personne.prenoms,
                "telephone": personne.telephone,
                "email": personne.email,
                "cree_le": personne.cree_le.to_string(),
                "modifie_le": personne.modifie_le.to_string(),
            }),
        }
    }
}

/// Nettoie et valide le nom.
fn nom_valide(brut: &str) -> Result<String, ErreurPersonne> {
    let nom = brut.trim();
    if nom.is_empty() || nom.chars().count() > NOM_MAX {
        return Err(ErreurPersonne::NomInvalide);
    }
    Ok(nom.to_owned())
}

/// Ramène une chaîne vide à `None`.
///
/// Sans cela, un formulaire dont le champ n'a pas été rempli enverrait `""`, et la base porterait
/// une chaîne vide indistinguable d'un numéro absent — puis `compte_identifiant_telephone_unique`
/// refuserait le **deuxième** compte sans téléphone.
fn normaliser(valeur: Option<String>) -> Option<String> {
    valeur
        .map(|v| v.trim().to_owned())
        .filter(|v| !v.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn un_nom_vide_ou_blanc_est_refuse() {
        assert!(nom_valide("").is_err());
        assert!(nom_valide("   ").is_err());
    }

    #[test]
    fn un_nom_trop_long_est_refuse_au_meme_seuil_que_la_base() {
        let long = "a".repeat(NOM_MAX + 1);
        assert!(nom_valide(&long).is_err());
        assert!(nom_valide(&"a".repeat(NOM_MAX)).is_ok());
    }

    /// Le nom est **nettoyé**, pas seulement validé.
    #[test]
    fn un_nom_est_nettoye_de_ses_blancs() {
        assert_eq!(nom_valide("  Kouassi  ").unwrap(), "Kouassi");
    }

    /// La chaîne vide devient `None` — sans quoi le second compte sans téléphone serait refusé.
    #[test]
    fn une_chaine_vide_devient_absente() {
        assert_eq!(normaliser(Some(String::new())), None);
        assert_eq!(normaliser(Some("   ".to_owned())), None);
        assert_eq!(normaliser(Some(" +2250700000001 ".to_owned())), Some("+2250700000001".to_owned()));
        assert_eq!(normaliser(None), None);
    }
}
