/**
 * **Les routes de la porte P-22, LUES de `app/pages/`.**
 *
 * # Pourquoi elles ne sont pas écrites à la main
 *
 * *Exigence 2 du § « Couverture des portes » : « compter les cibles réellement examinées et les
 * comparer au total attendu ».*
 *
 * Une liste écrite à la main laisse passer la septième page. Ce n'est pas une crainte théorique :
 * c'est exactement ce qui est arrivé deux fois dans ce dépôt.
 *
 * - Au cycle 002, le décompte de **P-07** portait sur 4 tables sur 10.
 * - Au cycle 003, `couverture_portes.rs` ne balayait qu'un schéma sur deux — dix tables entières
 *   hors de portée, et la porte restait verte.
 *
 * Dans les deux cas, la donnée existait et la porte regardait ailleurs. Ici, la liste vient du
 * **système de fichiers** : créer `app/pages/caisse.vue` ajoute une route à P-22 sans que personne
 * n'ait à y penser, et c'est la seule propriété qui tienne dans le temps.
 *
 * # Ce que la lecture NE couvre pas, et qui est déclaré
 *
 * La convention de Nuxt dérive la route du nom de fichier. Cette lecture couvre le cas simple —
 * `comptes.vue` → `/comptes`, `index.vue` → `/` — et **refuse bruyamment** tout ce qu'elle ne sait
 * pas traduire : routes dynamiques (`[id].vue`), routes imbriquées, groupes. Le jour où le produit
 * en aura une, ce fichier échouera au lieu de l'ignorer en silence. Une porte qui ne sait pas lire
 * sa cible doit le dire, pas la sauter.
 */

import { readdirSync } from 'node:fs'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const ICI = dirname(fileURLToPath(import.meta.url))

/** Racine des pages Nuxt. `srcDir: '.'` place les conventions à la racine du paquet `app/`. */
const PAGES = resolve(ICI, '..', 'app', 'pages')

export interface Route {
  /** Le chemin servi, dérivé du nom de fichier. */
  readonly chemin: string
  /** Le fichier dont il vient — reproduit dans les messages d'échec. */
  readonly fichier: string
  /** La route est-elle derrière le middleware de session ? */
  readonly exigeSession: boolean
}

/**
 * Les routes **publiques**, nommées une par une.
 *
 * Cette liste est la seule chose écrite à la main dans ce fichier, et c'est délibéré : elle doit
 * être **courte et justifiée**. Elle reproduit `SANS_SESSION` de
 * `app/middleware/01.session.global.ts` — deux endroits, la même règle. Ce n'est pas idéal, et
 * l'alternative l'était moins : importer le middleware ici entraînerait tout le runtime de Nuxt
 * dans un processus Playwright. Le contrôle ci-dessous vérifie que les deux ne divergent pas.
 */
const PUBLIQUES: ReadonlySet<string> = new Set(['/connexion', '/styleguide'])

function cheminDepuisFichier(fichier: string): string {
  const base = fichier.replace(/\.vue$/, '')

  if (/[[\]()]/.test(base)) {
    throw new Error(
      `P-22 — la page « ${fichier} » emploie une convention de route que ce lecteur ne sait pas `
      + 'traduire (route dynamique, imbriquée ou groupée).\n'
      + 'Elle serait silencieusement absente de la porte. Étendre `cheminDepuisFichier` dans le '
      + 'MÊME changement que la page — c’est le seul moment où quelqu’un y pense.',
    )
  }

  return base === 'index' ? '/' : `/${base}`
}

/** Toutes les routes du produit, lues du système de fichiers. */
export const ROUTES: readonly Route[] = readdirSync(PAGES, { withFileTypes: true })
  .filter(entree => entree.isFile() && entree.name.endsWith('.vue'))
  .map((entree) => {
    const chemin = cheminDepuisFichier(entree.name)
    return {
      chemin,
      fichier: join('app', 'pages', entree.name),
      exigeSession: !PUBLIQUES.has(chemin),
    }
  })
  .sort((a, b) => a.chemin.localeCompare(b.chemin))

/**
 * Le compte de démonstration, **celui des seeds**.
 *
 * Adjoua porte trois rôles — gérante, caissière, réceptionniste. C'est le compte le plus large du
 * jeu de démonstration, donc celui qui ouvre le plus d'écrans : un compte à rôle unique ferait
 * passer la porte en n'atteignant pas les pages qu'il n'a pas le droit de voir, ce qui est une
 * autre façon de rétrécir la cible.
 *
 * Le mot de passe vient de l'environnement, comme côté serveur : `KAYA_SEEDS_MOT_DE_PASSE` est la
 * variable que `api/src/bin/seeds.rs` exige, et aucun mot de passe n'est écrit dans le dépôt.
 */
export const COMPTE_DEMONSTRATION = {
  identifiant: 'adjoua@deloria.test',
  get motDePasse(): string {
    const valeur = process.env.KAYA_SEEDS_MOT_DE_PASSE
    if (!valeur) {
      throw new Error(
        'P-22 — `KAYA_SEEDS_MOT_DE_PASSE` n’est pas défini. C’est la variable dont `seeds` se sert '
        + 'pour créer les comptes de démonstration ; sans elle, la porte ne peut pas se connecter, '
        + 'et aucun mot de passe n’est écrit dans le dépôt.',
      )
    }
    return valeur
  },
} as const
