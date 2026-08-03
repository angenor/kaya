/**
 * **Le point unique d'où sort tout appel d'API de l'application.**
 *
 * # Pourquoi ce fichier existe, et ce qu'il empêche
 *
 * `creerClientKaya(baseUrl)` était appelé directement par chaque module d'écriture — dix
 * emplacements, dix clients distincts. Tant qu'un client n'était qu'un `fetch` typé, la
 * multiplication ne coûtait rien.
 *
 * Elle coûte à partir de ce cycle. Le témoin de synchronisation doit dire « connexion faible »
 * quand le réseau répond mal, et cette information ne peut venir que des **appels réels**. Un
 * observateur branché sur un client sur dix ne verrait qu'un dixième de la vérité — et un témoin
 * qui ment exactement quand il compte est pire qu'un témoin absent.
 *
 * D'où un point unique : **le client observe, et il observe tout**.
 *
 * # Ce que l'observation mesure, et ce qu'elle ne juge pas
 *
 * | Ce qui est relevé | Ce qui en est déduit |
 * |---|---|
 * | L'appel a-t-il abouti — quel que soit le code ? | Un `422` a abouti : le serveur a répondu, le réseau va bien |
 * | Sa durée | Au-delà du seuil paramétré, le réseau est « faible » |
 *
 * **Un refus métier n'est pas un problème de réseau.** Confondre les deux ferait passer le témoin
 * au rouge parce qu'une validation a refusé une saisie, ce qui est le contraire de l'information
 * utile.
 *
 * # Ce fichier ne met rien en cache, et c'est délibéré
 *
 * Une instance par appelant coûte un objet ; une instance partagée introduirait une question de
 * cycle de vie — que devient-elle à la déconnexion, quand la base d'API change, quand deux
 * établissements sont ouverts ? Le middleware, lui, est **partagé et sans état par appel** :
 * c'est la seule chose qui doit l'être.
 */

import { creerClientKaya, type Middleware } from '@kaya/client'

import { observerAppel } from '~/core/platform/observateur-appels'

/**
 * L'intercepteur qui alimente l'état réseau.
 *
 * # Pourquoi la durée est mesurée ici et non dans l'appelant
 *
 * Un appelant qui chronométrerait mesurerait aussi son propre travail — désérialisation, mise en
 * forme, rendu. Ce qui intéresse le témoin est **l'aller-retour**, et lui seul : c'est la seule
 * grandeur qui parle du réseau plutôt que de l'application.
 *
 * `onRequest` et `onResponse` reçoivent le même `id` d'appel, ce qui permet d'apparier départ et
 * arrivée même quand plusieurs appels se croisent — le cas normal sur un écran qui charge trois
 * listes.
 */
function observateurReseau(): Middleware {
  const departs = new Map<string, number>()

  const maintenant = (): number =>
    typeof performance !== 'undefined' ? performance.now() : Date.now()

  const conclure = (id: string, abouti: boolean): void => {
    const depart = departs.get(id)
    departs.delete(id)
    if (depart === undefined) {
      return
    }
    observerAppel({ abouti, dureeMs: maintenant() - depart })
  }

  return {
    onRequest({ id }) {
      departs.set(id, maintenant())
    },
    onResponse({ id }) {
      // **Abouti quel que soit le code.** Le serveur a répondu : le réseau a fait son travail, et
      // c'est ce que le témoin rapporte. Ce que le serveur a répondu regarde l'appelant.
      conclure(id, true)
    },
    onError({ id }) {
      // Ni réponse ni code : le réseau n'a pas porté l'appel. C'est le cas d'Abengourou — la
      // plateforme dit « en ligne » et rien ne passe.
      conclure(id, false)
    },
  }
}

/** L'intercepteur, construit une fois. Sans état entre appels : sa `Map` se vide à chaque issue. */
const OBSERVATEUR = observateurReseau()

/**
 * Le client typé de l'application, **avec l'observation du réseau branchée**.
 *
 * Tout module qui appelle l'API passe par ici. Importer `creerClientKaya` directement fonctionne
 * — et rend l'appel invisible au témoin, ce que `app/tests/temoin-sync.spec.ts` refuse.
 */
export function clientKaya(baseUrl: string) {
  return creerClientKaya(baseUrl, [OBSERVATEUR])
}
