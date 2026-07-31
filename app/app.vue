<script setup lang="ts">
// Coquille de l'application unique.
//
// **Ce cycle ne produit aucun écran, et c'est une décision vérifiée, pas une omission** : les
// onze codes d'écran maquettés de `docs/design/html/` sont C4, F2, G2, M4, P2, Q1, R1, R4, R7,
// S2 et V1 — aucun ne couvre les notes d'établissement — et la matrice de dérivation
// `docs/design/derivation.md` n'a aucune ligne pour cet écran. « Un écran qui n'hérite d'aucun
// motif ne se code pas » (principe XII).
//
// La couche écran du module doré est donc reportée au cycle ETB, qui dispose d'écrans réellement
// maquettés (G2, M4). Voir `docs/module-dore.md`, section « La septième couche, et pourquoi elle
// manque ».
const { t } = useI18n()
</script>

<template>
  <div class="min-h-screen bg-surf text-ink">
    <NuxtPage />
    <p class="sr-only">{{ t('app.coquille') }}</p>
  </div>
</template>
