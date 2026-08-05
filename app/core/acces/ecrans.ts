/**
 * **Quelle permission ouvre quel écran** — déclaration unique, lue par la tuile ET par la page.
 *
 * ═══════════════════════════════════════════════════════════════════════════════════════════
 *  POURQUOI UNE SEULE TABLE, ET PAS UNE DÉCLARATION DE CHAQUE CÔTÉ
 * ═══════════════════════════════════════════════════════════════════════════════════════════
 *
 * Deux mécanismes gardent un écran, et ils doivent dire **la même chose** :
 *
 * 1. **l'accueil**, qui n'affiche pas la tuile — FR-026, principe VII : absente, jamais grisée ;
 * 2. **la page**, qui refuse l'accès direct par l'URL — FR-029.
 *
 * Les écrire séparément produit deux vérités qui divergent au premier cycle, et la divergence a
 * **deux formes, toutes deux mauvaises** :
 *
 * - la page est plus stricte que la tuile → **une tuile qui ouvre sur un refus**, au comptoir,
 *   devant le client. C'est pire qu'une tuile absente : elle a promis.
 * - la tuile est plus stricte que la page → l'écran est **caché mais atteignable** par l'URL, et
 *   le contrôle ne tient plus qu'au serveur, qui répondra `403` sur une page muette.
 *
 * Une seule table les rend **structurellement impossibles** : il n'y a pas deux endroits où se
 * tromper. `core/accueil/tuiles.ts` la lit pour filtrer, chaque page la lit pour refuser, et
 * `app/tests/catalogue-accueil.spec.ts` vérifie que **toute** route de `app/pages/` y figure.
 *
 * ═══════════════════════════════════════════════════════════════════════════════════════════
 *  CE SONT LES PERMISSIONS DE LECTURE, JAMAIS CELLES DU GESTE
 * ═══════════════════════════════════════════════════════════════════════════════════════════
 *
 * Elles répondent à « cet écran peut-il s'AFFICHER ? », pas à « cette personne peut-elle agir ? ».
 * Le propriétaire consulte les séjours en cours sans pouvoir en clore un : l'écran s'ouvre, la
 * liste s'affiche, et le bouton « Faire partir le client » est absent — c'est la garde du
 * composant qui le retire, et c'est le bon endroit. Accrocher l'écran à `heb.sejour.clore` lui
 * retirerait la consultation qu'il a le droit d'avoir.
 *
 * Chaque liste vient de ce que le **serveur** exige sur les routes que l'écran appelle **au
 * montage** — `backend/api/src/routes/*.rs`, constantes `PERM_*`. Elles ne sont pas déduites du
 * nom de l'écran.
 *
 * ═══════════════════════════════════════════════════════════════════════════════════════════
 *  ⚠️ CE QUI MANQUAIT, ET QUI N'ÉTAIT PAS UNE FUITE
 * ═══════════════════════════════════════════════════════════════════════════════════════════
 *
 * Six écrans — `/hebergement`, `/chambres`, `/passage`, `/arrivee`, `/clients`, `/depart` — ne
 * gardaient que leurs **gestes** (`peutGerer`, `peutOuvrir`, `peutClore`). Seuls `/comptes` et
 * `/journal-audit` gardaient la **lecture**.
 *
 * **Aucune donnée ne fuyait** : le serveur refuse, et les six écrans ne montraient rien. Le défaut
 * était de **langue**. Sur URL directe sans permission, ils affichaient « Les chambres n'ont pas
 * pu être chargées. » — un message d'échec technique, qui envoie chercher un problème de réseau
 * qui n'existe pas. `/journal-audit` disait, lui, « Vous n'avez pas accès au registre des
 * actions. » Les huit écrans se comportent désormais pareil.
 */

import { detient, type Permissions } from '~/core/rbac'

/** Ce qui garde un écran. */
export interface AccesEcran {
  /**
   * Les permissions exigées pour que l'écran s'affiche — **toutes** requises.
   *
   * Une liste vide signifie « toute session ouverte », et exige alors un {@link motif}.
   */
  readonly permissions: readonly string[]
  /**
   * Pourquoi cet écran n'est gardé par AUCUNE permission.
   *
   * Obligatoire quand `permissions` est vide, et vérifié par `catalogue-accueil.spec.ts`. Un écran
   * que rien ne garde est une décision ; sans motif écrit, le prochain se posera par imitation.
   */
  readonly motif?: string
}

/**
 * **Toutes** les routes de `app/pages/`, et ce qui les ouvre.
 *
 * Le test de catalogue échoue en nommant la route qu'un cycle aurait ajoutée sans passer ici — il
 * découvre `app/pages/` plutôt que de lire cette table, sans quoi il ne vérifierait que sa propre
 * cohérence.
 */
export const ACCES_ECRANS: Readonly<Record<string, AccesEcran>> = {
  // ── Le comptoir ────────────────────────────────────────────────────────────────────────
  //
  // `chargerPassage` — dont `chargerArrivee` est un alias — fait TROIS lectures en parallèle :
  // l'état des unités (`heb.disponibilite.consulter`), les catégories et les formules
  // (`heb.offre.lire`). Les deux sont donc requises pour que l'écran affiche autre chose que son
  // message d'échec.
  '/passage': { permissions: ['heb.disponibilite.consulter', 'heb.offre.lire'] },
  '/arrivee': { permissions: ['heb.disponibilite.consulter', 'heb.offre.lire'] },
  // La liste des séjours en cours, et rien de plus au montage. `heb.sejour.clore` garde le geste,
  // DANS l'écran : le propriétaire consulte sans pouvoir faire partir.
  '/depart': { permissions: ['heb.sejour.lire'] },
  '/clients': { permissions: ['sej.client.lire'] },

  // ── L'équipe, et ce que l'appareil n'a pas envoyé ───────────────────────────────────────
  '/notes': { permissions: ['etb.note.lire'] },
  '/mes-envois': {
    permissions: [],
    motif:
      '`S1` ne montre AUCUNE donnée d’établissement : il montre la file de CET appareil — le '
      + 'travail que la personne connectée n’a pas encore réussi à envoyer. Il n’appelle pas une '
      + 'seule route de l’API, donc aucune permission serveur ne le garde ni ne pourrait le '
      + 'garder. Sa garde effective est la session, posée par `middleware/01.session.global.ts`, '
      + 'et elle suffit : sans session, la route renvoie sur `/connexion` comme les douze autres. '
      + 'Lui accrocher une permission d’établissement — `etb.etablissement.lire`, que les huit '
      + 'rôles portent — aurait marché aujourd’hui et serait un rattachement FAUX : le jour où un '
      + 'rôle la perd pour une raison sans rapport, la personne cesse de voir ce que son propre '
      + 'appareil n’a pas envoyé. Le témoin de synchronisation (composant 10) est dans la '
      + 'coquille, donc sur toutes les pages, et il annonce « En attente d’envoi (4) » à '
      + 'quelqu’un qui n’aurait alors aucun moyen d’aller voir. Un état annoncé vers lequel rien '
      + 'ne mène est pire que pas d’état du tout.',
  },

  // ── L'offre, et les réglages ───────────────────────────────────────────────────────────
  '/hebergement': { permissions: ['heb.offre.lire'] },
  '/chambres': { permissions: ['heb.offre.lire'] },
  '/etablissement': { permissions: ['etb.etablissement.lire'] },
  '/comptes': { permissions: ['cpt.compte.lire'] },
  '/journal-audit': { permissions: ['cpt.audit.consulter'] },

  // ── Les trois routes que rien ne garde, et pourquoi ────────────────────────────────────
  '/': {
    permissions: [],
    motif:
      'L’accueil n’a rien à garder : il ne montre QUE ce que la session ouvre déjà. Filtrer '
      + 'l’écran lui-même reviendrait à refuser l’entrée à quelqu’un dont on va de toute façon '
      + 'n’afficher aucune tuile — et à confondre « personne n’est connecté » avec « ce compte '
      + 'n’a aucun droit », qui se ressemblent à l’écran et n’ont rien à voir. Le second obtient '
      + 'une explication ; le premier, la connexion.',
  },
  '/connexion': {
    permissions: [],
    motif:
      'C’est l’écran par lequel on entre. Exiger une permission pour se connecter serait une '
      + 'boucle : il faut une session pour avoir des permissions.',
  },
  '/styleguide': {
    permissions: [],
    motif:
      'Surface de développement, retirée du routeur hors `KAYA_STYLEGUIDE=1` — la garde n’est pas '
      + 'une permission mais l’absence de la route en production, et c’est `app/tests/'
      + 'styleguide.spec.ts` qui la vérifie. Aucune donnée d’établissement n’y est lue : la page '
      + 'rend les seize composants sur des valeurs d’échantillon.',
  },
}

/**
 * Les permissions qui ouvrent cet écran.
 *
 * ⚠️ **Une route inconnue est REFUSÉE, pas laissée ouverte.** C'est le défaut sûr : un écran ajouté
 * sans passer par cette table doit être visible en développement — il l'est immédiatement, par un
 * refus — plutôt que de se retrouver sans garde en production. `catalogue-accueil.spec.ts` fait
 * échouer la CI dans le même mouvement, en nommant la route.
 */
export function accesDeLEcran(route: string): AccesEcran | undefined {
  return ACCES_ECRANS[route]
}

/**
 * Cette personne peut-elle ouvrir cet écran ?
 *
 * @param route Le chemin servi, tel que `app/pages/` le produit — `/passage`, `/`.
 * @param permissions Union des rôles portés, jamais celles d'un rôle « principal ».
 */
export function peutOuvrirEcran(route: string, permissions: Permissions): boolean {
  const acces = accesDeLEcran(route)
  if (!acces) {
    return false
  }
  // `every` sur une liste vide rend `true`, ce qui est exactement la sémantique voulue pour les
  // écrans que rien ne garde.
  return acces.permissions.every(permission => detient(permissions, permission))
}
