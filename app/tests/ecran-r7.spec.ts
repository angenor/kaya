// @vitest-environment happy-dom
/**
 * ★ **`R7` — La note et le départ.** Les cinq propriétés qu'une relecture ne verrait pas.
 *
 * | # | Ce qui est vérifié | Ce qu'une relecture manquerait |
 * |---|---|---|
 * | **1** | Les trois sections absentes sont **nommées**, pas omises | Une note qui s'arrête à l'hébergement se lit « rien consommé » — et le total est encaissé de bonne foi |
 * | **2** | La mention « Document non fiscal » est sur la note **ET** sur la fiche de police | Le principe V l'exige de **tous** les documents opérationnels ; l'oubli ne se voit qu'au contrôle |
 * | **3** | Les trois éléments de CAI/FIS sont **absents du HTML rendu**, jamais grisés | Un grisé promet une fonction que le produit n'a pas |
 * | **4** | Après le départ, l'écran dit que la note est **arrêtée et NON RÉGLÉE** | Sans la phrase, le trou se découvre au comptage de caisse, sans qu'on sache à quel séjour il se rattache |
 * | **5** | Hors ligne, l'action est **refusée avant le geste** | Une file « au cas où » figerait deux constats de taxe sur les mêmes faits |
 *
 * # Le cas 1 est le plus important, et c'est le moins évident
 *
 * Une note incomplète n'est pas une fonctionnalité manquante : c'est un **chiffre faux**. La
 * première se répare au cycle suivant, le second se paie devant le client. Le test assert donc la
 * **présence** de chaque section absente, ce qui est contre-intuitif à lire et exact à tenir.
 */

import { mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'

import EcranDepart from '../modules/sejours/EcranDepart.vue'
import fr from '../core/i18n/fr.json'

// =================================================================================================
//  Doublures
// =================================================================================================

const etatReseau = { value: 'connecte' as 'connecte' | 'hors_ligne' }

/**
 * ⚠️ **Double PARTIEL, jamais total.** `~/core/platform` réexporte tout l'adaptateur — dont
 * `stockagePersistantMoteur`, que la file hors-ligne importe. Un double total le ferait disparaître
 * du graphe et le fichier entier échouerait au chargement, avant la moindre assertion.
 */
vi.mock('~/core/platform', async (importOriginal) => ({
  ...(await importOriginal<typeof import('~/core/platform')>()),
  useEtatReseau: () => etatReseau,
}))

const departs: string[] = []

vi.mock('../modules/sejours/clore-sejour', () => ({
  TYPE_OPERATION_DEPART: 'hebergement_sejour.depart',
  cloreSejour: vi.fn(async (_c, reseau: string, _e: string, sejourId: string) => {
    if (reseau !== 'connecte') {
      return { issue: 'refus' as const, cle: 'sejours.depart.refus.reseau', reseau: true }
    }
    departs.push(sejourId)
    return {
      issue: 'succes' as const,
      sejour: { ...SEJOUR_COMPLET, note: { ...NOTE, statut: 'arretee', arretee_le: INSTANT } },
    }
  }),
}))

vi.mock('../modules/sejours/donnees', async (importer) => {
  const reel = await importer<typeof import('../modules/sejours/donnees')>()
  return {
    ...reel,
    chargerSejour: vi.fn(async () => SEJOUR_COMPLET),
    chargerSejours: vi.fn(async () => [SEJOUR_LISTE]),
  }
})

const INSTANT = '2026-08-03T11:12:00Z'

const NOTE = {
  id: 'n1',
  sejour_id: 's1',
  statut: 'ouverte',
  devise: 'XOF',
  total_mineur: 180_000,
  lignes: [
    {
      id: 'l1',
      libelle_cle: 'hebergement.note.ligne.hebergement',
      nature: 'hebergement',
      quantite: '4',
      prix_unitaire_mineur: 45_000,
      montant_mineur: 180_000,
      devise: 'XOF',
      periode_debut: '2026-07-24T14:00:00Z',
      periode_fin: '2026-07-28T12:00:00Z',
    },
  ],
}

const SEJOUR_COMPLET = {
  sejour: { id: 's1', statut: 'en_cours', etablissement_id: 'e1', ouvert_le: '2026-07-24T14:00:00Z' },
  occupation: { id: 'o1', unite_id: 'u1', fin_client: '2026-07-28T12:00:00Z' },
  note: NOTE,
  fiche_police: { id: 'fp1', numero: 12, complete: true, sejour_id: 's1', generee_le: INSTANT },
  instant_autorite: INSTANT,
}

const SEJOUR_LISTE = {
  sejour: { id: 's1', statut: 'en_cours', etablissement_id: 'e1', ouvert_le: '2026-07-24T14:00:00Z' },
  client_nom: 'Adama Traoré',
  client_telephone: '+2250707123456',
  nombre_personnes: 2,
  unite_id: 'u1',
  fin_prevue: '2026-07-28T12:00:00Z',
  total_mineur: 180_000,
  devise: 'XOF',
}

const DONNEES = { sejours: [SEJOUR_LISTE], codesUnites: { u1: '204' } }

const PERMISSIONS = ['heb.sejour.lire', 'heb.sejour.clore']

function traduire(cle: string, valeurs?: Record<string, unknown>): string {
  const brut = cle
    .split('.')
    .reduce<unknown>((noeud, part) => (noeud as Record<string, unknown>)?.[part], fr)
  if (typeof brut !== 'string') return cle
  return brut.replace(/\{(\w+)\}/g, (_, nom: string) => String(valeurs?.[nom] ?? ''))
}

// `useI18n` est un auto-import Nuxt : hors du pipeline, il faut l'exposer globalement. La
// traduction lit le **catalogue réel** — un faux qui renverrait la clé ferait passer le test alors
// que le libellé serait absent du catalogue.
;(globalThis as Record<string, unknown>).useI18n = () => ({ t: traduire, locale: { value: 'fr' } })

function monter(options: { permissions?: string[], sejourInitial?: string | null } = {}) {
  return mount(EcranDepart, {
    props: {
      contexte: { baseUrl: 'http://test', jeton: 'x' } as never,
      etablissementId: 'e1',
      donnees: DONNEES as never,
      permissions: options.permissions ?? PERMISSIONS,
      sejourInitial: options.sejourInitial === undefined ? 's1' : options.sejourInitial,
    },
    global: {
      mocks: { useI18n: () => ({ t: traduire, locale: { value: 'fr' } }) },
      config: {
        globalProperties: { useI18n: () => ({ t: traduire, locale: { value: 'fr' } }) },
      },
    },
  })
}

/** Attend la résolution des promesses de montage — le détail arrive après deux ticks. */
async function stabiliser(ecran: ReturnType<typeof monter>): Promise<void> {
  await Promise.resolve()
  await Promise.resolve()
  await ecran.vm.$nextTick()
}

// =================================================================================================
//  Les cinq propriétés
// =================================================================================================

describe('R7 — la note et le départ', () => {
  beforeEach(() => {
    departs.length = 0
    etatReseau.value = 'connecte'
  })

  /**
   * ★ **Les sections absentes sont NOMMÉES.**
   *
   * L'assertion porte sur la **présence** d'un marqueur par section manquante — ce qui se lit à
   * l'envers et se tient à l'endroit : la note doit dire ce qu'elle ne porte pas encore.
   */
  it('nomme les sections que ce cycle ne sert pas — restaurant, bar, autres frais, taxes', async () => {
    const ecran = monter()
    await stabiliser(ecran)

    for (const section of ['restaurant', 'bar', 'autres_frais', 'taxes']) {
      expect(
        ecran.find(`[data-section-absente="${section}"]`).exists(),
        `la section « ${section} » est ABSENTE sans être nommée : la note se lira « ce client n'a `
        + 'rien consommé », et le total sera encaissé de bonne foi',
      ).toBe(true)
    }

    // La section servie, elle, porte ses lignes et son sous-total.
    expect(ecran.findAll('[data-ligne]')).toHaveLength(1)
    expect(ecran.find('[data-sous-total]').text()).toContain('180')
  })

  /**
   * ★ **La mention obligatoire est sur les DEUX documents.**
   *
   * FIS-02 et le principe V l'exigent de **tous** les documents opérationnels — FR-048. L'oublier
   * sur la fiche de police est l'erreur naturelle : elle ne porte aucun montant, donc elle n'a pas
   * « l'air » d'une facture. Elle en est pourtant une aux yeux d'un client qui la présente.
   */
  it('porte « Document non fiscal » sur la note ET sur la fiche de police', async () => {
    const ecran = monter()
    await stabiliser(ecran)

    const mention = fr.documents.mention_non_fiscale
    expect(ecran.find('[data-mention-non-fiscale]').text()).toContain(mention)
    expect(ecran.find('[data-fiche-police]').text()).toContain(mention)
  })

  /**
   * ★ **Les trois éléments de CAI et FIS sont ABSENTS du HTML rendu**, jamais grisés.
   *
   * Le contrôle porte sur le texte rendu, et il attrape aussi bien un bouton grisé qu'un
   * commentaire de `<template>` — qui est **rendu dans le DOM** et atteindrait la page.
   */
  it('ne montre ni versement, ni reste à payer, ni promesse d\'encaissement', async () => {
    const ecran = monter()
    await stabiliser(ecran)
    const rendu = ecran.html()

    for (const absent of ['Déjà versé', 'Wave', 'resterait à payer', 'encaissement, facture']) {
      expect(
        rendu,
        `« ${absent} » figure dans le rendu. Cet élément relève de CAI ou de FIS et doit être `
        + 'ABSENT, jamais grisé : un grisé promet une fonction que le produit n\'a pas',
      ).not.toContain(absent)
    }
  })

  /**
   * ★ **Après le départ, l'écran dit que la note est arrêtée ET NON RÉGLÉE.**
   *
   * C'est la contrepartie du retrait de « encaissement, facture » : sans cette phrase, l'écran
   * laisserait croire au paiement. Le trou se découvrirait au comptage de caisse, le soir, sans
   * qu'on sache à quel séjour il se rattache.
   */
  it('après le départ, dit en toutes lettres que la note n\'est pas réglée', async () => {
    const ecran = monter()
    await stabiliser(ecran)

    await ecran.find('[data-action="faire-partir"]').trigger('click')
    await stabiliser(ecran)

    expect(departs).toEqual(['s1'])
    expect(ecran.find('[data-etat="clos"]').exists()).toBe(true)
    expect(ecran.find('[data-non-reglee]').text()).toContain('n\'est pas réglée')
    // Le total change d'étiquette : « provisoire » n'a plus de sens sur une note arrêtée.
    expect(ecran.text()).toContain(fr.sejours.note.total_arrete)
  })

  /**
   * ★ **Hors ligne, le refus précède le geste** — classe B, principe VI.
   *
   * L'action n'est pas grisée : elle est **remplacée** par la phrase qui dit pourquoi. Un départ
   * mis en file locale figerait un second constat de taxe sur des faits périmés, et les deux
   * vaudraient également.
   */
  it('hors ligne, refuse le départ AVANT le geste — sans grisé', async () => {
    etatReseau.value = 'hors_ligne'
    const ecran = monter()
    await stabiliser(ecran)

    expect(ecran.find('[data-hors-ligne]').exists()).toBe(true)
    expect(ecran.find('[data-action="faire-partir"]').exists()).toBe(false)
    expect(ecran.findAll('[disabled]')).toHaveLength(0)
    expect(departs).toEqual([])
  })

  /**
   * Sans `heb.sejour.clore`, l'action est **absente du HTML rendu**.
   *
   * Le contrôle porte sur le nœud, jamais sur un attribut : un attribut se retire depuis la
   * console du navigateur.
   */
  it('sans `heb.sejour.clore`, l\'action de départ est absente — jamais grisée', async () => {
    const ecran = monter({ permissions: ['heb.sejour.lire'] })
    await stabiliser(ecran)

    expect(ecran.find('[data-action="faire-partir"]').exists()).toBe(false)
    // La note reste lisible : lire n'est pas clore, et un rôle de lecture doit pouvoir consulter.
    expect(ecran.find('[data-total]').exists()).toBe(true)
  })

  /**
   * La route n'a **aucun segment dynamique** : sans séjour choisi, l'écran s'ouvre quand même.
   *
   * C'est ce qui rend `/depart` couvrable par P-22 sans faire dépendre une porte de parcours de
   * l'état des données.
   */
  it('s\'ouvre sans séjour choisi, et invite à en choisir un', async () => {
    const ecran = monter({ sejourInitial: null })
    await stabiliser(ecran)

    expect(ecran.text()).toContain(fr.sejours.depart.choisir_un_sejour)
    expect(ecran.findAll('[data-sejour]')).toHaveLength(1)
  })
})
