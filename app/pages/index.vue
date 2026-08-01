<script setup lang="ts">
/**
 * **`R1` — L'accueil**, coquille de page. *La dette du cycle 002 est soldée.*
 *
 * Le cycle 001 avait posé ici un placeholder — « Aucun écran n'est livré à ce cycle » —, et le
 * cycle 002 a explicitement reporté l'accueil « au cycle CPT » : sans permissions, il n'y avait
 * rien à filtrer. C'est fait.
 *
 * # La page reste une coquille, et c'est ce qui rend le chargement paresseux EFFECTIF
 *
 * Le contenu vit dans `app/modules/accueil/`, chargé par `defineAsyncComponent`. Ici plus
 * qu'ailleurs : l'accueil est le premier écran après la connexion, sur le pire réseau du produit.
 * Écrire son contenu dans la page ferait entrer le catalogue de tuiles et ses dépendances dans le
 * fragment de la route racine, que **tout le monde** télécharge.
 *
 * # Aucune session ⇒ la connexion, pas un écran vide
 *
 * `R0` est l'écran par lequel tout le monde entre. Y renvoyer plutôt que d'afficher un accueil
 * sans tuiles évite de confondre « personne n'est connecté » et « ce compte n'a aucun droit », qui
 * se ressemblent à l'écran et n'ont rien à voir.
 */
import { computed, defineAsyncComponent, onMounted, ref } from 'vue'

import { reprendreSession, sessionCourante } from '~/core/auth'
import { useEtatReseau } from '~/core/platform/reseau'
import type { Permissions } from '~/core/rbac'

const EcranAccueil = defineAsyncComponent(() => import('~/modules/accueil/EcranAccueil.vue'))

const { t } = useI18n()
const config = useRuntimeConfig()
const reseau = useEtatReseau()

const pret = ref(false)
const session = ref(sessionCourante())

const permissions = computed<Permissions>(() => session.value?.permissions ?? [])

/**
 * Les modules d'activité actifs — **vide à ce cycle, et c'est exact**.
 *
 * Aucune tuile du catalogue n'exige de module : les dix-sept permissions de `0016` sont toutes
 * transverses, et les écrans des verticales n'existent pas. Le jour où `HEBERGEMENT` ouvrira une
 * tuile, cette ligne lira la liste des services actifs — une requête, pas une refonte.
 */
const modulesActifs = computed<readonly string[]>(() => [])

/**
 * Le nom affiché vient de la session.
 *
 * `SessionUtilisateur` ne porte pas encore le nom de la personne : la réponse de connexion rend
 * `compte_id`, `tenant_id` et l'établissement actif. Le résoudre demanderait un appel de plus au
 * premier écran, sur le pire réseau du produit. La pastille et l'en-tête portent donc un libellé
 * générique tant qu'ETB-06 n'apporte pas le sélecteur de contexte permanent, qui le chargera de
 * toute façon. **Écrit plutôt que masqué par une chaîne vide.**
 */
const nomAffichage = computed(() => t('accueil.utilisateur'))

onMounted(async () => {
  // Reprise silencieuse : le jeton de rafraîchissement survit au rechargement, et redemander le
  // mot de passe à chaque ouverture d'onglet le ferait écrire sur un papier au comptoir.
  if (!session.value) {
    await reprendreSession(config.public.apiBaseUrl, reseau.value)
    session.value = sessionCourante()
  }

  if (!session.value) {
    await navigateTo('/connexion')
    return
  }
  pret.value = true
})
</script>

<template>
  <EcranAccueil
    v-if="pret && session"
    :nom-affichage="nomAffichage"
    :permissions="permissions"
    :modules-actifs="modulesActifs"
  />
  <main
    v-else
    class="flex min-h-screen items-center justify-center bg-bg p-6"
  >
    <p class="font-texte text-corps text-ink-2">
      {{ t('accueil.chargement') }}
    </p>
  </main>
</template>
