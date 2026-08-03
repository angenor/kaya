/**
 * Adaptateur **desktop** — Tauri sur poste fixe ou portable.
 *
 * **Coquille conforme du cycle 001.** Chaque capacité qu'elle ne sait pas encore fournir renvoie
 * `{ disponible: false }` avec sa raison — jamais un `throw`, jamais un silence. L'appelant est
 * donc déjà obligé de traiter le cas, avant même que l'implémentation existe : c'est ce qui
 * évitera de rouvrir chaque composant métier le jour où elle arrivera.
 */

import {
  etatReseauNavigateur,
  indisponible,
  stockageSecuriseAbsent,
  type ChampsPieceIdentite,
  stockagePersistantMoteur,
  type Desabonnement,
  type DocumentImprimable,
  type EtatReseau,
  type Notification,
  type PlatformAdapter,
  type Position,
  type ResultatCapacite,
} from './index'

export const adaptateurDesktop: PlatformAdapter = {
  nom: 'desktop',
  async imprimer(_doc: DocumentImprimable): Promise<ResultatCapacite> {
    // Impression thermique par le pont Tauri — cycle IMP. Tant qu'elle n'existe pas, le dire.
    return indisponible('plateforme_non_supportee')
  },

  async scanner(): Promise<ResultatCapacite<string>> {
    // Un poste desktop a rarement une caméra utilisable pour un QR ; le lecteur physique passera
    // par le pont Tauri.
    return indisponible('materiel_absent')
  },

  async ocrPieceIdentite(_image: Blob): Promise<ResultatCapacite<ChampsPieceIdentite>> {
    return indisponible('plateforme_non_supportee')
  },

  stockageSecurise: stockageSecuriseAbsent,

  // Le VOLUME de la file, chiffré. Il n'attend PAS la coquille Tauri : le moteur de rendu
  // fournit déjà un stockage persistant, et la file doit survivre au redémarrage dès
  // aujourd'hui.
  stockagePersistant: stockagePersistantMoteur,

  async notifier(_notification: Notification): Promise<ResultatCapacite> {
    return indisponible('plateforme_non_supportee')
  },

  async geolocaliser(): Promise<ResultatCapacite<Position>> {
    // Le géorepérage est SOUPLE et n'est JAMAIS bloquant sur une action critique (principe IX).
    // Son absence sur desktop est donc sans conséquence — c'est bien pour cela qu'il ne doit
    // jamais conditionner un encaissement.
    return indisponible('plateforme_non_supportee')
  },

  etatReseau(): EtatReseau {
    return etatReseauNavigateur()
  },

  /**
   * **Le focus de fenêtre de Tauri**, dès que la coquille existe.
   *
   * C'est le cas qui justifie l'abstraction : sur desktop, le signal utile n'est pas celui du
   * navigateur mais l'événement de fenêtre de Tauri, plus fin — il distingue une fenêtre revenue
   * au premier plan d'un simple retour de focus dans la page.
   *
   * **Provisoire nommé**, de même nature que la sélection d'adaptateur de `courant.ts` : la
   * coquille Tauri n'est pas construite. Les signaux du navigateur sont employés en attendant —
   * ils fonctionnent, et le jour où la coquille arrive, seul ce corps change.
   */
  surRetourPremierPlan(rappel: () => void): Desabonnement {
    if (typeof document === 'undefined' || typeof window === 'undefined') {
      return () => {}
    }
    const surVisibilite = (): void => {
      if (document.visibilityState === 'visible') {
        rappel()
      }
    }
    document.addEventListener('visibilitychange', surVisibilite)
    window.addEventListener('focus', rappel)
    return () => {
      document.removeEventListener('visibilitychange', surVisibilite)
      window.removeEventListener('focus', rappel)
    }
  },
}
