<script setup lang="ts">
/**
 * **Les notes internes — le premier écran qui écrit en classe A.**
 *
 * # ÉCRAN COMPOSÉ, cas (c) de `docs/design/derivation.md`
 *
 * Il n'a pas de maquette et n'en aura pas. Sa conception est **entièrement issue de la
 * bibliothèque** de `docs/design/composants.md`, composant par composant :
 *
 * | Élément | Composant canonique |
 * |---|---|
 * | Chaque note de la liste | **08** — ligne de liste |
 * | Le champ de saisie | **16** — champ de saisie |
 * | Le bouton « Ajouter » | **01** — bouton principal |
 * | La liste sans note | **11** — état vide illustré |
 * | Pendant le chargement | **13** — squelette de chargement |
 *
 * Les quatre conditions de la catégorie sont vérifiées une par une dans sa ligne de la matrice :
 * liste et formulaire suivant un motif posé · conception issue de la bibliothèque · note interne
 * consultée rarement par un utilisateur formé · aucun doute sur son apparence. **Zone de charme** :
 * ni client en face, ni argent en jeu — c'est ce qui autorise le cas (c) ici et l'interdit sur un
 * écran de comptoir.
 *
 * # Ce que cet écran démontre, et pourquoi il fallait qu'il existe
 *
 * Un mécanisme sans passager réel est du code exporté et appelé nulle part — le défaut exact
 * d'`initialiserTheme()`. La file hors-ligne avait ce statut depuis deux cycles.
 *
 * Ici, **hors ligne, la saisie est acceptée** : pas de message d'erreur, pas de confirmation à
 * demander, pas de grisé. La note entre en file, le témoin passe à `n+1`, et la ligne apparaît
 * dans la liste avec la mention « en attente d'envoi ». C'est l'inverse exact d'une opération de
 * classe C — et c'est toute la distinction que le principe VI porte.
 *
 * # Les notes en attente sont AFFICHÉES, mêlées aux notes envoyées
 *
 * Les cacher jusqu'à leur envoi ferait disparaître de l'écran ce que l'utilisateur vient d'écrire.
 * Sur un terminal de comptoir, cela se lit comme « ma saisie n'a pas été prise ». Elles sont donc
 * là, en tête — ordre d'affichage **local**, par `horodatageClient`, ce qui est l'un des trois
 * usages que la porte P-23 exempte nommément.
 */
import { computed, ref } from 'vue'

import ChampSaisie from '~/core/design-system/ChampSaisie.vue'
import type { ContexteAppel } from '~/core/auth'
import { fileCourante, type EtatSynchronisation } from '~/core/sync'
import { useEtatSynchronisation } from '~/core/sync'

import {
  composerEntreeNote,
  TYPE_NOTE_CREEE,
  type Note,
  type PageNotes,
} from './notes'

const { t } = useI18n()

const props = defineProps<{
  page: PageNotes
  contexte: ContexteAppel
  tenantId: string
  etablissementId: string
}>()

/** Longueur maximale, telle que le contrat la fixe — « entre 1 et 2000 caractères ». */
const LONGUEUR_MAX = 2000

const texte = ref('')
const erreurCle = ref<string | null>(null)
const enCours = ref(false)

/** Les notes déjà envoyées, telles que le serveur les rend. */
const envoyees = ref<Note[]>([...props.page.elements])

const etat: Readonly<{ value: EtatSynchronisation }> = useEtatSynchronisation()

/**
 * Les notes **en attente d'envoi**, lues de la file.
 *
 * Filtrées sur le type ET sur l'établissement du contexte **figé à la saisie** : une note écrite
 * pour un autre établissement pendant une coupure n'a rien à faire sur cet écran-ci.
 */
const enAttente = computed(() => {
  // La dépendance à `etat` n'est pas décorative : c'est elle qui fait relire la file quand une
  // écriture part. Sans elle, la liste garderait ses « en attente » après leur envoi.
  void etat.value.enAttente

  const file = fileCourante()
  if (!file) {
    return []
  }
  return file
    .lister()
    .filter(e => e.type === TYPE_NOTE_CREEE && e.contexte.etablissementId === props.etablissementId)
    .map(e => ({
      id: e.id,
      texte: (e.charge as unknown as { texte: string }).texte,
      horodatageClient: e.horodatageClient,
    }))
})

function heure(horodatage: string | null | undefined): string {
  if (!horodatage) {
    return ''
  }
  const date = new Date(horodatage)
  // **Espace ORDINAIRE avant « h »**, jamais l'espace fine insécable : `tokens.md` §2 réserve
  // U+202F aux montants, et les heures gardent l'espace ordinaire (« 17 h 30 »).
  return `${String(date.getHours()).padStart(2, '0')} h ${String(date.getMinutes()).padStart(2, '0')}`
}

/**
 * Enregistre la note. **Aucune garde hors ligne, et c'est le point de l'écran.**
 *
 * Une opération de classe A s'enregistre toujours : la file la portera. Le seul refus possible est
 * une saisie vide ou trop longue, et il est **local** — le dire tout de suite vaut mieux que
 * d'attendre un aller-retour pour apprendre que le champ était vide.
 */
function enregistrer(): void {
  erreurCle.value = null

  const propre = texte.value.trim()
  if (propre.length === 0) {
    erreurCle.value = 'notes.champ_vide'
    return
  }
  if (propre.length > LONGUEUR_MAX) {
    erreurCle.value = 'notes.champ_trop_long'
    return
  }

  const file = fileCourante()
  if (!file) {
    // La file n'est pas branchée : le plugin d'amorçage n'a pas tourné. Le dire plutôt que de
    // perdre la saisie en silence.
    erreurCle.value = 'notes.chargement_impossible'
    return
  }

  enCours.value = true
  try {
    file.enfiler(
      composerEntreeNote(propre, {
        tenantId: props.tenantId,
        etablissementId: props.etablissementId,
      }),
    )
    texte.value = ''
  }
  finally {
    enCours.value = false
  }
}
</script>

<template>
  <section class="flex flex-col">
    <div class="flex flex-col gap-1.5 px-3.5 pt-4 pb-1">
      <h2 class="font-titre text-chiffre font-semibold text-ink">
        {{ t('notes.titre') }}
      </h2>
      <p class="text-corps text-ink-2">
        {{ t('notes.sous_titre') }}
      </p>
    </div>

    <!-- COMPOSANT 16 · CHAMP DE SAISIE + COMPOSANT 01 · BOUTON PRINCIPAL.
         Aucune garde hors ligne : la note est de classe A, elle s'enregistre toujours. -->
    <form
      class="flex flex-col gap-3 px-3 pt-3"
      @submit.prevent="enregistrer"
    >
      <ChampSaisie
        v-model="texte"
        etiquette-cle="note_etablissement.champ_texte"
        :erreur-cle="erreurCle"
        :erreur-valeurs="{ max: LONGUEUR_MAX }"
        taille="comptoir"
      />

      <button
        type="submit"
        class="inline-flex h-11 cursor-pointer items-center justify-center gap-2 rounded-lg bg-prim px-4 font-titre text-action font-semibold text-prim-ink shadow-basse transition-colors duration-90 hover:bg-prim-dk active:scale-97"
        :disabled="enCours"
      >
        <i
          class="ph ph-plus text-corps"
          aria-hidden="true"
        />
        {{ enCours ? t('notes.ajout_en_cours') : t('notes.ajouter') }}
      </button>
    </form>

    <!-- COMPOSANT 11 · ÉTAT VIDE ILLUSTRÉ — le motif ocre, une phrase qui dit ce qui apparaîtra
         ici, et l'action qui démarre est déjà au-dessus. -->
    <div
      v-if="envoyees.length === 0 && enAttente.length === 0"
      class="mx-3 mt-4 flex flex-col items-center gap-3 rounded-lg bg-tile px-4 py-8 text-center"
    >
      <i
        class="ph ph-note-pencil text-titre-l text-ocre"
        aria-hidden="true"
      />
      <p class="text-corps text-ink-2">
        {{ t('notes.vide') }}
      </p>
    </div>

    <!-- COMPOSANT 08 · LIGNE DE LISTE. Les notes en attente d'envoi passent en tête : ce que
         l'utilisateur vient d'écrire doit être là où il regarde. -->
    <ul
      v-else
      class="flex flex-col divide-y divide-line px-3 pt-4 pb-3.5"
    >
      <li
        v-for="note in enAttente"
        :key="note.id"
        class="flex flex-col gap-1 py-3"
      >
        <div class="flex items-baseline gap-3">
          <span class="shrink-0 font-mono text-mini text-ink-3">{{ heure(note.horodatageClient) }}</span>
          <!-- L'état « en attente d'envoi » du composant 08. Le mot « file » n'apparaît jamais :
               le lexique est catégorique — l'utilisateur voit « en attente d'envoi », rien de
               plus technique. -->
          <span class="inline-flex items-center gap-1.5 font-titre text-mini font-semibold text-alerte-fort">
            <i
              class="ph ph-clock text-mini"
              aria-hidden="true"
            />
            {{ t('notes.en_attente_envoi') }}
          </span>
        </div>
        <p class="text-corps text-ink">
          {{ note.texte }}
        </p>
      </li>

      <li
        v-for="note in envoyees"
        :key="note.id"
        class="flex flex-col gap-1 py-3"
      >
        <div class="flex items-baseline gap-3">
          <!-- `cree_le` — l'horodatage d'AUTORITÉ, celui du serveur. C'est lui qui s'affiche dès
               qu'une note est partie, jamais l'horodatage du terminal. -->
          <span class="shrink-0 font-mono text-mini text-ink-3">{{ heure(note.cree_le) }}</span>
        </div>
        <p class="text-corps text-ink">
          {{ note.texte }}
        </p>
      </li>
    </ul>
  </section>
</template>
