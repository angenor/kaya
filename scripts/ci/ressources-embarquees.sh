#!/usr/bin/env bash
#
# Porte P-21b — toute ressource déclarée est effectivement embarquée.
#
#     pnpm porte:p21b
#
# **P-21 interdit ; P-21b exige.** C'est le « versant positif » de la constitution (§ Couverture des
# portes, exigence 4, corollaire) : *une porte qui refuse une source externe doit vérifier que le
# contenu local existe*. Sans ce versant, **supprimer la cible suffit à passer au vert**.
#
# Le mécanisme a produit deux fois le même défaut :
#   · cycle 002 — le CDN d'icônes retiré, aucune police embarquée à la place : P-21 verte, écran G1
#     sans une seule icône ;
#   · volet suivant — Archivo et Chivo Mono jamais embarquées : P-21 verte, application entière sur
#     les polices système de repli, et les colonnes de montants désalignées (`tokens.md` §2).
#
# ─────────────────────────────────────────────────────────────────────────────────────────────────
#  PÉRIMÈTRE INSPECTÉ — ce que la porte lit, et ce qu'elle ne lit pas
# ─────────────────────────────────────────────────────────────────────────────────────────────────
#
#  ELLE LIT
#    · `app/assets/css/theme.css` — le bloc `@theme`, pour la liste des familles DÉCLARÉES
#      (`--font-titre`, `--font-texte`, `--font-mono`).
#    · toute feuille `.css` de `app/` et `web/` — pour les `@font-face` qui les SERVENT, et pour
#      les fichiers qu'ils désignent.
#    · les `woff2` désignés par ces `@font-face` — **par lecture de leur table `cmap`**, pas de leur
#      `unicode-range` déclarée. C'est la distinction qui compte : Fontsource annonce `U+2000-206F`
#      pour le sous-ensemble `latin`, alors que U+202F n'y est pas.
#    · `app/**/*.{vue,ts}` — les classes `ph-…` réellement employées, et `app/assets/css/icones.css`
#      qui doit les porter.
#
#  ELLE NE LIT PAS, et c'est délibéré
#    · `docs/design/` — les maquettes chargent volontairement leurs polices depuis un CDN. Ce sont
#      des cibles de lecture, pas du code livré (principe XII).
#    · `backend/`, `node_modules/`, `.nuxt/`, `.output/`, `dist/`, `src-tauri/`.
#    · les familles **non citées** par un jeton `--font-*` et non employées par une classe `ph-…` :
#      il n'y en a aucune aujourd'hui, et une famille posée en dur dans un composant serait déjà
#      refusée par P-17 (aucune valeur littérale hors jetons).
#    · le RENDU. La porte prouve que le caractère est dans le fichier, pas qu'il s'affiche à l'écran.
#      Un contrôle de rendu demanderait un navigateur en CI ; la vérification par harfbuzz — le
#      moteur de mise en forme des navigateurs — est faite à la génération, dans
#      `app/scripts/generer-polices.ts`.
#
#  CINQ CONTRÔLES
#    1. Chaque famille déclarée dans `--font-*` est servie par un `@font-face` LOCAL.
#    2. Chaque fichier désigné par un `@font-face` existe sur le disque et n'est pas vide.
#    3. U+202F et le jeu latin étendu sont présents dans les polices de texte, **dans le fichier
#       dont la `unicode-range` couvre le caractère** — pas « quelque part dans la famille ».
#    4. Chaque glyphe `ph-…` employé est embarqué. **Ce contrôle vient de P-21, il n'y est pas
#       resté** : `scripts/ci/aucune-ressource-externe.sh` ne le porte plus. Une interdiction et son
#       versant positif se lisent mieux séparés, et le dupliquer les aurait laissés diverger.
#    5. Chaque police de `app/assets/fonts/` a son **fichier de licence** et son **avis de
#       copyright**. Même versant positif que les quatre autres : embarquer une police est une
#       **redistribution**, et l'OFL 1.1 (clause 2) comme le MIT exigent que l'avis accompagne
#       toutes les copies. Sans ce contrôle, ajouter une police au répertoire suffirait à mettre
#       le produit en défaut, et rien ne le dirait — ni la compilation, ni un test de rendu, ni
#       les quatre contrôles ci-dessus, qui ne regardent que ce qui S'AFFICHE.
#
#  La porte NE MODIFIE RIEN de ce qu'elle inspecte (exigence 3). Elle n'a besoin d'AUCUN
#  `node_modules` : le lecteur de woff2 de `app/scripts/woff2.ts` ne dépend que de `node:zlib`.

set -euo pipefail

racine="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$racine"

RACINE="$racine" node --experimental-strip-types --input-type=module - <<'JS'
import { existsSync, readFileSync, readdirSync, statSync } from 'node:fs'
import { dirname, join, relative, resolve } from 'node:path'

const RACINE = process.env.RACINE
const { lireCmap, cmapDe, couvertPar } = await import(`${RACINE}/app/scripts/woff2.ts`)

const ARBRES = ['app', 'web']
const IGNORES = new Set(['node_modules', '.nuxt', '.output', 'dist', 'src-tauri', '.git'])

let echec = false
const signaler = (message) => {
  console.error(`  ✗ ${message}`)
  echec = true
}

function fichiers(arbre, extensions) {
  const trouves = []
  const parcourir = (repertoire) => {
    let entrees
    try {
      entrees = readdirSync(repertoire)
    } catch {
      return
    }
    for (const entree of entrees) {
      if (IGNORES.has(entree)) continue
      const chemin = join(repertoire, entree)
      if (statSync(chemin).isDirectory()) parcourir(chemin)
      else if (extensions.test(entree)) trouves.push(chemin)
    }
  }
  parcourir(join(RACINE, arbre))
  return trouves.sort()
}

const feuilles = ARBRES.flatMap(a => fichiers(a, /\.css$/))

// =================================================================================================
//  1 — chaque famille déclarée dans --font-* est servie par un @font-face local
// =================================================================================================

console.log('── P-21b · 1/5 — les familles déclarées sont-elles servies ? ' + '─'.repeat(21))

const THEME = join(RACINE, 'app/assets/css/theme.css')
if (!existsSync(THEME)) {
  signaler('app/assets/css/theme.css absent — la porte ne sait plus quelles familles sont déclarées.')
}

const theme = existsSync(THEME) ? readFileSync(THEME, 'utf8') : ''

/**
 * Les familles à servir localement : la PREMIÈRE de chaque pile `--font-*`, et seulement si elle
 * est entre guillemets. Les suivantes — `ui-sans-serif`, `system-ui`, `"SFMono-Regular"` — sont des
 * replis fournis par la plateforme, qu'on n'embarque évidemment pas.
 */
const declarees = new Map()
for (const jeton of theme.matchAll(/--font-([a-z0-9-]+)\s*:\s*([^;]+);/g)) {
  const premiere = jeton[2].split(',')[0].trim()
  const entreGuillemets = premiere.match(/^["']([^"']+)["']$/)
  if (entreGuillemets) declarees.set(entreGuillemets[1], `--font-${jeton[1]}`)
}

/** Tous les `@font-face` des feuilles inspectées, avec leur famille, leur source et leur plage. */
const servies = []
for (const feuille of feuilles) {
  const contenu = readFileSync(feuille, 'utf8')
  for (const bloc of contenu.matchAll(/@font-face\s*\{([^}]*)\}/g)) {
    const corps = bloc[1]
    const famille = corps.match(/font-family\s*:\s*["']?([^;"']+)["']?\s*;/)
    const source = corps.match(/url\(\s*["']?([^)"']+)["']?\s*\)/)
    const plage = corps.match(/unicode-range\s*:\s*([^;]+);/)
    servies.push({
      feuille,
      famille: famille ? famille[1].trim() : null,
      source: source ? source[1].trim() : null,
      plage: plage ? plage[1].trim() : null,
      affichage: corps.match(/font-display\s*:\s*([a-z]+)/)?.[1] ?? null,
    })
  }
}

for (const [famille, jeton] of declarees) {
  const siennes = servies.filter(s => s.famille === famille)
  if (siennes.length === 0) {
    signaler(
      `« ${famille} » est déclarée par ${jeton} dans @theme, et AUCUN @font-face local ne la sert.\n`
      + "      L'application tourne sur une police système de repli. Ce n'est pas cosmétique :\n"
      + '      `tokens.md` §2 confie à Chivo Mono tabulaire l\'alignement des colonnes de montants.\n'
      + '      Exécuter `pnpm --filter @kaya/app polices:generer`.',
    )
  } else {
    // `swap` et non `block` pour du texte : sur une liaison lente, un texte invisible pendant trois
    // secondes est pire qu'un texte d'abord affiché dans la police de repli. `theme.css` le
    // prescrit explicitement.
    for (const s of siennes.filter(s => s.affichage !== 'swap')) {
      signaler(
        `${relative(RACINE, s.feuille)} — @font-face « ${famille} » sans « font-display: swap »`
        + ` (trouvé : ${s.affichage ?? 'rien'}).\n`
        + '      `theme.css`, section POLICES, le prescrit : le produit tourne sur des liaisons lentes.',
      )
    }
    console.log(`  ✓ ${famille.padEnd(12)} ${jeton.padEnd(14)} servie par ${siennes.length} @font-face local(aux)`)
  }
}

if (declarees.size === 0) {
  signaler('aucune famille relevée dans le bloc @theme — la porte ne garde RIEN (exigence 4).')
}
if (servies.length === 0) {
  signaler('aucun @font-face trouvé dans app/ ni web/ — la porte ne garde RIEN (exigence 4).')
}
console.log(`  ${declarees.size} famille(s) déclarée(s), ${servies.length} @font-face inspecté(s), ${feuilles.length} feuille(s)`)

// =================================================================================================
//  2 — chaque fichier désigné par un @font-face existe réellement
// =================================================================================================

console.log('── P-21b · 2/5 — les fichiers désignés existent-ils ? ' + '─'.repeat(28))

let verifies = 0
for (const s of servies) {
  if (!s.source) {
    signaler(`${relative(RACINE, s.feuille)} — @font-face « ${s.famille ?? '?'} » sans url().`)
    continue
  }
  if (/^(https?:)?\/\//.test(s.source)) {
    // P-21 le refuse déjà ; le redire ici évite qu'un contournement de l'une passe par l'autre.
    signaler(`${relative(RACINE, s.feuille)} — @font-face « ${s.famille} » pointe un hôte externe : ${s.source}`)
    continue
  }
  const chemin = resolve(dirname(s.feuille), s.source.replace(/[?#].*$/, ''))
  if (!existsSync(chemin) || statSync(chemin).size === 0) {
    signaler(
      `${relative(RACINE, s.feuille)} — @font-face « ${s.famille} » désigne un fichier absent ou vide :\n`
      + `      ${relative(RACINE, chemin)}\n`
      + '      Le navigateur passe au repli sans rien dire, et aucune compilation ne le signale.',
    )
    continue
  }
  s.chemin = chemin
  verifies += 1
}
console.log(`  ${verifies}/${servies.length} fichier(s) de police présent(s) sur le disque`)

// =================================================================================================
//  3 — U+202F et le jeu latin étendu, LUS DANS LA POLICE
// =================================================================================================

console.log('── P-21b · 3/5 — U+202F et le latin étendu sont-ils dessinés ? ' + '─'.repeat(19))

/**
 * Les témoins, et pourquoi chacun.
 *
 * U+202F porte la règle des montants de `tokens.md` §2. Les autres couvrent le français tel qu'il
 * s'écrit en Côte d'Ivoire — un nom propre, une commune, un libellé saisi par l'exploitant — et
 * séparent volontairement ce que `latin` porte de ce que seul `latin-ext` porte : c'est ce qui
 * rend la porte capable de refuser un embarquement réduit à `latin` seul.
 */
const TEMOINS = [
  { code: 0x202f, quoi: 'U+202F espace fine insécable', pourquoi: 'tokens.md §2 : « 12 500 F »' },
  { code: 0x0153, quoi: 'œ', pourquoi: 'cœur, sœur, œuvre' },
  { code: 0x0152, quoi: 'Œ', pourquoi: 'majuscule de la ligature' },
  { code: 0x00e9, quoi: 'é', pourquoi: 'français courant' },
  { code: 0x00e8, quoi: 'è', pourquoi: 'français courant' },
  { code: 0x00e0, quoi: 'à', pourquoi: 'français courant' },
  { code: 0x00e7, quoi: 'ç', pourquoi: 'français courant' },
  { code: 0x00f9, quoi: 'ù', pourquoi: 'français courant' },
  { code: 0x00fb, quoi: 'û', pourquoi: 'français courant' },
  { code: 0x00ee, quoi: 'î', pourquoi: 'français courant' },
  { code: 0x00f4, quoi: 'ô', pourquoi: 'français courant' },
  { code: 0x00eb, quoi: 'ë', pourquoi: 'tréma' },
  { code: 0x00ef, quoi: 'ï', pourquoi: 'tréma — Haïti, maïs' },
  { code: 0x00fc, quoi: 'ü', pourquoi: 'tréma' },
  { code: 0x00ff, quoi: 'ÿ', pourquoi: 'tréma' },
  { code: 0x00c9, quoi: 'É', pourquoi: 'capitale accentuée — un nom en capitales' },
  { code: 0x00c0, quoi: 'À', pourquoi: 'capitale accentuée' },
  { code: 0x00c7, quoi: 'Ç', pourquoi: 'capitale accentuée' },
  { code: 0x0178, quoi: 'Ÿ', pourquoi: 'LATIN ÉTENDU — absent de « latin » seul' },
  { code: 0x014c, quoi: 'Ō', pourquoi: 'LATIN ÉTENDU — macron' },
  { code: 0x0160, quoi: 'Š', pourquoi: 'LATIN ÉTENDU — caron' },
  { code: 0x1e9e, quoi: 'ẞ', pourquoi: 'LATIN ÉTENDU — plage U+1E00-1E9F' },
]

const couvertures = new Map()
let policesLues = 0

for (const s of servies) {
  if (!s.chemin || !declarees.has(s.famille)) continue
  try {
    const cmap = lireCmap(cmapDe(readFileSync(s.chemin), relative(RACINE, s.chemin)))
    couvertures.set(s, cmap)
    policesLues += 1
  } catch (erreur) {
    signaler(`${relative(RACINE, s.chemin)} — illisible : ${erreur.message}`)
  }
}

let controles = 0
for (const famille of declarees.keys()) {
  const siennes = servies.filter(s => s.famille === famille && couvertures.has(s))
  if (siennes.length === 0) continue

  for (const temoin of TEMOINS) {
    // **Le fichier interrogé est celui que le navigateur consulterait**, et un seul le sera.
    // Quand plusieurs `@font-face` d'une même famille couvrent le caractère avec le même style et
    // la même graisse — ce qui arrive : `œ` est annoncé par `latin` ET par `latin-ext` —, **c'est
    // le dernier déclaré qui l'emporte** (CSS Fonts 4, sélection par `unicode-range`). Contrôler
    // « quelque part dans la famille » laisserait passer un caractère rangé dans un fichier que le
    // navigateur n'ouvrira jamais pour lui : c'est le défaut que cette porte a trouvé le jour où
    // elle a été écrite, `polices.css` déclarant alors `latin` avant `latin-ext`.
    const candidats = siennes.filter(s => s.plage === null || couvertPar(s.plage, temoin.code))
    if (candidats.length === 0) {
      signaler(
        `${famille} — aucun @font-face ne déclare de plage couvrant ${temoin.quoi} (${temoin.pourquoi}).\n`
        + '      Le caractère tombe en repli quel que soit le contenu des fichiers.',
      )
      continue
    }

    const gagnant = candidats[candidats.length - 1]
    controles += 1
    if (!couvertures.get(gagnant).has(temoin.code)) {
      const autres = candidats.filter(c => couvertures.get(c).has(temoin.code))
      signaler(
        `${relative(RACINE, gagnant.chemin)} — ${temoin.quoi} (${temoin.pourquoi}) n'est PAS dessiné,\n`
        + '      alors que sa unicode-range le déclare couvert. La plage annoncée n\'est pas la\n'
        + '      couverture réelle : seule la table cmap fait foi. Le caractère tombe sur une police\n'
        + '      de repli, de chasse inconnue — et les colonnes de montants ne s\'alignent plus.\n'
        + (autres.length > 0
          ? `      Il EST dans ${autres.map(a => relative(RACINE, a.chemin)).join(', ')}, mais ce\n`
            + '      @font-face est déclaré AVANT : à plages recouvrantes, le dernier déclaré gagne.\n'
            + '      Corriger l\'ordre des @font-face, pas le contenu des fichiers.'
          : '      Aucun fichier de la famille ne le porte.'),
      )
    }
  }
}

if (policesLues === 0) {
  signaler('aucune police de texte lue — la porte ne garde RIEN (exigence 4).')
}
console.log(`  ${policesLues} police(s) de texte lue(s), ${controles} contrôle(s) de caractère`)

// =================================================================================================
//  4 — les glyphes d'icônes employés sont embarqués (contrôle venu de P-21)
// =================================================================================================

console.log('── P-21b · 4/5 — les icônes employées sont-elles embarquées ? ' + '─'.repeat(20))

const FEUILLE_ICONES = join(RACINE, 'app/assets/css/icones.css')
const STYLES = new Set(['ph-fill', 'ph-duotone', 'ph-thin', 'ph-bold', 'ph-light', 'ph-regular'])

if (!existsSync(FEUILLE_ICONES)) {
  signaler(
    'app/assets/css/icones.css absent — aucune icône n\'est embarquée.\n'
    + '      Exécuter `pnpm --filter @kaya/app icones:generer`.',
  )
} else {
  const feuille = readFileSync(FEUILLE_ICONES, 'utf8')
  const embarques = new Set([...feuille.matchAll(/\.(ph-[a-z0-9-]+):before/g)].map(m => m[1]))

  const employes = new Map()
  for (const chemin of fichiers('app', /\.(vue|ts)$/)) {
    if (chemin.includes('/scripts/') || chemin.includes('/assets/')) continue
    const contenu = readFileSync(chemin, 'utf8')
    for (const attribut of contenu.matchAll(/(?::?class\s*=\s*)(["'])([\s\S]*?)\1/g)) {
      for (const glyphe of attribut[2].matchAll(/\bph-[a-z0-9-]+\b/g)) {
        if (STYLES.has(glyphe[0])) continue
        if (!employes.has(glyphe[0])) employes.set(glyphe[0], relative(RACINE, chemin))
      }
    }
  }

  for (const [nom, ou] of [...employes].sort()) {
    if (!embarques.has(nom)) {
      signaler(
        `${ou} — « ${nom} » est employé mais absent de la police embarquée.\n`
        + "      L'icône ne s'affiche pas, et rien d'autre ne le dit : ni la compilation,\n"
        + '      ni un test de rendu. Exécuter `pnpm --filter @kaya/app icones:generer`.',
      )
    }
  }

  console.log(`  ${employes.size} glyphe(s) employé(s) dans app/, ${embarques.size} embarqué(s) dans la feuille`)
  if (employes.size === 0) {
    console.log('  · aucun glyphe employé — contrôle installé à vide, il reprendra au premier `<i class="ph …">`')
  }
}

// =================================================================================================
//  5 — chaque police embarquée a sa licence et son avis de copyright
// =================================================================================================

console.log('── P-21b · 5/5 — les polices embarquées sont-elles attribuées ? ' + '─'.repeat(18))

/**
 * Le répertoire des polices livrées. **Un `woff2` qui s'y trouve part chez le client** — c'est ce
 * qui déclenche l'obligation, pas le fait qu'il soit référencé par une feuille de style.
 */
const POLICES = join(RACINE, 'app/assets/fonts')

/**
 * Le fichier qui déclare ce qui a été modifié.
 *
 * Il n'est pas exigé par l'OFL — la clause 2 ne demande que l'avis et la licence. Il est exigé ici
 * parce que **deux des quatre polices de texte ont une table `cmap` réécrite** : une police
 * modifiée reste sous sa licence, mais la modification se déclare, faute de quoi personne ne saura
 * dans trois cycles pourquoi les fichiers diffèrent de l'amont.
 */
const DECLARATION = join(POLICES, 'MODIFICATIONS.md')

if (!existsSync(POLICES)) {
  signaler('app/assets/fonts/ absent — la porte ne garde RIEN sur les licences (exigence 4).')
}
else {
  const embarquees = readdirSync(POLICES).filter(f => /\.(woff2?|ttf|otf)$/.test(f)).sort()
  const licences = readdirSync(POLICES).filter(f => /-LICENCE\.txt$/.test(f)).sort()

  if (embarquees.length === 0) {
    signaler('aucune police dans app/assets/fonts/ — la porte ne garde RIEN (exigence 4).')
  }

  if (!existsSync(DECLARATION)) {
    signaler(
      'app/assets/fonts/MODIFICATIONS.md absent.\n'
      + '      Les woff2 embarqués ne sont pas les fichiers amont : la table cmap d\'Archivo et de\n'
      + '      Chivo Mono porte U+202F en plus, et les polices d\'icônes sont sous-réglées. Une\n'
      + '      modification non déclarée devient, en trois cycles, un écart que personne ne sait\n'
      + '      expliquer — et qu\'on « corrige » en régénérant depuis l\'amont, ce qui casse les\n'
      + '      montants.',
    )
  }

  /**
   * Chaque licence couvre les polices dont le nom commence par son préfixe.
   *
   * `chivo-mono-LICENCE.txt` couvre `chivo-mono-latin-kaya.woff2` et `chivo-mono-latin-ext-kaya.woff2` ;
   * `phosphor-LICENCE.txt` couvre `phosphor-kaya.woff2` et `phosphor-fill-kaya.woff2`. Le
   * rapprochement se fait par préfixe plutôt que par une table écrite ici : une table serait une
   * seconde source de vérité, à tenir à jour à chaque police ajoutée — exactement ce qu'on oublie.
   */
  const couverture = new Map(licences.map(l => [l.replace(/-LICENCE\.txt$/, ''), l]))

  for (const police of embarquees) {
    const prefixe = [...couverture.keys()]
      .filter(p => police.startsWith(`${p}-`))
      // Le plus long préfixe gagne : sans cela, une future licence « archivo » couvrirait à tort
      // une police « archivo-narrow » qui aurait la sienne.
      .sort((a, b) => b.length - a.length)[0]

    if (!prefixe) {
      signaler(
        `app/assets/fonts/${police} n'a aucun fichier de licence.\n`
        + '      Embarquer une police dans un binaire vendu par abonnement est une REDISTRIBUTION\n'
        + '      COMMERCIALE : l\'OFL 1.1 (clause 2) et le MIT exigent tous deux que l\'avis de\n'
        + `      copyright et la licence accompagnent toutes les copies. Attendu : un fichier\n`
        + `      « <préfixe>-LICENCE.txt » à côté, copie EXACTE de l'amont.\n`
        + '      Inventaire : docs/conformite/licences-tierces.md.',
      )
      continue
    }

    const texte = readFileSync(join(POLICES, couverture.get(prefixe)), 'utf8')

    // L'avis de copyright, pas seulement le texte de la licence : c'est lui que les deux licences
    // demandent d'inclure, et un fichier de licence générique copié sans l'avis ne suffit pas.
    if (!/copyright/i.test(texte)) {
      signaler(
        `app/assets/fonts/${couverture.get(prefixe)} ne porte aucun avis de copyright.\n`
        + '      « The above copyright notice … shall be included in all copies » : le texte de la\n'
        + '      licence seul ne satisfait pas la clause — c\'est l\'avis nominatif qu\'elle vise.',
      )
      continue
    }

    console.log(`  ✓ ${police.padEnd(30)} ← ${couverture.get(prefixe)}`)
  }

  // Une licence qui ne couvre plus rien signale une police retirée sans nettoyage — inoffensif,
  // mais c'est ainsi qu'un répertoire devient illisible.
  for (const [prefixe, fichier] of couverture) {
    if (!embarquees.some(p => p.startsWith(`${prefixe}-`))) {
      signaler(`app/assets/fonts/${fichier} ne couvre plus aucune police embarquée — la retirer.`)
    }
  }

  console.log(`  ${embarquees.length} police(s) embarquée(s), ${licences.length} licence(s), modifications déclarées`)
}

// =================================================================================================

console.log('── P-21b · bilan ' + '─'.repeat(64))

if (echec) {
  console.error('')
  console.error('P-21b ÉCHOUE — une ressource est DÉCLARÉE sans être EMBARQUÉE, ou une police est')
  console.error('embarquée sans son attribution. Retirer une source externe sans embarquer son')
  console.error('contenu fait passer P-21 au vert en n’affichant rien.')
  process.exit(1)
}

console.log('P-21b ✓ — familles servies en local, fichiers présents, U+202F et latin étendu dessinés,')
console.log('           icônes employées embarquées, polices attribuées.')
console.log('  Limite assumée : la porte prouve que le caractère est DANS le fichier, pas qu\'il')
console.log('  s\'affiche. Le contrôle de rendu par harfbuzz est fait à la génération des polices.')
console.log('  Limite assumée : le contrôle 5 vérifie qu\'une licence ACCOMPAGNE chaque police, pas')
console.log('  qu\'elle est la bonne. Comparer le texte à l\'amont demanderait node_modules, que')
console.log('  cette porte n\'a délibérément pas — c\'est ce qui lui permet de tourner sur un')
console.log('  changement de documentation seul.')
JS
