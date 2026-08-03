/**
 * **La frontière entre « à réessayer » et « définitif »** — écrite une fois, au bon endroit.
 *
 * # Pourquoi cette décision ne peut pas vivre dans la boucle d'envoi
 *
 * Elle est fine, elle a des cas que personne ne devine, et elle sera relue à chaque cycle qui
 * ajoutera une opération de classe A. L'enfouir dans une condition au milieu d'une boucle
 * garantirait qu'un cycle suivant en écrive une variante à côté.
 *
 * # La table, et le cas qu'on écrirait mal
 *
 * | Issue | Traitement | Motif |
 * |---|---|---|
 * | Réseau injoignable, délai dépassé | **réessai** | Rien n'a été décidé côté serveur |
 * | `408`, `429`, `5xx` | **réessai** | Le serveur dit lui-même « plus tard » |
 * | `401` | **rafraîchissement de session**, file **intacte** | Traité par le point de sortie unique, jamais par un vidage |
 * | `400`, `403`, `404`, `409`, `422` | **quarantaine** | Le serveur a décidé, et rejouer ne changera pas sa décision |
 * | `200`, `201` | **retrait de la file** | Y compris `200` |
 *
 * **`200` est le cas qu'on écrirait mal.** Une file qui traiterait « déjà présente » comme un
 * conflit remettrait l'écriture en tête et boucllerait indéfiniment. Le patron du module doré rend
 * `200` avec la ligne telle qu'elle est en base **précisément pour que ce cas soit le chemin
 * normal d'un rejeu** — jamais `409`, jamais une erreur.
 *
 * # La quarantaine n'est pas un cimetière
 *
 * Elle est **consultable** (écran `S1`), porte son motif en langue utilisateur branché sur le
 * `code` d'erreur — jamais sur le `message`, qui nomme des tables et parle anglais technique —, et
 * surtout **cesse de bloquer** les écritures suivantes. Une entrée refusée définitivement qui
 * resterait en tête de file empêcherait tout le service de partir pour une seule saisie fautive.
 *
 * Elle ne bloque pas non plus le geste de passer la main : `ecrituresEnAttente()` compte ce qui
 * **attend d'être envoyé**, pas ce qui a été refusé. Refuser une déconnexion pour une entrée que
 * le serveur ne reprendra jamais bloquerait le terminal.
 */

import type { EntreeFile } from './classes'

/** Ce qu'il faut faire d'une entrée après une tentative d'envoi. */
export type Suite =
  /** Acquittée — elle quitte la file. `201` comme `200`. */
  | 'retirer'
  /** Rien n'a été décidé, ou le serveur a dit « plus tard ». Elle reste, et on réessaie. */
  | 'reessayer'
  /** La session est finie. La file est **intacte** ; le point de sortie unique rafraîchit. */
  | 'session'
  /** Refus **définitif**. Elle sort de la file d'envoi et devient consultable. */
  | 'quarantaine'

/**
 * Classe une réponse HTTP.
 *
 * @param statut Le code de réponse, ou `null` si l'appel n'a pas abouti — réseau injoignable,
 *               délai dépassé, requête coupée. **`null` n'est pas `0`** : un code absent dit que
 *               le serveur n'a rien décidé, ce qui est exactement la raison de réessayer.
 */
export function classer(statut: number | null): Suite {
  if (statut === null) {
    return 'reessayer'
  }
  if (statut === 200 || statut === 201) {
    return 'retirer'
  }
  if (statut === 401) {
    return 'session'
  }
  if (statut === 408 || statut === 429 || statut >= 500) {
    return 'reessayer'
  }
  if (statut >= 400) {
    // 400, 403, 404, 409, 422 et tout autre refus de la famille. Le serveur a décidé.
    return 'quarantaine'
  }
  // 1xx, 2xx autres, 3xx : le contrat n'en produit aucun sur ces opérations. Réessayer est le
  // choix prudent — il ne perd rien, là où une quarantaine perdrait une écriture acceptable.
  return 'reessayer'
}

/** Une écriture définitivement refusée, telle que l'écran `S1` la montre. */
export interface EntreeQuarantaine {
  /** L'écriture telle qu'elle était. */
  readonly entree: EntreeFile
  /**
   * Le `code` d'erreur du serveur — **jamais le `message`**.
   *
   * C'est la règle du lexique, et elle vaut ici comme ailleurs : `message` est un diagnostic
   * destiné aux journaux, en anglais technique, qui nomme des tables. L'interface branche sa clé
   * i18n sur le `code`.
   */
  readonly code: string
  /** Quand le refus est tombé, en RFC 3339. */
  readonly refuseeLe: string
}

/**
 * La clé i18n d'un motif de refus.
 *
 * **Table ouverte à repli, et non fermée** — contrairement à celle du patron d'écriture
 * (`bascule-service.ts`), et la différence se justifie : là-bas, les codes sont ceux d'**une**
 * opération, connus et peu nombreux. Ici, la file transporte toute opération de classe A du
 * produit, présente et à venir. Une table fermée afficherait une clé brute au premier code d'un
 * cycle suivant.
 *
 * Le repli est **honnête** : il dit que la saisie a été refusée et invite à la ressaisir, plutôt
 * que d'afficher un code que personne ne sait lire.
 */
export function cleMotifRefus(code: string): string {
  const connus = new Set([
    'validation',
    'permission_refusee',
    'introuvable',
    'conflit',
    'etablissement_inconnu',
  ])
  return connus.has(code) ? `sync.quarantaine.motif.${code}` : 'sync.quarantaine.motif.inconnu'
}
