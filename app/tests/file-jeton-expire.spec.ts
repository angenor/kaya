// @vitest-environment happy-dom
/**
 * **L'ordre du retour de réseau** — research R-18, porte P-13, versant positif.
 *
 * # Ce que ce fichier existe pour attraper, et pourquoi rien d'autre ne l'attrape
 *
 * Une coupure de quatre-vingt-dix minutes est plus longue que le jeton d'accès, qui en dure
 * soixante. Au retour, une file qui **enverrait avant de rafraîchir** partirait avec un jeton
 * expiré : chaque élément reviendrait en `401`, et le service d'Aminata serait perdu.
 *
 * **En développement, les deux ordres passent.** La coupure y dure trente secondes, le jeton est
 * encore valide au retour, et rien ne distingue le code juste du code faux. C'est exactement le
 * profil d'un défaut qu'aucune relecture ne voit et qu'aucun test naïf ne reproduit — d'où ce
 * fichier, dont l'assertion centrale porte sur **l'ordre des appels**, pas sur leur succès.
 *
 * Les quatre autres cas vérifiés sont les conséquences de conception de R-18 : la file ne stocke
 * aucun jeton, elle accepte des écritures **sans session**, l'échec du rafraîchissement **ne la
 * vide pas**, et une opération qui n'est pas de classe A n'y entre jamais.
 */

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

import {
  effacerSession,
  oublierRafraichissement,
  rangerRafraichissement,
  sessionCourante,
} from '../core/auth'
import { FileLocale, marquerClasseA, viderFile, type EntreeFile } from '../core/sync'
import { CONTEXTE_TEST } from './commun/classes'
import { uuidV7 } from '../core/sync/uuid-v7'

const BASE = 'http://localhost:8080'

/** Durée de la coupure simulée — **une fois et demie la durée du jeton d'accès**. */
const COUPURE_MINUTES = 90
const JETON_ACCES_MINUTES = 60

const fetchOriginal = globalThis.fetch

/**
 * Le journal chronologique des appels — **c'est lui qui porte l'assertion du fichier**.
 *
 * Un seul tableau pour le rafraîchissement et les envois : deux compteurs séparés diraient
 * combien de fois chacun a eu lieu, jamais dans quel ordre.
 */
let journal: string[] = []

function corpsSession(rafraichissement: string) {
  return {
    acces: 'acces-frais',
    rafraichissement,
    expire_dans_s: JETON_ACCES_MINUTES * 60,
    permissions: ['etb.service.basculer'],
    etablissements: ['etb-1'],
    compte: { compte_id: 'compte-1', tenant_id: 'tenant-1', etablissement_actif: 'etb-1' },
  }
}

/** Faux serveur de rafraîchissement qui inscrit son passage au journal. */
function serveurRafraichissement(statut: number, corps: unknown) {
  globalThis.fetch = (async () => {
    journal.push('rafraichissement')
    return new Response(JSON.stringify(corps), {
      status: statut,
      headers: { 'content-type': 'application/json' },
    })
  }) as typeof fetch
}

/**
 * Une entrée acquittée par le serveur — `201`, ou `200` sur un rejeu.
 *
 * **Depuis le cycle 005, un envoyeur rend une ISSUE et non un booléen.** Un booléen ne distinguait
 * pas « le réseau n'a pas porté l'appel » de « le serveur a refusé définitivement », et les deux
 * exigent des traitements opposés : réessayer indéfiniment pour le premier, jamais pour le second.
 */
const ACQUITTE = { acquittee: true, statut: 201, code: '' } as const

/** Le réseau n'a pas porté l'appel. `statut: null` — **rien n'a été décidé côté serveur**. */
const RESEAU_COUPE = { acquittee: false, statut: null, code: 'reseau' } as const

/** Trois écritures de classe A, telles qu'une serveuse les produit pendant la coupure. */
function troisNotes(): EntreeFile[] {
  return [1, 2, 3].map(rang => ({
    id: uuidV7(),
    type: 'note_etablissement.creee',
    // Horodatage du terminal — **indicatif**. Il ordonne l'affichage local, jamais une règle.
    horodatageClient: new Date(Date.now() - (COUPURE_MINUTES - rang) * 60_000).toISOString(),
    charge: marquerClasseA(
      { texte: `commande ${rang}` },
      'A4 — append-only, sans effet monétaire, rejeu inoffensif',
    ),
    // Le contexte est FIGÉ à la saisie : changer d'établissement actif pendant la coupure ne
    // réattribue jamais une écriture déjà enfilée (cycle 005).
    contexte: CONTEXTE_TEST,
    tentatives: 0,
  }))
}

function fileGarnie(): FileLocale {
  const file = new FileLocale()
  troisNotes().forEach(entree => file.enfiler(entree))
  return file
}

beforeEach(async () => {
  journal = []
  effacerSession()
  await oublierRafraichissement()
})

afterEach(() => {
  globalThis.fetch = fetchOriginal
  vi.restoreAllMocks()
})

describe('la file accepte des écritures SANS jeton', () => {
  it('trois notes entrent en file alors qu’aucune session n’existe', () => {
    // Aucune session : `sessionCourante()` est nul, et pourtant l'enfilement passe. Exiger un
    // jeton à la mise en file voudrait dire qu'aucune commande ne part pendant la coupure.
    expect(sessionCourante()).toBeNull()

    const file = fileGarnie()

    expect(file.enAttente).toBe(3)
  })

  it('aucune entrée de la file ne porte de jeton', () => {
    const file = fileGarnie()

    // Le type n'a pas de champ où en mettre un ; l'assertion constate qu'aucun n'y a été glissé
    // par une charge utile. Un jeton mis en file serait périmé au retour, et le ranger
    // prolongerait la durée de vie d'un secret sur un terminal qu'on peut perdre.
    const serialise = JSON.stringify(file.lister())
    expect(serialise).not.toMatch(/acces|Bearer|rafraichissement|jeton/i)
  })
})

describe('R-18 — au retour du réseau, rafraîchir précède le premier envoi', () => {
  it('après une coupure de 90 minutes, le rafraîchissement est le PREMIER appel', async () => {
    await rangerRafraichissement('rafraichissement-avant-coupure')
    serveurRafraichissement(200, corpsSession('rafraichissement-apres-coupure'))
    const file = fileGarnie()

    const resultat = await viderFile(file, BASE, 'connecte', async (entree) => {
      journal.push(`envoi:${entree.id}`)
      return ACQUITTE
    })

    expect(resultat).toEqual({ issue: 'videe', envoyees: 3 })

    // **L'assertion du fichier.** Elle échoue si l'ordre s'inverse — y compris quand les deux
    // réussissent, ce qui est précisément le cas de développement où le défaut est invisible.
    expect(journal[0]).toBe('rafraichissement')
    expect(journal.filter(ligne => ligne === 'rafraichissement')).toHaveLength(1)
    expect(journal).toHaveLength(4)

    const premierEnvoi = journal.findIndex(ligne => ligne.startsWith('envoi:'))
    expect(premierEnvoi).toBeGreaterThan(journal.indexOf('rafraichissement'))
  })

  it('la coupure simulée est bien plus longue que le jeton — sans quoi le test ne prouverait rien', () => {
    // Écrit comme une assertion et non comme un commentaire : quelqu'un qui ramènerait la coupure
    // à trente secondes « pour accélérer le test » retomberait sur le cas où les deux ordres
    // passent, et le fichier cesserait silencieusement de vérifier quoi que ce soit.
    expect(COUPURE_MINUTES).toBeGreaterThan(JETON_ACCES_MINUTES)
  })

  it('une file vide ne consomme aucun jeton de rafraîchissement', async () => {
    await rangerRafraichissement('rafraichissement-1')
    serveurRafraichissement(200, corpsSession('rafraichissement-2'))

    const resultat = await viderFile(new FileLocale(), BASE, 'connecte', async () => {
      journal.push('envoi')
      return ACQUITTE
    })

    expect(resultat).toEqual({ issue: 'rien_a_faire' })
    // Rafraîchir pour n'envoyer rien ferait tourner la famille de jetons à chaque hoquet du réseau.
    expect(journal).toHaveLength(0)
  })
})

describe('l’échec du rafraîchissement NE VIDE PAS la file', () => {
  it('un 401 laisse les trois écritures intactes', async () => {
    await rangerRafraichissement('rafraichissement-mort')
    serveurRafraichissement(401, { code: 'session_invalide', message: 'x' })
    const file = fileGarnie()

    const resultat = await viderFile(file, BASE, 'connecte', async () => {
      journal.push('envoi')
      return ACQUITTE
    })

    expect(resultat).toMatchObject({ issue: 'reconnexion_requise', restantes: 3 })
    // Vider sur un `401` détruirait exactement les écritures qu'on cherche à sauver.
    expect(file.enAttente).toBe(3)
    expect(journal.filter(ligne => ligne === 'envoi')).toHaveLength(0)
  })

  it('hors ligne, la file est intacte et rien n’est même tenté', async () => {
    const file = fileGarnie()

    const resultat = await viderFile(file, BASE, 'hors_ligne', async () => {
      journal.push('envoi')
      return ACQUITTE
    })

    expect(resultat).toEqual({ issue: 'hors_ligne', restantes: 3 })
    expect(file.enAttente).toBe(3)
    expect(journal).toHaveLength(0)
  })

  it('un envoi refusé arrête le vidage et laisse le reste en file', async () => {
    await rangerRafraichissement('rafraichissement-1')
    serveurRafraichissement(200, corpsSession('rafraichissement-2'))
    const file = fileGarnie()

    let envois = 0
    const resultat = await viderFile(file, BASE, 'connecte', async () => {
      envois += 1
      // Le second envoi tombe sur un réseau injoignable — `statut: null`. Ce n'est PAS un refus
      // du serveur : rien n'a été décidé, donc on réessaiera, et l'écriture reste en file.
      return envois === 1 ? ACQUITTE : RESEAU_COUPE
    })

    expect(resultat).toEqual({ issue: 'partielle', envoyees: 1, restantes: 2 })
    expect(file.enAttente).toBe(2)
  })
})

describe('porte P-13 — la file reste fermée aux classes B, C et D', () => {
  it('une opération de ce cycle est refusée à l’enfilement', () => {
    const file = new FileLocale()

    expect(() =>
      file.enfiler({
        id: uuidV7(),
        // Attribution de rôle — **classe C**. Le registre l'interdit hors ligne, et le type
        // d'opération n'est pas déclaré dans `TYPES_CLASSE_A`.
        type: 'compte_role.attribue',
        horodatageClient: new Date().toISOString(),
        charge: marquerClasseA({ roleCode: 'caissier' }, 'marque abusive — le test la provoque'),
        contexte: CONTEXTE_TEST,
        tentatives: 0,
      }),
    ).toThrow(/n'est pas déclarée de classe A/)
  })
})
