/**
 * **L'ACCUEIL MÈNE-T-IL QUELQUE PART — AU DOIGT ?** Complément de P-22.
 *
 * ═══════════════════════════════════════════════════════════════════════════════════════════
 *  CE QUE CE FICHIER PROUVE, ET QUE LE CONTRÔLE STATIQUE NE PEUT PAS PROUVER
 * ═══════════════════════════════════════════════════════════════════════════════════════════
 *
 * `app/tests/catalogue-accueil.spec.ts` est le filet principal : il découvre les routes de
 * `app/pages/` et refuse qu'une seule n'ait ni tuile ni exemption motivée. Il tourne dans le job
 * `app` de la CI, sans base ni API. **C'est lui qu'il faut lire en premier.**
 *
 * Il ne peut pourtant pas répondre à la question que l'exploitant se pose : *est-ce que ça marche
 * quand je touche l'écran ?* Trois choses lui échappent, et chacune a un précédent dans ce dépôt :
 *
 * 1. **Que la tuile soit RENDUE.** Le catalogue peut être juste et le gabarit ne rien afficher —
 *    une permission absente de la session, un filtre par module trop strict. C'est exactement le
 *    défaut de ce lot : les deux tuiles d'hébergement étaient au catalogue depuis le cycle 004 et
 *    **invisibles pour tout le monde**, parce que `pages/index.vue` passait une liste de modules
 *    vide en dur.
 * 2. **Que le clic NAVIGUE.** Un `NuxtLink` mal formé, une route mal orthographiée d'un caractère,
 *    et le lien mène à un 404 que le contrôle statique tient pour une route valide.
 * 3. **Que l'écran d'arrivée SE MONTE.** Le cycle 006 a trouvé `/passage` cassé — importé d'un
 *    baril qui n'exportait pas `useEtatReseau` — alors que ses tests unitaires étaient verts :
 *    **ils doublaient le baril en fournissant l'export manquant**. Seul un navigateur qui charge
 *    le vrai module le dit.
 *
 * ═══════════════════════════════════════════════════════════════════════════════════════════
 *  LE PÉRIMÈTRE EST CE QUE L'ÉCRAN REND, PAS CE QUE LE CATALOGUE DÉCLARE
 * ═══════════════════════════════════════════════════════════════════════════════════════════
 *
 * Les tuiles exercées sont **relevées du DOM**, une par une, et non importées de
 * `core/accueil/tuiles.ts`. Importer le catalogue ferait de ce fichier une seconde lecture de la
 * même déclaration — il passerait au vert sur un accueil qui n'affiche rien, puisqu'il saurait
 * quoi chercher sans avoir à le trouver.
 *
 * Le décompte attendu vient donc du **compte de démonstration**, dont les permissions sont celles
 * des seeds : Adjoua cumule trois rôles, et c'est le compte le plus large du jeu.
 *
 * ═══════════════════════════════════════════════════════════════════════════════════════════
 *  ⚠️ CE FICHIER EXIGE L'API, LA BASE ET LES SEEDS — comme P-22, et pour la même raison
 * ═══════════════════════════════════════════════════════════════════════════════════════════
 *
 * Il n'est donc **pas** dans `.github/workflows/ci.yml`. C'est précisément pourquoi il ne peut pas
 * être le seul filet, et pourquoi le contrôle statique existe.
 *
 * Et il ne coexiste pas avec la suite backend : `exiger_grand_livre_sans_consommateur_concurrent`
 * refuse de dérouler les tests d'outbox quand un worker de publication tourne hors de
 * `cargo test` — c'est-à-dire quand l'API est allumée. Séquencer, et arrêter l'API **par port**.
 */

import { expect, test, type Browser, type BrowserContext, type Page } from '@playwright/test'

import { BASE_APP } from './adresses'
import { COMPTE_DEMONSTRATION, ROUTES } from './routes'

test.describe.configure({ mode: 'serial' })

/** Les erreurs de console tolérées, **nommées une par une, jamais par motif large**. */
const BRUIT_TOLERE: readonly RegExp[] = [
  /<Suspense> is an experimental feature/,
  /\[vite\] (connecting|connected)/,
]

/**
 * Le nombre de tuiles au-dessous duquel l'accueil d'Adjoua est en défaut.
 *
 * Elle porte `gerant` + `caissier` + `receptionniste` : toutes les lectures du produit **sauf**
 * `cpt.audit.consulter`, que CPT-04 réserve au propriétaire. Sur un établissement qui fait de
 * l'hébergement — Deloria en fait —, cela lui ouvre tout le catalogue moins le registre.
 *
 * Le seuil est un **plancher**, pas une égalité : un cycle qui ajoute une tuile ne doit pas avoir
 * à revenir corriger un nombre ici, et l'égalité stricte ferait de ce fichier une seconde
 * déclaration du catalogue. Ce qui compte est qu'il ne s'effondre pas — c'est **deux** tuiles
 * qu'Adjoua voyait quand ce lot a commencé.
 */
const PLANCHER_TUILES = 9

let contexte: BrowserContext
let page: Page
let erreurs: string[] = []

test.beforeAll(async ({ browser }: { browser: Browser }) => {
  // L'adresse vient de `./adresses`, jamais d'un littéral : sur un poste où le 3000 est
  // pris, Playwright sert sur 3001 et un `baseURL` en dur interrogerait le serveur du
  // voisin — un vert rendu sur une application qui n'est pas la nôtre.
  contexte = await browser.newContext({ locale: 'fr-FR', baseURL: BASE_APP })
  page = await contexte.newPage()

  page.on('console', (message) => {
    if (message.type() !== 'error' && message.type() !== 'warning') return
    const texte = message.text()
    if (!BRUIT_TOLERE.some(motif => motif.test(texte))) {
      erreurs.push(`console.${message.type()}: ${texte}`)
    }
  })
  page.on('pageerror', erreur => erreurs.push(`pageerror: ${erreur.message}`))

  // La connexion par le vrai formulaire — poser un jeton forgé irait plus vite et ne prouverait
  // rien sur ce que la session porte réellement.
  await page.goto('/connexion')
  await page.getByLabel(/identifiant/i).fill(COMPTE_DEMONSTRATION.identifiant)
  await page.getByLabel(/mot de passe/i).fill(COMPTE_DEMONSTRATION.motDePasse)
  await page.getByRole('button', { name: /se connecter/i }).click()
  await page.waitForURL(url => new URL(url).pathname === '/', { timeout: 20_000 })
})

test.afterAll(async () => {
  await contexte?.close()
})

test.beforeEach(() => {
  erreurs = []
})

/** Les tuiles **effectivement rendues**, relevées du DOM. */
async function tuilesAffichees(): Promise<{ code: string, route: string }[]> {
  await page.goto('/')
  await page.waitForLoadState('networkidle')
  // Les tuiles de verticale n'apparaissent qu'une fois les services de l'établissement lus : le
  // premier rendu sert le cache, le second l'état réel. On attend donc le **plancher**, pas un
  // délai — et pas une égalité non plus, qui ferait de ce fichier une copie du catalogue.
  await expect
    .poll(() => page.locator('[data-tuile]').count(), { timeout: 15_000 })
    .toBeGreaterThanOrEqual(PLANCHER_TUILES)

  return page.locator('[data-tuile]').evaluateAll(noeuds =>
    noeuds.map(noeud => ({
      code: noeud.getAttribute('data-tuile') ?? '',
      route: new URL((noeud as HTMLAnchorElement).href).pathname,
    })),
  )
}

// =================================================================================================
//  1. L'accueil PROPOSE — exigence 4, la cible n'est pas vide
// =================================================================================================

test('l’accueil d’Adjoua propose ses écrans, et pas deux tuiles', async () => {
  const tuiles = await tuilesAffichees()

  expect(
    tuiles.length,
    'L’accueil rend moins de tuiles qu’Adjoua n’a de droits.\n'
    + 'C’est le défaut de départ de ce lot : elle en voyait DEUX sur onze, et les neuf autres '
    + 'écrans n’étaient joignables qu’en tapant leur URL.',
  ).toBeGreaterThanOrEqual(PLANCHER_TUILES)

  // Les tuiles d'hébergement sont le cas qui a échoué : leur `moduleRequis` était comparé à une
  // liste de modules codée en dur à `[]`. Les nommer ici rend la régression impossible à rater.
  const codes = tuiles.map(t => t.code)
  for (const attendu of ['passage', 'arrivee', 'depart', 'clients', 'hebergement-offre']) {
    expect(codes, `la tuile « ${attendu} » n’est pas rendue`).toContain(attendu)
  }

  // Aucun doublon : une tuile issue de trois rôles apparaît une fois (FR-027), et c'est ici
  // vérifié sur le rendu réel d'un compte à trois rôles, pas sur une liste de permissions.
  expect(new Set(codes).size, `tuile(s) en double : ${codes.join(', ')}`).toBe(codes.length)

  console.warn(`Accueil — ${tuiles.length} tuile(s) : ${codes.join(', ')}`)
})

test('aucune description n’est coupée au milieu d’un mot', async () => {
  await tuilesAffichees()

  // ⚠️ **CE CONTRÔLE REMPLACE UN ARBITRAGE, IL NE LE CONTOURNE PAS.** Le composant tronque les
  // descriptions (`truncate`), et la question s'est posée de retirer la troncature. Décision :
  // la garder. Trois textes qui dépassent sont trois textes trop longs, pas un composant trop
  // strict — et retirer `truncate` déplacerait le problème sur la première description de vingt
  // mots qu'un cycle suivant écrira, en cassant la hauteur de tuile de la maquette `R1`.
  //
  // La contrainte doit donc être **vérifiée**, sinon elle n'est qu'une intention : une tuile dont
  // le sous-titre est coupé — « la chambre, la clé, et la pièc… » — ment sur ce qu'elle ouvre,
  // ce qui est pire que d'être brève. Trois des cinq descriptions d'origine étaient dans cet état.
  //
  // La mesure est celle du navigateur, jamais un compte de caractères : la largeur dépend des
  // glyphes rendus, et « Ce que vous proposez, et ce que les impôts voient. » tenait à 48
  // caractères et débordait à 49.
  const tronquees = await page.locator('[data-tuile] span.truncate').evaluateAll(noeuds =>
    noeuds
      .filter(noeud => noeud.scrollWidth > noeud.clientWidth + 1)
      .map(noeud => noeud.textContent?.trim() ?? ''),
  )

  expect(
    tronquees,
    'Ces descriptions de tuile sont coupées à l’écran.\n'
    + 'Les raccourcir — en passant par `docs/design/lexique.md` — plutôt que de retirer le '
    + '`truncate` : la troncature protège la hauteur de tuile de la maquette `R1`.',
  ).toEqual([])
})

test('aucune tuile ne mène ailleurs que sur une route du produit', async () => {
  const tuiles = await tuilesAffichees()
  const connues = new Set(ROUTES.map(r => r.chemin))

  for (const tuile of tuiles) {
    expect(connues, `« ${tuile.code} » pointe vers ${tuile.route}, absent de app/pages/`)
      .toContain(tuile.route)
  }
})

// =================================================================================================
//  2. LE CONTRÔLE CENTRAL — chaque tuile est CLIQUÉE
// =================================================================================================

test('chaque tuile s’ouvre AU CLIC, et l’écran se monte', async () => {
  const tuiles = await tuilesAffichees()
  const echecs: string[] = []

  for (const tuile of tuiles) {
    await page.goto('/')
    await page.waitForLoadState('networkidle')
    await expect(page.locator(`[data-tuile="${tuile.code}"]`)).toBeVisible({ timeout: 15_000 })
    erreurs = []

    // Le geste réel : un clic sur la tuile, pas un `page.goto` vers sa route. C'est le seul moyen
    // de prouver que le lien est bien celui que l'écran rend — et la navigation par le routeur est
    // le chemin qui vidait le `<main>` au cycle 003.
    await page.locator(`[data-tuile="${tuile.code}"]`).click()

    try {
      await page.waitForURL(url => new URL(url).pathname === tuile.route, { timeout: 20_000 })
    }
    catch {
      echecs.push(`${tuile.code} — le clic n’a pas mené à ${tuile.route} (URL : ${page.url()})`)
      continue
    }
    await page.waitForLoadState('networkidle')

    // ⚠️ **Le contrôle qui manquait au balayage hors ligne du cycle 005** : neuf cas y sont passés
    // au vert sur neuf fois le même écran de connexion. Une redirection n'est pas une ouverture.
    if (new URL(page.url()).pathname === '/connexion') {
      echecs.push(`${tuile.code} — a renvoyé sur /connexion alors que la session est ouverte`)
      continue
    }

    // L'écran s'est-il MONTÉ ? Un `<main>` vide est le symptôme exact de la racine multiple :
    // le fragment a un `el` nul, Vue lève à la navigation, l'ancien écran reste affiché.
    const contenu = await page.locator('main').innerText().catch(() => '')
    if (contenu.trim().length === 0) {
      echecs.push(`${tuile.code} — ${tuile.route} s’ouvre sur un <main> VIDE`)
    }

    if (erreurs.length > 0) {
      echecs.push(`${tuile.code} — erreurs de console : ${erreurs.join(' | ')}`)
    }
  }

  expect(
    echecs,
    'Des tuiles de l’accueil ne mènent pas à leur écran.\n'
    + 'Une tuile qui ne s’ouvre pas est pire qu’une tuile absente : elle promet, au comptoir, '
    + 'devant le client.',
  ).toEqual([])
})

// =================================================================================================
//  3. Versant négatif — ce qu'Adjoua ne doit PAS voir
// =================================================================================================

test('le registre des actions lui est ABSENT, pas grisé', async () => {
  // `cpt.audit.consulter` n'est pas dans son union : CPT-04 désigne le registre comme « ce que
  // M. Koffi achète », et la lecture par le surveillé change ce que le registre est.
  //
  // Sans ce contrôle, un accueil qui rendrait TOUT le catalogue sans filtrer passerait les
  // précédents — le décompte serait plus grand, jamais plus petit.
  const tuiles = await tuilesAffichees()

  expect(tuiles.map(t => t.code)).not.toContain('journal-audit')

  // Ni grisée, ni masquée par CSS : le lien n'est nulle part dans le HTML de l'accueil.
  expect(await page.content()).not.toContain('/journal-audit')
})
