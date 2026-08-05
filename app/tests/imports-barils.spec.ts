/**
 * ★ **Tout ce qu'on importe d'un baril doit y être VRAIMENT exporté.**
 *
 * ═══════════════════════════════════════════════════════════════════════════════════════════════
 *  CE FICHIER EST NÉ D'UN DÉFAUT RÉEL, TROUVÉ PAR P-22 AU CYCLE 006
 *
 *  `EcranPassage.vue` — l'écran du passage, celui dont le cadrage §5.6 fait une condition
 *  d'existence du produit — portait :
 *
 *      import { useEtatReseau } from '~/core/platform'
 *
 *  `~/core/platform/index.ts` **n'exporte pas** `useEtatReseau` : la fonction vit dans
 *  `core/platform/reseau.ts`, et les cinq écrans écrits avant l'importaient de là. Résultat en
 *  navigateur :
 *
 *      The requested module '/_nuxt/core/platform/index.ts'
 *      does not provide an export named 'useEtatReseau'
 *
 *  **L'écran ne se montait pas.** Aucun test unitaire ne l'a vu, et c'est structurel : les tests de
 *  composant doublent `~/core/platform` (`vi.mock`) et **fournissent** la fonction manquante. Le
 *  double rendait vrai ce que le baril rendait faux.
 *
 *  P-22 l'a attrapé — mais P-22 exige l'API, la base, les seeds et deux navigateurs. Ce contrôle-ci
 *  rend le même verdict en millisecondes, sur le seul texte des fichiers.
 * ═══════════════════════════════════════════════════════════════════════════════════════════════
 *
 * # Ce qu'il vérifie, et ce qu'il ne vérifie pas
 *
 * Il vérifie les imports **de valeur** — `import { x } from '~/core/platform'`. Les imports de
 * type (`import type { … }`) sont **hors sujet** : ils sont effacés à la compilation et ne peuvent
 * pas casser à l'exécution. C'est d'ailleurs ce qui rendait le défaut si peu visible en relecture :
 * dix fichiers importaient `EtatReseau` du même baril, en type, et personne ne les distinguait.
 *
 * Il suit **un seul niveau** de ré-export (`export * from './x'`) — `core/sync/index.ts` en emploie.
 * Au-delà, il **refuse bruyamment** plutôt que de conclure à l'absence d'un nom qui existe : une
 * lecture qui se trompe en silence est pire qu'une lecture qui s'arrête.
 */

import { readdirSync, readFileSync, statSync } from 'node:fs'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'

const ICI = dirname(fileURLToPath(import.meta.url))
const APP = resolve(ICI, '..')

/**
 * Les barils surveillés — un module `index.ts` dont d'autres fichiers importent des **valeurs**.
 *
 * La liste est courte et explicite : elle doit être lue, pas devinée. Un baril ajouté sans être
 * inscrit ici passerait sous le contrôle, et c'est le prix assumé d'une liste tenue à la main —
 * contrairement aux **cibles** (les fichiers scrutés), qui viennent du système de fichiers.
 */
const BARILS = ['~/core/platform', '~/core/sync', '~/core/auth', '~/core/rbac'] as const

/** Les répertoires scrutés — **lus du système de fichiers**, jamais énumérés. */
const RACINES = ['core', 'modules', 'pages', 'plugins', 'middleware', 'layouts']

const EXTENSIONS = ['.ts', '.vue']

function fichiers(repertoire: string): string[] {
  const chemin = join(APP, repertoire)
  let entrees: string[]
  try {
    entrees = readdirSync(chemin)
  }
  catch {
    return []
  }

  return entrees.flatMap((entree) => {
    const complet = join(chemin, entree)
    if (statSync(complet).isDirectory()) return fichiers(join(repertoire, entree))
    return EXTENSIONS.some(ext => entree.endsWith(ext)) ? [join(repertoire, entree)] : []
  })
}

/** Les noms exportés par un baril — `export function|const|class|type|interface`, et `export {}`. */
function exportsDe(baril: string): Set<string> {
  const relatif = baril.replace('~/', '')
  return nomsExportes(join(APP, relatif, 'index.ts'), baril, 0)
}

/** Profondeur maximale de ré-export suivie. Au-delà, le contrôle refuse plutôt que de deviner. */
const PROFONDEUR_MAX = 1

function nomsExportes(fichier: string, baril: string, profondeur: number): Set<string> {
  const source = readFileSync(fichier, 'utf8')

  const noms = new Set<string>()

  for (const capture of source.matchAll(
    /^export\s+(?:async\s+)?(?:function|const|let|class|type|interface|enum)\s+([A-Za-z_$][\w$]*)/gm,
  )) {
    noms.add(capture[1]!)
  }

  // `export { a, b as c }` — la forme rare, mais elle compte.
  for (const bloc of source.matchAll(/^export\s*\{([^}]*)\}/gm)) {
    for (const morceau of bloc[1]!.split(',')) {
      const nom = morceau.trim().split(/\s+as\s+/).pop()?.trim()
      if (nom) noms.add(nom)
    }
  }

  // ★ **Un ré-export global est SUIVI, sur un seul niveau.** L'ignorer rendrait la lecture fausse
  // en silence : elle conclurait à l'absence d'un nom qui existe, et le contrôle deviendrait un
  // générateur de faux positifs — donc un contrôle désactivé sous trois semaines.
  for (const capture of source.matchAll(/^export\s+\*\s+from\s*['"](\.[^'"]+)['"]/gm)) {
    if (profondeur >= PROFONDEUR_MAX) {
      throw new Error(
        `${baril} — ré-export imbriqué au-delà de ${PROFONDEUR_MAX} niveau(x) (« ${capture[1]} »). `
        + 'Étendre la lecture dans le MÊME changement que le ré-export : un contrôle qui devine se '
        + 'trompe en silence, et conclut à l’absence d’un nom qui existe.',
      )
    }
    for (const nom of nomsExportes(resoudre(fichier, capture[1]!), baril, profondeur + 1)) {
      noms.add(nom)
    }
  }

  return noms
}

/** Résout un chemin relatif de ré-export vers un fichier réel — `./x` → `x.ts` ou `x/index.ts`. */
function resoudre(depuis: string, relatif: string): string {
  const base = join(dirname(depuis), relatif)
  for (const candidat of [`${base}.ts`, join(base, 'index.ts')]) {
    try {
      statSync(candidat)
      return candidat
    }
    catch {
      // essai suivant
    }
  }
  throw new Error(`ré-export « ${relatif} » introuvable depuis ${depuis}`)
}

/** Les imports de **valeur** d'un baril dans un fichier. `import type` est ignoré. */
function importsDeValeur(source: string, baril: string): string[] {
  const motif = new RegExp(
    String.raw`import\s+(type\s+)?\{([^}]*)\}\s*from\s*['"]${baril}['"]`,
    'g',
  )

  const noms: string[] = []
  for (const capture of source.matchAll(motif)) {
    // `import type { … }` — effacé à la compilation, il ne peut pas casser à l'exécution.
    if (capture[1]) continue
    for (const morceau of capture[2]!.split(',')) {
      const brut = morceau.trim()
      if (!brut) continue
      // `type X` inline dans un import mixte : même raison, hors sujet.
      if (/^type\s/.test(brut)) continue
      noms.push(brut.split(/\s+as\s+/)[0]!.trim())
    }
  }
  return noms
}

describe('les imports de baril désignent des exports réels', () => {
  const cibles = RACINES.flatMap(fichiers)

  /**
   * **La cible n'est pas vide, et son décompte est asserté.**
   *
   * Exigence 2 de la section « Couverture des portes » : *une porte dont la cible est vide passe
   * toujours*. Renommer `app/modules/` ferait passer ce fichier au vert en n'inspectant rien.
   */
  it('inspecte un nombre plausible de fichiers', () => {
    expect(
      cibles.length,
      'aucun fichier lu : la cible est vide, et un contrôle à cible vide passe toujours',
    ).toBeGreaterThan(40)
  })

  for (const baril of BARILS) {
    it(`${baril} — chaque nom importé y est exporté`, () => {
      const disponibles = exportsDe(baril)
      expect(
        disponibles.size,
        `${baril}/index.ts n’expose aucun nom : la lecture est cassée`,
      ).toBeGreaterThan(0)

      const fautes: string[] = []
      for (const fichier of cibles) {
        const source = readFileSync(join(APP, fichier), 'utf8')
        for (const nom of importsDeValeur(source, baril)) {
          if (!disponibles.has(nom)) fautes.push(`${fichier} — « ${nom} »`)
        }
      }

      expect(
        fautes,
        `Ces fichiers importent une VALEUR que « ${baril} » n’exporte pas. En navigateur, le module `
        + 'ne se charge pas et l’écran ne se monte pas — sans qu’aucun test de composant ne le voie, '
        + 'puisqu’ils doublent le baril et fournissent le nom manquant.\n'
        + `Exports réels : ${[...disponibles].sort().join(', ')}\n  `
        + fautes.join('\n  '),
      ).toEqual([])
    })
  }
})
