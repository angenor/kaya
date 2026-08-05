/**
 * **Le référentiel des permissions, lu des MIGRATIONS** — source unique des tests du front.
 *
 * # Pourquoi les migrations, et pas la base ni le contrat
 *
 * Une permission écrite côté front est une **chaîne**. Une faute de frappe, un renommage côté
 * serveur, un code inventé de bonne foi : dans les trois cas, `detient()` rend `false`, et
 * l'action **disparaît silencieusement de l'interface**. Ni P-01 (les permissions ne sont pas dans
 * le contrat OpenAPI, ce sont des données) ni le compilateur (des chaînes des deux côtés) ne
 * peuvent le voir.
 *
 * Les `INSERT INTO comptes.permission` sont la source qui crée la ligne, et une migration
 * appliquée ne se modifie plus (P-02). Les lire ne demande ni base allumée ni réseau : ces tests
 * tournent dans le job `app` de la CI, qui n'allume rien.
 *
 * # Ce fichier est né d'une DEUXIÈME lecture des mêmes migrations
 *
 * `permissions.spec.ts` en portait une, et `catalogue-accueil.spec.ts` en a eu besoin d'une autre
 * — celle-ci ayant en plus besoin du `module_code`. Deux motifs d'extraction sur la même source
 * divergent : celui de `permissions.spec.ts` énumérait les préfixes reconnus (`etb|cpt|heb|…`), et
 * le cycle 006 a constaté que l'arrivée de `sej` y aurait été **silencieuse** si le décompte ne
 * l'avait pas rattrapée.
 *
 * D'où une lecture unique, et **plus large** : elle borne d'abord au bloc
 * `INSERT INTO comptes.permission (…) VALUES … ;` — ce qui la rend insensible au reste du fichier
 * — puis accepte n'importe quel code de la nomenclature. Un nouveau préfixe entre sans qu'on ait à
 * revenir ici ; c'est ce qui distingue une porte d'une liste.
 */

import { readFileSync, readdirSync } from 'node:fs'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const ICI = dirname(fileURLToPath(import.meta.url))

/**
 * **Le répertoire des migrations, pas un fichier.**
 *
 * Le test d'origine lisait `0016_roles_permissions.sql` seul. Il suffisait tant que ce fichier
 * portait toutes les permissions du produit ; le cycle 004 en a ajouté cinq dans `0022` et le
 * cycle 006 sept dans `0030`.
 */
const MIGRATIONS = resolve(ICI, '..', '..', '..', 'backend', 'migrations')

/**
 * Chaque permission définie, associée au **module d'activité qui la porte** — `null` si elle est
 * transverse.
 *
 * ⚠️ `sej.client.lire` et `sej.client.gerer` sont à `null`, et ce n'est pas un oubli : la fiche
 * client ne dépend d'aucun module d'activité. Un maquis en a besoin sans louer une chambre.
 */
export function referentielPermissions(): Map<string, string | null> {
  const table = new Map<string, string | null>()

  for (const fichier of readdirSync(MIGRATIONS).filter(nom => nom.endsWith('.sql')).sort()) {
    const sql = readFileSync(join(MIGRATIONS, fichier), 'utf8')
    for (const bloc of sql.matchAll(
      /INSERT\s+INTO\s+comptes\.permission\s*\([^)]*\)\s*VALUES([\s\S]*?);/gi,
    )) {
      for (const ligne of bloc[1]!.matchAll(
        /\(\s*'([a-z][a-z0-9_]*(?:\.[a-z0-9_]+)+)'\s*,\s*(NULL|'([A-Z_]+)')/gi,
      )) {
        table.set(ligne[1]!, ligne[3] ?? null)
      }
    }
  }

  return table
}

/** Les seuls codes, pour les appelants qui n'ont pas besoin du module. */
export function codesPermissions(): Set<string> {
  return new Set(referentielPermissions().keys())
}
