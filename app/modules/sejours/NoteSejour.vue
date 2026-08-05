<script setup lang="ts">
/**
 * ★ **`R7` — La note.** Écran **MAQUETTÉ** : `docs/design/html/R7-note-depart.html`.
 *
 * Valeurs lues, **HTML jamais copié** (porte P-19) — la maquette est autonome, non sémantique,
 * sans i18n et sans RBAC, quatre choses que ce composant doit avoir.
 *
 * # ⚠️ LA MAQUETTE MONTRE QUATRE SECTIONS. CE CYCLE N'EN SERT QU'UNE.
 *
 * `R7-note-depart.html` porte **Hébergement**, **Restaurant**, **Bar** et **Autres frais**, puis
 * un bloc de taxes. Ce cycle ne livre que la première : les trois autres viennent avec les points
 * de vente (**T2**), et les taxes avec **FIS** (T3).
 *
 * ★ **Leur absence est VISIBLE COMME UNE ABSENCE**, jamais comme un vide inexpliqué. Une note qui
 * s'arrêterait à l'hébergement sans rien dire se lirait « ce client n'a rien consommé » — et
 * Adjoua encaisserait un total faux en toute confiance. La note **nomme** ce qu'elle ne porte pas
 * encore et **pourquoi**.
 *
 * C'est la différence entre une fonctionnalité manquante et un chiffre faux : la première se
 * répare au cycle suivant, la seconde se paie devant le client.
 *
 * # Le total est PROVISOIRE, et le mot est de la maquette
 *
 * *« Total provisoire · Toutes taxes comprises, à l'instant »*. Il n'est provisoire ni par
 * prudence ni par pudeur : le séjour est ouvert, une nuit de plus peut s'ajouter, et **les taxes
 * ne sont pas calculées par ce cycle**. Écrire « Total » sec serait une affirmation que le produit
 * ne peut pas tenir.
 *
 * # La mention obligatoire, sur ce document comme sur la fiche de police
 *
 * « **Document non fiscal — ne tient pas lieu de facture** » — FIS-02, principe V, FR-048. Elle
 * n'est pas décorative : sans elle, un client pourrait présenter cette note à l'administration
 * comme une facture, et l'exploitant en répondrait.
 *
 * # Tout montant passe par `core/format/montant.ts`
 *
 * **Espace fine insécable** entre les groupes de milliers et avant le F, et colonne alignée en
 * Chivo Mono tabulaire. Les **heures gardent l'espace ORDINAIRE** (`11 h 12`) et ne passent pas
 * par le formateur de montant.
 */
import { computed } from 'vue'

import { formaterMontant } from '~/core/format/montant'
import type { LigneNote, NoteVue, SejourVue } from './donnees'

const { t, locale } = useI18n()

const props = defineProps<{
  note: NoteVue
  /** Le séjour dont la note est affichée — il porte le nom du client et le nombre de personnes. */
  sejour: SejourVue
  /** Le code de la chambre, `null` quand l'unité n'est plus lisible (séjour clos et purgé). */
  codeUnite: string | null
}>()

/**
 * Les lignes d'hébergement — **la seule section servie par ce cycle**.
 *
 * `nature` vaut `hebergement` ou `ajustement` : les deux appartiennent à la section hébergement
 * de la maquette, un ajustement étant un remboursement ou un supplément **sur la même prestation**.
 */
const lignesHebergement = computed(() => props.note.lignes)

const sousTotalHebergement = computed(() =>
  lignesHebergement.value.reduce((somme, ligne) => somme + ligne.montant_mineur, 0),
)

/**
 * ★ **Les sections que ce cycle ne sert pas — nommées, avec leur cycle.**
 *
 * Elles sont rendues **en creux**, pas omises. Un vide silencieux se lirait « rien consommé ».
 */
const SECTIONS_A_VENIR = ['restaurant', 'bar', 'autres_frais'] as const

const dateDe = (instant: string | null | undefined): string => {
  if (!instant) return ''
  return new Intl.DateTimeFormat(locale.value, {
    weekday: 'short',
    day: 'numeric',
    month: 'short',
  }).format(new Date(instant))
}

/** Une quantité — **`NUMERIC` au contrat**, donc une chaîne : jamais convertie en nombre. */
function quantiteAffichable(ligne: LigneNote): string {
  // `×1` sur chaque nuit serait du bruit : la maquette ne l'affiche que lorsqu'il compte.
  const valeur = Number(ligne.quantite)
  return Number.isFinite(valeur) && valeur === 1 ? '' : `×${ligne.quantite}`
}

const montant = (mineur: number): string => formaterMontant(mineur, props.note.devise)

/** La note est-elle arrêtée ? Alors elle ne bougera plus — et l'écran le dit au passé. */
const arretee = computed(() => props.note.statut === 'arretee')
</script>

<template>
  <section class="flex flex-col">
    <!-- ═══ EN-TÊTE : la chambre, le client, les dates ═══ -->
    <header class="flex flex-col gap-1.5 pb-4">
      <span class="text-etiquette uppercase text-ink-3">
        {{ t('sejours.note.titre') }}
      </span>
      <h2 class="font-titre text-titre-l font-semibold text-ink">
        {{ codeUnite
          ? t('sejours.note.chambre_et_client', {
            chambre: codeUnite,
            client: sejour.client_nom ?? t('sejours.note.client_sans_fiche'),
          })
          : (sejour.client_nom ?? t('sejours.note.client_sans_fiche')) }}
      </h2>
      <p class="text-corps text-ink-2">
        {{ t('sejours.note.sous_titre', {
          arrivee: dateDe(sejour.sejour.ouvert_le),
          depart: dateDe(sejour.fin_prevue),
          personnes: sejour.nombre_personnes,
        }) }}
      </p>
    </header>

    <!-- ═══ SECTION HÉBERGEMENT — la seule servie par ce cycle ═══ -->
    <div class="rounded-t-xl border border-b-0 border-line bg-surf">
      <div class="pt-4.5">
        <div class="flex items-baseline gap-2.5 px-5.5 pb-2">
          <span class="text-etiquette uppercase text-ocre">
            {{ t('sejours.note.section.hebergement') }}
          </span>
        </div>

        <div
          v-for="ligne in lignesHebergement"
          :key="ligne.id"
          class="flex items-center gap-3.5 border-t border-line px-5.5 py-2.5"
          data-ligne
        >
          <!-- Le filet de ligne de la maquette `R7` — `w-2.5 h-[3px]`. Écrit `h-0.75` et non
               `h-[3px]` : l'échelle de Tailwind 4 est `calc(var(--spacing) * n)`, et 0,75 × 4 px
               donne EXACTEMENT les 3 px de la maquette. Ce n'est pas un arrondi, c'est la même
               valeur exprimée par le jeton — ce que P-17 exige, et ce que la maquette voulait
               dire. Le dépôt emploie déjà `size-9.5`, `py-5.5` et `w-2.5` : les décimales de
               l'échelle sont l'usage, pas une astuce. -->
          <span class="h-0.75 w-2.5 shrink-0 rounded-xs bg-line-2" />
          <span class="flex min-w-0 flex-1 flex-col gap-0.5">
            <span class="font-titre text-action font-medium text-ink">
              {{ t(ligne.libelle_cle) }}
            </span>
            <span
              v-if="ligne.periode_debut"
              class="text-mini text-ink-3"
            >
              {{ t('sejours.note.du_au', {
                debut: dateDe(ligne.periode_debut),
                fin: dateDe(ligne.periode_fin),
              }) }}
            </span>
            <!--
              Le motif n'existe que sur un ajustement, et il n'est jamais deviné : le propriétaire
              y cherche pourquoi un montant a bougé.
            -->
            <span
              v-if="ligne.motif"
              class="text-mini text-ink-3"
            >
              {{ t(`sejours.note.motif.${ligne.motif}`) }}
            </span>
          </span>
          <span class="w-14.5 shrink-0 text-right font-mono text-corps text-ink-3">
            {{ quantiteAffichable(ligne) }}
          </span>
          <span class="w-29 shrink-0 text-right font-mono text-lead text-ink">
            {{ montant(ligne.montant_mineur) }}
          </span>
        </div>

        <div class="mt-1 flex items-center gap-3 border-t border-line px-5.5 pb-4 pt-2.5">
          <span class="flex-1 font-titre text-corps font-semibold text-ink-2">
            {{ t('sejours.note.sous_total_hebergement') }}
          </span>
          <span
            class="w-29 shrink-0 text-right font-mono text-lead font-semibold text-ink"
            data-sous-total
          >
            {{ montant(sousTotalHebergement) }}
          </span>
        </div>
      </div>

      <!--
        ★ LES SECTIONS ABSENTES, VISIBLES COMME DES ABSENCES.
        Un vide silencieux se lirait « ce client n'a rien consommé », et le total serait encaissé
        de bonne foi. Chacune nomme ce qu'elle porte et d'où elle viendra.
      -->
      <div
        v-for="section in SECTIONS_A_VENIR"
        :key="section"
        class="border-t border-dashed border-line-2 px-5.5 py-3.5"
        :data-section-absente="section"
      >
        <p class="flex flex-col gap-0.5">
          <span class="text-etiquette uppercase text-ink-3">
            {{ t(`sejours.note.section.${section}`) }}
          </span>
          <span class="text-mini text-ink-3">
            {{ t('sejours.note.section_a_venir') }}
          </span>
        </p>
      </div>

      <!--
        Les taxes viennent de FIS (T3). Ce cycle FIGE les faits du constat — nuits, personnes,
        paramétrage recopié — et ne calcule aucun montant : la règle fiscale vit dans
        l'adaptateur de juridiction, jamais ici.
      -->
      <div
        class="border-t border-dashed border-line-2 px-5.5 py-3.5"
        data-section-absente="taxes"
      >
        <p class="flex flex-col gap-0.5">
          <span class="text-etiquette uppercase text-ink-3">
            {{ t('sejours.note.section.taxes') }}
          </span>
          <span class="text-mini text-ink-3">
            {{ t('sejours.note.taxes_a_venir') }}
          </span>
        </p>
      </div>
    </div>

    <!-- ═══ PIED ÉPINGLÉ : le total ne défile jamais ═══ -->
    <div class="rounded-b-xl border border-line bg-surf shadow-basse">
      <div class="flex items-end gap-5.5 border-t-2 border-ink px-5.5 pb-4 pt-4.5">
        <div class="flex min-w-0 flex-1 flex-col gap-1">
          <span class="text-etiquette uppercase text-ink-3">
            {{ arretee ? t('sejours.note.total_arrete') : t('sejours.note.total_provisoire') }}
          </span>
          <span class="text-mini text-ink-2">
            {{ arretee ? t('sejours.note.total_arrete_aide') : t('sejours.note.total_provisoire_aide') }}
          </span>
        </div>
        <span
          class="shrink-0 whitespace-nowrap font-mono text-total font-bold text-ink"
          data-total
        >
          {{ montant(note.total_mineur) }}
        </span>
      </div>

      <!--
        ★ LA MENTION OBLIGATOIRE — FIS-02, principe V, FR-048. Sans elle, un client pourrait
        présenter cette note à l'administration comme une facture, et l'exploitant en répondrait.
      -->
      <div
        class="flex items-center gap-2.5 border-t border-dashed border-line-2 px-5.5 pb-3 pt-2.5"
        data-mention-non-fiscale
      >
        <span class="text-etiquette font-semibold uppercase text-ocre">
          {{ t('documents.mention_non_fiscale') }}
        </span>
      </div>
    </div>
  </section>
</template>
