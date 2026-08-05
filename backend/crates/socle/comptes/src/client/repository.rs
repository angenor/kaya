//! Accès aux données de la fiche client — **macros littérales, transaction en paramètre**.
//!
//! Les trois règles du module doré, couche 3, sans aménagement :
//!
//!   * toutes les requêtes passent par `query!` / `query_as!` sur **littéral**, donc vérifiées à
//!     la compilation contre la vraie base (porte P-18) ; `AssertSqlSafe` n'apparaît nulle part ;
//!   * le repository **prend** la transaction, il ne l'ouvre pas — c'est le service qui décide de
//!     la portée, parce que c'est lui qui doit y inclure l'événement outbox ;
//!   * aucune jointure entre schémas de modules (porte P-04). La jointure `personne × client` est
//!     **intra-schéma**, donc légale — et c'est elle qui rend la recherche faisable en une requête.
//!
//! # ⚠️ Ce fichier écrit `numero_piece`, et il l'écrit CHIFFRÉ
//!
//! Le repository ne chiffre ni ne déchiffre : il reçoit et rend la forme **stockée**. Le service
//! seul détient le coffre, parce que c'est lui qui sait quel tenant est en cours et qui doit
//! journaliser la lecture. Un repository qui chiffrerait rendrait la journalisation contournable
//! — il suffirait d'appeler la couche du dessous.
//!
//! Les deux fonctions de lecture sont donc nommées pour qu'on ne s'y trompe pas :
//! [`lire_avec_piece_chiffree`] rend la valeur telle qu'elle est en base, et son nom dit qu'elle
//! n'est pas lisible. Il n'existe **aucune** fonction rendant un numéro en clair dans ce fichier.

use time::{Date, OffsetDateTime};
use uuid::Uuid;

use super::modele::{
    ClientResume, CreerClient, ErreurClient, FormeRecherche, ModifierClient, Preference,
    SUFFIXE_TELEPHONE_MIN,
};
use kaya_etablissements::Issue;

/// Une fiche telle qu'elle sort de la base — **le numéro de pièce y est chiffré**.
///
/// Type interne au repository : il ne quitte jamais ce module autrement que par le service, qui
/// le convertit en [`super::modele::FicheClient`] après déchiffrement **et journalisation**.
pub struct FicheStockee {
    pub id: Uuid,
    pub nom: String,
    pub prenoms: Option<String>,
    pub telephone: Option<String>,
    pub email: Option<String>,
    pub date_naissance: Option<Date>,
    pub nationalite: Option<String>,
    pub type_piece: Option<String>,
    /// **Cryptogramme**, jamais un numéro. Le nom du champ le dit pour que personne ne le
    /// sérialise par mégarde.
    pub numero_piece_chiffre: Option<String>,
    pub piece_capturee_le: Option<OffsetDateTime>,
    pub horodatage_client: Option<OffsetDateTime>,
    pub cree_le: OffsetDateTime,
    pub modifie_le: OffsetDateTime,
}

/// Les valeurs repliées, calculées par le service et posées ici.
///
/// Elles voyagent groupées plutôt qu'en trois paramètres : trois `Option<String>` consécutifs
/// dans une signature s'échangent silencieusement, et l'erreur ne se verrait qu'à la recherche.
pub struct Replis {
    pub nom: Option<String>,
    pub telephone: Option<String>,
    pub numero_piece: Option<String>,
}

/// Insère une personne **et** sa qualification de cliente, dans la **même** transaction.
///
/// L'appelant fournit la transaction : les deux `INSERT` sont indissociables. Une personne créée
/// sans sa ligne `client` serait invisible à la recherche — donc une fiche perdue, que Yao
/// recréerait.
///
/// `ON CONFLICT (id) DO NOTHING ... RETURNING` renvoie une ligne quand l'insertion a eu lieu, et
/// **rien** en cas de conflit : c'est exactement ce qu'il faut pour distinguer `201` de `200`,
/// sans second aller-retour dans le cas normal (patron de `personne/repository.rs`).
pub async fn inserer(
    tx: &mut sqlx::PgTransaction<'_>,
    tenant_id: Uuid,
    demande: &CreerClient,
    replis: &Replis,
    numero_piece_chiffre: Option<&str>,
) -> Result<Issue, ErreurClient> {
    // `piece_capturee_le` n'est posé **que** si une pièce arrive (FR-013) : le poser à `now()`
    // sans pièce ferait démarrer la rétention de TRX-06 sur une donnée qui n'existe pas.
    let capturee_le = numero_piece_chiffre.map(|_| OffsetDateTime::now_utc());

    let insere = sqlx::query_scalar!(
        r#"
        INSERT INTO comptes.personne
            (id, tenant_id, nom, prenoms, telephone, email,
             type_piece, numero_piece, piece_capturee_le,
             nom_repli, telephone_repli, numero_piece_repli, horodatage_client)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
        ON CONFLICT (id) DO NOTHING
        RETURNING id
        "#,
        demande.id,
        tenant_id,
        demande.nom,
        demande.prenoms,
        demande.telephone,
        demande.email,
        demande.type_piece,
        numero_piece_chiffre,
        capturee_le,
        replis.nom,
        replis.telephone,
        replis.numero_piece,
        demande.horodatage_client,
    )
    .fetch_optional(&mut **tx)
    .await?;

    if insere.is_none() {
        // Rejeu : la personne existe. La qualification peut manquer si un cycle antérieur a créé
        // la personne sans elle — une femme de ménage devenue cliente, par exemple. On la pose
        // alors, ce qui est idempotent par la clé primaire.
        let qualifiee = sqlx::query_scalar!(
            r#"
            INSERT INTO comptes.client (personne_id, tenant_id, date_naissance, nationalite,
                                        horodatage_client)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (personne_id) DO NOTHING
            RETURNING personne_id
            "#,
            demande.id,
            tenant_id,
            demande.date_naissance,
            demande.nationalite,
            demande.horodatage_client,
        )
        .fetch_optional(&mut **tx)
        .await?;

        // Si NI la personne NI la qualification n'ont été écrites, c'est un rejeu complet.
        return Ok(if qualifiee.is_some() {
            Issue::Creee
        } else {
            Issue::DejaPresente
        });
    }

    sqlx::query!(
        r#"
        INSERT INTO comptes.client (personne_id, tenant_id, date_naissance, nationalite,
                                    horodatage_client)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (personne_id) DO NOTHING
        "#,
        demande.id,
        tenant_id,
        demande.date_naissance,
        demande.nationalite,
        demande.horodatage_client,
    )
    .execute(&mut **tx)
    .await?;

    Ok(Issue::Creee)
}

/// Remplace les champs modifiables d'une fiche.
///
/// `modifie_le` est posé par `now()` **en SQL**, jamais par l'horloge du processus : deux
/// instances d'API n'ont pas la même horloge, la base en a une seule (principe IV).
///
/// # `piece_capturee_le` ne se remet à zéro que si la pièce CHANGE
///
/// `COALESCE` sur l'ancienne valeur quand aucune pièce nouvelle n'arrive : sans lui, corriger
/// l'orthographe d'un nom repousserait la purge de 90 jours de TRX-06, et une fiche modifiée
/// chaque mois ne serait **jamais** purgée. Le défaut serait invisible — rien à l'écran ne le
/// dirait.
pub async fn modifier(
    tx: &mut sqlx::PgTransaction<'_>,
    id: Uuid,
    modification: &ModifierClient,
    replis: &Replis,
    numero_piece_chiffre: Option<&str>,
) -> Result<bool, ErreurClient> {
    let capturee_le = numero_piece_chiffre.map(|_| OffsetDateTime::now_utc());

    let touchee = sqlx::query_scalar!(
        r#"
        UPDATE comptes.personne
        SET nom                = $2,
            prenoms            = $3,
            telephone          = $4,
            email              = $5,
            type_piece         = $6,
            numero_piece       = $7,
            piece_capturee_le  = CASE WHEN $7::TEXT IS NULL THEN piece_capturee_le ELSE $8 END,
            nom_repli          = $9,
            telephone_repli    = $10,
            numero_piece_repli = $11,
            horodatage_client  = $12,
            modifie_le         = now()
        WHERE id = $1
        RETURNING id
        "#,
        id,
        modification.nom,
        modification.prenoms,
        modification.telephone,
        modification.email,
        modification.type_piece,
        numero_piece_chiffre,
        capturee_le,
        replis.nom,
        replis.telephone,
        replis.numero_piece,
        modification.horodatage_client,
    )
    .fetch_optional(&mut **tx)
    .await?;

    if touchee.is_none() {
        return Ok(false);
    }

    sqlx::query!(
        r#"
        UPDATE comptes.client
        SET date_naissance = $2,
            nationalite    = $3,
            modifie_le     = now()
        WHERE personne_id = $1
        "#,
        id,
        modification.date_naissance,
        modification.nationalite,
    )
    .execute(&mut **tx)
    .await?;

    Ok(true)
}

/// Lit une fiche **avec son numéro de pièce chiffré**.
///
/// ⚠️ Le nom de cette fonction est son mode d'emploi : elle rend un cryptogramme. Le déchiffrer
/// **et journaliser la lecture** est le travail du service. Aucune fonction de ce fichier ne rend
/// un numéro en clair.
///
/// La jointure `personne × client` est **intra-schéma**, donc légale, et le `INNER JOIN` porte la
/// qualification : une personne qui n'est pas cliente n'est **pas** une fiche client, et ne se lit
/// pas par ce chemin.
pub async fn lire_avec_piece_chiffree(
    tx: &mut sqlx::PgTransaction<'_>,
    id: Uuid,
) -> Result<Option<FicheStockee>, ErreurClient> {
    let fiche = sqlx::query_as!(
        FicheStockee,
        r#"
        SELECT p.id                AS "id!",
               p.nom               AS "nom!",
               p.prenoms,
               p.telephone,
               p.email,
               c.date_naissance,
               c.nationalite,
               p.type_piece,
               p.numero_piece      AS "numero_piece_chiffre",
               p.piece_capturee_le,
               p.horodatage_client,
               p.cree_le           AS "cree_le!",
               p.modifie_le        AS "modifie_le!"
        FROM comptes.personne p
        INNER JOIN comptes.client c ON c.personne_id = p.id
        WHERE p.id = $1
        "#,
        id
    )
    .fetch_optional(&mut **tx)
    .await?;

    Ok(fiche)
}

// =================================================================================================
//  La recherche — UNE requête, trois formes
// =================================================================================================

/// Cherche des fiches clientes.
///
/// # Une seule requête, et c'est la condition de la cible des 300 ms
///
/// Les trois formes cohabitent dans le même `WHERE`, chacune neutralisée par un paramètre `NULL`
/// quand elle ne s'applique pas. L'alternative — trois requêtes puis une fusion en Rust — coûterait
/// trois allers-retours **et** un tri applicatif sur des listes qui se recouvrent.
///
/// Le `INNER JOIN comptes.client` est ce qui rend la recherche honnête : **le personnel n'y
/// apparaît jamais** (FR-004). C'est aussi une jointure **intra-schéma**, donc légale.
///
/// # `limite + 1`, et pourquoi
///
/// On demande **un** résultat de plus que la limite. S'il arrive, il y avait davantage à voir :
/// `tronque` devient vrai et la ligne excédentaire est écartée. Un `COUNT(*)` séparé donnerait la
/// même information au prix d'un second parcours de l'index — pour un booléen.
pub async fn rechercher(
    tx: &mut sqlx::PgTransaction<'_>,
    forme: FormeRecherche,
    nom_replie: &str,
    telephone_replie: &str,
    piece_repliee: &str,
    limite: i64,
) -> Result<(Vec<ClientResume>, bool), ErreurClient> {
    // Chaque motif vaut `NULL` quand sa forme ne s'applique pas : le prédicat correspondant
    // devient faux, et l'optimiseur n'ouvre pas l'index.
    let motif_nom = match forme {
        FormeRecherche::Nom | FormeRecherche::Ambigue if !nom_replie.is_empty() => {
            Some(format!("{nom_replie}%"))
        }
        _ => None,
    };
    let motif_telephone = match forme {
        FormeRecherche::Telephone | FormeRecherche::Ambigue
            if telephone_replie.len() >= SUFFIXE_TELEPHONE_MIN =>
        {
            Some(format!("%{telephone_replie}"))
        }
        _ => None,
    };
    let motif_piece = match forme {
        FormeRecherche::Piece | FormeRecherche::Ambigue if !piece_repliee.is_empty() => {
            Some(piece_repliee.to_owned())
        }
        _ => None,
    };

    let lignes = sqlx::query_as!(
        ClientResume,
        r#"
        SELECT p.id                             AS "id!",
               p.nom                            AS "nom!",
               p.prenoms,
               p.telephone,
               (p.numero_piece IS NOT NULL)     AS "piece_enregistree!"
        FROM comptes.personne p
        INNER JOIN comptes.client c ON c.personne_id = p.id
        WHERE ($1::TEXT IS NOT NULL AND p.nom_repli          LIKE $1)
           OR ($2::TEXT IS NOT NULL AND p.telephone_repli    LIKE $2)
           OR ($3::TEXT IS NOT NULL AND p.numero_piece_repli =    $3)
        ORDER BY p.nom_repli, p.id
        LIMIT $4
        "#,
        motif_nom,
        motif_telephone,
        motif_piece,
        limite + 1,
    )
    .fetch_all(&mut **tx)
    .await?;

    let tronque = lignes.len() as i64 > limite;
    let mut clients = lignes;
    clients.truncate(limite as usize);

    Ok((clients, tronque))
}

// =================================================================================================
//  Préférences — append-only
// =================================================================================================

/// Enregistre une préférence. **`INSERT` seul** — le privilège rend l'`UPDATE` impossible.
///
/// Rend [`Issue::DejaPresente`] sur rejeu, ce qui commande au service de **n'émettre aucun second
/// événement**. C'est le contrôle qui existait pour `note_etablissement` et qui a été perdu à la
/// réécriture sur `occupation` : un rejeu qui émettrait ferait du grand livre le journal des
/// tentatives réseau du terminal.
pub async fn enregistrer_preference(
    tx: &mut sqlx::PgTransaction<'_>,
    tenant_id: Uuid,
    id: Uuid,
    personne_id: Uuid,
    texte: &str,
    horodatage_client: Option<OffsetDateTime>,
) -> Result<(Preference, Issue), ErreurClient> {
    let insere = sqlx::query_as!(
        Preference,
        r#"
        INSERT INTO comptes.preference_personne (id, tenant_id, personne_id, texte,
                                                 horodatage_client)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (id) DO NOTHING
        RETURNING id, personne_id, texte, horodatage_client, cree_le
        "#,
        id,
        tenant_id,
        personne_id,
        texte,
        horodatage_client,
    )
    .fetch_optional(&mut **tx)
    .await?;

    match insere {
        Some(preference) => Ok((preference, Issue::Creee)),
        None => {
            let existante = sqlx::query_as!(
                Preference,
                r#"
                SELECT id, personne_id, texte, horodatage_client, cree_le
                FROM comptes.preference_personne
                WHERE id = $1
                "#,
                id
            )
            .fetch_optional(&mut **tx)
            .await?
            .ok_or(ErreurClient::Inconnu)?;

            Ok((existante, Issue::DejaPresente))
        }
    }
}

/// Les préférences d'une personne, **de la plus récente à la plus ancienne**.
///
/// L'ordre est donné par `cree_le`, l'horodatage d'**autorité** — jamais par `horodatage_client`,
/// qui est indicatif (porte P-23). Deux terminaux mal réglés vidant leur file dans le désordre
/// doivent produire le même ordre d'affichage.
pub async fn preferences(
    tx: &mut sqlx::PgTransaction<'_>,
    personne_id: Uuid,
    limite: i64,
) -> Result<Vec<Preference>, ErreurClient> {
    let lignes = sqlx::query_as!(
        Preference,
        r#"
        SELECT id, personne_id, texte, horodatage_client, cree_le
        FROM comptes.preference_personne
        WHERE personne_id = $1
        ORDER BY cree_le DESC, id DESC
        LIMIT $2
        "#,
        personne_id,
        limite,
    )
    .fetch_all(&mut **tx)
    .await?;

    Ok(lignes)
}

/// Les résumés de plusieurs clients, **en une requête** — support du trait `AnnuaireClients`.
///
/// ⚠️ **Par lot, jamais un par un.** Une signature unitaire produirait N+1 requêtes sur la liste
/// des séjours en cours, et c'est le détail qui décide si l'écran de départ s'ouvre en 200 ms ou
/// en deux secondes.
///
/// Les identifiants inconnus sont **absents** de la réponse, jamais rendus en `None` : un séjour
/// dont le client a été purgé (TRX-06) reste lisible, sans nom.
pub async fn resumes(
    tx: &mut sqlx::PgTransaction<'_>,
    ids: &[Uuid],
) -> Result<Vec<ClientResume>, ErreurClient> {
    let lignes = sqlx::query_as!(
        ClientResume,
        r#"
        SELECT p.id                         AS "id!",
               p.nom                        AS "nom!",
               p.prenoms,
               p.telephone,
               (p.numero_piece IS NOT NULL) AS "piece_enregistree!"
        FROM comptes.personne p
        INNER JOIN comptes.client c ON c.personne_id = p.id
        WHERE p.id = ANY($1)
        "#,
        ids
    )
    .fetch_all(&mut **tx)
    .await?;

    Ok(lignes)
}

/// Vrai si l'identifiant désigne un client du tenant courant.
///
/// Appelé par l'ouverture d'un séjour pour refuser un `client_id` inventé. La politique de
/// sécurité empêcherait déjà la lecture d'un client d'un autre tenant, mais un refus explicite
/// vaut mieux qu'une ligne orpheline qu'aucune contrainte ne peut interdire — la clé étrangère
/// étant impossible entre deux schémas (principe II).
pub async fn existe(tx: &mut sqlx::PgTransaction<'_>, id: Uuid) -> Result<bool, ErreurClient> {
    let trouve = sqlx::query_scalar!(
        r#"SELECT personne_id FROM comptes.client WHERE personne_id = $1"#,
        id
    )
    .fetch_optional(&mut **tx)
    .await?;

    Ok(trouve.is_some())
}
