<script setup lang="ts">
/**
 * **Notes internes** — coquille de page. Le premier passager réel de la file hors-ligne.
 *
 * # ÉCRAN COMPOSÉ, cas (c) — inscrit à `docs/design/derivation.md`
 *
 * Sa référence n'est pas un fichier de maquette mais la **bibliothèque des seize composants** :
 * ligne de liste (08), champ de saisie (16), bouton principal (01), état vide (11), squelette
 * (13). Les quatre conditions du cas (c) sont vérifiées dans sa ligne de la matrice. Un écran hors
 * de la matrice ne se code pas — celui-ci y est.
 *
 * # Chargement paresseux, comme partout
 *
 * Un serveur de salle ne télécharge pas le code d'un écran qu'il n'ouvre pas (principe VII).
 *
 * # Racine UNIQUE, et c'est structurel
 *
 * Un `v-if`/`v-else` de premier niveau compile en fragment ; un fragment dont la branche active
 * est un `defineAsyncComponent` non résolu a un `el` nul, et Vue lève
 * `Cannot read properties of null (reading 'parentNode')` à la navigation suivante. C'est ce qui
 * rendait `G3` et `G4` inatteignables au cycle 003. Le `<div>` ci-dessous n'est pas décoratif.
 */
import { computed, defineAsyncComponent, onMounted, ref } from 'vue'

import { contexteAppel, sessionCourante, type ContexteAppel } from '~/core/auth'
import type { PageNotes } from '~/modules/etablissements/notes'

const EcranNotes = defineAsyncComponent(
  () => import('~/modules/etablissements/EcranNotes.vue'),
)

const { t } = useI18n()
const config = useRuntimeConfig()

const page = ref<PageNotes | null>(null)
const erreur = ref<string | null>(null)

const contexte = computed<ContexteAppel | null>(() => contexteAppel(config.public.apiBaseUrl))
const session = computed(() => sessionCourante())
const etablissementId = computed(
  () => session.value?.etablissementId ?? String(config.public.etablissementId),
)
const tenantId = computed(() => session.value?.tenantId ?? '')

onMounted(async () => {
  if (!contexte.value) {
    erreur.value = t('connexion.requise')
    return
  }

  const { chargerNotes } = await import('~/modules/etablissements/notes')
  try {
    page.value = await chargerNotes(contexte.value, etablissementId.value)
  }
  catch {
    // **La lecture échoue, l'écriture reste possible.** C'est le point de la classe A : ne pas
    // pouvoir afficher les notes déjà envoyées n'empêche pas d'en écrire une — elle partira quand
    // le réseau reviendra. Une page qui refuserait tout parce que la liste ne charge pas
    // transformerait une coupure en blocage.
    page.value = { elements: [], decalage: 0, limite: 0, total: 0 }
    erreur.value = t('notes.chargement_impossible')
  }
})
</script>

<template>
  <div class="flex flex-1 flex-col">
    <EcranNotes
      v-if="page && contexte"
      :page="page"
      :contexte="contexte"
      :tenant-id="tenantId"
      :etablissement-id="etablissementId"
    />
    <!-- COMPOSANT 13 · SQUELETTE DE CHARGEMENT — même hauteur de ligne que le contenu réel, pour
         que rien ne saute quand il arrive. -->
    <div
      v-else
      class="flex flex-1 flex-col gap-3 p-6"
    >
      <p
        v-if="erreur"
        class="text-corps text-ink-2"
      >
        {{ erreur }}
      </p>
      <template v-else>
        <span
          v-for="rang in 3"
          :key="rang"
          class="h-3 w-3/5 rounded-sm bg-tile"
          aria-hidden="true"
        />
        <p class="sr-only">
          {{ t('notes.chargement') }}
        </p>
      </template>
    </div>
  </div>
</template>
