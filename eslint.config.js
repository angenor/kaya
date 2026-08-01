/**
 * **Porte P-15** — aucune invocation de `window.__TAURI__` hors de `PlatformAdapter`.
 *
 *     pnpm porte:p15
 *
 * # Pourquoi une règle de lint et pas une revue
 *
 * Un composant qui importe `@tauri-apps/api` fonctionne parfaitement — sur desktop. Il casse sur
 * le web, et il rend impossible d'ajouter iOS sans rouvrir ce composant. L'erreur ne se voit donc
 * pas au moment où on l'écrit, mais au cycle suivant, sur une autre plateforme, dans un fichier
 * que personne ne relit plus.
 *
 * # Pourquoi cette configuration est à la RACINE, et non dans `app/`
 *
 * Elle y était, et **`web/` n'était couvert par rien**. Ce n'est pas un oubli anodin : `web/qr` et
 * `web/console` sont les deux surfaces **hors Tauri** du produit — une page publique ouverte par un
 * client sur son propre téléphone, et une console éditeur. Elles ne doivent JAMAIS importer
 * `@tauri-apps/api`, sous aucune dérogation. La porte y était donc plus critique qu'ailleurs, et
 * c'est là qu'elle ne regardait rien.
 *
 * Une configuration par paquet aurait dupliqué les règles, donc laissé les copies diverger — et la
 * copie qui dérive est toujours celle qu'on ne lit pas. Une seule configuration, une seule
 * déclaration de version d'ESLint, une seule commande.
 *
 * # PÉRIMÈTRE INSPECTÉ — ce que la porte lit, et ce qu'elle ne lit pas
 *
 * **Elle lit** : `app/**` et `web/**`, en `.js`, `.ts` et `.vue`.
 *
 * **Elle ne lit pas**, et c'est délibéré :
 *
 * - `clients/ts/types.gen.ts` — **généré** depuis le contrat OpenAPI (principe I(a), porte P-01).
 *   Un linter n'a rien à dire d'un fichier que personne n'écrit, et le signaler pousserait à
 *   l'éditer à la main, ce que P-01 refuse.
 * - `backend/` — c'est du Rust ; ses portes sont `cargo clippy` et les tests d'architecture.
 * - `docs/design/html/` — les maquettes. Autonomes, non sémantiques, jamais copiées vers `app/`
 *   (principe XII, porte P-19).
 * - les répertoires produits : `node_modules`, `.nuxt`, `.output`, `dist`, `src-tauri`.
 *
 * **La couverture est vérifiée, pas supposée** : `scripts/ci/pont-natif-confine.sh` compte les
 * fichiers réellement analysés par arbre et échoue si `web/qr` ou `web/console` en compte zéro.
 * C'est l'exigence 4 de « Couverture des portes » — une porte dont la cible est vide passe
 * toujours, et c'est exactement ce qui est arrivé ici au cycle 002.
 */

import js from '@eslint/js'
import vue from 'eslint-plugin-vue'
import ts from 'typescript-eslint'

export default [
  {
    ignores: [
      '**/node_modules/**',
      '**/.nuxt/**',
      '**/.output/**',
      '**/dist/**',
      'app/src-tauri/**',
      // Rust, documentation, infrastructure et spécifications : hors du champ d'un linter JS.
      'backend/**',
      'docs/**',
      'infra/**',
      'specs/**',
      '.specify/**',
      // **Généré depuis le contrat OpenAPI**, jamais écrit à la main (principe I(a)).
      'clients/**',
    ],
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

      // **Une collision de nom qui a réellement coûté un défaut muet.**
      //
      // Dans `SectionServices.vue`, la fonction `basculer()` recevait le retour du service dans une
      // variable nommée `resultat` — le nom exact du `ref` du bandeau déclaré au-dessus. Le `ref`
      // était masqué : `resultat.value = { … }` posait une propriété sur un objet ordinaire, et
      // **le bandeau ne s'affichait jamais**. Aucune erreur, aucun avertissement, aucun test rouge
      // — les tests regardaient chaque bandeau séparément, et chacun était correct.
      //
      // Vérifié avant d'écrire ces deux lignes : `js.configs.recommended`,
      // `ts.configs.recommended` et `vue.configs['flat/recommended']` ne contiennent **aucune** des
      // deux règles. Rien n'aurait attrapé le cas.
      //
      // La règle de base est éteinte au profit de celle de TypeScript, qui sait ne pas confondre un
      // type et une valeur du même nom — sans quoi chaque `interface Contexte` accompagnée d'un
      // `const contexte` produirait un faux positif, et la règle finirait désactivée.
      //
      // **Elle vaut pour `web/` aussi**, et c'est nouveau : les deux surfaces publiques n'avaient
      // aucune configuration. Le défaut de `SectionServices.vue` n'a rien de propre à `app/`.
      'no-shadow': 'off',
      '@typescript-eslint/no-shadow': 'error',

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
    // **Dérogation nommée, et strictement limitée.** `app/core/platform/` est l'unique répertoire
    // autorisé à toucher au pont natif — c'est sa raison d'être.
    //
    // Le préfixe `app/` compte : sans lui, un futur `web/console/core/platform/` hériterait de la
    // dérogation, et la surface la plus exposée du produit serait la seule à pouvoir importer
    // Tauri. Le motif est ancré, pas glissant.
    files: ['app/core/platform/**/*.ts'],
    rules: {
      'no-restricted-imports': 'off',
      'no-restricted-globals': 'off',
      'no-restricted-properties': 'off',
    },
  },

  {
    // **`web/` est HORS Tauri, et aucune dérogation n'y est concevable.**
    //
    // `web/qr` est ouverte par un client sur son propre téléphone, sans compte et sans application
    // installée ; `web/console` est la console éditeur, servie par un navigateur. Ni l'une ni
    // l'autre ne s'exécute dans une coquille Tauri : un import de `@tauri-apps/api` n'y casse pas
    // « plus tard, sur une autre plateforme » — il casse **tout de suite, pour tout le monde**.
    //
    // Le bloc redit donc l'interdiction avec son motif propre. Il ne change pas la règle : il fait
    // que le message dise la bonne chose au bon endroit, et il rend visible qu'aucune dérogation
    // ne s'applique ici.
    files: ['web/**/*.{js,ts,vue}'],
    rules: {
      'no-restricted-imports': [
        'error',
        {
          paths: [
            {
              name: '@tauri-apps/api',
              message:
                'web/qr et web/console sont des surfaces HORS Tauri — page publique ouverte sans '
                + 'application installée, console éditeur servie par un navigateur. Le pont natif '
                + "n'y existe pas : l'import échoue à l'exécution, pour tous les visiteurs. "
                + 'Aucune dérogation ici (porte P-15).',
            },
          ],
          patterns: [
            {
              group: ['@tauri-apps/api/*', '@tauri-apps/plugin-*'],
              message: 'Même règle : aucune dépendance Tauri dans les surfaces web.',
            },
          ],
        },
      ],
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
    // Scripts de porte, tests et fichiers de configuration : Node, pas navigateur.
    //
    // Les motifs sont ancrés sur `**/` plutôt que sur la racine : `app/scripts/`, `app/tests/` et
    // `web/qr/nuxt.config.ts` doivent tous être couverts depuis cette configuration, qui vit un
    // niveau au-dessus d'eux.
    files: ['**/scripts/**/*.ts', '**/tests/**/*.ts', '**/*.config.ts'],
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
