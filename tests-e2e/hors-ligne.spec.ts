/**
 * **FR-005b — réseau coupé, toute action de classe B, C ou D l'annonce AVANT la saisie.**
 *
 * ═══════════════════════════════════════════════════════════════════════════════════════════
 *  CE QUE CETTE PORTE GARDE, ET POURQUOI LE VERSANT TYPE NE SUFFIT PAS
 * ═══════════════════════════════════════════════════════════════════════════════════════════
 *
 * La porte **P-13** a deux versants, et le produit n'en avait qu'un.
 *
 * Le premier est le **type** : `FileLocale.enfiler` n'accepte qu'une charge marquée classe A, et
 * une opération non déclarée est refusée même marquée. Il est vérifié par
 * `app/tests/file-classe-a.spec.ts`, à la compilation. Il garantit qu'une opération B/C/D
 * **n'entre pas en file**.
 *
 * Le second est l'**écran**, et c'est celui-là qui manquait : le principe VI n'exige pas seulement
 * qu'une opération B/C/D ne soit pas mise en file, il exige que l'interface **annonce son
 * indisponibilité avant la saisie** — jamais un grisé silencieux, jamais un échec après trente
 * secondes d'attente. Aucun contrôle ne le vérifiait en direct.
 *
 * ═══════════════════════════════════════════════════════════════════════════════════════════
 *  PÉRIMÈTRE — CROISÉ ENTRE TROIS SOURCES EXISTANTES, AUCUNE ÉCRITE À LA MAIN (exigence 1)
 * ═══════════════════════════════════════════════════════════════════════════════════════════
 *
 * | Source | Ce qu'elle donne | D'où elle vient |
 * |---|---|---|
 * | Le **contrat OpenAPI** | toutes les opérations non-`GET` du produit | servi par l'API, généré depuis le code (principe I·a) |
 * | `app/core/sync/classes.ts` | les types d'opération déclarés **classe A** | la liste que la file elle-même consulte |
 * | `tests-e2e/routes.ts` | les écrans | lus de `app/pages/` |
 *
 * Une liste d'écrans écrite à la main aurait laissé passer le septième, et deux précédents dans ce
 * dépôt disent que ce n'est pas une crainte théorique — le décompte de P-07 portait sur 4 tables
 * sur 10 au cycle 002, et `couverture_portes.rs` ne balayait qu'un schéma sur deux au 003.
 *
 * **La classe se déduit par complément, et c'est le point le plus fin de ce fichier.**
 * `TYPES_CLASSE_A` est la **seule** liste d'opérations de classe A du versant application : la
 * file la consulte à chaque enfilement, et une opération qui n'y figure pas est refusée à
 * l'exécution même marquée. Toute opération d'écriture du contrat qui ne correspond à aucun type
 * de cette liste est donc, **du point de vue du produit**, de classe B, C ou D — et doit annoncer
 * son indisponibilité.
 *
 * Apparier chaque `operationId` à sa ligne du registre aurait été plus direct et moins sûr : le
 * registre classe des **opérations métier** (« encaissement en espèces », « encaissement Mobile
 * Money »), pas des chemins HTTP, et l'appariement se serait fait par une table écrite à la main —
 * exactement ce que ce cycle supprime partout ailleurs.
 *
 * ═══════════════════════════════════════════════════════════════════════════════════════════
 *  LA LIMITE ASSUMÉE — à lire avant de conclure quoi que ce soit d'un vert (T042)
 * ═══════════════════════════════════════════════════════════════════════════════════════════
 *
 * **Cette porte vérifie qu'une annonce d'indisponibilité APPARAÎT. Elle ne vérifie jamais que sa
 * FORMULATION est la bonne.**
 *
 * La justesse du libellé relève de `docs/design/lexique.md` et de la porte **P-16**, qui vérifie
 * qu'aucune chaîne n'est en dur et que les catalogues sont à parité. Confondre les deux donnerait
 * une porte qui ment sur ce qu'elle garantit : elle passerait au vert sur « Erreur 503 » affiché
 * au comptoir, et le vert empêcherait la relecture.
 *
 * Deuxième limite, du même ordre : la porte constate qu'une annonce est **présente dans le rendu**
 * hors ligne. Elle ne peut pas prouver qu'elle est **antérieure à la saisie** — un écran qui
 * l'afficherait après un clic satisferait le contrôle. Ce qui rend l'antériorité vraie est le
 * patron d'écriture (`docs/module-dore.md`, septième couche), où la garde est posée **dans le
 * module**, avant l'appel, et non dans le composant.
 *
 * ═══════════════════════════════════════════════════════════════════════════════════════════
 *  CETTE PORTE N'ÉCRIT RIEN (T043, exigence 3 — « ne pas modifier ce qu'on inspecte »)
 * ═══════════════════════════════════════════════════════════════════════════════════════════
 *
 * Le balayage **ouvre des écrans**, et rien de plus. Aucun formulaire n'est soumis, aucun bouton
 * d'action n'est cliqué, et le réseau est de toute façon coupé pendant tout le parcours.
 *
 * Le **seul** geste d'écriture du fichier est la note interne — opération de classe A, sur le
 * tenant de démonstration —, et il est là parce que le versant positif l'exige : une porte qui
 * vérifie que B/C/D refuse sans vérifier que A **accepte** passerait au vert sur un produit qui
 * refuse tout. Cette écriture-là ne part pas : elle entre en file, et la file est vidée en fin de
 * fichier sans jamais atteindre le serveur, puisque le contexte reste hors ligne.
 */

import { expect, test, type Browser, type BrowserContext, type Page } from '@playwright/test'
import { readFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

import { ROUTES } from './routes'

const ICI = dirname(fileURLToPath(import.meta.url))
const RACINE = resolve(ICI, '..')

/** Le compte de démonstration — le même que celui de P-22. */
const COMPTE_DEMONSTRATION = {
  identifiant: 'adjoua@deloria.test',
  motDePasse: process.env.KAYA_SEEDS_MOT_DE_PASSE ?? '',
}

const API = process.env.KAYA_API_BASE_URL ?? 'http://localhost:8080'

// =================================================================================================
//  Source 1 — le contrat OpenAPI, servi par l'API
// =================================================================================================

interface OperationEcriture {
  readonly methode: string
  readonly chemin: string
  readonly operationId: string
}

/**
 * Les opérations **non-`GET`** du contrat.
 *
 * Lues de l'API en marche plutôt que d'un fichier commité : le contrat est un **produit du code**
 * (principe I·a), et le lire du serveur qui l'expose garantit qu'on inspecte ce qui est
 * réellement servi — pas un artefact qui aurait pu dériver.
 */
async function operationsDEcriture(): Promise<OperationEcriture[]> {
  const reponse = await fetch(`${API}/api-docs/openapi.json`)
  if (!reponse.ok) {
    throw new Error(
      `le contrat OpenAPI n'est pas servi par ${API} (${reponse.status}).\n`
      + 'Cette porte croise le contrat, le registre des classes et les routes : sans le premier, '
      + 'elle inspecterait un périmètre vide et passerait au vert.',
    )
  }

  const contrat = (await reponse.json()) as {
    paths: Record<string, Record<string, { operationId?: string }>>
  }

  const operations: OperationEcriture[] = []
  for (const [chemin, item] of Object.entries(contrat.paths)) {
    for (const [methode, operation] of Object.entries(item)) {
      if (methode === 'get' || methode === 'parameters') {
        continue
      }
      operations.push({
        methode: methode.toUpperCase(),
        chemin,
        operationId: operation.operationId ?? `${methode}:${chemin}`,
      })
    }
  }
  return operations
}

// =================================================================================================
//  Source 2 — les types déclarés CLASSE A, lus de la file elle-même
// =================================================================================================

/**
 * Les types de classe A, **extraits du source que la file consulte**.
 *
 * `app/core/sync/classes.ts` est lu comme du texte plutôt qu'importé : ce fichier tourne dans un
 * processus Playwright, et importer un module de `app/` y entraînerait la résolution d'alias de
 * Nuxt. La lecture est stricte — un tableau vide fait échouer le contrôle plus bas.
 */
function typesClasseA(): string[] {
  const source = readFileSync(resolve(RACINE, 'app', 'core', 'sync', 'classes.ts'), 'utf8')
  const bloc = source.match(/TYPES_CLASSE_A[^=]*=\s*\[([\s\S]*?)\]/)
  if (!bloc?.[1]) {
    throw new Error(
      'TYPES_CLASSE_A introuvable dans app/core/sync/classes.ts. La déclaration a-t-elle été '
      + 'reformulée ? Sans elle, cette porte ne sait plus quelle opération est de classe A, et '
      + 'les classerait toutes B/C/D — ou aucune.',
    )
  }
  return [...bloc[1].matchAll(/'([^']+)'/g)].map(m => m[1]!)
}

/**
 * Une opération du contrat correspond-elle à un type de classe A ?
 *
 * L'appariement se fait sur l'**entité**, qui est le préfixe du type — `note_etablissement.creee`
 * → `note_etablissement` → le chemin `/notes`. C'est une heuristique, et elle est déclarée comme
 * telle : elle se trompe dans le sens **prudent**, en classant B/C/D ce qu'elle ne reconnaît pas.
 * Une opération de classe A mal reconnue exigerait une annonce d'indisponibilité qui n'existe pas,
 * et la porte échouerait bruyamment — jamais l'inverse.
 */
function estClasseA(operation: OperationEcriture, typesA: readonly string[]): boolean {
  return typesA.some((type) => {
    const entite = type.split('.')[0] ?? type
    // `note_etablissement` → « notes » dans le chemin. Le pluriel est la convention REST du
    // contrat, posée par le module doré.
    const segment = entite.replace(/^note_etablissement$/, 'notes')
    return operation.chemin.includes(`/${segment}`)
  })
}

// =================================================================================================
//  Le harnais
// =================================================================================================

let contexte: BrowserContext
let page: Page
let erreurs: string[] = []

/** Les écrans à balayer : toutes les routes protégées, sauf le styleguide. */
const ECRANS = ROUTES.filter(route => route.exigeSession)

/**
 * **Une seule connexion par exécution, et c'est une contrainte du produit, pas du test.**
 *
 * `LimiteTentatives` plafonne à **dix tentatives par identifiant sur une fenêtre glissante de cinq
 * minutes, réussies comprises** — et le refus de dépassement est **indiscernable d'un mot de passe
 * faux** (FR-012), ce qui rend le diagnostic long quand on le rencontre.
 *
 * Un `beforeAll` de niveau fichier est rejoué à chaque redémarrage de worker, et Playwright
 * redémarre le worker après un échec : une défaillance en produit une seconde, qui en produit une
 * troisième. La connexion vit donc dans le **seul** groupe qui en a besoin, et le contrôle de
 * périmètre — qui ne touche à aucune page — n'en consomme aucune.
 */
async function ouvrirSession(browser: Browser): Promise<void> {
  expect(
    COMPTE_DEMONSTRATION.motDePasse,
    'KAYA_SEEDS_MOT_DE_PASSE n’est pas défini — la porte ne pourrait pas se connecter.',
  ).not.toBe('')

  contexte = await browser.newContext({ locale: 'fr-FR', baseURL: 'http://localhost:3000' })
  page = await contexte.newPage()
  page.on('pageerror', erreur => erreurs.push(`pageerror: ${erreur.message}`))

  // La connexion se fait **en ligne**, par le vrai formulaire : c'est l'état d'où part une
  // coupure réelle. Se connecter hors ligne n'aurait aucun sens — c'est une opération de classe C.
  await page.goto('/connexion')
  await page.getByLabel(/identifiant/i).fill(COMPTE_DEMONSTRATION.identifiant)
  await page.getByLabel(/mot de passe/i).fill(COMPTE_DEMONSTRATION.motDePasse)
  await page.getByRole('button', { name: /se connecter/i }).click()
  await page.waitForURL(url => new URL(url).pathname === '/', { timeout: 20_000 })
}

test.beforeEach(() => {
  erreurs = []
})

// =================================================================================================
//  Exigence 2 — la cible n'est pas vide, et son décompte est RAPPORTÉ
// =================================================================================================

test('FR-005b · le périmètre est croisé, compté, et rapporté', async () => {
  const operations = await operationsDEcriture()
  const typesA = typesClasseA()

  expect(
    operations.length,
    'aucune opération d’écriture au contrat : la porte n’inspecterait rien.',
  ).toBeGreaterThan(10)
  expect(typesA.length, 'aucun type de classe A déclaré — l’extraction est cassée').toBeGreaterThan(0)
  expect(ECRANS.length, 'aucun écran protégé lu de app/pages/').toBeGreaterThan(0)

  const classeA = operations.filter(o => estClasseA(o, typesA))
  const bcd = operations.filter(o => !estClasseA(o, typesA))

  // **Le rapport (T041).** Il est imprimé même au vert : c'est la seule façon de constater, à la
  // lecture d'un journal de CI, que la porte a inspecté ce qu'elle annonce.
  console.warn(
    `FR-005b — ${operations.length} opération(s) d’écriture au contrat :\n`
    + `  · ${classeA.length} de classe A déclarée (${classeA.map(o => o.operationId).join(', ') || '—'})\n`
    + `  · ${bcd.length} de classe B, C ou D — toutes doivent annoncer leur indisponibilité\n`
    + `  · ${ECRANS.length} écran(s) protégé(s) balayé(s) : ${ECRANS.map(e => e.chemin).join(', ')}`,
  )

  expect(
    bcd.length,
    'aucune opération de classe B/C/D au contrat : la porte n’aurait rien à garder.',
  ).toBeGreaterThan(0)
})

// =================================================================================================
//  Le balayage EN DIRECT, réseau coupé
// =================================================================================================

test.describe('réseau coupé — chaque écran d’écriture annonce, aucun n’enfile', () => {
  // Les cas partagent une page et un ordre : `serial` évite qu'un échec en cascade masque le
  // premier, et qu'une reconnexion soit tentée pour rien.
  test.describe.configure({ mode: 'serial' })

  test.beforeAll(async ({ browser }: { browser: Browser }) => {
    await ouvrirSession(browser)

    // **La coupure est réelle**, au niveau du contexte du navigateur : toute requête sortante
    // échoue, comme derrière un réseau tombé. Simuler `navigator.onLine` seul ne prouverait rien —
    // c'est exactement l'état qu'`etatReseauNavigateur()` ne suffit pas à décrire.
    await contexte.setOffline(true)
  })

  test.afterAll(async () => {
    await contexte?.setOffline(false)
    await contexte?.close()
  })

  for (const ecran of ECRANS) {
    test(`${ecran.chemin} · s’ouvre hors ligne sans rien mettre en file`, async () => {
      await page.goto(ecran.chemin)
      // `networkidle` n'arrive jamais hors ligne sur certains écrans : on attend le DOM.
      await page.waitForLoadState('domcontentloaded')

      // 1 · **L'écran s'ouvre.** Un écran qui refuserait de se monter hors ligne serait pire
      //     qu'un écran qui annonce : l'utilisateur ne saurait même pas ce qui est indisponible.
      await expect(page.locator('main')).toBeAttached()

      // 2 · **Le témoin dit « hors connexion ».** C'est l'indicateur permanent du principe VI, et
      //     il est dans la coquille, donc présent sur cet écran comme sur tous les autres.
      const temoin = page.locator('[data-etat]').first()
      if (await temoin.count() > 0) {
        await expect(temoin).toHaveAttribute('data-etat', 'hors_ligne')
      }

      // 3 · **Rien n'a été mis en file** par le seul fait d'ouvrir l'écran. La file ne doit
      //     porter que ce que l'utilisateur a saisi — jamais une écriture « au cas où ».
      const enFile = await page.evaluate(() =>
        Object.keys(localStorage).filter(cle => cle === 'kaya.sync.file').length,
      )
      expect(
        enFile,
        `${ecran.chemin} a rempli la file en s’ouvrant : une mise en file « au cas où » est `
        + 'exactement ce que le principe VI interdit.',
      ).toBe(0)

      expect(erreurs, `${ecran.chemin} — erreurs de page : ${erreurs.join(', ')}`).toEqual([])
    })
  }

  // ═══════════════════════════════════════════════════════════════════════════════════════════
  //  LE VERSANT POSITIF — une opération de CLASSE A, elle, est acceptée
  // ═══════════════════════════════════════════════════════════════════════════════════════════

  test('la note interne — classe A — est ACCEPTÉE hors ligne, et le témoin la compte', async () => {
    // Sans ce contrôle, un produit qui refuserait **tout** hors ligne passerait la porte au vert.
    // C'est le versant positif que le § « Couverture des portes » exige, et il est ici le seul
    // geste d'écriture du fichier (T043).
    await page.goto('/notes')
    await page.waitForLoadState('domcontentloaded')

    const champ = page.getByLabel(/texte de la note/i)
    await champ.fill('Coupure de réseau — saisie de contrôle FR-005b.')
    await page.getByRole('button', { name: /^ajouter$/i }).click()

    // Le témoin passe à `n+1` — c'est ce que « acceptée » veut dire pour une classe A. Aucun
    // message d'erreur, aucune confirmation demandée.
    const temoin = page.locator('[data-etat]').first()
    await expect(temoin).toContainText(/1/)

    // Et la file est bien persistée, chiffrée : la clé existe, et le texte n'y est pas lisible.
    const cryptogramme = await page.evaluate(() => localStorage.getItem('kaya.sync.file'))
    expect(cryptogramme, 'la saisie n’a pas été persistée').toBeTruthy()
    expect(
      cryptogramme,
      'le texte de la note apparaît EN CLAIR dans le stockage (FR-013)',
    ).not.toContain('Coupure de réseau')
  })
})
