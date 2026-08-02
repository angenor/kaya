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
 * # Ce que la coquille NE porte pas encore, et pourquoi
 *
 * Ni barre de contexte unifiée, ni témoin de synchronisation. Ce ne sont pas des oublis :
 *
 * - `EcranAccueil` et `EcranEtablissement` portent chacun un `<header>` **différent** — pastille
 *   d'initiale et nom d'utilisateur d'un côté, icône d'établissement et résumé de l'autre, sur deux
 *   hauteurs distinctes. Les fondre en une barre unique est un **changement d'écran**, et
 *   `docs/design/derivation.md` est opposable : un écran hors de la matrice ne se code pas. Les
 *   deux en-têtes restent donc dans leur écran, sous ce `<main>`.
 * - Le témoin de synchronisation est le **composant 10**, et `docs/module-dore.md` le déclare
 *   toujours dû à ETB-06. Prêt n'est pas construit (principe X).
 */
</script>

<template>
  <main class="flex min-h-screen flex-col bg-bg">
    <slot />
  </main>
</template>
