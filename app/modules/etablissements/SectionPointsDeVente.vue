<script setup lang="ts">
/**
 * **Section « Points de vente »** — ETB-03, et les valeurs de configuration — ETB-04.
 *
 * # Un point de vente sans table s'affiche « Comptoir »
 *
 * Pas « point de vente sans tables », qui décrit un manque là où il s'agit d'une forme normale
 * (`docs/design/lexique.md`).
 *
 * # Les valeurs de configuration disent leur origine
 *
 * « Vaut pour tous vos établissements » ou « Modifié ici ». Les mots « héritage », « surcharge »
 * et « portée » n'atteignent jamais l'interface.
 *
 * **La section entière est absente quand il n'y a aucun point de vente** — c'est le cas de la
 * résidence meublée, et il n'y a rien à lui dire à ce sujet.
 */
import { computed } from 'vue'

import { cleOrigine } from './services-visibles'
import { cleQualificatif, type PointDeVenteVue } from './points-de-vente'

const { t } = useI18n()

const props = defineProps<{
  pointsDeVente: PointDeVenteVue[]
  configuration: { cle: string; valeur: unknown; origine: string }[]
}>()

const aDesPointsDeVente = computed(() => props.pointsDeVente.length > 0)
</script>

<template>
  <section
    v-if="aDesPointsDeVente"
    class="flex flex-col"
  >
    <div class="flex flex-col gap-1.5 px-3.5 pt-4 pb-1">
      <span class="text-etiquette uppercase text-ink-3">{{ t('etablissement.sur_titre') }}</span>
      <h2 class="font-titre text-chiffre font-semibold text-ink">
        {{ t('etablissement.points_de_vente.titre') }}
      </h2>
      <p class="text-corps text-ink-2">
        {{ t('etablissement.points_de_vente.intro') }}
      </p>
    </div>

    <ul class="flex flex-col gap-2.25 px-3 pt-3 pb-3.5">
      <li
        v-for="pointDeVente in pointsDeVente"
        :key="pointDeVente.id"
      >
        <div
          class="flex w-full items-center gap-3 rounded-l-xs rounded-r-xl border border-line border-l-4 border-l-line-2 bg-surf p-3 shadow-basse"
        >
          <span
            class="inline-flex size-10 shrink-0 items-center justify-center rounded-xl bg-tile"
          >
            <i
              class="ph ph-storefront text-titre-m text-ocre"
              aria-hidden="true"
            />
          </span>
          <span class="flex min-w-0 flex-1 flex-col items-start gap-0.75 text-left">
            <span class="font-titre text-titre-s font-semibold text-ink">
              {{ pointDeVente.nom }}
            </span>
            <span class="text-corps text-ink-2">
              {{ t(cleQualificatif(pointDeVente), { n: pointDeVente.tables.length }) }}
            </span>
          </span>
        </div>
      </li>
    </ul>

    <dl
      v-if="configuration.length > 0"
      class="flex flex-col gap-2.25 px-3 pb-3.5"
    >
      <div
        v-for="valeur in configuration"
        :key="valeur.cle"
        class="flex w-full flex-col items-start gap-0.75 rounded-l-xs rounded-r-xl border border-line border-l-4 border-l-line-2 bg-surf p-3 text-left shadow-basse"
      >
        <!-- Le `<span>` qui enveloppait ces deux nœuds a été retiré : `<dt>` et `<dd>` ne peuvent
             pas être enfants d'un `<span>`, et le compilateur de Vue le signalait à chaque
             construction — « can cause hydration errors ». La classe de mise en page passe donc
             sur le `<div>` du `v-for`, qui est un enfant licite de `<dl>`.

             Trouvé en lançant la porte P-22, qui démarre le serveur : l'avertissement sortait dans
             sa sortie. Aucun des 440 tests front ne pouvait le voir — ils compilent les composants
             sans lire les diagnostics du compilateur. -->
        <dt class="min-w-0 flex-1 font-titre text-titre-s font-semibold text-ink">
          {{ t(`configuration.${valeur.cle}.libelle`) }}
        </dt>
        <!-- L'origine, en mots d'utilisateur. C'est ce que `origine` obligatoire au résolveur
             rend possible : sans elle, cette ligne n'existerait pas. -->
        <dd class="text-mini text-ink-3">{{ t(cleOrigine(valeur.origine)) }}</dd>
      </div>
    </dl>
  </section>
</template>
