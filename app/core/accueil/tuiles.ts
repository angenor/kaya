/**
 * **Le catalogue des tuiles de l'accueil `R1`** — et la permission qui ouvre chacune.
 *
 * # L'accueil est un tableau de bord filtré, jamais un menu figé
 *
 * Principe VII : quatre comptes, quatre accueils, sur la même application. Une tuile dont
 * l'utilisateur n'a pas la permission est **absente**, pas grisée ; une tuile dont le module
 * d'activité n'est pas activé dans l'établissement est **absente** aussi. Le grisé apprend à
 * l'utilisateur, à chaque écran et tous les jours, qu'une partie du produit lui est refusée.
 *
 * # Le catalogue ne contient que des écrans QUI EXISTENT — principe X
 *
 * Les maquettes de `R1` montrent un accueil riche : tables ouvertes, chambres occupées, argent non
 * encaissé, « Ajouter une commande ». Ces tuiles ouvrent des écrans des cycles PDV, SEJ, CAI et
 * HEB, dont **aucun n'est livré**. Les poser maintenant produirait une interface qui promet ce
 * qu'elle ne sait pas faire — « prêt ≠ construit », et c'est le principe X qui l'interdit, pas une
 * prudence de rédaction.
 *
 * Trois tuiles à ce cycle, donc, et ce sont les trois écrans réels du produit :
 * `G1`, `G3` et `G4`.
 *
 * # Le mécanisme du module d'activité EXISTE, et aucune tuile ne l'emploie encore
 *
 * `moduleRequis` est là, filtré, et testé. Aucune des trois tuiles du cycle n'en porte : elles
 * sont toutes transverses, exactement comme les dix-sept permissions de `0016` ont toutes un
 * `module_code` à `NULL`. Le poser à vide n'est pas décoratif — c'est ce qui fait que le cycle
 * HEB ajoutera une ligne au catalogue, pas une branche dans le filtre.
 *
 * # Une tuile issue de plusieurs rôles n'apparaît QU'UNE FOIS — FR-027
 *
 * Elle est tenue par la structure, pas par un dédoublonnage : le catalogue est une liste de
 * tuiles, pas une liste par rôle. Adjoua porte trois rôles ; la tuile « Réglages » est nommée une
 * seule fois, et sa permission est dans l'union. Il n'y a **aucun endroit** où elle pourrait se
 * dupliquer.
 */

import { detient, type Permissions } from '~/core/rbac'

/** Une tuile de l'accueil. */
export interface Tuile {
  /** Identifiant stable — sert de clé de rendu et d'ancre de test. */
  readonly code: string
  /** Clé i18n du libellé. **Jamais une phrase.** */
  readonly libelleCle: string
  /** Clé i18n de la ligne d'explication, sous le libellé. */
  readonly descriptionCle: string
  /** Glyphe Phosphor, **décoratif** : la tuile porte déjà son libellé traduit. */
  readonly icone: string
  /** La permission qui l'ouvre. Une tuile sans permission serait une tuile que rien ne garde. */
  readonly permission: string
  /** Route Nuxt ouverte au clic. */
  readonly route: string
  /**
   * Module d'activité à activer pour que la tuile existe.
   *
   * `undefined` = transverse. **Aucune tuile de ce cycle n'en porte** — voir l'en-tête.
   */
  readonly moduleRequis?: string
}

/**
 * Les tuiles du produit, dans leur ordre d'affichage.
 *
 * L'ordre est **stable et indépendant de la locale** : trier sur le libellé traduit ferait changer
 * l'accueil en passant du français à l'anglais. Même raison que l'`ordre` des référentiels.
 */
export const CATALOGUE_TUILES: readonly Tuile[] = [
  {
    code: 'etablissement',
    libelleCle: 'accueil.tuiles.etablissement.libelle',
    descriptionCle: 'accueil.tuiles.etablissement.description',
    icone: 'ph-storefront',
    permission: 'etb.etablissement.lire',
    route: '/etablissement',
  },
  {
    code: 'comptes',
    libelleCle: 'accueil.tuiles.comptes.libelle',
    descriptionCle: 'accueil.tuiles.comptes.description',
    icone: 'ph-users-three',
    permission: 'cpt.compte.lire',
    route: '/comptes',
  },
  // ── Cycle 004 — la PREMIÈRE tuile rattachée à un module d'activité ──────────────────────
  //
  // `moduleRequis` existait depuis le cycle 003, filtré et testé, mais **aucune tuile ne le
  // portait** : son mécanisme était vérifié sur une cible fictive. C'est la première qui l'exerce
  // pour de vrai — un établissement qui ne fait pas d'hébergement n'a pas de tuile « Vos
  // formules », et elle est ABSENTE, jamais grisée (principe VII).
  {
    code: 'hebergement-offre',
    libelleCle: 'accueil.tuiles.hebergement_offre.libelle',
    descriptionCle: 'accueil.tuiles.hebergement_offre.description',
    icone: 'ph-bed',
    permission: 'heb.offre.lire',
    route: '/hebergement',
    moduleRequis: 'HEBERGEMENT',
  },
  {
    code: 'hebergement-chambres',
    libelleCle: 'accueil.tuiles.hebergement_chambres.libelle',
    descriptionCle: 'accueil.tuiles.hebergement_chambres.description',
    icone: 'ph-door-open',
    permission: 'heb.offre.lire',
    route: '/chambres',
    moduleRequis: 'HEBERGEMENT',
  },
  {
    code: 'journal-audit',
    libelleCle: 'accueil.tuiles.journal.libelle',
    descriptionCle: 'accueil.tuiles.journal.description',
    icone: 'ph-list-magnifying-glass',
    permission: 'cpt.audit.consulter',
    route: '/journal-audit',
  },
]

/**
 * Les tuiles **visibles** pour cet utilisateur, dans cet établissement.
 *
 * @param permissions Union des rôles portés — jamais celles d'un rôle « principal ».
 * @param modulesActifs Codes des modules d'activité activés. Une liste vide est **valide** : c'est
 *   l'état d'une résidence meublée, et le traiter comme une erreur ferait échouer l'accueil sur un
 *   état parfaitement normal.
 */
export function tuilesVisibles(
  permissions: Permissions,
  modulesActifs: readonly string[] = [],
): Tuile[] {
  return CATALOGUE_TUILES.filter((tuile) => {
    if (!detient(permissions, tuile.permission)) {
      return false
    }
    // Un module inactif rend la tuile **absente**, jamais grisée (principe VII).
    return !tuile.moduleRequis || modulesActifs.includes(tuile.moduleRequis)
  })
}

/**
 * L'accueil est-il vide pour cet utilisateur ?
 *
 * Le cas existe et n'est pas une erreur : un compte fraîchement créé, avant qu'on lui donne un
 * rôle. Il obtient un **état vide explicite**, pas un écran blanc ni un message d'échec — il n'y a
 * rien à réessayer, il y a quelqu'un à prévenir.
 */
export function accueilVide(permissions: Permissions, modulesActifs: readonly string[] = []): boolean {
  return tuilesVisibles(permissions, modulesActifs).length === 0
}
