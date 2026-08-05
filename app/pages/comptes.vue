<script setup lang="ts">
/**
 * **`G3` — Utilisateurs et rôles**, coquille de page.
 *
 * # Cette page est une coquille, et c'est ce qui rend le chargement paresseux EFFECTIF
 *
 * Le contenu métier vit dans `app/modules/comptes/`, chargé par `defineAsyncComponent` : le module
 * part alors dans un fragment séparé, vérifiable sur la sortie de construction. Écrire le contenu
 * ici le ferait entrer dans le fragment de la route, et « chargement paresseux par module »
 * resterait une intention.
 *
 * C'est le même patron que `pages/etablissement.vue` au cycle 002, et il compte davantage ici :
 * un serveur qui n'a pas `cpt.compte.lire` ne doit **télécharger aucun morceau** de cet écran
 * (FR-028). Le chargement paresseux se constate sur la sortie de construction ; il ne se déclare
 * pas.
 *
 * # Aucun appel natif — rien de nouveau pour la porte P-15
 *
 * L'écran n'écrit ni ne lit de secret. Le seul usage d'une capacité native du cycle est le
 * stockage du jeton de rafraîchissement, et il vit dans `core/auth`.
 */
import { computed, defineAsyncComponent, onMounted, ref } from 'vue'

import { contexteAppel, sessionCourante, type ContexteAppel } from '~/core/auth'
import { peutOuvrirEcran } from '~/core/acces/ecrans'
import type { Permissions } from '~/core/rbac'
import type { CompteVue, DonneesComptes } from '~/modules/comptes/donnees'

const EcranComptes = defineAsyncComponent(() => import('~/modules/comptes/EcranComptes.vue'))

const { t } = useI18n()
const config = useRuntimeConfig()

const donnees = ref<DonneesComptes | null>(null)
const erreur = ref<string | null>(null)

const contexte = computed<ContexteAppel | null>(() => contexteAppel(config.public.apiBaseUrl))
const permissions = computed<Permissions>(() => sessionCourante()?.permissions ?? [])
const etablissementId = computed(
  () => sessionCourante()?.etablissementId ?? String(config.public.etablissementId),
)

/**
 * **La permission de LIRE garde l'accès direct à la page** (FR-029).
 *
 * L'accueil ne montre pas la tuile sans elle ; encore faut-il que quelqu'un qui tape l'adresse
 * n'obtienne pas l'écran. Le serveur refuserait de toute façon en `403`, mais l'utilisateur
 * verrait alors un écran vide sans savoir pourquoi.
 */
// ⚠️ La permission n'est plus écrite ici. Elle vient de `core/acces/ecrans.ts`, la MÊME
// table que consulte l'accueil pour décider d'afficher la tuile : deux déclarations
// divergent au premier cycle, et la divergence donne soit une tuile qui ouvre sur un
// refus, soit un écran caché mais atteignable.
const peutLire = computed(() => peutOuvrirEcran('/comptes', permissions.value))

function remplacerComptes(comptes: CompteVue[]): void {
  if (donnees.value) {
    donnees.value = { ...donnees.value, comptes }
  }
}

onMounted(async () => {
  if (!contexte.value) {
    erreur.value = t('connexion.requise')
    return
  }
  if (!peutLire.value) {
    erreur.value = t('comptes.acces_refuse')
    return
  }

  const { chargerComptes } = await import('~/modules/comptes/donnees')
  try {
    donnees.value = await chargerComptes(contexte.value, etablissementId.value)
  }
  catch {
    // Aucun détail technique à l'écran : l'utilisateur ne peut rien en faire, et le message
    // traduit dit ce qu'il peut faire — réessayer.
    erreur.value = t('comptes.chargement_impossible')
  }
})
</script>

<template>
  <div class="flex flex-1 flex-col">
    <EcranComptes
      v-if="donnees && contexte"
      :comptes="donnees.comptes"
      :referentiel-roles="donnees.referentielRoles"
      :contexte="contexte"
      :etablissement-id="etablissementId"
      :permissions="permissions"
      @comptes-changes="remplacerComptes"
    />
    <div
      v-else
      class="flex flex-1 items-center justify-center p-6"
    >
      <p class="font-texte text-corps text-ink-2">
        {{ erreur ?? t('comptes.chargement') }}
      </p>
    </div>
  </div>
</template>
