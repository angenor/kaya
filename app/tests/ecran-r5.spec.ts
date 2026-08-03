// @vitest-environment happy-dom
/**
 * ★ **`R5` — Fiche client et recherche.** Les cinq propriétés qu'une relecture ne verrait pas.
 *
 * | # | Ce qui est vérifié | Ce qu'une relecture manquerait |
 * |---|---|---|
 * | **1** | La recherche est **débattue** : une frappe de cinq lettres ne produit **qu'un** appel | Cinq requêtes dont quatre périmées, sur un réseau qui les fait payer |
 * | **2** | ★ Une réponse **périmée** est jetée | La réponse d'un préfixe arrivée en retard écrase celle de la saisie complète — défaut intermittent, invisible en développement |
 * | **3** | La troncature se **dit** | Une liste coupée en silence : l'opérateur crée un doublon, qui ne se voit qu'au moment où deux historiques divergent |
 * | **4** | Sans `sej.client.gerer`, la création est **absente du HTML rendu** | Un `disabled` se retire depuis la console du navigateur |
 * | **5** | Un historique illisible **n'efface pas** la fiche | Un rôle portant `sej.client.lire` sans `heb.sejour.lire` verrait un écran vide au lieu d'une fiche sans historique — comportement voulu, pas erreur |
 *
 * # Le cas 2 est celui qui coûte le plus cher à trouver après coup
 *
 * Il ne se reproduit que si une réponse lente précède une réponse rapide — donc jamais en
 * développement, où le serveur est local. En production il donne une liste qui ne correspond pas
 * à ce qui est écrit dans le champ, et l'opérateur conclut que sa recherche « ne marche pas ».
 */

import { mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'

import EcranClients from '../modules/sejours/EcranClients.vue'
import fr from '../core/i18n/fr.json'

// =================================================================================================
//  Doublures
// =================================================================================================

vi.mock('~/core/platform', async (importOriginal) => ({
  ...(await importOriginal<typeof import('~/core/platform')>()),
  useEtatReseau: () => ({ value: 'connecte' as const }),
}))

/** Les termes réellement envoyés au serveur — c'est ce que mesure le cas 1. */
const termes: string[] = []

/** Réponses différées, pilotées par le test : c'est ce qui rend le cas 2 reproductible. */
const differees = new Map<string, { resoudre: () => void }>()

let historiqueEchoue = false

vi.mock('../modules/sejours/donnees', async (importer) => {
  const reel = await importer<typeof import('../modules/sejours/donnees')>()
  return {
    ...reel,
    chercherClients: vi.fn(async (_contexte, terme: string) => {
      termes.push(terme)
      // Un terme inscrit dans `differees` attend que le test le relâche.
      if (differees.has(terme)) {
        await new Promise<void>((resoudre) => {
          differees.set(terme, { resoudre })
        })
      }
      return {
        clients: [{ id: `cl-${terme}`, nom: `Client ${terme}`, piece_enregistree: false }],
        tronque: terme === 'a',
      }
    }),
    lireFicheClient: vi.fn(async (_contexte, clientId: string) => ({
      id: clientId,
      nom: 'Bakayoko',
      telephone: '+2250707123456',
      cree_le: '2026-07-01T10:00:00Z',
      modifie_le: '2026-07-01T10:00:00Z',
      preferences: [
        { id: 'p1', personne_id: clientId, texte: 'allergique aux arachides', cree_le: '2026-07-02T10:00:00Z' },
      ],
    })),
    chargerHistoriqueClient: vi.fn(async () => {
      if (historiqueEchoue) throw new Error('403')
      return [{
        sejour: { id: 's1', statut: 'clos', etablissement_id: 'e1', ouvert_le: '2026-07-24T14:00:00Z' },
        client_nom: 'Bakayoko',
        nombre_personnes: 2,
        unite_id: 'u1',
        fin_prevue: '2026-07-28T12:00:00Z',
        total_mineur: 180_000,
        devise: 'XOF',
      }]
    }),
  }
})

function traduire(cle: string, valeurs?: Record<string, unknown>): string {
  const brut = cle
    .split('.')
    .reduce<unknown>((noeud, part) => (noeud as Record<string, unknown>)?.[part], fr)
  if (typeof brut !== 'string') return cle
  return brut.replace(/\{(\w+)\}/g, (_, nom: string) => String(valeurs?.[nom] ?? ''))
}

;(globalThis as Record<string, unknown>).useI18n = () => ({ t: traduire, locale: { value: 'fr' } })

function monter(permissions: string[] = ['sej.client.lire', 'sej.client.gerer']) {
  return mount(EcranClients, {
    props: {
      contexte: { baseUrl: 'http://test', jeton: 'x' } as never,
      permissions,
    },
    global: {
      mocks: { useI18n: () => ({ t: traduire, locale: { value: 'fr' } }) },
      config: {
        globalProperties: { useI18n: () => ({ t: traduire, locale: { value: 'fr' } }) },
      },
    },
  })
}

/** Fait avancer le temps du débat, puis laisse les promesses se résoudre. */
async function apresLeDebat(ecran: ReturnType<typeof monter>): Promise<void> {
  await vi.advanceTimersByTimeAsync(300)
  await Promise.resolve()
  await ecran.vm.$nextTick()
}

// =================================================================================================
//  Les cinq propriétés
// =================================================================================================

describe('R5 — la fiche client et la recherche', () => {
  beforeEach(() => {
    termes.length = 0
    differees.clear()
    historiqueEchoue = false
    vi.useFakeTimers()
  })

  /**
   * ★ **Une frappe de cinq lettres ne produit qu'UN appel.**
   *
   * Sans débat, « Bakay » en produirait cinq, dont quatre sont périmées à leur arrivée. Sur le
   * réseau d'Abengourou, ce sont quatre allers-retours facturés pour rien — et quatre occasions de
   * la course du cas suivant.
   */
  it('débat la recherche : cinq lettres, un seul appel', async () => {
    const ecran = monter()
    const champ = ecran.find('input')

    for (const prefixe of ['B', 'Ba', 'Bak', 'Baka', 'Bakay']) {
      await champ.setValue(prefixe)
      await vi.advanceTimersByTimeAsync(40)
    }
    await apresLeDebat(ecran)

    expect(termes).toEqual(['Bakay'])
  })

  /**
   * ★ **Une réponse périmée est JETÉE.**
   *
   * Le test bloque la réponse de `a`, laisse partir celle de `ab`, puis relâche `a`. Sans
   * estampille, la liste finirait sur les résultats de `a` alors que le champ porte `ab` — et
   * l'opérateur conclurait que la recherche ne marche pas.
   */
  it('jette la réponse d\'un préfixe arrivée APRÈS celle de la saisie complète', async () => {
    const ecran = monter()
    const champ = ecran.find('input')

    // `a` part et reste en vol.
    differees.set('a', { resoudre: () => {} })
    await champ.setValue('a')
    await apresLeDebat(ecran)
    expect(termes).toEqual(['a'])

    // `ab` part et revient.
    await champ.setValue('ab')
    await apresLeDebat(ecran)
    expect(termes).toEqual(['a', 'ab'])
    expect(ecran.find('[data-client="cl-ab"]').exists()).toBe(true)

    // ★ `a` revient MAINTENANT — et ne doit rien écraser.
    differees.get('a')!.resoudre()
    await apresLeDebat(ecran)

    expect(
      ecran.find('[data-client="cl-a"]').exists(),
      'la réponse du préfixe « a » a écrasé celle de « ab » : la liste ne correspond plus à ce qui '
      + 'est écrit dans le champ. Le défaut ne se reproduit qu\'en réseau lent, jamais en local.',
    ).toBe(false)
    expect(ecran.find('[data-client="cl-ab"]').exists()).toBe(true)
  })

  /**
   * ★ **La troncature se dit.** Une liste coupée en silence pousse à créer un doublon, qui ne se
   * verra qu'au moment où deux historiques divergent — et qui ne se fusionne pas après coup.
   */
  it('déclare une liste tronquée', async () => {
    const ecran = monter()
    await ecran.find('input').setValue('a')
    await apresLeDebat(ecran)

    expect(ecran.find('[data-troncature]').exists()).toBe(true)
    expect(ecran.find('[data-troncature]').text()).toBe(fr.sejours.clients.resultats_tronques)
  })

  /**
   * ★ **Sans `sej.client.gerer`, la création est absente du HTML rendu**, jamais grisée.
   */
  it('sans `sej.client.gerer`, l\'action de création est absente — jamais grisée', () => {
    const ecran = monter(['sej.client.lire'])
    expect(ecran.find('[data-action="creer-fiche"]').exists()).toBe(false)
    expect(ecran.findAll('[disabled]')).toHaveLength(0)
  })

  /**
   * ★ **Un historique illisible n'efface pas la fiche.**
   *
   * Un rôle qui porte `sej.client.lire` sans `heb.sejour.lire` doit voir une fiche **sans**
   * historique : c'est le comportement voulu pour un compte de portée restreinte, pas une erreur.
   * Un `Promise.all` — au lieu de `allSettled` — aurait rejeté l'ensemble et affiché un écran
   * vide, ce qui se lit « cette fiche n'existe pas ».
   */
  it('affiche la fiche même quand l\'historique est refusé', async () => {
    historiqueEchoue = true
    const ecran = monter()
    await ecran.find('input').setValue('bak')
    await apresLeDebat(ecran)

    await ecran.find('[data-client="cl-bak"]').trigger('click')
    await apresLeDebat(ecran)

    expect(ecran.text()).toContain('Bakayoko')
    expect(ecran.findAll('[data-preference]')).toHaveLength(1)
    expect(ecran.findAll('[data-sejour-historique]')).toHaveLength(0)
    expect(ecran.text()).toContain(fr.sejours.fiche.sans_sejour)
  })

  /**
   * La fiche complète : préférences **et** historique, du plus récent au plus ancien.
   *
   * ⚠️ **Aucun cumul de montants.** `R5` dérive de `R7` **sans son bloc de total** : additionner
   * les séjours afficherait un chiffre qui ressemble à un solde, et l'exploitant y chercherait ce
   * que le client doit — que ce cycle ne calcule pas.
   */
  it('rend les préférences et l\'historique, sans jamais cumuler de total', async () => {
    const ecran = monter()
    await ecran.find('input').setValue('bak')
    await apresLeDebat(ecran)
    await ecran.find('[data-client="cl-bak"]').trigger('click')
    await apresLeDebat(ecran)

    expect(ecran.findAll('[data-preference]')).toHaveLength(1)
    expect(ecran.findAll('[data-sejour-historique]')).toHaveLength(1)
    expect(ecran.text()).toContain('allergique aux arachides')
    expect(ecran.text()).not.toContain(fr.sejours.note.total_provisoire)
    expect(ecran.text()).not.toContain(fr.sejours.note.total_arrete)
  })

  /**
   * ★ **Le numéro de pièce n'est pas affiché d'office.**
   *
   * Sa lecture est journalisée (FR-012, famille `consultation_piece_identite`). L'exposer à chaque
   * ouverture de fiche ferait tracer des consultations que personne n'a voulues, et **noierait les
   * vraies** — ce qui vide le journal de son intérêt sans que rien ne le signale.
   */
  it('n\'affiche pas le numéro de pièce d\'office', async () => {
    const ecran = monter()
    await ecran.find('input').setValue('bak')
    await apresLeDebat(ecran)
    await ecran.find('[data-client="cl-bak"]').trigger('click')
    await apresLeDebat(ecran)

    expect(ecran.text()).toContain('Bakayoko')
    expect(ecran.text()).not.toContain('numero_piece')
  })
})
