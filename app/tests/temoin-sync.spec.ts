// @vitest-environment happy-dom
/**
 * **Le témoin de synchronisation — douze combinaisons, et la règle qui n'en souffre aucune.**
 *
 * `docs/design/composants.md` §10 le nomme « le composant le plus important du produit ». Un
 * composant dont on dit cela mérite d'être vérifié dans **tous** ses états, dans les deux thèmes
 * et dans les deux langues : **3 × 2 × 2 = douze combinaisons**, et le décompte est asserté pour
 * qu'une boucle qui rétrécirait se voie.
 *
 * # Les quatre règles du composant, et ce que chacune coûterait si on la relâchait
 *
 * | Règle | Ce qu'elle coûte si on la relâche |
 * |---|---|
 * | **Jamais de pourcentage** | Un pourcentage suppose un total connu — faux : la file grandit pendant qu'elle se vide. Et il ne répond pas à « mon travail est-il parti ? » |
 * | **Une forme ET une phrase par état** | Sur un 1366 × 768 délavé par le soleil, vert et orange se ressemblent |
 * | **Passage hors ligne instantané** | Une pastille qui fondrait du vert au rouge ferait douter de l'instant où l'état a changé |
 * | **Le jargon n'atteint pas l'écran** | « Dégradé » est un terme d'ingénieur ; le lexique dit « Connexion faible » |
 *
 * # Ce que ce fichier ne couvre PAS
 *
 * Le rendu **en navigateur réel** — que le témoin soit visible sur chaque route, dans les deux
 * thèmes. C'est P-22 (`tests-e2e/parcours-reel.spec.ts`), et la distinction est celle du cycle
 * 003 : un composant monté en test n'est pas un écran atteint.
 */

import { mount } from '@vue/test-utils'
import { readFileSync } from 'node:fs'
import { join } from 'node:path'
import { afterEach, beforeEach, describe, expect, it } from 'vitest'

import en from '../core/i18n/en.json'
import fr from '../core/i18n/fr.json'
import { brancherFile, FileLocale } from '../core/sync'
import Temoin from '../core/design-system/TemoinSynchronisation.vue'

import { entreeDeTest } from './commun/classes'

const SOURCE = readFileSync(join(process.cwd(), 'core/design-system/TemoinSynchronisation.vue'), 'utf8')

/** Les trois états, tels que l'adaptateur les rapporte. */
const ETATS = ['connecte', 'degrade', 'hors_ligne'] as const
const THEMES = ['clair', 'sombre'] as const
const LANGUES = { fr, en } as const

type Etat = (typeof ETATS)[number]

/** Traduction depuis un catalogue réel — un faux qui rendrait la clé ne prouverait rien. */
function traducteur(catalogue: typeof fr | typeof en) {
  return (cle: string, valeurs?: Record<string, unknown>, choix?: number): string => {
    const brut = cle
      .split('.')
      .reduce<unknown>((noeud, part) => (noeud as Record<string, unknown>)?.[part], catalogue)
    if (typeof brut !== 'string') {
      return cle
    }
    // Pluriel à la mode de `vue-i18n` : « singulier | pluriel ».
    const formes = brut.split('|').map(f => f.trim())
    const forme = formes.length > 1 && choix !== undefined
      ? (formes[choix === 1 ? 0 : 1] ?? formes[0]!)
      : formes[0]!
    return forme.replace(/\{(\w+)\}/g, (_, nom: string) => String(valeurs?.[nom] ?? ''))
  }
}

/** Pose l'état du réseau **par le chemin réel** : `navigator.onLine` et l'observateur d'appels. */
async function poserEtatReseau(etat: Etat): Promise<void> {
  const { observerAppel, oublierObservations } = await import('../core/platform/observateur-appels')
  oublierObservations()

  Object.defineProperty(navigator, 'onLine', {
    configurable: true,
    get: () => etat !== 'hors_ligne',
  })

  if (etat === 'degrade') {
    // Un appel qui n'aboutit pas alors que la plateforme se dit « en ligne » : c'est le cas
    // d'Abengourou, celui pour lequel le troisième état existe.
    observerAppel({ abouti: false, dureeMs: 120 })
  }
}

function monter(catalogue: typeof fr | typeof en, compact = false) {
  const t = traducteur(catalogue)
  const g = globalThis as Record<string, unknown>
  g.useI18n = () => ({ t })

  return mount(Temoin, {
    props: { compact },
    global: { mocks: { useI18n: () => ({ t }) } },
  })
}

beforeEach(() => {
  brancherFile(null)
})

afterEach(async () => {
  brancherFile(null)
  const { oublierObservations } = await import('../core/platform/observateur-appels')
  oublierObservations()
})

// =================================================================================================
//  Les douze combinaisons
// =================================================================================================

describe('trois états × deux thèmes × deux langues', () => {
  it('les douze combinaisons rendent une FORME et une PHRASE, jamais un pourcentage', async () => {
    let combinaisons = 0

    for (const etat of ETATS) {
      for (const theme of THEMES) {
        for (const [code, catalogue] of Object.entries(LANGUES)) {
          await poserEtatReseau(etat)
          // Le thème passe par la classe `.dark` sur la racine du document — le mécanisme réel
          // (`core/theme`), pas une seconde palette.
          document.documentElement.classList.toggle('dark', theme === 'sombre')

          const temoin = monter(catalogue as typeof fr)
          const html = temoin.html()
          const texte = temoin.text()

          // **1 · La forme.** L'état est porté par un attribut de données ET par une icône, pas
          // seulement par une couleur.
          expect(
            temoin.attributes('data-etat'),
            `[${etat}/${theme}/${code}] l’état n’est pas porté par la forme`,
          ).toBe(etat)
          expect(html, `[${etat}/${theme}/${code}] aucune icône de forme`).toMatch(/ph-cloud/)

          // **2 · La phrase**, tirée du catalogue réel de la langue.
          const attendue = (catalogue as typeof fr).reseau[etat]
          expect(
            texte,
            `[${etat}/${theme}/${code}] la phrase du lexique n’est pas rendue`,
          ).toContain(attendue)

          // **3 · Jamais de pourcentage.** La règle explicite du composant 10.
          expect(
            texte,
            `[${etat}/${theme}/${code}] un pourcentage est affiché : « un nombre d’écritures et `
            + 'une heure, jamais une barre de progression »',
          ).not.toContain('%')

          combinaisons += 1
          temoin.unmount()
        }
      }
    }

    // Le décompte : une boucle qui rétrécirait passerait au vert sans rien vérifier.
    expect(combinaisons, 'le balayage n’a pas couvert les douze combinaisons').toBe(12)
    document.documentElement.classList.remove('dark')
  })
})

// =================================================================================================
//  L'attente l'emporte sur l'état du réseau
// =================================================================================================

describe('le nombre d’écritures en attente', () => {
  it('remplace la phrase d’état — dire « Enregistré » quand quatre attendent serait un mensonge', async () => {
    await poserEtatReseau('connecte')

    const file = new FileLocale()
    for (let rang = 0; rang < 4; rang += 1) {
      file.enfiler(entreeDeTest({ texte: `commande ${rang}` }))
    }
    brancherFile(file)

    const temoin = monter(fr)

    expect(temoin.text()).toContain('4')
    expect(
      temoin.text(),
      'le témoin annonce « Enregistré » alors que quatre écritures attendent',
    ).not.toBe(fr.reseau.connecte)
    expect(temoin.text()).not.toContain('%')
  })

  it('le mot « file » et le jargon technique n’atteignent jamais l’écran', async () => {
    await poserEtatReseau('degrade')
    const file = new FileLocale()
    file.enfiler(entreeDeTest())
    brancherFile(file)

    const rendu = monter(fr).text().toLowerCase()

    // Le lexique est catégorique : « idempotence, rejeu, file d'attente — n'apparaît jamais.
    // L'utilisateur voit *en attente d'envoi* et un nombre. »
    for (const proscrit of ['file', 'rejeu', 'idempot', 'synchronis', 'dégradé', 'quarantaine']) {
      expect(rendu, `le mot « ${proscrit} » atteint l’écran`).not.toContain(proscrit)
    }
  })
})

// =================================================================================================
//  Les contrôles statiques — ce qu'on ne peut pas voir dans un rendu
// =================================================================================================

describe('les règles du composant, lues dans sa source', () => {
  it('le passage hors ligne est INSTANTANÉ — aucune transition sur la pastille', () => {
    // Une pastille qui fondrait doucement du vert au rouge ferait douter de l'instant où l'état a
    // changé. Un témoin de sécurité ne s'anime pas quand il passe au rouge.
    expect(
      SOURCE,
      'la pastille ne déclare pas `transition-none` : le passage hors ligne serait animé',
    ).toContain('transition-none')

    // Et aucune transition de couleur n'est posée dessus.
    expect(SOURCE).not.toMatch(/rounded-pleine[^"]*transition-colors/)
  })

  it('le pouls est LENT et réservé à l’état connecté', () => {
    // 2,4 s — `--animate-pulse-reseau`. Il rassure ; il n'alerte pas. Le poser sur les trois états
    // ferait clignoter un témoin rouge, ce qui est exactement l'inverse de son rôle.
    expect(SOURCE).toContain('animate-pulse-reseau')
    expect(
      SOURCE,
      'le pouls n’est pas conditionné à l’état connecté',
    ).toMatch(/v-if="forme === 'connecte'"[\s\S]{0,200}animate-pulse-reseau/)
  })

  it('aucune couleur littérale — les jetons, et rien d’autre', () => {
    const litteraux = SOURCE.match(/#[0-9a-fA-F]{3,8}\b|rgb\(|hsl\(/g) ?? []
    expect(
      litteraux,
      'une couleur littérale dans le composant le plus important du produit : le mode sombre '
      + 'passe par des jetons dont les VALEURS changent sous `.dark`, jamais par une palette bis',
    ).toEqual([])
  })

  it('la variante compacte garde sa phrase pour les lecteurs d’écran', async () => {
    await poserEtatReseau('hors_ligne')
    const temoin = monter(fr, true)

    // Un témoin muet pour un lecteur d'écran ne dirait plus si le travail est en sécurité — ce qui
    // est exactement son objet.
    expect(temoin.attributes('aria-label')).toBe(fr.reseau.hors_ligne)
    expect(temoin.text()).not.toContain(fr.reseau.hors_ligne)
  })
})
