/**
 * **L'heure de cet appareil est fausse — dit à la personne qui le tient.**
 *
 * # Ce que le serveur constate, et ce que le terminal doit apprendre
 *
 * Le serveur écrit une entrée `derive_horloge_constatee` au registre des actions : c'est ce qui
 * permettra à M. Koffi de retrouver, après coup, **quel terminal** déviait pendant le service.
 *
 * Ce n'est pas la même information que celle qui manque à Aminata. Elle, il faut lui dire
 * **maintenant**, sur son écran, que l'heure de son appareil est à régler — sinon elle continuera
 * de lire des horaires faux toute la soirée.
 *
 * # Aucun endpoint nouveau, et c'est le point
 *
 * Le contrat n'a pas besoin d'une opération « quelle heure est-il ? » : **chaque réponse de
 * création porte déjà l'horodatage d'autorité de la ligne créée**. Le client compare sa propre
 * horloge à celui-là, et il apprend ce qui l'intéresse — que **sa** montre est fausse.
 *
 * C'est aussi la seule mesure honnête : un endpoint d'heure serveur mesurerait l'aller-retour
 * réseau autant que l'écart d'horloge, et sur une 3G d'Abengourou l'aller-retour est la plus
 * grande des deux valeurs.
 *
 * # ⚠️ Cette comparaison est une EXEMPTION nommée de la porte P-23
 *
 * La porte refuse tout calcul appuyé sur l'horloge du terminal. Ici, l'horloge du terminal **est
 * l'objet de la mesure** — c'est l'exemption « détection de dérive d'horloge », et la troisième,
 * « rendu de l'instant tel que le terminal l'a perçu », couvre l'affichage qui en découle.
 *
 * Ce que ce module ne fait **jamais**, et ce qui distingue l'exemption de la faute : il ne calcule
 * aucune durée métier, aucun montant, aucune échéance. La phrase de rassurance le dit à
 * l'utilisateur en toutes lettres, et elle est obligatoire.
 *
 * # Le vocabulaire — le lexique fait foi, et il est catégorique
 *
 * Le mot « dérive » **n'atteint jamais l'écran**. Aucune valeur technique non plus : ni secondes,
 * ni horodatage, ni seuil. L'utilisateur lit une phrase, dans le sens qui le concerne, suivie de
 * celle qui le rassure — et **la seconde est obligatoire** : un avertissement qui inquiète sur ce
 * qui va bien est pire que pas d'avertissement.
 */

import { ref, readonly, type Ref } from 'vue'

/**
 * Le seuil d'affichage, en millisecondes.
 *
 * **Cinq minutes**, la même valeur que le défaut du catalogue — et ce n'est pas une seconde source
 * de vérité : le serveur décide de ce qu'il **consigne**, ce module décide de ce qu'il **affiche**.
 * Les deux réponses ne sont pas au même public et n'ont pas à venir du même endroit.
 *
 * Le client ne peut pas lire la configuration d'établissement avant d'avoir une session, et il doit
 * pouvoir avertir dès la première réponse reçue.
 */
const SEUIL_AFFICHAGE_MS = 5 * 60 * 1000

/** Ce que l'écran affiche — ou `null` si l'horloge est bonne. */
export interface AvertissementHorloge {
  /** Clé i18n de la phrase, selon le sens. */
  readonly cle: 'sync.horloge.retard' | 'sync.horloge.avance'
  /** L'écart en minutes, arrondi — la seule valeur que l'utilisateur voit. */
  readonly minutes: number
}

const avertissement = ref<AvertissementHorloge | null>(null)

/**
 * Compare l'horloge locale à l'horodatage d'autorité d'une réponse.
 *
 * Appelée par tout module qui reçoit une réponse de création. **Elle ne rend rien** : l'écart va
 * dans l'état réactif, et l'écran l'affiche s'il y a lieu. Rendre une valeur inviterait un appelant
 * à en faire quelque chose — et il n'y a rien à en faire d'autre que de le dire.
 *
 * @param horodatageAutorite `cree_le` de la ligne créée, en RFC 3339. Le serveur fait foi.
 */
export function comparerAHorodatageAutorite(horodatageAutorite: string): void {
  const autorite = Date.parse(horodatageAutorite)
  if (Number.isNaN(autorite)) {
    return
  }

  // **La valeur absolue** : une horloge en avance est aussi fausse qu'une horloge en retard, et
  // l'utilisateur doit savoir dans quel sens régler son appareil. Comparer sur un écart signé
  // laisserait la moitié des cas sans phrase — c'est le défaut que le lexique 1.5.1 a corrigé.
  const ecartMs = Date.now() - autorite

  if (Math.abs(ecartMs) <= SEUIL_AFFICHAGE_MS) {
    // L'horloge est revenue à l'heure — l'avertissement disparaît. Le laisser afficher après une
    // correction apprendrait à l'ignorer.
    avertissement.value = null
    return
  }

  avertissement.value = {
    cle: ecartMs > 0 ? 'sync.horloge.avance' : 'sync.horloge.retard',
    minutes: Math.round(Math.abs(ecartMs) / 60_000),
  }
}

/**
 * L'avertissement courant, réactif.
 *
 * **Il s'accompagne TOUJOURS de `sync.horloge.rassurance`** — « Les durées et les montants restent
 * calculés sur l'heure du serveur. » L'écran qui l'affiche sans elle apprendrait à l'exploitant que
 * ses passages sont mal facturés, ce qui est faux : l'horodatage d'autorité les protège
 * (principe IV, porte P-23). Le lexique 1.5.1 rend cette seconde phrase obligatoire.
 */
export function avertissementHorloge(): Readonly<Ref<AvertissementHorloge | null>> {
  return readonly(avertissement) as Readonly<Ref<AvertissementHorloge | null>>
}

/** Oublie l'avertissement — à la déconnexion, et dans les tests. */
export function oublierAvertissementHorloge(): void {
  avertissement.value = null
}
