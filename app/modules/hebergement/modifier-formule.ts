/**
 * **L'écriture de `G2`** — le prix d'une formule et son réglage fiscal, selon la septième couche.
 *
 * `docs/module-dore.md`, « La septième couche — le patron d'écriture front », onze points. Ce
 * fichier les tient tous ; ceux qui méritent une note à cet endroit précis sont écrits ci-dessous.
 *
 * ─────────────────────────────────────────────────────────────────────────────────────────────
 *
 * # 1 · CLASSE C — le refus précède l'appel, et il s'explique
 *
 * `docs/registre-classes-offline.md` §7.1 : `formule` est de **classe C**, référentiel fiscal.
 * Trois conséquences, toutes tenues ici :
 *
 * - l'opération **n'entre jamais en file** — une opération de classe C n'a aucune garantie de
 *   rejeu, et la file la rejouerait sans que rien ne la déduplique ;
 * - hors ligne, l'interface **dit immédiatement** que l'action demande le réseau. Pas de grisé
 *   silencieux : l'utilisateur ne saurait pas pourquoi, et l'apprendrait en cliquant dans le vide ;
 * - la garde vit **ici**, pas dans le composant. Un second appelant oublierait de la reposer, et
 *   la faute ne se verrait qu'en clientèle.
 *
 * `navigator.onLine` dit qu'une interface réseau est active, pas que le serveur répond. À
 * Abengourou, une 3G qui affiche « en ligne » sans porter la moindre requête est le cas courant :
 * **la garde ne dispense donc pas du traitement d'erreur**, elle évite l'attente inutile.
 *
 * # 2 · Le message est traduit du `code`, jamais du `message`
 *
 * `CorpsErreur` porte trois choses distinctes. `message` est un diagnostic en anglais technique qui
 * nomme des tables : l'afficher mettrait `hebergement.formule` sous les yeux de l'exploitant. La
 * table {@link CLES_DE_REFUS} est **explicite et fermée** — un code inconnu tombe sur une phrase
 * honnête et générique plutôt que sur une clé i18n affichée en brut.
 *
 * # 3 · Ce que cette opération NE PERMET PAS de changer, et pourquoi
 *
 * Ni `famille`, ni `categorie_id`. Changer la famille d'une formule reviendrait à transformer une
 * nuitée en passage en gardant son identifiant : les occupations déjà attribuées désigneraient une
 * formule dont le sens a changé, et le montant dû sur un séjour en cours changerait sous les pieds
 * de l'exploitant. Le contrat ne les porte pas ; ce fichier ne les invente pas.
 */

import { creerClientKaya, type components } from '@kaya/client'

import { enTetesAuth, type ContexteAppel } from '~/core/auth'
import type { EtatReseau } from '~/core/platform'

/**
 * Permission requise — **une ligne du référentiel `comptes.permission`**, posée par la migration
 * `0022`, portée par `proprietaire` et `gerant`, refusée au `receptionniste`.
 *
 * Yao attribue des chambres, il ne fixe pas les tarifs. La règle d'affichage : permission absente
 * → action **absente**, jamais grisée.
 */
export const PERMISSION_GERER = 'heb.offre.gerer'

/** Permission de lecture — sans elle, l'écran entier est absent de l'accueil. */
export const PERMISSION_LIRE = 'heb.offre.lire'

/**
 * Type d'opération, au vocabulaire du registre hors-ligne.
 *
 * **Il n'est PAS dans `TYPES_CLASSE_A`**, et ce n'est pas un oubli : l'y mettre autoriserait la
 * mise en file d'une opération de classe C, ce que la porte P-13 refuse.
 */
export const TYPE_OPERATION = 'hebergement_formule.modification'

/** La règle de conversion de la taxe, telle que le contrat la nomme. */
export type RegleConversionTaxe = components['schemas']['RegleConversionTaxe']

/** Ce que l'écran envoie. */
export interface ChangementsFormule {
  prixMineur: number
  assujettieTaxeNuitee: boolean
  /** `null` **seulement** sur une formule non assujettie — la base le garantit. */
  regleConversionTaxe: RegleConversionTaxe | null
}

/** Ce qu'une modification produit. Un seul type de retour, jamais d'exception à rattraper au vol. */
export type ResultatModification =
  | { issue: 'succes' }
  | {
    issue: 'refus'
    /** Clé i18n du message à afficher. **Toujours renseignée.** */
    cle: string
    valeurs?: Record<string, unknown>
    /** Le refus vient-il de l'absence de réseau ? L'interface ne le rend pas de la même façon. */
    reseau?: boolean
  }

/**
 * Codes du contrat → clés i18n. **Table explicite et fermée.**
 *
 * Chaque entrée correspond à un code réellement produit par
 * `backend/crates/verticales/hebergement/src/referentiel/modele.rs`.
 */
const CLES_DE_REFUS: Record<string, string> = {
  formule_inconnue: 'hebergement.offre.refus.formule_inconnue',
  categorie_inconnue: 'hebergement.offre.refus.categorie_inconnue',
  etablissement_inconnu: 'hebergement.offre.refus.etablissement_inconnu',
  service_inactif: 'hebergement.offre.refus.service_inactif',
  bareme_absent: 'hebergement.offre.refus.bareme_absent',
  plages_absentes: 'hebergement.offre.refus.plages_absentes',
  famille_inconnue: 'hebergement.offre.refus.famille_inconnue',
  regle_conversion_inconnue: 'hebergement.offre.refus.regle_conversion_inconnue',
}

const REFUS_INATTENDU = 'hebergement.offre.refus.inattendue'
const REFUS_PERMISSION = 'hebergement.offre.refus.permission'
const REFUS_RESEAU = 'hebergement.offre.refus.reseau'

/** Le corps d'erreur du contrat, réduit à ce que l'interface en consomme. */
interface CorpsErreur {
  code?: string
  motif_cle?: string | null
  valeur?: string | null
}

/**
 * **Validation au champ** — un prix négatif se corrige là où on le saisit.
 *
 * Rend une clé i18n, jamais une phrase : le composant 16 l'affiche avec ses trois signaux —
 * bordure `danger`, message, icône d'avertissement.
 *
 * Elle vit ici, à côté de l'appel, plutôt que dans le composant : un second écran qui modifierait
 * un prix ne réécrirait pas la règle, et surtout ne l'écrirait pas différemment.
 */
export function validerPrix(saisie: string): string | null {
  const nettoye = saisie.trim()
  if (nettoye === '') {
    return 'champ.erreur.obligatoire'
  }
  // **Entier d'unité mineure** (principe V) : ni décimale, ni signe, ni espace. XOF en a zéro,
  // et accepter « 12 500,50 » ici produirait un montant tronqué en base sans que rien ne le dise.
  if (!/^\d+$/.test(nettoye)) {
    return 'hebergement.offre.erreur.prix_entier'
  }
  return null
}

/**
 * Modifie le prix et le réglage fiscal d'une formule.
 *
 * @param formule L'état **courant** de la formule, tel que le serveur l'a rendu. La modification
 *   est un **remplacement complet** au contrat : les champs qu'on ne touche pas doivent être
 *   renvoyés tels quels, sans quoi ils seraient effacés. Les prendre de l'état serveur — plutôt
 *   que de les reconstruire — est ce qui empêche l'écran de perdre un barème qu'il n'affiche pas.
 * @param reseau État réseau **au moment du geste**, lu depuis `PlatformAdapter`. Passé en
 *   paramètre plutôt que lu ici : la fonction reste testable sans navigateur, et l'appelant est
 *   obligé de constater qu'il y a une question d'état réseau à poser.
 */
export async function modifierFormule(
  contexte: ContexteAppel,
  etablissementId: string,
  formule: components['schemas']['FormuleVue'],
  changements: ChangementsFormule,
  reseau: EtatReseau,
): Promise<ResultatModification> {
  // CLASSE C — le refus est immédiat, et il précède l'appel. Pas de mise en file « au cas où » :
  // promettre un envoi qu'on ne sait pas rejouer est pire que dire non (principe VI).
  if (reseau !== 'connecte') {
    return { issue: 'refus', cle: REFUS_RESEAU, reseau: true }
  }

  const client = creerClientKaya(contexte.baseUrl)

  const reponse = await client.PUT(
    '/api/v1/etablissements/{etablissement_id}/hebergement/formules/{formule_id}',
    {
      params: {
        path: { etablissement_id: etablissementId, formule_id: formule.id },
      },
      body: {
        prix_mineur: changements.prixMineur,
        assujettie_taxe_nuitee: changements.assujettieTaxeNuitee,
        regle_conversion_taxe: changements.regleConversionTaxe,
        // **Renvoyés tels quels.** Le `PUT` est un remplacement complet : omettre le barème
        // l'effacerait, et une formule `PASSAGE` sans palier est refusée par le serveur (FR-025)
        // — donc le refus serait juste, et l'utilisateur ne comprendrait pas pourquoi changer un
        // prix casse son barème.
        duree_min_minutes: formule.duree_min_minutes,
        duree_max_minutes: formule.duree_max_minutes,
        heure_arrivee_standard: formule.heure_arrivee_standard,
        heure_depart_standard: formule.heure_depart_standard,
        jours_autorises: formule.jours_autorises,
        prix_heure_supplementaire_mineur: formule.prix_heure_supplementaire_mineur,
        paliers: formule.paliers.map(p => ({
          duree_minutes: p.duree_minutes,
          prix_mineur: p.prix_mineur,
        })),
        plages: formule.plages.map(p => ({
          heure_debut: p.heure_debut,
          heure_fin: p.heure_fin,
          libelle_cle: p.libelle_cle,
        })),
      },
      headers: enTetesAuth(contexte),
    },
  )

  if (!reponse.error) {
    return { issue: 'succes' }
  }

  // `403` : l'absence de permission ne se diagnostique pas, elle se constate. En pratique
  // l'utilisateur ne devrait jamais la voir — l'action lui est ABSENTE. Le message existe pour le
  // cas où ses droits changent pendant qu'il regarde l'écran, seul chemin qui y mène.
  if (reponse.response.status === 403) {
    return { issue: 'refus', cle: REFUS_PERMISSION }
  }

  const corps = reponse.error as CorpsErreur
  const cle = corps.motif_cle
    || (corps.code && CLES_DE_REFUS[corps.code])
    || REFUS_INATTENDU

  return { issue: 'refus', cle, valeurs: { valeur: corps.valeur ?? '' } }
}
