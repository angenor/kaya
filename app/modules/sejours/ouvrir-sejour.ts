/**
 * ★ **L'écriture du passage** — septième couche du module doré, appliquée à l'opération qui
 * décide du produit.
 *
 * # Un seul appel réseau bloquant, et c'est ce fichier qui le tient
 *
 * FR-031 borne le parcours à **au plus un appel bloquant** entre le premier geste et la
 * confirmation. Tout ce qui pourrait être demandé au serveur ici — vérifier que la chambre est
 * encore libre, calculer le tarif, réserver un numéro — est **fait par le serveur dans la même
 * transaction** (`POST .../sejours`). Ajouter une vérification préalable côté écran doublerait
 * l'appel **et** reproduirait le verrou applicatif que le principe IV refuse : entre la
 * vérification et l'attribution, une autre réception peut prendre la chambre.
 *
 * # ★ CLASSE B — le refus hors ligne est IMMÉDIAT, et il précède l'appel
 *
 * `sejour` est de classe **B** (registre §7.3) : une ressource unique, sérialisée. Pas de mise en
 * file « au cas où » — **promettre un envoi qu'on ne sait pas rejouer est pire que dire non**
 * (principe VI). Le refus est annoncé **avant** la saisie, pas après un échec, et il est
 * **explicite**, jamais un grisé silencieux.
 *
 * # L'identifiant est engendré ICI, avant l'appel
 *
 * UUID v7 côté client (FR-086) : c'est lui qui rend le rejeu inoffensif. Si le réseau tombe entre
 * l'envoi et la réponse, un second envoi du **même** identifiant rend `200` avec la ligne en base
 * — jamais `409`, jamais un second séjour. Le serveur déduplique ; il n'engendre pas.
 */

import { clientKaya } from '~/core/api/client'
import { enTetesAuth, type ContexteAppel } from '~/core/auth'
import { uuidV7 } from '~/core/sync/uuid-v7'
import type { EtatReseau } from '~/core/platform'
import type { SejourOuvert } from './donnees'

/**
 * Type d'opération — **absent de `TYPES_CLASSE_A`**, et ce n'est pas un oubli.
 *
 * L'y inscrire mettrait l'ouverture d'un séjour en file locale, ce que le registre interdit : deux
 * terminaux vidant leur file attribueraient la même chambre, et la contrainte d'exclusion
 * refuserait la seconde **après coup** — le client serait déjà dans la chambre.
 */
export const TYPE_OPERATION_SEJOUR = 'hebergement_sejour.ouverture'

/** Ce qu'une écriture produit. Un seul type de retour, jamais d'exception à rattraper au vol. */
export type ResultatOuverture =
  | { issue: 'succes', sejour: SejourOuvert }
  | { issue: 'refus', cle: string, valeurs?: Record<string, unknown>, reseau?: boolean }

/**
 * Les refus **traduits par le lexique** (`docs/design/lexique.md` v1.6.0).
 *
 * L'interface branche sa clé i18n sur le **code**, jamais sur le `message` — qui nomme des tables
 * et parle anglais technique (règle du cycle 002).
 */
const CLES_DE_REFUS: Record<string, string> = {
  unite_deja_occupee: 'sejours.passage.refus.unite_deja_occupee',
  service_inactif: 'sejours.passage.refus.service_inactif',
  etablissement_inconnu: 'sejours.passage.refus.etablissement_inconnu',
  client_inconnu: 'sejours.passage.refus.client_inconnu',
  formule_hors_categorie: 'sejours.passage.refus.formule_hors_categorie',
  plage_non_fractionnable: 'sejours.passage.refus.plage_non_fractionnable',
  intervalle_invalide: 'sejours.passage.refus.intervalle_invalide',
  duree_hors_contrainte: 'sejours.passage.refus.duree_hors_contrainte',
}

const REFUS_INATTENDU = 'sejours.passage.refus.inattendue'
const REFUS_PERMISSION = 'sejours.passage.refus.permission'
const REFUS_RESEAU = 'sejours.passage.refus.reseau'

interface CorpsErreur {
  code?: string
  motif_cle?: string | null
  valeur?: string | null
}

/** Ce que l'écran fournit — **deux gestes, aucune saisie libre obligatoire**. */
export interface DemandeOuverture {
  etablissementId: string
  uniteId: string
  formuleId: string
  /** Instant d'autorité du serveur, jamais l'horloge du terminal. */
  debutClient: Date
  /** Dérivé du palier touché — la durée, jamais une saisie. */
  finClient: Date
  /** **Absent pour un passage** : la pièce vient après la clé (FR-023). */
  clientId?: string
  /** Un nom suffit par accompagnant (FR-015). */
  accompagnants?: { nom: string, prenoms?: string }[]
}

/**
 * Ouvre un séjour.
 *
 * L'ordre des trois étapes, et chacune pour une raison :
 *
 * 1. **refus hors ligne, AVANT l'appel** — classe B, principe VI ;
 * 2. **un seul appel**, celui qui fait les cinq écritures dans une transaction ;
 * 3. traduction du refus **par le code**, jamais par le message.
 */
export async function ouvrirSejour(
  contexte: ContexteAppel,
  reseau: EtatReseau,
  demande: DemandeOuverture,
): Promise<ResultatOuverture> {
  // ── 1 · CLASSE B — le refus précède l'appel ───────────────────────────────────────────────
  if (reseau !== 'connecte') {
    return { issue: 'refus', cle: REFUS_RESEAU, reseau: true }
  }

  const client = clientKaya(contexte.baseUrl)

  // ── 2 · UN SEUL APPEL ─────────────────────────────────────────────────────────────────────
  const reponse = await client.POST(
    '/api/v1/etablissements/{etablissement_id}/sejours',
    {
      params: { path: { etablissement_id: demande.etablissementId } },
      body: {
        // **Engendré ici**, avant l'appel : c'est ce qui rend le rejeu inoffensif.
        id: uuidV7(),
        unite_id: demande.uniteId,
        formule_id: demande.formuleId,
        debut_client: demande.debutClient.toISOString(),
        fin_client: demande.finClient.toISOString(),
        ...(demande.clientId === undefined ? {} : { client_id: demande.clientId }),
        accompagnants: (demande.accompagnants ?? []).map((a) => ({
          id: uuidV7(),
          nom: a.nom,
          ...(a.prenoms === undefined ? {} : { prenoms: a.prenoms }),
        })),
      },
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
  // (FR-026). Le message existe pour le cas où ses droits changent pendant qu'il regarde, seul
  // chemin qui y mène.
  if (reponse.response.status === 403) {
    return { issue: 'refus', cle: REFUS_PERMISSION }
  }

  const corps = reponse.error as CorpsErreur
  const cle = corps.motif_cle
    || (corps.code && CLES_DE_REFUS[corps.code])
    || REFUS_INATTENDU

  return { issue: 'refus', cle, valeurs: { valeur: corps.valeur ?? '' } }
}

/**
 * Rattache une fiche client à un séjour **déjà ouvert** — « la pièce après la clé ».
 *
 * ⚠️ **Ne rouvre pas le séjour et ne remet pas en cause l'attribution** (FR-028). C'est le
 * parcours normal du passage : Yao donne la clé, puis saisit la pièce quand il a une minute. La
 * fiche de police passe de « Identité à compléter » à complète.
 *
 * Classe **B** comme l'ouverture : le refus hors ligne précède l'appel.
 */
export async function rattacherClient(
  contexte: ContexteAppel,
  reseau: EtatReseau,
  etablissementId: string,
  sejourId: string,
  clientId: string,
): Promise<ResultatOuverture | { issue: 'succes-sans-corps' }> {
  if (reseau !== 'connecte') {
    return { issue: 'refus', cle: REFUS_RESEAU, reseau: true }
  }

  const client = clientKaya(contexte.baseUrl)
  const reponse = await client.POST(
    '/api/v1/etablissements/{etablissement_id}/sejours/{sejour_id}/client',
    {
      params: { path: { etablissement_id: etablissementId, sejour_id: sejourId } },
      body: { client_id: clientId },
      headers: enTetesAuth(contexte),
    },
  )

  if (!reponse.error) {
    return { issue: 'succes-sans-corps' }
  }
  if (reponse.response.status === 403) {
    return { issue: 'refus', cle: REFUS_PERMISSION }
  }

  const corps = reponse.error as CorpsErreur
  const cle = corps.motif_cle
    || (corps.code && CLES_DE_REFUS[corps.code])
    || REFUS_INATTENDU
  return { issue: 'refus', cle }
}
