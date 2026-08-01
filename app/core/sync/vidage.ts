/**
 * Le vidage de la file — **rafraîchir d'abord, vider ensuite. Jamais l'inverse.**
 *
 * # Le défaut que ce fichier existe pour empêcher
 *
 * Le jeton d'accès dure soixante minutes. Aminata prend des commandes pendant une coupure de
 * quatre-vingt-dix. Au retour du réseau, une file qui enverrait avant de rafraîchir partirait avec
 * un jeton expiré : chaque élément reviendrait en `401`, et le service entier serait perdu.
 *
 * **Ce défaut ne se voit pas en développement.** La coupure y dure trente secondes, le jeton est
 * encore valide au retour, et les deux ordres passent. Il se manifeste à Abengourou, un soir de
 * service. C'est la raison pour laquelle l'ordre est porté par **une seule fonction**, que la file
 * n'a aucun autre chemin de sortie, et que `app/tests/file-jeton-expire.spec.ts` échoue si l'ordre
 * s'inverse **y compris quand les deux réussissent** (research R-18).
 *
 * # Trois règles, et ce qu'elles coûteraient si on les relâchait
 *
 * 1. **La file ne stocke aucun jeton.** {@link EntreeFile} n'a pas de champ où en mettre un, et
 *    c'est délibéré : un jeton mis en file serait périmé au retour, et le ranger prolongerait la
 *    durée de vie d'un secret sur un terminal qu'on peut perdre.
 * 2. **L'échec du rafraîchissement NE VIDE PAS la file.** Elle reste intacte, et l'utilisateur
 *    apprend qu'il doit se reconnecter. Vider sur un `401` détruirait exactement les écritures
 *    qu'on cherche à sauver.
 * 3. **Cette règle ne vaut que pour la classe A.** Les classes B, C et D n'entrent jamais en file
 *    (principe VI) : leur refus est immédiat et explicite, avant toute saisie. Le type de
 *    {@link FileLocale.enfiler} le tient à la compilation.
 *
 * # Pourquoi on rafraîchit TOUJOURS, sans consulter l'expiration locale
 *
 * Il serait tentant de ne rafraîchir que si `accesExpire()` le dit. Ce serait faire décider une
 * **horloge de terminal** — celle dont le principe IV dit qu'aucun calcul de durée ne s'y appuie.
 * Un téléphone mal réglé de deux heures conclurait « encore valide », enverrait, et perdrait la
 * file. Le rafraîchissement coûte un aller-retour ; la faute coûte un service.
 *
 * La rotation qui en découle est le comportement normal du serveur, pas un effet de bord : chaque
 * usage consomme le jeton présenté et en délivre un autre.
 */

import { rafraichirSession, type ResultatConnexion } from '~/core/auth'
import { contexteAppel, type ContexteAppel } from '~/core/auth/session'

import type { EntreeFile } from './classes'
import type { EtatReseau } from './index'
import type { FileLocale } from './index'

/** Ce qu'un vidage produit. */
export type ResultatVidage =
  /** Rien à envoyer — la file était vide. Aucun rafraîchissement n'a été demandé. */
  | { issue: 'rien_a_faire' }
  /** Le réseau manque. La file est **intacte**. */
  | { issue: 'hors_ligne', restantes: number }
  /**
   * Le rafraîchissement a échoué : la session est finie.
   *
   * **La file est intacte** — c'est le point de la règle 2. L'interface demande une reconnexion,
   * et le vidage reprendra après.
   */
  | { issue: 'reconnexion_requise', restantes: number, cle: string }
  /** Tout est parti. */
  | { issue: 'videe', envoyees: number }
  /** Une partie est partie ; le reste attend le prochain passage. */
  | { issue: 'partielle', envoyees: number, restantes: number }

/**
 * Envoie une entrée. Rendu `true` si elle est acquittée et peut quitter la file.
 *
 * Passé en paramètre plutôt qu'appelé ici : chaque type d'opération de classe A a son propre point
 * d'entrée d'API, et cette fonction n'a pas à les connaître. Elle ne connaît que l'**ordre**.
 */
export type Envoyeur = (entree: EntreeFile, contexte: ContexteAppel) => Promise<boolean>

/**
 * Vide la file — **et c'est la seule façon d'en sortir quelque chose**.
 *
 * @param reseau État réseau au moment du geste, lu depuis `PlatformAdapter`.
 */
export async function viderFile(
  file: FileLocale,
  baseUrl: string,
  reseau: EtatReseau,
  envoyer: Envoyeur,
): Promise<ResultatVidage> {
  if (file.enAttente === 0) {
    // Rafraîchir pour n'envoyer rien consommerait un jeton et ferait tourner la famille à chaque
    // hoquet du réseau, sans rien gagner.
    return { issue: 'rien_a_faire' }
  }

  if (reseau !== 'connecte') {
    return { issue: 'hors_ligne', restantes: file.enAttente }
  }

  // ─── 1. RAFRAÎCHIR ────────────────────────────────────────────────────────────────────────
  // Cette ligne vient AVANT la boucle d'envoi, et rien ne doit s'insérer entre les deux.
  const rafraichissement: ResultatConnexion = await rafraichirSession(baseUrl)

  if (rafraichissement.issue === 'refus') {
    // La file reste INTACTE. C'est la règle 2, et c'est ce qui distingue « on n'a pas pu envoyer »
    // de « on a perdu vos commandes ».
    return { issue: 'reconnexion_requise', restantes: file.enAttente, cle: rafraichissement.cle }
  }

  const contexte = contexteAppel(baseUrl)
  if (!contexte) {
    // Le rafraîchissement a réussi sans poser de session : cet état n'existe pas. Le traiter
    // comme une reconnexion requise plutôt que d'envoyer sans jeton.
    return { issue: 'reconnexion_requise', restantes: file.enAttente, cle: 'connexion.refus.session_expiree' }
  }

  // ─── 2. VIDER ─────────────────────────────────────────────────────────────────────────────
  let envoyees = 0
  // Copie de la liste : `retirer` modifie la file pendant l'itération.
  for (const entree of [...file.lister()]) {
    const acquittee = await envoyer(entree, contexte)
    if (!acquittee) {
      // On s'arrête au premier échec plutôt que de continuer. L'ordre d'arrivée est indifférent
      // pour la classe A (commutative), mais insister sur un réseau qui vient de refuser dépense
      // la batterie et le forfait de quelqu'un pour rien.
      break
    }
    file.retirer(entree.id)
    envoyees += 1
  }

  return file.enAttente === 0
    ? { issue: 'videe', envoyees }
    : { issue: 'partielle', envoyees, restantes: file.enAttente }
}
