// @vitest-environment happy-dom
/**
 * **La couche d'authentification, côté logique** — CPT-01, T033.
 *
 * `ecran-r0.spec.ts` vérifie ce que l'écran *montre* ; ce fichier vérifie ce que la couche
 * *décide*. Les deux sont nécessaires : une phrase unique correctement rendue par l'écran ne sert
 * à rien si la fonction distingue les cas en amont, et une garde hors-ligne dans la fonction ne
 * protège pas d'un composant qui appellerait ailleurs.
 *
 * # Les cinq propriétés vérifiées, et celle qui ne se voit pas en relecture
 *
 * 1. **Refus hors ligne AVANT l'appel** — aucune requête ne part, et l'état `degrade` compte
 *    comme hors ligne.
 * 2. **Tout `401` rend LA MÊME phrase**, quel que soit le code reçu. *C'est celle-là.* Un front
 *    qui brancherait sa table sur le code passerait ce test aujourd'hui et rouvrirait la fuite de
 *    FR-012 au premier code que le serveur ajouterait — sans que rien n'échoue. Le test présente
 *    donc **trois codes différents**, dont deux que le serveur ne produit pas.
 * 3. **Les permissions viennent du CORPS, jamais du jeton.** Le jeton du faux serveur en porte
 *    d'autres, exprès : si un jour quelqu'un décodait le JWT « pour vérifier », ce test le dirait.
 * 4. **Le stockage passe par `PlatformAdapter`** — et le jeton d'**accès** n'y va pas.
 * 5. **La déconnexion purge**, y compris quand l'appel réseau échoue.
 */

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { readFileSync, readdirSync } from 'node:fs'
import { join } from 'node:path'

import {
  effacerSession,
  fermerSession,
  lireRafraichissement,
  ouvrirSession,
  oublierRafraichissement,
  rafraichirSession,
  REFUS_IDENTIFIANTS,
  REFUS_INATTENDU,
  REFUS_METHODE,
  REFUS_RESEAU,
  REFUS_SESSION_EXPIREE,
  sessionCourante,
} from '../core/auth'

const BASE = 'http://localhost:8080'
const IDENTIFIANTS = { identifiant: '+2250700000001', motDePasse: 'chaise-tomate-abidjan' }

const fetchOriginal = globalThis.fetch

/**
 * Un jeton d'accès **dont la charge utile ment**.
 *
 * Ses permissions ne sont pas celles du corps de la réponse. C'est délibéré : c'est ce qui rend
 * observable un décodage de jeton côté front, que la research R-06 interdit.
 */
function jetonMenteur(): string {
  const entete = btoa(JSON.stringify({ alg: 'HS256', typ: 'JWT' }))
  const charge = btoa(JSON.stringify({ permissions: ['cpt.tout.faire'], sub: 'quelqu-un-dautre' }))
  return `${entete}.${charge}.signature-non-verifiee-par-le-front`
}

function corpsSession(permissions: string[]) {
  return {
    acces: jetonMenteur(),
    rafraichissement: 'rafraichissement-1',
    expire_dans_s: 3600,
    permissions,
    etablissements: ['etb-1', 'etb-2'],
    compte: { compte_id: 'compte-1', tenant_id: 'tenant-1', etablissement_actif: 'etb-1' },
  }
}

/** Remplace `fetch` par une réponse figée, et retient ce qui a été envoyé. */
function fauxServeur(statut: number, corps: unknown) {
  const appels: { url: string, corps: unknown }[] = []

  globalThis.fetch = (async (entree: string | URL | Request, options?: RequestInit) => {
    const requete = entree instanceof Request ? entree : null
    const brut = requete ? await requete.clone().text() : options?.body
    appels.push({
      url: requete ? requete.url : String(entree),
      corps: brut ? JSON.parse(String(brut)) : undefined,
    })
    return new Response(statut === 204 ? null : JSON.stringify(corps), {
      status: statut,
      headers: { 'content-type': 'application/json' },
    })
  }) as typeof fetch

  return appels
}

beforeEach(async () => {
  effacerSession()
  await oublierRafraichissement()
})

afterEach(() => {
  globalThis.fetch = fetchOriginal
  vi.restoreAllMocks()
})

describe('refus hors ligne — avant tout appel', () => {
  it('hors ligne, aucune requête ne part', async () => {
    const appels = fauxServeur(200, corpsSession([]))

    const resultat = await ouvrirSession(BASE, IDENTIFIANTS, 'hors_ligne')

    expect(resultat).toEqual({ issue: 'refus', cle: REFUS_RESEAU, reseau: true })
    expect(appels).toHaveLength(0)
  })

  it('réseau dégradé — traité COMME hors ligne, et pour la même raison', async () => {
    // `navigator.onLine` dit qu'une interface est active, pas que le serveur répond. Tenter la
    // connexion sur une 3G qui ne porte rien ferait attendre trente secondes pour un échec.
    const appels = fauxServeur(200, corpsSession([]))

    const resultat = await ouvrirSession(BASE, IDENTIFIANTS, 'degrade')

    expect(resultat).toMatchObject({ issue: 'refus', reseau: true })
    expect(appels).toHaveLength(0)
  })
})

describe('FR-012 — les échecs sont indiscernables', () => {
  // Les deux derniers codes ne sont PAS produits par le serveur : ils sont là pour que le test
  // échoue si quelqu'un branchait un jour la table des clés sur le code.
  it.each([
    'identifiants_invalides',
    'compte_inconnu',
    'trop_de_tentatives',
  ])('un 401 portant « %s » rend la phrase unique', async (code) => {
    fauxServeur(401, { code, message: `internal: ${code}` })

    const resultat = await ouvrirSession(BASE, IDENTIFIANTS, 'connecte')

    expect(resultat).toEqual({ issue: 'refus', cle: REFUS_IDENTIFIANTS })
  })

  it('le message de diagnostic du serveur n’atteint jamais le résultat', async () => {
    fauxServeur(401, { code: 'identifiants_invalides', message: 'no row in comptes.compte' })

    const resultat = await ouvrirSession(BASE, IDENTIFIANTS, 'connecte')

    expect(JSON.stringify(resultat)).not.toContain('comptes.compte')
  })

  it('la méthode non implémentée, elle, a sa propre phrase — c’est un refus qui enseigne', async () => {
    fauxServeur(422, { code: 'methode_non_implementee', message: 'OTP_SMS' })

    const resultat = await ouvrirSession(BASE, IDENTIFIANTS, 'connecte')

    expect(resultat).toEqual({ issue: 'refus', cle: REFUS_METHODE })
  })

  it('un code inconnu tombe sur une phrase honnête, jamais sur une clé affichée en brut', async () => {
    fauxServeur(503, { code: 'quelque_chose_de_neuf', message: 'x' })

    const resultat = await ouvrirSession(BASE, IDENTIFIANTS, 'connecte')

    expect(resultat).toEqual({ issue: 'refus', cle: REFUS_INATTENDU })
  })
})

describe('research R-06 — les permissions viennent du corps, jamais du jeton', () => {
  it('la session porte les permissions de la réponse', async () => {
    fauxServeur(200, corpsSession(['etb.service.basculer', 'cpt.compte.lire']))

    const resultat = await ouvrirSession(BASE, IDENTIFIANTS, 'connecte')

    expect(resultat.issue).toBe('succes')
    expect(sessionCourante()?.permissions).toEqual(['etb.service.basculer', 'cpt.compte.lire'])
    // Celle du jeton — si elle apparaissait, le front décoderait le JWT.
    expect(sessionCourante()?.permissions).not.toContain('cpt.tout.faire')
    expect(sessionCourante()?.compteId).toBe('compte-1')
    expect(sessionCourante()?.tenantId).toBe('tenant-1')
    expect(sessionCourante()?.etablissementId).toBe('etb-1')
  })

  it('aucun fichier de core/auth ne décode un jeton ni ne nomme un stockage de navigateur', () => {
    // Contrôle **statique**, et son périmètre est déclaré : les trois fichiers de `core/auth`.
    // Il attrape la faute au moment où elle est écrite, pas au moment où elle diverge.
    // `import.meta.url` n'est PAS un `file:` sous happy-dom — il rendrait un chemin absolu faux,
    // et `readdirSync` échouerait sur une racine qui n'existe pas. La racine de Vitest est `app/`.
    const racine = join(process.cwd(), 'core/auth')
    const fichiers = readdirSync(racine).filter(f => f.endsWith('.ts'))

    expect(fichiers.length).toBeGreaterThanOrEqual(3)

    for (const fichier of fichiers) {
      const source = readFileSync(join(racine, fichier), 'utf8')
      // Les commentaires disent « localStorage » pour expliquer pourquoi il n'est pas employé :
      // on n'inspecte donc que le code, commentaires retirés.
      const code = source
        .replace(/\/\*[\s\S]*?\*\//g, '')
        .replace(/^\s*\/\/.*$/gm, '')
      expect(code, `${fichier} nomme un stockage de navigateur`).not.toMatch(/localStorage|sessionStorage|indexedDB/)
      expect(code, `${fichier} décode un jeton`).not.toMatch(/\batob\b|jwtDecode|jwt_decode/)
    }
  })
})

describe('porte P-15 — le stockage passe par PlatformAdapter', () => {
  it('le rafraîchissement est rangé, le jeton d’accès ne l’est pas', async () => {
    fauxServeur(200, corpsSession([]))

    const resultat = await ouvrirSession(BASE, IDENTIFIANTS, 'connecte')

    expect(resultat).toMatchObject({ issue: 'succes', persistante: true })
    expect(await lireRafraichissement()).toBe('rafraichissement-1')

    // Le jeton d'accès vit en mémoire et nulle part ailleurs : il meurt avec l'onglet.
    const tout = Object.keys(localStorage).map(cle => localStorage.getItem(cle)).join('|')
    expect(tout).not.toContain(jetonMenteur())
  })

  it('la déconnexion purge le stockage, même si l’appel réseau échoue', async () => {
    fauxServeur(200, corpsSession([]))
    await ouvrirSession(BASE, IDENTIFIANTS, 'connecte')
    expect(await lireRafraichissement()).not.toBeNull()

    globalThis.fetch = (async () => {
      throw new Error('réseau coupé')
    }) as typeof fetch

    await fermerSession(BASE)

    expect(sessionCourante()).toBeNull()
    expect(await lireRafraichissement()).toBeNull()
  })
})

describe('rotation du rafraîchissement', () => {
  it('le nouveau jeton REMPLACE l’ancien — sinon le suivant échouerait', async () => {
    fauxServeur(200, corpsSession([]))
    await ouvrirSession(BASE, IDENTIFIANTS, 'connecte')

    fauxServeur(200, { ...corpsSession(['cpt.compte.lire']), rafraichissement: 'rafraichissement-2' })
    const resultat = await rafraichirSession(BASE)

    expect(resultat.issue).toBe('succes')
    expect(await lireRafraichissement()).toBe('rafraichissement-2')
    // Les permissions sont **recalculées** à chaque rafraîchissement : un rôle retiré prend
    // effet ici, au plus soixante minutes après.
    expect(sessionCourante()?.permissions).toEqual(['cpt.compte.lire'])
  })

  it('un jeton consommé, révoqué ou inconnu efface le rafraîchissement rangé', async () => {
    fauxServeur(200, corpsSession([]))
    await ouvrirSession(BASE, IDENTIFIANTS, 'connecte')

    fauxServeur(401, { code: 'session_invalide', message: 'x' })
    const resultat = await rafraichirSession(BASE)

    expect(resultat).toEqual({ issue: 'refus', cle: REFUS_SESSION_EXPIREE })
    // Le garder ferait échouer chaque tentative suivante à l'identique, sans que rien ne le dise.
    expect(await lireRafraichissement()).toBeNull()
  })

  it('sans jeton rangé, le rafraîchissement refuse sans partir en requête', async () => {
    const appels = fauxServeur(200, corpsSession([]))

    const resultat = await rafraichirSession(BASE)

    expect(resultat).toEqual({ issue: 'refus', cle: REFUS_SESSION_EXPIREE })
    expect(appels).toHaveLength(0)
  })
})

// =================================================================================================
//  Principe VI — aucune donnée de classe C en cache d'écriture, et purge à la déconnexion
// =================================================================================================

describe('les données d’identité ne restent pas sur le terminal', () => {
  it('la déconnexion purge TOUT le stockage, pas seulement le jeton', async () => {
    // Le terminal peut être partagé : au maquis, le téléphone du gérant sert aussi à la serveuse.
    // Retirer le jeton en laissant un cache de comptes serait une demi-purge.
    fauxServeur(200, corpsSession(['cpt.compte.lire']))
    await ouvrirSession(BASE, IDENTIFIANTS, 'connecte')

    // Une donnée de module rangée à côté — celle qu'une demi-purge laisserait derrière.
    localStorage.setItem('kaya.cache.comptes', JSON.stringify([{ nom: 'Adjoua' }]))

    globalThis.fetch = (async () => new Response(null, { status: 204 })) as typeof fetch
    await fermerSession(BASE)

    const restant = Object.keys(localStorage).filter(cle => cle.startsWith('kaya.'))
    expect(restant, `le stockage porte encore : ${restant.join(', ')}`).toEqual([])
  })

  it('aucune donnée de classe C n’est écrite par la couche d’authentification', async () => {
    fauxServeur(200, corpsSession(['cpt.compte.lire', 'etb.service.basculer']))
    await ouvrirSession(BASE, IDENTIFIANTS, 'connecte')

    // Seule la clé du rafraîchissement existe. Ni les permissions, ni le compte, ni le tenant :
    // ce sont des données de classe C, et le principe VI interdit leur cache d'écriture sur un
    // terminal. Elles vivent en mémoire, et meurent avec l'onglet.
    const cles = Object.keys(localStorage).filter(cle => cle.startsWith('kaya.'))
    expect(cles).toEqual(['kaya.auth.rafraichissement'])

    const contenu = localStorage.getItem('kaya.auth.rafraichissement') ?? ''
    expect(contenu).not.toContain('cpt.compte.lire')
    expect(contenu).not.toContain('compte-1')
    expect(contenu).not.toContain('tenant-1')
  })

  it('le stockage web DÉCLARE ce qu’il ne garantit pas', async () => {
    // La garantie est portée par le TYPE, pas par un commentaire : un appelant qui exigerait un
    // coffre matériel doit pouvoir la lire et refuser. Sur le web, elle vaut `'aucune'`.
    const { adaptateurCourant } = await import('../core/platform/courant')

    expect(adaptateurCourant().stockageSecurise.garantie).toBe('aucune')
  })
})
