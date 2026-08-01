<script setup lang="ts">
/**
 * **Le cadre à deux volets du styleguide** — le même contenu, rendu en clair et en sombre.
 *
 * Ce n'est **pas un composant canonique** : il ne figure pas dans les seize de
 * `docs/design/composants.md` et aucun écran du produit ne l'emploie. C'est l'outil qui rend le
 * styleguide vérifiable, et il vit ici plutôt que dans `pages/` parce que `pages/` porte des
 * routes, pas des pièces.
 *
 * # Pourquoi deux volets plutôt qu'une bascule
 *
 * La Definition of Done, point 8, demande que chaque écran soit vérifié « en mode clair **et** en
 * mode sombre ». Une bascule oblige à mémoriser ce qu'on vient de voir ; deux volets côte à côte
 * font apparaître l'écart. C'est le choix de `docs/design/styleguide.html`, et il est repris ici
 * pour la même raison.
 *
 * Le volet sombre porte la classe `.dark` sur son conteneur, pas sur `<html>` :
 * `theme.css` déclare `@custom-variant dark (&:where(.dark, .dark *))`, ce qui rend le mode sombre
 * **local à un sous-arbre**. C'est cette propriété qui permet d'afficher les deux à la fois.
 *
 * # Le slot est rendu DEUX FOIS
 *
 * Conséquence à connaître avant d'y mettre quoi que ce soit : un échantillon qui porte un état
 * (un `v-model`, un compteur) verrait ses deux copies évoluer indépendamment. Les échantillons du
 * styleguide sont donc sans état partagé — ce qu'ils sont naturellement, puisqu'ils montrent des
 * apparences.
 */

defineProps<{
  /** Libellé du volet clair — reçu **résolu**, pas sous forme de clé : voir l'en-tête de `pages/styleguide.vue`. */
  libelleClair: string
  /** Libellé du volet sombre. */
  libelleSombre: string
}>()
</script>

<template>
  <div class="grid grid-cols-1 overflow-hidden rounded-xl border border-line xl:grid-cols-2">
    <div class="flex flex-col gap-5 border-b border-line bg-bg p-6 xl:border-b-0 xl:border-r">
      <span class="inline-flex items-center gap-2 text-etiquette font-semibold text-ink-3 uppercase">
        <i
          class="ph ph-sun-horizon text-corps"
          aria-hidden="true"
        />{{ libelleClair }}
      </span>
      <slot />
    </div>

    <div class="dark flex flex-col gap-5 border-line bg-bg p-6">
      <span class="inline-flex items-center gap-2 text-etiquette font-semibold text-ink-3 uppercase">
        <i
          class="ph ph-moon-stars text-corps"
          aria-hidden="true"
        />{{ libelleSombre }}
      </span>
      <slot />
    </div>
  </div>
</template>
