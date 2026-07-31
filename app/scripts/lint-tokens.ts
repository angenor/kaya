/**
 * **Porte P-17** — aucune couleur ni espacement littéral hors des jetons de `theme.css`.
 *
 *     pnpm --filter @kaya/app lint:tokens
 *
 * # Ce qu'elle empêche vraiment
 *
 * Un `#1a1a1a` écrit dans un composant fonctionne — en mode clair. En mode sombre, il reste noir
 * sur fond noir. Le mode sombre passe par la **variante `dark:`** et par des tokens dont les
 * valeurs changent sous `.dark` (principe XII) ; une valeur littérale échappe à ce mécanisme, et
 * l'écart ne se voit qu'en basculant le thème — c'est-à-dire rarement, et jamais dans les tests.
 *
 * Un `padding: 14px` produit le même effet sur l'échelle d'espacement : il tient jusqu'au jour où
 * l'échelle change, et il faut alors retrouver chaque littéral un par un.
 *
 * # Le seul fichier exempté
 *
 * `assets/css/theme.css` — la copie exacte de `docs/design/theme.css`. **C'est lui qui définit
 * les jetons** ; lui interdire des valeurs littérales n'aurait aucun sens. C'est aussi la seule
 * exception du principe XII, et la porte P-19 vérifie par ailleurs qu'aucun autre fichier de
 * `docs/design/html/` n'a été copié.
 */

import { readFileSync, readdirSync, statSync } from 'node:fs'
import { join, relative } from 'node:path'

const RACINE = new URL('..', import.meta.url).pathname

/** Le seul fichier autorisé à porter des valeurs littérales : celui qui définit les jetons. */
const EXEMPTES = new Set(['assets/css/theme.css'])

const IGNORES = ['node_modules', '.nuxt', '.output', 'dist', 'src-tauri', 'tests', 'scripts']

let echec = false

function signaler(message: string): void {
  console.error(`  ✗ ${message}`)
  echec = true
}

function fichiersStyles(repertoire: string): string[] {
  const trouves: string[] = []
  let entrees: string[]
  try {
    entrees = readdirSync(repertoire)
  } catch {
    return trouves
  }
  for (const entree of entrees) {
    if (IGNORES.includes(entree)) continue
    const chemin = join(repertoire, entree)
    if (statSync(chemin).isDirectory()) {
      trouves.push(...fichiersStyles(chemin))
    } else if (/\.(vue|css|ts)$/.test(entree)) {
      trouves.push(chemin)
    }
  }
  return trouves
}

const MOTIFS: { nom: string, motif: RegExp, explication: string }[] = [
  {
    nom: 'couleur hexadécimale',
    motif: /#[0-9a-fA-F]{3,8}\b/g,
    explication:
      'une couleur littérale échappe à la bascule clair/sombre : elle reste identique sous `.dark`.',
  },
  {
    nom: 'couleur rgb()/rgba()',
    motif: /\brgba?\s*\(/g,
    explication: 'même effet qu’une hexadécimale — passer par un jeton de `@theme`.',
  },
  {
    nom: 'couleur hsl()/oklch()',
    motif: /\b(?:hsla?|oklch)\s*\(/g,
    explication: 'même effet — passer par un jeton de `@theme`.',
  },
  {
    nom: 'espacement en px',
    motif: /(?<![\w-])\d+(?:\.\d+)?px\b/g,
    explication:
      'un espacement littéral tient jusqu’au jour où l’échelle change, et il faut alors retrouver '
      + 'chaque valeur une par une.',
  },
]

/** Retire les commentaires : un `#rrggbb` cité dans une explication n'est pas un style. */
function sansCommentaires(contenu: string): string {
  return contenu
    .replace(/\/\*[\s\S]*?\*\//g, '')
    .replace(/(^|[^:])\/\/.*$/gm, '$1')
    .replace(/<!--[\s\S]*?-->/g, '')
}

const fichiers = fichiersStyles(RACINE)
console.log(`── P-17 — ${fichiers.length} fichier(s) analysé(s) ─────────────────────────────`)

for (const fichier of fichiers) {
  const relatif = relative(RACINE, fichier)
  if (EXEMPTES.has(relatif)) {
    console.log(`  · ${relatif} — exempté : c'est lui qui DÉFINIT les jetons`)
    continue
  }

  const contenu = sansCommentaires(readFileSync(fichier, 'utf8'))

  for (const { nom, motif, explication } of MOTIFS) {
    for (const capture of contenu.matchAll(motif)) {
      const avant = contenu.slice(0, capture.index ?? 0)
      const ligne = avant.split('\n').length
      signaler(`${relatif}:${ligne} — ${nom} « ${capture[0]} »\n      ${explication}`)
    }
  }
}

if (echec) {
  console.error('')
  console.error('P-17 ÉCHOUE — tout style s’exprime en utilitaires du noyau Tailwind référençant')
  console.error('les jetons de `@theme` (principe XII). Le CSS explicite est réservé à ce que')
  console.error('Tailwind n’exprime pas — @keyframes, impression thermique — et reste regroupé.')
  process.exit(1)
}

console.log('P-17 ✓ — aucune couleur ni espacement littéral hors des jetons.')
