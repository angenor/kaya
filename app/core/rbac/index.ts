/**
 * Permissions — **coquille structurelle. La logique est CPT-02.**
 *
 * # La règle qui décide de toute l'interface
 *
 * **Les rôles sont CUMULABLES, et c'est la norme, pas l'exception** (principe VII). Un gérant est
 * aussi caissier et réceptionniste ; ses permissions sont l'**union** de celles de ses rôles,
 * jamais l'intersection ni celles d'un rôle « principal ».
 *
 * Deux conséquences que le cycle ETB devra tenir dès son premier écran :
 *
 * - **l'accueil est un tableau de bord de tuiles filtrées par permission**, jamais un menu figé ;
 * - **un module d'activité ou une capacité inactifs sont ABSENTS de l'interface** — pas grisés,
 *   pas accompagnés d'un « disponible dans votre offre ». Absents.
 *
 * Le grisé est le réflexe naturel, et c'est celui que le principe VII interdit : il apprend à
 * l'utilisateur qu'une partie du produit lui est refusée, à chaque écran, tous les jours.
 */

/** Permissions cumulées d'un utilisateur — union de tous ses rôles. */
export type Permissions = readonly string[]

/** L'utilisateur détient-il cette permission ? */
export function detient(permissions: Permissions, permission: string): boolean {
  return permissions.includes(permission)
}

/**
 * Union des permissions de plusieurs rôles.
 *
 * Exposée dès maintenant parce que la faute qu'elle évite — prendre les permissions d'un rôle
 * « principal » — se commet à l'écriture du premier écran, pas au cycle CPT.
 */
export function cumuler(...rolesPermissions: Permissions[]): Permissions {
  return [...new Set(rolesPermissions.flat())]
}
