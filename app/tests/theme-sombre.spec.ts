/**
 * **Definition of Done, point 8 — `G1` vérifié en mode clair ET en mode sombre.**
 *
 * Le point était **sans objet au cycle 001** faute d'écran ; il devient exigible ici, et c'est la
 * dette que ce cycle solde.
 *
 * # Ce que ce fichier vérifie, et ce qu'il ne peut pas vérifier
 *
 * **Vérifié, mécaniquement** — que chaque jeton de couleur employé par les composants de `G1`
 * porte bien **une valeur sous `.dark`**. C'est la condition qui fait qu'un composant bascule
 * « tout seul » : les noms de jetons sont identiques dans les deux thèmes, seules les valeurs
 * changent (`theme.css`, règle de lecture 1). Un jeton défini en clair mais absent du bloc sombre
 * garde sa valeur claire — texte foncé sur fond foncé, et l'écart ne se voit qu'en basculant.
 *
 * **Non vérifié ici** — le **rendu visuel** : contrastes, lisibilité, harmonie. Aucun test ne
 * peut en juger. Cette partie a été faite à l'œil, section par section (T044), et ce fichier
 * garantit qu'elle ne régressera pas silencieusement.
 *
 * # Aucune palette dupliquée
 *
 * Le principe XII interdit une seconde palette : le mode sombre passe par la variante `dark:` et
 * par des jetons dont les valeurs changent. Ce fichier vérifie donc aussi qu'aucun composant
 * n'emploie de classe `dark:bg-…` ou `dark:text-…` **de couleur** — la variante `dark:` ne sert
 * qu'à ce qu'une couleur ne peut pas porter : une ombre remplacée par une bordure, une opacité,
 * une épaisseur de trait.
 */

import { readFileSync, readdirSync } from 'node:fs'
import { join } from 'node:path'

import { describe, expect, it } from 'vitest'

const RACINE = new URL('..', import.meta.url).pathname
const THEME = readFileSync(join(RACINE, 'assets/css/theme.css'), 'utf8')

/** Le bloc `.dark { … }` de `theme.css`. */
function blocSombre(): string {
  const debut = THEME.indexOf('.dark {')
  expect(debut, 'le bloc `.dark` a disparu de theme.css — le mode sombre n’existe plus').toBeGreaterThan(-1)
  const fin = THEME.indexOf('\n}', debut)
  return THEME.slice(debut, fin)
}

/**
 * Les composants soumis au contrôle : **tous les modules métier, le système de design, et les
 * pages qui portent du rendu**.
 *
 * `core/design-system` a été ajouté avec le composant 16 : c'est la pièce réutilisée par tous les
 * cycles suivants, donc celle dont un défaut de mode sombre se propagerait le plus loin.
 *
 * Le cycle 003 ajoute **trois modules et deux pages**. Les pages `comptes.vue` et
 * `journal-audit.vue` ne sont pas des coquilles vides : leur état d'erreur et de chargement porte
 * du fond et du texte, donc des jetons — et c'est l'écran qu'on voit quand quelque chose ne va
 * pas, celui où un texte illisible en mode sombre coûte le plus.
 *
 * **Le décompte est vérifié** : un répertoire renommé rendrait zéro fichier, et la porte passerait
 * au vert en n'inspectant rien.
 */
const REPERTOIRES_INSPECTES = [
  'modules/etablissements',
  'modules/comptes',
  'modules/audit',
  'modules/accueil',
  // Cycle 004 — la première verticale. Ses deux écrans portent des jetons de couleur comme les
  // autres, et l'un d'eux — `G5` — est **composé** : il n'a pas de maquette contre laquelle
  // comparer un rendu, donc le contrôle mécanique des jetons y compte davantage encore.
  'modules/hebergement',
  // Cycle 006 — le passage. **Zone de vitesse** : ses composants portent l'état d'une chambre en
  // trois tons, et c'est exactement le genre d'endroit où une seconde palette se glisserait —
  // la tentation d'écrire `dark:bg-vert-800` « juste pour ce cas ».
  'modules/sejours',
  'core/design-system',
  'pages',
] as const

/** Fichiers exemptés, **nommés un par un**, jamais par motif. */
const EXEMPTES = [
  // Surface de développement, retirée du routeur en production — même exemption nommée que
  // P-16, et bornée par la même contrepartie. Elle affiche délibérément les deux thèmes
  // côte à côte, donc porte des jetons dans les deux sens.
  'styleguide.vue',
]

function composantsG1(): { nom: string, contenu: string }[] {
  const composants = REPERTOIRES_INSPECTES.flatMap((relatif) => {
    const fichiers = readdirSync(join(RACINE, relatif)).filter(f => f.endsWith('.vue'))

    expect(
      fichiers.length,
      `« ${relatif} » ne rend aucun composant : la porte y passerait au vert sans rien inspecter`,
    ).toBeGreaterThan(0)

    return fichiers
      .filter(nom => !EXEMPTES.includes(nom))
      .map(nom => ({ nom, contenu: readFileSync(join(RACINE, relatif, nom), 'utf8') }))
  })

  expect(composants.length).toBeGreaterThanOrEqual(12)
  return composants
}

/**
 * Les jetons de **couleur** employés par un composant.
 *
 * # Cinq pièges d'extraction, neutralisés ici
 *
 * 1. **Les préfixes directionnels.** `border-l-line-2` porte le jeton `line-2`, pas `l-line-2`.
 *    Sans traitement, chaque bordure gauche produirait un faux jeton introuvable dans le thème.
 * 2. **Les largeurs de bordure.** `border-l-4` et `border-t` ne nomment aucune couleur — la
 *    première est une épaisseur, la seconde un simple trait. Les compter ferait échouer la porte
 *    sur des utilitaires parfaitement corrects.
 * 3. **Les échelles de taille.** `text-corps`, `text-titre-s` sont des corps de texte, pas des
 *    couleurs, et n'ont rien à faire dans le bloc sombre.
 * 4. **Les types de dégradé.** `bg-linear-to-r` nomme une *direction de dégradé*, pas une couleur —
 *    le jeton du dégradé est porté par `via-brillance`, pas par `bg-…`. Relevé au cycle de la
 *    couche d'écriture, sur le squelette de chargement, qui est le premier composant du produit à
 *    employer un dégradé.
 * 5. **Les propriétés CSS nommées dans une transition arbitraire.**
 *    `transition-[transform,border-color]` contient littéralement « border-color », que le motif
 *    capture comme le jeton « color ». Relevé sur l'accueil `R1`, dont les tuiles animent leur
 *    bordure au survol — la maquette le fait, et le premier écran qui la reprend l'a fait tomber.
 *    Un `[…]` entre crochets est une valeur arbitraire de Tailwind : elle nomme des propriétés
 *    CSS, jamais des jetons de couleur.
 * 6. **Les STYLES de bordure.** `border-dashed` nomme un trait discontinu, pas une couleur —
 *    `solid`, `dotted` et `double` sont dans le même cas. Relevé au cycle 004, sur l'encart « le
 *    passage n'est pas proposé ici » de `G2`, premier composant du produit à employer un trait
 *    discontinu. La maquette le fait ; le premier écran qui la reprend a fait tomber la porte.
 *
 * Une extraction qui produit des faux positifs est une porte qu'on désactive dans la semaine.
 *
 * # Les trois arrêts de dégradé SONT couverts
 *
 * `from-…`, `via-…` et `to-…` nomment de vraies couleurs — `via-brillance` est le jeton du
 * scintillement du squelette, et il a bien une valeur propre sous `.dark`. Les omettre aurait
 * laissé un composant entier hors du contrôle : la porte aurait été verte sans regarder le seul
 * endroit où le mode sombre change de mécanisme.
 */
function jetonsDeCouleur(contenu: string): Set<string> {
  const trouves = new Set<string>()
  // Les valeurs arbitraires de Tailwind — `transition-[transform,border-color]`, `border-[1.5px]`
  // — nomment des propriétés et des longueurs CSS, jamais des jetons. Les retirer AVANT
  // l'extraction est plus sûr que de les filtrer après : la liste des propriétés CSS contenant
  // « -color » est ouverte, celle des crochets ne l'est pas.
  contenu = contenu.replace(/\[[^\]]*\]/g, '')

  const motif
    = /\b(?:bg|text|border|fill|stroke|from|via|to)(?:-(?:l|r|t|b|x|y|s|e))?-([a-z][a-z0-9-]*)\b/g

  /** Corps de texte, graisses, arrondis, directions de dégradé — tout ce qui n'est pas une couleur. */
  const NON_COULEURS
    = /^(etiquette|mini|corps|action|lead|titre|chiffre|montant|recette|total|annonce|affiche|left|right|center|semibold|bold|medium|normal|xs|sm|base|lg|xl|pleine|transparent|current|inherit|linear|radial|conic|none|solid|dashed|dotted|double|hidden)/

  for (const capture of contenu.matchAll(motif)) {
    const jeton = capture[1]
    if (!jeton) continue
    // Largeur de bordure. `border-l-4` capture « l-4 » et `border-b` capture « b » : le moteur
    // d'expressions régulières préfère la capture longue au groupe directionnel optionnel. Le
    // filtre porte donc sur la FORME du jeton — une direction seule, ou une direction suivie d'une
    // épaisseur — plutôt que sur une regex plus retorse qui se casserait au premier utilitaire
    // inattendu.
    if (/^[lrtbxyse](-\d+)?$/.test(jeton)) continue
    if (/^\d/.test(jeton)) continue
    if (NON_COULEURS.test(jeton)) continue
    trouves.add(jeton)
  }
  return trouves
}

describe('G1 — mode clair et mode sombre', () => {
  it('chaque jeton de couleur employé porte une valeur sous `.dark`', () => {
    const sombre = blocSombre()
    const manquants: string[] = []

    for (const { nom, contenu } of composantsG1()) {
      for (const jeton of jetonsDeCouleur(contenu)) {
        if (!sombre.includes(`--color-${jeton}:`)) {
          manquants.push(`${nom} — jeton « ${jeton} »`)
        }
      }
    }

    expect(
      manquants,
      'Ces jetons n’ont aucune valeur sous `.dark` : ils garderont leur valeur CLAIRE en mode '
      + 'sombre — texte foncé sur fond foncé, et l’écart ne se voit qu’en basculant le thème.\n  '
      + manquants.join('\n  '),
    ).toEqual([])
  })

  it('aucune palette dupliquée — la variante `dark:` ne porte aucune couleur', () => {
    const fautes: string[] = []

    for (const { nom, contenu } of composantsG1()) {
      // `dark:` suivi d'un utilitaire de COULEUR. La variante reste légitime pour une ombre, une
      // opacité ou une épaisseur de trait — ce qu'une couleur ne peut pas porter.
      for (const capture of contenu.matchAll(
        /\bdark:(?:bg|text|border|fill|stroke)-[a-z][a-z0-9-]*/g,
      )) {
        fautes.push(`${nom} — « ${capture[0]} »`)
      }
    }

    expect(
      fautes,
      'Le mode sombre passe par des jetons dont les VALEURS changent sous `.dark`, jamais par une '
      + 'seconde palette (principe XII). Une classe `dark:` de couleur crée une seconde source, '
      + 'et les deux divergent à la première évolution du thème.\n  '
      + fautes.join('\n  '),
    ).toEqual([])
  })

  it('les composants inspectés sont bien ceux attendus', () => {
    // Sans cette assertion, renommer un fichier ferait passer le test en n'inspectant plus rien —
    // le défaut exact que la constitution décrit sous « une porte qui ne trouve jamais rien ».
    const noms = composantsG1().map(c => c.nom).sort()

    expect(noms).toEqual([
      // ── Cycle 004 — la première verticale ──────────────────────────────────────────────
      'CarteFormule.vue', // le motif central de `G2`
      'ChampSaisie.vue',
      // ── Cycle 006 — le passage, ZONE DE VITESSE ────────────────────────────────────────
      //
      // `ChoixDuree` et `GrilleUnites` portent l'état d'une chambre en trois tons — libre,
      // occupée, à nettoyer. C'est exactement le genre de composant où une seconde palette se
      // glisserait : la tentation est d'écrire `dark:bg-vert-800` « juste pour ce cas ».
      'ChoixDuree.vue',
      // ── Cycle 003 — les quatre écrans ──────────────────────────────────────────────────
      'EcranAccueil.vue', // `R1`
      // ── Cycle 006 — l'arrivée, écran DÉRIVÉ de `R4` ────────────────────────────────────
      'EcranArrivee.vue', // `R3` — le parcours long, même grammaire
      'EcranChambres.vue', // `G5` — écran COMPOSÉ, sans maquette
      'EcranComptes.vue', // `G3`
      'EcranEtablissement.vue',
      'EcranJournalAudit.vue', // `G4`
      // ── Cycle 005 — l'écran composé de la note interne, premier passager de la file ────
      'EcranNotes.vue',
      'EcranOffre.vue', // `G2` — écran maquetté, deux états
      'EcranPassage.vue', // `R4` — écran MAQUETTÉ, cinq états, zone de vitesse
      'FormulaireCategorie.vue',
      'FormulaireUnite.vue',
      'GrilleUnites.vue', // `R4` — l'attribution d'un seul tap
      'ListeAccompagnants.vue', // `R3` — un nom suffit par accompagnant
      'ListeUnites.vue',
      'SectionIdentite.vue',
      'SectionIdentiteVisuelle.vue',
      // L'attribution des polices et des icônes — clause 2 de l'OFL, clause du MIT. Elle est dans
      // `G1` faute d'écran « à propos » : aucun des 41 écrans de `derivation.md` n'en prévoit un.
      'SectionMentions.vue',
      'SectionPointsDeVente.vue',
      'SectionServices.vue',
      // ── Cycle 005 — le composant 10, monté dans la COQUILLE donc présent sur toutes les
      //    pages. Il porte trois tons distincts (succès, alerte, danger) : c'est exactement le
      //    genre de composant où une seconde palette se glisserait.
      'TemoinSynchronisation.vue',
      // Le cadre à deux volets du styleguide. Il n'est pas un composant canonique, mais il pose des
      // jetons de couleur et il porte la classe `.dark` — donc il relève exactement du contrôle
      // ci-dessus. L'exclure aurait été le placer hors de vue sans raison.
      'VitrineTheme.vue',
      // ── Les pages : leur état d'erreur et de chargement porte des jetons ────────────────
      //
      // Ce sont des coquilles, et elles rendent quand même quelque chose : le fond et le texte de
      // « chargement… » et du message d'erreur. C'est l'écran qu'on voit quand quelque chose ne va
      // pas — celui où un texte illisible en mode sombre coûte le plus cher.
      'arrivee.vue', // `R3` — jamais « /check-in » : le mot est écarté du lexique
      'chambres.vue', // `G5`
      'comptes.vue', // `G3`
      'connexion.vue', // `R0`
      'etablissement.vue', // `G1`
      'hebergement.vue', // `G2`
      'index.vue', // `R1`
      'journal-audit.vue', // `G4`
      // ── Cycle 005 — les deux pages de la file hors-ligne ──────────────────────────────
      'mes-envois.vue', // `S1`, écran DÉRIVÉ du composant 10 — le mot « synchronisation » est
                        // proscrit du visible, URL comprise
      'notes.vue', // écran COMPOSÉ, cas (c)
      // ── Cycle 006 — la route est `/passage`, jamais `/check-in` ───────────────────────
      //
      // Le mot « check-in » est **écarté du lexique** v1.6.0, et une URL est visible dans la
      // barre d'adresse : il serait rentré par la porte du nom de fichier, sans qu'aucune porte
      // i18n ne le voie. C'est la leçon `S1` du cycle 005, appliquée avant de la réapprendre.
      'passage.vue', // `R4`, écran MAQUETTÉ, zone de vitesse
    ])
  })
})
