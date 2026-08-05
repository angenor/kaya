<script setup lang="ts">
/**
 * ★ **`R3` — Arrivée.** Écran **DÉRIVÉ**, jamais maquetté.
 *
 * Référence : `docs/design/derivation.md`, ligne `R3` → motif **`R4`**. *« Parcours long : plus de
 * champs, même grammaire. »* C'est opposable : un écran hors de la matrice de dérivation ou du
 * répertoire des maquettes **ne se code pas** (porte P-19). Aucun motif neuf n'est inventé ici —
 * les filets, les jetons, la grille des chambres et le tap final viennent tous de `R4`.
 *
 * # ★ LA GRAMMAIRE DE `R4` EST CONSERVÉE : le tap reste le geste
 *
 * Les champs s'ajoutent, mais ils **ne deviennent pas un formulaire** : il n'y a pas de bouton
 * « Enregistrer » en bas de page. Le dernier geste est le **tap sur la chambre**, exactement comme
 * au passage — c'est lui qui attribue et qui ouvre le séjour, en un seul appel.
 *
 * La différence avec `R4` n'est pas le nombre de gestes, c'est le nombre de **décisions
 * préalables** : combien de nuits, à quelle heure, avec qui. Chacune a une valeur par défaut, et
 * la valeur par défaut est **applicable telle quelle**.
 *
 * # ★ CLIENT CONNU → RIEN N'EST À RETAPER (FR-035)
 *
 * Une fiche retenue affiche son nom et son téléphone **en lecture**, et la requête d'ouverture ne
 * porte **aucun champ d'identité** : seulement `client_id`. Ce n'est pas une commodité, c'est ce
 * qui empêche l'écran d'écraser la fiche par une copie périmée à chaque arrivée.
 * `backend/tests/sejour_arrivee.rs` l'éprouve sur le corps réellement envoyé.
 *
 * # Les heures standard viennent du PARAMÈTRE, jamais d'une constante
 *
 * `heure_arrivee_standard` et `heure_depart_standard` sont portées par la **formule** (HEB-03,
 * migration `0024`). Elles sont appliquées d'office **et modifiables** : un client qui arrive à
 * 22 h ne doit pas obliger Yao à sortir de l'écran. Écrire « 14 h / 12 h » en dur serait une règle
 * métier en dur — porte **P-12** — et ferait mentir l'écran au premier établissement qui pratique
 * autrement.
 *
 * # Ce qui est ABSENT, et n'est pas grisé (principe VII)
 *
 * | Élément | Cycle qui le doit |
 * |---|---|
 * | Le numéro de pièce de chaque accompagnant | **SEJ-06**, et la loi ne l'exige pas d'eux |
 * | L'encaissement d'un acompte | **CAI**, tranche T2 |
 * | La réservation à l'avance | **RSV**, tranche T4 |
 *
 * ⚠️ **Ces noms ne figurent QUE dans ce bloc.** Un commentaire de `<template>` est **rendu dans le
 * DOM** : il atteindrait la page et ferait échouer les contrôles d'absence pour une raison sans
 * rapport avec ce qu'ils mesurent. C'est un test qui l'a trouvé au cycle 006.
 */
import { computed, ref, watch } from 'vue'

import ChampSaisie from '~/core/design-system/ChampSaisie.vue'
import { formaterMontant } from '~/core/format/montant'
import { useEtatReseau } from '~/core/platform/reseau'
import type { Permissions } from '~/core/rbac'
import { detient } from '~/core/rbac'
import GrilleUnites from './GrilleUnites.vue'
import ListeAccompagnants from './ListeAccompagnants.vue'
import {
  rechargerEtatDesUnites,
  type AccompagnantSaisi,
  type ClientResume,
  type ContexteAppel,
  type DonneesArrivee,
  type EtatUniteVue,
  type FormuleVue,
  type SejourOuvert,
} from './donnees'
import { ouvrirSejour } from './ouvrir-sejour'

const { t, locale } = useI18n()

const props = defineProps<{
  contexte: ContexteAppel
  etablissementId: string
  donnees: DonneesArrivee
  permissions: Permissions
  /** La fiche retenue — **elle arrive par la recherche**, jamais par une reconnaissance devinée. */
  clientRetenu: ClientResume | null
}>()

const emit = defineEmits<{ oublierClient: [], retenirClient: [client: ClientResume] }>()

/** La permission d'ouvrir. **Sans elle la grille est ABSENTE**, jamais grisée (FR-026). */
const PERM_OUVRIR = 'heb.sejour.ouvrir'

/** Les nuitées proposées d'un tap. Au-delà, le champ de date de départ prend le relais. */
const NUITS_PROPOSEES = [1, 2, 3, 4, 5, 6, 7]

const reseau = useEtatReseau()
const horsLigne = computed(() => reseau.value !== 'connecte')
const peutOuvrir = computed(() => detient(props.permissions, PERM_OUVRIR))

const etat = ref<DonneesArrivee>(props.donnees)
const nuits = ref(1)
const accompagnants = ref<AccompagnantSaisi[]>([])
const uniteEnCours = ref<string | null>(null)
const enregistre = ref<SejourOuvert | null>(null)
const refus = ref<{ cle: string, valeurs?: Record<string, unknown> } | null>(null)
const rechercheClient = ref('')
const resultats = ref<ClientResume[]>([])
const rechercheTronquee = ref(false)
const rechercheEnCours = ref(false)

/**
 * La formule de nuitée servie par l'écran.
 *
 * ⚠️ **La première de famille `NUITEE`, et c'est la même simplification assumée qu'à `R4`.**
 * L'écran est mono-catégorie par construction : la matrice de dérivation ne donne à `R3` qu'un
 * parcours plus long, pas un sélecteur de catégorie — qui demanderait un motif, donc une maquette,
 * que ce cycle n'a pas.
 */
const formuleNuitee = computed<FormuleVue | null>(
  () => etat.value.formules.find((f) => f.famille === 'NUITEE') ?? null,
)

const categorieNuitee = computed(() => formuleNuitee.value?.categorie_id ?? null)

/** La capacité d'accueil — **paramètre de la catégorie**, jamais une constante. */
const capaciteAccueil = computed(() => {
  const categorie = etat.value.categories.find((c) => c.id === categorieNuitee.value)
  return categorie?.capacite_accueil ?? null
})

/** Les chambres de la catégorie servie — les autres n'ont rien à faire sur cet écran. */
const unitesDeLaCategorie = computed(() =>
  etat.value.etatDesUnites.unites.filter((u) => u.categorie_id === categorieNuitee.value),
)

// =================================================================================================
//  La recherche de fiche — trois formes, une seule entrée
// =================================================================================================

/**
 * Cherche une fiche cliente. **L'opérateur ne choisit pas un mode** : le serveur déduit s'il
 * s'agit d'un nom, d'un téléphone ou d'un numéro de pièce.
 *
 * ⚠️ **Une lecture, donc de classe A hors ligne** — mais la recherche interroge le serveur : hors
 * ligne elle échoue, et l'écran le dit plutôt que de rendre une liste vide, qui se lirait
 * « ce client n'existe pas ».
 */
async function chercher(): Promise<void> {
  const terme = rechercheClient.value.trim()
  if (!terme || rechercheEnCours.value) return

  if (horsLigne.value) {
    refus.value = { cle: 'sejours.arrivee.recherche_hors_ligne' }
    return
  }

  rechercheEnCours.value = true
  refus.value = null
  try {
    const { chercherClients } = await import('./donnees')
    const resultat = await chercherClients(props.contexte, terme)
    resultats.value = resultat.clients
    // ★ `tronque` n'est pas décoratif : une liste silencieusement coupée est un mensonge sur un
    // écran de comptoir — Yao conclurait que la fiche n'existe pas et en créerait une seconde.
    rechercheTronquee.value = resultat.tronque
  }
  catch {
    refus.value = { cle: 'sejours.arrivee.recherche_impossible' }
  }
  finally {
    rechercheEnCours.value = false
  }
}

function retenir(client: ClientResume): void {
  resultats.value = []
  rechercheTronquee.value = false
  emit('retenirClient', client)
}

// =================================================================================================
//  Les heures — appliquées d'office, MODIFIABLES, et jamais écrites en dur
// =================================================================================================

/** `HH:MM` de repli **quand le paramètre est absent**, et l'écran le DIT. */
const heureArrivee = ref('')
const heureDepart = ref('')

watch(
  formuleNuitee,
  (formule) => {
    heureArrivee.value = (formule?.heure_arrivee_standard ?? '').slice(0, 5)
    heureDepart.value = (formule?.heure_depart_standard ?? '').slice(0, 5)
  },
  { immediate: true },
)

/**
 * L'établissement n'a pas réglé ses heures standard — **l'écran le dit, il n'invente pas**.
 *
 * Poser « 14 h / 12 h » par défaut serait un paramètre métier en dur (porte P-12) déguisé en
 * commodité : l'exploitant croirait avoir réglé ce qu'il n'a pas réglé, et les durées facturées
 * seraient fausses sans que rien ne le signale.
 */
const heuresNonReglees = computed(
  () => !formuleNuitee.value?.heure_arrivee_standard || !formuleNuitee.value?.heure_depart_standard,
)

/** `HH:MM` valide — le seul contrôle de forme, fait au champ. */
const FORME_HEURE = /^([01]\d|2[0-3]):[0-5]\d$/

const erreurHeureArrivee = computed(() =>
  heureArrivee.value && !FORME_HEURE.test(heureArrivee.value)
    ? 'sejours.arrivee.heure_invalide'
    : null,
)
const erreurHeureDepart = computed(() =>
  heureDepart.value && !FORME_HEURE.test(heureDepart.value)
    ? 'sejours.arrivee.heure_invalide'
    : null,
)

/**
 * Compose un instant depuis **le jour de l'instant d'autorité** et une heure murale.
 *
 * ★ **Le jour vient du serveur, l'heure de l'opérateur.** Un terminal mal réglé d'un jour
 * afficherait — et enregistrerait — une arrivée la veille. C'est la même discipline que la porte
 * P-23 impose au backend, appliquée à ce que l'écran compose.
 */
function instantDuJour(heureMurale: string, joursApres: number): Date | null {
  if (!FORME_HEURE.test(heureMurale)) return null
  const [h, m] = heureMurale.split(':').map(Number)
  const jour = new Date(etat.value.etatDesUnites.instant_autorite)
  const compose = new Date(jour)
  compose.setDate(jour.getDate() + joursApres)
  compose.setHours(h as number, m as number, 0, 0)
  return compose
}

const debutPrevu = computed(() => instantDuJour(heureArrivee.value, 0))
const finPrevue = computed(() => instantDuJour(heureDepart.value, nuits.value))

/** Les deux heures sont-elles utilisables ? Sans elles, aucune attribution n'est possible. */
const intervalleUtilisable = computed(
  () => debutPrevu.value !== null && finPrevue.value !== null
    && finPrevue.value.getTime() > debutPrevu.value.getTime(),
)

const dateDe = (instant: Date): string =>
  new Intl.DateTimeFormat(locale.value, { weekday: 'long', day: 'numeric', month: 'long' })
    .format(instant)

const heureDe = (instant: string | Date): string =>
  new Intl.DateTimeFormat(locale.value, { hour: '2-digit', minute: '2-digit', hour12: false })
    .format(typeof instant === 'string' ? new Date(instant) : instant)
    // L'heure garde l'espace ORDINAIRE — la fine insécable est réservée aux montants.
    .replace(':', ' h ')

/** Le prix des nuitées, **entier d'unité mineure × nombre de nuits**. */
const totalPrevu = computed(() => {
  const formule = formuleNuitee.value
  if (!formule) return null
  return {
    montant: formule.prix_mineur * nuits.value,
    devise: formule.devise,
  }
})

const totalFormate = computed(() =>
  totalPrevu.value ? formaterMontant(totalPrevu.value.montant, totalPrevu.value.devise) : '',
)

// =================================================================================================
//  Le geste final — le tap sur la chambre
// =================================================================================================

/**
 * ★ **Le tap sur la chambre EST l'ouverture.** Pas de bouton « Enregistrer ».
 *
 * Un seul appel réseau bloquant : les cinq écritures sont faites par le serveur dans une même
 * transaction, accompagnants compris. Les envoyer ensuite par un second appel ferait, à la
 * première coupure, une **fiche de police qui sous-déclare**.
 */
async function attribuer(unite: EtatUniteVue): Promise<void> {
  if (!intervalleUtilisable.value || !formuleNuitee.value || uniteEnCours.value) return
  refus.value = null
  uniteEnCours.value = unite.unite_id

  const resultat = await ouvrirSejour(props.contexte, reseau.value, {
    etablissementId: props.etablissementId,
    uniteId: unite.unite_id,
    formuleId: formuleNuitee.value.id,
    debutClient: debutPrevu.value as Date,
    finClient: finPrevue.value as Date,
    ...(props.clientRetenu ? { clientId: props.clientRetenu.id } : {}),
    accompagnants: accompagnants.value.map((a) => ({ nom: a.nom })),
  })

  uniteEnCours.value = null

  if (resultat.issue === 'refus') {
    refus.value = { cle: resultat.cle, valeurs: resultat.valeurs }
    // **Rafraîchir AVANT de vider** — un refus vient souvent d'une chambre prise entre-temps :
    // montrer l'état à jour explique le refus au lieu de le subir.
    await rafraichir()
    return
  }

  enregistre.value = resultat.sejour
  await rafraichir()
}

async function rafraichir(): Promise<void> {
  try {
    etat.value = {
      ...etat.value,
      etatDesUnites: await rechargerEtatDesUnites(props.contexte, props.etablissementId),
    }
  }
  catch {
    // Un rafraîchissement manqué n'annule pas une écriture réussie : faire échouer la confirmation
    // ici ferait croire à Yao que le séjour n'a pas été enregistré, et il recommencerait.
  }
}

/** « Client suivant » — remet l'écran en nominal, sans rechargement. */
function clientSuivant(): void {
  enregistre.value = null
  refus.value = null
  accompagnants.value = []
  nuits.value = 1
  rechercheClient.value = ''
  emit('oublierClient')
}
</script>

<template>
  <!-- ⚠️ UNE SEULE RACINE, ET C'EST UN ÉLÉMENT. Voir la note de la page. -->
  <div class="flex flex-col gap-6">
    <!-- ═══ ÉTAT « ENREGISTRÉ » — la grammaire de `R4`, mot pour mot ═══ -->
    <section
      v-if="enregistre"
      class="flex flex-col gap-4 rounded-2xl border border-line bg-surf px-5 py-6"
      data-etat="enregistre"
      role="status"
    >
      <p class="font-titre text-titre-l font-semibold text-ink">
        {{ t('sejours.arrivee.c_est_fait') }}
      </p>
      <p class="flex flex-col gap-1">
        <span class="text-mini uppercase text-ink-3">
          {{ t('sejours.arrivee.depart_prevu') }}
        </span>
        <span class="font-mono text-titre-xl font-semibold text-ink">
          {{ heureDe(enregistre.occupation.fin_client) }}
        </span>
      </p>
      <p class="text-mini text-ink-3">
        {{ t('sejours.arrivee.fiche_numero', { numero: enregistre.fiche_police.numero }) }}
      </p>
      <button
        type="button"
        data-action="client-suivant"
        class="self-start rounded-xl bg-ocre px-5 py-3 font-titre text-corps font-semibold text-ocre-ink transition-colors duration-90 hover:bg-ocre-fort"
        @click="clientSuivant"
      >
        {{ t('sejours.arrivee.client_suivant') }}
      </button>
    </section>

    <template v-else>
      <!-- ═══ LE CLIENT ═══ -->
      <section
        class="flex flex-col gap-3"
        data-bloc="client"
      >
        <h2 class="font-titre text-titre-m font-semibold text-ink">
          {{ t('sejours.arrivee.le_client') }}
        </h2>

        <!--
          ★ Fiche retenue : NOM ET TÉLÉPHONE EN LECTURE. Rien n'est à retaper (FR-035), et la
          requête d'ouverture ne portera que `client_id`.
        -->
        <div
          v-if="clientRetenu"
          class="flex items-center justify-between gap-4 rounded-2xl bg-tile px-4 py-3"
          data-etat="client-retenu"
        >
          <p class="flex flex-col gap-0.5">
            <span class="font-titre text-corps font-semibold text-ink">
              {{ clientRetenu.nom }}
            </span>
            <span
              v-if="clientRetenu.telephone"
              class="font-mono text-mini text-ink-3"
            >
              {{ clientRetenu.telephone }}
            </span>
          </p>
          <button
            type="button"
            data-action="ce-n-est-pas-lui"
            class="rounded-lg border border-line px-3 py-2 text-mini text-ink-2 transition-colors duration-90 hover:bg-surf"
            @click="emit('oublierClient')"
          >
            {{ t('sejours.arrivee.ce_n_est_pas_lui') }}
          </button>
        </div>

        <div
          v-else
          class="flex flex-col gap-2"
        >
          <div class="flex items-end gap-2">
            <div class="flex-1">
              <ChampSaisie
                v-model="rechercheClient"
                etiquette-cle="sejours.arrivee.chercher_client"
                aide-cle="sejours.arrivee.chercher_client_aide"
                placeholder-cle="sejours.arrivee.chercher_client_invite"
                taille="comptoir"
                @keyup.enter="chercher"
              />
            </div>
            <button
              type="button"
              data-action="chercher-client"
              class="h-12 rounded-xl border border-line-2 px-4 font-titre text-corps font-semibold text-ink transition-colors duration-90 hover:bg-tile"
              @click="chercher"
            >
              {{ t('sejours.arrivee.chercher') }}
            </button>
          </div>

          <ul
            v-if="resultats.length"
            class="flex flex-col gap-1.5"
          >
            <li
              v-for="client in resultats"
              :key="client.id"
            >
              <button
                type="button"
                :data-client="client.id"
                class="flex w-full items-baseline justify-between gap-3 rounded-xl border border-line bg-surf px-3.5 py-3 text-left transition-colors duration-90 hover:bg-tile"
                @click="retenir(client)"
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
            ★ La troncature se DIT. Une liste silencieusement coupée est un mensonge sur un écran
            de comptoir : Yao conclurait que la fiche n'existe pas et en créerait une seconde.
          -->
          <p
            v-if="rechercheTronquee"
            class="text-mini text-ink-3"
            data-troncature
          >
            {{ t('sejours.arrivee.resultats_tronques') }}
          </p>

          <!--
            Une arrivée sans fiche reste possible : la pièce vient après la clé (FR-023). L'écran
            le dit plutôt que de bloquer — un blocage ici renverrait Yao au cahier papier.
          -->
          <p class="text-mini text-ink-3">
            {{ t('sejours.arrivee.sans_fiche_possible') }}
          </p>
        </div>
      </section>

      <!-- ═══ LE SÉJOUR — les nuits d'un tap, les heures modifiables ═══ -->
      <section
        class="flex flex-col gap-3"
        data-bloc="sejour"
      >
        <h2 class="font-titre text-titre-m font-semibold text-ink">
          {{ t('sejours.arrivee.combien_de_nuits') }}
        </h2>

        <div class="flex flex-wrap gap-2">
          <button
            v-for="n in NUITS_PROPOSEES"
            :key="n"
            type="button"
            :data-nuits="n"
            :aria-pressed="nuits === n"
            class="h-12 min-w-12 rounded-xl border px-4 font-titre text-corps font-semibold transition-colors duration-90"
            :class="nuits === n
              ? 'border-ocre bg-ocre-soft text-ink'
              : 'border-line bg-surf text-ink-2 hover:bg-tile'"
            @click="nuits = n"
          >
            {{ n }}
          </button>
        </div>

        <div class="grid gap-3 sm:grid-cols-2">
          <ChampSaisie
            v-model="heureArrivee"
            etiquette-cle="sejours.arrivee.heure_arrivee"
            aide-cle="sejours.arrivee.heure_format"
            :erreur-cle="erreurHeureArrivee"
            taille="comptoir"
          />
          <ChampSaisie
            v-model="heureDepart"
            etiquette-cle="sejours.arrivee.heure_depart"
            aide-cle="sejours.arrivee.heure_format"
            :erreur-cle="erreurHeureDepart"
            taille="comptoir"
          />
        </div>

        <!--
          ★ L'établissement n'a pas réglé ses heures : l'écran le DIT. Poser 14 h / 12 h par défaut
          serait une règle métier en dur déguisée en commodité (porte P-12).
        -->
        <p
          v-if="heuresNonReglees"
          class="rounded-xl border border-dashed border-line-2 px-3.5 py-2.5 text-mini text-ink-2"
          role="status"
          data-alerte="heures-non-reglees"
        >
          {{ t('sejours.arrivee.heures_non_reglees') }}
        </p>

        <p
          v-if="debutPrevu && finPrevue && intervalleUtilisable"
          class="text-mini text-ink-3"
          data-recapitulatif
        >
          {{ t('sejours.arrivee.recapitulatif', {
            arrivee: `${dateDe(debutPrevu)} ${heureDe(debutPrevu)}`,
            depart: `${dateDe(finPrevue)} ${heureDe(finPrevue)}`,
          }) }}
        </p>

        <p
          v-if="totalPrevu"
          class="font-titre text-titre-m font-semibold text-ink"
          data-total-prevu
        >
          {{ t('sejours.arrivee.total_prevu', { montant: totalFormate, n: nuits }) }}
        </p>
      </section>

      <!-- ═══ LES ACCOMPAGNANTS ═══ -->
      <ListeAccompagnants
        :accompagnants="accompagnants"
        :capacite-accueil="capaciteAccueil"
        @mettre-a-jour="accompagnants = $event"
      />

      <!-- Le refus, en LANGUE UTILISATEUR — jamais un code, jamais un message de diagnostic. -->
      <p
        v-if="refus"
        class="rounded-xl border border-line-2 bg-tile px-3.5 py-3 text-corps text-ink"
        role="alert"
        data-refus
      >
        {{ t(refus.cle, refus.valeurs ?? {}) }}
      </p>

      <!--
        ★ La grille n'existe QUE si la permission d'ouvrir est là — **absente**, jamais grisée
        (FR-026). Voir la note de tête : le contrôle porte sur le HTML rendu.
      -->
      <GrilleUnites
        v-if="peutOuvrir && intervalleUtilisable"
        :unites="unitesDeLaCategorie"
        :choisie="null"
        :hors-ligne="horsLigne"
        :en-cours="uniteEnCours"
        @attribuer="attribuer"
      />
    </template>
  </div>
</template>
