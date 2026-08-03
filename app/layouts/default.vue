<script setup lang="ts">
/**
 * **La coquille de l'application.** *Point d'amorçage n° 3.*
 *
 * # Ce que ce fichier corrige — et c'est la cause du défaut, pas un rangement
 *
 * Quatre pages sur six rendaient chacune leur propre `<main>`, dans la branche `v-else` d'un
 * couple `v-if`/`v-else` posé **à la racine du template**. Une racine multiple compile en
 * *fragment*, et un fragment dont la branche active devient un `defineAsyncComponent` non encore
 * résolu a un `el` **nul**. Au rendu suivant, Vue appelle `hostParentNode(prevTree.el)` et lève :
 *
 * ```
 * TypeError: Cannot read properties of null (reading 'parentNode')
 *     at ReactiveEffect.componentUpdateFn
 * ```
 *
 * La page ne se monte pas, l'ancienne reste à l'écran, et l'adresse a pourtant changé. C'est ce
 * qui rendait `G3` et `G4` inatteignables.
 *
 * **Les trois conditions ont été isolées une par une, sur des pages sondes** — le tableau ci-dessous
 * est le résultat de l'expérience, pas une déduction :
 *
 * | Racine du template | Composant | Bascule après montage | Erreur |
 * |---|---|---|---|
 * | fragment | paresseux | non | — |
 * | fragment | **paresseux** | **oui** | **`parentNode`** |
 * | fragment | synchrone | oui | — |
 * | **élément unique** | paresseux | oui | — |
 *
 * Il faut les **trois** réunies. Une racine unique suffit à l'éliminer — **sans toucher au
 * chargement paresseux**, que le principe VII exige module par module : « un serveur de salle ne
 * télécharge pas le code du back-office ».
 *
 * # Pourquoi un layout plutôt qu'un `<div>` recopié dans chaque page
 *
 * Parce que la recopie était déjà la faute. Le `<main class="flex min-h-screen …">` et son
 * squelette de chargement étaient écrits quatre fois, à quatre nuances près ; la cinquième page
 * l'aurait oublié comme les cinq avaient oublié la reprise de session. **Un layout le rend
 * structurel** : une page nouvelle hérite de la coquille sans rien écrire, et ne peut pas oublier.
 *
 * Deux règles portent ce fichier, et aucune n'est cosmétique :
 *
 * - **Racine unique**, exigée par Nuxt pour appliquer une transition entre layouts, et par le
 *   mécanisme ci-dessus.
 * - `<NuxtLayout>` **n'est pas la racine d'`app.vue`** : il rend son `<slot>` dans un
 *   `<Transition>`, et le poser en racine reproduit la famille de défauts qu'on vient de fermer.
 *   Le `<div>` d'`app.vue` reste donc, et l'enveloppe.
 *
 * ═══════════════════════════════════════════════════════════════════════════════════════════════
 *  LE PIED DE COQUILLE — « passer la main », et pourquoi il est ici et pas dans un écran
 * ═══════════════════════════════════════════════════════════════════════════════════════════════
 *
 * `fermerSession()` vivait dans `core/auth` depuis le cycle CPT **sans aucun appelant** : il
 * n'existait, littéralement, aucun moyen de sortir de sa session. Sur un terminal de comptoir, où
 * l'appareil ne bouge pas et où c'est la personne qui change, toute action de Yao entrait au
 * **journal d'audit au nom d'Aminata** — le registre dont le cadrage §8.3 dit qu'il est « ce que
 * le propriétaire achète ». Un audit qui attribue les actes à la mauvaise personne est pire qu'un
 * audit absent : le premier trompe, le second se sait manquant.
 *
 * **Il est dans la coquille pour la même raison que la reprise de session est dans le middleware.**
 * Le mettre dans un écran obligerait chaque écran suivant à s'en souvenir, et c'est exactement la
 * faute que ce fichier a réparée : cinq pages sur six avaient oublié la ligne qu'elles devaient
 * recopier. Ici, une page nouvelle hérite du geste sans rien écrire.
 *
 * # Ce que le pied de coquille NE construit PAS, et c'est délibéré
 *
 * **Ce n'est pas ETB-06.** La barre de contexte, le sélecteur d'établissement et le témoin de
 * synchronisation (composant 10) restent dus par cette story. Fondre les en-têtes d'`R1` et de
 * `G1` en une barre unique serait un **changement d'écran**, et `docs/design/derivation.md` est
 * opposable : un écran hors de la matrice ne se code pas. D'où un **pied**, et non un en-tête —
 * `R1` porte un `<header>` de 64 px avec pastille d'identité, `G1` un de 58 px avec carte
 * d'établissement, `G3` et `G4` n'en ont aucun. Se poser sous eux ne touche à aucun des trois ;
 * se poser au-dessus les aurait tous rouverts.
 *
 * ═══════════════════════════════════════════════════════════════════════════════════════════════
 *  LE TÉMOIN DE SYNCHRONISATION — cycle 005
 * ═══════════════════════════════════════════════════════════════════════════════════════════════
 *
 * Le composant 10 est monté ici, et non dans un écran. « Indicateur permanent de l'état réseau »
 * (principe VI) veut dire **présent sur toutes les pages**, et la seule façon de le garantir est
 * de le poser dans la coquille : une page nouvelle en hérite sans rien écrire, et ne peut pas
 * l'oublier.
 *
 * Le commentaire ci-dessus disait, au cycle 004, que le témoin « reste dû par ETB-06 ». Il
 * l'était en tant qu'élément de la **barre de contexte** — avec le sélecteur d'établissement,
 * qui, lui, reste dû. Le témoin seul, en pied, ne touche à aucun en-tête maquetté : c'est la même
 * précaution qui a mis « passer la main » en bas plutôt qu'en haut.
 *
 * # Le refus quand la file n'est pas vide — posé AVANT que la file existe
 *
 * `fermerSession()` purge le stockage de la plateforme (principe VI, cadrage §11.5 règle 5). Le
 * jour où SYN-01 branchera la file hors-ligne, des écritures en attente y passeraient avec le
 * reste — et la serveuse apprendrait au service suivant que ses quatre commandes n'existent pas.
 *
 * La garde est donc posée **maintenant**, sur `ecrituresEnAttente()`, qui rend `0` tant qu'aucune
 * file n'est branchée. Ce n'est pas une supposition : `app/tests/deconnexion.spec.ts` le vérifie
 * dans les deux sens — 0 à l'état débranché, et le compte réel dès qu'une file est branchée.
 * Attendre SYN pour poser cette question obligerait ce cycle à revenir ici, dans un code qui parle
 * d'autre chose.
 *
 * # Conséquence assumée : le thème retenu part avec la purge
 *
 * `purger()` retire **toutes** les clés `kaya.` — c'est la décision de `stockage-web.ts`, et elle
 * est juste : un jeton retiré à côté d'un cache de comptes serait une demi-purge. `kaya.theme` en
 * fait partie. Passer la main remet donc l'appareil sur le thème du système. Sur un poste partagé
 * c'est le comportement souhaitable ; sur un téléphone personnel c'est une préférence perdue. Écrit
 * ici plutôt que découvert : restreindre la purge pour la sauver affaiblirait la seule chose qui
 * protège les données d'identité des clients.
 */
import { ref, watch } from 'vue'

import TemoinSynchronisation from '~/core/design-system/TemoinSynchronisation.vue'
import { fermerSession, sessionCourante, type SessionUtilisateur } from '~/core/auth'
import { ecrituresEnAttente } from '~/core/sync'

const { t } = useI18n()
const config = useRuntimeConfig()
const route = useRoute()

/**
 * La session, **relue à chaque navigation**.
 *
 * `sessionCourante()` lit une variable de module, qui n'est pas réactive — c'est un choix de
 * `core/auth/session.ts`, et il tient. Mais la coquille, elle, **survit aux navigations** : lue
 * une seule fois au montage, elle garderait pour toujours l'état du premier rendu, et le bouton
 * resterait absent après une connexion réussie comme présent après une déconnexion.
 *
 * Le chemin de route est la dépendance qui convient parce que **les deux transitions passent par
 * lui** : `R0` termine par `navigateTo('/')`, et ce fichier par `navigateTo('/connexion')`. Poser
 * un magasin réactif dans `core/auth` donnerait une seconde source pour la même information.
 */
const session = ref<SessionUtilisateur | null>(null)

watch(() => route.fullPath, () => {
  session.value = sessionCourante()
}, { immediate: true })

/** L'appel en cours. Empêche la double soumission d'un geste ; ne dissimule aucune indisponibilité. */
const enCours = ref(false)

/** Refus de la dernière tentative — clé i18n. Un seul motif possible aujourd'hui. */
const refus = ref<string | null>(null)

/**
 * **Passer la main** — le geste, dans l'ordre où il doit se faire.
 *
 * L'ordre n'est pas indifférent : la garde d'abord (rien n'est détruit si elle refuse), l'appel
 * serveur ensuite (il révoque le jeton de rafraîchissement pendant qu'on a encore de quoi
 * s'authentifier), la purge locale enfin — les deux dernières sont dans `fermerSession`, qui les
 * tient ensemble et efface l'état local **même si le réseau a manqué** : quelqu'un qui demande à
 * quitter son poste doit quitter son poste.
 *
 * La navigation vient en dernier, **après** que la purge est terminée. L'inverse laisserait `R0`
 * se monter pendant que le stockage se vide, et le middleware global pourrait y reprendre une
 * session qu'on est en train de détruire.
 */
async function passerLaMain(): Promise<void> {
  refus.value = null

  // **Refus immédiat, avant toute destruction** (principe VI) — jamais un échec après coup.
  if (ecrituresEnAttente() > 0) {
    refus.value = 'deconnexion.refus.en_attente'
    return
  }

  enCours.value = true
  try {
    await fermerSession(config.public.apiBaseUrl)
  }
  finally {
    enCours.value = false
  }

  session.value = null
  await navigateTo('/connexion')
}
</script>

<template>
  <main class="flex min-h-screen flex-col bg-bg">
    <slot />

    <!-- Le pied n'existe que sous session : sur `R0` et sur le styleguide, il n'y a rien à
         quitter. ABSENT, jamais grisé (principe VII). -->
    <footer
      v-if="session"
      class="mt-auto flex flex-col items-end gap-2 px-4 py-3"
    >
      <!-- COMPOSANT 10 · TÉMOIN DE SYNCHRONISATION — « le composant le plus important du
           produit ». Il est ICI, dans la coquille, donc sur TOUTES les pages : c'est ce que
           « indicateur permanent » veut dire (principe VI). Le monter écran par écran
           garantirait qu'un écran l'oublie, et ce serait celui où l'on écrit.

           Il n'est pas dans un en-tête, et c'est la même raison qui a mis « passer la main » en
           pied : `R1` porte un `<header>` de 64 px, `G1` un de 58 px, `G3` et `G4` n'en ont
           aucun. Se poser au-dessus les aurait tous rouverts, et `docs/design/derivation.md` est
           opposable — un écran hors de la matrice ne se code pas. -->
      <TemoinSynchronisation />
      <!-- COMPOSANT 07 · BANDEAU — contrefort, fond `-soft`, texte `-fort`, une phrase. Il se
           pose à côté du bouton qui vient de refuser, pas en haut d'un écran qu'on ne regarde
           plus. -->
      <p
        v-if="refus"
        class="flex items-start gap-2.5 rounded-r-lg border-l-4 border-l-alerte bg-alerte-soft p-3 font-texte text-mini text-alerte-fort"
        role="alert"
      >
        <i
          class="ph ph-warning shrink-0 text-corps text-alerte"
          aria-hidden="true"
        />
        {{ t('deconnexion.refus.en_attente') }}
      </p>

      <!-- COMPOSANT 03 · BOUTON DISCRET — sans fond ni contour au repos, 36 px de haut, il ne
           quitte pas son bloc. C'est une action de bord, pas l'action qui fait avancer la
           journée. -->
      <button
        type="button"
        class="inline-flex h-9 cursor-pointer items-center gap-2 rounded-md px-3.5 font-titre text-mini font-semibold text-ink-2 transition-colors duration-90 hover:bg-tile hover:text-ink active:scale-97"
        :title="t('deconnexion.effet')"
        :disabled="enCours"
        @click="passerLaMain"
      >
        <i
          class="ph ph-sign-out text-corps"
          aria-hidden="true"
        />
        {{ enCours ? t('deconnexion.en_cours') : t('deconnexion.action') }}
      </button>
    </footer>
  </main>
</template>
