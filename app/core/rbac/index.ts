/**
 * Permissions — **le provisoire du cycle 002 est levé** (CPT-02).
 *
 * # La règle qui décide de toute l'interface
 *
 * **Les rôles sont CUMULABLES, et c'est la norme, pas l'exception** (principe VII). Un gérant est
 * aussi caissier et réceptionniste ; ses permissions sont l'**union** de celles de ses rôles,
 * jamais l'intersection ni celles d'un rôle « principal ».
 *
 * Deux conséquences que tout écran tient :
 *
 * - **l'accueil est un tableau de bord de tuiles filtrées par permission**, jamais un menu figé ;
 * - **un module d'activité ou une capacité inactifs sont ABSENTS de l'interface** — pas grisés,
 *   pas accompagnés d'un « disponible dans votre offre ». Absents.
 *
 * Le grisé est le réflexe naturel, et c'est celui que le principe VII interdit : il apprend à
 * l'utilisateur qu'une partie du produit lui est refusée, à chaque écran, tous les jours.
 *
 * # D'où viennent les permissions maintenant
 *
 * Elles ne viennent plus de la configuration en liste séparée par des virgules. Le serveur calcule
 * **l'union des rôles portés sur l'établissement actif** et la rend dans le corps de la réponse de
 * connexion (`SessionOuverteVue.permissions`, FR-017). Le front la lit là, **jamais en décodant le
 * jeton** : deux sources pour la même information, et une seule fait autorité (research R-06).
 *
 * `core/auth` la range dans la session ; `sessionCourante()?.permissions` est le seul point où un
 * écran la prend.
 *
 * # Ce que `cumuler` fait encore ici, alors que l'union se calcule côté serveur
 *
 * Elle n'est plus sur le chemin de la connexion, et c'est exact. Elle reste exposée parce que la
 * faute qu'elle évite — prendre les permissions d'un rôle « principal » — se commet à chaque
 * écran qui compose deux ensembles, et le premier viendra avec le sélecteur d'établissement
 * d'ETB-06. La retirer maintenant obligerait à la réécrire, probablement moins bien.
 */

/** Permissions cumulées d'un utilisateur — union de tous ses rôles. */
export type Permissions = readonly string[]

/**
 * L'utilisateur détient-il cette permission ?
 *
 * **Comparaison par égalité, jamais par préfixe.** `cpt.compte.lire` n'ouvre pas
 * `cpt.compte.gerer` : une hiérarchie par préfixe paraît élégante et rendrait `cpt.` équivalent à
 * tout. Le serveur applique exactement la même règle (`api/src/securite.rs`), et les deux se
 * répondent — un front plus permissif que le serveur afficherait des actions qui échouent, un
 * front plus strict cacherait des actions permises.
 */
export function detient(permissions: Permissions, permission: string): boolean {
  return permissions.includes(permission)
}

/** L'utilisateur détient-il **au moins une** de ces permissions ? */
export function detientUne(permissions: Permissions, requises: readonly string[]): boolean {
  return requises.some(requise => detient(permissions, requise))
}

/**
 * Union des permissions de plusieurs rôles.
 *
 * L'ordre du résultat suit celui de la première apparition ; aucun appelant n'en dépend, et un tri
 * ici masquerait un appelant qui en dépendrait à tort.
 */
export function cumuler(...rolesPermissions: Permissions[]): Permissions {
  return [...new Set(rolesPermissions.flat())]
}
