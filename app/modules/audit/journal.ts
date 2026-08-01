/**
 * Lecture du registre des actions — **`G4`**, CPT-04.
 *
 * # Ce module n'écrit rien, et il n'y a rien à écrire
 *
 * Le contrat n'expose **aucun point d'entrée d'écriture** (research R-17) : une entrée voyage
 * toujours avec l'opération qu'elle trace. Ce fichier n'a donc pas de couche d'écriture, pas de
 * garde de classe C, pas de table de refus métier — et c'est exact, pas une lacune.
 *
 * # Les quatre filtres sont COMBINABLES, et l'écran les cumule
 *
 * FR-037. Le serveur les applique tous ensemble ; le front n'en compose aucun localement. Filtrer
 * côté client sur une page déjà paginée donnerait une liste amputée qui aurait l'air complète —
 * le pire résultat possible pour un registre.
 */

import { creerClientKaya } from '@kaya/client'

import { enTetesAuth, type ContexteAppel } from '~/core/auth'

/** Les dix familles de la taxonomie — `docs/taxonomie-audit.md`. */
export const TYPES_ACTION = [
  'remise',
  'annulation_ligne_envoyee',
  'avoir',
  'ouverture_tiroir',
  'modification_tarif',
  'suppression',
  'changement_role',
  'ecart_caisse',
  'rebascule_palier_passage',
  'forcage_disponibilite',
] as const

export type TypeAction = (typeof TYPES_ACTION)[number]

/** L'auteur d'une entrée. **`nom` absent = compte illisible**, jamais un identifiant en repli. */
export interface AuteurVue {
  compte_id: string
  nom?: string | null
}

/** Une entrée du registre, telle que l'écran l'affiche. */
export interface EntreeJournal {
  id: string
  etablissement_id?: string | null
  type_action: TypeAction
  auteur: AuteurVue
  cible_type: string
  cible_id?: string | null
  contexte: Record<string, unknown>
  /** Indicatif — **jamais présenté comme la date de l'action**. */
  horodatage_client?: string | null
  /** Horodatage d'**autorité serveur**. C'est celui qui s'affiche. */
  cree_le: string
}

/** Une page du registre, avec son curseur de suite. */
export interface PageJournal {
  elements: EntreeJournal[]
  suivant_cree_le?: string | null
  suivant_id?: string | null
}

/** Les filtres, tels que l'écran les compose. Tous optionnels, tous cumulés par le serveur. */
export interface FiltresJournal {
  auteurCompteId?: string
  etablissementId?: string
  typeAction?: TypeAction
  /** Borne **inclusive** de début, en RFC 3339. */
  depuis?: string
  /** Borne **exclusive** de fin — une journée se demande `[J, J+1)`. */
  jusquA?: string
}

/** Position dans la page suivante. */
export interface Curseur {
  creeLe: string
  id: string
}

/**
 * Lit une page du registre.
 *
 * @param curseur Position rendue par la page précédente. `undefined` en tête de liste.
 */
export async function chargerJournal(
  contexte: ContexteAppel,
  filtres: FiltresJournal = {},
  curseur?: Curseur,
): Promise<PageJournal> {
  const client = creerClientKaya(contexte.baseUrl)

  const reponse = await client.GET('/api/v1/journal-audit', {
    params: {
      query: {
        auteur_compte_id: filtres.auteurCompteId,
        etablissement_id: filtres.etablissementId,
        type_action: filtres.typeAction,
        depuis: filtres.depuis,
        jusqu_a: filtres.jusquA,
        apres_cree_le: curseur?.creeLe,
        apres_id: curseur?.id,
      },
    },
    headers: enTetesAuth(contexte),
  })

  if (reponse.error) {
    throw new Error('registre des actions illisible')
  }

  // Une page absente vaut page vide : un établissement où rien ne s'est encore passé est
  // **valide**, et traiter l'absence comme une erreur ferait échouer l'écran sur un état normal.
  return (reponse.data ?? { elements: [] }) as unknown as PageJournal
}

/**
 * Clé i18n du libellé d'un type d'action.
 *
 * **Le nom technique ne s'affiche jamais.** `changement_role` devient « Ce que quelqu'un peut
 * faire a changé » ; `suppression` devient « Mise hors service ». Le lexique traduit, la taxonomie
 * nomme l'intention.
 */
export function cleTypeAction(type: string): string {
  return (TYPES_ACTION as readonly string[]).includes(type)
    ? `journal.types.${type}`
    : 'journal.types.inconnu'
}
