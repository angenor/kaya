<script setup lang="ts">
/**
 * ★ **`R4` — Le passage.** Écran **MAQUETTÉ**, zone de vitesse.
 *
 * Référence : `docs/design/html/R4-passage.html` et ses **quatre états** — `-connu`,
 * `-enregistre`, `-hors-ligne`, `-complet`. Valeurs lues, **HTML jamais copié** (porte P-19).
 *
 * # ★ Cet écran ne se compose jamais
 *
 * `docs/Kaya_Design.md` §1 est formel : *un écran de zone de vitesse ne se compose jamais*. Yao
 * est debout, pressé, un client en face. `R4` porte une intention dessinée qu'un assemblage depuis
 * la bibliothèque ne retrouverait pas — les tailles de la durée et de l'heure de fin, la place du
 * prix sur le bouton.
 *
 * # Le budget, qui est le vrai sujet
 *
 * | Contrainte | Valeur | Source |
 * |---|---|---|
 * | Interactions obligatoires | **exactement 2** | SC-001, FR-022 |
 * | Champs de saisie libre obligatoires | **0** | FR-023 |
 * | Appels réseau bloquants | **au plus 1** | FR-031 |
 *
 * `app/tests/budget-gestes.spec.ts` les **contraint** — il est écrit dans la même tâche que cet
 * écran, précisément pour qu'il ne se contente pas de *constater* le nombre de gestes.
 *
 * # Les cinq états, et ce qui les distingue
 *
 * | État | Ce qu'il montre |
 * |---|---|
 * | **nominal** | Les durées, les chambres. Rien d'autre |
 * | **client reconnu** | « M. Bakayoko — 7ᵉ passage », avec sa sortie « Ce n'est pas lui » |
 * | **enregistré** | « C'est fait », l'heure de fin **à redire au client**, « Client suivant » |
 * | **hors ligne** | Durées et prix **lisibles** avec leur fraîcheur ; attribution refusée **explicitement** |
 * | **complet** | Le nombre de chambres prises, et **ce qui se libère avec l'heure**, la plus proche en tête |
 *
 * # ⚠️ Quatre éléments de la maquette sont ABSENTS, jamais grisés (principe VII)
 *
 * | Élément | Cycle qui le doit |
 * |---|---|
 * | « Scanner la pièce » | **SEJ-06** — OCR, P1, tranche T4 |
 * | « Garder la 101 pour ce client » | **RSV** — maintien d'unité, tranche T4 |
 * | « Imprimer le reçu » | **IMP**, tranche T2 |
 * | « encaissé en espèces » | **CAI**, tranche T2 |
 *
 * Un bouton grisé promet une fonction que le produit n'a pas ; l'exploitant attend une mise à jour
 * qui ne vient pas, puis cesse de croire ce que l'écran lui dit.
 *
 * ⚠️ **Ces quatre noms ne figurent QUE dans ce bloc de documentation, jamais dans un commentaire
 * de `<template>`.** Un commentaire de gabarit est **rendu dans le DOM** : il atteindrait la page,
 * lisible par « afficher la source », et surtout il ferait échouer le contrôle de
 * `budget-gestes.spec.ts` qui vérifie leur absence sur le HTML rendu — pour une raison sans
 * rapport avec ce qu'il mesure. C'est le test qui l'a trouvé.
 */
import { computed, ref } from 'vue'

import { useEtatReseau } from '~/core/platform/reseau'
import type { Permissions } from '~/core/rbac'
import { detient } from '~/core/rbac'
import ChoixDuree from './ChoixDuree.vue'
import GrilleUnites from './GrilleUnites.vue'
import {
  paliersAffichables,
  rechargerEtatDesUnites,
  type ClientResume,
  type ContexteAppel,
  type DonneesPassage,
  type EtatUniteVue,
  type PalierAffichable,
  type SejourOuvert,
} from './donnees'
import { ouvrirSejour } from './ouvrir-sejour'

const { t, locale } = useI18n()

const props = defineProps<{
  contexte: ContexteAppel
  etablissementId: string
  donnees: DonneesPassage
  permissions: Permissions
  /** Le client reconnu au comptoir — état `-connu` de la maquette. `null` en nominal. */
  clientReconnu: ClientResume | null
  /** Nombre de séjours déjà faits par ce client — « 7ᵉ passage ». */
  passagesDuClient: number
}>()

const emit = defineEmits<{ oublierClient: [] }>()

/** La permission d'ouvrir. **Sans elle l'action est ABSENTE**, jamais grisée (FR-026). */
const PERM_OUVRIR = 'heb.sejour.ouvrir'

const reseau = useEtatReseau()
const horsLigne = computed(() => reseau.value !== 'connecte')
const peutOuvrir = computed(() => detient(props.permissions, PERM_OUVRIR))

/** L'état local de l'écran — **`donnees` reste la source, jamais recopiée**. */
const etat = ref<DonneesPassage>(props.donnees)
const palierChoisi = ref<PalierAffichable | null>(null)
const uniteEnCours = ref<string | null>(null)
const enregistre = ref<SejourOuvert | null>(null)
const refus = ref<{ cle: string, valeurs?: Record<string, unknown> } | null>(null)

/**
 * La catégorie servie par l'écran de passage.
 *
 * ⚠️ **La première qui porte une formule de passage, et c'est une simplification assumée.** `R4`
 * est mono-catégorie par construction : la maquette montre une seule grille et un seul barème. Un
 * établissement à deux catégories de passage aura besoin d'un sélecteur — **et d'une maquette**,
 * que ce cycle n'a pas. L'inventer ici serait improviser un motif dans un écran, ce que la
 * bibliothèque interdit.
 */
const categoriePassage = computed(() => {
  const formule = etat.value.formules.find((f) => f.famille === 'PASSAGE')
  return formule?.categorie_id ?? null
})

/**
 * ★ **Les chambres de la catégorie servie — les autres n'ont rien à faire ici.**
 *
 * ⚠️ **Défaut réel, trouvé par la porte de temps machine au cycle 006.** La grille rendait
 * **toutes** les unités de l'établissement alors que l'écran n'applique **qu'une** formule de
 * passage : toucher une chambre d'une autre catégorie produisait
 * `formule_hors_categorie` — « Cette formule ne s'applique pas à cette chambre » — un refus que Yao
 * subissait **après** le geste, devant le client, sur une chambre que l'écran venait de lui
 * présenter comme libre.
 *
 * Aucun test unitaire ne pouvait le voir : ils fournissent une seule catégorie. Il fallait un parc
 * réel à six catégories, c'est-à-dire les seeds.
 */
const unitesDeLaCategorie = computed(() =>
  etat.value.etatDesUnites.unites.filter((u) => u.categorie_id === categoriePassage.value),
)

const paliers = computed<PalierAffichable[]>(() => {
  if (!categoriePassage.value) return []
  return paliersAffichables(
    etat.value.formules,
    categoriePassage.value,
    etat.value.etatDesUnites.instant_autorite,
  )
})

const clePalierChoisi = computed(() =>
  palierChoisi.value ? `${palierChoisi.value.formuleId}:${palierChoisi.value.dureeMinutes}` : null,
)

/**
 * ★ **L'état `-complet`** : plus aucune chambre attribuable.
 *
 * La maquette ne se contente pas de dire « complet » — elle montre **ce qui se libère, avec
 * l'heure, la plus proche en tête**. C'est la différence entre un écran qui bloque et un écran qui
 * répond : Yao peut dire au client « dans vingt minutes », au lieu de « je n'ai rien ».
 */
const complet = computed(() =>
  unitesDeLaCategorie.value.length > 0
  && !unitesDeLaCategorie.value.some(
    (u) => u.etat === 'libre' && u.statut_menage !== 'a_nettoyer',
  ),
)

const prochainesLiberations = computed(() =>
  unitesDeLaCategorie.value
    .filter((u) => u.etat !== 'libre')
    .map((u) => ({
      unite: u,
      instant: u.fin_prevue ?? u.disponible_a ?? null,
    }))
    .filter((l): l is { unite: EtatUniteVue, instant: string } => l.instant !== null)
    .sort((a, b) => a.instant.localeCompare(b.instant))
    .slice(0, 3),
)

const chambresPrises = computed(() =>
  unitesDeLaCategorie.value.filter((u) => u.etat === 'occupee').length,
)

const heureDe = (instant: string): string =>
  new Intl.DateTimeFormat(locale.value, { hour: '2-digit', minute: '2-digit', hour12: false })
    .format(new Date(instant))
    .replace(':', ' h ')

/** Premier geste — la durée. **Aucun appel réseau** : tout est déjà chargé. */
function choisirDuree(palier: PalierAffichable): void {
  palierChoisi.value = palier
  refus.value = null
}

/**
 * ★ **Second geste — la chambre, et c'est la confirmation.**
 *
 * Pas de troisième tap : le tap sur la chambre **est** l'attribution. Ajouter un bouton
 * « Confirmer » ferait trois gestes là où le budget en autorise deux.
 */
async function attribuer(unite: EtatUniteVue): Promise<void> {
  if (!palierChoisi.value || uniteEnCours.value) return
  refus.value = null
  uniteEnCours.value = unite.unite_id

  const debut = new Date(etat.value.etatDesUnites.instant_autorite)
  const resultat = await ouvrirSejour(props.contexte, reseau.value, {
    etablissementId: props.etablissementId,
    uniteId: unite.unite_id,
    formuleId: palierChoisi.value.formuleId,
    debutClient: debut,
    finClient: palierChoisi.value.finPrevue,
    ...(props.clientReconnu ? { clientId: props.clientReconnu.id } : {}),
  })

  uniteEnCours.value = null

  if (resultat.issue === 'refus') {
    refus.value = { cle: resultat.cle, valeurs: resultat.valeurs }
    // **Rafraîchir AVANT de vider** — ordre imposé par le cycle 003. Un refus vient souvent d'une
    // chambre prise entre-temps : montrer l'état à jour explique le refus au lieu de le subir.
    await rafraichir()
    return
  }

  // **Rafraîchir AVANT de vider.** L'inverse laisserait un instant où l'écran est vide et la
  // confirmation absente — exactement le moment où Yao regarde.
  enregistre.value = resultat.sejour
  await rafraichir()
  palierChoisi.value = null
}

/** Recharge **le seul état des chambres** — la liste vient du serveur, jamais reconstruite. */
async function rafraichir(): Promise<void> {
  try {
    etat.value = {
      ...etat.value,
      etatDesUnites: await rechargerEtatDesUnites(props.contexte, props.etablissementId),
    }
  } catch {
    // Un rafraîchissement manqué n'annule pas une écriture réussie : l'écran garde son état
    // précédent avec sa fraîcheur. Faire échouer la confirmation ici ferait croire à Yao que le
    // séjour n'a pas été enregistré, et il recommencerait.
  }
}

/** « Client suivant » — remet l'écran en nominal, sans rechargement. */
function clientSuivant(): void {
  enregistre.value = null
  palierChoisi.value = null
  refus.value = null
  emit('oublierClient')
}
</script>

<template>
  <!-- ⚠️ UNE SEULE RACINE, ET C'EST UN ÉLÉMENT. Voir la note de la page. -->
  <div class="flex flex-col gap-6">
    <!--
      ═══ ÉTAT « ENREGISTRÉ » — `R4-passage-enregistre.html` ═══
      L'heure de fin est ce que Yao REDIT AU CLIENT : elle est la plus grosse chose de l'écran.
    -->
    <section
      v-if="enregistre"
      class="flex flex-col gap-4 rounded-2xl border border-line bg-surf px-5 py-6"
      data-etat="enregistre"
      role="status"
    >
      <p class="font-titre text-titre-l font-semibold text-ink">
        {{ t('sejours.passage.c_est_fait') }}
      </p>
      <p class="flex flex-col gap-1">
        <span class="text-mini uppercase text-ink-3">
          {{ t('sejours.passage.a_redire_au_client') }}
        </span>
        <span class="font-mono text-titre-xl font-semibold text-ink">
          {{ heureDe(enregistre.occupation.fin_client) }}
        </span>
      </p>
      <p class="text-mini text-ink-3">
        {{ t('sejours.passage.fiche_a_completer', { numero: enregistre.fiche_police.numero }) }}
      </p>

      <!-- Deux éléments d'autres cycles sont absents ici — voir la note de tête. -->
      <button
        type="button"
        data-action="client-suivant"
        class="self-start rounded-xl bg-ocre px-5 py-3 font-titre text-corps font-semibold text-ocre-ink transition-colors duration-90 hover:bg-ocre-fort"
        @click="clientSuivant"
      >
        {{ t('sejours.passage.client_suivant') }}
      </button>
    </section>

    <!-- ═══ ÉTAT NOMINAL, « CONNU », « HORS LIGNE » et « COMPLET » ═══ -->
    <template v-else>
      <!--
        ═══ « CLIENT RECONNU » — `R4-passage-connu.html` ═══
        La sortie « Ce n'est pas lui » n'est pas décorative : sans elle, une reconnaissance fausse
        rattacherait le séjour à la mauvaise fiche, et l'erreur se verrait au départ — trop tard.
      -->
      <section
        v-if="clientReconnu"
        class="flex items-center justify-between gap-4 rounded-2xl bg-tile px-4 py-3"
        data-etat="client-connu"
      >
        <p class="flex flex-col gap-0.5">
          <span class="font-titre text-corps font-semibold text-ink">
            {{ clientReconnu.nom }}
          </span>
          <span class="text-mini text-ink-3">
            {{ t('sejours.passage.enieme_passage', { n: passagesDuClient }) }}
          </span>
        </p>
        <button
          type="button"
          data-action="ce-n-est-pas-lui"
          class="rounded-lg border border-line px-3 py-2 text-mini text-ink-2 transition-colors duration-90 hover:bg-surf"
          @click="emit('oublierClient')"
        >
          {{ t('sejours.passage.ce_n_est_pas_lui') }}
        </button>
      </section>

      <!--
        ═══ « COMPLET » — `R4-passage-complet.html` ═══
        Ce qui se libère, AVEC L'HEURE, la plus proche en tête. Un écran qui répond, pas un écran
        qui bloque. Un élément d'un autre cycle est absent ici — voir la note de tête.
      -->
      <section
        v-if="complet"
        class="flex flex-col gap-3 rounded-2xl border border-line-2 bg-surf px-5 py-5"
        data-etat="complet"
        role="status"
      >
        <p class="font-titre text-titre-m font-semibold text-ink">
          {{ t('sejours.passage.complet', { n: chambresPrises }) }}
        </p>
        <ul
          v-if="prochainesLiberations.length"
          class="flex flex-col gap-1.5"
        >
          <li
            v-for="liberation in prochainesLiberations"
            :key="liberation.unite.unite_id"
            class="flex items-baseline gap-3"
          >
            <span class="font-mono text-corps font-semibold text-ink">
              {{ liberation.unite.code }}
            </span>
            <span class="text-mini text-ink-3">
              {{ t('sejours.passage.se_libere_a', { heure: heureDe(liberation.instant) }) }}
            </span>
          </li>
        </ul>
      </section>

      <!-- Le refus, en LANGUE UTILISATEUR — jamais un code, jamais un message de diagnostic. -->
      <p
        v-if="refus"
        class="rounded-xl border border-line-2 bg-tile px-3.5 py-3 text-corps text-ink"
        role="alert"
        data-refus
      >
        {{ t(refus.cle, refus.valeurs ?? {}) }}
      </p>

      <ChoixDuree
        :paliers="paliers"
        :choisi="clePalierChoisi"
        :hors-ligne="horsLigne"
        @choisir="choisirDuree"
      />

      <!--
        ★ La grille n'existe QUE si la permission d'ouvrir est là — **absente**, jamais grisée
        (FR-026). Le contrôle porte sur le HTML rendu, pas sur un attribut `disabled` : un attribut
        se retire depuis la console du navigateur.
      -->
      <GrilleUnites
        v-if="peutOuvrir"
        :unites="unitesDeLaCategorie"
        :choisie="null"
        :hors-ligne="horsLigne"
        :en-cours="uniteEnCours"
        @attribuer="attribuer"
      />
    </template>
  </div>
</template>
