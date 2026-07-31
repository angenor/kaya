/**
 * Client d'API Kaya.
 *
 * **Deux fichiers, deux régimes — c'est tout l'intérêt du choix de gel §3.2 :**
 *
 * - `types.gen.ts` est **généré** depuis `/api-docs/openapi.json` par `openapi-typescript`. Il ne
 *   contient que des types, aucun code d'exécution. Il n'est **jamais** édité à la main, et la
 *   porte **P-01** fait échouer le build si le fichier commité diffère de ce que le contrat
 *   produit.
 * - `index.ts` — ce fichier — est **écrit à la main** et ne se régénère jamais. `openapi-fetch`
 *   est une bibliothèque installée, pas un artefact.
 *
 * Un générateur de SDK complet produirait à chaque exécution des fichiers de client entiers,
 * multipliant les occasions de faux positif sur P-01. Ici la surface générée se limite à des
 * déclarations de types, donc au strict dérivé du contrat.
 */

import createClient from 'openapi-fetch'
import type { paths } from './types.gen'

/**
 * Construit un client typé sur le contrat OpenAPI.
 *
 * @param baseUrl Racine de l'API — jamais codée en dur dans un composant.
 */
export function creerClientKaya(baseUrl: string) {
  return createClient<paths>({ baseUrl })
}

export type ClientKaya = ReturnType<typeof creerClientKaya>
export type { paths, components, operations } from './types.gen'
