<script setup lang="ts">
/**
 * **`G2` — L'offre d'hébergement.** Écran **maquetté**, cas (a) de `docs/Kaya_Design.md` §2.
 *
 * Références exactes : `docs/design/html/G2-offre-hebergement.html` (hôtel, quatre formules) et
 * `docs/design/html/G2-offre-hebergement-residence.html` (résidence, deux formules).
 *
 * **Le HTML de maquette n'est jamais copié ni déplacé vers `app/`** (porte P-19) : on en lit les
 * valeurs et la structure, on réimplémente avec i18n, mode sombre, RBAC et chargement paresseux —
 * que l'export ne contient pas.
 *
 * # ⚠️ UNE SEULE RACINE, ET C'EST UN ÉLÉMENT
 *
 * Jamais un `v-if`/`v-else` de premier niveau. Une racine multiple compile en **fragment** ; un
 * fragment dont la branche active est un `defineAsyncComponent` non encore résolu a un `el` **nul**,
 * et Vue lève `Cannot read properties of null (reading 'parentNode')` à la navigation suivante.
 * L'écran ne se monte pas, l'ancien reste affiché, l'adresse a pourtant changé.
 *
 * La cause a été **établie par expérience** au cycle 003 — quatre pages sondes, une variable à la
 * fois — et il faut les trois conditions réunies : racine multiple, composant paresseux, bascule
 * après montage. Une racine unique suffit à l'éliminer, et le chargement paresseux reste intact.
 *
 * # Cette page est une coquille, et c'est ce qui rend le chargement paresseux EFFECTIF
 *
 * Le contenu métier vit dans `app/modules/hebergement/`, chargé par `defineAsyncComponent` : le
 * module part dans un fragment séparé, vérifiable sur la sortie de construction. L'écrire ici le
 * ferait entrer dans le fragment de la route — « un serveur de salle ne télécharge pas le code du
 * back-office » resterait une intention.
 *
 * # Le thème et la session ne sont PAS amorcés ici
 *
 * `plugins/01.theme.client.ts` pose le thème avant le rendu, `middleware/01.session.global.ts`
 * reprend la session avant chaque navigation, `layouts/default.vue` porte la coquille. C'est la
 * huitième couche du module doré, et sa règle : chaque page qui amorce pour elle-même ce qu'elle a
 * pensé à amorcer finit par en oublier un.
 */
import { computed, defineAsyncComponent, onMounted, ref } from 'vue'

import { contexteAppel, sessionCourante, type ContexteAppel } from '~/core/auth'
import type { Permissions } from '~/core/rbac'
import type { DonneesOffre, FormuleVue } from '~/modules/hebergement/donnees'

const EcranOffre = defineAsyncComponent(
  () => import('~/modules/hebergement/EcranOffre.vue'),
)

const { t } = useI18n()
const route = useRoute()
const config = useRuntimeConfig()

const donnees = ref<DonneesOffre | null>(null)
const erreur = ref<string | null>(null)

const contexte = computed<ContexteAppel | null>(() => contexteAppel(config.public.apiBaseUrl))
const permissions = computed<Permissions>(() => sessionCourante()?.permissions ?? [])

/**
 * L'établissement à afficher — **la SESSION d'abord**, la configuration en dernier recours.
 *
 * L'ordre est celui que P-22 a imposé au cycle 003 : `runtimeConfig.etablissementId` vaut `''`
 * depuis que CPT-01 en a retiré les valeurs d'identité, et une page qui le lisait en premier
 * produisait `GET /api/v1/etablissements/` — un 404 sur un établissement parfaitement lisible.
 *
 * La `query` reste **en tête** : c'est elle qui permettra à ETB-06 de pointer un autre
 * établissement que l'actif.
 */
const etablissementId = computed(() => String(
  route.query.etablissement ?? sessionCourante()?.etablissementId ?? config.public.etablissementId,
))

/** Remplace les formules après une écriture, **sans rechargement de page**. */
function remplacerFormules(formules: FormuleVue[]): void {
  if (donnees.value) {
    donnees.value = { ...donnees.value, formules }
  }
}

onMounted(async () => {
  if (!contexte.value) {
    erreur.value = t('connexion.requise')
    return
  }

  // Import dynamique du module de chargement lui aussi : le laisser en import statique le ferait
  // entrer dans le fragment de la route, et le client d'API avec lui.
  const { chargerOffre } = await import('~/modules/hebergement/donnees')
  try {
    donnees.value = await chargerOffre(contexte.value, etablissementId.value)
  }
  catch {
    // Aucun détail technique à l'écran : l'utilisateur ne peut rien en faire, et le message
    // traduit dit ce qu'il peut faire — réessayer.
    erreur.value = t('hebergement.offre.chargement_impossible')
  }
})
</script>

<template>
  <div class="flex flex-1 flex-col">
    <EcranOffre
      v-if="donnees && contexte"
      :categories="donnees.categories"
      :formules="donnees.formules"
      :contexte="contexte"
      :etablissement-id="etablissementId"
      :permissions="permissions"
      @formules-changees="remplacerFormules"
    />
    <div
      v-else
      class="flex flex-1 items-center justify-center p-6"
    >
      <p class="font-texte text-corps text-ink-2">
        {{ erreur ?? t('hebergement.offre.chargement') }}
      </p>
    </div>
  </div>
</template>
