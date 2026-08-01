//! Les deux jetons — **signés, jamais chiffrés**, et ils ne portent pas la même chose.
//!
//! # Un JWT est lisible par qui le détient
//!
//! Signer n'est pas chiffrer : la charge utile est en base64, donc en clair pour l'utilisateur, et
//! c'est acceptable — elle ne porte que ce que cet utilisateur sait déjà de lui-même. **Rien qui
//! ne lui appartienne n'y entre jamais** : ni condensat, ni identifiant d'un autre compte, ni
//! nom d'un tiers.
//!
//! # Pourquoi les permissions voyagent dans le jeton d'accès
//!
//! Elles y sont pour que **le serveur** n'ait pas à relire la base à chaque requête — le chemin le
//! plus chaud du produit. Le prix est explicite et assumé : un rôle retiré prend effet au
//! **rafraîchissement suivant**, soit au plus la durée du jeton d'accès (60 minutes par défaut,
//! paramétrable). C'est l'hypothèse 5 de la spec, et c'est aussi pourquoi la révocation de session,
//! elle, est **immédiate** : elle passe par Redis, pas par l'expiration du jeton.
//!
//! **Le front ne décode jamais ce jeton** (research R-06). Ses permissions viennent de la réponse
//! de connexion, en clair. Deux sources pour la même information, et une seule fait autorité : le
//! serveur. Un front qui décoderait ferait sa propre lecture des `claims` et divergerait au premier
//! changement de format.
//!
//! # Le jeton de rafraîchissement porte trois identifiants et rien d'autre
//!
//! `sid` (la session), `fid` (la famille) et `jti` (cet exemplaire-ci). C'est `jti` qui rend la
//! **détection de réutilisation** possible : Redis retient le seul `jti` valide de la famille, et
//! tout autre exemplaire présenté signifie qu'une copie circule.

use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// L'algorithme, **fixé et vérifié à la lecture**.
///
/// La validation impose cet algorithme et lui seul. Sans cette contrainte, un jeton présenté avec
/// `alg: none` — ou avec un algorithme asymétrique dont la clé publique serait devinée — pourrait
/// être accepté : c'est la faute historique la plus répandue sur les JWT, et elle se corrige en
/// une ligne à condition de l'écrire.
const ALGORITHME: Algorithm = Algorithm::HS256;

/// Tolérance d'horloge à la vérification de l'expiration, en secondes.
///
/// **Explicite parce que le défaut de la bibliothèque est de 60 secondes**, ce qui est beaucoup
/// pour une expiration de 60 minutes : un jeton révoqué resterait accepté une minute de plus, sur
/// le seul chemin où le produit promet une coupure *immédiate*.
///
/// Cinq secondes couvrent la dérive entre deux instances d'API derrière un répartiteur — la seule
/// raison légitime d'une tolérance. Zéro produirait des `401` fortuits que personne ne saurait
/// reproduire ; l'horodatage d'autorité du principe IV concerne la base, pas les horloges des
/// processus, et ces deux-là ne sont pas la même chose.
const TOLERANCE_HORLOGE_S: u64 = 5;

/// Charge utile du **jeton d'accès**.
///
/// Les noms de champs sont courts parce qu'ils voyagent à chaque requête ; les noms `sub`, `iat`
/// et `exp` sont ceux de la RFC 7519, que toute bibliothèque comprend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimsAcces {
    /// Le compte.
    pub sub: Uuid,
    pub tenant: Uuid,
    /// L'établissement actif. `None` pour un `admin_editeur`, qui n'en a pas.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub etablissement: Option<Uuid>,
    /// La session — c'est **elle** que la liste de révocation désigne.
    pub sid: Uuid,
    /// Les permissions effectives, **union des rôles portés** (FR-017).
    pub perms: Vec<String>,
    pub iat: i64,
    pub exp: i64,
}

/// Charge utile du **jeton de rafraîchissement**.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimsRafraichissement {
    pub sub: Uuid,
    pub tenant: Uuid,
    pub sid: Uuid,
    /// La famille. Un jeton réutilisé la révoque **entière**.
    pub fid: Uuid,
    /// L'exemplaire. Redis n'en retient qu'un par famille à la fois.
    pub jti: Uuid,
    pub iat: i64,
    pub exp: i64,
}

/// Signe un jeton d'accès.
pub fn signer_acces(
    cle: &[u8],
    claims: &ClaimsAcces,
) -> Result<String, jsonwebtoken::errors::Error> {
    encode(
        &Header::new(ALGORITHME),
        claims,
        &EncodingKey::from_secret(cle),
    )
}

/// Signe un jeton de rafraîchissement.
pub fn signer_rafraichissement(
    cle: &[u8],
    claims: &ClaimsRafraichissement,
) -> Result<String, jsonwebtoken::errors::Error> {
    encode(
        &Header::new(ALGORITHME),
        claims,
        &EncodingKey::from_secret(cle),
    )
}

/// Vérifie et décode un jeton d'accès.
///
/// L'expiration est vérifiée par la bibliothèque. **La révocation ne l'est pas ici** : elle vit en
/// Redis et se consulte à chaque requête, parce qu'un jeton signé reste mathématiquement valide
/// jusqu'à son `exp` quoi qu'il arrive. C'est la séparation qui rend la coupure immédiate possible.
pub fn verifier_acces(cle: &[u8], jeton: &str) -> Result<ClaimsAcces, jsonwebtoken::errors::Error> {
    decode::<ClaimsAcces>(jeton, &DecodingKey::from_secret(cle), &validation())
        .map(|donnees| donnees.claims)
}

/// Vérifie et décode un jeton de rafraîchissement.
pub fn verifier_rafraichissement(
    cle: &[u8],
    jeton: &str,
) -> Result<ClaimsRafraichissement, jsonwebtoken::errors::Error> {
    decode::<ClaimsRafraichissement>(jeton, &DecodingKey::from_secret(cle), &validation())
        .map(|donnees| donnees.claims)
}

/// La validation, **avec son algorithme épinglé**.
fn validation() -> Validation {
    let mut validation = Validation::new(ALGORITHME);
    // Épinglage explicite : `Validation::new` le pose déjà, l'écrire ici le rend visible et
    // survivrait à un remaniement qui construirait la validation autrement.
    validation.algorithms = vec![ALGORITHME];
    validation.validate_exp = true;
    validation.leeway = TOLERANCE_HORLOGE_S;
    validation
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::OffsetDateTime;

    fn cle() -> Vec<u8> {
        b"une-cle-de-test-de-trente-deux-octets!".to_vec()
    }

    fn claims_acces(expire_dans_s: i64) -> ClaimsAcces {
        let maintenant = OffsetDateTime::now_utc().unix_timestamp();
        ClaimsAcces {
            sub: Uuid::now_v7(),
            tenant: Uuid::now_v7(),
            etablissement: Some(Uuid::now_v7()),
            sid: Uuid::now_v7(),
            perms: vec!["cpt.compte.lire".to_owned(), "etb.note.lire".to_owned()],
            iat: maintenant,
            exp: maintenant + expire_dans_s,
        }
    }

    #[test]
    fn un_jeton_signe_se_relit_a_l_identique() {
        let cle = cle();
        let claims = claims_acces(3600);
        let jeton = signer_acces(&cle, &claims).expect("signature");
        let relu = verifier_acces(&cle, &jeton).expect("vérification");

        assert_eq!(relu.sub, claims.sub);
        assert_eq!(relu.sid, claims.sid);
        assert_eq!(relu.perms, claims.perms);
    }

    /// **Une autre clé ne vérifie rien.** C'est tout ce qui sépare un jeton d'une chaîne libre.
    #[test]
    fn un_jeton_signe_par_une_autre_cle_est_refuse() {
        let jeton = signer_acces(&cle(), &claims_acces(3600)).expect("signature");
        assert!(verifier_acces(b"une-autre-cle-de-trente-deux-octets!!", &jeton).is_err());
    }

    /// **Un jeton expiré est refusé**, sans que l'appelant ait à comparer des horloges.
    #[test]
    fn un_jeton_expire_est_refuse() {
        // Au-delà de la tolérance d'horloge : un jeton expiré depuis dix secondes serait encore
        // accepté, et c'est voulu.
        let jeton = signer_acces(&cle(), &claims_acces(-3600)).expect("signature");
        assert!(verifier_acces(&cle(), &jeton).is_err());
    }

    /// **`alg: none` est refusé.**
    ///
    /// La faute historique des JWT : accepter l'algorithme annoncé par le jeton lui-même, ce qui
    /// laisse quiconque forger une charge utile en déclarant qu'elle n'est pas signée.
    ///
    /// Le jeton est écrit **en dur, sans encodeur** — ce cycle n'ajoute aucune dépendance, et un
    /// littéral a l'avantage de rester exactement ce qu'un attaquant enverrait. En clair :
    /// en-tête `{"alg":"none","typ":"JWT"}`, charge utile réclamant `cpt.compte.gerer` avec une
    /// expiration lointaine, signature vide.
    #[test]
    fn un_jeton_non_signe_est_refuse() {
        let forge = concat!(
            "eyJhbGciOiJub25lIiwidHlwIjoiSldUIn0.",
            "eyJzdWIiOiIwMDAwMDAwMC0wMDAwLTAwMDAtMDAwMC0wMDAwMDAwMDAwMDAiLCJ0ZW5hbnQiOiIwMDAwMDA",
            "wMC0wMDAwLTAwMDAtMDAwMC0wMDAwMDAwMDAwMDAiLCJzaWQiOiIwMDAwMDAwMC0wMDAwLTAwMDAtMDAwMC",
            "0wMDAwMDAwMDAwMDAiLCJwZXJtcyI6WyJjcHQuY29tcHRlLmdlcmVyIl0sImlhdCI6MCwiZXhwIjo5OTk5O",
            "Tk5OTk5OX0.",
        );

        assert!(
            verifier_acces(&cle(), forge).is_err(),
            "un jeton déclarant `alg: none` a été accepté — n'importe qui pourrait alors forger \
             ses propres permissions"
        );
    }

    /// Le jeton de rafraîchissement porte **trois identifiants et rien d'autre**.
    ///
    /// Il ne porte notamment **aucune permission** : elles sont recalculées à chaque
    /// rafraîchissement, et les y figer rendrait le rafraîchissement incapable de propager un
    /// retrait de rôle — c'est-à-dire exactement ce qu'il est censé faire.
    #[test]
    fn le_jeton_de_rafraichissement_ne_porte_aucune_permission() {
        let serialise = serde_json::to_string(&ClaimsRafraichissement {
            sub: Uuid::now_v7(),
            tenant: Uuid::now_v7(),
            sid: Uuid::now_v7(),
            fid: Uuid::now_v7(),
            jti: Uuid::now_v7(),
            iat: 0,
            exp: 0,
        })
        .expect("sérialisation");

        assert!(!serialise.contains("perms"), "obtenu : {serialise}");
    }
}
