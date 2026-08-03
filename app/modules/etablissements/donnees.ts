/**
 * Chargement des données de `G1` — **par le client typé, jamais par un `fetch` écrit à la main**.
 *
 * `@kaya/client` est généré depuis le contrat OpenAPI (porte P-01) : un chemin ou un champ qui
 * change fait échouer la compilation ici, au lieu de produire un `undefined` à l'exécution que
 * personne ne verrait avant la démonstration.
 *
 * # Le provisoire de contexte est LEVÉ — CPT-01
 *
 * Le tenant et le compte voyageaient dans deux en-têtes, `x-kaya-tenant` et `x-kaya-compte`,
 * au titre de la dérogation `CONTEXTE_PAR_EN_TETES` ouverte au cycle 001. **L'API ne les accepte
 * plus** : le tenant, le compte, l'établissement actif et les permissions viennent du jeton
 * **vérifié** (`backend/api/src/contexte.rs`, refondu par T030).
 *
 * Le point d'entrée unique annoncé alors a tenu sa promesse : une fonction a changé, pas cinq
 * appels. `ContexteAppel` vit désormais dans `~/core/auth` — c'est lui qui sait ce qu'il faut
 * mettre dans une requête, et il n'y a plus rien à choisir.
 */

import type { components } from '@kaya/client'

import { clientKaya } from '~/core/api/client'
import { enTetesAuth, type ContexteAppel } from '~/core/auth'

import type { EntreeReferentiel, ServiceActif } from './services-visibles'
import type { PointDeVenteVue } from './points-de-vente'

export type { ContexteAppel }

/**
 * Une valeur de configuration résolue, avec son origine.
 *
 * `origine` vaut `TENANT` | `ETABLISSEMENT` | `MODULE` | `POINT_DE_VENTE` : c'est ce qui permet à
 * l'écran de distinguer « vaut pour tous vos établissements » de « modifié ici ».
 */
export type ValeurConfiguration = components['schemas']['ValeurVue']

/** L'établissement, tel que l'écran l'affiche. */
export type EtablissementVue = components['schemas']['EtablissementVue']

/** Tout ce dont `G1` a besoin, en une fois. */
export interface DonneesEcran {
  etablissement: EtablissementVue
  services: ServiceActif[]
  referentielModules: EntreeReferentiel[]
  pointsDeVente: PointDeVenteVue[]
  configuration: ValeurConfiguration[]
}

/**
 * Charge les données de l'écran.
 *
 * # Les appels sont parallèles, et le premier est bloquant
 *
 * L'établissement se lit d'abord : sans lui, il n'y a pas d'écran à afficher, et lancer les quatre
 * autres requêtes pour les jeter serait du trafic gaspillé sur le réseau intermittent de la
 * persona Aminata. Les quatre suivantes partent ensemble — elles ne dépendent pas les unes des
 * autres.
 *
 * La configuration est demandée **sans `cle`** : le contrat rend alors toutes les valeurs
 * applicables en **une seule descente de chaîne**. Une trentaine de paramètres à terme, un seul
 * aller-retour.
 */
export async function chargerEcran(
  contexte: ContexteAppel,
  etablissementId: string,
): Promise<DonneesEcran> {
  const client = clientKaya(contexte.baseUrl)
  const headers = enTetesAuth(contexte)

  const etablissement = await client.GET('/api/v1/etablissements/{etablissement_id}', {
    params: { path: { etablissement_id: etablissementId } },
    headers,
  })

  if (etablissement.error || !etablissement.data) {
    throw new Error(`établissement ${etablissementId} illisible`)
  }

  const [services, referentiel, pointsDeVente, configuration] = await Promise.all([
    client.GET('/api/v1/etablissements/{etablissement_id}/services', {
      params: { path: { etablissement_id: etablissementId } },
      headers,
    }),
    client.GET('/api/v1/referentiels/modules-activite', { headers }),
    client.GET('/api/v1/etablissements/{etablissement_id}/points-de-vente', {
      params: { path: { etablissement_id: etablissementId } },
      headers,
    }),
    client.GET('/api/v1/configuration', {
      params: { query: { etablissement_id: etablissementId } },
      headers,
    }),
  ])

  return {
    etablissement: etablissement.data,
    // Une liste absente vaut liste vide : un établissement sans service, sans point de vente ou
    // sans configuration est **valide**. C'est exactement le cas de la résidence meublée, et
    // traiter l'absence comme une erreur ferait échouer l'écran sur un état parfaitement normal.
    services: services.data ?? [],
    referentielModules: referentiel.data ?? [],
    pointsDeVente: pointsDeVente.data ?? [],
    configuration: configuration.data ?? [],
  }
}

/**
 * Recharge **les seuls services** d'un établissement.
 *
 * # Pourquoi une seconde fonction plutôt qu'un `chargerEcran` de plus
 *
 * Après une activation ou une désactivation, l'écran doit se remettre à jour **sans rechargement
 * de page** (point 7 du patron d'écriture). Rappeler `chargerEcran` ferait cinq requêtes pour en
 * rafraîchir une : l'identité, les points de vente et la configuration n'ont pas bougé. Sur le
 * réseau intermittent de la persona Aminata, quatre requêtes inutiles ne sont pas un détail.
 *
 * # L'écran ne relit PAS le corps de la réponse de bascule
 *
 * Le `PUT` rend bien l'état atteint, mais **une désactivation le rend absent de la liste des
 * actifs** — c'est exact, et c'est précisément l'effet à montrer. Relire la liste entière est donc
 * la seule façon d'obtenir ce que l'écran doit afficher, dans les deux sens. Et c'est le serveur
 * qui fait foi (principe VI), pas une liste modifiée à la main côté client.
 */
export async function chargerServices(
  contexte: ContexteAppel,
  etablissementId: string,
): Promise<ServiceActif[]> {
  const client = clientKaya(contexte.baseUrl)

  const services = await client.GET('/api/v1/etablissements/{etablissement_id}/services', {
    params: { path: { etablissement_id: etablissementId } },
    headers: enTetesAuth(contexte),
  })

  if (services.error) {
    throw new Error(`services de l'établissement ${etablissementId} illisibles`)
  }

  return services.data ?? []
}
