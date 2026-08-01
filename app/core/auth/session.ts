/**
 * L'état de session — **deux jetons, deux emplacements, et ce n'est pas un détail**.
 *
 * | Jeton | Durée | Où il vit | Pourquoi là |
 * |---|---|---|---|
 * | **Accès** | 60 min | **En mémoire seulement** | Il meurt avec l'onglet. L'écrire quelque part doublerait la surface d'attaque pour gagner une heure |
 * | **Rafraîchissement** | 90 jours | `PlatformAdapter.stockageSecurise` | Il doit survivre à un redémarrage : M. Diarra « vient une fois par mois » |
 *
 * # Le front ne décode JAMAIS le jeton
 *
 * Les permissions, le tenant, le compte et l'établissement actif viennent **du corps de la
 * réponse**, en clair (research R-06). Décoder le JWT donnerait une seconde source pour la même
 * information, et le jour où les deux divergeraient, il faudrait décider laquelle ment. Il n'y a
 * donc, dans tout `app/`, aucune ligne qui coupe une chaîne sur un point ni qui appelle `atob`.
 *
 * # Le stockage passe entièrement par `PlatformAdapter` — porte P-15
 *
 * C'est le **premier usage d'une capacité native par un écran** du produit. Aucun `localStorage`
 * n'apparaît ici : `core/platform/stockage-web.ts` est la seule implémentation qui le nomme, et
 * elle vit dans le seul répertoire où la détection de plateforme est autorisée.
 *
 * # La garantie du stockage est LUE, pas supposée
 *
 * Sur le web, `garantie` vaut `'aucune'` — voir {@link NiveauGarantieStockage}. Ce module
 * l'accepte, et c'est une décision : l'alternative serait de redemander le mot de passe à chaque
 * ouverture d'onglet, ce qui ferait écrire le mot de passe sur un papier au comptoir. La
 * contrepartie est portée par le serveur — rotation à chaque usage, révocation de toute la
 * famille dès qu'une copie est rejouée, coupure immédiate depuis « Appareils connectés ».
 * Ce qui n'est **pas** acceptable, et que le code refuse plus bas, c'est de ranger un jeton dans
 * un stockage `'indisponible'` en croyant l'avoir rangé.
 */

import { adaptateurCourant } from '~/core/platform/courant'

/** Clé du jeton de rafraîchissement dans le stockage de la plateforme. */
export const CLE_RAFRAICHISSEMENT = 'auth.rafraichissement'

/**
 * L'utilisateur connecté, tel que les écrans le voient.
 *
 * **Aucun jeton n'y figure.** Un écran qui recevrait le jeton en `prop` finirait par le mettre
 * dans un `<input>` de débogage, et il n'a de toute façon aucune raison de le lire : les appels
 * passent par {@link contexteAppel}.
 */
export interface SessionUtilisateur {
  readonly compteId: string
  readonly tenantId: string
  /** Établissement actif du sélecteur de contexte permanent (principe VII). */
  readonly etablissementId: string | null
  /** Permissions **cumulées** de tous les rôles portés — l'union, jamais un rôle principal. */
  readonly permissions: readonly string[]
  /** Les établissements accessibles. Le sélecteur permanent est **ETB-06**, hors périmètre. */
  readonly etablissements: readonly string[]
}

/**
 * Contexte d'appel de l'API — **ce qui remplace les deux en-têtes du cycle 001**.
 *
 * `x-kaya-tenant` et `x-kaya-compte` laissaient l'appelant choisir son tenant ; la dérogation
 * `CONTEXTE_PAR_EN_TETES` est **levée** côté serveur depuis T030, et l'API ne les accepte plus.
 * Le tenant, le compte et les permissions viennent désormais du jeton **vérifié**.
 */
export interface ContexteAppel {
  /** Racine de l'API — jamais codée en dur dans un composant. */
  readonly baseUrl: string
  /** Jeton d'accès, porté par `Authorization: Bearer`. */
  readonly acces: string
}

interface EtatSession {
  readonly session: SessionUtilisateur
  readonly acces: string
  /** Horodatage local d'expiration, en millisecondes. **Indicatif** — le serveur fait foi. */
  readonly expireLe: number
}

/**
 * L'état courant — une variable de module, volontairement.
 *
 * Un magasin réactif Pinia ou un `provide/inject` ajouterait une dépendance et une deuxième façon
 * d'y accéder. Une seule variable, quatre fonctions qui la touchent, et les écrans passent par
 * `useSession()` s'ils ont besoin de réactivité.
 */
let etat: EtatSession | null = null

/** La session en cours, ou `null` si personne n'est connecté. */
export function sessionCourante(): SessionUtilisateur | null {
  return etat?.session ?? null
}

/**
 * Le contexte d'appel courant, ou `null` hors session.
 *
 * Rendre `null` plutôt que de lever oblige l'appelant à traiter le cas « pas connecté », qui est
 * l'état normal de l'application au démarrage.
 */
export function contexteAppel(baseUrl: string): ContexteAppel | null {
  return etat ? { baseUrl, acces: etat.acces } : null
}

/**
 * En-têtes d'authentification, pour le client généré.
 *
 * **Un seul en-tête, et c'est tout le sujet du cycle** : il n'y a plus rien à choisir dans une
 * requête. Le serveur lit le tenant dans le jeton qu'il a signé.
 */
export function enTetesAuth(contexte: ContexteAppel): Record<string, string> {
  return { Authorization: `Bearer ${contexte.acces}` }
}

/** Le jeton d'accès est-il expiré, ou sur le point de l'être ? */
export function accesExpire(margeSecondes = 60): boolean {
  if (!etat) {
    return true
  }
  return Date.now() >= etat.expireLe - margeSecondes * 1000
}

/** Pose l'état de session en mémoire. Le rafraîchissement est rangé séparément. */
export function poserSession(
  session: SessionUtilisateur,
  acces: string,
  expireDansS: number,
): void {
  etat = { session, acces, expireLe: Date.now() + expireDansS * 1000 }
}

/** Efface l'état en mémoire. Ne touche pas au stockage — voir {@link oublierRafraichissement}. */
export function effacerSession(): void {
  etat = null
}

/**
 * Range le jeton de rafraîchissement dans le stockage de la plateforme.
 *
 * @returns `true` si le jeton est réellement rangé. `false` signifie que la session vivra le temps
 *   du jeton d'accès et pas davantage — l'appelant **le dit à l'utilisateur** plutôt que de le
 *   laisser découvrir une déconnexion inexpliquée une heure plus tard.
 */
export async function rangerRafraichissement(jeton: string): Promise<boolean> {
  const stockage = adaptateurCourant().stockageSecurise
  if (stockage.garantie === 'indisponible') {
    // Écrire quand même produirait un `{ disponible: false }` qu'on ignorerait : autant le
    // constater ici, où la raison est lisible.
    return false
  }
  const resultat = await stockage.ecrire(CLE_RAFRAICHISSEMENT, jeton)
  return resultat.disponible
}

/** Relit le jeton de rafraîchissement, ou `null` s'il n'y en a pas — ou pas de stockage. */
export async function lireRafraichissement(): Promise<string | null> {
  const resultat = await adaptateurCourant().stockageSecurise.lire(CLE_RAFRAICHISSEMENT)
  return resultat.disponible ? resultat.valeur : null
}

/**
 * **Purge à la déconnexion** — exigée par le principe VI.
 *
 * Elle vide *tout* le stockage de l'application, pas seulement le jeton : ce sont des données
 * d'identité, et le terminal peut être partagé — au maquis, le téléphone du gérant sert aussi à
 * la serveuse. Un jeton retiré à côté d'un cache de comptes serait une demi-purge.
 */
export async function oublierRafraichissement(): Promise<void> {
  await adaptateurCourant().stockageSecurise.purger()
}
