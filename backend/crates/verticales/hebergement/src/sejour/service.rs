//! ★ **Le service du séjour — le cœur du cycle 006.**
//!
//! # Une transaction, cinq écritures, un appel réseau
//!
//! ```text
//! POST /etablissements/{id}/sejours          ← UN appel, UNE transaction
//!    ├─ attribuer l'unité      (MoteurDisponibilite::attribuer — PREND la transaction)
//!    ├─ ouvrir le séjour       (+ pose de sejour_id sur l'occupation)
//!    ├─ ouvrir la note + sa ligne d'hébergement
//!    ├─ numéroter et produire la fiche de police
//!    └─ écrire l'événement outbox
//! ```
//!
//! C'est ce qui tient le budget de FR-031 : **au plus un appel réseau bloquant** entre le premier
//! geste et la confirmation. Le cadrage §5.6 en fait une condition d'existence du produit — *« le
//! module de passage doit être irréprochable en rapidité sinon il sera contourné »*.
//!
//! # ⚠️ Trois règles que ce fichier ne contourne jamais
//!
//! 1. **Tenter l'insertion et traduire la violation** — jamais lire d'abord pour décider. Une
//!    lecture préalable « cette chambre est-elle libre ? » paraîtrait prudente et rendrait la
//!    double attribution *improbable* au lieu d'*impossible*. La garantie est
//!    `occupation_sans_chevauchement`, et ce service la **traduit** sans la remplacer.
//! 2. **Aucune règle fiscale.** Ce module lit `assujettie_taxe_nuitee` et `regle_conversion_taxe`
//!    pour les **recopier** au constat ; il ne les interprète jamais (porte P-12).
//! 3. **Aucun numéro de pièce dans l'outbox.** Le grand livre est à rétention illimitée et
//!    immuable : une donnée sensible qui y entre ne peut jamais en sortir, et la rétention de
//!    90 jours de TRX-06 deviendrait inapplicable.

use rust_decimal::Decimal;
use serde_json::json;
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use kaya_comptes::AnnuaireClients;
use kaya_etablissements::tenant_context;
use kaya_etablissements::{EstablishmentDirectory, RegistreModules};
use kaya_synchronisation::{EvenementAEcrire, OutboxWriter};

use super::modele::{
    Accompagnant, IssueAccompagnant, NoteVue, NouvelAccompagnant, OuvrirSejour, Sejour,
    SejourOuvert, SejourVue, StatutSejour,
};
use super::repository;
use crate::erreurs::ErreurSejour;
use crate::note::{NouvelleLigne, repository as note_repo};
use crate::occupation::{DemandeAttribution, service::ServiceOccupation};
use crate::police::repository as police_repo;
use crate::referentiel::FamilleFormule;
use crate::{Issue, MODULE_HEBERGEMENT};

/// Nom de l'agrégat au grand livre.
pub const AGREGAT_SEJOUR: &str = "hebergement.sejour";
pub const AGREGAT_FICHE_POLICE: &str = "hebergement.fiche_police";
pub const AGREGAT_ACCOMPAGNANT: &str = "hebergement.accompagnant";

/// Types d'événements — nomenclature `agregat.action`.
pub const TYPE_SEJOUR_OUVERT: &str = "heb.sejour.ouvert";
pub const TYPE_FICHE_POLICE_GENEREE: &str = "heb.fiche_police.generee";
pub const TYPE_ACCOMPAGNANT_AJOUTE: &str = "sej.accompagnant.ajoute";

/// Version du format des charges utiles.
pub const VERSION_SCHEMA_SEJOUR: i16 = 1;

/// Clé i18n de la ligne d'hébergement — **jamais un libellé rendu** (porte P-16).
///
/// Écrire « Nuit du lun. 24 au mar. 25 » en base rendrait la note monolingue à jamais, et la
/// chaîne échapperait entièrement au contrôle des littéraux.
const LIBELLE_HEBERGEMENT: &str = "hebergement.note.ligne.hebergement";

/// Service du séjour.
///
/// # Pourquoi il porte tant de collaborateurs
///
/// Chacun sert **une** règle qu'on ne peut pas tenir sans lui :
///
/// | Collaborateur | Ce qu'il empêche |
/// |---|---|
/// | `occupation` | Une attribution par lecture préalable — le verrou applicatif du principe IV |
/// | `annuaire_clients` | Une jointure `hebergement × comptes` (porte P-04) |
/// | `annuaire` · `modules` | Un maquis qui ouvrirait un séjour sans module hébergement |
/// | `outbox` | Un événement écrit hors de la transaction (porte P-05) |
pub struct ServiceSejour<E, A, R, C>
where
    E: OutboxWriter + Clone,
    A: EstablishmentDirectory + Clone,
    R: RegistreModules + Clone,
    C: AnnuaireClients,
{
    pool: PgPool,
    tenant_id: Uuid,
    outbox: E,
    annuaire: A,
    modules: R,
    annuaire_clients: C,
    occupation: ServiceOccupation<E, A, R>,
}

impl<E, A, R, C> ServiceSejour<E, A, R, C>
where
    E: OutboxWriter + Clone,
    A: EstablishmentDirectory + Clone,
    R: RegistreModules + Clone,
    C: AnnuaireClients,
{
    pub fn nouveau(
        pool: PgPool,
        tenant_id: Uuid,
        outbox: E,
        annuaire: A,
        modules: R,
        annuaire_clients: C,
    ) -> Self {
        let occupation = ServiceOccupation::nouveau(
            pool.clone(),
            tenant_id,
            outbox.clone(),
            annuaire.clone(),
            modules.clone(),
        );
        Self {
            pool,
            tenant_id,
            outbox,
            annuaire,
            modules,
            annuaire_clients,
            occupation,
        }
    }

    /// L'établissement existe, et l'hébergement y est actif.
    ///
    /// **Sans elle, un maquis pourrait ouvrir un séjour.** Le module d'activité est ce qui décide
    /// de ce qu'un établissement rend ; un séjour dans un bar seul n'a aucun sens et produirait
    /// une fiche de police que personne n'a demandée.
    async fn garde(&self, etablissement_id: Uuid) -> Result<(), ErreurSejour> {
        self.annuaire
            .etablissement(etablissement_id)
            .await
            .map_err(|e| ErreurSejour::Annuaire(e.to_string()))?
            .ok_or(ErreurSejour::EtablissementInconnu)?;

        if !self
            .modules
            .module_actif(etablissement_id, MODULE_HEBERGEMENT)
            .await
            .map_err(|e| ErreurSejour::Annuaire(e.to_string()))?
        {
            return Err(ErreurSejour::ServiceInactif);
        }
        Ok(())
    }

    // =============================================================================================
    //  ★ OUVRIR — une transaction, cinq écritures, dans cet ordre
    // =============================================================================================

    /// Ouvre un séjour.
    ///
    /// # L'ordre des cinq écritures, et pourquoi il est celui-là
    ///
    /// 1. **`MoteurDisponibilite::attribuer`** — l'attribution vient **en premier** parce que
    ///    c'est elle qui peut échouer sur la contrainte d'exclusion. Écrire le séjour avant
    ///    obligerait à annuler quatre écritures pour un refus qui se produit à chaque chambre
    ///    disputée.
    /// 2. **`INSERT sejour`** et pose de `sejour_id` sur l'occupation.
    /// 3. **`INSERT note_sejour`** et sa ligne d'hébergement au tarif du barème.
    /// 4. **Numérotation puis `INSERT fiche_police`** — `complete = false` sans client rattaché.
    /// 5. **`OutboxWriter::ecrire`** — deux événements, `heb.sejour.ouvert` et
    ///    `heb.fiche_police.generee`.
    ///
    /// # L'identifiant vient du terminal, et c'est ce qui rend le rejeu inoffensif
    ///
    /// UUID v7 fourni par le client (FR-086). **Le serveur déduplique, il n'engendre pas** : un
    /// terminal qui rejoue sa file après une coupure reçoit `200` avec la ligne en base, jamais
    /// `409`.
    ///
    /// ⚠️ **Les deux `409` ne se confondent pas** : même `id` → `200` ; `id` différent sur un
    /// intervalle chevauchant → `409 unite_deja_occupee`. C'est la distinction posée au cycle 004,
    /// reprise telle quelle.
    #[tracing::instrument(skip(self, demande), fields(sejour.id = %demande.id, tenant.id = %self.tenant_id))]
    pub async fn ouvrir(
        &self,
        demande: OuvrirSejour,
    ) -> Result<(SejourOuvert, Issue), ErreurSejour> {
        self.garde(demande.etablissement_id).await?;

        // Le refus d'un `client_id` inventé se fait **avant** la transaction : la clé étrangère
        // étant impossible entre deux schémas (principe II), aucune contrainte de base ne peut le
        // tenir. La politique de sécurité empêcherait la lecture d'un client d'un autre tenant,
        // mais laisserait passer une ligne orpheline pour un identifiant qui n'existe nulle part.
        if let Some(client_id) = demande.client_id
            && !self
                .annuaire_clients
                .existe(self.tenant_id, client_id)
                .await
                .map_err(|e| ErreurSejour::Annuaire(e.to_string()))?
        {
            return Err(ErreurSejour::ClientInconnu);
        }

        let etablissement = self
            .annuaire
            .etablissement(demande.etablissement_id)
            .await
            .map_err(|e| ErreurSejour::Annuaire(e.to_string()))?
            .ok_or(ErreurSejour::EtablissementInconnu)?;

        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, self.tenant_id).await?;

        let resultat = self
            .ouvrir_dans(&mut tx, &demande, &etablissement.devise)
            .await;

        match resultat {
            Ok(valeur) => {
                tx.commit().await?;
                Ok(valeur)
            }
            Err(erreur) => {
                // Une violation de contrainte **empoisonne** la transaction : le `rollback` est
                // obligatoire, et son échec ne doit pas masquer l'erreur métier.
                let _ = tx.rollback().await;
                Err(erreur)
            }
        }
    }

    /// Le corps de l'ouverture, **dans la transaction fournie**.
    ///
    /// Séparé de [`Self::ouvrir`] pour une raison de test : `sejour_arrivee.rs` simule une panne
    /// **après l'attribution** et vérifie qu'il ne reste ni séjour, ni note, ni fiche de police,
    /// **ni occupation orpheline**. Une fonction qui commiterait elle-même rendrait ce test
    /// impossible à écrire.
    async fn ouvrir_dans(
        &self,
        tx: &mut sqlx::PgTransaction<'_>,
        demande: &OuvrirSejour,
        devise: &str,
    ) -> Result<(SejourOuvert, Issue), ErreurSejour> {
        // ── 1 · ATTRIBUER — tenter et traduire, jamais lire d'abord ───────────────────────────
        let (occupation, issue_occupation) = self
            .occupation
            .attribuer_dans(
                tx,
                DemandeAttribution {
                    // ⚠️ **L'occupation porte son PROPRE identifiant, dérivé de celui du séjour.**
                    //
                    // Réutiliser l'identifiant du séjour ferait collisionner les deux tables le
                    // jour où un séjour porterait deux occupations — ce que le changement d'unité
                    // produit. Le dériver plutôt que le tirer garde le **rejeu** inoffensif : le
                    // même séjour rejoué demande la même occupation.
                    id: identifiant_occupation(demande.id, 0),
                    etablissement_id: demande.etablissement_id,
                    unite_id: demande.unite_id,
                    formule_id: demande.formule_id,
                    debut_client: demande.debut_client,
                    fin_client: demande.fin_client,
                },
            )
            .await?;

        // ── 2 · LE SÉJOUR ─────────────────────────────────────────────────────────────────────
        let cree = repository::inserer(
            tx,
            self.tenant_id,
            demande.id,
            demande.etablissement_id,
            demande.client_id,
            demande.horodatage_client,
        )
        .await?;

        // **Rejeu complet** : ni l'occupation ni le séjour n'ont été créés. On rend l'état en base
        // sans rien réécrire et **sans émettre aucun événement** — un rejeu n'est pas une
        // transition d'état, et le grand livre a une rétention illimitée.
        if !cree && issue_occupation == Issue::DejaPresente {
            let vue = self.composer_ouvert(tx, demande.id, occupation).await?;
            return Ok((vue, Issue::DejaPresente));
        }

        repository::rattacher_occupation(tx, occupation.id, demande.id).await?;

        // ── 3 · LA NOTE ET SA LIGNE D'HÉBERGEMENT ─────────────────────────────────────────────
        let note_id = identifiant_note(demande.id);
        note_repo::ouvrir(tx, self.tenant_id, note_id, demande.id, devise).await?;

        let (quantite, prix_unitaire_mineur, montant_mineur) = self
            .tarif_prevu(tx, demande.formule_id, demande.debut_client, demande.fin_client)
            .await?;

        note_repo::ajouter_ligne(
            tx,
            self.tenant_id,
            note_id,
            &NouvelleLigne {
                id: identifiant_ligne(demande.id, 0),
                occupation_id: Some(occupation.id),
                nature: "hebergement",
                // **`None` : une ligne d'hébergement ordinaire n'a pas de motif.** La contrainte
                // `ligne_ajustement_motive` refuse un motif posé ici.
                motif: None,
                libelle_cle: LIBELLE_HEBERGEMENT.to_owned(),
                quantite,
                prix_unitaire_mineur,
                montant_mineur,
                devise: devise.to_owned(),
                periode_debut: Some(demande.debut_client),
                periode_fin: Some(demande.fin_client),
            },
        )
        .await?;

        // ── 3 bis · LES ACCOMPAGNANTS, dans la MÊME transaction ───────────────────────────────
        //
        // Un accompagnant déclaré à l'arrivée et perdu par un second appel manqué ferait une fiche
        // de police **fausse** — un document légal qui omet une personne déclarée.
        for accompagnant in &demande.accompagnants {
            repository::ajouter_accompagnant(tx, self.tenant_id, demande.id, accompagnant).await?;
        }

        // ── 4 · LA NUMÉROTATION, PUIS LA FICHE DE POLICE ──────────────────────────────────────
        //
        // L'`UPDATE … RETURNING` du compteur pose un **verrou de ligne** : deux arrivées
        // simultanées sur le même établissement s'attendent ici, et aucune ne reçoit le numéro de
        // l'autre. C'est la définition même de la classe B.
        let numero = police_repo::numero_suivant(tx, self.tenant_id, demande.etablissement_id)
            .await?;

        // **`complete = false` sans client rattaché** (FR-047). La fiche existe et est numérotée ;
        // aucun champ de remplissage n'y figure. Ni fabriquée, ni silencieusement omise.
        let complete = demande.client_id.is_some();
        police_repo::ecrire(
            tx,
            self.tenant_id,
            identifiant_fiche(demande.id),
            demande.etablissement_id,
            demande.id,
            numero,
            complete,
        )
        .await?;

        // ── 5 · LES ÉVÉNEMENTS, DANS LA TRANSACTION ───────────────────────────────────────────
        let vue = self.composer_ouvert(tx, demande.id, occupation).await?;

        // ⚠️ **Charge utile financière complète et dénormalisée** (TRX-02) : l'opération se
        // reconstitue sans consulter aucune autre table. Les montants sont des **entiers d'unité
        // mineure** sous le nommage réservé `<nom>_mineur`, avec la devise au même niveau — ce que
        // `scripts/ci/types-monetaires.sh` inspecte **jusque dans le JSONB**.
        //
        // ⚠️ **Aucun numéro de pièce d'identité**, ni du titulaire ni des accompagnants.
        self.emettre(
            tx,
            demande.etablissement_id,
            TYPE_SEJOUR_OUVERT,
            AGREGAT_SEJOUR,
            demande.id,
            json!({
                "sejour_id": demande.id,
                "client_id": demande.client_id,
                "unite_id": vue.occupation.unite_id,
                "formule_id": vue.occupation.formule_id,
                "debut_client": vue.occupation.debut_client.to_string(),
                "fin_client": vue.occupation.fin_client.to_string(),
                "nombre_personnes": 1 + demande.accompagnants.len(),
                "note_id": vue.note.id,
                "total_mineur": vue.note.total_mineur,
                "devise": vue.note.devise,
                "lignes": vue.note.lignes.iter().map(|l| json!({
                    "ligne_id": l.id,
                    "nature": l.nature,
                    "libelle_cle": l.libelle_cle,
                    "quantite": l.quantite,
                    "prix_unitaire_mineur": l.prix_unitaire_mineur,
                    "montant_mineur": l.montant_mineur,
                    "devise": l.devise,
                })).collect::<Vec<_>>(),
            }),
        )
        .await?;

        self.emettre(
            tx,
            demande.etablissement_id,
            TYPE_FICHE_POLICE_GENEREE,
            AGREGAT_FICHE_POLICE,
            vue.fiche_police.id,
            json!({
                "fiche_police_id": vue.fiche_police.id,
                "sejour_id": demande.id,
                "numero": vue.fiche_police.numero,
                "complete": vue.fiche_police.complete,
            }),
        )
        .await?;

        Ok((vue, Issue::Creee))
    }

    /// Le tarif **prévu** de la période demandée — ce que la ligne initiale porte.
    ///
    /// # Ce n'est PAS le `MoteurTarification`, et la distinction compte
    ///
    /// `MoteurTarification::calculer` décide du montant **réel** depuis `now()` et l'instant
    /// d'ouverture : c'est ce que le **départ** consomme, rebascule de palier comprise. Ici on
    /// écrit ce qui est **vendu à l'arrivée**, sur la période demandée. Les confondre ferait
    /// facturer zéro minute à l'ouverture, la durée écoulée y étant nulle.
    ///
    /// Le barème est **réemployé**, jamais réimplémenté : `bareme::calculer` est la seule
    /// implémentation du produit.
    async fn tarif_prevu(
        &self,
        tx: &mut sqlx::PgTransaction<'_>,
        formule_id: Uuid,
        debut: OffsetDateTime,
        fin: OffsetDateTime,
    ) -> Result<(Decimal, i64, i64), ErreurSejour> {
        let formule = sqlx::query!(
            r#"
            SELECT famille, prix_mineur, prix_heure_supplementaire_mineur
            FROM hebergement.formule
            WHERE id = $1
            "#,
            formule_id
        )
        .fetch_optional(&mut **tx)
        .await?
        .ok_or_else(|| {
            ErreurSejour::Attribution(crate::occupation::ErreurAttribution::FormuleInconnue)
        })?;

        // `depuis_code` rend une `ErreurReferentiel` : un code de famille inconnu en base signale
        // une donnée écrite hors du produit, pas une saisie. Il est traduit en `FormuleInconnue`
        // plutôt que remonté tel quel — l'écran n'a pas de phrase pour un défaut de référentiel.
        let famille = FamilleFormule::depuis_code(&formule.famille).map_err(|_| {
            ErreurSejour::Attribution(crate::occupation::ErreurAttribution::FormuleInconnue)
        })?;
        let duree_minutes = (fin - debut).whole_minutes();

        // Une nuitée se compte en **nuits**, un passage en **une** occurrence du palier retenu.
        // La quantité est en `NUMERIC` (porte P-10) : une demi-journée vaudra 0,5, et un mois au
        // prorata sera fractionnaire.
        match famille {
            FamilleFormule::Passage => {
                let paliers = sqlx::query!(
                    r#"
                    SELECT duree_minutes, prix_mineur
                    FROM hebergement.bareme_palier
                    WHERE formule_id = $1
                    ORDER BY duree_minutes
                    "#,
                    formule_id
                )
                .fetch_all(&mut **tx)
                .await?
                .into_iter()
                .map(|p| crate::tarification::bareme::Palier {
                    duree_minutes: p.duree_minutes,
                    prix_mineur: p.prix_mineur,
                })
                .collect::<Vec<_>>();

                // **Aucune bascule à l'ouverture** : la bascule est un fait du départ, quand la
                // durée réelle dépasse le seuil. L'annoncer ici facturerait une nuitée à qui
                // demande quatre heures.
                let calcul = crate::tarification::bareme::calculer(
                    duree_minutes,
                    &paliers,
                    formule.prix_heure_supplementaire_mineur,
                    None,
                    None,
                )
                .map_err(|_| {
                    ErreurSejour::Attribution(
                        crate::occupation::ErreurAttribution::FormuleInconnue,
                    )
                })?;

                Ok((Decimal::ONE, calcul.montant_du_mineur, calcul.montant_du_mineur))
            }
            FamilleFormule::Nuitee => {
                // Nombre de nuits **entamées** : quatre heures à cheval sur minuit font une nuit.
                // `div_ceil` plutôt qu'une division entière — arrondir vers le bas offrirait la
                // dernière nuit à qui part à 1 h du matin.
                let nuits = (duree_minutes.max(1) as f64 / (24.0 * 60.0)).ceil().max(1.0) as i64;
                let quantite = Decimal::from(nuits);
                let montant = formule.prix_mineur.saturating_mul(nuits);
                Ok((quantite, formule.prix_mineur, montant))
            }
            _ => {
                // Demi-journée, mois : le montant est le prix d'appel, la quantité est **une**
                // occurrence. Le prorata mensuel viendra avec sa story ; l'inventer ici serait
                // exactement l'anticipation que le principe X interdit.
                Ok((Decimal::ONE, formule.prix_mineur, formule.prix_mineur))
            }
        }
    }

    /// Compose la réponse complète — **tout ce que l'écran affiche, en un appel**.
    async fn composer_ouvert(
        &self,
        tx: &mut sqlx::PgTransaction<'_>,
        sejour_id: Uuid,
        occupation: crate::occupation::OccupationVue,
    ) -> Result<SejourOuvert, ErreurSejour> {
        let sejour = repository::lire(tx, sejour_id)
            .await?
            .ok_or(ErreurSejour::SejourInconnu)?;
        let note = note_repo::lire_par_sejour(tx, sejour_id)
            .await?
            .ok_or(ErreurSejour::NoteInconnue)?;
        let fiche_police = police_repo::lire_par_sejour(tx, sejour_id)
            .await?
            .ok_or(ErreurSejour::SejourInconnu)?;
        let instant_autorite = repository::maintenant(tx).await?;

        Ok(SejourOuvert {
            sejour,
            occupation,
            note,
            fiche_police,
            instant_autorite,
        })
    }

    // =============================================================================================
    //  Lectures
    // =============================================================================================

    /// Les séjours d'un établissement, **avec le nom de leur client**.
    ///
    /// ★ **Les noms sont résolus PAR LOT**, en un seul appel à `AnnuaireClients::resumes`. Une
    /// résolution unitaire produirait N+1 requêtes, et c'est le détail qui décide si l'écran de
    /// départ s'ouvre en 200 ms ou en deux secondes.
    pub async fn lister(
        &self,
        etablissement_id: Uuid,
        seulement_en_cours: bool,
    ) -> Result<Vec<SejourVue>, ErreurSejour> {
        self.garde(etablissement_id).await?;

        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, self.tenant_id).await?;
        let lignes = repository::lister(&mut tx, etablissement_id, seulement_en_cours).await?;

        let mut vues = Vec::with_capacity(lignes.len());
        for ligne in lignes {
            let note = note_repo::lire_par_sejour(&mut tx, ligne.sejour.id).await?;
            let personnes = repository::nombre_personnes(&mut tx, ligne.sejour.id).await?;
            vues.push((ligne, note, personnes));
        }
        tx.rollback().await?;

        self.habiller_de_noms(vues).await
    }

    /// L'historique des séjours d'un client — **servi depuis `hebergement`** (opération 5).
    pub async fn historique_du_client(
        &self,
        client_id: Uuid,
        limite: i64,
    ) -> Result<Vec<SejourVue>, ErreurSejour> {
        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, self.tenant_id).await?;
        let lignes = repository::historique_du_client(&mut tx, client_id, limite).await?;

        let mut vues = Vec::with_capacity(lignes.len());
        for ligne in lignes {
            let note = note_repo::lire_par_sejour(&mut tx, ligne.sejour.id).await?;
            let personnes = repository::nombre_personnes(&mut tx, ligne.sejour.id).await?;
            vues.push((ligne, note, personnes));
        }
        tx.rollback().await?;

        self.habiller_de_noms(vues).await
    }

    /// Résout les noms **en une requête** et compose les vues.
    async fn habiller_de_noms(
        &self,
        lignes: Vec<(repository::SejourEnCours, Option<NoteVue>, i32)>,
    ) -> Result<Vec<SejourVue>, ErreurSejour> {
        let ids: Vec<Uuid> = lignes
            .iter()
            .filter_map(|(l, _, _)| l.sejour.client_id)
            .collect();

        let resumes = self
            .annuaire_clients
            .resumes(self.tenant_id, &ids)
            .await
            .map_err(|e| ErreurSejour::Annuaire(e.to_string()))?;

        Ok(lignes
            .into_iter()
            .map(|(ligne, note, personnes)| {
                // Un identifiant absent de la réponse est un client **purgé** (TRX-06) ou jamais
                // rattaché : les deux se présentent de la même façon à l'écran, **sans nom**.
                let resume = ligne
                    .sejour
                    .client_id
                    .and_then(|id| resumes.iter().find(|r| r.id == id));

                SejourVue {
                    client_nom: resume.map(|r| match &r.prenoms {
                        Some(p) => format!("{} {p}", r.nom),
                        None => r.nom.clone(),
                    }),
                    client_telephone: resume.and_then(|r| r.telephone.clone()),
                    nombre_personnes: personnes,
                    unite_id: ligne.unite_id,
                    fin_prevue: ligne.fin_prevue,
                    total_mineur: note.as_ref().map(|n| n.total_mineur).unwrap_or(0),
                    devise: note.map(|n| n.devise).unwrap_or_default(),
                    sejour: ligne.sejour,
                }
            })
            .collect())
    }

    /// Lit un séjour complet.
    pub async fn lire(&self, sejour_id: Uuid) -> Result<Option<SejourOuvert>, ErreurSejour> {
        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, self.tenant_id).await?;

        let Some(sejour) = repository::lire(&mut tx, sejour_id).await? else {
            tx.rollback().await?;
            return Ok(None);
        };
        let note = note_repo::lire_par_sejour(&mut tx, sejour_id)
            .await?
            .ok_or(ErreurSejour::NoteInconnue)?;
        let fiche_police = police_repo::lire_par_sejour(&mut tx, sejour_id)
            .await?
            .ok_or(ErreurSejour::SejourInconnu)?;
        let occupation = self
            .occupation
            .lire(identifiant_occupation(sejour_id, 0))
            .await?
            .ok_or(ErreurSejour::SejourInconnu)?;
        let instant_autorite = repository::maintenant(&mut tx).await?;
        tx.rollback().await?;

        Ok(Some(SejourOuvert {
            sejour,
            occupation,
            note,
            fiche_police,
            instant_autorite,
        }))
    }

    /// Les accompagnants **non retirés** d'un séjour.
    pub async fn accompagnants(
        &self,
        sejour_id: Uuid,
    ) -> Result<Vec<Accompagnant>, ErreurSejour> {
        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, self.tenant_id).await?;
        let liste = repository::accompagnants(&mut tx, sejour_id).await?;
        tx.rollback().await?;
        Ok(liste)
    }

    // =============================================================================================
    //  Rattacher un client APRÈS coup — le parcours du passage
    // =============================================================================================

    /// Rattache une fiche client à un séjour déjà ouvert.
    ///
    /// **Ne rouvre pas le séjour et ne remet pas en cause l'attribution** (FR-028). C'est le
    /// parcours normal du passage : la pièce vient **après** la clé (FR-023). La fiche de police
    /// passe à `complete = true` dans la même transaction.
    pub async fn rattacher_client(
        &self,
        etablissement_id: Uuid,
        sejour_id: Uuid,
        client_id: Uuid,
    ) -> Result<Sejour, ErreurSejour> {
        self.garde(etablissement_id).await?;

        if !self
            .annuaire_clients
            .existe(self.tenant_id, client_id)
            .await
            .map_err(|e| ErreurSejour::Annuaire(e.to_string()))?
        {
            return Err(ErreurSejour::ClientInconnu);
        }

        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, self.tenant_id).await?;

        if !repository::rattacher_client(&mut tx, sejour_id, client_id).await? {
            let _ = tx.rollback().await;
            return Err(ErreurSejour::SejourInconnu);
        }
        police_repo::completer(&mut tx, sejour_id).await?;

        let sejour = repository::lire(&mut tx, sejour_id)
            .await?
            .ok_or(ErreurSejour::SejourInconnu)?;
        tx.commit().await?;

        Ok(sejour)
    }

    // =============================================================================================
    //  Accompagnants — classe A, et ★ le cas orphelin
    // =============================================================================================

    /// Ajoute un accompagnant.
    ///
    /// ★ **Trois issues, et la troisième est celle du principe VI.**
    ///
    /// Un ajout sur un séjour **clos** n'est ni accepté — `201` serait un ajout d'office — ni
    /// rejeté — `409` serait un rejet silencieux. Le principe VI interdit les deux : l'écriture
    /// part en **file de réconciliation**, avec son motif et **sa charge utile**, et le séjour
    /// n'est pas touché.
    ///
    /// C'est le **premier cas réel d'écriture orpheline du produit**. `accompagnant` est de classe
    /// A — écrit hors ligne, mis en file, vidé au retour du réseau — donc susceptible d'arriver
    /// après le départ.
    pub async fn ajouter_accompagnant(
        &self,
        etablissement_id: Uuid,
        sejour_id: Uuid,
        nouveau: NouvelAccompagnant,
    ) -> Result<IssueAccompagnant, ErreurSejour> {
        self.garde(etablissement_id).await?;

        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, self.tenant_id).await?;

        let sejour = repository::lire(&mut tx, sejour_id)
            .await?
            .ok_or(ErreurSejour::SejourInconnu)?;

        // ── ★ LE CAS ORPHELIN ─────────────────────────────────────────────────────────────────
        if sejour.statut == StatutSejour::Clos {
            let reconciliation_id = identifiant_reconciliation(nouveau.id);

            // ⚠️ **La charge utile est composée ICI, du côté de la verticale**, et traverse en
            // JSON opaque. `kaya_synchronisation` ne doit connaître ni `Accompagnant` ni `Sejour`
            // (porte P-03) — c'est le piège concret que `traits-exposes.md` désigne nommément.
            //
            // ⚠️ **Le numéro de pièce N'Y ENTRE PAS.** La file est consultée par SYN-03 et sa
            // rétention n'est pas celle de TRX-06 ; y recopier un numéro créerait une troisième
            // durée de conservation pour la même donnée.
            let charge_utile = json!({
                "accompagnant_id": nouveau.id,
                "nom": nouveau.nom,
                "prenoms": nouveau.prenoms,
                "date_naissance": nouveau.date_naissance.map(|d| d.to_string()),
                "nationalite": nouveau.nationalite,
                "piece_fournie": nouveau.numero_piece.is_some(),
            });

            repository::inscrire_orpheline(
                &mut tx,
                self.tenant_id,
                etablissement_id,
                reconciliation_id,
                nouveau.id,
                "accompagnant",
                sejour_id,
                charge_utile,
                "sejour_clos",
                nouveau.horodatage_client,
            )
            .await?;

            tx.commit().await?;
            return Ok(IssueAccompagnant::Orphelin { reconciliation_id });
        }

        let cree =
            repository::ajouter_accompagnant(&mut tx, self.tenant_id, sejour_id, &nouveau).await?;

        // **Aucun second événement sur rejeu** — le contrôle perdu à la réécriture sur
        // `occupation`, et que `tester_classe_a!` rétablit.
        if cree {
            self.emettre(
                &mut tx,
                etablissement_id,
                TYPE_ACCOMPAGNANT_AJOUTE,
                AGREGAT_ACCOMPAGNANT,
                nouveau.id,
                json!({
                    "accompagnant_id": nouveau.id,
                    "sejour_id": sejour_id,
                    "nom": nouveau.nom,
                    "prenoms": nouveau.prenoms,
                }),
            )
            .await?;
        }

        let accompagnant = repository::lire_accompagnant(&mut tx, nouveau.id)
            .await?
            .ok_or(ErreurSejour::AccompagnantInconnu)?;
        tx.commit().await?;

        Ok(if cree {
            IssueAccompagnant::Ajoute(accompagnant)
        } else {
            IssueAccompagnant::Rejeu(accompagnant)
        })
    }

    /// Retire un accompagnant — **`retire_le`, jamais un `DELETE`**.
    pub async fn retirer_accompagnant(
        &self,
        etablissement_id: Uuid,
        accompagnant_id: Uuid,
    ) -> Result<Accompagnant, ErreurSejour> {
        self.garde(etablissement_id).await?;

        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, self.tenant_id).await?;

        repository::retirer_accompagnant(&mut tx, accompagnant_id).await?;
        let accompagnant = repository::lire_accompagnant(&mut tx, accompagnant_id)
            .await?
            .ok_or(ErreurSejour::AccompagnantInconnu)?;
        tx.commit().await?;

        Ok(accompagnant)
    }

    // =============================================================================================
    //  Fonctions internes
    // =============================================================================================

    async fn emettre(
        &self,
        tx: &mut sqlx::PgTransaction<'_>,
        etablissement_id: Uuid,
        type_evenement: &str,
        agregat: &str,
        agregat_id: Uuid,
        payload: serde_json::Value,
    ) -> Result<(), ErreurSejour> {
        self.outbox
            .ecrire(
                tx,
                EvenementAEcrire {
                    id: Uuid::now_v7(),
                    tenant_id: self.tenant_id,
                    etablissement_id: Some(etablissement_id),
                    type_evenement: type_evenement.to_owned(),
                    agregat: agregat.to_owned(),
                    agregat_id,
                    version_schema: VERSION_SCHEMA_SEJOUR,
                    payload,
                },
            )
            .await?;
        Ok(())
    }
}

// =================================================================================================
//  Identifiants dérivés — ce qui rend le rejeu inoffensif sur CINQ tables
// =================================================================================================
//
// ★ **Le terminal fournit UN identifiant, celui du séjour. Les quatre autres en sont DÉRIVÉS.**
//
// La question qu'on se pose autrement : que devient le rejeu d'un séjour si la note, la ligne et
// la fiche de police tirent des identifiants neufs à chaque tentative ? Le `ON CONFLICT` du séjour
// constaterait le rejeu, et les quatre autres écriraient des lignes en double — une note de plus,
// une ligne de plus, une fiche de police de plus, **avec un numéro de plus**. La numérotation
// perdrait sa continuité pour une raison invisible : une coupure réseau.
//
// La dérivation est **déterministe** : le même séjour rejoué demande les mêmes lignes, et chaque
// `ON CONFLICT (id) DO NOTHING` fait son travail. Elle est aussi **sans collision entre tables** —
// un octet de discriminant par famille —, ce qui évite qu'une note et une fiche de police
// partagent un identifiant le jour où quelqu'un les joindrait par erreur.

/// Discriminants de famille — **un par table**, jamais réutilisé.
const DISCRIMINANT_OCCUPATION: u8 = 0x01;
const DISCRIMINANT_NOTE: u8 = 0x02;
const DISCRIMINANT_LIGNE: u8 = 0x03;
const DISCRIMINANT_FICHE: u8 = 0x04;
const DISCRIMINANT_RECONCILIATION: u8 = 0x05;

/// Dérive un identifiant depuis celui du séjour, un discriminant de famille et un rang.
///
/// Le rang sert aux familles qui peuvent avoir plusieurs lignes pour un séjour — les occupations
/// (changement d'unité) et les lignes de note (ajustements).
fn deriver(source: Uuid, discriminant: u8, rang: u16) -> Uuid {
    let mut octets = *source.as_bytes();
    // Les deux derniers octets portent le rang, l'avant-dernier groupe le discriminant : la partie
    // haute d'un UUID v7 est l'horodatage, et la conserver garde l'ordre temporel des identifiants
    // dérivés — ce qui rend les index de ces tables aussi efficaces que ceux du séjour.
    octets[13] ^= discriminant;
    octets[14] ^= (rang >> 8) as u8;
    octets[15] ^= (rang & 0xFF) as u8;
    Uuid::from_bytes(octets)
}

fn identifiant_occupation(sejour_id: Uuid, rang: u16) -> Uuid {
    deriver(sejour_id, DISCRIMINANT_OCCUPATION, rang)
}

fn identifiant_note(sejour_id: Uuid) -> Uuid {
    deriver(sejour_id, DISCRIMINANT_NOTE, 0)
}

fn identifiant_ligne(sejour_id: Uuid, rang: u16) -> Uuid {
    deriver(sejour_id, DISCRIMINANT_LIGNE, rang)
}

fn identifiant_fiche(sejour_id: Uuid) -> Uuid {
    deriver(sejour_id, DISCRIMINANT_FICHE, 0)
}

fn identifiant_reconciliation(accompagnant_id: Uuid) -> Uuid {
    deriver(accompagnant_id, DISCRIMINANT_RECONCILIATION, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **La dérivation est déterministe** — c'est ce qui rend le rejeu inoffensif sur cinq tables.
    #[test]
    fn la_derivation_est_deterministe() {
        let sejour = Uuid::now_v7();
        assert_eq!(identifiant_note(sejour), identifiant_note(sejour));
        assert_eq!(identifiant_fiche(sejour), identifiant_fiche(sejour));
        assert_eq!(
            identifiant_occupation(sejour, 0),
            identifiant_occupation(sejour, 0)
        );
    }

    /// **Aucune famille ne collisionne avec une autre.**
    ///
    /// Sans discriminant distinct, une note et une fiche de police porteraient le même
    /// identifiant — sans conséquence tant que personne ne les joint, et avec des conséquences
    /// silencieuses le jour où quelqu'un le fait.
    #[test]
    fn les_familles_ne_collisionnent_pas() {
        let sejour = Uuid::now_v7();
        let derives = [
            identifiant_occupation(sejour, 0),
            identifiant_note(sejour),
            identifiant_ligne(sejour, 0),
            identifiant_fiche(sejour),
            identifiant_reconciliation(sejour),
        ];
        let mut uniques = derives.to_vec();
        uniques.sort();
        uniques.dedup();
        assert_eq!(uniques.len(), derives.len(), "deux familles partagent un identifiant");
    }

    /// **Deux rangs de la même famille diffèrent** — le changement d'unité produit deux
    /// occupations sur un séjour, et les ajustements plusieurs lignes.
    #[test]
    fn deux_rangs_de_la_meme_famille_different() {
        let sejour = Uuid::now_v7();
        assert_ne!(
            identifiant_occupation(sejour, 0),
            identifiant_occupation(sejour, 1)
        );
        assert_ne!(identifiant_ligne(sejour, 0), identifiant_ligne(sejour, 7));
    }

    /// **Deux séjours différents ne partagent aucun dérivé.**
    #[test]
    fn deux_sejours_ne_partagent_aucun_derive() {
        let a = Uuid::now_v7();
        let b = Uuid::now_v7();
        assert_ne!(identifiant_note(a), identifiant_note(b));
        assert_ne!(identifiant_fiche(a), identifiant_fiche(b));
    }
}
