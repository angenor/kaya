//! Les sessions en Redis — **trois familles de clés, et pas une quatrième**.
//!
//! # Pourquoi Redis et pas une table
//!
//! Une session est éphémère et **reconstructible** : Redis vidé, tout le monde se reconnecte et
//! aucune donnée métier ne manque. La mettre en table lui donnerait une sauvegarde, une place au
//! registre des classes hors-ligne, et une migration à écrire le jour où sa forme change — trois
//! obligations pour une donnée dont la perte est sans conséquence.
//!
//! Le corollaire est écrit dans `api/src/secrets.rs` : **Redis n'est plus optionnel**. Son statut
//! de stockage éphémère ne change pas, son exigence de disponibilité si.
//!
//! # Les trois familles de clés
//!
//! | Clé | Type | Durée | Ce qu'elle porte |
//! |---|---|---|---|
//! | `session:{compte_id}` | HASH | 90 j | Une entrée par session du compte, champ = `session_id` |
//! | `revoquees:{session_id}` | STRING | 60 min | La marque de révocation, consultée **à chaque requête** |
//! | `famille:{famille_id}` | STRING | 90 j | Le **seul** `jti` de rafraîchissement encore valide |
//!
//! ## Le hachage par compte, et non une clé par session
//!
//! C'est le seul choix de forme qui mérite d'être justifié, parce que la forme évidente serait
//! `session:{session_id}`. Elle rendrait `session_lister_actives` impossible sans un `SCAN` de
//! l'espace de clés entier — donc un coût proportionnel au **nombre total de sessions de tous les
//! clients**, sur un Redis partagé. Un hachage par compte donne la lecture d'une session en `HGET`
//! et la liste en `HGETALL`, toutes deux proportionnelles au seul compte concerné.
//!
//! Ce n'est possible que parce que **le jeton porte le compte** (`sub`) en plus de la session
//! (`sid`) : la clé se reconstruit sans lecture préalable.
//!
//! Le prix est explicite : Redis n'expire pas un champ de hachage individuellement. Une session
//! périmée reste donc **présente et inerte** jusqu'à l'expiration du hachage entier. C'est pour
//! cela que chaque session porte `expire_le` et que la lecture filtre — la donnée fait foi, pas la
//! présence de la clé.
//!
//! ## La marque de révocation dure 60 minutes, et c'est exactement ce qu'il faut
//!
//! Au-delà, le jeton d'accès qu'elle invalide a **expiré de lui-même** : le garder plus longtemps
//! ferait grossir l'espace de clés sans rien protéger de plus. La durée est donc **liée à celle du
//! jeton d'accès**, pas choisie séparément — [`marquer_revoquee`] la prend en paramètre pour que
//! le lien reste visible chez l'appelant.

use redis::AsyncCommands;
use uuid::Uuid;

use super::modele::Session;

/// Préfixes — écrits une fois, employés partout.
const PREFIXE_SESSIONS: &str = "session";
const PREFIXE_REVOQUEES: &str = "revoquees";
const PREFIXE_FAMILLE: &str = "famille";

fn cle_sessions(compte_id: Uuid) -> String {
    format!("{PREFIXE_SESSIONS}:{compte_id}")
}

fn cle_revoquee(session_id: Uuid) -> String {
    format!("{PREFIXE_REVOQUEES}:{session_id}")
}

fn cle_famille(famille_id: Uuid) -> String {
    format!("{PREFIXE_FAMILLE}:{famille_id}")
}

/// Accès à l'entrepôt des sessions.
///
/// Le client `redis` est **clonable et gère son propre pool de connexions multiplexées** : le
/// conserver ici plutôt que d'en ouvrir un par appel évite un aller-retour de connexion sur le
/// chemin le plus chaud du produit.
#[derive(Clone)]
pub struct Entrepot {
    client: redis::Client,
}

impl Entrepot {
    /// Construit l'entrepôt depuis une URL Redis.
    pub fn nouveau(url: &str) -> Result<Self, redis::RedisError> {
        Ok(Self {
            client: redis::Client::open(url)?,
        })
    }

    async fn connexion(&self) -> Result<redis::aio::MultiplexedConnection, redis::RedisError> {
        self.client.get_multiplexed_async_connection().await
    }

    // ── Les sessions ───────────────────────────────────────────────────────────────────────

    /// Écrit ou met à jour une session, et **replace la durée de vie du hachage**.
    ///
    /// Le `EXPIRE` est reposé à chaque écriture : sans lui, le hachage d'un compte actif
    /// disparaîtrait 90 jours après sa **première** session, coupant toutes les autres d'un coup.
    pub async fn enregistrer(
        &self,
        session: &Session,
        duree_s: i64,
    ) -> Result<(), redis::RedisError> {
        let mut cx = self.connexion().await?;
        let charge = serde_json::to_string(session).map_err(erreur_serialisation)?;

        let _: () = cx
            .hset(cle_sessions(session.compte_id), session.id.to_string(), charge)
            .await?;
        let _: () = cx.expire(cle_sessions(session.compte_id), duree_s).await?;
        Ok(())
    }

    /// Lit une session d'un compte, **si elle n'est pas périmée**.
    ///
    /// Un champ périmé est traité comme absent, et **retiré au passage** : Redis n'expirant pas
    /// les champs de hachage, c'est la lecture qui fait le ménage. Le faire ici plutôt que par une
    /// tâche de fond évite d'ajouter un processus pour balayer ce que la lecture croise déjà.
    pub async fn lire(
        &self,
        compte_id: Uuid,
        session_id: Uuid,
    ) -> Result<Option<Session>, redis::RedisError> {
        let mut cx = self.connexion().await?;
        let charge: Option<String> = cx
            .hget(cle_sessions(compte_id), session_id.to_string())
            .await?;

        let Some(session) = charge.and_then(|c| serde_json::from_str::<Session>(&c).ok()) else {
            return Ok(None);
        };

        if session.expire_le <= time::OffsetDateTime::now_utc() {
            let _: () = cx
                .hdel(cle_sessions(compte_id), session_id.to_string())
                .await?;
            return Ok(None);
        }

        Ok(Some(session))
    }

    /// Toutes les sessions vivantes d'un compte, de la plus récemment active à la plus ancienne.
    ///
    /// Le tri est fait ici et non par Redis : un hachage n'a pas d'ordre, et trier côté serveur sur
    /// quelques dizaines d'entrées coûte moins qu'un index à tenir.
    pub async fn lister(&self, compte_id: Uuid) -> Result<Vec<Session>, redis::RedisError> {
        let mut cx = self.connexion().await?;
        let charges: std::collections::HashMap<String, String> =
            cx.hgetall(cle_sessions(compte_id)).await?;

        let maintenant = time::OffsetDateTime::now_utc();
        let mut sessions: Vec<Session> = charges
            .values()
            .filter_map(|c| serde_json::from_str::<Session>(c).ok())
            .filter(|s| s.expire_le > maintenant)
            .collect();

        sessions.sort_by(|a, b| b.derniere_activite_le.cmp(&a.derniere_activite_le));
        Ok(sessions)
    }

    /// Retire une session du hachage — la fermeture volontaire.
    pub async fn oublier(
        &self,
        compte_id: Uuid,
        session_id: Uuid,
    ) -> Result<(), redis::RedisError> {
        let mut cx = self.connexion().await?;
        let _: () = cx
            .hdel(cle_sessions(compte_id), session_id.to_string())
            .await?;
        Ok(())
    }

    // ── La liste de révocation ─────────────────────────────────────────────────────────────

    /// Marque une session comme révoquée, **pour la durée de vie d'un jeton d'accès**.
    ///
    /// C'est ce qui rend la coupure immédiate : le jeton en circulation reste
    /// mathématiquement valide, mais la requête suivante consulte cette marque et le refuse. Sans
    /// elle, il faudrait attendre l'expiration — jusqu'à 90 jours pour le rafraîchissement, ce qui
    /// laisserait un téléphone volé ouvert un trimestre.
    pub async fn marquer_revoquee(
        &self,
        session_id: Uuid,
        duree_acces_s: i64,
    ) -> Result<(), redis::RedisError> {
        let mut cx = self.connexion().await?;
        // La valeur ne porte rien : c'est la **présence** de la clé qui est l'information.
        let _: () = cx
            .set_ex(cle_revoquee(session_id), "1", duree_acces_s.max(1) as u64)
            .await?;
        Ok(())
    }

    /// **Consultée à chaque requête authentifiée.** Un seul aller-retour, une seule clé.
    pub async fn est_revoquee(&self, session_id: Uuid) -> Result<bool, redis::RedisError> {
        let mut cx = self.connexion().await?;
        let presente: bool = cx.exists(cle_revoquee(session_id)).await?;
        Ok(presente)
    }

    // ── La famille de jetons de rafraîchissement ───────────────────────────────────────────

    /// Pose le **seul** exemplaire de rafraîchissement valide de la famille.
    ///
    /// Écrase le précédent : c'est la rotation. L'exemplaire remplacé devient de ce fait
    /// détectable — le présenter signale qu'une copie circule.
    pub async fn poser_exemplaire(
        &self,
        famille_id: Uuid,
        jti: Uuid,
        duree_s: i64,
    ) -> Result<(), redis::RedisError> {
        let mut cx = self.connexion().await?;
        let _: () = cx
            .set_ex(cle_famille(famille_id), jti.to_string(), duree_s.max(1) as u64)
            .await?;
        Ok(())
    }

    /// L'exemplaire encore valide de la famille, s'il y en a un.
    pub async fn exemplaire_valide(
        &self,
        famille_id: Uuid,
    ) -> Result<Option<Uuid>, redis::RedisError> {
        let mut cx = self.connexion().await?;
        let valeur: Option<String> = cx.get(cle_famille(famille_id)).await?;
        Ok(valeur.and_then(|v| Uuid::parse_str(&v).ok()))
    }

    /// **Révoque la famille entière** — la réponse à un jeton réutilisé.
    ///
    /// Effacer la clé suffit : plus aucun exemplaire ne peut correspondre, y compris celui que
    /// détient le titulaire légitime. C'est voulu. Révoquer le seul exemplaire présenté laisserait
    /// le voleur et la victime en course, et **le premier des deux gagnerait** — sans qu'aucun des
    /// deux ne sache qu'il y a eu course.
    pub async fn revoquer_famille(&self, famille_id: Uuid) -> Result<(), redis::RedisError> {
        let mut cx = self.connexion().await?;
        let _: () = cx.del(cle_famille(famille_id)).await?;
        Ok(())
    }
}

/// Une charge JSON illisible est une erreur de l'entrepôt, pas une session absente.
fn erreur_serialisation(erreur: serde_json::Error) -> redis::RedisError {
    redis::RedisError::from((
        redis::ErrorKind::Parse,
        "sérialisation de la session",
        erreur.to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Les trois familles de clés sont **nommées, distinctes et sans collision possible**.
    ///
    /// Un préfixe partagé ferait qu'une révocation écraserait une session, ou qu'un `DEL` de
    /// famille emporterait autre chose. Le test fige les formes plutôt que de les redécouvrir.
    #[test]
    fn les_trois_familles_de_cles_ne_se_recouvrent_pas() {
        let id = Uuid::nil();
        let cles = [cle_sessions(id), cle_revoquee(id), cle_famille(id)];

        assert_eq!(cles[0], "session:00000000-0000-0000-0000-000000000000");
        assert_eq!(cles[1], "revoquees:00000000-0000-0000-0000-000000000000");
        assert_eq!(cles[2], "famille:00000000-0000-0000-0000-000000000000");

        // Aucun préfixe n'est préfixe d'un autre — sans quoi un `SCAN` de maintenance sur
        // `session:*` emporterait une famille.
        for (i, a) in cles.iter().enumerate() {
            for (j, b) in cles.iter().enumerate() {
                if i != j {
                    assert!(!a.starts_with(b.split(':').next().unwrap()));
                }
            }
        }
    }
}
