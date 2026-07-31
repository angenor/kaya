/**
 * Adaptateur **iOS** — Tauri v2 mobile.
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
  type DocumentImprimable,
  type EtatReseau,
  type Notification,
  type PlatformAdapter,
  type Position,
  type ResultatCapacite,
} from './index'

export const adaptateurIos: PlatformAdapter = {
  nom: 'ios',
  async imprimer(_doc: DocumentImprimable): Promise<ResultatCapacite> {
    return indisponible('plateforme_non_supportee')
  },

  async scanner(): Promise<ResultatCapacite<string>> {
    return indisponible('plateforme_non_supportee')
  },

  async ocrPieceIdentite(_image: Blob): Promise<ResultatCapacite<ChampsPieceIdentite>> {
    return indisponible('plateforme_non_supportee')
  },

  // Keychain — cycle CPT-05.
  stockageSecurise: stockageSecuriseAbsent,

  async notifier(_notification: Notification): Promise<ResultatCapacite> {
    return indisponible('plateforme_non_supportee')
  },

  async geolocaliser(): Promise<ResultatCapacite<Position>> {
    return indisponible('permission_refusee')
  },

  etatReseau(): EtatReseau {
    // **iOS n'a pas de synchronisation en arrière-plan.** La file se vide au retour au premier
    // plan, sur toutes les plateformes (principe VI) — c'est iOS qui impose cette règle aux
    // autres, et non l'inverse.
    return etatReseauNavigateur()
  },
}
