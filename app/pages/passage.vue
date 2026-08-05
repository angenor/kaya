<script setup lang="ts">
/**
 * **`R4` — Le passage.** Écran **MAQUETTÉ**, cas (a) de `docs/Kaya_Design.md` §2.
 *
 * Référence : `docs/design/html/R4-passage.html` et ses quatre états.
 *
 * # La route est `/passage`, et le nom du fichier la décide
 *
 * Jamais `/check-in` : « check-in » est **écarté du lexique** (v1.6.0), et **une URL est visible**
 * dans la barre d'adresse. C'est la leçon `S1` du cycle 005 — le mot proscrit rentrerait par la
 * porte du nom de fichier, sans qu'aucune porte i18n ne le voie.
 *
 * # ⚠️ UNE SEULE RACINE, ET C'EST UN ÉLÉMENT
 *
 * Jamais un `v-if`/`v-else` de premier niveau. Une racine multiple compile en fragment ; un
 * fragment dont la branche active est un `defineAsyncComponent` non résolu a un `el` nul, et Vue
 * lève `Cannot read properties of null (reading 'parentNode')` **à la navigation suivante** —
 * l'écran ne se monte pas, l'ancien reste affiché, et l'adresse a pourtant changé. C'est la leçon
 * la plus chère du cycle 003.
 *
 * # Coquille, et chargement paresseux effectif
 *
 * Le contenu métier vit dans `app/modules/sejours/`, chargé par `defineAsyncComponent` : un
 * établissement sans module hébergement ne télécharge pas cet écran.
 */
import { computed, defineAsyncComponent, onMounted, ref } from 'vue'

import { contexteAppel, sessionCourante, type ContexteAppel } from '~/core/auth'
import { peutOuvrirEcran } from '~/core/acces/ecrans'
import type { Permissions } from '~/core/rbac'
import type { ClientResume, DonneesPassage } from '~/modules/sejours/donnees'

const EcranPassage = defineAsyncComponent(
  () => import('~/modules/sejours/EcranPassage.vue'),
)

const { t } = useI18n()
const route = useRoute()
const config = useRuntimeConfig()

const donnees = ref<DonneesPassage | null>(null)
const erreur = ref<string | null>(null)

/**
 * Le client reconnu au comptoir — état `-connu` de la maquette.
 *
 * ⚠️ **Il arrive par la recherche, jamais par une reconnaissance automatique.** Deviner qui est
 * en face à partir d'un numéro de téléphone rattacherait un séjour à la mauvaise fiche une fois
 * sur cent, et l'erreur ne se verrait qu'au départ.
 */
const clientReconnu = ref<ClientResume | null>(null)
const passagesDuClient = ref(0)

const contexte = computed<ContexteAppel | null>(() => contexteAppel(config.public.apiBaseUrl))
const permissions = computed<Permissions>(() => sessionCourante()?.permissions ?? [])

/** La SESSION d'abord, la configuration en dernier recours — l'ordre imposé par P-22. */
const etablissementId = computed(() => String(
  route.query.etablissement ?? sessionCourante()?.etablissementId ?? config.public.etablissementId,
))

function oublierClient(): void {
  clientReconnu.value = null
  passagesDuClient.value = 0
}

/**
 * **Sans la permission, la tuile est absente ET l'accès direct refusé** — FR-026 + FR-029.
 *
 * La question et sa réponse viennent de `core/acces/ecrans.ts`, la MÊME table que consulte
 * l'accueil pour décider d'afficher la tuile. Les écrire des deux côtés produirait deux vérités
 * qui divergent : une tuile qui ouvre sur un refus, ou un écran caché mais atteignable par l'URL.
 *
 * Le serveur refuserait de toute façon en `403`. Ce que la garde ajoute n'est pas la sécurité,
 * c'est **la langue** : sans elle, l'écran affichait « sejours.passage.chargement_impossible » — un message d'échec
 * technique, qui envoie chercher un problème de réseau qui n'existe pas.
 */
const peutOuvrir = computed(() => peutOuvrirEcran('/passage', permissions.value))

onMounted(async () => {
  if (!contexte.value) {
    erreur.value = t('connexion.requise')
    return
  }
  // AVANT l'appel, jamais après : un refus obtenu au bout de trente secondes d'attente
  // réseau est un refus que l'utilisateur a déjà cessé de lire.
  if (!peutOuvrir.value) {
    erreur.value = t('sejours.passage.acces_refuse')
    return
  }

  const { chargerPassage } = await import('~/modules/sejours/donnees')
  try {
    donnees.value = await chargerPassage(contexte.value, etablissementId.value)
  }
  catch {
    erreur.value = t('sejours.passage.chargement_impossible')
  }
})
</script>

<template>
  <div class="flex flex-1 flex-col p-4 sm:p-6">
    <EcranPassage
      v-if="donnees && contexte"
      :contexte="contexte"
      :etablissement-id="etablissementId"
      :donnees="donnees"
      :permissions="permissions"
      :client-reconnu="clientReconnu"
      :passages-du-client="passagesDuClient"
      @oublier-client="oublierClient"
    />
    <div
      v-else
      class="flex flex-1 items-center justify-center p-6"
    >
      <p class="font-texte text-corps text-ink-2">
        {{ erreur ?? t('sejours.passage.chargement') }}
      </p>
    </div>
  </div>
</template>
