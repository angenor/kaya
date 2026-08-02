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
 * # Aucune session ⇒ la connexion, pas un écran vide — et c'est le MIDDLEWARE qui s'en charge
 *
 * `R0` est l'écran par lequel tout le monde entre. Y renvoyer plutôt que d'afficher un accueil
 * sans tuiles évite de confondre « personne n'est connecté » et « ce compte n'a aucun droit », qui
 * se ressemblent à l'écran et n'ont rien à voir.
 *
 * Cette page portait la reprise de session **pour elle seule**, et les cinq autres routes ne
 * l'avaient pas. Elle vit désormais dans `middleware/01.session.global.ts`, qui s'exécute avant
 * chaque navigation. La rappeler ici serait une seconde source de la même règle, donc une
 * divergence en attente : quand le middleware a laissé passer, la session existe.
 */
import { computed, defineAsyncComponent, onMounted, ref } from 'vue'

import { sessionCourante } from '~/core/auth'
import type { Permissions } from '~/core/rbac'

const EcranAccueil = defineAsyncComponent(() => import('~/modules/accueil/EcranAccueil.vue'))

const { t } = useI18n()

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

onMounted(() => {
  // Le middleware global a déjà repris la session — ou redirigé vers `R0`. Arriver ici sans
  // session est impossible ; la garde du template le vérifie quand même, parce qu'une invariante
  // qu'on affirme sans la relire finit par être fausse.
  session.value = sessionCourante()
  pret.value = true
})
</script>

<template>
  <div class="flex flex-1 flex-col">
    <EcranAccueil
      v-if="pret && session"
      :nom-affichage="nomAffichage"
      :permissions="permissions"
      :modules-actifs="modulesActifs"
    />
    <div
      v-else
      class="flex flex-1 items-center justify-center p-6"
    >
      <p class="font-texte text-corps text-ink-2">
        {{ t('accueil.chargement') }}
      </p>
    </div>
  </div>
</template>
