/**
 * **La règle des montants de `tokens.md` §2, vérifiée par code de caractère.**
 *
 * # Pourquoi ce fichier ne peut pas se contenter de comparer des chaînes
 *
 * `'12 500 F'` et `'12 500 F'` sont visuellement identiques dans un diff, dans un terminal et dans
 * un éditeur. Le premier porte U+202F, le second l'espace ordinaire U+0020 — et **seul le premier
 * empêche le montant de se couper en fin de ligne et aligne les colonnes en Chivo Mono tabulaire**.
 * Un test qui écrit la valeur attendue en clair passerait donc au vert sur la mauvaise, et la faute
 * ne se verrait qu'à la clôture, sur un écran de caisse, en production.
 *
 * Les assertions vont donc chercher le **code du caractère**, jamais son apparence.
 *
 * # Ce que ce fichier vérifie
 *
 * 1. Les séparateurs sont aux bons endroits — et seulement là.
 * 2. Le caractère est U+202F, ni U+0020, ni U+00A0, ni U+2009.
 * 3. XOF a **zéro** décimale : les unités mineures s'écrivent telles quelles.
 * 4. Une devise à **deux** décimales sort un autre résultat pour le même entier — c'est ce qui
 *    prouve que la devise est **lue**, et non supposée. Avec la seule XOF, une fonction qui ignore
 *    la devise et une fonction qui la lit produisent exactement la même chose.
 * 5. **Il n'existe qu'une seule implémentation** dans toute l'application, et aucun formateur
 *    d'heure ne s'est glissé à côté avec la fine.
 */

import { readFileSync, readdirSync, statSync } from 'node:fs'
import { join, relative } from 'node:path'

import { describe, expect, it } from 'vitest'

import { deviseDe, formaterMontant, MontantNonFormatable } from '../core/format/montant'

const RACINE = new URL('..', import.meta.url).pathname

/** Les quatre espaces qu'on peut confondre à l'œil, et une seule est la bonne. */
const FINE_INSECABLE = 0x202f
const ESPACE_ORDINAIRE = 0x0020
const INSECABLE_ORDINAIRE = 0x00a0
const FINE_SECABLE = 0x2009

/** Les points de code d'une chaîne, pour assertions sur le caractère et non sur l'apparence. */
function codes(chaine: string): number[] {
  return [...chaine].map(c => c.codePointAt(0)!)
}

// =================================================================================================
//  Les séparateurs, et le caractère qui les porte
// =================================================================================================

describe('la fine insécable, vérifiée par code de caractère', () => {
  it('sépare les milliers ET précède le symbole — deux U+202F pour « 12 500 F »', () => {
    const ecrit = formaterMontant(12_500, 'XOF')

    // `1 2 ␦ 5 0 0 ␦ F` — les positions comptent autant que le caractère.
    expect(codes(ecrit)).toEqual([
      0x31, 0x32,
      FINE_INSECABLE,
      0x35, 0x30, 0x30,
      FINE_INSECABLE,
      0x46,
    ])
  })

  it('n’emploie AUCUNE des trois espaces avec lesquelles U+202F se confond', () => {
    const ecrit = formaterMontant(1_250_000, 'XOF')
    const presents = new Set(codes(ecrit))

    expect(presents.has(FINE_INSECABLE)).toBe(true)
    // U+0020 se coupe en fin de ligne ; U+00A0 ne se coupe pas mais a la chasse d'une espace mot,
    // ce qui casse la fine typographique ; U+2009 est la fine, mais SÉCABLE.
    expect(presents.has(ESPACE_ORDINAIRE)).toBe(false)
    expect(presents.has(INSECABLE_ORDINAIRE)).toBe(false)
    expect(presents.has(FINE_SECABLE)).toBe(false)
  })

  it('groupe par trois depuis la droite, jamais en tête du nombre', () => {
    // Les quatre largeurs de la colonne du styleguide : c'est sur elles que le désalignement se voit.
    const fine = String.fromCodePoint(FINE_INSECABLE)

    expect(formaterMontant(1_500, 'XOF')).toBe(`1${fine}500${fine}F`)
    expect(formaterMontant(12_500, 'XOF')).toBe(`12${fine}500${fine}F`)
    expect(formaterMontant(150_000, 'XOF')).toBe(`150${fine}000${fine}F`)
    expect(formaterMontant(1_250_000, 'XOF')).toBe(`1${fine}250${fine}000${fine}F`)
  })

  it('ne sépare rien en dessous de mille — « 150 F » n’a qu’une fine, celle du symbole', () => {
    const ecrit = formaterMontant(150, 'XOF')

    expect(codes(ecrit).filter(c => c === FINE_INSECABLE)).toHaveLength(1)
    expect(ecrit).toBe(`150${String.fromCodePoint(FINE_INSECABLE)}F`)
  })

  it('écrit zéro, et le sépare quand même de son symbole', () => {
    expect(formaterMontant(0, 'XOF')).toBe(`0${String.fromCodePoint(FINE_INSECABLE)}F`)
  })

  it('porte le signe d’un avoir en tête, sans le séparer du nombre', () => {
    const fine = String.fromCodePoint(FINE_INSECABLE)

    // Un avoir se contre-passe par un montant négatif (principe V) : le cas arrivera, autant qu'il
    // soit écrit plutôt que découvert.
    expect(formaterMontant(-12_500, 'XOF')).toBe(`-12${fine}500${fine}F`)
  })
})

// =================================================================================================
//  La devise est LUE, pas supposée
// =================================================================================================

describe('le nombre de décimales et le symbole viennent de la devise', () => {
  it('XOF a zéro décimale : les unités mineures s’écrivent telles quelles', () => {
    expect(deviseDe('XOF').decimales).toBe(0)
    expect(formaterMontant(12_500, 'XOF')).toBe(`12${String.fromCodePoint(FINE_INSECABLE)}500${String.fromCodePoint(FINE_INSECABLE)}F`)
  })

  it('une devise à deux décimales donne un AUTRE résultat pour le même entier', () => {
    const fine = String.fromCodePoint(FINE_INSECABLE)

    // 1250 unités mineures : mille deux cent cinquante francs, mais douze euros cinquante. C'est
    // l'assertion qui distingue une fonction qui lit la devise d'une fonction qui suppose XOF —
    // avec la seule XOF au référentiel, les deux seraient indiscernables.
    expect(formaterMontant(1_250, 'XOF')).toBe(`1${fine}250${fine}F`)
    expect(formaterMontant(1_250, 'EUR')).toBe(`12,50${fine}€`)
  })

  it('complète les décimales à gauche — 5 unités mineures font « 0,05 », pas « 0,5 »', () => {
    const fine = String.fromCodePoint(FINE_INSECABLE)

    expect(formaterMontant(5, 'EUR')).toBe(`0,05${fine}€`)
    expect(formaterMontant(100, 'EUR')).toBe(`1,00${fine}€`)
  })

  it('groupe les milliers de la partie entière, pas les décimales', () => {
    const fine = String.fromCodePoint(FINE_INSECABLE)

    // 1 234 567 centimes = 12 345,67 € : la fine sépare « 12 » de « 345 », et rien après la virgule.
    expect(formaterMontant(1_234_567, 'EUR')).toBe(`12${fine}345,67${fine}€`)
  })

  it('le symbole du XOF est « F » — ni « FCFA », ni « XOF », ni « ₣ »', () => {
    // `tokens.md` §2 et les vingt-neuf fichiers de maquette écrivent tous « 12 500 F ».
    expect(deviseDe('XOF').symbole).toBe('F')
    expect(formaterMontant(1_500, 'XOF').endsWith('F')).toBe(true)
    expect(formaterMontant(1_500, 'XOF')).not.toContain('CFA')
  })
})

// =================================================================================================
//  Ce qui est refusé explicitement, jamais ignoré
// =================================================================================================

describe('les refus', () => {
  it('refuse une devise hors référentiel plutôt que d’inventer ses décimales', () => {
    // Se rabattre sur deux décimales afficherait « 125,00 » là où il faut « 12 500 » : un montant
    // faux d'un facteur cent, que rien ne signalerait.
    expect(() => formaterMontant(12_500, 'USD')).toThrow(MontantNonFormatable)
    expect(() => deviseDe('')).toThrow(MontantNonFormatable)
  })

  it('refuse un montant qui n’est pas un entier d’unité mineure', () => {
    // Le versant applicatif de la porte P-10. Un décimal ici vient presque toujours d'un calcul
    // fait en unités majeures — c'est le calcul qu'il faut reprendre, pas l'affichage.
    expect(() => formaterMontant(12_500.5, 'XOF')).toThrow(MontantNonFormatable)
    expect(() => formaterMontant(Number.NaN, 'XOF')).toThrow(MontantNonFormatable)
    expect(() => formaterMontant(Number.POSITIVE_INFINITY, 'XOF')).toThrow(MontantNonFormatable)
  })

  it('dit dans son message ce qu’il attendait — un refus muet se contourne par un try/catch', () => {
    expect(() => formaterMontant(1, 'GBP')).toThrow(/XOF/)
  })
})

// =================================================================================================
//  UNE SEULE fonction la porte — le versant positif de la règle
// =================================================================================================

/**
 * Les fichiers de l'application où un montant pourrait s'écrire.
 *
 * `scripts/` est hors périmètre : `generer-polices.ts` manipule U+202F **comme un point de code à
 * embarquer**, pas comme un séparateur d'affichage. `assets/css/polices.css` en parle de même dans
 * son en-tête. Les inclure ferait échouer le contrôle sur la seule chose qui rend U+202F possible.
 */
function fichiersApplicatifs(): string[] {
  const trouves: string[] = []
  const parcourir = (repertoire: string): void => {
    let entrees: string[]
    try {
      entrees = readdirSync(repertoire)
    }
    catch {
      return
    }
    for (const entree of entrees) {
      const chemin = join(repertoire, entree)
      if (statSync(chemin).isDirectory()) parcourir(chemin)
      else if (/\.(vue|ts)$/.test(entree)) trouves.push(chemin)
    }
  }
  for (const arbre of ['core', 'modules', 'pages']) parcourir(join(RACINE, arbre))
  return trouves.sort()
}

const MODULE_UNIQUE = 'core/format/montant.ts'

describe('une seule fonction porte la règle', () => {
  it('inspecte réellement des fichiers — une cible vide passerait toujours', () => {
    // Exigence 4 de « Couverture des portes » : prouver que le contrôle a une cible non vide.
    const fichiers = fichiersApplicatifs()

    expect(fichiers.length).toBeGreaterThan(5)
    expect(fichiers.map(f => relative(RACINE, f))).toContain(MODULE_UNIQUE)
  })

  it('aucun autre fichier ne pose U+202F ni ne regroupe les milliers lui-même', () => {
    const fautes: string[] = []

    for (const fichier of fichiersApplicatifs()) {
      const relatif = relative(RACINE, fichier)
      if (relatif === MODULE_UNIQUE) continue

      const contenu = readFileSync(fichier, 'utf8')

      // Le caractère lui-même, sous ses deux écritures. Le laisser passer suffirait à recréer un
      // second formateur : il n'y a pas d'autre raison d'écrire U+202F dans un composant.
      if (contenu.includes(String.fromCodePoint(FINE_INSECABLE)) || /\\u202[fF]/.test(contenu)) {
        fautes.push(`${relatif} — pose U+202F au lieu d'appeler formaterMontant()`)
      }

      // Le groupement de milliers, sous les deux formes qui circulent : l'expression régulière de
      // `tokens.md` et les fonctions de la plateforme. `Intl.NumberFormat` est le piège le plus
      // probable, parce qu'il *semble* faire le travail — mais son séparateur dépend de l'ICU
      // embarqué : U+202F sur les versions récentes, U+00A0 sur les autres.
      for (const [motif, quoi] of [
        [/\\B\(\?=\(\\d\{3\}\)/, 'regroupe les milliers par expression régulière'],
        [/\bIntl\.NumberFormat\b/, 'emploie Intl.NumberFormat, dont le séparateur dépend de l\'ICU'],
        [/\.toLocaleString\s*\(/, 'emploie toLocaleString, même problème qu\'Intl.NumberFormat'],
      ] as const) {
        if (motif.test(contenu)) fautes.push(`${relatif} — ${quoi}`)
      }
    }

    expect(fautes, fautes.join('\n')).toEqual([])
  })

  it('n’expose aucun formateur d’heure — les heures gardent l’espace ORDINAIRE', () => {
    // `tokens.md` §2 : « Les heures gardent une espace ordinaire (17 h 30) : elles ne se coupent
    // pas, la lettre h tient les deux nombres ensemble. » Le risque n'est pas théorique : le jour
    // où un écran affichera une heure de départ, le réflexe sera de l'écrire à côté du montant,
    // dans le même module, avec la même fine. Ce test est là pour qu'on le remarque.
    const source = readFileSync(join(RACINE, MODULE_UNIQUE), 'utf8')
    const exportations = [...source.matchAll(/export (?:function|class|interface|const) (\w+)/g)]
      .map(m => m[1])

    expect(exportations.sort()).toEqual(['Devise', 'MontantNonFormatable', 'deviseDe', 'formaterMontant'].sort())
  })
})
