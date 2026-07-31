import tailwindcss from '@tailwindcss/vite'

// Page publique de commande par QR — **coquille du cycle QRC**.
//
// `ssr: true` est ici l'inverse du choix de `app/`, et pour une raison précise : cette page est
// ouverte par un client sur son propre téléphone, sans compte, sans application installée et
// souvent sur un réseau lent. Le rendu serveur est ce qui la rend utilisable dans ces
// conditions ; l'application métier, elle, doit fonctionner sans serveur joignable.
export default defineNuxtConfig({
  compatibilityDate: '2026-07-30',
  ssr: true,
  vite: { plugins: [tailwindcss()] },
  devtools: { enabled: false },
})
