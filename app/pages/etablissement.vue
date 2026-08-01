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

import type { Permissions } from '~/core/rbac'
import type { ContexteAppel, DonneesEcran } from '~/modules/etablissements/donnees'
import type { ServiceActif } from '~/modules/etablissements/services-visibles'

const EcranEtablissement = defineAsyncComponent(
  () => import('~/modules/etablissements/EcranEtablissement.vue'),
)

const { t } = useI18n()
const route = useRoute()
const config = useRuntimeConfig()

const donnees = ref<DonneesEcran | null>(null)
const erreur = ref<string | null>(null)

/** **Provisoire nommé** — le contexte vient de deux en-têtes tant que CPT-01 n'a pas livré l'authentification par jeton. */
const contexte = computed<ContexteAppel>(() => ({
  baseUrl: config.public.apiBaseUrl,
  tenantId: config.public.tenantId,
  compteId: config.public.compteId,
}))

/**
 * Permissions de l'utilisateur — **provisoire nommé, levé par CPT-02**.
 *
 * Les rôles n'existent pas encore : elles viennent de la configuration, en liste séparée par des
 * virgules. Ce qui est établi ici et ne changera pas, c'est la **règle d'affichage** — les rôles
 * sont cumulables et les permissions sont leur **union** (principe VII). Le jour où CPT-02 livre
 * les rôles, c'est cette ligne qui change, et une seule.
 *
 * **Poser une valeur par défaut vide est délibéré** : sans permission, l'écran se rend en lecture
 * seule, sans aucune action. C'est le comportement sûr, et c'est ce qui rend l'absence
 * observable — un défaut « tout permis » masquerait la règle jusqu'au cycle CPT.
 */
const permissions = computed<Permissions>(() =>
  String(config.public.permissions ?? '')
    .split(',')
    .map(p => p.trim())
    .filter(Boolean),
)

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
  const { chargerEcran } = await import('~/modules/etablissements/donnees')
  try {
    // Le contexte est celui du `computed` ci-dessus — voir `backend/api/src/contexte.rs`,
    // dérogation `CONTEXTE_PAR_EN_TETES`.
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
    v-if="donnees"
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
