//! Accès aux données des comptes — **deux chemins de lecture, et ils ne se croisent jamais**.
//!
//! Celui de l'authentification lit le condensat ; celui de l'affichage ne le sélectionne même pas.
//! La séparation est portée par les **requêtes**, pas seulement par les types : `lister` et `lire`
//! n'écrivent nulle part `condensat_mot_de_passe`, donc aucun remaniement de structure ne pourrait
//! le faire remonter par accident.

use uuid::Uuid;

use super::modele::{CompteAuthentification, CompteVue, CreerCompte, ErreurCompte, RolePorte};
use kaya_etablissements::Issue;

/// Résout un identifiant de connexion **avant que le tenant soit connu**.
///
/// Passe par `comptes.resoudre_identifiant`, fonction `SECURITY DEFINER` de la migration `0020` —
/// **la seule dérogation à l'isolation par tenant du produit**, dont le périmètre est la
/// signature. Voir cette migration pour les deux solutions écartées et leurs raisons.
///
/// La transaction n'a **pas** de tenant posé quand cette fonction est appelée : c'est le seul
/// endroit du produit où c'est normal.
pub async fn resoudre_identifiant(
    tx: &mut sqlx::PgTransaction<'_>,
    identifiant: &str,
) -> Result<Option<CompteAuthentification>, ErreurCompte> {
    let ligne = sqlx::query!(
        r#"
        SELECT compte_id              AS "compte_id!",
               tenant_id              AS "tenant_id!",
               condensat_mot_de_passe AS "condensat_mot_de_passe!",
               methode_code           AS "methode_code!",
               actif                  AS "actif!",
               personne_id            AS "personne_id!"
        FROM comptes.resoudre_identifiant($1)
        "#,
        identifiant
    )
    .fetch_optional(&mut **tx)
    .await?;

    Ok(ligne.map(|l| CompteAuthentification {
        id: l.compte_id,
        tenant_id: l.tenant_id,
        condensat_mot_de_passe: l.condensat_mot_de_passe,
        methode_code: l.methode_code,
        actif: l.actif,
        personne_id: l.personne_id,
    }))
}

/// Relit un compte pour l'authentification, **tenant déjà posé**.
///
/// Employée par le rafraîchissement, qui connaît le tenant par le jeton et n'a donc aucune raison
/// de passer par la dérogation.
pub async fn lire_pour_authentification(
    tx: &mut sqlx::PgTransaction<'_>,
    compte_id: Uuid,
) -> Result<Option<CompteAuthentification>, ErreurCompte> {
    let ligne = sqlx::query!(
        r#"
        SELECT id, tenant_id, condensat_mot_de_passe, methode_code, actif, personne_id
        FROM comptes.compte
        WHERE id = $1
        "#,
        compte_id
    )
    .fetch_optional(&mut **tx)
    .await?;

    Ok(ligne.map(|l| CompteAuthentification {
        id: l.id,
        tenant_id: l.tenant_id,
        condensat_mot_de_passe: l.condensat_mot_de_passe,
        methode_code: l.methode_code,
        actif: l.actif,
        personne_id: l.personne_id,
    }))
}

/// Remplace le condensat d'un compte.
///
/// Employée au changement de mot de passe **et** au rehachage silencieux après une connexion
/// réussie sur des paramètres périmés.
pub async fn ecrire_condensat(
    tx: &mut sqlx::PgTransaction<'_>,
    compte_id: Uuid,
    condensat: &str,
) -> Result<(), ErreurCompte> {
    sqlx::query!(
        r#"
        UPDATE comptes.compte
        SET condensat_mot_de_passe = $2, modifie_le = now()
        WHERE id = $1
        "#,
        compte_id,
        condensat
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

/// Insère un compte, ou constate qu'il existe déjà.
pub async fn inserer(
    tx: &mut sqlx::PgTransaction<'_>,
    tenant_id: Uuid,
    compte: &CreerCompte,
    condensat: &str,
) -> Result<(Uuid, Issue), ErreurCompte> {
    let insere = sqlx::query_scalar!(
        r#"
        INSERT INTO comptes.compte
            (id, tenant_id, personne_id, identifiant_telephone, identifiant_email,
             condensat_mot_de_passe, horodatage_client)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (id) DO NOTHING
        RETURNING id
        "#,
        compte.id,
        tenant_id,
        compte.personne_id,
        compte.identifiant_telephone,
        compte.identifiant_email,
        condensat,
        compte.horodatage_client,
    )
    .fetch_optional(&mut **tx)
    .await?;

    match insere {
        Some(id) => Ok((id, Issue::Creee)),
        None => Ok((compte.id, Issue::DejaPresente)),
    }
}

/// Change l'état d'activité d'un compte et rend **l'état précédent**.
///
/// Rend `None` si le compte est inconnu du tenant courant.
///
/// # Deux requêtes, et c'est délibéré
///
/// L'ancien état pourrait s'obtenir en une seule instruction, par une sous-requête dans le
/// `RETURNING` : PostgreSQL l'évaluerait sur l'instantané d'avant la mise à jour et rendrait bien
/// l'ancienne valeur. Cela **marche**, et personne ne peut le relire sans aller vérifier la
/// sémantique des instantanés — d'où deux requêtes lisibles, dans une transaction que l'appelant
/// tient déjà. Le tour de force ne se paie qu'à la relecture, et il se paie longtemps.
///
/// L'ancien état n'est pas décoratif : il décide s'il y a **transition**. Désactiver un compte
/// déjà inactif ne doit produire ni événement, ni entrée d'audit.
pub async fn changer_etat(
    tx: &mut sqlx::PgTransaction<'_>,
    compte_id: Uuid,
    actif: bool,
) -> Result<Option<bool>, ErreurCompte> {
    let ancien = sqlx::query_scalar!(
        r#"SELECT actif FROM comptes.compte WHERE id = $1"#,
        compte_id
    )
    .fetch_optional(&mut **tx)
    .await?;

    let Some(ancien) = ancien else {
        return Ok(None);
    };

    sqlx::query!(
        r#"UPDATE comptes.compte SET actif = $2, modifie_le = now() WHERE id = $1"#,
        compte_id,
        actif
    )
    .execute(&mut **tx)
    .await?;

    Ok(Some(ancien))
}

/// Lit un compte **pour l'affichage** — le condensat n'apparaît pas dans cette requête.
pub async fn lire(
    tx: &mut sqlx::PgTransaction<'_>,
    compte_id: Uuid,
) -> Result<Option<CompteVue>, ErreurCompte> {
    // La jointure avec `personne` est **intra-schéma** : les deux tables sont dans `comptes`.
    // C'est la seule forme de jointure que le principe II autorise (porte P-04).
    let ligne = sqlx::query!(
        r#"
        SELECT c.id, c.personne_id, c.identifiant_telephone, c.identifiant_email,
               c.methode_code, c.actif, c.cree_le, c.modifie_le,
               p.nom, p.prenoms
        FROM comptes.compte c
        JOIN comptes.personne p ON p.id = c.personne_id
        WHERE c.id = $1
        "#,
        compte_id
    )
    .fetch_optional(&mut **tx)
    .await?;

    let Some(l) = ligne else { return Ok(None) };

    let roles = roles_portes(tx, &[l.id]).await?;

    Ok(Some(CompteVue {
        id: l.id,
        personne_id: l.personne_id,
        nom_affichage: nom_affichage(&l.nom, l.prenoms.as_deref()),
        identifiant_telephone: l.identifiant_telephone,
        identifiant_email: l.identifiant_email,
        methode_code: l.methode_code,
        actif: l.actif,
        roles: roles.get(&l.id).cloned().unwrap_or_default(),
        cree_le: l.cree_le,
        modifie_le: l.modifie_le,
    }))
}

/// Liste les comptes du tenant courant, avec leurs rôles.
///
/// Les trois filtres du contrat sont **combinables**, et chacun est optionnel. `NULL` signifie
/// « pas de filtre » — la forme `($n IS NULL OR colonne = $n)` évite de construire la requête par
/// concaténation, ce qui imposerait `AssertSqlSafe` sur le chemin qui lit les comptes.
pub async fn lister(
    tx: &mut sqlx::PgTransaction<'_>,
    etablissement_id: Option<Uuid>,
    actif: Option<bool>,
    role_code: Option<&str>,
) -> Result<Vec<CompteVue>, ErreurCompte> {
    let lignes = sqlx::query!(
        r#"
        SELECT DISTINCT c.id, c.personne_id, c.identifiant_telephone, c.identifiant_email,
               c.methode_code, c.actif, c.cree_le, c.modifie_le,
               p.nom, p.prenoms
        FROM comptes.compte c
        JOIN comptes.personne p ON p.id = c.personne_id
        LEFT JOIN comptes.compte_role cr ON cr.compte_id = c.id
        WHERE ($1::uuid IS NULL OR cr.etablissement_id = $1)
          AND ($2::bool IS NULL OR c.actif = $2)
          AND ($3::text IS NULL OR cr.role_code = $3)
        ORDER BY p.nom, c.cree_le, c.id
        "#,
        etablissement_id,
        actif,
        role_code,
    )
    .fetch_all(&mut **tx)
    .await?;

    let ids: Vec<Uuid> = lignes.iter().map(|l| l.id).collect();
    let roles = roles_portes(tx, &ids).await?;

    Ok(lignes
        .into_iter()
        .map(|l| CompteVue {
            roles: roles.get(&l.id).cloned().unwrap_or_default(),
            id: l.id,
            personne_id: l.personne_id,
            nom_affichage: nom_affichage(&l.nom, l.prenoms.as_deref()),
            identifiant_telephone: l.identifiant_telephone,
            identifiant_email: l.identifiant_email,
            methode_code: l.methode_code,
            actif: l.actif,
            cree_le: l.cree_le,
            modifie_le: l.modifie_le,
        })
        .collect())
}

/// Les rôles portés par un lot de comptes — **une requête, pas une par compte**.
///
/// L'écran `G3` affiche une page de comptes ; sans lot, il ferait autant de requêtes que de
/// lignes. C'est le même raisonnement que pour `AnnuaireComptes::comptes`, et il est moins cher
/// de le fermer ici qu'à la première lenteur en clientèle.
pub async fn roles_portes(
    tx: &mut sqlx::PgTransaction<'_>,
    compte_ids: &[Uuid],
) -> Result<std::collections::BTreeMap<Uuid, Vec<RolePorte>>, ErreurCompte> {
    if compte_ids.is_empty() {
        return Ok(std::collections::BTreeMap::new());
    }

    let lignes = sqlx::query!(
        r#"
        SELECT cr.compte_id, cr.role_code, cr.etablissement_id
        FROM comptes.compte_role cr
        JOIN comptes.role r ON r.code = cr.role_code
        WHERE cr.compte_id = ANY($1)
        ORDER BY r.ordre, cr.etablissement_id
        "#,
        compte_ids
    )
    .fetch_all(&mut **tx)
    .await?;

    let mut carte: std::collections::BTreeMap<Uuid, Vec<RolePorte>> = Default::default();
    for ligne in lignes {
        carte.entry(ligne.compte_id).or_default().push(RolePorte {
            role_code: ligne.role_code,
            etablissement_id: ligne.etablissement_id,
        });
    }
    Ok(carte)
}

/// Compose le nom affichable — **depuis `personne`, jamais depuis l'identifiant de connexion**.
pub fn nom_affichage(nom: &str, prenoms: Option<&str>) -> String {
    match prenoms.map(str::trim).filter(|p| !p.is_empty()) {
        Some(prenoms) => format!("{prenoms} {nom}"),
        None => nom.to_owned(),
    }
}
