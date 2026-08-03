/**
 * **Ce que le réseau fait vraiment** — la source du troisième état.
 *
 * # La ligne que ce fichier honore
 *
 * `reseau.ts` porte, en commentaire de tête depuis le cycle 001 :
 *
 * > « D'où le troisième état, `degrade`, que **le cycle SYN alimentera depuis les échecs réels de
 * > requête** — il n'est produit par personne aujourd'hui, et c'est écrit plutôt que supposé. »
 *
 * C'est fait ici. `navigator.onLine` dit qu'une **interface réseau est active**, pas que le
 * serveur répond. À Abengourou, une 3G qui affiche « en ligne » sans porter la moindre requête est
 * le cas courant, pas l'exception : sans troisième état, le témoin mentirait exactement au moment
 * où il compte.
 *
 * # Ce que l'observateur retient, et ce qu'il ne retient pas
 *
 * **Le dernier aller-retour, et lui seul** — son issue et sa durée. Pas un historique, pas une
 * moyenne glissante, pas un compteur d'échecs.
 *
 * C'est délibéré. Une moyenne sur dix appels mettrait dix appels à redescendre après le retour du
 * réseau : le témoin resterait « connexion faible » alors que tout va bien, et l'utilisateur
 * apprendrait à ne plus le croire. Le témoin le plus important du produit doit dire l'état
 * **maintenant**, et se corriger au premier appel qui réussit.
 *
 * # Le seuil est un PARAMÈTRE, jamais une constante
 *
 * `sync.latence_degradee_seuil_ms` (défaut 3 000), migration `0028`. Principe I(c) : une valeur
 * métier en dur imposerait une livraison pour l'ajuster, alors que la qualité du réseau varie
 * d'un établissement à l'autre — et c'est précisément la variable qu'un exploitant veut régler.
 *
 * # Ce fichier ne fait AUCUN appel
 *
 * Il observe ceux que d'autres font. C'est ce qui lui permet de vivre dans `core/platform/` sans
 * connaître ni le client typé, ni les routes, ni l'authentification — et ce qui empêche l'état
 * réseau de devenir une seconde couche d'appel.
 */

/** Ce qu'un aller-retour a produit. */
export interface IssueAppel {
  /** L'appel a-t-il abouti — quel que soit le code de réponse ? Un `422` a abouti. */
  readonly abouti: boolean
  /** Sa durée, en millisecondes. */
  readonly dureeMs: number
}

/**
 * Le dernier aller-retour observé. `null` tant qu'aucun appel n'a été fait.
 *
 * `null` et « tout va bien » sont deux états distincts : au démarrage, rien ne permet de dire que
 * le réseau est mauvais, et l'annoncer dégradé d'emblée serait un mensonge dans l'autre sens.
 */
let dernier: IssueAppel | null = null

/**
 * Le seuil de latence en vigueur, en millisecondes.
 *
 * **Le défaut du catalogue est répété ici**, et il faut dire pourquoi ce n'est pas une seconde
 * source de vérité : la valeur réelle vient de la configuration d'établissement, qui arrive après
 * la connexion. Avant elle, l'observateur doit bien comparer à quelque chose. Ce nombre est donc
 * une **valeur d'attente**, remplacée par {@link poserSeuilLatence} dès que la configuration est
 * lue — et le catalogue reste la source, comme le principe I(c) l'exige.
 */
const SEUIL_ATTENTE_MS = 3_000

let seuilMs = SEUIL_ATTENTE_MS

/**
 * Pose le seuil lu de la configuration d'établissement.
 *
 * Appelée par le plugin de synchronisation, jamais par un composant : un écran qui poserait un
 * seuil ferait dépendre l'état réseau de la page ouverte.
 */
export function poserSeuilLatence(millisecondes: number): void {
  if (Number.isFinite(millisecondes) && millisecondes > 0) {
    seuilMs = millisecondes
  }
}

/** Le seuil en vigueur — pour que l'écran `S1` puisse dire sur quoi il se fonde. */
export function seuilLatenceMs(): number {
  return seuilMs
}

/**
 * Enregistre l'issue d'un aller-retour.
 *
 * Appelée par la couche qui fait l'appel, à chaque fois, **succès comme échec**. N'enregistrer
 * que les échecs laisserait l'état `degrade` collé jusqu'au prochain échec.
 */
export function observerAppel(issue: IssueAppel): void {
  dernier = issue
}

/**
 * Le dernier appel dit-il que le réseau est mauvais ?
 *
 * Deux cas, et ils ne sont pas redondants :
 *
 * - **l'appel n'a pas abouti** — le réseau est là au sens de la plateforme, et pourtant rien ne
 *   passe. C'est le cas d'Abengourou ;
 * - **l'appel a abouti mais lentement** — au-delà du seuil, l'exploitant a le temps de croire que
 *   l'application est bloquée. Le lui dire vaut mieux que de le laisser cliquer deux fois.
 */
export function reseauMauvais(): boolean {
  if (dernier === null) {
    return false
  }
  return !dernier.abouti || dernier.dureeMs > seuilMs
}

/**
 * Oublie ce qui a été observé — **pour les tests, et pour la déconnexion**.
 *
 * À la déconnexion, l'état du réseau de la session précédente n'apprend rien à la suivante : le
 * conserver ferait démarrer Yao sur le mauvais réseau d'Aminata.
 */
export function oublierObservations(): void {
  dernier = null
  seuilMs = SEUIL_ATTENTE_MS
}
