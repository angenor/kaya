//! Le service d'authentification — **et l'indiscernabilité, qui est la moitié du travail**.
//!
//! # Ce que le message identique ne tient pas
//!
//! FR-012 exige que deux échecs de connexion soient indiscernables. Rendre le même code et la même
//! phrase est la partie facile, et elle ne suffit pas : sur identifiant **inconnu**, un service
//! naïf répond en une fraction de milliseconde — il n'a rien trouvé, il n'a rien à vérifier —
//! alors que sur identifiant **existant** il paie 19 Mio et quelques dizaines de millisecondes
//! d'Argon2. L'écart se mesure depuis n'importe quel réseau, et **il dit qui est client de Kaya**.
//!
//! La liste du personnel d'un hôtel est affichée à l'accueil. Savoir qu'un numéro correspond à un
//! compte, c'est savoir sur qui insister.
//!
//! D'où [`ServiceAuthentification::ouvrir`] : sur identifiant inconnu, la vérification Argon2 est
//! **exécutée quand même**, contre le condensat factice de
//! [`crate::authentification::condensat_factice`]. Le coût est le même dans les deux cas, et le
//! refus est le même objet.
//!
//! # Les trois refus qui n'en font qu'un
//!
//! Compte inconnu, mot de passe faux, compte désactivé : `identifiants_invalides`, sans
//! distinction. Un `compte_desactive` obligeant serait pourtant utile à l'utilisateur légitime —
//! et dirait à l'attaquant que le compte existe. L'arbitrage est tranché par FR-012, et le
//! diagnostic part dans les journaux, où le support le retrouve.
//!
//! # Ce qui n'émet AUCUN événement, et pourquoi
//!
//! Ni la connexion, ni le rafraîchissement, ni l'échec (research R-15). Ce ne sont pas des
//! transitions d'état métier, et le grand livre a une **rétention illimitée** : les y inscrire y
//! écrirait la liste horodatée des présences du personnel, pour toujours. La **révocation**, elle,
//! en émet un — c'est un acte d'administration, pas une trace de présence.

use std::collections::BTreeSet;

use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use kaya_etablissements::tenant_context;
use kaya_synchronisation::{EvenementAEcrire, OutboxWriter};

use crate::audit::{EntreeAudit, JournalAudit, TypeActionAudit};
use crate::compte::repository as comptes;
use crate::roles::repository as roles;
use crate::session::entrepot::Entrepot;
use crate::session::jeton::{self, ClaimsAcces, ClaimsRafraichissement};
use crate::session::modele::{ErreurSession, JetonsDelivres, Session, SessionVue};
use crate::session::parametres::{self, DureesSession};

/// Nom de l'agrégat au grand livre, pour les événements de session.
pub const AGREGAT_COMPTE: &str = "compte";

/// Le seul événement que ce service émet.
pub const TYPE_SESSION_REVOQUEE: &str = "session.revoquee";

/// Version du format de la charge utile.
pub const VERSION_SCHEMA_SESSION: i16 = 1;

/// Ce que la connexion rend à l'appelant, avant mise en forme HTTP.
#[derive(Debug, Clone)]
pub struct SessionOuverte {
    pub jetons: JetonsDelivres,
    pub compte_id: Uuid,
    pub tenant_id: Uuid,
    pub etablissement_actif: Option<Uuid>,
    /// **L'union** des permissions des rôles portés. Le front la lit **ici**, jamais en décodant
    /// le jeton (research R-06).
    pub permissions: BTreeSet<String>,
    pub etablissements: Vec<Uuid>,
}

/// Service d'authentification et de session.
pub struct ServiceAuthentification<E: OutboxWriter, J: JournalAudit> {
    pool: PgPool,
    entrepot: Entrepot,
    cle_jwt: Vec<u8>,
    outbox: E,
    audit: J,
}

impl<E: OutboxWriter, J: JournalAudit> ServiceAuthentification<E, J> {
    pub fn nouveau(pool: PgPool, entrepot: Entrepot, cle_jwt: Vec<u8>, outbox: E, audit: J) -> Self {
        Self {
            pool,
            entrepot,
            cle_jwt,
            outbox,
            audit,
        }
    }

    // ═════════════════════════════════════════════════════════════════════════════════════════
    //  Ouvrir une session
    // ═════════════════════════════════════════════════════════════════════════════════════════

    /// Ouvre une session — **ou refuse, en un temps qui ne dit rien**.
    ///
    /// Ordre des opérations, et chacun compte :
    ///
    ///   1. résoudre l'identifiant, **sans tenant** (migration `0020`, la seule dérogation) ;
    ///   2. vérifier le mot de passe — contre le vrai condensat, **ou contre le factice** ;
    ///   3. refuser d'un seul refus si quoi que ce soit cloche ;
    ///   4. poser le tenant, calculer permissions et établissements ;
    ///   5. rehacher si les paramètres du condensat ont vieilli ;
    ///   6. délivrer les deux jetons et enregistrer la session.
    ///
    /// **L'étape 2 est celle qu'on écrirait mal.** La tentation est d'écrire
    /// `let Some(compte) = … else { return Err(…) }` juste après l'étape 1 : c'est correct
    /// fonctionnellement, et cela rétablit exactement l'écart de temps que FR-012 ferme.
    #[tracing::instrument(skip(self, mot_de_passe), fields(identifiant.longueur = identifiant.len()))]
    pub async fn ouvrir(
        &self,
        identifiant: &str,
        mot_de_passe: &str,
        etablissement_demande: Option<Uuid>,
        libelle_appareil: Option<String>,
    ) -> Result<SessionOuverte, ErreurSession> {
        let mut tx = self.pool.begin().await?;

        let trouve = comptes::resoudre_identifiant(&mut tx, identifiant)
            .await
            .map_err(|e| {
                tracing::warn!(erreur = %e, "résolution de l'identifiant impossible");
                ErreurSession::IdentifiantsInvalides
            })?;

        // Le condensat contre lequel on vérifie : le vrai, ou le factice. **Dans les deux cas, on
        // vérifie.** C'est la ligne qui tient l'indiscernabilité temporelle.
        let condensat = trouve
            .as_ref()
            .map(|c| c.condensat_mot_de_passe.as_str())
            .unwrap_or_else(|| crate::authentification::condensat_factice());

        let verification = crate::authentification::verifier(condensat, mot_de_passe)?;

        // Trois causes de refus, un seul refus. Le `match` est écrit à plat plutôt qu'en cascade
        // de `if` pour qu'aucune branche ne puisse revenir en avance.
        let compte = match trouve {
            Some(compte) if verification.valide && compte.actif => compte,
            autre => {
                // Le diagnostic part dans les journaux — jamais dans la réponse.
                tracing::info!(
                    connu = autre.is_some(),
                    mot_de_passe_valide = verification.valide,
                    actif = autre.as_ref().map(|c| c.actif),
                    "échec d'authentification"
                );
                return Err(ErreurSession::IdentifiantsInvalides);
            }
        };

        // `OTP_SMS` : refus **nommé**, jamais un repli silencieux sur le mot de passe (FR-008).
        // Le contrôle vient APRÈS la vérification du mot de passe, et c'est délibéré : le placer
        // avant permettrait de savoir qu'un compte existe en observant un `422` là où un compte
        // inconnu rendrait `401`.
        if compte.methode_code != "MOT_DE_PASSE" {
            return Err(ErreurSession::MethodeNonImplementee(compte.methode_code));
        }

        tenant_context::poser_tenant(&mut tx, compte.tenant_id).await?;

        let etablissements = roles::etablissements_accessibles(&mut tx, compte.id)
            .await
            .map_err(erreur_roles)?;
        let porte_editeur = roles::porte_un_role_editeur(&mut tx, compte.id)
            .await
            .map_err(erreur_roles)?;

        // Sans `etablissement_id` demandé, **le premier accessible par ordre stable** devient
        // actif (contrat §1). Un `admin_editeur` n'en a aucun, et c'est normal.
        let etablissement_actif = match etablissement_demande {
            Some(demande) if etablissements.contains(&demande) => Some(demande),
            // Un établissement demandé mais non accessible ne produit **pas** d'erreur distincte :
            // ce serait un moyen d'énumérer les établissements d'un tenant. On retombe sur le
            // premier accessible, comme si rien n'avait été demandé.
            Some(_) | None => etablissements.first().copied(),
        };
        let etablissement_actif = if porte_editeur && etablissement_actif.is_none() {
            None
        } else {
            etablissement_actif
        };

        let permissions = roles::permissions_effectives(&mut tx, compte.id, etablissement_actif)
            .await
            .map_err(erreur_roles)?;

        // **Rehachage silencieux.** Il suit la vérification réussie, jamais l'échec : rehacher
        // après un mauvais mot de passe écrirait le condensat du mot de passe... faux.
        if verification.rehachage_requis {
            let neuf = crate::authentification::hacher(mot_de_passe)?;
            comptes::ecrire_condensat(&mut tx, compte.id, &neuf)
                .await
                .map_err(|e| {
                    tracing::warn!(erreur = %e, "rehachage impossible");
                    ErreurSession::IdentifiantsInvalides
                })?;
            tracing::info!(compte_id = %compte.id, "condensat rehaché aux paramètres du jour");
        }

        let durees = parametres::resoudre(&mut tx, etablissement_actif).await?;
        tx.commit().await?;

        let session = Session {
            id: Uuid::now_v7(),
            compte_id: compte.id,
            tenant_id: compte.tenant_id,
            etablissement_id: etablissement_actif,
            famille_id: Uuid::now_v7(),
            libelle_appareil,
            ouverte_le: OffsetDateTime::now_utc(),
            derniere_activite_le: OffsetDateTime::now_utc(),
            expire_le: OffsetDateTime::now_utc() + time::Duration::seconds(durees.rafraichissement_s),
        };

        let jetons = self.delivrer(&session, &permissions, durees).await?;

        Ok(SessionOuverte {
            jetons,
            compte_id: compte.id,
            tenant_id: compte.tenant_id,
            etablissement_actif,
            permissions,
            etablissements,
        })
    }

    // ═════════════════════════════════════════════════════════════════════════════════════════
    //  Rafraîchir
    // ═════════════════════════════════════════════════════════════════════════════════════════

    /// Rafraîchit une session — **rotation, et détection de réutilisation**.
    ///
    /// Le jeton présenté est consommé et remplacé. S'il ne correspond pas à l'exemplaire que Redis
    /// retient pour la famille, **une copie circule** : la famille entière est révoquée, un
    /// événement et une entrée d'audit sont écrits, et l'appelant reçoit `401 session_invalide`.
    ///
    /// **Les permissions sont recalculées ici.** C'est le moment où un rôle retiré prend effet —
    /// au plus une durée de jeton d'accès après le retrait (hypothèse 5 de la spec).
    #[tracing::instrument(skip(self, jeton_presente))]
    pub async fn rafraichir(
        &self,
        jeton_presente: &str,
        etablissement_demande: Option<Uuid>,
    ) -> Result<SessionOuverte, ErreurSession> {
        let claims = jeton::verifier_rafraichissement(&self.cle_jwt, jeton_presente)
            .map_err(|_| ErreurSession::SessionInvalide)?;

        // La révocation prime sur tout le reste : une session révoquée ne se rafraîchit pas, même
        // avec un jeton parfaitement signé et non expiré.
        if self.entrepot.est_revoquee(claims.sid).await? {
            return Err(ErreurSession::SessionInvalide);
        }

        match self.entrepot.exemplaire_valide(claims.fid).await? {
            Some(attendu) if attendu == claims.jti => {}
            _ => {
                // **Réutilisation, ou famille déjà révoquée.** Dans les deux cas on révoque à
                // nouveau : l'opération est idempotente, et la refaire coûte moins qu'un `if` de
                // plus sur un chemin de sécurité.
                self.revoquer_pour_reutilisation(&claims).await?;
                return Err(ErreurSession::SessionInvalide);
            }
        }

        let Some(session) = self.entrepot.lire(claims.sub, claims.sid).await? else {
            return Err(ErreurSession::SessionInvalide);
        };

        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, session.tenant_id).await?;

        // Le compte a pu être désactivé depuis l'ouverture. Le rafraîchissement est le seul
        // endroit où cela se constate sans coût supplémentaire — et une désactivation qui ne
        // couperait qu'à l'expiration naturelle laisserait 90 jours d'accès.
        let compte = comptes::lire_pour_authentification(&mut tx, session.compte_id)
            .await
            .map_err(|_| ErreurSession::SessionInvalide)?;
        match compte {
            Some(c) if c.actif => {}
            _ => {
                tx.rollback().await?;
                return Err(ErreurSession::SessionInvalide);
            }
        }

        let etablissements = roles::etablissements_accessibles(&mut tx, session.compte_id)
            .await
            .map_err(erreur_roles)?;
        let etablissement_actif = match etablissement_demande {
            Some(demande) if etablissements.contains(&demande) => Some(demande),
            _ => session.etablissement_id.filter(|e| etablissements.contains(e)),
        };

        let permissions =
            roles::permissions_effectives(&mut tx, session.compte_id, etablissement_actif)
                .await
                .map_err(erreur_roles)?;
        let durees = parametres::resoudre(&mut tx, etablissement_actif).await?;
        tx.commit().await?;

        let session = Session {
            etablissement_id: etablissement_actif,
            derniere_activite_le: OffsetDateTime::now_utc(),
            ..session
        };
        let jetons = self.delivrer(&session, &permissions, durees).await?;

        Ok(SessionOuverte {
            jetons,
            compte_id: session.compte_id,
            tenant_id: session.tenant_id,
            etablissement_actif,
            permissions,
            etablissements,
        })
    }

    // ═════════════════════════════════════════════════════════════════════════════════════════
    //  Fermer, lister, révoquer
    // ═════════════════════════════════════════════════════════════════════════════════════════

    /// Ferme la session courante — la déconnexion volontaire.
    ///
    /// **N'émet aucun événement et n'écrit aucune entrée d'audit** : se déconnecter de son propre
    /// appareil n'est pas un acte d'administration. La révocation d'une **autre** session, si.
    pub async fn fermer(&self, compte_id: Uuid, session_id: Uuid) -> Result<(), ErreurSession> {
        if let Some(session) = self.entrepot.lire(compte_id, session_id).await? {
            self.entrepot.revoquer_famille(session.famille_id).await?;
        }
        self.entrepot.oublier(compte_id, session_id).await?;
        Ok(())
    }

    /// Les sessions actives du compte appelant.
    ///
    /// Reconstruites depuis Redis : si Redis a été vidé, la liste est vide et **tout le monde
    /// s'est reconnecté** — ce qui est exact, pas une panne (research R-01).
    pub async fn lister_actives(
        &self,
        compte_id: Uuid,
        session_courante: Uuid,
    ) -> Result<Vec<SessionVue>, ErreurSession> {
        Ok(self
            .entrepot
            .lister(compte_id)
            .await?
            .iter()
            .map(|s| s.en_vue(session_courante))
            .collect())
    }

    /// Révoque une session — **effet immédiat, et c'est tout le sujet**.
    ///
    /// La marque Redis est consultée à chaque requête authentifiée : le jeton d'accès en
    /// circulation cesse d'être accepté **à l'appel suivant**, sans attendre son expiration. C'est
    /// la « coupure immédiate au départ d'un employé » du cadrage §12.2, et le seul recours contre
    /// un téléphone volé avant l'enrôlement d'appareil de CPT-05.
    ///
    /// Émet `session.revoquee` **et** son entrée d'audit.
    #[tracing::instrument(skip(self))]
    pub async fn revoquer(
        &self,
        auteur_compte_id: Uuid,
        tenant_id: Uuid,
        proprietaire_compte_id: Uuid,
        session_id: Uuid,
        duree_acces_s: i64,
    ) -> Result<(), ErreurSession> {
        // La marque d'abord : c'est elle qui coupe. Tout ce qui suit est de la trace, et un échec
        // de trace ne doit pas laisser une session ouverte.
        self.entrepot
            .marquer_revoquee(session_id, duree_acces_s)
            .await?;

        if let Some(session) = self.entrepot.lire(proprietaire_compte_id, session_id).await? {
            self.entrepot.revoquer_famille(session.famille_id).await?;
        }
        self.entrepot
            .oublier(proprietaire_compte_id, session_id)
            .await?;

        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, tenant_id).await?;

        self.audit
            .tracer(
                &mut tx,
                tenant_id,
                EntreeAudit {
                    id: Uuid::now_v7(),
                    etablissement_id: None,
                    type_action: TypeActionAudit::Suppression,
                    auteur_compte_id,
                    cible_type: "session".to_owned(),
                    cible_id: Some(session_id),
                    contexte: serde_json::json!({
                        "compte_cible_id": proprietaire_compte_id,
                        "soi_meme": auteur_compte_id == proprietaire_compte_id,
                    }),
                    horodatage_client: None,
                },
            )
            .await?;

        self.outbox
            .ecrire(
                &mut tx,
                EvenementAEcrire {
                    id: Uuid::now_v7(),
                    tenant_id,
                    etablissement_id: None,
                    type_evenement: TYPE_SESSION_REVOQUEE.to_owned(),
                    agregat: AGREGAT_COMPTE.to_owned(),
                    agregat_id: proprietaire_compte_id,
                    version_schema: VERSION_SCHEMA_SESSION,
                    payload: serde_json::json!({
                        "compte_id": proprietaire_compte_id,
                        "session_id": session_id,
                        "auteur_compte_id": auteur_compte_id,
                        "motif": "revocation",
                    }),
                },
            )
            .await?;

        tx.commit().await?;
        Ok(())
    }

    /// Révoque **toutes les autres sessions** d'un compte — appelé au changement de mot de passe.
    ///
    /// « Toutes les autres » et non « toutes » : celui qui change son mot de passe ne doit pas se
    /// déconnecter lui-même en le faisant. `session_courante` vaut `None` quand un habilité change
    /// le mot de passe de quelqu'un d'autre — et alors **tout** tombe, ce qui est le comportement
    /// voulu.
    pub async fn revoquer_les_autres(
        &self,
        compte_id: Uuid,
        session_courante: Option<Uuid>,
        duree_acces_s: i64,
    ) -> Result<usize, ErreurSession> {
        let sessions = self.entrepot.lister(compte_id).await?;
        let mut revoquees = 0;

        for session in sessions {
            if Some(session.id) == session_courante {
                continue;
            }
            self.entrepot
                .marquer_revoquee(session.id, duree_acces_s)
                .await?;
            self.entrepot.revoquer_famille(session.famille_id).await?;
            self.entrepot.oublier(compte_id, session.id).await?;
            revoquees += 1;
        }

        Ok(revoquees)
    }

    /// La session est-elle révoquée ? **Consultée à chaque requête authentifiée.**
    pub async fn est_revoquee(&self, session_id: Uuid) -> Result<bool, ErreurSession> {
        Ok(self.entrepot.est_revoquee(session_id).await?)
    }

    // ═════════════════════════════════════════════════════════════════════════════════════════
    //  Interne
    // ═════════════════════════════════════════════════════════════════════════════════════════

    /// Signe le couple de jetons, pose l'exemplaire de famille et enregistre la session.
    async fn delivrer(
        &self,
        session: &Session,
        permissions: &BTreeSet<String>,
        durees: DureesSession,
    ) -> Result<JetonsDelivres, ErreurSession> {
        let maintenant = OffsetDateTime::now_utc().unix_timestamp();
        let jti = Uuid::now_v7();

        let acces = jeton::signer_acces(
            &self.cle_jwt,
            &ClaimsAcces {
                sub: session.compte_id,
                tenant: session.tenant_id,
                etablissement: session.etablissement_id,
                sid: session.id,
                perms: permissions.iter().cloned().collect(),
                iat: maintenant,
                exp: maintenant + durees.acces_s,
            },
        )?;

        let rafraichissement = jeton::signer_rafraichissement(
            &self.cle_jwt,
            &ClaimsRafraichissement {
                sub: session.compte_id,
                tenant: session.tenant_id,
                sid: session.id,
                fid: session.famille_id,
                jti,
                iat: maintenant,
                exp: maintenant + durees.rafraichissement_s,
            },
        )?;

        // L'exemplaire est posé **après** la signature et avant l'enregistrement : si la signature
        // échoue, aucune rotation n'a eu lieu et le jeton précédent reste valide.
        self.entrepot
            .poser_exemplaire(session.famille_id, jti, durees.rafraichissement_s)
            .await?;
        self.entrepot
            .enregistrer(session, durees.rafraichissement_s)
            .await?;

        Ok(JetonsDelivres {
            acces,
            expire_dans_s: durees.acces_s,
            rafraichissement,
            session_id: session.id,
        })
    }

    /// Réponse à un jeton de rafraîchissement réutilisé.
    ///
    /// **Toute la famille tombe**, pas l'exemplaire présenté. Révoquer le seul exemplaire
    /// laisserait le voleur et la victime en course, et le premier des deux gagnerait — sans
    /// qu'aucun des deux ne sache qu'il y a eu course.
    async fn revoquer_pour_reutilisation(
        &self,
        claims: &ClaimsRafraichissement,
    ) -> Result<(), ErreurSession> {
        tracing::warn!(
            session_id = %claims.sid,
            famille_id = %claims.fid,
            "jeton de rafraîchissement réutilisé — révocation de la famille entière"
        );

        self.entrepot.revoquer_famille(claims.fid).await?;
        // La marque de révocation coupe aussi le jeton d'ACCÈS en circulation, qui reste
        // mathématiquement valide. Sans elle, le voleur garderait un accès jusqu'à l'expiration.
        self.entrepot
            .marquer_revoquee(claims.sid, DureesSession::repli().acces_s)
            .await?;
        self.entrepot.oublier(claims.sub, claims.sid).await?;

        let mut tx = self.pool.begin().await?;
        tenant_context::poser_tenant(&mut tx, claims.tenant).await?;

        self.audit
            .tracer(
                &mut tx,
                claims.tenant,
                EntreeAudit {
                    id: Uuid::now_v7(),
                    etablissement_id: None,
                    type_action: TypeActionAudit::Suppression,
                    auteur_compte_id: claims.sub,
                    cible_type: "session".to_owned(),
                    cible_id: Some(claims.sid),
                    contexte: serde_json::json!({
                        "motif": "reutilisation_jeton_rafraichissement",
                        "famille_id": claims.fid,
                    }),
                    horodatage_client: None,
                },
            )
            .await?;

        self.outbox
            .ecrire(
                &mut tx,
                EvenementAEcrire {
                    id: Uuid::now_v7(),
                    tenant_id: claims.tenant,
                    etablissement_id: None,
                    type_evenement: TYPE_SESSION_REVOQUEE.to_owned(),
                    agregat: AGREGAT_COMPTE.to_owned(),
                    agregat_id: claims.sub,
                    version_schema: VERSION_SCHEMA_SESSION,
                    payload: serde_json::json!({
                        "compte_id": claims.sub,
                        "session_id": claims.sid,
                        "auteur_compte_id": claims.sub,
                        "motif": "reutilisation_jeton_rafraichissement",
                    }),
                },
            )
            .await?;

        tx.commit().await?;
        Ok(())
    }
}

/// Aplati une erreur de lecture des droits sur le refus commun.
///
/// Une panne de base pendant le calcul des permissions ne doit **pas** produire un message
/// différent : ce serait un canal par lequel distinguer un compte existant d'un compte inconnu en
/// provoquant l'erreur.
fn erreur_roles(erreur: crate::roles::ErreurRoles) -> ErreurSession {
    tracing::error!(erreur = %erreur, "lecture des droits impossible");
    ErreurSession::IdentifiantsInvalides
}
