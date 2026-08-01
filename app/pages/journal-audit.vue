<script setup lang="ts">
/**
 * **`G4` — Registre des actions**, coquille de page.
 *
 * # Sans la permission, la tuile est absente ET l'accès direct refusé
 *
 * FR-029. L'accueil ne montre pas la tuile sans `cpt.audit.consulter` ; encore faut-il que
 * quelqu'un qui tape l'adresse n'obtienne pas l'écran. Le serveur refuserait de toute façon en
 * `403`, mais l'utilisateur verrait alors une liste vide sans savoir pourquoi — et une liste vide
 * est le pire message qu'un registre puisse donner.
 *
 * # Chargement paresseux — il compte ici plus qu'ailleurs
 *
 * `cpt.audit.consulter` n'appartient qu'à `proprietaire` et `comptable`. Une serveuse ne doit
 * **télécharger aucun morceau** de cet écran : c'est ce que `defineAsyncComponent` garantit, et
 * cela se constate sur la sortie de construction.
 */
import { computed, defineAsyncComponent, onMounted, ref } from 'vue'

import { contexteAppel, sessionCourante, type ContexteAppel } from '~/core/auth'
import { detient, type Permissions } from '~/core/rbac'
import type { PageJournal } from '~/modules/audit/journal'

const EcranJournalAudit = defineAsyncComponent(
  () => import('~/modules/audit/EcranJournalAudit.vue'),
)

const { t } = useI18n()
const config = useRuntimeConfig()

const page = ref<PageJournal | null>(null)
const erreur = ref<string | null>(null)

const contexte = computed<ContexteAppel | null>(() => contexteAppel(config.public.apiBaseUrl))
const permissions = computed<Permissions>(() => sessionCourante()?.permissions ?? [])
const etablissementId = computed(
  () => sessionCourante()?.etablissementId ?? String(config.public.etablissementId),
)

const peutConsulter = computed(() => detient(permissions.value, 'cpt.audit.consulter'))

onMounted(async () => {
  if (!contexte.value) {
    erreur.value = t('connexion.requise')
    return
  }
  if (!peutConsulter.value) {
    erreur.value = t('journal.acces_refuse')
    return
  }

  const { chargerJournal } = await import('~/modules/audit/journal')
  try {
    page.value = await chargerJournal(contexte.value, {
      etablissementId: etablissementId.value,
    })
  }
  catch {
    erreur.value = t('journal.chargement_impossible')
  }
})
</script>

<template>
  <EcranJournalAudit
    v-if="page && contexte"
    :page="page"
    :contexte="contexte"
    :etablissement-id="etablissementId"
  />
  <main
    v-else
    class="flex min-h-screen items-center justify-center bg-bg p-6"
  >
    <p class="font-texte text-corps text-ink-2">
      {{ erreur ?? t('journal.chargement') }}
    </p>
  </main>
</template>
