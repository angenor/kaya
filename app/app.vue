<script setup lang="ts">
// **Le point de montage de l'application.**
//
// Ce fichier a longtemps porté un commentaire du cycle 001 — « ce cycle ne produit aucun écran » —
// resté là deux cycles après que ce fut faux. Il portait aussi, et c'est plus grave, la totalité
// de l'amorçage de l'application : rien. Ni plugin, ni middleware, ni layout. Chaque page
// amorçait pour elle-même ce qu'elle avait pensé à amorcer, et cinq sur six avaient oublié la
// reprise de session.
//
// L'amorçage vit désormais aux trois endroits que Nuxt prévoit pour lui, et il n'y a plus rien à
// faire ici qu'à les monter :
//
//   plugins/01.theme.client.ts       le thème, avant le premier rendu
//   middleware/01.session.global.ts  la reprise de session, avant chaque navigation
//   layouts/default.vue              la coquille : une racine stable, un seul `<main>`
//
// **`<NuxtLayout>` n'est PAS la racine, et ce `<div>` n'est pas décoratif.** NuxtLayout rend son
// `<slot>` enveloppé dans un `<Transition>` ; le poser en racine reproduit la famille de défauts
// que `layouts/default.vue` documente — un nœud dont le `parentNode` est nul au moment de la
// transition. Le `<div>` l'enveloppe et lui donne un parent qui ne bouge pas.
const { t } = useI18n()
</script>

<template>
  <div class="min-h-screen bg-surf text-ink">
    <NuxtLayout>
      <NuxtPage />
    </NuxtLayout>
    <p class="sr-only">
      {{ t('app.coquille') }}
    </p>
  </div>
</template>
