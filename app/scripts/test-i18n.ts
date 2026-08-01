/**
 * **Porte P-16** — aucune chaîne utilisateur en dur ; parité des clés `fr` et `en`.
 *
 *     pnpm --filter @kaya/app test:i18n
 *
 * # Deux vérifications, deux fautes différentes
 *
 * 1. **Parité des catalogues.** Une clé présente en `fr` et absente en `en` produit un libellé
 *    manquant à l'exécution — visible seulement par quelqu'un qui utilise l'application en
 *    anglais, c'est-à-dire personne pendant les six premiers mois.
 *
 * 2. **Littéraux affichés dans les templates.** C'est la faute la plus fréquente et la plus
 *    coûteuse : rétrofiter l'i18n coûte plusieurs fois son prix initial (principe VIII), parce
 *    qu'il faut rouvrir chaque écran, retrouver chaque chaîne et inventer sa clé.
 *
 * # La limite de la seconde vérification, écrite ici plutôt qu'enfouie
 *
 * La détection des littéraux est une **heuristique**. Elle repère le texte visible entre balises
 * et dans les attributs porteurs de libellé ; elle ne peut pas distinguer un libellé d'une valeur
 * technique dans tous les cas. Elle attrape le cas courant — du texte français écrit à la main
 * dans un template — et la revue couvre le reste.
 *
 * Prétendre le contraire produirait une porte qui ment. La signaler ici évite qu'un développeur
 * conclue de son silence que tout est externalisé.
 *
 * # Une exemption, nommée, bornée, ET vérifiée par sa contrepartie
 *
 * `pages/styleguide.vue` porte ses libellés d'échantillon **en clair** : « Repos », « Survol »,
 * « Losange — acquis, terminé ». Ce sont les noms d'états de `docs/design/composants.md`, pas des
 * chaînes produit. Deux raisons de ne pas les externaliser, dans cet ordre :
 *
 * 1. **Les catalogues sont livrés en production.** Y verser cent cinquante clés de vocabulaire de
 *    design ferait voyager du texte mort dans chaque installation, pour une page que l'utilisateur
 *    n'ouvrira jamais.
 * 2. Les traduire n'aurait pas de sens, et les maintenir à parité `fr`/`en` pour toujours encore
 *    moins.
 *
 * **Une exemption sans contrepartie serait un trou** — c'est la leçon de la porte P-17, dont
 * l'exclusion de `couleur_primaire` s'accompagne d'une assertion qui la borne. Celle-ci a la
 * sienne, et c'est le §3 ci-dessous : la porte vérifie que le fichier exempté est bien **retiré du
 * routeur** hors développement. Le jour où quelqu'un monterait le styleguide en production, ou
 * exempterait un second fichier sans cette garantie, P-16 échoue.
 */

import { readFileSync, readdirSync, statSync } from 'node:fs'
import { join, relative } from 'node:path'

const RACINE = new URL('..', import.meta.url).pathname
const CATALOGUES = join(RACINE, 'core/i18n')
const LOCALE_REFERENCE = 'fr'
const LOCALES = ['fr', 'en']

let echec = false

function signaler(message: string): void {
  console.error(`  ✗ ${message}`)
  echec = true
}

// =================================================================================================
//  1. Parité des catalogues
// =================================================================================================

function clesPlates(objet: unknown, prefixe = ''): Set<string> {
  const cles = new Set<string>()
  if (typeof objet !== 'object' || objet === null) {
    return cles
  }
  for (const [cle, valeur] of Object.entries(objet)) {
    const chemin = prefixe ? `${prefixe}.${cle}` : cle
    if (typeof valeur === 'object' && valeur !== null && !Array.isArray(valeur)) {
      for (const sous of clesPlates(valeur, chemin)) {
        cles.add(sous)
      }
    } else {
      cles.add(chemin)
    }
  }
  return cles
}

console.log('── P-16 · 1/3 — parité des catalogues ────────────────────────────────────────')

const jeux = new Map<string, Set<string>>()
for (const locale of LOCALES) {
  const chemin = join(CATALOGUES, `${locale}.json`)
  const contenu = JSON.parse(readFileSync(chemin, 'utf8'))
  jeux.set(locale, clesPlates(contenu))
  console.log(`  ${locale} : ${jeux.get(locale)!.size} clés`)
}

const reference = jeux.get(LOCALE_REFERENCE)!
for (const locale of LOCALES) {
  if (locale === LOCALE_REFERENCE) continue
  const jeu = jeux.get(locale)!

  const manquantes = [...reference].filter(c => !jeu.has(c))
  const surplus = [...jeu].filter(c => !reference.has(c))

  if (manquantes.length > 0) {
    signaler(
      `${locale} : ${manquantes.length} clé(s) manquante(s) — ${manquantes.slice(0, 10).join(', ')}`
      + (manquantes.length > 10 ? ` … (+${manquantes.length - 10})` : ''),
    )
  }
  if (surplus.length > 0) {
    // Une clé en trop n'est pas anodine : elle signale soit un oubli côté `fr`, soit une clé
    // morte que personne n'ose supprimer.
    signaler(
      `${locale} : ${surplus.length} clé(s) sans équivalent en ${LOCALE_REFERENCE} — `
      + surplus.slice(0, 10).join(', '),
    )
  }
}

// =================================================================================================
//  2. Littéraux affichés
// =================================================================================================

console.log('── P-16 · 2/2 — littéraux affichés dans les templates ────────────────────────')

function fichiersVue(repertoire: string): string[] {
  const trouves: string[] = []
  let entrees: string[]
  try {
    entrees = readdirSync(repertoire)
  } catch {
    return trouves
  }
  for (const entree of entrees) {
    if (['node_modules', '.nuxt', '.output', 'dist', 'src-tauri'].includes(entree)) continue
    const chemin = join(repertoire, entree)
    if (statSync(chemin).isDirectory()) {
      trouves.push(...fichiersVue(chemin))
    } else if (entree.endsWith('.vue')) {
      trouves.push(chemin)
    }
  }
  return trouves
}

// Au moins deux lettres consécutives : écarte les symboles, les nombres et les unités isolées.
const TEXTE_VISIBLE = /^[^<>{}\n]*[A-Za-zÀ-ÿ]{2,}[^<>{}\n]*$/
const ATTRIBUTS_LIBELLE = /\s(?:placeholder|title|aria-label|alt)="([^"]*[A-Za-zÀ-ÿ]{2,}[^"]*)"/g

/**
 * **Le seul fichier exempté du contrôle des littéraux** — surface de développement, jamais montée
 * en production. Voir l'en-tête, et la contrepartie vérifiée au §3.
 */
const EXEMPTES = new Set(['pages/styleguide.vue'])

const fichiers = fichiersVue(RACINE)
console.log(`  ${fichiers.length} fichier(s) .vue analysé(s)`)

/** Compté et affiché : une porte qui n'inspecte rien passe toujours (exigence 4). */
let inspectes = 0

for (const fichier of fichiers) {
  const contenu = readFileSync(fichier, 'utf8')
  const relatif = relative(RACINE, fichier)

  if (EXEMPTES.has(relatif)) {
    console.log(`  · ${relatif} — exempté : surface de développement, non montée en production (§3)`)
    continue
  }
  inspectes += 1

  const debut = contenu.indexOf('<template>')
  const fin = contenu.lastIndexOf('</template>')
  if (debut === -1 || fin === -1) continue
  const template = contenu.slice(debut + '<template>'.length, fin)

  // Texte entre balises : `>texte<`.
  for (const capture of template.matchAll(/>([^<>{}]+)</g)) {
    const texte = capture[1].trim()
    if (!texte || !TEXTE_VISIBLE.test(texte)) continue
    signaler(
      `${relatif} — chaîne en dur dans le template : « ${texte.slice(0, 60)} »\n`
      + '      Externaliser en clé i18n fr ET en (principe VIII). Rétrofiter coûte plusieurs '
      + 'fois le prix initial.',
    )
  }

  // Attributs porteurs de libellé.
  for (const capture of template.matchAll(ATTRIBUTS_LIBELLE)) {
    signaler(`${relatif} — attribut en dur : « ${capture[1].slice(0, 60)} »`)
  }
}

// Une porte dont la cible est vide est indistinguable d'une porte qui passe (exigence 4). Le seuil
// est bas à dessein : il ne prétend pas mesurer la couverture, seulement refuser le zéro.
if (inspectes === 0) {
  signaler('aucun fichier .vue inspecté — la porte ne garde RIEN (exigence 4).')
}

// =================================================================================================
//  3. La contrepartie de l'exemption — le fichier exempté n'atteint pas la production
// =================================================================================================

console.log('── P-16 · 3/3 — l’exemption est-elle bornée ? ────────────────────────────────')

const CONFIG_NUXT = join(RACINE, 'nuxt.config.ts')
const MONTAGE = join(RACINE, 'core/design-system/montage.ts')

for (const relatif of EXEMPTES) {
  // Le fichier existe-t-il encore ? Une exemption qui ne désigne rien passe toujours, et masque le
  // jour où le fichier revient sous un autre nom.
  const cible = join(RACINE, relatif)
  let existe = true
  try {
    statSync(cible)
  } catch {
    existe = false
  }
  if (!existe) {
    signaler(
      `${relatif} est exempté mais n'existe plus — retirer l'exemption plutôt que de la laisser\n`
      + '      désigner un fichier absent : elle protégerait le prochain qui portera ce nom.',
    )
    continue
  }

  // La route est-elle **retirée du routeur** hors développement ? C'est ce qui rend l'exemption
  // acceptable : les libellés en clair ne sont jamais servis à un utilisateur.
  const config = readFileSync(CONFIG_NUXT, 'utf8')
  const montage = readFileSync(MONTAGE, 'utf8')

  const route = `/${relatif.replace(/^pages\//, '').replace(/\.vue$/, '')}`
  const declareLaRoute = montage.includes(`'${route}'`)
  const retireLaPage = /'pages:extend'/.test(config)
    && /pages\.splice\(/.test(config)
    && /styleguideMonte\(/.test(config)

  if (!declareLaRoute || !retireLaPage) {
    signaler(
      `${relatif} est exempté du contrôle des littéraux, mais rien ne garantit qu'il reste hors\n`
      + `      production. Attendu : « ${route} » nommée dans core/design-system/montage.ts, et un\n`
      + '      hook `pages:extend` qui la retire du routeur quand KAYA_STYLEGUIDE n\'est pas posée.\n'
      + '      Sans cette garantie, l\'exemption devient un trou : du texte non traduit atteindrait\n'
      + '      un utilisateur (principe VIII).',
    )
    continue
  }

  console.log(`  ✓ ${relatif} — route « ${route} » retirée du routeur hors KAYA_STYLEGUIDE`)
}

// =================================================================================================

if (echec) {
  console.error('')
  console.error('P-16 ÉCHOUE.')
  process.exit(1)
}

console.log(
  `P-16 ✓ — catalogues à parité, ${inspectes} template(s) inspecté(s) sans littéral, `
  + `${EXEMPTES.size} exemption(s) bornée(s).`,
)
console.log('  Limite assumée : la détection des littéraux est heuristique. Elle attrape le cas')
console.log('  courant ; la revue couvre le reste (voir l’en-tête de ce fichier).')
