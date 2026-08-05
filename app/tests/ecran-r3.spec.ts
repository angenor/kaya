// @vitest-environment happy-dom
/**
 * ★ **`R3` — Arrivée.** Les quatre propriétés que cet écran doit tenir, et qu'une relecture rate.
 *
 * ═══════════════════════════════════════════════════════════════════════════════════════════════
 *  ÉCRIT DANS LA MÊME TÂCHE QUE L'ÉCRAN, pour la raison de `budget-gestes.spec.ts`
 *
 *  Écrit après, il **constaterait** ce que l'écran fait ; écrit avec, il le **contraint**.
 * ═══════════════════════════════════════════════════════════════════════════════════════════════
 *
 * | # | Ce qui est vérifié | Ce qu'une relecture manquerait |
 * |---|---|---|
 * | **1** | Fiche retenue → **aucun champ d'identité** dans la requête, et le corps ne porte que `client_id` | Un écran qui pré-remplit **et renvoie** la copie : chaque arrivée écraserait la fiche par une version périmée |
 * | **2** | Les heures standard viennent de la **formule**, jamais d'une constante | « 14 h / 12 h » en dur passerait toute revue et ferait mentir l'écran au premier établissement qui pratique autrement |
 * | **3** | Le paramètre absent est **dit**, pas remplacé par un défaut inventé | Un défaut silencieux fait croire à l'exploitant qu'il a réglé ce qu'il n'a pas réglé |
 * | **4** | Sans `heb.sejour.ouvrir`, la grille est **absente du HTML rendu** | Un `disabled` se retire depuis la console du navigateur |
 *
 * # Le geste final reste le TAP, et c'est la grammaire de `R4`
 *
 * L'écran est plus long — un client, des nuits, des heures, des accompagnants — mais il **ne
 * devient pas un formulaire** : il n'y a pas de bouton « Enregistrer ». Le test le vérifie en
 * cherchant l'absence de tout `type="submit"` et en ouvrant le séjour **par le tap sur la
 * chambre**.
 */

import { mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'

import EcranArrivee from '../modules/sejours/EcranArrivee.vue'
import fr from '../core/i18n/fr.json'

// =================================================================================================
//  Doublures
// =================================================================================================

/**
 * ⚠️ **Le double porte sur `~/core/platform/reseau`, PAS sur le baril** — `useEtatReseau` vit là,
 * et le baril ne le réexporte pas. Doubler le baril fournirait un export inexistant et rendrait
 * vrai en test ce qui échoue en navigateur (défaut du cycle 006, voir `imports-barils.spec.ts`).
 */
vi.mock('~/core/platform/reseau', () => ({
  useEtatReseau: () => ({ value: 'connecte' as const }),
}))

/** Les demandes d'ouverture, **capturées telles qu'elles partent**. */
const demandes: Record<string, unknown>[] = []

vi.mock('../modules/sejours/ouvrir-sejour', () => ({
  TYPE_OPERATION_SEJOUR: 'hebergement_sejour.ouverture',
  ouvrirSejour: vi.fn(async (_contexte, _reseau, demande: Record<string, unknown>) => {
    demandes.push(demande)
    return {
      issue: 'succes' as const,
      sejour: {
        sejour: { id: 's1', statut: 'en_cours' },
        occupation: { id: 'o1', fin_client: '2026-08-04T12:00:00Z' },
        note: { total_mineur: 12500, devise: 'XOF', lignes: [] },
        fiche_police: { numero: 7, complete: true },
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
    rechargerEtatDesUnites: vi.fn(async () => ETAT_DES_UNITES),
    chercherClients: vi.fn(async () => ({ clients: [CLIENT], tronque: false })),
  }
})

const INSTANT = '2026-08-03T15:30:00Z'

const ETAT_DES_UNITES = {
  instant_autorite: INSTANT,
  unites: [
    { unite_id: 'u1', code: 'B3', categorie_id: 'c1', etage: 1, etat: 'libre', fin_prevue: null, disponible_a: null, statut_menage: 'propre', sejour_id: null },
    { unite_id: 'u2', code: 'B4', categorie_id: 'c1', etage: 1, etat: 'occupee', fin_prevue: '2026-08-04T12:00:00Z', disponible_a: null, statut_menage: 'propre', sejour_id: 's0' },
    // ⚠️ Une chambre d'une AUTRE catégorie : l'écran ne doit pas la proposer, sa formule n'est pas
    // celle qu'il applique. Sans elle, « on affiche tout » et « on filtre par catégorie » seraient
    // indistinguables.
    { unite_id: 'u9', code: 'Z9', categorie_id: 'c2', etage: 2, etat: 'libre', fin_prevue: null, disponible_a: null, statut_menage: 'propre', sejour_id: null },
  ],
}

/** La formule de nuitée — **elle porte les heures standard**, et c'est tout le point du cas 2. */
const FORMULE_NUITEE = {
  id: 'f-nuitee',
  categorie_id: 'c1',
  famille: 'NUITEE',
  devise: 'XOF',
  prix_mineur: 12500,
  assujettie_taxe_nuitee: true,
  heure_arrivee_standard: '14:00:00',
  heure_depart_standard: '11:30:00',
  paliers: [],
  plages: [],
}

const CLIENT = {
  id: 'cl-1',
  nom: 'Bakayoko',
  telephone: '+2250707123456',
  piece_enregistree: true,
}

const DONNEES = {
  etatDesUnites: ETAT_DES_UNITES,
  categories: [
    { id: 'c1', nom: 'Standard', capacite_accueil: 2, etablissement_id: 'e1', temps_remise_en_etat: [] },
    { id: 'c2', nom: 'Suite', capacite_accueil: 4, etablissement_id: 'e1', temps_remise_en_etat: [] },
  ],
  formules: [FORMULE_NUITEE],
}

const PERMISSIONS = ['heb.sejour.ouvrir', 'heb.sejour.lire', 'sej.client.lire']

function traduire(cle: string, valeurs?: Record<string, unknown>): string {
  const brut = cle
    .split('.')
    .reduce<unknown>((noeud, part) => (noeud as Record<string, unknown>)?.[part], fr)
  if (typeof brut !== 'string') return cle
  return brut.replace(/\{(\w+)\}/g, (_, nom: string) => String(valeurs?.[nom] ?? ''))
}

// `useI18n` est un auto-import Nuxt : hors du pipeline, il faut l'exposer globalement — sinon le
// `setup` des composants enfants lève, et Vue rend un `vnode` indéfini dont le message
// (« Invalid vnode type ») ne désigne pas la cause. La traduction lit le **catalogue réel** : un
// faux qui renverrait la clé ferait passer le test alors que le libellé serait absent.
;(globalThis as Record<string, unknown>).useI18n = () => ({ t: traduire, locale: { value: 'fr' } })

function monter(options: {
  clientRetenu?: typeof CLIENT | null
  permissions?: string[]
  formules?: unknown[]
} = {}) {
  return mount(EcranArrivee, {
    props: {
      contexte: { baseUrl: 'http://test', jeton: 'x' } as never,
      etablissementId: 'e1',
      donnees: {
        ...DONNEES,
        ...(options.formules ? { formules: options.formules } : {}),
      } as never,
      permissions: options.permissions ?? PERMISSIONS,
      clientRetenu: (options.clientRetenu ?? null) as never,
    },
    global: {
      mocks: { useI18n: () => ({ t: traduire, locale: { value: 'fr' } }) },
      config: {
        globalProperties: { useI18n: () => ({ t: traduire, locale: { value: 'fr' } }) },
      },
    },
  })
}

// =================================================================================================
//  ★ 1 · CLIENT CONNU — la requête ne porte AUCUN champ d'identité (FR-035)
// =================================================================================================

describe('R3 — l\'arrivée', () => {
  beforeEach(() => {
    demandes.length = 0
  })

  /**
   * ★ **La fiche retenue ne produit qu'un `clientId`.**
   *
   * L'assertion porte sur la **demande réellement émise**, pas sur ce que l'écran affiche. Un
   * écran qui pré-remplit et renvoie la copie paraîtrait identique en revue : il montrerait le bon
   * nom, il enverrait juste, en plus, une version périmée de la fiche — qui écraserait celle du
   * serveur à chaque arrivée. Le défaut ne se verrait qu'après un changement de numéro de
   * téléphone, revenu tout seul à l'ancienne valeur.
   */
  it('avec une fiche retenue, la demande ne porte que `clientId` — aucun champ d\'identité', async () => {
    const ecran = monter({ clientRetenu: CLIENT })

    // Le nom et le téléphone sont **affichés**, en lecture.
    expect(ecran.html()).toContain('Bakayoko')
    expect(ecran.html()).toContain('+2250707123456')

    // Aucun champ de saisie ne porte l'identité : elle n'est pas retapable, donc pas retapée.
    const valeurs = ecran.findAll('input').map((i) => (i.element as HTMLInputElement).value)
    expect(valeurs).not.toContain('Bakayoko')
    expect(valeurs).not.toContain('+2250707123456')

    // ── Le tap sur la chambre — et c'est le geste final ────────────────────────────────────────
    const libres = ecran.findAll('[data-unite][data-etat="libre"]')
    expect(libres.length).toBe(1)
    await libres[0]!.trigger('click')
    await ecran.vm.$nextTick()

    expect(demandes).toHaveLength(1)
    const demande = demandes[0]!
    expect(demande.clientId).toBe('cl-1')
    for (const interdit of ['nom', 'prenoms', 'telephone', 'email', 'typePiece', 'numeroPiece']) {
      expect(
        Object.keys(demande),
        `la demande porte « ${interdit} » : c'est une COPIE de la fiche, et elle l'écrasera à `
        + 'la prochaine arrivée',
      ).not.toContain(interdit)
    }
  })

  /**
   * ★ **Le geste final est le TAP, jamais un bouton « Enregistrer ».**
   *
   * C'est la grammaire de `R4`, conservée : *« plus de champs, même grammaire »*
   * (`docs/design/derivation.md`). Un bouton de soumission en bas de page ferait de cet écran un
   * formulaire — et un formulaire de comptoir se remplit dans l'ordre, alors qu'une arrivée se
   * fait dans le désordre, en parlant au client.
   */
  it('n\'a aucun bouton de soumission : le tap sur la chambre EST l\'ouverture', () => {
    const ecran = monter({ clientRetenu: CLIENT })
    expect(ecran.findAll('[type="submit"]')).toHaveLength(0)
    expect(ecran.findAll('form')).toHaveLength(0)
  })

  // ===============================================================================================
  //  ★ 2 · LES HEURES VIENNENT DU PARAMÈTRE (porte P-12)
  // ===============================================================================================

  /**
   * ★ **`14:00` et `11:30` sont lus de la formule, et rien d'autre ne les produit.**
   *
   * La valeur `11:30` est choisie **exprès** : un écran qui aurait « 12 h » en dur — la valeur
   * qu'on écrit d'instinct — passerait un test posé sur `12:00`. Celui-ci échouerait.
   */
  it('applique d\'office les heures standard DE LA FORMULE, et les rend modifiables', async () => {
    const ecran = monter({ clientRetenu: CLIENT })

    const champs = ecran.findAll('input')
    const valeurs = champs.map((c) => (c.element as HTMLInputElement).value)
    expect(valeurs).toContain('14:00')
    expect(valeurs).toContain('11:30')

    // ── MODIFIABLES : un client qui arrive à 22 h ne doit pas faire sortir Yao de l'écran ──────
    const champArrivee = champs.find((c) => (c.element as HTMLInputElement).value === '14:00')!
    await champArrivee.setValue('22:00')
    await ecran.vm.$nextTick()

    const libres = ecran.findAll('[data-unite][data-etat="libre"]')
    await libres[0]!.trigger('click')
    await ecran.vm.$nextTick()

    const debut = demandes[0]!.debutClient as Date
    expect(debut.getHours()).toBe(22)
  })

  /**
   * ★ **Le paramètre absent est DIT, il n'est pas remplacé par un défaut inventé.**
   *
   * Poser « 14 h / 12 h » quand l'établissement n'a rien réglé serait une règle métier en dur
   * déguisée en commodité : l'exploitant croirait avoir réglé ce qu'il n'a pas réglé, et les
   * durées facturées seraient fausses **sans que rien ne le signale**.
   */
  it('dit que les heures ne sont pas réglées plutôt que d\'en inventer', () => {
    const sansHeures = { ...FORMULE_NUITEE, heure_arrivee_standard: null, heure_depart_standard: null }
    const ecran = monter({ clientRetenu: CLIENT, formules: [sansHeures] })

    expect(ecran.find('[data-alerte="heures-non-reglees"]').exists()).toBe(true)
    const valeurs = ecran.findAll('input').map((i) => (i.element as HTMLInputElement).value)
    expect(valeurs).not.toContain('14:00')
    expect(valeurs).not.toContain('12:00')
  })

  // ===============================================================================================
  //  ★ 3 · LA CHAMBRE PROPOSÉE EST DE LA BONNE CATÉGORIE
  // ===============================================================================================

  /**
   * L'écran applique **une** formule ; proposer une chambre d'une autre catégorie produirait un
   * `formule_hors_categorie` — un refus que Yao subirait après le geste, devant le client.
   */
  it('ne propose que les chambres de la catégorie servie par la formule', () => {
    const ecran = monter({ clientRetenu: CLIENT })
    expect(ecran.find('[data-unite="u1"]').exists()).toBe(true)
    expect(ecran.find('[data-unite="u9"]').exists()).toBe(false)
  })

  // ===============================================================================================
  //  ★ 4 · SANS PERMISSION, LA GRILLE EST ABSENTE DU HTML RENDU
  // ===============================================================================================

  /**
   * ★ **Le contrôle porte sur le HTML rendu, jamais sur un attribut `disabled`.**
   *
   * Un attribut se retire depuis la console du navigateur ; un nœud absent n'existe pas. Et
   * l'action **absente** vaut mieux que grisée (principe VII) : un bouton grisé promet une
   * fonction que le compte n'a pas, et l'utilisateur attend un droit qui ne viendra pas.
   */
  it('sans `heb.sejour.ouvrir`, la grille des chambres est absente — jamais grisée', () => {
    const ecran = monter({ clientRetenu: CLIENT, permissions: ['heb.sejour.lire'] })
    expect(ecran.findAll('[data-unite]')).toHaveLength(0)
    // Aucun nœud désactivé non plus : l'action est **absente**, pas grisée. Assertion posée sur le
    // sélecteur d'attribut et non sur le texte du HTML — un commentaire de `<template>` est rendu
    // dans le DOM, et un commentaire qui NOMME l'attribut ferait échouer le test pour une raison
    // sans rapport avec ce qu'il mesure. C'est ce test qui l'a trouvé, une seconde fois.
    expect(ecran.findAll('[disabled]')).toHaveLength(0)
  })

  // ===============================================================================================
  //  Les accompagnants — un nom suffit, et ils partent avec l'ouverture
  // ===============================================================================================

  /**
   * ★ **Les accompagnants voyagent DANS la demande d'ouverture**, jamais par un second appel.
   *
   * Un accompagnant déclaré à l'arrivée et perdu par un appel manqué ferait une **fiche de police
   * qui sous-déclare** — un document légal faux, pour une coupure réseau.
   */
  it('joint les accompagnants à la demande d\'ouverture, avec le nom SEUL', async () => {
    const ecran = monter({ clientRetenu: CLIENT })

    const champNom = ecran.findAll('input').find(
      (i) => (i.element as HTMLInputElement).value === '',
    )!
    await champNom.setValue('Aya')
    await ecran.find('[data-action="ajouter-accompagnant"]').trigger('click')
    await ecran.vm.$nextTick()

    expect(ecran.findAll('[data-accompagnant]')).toHaveLength(1)

    await ecran.find('[data-unite][data-etat="libre"]').trigger('click')
    await ecran.vm.$nextTick()

    expect(demandes[0]!.accompagnants).toEqual([{ nom: 'Aya' }])
  })
})
