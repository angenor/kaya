/**
 * Chargement des données de `G3` — **par le client typé, jamais par un `fetch` écrit à la main**.
 *
 * `@kaya/client` est généré depuis le contrat OpenAPI (porte P-01) : un chemin ou un champ qui
 * change fait échouer la compilation ici, au lieu de produire un `undefined` à l'exécution que
 * personne ne verrait avant la démonstration.
 *
 * # Deux appels, et le second n'est pas facultatif
 *
 * L'écran a besoin des comptes **et** du référentiel des rôles. Le second sert à deux choses que
 * l'on confondrait volontiers : composer la liste d'attribution, et **traduire les codes de rôles
 * portés**. C'est aussi pourquoi `compte_lister` ne rend pas de libellé par ligne — sur cinquante
 * comptes, la même clé partirait deux cents fois.
 */

import { creerClientKaya } from '@kaya/client'

import { enTetesAuth, type ContexteAppel } from '~/core/auth'

/** Un rôle porté par un compte, tel que l'API le rend. */
export interface RolePorte {
  role_code: string
  /** `null` pour `admin_editeur`, dont la portée est l'éditeur. */
  etablissement_id?: string | null
}

/** Un compte, tel que l'écran l'affiche. **Aucun condensat, sur aucun chemin.** */
export interface CompteVue {
  id: string
  personne_id: string
  /** Lu de `personne`, **jamais** l'identifiant de connexion. */
  nom_affichage: string
  identifiant_telephone?: string | null
  identifiant_email?: string | null
  methode_code: string
  actif: boolean
  roles: RolePorte[]
  cree_le: string
  modifie_le: string
}

/** Une entrée du référentiel des rôles. */
export interface EntreeRole {
  code: string
  /** **Clé i18n, jamais un libellé.** */
  libelle_cle: string
  ordre: number
  /** `ETABLISSEMENT` ou `EDITEUR`. */
  portee?: string | null
}

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
    comptes: (comptes.data ?? []) as unknown as CompteVue[],
    referentielRoles: (roles.data ?? []) as unknown as EntreeRole[],
  }
}
