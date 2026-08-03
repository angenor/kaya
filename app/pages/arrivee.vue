<script setup lang="ts">
/**
 * **`R3` — Arrivée.** Écran **DÉRIVÉ** de `R4`, cas (b) de `docs/Kaya_Design.md` §2.
 *
 * Référence : `docs/design/derivation.md`, ligne `R3`. La matrice est **opposable** — un écran qui
 * n'y figure pas et n'a pas de maquette ne se code pas (porte P-19).
 *
 * # La route est `/arrivee`, et le nom du fichier la décide
 *
 * Jamais `/check-in` : « check-in » est **écarté du lexique** (v1.6.0), et **une URL est visible**
 * dans la barre d'adresse. C'est la leçon `S1` du cycle 005 — le mot proscrit rentrerait par la
 * porte du nom de fichier, sans qu'aucune porte i18n ne le voie.
 *
 * # ⚠️ UNE SEULE RACINE, ET C'EST UN ÉLÉMENT
 *
 * Jamais un `v-if`/`v-else` de premier niveau. Une racine multiple compile en fragment ; un
 * fragment dont la branche active est un `defineAsyncComponent` non résolu a un `el` nul, et Vue
 * lève `Cannot read properties of null (reading 'parentNode')` **à la navigation suivante** —
 * l'écran ne se monte pas, l'ancien reste affiché, et l'adresse a pourtant changé. C'est la leçon
 * la plus chère du cycle 003.
 *
 * # L'entrée est ABSENTE sans le module `HEBERGEMENT` (principe VII)
 *
 * L'accueil filtre ses tuiles par module actif **et** par permission ; un maquis ne voit pas cette
 * route. Elle n'est pas grisée : un module inactif est **absent**, jamais promis.
 */
import { computed, defineAsyncComponent, onMounted, ref } from 'vue'

import { contexteAppel, sessionCourante, type ContexteAppel } from '~/core/auth'
import type { Permissions } from '~/core/rbac'
import type { ClientResume, DonneesArrivee } from '~/modules/sejours/donnees'

const EcranArrivee = defineAsyncComponent(
  () => import('~/modules/sejours/EcranArrivee.vue'),
)

const { t } = useI18n()
const route = useRoute()
const config = useRuntimeConfig()

const donnees = ref<DonneesArrivee | null>(null)
const erreur = ref<string | null>(null)

/**
 * La fiche retenue au comptoir.
 *
 * ⚠️ **Elle arrive par la recherche, jamais par une reconnaissance automatique.** Deviner qui est
 * en face à partir d'un numéro rattacherait un séjour à la mauvaise fiche une fois sur cent, et
 * l'erreur ne se verrait qu'au départ — trop tard pour la corriger sans toucher un document légal.
 */
const clientRetenu = ref<ClientResume | null>(null)

const contexte = computed<ContexteAppel | null>(() => contexteAppel(config.public.apiBaseUrl))
const permissions = computed<Permissions>(() => sessionCourante()?.permissions ?? [])

/** La SESSION d'abord, la configuration en dernier recours — l'ordre imposé par P-22. */
const etablissementId = computed(() => String(
  route.query.etablissement ?? sessionCourante()?.etablissementId ?? config.public.etablissementId,
))

onMounted(async () => {
  if (!contexte.value) {
    erreur.value = t('connexion.requise')
    return
  }

  const { chargerArrivee } = await import('~/modules/sejours/donnees')
  try {
    donnees.value = await chargerArrivee(contexte.value, etablissementId.value)
  }
  catch {
    erreur.value = t('sejours.arrivee.chargement_impossible')
  }
})
</script>

<template>
  <div class="flex flex-1 flex-col p-4 sm:p-6">
    <EcranArrivee
      v-if="donnees && contexte"
      :contexte="contexte"
      :etablissement-id="etablissementId"
      :donnees="donnees"
      :permissions="permissions"
      :client-retenu="clientRetenu"
      @retenir-client="clientRetenu = $event"
      @oublier-client="clientRetenu = null"
    />
    <div
      v-else
      class="flex flex-1 items-center justify-center p-6"
    >
      <p class="font-texte text-corps text-ink-2">
        {{ erreur ?? t('sejours.arrivee.chargement') }}
      </p>
    </div>
  </div>
</template>
