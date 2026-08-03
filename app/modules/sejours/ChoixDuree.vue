<script setup lang="ts">
/**
 * **Le premier des deux gestes** — « Combien de temps ? ».
 *
 * Écran `R4`, **maquetté** : `docs/design/html/R4-passage.html`. Les valeurs sont lues de la
 * maquette ; **son HTML n'est jamais copié** (porte P-19) — il est autonome, non sémantique, sans
 * i18n, sans RBAC et sans chargement paresseux, quatre choses que cet écran doit avoir.
 *
 * # ★ Le prix est SUR le bouton, et l'heure de fin aussi
 *
 * La maquette met trois valeurs par bouton — « 2 h · 2 800 F · jusqu'à 17 h 30 » — et c'est
 * l'intention dessinée de l'écran : **Yao n'a rien à calculer de tête** et redit l'heure au client
 * sans réfléchir. Un bouton qui ne porterait que la durée obligerait à une seconde lecture pour
 * connaître le prix, donc à un aller-retour du regard, donc à du temps sur un écran dont le
 * cadrage §5.6 fait une condition d'existence du produit.
 *
 * Les tailles de la durée et de l'heure de fin sont **l'intention dessinée** : la durée domine,
 * le prix suit, l'heure de fin est la ligne de rappel. Un assemblage depuis la bibliothèque ne les
 * retrouverait pas — c'est précisément pourquoi un écran de **zone de vitesse ne se compose
 * jamais** (`docs/Kaya_Design.md` §1).
 *
 * # Ce qui n'est PAS ici, et n'est pas grisé
 *
 * **« Scanner la pièce »** figure sur la maquette et relève de **SEJ-06** (OCR, P1, tranche T4).
 * Le bouton est **absent**, jamais grisé (principe VII) : un bouton grisé promet une fonction que
 * le produit n'a pas, et l'exploitant attend une mise à jour qui ne vient pas. Seul « Saisir ·
 * téléphone » est livré.
 *
 * # Aucun montant écrit à la main
 *
 * `formaterMontant(montantMineur, devise)` — le nombre de décimales et le symbole viennent de la
 * **devise**, jamais d'une constante (principe V). Les **heures gardent l'espace ORDINAIRE**
 * (`17 h 30`) et ne passent pas par le formateur de montant, dont l'espace est **fine
 * insécable**.
 */
import { computed } from 'vue'

import { formaterMontant } from '~/core/format/montant'
import type { PalierAffichable } from './donnees'

const { t, locale } = useI18n()

const props = defineProps<{
  paliers: PalierAffichable[]
  /** Le palier retenu — `null` tant qu'aucun geste n'a eu lieu. */
  choisi: string | null
  /** Vrai hors ligne : les prix restent **lisibles**, l'attribution est refusée plus loin. */
  horsLigne: boolean
}>()

const emit = defineEmits<{ choisir: [palier: PalierAffichable] }>()

/**
 * Une clé stable par palier — **durée ET formule**.
 *
 * La seule durée collisionnerait le jour où deux catégories offriraient « 2 h » à des prix
 * différents, et Vue réutiliserait le mauvais nœud.
 */
function cle(palier: PalierAffichable): string {
  return `${palier.formuleId}:${palier.dureeMinutes}`
}

/**
 * L'heure de fin, **avec l'espace ORDINAIRE** — `17 h 30`.
 *
 * ⚠️ Elle ne passe **pas** par `formaterMontant`, dont le séparateur est l'espace **fine
 * insécable** U+202F. `docs/design/tokens.md` §2 réserve la fine aux montants ; une heure écrite
 * avec elle se lirait différemment de partout ailleurs dans le produit.
 *
 * `Intl.DateTimeFormat` est employé ici — contrairement à `Intl.NumberFormat`, écarté pour les
 * montants — parce que ce qu'on formate est une **heure murale**, dont la représentation est
 * exactement ce que l'ICU sait faire, et qu'aucun séparateur de groupe n'entre en jeu.
 */
const heureDe = computed(() => (instant: Date): string => {
  const heures = new Intl.DateTimeFormat(locale.value, {
    hour: '2-digit',
    minute: '2-digit',
    hour12: false,
  }).format(instant)
  // `14:30` → `14 h 30`, la forme française de `tokens.md` §2.
  return heures.replace(':', ' h ')
})

const duree = computed(() => (minutes: number): string => {
  const heures = Math.floor(minutes / 60)
  const reste = minutes % 60
  return reste === 0
    ? t('sejours.passage.duree_heures', { n: heures })
    : t('sejours.passage.duree_heures_minutes', { h: heures, m: reste })
})
</script>

<template>
  <section class="flex flex-col gap-3">
    <header class="flex flex-col gap-1">
      <h2 class="font-titre text-titre-m font-semibold text-ink">
        {{ t('sejours.passage.combien_de_temps') }}
      </h2>
      <p class="text-mini text-ink-3">
        {{ t('sejours.passage.prix_sur_le_bouton') }}
      </p>
    </header>

    <!--
      La grille reste à quatre colonnes au plus : au-delà, les boutons deviennent trop étroits pour
      une cible tactile confortable, et la zone de vitesse perd ce qui la définit.
    -->
    <div class="grid grid-cols-2 gap-3 sm:grid-cols-4">
      <button
        v-for="palier in props.paliers"
        :key="cle(palier)"
        type="button"
        :aria-pressed="props.choisi === cle(palier)"
        :data-palier="cle(palier)"
        class="flex flex-col items-start gap-1 rounded-2xl border px-4 py-4 text-left transition-colors duration-90"
        :class="props.choisi === cle(palier)
          ? 'border-ocre bg-ocre-soft text-ocre-fort'
          : 'border-line bg-surf hover:bg-tile'"
        @click="emit('choisir', palier)"
      >
        <!-- La durée DOMINE — intention dessinée de la maquette. -->
        <span class="font-titre text-titre-l font-semibold text-ink">
          {{ duree(palier.dureeMinutes) }}
        </span>
        <!-- Le prix suit, en mono tabulaire : les colonnes s'alignent à l'œil. -->
        <span class="font-mono text-corps font-medium text-ink">
          {{ formaterMontant(palier.prixMineur, palier.devise) }}
        </span>
        <!-- L'heure de fin est la ligne de rappel — espace ORDINAIRE. -->
        <span class="text-mini text-ink-3">
          {{ t('sejours.passage.jusqu_a', { heure: heureDe(palier.finPrevue) }) }}
        </span>
      </button>
    </div>

    <!--
      Hors ligne, les durées et les prix restent LISIBLES avec leur fraîcheur : c'est ce que montre
      `R4-passage-hors-ligne.html`. Le refus porte sur l'attribution, plus loin, et il est
      IMMÉDIAT et EXPLICITE — jamais un grisé silencieux (principe VI).
    -->
    <p
      v-if="props.horsLigne"
      class="rounded-xl border border-dashed border-line-2 px-3.5 py-2.5 text-mini text-ink-3"
    >
      {{ t('sejours.passage.hors_ligne_tarifs') }}
    </p>

    <!-- Bloc « après la clé » — voir la note de tête : un élément d'un autre cycle est absent. -->
    <div class="flex flex-col gap-1 rounded-xl bg-tile px-3.5 py-3">
      <p class="font-titre text-corps font-semibold text-ink">
        {{ t('sejours.passage.piece_apres_la_cle') }}
      </p>
      <p class="text-mini text-ink-3">
        {{ t('sejours.passage.piece_a_completer') }}
      </p>
    </div>
  </section>
</template>
