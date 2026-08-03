<script setup lang="ts">
/**
 * ★ **`R5` — Fiche client et recherche.** Écran **DÉRIVÉ**, jamais maquetté.
 *
 * Référence : `docs/design/derivation.md`, ligne `R5` → motif **`R7`** : *« Liste + fiche, pas de
 * total. »*
 *
 * # ★ TROIS FORMES, UNE SEULE ENTRÉE — et l'opérateur ne choisit pas de mode
 *
 * Un nom, un numéro de téléphone ou un numéro de pièce : **le serveur déduit**. Un sélecteur de
 * mode serait un geste de plus au comptoir et, surtout, une occasion de se tromper de mode et de
 * conclure que la fiche n'existe pas — puis d'en créer une seconde. Le doublon ne se verrait
 * qu'au moment où deux historiques divergent.
 *
 * # ★ LA RECHERCHE RÉDUIT LA LISTE PENDANT LA FRAPPE
 *
 * Elle est **débattue** — 250 ms — pour ne pas envoyer une requête par touche sur le réseau
 * d'Abengourou. Le délai n'est pas un réglage de confort : sans lui, « Bakayoko » produit huit
 * requêtes dont sept sont périmées à leur arrivée, et **la réponse de la sixième peut arriver
 * après celle de la huitième** — la liste afficherait alors les résultats d'un préfixe. Chaque
 * réponse est donc **estampillée** de son terme, et une réponse qui ne correspond plus à la
 * saisie courante est jetée.
 *
 * # La troncature se DIT
 *
 * `tronque` n'est pas décoratif : une liste silencieusement coupée est un mensonge sur un écran
 * de comptoir.
 *
 * # L'action de création est ABSENTE sans `sej.client.gerer` (FR-026)
 *
 * Jamais grisée. Le contrôle porte sur le **HTML rendu**, jamais sur un attribut — un attribut se
 * retire depuis la console du navigateur.
 *
 * # Cet écran ne dépend d'AUCUN module d'activité
 *
 * Les deux permissions sont **transversales** (`module_code = NULL`, migration `0030`) : un maquis
 * ou un bar seul en aura besoin dès SEJ-05, sans hébergement.
 */
import { computed, onBeforeUnmount, ref, watch } from 'vue'

import ChampSaisie from '~/core/design-system/ChampSaisie.vue'
import { useEtatReseau } from '~/core/platform'
import type { Permissions } from '~/core/rbac'
import { detient } from '~/core/rbac'
import FicheClient from './FicheClient.vue'
import {
  chargerHistoriqueClient,
  chercherClients,
  lireFicheClient,
  type ClientResume,
  type ContexteAppel,
  type FicheClientDetail,
  type SejourVue,
} from './donnees'

const { t } = useI18n()

const props = defineProps<{
  contexte: ContexteAppel
  permissions: Permissions
}>()

/** La permission de gérer. **Sans elle la création est ABSENTE**, jamais grisée (FR-026). */
const PERM_GERER = 'sej.client.gerer'

/**
 * Le délai de débat, en millisecondes.
 *
 * ⚠️ **Ce n'est pas un paramètre métier** (porte P-12) : il ne décide d'aucune règle, il borne un
 * nombre de requêtes. Le régler par établissement n'aurait aucun sens — c'est une propriété de la
 * frappe humaine, pas de l'exploitation.
 */
const DEBAT_MS = 250

const reseau = useEtatReseau()
const horsLigne = computed(() => reseau.value !== 'connecte')
const peutGerer = computed(() => detient(props.permissions, PERM_GERER))

const recherche = ref('')
const resultats = ref<ClientResume[]>([])
const tronque = ref(false)
const chercheEnCours = ref(false)
const messageCle = ref<string | null>(null)

const choisi = ref<string | null>(null)
const fiche = ref<FicheClientDetail | null>(null)
const historique = ref<SejourVue[]>([])
const chargementFiche = ref(false)
const chargementHistorique = ref(false)

let minuteur: ReturnType<typeof setTimeout> | null = null

/**
 * ★ **Chaque réponse est estampillée de son terme.**
 *
 * Sans cela, la réponse d'un préfixe arrivée en retard écraserait celle de la saisie complète, et
 * l'écran afficherait des résultats qui ne correspondent plus à ce qui est écrit dans le champ.
 * Le défaut est intermittent et dépend du réseau — donc invisible en développement.
 */
async function lancerRecherche(terme: string): Promise<void> {
  if (!terme.trim()) {
    resultats.value = []
    tronque.value = false
    messageCle.value = null
    return
  }

  if (horsLigne.value) {
    resultats.value = []
    messageCle.value = 'sejours.clients.hors_ligne'
    return
  }

  chercheEnCours.value = true
  messageCle.value = null
  try {
    const resultat = await chercherClients(props.contexte, terme)
    // La saisie a changé pendant l'aller-retour : cette réponse est périmée.
    if (terme !== recherche.value.trim()) return
    resultats.value = resultat.clients
    tronque.value = resultat.tronque
    if (!resultat.clients.length) messageCle.value = 'sejours.clients.aucun_resultat'
  }
  catch {
    if (terme === recherche.value.trim()) messageCle.value = 'sejours.clients.recherche_impossible'
  }
  finally {
    if (terme === recherche.value.trim()) chercheEnCours.value = false
  }
}

watch(recherche, (terme) => {
  if (minuteur) clearTimeout(minuteur)
  minuteur = setTimeout(() => void lancerRecherche(terme.trim()), DEBAT_MS)
})

onBeforeUnmount(() => {
  if (minuteur) clearTimeout(minuteur)
})

/**
 * Ouvre une fiche — **deux appels, en parallèle**.
 *
 * ⚠️ **L'historique est servi par le crate `hebergement`**, la fiche par `socle/comptes` : les
 * réunir côté serveur serait une jointure inter-schémas (P-04) *et* une arête `socle/ →
 * verticales/` (P-03). Le chemin HTTP cache ce découpage ; c'est l'écran qui les rassemble.
 */
async function ouvrir(client: ClientResume): Promise<void> {
  choisi.value = client.id
  fiche.value = null
  historique.value = []
  chargementFiche.value = true
  chargementHistorique.value = true
  messageCle.value = null

  const [detail, sejours] = await Promise.allSettled([
    lireFicheClient(props.contexte, client.id),
    chargerHistoriqueClient(props.contexte, client.id),
  ])

  chargementFiche.value = false
  chargementHistorique.value = false

  if (detail.status === 'fulfilled') {
    fiche.value = detail.value
  }
  else {
    messageCle.value = 'sejours.clients.fiche_illisible'
    return
  }

  // ★ Un historique illisible n'efface pas la fiche : un rôle peut porter `sej.client.lire` sans
  // `heb.sejour.lire`, et voir alors une fiche **sans** historique est le comportement voulu pour
  // un compte de portée restreinte — pas une erreur.
  historique.value = sejours.status === 'fulfilled' ? sejours.value : []
}
</script>

<template>
  <!-- ⚠️ UNE SEULE RACINE, ET C'EST UN ÉLÉMENT. Voir la note de la page. -->
  <div class="flex flex-col gap-6 lg:flex-row lg:items-start">
    <!-- ═══ COLONNE GAUCHE — la recherche et la liste ═══ -->
    <section class="flex flex-col gap-3 lg:w-96 lg:shrink-0">
      <div class="flex items-end gap-2">
        <div class="flex-1">
          <ChampSaisie
            v-model="recherche"
            etiquette-cle="sejours.clients.chercher"
            aide-cle="sejours.clients.chercher_aide"
            placeholder-cle="sejours.clients.chercher_invite"
            taille="comptoir"
          />
        </div>
        <!--
          ★ La création n'existe QUE si la permission de gérer est là — absente, jamais grisée.
        -->
        <button
          v-if="peutGerer"
          type="button"
          data-action="creer-fiche"
          class="h-12 rounded-xl border border-line-2 px-4 font-titre text-corps font-semibold text-ink transition-colors duration-90 hover:bg-tile"
        >
          {{ t('sejours.clients.creer') }}
        </button>
      </div>

      <!-- Squelette pendant la frappe — jamais un écran vide (composant 13). -->
      <div
        v-if="chercheEnCours"
        class="flex flex-col gap-2"
        data-chargement
      >
        <span class="h-11 w-full rounded-xl bg-tile animate-souffle" />
        <span class="h-11 w-full rounded-xl bg-tile animate-souffle" />
      </div>

      <ul
        v-else-if="resultats.length"
        class="flex flex-col gap-1.5"
      >
        <li
          v-for="client in resultats"
          :key="client.id"
        >
          <button
            type="button"
            :data-client="client.id"
            :aria-pressed="choisi === client.id"
            class="flex w-full items-baseline justify-between gap-3 rounded-xl border px-3.5 py-3 text-left transition-colors duration-90"
            :class="choisi === client.id
              ? 'border-ocre bg-ocre-soft'
              : 'border-line bg-surf hover:bg-tile'"
            @click="ouvrir(client)"
          >
            <span class="font-titre text-corps font-semibold text-ink">{{ client.nom }}</span>
            <span
              v-if="client.telephone"
              class="font-mono text-mini text-ink-3"
            >{{ client.telephone }}</span>
          </button>
        </li>
      </ul>

      <!--
        ★ La troncature se DIT. Une liste silencieusement coupée est un mensonge sur un écran de
        comptoir : l'opérateur conclurait que la fiche n'existe pas et en créerait une seconde.
      -->
      <p
        v-if="tronque"
        class="text-mini text-ink-3"
        data-troncature
      >
        {{ t('sejours.clients.resultats_tronques') }}
      </p>

      <p
        v-if="messageCle"
        class="rounded-xl border border-dashed border-line-2 px-3.5 py-2.5 text-mini text-ink-2"
        role="status"
        data-message
      >
        {{ t(messageCle) }}
      </p>
    </section>

    <!-- ═══ COLONNE DROITE — la fiche ═══ -->
    <section class="flex min-w-0 flex-1 flex-col gap-3">
      <div
        v-if="chargementFiche"
        class="flex flex-col gap-2.5"
        data-chargement-fiche
      >
        <span class="h-6 w-1/2 rounded bg-tile animate-souffle" />
        <span class="h-32 w-full rounded-xl bg-tile animate-souffle" />
      </div>

      <FicheClient
        v-else-if="fiche"
        :fiche="fiche"
        :historique="historique"
        :chargement-historique="chargementHistorique"
      />

      <p
        v-else
        class="rounded-xl border border-dashed border-line-2 px-3.5 py-3 text-corps text-ink-2"
        role="status"
      >
        {{ t('sejours.clients.choisir_une_fiche') }}
      </p>
    </section>
  </div>
</template>
