/**
 * **Porte P-15** — aucune invocation de `window.__TAURI__` hors de `PlatformAdapter`.
 *
 * # Pourquoi une règle de lint et pas une revue
 *
 * Un composant qui importe `@tauri-apps/api` fonctionne parfaitement — sur desktop. Il casse sur
 * le web, et il rend impossible d'ajouter iOS sans rouvrir ce composant. L'erreur ne se voit donc
 * pas au moment où on l'écrit, mais au cycle suivant, sur une autre plateforme, dans un fichier
 * que personne ne relit plus.
 *
 * La dérogation est **nommée** et limitée au seul répertoire `core/platform/` : c'est là que vit
 * l'adaptateur, et nulle part ailleurs.
 */

import js from '@eslint/js'
import vue from 'eslint-plugin-vue'
import ts from 'typescript-eslint'

export default [
  {
    ignores: ['.nuxt/**', '.output/**', 'node_modules/**', 'dist/**', 'src-tauri/**'],
  },

  js.configs.recommended,
  // Les parseurs TypeScript et Vue ne sont pas un confort : sans eux, ESLint ne sait pas lire les
  // fichiers où la porte P-15 doit chercher. Une porte qui échoue à parser ses cibles est une
  // porte qui ne vérifie rien — pire, elle échoue bruyamment pour une raison sans rapport, et
  // finit désactivée.
  ...ts.configs.recommended,
  ...vue.configs['flat/recommended'],

  {
    files: ['**/*.{js,ts,vue}'],
    languageOptions: {
      ecmaVersion: 2023,
      sourceType: 'module',
    },
    rules: {
      // Les coquilles de `PlatformAdapter` reçoivent des paramètres qu'elles n'utilisent pas
      // encore — c'est leur nature : elles honorent une signature en annonçant que la capacité
      // n'existe pas. Le préfixe `_` est la convention qui le dit ; sans cette ligne, chaque
      // implémentation à venir devrait ajouter une dérogation locale.
      '@typescript-eslint/no-unused-vars': [
        'error',
        { argsIgnorePattern: '^_', varsIgnorePattern: '^_' },
      ],

      'no-restricted-imports': [
        'error',
        {
          paths: [
            {
              name: '@tauri-apps/api',
              message:
                'Le pont natif ne se consomme que par PlatformAdapter (app/core/platform/). '
                + "Un composant qui importe @tauri-apps/api fonctionne sur desktop, casse sur le web, "
                + "et rend impossible l'ajout d'iOS sans le rouvrir (principe VII, porte P-15).",
            },
          ],
          patterns: [
            {
              group: ['@tauri-apps/api/*', '@tauri-apps/plugin-*'],
              message:
                'Même règle : tout le natif passe par PlatformAdapter (app/core/platform/).',
            },
          ],
        },
      ],

      // `window.__TAURI__` contourne l'import et donc la règle ci-dessus. Il faut le viser
      // séparément, sans quoi la porte se franchit en une ligne.
      //
      // **Limite constatée, écrite ici plutôt que découverte plus tard** : `(window as any).__TAURI__`
      // échappe à `no-restricted-properties`, le cast changeant la forme de l'expression dans
      // l'arbre syntaxique. En pratique, `@typescript-eslint/no-explicit-any` l'attrape — mais
      // par un autre chemin, et un `eslint-disable` local suffirait à passer les deux. La revue
      // reste donc nécessaire sur ce cas précis.
      'no-restricted-globals': [
        'error',
        {
          name: '__TAURI__',
          message: 'Passer par PlatformAdapter (app/core/platform/) — porte P-15.',
        },
      ],
      'no-restricted-properties': [
        'error',
        {
          object: 'window',
          property: '__TAURI__',
          message: 'Passer par PlatformAdapter (app/core/platform/) — porte P-15.',
        },
      ],
    },
  },

  {
    // **Dérogation nommée, et strictement limitée.** `core/platform/` est l'unique répertoire
    // autorisé à toucher au pont natif — c'est sa raison d'être.
    files: ['core/platform/**/*.ts'],
    rules: {
      'no-restricted-imports': 'off',
      'no-restricted-globals': 'off',
      'no-restricted-properties': 'off',
    },
  },

  {
    // **Le parseur TypeScript, DANS le bloc `<script>` des composants.**
    //
    // `vue-eslint-parser` lit le gabarit et délègue le script à un sous-parseur — par défaut
    // `espree`, qui ne connaît pas TypeScript. Un composant écrit en `<script setup lang="ts">`
    // produit alors « Parsing error: Unexpected token », et **aucune règle ne s'exécute sur lui**.
    //
    // C'est exactement le défaut que l'en-tête de ce fichier décrit : une porte qui échoue à
    // parser ses cibles ne vérifie rien. Il est resté invisible au cycle 001, dont l'unique
    // composant ne portait aucune annotation de type — le premier écran réel du produit l'a
    // révélé.
    files: ['**/*.vue'],
    languageOptions: {
      parserOptions: {
        parser: ts.parser,
      },
    },
  },

  {
    // Composants Vue sous Nuxt.
    //
    // Deux règles sont désactivées, et c'est un choix, pas une facilité :
    //
    //   `no-undef` — Nuxt **auto-importe** `useI18n`, `defineNuxtConfig`, `ref`… Les signaler
    //   comme indéfinis produirait un bruit permanent qui masquerait les vraies erreurs, dont
    //   celles de la porte P-15.
    //
    //   `vue/multi-word-component-names` — le nom d'un composant de page est imposé par le
    //   routage de fichiers : `pages/index.vue` ne peut pas s'appeler autrement.
    files: ['**/*.vue'],
    rules: {
      'no-undef': 'off',
      'vue/multi-word-component-names': 'off',
    },
  },

  {
    // Scripts de porte et tests : Node, pas navigateur.
    files: ['scripts/**/*.ts', 'tests/**/*.ts', '*.config.ts'],
    languageOptions: {
      globals: {
        process: 'readonly',
        console: 'readonly',
        URL: 'readonly',
      },
    },
    rules: {
      'no-undef': 'off',
    },
  },
]
