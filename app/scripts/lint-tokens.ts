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

/**
 * **`branding.couleur_primaire` n'est PAS un style d'application** — exclusion nommée.
 *
 * C'est une **donnée client**, stockée en hexadécimal et appliquée aux **documents produits**
 * (FR-059). Un composant qui la lit manipule donc une valeur de couleur sans en poser aucune, et
 * la porte la signalerait à tort.
 *
 * Sans cette exclusion écrite, le réflexe serait de désactiver la règle sur le fichier — ce qui
 * l'aveuglerait aussi sur les vraies couleurs littérales qu'il pourrait porter un jour.
 *
 * **L'assertion ci-dessous remplace le signal supprimé** : elle vérifie que la couleur d'identité
 * visuelle n'est APPLIQUÉE nulle part dans les composants. Une exclusion sans contrepartie serait
 * un trou ; celle-ci en est une avec sa garde.
 */
const CHAMP_COULEUR_CLIENT = 'couleur_primaire'

/**
 * Un composant applique-t-il la couleur d'identité visuelle à l'interface ?
 *
 * On cherche les formes qui **posent** une couleur : liaison de style, propriété CSS custom,
 * attribut `style`. Nommer le champ dans un commentaire, un type ou un libellé reste libre — c'est
 * ce que fait la section d'identité visuelle de `G1`, qui l'affiche comme une valeur lisible.
 */
function appliqueLaCouleurClient(contenu: string): string[] {
  const formes: { motif: RegExp, quoi: string }[] = [
    { motif: /:style\s*=\s*["'][^"']*couleur_primaire/g, quoi: 'liaison :style' },
    { motif: /style\s*=\s*["'][^"']*couleur_primaire/g, quoi: 'attribut style' },
    { motif: /--[\w-]+\s*:\s*[^;]*couleur_primaire/g, quoi: 'propriété CSS custom' },
    { motif: /(?:background|color|border-color|fill|stroke)\s*:\s*[^;]*couleur_primaire/g, quoi: 'propriété de couleur' },
  ]

  const trouves: string[] = []
  for (const { motif, quoi } of formes) {
    if (motif.test(contenu)) trouves.push(quoi)
  }
  return trouves
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

  // **FR-059** — la couleur d'identité visuelle ne touche jamais l'interface.
  for (const forme of appliqueLaCouleurClient(contenu)) {
    signaler(
      `${relatif} — « ${CHAMP_COULEUR_CLIENT} » est APPLIQUÉE à l'interface (${forme}).\n`
      + '      C\'est une donnée client, pas un jeton de design : elle vaut pour les DOCUMENTS\n'
      + '      produits, jamais pour l\'application (FR-059). L\'appliquer à un bouton ferait\n'
      + '      prendre au produit la couleur de chaque client, et la bascule clair/sombre ne\n'
      + '      s\'appliquerait plus à cet élément.',
    )
  }

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
console.log(
  `  · « ${CHAMP_COULEUR_CLIENT} » exclue nommément — donnée client, jamais un style\n`
  + '    d\'application. Vérifié en contrepartie : elle n\'est appliquée dans aucun composant.',
)
