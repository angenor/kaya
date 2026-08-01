//! Lecture filtrée du registre des actions — **quatre filtres combinables, un curseur**.
//!
//! # Aucun tri sur `horodatage_client`, jamais
//!
//! La colonne existe et est rendue à part. Trier dessus ferait remonter en tête l'entrée d'un
//! appareil mal réglé — un téléphone en avance de deux heures apparaîtrait comme la dernière
//! action de la journée, tous les jours. Le registre est lu par un propriétaire qui cherche « ce
//! qui vient de se passer » : le seul tri défendable est l'**horodatage d'autorité serveur**
//! (principe IV).
//!
//! # La pagination est par CURSEUR, pas par décalage
//!
//! `(cree_le DESC, id DESC)`. Un `OFFSET` sur un registre qui grossit pendant qu'on le lit fait
//! sauter des lignes : une entrée écrite entre la page 1 et la page 2 décale tout, et la dernière
//! ligne de la page 1 réapparaît en tête de la page 2 — ou disparaît. Sur un registre d'audit,
//! une entrée sautée est exactement celle qu'on cherchait.
//!
//! Le couple `(cree_le, id)` est **total** : `cree_le` seul ne l'est pas, deux entrées de la même
//! transaction partagent la microseconde. `id` est un UUID v7, donc croissant dans le temps, ce
//! qui rend le second critère cohérent avec le premier plutôt qu'arbitraire.
//!
//! # Ce que ce fichier ne fait PAS
//!
//! Aucune écriture — elle est dans `service.rs`, et elle prend la transaction de l'opération
//! tracée. Aucune suppression, aucune correction : la table n'a que `SELECT, INSERT`, et une
//! correction est une nouvelle entrée (FR-033).

use time::OffsetDateTime;
use uuid::Uuid;

use super::modele::{EntreeAuditEnregistree, ErreurAudit, TypeActionAudit};

/// Plafond de page — **une borne de sûreté, pas une politique**.
///
/// Une page de mille entrées ferait un document de plusieurs mégaoctets sur le réseau
/// d'Abengourou. Le client demande ce qu'il veut ; le serveur ne rend jamais plus que ceci.
pub const LIMITE_MAX: i64 = 100;

/// Taille de page par défaut.
pub const LIMITE_DEFAUT: i64 = 50;

/// Les quatre filtres du contrat — **combinables**, chacun optionnel.
///
/// `None` signifie « pas de filtre ». La forme `($n IS NULL OR colonne = $n)` évite de construire
/// la requête par concaténation, ce qui imposerait `AssertSqlSafe` sur le chemin qui lit le
/// registre censé prouver ce qui s'est passé.
#[derive(Debug, Clone, Default)]
pub struct FiltresAudit {
    pub auteur_compte_id: Option<Uuid>,
    pub etablissement_id: Option<Uuid>,
    pub type_action: Option<TypeActionAudit>,
    /// Borne **inclusive** de début, sur l'horodatage d'autorité.
    pub depuis: Option<OffsetDateTime>,
    /// Borne **exclusive** de fin. Exclusive pour qu'une journée se demande `[J, J+1)` sans se
    /// chevaucher avec la suivante — le piège classique des intervalles de dates fermés.
    pub jusqu_a: Option<OffsetDateTime>,
}

/// Position dans la page suivante. `None` en tête de liste.
#[derive(Debug, Clone, Copy)]
pub struct Curseur {
    pub cree_le: OffsetDateTime,
    pub id: Uuid,
}

/// Une page du registre.
#[derive(Debug, Clone)]
pub struct PageAudit {
    pub elements: Vec<EntreeAuditEnregistree>,
    /// Curseur de la page suivante, ou `None` s'il n'y en a pas.
    pub suivant: Option<Curseur>,
}

/// Lit une page du registre.
///
/// # Le `+1` qui décide s'il y a une suite
///
/// La requête demande `limite + 1` lignes. Si elle en rend autant, il y a une page suivante et la
/// dernière est **jetée** ; sinon, on est au bout. C'est moins cher qu'un `COUNT(*)` sur une table
/// à rétention illimitée, et surtout c'est **exact** — un décompte total serait périmé à l'instant
/// où il est rendu.
pub async fn lister(
    tx: &mut sqlx::PgTransaction<'_>,
    filtres: &FiltresAudit,
    curseur: Option<Curseur>,
    limite: i64,
) -> Result<PageAudit, ErreurAudit> {
    let limite = limite.clamp(1, LIMITE_MAX);
    let type_action = filtres.type_action.map(|t| t.code());

    // Le curseur est éclaté en deux paramètres : `sqlx` lie des scalaires, et une comparaison de
    // n-uplets `(cree_le, id) < ($5, $6)` ne se paramètre pas aussi simplement. La condition
    // écrite ci-dessous en est l'équivalent exact — et elle se relit sans connaître la sémantique
    // des n-uplets SQL.
    let (curseur_date, curseur_id) = match curseur {
        Some(c) => (Some(c.cree_le), Some(c.id)),
        None => (None, None),
    };

    let lignes = sqlx::query!(
        r#"
        SELECT id, etablissement_id, type_action, auteur_compte_id,
               cible_type, cible_id, contexte, horodatage_client, cree_le
        FROM comptes.journal_audit
        WHERE ($1::uuid IS NULL OR auteur_compte_id = $1)
          AND ($2::uuid IS NULL OR etablissement_id = $2)
          AND ($3::text IS NULL OR type_action = $3)
          AND ($4::timestamptz IS NULL OR cree_le >= $4)
          AND ($5::timestamptz IS NULL OR cree_le < $5)
          AND ($6::timestamptz IS NULL
               OR cree_le < $6
               OR (cree_le = $6 AND id < $7))
        ORDER BY cree_le DESC, id DESC
        LIMIT $8
        "#,
        filtres.auteur_compte_id,
        filtres.etablissement_id,
        type_action,
        filtres.depuis,
        filtres.jusqu_a,
        curseur_date,
        curseur_id,
        limite + 1,
    )
    .fetch_all(&mut **tx)
    .await?;

    let suite = lignes.len() as i64 > limite;
    let page: Vec<_> = lignes.into_iter().take(limite as usize).collect();

    // **Le curseur se prend sur la dernière ligne LUE, avant tout écartement.** C'est la position
    // réelle dans le registre : le prendre après aurait fait sauter, au chargement suivant, toutes
    // les lignes écartées en queue de page.
    let suivant = if suite {
        page.last().map(|l| Curseur {
            cree_le: l.cree_le,
            id: l.id,
        })
    }
    else {
        None
    };

    let elements = page
        .into_iter()
        .filter_map(|l| {
            // Un code écrit par une version ultérieure du produit — cas réel en auto-hébergé, où
            // les binaires ne sont pas tous à jour au même instant — **ne fait pas tomber la
            // lecture du registre entier**. La ligne est écartée de l'affichage et signalée aux
            // journaux ; elle reste en base, où elle est immuable et relisible par la version qui
            // la comprend.
            let Some(type_action) = TypeActionAudit::depuis_code(&l.type_action) else {
                tracing::warn!(
                    entree.id = %l.id,
                    type_action = %l.type_action,
                    "type d'action inconnu au registre — entrée écartée de l'affichage, jamais de la base"
                );
                return None;
            };

            Some(EntreeAuditEnregistree {
                id: l.id,
                etablissement_id: l.etablissement_id,
                type_action,
                auteur_compte_id: l.auteur_compte_id,
                cible_type: l.cible_type,
                cible_id: l.cible_id,
                contexte: l.contexte,
                horodatage_client: l.horodatage_client,
                cree_le: l.cree_le,
            })
        })
        .collect();

    Ok(PageAudit { elements, suivant })
}

/// Les identifiants d'auteurs d'une page — **pour la résolution en lot**.
///
/// Extraits ici plutôt que dans le handler : c'est la seule chose que le repository sait de la
/// façon dont la page sera affichée, et elle évite au handler de connaître la structure interne
/// des entrées.
pub fn auteurs(page: &PageAudit) -> Vec<Uuid> {
    let mut ids: Vec<Uuid> = page.elements.iter().map(|e| e.auteur_compte_id).collect();
    ids.sort_unstable();
    ids.dedup();
    ids
}
