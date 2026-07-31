/**
 * Authentification — **coquille structurelle. La logique est CPT-01.**
 *
 * Ce fichier n'existe que pour que la place soit prise et que les types que le reste de
 * l'application consommera soient déjà nommés.
 *
 * # Ce qui est déjà arrêté, et qu'il ne faudra pas redécider
 *
 * - **JWT court + refresh révocable** (gel §3.1, `jsonwebtoken`).
 * - **Enrôlement d'appareil par paire de clés** générée dans le Keystore/Keychain, qui signe
 *   chaque requête (principe IX). **Le verrouillage par adresse MAC n'est jamais implémenté** :
 *   iOS 14 et Android 10 randomisent la MAC par réseau, et Android n'expose pas la MAC
 *   matérielle. Ce n'est pas une difficulté à contourner, c'est une impossibilité.
 * - **Aucun secret dans le binaire Tauri** — il est décompilable.
 * - **Aucune élévation de privilège hors ligne, jamais** (registre §5.2, classe C).
 */

export interface SessionUtilisateur {
  readonly compteId: string
  readonly tenantId: string
  /** Établissement actif du sélecteur de contexte permanent (principe VII). */
  readonly etablissementId: string | null
  /** Permissions **cumulées** de tous les rôles portés — voir `core/rbac`. */
  readonly permissions: readonly string[]
}

/** Aucune session tant que CPT-01 n'est pas livré. */
export function sessionCourante(): SessionUtilisateur | null {
  return null
}
