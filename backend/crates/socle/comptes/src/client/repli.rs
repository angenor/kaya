//! **Le repli d'un nom pour la recherche** — minuscules, sans signes diacritiques, sans
//! apostrophe.
//!
//! Chercher « kouame » doit trouver « KOUAMÉ ». Chercher « nguessan » doit trouver « N'Guessan »
//! **et** « N’Guessan », écrits avec deux apostrophes différentes que rien ne distingue à l'œil.
//!
//! # Pourquoi une table écrite à la main plutôt qu'`unicode-normalization`
//!
//! C'est une décision, pas un raccourci (research R-04, `plan.md` § Technical Context).
//!
//! La bibliothèque naturelle — décomposer en NFD puis retirer les marques combinantes — **n'est
//! pas au gel des versions**. L'ajouter imposerait une entrée nouvelle à `docs/versions-gelees.md`,
//! donc une décision de revue mensuelle, pour un besoin que soixante correspondances couvrent
//! entièrement.
//!
//! Le bénéfice n'est pas seulement d'éviter une dépendance : **le produit décide de ce qu'il
//! replie** au lieu d'hériter du choix d'une bibliothèque. NFD replierait aussi le grec, le
//! cyrillique et le vietnamien — que ce produit ne cherche pas — et laisserait passer `Ø` et `Đ`,
//! qui ne sont pas des lettres accentuées mais des lettres à barre, sans décomposition canonique.
//! Ici, `Ø → o` et `Đ → d` sont écrits, donc vus.
//!
//! # Ce que le repli fait, dans l'ordre
//!
//! 1. minuscules ;
//! 2. signes diacritiques latins retirés, **par la table ci-dessous** ;
//! 3. apostrophes retirées — **droite `U+0027` ET typographique `U+2019`** ;
//! 4. tirets et espaces réduits à un espace unique, bords rognés.
//!
//! # ⚠️ Le point 3 se paie cher s'il est manqué
//!
//! `N'Guessan` est l'un des noms les plus répandus de Côte d'Ivoire. Les claviers logiciels
//! d'Android et d'iOS produisent l'apostrophe **typographique** `’` par correction automatique,
//! là où un clavier physique produit la **droite** `'`. Une fiche créée au comptoir sur tablette
//! et cherchée depuis un poste fixe ne se retrouverait pas, et le symptôme — « la fiche a
//! disparu » — n'oriente vers aucune cause.
//!
//! Les deux sont donc **retirées**, pas remplacées par un caractère commun : `nguessan` est la
//! forme cherchable, et c'est aussi ce que quelqu'un tape spontanément.

/// Correspondances des signes diacritiques latins vers leur lettre de base.
///
/// **Minuscules seulement** : le repli passe en minuscules **avant** de consulter cette table, ce
/// qui divise par deux le nombre d'entrées et supprime toute possibilité qu'une majuscule accentuée
/// soit repliée différemment de sa minuscule.
///
/// Ordonnée par lettre de base, pour qu'une lacune se voie à la lecture.
const DIACRITIQUES: &[(char, &str)] = &[
    // a
    ('à', "a"), ('á', "a"), ('â', "a"), ('ã', "a"), ('ä', "a"), ('å', "a"), ('ā', "a"),
    ('ă', "a"), ('ą', "a"),
    // æ — une ligature, donc DEUX lettres. Le rendre « a » perdrait le « e » et rendrait
    // « Lætitia » introuvable en tapant « laetitia », qui est l'orthographe usuelle.
    ('æ', "ae"),
    // c
    ('ç', "c"), ('ć', "c"), ('ĉ', "c"), ('č', "c"),
    // d — `đ` est une lettre BARRÉE : elle n'a aucune décomposition canonique, et une approche
    // par NFD la laisserait passer telle quelle.
    ('ď', "d"), ('đ', "d"),
    // e
    ('è', "e"), ('é', "e"), ('ê', "e"), ('ë', "e"), ('ē', "e"), ('ĕ', "e"), ('ė', "e"),
    ('ę', "e"), ('ě', "e"),
    // g
    ('ĝ', "g"), ('ğ', "g"), ('ġ', "g"), ('ģ', "g"),
    // i
    ('ì', "i"), ('í', "i"), ('î', "i"), ('ï', "i"), ('ĩ', "i"), ('ī', "i"), ('į', "i"),
    ('ı', "i"),
    // l
    ('ł', "l"), ('ľ', "l"), ('ĺ', "l"), ('ļ', "l"),
    // n
    ('ñ', "n"), ('ń', "n"), ('ň', "n"), ('ņ', "n"),
    // o — `ø` est barrée, même remarque que `đ`.
    ('ò', "o"), ('ó', "o"), ('ô', "o"), ('õ', "o"), ('ö', "o"), ('ø', "o"), ('ō', "o"),
    ('ŏ', "o"), ('ő', "o"),
    // œ — ligature, deux lettres. « Sœur » se cherche « soeur ».
    ('œ', "oe"),
    // r
    ('ŕ', "r"), ('ř', "r"), ('ŗ', "r"),
    // s
    ('ś', "s"), ('ŝ', "s"), ('ş', "s"), ('š', "s"),
    // ß — une ligature aussi, et la forme cherchable est « ss ».
    ('ß', "ss"),
    // t
    ('ţ', "t"), ('ť', "t"), ('ŧ', "t"),
    // u
    ('ù', "u"), ('ú', "u"), ('û', "u"), ('ü', "u"), ('ũ', "u"), ('ū', "u"), ('ŭ', "u"),
    ('ů', "u"), ('ű', "u"), ('ų', "u"),
    // y
    ('ý', "y"), ('ÿ', "y"), ('ŷ', "y"),
    // z
    ('ź', "z"), ('ż', "z"), ('ž', "z"),
];

/// Les deux apostrophes que les claviers produisent, **retirées** et non remplacées.
///
/// `U+2019` est ce que produit la correction automatique d'Android et d'iOS ; `U+0027` est ce que
/// produit un clavier physique. Rien ne les distingue à l'œil, et `N'Guessan` saisi au comptoir
/// serait introuvable depuis un poste fixe.
const APOSTROPHES: &[char] = &['\u{0027}', '\u{2019}'];

/// Replie une chaîne pour la recherche.
///
/// Rend une chaîne **toujours** en minuscules, sans signe diacritique connu, sans apostrophe,
/// sans espace ni tiret superflu. Une entrée vide ou faite d'espaces rend une chaîne vide — c'est
/// à l'appelant de décider si cela vaut `NULL` en base.
pub fn repli(entree: &str) -> String {
    // ── 1 · minuscules AVANT la table : les majuscules accentuées n'ont pas à y figurer ────────
    let minuscules = entree.to_lowercase();

    let mut sortie = String::with_capacity(minuscules.len());
    for caractere in minuscules.chars() {
        // ── 3 · les deux apostrophes disparaissent ────────────────────────────────────────────
        if APOSTROPHES.contains(&caractere) {
            continue;
        }

        // ── 4 · tiret et espace deviennent le même séparateur ─────────────────────────────────
        if caractere.is_whitespace() || caractere == '-' || caractere == '\u{2010}' {
            sortie.push(' ');
            continue;
        }

        // ── 2 · le repli des signes diacritiques ──────────────────────────────────────────────
        match DIACRITIQUES.iter().find(|(source, _)| *source == caractere) {
            Some((_, remplacement)) => sortie.push_str(remplacement),
            None => sortie.push(caractere),
        }
    }

    // Réduction des espaces consécutifs et rognage — `split_whitespace` fait les deux.
    sortie.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Replie un numéro de téléphone : **les chiffres seuls**.
///
/// Un `+` initial disparaît comme le reste : la comparaison se fait par **suffixe**, et
/// « 0707123456 » doit retrouver « +2250707123456 ». La préfixation par l'indicatif de
/// l'établissement est faite par le service, qui seul connaît la configuration héritée (CPT-01) —
/// ce n'est pas le travail de cette fonction, qui ne doit rien savoir d'un établissement.
pub fn repli_telephone(entree: &str) -> String {
    entree.chars().filter(char::is_ascii_digit).collect()
}

/// Replie un numéro de pièce d'identité : **alphanumérique en majuscules**.
///
/// Espaces, tirets, points et barres obliques disparaissent. « CI-0012 3456 » et « ci00123456 »
/// sont le même numéro, et le sont pour tout le monde sauf pour une comparaison littérale.
///
/// ⚠️ **Cette fonction ne voit jamais que la forme repliée d'un numéro, et le repli n'est pas du
/// chiffrement.** `numero_piece_repli` est un index de recherche, stocké en clair ; le numéro
/// lui-même est chiffré au repos et sa lecture journalisée (FR-012). Deux traitements distincts
/// pour deux usages distincts — écrit ici pour qu'une relecture ne prenne pas l'un pour l'autre.
pub fn repli_piece(entree: &str) -> String {
    entree
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .flat_map(char::to_uppercase)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Le jeu de noms est ivoirien et réel** — pas un alphabet de test.
    ///
    /// Chacun de ces noms est courant à Abengourou. Un jeu synthétique du type « aàâä » aurait
    /// couvert la table sans jamais montrer que `N'Guessan` a deux apostrophes possibles, ni que
    /// `Éboué` en porte deux différents.
    #[test]
    fn le_jeu_de_noms_ivoiriens_se_replie_comme_on_le_taperait() {
        let cas = [
            ("Kouamé", "kouame"),
            ("N'Guessan", "nguessan"),   // apostrophe DROITE — clavier physique
            ("N’Guessan", "nguessan"),   // apostrophe TYPOGRAPHIQUE — correction automatique
            ("Aïcha", "aicha"),
            ("Traoré", "traore"),
            ("Koffi", "koffi"),
            ("Yao", "yao"),
            ("Bakayoko", "bakayoko"),
            ("Adjoua", "adjoua"),
            ("Éboué", "eboue"),
            ("Gbagbo", "gbagbo"),
            ("Ouattara", "ouattara"),
        ];

        for (saisie, attendu) in cas {
            assert_eq!(
                repli(saisie),
                attendu,
                "« {saisie} » devrait se replier en « {attendu} » — c'est ce que quelqu'un tape"
            );
        }
    }

    /// **Les deux apostrophes doivent produire LA MÊME chaîne.**
    ///
    /// C'est l'assertion qui compte : l'égalité entre les deux formes, et pas seulement le fait
    /// que chacune se replie. Une implémentation qui retirerait `U+0027` et laisserait `U+2019`
    /// passerait le test précédent sur onze cas sur douze.
    #[test]
    fn les_deux_apostrophes_sont_indiscernables_apres_repli() {
        assert_eq!(repli("N'Guessan"), repli("N’Guessan"));
        assert_eq!(repli("N'Dri"), repli("N’Dri"));
        assert_eq!(repli("D'Almeida"), repli("D’Almeida"));
    }

    #[test]
    fn le_nom_complet_reduit_ses_espaces_et_ses_traits_d_union() {
        assert_eq!(repli("  Kouamé   Yao  "), "kouame yao");
        assert_eq!(repli("Marie-Claire"), "marie claire");
        assert_eq!(repli("Jean--Baptiste"), "jean baptiste");
    }

    /// Les ligatures rendent **deux** lettres. « Lætitia » se cherche « laetitia ».
    #[test]
    fn les_ligatures_rendent_deux_lettres() {
        assert_eq!(repli("Lætitia"), "laetitia");
        assert_eq!(repli("Sœur"), "soeur");
        assert_eq!(repli("Straße"), "strasse");
    }

    /// Les lettres **barrées** n'ont aucune décomposition canonique : une approche par NFD les
    /// laisserait passer. Elles sont dans la table, donc vues.
    #[test]
    fn les_lettres_barrees_sont_repliees_alors_que_nfd_les_laisserait() {
        assert_eq!(repli("Søren"), "soren");
        assert_eq!(repli("Đan"), "dan");
        assert_eq!(repli("Łukasz"), "lukasz");
    }

    /// ★ **La limite du repli est ÉCRITE, pas découverte en production.**
    ///
    /// La table couvre le répertoire **latin européen**. Elle ne couvre **pas** les marques de ton
    /// vietnamiennes (`ặ`, `ế`, `ộ`), et c'est cohérent avec la décision de tête de module : NFD
    /// replierait le grec, le cyrillique et le vietnamien, que ce produit ne cherche pas, et c'est
    /// précisément pourquoi la table est écrite plutôt qu'héritée.
    ///
    /// Ce test **asserte la limite** au lieu de la taire. Le jour où un pilote vietnamien
    /// arriverait, il rougirait — ce qui est exactement ce qu'on veut : une lacune qui se signale
    /// vaut mieux qu'une recherche qui ne trouve pas sans dire pourquoi.
    #[test]
    fn les_marques_de_ton_vietnamiennes_ne_sont_pas_couvertes_et_c_est_declare() {
        // La lettre barrée passe — elle est au répertoire latin.
        assert!(repli("Đặng").starts_with('d'));
        // La marque de ton ne passe pas, et c'est le périmètre déclaré.
        assert_ne!(
            repli("Đặng"),
            "dang",
            "si ce test rougit, la table a gagné le vietnamien : mettre à jour le commentaire de \
             tête du module, qui déclare le répertoire latin européen"
        );
    }

    #[test]
    fn une_saisie_vide_ou_blanche_rend_une_chaine_vide() {
        assert_eq!(repli(""), "");
        assert_eq!(repli("   "), "");
        assert_eq!(repli(" - "), "");
    }

    #[test]
    fn le_telephone_ne_garde_que_les_chiffres() {
        assert_eq!(repli_telephone("+225 07 07 12 34 56"), "2250707123456");
        assert_eq!(repli_telephone("07-07-12-34-56"), "0707123456");
        assert_eq!(repli_telephone("(+225) 0707123456"), "2250707123456");
    }

    #[test]
    fn le_numero_de_piece_ignore_la_ponctuation_et_la_casse() {
        assert_eq!(repli_piece("CI-0012 3456"), "CI00123456");
        assert_eq!(repli_piece("ci00123456"), "CI00123456");
        assert_eq!(repli_piece("C.I. 0012/3456"), "CI00123456");
    }

    /// Le repli est **idempotent** : replier une forme déjà repliée ne la change pas.
    ///
    /// Sans cette propriété, une fiche recalculée à la modification pourrait dériver de sa forme
    /// initiale, et la recherche cesserait de trouver une fiche qu'elle trouvait avant.
    #[test]
    fn le_repli_est_idempotent() {
        for nom in ["Kouamé", "N’Guessan", "Marie-Claire", "Lætitia", "  Yao  "] {
            let une_fois = repli(nom);
            assert_eq!(repli(&une_fois), une_fois, "repli non idempotent sur « {nom} »");
        }
    }
}
