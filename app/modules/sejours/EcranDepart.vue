<script setup lang="ts">
/**
 * ★ **`R7` — La note et le départ.** Écran **MAQUETTÉ** : `docs/design/html/R7-note-depart.html`.
 *
 * Valeurs lues, **HTML jamais copié** (porte P-19).
 *
 * # ⚠️ TROIS ÉLÉMENTS DE LA MAQUETTE SONT ABSENTS, JAMAIS GRISÉS (principe VII)
 *
 * | Élément de `R7-note-depart.html` | Cycle qui le doit |
 * |---|---|
 * | « Déjà versé — 100 000 F, à l'arrivée, par Wave » | **CAI**, tranche T2 |
 * | « Il resterait à payer aujourd'hui » | **CAI**, tranche T2 |
 * | Le sous-titre « compte final, encaissement, facture » de l'action finale | **CAI** et **FIS** |
 *
 * Un bouton grisé promet une fonction que le produit n'a pas ; l'exploitant attend une mise à jour
 * qui ne vient pas, puis cesse de croire ce que l'écran lui dit. Ces trois-là sont **retirés**.
 *
 * ★ **Et le retrait du troisième oblige à dire ce qui se passe vraiment.** Le séjour se clôt sur
 * une note **arrêtée et NON RÉGLÉE** — l'écran l'écrit en toutes lettres, avant et après le geste.
 * Laisser « Faire partir le client » seul ferait croire au paiement, et le trou se découvrirait au
 * comptage de caisse, le soir, sans qu'on sache à quel séjour il se rattache.
 *
 * ⚠️ **Ces noms ne figurent QUE dans ce bloc de documentation.** Un commentaire de `<template>`
 * est **rendu dans le DOM** : il atteindrait la page et ferait échouer le contrôle d'absence pour
 * une raison sans rapport avec ce qu'il mesure. Deux fois trouvé au cycle 006.
 *
 * # La fiche de police est rendue ici (opération 16)
 *
 * Elle porte la **même mention obligatoire** que la note — « Document non fiscal — ne tient pas
 * lieu de facture » : c'est un **document opérationnel** au sens de FIS-02, et le principe V
 * l'exige de tous (FR-048).
 *
 * # Le départ est de classe B : le refus hors ligne est IMMÉDIAT
 *
 * Annoncé **avant** le geste, jamais après un échec, et jamais par un grisé silencieux.
 */
import { computed, ref } from 'vue'

import { formaterMontant } from '~/core/format/montant'
import { useEtatReseau } from '~/core/platform'
import type { Permissions } from '~/core/rbac'
import { detient } from '~/core/rbac'
import NoteSejour from './NoteSejour.vue'
import {
  chargerSejour,
  chargerSejours,
  type ContexteAppel,
  type DonneesDepart,
  type SejourOuvert,
  type SejourVue,
} from './donnees'
import { cloreSejour } from './clore-sejour'

const { t, locale } = useI18n()

const props = defineProps<{
  contexte: ContexteAppel
  etablissementId: string
  donnees: DonneesDepart
  permissions: Permissions
  /** Le séjour ouvert d'emblée — vient du **paramètre de requête** de la page, jamais d'une route. */
  sejourInitial: string | null
}>()

/** La permission de clore. **Sans elle l'action est ABSENTE**, jamais grisée (FR-026). */
const PERM_CLORE = 'heb.sejour.clore'

const reseau = useEtatReseau()
const horsLigne = computed(() => reseau.value !== 'connecte')
const peutClore = computed(() => detient(props.permissions, PERM_CLORE))

const sejours = ref<SejourVue[]>(props.donnees.sejours)
const choisi = ref<string | null>(props.sejourInitial)
const detail = ref<SejourOuvert | null>(null)
const chargementDetail = ref(false)
const refus = ref<{ cle: string, valeurs?: Record<string, unknown> } | null>(null)
const clos = ref(false)

const ligneChoisie = computed(() => sejours.value.find((s) => s.sejour.id === choisi.value) ?? null)

/**
 * Le code de la chambre — **résolu depuis l'état des unités**, jamais recopié dans le séjour.
 *
 * `null` quand l'unité n'est plus dans l'état courant : l'écran affiche alors le client sans code
 * de chambre plutôt qu'un identifiant technique, qui ne dit rien à personne au comptoir.
 */
const codeUnite = computed(() => {
  const uniteId = detail.value?.occupation.unite_id
  return uniteId ? (props.donnees.codesUnites[uniteId] ?? null) : null
})

const heureDe = (instant: string | null | undefined): string => {
  if (!instant) return ''
  return new Intl.DateTimeFormat(locale.value, {
    hour: '2-digit',
    minute: '2-digit',
    hour12: false,
  })
    .format(new Date(instant))
    // L'heure garde l'espace ORDINAIRE — la fine insécable est réservée aux montants.
    .replace(':', ' h ')
}

const totalDe = (sejour: SejourVue): string =>
  formaterMontant(sejour.total_mineur, sejour.devise)

/**
 * Ouvre un séjour. **Squelette pendant le chargement** — jamais un écran vide (composant 13).
 *
 * Cet appel est de **lecture** : hors ligne il échoue, et l'écran le dit plutôt que d'afficher
 * une note vide, qui se lirait « ce client n'a rien consommé ».
 */
async function ouvrir(sejourId: string): Promise<void> {
  choisi.value = sejourId
  detail.value = null
  refus.value = null
  clos.value = false
  chargementDetail.value = true
  try {
    detail.value = await chargerSejour(props.contexte, props.etablissementId, sejourId)
  }
  catch {
    refus.value = { cle: 'sejours.depart.note_illisible' }
  }
  finally {
    chargementDetail.value = false
  }
}

if (props.sejourInitial) void ouvrir(props.sejourInitial)

/**
 * ★ **Le départ.** Un seul appel, six écritures dans une transaction.
 *
 * L'ordre **rafraîchir-avant-vider** du cycle 003 s'applique : la liste est rechargée **avant** que
 * l'écran ne se remette en nominal, sinon il existerait un instant où la liste est vide et la
 * confirmation absente — exactement le moment où Adjoua regarde.
 */
async function fairePartir(): Promise<void> {
  if (!choisi.value || chargementDetail.value) return
  refus.value = null

  const resultat = await cloreSejour(
    props.contexte,
    reseau.value,
    props.etablissementId,
    choisi.value,
  )

  if (resultat.issue === 'refus') {
    refus.value = { cle: resultat.cle, valeurs: resultat.valeurs }
    return
  }

  // La note **arrêtée** revient du serveur : c'est elle qui s'affiche, jamais une recomposition
  // locale — le serveur fait foi (principe VI).
  detail.value = resultat.sejour
  clos.value = true

  try {
    sejours.value = await chargerSejours(props.contexte, props.etablissementId)
  }
  catch {
    // Un rafraîchissement manqué n'annule pas une clôture réussie. Faire échouer la confirmation
    // ici ferait croire à Adjoua que le client n'est pas parti, et elle recommencerait.
  }
}
</script>

<template>
  <!-- ⚠️ UNE SEULE RACINE, ET C'EST UN ÉLÉMENT. Voir la note de la page. -->
  <div class="flex flex-col gap-6 lg:flex-row lg:items-start">
    <!-- ═══ COLONNE GAUCHE — les séjours en cours ═══ -->
    <section class="flex flex-col gap-3 lg:w-80 lg:shrink-0">
      <h2 class="font-titre text-titre-m font-semibold text-ink">
        {{ t('sejours.depart.en_cours') }}
      </h2>

      <p
        v-if="!sejours.length"
        class="rounded-xl border border-dashed border-line-2 px-3.5 py-3 text-mini text-ink-2"
        role="status"
      >
        {{ t('sejours.depart.aucun_sejour') }}
      </p>

      <ul class="flex flex-col gap-2">
        <li
          v-for="sejour in sejours"
          :key="sejour.sejour.id"
        >
          <button
            type="button"
            :data-sejour="sejour.sejour.id"
            :aria-pressed="choisi === sejour.sejour.id"
            class="flex w-full flex-col gap-1 rounded-xl border px-3.5 py-3 text-left transition-colors duration-90"
            :class="choisi === sejour.sejour.id
              ? 'border-ocre bg-ocre-soft'
              : 'border-line bg-surf hover:bg-tile'"
            @click="ouvrir(sejour.sejour.id)"
          >
            <span class="font-titre text-corps font-semibold text-ink">
              {{ sejour.client_nom ?? t('sejours.note.client_sans_fiche') }}
            </span>
            <span class="flex items-baseline justify-between gap-2">
              <span class="text-mini text-ink-3">
                {{ t('sejours.depart.personnes_et_fin', {
                  n: sejour.nombre_personnes,
                  heure: heureDe(sejour.fin_prevue),
                }) }}
              </span>
              <span class="font-mono text-mini text-ink-2">{{ totalDe(sejour) }}</span>
            </span>
          </button>
        </li>
      </ul>
    </section>

    <!-- ═══ COLONNE DROITE — la note, puis l'action ═══ -->
    <section class="flex min-w-0 flex-1 flex-col gap-4">
      <!-- Squelette de chargement (composant 13) — jamais un écran vide. -->
      <div
        v-if="chargementDetail"
        class="flex flex-col gap-2.5"
        data-chargement
      >
        <span class="h-6 w-1/2 rounded bg-tile animate-souffle" />
        <span class="h-4 w-1/3 rounded bg-tile animate-souffle" />
        <span class="h-40 w-full rounded-xl bg-tile animate-souffle" />
      </div>

      <p
        v-else-if="!ligneChoisie"
        class="rounded-xl border border-dashed border-line-2 px-3.5 py-3 text-corps text-ink-2"
        role="status"
      >
        {{ t('sejours.depart.choisir_un_sejour') }}
      </p>

      <template v-else-if="detail">
        <NoteSejour
          :note="detail.note"
          :sejour="ligneChoisie"
          :code-unite="codeUnite"
        />

        <!-- ═══ LA FICHE DE POLICE — opération 16, MÊME mention obligatoire ═══ -->
        <section
          class="flex flex-col gap-2 rounded-xl border border-line bg-surf px-4 py-3.5"
          data-fiche-police
        >
          <span class="text-etiquette uppercase text-ink-3">
            {{ t('sejours.depart.fiche_police') }}
          </span>
          <p class="flex items-baseline gap-3">
            <span class="font-mono text-titre-s font-semibold text-ink">
              {{ t('sejours.depart.fiche_numero', { numero: detail.fiche_police.numero }) }}
            </span>
            <span class="text-mini text-ink-3">
              {{ detail.fiche_police.complete
                ? t('sejours.depart.fiche_complete')
                : t('sejours.depart.fiche_incomplete') }}
            </span>
          </p>
          <span class="text-etiquette font-semibold uppercase text-ocre">
            {{ t('documents.mention_non_fiscale') }}
          </span>
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

        <!-- ═══ APRÈS LE DÉPART — la note est arrêtée, et NON RÉGLÉE ═══ -->
        <section
          v-if="clos"
          class="flex flex-col gap-2 rounded-2xl border border-line bg-surf px-5 py-5"
          data-etat="clos"
          role="status"
        >
          <p class="font-titre text-titre-m font-semibold text-ink">
            {{ t('sejours.depart.c_est_fait') }}
          </p>
          <!--
            ★ Dit en toutes lettres : la note est ARRÊTÉE et NON RÉGLÉE. Sans cette phrase, l'écran
            laisserait croire au paiement, et le trou se découvrirait au comptage de caisse — sans
            qu'on sache à quel séjour il se rattache.
          -->
          <p
            class="text-corps text-ink-2"
            data-non-reglee
          >
            {{ t('sejours.depart.note_non_reglee') }}
          </p>
        </section>

        <!-- ═══ L'ACTION FINALE ═══ -->
        <template v-else>
          <!--
            ★ Hors ligne : le refus est IMMÉDIAT et EXPLICITE, annoncé AVANT le geste (principe
            VI). Le départ fige un constat de taxe : deux terminaux vidant leur file en figeraient
            deux sur les mêmes faits.
          -->
          <p
            v-if="horsLigne"
            class="rounded-xl border border-dashed border-line-2 px-3.5 py-2.5 text-mini text-ink-2"
            role="status"
            data-hors-ligne
          >
            {{ t('sejours.depart.hors_ligne') }}
          </p>

          <button
            v-else-if="peutClore"
            type="button"
            data-action="faire-partir"
            class="flex h-18 w-full flex-col items-center justify-center gap-1 rounded-xl bg-prim font-titre text-titre-m font-semibold text-prim-ink shadow-bouton-grand transition-[transform,box-shadow] duration-90 ease-entree active:translate-y-0.75 active:shadow-none"
            @click="fairePartir"
          >
            {{ t('sejours.depart.faire_partir') }}
          </button>

          <!--
            La phrase de ce que le départ fait — et de ce qu'il ne fait PAS. Elle est affichée
            AVANT le geste, pas après : c'est là qu'elle change une décision.
          -->
          <p class="text-mini text-ink-3">
            {{ t('sejours.depart.ce_que_fait_le_depart') }}
          </p>
        </template>
      </template>
    </section>
  </div>
</template>
