<script setup lang="ts">
/**
 * **`G1` — Établissement et modules.** Le premier écran du produit.
 *
 * Référence visuelle — cas (b), **écran dérivé** : `docs/design/derivation.md` ligne
 * « `G1` Établissement et modules **hérite de `G2`** — Configuration ». Maquette lue :
 * `docs/design/html/G2-offre-hebergement.html`.
 *
 * **Le HTML de maquette n'est jamais copié ni déplacé vers `app/`** (porte P-19). On en lit les
 * valeurs et la structure — sélecteur en tête, sections à `h2` `font-titre text-chiffre`,
 * lignes-boutons `rounded-l-xs rounded-r-xl border-l-4`, bouton principal `h-13 rounded-xl
 * bg-prim` — et on réimplémente en composants Nuxt avec i18n, mode sombre et chargement paresseux,
 * que l'export ne contient pas.
 *
 * # Cette page est une coquille, et c'est ce qui rend le chargement paresseux EFFECTIF
 *
 * Le contenu métier vit dans `app/modules/etablissements/`, chargé par `defineAsyncComponent` :
 * le module part alors dans un fragment séparé, vérifiable sur la sortie de construction. Écrire
 * le contenu ici le ferait entrer dans le fragment de la route, et « chargement paresseux par
 * module » resterait une intention.
 *
 * # Aucun appel natif — rien de nouveau pour la porte P-15
 *
 * Le choix de fichier du logo est un `<input type="file">` **standard**. Aucune extension de
 * `PlatformAdapter` n'est nécessaire, et aucun `window.__TAURI__` n'apparaît ici ni dans les
 * composants du module.
 */
import { computed, defineAsyncComponent, onMounted, ref } from 'vue'

import { contexteAppel, sessionCourante, type ContexteAppel } from '~/core/auth'
import type { Permissions } from '~/core/rbac'
import type { DonneesEcran } from '~/modules/etablissements/donnees'
import type { ServiceActif } from '~/modules/etablissements/services-visibles'

const EcranEtablissement = defineAsyncComponent(
  () => import('~/modules/etablissements/EcranEtablissement.vue'),
)

const { t } = useI18n()
const route = useRoute()
const config = useRuntimeConfig()

const donnees = ref<DonneesEcran | null>(null)
const erreur = ref<string | null>(null)

/**
 * Contexte d'appel — **le provisoire de deux en-têtes est levé** (CPT-01).
 *
 * Il vient de la session ouverte, donc du jeton vérifié par le serveur. `null` tant que personne
 * n'est connecté : la page ne charge rien et renvoie à `R0`.
 */
const contexte = computed<ContexteAppel | null>(() => contexteAppel(config.public.apiBaseUrl))

/**
 * Permissions de l'utilisateur — **le provisoire nommé est levé** (CPT-02).
 *
 * Elles ne viennent plus de la configuration en liste séparée par des virgules, mais de la
 * **réponse de connexion**, où le serveur a calculé l'union des rôles portés sur l'établissement
 * actif (FR-017). La règle d'affichage, elle, n'a pas changé d'un mot : permission absente →
 * action **absente**.
 *
 * Une session absente donne une liste vide, donc un écran en lecture seule. C'est le comportement
 * sûr, et il rend l'absence observable.
 */
const permissions = computed<Permissions>(() => sessionCourante()?.permissions ?? [])

/**
 * Remplace la liste des services après une écriture, **sans rechargement de page**.
 *
 * La nouvelle liste vient du serveur, relue par la section qui a écrit — jamais reconstruite à la
 * main côté client : le serveur fait foi (principe VI).
 */
function remplacerServices(services: ServiceActif[]): void {
  if (donnees.value) {
    donnees.value = { ...donnees.value, services }
  }
}

// Le module de chargement est importé **dynamiquement lui aussi** : le laisser en import statique
// le ferait entrer dans le fragment de la route, et le client d'API avec lui — ce qui annulerait
// le bénéfice du chargement paresseux sur la page la plus légère du produit.
onMounted(async () => {
  // Sans session, il n'y a pas de contexte d'appel : l'écran le dit et renvoie à la connexion,
  // plutôt que de partir en requête pour récolter un `401`.
  if (!contexte.value) {
    erreur.value = t('connexion.requise')
    return
  }

  const { chargerEcran } = await import('~/modules/etablissements/donnees')
  try {
    // Le contexte porte le jeton d'accès — `backend/api/src/contexte.rs`, refondu par CPT-01.
    donnees.value = await chargerEcran(
      contexte.value,
      String(route.query.etablissement ?? config.public.etablissementId),
    )
  }
  catch {
    // Aucun détail technique à l'écran : l'utilisateur ne peut rien en faire, et le message
    // traduit dit ce qu'il peut faire — réessayer.
    erreur.value = t('etablissement.chargement_impossible')
  }
})
</script>

<template>
  <EcranEtablissement
    v-if="donnees && contexte"
    :etablissement="donnees.etablissement"
    :services="donnees.services"
    :referentiel-modules="donnees.referentielModules"
    :points-de-vente="donnees.pointsDeVente"
    :configuration="donnees.configuration"
    :contexte="contexte"
    :permissions="permissions"
    @services-changes="remplacerServices"
  />
  <main
    v-else
    class="flex min-h-screen items-center justify-center bg-bg p-6"
  >
    <p class="font-texte text-corps text-ink-2">
      {{ erreur ?? t('etablissement.chargement') }}
    </p>
  </main>
</template>
