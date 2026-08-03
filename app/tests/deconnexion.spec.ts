// @vitest-environment happy-dom
/**
 * **Passer la main — ce que le pied de coquille fait vraiment.**
 *
 * # Le défaut que ce fichier ferme
 *
 * `fermerSession()` a vécu un cycle entier dans `core/auth`, exportée, testée par
 * `auth-session.spec.ts`, **et appelée nulle part**. Il n'existait aucun moyen de quitter sa
 * session. Sur un terminal de comptoir, c'est le journal d'audit qui devient faux : les actions de
 * Yao entrent au nom d'Aminata, et le registre dont le cadrage §8.3 dit qu'il est « ce que le
 * propriétaire achète » attribue les actes à la mauvaise personne.
 *
 * `amorcage.spec.ts` vérifie que la fonction est **branchée** ; ce fichier vérifie que le geste
 * **fait ce qu'il annonce**. Les deux propriétés sont distinctes : un bouton câblé sur une
 * fonction qui n'appellerait pas le serveur laisserait le jeton de rafraîchissement vivant
 * quatre-vingt-dix jours.
 *
 * # Les cinq propriétés vérifiées
 *
 * 1. **La garde de file répond « vide » faute de FILE, et le test le constate** — au lieu de
 *    supposer qu'une file vide et une file absente sont la même chose. Le versant positif suit :
 *    branchée avec des entrées, la même fonction rend le compte réel.
 * 2. **Une file non vide REFUSE le geste** : rien n'est appelé, rien n'est purgé, on ne navigue
 *    pas. Le refus est annoncé **avant** toute destruction.
 * 3. **Le geste appelle `DELETE /api/v1/session`** — la révocation côté serveur, sans laquelle le
 *    jeton rangé resterait rejouable.
 * 4. **Le stockage local est purgé**, toutes clés `kaya.` confondues (principe VI, cadrage §11.5
 *    règle 5) — et la purge est **terminée avant** la navigation.
 * 5. **Le bouton est ABSENT sans session**, jamais grisé (principe VII).
 *
 * Plus deux contrôles statiques, sur le modèle d'`ecran-r0.spec.ts` : aucune couleur littérale,
 * aucun stockage de navigateur nommé hors de `PlatformAdapter`.
 *
 * # Ce que ce fichier NE couvre pas, et qui l'est ailleurs
 *
 * La **réactivité au changement de route** — le bouton qui apparaît après connexion et disparaît
 * après déconnexion — dépend du routeur, que `@vue/test-utils` contourne. C'est précisément la
 * classe de défaut du cycle 003. Elle est couverte par **P-22**, en navigateur réel, sur les deux
 * moteurs (`tests-e2e/parcours-reel.spec.ts`, section « passer la main »).
 */

import { flushPromises, mount } from '@vue/test-utils'
import { readFileSync } from 'node:fs'
import { join } from 'node:path'
import { afterEach, beforeEach, describe, expect, it } from 'vitest'

import { CLE_RAFRAICHISSEMENT, effacerSession, poserSession } from '../core/auth'
import fr from '../core/i18n/fr.json'
import { adaptateurCourant } from '../core/platform/courant'
import { brancherFile, ecrituresEnAttente, FileLocale, fileBranchee } from '../core/sync'
import { entreeDeTest } from './commun/classes'
import Coquille from '../layouts/default.vue'

const SOURCE = readFileSync(join(process.cwd(), 'layouts/default.vue'), 'utf8')

const fetchOriginal = globalThis.fetch
const BASE = 'http://localhost:8080'

/** Traduction depuis le catalogue français réel — un faux qui rendrait la clé ne prouverait rien. */
function traduire(cle: string): string {
  const brut = cle
    .split('.')
    .reduce<unknown>((noeud, part) => (noeud as Record<string, unknown>)?.[part], fr)
  return typeof brut === 'string' ? brut : cle
}

/** Ce que le test observe du monde extérieur : appels partis, adresses visitées. */
interface Journal {
  readonly appels: { url: string, methode: string }[]
  readonly navigations: string[]
  /** Clés `kaya.` encore présentes **au moment où la navigation est demandée**. */
  readonly stockageALaNavigation: string[][]
}

function poserGlobaux(journal: Journal): void {
  const g = globalThis as Record<string, unknown>
  g.useI18n = () => ({ t: traduire })
  g.useRuntimeConfig = () => ({ public: { apiBaseUrl: BASE } })
  g.useRoute = () => ({ fullPath: '/' })
  g.navigateTo = async (cible: string) => {
    // **L'ordre est vérifié ici, pas déduit.** On relève l'état du stockage à l'instant où la
    // navigation part : si la purge était lancée sans être attendue, une clé `kaya.` serait
    // encore là et le contrôle 4 le dirait.
    journal.stockageALaNavigation.push(clesKaya())
    journal.navigations.push(cible)
    return undefined
  }

  globalThis.fetch = (async (entree: string | URL | Request, init?: RequestInit) => {
    const url = entree instanceof Request ? entree.url : String(entree)
    const methode = entree instanceof Request ? entree.method : (init?.method ?? 'GET')
    journal.appels.push({ url, methode })
    return new Response(null, { status: 204 })
  }) as typeof fetch
}

function journalVierge(): Journal {
  return { appels: [], navigations: [], stockageALaNavigation: [] }
}

/** Les clés de l'application dans le stockage — **lues à la main**, pas par l'adaptateur. */
function clesKaya(): string[] {
  return Object.keys(localStorage).filter(cle => cle.startsWith('kaya.'))
}

function monter() {
  return mount(Coquille, {
    global: {
      mocks: { useI18n: () => ({ t: traduire }) },
      config: { globalProperties: { useI18n: () => ({ t: traduire }) } },
    },
  })
}

/**
 * Une file portant **une** écriture de classe A.
 *
 * `note_etablissement.creee` est le seul type que `TYPES_CLASSE_A` déclare — la file en refuse
 * tout autre à l'exécution, et le type de `enfiler` refuse toute charge non marquée à la
 * compilation.
 */
function fileAvecUneEcriture(id: string): FileLocale {
  const file = new FileLocale()
  file.enfiler(entreeDeTest({ id }))
  return file
}

/** Ouvre une session en mémoire **et** range un jeton, comme une connexion réussie le ferait. */
async function ouvrirUneSession(): Promise<void> {
  poserSession(
    {
      compteId: '018f0000-0000-7000-8000-000000000001',
      tenantId: '018f0000-0000-7000-8000-0000000000aa',
      etablissementId: null,
      permissions: [],
      etablissements: [],
    },
    'jeton-acces-de-test',
    3600,
  )
  await adaptateurCourant().stockageSecurise.ecrire(CLE_RAFRAICHISSEMENT, 'jeton-rafraichissement')
  // Une seconde clé, qui n'est pas le jeton : la purge doit l'emporter aussi (principe VI).
  await adaptateurCourant().stockageSecurise.ecrire('cache.comptes', '[{"nom":"Adjoua"}]')
}

let journal: Journal

beforeEach(() => {
  journal = journalVierge()
  poserGlobaux(journal)
  localStorage.clear()
  effacerSession()
  brancherFile(null)
})

afterEach(() => {
  globalThis.fetch = fetchOriginal
  brancherFile(null)
  effacerSession()
  localStorage.clear()
})

// =================================================================================================
//  1. La garde de file — les deux versants
// =================================================================================================

describe('la garde de file hors-ligne', () => {
  it('répond 0 parce qu’AUCUNE file n’est branchée — constaté, pas supposé', () => {
    // La distinction que ce contrôle existe pour écrire : `ecrituresEnAttente()` rend 0, et ce
    // n'est PAS le constat qu'une file est vide. Le jour où SYN-01 appellera `brancherFile`,
    // `amorcage.spec.ts` échouera tant que sa ligne restera « due » — et cette assertion-ci
    // basculera dans le même changement.
    expect(
      fileBranchee(),
      'une file est branchée : la garde ne mesure plus ce que ce test décrit.\n'
      + 'Faire passer la ligne « brancherFile » de amorcage.spec.ts à « branché », et remplacer '
      + 'ce contrôle par celui de la file réelle.',
    ).toBe(false)
    expect(ecrituresEnAttente()).toBe(0)
  })

  it('rend le compte RÉEL dès qu’une file est branchée — le versant positif', () => {
    // Sans ce contrôle, `ecrituresEnAttente()` pourrait rendre `0` en dur : tous les autres tests
    // passeraient, et la garde ne garderait rien le jour où la file arrive. C'est le corollaire
    // du versant positif de l'exigence 4 du § « Couverture des portes ».
    brancherFile(fileAvecUneEcriture('018f0000-0000-7000-8000-0000000000f1'))

    expect(fileBranchee()).toBe(true)
    expect(ecrituresEnAttente()).toBe(1)
  })
})

// =================================================================================================
//  2. Le refus — rien n'est détruit
// =================================================================================================

describe('une file non vide refuse le geste', () => {
  it('n’appelle rien, ne purge rien, ne navigue pas, et dit pourquoi', async () => {
    await ouvrirUneSession()
    brancherFile(fileAvecUneEcriture('018f0000-0000-7000-8000-0000000000f2'))

    const coquille = monter()
    await coquille.find('footer button').trigger('click')
    await flushPromises()

    expect(journal.appels, 'un appel est parti alors que la file n’est pas vide').toEqual([])
    expect(journal.navigations, 'on a quitté l’écran malgré le refus').toEqual([])
    expect(clesKaya().sort(), 'le stockage a été purgé malgré le refus')
      .toEqual(['kaya.auth.rafraichissement', 'kaya.cache.comptes'])

    // La phrase vient du catalogue, pas du code : c'est celle du lexique.
    expect(coquille.find('[role="alert"]').text())
      .toBe(fr.deconnexion.refus.en_attente)
  })
})

// =================================================================================================
//  3, 4. Le geste nominal
// =================================================================================================

describe('passer la main', () => {
  it('révoque côté serveur, purge le stockage, puis navigue vers /connexion', async () => {
    await ouvrirUneSession()
    expect(clesKaya(), 'la mise en place du test n’a rien rangé').toHaveLength(2)

    const coquille = monter()
    await coquille.find('footer button').trigger('click')
    await flushPromises()

    // 3 — la révocation côté serveur. Sans elle, le jeton rangé resterait rejouable
    // quatre-vingt-dix jours, et « passer la main » ne serait qu'un effacement local.
    expect(journal.appels).toHaveLength(1)
    expect(journal.appels[0]!.methode).toBe('DELETE')
    expect(journal.appels[0]!.url).toBe(`${BASE}/api/v1/session`)

    // 4 — la purge, ET son ordre. `stockageALaNavigation` est relevé au moment exact où
    // `navigateTo` est appelée : une purge lancée sans être attendue y laisserait une clé.
    expect(journal.navigations).toEqual(['/connexion'])
    expect(
      journal.stockageALaNavigation[0],
      'des clés `kaya.` subsistaient au moment de la navigation — la purge n’a pas été attendue.\n'
      + 'Le middleware global peut alors reprendre une session qu’on est en train de détruire.',
    ).toEqual([])
    expect(clesKaya()).toEqual([])
  })

  it('le libellé rendu est celui du lexique, et le jargon n’atteint pas l’écran', async () => {
    await ouvrirUneSession()
    const coquille = monter()

    const bouton = coquille.find('footer button')
    expect(bouton.text()).toBe(fr.deconnexion.action)
    expect(bouton.attributes('title')).toBe(fr.deconnexion.effet)

    // `docs/design/lexique.md` : « session », « jeton » et « JWT » n'apparaissent jamais, et
    // « se déconnecter » a été écarté nommément au profit du geste réel.
    //
    // Le contrôle porte sur ce que l'utilisateur LIT — texte rendu et attributs de libellé —, pas
    // sur le HTML brut : celui-ci porte les commentaires du gabarit, où le mot « session » est à sa
    // place. Élargir la cible ici obligerait à surveiller son propre vocabulaire de commentaire, ce
    // qui n'apprendrait rien sur l'interface.
    const lu = [coquille.text(), bouton.attributes('title') ?? ''].join(' ')
    expect(lu).not.toMatch(/jeton|JWT|session|déconnect/i)
  })
})

// =================================================================================================
//  5. Absent sans session
// =================================================================================================

describe('sans session', () => {
  it('le pied de coquille n’existe pas dans le HTML rendu — absent, jamais grisé', () => {
    const coquille = monter()

    expect(coquille.find('footer').exists(), 'le pied est rendu sans session').toBe(false)
    expect(coquille.findAll('button')).toHaveLength(0)
    // Un `disabled` ici serait la faute : il apprendrait à l'utilisateur qu'une partie du produit
    // lui est refusée, sans lui dire laquelle ni jusqu'à quand (principe VII).
    expect(coquille.html()).not.toContain('disabled')
  })

  it('la coquille garde son `<main>` unique', () => {
    // La propriété que le lot d'amorçage a payée cher : `verifierCoquille` de P-22 compte les
    // `<main>`, et 0 comme 2 sont des défauts. Le pied ne doit pas en ajouter un.
    expect(monter().findAll('main')).toHaveLength(1)
  })
})

// =================================================================================================
//  Contrôles statiques — en plus des portes P-17 et P-15
// =================================================================================================

describe('la coquille respecte les portes de style', () => {
  it('aucune couleur littérale', () => {
    expect(SOURCE).not.toMatch(/#[0-9a-fA-F]{3,8}\b/)
    expect(SOURCE).not.toMatch(/\b(?:rgba?|hsla?|oklch)\s*\(/)
  })

  it('aucun stockage de navigateur ni pont natif nommé', () => {
    // La coquille purge le stockage — c'est l'endroit du produit où la tentation d'appeler
    // `localStorage` directement est la plus forte. Tout passe par `PlatformAdapter` (porte P-15).
    const code = SOURCE.replace(/\/\*[\s\S]*?\*\//g, '').replace(/<!--[\s\S]*?-->/g, '')
    expect(code).not.toMatch(/localStorage|sessionStorage|indexedDB|__TAURI__/)
  })
})
