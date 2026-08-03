/**
 * Adaptateur **Android** — Tauri v2 mobile.
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

export const adaptateurAndroid: PlatformAdapter = {
  nom: 'android',
  async imprimer(_doc: DocumentImprimable): Promise<ResultatCapacite> {
    return indisponible('plateforme_non_supportee')
  },

  async scanner(): Promise<ResultatCapacite<string>> {
    return indisponible('plateforme_non_supportee')
  },

  async ocrPieceIdentite(_image: Blob): Promise<ResultatCapacite<ChampsPieceIdentite>> {
    return indisponible('plateforme_non_supportee')
  },

  // Keystore Android — cycle CPT-05, enrôlement d'appareil par paire de clés.
  // **Le verrouillage par adresse MAC n'est jamais implémenté** (principe IX) : Android 10 et
  // suivants randomisent la MAC par réseau et n'exposent pas la MAC matérielle.
  stockageSecurise: stockageSecuriseAbsent,

  // Le VOLUME de la file, chiffré. Il n'attend PAS la coquille Tauri : le moteur de rendu
  // fournit déjà un stockage persistant, et la file doit survivre au redémarrage dès
  // aujourd'hui.
  stockagePersistant: stockagePersistantMoteur,

  async notifier(_notification: Notification): Promise<ResultatCapacite> {
    return indisponible('plateforme_non_supportee')
  },

  async geolocaliser(): Promise<ResultatCapacite<Position>> {
    return indisponible('permission_refusee')
  },

  etatReseau(): EtatReseau {
    return etatReseauNavigateur()
  },

  /**
   * **La reprise d'activité**, dès que la coquille mobile existe.
   *
   * `WorkManager` **n'est pas ici** : il est MOB-06, une optimisation. La file est conçue pour se
   * vider au retour au premier plan, et le produit doit être complet sans tâche de fond — c'est
   * la cible d'Aminata, un Android d'entrée de gamme dont la batterie doit tenir un service.
   *
   * **Provisoire nommé** : la coquille Tauri mobile n'est pas construite, les signaux du moteur de
   * rendu font le travail en attendant.
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
