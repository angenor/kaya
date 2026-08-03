/**
 * **La file survit à l'extinction, et elle est illisible en clair.**
 *
 * # Les deux exigences, et pourquoi la seconde n'est pas de la précaution
 *
 * FR-012 — la file survit au rechargement **et** à l'extinction. C'est le cas fréquent, pas le cas
 * limite : Aminata recharge la page quand l'écran se fige, et le terminal de comptoir s'éteint le
 * soir.
 *
 * FR-013 — la charge est chiffrée dès le premier octet. Le motif n'est pas théorique et il est
 * daté : **l'extraction OCR d'une pièce d'identité est de classe A**, donc éligible à cette file,
 * et elle produit des données d'identité de clients — soumises au registre ARTCI et à une
 * rétention de quatre-vingt-dix jours. Une file en clair dans `localStorage` les y laisserait
 * indéfiniment, lisibles par tout script de l'origine.
 *
 * # Le montage : une clé au coffre, le volume à côté
 *
 * ```text
 * clé AES-GCM 256  →  PlatformAdapter.stockageSecurise    (coffre système ; « aucune » sur web)
 * cryptogramme     →  PlatformAdapter.stockagePersistant  (ordinaire, illisible sans la clé)
 * ```
 *
 * **Le coffre n'est pas un magasin** (research R-06) : Keystore et Keychain servent des secrets
 * courts et peu nombreux, pas une file réécrite à chaque saisie. Y verser le volume échouerait
 * d'abord sur l'Android d'entrée de gamme d'Aminata, c'est-à-dire sur la cible.
 *
 * # AUCUNE dépendance nouvelle
 *
 * **WebCrypto est une API du moteur**, présente sur les quatre cibles — pas une bibliothèque.
 * `docs/versions-gelees.md` est inchangé par ce cycle, et c'est un effet direct de ce choix : une
 * bibliothèque de chiffrement aurait ajouté une dépendance native à vérifier sur deux
 * architectures (le poste est `arm64`, la cible `amd64`) pour remplacer ce que le moteur fait.
 *
 * # ⚠️ Deux limites, écrites plutôt que découvertes
 *
 * - **Sur le web, la garantie du coffre est `aucune`**, et le type le dit. Un script de la même
 *   origine peut lire la clé. Le produit ne prétendra pas le contraire : ce que le chiffrement
 *   achète ici est qu'un accès au **stockage seul** — sauvegarde de navigateur, extension,
 *   inspection du disque — ne rende rien de lisible. La contrepartie du web est portée ailleurs :
 *   purge à la déconnexion, rotation des jetons, coupure depuis « Appareils connectés ».
 * - **`crypto.subtle` exige un contexte sécurisé.** Tauri sert l'application depuis un protocole
 *   personnalisé qui en est un, et P-22 le vérifie sur Chromium et WebKit — mais **le WebKit de
 *   Playwright n'est pas WKWebView**. Le vert dit « tourne sur un moteur WebKit », jamais
 *   « vérifié sur la cible ». Le contrôle réel viendra avec la coquille Tauri.
 *
 * # Ce qui se passe quand le déchiffrement échoue, et pourquoi ce n'est pas une erreur
 *
 * Clé perdue, cryptogramme tronqué, format d'une version antérieure : la file repart **vide**, et
 * le cryptagramme illisible est effacé. C'est le seul comportement défendable — refuser de
 * démarrer bloquerait le terminal sur un état qu'aucun exploitant ne peut réparer, et garder un
 * bloc indéchiffrable ferait échouer chaque enregistrement suivant.
 *
 * Ce que ça coûte est réel et il faut le dire : **les écritures de ce bloc sont perdues**. C'est
 * pourquoi la clé vit au coffre, qui survit à la purge du cache, et non à côté du cryptogramme.
 */

import type { PlatformAdapter } from '~/core/platform'

import type { EntreeFile } from './classes'

/** Clé du cryptogramme dans le stockage ordinaire. */
const CLE_CRYPTOGRAMME = 'sync.file'

/** Clé du secret dans le coffre. */
const CLE_SECRET = 'sync.cle-file'

/** AES-GCM, 256 bits — le choix que WebCrypto sert partout et qui authentifie la charge. */
const ALGORITHME = 'AES-GCM'
const LONGUEUR_CLE = 256

/**
 * Longueur du vecteur d'initialisation, en octets.
 *
 * **12, et pas 16.** C'est la longueur pour laquelle GCM est spécifié : un IV de 96 bits est
 * employé tel quel, tout autre passe par une dérivation qui n'apporte rien et coûte de la
 * compatibilité.
 */
const LONGUEUR_IV = 12

/** Le magasin de la file — ce que `FileLocale` sait faire de sa persistance. */
export interface MagasinFile {
  /** Ce qui était rangé. Tableau **vide** si rien, ou si rien n'est déchiffrable. */
  charger(): Promise<EntreeFile[]>
  /** Remplace le contenu rangé. */
  enregistrer(entrees: readonly EntreeFile[]): Promise<void>
  /** Efface le cryptogramme **et** la clé — la déconnexion, rien de moins. */
  purger(): Promise<void>
}

/** `crypto.subtle` est-il là ? Faux en rendu serveur et hors contexte sécurisé. */
function chiffrementDisponible(): boolean {
  return typeof crypto !== 'undefined' && typeof crypto.subtle !== 'undefined'
}

function versBase64(octets: Uint8Array): string {
  let binaire = ''
  for (const octet of octets) {
    binaire += String.fromCharCode(octet)
  }
  return btoa(binaire)
}

function depuisBase64(texte: string): Uint8Array {
  const binaire = atob(texte)
  const octets = new Uint8Array(binaire.length)
  for (let rang = 0; rang < binaire.length; rang += 1) {
    octets[rang] = binaire.charCodeAt(rang)
  }
  return octets
}

/**
 * La clé de l'appareil — **lue du coffre, ou engendrée et rangée**.
 *
 * Elle est engendrée **sur l'appareil** et n'en sort jamais (principe IX) : aucun secret n'est
 * dans le binaire, aucun n'arrive du serveur. Deux terminaux du même établissement ont donc deux
 * clés distinctes, et c'est correct — la file est locale à un terminal par nature.
 */
async function cleDeLAppareil(adaptateur: PlatformAdapter): Promise<CryptoKey | null> {
  if (!chiffrementDisponible()) {
    return null
  }

  const rangee = await adaptateur.stockageSecurise.lire(CLE_SECRET)
  if (rangee.disponible && rangee.valeur) {
    try {
      return await crypto.subtle.importKey(
        'raw',
        depuisBase64(rangee.valeur) as unknown as BufferSource,
        ALGORITHME,
        false,
        ['encrypt', 'decrypt'],
      )
    }
    catch {
      // Secret illisible — engendrer plutôt que d'échouer. Voir la note de tête.
    }
  }

  const cle = await crypto.subtle.generateKey(
    { name: ALGORITHME, length: LONGUEUR_CLE },
    true,
    ['encrypt', 'decrypt'],
  )
  const brute = new Uint8Array(await crypto.subtle.exportKey('raw', cle))
  await adaptateur.stockageSecurise.ecrire(CLE_SECRET, versBase64(brute))

  // Réimportée **non extractible** : une fois rangée, la clé n'a plus de raison de pouvoir sortir
  // de l'objet `CryptoKey`, et le dire au moteur retire une surface pour rien.
  return crypto.subtle.importKey(
    'raw',
    brute as unknown as BufferSource,
    ALGORITHME,
    false,
    ['encrypt', 'decrypt'],
  )
}

/**
 * Ouvre le magasin de la file pour cet appareil.
 *
 * # Le cas sans chiffrement est un REFUS de persister, jamais un repli en clair
 *
 * Si `crypto.subtle` manque — rendu serveur, contexte non sécurisé —, le magasin rendu est
 * **inerte** : il ne charge rien et n'écrit rien. La file vit alors en mémoire seule.
 *
 * C'est la seule lecture possible de FR-013 : « chiffrée dès le premier octet » n'admet pas de
 * repli. Écrire en clair « parce que c'est mieux que rien » mettrait des données d'identité de
 * clients dans le stockage d'un navigateur, ce qu'aucun gain de confort ne rachète.
 */
export async function ouvrirMagasin(adaptateur: PlatformAdapter): Promise<MagasinFile> {
  const cle = await cleDeLAppareil(adaptateur)

  if (cle === null) {
    return {
      async charger() {
        return []
      },
      async enregistrer() {
        // Inerte, délibérément. Voir la note ci-dessus.
      },
      async purger() {
        await adaptateur.stockagePersistant.supprimer(CLE_CRYPTOGRAMME)
      },
    }
  }

  return {
    async charger(): Promise<EntreeFile[]> {
      const range = await adaptateur.stockagePersistant.lire(CLE_CRYPTOGRAMME)
      if (!range) {
        return []
      }

      try {
        const brut = depuisBase64(range)
        const iv = brut.slice(0, LONGUEUR_IV)
        const charge = brut.slice(LONGUEUR_IV)

        const clair = await crypto.subtle.decrypt(
          { name: ALGORITHME, iv: iv as unknown as BufferSource },
          cle,
          charge as unknown as BufferSource,
        )
        const entrees: unknown = JSON.parse(new TextDecoder().decode(clair))
        return Array.isArray(entrees) ? (entrees as EntreeFile[]) : []
      }
      catch {
        // Indéchiffrable : on efface plutôt que de garder un bloc qui ferait échouer chaque
        // enregistrement suivant. Les écritures de ce bloc sont perdues — c'est écrit en tête.
        await adaptateur.stockagePersistant.supprimer(CLE_CRYPTOGRAMME)
        return []
      }
    },

    async enregistrer(entrees: readonly EntreeFile[]): Promise<void> {
      if (entrees.length === 0) {
        // Une file vide n'a pas de cryptogramme. Garder un bloc qui chiffre `[]` laisserait
        // croire, à l'inspection du stockage, qu'il reste quelque chose à envoyer.
        await adaptateur.stockagePersistant.supprimer(CLE_CRYPTOGRAMME)
        return
      }

      // **Un IV neuf à chaque écriture.** Réutiliser un IV avec la même clé casse GCM — pas
      // « affaiblit » : casse. La file est réécrite à chaque saisie, donc le cas est fréquent.
      const iv = crypto.getRandomValues(new Uint8Array(LONGUEUR_IV))
      const clair = new TextEncoder().encode(JSON.stringify(entrees))

      const chiffre = new Uint8Array(
        await crypto.subtle.encrypt(
          { name: ALGORITHME, iv: iv as unknown as BufferSource },
          cle,
          clair as unknown as BufferSource,
        ),
      )

      const bloc = new Uint8Array(iv.length + chiffre.length)
      bloc.set(iv, 0)
      bloc.set(chiffre, iv.length)

      await adaptateur.stockagePersistant.ecrire(CLE_CRYPTOGRAMME, versBase64(bloc))
    },

    async purger(): Promise<void> {
      await adaptateur.stockagePersistant.supprimer(CLE_CRYPTOGRAMME)
      await adaptateur.stockageSecurise.supprimer(CLE_SECRET)
    },
  }
}

/** Les deux clés employées — pour que les tests inspectent le stockage sans les redéclarer. */
export const CLES_PERSISTANCE = {
  cryptogramme: CLE_CRYPTOGRAMME,
  secret: CLE_SECRET,
} as const
