<script setup lang="ts">
/**
 * **`G5` — Chambres et types de chambre.** Écran **COMPOSÉ**, cas (c) de `docs/Kaya_Design.md` §2.
 *
 * Inscrit à `docs/design/derivation.md` (v1.3.0) avec les mentions « composé » et « à valider à
 * l'atelier terrain ». **Aucune maquette** : l'assemblage vient des seize composants canoniques, et
 * la couverture a été vérifiée motif par motif avant d'écrire une ligne.
 *
 * | Besoin | Composant |
 * |---|---|
 * | Liste des chambres, groupées par type | **08** ligne de liste |
 * | Deux formulaires | **16** champ de saisie |
 * | Choix du type | **16**, état « choix fermé » — **pas le 12** : six types, et sa règle s'arrête à quatre |
 * | Actions | **01 · 02 · 03** |
 * | Aucune chambre dans un type | **11** état vide illustré |
 * | Chargement | **13** squelette |
 *
 * # Zone de CHARME, et c'est ce qui autorise le cas (c)
 *
 * Adjoua règle son parc à l'ouverture puis y revient à la marge : elle n'est ni debout, ni
 * pressée, sans client en face ni argent en jeu. **Un écran de comptoir se maquette toujours** —
 * celui-ci n'en est pas un.
 *
 * # Ce que l'écran ne montre pas
 *
 * Aucun statut d'occupation — il est dérivé, et son écran est `R2` (tranche SEJ). Aucune action
 * sur le sous-statut de ménage — c'est HEB-06, hors périmètre.
 */
import { computed, ref } from 'vue'

import { useEtatReseau } from '~/core/platform/reseau'
import { detient, type Permissions } from '~/core/rbac'
import FormulaireCategorie from './FormulaireCategorie.vue'
import FormulaireUnite from './FormulaireUnite.vue'
import ListeUnites from './ListeUnites.vue'
import {
  chargerCategories,
  chargerUnites,
  type CategorieVue,
  type ContexteAppel,
  type UniteVue,
} from './donnees'
import { corrigerUnite, creerCategorie, creerUnite } from './ecrire-parc'
import { PERMISSION_GERER } from './modifier-formule'

const { t } = useI18n()

const props = defineProps<{
  categories: CategorieVue[]
  unites: UniteVue[]
  contexte: ContexteAppel
  etablissementId: string
  permissions: Permissions
}>()

const emit = defineEmits<{
  'categories-changees': [categories: CategorieVue[]]
  'unites-changees': [unites: UniteVue[]]
}>()

const reseau = useEtatReseau()
const enLigne = computed(() => reseau.value === 'connecte')
const peutGerer = computed(() => detient(props.permissions, PERMISSION_GERER))

/** Le panneau ouvert. `null` = la liste seule. */
const panneau = ref<'unite' | 'categorie' | null>(null)
/** La chambre en correction. `null` avec `panneau === 'unite'` = création. */
const uniteChoisie = ref<UniteVue | null>(null)
/** Chargement — porte la CIBLE, pas un booléen (composant 13). */
const enCours = ref<string | null>(null)
/** Refus métier. Une seule variable : **jamais deux bandeaux empilés** (composant 07). */
const refus = ref<{ cle: string, valeurs?: Record<string, unknown> } | null>(null)

function ouvrirCreation(): void {
  uniteChoisie.value = null
  panneau.value = 'unite'
  refus.value = null
}

function ouvrirCorrection(unite: UniteVue): void {
  uniteChoisie.value = unite
  panneau.value = 'unite'
  refus.value = null
}

function ouvrirTypeDeChambre(): void {
  panneau.value = 'categorie'
  refus.value = null
}

function fermer(): void {
  panneau.value = null
  uniteChoisie.value = null
  refus.value = null
}

async function enregistrerUnite(valeurs: {
  categorieId: string
  code: string
  etage: number | null
}): Promise<void> {
  const cible = uniteChoisie.value
  enCours.value = cible?.id ?? 'creation'
  refus.value = null

  const resultat = cible
    ? await corrigerUnite(
        props.contexte,
        props.etablissementId,
        cible.id,
        valeurs.code,
        valeurs.etage,
        reseau.value,
      )
    : await creerUnite(
        props.contexte,
        props.etablissementId,
        valeurs.categorieId,
        valeurs.code,
        valeurs.etage,
        reseau.value,
      )

  enCours.value = null

  if (resultat.issue === 'refus') {
    refus.value = { cle: resultat.cle, valeurs: resultat.valeurs }
    return
  }

  // **Le rafraîchissement suit le succès.** Une requête, pas deux : les types n'ont pas bougé. Et
  // la liste vient du serveur, jamais reconstruite côté client — il fait foi en conflit.
  emit('unites-changees', await chargerUnites(props.contexte, props.etablissementId))
  fermer()
}

async function enregistrerCategorie(valeurs: {
  nom: string
  capaciteAccueil: number
}): Promise<void> {
  enCours.value = 'categorie'
  refus.value = null

  const resultat = await creerCategorie(
    props.contexte,
    props.etablissementId,
    valeurs.nom,
    valeurs.capaciteAccueil,
    reseau.value,
  )

  enCours.value = null

  if (resultat.issue === 'refus') {
    refus.value = { cle: resultat.cle, valeurs: resultat.valeurs }
    return
  }

  emit('categories-changees', await chargerCategories(props.contexte, props.etablissementId))
  fermer()
}
</script>

<template>
  <section class="flex flex-1 flex-col">
    <div class="px-3.5 pt-4 pb-1 flex flex-col gap-1.5">
      <span class="text-etiquette uppercase text-ink-3">
        {{ t('hebergement.chambres.surtitre') }}
      </span>
      <h2 class="font-titre text-chiffre font-semibold text-ink">
        {{ t('hebergement.chambres.titre') }}
      </h2>
      <p class="text-corps text-ink-2">
        {{ t('hebergement.chambres.sous_titre', { nombre: unites.length }) }}
      </p>
    </div>

    <div
      v-if="refus"
      class="mx-3 mt-2 rounded-l-xs rounded-r-xl border-l-4 border-l-danger bg-danger-soft px-3.5 py-3"
      role="alert"
    >
      <p class="text-corps text-danger-fort">
        <i
          class="ph-fill ph-warning-circle"
          aria-hidden="true"
        />
        {{ t(refus.cle, refus.valeurs ?? {}) }}
      </p>
    </div>

    <!-- Hors ligne : les actions disparaissent et un bandeau DIT pourquoi. Classe C. -->
    <div
      v-else-if="!enLigne"
      class="mx-3 mt-2 rounded-l-xs rounded-r-xl border-l-4 border-l-info bg-info-soft px-3.5 py-3"
    >
      <p class="text-corps text-info-fort">
        {{ t('hebergement.chambres.refus.reseau') }}
      </p>
    </div>

    <div class="flex-1 min-h-0 overflow-y-auto py-3">
      <ListeUnites
        :categories="categories"
        :unites="unites"
        :peut-gerer="peutGerer && enLigne"
        :en-cours="enCours"
        @corriger="ouvrirCorrection"
      />
    </div>

    <!-- Panneau de saisie — un seul à la fois, ouvert par une action, fermé par « Annuler ». -->
    <div
      v-if="panneau"
      class="mx-3 mb-3 rounded-2xl border border-line bg-surf p-3.5 shadow-carte"
    >
      <FormulaireUnite
        v-if="panneau === 'unite'"
        :categories="categories"
        :unite="uniteChoisie"
        @enregistrer="enregistrerUnite"
        @annuler="fermer"
      />
      <FormulaireCategorie
        v-else
        @enregistrer="enregistrerCategorie"
        @annuler="fermer"
      />
    </div>

    <!-- Actions — ABSENTES sans la permission et hors ligne. Ni `disabled`, ni infobulle. -->
    <div
      v-if="peutGerer && enLigne && !panneau"
      class="shrink-0 px-3 pt-2.75 pb-3.5 bg-surf border-t border-line flex flex-col gap-2"
    >
      <button
        type="button"
        class="w-full h-13 rounded-xl bg-prim text-prim-ink font-titre text-titre-s font-semibold inline-flex items-center justify-center gap-2.5 cursor-pointer shadow-bouton-grand transition-[transform,box-shadow] duration-90 ease-entree active:translate-y-0.75 active:shadow-none"
        @click="ouvrirCreation"
      >
        <i
          class="ph ph-plus text-titre-m"
          aria-hidden="true"
        />
        {{ t('hebergement.chambres.ajouter_unite') }}
      </button>
      <!-- Action discrète — composant 03, hors chemin critique : on crée un type de chambre une
           fois par saison, une chambre plusieurs fois par an. -->
      <button
        type="button"
        class="w-full h-9 rounded-md text-prim font-titre text-corps font-medium cursor-pointer transition-colors duration-90 hover:bg-prim-soft"
        @click="ouvrirTypeDeChambre"
      >
        {{ t('hebergement.chambres.ajouter_categorie') }}
      </button>
    </div>
  </section>
</template>
