<script setup lang="ts">
/**
 * **`G5` — Chambres et types de chambre.** Écran **COMPOSÉ**, cas (c) de `docs/Kaya_Design.md` §2.
 *
 * Inscrit à `docs/design/derivation.md` v1.3.0, mentions « composé » et « à valider à l'atelier
 * terrain ». Sans cette ligne, la porte P-19 refuserait l'écran — même mécanique que `R0` au
 * cycle 003.
 *
 * # ⚠️ UNE SEULE RACINE, ET C'EST UN ÉLÉMENT
 *
 * Jamais un `v-if`/`v-else` de premier niveau. Une racine multiple compile en fragment ; un
 * fragment dont la branche active est un `defineAsyncComponent` non résolu a un `el` nul, et Vue
 * lève `Cannot read properties of null (reading 'parentNode')` à la navigation suivante.
 *
 * # Coquille, et chargement paresseux effectif
 *
 * Le contenu métier vit dans `app/modules/hebergement/`, chargé par `defineAsyncComponent`.
 */
import { computed, defineAsyncComponent, onMounted, ref } from 'vue'

import { contexteAppel, sessionCourante, type ContexteAppel } from '~/core/auth'
import { peutOuvrirEcran } from '~/core/acces/ecrans'
import type { Permissions } from '~/core/rbac'
import type { CategorieVue, DonneesChambres, UniteVue } from '~/modules/hebergement/donnees'

const EcranChambres = defineAsyncComponent(
  () => import('~/modules/hebergement/EcranChambres.vue'),
)

const { t } = useI18n()
const route = useRoute()
const config = useRuntimeConfig()

const donnees = ref<DonneesChambres | null>(null)
const erreur = ref<string | null>(null)

const contexte = computed<ContexteAppel | null>(() => contexteAppel(config.public.apiBaseUrl))
const permissions = computed<Permissions>(() => sessionCourante()?.permissions ?? [])

/** La SESSION d'abord, la configuration en dernier recours — l'ordre imposé par P-22. */
const etablissementId = computed(() => String(
  route.query.etablissement ?? sessionCourante()?.etablissementId ?? config.public.etablissementId,
))

function remplacerUnites(unites: UniteVue[]): void {
  if (donnees.value) {
    donnees.value = { ...donnees.value, unites }
  }
}

function remplacerCategories(categories: CategorieVue[]): void {
  if (donnees.value) {
    donnees.value = { ...donnees.value, categories }
  }
}

/**
 * **Sans la permission, la tuile est absente ET l'accès direct refusé** — FR-026 + FR-029.
 *
 * La question et sa réponse viennent de `core/acces/ecrans.ts`, la MÊME table que consulte
 * l'accueil pour décider d'afficher la tuile. Les écrire des deux côtés produirait deux vérités
 * qui divergent : une tuile qui ouvre sur un refus, ou un écran caché mais atteignable par l'URL.
 *
 * Le serveur refuserait de toute façon en `403`. Ce que la garde ajoute n'est pas la sécurité,
 * c'est **la langue** : sans elle, l'écran affichait « hebergement.chambres.chargement_impossible » — un message d'échec
 * technique, qui envoie chercher un problème de réseau qui n'existe pas.
 */
const peutOuvrir = computed(() => peutOuvrirEcran('/chambres', permissions.value))

onMounted(async () => {
  if (!contexte.value) {
    erreur.value = t('connexion.requise')
    return
  }
  // AVANT l'appel, jamais après : un refus obtenu au bout de trente secondes d'attente
  // réseau est un refus que l'utilisateur a déjà cessé de lire.
  if (!peutOuvrir.value) {
    erreur.value = t('hebergement.chambres.acces_refuse')
    return
  }

  const { chargerChambres } = await import('~/modules/hebergement/donnees')
  try {
    donnees.value = await chargerChambres(contexte.value, etablissementId.value)
  }
  catch {
    erreur.value = t('hebergement.chambres.chargement_impossible')
  }
})
</script>

<template>
  <div class="flex flex-1 flex-col">
    <EcranChambres
      v-if="donnees && contexte"
      :categories="donnees.categories"
      :unites="donnees.unites"
      :contexte="contexte"
      :etablissement-id="etablissementId"
      :permissions="permissions"
      @unites-changees="remplacerUnites"
      @categories-changees="remplacerCategories"
    />
    <div
      v-else
      class="flex flex-1 items-center justify-center p-6"
    >
      <p class="font-texte text-corps text-ink-2">
        {{ erreur ?? t('hebergement.chambres.chargement') }}
      </p>
    </div>
  </div>
</template>
