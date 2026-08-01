/**
 * **Le patron d'écriture front du produit** — ETB-02, activation et désactivation d'un service.
 *
 * Une seule opération sur les vingt et une que l'API expose. C'est délibéré : le module doré a
 * manqué son patron front en visant trop large, et *« une opération, complète et documentée, vaut
 * mieux que vingt et une approximatives »*. Les vingt autres suivront ce fichier, cycle par cycle.
 *
 * Elle a été retenue parce qu'elle exerce **tout** ce qu'il fallait établir : appel typé,
 * chargement, refus métier réel, refus de permission, refus hors ligne, et un effet visible
 * immédiatement puisqu'un service inactif est **absent** de l'interface, jamais grisé.
 *
 * Le déroulé complet, avec ses huit points, est écrit dans `docs/module-dore.md`, section
 * « La septième couche — le patron d'écriture front ».
 *
 * ─────────────────────────────────────────────────────────────────────────────────────────────
 *
 * # 1 · CLASSE C — le refus vient AVANT l'appel, jamais après
 *
 * `docs/registre-classes-offline.md` §5.1 : `etablissement_module` — activation, désactivation —
 * est de **classe C**. Trois conséquences, toutes tenues ici :
 *
 * - l'opération **n'entre jamais en file** ; `core/sync` la refuserait de toute façon, son type
 *   n'étant pas déclaré dans `TYPES_CLASSE_A` ;
 * - hors ligne, l'interface **dit immédiatement** que l'action demande le réseau — pas de grisé
 *   silencieux, pas d'échec après trente secondes d'attente (principe VI) ;
 * - la vérification se fait **avant** l'appel réseau, dans {@link basculerService}, et pas dans le
 *   composant : un second appelant oublierait la garde, et la faute ne se verrait qu'en clientèle.
 *
 * # 2 · Le message d'erreur est traduit du CODE, jamais du texte du serveur
 *
 * `CorpsErreur` porte trois choses distinctes, et la confusion entre elles est la faute qu'on
 * commettrait :
 *
 * | Champ | Ce que c'est | Ce qu'on en fait |
 * |---|---|---|
 * | `code` | Identifiant stable, **jamais traduit** | **La clé sur laquelle on branche l'i18n** |
 * | `message` | Diagnostic pour les journaux | **Jamais affiché** — il est en anglais technique et nomme des tables |
 * | `motif_cle` | Clé i18n fournie par le référentiel | Affichée quand elle est là, elle enseigne là où le code constate |
 *
 * Afficher `message` mettrait un nom de table sous les yeux de l'exploitant. La table
 * {@link CLES_DE_REFUS} est **explicite et fermée** : un code inconnu tombe sur un message
 * générique honnête plutôt que sur une clé i18n manquante affichée en brut.
 */

import { creerClientKaya } from '@kaya/client'

import { enTetesAuth, type ContexteAppel } from '~/core/auth'
import { uuidV7 } from '~/core/sync/uuid-v7'
import type { EtatReseau } from '~/core/platform'

/**
 * Type d'opération, au vocabulaire du registre hors-ligne.
 *
 * **Il n'est PAS dans `TYPES_CLASSE_A`**, et ce n'est pas un oubli : l'y mettre autoriserait la
 * mise en file d'une opération de classe C, ce que la porte P-13 refuse.
 */
export const TYPE_OPERATION = 'etablissement_module.bascule'

/**
 * Permission requise — **le provisoire nommé est levé** (CPT-02).
 *
 * Ce code n'est plus une convention anticipée : c'est une **ligne du référentiel**
 * `comptes.permission`, posée par la migration `0016`, portée par `proprietaire` et `gerant`, et
 * rendue par `GET /api/v1/referentiels/permissions`. Le serveur la vérifie sur le même code
 * (`api/src/routes/services.rs`), et `app/tests/permissions.spec.ts` échoue si une permission
 * nommée par le front n'existe dans aucune migration — une permission qui ne garde rien est une
 * promesse sans contrepartie (FR-021).
 *
 * La **règle d'affichage** n'a pas changé d'un mot, et c'était le point de la nommer d'avance :
 * permission absente → action **absente**.
 */
export const PERMISSION_BASCULER = 'etb.service.basculer'

/** Un obstacle à la désactivation, tel que l'API le rend. */
export interface ObstacleVue {
  module_code: string
  /** **Clé i18n — jamais une phrase.** */
  motif_cle: string
  /** Séparé du motif : le pluriel ne s'accorde pas partout de la même façon. */
  nombre: number
}

/** Ce qu'une bascule produit. Un seul type de retour, jamais d'exception à rattraper au vol. */
export type ResultatBascule =
  | { issue: 'succes' }
  | {
    issue: 'refus'
    /** Clé i18n du message à afficher. Toujours renseignée. */
    cle: string
    /** Valeurs d'interpolation — le nom du service, la valeur refusée. */
    valeurs?: Record<string, unknown>
    /** Obstacles à une désactivation, chacun avec sa propre clé et son nombre. */
    obstacles?: ObstacleVue[]
    /** Le refus vient-il de l'absence de réseau ? L'interface ne le rend pas de la même façon. */
    reseau?: boolean
  }

/**
 * Codes du contrat → clés i18n. **Table explicite et fermée.**
 *
 * Chaque entrée correspond à un code réellement produit par
 * `backend/crates/socle/etablissements/src/modules/modele.rs`. Un code absent d'ici tombe sur
 * `inattendue` : mieux vaut une phrase honnête et générique qu'une clé i18n affichée telle quelle.
 */
const CLES_DE_REFUS: Record<string, string> = {
  module_inconnu: 'etablissement.services.refus.module_inconnu',
  module_non_implemente: 'etablissement.services.refus.module_non_implemente',
  desactivation_bloquee: 'etablissement.services.refus.desactivation_bloquee',
  etablissement_inconnu: 'etablissement.services.refus.etablissement_inconnu',
}

const REFUS_INATTENDU = 'etablissement.services.refus.inattendue'
const REFUS_PERMISSION = 'etablissement.services.refus.permission'
const REFUS_RESEAU = 'etablissement.services.refus.reseau'

/** Le corps d'erreur du contrat, réduit à ce que l'interface en consomme. */
interface CorpsErreur {
  code?: string
  motif_cle?: string | null
  obstacles?: ObstacleVue[]
  valeur?: string | null
}

/**
 * Active ou désactive un service.
 *
 * **Un seul point d'entrée pour les deux sens**, comme le `PUT` qu'il appelle : deux fonctions
 * distinctes laisseraient deux chemins pour un même état, et un jour deux comportements.
 *
 * @param reseau État réseau **au moment du geste**, lu depuis `PlatformAdapter`. Passé en
 *   paramètre plutôt que lu ici : la fonction reste testable sans navigateur, et l'appelant est
 *   obligé de constater qu'il y a une question d'état réseau à poser.
 */
export async function basculerService(
  contexte: ContexteAppel,
  etablissementId: string,
  moduleCode: string,
  actif: boolean,
  reseau: EtatReseau,
): Promise<ResultatBascule> {
  // CLASSE C — le refus est immédiat, et il précède l'appel. Pas de mise en file « au cas où » :
  // promettre un envoi qu'on ne sait pas rejouer est pire que dire non (principe VI).
  if (reseau !== 'connecte') {
    return { issue: 'refus', cle: REFUS_RESEAU, reseau: true }
  }

  const client = creerClientKaya(contexte.baseUrl)

  // **Appel typé par le client généré, jamais un `fetch` écrit à la main.** Le chemin, la forme du
  // corps et celle de la réponse viennent de `clients/ts/types.gen.ts`, dérivé du contrat OpenAPI
  // (porte P-01). Renommer `module_code` côté serveur fait échouer la compilation ICI, au lieu de
  // produire un `undefined` que personne ne verrait avant la démonstration.
  const reponse = await client.PUT(
    '/api/v1/etablissements/{etablissement_id}/services/{module_code}',
    {
      params: { path: { etablissement_id: etablissementId, module_code: moduleCode } },
      // UUID v7 **généré côté client** : c'est lui qui rend le rejeu inoffensif (principe VI).
      // Il n'est employé qu'à la première activation — une réactivation met à jour la ligne
      // existante, ce qui est exactement ce qui restitue l'état antérieur.
      body: { id: uuidV7(), actif },
      // **Un seul en-tête depuis CPT-01.** `x-kaya-tenant` et `x-kaya-compte` laissaient
      // l'appelant choisir son tenant ; l'API ne les accepte plus, et le serveur lit le tenant
      // dans le jeton qu'il a signé.
      headers: enTetesAuth(contexte),
    },
  )

  if (!reponse.error) {
    return { issue: 'succes' }
  }

  // `403` n'a pas de corps au contrat : l'absence de permission ne se diagnostique pas, elle se
  // constate. En pratique l'utilisateur ne devrait jamais la voir — l'action lui est ABSENTE
  // (principe VII). Le message existe pour le cas où ses droits changent pendant qu'il regarde
  // l'écran, ce qui est le seul chemin qui y mène.
  if (reponse.response.status === 403) {
    return { issue: 'refus', cle: REFUS_PERMISSION }
  }

  const corps = reponse.error as CorpsErreur

  // `motif_cle` PRIME sur la clé du code quand le référentiel en fournit une : c'est ce qui permet
  // à deux refus de même code de dire deux choses différentes — l'un enseigne (« une capacité non
  // consommée ne se déclare pas »), l'autre constate (« pas encore implémenté »). Elle vient du
  // serveur, mais c'est une CLÉ, pas une phrase : elle traverse `t()` comme les autres.
  const cle = corps.motif_cle
    || (corps.code && CLES_DE_REFUS[corps.code])
    || REFUS_INATTENDU

  return {
    issue: 'refus',
    cle,
    valeurs: { valeur: corps.valeur ?? moduleCode },
    obstacles: corps.obstacles,
  }
}
