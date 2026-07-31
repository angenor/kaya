/**
 * **Definition of Done, point 8 — `G1` vérifié en mode clair ET en mode sombre.**
 *
 * Le point était **sans objet au cycle 001** faute d'écran ; il devient exigible ici, et c'est la
 * dette que ce cycle solde.
 *
 * # Ce que ce fichier vérifie, et ce qu'il ne peut pas vérifier
 *
 * **Vérifié, mécaniquement** — que chaque jeton de couleur employé par les composants de `G1`
 * porte bien **une valeur sous `.dark`**. C'est la condition qui fait qu'un composant bascule
 * « tout seul » : les noms de jetons sont identiques dans les deux thèmes, seules les valeurs
 * changent (`theme.css`, règle de lecture 1). Un jeton défini en clair mais absent du bloc sombre
 * garde sa valeur claire — texte foncé sur fond foncé, et l'écart ne se voit qu'en basculant.
 *
 * **Non vérifié ici** — le **rendu visuel** : contrastes, lisibilité, harmonie. Aucun test ne
 * peut en juger. Cette partie a été faite à l'œil, section par section (T044), et ce fichier
 * garantit qu'elle ne régressera pas silencieusement.
 *
 * # Aucune palette dupliquée
 *
 * Le principe XII interdit une seconde palette : le mode sombre passe par la variante `dark:` et
 * par des jetons dont les valeurs changent. Ce fichier vérifie donc aussi qu'aucun composant
 * n'emploie de classe `dark:bg-…` ou `dark:text-…` **de couleur** — la variante `dark:` ne sert
 * qu'à ce qu'une couleur ne peut pas porter : une ombre remplacée par une bordure, une opacité,
 * une épaisseur de trait.
 */

import { readFileSync, readdirSync } from 'node:fs'
import { join } from 'node:path'

import { describe, expect, it } from 'vitest'

const RACINE = new URL('..', import.meta.url).pathname
const THEME = readFileSync(join(RACINE, 'assets/css/theme.css'), 'utf8')

/** Le bloc `.dark { … }` de `theme.css`. */
function blocSombre(): string {
  const debut = THEME.indexOf('.dark {')
  expect(debut, 'le bloc `.dark` a disparu de theme.css — le mode sombre n’existe plus').toBeGreaterThan(-1)
  const fin = THEME.indexOf('\n}', debut)
  return THEME.slice(debut, fin)
}

/** Les fichiers de `G1`. */
function composantsG1(): { nom: string, contenu: string }[] {
  const repertoire = join(RACINE, 'modules/etablissements')
  return readdirSync(repertoire)
    .filter((f) => f.endsWith('.vue'))
    .map((nom) => ({ nom, contenu: readFileSync(join(repertoire, nom), 'utf8') }))
}

/**
 * Les jetons de **couleur** employés par un composant.
 *
 * # Trois pièges d'extraction, neutralisés ici
 *
 * 1. **Les préfixes directionnels.** `border-l-line-2` porte le jeton `line-2`, pas `l-line-2`.
 *    Sans traitement, chaque bordure gauche produirait un faux jeton introuvable dans le thème.
 * 2. **Les largeurs de bordure.** `border-l-4` et `border-t` ne nomment aucune couleur — la
 *    première est une épaisseur, la seconde un simple trait. Les compter ferait échouer la porte
 *    sur des utilitaires parfaitement corrects.
 * 3. **Les échelles de taille.** `text-corps`, `text-titre-s` sont des corps de texte, pas des
 *    couleurs, et n'ont rien à faire dans le bloc sombre.
 *
 * Une extraction qui produit des faux positifs est une porte qu'on désactive dans la semaine.
 */
function jetonsDeCouleur(contenu: string): Set<string> {
  const trouves = new Set<string>()
  const motif = /\b(?:bg|text|border|fill|stroke)(?:-(?:l|r|t|b|x|y|s|e))?-([a-z][a-z0-9-]*)\b/g

  /** Corps de texte, graisses, arrondis — tout ce qui n'est pas une couleur. */
  const NON_COULEURS
    = /^(etiquette|mini|corps|action|lead|titre|chiffre|montant|recette|total|annonce|affiche|left|right|center|semibold|bold|medium|normal|xs|sm|base|lg|xl|pleine|transparent|current|inherit)/

  for (const capture of contenu.matchAll(motif)) {
    const jeton = capture[1]
    if (!jeton) continue
    // Largeur de bordure. `border-l-4` capture « l-4 » et `border-b` capture « b » : le moteur
    // d'expressions régulières préfère la capture longue au groupe directionnel optionnel. Le
    // filtre porte donc sur la FORME du jeton — une direction seule, ou une direction suivie d'une
    // épaisseur — plutôt que sur une regex plus retorse qui se casserait au premier utilitaire
    // inattendu.
    if (/^[lrtbxyse](-\d+)?$/.test(jeton)) continue
    if (/^\d/.test(jeton)) continue
    if (NON_COULEURS.test(jeton)) continue
    trouves.add(jeton)
  }
  return trouves
}

describe('G1 — mode clair et mode sombre', () => {
  it('chaque jeton de couleur employé porte une valeur sous `.dark`', () => {
    const sombre = blocSombre()
    const manquants: string[] = []

    for (const { nom, contenu } of composantsG1()) {
      for (const jeton of jetonsDeCouleur(contenu)) {
        if (!sombre.includes(`--color-${jeton}:`)) {
          manquants.push(`${nom} — jeton « ${jeton} »`)
        }
      }
    }

    expect(
      manquants,
      'Ces jetons n’ont aucune valeur sous `.dark` : ils garderont leur valeur CLAIRE en mode '
      + 'sombre — texte foncé sur fond foncé, et l’écart ne se voit qu’en basculant le thème.\n  '
      + manquants.join('\n  '),
    ).toEqual([])
  })

  it('aucune palette dupliquée — la variante `dark:` ne porte aucune couleur', () => {
    const fautes: string[] = []

    for (const { nom, contenu } of composantsG1()) {
      // `dark:` suivi d'un utilitaire de COULEUR. La variante reste légitime pour une ombre, une
      // opacité ou une épaisseur de trait — ce qu'une couleur ne peut pas porter.
      for (const capture of contenu.matchAll(
        /\bdark:(?:bg|text|border|fill|stroke)-[a-z][a-z0-9-]*/g,
      )) {
        fautes.push(`${nom} — « ${capture[0]} »`)
      }
    }

    expect(
      fautes,
      'Le mode sombre passe par des jetons dont les VALEURS changent sous `.dark`, jamais par une '
      + 'seconde palette (principe XII). Une classe `dark:` de couleur crée une seconde source, '
      + 'et les deux divergent à la première évolution du thème.\n  '
      + fautes.join('\n  '),
    ).toEqual([])
  })

  it('les cinq composants de G1 sont bien couverts par ce contrôle', () => {
    // Sans cette assertion, renommer un fichier ferait passer le test en n'inspectant plus rien —
    // le défaut exact que la constitution décrit sous « une porte qui ne trouve jamais rien ».
    const noms = composantsG1().map((c) => c.nom).sort()

    expect(noms).toEqual([
      'EcranEtablissement.vue',
      'SectionIdentite.vue',
      'SectionIdentiteVisuelle.vue',
      'SectionPointsDeVente.vue',
      'SectionServices.vue',
    ])
  })
})
