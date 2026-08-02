/**
 * **La reprise de session, avant chaque navigation.** *Point d'amorçage n° 2.*
 *
 * # Ce que ce fichier corrige
 *
 * `reprendreSession()` n'était appelée que par `pages/index.vue`. Les cinq autres routes
 * n'amorçaient rien : ouvrir `/comptes` ou `/journal-audit` directement — un signet, un lien
 * copié, un rechargement — affichait « Connectez-vous pour continuer » **alors qu'un jeton de
 * rafraîchissement valide dormait dans le stockage**. L'écran mentait, et il mentait sur le seul
 * chemin qu'un utilisateur emprunte quand il revient à ce qu'il faisait.
 *
 * Un middleware global s'exécute avant **chaque** changement de route, la première comprise.
 * C'est la seule place qui couvre les six routes sans être recopiée six fois — et la recopie
 * était précisément la faute : cinq pages sur six avaient oublié la ligne.
 *
 * # Il s'exécute bien à la première navigation, en SPA aussi
 *
 * Le `router.beforeEach` qui porte les middlewares est posé **après** `router.isReady()` : la
 * toute première résolution lui échappe. Nuxt rejoue alors la route initiale au hook
 * `app:created`, avec `force: true`, et c'est ce rejeu qui la fait passer par les middlewares. Le
 * comportement est celui qu'on veut ; le mécanisme, lui, ne se devine pas.
 *
 * Corollaire utile : un middleware s'exécute **après** tous les plugins. Quand celui-ci tourne,
 * `plugins/01.theme.client.ts` a déjà posé le thème.
 *
 * # Ce que ce middleware ne fait PAS
 *
 * **Il ne garde aucune permission.** Une permission absente doit produire une phrase traduite dans
 * la coquille de l'écran — « vous n'avez pas accès à ceci » —, pas une redirection qui laisse
 * croire à une déconnexion. Les gardes de permission restent donc au niveau page, où `t()` existe.
 * Ici, il n'y a ni composant ni portée d'effet : `useI18n()` n'y rendrait rien, et `useEtatReseau()`
 * y fuirait ses écouteurs faute de `onScopeDispose` — d'où la lecture directe de
 * `adaptateurCourant().etatReseau()`.
 */

import { reprendreSession, sessionCourante } from '~/core/auth'
import { ROUTE_STYLEGUIDE } from '~/core/design-system/montage'
import { adaptateurCourant } from '~/core/platform/courant'

/**
 * Les deux seules routes qui n'exigent pas de session, **nommées une par une**.
 *
 * Un motif — « tout ce qui commence par `/public` » — laisserait passer toute route future qui s'y
 * conformerait par accident. Chaque exemption s'écrit, donc se justifie :
 *
 * - `/connexion` est l'écran par lequel tout le monde entre. L'exempter n'est pas une commodité :
 *   sans elle, la redirection vers `/connexion` se redéclencherait à l'infini.
 * - `/styleguide` est retiré du routeur hors développement (`nuxt.config.ts`, `pages:extend`).
 *   L'entrée est donc **inerte en production**, et la constante vient de `core/design-system/montage`
 *   plutôt que d'un littéral — c'est la raison d'être de ce module : le retrait de route et la page
 *   ne doivent jamais diverger.
 */
const SANS_SESSION: ReadonlySet<string> = new Set([ROUTE_STYLEGUIDE, '/connexion'])

export default defineNuxtRouteMiddleware(async (to) => {
  // Sur `to.path`, jamais `to.fullPath` : `/etablissement` porte déjà `?etablissement=…`, et une
  // comparaison sur la chaîne complète cesserait de reconnaître la route au premier paramètre.
  if (SANS_SESSION.has(to.path)) {
    return
  }
  if (sessionCourante()) {
    return
  }

  // Reprise silencieuse. Redemander le mot de passe à chaque ouverture d'onglet le ferait écrire
  // sur un papier au comptoir — c'est le raisonnement du cycle 003, appliqué ici à toutes les
  // routes plutôt qu'à une seule.
  const config = useRuntimeConfig()
  await reprendreSession(config.public.apiBaseUrl, adaptateurCourant().etatReseau())

  if (sessionCourante()) {
    return
  }

  // `return navigateTo(...)`, jamais `abortNavigation()` ni `return false`. Sur la navigation
  // initiale d'une application sans rendu serveur, ces deux-là n'annulent pas : ils affichent la
  // page d'erreur « Page Not Found ». Un utilisateur dont la session a expiré verrait une 404 au
  // lieu de l'écran de connexion.
  return navigateTo('/connexion')
})
