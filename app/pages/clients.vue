<script setup lang="ts">
/**
 * **`R5` — Fiche client et recherche.** Écran **DÉRIVÉ** de `R7`, cas (b) de
 * `docs/Kaya_Design.md` §2.
 *
 * Référence : `docs/design/derivation.md`, ligne `R5`. La matrice est **opposable** — un écran qui
 * n'y figure pas et n'a pas de maquette ne se code pas (porte P-19).
 *
 * # ★ CETTE ENTRÉE RESTE DISPONIBLE SANS MODULE HÉBERGEMENT
 *
 * La fiche client ne dépend d'**aucun** module d'activité : elle est du **tenant** (FR-002), et
 * ses deux permissions sont transversales (`module_code = NULL`, migration `0030`). Un maquis, un
 * bar seul, un pressing en auront besoin dès SEJ-05. La filtrer sur `HEBERGEMENT` serait
 * reproduire dans l'interface exactement la contamination du noyau par l'hôtellerie que le
 * principe II interdit au backend.
 *
 * # ⚠️ UNE SEULE RACINE, ET C'EST UN ÉLÉMENT
 *
 * Jamais un `v-if`/`v-else` de premier niveau. Une racine multiple compile en fragment ; un
 * fragment dont la branche active est un `defineAsyncComponent` non résolu a un `el` nul, et Vue
 * lève `Cannot read properties of null (reading 'parentNode')` **à la navigation suivante**.
 * C'est la leçon la plus chère du cycle 003.
 *
 * # Aucun chargement au montage, et c'est délibéré
 *
 * L'écran s'ouvre **vide**, sur son champ de recherche. Précharger « les derniers clients »
 * paraîtrait accueillant et ferait lire des fiches que personne n'a demandées — donc journaliser
 * des consultations de pièces d'identité que personne n'a voulues (FR-012).
 */
import { computed, defineAsyncComponent } from 'vue'

import { contexteAppel, sessionCourante, type ContexteAppel } from '~/core/auth'
import type { Permissions } from '~/core/rbac'

const EcranClients = defineAsyncComponent(
  () => import('~/modules/sejours/EcranClients.vue'),
)

const { t } = useI18n()
const config = useRuntimeConfig()

const contexte = computed<ContexteAppel | null>(() => contexteAppel(config.public.apiBaseUrl))
const permissions = computed<Permissions>(() => sessionCourante()?.permissions ?? [])
</script>

<template>
  <div class="flex flex-1 flex-col p-4 sm:p-6">
    <EcranClients
      v-if="contexte"
      :contexte="contexte"
      :permissions="permissions"
    />
    <div
      v-else
      class="flex flex-1 items-center justify-center p-6"
    >
      <p class="font-texte text-corps text-ink-2">
        {{ t('connexion.requise') }}
      </p>
    </div>
  </div>
</template>
