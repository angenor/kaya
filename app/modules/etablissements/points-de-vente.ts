/**
 * Types des points de vente, côté application.
 *
 * **Un comptoir est un point de vente sans aucune table** (`docs/design/lexique.md`). Il n'y a
 * donc pas de champ `est_comptoir` : `tables.length === 0` dit la même chose, et une seconde
 * source pourrait la contredire.
 */

import type { components } from '@kaya/client'

/** Une table de salle. **Alias du contrat, pas une copie** — voir la note de `donnees.ts`. */
export type TableVue = components['schemas']['TableVue']

/**
 * Un point de vente. Son champ `tables` **vide vaut comptoir** : forme normale d'un maquis, pas
 * un cas dégradé.
 */
export type PointDeVenteVue = components['schemas']['PointDeVenteVue']

/**
 * La clé i18n qui qualifie un point de vente.
 *
 * « Comptoir » quand il n'a aucune table, sinon le nombre de tables. Jamais « point de vente sans
 * tables », qui décrit un manque là où il s'agit d'une forme normale.
 */
export function cleQualificatif(pointDeVente: PointDeVenteVue): string {
  return pointDeVente.tables.length === 0
    ? 'etablissement.points_de_vente.comptoir'
    : 'etablissement.points_de_vente.avec_tables'
}
