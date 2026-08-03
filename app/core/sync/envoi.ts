/**
 * **Les quatre déclencheurs d'envoi — et aucune minuterie de scrutation.**
 *
 * # La décision qui structure ce fichier, et son motif est la batterie
 *
 * Une minuterie qui réveille la radio toutes les trente secondes coûte, sur un Android d'entrée de
 * gamme, la batterie d'un service entier. C'est la cible d'Aminata. Et elle le coûte pour un gain
 * que le retour au premier plan couvre déjà : le cadrage est explicite, **la file est conçue pour
 * se vider au retour au premier plan, sur toutes les plateformes**, et tout le reste est
 * optimisation.
 *
 * | Déclencheur | Rôle |
 * |---|---|
 * | **Retour au premier plan** | Le déclencheur par défaut. **Doit suffire seul**, partout |
 * | Passage de l'état réseau à `connecte` | Opportuniste — le réseau vient de revenir |
 * | Après une écriture réussie | La file profite d'un réseau qu'on vient de constater bon |
 * | Réessai à **intervalle croissant plafonné** | Après échec, jusqu'au prochain déclencheur naturel |
 *
 * Le quatrième n'est pas une scrutation : il ne s'arme **qu'après un échec**, et il s'éteint dès
 * qu'un envoi réussit ou que la file se vide. Une file vide n'arme aucune minuterie — c'est la
 * différence entre « réessayer » et « surveiller ».
 *
 * # L'ordre n'est pas ici, et c'est le point
 *
 * Ce fichier décide **quand** envoyer. Il ne décide pas **comment** : `viderFile` porte l'ordre
 * rafraîchir-avant-vider, et reste le seul chemin de sortie de la file. Les deux responsabilités
 * séparées est ce qui empêche un déclencheur nouveau de réintroduire un envoi sans
 * rafraîchissement — le défaut qui perd un service entier après une coupure plus longue que le
 * jeton.
 */

import { adaptateurCourant } from '~/core/platform/courant'
import type { Desabonnement } from '~/core/platform'

import { fileCourante } from './attente'
import { signalerChangement } from './etat'
import { viderFile, type Envoyeur, type ResultatVidage } from './vidage'

/**
 * Le premier délai de réessai, en millisecondes.
 *
 * **Deux secondes, et non deux cents millisecondes.** Un réessai immédiat après un échec réseau
 * retombe presque toujours sur le même échec, et consomme un aller-retour pour rien. Deux secondes
 * laissent le temps à une coupure brève de se résorber.
 */
const DELAI_INITIAL_MS = 2_000

/**
 * Le plafond, en millisecondes.
 *
 * **Trente secondes.** Au-delà, le réessai n'apporte plus rien : le retour au premier plan et le
 * signal `online` couvrent tous les cas où le réseau revient vraiment, et allonger encore
 * reviendrait à garder une minuterie armée pour rien.
 */
const DELAI_PLAFOND_MS = 30_000

/** Ce que le module tient entre deux appels — sans état global exporté. */
interface Etat {
  /** Le minuteur de réessai armé, s'il y en a un. */
  minuteur: ReturnType<typeof setTimeout> | null
  /** Le délai du prochain réessai. Double à chaque échec, jusqu'au plafond. */
  delaiMs: number
  /** Un envoi est-il en cours ? Deux envois simultanés enverraient deux fois la même entrée. */
  enCours: boolean
  /** Les désabonnements posés par {@link brancherEnvoi}. */
  desabonnements: Desabonnement[]
}

const etat: Etat = {
  minuteur: null,
  delaiMs: DELAI_INITIAL_MS,
  enCours: false,
  desabonnements: [],
}

function annulerMinuteur(): void {
  if (etat.minuteur !== null) {
    clearTimeout(etat.minuteur)
    etat.minuteur = null
  }
}

/**
 * Arme le réessai — **seulement après un échec, et seulement si la file n'est pas vide**.
 *
 * Une file vide n'arme rien : c'est ce qui distingue ce mécanisme d'une scrutation.
 */
function armerReessai(envoyer: Envoyeur, baseUrl: string): void {
  const file = fileCourante()
  if (!file || file.enAttente === 0) {
    return
  }
  annulerMinuteur()
  etat.minuteur = setTimeout(() => {
    etat.minuteur = null
    void declencherEnvoi(envoyer, baseUrl)
  }, etat.delaiMs)
  etat.delaiMs = Math.min(etat.delaiMs * 2, DELAI_PLAFOND_MS)
}

/**
 * Tente un envoi. **Idempotente vis-à-vis d'elle-même** : deux appels concurrents n'en font qu'un.
 *
 * Le verrou n'est pas de la prudence : les quatre déclencheurs peuvent tomber ensemble — le réseau
 * revient au moment où l'utilisateur repasse au premier plan, ce qui est le cas *normal*. Sans
 * verrou, la même entrée partirait deux fois. Le serveur la dédupliquerait (le rejeu est
 * inoffensif, c'est tout l'objet de l'UUID v7 client), mais le terminal aurait dépensé deux fois
 * le forfait de quelqu'un.
 */
export async function declencherEnvoi(
  envoyer: Envoyeur,
  baseUrl: string,
): Promise<ResultatVidage | null> {
  const file = fileCourante()
  if (!file || etat.enCours) {
    return null
  }

  etat.enCours = true
  try {
    const resultat = await viderFile(file, baseUrl, adaptateurCourant().etatReseau(), envoyer)

    switch (resultat.issue) {
      case 'videe':
      case 'rien_a_faire':
        // Le réseau va bien : le compteur de réessai repart de zéro, et rien ne reste armé.
        annulerMinuteur()
        etat.delaiMs = DELAI_INITIAL_MS
        break
      case 'partielle':
        // Une partie est passée : le réseau existe. On réessaie, mais depuis le délai initial —
        // doubler après un succès partiel punirait un réseau qui fonctionne.
        etat.delaiMs = DELAI_INITIAL_MS
        armerReessai(envoyer, baseUrl)
        break
      case 'hors_ligne':
        // **Aucun réessai armé.** Le retour du réseau a son propre signal ; battre la mesure
        // pendant une coupure de quatre-vingt-dix minutes ne ferait que vider la batterie.
        annulerMinuteur()
        break
      case 'reconnexion_requise':
        // La session est finie et la file est **intacte**. Réessayer sans reconnexion produirait
        // la même réponse indéfiniment ; c'est à l'utilisateur de se reconnecter.
        annulerMinuteur()
        break
    }

    signalerChangement()
    return resultat
  }
  finally {
    etat.enCours = false
  }
}

/**
 * Branche les trois déclencheurs permanents. **Appelée une fois, au démarrage.**
 *
 * Rend le désabonnement — jamais `void`. Un test qui brancherait sans pouvoir débrancher
 * contaminerait les suivants, et une coquille qui ne débrancherait pas ferait fuir la mémoire à
 * chaque reprise de session.
 */
export function brancherEnvoi(envoyer: Envoyeur, baseUrl: string): Desabonnement {
  debrancherEnvoi()

  const tenter = (): void => {
    void declencherEnvoi(envoyer, baseUrl)
  }

  // 1 · Le retour au premier plan — **le déclencheur qui doit suffire seul**. Il passe par
  //     l'adaptateur, jamais par un écouteur posé ici : sur desktop, Tauri fournit un signal de
  //     fenêtre plus fin que celui du navigateur, et c'est la raison d'être de l'abstraction.
  etat.desabonnements.push(adaptateurCourant().surRetourPremierPlan(tenter))

  // 2 · Le retour du réseau.
  if (typeof window !== 'undefined') {
    window.addEventListener('online', tenter)
    etat.desabonnements.push(() => window.removeEventListener('online', tenter))
  }

  return debrancherEnvoi
}

/** Retire les déclencheurs et désarme le réessai. */
export function debrancherEnvoi(): void {
  annulerMinuteur()
  etat.delaiMs = DELAI_INITIAL_MS
  for (const desabonner of etat.desabonnements) {
    desabonner()
  }
  etat.desabonnements = []
}

/**
 * Le troisième déclencheur : **après une écriture réussie**.
 *
 * Appelé par l'écran qui vient d'écrire en ligne. La file profite d'un réseau qu'on vient de
 * constater bon — c'est le moment le moins cher pour vider, puisqu'aucun aller-retour de sonde
 * n'est nécessaire pour savoir que ça passe.
 */
export function apresEcritureReussie(envoyer: Envoyeur, baseUrl: string): void {
  void declencherEnvoi(envoyer, baseUrl)
}

/** Le délai de réessai en vigueur — pour que `S1` puisse dire quand la prochaine tentative aura lieu. */
export function delaiReessaiMs(): number {
  return etat.delaiMs
}
