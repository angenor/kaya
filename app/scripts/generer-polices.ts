/**
 * **Porte P-21b, versant production** — embarque Archivo et Chivo Mono, et leur ajoute le seul
 * caractère dont le produit a besoin et que la police ne dessine pas : **U+202F**.
 *
 *     pnpm --filter @kaya/app polices:generer            # régénère les artefacts
 *     pnpm --filter @kaya/app polices:generer --verifier # échoue sur tout écart (CI)
 *
 * # Ce que ce script résout
 *
 * `docs/design/theme.css`, section « POLICES » : « Les maquettes chargent Archivo et Chivo Mono
 * depuis Google Fonts. En production, servez-les en local (woff2, `font-display: swap`) : le
 * produit tourne sur des liaisons lentes et doit s'afficher hors ligne. **Le nom des familles ne
 * change pas — seule la source change.** » C'est pourquoi ce script écrit ses propres `@font-face`
 * au lieu d'importer ceux de Fontsource, qui nomment les familles « Archivo Variable » et
 * « Chivo Mono Variable » : sous ces noms, les jetons `--font-titre`, `--font-texte` et
 * `--font-mono` de `@theme` ne trouveraient rien.
 *
 * Sans ces fichiers, l'application tourne sur les polices système de repli. Ce n'est pas cosmétique :
 * `docs/design/tokens.md` §2 fait de **Chivo Mono tabulaire** la condition de l'alignement des
 * colonnes de montants. Sur les polices de repli, un écran de caisse ou de clôture affiche des
 * montants désalignés.
 *
 * # U+202F — le caractère que la police amont ne dessine pas
 *
 * `tokens.md` §2, règle des montants **sans exception** : « Un montant s'écrit avec une espace fine
 * insécable **U+202F** entre les groupes de milliers *et* avant le F : `12 500 F`. »
 *
 * **Constat, par lecture des fichiers et non par supposition** : U+202F est absent des `woff2` de
 * Fontsource — et absent des `ttf` amont de Google Fonts, vérifié sur `Archivo_400Regular.ttf` et
 * `ChivoMono_400Regular.ttf`. Ce n'est donc pas un défaut de sous-réglage : **ni Archivo ni Chivo
 * Mono ne contiennent ce glyphe**. La `unicode-range` déclarée par Fontsource pour le sous-ensemble
 * `latin` couvre pourtant `U+2000-206F` : la plage annoncée n'est pas la couverture réelle, et
 * c'est exactement le genre d'écart qu'une porte doit lire dans le fichier.
 *
 * Conséquence si on ne fait rien : chaque montant du produit fait tomber son séparateur sur une
 * police de repli, de chasse inconnue — donc **les colonnes de montants ne s'alignent plus**, la
 * propriété même que `tokens.md` demande à Chivo Mono.
 *
 * **Ce que ce script fait** : il ajoute à la table `cmap` une entrée `U+202F → glyphe de U+2009`
 * (THIN SPACE), présent dans les deux polices. Aucun glyphe n'est créé ni dessiné — seule une
 * association de plus est écrite. Le choix de U+2009 est **mesuré, pas supposé** (chasses relevées
 * dans les `ttf` amont, unités de 1000) :
 *
 * | Police | U+0020 | U+00A0 | U+2009 | Ce que ça donne pour U+202F |
 * |---|---|---|---|---|
 * | Archivo (proportionnelle) | 209 | 209 | **193** | une fine, plus étroite que l'espace mot — l'usage typographique |
 * | Chivo Mono (monospace) | 600 | 600 | **600** | la cellule pleine, comme tout autre caractère — **l'alignement tabulaire est tenu** |
 *
 * En monospace, toutes les chasses valent la cellule : mapper sur U+2009 ne casse pas la grille.
 * En proportionnelle, U+2009 est la fine attendue. Un seul choix convient donc aux deux.
 *
 * L'insécabilité, elle, ne vient pas de la police mais du **caractère** : U+202F est de catégorie
 * Unicode `Zs` avec la propriété non-sécable. Elle est préservée, puisque le texte reste U+202F —
 * on ne substitue pas le caractère, on lui donne un dessin.
 *
 * # Pourquoi la variante variable, et pourquoi PAS de sous-réglage
 *
 * **Variable, mesuré** : les quatre fichiers variables (Archivo et Chivo Mono, `latin` et
 * `latin-ext`) pèsent **114,0 ko**. L'équivalent statique — les quatre graisses d'Archivo
 * réellement employées (400 par défaut, 500 `font-medium`, 600 `font-semibold`, 700 `font-bold`)
 * et les deux de Chivo Mono, sur les deux sous-ensembles — fait **douze** fichiers pour
 * **152,7 ko**. Le variable pèse 75 % du statique et ajoute une graisse sans ajouter un fichier.
 *
 * **Aucun sous-réglage de caractères, contrairement aux icônes.** Les glyphes d'icônes forment un
 * ensemble fini et connu ; le texte du produit est dynamique — noms de clients, communes, libellés
 * saisis par l'exploitant. Sous-régler produirait un caractère manquant sur un nom propre ivoirien,
 * découvert en production. On se limite aux **sous-ensembles de script** : `latin` et `latin-ext`,
 * jamais `latin` seul, parce que `latin` ne porte pas Ÿ ni les latines étendues.
 *
 * # PÉRIMÈTRE — ce que ce script lit, et ce qu'il ne lit pas
 *
 * **Il lit** les quatre `woff2` de `@fontsource-variable/{archivo,chivo-mono}` (gel §3.2) et leur
 * `unicode.json`, source des `unicode-range` — jamais écrites de mémoire.
 *
 * **Il ne lit pas** : le sous-ensemble `vietnamese`, non embarqué. Limite assumée : un caractère
 * vietnamien tomberait en repli. Le produit vise la Côte d'Ivoire ; l'ajouter coûterait 23 ko pour
 * un besoin qui n'existe pas. Il n'embarque pas non plus l'**italique** — vérifié : les trois
 * occurrences d'`italic` dans `docs/design/` sont des `not-italic`, qui la désactivent.
 *
 * # Déterminisme
 *
 * Brotli est déterministe à paramètres fixés, et les paramètres sont posés explicitement ci-dessous.
 * C'est ce qui rend le mode `--verifier` opposable : sans cette propriété, la porte échouerait au
 * hasard et serait désactivée sous trois semaines — la leçon du gel §3.2 sur le générateur de client.
 * Vérifié en régénérant deux fois de suite et en comparant les octets.
 */

import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs'
import { join } from 'node:path'

import subsetFont from 'subset-font'

// Le lecteur et l'écrivain de woff2 vivent à part : la porte P-21b les emploie aussi, pour relire
// ce que ce script écrit. Recopiés ici, ils dériveraient de ce que la porte sait lire.
import { cmapDe, couvertPar, ecrireCmap, ecrireWoff2, lireCmap, lireWoff2 } from './woff2.ts'

const APP = new URL('..', import.meta.url).pathname
const VERIFIER = process.argv.includes('--verifier')

/** Espace fine insécable — `tokens.md` §2, règle des montants. */
const FINE_INSECABLE = 0x202f
/** Espace fine sécable — présente dans les deux polices, c'est elle qui fournit le dessin. */
const FINE = 0x2009

/**
 * Les quatre fichiers embarqués. **`famille` est le nom de `theme.css`**, pas celui de Fontsource :
 * « le nom des familles ne change pas, seule la source change ».
 *
 * ⚠️ **L'ORDRE COMPTE, et il se lit à l'envers de l'intuition : `latin-ext` AVANT `latin`.**
 * Les deux plages se recouvrent — `œ` et `Œ` (U+0152-0153) sont annoncés par `latin` *et* par le
 * `U+0100-02BA` de `latin-ext`, alors que seul `latin` les dessine. Quand plusieurs `@font-face`
 * d'une même famille couvrent un caractère avec le même style et la même graisse, **c'est le
 * dernier déclaré qui l'emporte** (CSS Fonts 4, sélection par `unicode-range`). Déclarer `latin`
 * en premier enverrait donc `œ` chercher son dessin dans `latin-ext`, qui ne l'a pas : « cœur »
 * s'afficherait avec un caractère de repli au milieu. C'est l'ordre que Google Fonts et Fontsource
 * emploient — vietnamese, latin-ext, latin — et il n'est pas décoratif.
 *
 * La porte **P-21b** vérifie cette règle sur le fichier gagnant, pas « quelque part dans la
 * famille » : c'est elle qui a trouvé l'ordre inverse ici.
 */
const FICHIERS = [
  { paquet: 'archivo', famille: 'Archivo', sousEnsemble: 'latin-ext', source: 'archivo-latin-ext-wght-normal.woff2', sortie: 'archivo-latin-ext-kaya.woff2' },
  { paquet: 'archivo', famille: 'Archivo', sousEnsemble: 'latin', source: 'archivo-latin-wght-normal.woff2', sortie: 'archivo-latin-kaya.woff2' },
  { paquet: 'chivo-mono', famille: 'Chivo Mono', sousEnsemble: 'latin-ext', source: 'chivo-mono-latin-ext-wght-normal.woff2', sortie: 'chivo-mono-latin-ext-kaya.woff2' },
  { paquet: 'chivo-mono', famille: 'Chivo Mono', sousEnsemble: 'latin', source: 'chivo-mono-latin-wght-normal.woff2', sortie: 'chivo-mono-latin-kaya.woff2' },
] as const

const CSS_SORTIE = 'assets/css/polices.css'

// =================================================================================================
//  Produire les quatre fichiers
// =================================================================================================


const attendus = new Map<string, Buffer>()
const blocs: string[] = []
/** Fichiers de sortie qui portent effectivement U+202F — contrôle de complétude en fin de course. */
const porteLaFine = new Set<string>()

console.log('── Polices · U+202F et sous-ensembles de script ───────────────────────────────')

for (const fichier of FICHIERS) {
  const racinePaquet = join(APP, 'node_modules/@fontsource-variable', fichier.paquet)
  const source = join(racinePaquet, 'files', fichier.source)
  if (!existsSync(source)) {
    console.error(`  ✗ ${fichier.source} absent — installer @fontsource-variable/${fichier.paquet} (gel §3.2).`)
    process.exit(1)
  }

  const plages = JSON.parse(readFileSync(join(racinePaquet, 'unicode.json'), 'utf8')) as Record<string, string>
  const plage = plages[fichier.sousEnsemble]
  if (!plage) {
    console.error(`  ✗ ${fichier.paquet}/unicode.json n'a pas de plage « ${fichier.sousEnsemble} ».`)
    process.exit(1)
  }

  const octetsSource = readFileSync(source)
  const police = lireWoff2(octetsSource, fichier.source)
  const table = police.tables.find(t => t.etiquette === 'cmap')
  if (!table) {
    console.error(`  ✗ ${fichier.source} n'a pas de table cmap.`)
    process.exit(1)
  }
  if (table.longueur !== table.origine) {
    console.error(`  ✗ ${fichier.source} : la table cmap est transformée — cas non prévu.`)
    process.exit(1)
  }

  const associations = lireCmap(table.donnees)
  const avant = associations.size

  // U+202F n'est ajouté qu'au fichier dont la `unicode-range` le couvre — le sous-ensemble `latin`,
  // qui déclare `U+2000-206F`. `latin-ext` ne le couvre pas : l'y mettre serait un glyphe que le
  // navigateur ne demandera jamais à ce fichier, donc une porte verte sur un montant en repli.
  const aBesoinDeLaFine = couvertPar(plage, FINE_INSECABLE)
  let glypheFine: number | undefined

  if (aBesoinDeLaFine) {
    glypheFine = associations.get(FINE)
    if (glypheFine === undefined) {
      // Sans U+2009, il n'y a plus de dessin de fine à réutiliser, et la substitution devrait se
      // rabattre sur l'espace mot — ce qui changerait la typographie des montants en silence.
      console.error(`  ✗ ${fichier.source} : U+2009 absent, aucun dessin de fine à associer à U+202F.`)
      process.exit(1)
    }
    associations.set(FINE_INSECABLE, glypheFine)
    porteLaFine.add(fichier.sortie)
  }

  table.donnees = ecrireCmap(associations)
  table.origine = table.donnees.length
  table.longueur = table.donnees.length

  const woff2 = ecrireWoff2(police)
  attendus.set(join('assets/fonts', fichier.sortie), woff2)

  // Relecture du fichier PRODUIT, pas de l'objet en mémoire : les associations d'origine sont-elles
  // toutes là, et U+202F en plus ? Un ré-encodeur qui perd des segments le perdrait en silence.
  const attendu = avant + (aBesoinDeLaFine ? 1 : 0)
  const controle = lireCmap(cmapDe(woff2, fichier.sortie))
  if (controle.size !== attendu || (aBesoinDeLaFine && !controle.has(FINE_INSECABLE))) {
    console.error(`  ✗ ${fichier.sortie} : ${attendu} association(s) attendues, ${controle.size} relues.`)
    process.exit(1)
  }

  console.log(
    `  ${fichier.famille.padEnd(11)} ${fichier.sousEnsemble.padEnd(10)}`
    + ` ${String(controle.size).padStart(4)} caractère(s) · ${(woff2.length / 1024).toFixed(1)} ko`
    + ` (source ${(octetsSource.length / 1024).toFixed(1)} ko) · `
    + (aBesoinDeLaFine ? `U+202F → glyphe ${glypheFine}` : 'U+202F hors de sa plage, non ajouté'),
  )

  blocs.push(
    `/* ${fichier.famille} · ${fichier.sousEnsemble} */`,
    `@font-face {`,
    `  font-family: "${fichier.famille}";`,
    `  font-style: normal;`,
    // Un seul fichier couvre 100→900 : c'est l'axe `wght` de la police variable.
    `  font-weight: 100 900;`,
    // `swap` et non `block` : le texte doit être lisible tout de suite sur une liaison lente. Le
    // contraire des icônes, où `block` évite qu'un carré de substitution change de forme.
    `  font-display: swap;`,
    `  src: url("../fonts/${fichier.sortie}") format("woff2");`,
    `  unicode-range: ${plage};`,
    `}`,
    ``,
  )
}

// =================================================================================================
//  4. Contrôle par un tiers — harfbuzz, via subset-font
// =================================================================================================

// Le décodeur ci-dessus est le mien : il relirait ses propres erreurs sans les voir. harfbuzz est
// le moteur de mise en forme employé par les navigateurs. S'il ouvre le fichier ET retient U+202F
// en sous-réglant sur un montant réel, la police est valide pour un rendu — pas seulement pour moi.
console.log('── Polices · contrôle par harfbuzz ────────────────────────────────────────────')

// Chaque famille de `theme.css` doit porter U+202F dans au moins un de ses fichiers : Chivo Mono
// pour les colonnes de montants, Archivo parce qu'un montant s'écrit aussi dans une phrase.
for (const famille of new Set(FICHIERS.map(f => f.famille))) {
  const sesFichiers = FICHIERS.filter(f => f.famille === famille)
  if (!sesFichiers.some(f => porteLaFine.has(f.sortie))) {
    console.error(`  ✗ ${famille} : aucun de ses fichiers ne porte U+202F.`)
    process.exit(1)
  }
}

const MONTANT = `12${String.fromCodePoint(FINE_INSECABLE)}500${String.fromCodePoint(FINE_INSECABLE)}F`

for (const fichier of FICHIERS) {
  const relatif = join('assets/fonts', fichier.sortie)
  const reduit = await subsetFont(attendus.get(relatif)!, MONTANT, { targetFormat: 'woff2' })
  const controle = lireCmap(cmapDe(Buffer.from(reduit), `${relatif} sous-réglé par harfbuzz`))
  if (porteLaFine.has(fichier.sortie) && !controle.has(FINE_INSECABLE)) {
    console.error(`  ✗ ${relatif} : harfbuzz n'a pas retenu U+202F en sous-réglant « 12 500 F ».`)
    process.exit(1)
  }
  console.log(
    `  ✓ ${relatif} — harfbuzz ouvre le fichier`
    + (porteLaFine.has(fichier.sortie) ? ' et retient U+202F sur « 12 500 F »' : ''),
  )
}

// =================================================================================================
//  5. Écrire ou vérifier
// =================================================================================================

const enTete = `/* ═══════════════════════════════════════════════════════════════════════════
   KAYA — POLICES EMBARQUÉES  ·  FICHIER GÉNÉRÉ, NE PAS ÉDITER À LA MAIN

   Produit par \`app/scripts/generer-polices.ts\` depuis
   @fontsource-variable/archivo et @fontsource-variable/chivo-mono (gel §3.2).

       pnpm --filter @kaya/app polices:generer

   PORTE P-21 — aucune ressource d'hôte externe : les maquettes chargent ces
   deux familles depuis Google Fonts, le produit ne le fait JAMAIS.
   PORTE P-21b — ce qui est déclaré est effectivement embarqué : les familles
   des jetons \`--font-titre\`, \`--font-texte\` et \`--font-mono\` de \`@theme\`
   sont servies ici, en local.

   \`theme.css\`, section « POLICES » : « le nom des familles ne change pas —
   seule la source change ». D'où ces \`@font-face\` écrits ici plutôt que les
   feuilles de Fontsource, qui nomment les familles « Archivo Variable » et
   « Chivo Mono Variable ».

   U+202F, l'espace fine insécable des montants (\`tokens.md\` §2), n'existe
   NI dans Archivo NI dans Chivo Mono à la source. Le script l'ajoute à la
   table cmap, associé au dessin de U+2009. Sans cela, chaque montant fait
   tomber son séparateur sur une police de repli et les colonnes de la
   clôture se désalignent.
   ═══════════════════════════════════════════════════════════════════════════ */

`

attendus.set(CSS_SORTIE, Buffer.from(enTete + blocs.join('\n')))

let echec = false

for (const [relatif, contenu] of attendus) {
  const chemin = join(APP, relatif)

  if (VERIFIER) {
    if (!existsSync(chemin)) {
      console.error(`  ✗ ${relatif} absent — exécuter \`pnpm --filter @kaya/app polices:generer\`.`)
      echec = true
      continue
    }
    if (Buffer.compare(readFileSync(chemin), contenu) !== 0) {
      console.error(`  ✗ ${relatif} diffère de ce que les sources produisent.`)
      echec = true
      continue
    }
    console.log(`  ✓ ${relatif} à jour`)
    continue
  }

  mkdirSync(join(chemin, '..'), { recursive: true })
  writeFileSync(chemin, contenu)
  console.log(`  → ${relatif} (${(contenu.length / 1024).toFixed(1)} ko)`)
}

if (echec) {
  console.error('')
  console.error("Les polices embarquées ne correspondent plus à leurs sources. L'application")
  console.error('tournerait sur les polices système de repli — et les colonnes de montants,')
  console.error('que `tokens.md` §2 confie à Chivo Mono tabulaire, se désaligneraient.')
  process.exit(1)
}

console.log(VERIFIER ? 'Polices ✓ — embarquement à jour.' : 'Polices ✓ — embarquement régénéré.')
