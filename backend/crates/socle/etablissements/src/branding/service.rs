//! Couche service de l'identité visuelle — ETB-05.
//!
//! # Le plafond de taille du logo est une CONSTANTE TECHNIQUE, pas un paramètre
//!
//! Un exploitant n'a aucune raison de régler la taille maximale d'un logo, et l'inscrire au
//! catalogue de paramètres ferait entrer au récapitulatif du principe I·c une valeur qui ne relève
//! pas de l'exploitation. Elle est donc déclarée dans le code, **avec sa justification**, et son
//! dépassement produit un message qui **donne la limite** — jamais un refus muet.

use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use kaya_synchronisation::{EvenementAEcrire, OutboxWriter};

use super::modele::{
    BrandingNiveau, BrandingResolu, ChampResolu, EcrireBranding, ErreurBranding, couleur_valide,
};
use super::repository;
use crate::tenant_context;

pub const VERSION_SCHEMA: i16 = 1;
pub const AGREGAT: &str = "branding";
pub const TYPE_MODIFIE: &str = "branding.modifie";

/// Taille maximale d'un logo, en octets.
///
/// **512 kio, et voici pourquoi ce nombre.** Un logo d'établissement est imprimé sur des tickets
/// de 58 mm et des factures A4 : au-delà de 512 kio, on transporte des pixels que ni l'un ni
/// l'autre ne restitue. La contrainte réelle est le terrain — la persona Aminata travaille sur un
/// Android d'entrée de gamme en réseau intermittent, et un logo de 5 Mio rendrait chaque
/// synchronisation d'identité visuelle impossible à terminer.
///
/// **Ce n'est pas un paramètre d'établissement** : le régler n'aurait aucun sens métier, et
/// l'inscrire au catalogue polluerait le récapitulatif du principe I·c.
pub const LOGO_TAILLE_MAX: usize = 512 * 1024;

/// **Mention obligatoire sur tout document non fiscal** (principe V, FR-058).
///
/// Un aperçu d'identité visuelle ressemble à une facture : mêmes en-tête, logo, coordonnées et
/// mentions légales. Sans cette phrase, le premier aperçu imprimé serait présenté à un client
/// comme un justificatif — et il n'en est pas un.
pub const MENTION_NON_FISCALE: &str = "Document non fiscal — ne tient pas lieu de facture";

/// Service de l'identité visuelle.
pub struct ServiceBranding<E: OutboxWriter> {
    pool: PgPool,
    outbox: E,
}

impl<E: OutboxWriter> ServiceBranding<E> {
    pub fn nouveau(pool: PgPool, outbox: E) -> Self {
        Self { pool, outbox }
    }

    /// Résout l'identité visuelle **champ par champ**.
    ///
    /// Première valeur non nulle en descendant du tenant vers l'établissement. Sans
    /// `etablissement_id`, rend l'identité du tenant telle quelle.
    pub async fn resoudre(
        &self,
        tenant_id: Uuid,
        etablissement_id: Option<Uuid>,
    ) -> Result<BrandingResolu, ErreurBranding> {
        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, tenant_id).await?;

        if let Some(id) = etablissement_id
            && !repository::etablissement_existe(&mut tx, id).await?
        {
            return Err(ErreurBranding::EtablissementInconnu);
        }

        let niveau_tenant = repository::lire_niveau(&mut tx, None).await?;
        let niveau_etablissement = match etablissement_id {
            Some(id) => repository::lire_niveau(&mut tx, Some(id)).await?,
            None => None,
        };

        tx.rollback().await?;
        Ok(fusionner(niveau_tenant, niveau_etablissement))
    }

    /// Lit l'identité visuelle d'un **niveau précis**, sans fusion.
    ///
    /// C'est ce que l'écran édite : on modifie ce qu'on a posé à son propre niveau, pas le
    /// résultat de la fusion — sinon enregistrer figerait chez soi tout ce dont on héritait.
    pub async fn lire_niveau(
        &self,
        tenant_id: Uuid,
        etablissement_id: Option<Uuid>,
    ) -> Result<BrandingNiveau, ErreurBranding> {
        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, tenant_id).await?;
        let niveau = repository::lire_niveau(&mut tx, etablissement_id).await?;
        tx.rollback().await?;
        Ok(niveau.unwrap_or_default())
    }

    /// Écrit l'identité visuelle d'un niveau.
    #[tracing::instrument(skip(self, demande), fields(tenant.id = %tenant_id))]
    pub async fn ecrire(
        &self,
        tenant_id: Uuid,
        demande: EcrireBranding,
    ) -> Result<BrandingNiveau, ErreurBranding> {
        if let Some(couleur) = &demande.contenu.couleur_primaire
            && !couleur_valide(couleur)
        {
            return Err(ErreurBranding::CouleurInvalide(couleur.clone()));
        }

        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, tenant_id).await?;

        if let Some(id) = demande.etablissement_id
            && !repository::etablissement_existe(&mut tx, id).await?
        {
            return Err(ErreurBranding::EtablissementInconnu);
        }

        let avant = repository::ecrire(
            &mut tx,
            tenant_id,
            demande.id,
            demande.etablissement_id,
            &demande.contenu,
        )
        .await?;

        let champs_touches = champs_touches(avant.as_ref(), &demande.contenu);
        if !champs_touches.is_empty() {
            self.outbox
                .ecrire(
                    &mut tx,
                    EvenementAEcrire {
                        id: Uuid::now_v7(),
                        tenant_id,
                        etablissement_id: demande.etablissement_id,
                        type_evenement: TYPE_MODIFIE.to_owned(),
                        agregat: AGREGAT.to_owned(),
                        agregat_id: demande.id,
                        version_schema: VERSION_SCHEMA,
                        payload: json!({
                            "niveau": if demande.etablissement_id.is_some() {
                                "ETABLISSEMENT"
                            } else {
                                "TENANT"
                            },
                            "etablissement_id": demande.etablissement_id,
                            "champs_touches": champs_touches,
                            // **La clé d'objet, jamais le binaire.** Le grand livre est à
                            // rétention illimitée : y écrire des logos le ferait grossir sans fin
                            // pour une information que le stockage objet porte déjà.
                            "logo_objet_cle": demande.contenu.logo_objet_cle,
                        }),
                    },
                )
                .await?;
        }

        let apres = repository::lire_niveau(&mut tx, demande.etablissement_id)
            .await?
            .unwrap_or_default();

        tx.commit().await?;
        Ok(apres)
    }

    /// Enregistre la clé d'objet d'un logo téléversé.
    pub async fn poser_logo(
        &self,
        tenant_id: Uuid,
        etablissement_id: Option<Uuid>,
        objet_cle: &str,
    ) -> Result<(), ErreurBranding> {
        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, tenant_id).await?;

        if let Some(id) = etablissement_id
            && !repository::etablissement_existe(&mut tx, id).await?
        {
            return Err(ErreurBranding::EtablissementInconnu);
        }

        repository::poser_logo(
            &mut tx,
            tenant_id,
            Uuid::now_v7(),
            etablissement_id,
            objet_cle,
        )
        .await?;

        self.outbox
            .ecrire(
                &mut tx,
                EvenementAEcrire {
                    id: Uuid::now_v7(),
                    tenant_id,
                    etablissement_id,
                    type_evenement: TYPE_MODIFIE.to_owned(),
                    agregat: AGREGAT.to_owned(),
                    agregat_id: Uuid::now_v7(),
                    version_schema: VERSION_SCHEMA,
                    payload: json!({
                        "niveau": if etablissement_id.is_some() { "ETABLISSEMENT" } else { "TENANT" },
                        "etablissement_id": etablissement_id,
                        "champs_touches": ["logo_objet_cle"],
                        "logo_objet_cle": objet_cle,
                    }),
                },
            )
            .await?;

        tx.commit().await?;
        Ok(())
    }
}

/// Fusionne deux niveaux, **champ par champ**, et rend l'origine de chacun.
///
/// C'est toute la logique de surcharge partielle, et elle tient en six lignes parce que la
/// nullabilité des colonnes la porte déjà.
fn fusionner(tenant: Option<BrandingNiveau>, etablissement: Option<BrandingNiveau>) -> BrandingResolu {
    let t = tenant.unwrap_or_default();
    let e = etablissement.unwrap_or_default();

    let champ = |niveau_bas: Option<String>, niveau_haut: Option<String>| -> Option<ChampResolu> {
        match (niveau_bas, niveau_haut) {
            (Some(v), _) => Some(ChampResolu {
                valeur: v,
                origine: "ETABLISSEMENT".to_owned(),
            }),
            (None, Some(v)) => Some(ChampResolu {
                valeur: v,
                origine: "TENANT".to_owned(),
            }),
            (None, None) => None,
        }
    };

    BrandingResolu {
        logo_objet_cle: champ(e.logo_objet_cle, t.logo_objet_cle),
        couleur_primaire: champ(e.couleur_primaire, t.couleur_primaire),
        entete_document: champ(e.entete_document, t.entete_document),
        pied_document: champ(e.pied_document, t.pied_document),
        mentions_legales: champ(e.mentions_legales, t.mentions_legales),
        coordonnees: champ(e.coordonnees, t.coordonnees),
    }
}

/// Quels champs ont changé — pour la charge utile de l'événement.
fn champs_touches(avant: Option<&BrandingNiveau>, apres: &BrandingNiveau) -> Vec<&'static str> {
    let vide = BrandingNiveau::default();
    let avant = avant.unwrap_or(&vide);
    let mut touches = Vec::new();

    if avant.logo_objet_cle != apres.logo_objet_cle {
        touches.push("logo_objet_cle");
    }
    if avant.couleur_primaire != apres.couleur_primaire {
        touches.push("couleur_primaire");
    }
    if avant.entete_document != apres.entete_document {
        touches.push("entete_document");
    }
    if avant.pied_document != apres.pied_document {
        touches.push("pied_document");
    }
    if avant.mentions_legales != apres.mentions_legales {
        touches.push("mentions_legales");
    }
    if avant.coordonnees != apres.coordonnees {
        touches.push("coordonnees");
    }

    touches
}

/// Rend le **document de test** — un aperçu, **sans rien enregistrer**.
///
/// # La mention non fiscale n'est pas optionnelle
///
/// L'aperçu porte le logo, l'en-tête, les coordonnées et les mentions légales de l'exploitant : il
/// ressemble à une facture. La mention « Document non fiscal — ne tient pas lieu de facture » est
/// ce qui empêche le premier aperçu imprimé d'être présenté à un client comme un justificatif.
///
/// Elle est **ajoutée ici**, pas laissée à l'appelant : un appelant peut l'oublier, une fonction
/// qui la concatène ne le peut pas.
pub fn rendre_document_test(identite: &BrandingResolu, nom_etablissement: &str) -> String {
    let valeur = |champ: &Option<ChampResolu>| -> String {
        champ.as_ref().map(|c| c.valeur.clone()).unwrap_or_default()
    };

    let mut document = String::new();
    document.push_str(&format!("{}\n", valeur(&identite.entete_document)));
    document.push_str(&format!("{nom_etablissement}\n"));

    let coordonnees = valeur(&identite.coordonnees);
    if !coordonnees.is_empty() {
        document.push_str(&format!("{coordonnees}\n"));
    }

    document.push_str("\n--- Aperçu de votre identité visuelle ---\n\n");

    let mentions = valeur(&identite.mentions_legales);
    if !mentions.is_empty() {
        document.push_str(&format!("{mentions}\n"));
    }

    let pied = valeur(&identite.pied_document);
    if !pied.is_empty() {
        document.push_str(&format!("{pied}\n"));
    }

    // **Toujours en dernier, toujours présente.**
    document.push_str(&format!("\n{MENTION_NON_FISCALE}\n"));
    document
}
