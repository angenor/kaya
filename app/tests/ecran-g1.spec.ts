// @vitest-environment happy-dom
/**
 * **SC-005, second niveau : le HTML RENDU.**
 *
 * La fonction de sélection dit ce que l'écran doit montrer ; ce fichier constate ce qu'il montre
 * **réellement**. La distinction n'est pas théorique : un composant peut perdre la propriété « un
 * service inactif est absent » par un `v-show` au lieu d'un `v-if`, par un attribut `title`, par
 * une liste de secours dans un commentaire de gabarit — sans que la fonction de sélection change
 * d'une ligne.
 *
 * C'est la raison pour laquelle T004 a été tranchée dans le sens de l'ajout des paquets de test
 * front (gel 1.0.6) : le refuser aurait réduit SC-005 à vérifier l'intention.
 *
 * # Ce que ce fichier n'inspecte pas
 *
 * Le **rendu visuel** — couleurs, espacements, bascule clair/sombre. Un test de HTML ne peut pas
 * en juger : le mode sombre passe par des tokens dont les valeurs changent sous `.dark`, ce qui ne
 * se voit qu'à l'œil. La vérification des deux thèmes est faite section par section (T044), et les
 * portes P-16 et P-17 gardent le reste.
 */

import { flushPromises, mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'

import SectionServices from '../modules/etablissements/SectionServices.vue'
import type { EntreeReferentiel, ServiceActif } from '../modules/etablissements/services-visibles'
import fr from '../core/i18n/fr.json'

const REFERENTIEL: EntreeReferentiel[] = [
  { code: 'HEBERGEMENT', libelle_cle: 'services.modules.HEBERGEMENT', implementee: true, ordre: 10 },
  {
    code: 'RESTAURATION',
    libelle_cle: 'services.modules.RESTAURATION',
    implementee: true,
    ordre: 20,
  },
  { code: 'BAR', libelle_cle: 'services.modules.BAR', implementee: true, ordre: 30 },
  { code: 'PRESSING', libelle_cle: 'services.modules.PRESSING', implementee: true, ordre: 40 },
  {
    code: 'SALLE_REUNION',
    libelle_cle: 'services.modules.SALLE_REUNION',
    implementee: true,
    ordre: 50,
  },
]

/** Contexte d'appel — **provisoire nommé**, levé par CPT-01. Aucun appel n'est émis dans ces tests. */
const CONTEXTE = { baseUrl: 'http://localhost:8080', tenantId: 't', compteId: 'c' }

/** La permission de modifier les services — voir `bascule-service.ts`, provisoire levé par CPT-02. */
const PEUT_MODIFIER = ['etb.service.basculer']

/**
 * Monte le composant avec un `useI18n` minimal.
 *
 * Le module `@nuxtjs/i18n` fournit `useI18n` en auto-import ; hors du pipeline Nuxt il faut le
 * poser soi-même. La traduction lit le **catalogue réel** `core/i18n/fr.json` : un faux qui
 * renverrait la clé ferait passer le test alors que le libellé serait absent du catalogue.
 *
 * Les permissions valent **la liste vide par défaut** — le cas sûr, et celui qui rend observable
 * la règle du principe VII : sans droit, l'action n'existe pas dans le HTML.
 */
function monter(
  services: ServiceActif[],
  options: { referentiel?: EntreeReferentiel[], permissions?: string[] } = {},
) {
  return mount(SectionServices, {
    props: {
      services,
      referentiel: options.referentiel ?? REFERENTIEL,
      contexte: CONTEXTE,
      etablissementId: 'etb-1',
      permissions: options.permissions ?? [],
    },
    global: {
      mocks: {
        useI18n: () => ({ t: traduire }),
      },
      provide: {},
      config: {
        globalProperties: {
          useI18n: () => ({ t: traduire }),
        },
      },
    },
  })
}

/** Traduction depuis le catalogue français réel. */
function traduire(cle: string, valeurs?: Record<string, unknown>): string {
  const brut = cle
    .split('.')
    .reduce<unknown>((noeud, part) => (noeud as Record<string, unknown>)?.[part], fr)

  if (typeof brut !== 'string') return cle

  // Forme plurielle « singulier | pluriel » de vue-i18n : on prend la branche selon `n`.
  if (brut.includes('|')) {
    const branches = brut.split('|').map((b) => b.trim())
    const n = Number(valeurs?.n ?? valeurs?.services ?? 0)
    const branche = n === 1 ? branches[0] : (branches[1] ?? branches[0])
    return (branche ?? cle).replace(/\{(\w+)\}/g, (_, nom: string) => String(valeurs?.[nom] ?? ''))
  }

  return brut.replace(/\{(\w+)\}/g, (_, nom: string) => String(valeurs?.[nom] ?? ''))
}

/**
 * Compte les bandeaux d'alerte du rendu (composant 07).
 *
 * Le contrefort — `border-l-4` sur un bloc `rounded-r-lg` — est la signature du composant, et il
 * n'appartient qu'à lui : les lignes de service portent `rounded-l-xs rounded-r-xl`, pas `-lg`.
 */
function compterBandeaux(html: string): number {
  return (html.match(/rounded-r-lg border-l-4/g) ?? []).length
}

function service(code: string): ServiceActif {
  return {
    id: `id-${code}`,
    module_code: code,
    libelle_cle: `services.modules.${code}`,
    ordre: 0,
    capacites: [],
  }
}

// `useI18n` est un auto-import Nuxt : hors du pipeline, il faut l'exposer globalement.
;(globalThis as Record<string, unknown>).useI18n = () => ({ t: traduire })

describe('G1 — section « Vos services »', () => {
  it("un établissement à service unique n'affiche AUCUN des quatre autres", () => {
    const rendu = monter([service('HEBERGEMENT')])
    const html = rendu.html()

    expect(html).toContain('Hébergement')

    // **Ni le libellé, ni le code.** Un code laissé dans un attribut `data-*` ou une classe
    // suffirait à trahir un service que l'établissement n'a pas.
    for (const [code, libelle] of [
      ['RESTAURATION', 'Restauration'],
      ['BAR', 'Bar'],
      ['PRESSING', 'Pressing'],
      ['SALLE_REUNION', 'Salle de réunion'],
    ]) {
      expect(html, `le libellé « ${libelle} » ne doit pas apparaître`).not.toContain(libelle)
      expect(html, `le code « ${code} » ne doit pas apparaître`).not.toContain(code)
    }
  })

  it('un maquis affiche RESTAURATION et rien de plus', () => {
    const html = monter([service('RESTAURATION')]).html()

    expect(html).toContain('Restauration')
    expect(html).not.toContain('Hébergement')
    expect(html).not.toContain('HEBERGEMENT')
  })

  it('cinq services actifs les affichent tous les cinq', () => {
    const html = monter(REFERENTIEL.map((e) => service(e.code))).html()

    for (const libelle of [
      'Hébergement',
      'Restauration',
      'Bar',
      'Pressing',
      'Salle de réunion',
    ]) {
      expect(html, `« ${libelle} » doit apparaître`).toContain(libelle)
    }
  })

  it('le mot « capacité » n’apparaît nulle part dans le rendu', () => {
    // Le terme est un mot d'architecture. L'exploitant ne voit que la capacité concrète —
    // « Suivi du stock » — sous le service qui la consomme (docs/design/lexique.md).
    const avecStock: ServiceActif = {
      ...service('RESTAURATION'),
      capacites: [
        {
          id: 'c1',
          capacite_code: 'STOCK',
          profil_code: 'SIMPLE',
          libelle_cle: 'services.capacites.STOCK',
        },
      ],
    }

    const html = monter([avecStock]).html()

    expect(html).toContain('Suivi du stock')
    expect(html.toLowerCase()).not.toContain('capacité')
    expect(html.toLowerCase()).not.toContain('capacite')
  })

  it("n'affiche aucune valeur non implémentée parmi les services activables", () => {
    const avecProvision: EntreeReferentiel[] = [
      ...REFERENTIEL,
      { code: 'SPA', libelle_cle: 'services.modules.SPA', implementee: false, ordre: 60 },
    ]

    const html = monter([], {
      referentiel: avecProvision,
      permissions: PEUT_MODIFIER,
    }).html()

    expect(html).not.toContain('SPA')
  })
})

/**
 * **Le patron d'écriture, vu depuis le HTML rendu.**
 *
 * Ces quatre assertions portent sur ce que le principe VII et le principe VI garantissent à
 * l'utilisateur, et qu'aucune relecture ne tient dans le temps : une action refusée est **absente**,
 * pas grisée, et une action indisponible faute de réseau **le dit** au lieu de disparaître en
 * silence. Les deux se perdent d'un `:disabled` posé par réflexe.
 */
describe('G1 — le patron d’écriture (ETB-02)', () => {
  it('sans permission, les actions sont ABSENTES du HTML — pas désactivées', () => {
    const html = monter([service('HEBERGEMENT')], { permissions: [] }).html()

    expect(html, "le bouton d'ajout ne doit pas exister").not.toContain('Ajouter un service')
    expect(html, 'le bouton de retrait ne doit pas exister').not.toContain('Retirer')

    // **Le point qui distingue « absent » de « grisé ».** Un `disabled` dans le HTML signifierait
    // que l'action est là, refusée — exactement ce que le principe VII interdit.
    expect(html, 'aucun attribut `disabled` : absent, pas grisé').not.toContain('disabled')

    // Et le service, lui, reste bien affiché : c'est l'ACTION qui manque, pas le contenu.
    expect(html).toContain('Hébergement')
  })

  it('avec la permission, les deux actions apparaissent', () => {
    const html = monter([service('HEBERGEMENT')], { permissions: PEUT_MODIFIER }).html()

    expect(html).toContain('Ajouter un service')
    expect(html).toContain('Retirer')
  })

  it('hors ligne, l’action disparaît ET une phrase dit pourquoi (classe C)', () => {
    const enLigne = Object.getOwnPropertyDescriptor(
      globalThis.Navigator.prototype,
      'onLine',
    )
    Object.defineProperty(globalThis.navigator, 'onLine', { value: false, configurable: true })

    try {
      const html = monter([service('HEBERGEMENT')], { permissions: PEUT_MODIFIER }).html()

      // Ni grisé silencieux, ni file d'attente : l'opération est de classe C, elle ne se rejoue
      // pas, et l'interface l'annonce IMMÉDIATEMENT (principe VI).
      expect(html).not.toContain('Ajouter un service')
      expect(html).toContain('demande une connexion')
    }
    finally {
      // La propriété est restaurée même si l'assertion échoue : la laisser à `false` ferait
      // échouer les tests suivants pour une raison sans rapport avec ce qu'ils vérifient.
      if (enLigne) {
        Object.defineProperty(globalThis.navigator, 'onLine', enLigne)
      }
    }
  })

  it('jamais deux bandeaux empilés — même quand un refus précède une coupure', async () => {
    // **Régression réelle, trouvée à l'œil et pas ici.** La première rédaction affichait le refus
    // métier et l'avis hors-ligne l'un sous l'autre : deux `v-if` voisins, chacun correct pris
    // séparément. Le composant 07 l'interdit — « le plus grave gagne, l'autre attend ».
    const fetchOriginal = globalThis.fetch
    globalThis.fetch = (async () =>
      new Response(JSON.stringify({ code: 'desactivation_bloquee', message: 'diagnostic' }), {
        status: 422,
        headers: { 'content-type': 'application/json' },
      })) as typeof fetch

    const enLigne = Object.getOwnPropertyDescriptor(globalThis.Navigator.prototype, 'onLine')

    try {
      const rendu = monter([service('HEBERGEMENT')], { permissions: PEUT_MODIFIER })

      // 1 · un refus métier s'affiche
      await rendu.get('button').trigger('click')
      await flushPromises()
      await new Promise(resoudre => setTimeout(resoudre, 20))
      await flushPromises()
      expect(compterBandeaux(rendu.html()), 'un bandeau après le refus').toBe(1)

      // 2 · le réseau tombe pendant que le refus est encore à l'écran
      Object.defineProperty(globalThis.navigator, 'onLine', { value: false, configurable: true })
      window.dispatchEvent(new Event('offline'))
      await flushPromises()

      const html = rendu.html()
      expect(compterBandeaux(html), 'toujours UN SEUL bandeau, jamais deux').toBe(1)
      // Et c'est le plus grave qui gagne : hors ligne conditionne l'écran entier.
      expect(html).toContain('demande une connexion')
    }
    finally {
      globalThis.fetch = fetchOriginal
      if (enLigne) Object.defineProperty(globalThis.navigator, 'onLine', enLigne)
    }
  })

  it('le formulaire d’ajout n’apparaît qu’après le geste, avec son champ étiqueté', async () => {
    const rendu = monter([service('HEBERGEMENT')], { permissions: PEUT_MODIFIER })

    expect(rendu.html(), 'le champ ne précède pas le geste').not.toContain('<select')

    // Le **dernier** bouton de la section : celui du pied. Prendre le premier cliquerait sur le
    // retrait de la première ligne — la faute que ce test a réellement commise à sa rédaction.
    const boutons = rendu.findAll('button')
    await boutons[boutons.length - 1]!.trigger('click')

    const html = rendu.html()
    expect(html).toContain('<select')
    // L'étiquette est TOUJOURS visible — jamais remplacée par le texte d'invite (composant 16).
    expect(html).toContain('Service à ajouter')
    expect(html).toContain('Choisissez un service')
  })
})
