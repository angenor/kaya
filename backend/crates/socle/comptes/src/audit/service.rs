//! Écriture au registre des actions — **la trace et l'opération tombent ou passent ensemble**.
//!
//! C'est la couche qui porte la garantie de FR-024, et elle repose sur la même mécanique que
//! `OutboxWriter::ecrire` du cycle 001 : une **signature**, pas une discipline.
//!
//! ```ignore
//! async fn tracer(&self, tx: &mut sqlx::PgTransaction<'_>, entree: EntreeAudit) -> …
//! ```
//!
//! `tracer` **prend la transaction et n'en ouvre jamais une**. Écrire l'entrée ailleurs
//! demanderait de fabriquer une seconde transaction et de la passer explicitement — ce qui se voit
//! en revue et ne s'écrit pas par distraction. Un trait qui prendrait un pool laisserait la
//! garantie reposer sur l'attention du développeur, et une attribution de rôle réussie sans sa
//! trace serait exactement le trou que CPT-04 doit fermer.

use serde_json::Value;
use sqlx::PgTransaction;
use uuid::Uuid;

use super::modele::{EntreeAudit, ErreurAudit};

/// Suffixe réservé des clés monétaires dans `contexte`.
///
/// Constitution **1.6.0** : le principe V cessait de tenir à la frontière du `JSONB`, sur le
/// registre même qui trace les écarts de caisse, les modifications de tarif et les remises.
pub const SUFFIXE_MONETAIRE: &str = "_mineur";

/// Clé qui doit accompagner tout montant, **au même niveau d'objet**.
pub const CLE_DEVISE: &str = "devise";

/// Noms qu'un montant ne peut pas porter nu dans un document d'audit.
///
/// `{"montant": 12500}` est refusé : le nombre de décimales viendrait de nulle part, et rien ne
/// dirait s'il s'agit d'unités ou de centimes. Le nommage réservé n'est pas de la coquetterie —
/// c'est la seule chose qui rende un entier JSON interprétable six mois plus tard.
pub const NOMS_MONETAIRES_NUS: &[&str] = &["montant", "prix", "total", "somme", "cout"];

/// Écriture d'une entrée d'audit **dans la transaction fournie**.
///
/// # Ce que la signature garantit
///
/// Voir le commentaire de tête. En un mot : la trace ne peut pas exister sans l'opération, ni
/// l'opération sans la trace.
///
/// # Pourquoi `#[async_trait]`
///
/// Rust sait écrire `async fn` dans un trait depuis 1.75, mais un tel trait n'est pas
/// dyn-compatible. L'injection de dépendances du cadrage §13.2 suppose `Arc<dyn JournalAudit>` :
/// l'annotation est un choix contraint, pas une habitude reprise d'un exemple.
///
/// # Aucune méthode de lecture ici
///
/// La lecture filtrée est un **repository**, appelé par le seul endpoint `journal_audit_lister`.
/// L'exposer sur ce trait mettrait le registre à portée de tout consommateur du socle, alors que
/// sa consultation est gardée par la permission `cpt.audit.consulter`.
#[async_trait::async_trait]
pub trait JournalAudit: Send + Sync {
    async fn tracer(
        &self,
        tx: &mut PgTransaction<'_>,
        tenant_id: Uuid,
        entree: EntreeAudit,
    ) -> Result<(), ErreurAudit>;
}

/// Implémentation PostgreSQL.
#[derive(Debug, Clone, Copy, Default)]
pub struct JournalAuditPostgres;

#[async_trait::async_trait]
impl JournalAudit for JournalAuditPostgres {
    async fn tracer(
        &self,
        tx: &mut PgTransaction<'_>,
        tenant_id: Uuid,
        entree: EntreeAudit,
    ) -> Result<(), ErreurAudit> {
        // La validation précède l'écriture : un contexte fautif ne doit pas atteindre un registre
        // immuable, où rien ne se corrige.
        valider_contexte(&entree.contexte)?;

        sqlx::query!(
            r#"
            INSERT INTO comptes.journal_audit
                (id, tenant_id, etablissement_id, type_action, auteur_compte_id,
                 cible_type, cible_id, contexte, horodatage_client)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (id) DO NOTHING
            "#,
            entree.id,
            tenant_id,
            entree.etablissement_id,
            entree.type_action.code(),
            entree.auteur_compte_id,
            entree.cible_type,
            entree.cible_id,
            entree.contexte,
            entree.horodatage_client,
        )
        .execute(&mut **tx)
        .await?;

        // `ON CONFLICT DO NOTHING` sans `RETURNING`, et sans que l'appelant sache s'il y a eu
        // insertion : contrairement à une note, une entrée d'audit n'a pas de code HTTP à rendre.
        // Le rejeu est inoffensif et silencieux, ce qui est exactement le comportement d'une
        // entité de classe A.
        Ok(())
    }
}

/// **Valide la convention monétaire du document `contexte`** — porte P-10 étendue.
///
/// # Pourquoi cette validation existe EN PLUS du contrôle statique
///
/// `scripts/ci/types-monetaires.sh` inspecte les littéraux du code source. Il ne voit pas un
/// document construit dynamiquement — `json!({ cle_calculee: valeur })` — et c'est la forme que
/// prendra le service de caisse le jour où il tracera un écart. Réciproquement, cette validation
/// ne voit pas un littéral mal nommé dans du code qui ne s'exécute pas encore.
///
/// **Les deux sont nécessaires, et aucun ne remplace l'autre.** Écrit ici pour qu'on ne croie pas
/// l'un suffisant.
///
/// # Les trois règles
///
/// 1. toute clé se terminant par `_mineur` porte un **entier** — jamais `12500.5`, jamais
///    `"12 500 F"` ;
/// 2. une clé `devise` l'accompagne **au même niveau d'objet** — le nombre de décimales vient de
///    la devise, jamais d'une constante (principe V) ;
/// 3. aucun montant ne se nomme `montant`, `prix`, `total`, `somme` ni `cout` **nu**.
///
/// La descente est **récursive** : un montant enfoui dans un sous-objet est un montant.
pub fn valider_contexte(contexte: &Value) -> Result<(), ErreurAudit> {
    valider_valeur(contexte, "contexte")
}

fn valider_valeur(valeur: &Value, chemin: &str) -> Result<(), ErreurAudit> {
    match valeur {
        Value::Object(objet) => {
            let porte_devise = objet.contains_key(CLE_DEVISE);

            for (cle, sous_valeur) in objet {
                let sous_chemin = format!("{chemin}.{cle}");

                if NOMS_MONETAIRES_NUS.contains(&cle.as_str()) {
                    return Err(ErreurAudit::ContexteInvalide(format!(
                        "« {sous_chemin} » nomme un montant sans dire son unité. Un montant \
                         s'écrit en entier d'unité mineure, sous une clé suffixée « {SUFFIXE_MONETAIRE} », \
                         accompagnée de « {CLE_DEVISE} » au même niveau : \
                         {{ \"{cle}{SUFFIXE_MONETAIRE}\": -12500, \"{CLE_DEVISE}\": \"XOF\" }}. \
                         Sans le suffixe, rien ne dit si 12500 est en unités ou en centimes, et \
                         le registre qui sert à prouver un écart ne prouve plus rien."
                    )));
                }

                if cle.ends_with(SUFFIXE_MONETAIRE) {
                    // `as_i64` échoue sur un flottant comme sur une chaîne — les deux formes que
                    // le principe V interdit, et les deux qu'un document JSON accepte sans
                    // broncher.
                    if sous_valeur.as_i64().is_none() {
                        return Err(ErreurAudit::ContexteInvalide(format!(
                            "« {sous_chemin} » vaut {sous_valeur}, qui n'est pas un entier. Un \
                             montant est un ENTIER d'unité mineure (principe V) : ni décimal, ni \
                             chaîne formatée. Un écart de caisse stocké en flottant, et l'audit \
                             ment sur le montant qu'il est censé prouver."
                        )));
                    }

                    if !porte_devise {
                        return Err(ErreurAudit::ContexteInvalide(format!(
                            "« {sous_chemin} » porte un montant sans clé « {CLE_DEVISE} » au même \
                             niveau d'objet. Le nombre de décimales vient de la DEVISE, jamais \
                             d'une constante : 12500 XOF vaut 12 500 F, 12500 EUR vaut 125,00 €."
                        )));
                    }
                }

                valider_valeur(sous_valeur, &sous_chemin)?;
            }
            Ok(())
        }
        Value::Array(elements) => {
            for (index, element) in elements.iter().enumerate() {
                valider_valeur(element, &format!("{chemin}[{index}]"))?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn refus(valeur: Value) -> String {
        match valider_contexte(&valeur) {
            Err(ErreurAudit::ContexteInvalide(motif)) => motif,
            Err(autre) => panic!("erreur inattendue : {autre}"),
            Ok(()) => panic!("ce contexte aurait dû être refusé : {valeur}"),
        }
    }

    /// **Le cas nominal de la constitution 1.6.0.**
    #[test]
    fn un_montant_entier_avec_sa_devise_est_accepte() {
        assert!(
            valider_contexte(&json!({
                "ecart_mineur": -12500,
                "devise": "XOF",
                "motif": "erreur de rendu monnaie"
            }))
            .is_ok()
        );
    }

    /// **Un flottant est refusé** — la forme que le `JSONB` accepte et que le principe V interdit.
    #[test]
    fn un_montant_decimal_est_refuse() {
        let motif = refus(json!({ "ecart_mineur": -12500.5, "devise": "XOF" }));
        assert!(motif.contains("ecart_mineur"), "motif : {motif}");
        assert!(motif.contains("ENTIER"), "motif : {motif}");
    }

    /// **Une chaîne formatée est refusée** — c'est la forme qui vient naturellement quand on
    /// construit un document depuis ce qu'affiche l'écran.
    #[test]
    fn un_montant_en_chaine_formatee_est_refuse() {
        let motif = refus(json!({ "ecart_mineur": "12 500 F", "devise": "XOF" }));
        assert!(motif.contains("ecart_mineur"), "motif : {motif}");
    }

    /// **Un montant sans devise est refusé.**
    #[test]
    fn un_montant_sans_devise_est_refuse() {
        let motif = refus(json!({ "ecart_mineur": -12500 }));
        assert!(motif.contains("devise"), "motif : {motif}");
    }

    /// **Un montant nu est refusé, même entier.**
    #[test]
    fn un_montant_nommement_nu_est_refuse_meme_entier() {
        for nom in NOMS_MONETAIRES_NUS {
            let motif = refus(json!({ *nom: 12500, "devise": "XOF" }));
            assert!(motif.contains(nom), "motif : {motif}");
        }
    }

    /// **La descente est récursive** : un montant enfoui reste un montant.
    #[test]
    fn un_montant_enfoui_dans_un_sous_objet_est_inspecte() {
        let motif = refus(json!({
            "devise": "XOF",
            "avant": { "prix_mineur": 1500.0, "devise": "XOF" }
        }));
        assert!(motif.contains("contexte.avant.prix_mineur"), "motif : {motif}");

        // Et dans un tableau.
        let motif = refus(json!({
            "lignes": [ { "remise_mineur": 500 } ]
        }));
        assert!(motif.contains("contexte.lignes[0].remise_mineur"), "motif : {motif}");
    }

    /// **La devise se cherche au MÊME niveau d'objet, pas dans un parent.**
    ///
    /// Une devise héritée d'un niveau supérieur serait presque toujours juste, et fausse le jour
    /// où un document mêlerait deux devises — ce qui arrive dès qu'un établissement change de
    /// devise ou qu'un rapport agrège deux tenants. Le presque-toujours-juste est le pire des
    /// régimes : il ne se manifeste jamais en test.
    #[test]
    fn la_devise_ne_s_herite_pas_d_un_niveau_superieur() {
        let motif = refus(json!({
            "devise": "XOF",
            "detail": { "ecart_mineur": -12500 }
        }));
        assert!(motif.contains("contexte.detail.ecart_mineur"), "motif : {motif}");
    }

    /// Un contexte sans aucun montant passe — c'est le cas de tout ce que ce cycle écrit.
    #[test]
    fn un_contexte_sans_montant_est_accepte() {
        assert!(valider_contexte(&json!({})).is_ok());
        assert!(
            valider_contexte(&json!({
                "role_code": "caissier",
                "etablissement_id": "018f-…",
                "sens": "attribution"
            }))
            .is_ok()
        );
    }
}
