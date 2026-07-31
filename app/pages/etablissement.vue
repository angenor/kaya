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
import { defineAsyncComponent, onMounted, ref } from 'vue'

import type { DonneesEcran } from '~/modules/etablissements/donnees'

const EcranEtablissement = defineAsyncComponent(
  () => import('~/modules/etablissements/EcranEtablissement.vue'),
)

const { t } = useI18n()
const route = useRoute()
const config = useRuntimeConfig()

const donnees = ref<DonneesEcran | null>(null)
const erreur = ref<string | null>(null)

// Le module de chargement est importé **dynamiquement lui aussi** : le laisser en import statique
// le ferait entrer dans le fragment de la route, et le client d'API avec lui — ce qui annulerait
// le bénéfice du chargement paresseux sur la page la plus légère du produit.
onMounted(async () => {
  const { chargerEcran } = await import('~/modules/etablissements/donnees')
  try {
    donnees.value = await chargerEcran(
      {
        baseUrl: config.public.apiBaseUrl,
        // **Provisoire nommé** — le contexte vient de deux en-têtes tant que CPT-01 n'a pas livré
        // l'authentification par jeton. Voir `backend/api/src/contexte.rs`, dérogation
        // `CONTEXTE_PAR_EN_TETES`.
        tenantId: config.public.tenantId,
        compteId: config.public.compteId,
      },
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
