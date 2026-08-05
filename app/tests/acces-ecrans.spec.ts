// @vitest-environment node
/**
 * **CHAQUE ÉCRAN GARDÉ POSE-T-IL VRAIMENT SA GARDE ?** — FR-029, versant branchement.
 *
 * ═══════════════════════════════════════════════════════════════════════════════════════════
 *  CE QUI EST ARRIVÉ
 * ═══════════════════════════════════════════════════════════════════════════════════════════
 *
 * Six écrans — `/hebergement`, `/chambres`, `/passage`, `/arrivee`, `/clients`, `/depart` — ne
 * gardaient que leurs **gestes** : `peutGerer`, `peutOuvrir`, `peutClore`, tous posés **dans le
 * composant**, sur le bouton. Seuls `/comptes` et `/journal-audit` gardaient la **lecture**.
 *
 * Aucune donnée ne fuyait — le serveur refuse en `403` et les six écrans ne montraient rien. Le
 * défaut était de **langue** : sur URL directe, ils affichaient « Les chambres n'ont pas pu être
 * chargées. », un message d'échec **technique**. Une réceptionniste qui le lit appelle le support
 * pour un problème de réseau qui n'existe pas. `/journal-audit` disait, lui, « Vous n'avez pas
 * accès au registre des actions. » — qui dit la vraie cause, et à qui s'adresser.
 *
 * ═══════════════════════════════════════════════════════════════════════════════════════════
 *  POURQUOI UN CONTRÔLE STATIQUE, ET PAS UN MONTAGE NI UN CAS E2E
 * ═══════════════════════════════════════════════════════════════════════════════════════════
 *
 * **Aucun compte des seeds ne permet d'exercer ce refus en bout en bout.** Les quatre rôles du jeu
 * de démonstration portent tous `heb.offre.lire`, `heb.sejour.lire` et `sej.client.lire` — c'est
 * exact et voulu : ce sont les lectures du métier. Un cas e2e devrait forger une session amputée,
 * c'est-à-dire prouver le produit contre un jeton que le produit n'émet pas.
 *
 * Reste la question qui compte vraiment, et elle est statique : **la garde est-elle branchée ?**
 * `core/acces/ecrans.ts` peut être parfait et n'être appelé par personne — c'est exactement ce qui
 * est arrivé à `initialiserTheme()`, exportée et documentée « à appeler au démarrage » pendant
 * deux cycles, appelée nulle part. Une unité écrite n'est ni testée ni branchée par défaut.
 *
 * ═══════════════════════════════════════════════════════════════════════════════════════════
 *  LE PÉRIMÈTRE EST DÉCOUVERT — la règle du cycle 005
 * ═══════════════════════════════════════════════════════════════════════════════════════════
 *
 * Les écrans à vérifier ne sont pas énumérés ici : ce sont **ceux que `ACCES_ECRANS` déclare
 * gardés**. Le cycle 007 déclarera `/caisse` avec sa permission, et ce fichier exigera sa garde
 * sans que personne ait eu à y penser. Un écran déclaré sans permission — `/`, `/connexion`,
 * `/styleguide`, `/mes-envois` — n'a rien à garder, et il est vérifié qu'il **ne** pose **pas**
 * de garde inutile.
 */

import { readFileSync } from 'node:fs'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'

import { ACCES_ECRANS } from '../core/acces/ecrans'

const ICI = dirname(fileURLToPath(import.meta.url))
const PAGES = resolve(ICI, '..', 'pages')

/** Le fichier de page d'une route — l'inverse de la convention de Nuxt. */
function fichierDeLaRoute(route: string): string {
  return join(PAGES, `${route === '/' ? 'index' : route.slice(1)}.vue`)
}

const GARDES = Object.entries(ACCES_ECRANS).filter(([, acces]) => acces.permissions.length > 0)
const LIBRES = Object.entries(ACCES_ECRANS).filter(([, acces]) => acces.permissions.length === 0)

describe('la cible n’est pas vide — exigence 4', () => {
  it('des écrans sont déclarés gardés, et pas zéro', () => {
    // Si `ACCES_ECRANS` se vidait — ou si le filtre cessait de fonctionner —, la boucle
    // ci-dessous n'inspecterait rien et le fichier passerait au vert en ne vérifiant rien.
    expect(GARDES.length).toBeGreaterThanOrEqual(8)
    expect(LIBRES.length).toBeGreaterThanOrEqual(3)
  })
})

describe('tout écran gardé BRANCHE sa garde, et refuse en langue utilisateur', () => {
  it.each(GARDES.map(([route]) => route))('%s appelle peutOuvrirEcran sur SA route', (route) => {
    const source = readFileSync(fichierDeLaRoute(route), 'utf8')

    // La route est citée en toutes lettres dans l'appel : un copier-coller depuis la page voisine
    // garderait le mauvais écran — `/arrivee` protégé par les permissions de `/depart` — et
    // passerait tout contrôle qui se contenterait de chercher le nom de la fonction.
    expect(
      source,
      `« ${route} » ne branche pas sa garde. `
      + '`core/acces/ecrans.ts` peut être parfait et n’être appelé par personne : c’est ce qui est '
      + 'arrivé à `initialiserTheme()`, exportée et documentée pendant deux cycles, appelée nulle '
      + 'part.',
    ).toContain(`peutOuvrirEcran('${route}'`)
  })

  it.each(GARDES.map(([route]) => route))('%s dit le refus, il ne le laisse pas tomber en panne', (route) => {
    const source = readFileSync(fichierDeLaRoute(route), 'utf8')

    // ⚠️ C'est LE point du lot. Un écran qui refuse sans phrase affiche son message d'échec de
    // chargement — « n’ont pas pu être chargées » —, qui décrit une panne là où il s’agit d’un
    // droit. Le lexique proscrit exactement cette confusion.
    expect(
      source,
      `« ${route} » refuse l’accès sans clé « acces_refuse » : l’utilisateur verra un message `
      + 'd’échec technique et cherchera un problème de réseau qui n’existe pas.',
    ).toMatch(/acces_refuse/)
  })
})

describe('un écran que rien ne garde ne pose PAS de garde — versant négatif', () => {
  it.each(LIBRES.map(([route]) => route))('%s n’appelle pas peutOuvrirEcran', (route) => {
    // Sans ce versant, les deux contrôles ci-dessus passeraient sur un produit qui garderait
    // TOUT, y compris `/connexion` — ce qui serait une boucle : il faut une session pour avoir
    // des permissions.
    const source = readFileSync(fichierDeLaRoute(route), 'utf8')

    expect(source, `« ${route} » n’a aucune permission déclarée et pose pourtant une garde.`)
      .not.toContain('peutOuvrirEcran(')
  })
})
