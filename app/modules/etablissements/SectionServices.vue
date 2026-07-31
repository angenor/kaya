<script setup lang="ts">
/**
 * **Section « Vos services »** — ETB-02, ETB-02b. Le motif de ligne-bouton vient de `G2`.
 *
 * # Un service inactif est ABSENT
 *
 * Ni entrée grisée, ni mention « disponible dans votre offre », ni marqueur masqué par CSS
 * (principe VII). La liste rendue ne contient que des services actifs, et le HTML produit ne porte
 * **aucun** libellé ni code des autres — c'est ce que vérifie le test de rendu de SC-005.
 *
 * # Le mot « capacité » n'apparaît nulle part
 *
 * Seule la capacité concrète est nommée — « Suivi du stock » — **sous le service qui la
 * consomme**, jamais dans une rubrique à part (`docs/design/lexique.md`).
 */
import { computed } from 'vue'

import {
  capacitesVisibles,
  servicesActivables,
  servicesVisibles,
  type EntreeReferentiel,
  type ServiceActif,
} from './services-visibles'

const { t } = useI18n()

const props = defineProps<{
  services: ServiceActif[]
  referentiel: EntreeReferentiel[]
}>()

const visibles = computed(() => servicesVisibles(props.services, props.referentiel))
const activables = computed(() => servicesActivables(props.services, props.referentiel))

/** Icône par service. Décorative : chaque ligne porte déjà son libellé traduit. */
const ICONES: Record<string, string> = {
  HEBERGEMENT: 'ph-moon-stars',
  RESTAURATION: 'ph-fork-knife',
  BAR: 'ph-martini',
  PRESSING: 'ph-t-shirt',
  SALLE_REUNION: 'ph-users-three',
}
</script>

<template>
  <section class="flex flex-col">
    <div class="flex flex-col gap-1.5 px-3.5 pt-4 pb-1">
      <span class="text-etiquette uppercase text-ink-3">{{ t('etablissement.sur_titre') }}</span>
      <h2 class="font-titre text-chiffre font-semibold text-ink">
        {{ t('etablissement.services.titre') }}
      </h2>
      <p class="text-corps text-ink-2">
        {{ t('etablissement.services.intro') }}
      </p>
    </div>

    <ul class="flex flex-col gap-2.25 px-3 pt-3 pb-3.5">
      <li
        v-for="service in visibles"
        :key="service.id"
      >
        <div
          class="flex w-full items-center gap-3 rounded-l-xs rounded-r-xl border border-line border-l-4 border-l-line-2 bg-surf p-3 shadow-basse"
        >
          <span
            class="inline-flex size-10 shrink-0 items-center justify-center rounded-xl bg-tile"
          >
            <i
              class="ph text-titre-m text-ocre"
              :class="ICONES[service.module_code] ?? 'ph-buildings'"
              aria-hidden="true"
            />
          </span>
          <span class="flex min-w-0 flex-1 flex-col items-start gap-0.75 text-left">
            <span class="font-titre text-titre-s font-semibold text-ink">
              {{ t(service.libelle_cle) }}
            </span>
            <!-- Ce que le service suit, nommé concrètement — « Suivi du stock ». Aucune ligne
                 quand il n'y a rien : une liste vide est la forme normale, et elle ne se signale
                 pas.

                 Le mot d'architecture qui désigne ces lignes n'apparaît PAS ici, pas même en
                 commentaire : un commentaire de gabarit part dans le HTML livré, et le test de
                 rendu le voit. C'est délibéré — il vaut mieux qu'il attrape un commentaire de
                 trop que de laisser passer un libellé. -->
            <span
              v-for="capacite in capacitesVisibles(service)"
              :key="capacite.id"
              class="text-mini text-ink-3"
            >
              {{ t(capacite.libelle_cle) }}
            </span>
          </span>
        </div>
      </li>
    </ul>

    <!-- Bouton principal en pied, motif de `G2`. Seuls les services IMPLÉMENTÉS et non encore
         activés sont proposables : proposer une valeur non implémentée garantirait un refus 422
         que l'exploitant n'a aucune raison de rencontrer (FR-036). -->
    <div
      v-if="activables.length > 0"
      class="flex shrink-0 flex-col gap-2 border-t border-line bg-surf px-3 pt-2.75 pb-3.5"
    >
      <button
        type="button"
        class="inline-flex h-13 w-full cursor-pointer items-center justify-center gap-2.5 rounded-xl bg-prim font-titre text-titre-s font-semibold text-prim-ink shadow-bouton-grand transition-[transform,box-shadow] duration-90 ease-entree active:translate-y-0.75 active:shadow-none"
      >
        <i
          class="ph ph-plus text-titre-m"
          aria-hidden="true"
        />
        {{ t('etablissement.services.action_activer') }}
      </button>
    </div>
  </section>
</template>
