/**
 * Chargement des données des deux écrans du cycle — **par le client typé, jamais par un `fetch`
 * écrit à la main**.
 *
 * `@kaya/client` est généré depuis le contrat OpenAPI (porte P-01) : un chemin ou un champ qui
 * change fait échouer la compilation ici, au lieu de produire un `undefined` à l'exécution que
 * personne ne verrait avant la démonstration.
 *
 * # Les types consommés viennent du contrat, jamais d'une interface qui lui ressemble
 *
 * `components['schemas'][…]`, et pas une copie fidèle. Le module doré le dit après l'avoir payé :
 * *« une copie fidèle le reste jusqu'au premier champ ajouté d'un côté »*, et quatre fichiers du
 * cycle 002 redéclaraient leurs types à la main puis convertissaient par `as unknown as` — la
 * seule construction de TypeScript qui relie deux types sans rapport. P-01 restait verte : elle
 * compare le client généré au client commité, et la rupture était un cran plus loin.
 */

import { creerClientKaya, type components } from '@kaya/client'

import { enTetesAuth, type ContexteAppel } from '~/core/auth'

export type { ContexteAppel }

/** Une formule, telle que le contrat la rend — **prix entier, devise au même niveau**. */
export type FormuleVue = components['schemas']['FormuleVue']

/** Un type de chambre. Terme utilisateur : « type de chambre », jamais « catégorie d'unité ». */
export type CategorieVue = components['schemas']['CategorieVue']

/** Une chambre, un logement, une salle. */
export type UniteVue = components['schemas']['UniteVue']

/** Ce dont `G2` a besoin : les formules, et les types de chambre qui les portent. */
export interface DonneesOffre {
  categories: CategorieVue[]
  formules: FormuleVue[]
}

/** Ce dont `G5` a besoin. */
export interface DonneesChambres {
  categories: CategorieVue[]
  unites: UniteVue[]
}

/**
 * Charge l'offre — `G2`.
 *
 * Les deux appels partent **ensemble** : ils ne dépendent pas l'un de l'autre, et le réseau
 * d'Abengourou fait payer chaque aller-retour séquentiel.
 */
export async function chargerOffre(
  contexte: ContexteAppel,
  etablissementId: string,
): Promise<DonneesOffre> {
  const client = creerClientKaya(contexte.baseUrl)
  const headers = enTetesAuth(contexte)

  const [categories, formules] = await Promise.all([
    client.GET('/api/v1/etablissements/{etablissement_id}/hebergement/categories', {
      params: { path: { etablissement_id: etablissementId } },
      headers,
    }),
    client.GET('/api/v1/etablissements/{etablissement_id}/hebergement/formules', {
      params: { path: { etablissement_id: etablissementId } },
      headers,
    }),
  ])

  if (categories.error || !categories.data || formules.error || !formules.data) {
    throw new Error(`offre d'hébergement illisible pour ${etablissementId}`)
  }

  return { categories: categories.data, formules: formules.data }
}

/**
 * Recharge **les seules formules**, après une écriture.
 *
 * Une requête, pas deux : les types de chambre n'ont pas bougé. Et la liste vient du **serveur**,
 * jamais reconstruite à la main côté client — le serveur fait foi en conflit (principe VI).
 */
export async function chargerFormules(
  contexte: ContexteAppel,
  etablissementId: string,
): Promise<FormuleVue[]> {
  const client = creerClientKaya(contexte.baseUrl)
  const reponse = await client.GET(
    '/api/v1/etablissements/{etablissement_id}/hebergement/formules',
    {
      params: { path: { etablissement_id: etablissementId } },
      headers: enTetesAuth(contexte),
    },
  )

  if (reponse.error || !reponse.data) {
    throw new Error(`formules illisibles pour ${etablissementId}`)
  }
  return reponse.data
}

/** Charge le parc de chambres — `G5`. */
export async function chargerChambres(
  contexte: ContexteAppel,
  etablissementId: string,
): Promise<DonneesChambres> {
  const client = creerClientKaya(contexte.baseUrl)
  const headers = enTetesAuth(contexte)

  const [categories, unites] = await Promise.all([
    client.GET('/api/v1/etablissements/{etablissement_id}/hebergement/categories', {
      params: { path: { etablissement_id: etablissementId } },
      headers,
    }),
    client.GET('/api/v1/etablissements/{etablissement_id}/hebergement/unites', {
      params: { path: { etablissement_id: etablissementId } },
      headers,
    }),
  ])

  if (categories.error || !categories.data || unites.error || !unites.data) {
    throw new Error(`parc de chambres illisible pour ${etablissementId}`)
  }

  return { categories: categories.data, unites: unites.data }
}

/** Recharge **les seules chambres**, après une écriture. */
export async function chargerUnites(
  contexte: ContexteAppel,
  etablissementId: string,
): Promise<UniteVue[]> {
  const client = creerClientKaya(contexte.baseUrl)
  const reponse = await client.GET(
    '/api/v1/etablissements/{etablissement_id}/hebergement/unites',
    {
      params: { path: { etablissement_id: etablissementId } },
      headers: enTetesAuth(contexte),
    },
  )

  if (reponse.error || !reponse.data) {
    throw new Error(`chambres illisibles pour ${etablissementId}`)
  }
  return reponse.data
}

/** Recharge **les seuls types de chambre**, après une écriture. */
export async function chargerCategories(
  contexte: ContexteAppel,
  etablissementId: string,
): Promise<CategorieVue[]> {
  const client = creerClientKaya(contexte.baseUrl)
  const reponse = await client.GET(
    '/api/v1/etablissements/{etablissement_id}/hebergement/categories',
    {
      params: { path: { etablissement_id: etablissementId } },
      headers: enTetesAuth(contexte),
    },
  )

  if (reponse.error || !reponse.data) {
    throw new Error(`types de chambre illisibles pour ${etablissementId}`)
  }
  return reponse.data
}
