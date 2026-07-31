<script setup lang="ts">
// Route racine — présente pour que `nuxt build` produise une application valide.
// Elle n'affiche aucune donnée : le premier écran réel vient du cycle ETB.
const { t } = useI18n()
</script>

<template>
  <main class="flex min-h-screen items-center justify-center p-lg">
    <p class="font-texte text-corps text-ink-doux">{{ t('app.aucun_ecran') }}</p>
  </main>
</template>
