//! Limitation de débit des tentatives de connexion — **deux clés, et il en faut deux**.
//!
//! # Compter par identifiant seul, ou par origine seule, ne protège de rien
//!
//! | Ce qu'on compte | Ce que ça arrête | Ce que ça laisse passer |
//! |---|---|---|
//! | L'**identifiant** seul | L'acharnement sur un compte | Le **balayage** : mille comptes à une tentative chacun, aucun compteur ne bouge |
//! | L'**origine** seule | Un attaquant à une adresse | L'attaque **distribuée** sur un compte unique : mille adresses, une tentative chacune |
//!
//! Les deux fautes sont symétriques, et chacune est invisible depuis l'autre compteur. D'où deux
//! fenêtres, comptées séparément, et un refus dès que **l'une** des deux déborde.
//!
//! # Le refus est indiscernable, sans quoi il rouvre ce que FR-012 ferme
//!
//! Un message « trop de tentatives » sur un identifiant existant, et rien sur un identifiant
//! inconnu, rétablirait exactement la fuite que le condensat factice referme — en plus lisible.
//! Le dépassement rend donc le **même** refus que les trois autres causes :
//! `identifiants_invalides`.
//!
//! # Aucun verrouillage définitif, jamais
//!
//! Un compte bloqué après N échecs est un **déni de service offert à qui connaît le téléphone
//! d'Adjoua** : dix tentatives ratées, et la caissière ne peut plus ouvrir sa caisse de la
//! journée. La fenêtre glisse, elle ne se referme pas — au pire, l'attaquant impose une attente
//! de quelques minutes, et le personnel légitime la subit aussi, ce qui est le compromis le moins
//! mauvais.
//!
//! # Une fenêtre glissante, pas un seau à jetons
//!
//! Un compteur à fenêtre **fixe** (`INCR` + `EXPIRE`) laisse passer deux fois le quota au moment
//! du basculement : N tentatives à la fin d'une fenêtre, N au début de la suivante, dans la même
//! seconde. La fenêtre glissante compte les tentatives des D dernières secondes à chaque appel,
//! ce qui n'a pas de bord.
//!
//! Elle est portée par un **ensemble ordonné** Redis : chaque tentative y entre avec son
//! horodatage pour score, et les scores trop vieux sont retirés avant chaque décompte. Le coût est
//! d'une clé par identifiant et par origine, chacune expirant d'elle-même.

use time::OffsetDateTime;
use uuid::Uuid;

/// Durée de la fenêtre, en secondes.
///
/// **Cinq minutes.** Assez long pour qu'un balayage automatisé bute dessus, assez court pour
/// qu'une caissière qui s'est trompée trois fois de suite ne reste pas dehors le temps d'un
/// service.
pub const FENETRE_S: i64 = 300;

/// Tentatives admises par identifiant dans la fenêtre.
///
/// **Dix.** Un humain qui hésite sur son mot de passe en fait trois ou quatre ; au-delà, il
/// appelle le gérant. Un automate en fait des milliers.
pub const MAX_PAR_IDENTIFIANT: usize = 10;

/// Tentatives admises par origine dans la fenêtre.
///
/// **Soixante.** Plus large que le seuil par identifiant, et pour une raison concrète : dans un
/// hôtel, tout le poste de réception sort par la **même adresse**. Un seuil serré y bloquerait
/// l'équipe entière au moment de la relève, quand tout le monde se connecte en même temps.
pub const MAX_PAR_ORIGINE: usize = 60;

const PREFIXE_IDENTIFIANT: &str = "tentatives:id";
const PREFIXE_ORIGINE: &str = "tentatives:ip";

/// Quelle limite a débordé — **pour les journaux, jamais pour la réponse**.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Depassement {
    Identifiant,
    Origine,
}

/// Compteur de tentatives.
#[derive(Clone)]
pub struct LimiteTentatives {
    client: redis::Client,
}

impl LimiteTentatives {
    pub fn nouveau(url: &str) -> Result<Self, redis::RedisError> {
        Ok(Self {
            client: redis::Client::open(url)?,
        })
    }

    /// Enregistre une tentative et dit si l'une des deux fenêtres déborde.
    ///
    /// **La tentative est comptée avant le verdict**, et c'est délibéré : compter seulement les
    /// échecs laisserait un attaquant faire autant de tentatives réussies qu'il veut avec un
    /// identifiant volé. Ce qu'on limite est le **débit d'essais**, pas le nombre d'erreurs.
    ///
    /// `origine` est l'adresse de l'appelant telle que la couche HTTP la connaît. Elle est
    /// **hachée en identifiant opaque** avant d'entrer en clé : une adresse IP est une donnée
    /// personnelle, et Redis n'a aucune raison d'en tenir la liste en clair.
    pub async fn enregistrer(
        &self,
        identifiant: &str,
        origine: &str,
    ) -> Result<Option<Depassement>, redis::RedisError> {
        let maintenant = OffsetDateTime::now_utc().unix_timestamp();

        let par_identifiant = self
            .compter(&cle(PREFIXE_IDENTIFIANT, identifiant), maintenant)
            .await?;
        let par_origine = self
            .compter(&cle(PREFIXE_ORIGINE, origine), maintenant)
            .await?;

        // L'ordre du `if` n'a aucune portée fonctionnelle — le refus est le même — mais il décide
        // de ce qui part dans les journaux. L'identifiant d'abord : c'est le signal le plus utile
        // au support, qui reçoit l'appel de l'utilisateur bloqué.
        if par_identifiant > MAX_PAR_IDENTIFIANT {
            return Ok(Some(Depassement::Identifiant));
        }
        if par_origine > MAX_PAR_ORIGINE {
            return Ok(Some(Depassement::Origine));
        }
        Ok(None)
    }

    /// Ajoute la tentative à la fenêtre et rend le nombre de tentatives encore dedans.
    ///
    /// Les trois opérations sont envoyées **en pipeline** : trois allers-retours réseau par
    /// compteur, donc six par tentative de connexion, seraient perceptibles sur le réseau
    /// d'Abengourou.
    async fn compter(&self, cle: &str, maintenant: i64) -> Result<usize, redis::RedisError> {
        let mut cx = self.client.get_multiplexed_async_connection().await?;

        // Le membre de l'ensemble doit être **unique** : deux tentatives dans la même seconde
        // partageraient un score, et un membre identique n'en ferait qu'une. Un UUID v7 par
        // tentative règle la question sans horloge de plus haute résolution.
        let membre = Uuid::now_v7().to_string();

        let (_, _, nombre, _): ((), (), usize, ()) = redis::pipe()
            .atomic()
            .zrembyscore(cle, i64::MIN, maintenant - FENETRE_S)
            .zadd(cle, &membre, maintenant)
            .zcard(cle)
            // La clé expire d'elle-même : sans cela, une adresse vue une fois resterait en
            // mémoire indéfiniment, et l'espace de clés croîtrait avec le nombre d'adresses de
            // l'internet.
            .expire(cle, FENETRE_S)
            .query_async(&mut cx)
            .await?;

        Ok(nombre)
    }
}

/// Construit une clé — **la valeur sensible est réduite à une empreinte**.
///
/// Un identifiant de connexion est un numéro de téléphone, une origine est une adresse IP : deux
/// données personnelles. Les écrire en clair en clé mettrait dans Redis la liste horodatée de qui
/// tente de se connecter et d'où — exactement le fichier que le grand livre refuse de constituer
/// (research R-15). L'empreinte suffit : on ne cherche jamais une clé, on ne fait que la relire.
fn cle(prefixe: &str, valeur: &str) -> String {
    format!("{prefixe}:{:016x}", empreinte(valeur))
}

/// Empreinte non cryptographique, stable dans le processus **et entre processus**.
///
/// `DefaultHasher` de la bibliothèque standard ne convient pas : sa graine peut changer d'une
/// version à l'autre, et deux instances d'API derrière un répartiteur produiraient alors deux
/// clés pour le même identifiant — donc deux fois le quota. FNV-1a est spécifié, tient en six
/// lignes, et n'ajoute aucune dépendance.
///
/// **Ce n'est pas un condensat de sécurité et il ne prétend pas l'être** : il ne protège pas d'une
/// attaque par dictionnaire sur un numéro de téléphone ivoirien, dont l'espace est petit. Il
/// évite que la liste soit *lisible*, ce qui est le but ; protéger ce que Redis contient relève de
/// l'accès à Redis, pas d'un condensat.
fn empreinte(valeur: &str) -> u64 {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const NOMBRE_PREMIER: u64 = 0x0000_0100_0000_01b3;

    valeur.bytes().fold(OFFSET, |empreinte, octet| {
        (empreinte ^ u64::from(octet)).wrapping_mul(NOMBRE_PREMIER)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// L'empreinte est **stable** : la même chaîne donne toujours la même clé.
    ///
    /// Si elle ne l'était pas, chaque redémarrage remettrait tous les compteurs à zéro et la
    /// limitation ne limiterait rien — sans qu'aucun test fonctionnel ne s'en aperçoive.
    #[test]
    fn l_empreinte_est_stable_entre_deux_appels() {
        assert_eq!(empreinte("+2250700000001"), empreinte("+2250700000001"));
        assert_ne!(empreinte("+2250700000001"), empreinte("+2250700000002"));
    }

    /// La valeur sensible **n'apparaît pas** dans la clé.
    #[test]
    fn la_cle_ne_porte_pas_la_valeur_en_clair() {
        let cle = cle(PREFIXE_IDENTIFIANT, "+2250700000001");
        assert!(
            !cle.contains("0700000001"),
            "le numéro apparaît en clair dans « {cle} » — Redis porterait la liste horodatée de \
             qui tente de se connecter"
        );
        assert!(cle.starts_with("tentatives:id:"));
    }

    /// Les deux préfixes **ne se recouvrent pas**.
    ///
    /// Un préfixe partagé ferait qu'un identifiant et une origine de même empreinte
    /// s'additionneraient dans le même compteur, et le seuil le plus bas s'appliquerait aux deux.
    #[test]
    fn les_deux_compteurs_ne_partagent_aucune_cle() {
        assert_ne!(
            cle(PREFIXE_IDENTIFIANT, "a"),
            cle(PREFIXE_ORIGINE, "a"),
            "un identifiant et une origine de même valeur partagent leur compteur"
        );
    }

    /// **Le seuil par origine est plus large que celui par identifiant.**
    ///
    /// L'inverse bloquerait le poste de réception d'un hôtel à la relève, quand toute l'équipe se
    /// connecte depuis la même adresse en quelques minutes. Le test fige le rapport pour qu'un
    /// réglage « pour durcir » ne l'inverse pas sans s'en apercevoir.
    #[test]
    fn le_seuil_par_origine_est_plus_large_que_celui_par_identifiant() {
        assert!(MAX_PAR_ORIGINE > MAX_PAR_IDENTIFIANT);
    }
}
