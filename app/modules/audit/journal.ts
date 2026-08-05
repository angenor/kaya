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

import type { components } from '@kaya/client'

import { clientKaya } from '~/core/api/client'
import { enTetesAuth, type ContexteAppel } from '~/core/auth'

/**
 * Une famille de la taxonomie d'audit — **le type du contrat**, `docs/taxonomie-audit.md`.
 *
 * `TypeActionAudit` est une énumération **fermée** côté serveur : le contrat la rend comme une
 * union de littéraux. La reprendre ici plutôt que de la redéclarer fait qu'une famille
 * renommée ou retirée casse la compilation du front.
 */
export type TypeAction = components['schemas']['TypeActionAudit']

/**
 * Les **douze** familles, dans l'ordre d'affichage du filtre.
 *
 * Le type ne suffit pas : une union n'est pas énumérable à l'exécution, et l'écran a besoin de la
 * liste pour composer son sélecteur. `satisfies` relie les deux — une famille **renommée ou
 * retirée** du contrat fait échouer la compilation ici. Le versant complémentaire — une famille
 * **ajoutée** au contrat et absente de cette liste — est vérifié par `app/tests/ecran-g4.spec.ts`,
 * qui ne peut pas s'écrire ici : il demande une assertion de type, pas une valeur.
 */
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
  // Cycle 005 (SYN-04) — la première famille qui ne trace aucun geste d'utilisateur : elle
  // constate que l'heure d'un terminal s'écarte de celle du serveur. Elle est au filtre comme
  // les dix autres, parce que c'est exactement ce qu'un propriétaire vient chercher au registre
  // après un service où les horaires paraissaient faux.
  'derive_horloge_constatee',
  // Cycle 006 (SEJ-01) — la première famille qui trace une **LECTURE** et non une modification.
  // FR-012 exige un journal d'accès à la pièce d'identité ; aucune des onze ne couvrait une
  // consultation — `suppression` trace une mise hors service, `changement_role` une attribution,
  // toutes tracent un geste qui MODIFIE.
  //
  // ⚠️ Le contexte de l'entrée ne porte **jamais la valeur lue** : recopier un numéro de pièce
  // dans un registre immuable et à rétention illimitée créerait la fuite que ce journal existe
  // pour surveiller.
  'consultation_piece_identite',
] as const satisfies readonly TypeAction[]

/** L'auteur d'une entrée. **`nom` absent = compte illisible**, jamais un identifiant en repli. */
export type AuteurVue = components['schemas']['AuteurVue']

/**
 * Une entrée du registre, telle que l'écran l'affiche.
 *
 * **Alias du contrat, pas une copie** — voir la note de tête de `modules/comptes/donnees.ts`.
 */
export type EntreeJournal = components['schemas']['EntreeJournalVue']

/** Une page du registre, avec son curseur de suite. */
export type PageJournal = components['schemas']['PageJournalVue']

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
  const client = clientKaya(contexte.baseUrl)

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
  return reponse.data ?? { elements: [] }
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
