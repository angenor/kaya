/**
 * `PlatformAdapter` — **la seule porte vers le natif** (principe VII).
 *
 * Impression, scan, OCR, stockage sécurisé, notifications, géolocalisation et état réseau passent
 * par ici. **Aucun composant n'importe `@tauri-apps/api`** : la règle ESLint
 * `no-restricted-imports` (porte P-15) le refuse partout sauf dans ce répertoire.
 *
 * # Le type de retour EST le contrat
 *
 * {@link ResultatCapacite} force chaque appelant à traiter le cas « cette plateforme ne sait pas
 * faire ça ». Une méthode qui renverrait `Promise<void>` et lèverait une exception laisserait le
 * choix de l'ignorer — et l'interface grisée-sans-explication réapparaîtrait au premier écran
 * mobile.
 *
 * Le principe VII l'exige explicitement : **une capacité absente le DIT à l'utilisateur**, elle
 * n'échoue jamais en silence.
 */

/** Motif d'indisponibilité d'une capacité. */
export type CapaciteIndisponible =
  /** La plateforme ne fournit pas cette capacité — un navigateur n'imprime pas en thermique. */
  | 'plateforme_non_supportee'
  /** L'utilisateur a refusé l'autorisation système. */
  | 'permission_refusee'
  /** Le matériel est absent : ni imprimante appairée, ni caméra. */
  | 'materiel_absent'
  /** La capacité exige le réseau, et il n'est pas là. */
  | 'reseau_requis'

/**
 * Résultat d'un appel de capacité.
 *
 * **Jamais un `throw` silencieux.** Le discriminant `disponible` oblige à traiter les deux cas ;
 * `raison` porte de quoi construire un message que l'utilisateur comprend.
 */
export type ResultatCapacite<T = void> =
  | { disponible: true, valeur: T }
  | { disponible: false, raison: CapaciteIndisponible }

/** État du réseau, affiché en permanence (principe VI). */
export type EtatReseau = 'connecte' | 'degrade' | 'hors_ligne'

/** Document à imprimer. */
export interface DocumentImprimable {
  /** Contenu prêt à imprimer. */
  readonly contenu: string
  /** Largeur du papier, en millimètres — 58 ou 80 sur les thermiques du marché. */
  readonly largeurMm: 58 | 80
  /**
   * Un document **opérationnel** porte obligatoirement la mention
   * « Document non fiscal — ne tient pas lieu de facture » (principe V).
   */
  readonly fiscal: boolean
}

/** Champs extraits d'une pièce d'identité par OCR. */
export interface ChampsPieceIdentite {
  readonly nom?: string
  readonly prenoms?: string
  readonly numero?: string
  readonly dateNaissance?: string
}

/** Position géographique. */
export interface Position {
  readonly latitude: number
  readonly longitude: number
  readonly precisionMetres: number
}

/** Notification locale. */
export interface Notification {
  readonly titre: string
  readonly corps: string
}

/**
 * Stockage sécurisé — Keystore Android, Keychain iOS.
 *
 * **Aucun secret dans le binaire Tauri** (principe IX) : il est décompilable. Les clés
 * d'enrôlement d'appareil vivent ici, générées sur l'appareil, et n'en sortent jamais.
 */
export interface StockageSecurise {
  lire(cle: string): Promise<ResultatCapacite<string | null>>
  ecrire(cle: string, valeur: string): Promise<ResultatCapacite>
  supprimer(cle: string): Promise<ResultatCapacite>
  /** **Purge à la déconnexion** — exigée par le principe VI. */
  purger(): Promise<ResultatCapacite>
}

/** Capacités natives, vues par le métier. */
export interface PlatformAdapter {
  readonly nom: 'desktop' | 'android' | 'ios' | 'web'

  imprimer(doc: DocumentImprimable): Promise<ResultatCapacite>
  scanner(): Promise<ResultatCapacite<string>>
  ocrPieceIdentite(image: Blob): Promise<ResultatCapacite<ChampsPieceIdentite>>
  readonly stockageSecurise: StockageSecurise
  notifier(notification: Notification): Promise<ResultatCapacite>
  geolocaliser(): Promise<ResultatCapacite<Position>>
  etatReseau(): EtatReseau
}

/** Raccourci pour déclarer une capacité absente. */
export function indisponible(raison: CapaciteIndisponible): { disponible: false, raison: CapaciteIndisponible } {
  return { disponible: false, raison }
}

/** Raccourci pour un résultat disponible. */
export function disponible<T>(valeur: T): { disponible: true, valeur: T } {
  return { disponible: true, valeur }
}

/**
 * Stockage sécurisé **absent** — implémentation partagée par les plateformes qui n'en ont pas.
 *
 * Le web est le cas réel : `localStorage` n'est pas un stockage sécurisé, et l'y faire passer
 * donnerait à l'appelant une garantie fausse. Mieux vaut annoncer l'absence.
 */
export const stockageSecuriseAbsent: StockageSecurise = {
  async lire() {
    return indisponible('plateforme_non_supportee')
  },
  async ecrire() {
    return indisponible('plateforme_non_supportee')
  },
  async supprimer() {
    return indisponible('plateforme_non_supportee')
  },
  async purger() {
    return indisponible('plateforme_non_supportee')
  },
}

/**
 * État du réseau, tel que le navigateur le rapporte.
 *
 * **`navigator.onLine` ne dit pas si le serveur est joignable** — seulement si une interface
 * réseau est active. À Abengourou, une connexion 3G qui affiche « en ligne » sans porter la
 * moindre requête est le cas courant, pas l'exception. D'où l'état intermédiaire `degrade`, que
 * le cycle SYN alimentera depuis les échecs réels de requête. Le rapporter comme `connecte`
 * produirait exactement l'échec après coup que le principe VI interdit.
 */
export function etatReseauNavigateur(): EtatReseau {
  if (typeof navigator === 'undefined') {
    return 'connecte'
  }
  return navigator.onLine ? 'connecte' : 'hors_ligne'
}
