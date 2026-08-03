/**
 * **L'écriture de `G5`** — créer et corriger une chambre, créer un type de chambre.
 *
 * Même patron que `modifier-formule.ts`, septième couche du module doré. Ce qui suit ne redit que
 * ce qui est propre à ce fichier.
 *
 * # Ce que la correction d'une chambre NE PERMET PAS de changer
 *
 * Trois champs, et chacun pour une raison différente :
 *
 * | Champ | Pourquoi il n'est pas ici |
 * |---|---|
 * | `categorie_id` | Changer le type change les formules applicables, **donc les tarifs**. Effet fiscal que le registre des classes ne classe **nulle part** : ça se spécifie, ça ne se glisse pas dans une correction |
 * | `statut_menage` | **Classe A**, dernier-écrit-gagne — HEB-06. Autre chemin, autre classe, autre cycle |
 * | Mise hors service | **Classe B** — elle retire une ressource de la disponibilité : c'est une opération de disponibilité, pas de référentiel |
 *
 * Le serveur les **refuse nommément** s'ils apparaissent dans le corps ; ce fichier ne les envoie
 * pas. Les deux, et pas l'un à la place de l'autre.
 *
 * # Pourquoi une correction, et pas seulement une création
 *
 * Sans elle, **une chambre mal nommée puis occupée deviendrait définitive** : la suppression est
 * impossible dès qu'une occupation la référence. Un écran de gestion qui ne saurait pas corriger
 * un numéro de chambre n'est pas un écran de gestion.
 */


import { clientKaya } from '~/core/api/client'
import { enTetesAuth, type ContexteAppel } from '~/core/auth'
import { uuidV7 } from '~/core/sync/uuid-v7'
import type { EtatReseau } from '~/core/platform'

/** Type d'opération — **absent de `TYPES_CLASSE_A`** : `unite` est de classe C (registre §7.1). */
export const TYPE_OPERATION_UNITE = 'hebergement_unite.ecriture'

/** Ce qu'une écriture produit. Un seul type de retour, jamais d'exception à rattraper au vol. */
export type ResultatEcriture =
  | { issue: 'succes' }
  | { issue: 'refus', cle: string, valeurs?: Record<string, unknown>, reseau?: boolean }

const CLES_DE_REFUS: Record<string, string> = {
  unite_inconnue: 'hebergement.chambres.refus.unite_inconnue',
  categorie_inconnue: 'hebergement.chambres.refus.categorie_inconnue',
  etablissement_inconnu: 'hebergement.chambres.refus.etablissement_inconnu',
  service_inactif: 'hebergement.chambres.refus.service_inactif',
  champ_non_modifiable: 'hebergement.chambres.refus.champ_non_modifiable',
}

const REFUS_INATTENDU = 'hebergement.chambres.refus.inattendue'
const REFUS_PERMISSION = 'hebergement.chambres.refus.permission'
const REFUS_RESEAU = 'hebergement.chambres.refus.reseau'

interface CorpsErreur {
  code?: string
  motif_cle?: string | null
  valeur?: string | null
}

function traduire(statut: number, corps: CorpsErreur): ResultatEcriture {
  if (statut === 403) {
    return { issue: 'refus', cle: REFUS_PERMISSION }
  }
  const cle = corps.motif_cle || (corps.code && CLES_DE_REFUS[corps.code]) || REFUS_INATTENDU
  return { issue: 'refus', cle, valeurs: { valeur: corps.valeur ?? '' } }
}

/**
 * **Validation au champ** du numéro de chambre.
 *
 * Rend une clé i18n, jamais une phrase : le composant 16 l'affiche avec ses trois signaux —
 * bordure `danger`, message, icône d'avertissement.
 */
export function validerCode(saisie: string): string | null {
  return saisie.trim() === '' ? 'hebergement.chambres.erreur.code_obligatoire' : null
}

/**
 * **Validation au champ** de l'étage. Vide est valide — c'est le rez-de-chaussée.
 *
 * `null` dit « pas d'étage », `0` dirait « rez-de-chaussée numéroté zéro » : deux faits
 * différents, deux valeurs, et c'est la colonne `SMALLINT NULL` qui les distingue.
 */
export function validerEtage(saisie: string): string | null {
  const nettoye = saisie.trim()
  if (nettoye === '') {
    return null
  }
  return /^-?\d+$/.test(nettoye) ? null : 'hebergement.chambres.erreur.etage_entier'
}

/** L'étage tel que le contrat l'attend — `null` quand la saisie est vide. */
export function etageDepuisSaisie(saisie: string): number | null {
  const nettoye = saisie.trim()
  return nettoye === '' ? null : Number(nettoye)
}

/** Crée une chambre. */
export async function creerUnite(
  contexte: ContexteAppel,
  etablissementId: string,
  categorieId: string,
  code: string,
  etage: number | null,
  reseau: EtatReseau,
): Promise<ResultatEcriture> {
  // CLASSE C — le refus est immédiat et précède l'appel. Pas de mise en file « au cas où ».
  if (reseau !== 'connecte') {
    return { issue: 'refus', cle: REFUS_RESEAU, reseau: true }
  }

  const client = clientKaya(contexte.baseUrl)
  const reponse = await client.POST(
    '/api/v1/etablissements/{etablissement_id}/hebergement/unites',
    {
      params: { path: { etablissement_id: etablissementId } },
      // UUID v7 **généré côté client** : c'est lui qui rend le rejeu inoffensif. Un double-clic
      // sur « Enregistrer » ne doit pas créer deux chambres.
      body: { id: uuidV7(), categorie_id: categorieId, code, etage },
      headers: enTetesAuth(contexte),
    },
  )

  return reponse.error
    ? traduire(reponse.response.status, reponse.error as CorpsErreur)
    : { issue: 'succes' }
}

/**
 * Corrige une chambre — **`code` et `etage`, et rien d'autre**.
 *
 * Le corps ne porte que ces deux champs. Y ajouter `categorie_id` produirait un `422
 * champ_non_modifiable` nommé par le serveur : le refus existe pour l'appel forgé, pas pour ce
 * chemin-ci.
 */
export async function corrigerUnite(
  contexte: ContexteAppel,
  etablissementId: string,
  uniteId: string,
  code: string,
  etage: number | null,
  reseau: EtatReseau,
): Promise<ResultatEcriture> {
  if (reseau !== 'connecte') {
    return { issue: 'refus', cle: REFUS_RESEAU, reseau: true }
  }

  const client = clientKaya(contexte.baseUrl)
  const reponse = await client.PUT(
    '/api/v1/etablissements/{etablissement_id}/hebergement/unites/{unite_id}',
    {
      params: { path: { etablissement_id: etablissementId, unite_id: uniteId } },
      body: { code, etage },
      headers: enTetesAuth(contexte),
    },
  )

  return reponse.error
    ? traduire(reponse.response.status, reponse.error as CorpsErreur)
    : { issue: 'succes' }
}

/** **Validation au champ** de la capacité d'accueil. */
export function validerCapacite(saisie: string): string | null {
  const nettoye = saisie.trim()
  if (!/^\d+$/.test(nettoye) || Number(nettoye) < 1) {
    return 'hebergement.chambres.erreur.capacite_positive'
  }
  return null
}

/**
 * Crée un type de chambre.
 *
 * **Aucun battement de remise en état n'est envoyé.** Le temps de ménage varie par type ET par
 * formule, et son écran est celui du réglage des formules — pas celui du parc. L'envoyer vide ici
 * ne l'efface pas : c'est une création, il n'y avait rien à effacer.
 */
export async function creerCategorie(
  contexte: ContexteAppel,
  etablissementId: string,
  nom: string,
  capaciteAccueil: number,
  reseau: EtatReseau,
): Promise<ResultatEcriture> {
  if (reseau !== 'connecte') {
    return { issue: 'refus', cle: REFUS_RESEAU, reseau: true }
  }

  const client = clientKaya(contexte.baseUrl)
  const reponse = await client.POST(
    '/api/v1/etablissements/{etablissement_id}/hebergement/categories',
    {
      params: { path: { etablissement_id: etablissementId } },
      body: { id: uuidV7(), nom, capacite_accueil: capaciteAccueil, temps_remise_en_etat: [] },
      headers: enTetesAuth(contexte),
    },
  )

  return reponse.error
    ? traduire(reponse.response.status, reponse.error as CorpsErreur)
    : { issue: 'succes' }
}
