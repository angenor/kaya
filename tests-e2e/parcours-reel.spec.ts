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
 * Puis, **une fois toutes les routes exercées**, trois contrôles de sortie de session (§ « passer
 * la main » plus bas) : le geste ferme la session et purge le stockage, et une route protégée
 * renvoie ensuite sur `/connexion` — **en chargement direct comme en navigation interne**, les deux
 * mêmes chemins que ci-dessus.
 *
 * **Sur DEUX moteurs de rendu**, et c'est le produit qui l'impose. Tauri v2 n'embarque aucun
 * navigateur : il emprunte celui du système. La correspondance décide de ce qui est réellement
 * couvert :
 *
 * | Moteur du système | Cible du produit | Projet Playwright |
 * |---|---|---|
 * | **WebView2** (Chromium) | Windows | `chromium` |
 * | **Android System WebView** (Chromium) | Android | `chromium` |
 * | **WKWebView** (WebKit) | **macOS** — le poste de développement | `webkit` |
 * | **WKWebView** (WebKit) | **iOS** | `webkit` |
 * | **WebKitGTK** (WebKit) | **Linux** | `webkit` |
 *
 * Trois cibles sur cinq sont WebKit. Chromium seul validait donc le moteur que le produit
 * n'utilisera pas sur la majorité d'entre elles, à commencer par celle sur laquelle il est écrit.
 * Les deux projets exécutent **les mêmes tests, sans exclusion** : un cas qui tombe sous WebKit est
 * un écart que la coquille Tauri rencontrera, pas un cas à retirer de la porte.
 *
 * **Non inspecté, et écrit ici plutôt que supposé.**
 *
 * - **L'apparence.** Cette porte vérifie qu'une page s'ouvre et qu'elle bascule de thème ; elle ne
 *   dit rien de sa justesse visuelle. Aucune capture n'est comparée. La conformité à la maquette
 *   reste humaine — c'est la même limite assumée que `classes_offline.rs` pour la justesse des
 *   classes hors-ligne.
 * - **Le contenu métier.** Qu'un écran s'affiche ne dit pas qu'il affiche les bonnes lignes. Les
 *   tests front couvrent cela, et cette porte ne les remplace pas : elle couvre ce qu'ils ne
 *   peuvent pas voir.
 * - **WKWebView lui-même.** *La limite la plus facile à mal lire.* Le `webkit` de Playwright est
 *   une construction de WebKit maintenue par l'équipe Playwright : **plus proche de la cible que
 *   Chromium, et pas identique**. Il ne porte ni les réglages du composant système d'Apple, ni son
 *   intégration au processus hôte, ni les restrictions propres à iOS. Le contrôle réel de macOS et
 *   d'iOS viendra avec la coquille Tauri. Un vert sur `webkit` dit « le produit tourne sur un
 *   moteur WebKit », jamais « le produit est vérifié sur la cible ».
 * - **Firefox.** Aucune cible du produit n'emploie Gecko. Non couvert, et sans conséquence.
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
 * ⚠️ **ET CETTE VARIABLE N'ARRIVE PAS TOUJOURS.** `playwright.config.ts` la pose dans
 * `webServer.env` — qui ne s'applique **que si Playwright démarre le serveur**. Avec
 * `reuseExistingServer: true`, un serveur de développement déjà lancé est réutilisé **et l'`env`
 * est ignoré en silence**. Les deux lignes se contredisent, et le verdict de la porte dépendait
 * alors de qui avait lancé le serveur :
 *
 * | Serveur | Effet |
 * |---|---|
 * | démarré par Playwright | la variable est là, la porte est juste |
 * | réutilisé, **sans** la variable | **faux rouge** — 404 sur une route qui devrait exister |
 * | réutilisé, **avec** la variable | **faux vert POSSIBLE** — voir ci-dessous |
 *
 * Le troisième cas est le plus grave : il ferait passer au vert un dépôt où la route du
 * styleguide aurait cessé d'être retirée en production. Ce versant-là n'est pas gardé ici mais
 * par `app/tests/styleguide.spec.ts`, qui vérifie que la décision de montage est **fausse par
 * défaut** et que la route est retirée du routeur — pas cachée derrière une garde de rendu.
 *
 * `exigerStyleguideServi()` referme le deuxième cas : il constate, **avant la connexion**, que la
 * route est bien montée, et échoue en disant **le geste** — pas seulement le constat. Une porte
 * qui refuse sans dire quoi faire finit par être ignorée, et une porte ignorée ne garde rien.
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

import { BASE_APP, PORT_APP } from './adresses'
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

/**
 * Les deux routes que la section « passer la main » nomme — **cherchées dans `ROUTES`**, jamais
 * écrites à la main.
 *
 * C'est la règle de `routes.ts`, appliquée ici : une liste écrite à la main laisse passer la
 * septième page, et une constante écrite à la main continue de désigner une route supprimée — le
 * test s'exercerait alors sur une 404 en la prenant pour une redirection. Une cible introuvable
 * **lève** au chargement du fichier plutôt que de se replier en silence sur autre chose.
 */
function routeDeLaPorte(predicat: (route: Route) => boolean, quoi: string): Route {
  const trouvee = ROUTES.find(predicat)
  if (!trouvee) {
    throw new Error(
      `P-22 — aucune route ${quoi} parmi les ${ROUTES.length} lues de app/pages/.\n`
      + 'Les contrôles de sortie de session n’ont plus de cible : ils passeraient au vert sans '
      + 'rien exercer.',
    )
  }
  return trouvee
}

/** L'écran de connexion — la cible de toute redirection hors session. */
const R0 = routeDeLaPorte(route => route.chemin === '/connexion', 'de connexion')

/**
 * La route protégée sur laquelle la redirection est vérifiée.
 *
 * La racine est écartée : elle est la destination par défaut de trop de chemins, et une
 * redirection qui y aboutirait par hasard ressemblerait à une redirection réussie.
 */
const PROTEGEE = routeDeLaPorte(
  route => route.exigeSession && route.chemin !== '/',
  'protégée autre que la racine',
)

/**
 * Le port exercé — **lu de `./adresses`**, avec les deux autres fichiers de portes.
 *
 * Il était écrit `3000` en dur ici alors que `playwright.config.ts` honore `KAYA_PORT_E2E` depuis
 * le cycle 004. La correction a d'abord été faite ICI SEULEMENT, et le défaut a été reproduit le
 * jour même dans un fichier neuf : d'où la source unique, et le contrôle qui la rend opposable.
 */
const PORT = PORT_APP

/**
 * **Le serveur exercé sert-il bien la route du styleguide ?** — à appeler AVANT la connexion.
 *
 * Voir la section « Le styleguide est couvert » en tête de fichier : `webServer.env` n'a aucun
 * effet sur un serveur réutilisé, et la porte rendait alors un faux rouge sur `/styleguide` pour
 * une raison d'environnement — le genre d'échec qui fait ignorer une porte.
 *
 * Le contrôle passe par le **navigateur** et non par une requête HTTP : en développement, Nuxt
 * rend le même squelette de 3,5 ko pour une page servie et pour sa page d'erreur, et l'écart
 * n'apparaît qu'après hydratation. Une sonde `curl` aurait validé les deux cas indifféremment —
 * ce qui est la définition d'un contrôle qui ne contrôle rien.
 *
 * ⚠️ **Il est appelé avant la connexion, et c'est délibéré.** Le limiteur compte dix tentatives
 * par identifiant sur cinq minutes glissantes, **réussies comprises** : échouer après s'être
 * connecté consommerait une tentative à chaque essai de diagnostic, et le refus qui s'ensuivrait
 * est indiscernable d'un mot de passe faux (FR-012).
 */
async function exigerStyleguideServi(page: Page): Promise<void> {
  await page.goto('/styleguide')
  await page.waitForLoadState('networkidle')

  const monte = await page.locator('main').count() > 0
  if (monte) return

  throw new Error(
    'P-22 — le serveur qui tourne sur :' + PORT + ' ne sert PAS « /styleguide ».\n'
    + '\n'
    + 'Ce n’est pas un défaut du produit : c’est `KAYA_STYLEGUIDE=1` qui manque au serveur.\n'
    + '`playwright.config.ts` pose la variable dans `webServer.env`, et `webServer.env` ne\n'
    + 's’applique QUE si Playwright démarre le serveur. Avec `reuseExistingServer: true`, un\n'
    + 'serveur de développement déjà lancé est réutilisé et la variable est ignorée en silence.\n'
    + '\n'
    + 'DEUX GESTES, AU CHOIX :\n'
    + `  1. arrêter le serveur — PAR PORT, jamais par nom : lsof -ti:${PORT} | xargs kill\n`
    + '     puis relancer la porte : Playwright démarrera le sien, avec la variable ;\n'
    + `  2. ou le relancer avec : KAYA_STYLEGUIDE=1 pnpm --filter @kaya/app dev --port ${PORT}\n`
    + '\n'
    + '⚠️ NE JAMAIS employer `pkill -f "nuxt"` : un pkill a déjà tué le serveur de\n'
    + '   développement d’un AUTRE projet de ce poste.',
  )
}

let contexte: BrowserContext
let page: Page
/** Rempli par le collecteur, vidé avant chaque contrôle. */
let erreurs: string[] = []

test.beforeAll(async ({ browser }: { browser: Browser }) => {
  contexte = await browser.newContext({
    locale: 'fr-FR',
    baseURL: BASE_APP,
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

  // L'environnement AVANT le compte : un serveur mal configuré ferait échouer treize routes sur
  // une cause unique, et consommerait le limiteur à chaque tentative de diagnostic.
  await exigerStyleguideServi(page)
  erreurs = []

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
//  Passer la main — EN DERNIER, et ce n'est pas une commodité d'écriture
// =================================================================================================
//
//  Ces trois contrôles **détruisent la session partagée**, celle que `beforeAll` a ouverte une
//  seule fois. Les placer ailleurs qu'à la fin ferait échouer toutes les routes qui suivent, et
//  la rouvrir coûterait une tentative de plus au compteur de `LimiteTentatives` — dix par
//  identifiant sur cinq minutes, réussies comprises. L'ordre est donc structurel, et
//  `test.describe.configure({ mode: 'serial' })` en tête de fichier le garantit.
//
//  Ce qu'ils gardent : `fermerSession()` a vécu un cycle entier exportée et **appelée nulle part**.
//  Sur un terminal de comptoir, où l'appareil ne bouge pas et où c'est la personne qui change, les
//  actions de Yao entraient au journal d'audit **au nom d'Aminata**. `app/tests/deconnexion.spec.ts`
//  vérifie que le geste fait ce qu'il annonce ; ici, on vérifie qu'il est **atteignable dans le
//  produit**, et que ce qu'il ferme reste fermé sur les deux chemins d'accès — c'est exactement la
//  distinction que P-22 existe pour porter.

test.describe('P-22 · passer la main', () => {
  test('6 · le bouton ferme la session, purge le stockage et renvoie sur /connexion', async () => {
    await page.goto('/')
    await page.waitForLoadState('networkidle')
    erreurs = []

    // Le bouton est cherché par son **rôle et son libellé**, comme un utilisateur le trouverait —
    // pas par une classe ni un `data-testid`. Un sélecteur technique passerait encore le jour où
    // le libellé cesserait d'être celui du lexique.
    const bouton = page.getByRole('button', { name: /passer la main/i })
    await expect(
      bouton,
      'aucun bouton « passer la main » dans la coquille sous session.\n'
      + 'C’est l’état d’avant ce lot : `fermerSession()` existait sans aucun appelant, et il '
      + 'n’y avait aucun moyen de quitter sa session.',
    ).toBeVisible()

    await bouton.click()
    await page.waitForURL(url => new URL(url).pathname === '/connexion', { timeout: 20_000 })

    // **La purge, constatée dans le navigateur** — pas déduite de l'appel. Le cadrage §11.5
    // règle 5 l'impose : ce sont des données d'identité de clients.
    const restant = await page.evaluate(
      () => Object.keys(localStorage).filter(cle => cle.startsWith('kaya.')),
    )
    expect(
      restant,
      `des clés « kaya. » ont survécu à la déconnexion : ${restant.join(', ')}`,
    ).toEqual([])

    await verifierCoquille(R0, 'après déconnexion')
  })

  test('7 · une route protégée en CHARGEMENT DIRECT renvoie sur /connexion', async () => {
    // Le chemin du signet et du rechargement. Sans jeton rangé, le middleware global ne peut plus
    // reprendre de session : il doit renvoyer, pas afficher un écran vide.
    await page.goto(PROTEGEE.chemin)
    await page.waitForLoadState('networkidle')

    await expect(page).toHaveURL(/\/connexion\/?$/)
    await verifierCoquille(R0, 'protégée en direct, déconnecté')
  })

  test('8 · une route protégée en NAVIGATION INTERNE renvoie sur /connexion', async () => {
    await page.goto('/connexion')
    await page.waitForLoadState('networkidle')
    erreurs = []

    await page.evaluate((chemin) => {
      history.pushState({}, '', chemin)
      window.dispatchEvent(new PopStateEvent('popstate'))
    }, PROTEGEE.chemin)
    await page.waitForLoadState('networkidle')

    await expect(
      page,
      `${PROTEGEE.chemin} s’est ouverte sans session par navigation interne.\n`
      + 'L’adresse a changé sans que le middleware ne redirige — c’est la moitié du défaut du '
      + 'cycle 003, sur le chemin où il ne se voyait pas.',
    ).toHaveURL(/\/connexion\/?$/)
    await verifierCoquille(R0, 'protégée en interne, déconnecté')
  })
})

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
