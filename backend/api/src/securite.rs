//! La garde de permission des handlers — **et l'aveu qu'elle ne devrait jamais se déclencher**.
//!
//! # Ce code existe pour l'appel direct, pas pour le parcours normal
//!
//! Le principe VII et FR-026 sont sans ambiguïté : une action que l'utilisateur n'a pas le droit
//! de faire est **absente** de l'interface, pas refusée. Un utilisateur qui arrive ici a donc
//! contourné l'interface — outil en ligne de commande, appel forgé —, ou bien ses droits ont
//! changé pendant qu'il regardait son écran, ce qui est le seul chemin légitime.
//!
//! **Ce n'est pas une raison de s'en passer.** L'interface décide de ce qu'elle montre ; le
//! serveur décide de ce qu'il fait. Confondre les deux, c'est faire dépendre l'autorisation du
//! client, et le client est un binaire décompilable installé sur le téléphone de quelqu'un.
//!
//! # Une garde, un endroit
//!
//! Les services de `socle/` ne vérifient **aucune** permission — c'est écrit dans
//! `roles/service.rs`. Mêler la garde au service obligerait à passer les permissions de l'appelant
//! à chaque service du produit, et le jour où l'un d'eux oublierait de les consulter, rien ne le
//! signalerait. Une garde à un seul endroit se relit ; une garde dispersée se contourne.
//!
//! # Pourquoi les permissions viennent du jeton, et ce que ça coûte
//!
//! `ContexteAppel` les porte, extraites du jeton signé à la connexion (research R-06). Aucune
//! lecture SQL n'a lieu ici : la garde est une comparaison de chaînes sur une liste de dix-sept
//! éléments au plus.
//!
//! La contrepartie est écrite au plan et vaut d'être répétée : **un rôle retiré prend effet au
//! rafraîchissement suivant**, soit au plus soixante minutes. Ce délai vaut pour un ajustement de
//! droits, jamais pour un départ ou un vol — la **révocation de session**, elle, est immédiate, et
//! c'est elle qu'on emploie quand quelqu'un s'en va.
//!
//! # Ce que ce fichier n'est pas
//!
//! Ce n'est pas `AccessController`. Le trait lit les droits **en base**, pour un compte
//! quelconque, et sert aux consommateurs du socle. Cette garde-ci lit les droits **de l'appelant**,
//! dans son jeton, et sert aux handlers. Les faire passer par le trait ajouterait deux requêtes
//! SQL à chaque opération du produit pour une information qu'on a déjà en main.

use crate::contexte::ContexteAppel;
use crate::routes::erreurs::CorpsErreur;

/// Le code de refus, écrit une seule fois.
///
/// C'est sur lui que le front branche sa clé i18n ; le recopier dans chaque handler produirait le
/// jour où quelqu'un écrirait `permissions_absentes` un refus que l'interface ne saurait pas
/// traduire — et elle afficherait une phrase générique sans que rien n'échoue.
pub const CODE_PERMISSION_ABSENTE: &str = "permission_absente";

/// Exige une permission de l'appelant, ou rend `403 permission_absente`.
///
/// # La permission refusée est nommée dans `valeur`, et c'est sans danger
///
/// `CorpsErreur.valeur` porte le code de la permission manquante. Le lui cacher n'apporterait
/// rien : l'appelant sait quelle opération il a tentée, et le catalogue des permissions est un
/// **référentiel public** que `referentiel_permissions` rend à tout compte authentifié. En
/// revanche, la nommer rend les journaux exploitables — « qui a tenté quoi » plutôt que « un refus
/// quelque part ».
///
/// Le `message`, lui, reste un diagnostic : il n'est jamais affiché (voir `routes/erreurs.rs`).
pub fn exiger(contexte: &ContexteAppel, permission: &str) -> Result<(), actix_web::Error> {
    if contexte.detient(permission) {
        return Ok(());
    }

    tracing::info!(
        compte.id = %contexte.compte_id,
        tenant.id = %contexte.tenant_id,
        permission = permission,
        "refus de permission — l'interface ne devrait pas avoir proposé cette action"
    );

    Err(CorpsErreur::nouveau(
        CODE_PERMISSION_ABSENTE,
        Some(permission.to_owned()),
        format!("le compte appelant ne détient pas « {permission} »"),
    )
    .en_403())
}

/// Exige **l'une** des permissions, ou d'agir sur soi-même.
///
/// # Le patron « ou soi », et pourquoi il mérite une fonction
///
/// Trois opérations du contrat le portent : `compte_changer_mot_de_passe`, `session_revoquer` et,
/// par extension, toute action qu'un compte exerce sur son propre compte. Le contrat les note
/// « `cpt.compte.gerer` **ou** soi ».
///
/// Écrire l'alternative à la main dans chaque handler la ferait écrire trois fois, et la
/// troisième finirait par tester l'égalité dans le mauvais sens — comparer la **cible** à
/// l'appelant, ce qui est juste, contre comparer l'appelant à lui-même, ce qui est toujours vrai
/// et n'interdit rien.
///
/// L'ordre des deux conditions n'est pas indifférent : **« soi » est testé d'abord**, parce que
/// c'est le cas de très loin le plus fréquent et le seul qui ne mérite aucune trace.
pub fn exiger_ou_soi(
    contexte: &ContexteAppel,
    permission: &str,
    compte_cible: uuid::Uuid,
) -> Result<(), actix_web::Error> {
    if contexte.compte_id == compte_cible {
        return Ok(());
    }
    exiger(contexte, permission)
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn appelant(permissions: &[&str]) -> ContexteAppel {
        ContexteAppel {
            tenant_id: Uuid::now_v7(),
            compte_id: Uuid::now_v7(),
            session_id: Uuid::now_v7(),
            etablissement_actif: None,
            permissions: permissions.iter().map(|p| (*p).to_owned()).collect(),
        }
    }

    #[test]
    fn une_permission_detenue_laisse_passer() {
        let contexte = appelant(&["cpt.role.attribuer", "cpt.compte.lire"]);
        assert!(exiger(&contexte, "cpt.role.attribuer").is_ok());
    }

    #[test]
    fn une_permission_absente_refuse() {
        let contexte = appelant(&["cpt.compte.lire"]);
        assert!(exiger(&contexte, "cpt.role.attribuer").is_err());
    }

    /// **Aucune correspondance par préfixe.** `cpt.compte.lire` n'ouvre pas `cpt.compte.gerer`.
    ///
    /// La faute serait tentante — une hiérarchie de permissions par préfixe paraît élégante — et
    /// elle rendrait `cpt.` équivalent à tout. Les permissions sont des codes opaques comparés
    /// par égalité, et rien d'autre.
    #[test]
    fn la_comparaison_est_une_egalite_et_jamais_un_prefixe() {
        let contexte = appelant(&["cpt.compte.lire"]);
        assert!(exiger(&contexte, "cpt.compte.gerer").is_err());
        assert!(exiger(&contexte, "cpt.compte").is_err());
        assert!(exiger(&contexte, "cpt.compte.lire.tout").is_err());
    }

    /// Un compte sans aucune permission est refusé partout — et il **existe** : c'est le compte
    /// fraîchement créé, avant qu'on lui donne un rôle.
    #[test]
    fn un_compte_sans_permission_est_refuse_sans_planter() {
        let contexte = appelant(&[]);
        assert!(exiger(&contexte, "cpt.compte.lire").is_err());
    }

    #[test]
    fn agir_sur_soi_ne_demande_aucune_permission() {
        let contexte = appelant(&[]);
        let soi = contexte.compte_id;
        assert!(exiger_ou_soi(&contexte, "cpt.compte.gerer", soi).is_ok());
    }

    /// **Agir sur un autre exige la permission**, et c'est le sens de la comparaison qui le tient.
    ///
    /// Le comparer à l'envers — l'appelant à lui-même — serait toujours vrai et n'interdirait
    /// rien. Ce test échoue si quelqu'un l'écrit dans ce sens.
    #[test]
    fn agir_sur_un_autre_exige_la_permission() {
        let contexte = appelant(&[]);
        assert!(exiger_ou_soi(&contexte, "cpt.compte.gerer", Uuid::now_v7()).is_err());

        let habilite = appelant(&["cpt.compte.gerer"]);
        assert!(exiger_ou_soi(&habilite, "cpt.compte.gerer", Uuid::now_v7()).is_ok());
    }
}
