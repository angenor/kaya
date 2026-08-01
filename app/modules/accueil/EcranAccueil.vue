<script setup lang="ts">
/**
 * **`R1` — L'accueil.** Quatre comptes, quatre accueils, sur la même application.
 *
 * Référence visuelle — cas (a), **écran maquetté** : `docs/design/html/R1-accueil.html` et ses
 * trois états `-maquis`, `-proprietaire`, `-serveuse`. **Le HTML de maquette n'est jamais copié**
 * (porte P-19) : on en lit les valeurs — en-tête `h-16 px-5 bg-surf border-b`, pastille d'identité
 * `size-9 rounded-pleine bg-prim-soft`, tuiles `rounded-xl border border-line bg-surf` avec leur
 * pastille d'icône `size-9.5 rounded-lg bg-prim-soft` — et on réimplémente avec i18n, mode sombre
 * et filtrage par permission, que l'export ne contient pas.
 *
 * # Ce que cet écran NE montre pas, et pourquoi
 *
 * Les maquettes affichent des tables ouvertes, des chambres occupées, de l'argent non encaissé.
 * Ces chiffres viennent des cycles PDV, SEJ, CAI et HEB, **dont aucun n'est livré**. Les afficher
 * à zéro donnerait un accueil qui ment ; les afficher en dur donnerait un accueil qui ment
 * autrement. L'accueil du cycle 003 montre donc ce qui existe : les tuiles des trois écrans réels
 * du produit, filtrées par permission.
 *
 * C'est le principe X — « prêt ≠ construit ». La maquette reste la cible ; ce fichier en livre la
 * moitié qui a un contenu.
 *
 * # La règle qui décide de tout : ABSENCE, jamais grisé
 *
 * Une tuile dont l'utilisateur n'a pas la permission n'existe pas dans le HTML rendu. Le catalogue
 * et son filtre vivent dans `core/accueil/tuiles.ts` — pas ici : le filtrage est une règle du
 * produit, et un second appelant (la barre de navigation d'ETB-06) devra la reposer à l'identique.
 */
import { computed } from 'vue'

import { tuilesVisibles } from '~/core/accueil/tuiles'
import type { Permissions } from '~/core/rbac'

const { t } = useI18n()

const props = defineProps<{
  /** Nom affichable de l'utilisateur connecté — lu de `personne`, jamais son identifiant. */
  nomAffichage: string
  /** Permissions cumulées — union de ses rôles, jamais celles d'un rôle « principal ». */
  permissions: Permissions
  /** Codes des modules d'activité actifs. Vide est **valide** : c'est la résidence meublée. */
  modulesActifs: readonly string[]
}>()

const tuiles = computed(() => tuilesVisibles(props.permissions, props.modulesActifs))

/** L'initiale de la pastille d'identité. Décorative — le nom complet est à côté. */
const initiale = computed(() => props.nomAffichage.trim().charAt(0).toUpperCase() || '?')
</script>

<template>
  <main class="flex min-h-screen flex-col bg-bg">
    <header class="flex h-16 items-center gap-3.5 border-b border-line bg-surf px-5">
      <span class="inline-flex size-9 shrink-0 items-center justify-center rounded-pleine bg-prim-soft font-titre text-corps font-semibold text-prim">
        {{ initiale }}
      </span>
      <div class="flex min-w-0 flex-col gap-0.5">
        <span class="truncate font-titre text-corps font-semibold text-ink">
          {{ nomAffichage }}
        </span>
        <span class="text-mini text-ink-3">{{ t('accueil.sous_titre') }}</span>
      </div>
    </header>

    <div class="flex flex-1 flex-col gap-4 px-6 py-5.5">
      <h1 class="font-titre text-chiffre font-semibold text-ink">
        {{ t('accueil.titre') }}
      </h1>

      <!-- ÉTAT VIDE EXPLICITE — un compte sans aucun rôle. Ce n'est pas une erreur : il n'y a rien
           à réessayer, il y a quelqu'un à prévenir. Un écran blanc laisserait croire à une panne. -->
      <div
        v-if="tuiles.length === 0"
        class="flex flex-col items-start gap-2 rounded-xl border border-line bg-surf p-6"
      >
        <i
          class="ph ph-hand-waving text-titre-l text-ink-3"
          aria-hidden="true"
        />
        <p class="font-titre text-titre-s font-semibold text-ink">
          {{ t('accueil.vide.titre') }}
        </p>
        <p class="text-corps text-ink-2">
          {{ t('accueil.vide.explication') }}
        </p>
      </div>

      <ul
        v-else
        class="flex flex-col gap-2.5"
      >
        <li
          v-for="tuile in tuiles"
          :key="tuile.code"
        >
          <!-- Une seule occurrence par tuile, quelle que soit le nombre de rôles qui l'ouvrent
               (FR-027) : le catalogue est une liste de tuiles, pas une liste par rôle. -->
          <NuxtLink
            :to="tuile.route"
            :data-tuile="tuile.code"
            class="flex items-center gap-3.5 rounded-xl border border-line bg-surf px-4 py-3.5 transition-[transform,border-color] duration-90 ease-entree hover:translate-x-0.5 hover:border-line-2"
          >
            <span class="inline-flex size-9.5 shrink-0 items-center justify-center rounded-lg bg-prim-soft">
              <i
                class="ph text-titre-m text-prim"
                :class="tuile.icone"
                aria-hidden="true"
              />
            </span>
            <span class="flex min-w-0 flex-1 flex-col gap-0.5">
              <span class="font-titre text-action font-semibold text-ink">
                {{ t(tuile.libelleCle) }}
              </span>
              <span class="truncate text-mini text-ink-3">
                {{ t(tuile.descriptionCle) }}
              </span>
            </span>
            <i
              class="ph ph-caret-right shrink-0 text-corps text-ink-3"
              aria-hidden="true"
            />
          </NuxtLink>
        </li>
      </ul>
    </div>
  </main>
</template>
