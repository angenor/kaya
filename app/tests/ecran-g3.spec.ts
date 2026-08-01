// @vitest-environment happy-dom
/**
 * **`G3` — ce que l'écran des accès MONTRE, et ce que la couche d'appel DÉCIDE.**
 *
 * # La propriété centrale : l'action absente est absente du HTML
 *
 * Sans `cpt.role.attribuer`, il n'y a **rien** dans le HTML rendu — ni bouton désactivé, ni
 * `title` explicatif, ni élément masqué par CSS. C'est le HTML qui est vérifié, pas la valeur d'un
 * booléen : un composant peut perdre cette propriété par un `v-show` au lieu d'un `v-if` sans que
 * la valeur du `computed` change d'une ligne.
 *
 * # Et la seconde : hors ligne, l'action disparaît ET un bandeau dit pourquoi
 *
 * Les deux absences se ressemblent à l'écran et n'ont rien à voir. Un droit manquant n'est pas une
 * nouvelle à annoncer — l'action n'existe pas, et rien ne le dit. Une coupure réseau, si : elle
 * est temporaire, l'utilisateur doit savoir qu'il peut réessayer. Ce fichier vérifie que les deux
 * cas se distinguent **dans le rendu**.
 *
 * L'état `degrade` compte comme hors ligne pour une opération de classe C — `navigator.onLine` dit
 * qu'une interface réseau est active, pas que le serveur répond.
 */

import { flushPromises, mount } from '@vue/test-utils'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

import EcranComptes from '../modules/comptes/EcranComptes.vue'
import { attribuerRole, PERMISSION_ATTRIBUER, retirerRole, TYPE_ATTRIBUTION, TYPE_RETRAIT } from '../modules/comptes/roles'
import { estTypeClasseA } from '../core/sync/classes'
import fr from '../core/i18n/fr.json'
import type { CompteVue, EntreeRole } from '../modules/comptes/donnees'

const CONTEXTE = { baseUrl: 'http://localhost:8080', acces: 'jeton-de-test' }
const ETABLISSEMENT = '018f0000-0000-7000-8000-000000000001'

const REFERENTIEL: EntreeRole[] = [
  { code: 'proprietaire', libelle_cle: 'comptes.roles.proprietaire', ordre: 10, portee: 'ETABLISSEMENT' },
  { code: 'gerant', libelle_cle: 'comptes.roles.gerant', ordre: 20, portee: 'ETABLISSEMENT' },
  { code: 'caissier', libelle_cle: 'comptes.roles.caissier', ordre: 50, portee: 'ETABLISSEMENT' },
  { code: 'comptable', libelle_cle: 'comptes.roles.comptable', ordre: 70, portee: 'ETABLISSEMENT' },
  { code: 'admin_editeur', libelle_cle: 'comptes.roles.admin_editeur', ordre: 80, portee: 'EDITEUR' },
]

const ADJOUA: CompteVue = {
  id: '018f0000-0000-7000-8000-0000000000a1',
  personne_id: '018f0000-0000-7000-8000-0000000000b1',
  nom_affichage: 'Adjoua',
  identifiant_telephone: '+2250700000001',
  identifiant_email: null,
  methode_code: 'MOT_DE_PASSE',
  actif: true,
  roles: [
    { role_code: 'gerant', etablissement_id: ETABLISSEMENT },
    { role_code: 'caissier', etablissement_id: ETABLISSEMENT },
    // Un rôle porté **sur un autre établissement** — il ne doit pas apparaître sur cet écran.
    { role_code: 'comptable', etablissement_id: '018f0000-0000-7000-8000-0000000000ff' },
  ],
  cree_le: '2026-08-01T10:00:00Z',
  modifie_le: '2026-08-01T10:00:00Z',
}

const fetchOriginal = globalThis.fetch

function traduire(cle: string, valeurs?: Record<string, unknown>): string {
  const brut = cle
    .split('.')
    .reduce<unknown>((noeud, part) => (noeud as Record<string, unknown>)?.[part], fr)
  if (typeof brut !== 'string') return cle
  return brut.replace(/\{(\w+)\}/g, (_, nom: string) => String(valeurs?.[nom] ?? ''))
}

function poserReseau(enLigne: boolean): void {
  Object.defineProperty(globalThis.navigator, 'onLine', { value: enLigne, configurable: true })
}

function fauxServeur(statut: number, corps: unknown): { url: string, methode: string }[] {
  const appels: { url: string, methode: string }[] = []
  globalThis.fetch = (async (entree: string | URL | Request) => {
    const requete = entree instanceof Request ? entree : null
    appels.push({
      url: requete ? requete.url : String(entree),
      methode: requete ? requete.method : 'GET',
    })
    return new Response(statut === 204 ? null : JSON.stringify(corps), {
      status: statut,
      headers: { 'content-type': 'application/json' },
    })
  }) as typeof fetch
  return appels
}

function monter(options: { permissions?: string[], comptes?: CompteVue[] } = {}) {
  return mount(EcranComptes, {
    props: {
      comptes: options.comptes ?? [ADJOUA],
      referentielRoles: REFERENTIEL,
      contexte: CONTEXTE,
      etablissementId: ETABLISSEMENT,
      permissions: options.permissions ?? [],
    },
    global: {
      mocks: { useI18n: () => ({ t: traduire }) },
      config: { globalProperties: { useI18n: () => ({ t: traduire }) } },
    },
  })
}

beforeEach(() => {
  poserReseau(true)
  ;(globalThis as Record<string, unknown>).useI18n = () => ({ t: traduire })
})

afterEach(() => {
  globalThis.fetch = fetchOriginal
  vi.restoreAllMocks()
})

describe('principe VII — sans permission, l’action est ABSENTE du HTML', () => {
  it('aucun bouton d’ajout, aucun bouton de retrait', () => {
    const ecran = monter({ permissions: [] })
    const html = ecran.html()

    expect(html).not.toContain(fr.comptes.action_ajouter)
    // Pas de grisé déguisé : ni `disabled`, ni `aria-disabled`, ni élément masqué par CSS.
    expect(html).not.toContain('disabled')
    expect(html).not.toContain('aria-disabled')
    // Ni masquage par CSS : un élément présent et invisible reste dans le HTML, donc atteignable
    // par un outil, et il apprend à qui le trouve qu'une action lui est refusée.
    expect(html).not.toMatch(/display:\s*none/)
    expect(html).not.toMatch(/visibility:\s*hidden/)
    expect(ecran.findAll('button')).toHaveLength(0)
  })

  it('mais la liste, elle, s’affiche — la lecture n’est pas l’écriture', () => {
    // Versant positif : sans lui, l'assertion précédente serait vraie sur un écran qui ne rend
    // rien du tout.
    const ecran = monter({ permissions: [] })

    expect(ecran.text()).toContain('Adjoua')
    expect(ecran.text()).toContain(fr.comptes.roles.gerant)
  })

  it('avec la permission ET le réseau, les actions existent', () => {
    const ecran = monter({ permissions: [PERMISSION_ATTRIBUER] })

    expect(ecran.text()).toContain(fr.comptes.action_ajouter)
    expect(ecran.findAll('button').length).toBeGreaterThan(0)
  })
})

describe('classe C — hors ligne, l’action disparaît et un bandeau dit pourquoi', () => {
  it('le bouton n’est plus rendu, et le bandeau explique', async () => {
    poserReseau(false)
    const ecran = monter({ permissions: [PERMISSION_ATTRIBUER] })
    await flushPromises()

    expect(ecran.html()).not.toContain(fr.comptes.action_ajouter)
    const bandeau = ecran.find('[role="status"]')
    expect(bandeau.exists()).toBe(true)
    expect(bandeau.text()).toContain(fr.comptes.refus.reseau)
  })

  it('les deux absences se distinguent — sans droit, AUCUN bandeau', async () => {
    // C'est la propriété qu'on perdrait le plus facilement : afficher l'avis réseau à qui n'a pas
    // le droit lui apprendrait qu'une action existe et qu'elle lui est refusée.
    poserReseau(false)
    const ecran = monter({ permissions: [] })
    await flushPromises()

    expect(ecran.find('[role="status"]').exists()).toBe(false)
    expect(ecran.find('[role="alert"]').exists()).toBe(false)
  })

  it('aucune requête ne part quand le réseau manque', async () => {
    const appels = fauxServeur(201, {})

    const resultat = await attribuerRole(CONTEXTE, ADJOUA.id, 'caissier', ETABLISSEMENT, 'hors_ligne')

    expect(resultat).toMatchObject({ issue: 'refus', reseau: true })
    expect(appels).toHaveLength(0)
  })

  it('l’état `degrade` compte COMME hors ligne', async () => {
    // `navigator.onLine` dit qu'une interface est active, pas que le serveur répond. Une 3G qui
    // affiche « en ligne » sans porter la moindre requête est le cas courant à Abengourou.
    const appels = fauxServeur(204, null)

    const resultat = await retirerRole(CONTEXTE, ADJOUA.id, 'caissier', ETABLISSEMENT, 'degrade')

    expect(resultat).toMatchObject({ issue: 'refus', reseau: true })
    expect(appels).toHaveLength(0)
  })

  it('une élévation de privilège n’entre JAMAIS en file', () => {
    // Porte P-13, versant négatif : les deux types de ce module ne sont pas déclarés classe A, et
    // la file les refuserait même marqués.
    expect(estTypeClasseA(TYPE_ATTRIBUTION)).toBe(false)
    expect(estTypeClasseA(TYPE_RETRAIT)).toBe(false)
  })
})

describe('les rôles affichés sont ceux de CET établissement', () => {
  it('un rôle porté ailleurs n’apparaît pas', () => {
    const ecran = monter({ permissions: [PERMISSION_ATTRIBUER] })

    expect(ecran.text()).toContain(fr.comptes.roles.gerant)
    expect(ecran.text()).toContain(fr.comptes.roles.caissier)
    // `comptable` est porté sur un AUTRE établissement : l'afficher ici donnerait une liste fausse
    // de ce que cette personne peut faire sur cet établissement-ci.
    expect(ecran.text()).not.toContain(fr.comptes.roles.comptable)
  })

  it('`admin_editeur` n’est pas proposé à l’attribution', () => {
    // Sa portée est l'éditeur : l'attribuer avec un `etablissement_id` est refusé en `422` par le
    // serveur. Le proposer produirait une action qui échoue à tous les coups.
    const ecran = monter({ permissions: [PERMISSION_ATTRIBUER] })

    expect(ecran.html()).not.toContain('admin_editeur')
    expect(ecran.text()).not.toContain(fr.comptes.roles.admin_editeur)
  })

  it('un compte sans rôle ici le DIT, plutôt que de laisser une ligne vide', () => {
    const nu: CompteVue = { ...ADJOUA, id: 'x', nom_affichage: 'Yao', roles: [] }
    const ecran = monter({ permissions: [], comptes: [nu] })

    expect(ecran.text()).toContain(fr.comptes.aucun_role)
  })

  it('une liste vide a son état explicite', () => {
    const ecran = monter({ permissions: [], comptes: [] })

    expect(ecran.text()).toContain(fr.comptes.aucun)
  })
})

describe('les refus sont traduits du CODE, jamais du message du serveur', () => {
  it('`derniere_habilitation` rend la phrase qui dit quoi faire', async () => {
    fauxServeur(409, {
      code: 'derniere_habilitation',
      message: 'last holder of cpt.role.attribuer on etablissement',
    })

    const resultat = await retirerRole(CONTEXTE, ADJOUA.id, 'gerant', ETABLISSEMENT, 'connecte')

    expect(resultat).toMatchObject({
      issue: 'refus',
      cle: 'comptes.refus.derniere_habilitation',
    })
    // Le seul refus métier du cycle est irréversible sans l'éditeur : la phrase dit quoi faire.
    expect(fr.comptes.refus.derniere_habilitation).toContain("quelqu'un d'autre")
  })

  it('`403` rend le refus de permission, sans diagnostic', async () => {
    fauxServeur(403, { code: 'permission_absente', valeur: 'cpt.role.attribuer' })

    const resultat = await attribuerRole(CONTEXTE, ADJOUA.id, 'caissier', ETABLISSEMENT, 'connecte')

    expect(resultat).toEqual({ issue: 'refus', cle: 'comptes.refus.permission' })
  })

  it('un code inconnu tombe sur une phrase honnête, jamais sur une clé en brut', async () => {
    fauxServeur(422, { code: 'quelque_chose_de_neuf', message: 'x' })

    const resultat = await attribuerRole(CONTEXTE, ADJOUA.id, 'caissier', ETABLISSEMENT, 'connecte')

    // La phrase générique est celle de la table PARTAGÉE : un code que personne ne connaît ne
    // mérite pas une formulation par module, il mérite une phrase honnête et la même partout.
    expect(resultat).toMatchObject({ issue: 'refus', cle: 'erreurs.inattendue' })
  })

  it('le message de diagnostic du serveur n’atteint jamais le résultat', async () => {
    fauxServeur(422, { code: 'portee_incompatible', message: 'comptes.compte_role constraint' })

    const resultat = await attribuerRole(CONTEXTE, ADJOUA.id, 'caissier', null, 'connecte')

    expect(JSON.stringify(resultat)).not.toContain('comptes.compte_role')
  })
})

describe('l’appel passe par le client généré, avec un UUID v7', () => {
  it('l’attribution poste sur le chemin du contrat et porte le jeton', async () => {
    const appels = fauxServeur(201, {})

    await attribuerRole(CONTEXTE, ADJOUA.id, 'caissier', ETABLISSEMENT, 'connecte')

    expect(appels).toHaveLength(1)
    expect(appels[0]!.url).toContain(`/api/v1/comptes/${ADJOUA.id}/roles`)
    expect(appels[0]!.methode).toBe('POST')
  })

  it('le retrait passe le rôle dans le chemin et l’établissement en requête', async () => {
    const appels = fauxServeur(204, null)

    await retirerRole(CONTEXTE, ADJOUA.id, 'caissier', ETABLISSEMENT, 'connecte')

    expect(appels[0]!.methode).toBe('DELETE')
    expect(appels[0]!.url).toContain(`/api/v1/comptes/${ADJOUA.id}/roles/caissier`)
    expect(appels[0]!.url).toContain(`etablissement_id=${ETABLISSEMENT}`)
  })
})

describe('vocabulaire — le lexique est opposable', () => {
  it('ni « rôle », ni « permission », ni « jeton » dans le HTML rendu', () => {
    // `docs/design/lexique.md` : RBAC → « Ce que chacun peut faire ». Les mots de la mécanique
    // n'atteignent jamais l'interface.
    const ecran = monter({ permissions: [PERMISSION_ATTRIBUER] })
    const texte = ecran.text()

    for (const interdit of ['permission', 'jeton', 'JWT', 'token']) {
      expect(texte.toLowerCase()).not.toContain(interdit.toLowerCase())
    }
    // « rôle » est cherché en mot entier : « contrôle » le contient.
    expect(texte).not.toMatch(/\brôles?\b/i)
  })

  it('aucun identifiant technique ne s’affiche', () => {
    const ecran = monter({ permissions: [PERMISSION_ATTRIBUER] })
    const texte = ecran.text()

    expect(texte).not.toContain(ADJOUA.id)
    expect(texte).not.toContain('cpt.role.attribuer')
    // Le numéro de téléphone est un contact personnel : il n'a rien à faire dans une liste
    // consultable par tout compte habilité à lire.
    expect(texte).not.toContain('+2250700000001')
  })
})
