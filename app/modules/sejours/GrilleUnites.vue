<script setup lang="ts">
/**
 * **Le second des deux gestes** — « Les chambres. Un tap pour changer. »
 *
 * Écran `R4`, **maquetté** : `docs/design/html/R4-passage.html`. Valeurs lues, HTML **jamais
 * copié** (porte P-19).
 *
 * # ★ L'attribution est UN TAP, et c'est tout le sujet
 *
 * Pas de sélection puis de confirmation, pas de menu, pas de champ. La maquette l'écrit :
 * « Un tap pour changer. Rien d'autre à faire. » Ajouter une confirmation ferait **trois** gestes
 * là où le budget de SC-001 en autorise **deux**, et le cadrage §5.6 dit ce qui arrive à un écran
 * de comptoir trop lent : il est contourné.
 *
 * # L'état est DÉRIVÉ, jamais posé à la main
 *
 * `libre` · `occupee` avec son heure de fin · `remise_en_etat` avec son heure de disponibilité.
 * Les trois viennent du serveur, qui les dérive des occupations (principe IV). Une colonne
 * `statut` sur la chambre se désynchroniserait à la première libération manquée, et deux personnes
 * verraient la même chambre libre et occupée — ce que le cadrage désigne nommément comme la cause
 * des doubles attributions.
 *
 * **Le sous-statut de ménage est affiché en lecture seule** : le modifier est HEB-06, hors
 * périmètre. Aucun bouton ne le propose, et il n'est pas grisé — il est **absent**.
 *
 * # Ce que la liste ne garantit pas, et le dit
 *
 * Entre l'affichage et le tap, une autre réception peut prendre la chambre. **C'est normal** : la
 * garantie est la contrainte d'exclusion, jamais cette liste (FR-013). L'écran affiche donc la
 * fraîcheur de sa lecture, et le refus éventuel est un fait d'exploitation lisible — « Cette
 * chambre est déjà prise sur cette période » — pas une erreur technique.
 */
import { computed } from 'vue'

import type { EtatUniteVue } from './donnees'

const { t, locale } = useI18n()

const props = defineProps<{
  unites: EtatUniteVue[]
  /** La chambre retenue — `null` tant qu'aucun geste n'a eu lieu. */
  choisie: string | null
  /** Vrai hors ligne : l'attribution est refusée **immédiatement**, jamais grisée. */
  horsLigne: boolean
  /** Attribution en cours — squelette sur la seule chambre concernée (composant 13). */
  enCours: string | null
}>()

const emit = defineEmits<{ attribuer: [unite: EtatUniteVue] }>()

/**
 * L'heure murale, **avec l'espace ORDINAIRE** — `16 h 10`.
 *
 * ⚠️ Jamais l'espace fine insécable, réservée aux montants (`docs/design/tokens.md` §2).
 */
const heureDe = computed(() => (instant: string | null | undefined): string => {
  if (!instant) return ''
  return new Intl.DateTimeFormat(locale.value, {
    hour: '2-digit',
    minute: '2-digit',
    hour12: false,
  })
    .format(new Date(instant))
    .replace(':', ' h ')
})

/** Le libellé d'état d'une chambre — **une clé i18n par cas**, jamais une chaîne composée. */
function libelleEtat(unite: EtatUniteVue): string {
  if (unite.etat === 'occupee') {
    return t('sejours.grille.occupee', { heure: heureDe.value(unite.fin_prevue) })
  }
  if (unite.etat === 'remise_en_etat') {
    return t('sejours.grille.remise_en_etat', { heure: heureDe.value(unite.disponible_a) })
  }
  // Le sous-statut de ménage n'apparaît **que** sur une chambre libre : sur une chambre occupée,
  // ce qui compte est l'heure de sortie, pas la propreté d'hier.
  return unite.statut_menage === 'a_nettoyer'
    ? t('sejours.grille.a_nettoyer')
    : t('sejours.grille.libre')
}

/** Seule une chambre **libre** est attribuable — et le refus est visible, pas silencieux. */
function attribuable(unite: EtatUniteVue): boolean {
  return unite.etat === 'libre' && unite.statut_menage !== 'a_nettoyer'
}
</script>

<template>
  <section class="flex flex-col gap-3">
    <header class="flex flex-col gap-1">
      <h2 class="font-titre text-titre-m font-semibold text-ink">
        {{ t('sejours.grille.les_chambres') }}
      </h2>
      <p class="text-mini text-ink-3">
        {{ t('sejours.grille.un_tap') }}
      </p>
    </header>

    <div class="grid grid-cols-3 gap-2.5 sm:grid-cols-4 lg:grid-cols-6">
      <button
        v-for="unite in props.unites"
        :key="unite.unite_id"
        type="button"
        :data-unite="unite.unite_id"
        :data-etat="unite.etat"
        :disabled="!attribuable(unite) || props.enCours !== null"
        :aria-pressed="props.choisie === unite.unite_id"
        class="flex flex-col items-start gap-0.5 rounded-xl border px-3 py-3 text-left transition-colors duration-90"
        :class="[
          props.choisie === unite.unite_id
            ? 'border-ocre bg-ocre-soft'
            : 'border-line bg-surf',
          attribuable(unite) && props.enCours === null
            ? 'cursor-pointer hover:bg-tile'
            : 'cursor-not-allowed opacity-60',
        ]"
        @click="emit('attribuer', unite)"
      >
        <!-- Le code de chambre est en mono, largeur fixe : la grille se parcourt à l'œil. -->
        <span class="font-mono text-corps font-semibold text-ink">
          {{ unite.code }}
        </span>
        <span
          v-if="props.enCours === unite.unite_id"
          class="h-4 w-16 rounded bg-tile animate-souffle"
          :aria-label="t('sejours.grille.en_cours')"
        />
        <span
          v-else
          class="text-mini"
          :class="unite.etat === 'occupee' ? 'text-occupe-fort' : 'text-ink-3'"
        >
          {{ libelleEtat(unite) }}
        </span>
      </button>
    </div>

    <!--
      La légende de la maquette. Elle n'est pas décorative : sans elle, un état de couleur seul
      serait invisible à qui distingue mal les teintes, et l'écran cesserait d'être lisible pour
      une partie du personnel.
    -->
    <ul class="flex flex-wrap gap-4 text-mini text-ink-3">
      <li>{{ t('sejours.grille.libre') }}</li>
      <li>{{ t('sejours.grille.legende_occupee') }}</li>
      <li>{{ t('sejours.grille.a_nettoyer') }}</li>
    </ul>

    <!--
      ★ Hors ligne : le refus est IMMÉDIAT et EXPLICITE, annoncé AVANT le geste (principe VI).
      Jamais un grisé silencieux, jamais une file « au cas où » : l'ouverture d'un séjour est de
      classe B, et le registre l'interdit hors ligne.
    -->
    <p
      v-if="props.horsLigne"
      class="rounded-xl border border-dashed border-line-2 px-3.5 py-2.5 text-mini text-ink-2"
      role="status"
    >
      {{ t('sejours.grille.hors_ligne_attribution') }}
    </p>
  </section>
</template>
