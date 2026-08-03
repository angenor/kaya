/**
 * ★ **Le départ** — septième couche du module doré, appliquée à l'opération qui ARRÊTE la note.
 *
 * # ★ CLASSE B — le refus hors ligne est IMMÉDIAT, et il précède l'appel
 *
 * `sejour` — check-out — est de classe **B** (registre §7.3, ligne « taxe de nuitée **figée** ») :
 * il clôt la note, libère l'unité et **fige un constat de taxe**. Pas de mise en file « au cas
 * où » — **promettre un envoi qu'on ne sait pas rejouer est pire que dire non** (principe VI). Un
 * départ rejoué depuis une file locale figerait un second constat sur des faits périmés, et les
 * deux vaudraient également.
 *
 * # ⚠️ CE QUE CETTE OPÉRATION NE FAIT PAS, ET QUE L'ÉCRAN DOIT DIRE
 *
 * | Ce qui n'arrive pas | Cycle qui le doit |
 * |---|---|
 * | Aucun encaissement | **CAI**, tranche T2 |
 * | Aucune facture, aucune certification fiscale | **FIS**, tranche T3 |
 * | Aucun montant de taxe calculé | **FIS-03** — ce cycle fige des **faits**, pas une assiette |
 *
 * La note se ferme **arrêtée et non réglée**. C'est un état parfaitement valide du produit à ce
 * stade, et il **doit être dit en toutes lettres** : un écran qui afficherait « Faire partir le
 * client » sans plus laisserait croire que le paiement est enregistré, et l'exploitant découvrirait
 * le trou au comptage de caisse.
 *
 * # Le rejeu est inoffensif, et il n'y a rien à engendrer
 *
 * Contrairement à l'ouverture, le départ ne crée pas d'agrégat : il en **transitionne** un dont
 * l'identifiant existe déjà. Un second envoi rend `409 sejour_deja_clos`, qui n'est pas une erreur
 * de saisie mais un **constat** — et l'écran le traduit en « Ce séjour est déjà terminé. »
 */

import { clientKaya } from '~/core/api/client'
import { enTetesAuth, type ContexteAppel } from '~/core/auth'
import type { EtatReseau } from '~/core/platform'
import type { SejourOuvert } from './donnees'

/**
 * Type d'opération — **absent de `TYPES_CLASSE_A`**, et ce n'est pas un oubli.
 *
 * L'y inscrire mettrait le départ en file locale, ce que le registre interdit : deux terminaux
 * vidant leur file figeraient deux constats de taxe sur le même séjour.
 */
export const TYPE_OPERATION_DEPART = 'hebergement_sejour.depart'

export type ResultatDepart =
  | { issue: 'succes', sejour: SejourOuvert }
  | { issue: 'refus', cle: string, valeurs?: Record<string, unknown>, reseau?: boolean }

/**
 * Les refus **traduits par le lexique** (`docs/design/lexique.md` v1.6.0).
 *
 * L'interface branche sa clé i18n sur le **code**, jamais sur le `message` — qui nomme des tables
 * et parle anglais technique (règle du cycle 002).
 */
const CLES_DE_REFUS: Record<string, string> = {
  sejour_deja_clos: 'sejours.depart.refus.sejour_deja_clos',
  sejour_inconnu: 'sejours.depart.refus.sejour_inconnu',
  service_inactif: 'sejours.depart.refus.service_inactif',
  etablissement_inconnu: 'sejours.depart.refus.etablissement_inconnu',
}

const REFUS_INATTENDU = 'sejours.depart.refus.inattendue'
const REFUS_PERMISSION = 'sejours.depart.refus.permission'
const REFUS_RESEAU = 'sejours.depart.refus.reseau'

interface CorpsErreur {
  code?: string
  motif_cle?: string | null
  valeur?: string | null
}

/**
 * Clôt un séjour : la note s'arrête, l'unité se libère, le constat de taxe se fige.
 *
 * L'ordre des trois étapes, et chacune pour une raison :
 *
 * 1. **refus hors ligne, AVANT l'appel** — classe B, principe VI ;
 * 2. **un seul appel**, celui qui fait les six écritures dans une transaction ;
 * 3. traduction du refus **par le code**, jamais par le message.
 */
export async function cloreSejour(
  contexte: ContexteAppel,
  reseau: EtatReseau,
  etablissementId: string,
  sejourId: string,
): Promise<ResultatDepart> {
  // ── 1 · CLASSE B — le refus précède l'appel ───────────────────────────────────────────────
  if (reseau !== 'connecte') {
    return { issue: 'refus', cle: REFUS_RESEAU, reseau: true }
  }

  const client = clientKaya(contexte.baseUrl)

  // ── 2 · UN SEUL APPEL ─────────────────────────────────────────────────────────────────────
  const reponse = await client.POST(
    '/api/v1/etablissements/{etablissement_id}/sejours/{sejour_id}/depart',
    {
      params: { path: { etablissement_id: etablissementId, sejour_id: sejourId } },
      headers: enTetesAuth(contexte),
    },
  )

  if (!reponse.error && reponse.data) {
    return { issue: 'succes', sejour: reponse.data }
  }

  // ── 3 · LA TRADUCTION ─────────────────────────────────────────────────────────────────────
  //
  // `403` : l'absence de permission ne se diagnostique pas, elle se constate. En pratique
  // l'utilisateur ne devrait **jamais** la voir — l'action lui est **absente** de l'écran
  // (FR-026). Le message existe pour le cas où ses droits changent pendant qu'il regarde.
  if (reponse.response.status === 403) {
    return { issue: 'refus', cle: REFUS_PERMISSION }
  }

  const corps = reponse.error as CorpsErreur
  const cle = corps.motif_cle
    || (corps.code && CLES_DE_REFUS[corps.code])
    || REFUS_INATTENDU

  return { issue: 'refus', cle, valeurs: { valeur: corps.valeur ?? '' } }
}
