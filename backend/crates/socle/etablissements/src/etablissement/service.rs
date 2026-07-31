//! Couche service de l'établissement — **la transaction, et l'événement dans la transaction**.
//!
//! > Toute transition d'état écrit un événement outbox **dans la même transaction**
//! > (principe II, porte P-05).
//!
//! Ce n'est pas une discipline de rédaction : `OutboxWriter::ecrire` prend la transaction en
//! paramètre et n'en ouvre jamais une. Écrire l'événement ailleurs demanderait de fabriquer une
//! seconde transaction et de la passer explicitement — ce qui se voit en revue et ne s'écrit pas
//! par distraction.
//!
//! # Deux changements ont leur PROPRE type d'événement
//!
//! Ils pourraient tenir dans `etablissement.modifie`, et c'est justement le problème : ils y
//! seraient noyés.
//!
//! - **`classement_change`** — le classement décide du barème de la taxe communale de nuitée. Un
//!   changement rétroactif fausserait un reversement, et il faut pouvoir le retrouver sans relire
//!   toutes les modifications ;
//! - **`fuseau_change`** — il réinterprète **tout regroupement par journée locale**. Une clôture
//!   journalière déjà produite ne couvre plus la même période.
//!
//! Aucune table d'historique : le journal d'événements du cycle 001 la remplace — immuable,
//! rétention illimitée, déjà en place.

use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use kaya_synchronisation::{EvenementAEcrire, OutboxWriter};

use super::modele::{
    Changements, CreerEtablissement, ErreurEtablissement, EtablissementVue, ModifierEtablissement,
};
use super::repository;
use crate::{Classement, Issue, tenant_context};

/// Version du format des charges utiles de ce sous-module.
///
/// **Toute évolution du format l'incrémente** : en phase 2, la génération SYSCOHADA rétroactive
/// relira des événements écrits par des versions du code qui n'existeront plus.
pub const VERSION_SCHEMA: i16 = 1;

pub const AGREGAT: &str = "etablissement";

pub const TYPE_CREE: &str = "etablissement.cree";
pub const TYPE_MODIFIE: &str = "etablissement.modifie";
pub const TYPE_CLASSEMENT_CHANGE: &str = "etablissement.classement_change";
pub const TYPE_FUSEAU_CHANGE: &str = "etablissement.fuseau_change";

/// Longueur maximale du nom. **Doit rester alignée sur la validation de la couche HTTP** — la
/// base ne la contraint pas, un nom n'ayant pas de longueur légale.
pub const NOM_MAX: usize = 200;

/// Avertissement rendu à l'appelant, que l'interface **doit** présenter avant de confirmer.
pub const AVERTISSEMENT_FUSEAU: &str = "fuseau_change";

/// Service des établissements.
pub struct ServiceEtablissement<E: OutboxWriter> {
    pool: PgPool,
    outbox: E,
}

/// Résultat d'une modification — la vue à jour, et ce qu'il faut dire à l'opérateur.
pub struct IssueModification {
    pub etablissement: EtablissementVue,
    /// `Some("fuseau_change")` quand le fuseau a changé. **Non pas un message**, un code : la
    /// phrase vit dans le catalogue i18n (porte P-16).
    pub avertissement: Option<&'static str>,
}

impl<E: OutboxWriter> ServiceEtablissement<E> {
    pub fn nouveau(pool: PgPool, outbox: E) -> Self {
        Self { pool, outbox }
    }

    /// Crée un établissement **et** son événement, dans une seule transaction.
    ///
    /// Ordre des opérations du module doré, chacun pour une raison :
    ///
    ///   1. valider — inutile d'ouvrir une transaction pour un nom vide ;
    ///   2. transaction, puis **pose du tenant courant** : sans elle, la politique de sécurité ne
    ///      verrait rien et l'insertion échouerait sur `WITH CHECK` ;
    ///   3. insertion idempotente ;
    ///   4. **événement, uniquement si la ligne vient d'être créée** ;
    ///   5. commit.
    ///
    /// **Le point 4 est celui qu'on écrirait mal.** Un rejeu ne produit aucun nouvel événement :
    /// l'émettre à chaque tentative ferait du grand livre le journal des tentatives réseau du
    /// terminal, et non celui des transitions d'état.
    #[tracing::instrument(skip(self, demande), fields(etablissement.id = %demande.id, tenant.id = %tenant_id))]
    pub async fn creer(
        &self,
        tenant_id: Uuid,
        demande: CreerEtablissement,
    ) -> Result<(EtablissementVue, Issue), ErreurEtablissement> {
        let demande = valider_creation(demande)?;

        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, tenant_id).await?;

        let (vue, issue) = repository::inserer(&mut tx, tenant_id, &demande).await?;

        if issue == Issue::Creee {
            // Charge utile **complète et dénormalisée** : un lecteur qui n'a que cette ligne doit
            // pouvoir dire ce qui s'est passé sans consulter aucune autre table.
            let evenement = EvenementAEcrire {
                id: Uuid::now_v7(),
                tenant_id,
                etablissement_id: Some(vue.id),
                type_evenement: TYPE_CREE.to_owned(),
                agregat: AGREGAT.to_owned(),
                agregat_id: vue.id,
                version_schema: VERSION_SCHEMA,
                payload: json!({
                    "etablissement_id": vue.id,
                    "nom": vue.nom,
                    "juridiction": vue.juridiction,
                    "classement": vue.classement,
                    "etoiles": vue.etoiles,
                    "commune": vue.commune,
                    "fuseau_horaire": vue.fuseau_horaire,
                    "devise": vue.devise,
                    "adresse": vue.adresse,
                    "ncc": vue.ncc,
                    "cree_le": vue.cree_le.to_string(),
                }),
            };
            self.outbox.ecrire(&mut tx, evenement).await?;
        }

        tx.commit().await?;
        Ok((vue, issue))
    }

    /// Modifie un établissement, et émet **un à trois** événements selon ce qui a changé.
    ///
    /// Une modification qui ne change rien n'émet **aucun** événement : le grand livre enregistre
    /// les transitions d'état, pas les requêtes reçues. C'est le même principe que « aucun
    /// événement sur rejeu ».
    #[tracing::instrument(skip(self, demande), fields(etablissement.id = %id, tenant.id = %tenant_id))]
    pub async fn modifier(
        &self,
        tenant_id: Uuid,
        id: Uuid,
        demande: ModifierEtablissement,
    ) -> Result<IssueModification, ErreurEtablissement> {
        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, tenant_id).await?;

        let avant = repository::lire(&mut tx, id)
            .await?
            .ok_or(ErreurEtablissement::Inconnu)?;

        let classement_avant = avant
            .classement_type()
            .ok_or(ErreurEtablissement::ClassementIncoherent(
                avant.classement.clone(),
            ))?;

        let nom = demande.nom.clone().unwrap_or_else(|| avant.nom.clone());
        let classement = demande.classement.unwrap_or(classement_avant);
        let commune = demande
            .commune
            .clone()
            .unwrap_or_else(|| avant.commune.clone());
        let fuseau = demande
            .fuseau_horaire
            .clone()
            .unwrap_or_else(|| avant.fuseau_horaire.clone());
        let devise = demande
            .devise
            .clone()
            .unwrap_or_else(|| avant.devise.clone());
        let adresse = demande.adresse.clone().or_else(|| avant.adresse.clone());
        let ncc = demande.ncc.clone().or_else(|| avant.ncc.clone());

        valider_champs(&nom, &commune, &fuseau, &devise, ncc.as_deref())?;

        // **`devise_figee`.** Le contrôle est posé maintenant et branché par CAI : ajouté après
        // coup, il arriverait une fois qu'un établissement aura changé de devise en production,
        // laissant des montants dont on ne sait plus dans quelle unité ils sont libellés.
        if devise != avant.devise
            && repository::compter_operations_financieres(&mut tx, id).await? > 0
        {
            return Err(ErreurEtablissement::DeviseFigee);
        }

        let mut changements = Changements::default();
        if nom != avant.nom {
            changements.ajouter("nom", avant.nom.clone(), nom.clone());
        }
        if commune != avant.commune {
            changements.ajouter("commune", avant.commune.clone(), commune.clone());
        }
        if devise != avant.devise {
            changements.ajouter("devise", avant.devise.clone(), devise.clone());
        }
        if adresse != avant.adresse {
            changements.ajouter(
                "adresse",
                json!(avant.adresse.clone()),
                json!(adresse.clone()),
            );
        }
        if ncc != avant.ncc {
            changements.ajouter("ncc", json!(avant.ncc.clone()), json!(ncc.clone()));
        }

        let classement_a_change = classement != classement_avant;
        let fuseau_a_change = fuseau != avant.fuseau_horaire;

        if changements.est_vide() && !classement_a_change && !fuseau_a_change {
            // Rien n'a changé : ni écriture, ni événement. `modifie_le` n'est pas touché — le
            // bouger sans changement ferait croire à une modification qui n'a pas eu lieu.
            tx.rollback().await?;
            return Ok(IssueModification {
                etablissement: avant,
                avertissement: None,
            });
        }

        let apres = repository::modifier(
            &mut tx,
            id,
            &nom,
            classement,
            &commune,
            &fuseau,
            &devise,
            adresse.as_deref(),
            ncc.as_deref(),
        )
        .await?;

        if !changements.est_vide() {
            self.emettre(
                &mut tx,
                tenant_id,
                id,
                TYPE_MODIFIE,
                json!({
                    "etablissement_id": id,
                    "champs": changements.en_json(),
                }),
            )
            .await?;
        }

        if classement_a_change {
            self.emettre(
                &mut tx,
                tenant_id,
                id,
                TYPE_CLASSEMENT_CHANGE,
                json!({
                    "etablissement_id": id,
                    "avant": { "classement": classement_avant.code(), "etoiles": classement_avant.etoiles() },
                    "apres": { "classement": classement.code(), "etoiles": classement.etoiles() },
                    "consequence": "bareme_taxe_nuitee",
                }),
            )
            .await?;
        }

        if fuseau_a_change {
            self.emettre(
                &mut tx,
                tenant_id,
                id,
                TYPE_FUSEAU_CHANGE,
                json!({
                    "etablissement_id": id,
                    "avant": avant.fuseau_horaire,
                    "apres": fuseau,
                    // L'événement enregistre l'avertissement **présenté** — non pas qu'il ait été
                    // affiché, mais qu'il faisait partie de l'opération.
                    "avertissement": AVERTISSEMENT_FUSEAU,
                    "consequence": "regroupement_par_journee_locale",
                }),
            )
            .await?;
        }

        tx.commit().await?;

        Ok(IssueModification {
            etablissement: apres,
            avertissement: fuseau_a_change.then_some(AVERTISSEMENT_FUSEAU),
        })
    }

    /// Liste les établissements du tenant.
    pub async fn lister(
        &self,
        tenant_id: Uuid,
    ) -> Result<Vec<EtablissementVue>, ErreurEtablissement> {
        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, tenant_id).await?;
        let liste = repository::lister(&mut tx).await?;
        tx.rollback().await?;
        Ok(liste)
    }

    /// Lit un établissement du tenant.
    pub async fn lire(
        &self,
        tenant_id: Uuid,
        id: Uuid,
    ) -> Result<EtablissementVue, ErreurEtablissement> {
        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, tenant_id).await?;
        let vue = repository::lire(&mut tx, id).await?;
        tx.rollback().await?;
        vue.ok_or(ErreurEtablissement::Inconnu)
    }

    async fn emettre(
        &self,
        tx: &mut sqlx::PgTransaction<'_>,
        tenant_id: Uuid,
        etablissement_id: Uuid,
        type_evenement: &str,
        payload: serde_json::Value,
    ) -> Result<(), ErreurEtablissement> {
        self.outbox
            .ecrire(
                tx,
                EvenementAEcrire {
                    id: Uuid::now_v7(),
                    tenant_id,
                    etablissement_id: Some(etablissement_id),
                    type_evenement: type_evenement.to_owned(),
                    agregat: AGREGAT.to_owned(),
                    agregat_id: etablissement_id,
                    version_schema: VERSION_SCHEMA,
                    payload,
                },
            )
            .await?;
        Ok(())
    }
}

fn valider_creation(
    demande: CreerEtablissement,
) -> Result<CreerEtablissement, ErreurEtablissement> {
    let nom = demande.nom.trim().to_owned();
    let commune = demande.commune.trim().to_owned();
    let ncc = demande.ncc.map(|v| v.trim().to_owned());
    let adresse = demande
        .adresse
        .map(|v| v.trim().to_owned())
        .filter(|v| !v.is_empty());

    valider_champs(
        &nom,
        &commune,
        &demande.fuseau_horaire,
        &demande.devise,
        ncc.as_deref(),
    )?;

    Ok(CreerEtablissement {
        nom,
        commune,
        ncc: ncc.filter(|v| !v.is_empty()),
        adresse,
        ..demande
    })
}

fn valider_champs(
    nom: &str,
    commune: &str,
    fuseau: &str,
    devise: &str,
    ncc: Option<&str>,
) -> Result<(), ErreurEtablissement> {
    if nom.trim().is_empty() || nom.chars().count() > NOM_MAX {
        return Err(ErreurEtablissement::NomInvalide);
    }
    if commune.trim().is_empty() {
        return Err(ErreurEtablissement::CommuneInvalide);
    }
    if !fuseau_connu(fuseau) {
        return Err(ErreurEtablissement::FuseauInconnu(fuseau.to_owned()));
    }
    if !devise_valide(devise) {
        return Err(ErreurEtablissement::DeviseInvalide(devise.to_owned()));
    }
    if ncc.is_some_and(|v| v.trim().is_empty()) {
        return Err(ErreurEtablissement::NccInvalide);
    }
    Ok(())
}

/// Le fuseau est-il un identifiant IANA plausible ?
///
/// # Une vérification de forme, et elle est nommée comme telle
///
/// Charger la base IANA complète pour valider un champ saisi une fois par établissement coûterait
/// une dépendance et une mise à jour trimestrielle. Ce qui est refusé ici, ce sont les fautes de
/// saisie qui produiraient des calculs de journée locale absurdes : chaîne vide, absence de
/// région, espaces. Un identifiant retiré de la base IANA passerait — et c'est un défaut connu,
/// pas un oubli.
fn fuseau_connu(fuseau: &str) -> bool {
    let fuseau = fuseau.trim();
    if fuseau == "UTC" {
        return true;
    }
    let Some((region, ville)) = fuseau.split_once('/') else {
        return false;
    };
    !region.is_empty()
        && !ville.is_empty()
        && region.chars().next().is_some_and(char::is_uppercase)
        && fuseau
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '/' | '_' | '-' | '+'))
}

/// Code ISO 4217 — **trois lettres majuscules**.
///
/// Aucune liste fermée ici : la validité d'une devise dans une juridiction donnée relève du
/// `JurisdictionAdapter` (principe V, porte P-12). Ce qui est vérifié est la **forme**, qui vaut
/// partout.
fn devise_valide(devise: &str) -> bool {
    devise.len() == 3 && devise.chars().all(|c| c.is_ascii_uppercase())
}

/// Reconstruit un [`Classement`] depuis le couple `(code, étoiles)` d'un corps de requête.
///
/// **C'est ici que naît `classement_incoherent`** — le `422` du contrat HTTP. La base porte la
/// même règle par égalité de conditions ; la refuser d'abord ici donne un message qui nomme la
/// valeur, au lieu d'une violation de contrainte.
pub fn classement_depuis_requete(
    code: &str,
    etoiles: Option<u8>,
) -> Result<Classement, ErreurEtablissement> {
    match (code, etoiles) {
        ("ETOILES", Some(n)) if n > 0 => Ok(Classement::Etoiles(n)),
        ("ETOILES", _) => Err(ErreurEtablissement::ClassementIncoherent(
            "ETOILES sans nombre d'étoiles strictement positif".to_owned(),
        )),
        ("NON_CLASSE", None) => Ok(Classement::NonClasse),
        ("RESIDENCE_MEUBLEE", None) => Ok(Classement::ResidenceMeublee),
        ("NON_CLASSE" | "RESIDENCE_MEUBLEE", Some(_)) => Err(
            ErreurEtablissement::ClassementIncoherent(format!("{code} avec un nombre d'étoiles")),
        ),
        _ => Err(ErreurEtablissement::ClassementIncoherent(code.to_owned())),
    }
}
