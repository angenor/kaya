<script setup lang="ts">
/**
 * **Section « Vos services »** — ETB-02, ETB-02b, et **le patron d'écriture front du produit**.
 *
 * Le déroulé complet et son pourquoi : `docs/module-dore.md`, section « La septième couche — le
 * patron d'écriture front ». Ce qui suit ne redit que ce qui se lit dans ce fichier.
 *
 * # Un service inactif est ABSENT
 *
 * Ni entrée grisée, ni mention « disponible dans votre offre », ni marqueur masqué par CSS
 * (principe VII). La liste rendue ne contient que des services actifs, et le HTML produit ne porte
 * **aucun** libellé ni code des autres — c'est ce que vérifie le test de rendu de SC-005.
 *
 * **La même règle vaut pour les ACTIONS**, et c'est le point que ce cycle ajoute : sans la
 * permission de modifier les services, le bouton d'ajout et les boutons de retrait ne sont pas
 * désactivés — ils **n'existent pas dans le HTML rendu**. Le grisé est le réflexe naturel, et
 * c'est celui que le principe VII interdit : il apprend à l'utilisateur, à chaque écran et tous
 * les jours, qu'une partie du produit lui est refusée.
 *
 * # Le mot « capacité » n'apparaît nulle part
 *
 * Seule la capacité concrète est nommée — « Suivi du stock » — **sous le service qui la
 * consomme**, jamais dans une rubrique à part (`docs/design/lexique.md`).
 *
 * # Hors ligne, l'action DIT qu'elle exige le réseau — elle ne se grise pas
 *
 * `etablissement_module` est de **classe C** (`docs/registre-classes-offline.md` §5.1). Hors ligne
 * ou en réseau dégradé, les actions disparaissent et **un bandeau les remplace**, qui dit pourquoi
 * en une phrase. Jamais de grisé silencieux, jamais de mise en file « au cas où », jamais d'échec
 * après trente secondes d'attente (principe VI).
 */
import { computed, ref } from 'vue'

import ChampSaisie from '~/core/design-system/ChampSaisie.vue'
import { useEtatReseau } from '~/core/platform/reseau'
import { detient, type Permissions } from '~/core/rbac'
import {
  basculerService,
  PERMISSION_BASCULER,
  type ObstacleVue,
} from './bascule-service'
import { chargerServices, type ContexteAppel } from './donnees'
import {
  capacitesVisibles,
  servicesActivables,
  servicesVisibles,
  type EntreeReferentiel,
  type ServiceActif,
} from './services-visibles'

const { t } = useI18n()

const props = defineProps<{
  services: ServiceActif[]
  referentiel: EntreeReferentiel[]
  contexte: ContexteAppel
  etablissementId: string
  /** Permissions cumulées de l'utilisateur — union de ses rôles, jamais celles d'un rôle principal. */
  permissions: Permissions
}>()

const emit = defineEmits<{ 'services-changes': [ServiceActif[]] }>()

const visibles = computed(() => servicesVisibles(props.services, props.referentiel))
const activables = computed(() => servicesActivables(props.services, props.referentiel))

/** Icône par service. Décorative : chaque ligne porte déjà son libellé traduit. */
const ICONES: Record<string, string> = {
  HEBERGEMENT: 'ph-moon-stars',
  RESTAURATION: 'ph-fork-knife',
  BAR: 'ph-martini',
  PRESSING: 'ph-t-shirt',
  SALLE_REUNION: 'ph-users-three',
}

// ═══════════════════════════════════════════════════════════════════════════════════════════════
//  Ce qui décide de la PRÉSENCE des actions — deux conditions, deux traitements opposés
// ═══════════════════════════════════════════════════════════════════════════════════════════════

const reseau = useEtatReseau()

/** **Absence pure** (principe VII) : sans le droit, l'action n'existe pas, et rien ne le dit. */
const peutModifier = computed(() => detient(props.permissions, PERMISSION_BASCULER))

/**
 * **Absence expliquée** (principe VI) : le réseau manque, l'action disparaît **et un bandeau dit
 * pourquoi**. La différence avec le cas précédent n'est pas cosmétique — un droit manquant n'est
 * pas une nouvelle à annoncer, une coupure réseau si.
 */
const reseauRequis = computed(() => reseau.value !== 'connecte')

const actionsVisibles = computed(() => peutModifier.value && !reseauRequis.value)

// ═══════════════════════════════════════════════════════════════════════════════════════════════
//  L'écriture
// ═══════════════════════════════════════════════════════════════════════════════════════════════

/**
 * L'opération en cours, avec **son sens et sa cible**.
 *
 * Le squelette (composant 13) doit occuper « la forme exacte de ce qui arrive, pour que rien ne
 * saute ». Un simple booléen ne suffirait pas : à l'ajout, la forme qui arrive est une ligne **en
 * fin de liste** ; au retrait, c'est la ligne **concernée** qui s'en va. Un indicateur générique —
 * une roue au milieu de l'écran — ne dirait ni l'un ni l'autre.
 */
const enCours = ref<{ sens: 'ajout' | 'retrait', moduleCode: string } | null>(null)

/** Résultat de la dernière écriture. Rendu par le composant 07. */
const resultat = ref<{
  ton: 'succes' | 'danger'
  cle: string
  valeurs?: Record<string, unknown>
  obstacles?: ObstacleVue[]
} | null>(null)

/**
 * **Le bandeau affiché — un seul, jamais deux empilés** (composant 07 : « le plus grave gagne,
 * l'autre attend »).
 *
 * La règle est tenue par un `computed`, pas par deux `v-if` que le prochain état viendrait
 * contredire. C'est une faute réellement commise ici : la première rédaction empilait le refus
 * métier et l'avis hors-ligne, et **seule la vérification à l'œil l'a vue** — les tests
 * regardaient chaque bandeau séparément, et chacun était correct.
 *
 * **Hors ligne gagne**, pour deux raisons : il concerne l'écran entier là où le résultat concerne
 * une ligne, et il rend le résultat précédent caduc — plus rien n'est actionnable tant que le
 * réseau manque.
 */
const bandeau = computed(() => {
  if (reseauRequis.value && peutModifier.value) {
    return { ton: 'alerte' as const, cle: 'etablissement.services.refus.reseau' }
  }
  return resultat.value
})

/** Ton → fond, contrefort, texte et icône. Trois tons, une seule structure (composant 07). */
const TONS = {
  succes: { fond: 'border-l-succes bg-succes-soft', texte: 'text-succes-fort', icone: 'ph-check-circle text-succes' },
  alerte: { fond: 'border-l-alerte bg-alerte-soft', texte: 'text-alerte-fort', icone: 'ph-warning text-alerte' },
  danger: { fond: 'border-l-danger bg-danger-soft', texte: 'text-danger-fort', icone: 'ph-x-circle text-danger' },
} as const

const formulaireOuvert = ref(false)
const moduleChoisi = ref('')
/** Erreur **au champ**, pas au bandeau : elle porte sur ce qui est saisi, pas sur ce qui s'est passé. */
const erreurChamp = ref<string | null>(null)

const optionsServices = computed(() =>
  activables.value.map(entree => ({ valeur: entree.code, libelleCle: entree.libelle_cle })),
)

function libelleDe(moduleCode: string): string {
  const entree = props.referentiel.find(e => e.code === moduleCode)
  return entree ? t(entree.libelle_cle) : moduleCode
}

function ouvrirFormulaire(): void {
  formulaireOuvert.value = true
  moduleChoisi.value = ''
  erreurChamp.value = null
  resultat.value = null
}

function fermerFormulaire(): void {
  formulaireOuvert.value = false
  moduleChoisi.value = ''
  erreurChamp.value = null
}

/**
 * Le geste unique, dans les deux sens.
 *
 * Ordre des opérations, et le point qu'on écrirait mal : **le rafraîchissement suit le succès**,
 * il ne le précède pas et ne se fait pas « en parallèle pour aller plus vite ». Relire la liste
 * avant que le serveur ait tranché afficherait l'état d'avant en donnant l'impression qu'il s'agit
 * de celui d'après.
 */
async function basculer(moduleCode: string, actif: boolean): Promise<void> {
  enCours.value = { sens: actif ? 'ajout' : 'retrait', moduleCode }
  resultat.value = null

  try {
    // Nommée `issue`, jamais `resultat` : ce dernier est le `ref` du bandeau, et une variable
    // locale du même nom le masquerait sans qu'aucune règle de lint ne s'en plaigne — `resultat.value`
    // deviendrait une propriété posée sur un objet ordinaire, et **le bandeau ne s'afficherait
    // jamais**. Faute réellement commise ici, trouvée par le test « jamais deux bandeaux empilés ».
    const issue = await basculerService(
      props.contexte,
      props.etablissementId,
      moduleCode,
      actif,
      reseau.value,
    )

    if (issue.issue === 'refus') {
      // **Le message est traduit d'une CLÉ, jamais du texte du serveur** — qui nomme des tables et
      // parle anglais technique. Voir `bascule-service.ts`, la table `CLES_DE_REFUS`.
      resultat.value = {
        ton: 'danger',
        cle: issue.cle,
        valeurs: { ...issue.valeurs, service: libelleDe(moduleCode) },
        obstacles: issue.obstacles,
      }
      return
    }

    // **Rafraîchissement sans rechargement de page.** Le serveur fait foi : on relit la liste
    // plutôt que de la modifier à la main côté client.
    emit('services-changes', await chargerServices(props.contexte, props.etablissementId))

    resultat.value = {
      ton: 'succes',
      cle: actif ? 'etablissement.services.succes.ajout' : 'etablissement.services.succes.retrait',
      valeurs: { service: libelleDe(moduleCode) },
    }
    fermerFormulaire()
  }
  catch {
    // Une coupure survenue **pendant** l'appel : `navigator.onLine` dit qu'une interface est
    // active, pas que le serveur répond (voir `core/platform/reseau.ts`). La garde hors-ligne ne
    // dispense donc pas du traitement d'erreur — elle évite seulement l'attente inutile.
    resultat.value = { ton: 'danger', cle: 'etablissement.services.refus.inattendue' }
  }
  finally {
    enCours.value = null
  }
}

function confirmerAjout(): void {
  if (!moduleChoisi.value) {
    // **Validation AU CHAMP** (composant 16), pas au bandeau : l'erreur porte sur ce qui est
    // saisi, et le message doit être à côté de l'endroit où on corrige.
    erreurChamp.value = 'champ.erreur.obligatoire'
    return
  }
  erreurChamp.value = null
  void basculer(moduleChoisi.value, true)
}
</script>

<template>
  <section class="flex flex-col">
    <div class="flex flex-col gap-1.5 px-3.5 pt-4 pb-1">
      <span class="text-etiquette uppercase text-ink-3">{{ t('etablissement.sur_titre') }}</span>
      <h2 class="font-titre text-chiffre font-semibold text-ink">
        {{ t('etablissement.services.titre') }}
      </h2>
      <p class="text-corps text-ink-2">
        {{ t('etablissement.services.intro') }}
      </p>
    </div>

    <!-- COMPOSANT 07 · BANDEAU D'ALERTE — structure fixe : icône, une phrase au passé qui dit ce
         qui s'est produit. **UN SEUL, jamais deux empilés** : c'est le `computed` ci-dessus qui le
         garantit, pas la discipline de deux `v-if` voisins.

         En ton `alerte`, c'est l'avis de classe C — le réseau manque, l'action est ABSENTE **et
         dit pourquoi**, immédiatement. Ni grisé silencieux, ni mise en file : l'opération ne se
         rejoue pas (principe VI).

         Le contrefort de 4 px et le couple fond `-soft` / texte `-fort` basculent en mode sombre
         par les jetons eux-mêmes : aucune variante `dark:` n'est nécessaire ici, les noms sont
         identiques dans les deux modes et seules les valeurs changent. -->
    <div
      v-if="bandeau"
      class="mx-3 mt-2 flex items-start gap-3 rounded-r-lg border-l-4 p-3.5"
      :class="TONS[bandeau.ton].fond"
      :role="bandeau.ton === 'danger' ? 'alert' : 'status'"
    >
      <i
        class="ph-fill mt-0.5 shrink-0 text-titre-s"
        :class="TONS[bandeau.ton].icone"
        aria-hidden="true"
      />
      <div class="flex flex-1 flex-col gap-1">
        <p
          class="text-corps"
          :class="TONS[bandeau.ton].texte"
        >
          {{ t(bandeau.cle, ('valeurs' in bandeau ? bandeau.valeurs : undefined) ?? {}) }}
        </p>
        <!-- Les obstacles à une désactivation : chacun porte sa propre clé i18n et son nombre,
             séparés parce que le pluriel ne s'accorde pas partout de la même façon. -->
        <p
          v-for="obstacle in ('obstacles' in bandeau ? bandeau.obstacles : undefined)"
          :key="obstacle.motif_cle"
          class="text-mini"
          :class="TONS[bandeau.ton].texte"
        >
          {{ t(obstacle.motif_cle, { n: obstacle.nombre }, obstacle.nombre) }}
        </p>
      </div>
    </div>

    <ul class="flex flex-col gap-2.25 px-3 pt-3 pb-3.5">
      <li
        v-for="service in visibles"
        :key="service.id"
      >
        <!-- COMPOSANT 13 · SQUELETTE — pendant le retrait, la ligne concernée prend la forme
             exacte de ce qui s'en va : même hauteur, même largeur de colonne, rien ne saute.
             Un indicateur générique au milieu de l'écran ne dirait pas QUELLE ligne part. -->
        <div
          v-if="enCours && enCours.moduleCode === service.module_code"
          class="flex w-full items-center gap-3 rounded-l-xs rounded-r-xl border border-line border-l-4 border-l-line-2 bg-surf p-3 shadow-basse"
          aria-busy="true"
        >
          <span class="relative size-10 shrink-0 overflow-hidden rounded-xl bg-tile">
            <span
              class="absolute inset-0 bg-linear-to-r from-transparent via-brillance to-transparent animate-scintillement"
            />
          </span>
          <span class="flex min-w-0 flex-1 flex-col gap-2">
            <span class="relative h-3 w-2/5 overflow-hidden rounded-sm bg-tile">
              <span
                class="absolute inset-0 bg-linear-to-r from-transparent via-brillance to-transparent animate-scintillement"
              />
            </span>
            <span class="relative h-2.5 w-1/4 overflow-hidden rounded-sm bg-tile">
              <span
                class="absolute inset-0 bg-linear-to-r from-transparent via-brillance to-transparent animate-scintillement"
              />
            </span>
          </span>
        </div>

        <div
          v-else
          class="flex w-full items-center gap-3 rounded-l-xs rounded-r-xl border border-line border-l-4 border-l-line-2 bg-surf p-3 shadow-basse"
        >
          <span
            class="inline-flex size-10 shrink-0 items-center justify-center rounded-xl bg-tile"
          >
            <i
              class="ph text-titre-m text-ocre"
              :class="ICONES[service.module_code] ?? 'ph-buildings'"
              aria-hidden="true"
            />
          </span>
          <span class="flex min-w-0 flex-1 flex-col items-start gap-0.75 text-left">
            <span class="font-titre text-titre-s font-semibold text-ink">
              {{ t(service.libelle_cle) }}
            </span>
            <!-- Ce que le service suit, nommé concrètement — « Suivi du stock ». Aucune ligne
                 quand il n'y a rien : une liste vide est la forme normale, et elle ne se signale
                 pas.

                 Le mot d'architecture qui désigne ces lignes n'apparaît PAS ici, pas même en
                 commentaire : un commentaire de gabarit part dans le HTML livré, et le test de
                 rendu le voit. C'est délibéré — il vaut mieux qu'il attrape un commentaire de
                 trop que de laisser passer un libellé. -->
            <span
              v-for="capacite in capacitesVisibles(service)"
              :key="capacite.id"
              class="text-mini text-ink-3"
            >
              {{ t(capacite.libelle_cle) }}
            </span>
          </span>

          <!-- COMPOSANT 03 · BOUTON DISCRET, en variante danger. **Absent** sans la permission,
               absent hors ligne — jamais grisé. `aria-label` porte le nom du service : dix boutons
               au libellé identique ne se distinguent pas au lecteur d'écran.

               Aucun libellé n'est cité dans ce commentaire, et c'est volontaire : un commentaire
               de gabarit part dans le HTML rendu, où le test de SC-005 le lit comme du contenu.
               C'est lui qui a attrapé la première rédaction de ces trois lignes. -->
          <button
            v-if="actionsVisibles"
            type="button"
            class="h-9 shrink-0 cursor-pointer rounded-md px-3.5 font-titre text-mini font-semibold text-ink-2 transition-colors duration-90 hover:bg-danger-soft hover:text-danger-fort active:scale-97"
            :disabled="enCours !== null"
            :aria-label="t('etablissement.services.action_retirer_detail', { service: t(service.libelle_cle) })"
            @click="basculer(service.module_code, false)"
          >
            {{ t('etablissement.services.action_retirer') }}
          </button>
        </div>
      </li>

      <!-- Squelette de la ligne QUI ARRIVE — en fin de liste, à la place où elle se posera. -->
      <li v-if="enCours?.sens === 'ajout'">
        <div
          class="flex w-full items-center gap-3 rounded-l-xs rounded-r-xl border border-line border-l-4 border-l-line-2 bg-surf p-3 shadow-basse"
          aria-busy="true"
        >
          <span class="relative size-10 shrink-0 overflow-hidden rounded-xl bg-tile">
            <span
              class="absolute inset-0 bg-linear-to-r from-transparent via-brillance to-transparent animate-scintillement"
            />
          </span>
          <span class="flex min-w-0 flex-1 flex-col gap-2">
            <span class="relative h-3 w-2/5 overflow-hidden rounded-sm bg-tile">
              <span
                class="absolute inset-0 bg-linear-to-r from-transparent via-brillance to-transparent animate-scintillement"
              />
            </span>
            <span class="relative h-2.5 w-1/4 overflow-hidden rounded-sm bg-tile">
              <span
                class="absolute inset-0 bg-linear-to-r from-transparent via-brillance to-transparent animate-scintillement"
              />
            </span>
          </span>
        </div>
      </li>
    </ul>

    <!-- Pied de section, motif de `G2`. Seuls les services IMPLÉMENTÉS et non encore activés sont
         proposables : proposer une valeur non implémentée garantirait un refus 422 que
         l'exploitant n'a aucune raison de rencontrer (FR-036). -->
    <div
      v-if="actionsVisibles && activables.length > 0"
      class="flex shrink-0 flex-col gap-2 border-t border-line bg-surf px-3 pt-2.75 pb-3.5"
    >
      <!-- COMPOSANT 16 · CHAMP DE SAISIE — le formulaire minimal : un choix, et rien de plus. -->
      <template v-if="formulaireOuvert">
        <ChampSaisie
          v-model="moduleChoisi"
          etiquette-cle="etablissement.services.formulaire.champ"
          aide-cle="etablissement.services.formulaire.aide"
          placeholder-cle="etablissement.services.formulaire.invite"
          :erreur-cle="erreurChamp"
          :options="optionsServices"
          :desactive="enCours !== null"
          taille="comptoir"
        />
        <div class="flex items-center gap-2 pt-1">
          <!-- COMPOSANT 01 · BOUTON PRINCIPAL. En cours : le libellé ne change pas et la roue
               s'ajoute — elle ne remplace rien. Le squelette de la liste porte déjà l'attente ;
               deux annonces de la même chose en font lire zéro. -->
          <button
            type="button"
            class="inline-flex h-12 flex-1 cursor-pointer items-center justify-center gap-2.5 rounded-lg bg-prim font-titre text-action font-semibold text-prim-ink shadow-bouton transition-[transform,box-shadow] duration-90 ease-entree active:translate-y-0.5 active:shadow-bouton-appui"
            :disabled="enCours !== null"
            @click="confirmerAjout"
          >
            <span
              v-if="enCours !== null"
              class="size-4 rounded-pleine border-2 border-prim-ink/30 border-t-prim-ink animate-roue"
              aria-hidden="true"
            />
            {{ t('etablissement.services.formulaire.confirmer') }}
          </button>
          <!-- COMPOSANT 02 · BOUTON SECONDAIRE, variante neutre : annuler n'est pas une action de
               produit. Épaisseur 1,5 px — décision n° 1 du README de design. -->
          <button
            type="button"
            class="h-12 min-w-32 cursor-pointer rounded-lg border-[1.5px] border-line-2 bg-transparent px-4.5 font-titre text-action font-semibold text-ink-2 transition-colors duration-90 hover:bg-tile"
            :disabled="enCours !== null"
            @click="fermerFormulaire"
          >
            {{ t('etablissement.services.formulaire.annuler') }}
          </button>
        </div>
      </template>

      <button
        v-else
        type="button"
        class="inline-flex h-13 w-full cursor-pointer items-center justify-center gap-2.5 rounded-xl bg-prim font-titre text-titre-s font-semibold text-prim-ink shadow-bouton-grand transition-[transform,box-shadow] duration-90 ease-entree active:translate-y-0.75 active:shadow-none"
        :disabled="enCours !== null"
        @click="ouvrirFormulaire"
      >
        <i
          class="ph ph-plus text-titre-m"
          aria-hidden="true"
        />
        {{ t('etablissement.services.action_activer') }}
      </button>
    </div>
  </section>
</template>
