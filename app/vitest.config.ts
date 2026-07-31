import { defineConfig } from 'vitest/config'

// Tests de l'application. Ils ne montent pas Nuxt : ce cycle ne produit aucun écran, et les
// modules testés — la file de classe A, l'adaptateur de plateforme — sont du TypeScript pur.
export default defineConfig({
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
