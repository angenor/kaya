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

import { mount } from '@vue/test-utils'
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

/**
 * Monte le composant avec un `useI18n` minimal.
 *
 * Le module `@nuxtjs/i18n` fournit `useI18n` en auto-import ; hors du pipeline Nuxt il faut le
 * poser soi-même. La traduction lit le **catalogue réel** `core/i18n/fr.json` : un faux qui
 * renverrait la clé ferait passer le test alors que le libellé serait absent du catalogue.
 */
function monter(services: ServiceActif[]) {
  return mount(SectionServices, {
    props: { services, referentiel: REFERENTIEL },
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

    const html = mount(SectionServices, {
      props: { services: [], referentiel: avecProvision },
    }).html()

    expect(html).not.toContain('SPA')
  })
})
