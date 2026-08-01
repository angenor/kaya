/**
 * **L'inventaire des licences tierces embarquées dans l'application.**
 *
 * # Pourquoi ce module existe
 *
 * Le produit embarque six fichiers de police dans un binaire Tauri **vendu par abonnement**. C'est
 * une redistribution commerciale, et l'attribution est due :
 *
 * - **OFL 1.1, clause 2** — « the above copyright notice and this license notice shall be included
 *   in all copies of one or more of the Font Software typefaces ». *Toutes les copies*, pas
 *   seulement le dépôt.
 * - **MIT** — même exigence, formulée « The above copyright notice and this permission notice shall
 *   be included in all copies or substantial portions of the Software ».
 *
 * Les textes sont donc importés **en clair** (`?raw`) plutôt que désignés par un chemin : un `.txt`
 * de `assets/` que rien n'importe n'entre pas dans la sortie de construction. Ce module est ce qui
 * les fait voyager avec le logiciel.
 *
 * # Ce qu'il n'est pas
 *
 * Ce n'est pas un inventaire des dépendances du projet — celui-là vit dans les lockfiles et dans
 * `docs/versions-gelees.md`. Ce module ne couvre que ce qui est **redistribué dans le binaire** :
 * aujourd'hui, les polices. Une bibliothèque compilée dans le paquet et soumise à attribution
 * viendrait ici ; une dépendance de développement, non.
 *
 * L'inventaire narratif, avec les autres obligations, est dans `docs/conformite/licences-tierces.md`.
 *
 * # Les avis de copyright ne sont PAS des chaînes d'interface
 *
 * Ils ne passent donc pas par l'i18n, et c'est délibéré : un avis de copyright est un **texte
 * légal**, invariable, qui perdrait sa valeur juridique traduit. Ce sont des données, au même titre
 * que le nom d'un établissement. Seuls les libellés qui les entourent — le titre de la section,
 * l'intitulé « Licence » — sont des clés `fr`/`en`.
 */

import ofLArchivo from '~/assets/fonts/archivo-LICENCE.txt?raw'
import ofLChivoMono from '~/assets/fonts/chivo-mono-LICENCE.txt?raw'
import mitPhosphor from '~/assets/fonts/phosphor-LICENCE.txt?raw'

/** Une œuvre tierce redistribuée dans le binaire. */
export interface LicenceTierce {
  /** Identifiant stable — sert de clé de rendu et de préfixe des fichiers embarqués. */
  readonly id: string
  /** Nom tel que ses auteurs l'écrivent. */
  readonly nom: string
  /** Ce que le produit en fait, en une ligne. Clé i18n : c'est un libellé, pas une donnée. */
  readonly usageCle: string
  /** Identifiant SPDX de la licence. */
  readonly licence: string
  /** L'avis de copyright, **mot pour mot**. Donnée légale, jamais traduite. */
  readonly copyright: string
  /** Le texte intégral, embarqué dans le paquet. */
  readonly texte: string
  /**
   * La modification apportée, ou `null` si le fichier amont est repris tel quel.
   *
   * Clé i18n : c'est une phrase d'explication destinée à l'exploitant, pas un texte légal. Le
   * détail technique, lui, vit dans `app/assets/fonts/MODIFICATIONS.md`.
   */
  readonly modificationCle: string | null
  /** Les fichiers embarqués qu'elle couvre — ceux que la porte P-21b rapproche de leur licence. */
  readonly fichiers: readonly string[]
}

/**
 * **Les trois œuvres tierces embarquées à ce jour.**
 *
 * L'ordre est celui de leur poids dans le produit : le texte d'abord, les chiffres ensuite, les
 * icônes en dernier.
 */
export const LICENCES_TIERCES: readonly LicenceTierce[] = [
  {
    id: 'archivo',
    nom: 'Archivo',
    usageCle: 'mentions.usage.texte',
    licence: 'OFL-1.1',
    copyright: 'Copyright 2020 The Archivo Project Authors (https://github.com/Omnibus-Type/Archivo)',
    texte: ofLArchivo,
    modificationCle: 'mentions.modification.fine',
    fichiers: ['archivo-latin-kaya.woff2', 'archivo-latin-ext-kaya.woff2'],
  },
  {
    id: 'chivo-mono',
    nom: 'Chivo Mono',
    usageCle: 'mentions.usage.chiffres',
    licence: 'OFL-1.1',
    copyright: 'Copyright 2018 The Chivo Project Authors (https://github.com/Omnibus-Type/Chivo)',
    texte: ofLChivoMono,
    modificationCle: 'mentions.modification.fine',
    fichiers: ['chivo-mono-latin-kaya.woff2', 'chivo-mono-latin-ext-kaya.woff2'],
  },
  {
    id: 'phosphor',
    nom: 'Phosphor Icons',
    usageCle: 'mentions.usage.icones',
    licence: 'MIT',
    copyright: 'Copyright (c) 2020-2021 Phosphor Icons',
    texte: mitPhosphor,
    modificationCle: 'mentions.modification.sous_reglage',
    fichiers: ['phosphor-kaya.woff2', 'phosphor-fill-kaya.woff2'],
  },
]
