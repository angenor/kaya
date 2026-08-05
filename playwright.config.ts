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
 * ⚠️ **`env` ET `reuseExistingServer` SE CONTREDISENT, ET CE FICHIER LES DOCUMENTAIT TOUS DEUX
 * SANS LE VOIR.** Les deux paragraphes ci-dessus étaient exacts séparément et faux ensemble :
 * `webServer.env` ne s'applique **que si Playwright démarre le serveur**. Quand il en réutilise
 * un — le cas ordinaire sur un poste de développement —, la variable est ignorée **en silence**,
 * et le verdict de la porte la plus chère du projet dépendait de qui avait lancé le serveur.
 *
 * Le correctif n'est pas `reuseExistingServer: false`, qui coûterait un redémarrage de Nuxt à
 * chaque exécution locale. C'est `exigerStyleguideServi()`, dans le `beforeAll` de
 * `parcours-reel.spec.ts` : il **constate** l'état réel du serveur exercé et échoue en disant le
 * geste — arrêter par port, ou relancer avec la variable. Une porte qui refuse doit dire quoi
 * faire, pas seulement ce qui ne va pas.
 *
 * # DEUX moteurs, parce que la cible est Tauri — et Tauri n'embarque pas Chromium
 *
 * Tauri v2 n'embarque **aucun** navigateur : il utilise le moteur du système. Le tableau de
 * `tests-e2e/parcours-reel.spec.ts` en donne la correspondance exacte, et elle décide de ce qui est
 * couvert. Sur trois cibles du produit — macOS, iOS, Linux — le moteur est WebKit. Chromium seul
 * validait donc le moteur que le produit **n'utilise pas** sur la majorité de ses cibles, à
 * commencer par le poste de développement.
 *
 * Les deux projets exécutent **les mêmes tests**, sans exclusion : un cas qui tomberait sous WebKit
 * est un défaut du produit que la coquille Tauri rencontrera, pas un test à exclure.
 *
 * # La limite qui reste, et il faut la connaître
 *
 * **Le WebKit de Playwright n'est pas WKWebView.** C'est une construction de WebKit maintenue par
 * l'équipe Playwright, plus proche de la cible que Chromium et **pas identique** : elle ne porte ni
 * les réglages du composant système d'Apple, ni son intégration au processus hôte. Le vrai contrôle
 * macOS et iOS viendra avec la coquille Tauri elle-même. Écrit ici pour qu'un rapport vert ne se
 * lise pas comme « vérifié sur la cible ».
 *
 * # Deux projets doublent les connexions — et le limiteur les compte
 *
 * `LimiteTentatives` plafonne à **dix tentatives par identifiant sur cinq minutes**, réussies
 * comprises. Chaque projet ouvre **une** session dans son `beforeAll` : une exécution complète en
 * consomme deux, le test négatif deux de plus. Quatre passages rapprochés de la porte butent donc
 * sur le seuil, et le refus est **indiscernable d'un mot de passe faux** (FR-012) — le symptôme
 * serait « la porte ne sait plus se connecter » sans autre indice. Attendre cinq minutes suffit.
 */

import { defineConfig, devices } from '@playwright/test'

/**
 * Port du serveur Nuxt exercé par la porte. **3000 par défaut**, ce qui laisse la CI et le poste
 * de développement inchangés.
 *
 * `KAYA_PORT_E2E` existe pour un cas rencontré au cycle 004 : un serveur Nuxt d'un **autre projet**
 * occupait déjà le 3000. Nuxt se rabat alors silencieusement sur 3001, Playwright continue
 * d'attendre sur 3000, et la porte échoue au bout de trois minutes sur `Timed out waiting
 * 180000ms` — un message qui ne dit rien de la cause et envoie chercher un défaut dans
 * l'application.
 *
 * Changer de port ne change rien à ce que la porte vérifie : elle exerce les mêmes routes, sur les
 * mêmes moteurs, dans les mêmes thèmes.
 */
const PORT = Number(process.env.KAYA_PORT_E2E ?? 3000)
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

  // Les deux moteurs, dans l'ordre où ils couvrent le produit. `fullyParallel: false` et
  // `workers: 1` les enchaînent : ils partagent la base, le compte de démonstration et le
  // compteur de tentatives, et les croiser produirait des échecs qui dépendent de
  // l'ordonnancement.
  projects: [
    { name: 'chromium', use: { ...devices['Desktop Chrome'] } },
    { name: 'webkit', use: { ...devices['Desktop Safari'] } },
  ],

  webServer: {
    // `--port` explicite : sans lui, Nuxt choisit le port suivant quand le sien est pris, et
    // Playwright attend sur une adresse que personne ne sert.
    command: `pnpm --filter @kaya/app dev --port ${PORT}`,
    url: `${BASE}/connexion`,
    reuseExistingServer: true,
    timeout: 180_000,
    env: { KAYA_STYLEGUIDE: '1' },
  },
})
