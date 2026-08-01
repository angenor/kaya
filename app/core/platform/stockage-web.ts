/**
 * Stockage du navigateur — **et ce qu'il ne garantit pas**.
 *
 * # Ce fichier est écrit pour être lu avant d'y ranger quoi que ce soit
 *
 * Le web n'a pas de coffre système. Il n'existe **aucun** emplacement, dans un navigateur, qu'un
 * script de la même origine ne puisse pas lire : ni `localStorage`, ni `sessionStorage`, ni
 * IndexedDB, ni une variable de module. Un cookie `HttpOnly` échapperait au script — mais il est
 * posé par le serveur, et le jeton de rafraîchissement doit être présenté par le client à une
 * opération publique (`session_rafraichir`), pas renvoyé automatiquement à chaque requête.
 *
 * D'où la déclaration `garantie: 'aucune'`, portée par le **type** et non par ce commentaire :
 * {@link NiveauGarantieStockage}. Un appelant qui exige un coffre matériel doit la lire et
 * refuser ; l'appelant de CPT-01 la lit, l'accepte, et **dit pourquoi** — voir
 * `app/core/auth/session.ts`.
 *
 * # Ce qui rend le compromis acceptable, et qui n'est pas ici
 *
 * Trois contreparties, toutes portées par le serveur ou par d'autres portes :
 *
 * 1. **Rotation à chaque usage** — un jeton présenté deux fois révoque **toute la famille**
 *    (contrat, `session_rafraichir`). Une copie volée et rejouée coupe la session de son
 *    propriétaire, ce qui rend le vol visible au lieu de le rendre durable.
 * 2. **Coupure immédiate** — « Appareils connectés » révoque une session, et la liste de
 *    révocation est consultée à chaque requête authentifiée.
 * 3. **Aucune ressource d'hôte externe** (portes P-21 et P-21b) : polices, icônes et scripts sont
 *    embarqués. La surface d'injection d'un script tiers dans l'origine est donc réduite à ce que
 *    le dépôt contient.
 *
 * **Le jeton d'accès, lui, ne passe jamais par ici.** Il vit en mémoire, meurt avec l'onglet, et
 * dure soixante minutes. L'écrire dans `localStorage` doublerait la surface pour rien.
 *
 * # Pourquoi `localStorage` plutôt que `sessionStorage`
 *
 * Le jeton de rafraîchissement dure quatre-vingt-dix jours parce que M. Diarra « vient une fois
 * par mois ». `sessionStorage` disparaît à la fermeture de l'onglet : il faudrait se reconnecter
 * à chaque ouverture du navigateur, ce qui annulerait exactement la décision qui a fixé
 * quatre-vingt-dix jours.
 */

import {
  disponible,
  indisponible,
  type ResultatCapacite,
  type StockageSecurise,
} from './index'

/**
 * Préfixe des clés — **et l'unité de la purge**.
 *
 * `purger()` ne vide pas `localStorage` : l'application n'est pas seule sur son origine en
 * développement, et effacer les clés d'un voisin serait un défaut qu'on mettrait des mois à
 * imputer. Elle retire les seules clés de ce préfixe.
 */
const PREFIXE = 'kaya.'

/** `localStorage` existe-t-il, et répond-il ? */
function magasin(): Storage | null {
  if (typeof localStorage === 'undefined') {
    return null
  }
  try {
    // Safari en navigation privée déclare `localStorage` et lève à l'écriture. Le constater ici
    // évite que chaque appelant ait à rattraper une exception de plateforme.
    const sonde = `${PREFIXE}__sonde`
    localStorage.setItem(sonde, '1')
    localStorage.removeItem(sonde)
    return localStorage
  }
  catch {
    return null
  }
}

/**
 * Stockage web — **garantie `aucune`, et c'est dans le type**.
 *
 * Il n'est pas exporté comme « sécurisé » par confort de nommage : il satisfait l'interface
 * {@link StockageSecurise} parce que c'est la porte unique du principe VII vers la persistance de
 * secrets, et son champ `garantie` dit exactement ce qu'il vaut.
 */
export const stockageWeb: StockageSecurise = {
  garantie: 'aucune',

  async lire(cle: string): Promise<ResultatCapacite<string | null>> {
    const m = magasin()
    if (!m) {
      return indisponible('plateforme_non_supportee')
    }
    return disponible(m.getItem(PREFIXE + cle))
  },

  async ecrire(cle: string, valeur: string): Promise<ResultatCapacite> {
    const m = magasin()
    if (!m) {
      return indisponible('plateforme_non_supportee')
    }
    try {
      m.setItem(PREFIXE + cle, valeur)
      return disponible(undefined)
    }
    catch {
      // Quota dépassé. Le dire plutôt que de laisser l'appelant croire que c'est écrit — il
      // redemanderait le mot de passe au prochain démarrage sans savoir pourquoi.
      return indisponible('materiel_absent')
    }
  },

  async supprimer(cle: string): Promise<ResultatCapacite> {
    const m = magasin()
    if (!m) {
      return indisponible('plateforme_non_supportee')
    }
    m.removeItem(PREFIXE + cle)
    return disponible(undefined)
  },

  async purger(): Promise<ResultatCapacite> {
    const m = magasin()
    if (!m) {
      return indisponible('plateforme_non_supportee')
    }
    const aRetirer: string[] = []
    for (let rang = 0; rang < m.length; rang += 1) {
      const cle = m.key(rang)
      if (cle && cle.startsWith(PREFIXE)) {
        aRetirer.push(cle)
      }
    }
    // Deux temps : retirer pendant l'itération décale les index et en laisserait une sur deux.
    aRetirer.forEach(cle => m.removeItem(cle))
    return disponible(undefined)
  },
}
