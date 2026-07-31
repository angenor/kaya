<script setup lang="ts">
/**
 * **`G1` — les quatre sections.** Hérite de `G2` (matrice de dérivation).
 *
 * Ce qui est repris de `G2` : le sélecteur d'établissement en tête, les sections introduites par
 * une sur-étiquette `text-etiquette uppercase text-ink-3` puis un `h2` `font-titre text-chiffre`,
 * les lignes-boutons `rounded-l-xs rounded-r-xl border-l-4`, et le bouton principal
 * `h-13 rounded-xl bg-prim` en pied.
 *
 * Ce que `G2` ne contient pas et qui est ajouté ici, l'export étant une cible et non une source :
 * **i18n** (aucune chaîne en dur), **mode sombre** par les tokens dont les valeurs basculent sous
 * `.dark`, et **chargement paresseux** par la coquille de page.
 */
import { computed } from 'vue'

import SectionIdentite from './SectionIdentite.vue'
import SectionIdentiteVisuelle from './SectionIdentiteVisuelle.vue'
import SectionPointsDeVente from './SectionPointsDeVente.vue'
import SectionServices from './SectionServices.vue'
import type { EntreeReferentiel, ServiceActif } from './services-visibles'
import type { PointDeVenteVue } from './points-de-vente'

const { t } = useI18n()

const props = defineProps<{
  etablissement: {
    id: string
    nom: string
    classement: string
    etoiles: number | null
    commune: string
    fuseau_horaire: string
    devise: string
    ncc: string | null
  }
  services: ServiceActif[]
  referentielModules: EntreeReferentiel[]
  pointsDeVente: PointDeVenteVue[]
  configuration: { cle: string; valeur: unknown; origine: string }[]
}>()

/**
 * Résumé sous le nom, à la manière du sélecteur de `G2` (« 8 chambres · Yopougon »).
 *
 * **Le compte est passé en TROISIÈME argument, pas seulement dans les valeurs nommées.** `vue-i18n`
 * choisit la branche d'une forme « singulier | pluriel » sur ce paramètre-là ; le fournir seulement
 * comme valeur nommée affiche « 5 service » — le texte est bien interpolé, mais la branche est
 * toujours la première. Le défaut ne se voit qu'à partir de deux éléments.
 */
const resume = computed(() =>
  t(
    'etablissement.resume',
    { n: props.services.length, commune: props.etablissement.commune },
    props.services.length,
  ),
)
</script>

<template>
  <main class="min-h-screen bg-bg font-texte text-ink">
    <!-- Sélecteur d'établissement, repris de G2. Il n'OUVRE rien à ce cycle : le sélecteur de
         contexte est ETB-06, hors périmètre. Le chevron de G2 est donc absent — un chevron qui
         n'ouvre rien est une promesse non tenue. -->
    <header class="flex h-14.5 shrink-0 items-center gap-2 border-b border-line bg-surf px-3">
      <div
        class="flex h-11.5 min-w-0 flex-1 items-center gap-2.5 rounded-xl border border-line bg-tile px-3"
      >
        <i
          class="ph ph-buildings text-titre-m text-ocre"
          aria-hidden="true"
        />
        <span class="flex min-w-0 flex-1 flex-col items-start gap-0.5">
          <span class="w-full truncate text-left font-titre text-lead font-semibold text-ink">
            {{ etablissement.nom }}
          </span>
          <span class="w-full truncate text-left text-mini text-ink-3">{{ resume }}</span>
        </span>
      </div>
    </header>

    <div class="flex flex-col">
      <SectionIdentite :etablissement="etablissement" />
      <SectionServices
        :services="services"
        :referentiel="referentielModules"
      />
      <SectionPointsDeVente
        :points-de-vente="pointsDeVente"
        :configuration="configuration"
      />
      <SectionIdentiteVisuelle :nom-etablissement="etablissement.nom" />
    </div>
  </main>
</template>
