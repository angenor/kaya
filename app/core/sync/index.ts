/**
 * File de synchronisation locale — **coquille structurelle du cycle 001**.
 *
 * La logique de synchronisation relève du cycle SYN. Ce que ce cycle livre est la **contrainte de
 * type** qui rend la porte P-13 opposable, et l'indicateur d'état réseau que le principe VI
 * impose.
 *
 * # Trois règles du principe VI, posées ici avant tout code métier
 *
 * 1. **Aucune donnée B, C ou D en cache d'écriture sur un terminal.** Ces entités sont en lecture
 *    seule côté client. La file n'accepte que des {@link OperationClasseA} — au niveau du type.
 * 2. **L'interface annonce immédiatement toute action indisponible.** Jamais de grisé silencieux,
 *    jamais d'échec après coup, **jamais de mise en file « au cas où »**. C'est pourquoi
 *    {@link enfilerOperation} refuse au lieu de stocker : mettre en file une opération qu'on ne
 *    sait pas rejouer revient à promettre à l'utilisateur quelque chose qui n'arrivera pas.
 * 3. **La file se vide au retour au premier plan**, sur toutes les plateformes. iOS n'a pas de
 *    synchronisation en arrière-plan ; `BGTaskScheduler` et `WorkManager` sont des optimisations,
 *    jamais des hypothèses.
 *
 * # Le vidage a un ORDRE, et il vit dans `vidage.ts`
 *
 * **Rafraîchir d'abord, vider ensuite** — jamais l'inverse (research R-18). L'ordre est porté par
 * {@link viderFile}, seule sortie de la file, et non par la discipline de l'appelant : le défaut
 * qu'il évite ne se manifeste qu'après une coupure plus longue que la durée du jeton, donc jamais
 * en développement.
 */

import { estTypeClasseA, type EntreeFile, type OperationClasseA } from './classes'

export * from './classes'
export { viderFile, type Envoyeur, type ResultatVidage } from './vidage'

/** État du réseau, affiché en permanence (principe VI). */
export type EtatReseau = 'connecte' | 'degrade' | 'hors_ligne'

/** Refus d'enfilement, avec son motif. */
export class OperationRefusee extends Error {
  constructor(
    readonly type: string,
    motif: string,
  ) {
    super(motif)
    this.name = 'OperationRefusee'
  }
}

/**
 * File locale — **coquille**. La persistance et le vidage viennent du cycle SYN.
 */
export class FileLocale {
  private entrees: EntreeFile[] = []

  /**
   * Enfile une opération de classe A.
   *
   * La signature refuse à la **compilation** toute charge non marquée : `charge` est de type
   * {@link OperationClasseA}, qui ne peut s'obtenir que par `marquerClasseA`.
   *
   * La vérification du type d'opération, elle, se fait à l'**exécution** : elle attrape le cas où
   * `marquerClasseA` aurait été appelée sur une opération qui n'est pas de classe A. Deux
   * barrières, deux moments — la première empêche l'erreur d'être écrite, la seconde empêche une
   * marque abusive d'avoir un effet.
   */
  enfiler<T>(entree: EntreeFile<T>): void {
    if (!estTypeClasseA(entree.type)) {
      throw new OperationRefusee(
        entree.type,
        `« ${entree.type} » n'est pas déclarée de classe A dans docs/registre-classes-offline.md. `
          + 'Une opération B, C ou D ne va JAMAIS en file : elle est annoncée indisponible à '
          + "l'utilisateur, immédiatement (principe VI).",
      )
    }
    this.entrees.push(entree as EntreeFile)
  }

  /**
   * Retire une entrée **acquittée par le serveur**.
   *
   * Le seul appelant légitime est {@link viderFile} : c'est lui qui porte l'ordre
   * « rafraîchir d'abord, vider ensuite ». Retirer ailleurs reviendrait à jeter une écriture que
   * personne n'a confirmée.
   */
  retirer(id: string): void {
    this.entrees = this.entrees.filter(entree => entree.id !== id)
  }

  /** Nombre d'éléments en attente — affiché en permanence par l'indicateur de synchronisation. */
  get enAttente(): number {
    return this.entrees.length
  }

  /** Contenu de la file, en lecture seule. */
  lister(): readonly EntreeFile[] {
    return this.entrees
  }
}

/**
 * Une opération de cette classe est-elle réalisable dans l'état réseau courant ?
 *
 * Utilisée par l'interface **avant** de proposer l'action, jamais après l'avoir tentée. C'est
 * cette distinction qui fait la différence entre « cette action demande une connexion » affiché
 * tout de suite, et un échec après trente secondes d'attente.
 */
export function operationRealisable(type: string, reseau: EtatReseau): boolean {
  if (reseau === 'connecte') {
    return true
  }
  // Hors ligne ou dégradé : seules les opérations de classe A passent.
  return estTypeClasseA(type)
}

export type { EntreeFile, OperationClasseA }
