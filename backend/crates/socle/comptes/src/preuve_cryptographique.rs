//! **Constat de construction, pas une brique du produit.**
//!
//! # Pourquoi ce fichier existe, et quand il disparaît
//!
//! Le poste de développement est `darwin/arm64`, la cible de production `linux/amd64`. `argon2`
//! (RustCrypto) est du Rust pur et se construit partout ; **la chaîne cryptographique de
//! `jsonwebtoken` porte de l'assembleur par architecture** — `ring` en dépend transitivement. Les
//! deux cibles sont annoncées supportées ; les deux ont donc été **constatées**, par un
//! `docker buildx build --platform linux/amd64` exécuté au tout début du cycle 003
//! (research.md R-16).
//!
//! Le constat coûte une heure ici. Découvert au recollement de fin de cycle, quand vingt tâches
//! reposent sur la signature de jetons, il coûte une semaine.
//!
//! **Un test unitaire n'aurait rien prouvé** : `cargo test` ne s'exécute pas dans l'image de
//! production, et l'étape de construction du `Dockerfile.api` ne compile que le binaire. Il fallait
//! du code **atteint par `cargo build --release -p kaya-api`**, donc appelé depuis le graphe du
//! binaire — d'où l'appel dans `main.rs`, sous la journalisation de démarrage.
//!
//! Ce module est retiré par la tâche T027, quand `authentification/argon2.rs` et `session/jeton.rs`
//! exercent les deux crates sur les vrais chemins. **Le supprimer plus tôt reviendrait à retirer
//! l'échafaudage avant que le mur tienne.**

/// Empreinte des deux chaînes cryptographiques du cycle, exercées **à l'exécution**.
///
/// Rend une description courte, journalisée au démarrage. Le contenu importe peu ; ce qui compte
/// est que ces deux appels aient dû être **compilés et liés** pour l'architecture cible.
pub fn empreinte_chaines() -> String {
    let argon = empreinte_argon2();
    let jeton = aller_retour_jeton();
    format!("argon2={argon} jsonwebtoken={jeton}")
}

/// Un hachage Argon2id sur un sel constant.
///
/// **Ni les paramètres ni le sel ne préfigurent ceux du produit** : ils sont fixés par
/// `authentification/argon2.rs` (T025), avec la source de la recommandation. Ici, un sel constant
/// suffit — et évite d'exercer le générateur d'aléa au démarrage pour un constat.
fn empreinte_argon2() -> &'static str {
    use argon2::{Argon2, PasswordHasher, password_hash::SaltString};

    let sel = SaltString::from_b64("a2F5YS1jb25zdGF0LTAwMQ").expect("sel de constat valide");
    match Argon2::default().hash_password(b"constat-de-construction", &sel) {
        Ok(_) => "ok",
        Err(_) => "indisponible",
    }
}

/// Un aller-retour de signature et de vérification HS256.
///
/// La clé est littérale et sans valeur : **aucun secret ne vit dans le binaire** (principe IX). La
/// clé de production est `KAYA_JWT_CLE`, lue de l'environnement, et le démarrage échoue sans elle.
fn aller_retour_jeton() -> &'static str {
    use jsonwebtoken::{
        Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode,
        get_current_timestamp,
    };
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    struct Constat {
        sub: String,
        exp: u64,
    }

    let cle = b"clef-de-constat-sans-valeur-0000";
    let charge = Constat {
        sub: "constat".to_owned(),
        exp: get_current_timestamp() + 60,
    };

    let signe = match encode(
        &Header::new(Algorithm::HS256),
        &charge,
        &EncodingKey::from_secret(cle),
    ) {
        Ok(jeton) => jeton,
        Err(_) => return "signature indisponible",
    };

    match decode::<Constat>(
        &signe,
        &DecodingKey::from_secret(cle),
        &Validation::new(Algorithm::HS256),
    ) {
        Ok(_) => "ok",
        Err(_) => "vérification indisponible",
    }
}

#[cfg(test)]
mod tests {
    /// Les deux chaînes répondent — sur l'architecture qui exécute ce test.
    ///
    /// Ce test ne remplace pas la construction `linux/amd64` : il s'exécute sur le poste de
    /// développement, donc `arm64`. C'est le `docker buildx` qui porte le constat de la cible.
    #[test]
    fn les_deux_chaines_repondent() {
        let empreinte = super::empreinte_chaines();
        assert_eq!(
            empreinte, "argon2=ok jsonwebtoken=ok",
            "une des deux chaînes cryptographiques n'a pas répondu : {empreinte}"
        );
    }
}
