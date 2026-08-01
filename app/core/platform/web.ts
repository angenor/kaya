/**
 * Adaptateur **web** — navigateur, sans Tauri.
 *
 * **Coquille conforme du cycle 001.** Chaque capacité qu'elle ne sait pas encore fournir renvoie
 * `{ disponible: false }` avec sa raison — jamais un `throw`, jamais un silence. L'appelant est
 * donc déjà obligé de traiter le cas, avant même que l'implémentation existe : c'est ce qui
 * évitera de rouvrir chaque composant métier le jour où elle arrivera.
 */

import { stockageWeb } from './stockage-web'
import {
  etatReseauNavigateur,
  indisponible,
  type ChampsPieceIdentite,
  type DocumentImprimable,
  type EtatReseau,
  type Notification,
  type PlatformAdapter,
  type Position,
  type ResultatCapacite,
} from './index'

export const adaptateurWeb: PlatformAdapter = {
  nom: 'web',
  async imprimer(_doc: DocumentImprimable): Promise<ResultatCapacite> {
    // Un navigateur n'imprime pas en thermique. Ce n'est pas une lacune à combler : c'est une
    // capacité que la plateforme n'a pas, et l'utilisateur doit l'apprendre tout de suite.
    return indisponible('plateforme_non_supportee')
  },

  async scanner(): Promise<ResultatCapacite<string>> {
    return indisponible('plateforme_non_supportee')
  },

  async ocrPieceIdentite(_image: Blob): Promise<ResultatCapacite<ChampsPieceIdentite>> {
    // L'OCR de pièce d'identité se fait côté serveur (données personnelles, registre ARTCI).
    return indisponible('reseau_requis')
  },

  // `localStorage` n'est PAS un stockage sécurisé, et l'implémentation ne prétend pas le
  // contraire : elle déclare `garantie: 'aucune'` **dans son type**, ce qui oblige l'appelant qui
  // exige un coffre matériel à s'en apercevoir. Le détail — ce que le compromis coûte et ce qui le
  // rachète — est en tête de `stockage-web.ts`.
  //
  // Refuser tout stockage était le comportement du cycle 001, et il ne tient plus : CPT-01 doit
  // ranger un jeton de rafraîchissement de quatre-vingt-dix jours, et la seule alternative serait
  // de redemander le mot de passe à chaque ouverture d'onglet.
  stockageSecurise: stockageWeb,

  async notifier(_notification: Notification): Promise<ResultatCapacite> {
    return indisponible('permission_refusee')
  },

  async geolocaliser(): Promise<ResultatCapacite<Position>> {
    return indisponible('permission_refusee')
  },

  etatReseau(): EtatReseau {
    return etatReseauNavigateur()
  },
}
