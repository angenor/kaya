/**
 * **Le styleguide est-il monté ?**
 *
 * # Le même mécanisme que la Swagger UI du cycle 001, et pour la même raison
 *
 * `backend/api/src/application.rs`, `swagger_ui_activee()` : « **Décision de configuration au
 * démarrage, jamais un test de variable dispersé dans les handlers.** Une route non montée ne peut
 * pas fuir par oubli de garde ; une route montée derrière un `if` finit toujours par être atteinte
 * par un chemin qu'on n'avait pas prévu. »
 *
 * Le styleguide pose exactement le même problème : c'est une surface de **développement**, qui
 * affiche les seize composants dans tous leurs états, avec des libellés d'échantillon écrits en
 * clair et sans i18n. Une page cachée derrière un `v-if` reste dans le fragment de production,
 * atteignable par quiconque tape son adresse. Elle est donc **retirée du routeur** au moment de la
 * construction (`nuxt.config.ts`, hook `pages:extend`) : sans route, Nuxt ne compile même pas le
 * fichier.
 *
 * # Pourquoi l'environnement est un paramètre, et non `process.env` lu ici
 *
 * Deux raisons, dans cet ordre :
 *
 * 1. **Testabilité.** Une fonction qui lit l'environnement global se teste en le mutant, ce qui
 *    fuit d'un test à l'autre. Celle-ci se teste en lui passant trois objets.
 * 2. `process` n'existe pas dans un module de `core/` du point de vue d'ESLint — et c'est correct :
 *    ces modules sont compilés dans un binaire Tauri, où il n'y a pas de processus Node. Le seul
 *    appelant est `nuxt.config.ts`, qui s'exécute, lui, à la construction.
 */

/** L'adresse de la page. Nommée ici pour que le retrait de route et la page ne divergent jamais. */
export const ROUTE_STYLEGUIDE = '/styleguide'

/**
 * La variable d'environnement qui monte le styleguide.
 *
 * Même convention que `KAYA_SWAGGER_UI` : le préfixe `KAYA_`, et deux valeurs acceptées seulement.
 */
export const VARIABLE_STYLEGUIDE = 'KAYA_STYLEGUIDE'

/**
 * Le styleguide est-il monté ?
 *
 * **Faux par défaut**, comme la Swagger UI. Une valeur absente, vide, `0`, `false` ou quoi que ce
 * soit d'autre laisse la page hors du routeur : c'est le sens de la garde. Un défaut « monté sauf
 * si » ferait dépendre la production d'une variable correctement posée sur chaque déploiement.
 */
export function styleguideMonte(env: Readonly<Record<string, string | undefined>>): boolean {
  const valeur = env[VARIABLE_STYLEGUIDE]
  return valeur === '1' || valeur === 'true'
}
