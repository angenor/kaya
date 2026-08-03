//! **SC-009 — la clôture au franc près malgré une coupure.** *Porte installée à VIDE.*
//!
//! ═══════════════════════════════════════════════════════════════════════════════════════════
//!  CE QUE CE FICHIER GARDE, ET POURQUOI IL EST VIDE
//! ═══════════════════════════════════════════════════════════════════════════════════════════
//!
//! `docs/registre-classes-offline.md` §11 impose **deux tests transverses permanents**, et le
//! premier est celui-ci :
//!
//! > **Réseau coupé puis rétabli** au milieu d'une journée d'exploitation simulée — la clôture
//! > journalière tombe **au franc près** (SYN-04).
//!
//! **La clôture journalière n'existe pas.** Elle est de la tranche T3 (CAI-06), et rien de ce que
//! le cycle 005 livre ne la rapproche. Écrire ce test contre un mécanisme absent produirait soit
//! un test qui ne compile pas, soit — bien pire — un test qui passe en n'exerçant rien.
//!
//! # Alors pourquoi le fichier existe-t-il ?
//!
//! Pour la même raison que `portes_a_vide.rs` existe depuis le cycle 001, et la raison est écrite
//! dans la constitution : *une liste de choses à faire qui vit dans une spécification se perd ; une
//! liste qui vit dans un harnais de test ne se perd pas*.
//!
//! Le cycle qui livrera la clôture trouvera ce fichier **rouge**, avec le scénario à écrire dans le
//! message d'échec. Il n'aura pas à retrouver, dans un document, qu'une exigence transverse de deux
//! ans le concerne.
//!
//! # L'assertion de non-régression — ce qui empêche ce fichier d'être décoratif
//!
//! Un test vide qui passerait toujours serait une case cochée sans contrepartie. Celui-ci **échoue
//! dès que sa cible apparaît** : il cherche les signes d'une clôture journalière dans le produit —
//! une table, un service, un endpoint — et refuse dès qu'il en trouve un.
//!
//! C'est le patron exact de `portes_a_vide.rs`, et il a déjà fonctionné deux fois : P-06 a été
//! levée au cycle 002, P-09 au cycle 004, chacune parce que sa cible était apparue.

mod commun;

use commun::perimetre;

/// Les signes d'une clôture journalière dans le produit.
///
/// **Des noms d'entité, pas des mots isolés.** Chercher « cloture » seul échouerait sur la clôture
/// d'un exercice comptable — qui existe déjà en provision — et sur la clôture d'une table de
/// restaurant, qui n'a rien à voir. Ce qu'on cherche est la **journée d'exploitation** : le geste
/// de fin de service qui arrête les compteurs et produit le récapitulatif de caisse.
const SIGNES_DE_CLOTURE: &[&str] = &[
    "cloture_journaliere",
    "ClotureJournaliere",
    "journee_exploitation",
    "JourneeExploitation",
];

/// **La clôture journalière n'existe pas encore — et ce test le constate.**
///
/// # Ce qu'il faudra écrire quand elle existera
///
/// Le scénario est dans le message d'échec plutôt que dans un commentaire : c'est là qu'on le lira,
/// au moment exact où il devient pertinent.
#[test]
fn sc009_la_cloture_avec_coupure_reste_a_ecrire() {
    let fichiers = perimetre::sources_des_crates_metier();
    let racine = perimetre::racine_backend();

    assert!(
        fichiers.len() > 50,
        "seulement {} fichier(s) inspecté(s) : le périmètre découvert est vide, et cette porte \
         resterait verte quoi qu'il arrive.",
        fichiers.len()
    );

    let mut trouves = Vec::new();
    for fichier in &fichiers {
        let Ok(contenu) = std::fs::read_to_string(fichier) else {
            continue;
        };
        for signe in SIGNES_DE_CLOTURE {
            if contenu.contains(signe) {
                let chemin = fichier
                    .strip_prefix(&racine)
                    .unwrap_or(fichier)
                    .display()
                    .to_string();
                trouves.push(format!("{chemin} — « {signe} »"));
            }
        }
    }

    assert!(
        trouves.is_empty(),
        "SC-009 — la clôture journalière est apparue dans le produit :\n  {}\n\n\
         ═══════════════════════════════════════════════════════════════════════════════════\n\
         CE TEST DOIT MAINTENANT ÊTRE ÉCRIT. Voici le scénario, mot pour mot :\n\
         ═══════════════════════════════════════════════════════════════════════════════════\n\
         \n\
         1. Ouvrir une journée d'exploitation sur un établissement de démonstration.\n\
         2. Enregistrer une suite d'écritures EN LIGNE — ventes, encaissements, séjours.\n\
         3. **Couper le réseau.** Enregistrer des écritures de classe A sur le terminal :\n\
            elles entrent en file locale, chiffrées, et n'atteignent pas le serveur.\n\
         4. Rétablir le réseau. La file se vide — rafraîchissement AVANT envoi (R-18).\n\
         5. Clôturer la journée.\n\
         \n\
         **CE QUI DOIT ÊTRE VRAI, et qui est l'objet du test :**\n\
         \n\
           · le total de clôture est identique, AU FRANC PRÈS, à celui d'une journée\n\
             équivalente sans coupure — même jeu d'écritures, aucun réseau coupé ;\n\
           · les écritures différées portent l'horodatage d'AUTORITÉ de leur arrivée réelle,\n\
             jamais celui du terminal (principe IV, porte P-23) ;\n\
           · une écriture arrivée APRÈS la clôture ne modifie pas le total : elle produit un\n\
             constat de réconciliation orpheline (`synchronisation.reconciliation_orpheline`,\n\
             migration 0027), dont la résolution est HUMAINE et obligatoire (cadrage §11.4).\n\
         \n\
         **Le troisième point est celui qu'on écrirait mal.** Une clôture qui accepterait une\n\
         écriture en retard « puisqu'elle appartient à la journée » modifierait un total déjà\n\
         imprimé et remis au client. C'est précisément le conflit que la table de réconciliation\n\
         existe pour rendre visible plutôt que résoudre d'office.\n\
         \n\
         Retirer ensuite ce test-ci et le remplacer par le vrai : une porte à vide qui reste à\n\
         vide alors que sa cible existe est pire qu'une porte absente.",
        trouves.join("\n  ")
    );

    println!(
        "SC-009 — installée à vide. {} fichier(s) inspecté(s), aucun signe de clôture \
         journalière. La cible est de la tranche T3 (CAI-06) ; ce test échouera dès qu'elle \
         apparaîtra, avec le scénario à écrire dans son message.",
        fichiers.len()
    );
}

/// **La provision de réconciliation, elle, existe déjà** — et c'est ce qui rend le point 3 du
/// scénario écrivable le jour venu.
///
/// Sans cette assertion, la table pourrait disparaître d'ici la tranche T3 et le scénario
/// ci-dessus deviendrait inapplicable sans que rien ne le dise.
#[tokio::test]
async fn la_table_de_reconciliation_attend_deja_la_cloture() {
    let pool = commun::pool_owner().await;

    let existe: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS (
            SELECT 1 FROM information_schema.tables
            WHERE table_schema = 'synchronisation'
              AND table_name = 'reconciliation_orpheline'
        )
        "#,
    )
    .fetch_one(&pool)
    .await
    .expect("lecture du catalogue");

    assert!(
        existe,
        "`synchronisation.reconciliation_orpheline` a disparu. Le troisième point du scénario \
         SC-009 — une écriture arrivée APRÈS la clôture ne modifie pas le total — n'aurait plus \
         où s'écrire, et le conflit que le cadrage §11.4 nomme « le plus fréquent en exploitation \
         réelle » se résoudrait d'office, en silence."
    );
}
