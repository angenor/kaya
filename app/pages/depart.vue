<script setup lang="ts">
/**
 * **`R7` — La note et le départ.** Écran **MAQUETTÉ**, cas (a) de `docs/Kaya_Design.md` §2.
 *
 * Référence : `docs/design/html/R7-note-depart.html`.
 *
 * # ★ LA ROUTE N'A AUCUN SEGMENT DYNAMIQUE, ET C'EST DÉLIBÉRÉ
 *
 * Le séjour choisi passe en **paramètre de requête** — `/depart?sejour=…` — jamais en segment de
 * chemin. Une route sans paramètre est **couvrable par P-22** : la porte ouvre chaque route en
 * direct et par navigation, et une route `/depart/{id}` exigerait d'inventer un identifiant
 * valide pour être visitée, donc de faire dépendre une porte de parcours de l'état des données.
 * Elle deviendrait rouge le jour où les seeds changeraient, pour une raison sans rapport avec ce
 * qu'elle mesure.
 *
 * # ⚠️ UNE SEULE RACINE, ET C'EST UN ÉLÉMENT
 *
 * Jamais un `v-if`/`v-else` de premier niveau. Une racine multiple compile en fragment ; un
 * fragment dont la branche active est un `defineAsyncComponent` non résolu a un `el` nul, et Vue
 * lève `Cannot read properties of null (reading 'parentNode')` **à la navigation suivante**.
 * C'est la leçon la plus chère du cycle 003.
 *
 * # L'entrée est ABSENTE sans le module `HEBERGEMENT` (principe VII)
 */
import { computed, defineAsyncComponent, onMounted, ref } from 'vue'

import { contexteAppel, sessionCourante, type ContexteAppel } from '~/core/auth'
import type { Permissions } from '~/core/rbac'
import type { DonneesDepart } from '~/modules/sejours/donnees'

const EcranDepart = defineAsyncComponent(
  () => import('~/modules/sejours/EcranDepart.vue'),
)

const { t } = useI18n()
const route = useRoute()
const config = useRuntimeConfig()

const donnees = ref<DonneesDepart | null>(null)
const erreur = ref<string | null>(null)

const contexte = computed<ContexteAppel | null>(() => contexteAppel(config.public.apiBaseUrl))
const permissions = computed<Permissions>(() => sessionCourante()?.permissions ?? [])

/** La SESSION d'abord, la configuration en dernier recours — l'ordre imposé par P-22. */
const etablissementId = computed(() => String(
  route.query.etablissement ?? sessionCourante()?.etablissementId ?? config.public.etablissementId,
))

/** Le séjour ouvert d'emblée, s'il y en a un. **Facultatif** : la route s'ouvre sans lui. */
const sejourInitial = computed(() => {
  const brut = route.query.sejour
  return typeof brut === 'string' && brut.length > 0 ? brut : null
})

onMounted(async () => {
  if (!contexte.value) {
    erreur.value = t('connexion.requise')
    return
  }

  const { chargerDepart } = await import('~/modules/sejours/donnees')
  try {
    donnees.value = await chargerDepart(contexte.value, etablissementId.value)
  }
  catch {
    erreur.value = t('sejours.depart.chargement_impossible')
  }
})
</script>

<template>
  <div class="flex flex-1 flex-col p-4 sm:p-6">
    <EcranDepart
      v-if="donnees && contexte"
      :contexte="contexte"
      :etablissement-id="etablissementId"
      :donnees="donnees"
      :permissions="permissions"
      :sejour-initial="sejourInitial"
    />
    <div
      v-else
      class="flex flex-1 items-center justify-center p-6"
    >
      <p class="font-texte text-corps text-ink-2">
        {{ erreur ?? t('sejours.depart.chargement') }}
      </p>
    </div>
  </div>
</template>
