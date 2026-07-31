<script setup lang="ts">
/**
 * **Section « Identité visuelle »** avec aperçu — ETB-05.
 *
 * # Aucune capacité native, donc rien de nouveau pour la porte P-15
 *
 * Le choix de fichier passe par un `<input type="file">` **standard**. Aucun `window.__TAURI__`,
 * aucune extension de `PlatformAdapter` : la règle ESLint `no-restricted-imports` n'a rien à
 * garder ici, et c'est délibéré.
 *
 * # `couleur_primaire` n'est JAMAIS appliquée à l'interface (FR-059)
 *
 * C'est une **donnée client**, pas un jeton de design. Elle s'applique aux **documents produits**,
 * et à eux seuls. Elle est donc affichée ici comme une **valeur** — un code hexadécimal lisible —
 * jamais posée en style. L'appliquer à un bouton ferait prendre au produit la couleur de chaque
 * client, et la porte P-17 la verrait passer pour une couleur littérale.
 */
import { ref } from 'vue'

const { t } = useI18n()

defineProps<{ nomEtablissement: string }>()

/** L'aperçu rendu par le serveur — il n'enregistre rien (FR-057). */
const apercu = ref<string | null>(null)
</script>

<template>
  <section class="flex flex-col">
    <div class="flex flex-col gap-1.5 px-3.5 pt-4 pb-1">
      <span class="text-etiquette uppercase text-ink-3">{{ t('etablissement.sur_titre') }}</span>
      <h2 class="font-titre text-chiffre font-semibold text-ink">
        {{ t('etablissement.identite_visuelle.titre') }}
      </h2>
      <p class="text-corps text-ink-2">
        {{ t('etablissement.identite_visuelle.intro') }}
      </p>
    </div>

    <div class="flex flex-col gap-2.25 px-3 pt-3 pb-3.5">
      <label
        class="flex w-full cursor-pointer items-center gap-3 rounded-l-xs rounded-r-xl border border-line border-l-4 border-l-line-2 bg-surf p-3 shadow-basse"
      >
        <span
          class="inline-flex size-10 shrink-0 items-center justify-center rounded-xl bg-tile"
        >
          <i
            class="ph ph-image text-titre-m text-ocre"
            aria-hidden="true"
          />
        </span>
        <span class="flex min-w-0 flex-1 flex-col items-start gap-0.75 text-left">
          <span class="font-titre text-titre-s font-semibold text-ink">
            {{ t('etablissement.identite_visuelle.logo') }}
          </span>
          <span class="text-mini text-ink-3">
            {{ t('etablissement.identite_visuelle.logo_limite') }}
          </span>
        </span>
        <!-- `<input type="file">` standard — aucun pont natif. -->
        <input
          type="file"
          accept="image/*"
          class="sr-only"
        >
      </label>
    </div>

    <div class="flex shrink-0 flex-col gap-2 border-t border-line bg-surf px-3 pt-2.75 pb-3.5">
      <button
        type="button"
        class="inline-flex h-13 w-full cursor-pointer items-center justify-center gap-2.5 rounded-xl bg-prim font-titre text-titre-s font-semibold text-prim-ink shadow-bouton-grand transition-[transform,box-shadow] duration-90 ease-entree active:translate-y-0.75 active:shadow-none"
      >
        <i
          class="ph ph-eye text-titre-m"
          aria-hidden="true"
        />
        {{ t('etablissement.identite_visuelle.action_apercu') }}
      </button>
    </div>

    <!-- L'aperçu s'affiche SANS enregistrement préalable. La mention non fiscale vient du
         serveur, qui la concatène toujours : la reproduire ici en ferait une seconde source
         susceptible de diverger. -->
    <pre
      v-if="apercu"
      class="mx-3 mb-3.5 overflow-x-auto rounded-xl border border-line bg-tile p-3 font-mono text-mini text-ink-2"
    >{{ apercu }}</pre>
  </section>
</template>
