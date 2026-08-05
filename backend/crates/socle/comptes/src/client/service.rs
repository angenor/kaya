//! Couche service de la fiche client — **la transaction, l'événement dedans, et le journal
//! d'accès à la pièce d'identité**.
//!
//! > Toute transition d'état écrit un événement outbox **dans la même transaction**
//! > (principe II, porte P-05).
//!
//! La garantie tient à une signature, pas à une discipline : `OutboxWriter::ecrire` prend la
//! transaction et n'en ouvre jamais une. Il en va de même de `JournalAudit::tracer`.
//!
//! # ⚠️ Aucun numéro de pièce d'identité n'entre dans l'outbox, et c'est une décision
//!
//! Le grand livre est à rétention **illimitée** et **immuable** (principe II, porte P-05b). Une
//! donnée sensible qui y entre ne peut **jamais** en sortir, et la rétention de 90 jours de
//! TRX-06 deviendrait inapplicable sur la copie : le numéro serait purgé de `comptes.personne` et
//! conservé pour toujours dans l'outbox. Les charges utiles de ce fichier portent des
//! identifiants et des noms ; jamais un numéro, jamais un cryptogramme.
//!
//! # ⚠️ Le journal d'accès est ici, et pas dans le coffre
//!
//! [`super::coffre::CoffreTenant::dechiffrer`] ne journalise rien : il ne connaît ni l'auteur, ni
//! le motif, ni la transaction. Ce service, lui, connaît les trois — et **écrit la trace dans la
//! même transaction que la lecture**. Mettre la journalisation dans le coffre l'aurait rendue
//! contournable : il aurait suffi d'appeler la couche du dessous.
//!
//! **Il n'existe qu'un seul chemin qui déchiffre**, et il trace toujours : [`ServiceClient::lire`].

use serde_json::json;
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use kaya_etablissements::Issue;
use kaya_etablissements::tenant_context;
use kaya_synchronisation::{EvenementAEcrire, OutboxWriter};

use crate::audit::{EntreeAudit, JournalAudit, TypeActionAudit};

use super::coffre::CoffreTenant;
use super::modele::{
    ClientResume, CreerClient, ErreurClient, FicheClient, LIMITE_DEFAUT, LIMITE_MAX,
    ModifierClient, NOM_MAX, PREFERENCE_MAX, Preference, ResultatRecherche, deduire_forme,
};
use super::repli::{repli, repli_piece, repli_telephone};
use super::repository::{self, Replis};

/// Noms des agrégats au grand livre.
pub const AGREGAT_CLIENT: &str = "comptes.client";
pub const AGREGAT_PREFERENCE: &str = "comptes.preference_personne";

/// Types d'événements — nomenclature `agregat.action`.
pub const TYPE_CLIENT_CREE: &str = "sej.client.cree";
pub const TYPE_CLIENT_MODIFIE: &str = "sej.client.modifie";
pub const TYPE_PREFERENCE_ENREGISTREE: &str = "sej.preference.enregistree";

/// Version du format des charges utiles ci-dessous.
pub const VERSION_SCHEMA_CLIENT: i16 = 1;

/// Service de la fiche client.
///
/// # Pourquoi le coffre est porté par l'instance
///
/// Le coffre dérive une clé par tenant et la met en cache. Le construire à chaque appel viderait
/// le cache à chaque requête, et l'écran de recherche paierait une dérivation Argon2 par
/// résultat — la cible des 300 ms serait perdue pour une raison sans rapport avec la recherche.
pub struct ServiceClient<E, J>
where
    E: OutboxWriter,
    J: JournalAudit,
{
    pool: PgPool,
    outbox: E,
    journal: J,
    coffre: std::sync::Arc<CoffreTenant>,
}

impl<E, J> ServiceClient<E, J>
where
    E: OutboxWriter,
    J: JournalAudit,
{
    pub fn nouveau(
        pool: PgPool,
        outbox: E,
        journal: J,
        coffre: std::sync::Arc<CoffreTenant>,
    ) -> Self {
        Self {
            pool,
            outbox,
            journal,
            coffre,
        }
    }

    // =============================================================================================
    //  Créer — une transaction, deux INSERT, un événement
    // =============================================================================================

    /// Crée une fiche client.
    ///
    /// L'identifiant est l'**UUID v7 fourni par le terminal** (FR-086) : c'est lui, et non une clé
    /// engendrée côté serveur, qui rend le rejeu inoffensif. Le serveur **déduplique**, il
    /// n'engendre pas.
    ///
    /// # `indicatif_par_defaut` vient de la configuration héritée, jamais d'une constante
    ///
    /// Au comptoir, personne ne tape l'indicatif. La normalisation le préfixe quand la saisie n'en
    /// porte pas — mais l'indicatif est un **paramètre d'établissement** (`CPT-01`,
    /// `indicatif_telephonique_defaut`) : `+225` écrit en dur ferait échouer le premier
    /// établissement togolais, et le ferait échouer **silencieusement**, en rendant introuvables
    /// des fiches pourtant créées.
    #[tracing::instrument(skip(self, demande), fields(client.id = %demande.id, tenant.id = %tenant_id))]
    pub async fn creer(
        &self,
        tenant_id: Uuid,
        indicatif_par_defaut: &str,
        demande: CreerClient,
    ) -> Result<(FicheClient, Issue), ErreurClient> {
        let demande = self.valider(demande)?;
        let replis = self.replis(&demande.nom, &demande.prenoms, &demande.telephone,
                                 &demande.numero_piece, indicatif_par_defaut);

        // Le chiffrement se fait AVANT la transaction : une erreur de coffre est un défaut de
        // configuration, pas une raison d'ouvrir puis d'annuler une transaction.
        let chiffre = self.chiffrer_piece(tenant_id, demande.numero_piece.as_deref())?;

        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, tenant_id).await?;

        let issue =
            repository::inserer(&mut tx, tenant_id, &demande, &replis, chiffre.as_deref()).await?;

        // ⚠️ **Aucun numéro de pièce dans la charge utile.** Voir le commentaire de tête.
        if issue == Issue::Creee {
            self.emettre(
                &mut tx,
                TYPE_CLIENT_CREE,
                AGREGAT_CLIENT,
                demande.id,
                tenant_id,
                json!({
                    "client_id": demande.id,
                    "nom": demande.nom,
                    "prenoms": demande.prenoms,
                    "piece_enregistree": chiffre.is_some(),
                }),
            )
            .await?;
        }

        let stockee = repository::lire_avec_piece_chiffree(&mut tx, demande.id)
            .await?
            .ok_or(ErreurClient::Inconnu)?;
        tx.commit().await?;

        // La fiche rendue à l'appelant qui vient de la créer **ne déchiffre pas** : il connaît
        // déjà le numéro, il vient de l'envoyer. Une lecture inutile serait une entrée inutile au
        // registre, et un registre bruyant n'est plus lu.
        Ok((fiche_sans_dechiffrer(stockee), issue))
    }

    // =============================================================================================
    //  Modifier
    // =============================================================================================

    #[tracing::instrument(skip(self, modification), fields(client.id = %id, tenant.id = %tenant_id))]
    pub async fn modifier(
        &self,
        tenant_id: Uuid,
        indicatif_par_defaut: &str,
        id: Uuid,
        modification: ModifierClient,
    ) -> Result<FicheClient, ErreurClient> {
        let modification = self.valider_modification(modification)?;
        let replis = self.replis(
            &modification.nom,
            &modification.prenoms,
            &modification.telephone,
            &modification.numero_piece,
            indicatif_par_defaut,
        );
        let chiffre = self.chiffrer_piece(tenant_id, modification.numero_piece.as_deref())?;

        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, tenant_id).await?;

        let touchee =
            repository::modifier(&mut tx, id, &modification, &replis, chiffre.as_deref()).await?;
        if !touchee {
            let _ = tx.rollback().await;
            return Err(ErreurClient::Inconnu);
        }

        self.emettre(
            &mut tx,
            TYPE_CLIENT_MODIFIE,
            AGREGAT_CLIENT,
            id,
            tenant_id,
            json!({
                "client_id": id,
                "nom": modification.nom,
                "prenoms": modification.prenoms,
                "piece_enregistree": chiffre.is_some(),
            }),
        )
        .await?;

        let stockee = repository::lire_avec_piece_chiffree(&mut tx, id)
            .await?
            .ok_or(ErreurClient::Inconnu)?;
        tx.commit().await?;

        Ok(fiche_sans_dechiffrer(stockee))
    }

    // =============================================================================================
    //  Lire — ★ LE SEUL CHEMIN QUI DÉCHIFFRE, ET IL TRACE TOUJOURS
    // =============================================================================================

    /// Lit une fiche complète, **numéro de pièce déchiffré**, et **journalise la consultation**.
    ///
    /// # Pourquoi la trace est dans la même transaction que la lecture
    ///
    /// Une trace écrite après coup peut manquer : le processus tombe entre les deux, et le numéro
    /// a été lu sans qu'aucune ligne ne le dise. Dans la transaction, l'un ne va pas sans l'autre
    /// — et si l'écriture au registre échoue, la lecture est annulée. **C'est le comportement
    /// voulu** : ne pas pouvoir tracer un accès à une donnée sensible est une raison suffisante de
    /// refuser l'accès (principe IX).
    ///
    /// # La trace n'est écrite QUE si un numéro est réellement déchiffré
    ///
    /// Lire une fiche sans pièce n'est pas un accès à une pièce. Tracer toutes les lectures
    /// noierait les vraies consultations sous des entrées vides, et un registre illisible n'est
    /// plus lu — le même raisonnement que la fréquence de `derive_horloge_constatee`.
    #[tracing::instrument(skip(self), fields(client.id = %id, tenant.id = %tenant_id))]
    pub async fn lire(
        &self,
        tenant_id: Uuid,
        auteur_compte_id: Uuid,
        id: Uuid,
    ) -> Result<FicheClient, ErreurClient> {
        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, tenant_id).await?;

        let stockee = repository::lire_avec_piece_chiffree(&mut tx, id)
            .await?
            .ok_or(ErreurClient::Inconnu)?;

        let numero_piece = match stockee.numero_piece_chiffre.as_deref() {
            None => None,
            Some(cryptogramme) => {
                let clair = self
                    .coffre
                    .dechiffrer(tenant_id, cryptogramme)
                    .map_err(|e| ErreurClient::Coffre(e.to_string()))?;

                // ★ La trace, DANS la transaction. Le contexte ne porte **jamais** la valeur lue.
                self.journal
                    .tracer(
                        &mut tx,
                        tenant_id,
                        EntreeAudit {
                            id: Uuid::now_v7(),
                            etablissement_id: None,
                            type_action: TypeActionAudit::ConsultationPieceIdentite,
                            auteur_compte_id,
                            cible_type: "personne".to_owned(),
                            cible_id: Some(id),
                            contexte: json!({ "motif": "lecture_fiche_client" }),
                            horodatage_client: None,
                        },
                    )
                    .await
                    .map_err(|e| ErreurClient::Audit(e.to_string()))?;

                Some(clair)
            }
        };

        tx.commit().await?;

        Ok(FicheClient {
            numero_piece,
            ..fiche_sans_dechiffrer(stockee)
        })
    }

    // =============================================================================================
    //  Rechercher
    // =============================================================================================

    /// Cherche des fiches — **trois formes, une entrée, une requête**.
    ///
    /// La forme est **déduite** de la saisie : l'opérateur ne choisit pas un mode. Une saisie
    /// ambiguë interroge les trois et fusionne, ce qui est le comportement attendu au comptoir.
    ///
    /// ⚠️ **Cette lecture ne déchiffre rien et ne journalise rien** : `ClientResume` ne porte
    /// aucun numéro de pièce, seulement `piece_enregistree`. Sans cette propriété, chaque frappe
    /// de Yao produirait vingt entrées au registre et une dérivation par résultat.
    pub async fn rechercher(
        &self,
        tenant_id: Uuid,
        indicatif_par_defaut: &str,
        saisie: &str,
        limite: Option<i64>,
    ) -> Result<ResultatRecherche, ErreurClient> {
        let forme = deduire_forme(saisie);
        let limite = limite.unwrap_or(LIMITE_DEFAUT).clamp(1, LIMITE_MAX);

        let nom_replie = repli(saisie);
        let telephone_replie = normaliser_telephone(saisie, indicatif_par_defaut);
        let piece_repliee = repli_piece(saisie);

        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, tenant_id).await?;
        let (clients, tronque) = repository::rechercher(
            &mut tx,
            forme,
            &nom_replie,
            &telephone_replie,
            &piece_repliee,
            limite,
        )
        .await?;
        // Lecture : la transaction est annulée.
        tx.rollback().await?;

        Ok(ResultatRecherche { clients, tronque })
    }

    // =============================================================================================
    //  Préférences — classe A, append-only
    // =============================================================================================

    /// Enregistre une préférence.
    ///
    /// **`INSERT` seul, jamais un `UPDATE`** : la préférence courante est la ligne la plus
    /// récente. Une correction est une ligne nouvelle. C'est ce qui rend le rejeu inoffensif et le
    /// désordre commutatif — les deux propriétés que `tester_classe_a!` vérifie.
    ///
    /// `horodatage_client` est **accepté et indicatif** : il est écrit en colonne, aucune règle ne
    /// s'y appuie (porte P-23). **Écrire la colonne n'est pas s'appuyer dessus** — l'ordre vient
    /// de `cree_le`, l'horodatage d'autorité.
    pub async fn enregistrer_preference(
        &self,
        tenant_id: Uuid,
        personne_id: Uuid,
        id: Uuid,
        texte: &str,
        horodatage_client: Option<OffsetDateTime>,
    ) -> Result<(Preference, Issue), ErreurClient> {
        let texte = texte.trim();
        if texte.is_empty() || texte.chars().count() > PREFERENCE_MAX {
            return Err(ErreurClient::PreferenceInvalide);
        }

        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, tenant_id).await?;

        if !repository::existe(&mut tx, personne_id).await? {
            let _ = tx.rollback().await;
            return Err(ErreurClient::Inconnu);
        }

        let (preference, issue) = repository::enregistrer_preference(
            &mut tx,
            tenant_id,
            id,
            personne_id,
            texte,
            horodatage_client,
        )
        .await?;

        // **Aucun second événement sur rejeu** — le contrôle perdu à la réécriture sur
        // `occupation`, et que `tester_classe_a!` rétablit ici.
        if issue == Issue::Creee {
            self.emettre(
                &mut tx,
                TYPE_PREFERENCE_ENREGISTREE,
                AGREGAT_PREFERENCE,
                preference.id,
                tenant_id,
                json!({
                    "preference_id": preference.id,
                    "personne_id": personne_id,
                    "texte": preference.texte,
                    "cree_le": preference.cree_le.to_string(),
                }),
            )
            .await?;
        }

        tx.commit().await?;
        Ok((preference, issue))
    }

    /// Les préférences d'une personne, de la plus récente à la plus ancienne.
    pub async fn preferences(
        &self,
        tenant_id: Uuid,
        personne_id: Uuid,
    ) -> Result<Vec<Preference>, ErreurClient> {
        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, tenant_id).await?;
        let liste = repository::preferences(&mut tx, personne_id, LIMITE_MAX).await?;
        tx.rollback().await?;
        Ok(liste)
    }

    /// Les résumés de plusieurs clients — support du trait `AnnuaireClients`.
    pub async fn resumes(
        &self,
        tenant_id: Uuid,
        ids: &[Uuid],
    ) -> Result<Vec<ClientResume>, ErreurClient> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, tenant_id).await?;
        let liste = repository::resumes(&mut tx, ids).await?;
        tx.rollback().await?;
        Ok(liste)
    }

    /// Vrai si l'identifiant désigne un client du tenant courant.
    pub async fn existe(&self, tenant_id: Uuid, id: Uuid) -> Result<bool, ErreurClient> {
        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, tenant_id).await?;
        let trouve = repository::existe(&mut tx, id).await?;
        tx.rollback().await?;
        Ok(trouve)
    }

    // =============================================================================================
    //  Fonctions internes
    // =============================================================================================

    fn valider(&self, demande: CreerClient) -> Result<CreerClient, ErreurClient> {
        Ok(CreerClient {
            nom: nom_valide(&demande.nom)?,
            prenoms: normaliser(demande.prenoms),
            telephone: telephone_valide(normaliser(demande.telephone))?,
            email: normaliser(demande.email),
            nationalite: nationalite_valide(normaliser(demande.nationalite))?,
            type_piece: normaliser(demande.type_piece),
            numero_piece: normaliser(demande.numero_piece),
            ..demande
        })
    }

    fn valider_modification(
        &self,
        modification: ModifierClient,
    ) -> Result<ModifierClient, ErreurClient> {
        Ok(ModifierClient {
            nom: nom_valide(&modification.nom)?,
            prenoms: normaliser(modification.prenoms),
            telephone: telephone_valide(normaliser(modification.telephone))?,
            email: normaliser(modification.email),
            nationalite: nationalite_valide(normaliser(modification.nationalite))?,
            type_piece: normaliser(modification.type_piece),
            numero_piece: normaliser(modification.numero_piece),
            ..modification
        })
    }

    /// Les trois formes cherchables, calculées **à l'écriture**.
    ///
    /// Replier dix mille lignes à chaque frappe coûterait exactement la cible qu'on essaie de
    /// tenir. Le nom replié couvre `nom` **et** `prenoms` : au comptoir, on cherche « Yao » aussi
    /// souvent comme prénom que comme nom.
    fn replis(
        &self,
        nom: &str,
        prenoms: &Option<String>,
        telephone: &Option<String>,
        numero_piece: &Option<String>,
        indicatif_par_defaut: &str,
    ) -> Replis {
        let complet = match prenoms {
            Some(p) => format!("{nom} {p}"),
            None => nom.to_owned(),
        };
        Replis {
            nom: Some(repli(&complet)).filter(|s| !s.is_empty()),
            telephone: telephone
                .as_deref()
                .map(|t| normaliser_telephone(t, indicatif_par_defaut))
                .filter(|s| !s.is_empty()),
            numero_piece: numero_piece
                .as_deref()
                .map(repli_piece)
                .filter(|s| !s.is_empty()),
        }
    }

    fn chiffrer_piece(
        &self,
        tenant_id: Uuid,
        numero: Option<&str>,
    ) -> Result<Option<String>, ErreurClient> {
        numero
            .map(|n| {
                self.coffre
                    .chiffrer(tenant_id, n)
                    .map_err(|e| ErreurClient::Coffre(e.to_string()))
            })
            .transpose()
    }

    async fn emettre(
        &self,
        tx: &mut sqlx::PgTransaction<'_>,
        type_evenement: &str,
        agregat: &str,
        agregat_id: Uuid,
        tenant_id: Uuid,
        payload: serde_json::Value,
    ) -> Result<(), ErreurClient> {
        self.outbox
            .ecrire(
                tx,
                EvenementAEcrire {
                    id: Uuid::now_v7(),
                    tenant_id,
                    // **`None` : la fiche est du TENANT, pas d'un établissement** (FR-002). Un
                    // client de Deloria enregistré à l'accueil est le même client au restaurant.
                    etablissement_id: None,
                    type_evenement: type_evenement.to_owned(),
                    agregat: agregat.to_owned(),
                    agregat_id,
                    version_schema: VERSION_SCHEMA_CLIENT,
                    payload,
                },
            )
            .await?;
        Ok(())
    }
}

/// Construit la fiche rendue **sans toucher au numéro de pièce**, qui reste `None`.
///
/// Le nom dit ce qu'elle fait : elle ne déchiffre pas. Le seul chemin qui déchiffre est
/// [`ServiceClient::lire`], et il trace.
fn fiche_sans_dechiffrer(stockee: repository::FicheStockee) -> FicheClient {
    FicheClient {
        id: stockee.id,
        nom: stockee.nom,
        prenoms: stockee.prenoms,
        telephone: stockee.telephone,
        email: stockee.email,
        date_naissance: stockee.date_naissance,
        nationalite: stockee.nationalite,
        type_piece: stockee.type_piece,
        numero_piece: None,
        piece_capturee_le: stockee.piece_capturee_le,
        horodatage_client: stockee.horodatage_client,
        cree_le: stockee.cree_le,
        modifie_le: stockee.modifie_le,
    }
}

/// Normalise un téléphone pour la recherche : chiffres seuls, **préfixés de l'indicatif de
/// l'établissement** quand la saisie n'en porte pas.
///
/// C'est ce qui fait que « 0707123456 » et « +2250707123456 » se retrouvent l'un l'autre. La
/// détection d'un indicatif déjà présent tient à un `+` initial — imparfait, et suffisant : au
/// pire, une saisie sans `+` mais avec indicatif produit un numéro doublement préfixé, que la
/// comparaison **par suffixe** retrouve quand même.
fn normaliser_telephone(saisie: &str, indicatif_par_defaut: &str) -> String {
    let chiffres = repli_telephone(saisie);
    if chiffres.is_empty() {
        return chiffres;
    }
    if saisie.trim_start().starts_with('+') {
        return chiffres;
    }
    // ⚠️ **Le zéro de tête N'EST PAS retiré, et c'est une décision.**
    //
    // Le réflexe est de le traiter comme un **préfixe interurbain** — en France, `06 12…` devient
    // `+33 6 12…`. **La Côte d'Ivoire n'en a pas** : depuis la renumérotation de 2021, le numéro
    // national compte dix chiffres et s'écrit à l'international `+225 07 07 12 34 56`, zéro
    // compris. Le retirer produirait `225707123456` là où le pays écrit `2250707123456` — un
    // chiffre de moins, et une fiche introuvable dès qu'on la cherche autrement qu'on l'a créée.
    //
    // ⚠️ **Cette règle est celle de la Côte d'Ivoire, et le produit servira d'autres pays.** Le
    // jour où un indicatif dont le plan de numérotation retire le zéro entrera au produit, ce
    // n'est pas cette ligne qu'il faudra corriger mais une **règle par juridiction** — le même
    // raisonnement que `JurisdictionAdapter` pour la fiscalité. Écrit ici pour que la
    // simplification se voie plutôt qu'elle ne se découvre.
    let indicatif = repli_telephone(indicatif_par_defaut);
    format!("{indicatif}{chiffres}")
}

fn nom_valide(brut: &str) -> Result<String, ErreurClient> {
    let nettoye = brut.trim();
    if nettoye.is_empty() || nettoye.chars().count() > NOM_MAX {
        return Err(ErreurClient::NomVide);
    }
    Ok(nettoye.to_owned())
}

/// Le téléphone n'a **aucune contrainte de format national**.
///
/// L'indicatif par défaut est un paramètre d'établissement (porte P-12) : une contrainte
/// ivoirienne posée en dur ferait échouer le premier établissement togolais. Ce qui est refusé est
/// seulement ce qui ne peut être aucun numéro — moins de quatre chiffres.
fn telephone_valide(valeur: Option<String>) -> Result<Option<String>, ErreurClient> {
    match valeur {
        None => Ok(None),
        Some(t) => {
            if repli_telephone(&t).len() < 4 {
                Err(ErreurClient::TelephoneInvalide)
            } else {
                Ok(Some(t))
            }
        }
    }
}

/// Bornes **alignées sur le `CHECK` de la migration `0029`**.
///
/// La validation applicative existe pour rendre un `422` intelligible, pas pour remplacer la
/// contrainte de base : un script de maintenance contournerait la première, jamais la seconde.
fn nationalite_valide(valeur: Option<String>) -> Result<Option<String>, ErreurClient> {
    match valeur {
        None => Ok(None),
        Some(n) => {
            let compte = n.trim().chars().count();
            if !(2..=80).contains(&compte) {
                Err(ErreurClient::NationaliteInvalide)
            } else {
                Ok(Some(n))
            }
        }
    }
}

/// Une chaîne vide ou blanche vaut `None` — jamais `Some("")`.
///
/// Sans cela, effacer un champ à l'écran laisserait une chaîne vide en base, qui n'est ni une
/// valeur ni une absence, et que les deux moitiés du code traiteraient différemment.
fn normaliser(valeur: Option<String>) -> Option<String> {
    valeur
        .map(|v| v.trim().to_owned())
        .filter(|v| !v.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_telephone_sans_indicatif_recoit_celui_de_l_etablissement() {
        assert_eq!(normaliser_telephone("0707123456", "+225"), "2250707123456");
    }

    #[test]
    fn le_telephone_avec_indicatif_n_est_pas_prefixe_deux_fois() {
        assert_eq!(normaliser_telephone("+225 07 07 12 34 56", "+225"), "2250707123456");
    }

    /// **L'indicatif vient du paramètre, jamais d'une constante.** Un établissement togolais
    /// reçoit `+228`, et rien dans ce code ne connaît la Côte d'Ivoire.
    #[test]
    fn l_indicatif_vient_du_parametre_et_non_d_une_constante() {
        assert_eq!(normaliser_telephone("90123456", "+228"), "22890123456");
    }

    #[test]
    fn un_champ_blanc_vaut_une_absence_et_non_une_chaine_vide() {
        assert_eq!(normaliser(Some("   ".to_owned())), None);
        assert_eq!(normaliser(Some(String::new())), None);
        assert_eq!(normaliser(Some(" Yao ".to_owned())), Some("Yao".to_owned()));
    }

    #[test]
    fn un_nom_vide_est_refuse_avec_son_code_stable() {
        assert_eq!(nom_valide("   ").unwrap_err().code(), "nom_vide");
    }

    #[test]
    fn un_telephone_trop_court_pour_etre_un_numero_est_refuse() {
        assert!(telephone_valide(Some("12".to_owned())).is_err());
        assert!(telephone_valide(Some("0707123456".to_owned())).is_ok());
        assert!(telephone_valide(None).is_ok());
    }

    #[test]
    fn une_nationalite_hors_bornes_est_refusee() {
        assert!(nationalite_valide(Some("F".to_owned())).is_err());
        assert!(nationalite_valide(Some("Ivoirienne".to_owned())).is_ok());
        assert!(nationalite_valide(Some("x".repeat(81))).is_err());
    }
}
