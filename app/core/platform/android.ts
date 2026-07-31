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

  async notifier(_notification: Notification): Promise<ResultatCapacite> {
    return indisponible('plateforme_non_supportee')
  },

  async geolocaliser(): Promise<ResultatCapacite<Position>> {
    return indisponible('permission_refusee')
  },

  etatReseau(): EtatReseau {
    return etatReseauNavigateur()
  },
}
