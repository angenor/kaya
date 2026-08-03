<script setup lang="ts">
/**
 * **Composant 10 · Témoin de synchronisation** — « le composant le plus important du produit ».
 *
 * `docs/design/composants.md` §10, et le superlatif est du document, pas de ce fichier. Son rôle y
 * tient en une phrase : **dire si le travail est en sécurité**.
 *
 * # Trois états, chacun avec sa FORME et sa PHRASE
 *
 * | État | Forme | Phrase | Pouls |
 * |---|---|---|---|
 * | connecté | pastille pleine, `succes` | « Enregistré » | oui, lent (2,4 s) |
 * | connexion faible | pastille creuse, `alerte` | « Connexion faible » | non |
 * | hors connexion | pastille barrée, `danger` | « Hors connexion » | non |
 *
 * **La couleur ne porte jamais l'état seule** (règle 2 de `tokens.md` §1). Sur un 1366 × 768
 * délavé par le soleil d'Abengourou, une pastille verte et une pastille orange se ressemblent. La
 * forme et le texte le disent aussi — c'est la même règle que le champ de saisie, qui signale une
 * erreur par trois signaux et non un.
 *
 * # JAMAIS de pourcentage — la règle explicite du composant
 *
 * « Un nombre d'écritures et une heure, jamais une barre de progression. » Un pourcentage suppose
 * qu'on connaisse le total, ce qui est faux : la file grandit pendant qu'elle se vide. Et il ne
 * répond pas à la question posée — **mon travail est-il parti ?** — à laquelle « 4 en attente
 * d'envoi » répond exactement.
 *
 * `app/tests/temoin-sync.spec.ts` refuse tout `%` dans le rendu, dans les trois états et les deux
 * langues.
 *
 * # Le passage hors ligne est INSTANTANÉ, sans transition
 *
 * Règle du composant, et elle a un motif : une pastille qui fondrait doucement du vert au rouge
 * ferait douter de l'instant où l'état a changé. Un témoin de sécurité ne s'anime pas quand il
 * passe au rouge. Le pouls, lui, est lent — 2,4 s, `--animate-pulse-reseau` — parce qu'il
 * **rassure** ; il n'alerte pas.
 *
 * Concrètement : `transition-none` sur la pastille, et l'animation posée **uniquement** sur l'état
 * connecté.
 *
 * # Le mot « dégradé » n'atteint jamais l'écran
 *
 * `docs/design/lexique.md` : « Dégradé » est un terme d'ingénieur ; on dit **« Connexion
 * faible »**. De même « Enregistré » plutôt que « Connecté » — le premier dit ce qui compte pour
 * Aminata, le second décrit le réseau. L'i18n disait « Connecté » ; le lexique fait foi, et
 * l'écart est corrigé dans le même cycle.
 *
 * # Il est dans la COQUILLE, donc sur toutes les pages
 *
 * C'est ce que « indicateur permanent » veut dire (principe VI). Le monter écran par écran
 * garantirait qu'un écran l'oublie — et ce serait celui où l'on écrit.
 */
import { computed } from 'vue'

import { useEtatSynchronisation } from '~/core/sync'

const { t } = useI18n()

const props = withDefaults(
  defineProps<{
    /**
     * Variante **compacte** — pastille seule, sans phrase (composants.md §10, « barre d'en-tête »).
     *
     * La phrase reste accessible : elle passe en `aria-label`. Un témoin muet pour un lecteur
     * d'écran ne dirait plus si le travail est en sécurité, ce qui est exactement son objet.
     */
    compact?: boolean
  }>(),
  { compact: false },
)

const etat = useEtatSynchronisation()

/** L'état d'affichage — **trois valeurs, jamais quatre**. */
const forme = computed(() => {
  switch (etat.value.reseau) {
    case 'hors_ligne':
      return 'hors_ligne' as const
    case 'degrade':
      return 'degrade' as const
    default:
      return 'connecte' as const
  }
})

/**
 * La phrase, **et le nombre s'il y a lieu**.
 *
 * L'attente l'emporte sur l'état du réseau : « 4 en attente d'envoi » est plus utile que
 * « Enregistré » quand quatre écritures n'ont pas encore été acceptées par le serveur. Dire
 * « Enregistré » alors que quelque chose attend serait le seul mensonge que ce composant ne peut
 * pas se permettre.
 */
const phrase = computed(() => {
  if (etat.value.enAttente > 0) {
    return t('reseau.en_attente', { n: etat.value.enAttente }, etat.value.enAttente)
  }
  return t(`reseau.${forme.value}`)
})

/** L'icône de la pastille — **la forme**, distincte de la couleur. */
const icone = computed(() => {
  switch (forme.value) {
    case 'hors_ligne':
      return 'ph-cloud-slash'
    case 'degrade':
      return 'ph-cloud-warning'
    default:
      return 'ph-cloud-check'
  }
})

const tonTexte = computed(() => {
  switch (forme.value) {
    case 'hors_ligne':
      return 'text-danger-fort'
    case 'degrade':
      return 'text-alerte-fort'
    default:
      return 'text-succes-fort'
  }
})

const tonPastille = computed(() => {
  switch (forme.value) {
    case 'hors_ligne':
      return 'bg-danger'
    case 'degrade':
      return 'bg-alerte'
    default:
      return 'bg-succes'
  }
})
</script>

<template>
  <!-- Racine UNIQUE et élément — jamais un fragment. La leçon du cycle 004 : un fragment dont la
       branche active est un composant paresseux non résolu a un `el` nul, et Vue lève
       `Cannot read properties of null` à la navigation suivante. -->
  <span
    class="inline-flex items-center gap-2 font-titre text-mini font-semibold"
    :class="tonTexte"
    role="status"
    aria-live="polite"
    :aria-label="phrase"
    :data-etat="forme"
  >
    <!-- LA FORME — pastille + icône. Deux signaux en plus de la couleur, parce qu'un état n'est
         jamais porté par la couleur seule. -->
    <span class="relative inline-flex size-2.5 shrink-0 transition-none">
      <span
        class="absolute inset-0 rounded-pleine transition-none"
        :class="tonPastille"
      />
      <!-- Le POULS, et lui seul, est animé — 2,4 s, et seulement à l'état connecté. Il rassure ;
           il n'alerte pas. Le passage hors ligne, lui, est instantané. -->
      <span
        v-if="forme === 'connecte'"
        class="absolute inset-0 rounded-pleine animate-pulse-reseau"
        :class="tonPastille"
        aria-hidden="true"
      />
    </span>

    <i
      class="ph text-corps"
      :class="icone"
      aria-hidden="true"
    />

    <!-- LA PHRASE. Absente en variante compacte, où elle reste portée par `aria-label`. -->
    <span v-if="!props.compact">{{ phrase }}</span>

    <!-- La quarantaine n'est PAS sur le témoin : elle vit sur `S1`. Le témoin répond à « mon
         travail est-il parti ? » ; ce qui a été refusé définitivement demande une décision, donc
         un écran, donc du temps que le comptoir n'a pas. -->
  </span>
</template>
