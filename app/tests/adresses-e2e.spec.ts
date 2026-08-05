// @vitest-environment node
/**
 * **AUCUNE ADRESSE DE SERVEUR EN DUR DANS `tests-e2e/`** — un vert rendu sur le produit du voisin.
 *
 * ═══════════════════════════════════════════════════════════════════════════════════════════
 *  CE QUI EST ARRIVÉ, ET CE QUE ÇA COÛTE
 * ═══════════════════════════════════════════════════════════════════════════════════════════
 *
 * `playwright.config.ts` honore `KAYA_PORT_E2E` depuis le cycle 004, et il dit pourquoi :
 *
 * > un serveur Nuxt d'un **autre projet** occupait déjà le 3000.
 *
 * Trois fichiers de portes écrivaient pourtant `baseURL: 'http://localhost:3000'` en dur dans leur
 * `browser.newContext()`. Sur un poste où le 3000 est pris, Nuxt sert sur 3001 **et les portes
 * interrogent le serveur du voisin** — rendant un verdict sur une application qui n'est pas celle
 * du dépôt. Un vert obtenu sur le produit de quelqu'un d'autre est le pire vert possible : il ne
 * casse rien, donc il dure.
 *
 * C'est le pendant silencieux du `pkill -f "nuxt.mjs dev"` qui a tué le serveur d'un autre projet
 * de ce poste. L'un détruit le travail du voisin, l'autre le prend pour le nôtre.
 *
 * ═══════════════════════════════════════════════════════════════════════════════════════════
 *  ⚠️ POURQUOI UN CONTRÔLE, ET PAS TROIS CORRECTIONS
 * ═══════════════════════════════════════════════════════════════════════════════════════════
 *
 * Le défaut a été **diagnostiqué, expliqué par écrit, corrigé sur une occurrence — puis reproduit
 * le jour même dans un fichier neuf du même lot.** Ce n'est pas un défaut d'attention :
 * `browser.newContext({ baseURL })` est un endroit où l'on écrit une adresse sans y penser, et
 * rien ne le gardait. Trois fichiers, trois occasions de se tromper, aucun contrôle.
 *
 * Corriger les instances laisse la classe ouverte. Ce fichier la ferme : toute adresse de serveur
 * littérale dans `tests-e2e/` échoue, en nommant le fichier, la ligne et le remplacement.
 *
 * Il tourne dans le job `app`, **sans base, sans API, sans navigateur** — contrairement aux portes
 * qu'il garde. Dans ce dépôt, une porte qui n'a besoin de rien s'exécute ; une porte qui a besoin
 * de services ne s'exécute pas. C'est le niveau qui compte.
 */

import { readFileSync, readdirSync } from 'node:fs'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'

const ICI = dirname(fileURLToPath(import.meta.url))
const E2E = resolve(ICI, '..', '..', 'tests-e2e')

/**
 * Le seul fichier autorisé à porter les adresses — **celui qui les DÉFINIT**.
 *
 * Même régime que `assets/css/theme.css` pour P-17 : l'exemption porte sur la source, elle est
 * nommée, et sa contrepartie est vérifiée plus bas — le fichier doit réellement exporter ce que
 * les autres consomment, sinon l'exemption garderait un fichier vide.
 */
const SOURCE = 'adresses.ts'

/**
 * Une adresse de serveur écrite en clair.
 *
 * Le motif vise `localhost` et les adresses de bouclage numériques : ce sont les trois formes qui
 * apparaissent réellement dans un `baseURL` ou un `fetch` de test. Un nom d'hôte quelconque n'est
 * pas visé — il n'y en a aucun dans ce dépôt, et le viser produirait des faux positifs sur les
 * URL de registres citées dans la prose.
 */
const ADRESSE_EN_DUR = /(?:localhost|127\.0\.0\.1|0\.0\.0\.0):\d+/g

/**
 * Retire les commentaires **en conservant les sauts de ligne**.
 *
 * ⚠️ La conservation n'est pas cosmétique : c'est le défaut que P-17 portait — `sansCommentaires()`
 * y remplaçait chaque bloc par une chaîne vide, puis comptait les lignes sur le texte raccourci.
 * Un fichier bien commenté voyait donc sa ligne signalée d'autant plus fausse qu'il était
 * conforme. Un défaut qui s'aggrave avec le respect de la règle ne se corrige jamais par plus de
 * discipline.
 *
 * Et le retrait lui-même est nécessaire : `hors-ligne.spec.ts` **explique** dans sa prose que Nuxt
 * sert sur `localhost:3000`. Citer une adresse n'est pas s'y connecter.
 */
function sansCommentaires(source: string): string {
  const vider = (bloc: string): string => '\n'.repeat((bloc.match(/\n/g) ?? []).length)
  return source
    .replace(/\/\*[\s\S]*?\*\//g, vider)
    .replace(/(^|[^:])\/\/.*$/gm, '$1')
}

const FICHIERS = readdirSync(E2E)
  .filter(nom => nom.endsWith('.ts'))
  .sort()

describe('la cible n’est pas vide — exigence 4', () => {
  it('des fichiers de portes sont bien lus', () => {
    // Un glob devenu faux — répertoire renommé, extension changée — rendrait zéro fichier, et le
    // contrôle passerait au vert en n'inspectant rien.
    expect(FICHIERS.length).toBeGreaterThanOrEqual(4)
    expect(FICHIERS).toContain(SOURCE)
  })

  it('la source exporte réellement les adresses que les autres consomment', () => {
    // La contrepartie de l'exemption. Sans elle, vider `adresses.ts` ferait passer ce fichier au
    // vert tout en obligeant les portes à réécrire leurs adresses en dur.
    const source = readFileSync(join(E2E, SOURCE), 'utf8')

    for (const exporte of ['BASE_APP', 'BASE_API', 'PORT_APP']) {
      expect(source, `« ${exporte} » n’est plus exporté par ${SOURCE}`)
        .toMatch(new RegExp(`export const ${exporte}\\b`))
    }
    // Et elles sont dérivées de l'environnement, pas figées.
    expect(source).toContain('KAYA_PORT_E2E')
    expect(source).toContain('KAYA_API_BASE_URL')
  })
})

describe('aucune adresse de serveur en dur', () => {
  it.each(FICHIERS.filter(nom => nom !== SOURCE))('%s', (nom) => {
    const source = sansCommentaires(readFileSync(join(E2E, nom), 'utf8'))
    const trouvees: string[] = []

    for (const capture of source.matchAll(ADRESSE_EN_DUR)) {
      const ligne = source.slice(0, capture.index ?? 0).split('\n').length
      trouvees.push(`tests-e2e/${nom}:${ligne} — « ${capture[0]} »`)
    }

    expect(
      trouvees,
      'Une adresse de serveur est écrite en dur dans une porte.\n'
      + 'Sur un poste où le port est pris par un AUTRE projet — le cas qui a motivé '
      + '`KAYA_PORT_E2E` au cycle 004 —, Playwright sert ailleurs et la porte interroge le serveur '
      + 'du voisin. Le verdict porte alors sur une application qui n’est pas la nôtre.\n'
      + 'Remplacer par `BASE_APP`, `BASE_API` ou `PORT_APP` de `tests-e2e/adresses.ts`.',
    ).toEqual([])
  })
})

describe('les portes emploient bien la source', () => {
  it('tout fichier qui ouvre un contexte de navigateur lit `adresses.ts`', () => {
    // Le versant positif. Sans lui, un fichier pourrait éviter le contrôle ci-dessus en
    // construisant son adresse autrement — concaténation, variable locale — et personne ne le
    // verrait. Ce qui est vérifié ici : celui qui ouvre un contexte a bien importé la source.
    const manquants: string[] = []

    for (const nom of FICHIERS.filter(n => n !== SOURCE)) {
      const source = readFileSync(join(E2E, nom), 'utf8')
      if (!source.includes('newContext(')) continue
      if (!source.includes("from './adresses'")) {
        manquants.push(`tests-e2e/${nom}`)
      }
    }

    expect(
      manquants,
      'Ces fichiers ouvrent un contexte de navigateur sans lire `tests-e2e/adresses.ts`.',
    ).toEqual([])
  })
})
