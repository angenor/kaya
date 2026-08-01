/**
 * Authentification — **la coquille du cycle 001 est remplacée** (CPT-01).
 *
 * # Ce qui est arrêté, et qu'il ne faut pas redécider
 *
 * - **JWT court + rafraîchissement révocable** (gel §3.1, `jsonwebtoken`). Soixante minutes et
 *   quatre-vingt-dix jours, tous deux **paramètres d'établissement**, jamais des constantes.
 * - **Le front ne décode jamais le jeton.** Permissions, tenant, compte et établissement actif
 *   viennent du corps de la réponse, en clair (research R-06).
 * - **Le stockage du rafraîchissement passe entièrement par `PlatformAdapter`** — porte P-15, et
 *   c'est le premier usage d'une capacité native par un écran du produit.
 * - **Enrôlement d'appareil par paire de clés** générée dans le Keystore/Keychain (CPT-05).
 *   **Le verrouillage par adresse MAC n'est jamais implémenté** : iOS 14 et Android 10
 *   randomisent la MAC par réseau. Ce n'est pas une difficulté à contourner, c'est une
 *   impossibilité.
 * - **Aucun secret dans le binaire Tauri** — il est décompilable.
 * - **Aucune élévation de privilège hors ligne, jamais** (registre §5.2, classe C).
 *
 * # Les trois fichiers
 *
 * | Fichier | Ce qu'il porte |
 * |---|---|
 * | `session.ts` | L'état — deux jetons, deux emplacements — et le contexte d'appel |
 * | `connexion.ts` | Les quatre opérations et leur table de refus |
 * | `index.ts` | Ce fichier : la surface publique, et rien d'autre |
 */

export {
  accesExpire,
  CLE_RAFRAICHISSEMENT,
  contexteAppel,
  effacerSession,
  enTetesAuth,
  lireRafraichissement,
  oublierRafraichissement,
  poserSession,
  rangerRafraichissement,
  sessionCourante,
  type ContexteAppel,
  type SessionUtilisateur,
} from './session'

export {
  fermerSession,
  ouvrirSession,
  rafraichirSession,
  REFUS_IDENTIFIANTS,
  REFUS_INATTENDU,
  REFUS_METHODE,
  REFUS_RESEAU,
  REFUS_SESSION_EXPIREE,
  reprendreSession,
  type Identifiants,
  type ResultatConnexion,
} from './connexion'
