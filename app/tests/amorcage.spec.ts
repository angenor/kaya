/**
 * **Harnais d'amorçage — « une unité écrite n'est pas une unité branchée ».**
 *
 * # Ce que ce fichier existe pour empêcher
 *
 * `initialiserTheme()` a vécu deux cycles dans `core/theme/index.ts` : exportée, documentée
 * « à appeler au démarrage de l'application », et **appelée nulle part**. Le produit n'a jamais
 * appliqué la classe `.dark`. Personne ne l'a vu, parce que rien ne regardait cette question — et
 * la revue du cycle 003 avait coché le mode sombre.
 *
 * Aggravant, et découvert en vérifiant : elle n'était même pas **testée**. `theme-sombre.spec.ts`
 * n'importe pas `core/theme` ; il lit les jetons de `theme.css`. Les 67 lignes du module étaient
 * sans appelant **et** sans test. La règle est donc plus dure que « une unité testée n'est pas une
 * unité branchée » : **une unité écrite n'est ni testée ni branchée par défaut**, et il faut un
 * contrôle pour chacune des deux propriétés.
 *
 * Ce harnais couvre la seconde. Il est calqué sur `backend/tests/audit_taxonomie.rs`, dont le
 * mécanisme a fait ses preuves : une liste à deux états, `branché` et `dû`, où **les deux versants
 * sont vérifiés**. Un état `dû` qui acquiert un appelant fait échouer le build ; un état `branché`
 * qui le perd aussi. Sans le second versant, il suffirait de tout déclarer branché pour rendre le
 * harnais muet — c'est la dissymétrie que P-05b a corrigée côté backend.
 *
 * # Périmètre inspecté — exigence 1 du § « Couverture des portes »
 *
 * **Inspecté.** Tous les fichiers `.ts` et `.vue` sous `app/plugins/`, `app/middleware/`,
 * `app/layouts/`, `app/pages/`, `app/modules/`, `app/core/`, plus `app/app.vue` et
 * `app/nuxt.config.ts`.
 *
 * **Exclu, nommément.**
 *
 *   · `app/tests/` — un test qui appelle une fonction d'amorçage ne la branche pas. Les inclure
 *     ferait passer les cinq fonctions de la file hors-ligne pour branchées, alors qu'elles ne
 *     sont exercées que par leurs propres tests. C'est exactement l'erreur à ne pas faire.
 *   · Le **module de définition** de chaque fonction. Y voir son propre nom n'apprend rien.
 *
 * **Ce que le harnais ne sait pas voir**, écrit pour qu'on ne le croie pas plus fort qu'il n'est :
 * un appel construit dynamiquement — `obj[nomVariable]()`, une réexportation en étoile suivie d'un
 * accès indirect. La liste est donc **fermée et courte**, et sa colonne « où » est vérifiée : c'est
 * le fichier attendu qui doit contenir l'appel, pas n'importe lequel.
 */

import { readdirSync, readFileSync, statSync } from 'node:fs'
import { join, resolve } from 'node:path'

import { describe, expect, it } from 'vitest'

const RACINE = resolve(import.meta.dirname, '..')

/** Les arbres du **parcours réel**. `tests/` en est absent, et c'est le point. */
const ARBRES = ['plugins', 'middleware', 'layouts', 'pages', 'modules', 'core'] as const

/** Fichiers isolés du parcours réel. */
const FICHIERS_ISOLES = ['app.vue', 'nuxt.config.ts'] as const

type Etat = 'branché' | 'dû'

interface Amorcage {
  /** Le symbole à chercher. */
  readonly nom: string
  /** Son module de définition — **exclu du balayage**. */
  readonly definition: string
  /** Ce qu'elle amorce, en une phrase. */
  readonly role: string
  readonly etat: Etat
  /**
   * Pour un `branché` : le fichier qui doit l'appeler. Un appel ailleurs ne compte pas — c'est ce
   * qui distingue « quelqu'un l'appelle » de « le point d'amorçage l'appelle ».
   * Pour un `dû` : la story qui la branchera.
   */
  readonly ou: string
}

/**
 * **Les points d'entrée d'amorçage du produit.** Liste fermée, onze entrées.
 *
 * Établie par balayage exhaustif de `app/core/` — toute fonction exportée qu'un chemin d'amorçage
 * doit appeler pour que le produit fonctionne. Les fonctions appelées **par** l'une d'elles dans
 * leur propre module n'y figurent pas : elles sont branchées par transitivité, et la note sur
 * `appliquerMode` ci-dessous dit pourquoi la distinction compte.
 */
const AMORCAGE: readonly Amorcage[] = [
  // ── Branchées par le lot d'amorçage ───────────────────────────────────────────────────────
  {
    nom: 'initialiserTheme',
    definition: 'core/theme/index.ts',
    role: 'applique le mode clair/sombre retenu — sans elle, `.dark` n’est jamais posée',
    etat: 'branché',
    ou: 'plugins/01.theme.client.ts',
  },
  {
    nom: 'reprendreSession',
    definition: 'core/auth/connexion.ts',
    role: 'reprend la session depuis le jeton rangé, avant chaque navigation',
    etat: 'branché',
    ou: 'middleware/01.session.global.ts',
  },
  // `appliquerMode` n'EST PAS dans cette liste, et c'est une décision, pas un oubli. Elle est
  // appelée par `initialiserTheme`, **dans son propre module** — donc branchée par transitivité,
  // et ce harnais n'a rien à ajouter. Ce qu'il inventorie sont les **points d'entrée** : les
  // fonctions qu'un chemin d'amorçage doit appeler pour que le produit fonctionne. Le jour où une
  // bascule de thème l'appellera depuis un composant, elle deviendra un point d'entrée et prendra
  // sa ligne ici.
  //
  // La distinction s'est imposée d'elle-même : la première version de cette liste la déclarait
  // « branchée dans core/theme/index.ts », et le harnais a refusé — le module de définition est
  // exclu du balayage. Le contrôle a corrigé sa propre déclaration avant de corriger du code.
  {
    nom: 'poserSession',
    definition: 'core/auth/session.ts',
    role: 'pose l’état de session en mémoire après une connexion réussie',
    etat: 'branché',
    ou: 'core/auth/connexion.ts',
  },
  {
    nom: 'adaptateurCourant',
    definition: 'core/platform/courant.ts',
    role: 'résout l’implémentation de `PlatformAdapter` de la plateforme courante',
    etat: 'branché',
    ou: 'middleware/01.session.global.ts',
  },
  {
    nom: 'useEtatReseau',
    definition: 'core/platform/reseau.ts',
    role: 'expose l’état réseau réactif aux écrans qui refusent une action hors ligne',
    etat: 'branché',
    ou: 'pages/connexion.vue',
  },
  {
    nom: 'styleguideMonte',
    definition: 'core/design-system/montage.ts',
    role: 'décide si la route du styleguide est montée — hors production, jamais',
    etat: 'branché',
    ou: 'nuxt.config.ts',
  },

  // ── Dues : le mécanisme existe, aucun chemin du produit ne l'atteint ──────────────────────
  //
  // Ce ne sont PAS des défauts à corriger ici. Ce sont des mécanismes en attente de l'écran qui
  // les emploiera, et les déclarer « dus » est ce qui garantit qu'on ne les branchera pas en
  // silence : le jour où un appelant apparaît, ce fichier échoue et la ligne doit passer à
  // « branché » dans le MÊME changement.
  {
    nom: 'FileLocale',
    definition: 'core/sync/index.ts',
    role: 'la file hors-ligne — jamais instanciée : `TYPES_CLASSE_A` ne porte que '
      + '`note_etablissement.creee`, dont l’écran n’existe pas',
    etat: 'dû',
    ou: 'le premier écran qui écrit une opération de classe A',
  },
  {
    nom: 'marquerClasseA',
    definition: 'core/sync/classes.ts',
    role: 'le seul point d’entrée de la file — rien n’y passe encore',
    etat: 'dû',
    ou: 'le premier écran qui écrit une opération de classe A',
  },
  {
    nom: 'viderFile',
    definition: 'core/sync/vidage.ts',
    role: 'vide la file au retour du réseau, **rafraîchir avant d’envoyer** — aucun crochet de '
      + 'retour au premier plan ne l’appelle',
    etat: 'dû',
    ou: 'SYN-01 · le témoin de synchronisation et le crochet de reprise',
  },
  {
    nom: 'operationRealisable',
    definition: 'core/sync/index.ts',
    role: 'la garde hors-ligne canonique — six emplacements réimplémentent `reseau !== '
      + '\'connecte\'` à la main, ce qui contourne le registre `TYPES_CLASSE_A`',
    etat: 'dû',
    ou: 'SYN-01 · à substituer aux six gardes écrites à la main',
  },
  {
    nom: 'fermerSession',
    definition: 'core/auth/connexion.ts',
    role: 'efface la session et purge le stockage (principe VI, terminal partagé) — **aucun '
      + 'bouton de déconnexion n’existe dans le produit**, ni aucune clé i18n',
    etat: 'dû',
    ou: 'ETB-06 · le sélecteur de contexte permanent, qui porte la déconnexion',
  },
]

// =================================================================================================
//  Le balayage
// =================================================================================================

function fichiersDuParcours(): string[] {
  const trouves: string[] = []

  const descendre = (repertoire: string): void => {
    let entrees: string[]
    try {
      entrees = readdirSync(repertoire)
    }
    catch {
      return
    }
    for (const entree of entrees) {
      const chemin = join(repertoire, entree)
      if (statSync(chemin).isDirectory()) {
        descendre(chemin)
      }
      else if (chemin.endsWith('.ts') || chemin.endsWith('.vue')) {
        trouves.push(chemin)
      }
    }
  }

  for (const arbre of ARBRES) descendre(join(RACINE, arbre))
  for (const fichier of FICHIERS_ISOLES) trouves.push(join(RACINE, fichier))

  return trouves.map(c => c.slice(RACINE.length + 1))
}

/**
 * Les fichiers qui **appellent** un symbole, hors de son module de définition.
 *
 * Un appel se reconnaît à `nom(` ou `new nom(` — une importation seule ne compte pas. C'est la
 * distinction qui fait tout le sujet : `core/sync/index.ts` réexporte `viderFile`, ce qui ne la
 * branche pas.
 */
function appelants(amorcage: Amorcage, fichiers: readonly string[]): string[] {
  const motifs = [
    new RegExp(`(?<![\\w.])${amorcage.nom}\\s*\\(`),
    new RegExp(`new\\s+${amorcage.nom}\\s*\\(`),
  ]

  return fichiers
    .filter(f => f !== amorcage.definition)
    .filter((f) => {
      const contenu = readFileSync(join(RACINE, f), 'utf8')
      // Les blocs de commentaire sont retirés : ce projet documente abondamment, et un
      // `{@link viderFile}` ferait passer une fonction due pour branchée.
      const code = contenu.replace(/\/\*[\s\S]*?\*\//g, '').replace(/^\s*\/\/.*$/gm, '')
      return motifs.some(motif => motif.test(code))
    })
}

// =================================================================================================
//  Les quatre contrôles
// =================================================================================================

describe('amorçage de l’application', () => {
  const fichiers = fichiersDuParcours()

  it('le balayage a une cible non vide', () => {
    // Exigence 4. Une porte qui n'inspecte rien passe toujours : ce contrôle est ce qui distingue
    // « aucun défaut » de « rien n'a été lu ».
    expect(
      fichiers.length,
      'aucun fichier du parcours réel balayé — le harnais ne vérifierait rien.',
    ).toBeGreaterThan(20)

    for (const arbre of ARBRES) {
      expect(
        fichiers.filter(f => f.startsWith(`${arbre}/`)).length,
        `aucun fichier balayé sous app/${arbre}/ — l’arbre a-t-il été déplacé ?\n`
        + 'Un arbre vide retire silencieusement sa part de la couverture.',
      ).toBeGreaterThan(0)
    }

    expect(AMORCAGE.length, 'la liste d’amorçage est vide').toBeGreaterThan(0)
    expect(
      new Set(AMORCAGE.map(a => a.nom)).size,
      'deux entrées portent le même nom dans AMORCAGE',
    ).toBe(AMORCAGE.length)
  })

  it('toute fonction déclarée « branchée » est RÉELLEMENT appelée, et à l’endroit annoncé', () => {
    const muettes = AMORCAGE
      .filter(a => a.etat === 'branché')
      .map(a => ({ a, trouves: appelants(a, fichiers) }))
      .filter(({ a, trouves }) => !trouves.includes(a.ou))

    expect(
      muettes.map(({ a, trouves }) =>
        `  · « ${a.nom} » déclarée branchée dans « ${a.ou} », appelée dans : `
        + `${trouves.length ? trouves.join(', ') : 'AUCUN fichier du parcours réel'}`),
      'des fonctions déclarées branchées ne le sont pas à l’endroit annoncé.\n\n'
      + 'C’est le défaut d’`initialiserTheme` : exportée, documentée « à appeler au démarrage », '
      + 'et appelée nulle part pendant deux cycles.\n'
      + 'Soit l’appel a été retiré — remettre la ligne à « dû » avec la story qui le rétablira —, '
      + 'soit il a déménagé, et c’est la colonne « où » qu’il faut corriger.',
    ).toEqual([])
  })

  it('aucune fonction déclarée « due » n’a d’appelant', () => {
    // Le versant qui empêche de rendre le harnais muet. Sans lui, tout déclarer « branché »
    // suffirait — et déclarer « dû » ce qu'on vient de brancher passerait inaperçu.
    const fautives = AMORCAGE
      .filter(a => a.etat === 'dû')
      .map(a => ({ a, trouves: appelants(a, fichiers) }))
      .filter(({ trouves }) => trouves.length > 0)

    expect(
      fautives.map(({ a, trouves }) =>
        `  · « ${a.nom} » (due par ${a.ou}) est appelée dans : ${trouves.join(', ')}`),
      'des fonctions déclarées dues ont un appelant.\n\n'
      + 'Si le branchement est voulu, faire passer la ligne à « branché » dans CE changement, avec '
      + 'le fichier qui l’appelle — c’est tout ce que le harnais demande.\n'
      + 'S’il n’est pas voulu, un mécanisme d’amorçage vient d’entrer en service sans que personne '
      + 'ne l’ait décidé.',
    ).toEqual([])
  })

  it('décompte — l’état d’avancement, affiché plutôt que deviné', () => {
    const branchees = AMORCAGE.filter(a => a.etat === 'branché')
    const dues = AMORCAGE.filter(a => a.etat === 'dû')

    expect(branchees.length + dues.length).toBe(AMORCAGE.length)

    const lignes = [
      `amorçage : ${branchees.length} branchée(s) / ${dues.length} due(s) sur ${AMORCAGE.length},`
      + ` ${fichiers.length} fichier(s) du parcours réel inspecté(s)`,
      ...AMORCAGE.map(a =>
        `  [${a.etat === 'branché' ? 'branché' : 'dû     '}] ${a.nom.padEnd(21)} ${a.ou}`),
    ]
    console.warn(lignes.join('\n'))
  })
})

// =================================================================================================
//  Test négatif — le harnais sait échouer
// =================================================================================================

describe('le harnais sait échouer', () => {
  it('un « branché » sans appelant et un « dû » avec appelant sont tous deux signalés', () => {
    // Exercé sur un jeu simulé : injecter une entrée factice dans AMORCAGE ferait échouer les
    // contrôles ci-dessus au hasard de l'ordonnancement.
    const simule: readonly Amorcage[] = [
      { nom: 'jamaisAppelee', definition: 'core/x.ts', role: '', etat: 'branché', ou: 'plugins/y.ts' },
      { nom: 'dejaAppelee', definition: 'core/z.ts', role: '', etat: 'dû', ou: 'SYN-01' },
    ]
    const appels: Record<string, string[]> = {
      jamaisAppelee: [],
      dejaAppelee: ['pages/index.vue'],
    }

    const branchesMuets = simule
      .filter(a => a.etat === 'branché' && !appels[a.nom]!.includes(a.ou))
    expect(branchesMuets, 'le versant positif ne signale rien : le harnais ne protège pas')
      .toHaveLength(1)

    const dusFautifs = simule.filter(a => a.etat === 'dû' && appels[a.nom]!.length > 0)
    expect(dusFautifs, 'le versant négatif ne signale rien : tout déclarer branché suffirait')
      .toHaveLength(1)
  })

  it('une importation seule ne compte PAS comme un appel', () => {
    // La distinction qui fait tout le sujet : `core/sync/index.ts` réexporte `viderFile`, ce qui
    // ne la branche pas. Si ce contrôle cassait, les cinq fonctions dues passeraient pour
    // branchées et le harnais deviendrait muet.
    const motif = /(?<![\w.])viderFile\s*\(/
    expect(motif.test("export { viderFile } from './vidage'")).toBe(false)
    expect(motif.test("import { viderFile } from '~/core/sync'")).toBe(false)
    expect(motif.test('await viderFile(file, envoyeur)')).toBe(true)
    // Et un appel de méthode homonyme ne compte pas non plus.
    expect(motif.test('file.viderFile()')).toBe(false)
  })
})
