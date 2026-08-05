// @vitest-environment happy-dom
/**
 * ★ **SC-001 — le budget de gestes du passage, CONTRAINT et non constaté.**
 *
 * ═══════════════════════════════════════════════════════════════════════════════════════════════
 *  POURQUOI CE FICHIER EST ÉCRIT DANS LA MÊME TÂCHE QUE L'ÉCRAN
 *
 *  Écrit **après** `EcranPassage.vue`, il **constaterait** le nombre de gestes : quel qu'il soit,
 *  on l'inscrirait dans l'assertion et le test serait vert. Écrit **avec** l'écran, il le
 *  **contraint** — et c'est la seule forme qui protège quelque chose.
 *
 *  Le cadrage §5.6 en fait une condition d'existence du produit : *« le module de passage doit
 *  être irréprochable en rapidité (moins de 30 secondes) sinon il sera contourné »*. Un écran de
 *  comptoir contourné, c'est un cahier papier qui revient — et tout le reste du produit avec lui.
 * ═══════════════════════════════════════════════════════════════════════════════════════════════
 *
 * # Les trois contraintes, et ce que chacune empêche
 *
 * | Contrainte | Valeur | Ce qu'elle empêche |
 * |---|---|---|
 * | Interactions obligatoires | **exactement 2** | Un bouton « Confirmer » ajouté « pour la sécurité » |
 * | Champs de saisie libre obligatoires | **0** | Un nom, un téléphone, une pièce demandés AVANT la clé |
 * | Appels réseau bloquants | **au plus 1** | Une vérification « la chambre est-elle libre ? » avant l'attribution |
 *
 * La troisième n'est pas qu'une affaire de vitesse : une vérification préalable serait exactement
 * le **verrou applicatif** que le principe IV refuse — elle rendrait la double attribution
 * *improbable* au lieu d'*impossible*.
 *
 * # Ce que ce test ne mesure PAS
 *
 * **Le temps.** Ni celui de la machine — c'est `tests-e2e/passage.spec.ts`, sur deux moteurs —
 * ni celui de l'humain, qui se chronomètre au terrain et se consigne dans
 * `specs/006-…/mesures-terrain.md`. Ce fichier mesure une propriété **déterministe** de l'écran :
 * combien de gestes il exige. C'est ce qui le rend opposable en intégration continue, là où une
 * mesure de temps rougirait au hasard et serait désactivée dans le mois.
 */

import { mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'

import EcranPassage from '../modules/sejours/EcranPassage.vue'
import fr from '../core/i18n/fr.json'

// =================================================================================================
//  Doublures
// =================================================================================================

/**
 * Le réseau est **connecté** : ce test mesure le parcours nominal.
 *
 * ⚠️ **Le double porte sur `~/core/platform/reseau`, PAS sur le baril.** C'est la seule forme
 * juste : `useEtatReseau` vit dans ce module, et `~/core/platform/index.ts` **ne le réexporte
 * pas**. Ce fichier doublait le baril, ce qui **fournissait** un export inexistant — le test était
 * vert et l'écran ne se montait pas en navigateur. Le défaut a été trouvé par P-22 au cycle 006 ;
 * `tests/imports-barils.spec.ts`, né de là, le rend désormais visible en millisecondes.
 */
vi.mock('~/core/platform/reseau', () => ({
  useEtatReseau: () => ({ value: 'connecte' as const }),
}))

/** Les appels réseau, **comptés un par un** — c'est la troisième contrainte. */
const appels: string[] = []

vi.mock('../modules/sejours/ouvrir-sejour', () => ({
  TYPE_OPERATION_SEJOUR: 'hebergement_sejour.ouverture',
  ouvrirSejour: vi.fn(async () => {
    appels.push('POST /sejours')
    return {
      issue: 'succes' as const,
      sejour: {
        sejour: { id: 's1', statut: 'en_cours' },
        occupation: { id: 'o1', fin_client: '2026-08-03T17:30:00Z' },
        note: { total_mineur: 2800, devise: 'XOF', lignes: [] },
        fiche_police: { numero: 1, complete: false },
        instant_autorite: '2026-08-03T15:30:00Z',
      },
    }
  }),
  rattacherClient: vi.fn(),
}))

vi.mock('../modules/sejours/donnees', async (importer) => {
  const reel = await importer<typeof import('../modules/sejours/donnees')>()
  return {
    ...reel,
    rechargerEtatDesUnites: vi.fn(async () => {
      appels.push('GET /etat-des-unites')
      return ETAT_DES_UNITES
    }),
  }
})

const INSTANT = '2026-08-03T15:30:00Z'

const ETAT_DES_UNITES = {
  instant_autorite: INSTANT,
  unites: [
    { unite_id: 'u1', code: 'A1', categorie_id: 'c1', etage: 1, etat: 'libre', fin_prevue: null, disponible_a: null, statut_menage: 'propre', sejour_id: null },
    { unite_id: 'u2', code: 'A2', categorie_id: 'c1', etage: 1, etat: 'occupee', fin_prevue: '2026-08-03T16:10:00Z', disponible_a: null, statut_menage: 'propre', sejour_id: 's0' },
    { unite_id: 'u3', code: 'A3', categorie_id: 'c1', etage: 1, etat: 'libre', fin_prevue: null, disponible_a: null, statut_menage: 'propre', sejour_id: null },
  ],
}

const FORMULE_PASSAGE = {
  id: 'f1',
  categorie_id: 'c1',
  famille: 'PASSAGE',
  devise: 'XOF',
  prix_mineur: 1500,
  assujettie_taxe_nuitee: false,
  paliers: [
    { duree_minutes: 60, prix_mineur: 1500 },
    { duree_minutes: 120, prix_mineur: 2800 },
    { duree_minutes: 180, prix_mineur: 4000 },
    { duree_minutes: 240, prix_mineur: 5000 },
  ],
  plages: [],
}

const DONNEES = {
  etatDesUnites: ETAT_DES_UNITES,
  categories: [{ id: 'c1', nom: 'Standard', capacite_accueil: 2 }],
  formules: [FORMULE_PASSAGE],
}

/** Toutes les permissions : ce test mesure les gestes, pas les droits. */
const PERMISSIONS = ['heb.sejour.ouvrir', 'heb.sejour.lire']

function traduire(cle: string, valeurs?: Record<string, unknown>): string {
  const brut = cle
    .split('.')
    .reduce<unknown>((noeud, part) => (noeud as Record<string, unknown>)?.[part], fr)
  if (typeof brut !== 'string') return cle
  return brut.replace(/\{(\w+)\}/g, (_, nom: string) => String(valeurs?.[nom] ?? ''))
}

// `useI18n` est un auto-import Nuxt : hors du pipeline, il faut l'exposer globalement — sinon le
// `setup` des composants enfants lève, et Vue rend un `vnode` indéfini dont le message
// (« Invalid vnode type ») ne désigne pas la cause.
//
// La traduction lit le **catalogue réel** `core/i18n/fr.json` : un faux qui renverrait la clé
// ferait passer le test alors que le libellé serait absent du catalogue.
;(globalThis as Record<string, unknown>).useI18n = () => ({ t: traduire, locale: { value: 'fr' } })

function monter() {
  return mount(EcranPassage, {
    props: {
      contexte: { baseUrl: 'http://test', jeton: 'x' } as never,
      etablissementId: 'e1',
      donnees: DONNEES as never,
      permissions: PERMISSIONS,
      clientReconnu: null,
      passagesDuClient: 0,
    },
    global: {
      mocks: { useI18n: () => ({ t: traduire, locale: { value: 'fr' } }) },
      provide: {},
      config: {
        globalProperties: { useI18n: () => ({ t: traduire, locale: { value: 'fr' } }) },
      },
    },
  })
}

// =================================================================================================
//  Les trois contraintes
// =================================================================================================

describe('SC-001 — le budget de gestes du passage', () => {
  beforeEach(() => {
    appels.length = 0
  })

  /**
   * ★ **EXACTEMENT DEUX interactions obligatoires**, du premier geste à la confirmation.
   *
   * Le second tap **est** la confirmation : il n'y a pas de troisième écran, pas de bouton
   * « Valider », pas de boîte de dialogue. La maquette l'écrit — « Un tap pour changer. Rien
   * d'autre à faire. »
   *
   * Le test compte les gestes **réellement nécessaires** pour arriver à l'état « enregistré ».
   * Un bouton ajouté au parcours le ferait échouer, quel que soit son libellé.
   */
  it('exige exactement deux interactions du premier geste à la confirmation', async () => {
    const ecran = monter()
    let gestes = 0

    // ── Geste 1 · la durée ────────────────────────────────────────────────────────────────────
    const boutonsDuree = ecran.findAll('[data-palier]')
    expect(boutonsDuree.length).toBeGreaterThan(0)
    await boutonsDuree[1]!.trigger('click')
    gestes += 1

    // ── Geste 2 · la chambre — ET C'EST LA CONFIRMATION ───────────────────────────────────────
    const chambresLibres = ecran.findAll('[data-unite][data-etat="libre"]')
    expect(chambresLibres.length).toBeGreaterThan(0)
    await chambresLibres[0]!.trigger('click')
    gestes += 1

    await ecran.vm.$nextTick()
    await new Promise((r) => setTimeout(r, 0))
    await ecran.vm.$nextTick()

    expect(
      ecran.find('[data-etat="enregistre"]').exists(),
      'après DEUX gestes, l\'écran doit être à l\'état « enregistré ». S\'il ne l\'est pas, un '
      + 'troisième geste a été ajouté au parcours — et le budget de SC-001 est dépassé.',
    ).toBe(true)

    expect(gestes).toBe(2)
  })

  /**
   * ★ **ZÉRO champ de saisie libre obligatoire** avant la confirmation.
   *
   * « Pièce d'identité : après la clé, pas avant » — la mention est **normative** sur la maquette.
   * Un champ obligatoire ici, si petit soit-il, transforme un parcours de deux taps en une saisie,
   * et Yao reprend son cahier.
   *
   * Le contrôle porte sur le **HTML rendu**, pas sur une intention : un `<input required>` ajouté
   * par mégarde le fait échouer même si aucune validation ne l'exige côté code.
   */
  it('n\'exige aucun champ de saisie libre avant la confirmation', () => {
    const ecran = monter()

    const champs = ecran.findAll('input, textarea, select')
    const obligatoires = champs.filter((c) => {
      const el = c.element as HTMLInputElement
      return el.required || el.getAttribute('aria-required') === 'true'
    })

    expect(
      obligatoires.length,
      'un champ de saisie obligatoire est apparu sur le parcours du passage. « Pièce d\'identité : '
      + 'après la clé, pas avant » est une mention NORMATIVE de la maquette R4 : la saisie vient '
      + 'après, jamais avant.',
    ).toBe(0)
  })

  /**
   * ★ **AU PLUS UN appel réseau bloquant** entre le premier geste et la confirmation.
   *
   * Le rafraîchissement de la grille a lieu **après** la confirmation : il n'est pas bloquant, et
   * il est compté à part.
   *
   * ⚠️ **Ce n'est pas qu'une affaire de vitesse.** Un second appel serait, presque à coup sûr, une
   * vérification « cette chambre est-elle encore libre ? » — c'est-à-dire le **verrou applicatif**
   * que le principe IV refuse : entre la vérification et l'attribution, une autre réception peut
   * prendre la chambre, et la double attribution redeviendrait *improbable* au lieu
   * d'*impossible*.
   */
  it('ne fait au plus qu\'un appel réseau bloquant jusqu\'à la confirmation', async () => {
    const ecran = monter()

    await ecran.findAll('[data-palier]')[1]!.trigger('click')
    expect(
      appels,
      'le choix de la durée ne doit déclencher AUCUN appel : les paliers sont déjà chargés au '
      + 'montage de l\'écran, avant le premier geste.',
    ).toHaveLength(0)

    await ecran.findAll('[data-unite][data-etat="libre"]')[0]!.trigger('click')
    await new Promise((r) => setTimeout(r, 0))

    const bloquants = appels.filter((a) => a.startsWith('POST'))
    expect(
      bloquants,
      'exactement UN appel bloquant est autorisé (FR-031). Un second serait une vérification '
      + 'préalable — le verrou applicatif que le principe IV refuse.',
    ).toHaveLength(1)
  })

  /**
   * **Sans la permission d'ouvrir, la grille est ABSENTE du HTML** — jamais grisée (FR-026).
   *
   * Le contrôle porte sur le **HTML rendu**, pas sur un attribut `disabled` : un attribut se
   * retire depuis la console du navigateur, une absence non.
   */
  it('rend la grille ABSENTE du HTML sans la permission d\'ouvrir', () => {
    const ecran = mount(EcranPassage, {
      props: {
        contexte: { baseUrl: 'http://test', jeton: 'x' } as never,
        etablissementId: 'e1',
        donnees: DONNEES as never,
        permissions: ['heb.sejour.lire'],
        clientReconnu: null,
        passagesDuClient: 0,
      },
      global: {
        mocks: { useI18n: () => ({ t: traduire, locale: { value: 'fr' } }) },
        config: {
          globalProperties: { useI18n: () => ({ t: traduire, locale: { value: 'fr' } }) },
        },
      },
    })

    expect(ecran.findAll('[data-unite]')).toHaveLength(0)
    expect(
      ecran.html(),
      'sans permission, l\'action doit être ABSENTE du HTML rendu, pas désactivée',
    ).not.toContain('data-unite')
  })

  /**
   * **Les quatre éléments d'autres cycles sont ABSENTS**, jamais grisés (principe VII).
   *
   * Un bouton grisé promet une fonction que le produit n'a pas ; l'exploitant attend une mise à
   * jour qui ne vient pas, puis cesse de croire ce que l'écran lui dit.
   */
  it('n\'affiche aucun élément relevant d\'un autre cycle', () => {
    const html = monter().html().toLowerCase()

    for (const [element, cycle] of [
      ['scanner', 'SEJ-06 — OCR, tranche T4'],
      ['garder la', 'RSV — maintien d\'unité, tranche T4'],
      ['imprimer', 'IMP — tranche T2'],
      ['espèces', 'CAI — tranche T2'],
    ] as const) {
      expect(
        html.includes(element),
        `« ${element} » apparaît sur l'écran : il relève de ${cycle} et doit être ABSENT, `
        + 'jamais grisé (principe VII).',
      ).toBe(false)
    }
  })
})
