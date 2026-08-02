<script setup lang="ts">
/**
 * **Les chambres, groupées par type** — composant **08 · ligne de liste**.
 *
 * Écran `G5`, **composé** (`docs/design/derivation.md`, tableau « Les écrans composés »). Aucune
 * maquette : l'assemblage vient de la bibliothèque, motif par motif.
 *
 * | Besoin | Composant |
 * |---|---|
 * | Liste des chambres | **08** — son rôle nomme littéralement « chambres » |
 * | Aucune chambre dans un type | **11** — état vide illustré |
 * | Chargement | **13** — squelette, même hauteur de ligne que le contenu réel |
 *
 * # Ce que cette liste n'affiche PAS, et ce n'est pas un oubli
 *
 * - **Aucun statut d'occupation.** Il est *dérivé* des occupations, et son écran est `R2` (tranche
 *   SEJ). L'afficher ici demanderait de le calculer sur un écran de réglage, où il serait périmé
 *   à la seconde suivante.
 * - **Aucune action sur le sous-statut de ménage.** C'est HEB-06, hors périmètre — la colonne
 *   existe, l'endpoint non (principe X).
 *
 * # Le code de chambre est en mono, largeur fixe
 *
 * `docs/design/tokens.md` : Chivo Mono pour « un montant, une quantité, **un numéro de chambre**,
 * une heure ». La largeur fixe fait que « A1 » et « SALLE-1 » n'ont pas la même longueur mais
 * commencent au même endroit — la liste se parcourt à l'œil, en descendant la colonne.
 */
import { computed } from 'vue'

import type { CategorieVue, UniteVue } from './donnees'

const { t } = useI18n()

const props = defineProps<{
  categories: CategorieVue[]
  unites: UniteVue[]
  /** Sans la permission de gérer, les actions de bord n'existent pas dans le HTML rendu. */
  peutGerer: boolean
  /** Chargement en cours sur cette chambre — squelette de la ligne concernée (composant 13). */
  enCours: string | null
}>()

defineEmits<{ corriger: [unite: UniteVue] }>()

/** Les types de chambre, chacun avec ses chambres — **triés comme le serveur les rend**. */
const groupes = computed(() =>
  props.categories.map(categorie => ({
    categorie,
    unites: props.unites.filter(u => u.categorie_id === categorie.id),
  })),
)

/** L'étage, en clair. Vide = rez-de-chaussée, ce qui est un fait, pas une absence. */
function libelleEtage(unite: UniteVue): string {
  return unite.etage === null || unite.etage === undefined
    ? t('hebergement.chambres.sans_etage')
    : t('hebergement.chambres.etage', { numero: unite.etage })
}
</script>

<template>
  <div class="flex flex-col gap-4">
    <section
      v-for="groupe in groupes"
      :key="groupe.categorie.id"
      class="flex flex-col gap-1.5"
    >
      <h3 class="px-3.5 text-etiquette uppercase text-ink-3">
        {{ groupe.categorie.nom }}
      </h3>

      <!-- État vide illustré — composant 11. Un type de chambre sans chambre n'est pas une
           erreur : c'est un type qu'on vient de créer. -->
      <div
        v-if="groupe.unites.length === 0"
        class="mx-3 flex flex-col items-center gap-1.5 rounded-xl border border-dashed border-line-2 px-6 py-6 text-center"
      >
        <i
          class="ph ph-bed text-titre-l text-ocre"
          aria-hidden="true"
        />
        <p class="font-titre text-corps font-semibold text-ink">
          {{ t('hebergement.chambres.vide_titre') }}
        </p>
        <p class="text-mini text-ink-3">
          {{ t('hebergement.chambres.vide_aide') }}
        </p>
      </div>

      <ul
        v-else
        class="flex flex-col"
      >
        <li
          v-for="unite in groupe.unites"
          :key="unite.id"
        >
          <!-- Squelette de la LIGNE CONCERNÉE, même hauteur que le contenu réel : rien ne saute
               quand la ligne revient. -->
          <div
            v-if="enCours === unite.id"
            class="mx-3 my-0.5 h-13 rounded-xl bg-tile animate-souffle"
            :aria-label="t('hebergement.chambres.enregistrement')"
          />
          <!-- Ligne entière cliquable — composant 08. La cible tactile est la ligne, pas une
               icône de 16 px au bout. -->
          <button
            v-else
            type="button"
            class="w-full h-13 px-3.5 flex items-center gap-3 border-b border-line bg-surf text-left cursor-pointer transition-colors duration-90 hover:bg-tile"
            @click="$emit('corriger', unite)"
          >
            <span class="w-20 shrink-0 font-mono text-corps font-medium text-ink">
              {{ unite.code }}
            </span>
            <span class="flex-1 min-w-0 truncate text-mini text-ink-3">
              {{ libelleEtage(unite) }}
            </span>
            <!-- Action de bord — ABSENTE sans la permission, jamais grisée. -->
            <span
              v-if="peutGerer"
              class="shrink-0 text-mini font-medium text-prim"
            >
              {{ t('hebergement.chambres.modifier_unite') }}
            </span>
          </button>
        </li>
      </ul>
    </section>
  </div>
</template>
