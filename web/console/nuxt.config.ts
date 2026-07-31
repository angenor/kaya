import tailwindcss from '@tailwindcss/vite'

// Console éditeur — **coquille du cycle EDT**.
//
// Surface distincte de `app/` : son public est l'éditeur, pas l'établissement. Les mêler
// mettrait des écrans de parc et de télémétrie derrière la même authentification que la caisse.
export default defineNuxtConfig({
  compatibilityDate: '2026-07-30',
  ssr: false,
  vite: { plugins: [tailwindcss()] },
  devtools: { enabled: false },
})
