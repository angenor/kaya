//! **La taxonomie du registre des actions — une énumération FERMÉE.**
//!
//! # Pourquoi ce n'est pas un `String`
//!
//! Un `String` marcherait, et c'est exactement le problème. Il laisserait un cycle écrire
//! `remise_appliquee` là où un autre a écrit `remise`, sans qu'aucune compilation ne le signale.
//! Le registre porterait alors les deux, et le filtre par type de l'écran `G4` cesserait de
//! trouver la moitié des entrées — **sans erreur, sans page vide, sans rien qui alerte** : la
//! liste serait simplement plus courte que la réalité, ce que personne ne peut constater.
//!
//! Une énumération fermée déplace ce défaut au moment de la compilation. Et parce que la table
//! `journal_audit` porte du `TEXT` sans `CHECK ... IN`, c'est **ce type-ci** qui la ferme :
//! `type_action` ne s'alimente jamais depuis un `String` d'appelant.
//!
//! # Le harnais, et ce qu'il vérifie dans les deux sens
//!
//! `docs/taxonomie-audit.md` déclare **onze** familles, chacune avec son état — `branché` ou
//! `dû` —
//! et la story qui la doit. `backend/tests/audit_taxonomie.rs` compare :
//!
//!   * **code → document** : toute variante d'ici figure au document ;
//!   * **état → réalité** : un type déclaré `dû` n'a aucun chemin d'écriture, un type déclaré
//!     `branché` en a un.
//!
//! Le second sens est celui qui travaille. Le jour où PDV-03 écrit une remise, le build échoue
//! **avant** la revue, et la ligne passe à `branché` dans le même changement.
//!
//! # Ce que ce fichier n'est PAS
//!
//! Ce n'est pas la liste des types d'événements outbox. Deux registres, deux publics, deux
//! classes (research R-08) : une attribution de rôle produit `role.attribue` **et**
//! `changement_role`, dans la même transaction, et ce n'est pas une redondance — l'un alimente
//! les projections, l'autre est un produit que le propriétaire achète.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Les **onze** familles d'actions tracées au registre — dix de CPT-04, une de SYN-04.
///
/// L'ordre des variantes suit celui de `docs/taxonomie-audit.md`, qui suit lui-même celui de
/// CPT-04. Il n'a aucune portée fonctionnelle — il rend seulement la comparaison des deux listes
/// lisible à l'œil.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum TypeActionAudit {
    /// Une remise accordée sur une ligne ou une note. **Dû par PDV-03 / SEJ-03.**
    Remise,
    /// L'annulation d'une ligne **déjà partie en cuisine ou au bar**. **Dû par PDV-03.**
    AnnulationLigneEnvoyee,
    /// L'émission d'un avoir sur une facture certifiée. **Dû par FIS-06.**
    Avoir,
    /// Une ouverture de tiroir-caisse hors encaissement. **Dû par IMP-01.**
    OuvertureTiroir,
    /// Le changement du prix d'un article vendable. **Dû par PDV-01.**
    ModificationTarif,
    /// La mise hors service de ce qui ne se supprime jamais.
    ///
    /// **Le mot est faux et gardé quand même** : rien ne se supprime dans Kaya (FR-014), un
    /// compte se désactive, un service se retire. Mais c'est le geste que l'utilisateur croit
    /// faire, et le registre est lu par un propriétaire qui cherche « qui a supprimé ça ». Le
    /// lexique traduit ; la taxonomie nomme l'intention.
    ///
    /// Au cycle 003 : la **désactivation d'un compte**.
    Suppression,
    /// Une attribution ou un retrait de rôle. **Deux actes distincts**, deux entrées — c'est la
    /// raison pour laquelle `compte_role` n'a pas de privilège `UPDATE`.
    ChangementRole,
    /// Un écart constaté au comptage de fin de shift. **Dû par CAI-04.**
    EcartCaisse,
    /// Le passage automatique au palier tarifaire supérieur. **Dû par HEB-04.**
    RebasculePalierPassage,
    /// L'attribution d'une unité que le système déclarait indisponible. **Dû par HEB.**
    ForcageDisponibilite,
    /// L'heure d'un terminal s'écarte de celle du serveur au-delà du seuil. **SYN-04.**
    ///
    /// **La première famille qui ne trace aucun geste d'utilisateur** : elle constate un état de
    /// l'appareil, pas une décision d'une personne. Elle a sa place ici parce que son public est
    /// celui des dix autres — l'exploitant, qui doit pouvoir retrouver après coup quel terminal
    /// déviait pendant le service.
    ///
    /// **Elle ne refuse jamais l'écriture qui l'a révélée** (FR-036), et elle est écrite **une
    /// fois par épisode**, non une fois par écriture : deux cents saisies pendant un service
    /// produiraient deux cents entrées identiques, et un registre illisible n'est plus lu.
    DeriveHorlogeConstatee,
}

impl TypeActionAudit {
    /// Le code stocké en base et rendu par l'API — celui de `docs/taxonomie-audit.md`.
    ///
    /// **Écrit à la main plutôt que dérivé de `serde`.** La sérialisation sert au contrat HTTP ;
    /// cette fonction sert à la colonne `type_action`. Les faire dépendre l'une de l'autre
    /// signifierait qu'un changement de représentation JSON réécrit silencieusement des données
    /// déjà en base — dans un registre immuable, où aucune correction n'est possible.
    pub fn code(self) -> &'static str {
        match self {
            TypeActionAudit::Remise => "remise",
            TypeActionAudit::AnnulationLigneEnvoyee => "annulation_ligne_envoyee",
            TypeActionAudit::Avoir => "avoir",
            TypeActionAudit::OuvertureTiroir => "ouverture_tiroir",
            TypeActionAudit::ModificationTarif => "modification_tarif",
            TypeActionAudit::Suppression => "suppression",
            TypeActionAudit::ChangementRole => "changement_role",
            TypeActionAudit::EcartCaisse => "ecart_caisse",
            TypeActionAudit::RebasculePalierPassage => "rebascule_palier_passage",
            TypeActionAudit::ForcageDisponibilite => "forcage_disponibilite",
            TypeActionAudit::DeriveHorlogeConstatee => "derive_horloge_constatee",
        }
    }

    /// Reconstruit le type depuis le code lu en base.
    ///
    /// Rend `None` sur un code inconnu plutôt que de paniquer : une ligne écrite par une version
    /// ultérieure du produit — cas réel en mode auto-hébergé, où les binaires ne sont pas tous à
    /// jour au même instant — ne doit pas faire tomber la lecture du registre entier.
    pub fn depuis_code(code: &str) -> Option<Self> {
        TypeActionAudit::TOUS
            .iter()
            .copied()
            .find(|t| t.code() == code)
    }

    /// Les onze, dans l'ordre du document. Employé par le filtre de `G4` et par le harnais.
    pub const TOUS: [TypeActionAudit; 11] = [
        TypeActionAudit::Remise,
        TypeActionAudit::AnnulationLigneEnvoyee,
        TypeActionAudit::Avoir,
        TypeActionAudit::OuvertureTiroir,
        TypeActionAudit::ModificationTarif,
        TypeActionAudit::Suppression,
        TypeActionAudit::ChangementRole,
        TypeActionAudit::EcartCaisse,
        TypeActionAudit::RebasculePalierPassage,
        TypeActionAudit::ForcageDisponibilite,
        TypeActionAudit::DeriveHorlogeConstatee,
    ];
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Les codes sont **stables** : ils sont écrits en base, dans un registre immuable.
    ///
    /// Ce test n'est pas une tautologie. Il fige la chaîne exacte de chaque variante, de sorte
    /// qu'un renommage de variante — geste anodin, que tout outil de remaniement propose — ne
    /// change pas silencieusement ce qui part en base. Les entrées déjà écrites porteraient
    /// l'ancien code, les nouvelles le nouveau, et le filtre de `G4` n'en trouverait plus que la
    /// moitié.
    #[test]
    fn les_codes_sont_ceux_du_document_et_ne_bougent_pas() {
        let attendus = [
            "remise",
            "annulation_ligne_envoyee",
            "avoir",
            "ouverture_tiroir",
            "modification_tarif",
            "suppression",
            "changement_role",
            "ecart_caisse",
            "rebascule_palier_passage",
            "forcage_disponibilite",
            "derive_horloge_constatee",
        ];

        let reels: Vec<&str> = TypeActionAudit::TOUS.iter().map(|t| t.code()).collect();
        assert_eq!(reels, attendus);
    }

    /// `TOUS` couvre l'énumération entière.
    ///
    /// Une variante ajoutée sans être portée à `TOUS` serait invisible du filtre de `G4` et du
    /// harnais : elle s'écrirait en base et ne se relirait jamais. Le `match` exhaustif de
    /// `code()` force le compilateur à signaler l'ajout ; ce test force à compléter `TOUS`.
    #[test]
    fn tous_couvre_l_enumeration_entiere() {
        assert_eq!(
            TypeActionAudit::TOUS.len(),
            11,
            "une famille a été ajoutée ou retirée sans que `TOUS` suive"
        );

        for type_action in TypeActionAudit::TOUS {
            assert_eq!(
                TypeActionAudit::depuis_code(type_action.code()),
                Some(type_action),
                "« {} » ne se relit pas depuis son propre code",
                type_action.code()
            );
        }
    }

    /// Un code inconnu ne fait pas tomber la lecture.
    #[test]
    fn un_code_inconnu_rend_none_plutot_que_de_paniquer() {
        assert_eq!(TypeActionAudit::depuis_code("remise_appliquee"), None);
        assert_eq!(TypeActionAudit::depuis_code(""), None);
    }
}
