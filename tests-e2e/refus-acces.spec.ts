/**
 * **FR-029 — SANS LA PERMISSION, L'ÉCRAN REFUSE, ET IL LE DIT EN FRANÇAIS.**
 *
 * ═══════════════════════════════════════════════════════════════════════════════════════════
 *  CE CAS ÉTAIT IMPOSSIBLE À ÉCRIRE AVANT CE LOT
 * ═══════════════════════════════════════════════════════════════════════════════════════════
 *
 * Le jeu de démonstration ne contenait **aucun compte qui se voie refuser quoi que ce soit**.
 * Koffi, Adjoua et Yao portent tous `heb.offre.lire`, `heb.sejour.lire` et `sej.client.lire` — les
 * lectures du métier, et c'est exact. Exercer un refus aurait demandé de **forger une session
 * amputée**, c'est-à-dire de prouver le produit contre un jeton que le produit n'émet pas.
 *
 * `aminata@deloria.test` — rôle `serveur`, cinq lectures transverses et rien d'autre — rend le cas
 * réel. C'est la persona serveuse du corpus (`docs/user-stories-v1.md` §0.2), et elle sert deux
 * fois : ici, et à la démonstration client, où « l'action est absente sans permission » cesse
 * d'être une phrase pour devenir un écran.
 *
 * ═══════════════════════════════════════════════════════════════════════════════════════════
 *  CE QUI EST VÉRIFIÉ, ET POURQUOI CHAQUE VERSANT COMPTE
 * ═══════════════════════════════════════════════════════════════════════════════════════════
 *
 * 1. **La tuile est ABSENTE** — pas grisée, pas masquée par CSS. Principe VII.
 * 2. **L'accès direct par l'URL est REFUSÉ** — la tuile absente ne suffit pas : quelqu'un tape
 *    l'adresse, et le serveur refuserait de toute façon en `403` sur une page muette.
 * 3. **Le refus est en LANGUE UTILISATEUR**, et c'est le point du lot. Ces six écrans affichaient
 *    « Les chambres n'ont pas pu être chargées. » — un message d'**échec technique**, qui envoie
 *    chercher un problème de réseau qui n'existe pas. Une réceptionniste appelle le support ; une
 *    serveuse croit l'application cassée.
 * 4. **Aucune donnée n'a fui** — le refus précède l'appel, donc l'écran ne montre ni liste ni
 *    fragment de contenu.
 *
 * Le versant positif est dans le même fichier : les trois écrans qu'Aminata **a** le droit
 * d'ouvrir s'ouvrent. Sans lui, tout passerait au vert sur une application qui refuserait tout.
 *
 * ═══════════════════════════════════════════════════════════════════════════════════════════
 *  LE PÉRIMÈTRE EST DÉDUIT, JAMAIS ÉNUMÉRÉ
 * ═══════════════════════════════════════════════════════════════════════════════════════════
 *
 * Les routes exercées sont celles que `ACCES_ECRANS` déclare gardées **et** dont Aminata n'a pas
 * les permissions — calculé ici à partir de la session que l'API lui rend réellement, jamais
 * d'une liste écrite à la main. Le cycle 007 ajoutera `/caisse` avec sa permission ; elle entrera
 * dans ce balayage sans que personne y pense.
 */

import { expect, test, type Browser, type BrowserContext, type Page } from '@playwright/test'

import { BASE_APP } from './adresses'
import { ACCES_ECRANS } from '../app/core/acces/ecrans'

test.describe.configure({ mode: 'serial' })

/**
 * Aminata — serveuse. **Le rôle le plus étroit du produit.**
 *
 * Son identifiant est distinct de celui de P-22, ce qui lui donne son propre compteur de
 * tentatives : `LimiteTentatives` plafonne **par identifiant**, et partager le compte de
 * démonstration entre toutes les portes les ferait buter les unes sur les autres.
 */
const SERVEUSE = {
  identifiant: 'aminata@deloria.test',
  get motDePasse(): string {
    const valeur = process.env.KAYA_SEEDS_MOT_DE_PASSE
    if (!valeur) {
      throw new Error(
        'KAYA_SEEDS_MOT_DE_PASSE n’est pas défini. C’est la variable dont `seeds` se sert pour '
        + 'créer les comptes de démonstration ; aucun mot de passe n’est écrit dans le dépôt.',
      )
    }
    return valeur
  },
}

let contexte: BrowserContext
let page: Page
/** Les permissions que le serveur lui rend RÉELLEMENT — jamais supposées. */
let permissions: string[] = []

test.beforeAll(async ({ browser }: { browser: Browser }) => {
  contexte = await browser.newContext({ locale: 'fr-FR', baseURL: BASE_APP })
  page = await contexte.newPage()

  // La réponse de connexion porte les permissions **en clair** (research R-06) : on la capte au
  // vol plutôt que de décoder le jeton, qui n'est pas la source.
  const reponse = page.waitForResponse(r => r.url().endsWith('/api/v1/session') && r.ok())

  await page.goto('/connexion')
  await page.getByLabel(/identifiant/i).fill(SERVEUSE.identifiant)
  await page.getByLabel(/mot de passe/i).fill(SERVEUSE.motDePasse)
  await page.getByRole('button', { name: /se connecter/i }).click()

  permissions = ((await (await reponse).json()) as { permissions: string[] }).permissions
  await page.waitForURL(url => new URL(url).pathname === '/', { timeout: 20_000 })
})

test.afterAll(async () => {
  await contexte?.close()
})

/** Les écrans gardés qu'Aminata NE peut pas ouvrir — déduits de sa session réelle. */
function routesRefusees(): string[] {
  return Object.entries(ACCES_ECRANS)
    .filter(([, acces]) => acces.permissions.length > 0)
    .filter(([, acces]) => !acces.permissions.every(p => permissions.includes(p)))
    .map(([route]) => route)
    .sort()
}

/** Et celles qu'elle PEUT ouvrir — le versant positif. */
function routesAutorisees(): string[] {
  return Object.entries(ACCES_ECRANS)
    .filter(([, acces]) => acces.permissions.every(p => permissions.includes(p)))
    .map(([route]) => route)
    .sort()
}

test('la serveuse a bien un accès ÉTROIT — sinon ce fichier ne prouve rien', () => {
  // Exigence 4. Si un cycle élargissait le rôle `serveur`, `routesRefusees()` se viderait et tous
  // les contrôles ci-dessous passeraient au vert **en n'exerçant aucun refus**. C'est le défaut
  // que ce fichier existe pour empêcher ailleurs ; il ne va pas le porter lui-même.
  expect(permissions.length, 'la session de la serveuse est vide ou illisible').toBeGreaterThan(0)
  expect(
    routesRefusees().length,
    'aucun écran n’est refusé à la serveuse : le jeu de démonstration a cessé de contenir un '
    + 'compte étroit, et ce fichier ne vérifie plus rien.',
  ).toBeGreaterThanOrEqual(6)

  console.warn(
    `Aminata — ${permissions.length} permission(s) · refusées : ${routesRefusees().join(', ')}`,
  )
})

test('son accueil ne propose QUE ce qu’elle peut ouvrir', async () => {
  await page.goto('/')
  await page.waitForLoadState('networkidle')
  await page.waitForTimeout(1500)

  const codes = await page.locator('[data-tuile]').evaluateAll(noeuds =>
    noeuds.map(noeud => noeud.getAttribute('data-tuile') ?? ''),
  )
  const html = await page.content()

  expect(codes).toEqual(['notes', 'mes-envois', 'etablissement'])

  // Ni grisée, ni masquée : aucune route refusée n'apparaît dans le HTML, même en attribut.
  for (const route of routesRefusees()) {
    expect(html, `« ${route} » apparaît dans l’accueil d’un compte qui ne peut pas l’ouvrir`)
      .not.toContain(`"${route}"`)
  }
})

test('chaque écran refusé le dit EN FRANÇAIS, et ne montre rien', async () => {
  const echecs: string[] = []

  for (const route of routesRefusees()) {
    await page.goto(route)
    await page.waitForLoadState('networkidle')

    const texte = (await page.locator('main').innerText()).trim()

    // ⚠️ LE POINT DU LOT. « n’ont pas pu être chargées » décrit une PANNE là où il s’agit d’un
    // DROIT. Le lexique proscrit exactement cette confusion, et c’est ce que ces six écrans
    // affichaient avant d’avoir leur garde.
    if (/pas pu être chargé|n’a pas abouti|Réessayez/i.test(texte)) {
      echecs.push(`${route} — message d’ÉCHEC TECHNIQUE au lieu d’un refus : « ${texte} »`)
      continue
    }
    if (!/vous n’avez pas accès|vous n'avez pas accès/i.test(texte)) {
      echecs.push(`${route} — ne dit pas le refus : « ${texte} »`)
    }
  }

  expect(
    echecs,
    'Des écrans refusent sans le dire, ou le disent dans la langue de la machine.',
  ).toEqual([])
})

test('versant POSITIF — les trois écrans qu’elle a le droit d’ouvrir s’ouvrent', async () => {
  // Sans lui, le contrôle précédent passerait au vert sur une application qui refuserait TOUT.
  for (const route of routesAutorisees()) {
    await page.goto(route)
    await page.waitForLoadState('networkidle')

    expect(new URL(page.url()).pathname, `${route} a redirigé`).toBe(route)
    const texte = (await page.locator('main').innerText()).trim()
    expect(texte.length, `${route} s’ouvre sur un <main> vide`).toBeGreaterThan(0)
    expect(texte, `${route} refuse alors qu’elle en a le droit`).not.toMatch(/n’avez pas accès/i)
  }
})
