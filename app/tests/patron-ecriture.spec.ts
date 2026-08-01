/**
 * **Le patron d'écriture front, côté logique** — ETB-02.
 *
 * `ecran-g1.spec.ts` vérifie ce que l'écran *montre* ; ce fichier vérifie ce que la couche d'appel
 * *décide*. Les deux sont nécessaires et ne se remplacent pas : une garde hors-ligne correcte dans
 * la fonction ne sert à rien si le composant l'ignore, et un bouton correctement caché ne protège
 * pas le second appelant qui viendra.
 *
 * # Ce qui est vérifié ici
 *
 * 1. **Classe C** — hors ligne et en réseau dégradé, aucun appel ne part, et le refus est immédiat.
 * 2. **La traduction des refus part du `code`**, jamais du `message` de diagnostic du serveur.
 * 3. **`motif_cle` prime** quand le référentiel en fournit une — elle enseigne là où le code
 *    constate.
 * 4. **L'identifiant est un UUID v7**, et non un v4 : c'est lui qui rend le rejeu inoffensif et qui
 *    porte l'ordre temporel dont dépend la pagination du repository.
 */

import { afterEach, describe, expect, it, vi } from 'vitest'

import { basculerService, PERMISSION_BASCULER, TYPE_OPERATION } from '../modules/etablissements/bascule-service'
import { estTypeClasseA } from '../core/sync/classes'
import { uuidV7 } from '../core/sync/uuid-v7'

/** Contexte d'appel — le jeton d'accès depuis CPT-01, plus aucun en-tête de tenant. */
const CONTEXTE = { baseUrl: 'http://localhost:8080', acces: 'jeton-de-test' }

const fetchOriginal = globalThis.fetch

afterEach(() => {
  globalThis.fetch = fetchOriginal
  vi.restoreAllMocks()
})

/**
 * Remplace `fetch` par une réponse figée, et retient ce qui a été envoyé.
 *
 * `openapi-fetch` appelle `fetch` avec un objet **`Request` construit**, pas avec une URL et des
 * options. Lire `String(entree)` donnerait `[object Request]` — la faute que ce faux serveur a
 * réellement commise à sa rédaction, et qui aurait fait passer une assertion vide.
 */
function fauxServeur(statut: number, corps: unknown) {
  const appels: { url: string, corps: unknown }[] = []

  globalThis.fetch = (async (entree: string | URL | Request, options?: RequestInit) => {
    const requete = entree instanceof Request ? entree : null
    const brut = requete ? await requete.clone().text() : options?.body

    appels.push({
      url: requete ? requete.url : String(entree),
      corps: brut ? JSON.parse(String(brut)) : undefined,
    })

    return new Response(JSON.stringify(corps), {
      status: statut,
      headers: { 'content-type': 'application/json' },
    })
  }) as typeof fetch

  return appels
}

describe('Patron d’écriture — la garde de classe C', () => {
  it('hors ligne : aucun appel ne part, et le refus est immédiat', async () => {
    const appels = fauxServeur(200, {})

    const resultat = await basculerService(CONTEXTE, 'etb-1', 'BAR', true, 'hors_ligne')

    expect(resultat).toEqual({
      issue: 'refus',
      cle: 'etablissement.services.refus.reseau',
      reseau: true,
    })
    // **Jamais de mise en file « au cas où », jamais d'échec après coup** (principe VI) : le refus
    // précède l'appel, il ne le suit pas.
    expect(appels, "aucune requête ne doit partir").toHaveLength(0)
  })

  it('réseau dégradé : même refus — « dégradé » n’est pas « connecté »', async () => {
    const appels = fauxServeur(200, {})

    const resultat = await basculerService(CONTEXTE, 'etb-1', 'BAR', true, 'degrade')

    expect(resultat).toMatchObject({ issue: 'refus', reseau: true })
    expect(appels).toHaveLength(0)
  })

  it('l’opération n’est PAS déclarée de classe A — elle n’entre jamais en file', () => {
    // L'y déclarer autoriserait sa mise en file, ce que la porte P-13 refuse : `etablissement_module`
    // est de classe C au registre (`docs/registre-classes-offline.md` §5.1).
    expect(estTypeClasseA(TYPE_OPERATION)).toBe(false)
  })
})

describe('Patron d’écriture — l’appel et ses refus', () => {
  it('succès : l’appel porte un UUID v7 client et le sens demandé', async () => {
    const appels = fauxServeur(201, { module_code: 'BAR', actif: true })

    const resultat = await basculerService(CONTEXTE, 'etb-1', 'BAR', true, 'connecte')

    expect(resultat).toEqual({ issue: 'succes' })
    expect(appels).toHaveLength(1)
    expect(appels[0]!.url).toContain('/api/v1/etablissements/etb-1/services/BAR')

    const corps = appels[0]!.corps as { id: string, actif: boolean }
    expect(corps.actif).toBe(true)
    // Version 7 — le treizième caractère hexadécimal. `crypto.randomUUID()` produirait un `4`.
    expect(corps.id[14], 'l’identifiant doit être un UUID v7, pas un v4').toBe('7')
  })

  it('refus métier : le message vient du CODE, jamais du texte du serveur', async () => {
    fauxServeur(422, {
      code: 'desactivation_bloquee',
      // Ce diagnostic ne doit atteindre l'écran sous aucune forme : il est en anglais technique
      // et nomme des tables.
      message: 'cannot deactivate: 2 open stays in etablissements.sejour',
      obstacles: [{ module_code: 'HEBERGEMENT', motif_cle: 'services.obstacle.sejours_en_cours', nombre: 2 }],
    })

    const resultat = await basculerService(CONTEXTE, 'etb-1', 'HEBERGEMENT', false, 'connecte')

    expect(resultat).toMatchObject({
      issue: 'refus',
      cle: 'etablissement.services.refus.desactivation_bloquee',
    })
    expect(JSON.stringify(resultat), 'le diagnostic serveur ne franchit pas la frontière')
      .not.toContain('etablissements.sejour')

    if (resultat.issue !== 'refus') throw new Error('refus attendu')
    expect(resultat.obstacles).toHaveLength(1)
    expect(resultat.obstacles![0]!.nombre).toBe(2)
  })

  it('`motif_cle` prime sur le code — elle enseigne là où le code constate', async () => {
    fauxServeur(422, {
      code: 'module_non_implemente',
      message: 'not implemented',
      motif_cle: 'services.refus.profil.VALORISE',
    })

    const resultat = await basculerService(CONTEXTE, 'etb-1', 'SPA', true, 'connecte')

    expect(resultat).toMatchObject({ issue: 'refus', cle: 'services.refus.profil.VALORISE' })
  })

  it('code inconnu : une phrase honnête, pas une clé i18n affichée en brut', async () => {
    fauxServeur(500, { code: 'un_code_que_le_client_ne_connait_pas', message: 'boom' })

    const resultat = await basculerService(CONTEXTE, 'etb-1', 'BAR', true, 'connecte')

    expect(resultat).toMatchObject({ issue: 'refus', cle: 'etablissement.services.refus.inattendue' })
  })

  it('403 : un message existe, même si l’action ne devrait jamais être atteignable', async () => {
    // L'utilisateur ne devrait pas y arriver — l'action lui est ABSENTE (principe VII). Le seul
    // chemin qui y mène est un changement de droits pendant qu'il regarde l'écran.
    fauxServeur(403, {})

    const resultat = await basculerService(CONTEXTE, 'etb-1', 'BAR', true, 'connecte')

    expect(resultat).toMatchObject({ issue: 'refus', cle: 'etablissement.services.refus.permission' })
  })

  it('la permission suit la convention de CPT-02', () => {
    expect(PERMISSION_BASCULER).toBe('etb.service.basculer')
  })
})

describe('UUID v7', () => {
  it('porte la version 7 et la variante RFC 4122', () => {
    for (let i = 0; i < 200; i += 1) {
      const uuid = uuidV7()
      expect(uuid).toMatch(/^[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/)
    }
  })

  it('est croissant dans le temps — c’est tout l’intérêt du v7 sur le v4', async () => {
    const premier = uuidV7()
    await new Promise(resoudre => setTimeout(resoudre, 3))
    const second = uuidV7()

    // Les 48 premiers bits sont l'horodatage : la comparaison lexicographique des douze premiers
    // caractères hexadécimaux suffit. C'est cette propriété dont dépendent l'ordre secondaire
    // `ORDER BY cree_le DESC, id DESC` du repository et la localité d'insertion des index.
    expect(second.replace(/-/g, '').slice(0, 12) > premier.replace(/-/g, '').slice(0, 12)).toBe(true)
  })

  it('ne se répète pas', () => {
    const tires = new Set(Array.from({ length: 5000 }, () => uuidV7()))
    expect(tires.size).toBe(5000)
  })
})
