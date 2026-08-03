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
import type { Middleware } from 'openapi-fetch'
import type { paths } from './types.gen'

/**
 * Construit un client typé sur le contrat OpenAPI.
 *
 * @param baseUrl    Racine de l'API — jamais codée en dur dans un composant.
 * @param middleware Intercepteurs facultatifs, appliqués à chaque appel de ce client.
 *
 * # Pourquoi `middleware` est un paramètre, et non un réglage global
 *
 * Le cycle SYN doit observer **l'issue et la durée** de chaque aller-retour pour alimenter l'état
 * réseau « connexion faible » — l'observateur ne peut pas vivre ici : il appartient à
 * `PlatformAdapter` (principe VII), et ce paquet ne connaît ni la plateforme ni l'application.
 *
 * Le passer en paramètre garde l'inversion dans le bon sens : ce fichier sait **qu'on peut**
 * observer, l'application sait **quoi** faire de l'observation. Un réglage global de module aurait
 * fait dépendre le client d'un état posé ailleurs, et deux applications sur la même origine —
 * cas de la surface QR — se seraient marché dessus.
 */
export function creerClientKaya(baseUrl: string, middleware: Middleware[] = []) {
  const client = createClient<paths>({ baseUrl })
  // `use()` plutôt que l'option `middleware` du constructeur : celle-ci n'existe que sur les
  // options de REQUÊTE (`ClientRequestOptions`), pas sur celles du client. La distinction n'est
  // pas documentée côté README et se voit au `tsc` — elle est notée ici pour la prochaine fois.
  if (middleware.length > 0) {
    client.use(...middleware)
  }
  return client
}

export type ClientKaya = ReturnType<typeof creerClientKaya>
export type { Middleware } from 'openapi-fetch'
export type { paths, components, operations } from './types.gen'
