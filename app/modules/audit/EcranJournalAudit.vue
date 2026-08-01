<script setup lang="ts">
/**
 * **`G4` — Registre des actions.** Ce que le propriétaire achète.
 *
 * Référence visuelle — cas (b), **écran dérivé** : `docs/design/derivation.md` ligne
 * « `G4` Journal d'audit **hérite de `R5` + `F2`** — Liste filtrable, registre sobre ». Maquettes
 * lues : `docs/design/html/F2-registre-grave.html` et `S2-registre-grave.html`. **Le HTML de
 * maquette n'est jamais copié** (porte P-19).
 *
 * Registre **sobre** : pas de couleur d'accentuation par ligne, pas d'icône par famille d'action.
 * Ce que `F2` établit — et que le premier réflexe défait — c'est qu'un registre grave se lit à
 * plat. Une remise en rouge et une ouverture de tiroir en orange transformeraient une liste de
 * faits en tableau d'accusations.
 *
 * # L'horodatage affiché est celui d'AUTORITÉ, jamais celui du terminal
 *
 * `cree_le` est posé par la base. `horodatage_client` est rendu par l'API et **n'est pas affiché
 * comme la date de l'action** : un téléphone en avance de deux heures ferait mentir le registre
 * qui sert à prouver ce qui s'est passé (principe IV).
 *
 * # Le nom technique de l'action n'atteint jamais l'écran
 *
 * `changement_role` devient « Ce que quelqu'un peut faire a changé ». Le lexique traduit ; la
 * taxonomie nomme l'intention. Un code affiché en brut serait un identifiant technique sous les
 * yeux de l'exploitant.
 *
 * # Aucune écriture, aucune classe C
 *
 * Le contrat n'expose aucun point d'entrée d'écriture. Cet écran n'a donc ni garde hors-ligne, ni
 * table de refus métier — il lit, et l'échec de lecture est un échec de lecture.
 */
import { computed, ref } from 'vue'

import ChampSaisie from '~/core/design-system/ChampSaisie.vue'
import type { ContexteAppel } from '~/core/auth'

import {
  chargerJournal,
  cleTypeAction,
  TYPES_ACTION,
  type EntreeJournal,
  type FiltresJournal,
  type PageJournal,
  type TypeAction,
} from './journal'

const { t } = useI18n()

const props = defineProps<{
  page: PageJournal
  contexte: ContexteAppel
  etablissementId: string
}>()

const entrees = ref<EntreeJournal[]>([...props.page.elements])
const suivant = ref<{ creeLe: string, id: string } | null>(
  props.page.suivant_cree_le && props.page.suivant_id
    ? { creeLe: props.page.suivant_cree_le, id: props.page.suivant_id }
    : null,
)

const enChargement = ref(false)
const erreur = ref<string | null>(null)

/** Les quatre filtres, **combinables** — le serveur les cumule (FR-037). */
const filtreType = ref('')
const filtreDepuis = ref('')
const filtreJusquA = ref('')

const optionsTypes = computed(() =>
  TYPES_ACTION.map(type => ({ valeur: type, libelleCle: cleTypeAction(type) })),
)

function filtres(): FiltresJournal {
  return {
    etablissementId: props.etablissementId,
    typeAction: (filtreType.value || undefined) as TypeAction | undefined,
    // Une date de champ vaut un jour entier : `[J, J+1)`. La borne de fin est **exclusive** côté
    // serveur, ce qui évite qu'une journée chevauche la suivante.
    depuis: filtreDepuis.value ? `${filtreDepuis.value}T00:00:00Z` : undefined,
    jusquA: filtreJusquA.value ? `${filtreJusquA.value}T00:00:00Z` : undefined,
  }
}

/** Recharge depuis le début — **tout changement de filtre repart de la tête**. */
async function appliquerFiltres(): Promise<void> {
  enChargement.value = true
  erreur.value = null
  try {
    const page = await chargerJournal(props.contexte, filtres())
    // Remplacement, jamais concaténation : un filtre appliqué qui ajouterait à la liste
    // existante rendrait un registre où figurent des entrées que le filtre exclut.
    entrees.value = page.elements
    suivant.value = page.suivant_cree_le && page.suivant_id
      ? { creeLe: page.suivant_cree_le, id: page.suivant_id }
      : null
  }
  catch {
    erreur.value = 'journal.chargement_impossible'
  }
  finally {
    enChargement.value = false
  }
}

/** Charge la page suivante — **concaténation, cette fois**, c'est la suite du même parcours. */
async function chargerSuite(): Promise<void> {
  if (!suivant.value) return
  enChargement.value = true
  erreur.value = null
  try {
    const page = await chargerJournal(props.contexte, filtres(), suivant.value)
    entrees.value = [...entrees.value, ...page.elements]
    suivant.value = page.suivant_cree_le && page.suivant_id
      ? { creeLe: page.suivant_cree_le, id: page.suivant_id }
      : null
  }
  catch {
    erreur.value = 'journal.chargement_impossible'
  }
  finally {
    enChargement.value = false
  }
}

/**
 * L'horodatage d'autorité, mis en forme.
 *
 * `Intl.DateTimeFormat` est employé ici et **pas** pour les montants : une date n'a pas d'unité
 * mineure ni de devise, et son format local est exactement ce que l'ICU du navigateur sait faire.
 * Les montants, eux, passent par `core/format/montant.ts` — et le registre de ce cycle n'en porte
 * aucun.
 */
function horodatage(iso: string): string {
  const date = new Date(iso)
  return new Intl.DateTimeFormat('fr-FR', {
    dateStyle: 'short',
    timeStyle: 'short',
  }).format(date)
}

/** Le nom de l'auteur, ou une phrase honnête. **Jamais un UUID.** */
function auteur(entree: EntreeJournal): string {
  return entree.auteur.nom || t('journal.auteur_inconnu')
}
</script>

<template>
  <section class="flex flex-col">
    <div class="flex flex-col gap-1.5 px-3.5 pt-4 pb-1">
      <span class="text-etiquette uppercase text-ink-3">{{ t('journal.sur_titre') }}</span>
      <h2 class="font-titre text-chiffre font-semibold text-ink">
        {{ t('journal.titre') }}
      </h2>
      <p class="text-corps text-ink-2">
        {{ t('journal.intro') }}
      </p>
    </div>

    <!-- Les filtres — combinables, et cumulés par le SERVEUR. Filtrer côté client sur une page
         déjà paginée donnerait une liste amputée qui aurait l'air complète. -->
    <div class="flex flex-col gap-3 px-3 pt-3">
      <ChampSaisie
        v-model="filtreType"
        etiquette-cle="journal.filtres.type"
        placeholder-cle="journal.filtres.type_tous"
        :options="optionsTypes"
        :desactive="enChargement"
      />
      <div class="flex gap-3">
        <div class="flex-1">
          <ChampSaisie
            v-model="filtreDepuis"
            etiquette-cle="journal.filtres.depuis"
            aide-cle="journal.filtres.format_date"
            :desactive="enChargement"
          />
        </div>
        <div class="flex-1">
          <ChampSaisie
            v-model="filtreJusquA"
            etiquette-cle="journal.filtres.jusqu_a"
            aide-cle="journal.filtres.format_date"
            :desactive="enChargement"
          />
        </div>
      </div>
      <button
        type="button"
        class="inline-flex h-11 w-full cursor-pointer items-center justify-center gap-2 rounded-lg border-[1.5px] border-line-2 bg-transparent font-titre text-action font-semibold text-ink-2 transition-colors duration-90 hover:bg-tile"
        :disabled="enChargement"
        @click="appliquerFiltres"
      >
        <i
          class="ph ph-funnel text-corps"
          aria-hidden="true"
        />
        {{ t('journal.filtres.appliquer') }}
      </button>
    </div>

    <div
      v-if="erreur"
      class="mx-3 mt-3 flex items-start gap-3 rounded-r-lg border-l-4 border-l-danger bg-danger-soft p-3.5"
      role="alert"
    >
      <i
        class="ph-fill ph-x-circle mt-0.5 shrink-0 text-titre-s text-danger"
        aria-hidden="true"
      />
      <p class="text-corps text-danger-fort">
        {{ t(erreur) }}
      </p>
    </div>

    <p
      v-if="entrees.length === 0 && !enChargement"
      class="px-3.5 py-6 text-corps text-ink-3"
    >
      {{ t('journal.aucune_entree') }}
    </p>

    <!-- REGISTRE SOBRE (`F2`) : pas de couleur par famille d'action, pas d'icône par ligne. Une
         remise en rouge et une ouverture de tiroir en orange transformeraient une liste de faits
         en tableau d'accusations. -->
    <ul class="flex flex-col divide-y divide-line px-3 pt-3 pb-3.5">
      <li
        v-for="entree in entrees"
        :key="entree.id"
        :data-entree="entree.id"
        class="flex flex-col gap-1 py-3"
      >
        <div class="flex items-baseline gap-3">
          <!-- L'horodatage d'AUTORITÉ, en chasse fixe pour que la colonne s'aligne. -->
          <span class="shrink-0 font-mono text-mini text-ink-3">
            {{ horodatage(entree.cree_le) }}
          </span>
          <span class="font-titre text-action font-semibold text-ink">
            {{ t(cleTypeAction(entree.type_action)) }}
          </span>
        </div>
        <p class="text-corps text-ink-2">
          {{ t('journal.par', { auteur: auteur(entree) }) }}
        </p>
      </li>
    </ul>

    <button
      v-if="suivant"
      type="button"
      class="mx-3 mb-4 inline-flex h-11 cursor-pointer items-center justify-center gap-2 rounded-lg border-[1.5px] border-line-2 bg-transparent font-titre text-action font-semibold text-ink-2 transition-colors duration-90 hover:bg-tile"
      :disabled="enChargement"
      @click="chargerSuite"
    >
      {{ t('journal.charger_suite') }}
    </button>
  </section>
</template>
