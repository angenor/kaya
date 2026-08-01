<script setup lang="ts">
/**
 * **Le styleguide servi par l'application** — les seize composants canoniques, tous leurs états, en
 * clair et en sombre, **avec les polices réellement embarquées**.
 *
 * # Pourquoi `docs/design/styleguide.html` ne pouvait pas rendre ce service
 *
 * Constat vérifié, lignes 7 à 10 de ce fichier : il charge Archivo et Chivo Mono depuis Google
 * Fonts, et Phosphor depuis unpkg. C'est **normal** — c'est la maquette, autonome par construction
 * (principe XII). Mais la conséquence l'est aussi : il affichera **toujours** les vraies polices,
 * y compris le jour où l'application, elle, tombe en repli. Il ne peut donc pas valider ce que le
 * volet précédent vient d'embarquer.
 *
 * Cette page-ci est servie par l'application, avec `polices.css`, `theme.css` et `icones.css` —
 * les fichiers réels. Si un `@font-face` est cassé, si U+202F retombe en repli, si un glyphe manque,
 * **ça se voit ici et nulle part ailleurs** : ni harfbuzz, ni P-21b, ni un test happy-dom ne
 * prouvent qu'un caractère s'affiche à la bonne largeur. Ils prouvent qu'il est dans le fichier.
 *
 * # Elle n'est PAS montée en production
 *
 * `core/design-system/montage.ts`, et le hook `pages:extend` de `nuxt.config.ts` : la route est
 * **retirée du routeur** quand `KAYA_STYLEGUIDE` n'est pas posée. Même mécanisme que la Swagger UI
 * du cycle 001, et même raison — « une route non montée ne peut pas fuir par oubli de garde ».
 *
 *     KAYA_STYLEGUIDE=1 pnpm --filter @kaya/app dev     # puis /styleguide
 *
 * # Les libellés d'échantillon sont en clair, et c'est une exemption écrite
 *
 * `scripts/test-i18n.ts` (porte P-16) **exempte nommément ce fichier**, avec sa contrepartie : la
 * porte vérifie que la page exemptée est bien celle qui n'est pas montée en production. Deux
 * raisons, dans cet ordre :
 *
 * 1. Les catalogues `fr.json` et `en.json` sont **livrés dans le paquet de production**. Y verser
 *    cent cinquante clés de vocabulaire de design — « Repos », « Survol », « Appui », « Losange » —
 *    ferait voyager du texte mort dans chaque installation, pour une page que personne n'ouvrira.
 * 2. Ce ne sont pas des chaînes **produit** : ce sont les noms d'états de `docs/design/composants.md`.
 *    Les traduire n'aurait pas de sens ; les maintenir à parité fr/en pour toujours, encore moins.
 *
 * Les composants, eux, reçoivent bien des **clés réelles du catalogue** — `ChampSaisie` n'accepte
 * que ça, et c'est ce qui prouve qu'il fonctionne pour de vrai ici.
 */
import { ref } from 'vue'

import ChampSaisie from '~/core/design-system/ChampSaisie.vue'
import VitrineTheme from '~/core/design-system/VitrineTheme.vue'
import { formaterMontant } from '~/core/format/montant'

/** Devise du pilote — Résidence Hôtel Deloria, Abengourou. Zéro décimale. */
const DEVISE = 'XOF'

/**
 * **Les quatre largeurs de la section « montants ».**
 *
 * Choisies pour que le désalignement se voie s'il existe : quatre, cinq, six puis sept chiffres,
 * donc un, un, un puis deux séparateurs. Une colonne de montants tous à la même largeur ne prouve
 * rien — c'est le passage de `1 500` à `1 250 000` qui met la chasse tabulaire à l'épreuve.
 */
const COLONNE_MONTANTS = [1_500, 12_500, 150_000, 1_250_000]

/** L'échelle typographique de `tokens.md` §2, dans l'ordre du tableau. */
const ECHELLE = [
  { classe: 'text-etiquette', jeton: '--text-etiquette', corps: '11 px', usage: 'Étiquette en capitales' },
  { classe: 'text-mini', jeton: '--text-mini', corps: '12,5 px', usage: 'Méta, bouton discret, pastille' },
  { classe: 'text-corps', jeton: '--text-corps', corps: '13,5 px', usage: 'Corps de texte — taille du body' },
  { classe: 'text-action', jeton: '--text-action', corps: '14,5 px', usage: 'Libellé de bouton — ne descend jamais plus bas' },
  { classe: 'text-lead', jeton: '--text-lead', corps: '15 px', usage: 'Chapeau, pastille de durée tactile' },
  { classe: 'text-titre-s', jeton: '--text-titre-s', corps: '17 px', usage: 'Titre de carte' },
  { classe: 'text-titre-m', jeton: '--text-titre-m', corps: '20 px', usage: 'Titre d’écran' },
  { classe: 'text-chiffre', jeton: '--text-chiffre', corps: '24 px', usage: 'Montant en ligne, total' },
  { classe: 'text-chiffre-l', jeton: '--text-chiffre-l', corps: '30 px', usage: 'Chiffre de carte' },
] as const

/** Les six formes du vocabulaire d'état — composant 04. Forme **plus** couleur, jamais la couleur seule. */
const FORMES = [
  { marque: 'size-2 bg-succes rotate-45', fond: 'bg-succes-soft', texte: 'text-succes-fort', libelle: 'Payé', forme: 'Losange — acquis, terminé' },
  { marque: 'size-2 rounded-pleine bg-alerte', fond: 'bg-alerte-soft', texte: 'text-alerte-fort', libelle: 'Partiel', forme: 'Rond plein — en cours, occupé' },
  { marque: 'size-2 rounded-xs bg-succes', fond: 'bg-succes-soft', texte: 'text-succes-fort', libelle: 'Libre', forme: 'Carré — libre, disponible' },
  { marque: 'size-2 bg-danger [clip-path:polygon(50%_0,100%_100%,0_100%)]', fond: 'bg-danger-soft', texte: 'text-danger-fort', libelle: 'Impayé', forme: 'Triangle — cassé, impayé' },
  { marque: 'size-2 rounded-pleine border-2 border-ink-3', fond: 'bg-tile', texte: 'text-ink-2', libelle: 'Hors ligne', forme: 'Cercle vide — non applicable' },
  { marque: 'size-3 rounded-pleine border-2 border-info/30 border-t-info animate-roue', fond: 'bg-info-soft', texte: 'text-info-fort', libelle: 'Envoi…', forme: 'Roue — attente indéterminée' },
] as const

/** Les seize sections, pour la navigation d'ancres. */
const SOMMAIRE = [
  { ancre: 'montants', numero: '§', titre: 'Montants' },
  { ancre: 'typographie', numero: '§', titre: 'Typographie' },
  { ancre: 'c1', numero: '01', titre: 'Bouton principal' },
  { ancre: 'c2', numero: '02', titre: 'Bouton secondaire' },
  { ancre: 'c3', numero: '03', titre: 'Bouton discret' },
  { ancre: 'c4', numero: '04', titre: 'Pastille d’état' },
  { ancre: 'c5', numero: '05', titre: 'Tuile d’action' },
  { ancre: 'c6', numero: '06', titre: 'Carte chiffre' },
  { ancre: 'c7', numero: '07', titre: 'Bandeau d’alerte' },
  { ancre: 'c8', numero: '08', titre: 'Ligne de liste' },
  { ancre: 'c9', numero: '09', titre: 'Sélecteur d’établissement' },
  { ancre: 'c10', numero: '10', titre: 'Témoin de synchronisation' },
  { ancre: 'c11', numero: '11', titre: 'État vide illustré' },
  { ancre: 'c12', numero: '12', titre: 'Sélecteur segmenté' },
  { ancre: 'c13', numero: '13', titre: 'Squelette de chargement' },
  { ancre: 'c14', numero: '14', titre: 'Bandeau d’annulation' },
  { ancre: 'c15', numero: '15', titre: 'Barre de proportion' },
  { ancre: 'c16', numero: '16', titre: 'Champ de saisie' },
] as const

/**
 * Les modèles du composant 16.
 *
 * **Partagés par les deux volets de la vitrine** : le slot est rendu deux fois, donc taper dans le
 * champ clair met à jour le champ sombre. C'est voulu — on compare deux apparences du même état,
 * pas deux états.
 */
const saisieRepos = ref('')
const saisieRemplie = ref('Résidence Hôtel Deloria')
const saisieErreur = ref('Maquis')
const saisieChoix = ref('BAR')

/** Trois clés réelles du catalogue — celles que `SectionServices.vue` passe déjà au composant. */
const CLE_ETIQUETTE = 'etablissement.services.formulaire.champ'
const CLE_AIDE = 'etablissement.services.formulaire.aide'
const CLE_INVITE = 'etablissement.services.formulaire.invite'
const CLE_ERREUR = 'etablissement.services.refus.module_inconnu'

const OPTIONS_SERVICE = [
  { valeur: 'BAR', libelleCle: 'services.modules.BAR' },
  { valeur: 'PRESSING', libelleCle: 'services.modules.PRESSING' },
  { valeur: 'RESTAURATION', libelleCle: 'services.modules.RESTAURATION' },
]
</script>

<template>
  <div class="min-h-screen bg-bg font-texte text-corps text-ink">
    <!-- ══ En-tête et sommaire ══════════════════════════════════════════════════════════════ -->
    <header class="sticky top-0 z-10 border-b border-line bg-surf/95 backdrop-blur">
      <div class="mx-auto flex max-w-7xl flex-col gap-3 px-8 py-5">
        <div class="flex flex-wrap items-baseline gap-3">
          <h1 class="m-0 font-titre text-titre-m font-semibold">
            Kaya — styleguide servi par l’application
          </h1>
          <span class="font-mono text-mini text-ink-3">
            polices embarquées · icônes sous-réglées · U+202F
          </span>
        </div>
        <p class="m-0 max-w-prose text-mini text-ink-2">
          Cette page n’est pas montée en production. Elle existe pour voir les seize composants avec
          les polices réellement servies — ce qu’aucun test ne peut vérifier.
        </p>
        <nav class="flex flex-wrap gap-1.5">
          <a
            v-for="entree in SOMMAIRE"
            :key="entree.ancre"
            :href="`#${entree.ancre}`"
            class="h-9 rounded-md px-3 font-titre text-mini font-semibold text-ink-2 inline-flex items-center gap-1.5 transition-colors duration-90 hover:bg-tile hover:text-ink"
          >
            <span class="font-mono text-ink-3">{{ entree.numero }}</span>{{ entree.titre }}
          </a>
        </nav>
      </div>
    </header>

    <main class="mx-auto flex max-w-7xl flex-col gap-12 px-8 py-10">
      <!-- ══ MONTANTS ══════════════════════════════════════════════════════════════════════ -->
      <section
        id="montants"
        class="flex scroll-mt-6 flex-col gap-3.5"
      >
        <div class="flex items-baseline gap-3">
          <span class="font-mono text-mini text-ink-3">§</span>
          <h2 class="m-0 font-titre text-titre-m font-semibold">
            Montants
          </h2>
          <span class="h-px flex-1 bg-line" />
          <span class="font-mono text-mini text-ink-3">U+202F · Chivo Mono tabulaire</span>
        </div>
        <p class="m-0 max-w-prose text-corps text-ink-2">
          Une seule fonction porte la règle — <code class="font-mono text-mini">core/format/montant.ts</code>.
          Le séparateur est l’espace fine insécable U+202F, absente d’Archivo et de Chivo Mono à la
          source et ajoutée à leur table cmap. Ce qui se vérifie ici et nulle part ailleurs : que la
          fine est visiblement plus étroite qu’une espace mot, que le montant ne se coupe pas, et
          que la colonne s’aligne au chiffre près.
        </p>

        <VitrineTheme
          libelle-clair="Clair"
          libelle-sombre="Sombre"
        >
          <div class="flex flex-col gap-2">
            <span class="text-etiquette text-ink-3 uppercase">
              La colonne — quatre largeurs, alignée à droite
            </span>
            <div class="w-fit rounded-xl border border-line bg-surf">
              <div
                v-for="montant in COLONNE_MONTANTS"
                :key="montant"
                class="flex h-14 items-center gap-6 border-b border-line px-4 last:border-b-0"
              >
                <span class="w-32 text-corps text-ink-2">Chambre {{ 100 + COLONNE_MONTANTS.indexOf(montant) + 1 }}</span>
                <span class="w-36 text-right font-mono text-corps font-bold whitespace-nowrap text-ink">
                  {{ formaterMontant(montant, DEVISE) }}
                </span>
              </div>
              <!-- Le total garde le CORPS de la colonne, pas un corps plus grand : à 24 px la
                   chasse de Chivo Mono n'est plus 8,1 px mais 14,4, et le total cesserait de
                   s'aligner sur les montants au-dessus. C'est un vrai piège de composition, pas
                   une contrainte du styleguide — le composant 08 fait le même choix. -->
              <div class="flex h-14 items-center gap-6 border-t-2 border-line-2 px-4">
                <span class="w-32 font-titre text-corps font-semibold text-ink">Total</span>
                <span class="w-36 text-right font-mono text-corps font-bold whitespace-nowrap text-ink">
                  {{ formaterMontant(COLONNE_MONTANTS.reduce((s, m) => s + m, 0), DEVISE) }}
                </span>
              </div>
            </div>
          </div>

          <div class="flex flex-col gap-2">
            <span class="text-etiquette text-ink-3 uppercase">
              La fine contre l’espace mot — en Archivo, la seule où elle se voie
            </span>
            <div class="flex flex-col gap-1 rounded-xl border border-line bg-surf p-4">
              <span class="font-texte text-chiffre whitespace-nowrap text-ink">{{ formaterMontant(12_500, DEVISE) }}</span>
              <span class="font-texte text-chiffre text-ink-3">12 500 F</span>
              <span class="max-w-prose text-mini text-ink-3">
                En haut, U+202F ; en bas, l’espace ordinaire. En Archivo, la fine fait 193 unités
                contre 209 — la ligne du haut est donc <strong>un peu</strong> plus courte. Si les
                deux ont exactement la même largeur, U+202F est tombée en repli.
              </span>
            </div>
            <div class="flex flex-col gap-1 rounded-xl border border-line bg-surf p-4">
              <span class="font-mono text-chiffre whitespace-nowrap text-ink">{{ formaterMontant(12_500, DEVISE) }}</span>
              <span class="font-mono text-chiffre text-ink-3">12 500 F</span>
              <span class="max-w-prose text-mini text-ink-3">
                Les mêmes en Chivo Mono, et ici les deux largeurs sont
                <strong>identiques — c’est voulu</strong>. En monospace toutes les chasses valent la
                cellule, y compris celle de la fine : c’est exactement ce qui tient l’alignement de
                la colonne ci-dessus. La fine ne se voit pas dans les chiffres ; elle s’y compte.
              </span>
            </div>
          </div>

          <div class="flex flex-col gap-2">
            <span class="text-etiquette text-ink-3 uppercase">
              Insécabilité — un conteneur trop étroit pour la ligne
            </span>
            <div class="w-40 rounded-xl border border-line bg-surf p-3">
              <p class="m-0 font-mono text-corps text-ink">
                Reste à payer {{ formaterMontant(1_250_000, DEVISE) }} avant le départ
              </p>
            </div>
            <span class="text-mini text-ink-3">
              Le montant doit passer à la ligne d’un bloc, jamais se couper entre ses groupes.
            </span>
          </div>

          <div class="flex flex-col gap-2">
            <span class="text-etiquette text-ink-3 uppercase">L’échelle du comptoir</span>
            <div class="flex flex-wrap items-end gap-6">
              <span class="font-mono text-montant font-bold whitespace-nowrap text-ink">{{ formaterMontant(184_000, DEVISE) }}</span>
              <span class="font-mono text-recette font-bold whitespace-nowrap text-ink">{{ formaterMontant(12_500, DEVISE) }}</span>
            </div>
          </div>
        </VitrineTheme>
      </section>

      <!-- ══ TYPOGRAPHIE ═══════════════════════════════════════════════════════════════════ -->
      <section
        id="typographie"
        class="flex scroll-mt-6 flex-col gap-3.5"
      >
        <div class="flex items-baseline gap-3">
          <span class="font-mono text-mini text-ink-3">§</span>
          <h2 class="m-0 font-titre text-titre-m font-semibold">
            Typographie
          </h2>
          <span class="h-px flex-1 bg-line" />
          <span class="font-mono text-mini text-ink-3">Archivo · demi-valeurs</span>
        </div>
        <p class="m-0 max-w-prose text-corps text-ink-2">
          Les demi-valeurs de <code class="font-mono text-mini">tokens.md</code> — 13,5 px de corps,
          14,5 px d’action, 12,5 px de méta — ont été réglées pour Archivo, « pour tenir le 13 px
          lisible sur un 1366 × 768 délavé par le soleil ». C’est le premier endroit où elles se
          voient dans la police pour laquelle elles ont été choisies.
        </p>

        <VitrineTheme
          libelle-clair="Clair"
          libelle-sombre="Sombre"
        >
          <div class="flex flex-col gap-3">
            <div
              v-for="niveau in ECHELLE"
              :key="niveau.jeton"
              class="flex flex-wrap items-baseline gap-4 border-b border-line pb-2 last:border-b-0"
            >
              <span class="w-28 font-mono text-mini text-ink-3">{{ niveau.corps }}</span>
              <span
                class="flex-1 font-texte text-ink"
                :class="niveau.classe"
              >Résidence Hôtel Deloria — Abengourou</span>
              <span class="font-mono text-mini text-ink-3">{{ niveau.jeton }}</span>
            </div>
          </div>

          <div class="flex flex-col gap-2">
            <span class="text-etiquette text-ink-3 uppercase">
              Le latin étendu — ce que « latin » seul ne dessine pas
            </span>
            <p class="m-0 font-texte text-corps text-ink">
              cœur sœur œuvre · Ÿ Ō Š ẞ · À Ç É È Ù Û Î Ô Ë Ï Ü Ÿ · Haïti maïs Abengourou
            </p>
            <p class="m-0 font-mono text-corps text-ink">
              cœur sœur œuvre · Ÿ Ō Š ẞ · 0123456789
            </p>
            <span class="text-mini text-ink-3">
              Un caractère plus large ou d’un autre dessin est un caractère tombé en repli.
            </span>
          </div>
        </VitrineTheme>
      </section>

      <!-- ══ 01 BOUTON PRINCIPAL ═══════════════════════════════════════════════════════════ -->
      <section
        id="c1"
        class="flex scroll-mt-6 flex-col gap-3.5"
      >
        <div class="flex items-baseline gap-3">
          <span class="font-mono text-mini text-ink-3">01</span>
          <h2 class="m-0 font-titre text-titre-m font-semibold">
            Bouton principal
          </h2>
          <span class="h-px flex-1 bg-line" />
          <span class="font-mono text-mini text-ink-3">h-11 · indigo plein · ombre 2 px</span>
        </div>
        <p class="m-0 max-w-prose text-corps text-ink-2">
          Un seul par écran. L’ombre pleine tombe à l’appui — seul relief du système, et seul
          mouvement jamais réduit.
        </p>

        <VitrineTheme
          libelle-clair="Clair"
          libelle-sombre="Sombre"
        >
          <div class="flex flex-wrap items-end gap-4">
            <div class="flex flex-col gap-2">
              <span class="text-etiquette text-ink-3 uppercase">Repos</span>
              <button
                type="button"
                class="h-11 min-w-42 cursor-pointer rounded-lg bg-prim px-5 font-titre text-action font-semibold text-prim-ink shadow-bouton transition-[transform,box-shadow,background-color] duration-90 ease-entree hover:bg-prim-dk active:translate-y-0.5 active:shadow-bouton-appui"
              >
                Enregistrer l’arrivée
              </button>
            </div>
            <div class="flex flex-col gap-2">
              <span class="text-etiquette text-ink-3 uppercase">Survol</span>
              <button
                type="button"
                class="h-11 min-w-42 cursor-pointer rounded-lg bg-prim-dk px-5 font-titre text-action font-semibold text-prim-ink shadow-bouton"
              >
                Enregistrer l’arrivée
              </button>
            </div>
            <div class="flex flex-col gap-2">
              <span class="text-etiquette text-ink-3 uppercase">Appui</span>
              <button
                type="button"
                class="h-11 min-w-42 translate-y-0.5 cursor-pointer rounded-lg bg-prim px-5 font-titre text-action font-semibold text-prim-ink shadow-bouton-appui"
              >
                Enregistrer l’arrivée
              </button>
            </div>
          </div>
          <div class="flex flex-wrap items-end gap-4">
            <div class="flex flex-col gap-2">
              <span class="text-etiquette text-ink-3 uppercase">Focus clavier</span>
              <button
                type="button"
                class="h-11 min-w-42 cursor-pointer rounded-lg bg-prim px-5 font-titre text-action font-semibold text-prim-ink shadow-bouton outline-2 outline-offset-2 outline-prim"
              >
                Enregistrer l’arrivée
              </button>
            </div>
            <div class="flex flex-col gap-2">
              <span class="text-etiquette text-ink-3 uppercase">En cours</span>
              <button
                type="button"
                disabled
                class="h-11 min-w-42 inline-flex items-center justify-center gap-2.5 rounded-lg bg-prim px-5 font-titre text-action font-semibold text-prim-ink"
              >
                <span class="size-4 rounded-pleine border-2 border-prim-ink/35 border-t-prim-ink animate-roue" />Envoi…
              </button>
            </div>
            <div class="flex flex-col gap-2">
              <span class="text-etiquette text-ink-3 uppercase">Désactivé</span>
              <button
                type="button"
                disabled
                class="h-11 min-w-42 rounded-lg bg-prim px-5 font-titre text-action font-semibold text-prim-ink"
              >
                Enregistrer l’arrivée
              </button>
            </div>
          </div>
          <div class="flex flex-col gap-2">
            <span class="text-etiquette text-ink-3 uppercase">Pleine largeur · comptoir · h-12</span>
            <button
              type="button"
              class="h-12 w-full max-w-md cursor-pointer rounded-lg bg-prim px-5 font-titre text-action font-semibold text-prim-ink shadow-bouton transition-[transform,box-shadow] duration-90 ease-entree hover:bg-prim-dk active:translate-y-0.5 active:shadow-bouton-appui"
            >
              Encaisser {{ formaterMontant(12_500, DEVISE) }}
            </button>
          </div>
          <div class="flex flex-col gap-2">
            <span class="text-etiquette text-ink-3 uppercase">Variante danger · irréversible</span>
            <button
              type="button"
              class="h-11 min-w-42 cursor-pointer rounded-lg bg-danger px-5 font-titre text-action font-semibold text-prim-ink shadow-bouton-danger transition-transform duration-90 ease-entree active:translate-y-0.5 active:shadow-none"
            >
              Annuler la note
            </button>
          </div>
        </VitrineTheme>
      </section>

      <!-- ══ 02 BOUTON SECONDAIRE ══════════════════════════════════════════════════════════ -->
      <section
        id="c2"
        class="flex scroll-mt-6 flex-col gap-3.5"
      >
        <div class="flex items-baseline gap-3">
          <span class="font-mono text-mini text-ink-3">02</span>
          <h2 class="m-0 font-titre text-titre-m font-semibold">
            Bouton secondaire
          </h2>
          <span class="h-px flex-1 bg-line" />
          <span class="font-mono text-mini text-ink-3">h-11 · contour indigo · fond transparent</span>
        </div>
        <p class="m-0 max-w-prose text-corps text-ink-2">
          L’issue à côté du bouton principal. Jamais deux côte à côte : si deux sorties existent,
          l’une est un bouton discret.
        </p>

        <VitrineTheme
          libelle-clair="Clair"
          libelle-sombre="Sombre"
        >
          <div class="flex flex-wrap items-end gap-4">
            <div class="flex flex-col gap-2">
              <span class="text-etiquette text-ink-3 uppercase">Repos</span>
              <button
                type="button"
                class="h-11 min-w-32 cursor-pointer rounded-lg border-[1.5px] border-prim bg-transparent px-4.5 font-titre text-action font-semibold text-prim transition-colors duration-90 hover:bg-prim-soft active:translate-y-0.5"
              >
                Annuler
              </button>
            </div>
            <div class="flex flex-col gap-2">
              <span class="text-etiquette text-ink-3 uppercase">Survol</span>
              <button
                type="button"
                class="h-11 min-w-32 cursor-pointer rounded-lg border-[1.5px] border-prim bg-prim-soft px-4.5 font-titre text-action font-semibold text-prim"
              >
                Annuler
              </button>
            </div>
            <div class="flex flex-col gap-2">
              <span class="text-etiquette text-ink-3 uppercase">Désactivé</span>
              <button
                type="button"
                disabled
                class="h-11 min-w-32 rounded-lg border-[1.5px] border-prim bg-transparent px-4.5 font-titre text-action font-semibold text-prim"
              >
                Annuler
              </button>
            </div>
            <div class="flex flex-col gap-2">
              <span class="text-etiquette text-ink-3 uppercase">Avec icône</span>
              <button
                type="button"
                class="h-11 min-w-32 inline-flex cursor-pointer items-center justify-center gap-2 rounded-lg border-[1.5px] border-prim bg-transparent px-4.5 font-titre text-action font-semibold text-prim hover:bg-prim-soft"
              >
                <i
                  class="ph ph-printer"
                  aria-hidden="true"
                />Imprimer
              </button>
            </div>
            <div class="flex flex-col gap-2">
              <span class="text-etiquette text-ink-3 uppercase">Neutre — pas une action produit</span>
              <button
                type="button"
                class="h-11 min-w-32 cursor-pointer rounded-lg border-[1.5px] border-line-2 bg-transparent px-4.5 font-titre text-action font-semibold text-ink-2 hover:bg-tile"
              >
                Plus tard
              </button>
            </div>
          </div>
        </VitrineTheme>
      </section>

      <!-- ══ 03 BOUTON DISCRET ═════════════════════════════════════════════════════════════ -->
      <section
        id="c3"
        class="flex scroll-mt-6 flex-col gap-3.5"
      >
        <div class="flex items-baseline gap-3">
          <span class="font-mono text-mini text-ink-3">03</span>
          <h2 class="m-0 font-titre text-titre-m font-semibold">
            Bouton discret
          </h2>
          <span class="h-px flex-1 bg-line" />
          <span class="font-mono text-mini text-ink-3">h-9 · sans fond ni contour · size-11 en icône seule</span>
        </div>
        <p class="m-0 max-w-prose text-corps text-ink-2">
          Les actions de bord. En icône seule il passe à 44 px même s’il paraît plus petit.
        </p>

        <VitrineTheme
          libelle-clair="Clair"
          libelle-sombre="Sombre"
        >
          <div class="flex flex-wrap items-end gap-4">
            <div class="flex flex-col gap-2">
              <span class="text-etiquette text-ink-3 uppercase">Repos</span>
              <button
                type="button"
                class="h-9 cursor-pointer rounded-md px-3.5 font-titre text-mini font-semibold text-ink-2 transition-colors duration-90 hover:bg-tile hover:text-ink active:scale-97"
              >
                Modifier
              </button>
            </div>
            <div class="flex flex-col gap-2">
              <span class="text-etiquette text-ink-3 uppercase">Survol</span>
              <button
                type="button"
                class="h-9 cursor-pointer rounded-md bg-tile px-3.5 font-titre text-mini font-semibold text-ink"
              >
                Modifier
              </button>
            </div>
            <div class="flex flex-col gap-2">
              <span class="text-etiquette text-ink-3 uppercase">Filtre actif</span>
              <button
                type="button"
                class="h-9 cursor-pointer rounded-md bg-prim-soft px-3.5 font-titre text-mini font-semibold text-prim"
              >
                Impayées
              </button>
            </div>
            <div class="flex flex-col gap-2">
              <span class="text-etiquette text-ink-3 uppercase">Danger</span>
              <button
                type="button"
                class="h-9 cursor-pointer rounded-md px-3.5 font-titre text-mini font-semibold text-danger-fort hover:bg-danger-soft"
              >
                Supprimer
              </button>
            </div>
            <div class="flex flex-col gap-2">
              <span class="text-etiquette text-ink-3 uppercase">Désactivé</span>
              <button
                type="button"
                disabled
                class="h-9 rounded-md px-3.5 font-titre text-mini font-semibold text-ink-2"
              >
                Modifier
              </button>
            </div>
          </div>
          <div class="flex flex-wrap items-end gap-4">
            <div class="flex flex-col gap-2">
              <span class="text-etiquette text-ink-3 uppercase">Icône seule — 44 px</span>
              <div class="flex gap-2">
                <button
                  type="button"
                  class="size-11 inline-flex cursor-pointer items-center justify-center rounded-md text-titre-s text-ink-2 hover:bg-tile hover:text-ink"
                >
                  <i
                    class="ph ph-funnel"
                    aria-hidden="true"
                  />
                </button>
                <button
                  type="button"
                  class="size-11 inline-flex cursor-pointer items-center justify-center rounded-md bg-tile text-titre-s text-ink"
                >
                  <i
                    class="ph ph-magnifying-glass"
                    aria-hidden="true"
                  />
                </button>
                <button
                  type="button"
                  class="size-11 inline-flex cursor-pointer items-center justify-center rounded-md bg-prim-soft text-titre-s text-prim"
                >
                  <i
                    class="ph ph-arrows-down-up"
                    aria-hidden="true"
                  />
                </button>
              </div>
            </div>
            <div class="flex flex-col gap-2">
              <span class="text-etiquette text-ink-3 uppercase">Lien de retour</span>
              <button
                type="button"
                class="h-9 inline-flex cursor-pointer items-center gap-1.5 rounded-md px-3.5 font-titre text-mini font-semibold text-ink-2 hover:text-ink"
              >
                <i
                  class="ph ph-arrow-left"
                  aria-hidden="true"
                />Retour au registre
              </button>
            </div>
          </div>
        </VitrineTheme>
      </section>

      <!-- ══ 04 PASTILLE D'ÉTAT ════════════════════════════════════════════════════════════ -->
      <section
        id="c4"
        class="flex scroll-mt-6 flex-col gap-3.5"
      >
        <div class="flex items-baseline gap-3">
          <span class="font-mono text-mini text-ink-3">04</span>
          <h2 class="m-0 font-titre text-titre-m font-semibold">
            Pastille d’état
          </h2>
          <span class="h-px flex-1 bg-line" />
          <span class="font-mono text-mini text-ink-3">h-7 · forme + couleur, jamais la couleur seule</span>
        </div>
        <p class="m-0 max-w-prose text-corps text-ink-2">
          Le vocabulaire de formes est fixe et vaut pour tout le produit. Sur un écran délavé par le
          soleil — et pour un daltonien — c’est la forme qui porte l’état.
        </p>

        <VitrineTheme
          libelle-clair="Clair"
          libelle-sombre="Sombre"
        >
          <div class="flex flex-col gap-3">
            <div
              v-for="etat in FORMES"
              :key="etat.libelle"
              class="flex flex-wrap items-center gap-4"
            >
              <span
                class="h-7 inline-flex items-center gap-1.5 rounded-pleine pr-2.5 pl-2 text-mini font-semibold"
                :class="[etat.fond, etat.texte]"
              >
                <span :class="etat.marque" />{{ etat.libelle }}
              </span>
              <span class="text-mini text-ink-3">{{ etat.forme }}</span>
            </div>
          </div>
          <div class="flex flex-col gap-2">
            <span class="text-etiquette text-ink-3 uppercase">Variante contour — sur bg-tile</span>
            <div class="flex flex-wrap gap-2 rounded-lg bg-tile p-3">
              <span class="h-7 inline-flex items-center gap-1.5 rounded-pleine border border-succes pr-2.5 pl-2 text-mini font-semibold text-succes-fort">
                <span class="size-2 rotate-45 bg-succes" />Payé
              </span>
              <span class="h-7 inline-flex items-center gap-1.5 rounded-pleine border border-occupe pr-2.5 pl-2 text-mini font-semibold text-occupe-fort">
                <span class="size-2 rounded-pleine bg-occupe" />Occupée
              </span>
              <span class="h-7 inline-flex items-center gap-1.5 rounded-pleine border border-alerte pr-2.5 pl-2 text-mini font-semibold text-alerte-fort">
                <span class="size-2 rounded-pleine bg-alerte" />À nettoyer
              </span>
            </div>
          </div>
        </VitrineTheme>
      </section>

      <!-- ══ 05 TUILE D'ACTION ═════════════════════════════════════════════════════════════ -->
      <section
        id="c5"
        class="flex scroll-mt-6 flex-col gap-3.5"
      >
        <div class="flex items-baseline gap-3">
          <span class="font-mono text-mini text-ink-3">05</span>
          <h2 class="m-0 font-titre text-titre-m font-semibold">
            Tuile d’action
          </h2>
          <span class="h-px flex-1 bg-line" />
          <span class="font-mono text-mini text-ink-3">min-h-28 · surface entièrement cliquable</span>
        </div>
        <p class="m-0 max-w-prose text-corps text-ink-2">
          Le compteur ne s’affiche que s’il y a du travail en attente, jamais à zéro. Désactivée,
          elle dit <em>pourquoi</em>.
        </p>

        <VitrineTheme
          libelle-clair="Clair"
          libelle-sombre="Sombre"
        >
          <div class="grid grid-cols-1 gap-4 sm:grid-cols-3">
            <button
              type="button"
              class="min-h-28 flex cursor-pointer flex-col items-start justify-between rounded-lg border border-line bg-surf p-4 text-left shadow-basse transition-[transform,border-color] duration-90 ease-entree hover:border-prim active:scale-98"
            >
              <i
                class="ph ph-bed text-titre-l text-ocre"
                aria-hidden="true"
              />
              <span class="font-titre text-action font-semibold text-ink">Enregistrer une arrivée</span>
            </button>
            <button
              type="button"
              class="min-h-28 relative flex cursor-pointer flex-col items-start justify-between rounded-lg border border-line bg-surf p-4 text-left shadow-basse hover:border-prim"
            >
              <i
                class="ph ph-broom text-titre-l text-ocre"
                aria-hidden="true"
              />
              <span class="font-titre text-action font-semibold text-ink">Ménage</span>
              <span class="h-7 absolute top-3 right-3 inline-flex items-center rounded-pleine bg-alerte px-2.5 font-mono text-mini font-bold text-prim-ink">4</span>
            </button>
            <button
              type="button"
              disabled
              class="min-h-28 flex flex-col items-start justify-between rounded-lg border border-line bg-tile p-4 text-left"
            >
              <i
                class="ph ph-cash-register text-titre-l text-ink-3"
                aria-hidden="true"
              />
              <span class="flex flex-col gap-0.5">
                <span class="font-titre text-action font-semibold text-ink-2">Clôturer la caisse</span>
                <span class="text-mini text-ink-3">rôle serveuse</span>
              </span>
            </button>
          </div>
        </VitrineTheme>
      </section>

      <!-- ══ 06 CARTE CHIFFRE ══════════════════════════════════════════════════════════════ -->
      <section
        id="c6"
        class="flex scroll-mt-6 flex-col gap-3.5"
      >
        <div class="flex items-baseline gap-3">
          <span class="font-mono text-mini text-ink-3">06</span>
          <h2 class="m-0 font-titre text-titre-m font-semibold">
            Carte chiffre
          </h2>
          <span class="h-px flex-1 bg-line" />
          <span class="font-mono text-mini text-ink-3">montant en font-mono · whitespace-nowrap obligatoire</span>
        </div>
        <p class="m-0 max-w-prose text-corps text-ink-2">
          Deux cartes côte à côte alignent leurs unités sans réglage — à condition que le montant
          passe par la fonction unique. Le delta est un triangle <em>plus</em> une couleur.
        </p>

        <VitrineTheme
          libelle-clair="Clair"
          libelle-sombre="Sombre"
        >
          <div class="grid grid-cols-1 gap-4 sm:grid-cols-3">
            <div class="flex flex-col gap-1.5 rounded-xl border border-line bg-surf p-4 shadow-basse">
              <span class="text-etiquette text-ink-3 uppercase">Recette du jour</span>
              <span class="font-mono text-chiffre-l font-bold whitespace-nowrap text-ink">{{ formaterMontant(184_000, DEVISE) }}</span>
              <span class="inline-flex items-center gap-1.5 text-mini font-semibold text-succes-fort">
                <span class="size-2 bg-succes [clip-path:polygon(50%_0,100%_100%,0_100%)]" />+ 12 % contre hier
              </span>
            </div>
            <div class="flex flex-col gap-1.5 rounded-xl border border-line bg-surf p-4 shadow-basse">
              <span class="text-etiquette text-ink-3 uppercase">Encaissé</span>
              <span class="font-mono text-chiffre-l font-bold whitespace-nowrap text-ink">{{ formaterMontant(1_250_000, DEVISE) }}</span>
              <span class="inline-flex items-center gap-1.5 text-mini font-semibold text-danger-fort">
                <span class="size-2 rotate-180 bg-danger [clip-path:polygon(50%_0,100%_100%,0_100%)]" />− 4 % contre hier
              </span>
            </div>
            <div class="flex flex-col gap-1.5 rounded-xl border border-line bg-surf p-4 shadow-basse">
              <span class="text-etiquette text-ink-3 uppercase">En chargement</span>
              <span class="h-8 w-3/5 relative overflow-hidden rounded-sm bg-tile">
                <span class="absolute inset-0 animate-scintillement bg-linear-to-r from-transparent via-brillance to-transparent" />
              </span>
              <span class="h-3 w-2/5 relative overflow-hidden rounded-sm bg-tile">
                <span class="absolute inset-0 animate-scintillement bg-linear-to-r from-transparent via-brillance to-transparent" />
              </span>
            </div>
          </div>
        </VitrineTheme>
      </section>

      <!-- ══ 07 BANDEAU D'ALERTE ═══════════════════════════════════════════════════════════ -->
      <section
        id="c7"
        class="flex scroll-mt-6 flex-col gap-3.5"
      >
        <div class="flex items-baseline gap-3">
          <span class="font-mono text-mini text-ink-3">07</span>
          <h2 class="m-0 font-titre text-titre-m font-semibold">
            Bandeau d’alerte
          </h2>
          <span class="h-px flex-1 bg-line" />
          <span class="font-mono text-mini text-ink-3">contrefort 4 px · jamais deux empilés</span>
        </div>
        <p class="m-0 max-w-prose text-corps text-ink-2">
          Structure fixe : icône, une phrase au passé qui dit ce qui s’est produit, l’action à
          droite. Le plus grave gagne, l’autre attend.
        </p>

        <VitrineTheme
          libelle-clair="Clair"
          libelle-sombre="Sombre"
        >
          <div class="flex items-start gap-3 rounded-r-lg border-l-4 border-l-info bg-info-soft p-3.5">
            <i
              class="ph-fill ph-info mt-0.5 text-titre-s text-info"
              aria-hidden="true"
            />
            <p class="m-0 flex-1 text-corps text-info-fort">
              La note a été envoyée à la certification.
            </p>
          </div>
          <div class="flex items-start gap-3 rounded-r-lg border-l-4 border-l-succes bg-succes-soft p-3.5">
            <i
              class="ph-fill ph-check-circle mt-0.5 text-titre-s text-succes"
              aria-hidden="true"
            />
            <p class="m-0 flex-1 text-corps text-succes-fort">
              Bar a été ajouté à vos services.
            </p>
          </div>
          <div class="flex items-start gap-3 rounded-r-lg border-l-4 border-l-alerte bg-alerte-soft p-3.5">
            <i
              class="ph-fill ph-warning mt-0.5 text-titre-s text-alerte"
              aria-hidden="true"
            />
            <p class="m-0 flex-1 text-corps text-alerte-fort">
              Trois consommations attendent d’être envoyées.
            </p>
            <button
              type="button"
              class="h-9 cursor-pointer rounded-md px-3.5 font-titre text-mini font-semibold text-alerte-fort hover:bg-alerte-soft"
            >
              Voir
            </button>
          </div>
          <div class="flex items-start gap-3 rounded-r-lg border-l-4 border-l-danger bg-danger-soft p-3.5">
            <i
              class="ph-fill ph-x-circle mt-0.5 text-titre-s text-danger"
              aria-hidden="true"
            />
            <p class="m-0 flex-1 text-corps text-danger-fort">
              La certification a été refusée : le numéro de compte contribuable est absent.
            </p>
          </div>
          <div class="flex flex-col gap-2">
            <span class="text-etiquette text-ink-3 uppercase">Pleine largeur — hors ligne</span>
            <div class="flex items-center gap-3 rounded-lg bg-ink p-3.5">
              <i
                class="ph-fill ph-wifi-slash text-titre-s text-bg"
                aria-hidden="true"
              />
              <p class="m-0 flex-1 text-corps font-semibold text-bg">
                Hors ligne depuis 14 h 05 — 12 éléments en attente.
              </p>
            </div>
          </div>
        </VitrineTheme>
      </section>

      <!-- ══ 08 LIGNE DE LISTE ═════════════════════════════════════════════════════════════ -->
      <section
        id="c8"
        class="flex scroll-mt-6 flex-col gap-3.5"
      >
        <div class="flex items-baseline gap-3">
          <span class="font-mono text-mini text-ink-3">08</span>
          <h2 class="m-0 font-titre text-titre-m font-semibold">
            Ligne de liste
          </h2>
          <span class="h-px flex-1 bg-line" />
          <span class="font-mono text-mini text-ink-3">h-14 · colonne de montant fixe, alignée à droite</span>
        </div>
        <p class="m-0 max-w-prose text-corps text-ink-2">
          Numéro et montant en mono, en colonne de largeur fixe : l’œil descend une colonne, pas un
          texte. C’est ici que le désalignement d’une police de repli se voit le mieux.
        </p>

        <VitrineTheme
          libelle-clair="Clair"
          libelle-sombre="Sombre"
        >
          <div class="overflow-hidden rounded-xl border border-line bg-surf">
            <div class="h-14 flex cursor-pointer items-center gap-3 border-b border-line px-4 transition-colors duration-90 hover:bg-prim-soft">
              <span class="w-9 font-mono text-corps text-ink-3">101</span>
              <span class="flex flex-1 flex-col">
                <span class="font-titre text-corps font-semibold text-ink">Kouassi Adjoua</span>
                <span class="text-mini text-ink-3">Nuitée · départ demain 12 h 00</span>
              </span>
              <span class="w-28 text-right font-mono text-corps font-bold whitespace-nowrap text-ink">{{ formaterMontant(12_500, DEVISE) }}</span>
            </div>
            <div class="h-14 flex cursor-pointer items-center gap-3 border-b border-line border-l-4 border-l-prim bg-prim-soft px-4">
              <span class="w-9 font-mono text-corps text-ink-3">102</span>
              <span class="flex flex-1 flex-col">
                <span class="font-titre text-corps font-semibold text-ink">Yao N’Guessan</span>
                <span class="text-mini text-ink-3">Passage · 3 h</span>
              </span>
              <span class="w-28 text-right font-mono text-corps font-bold whitespace-nowrap text-ink">{{ formaterMontant(1_500, DEVISE) }}</span>
            </div>
            <div class="h-14 flex items-center gap-3 border-b border-line px-4">
              <span class="w-9 font-mono text-corps text-ink-3">103</span>
              <span class="flex flex-1 flex-col">
                <span class="font-titre text-corps font-semibold text-ink">Traoré Fatou</span>
                <span class="inline-flex items-center gap-1.5 text-mini text-info-fort">
                  <span class="size-3 rounded-pleine border-2 border-info/30 border-t-info animate-roue" />En attente d’envoi
                </span>
              </span>
              <span class="w-28 text-right font-mono text-corps font-bold whitespace-nowrap text-ink">{{ formaterMontant(150_000, DEVISE) }}</span>
            </div>
            <div class="h-14 flex items-center gap-3 border-b border-line px-4 opacity-60">
              <span class="w-9 font-mono text-corps text-ink-3 line-through">104</span>
              <span class="flex flex-1 flex-col">
                <span class="font-titre text-corps font-semibold text-ink line-through">Bamba Sékou</span>
                <span class="text-mini text-ink-3">Annulée</span>
              </span>
              <span class="w-28 text-right font-mono text-corps font-bold whitespace-nowrap text-ink line-through">{{ formaterMontant(1_250_000, DEVISE) }}</span>
            </div>
            <div class="h-14 flex items-center gap-3 border-t-2 border-line-2 px-4">
              <span class="w-9" />
              <span class="flex-1 font-titre text-corps font-semibold text-ink">Total du jour</span>
              <span class="w-28 text-right font-mono text-corps font-bold whitespace-nowrap text-ink">{{ formaterMontant(1_414_000, DEVISE) }}</span>
            </div>
          </div>
        </VitrineTheme>
      </section>

      <!-- ══ 09 SÉLECTEUR D'ÉTABLISSEMENT ══════════════════════════════════════════════════ -->
      <section
        id="c9"
        class="flex scroll-mt-6 flex-col gap-3.5"
      >
        <div class="flex items-baseline gap-3">
          <span class="font-mono text-mini text-ink-3">09</span>
          <h2 class="m-0 font-titre text-titre-m font-semibold">
            Sélecteur d’établissement
          </h2>
          <span class="h-px flex-1 bg-line" />
          <span class="font-mono text-mini text-ink-3">h-11 · initiale en ocre, jamais en indigo</span>
        </div>
        <p class="m-0 max-w-prose text-corps text-ink-2">
          Savoir <em>où on est</em> avant de faire quoi que ce soit. Avec un seul établissement il
          perd son chevron et cesse d’être un bouton.
        </p>

        <VitrineTheme
          libelle-clair="Clair"
          libelle-sombre="Sombre"
        >
          <div class="flex flex-wrap items-end gap-4">
            <div class="flex flex-col gap-2">
              <span class="text-etiquette text-ink-3 uppercase">Un seul établissement</span>
              <span class="h-11 inline-flex items-center gap-2.5 rounded-lg border border-line bg-surf px-3 shadow-basse">
                <span class="size-7 inline-flex items-center justify-center rounded-md bg-ocre-soft font-titre text-corps font-bold text-ocre">D</span>
                <span class="font-titre text-corps font-semibold text-ink">Résidence Deloria</span>
              </span>
            </div>
            <div class="flex flex-col gap-2">
              <span class="text-etiquette text-ink-3 uppercase">Plusieurs</span>
              <button
                type="button"
                class="h-11 inline-flex cursor-pointer items-center gap-2.5 rounded-lg border border-line bg-surf px-3 shadow-basse hover:border-prim"
              >
                <span class="size-7 inline-flex items-center justify-center rounded-md bg-ocre-soft font-titre text-corps font-bold text-ocre">D</span>
                <span class="font-titre text-corps font-semibold text-ink">Résidence Deloria</span>
                <i
                  class="ph ph-caret-up-down text-ink-3"
                  aria-hidden="true"
                />
              </button>
            </div>
            <div class="flex flex-col gap-2">
              <span class="text-etiquette text-ink-3 uppercase">Avec alerte ailleurs</span>
              <button
                type="button"
                class="h-11 inline-flex cursor-pointer items-center gap-2.5 rounded-lg border border-line bg-surf px-3 shadow-basse hover:border-prim"
              >
                <span class="size-7 relative inline-flex items-center justify-center rounded-md bg-ocre-soft font-titre text-corps font-bold text-ocre">
                  D
                  <span class="size-2 absolute -top-0.5 -right-0.5 rounded-pleine bg-alerte" />
                </span>
                <span class="font-titre text-corps font-semibold text-ink">Résidence Deloria</span>
                <i
                  class="ph ph-caret-up-down text-ink-3"
                  aria-hidden="true"
                />
              </button>
            </div>
          </div>
          <div class="flex flex-col gap-2">
            <span class="text-etiquette text-ink-3 uppercase">Ouvert</span>
            <div class="w-72 overflow-hidden rounded-lg border border-line bg-surf shadow-panneau">
              <div class="h-14 flex items-center gap-2.5 bg-prim-soft px-3">
                <span class="size-7 inline-flex items-center justify-center rounded-md bg-ocre-soft font-titre text-corps font-bold text-ocre">D</span>
                <span class="flex-1 font-titre text-corps font-semibold text-prim-fort">Résidence Deloria</span>
                <i
                  class="ph ph-check text-prim"
                  aria-hidden="true"
                />
              </div>
              <div class="h-14 flex cursor-pointer items-center gap-2.5 px-3 hover:bg-prim-soft">
                <span class="size-7 inline-flex items-center justify-center rounded-md bg-ocre-soft font-titre text-corps font-bold text-ocre">M</span>
                <span class="flex-1 font-titre text-corps text-ink">Maquis du Carrefour</span>
                <span class="size-2 rounded-pleine bg-alerte" />
              </div>
            </div>
          </div>
        </VitrineTheme>
      </section>

      <!-- ══ 10 TÉMOIN DE SYNCHRONISATION ══════════════════════════════════════════════════ -->
      <section
        id="c10"
        class="flex scroll-mt-6 flex-col gap-3.5"
      >
        <div class="flex items-baseline gap-3">
          <span class="font-mono text-mini text-ink-3">10</span>
          <h2 class="m-0 font-titre text-titre-m font-semibold">
            Témoin de synchronisation
          </h2>
          <span class="h-px flex-1 bg-line" />
          <span class="font-mono text-mini text-ink-3">le composant le plus important du produit</span>
        </div>
        <p class="m-0 max-w-prose text-corps text-ink-2">
          Trois états seulement, chacun avec sa forme et sa phrase. <strong>Jamais de
            pourcentage</strong> : un nombre d’écritures et une heure. Le pouls est lent — il
          rassure, il n’alerte pas.
        </p>

        <VitrineTheme
          libelle-clair="Clair"
          libelle-sombre="Sombre"
        >
          <div class="flex flex-col gap-3">
            <span class="inline-flex items-center gap-2.5 text-corps text-ink-2">
              <span class="size-2.5 relative inline-block">
                <span class="absolute inset-0 rounded-pleine bg-succes" />
                <span class="absolute inset-0 animate-pulse-reseau rounded-pleine bg-succes" />
              </span>Connecté
            </span>
            <span class="inline-flex items-center gap-2.5 text-corps text-ink-2">
              <span class="size-2.5 rounded-pleine bg-alerte" />Réseau dégradé — 3 éléments en attente
            </span>
            <span class="inline-flex items-center gap-2.5 text-corps text-ink-2">
              <span class="size-2.5 rounded-pleine border-2 border-ink-3" />Hors ligne depuis 14 h 05 — 12 éléments en attente
            </span>
            <span class="inline-flex items-center gap-2.5 text-corps text-ink-2">
              <i
                class="ph ph-cloud-arrow-up text-info"
                aria-hidden="true"
              />Envoi en cours…
            </span>
          </div>
          <div class="flex flex-col gap-2">
            <span class="text-etiquette text-ink-3 uppercase">Variante compacte — barre d’en-tête</span>
            <div class="h-11 inline-flex items-center gap-2 rounded-lg bg-tile px-3">
              <span class="size-2 rounded-pleine bg-succes" />
              <span class="font-mono text-mini text-ink-2">0</span>
            </div>
          </div>
        </VitrineTheme>
      </section>

      <!-- ══ 11 ÉTAT VIDE ILLUSTRÉ ═════════════════════════════════════════════════════════ -->
      <section
        id="c11"
        class="flex scroll-mt-6 flex-col gap-3.5"
      >
        <div class="flex items-baseline gap-3">
          <span class="font-mono text-mini text-ink-3">11</span>
          <h2 class="m-0 font-titre text-titre-m font-semibold">
            État vide illustré
          </h2>
          <span class="h-px flex-1 bg-line" />
          <span class="font-mono text-mini text-ink-3">motif ocre · jamais un personnage</span>
        </div>
        <p class="m-0 max-w-prose text-corps text-ink-2">
          Trois éléments, dans cet ordre : le motif, une phrase qui dit ce qui apparaîtra ici,
          l’action qui démarre. Le vide de résultat n’a pas de motif — il a une porte de sortie.
        </p>

        <VitrineTheme
          libelle-clair="Clair"
          libelle-sombre="Sombre"
        >
          <div class="flex flex-col items-center gap-4 rounded-xl border border-line bg-surf p-8 text-center">
            <span class="size-16 inline-flex items-center justify-center rounded-2xl bg-ocre-soft">
              <i
                class="ph ph-bed text-titre-l text-ocre"
                aria-hidden="true"
              />
            </span>
            <p class="m-0 max-w-prose text-corps text-ink-2">
              Les séjours en cours apparaîtront ici.
            </p>
            <button
              type="button"
              class="h-11 min-w-42 cursor-pointer rounded-lg bg-prim px-5 font-titre text-action font-semibold text-prim-ink shadow-bouton hover:bg-prim-dk"
            >
              Enregistrer une arrivée
            </button>
          </div>
          <div class="flex flex-col items-center gap-3 rounded-xl border border-line bg-surf p-8 text-center">
            <i
              class="ph ph-magnifying-glass text-titre-l text-ink-3"
              aria-hidden="true"
            />
            <p class="m-0 text-corps text-ink-2">
              Aucun séjour ne correspond à ce filtre.
            </p>
            <button
              type="button"
              class="h-9 cursor-pointer rounded-md px-3.5 font-titre text-mini font-semibold text-prim hover:bg-prim-soft"
            >
              Chercher dans toute l’année
            </button>
          </div>
        </VitrineTheme>
      </section>

      <!-- ══ 12 SÉLECTEUR SEGMENTÉ ═════════════════════════════════════════════════════════ -->
      <section
        id="c12"
        class="flex scroll-mt-6 flex-col gap-3.5"
      >
        <div class="flex items-baseline gap-3">
          <span class="font-mono text-mini text-ink-3">12</span>
          <h2 class="m-0 font-titre text-titre-m font-semibold">
            Sélecteur segmenté
          </h2>
          <span class="h-px flex-1 bg-line" />
          <span class="font-mono text-mini text-ink-3">h-10 · piste bg-tile · h-12 au comptoir</span>
        </div>
        <p class="m-0 max-w-prose text-corps text-ink-2">
          Deux à quatre options courtes, toutes visibles. Au-delà de quatre, c’est une liste.
        </p>

        <VitrineTheme
          libelle-clair="Clair"
          libelle-sombre="Sombre"
        >
          <div class="flex flex-wrap items-end gap-4">
            <div class="flex flex-col gap-2">
              <span class="text-etiquette text-ink-3 uppercase">Deux options</span>
              <div class="h-10 inline-flex gap-1 rounded-lg bg-tile p-1">
                <button
                  type="button"
                  class="h-8 cursor-pointer rounded-md bg-prim px-4.5 font-titre text-mini font-semibold text-prim-ink"
                >
                  Toutes
                </button>
                <button
                  type="button"
                  class="h-8 cursor-pointer rounded-md px-4.5 font-titre text-mini font-semibold text-ink-2 hover:text-ink"
                >
                  Impayées
                </button>
              </div>
            </div>
            <div class="flex flex-col gap-2">
              <span class="text-etiquette text-ink-3 uppercase">Trois options avec compteur</span>
              <div class="h-10 inline-flex gap-1 rounded-lg bg-tile p-1">
                <button
                  type="button"
                  class="h-8 cursor-pointer rounded-md px-4.5 font-titre text-mini font-semibold text-ink-2 hover:text-ink"
                >
                  Toutes
                </button>
                <button
                  type="button"
                  class="h-8 inline-flex cursor-pointer items-center gap-1.5 rounded-md bg-prim px-4.5 font-titre text-mini font-semibold text-prim-ink"
                >
                  À nettoyer<span class="font-mono">4</span>
                </button>
                <button
                  type="button"
                  class="h-8 cursor-pointer rounded-md px-4.5 font-titre text-mini font-semibold text-ink-2 hover:text-ink"
                >
                  Libres
                </button>
              </div>
            </div>
          </div>
          <div class="flex flex-col gap-2">
            <span class="text-etiquette text-ink-3 uppercase">Variante tactile — les durées de passage</span>
            <div class="inline-flex flex-wrap gap-2">
              <button
                type="button"
                class="h-12 inline-flex cursor-pointer items-center gap-2 rounded-lg bg-prim px-5 font-titre text-lead font-semibold text-prim-ink"
              >
                <span class="size-3 rounded-pleine bg-prim-ink" />3 h
              </button>
              <button
                type="button"
                class="h-12 inline-flex cursor-pointer items-center gap-2 rounded-lg bg-tile px-5 font-titre text-lead font-semibold text-ink-2"
              >
                <span class="size-3 rounded-pleine border-2 border-ink-3" />6 h
              </button>
              <button
                type="button"
                class="h-12 inline-flex cursor-pointer items-center gap-2 rounded-lg bg-tile px-5 font-titre text-lead font-semibold text-ink-2"
              >
                <span class="size-3 rounded-pleine border-2 border-ink-3" />Nuitée
              </button>
            </div>
          </div>
        </VitrineTheme>
      </section>

      <!-- ══ 13 SQUELETTE DE CHARGEMENT ════════════════════════════════════════════════════ -->
      <section
        id="c13"
        class="flex scroll-mt-6 flex-col gap-3.5"
      >
        <div class="flex items-baseline gap-3">
          <span class="font-mono text-mini text-ink-3">13</span>
          <h2 class="m-0 font-titre text-titre-m font-semibold">
            Squelette de chargement
          </h2>
          <span class="h-px flex-1 bg-line" />
          <span class="font-mono text-mini text-ink-3">bande translatée · jamais un dégradé animé</span>
        </div>
        <p class="m-0 max-w-prose text-corps text-ink-2">
          Même hauteur de ligne et même largeur de colonne que le contenu réel, pour que rien ne
          saute. La roue est réservée à une attente dont on ne connaît pas la forme.
        </p>

        <VitrineTheme
          libelle-clair="Clair"
          libelle-sombre="Sombre"
        >
          <div class="overflow-hidden rounded-xl border border-line bg-surf">
            <div
              v-for="ligne in 3"
              :key="ligne"
              class="h-14 flex items-center gap-3 border-b border-line px-4 last:border-b-0"
            >
              <span class="h-3 w-9 relative overflow-hidden rounded-sm bg-tile">
                <span class="absolute inset-0 animate-scintillement bg-linear-to-r from-transparent via-brillance to-transparent" />
              </span>
              <span class="h-3 w-3/5 relative overflow-hidden rounded-sm bg-tile">
                <span class="absolute inset-0 animate-scintillement bg-linear-to-r from-transparent via-brillance to-transparent" />
              </span>
              <span class="h-3 w-24 relative ml-auto overflow-hidden rounded-sm bg-tile">
                <span class="absolute inset-0 animate-scintillement bg-linear-to-r from-transparent via-brillance to-transparent" />
              </span>
            </div>
          </div>
          <div class="flex items-center gap-3">
            <span class="size-6 rounded-pleine border-2 border-info/30 border-t-info animate-roue" />
            <span class="text-corps text-ink-2">Attente réseau de forme inconnue</span>
          </div>
        </VitrineTheme>
      </section>

      <!-- ══ 14 BANDEAU D'ANNULATION ═══════════════════════════════════════════════════════ -->
      <section
        id="c14"
        class="flex scroll-mt-6 flex-col gap-3.5"
      >
        <div class="flex items-baseline gap-3">
          <span class="font-mono text-mini text-ink-3">14</span>
          <h2 class="m-0 font-titre text-titre-m font-semibold">
            Bandeau d’annulation
          </h2>
          <span class="h-px flex-1 bg-line" />
          <span class="font-mono text-mini text-ink-3">h-12 · 8 secondes · pas de fenêtre de confirmation</span>
        </div>
        <p class="m-0 max-w-prose text-corps text-ink-2">
          Toute action destructrice s’exécute immédiatement et laisse huit secondes pour revenir.
          Exception : ce qui est fiscalement irréversible demande une confirmation explicite.
        </p>

        <VitrineTheme
          libelle-clair="Clair"
          libelle-sombre="Sombre"
        >
          <div class="h-12 inline-flex items-center gap-3 rounded-lg bg-ink pr-2 pl-4 shadow-panneau">
            <span class="flex-1 text-corps font-semibold text-bg">Consommation supprimée</span>
            <span class="font-mono text-mini text-bg/60">8 s</span>
            <button
              type="button"
              class="h-9 cursor-pointer rounded-md border border-bg/30 px-3.5 font-titre text-mini font-semibold text-bg"
            >
              Annuler
            </button>
          </div>
          <div class="flex flex-col gap-2">
            <span class="text-etiquette text-ink-3 uppercase">Avec barre</span>
            <div class="w-96 overflow-hidden rounded-lg bg-ink shadow-panneau">
              <div class="h-12 flex items-center gap-3 pr-2 pl-4">
                <span class="flex-1 text-corps font-semibold text-bg">Ligne annulée</span>
                <button
                  type="button"
                  class="h-9 cursor-pointer rounded-md border border-bg/30 px-3.5 font-titre text-mini font-semibold text-bg"
                >
                  Rétablir
                </button>
              </div>
              <span class="h-1 block w-2/3 bg-bg/40" />
            </div>
          </div>
          <div class="flex flex-col gap-2">
            <span class="text-etiquette text-ink-3 uppercase">Cas non annulable — facture émise</span>
            <div class="flex items-start gap-3 rounded-r-lg border-l-4 border-l-danger bg-danger-soft p-3.5">
              <i
                class="ph-fill ph-warning-circle mt-0.5 text-titre-s text-danger"
                aria-hidden="true"
              />
              <p class="m-0 flex-1 text-corps text-danger-fort">
                Cette facture est certifiée : elle ne s’annule pas. Elle se contre-passe par un avoir.
              </p>
            </div>
          </div>
        </VitrineTheme>
      </section>

      <!-- ══ 15 BARRE DE PROPORTION ════════════════════════════════════════════════════════ -->
      <section
        id="c15"
        class="flex scroll-mt-6 flex-col gap-3.5"
      >
        <div class="flex items-baseline gap-3">
          <span class="font-mono text-mini text-ink-3">15</span>
          <h2 class="m-0 font-titre text-titre-m font-semibold">
            Barre de proportion
          </h2>
          <span class="h-px flex-1 bg-line" />
          <span class="font-mono text-mini text-ink-3">hors série — décision à prendre</span>
        </div>
        <p class="m-0 max-w-prose text-corps text-ink-2">
          Elle porte toujours son chiffre à côté d’elle : une barre seule ne se lit pas. Elle entre
          dans le canon avec ses états, ou elle reste une composition locale de la carte chiffre.
        </p>

        <VitrineTheme
          libelle-clair="Clair"
          libelle-sombre="Sombre"
        >
          <div class="flex max-w-md flex-col gap-4">
            <div class="flex flex-col gap-1.5">
              <div class="flex items-baseline justify-between">
                <span class="text-etiquette text-ink-3 uppercase">Taux d’occupation</span>
                <span class="font-mono text-corps font-bold text-ink">72 %</span>
              </div>
              <span class="h-2 block overflow-hidden rounded-pleine bg-tile">
                <span class="h-2 block w-3/4 rounded-pleine bg-prim" />
              </span>
            </div>
            <div class="flex flex-col gap-1.5">
              <div class="flex items-baseline justify-between">
                <span class="text-etiquette text-ink-3 uppercase">Part du passage</span>
                <span class="font-mono text-corps font-bold text-ink">31 %</span>
              </div>
              <span class="h-2 block overflow-hidden rounded-pleine bg-tile">
                <span class="h-2 block w-1/3 rounded-pleine bg-ocre" />
              </span>
            </div>
            <div class="flex flex-col gap-1.5">
              <div class="flex items-baseline justify-between">
                <span class="text-etiquette text-ink-3 uppercase">Objectif atteint</span>
                <span class="font-mono text-corps font-bold text-ink">{{ formaterMontant(184_000, DEVISE) }}</span>
              </div>
              <span class="h-2 block overflow-hidden rounded-pleine bg-tile">
                <span class="h-2 block w-full rounded-pleine bg-succes" />
              </span>
            </div>
          </div>
        </VitrineTheme>
      </section>

      <!-- ══ 16 CHAMP DE SAISIE ════════════════════════════════════════════════════════════ -->
      <section
        id="c16"
        class="flex scroll-mt-6 flex-col gap-3.5"
      >
        <div class="flex items-baseline gap-3">
          <span class="font-mono text-mini text-ink-3">16</span>
          <h2 class="m-0 font-titre text-titre-m font-semibold">
            Champ de saisie
          </h2>
          <span class="h-px flex-1 bg-line" />
          <span class="font-mono text-mini text-ink-3">le seul composant Vue écrit à ce jour</span>
        </div>
        <p class="m-0 max-w-prose text-corps text-ink-2">
          Le vrai composant, monté ici — pas une imitation en classes. Il reçoit des clés i18n, et
          les libellés ci-dessous viennent donc du catalogue réel.
        </p>

        <VitrineTheme
          libelle-clair="Clair"
          libelle-sombre="Sombre"
        >
          <div class="grid max-w-2xl grid-cols-1 gap-5 sm:grid-cols-2">
            <div class="flex flex-col gap-2">
              <span class="text-etiquette text-ink-3 uppercase">Repos</span>
              <ChampSaisie
                v-model="saisieRepos"
                :etiquette-cle="CLE_ETIQUETTE"
                :placeholder-cle="CLE_INVITE"
              />
            </div>
            <div class="flex flex-col gap-2">
              <span class="text-etiquette text-ink-3 uppercase">Saisie · avec aide</span>
              <ChampSaisie
                v-model="saisieRemplie"
                :etiquette-cle="CLE_ETIQUETTE"
                :aide-cle="CLE_AIDE"
              />
            </div>
            <div class="flex flex-col gap-2">
              <span class="text-etiquette text-ink-3 uppercase">Erreur — trois signaux</span>
              <ChampSaisie
                v-model="saisieErreur"
                :etiquette-cle="CLE_ETIQUETTE"
                :aide-cle="CLE_AIDE"
                :erreur-cle="CLE_ERREUR"
              />
            </div>
            <div class="flex flex-col gap-2">
              <span class="text-etiquette text-ink-3 uppercase">Lecture seule — se copie</span>
              <ChampSaisie
                v-model="saisieRemplie"
                :etiquette-cle="CLE_ETIQUETTE"
                lecture-seule
              />
            </div>
            <div class="flex flex-col gap-2">
              <span class="text-etiquette text-ink-3 uppercase">Désactivé — ne se copie pas</span>
              <ChampSaisie
                v-model="saisieRemplie"
                :etiquette-cle="CLE_ETIQUETTE"
                desactive
              />
            </div>
            <div class="flex flex-col gap-2">
              <span class="text-etiquette text-ink-3 uppercase">Comptoir — h-12</span>
              <ChampSaisie
                v-model="saisieRepos"
                :etiquette-cle="CLE_ETIQUETTE"
                :placeholder-cle="CLE_INVITE"
                taille="comptoir"
              />
            </div>
            <div class="flex flex-col gap-2">
              <span class="text-etiquette text-ink-3 uppercase">Mot de passe — masqué</span>
              <!-- État ajouté par `R0` (CPT-01). Le type est fermé à deux valeurs : `email`,
                   `tel` et `number` sont délibérément absents, leurs claviers et leurs
                   validations de navigateur contrediraient les règles du produit. -->
              <ChampSaisie
                v-model="saisieRemplie"
                :etiquette-cle="CLE_ETIQUETTE"
                type="mot_de_passe"
                autocompletion="current-password"
                taille="comptoir"
              />
            </div>
            <div class="flex flex-col gap-2 sm:col-span-2">
              <span class="text-etiquette text-ink-3 uppercase">Choix fermé — même enveloppe</span>
              <ChampSaisie
                v-model="saisieChoix"
                :etiquette-cle="CLE_ETIQUETTE"
                :placeholder-cle="CLE_INVITE"
                :aide-cle="CLE_AIDE"
                :options="OPTIONS_SERVICE"
              />
            </div>
          </div>
        </VitrineTheme>
      </section>
    </main>
  </div>
</template>
