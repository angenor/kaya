import tailwindcss from '@tailwindcss/vite'

import { ROUTE_STYLEGUIDE, styleguideMonte, VARIABLE_STYLEGUIDE } from './core/design-system/montage'

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
  //
  // `polices.css` et `icones.css` sont **générés** par `scripts/generer-polices.ts` et
  // `scripts/generer-icones.ts`, et servent en local ce que les maquettes chargent depuis Google
  // Fonts et un CDN (portes P-21 et P-21b). Une police qui dépend du réseau est un écran qui
  // dépend du réseau, ce que le principe VI interdit — et sans Chivo Mono embarquée, les colonnes
  // de montants se désalignent (`tokens.md` §2).
  //
  // `polices.css` vient AVANT `theme.css` : les `@font-face` doivent être déclarés quand le bloc
  // `@theme` nomme les familles dans `--font-titre`, `--font-texte` et `--font-mono`.
  css: ['~/assets/css/polices.css', '~/assets/css/theme.css', '~/assets/css/icones.css'],

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

  // **Aucune valeur d'environnement codée en dur dans un composant.** Les trois dernières sont
  // le provisoire de contexte ouvert au cycle 001 (`CONTEXTE_PAR_EN_TETES`) : elles disparaissent
  // avec CPT-01, qui livrera l'authentification par jeton. Les laisser ici, nommées et groupées,
  // rend visible ce qu'il y aura à retirer — un identifiant dispersé dans cinq composants ne se
  // retrouverait pas.
  runtimeConfig: {
    public: {
      apiBaseUrl: 'http://localhost:8080',
      tenantId: '',
      compteId: '',
      etablissementId: '',
      // **Provisoire nommé, levé par CPT-02.** Permissions cumulées de l'utilisateur, séparées par
      // des virgules. Vide par défaut : sans permission, l'écran se rend en lecture seule et
      // aucune action n'apparaît — le comportement sûr, et celui qui rend la règle observable.
      // Un défaut « tout permis » masquerait le principe VII jusqu'au cycle CPT.
      permissions: '',
    },
  },

  typescript: {
    strict: true,
    typeCheck: false,
  },

  devtools: { enabled: false },

  // **Le styleguide est RETIRÉ du routeur hors développement** — même mécanisme que la Swagger UI
  // du cycle 001 (`backend/api/src/application.rs`, `swagger_ui_activee()`), et pour la même raison :
  // « une route non montée ne peut pas fuir par oubli de garde ; une route montée derrière un `if`
  // finit toujours par être atteinte par un chemin qu'on n'avait pas prévu ».
  //
  // Ce n'est pas qu'une question d'accès : la page porte des libellés d'échantillon **en clair**,
  // au titre d'une exemption nommée de la porte P-16. C'est sa non-présence en production qui rend
  // cette exemption acceptable — et `scripts/test-i18n.ts` vérifie la réciproque, à savoir que le
  // fichier exempté est bien celui qui est retiré ici.
  //
  //     KAYA_STYLEGUIDE=1 pnpm --filter @kaya/app dev
  hooks: {
    'pages:extend': (pages) => {
      if (styleguideMonte(process.env)) {
        // Bruyant à dessein, comme le `tracing::warn!` du backend : attendu en développement,
        // jamais en production.
        console.warn(
          `Le styleguide est monté sur ${ROUTE_STYLEGUIDE} — attendu en développement uniquement `
          + `(${VARIABLE_STYLEGUIDE}).`,
        )
        return
      }
      const rang = pages.findIndex(page => page.path === ROUTE_STYLEGUIDE)
      if (rang !== -1) pages.splice(rang, 1)
    },
  },
})
