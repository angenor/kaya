<script setup lang="ts">
/**
 * **`S1` — « Mes envois »**, le développement du témoin de synchronisation.
 *
 * # ÉCRAN DÉRIVÉ, cas (b) — sa ligne de `docs/design/derivation.md` 1.2.1
 *
 * > *`S1` Panneau de synchronisation | **Composant 10** — témoin de synchronisation |
 * > Développement du composant : le témoin dit l'état d'un coup d'œil, le panneau détaille ce qui
 * > attend et permet d'agir*
 *
 * La matrice le faisait dériver du « composant 8 », qui est la **ligne de liste** — reste d'une
 * numérotation antérieure, corrigé avant ce cycle. La distinction n'est pas cosmétique : dériver
 * d'une ligne de liste aurait donné un registre de plus, alors que ce qu'il faut est le **même
 * vocabulaire visuel que le témoin**, en plus détaillé.
 *
 * # ⚠️ LE NOM DU FICHIER DÉCIDE DE LA ROUTE, et c'est pour cela qu'il est celui-là
 *
 * Le lexique proscrit le mot « synchronisation » du visible. **Une URL est visible** : elle
 * s'affiche dans la barre d'adresse, se copie, se lit à voix haute au téléphone avec le support.
 * `/synchronisation` aurait fait entrer par la porte du nom de fichier un mot que tout le reste du
 * produit écarte. Le titre est « Mes envois » — court, possessif, parce que c'est **son travail**
 * qui est en jeu, pas un mécanisme.
 *
 * # Ce que cet écran ajoute au témoin, et rien de plus
 *
 * | Le témoin | Cet écran |
 * |---|---|
 * | « 4 en attente d'envoi » | **lesquelles**, et depuis quand |
 * | *(rien)* | ce qui a été **refusé définitivement**, et pourquoi |
 * | *(rien)* | le geste de **renvoyer** une saisie refusée |
 *
 * La quarantaine n'est **pas** sur le témoin, et c'est délibéré : le témoin répond à « mon travail
 * est-il parti ? » d'un coup d'œil. Ce qui a été refusé demande une décision, donc du temps que le
 * comptoir n'a pas — donc un écran.
 *
 * # Les motifs sont branchés sur le `code`, JAMAIS sur le `message`
 *
 * Règle du lexique, et elle vaut ici comme ailleurs : le `message` du serveur est un diagnostic
 * destiné aux journaux — anglais technique, noms de tables. L'interface branche sa clé i18n sur le
 * `code`, avec un repli honnête pour un code inconnu.
 */
import { computed, ref } from 'vue'

import TemoinSynchronisation from '~/core/design-system/TemoinSynchronisation.vue'
import {
  avertissementHorloge,
  cleMotifRefus,
  fileCourante,
  signalerChangement,
  useEtatSynchronisation,
} from '~/core/sync'

const { t } = useI18n()

const etat = useEtatSynchronisation()

/**
 * L'heure de l'appareil, quand elle est fausse.
 *
 * **Le mot « dérive » n'apparaît pas**, et aucune valeur technique non plus : ni secondes, ni
 * horodatage, ni seuil. L'utilisateur lit une phrase, dans le sens qui le concerne — et la seconde,
 * celle qui le rassure, est **obligatoire** (lexique 1.5.1) : un avertissement qui inquiète sur ce
 * qui va bien est pire que pas d'avertissement.
 */
const horloge = avertissementHorloge()

/** Une relecture forcée après un geste — la file n'est pas réactive, l'état l'est. */
const revision = ref(0)

const enAttente = computed(() => {
  void revision.value
  void etat.value.enAttente
  return [...(fileCourante()?.lister() ?? [])]
})

const refusees = computed(() => {
  void revision.value
  void etat.value.enQuarantaine
  return [...(fileCourante()?.quarantaine() ?? [])]
})

function heure(horodatage: string): string {
  const date = new Date(horodatage)
  // Espace ORDINAIRE avant « h » — U+202F est réservé aux montants (`tokens.md` §2).
  return `${String(date.getHours()).padStart(2, '0')} h ${String(date.getMinutes()).padStart(2, '0')}`
}

/**
 * Renvoyer une saisie refusée — **geste explicite, jamais automatique**.
 *
 * Un rejeu automatique d'un refus définitif boucllerait indéfiniment : le serveur a décidé, et
 * rejouer ne changera pas sa décision. C'est l'exploitant qui tranche, après avoir lu le motif.
 */
function relancer(id: string): void {
  fileCourante()?.relancerDepuisQuarantaine(id)
  revision.value += 1
  signalerChangement()
}
</script>

<template>
  <div class="flex flex-1 flex-col">
    <div class="flex flex-col gap-2 px-3.5 pt-4 pb-1">
      <h1 class="font-titre text-chiffre font-semibold text-ink">
        {{ t('sync.titre') }}
      </h1>
      <p class="text-corps text-ink-2">
        {{ t('sync.sous_titre') }}
      </p>
      <!-- Le témoin lui-même, en tête : l'écran est son développement, il commence donc par ce
           qu'il développe. -->
      <TemoinSynchronisation />

      <!-- COMPOSANT 07 · BANDEAU — l'heure de l'appareil, si elle est fausse. Contrefort, fond
           `-soft`, texte `-fort`. Deux phrases : celle qui alerte, et celle qui rassure. -->
      <div
        v-if="horloge"
        class="flex items-start gap-2.5 rounded-r-lg border-l-4 border-l-alerte bg-alerte-soft p-3 font-texte text-mini text-alerte-fort"
        role="status"
      >
        <i
          class="ph ph-clock-countdown mt-0.5 shrink-0 text-corps text-alerte"
          aria-hidden="true"
        />
        <span class="flex flex-col gap-1">
          <span>{{ t(horloge.cle, { n: horloge.minutes }) }}</span>
          <span>{{ t('sync.horloge.rassurance') }}</span>
        </span>
      </div>
    </div>

    <!-- ─────────────────────────────────────────────────────────────────────────────────────
         EN ATTENTE D'ENVOI — ce qui partira dès que le réseau reviendra
         ───────────────────────────────────────────────────────────────────────────────────── -->
    <section class="flex flex-col px-3 pt-4">
      <h2 class="px-0.5 pb-1 text-etiquette uppercase text-ink-3">
        {{ t('sync.en_attente_titre') }}
      </h2>

      <p
        v-if="enAttente.length === 0"
        class="px-0.5 py-3 text-corps text-ink-2"
      >
        {{ t('sync.en_attente_vide') }}
      </p>

      <!-- COMPOSANT 08 · LIGNE DE LISTE, état « en attente d'envoi ». -->
      <ul
        v-else
        class="flex flex-col divide-y divide-line"
      >
        <li
          v-for="entree in enAttente"
          :key="entree.id"
          class="flex flex-col gap-1 py-3"
        >
          <div class="flex items-baseline gap-3">
            <span class="shrink-0 font-mono text-mini text-ink-3">
              {{ t('sync.saisie_le', { heure: heure(entree.horodatageClient) }) }}
            </span>
            <span
              v-if="entree.tentatives > 0"
              class="font-texte text-mini text-ink-3"
            >
              {{ t('sync.tentatives', { n: entree.tentatives }, entree.tentatives) }}
            </span>
          </div>
          <p class="text-corps text-ink">
            {{ (entree.charge as unknown as { texte?: string }).texte ?? entree.type }}
          </p>
        </li>
      </ul>
    </section>

    <!-- ─────────────────────────────────────────────────────────────────────────────────────
         SAISIES REFUSÉES — la quarantaine, consultable et actionnable
         ───────────────────────────────────────────────────────────────────────────────────── -->
    <section class="flex flex-col px-3 pt-5 pb-6">
      <h2 class="px-0.5 pb-1 text-etiquette uppercase text-ink-3">
        {{ t('sync.quarantaine.titre') }}
      </h2>

      <p
        v-if="refusees.length === 0"
        class="px-0.5 py-3 text-corps text-ink-2"
      >
        {{ t('sync.quarantaine.vide') }}
      </p>

      <ul
        v-else
        class="flex flex-col gap-3"
      >
        <li
          v-for="refus in refusees"
          :key="refus.entree.id"
          class="flex flex-col gap-2 rounded-r-lg border-l-4 border-l-danger bg-danger-soft p-3.5"
        >
          <div class="flex items-baseline gap-3">
            <span class="shrink-0 font-mono text-mini text-ink-3">
              {{ t('sync.quarantaine.refusee_le', { heure: heure(refus.refuseeLe) }) }}
            </span>
          </div>
          <p class="text-corps text-ink">
            {{ (refus.entree.charge as unknown as { texte?: string }).texte ?? refus.entree.type }}
          </p>
          <!-- Le motif, branché sur le `code` — jamais sur le `message`. -->
          <p class="text-corps text-danger-fort">
            {{ t(cleMotifRefus(refus.code)) }}
          </p>
          <!-- COMPOSANT 02 · BOUTON SECONDAIRE — le geste explicite de renvoyer. -->
          <button
            type="button"
            class="inline-flex h-9 w-fit cursor-pointer items-center gap-2 rounded-md border-[1.5px] border-line-2 bg-transparent px-3.5 font-titre text-mini font-semibold text-ink-2 transition-colors duration-90 hover:bg-tile"
            @click="relancer(refus.entree.id)"
          >
            <i
              class="ph ph-arrow-counter-clockwise text-corps"
              aria-hidden="true"
            />
            {{ t('sync.quarantaine.relancer') }}
          </button>
        </li>
      </ul>
    </section>
  </div>
</template>
