//! **La dérive d'horloge — constatée, jamais opposée.**
//!
//! # Le fait du terrain, et pourquoi il ne peut pas être ignoré
//!
//! Le cadrage §11.4 est explicite : « un téléphone d'entrée de gamme dérive et le personnel change
//! l'heure ». Ce n'est pas une hypothèse de conception, c'est ce que fait un exploitant qui trouve
//! l'affichage faux. Et « le passage aggrave la sensibilité à l'horloge », puisqu'il se facture à
//! l'heure.
//!
//! Le principe IV en tire la règle : **toute durée, toute taxe et toute clôture partent de
//! l'horodatage d'autorité serveur**, jamais de l'horloge d'un terminal. La porte **P-23** la garde
//! désormais.
//!
//! Reste ce que la règle ne dit pas : **que faire quand on constate qu'une horloge est fausse ?**
//! Deux réponses sont possibles, et une seule est tenable.
//!
//! | Réponse | Ce qu'elle coûte |
//! |---|---|
//! | Refuser l'écriture | Une serveuse dont le téléphone retarde de dix minutes ne peut plus rien saisir. Le produit devient inutilisable pour une raison qu'elle ne peut pas corriger |
//! | **Accepter et signaler** | L'écriture passe, l'exploitant peut constater après coup quel terminal déviait |
//!
//! **La dérive n'est JAMAIS un motif de refus** (FR-036). C'est la décision la plus importante de
//! ce module, et elle est structurelle : cette fonction rend une `Option`, pas un `Result`. Il n'y
//! a rien à propager, rien à `?`, rien qui puisse remonter en erreur — le type interdit l'usage
//! qu'on voudrait éviter.
//!
//! # La valeur ABSOLUE, et les deux sens qui en découlent
//!
//! Une horloge **en avance** est aussi fausse qu'une horloge **en retard**. Comparer sur un écart
//! signé aurait laissé passer la moitié des cas, et le lexique donne bien deux formulations — «
//! retarde de {n} minutes » et « avance de {n} minutes » — parce que l'utilisateur doit savoir dans
//! quel sens régler son appareil.
//!
//! # Où vit ce module, et pourquoi il n'a AUCUNE dépendance
//!
//! `socle/synchronisation` est le crate le plus bas de la hiérarchie. `JournalAudit` vit dans
//! `socle/comptes`, **qui dépend de lui** :
//!
//! ```text
//! socle/synchronisation  ←  socle/etablissements  ←  socle/comptes
//!    (outbox, dérive)         (tenant_context)        (JournalAudit)
//! ```
//!
//! Faire écrire l'audit **par** ce module créerait un cycle de dépendances — refusé par le
//! compilateur, et par la porte P-03 avant lui. Ce module expose donc [`constater_derive`] (sans
//! aucune dépendance) et le trait [`SignalDerive`] ; **la couche API, qui connaît tout le monde,
//! câble l'un sur l'autre**. C'est le montage déjà éprouvé d'`OutboxWriter`.

use std::time::Duration;

use time::OffsetDateTime;
use uuid::Uuid;

/// De quel côté l'horloge du terminal se trompe.
///
/// **Les deux valeurs sont dues** : la détection porte sur la valeur absolue de l'écart, et une
/// seule forme laisserait la moitié des cas sans phrase à l'écran.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensDerive {
    /// L'horloge du terminal est **en retard** sur celle du serveur.
    Retard,
    /// L'horloge du terminal est **en avance** sur celle du serveur.
    Avance,
}

impl SensDerive {
    /// Le code écrit au contexte de l'entrée d'audit — **minuscules françaises**, comme toute
    /// valeur d'énumération persistée du produit.
    pub fn code(self) -> &'static str {
        match self {
            SensDerive::Retard => "retard",
            SensDerive::Avance => "avance",
        }
    }
}

/// Un écart constaté entre l'horloge d'un terminal et celle du serveur.
///
/// **Aucune valeur monétaire**, et c'est vérifié par la porte P-10 jusque dans le JSONB du
/// registre : cette structure décrit un temps, pas un montant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Derive {
    /// L'écart, en **secondes**, toujours positif.
    pub ecart_secondes: u64,
    /// Le seuil qui a été dépassé, en secondes — reproduit pour que l'entrée d'audit se lise seule.
    pub seuil_secondes: u64,
    pub sens: SensDerive,
}

impl Derive {
    /// L'écart en **minutes**, arrondi au plus proche — ce que l'écran affiche.
    ///
    /// L'arrondi se fait ici et non à l'affichage : « votre horloge avance de 5 minutes » et
    /// « de 4,7 minutes » disent la même chose à l'exploitant, et la seconde forme donne à croire
    /// à une précision que la mesure n'a pas.
    pub fn ecart_minutes(self) -> u64 {
        (self.ecart_secondes + 30) / 60
    }
}

/// **Constate une dérive.** Fonction pure : aucune entrée-sortie, aucune horloge lue.
///
/// # Pourquoi les deux instants sont des PARAMÈTRES
///
/// Lire `OffsetDateTime::now_utc()` ici rendrait la fonction intestable — il faudrait figer le
/// temps du processus — et surtout ferait décider l'horloge du **processus**, alors que ce qui
/// fait autorité est l'horodatage que la base a posé (`cree_le`, `DEFAULT now()`). Les deux
/// diffèrent : un serveur d'application et sa base ne sont pas la même machine.
///
/// # Rendre `None` sous le seuil n'est pas « rien constater »
///
/// C'est constater qu'il n'y a **rien à signaler**. Un écart de trois secondes existe toujours —
/// deux horloges ne sont jamais exactement d'accord — et l'écrire au registre à chaque saisie
/// rendrait celui-ci illisible, donc inutilisé.
///
/// @param horodatage_client L'instant tel que le terminal l'a perçu. **Indicatif** : il ne décide
///   de rien, et c'est précisément ce qui permet de le comparer sans risque.
/// @param autorite L'horodatage d'autorité serveur — `cree_le` de la ligne écrite.
/// @param seuil Le seuil paramétré (`sync.derive_horloge_seuil_secondes`, défaut 300).
pub fn constater_derive(
    horodatage_client: OffsetDateTime,
    autorite: OffsetDateTime,
    seuil: Duration,
) -> Option<Derive> {
    let ecart = autorite - horodatage_client;

    // **La valeur absolue, et rien d'autre.** Une horloge en avance est aussi fausse qu'une
    // horloge en retard ; comparer sur l'écart signé laisserait passer la moitié des cas.
    let ecart_secondes = ecart.whole_seconds().unsigned_abs();
    let seuil_secondes = seuil.as_secs();

    if ecart_secondes <= seuil_secondes {
        return None;
    }

    // `autorite - client > 0` veut dire que le client est **avant** le serveur : il retarde.
    let sens = if ecart.is_positive() {
        SensDerive::Retard
    } else {
        SensDerive::Avance
    };

    Some(Derive {
        ecart_secondes,
        seuil_secondes,
        sens,
    })
}

/// Qui l'écriture d'une dérive concerne — **un terminal, pas une personne**.
///
/// Le triplet sert deux choses distinctes : il compose le contexte de l'entrée d'audit, et il est
/// la clé du débrayage par épisode. C'est l'appareil qui dévie ; le compte et le tenant disent où
/// le chercher.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrigineDerive {
    pub tenant_id: Uuid,
    pub compte_id: Uuid,
    /// L'appareil, quand il est identifiable. `None` tant que l'enrôlement (CPT-05) n'existe pas —
    /// le débrayage retombe alors sur le couple `(tenant, compte)`, qui est plus large et suffit.
    pub appareil_id: Option<Uuid>,
}

/// **Le canal de signalement** — implémenté par la couche qui connaît le registre des actions.
///
/// # Pourquoi un trait, et pourquoi il est ici plutôt que chez l'appelant
///
/// Ce module ne peut pas écrire au registre : `JournalAudit` vit dans `socle/comptes`, qui dépend
/// de lui, et l'inverse serait un cycle. Mais il ne peut pas non plus rendre la dérive à
/// l'appelant en le laissant décider quoi en faire — ce serait multiplier les traitements, et le
/// débrayage par épisode se réimplémenterait à chaque service.
///
/// Le trait est la troisième voie, et c'est le montage déjà éprouvé d'`OutboxWriter` : **le contrat
/// est ici, l'implémentation chez celui qui a l'information**.
///
/// # Il ne rend PAS de `Result`, et c'est le même raisonnement qu'ailleurs
///
/// Un signalement qui échoue ne doit **jamais** faire échouer l'écriture qu'il accompagne. Rendre
/// un `Result` inviterait un appelant à l'`?`, et une entrée de registre indisponible refuserait
/// une commande au comptoir. L'implémentation trace son propre échec ; l'appelant continue.
#[async_trait::async_trait]
pub trait SignalDerive: Send + Sync {
    /// Le seuil en vigueur pour cet établissement, lu de la configuration.
    ///
    /// **Le seuil n'est pas une constante** (principe I·c) : `sync.derive_horloge_seuil_secondes`,
    /// migration `0028`, défaut 300 s. Un établissement dont le parc de terminaux est mauvais doit
    /// pouvoir le resserrer sans livraison.
    ///
    /// L'implémentation rend le défaut du catalogue si la configuration est illisible : refuser de
    /// constater parce qu'un paramètre manque reviendrait à désactiver le signalement au moment où
    /// la base est le plus perturbée.
    async fn seuil(&self, tenant_id: uuid::Uuid, etablissement_id: uuid::Uuid) -> Duration;

    /// Écrit le constat au registre des actions. **Appelée seulement si une dérive est constatée.**
    ///
    /// C'est ici que vit le débrayage par épisode : deux cents saisies pendant un service ne
    /// doivent produire qu'une entrée.
    async fn consigner(&self, origine: OrigineDerive, derive: Derive);

    /// **Le point d'entrée des services** — constate, puis consigne s'il y a lieu.
    ///
    /// # Pourquoi une méthode par défaut, et non trois appels chez l'appelant
    ///
    /// Un service qui appellerait `seuil()`, puis `constater_derive()`, puis `consigner()` aurait
    /// trois occasions de se tromper — oublier la valeur absolue, comparer au mauvais seuil,
    /// consigner sans débrayer. Et le prochain service la réécrirait à sa façon.
    ///
    /// L'enchaînement est donc ici, une fois. **Ce qui reste à l'appelant est ce qu'il est seul à
    /// savoir** : quels sont les deux instants, et de quel terminal ils viennent.
    async fn constater_et_signaler(
        &self,
        origine: OrigineDerive,
        etablissement_id: uuid::Uuid,
        horodatage_client: OffsetDateTime,
        autorite: OffsetDateTime,
    ) {
        let seuil = self.seuil(origine.tenant_id, etablissement_id).await;
        if let Some(derive) = constater_derive(horodatage_client, autorite, seuil) {
            self.consigner(origine, derive).await;
        }
    }
}

/// **Le débrayage par épisode** — une entrée par épisode de dérive, jamais une par écriture.
///
/// # Le défaut que ce mécanisme empêche, et il est quantifié
///
/// Un terminal dont l'horloge est fausse l'est **pendant tout un service**. Sans débrayage, deux
/// cents saisies produiraient deux cents entrées identiques au registre des actions, qui deviendrait
/// illisible — donc inutilisé. C'est la façon la plus sûre de neutraliser un registre : le noyer.
///
/// # La clé est ÉPHÉMÈRE RECONSTRUCTIBLE, au sens du principe II
///
/// Elle vit en Redis, avec une durée de vie. **La perdre produit une entrée d'audit de plus, jamais
/// une donnée manquante** — c'est exactement le critère qui autorise Redis dans ce produit, et
/// c'est pourquoi le débrayage ne peut pas être une colonne en base : une table de « dernier
/// signalement par terminal » serait une donnée dérivée à sauvegarder, à migrer et à purger.
///
/// # Ce que le débrayage NE fait pas
///
/// Il ne dédoublonne pas la dérive : deux terminaux qui dévient produisent deux entrées, et c'est
/// juste — l'exploitant cherche **quel** appareil régler. La clé porte donc le triplet
/// `(tenant, compte, appareil)`, pas le seul tenant.
pub trait DebrayageEpisode: Send + Sync {
    /// Faut-il signaler cette dérive, ou l'épisode est-il déjà consigné ?
    ///
    /// **Rend `true` en cas de doute** — Redis injoignable, clé illisible. Une entrée de trop est
    /// un bruit ; une entrée manquante est une information perdue dans un registre immuable, où
    /// rien ne se rattrape après coup.
    fn premier_du_episode(&self, origine: OrigineDerive) -> bool;
}

/// La clé de débrayage d'une origine — **la forme est stable**, elle vit en Redis.
///
/// L'appareil absent retombe sur `sans-appareil` plutôt que sur une clé plus courte : deux formes
/// de clé pour la même notion se répondraient mal le jour où l'enrôlement (CPT-05) arrivera, et
/// c'est un débrayage qui cesserait de débrayer sans que rien ne le dise.
pub fn cle_debrayage(origine: OrigineDerive) -> String {
    let appareil = origine
        .appareil_id
        .map(|id| id.to_string())
        .unwrap_or_else(|| "sans-appareil".to_owned());
    format!(
        "derive:{}:{}:{}",
        origine.tenant_id, origine.compte_id, appareil
    )
}

/// Un canal qui ne signale rien — pour les assemblages où le registre n'existe pas.
///
/// **Ce n'est pas un bouchon de test.** C'est le comportement exact d'un montage sans journal
/// d'audit : le nœud de site, par exemple, qui n'a pas de registre local. Le nommer évite qu'un
/// tel montage passe `Option<Box<dyn SignalDerive>>` et que chaque appelant traite le `None`.
#[derive(Debug, Clone, Copy, Default)]
pub struct SignalMuet;

#[async_trait::async_trait]
impl SignalDerive for SignalMuet {
    async fn seuil(&self, _tenant_id: Uuid, _etablissement_id: Uuid) -> Duration {
        SEUIL_DERIVE_DEFAUT
    }

    async fn consigner(&self, _origine: OrigineDerive, _derive: Derive) {}
}

/// Le défaut du catalogue — **300 secondes**, valeur du cadrage §11.4.
///
/// Il est répété ici et **ce n'est pas une seconde source de vérité** : le catalogue décide, ce
/// nombre sert quand la configuration est illisible. Le principe I·c interdit d'inscrire une valeur
/// métier en dur ; il n'interdit pas d'avoir un comportement quand la valeur ne se lit pas.
pub const SEUIL_DERIVE_DEFAUT: Duration = Duration::from_secs(300);

/// La clé du catalogue — nommée ici pour que l'implémentation et les tests la partagent.
pub const CLE_SEUIL_DERIVE: &str = "sync.derive_horloge_seuil_secondes";

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::datetime;

    const SEUIL: Duration = Duration::from_secs(300);

    #[test]
    fn sous_le_seuil_rien_n_est_constate() {
        // Deux horloges ne sont jamais exactement d'accord. Écrire au registre à chaque saisie le
        // rendrait illisible, donc inutilisé.
        let autorite = datetime!(2026-08-03 18:00:00 UTC);
        let client = datetime!(2026-08-03 17:58:00 UTC); // deux minutes de retard

        assert_eq!(constater_derive(client, autorite, SEUIL), None);
    }

    #[test]
    fn le_seuil_exact_ne_declenche_pas() {
        // `<=` et non `<` : un seuil de cinq minutes veut dire « au-delà de cinq minutes », et
        // c'est ainsi que le cadrage §11.4 l'écrit — « alerte au-delà de 5 minutes de dérive ».
        let autorite = datetime!(2026-08-03 18:00:00 UTC);
        let client = datetime!(2026-08-03 17:55:00 UTC);

        assert_eq!(constater_derive(client, autorite, SEUIL), None);
    }

    #[test]
    fn un_terminal_en_retard_est_constate() {
        let autorite = datetime!(2026-08-03 18:00:00 UTC);
        let client = datetime!(2026-08-03 17:50:00 UTC); // dix minutes de retard

        let derive = constater_derive(client, autorite, SEUIL).expect("dérive attendue");

        assert_eq!(derive.sens, SensDerive::Retard);
        assert_eq!(derive.ecart_secondes, 600);
        assert_eq!(derive.ecart_minutes(), 10);
        assert_eq!(derive.seuil_secondes, 300);
    }

    #[test]
    fn un_terminal_en_avance_est_constate_aussi() {
        // **Le cas que l'écart signé aurait laissé passer**, et c'est celui du scénario de recette
        // du quickstart : un horodatage client trois heures dans le futur.
        let autorite = datetime!(2026-08-03 18:00:00 UTC);
        let client = datetime!(2026-08-03 21:00:00 UTC);

        let derive = constater_derive(client, autorite, SEUIL).expect("dérive attendue");

        assert_eq!(derive.sens, SensDerive::Avance);
        assert_eq!(derive.ecart_secondes, 3 * 3600);
        assert_eq!(derive.ecart_minutes(), 180);
    }

    #[test]
    fn les_deux_sens_sont_symetriques_a_ecart_egal() {
        let autorite = datetime!(2026-08-03 18:00:00 UTC);
        let retard = constater_derive(datetime!(2026-08-03 17:50:00 UTC), autorite, SEUIL)
            .expect("retard");
        let avance = constater_derive(datetime!(2026-08-03 18:10:00 UTC), autorite, SEUIL)
            .expect("avance");

        assert_eq!(retard.ecart_secondes, avance.ecart_secondes);
        assert_ne!(retard.sens, avance.sens);
    }

    #[test]
    fn l_arrondi_des_minutes_va_au_plus_proche() {
        // 5 min 31 s → 6 minutes. Afficher « 5,5 minutes » donnerait à croire à une précision que
        // la mesure n'a pas.
        let autorite = datetime!(2026-08-03 18:00:00 UTC);
        let client = autorite - Duration::from_secs(331);

        let derive = constater_derive(client, autorite, SEUIL).expect("dérive attendue");
        assert_eq!(derive.ecart_minutes(), 6);
    }

    #[test]
    fn les_deux_codes_de_sens_sont_stables() {
        // Ils partent dans le contexte JSONB d'un registre **immuable, à rétention illimitée**.
        // Un renommage de variante ne doit pas changer ce qui est déjà écrit.
        assert_eq!(SensDerive::Retard.code(), "retard");
        assert_eq!(SensDerive::Avance.code(), "avance");
    }
}
