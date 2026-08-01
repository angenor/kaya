/**
 * **Porte P-21, versant production** — embarque le sous-ensemble d'icônes réellement employé.
 *
 *     pnpm --filter @kaya/app icones:generer            # régénère les artefacts
 *     pnpm --filter @kaya/app icones:generer --verifier # échoue sur tout écart (CI)
 *
 * # Ce que ce script résout
 *
 * Les maquettes de `docs/design/` chargent Phosphor depuis `unpkg.com`. Reprise telle quelle, la
 * ligne rend l'écran **dépendant du réseau** — ce que le principe VI interdit et ce que la porte
 * **P-21** refuse désormais. Sans police embarquée, les `<i class="ph ph-…">` de `G1` ne
 * rendent rien du tout : c'est l'état constaté à la fin du cycle 002.
 *
 * # Pourquoi un sous-ensemble et pas la police entière
 *
 * `Phosphor.woff2` pèse 147 ko pour ~1530 glyphes ; `Phosphor-Fill.woff2` 132 ko. Le produit en
 * emploie **moins de quatre-vingts**. La persona Aminata travaille sur un Android d'entrée de
 * gamme en réseau intermittent : 279 ko d'icônes jamais affichées sont 279 ko qui retardent le
 * premier écran à chaque installation.
 *
 * # PÉRIMÈTRE INSPECTÉ — ce que ce script lit, et ce qu'il ne lit pas
 *
 * **Il lit** :
 *
 * - `docs/design/**` — tout le HTML de référence : `html/`, `styleguide.html`, `fondation/`,
 *   `proto/`, `documents/`. C'est la **cible visuelle** : un glyphe qui y figure sera employé tôt
 *   ou tard par l'écran qui en hérite, et l'absence ne se verrait qu'au moment de le coder.
 * - `app/**` — `.vue` et `.ts`, hors `node_modules`, `.nuxt`, `.output`, `scripts`. C'est
 *   l'**emploi réel**, celui que la porte P-21 vérifie.
 *
 * **Il ne lit pas** : `web/` (les deux surfaces publiques n'ont aucun écran à ce jour), ni les
 * chaînes de nom de glyphe **calculées** à l'exécution. La seconde limite est réelle et assumée :
 * un nom construit par concaténation (`'ph-' + code.toLowerCase()`) échapperait à l'analyse. Le
 * produit n'en contient aucun aujourd'hui — `SectionServices.vue` emploie une table littérale,
 * exactement pour cette raison — et la règle est écrite ici pour qu'elle ne se perde pas.
 *
 * # Déterminisme
 *
 * `subset-font` est déterministe à l'octet, vérifié en régénérant deux fois de suite. C'est ce
 * qui rend le mode `--verifier` opposable : sans cette propriété, la porte échouerait au hasard
 * et serait désactivée sous trois semaines — la leçon du gel §3.2 sur le générateur de client.
 */

import { existsSync, mkdirSync, readFileSync, readdirSync, statSync, writeFileSync } from 'node:fs'
import { join, relative } from 'node:path'

import subsetFont from 'subset-font'

const APP = new URL('..', import.meta.url).pathname
const RACINE = new URL('../..', import.meta.url).pathname

const VERIFIER = process.argv.includes('--verifier')

/** Les deux variantes employées par le produit. `duotone`, `thin`, `bold` et `light` ne le sont pas. */
const VARIANTES = [
  {
    /** Nom de famille CSS — **identique à Phosphor**, pour que le HTML de maquette se transpose sans réécriture. */
    famille: 'Phosphor',
    /** Sélecteur des règles de contenu dans le CSS amont. */
    prefixe: 'ph',
    source: 'node_modules/@phosphor-icons/web/src/regular',
    police: 'Phosphor.ttf',
    style: 'style.css',
    sortie: 'assets/fonts/phosphor-kaya.woff2',
  },
  {
    famille: 'Phosphor-Fill',
    prefixe: 'ph-fill',
    source: 'node_modules/@phosphor-icons/web/src/fill',
    police: 'Phosphor-Fill.ttf',
    style: 'style.css',
    sortie: 'assets/fonts/phosphor-fill-kaya.woff2',
  },
] as const

const CSS_SORTIE = 'assets/css/icones.css'

// =================================================================================================
//  1. Relever les glyphes employés
// =================================================================================================

/** `ph-fill` et `ph-duotone` nomment un **style**, pas un glyphe : ils ne se sous-règlent pas. */
const STYLES = new Set(['ph-fill', 'ph-duotone', 'ph-thin', 'ph-bold', 'ph-light', 'ph-regular'])

const IGNORES = new Set(['node_modules', '.nuxt', '.output', 'dist', 'src-tauri', 'scripts', 'assets'])

function fichiers(repertoire: string, extensions: RegExp): string[] {
  const trouves: string[] = []
  let entrees: string[]
  try {
    entrees = readdirSync(repertoire)
  } catch {
    return trouves
  }
  for (const entree of entrees) {
    if (IGNORES.has(entree)) continue
    const chemin = join(repertoire, entree)
    if (statSync(chemin).isDirectory()) {
      trouves.push(...fichiers(chemin, extensions))
    } else if (extensions.test(entree)) {
      trouves.push(chemin)
    }
  }
  return trouves
}

/**
 * Relève les noms de glyphe d'un jeu de fichiers, **par variante**.
 *
 * On cherche le nom **dans un attribut de classe**, pas n'importe où dans le fichier : un
 * `ph-quelque-chose` cité dans un commentaire ou une prose n'est pas un glyphe rendu, et
 * l'embarquer gonflerait la police sans raison.
 *
 * La variante se lit dans le même attribut : `class="ph-fill ph-warning"` demande le glyphe
 * **plein**, `class="ph ph-warning"` le glyphe **au trait**. Les deux polices sont distinctes,
 * et sous-régler les deux sur l'union ferait porter à chacune des glyphes que personne ne
 * demande — la moitié du gain du sous-réglage.
 */
function releverGlyphes(chemins: string[]): Map<string, Map<string, string[]>> {
  const trouves = new Map<string, Map<string, string[]>>(
    VARIANTES.map(v => [v.prefixe, new Map<string, string[]>()]),
  )
  for (const chemin of chemins) {
    const contenu = readFileSync(chemin, 'utf8')
    for (const attribut of contenu.matchAll(/(?::?class\s*=\s*)(["'])([\s\S]*?)\1/g)) {
      const classes = attribut[2]
      const prefixe = /\bph-fill\b/.test(classes) ? 'ph-fill' : 'ph'
      for (const glyphe of classes.matchAll(/\bph-[a-z0-9-]+\b/g)) {
        const nom = glyphe[0]
        if (STYLES.has(nom)) continue
        const source = relative(RACINE, chemin)
        const parVariante = trouves.get(prefixe)!
        const liste = parVariante.get(nom)
        if (liste) {
          if (!liste.includes(source)) liste.push(source)
        } else {
          parVariante.set(nom, [source])
        }
      }
    }
  }
  return trouves
}

const glyphesMaquette = releverGlyphes(fichiers(join(RACINE, 'docs/design'), /\.html$/))
const glyphesApp = releverGlyphes(fichiers(APP, /\.(vue|ts)$/))

/** Les glyphes à embarquer, variante par variante — union de la maquette et de l'application. */
const employes = new Map<string, Map<string, string[]>>()
for (const variante of VARIANTES) {
  const union = new Map<string, string[]>()
  for (const source of [glyphesMaquette, glyphesApp]) {
    for (const [nom, ou] of source.get(variante.prefixe)!) {
      union.set(nom, [...(union.get(nom) ?? []), ...ou])
    }
  }
  employes.set(variante.prefixe, union)
}

const total = [...employes.values()].reduce((n, jeu) => n + jeu.size, 0)

console.log('── Icônes · périmètre relevé ─────────────────────────────────────────────────')
for (const variante of VARIANTES) {
  console.log(
    `  ${variante.prefixe.padEnd(8)} docs/design/ : ${String(glyphesMaquette.get(variante.prefixe)!.size).padStart(3)}`
    + ` · app/ : ${String(glyphesApp.get(variante.prefixe)!.size).padStart(3)}`
    + ` · à embarquer : ${employes.get(variante.prefixe)!.size}`,
  )
}

if (total === 0) {
  // Exigence 4 de la constitution, § Couverture des portes : une porte dont la cible est vide
  // passe toujours. Ici, une cible vide signifierait que l'extraction est cassée, pas que le
  // produit n'a pas d'icône — et le sous-ensemble produit serait une police sans glyphe.
  console.error('  ✗ AUCUN glyphe relevé. L’extraction est cassée : elle ne peut pas rendre zéro')
  console.error('    alors que `docs/design/html/` en emploie plusieurs dizaines.')
  process.exit(1)
}

// =================================================================================================
//  2. Sous-régler chaque variante
// =================================================================================================

/** Table `nom de glyphe → point de code` lue dans le CSS amont, jamais devinée. */
function tableDesPoints(cssAmont: string, prefixe: string): Map<string, number> {
  const table = new Map<string, number>()
  const motif = new RegExp(`\\.${prefixe}\\.(ph-[a-z0-9-]+):before\\s*\\{\\s*content:\\s*"\\\\([0-9a-f]+)";`, 'g')
  for (const regle of cssAmont.matchAll(motif)) {
    table.set(regle[1], Number.parseInt(regle[2], 16))
  }
  return table
}

const ecarts: string[] = []
const regles: string[] = []
const attendus = new Map<string, Buffer>()

for (const variante of VARIANTES) {
  const source = join(APP, variante.source)
  if (!existsSync(source)) {
    console.error(`  ✗ ${variante.source} absent — installer @phosphor-icons/web (gel §3.2).`)
    process.exit(1)
  }

  const cssAmont = readFileSync(join(source, variante.style), 'utf8')
  const points = tableDesPoints(cssAmont, variante.prefixe)
  const demandes = employes.get(variante.prefixe)!

  // Un glyphe employé mais inconnu du référentiel amont ne rendrait rien : c'est une faute de
  // frappe dans un nom de classe, et elle ne se voit pas autrement qu'ici.
  const retenus = [...demandes.keys()].filter(nom => points.has(nom)).sort()

  // Le paramètre du filtre est nommé `candidat` et non `nom` : sous le même nom il masquerait la
  // variable de boucle, et la règle `@typescript-eslint/no-shadow` le refuse. Ici les deux
  // désignaient la même chose, donc rien ne cassait — mais c'est la forme exacte de la collision
  // qui, dans `SectionServices.vue`, a empêché un bandeau de s'afficher.
  for (const nom of [...demandes.keys()].filter(candidat => !points.has(candidat)).sort()) {
    ecarts.push(`« ${nom} » n'existe pas dans ${variante.famille} — ${demandes.get(nom)!.join(', ')}`)
  }

  const texte = retenus.map(nom => String.fromCodePoint(points.get(nom)!)).join('')
  const woff2 = await subsetFont(readFileSync(join(source, variante.police)), texte, {
    targetFormat: 'woff2',
  })

  attendus.set(variante.sortie, Buffer.from(woff2))

  const complet = statSync(join(source, variante.police)).size
  console.log(
    `  ${variante.famille.padEnd(14)} ${String(retenus.length).padStart(3)} glyphe(s) · `
    + `${(woff2.length / 1024).toFixed(1)} ko (police complète : ${(complet / 1024).toFixed(0)} ko de ttf)`,
  )

  regles.push(
    `/* ── ${variante.famille} — ${retenus.length} glyphe(s) ${'─'.repeat(Math.max(0, 46 - variante.famille.length))} */`,
    `@font-face {`,
    `  font-family: "${variante.famille}";`,
    `  src: url("../fonts/${variante.sortie.split('/').pop()}") format("woff2");`,
    `  font-weight: normal;`,
    `  font-style: normal;`,
    // `block` et non `swap` : une icône qui apparaît d'abord en carré de substitution puis change
    // de forme est plus dérangeante qu'une icône qui arrive un instant plus tard. La police est
    // locale, donc l'instant est court.
    `  font-display: block;`,
    `}`,
    ``,
    `.${variante.prefixe} {`,
    `  font-family: "${variante.famille}";`,
    `  font-style: normal;`,
    `  font-weight: normal;`,
    `  font-variant: normal;`,
    `  text-transform: none;`,
    `  line-height: 1;`,
    `  -webkit-font-smoothing: antialiased;`,
    `  -moz-osx-font-smoothing: grayscale;`,
    `}`,
    ``,
    ...retenus.map(
      nom => `.${variante.prefixe}.${nom}:before { content: "\\${points.get(nom)!.toString(16)}"; }`,
    ),
    ``,
  )
}

if (ecarts.length > 0) {
  console.error('')
  for (const ecart of ecarts) console.error(`  ✗ ${ecart}`)
  console.error('')
  console.error('Un nom de glyphe inconnu du référentiel Phosphor ne rend RIEN à l’écran, et rien')
  console.error('ne le signale : ni la compilation, ni un test de rendu. Corriger le nom.')
  process.exit(1)
}

// =================================================================================================
//  3. Écrire ou vérifier
// =================================================================================================

const enTete = `/* ═══════════════════════════════════════════════════════════════════════════
   KAYA — ICÔNES EMBARQUÉES  ·  FICHIER GÉNÉRÉ, NE PAS ÉDITER À LA MAIN

   Produit par \`app/scripts/generer-icones.ts\` depuis @phosphor-icons/web
   (gel §3.2). Sous-ensemble : seuls les glyphes réellement employés par
   \`docs/design/\` et \`app/\` sont embarqués.

       pnpm --filter @kaya/app icones:generer

   PORTE P-21 — aucune ressource d'hôte externe. Les maquettes chargent
   Phosphor depuis unpkg.com ; le produit ne le fait JAMAIS. Un CDN rend
   l'écran dépendant du réseau, ce que le principe VI interdit.

   Ce fichier ne porte aucune couleur : une icône hérite de \`currentColor\`,
   donc du jeton de texte posé sur elle (\`text-ocre\`, \`text-prim\`…). C'est
   ce qui la fait basculer en mode sombre sans une seule règle de plus.
   ═══════════════════════════════════════════════════════════════════════════ */

`

attendus.set(CSS_SORTIE, Buffer.from(enTete + regles.join('\n')))

let echec = false

for (const [relatifSortie, contenu] of attendus) {
  const chemin = join(APP, relatifSortie)

  if (VERIFIER) {
    if (!existsSync(chemin)) {
      console.error(`  ✗ ${relatifSortie} absent — exécuter \`pnpm --filter @kaya/app icones:generer\`.`)
      echec = true
      continue
    }
    if (Buffer.compare(readFileSync(chemin), contenu) !== 0) {
      console.error(`  ✗ ${relatifSortie} diffère de ce que le périmètre produit.`)
      echec = true
      continue
    }
    console.log(`  ✓ ${relatifSortie} à jour`)
    continue
  }

  mkdirSync(join(chemin, '..'), { recursive: true })
  writeFileSync(chemin, contenu)
  console.log(`  → ${relatifSortie} (${(contenu.length / 1024).toFixed(1)} ko)`)
}

if (echec) {
  console.error('')
  console.error('Les icônes embarquées ne correspondent plus aux glyphes employés. Un glyphe ajouté')
  console.error('à un composant sans régénération ne s’affiche pas — et rien d’autre ne le dit.')
  process.exit(1)
}

console.log(VERIFIER ? 'Icônes ✓ — sous-ensemble à jour.' : 'Icônes ✓ — sous-ensemble régénéré.')
