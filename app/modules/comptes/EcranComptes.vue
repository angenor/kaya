<script setup lang="ts">
/**
 * **`G3` — Utilisateurs et rôles.** Ce que chacun peut faire.
 *
 * Référence visuelle — cas (b), **écran dérivé** : `docs/design/derivation.md` ligne
 * « `G3` Utilisateurs et rôles **hérite de `G2`** — Configuration ». Maquette lue :
 * `docs/design/html/G2-offre-hebergement.html`. **Le HTML de maquette n'est jamais copié**
 * (porte P-19) : on en lit les valeurs — sur-étiquette, `h2` `font-titre text-chiffre`,
 * lignes-boutons `rounded-l-xs rounded-r-xl border-l-4`, bouton principal `h-13 rounded-xl
 * bg-prim` — et on réimplémente.
 *
 * # Les mots « rôle » et « permission » n'apparaissent nulle part
 *
 * `docs/design/lexique.md` : RBAC → « **Ce que chacun peut faire** ». L'utilisateur voit des
 * personnes et ce qu'elles peuvent faire ; la mécanique qui l'autorise reste au code.
 *
 * # Hors ligne, l'action DISPARAÎT et un bandeau dit pourquoi
 *
 * `compte_role` est de **classe C**. La garde vit dans `roles.ts`, pas ici — un second appelant
 * oublierait de la reposer. Ce composant ne fait qu'en refléter la conséquence à l'écran :
 * l'action absente, et un bandeau à sa place. L'état `degrade` compte comme hors ligne.
 *
 * # Sans permission, l'action n'existe pas dans le HTML rendu
 *
 * Aucun `disabled`, aucun `title` explicatif. Le grisé apprend à l'utilisateur qu'une partie du
 * produit lui est refusée, à chaque écran et tous les jours (principe VII).
 */
import { computed, ref } from 'vue'

import ChampSaisie from '~/core/design-system/ChampSaisie.vue'
import { useEtatReseau } from '~/core/platform/reseau'
import { detient, type Permissions } from '~/core/rbac'
import type { ContexteAppel } from '~/core/auth'

import { chargerComptes, type CompteVue, type EntreeRole } from './donnees'
import { attribuerRole, PERMISSION_ATTRIBUER, retirerRole } from './roles'

const { t } = useI18n()

const props = defineProps<{
  comptes: CompteVue[]
  referentielRoles: EntreeRole[]
  contexte: ContexteAppel
  /** L'établissement actif — celui sur lequel les rôles s'attribuent. */
  etablissementId: string
  /** Permissions cumulées de l'utilisateur — union de ses rôles, jamais celles d'un rôle principal. */
  permissions: Permissions
}>()

const emit = defineEmits<{ 'comptes-changes': [CompteVue[]] }>()

const reseau = useEtatReseau()

/** **Absence pure** (principe VII) : sans le droit, l'action n'existe pas, et rien ne le dit. */
const peutAttribuer = computed(() => detient(props.permissions, PERMISSION_ATTRIBUER))

/**
 * **Absence expliquée** (principe VI) : le réseau manque, l'action disparaît **et un bandeau dit
 * pourquoi**. La différence avec le cas précédent n'est pas cosmétique — un droit manquant n'est
 * pas une nouvelle à annoncer, une coupure réseau si.
 */
const reseauRequis = computed(() => reseau.value !== 'connecte')

const actionsVisibles = computed(() => peutAttribuer.value && !reseauRequis.value)

/**
 * Les rôles **attribuables sur cet établissement** — ceux de portée `ETABLISSEMENT`.
 *
 * `admin_editeur` en est absent, et ce n'est pas un filtrage cosmétique : son attribution avec un
 * `etablissement_id` est refusée par le serveur en `422 portee_incompatible`. Le proposer
 * produirait une action qui échoue à tous les coups.
 */
const rolesAttribuables = computed(() =>
  props.referentielRoles
    .filter(role => role.portee === 'ETABLISSEMENT')
    .map(role => ({ valeur: role.code, libelleCle: role.libelle_cle })),
)

/** Code de rôle → clé i18n, résolue une fois depuis le référentiel. */
const libelleParCode = computed(() =>
  Object.fromEntries(props.referentielRoles.map(role => [role.code, role.libelle_cle])),
)

function libelleRole(code: string): string {
  const cle = libelleParCode.value[code]
  // Un code absent du référentiel ne s'affiche pas en brut : ce serait un identifiant technique
  // sous les yeux de l'exploitant. Il tombe sur une phrase générique.
  return cle ? t(cle) : t('comptes.role_inconnu')
}

/**
 * Les rôles d'un compte **sur l'établissement affiché**, plus ceux de portée éditeur.
 *
 * Un même compte peut être caissier ici et réceptionniste là : afficher tous ses rôles sur l'écran
 * d'un établissement donnerait une liste fausse de ce qu'il peut y faire.
 */
function rolesIci(compte: CompteVue): string[] {
  return compte.roles
    .filter(role => !role.etablissement_id || role.etablissement_id === props.etablissementId)
    .map(role => role.role_code)
}

/**
 * L'opération en cours, avec **son sens et sa cible** (composant 13).
 *
 * Un simple booléen ne suffirait pas : à l'attribution, la forme qui arrive est une étiquette **en
 * fin de liste** ; au retrait, c'est **l'étiquette concernée** qui s'en va. Un indicateur générique
 * ne dirait ni l'un ni l'autre.
 */
const enCours = ref<{ sens: 'attribution' | 'retrait', compteId: string, roleCode: string } | null>(null)

/** Résultat de la dernière écriture. Rendu par le composant 07. */
const resultat = ref<{ ton: 'succes' | 'danger', cle: string, valeurs?: Record<string, unknown> } | null>(null)

/**
 * **Le bandeau affiché — un seul, jamais deux empilés** (composant 07).
 *
 * La règle est tenue par un `computed`, pas par deux `v-if` que le prochain état viendrait
 * contredire. **Hors ligne gagne** : il concerne l'écran entier là où le résultat concerne une
 * ligne, et il rend le résultat précédent caduc.
 */
const bandeau = computed(() => {
  if (reseauRequis.value && peutAttribuer.value) {
    return { ton: 'alerte' as const, cle: 'comptes.refus.reseau' }
  }
  return resultat.value
})

/** Ton → fond, contrefort, texte et icône. Trois tons, une seule structure. */
const TONS = {
  succes: { fond: 'border-l-succes bg-succes-soft', texte: 'text-succes-fort', icone: 'ph-check-circle text-succes' },
  alerte: { fond: 'border-l-alerte bg-alerte-soft', texte: 'text-alerte-fort', icone: 'ph-warning text-alerte' },
  danger: { fond: 'border-l-danger bg-danger-soft', texte: 'text-danger-fort', icone: 'ph-x-circle text-danger' },
} as const

/** Le compte dont le formulaire d'attribution est ouvert. Un seul à la fois. */
const formulairePour = ref<string | null>(null)
const roleChoisi = ref('')
/** Erreur **au champ**, pas au bandeau : elle porte sur ce qui est saisi. */
const erreurChamp = ref<string | null>(null)

function ouvrirFormulaire(compteId: string): void {
  formulairePour.value = compteId
  roleChoisi.value = ''
  erreurChamp.value = null
  resultat.value = null
}

function fermerFormulaire(): void {
  formulairePour.value = null
  roleChoisi.value = ''
  erreurChamp.value = null
}

/** Relit la liste depuis le serveur — **sans rechargement de page**. */
async function rafraichir(): Promise<void> {
  const donnees = await chargerComptes(props.contexte, props.etablissementId)
  emit('comptes-changes', donnees.comptes)
}

async function confirmerAttribution(compteId: string): Promise<void> {
  // **Validation au champ**, avant tout appel.
  if (!roleChoisi.value) {
    erreurChamp.value = 'champ.erreur.obligatoire'
    return
  }

  await ecrire('attribution', compteId, roleChoisi.value, () =>
    attribuerRole(props.contexte, compteId, roleChoisi.value, props.etablissementId, reseau.value),
  )
}

async function retirer(compteId: string, roleCode: string): Promise<void> {
  await ecrire('retrait', compteId, roleCode, () =>
    retirerRole(props.contexte, compteId, roleCode, props.etablissementId, reseau.value),
  )
}

/**
 * Le geste unique, dans les deux sens.
 *
 * Ordre des opérations, et le point qu'on écrirait mal : **le rafraîchissement suit le succès**,
 * il ne le précède pas. Relire avant que le serveur ait tranché afficherait l'état d'avant en
 * donnant l'impression qu'il s'agit de celui d'après.
 */
async function ecrire(
  sens: 'attribution' | 'retrait',
  compteId: string,
  roleCode: string,
  appel: () => Promise<{ issue: 'succes' } | { issue: 'refus', cle: string, valeurs?: Record<string, unknown> }>,
): Promise<void> {
  enCours.value = { sens, compteId, roleCode }
  resultat.value = null

  try {
    // Nommée `issue`, jamais `resultat` : ce dernier est le `ref` du bandeau, et une variable
    // locale du même nom le masquerait sans qu'aucune règle de lint ne s'en plaigne.
    const issue = await appel()

    if (issue.issue === 'refus') {
      // **Le message est traduit d'une CLÉ, jamais du texte du serveur** — qui nomme des tables
      // et parle anglais technique.
      resultat.value = {
        ton: 'danger',
        cle: issue.cle,
        valeurs: { ...issue.valeurs, role: libelleRole(roleCode) },
      }
      return
    }

    await rafraichir()
    resultat.value = {
      ton: 'succes',
      cle: sens === 'attribution' ? 'comptes.succes.attribution' : 'comptes.succes.retrait',
      valeurs: { role: libelleRole(roleCode) },
    }
    fermerFormulaire()
  }
  catch {
    // Une coupure survenue **pendant** l'appel : la garde hors-ligne évite l'attente inutile, elle
    // ne remplace pas le traitement d'erreur (`core/platform/reseau.ts`).
    resultat.value = { ton: 'danger', cle: 'comptes.refus.inattendue' }
  }
  finally {
    enCours.value = null
  }
}
</script>

<template>
  <section class="flex flex-col">
    <div class="flex flex-col gap-1.5 px-3.5 pt-4 pb-1">
      <span class="text-etiquette uppercase text-ink-3">{{ t('comptes.sur_titre') }}</span>
      <h2 class="font-titre text-chiffre font-semibold text-ink">
        {{ t('comptes.titre') }}
      </h2>
      <p class="text-corps text-ink-2">
        {{ t('comptes.intro') }}
      </p>
    </div>

    <!-- COMPOSANT 07 · BANDEAU — un seul, jamais deux empilés. -->
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
      <p
        class="text-corps"
        :class="TONS[bandeau.ton].texte"
      >
        {{ t(bandeau.cle, ('valeurs' in bandeau ? bandeau.valeurs : undefined) ?? {}) }}
      </p>
    </div>

    <!-- État vide explicite — jamais une liste blanche sans explication. -->
    <p
      v-if="comptes.length === 0"
      class="px-3.5 py-6 text-corps text-ink-3"
    >
      {{ t('comptes.aucun') }}
    </p>

    <ul class="flex flex-col gap-2.25 px-3 pt-3 pb-3.5">
      <li
        v-for="compte in comptes"
        :key="compte.id"
        class="flex w-full flex-col gap-2.5 rounded-l-xs rounded-r-xl border border-line border-l-4 border-l-line-2 bg-surf p-3 shadow-basse"
      >
        <div class="flex items-center gap-3">
          <span class="inline-flex size-10 shrink-0 items-center justify-center rounded-xl bg-tile">
            <i
              class="ph ph-user text-titre-s text-ink-2"
              aria-hidden="true"
            />
          </span>
          <div class="flex min-w-0 flex-1 flex-col">
            <span class="truncate font-titre text-titre-s font-semibold text-ink">
              {{ compte.nom_affichage }}
            </span>
            <!-- Un compte désactivé le DIT. Ce n'est pas un grisé : c'est un état de la donnée,
                 pas une action refusée. -->
            <span
              v-if="!compte.actif"
              class="text-mini text-ink-3"
            >{{ t('comptes.desactive') }}</span>
          </div>
        </div>

        <!-- Ce que cette personne peut faire ici. Les codes sont traduits par le référentiel ;
             aucun identifiant technique n'atteint l'écran. -->
        <div class="flex flex-wrap items-center gap-1.5">
          <span
            v-if="rolesIci(compte).length === 0"
            class="text-mini text-ink-3"
          >{{ t('comptes.aucun_role') }}</span>

          <span
            v-for="code in rolesIci(compte)"
            :key="code"
            class="inline-flex items-center gap-1.5 rounded-md bg-tile px-2.5 py-1 text-mini text-ink-2"
          >
            <!-- COMPOSANT 13 · SQUELETTE — au retrait, c'est l'étiquette CONCERNÉE qui prend la
                 forme de ce qui s'en va. Une roue au milieu de l'écran ne dirait pas laquelle. -->
            <span
              v-if="enCours?.sens === 'retrait' && enCours.compteId === compte.id && enCours.roleCode === code"
              class="h-3 w-16 animate-pulse rounded-full bg-line-2"
              aria-hidden="true"
            />
            <template v-else>
              {{ libelleRole(code) }}
              <button
                v-if="actionsVisibles"
                type="button"
                class="cursor-pointer text-ink-3 transition-colors duration-90 hover:text-danger-fort"
                :aria-label="t('comptes.action_retirer_detail', { role: libelleRole(code) })"
                :disabled="enCours !== null"
                @click="retirer(compte.id, code)"
              >
                <i
                  class="ph ph-x text-corps"
                  aria-hidden="true"
                />
              </button>
            </template>
          </span>

          <!-- Squelette d'ajout, EN FIN DE LISTE — là où l'étiquette se posera. -->
          <span
            v-if="enCours?.sens === 'attribution' && enCours.compteId === compte.id"
            class="h-6 w-24 animate-pulse rounded-md bg-tile"
            aria-hidden="true"
          />
        </div>

        <template v-if="actionsVisibles">
          <template v-if="formulairePour === compte.id">
            <ChampSaisie
              v-model="roleChoisi"
              etiquette-cle="comptes.formulaire.champ"
              placeholder-cle="comptes.formulaire.invite"
              aide-cle="comptes.formulaire.aide"
              :erreur-cle="erreurChamp"
              :options="rolesAttribuables"
              :desactive="enCours !== null"
              taille="comptoir"
            />
            <div class="flex items-center gap-2 pt-1">
              <button
                type="button"
                class="inline-flex h-12 flex-1 cursor-pointer items-center justify-center gap-2.5 rounded-lg bg-prim font-titre text-action font-semibold text-prim-ink shadow-bouton transition-[transform,box-shadow] duration-90 ease-entree active:translate-y-0.5 active:shadow-bouton-appui"
                :disabled="enCours !== null"
                @click="confirmerAttribution(compte.id)"
              >
                <span
                  v-if="enCours !== null"
                  class="size-4 rounded-pleine border-2 border-prim-ink/30 border-t-prim-ink animate-roue"
                  aria-hidden="true"
                />
                {{ t('comptes.formulaire.confirmer') }}
              </button>
              <button
                type="button"
                class="h-12 min-w-32 cursor-pointer rounded-lg border-[1.5px] border-line-2 bg-transparent px-4.5 font-titre text-action font-semibold text-ink-2 transition-colors duration-90 hover:bg-tile"
                :disabled="enCours !== null"
                @click="fermerFormulaire"
              >
                {{ t('comptes.formulaire.annuler') }}
              </button>
            </div>
          </template>

          <button
            v-else
            type="button"
            class="inline-flex h-11 w-full cursor-pointer items-center justify-center gap-2 rounded-lg border-[1.5px] border-line-2 bg-transparent font-titre text-action font-semibold text-ink-2 transition-colors duration-90 hover:bg-tile"
            :disabled="enCours !== null"
            @click="ouvrirFormulaire(compte.id)"
          >
            <i
              class="ph ph-plus text-corps"
              aria-hidden="true"
            />
            {{ t('comptes.action_ajouter') }}
          </button>
        </template>
      </li>
    </ul>
  </section>
</template>
