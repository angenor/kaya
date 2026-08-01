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

/**
 * **Les épaisseurs de bordure ne sont PAS des espacements** — exclusion nommée et bornée.
 *
 * `docs/design/README.md`, décision n° 1 : Tailwind 4 **n'a pas d'espace de nom pour les
 * épaisseurs de bordure**. Deux valeurs sont concernées, et elles sont assumées à ce titre :
 * **1,5 px** (contour de bouton secondaire, bordure de champ) sur 21 fichiers de maquette, et
 * **3 px** (contrefort de carte de vérification) sur 9. Il n'existe aucune façon de les écrire en
 * jeton ; l'échelle d'espacement, elle, n'a rien à voir avec elles — un `padding: 14px` reste une
 * faute, et le reste de la porte continue de l'attraper.
 *
 * **L'exception est bornée sur trois axes**, sans quoi elle rouvrirait la porte en grand :
 *
 * 1. seules ces **deux valeurs** passent — `2.5px` ou `1.4px` restent refusés ;
 * 2. seulement dans une **utilitaire de bordure** — `border-[…]`, `border-l-[…]`, `border-t-[…]` ;
 *    la même valeur en `p-[1.5px]` ou `gap-[3px]` reste refusée ;
 * 3. les emplois sont **comptés et affichés** à chaque exécution. Une exception silencieuse
 *    devient invisible en trois cycles ; celle-ci se voit à chaque passage de la porte.
 */
const EPAISSEURS_ASSUMEES = /\bborder(?:-[trblxyse])?-\[(1\.5px|3px)\]/g

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

/** Les emplois d'épaisseur assumée, comptés pour rester visibles. */
const epaisseurs: string[] = []

for (const fichier of fichiers) {
  const relatif = relative(RACINE, fichier)
  if (EXEMPTES.has(relatif)) {
    console.log(`  · ${relatif} — exempté : c'est lui qui DÉFINIT les jetons`)
    continue
  }

  let contenu = sansCommentaires(readFileSync(fichier, 'utf8'))

  // Les deux épaisseurs de bordure de la décision n° 1 sont retirées AVANT l'analyse, et
  // seulement dans une utilitaire de bordure. Le reste du fichier est analysé normalement.
  contenu = contenu.replace(EPAISSEURS_ASSUMEES, (emploi, valeur) => {
    epaisseurs.push(`${relatif} — ${emploi}`)
    return `border-EPAISSEUR-ASSUMEE-${valeur.replace('.', '-').replace('px', '')}`
  })

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
console.log(
  `  · ${epaisseurs.length} épaisseur(s) de bordure assumée(s) — 1,5 px et 3 px, décision n° 1 du\n`
  + '    README de design. Tailwind 4 n\'a pas d\'espace de nom pour elles. Comptées ici pour\n'
  + '    qu\'une exception ne devienne pas invisible :',
)
for (const emploi of epaisseurs) {
  console.log(`      ${emploi}`)
}
