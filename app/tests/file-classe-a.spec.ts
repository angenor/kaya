/**
 * **Porte P-13** — aucune opération B, C ou D atteignable depuis un chemin exécutable hors ligne.
 *
 * Deux niveaux de vérification, parce qu'aucun ne suffit seul :
 *
 * - **à la compilation** — la signature de `FileLocale.enfiler` refuse une charge non marquée.
 *   Vérifié par les `@ts-expect-error` ci-dessous : ils échouent si l'erreur attendue
 *   *n'apparaît pas*, ce qui fait de chacun un test à part entière ;
 * - **à l'exécution** — un type d'opération absent de `TYPES_CLASSE_A` est refusé, même marqué.
 *   Cette seconde barrière attrape la marque abusive : `marquerClasseA` appelée sur une opération
 *   qui n'est pas de classe A.
 */

import { describe, expect, it } from 'vitest'

import {
  FileLocale,
  OperationRefusee,
  estTypeClasseA,
  marquerClasseA,
  operationRealisable,
  TYPES_CLASSE_A,
} from '../core/sync'

const HORODATAGE = '2026-07-31T09:00:00.000Z'

describe('file locale — classe A seulement', () => {
  it('accepte une opération de classe A déclarée au registre', () => {
    const file = new FileLocale()

    file.enfiler({
      id: '0198c4a0-0000-7000-8000-000000000101',
      type: 'note_etablissement.creee',
      horodatageClient: HORODATAGE,
      charge: marquerClasseA(
        { texte: 'Le groupe électrogène a démarré à 19 h 40.' },
        'A4 — append-only, commutative, sans effet monétaire',
      ),
    })

    expect(file.enAttente).toBe(1)
  })

  it("refuse une opération dont le type n'est pas déclaré de classe A", () => {
    const file = new FileLocale()

    // La marque est là — c'est bien le scénario de la marque abusive : quelqu'un a appelé
    // `marquerClasseA` sur un encaissement, qui est de classe B (espèces) ou D (Mobile Money).
    const enfiler = () =>
      file.enfiler({
        id: '0198c4a0-0000-7000-8000-000000000102',
        type: 'encaissement.enregistre',
        horodatageClient: HORODATAGE,
        charge: marquerClasseA({ montant_mineur: 15500 }, 'justification abusive'),
      })

    expect(enfiler).toThrow(OperationRefusee)
    expect(enfiler).toThrow(/classe A/)
    expect(file.enAttente).toBe(0)
  })

  it('exige une justification à la marque', () => {
    expect(() => marquerClasseA({ a: 1 }, '')).toThrow(/justification/)
    expect(() => marquerClasseA({ a: 1 }, '   ')).toThrow(/justification/)
  })

  it('ne compile pas si la charge n’est pas marquée', () => {
    const file = new FileLocale()

    file.enfiler({
      id: '0198c4a0-0000-7000-8000-000000000103',
      type: 'note_etablissement.creee',
      horodatageClient: HORODATAGE,
      // @ts-expect-error — une charge non marquée doit être refusée par le compilateur.
      // C'est LA garantie de la porte P-13 : si cette ligne cessait d'être une erreur,
      // `@ts-expect-error` deviendrait lui-même une erreur, et le test échouerait.
      charge: { texte: 'charge brute, sans marque' },
    })

    expect(file.enAttente).toBe(1)
  })
})

describe('disponibilité selon l’état réseau', () => {
  it('tout est réalisable quand le réseau est là', () => {
    expect(operationRealisable('encaissement.enregistre', 'connecte')).toBe(true)
    expect(operationRealisable('note_etablissement.creee', 'connecte')).toBe(true)
  })

  it('hors ligne, seules les opérations de classe A passent', () => {
    expect(operationRealisable('note_etablissement.creee', 'hors_ligne')).toBe(true)
    expect(operationRealisable('encaissement.enregistre', 'hors_ligne')).toBe(false)
    expect(operationRealisable('compte_role.attribue', 'hors_ligne')).toBe(false)
  })

  it('réseau dégradé : même règle que hors ligne', () => {
    // Un réseau dégradé n'est pas un réseau lent qu'on peut attendre : c'est un réseau dont on ne
    // sait pas s'il portera la requête. Traiter « dégradé » comme « connecté » produirait
    // exactement l'échec après coup que le principe VI interdit.
    expect(operationRealisable('encaissement.enregistre', 'degrade')).toBe(false)
  })
})

describe('registre des types de classe A', () => {
  it('ne contient que des types réellement déclarés A au registre', () => {
    // Le registre backend est la source de vérité ; cette liste en est le reflet côté client.
    // Elle est courte à ce cycle — une seule entité de classe A existe.
    expect(TYPES_CLASSE_A).toEqual(['note_etablissement.creee'])
  })

  it('ne reconnaît aucune opération à effet monétaire', () => {
    // Les opérations à effet monétaire sont B, C ou D **sans exception** (cadrage §11.2). Ce test
    // échouera le jour où quelqu'un ajoutera un encaissement à la liste — et c'est son objet.
    for (const type of TYPES_CLASSE_A) {
      expect(type).not.toMatch(/encaissement|paiement|reglement|cloture|remise|avoir/)
    }
    expect(estTypeClasseA('encaissement.enregistre')).toBe(false)
  })
})

/**
 * **P-13, côté application — le cycle 002 n'a ajouté AUCUN type à la file locale.**
 *
 * Les onze entités du cycle sont toutes de **classe C** : référentiels, activations, points de
 * vente, configuration, identité visuelle. Aucune ne s'écrit hors ligne, donc aucune n'entre ici.
 *
 * # Pourquoi vérifier une absence
 *
 * Parce qu'une absence ne se voit pas. `TYPES_CLASSE_A` est une liste que chaque cycle est invité
 * à remplir — le commentaire qui la suit le dit explicitement. Le jour où quelqu'un y ajoutera
 * `etablissement_module.active` « pour que l'activation marche hors ligne », rien d'autre ne
 * l'arrêtera : le type compile, la file l'accepte, et l'activation d'un service partirait dans une
 * file locale alors qu'elle exige la base.
 *
 * Ce test est ce qui rend cette liste **opposable**.
 */
describe('P-13 — le cycle 002 n’ajoute aucun type de classe A', () => {
  /** Les onze entités du cycle 002, toutes de classe C au registre. */
  const ENTITES_CYCLE_002 = [
    'etablissement',
    'module_activite',
    'capacite',
    'profil_stock',
    'etablissement_module',
    'module_capacite',
    'point_de_vente',
    'table_pdv',
    'parametre_catalogue',
    'parametre_configuration',
    'branding',
  ]

  it('aucune entité du cycle 002 ne figure dans TYPES_CLASSE_A', () => {
    const intrus = TYPES_CLASSE_A.filter((type) =>
      ENTITES_CYCLE_002.some((entite) => type.startsWith(`${entite}.`)),
    )

    expect(
      intrus,
      'Ces types appartiennent à des entités de classe C : elles ne s’écrivent JAMAIS hors ligne '
      + '(docs/registre-classes-offline.md §5.1). Les mettre en file locale ferait partir une '
      + 'activation de service ou une écriture de configuration dans une file qui sera rejouée — '
      + 'alors que ces opérations exigent la base pour valider leurs contraintes.\n  '
      + intrus.join('\n  '),
    ).toEqual([])
  })

  it('la file locale ne porte encore que le type du module doré', () => {
    // Assertion de non-régression : si un cycle ajoute légitimement un type de classe A, ce test
    // échoue et l'oblige à venir écrire ici pourquoi. Sans elle, la liste grossirait sans que
    // personne ne repasse sur la question.
    expect(TYPES_CLASSE_A).toEqual(['note_etablissement.creee'])
  })

  it('aucune entité du cycle 002 n’est réalisable hors ligne', () => {
    for (const entite of ENTITES_CYCLE_002) {
      expect(
        estTypeClasseA(`${entite}.creee`),
        `« ${entite} » est de classe C : aucune de ses opérations ne doit être réalisable hors ligne`,
      ).toBe(false)
    }
  })
})
