/**
 * **Le styleguide est-il vraiment hors production ?**
 *
 * Toute la valeur de cette page tient à une chose : elle est servie par l'**application**, avec les
 * polices réellement embarquées, donc elle peut montrer ce qu'aucun test ne voit. Et tout son
 * risque tient à la même chose : c'est une page de développement, avec des libellés d'échantillon
 * en clair, exemptée de la porte P-16.
 *
 * Le mécanisme qui rend l'un acceptable sans perdre l'autre est repris de la Swagger UI du
 * cycle 001 : **la route est retirée du routeur**, pas cachée derrière un `v-if`. Ce fichier vérifie
 * les deux moitiés — la décision, et son câblage.
 *
 * # Ce qu'il ne peut pas vérifier
 *
 * Que Nuxt applique effectivement le hook. Cela demanderait une construction complète pour chaque
 * assertion. Ce qui est vérifié ici, c'est que la décision est **juste** et que `nuxt.config.ts`
 * l'**emploie** ; que le hook `pages:extend` fasse ce qu'il annonce relève du framework, et le
 * constat visuel du volet suivant l'a confirmé sur une construction réelle.
 */

import { readFileSync } from 'node:fs'
import { join } from 'node:path'

import { describe, expect, it } from 'vitest'

import { ROUTE_STYLEGUIDE, styleguideMonte, VARIABLE_STYLEGUIDE } from '../core/design-system/montage'

const RACINE = new URL('..', import.meta.url).pathname
const CONFIG = readFileSync(join(RACINE, 'nuxt.config.ts'), 'utf8')

// =================================================================================================
//  La décision — faux par défaut, comme swagger_ui_activee()
// =================================================================================================

describe('la décision de montage', () => {
  it('est FAUSSE quand la variable est absente — le défaut sûr', () => {
    // Un défaut « monté sauf si » ferait dépendre la production d'une variable correctement posée
    // sur chaque déploiement. C'est le sens de la garde du cycle 001, repris ici.
    expect(styleguideMonte({})).toBe(false)
  })

  it('n’accepte que « 1 » et « true »', () => {
    expect(styleguideMonte({ [VARIABLE_STYLEGUIDE]: '1' })).toBe(true)
    expect(styleguideMonte({ [VARIABLE_STYLEGUIDE]: 'true' })).toBe(true)
  })

  it('refuse tout le reste, y compris ce qui ressemble à un oui', () => {
    for (const valeur of ['', '0', 'false', 'oui', 'yes', 'TRUE', 'on', ' 1']) {
      expect(styleguideMonte({ [VARIABLE_STYLEGUIDE]: valeur }), valeur).toBe(false)
    }
  })

  it('ne lit QUE sa variable — une autre valeur d’environnement ne la monte pas', () => {
    expect(styleguideMonte({ NODE_ENV: 'development', KAYA_SWAGGER_UI: '1' })).toBe(false)
  })
})

// =================================================================================================
//  Le câblage — la décision est-elle employée là où elle compte ?
// =================================================================================================

describe('le câblage dans nuxt.config.ts', () => {
  it('retire la page du routeur au lieu de la cacher derrière une garde de rendu', () => {
    // « Une route non montée ne peut pas fuir par oubli de garde ; une route montée derrière un
    // `if` finit toujours par être atteinte par un chemin qu'on n'avait pas prévu. »
    expect(CONFIG).toContain('\'pages:extend\'')
    expect(CONFIG).toContain('pages.splice(')
    expect(CONFIG).toContain('styleguideMonte(')
  })

  it('n’écrit ni la route ni le nom de la variable en dur — une seconde copie dériverait', () => {
    expect(CONFIG).toContain('ROUTE_STYLEGUIDE')
    expect(CONFIG).toContain('VARIABLE_STYLEGUIDE')
    // La chaîne littérale n'apparaît que dans `montage.ts`, jamais ici.
    expect(CONFIG).not.toContain('\'/styleguide\'')
  })

  it('la route déclarée correspond au fichier de page', () => {
    // `pages/styleguide.vue` produit `/styleguide` : si l'un des deux est renommé sans l'autre, le
    // `splice` ne trouve plus rien et la page part en production **sans que rien n'échoue**.
    expect(ROUTE_STYLEGUIDE).toBe('/styleguide')
    expect(() => readFileSync(join(RACINE, 'pages/styleguide.vue'), 'utf8')).not.toThrow()
  })
})

// =================================================================================================
//  Ce que la page doit montrer pour servir à quelque chose
// =================================================================================================

describe('la page elle-même', () => {
  const PAGE = readFileSync(join(RACINE, 'pages/styleguide.vue'), 'utf8')

  it('écrit ses montants par la fonction unique — sauf DEUX, les contre-exemples', () => {
    // Un montant écrit à la main dans le styleguide afficherait la bonne chose **ici** et mentirait
    // sur ce que le produit fera : c'est exactement le défaut que cette page existe pour révéler.
    expect(PAGE).toContain('formaterMontant(')

    // **Deux exceptions, et elles sont le sujet de la démonstration** : la section « montants »
    // affiche « 12 500 F » avec l'espace ORDINAIRE sous le même montant écrit avec U+202F —
    // **une fois en Archivo, une fois en Chivo Mono**, parce que les deux polices ne racontent pas
    // la même chose. En Archivo la fine est un peu plus étroite (193 unités contre 209) ; en Chivo
    // Mono elle vaut la cellule comme tout le reste, et c'est précisément ce qui aligne la colonne.
    // Une seule des deux paires laisserait croire que la fine « ne marche pas » dans l'autre police.
    //
    // Les compter plutôt que les interdire évite à la fois de perdre la démonstration et de laisser
    // entrer un troisième montant en dur.
    // U+202F est écrit en échappement : à l'œil, dans un éditeur, il est indistinguable de
    // l'espace ordinaire — et `no-irregular-whitespace` le refuse en clair, à juste titre.
    const enClair = [...PAGE.matchAll(/\d[\s\u202F]\d{3}[\s\u202F]F</g)]

    expect(enClair, enClair.map(m => m[0]).join(' · ')).toHaveLength(2)
  })

  it('monte le vrai composant 16, pas une imitation en classes', () => {
    // Les quinze autres composants n'existent qu'en classes Tailwind — c'est la règle de
    // `core/design-system/README.md`. Le seizième est un composant Vue, et le styleguide doit
    // exercer celui-là, sans quoi il vérifierait une copie.
    expect(PAGE).toContain('ChampSaisie')
    expect(PAGE).toContain('etiquette-cle')
  })

  it('affiche les deux thèmes côte à côte — Definition of Done, point 8', () => {
    expect(PAGE).toContain('VitrineTheme')
  })

  it('couvre les seize composants canoniques', () => {
    const ancres = [...PAGE.matchAll(/id="(c\d+)"/g)].map(m => m[1])

    expect(new Set(ancres).size).toBe(16)
    for (let n = 1; n <= 16; n += 1) expect(ancres, `composant ${n}`).toContain(`c${n}`)
  })

  it('porte la section « montants », avec les quatre largeurs qui révèlent un désalignement', () => {
    expect(PAGE).toContain('id="montants"')
    // Quatre, cinq, six, sept chiffres : c'est le passage de l'un à l'autre qui met la chasse
    // tabulaire à l'épreuve. Une colonne de montants de même largeur ne prouverait rien.
    for (const montant of ['1_500', '12_500', '150_000', '1_250_000']) {
      expect(PAGE, montant).toContain(montant)
    }
  })
})
