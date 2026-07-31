/**
 * Chargement des données de `G1` — **par le client typé, jamais par un `fetch` écrit à la main**.
 *
 * `@kaya/client` est généré depuis le contrat OpenAPI (porte P-01) : un chemin ou un champ qui
 * change fait échouer la compilation ici, au lieu de produire un `undefined` à l'exécution que
 * personne ne verrait avant la démonstration.
 *
 * # Le contexte d'appel est un provisoire NOMMÉ
 *
 * L'authentification est **CPT-01**, hors du périmètre de ce cycle. En attendant, le tenant et le
 * compte voyagent dans deux en-têtes — c'est la dérogation `CONTEXTE_PAR_EN_TETES` ouverte au
 * cycle 001, dont l'API refuse de démarrer si elle n'est pas explicitement activée.
 *
 * **Elle est reprise ici plutôt que contournée** : écrire un chargement sans contexte donnerait un
 * écran qui « marche » en développement et qui devra être rouvert au cycle CPT. Le point d'entrée
 * unique ci-dessous est ce qu'il y aura à changer — une fonction, pas cinq appels.
 */

import { creerClientKaya } from '@kaya/client'

import type { EntreeReferentiel, ServiceActif } from './services-visibles'
import type { PointDeVenteVue } from './points-de-vente'

/** Une valeur de configuration résolue, avec son origine. */
export interface ValeurConfiguration {
  cle: string
  valeur: unknown
  /** `TENANT` | `ETABLISSEMENT` | `MODULE` | `POINT_DE_VENTE`. */
  origine: string
}

/** L'établissement, tel que l'écran l'affiche. */
export interface EtablissementVue {
  id: string
  nom: string
  classement: string
  etoiles: number | null
  commune: string
  fuseau_horaire: string
  devise: string
  ncc: string | null
}

/** Tout ce dont `G1` a besoin, en une fois. */
export interface DonneesEcran {
  etablissement: EtablissementVue
  services: ServiceActif[]
  referentielModules: EntreeReferentiel[]
  pointsDeVente: PointDeVenteVue[]
  configuration: ValeurConfiguration[]
}

/** Contexte d'appel — **provisoire, levé par CPT-01**. */
export interface ContexteAppel {
  baseUrl: string
  tenantId: string
  compteId: string
}

function enTetes(contexte: ContexteAppel): Record<string, string> {
  return {
    'x-kaya-tenant': contexte.tenantId,
    'x-kaya-compte': contexte.compteId,
  }
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
  const client = creerClientKaya(contexte.baseUrl)
  const headers = enTetes(contexte)

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
    etablissement: etablissement.data as unknown as EtablissementVue,
    // Une liste absente vaut liste vide : un établissement sans service, sans point de vente ou
    // sans configuration est **valide**. C'est exactement le cas de la résidence meublée, et
    // traiter l'absence comme une erreur ferait échouer l'écran sur un état parfaitement normal.
    services: (services.data ?? []) as unknown as ServiceActif[],
    referentielModules: (referentiel.data ?? []) as unknown as EntreeReferentiel[],
    pointsDeVente: (pointsDeVente.data ?? []) as unknown as PointDeVenteVue[],
    configuration: (configuration.data ?? []) as unknown as ValeurConfiguration[],
  }
}
