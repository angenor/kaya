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
 * Ce que le stockage garantit **réellement** contre la lecture par un tiers.
 *
 * # Pourquoi c'est dans le type, et pas dans un commentaire
 *
 * `StockageSecurise` est un nom, et un nom ne garantit rien. Le web n'a pas de coffre système :
 * son implémentation s'appuie sur `localStorage`, lisible par tout script de la même origine.
 * L'appeler « sécurisé » sans plus de précision donnerait à l'appelant une garantie fausse — et la
 * faute ne se verrait qu'au premier audit, sur un jeton de rafraîchissement de quatre-vingt-dix
 * jours.
 *
 * Porter le niveau **dans le type** oblige l'appelant qui exige un coffre matériel à le vérifier,
 * et laisse celui qui accepte le compromis le déclarer explicitement.
 */
export type NiveauGarantieStockage =
  /** Coffre du système — Keystore Android, Keychain iOS. La valeur ne sort pas de l'appareil. */
  | 'coffre_systeme'
  /**
   * Aucune garantie contre un script de la même origine : c'est le web.
   *
   * La contrepartie est portée ailleurs — rotation du jeton à chaque usage, révocation de toute
   * la famille dès qu'une copie est détectée, et coupure immédiate depuis « Appareils connectés ».
   */
  | 'aucune'
  /** La plateforme ne fournit aucun stockage : tout appel échoue. */
  | 'indisponible'

/**
 * Stockage sécurisé — Keystore Android, Keychain iOS.
 *
 * **Aucun secret dans le binaire Tauri** (principe IX) : il est décompilable. Les clés
 * d'enrôlement d'appareil vivent ici, générées sur l'appareil, et n'en sortent jamais.
 */
export interface StockageSecurise {
  /** Ce que cette implémentation garantit — **à lire avant d'y ranger un secret**. */
  readonly garantie: NiveauGarantieStockage
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
 * C'est le cas des trois coquilles Tauri (desktop, Android, iOS) tant que l'enrôlement d'appareil
 * de CPT-05 n'a pas livré l'accès au Keystore et au Keychain. Annoncer l'absence est ce qui rend
 * la lacune visible : un appelant reçoit `plateforme_non_supportee` et le dit à l'utilisateur,
 * au lieu de croire qu'il vient de ranger un secret quelque part.
 *
 * **Le web, lui, a une implémentation** — `stockage-web.ts`, garantie `aucune`, et c'est écrit
 * dans son type. Voir {@link NiveauGarantieStockage}.
 */
export const stockageSecuriseAbsent: StockageSecurise = {
  garantie: 'indisponible',
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
