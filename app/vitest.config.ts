import { fileURLToPath } from 'node:url'

import vue from '@vitejs/plugin-vue'
import { defineConfig } from 'vitest/config'

// Tests de l'application.
//
// **Ils ne montent pas Nuxt**, et c'est délibéré : monter le framework pour vérifier qu'un
// composant n'affiche pas un service inactif ferait dépendre le test du pipeline entier, avec
// ses auto-imports et son routeur. Ce qui est testé ici est du TypeScript pur — file de classe A,
// adaptateur de plateforme, sélection des services visibles — plus le **rendu** des composants de
// `G1`, monté directement par `@vue/test-utils`.
//
// L'environnement reste `node` par défaut : les tests qui ont besoin d'un DOM le déclarent
// fichier par fichier avec `// @vitest-environment happy-dom`. Basculer tout le monde sous un DOM
// coûterait à chaque test celui qui n'en a pas besoin.
export default defineConfig({
  plugins: [vue()],
  // L'alias `~` est fourni par Nuxt, que ces tests ne montent pas. Sans lui, tout composant qui
  // importe une coquille transverse — `~/core/rbac`, `~/core/platform/reseau` — échoue à se
  // résoudre, et le fichier de test entier tombe **avant d'exécuter la moindre assertion**. Un
  // fichier qui ne s'exécute pas est indistinguable d'un fichier qui passe.
  resolve: {
    alias: { '~': fileURLToPath(new URL('.', import.meta.url)) },
  },
  test: {
    include: ['tests/**/*.spec.ts'],
    environment: 'node',
    typecheck: {
      // **C'est ce réglage qui rend la porte P-13 opposable.** Sans lui, les `@ts-expect-error`
      // du test ne seraient jamais évalués : Vitest transpile sans vérifier les types, et une
      // charge non marquée passerait sans que rien ne le signale.
      enabled: true,
      include: ['tests/**/*.spec.ts'],
      tsconfig: './tsconfig.test.json',
    },
  },
})
