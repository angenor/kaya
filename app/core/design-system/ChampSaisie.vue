<script setup lang="ts">
/**
 * **Composant 16 · Champ de saisie** — la pièce canonique de toute écriture du produit.
 *
 * # Pourquoi il n'a pas de maquette, et ce qu'on en a fait
 *
 * Aucun des 29 fichiers de `docs/design/html/` ne contient d'`<input>`, de `<select>` ni de
 * `<textarea>` : les onze écrans maquettés sont tous des écrans de **lecture et de geste**, pas de
 * saisie. `docs/design/composants.md` n'avait donc pas de champ, alors que TRX-08 en annonçait un.
 *
 * Il n'est pas inventé pour autant. **Chacune de ses valeurs sort de `docs/design/tokens.md`**,
 * qui décrit déjà le champ sans le dessiner :
 *
 * | Ce qui est posé ici | Le jeton, et ce qu'il dit littéralement |
 * |---|---|
 * | Bordure au repos | `--color-line-2` — « filet appuyé, **bordure de champ au repos**, ligne de total » |
 * | Rayon | `--radius-md` (8 px) — « **champ**, bouton discret, segment » |
 * | Hauteur | `h-11` (44 px) — « **plancher tactile. Jamais moins** » · `h-12` (48 px) au comptoir |
 * | Corps du texte | `--text-corps` (13,5 px) — « corps de texte, taille du `body` » |
 * | Étiquette | `--text-etiquette` (11 px) — « étiquette en capitales » |
 * | Erreur | `--color-danger` / `-fort`, avec **une forme** (composant 04) |
 * | Épaisseur 1,5 px | Valeur arbitraire **assumée** — `README.md` décision n° 1, celle du bouton secondaire |
 *
 * Deux états n'ont **rien** à déclarer ici, parce que `theme.css` les porte globalement :
 * le **focus clavier** (`:focus-visible` → anneau indigo de 2 px, jamais le bleu du navigateur) et
 * le **désactivé** (`[disabled]` → opacité 0,45, curseur interdit). Les redéclarer produirait une
 * seconde source de vérité pour la même chose.
 *
 * # Un état n'est jamais porté par la couleur seule
 *
 * Règle 2 des couleurs (`tokens.md` §1) et règle transversale de `composants.md`. L'erreur porte
 * donc **trois** signaux et non un : la bordure passe en `danger`, un message apparaît sous le
 * champ, et ce message porte l'icône d'avertissement. Sur un 1366 × 768 délavé par le soleil, la
 * bordure rouge seule ne se voit pas.
 *
 * # Il reçoit des CLÉS i18n, jamais du texte
 *
 * `etiquette-cle`, `aide-cle`, `erreur-cle`. Une chaîne en dur passée en prop afficherait la clé
 * brute à l'écran — la faute se voit au premier rendu, au lieu d'attendre qu'un anglophone ouvre
 * l'application (principe VIII, porte P-16).
 */
import { computed, useId } from 'vue'

const { t } = useI18n()

const modele = defineModel<string>({ required: true })

const props = withDefaults(
  defineProps<{
    /** Clé i18n de l'étiquette. **Toujours visible** : un champ sans étiquette n'existe pas. */
    etiquetteCle: string
    /** Clé i18n d'un texte d'aide, sous le champ. Effacé pendant qu'une erreur s'affiche. */
    aideCle?: string
    /** Clé i18n du message d'erreur. Sa présence **est** l'état d'erreur. */
    erreurCle?: string | null
    /** Valeurs d'interpolation du message d'erreur — un nombre d'obstacles, une valeur refusée. */
    erreurValeurs?: Record<string, unknown>
    /** Clé i18n d'un texte d'invite. Ne remplace **jamais** l'étiquette. */
    placeholderCle?: string
    /**
     * Options d'un choix fermé. Fournies, le champ rend un `<select>` ; absentes, un `<input>`.
     * La même enveloppe porte les deux — étiquette, aide, erreur — parce que c'est l'enveloppe
     * qui fait le champ, pas le contrôle.
     */
    options?: { valeur: string, libelleCle: string }[]
    /**
     * `touche` = 44 px, le plancher tactile. `comptoir` = 48 px, « une main, vite, debout ».
     * Jamais moins de 44, y compris pour un champ qui paraît secondaire.
     */
    taille?: 'touche' | 'comptoir'
    /**
     * Nature du contrôle — **fermée à deux valeurs**, ajoutée par `R0` (CPT-01).
     *
     * `mot_de_passe` masque la saisie. Les autres types HTML (`email`, `tel`, `number`) sont
     * **délibérément absents** : ils déclenchent des claviers et des validations de navigateur qui
     * contrediraient les règles du produit — `identifiant` accepte un numéro **ou** un courriel
     * dans le même champ, et une quantité est un `NUMERIC` dont le séparateur décimal dépend de la
     * locale du navigateur, pas de la nôtre.
     */
    type?: 'texte' | 'mot_de_passe'
    /**
     * Valeur d'`autocomplete` — `username`, `current-password`, `new-password`.
     *
     * Ce n'est pas un confort. Un formulaire de connexion sans ces indications empêche les
     * gestionnaires de mots de passe de fonctionner, et pousse à choisir un mot de passe qu'on
     * retient — donc à l'écrire sur un papier au comptoir. C'est exactement ce que la politique de
     * mot de passe du cycle cherche à éviter en refusant les règles de composition.
     */
    autocompletion?: string
    desactive?: boolean
    lectureSeule?: boolean
    requis?: boolean
  }>(),
  {
    taille: 'touche',
    type: 'texte',
    autocompletion: undefined,
    // Les cinq suivants sont **absents** par défaut, pas vides : `aide-cle: ''` ferait rendre un
    // paragraphe blanc sous chaque champ, et `erreur-cle: ''` un champ en erreur permanente. La
    // règle `vue/require-default-prop` demande une valeur ; `undefined` est celle qui dit « rien ».
    aideCle: undefined,
    erreurCle: undefined,
    erreurValeurs: undefined,
    placeholderCle: undefined,
    options: undefined,
  },
)

const id = useId()
const idAide = `${id}-aide`
const idErreur = `${id}-erreur`

const enErreur = computed(() => Boolean(props.erreurCle))
const message = computed(() =>
  props.erreurCle ? t(props.erreurCle, props.erreurValeurs ?? {}) : '',
)

/**
 * Le champ décrit-il quelque chose à voix haute ?
 *
 * `aria-describedby` ne doit pointer que sur un élément **présent** : une référence morte fait
 * annoncer un identifiant au lieu d'un texte par certains lecteurs d'écran.
 */
const decritPar = computed(() => {
  if (enErreur.value) return idErreur
  return props.aideCle ? idAide : undefined
})

/** Hauteur : `h-11` plancher tactile, `h-12` au comptoir. Aucune troisième valeur. */
const hauteur = computed(() => (props.taille === 'comptoir' ? 'h-12' : 'h-11'))

/**
 * Bordure : `line-2` au repos — le jeton dit littéralement « bordure de champ au repos » —
 * `danger` en erreur, `line` en lecture seule et désactivé, où le filet appuyé mentirait en
 * suggérant qu'on peut écrire.
 */
const bordure = computed(() => {
  if (enErreur.value) return 'border-danger'
  if (props.lectureSeule || props.desactive) return 'border-line'
  return 'border-line-2'
})

/** Fond : `surf` — « feuille posée sur le fond ». `tile` quand le champ ne se saisit pas. */
const fond = computed(() =>
  props.lectureSeule || props.desactive ? 'bg-tile text-ink-2' : 'bg-surf text-ink',
)
</script>

<template>
  <div class="flex flex-col gap-1.5">
    <label
      :for="id"
      class="text-etiquette uppercase text-ink-3"
    >{{ t(etiquetteCle) }}</label>

    <!-- `relative` porte le chevron du choix fermé. Sans options, il n'y a pas de chevron et le
         conteneur ne coûte rien. -->
    <div class="relative flex items-center">
      <select
        v-if="options"
        :id="id"
        v-model="modele"
        class="w-full appearance-none rounded-md border-[1.5px] pr-9 pl-3 font-texte text-corps transition-colors duration-90 ease-entree focus:border-prim"
        :class="[hauteur, bordure, fond, desactive ? 'cursor-not-allowed' : 'cursor-pointer']"
        :disabled="desactive || lectureSeule"
        :required="requis"
        :aria-invalid="enErreur || undefined"
        :aria-describedby="decritPar"
      >
        <option
          v-if="placeholderCle"
          value=""
        >
          {{ t(placeholderCle) }}
        </option>
        <option
          v-for="option in options"
          :key="option.valeur"
          :value="option.valeur"
        >
          {{ t(option.libelleCle) }}
        </option>
      </select>

      <input
        v-else
        :id="id"
        v-model="modele"
        :type="type === 'mot_de_passe' ? 'password' : 'text'"
        :autocomplete="autocompletion"
        class="w-full rounded-md border-[1.5px] px-3 font-texte text-corps transition-colors duration-90 ease-entree placeholder:text-ink-3 focus:border-prim"
        :class="[hauteur, bordure, fond]"
        :placeholder="placeholderCle ? t(placeholderCle) : undefined"
        :disabled="desactive"
        :readonly="lectureSeule"
        :required="requis"
        :aria-invalid="enErreur || undefined"
        :aria-describedby="decritPar"
      >

      <!-- Décoratif : le `<select>` annonce déjà sa nature au lecteur d'écran, et le chevron ne
           se clique pas — c'est le champ entier qui s'ouvre. -->
      <i
        v-if="options"
        class="ph ph-caret-down pointer-events-none absolute right-3 text-corps text-ink-3"
        aria-hidden="true"
      />
    </div>

    <!-- L'aide s'efface pendant l'erreur : deux phrases sous un champ en font lire zéro. -->
    <p
      v-if="aideCle && !enErreur"
      :id="idAide"
      class="text-mini text-ink-3"
    >
      {{ t(aideCle) }}
    </p>

    <!-- COULEUR **PLUS** FORME (composant 04) : bordure `danger`, icône d'avertissement, phrase.
         La bordure rouge seule ne se voit pas en plein soleil, et pas du tout pour un daltonien. -->
    <p
      v-if="enErreur"
      :id="idErreur"
      role="alert"
      class="flex items-start gap-1.5 text-mini text-danger-fort"
    >
      <i
        class="ph-fill ph-warning-circle mt-0.5 shrink-0 text-corps text-danger"
        aria-hidden="true"
      />
      {{ message }}
    </p>
  </div>
</template>
