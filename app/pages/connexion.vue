<script setup lang="ts">
/**
 * **`R0` — Connexion.** L'écran par lequel tout le monde entre.
 *
 * Référence visuelle — cas (b), **écran dérivé** : `docs/design/derivation.md` ligne
 * « `R0` Connexion **hérite de `G2`** — Formulaire minimal ; états d'erreur et vides de `S3` ».
 * L'écran n'existait dans **aucun** document jusqu'au 2026-08-01 : la matrice de dérivation a été
 * amendée d'abord (T001), et l'écran codé ensuite. Un écran absent des deux ne se code pas
 * (porte P-19).
 *
 * Valeurs lues de `G2`, jamais copiées : sur-étiquette `text-etiquette uppercase text-ink-3`,
 * titre `font-titre text-chiffre`, carte `rounded-xl border border-line bg-surf shadow-basse`,
 * bouton principal `h-13 rounded-xl bg-prim`.
 *
 * # Une seule phrase pour les deux échecs — FR-012
 *
 * Identifiant inconnu, mot de passe faux, compte désactivé, trop de tentatives : **la même**.
 * L'écran ne fait ici que rendre ce que `core/auth` a déjà décidé — la table de refus ne consulte
 * pas le code du serveur sur un `401`. Les deux niveaux sont testés : `auth-session.spec.ts` pour
 * la décision, `ecran-r0.spec.ts` pour la phrase rendue.
 *
 * # Le refus hors ligne est immédiat, et l'action DISPARAÎT
 *
 * Comme pour une opération de classe C : hors ligne ou en réseau dégradé, le bouton n'existe pas
 * dans le HTML rendu et **un bandeau dit pourquoi**, en une phrase. Jamais de grisé silencieux,
 * jamais d'échec après trente secondes d'attente (principe VI).
 *
 * Le `disabled` du bouton **pendant l'appel** est autre chose, et la distinction vaut d'être
 * écrite : il empêche une double soumission d'un geste en cours, il ne dissimule pas une
 * indisponibilité. C'est le grisé de « ça travaille », pas celui de « ce n'est pas pour vous ».
 *
 * # Aucun champ ne s'écrit à la main
 *
 * Les deux passent par **`ChampSaisie`**, composant 16 (`docs/design/composants.md`). Le second
 * emploie son type `mot_de_passe`, ajouté par ce cycle, et les deux portent leur `autocomplete` :
 * sans lui, les gestionnaires de mots de passe ne fonctionnent pas, et l'utilisateur choisit un
 * mot de passe qu'il retient — donc qu'il écrit sur un papier au comptoir.
 */
import { computed, ref } from 'vue'

import { ouvrirSession, REFUS_RESEAU } from '~/core/auth'
import ChampSaisie from '~/core/design-system/ChampSaisie.vue'
import { useEtatReseau } from '~/core/platform/reseau'

const { t } = useI18n()
const config = useRuntimeConfig()
const reseau = useEtatReseau()

const identifiant = ref('')
const motDePasse = ref('')

/** Erreurs **au champ** : elles portent sur ce qui est saisi, pas sur ce qui s'est passé. */
const erreurIdentifiant = ref<string | null>(null)
const erreurMotDePasse = ref<string | null>(null)

/**
 * L'appel en cours.
 *
 * Un booléen suffit **ici**, contrairement au patron d'ETB-02 qui porte le sens et la cible : il
 * n'y a qu'une opération sur cet écran, et ce qui arrive après elle n'est pas une ligne de liste
 * mais un changement d'écran. Le squelette occupe donc la forme du **bouton**, seul élément qui
 * change d'état.
 */
const enCours = ref(false)

/** Refus de la dernière tentative — clé i18n. Un seul, jamais empilé (composant 07). */
const refus = ref<string | null>(null)

/**
 * **Absence expliquée** (principe VI) : le réseau manque, l'action disparaît et un bandeau dit
 * pourquoi. L'état `degrade` compte comme hors ligne — `navigator.onLine` dit qu'une interface est
 * active, pas que le serveur répond.
 */
const reseauRequis = computed(() => reseau.value !== 'connecte')

/**
 * **Le bandeau affiché — un seul, jamais deux empilés.**
 *
 * Hors ligne gagne : il concerne l'écran entier là où un refus concerne une tentative, et il rend
 * cette tentative caduque — plus rien n'est actionnable tant que le réseau manque.
 */
const bandeau = computed<{ ton: 'alerte' | 'danger', cle: string } | null>(() => {
  if (reseauRequis.value) {
    return { ton: 'alerte', cle: REFUS_RESEAU }
  }
  return refus.value ? { ton: 'danger', cle: refus.value } : null
})

/** Ton → fond, contrefort, texte et icône. Deux tons ici, une seule structure (composant 07). */
const TONS = {
  alerte: { fond: 'border-l-alerte bg-alerte-soft', texte: 'text-alerte-fort', icone: 'ph-warning text-alerte' },
  danger: { fond: 'border-l-danger bg-danger-soft', texte: 'text-danger-fort', icone: 'ph-x-circle text-danger' },
} as const

/**
 * La session est ouverte mais **ne survivra pas au redémarrage**.
 *
 * Le cas arrive sur une plateforme sans stockage. Le dire est le seul comportement honnête : sans
 * cet avis, l'utilisateur découvrirait une déconnexion inexpliquée une heure plus tard.
 */
const sansPersistance = ref(false)

async function seConnecter(): Promise<void> {
  erreurIdentifiant.value = null
  erreurMotDePasse.value = null
  refus.value = null
  sansPersistance.value = false

  // **Validation au champ**, avant tout appel. Elle ne dit pas « le formulaire est invalide » :
  // elle se pose à côté de l'endroit où l'on corrige.
  if (!identifiant.value.trim()) {
    erreurIdentifiant.value = 'champ.erreur.obligatoire'
  }
  if (!motDePasse.value) {
    erreurMotDePasse.value = 'champ.erreur.obligatoire'
  }
  if (erreurIdentifiant.value || erreurMotDePasse.value) {
    return
  }

  enCours.value = true
  try {
    const issue = await ouvrirSession(
      config.public.apiBaseUrl,
      { identifiant: identifiant.value.trim(), motDePasse: motDePasse.value },
      reseau.value,
    )

    if (issue.issue === 'refus') {
      // **La clé vient de `core/auth`, jamais du `message` du serveur** — qui nomme des tables et
      // parle anglais technique.
      refus.value = issue.cle
      return
    }

    sansPersistance.value = !issue.persistante
    // Le mot de passe ne reste pas en mémoire une milliseconde de plus que nécessaire.
    motDePasse.value = ''
    await navigateTo('/')
  }
  catch {
    // Une coupure survenue **pendant** l'appel. La garde hors-ligne évite l'attente inutile, elle
    // ne remplace pas le traitement d'erreur (`core/platform/reseau.ts`).
    refus.value = 'connexion.refus.inattendue'
  }
  finally {
    enCours.value = false
  }
}
</script>

<template>
  <main class="flex min-h-screen items-center justify-center bg-bg p-6">
    <div class="w-full max-w-96">
      <header class="mb-6 flex flex-col gap-1">
        <p class="text-etiquette uppercase text-ink-3">
          {{ t('connexion.sur_titre') }}
        </p>
        <h1 class="font-titre text-chiffre font-semibold text-ink">
          {{ t('connexion.titre') }}
        </h1>
      </header>

      <!-- COMPOSANT 07 — contrefort de 4 px, fond `-soft`, texte `-fort`, une phrase.
           Un seul bandeau : `bandeau` est un `computed`, pas deux `v-if` qui se contrediraient. -->
      <div
        v-if="bandeau"
        class="mb-4 flex items-start gap-3 rounded-r-lg border-l-4 p-3.5"
        :class="TONS[bandeau.ton].fond"
        role="alert"
      >
        <i
          class="ph shrink-0 text-titre-s"
          :class="TONS[bandeau.ton].icone"
          aria-hidden="true"
        />
        <p
          class="font-texte text-corps"
          :class="TONS[bandeau.ton].texte"
        >
          {{ t(bandeau.cle) }}
        </p>
      </div>

      <form
        class="flex flex-col gap-4 rounded-xl border border-line bg-surf p-5 shadow-basse"
        @submit.prevent="seConnecter"
      >
        <ChampSaisie
          v-model="identifiant"
          etiquette-cle="connexion.champ.identifiant"
          aide-cle="connexion.champ.identifiant_aide"
          :erreur-cle="erreurIdentifiant"
          autocompletion="username"
          taille="comptoir"
          requis
        />

        <ChampSaisie
          v-model="motDePasse"
          etiquette-cle="connexion.champ.mot_de_passe"
          type="mot_de_passe"
          :erreur-cle="erreurMotDePasse"
          autocompletion="current-password"
          taille="comptoir"
          requis
        />

        <!-- HORS LIGNE : le bouton n'est pas grisé, il est ABSENT. Le bandeau ci-dessus dit
             pourquoi. Griser apprendrait à l'utilisateur qu'une partie du produit lui est
             refusée, sans lui dire laquelle ni jusqu'à quand (principe VII). -->
        <button
          v-if="!reseauRequis"
          type="submit"
          class="inline-flex h-13 w-full cursor-pointer items-center justify-center gap-2.5 rounded-xl bg-prim font-titre text-titre-s font-semibold text-prim-ink shadow-bouton-grand transition-[transform,box-shadow] duration-90 ease-entree active:translate-y-0.75 active:shadow-none"
          :disabled="enCours"
        >
          <!-- COMPOSANT 13 — le squelette occupe la forme exacte de ce qui revient : le libellé
               du bouton. Une roue au milieu de l'écran ne dirait pas où l'attente se résout. -->
          <span
            v-if="enCours"
            class="h-3.5 w-32 animate-pulse rounded-full bg-prim-ink/40"
            aria-hidden="true"
          />
          <template v-else>
            <i
              class="ph ph-sign-in text-titre-s"
              aria-hidden="true"
            />
            {{ t('connexion.action') }}
          </template>
          <span class="sr-only">{{ enCours ? t('connexion.en_cours') : '' }}</span>
        </button>
      </form>

      <p
        v-if="sansPersistance"
        class="mt-4 font-texte text-mini text-ink-3"
      >
        {{ t('connexion.sans_persistance') }}
      </p>
    </div>
  </main>
</template>
