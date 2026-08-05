// @vitest-environment node
/**
 * **L'ACCUEIL MÈNE-T-IL QUELQUE PART ?** — la porte que le catalogue de tuiles n'avait pas.
 *
 * ═══════════════════════════════════════════════════════════════════════════════════════════
 *  CE QUI EST ARRIVÉ, ET POURQUOI UNE PORTE DE PLUS
 * ═══════════════════════════════════════════════════════════════════════════════════════════
 *
 * Les cycles SYN et SEJ ont livré six écrans — `/notes`, `/mes-envois`, `/passage`, `/arrivee`,
 * `/clients`, `/depart`. Les six s'ouvraient, affichaient de vraies données, passaient leurs
 * tests. **Aucun n'avait de tuile à l'accueil.** Onze routes sur treize n'étaient atteignables
 * qu'en tapant l'URL, et rien, nulle part, ne l'a signalé pendant deux cycles.
 *
 * C'est le motif du cycle 003 sous une autre forme — là, `G3` et `G4` ne se montaient pas ; ici,
 * ils se montaient parfaitement et personne ne pouvait y arriver. Dans les deux cas, **le parcours
 * n'était couvert par rien** : les portes vérifiaient les écrans un par un, jamais le chemin qui y
 * mène.
 *
 * ═══════════════════════════════════════════════════════════════════════════════════════════
 *  LE PÉRIMÈTRE EST DÉCOUVERT, JAMAIS ÉNUMÉRÉ — la règle du cycle 005
 * ═══════════════════════════════════════════════════════════════════════════════════════════
 *
 * Le catalogue est une liste écrite à la main ; c'est légitime, une tuile porte un libellé et une
 * icône que rien ne déduit. Ce qui ne peut pas être écrit à la main, c'est **la liste des routes
 * qu'il doit couvrir**. Elle vient donc du système de fichiers, comme `backend/tests/commun/
 * perimetre.rs` lit `pg_namespace` et les `[workspace] members`, et comme `tests-e2e/routes.ts`
 * lit déjà `app/pages/` pour P-22.
 *
 * Créer `app/pages/caisse.vue` au cycle 007 fera échouer ce fichier **en nommant la route**, sans
 * que personne ait eu à y penser. C'est la seule propriété qui tienne dans le temps.
 *
 * ═══════════════════════════════════════════════════════════════════════════════════════════
 *  POURQUOI ICI, ET PAS DANS P-22
 * ═══════════════════════════════════════════════════════════════════════════════════════════
 *
 * Deux raisons, et la seconde est la plus dure.
 *
 * 1. **P-22 navigue par le routeur.** Elle ouvre chaque route en direct et par navigation ; elle
 *    n'a aucun moyen de savoir qu'une route n'est proposée nulle part. Elle serait restée verte
 *    sur les six écrans orphelins, et elle l'est restée.
 * 2. **P-22 n'est pas dans `.github/workflows/ci.yml`** — elle exige l'API, la base et les seeds.
 *    Dans ce dépôt, une porte qui n'a besoin de rien s'exécute ; une porte qui a besoin de
 *    services ne s'exécute pas automatiquement. Ce fichier tourne dans le job `app`, par
 *    `pnpm --filter @kaya/app test`, sans rien allumer.
 *
 * Un cas P-22 qui **clique** une tuile depuis l'accueil complète celui-ci — il vérifie que le lien
 * mène vraiment quelque part, ce qu'un contrôle statique ne peut pas prouver. Il existe
 * (`tests-e2e/accueil-tuiles.spec.ts`) et il n'est pas le filet principal.
 *
 * ═══════════════════════════════════════════════════════════════════════════════════════════
 *  EXIGENCE 4 DE LA COUVERTURE DES PORTES — compter ce qu'on examine
 * ═══════════════════════════════════════════════════════════════════════════════════════════
 *
 * Un glob devenu faux — un répertoire renommé, une extension changée — rendrait zéro route, et
 * tous les contrôles ci-dessous passeraient au vert **en ne vérifiant rien**. Le décompte est donc
 * un contrôle à part entière, et il est le premier du fichier.
 */

import { readFileSync, readdirSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'

import { ACCES_ECRANS, peutOuvrirEcran } from '../core/acces/ecrans'
import { CATALOGUE_TUILES, ROUTES_SANS_TUILE } from '../core/accueil/tuiles'
import { referentielPermissions } from './commun/referentiel-permissions'
import fr from '../core/i18n/fr.json'
import en from '../core/i18n/en.json'

const ICI = dirname(fileURLToPath(import.meta.url))
const PAGES = resolve(ICI, '..', 'pages')
const CSS_ICONES = resolve(ICI, '..', 'assets', 'css', 'icones.css')

/**
 * Le chemin servi, dérivé du nom de fichier — **et un refus bruyant sur tout le reste**.
 *
 * Même règle que `tests-e2e/routes.ts` : une convention que ce lecteur ne sait pas traduire
 * (route dynamique, imbriquée, groupée) serait silencieusement absente du périmètre. Une porte qui
 * ne sait pas lire sa cible doit le dire, pas la sauter.
 */
function cheminDepuisFichier(fichier: string): string {
  const base = fichier.replace(/\.vue$/, '')

  if (/[[\]()]/.test(base)) {
    throw new Error(
      `Catalogue de l’accueil — la page « ${fichier} » emploie une convention de route que ce `
      + 'lecteur ne sait pas traduire (route dynamique, imbriquée ou groupée). Elle serait '
      + 'silencieusement hors du périmètre. Étendre `cheminDepuisFichier` dans le MÊME changement '
      + 'que la page — c’est le seul moment où quelqu’un y pense.',
    )
  }

  return base === 'index' ? '/' : `/${base}`
}

/** Toutes les routes du produit, **lues du système de fichiers**. */
const ROUTES: readonly string[] = readdirSync(PAGES, { withFileTypes: true })
  .filter(entree => entree.isFile() && entree.name.endsWith('.vue'))
  .map(entree => cheminDepuisFichier(entree.name))
  .sort()

/** Les routes que le catalogue propose. */
const ROUTES_AVEC_TUILE = new Set(CATALOGUE_TUILES.map(tuile => tuile.route))

/**
 * Le référentiel, lu des migrations — **avec le module qui porte chaque permission**.
 *
 * C'est ce second champ qui rend possible le contrôle de cohérence plus bas : une tuile qui exige
 * `heb.sejour.lire` doit déclarer `moduleRequis: 'HEBERGEMENT'`, sinon elle s'affiche dans un
 * maquis. `permissions.spec.ts` lit la même source pour vérifier l'existence des codes.
 */
const PERMISSIONS = referentielPermissions()

/** Traduit une clé pointée dans un catalogue i18n. Rend `undefined` si le chemin n'existe pas. */
function valeur(catalogue: unknown, cle: string): unknown {
  return cle.split('.').reduce<unknown>(
    (noeud, part) => (noeud as Record<string, unknown> | undefined)?.[part],
    catalogue,
  )
}

// =================================================================================================
//  1. La cible n'est pas vide — exigence 4
// =================================================================================================

describe('le périmètre examiné est réel', () => {
  it('découvre des routes, et pas zéro', () => {
    // Un glob devenu faux passerait tous les autres contrôles en n'en vérifiant aucun. Le seuil
    // n'est pas `> 0` mais un ordre de grandeur : le produit a treize écrans plus le styleguide,
    // et un lecteur qui n'en rendrait que deux serait cassé sans être vide.
    expect(ROUTES.length).toBeGreaterThanOrEqual(10)
  })

  it('lit des permissions dans les migrations, et pas zéro', () => {
    // Même raison : si l'extraction SQL cassait, « toute permission du catalogue est définie »
    // deviendrait « aucune permission n'est vérifiée », au vert.
    expect(PERMISSIONS.size).toBeGreaterThanOrEqual(25)
    expect(PERMISSIONS.get('heb.offre.lire')).toBe('HEBERGEMENT')
    expect(PERMISSIONS.get('sej.client.lire')).toBeNull()
  })

  it('le catalogue n’est pas vide', () => {
    expect(CATALOGUE_TUILES.length).toBeGreaterThanOrEqual(10)
  })
})

// =================================================================================================
//  2. LE CONTRÔLE CENTRAL — aucune route orpheline
// =================================================================================================

describe('toute route est atteignable depuis l’accueil, ou exemptée avec son motif', () => {
  it('aucun écran n’est joignable seulement en tapant son URL', () => {
    const orphelines = ROUTES.filter(
      route => !ROUTES_AVEC_TUILE.has(route) && !(route in ROUTES_SANS_TUILE),
    )

    expect(
      orphelines,
      'Ces routes existent dans `app/pages/` et l’accueil ne les propose pas.\n'
      + 'Deux issues, aucune troisième : leur donner une tuile dans `core/accueil/tuiles.ts` — '
      + 'avec la permission que l’écran exige VRAIMENT, celle que le serveur contrôle sur les '
      + 'routes qu’il appelle au montage —, ou les inscrire à `ROUTES_SANS_TUILE` AVEC LEUR '
      + 'MOTIF.\n'
      + 'C’est le défaut qui a laissé onze écrans sur treize hors de portée du doigt pendant deux '
      + 'cycles.',
    ).toEqual([])
  })

  it('aucune tuile ne pointe vers un écran qui n’existe pas', () => {
    const fantomes = CATALOGUE_TUILES
      .filter(tuile => !ROUTES.includes(tuile.route))
      .map(tuile => `${tuile.code} → ${tuile.route}`)

    // Le symétrique du contrôle précédent, et il attrape autre chose : une page supprimée ou
    // renommée laisserait une tuile qui mène à un 404 — visible, cliquable, morte.
    expect(fantomes, 'Ces tuiles ouvrent une route sans fichier dans `app/pages/`.').toEqual([])
  })

  it('aucune exemption morte', () => {
    const mortes = Object.keys(ROUTES_SANS_TUILE).filter(
      route => ROUTES_AVEC_TUILE.has(route) || !ROUTES.includes(route),
    )

    // Une exemption qui survit à sa raison — la route a gagné une tuile, ou a disparu — rend le
    // tableau moins lisible à chaque cycle, et finit par exempter ce que personne n'a relu.
    expect(
      mortes,
      'Ces exemptions ne portent plus sur rien : la route a une tuile, ou elle n’existe plus.',
    ).toEqual([])
  })

  it('chaque exemption porte un motif, pas une ligne muette', () => {
    for (const [route, motif] of Object.entries(ROUTES_SANS_TUILE)) {
      expect(motif.length, `L’exemption de « ${route} » n’explique rien.`).toBeGreaterThan(40)
    }
  })
})

// =================================================================================================
//  3. Une tuile qui s'affiche doit s'ouvrir — permissions, module, libellés, icône
// =================================================================================================

describe('toute route déclare CE QUI L’OUVRE — `core/acces/ecrans.ts`', () => {
  it('aucune route n’est absente de la table d’accès', () => {
    const absentes = ROUTES.filter(route => !(route in ACCES_ECRANS))

    // ⚠️ C'est ce contrôle qui rend la table opposable. `peutOuvrirEcran` REFUSE une route
    // inconnue — défaut sûr — mais un refus silencieux en développement se contourne en
    // retirant la garde ; ici, la CI nomme la route et le geste est d'y répondre.
    expect(
      absentes,
      'Ces routes existent dans `app/pages/` et rien ne dit ce qui les ouvre.\n'
      + 'Déclarer leurs permissions de LECTURE — celles que le serveur exige sur les routes que '
      + 'l’écran appelle au montage — ou une liste vide AVEC son motif.',
    ).toEqual([])
  })

  it('aucune entrée ne porte sur une route disparue', () => {
    const fantomes = Object.keys(ACCES_ECRANS).filter(route => !ROUTES.includes(route))

    expect(fantomes, 'Ces entrées gardent des écrans qui n’existent plus.').toEqual([])
  })

  it('toute permission exigée existe en base', () => {
    const inconnues = Object.entries(ACCES_ECRANS).flatMap(([route, acces]) =>
      acces.permissions
        .filter(permission => !PERMISSIONS.has(permission))
        .map(permission => `${route} → ${permission}`),
    )

    // Une permission mal orthographiée ne casse rien de visible : `detient` rend `false`, la tuile
    // disparaît pour tout le monde ET l'écran refuse l'accès direct. Personne ne le remarque avant
    // qu'un exploitant demande pourquoi il ne peut plus entrer quelque part.
    expect(inconnues, 'Ces permissions ne sont définies par aucune migration.').toEqual([])
  })

  it('un écran SANS permission dit pourquoi', () => {
    for (const [route, acces] of Object.entries(ACCES_ECRANS)) {
      if (acces.permissions.length > 0) continue
      expect(
        acces.motif ?? '',
        `« ${route} » n’est gardée par aucune permission et n’explique pas pourquoi. `
        + 'Un écran que rien ne garde est une décision ; sans motif écrit, le prochain se posera '
        + 'par imitation.',
      ).toSatisfy((motif: string) => motif.length > 80)
    }
  })

  it('un motif ne traîne pas sur un écran qui a des permissions', () => {
    const bavards = Object.entries(ACCES_ECRANS)
      .filter(([, acces]) => acces.permissions.length > 0 && acces.motif)
      .map(([route]) => route)

    expect(bavards, 'Ces écrans portent un motif d’absence de permission… et des permissions.')
      .toEqual([])
  })

  it('⚠️ une route inconnue est REFUSÉE, jamais laissée ouverte', () => {
    // Le défaut sûr, vérifié plutôt que supposé : un écran ajouté sans passer par la table doit
    // se voir tout de suite, par un refus, et non atteindre la production sans garde.
    expect(peutOuvrirEcran('/caisse', ['etb.etablissement.lire'])).toBe(false)
    expect(peutOuvrirEcran('/', [])).toBe(true)
  })
})

describe('une tuile proposée est une tuile qui s’ouvre', () => {
  it('⚠️ la tuile et la page posent la MÊME question — jamais deux déclarations', () => {
    // La divergence a deux formes, toutes deux mauvaises : une tuile qui ouvre sur un refus, ou
    // un écran caché mais atteignable. Une seule table les rend impossibles ; ce contrôle vérifie
    // que le catalogue n'a pas recommencé à déclarer les siennes.
    for (const tuile of CATALOGUE_TUILES) {
      expect(
        Object.keys(tuile),
        `« ${tuile.code} » redéclare ses permissions au lieu de les lire de ACCES_ECRANS.`,
      ).not.toContain('permissionsRequises')
    }

    // Et le versant positif : chaque tuile est bien gardée par la table.
    const nonGardees = CATALOGUE_TUILES.filter(tuile => !(tuile.route in ACCES_ECRANS))
    expect(nonGardees.map(t => t.code)).toEqual([])
  })

  it('une tuile qui exige une permission de module DÉCLARE ce module', () => {
    const incoherentes = CATALOGUE_TUILES.flatMap(tuile =>
      (ACCES_ECRANS[tuile.route]?.permissions ?? [])
        .map(permission => ({ permission, module: PERMISSIONS.get(permission) ?? null }))
        .filter(({ module }) => module !== null && module !== tuile.moduleRequis)
        .map(({ permission, module }) =>
          `${tuile.code} exige ${permission} (module ${module}) sans moduleRequis: '${module}'`),
    )

    // Les permissions sont attribuées par RÔLE, indépendamment des services activés par
    // l'établissement. Une réceptionniste garde `heb.sejour.lire` dans un maquis qui ne loue rien :
    // sans `moduleRequis`, la tuile « Un départ » s'y afficherait et ouvrirait sur un écran vide.
    expect(incoherentes).toEqual([])
  })

  it('les codes et les routes sont uniques', () => {
    expect(new Set(CATALOGUE_TUILES.map(t => t.code)).size).toBe(CATALOGUE_TUILES.length)
    expect(new Set(CATALOGUE_TUILES.map(t => t.route)).size).toBe(CATALOGUE_TUILES.length)
  })

  it('chaque libellé et chaque description existent en fr ET en en', () => {
    for (const tuile of CATALOGUE_TUILES) {
      for (const cle of [tuile.libelleCle, tuile.descriptionCle]) {
        for (const [langue, catalogue] of [['fr', fr], ['en', en]] as const) {
          const traduction = valeur(catalogue, cle)
          // Une clé manquante rend la clé elle-même à l'écran — `accueil.tuiles.passage.libelle`
          // sur une tuile, au comptoir. P-16 vérifie la PARITÉ des deux catalogues ; il ne
          // vérifie pas qu'une clé citée par le code existe dans l'un d'eux.
          expect(typeof traduction, `${cle} manque en ${langue}`).toBe('string')
          expect((traduction as string).trim().length, `${cle} est vide en ${langue}`)
            .toBeGreaterThan(0)
        }
      }
    }
  })

  it('chaque icône est RÉELLEMENT embarquée dans la police sous-réglée', () => {
    const css = readFileSync(CSS_ICONES, 'utf8')
    const absentes = CATALOGUE_TUILES
      .filter(tuile => !new RegExp(`\\.${tuile.icone}:before`).test(css))
      .map(tuile => `${tuile.code} → ${tuile.icone}`)

    // ⚠️ CE CONTRÔLE A TROUVÉ UN DÉFAUT VIEUX DE TROIS CYCLES. Le catalogue nomme ses glyphes en
    // donnée (`icone: 'ph-...'`), rendus par `:class="tuile.icone"` ; le générateur ne relevait
    // que les attributs de classe littéraux et ne les voyait pas. `ph-list-magnifying-glass`
    // manquait au woff2 depuis le cycle 003 : la tuile « Registre des actions » s'affichait sans
    // icône chez le propriétaire. P-21b ne pouvait pas le dire — elle vérifie que ce qui est
    // DÉCLARÉ est embarqué, jamais que ce qui est RENDU est déclaré.
    expect(
      absentes,
      'Ces glyphes ne sont pas dans `assets/css/icones.css` : la tuile rendra une case vide.\n'
      + 'Régénérer : `pnpm --filter @kaya/app icones:generer`.',
    ).toEqual([])
  })
})
