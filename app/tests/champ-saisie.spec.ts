// @vitest-environment happy-dom
/**
 * **Composant 16 · Champ de saisie — vérifié pour lui-même.**
 *
 * # Pourquoi ce fichier existe
 *
 * `ChampSaisie.vue` est la pièce de **toute écriture** du produit : les vingt et une opérations
 * d'écriture des cycles à venir passeront par lui. Jusqu'ici, il n'était exercé
 * qu'**indirectement**, par `ecran-g1.spec.ts`, et dans un seul de ses états — le choix fermé du
 * formulaire d'ajout de service. Ses états `readonly`, `disabled` et `comptoir` n'étaient vérifiés
 * qu'à l'œil, sur le styleguide.
 *
 * Un composant canonique vérifié à l'œil est un composant dont la prochaine modification casse un
 * état que personne ne regarde. Ce fichier prend **un cas par état déclaré** dans
 * `docs/design/composants.md` n° 16 — repos, focus, saisie, erreur, désactivé, lecture seule,
 * variante comptoir, choix fermé — plus la règle transversale du composant 04.
 *
 * # Ce que ce fichier n'inspecte PAS, et pourquoi
 *
 * **L'apparence.** Un test de HTML lit des classes utilitaires, pas des pixels : il constate que
 * `border-danger` est posée, jamais que la bordure est rouge à l'écran. Il ne peut pas davantage
 * juger la bascule clair/sombre, dont les valeurs changent sous `.dark` sans que le HTML bouge.
 * C'est la limite assumée, et c'est pourquoi la vérification des deux thèmes reste faite à l'œil
 * (`theme-sombre.spec.ts` et T044) et que P-17 garde les valeurs littérales.
 *
 * **Le focus visible.** `:focus-visible` est une pseudo-classe CSS, portée globalement par
 * `theme.css`. happy-dom n'applique aucune feuille : le test vérifie donc ce qui est vérifiable —
 * que le champ **peut** recevoir le focus, et que le composant ne redéclare pas d'anneau qui
 * ferait doublon avec `theme.css`.
 */

import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'

import ChampSaisie from '../core/design-system/ChampSaisie.vue'
import fr from '../core/i18n/fr.json'

/**
 * Traduction depuis le **catalogue français réel**, comme dans `ecran-g1.spec.ts`.
 *
 * Un faux qui renverrait la clé ferait passer chaque test alors que le libellé serait absent du
 * catalogue : le composant afficherait `etablissement.services.formulaire.champ` à l'écran et rien
 * ne le dirait (principe VIII, porte P-16).
 */
function traduire(cle: string, valeurs?: Record<string, unknown>): string {
  const brut = cle
    .split('.')
    .reduce<unknown>((noeud, part) => (noeud as Record<string, unknown>)?.[part], fr)

  if (typeof brut !== 'string') return cle
  return brut.replace(/\{(\w+)\}/g, (_, nom: string) => String(valeurs?.[nom] ?? ''))
}

// `useI18n` est un auto-import Nuxt : hors du pipeline, il faut l'exposer globalement.
;(globalThis as Record<string, unknown>).useI18n = () => ({ t: traduire })

/** Trois clés réelles du catalogue, celles que `SectionServices.vue` passe déjà au composant. */
const ETIQUETTE = 'etablissement.services.formulaire.champ'
const AIDE = 'etablissement.services.formulaire.aide'
const INVITE = 'etablissement.services.formulaire.invite'
/** Une clé de refus réelle, employée ici comme message d'erreur au champ. */
const ERREUR = 'etablissement.services.refus.module_inconnu'

function monter(props: Record<string, unknown> = {}) {
  return mount(ChampSaisie, {
    props: { modelValue: '', etiquetteCle: ETIQUETTE, ...props },
  })
}

/**
 * Montage **attaché au document**, indispensable aux trois tests de focus.
 *
 * Hors du document, `element.focus()` ne fait rien et `document.activeElement` reste `<body>` :
 * un test de focus non attaché passe pour la mauvaise raison — il constate l'absence de focus sur
 * un champ qui ne pouvait pas en recevoir. C'est ainsi que la première version de ce fichier
 * « vérifiait » qu'un champ désactivé n'est pas focusable.
 */
function monterAttache(props: Record<string, unknown> = {}) {
  return mount(ChampSaisie, {
    props: { modelValue: '', etiquetteCle: ETIQUETTE, ...props },
    attachTo: document.body,
  })
}

// =================================================================================================
//  Repos — l'état par défaut, et le seul dont tous les autres se distinguent
// =================================================================================================

describe('état de repos', () => {
  it('rend un <input>, pas un <select>, tant qu’aucune option n’est fournie', () => {
    const champ = monter()

    expect(champ.find('input').exists()).toBe(true)
    expect(champ.find('select').exists()).toBe(false)
  })

  it('porte la bordure de champ au repos — le jeton dit littéralement « bordure de champ au repos »', () => {
    const classes = monter().find('input').classes()

    expect(classes).toContain('border-line-2')
    expect(classes).not.toContain('border-danger')
    expect(classes).toContain('bg-surf')
  })

  it('affiche son étiquette, traduite, et la relie au champ par for/id', () => {
    const champ = monter()
    const etiquette = champ.find('label')

    // « Un champ sans étiquette n'existe pas », et une étiquette non reliée n'est pas annoncée par
    // un lecteur d'écran : les deux moitiés de la règle sont vérifiées ensemble.
    expect(etiquette.text()).toBe('Service à ajouter')
    expect(etiquette.attributes('for')).toBe(champ.find('input').attributes('id'))
    expect(etiquette.attributes('for')).toBeTruthy()
  })

  it('n’annonce ni erreur ni description quand il n’a rien à décrire', () => {
    const champ = monter()

    // `aria-describedby` ne doit pointer que sur un élément PRÉSENT : une référence morte fait
    // annoncer un identifiant au lieu d'un texte par certains lecteurs d'écran.
    expect(champ.find('input').attributes('aria-describedby')).toBeUndefined()
    expect(champ.find('input').attributes('aria-invalid')).toBeUndefined()
    expect(champ.find('[role="alert"]').exists()).toBe(false)
  })

  it('fait 44 px de haut — plancher tactile, jamais moins', () => {
    expect(monter().find('input').classes()).toContain('h-11')
  })

  it('rend son texte d’aide, et le relie au champ', () => {
    const champ = monter({ aideCle: AIDE })
    const aide = champ.find('p')

    expect(aide.text()).toBe('Vous pourrez le retirer plus tard : rien ne sera supprimé.')
    expect(champ.find('input').attributes('aria-describedby')).toBe(aide.attributes('id'))
  })
})

// =================================================================================================
//  Focus — ce que le composant déclare, et ce qu'il laisse à theme.css
// =================================================================================================

describe('état de focus', () => {
  it('peut recevoir le focus', () => {
    const champ = monterAttache()
    const element = champ.find('input').element as HTMLInputElement

    element.focus()

    expect(document.activeElement).toBe(element)
  })

  it('n’ajoute que la bordure indigo, et ne redéclare PAS l’anneau de theme.css', () => {
    const classes = monter().find('input').classes()

    // « L'indigo est l'action, et un champ actif se touche » — mais `:focus-visible` et son anneau
    // de 2 px sont portés GLOBALEMENT par `theme.css`. Les redéclarer ici créerait une seconde
    // source de vérité pour la même chose, et c'est l'erreur que ce test rend impossible à commettre
    // en silence.
    expect(classes).toContain('focus:border-prim')
    expect(classes.filter(c => c.includes('ring'))).toEqual([])
    expect(classes.filter(c => c.startsWith('outline'))).toEqual([])
  })
})

// =================================================================================================
//  Saisie
// =================================================================================================

describe('état de saisie', () => {
  it('remonte ce qui est tapé par le modèle', async () => {
    const champ = monter()

    await champ.find('input').setValue('Résidence Deloria')

    expect(champ.emitted('update:modelValue')?.at(-1)).toEqual(['Résidence Deloria'])
  })

  it('garde son étiquette visible pendant la saisie — l’invite ne la remplace jamais', async () => {
    const champ = monter({ placeholderCle: INVITE })

    expect(champ.find('input').attributes('placeholder')).toBe('Choisissez un service')

    await champ.find('input').setValue('Abengourou')

    // Un champ dont l'étiquette disparaît à la saisie est un champ dont on ne sait plus ce qu'il
    // attend. Le texte d'invite ne la remplace ni avant ni après.
    expect(champ.find('label').text()).toBe('Service à ajouter')
  })
})

// =================================================================================================
//  Erreur — et la règle transversale : jamais la couleur seule
// =================================================================================================

describe('état d’erreur', () => {
  it('bascule la bordure en danger, et annonce l’invalidité', () => {
    const champ = monter({ erreurCle: ERREUR })
    const saisie = champ.find('input')

    expect(saisie.classes()).toContain('border-danger')
    expect(saisie.classes()).not.toContain('border-line-2')
    expect(saisie.attributes('aria-invalid')).toBe('true')
  })

  it('affiche le message traduit, dans un role="alert" relié au champ', () => {
    const champ = monter({ erreurCle: ERREUR })
    const message = champ.find('[role="alert"]')

    expect(message.text()).toContain("Ce service n'existe pas.")
    expect(champ.find('input').attributes('aria-describedby')).toBe(message.attributes('id'))
  })

  it('efface l’aide pendant l’erreur — deux phrases sous un champ en font lire zéro', () => {
    const champ = monter({ aideCle: AIDE, erreurCle: ERREUR })

    expect(champ.text()).not.toContain('Vous pourrez le retirer plus tard')
    expect(champ.text()).toContain("Ce service n'existe pas.")
  })

  it("interpole les valeurs du message plutôt que d'afficher {service}", () => {
    const champ = monter({
      erreurCle: 'etablissement.services.action_retirer_detail',
      erreurValeurs: { service: 'Bar' },
    })

    expect(champ.find('[role="alert"]').text()).toContain('Retirer Bar')
    expect(champ.text()).not.toContain('{service}')
  })

  /**
   * **La règle transversale du composant 04 : « un état n'est jamais porté par la couleur seule. »**
   *
   * Sur un 1366 × 768 délavé par le soleil, la bordure rouge seule ne se voit pas — et pas du tout
   * pour un daltonien. Le test le vérifie de la seule manière opposable : en retirant du rendu
   * **tout ce qui est une couleur**, et en constatant que l'erreur reste détectable.
   */
  it('reste détectable une fois toute couleur retirée du rendu', () => {
    const champ = monter({ erreurCle: ERREUR })

    // Chaque classe utilitaire qui porte une couleur est effacée du HTML : `border-danger`,
    // `text-danger-fort`, `text-danger`. Ce qui subsiste est ce qu'un daltonien perçoit.
    const sansCouleur = champ.html().replace(/\b\w[\w:./[\]-]*-(danger|succes|alerte)(-\w+)?\b/g, '')

    expect(sansCouleur).toMatch(/role="alert"/)
    expect(sansCouleur).toMatch(/aria-invalid="true"/)
    expect(sansCouleur).toContain("Ce service n'existe pas.")
    // La FORME : l'icône d'avertissement, décorative pour le lecteur d'écran — qui, lui, a déjà
    // `role="alert"` — mais porteuse pour qui regarde.
    expect(sansCouleur).toContain('ph-warning-circle')
  })

  it("l'icône du message est décorative, pas annoncée deux fois", () => {
    const champ = monter({ erreurCle: ERREUR })
    const icone = champ.find('.ph-warning-circle')

    expect(icone.attributes('aria-hidden')).toBe('true')
  })
})

// =================================================================================================
//  Désactivé et lecture seule — deux états distincts, souvent confondus
// =================================================================================================

describe('état désactivé', () => {
  it('pose disabled, et le champ ne peut plus recevoir le focus', () => {
    const champ = monterAttache({ desactive: true })
    const element = champ.find('input').element as HTMLInputElement

    expect(element.disabled).toBe(true)

    element.focus()
    expect(document.activeElement).not.toBe(element)
  })

  it('passe sur le fond de tuile et le filet léger', () => {
    const classes = monter({ desactive: true }).find('input').classes()

    // Le filet appuyé mentirait en suggérant qu'on peut écrire.
    expect(classes).toContain('bg-tile')
    expect(classes).toContain('border-line')
    expect(classes).not.toContain('border-line-2')
  })

  it('ne redéclare PAS l’opacité — theme.css la porte sur [disabled]', () => {
    const classes = monter({ desactive: true }).find('input').classes()

    expect(classes.filter(c => c.startsWith('opacity'))).toEqual([])
    expect(classes.filter(c => c.includes('cursor-not-allowed'))).toEqual([])
  })
})

describe('état de lecture seule', () => {
  it('pose readonly et NON disabled — la lecture seule se sélectionne et se copie', () => {
    const champ = monterAttache({ lectureSeule: true })
    const element = champ.find('input').element as HTMLInputElement

    expect(element.readOnly).toBe(true)
    expect(element.disabled).toBe(false)

    // La distinction n'est pas cosmétique : un champ désactivé est retiré de l'ordre de tabulation
    // et son contenu ne se copie pas. Confondre les deux rend illisible un montant qu'on voulait
    // seulement protéger.
    element.focus()
    expect(document.activeElement).toBe(element)
  })

  it('partage l’apparence du désactivé, sans en partager le comportement', () => {
    const classes = monter({ lectureSeule: true }).find('input').classes()

    expect(classes).toContain('bg-tile')
    expect(classes).toContain('border-line')
  })
})

// =================================================================================================
//  Variante comptoir
// =================================================================================================

describe('variante comptoir', () => {
  it('passe à 48 px — « une main, vite, debout »', () => {
    const classes = monter({ taille: 'comptoir' }).find('input').classes()

    expect(classes).toContain('h-12')
    expect(classes).not.toContain('h-11')
  })

  it('n’a que deux hauteurs, jamais une troisième', () => {
    for (const [taille, attendue] of [['touche', 'h-11'], ['comptoir', 'h-12']] as const) {
      const classes = monter({ taille }).find('input').classes()
      const hauteurs = classes.filter(c => /^h-\d+$/.test(c))

      expect(hauteurs).toEqual([attendue])
    }
  })
})

// =================================================================================================
//  Choix fermé — même enveloppe, autre contrôle
// =================================================================================================

describe('choix fermé', () => {
  const OPTIONS = [
    { valeur: 'BAR', libelleCle: 'services.modules.BAR' },
    { valeur: 'PRESSING', libelleCle: 'services.modules.PRESSING' },
  ]

  it('rend un <select> dès qu’il reçoit des options', () => {
    const champ = monter({ options: OPTIONS })

    expect(champ.find('select').exists()).toBe(true)
    expect(champ.find('input').exists()).toBe(false)
  })

  it('traduit chaque option, et place l’invite en tête avec une valeur vide', () => {
    const champ = monter({ options: OPTIONS, placeholderCle: INVITE })
    const options = champ.findAll('option')

    expect(options[0]?.text()).toBe('Choisissez un service')
    expect(options[0]?.attributes('value')).toBe('')
    expect(options.map(o => o.text())).toEqual(['Choisissez un service', 'Bar', 'Pressing'])
  })

  it('garde l’enveloppe du champ — étiquette, aide, erreur', () => {
    const champ = monter({ options: OPTIONS, aideCle: AIDE, erreurCle: ERREUR })

    // « C'est l'enveloppe qui fait le champ, pas le contrôle. »
    expect(champ.find('label').text()).toBe('Service à ajouter')
    expect(champ.find('select').classes()).toContain('border-danger')
    expect(champ.find('select').attributes('aria-invalid')).toBe('true')
    expect(champ.find('[role="alert"]').exists()).toBe(true)
  })

  it('porte un chevron décoratif qui ne capte pas le clic', () => {
    const champ = monter({ options: OPTIONS })
    const chevron = champ.find('.ph-caret-down')

    // Le `<select>` annonce déjà sa nature au lecteur d'écran, et le chevron ne se clique pas —
    // c'est le champ entier qui s'ouvre.
    expect(chevron.attributes('aria-hidden')).toBe('true')
    expect(chevron.classes()).toContain('pointer-events-none')
  })

  it('remonte le choix par le modèle', async () => {
    const champ = monter({ options: OPTIONS })

    await champ.find('select').setValue('PRESSING')

    expect(champ.emitted('update:modelValue')?.at(-1)).toEqual(['PRESSING'])
  })

  it('se ferme à la saisie en lecture seule — un <select> n’a pas de readonly', () => {
    // C'est la seule différence assumée entre les deux contrôles : `readonly` n'existe pas sur un
    // `<select>`, donc la lecture seule y passe par `disabled`. L'écrire en test évite qu'on
    // « corrige » un jour le composant en croyant à un oubli.
    const champ = monter({ options: OPTIONS, lectureSeule: true })

    expect((champ.find('select').element as HTMLSelectElement).disabled).toBe(true)
  })
})

// =================================================================================================
//  Il reçoit des CLÉS, jamais du texte
// =================================================================================================

describe('interface en clés i18n', () => {
  it('affiche le libellé du catalogue, pas la clé', () => {
    const champ = monter({ aideCle: AIDE, placeholderCle: INVITE })

    expect(champ.text()).not.toContain('etablissement.services')
    expect(champ.find('input').attributes('placeholder')).not.toContain('etablissement.')
  })

  it('une clé absente du catalogue se voit à l’écran, elle ne se tait pas', () => {
    // Le comportement est voulu : une chaîne en dur passée en prop afficherait la clé brute au
    // premier rendu, au lieu d'attendre qu'un anglophone ouvre l'application (porte P-16).
    const champ = monter({ etiquetteCle: 'clé.qui.nexiste.pas' })

    expect(champ.find('label').text()).toBe('clé.qui.nexiste.pas')
  })
})
