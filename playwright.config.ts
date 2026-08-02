/**
 * **Configuration de la porte P-22 — parcours réel.**
 *
 * # Ce que cette configuration démarre, et pourquoi elle le démarre elle-même
 *
 * P-22 exige que l'**application** s'ouvre, pas qu'un composant se monte. Cela suppose l'API, la
 * base et le serveur de développement. Les laisser à la charge de qui lance la porte produirait
 * l'échec le plus coûteux qui soit : une porte rouge pour une raison d'environnement, qu'on finit
 * par ignorer — et une porte qu'on ignore ne garde rien.
 *
 * `webServer` démarre donc Nuxt, et `reuseExistingServer` laisse la main à un serveur déjà lancé
 * en développement. **L'API n'est PAS démarrée ici** : elle a besoin de Postgres, de Redis et des
 * seeds, c'est-à-dire de `scripts/dev/preparer-base.sh`. Le script de la porte s'en charge et
 * échoue avec un message qui dit quoi faire.
 *
 * # `KAYA_STYLEGUIDE=1` — la route du styleguide DOIT exister pendant la porte
 *
 * P-22 couvre `/styleguide` (voir la décision écrite dans `tests-e2e/parcours-reel.spec.ts`). Sans
 * cette variable, `pages:extend` retire la route et la porte constaterait une 404 — ou pire,
 * l'exclurait. Une porte qui saute une page pour passer au vert est le défaut que le
 * § « Couverture des portes » nomme.
 *
 * # Un seul navigateur, et c'est déclaré
 *
 * Chromium. Un défaut propre à WebKit passerait — limite assumée, écrite ici plutôt que supposée.
 * L'application cible est une coquille Tauri, dont le moteur varie par plateforme ; couvrir WebKit
 * relèvera du cycle qui construit la coquille.
 */

import { defineConfig, devices } from '@playwright/test'

const PORT = 3000
const BASE = `http://localhost:${PORT}`

export default defineConfig({
  testDir: './tests-e2e',
  // Séquentiel. Les tests partagent une base de données et un compte de démonstration ; les
  // paralléliser produirait des échecs qui dépendent de l'ordonnancement — exactement ce qui
  // fait désactiver une porte.
  fullyParallel: false,
  workers: 1,
  // Aucun réessai. Un test qui ne passe qu'au second essai cache un défaut de synchronisation, et
  // le masquer ici le laisserait atteindre la production.
  retries: 0,
  reporter: [['list']],
  timeout: 60_000,
  expect: { timeout: 10_000 },

  use: {
    baseURL: BASE,
    trace: 'retain-on-failure',
    screenshot: 'only-on-failure',
    // **La locale est fixée, et ce n'est pas un détail de confort.** Playwright démarre Chromium
    // en `en-US` ; `@nuxtjs/i18n` détecte la langue du navigateur et sert l'anglais, alors que le
    // produit a le français par défaut (principe VIII). Sans cette ligne, la porte cherche
    // « Identifiant » dans une page qui affiche « ID » et échoue sur un faux motif — le pire
    // genre d'échec, parce qu'il donne l'air d'un défaut du produit.
    //
    // La fixer ici ne masque rien : la parité fr/en est vérifiée par P-16, qui compare les deux
    // catalogues clé à clé. P-22 vérifie qu'une page s'ouvre, dans une langue à la fois.
    locale: 'fr-FR',
  },

  projects: [
    { name: 'chromium', use: { ...devices['Desktop Chrome'] } },
  ],

  webServer: {
    command: 'pnpm --filter @kaya/app dev',
    url: `${BASE}/connexion`,
    reuseExistingServer: true,
    timeout: 180_000,
    env: { KAYA_STYLEGUIDE: '1' },
  },
})
