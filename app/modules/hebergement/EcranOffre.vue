<script setup lang="ts">
/**
 * **`G2` — L'offre d'hébergement.** Écran **maquetté**, cas (a) de `docs/Kaya_Design.md` §2.
 *
 * | État | Fichier de référence |
 * |---|---|
 * | Hôtel — quatre formules | `docs/design/html/G2-offre-hebergement.html` |
 * | Résidence — deux formules | `docs/design/html/G2-offre-hebergement-residence.html` |
 *
 * **Le HTML de maquette n'est jamais copié ni déplacé vers `app/`** (porte P-19). On en lit les
 * valeurs et la structure — en-tête `text-etiquette uppercase`, titre `font-titre text-chiffre`,
 * cartes de formule, bouton principal `h-13 rounded-xl bg-prim` — et on réimplémente en composants
 * Nuxt avec i18n, mode sombre, RBAC et chargement paresseux, que l'export ne contient pas.
 *
 * # Les deux états ne sont pas deux écrans, et c'est la démonstration de l'écran
 *
 * La résidence n'a que deux formules, et l'absence du passage y est **écrite en clair** avec le
 * geste pour l'ajouter. C'est la preuve visuelle qu'aucune formule n'est réservée à un type
 * d'établissement (FR-017, FR-019) : l'offre suit l'établissement, elle n'est pas un gabarit à
 * remplir. Un vide expliqué vaut mieux qu'une case grisée.
 *
 * Le second état n'est donc pas un cas dégradé à traiter : c'est le même rendu, avec moins de
 * formules et un encart de plus.
 *
 * # Sans la permission de gérer, les actions sont ABSENTES
 *
 * Ni `disabled`, ni infobulle explicative, **rien dans le HTML rendu** (principe VII). Le grisé
 * apprend à qui ne peut pas agir ce qu'il pourrait faire ailleurs, et l'invite à chercher comment.
 *
 * # Hors ligne, l'action DIT qu'elle exige le réseau
 *
 * `formule` est de **classe C** (registre §7.1). Hors ligne ou en réseau dégradé, les actions
 * disparaissent et **un bandeau les remplace**, qui dit pourquoi en une phrase. Jamais de grisé
 * silencieux, jamais de mise en file « au cas où ».
 */
import { computed, ref } from 'vue'

import ChampSaisie from '~/core/design-system/ChampSaisie.vue'
import { useEtatReseau } from '~/core/platform/reseau'
import { detient, type Permissions } from '~/core/rbac'
import CarteFormule from './CarteFormule.vue'
import { chargerFormules, type CategorieVue, type ContexteAppel, type FormuleVue } from './donnees'
import {
  modifierFormule,
  PERMISSION_GERER,
  validerPrix,
  type RegleConversionTaxe,
} from './modifier-formule'

const { t } = useI18n()

const props = defineProps<{
  categories: CategorieVue[]
  formules: FormuleVue[]
  contexte: ContexteAppel
  etablissementId: string
  /** Permissions cumulées — l'union des rôles portés, jamais celles d'un rôle principal. */
  permissions: Permissions
}>()

const emit = defineEmits<{ 'formules-changees': [formules: FormuleVue[]] }>()

const reseau = useEtatReseau()

/**
 * L'état `degrade` est traité **comme** hors ligne pour une opération de classe C. Personne ne le
 * produit encore ; le cycle SYN l'alimentera depuis les échecs réels de requête.
 */
const enLigne = computed(() => reseau.value === 'connecte')

const peutGerer = computed(() => detient(props.permissions, PERMISSION_GERER))

/** Nom du type de chambre d'une formule — jamais son identifiant, qui ne dit rien à personne. */
function nomCategorie(formule: FormuleVue): string {
  return props.categories.find(c => c.id === formule.categorie_id)?.nom ?? ''
}

/**
 * **L'affordance « Ajouter le passage ici »** — l'état résidence de la maquette.
 *
 * Elle apparaît quand aucune formule de passage n'existe. Elle n'est pas conditionnée au *type*
 * d'établissement : rien dans le produit ne réserve le passage aux hôtels, et une résidence qui
 * veut en proposer doit pouvoir le faire d'un geste.
 */
const passageAbsent = computed(() => !props.formules.some(f => f.famille === 'PASSAGE'))

// ── L'écriture — la septième couche ──────────────────────────────────────────────────────────

/** La formule ouverte au réglage. `null` = aucune. */
const formuleChoisie = ref<FormuleVue | null>(null)
const prixSaisi = ref('')
const assujettie = ref(false)
const regle = ref<RegleConversionTaxe>('une_nuitee_par_occupation')

/** Erreur **au champ** — elle porte sur ce qui est saisi, à côté de l'endroit où l'on corrige. */
const erreurChamp = ref<string | null>(null)

/**
 * Refus **métier**, rendu par un bandeau. Une seule variable, jamais une liste : **jamais deux
 * bandeaux empilés** (composant 07).
 */
const refus = ref<{ cle: string, valeurs?: Record<string, unknown> } | null>(null)

/**
 * Chargement — **un squelette, pas un indicateur générique** (composant 13). L'état porte la
 * **cible** de l'opération, pas un simple booléen : c'est la ligne concernée qui devient un
 * squelette, et une roue au milieu de l'écran ne dirait pas laquelle.
 */
const enCours = ref<string | null>(null)

function ouvrir(formule: FormuleVue): void {
  formuleChoisie.value = formule
  prixSaisi.value = String(formule.prix_mineur)
  assujettie.value = formule.assujettie_taxe_nuitee
  regle.value = formule.regle_conversion_taxe ?? 'une_nuitee_par_occupation'
  erreurChamp.value = null
  refus.value = null
}

function fermer(): void {
  formuleChoisie.value = null
  erreurChamp.value = null
  refus.value = null
}

/**
 * Les deux libellés du choix fiscal — **validés au terrain le 2026-08-02**.
 *
 * « Une seule taxe pour tout le séjour » / « Une taxe par nuit ». Ni le mot « conversion », ni le
 * mot « prorata », ni le nom de l'énumération n'atteignent l'interface. Et ces deux formulations
 * **ne disent rien des personnes** : c'est ce qui les rend employables alors que l'axe « par
 * client » de la taxe n'est pas tranché (B-10).
 */
const OPTIONS_REGLE: { valeur: string, libelleCle: string }[] = [
  { valeur: 'une_nuitee_par_occupation', libelleCle: 'hebergement.offre.taxe_une_par_sejour' },
  { valeur: 'au_prorata', libelleCle: 'hebergement.offre.taxe_une_par_nuit' },
]

async function enregistrer(): Promise<void> {
  const formule = formuleChoisie.value
  if (!formule) {
    return
  }

  // **Validation au champ, avant tout appel** : elle porte sur ce qui est saisi.
  const erreur = validerPrix(prixSaisi.value)
  if (erreur) {
    erreurChamp.value = erreur
    return
  }
  erreurChamp.value = null
  refus.value = null
  enCours.value = formule.id

  const resultat = await modifierFormule(
    props.contexte,
    props.etablissementId,
    formule,
    {
      prixMineur: Number(prixSaisi.value.trim()),
      assujettieTaxeNuitee: assujettie.value,
      // `null` **seulement** sur une formule non assujettie — la base rend l'autre combinaison
      // impossible, et l'envoyer produirait un refus que l'utilisateur ne comprendrait pas.
      regleConversionTaxe: assujettie.value ? regle.value : null,
    },
    reseau.value,
  )

  enCours.value = null

  if (resultat.issue === 'refus') {
    refus.value = { cle: resultat.cle, valeurs: resultat.valeurs }
    return
  }

  // **Le rafraîchissement suit le succès, il ne l'accompagne pas.** Relire avant que le serveur
  // ait tranché afficherait l'état d'avant en donnant l'impression qu'il s'agit de celui d'après.
  // Et la liste vient du serveur, jamais reconstruite à la main : il fait foi en conflit.
  emit('formules-changees', await chargerFormules(props.contexte, props.etablissementId))
  fermer()
}
</script>

<template>
  <section class="flex flex-1 flex-col">
    <div class="px-3.5 pt-4 pb-1 flex flex-col gap-1.5">
      <span class="text-etiquette uppercase text-ink-3">{{ t('hebergement.offre.surtitre') }}</span>
      <h2 class="font-titre text-chiffre font-semibold text-ink">
        {{ t('hebergement.offre.titre') }}
      </h2>
      <p class="text-corps text-ink-2">
        {{ t('hebergement.offre.sous_titre', { nombre: formules.length }) }}
      </p>
    </div>

    <!-- Bandeau de refus — composant 07, contrefort de 4 px, une phrase au passé. JAMAIS deux
         bandeaux empilés : d'où une seule variable d'état, pas une liste. -->
    <div
      v-if="refus"
      class="mx-3 mt-2 rounded-l-xs rounded-r-xl border-l-4 border-l-danger bg-danger-soft px-3.5 py-3"
      role="alert"
    >
      <p class="text-corps text-danger-fort">
        <i
          class="ph-fill ph-warning-circle"
          aria-hidden="true"
        />
        {{ t(refus.cle, refus.valeurs ?? {}) }}
      </p>
    </div>

    <!-- Hors ligne : l'action disparaît et un bandeau DIT pourquoi, immédiatement. Classe C. -->
    <div
      v-else-if="!enLigne"
      class="mx-3 mt-2 rounded-l-xs rounded-r-xl border-l-4 border-l-info bg-info-soft px-3.5 py-3"
    >
      <p class="text-corps text-info-fort">
        {{ t('hebergement.offre.refus.reseau') }}
      </p>
    </div>

    <div class="px-3 pt-3 pb-3.5 flex flex-col gap-2.25">
      <template
        v-for="formule in formules"
        :key="formule.id"
      >
        <!-- Squelette de la LIGNE CONCERNÉE — même hauteur que le contenu réel, pour que rien ne
             saute (composant 13). -->
        <div
          v-if="enCours === formule.id"
          class="w-full h-21 rounded-l-xs rounded-r-xl bg-tile animate-souffle"
          :aria-label="t('hebergement.offre.enregistrement')"
        />
        <CarteFormule
          v-else
          :formule="formule"
          :nom-categorie="nomCategorie(formule)"
          @choisir="ouvrir"
        />
      </template>

      <!-- L'état résidence : l'absence du passage écrite en clair, avec le geste pour l'ajouter.
           Un vide expliqué vaut mieux qu'une case grisée. -->
      <div
        v-if="passageAbsent && formules.length > 0"
        class="mt-0.5 px-3.75 py-3.25 rounded-xl border border-dashed border-line-2 flex flex-col gap-2"
      >
        <span class="text-corps font-medium text-ink-2">
          {{ t('hebergement.offre.passage_absent') }}
        </span>
        <button
          v-if="peutGerer && enLigne"
          type="button"
          class="self-start h-10 px-3.5 rounded-lg border-[1.5px] border-prim bg-transparent text-prim font-titre text-corps font-semibold cursor-pointer transition-[transform,background-color] duration-90 ease-entree hover:bg-prim-soft active:translate-y-0.5"
        >
          {{ t('hebergement.offre.ajouter_passage') }}
        </button>
      </div>

      <!-- État vide illustré — composant 11. Un établissement qui vient d'activer l'hébergement
           n'a aucune formule, et ce n'est pas une erreur. -->
      <div
        v-if="formules.length === 0"
        class="mt-2 flex flex-col items-center gap-2 px-6 py-8 text-center"
      >
        <i
          class="ph ph-bed text-affiche text-ocre"
          aria-hidden="true"
        />
        <p class="font-titre text-titre-s font-semibold text-ink">
          {{ t('hebergement.offre.vide_titre') }}
        </p>
        <p class="text-corps text-ink-2">
          {{ t('hebergement.offre.vide_aide') }}
        </p>
      </div>
    </div>

    <!-- Panneau de réglage — le motif « Configuration » de G2, dont G1 hérite déjà. -->
    <div
      v-if="formuleChoisie"
      class="mx-3 mb-3 rounded-2xl border border-line bg-surf p-3.5 flex flex-col gap-3 shadow-carte"
    >
      <h3 class="font-titre text-titre-s font-semibold text-ink">
        {{ t(`hebergement.familles.${formuleChoisie.famille}`) }}
      </h3>

      <ChampSaisie
        v-model="prixSaisi"
        etiquette-cle="hebergement.offre.champ_prix"
        aide-cle="hebergement.offre.champ_prix_aide"
        :erreur-cle="erreurChamp"
      />

      <label class="flex items-center gap-2.5 text-corps text-ink">
        <input
          v-model="assujettie"
          type="checkbox"
          class="size-5 accent-prim"
        >
        {{ t('hebergement.offre.champ_taxe') }}
      </label>

      <ChampSaisie
        v-if="assujettie"
        v-model="regle"
        etiquette-cle="hebergement.offre.champ_regle"
        :options="OPTIONS_REGLE"
      />

      <div class="flex gap-2">
        <!-- Action principale — ABSENTE sans la permission, et absente hors ligne. Ni `disabled`,
             ni infobulle : rien dans le HTML rendu. -->
        <button
          v-if="peutGerer && enLigne"
          type="button"
          class="flex-1 h-12 rounded-xl bg-prim text-prim-ink font-titre text-action font-semibold cursor-pointer shadow-bouton transition-[transform,box-shadow] duration-90 ease-entree active:translate-y-0.5 active:shadow-bouton-appui"
          @click="enregistrer"
        >
          {{ t('hebergement.offre.enregistrer') }}
        </button>
        <button
          type="button"
          class="h-12 px-4 rounded-xl border border-line bg-tile text-ink font-titre text-action font-semibold cursor-pointer"
          @click="fermer"
        >
          {{ t('hebergement.offre.annuler') }}
        </button>
      </div>
    </div>

    <div class="shrink-0 px-3 pt-2.75 pb-3.5 bg-surf border-t border-line flex flex-col gap-2">
      <button
        v-if="peutGerer && enLigne"
        type="button"
        class="w-full h-13 rounded-xl bg-prim text-prim-ink font-titre text-titre-s font-semibold inline-flex items-center justify-center gap-2.5 cursor-pointer shadow-bouton-grand transition-[transform,box-shadow] duration-90 ease-entree active:translate-y-0.75 active:shadow-none"
      >
        <i
          class="ph ph-plus text-titre-m"
          aria-hidden="true"
        />
        {{ t('hebergement.offre.ajouter_formule') }}
      </button>
    </div>
  </section>
</template>
