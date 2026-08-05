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
 *
 * # ⚠️ LES MODULES ACTIFS ÉTAIENT EN DUR À `[]`, ET DEUX ÉCRANS LIVRÉS ÉTAIENT INVISIBLES
 *
 * La ligne — supprimée ci-dessous — disait « vide à ce cycle, et c'est exact », ce qui l'était au
 * cycle 003 et a cessé de l'être au 004, quand « Vos formules » et « Vos chambres » ont pris
 * `moduleRequis: 'HEBERGEMENT'`. Personne n'est revenu la lire. Le raisonnement complet est en
 * tête de `core/accueil/modules-actifs.ts` ; ce qu'il faut retenir ici : **une valeur en dur
 * accompagnée d'un commentaire qui la justifie ne se relit plus.**
 *
 * Le câblage se fait dans la page, et c'est voulu : `core/` tient la règle et le cache,
 * `modules/etablissements` tient l'appel, et aucun des deux ne dépend de l'autre.
 */
import { computed, defineAsyncComponent, onMounted, ref } from 'vue'

import { contexteAppel, sessionCourante } from '~/core/auth'
import { chargerModulesActifs, modulesEnCache } from '~/core/accueil/modules-actifs'
import { adaptateurCourant } from '~/core/platform/courant'
import type { Permissions } from '~/core/rbac'

const EcranAccueil = defineAsyncComponent(() => import('~/modules/accueil/EcranAccueil.vue'))

const { t } = useI18n()
const config = useRuntimeConfig()

const pret = ref(false)
const session = ref(sessionCourante())

const permissions = computed<Permissions>(() => session.value?.permissions ?? [])

/**
 * Les modules d'activité actifs de l'établissement — **lus, jamais supposés**.
 *
 * Une tuile de verticale n'existe que si l'établissement propose le service. `HEBERGEMENT` est le
 * seul module que le produit sache activer aujourd'hui, mais rien ici ne le sait : la liste vient
 * de l'établissement, et un maquis qui n'ouvre que la restauration verra disparaître les cinq
 * tuiles d'hébergement sans qu'une ligne change.
 */
const modulesActifs = ref<readonly string[]>([])

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
  // Le middleware global a déjà repris la session — ou redirigé vers `R0`. Arriver ici sans
  // session est impossible ; la garde du template le vérifie quand même, parce qu'une invariante
  // qu'on affirme sans la relire finit par être fausse.
  session.value = sessionCourante()

  const etablissementId = session.value?.etablissementId
  const stockage = adaptateurCourant().stockagePersistant

  if (!etablissementId) {
    // Un compte sans établissement actif n'a aucun service à lire. Il garde ses tuiles
    // transverses — dont « Mes envois » — et n'attend pas un appel qui n'a pas de cible.
    pret.value = true
    return
  }

  // 1. Le dernier état connu, **tout de suite**. Sans réseau, c'est déjà l'accueil complet.
  modulesActifs.value = (await modulesEnCache(stockage, etablissementId)).codes
  pret.value = true

  // 2. Puis l'état réel. Le chargeur ne lève jamais : hors ligne il rend ce que le cache portait,
  //    donc cette seconde passe ne peut pas vider ce que la première a affiché.
  const contexte = contexteAppel(config.public.apiBaseUrl)
  if (!contexte) {
    return
  }
  const { chargerServices } = await import('~/modules/etablissements/donnees')
  const actifs = await chargerModulesActifs(
    () => chargerServices(contexte, etablissementId),
    etablissementId,
    stockage,
  )
  modulesActifs.value = actifs.codes
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
