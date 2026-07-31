/**
 * Types des points de vente, côté application.
 *
 * **Un comptoir est un point de vente sans aucune table** (`docs/design/lexique.md`). Il n'y a
 * donc pas de champ `est_comptoir` : `tables.length === 0` dit la même chose, et une seconde
 * source pourrait la contredire.
 */

export interface TableVue {
  id: string
  libelle: string
}

export interface PointDeVenteVue {
  id: string
  module_code: string
  nom: string
  caisse_id: string | null
  /** **Vide ⇒ comptoir.** Forme normale d'un maquis, pas un cas dégradé. */
  tables: TableVue[]
}

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
