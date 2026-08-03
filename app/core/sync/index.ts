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
 *
 * # « Combien attendent ? » se demande d'ailleurs — `attente.ts`
 *
 * La file est une instance ; le produit a besoin de l'interroger sans la tenir. C'est ce que porte
 * `attente.ts` : un point unique où l'on demande combien d'écritures ne sont pas parties, branché
 * par SYN-01 et **répondant 0 en attendant**. Le premier appelant est le bouton « passer la main »
 * de la coquille, qui refuse de purger le stockage sur une file non vide.
 */

import type { PlatformAdapter } from '~/core/platform'

import { estTypeClasseA, type EntreeFile, type OperationClasseA } from './classes'
import { ouvrirMagasin, type MagasinFile } from './persistance'
import { type EntreeQuarantaine } from './quarantaine'

export * from './classes'
export { brancherFile, ecrituresEnAttente, fileBranchee, fileCourante } from './attente'
export { viderFile, type Envoyeur, type IssueEnvoi, type ResultatVidage } from './vidage'
export {
  classer,
  cleMotifRefus,
  type EntreeQuarantaine,
  type Suite,
} from './quarantaine'
export { CLES_PERSISTANCE, ouvrirMagasin, type MagasinFile } from './persistance'
export {
  apresEcritureReussie,
  brancherEnvoi,
  debrancherEnvoi,
  declencherEnvoi,
  delaiReessaiMs,
} from './envoi'
export {
  etatSynchronisation,
  signalerChangement,
  useEtatSynchronisation,
  type EtatSynchronisation,
} from './etat'
export { uuidV7 } from './uuid-v7'

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
 * File locale — **persistante depuis le cycle 005**.
 *
 * # Ce qui a changé, et ce qui n'a surtout pas changé
 *
 * Elle survit désormais au rechargement et à l'extinction, chiffrée (voir `persistance.ts`), et
 * porte sa quarantaine. **Ce qui n'a pas changé est la règle qui la tient** : la file n'a
 * toujours **aucun chemin de sortie autre que {@link viderFile}**, et c'est ce qui porte l'ordre
 * rafraîchir-avant-vider — pas la discipline des appelants.
 *
 * # Pourquoi `ouvrir` est asynchrone, et pas le constructeur
 *
 * La clé de chiffrement vient du coffre système, dont l'accès est asynchrone sur les quatre
 * plateformes. Un constructeur ne peut pas attendre : il rendrait une file vide qui se remplirait
 * « plus tard », et le témoin afficherait zéro pendant ce temps — c'est-à-dire au moment précis où
 * l'utilisateur ouvre l'application pour vérifier que son travail est parti.
 *
 * # L'écriture en mémoire est SYNCHRONE, la persistance suit
 *
 * `enfiler` ne rend pas de promesse, et c'est délibéré : une saisie doit être acceptée
 * immédiatement (FR-002 de la story — « acceptée, sans message d'erreur »). L'écriture au
 * stockage part derrière, et son échec **ne fait pas échouer la saisie** — la file reste en
 * mémoire, ce qui est déjà mieux que de perdre la commande. La conséquence est écrite plutôt que
 * découverte : dans ce cas, elle ne survivra pas au rechargement.
 */
export class FileLocale {
  private entrees: EntreeFile[] = []

  /** Ce qui a été définitivement refusé — **consultable, jamais bloquant**. */
  private refusees: EntreeQuarantaine[] = []

  /**
   * Le magasin chiffré. `null` pour une file **de mémoire seule** — celle qu'un test construit,
   * et celle qui sert tant qu'aucun adaptateur n'est passé.
   */
  private magasin: MagasinFile | null = null

  /**
   * Ouvre la file de cet appareil : lit la clé au coffre, déchiffre ce qui attendait.
   *
   * Un cryptogramme illisible fait repartir la file **vide** plutôt qu'échouer — voir la note de
   * `persistance.ts`. Refuser de démarrer bloquerait le terminal sur un état qu'aucun exploitant
   * ne peut réparer.
   */
  static async ouvrir(adaptateur: PlatformAdapter): Promise<FileLocale> {
    const file = new FileLocale()
    file.magasin = await ouvrirMagasin(adaptateur)
    file.entrees = await file.magasin.charger()
    return file
  }

  /**
   * Écrit l'état courant au stockage. **Volontairement sans `await` chez l'appelant.**
   *
   * Une saisie ne doit pas attendre le disque. Les échecs sont absorbés par le stockage lui-même
   * (voir `stockagePersistantMoteur`), qui ne lève jamais.
   */
  private persister(): void {
    void this.magasin?.enregistrer(this.entrees)
  }

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
    this.persister()
  }

  /**
   * Écarte une entrée **définitivement refusée** par le serveur.
   *
   * Elle quitte la file d'envoi et devient consultable. Elle ne bloque plus rien : ni les
   * écritures suivantes, ni le geste de passer la main — `ecrituresEnAttente()` compte ce qui
   * attend de partir, pas ce que le serveur ne reprendra jamais.
   */
  mettreEnQuarantaine(id: string, code: string, refuseeLe: string): void {
    const entree = this.entrees.find(e => e.id === id)
    if (!entree) {
      return
    }
    this.entrees = this.entrees.filter(e => e.id !== id)
    this.refusees.push({ entree, code, refuseeLe })
    this.persister()
  }

  /**
   * Remet une entrée de quarantaine dans la file — **geste explicite de l'utilisateur**, jamais
   * automatique.
   *
   * Un rejeu automatique d'un refus définitif boucllerait indéfiniment. C'est l'exploitant qui
   * décide, depuis l'écran `S1`, après avoir lu le motif.
   */
  relancerDepuisQuarantaine(id: string): void {
    const rang = this.refusees.findIndex(e => e.entree.id === id)
    if (rang === -1) {
      return
    }
    const [reprise] = this.refusees.splice(rang, 1)
    if (reprise) {
      // Le compteur de tentatives repart de zéro : c'est une décision humaine nouvelle, pas la
      // suite de la série qui avait échoué.
      this.entrees.push({ ...reprise.entree, tentatives: 0 })
    }
    this.persister()
  }

  /** Compte une tentative d'envoi sur une entrée — alimente l'intervalle croissant et `S1`. */
  compterTentative(id: string): void {
    this.entrees = this.entrees.map(e =>
      e.id === id ? { ...e, tentatives: e.tentatives + 1 } : e,
    )
    this.persister()
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
    this.persister()
  }

  /** Nombre d'éléments en attente — affiché en permanence par l'indicateur de synchronisation. */
  get enAttente(): number {
    return this.entrees.length
  }

  /** Nombre d'écritures **définitivement refusées** — affiché par `S1`, jamais par le témoin. */
  get enQuarantaine(): number {
    return this.refusees.length
  }

  /** Contenu de la file, en lecture seule. */
  lister(): readonly EntreeFile[] {
    return this.entrees
  }

  /** Ce qui a été refusé, en lecture seule. */
  quarantaine(): readonly EntreeQuarantaine[] {
    return this.refusees
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
