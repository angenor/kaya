import tailwindcss from '@tailwindcss/vite'

// Application unique Kaya — Nuxt 4, mode SPA sous Tauri v2.
//
// Trois réglages portent chacun un principe, et aucun n'est un choix de confort.

export default defineNuxtConfig({
  compatibilityDate: '2026-07-30',

  // L'arborescence `app/core/`, `app/modules/`, `app/assets/` est **imposée** par
  // `plan.md` § Project Structure et la constitution, pas choisie ici. Deux réglages la
  // réconcilient avec les conventions de Nuxt 4 :
  //
  //   `srcDir: '.'`        — sans lui, Nuxt chercherait les sources dans `app/app/`, puisque
  //                          son `srcDir` par défaut est déjà `app/` relatif à la racine du
  //                          paquet. Le paquet *est* `app/`.
  //   `dir.modules`        — `modules/` porte ici les **modules métier** (etablissements,
  //                          hebergement…), pas des extensions Nuxt. Sans ce renommage, Nuxt
  //                          tenterait de charger `modules/etablissements/index.ts` comme une
  //                          extension du framework le jour où ce fichier existera.
  srcDir: '.',
  dir: {
    modules: 'nuxt-extensions',
  },

  // SPA. L'application tourne sous Tauri, sur un poste ou un téléphone, souvent sans réseau
  // (principe VII). Un rendu serveur supposerait un serveur joignable — l'hypothèse exacte que
  // le mode hors-ligne interdit. Les deux surfaces publiques de `web/` font l'inverse quand
  // c'est justifié : `web/qr` est en SSR parce qu'un client l'ouvre sans rien installer.
  ssr: false,

  // Le thème vient d'un unique fichier CSS — la copie exacte de `docs/design/theme.css`, seule
  // exception du principe XII. Aucun token n'est redéclaré ici : une seconde source dériverait.
  css: ['~/assets/css/theme.css'],

  vite: {
    plugins: [tailwindcss()],
  },

  modules: ['@nuxtjs/i18n'],

  i18n: {
    // **fr par défaut** (principe VIII). L'anglais existe dès le premier jour, pas en
    // rétrofit : ajouter l'i18n après coup coûte plusieurs fois son prix initial.
    defaultLocale: 'fr',
    strategy: 'no_prefix',
    // Sans ce réglage, `@nuxtjs/i18n` cherche ses catalogues sous `<srcDir>/i18n/`. Or
    // l'arborescence imposée les place dans `app/core/i18n/`, avec `auth`, `rbac`, `theme`,
    // `sync` et `platform` — les six coquilles transverses au même niveau. Ramener la racine à
    // `srcDir` est ce qui réconcilie la convention du module avec la structure du plan.
    restructureDir: '.',
    locales: [
      { code: 'fr', language: 'fr-FR', name: 'Français', file: 'fr.json' },
      { code: 'en', language: 'en-US', name: 'English', file: 'en.json' },
    ],
    langDir: 'core/i18n',
    bundle: { optimizeTranslationDirective: false },
  },

  // Mode sombre : la classe `.dark` sur `<html>`, consommée par le `@custom-variant dark` de
  // `theme.css`. **Jamais une seconde palette** (principe XII) — les noms de tokens sont
  // identiques en clair et en sombre, seules les valeurs changent sous `.dark`.
  app: {
    head: {
      htmlAttrs: { lang: 'fr' },
    },
  },

  typescript: {
    strict: true,
    typeCheck: false,
  },

  devtools: { enabled: false },
})
