// @vitest-environment happy-dom
/**
 * **La file survit à l'extinction, et elle est illisible dans le stockage.**
 *
 * # Les deux propriétés, et pourquoi la seconde ne se déduit pas de la première
 *
 * FR-012 — la file survit au rechargement **et** à l'extinction. FR-013 — la charge est chiffrée
 * dès le premier octet.
 *
 * Une file qui persiste sans chiffrer satisferait la première et violerait la seconde, **et rien
 * ne le dirait** : le scénario de recette passerait, les quatre notes seraient là après le
 * rechargement, et les données d'identité que l'OCR de classe A produira au cycle suivant
 * resteraient en clair dans `localStorage`. Les deux propriétés sont donc vérifiées séparément.
 *
 * # Ce que ce fichier NE prouve pas, et qui est écrit plutôt que supposé
 *
 * - **Que le web soit sûr.** La clé vit dans un stockage dont la garantie est `aucune`, et le type
 *   le dit. Ce que le chiffrement achète est qu'un accès au **stockage seul** — sauvegarde de
 *   navigateur, extension, inspection du disque — ne rende rien de lisible. Un script de la même
 *   origine, lui, peut lire la clé, et le produit ne prétend pas le contraire.
 * - **Que `crypto.subtle` fonctionne sur WKWebView.** Il exige un contexte sécurisé, et le WebKit
 *   de Playwright n'est pas WKWebView. Le contrôle réel viendra avec la coquille Tauri.
 */

import { afterEach, beforeEach, describe, expect, it } from 'vitest'

import { CLES_PERSISTANCE, FileLocale, ouvrirMagasin } from '../core/sync'
import { adaptateurCourant } from '../core/platform/courant'

import { entreeDeTest } from './commun/classes'

/** Les clés `kaya.` présentes — lues à la main, jamais par l'adaptateur. */
function clesKaya(): string[] {
  return Object.keys(localStorage).filter(cle => cle.startsWith('kaya.'))
}

function cryptogramme(): string | null {
  return localStorage.getItem(`kaya.${CLES_PERSISTANCE.cryptogramme}`)
}

beforeEach(() => {
  localStorage.clear()
})

afterEach(() => {
  localStorage.clear()
})

describe('la file survit à l’extinction', () => {
  it('quatre écritures rangées se retrouvent après une RÉOUVERTURE complète', async () => {
    // Le scénario A du quickstart, en un test : quatre notes hors ligne, puis on éteint.
    const file = await FileLocale.ouvrir(adaptateurCourant())
    for (let rang = 1; rang <= 4; rang += 1) {
      file.enfiler(entreeDeTest({ texte: `commande ${rang}` }))
    }
    expect(file.enAttente).toBe(4)

    // La persistance part sans être attendue — une saisie ne doit pas attendre le disque. On la
    // laisse aboutir avant de rouvrir, ce que le rechargement d'une page fait naturellement.
    await new Promise(resoudre => setTimeout(resoudre, 0))

    // **Réouverture complète** : une instance neuve, comme après une extinction. C'est le cas
    // fréquent — le terminal de comptoir s'éteint le soir — et c'est celui qu'on manque.
    const rouverte = await FileLocale.ouvrir(adaptateurCourant())

    expect(
      rouverte.enAttente,
      'les écritures n’ont pas survécu à la réouverture : la serveuse apprendrait au service '
      + 'suivant que ses quatre commandes n’existent pas',
    ).toBe(4)

    const textes = rouverte.lister().map(e => (e.charge as unknown as { texte: string }).texte)
    expect(textes).toEqual(['commande 1', 'commande 2', 'commande 3', 'commande 4'])
  })

  it('le contexte figé à la saisie survit lui aussi', async () => {
    // Sans lui, les écritures partiraient sur l'établissement actif AU RETOUR du réseau — une
    // faute silencieuse, que rien ne signale et que personne ne peut démêler après coup.
    const file = await FileLocale.ouvrir(adaptateurCourant())
    file.enfiler(
      entreeDeTest({
        contexte: {
          tenantId: '018f0000-0000-7000-8000-00000000ffaa',
          etablissementId: '018f0000-0000-7000-8000-00000000ffbb',
        },
      }),
    )
    await new Promise(resoudre => setTimeout(resoudre, 0))

    const rouverte = await FileLocale.ouvrir(adaptateurCourant())
    expect(rouverte.lister()[0]?.contexte.etablissementId).toBe(
      '018f0000-0000-7000-8000-00000000ffbb',
    )
  })

  it('une file vidée ne laisse AUCUN cryptogramme derrière elle', async () => {
    const file = await FileLocale.ouvrir(adaptateurCourant())
    const entree = entreeDeTest()
    file.enfiler(entree)
    await new Promise(resoudre => setTimeout(resoudre, 0))
    expect(cryptogramme()).not.toBeNull()

    file.retirer(entree.id)
    await new Promise(resoudre => setTimeout(resoudre, 0))

    // Garder un bloc qui chiffre `[]` laisserait croire, à l'inspection du stockage, qu'il reste
    // quelque chose à envoyer.
    expect(cryptogramme()).toBeNull()
  })
})

describe('la charge est ILLISIBLE dans le stockage', () => {
  it('le texte d’une note n’apparaît nulle part en clair', async () => {
    const file = await FileLocale.ouvrir(adaptateurCourant())
    file.enfiler(entreeDeTest({ texte: 'Le groupe électrogène a démarré à 19 h 40.' }))
    await new Promise(resoudre => setTimeout(resoudre, 0))

    const contenuEntier = clesKaya()
      .map(cle => localStorage.getItem(cle) ?? '')
      .join('\n')

    expect(
      contenuEntier,
      'le texte d’une écriture apparaît EN CLAIR dans le stockage.\n'
      + 'FR-013 exige que la charge soit chiffrée dès le premier octet — le motif est daté : '
      + 'l’extraction OCR d’une pièce d’identité est de classe A, donc éligible à cette file.',
    ).not.toContain('groupe électrogène')

    expect(contenuEntier).not.toContain('Le groupe')
  })

  it('sans la CLÉ, le cryptogramme ne rend rien', async () => {
    const file = await FileLocale.ouvrir(adaptateurCourant())
    file.enfiler(entreeDeTest({ texte: 'saisie confidentielle' }))
    await new Promise(resoudre => setTimeout(resoudre, 0))

    const bloc = cryptogramme()
    expect(bloc, 'aucun cryptogramme rangé — le test ne prouverait rien').not.toBeNull()

    // On retire la clé du coffre, en laissant le cryptogramme. C'est exactement ce que voit
    // quelqu'un qui accède au stockage sans le coffre système.
    await adaptateurCourant().stockageSecurise.supprimer(CLES_PERSISTANCE.secret)

    const rouverte = await FileLocale.ouvrir(adaptateurCourant())

    expect(
      rouverte.enAttente,
      'la file s’est rouverte sans la clé : la charge était donc lisible sans elle',
    ).toBe(0)
  })

  it('un cryptogramme illisible fait repartir la file VIDE, sans bloquer le démarrage', async () => {
    // Refuser de démarrer bloquerait le terminal sur un état qu'aucun exploitant ne peut réparer,
    // et garder un bloc indéchiffrable ferait échouer chaque enregistrement suivant.
    localStorage.setItem(`kaya.${CLES_PERSISTANCE.cryptogramme}`, 'ceci-n-est-pas-du-base64-valide')

    const file = await FileLocale.ouvrir(adaptateurCourant())

    expect(file.enAttente).toBe(0)
    expect(cryptogramme(), 'le bloc illisible a été gardé : chaque écriture suivante échouerait')
      .toBeNull()
  })
})

describe('le magasin, pris isolément', () => {
  it('enregistre puis relit exactement ce qu’on lui a donné', async () => {
    const magasin = await ouvrirMagasin(adaptateurCourant())
    const entrees = [entreeDeTest({ texte: 'une' }), entreeDeTest({ texte: 'deux' })]

    await magasin.enregistrer(entrees)
    const relues = await magasin.charger()

    expect(relues).toHaveLength(2)
    expect(relues.map(e => (e.charge as unknown as { texte: string }).texte)).toEqual(['une', 'deux'])
  })

  it('la purge emporte le cryptogramme ET la clé', async () => {
    const magasin = await ouvrirMagasin(adaptateurCourant())
    await magasin.enregistrer([entreeDeTest()])
    expect(clesKaya().length).toBeGreaterThan(0)

    await magasin.purger()

    // Laisser la clé survivre à un changement de personne laisserait les écritures d'Aminata
    // déchiffrables sur le terminal de Yao.
    expect(clesKaya()).toEqual([])
  })

  it('deux enregistrements successifs produisent deux cryptogrammes DIFFÉRENTS', async () => {
    // Un IV réutilisé avec la même clé **casse** GCM — pas « affaiblit » : casse. La file est
    // réécrite à chaque saisie, donc le cas est fréquent, et deux blocs identiques sur le même
    // contenu diraient que l'IV ne change pas.
    const magasin = await ouvrirMagasin(adaptateurCourant())
    const memeContenu = [entreeDeTest({ id: '018f0000-0000-7000-8000-0000000000c1' })]

    await magasin.enregistrer(memeContenu)
    const premier = cryptogramme()
    await magasin.enregistrer(memeContenu)
    const second = cryptogramme()

    expect(premier).not.toBeNull()
    expect(second).not.toBe(premier)
  })
})
