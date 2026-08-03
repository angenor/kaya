/**
 * Attribution et retrait de rôle — **le patron d'écriture front, seconde application**.
 *
 * Le déroulé complet et son pourquoi : `docs/module-dore.md`, « La septième couche ». Ce fichier
 * suit ETB-02 point pour point, et n'écrit ici que ce qui lui est propre.
 *
 * # 1 · CLASSE C — le refus vient AVANT l'appel, jamais après
 *
 * `compte_role` est de **classe C** (`docs/registre-classes-offline.md` §5.2). Trois conséquences,
 * toutes tenues ici :
 *
 * - l'opération **n'entre jamais en file** ; `core/sync` la refuserait de toute façon, son type
 *   n'étant pas déclaré dans `TYPES_CLASSE_A` ;
 * - hors ligne, l'interface **dit immédiatement** que l'action demande le réseau ;
 * - la vérification se fait **avant** l'appel réseau, **dans ce fichier** et pas dans le
 *   composant : un second appelant oublierait la garde, et la faute ne se verrait qu'en clientèle.
 *
 * **Une élévation de privilège hors ligne serait la pire faute possible du produit.** C'est
 * exactement ce que le principe VI nomme en interdisant les classes B, C et D hors ligne : un
 * terminal qui s'accorderait un rôle pendant une coupure, puis le synchroniserait, aurait obtenu
 * un droit que personne n'a accordé.
 *
 * # 2 · Deux actes, jamais un « changer de rôle »
 *
 * Il n'existe pas d'opération de modification : on retire, puis on attribue. `compte_role` n'a pas
 * de privilège `UPDATE` en base, et le registre des actions porte **deux** entrées. Une opération
 * unique en cacherait une des deux.
 */


import { clientKaya } from '~/core/api/client'
import { enTetesAuth, type ContexteAppel } from '~/core/auth'
import { cleDeRefus } from '~/core/erreurs/codes'
import type { EtatReseau } from '~/core/platform'
import { uuidV7 } from '~/core/sync/uuid-v7'

/**
 * Types d'opération, au vocabulaire du registre hors-ligne.
 *
 * **Ils ne sont PAS dans `TYPES_CLASSE_A`**, et ce n'est pas un oubli : les y mettre autoriserait
 * la mise en file d'une opération de classe C, ce que la porte P-13 refuse.
 */
export const TYPE_ATTRIBUTION = 'compte_role.attribue'
export const TYPE_RETRAIT = 'compte_role.retire'

/**
 * Permission requise — **ligne du référentiel `comptes.permission`**, migration `0016`.
 *
 * Portée par `proprietaire` et `gerant`. C'est aussi elle que FR-023 protège : le dernier compte
 * qui la détient sur un établissement ne peut pas se la retirer.
 */
export const PERMISSION_ATTRIBUER = 'cpt.role.attribuer'

/** Ce qu'un changement de rôle produit. Un seul type de retour, jamais d'exception au vol. */
export type ResultatRole =
  | { issue: 'succes' }
  | {
    issue: 'refus'
    /** Clé i18n du message à afficher. Toujours renseignée. */
    cle: string
    valeurs?: Record<string, unknown>
    /** Le refus vient-il de l'absence de réseau ? L'interface ne le rend pas de la même façon. */
    reseau?: boolean
  }

/**
 * Les codes **propres à ce module**, consultés avant la table partagée.
 *
 * `portee_incompatible`, `etablissement_inconnu` et `derniere_habilitation` n'y sont **pas** :
 * ils vivent dans `core/erreurs/codes.ts`, parce que d'autres modules les rendent aussi. Les
 * recopier ici en ferait deux copies, et celle qui dérive est toujours celle qu'on ne relit pas.
 */
const CLES_DU_MODULE: Record<string, string> = {
  role_inconnu: 'comptes.refus.role_inconnu',
  compte_inconnu: 'comptes.refus.compte_inconnu',
}

const REFUS_PERMISSION = 'comptes.refus.permission'
const REFUS_RESEAU = 'comptes.refus.reseau'

/** Le corps d'erreur du contrat, réduit à ce que l'interface en consomme. */
interface CorpsErreur {
  code?: string
  motif_cle?: string | null
  valeur?: string | null
}

/** Traduit une réponse d'erreur en refus affichable. */
function refuser(statut: number, corps: CorpsErreur | undefined, valeur: string): ResultatRole {
  // `403` : l'absence de permission ne se diagnostique pas, elle se constate. En pratique
  // l'utilisateur ne devrait jamais la voir — l'action lui est ABSENTE (principe VII). Le message
  // existe pour le cas où ses droits changent pendant qu'il regarde l'écran, seul chemin qui y mène.
  if (statut === 403) {
    return { issue: 'refus', cle: REFUS_PERMISSION }
  }

  const cle = cleDeRefus(corps?.code, corps?.motif_cle, CLES_DU_MODULE)

  return { issue: 'refus', cle, valeurs: { valeur: corps?.valeur ?? valeur } }
}

/**
 * Attribue un rôle.
 *
 * @param reseau État réseau **au moment du geste**, lu depuis `PlatformAdapter`. Passé en
 *   paramètre plutôt que lu ici : la fonction reste testable sans navigateur, et l'appelant est
 *   obligé de constater qu'il y a une question d'état réseau à poser.
 */
export async function attribuerRole(
  contexte: ContexteAppel,
  compteId: string,
  roleCode: string,
  etablissementId: string | null,
  reseau: EtatReseau,
): Promise<ResultatRole> {
  // CLASSE C — refus immédiat, avant l'appel. Pas de mise en file « au cas où » : une opération de
  // classe C n'a aucune garantie de rejeu, et une élévation de privilège différée serait la pire
  // faute possible du produit.
  if (reseau !== 'connecte') {
    return { issue: 'refus', cle: REFUS_RESEAU, reseau: true }
  }

  const client = clientKaya(contexte.baseUrl)
  const reponse = await client.POST('/api/v1/comptes/{compte_id}/roles', {
    params: { path: { compte_id: compteId } },
    // UUID v7 **généré côté client** : c'est lui qui rend le rejeu inoffensif (principe VI).
    body: { id: uuidV7(), role_code: roleCode, etablissement_id: etablissementId },
    headers: enTetesAuth(contexte),
  })

  if (!reponse.error) {
    return { issue: 'succes' }
  }

  return refuser(reponse.response.status, reponse.error as CorpsErreur, roleCode)
}

/**
 * Retire un rôle.
 *
 * **`derniere_habilitation` est le seul refus métier du cycle**, et il est irréversible sans
 * l'éditeur : le message qu'il produit doit dire quoi faire, pas seulement que c'est refusé.
 */
export async function retirerRole(
  contexte: ContexteAppel,
  compteId: string,
  roleCode: string,
  etablissementId: string | null,
  reseau: EtatReseau,
): Promise<ResultatRole> {
  if (reseau !== 'connecte') {
    return { issue: 'refus', cle: REFUS_RESEAU, reseau: true }
  }

  const client = clientKaya(contexte.baseUrl)
  const reponse = await client.DELETE('/api/v1/comptes/{compte_id}/roles/{role_code}', {
    params: {
      path: { compte_id: compteId, role_code: roleCode },
      query: etablissementId ? { etablissement_id: etablissementId } : {},
    },
    headers: enTetesAuth(contexte),
  })

  if (!reponse.error) {
    return { issue: 'succes' }
  }

  return refuser(reponse.response.status, reponse.error as CorpsErreur, roleCode)
}
