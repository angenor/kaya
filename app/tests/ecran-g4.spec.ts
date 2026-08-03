// @vitest-environment happy-dom
/**
 * **`G4` — ce que le registre des actions MONTRE.**
 *
 * # Les trois propriétés qui font ou défont ce que le propriétaire achète
 *
 * 1. **L'horodatage affiché est celui d'AUTORITÉ**, jamais celui du terminal. Un téléphone en
 *    avance de deux heures ferait mentir le registre qui sert à prouver ce qui s'est passé
 *    (principe IV). Le test fournit un `horodatage_client` **délibérément différent** de deux
 *    heures, et vérifie que c'est `cree_le` qui s'affiche.
 * 2. **Les quatre filtres sont cumulés par le SERVEUR.** Filtrer côté client sur une page déjà
 *    paginée donnerait une liste amputée qui aurait l'air complète — le pire résultat possible
 *    pour un registre. Le test lit la requête réellement émise.
 * 3. **Aucun nom technique n'atteint l'écran.** `changement_role` devient une phrase ; un UUID
 *    d'auteur illisible devient « Compte introuvable », jamais l'UUID.
 *
 * # Registre SOBRE — la propriété qu'on défait au premier réflexe
 *
 * `F2` établit qu'un registre grave se lit à plat : pas de couleur par famille d'action, pas
 * d'icône par ligne. Une remise en rouge et une ouverture de tiroir en orange transformeraient une
 * liste de faits en tableau d'accusations. Le test le vérifie sur les classes rendues.
 */

import { flushPromises, mount } from '@vue/test-utils'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

import EcranJournalAudit from '../modules/audit/EcranJournalAudit.vue'
import { TYPES_ACTION, chargerJournal, cleTypeAction } from '../modules/audit/journal'
import fr from '../core/i18n/fr.json'
import type { PageJournal, TypeAction } from '../modules/audit/journal'

const CONTEXTE = { baseUrl: 'http://localhost:8080', acces: 'jeton-de-test' }
const ETABLISSEMENT = '018f0000-0000-7000-8000-000000000001'

/** L'horodatage d'autorité, et celui du terminal — **délibérément décalés de deux heures**. */
const AUTORITE = '2026-08-01T14:00:00Z'
const TERMINAL = '2026-08-01T16:00:00Z'

const PAGE: PageJournal = {
  elements: [
    {
      id: '018f0000-0000-7000-8000-0000000000e1',
      etablissement_id: ETABLISSEMENT,
      type_action: 'changement_role',
      auteur: { compte_id: '018f0000-0000-7000-8000-0000000000a1', nom: 'Koffi' },
      cible_type: 'compte',
      cible_id: '018f0000-0000-7000-8000-0000000000a2',
      contexte: { role_code: 'caissier', sens: 'attribution' },
      horodatage_client: TERMINAL,
      cree_le: AUTORITE,
    },
    {
      id: '018f0000-0000-7000-8000-0000000000e2',
      etablissement_id: ETABLISSEMENT,
      type_action: 'suppression',
      // Auteur illisible — compte d'un autre tenant, ou compte que l'annuaire ne rend pas.
      auteur: { compte_id: '018f0000-0000-7000-8000-0000000000a9', nom: null },
      cible_type: 'session',
      cible_id: null,
      contexte: {},
      horodatage_client: null,
      cree_le: '2026-08-01T13:00:00Z',
    },
  ],
  suivant_cree_le: '2026-08-01T13:00:00Z',
  suivant_id: '018f0000-0000-7000-8000-0000000000e2',
}

const fetchOriginal = globalThis.fetch

function traduire(cle: string, valeurs?: Record<string, unknown>): string {
  const brut = cle
    .split('.')
    .reduce<unknown>((noeud, part) => (noeud as Record<string, unknown>)?.[part], fr)
  if (typeof brut !== 'string') return cle
  return brut.replace(/\{(\w+)\}/g, (_, nom: string) => String(valeurs?.[nom] ?? ''))
}

function fauxServeur(corps: unknown): string[] {
  const urls: string[] = []
  globalThis.fetch = (async (entree: string | URL | Request) => {
    urls.push(entree instanceof Request ? entree.url : String(entree))
    return new Response(JSON.stringify(corps), {
      status: 200,
      headers: { 'content-type': 'application/json' },
    })
  }) as typeof fetch
  return urls
}

function monter(page: PageJournal = PAGE) {
  return mount(EcranJournalAudit, {
    props: { page, contexte: CONTEXTE, etablissementId: ETABLISSEMENT },
    global: {
      mocks: { useI18n: () => ({ t: traduire }) },
      config: { globalProperties: { useI18n: () => ({ t: traduire }) } },
    },
  })
}

beforeEach(() => {
  ;(globalThis as Record<string, unknown>).useI18n = () => ({ t: traduire })
})

afterEach(() => {
  globalThis.fetch = fetchOriginal
  vi.restoreAllMocks()
})

describe('principe IV — l’horodatage affiché est celui d’AUTORITÉ', () => {
  it('l’heure du serveur s’affiche, celle du terminal jamais', () => {
    const ecran = monter()
    const texte = ecran.text()

    // 14 h UTC = 16 h à Paris ; le terminal, lui, dit 16 h UTC = 18 h. Les deux formats locaux
    // diffèrent, et c'est ce qui rend l'assertion opposable sans dépendre du fuseau de la CI.
    const attendu = new Intl.DateTimeFormat('fr-FR', { dateStyle: 'short', timeStyle: 'short' })
      .format(new Date(AUTORITE))
    const interdit = new Intl.DateTimeFormat('fr-FR', { dateStyle: 'short', timeStyle: 'short' })
      .format(new Date(TERMINAL))

    expect(attendu).not.toBe(interdit)
    expect(texte).toContain(attendu)
    expect(texte).not.toContain(interdit)
  })

  it('le champ `horodatage_client` n’apparaît nulle part dans le HTML', () => {
    // Ni affiché, ni glissé dans un attribut « pour plus tard » — ce qui reviendrait à le publier.
    expect(monter().html()).not.toContain(TERMINAL)
  })
})

describe('les noms techniques n’atteignent jamais l’écran', () => {
  it('le type d’action est traduit, jamais rendu en brut', () => {
    const ecran = monter()
    const texte = ecran.text()

    expect(texte).toContain(fr.journal.types.changement_role)
    expect(texte).toContain(fr.journal.types.suppression)
    expect(texte).not.toContain('changement_role')
    expect(texte).not.toContain('suppression')
  })

  it('un auteur illisible devient une phrase, jamais son UUID', () => {
    const ecran = monter()
    const texte = ecran.text()

    expect(texte).toContain('Koffi')
    expect(texte).toContain(fr.journal.auteur_inconnu)
    expect(texte).not.toContain('018f0000-0000-7000-8000-0000000000a9')
  })

  it('aucun identifiant d’entrée ni de cible dans le texte rendu', () => {
    const texte = monter().text()

    for (const entree of PAGE.elements) {
      expect(texte).not.toContain(entree.id)
    }
  })

  it('les dix familles de la taxonomie ont toutes une clé i18n', () => {
    // Une famille sans libellé s'afficherait en brut le jour où PDV-03 branchera la remise — et
    // l'écran est déjà en production à ce moment-là.
    for (const type of TYPES_ACTION) {
      const cle = cleTypeAction(type)
      expect(cle).toBe(`journal.types.${type}`)
      expect(traduire(cle), `« ${type} » n'a pas de libellé`).not.toBe(cle)
    }
    // Et un code inconnu tombe sur une phrase, jamais sur la clé.
    expect(cleTypeAction('remise_appliquee')).toBe('journal.types.inconnu')
  })

  it('TYPES_ACTION couvre exactement la taxonomie du CONTRAT', () => {
    // `TYPES_ACTION` porte déjà `satisfies readonly TypeAction[]`, qui attrape une famille
    // **renommée ou retirée** du contrat. Il ne peut pas attraper une famille **ajoutée** : une
    // liste incomplète satisfait toujours son type.
    //
    // Le manque n'est pas théorique. Le test au-dessus parcourt `TYPES_ACTION` pour vérifier que
    // chaque famille a un libellé : une onzième famille absente de la liste passerait les deux
    // tests, et s'afficherait en brut à l'écran le jour où quelqu'un la branche.
    //
    // ⚠️ **Ce contrôle ne contrôlait rien, et le cycle 005 l'a constaté en ajoutant une onzième
    // famille au contrat.** Sa forme précédente était :
    //
    //     const couvertureComplete: FamillesDuContratNonListees[] = []
    //
    // Un tableau **vide** est assignable à `('derive_horloge_constatee')[]` comme à `never[]` :
    // l'affectation compilait dans les deux cas, et le test annonçait une garantie qu'il n'avait
    // pas. C'est le mode de défaillance que ce dépôt documente depuis le cycle 001 — un vert qui
    // donne l'assurance empêchant la relecture.
    //
    // La forme qui tient compare le type à `never` **sans distribution** : `[T] extends [never]`.
    // L'écrire `T extends never ? …` ne marcherait pas non plus — un type conditionnel sur
    // `never` est distributif et rend `never`, donc ni `true` ni `false`.
    type FamillesDuContratNonListees = Exclude<TypeAction, (typeof TYPES_ACTION)[number]>
    type CouvertureComplete = [FamillesDuContratNonListees] extends [never] ? true : false

    // Une famille du contrat absente de `TYPES_ACTION` rend `CouvertureComplete` égal à `false`,
    // et `const … : false = true` cesse de compiler. Contrôle de TYPE : il échoue au `tsc` de
    // `vitest --typecheck`, pas à l'exécution.
    const couvertureComplete: CouvertureComplete = true

    expect(couvertureComplete).toBe(true)
    // Onze au cycle 005, **douze depuis le cycle 006** : `consultation_piece_identite`
    // (SEJ-01) est la première famille qui trace une LECTURE et non une modification.
    expect(TYPES_ACTION).toHaveLength(12)
  })
})

describe('registre SOBRE — F2, et le réflexe qui le défait', () => {
  it('aucune couleur d’accentuation par famille d’action', () => {
    const html = monter().html()

    // Un registre grave se lit à plat. Une remise en rouge et une ouverture de tiroir en orange
    // transformeraient une liste de faits en tableau d'accusations.
    for (const ton of ['bg-danger-soft', 'bg-alerte-soft', 'bg-succes-soft', 'text-danger-fort']) {
      expect(html, `le ton « ${ton} » colore une ligne du registre`).not.toContain(ton)
    }
  })

  it('mais un échec de LECTURE, lui, se signale', async () => {
    // Versant positif : l'assertion précédente serait vraie sur un écran incapable de signaler
    // quoi que ce soit.
    globalThis.fetch = (async () => {
      throw new Error('réseau coupé')
    }) as typeof fetch

    const ecran = monter()
    await ecran.find('button').trigger('click')
    await flushPromises()

    const alerte = ecran.find('[role="alert"]')
    expect(alerte.exists()).toBe(true)
    expect(alerte.text()).toContain(fr.journal.chargement_impossible)
  })
})

describe('FR-037 — les filtres sont cumulés par le SERVEUR', () => {
  it('la requête porte les quatre filtres ensemble', async () => {
    const urls = fauxServeur({ elements: [] })
    const ecran = monter()

    const champs = ecran.findAll('input')
    await champs[0]!.setValue('2026-08-01')
    await champs[1]!.setValue('2026-08-02')
    await ecran.find('select').setValue('changement_role')
    await ecran.find('button').trigger('click')
    await flushPromises()

    expect(urls).toHaveLength(1)
    const url = urls[0]!
    expect(url).toContain('type_action=changement_role')
    expect(url).toContain('depuis=2026-08-01')
    expect(url).toContain('jusqu_a=2026-08-02')
    expect(url).toContain(`etablissement_id=${ETABLISSEMENT}`)
  })

  it('un filtre appliqué REMPLACE la liste, il ne s’y ajoute pas', async () => {
    fauxServeur({ elements: [] })
    const ecran = monter()

    expect(ecran.findAll('[data-entree]')).toHaveLength(2)

    await ecran.find('button').trigger('click')
    await flushPromises()

    // Concaténer laisserait dans le registre des entrées que le filtre exclut — une liste amputée
    // qui aurait l'air complète, à l'envers.
    expect(ecran.findAll('[data-entree]')).toHaveLength(0)
    expect(ecran.text()).toContain(fr.journal.aucune_entree)
  })

  it('la page suivante, elle, S’AJOUTE — c’est la suite du même parcours', async () => {
    fauxServeur({
      elements: [
        {
          id: '018f0000-0000-7000-8000-0000000000e3',
          type_action: 'changement_role',
          auteur: { compte_id: 'x', nom: 'Adjoua' },
          cible_type: 'compte',
          contexte: {},
          cree_le: '2026-08-01T12:00:00Z',
        },
      ],
    })
    const ecran = monter()

    const boutons = ecran.findAll('button')
    // Le dernier bouton est « Voir les précédentes » — il n'existe que s'il y a un curseur.
    await boutons[boutons.length - 1]!.trigger('click')
    await flushPromises()

    expect(ecran.findAll('[data-entree]')).toHaveLength(3)
  })

  it('sans curseur, aucun bouton de suite n’est rendu', () => {
    const ecran = monter({ elements: PAGE.elements })

    expect(ecran.html()).not.toContain(fr.journal.charger_suite)
  })
})

describe('la couche de lecture', () => {
  it('une page absente vaut page vide, jamais une erreur', async () => {
    globalThis.fetch = (async () =>
      new Response('null', {
        status: 200,
        headers: { 'content-type': 'application/json' },
      })) as typeof fetch

    // Un établissement où rien ne s'est encore passé est valide ; traiter l'absence comme une
    // erreur ferait échouer l'écran sur un état parfaitement normal.
    await expect(chargerJournal(CONTEXTE)).resolves.toEqual({ elements: [] })
  })

  it('le curseur voyage dans les deux paramètres du contrat', async () => {
    const urls = fauxServeur({ elements: [] })

    await chargerJournal(CONTEXTE, {}, { creeLe: AUTORITE, id: 'abc' })

    expect(urls[0]).toContain('apres_cree_le=')
    expect(urls[0]).toContain('apres_id=abc')
  })
})
