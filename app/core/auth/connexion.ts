/**
 * Ouverture, rafraîchissement et fermeture de session — **le patron d'écriture front appliqué**.
 *
 * Les huit points de `docs/module-dore.md`, « La septième couche », s'appliquent tels quels : appel
 * par le client généré, refus traduit du `code` et jamais du `message`, refus **avant** l'appel
 * quand le réseau manque. Deux choses lui sont propres, et elles sont le cœur de CPT-01.
 *
 * # 1 · Les deux échecs rendent LA MÊME phrase — FR-012
 *
 * Le serveur ne rend qu'un seul code, `identifiants_invalides`, pour un identifiant inconnu comme
 * pour un mot de passe faux, un compte désactivé ou un dépassement de tentatives — et il y met le
 * **même temps** (`backend/tests/authentification_indiscernable.rs`). Le front ne défait pas ce
 * travail : **tout `401` sur l'ouverture rend la phrase unique**, quel que soit le code reçu.
 *
 * Brancher la table des clés sur le code aurait marché aujourd'hui et ouvert la fuite demain : il
 * aurait suffi qu'un cycle ultérieur ajoute un code au serveur pour que l'écran se mette à
 * distinguer les deux cas, sans que rien n'échoue. Le `401` est traité en bloc, et
 * `app/tests/ecran-r0.spec.ts` le vérifie sur trois codes différents.
 *
 * # 2 · La rotation est invisible à l'appelant, la détection de vol ne l'est pas
 *
 * Un rafraîchissement consomme le jeton présenté et en délivre un autre : le nouveau **remplace**
 * l'ancien dans le stockage, sinon le suivant échouerait. Un jeton déjà consommé rend `401` et
 * révoque **toute la famille** côté serveur — ce n'est pas une erreur ordinaire, c'est le signal
 * qu'une copie circule. L'appelant, lui, retombe sur l'écran de connexion : c'est ce qu'il y a de
 * mieux à faire, et c'est aussi ce qui rend le vol visible à la victime.
 */

import type { components } from '@kaya/client'

import { clientKaya } from '~/core/api/client'

import type { EtatReseau } from '~/core/platform'
import {
  contexteAppel,
  effacerSession,
  enTetesAuth,
  lireRafraichissement,
  oublierJetonRafraichissement,
  oublierRafraichissement,
  poserSession,
  rangerRafraichissement,
  type SessionUtilisateur,
} from './session'

/**
 * **La phrase unique.** Un identifiant inconnu, un mot de passe faux, un compte désactivé et un
 * dépassement de tentatives affichent tous celle-ci.
 */
export const REFUS_IDENTIFIANTS = 'connexion.refus.identifiants'

/** La méthode d'authentification du compte n'est pas implémentée — `OTP_SMS` (FR-008). */
export const REFUS_METHODE = 'connexion.refus.methode_non_implementee'

/** Le réseau manque. Refus **immédiat**, avant toute tentative. */
export const REFUS_RESEAU = 'connexion.refus.reseau'

/** Refus qu'on ne sait pas nommer : une phrase honnête vaut mieux qu'une clé i18n en brut. */
export const REFUS_INATTENDU = 'connexion.refus.inattendue'

/** La session ne peut pas reprendre : jeton absent, expiré, révoqué ou déjà consommé. */
export const REFUS_SESSION_EXPIREE = 'connexion.refus.session_expiree'

/** Identifiants saisis à l'écran `R0`. */
export interface Identifiants {
  /**
   * Numéro de téléphone **ou** courriel — **un seul champ**.
   *
   * L'utilisateur ne sait pas dans quelle colonne son identifiant est rangé, et le lui demander
   * serait lui faire porter un détail de modèle de données.
   */
  readonly identifiant: string
  readonly motDePasse: string
  /** Établissement souhaité. Sans lui, le premier accessible par ordre stable devient actif. */
  readonly etablissementId?: string
  /** Libellé d'appareil, **indicatif** : il sert à reconnaître son téléphone avant de couper l'autre. */
  readonly libelleAppareil?: string
}

/** Ce qu'une ouverture ou une reprise produit. Un seul type de retour, jamais d'exception au vol. */
export type ResultatConnexion =
  | {
    issue: 'succes'
    session: SessionUtilisateur
    /**
     * Le jeton de rafraîchissement a-t-il pu être rangé ?
     *
     * `false` sur une plateforme sans stockage : la session vivra le temps du jeton d'accès et
     * pas davantage. L'écran **le dit**, plutôt que de laisser découvrir une déconnexion
     * inexpliquée une heure plus tard.
     */
    persistante: boolean
  }
  | {
    issue: 'refus'
    /** Clé i18n du message à afficher. Toujours renseignée. */
    cle: string
    /** Le refus vient-il de l'absence de réseau ? L'interface ne le rend pas de la même façon. */
    reseau?: boolean
  }

/** Le corps d'erreur du contrat, réduit à ce que cette couche en consomme. */
interface CorpsErreur {
  code?: string
}

/**
 * Ce que rendent `session_ouvrir` et `session_rafraichir`.
 *
 * **Alias du contrat, pas une copie.** Cette déclaration était écrite à la main, « réduite à ce
 * qui est consommé ici », et les deux appels y convertissaient par `as unknown as` : renommer
 * `permissions` côté serveur aurait laissé le front compiler pour ne poser que des permissions
 * `undefined` — sur le chemin qui décide de ce que chacun voit. Constaté en T062.
 */
type SessionOuverteVue = components['schemas']['SessionOuverteVue']

/** Pose l'état de session à partir de ce que le serveur vient de rendre. */
async function adopter(vue: SessionOuverteVue): Promise<ResultatConnexion> {
  const session: SessionUtilisateur = {
    compteId: vue.compte.compte_id,
    tenantId: vue.compte.tenant_id,
    etablissementId: vue.compte.etablissement_actif ?? null,
    // **Les permissions viennent d'ici, en clair.** Jamais du jeton décodé (research R-06).
    permissions: vue.permissions,
    etablissements: vue.etablissements,
  }
  poserSession(session, vue.acces, vue.expire_dans_s)
  // Le rafraîchissement est rangé APRÈS la pose de l'état : si le stockage échoue, la session est
  // quand même utilisable tout de suite, et c'est l'échec de persistance qu'on rapporte.
  const persistante = await rangerRafraichissement(vue.rafraichissement)
  return { issue: 'succes', session, persistante }
}

/**
 * Ouvre une session.
 *
 * @param reseau État réseau **au moment du geste**, lu depuis `PlatformAdapter`. Passé en
 *   paramètre plutôt que lu ici : la fonction reste testable sans navigateur, et l'appelant est
 *   obligé de constater qu'il y a une question d'état réseau à poser.
 */
export async function ouvrirSession(
  baseUrl: string,
  identifiants: Identifiants,
  reseau: EtatReseau,
): Promise<ResultatConnexion> {
  // Le refus est **immédiat et explicite**, et il précède l'appel. Une connexion ne se met pas en
  // file « au cas où » : elle n'a rien à rejouer, et promettre un envoi qu'on ne sait pas tenir est
  // pire que dire non (principe VI).
  if (reseau !== 'connecte') {
    return { issue: 'refus', cle: REFUS_RESEAU, reseau: true }
  }

  const client = clientKaya(baseUrl)
  const reponse = await client.POST('/api/v1/session', {
    body: {
      identifiant: identifiants.identifiant,
      mot_de_passe: identifiants.motDePasse,
      etablissement_id: identifiants.etablissementId ?? null,
      libelle_appareil: identifiants.libelleAppareil ?? null,
    },
  })

  if (!reponse.error && reponse.data) {
    return adopter(reponse.data)
  }

  // **Tout `401` rend la phrase unique** — le code n'est pas consulté. Voir le commentaire de
  // tête : brancher sur le code marcherait aujourd'hui et fuirait au premier code ajouté.
  if (reponse.response.status === 401) {
    return { issue: 'refus', cle: REFUS_IDENTIFIANTS }
  }

  const corps = reponse.error as CorpsErreur | undefined
  if (reponse.response.status === 422 && corps?.code === 'methode_non_implementee') {
    return { issue: 'refus', cle: REFUS_METHODE }
  }

  return { issue: 'refus', cle: REFUS_INATTENDU }
}

/**
 * Rafraîchit la session — **rotation à chaque usage**.
 *
 * Employée à trois moments : à la reprise de l'application ({@link reprendreSession}), avant de
 * vider la file hors ligne (`core/sync`, T034), et quand le jeton d'accès approche de sa fin.
 *
 * **Un échec n'efface pas la session en mémoire.** Le jeton d'accès courant peut encore être
 * valide, et couper l'utilisateur au premier rafraîchissement raté lui ferait perdre sa saisie sur
 * un réseau qui a hoqueté. Ce qui est effacé, c'est le jeton de rafraîchissement rangé : il est
 * mort, le garder ferait échouer chaque tentative suivante à l'identique.
 */
export async function rafraichirSession(
  baseUrl: string,
  etablissementId?: string,
): Promise<ResultatConnexion> {
  const rafraichissement = await lireRafraichissement()
  if (!rafraichissement) {
    return { issue: 'refus', cle: REFUS_SESSION_EXPIREE }
  }

  const client = clientKaya(baseUrl)
  const reponse = await client.POST('/api/v1/session/rafraichir', {
    body: { rafraichissement, etablissement_id: etablissementId ?? null },
  })

  if (!reponse.error && reponse.data) {
    return adopter(reponse.data)
  }

  if (reponse.response.status === 401) {
    // Le jeton est consommé, révoqué ou inconnu. Dans le cas le plus grave — une copie rejouée —
    // le serveur vient de couper toute la famille. Retomber sur la connexion est la seule suite
    // possible, et c'est aussi ce qui rend le vol visible à la victime.
    //
    // **Le JETON part, et lui seul.** Une purge totale emporterait la clé qui déchiffre la file
    // hors-ligne, et les écritures non parties deviendraient illisibles — y compris après une
    // reconnexion réussie. C'est le contraire de la règle 2 de `core/sync/vidage.ts`, et la faute
    // ne se voit qu'après une coupure plus longue que le jeton, donc jamais en développement.
    // La purge totale reste le geste de `fermerSession`, où la personne change vraiment.
    await oublierJetonRafraichissement()
    return { issue: 'refus', cle: REFUS_SESSION_EXPIREE }
  }

  return { issue: 'refus', cle: REFUS_INATTENDU }
}

/**
 * Reprend une session au démarrage de l'application.
 *
 * Sans elle, un rechargement de page renverrait à l'écran de connexion alors que le jeton de
 * rafraîchissement est encore bon pour quatre-vingt-dix jours — ce qui reviendrait à demander le
 * mot de passe plusieurs fois par jour, et à le faire écrire sur un papier au comptoir.
 */
export async function reprendreSession(baseUrl: string, reseau: EtatReseau): Promise<ResultatConnexion> {
  if (reseau !== 'connecte') {
    return { issue: 'refus', cle: REFUS_RESEAU, reseau: true }
  }
  return rafraichirSession(baseUrl)
}

/**
 * Ferme la session — **la déconnexion volontaire**.
 *
 * L'état local est effacé **quoi qu'il arrive**, y compris si l'appel échoue : un utilisateur qui
 * demande à se déconnecter sur un réseau coupé doit être déconnecté de son terminal. Le serveur,
 * lui, laissera la session expirer — c'est le comportement de l'opération, pas une lacune du
 * client.
 */
export async function fermerSession(baseUrl: string): Promise<void> {
  const contexte = contexteAppel(baseUrl)
  if (contexte) {
    const client = clientKaya(baseUrl)
    try {
      await client.DELETE('/api/v1/session', { headers: enTetesAuth(contexte) })
    }
    catch {
      // Le réseau a manqué. La purge locale ci-dessous vaut toujours.
    }
  }
  effacerSession()
  // **Purge des données d'identité** (principe VI) — le terminal peut être partagé.
  await oublierRafraichissement()
}
