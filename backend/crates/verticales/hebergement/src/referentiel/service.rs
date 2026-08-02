//! Couche service du référentiel — **la transaction, les validations que la base ne porte pas,
//! et l'événement dans la transaction**.
//!
//! # Les deux validations que la base ne peut pas exprimer
//!
//! Qu'une formule `PASSAGE` porte au moins un palier (FR-025) et qu'une `DEMI_JOURNEE` porte au
//! moins une plage (FR-033) ne s'écrit **pas** en contrainte de table : la dépendance va de
//! l'enfant au parent, et la ligne parente existe forcément avant ses enfants. Aucun `CHECK`,
//! aucun déclencheur raisonnable ne l'exprime sans se déclencher au mauvais moment.
//!
//! C'est donc ici, dans la transaction de création, et
//! `backend/tests/hebergement_referentiel.rs` le vérifie.
//!
//! # L'ordre des opérations, et le point qu'on écrirait mal
//!
//! 1. valider — inutile d'ouvrir une transaction pour une famille inconnue ;
//! 2. ouvrir la transaction, puis **poser le tenant courant** ;
//! 3. vérifier que le module `HEBERGEMENT` est actif — refus normalisé du cycle 002 ;
//! 4. insérer, idempotent ;
//! 5. **émettre l'événement uniquement si la ligne vient d'être créée** ;
//! 6. commit.
//!
//! Le point 5 est celui qu'on écrirait mal. Un rejeu ne produit **aucun** nouvel événement :
//! l'émettre à chaque tentative ferait du grand livre le journal des tentatives réseau du
//! terminal, et non celui des transitions d'état.

use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use kaya_etablissements::tenant_context;
use kaya_etablissements::{EstablishmentDirectory, RegistreModules};
use kaya_synchronisation::{EvenementAEcrire, OutboxWriter};

use super::modele::{
    CategorieVue, CreerCategorie, CreerFormule, CreerUnite, ErreurReferentiel, FamilleFormule,
    FormuleVue, ModifierCategorie, ModifierFormule, ModifierUnite, UniteVue,
};
use super::repository;
use crate::{Issue, MODULE_HEBERGEMENT};

/// Version du format des charges utiles de ce cycle.
///
/// **Toute évolution du format l'incrémente** (R-06) : la génération SYSCOHADA rétroactive relira
/// des événements écrits par des versions du code qui n'existeront plus.
pub const VERSION_SCHEMA_HEB: i16 = 1;

pub const TYPE_FORMULE_CREEE: &str = "heb.formule.creee";
pub const TYPE_FORMULE_MODIFIEE: &str = "heb.formule.modifiee";
pub const TYPE_CATEGORIE_TARIF_MODIFIE: &str = "heb.categorie.tarif_modifie";

pub const AGREGAT_FORMULE: &str = "hebergement.formule";
pub const AGREGAT_CATEGORIE: &str = "hebergement.categorie";

/// Longueur maximale d'un nom de catégorie et d'un code d'unité.
///
/// La validation applicative existe pour renvoyer un refus intelligible, pas pour remplacer une
/// contrainte de base : un script de maintenance contournerait la première, jamais la seconde.
/// Ici la base n'en pose aucune — le nom est libre —, donc cette borne est la seule, et elle est
/// large : un type de chambre s'appelle « Supérieure vue lagune », pas plus.
pub const NOM_MAX: usize = 120;
pub const CODE_MAX: usize = 32;

/// Service du référentiel d'hébergement.
pub struct ServiceReferentiel<E, A, R>
where
    E: OutboxWriter,
    A: EstablishmentDirectory,
    R: RegistreModules,
{
    pool: PgPool,
    outbox: E,
    annuaire: A,
    modules: R,
}

impl<E, A, R> ServiceReferentiel<E, A, R>
where
    E: OutboxWriter,
    A: EstablishmentDirectory,
    R: RegistreModules,
{
    pub fn nouveau(pool: PgPool, outbox: E, annuaire: A, modules: R) -> Self {
        Self {
            pool,
            outbox,
            annuaire,
            modules,
        }
    }

    // =============================================================================================
    //  Garde commune — l'établissement existe, et le module y est actif
    // =============================================================================================

    /// Vérifie l'établissement **par le trait**, jamais par jointure, et rend sa devise.
    ///
    /// Deux refus distincts, et la distinction compte pour l'écran :
    /// `etablissement_inconnu` est un `404` — il n'y a rien à voir ; `service_inactif` est un
    /// `409` — l'établissement existe, mais il ne fait pas d'hébergement, et l'interface doit
    /// alors proposer d'ajouter le service plutôt qu'afficher une erreur.
    async fn garde(&self, etablissement_id: Uuid) -> Result<String, ErreurReferentiel> {
        let etablissement = self
            .annuaire
            .etablissement(etablissement_id)
            .await?
            .ok_or(ErreurReferentiel::EtablissementInconnu)?;

        if !self
            .modules
            .module_actif(etablissement_id, MODULE_HEBERGEMENT)
            .await?
        {
            return Err(ErreurReferentiel::ServiceInactif);
        }

        Ok(etablissement.devise)
    }

    // =============================================================================================
    //  Catégories
    // =============================================================================================

    pub async fn lister_categories(
        &self,
        tenant_id: Uuid,
        etablissement_id: Uuid,
    ) -> Result<Vec<CategorieVue>, ErreurReferentiel> {
        self.garde(etablissement_id).await?;

        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, tenant_id).await?;
        let categories = repository::lister_categories(&mut tx, etablissement_id).await?;
        // Lecture : la transaction est **annulée**. Un `commit` sur une transaction sans écriture
        // ne dirait rien de plus tout en laissant croire le contraire.
        tx.rollback().await?;
        Ok(categories)
    }

    /// Crée un type de chambre, avec ses battements de remise en état.
    ///
    /// **Aucun événement outbox.** Une catégorie n'est pas une transition d'état du produit : elle
    /// n'a d'effet ni monétaire, ni fiscal, ni sur la disponibilité. Le modèle de données en
    /// déclare cinq, et `heb.categorie.tarif_modifie` — le seul qui la concerne — est émis par la
    /// modification d'un **prix**, pas par la création d'un type de chambre. En émettre un ici
    /// gonflerait le grand livre d'un fait que personne ne rejouera.
    pub async fn creer_categorie(
        &self,
        tenant_id: Uuid,
        demande: CreerCategorie,
    ) -> Result<(CategorieVue, Issue), ErreurReferentiel> {
        let nom = demande.nom.trim().to_owned();
        if nom.is_empty() || nom.chars().count() > NOM_MAX {
            return Err(ErreurReferentiel::ChampNonModifiable("nom".to_owned()));
        }
        let demande = CreerCategorie { nom, ..demande };

        self.garde(demande.etablissement_id).await?;

        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, tenant_id).await?;

        let creee = repository::inserer_categorie(&mut tx, tenant_id, &demande).await?;
        if creee {
            repository::remplacer_temps_remise(
                &mut tx,
                tenant_id,
                demande.id,
                &demande.temps_remise_en_etat,
            )
            .await?;
        }

        let vue = repository::lire_categorie(&mut tx, demande.id)
            .await?
            .ok_or(ErreurReferentiel::CategorieInconnue)?;

        tx.commit().await?;

        Ok((
            vue,
            if creee {
                Issue::Creee
            } else {
                Issue::DejaPresente
            },
        ))
    }

    pub async fn modifier_categorie(
        &self,
        tenant_id: Uuid,
        etablissement_id: Uuid,
        categorie_id: Uuid,
        changements: ModifierCategorie,
    ) -> Result<CategorieVue, ErreurReferentiel> {
        let nom = changements.nom.trim().to_owned();
        if nom.is_empty() || nom.chars().count() > NOM_MAX {
            return Err(ErreurReferentiel::ChampNonModifiable("nom".to_owned()));
        }
        let changements = ModifierCategorie { nom, ..changements };

        self.garde(etablissement_id).await?;

        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, tenant_id).await?;

        if !repository::modifier_categorie(&mut tx, categorie_id, &changements).await? {
            return Err(ErreurReferentiel::CategorieInconnue);
        }
        repository::remplacer_temps_remise(
            &mut tx,
            tenant_id,
            categorie_id,
            &changements.temps_remise_en_etat,
        )
        .await?;

        let vue = repository::lire_categorie(&mut tx, categorie_id)
            .await?
            .ok_or(ErreurReferentiel::CategorieInconnue)?;

        tx.commit().await?;
        Ok(vue)
    }

    // =============================================================================================
    //  Unités
    // =============================================================================================

    pub async fn lister_unites(
        &self,
        tenant_id: Uuid,
        etablissement_id: Uuid,
    ) -> Result<Vec<UniteVue>, ErreurReferentiel> {
        self.garde(etablissement_id).await?;

        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, tenant_id).await?;
        let unites = repository::lister_unites(&mut tx, etablissement_id).await?;
        tx.rollback().await?;
        Ok(unites)
    }

    pub async fn creer_unite(
        &self,
        tenant_id: Uuid,
        demande: CreerUnite,
    ) -> Result<(UniteVue, Issue), ErreurReferentiel> {
        let code = demande.code.trim().to_owned();
        if code.is_empty() || code.chars().count() > CODE_MAX {
            return Err(ErreurReferentiel::ChampNonModifiable("code".to_owned()));
        }
        let demande = CreerUnite { code, ..demande };

        self.garde(demande.etablissement_id).await?;

        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, tenant_id).await?;

        // La catégorie est vérifiée avant l'insertion pour rendre un `404` intelligible plutôt
        // qu'une violation de clé étrangère.
        if repository::lire_categorie(&mut tx, demande.categorie_id)
            .await?
            .is_none()
        {
            return Err(ErreurReferentiel::CategorieInconnue);
        }

        let creee = repository::inserer_unite(&mut tx, tenant_id, &demande).await?;
        let vue = repository::lire_unite(&mut tx, demande.id)
            .await?
            .ok_or(ErreurReferentiel::UniteInconnue)?;

        tx.commit().await?;

        Ok((
            vue,
            if creee {
                Issue::Creee
            } else {
                Issue::DejaPresente
            },
        ))
    }

    /// **Corrige `code` et `etage`, et rien d'autre.**
    ///
    /// Sans cette opération, une unité mal nommée puis occupée deviendrait définitive : la
    /// suppression est impossible dès qu'une occupation la référence.
    pub async fn modifier_unite(
        &self,
        tenant_id: Uuid,
        etablissement_id: Uuid,
        unite_id: Uuid,
        changements: ModifierUnite,
    ) -> Result<UniteVue, ErreurReferentiel> {
        let code = changements.code.trim().to_owned();
        if code.is_empty() || code.chars().count() > CODE_MAX {
            return Err(ErreurReferentiel::ChampNonModifiable("code".to_owned()));
        }
        let changements = ModifierUnite { code, ..changements };

        self.garde(etablissement_id).await?;

        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, tenant_id).await?;

        if !repository::modifier_unite(&mut tx, unite_id, &changements).await? {
            return Err(ErreurReferentiel::UniteInconnue);
        }
        let vue = repository::lire_unite(&mut tx, unite_id)
            .await?
            .ok_or(ErreurReferentiel::UniteInconnue)?;

        tx.commit().await?;
        Ok(vue)
    }

    /// Combien d'unités porte une catégorie — **le refus nomme ce qui occupe**.
    ///
    /// Aucun endpoint ne supprime une catégorie à ce cycle (contrat §1 : neuf opérations, aucune
    /// `DELETE`). Cette lecture existe parce que le refus doit pouvoir se composer le jour où la
    /// suppression se spécifiera, et parce que `backend/tests/hebergement_referentiel.rs` constate
    /// **aujourd'hui** qu'une catégorie occupée ne se supprime pas — par la clé étrangère.
    ///
    /// Elle est appelée par ce test ; elle n'est pas du code mort en attente d'un cycle.
    pub async fn unites_de_categorie(
        &self,
        tenant_id: Uuid,
        categorie_id: Uuid,
    ) -> Result<i64, ErreurReferentiel> {
        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, tenant_id).await?;
        let total = repository::compter_unites_de_categorie(&mut tx, categorie_id).await?;
        tx.rollback().await?;
        Ok(total)
    }

    // =============================================================================================
    //  Formules — et les deux validations que la base ne porte pas
    // =============================================================================================

    pub async fn lister_formules(
        &self,
        tenant_id: Uuid,
        etablissement_id: Uuid,
    ) -> Result<Vec<FormuleVue>, ErreurReferentiel> {
        let devise = self.garde(etablissement_id).await?;

        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, tenant_id).await?;
        let formules = repository::lister_formules(&mut tx, etablissement_id, &devise).await?;
        tx.rollback().await?;
        Ok(formules)
    }

    /// **FR-025 et FR-033 — les deux validations que la base ne peut pas exprimer.**
    fn valider_enfants(
        famille: FamilleFormule,
        paliers: usize,
        plages: usize,
    ) -> Result<(), ErreurReferentiel> {
        match famille {
            FamilleFormule::Passage if paliers == 0 => Err(ErreurReferentiel::BaremeAbsent),
            FamilleFormule::DemiJournee if plages == 0 => Err(ErreurReferentiel::PlagesAbsentes),
            _ => Ok(()),
        }
    }

    pub async fn creer_formule(
        &self,
        tenant_id: Uuid,
        demande: CreerFormule,
    ) -> Result<(FormuleVue, Issue), ErreurReferentiel> {
        Self::valider_enfants(
            demande.famille,
            demande.paliers.len(),
            demande.plages.len(),
        )?;

        let devise = self.garde(demande.etablissement_id).await?;

        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, tenant_id).await?;

        if repository::lire_categorie(&mut tx, demande.categorie_id)
            .await?
            .is_none()
        {
            return Err(ErreurReferentiel::CategorieInconnue);
        }

        let creee = repository::inserer_formule(&mut tx, tenant_id, &demande).await?;
        if creee {
            repository::remplacer_paliers(&mut tx, tenant_id, demande.id, &demande.paliers).await?;
            repository::remplacer_plages(&mut tx, tenant_id, demande.id, &demande.plages).await?;
        }

        let vue = repository::lire_formule(&mut tx, demande.id, &devise)
            .await?
            .ok_or(ErreurReferentiel::FormuleInconnue)?;

        // **Événement uniquement à la création.** Un rejeu n'en produit aucun.
        if creee {
            self.emettre(
                &mut tx,
                tenant_id,
                demande.etablissement_id,
                TYPE_FORMULE_CREEE,
                AGREGAT_FORMULE,
                vue.id,
                json!({
                    "formule_id": vue.id,
                    "categorie_id": vue.categorie_id,
                    "famille": vue.famille,
                    // **Nommage monétaire réservé (P-10)** : `prix_mineur` entier et `devise` au
                    // même niveau. Jamais `prix`, `montant` ni `total` nus.
                    "prix_mineur": vue.prix_mineur,
                    "devise": vue.devise,
                    "assujettie_taxe_nuitee": vue.assujettie_taxe_nuitee,
                    "regle_conversion_taxe": vue.regle_conversion_taxe,
                }),
            )
            .await?;
        }

        tx.commit().await?;

        Ok((
            vue,
            if creee {
                Issue::Creee
            } else {
                Issue::DejaPresente
            },
        ))
    }

    /// Modifie une formule — **c'est ici que l'exploitant règle la taxe et le prix**.
    ///
    /// Deux événements peuvent partir, et ils disent deux choses différentes :
    ///
    /// - `heb.formule.modifiee` — **toujours**, avec les champs changés ;
    /// - `heb.categorie.tarif_modifie` — **seulement si le prix a bougé**, avec l'avant et
    ///   l'après. C'est celui que la reconstitution financière lira ; le noyer dans le premier
    ///   obligerait un lecteur à comparer deux charges utiles pour savoir si un tarif a changé.
    pub async fn modifier_formule(
        &self,
        tenant_id: Uuid,
        etablissement_id: Uuid,
        formule_id: Uuid,
        changements: ModifierFormule,
    ) -> Result<FormuleVue, ErreurReferentiel> {
        let devise = self.garde(etablissement_id).await?;

        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, tenant_id).await?;

        let avant = repository::lire_formule(&mut tx, formule_id, &devise)
            .await?
            .ok_or(ErreurReferentiel::FormuleInconnue)?;

        // La famille n'est pas modifiable : la validation porte donc sur celle qui est en base,
        // pas sur une famille reçue — il n'y en a pas.
        Self::valider_enfants(
            avant.famille,
            changements.paliers.len(),
            changements.plages.len(),
        )?;

        if !repository::modifier_formule(&mut tx, formule_id, &changements).await? {
            return Err(ErreurReferentiel::FormuleInconnue);
        }
        repository::remplacer_paliers(&mut tx, tenant_id, formule_id, &changements.paliers).await?;
        repository::remplacer_plages(&mut tx, tenant_id, formule_id, &changements.plages).await?;

        let apres = repository::lire_formule(&mut tx, formule_id, &devise)
            .await?
            .ok_or(ErreurReferentiel::FormuleInconnue)?;

        self.emettre(
            &mut tx,
            tenant_id,
            etablissement_id,
            TYPE_FORMULE_MODIFIEE,
            AGREGAT_FORMULE,
            formule_id,
            json!({
                "formule_id": formule_id,
                "categorie_id": apres.categorie_id,
                "famille": apres.famille,
                "prix_mineur": apres.prix_mineur,
                "devise": apres.devise,
                "assujettie_taxe_nuitee": apres.assujettie_taxe_nuitee,
                "regle_conversion_taxe": apres.regle_conversion_taxe,
                "paliers": apres.paliers,
            }),
        )
        .await?;

        if avant.prix_mineur != apres.prix_mineur {
            self.emettre(
                &mut tx,
                tenant_id,
                etablissement_id,
                TYPE_CATEGORIE_TARIF_MODIFIE,
                AGREGAT_CATEGORIE,
                apres.categorie_id,
                json!({
                    "formule_id": formule_id,
                    "categorie_id": apres.categorie_id,
                    "famille": apres.famille,
                    "prix_avant_mineur": avant.prix_mineur,
                    "prix_apres_mineur": apres.prix_mineur,
                    "devise": apres.devise,
                }),
            )
            .await?;
        }

        tx.commit().await?;
        Ok(apres)
    }

    // =============================================================================================
    //  L'écriture au grand livre — une seule forme
    // =============================================================================================

    /// Écrit un événement **dans la transaction fournie**.
    ///
    /// La signature d'`OutboxWriter::ecrire` prend la transaction et n'en ouvre jamais une :
    /// écrire l'événement ailleurs demanderait de fabriquer une seconde transaction et de la
    /// passer explicitement, ce qui se voit en revue et ne s'écrit pas par distraction.
    #[allow(clippy::too_many_arguments)]
    async fn emettre(
        &self,
        tx: &mut sqlx::PgTransaction<'_>,
        tenant_id: Uuid,
        etablissement_id: Uuid,
        type_evenement: &str,
        agregat: &str,
        agregat_id: Uuid,
        payload: serde_json::Value,
    ) -> Result<(), ErreurReferentiel> {
        self.outbox
            .ecrire(
                tx,
                EvenementAEcrire {
                    id: Uuid::now_v7(),
                    tenant_id,
                    etablissement_id: Some(etablissement_id),
                    type_evenement: type_evenement.to_owned(),
                    agregat: agregat.to_owned(),
                    agregat_id,
                    version_schema: VERSION_SCHEMA_HEB,
                    payload,
                },
            )
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **FR-025** — un passage sans palier ne sait rien facturer.
    #[test]
    fn un_passage_sans_palier_est_refuse() {
        let erreur =
            ServiceReferentiel::<
                kaya_synchronisation::outbox::PgOutboxWriter,
                kaya_etablissements::etablissement::PgEstablishmentDirectory,
                kaya_etablissements::modules::PgRegistreModules,
            >::valider_enfants(FamilleFormule::Passage, 0, 0)
            .unwrap_err();
        assert_eq!(erreur.code(), "bareme_absent");
    }

    /// **FR-033** — une demi-journée sans plage n'a rien à vendre.
    #[test]
    fn une_demi_journee_sans_plage_est_refusee() {
        let erreur =
            ServiceReferentiel::<
                kaya_synchronisation::outbox::PgOutboxWriter,
                kaya_etablissements::etablissement::PgEstablishmentDirectory,
                kaya_etablissements::modules::PgRegistreModules,
            >::valider_enfants(FamilleFormule::DemiJournee, 0, 0)
            .unwrap_err();
        assert_eq!(erreur.code(), "plages_absentes");
    }

    /// **La nuitée et le mensuel n'ont ni palier ni plage, et c'est normal.** Une validation trop
    /// large refuserait la formule la plus courante du produit.
    #[test]
    fn la_nuitee_et_le_mensuel_n_exigent_ni_palier_ni_plage() {
        for famille in [FamilleFormule::Nuitee, FamilleFormule::Mensuel] {
            assert!(
                ServiceReferentiel::<
                    kaya_synchronisation::outbox::PgOutboxWriter,
                    kaya_etablissements::etablissement::PgEstablishmentDirectory,
                    kaya_etablissements::modules::PgRegistreModules,
                >::valider_enfants(famille, 0, 0)
                .is_ok(),
                "{famille:?} ne doit exiger ni palier ni plage"
            );
        }
    }
}
