//! Service de tarification — **la durée vient du serveur, jamais du terminal**.
//!
//! # Ce que ce service calcule, et ce qu'il ne fait pas
//!
//! > **Le moteur calcule, il ne facture pas.**
//!
//! Aucune ligne de note n'est écrite : la note est SEJ-03, tranche T2. Ce que ce service produit
//! est une **décision de tarification**.
//!
//! # FR-029 — l'horodatage d'autorité, et le piège du passage
//!
//! La durée réelle se calcule entre `occupation.cree_le` et `now()`, **tous deux lus en SQL**.
//! L'appel ne prend aucun instant en paramètre : un client ne peut donc pas influencer la durée
//! facturée, même avec une horloge décalée de quarante minutes.
//!
//! Le cadrage §11 le désigne nommément : « le passage aggrave la sensibilité à l'horloge ». Sur une
//! nuitée, un décalage d'une heure ne change rien au montant ; sur un passage à 1 500 F l'heure,
//! il en change un septième.
//!
//! # La rebascule est tracée au registre des actions, dans la MÊME transaction
//!
//! `TypeActionAudit::RebasculePalierPassage` existe à la taxonomie depuis le cycle 003, avec la
//! mention « **Dû par HEB-04** ». Ce service l'honore. C'est ce que M. Koffi lira : « Durée
//! dépassée : passé au tarif 4 h ».

use serde_json::json;
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use kaya_comptes::audit::{EntreeAudit, JournalAudit, TypeActionAudit};
use kaya_etablissements::tenant_context;
use kaya_etablissements::{EstablishmentDirectory, RegistreModules};

use super::bareme::{self, Palier};
use crate::occupation::ErreurAttribution;
use crate::referentiel::FamilleFormule;
use crate::traits::{DecisionTarification, Rebascule};
use crate::{MODULE_HEBERGEMENT, referentiel};

/// Service du calcul de tarif.
pub struct ServiceTarification<A, R, J>
where
    A: EstablishmentDirectory,
    R: RegistreModules,
    J: JournalAudit,
{
    pool: PgPool,
    tenant_id: Uuid,
    annuaire: A,
    modules: R,
    journal: J,
}

impl<A, R, J> ServiceTarification<A, R, J>
where
    A: EstablishmentDirectory,
    R: RegistreModules,
    J: JournalAudit,
{
    pub fn nouveau(pool: PgPool, tenant_id: Uuid, annuaire: A, modules: R, journal: J) -> Self {
        Self {
            pool,
            tenant_id,
            annuaire,
            modules,
            journal,
        }
    }

    /// Calcule le montant dû pour une occupation, **à l'instant d'autorité serveur**.
    ///
    /// `auteur_compte_id` sert à la trace de rebascule : le registre des actions dit **qui** a
    /// constaté le dépassement, et une entrée sans auteur ne répondrait pas à la question que le
    /// propriétaire pose.
    pub async fn calculer(
        &self,
        etablissement_id: Uuid,
        occupation_id: Uuid,
        auteur_compte_id: Uuid,
    ) -> Result<DecisionTarification, ErreurAttribution> {
        let etablissement = self
            .annuaire
            .etablissement(etablissement_id)
            .await?
            .ok_or(ErreurAttribution::EtablissementInconnu)?;

        if !self
            .modules
            .module_actif(etablissement_id, MODULE_HEBERGEMENT)
            .await?
        {
            return Err(ErreurAttribution::ServiceInactif);
        }

        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, self.tenant_id).await?;

        // ── La durée réelle, **entièrement calculée en SQL** ──────────────────────────────────
        //
        // `cree_le` et `now()` viennent tous deux de la base. Les rapatrier pour soustraire en
        // Rust donnerait le même résultat aujourd'hui et ouvrirait la porte au jour où quelqu'un
        // remplacerait `now()` par `OffsetDateTime::now_utc()` — l'horloge du processus, dont il
        // existe plusieurs instances.
        let mesure = sqlx::query!(
            r#"
            SELECT
                o.formule_id,
                o.cree_le,
                now() AS "instant_autorite!",
                (EXTRACT(EPOCH FROM (now() - o.cree_le)) / 60)::BIGINT AS "duree_minutes!",
                f.famille,
                f.prix_mineur,
                f.prix_heure_supplementaire_mineur,
                f.categorie_id
            FROM hebergement.occupation o
            JOIN hebergement.formule f ON f.id = o.formule_id
            WHERE o.id = $1
            "#,
            occupation_id
        )
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(ErreurAttribution::OccupationInconnue)?;

        let famille = FamilleFormule::depuis_code(&mesure.famille)?;

        // Une nuitée, un mois ou une demi-journée n'ont pas de barème : leur montant est leur prix
        // d'appel. Le calcul de palier ne les concerne pas, et prétendre le contraire produirait
        // un `bareme_absent` sur une formule parfaitement valide.
        if famille != FamilleFormule::Passage {
            tx.rollback().await?;
            return Ok(DecisionTarification {
                duree_reelle_minutes: mesure.duree_minutes,
                formule_appliquee: famille,
                palier_retenu_minutes: None,
                heures_supplementaires: 0,
                montant_du_mineur: mesure.prix_mineur,
                devise: etablissement.devise,
                rebascule: None,
                instant_autorite: mesure.instant_autorite,
            });
        }

        let paliers = sqlx::query!(
            r#"
            SELECT duree_minutes, prix_mineur
            FROM hebergement.bareme_palier
            WHERE formule_id = $1
            ORDER BY duree_minutes
            "#,
            mesure.formule_id
        )
        .fetch_all(&mut *tx)
        .await?
        .into_iter()
        .map(|p| Palier {
            duree_minutes: p.duree_minutes,
            prix_mineur: p.prix_mineur,
        })
        .collect::<Vec<_>>();

        // ── Le seuil de bascule et le prix de la nuitée — DES PARAMÈTRES, jamais des constantes ─
        //
        // Le seuil vient du catalogue de configuration (`seuil_bascule_nuitee_minutes`, migration
        // `0023`), et le prix de la nuitée de la formule `NUITEE` de la **même catégorie**. Écrire
        // « 480 » ici ferait de la pratique de Deloria une règle du produit.
        let seuil = self.seuil_bascule(&mut tx, etablissement_id).await?;
        let prix_nuitee = sqlx::query_scalar!(
            r#"
            SELECT prix_mineur
            FROM hebergement.formule
            WHERE categorie_id = $1 AND famille = 'NUITEE'
            "#,
            mesure.categorie_id
        )
        .fetch_optional(&mut *tx)
        .await?;

        let calcul = bareme::calculer(
            mesure.duree_minutes,
            &paliers,
            mesure.prix_heure_supplementaire_mineur,
            seuil,
            prix_nuitee,
        )
        .map_err(|_| ErreurAttribution::Referentiel(referentiel::ErreurReferentiel::BaremeAbsent))?;

        // ── La rebascule : le palier VENDU contre celui qui s'applique ────────────────────────
        //
        // Le palier vendu est celui que la durée **commerciale** annonçait — `fin_client` moins
        // `debut_client`. S'il diffère du palier retenu, il y a rebascule, et elle se trace.
        let rebascule = self
            .rebascule(&mut tx, occupation_id, &paliers, &calcul)
            .await?;

        if let Some(ref r) = rebascule {
            self.journal
                .tracer(
                    &mut tx,
                    self.tenant_id,
                    EntreeAudit {
                        id: Uuid::now_v7(),
                        etablissement_id: Some(etablissement_id),
                        type_action: TypeActionAudit::RebasculePalierPassage,
                        auteur_compte_id,
                        cible_type: "occupation".to_owned(),
                        cible_id: Some(occupation_id),
                        // **Nommage monétaire réservé** : toute clé monétaire porte `_mineur`, une
                        // valeur entière, et une clé `devise` au même niveau — `valider_contexte`
                        // du socle le fait échouer sinon.
                        contexte: json!({
                            "duree_reelle_minutes": mesure.duree_minutes,
                            "palier_vendu_minutes": r.palier_vendu_minutes,
                            "palier_retenu_minutes": calcul.palier_retenu_minutes,
                            "montant_vendu_mineur": r.montant_vendu_mineur,
                            "montant_du_mineur": calcul.montant_du_mineur,
                            "difference_mineur": r.difference_mineur,
                            "devise": etablissement.devise,
                        }),
                        horodatage_client: None,
                    },
                )
                .await
                .map_err(|e| {
                    tracing::error!(erreur = %e, "trace de rebascule impossible");
                    ErreurAttribution::Base(sqlx::Error::PoolClosed)
                })?;
        }

        // **La transaction est validée** parce qu'elle porte une écriture : la trace d'audit. Sans
        // rebascule, il n'y a rien à écrire — mais valider une transaction de lecture ne coûte
        // rien et évite un chemin de sortie de plus.
        tx.commit().await?;

        Ok(DecisionTarification {
            duree_reelle_minutes: mesure.duree_minutes,
            formule_appliquee: calcul.formule_appliquee,
            palier_retenu_minutes: calcul.palier_retenu_minutes,
            heures_supplementaires: calcul.heures_supplementaires,
            montant_du_mineur: calcul.montant_du_mineur,
            devise: etablissement.devise,
            rebascule,
            instant_autorite: mesure.instant_autorite,
        })
    }

    /// Le seuil de bascule en nuitée, **lu de la configuration héritée**.
    ///
    /// `None` quand aucune valeur n'est posée : le résolveur rend `Option`, **jamais un défaut**
    /// (principe I·c). Un établissement sans seuil ne bascule pas, et c'est un choix visible plutôt
    /// qu'une constante cachée dans ce fichier.
    async fn seuil_bascule(
        &self,
        tx: &mut sqlx::PgTransaction<'_>,
        etablissement_id: Uuid,
    ) -> Result<Option<i32>, ErreurAttribution> {
        // La lecture passe par la table de configuration du socle, dans le schéma
        // `etablissements` — une **seule** table nommée, sans jointure avec `hebergement` : ce
        // n'est pas une jointure inter-schémas, c'est une lecture de paramètre.
        let valeur = sqlx::query_scalar!(
            r#"
            SELECT valeur
            FROM etablissements.parametre_configuration
            WHERE cle = 'seuil_bascule_nuitee_minutes'
              AND etablissement_id = $1
            "#,
            etablissement_id
        )
        .fetch_optional(&mut **tx)
        .await?;

        Ok(valeur.and_then(|v| v.as_i64()).and_then(|n| i32::try_from(n).ok()))
    }

    /// La rebascule, **s'il y en a une**.
    async fn rebascule(
        &self,
        tx: &mut sqlx::PgTransaction<'_>,
        occupation_id: Uuid,
        paliers: &[Palier],
        calcul: &bareme::Calcul,
    ) -> Result<Option<Rebascule>, ErreurAttribution> {
        let vendue = sqlx::query_scalar!(
            r#"
            SELECT (EXTRACT(EPOCH FROM (fin_client - debut_client)) / 60)::BIGINT AS "minutes!"
            FROM hebergement.occupation
            WHERE id = $1
            "#,
            occupation_id
        )
        .fetch_one(&mut **tx)
        .await?;

        // Le palier que la durée commerciale annonçait. S'il n'y en a pas — durée vendue au-delà
        // du dernier palier —, il n'y a pas de « vendu » à comparer, donc pas de rebascule à
        // annoncer : le client savait déjà qu'il dépassait.
        let Some(palier_vendu) = paliers.iter().find(|p| i64::from(p.duree_minutes) >= vendue)
        else {
            return Ok(None);
        };

        let a_change = match calcul.palier_retenu_minutes {
            // Bascule en nuitée : c'est le cas de rebascule le plus fort.
            None => true,
            Some(retenu) => retenu != palier_vendu.duree_minutes,
        };

        if !a_change {
            return Ok(None);
        }

        Ok(Some(Rebascule {
            palier_vendu_minutes: palier_vendu.duree_minutes,
            montant_vendu_mineur: palier_vendu.prix_mineur,
            // **Peut être négatif** — un départ anticipé existe, et la différence le dit plutôt
            // que de la ramener à zéro.
            difference_mineur: calcul.montant_du_mineur - palier_vendu.prix_mineur,
        }))
    }
}

/// L'instant d'autorité, exposé pour les tests et le diagnostic.
///
/// L'horloge du processus applicatif n'est pas celle de la base, et deux instances d'API n'ont pas
/// la même — la base, elle, est unique.
pub async fn instant_autorite(pool: &PgPool) -> Result<OffsetDateTime, sqlx::Error> {
    sqlx::query_scalar!(r#"SELECT now() AS "maintenant!""#)
        .fetch_one(pool)
        .await
}
