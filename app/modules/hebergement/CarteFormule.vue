<script setup lang="ts">
/**
 * **Une formule, sur une ligne** — le motif central de `G2`.
 *
 * Référence visuelle : `docs/design/html/G2-offre-hebergement.html`, le bouton
 * `rounded-l-xs rounded-r-xl border border-line border-l-4 border-l-line-2 bg-surf shadow-basse`.
 * **Le HTML n'est pas copié** (porte P-19) : on en lit les valeurs et la structure, et on
 * réimplémente avec i18n, mode sombre et RBAC, que l'export ne contient pas.
 *
 * # Trois lignes de texte, et la troisième est la raison de l'écran
 *
 * La maquette met le prix et la règle de taxe **sur la même carte** parce que ce sont « les deux
 * seules choses qu'un propriétaire vient vérifier ». La troisième ligne n'est donc pas un détail
 * de complétude : c'est la moitié de ce que M. Koffi est venu voir.
 *
 * # Le montant passe par `formaterMontant`, et par rien d'autre
 *
 * `app/core/format/montant.ts` — l'unique implémentation du produit. Le montant est un **entier
 * d'unité mineure** et la devise vient de l'établissement : ni `Intl.NumberFormat`, dont le
 * séparateur dépend de l'ICU embarqué, ni un `money(n)` recopié de `tokens.md`, qui est du code de
 * maquette mono-devise.
 *
 * # Aucune classe `dark:`
 *
 * Les noms de jetons sont identiques dans les deux thèmes et seules les valeurs changent sous
 * `.dark` : `bg-surf text-ink` bascule tout seul.
 */
import { computed } from 'vue'

import { formaterMontant } from '~/core/format/montant'
import type { FormuleVue } from './donnees'

const { t } = useI18n()

const props = defineProps<{
  formule: FormuleVue
  /** Le type de chambre qui porte la formule — son nom se lit sous le libellé de famille. */
  nomCategorie: string
}>()

defineEmits<{ choisir: [formule: FormuleVue] }>()

/**
 * L'icône Phosphor de la famille, telle que la maquette la pose.
 *
 * Une table explicite plutôt qu'une convention de nommage : les quatre glyphes sont **sous-réglés**
 * dans la police embarquée (P-21b), et un nom calculé produirait un carré vide le jour où une
 * famille s'ajouterait sans que personne ne régénère la police.
 */
const ICONES: Record<string, string> = {
  NUITEE: 'ph-moon-stars',
  PASSAGE: 'ph-clock',
  DEMI_JOURNEE: 'ph-sun-horizon',
  MENSUEL: 'ph-calendar-dots',
}

const icone = computed(() => ICONES[props.formule.famille] ?? 'ph-bed')

/** Clé i18n du nom de la famille — jamais le code brut, qui est du vocabulaire de table. */
const libelleFamille = computed(() => `hebergement.familles.${props.formule.famille}`)

/**
 * **Le prix d'appel du passage est « à partir de »**, celui des autres est ferme.
 *
 * La maquette écrit « à partir de 1 500 F l'heure » pour le passage et « 12 500 F la nuit » pour
 * la nuitée. La différence n'est pas cosmétique : le passage a un barème, et annoncer son premier
 * palier comme un prix ferme donnerait un montant faux à qui reste quatre heures.
 */
const cleUnite = computed(() =>
  props.formule.famille === 'PASSAGE'
    ? 'hebergement.offre.prix_a_partir_de'
    : `hebergement.offre.prix_${props.formule.famille}`,
)

const montant = computed(() =>
  formaterMontant(props.formule.prix_mineur, props.formule.devise),
)

/**
 * La mention fiscale — **les deux seules du lexique**, et il n'en existe pas de troisième.
 *
 * La contrainte `formule_regle_fiscale_coherente` rend impossible une formule assujettie sans
 * règle de conversion : « paramétrage fiscal en attente » n'existe donc ni ici, ni au lexique, ni
 * à la maquette.
 */
const mentionFiscale = computed(() =>
  props.formule.assujettie_taxe_nuitee
    ? 'hebergement.offre.taxe_comprise'
    : 'hebergement.offre.taxe_absente',
)
</script>

<template>
  <button
    type="button"
    class="w-full p-3 rounded-l-xs rounded-r-xl border border-line border-l-4 border-l-line-2 bg-surf shadow-basse flex items-center gap-3 cursor-pointer transition-transform duration-90 ease-entree active:translate-y-px"
    @click="$emit('choisir', formule)"
  >
    <span class="size-10 shrink-0 rounded-xl bg-tile inline-flex items-center justify-center">
      <i
        :class="['ph', icone, 'text-titre-m', 'text-ocre']"
        aria-hidden="true"
      />
    </span>
    <span class="flex-1 min-w-0 flex flex-col items-start gap-0.75 text-left">
      <span class="font-titre text-titre-s font-semibold text-ink">
        {{ t(libelleFamille) }}
      </span>
      <span class="text-mini text-ink-3">{{ nomCategorie }}</span>
      <span class="text-corps text-ink-2">
        <!-- Le montant est en Chivo Mono tabulaire et `whitespace-nowrap` : la fine insécable
             U+202F empêche déjà la coupure, la classe la rend impossible même si la police de
             repli devait servir. -->
        <i18n-t
          :keypath="cleUnite"
          scope="global"
        >
          <template #montant>
            <span class="font-mono whitespace-nowrap">{{ montant }}</span>
          </template>
        </i18n-t>
      </span>
      <span class="text-mini text-ink-3">{{ t(mentionFiscale) }}</span>
    </span>
    <span
      class="w-0 h-0 shrink-0 border-y-5 border-y-transparent border-l-6 border-l-ink-3"
      aria-hidden="true"
    />
  </button>
</template>
