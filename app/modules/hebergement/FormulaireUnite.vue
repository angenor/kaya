<script setup lang="ts">
/**
 * **Créer ou corriger une chambre** — composant **16 · champ de saisie**, et rien d'autre.
 *
 * Écran `G5`, composé. Trois champs, et le troisième est celui qui demandait une décision.
 *
 * # Le choix du type emploie le composant 16 en « choix fermé », PAS le composant 12
 *
 * La règle du contrôle segmenté est explicite : « au-delà de quatre options c'est une liste, pas
 * un segment ». Deloria a **six** types de chambre, salle de réunion comprise, et un segmenté à six
 * options ne tient pas sur 372 px.
 *
 * # Le type ne se change QUE à la création
 *
 * En correction, il est affiché en lecture seule. Changer le type d'une chambre change les
 * formules applicables, **donc les tarifs** : c'est une opération à effet fiscal que le registre
 * des classes ne classe nulle part. Elle se spécifie ; elle ne se glisse pas dans un formulaire de
 * correction.
 *
 * # L'erreur porte trois signaux, jamais la couleur seule
 *
 * Bordure `danger`, message, icône `ph-fill ph-warning-circle` — c'est le composant 16 qui les
 * rend, et l'aide s'efface pendant l'erreur.
 */
import { computed, ref, watch } from 'vue'

import ChampSaisie from '~/core/design-system/ChampSaisie.vue'
import type { CategorieVue, UniteVue } from './donnees'
import { etageDepuisSaisie, validerCode, validerEtage } from './ecrire-parc'

const { t } = useI18n()

const props = defineProps<{
  categories: CategorieVue[]
  /** `null` = création. Renseignée = correction, et le type devient lecture seule. */
  unite: UniteVue | null
}>()

const emit = defineEmits<{
  enregistrer: [valeurs: { categorieId: string, code: string, etage: number | null }]
  annuler: []
}>()

const code = ref('')
const etage = ref('')
const categorieId = ref('')
const erreurCode = ref<string | null>(null)
const erreurEtage = ref<string | null>(null)

const enCorrection = computed(() => props.unite !== null)

const optionsCategorie = computed(() =>
  props.categories.map(c => ({ valeur: c.id, libelleCle: c.nom })),
)

/** Le nom du type, en correction — jamais son identifiant, qui ne dit rien à personne. */
const nomCategorie = computed(
  () => props.categories.find(c => c.id === categorieId.value)?.nom ?? '',
)

watch(
  () => props.unite,
  (unite) => {
    code.value = unite?.code ?? ''
    etage.value = unite?.etage === null || unite?.etage === undefined ? '' : String(unite.etage)
    categorieId.value = unite?.categorie_id ?? props.categories[0]?.id ?? ''
    erreurCode.value = null
    erreurEtage.value = null
  },
  { immediate: true },
)

function soumettre(): void {
  // **Validation au champ, avant tout appel** : elle porte sur ce qui est saisi, et le message
  // s'affiche à côté de l'endroit où l'on corrige.
  erreurCode.value = validerCode(code.value)
  erreurEtage.value = validerEtage(etage.value)
  if (erreurCode.value || erreurEtage.value) {
    return
  }

  emit('enregistrer', {
    categorieId: categorieId.value,
    code: code.value.trim(),
    etage: etageDepuisSaisie(etage.value),
  })
}
</script>

<template>
  <form
    class="flex flex-col gap-3"
    @submit.prevent="soumettre"
  >
    <ChampSaisie
      v-model="code"
      etiquette-cle="hebergement.chambres.champ_code"
      aide-cle="hebergement.chambres.champ_code_aide"
      :erreur-cle="erreurCode"
      requis
    />

    <ChampSaisie
      v-model="etage"
      etiquette-cle="hebergement.chambres.champ_etage"
      aide-cle="hebergement.chambres.champ_etage_aide"
      :erreur-cle="erreurEtage"
    />

    <!-- Création : choix fermé (composant 16, état « choix fermé »). Correction : lecture seule —
         changer le type change les tarifs, et ça se spécifie. -->
    <ChampSaisie
      v-if="!enCorrection"
      v-model="categorieId"
      etiquette-cle="hebergement.chambres.champ_type"
      :options="optionsCategorie"
    />
    <ChampSaisie
      v-else
      v-model="nomCategorie"
      etiquette-cle="hebergement.chambres.champ_type"
      lecture-seule
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
