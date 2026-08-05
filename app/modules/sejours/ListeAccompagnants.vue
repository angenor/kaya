<script setup lang="ts">
/**
 * **Les accompagnants** — un nom suffit, et c'est une décision de budget.
 *
 * Écran `R3` (Arrivée), **dérivé de `R4`** — `docs/design/derivation.md`. Motif hérité : la
 * grammaire du tap, les mêmes filets, les mêmes jetons. Aucun motif neuf n'est inventé ici.
 *
 * # ★ UN NOM SUFFIT (FR-015), et le reste est facultatif
 *
 * Demander une pièce d'identité **par accompagnant** coûterait la cible des 60 secondes de
 * l'arrivée — quatre personnes, quatre pièces, quatre saisies de numéro. La fiche de police porte
 * les accompagnants nommés ; ce que la loi exige d'eux est le nom, pas la pièce. Le champ de
 * pièce existe côté serveur (`accompagnant.numero_piece`) et l'écran ne le propose pas : c'est une
 * **absence assumée**, pas un oubli.
 *
 * ⚠️ **`hebergement.accompagnant` est la SECONDE surface de rétention du produit.** La purge de
 * 90 jours de TRX-06 portera sur **deux** tables, `comptes.personne` et celle-ci. Ne rien
 * collecter ici qui ne serve pas la fiche de police est donc la manière la moins chère de tenir
 * cette obligation.
 *
 * # Tout champ passe par le composant 16
 *
 * `ChampSaisie.vue` — étiquette toujours visible, erreur au champ, plancher tactile de 44 px.
 * **Sans exception** : un `<input>` écrit à la main perdrait l'étiquette, l'`aria-describedby` et
 * la hauteur, trois choses qu'aucune relecture ne réclame et qu'un écran de comptoir paie.
 *
 * # L'ajout est LOCAL, et c'est ce qui tient la transaction unique
 *
 * Les accompagnants ne partent pas un par un : ils sont joints au corps de l'ouverture et écrits
 * **dans la même transaction** que le séjour. Un accompagnant déclaré à l'arrivée et perdu par un
 * second appel manqué ferait une **fiche de police fausse** — un document légal qui sous-déclare.
 */
import { computed, ref } from 'vue'

import ChampSaisie from '~/core/design-system/ChampSaisie.vue'
// ⚠️ `AccompagnantSaisi` vit dans `donnees.ts` et non ici : **`<script setup>` n'admet aucun
// `export`**. Le déclarer dans le bloc ferait échouer la compilation du SFC avec un message qui
// ne nomme pas la cause.
import type { AccompagnantSaisi } from './donnees'

const { t } = useI18n()

const props = defineProps<{
  accompagnants: AccompagnantSaisi[]
  /**
   * Nombre de personnes que la chambre accueille, **paramètre d'établissement**
   * (`categorie.capacite_accueil`) — jamais une constante. `null` quand aucune catégorie n'est
   * encore choisie : l'écran n'invente alors aucune limite.
   */
  capaciteAccueil: number | null
}>()

const emit = defineEmits<{ 'mettre-a-jour': [liste: AccompagnantSaisi[]] }>()

const nomSaisi = ref('')
const erreurCle = ref<string | null>(null)

/**
 * Le titulaire compte pour un — **le même calcul que le serveur** (FR-018).
 *
 * Il n'est pas recopié d'une colonne : le serveur le dérive à chaque lecture, et l'écran fait de
 * même. Deux dérivations qui s'accordent valent mieux qu'une valeur stockée qui se désynchronise
 * au premier retrait.
 */
const nombrePersonnes = computed(() => props.accompagnants.length + 1)

/** La capacité est-elle dépassée ? **Alerte, jamais blocage** : l'exploitant décide. */
const capaciteDepassee = computed(() =>
  props.capaciteAccueil !== null && nombrePersonnes.value > props.capaciteAccueil,
)

function ajouter(): void {
  const nom = nomSaisi.value.trim()
  if (!nom) {
    erreurCle.value = 'sejours.arrivee.accompagnants.nom_requis'
    return
  }
  erreurCle.value = null
  emit('mettre-a-jour', [
    ...props.accompagnants,
    // La clé de rendu est dérivée du rang et du nom : elle n'a pas besoin d'être un UUID, et en
    // engendrer un ici le rendrait différent de celui réellement envoyé.
    { cle: `${props.accompagnants.length}:${nom}`, nom },
  ])
  nomSaisi.value = ''
}

function retirer(cle: string): void {
  emit('mettre-a-jour', props.accompagnants.filter((a) => a.cle !== cle))
}
</script>

<template>
  <section class="flex flex-col gap-3">
    <header class="flex flex-col gap-1">
      <h2 class="font-titre text-titre-m font-semibold text-ink">
        {{ t('sejours.arrivee.accompagnants.titre') }}
      </h2>
      <p class="text-mini text-ink-3">
        {{ t('sejours.arrivee.accompagnants.un_nom_suffit') }}
      </p>
    </header>

    <ul
      v-if="props.accompagnants.length"
      class="flex flex-col gap-2"
    >
      <li
        v-for="accompagnant in props.accompagnants"
        :key="accompagnant.cle"
        class="flex items-center justify-between gap-3 rounded-xl border border-line bg-surf px-3.5 py-2.5"
        data-accompagnant
      >
        <span class="font-texte text-corps text-ink">{{ accompagnant.nom }}</span>
        <button
          type="button"
          data-action="retirer-accompagnant"
          class="rounded-lg border border-line px-3 py-1.5 text-mini text-ink-2 transition-colors duration-90 hover:bg-tile"
          @click="retirer(accompagnant.cle)"
        >
          {{ t('sejours.arrivee.accompagnants.retirer') }}
        </button>
      </li>
    </ul>

    <div class="flex items-end gap-2">
      <div class="flex-1">
        <ChampSaisie
          v-model="nomSaisi"
          etiquette-cle="sejours.arrivee.accompagnants.nom"
          placeholder-cle="sejours.arrivee.accompagnants.nom_invite"
          :erreur-cle="erreurCle"
          taille="comptoir"
          @keyup.enter="ajouter"
        />
      </div>
      <button
        type="button"
        data-action="ajouter-accompagnant"
        class="h-12 rounded-xl border border-line-2 px-4 font-titre text-corps font-semibold text-ink transition-colors duration-90 hover:bg-tile"
        @click="ajouter"
      >
        {{ t('sejours.arrivee.accompagnants.ajouter') }}
      </button>
    </div>

    <p class="text-mini text-ink-3">
      {{ t('sejours.arrivee.accompagnants.total_personnes', { n: nombrePersonnes }) }}
    </p>

    <!--
      ★ La capacité ALERTE, elle ne bloque pas. Un lit d'appoint, un enfant, une nuit de dépannage :
      l'exploitant sait des choses que la fiche de catégorie ignore. Bloquer ici le pousserait à
      saisir un accompagnant de moins — et la fiche de police deviendrait fausse pour de bon.
    -->
    <p
      v-if="capaciteDepassee"
      class="rounded-xl border border-dashed border-line-2 px-3.5 py-2.5 text-mini text-ink-2"
      role="status"
      data-alerte="capacite"
    >
      {{ t('sejours.arrivee.accompagnants.capacite_depassee', { n: props.capaciteAccueil }) }}
    </p>
  </section>
</template>
