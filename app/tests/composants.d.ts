/**
 * Déclaration des composants monofichiers pour le **typecheck des tests**.
 *
 * `vitest --typecheck` passe par `tsc`, qui ne sait pas lire un fichier `.vue` — c'est le rôle de
 * `vue-tsc`, absent ici. Sans cette déclaration, l'import de `SectionServices.vue` échoue au
 * typage alors que le test lui-même passe : `pnpm test` sortirait en échec permanent, exactement
 * le défaut que ce cycle vient de corriger côté `@types/node`.
 *
 * Le type est volontairement lâche : ce que les tests vérifient est le **HTML rendu**, pas la
 * signature des props — celle-ci est déjà tenue par `defineProps` dans chaque composant, que le
 * pipeline Nuxt typecheck à la construction.
 */
declare module '*.vue' {
  import type { DefineComponent } from 'vue'

  const composant: DefineComponent<Record<string, unknown>, Record<string, unknown>, unknown>
  export default composant
}
