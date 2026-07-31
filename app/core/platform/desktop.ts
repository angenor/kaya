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
}
