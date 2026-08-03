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

import { readFileSync } from 'node:fs'
import { join } from 'node:path'

import { describe, expect, it } from 'vitest'

import {
  FileLocale,
  OperationRefusee,
  estTypeClasseA,
  marquerClasseA,
  operationRealisable,
  TYPES_CLASSE_A,
} from '../core/sync'
import { CONTEXTE_TEST } from './commun/classes'

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
      // Les deux champs du cycle 005. `contexte` est FIGÉ à la saisie — changer d'établissement
      // actif pendant une coupure ne réattribue jamais une écriture déjà enfilée.
      contexte: CONTEXTE_TEST,
      tentatives: 0,
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
        contexte: CONTEXTE_TEST,
        tentatives: 0,
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
      contexte: CONTEXTE_TEST,
      tentatives: 0,
    })

    expect(file.enAttente).toBe(1)
  })

  // ═══════════════════════════════════════════════════════════════════════════════════════════
  //  Les DEUX champs du cycle 005 — le compilateur les exige, et c'est le point
  // ═══════════════════════════════════════════════════════════════════════════════════════════

  it('une entrée SANS contexte ne compile pas', () => {
    const file = new FileLocale()

    // @ts-expect-error — `contexte` est OBLIGATOIRE depuis le cycle 005.
    //
    // Le défaut qu'il empêche est silencieux : Aminata saisit quatre commandes hors ligne, change
    // d'établissement actif, le réseau revient. Sans ce champ, les quatre partent sur le mauvais
    // établissement — rien n'échoue, le serveur accepte, et la faute ne se voit qu'à la clôture,
    // quand le chiffre d'affaires de l'un manque et que celui de l'autre est faux.
    //
    // Si cette ligne cessait d'être une erreur, `@ts-expect-error` en deviendrait une lui-même,
    // et ce test échouerait. C'est la mécanique de P-13 côté type.
    file.enfiler({
      id: '0198c4a0-0000-7000-8000-000000000104',
      type: 'note_etablissement.creee',
      horodatageClient: HORODATAGE,
      charge: marquerClasseA({ texte: 'sans contexte' }, 'A4 — jeu d’essai'),
      tentatives: 0,
    })

    expect(file.enAttente).toBe(1)
  })

  it('une entrée SANS compteur de tentatives ne compile pas', () => {
    const file = new FileLocale()

    // @ts-expect-error — `tentatives` est OBLIGATOIRE depuis le cycle 005. Il alimente
    // l'intervalle croissant de réessai et le diagnostic de `S1` : « cette écriture a été tentée
    // sept fois » est une information portable au support, là où « en attente » ne dit rien.
    file.enfiler({
      id: '0198c4a0-0000-7000-8000-000000000105',
      type: 'note_etablissement.creee',
      horodatageClient: HORODATAGE,
      charge: marquerClasseA({ texte: 'sans tentatives' }, 'A4 — jeu d’essai'),
      contexte: CONTEXTE_TEST,
    })

    expect(file.enAttente).toBe(1)
  })

  it('la file ne porte TOUJOURS aucun champ de jeton — l’absence est ce qui l’empêche', () => {
    const file = new FileLocale()

    file.enfiler({
      id: '0198c4a0-0000-7000-8000-000000000106',
      type: 'note_etablissement.creee',
      horodatageClient: HORODATAGE,
      charge: marquerClasseA({ texte: 'aucun jeton' }, 'A4 — jeu d’essai'),
      contexte: CONTEXTE_TEST,
      tentatives: 0,
      // @ts-expect-error — il n'y a AUCUN champ où ranger un jeton, et c'est délibéré : un jeton
      // mis en file serait périmé au retour (soixante minutes de jeton contre quatre-vingt-dix de
      // coupure), et le ranger prolongerait la durée de vie d'un secret sur un terminal qu'on peut
      // perdre. Le compilateur refuse la propriété excédentaire — c'est l'absence de champ qui
      // l'empêche, pas une discipline.
      acces: 'Bearer jeton-qui-ne-doit-pas-exister',
    })

    // Le versant statique : l'interface elle-même ne déclare aucun champ de secret. Le contrôle
    // ci-dessus refuse la propriété au type ; celui-ci refuse qu'on l'ajoute à l'interface.
    const source = readFileSync(join(process.cwd(), 'core/sync/classes.ts'), 'utf8')
    const interfaceEntree = source.match(/export interface EntreeFile<[\s\S]*?\n\}/)?.[0] ?? ''
    expect(interfaceEntree, 'l’interface EntreeFile est introuvable').not.toBe('')
    for (const secret of ['acces', 'bearer', 'jeton', 'token', 'rafraichissement']) {
      expect(
        interfaceEntree.toLowerCase().includes(`${secret}:`),
        `« ${secret} » est devenu un champ d’EntreeFile : un secret entre en file.`,
      ).toBe(false)
    }
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

// =================================================================================================
//  P-13 — le cycle 003 n'ajoute aucun type de classe A, et purge à la déconnexion
// =================================================================================================

describe('P-13 — le cycle 003 n’ajoute aucun type de classe A', () => {
  /**
   * Les dix entités du cycle 003.
   *
   * **`journal_audit` est de classe A au registre** — c'est la seule du cycle — et pourtant elle
   * ne figure pas dans `TYPES_CLASSE_A`. Ce n'est pas une contradiction : le contrat n'expose
   * **aucun point d'entrée d'écriture** d'audit (research R-17), donc le front n'a rien à mettre
   * en file. Une entrée voyage avec l'opération qu'elle trace, et les sept opérations de ce cycle
   * sont toutes de classe C.
   *
   * Le jour où une opération de classe A tracera une entrée hors ligne — l'ouverture de tiroir
   * d'IMP-01 —, c'est **son** type qui entrera dans la liste, pas `journal_audit`.
   */
  const ENTITES_CYCLE_003 = [
    'personne',
    'compte',
    'methode_authentification',
    'role',
    'permission',
    'role_permission',
    'compte_role',
    'employe',
    'appareil_enrole',
  ]

  it('aucune entité de classe C du cycle 003 ne figure dans TYPES_CLASSE_A', () => {
    const intrus = TYPES_CLASSE_A.filter(type =>
      ENTITES_CYCLE_003.some(entite => type.startsWith(`${entite}.`)),
    )

    expect(
      intrus,
      'Ces types appartiennent à des entités de classe C. Une élévation de privilège mise en '
      + 'file serait la pire faute possible du produit : un terminal s’accorderait un rôle '
      + 'pendant une coupure, puis le synchroniserait — et aurait obtenu un droit que personne '
      + 'n’a accordé.\n  '
      + intrus.join('\n  '),
    ).toEqual([])
  })

  it('les sept opérations d’écriture du cycle sont refusées à l’enfilement', () => {
    // Les sept de `docs/registre-classes-offline.md` §5.2, nommées comme le front les nomme.
    const SEPT = [
      'personne.creee',
      'compte.cree',
      'compte.etat_change',
      'compte.mot_de_passe_change',
      'compte_role.attribue',
      'compte_role.retire',
      'session.revoquee',
    ]

    let refusees = 0
    for (const type of SEPT) {
      const file = new FileLocale()
      expect(() =>
        file.enfiler({
          id: '0198c4a0-0000-7000-8000-000000000201',
          type,
          horodatageClient: HORODATAGE,
          charge: marquerClasseA({}, 'marque abusive — le test la provoque'),
          contexte: CONTEXTE_TEST,
          tentatives: 0,
        }),
      ).toThrow(OperationRefusee)
      refusees += 1
    }

    // **Décompte** : une liste qui rétrécit passerait au vert sans rien vérifier.
    expect(refusees).toBe(7)
  })

  it('la file locale ne porte TOUJOURS que le type du module doré', () => {
    // Le cycle 003 crée dix tables et n'ajoute aucun type de classe A à la file. Si un cycle en
    // ajoute un légitimement, ce test échoue et l'oblige à venir écrire ici pourquoi.
    expect(TYPES_CLASSE_A).toEqual(['note_etablissement.creee'])
  })

  it('aucune entité du cycle 003 n’est réalisable hors ligne, même en réseau dégradé', () => {
    for (const entite of ENTITES_CYCLE_003) {
      expect(operationRealisable(`${entite}.creee`, 'hors_ligne')).toBe(false)
      // L'état `degrade` compte COMME hors ligne : `navigator.onLine` dit qu'une interface est
      // active, pas que le serveur répond.
      expect(operationRealisable(`${entite}.creee`, 'degrade')).toBe(false)
    }
  })
})
