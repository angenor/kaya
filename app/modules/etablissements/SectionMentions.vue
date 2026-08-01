<script setup lang="ts">
/**
 * **Section « Mentions »** — l'attribution des œuvres tierces, atteignable depuis le produit.
 *
 * # Pourquoi elle est ici, et pas sur un écran « à propos »
 *
 * Il n'existait pas d'écran « à propos » quand cette section a été écrite.
 * `docs/design/derivation.md` liste les écrans du produit et pose une règle opposable : « Un écran
 * absent des deux NE SE CODE PAS ». En inventer un pour y loger des mentions légales aurait coûté
 * plus cher que le problème.
 *
 * `G1` est le seul écran de back-office du produit à ce jour, et il hérite de `G2` — Configuration.
 * Une section de plus dans un écran de configuration ne crée **aucun motif nouveau** : c'est
 * exactement la composition que `G2` porte déjà, et que les cinq autres sections de `G1` emploient.
 *
 * **Ces mentions migreront vers `A1` « À propos »**, désormais inscrit à la matrice de dérivation
 * (hérite de `G2`, configuration en lecture seule) : les licences du produit ne sont pas un
 * réglage d'établissement. `A1` n'est pas construit — aucune story ne l'appelle encore — et la
 * section reste donc ici en attendant, à déménager d'un bloc.
 *
 * # Pourquoi l'attribution doit être visible, et pas seulement présente
 *
 * Le produit embarque six fichiers de police dans un binaire **vendu par abonnement**. La clause 2
 * de l'OFL 1.1 et la clause d'attribution du MIT demandent que l'avis de copyright et la licence
 * accompagnent **toutes les copies**. Les fichiers de licence vivent à côté des polices
 * (`app/assets/fonts/*-LICENCE.txt`) et sont importés en clair par `core/licences/` — c'est ce qui
 * les fait entrer dans le paquet. Cette section est l'autre moitié : ce qui rend l'attribution
 * atteignable par quelqu'un, plutôt qu'enfouie dans un fichier que personne n'ouvre.
 *
 * # Les avis de copyright ne passent pas par l'i18n
 *
 * Ce sont des **textes légaux**, invariables : un avis traduit perd sa valeur. Ce sont des données,
 * au même titre que le nom d'un établissement — voir `core/licences/index.ts`. Tout ce qui les
 * entoure, en revanche, est en clés `fr`/`en`.
 */
import { LICENCES_TIERCES } from '~/core/licences'

const { t } = useI18n()
</script>

<template>
  <section class="flex flex-col">
    <div class="flex flex-col gap-1.5 px-3.5 pt-4 pb-1">
      <span class="text-etiquette uppercase text-ink-3">{{ t('etablissement.sur_titre') }}</span>
      <h2 class="font-titre text-chiffre font-semibold text-ink">
        {{ t('mentions.titre') }}
      </h2>
      <p class="text-corps text-ink-2">
        {{ t('mentions.intro') }}
      </p>
    </div>

    <ul class="flex list-none flex-col gap-2.25 px-3 pt-3 pb-3.5">
      <li
        v-for="licence in LICENCES_TIERCES"
        :key="licence.id"
        class="flex flex-col gap-2 rounded-l-xs rounded-r-xl border border-line border-l-4 border-l-ocre bg-surf p-3 shadow-basse"
      >
        <div class="flex items-baseline gap-3">
          <span class="min-w-0 flex-1 font-titre text-titre-s font-semibold text-ink">{{ licence.nom }}</span>
          <span class="shrink-0 font-mono text-mini text-ink-3">{{ licence.licence }}</span>
        </div>

        <p class="text-corps text-ink-2">
          {{ t(licence.usageCle) }}
        </p>

        <!-- Mot pour mot, jamais traduit : c'est l'avis que la licence demande d'inclure. -->
        <p class="font-mono text-mini text-ink-3">
          {{ licence.copyright }}
        </p>

        <p
          v-if="licence.modificationCle"
          class="flex items-start gap-1.5 text-mini text-ink-3"
        >
          <i
            class="ph ph-info mt-0.5 shrink-0 text-corps"
            aria-hidden="true"
          />
          {{ t(licence.modificationCle) }}
        </p>

        <!-- Le texte intégral, embarqué et consultable. Replié par défaut : il fait quatre-vingt-
             treize lignes, et personne ne le lit avant d'en avoir besoin. -->
        <details class="flex flex-col gap-2">
          <summary class="h-9 inline-flex cursor-pointer items-center rounded-md font-titre text-mini font-semibold text-prim">
            {{ t('mentions.voir_licence') }}
          </summary>
          <pre class="mt-2 max-h-96 overflow-auto rounded-md bg-tile p-3 font-mono text-mini whitespace-pre-wrap text-ink-2">{{ licence.texte }}</pre>
        </details>
      </li>
    </ul>
  </section>
</template>
