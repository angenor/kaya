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

console.log('── P-16 · 1/2 — parité des catalogues ────────────────────────────────────────')

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

const fichiers = fichiersVue(RACINE)
console.log(`  ${fichiers.length} fichier(s) .vue analysé(s)`)

for (const fichier of fichiers) {
  const contenu = readFileSync(fichier, 'utf8')
  const relatif = relative(RACINE, fichier)

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

// =================================================================================================

if (echec) {
  console.error('')
  console.error('P-16 ÉCHOUE.')
  process.exit(1)
}

console.log('P-16 ✓ — catalogues à parité, aucun littéral détecté dans les templates.')
console.log('  Limite assumée : la détection des littéraux est heuristique. Elle attrape le cas')
console.log('  courant ; la revue couvre le reste (voir l’en-tête de ce fichier).')
