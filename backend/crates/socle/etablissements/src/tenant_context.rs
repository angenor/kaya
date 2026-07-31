//! Pose du tenant courant — **le chemin de code le plus sensible du produit**.
//!
//! C'est lui qui décide quelles lignes un client voit. Tout ce qui suit est écrit pour qu'aucune
//! chaîne SQL ne soit jamais concaténée ici.
//!
//! # Pourquoi `set_config` et jamais `SET LOCAL`
//!
//! Le principe III le prescrit littéralement, et la raison est mécanique plutôt que stylistique.
//!
//! `SET LOCAL app.current_tenant = ...` est une **commande utilitaire** : elle n'accepte aucun
//! paramètre lié. L'employer imposerait d'interpoler l'identifiant du tenant dans la chaîne SQL,
//! donc de passer par `sqlx::raw_sql`, donc d'envelopper le tout dans `sqlx::AssertSqlSafe` —
//! obligatoire en sqlx 0.9 sur toute requête non littérale (`#3723`). On placerait alors une
//! concaténation de chaîne SQL sur le chemin exact qui décide de la visibilité des données.
//!
//! `set_config()` est une **fonction** : son argument se lie normalement, la requête reste
//! littérale, `query!` la vérifie à la compilation, et `AssertSqlSafe` n'apparaît jamais ici.
//!
//! Le troisième argument `true` donne exactement la sémantique de `SET LOCAL` : la valeur
//! retombe à la fin de la transaction. C'est ce qui rend le réglage compatible avec un pool de
//! connexions — un `SET` posé à l'ouverture de connexion survivrait au client suivant, et c'est
//! la différence exacte entre l'isolation et la fuite de données entre clients.

use uuid::Uuid;

/// Échec de pose du contexte de tenant.
#[derive(Debug, thiserror::Error)]
pub enum ErreurContexteTenant {
    #[error("impossible de poser le tenant courant : {0}")]
    Base(#[from] sqlx::Error),
}

/// Pose le tenant courant **pour la durée de la transaction**.
///
/// À appeler en tête de **chaque** transaction, jamais à l'ouverture de la connexion
/// (principe III).
///
/// Une transaction sans appel à cette fonction ne voit **aucune ligne** — pas une erreur, zéro
/// ligne. C'est la propriété exigée par le scénario US4.6, et elle vient du second argument
/// `true` de `current_setting` dans les politiques : le paramètre absent vaut NULL, la
/// comparaison vaut NULL, aucune ligne ne passe.
#[tracing::instrument(skip(tx), fields(tenant.id = %tenant_id))]
pub async fn poser_tenant(
    tx: &mut sqlx::PgTransaction<'_>,
    tenant_id: Uuid,
) -> Result<(), ErreurContexteTenant> {
    // `set_config` est une fonction : elle se lit avec `fetch_one`, pas avec `execute`. La
    // valeur renvoyée — le réglage tel qu'il vient d'être posé — n'a pas d'usage ici ; ce qui
    // compte est que l'appel ait réussi dans **cette** transaction.
    sqlx::query_scalar!(
        "SELECT set_config('app.current_tenant', $1, true)",
        tenant_id.to_string()
    )
    .fetch_one(&mut **tx)
    .await?;

    Ok(())
}

/// Lit le tenant courant de la transaction, s'il est posé.
///
/// Utile aux tests et au diagnostic. Renvoie `None` quand aucun contexte n'est posé — ce qui est
/// un état valide, pas une erreur : c'est celui d'une transaction qui ne verra rien.
pub async fn tenant_courant(
    tx: &mut sqlx::PgTransaction<'_>,
) -> Result<Option<Uuid>, ErreurContexteTenant> {
    let valeur: Option<String> =
        sqlx::query_scalar!("SELECT current_setting('app.current_tenant', true)")
            .fetch_one(&mut **tx)
            .await?;

    Ok(valeur
        .filter(|v| !v.is_empty())
        .and_then(|v| Uuid::parse_str(&v).ok()))
}
