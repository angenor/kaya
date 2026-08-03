//! **Le coffre par tenant** — chiffrement au repos du numéro de pièce d'identité (FR-012,
//! principe IX, cadrage §12.1).
//!
//! # Pourquoi ce fichier existe, et pourquoi il n'a pas attendu TRX-06
//!
//! Le cadrage §12.1 pose un « coffre chiffré **par tenant** » ; jusqu'ici il n'avait aucun
//! contenu, aucune donnée sensible n'étant encore stockée. **La donnée naît à ce cycle** : c'est
//! `0029` qui alimente `comptes.personne.numero_piece`, posée et vide depuis le cycle 003.
//!
//! **Ne pas repousser à TRX-06.** TRX-06 (P1) apporte l'export, la suppression et la purge
//! paramétrable — c'est-à-dire ce qu'on fait d'une donnée protégée. Il n'apporte **pas** la
//! protection. Repousser reviendrait à constituer un fichier d'identités en clair pendant une
//! tranche entière, ce que l'ARTCI interdit et que le commentaire de `0015` annonçait déjà comme
//! « le moyen le plus simple » de le faire.
//!
//! # Ce que ce coffre garantit, et ce qu'il ne garantit pas
//!
//! | Garanti | Pas garanti |
//! |---|---|
//! | Un vidage de la base ne rend aucun numéro lisible | Une compromission du processus applicatif |
//! | Deux tenants n'ont **pas** la même clé | La rotation de clé — voir la dette ci-dessous |
//! | Un cryptogramme déplacé d'un tenant à l'autre **ne se déchiffre pas** | — |
//!
//! Le troisième point n'est pas gratuit : le `tenant_id` entre dans les **données authentifiées
//! additionnelles** de l'AEAD, en plus de servir à dériver la clé. Recopier une ligne d'un tenant
//! vers un autre par une requête directe produit un échec d'authentification, pas un déchiffrement
//! silencieux.
//!
//! # ⚠️ Dette nommée : la rotation de clé n'est pas implémentée
//!
//! Le cryptogramme porte un **numéro de version de clé** (`v1:` en tête) précisément pour qu'une
//! rotation soit possible **sans migration** : une version nouvelle se déchiffre par la clé
//! nouvelle, l'ancienne reste lisible. Le mécanisme de rotation lui-même — deux clés maîtresses
//! en vol, réécriture progressive — relève de **TRX-06** et n'est pas ici. Ce qui est ici, c'est
//! le **format qui la rendra possible** ; l'omettre aurait imposé une migration de toutes les
//! lignes le jour venu.
//!
//! # Pourquoi Argon2id pour dériver, et pourquoi le cache n'est pas une optimisation
//!
//! `argon2` est **déjà** une dépendance de ce crate : dériver avec lui n'ajoute aucune dépendance
//! de dérivation. Il est lent **par construction** — c'est sa raison d'être —, ce qui serait
//! inacceptable à chaque lecture de fiche. La clé dérivée est donc **mise en cache par tenant**
//! dans le processus. Ce n'est pas une optimisation prématurée : sans cache, l'écran de recherche
//! paierait une dérivation Argon2 par résultat, et la cible des 300 ms serait perdue pour une
//! raison qui n'a rien à voir avec la recherche.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

// ⚠️ **API d'`aes-gcm` 0.11, pas de 0.10.** Le tirage du nonce est passé de
// `Aes256Gcm::generate_nonce(&mut OsRng)` au trait `Generate` — `Nonce::generate()` —, et `OsRng`
// n'est plus réexporté depuis `aead`. Tout extrait trouvé en ligne vise 0.10 et ne compilera pas :
// c'est le même piège que sqlx 0.9 contre 0.8, consigné en tête de `CLAUDE.md`.
use aes_gcm::aead::{Aead, AeadCore, Generate, KeyInit, Payload};
use aes_gcm::{Aes256Gcm, Key, Nonce};

/// Le nonce de GCM, tel qu'`aes-gcm` 0.11 le type.
///
/// ⚠️ **`Nonce<T>` est paramétré par la TAILLE du nonce, pas par le chiffreur.** Écrire
/// `Nonce::<Aes256Gcm>` compile en apparence et échoue sur une borne `ArraySize` dont le message
/// tient sur quinze lignes de types imbriqués. L'alias ci-dessous prend la taille depuis le
/// chiffreur lui-même, ce qui la rend juste par construction et lisible à la lecture.
type NonceGcm = Nonce<<Aes256Gcm as AeadCore>::NonceSize>;

/// Longueur du nonce d'AES-GCM, en octets — **lue du type, jamais écrite en dur**.
///
/// Un littéral `12` posé dans la validation du format resterait au bon endroit et deviendrait faux
/// le jour d'un changement de chiffreur, sans qu'aucune compilation ne le signale.
const NONCE_OCTETS: usize =
    <<Aes256Gcm as AeadCore>::NonceSize as aes_gcm::aead::array::typenum::Unsigned>::USIZE;

use uuid::Uuid;

/// Nom de la variable d'environnement portant la clé maîtresse.
///
/// **Jamais dans le binaire** (cadrage §12.1 — « aucun secret dans le binaire Tauri,
/// décompilable »). Le même régime que `KAYA_JWT_CLE`.
pub const VARIABLE_CLE_MAITRESSE: &str = "KAYA_COFFRE_CLE";

/// Préfixe de version du cryptogramme.
///
/// **C'est ce qui rendra la rotation possible sans migration.** Un cryptogramme sans version
/// obligerait, le jour d'une rotation, à réécrire toutes les lignes en une fois — donc à une
/// fenêtre d'indisponibilité sur la table des identités.
const VERSION_COURANTE: &str = "v1";

/// Longueur minimale de la clé maîtresse, en octets une fois décodée de son texte.
///
/// Trente-deux, soit la taille d'une clé AES-256 : accepter plus court laisserait croire à une
/// protection que l'entropie fournie ne donne pas.
const CLE_MAITRESSE_MIN: usize = 32;

/// Échec du coffre.
///
/// ⚠️ **Aucune variante ne porte la valeur en cause**, ni en clair ni chiffrée. Une erreur qui
/// citerait le numéro qu'elle n'a pas su déchiffrer le publierait dans les journaux — exactement
/// la fuite que ce fichier existe pour empêcher.
#[derive(Debug, thiserror::Error)]
pub enum ErreurCoffre {
    #[error("clé maîtresse absente : la variable {VARIABLE_CLE_MAITRESSE} n'est pas définie")]
    CleMaitresseAbsente,

    #[error("clé maîtresse trop courte : au moins {CLE_MAITRESSE_MIN} octets attendus")]
    CleMaitresseTropCourte,

    #[error("dérivation de la clé du tenant impossible")]
    Derivation,

    #[error("cryptogramme illisible : format inattendu")]
    FormatInvalide,

    #[error("version de clé inconnue : {0}")]
    VersionInconnue(String),

    /// Le déchiffrement a échoué. **Trois causes indistinguables, et c'est délibéré** : clé fausse,
    /// cryptogramme altéré, ou ligne déplacée d'un autre tenant. Les distinguer donnerait à un
    /// attaquant un oracle sur la cause de son échec.
    #[error("déchiffrement refusé")]
    AuthentificationRefusee,

    #[error("chiffrement impossible")]
    ChiffrementImpossible,
}

/// Le coffre : une clé maîtresse, et une clé dérivée par tenant.
///
/// Construit **une fois** par la couche d'assemblage et partagé par `Arc` — le cache de clés
/// dérivées n'a d'intérêt que s'il est partagé.
pub struct CoffreTenant {
    cle_maitresse: Vec<u8>,
    /// Clés dérivées, par tenant. `RwLock` plutôt que `Mutex` : les lectures dominent largement,
    /// une dérivation n'ayant lieu qu'au premier accès d'un tenant dans le processus.
    cache: RwLock<HashMap<Uuid, Arc<Aes256Gcm>>>,
}

impl std::fmt::Debug for CoffreTenant {
    /// **La clé maîtresse ne s'imprime pas.** Un `derive(Debug)` la ferait apparaître au premier
    /// `tracing::debug!` posé sur une structure qui contient le coffre.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CoffreTenant")
            .field("cle_maitresse", &"<masquée>")
            .finish_non_exhaustive()
    }
}

impl CoffreTenant {
    /// Construit le coffre depuis l'environnement.
    pub fn depuis_environnement() -> Result<Self, ErreurCoffre> {
        let brute =
            std::env::var(VARIABLE_CLE_MAITRESSE).map_err(|_| ErreurCoffre::CleMaitresseAbsente)?;
        Self::avec_cle_maitresse(brute.as_bytes())
    }

    /// Construit le coffre depuis une clé fournie — employé par les tests.
    pub fn avec_cle_maitresse(cle: &[u8]) -> Result<Self, ErreurCoffre> {
        if cle.len() < CLE_MAITRESSE_MIN {
            return Err(ErreurCoffre::CleMaitresseTropCourte);
        }
        Ok(Self {
            cle_maitresse: cle.to_vec(),
            cache: RwLock::new(HashMap::new()),
        })
    }

    /// La clé du tenant, dérivée au premier appel puis mise en cache.
    ///
    /// **Le `tenant_id` sert de sel.** Il est public, ce qui est sans conséquence ici : le sel d'un
    /// KDF n'a pas à être secret, il a à être **distinct**, et un UUID l'est par construction. Ce
    /// que cela garantit est précis — deux tenants n'ont pas la même clé, donc une base compromise
    /// sur un tenant ne livre pas les autres.
    fn cle_du_tenant(&self, tenant_id: Uuid) -> Result<Arc<Aes256Gcm>, ErreurCoffre> {
        if let Ok(cache) = self.cache.read()
            && let Some(cle) = cache.get(&tenant_id)
        {
            return Ok(Arc::clone(cle));
        }

        let mut octets = [0u8; 32];
        argon2::Argon2::default()
            .hash_password_into(&self.cle_maitresse, tenant_id.as_bytes(), &mut octets)
            .map_err(|_| ErreurCoffre::Derivation)?;

        let chiffreur = Arc::new(Aes256Gcm::new(&Key::<Aes256Gcm>::from(octets)));

        if let Ok(mut cache) = self.cache.write() {
            cache.insert(tenant_id, Arc::clone(&chiffreur));
        }
        Ok(chiffreur)
    }

    /// Chiffre une valeur pour un tenant.
    ///
    /// Rend `v1:<nonce en hexadécimal>:<cryptogramme en hexadécimal>`. Le format est **textuel**
    /// parce que la colonne est `TEXT` : la rendre `BYTEA` aurait imposé de modifier `0015`, ce que
    /// P-02 interdit.
    ///
    /// # Le nonce est tiré à chaque appel, et c'est la propriété qui compte
    ///
    /// Réutiliser un nonce avec la même clé **détruit** la garantie de GCM — pas la dégrade, la
    /// détruit. Deux fiches portant le même numéro de pièce produisent donc deux cryptogrammes
    /// différents, ce qui a un effet de bord utile : on ne peut pas déduire l'égalité de deux
    /// numéros en comparant les colonnes.
    pub fn chiffrer(&self, tenant_id: Uuid, clair: &str) -> Result<String, ErreurCoffre> {
        let chiffreur = self.cle_du_tenant(tenant_id)?;
        let nonce = NonceGcm::generate();

        // Le `tenant_id` entre dans les données authentifiées additionnelles : un cryptogramme
        // déplacé vers un autre tenant par une requête directe échoue à l'authentification.
        let cryptogramme = chiffreur
            .encrypt(
                &nonce,
                Payload {
                    msg: clair.as_bytes(),
                    aad: tenant_id.as_bytes(),
                },
            )
            .map_err(|_| ErreurCoffre::ChiffrementImpossible)?;

        Ok(format!(
            "{VERSION_COURANTE}:{}:{}",
            hexadecimal(&nonce),
            hexadecimal(&cryptogramme)
        ))
    }

    /// Déchiffre une valeur pour un tenant.
    ///
    /// ⚠️ **Cette fonction ne journalise rien.** La journalisation de l'accès est le travail du
    /// service, qui seul connaît l'auteur et le contexte — et qui l'écrit **dans la transaction**.
    /// La mettre ici la rendrait invisible au test d'immuabilité du registre, et surtout la ferait
    /// dépendre d'un pool que le coffre n'a pas.
    pub fn dechiffrer(&self, tenant_id: Uuid, stocke: &str) -> Result<String, ErreurCoffre> {
        let mut parties = stocke.splitn(3, ':');
        let version = parties.next().ok_or(ErreurCoffre::FormatInvalide)?;
        let nonce_hex = parties.next().ok_or(ErreurCoffre::FormatInvalide)?;
        let corps_hex = parties.next().ok_or(ErreurCoffre::FormatInvalide)?;

        if version != VERSION_COURANTE {
            return Err(ErreurCoffre::VersionInconnue(version.to_owned()));
        }

        let nonce_octets = depuis_hexadecimal(nonce_hex).ok_or(ErreurCoffre::FormatInvalide)?;
        let corps = depuis_hexadecimal(corps_hex).ok_or(ErreurCoffre::FormatInvalide)?;
        let nonce = NonceGcm::try_from(&nonce_octets[..])
            .map_err(|_| ErreurCoffre::FormatInvalide)?;

        let chiffreur = self.cle_du_tenant(tenant_id)?;
        let clair = chiffreur
            .decrypt(
                &nonce,
                Payload {
                    msg: &corps,
                    aad: tenant_id.as_bytes(),
                },
            )
            .map_err(|_| ErreurCoffre::AuthentificationRefusee)?;

        String::from_utf8(clair).map_err(|_| ErreurCoffre::AuthentificationRefusee)
    }

    /// Vrai si la valeur stockée a la forme d'un cryptogramme de ce coffre.
    ///
    /// Employé par le test qui vérifie qu'**aucune requête directe sous le rôle applicatif ne lit
    /// un numéro en clair** : la colonne ne doit jamais contenir autre chose que cette forme.
    pub fn est_un_cryptogramme(valeur: &str) -> bool {
        let mut parties = valeur.splitn(3, ':');
        matches!(
            (parties.next(), parties.next(), parties.next()),
            (Some(VERSION_COURANTE), Some(n), Some(c))
                if n.len() == NONCE_OCTETS * 2
                    && n.chars().all(|c| c.is_ascii_hexdigit())
                    && !c.is_empty()
                    && c.chars().all(|c| c.is_ascii_hexdigit())
        )
    }
}

/// Encodage hexadécimal — **écrit ici plutôt qu'importé**.
///
/// `hex` est au `Cargo.lock` de manière transitive, mais l'ajouter en dépendance directe pour deux
/// boucles imposerait une entrée de plus à la revue mensuelle du gel. Le principe XI vise
/// exactement ce genre d'arbitrage : une dépendance se justifie par ce qu'elle évite d'écrire, et
/// ces deux fonctions tiennent en dix lignes.
fn hexadecimal(octets: &[u8]) -> String {
    let mut sortie = String::with_capacity(octets.len() * 2);
    for octet in octets {
        use std::fmt::Write as _;
        let _ = write!(sortie, "{octet:02x}");
    }
    sortie
}

fn depuis_hexadecimal(texte: &str) -> Option<Vec<u8>> {
    if !texte.len().is_multiple_of(2) {
        return None;
    }
    (0..texte.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&texte[i..i + 2], 16).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn coffre() -> CoffreTenant {
        CoffreTenant::avec_cle_maitresse(b"une cle maitresse de test suffisamment longue")
            .expect("clé de test valide")
    }

    #[test]
    fn un_numero_chiffre_puis_dechiffre_est_identique() {
        let coffre = coffre();
        let tenant = Uuid::now_v7();
        let chiffre = coffre.chiffrer(tenant, "CI00123456").expect("chiffrement");
        assert_eq!(coffre.dechiffrer(tenant, &chiffre).expect("déchiffrement"), "CI00123456");
    }

    /// **Le cryptogramme ne contient jamais le clair.** L'assertion paraît naïve ; elle attrape
    /// l'erreur d'un jour de fatigue où `chiffrer` rendrait son entrée.
    #[test]
    fn le_cryptogramme_ne_contient_pas_le_clair() {
        let coffre = coffre();
        let chiffre = coffre.chiffrer(Uuid::now_v7(), "CI00123456").expect("chiffrement");
        assert!(!chiffre.contains("CI00123456"));
        assert!(CoffreTenant::est_un_cryptogramme(&chiffre));
    }

    /// **Deux chiffrements de la même valeur diffèrent** — le nonce est tiré à chaque appel.
    ///
    /// Effet de bord utile : on ne peut pas déduire que deux fiches portent le même numéro en
    /// comparant les colonnes.
    #[test]
    fn deux_chiffrements_de_la_meme_valeur_different() {
        let coffre = coffre();
        let tenant = Uuid::now_v7();
        let a = coffre.chiffrer(tenant, "CI00123456").expect("chiffrement");
        let b = coffre.chiffrer(tenant, "CI00123456").expect("chiffrement");
        assert_ne!(a, b, "un nonce réutilisé DÉTRUIT la garantie de GCM");
    }

    /// ★ **Un cryptogramme déplacé vers un autre tenant ne se déchiffre pas.**
    ///
    /// C'est ce que le `tenant_id` en données authentifiées additionnelles achète, en plus de la
    /// dérivation. Sans lui, une requête directe recopiant une ligne d'un tenant à l'autre
    /// produirait un déchiffrement silencieux le jour où les deux clés se rejoindraient.
    #[test]
    fn un_cryptogramme_d_un_autre_tenant_est_refuse() {
        let coffre = coffre();
        let tenant_a = Uuid::now_v7();
        let tenant_b = Uuid::now_v7();

        let chiffre = coffre.chiffrer(tenant_a, "CI00123456").expect("chiffrement");
        let resultat = coffre.dechiffrer(tenant_b, &chiffre);

        assert!(
            matches!(resultat, Err(ErreurCoffre::AuthentificationRefusee)),
            "un cryptogramme d'un autre tenant doit être REFUSÉ, jamais déchiffré"
        );
    }

    /// Un cryptogramme altéré d'un seul caractère est refusé — c'est l'authentification de GCM.
    #[test]
    fn un_cryptogramme_altere_est_refuse() {
        let coffre = coffre();
        let tenant = Uuid::now_v7();
        let chiffre = coffre.chiffrer(tenant, "CI00123456").expect("chiffrement");

        let mut altere: Vec<char> = chiffre.chars().collect();
        let dernier = altere.len() - 1;
        altere[dernier] = if altere[dernier] == 'a' { 'b' } else { 'a' };
        let altere: String = altere.into_iter().collect();

        assert!(matches!(
            coffre.dechiffrer(tenant, &altere),
            Err(ErreurCoffre::AuthentificationRefusee)
        ));
    }

    #[test]
    fn une_cle_maitresse_trop_courte_est_refusee() {
        assert!(matches!(
            CoffreTenant::avec_cle_maitresse(b"trop court"),
            Err(ErreurCoffre::CleMaitresseTropCourte)
        ));
    }

    /// Le format porte sa version — **c'est ce qui rendra la rotation possible sans migration**.
    #[test]
    fn le_cryptogramme_porte_sa_version_de_cle() {
        let coffre = coffre();
        let chiffre = coffre.chiffrer(Uuid::now_v7(), "CI00123456").expect("chiffrement");
        assert!(chiffre.starts_with("v1:"));
    }

    #[test]
    fn une_version_inconnue_est_nommee_et_non_confondue_avec_une_alteration() {
        let coffre = coffre();
        let resultat = coffre.dechiffrer(Uuid::now_v7(), "v9:0011223344556677889900aa:ffee");
        assert!(matches!(resultat, Err(ErreurCoffre::VersionInconnue(v)) if v == "v9"));
    }

    #[test]
    fn une_valeur_en_clair_n_est_pas_prise_pour_un_cryptogramme() {
        assert!(!CoffreTenant::est_un_cryptogramme("CI00123456"));
        assert!(!CoffreTenant::est_un_cryptogramme("v1:pastexadecimal:ffee"));
        assert!(!CoffreTenant::est_un_cryptogramme(""));
    }

    /// **La clé maîtresse ne s'imprime pas.** Sans `Debug` manuel, un `tracing::debug!` posé sur
    /// une structure contenant le coffre la publierait.
    #[test]
    fn la_cle_maitresse_ne_s_imprime_pas() {
        let rendu = format!("{:?}", coffre());
        assert!(rendu.contains("masquée"));
        assert!(!rendu.contains("une cle maitresse"));
    }
}
