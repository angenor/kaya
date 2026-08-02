<script setup lang="ts">
/**
 * **Créer un type de chambre** — composant **16 · champ de saisie**, deux champs.
 *
 * Écran `G5`, composé. Le terme utilisateur est « type de chambre » : ni « catégorie », ni
 * « catégorie d'unité », qui collent deux mots de table dont l'un est déjà écarté du lexique.
 *
 * # Aucun champ de temps de remise en état ici
 *
 * Le battement varie par type **ET** par formule : il se règle là où l'on règle les formules, pas
 * là où l'on nomme un type de chambre. Un champ unique ici obligerait à choisir laquelle des
 * quatre familles il concerne, et le premier qui répondrait « toutes » aurait tort.
 */
import { ref } from 'vue'

import ChampSaisie from '~/core/design-system/ChampSaisie.vue'
import { validerCapacite } from './ecrire-parc'

const { t } = useI18n()

const emit = defineEmits<{
  enregistrer: [valeurs: { nom: string, capaciteAccueil: number }]
  annuler: []
}>()

const nom = ref('')
const capacite = ref('2')
const erreurNom = ref<string | null>(null)
const erreurCapacite = ref<string | null>(null)

function soumettre(): void {
  erreurNom.value = nom.value.trim() === '' ? 'champ.erreur.obligatoire' : null
  erreurCapacite.value = validerCapacite(capacite.value)
  if (erreurNom.value || erreurCapacite.value) {
    return
  }

  emit('enregistrer', {
    nom: nom.value.trim(),
    capaciteAccueil: Number(capacite.value.trim()),
  })
}
</script>

<template>
  <form
    class="flex flex-col gap-3"
    @submit.prevent="soumettre"
  >
    <ChampSaisie
      v-model="nom"
      etiquette-cle="hebergement.chambres.champ_nom_type"
      :erreur-cle="erreurNom"
      requis
    />
    <ChampSaisie
      v-model="capacite"
      etiquette-cle="hebergement.chambres.champ_capacite"
      :erreur-cle="erreurCapacite"
      requis
    />

    <div class="flex gap-2">
      <button
        type="submit"
        class="flex-1 h-12 rounded-xl bg-prim text-prim-ink font-titre text-action font-semibold cursor-pointer shadow-bouton transition-[transform,box-shadow] duration-90 ease-entree active:translate-y-0.5 active:shadow-bouton-appui"
      >
        {{ t('hebergement.chambres.enregistrer') }}
      </button>
      <button
        type="button"
        class="h-12 px-4 rounded-xl border border-line bg-tile text-ink font-titre text-action font-semibold cursor-pointer"
        @click="emit('annuler')"
      >
        {{ t('hebergement.chambres.annuler') }}
      </button>
    </div>
  </form>
</template>
