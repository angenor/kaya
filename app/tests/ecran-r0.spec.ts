// @vitest-environment happy-dom
/**
 * **`R0` — ce que l'écran de connexion MONTRE.**
 *
 * `auth-session.spec.ts` vérifie ce que la couche d'appel *décide* ; ce fichier constate ce que
 * l'écran *rend*. La distinction n'est pas théorique : une table de refus parfaite ne sert à rien
 * si le gabarit affiche le code au lieu de la phrase, et un bouton correctement caché hors ligne
 * ne prouve rien si la garde vit dans le composant plutôt que dans la fonction d'appel.
 *
 * # Les quatre propriétés vérifiées
 *
 * 1. **Les deux échecs rendent LA MÊME phrase** — et c'est le HTML rendu qui est comparé, pas une
 *    clé i18n ni un booléen.
 * 2. **Hors ligne, le refus est annoncé AVANT toute tentative** : le bouton est **absent du HTML**,
 *    et aucune requête ne part. L'état `degrade` compte comme hors ligne.
 * 3. **Aucune couleur littérale** — contrôle statique du fichier, en plus de la porte P-17.
 * 4. **Aucun `window.__TAURI__`, aucun stockage de navigateur nommé** hors de `PlatformAdapter` —
 *    contrôle statique, en plus de la porte P-15. L'écran de connexion est le premier à ranger un
 *    secret : c'est ici que la tentation d'écrire `localStorage` directement est la plus forte.
 */

import { flushPromises, mount } from '@vue/test-utils'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { readFileSync } from 'node:fs'
import { join } from 'node:path'

import Connexion from '../pages/connexion.vue'
import fr from '../core/i18n/fr.json'

const SOURCE = readFileSync(join(process.cwd(), 'pages/connexion.vue'), 'utf8')

const fetchOriginal = globalThis.fetch

/** Traduction depuis le catalogue français réel — un faux qui rendrait la clé ne prouverait rien. */
function traduire(cle: string, valeurs?: Record<string, unknown>): string {
  const brut = cle
    .split('.')
    .reduce<unknown>((noeud, part) => (noeud as Record<string, unknown>)?.[part], fr)
  if (typeof brut !== 'string') return cle
  return brut.replace(/\{(\w+)\}/g, (_, nom: string) => String(valeurs?.[nom] ?? ''))
}

/** Force l'état réseau que `navigator.onLine` rapporte. */
function poserReseau(enLigne: boolean): void {
  Object.defineProperty(globalThis.navigator, 'onLine', {
    value: enLigne,
    configurable: true,
  })
}

/** Faux serveur, et le journal de ce qui est réellement parti. */
function fauxServeur(statut: number, corps: unknown): string[] {
  const appels: string[] = []
  globalThis.fetch = (async (entree: string | URL | Request) => {
    appels.push(entree instanceof Request ? entree.url : String(entree))
    return new Response(JSON.stringify(corps), {
      status: statut,
      headers: { 'content-type': 'application/json' },
    })
  }) as typeof fetch
  return appels
}

function monter() {
  return mount(Connexion, {
    global: {
      mocks: { useI18n: () => ({ t: traduire }) },
      config: { globalProperties: { useI18n: () => ({ t: traduire }) } },
    },
  })
}

/** Saisit les deux champs et soumet. */
async function seConnecter(composant: ReturnType<typeof monter>): Promise<void> {
  const champs = composant.findAll('input')
  await champs[0]!.setValue('+2250700000001')
  await champs[1]!.setValue('chaise-tomate-abidjan')
  await composant.find('form').trigger('submit')
  await flushPromises()
}

beforeEach(() => {
  poserReseau(true)
  // Les auto-imports de Nuxt n'existent pas hors du pipeline : on les pose, et rien d'autre.
  ;(globalThis as Record<string, unknown>).useI18n = () => ({ t: traduire })
  ;(globalThis as Record<string, unknown>).useRuntimeConfig = () => ({
    public: { apiBaseUrl: 'http://localhost:8080' },
  })
  ;(globalThis as Record<string, unknown>).navigateTo = async () => undefined
})

afterEach(() => {
  globalThis.fetch = fetchOriginal
  vi.restoreAllMocks()
})

describe('FR-012 — les deux échecs rendent la même phrase', () => {
  it('identifiant inconnu et mot de passe faux affichent un texte identique', async () => {
    // Le serveur ne rend qu'un code ; le test présente les deux qu'un serveur bavard aurait pu
    // rendre. Si l'écran les distinguait un jour, les deux rendus cesseraient d'être égaux.
    fauxServeur(401, { code: 'identifiants_invalides', message: 'unknown identifier' })
    const premier = monter()
    await seConnecter(premier)
    const phrasePremier = premier.find('[role="alert"]').text()

    fauxServeur(401, { code: 'mot_de_passe_invalide', message: 'argon2 mismatch' })
    const second = monter()
    await seConnecter(second)
    const phraseSecond = second.find('[role="alert"]').text()

    expect(phrasePremier).toBe(phraseSecond)
    expect(phrasePremier).toContain(fr.connexion.refus.identifiants)
  })

  it('la phrase rendue est celle du lexique, et rien d’autre n’en sort', async () => {
    fauxServeur(401, { code: 'identifiants_invalides', message: 'no row in comptes.compte' })
    const composant = monter()
    await seConnecter(composant)

    const html = composant.html()
    // Ni le diagnostic du serveur, ni le nom d'une table, ni le mot « rôle », « permission » ou
    // « jeton » — que `docs/design/lexique.md` interdit à l'interface.
    expect(html).not.toContain('comptes.compte')
    expect(html).not.toMatch(/jeton|JWT|permission|rôle/i)
    expect(composant.find('[role="alert"]').text()).toBe(fr.connexion.refus.identifiants)
  })

  it('la méthode non implémentée, elle, a sa propre phrase', async () => {
    fauxServeur(422, { code: 'methode_non_implementee', message: 'OTP_SMS' })
    const composant = monter()
    await seConnecter(composant)

    expect(composant.find('[role="alert"]').text())
      .toContain(fr.connexion.refus.methode_non_implementee)
  })
})

/**
 * **Le versant positif, et il n'est pas décoratif.**
 *
 * Les trois assertions hors-ligne du bloc suivant portent sur une **absence** : bouton absent,
 * bandeau unique, zéro requête. Toutes les trois passeraient si l'écran ne se montait pas du tout.
 * Ce bloc établit qu'en ligne il se monte, qu'il agit, et que la connexion aboutit — sans quoi la
 * porte serait verte en n'ayant rien à inspecter.
 */
describe('en ligne — l’écran se monte et la connexion aboutit', () => {
  it('le bouton existe, la requête part, et l’écran change', async () => {
    const appels = fauxServeur(200, {
      acces: 'acces-1',
      rafraichissement: 'rafraichissement-1',
      expire_dans_s: 3600,
      permissions: ['etb.service.basculer'],
      etablissements: ['etb-1'],
      compte: { compte_id: 'c-1', tenant_id: 't-1', etablissement_actif: 'etb-1' },
    })
    let destination: string | null = null
    ;(globalThis as Record<string, unknown>).navigateTo = async (cible: string) => {
      destination = cible
    }

    const composant = monter()
    expect(composant.find('button[type="submit"]').exists()).toBe(true)

    await seConnecter(composant)

    expect(appels).toHaveLength(1)
    expect(appels[0]).toContain('/api/v1/session')
    expect(destination).toBe('/')
    // Aucun bandeau de refus : la réussite ne s'annonce pas, elle se constate au changement
    // d'écran (principe VII — l'effet visible vaut mieux qu'un message).
    expect(composant.findAll('[role="alert"]')).toHaveLength(0)
  })
})

describe('hors ligne — le refus précède toute tentative', () => {
  it('le bouton est ABSENT du HTML rendu, pas grisé', async () => {
    poserReseau(false)
    const composant = monter()
    await flushPromises()

    // Absence, pas `disabled` : le grisé apprend à l'utilisateur qu'une partie du produit lui est
    // refusée, sans dire laquelle ni jusqu'à quand (principe VII).
    expect(composant.find('button[type="submit"]').exists()).toBe(false)
    expect(composant.html()).not.toContain('disabled')
  })

  it('un bandeau dit pourquoi, en une phrase, avant toute saisie', async () => {
    poserReseau(false)
    const composant = monter()
    await flushPromises()

    const bandeaux = composant.findAll('[role="alert"]')
    // **Un seul bandeau, jamais deux empilés** (composant 07).
    expect(bandeaux).toHaveLength(1)
    expect(bandeaux[0]!.text()).toContain(fr.connexion.refus.reseau)
  })

  it('aucune requête ne part, même si la soumission est provoquée', async () => {
    poserReseau(false)
    const appels = fauxServeur(200, {})
    const composant = monter()
    await seConnecter(composant)

    expect(appels).toHaveLength(0)
  })
})

describe('validation au champ', () => {
  it('un identifiant vide se signale AU CHAMP, pas au bandeau', async () => {
    const appels = fauxServeur(200, {})
    const composant = monter()

    await composant.find('form').trigger('submit')
    await flushPromises()

    // Aucun appel : la validation précède la requête.
    expect(appels).toHaveLength(0)
    // Le message est celui du champ, rendu par le composant 16 — donc dans un `[role="alert"]`
    // porté par le champ lui-même, pas dans un bandeau d'écran.
    const messages = composant.findAll('[role="alert"]').map(n => n.text())
    expect(messages.length).toBeGreaterThanOrEqual(2)
    for (const message of messages) {
      expect(message).toContain(fr.champ.erreur.obligatoire)
    }
  })

  it('l’erreur de champ porte trois signaux, jamais la couleur seule', async () => {
    const composant = monter()
    await composant.find('form').trigger('submit')
    await flushPromises()

    // Bordure `danger`, message, icône d'avertissement — composant 04. Sur un écran délavé par le
    // soleil, la bordure rouge seule ne se voit pas.
    expect(composant.html()).toContain('border-danger')
    expect(composant.html()).toContain('ph-warning-circle')
    expect(composant.find('input[aria-invalid="true"]').exists()).toBe(true)
  })
})

describe('le champ mot de passe est un vrai champ masqué', () => {
  it('il passe par le composant 16 et porte son autocomplete', () => {
    const composant = monter()

    const motDePasse = composant.find('input[type="password"]')
    expect(motDePasse.exists()).toBe(true)
    // Sans `autocomplete`, les gestionnaires de mots de passe ne fonctionnent pas — et l'on
    // choisit alors un mot de passe qu'on retient, donc qu'on écrit au comptoir.
    expect(motDePasse.attributes('autocomplete')).toBe('current-password')
    expect(composant.find('input[autocomplete="username"]').exists()).toBe(true)
  })

  it('le mot de passe n’est jamais rendu en clair dans le HTML', async () => {
    fauxServeur(401, { code: 'identifiants_invalides', message: 'x' })
    const composant = monter()
    await seConnecter(composant)

    expect(composant.html()).not.toContain('chaise-tomate-abidjan')
  })
})

describe('contrôles statiques — P-17 et P-15 sur cet écran', () => {
  it('aucune couleur littérale', () => {
    // Les jetons portent les couleurs ; une valeur littérale ne changerait pas sous `.dark`.
    expect(SOURCE).not.toMatch(/#[0-9a-fA-F]{3,8}\b/)
    expect(SOURCE).not.toMatch(/\brgba?\(/)
    expect(SOURCE).not.toMatch(/\bhsla?\(/)
  })

  it('aucun accès natif ni stockage de navigateur — tout passe par PlatformAdapter', () => {
    expect(SOURCE).not.toContain('__TAURI__')
    expect(SOURCE).not.toContain('@tauri-apps/')
    // L'écran range un jeton de rafraîchissement par `core/auth`, qui passe lui-même par
    // `PlatformAdapter`. Le nommer ici court-circuiterait la seule porte du principe VII.
    expect(SOURCE).not.toMatch(/localStorage|sessionStorage|indexedDB|document\.cookie/)
  })

  it('l’écran ne décode aucun jeton — les permissions viennent du corps de la réponse', () => {
    expect(SOURCE).not.toMatch(/\batob\b|jwtDecode|jwt_decode/)
  })

  it('la matrice de dérivation porte bien R0 — sans quoi cet écran ne se code pas (P-19)', () => {
    const derivation = readFileSync(
      join(process.cwd(), '../docs/design/derivation.md'),
      'utf8',
    )
    // « Un écran absent des deux ne se code pas. » La ligne a été ajoutée par T001, AVANT
    // l'écran ; l'assertion empêche qu'on la retire en laissant l'écran derrière.
    expect(derivation).toMatch(/\|\s*`R0` Connexion\s*\|\s*`G2`\s*\|/)
  })
})
