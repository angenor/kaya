/**
 * **Porte P-22 — PARCOURS RÉEL.** *L'application démarre et chaque route déclarée s'atteint.*
 *
 * ═══════════════════════════════════════════════════════════════════════════════════════════
 *  POURQUOI CETTE PORTE EXISTE
 * ═══════════════════════════════════════════════════════════════════════════════════════════
 *
 * Le cycle 003 a été livré avec **24 portes vertes, 224 tests backend et 428 tests front** — et
 * deux des quatre écrans du produit, `G3` et `G4`, étaient **inatteignables en navigateur**. Aucun
 * contrôle ne l'a vu, parce qu'aucun ne montait l'application.
 *
 * Les 428 tests front montent les composants avec `@vue/test-utils`. C'est utile et ce n'est pas
 * la même chose : cela contourne le routeur, `<Suspense>`, les layouts et les plugins —
 * c'est-à-dire **tout ce qui fait qu'une page existe pour un utilisateur**. Un composant qui passe
 * ses tests et une page qu'on peut ouvrir sont deux propriétés distinctes ; la seconde n'était
 * vérifiée nulle part.
 *
 * Cette porte rend enfin opposable le **point 8 de la Definition of Done** — « écran vérifié en
 * mode clair et en mode sombre » — qui n'a été coché pour **aucune story depuis le début du
 * projet**, faute d'être vérifiable autrement qu'à la main.
 *
 * ═══════════════════════════════════════════════════════════════════════════════════════════
 *  PÉRIMÈTRE INSPECTÉ — exigence 1 du § « Couverture des portes »
 * ═══════════════════════════════════════════════════════════════════════════════════════════
 *
 * **Inspecté.** Toutes les routes de `app/pages/`, **lues du système de fichiers**, jamais
 * énumérées à la main — voir `routes.ts` et la note qui l'accompagne. Pour chacune, cinq
 * contrôles :
 *
 *   1. **chargement DIRECT de l'adresse** — le chemin qui ne reprenait pas la session ;
 *   2. **navigation INTERNE depuis l'accueil** — le chemin qui vidait le `<main>` ;
 *   3. **aucune erreur de console ni `pageerror`** pendant les deux ;
 *   4. **le `<main>` est toujours dans le DOM** après navigation ;
 *   5. **la classe `.dark` s'applique** et la page reste lisible dans les deux thèmes.
 *
 * **Non inspecté, et écrit ici plutôt que supposé.**
 *
 * - **L'apparence.** Cette porte vérifie qu'une page s'ouvre et qu'elle bascule de thème ; elle ne
 *   dit rien de sa justesse visuelle. Aucune capture n'est comparée. La conformité à la maquette
 *   reste humaine — c'est la même limite assumée que `classes_offline.rs` pour la justesse des
 *   classes hors-ligne.
 * - **Le contenu métier.** Qu'un écran s'affiche ne dit pas qu'il affiche les bonnes lignes. Les
 *   428 tests front couvrent cela, et cette porte ne les remplace pas : elle couvre ce qu'ils ne
 *   peuvent pas voir.
 * - **Les autres navigateurs.** Chromium seul. Un défaut propre à WebKit passerait.
 *
 * ═══════════════════════════════════════════════════════════════════════════════════════════
 *  LE STYLEGUIDE EST COUVERT — décision écrite, pas laissée implicite
 * ═══════════════════════════════════════════════════════════════════════════════════════════
 *
 * `pages/styleguide.vue` est exemptée de **P-16** (littéraux en clair), avec une contrepartie
 * vérifiée : la route est retirée du routeur hors développement.
 *
 * **Elle n'est PAS exemptée de P-22.** Trois raisons, dans cet ordre :
 *
 * 1. Exclure une page pour faire passer une porte est exactement ce que le § « Couverture des
 *    portes » interdit. Une porte dont la cible rétrécit est indistinguable d'une porte qui passe.
 * 2. Le styleguide est **la seule surface** où les seize composants se voient avec les polices
 *    réellement embarquées. S'il cessait de s'ouvrir, on le découvrirait au moment où on en a
 *    besoin.
 * 3. Sa raison d'exemption à P-16 — des libellés d'échantillon en clair — n'a **aucun rapport**
 *    avec le fait de s'ouvrir sans erreur. Une exemption ne se propage pas d'une porte à l'autre.
 *
 * La porte l'exerce donc avec `KAYA_STYLEGUIDE=1`, la variable qui monte sa route.
 *
 * ═══════════════════════════════════════════════════════════════════════════════════════════
 *  UN SEUL CONTEXTE DE NAVIGATEUR, ET C'EST LE PRODUIT QUI L'IMPOSE
 * ═══════════════════════════════════════════════════════════════════════════════════════════
 *
 * Deux mécanismes de sécurité, tous deux corrects, interdisent la manière habituelle de faire :
 *
 * - **`LimiteTentatives` plafonne à dix tentatives par identifiant**, et les compte **avant** de
 *   vérifier le mot de passe, réussies comprises. Un compteur qui ne compterait que les échecs
 *   rétablirait par différence de comportement la fuite que FR-012 referme. Vingt tests qui se
 *   connecteraient chacun seraient refusés à partir du onzième.
 * - **Le jeton de rafraîchissement est à usage unique, avec rotation**, et le rejouer **révoque
 *   toute la famille** — c'est la détection de vol de CPT-01. Un `storageState` recopié dans N
 *   contextes présenterait N fois le même jeton : le premier passerait, les autres feraient
 *   révoquer la session pour tout le monde.
 *
 * D'où **un contexte, une page, une connexion**, partagés par tous les tests. Ce n'est pas un
 * contournement : chaque `page.goto()` perd le jeton d'accès, qui vit en mémoire, et oblige le
 * middleware global à **reprendre la session depuis le jeton rangé** — précisément le chemin qui
 * ne fonctionnait pas avant ce lot. La contrainte a rendu la porte plus exigeante.
 */

import { expect, test, type Browser, type BrowserContext, type Page } from '@playwright/test'

import { COMPTE_DEMONSTRATION, ROUTES, type Route } from './routes'

// Un contexte partagé impose l'ordre : deux tests concurrents sur la même page se marcheraient
// dessus, et l'échec dépendrait de l'ordonnancement — ce qui fait désactiver une porte.
test.describe.configure({ mode: 'serial' })

/** Les erreurs de console tolérées, **nommées une par une, jamais par motif large**. */
const BRUIT_TOLERE: readonly RegExp[] = [
  // Nuxt l'émet sur chaque page en développement. C'est une information du framework sur une
  // fonctionnalité expérimentale de Vue, pas un défaut du produit.
  /<Suspense> is an experimental feature/,
  // Vite en mode développement. Absent de la construction de production.
  /\[vite\] (connecting|connected)/,
]

let contexte: BrowserContext
let page: Page
/** Rempli par le collecteur, vidé avant chaque contrôle. */
let erreurs: string[] = []

test.beforeAll(async ({ browser }: { browser: Browser }) => {
  contexte = await browser.newContext({
    locale: 'fr-FR',
    baseURL: 'http://localhost:3000',
  })
  page = await contexte.newPage()

  page.on('console', (message) => {
    if (message.type() !== 'error' && message.type() !== 'warning') return
    const texte = message.text()
    if (!BRUIT_TOLERE.some(motif => motif.test(texte))) {
      erreurs.push(`console.${message.type()}: ${texte}`)
    }
  })
  page.on('pageerror', erreur => erreurs.push(`pageerror: ${erreur.message}`))

  // **La connexion, une fois, par le vrai formulaire.** Poser un jeton forgé dans le stockage
  // irait plus vite et ne prouverait rien : c'est le raisonnement d'`isolation_tenant.rs`, dont
  // les requêtes obtiennent leur jeton par `session_ouvrir` plutôt que par la clé de signature.
  await page.goto('/connexion')
  await page.getByLabel(/identifiant/i).fill(COMPTE_DEMONSTRATION.identifiant)
  await page.getByLabel(/mot de passe/i).fill(COMPTE_DEMONSTRATION.motDePasse)
  await page.getByRole('button', { name: /se connecter/i }).click()
  await page.waitForURL(url => new URL(url).pathname === '/', { timeout: 20_000 })

  const range = await page.evaluate(() => localStorage.getItem('kaya.auth.rafraichissement'))
  expect(
    range,
    'le jeton de rafraîchissement n’est pas dans le stockage après connexion.\n'
    + 'Tous les contrôles de P-22 en dépendent : sans lui, chaque chargement direct repartirait '
    + 'd’un stockage vide et la porte ne mesurerait plus que la redirection vers R0.',
  ).toBeTruthy()
})

test.afterAll(async () => {
  await contexte?.close()
})

test.beforeEach(() => {
  erreurs = []
})

// =================================================================================================
//  Exigence 4 — la cible n'est pas vide
// =================================================================================================

test('P-22 · la cible n’est pas vide — les routes sont LUES, pas énumérées', () => {
  expect(
    ROUTES.length,
    'aucune route lue de app/pages/ : la porte n’inspecterait rien, et passerait au vert.\n'
    + 'Une porte dont la cible est vide est indistinguable d’une porte qui passe.',
  ).toBeGreaterThan(0)

  const chemins = ROUTES.map(r => r.chemin)
  expect(new Set(chemins).size, `deux routes de même chemin : ${chemins.join(', ')}`)
    .toBe(chemins.length)

  console.warn(`P-22 — ${ROUTES.length} route(s) lues de app/pages/ : ${chemins.join(', ')}`)
})

// =================================================================================================
//  Les cinq contrôles, pour chaque route
// =================================================================================================

for (const route of ROUTES) {
  test.describe(`P-22 · ${route.chemin}`, () => {
    test('1 · chargement DIRECT de l’adresse', async () => {
      // Le chemin d'un signet, d'un lien copié, d'un rechargement. Il ne reprenait pas la
      // session : la page annonçait « Connectez-vous pour continuer » alors qu'un jeton valide
      // dormait dans le stockage.
      await page.goto(route.chemin)
      await page.waitForLoadState('networkidle')

      await expect(page).toHaveURL(new RegExp(`${echapper(route.chemin)}/?$`))
      await verifierCoquille(route, 'chargement direct')
    })

    test('2 · navigation INTERNE depuis l’accueil', async () => {
      await page.goto(route.exigeSession ? '/' : '/connexion')
      await page.waitForLoadState('networkidle')
      erreurs = []

      // Navigation par le routeur, sans rechargement — le chemin qui vidait le `<main>`.
      await page.evaluate((chemin) => {
        history.pushState({}, '', chemin)
        window.dispatchEvent(new PopStateEvent('popstate'))
      }, route.chemin)
      await page.waitForLoadState('networkidle')

      await expect(page).toHaveURL(new RegExp(`${echapper(route.chemin)}/?$`))
      await verifierCoquille(route, 'navigation interne')
    })

    test('5 · la classe .dark s’applique, et la page reste lisible', async () => {
      // Le mode est posé dans le stockage, comme le ferait un utilisateur qui l'a choisi. Ce sont
      // le script en ligne du `<head>` et `plugins/01.theme.client.ts` qui doivent le lire — pas
      // ce test, qui ne touche jamais `classList`.
      await page.evaluate(() => localStorage.setItem('kaya.theme', 'sombre'))
      await page.goto(route.chemin)
      await page.waitForLoadState('networkidle')
      erreurs = []

      const sombre = await page.evaluate(() => document.documentElement.classList.contains('dark'))
      expect(
        sombre,
        `${route.chemin} — la classe .dark n’est pas appliquée alors que le mode sombre est `
        + 'retenu.\nLe mécanisme existe dans core/theme depuis le cycle 001 ; ce qui manquait est '
        + 'le chemin qui l’appelle au démarrage.',
      ).toBe(true)

      // « Lisible » se vérifie sur ce qui est mesurable : le fond a bien basculé, donc les valeurs
      // servies sous `.dark` le sont. Comparer les couleurs une à une reproduirait
      // `theme-sombre.spec.ts`, qui le fait déjà sur les jetons.
      const fond = await page.evaluate(() => getComputedStyle(document.body).backgroundColor)
      expect(fond, `${route.chemin} — fond non résolu en mode sombre`).not.toBe('')
      expect(fond, `${route.chemin} — le fond est resté blanc pur en mode sombre`)
        .not.toBe('rgb(255, 255, 255)')

      await verifierCoquille(route, 'mode sombre')

      // Le mode clair repasse, sinon le test suivant hériterait du sombre.
      await page.evaluate(() => localStorage.removeItem('kaya.theme'))
    })
  })
}

// =================================================================================================
//  Les vérifications communes — contrôles 3 et 4
// =================================================================================================

async function verifierCoquille(route: Route, etape: string): Promise<void> {
  // Contrôle 4 — le `<main>` est TOUJOURS là, et il est UNIQUE. C'est le symptôme exact du défaut
  // du cycle 003 : l'adresse changeait, l'ancienne page restait, puis le `<main>` disparaissait.
  const mains = await page.locator('main').count()
  expect(
    mains,
    `${route.chemin} (${etape}) — ${mains} élément(s) <main> dans le document, 1 attendu.\n`
    + '0 : la page ne s’est pas montée — c’est le défaut « parentNode » du cycle 003.\n'
    + '2+ : un écran rend son propre <main> alors que le layout en rend déjà un.',
  ).toBe(1)

  // Contrôle 3 — aucune erreur de console ni pageerror.
  expect(
    erreurs,
    `${route.chemin} (${etape}) — ${erreurs.length} erreur(s) :\n  ${erreurs.join('\n  ')}`,
  ).toEqual([])
}

/** Échappe un chemin pour l'insérer dans une expression régulière. */
function echapper(chemin: string): string {
  return chemin.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
}
