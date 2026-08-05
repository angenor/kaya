<script setup lang="ts">
/**
 * ★ **`R5` — La fiche client.** Écran **DÉRIVÉ**, jamais maquetté.
 *
 * Référence : `docs/design/derivation.md`, ligne `R5` → motif **`R7`**. *« Liste + fiche, pas de
 * total. »* La matrice est **opposable** : un écran hors d'elle et hors des maquettes ne se code
 * pas (porte P-19). Ce composant est la **colonne de droite de `R7`**, **sans son bloc de total** —
 * une fiche client n'a pas de montant à porter, et lui en donner un ferait croire à un solde.
 *
 * # ★ L'HISTORIQUE COUVRE TOUS LES ÉTABLISSEMENTS DU TENANT
 *
 * La fiche est du **tenant**, pas d'un établissement (FR-002) : un client de Deloria enregistré à
 * l'accueil est le même client au restaurant, et ses préférences le suivent. L'historique le dit
 * — du plus récent au plus ancien, l'ordre dans lequel on cherche « la dernière fois ».
 *
 * # Les préférences sont APPEND-ONLY, et l'écran le montre
 *
 * Aucun bouton pour modifier ou effacer une préférence : la table n'accorde que `SELECT` et
 * `INSERT`. « Allergique aux arachides » raturé et réécrit ne laisserait aucune trace de qui a
 * raturé — et cette information-là peut coûter une hospitalisation.
 *
 * # Le numéro de pièce n'est PAS affiché ici
 *
 * `GET /clients/{id}` le rend, et sa lecture est **journalisée** au registre des actions
 * (FR-012) — famille `consultation_piece_identite`. L'afficher d'office ferait tracer une
 * consultation à chaque ouverture de fiche, et **noierait les vraies consultations** sous des
 * entrées que personne n'a voulues. Il se demande, il ne s'expose pas.
 */
import { computed } from 'vue'

import { formaterMontant } from '~/core/format/montant'
import type { FicheClientDetail, SejourVue } from './donnees'

const { t, locale } = useI18n()

const props = defineProps<{
  fiche: FicheClientDetail
  /** Les séjours du client, **tous établissements du tenant confondus**. */
  historique: SejourVue[]
  /** Vrai pendant le chargement de l'historique — squelette, jamais un écran vide. */
  chargementHistorique: boolean
}>()

const nomComplet = computed(() =>
  props.fiche.prenoms ? `${props.fiche.prenoms} ${props.fiche.nom}` : props.fiche.nom,
)

const dateDe = (instant: string | null | undefined): string => {
  if (!instant) return ''
  return new Intl.DateTimeFormat(locale.value, {
    day: 'numeric',
    month: 'long',
    year: 'numeric',
  }).format(new Date(instant))
}

const totalDe = (sejour: SejourVue): string =>
  formaterMontant(sejour.total_mineur, sejour.devise)

/** Les coordonnées réellement renseignées — un champ vide ne se rend pas, il n'existe pas. */
const coordonnees = computed(() => [
  { cle: 'sejours.fiche.telephone', valeur: props.fiche.telephone },
  { cle: 'sejours.fiche.email', valeur: props.fiche.email },
  { cle: 'sejours.fiche.nationalite', valeur: props.fiche.nationalite },
].filter((c): c is { cle: string, valeur: string } => Boolean(c.valeur)))
</script>

<template>
  <section class="flex flex-col gap-3.5">
    <!-- ═══ IDENTITÉ ═══ -->
    <header class="flex flex-col gap-2.5 rounded-xl border border-line bg-tile px-4.5 py-4">
      <h2 class="font-titre text-titre-s font-semibold text-ink">
        {{ nomComplet }}
      </h2>
      <dl
        v-if="coordonnees.length"
        class="flex flex-wrap gap-4.5"
      >
        <div
          v-for="coordonnee in coordonnees"
          :key="coordonnee.cle"
          class="flex flex-col gap-0.5"
        >
          <dt class="text-etiquette text-ink-3">
            {{ t(coordonnee.cle) }}
          </dt>
          <dd class="font-mono text-corps text-ink">
            {{ coordonnee.valeur }}
          </dd>
        </div>
      </dl>
      <p
        v-else
        class="text-mini text-ink-3"
      >
        {{ t('sejours.fiche.sans_coordonnees') }}
      </p>

      <!--
        La pièce d'identité : on dit qu'elle est enregistrée, on ne la montre pas. La lire est une
        consultation journalisée ; l'afficher d'office noierait les vraies consultations.
      -->
      <p class="text-mini text-ink-3">
        {{ fiche.piece_capturee_le
          ? t('sejours.fiche.piece_enregistree', { date: dateDe(fiche.piece_capturee_le) })
          : t('sejours.fiche.piece_absente') }}
      </p>
    </header>

    <!-- ═══ PRÉFÉRENCES ═══ -->
    <section class="flex flex-col gap-2 rounded-xl border border-line px-4.5 py-4">
      <span class="text-etiquette uppercase text-ink-3">
        {{ t('sejours.fiche.preferences') }}
      </span>
      <ul
        v-if="fiche.preferences.length"
        class="flex flex-col gap-1.5"
      >
        <li
          v-for="preference in fiche.preferences"
          :key="preference.id"
          class="flex flex-col gap-0.5"
          data-preference
        >
          <span class="text-corps text-ink">{{ preference.texte }}</span>
          <span class="text-mini text-ink-3">{{ dateDe(preference.cree_le) }}</span>
        </li>
      </ul>
      <p
        v-else
        class="text-mini text-ink-3"
      >
        {{ t('sejours.fiche.sans_preference') }}
      </p>
    </section>

    <!-- ═══ HISTORIQUE — tous établissements du tenant, du plus récent au plus ancien ═══ -->
    <section class="flex flex-col gap-2 rounded-xl border border-line px-4.5 py-4">
      <span class="text-etiquette uppercase text-ink-3">
        {{ t('sejours.fiche.historique') }}
      </span>

      <div
        v-if="chargementHistorique"
        class="flex flex-col gap-2"
        data-chargement-historique
      >
        <span class="h-4 w-full rounded bg-tile animate-souffle" />
        <span class="h-4 w-2/3 rounded bg-tile animate-souffle" />
      </div>

      <ul
        v-else-if="historique.length"
        class="flex flex-col gap-2"
      >
        <li
          v-for="sejour in historique"
          :key="sejour.sejour.id"
          class="flex items-baseline justify-between gap-3 border-t border-line pt-2 first:border-0 first:pt-0"
          data-sejour-historique
        >
          <span class="flex min-w-0 flex-col gap-0.5">
            <span class="font-titre text-corps font-medium text-ink">
              {{ dateDe(sejour.sejour.ouvert_le) }}
            </span>
            <span class="text-mini text-ink-3">
              {{ t('sejours.fiche.personnes', { n: sejour.nombre_personnes }) }}
            </span>
          </span>
          <!--
            Le montant du séjour, jamais un cumul : la fiche client dérive de `R7` SANS son bloc de
            total. Additionner les séjours afficherait un chiffre qui ressemble à un solde, et
            l'exploitant y chercherait ce que le client doit — que ce cycle ne calcule pas.
          -->
          <span class="shrink-0 font-mono text-corps text-ink-2">{{ totalDe(sejour) }}</span>
        </li>
      </ul>

      <p
        v-else
        class="text-mini text-ink-3"
      >
        {{ t('sejours.fiche.sans_sejour') }}
      </p>
    </section>
  </section>
</template>
