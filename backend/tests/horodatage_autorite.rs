//! **PORTE P-23 — PROVENANCE DE L'INSTANT.**
//!
//! > *Aucun calcul métier, fiscal, de clôture ou de durée ne s'appuie sur `horodatage_client`.
//! > Seul l'horodatage d'autorité serveur fait foi.* — constitution 1.8.0, principe IV.
//!
//! ═══════════════════════════════════════════════════════════════════════════════════════════
//!  POURQUOI CETTE PORTE EXISTE, ET POURQUOI MAINTENANT
//! ═══════════════════════════════════════════════════════════════════════════════════════════
//!
//! Le principe IV exige **en toutes lettres**, depuis la ratification, que « toute logique métier,
//! tout calcul fiscal, toute clôture et tout calcul de durée de passage s'appuient exclusivement
//! sur l'horodatage d'autorité serveur ». **Aucune des vingt-cinq portes ne le gardait.** P-09
//! vérifie que les occupations sont des intervalles protégés par une contrainte d'exclusion — pas
//! la *provenance* d'un instant.
//!
//! La colonne `horodatage_client` existe sur quatre tables depuis les cycles précédents, et rien
//! n'empêchait un calcul de s'y appuyer.
//!
//! **Le moment est celui qui coûte le moins.** SEJ et FIS écriront les premières règles de durée de
//! passage et de taxe de nuitée — exactement les calculs que le principe IV vise. Poser la porte
//! maintenant coûte ce fichier ; la poser après coûte la revue de deux moteurs déjà écrits. Le
//! cadrage §11.4 en donne la raison : « un téléphone d'entrée de gamme dérive et le personnel
//! change l'heure », et « le passage aggrave la sensibilité à l'horloge » puisqu'il se facture à
//! l'heure.
//!
//! ═══════════════════════════════════════════════════════════════════════════════════════════
//!  PÉRIMÈTRE — DÉCOUVERT, JAMAIS ÉNUMÉRÉ (exigence 1)
//! ═══════════════════════════════════════════════════════════════════════════════════════════
//!
//! **Inspecté** : tous les fichiers `.rs` des crates métier que `commun::perimetre` découvre —
//! socle, capacités, verticales —, plus `domain` et `api/src`. Aucun chemin n'est écrit ici :
//! l'énumération à la main a laissé un trou à chacun des quatre cycles précédents.
//!
//! **Non inspecté**, et il faut le lire avant de conclure quoi que ce soit d'un vert :
//!
//! - `backend/tests/` — un test qui compare deux horodatages n'est pas un calcul métier du
//!   produit. Les inclure ferait échouer la porte sur les tests de la dérive elle-même.
//! - Les **commentaires**. Ce fichier-ci nomme la colonne dans chaque paragraphe, et une porte qui
//!   échouerait sur sa propre documentation serait désactivée dans la semaine.
//! - Le **front**. `app/core/sync/horloge.ts` compare bien l'horloge locale à l'horodatage
//!   d'autorité de la réponse — c'est l'exemption « rendu de l'instant tel que le terminal l'a
//!   perçu », et le versant TypeScript n'est pas dans le périmètre de cette porte-ci.
//!
//! ═══════════════════════════════════════════════════════════════════════════════════════════
//!  LES TROIS EXEMPTIONS — LIMITATIVEMENT ÉNUMÉRÉES, ET LA LISTE EST CLOSE
//! ═══════════════════════════════════════════════════════════════════════════════════════════
//!
//! La constitution les nomme, à la lettre :
//!
//!   1. **ordre d'affichage local** ;
//!   2. **détection de dérive d'horloge** ;
//!   3. **rendu de l'instant tel que le terminal l'a perçu**.
//!
//! **Ce que « limitativement » impose, et qu'on lirait mal.** La liste est close. La couche de
//! persistance qui **écrit** la colonne n'y figure pas — et n'a pas à y figurer : *écrire une
//! valeur n'est pas s'appuyer dessus*. Un `INSERT … horodatage_client` ne calcule rien, ne compare
//! rien, ne décide rien ; il range une donnée indicative que l'ordre d'affichage relira.
//!
//! Lui inventer une quatrième exemption élargirait la porte de sa propre autorité, ce qui est
//! exactement ce que le mot interdit. La distinction se fait donc par ce que le code **fait** de la
//! colonne, pas par l'endroit où il se trouve — voir [`usage_suspect`].

mod commun;

use std::path::PathBuf;

use commun::perimetre;

/// Le nom de la colonne indicative, sur toutes ses formes.
///
/// Le champ Rust, la colonne SQL et le champ JSON portent le même nom : c'est une convention du
/// produit, et elle rend cette porte praticable.
const COLONNE_CLIENT: &str = "horodatage_client";

/// Les fichiers du périmètre — **découverts**.
fn fichiers_inspectes() -> Vec<PathBuf> {
    let racine = perimetre::racine_backend();
    let mut fichiers = perimetre::sources_des_crates_metier();
    fichiers.extend(perimetre::sources_rust(
        &racine.join(perimetre::chemin_domain()),
    ));
    fichiers.extend(perimetre::sources_rust(&racine.join("api/src")));
    fichiers.sort();
    fichiers.dedup();
    fichiers
}

/// Une ligne fautive : elle **s'appuie** sur l'horodatage client.
#[derive(Debug)]
struct Violation {
    fichier: String,
    ligne: usize,
    code: String,
    motif: &'static str,
}

/// **Ce que « s'appuyer sur » veut dire, opérationnellement.**
///
/// Trois formes, et chacune correspond à une faute réelle qu'un cycle pourrait commettre :
///
/// | Forme | Ce qu'elle produirait |
/// |---|---|
/// | Une **soustraction** ou une comparaison d'intervalle | Une durée de passage calculée sur l'horloge d'un terminal — facturée à l'heure |
/// | Un **`ORDER BY`** ou un `WHERE` de plage en SQL | Une clôture qui inclut ou exclut des lignes selon l'heure d'un téléphone |
/// | Une **affectation** vers un champ d'autorité | L'horodatage client devenu l'horodatage d'autorité, silencieusement |
///
/// Ce que la fonction **ne** signale pas : lire la colonne, l'écrire, la sérialiser, la passer en
/// paramètre. Aucun de ces gestes ne fait dépendre une décision de l'horloge d'un terminal.
fn usage_suspect(ligne: &str) -> Option<&'static str> {
    let nu = ligne.trim();

    if !nu.contains(COLONNE_CLIENT) {
        return None;
    }

    // ── Forme 1 · arithmétique et comparaison d'instants ──────────────────────────────────────
    //
    // `autorite - horodatage_client`, `horodatage_client + duree`, `horodatage_client <`… Toute
    // durée tirée de la colonne indicative est une durée fausse dès qu'un terminal dérive.
    let arithmetique = [
        " - ", " + ", " < ", " > ", " <= ", " >= ", ".duration_since(", "num_seconds(",
        "whole_seconds(", "whole_minutes(",
    ];
    if arithmetique.iter().any(|op| {
        nu.split(COLONNE_CLIENT)
            .enumerate()
            .any(|(rang, fragment)| rang > 0 && fragment.starts_with(op))
            || nu.contains(&format!("{op}{COLONNE_CLIENT}"))
    }) {
        return Some("calcul de durée ou comparaison d'instants");
    }

    // ── Forme 2 · tri et filtrage SQL ─────────────────────────────────────────────────────────
    //
    // Le module doré trie par `cree_le DESC, id DESC` — l'horodatage d'autorité, départagé par
    // l'UUID v7. Trier par la colonne indicative rendrait l'ordre d'un registre dépendant de
    // l'heure des terminaux qui l'ont alimenté.
    if nu.contains("ORDER BY") || nu.contains("order by") {
        return Some("tri SQL sur l'horodatage indicatif");
    }
    if (nu.contains("WHERE") || nu.contains("AND ")) && nu.contains("BETWEEN") {
        return Some("filtrage de plage SQL sur l'horodatage indicatif");
    }

    // ── Forme 3 · affectation vers un champ d'autorité ────────────────────────────────────────
    //
    // `cree_le: horodatage_client` — la faute la plus discrète, et la plus grave : elle ne calcule
    // rien, elle **remplace** l'autorité.
    for champ_autorite in ["cree_le:", "survenu_le:", "resolue_le:", "modifie_le:"] {
        if nu.contains(champ_autorite)
            && nu
                .split(champ_autorite)
                .nth(1)
                .is_some_and(|apres| apres.contains(COLONNE_CLIENT))
        {
            return Some("l'horodatage indicatif affecté à un champ d'AUTORITÉ");
        }
    }

    None
}

/// Ce fichier est-il exempté, et pourquoi ?
///
/// **Une seule exemption, et elle correspond à la deuxième de la constitution** : la détection de
/// dérive compare les deux horodatages — c'est sa raison d'être, et l'interdire reviendrait à
/// interdire de constater qu'une horloge est fausse.
///
/// Les deux autres exemptions ratifiées — ordre d'affichage local, rendu de l'instant perçu — ne
/// demandent **aucun fichier exempté** : elles s'expriment par des gestes que [`usage_suspect`] ne
/// signale pas. Le dire vaut mieux que de les lister par précaution : une exemption inutile est une
/// porte élargie sans motif.
fn exemption(chemin: &str) -> Option<&'static str> {
    // Le chemin est **composé depuis les `[workspace] members`**, jamais écrit ici : une exemption
    // qui survivrait au crate qu'elle exempte serait pire qu'inutile — elle exclurait un fichier
    // inexistant, et le module réel entrerait dans le balayage sans que rien ne le dise.
    let module_de_derive = perimetre::fichier_du_crate(
        perimetre::Famille::Socle,
        "synchronisation",
        "src/derive.rs",
    );
    if chemin.ends_with(&module_de_derive) {
        return Some(
            "exemption 2 de la constitution — DÉTECTION DE DÉRIVE D'HORLOGE. Comparer les deux \
             horodatages EST sa raison d'être ; l'interdire reviendrait à interdire de constater \
             qu'une horloge est fausse.",
        );
    }
    None
}

// =================================================================================================
//  La porte
// =================================================================================================

/// **P-23 — aucun calcul ne s'appuie sur l'horodatage client.**
#[test]
fn p23_aucun_calcul_ne_s_appuie_sur_l_horodatage_client() {
    let racine = perimetre::racine_backend();
    let fichiers = fichiers_inspectes();

    assert!(
        fichiers.len() > 50,
        "seulement {} fichier(s) inspecté(s) : le périmètre découvert est vide ou cassé, et la \
         porte passerait au vert sans rien lire.",
        fichiers.len()
    );

    let mut violations = Vec::new();
    let mut fichiers_portant_la_colonne = 0usize;
    let mut exemptes = Vec::new();

    for fichier in &fichiers {
        let chemin = fichier
            .strip_prefix(&racine)
            .unwrap_or(fichier)
            .display()
            .to_string()
            .replace('\\', "/");

        let Ok(contenu) = std::fs::read_to_string(fichier) else {
            continue;
        };
        if !contenu.contains(COLONNE_CLIENT) {
            continue;
        }
        fichiers_portant_la_colonne += 1;

        if let Some(motif) = exemption(&chemin) {
            exemptes.push(format!("{chemin} — {motif}"));
            continue;
        }

        for (rang, ligne) in contenu.lines().enumerate() {
            let nu = ligne.trim();
            // Les commentaires sont écartés : ce dépôt documente abondamment, et une porte qui
            // échouerait sur sa propre justification serait désactivée dans la semaine.
            if nu.starts_with("//") || nu.starts_with("*") || nu.starts_with("///") {
                continue;
            }
            if let Some(motif) = usage_suspect(ligne) {
                violations.push(Violation {
                    fichier: chemin.clone(),
                    ligne: rang + 1,
                    code: nu.chars().take(120).collect(),
                    motif,
                });
            }
        }
    }

    // **La cible n'est pas vide** (exigence 4). Une porte qui n'aurait aucun fichier portant la
    // colonne passerait au vert en n'ayant rien à inspecter — et c'est exactement l'état qu'elle
    // aurait eu si la convention de nommage avait changé sans que personne le dise.
    assert!(
        fichiers_portant_la_colonne >= 3,
        "seulement {fichiers_portant_la_colonne} fichier(s) portent « {COLONNE_CLIENT} ». La \
         colonne existe sur quatre tables depuis les cycles précédents : soit la convention de \
         nommage a changé, soit le périmètre découvert ne voit plus le code du produit. Dans les \
         deux cas, cette porte ne garde plus rien."
    );

    assert!(
        violations.is_empty(),
        "P-23 ÉCHOUE — {} usage(s) de l'horodatage CLIENT dans un calcul :\n{}\n\n\
         Seul l'horodatage d'AUTORITÉ serveur fait foi (principe IV). Un téléphone d'entrée de \
         gamme dérive, et le personnel change l'heure : une durée de passage calculée sur \
         l'horloge d'un terminal est une facture fausse, et une clôture qui s'y appuie est fausse \
         au franc près.\n\n\
         Les trois exemptions sont LIMITATIVEMENT énumérées par la constitution — ordre \
         d'affichage local, détection de dérive d'horloge, rendu de l'instant tel que le terminal \
         l'a perçu — et la liste est CLOSE. Écrire la colonne n'en fait pas partie, et n'en a pas \
         besoin : écrire une valeur n'est pas s'appuyer dessus.",
        violations.len(),
        violations
            .iter()
            .map(|v| format!("  · {}:{} — {} :\n      {}", v.fichier, v.ligne, v.motif, v.code))
            .collect::<Vec<_>>()
            .join("\n")
    );

    println!(
        "P-23 — {} fichier(s) inspecté(s) sur le périmètre découvert, {fichiers_portant_la_colonne} \
         portant « {COLONNE_CLIENT} », {} exempté(s) :",
        fichiers.len(),
        exemptes.len()
    );
    for exempte in &exemptes {
        println!("    {exempte}");
    }
}

/// **Les trois exemptions du script sont CELLES de la constitution, à la lettre.**
///
/// # Pourquoi ce contrôle existe
///
/// Une porte dont les exemptions dérivent de son texte fondateur devient une porte qui garde autre
/// chose que ce qu'elle annonce — et elle le devient un cycle à la fois, chacun ajoutant « juste
/// un cas ». La constitution nomme trois exemptions ; ce test les relit **dans la constitution**
/// et refuse qu'il y en ait davantage.
#[test]
fn les_exemptions_sont_celles_de_la_constitution_ni_plus_ni_moins() {
    const CONSTITUTION: &str = include_str!("../../.specify/memory/constitution.md");

    // **La ligne du TABLEAU des portes**, pas celle du rapport d'impact de la version. Les deux
    // nomment P-23 ; seule la première fait foi — le rapport d'impact est un historique, et il
    // resterait inchangé si le texte de la porte était amendé.
    let ligne_p23 = CONSTITUTION
        .lines()
        .find(|l| l.trim_start().starts_with("| P-23 |"))
        .expect(
            "la ligne de P-23 a disparu du TABLEAU des portes de la constitution. Cette porte n'a \
             plus de texte fondateur, et ses exemptions ne se comparent plus à rien.",
        );

    for exemption in [
        "ordre d'affichage local",
        "détection de dérive d'horloge",
        "rendu de l'instant tel que le terminal l'a perçu",
    ] {
        assert!(
            ligne_p23.contains(exemption),
            "l'exemption « {exemption} » a disparu du texte de P-23 dans la constitution.\n\
             Le script s'y adosse : si la constitution en retire une, le script doit la retirer \
             aussi, et l'inverse est vrai — une exemption ajoutée au script sans amendement \
             élargirait la porte de sa propre autorité."
        );
    }

    // **Le mot qui ferme la liste.** Sans lui, un cycle pourrait ajouter une exemption en
    // considérant que la liste était indicative.
    assert!(
        ligne_p23.contains("limitativement"),
        "le mot « limitativement » a disparu du texte de P-23. C'est lui qui ferme la liste des \
         exemptions ; sans lui, la couche de persistance ou n'importe quel autre cas s'y \
         ajouterait « par bon sens »."
    );

    // Et le script n'en porte **qu'une** : les deux autres ne demandent aucun fichier exempté.
    let module_de_derive = perimetre::fichier_du_crate(
        perimetre::Famille::Socle,
        "synchronisation",
        "src/derive.rs",
    );
    assert!(
        exemption(&module_de_derive).is_some(),
        "la détection de dérive n'est plus exemptée : la porte échouerait sur le module qui a \
         pour raison d'être de comparer les deux horodatages"
    );

    let couche_de_persistance = perimetre::fichier_du_crate(
        perimetre::Famille::Socle,
        "etablissements",
        "src/note/repository.rs",
    );
    assert!(
        exemption(&couche_de_persistance).is_none(),
        "la couche de PERSISTANCE est exemptée. Elle n'a pas à l'être, et la constitution ne le \
         permet pas : écrire une valeur n'est pas s'appuyer dessus. L'exempter élargirait la porte \
         de sa propre autorité — exactement ce que « limitativement » interdit."
    );
}

/// **Test négatif — la porte sait reconnaître les trois formes de faute.**
///
/// Un contrôle qui ne trouve rien peut être un contrôle qui fonctionne, ou un contrôle cassé. Rien
/// ne les distingue depuis le vert. La reconnaissance est donc exercée sur des fragments écrits
/// ici, sans toucher au dépôt — le précédent est `classes_offline.rs`, dont le test négatif simule
/// sa table fautive plutôt que de la créer.
#[test]
fn test_negatif_la_porte_reconnait_les_trois_formes() {
    // Forme 1 — une durée calculée sur l'horloge du terminal. C'est la facture fausse.
    assert!(
        usage_suspect("        let duree = fin - horodatage_client;").is_some(),
        "la porte ne reconnaît plus un calcul de durée : elle laisserait passer une durée de \
         passage tirée de l'horloge d'un téléphone"
    );

    // Forme 2 — un tri qui dépend de l'heure des terminaux qui ont alimenté le registre.
    assert!(
        usage_suspect(r#"        "SELECT * FROM notes ORDER BY horodatage_client DESC""#).is_some(),
        "la porte ne reconnaît plus un tri SQL sur la colonne indicative"
    );

    // Forme 3 — la plus discrète : l'autorité remplacée.
    assert!(
        usage_suspect("            cree_le: horodatage_client,").is_some(),
        "la porte ne reconnaît plus l'affectation de l'horodatage client à un champ d'autorité — \
         la faute qui ne calcule rien et qui remplace tout"
    );

    // ── Et elle ne se déclenche PAS sur ce qui est légitime ────────────────────────────────────
    //
    // Sans ce versant, la porte échouerait sur tout et serait désactivée dans la semaine. Chacune
    // de ces lignes est un geste réel du produit.
    for legitime in [
        "    pub horodatage_client: Option<OffsetDateTime>,",
        "        .bind(note.horodatage_client)",
        "            horodatage_client: corps.horodatage_client,",
        "        let persiste: OffsetDateTime = ligne.get(\"horodatage_client\");",
    ] {
        assert!(
            usage_suspect(legitime).is_none(),
            "la porte signale un usage LÉGITIME — déclarer, écrire ou relire la colonne ne fait \
             dépendre aucune décision de l'horloge d'un terminal :\n  {legitime}"
        );
    }
}
