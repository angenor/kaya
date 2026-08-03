// @vitest-environment happy-dom
/**
 * **Les déclencheurs d'envoi — et les DEUX preuves dues par `surRetourPremierPlan`.**
 *
 * # Pourquoi ce fichier existe séparément d'`amorcage.spec.ts`
 *
 * L'exigence 6 du § « Couverture des portes » demande **deux preuves** pour toute fonction
 * d'amorçage : un test qui l'**exerce**, et un test qui vérifie qu'elle est **appelée dans le
 * parcours réel**. `amorcage.spec.ts` apporte la seconde — pour les fonctions qu'il sait voir.
 *
 * `surRetourPremierPlan` n'en fait pas partie, et la limite est écrite dans son en-tête : c'est
 * une **méthode** de `PlatformAdapter`, dont l'appel s'écrit
 * `adaptateurCourant().surRetourPremierPlan(tenter)`. Le motif du harnais rejette délibérément les
 * appels précédés d'un point — c'est ce qui empêche `file.viderFile()` de compter comme un appel
 * de `viderFile`.
 *
 * Les deux preuves lui sont donc apportées ici, et elles sont explicites :
 *
 * | Preuve | Le test |
 * |---|---|
 * | Elle **fonctionne** | `surRetourPremierPlan` rappelle au retour au premier plan, et le désabonnement l'arrête |
 * | Elle est **appelée dans le parcours réel** | `brancherEnvoi` l'appelle sur l'adaptateur — vérifié en observant l'adaptateur, pas en lisant du texte |
 *
 * # Ce qui est vérifié en plus, et qui est le cœur de R-09
 *
 * **Aucune minuterie de scrutation.** Une file vide n'arme rien ; une coupure n'arme rien non
 * plus — le retour du réseau a son propre signal, et battre la mesure pendant quatre-vingt-dix
 * minutes viderait la batterie d'un Android d'entrée de gamme, c'est-à-dire de la cible.
 */

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

import { adaptateurCourant } from '../core/platform/courant'
import { adaptateurWeb } from '../core/platform/web'
import {
  brancherEnvoi,
  brancherFile,
  debrancherEnvoi,
  declencherEnvoi,
  FileLocale,
  type Envoyeur,
} from '../core/sync'
import { CLE_RAFRAICHISSEMENT, effacerSession, poserSession } from '../core/auth'

import { entreeDeTest } from './commun/classes'

const BASE = 'http://localhost:8080'
const fetchOriginal = globalThis.fetch

/** Un envoyeur qui acquitte tout et compte ses passages. */
function envoyeurQuiAcquitte(journal: string[]): Envoyeur {
  return async (entree) => {
    journal.push(`envoi:${entree.id}`)
    return { acquittee: true, statut: 201, code: '' }
  }
}

/** Le serveur répond au rafraîchissement — sans quoi `viderFile` s'arrête avant d'envoyer. */
function serveurQuiRafraichit(journal: string[]): void {
  globalThis.fetch = (async () => {
    journal.push('rafraichissement')
    return new Response(
      JSON.stringify({
        acces: 'acces-frais',
        rafraichissement: 'rafraichissement-frais',
        expire_dans_s: 3600,
        permissions: [],
        etablissements: ['etb-1'],
        compte: {
          compte_id: '018f0000-0000-7000-8000-000000000001',
          tenant_id: '018f0000-0000-7000-8000-0000000000aa',
          etablissement_actif: 'etb-1',
        },
      }),
      { status: 200, headers: { 'content-type': 'application/json' } },
    )
  }) as typeof fetch
}

beforeEach(async () => {
  localStorage.clear()
  effacerSession()
  brancherFile(null)
  debrancherEnvoi()
  Object.defineProperty(navigator, 'onLine', { configurable: true, get: () => true })
  await adaptateurCourant().stockageSecurise.ecrire(CLE_RAFRAICHISSEMENT, 'jeton-range')
  poserSession(
    {
      compteId: '018f0000-0000-7000-8000-000000000001',
      tenantId: '018f0000-0000-7000-8000-0000000000aa',
      etablissementId: null,
      permissions: [],
      etablissements: [],
    },
    'acces',
    3600,
  )
})

afterEach(() => {
  globalThis.fetch = fetchOriginal
  debrancherEnvoi()
  brancherFile(null)
  effacerSession()
  localStorage.clear()
  vi.restoreAllMocks()
})

// =================================================================================================
//  PREUVE 1 — la capacité fonctionne
// =================================================================================================

describe('surRetourPremierPlan — la capacité elle-même', () => {
  it('rappelle au retour au premier plan, et le désabonnement l’arrête', () => {
    let rappels = 0
    const desabonner = adaptateurWeb.surRetourPremierPlan(() => {
      rappels += 1
    })

    // Le changement d'onglet.
    Object.defineProperty(document, 'visibilityState', {
      configurable: true,
      get: () => 'visible',
    })
    document.dispatchEvent(new Event('visibilitychange'))
    expect(rappels, 'le retour d’onglet n’a rien déclenché').toBe(1)

    // Le retour de fenêtre — **l'autre signal**. Aminata passe de l'application au clavier de la
    // caisse et revient : n'écouter que le premier laisserait la file pleine dans ce cas-là.
    window.dispatchEvent(new Event('focus'))
    expect(rappels, 'le retour de fenêtre n’a rien déclenché').toBe(2)

    // **Le désabonnement rend une fonction, jamais `void`** — un écouteur qu'on ne peut pas
    // retirer fait fuir la mémoire à chaque navigation.
    expect(typeof desabonner).toBe('function')
    desabonner()

    document.dispatchEvent(new Event('visibilitychange'))
    window.dispatchEvent(new Event('focus'))
    expect(rappels, 'l’écouteur survit au désabonnement : il fuit').toBe(2)
  })

  it('le départ vers l’arrière-plan ne déclenche RIEN', () => {
    let rappels = 0
    const desabonner = adaptateurWeb.surRetourPremierPlan(() => {
      rappels += 1
    })

    Object.defineProperty(document, 'visibilityState', {
      configurable: true,
      get: () => 'hidden',
    })
    document.dispatchEvent(new Event('visibilitychange'))

    // Réagir au départ déclencherait un envoi au moment précis où l'utilisateur quitte
    // l'application — c'est-à-dire au pire moment.
    expect(rappels).toBe(0)
    desabonner()
  })
})

// =================================================================================================
//  PREUVE 2 — elle est appelée dans le parcours réel
// =================================================================================================

describe('brancherEnvoi APPELLE la capacité sur l’adaptateur', () => {
  it('un retour au premier plan déclenche un envoi, sans qu’aucun test ne le simule à la main', async () => {
    const journal: string[] = []
    serveurQuiRafraichit(journal)

    const file = new FileLocale()
    file.enfiler(entreeDeTest())
    brancherFile(file)

    // Le geste réel du produit — celui que le plugin d'amorçage fait au démarrage.
    brancherEnvoi(envoyeurQuiAcquitte(journal), BASE)

    Object.defineProperty(document, 'visibilityState', {
      configurable: true,
      get: () => 'visible',
    })
    document.dispatchEvent(new Event('visibilitychange'))
    await new Promise(resoudre => setTimeout(resoudre, 0))

    expect(
      journal.filter(l => l.startsWith('envoi:')),
      'le retour au premier plan n’a rien envoyé : `brancherEnvoi` n’a pas abonné la capacité de '
      + 'l’adaptateur, et le déclencheur QUI DOIT SUFFIRE SEUL ne suffit pas',
    ).toHaveLength(1)

    // Et l'ordre du point de sortie unique tient : rafraîchir AVANT d'envoyer.
    expect(journal[0]).toBe('rafraichissement')
    expect(file.enAttente).toBe(0)
  })

  it('le retour du RÉSEAU déclenche aussi — le second déclencheur', async () => {
    const journal: string[] = []
    serveurQuiRafraichit(journal)

    const file = new FileLocale()
    file.enfiler(entreeDeTest())
    brancherFile(file)
    brancherEnvoi(envoyeurQuiAcquitte(journal), BASE)

    window.dispatchEvent(new Event('online'))
    await new Promise(resoudre => setTimeout(resoudre, 0))

    expect(journal.filter(l => l.startsWith('envoi:'))).toHaveLength(1)
  })

  it('après débranchement, plus aucun déclencheur ne réagit', async () => {
    const journal: string[] = []
    serveurQuiRafraichit(journal)

    const file = new FileLocale()
    file.enfiler(entreeDeTest())
    brancherFile(file)

    const debrancher = brancherEnvoi(envoyeurQuiAcquitte(journal), BASE)
    debrancher()

    window.dispatchEvent(new Event('online'))
    document.dispatchEvent(new Event('visibilitychange'))
    await new Promise(resoudre => setTimeout(resoudre, 0))

    expect(journal, 'un déclencheur survit au débranchement').toHaveLength(0)
    expect(file.enAttente).toBe(1)
  })
})

// =================================================================================================
//  R-09 — aucune minuterie de scrutation
// =================================================================================================

describe('aucune minuterie de scrutation', () => {
  it('une file VIDE n’arme aucun minuteur', async () => {
    vi.useFakeTimers()
    try {
      const journal: string[] = []
      serveurQuiRafraichit(journal)
      brancherFile(new FileLocale())
      brancherEnvoi(envoyeurQuiAcquitte(journal), BASE)

      await declencherEnvoi(envoyeurQuiAcquitte(journal), BASE)

      expect(
        vi.getTimerCount(),
        'un minuteur est armé sur une file vide : c’est une scrutation, et elle coûte la batterie '
        + 'd’un service entier sur un Android d’entrée de gamme',
      ).toBe(0)
    }
    finally {
      vi.useRealTimers()
    }
  })

  it('une COUPURE n’arme aucun minuteur — le retour du réseau a son propre signal', async () => {
    vi.useFakeTimers()
    try {
      Object.defineProperty(navigator, 'onLine', { configurable: true, get: () => false })

      const journal: string[] = []
      const file = new FileLocale()
      file.enfiler(entreeDeTest())
      brancherFile(file)

      const resultat = await declencherEnvoi(envoyeurQuiAcquitte(journal), BASE)

      expect(resultat).toMatchObject({ issue: 'hors_ligne' })
      expect(
        vi.getTimerCount(),
        'un minuteur bat la mesure pendant une coupure. Une coupure de service dure '
        + 'quatre-vingt-dix minutes ; le retour du réseau, lui, se signale tout seul.',
      ).toBe(0)
      expect(file.enAttente).toBe(1)
    }
    finally {
      vi.useRealTimers()
    }
  })

  it('un échec réseau, LUI, arme un réessai — et un seul', async () => {
    vi.useFakeTimers()
    try {
      const journal: string[] = []
      serveurQuiRafraichit(journal)

      const file = new FileLocale()
      file.enfiler(entreeDeTest())
      brancherFile(file)

      // `statut: null` — le réseau n'a pas porté l'appel, **rien n'a été décidé côté serveur**.
      const envoyeurMuet: Envoyeur = async () => ({ acquittee: false, statut: null, code: 'reseau' })
      await declencherEnvoi(envoyeurMuet, BASE)

      expect(
        vi.getTimerCount(),
        'aucun réessai armé après un échec réseau : la file attendrait le prochain déclencheur '
        + 'naturel, ce qui peut être long',
      ).toBe(1)
      expect(file.enAttente).toBe(1)
    }
    finally {
      vi.useRealTimers()
    }
  })
})

// =================================================================================================
//  La quarantaine — un refus définitif ne bloque pas ce qui suit
// =================================================================================================

describe('un refus définitif sort de la file et ne bloque pas la suite', () => {
  it('la première écriture est refusée, les deux suivantes partent quand même', async () => {
    const journal: string[] = []
    serveurQuiRafraichit(journal)

    const file = new FileLocale()
    const refusee = entreeDeTest({ texte: 'saisie fautive' })
    file.enfiler(refusee)
    file.enfiler(entreeDeTest({ texte: 'deux' }))
    file.enfiler(entreeDeTest({ texte: 'trois' }))
    brancherFile(file)

    const envoyeur: Envoyeur = async entree =>
      entree.id === refusee.id
        ? { acquittee: false, statut: 422, code: 'validation' }
        : { acquittee: true, statut: 201, code: '' }

    await declencherEnvoi(envoyeur, BASE)

    expect(
      file.enAttente,
      'une écriture refusée définitivement bloque la file : tout le service resterait à quai pour '
      + 'une seule saisie fautive',
    ).toBe(0)
    expect(file.enQuarantaine).toBe(1)
    expect(file.quarantaine()[0]?.code).toBe('validation')
  })

  it('la quarantaine ne compte PAS dans les écritures en attente', () => {
    const file = new FileLocale()
    const entree = entreeDeTest()
    file.enfiler(entree)
    file.mettreEnQuarantaine(entree.id, 'validation', new Date().toISOString())

    // Refuser une déconnexion pour une entrée que le serveur ne reprendra jamais bloquerait le
    // terminal — et « passer la main » est le geste qui rend l'audit exact.
    expect(file.enAttente).toBe(0)
    expect(file.enQuarantaine).toBe(1)
  })

  it('le geste de relance la remet en file, avec un compteur remis à zéro', () => {
    const file = new FileLocale()
    const entree = entreeDeTest({ tentatives: 7 })
    file.enfiler(entree)
    file.mettreEnQuarantaine(entree.id, 'validation', new Date().toISOString())

    file.relancerDepuisQuarantaine(entree.id)

    expect(file.enAttente).toBe(1)
    expect(file.enQuarantaine).toBe(0)
    // C'est une décision humaine nouvelle, pas la suite de la série qui avait échoué.
    expect(file.lister()[0]?.tentatives).toBe(0)
  })
})
