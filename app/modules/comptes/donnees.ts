/**
 * Chargement des données de `G3` — **par le client typé, jamais par un `fetch` écrit à la main**.
 *
 * `@kaya/client` est généré depuis le contrat OpenAPI (porte P-01) : un chemin ou un champ qui
 * change fait échouer la compilation ici, au lieu de produire un `undefined` à l'exécution que
 * personne ne verrait avant la démonstration.
 *
 * # Les types viennent du contrat, ils ne le paraphrasent pas
 *
 * Ce fichier a d'abord **redéclaré à la main** `CompteVue` et `EntreeRole`, puis converti les
 * réponses par `as unknown as`. La double conversion est le seul mécanisme de TypeScript qui
 * accepte de relier deux types sans rapport : elle efface exactement la garantie que la phrase
 * ci-dessus annonce. Renommer `nom_affichage` côté serveur laissait alors le front compiler —
 * vérifié en T062, et c'est ce qui a fait trouver le défaut.
 *
 * Les types sont donc des **alias** de `components['schemas'][…]`. Ce ne sont pas des copies
 * fidèles : ce sont les mêmes types.
 *
 * # Deux appels, et le second n'est pas facultatif
 *
 * L'écran a besoin des comptes **et** du référentiel des rôles. Le second sert à deux choses que
 * l'on confondrait volontiers : composer la liste d'attribution, et **traduire les codes de rôles
 * portés**. C'est aussi pourquoi `compte_lister` ne rend pas de libellé par ligne — sur cinquante
 * comptes, la même clé partirait deux cents fois.
 */

import { creerClientKaya, type components } from '@kaya/client'

import { enTetesAuth, type ContexteAppel } from '~/core/auth'

/** Un rôle porté par un compte, tel que l'API le rend. */
export type RolePorte = components['schemas']['RolePorte']

/** Un compte, tel que l'écran l'affiche. **Aucun condensat, sur aucun chemin.** */
export type CompteVue = components['schemas']['CompteVue']

/** Une entrée du référentiel des rôles. */
export type EntreeRole = components['schemas']['EntreeReferentielRole']

/** Tout ce dont `G3` a besoin, en une fois. */
export interface DonneesComptes {
  comptes: CompteVue[]
  referentielRoles: EntreeRole[]
}

/**
 * Charge l'écran.
 *
 * Les deux appels partent **ensemble** : ils ne dépendent pas l'un de l'autre, et les enchaîner
 * doublerait l'attente sur le réseau intermittent de la persona Aminata.
 */
export async function chargerComptes(
  contexte: ContexteAppel,
  etablissementId?: string,
): Promise<DonneesComptes> {
  const client = creerClientKaya(contexte.baseUrl)
  const headers = enTetesAuth(contexte)

  const [comptes, roles] = await Promise.all([
    client.GET('/api/v1/comptes', {
      params: { query: etablissementId ? { etablissement_id: etablissementId } : {} },
      headers,
    }),
    client.GET('/api/v1/referentiels/roles', { headers }),
  ])

  if (comptes.error) {
    throw new Error('liste des comptes illisible')
  }

  return {
    // Une liste absente vaut liste vide : un établissement dont un seul compte a accès est
    // valide, et traiter l'absence comme une erreur ferait échouer l'écran sur un état normal.
    //
    // **Aucune conversion.** `comptes.data` est déjà `CompteVue[]`, parce que `CompteVue` EST le
    // type du contrat. Un `as unknown as` ici rendrait faux le commentaire de tête de ce
    // fichier — voir la note de T062.
    comptes: comptes.data ?? [],
    referentielRoles: roles.data ?? [],
  }
}
