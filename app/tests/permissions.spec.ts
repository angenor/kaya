/**
 * **Les permissions nommées par le front existent-elles au référentiel du serveur ?**
 *
 * # Le défaut que ce fichier attrape, et que rien d'autre n'attraperait
 *
 * Une permission écrite côté front — `'cpt.role.attribuer'` dans `roles.ts`, `'etb.service.
 * basculer'` dans `bascule-service.ts` — est une **chaîne**. Une faute de frappe, un renommage
 * côté serveur, un code inventé de bonne foi : dans les trois cas, `detient()` rend `false`, et
 * l'action **disparaît silencieusement de l'interface**.
 *
 * C'est le pire des symptômes possibles. Il ne produit ni erreur, ni page blanche, ni ligne de
 * journal : l'écran se rend parfaitement, et le bouton n'y est simplement pas. Personne ne le
 * remarque avant qu'un exploitant demande pourquoi il ne peut plus faire quelque chose.
 *
 * La porte P-01 ne le voit pas — les permissions ne sont pas dans le contrat OpenAPI, ce sont des
 * **données** de la table `comptes.permission`. Le compilateur ne le voit pas non plus : ce sont
 * des chaînes des deux côtés. D'où cette comparaison, contre la **migration**, qui est la source.
 *
 * # Périmètre inspecté, et ce qui ne l'est pas
 *
 * **Inspecté** : tous les fichiers `.ts` et `.vue` de `app/core/` et `app/modules/`, plus
 * `app/pages/`. La recherche porte sur les littéraux de forme `<module>.<objet>.<action>` —
 * exactement la nomenclature de `0016`.
 *
 * **Non inspecté** : une permission construite dynamiquement (`` `cpt.${objet}.lire` ``). Le
 * produit n'en contient aucune, et la convention est de ne pas en écrire — un code de permission
 * assemblé à l'exécution échappe à toute vérification statique, ici comme côté serveur.
 */

import { describe, expect, it } from 'vitest'
import { readFileSync, readdirSync, statSync } from 'node:fs'
import { join } from 'node:path'

import { PERMISSION_ATTRIBUER } from '../modules/comptes/roles'
import { PERMISSION_BASCULER } from '../modules/etablissements/bascule-service'
import { CATALOGUE_TUILES } from '../core/accueil/tuiles'
import {
  CLES_ERREUR_METIER,
  CLES_MOTIF_MOT_DE_PASSE,
  REFUS_INATTENDU,
  cleDeRefus,
} from '../core/erreurs/codes'
import { cumuler, detient, detientUne } from '../core/rbac'
import en from '../core/i18n/en.json'
import fr from '../core/i18n/fr.json'

const RACINE = process.cwd()
const MIGRATION = join(RACINE, '../backend/migrations/0016_roles_permissions.sql')

/** Les arbres réellement balayés. Déclarés, et leur non-vacuité est vérifiée. */
const ARBRES = ['core', 'modules', 'pages']

/** Nomenclature `<module>.<objet>.<action>` — celle de la migration `0016`. */
const FORME_PERMISSION = /'((?:etb|cpt|pdv|heb|cai|stk|fis)\.[a-z_]+\.[a-z_]+)'/g

/** Les dix-sept codes du référentiel, lus de la migration qui les insère. */
function referentiel(): Set<string> {
  const sql = readFileSync(MIGRATION, 'utf8')
  const codes = [...sql.matchAll(/\('((?:etb|cpt)\.[a-z_]+\.[a-z_]+)',\s+NULL,/g)].map(m => m[1]!)
  return new Set(codes)
}

/** Tous les fichiers source d'un arbre. */
function fichiers(relatif: string): string[] {
  const trouves: string[] = []
  const descendre = (chemin: string) => {
    for (const entree of readdirSync(chemin)) {
      const complet = join(chemin, entree)
      if (statSync(complet).isDirectory()) {
        descendre(complet)
      }
      else if (/\.(ts|vue)$/.test(entree)) {
        trouves.push(complet)
      }
    }
  }
  descendre(join(RACINE, relatif))
  return trouves
}

describe('le référentiel est lisible et non vide', () => {
  it('la migration 0016 porte bien les dix-sept permissions', () => {
    // Une porte dont la cible est vide passe toujours : si l'extraction cassait — migration
    // renommée, format d'`INSERT` changé —, toutes les assertions suivantes deviendraient
    // vacuellement vraies. Celle-ci l'empêche.
    const codes = referentiel()

    expect(codes.size).toBe(17)
    expect(codes).toContain('cpt.role.attribuer')
    expect(codes).toContain('etb.service.basculer')
  })
})

describe('toute permission nommée par le front existe au référentiel', () => {
  it('les constantes exportées', () => {
    const codes = referentiel()

    expect(codes, 'PERMISSION_ATTRIBUER').toContain(PERMISSION_ATTRIBUER)
    expect(codes, 'PERMISSION_BASCULER').toContain(PERMISSION_BASCULER)
  })

  it('les permissions du catalogue de tuiles', () => {
    const codes = referentiel()

    for (const tuile of CATALOGUE_TUILES) {
      expect(codes, `la tuile « ${tuile.code} »`).toContain(tuile.permission)
    }
  })

  it('tout littéral de forme `<module>.<objet>.<action>` du code source', () => {
    const codes = referentiel()
    const inspectes: string[] = []
    const inconnues: string[] = []

    for (const arbre of ARBRES) {
      const trouves = fichiers(arbre)
      // Décompte par arbre : un arbre vide rendrait la porte muette sans qu'on le remarque.
      expect(trouves.length, `l'arbre « ${arbre} » est vide`).toBeGreaterThan(0)

      for (const chemin of trouves) {
        inspectes.push(chemin)
        const source = readFileSync(chemin, 'utf8')
        for (const trouve of source.matchAll(FORME_PERMISSION)) {
          const code = trouve[1]!
          if (!codes.has(code)) {
            inconnues.push(`${chemin.slice(RACINE.length + 1)} → ${code}`)
          }
        }
      }
    }

    expect(inspectes.length).toBeGreaterThan(20)
    expect(
      inconnues,
      `Ces codes ne figurent dans aucune migration. Une permission que le référentiel ne \n`
      + `connaît pas rend « false » à tous les coups : l'action disparaît de l'interface SANS \n`
      + `erreur, sans page blanche et sans ligne de journal.\n  ${inconnues.join('\n  ')}`,
    ).toEqual([])
  })
})

describe('la comparaison est une égalité, jamais un préfixe', () => {
  it('`cpt.compte.lire` n’ouvre pas `cpt.compte.gerer`', () => {
    // La même règle que `api/src/securite.rs`. Un front plus permissif que le serveur afficherait
    // des actions qui échouent ; un front plus strict cacherait des actions permises.
    const permissions = ['cpt.compte.lire']

    expect(detient(permissions, 'cpt.compte.gerer')).toBe(false)
    expect(detient(permissions, 'cpt.compte')).toBe(false)
    expect(detient(permissions, 'cpt.compte.lire.tout')).toBe(false)
    expect(detient(permissions, 'cpt.compte.lire')).toBe(true)
  })

  it('`detientUne` rend vrai dès la première trouvée, et faux sur une liste vide', () => {
    const permissions = ['cpt.compte.lire']

    expect(detientUne(permissions, ['cpt.compte.gerer', 'cpt.compte.lire'])).toBe(true)
    expect(detientUne(permissions, ['cpt.audit.consulter'])).toBe(false)
    expect(detientUne(permissions, [])).toBe(false)
  })
})

describe('l’union, et la faute qu’elle évite', () => {
  it('cumuler dédoublonne, et ne privilégie aucun rôle', () => {
    const gerant = ['etb.note.lire', 'etb.service.basculer']
    const caissier = ['etb.note.lire']
    const comptable = ['etb.note.lire', 'cpt.audit.consulter']

    const union = cumuler(gerant, caissier, comptable)

    expect(union).toHaveLength(3)
    // La faute de FR-017 : prendre les permissions d'un rôle « principal ». Ici, aucun ordre
    // d'argument ne change le résultat.
    expect(cumuler(comptable, caissier, gerant).slice().sort()).toEqual(union.slice().sort())
  })

  it('un compte sans rôle cumule à vide, sans erreur', () => {
    expect(cumuler()).toEqual([])
    expect(cumuler([], [])).toEqual([])
  })
})

// =================================================================================================
//  Les dix codes d'erreur métier du contrat ont TOUS leur phrase
// =================================================================================================

describe('les dix codes d’erreur métier du contrat', () => {
  /**
   * Les dix de `contracts/http-api.md`, § « Codes d'erreur métier introduits ».
   *
   * Un code que la table ne connaît pas tombe sur une phrase générique honnête — ce qui est
   * mieux qu'une clé affichée en brut, et pire que la phrase qui explique. Sur
   * `derniere_habilitation`, la différence est celle entre « ça n'a pas marché » et « donnez ce
   * droit à quelqu'un d'autre avant de vous le retirer ».
   */
  const CODES_DU_CONTRAT = [
    'identifiants_invalides',
    'session_invalide',
    'permission_absente',
    'methode_non_implementee',
    'identifiant_absent',
    'mot_de_passe_refuse',
    'identifiant_refuse',
    'portee_incompatible',
    'etablissement_inconnu',
    'derniere_habilitation',
  ]

  it('les dix figurent à la table partagée', () => {
    for (const code of CODES_DU_CONTRAT) {
      expect(CLES_ERREUR_METIER, `« ${code} »`).toHaveProperty(code)
    }
    // **Dix, et pas onze.** Une entrée de plus signifierait qu'un code a été inventé côté front,
    // ou que le contrat en a gagné un sans que ce test suive.
    expect(Object.keys(CLES_ERREUR_METIER)).toHaveLength(10)
  })

  it('chacune de leurs clés existe dans les DEUX catalogues', () => {
    for (const code of CODES_DU_CONTRAT) {
      const cle = CLES_ERREUR_METIER[code]!
      for (const [langue, catalogue] of [['fr', fr], ['en', en]] as const) {
        const phrase = cle
          .split('.')
          .reduce<unknown>((noeud, part) => (noeud as Record<string, unknown>)?.[part], catalogue)
        expect(typeof phrase, `« ${code} » → « ${cle} » manque en ${langue}`).toBe('string')
      }
    }
  })

  it('les trois motifs de mot de passe distinguent ce qu’il faut corriger', () => {
    // Le code seul dirait « refusé » ; l'utilisateur doit savoir s'il faut allonger son mot de
    // passe ou en changer complètement.
    for (const [motif, cle] of Object.entries(CLES_MOTIF_MOT_DE_PASSE)) {
      const phrase = cle
        .split('.')
        .reduce<unknown>((noeud, part) => (noeud as Record<string, unknown>)?.[part], fr)
      expect(typeof phrase, `« ${motif} » n'a pas de phrase`).toBe('string')
    }
    expect(fr.erreurs.mot_de_passe.trop_court).not.toBe(fr.erreurs.mot_de_passe.compromis)
  })

  it('`motif_cle` PRIME sur le code — elle enseigne là où le code constate', () => {
    expect(cleDeRefus('portee_incompatible', 'un.motif.du.referentiel'))
      .toBe('un.motif.du.referentiel')
  })

  it('un code inconnu tombe sur une phrase honnête, jamais sur une clé en brut', () => {
    expect(cleDeRefus('quelque_chose_de_neuf')).toBe(REFUS_INATTENDU)
    expect(cleDeRefus(undefined)).toBe(REFUS_INATTENDU)
    expect(typeof fr.erreurs.inattendue).toBe('string')
  })

  it('une table de module PRIME sur la table partagée', () => {
    // C'est ce qui permet à un module de dire mieux, sans que les autres perdent la phrase
    // commune.
    expect(cleDeRefus('etablissement_inconnu', null, { etablissement_inconnu: 'x.propre' }))
      .toBe('x.propre')
    expect(cleDeRefus('etablissement_inconnu')).toBe('comptes.refus.etablissement_inconnu')
  })
})
