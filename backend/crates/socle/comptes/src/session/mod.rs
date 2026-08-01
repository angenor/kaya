//! `session` — **la connexion, la rotation, et la coupure immédiate** (CPT-01).
//!
//! Terme utilisateur : **« Appareil connecté »** / *Connected device*. Les mots « session »,
//! « jeton » et « JWT » n'atteignent jamais l'interface (`docs/design/lexique.md`).
//!
//! # Ce que ce sous-module tient, en trois phrases
//!
//! Un jeton d'accès **signé et court** porte le contexte d'appel, ce qui évite de relire la base à
//! chaque requête. Un jeton de rafraîchissement **long et à rotation** le renouvelle, et sa
//! réutilisation révèle qu'une copie circule. Une **liste de révocation** en Redis, consultée à
//! chaque requête, permet de couper une session en cours sans attendre l'expiration de quoi que
//! ce soit.
//!
//! # Le point qu'on écrirait mal — révoquer le jeton présenté
//!
//! Sur détection de réutilisation, c'est **toute la famille** qui tombe, pas l'exemplaire
//! présenté. Révoquer le seul exemplaire laisserait le voleur et la victime en course, et le
//! premier des deux gagnerait — sans qu'aucun des deux ne sache qu'il y a eu course. En révoquant
//! la famille, les **deux** sont déconnectés : le voleur perd l'accès, et la victime apprend
//! qu'il s'est passé quelque chose au moment où elle doit se reconnecter.
//!
//! | Fichier | Ce qu'il porte |
//! |---|---|
//! | [`modele`] | Les types, et le refus commun `identifiants_invalides` |
//! | [`jeton`] | Signature et vérification des deux jetons |
//! | [`entrepot`] | Les **trois familles de clés** Redis |
//! | [`parametres`] | Les durées, lues du catalogue |
//! | [`limite`] | La limitation de débit — **deux clés**, l'identifiant *et* l'origine |

pub mod entrepot;
pub mod jeton;
pub mod limite;
pub mod modele;
pub mod parametres;

pub use entrepot::Entrepot;
pub use jeton::{ClaimsAcces, ClaimsRafraichissement};
pub use limite::LimiteTentatives;
pub use modele::{ErreurSession, JetonsDelivres, Session, SessionVue};
pub use parametres::DureesSession;
