/**
 * **La note interne — le premier passager réel de la file hors-ligne.**
 *
 * # Pourquoi c'est elle, et pourquoi elle seule
 *
 * `note_etablissement.creee` est la **seule** opération que `TYPES_CLASSE_A` déclare, et elle l'est
 * depuis le cycle 001. Sa table n'accorde ni `UPDATE` ni `DELETE` à `kaya_app` : une correction est
 * une note nouvelle. Elle est append-only, commutative, sans contrainte d'unicité métier et sans
 * effet monétaire — branche **A4** de l'arbre de décision du cadrage §11.2.
 *
 * Un mécanisme sans passager réel est du code exporté et appelé nulle part — le défaut exact
 * d'`initialiserTheme()`, qui a vécu deux cycles. La file a donc son écran, et c'est celui-là.
 *
 * # Le patron d'écriture, appliqué à une opération de CLASSE A
 *
 * `docs/module-dore.md` § « La septième couche » décrit le patron sur une opération de classe C
 * (la bascule d'un service). **Une opération de classe A l'inverse sur un point, et c'est le
 * point** :
 *
 * | | Classe C — bascule de service | Classe A — note interne |
 * |---|---|---|
 * | Hors ligne | Refus **avant la saisie** | **Acceptée**, mise en file, aucun message d'erreur |
 * | Témoin | inchangé | passe à `n+1` |
 * | Au retour du réseau | rien à rejouer | la file part, rafraîchissement d'abord |
 *
 * C'est la distinction que le principe VI porte, et la confondre donnerait soit un produit qui
 * refuse une note pendant une coupure, soit une file qui transporte un encaissement.
 *
 * # Le rejeu est inoffensif, et c'est l'identifiant client qui le rend tel
 *
 * L'UUID v7 est engendré **ici**, avant l'envoi. Trois envois du même identifiant produisent
 * **une** ligne et **un** événement outbox — jamais trois. Le serveur rend `201` la première fois,
 * `200` ensuite, avec la ligne telle qu'elle est en base : `200` est le **chemin normal d'un
 * rejeu**, jamais une erreur.
 */

import type { components } from '@kaya/client'

import { clientKaya } from '~/core/api/client'
import { enTetesAuth, type ContexteAppel } from '~/core/auth'
import {
  comparerAHorodatageAutorite,
  marquerClasseA,
  uuidV7,
  type ContexteEcriture,
  type EntreeFile,
} from '~/core/sync'

/** Une note, telle que le contrat la rend. **Alias du contrat, jamais une copie.** */
export type Note = components['schemas']['NoteEtablissement']

/** Une page de notes. */
export type PageNotes = components['schemas']['PageNotes']

/** Le type d'opération, au vocabulaire du registre hors-ligne. */
export const TYPE_NOTE_CREEE = 'note_etablissement.creee'

/** La charge d'une note en attente d'envoi. */
export interface ChargeNote {
  readonly texte: string
}

/**
 * Compose l'entrée de file d'une note.
 *
 * # `marquerClasseA` est le seul point d'entrée, et sa justification n'est pas décorative
 *
 * Elle force à nommer la branche de l'arbre de décision du cadrage §11.2. C'est le moment — le
 * seul — où la question « cette opération est-elle vraiment de classe A ? » se pose, et un appel
 * sans justification recevable se voit en revue.
 *
 * # Le contexte est FIGÉ ici, pas relu à l'envoi
 *
 * Changer d'établissement actif pendant une coupure ne doit jamais réattribuer une écriture déjà
 * saisie. Voir {@link ContexteEcriture} — la faute serait silencieuse et impossible à démêler.
 */
export function composerEntreeNote(
  texte: string,
  contexte: ContexteEcriture,
): EntreeFile<ChargeNote> {
  return {
    id: uuidV7(),
    type: TYPE_NOTE_CREEE,
    // Horloge **locale**, et c'est correct : cet horodatage est indicatif, il sert l'ordre
    // d'affichage local et rien d'autre. L'autorité est `cree_le`, posé par le serveur
    // (principe IV, porte P-23).
    horodatageClient: new Date().toISOString(),
    charge: marquerClasseA(
      { texte },
      'A4 — append-only, commutative, sans contrainte d’unicité métier, sans effet monétaire',
    ),
    contexte,
    tentatives: 0,
  }
}

/**
 * Envoie une entrée de note. **Rend `true` si elle est acquittée et peut quitter la file.**
 *
 * # Ce que la fonction NE fait pas, et pourquoi
 *
 * Elle ne rafraîchit pas la session, ne décide pas de réessayer, ne met rien en quarantaine. Ces
 * trois décisions appartiennent à `viderFile` et à `quarantaine.ts`, qui les portent pour **toutes**
 * les opérations de classe A. Les réimplémenter ici donnerait une seconde version de la frontière
 * `4xx` / `5xx`, et la seconde version serait fausse.
 *
 * # Le contexte de l'entrée l'emporte sur celui de l'appel
 *
 * `contexte` porte le jeton, frais, obtenu par le rafraîchissement qui vient d'avoir lieu.
 * `entree.contexte` porte l'établissement, figé à la saisie. Les deux sont nécessaires et ne
 * disent pas la même chose — c'est exactement pourquoi le second existe.
 */
export async function envoyerNote(
  entree: EntreeFile,
  contexte: ContexteAppel,
): Promise<{ acquittee: boolean, statut: number | null, code: string }> {
  const charge = entree.charge as unknown as ChargeNote
  const client = clientKaya(contexte.baseUrl)

  try {
    const { data, response } = await client.POST(
      '/api/v1/etablissements/{etablissement_id}/notes',
      {
        params: { path: { etablissement_id: entree.contexte.etablissementId } },
        headers: enTetesAuth(contexte),
        body: {
          id: entree.id,
          texte: charge.texte,
          horodatage_client: entree.horodatageClient,
        },
      },
    )

    // **L'horodatage d'AUTORITÉ, comparé à l'horloge locale.** Aucun endpoint d'heure serveur
    // n'existe et il n'en faut pas : chaque réponse de création porte déjà `cree_le`. Un endpoint
    // dédié mesurerait l'aller-retour réseau autant que l'écart d'horloge, et sur une 3G
    // d'Abengourou l'aller-retour est la plus grande des deux valeurs.
    if (data?.cree_le) {
      comparerAHorodatageAutorite(data.cree_le)
    }

    return {
      // **`200` acquitte autant que `201`** — c'est un rejeu réussi, pas un conflit. Une file qui
      // le traiterait comme une erreur remettrait l'écriture en tête et boucllerait.
      acquittee: response.status === 200 || response.status === 201,
      statut: response.status,
      code: codeDepuisStatut(response.status),
    }
  }
  catch {
    // Réseau injoignable, délai dépassé : **rien n'a été décidé côté serveur**. `null` le dit, et
    // `classer()` en déduit qu'il faut réessayer — jamais mettre en quarantaine.
    return { acquittee: false, statut: null, code: 'reseau' }
  }
}

/**
 * Le code de refus, déduit du statut.
 *
 * Les quatre réponses d'erreur de `notes_creer` **ne portent aucun corps** — le contrat le dit :
 * `content?: never` sur 400, 401, 403 et 404. Il n'y a donc pas de `code` à lire, et en inventer
 * un depuis le `message` serait doublement faux, puisqu'il n'y a pas de message non plus.
 *
 * Le statut est la seule information disponible ; la table est explicite et rend un code stable,
 * sur lequel l'interface branche sa clé i18n comme pour n'importe quel refus.
 */
function codeDepuisStatut(statut: number): string {
  switch (statut) {
    case 400:
      return 'validation'
    case 403:
      return 'permission_refusee'
    case 404:
      return 'etablissement_inconnu'
    case 409:
      return 'conflit'
    case 422:
      return 'validation'
    default:
      return 'inconnu'
  }
}

/** Charge les notes d'un établissement — la lecture de l'écran. */
export async function chargerNotes(
  contexte: ContexteAppel,
  etablissementId: string,
): Promise<PageNotes> {
  const client = clientKaya(contexte.baseUrl)
  const { data, error } = await client.GET(
    '/api/v1/etablissements/{etablissement_id}/notes',
    {
      params: { path: { etablissement_id: etablissementId } },
      headers: enTetesAuth(contexte),
    },
  )

  if (error || !data) {
    throw new Error('notes_illisibles')
  }
  return data
}
