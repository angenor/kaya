/**
 * **Le catalogue des tuiles de l'accueil `R1`** — et les permissions qui ouvrent chacune.
 *
 * # L'accueil est un tableau de bord filtré, jamais un menu figé
 *
 * Principe VII : quatre comptes, quatre accueils, sur la même application. Une tuile dont
 * l'utilisateur n'a pas la permission est **absente**, pas grisée ; une tuile dont le module
 * d'activité n'est pas activé dans l'établissement est **absente** aussi. Le grisé apprend à
 * l'utilisateur, à chaque écran et tous les jours, qu'une partie du produit lui est refusée.
 *
 * # Le catalogue ne contient que des écrans QUI EXISTENT — principe X
 *
 * Les maquettes de `R1` montrent un accueil riche : tables ouvertes, chambres occupées, argent non
 * encaissé, « Ajouter une commande ». Ces tuiles ouvrent des écrans des cycles PDV, CAI et FIS,
 * dont aucun n'est livré. Les poser produirait une interface qui promet ce qu'elle ne sait pas
 * faire — « prêt ≠ construit », et c'est le principe X qui l'interdit, pas une prudence de
 * rédaction.
 *
 * # ⚠️ CE CATALOGUE A ÉTÉ EN RETARD DE DEUX CYCLES, ET RIEN NE L'A DIT
 *
 * Les cycles SYN et SEJ ont livré six écrans — `/notes`, `/mes-envois`, `/passage`, `/arrivee`,
 * `/clients`, `/depart` — **sans en inscrire un seul ici**. Les treize routes s'ouvraient, leurs
 * données étaient réelles, les portes étaient vertes, et l'accueil d'Adjoua ne menait qu'à deux
 * d'entre elles : les onze autres n'existaient que pour qui savait taper l'URL.
 *
 * C'est le motif du cycle 003 sous une autre forme — là, les écrans ne se montaient pas ; ici ils
 * se montaient parfaitement et personne ne pouvait y arriver. Dans les deux cas, **le parcours
 * n'était couvert par rien**.
 *
 * `app/tests/catalogue-accueil.spec.ts` referme la porte : il **découvre** les routes de
 * `app/pages/` — il ne les énumère pas — et échoue en nommant celles qui n'ont ni tuile ici ni
 * exemption motivée. Le cycle 007 ajoutera ses écrans PDV ; il ne pourra pas les oublier en
 * silence.
 *
 * # ⚠️ UNE TUILE NE DÉCLARE PLUS SES PERMISSIONS — ELLE DÉCLARE SA ROUTE
 *
 * Elles vivent dans `core/acces/ecrans.ts`, avec celles que la **page** oppose à l'accès direct
 * par l'URL. Les écrire des deux côtés produisait deux vérités qui divergent au premier cycle, et
 * la divergence a deux formes, toutes deux mauvaises : une tuile qui ouvre sur un refus — au
 * comptoir, devant le client —, ou un écran caché mais atteignable.
 *
 * Une seule table les rend **structurellement impossibles** : il n'y a pas deux endroits où se
 * tromper. Ce fichier décide de ce qui est **proposé** ; `core/acces/ecrans.ts` décide de ce qui
 * est **ouvert**.
 *
 * # Une tuile issue de plusieurs rôles n'apparaît QU'UNE FOIS — FR-027
 *
 * Elle est tenue par la structure, pas par un dédoublonnage : le catalogue est une liste de
 * tuiles, pas une liste par rôle. Adjoua porte trois rôles ; chaque tuile est nommée une seule
 * fois, et ses permissions sont cherchées dans l'union. Il n'y a **aucun endroit** où elle
 * pourrait se dupliquer.
 */

import { accesDeLEcran, peutOuvrirEcran } from '~/core/acces/ecrans'
import type { Permissions } from '~/core/rbac'

/** Une tuile de l'accueil. */
export interface Tuile {
  /** Identifiant stable — sert de clé de rendu et d'ancre de test. */
  readonly code: string
  /** Clé i18n du libellé. **Jamais une phrase.** */
  readonly libelleCle: string
  /** Clé i18n de la ligne d'explication, sous le libellé. */
  readonly descriptionCle: string
  /** Glyphe Phosphor, **décoratif** : la tuile porte déjà son libellé traduit. */
  readonly icone: string
  /**
   * Route Nuxt ouverte au clic — **et clé de ce qui la garde**.
   *
   * Les permissions viennent de `ACCES_ECRANS[route]`, jamais d'un champ d'ici : voir l'en-tête.
   */
  readonly route: string
  /**
   * Module d'activité à activer pour que la tuile existe.
   *
   * `undefined` = transverse — le module est alors sans objet, pas « inconnu ». C'est le cas des
   * permissions dont `comptes.permission.module_code` vaut `NULL` : les `etb.*`, les `cpt.*` et
   * **les `sej.*`**, qui sont transverses alors que leurs écrans parlent d'hôtellerie.
   */
  readonly moduleRequis?: string
}

/**
 * Les tuiles du produit, dans leur ordre d'affichage.
 *
 * L'ordre est **stable et indépendant de la locale** : trier sur le libellé traduit ferait changer
 * l'accueil en passant du français à l'anglais. Même raison que l'`ordre` des référentiels.
 *
 * **Il va du geste au réglage**, et pas l'inverse. Le sous-titre de l'écran dit « Ce que vous
 * pouvez faire aujourd'hui » : ce qu'Adjoua fait quarante fois par jour — recevoir, faire partir,
 * chercher une fiche — vient avant ce qu'elle règle deux fois par an. L'ordre précédent ouvrait
 * sur « Votre établissement », ce qui était tenable avec trois tuiles et ne l'est plus avec onze.
 */
export const CATALOGUE_TUILES: readonly Tuile[] = [
  // ── Le comptoir : les quatre gestes du cycle SEJ ────────────────────────────────────────
  //
  // Leurs permissions de lecture sont celles que le serveur exige sur les routes que ces écrans
  // appellent AU MONTAGE — `backend/api/src/routes/hebergement_disponibilite.rs`,
  // `hebergement_referentiel.rs`, `sejours.rs`, `clients.rs`. Elles ne sont pas devinées du nom de
  // l'écran : une tuile qui ouvre sur un refus est pire qu'une tuile absente.
  {
    code: 'passage',
    libelleCle: 'accueil.tuiles.passage.libelle',
    descriptionCle: 'accueil.tuiles.passage.description',
    icone: 'ph-lightning',
    // `chargerPassage` fait TROIS lectures en parallèle : l'état des unités
    // (`heb.disponibilite.consulter`), les catégories et les formules (`heb.offre.lire`). Les deux
    // sont donc requises pour que l'écran affiche autre chose que son message d'échec.
    route: '/passage',
    moduleRequis: 'HEBERGEMENT',
  },
  {
    code: 'arrivee',
    libelleCle: 'accueil.tuiles.arrivee.libelle',
    descriptionCle: 'accueil.tuiles.arrivee.description',
    icone: 'ph-suitcase',
    // `chargerArrivee` **est** `chargerPassage` — même chargeur, mêmes lectures, mêmes
    // permissions. Le partage est déclaré dans `modules/sejours/donnees.ts`.
    route: '/arrivee',
    moduleRequis: 'HEBERGEMENT',
  },
  {
    code: 'depart',
    libelleCle: 'accueil.tuiles.depart.libelle',
    descriptionCle: 'accueil.tuiles.depart.description',
    icone: 'ph-sign-out',
    // La liste des séjours en cours, et rien de plus au montage. `heb.sejour.clore` garde le
    // geste, DANS l'écran : le propriétaire consulte sans pouvoir faire partir.
    route: '/depart',
    moduleRequis: 'HEBERGEMENT',
  },
  {
    code: 'clients',
    libelleCle: 'accueil.tuiles.clients.libelle',
    descriptionCle: 'accueil.tuiles.clients.description',
    icone: 'ph-address-book',
    route: '/clients',
    // ⚠️ **Aucun module requis, et ce n'est pas un oubli.** `sej.client.*` porte
    // `module_code = NULL` en base (migration `0030`) : un maquis tient des fiches clients sans
    // louer une seule chambre. Rattacher la tuile à `HEBERGEMENT` la ferait disparaître du seul
    // établissement où elle est parfois le plus utile.
  },

  // ── Ce que l'équipe se laisse, et ce que l'appareil n'a pas encore envoyé ───────────────
  {
    code: 'notes',
    libelleCle: 'accueil.tuiles.notes.libelle',
    descriptionCle: 'accueil.tuiles.notes.description',
    icone: 'ph-note-pencil',
    route: '/notes',
  },
  {
    code: 'mes-envois',
    libelleCle: 'accueil.tuiles.mes_envois.libelle',
    descriptionCle: 'accueil.tuiles.mes_envois.description',
    icone: 'ph-paper-plane-tilt',
    // ⚠️ **LA SEULE TUILE DU PRODUIT SANS PERMISSION** — décision, pas oubli. Voir le motif.
    route: '/mes-envois',
  },

  // ── L'offre d'hébergement — cycle 004, les PREMIÈRES tuiles rattachées à un module ──────
  //
  // `moduleRequis` existait depuis le cycle 003, filtré et testé, mais aucune tuile ne le portait :
  // son mécanisme était vérifié sur une cible fictive. Un établissement qui ne fait pas
  // d'hébergement n'a pas de tuile « Vos formules », et elle est ABSENTE, jamais grisée.
  {
    code: 'hebergement-offre',
    libelleCle: 'accueil.tuiles.hebergement_offre.libelle',
    descriptionCle: 'accueil.tuiles.hebergement_offre.description',
    icone: 'ph-bed',
    route: '/hebergement',
    moduleRequis: 'HEBERGEMENT',
  },
  {
    code: 'hebergement-chambres',
    libelleCle: 'accueil.tuiles.hebergement_chambres.libelle',
    descriptionCle: 'accueil.tuiles.hebergement_chambres.description',
    icone: 'ph-door-open',
    route: '/chambres',
    moduleRequis: 'HEBERGEMENT',
  },

  // ── Les réglages, et ce que le propriétaire achète ──────────────────────────────────────
  {
    code: 'etablissement',
    libelleCle: 'accueil.tuiles.etablissement.libelle',
    descriptionCle: 'accueil.tuiles.etablissement.description',
    icone: 'ph-storefront',
    route: '/etablissement',
  },
  {
    code: 'comptes',
    libelleCle: 'accueil.tuiles.comptes.libelle',
    descriptionCle: 'accueil.tuiles.comptes.description',
    icone: 'ph-users-three',
    route: '/comptes',
  },
  {
    code: 'journal-audit',
    libelleCle: 'accueil.tuiles.journal.libelle',
    descriptionCle: 'accueil.tuiles.journal.description',
    icone: 'ph-list-magnifying-glass',
    route: '/journal-audit',
  },
]

/**
 * Les routes de `app/pages/` qui n'ont **volontairement** aucune tuile, et pourquoi.
 *
 * # C'est la seule chose écrite à la main, et elle doit rester courte et motivée
 *
 * `app/tests/catalogue-accueil.spec.ts` découvre les routes du système de fichiers et exige que
 * chacune soit **soit** au catalogue, **soit** ici avec son motif. Une liste muette de chemins
 * exemptés se serait allongée sans que personne ne relise ; un motif par ligne se relit, et se
 * conteste.
 *
 * Le test refuse aussi une exemption **morte** — une route qui a une tuile ET une exemption —
 * pour que ce tableau ne survive pas à la raison qui l'a rempli.
 */
export const ROUTES_SANS_TUILE: Readonly<Record<string, string>> = {
  '/': 'C’est l’accueil lui-même. Une tuile vers soi est une boucle, pas un chemin.',
  '/connexion':
    'On n’y va pas DEPUIS l’accueil : c’est l’écran par lequel on entre, et le middleware de '
    + 'session y renvoie tout seul quand il n’y a plus de session. Le geste inverse — quitter son '
    + 'poste — est « Passer la main », et il vit dans la coquille, pas dans une tuile.',
  '/styleguide':
    'Retirée du routeur hors développement, comme la Swagger UI du cycle 001 : elle n’existe que '
    + 'sous `KAYA_STYLEGUIDE=1`. Une tuile la promettrait en production, où la route rend 404.',
}

/**
 * Les tuiles **visibles** pour cet utilisateur, dans cet établissement.
 *
 * @param permissions Union des rôles portés — jamais celles d'un rôle « principal ».
 * @param modulesActifs Codes des modules d'activité activés. Une liste vide est **valide** : c'est
 *   l'état d'une résidence meublée, et le traiter comme une erreur ferait échouer l'accueil sur un
 *   état parfaitement normal.
 */
export function tuilesVisibles(
  permissions: Permissions,
  modulesActifs: readonly string[] = [],
): Tuile[] {
  return CATALOGUE_TUILES.filter((tuile) => {
    // **La MÊME question que la page pose à l'accès direct**, et la même réponse : c'est ce qui
    // rend impossible une tuile qui ouvre sur un refus.
    if (!peutOuvrirEcran(tuile.route, permissions)) {
      return false
    }
    // Un module inactif rend la tuile **absente**, jamais grisée (principe VII).
    return !tuile.moduleRequis || modulesActifs.includes(tuile.moduleRequis)
  })
}

/**
 * Ce compte s'est-il vu accorder **quoi que ce soit** ?
 *
 * Le cas existe et n'est pas une erreur : un compte fraîchement créé, avant qu'on lui donne un
 * rôle. Il obtient une **explication**, pas un écran blanc ni un message d'échec — il n'y a rien à
 * réessayer, il y a quelqu'un à prévenir.
 *
 * # ⚠️ CE N'EST PLUS « L'ACCUEIL EST VIDE », ET LE RENOMMAGE EST LE POINT
 *
 * La fonction s'appelait `accueilVide` et comptait les tuiles rendues. Depuis que « Mes envois »
 * existe — la seule tuile sans permission du produit —, ce décompte ne vaut **jamais** zéro pour
 * une session ouverte : un compte sans aucun rôle aurait obtenu une tuile solitaire et **aucune
 * explication**, ce qui est le pire des deux mondes. Il aurait cherché ce qu'il a fait de travers.
 *
 * La question posée est donc redevenue la vraie : *quelqu'un lui a-t-il donné accès à quelque
 * chose ?* Elle porte sur les tuiles **gardées par une permission**, et l'explication s'affiche
 * **en plus** des tuiles ouvertes à toute session, jamais à leur place.
 */
export function aucunAccesAccorde(
  permissions: Permissions,
  modulesActifs: readonly string[] = [],
): boolean {
  return tuilesVisibles(permissions, modulesActifs)
    .every(tuile => (accesDeLEcran(tuile.route)?.permissions.length ?? 0) === 0)
}
