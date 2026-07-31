/**
 * **SC-005** — aucun service inactif n'apparaît nulle part.
 *
 * Deux niveaux, et le second n'est pas redondant :
 *
 * 1. la **fonction de sélection**, pure, testée sans DOM — elle dit ce que l'écran doit montrer ;
 * 2. le **rendu**, dans `ecran-g1.spec.ts` — il constate que le HTML produit ne porte aucun
 *    libellé ni code des autres services.
 *
 * Le premier seul testerait l'intention. « Un service inactif est **absent**, jamais grisé »
 * (principe VII) est une propriété du **résultat** : un composant peut la perdre — par un `v-show`
 * au lieu d'un `v-if`, par une liste de secours, par un attribut `title` — sans que la fonction de
 * sélection change d'une ligne.
 */

import { describe, expect, it } from 'vitest'

import {
  capacitesVisibles,
  cleOrigine,
  servicesActivables,
  servicesVisibles,
  type EntreeReferentiel,
  type ServiceActif,
} from '../modules/etablissements/services-visibles'

/** Le référentiel réel : cinq modules, tous implémentés. */
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

function service(code: string, capacites: ServiceActif['capacites'] = []): ServiceActif {
  return {
    id: `id-${code}`,
    module_code: code,
    libelle_cle: `services.modules.${code}`,
    ordre: 0,
    capacites,
  }
}

describe('servicesVisibles', () => {
  it("ne rend que les services actifs, dans l'ordre du référentiel", () => {
    // Volontairement dans le désordre : c'est le référentiel qui décide de l'ordre, pas l'API.
    const actifs = [service('BAR'), service('HEBERGEMENT'), service('RESTAURATION')]

    const visibles = servicesVisibles(actifs, REFERENTIEL)

    expect(visibles.map((s) => s.module_code)).toEqual(['HEBERGEMENT', 'RESTAURATION', 'BAR'])
  })

  it("un établissement à service unique n'en montre qu'un — la résidence meublée", () => {
    const visibles = servicesVisibles([service('HEBERGEMENT')], REFERENTIEL)

    expect(visibles).toHaveLength(1)
    expect(visibles[0]?.module_code).toBe('HEBERGEMENT')
  })

  it('un code absent du référentiel est écarté plutôt que rendu muet', () => {
    // Sans libellé ni ordre, il produirait une ligne vide à une place arbitraire.
    const visibles = servicesVisibles([service('MODULE_FICTIF_TEST')], REFERENTIEL)

    expect(visibles).toHaveLength(0)
  })

  it('rend une liste vide sans erreur — un établissement sans service reste valide', () => {
    expect(servicesVisibles([], REFERENTIEL)).toEqual([])
  })
})

describe('servicesActivables', () => {
  it("ne propose jamais un service déjà actif", () => {
    const activables = servicesActivables([service('HEBERGEMENT')], REFERENTIEL)

    expect(activables.map((e) => e.code)).not.toContain('HEBERGEMENT')
    expect(activables).toHaveLength(4)
  })

  it('ne propose JAMAIS une valeur non implémentée', () => {
    // Le référentiel rend `implementee` délibérément — pour la console éditeur et pour distinguer
    // « inconnu » de « pas encore » dans un message d'erreur. La proposer à l'activation
    // garantirait un refus 422 que l'exploitant n'a aucune raison de rencontrer (FR-036).
    const avecProvision: EntreeReferentiel[] = [
      ...REFERENTIEL,
      { code: 'SPA', libelle_cle: 'services.modules.SPA', implementee: false, ordre: 60 },
    ]

    const activables = servicesActivables([], avecProvision)

    expect(activables.map((e) => e.code)).not.toContain('SPA')
    expect(activables).toHaveLength(5)
  })
})

describe('capacitesVisibles', () => {
  it('rend les capacités du service, ordre stable', () => {
    const restauration = service('RESTAURATION', [
      {
        id: 'c1',
        capacite_code: 'STOCK',
        profil_code: 'SIMPLE',
        libelle_cle: 'services.capacites.STOCK',
      },
    ])

    expect(capacitesVisibles(restauration).map((c) => c.capacite_code)).toEqual(['STOCK'])
  })

  it('une liste vide est la forme normale — la résidence meublée ne consomme rien', () => {
    expect(capacitesVisibles(service('HEBERGEMENT'))).toEqual([])
  })
})

describe('cleOrigine', () => {
  it('traduit TENANT en « vaut pour tous vos établissements »', () => {
    expect(cleOrigine('TENANT')).toBe('etablissement.origine.herite')
  })

  it('traduit tout niveau inférieur en « modifié ici »', () => {
    for (const origine of ['ETABLISSEMENT', 'MODULE', 'POINT_DE_VENTE']) {
      expect(cleOrigine(origine)).toBe('etablissement.origine.modifie_ici')
    }
  })
})
