/**
 * **Lecture et écriture de woff2, sans aucune dépendance.** `node:zlib` porte Brotli depuis
 * Node 12 : décoder une police n'exige donc rien d'installé.
 *
 * # Pourquoi ce module est à part
 *
 * Deux programmes en ont besoin, et pour des raisons opposées :
 *
 * - `app/scripts/generer-polices.ts` **écrit** les polices embarquées ;
 * - `scripts/ci/ressources-embarquees.sh` — la porte **P-21b** — les **relit** pour vérifier que
 *   U+202F et le jeu latin étendu y sont réellement.
 *
 * Un analyseur binaire recopié dans deux fichiers dérive : la porte finirait par valider un format
 * que le générateur n'écrit plus. Il n'y en a donc qu'un, et **la porte n'a besoin d'aucun
 * `node_modules`** pour l'employer — ce qui lui permet de tourner dans le job des portes statiques.
 *
 * # Ce qu'il couvre, et ce qu'il ne couvre pas
 *
 * Il lit le répertoire de tables, décompresse le bloc Brotli et rend chaque table telle quelle. Il
 * **ne reconstruit pas** les tables transformées `glyf` et `loca` : il les transporte sans les
 * comprendre, ce qui suffit à lire une `cmap` et à réécrire un fichier valide. Les métadonnées
 * étendues et les données privées sont **refusées** plutôt qu'ignorées : aucune police du gel n'en
 * porte, et les perdre en silence effacerait des mentions de licence.
 */

import { brotliCompressSync, brotliDecompressSync, constants as zlib } from 'node:zlib'

/** Les 63 étiquettes de table indexées par le format woff2 ; l'index 63 annonce une étiquette libre. */
const ETIQUETTES = [
  'cmap', 'head', 'hhea', 'hmtx', 'maxp', 'name', 'OS/2', 'post', 'cvt ', 'fpgm', 'glyf', 'loca',
  'prep', 'CFF ', 'VORG', 'EBDT', 'EBLC', 'gasp', 'hdmx', 'kern', 'LTSH', 'PCLT', 'VDMX', 'vhea',
  'vmtx', 'BASE', 'GDEF', 'GPOS', 'GSUB', 'EBSC', 'JSTF', 'MATH', 'CBDT', 'CBLC', 'COLR', 'CPAL',
  'SVG ', 'sbix', 'acnt', 'avar', 'bdat', 'bloc', 'bsln', 'cvar', 'fdsc', 'feat', 'fmtx', 'fvar',
  'gvar', 'hsty', 'just', 'lcar', 'mort', 'morx', 'opbd', 'prop', 'trak', 'Zapf', 'Silf', 'Glat',
  'Gloc', 'Feat', 'Sill',
]

export interface TableWoff2 {
  drapeaux: number
  etiquette: string
  /** Taille de la table une fois reconstruite — c'est elle qui entre dans `totalSfntSize`. */
  origine: number
  /** Taille de la table telle qu'elle est stockée ; égale à `origine` si elle n'est pas transformée. */
  longueur: number
  donnees: Buffer
}

export interface PoliceWoff2 {
  flavor: number
  majeure: number
  mineure: number
  tables: TableWoff2[]
}

function lireBase128(buf: Buffer, curseur: { i: number }): number {
  let valeur = 0
  for (let n = 0; n < 5; n++) {
    const octet = buf[curseur.i++]!
    valeur = (valeur << 7) | (octet & 0x7f)
    if ((octet & 0x80) === 0) return valeur >>> 0
  }
  throw new Error('UIntBase128 de plus de cinq octets — fichier woff2 invalide')
}

function ecrireBase128(valeur: number): Buffer {
  const octets: number[] = []
  do {
    octets.unshift(valeur & 0x7f)
    valeur >>>= 7
  } while (valeur > 0)
  for (let n = 0; n < octets.length - 1; n++) octets[n]! |= 0x80
  return Buffer.from(octets)
}

/** Décompose un woff2 tenu en mémoire. `quoi` ne sert qu'aux messages d'erreur. */
export function lireWoff2(buf: Buffer, quoi: string): PoliceWoff2 {
  if (buf.length < 48 || buf.toString('latin1', 0, 4) !== 'wOF2') {
    throw new Error(`${quoi} n'est pas un woff2`)
  }
  if (buf.readUInt32BE(28) !== 0 || buf.readUInt32BE(40) !== 0) {
    throw new Error(`${quoi} porte des métadonnées ou des données privées — non pris en charge`)
  }

  const nb = buf.readUInt16BE(12)
  const curseur = { i: 48 }
  const entrees: Omit<TableWoff2, 'donnees'>[] = []

  for (let n = 0; n < nb; n++) {
    const drapeaux = buf[curseur.i++]!
    const index = drapeaux & 0x3f
    let etiquette: string
    if (index === 63) {
      etiquette = buf.toString('latin1', curseur.i, curseur.i + 4)
      curseur.i += 4
    } else {
      etiquette = ETIQUETTES[index]!
    }
    const version = (drapeaux >> 6) & 0x03
    const origine = lireBase128(buf, curseur)
    // `glyf` et `loca` sont transformées quand la version vaut 0 ; toutes les autres tables le sont
    // quand elle ne vaut PAS 0. C'est l'inversion de la spécification, et elle se lit de travers.
    const transformee = (etiquette === 'glyf' || etiquette === 'loca') ? version === 0 : version !== 0
    const longueur = transformee ? lireBase128(buf, curseur) : origine
    entrees.push({ drapeaux, etiquette, origine, longueur })
  }

  const brut = brotliDecompressSync(buf.subarray(curseur.i))
  const tables: TableWoff2[] = []
  let decalage = 0
  for (const e of entrees) {
    tables.push({ ...e, donnees: brut.subarray(decalage, decalage + e.longueur) })
    decalage += e.longueur
  }
  if (decalage !== brut.length) {
    throw new Error(`${quoi} : ${brut.length - decalage} octet(s) de trop après les tables`)
  }

  return {
    flavor: buf.readUInt32BE(4),
    majeure: buf.readUInt16BE(24),
    mineure: buf.readUInt16BE(26),
    tables,
  }
}

/** Recompose un woff2. Les tables transformées sont transportées telles quelles. */
export function ecrireWoff2(police: PoliceWoff2): Buffer {
  const repertoire: Buffer[] = []
  let tailleSfnt = 12 + 16 * police.tables.length

  for (const t of police.tables) {
    const morceaux: Buffer[] = [Buffer.from([t.drapeaux])]
    if ((t.drapeaux & 0x3f) === 63) morceaux.push(Buffer.from(t.etiquette, 'latin1'))
    morceaux.push(ecrireBase128(t.origine))
    if (t.longueur !== t.origine) morceaux.push(ecrireBase128(t.longueur))
    repertoire.push(Buffer.concat(morceaux))
    tailleSfnt += (t.origine + 3) & ~3
  }

  const brut = Buffer.concat(police.tables.map(t => t.donnees))
  // Paramètres posés explicitement : le déterminisme du mode `--verifier` du générateur en dépend.
  const comprime = brotliCompressSync(brut, {
    params: {
      [zlib.BROTLI_PARAM_MODE]: zlib.BROTLI_MODE_GENERIC,
      [zlib.BROTLI_PARAM_QUALITY]: 11,
      [zlib.BROTLI_PARAM_LGWIN]: 22,
      [zlib.BROTLI_PARAM_SIZE_HINT]: brut.length,
    },
  })

  const tailleRepertoire = repertoire.reduce((n, b) => n + b.length, 0)

  // **Le fichier est complété jusqu'à une frontière de quatre octets, et ce n'est pas de
  // l'esthétique.** Le décodeur de référence de Google — celui qu'emploie harfbuzz, donc les
  // navigateurs — REFUSE un woff2 non aligné : « ConvertWOFF2ToTTF failed », sans autre
  // explication. Les fichiers amont portent ce complément ; les réécrire sans lui produit une
  // police que le lecteur ci-dessus relit parfaitement et qu'aucun navigateur n'ouvre.
  //
  // `length` compte le complément ; `totalCompressedSize` ne compte que le flux Brotli.
  const complement = (4 - (48 + tailleRepertoire + comprime.length) % 4) % 4

  const entete = Buffer.alloc(48)
  entete.write('wOF2', 0, 'latin1')
  entete.writeUInt32BE(police.flavor, 4)
  entete.writeUInt32BE(48 + tailleRepertoire + comprime.length + complement, 8)
  entete.writeUInt16BE(police.tables.length, 12)
  entete.writeUInt16BE(0, 14)
  entete.writeUInt32BE(tailleSfnt, 16)
  entete.writeUInt32BE(comprime.length, 20)
  entete.writeUInt16BE(police.majeure, 24)
  entete.writeUInt16BE(police.mineure, 26)
  // metaOffset, metaLength, metaOrigLength, privOffset, privLength restent à zéro.

  return Buffer.concat([entete, ...repertoire, comprime, Buffer.alloc(complement)])
}

// =================================================================================================
//  La table cmap
// =================================================================================================

/** Toutes les associations `point de code → glyphe` d'une table cmap. Le glyphe 0 n'en est pas une. */
export function lireCmap(cmap: Buffer): Map<number, number> {
  const associations = new Map<number, number>()
  const nb = cmap.readUInt16BE(2)

  for (let n = 0; n < nb; n++) {
    const debutTable = cmap.readUInt32BE(4 + n * 8 + 4)
    const format = cmap.readUInt16BE(debutTable)

    if (format === 4) {
      const segX2 = cmap.readUInt16BE(debutTable + 6)
      const fins = debutTable + 14
      const debuts = fins + segX2 + 2
      const deltas = debuts + segX2
      const decalages = deltas + segX2
      for (let s = 0; s < segX2 / 2; s++) {
        const premier = cmap.readUInt16BE(debuts + s * 2)
        const dernier = cmap.readUInt16BE(fins + s * 2)
        if (premier === 0xffff) continue
        const delta = cmap.readInt16BE(deltas + s * 2)
        const decalage = cmap.readUInt16BE(decalages + s * 2)
        for (let code = premier; code <= dernier; code++) {
          let glyphe: number
          if (decalage === 0) {
            glyphe = (code + delta) & 0xffff
          } else {
            const p = decalages + s * 2 + decalage + (code - premier) * 2
            if (p + 1 >= cmap.length) continue
            glyphe = cmap.readUInt16BE(p)
            if (glyphe !== 0) glyphe = (glyphe + delta) & 0xffff
          }
          if (glyphe !== 0) associations.set(code, glyphe)
        }
      }
    } else if (format === 12) {
      const groupes = cmap.readUInt32BE(debutTable + 12)
      for (let g = 0; g < groupes; g++) {
        const p = debutTable + 16 + g * 12
        const premier = cmap.readUInt32BE(p)
        const dernier = cmap.readUInt32BE(p + 4)
        const premierGlyphe = cmap.readUInt32BE(p + 8)
        for (let code = premier; code <= dernier; code++) {
          const glyphe = premierGlyphe + (code - premier)
          if (glyphe !== 0) associations.set(code, glyphe)
        }
      }
    } else if (format !== 0 && format !== 6) {
      throw new Error(`format de sous-table cmap non pris en charge : ${format}`)
    }
  }
  return associations
}

/**
 * Écrit une table cmap au **format 4**, avec deux sous-tables — `(0, 3)` Unicode BMP et `(3, 1)`
 * Windows BMP — qui pointent la même. C'est le couple que tout moteur lit.
 *
 * Le format 4 suffit parce que **tout ce que le produit embarque tient dans le plan multilingue de
 * base** : la plage la plus haute de `latin-ext` s'arrête à `U+A7FF`. Un point de code au-delà de
 * `U+FFFF` fait échouer, il ne se perd pas en silence.
 */
export function ecrireCmap(associations: Map<number, number>): Buffer {
  const codes = [...associations.keys()].sort((a, b) => a - b)
  for (const code of codes) {
    if (code > 0xffff) throw new Error(`U+${code.toString(16)} hors du plan de base — format 4 insuffisant`)
  }

  // Segments : suites de points de code consécutifs dont l'écart au glyphe reste constant.
  const segments: { premier: number, dernier: number, delta: number }[] = []
  for (const code of codes) {
    const glyphe = associations.get(code)!
    const delta = (glyphe - code) & 0xffff
    const dernier = segments[segments.length - 1]
    if (dernier && dernier.dernier === code - 1 && dernier.delta === delta) {
      dernier.dernier = code
    } else {
      segments.push({ premier: code, dernier: code, delta })
    }
  }
  segments.push({ premier: 0xffff, dernier: 0xffff, delta: 1 })

  const nbSegments = segments.length
  const segX2 = nbSegments * 2
  const longueurFormat4 = 16 + segX2 * 4
  const format4 = Buffer.alloc(longueurFormat4)

  let puissance = 1
  while (puissance * 2 <= nbSegments) puissance *= 2
  format4.writeUInt16BE(4, 0)
  format4.writeUInt16BE(longueurFormat4, 2)
  format4.writeUInt16BE(0, 4)
  format4.writeUInt16BE(segX2, 6)
  format4.writeUInt16BE(puissance * 2, 8)
  format4.writeUInt16BE(Math.log2(puissance), 10)
  format4.writeUInt16BE(segX2 - puissance * 2, 12)
  segments.forEach((s, i) => {
    format4.writeUInt16BE(s.dernier, 14 + i * 2)
    format4.writeUInt16BE(s.premier, 16 + segX2 + i * 2)
    format4.writeUInt16BE(s.delta, 16 + segX2 * 2 + i * 2)
    format4.writeUInt16BE(0, 16 + segX2 * 3 + i * 2)
  })

  const entete = Buffer.alloc(20)
  entete.writeUInt16BE(0, 0)
  entete.writeUInt16BE(2, 2)
  entete.writeUInt16BE(0, 4); entete.writeUInt16BE(3, 6); entete.writeUInt32BE(20, 8)
  entete.writeUInt16BE(3, 12); entete.writeUInt16BE(1, 14); entete.writeUInt32BE(20, 16)

  return Buffer.concat([entete, format4])
}

/** Raccourci : la table cmap d'un woff2 tenu en mémoire. */
export function cmapDe(buf: Buffer, quoi: string): Buffer {
  const table = lireWoff2(buf, quoi).tables.find(t => t.etiquette === 'cmap')
  if (!table) throw new Error(`${quoi} n'a pas de table cmap`)
  return table.donnees
}

/**
 * Un point de code tombe-t-il dans une `unicode-range` CSS ?
 *
 * Sert des deux côtés : le générateur décide **quel fichier** doit porter U+202F, la porte P-21b
 * décide **quel fichier interroger**. Ajouter — ou chercher — un caractère dans un fichier que le
 * navigateur ne consultera jamais pour lui donnerait une porte verte sur un montant en repli.
 */
export function couvertPar(plage: string, code: number): boolean {
  for (const morceau of plage.split(',')) {
    const bornes = morceau.trim().replace(/^U\+/i, '').split('-')
    const premier = Number.parseInt(bornes[0]!, 16)
    const dernier = bornes[1] ? Number.parseInt(bornes[1], 16) : premier
    if (Number.isNaN(premier)) continue
    if (code >= premier && code <= dernier) return true
  }
  return false
}
