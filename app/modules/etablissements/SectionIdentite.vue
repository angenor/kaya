<script setup lang="ts">
/**
 * **Section « Identité »** — ETB-01. Motif de section repris de `G2`.
 *
 * `classement` et « numéro de compte contribuable (NCC) » gardent leur **nom officiel** : ce sont
 * les termes que l'administration emploie et que l'exploitant lit sur ses propres papiers
 * (`docs/design/lexique.md`, règle 2). Les reformuler les rendrait méconnaissables.
 */
import { computed } from 'vue'

const { t } = useI18n()

const props = defineProps<{
  etablissement: {
    nom: string
    classement: string
    etoiles: number | null
    commune: string
    fuseau_horaire: string
    devise: string
    ncc: string | null
  }
}>()

/**
 * Le classement, en toutes lettres.
 *
 * Le nombre d'étoiles n'existe que pour la variante `ETOILES` — la base l'impose par une égalité
 * de conditions, et l'affichage suit la même règle plutôt que de la redécouvrir.
 */
const classementLisible = computed(() => {
  if (props.etablissement.classement === 'ETOILES') {
    return t('etablissement.classement.etoiles', { n: props.etablissement.etoiles ?? 0 })
  }
  return t(`etablissement.classement.${props.etablissement.classement}`)
})

const lignes = computed(() => [
  { cle: 'etablissement.champ.classement', valeur: classementLisible.value, mono: false },
  { cle: 'etablissement.champ.commune', valeur: props.etablissement.commune, mono: false },
  { cle: 'etablissement.champ.fuseau', valeur: props.etablissement.fuseau_horaire, mono: true },
  { cle: 'etablissement.champ.devise', valeur: props.etablissement.devise, mono: true },
  {
    cle: 'etablissement.champ.ncc',
    valeur: props.etablissement.ncc ?? t('etablissement.champ.non_renseigne'),
    mono: true,
  },
])
</script>

<template>
  <section class="flex flex-col">
    <div class="flex flex-col gap-1.5 px-3.5 pt-4 pb-1">
      <span class="text-etiquette uppercase text-ink-3">{{ t('etablissement.sur_titre') }}</span>
      <h2 class="font-titre text-chiffre font-semibold text-ink">
        {{ t('etablissement.identite.titre') }}
      </h2>
      <p class="text-corps text-ink-2">
        {{ t('etablissement.identite.intro') }}
      </p>
    </div>

    <dl class="flex flex-col gap-2.25 px-3 pt-3 pb-3.5">
      <div
        v-for="ligne in lignes"
        :key="ligne.cle"
        class="flex w-full items-center gap-3 rounded-l-xs rounded-r-xl border border-line border-l-4 border-l-line-2 bg-surf p-3 shadow-basse"
      >
        <dt class="min-w-0 flex-1 text-corps text-ink-2">
          {{ t(ligne.cle) }}
        </dt>
        <dd
          class="shrink-0 font-titre text-titre-s font-semibold text-ink"
          :class="ligne.mono ? 'font-mono whitespace-nowrap' : ''"
        >
          {{ ligne.valeur }}
        </dd>
      </div>
    </dl>
  </section>
</template>
