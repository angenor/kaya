//! Accès aux données du séjour — **macros littérales, transaction en paramètre**.
//!
//! Les trois règles du module doré, couche 3, sans aménagement : requêtes littérales vérifiées à
//! la compilation (porte P-18), le repository **prend** la transaction sans l'ouvrir, et aucune
//! jointure entre schémas de modules (porte P-04).
//!
//! # ⚠️ Aucune requête de ce fichier ne nomme le schéma `comptes`
//!
//! Un séjour affiche toujours le nom de son client — c'est la jointure que tout le monde
//! écrirait. Elle n'existe pas : le nom vient du trait `AnnuaireClients`, résolu **par lot** au
//! service. La porte P-04 déclare `comptes × hebergement` comme paire sensible et vérifie que ses
//! deux côtés ont bien des requêtes à inspecter.

use time::OffsetDateTime;
use uuid::Uuid;

use super::modele::{Accompagnant, NouvelAccompagnant, Sejour, StatutSejour};
use crate::erreurs::ErreurSejour;

/// Insère un séjour, ou constate qu'il existe déjà.
///
/// `ON CONFLICT (id) DO NOTHING ... RETURNING` renvoie une ligne quand l'insertion a eu lieu, et
/// **rien** en cas de conflit : c'est exactement ce qu'il faut pour distinguer `201` de `200` sans
/// second aller-retour dans le cas normal.
pub async fn inserer(
    tx: &mut sqlx::PgTransaction<'_>,
    tenant_id: Uuid,
    id: Uuid,
    etablissement_id: Uuid,
    client_id: Option<Uuid>,
    horodatage_client: Option<OffsetDateTime>,
) -> Result<bool, ErreurSejour> {
    let insere = sqlx::query_scalar!(
        r#"
        INSERT INTO hebergement.sejour
            (id, tenant_id, etablissement_id, client_id, horodatage_client)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (id) DO NOTHING
        RETURNING id
        "#,
        id,
        tenant_id,
        etablissement_id,
        client_id,
        horodatage_client,
    )
    .fetch_optional(&mut **tx)
    .await?;

    Ok(insere.is_some())
}

/// Lit un séjour du tenant courant.
pub async fn lire(
    tx: &mut sqlx::PgTransaction<'_>,
    id: Uuid,
) -> Result<Option<Sejour>, ErreurSejour> {
    let ligne = sqlx::query!(
        r#"
        SELECT id, etablissement_id, client_id, statut, ouvert_le, clos_le
        FROM hebergement.sejour
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(&mut **tx)
    .await?;

    Ok(ligne.map(|l| Sejour {
        id: l.id,
        etablissement_id: l.etablissement_id,
        client_id: l.client_id,
        // Un code inconnu vaut « en cours » plutôt que de faire tomber la lecture : une ligne
        // écrite par une version ultérieure du produit — cas réel en mode auto-hébergé — ne doit
        // pas rendre la liste entière illisible. Le classement le plus prudent est celui qui
        // n'affirme pas qu'un séjour est terminé.
        statut: StatutSejour::depuis_code(&l.statut).unwrap_or(StatutSejour::EnCours),
        ouvert_le: l.ouvert_le,
        clos_le: l.clos_le,
    }))
}

/// Pose `sejour_id` sur une occupation.
///
/// Séparé de l'insertion de l'occupation, qui appartient au moteur du cycle 004 : le rattachement
/// est le fait du séjour, et le faire porter par `DemandeAttribution` aurait donné au moteur de
/// disponibilité une notion de séjour qu'il n'a pas à connaître.
pub async fn rattacher_occupation(
    tx: &mut sqlx::PgTransaction<'_>,
    occupation_id: Uuid,
    sejour_id: Uuid,
) -> Result<(), ErreurSejour> {
    sqlx::query!(
        r#"UPDATE hebergement.occupation SET sejour_id = $2 WHERE id = $1"#,
        occupation_id,
        sejour_id,
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

/// Rattache un client à un séjour, **sans rien remettre en cause** (FR-028).
///
/// Le rattachement ultérieur d'une identité ne rouvre pas le séjour et ne touche pas à
/// l'attribution : c'est le parcours normal du passage, où la pièce vient après la clé.
///
/// Rend `false` quand aucune ligne n'a été touchée — séjour inconnu **ou** d'un autre tenant, deux
/// cas que la politique de sécurité rend volontairement indistinguables.
pub async fn rattacher_client(
    tx: &mut sqlx::PgTransaction<'_>,
    sejour_id: Uuid,
    client_id: Uuid,
) -> Result<bool, ErreurSejour> {
    let touche = sqlx::query_scalar!(
        r#"
        UPDATE hebergement.sejour
        SET client_id = $2, modifie_le = now()
        WHERE id = $1
        RETURNING id
        "#,
        sejour_id,
        client_id,
    )
    .fetch_optional(&mut **tx)
    .await?;

    Ok(touche.is_some())
}

/// Les séjours **en cours** d'un établissement, du plus récent au plus ancien.
///
/// Rend aussi l'unité et la fin prévue de l'occupation active — c'est ce que l'écran de départ
/// affiche, et les chercher séparément produirait N+1 requêtes.
pub struct SejourEnCours {
    pub sejour: Sejour,
    pub unite_id: Option<Uuid>,
    pub fin_prevue: Option<OffsetDateTime>,
}

pub async fn lister(
    tx: &mut sqlx::PgTransaction<'_>,
    etablissement_id: Uuid,
    seulement_en_cours: bool,
) -> Result<Vec<SejourEnCours>, ErreurSejour> {
    // La jointure porte sur **deux tables du même schéma** : légale (principe II).
    // `DISTINCT ON` retient l'occupation la plus récente d'un séjour qui en porte plusieurs —
    // cas du changement d'unité, où l'écran doit montrer la chambre ACTUELLE.
    let lignes = sqlx::query!(
        r#"
        SELECT DISTINCT ON (s.id)
               s.id, s.etablissement_id, s.client_id, s.statut, s.ouvert_le, s.clos_le,
               o.unite_id     AS "unite_id?",
               o.fin_client   AS "fin_prevue?"
        FROM hebergement.sejour s
        LEFT JOIN hebergement.occupation o ON o.sejour_id = s.id
        WHERE s.etablissement_id = $1
          AND ($2 = false OR s.statut = 'en_cours')
        ORDER BY s.id, o.cree_le DESC
        "#,
        etablissement_id,
        seulement_en_cours,
    )
    .fetch_all(&mut **tx)
    .await?;

    let mut liste: Vec<SejourEnCours> = lignes
        .into_iter()
        .map(|l| SejourEnCours {
            sejour: Sejour {
                id: l.id,
                etablissement_id: l.etablissement_id,
                client_id: l.client_id,
                statut: StatutSejour::depuis_code(&l.statut).unwrap_or(StatutSejour::EnCours),
                ouvert_le: l.ouvert_le,
                clos_le: l.clos_le,
            },
            unite_id: l.unite_id,
            fin_prevue: l.fin_prevue,
        })
        .collect();

    // `DISTINCT ON` impose son propre `ORDER BY` ; le tri d'affichage se fait donc ici.
    liste.sort_by(|a, b| b.sejour.ouvert_le.cmp(&a.sejour.ouvert_le));
    Ok(liste)
}

/// L'historique des séjours d'un client — **servi depuis `hebergement`, jamais depuis `comptes`**.
///
/// ⚠️ Si `socle/comptes` lisait cette table, ce serait **deux violations d'un coup** : jointure
/// inter-schémas (**P-04**) *et* arête `socle/ → verticales/` (**P-03**). Le chemin HTTP
/// `/api/v1/clients/{id}/sejours` cache ce découpage à l'appelant, et c'est normal — le contrat
/// est une façade, pas une carte des crates.
///
/// La requête emploie `sejour_par_client_idx`.
pub async fn historique_du_client(
    tx: &mut sqlx::PgTransaction<'_>,
    client_id: Uuid,
    limite: i64,
) -> Result<Vec<SejourEnCours>, ErreurSejour> {
    let lignes = sqlx::query!(
        r#"
        SELECT DISTINCT ON (s.id)
               s.id, s.etablissement_id, s.client_id, s.statut, s.ouvert_le, s.clos_le,
               o.unite_id   AS "unite_id?",
               o.fin_client AS "fin_prevue?"
        FROM hebergement.sejour s
        LEFT JOIN hebergement.occupation o ON o.sejour_id = s.id
        WHERE s.client_id = $1
        ORDER BY s.id, o.cree_le DESC
        LIMIT $2
        "#,
        client_id,
        limite,
    )
    .fetch_all(&mut **tx)
    .await?;

    let mut liste: Vec<SejourEnCours> = lignes
        .into_iter()
        .map(|l| SejourEnCours {
            sejour: Sejour {
                id: l.id,
                etablissement_id: l.etablissement_id,
                client_id: l.client_id,
                statut: StatutSejour::depuis_code(&l.statut).unwrap_or(StatutSejour::EnCours),
                ouvert_le: l.ouvert_le,
                clos_le: l.clos_le,
            },
            unite_id: l.unite_id,
            fin_prevue: l.fin_prevue,
        })
        .collect();

    liste.sort_by(|a, b| b.sejour.ouvert_le.cmp(&a.sejour.ouvert_le));
    Ok(liste)
}

/// Clôt un séjour à l'instant d'autorité de la base.
///
/// ⚠️ **`now()` en SQL, jamais l'horloge du processus** (porte P-23). Deux instances d'API n'ont
/// pas la même horloge ; la base en a une seule. Rend l'instant posé, pour que l'appelant l'écrive
/// au constat sans le relire.
///
/// Rend `None` quand le séjour était **déjà clos** — la condition `statut = 'en_cours'` en fait un
/// refus de la base, pas d'une lecture préalable.
pub async fn clore(
    tx: &mut sqlx::PgTransaction<'_>,
    sejour_id: Uuid,
) -> Result<Option<OffsetDateTime>, ErreurSejour> {
    let instant = sqlx::query_scalar!(
        r#"
        UPDATE hebergement.sejour
        SET statut = 'clos', clos_le = now(), modifie_le = now()
        WHERE id = $1 AND statut = 'en_cours'
        RETURNING clos_le
        "#,
        sejour_id
    )
    .fetch_optional(&mut **tx)
    .await?;

    Ok(instant.flatten())
}

// =================================================================================================
//  Accompagnants — classe A
// =================================================================================================

/// Ajoute un accompagnant, ou constate qu'il existe déjà.
pub async fn ajouter_accompagnant(
    tx: &mut sqlx::PgTransaction<'_>,
    tenant_id: Uuid,
    sejour_id: Uuid,
    nouveau: &NouvelAccompagnant,
) -> Result<bool, ErreurSejour> {
    // `piece_capturee_le` n'est posé **que** si une pièce arrive : le poser sans pièce ferait
    // démarrer la rétention de TRX-06 sur une donnée qui n'existe pas.
    let capturee_le = nouveau
        .numero_piece
        .as_ref()
        .map(|_| OffsetDateTime::now_utc());

    let insere = sqlx::query_scalar!(
        r#"
        INSERT INTO hebergement.accompagnant
            (id, tenant_id, sejour_id, nom, prenoms, date_naissance, nationalite,
             type_piece, numero_piece, piece_capturee_le, horodatage_client)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        ON CONFLICT (id) DO NOTHING
        RETURNING id
        "#,
        nouveau.id,
        tenant_id,
        sejour_id,
        nouveau.nom,
        nouveau.prenoms,
        nouveau.date_naissance,
        nouveau.nationalite,
        nouveau.type_piece,
        nouveau.numero_piece,
        capturee_le,
        nouveau.horodatage_client,
    )
    .fetch_optional(&mut **tx)
    .await?;

    Ok(insere.is_some())
}

/// Retire un accompagnant — **`retire_le`, jamais un `DELETE`**.
///
/// Sans cela, la fiche de police perdrait la trace d'une personne qui a bien été déclarée, et un
/// registre légal qui perd une déclaration est un document faux devant la gendarmerie.
///
/// Rend `false` sur un accompagnant déjà retiré : c'est un **rejeu**, pas une erreur.
pub async fn retirer_accompagnant(
    tx: &mut sqlx::PgTransaction<'_>,
    accompagnant_id: Uuid,
) -> Result<bool, ErreurSejour> {
    let touche = sqlx::query_scalar!(
        r#"
        UPDATE hebergement.accompagnant
        SET retire_le = now()
        WHERE id = $1 AND retire_le IS NULL
        RETURNING id
        "#,
        accompagnant_id
    )
    .fetch_optional(&mut **tx)
    .await?;

    Ok(touche.is_some())
}

/// Lit un accompagnant.
pub async fn lire_accompagnant(
    tx: &mut sqlx::PgTransaction<'_>,
    id: Uuid,
) -> Result<Option<Accompagnant>, ErreurSejour> {
    let ligne = sqlx::query!(
        r#"
        SELECT id, sejour_id, nom, prenoms,
               (numero_piece IS NOT NULL) AS "piece_enregistree!",
               retire_le, cree_le
        FROM hebergement.accompagnant
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(&mut **tx)
    .await?;

    Ok(ligne.map(|l| Accompagnant {
        id: l.id,
        sejour_id: l.sejour_id,
        nom: l.nom,
        prenoms: l.prenoms,
        piece_enregistree: l.piece_enregistree,
        retire_le: l.retire_le,
        cree_le: l.cree_le,
    }))
}

/// Les accompagnants **non retirés** d'un séjour.
pub async fn accompagnants(
    tx: &mut sqlx::PgTransaction<'_>,
    sejour_id: Uuid,
) -> Result<Vec<Accompagnant>, ErreurSejour> {
    let lignes = sqlx::query!(
        r#"
        SELECT id, sejour_id, nom, prenoms,
               (numero_piece IS NOT NULL) AS "piece_enregistree!",
               retire_le, cree_le
        FROM hebergement.accompagnant
        WHERE sejour_id = $1 AND retire_le IS NULL
        ORDER BY cree_le, id
        "#,
        sejour_id
    )
    .fetch_all(&mut **tx)
    .await?;

    Ok(lignes
        .into_iter()
        .map(|l| Accompagnant {
            id: l.id,
            sejour_id: l.sejour_id,
            nom: l.nom,
            prenoms: l.prenoms,
            piece_enregistree: l.piece_enregistree,
            retire_le: l.retire_le,
            cree_le: l.cree_le,
        })
        .collect())
}

/// Le nombre de personnes d'un séjour — **dérivé**, jamais saisi (FR-018).
///
/// Le titulaire compte pour un, plus les accompagnants non retirés. Une colonne
/// `nombre_personnes` sur `sejour` se désynchroniserait au premier retrait, et le constat de taxe
/// en porterait la valeur fausse **pour toujours** — il est figé.
pub async fn nombre_personnes(
    tx: &mut sqlx::PgTransaction<'_>,
    sejour_id: Uuid,
) -> Result<i32, ErreurSejour> {
    let accompagnants = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) AS "compte!"
        FROM hebergement.accompagnant
        WHERE sejour_id = $1 AND retire_le IS NULL
        "#,
        sejour_id
    )
    .fetch_one(&mut **tx)
    .await?;

    Ok(1 + i32::try_from(accompagnants).unwrap_or(i32::MAX - 1))
}

/// L'instant d'autorité de la base — **jamais l'horloge du processus** (porte P-23).
pub async fn maintenant(
    tx: &mut sqlx::PgTransaction<'_>,
) -> Result<OffsetDateTime, ErreurSejour> {
    let instant = sqlx::query_scalar!(r#"SELECT now() AS "maintenant!""#)
        .fetch_one(&mut **tx)
        .await?;
    Ok(instant)
}

// =================================================================================================
//  La file de réconciliation — ★ le cas orphelin
// =================================================================================================

/// Inscrit une écriture orpheline à la file de réconciliation.
///
/// ★ **C'est le premier écrivain de `synchronisation.reconciliation_orpheline`**, posée au
/// cycle 005 avec `GRANT SELECT` seul. Elle cesse d'être une provision.
///
/// # La charge utile est du JSON **opaque**
///
/// `kaya_synchronisation` ne doit connaître ni `Accompagnant`, ni `Sejour` (porte **P-03**) :
/// faire remonter un type de verticale dans une signature du socle est le piège concret que
/// `contracts/traits-exposes.md` désigne nommément. La charge utile est donc composée **ici**, du
/// côté de la verticale, et traverse en `serde_json::Value`.
///
/// # ⚠️ Elle porte le nom, et c'est tout l'objet
///
/// Le séjour étant clos, la ligne `hebergement.accompagnant` **n'est pas écrite**. Sans charge
/// utile, la file ne retiendrait que des identifiants et SYN-03 n'aurait rien à rattacher — le
/// défaut que la migration `0034` corrige.
pub async fn inscrire_orpheline(
    tx: &mut sqlx::PgTransaction<'_>,
    tenant_id: Uuid,
    etablissement_id: Uuid,
    reconciliation_id: Uuid,
    ecriture_id: Uuid,
    ecriture_type: &str,
    sejour_id: Uuid,
    charge_utile: serde_json::Value,
    motif: &str,
    horodatage_client: Option<OffsetDateTime>,
) -> Result<(), ErreurSejour> {
    sqlx::query!(
        r#"
        INSERT INTO synchronisation.reconciliation_orpheline
            (id, tenant_id, etablissement_id, ecriture_id, ecriture_type,
             agregat_type, agregat_id, charge_utile, motif, horodatage_client)
        VALUES ($1, $2, $3, $4, $5, 'sejour', $6, $7, $8, $9)
        ON CONFLICT (id) DO NOTHING
        "#,
        reconciliation_id,
        tenant_id,
        etablissement_id,
        ecriture_id,
        ecriture_type,
        sejour_id,
        charge_utile,
        motif,
        horodatage_client,
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}
