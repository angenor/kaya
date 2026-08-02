/**
 * **Le thème, posé avant le premier rendu.** *Point d'amorçage n° 1.*
 *
 * # Ce que ce fichier corrige
 *
 * `initialiserTheme()` existait depuis le cycle 001, exportée, documentée « à appeler au démarrage
 * de l'application » — et **appelée nulle part**. Le produit n'a donc jamais appliqué la classe
 * `.dark`, alors que `theme.css` la sert, que `theme-sombre.spec.ts` vérifie que chaque jeton
 * employé porte une valeur sous `.dark`, et que la revue du cycle 003 cochait le mode sombre.
 * Trois contrôles verts sur un mécanisme que rien n'activait.
 *
 * La règle qui en découle vaut au-delà du thème : **une unité écrite n'est pas une unité
 * branchée**, et le test qui interroge l'unité ne dit rien du branchement.
 *
 * # Pourquoi un plugin, et pas un `onMounted` quelque part
 *
 * `entry.js` de Nuxt résout **tous** les plugins (`await applyPlugins`) avant `vueApp.mount()` :
 * un plugin est le dernier endroit qui s'exécute avant le premier rendu de composant. Un
 * `onMounted` dans une page n'amorcerait que cette page — c'est exactement le défaut que ce volet
 * répare, et le reproduire ici serait le déplacer.
 *
 * Le suffixe `.client` est **obligatoire** : ce module touche `document` et `localStorage`. Le
 * préfixe `01.` fixe l'ordre — les plugins sont chargés triés par nom de fichier, et le thème doit
 * précéder tout ce qui pourrait peindre.
 *
 * # Ce plugin ne suffit PAS à lui seul, et c'est écrit ici plutôt que découvert
 *
 * En SPA (`ssr: false`), la coquille HTML est analysée et `body { background-color: var(--color-bg) }`
 * appliqué **avant** que ce chunk ne s'exécute. Un plugin, même en `enforce: 'pre'`, arrive après
 * le premier pixel. Le script en ligne du `<head>` (voir `nuxt.config.ts`, bloc `app.head`) pose la
 * classe plus tôt ; ce plugin repasse derrière et **fait foi** — `initialiserTheme()` est
 * idempotente, la relire ne coûte rien et garantit que la source unique reste ce module.
 */

import { initialiserTheme } from '~/core/theme'

export default defineNuxtPlugin({
  name: 'kaya:theme',
  enforce: 'pre',
  setup() {
    initialiserTheme()
  },
})
