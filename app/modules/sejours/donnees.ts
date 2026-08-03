/**
 * Chargement des données des écrans du cycle SEJ — **par le client typé, jamais par un `fetch`
 * écrit à la main**.
 *
 * `@kaya/client` est généré depuis le contrat OpenAPI (porte P-01) : un chemin ou un champ qui
 * change fait échouer la compilation ici, au lieu de produire un `undefined` à l'exécution que
 * personne ne verrait avant la démonstration.
 *
 * # Les types viennent du contrat, jamais d'une interface qui lui ressemble
 *
 * `components['schemas'][…]`, et pas une copie fidèle. Le module doré le dit après l'avoir payé :
 * *« une copie fidèle le reste jusqu'au premier champ ajouté d'un côté »*. Quatre fichiers du
 * cycle 002 redéclaraient leurs types à la main puis convertissaient par `as unknown as` — la
 * seule construction de TypeScript qui relie deux types sans rapport. **P-01 restait verte** :
 * elle compare le client généré au client commité, et la rupture était un cran plus loin.
 *
 * # ★ Les deux appels de montage de `R4` partent ENSEMBLE
 *
 * L'état des unités et le barème de passage ne dépendent pas l'un de l'autre, et le réseau
 * d'Abengourou fait payer chaque aller-retour séquentiel. Surtout : **ces deux appels se font au
 * montage de l'écran, AVANT le premier geste** — ils ne comptent donc pas dans le budget de
 * FR-031, qui court du premier geste à la confirmation. Les précharger est précisément ce qui
 * permet à l'attribution de n'être qu'un tap.
 */

import type { components } from '@kaya/client'

import { clientKaya } from '~/core/api/client'
import { enTetesAuth, type ContexteAppel } from '~/core/auth'

export type { ContexteAppel }

/** Une formule — **prix entier d'unité mineure, devise au même niveau**. */
export type FormuleVue = components['schemas']['FormuleVue']

/** Un type de chambre. Terme utilisateur : « type de chambre », jamais « catégorie d'unité ». */
export type CategorieVue = components['schemas']['CategorieVue']

/** L'état **dérivé** d'une chambre — `libre` · `occupee` · `remise_en_etat`. */
export type EtatUniteVue = components['schemas']['EtatUniteVue']

/** L'état de toutes les chambres, **et l'instant qui dit quand il était vrai**. */
export type EtatDesUnites = components['schemas']['EtatDesUnites']

/** Un séjour ouvert — tout ce que l'écran affiche après la confirmation, en un seul corps. */
export type SejourOuvert = components['schemas']['SejourOuvert']

/** Un séjour dans une liste, **avec le nom de son client** résolu par le trait. */
export type SejourVue = components['schemas']['SejourVue']

/** Un résumé de fiche client — **il ne porte aucun numéro de pièce**. */
export type ClientResume = components['schemas']['ClientResume']

/** Le résultat d'une recherche, **avec son indication de troncature**. */
export type ResultatRecherche = components['schemas']['ResultatRecherche']

/**
 * Un palier de passage, **prêt à afficher** : sa durée, son prix, et son heure de fin.
 *
 * ★ **L'heure de fin est calculée ici, jamais par le serveur.** C'est ce que la maquette `R4`
 * met sur le bouton — « 2 h · 2 800 F · jusqu'à 17 h 30 » — et c'est la ligne qui fait que Yao
 * n'a rien à calculer de tête. Elle dépend de **l'instant d'autorité** rendu par l'appel d'état
 * des unités, jamais de l'horloge du terminal : un appareil mal réglé afficherait une heure
 * fausse au client, et le client la retiendrait.
 */
export interface PalierAffichable {
  formuleId: string
  dureeMinutes: number
  /** **Entier d'unité mineure** — le formatage passe par `core/format/montant.ts`. */
  prixMineur: number
  devise: string
  /** L'instant de fin, dérivé de l'instant d'autorité. */
  finPrevue: Date
}

/** Ce dont l'écran `R4` a besoin **au montage**, avant le premier geste. */
export interface DonneesPassage {
  etatDesUnites: EtatDesUnites
  categories: CategorieVue[]
  formules: FormuleVue[]
}

/**
 * Ce dont l'écran `R3` (Arrivée) a besoin — **exactement les trois mêmes lectures que `R4`**.
 *
 * Ce n'est pas une coïncidence : la matrice de dérivation fait de `R3` un `R4` **au parcours plus
 * long**, pas un écran d'un autre genre. Lui donner un chargeur distinct dupliquerait trois appels
 * identiques, et les deux divergeraient au premier champ ajouté à l'un des trois contrats.
 */
export type DonneesArrivee = DonneesPassage

/** Charge l'écran d'arrivée. **Même chargeur que `R4`** — voir `DonneesArrivee`. */
export const chargerArrivee = chargerPassage

/**
 * Un accompagnant en cours de déclaration — **local, jamais encore envoyé**.
 *
 * ⚠️ **Ce type vit ici et non dans `ListeAccompagnants.vue`** : un bloc `<script setup>` n'admet
 * aucun `export`, et l'y déclarer ferait échouer la compilation du SFC avec un message qui ne
 * nomme pas la cause.
 */
export interface AccompagnantSaisi {
  /** Clé de rendu locale ; l'UUID v7 réel est engendré à l'envoi par `ouvrir-sejour.ts`. */
  cle: string
  nom: string
  prenoms?: string
}

/** Ce dont l'écran `R7` a besoin. */
export interface DonneesDepart {
  sejours: SejourVue[]
}

/**
 * Charge ce qu'il faut à l'écran de passage — **deux appels, en parallèle**.
 *
 * ⚠️ **Cet appel est de LECTURE.** Hors ligne il échoue, et l'écran doit alors afficher les
 * durées et les prix depuis son cache **avec leur fraîcheur** (maquette `R4-passage-hors-ligne`),
 * pas une page vide. La lecture en cache d'un référentiel est de **classe A** — règle de portée
 * générale du registre, §1.0.2.
 */
export async function chargerPassage(
  contexte: ContexteAppel,
  etablissementId: string,
): Promise<DonneesPassage> {
  const client = clientKaya(contexte.baseUrl)
  const headers = enTetesAuth(contexte)

  const [etat, categories, formules] = await Promise.all([
    client.GET('/api/v1/etablissements/{etablissement_id}/hebergement/etat-des-unites', {
      params: { path: { etablissement_id: etablissementId } },
      headers,
    }),
    client.GET('/api/v1/etablissements/{etablissement_id}/hebergement/categories', {
      params: { path: { etablissement_id: etablissementId } },
      headers,
    }),
    client.GET('/api/v1/etablissements/{etablissement_id}/hebergement/formules', {
      params: { path: { etablissement_id: etablissementId } },
      headers,
    }),
  ])

  if (
    etat.error || !etat.data
    || categories.error || !categories.data
    || formules.error || !formules.data
  ) {
    throw new Error(`état des chambres illisible pour ${etablissementId}`)
  }

  return {
    etatDesUnites: etat.data,
    categories: categories.data,
    formules: formules.data,
  }
}

/**
 * Recharge **le seul état des chambres**, après une écriture.
 *
 * Une requête, pas trois : ni les types de chambre ni les formules n'ont bougé. Et la liste vient
 * du **serveur**, jamais reconstruite à la main côté client — le serveur fait foi en conflit
 * (principe VI). Reconstruire localement « la 103 est maintenant occupée » ferait diverger
 * l'écran de la base dès qu'une autre réception attribue en même temps.
 */
export async function rechargerEtatDesUnites(
  contexte: ContexteAppel,
  etablissementId: string,
): Promise<EtatDesUnites> {
  const client = clientKaya(contexte.baseUrl)
  const reponse = await client.GET(
    '/api/v1/etablissements/{etablissement_id}/hebergement/etat-des-unites',
    {
      params: { path: { etablissement_id: etablissementId } },
      headers: enTetesAuth(contexte),
    },
  )

  if (reponse.error || !reponse.data) {
    throw new Error(`état des chambres illisible pour ${etablissementId}`)
  }
  return reponse.data
}

/** Charge les séjours en cours d'un établissement — écran `R7`. */
export async function chargerSejours(
  contexte: ContexteAppel,
  etablissementId: string,
  enCoursSeulement = true,
): Promise<DonneesDepart> {
  const client = clientKaya(contexte.baseUrl)
  const reponse = await client.GET(
    '/api/v1/etablissements/{etablissement_id}/sejours',
    {
      params: {
        path: { etablissement_id: etablissementId },
        query: { en_cours: enCoursSeulement },
      },
      headers: enTetesAuth(contexte),
    },
  )

  if (reponse.error || !reponse.data) {
    throw new Error(`séjours illisibles pour ${etablissementId}`)
  }
  return { sejours: reponse.data }
}

/**
 * Cherche des fiches clientes — **trois formes servies par une seule entrée**.
 *
 * L'opérateur ne choisit pas un mode : le serveur déduit s'il s'agit d'un nom, d'un téléphone ou
 * d'un numéro de pièce. `tronque` dit qu'il y avait plus de résultats que la limite — une liste
 * silencieusement coupée est un mensonge sur un écran de comptoir.
 */
export async function chercherClients(
  contexte: ContexteAppel,
  recherche: string,
  limite?: number,
): Promise<ResultatRecherche> {
  const client = clientKaya(contexte.baseUrl)
  const reponse = await client.GET('/api/v1/clients', {
    params: { query: { recherche, ...(limite === undefined ? {} : { limite }) } },
    headers: enTetesAuth(contexte),
  })

  if (reponse.error || !reponse.data) {
    throw new Error('recherche de fiches clientes impossible')
  }
  return reponse.data
}

/** Charge l'historique des séjours d'un client — écran `R5`. */
export async function chargerHistoriqueClient(
  contexte: ContexteAppel,
  clientId: string,
): Promise<SejourVue[]> {
  const client = clientKaya(contexte.baseUrl)
  const reponse = await client.GET('/api/v1/clients/{client_id}/sejours', {
    params: { path: { client_id: clientId } },
    headers: enTetesAuth(contexte),
  })

  if (reponse.error || !reponse.data) {
    throw new Error(`historique illisible pour ${clientId}`)
  }
  return reponse.data
}

/**
 * Compose les paliers affichables de la formule de **passage** d'une catégorie.
 *
 * ★ **Le prix et l'heure de fin viennent du barème et de l'instant d'autorité, jamais d'une
 * constante.** La maquette `R4` montre « 2 h · 2 800 F · jusqu'à 17 h 30 » ; les trois valeurs
 * sont dérivées, et écrire l'une d'elles en dur ferait mentir l'écran au premier changement de
 * tarif — sans que rien ne le signale, puisque le bouton continuerait de s'afficher.
 *
 * # Pourquoi l'instant d'autorité et non `new Date()`
 *
 * Un terminal mal réglé afficherait une heure de fin fausse, **et le client la retiendrait**.
 * L'instant vient de la réponse serveur, qui dit *quand* elle était vraie. C'est la même
 * discipline que la porte P-23 impose au backend, appliquée à l'affichage.
 */
export function paliersAffichables(
  formules: FormuleVue[],
  categorieId: string,
  instantAutorite: string,
): PalierAffichable[] {
  const depart = new Date(instantAutorite)

  return formules
    .filter((f) => f.categorie_id === categorieId && f.famille === 'PASSAGE')
    // `paliers` n'est jamais absent au contrat — il est **vide** hors `PASSAGE`, et le filtre
    // ci-dessus a déjà écarté ces cas. Un `?? []` ici laisserait croire à une optionalité que le
    // contrat n'a pas, et masquerait le jour où elle apparaîtrait.
    .flatMap((formule) => formule.paliers.map((palier) => ({
      formuleId: formule.id,
      dureeMinutes: palier.duree_minutes,
      prixMineur: palier.prix_mineur,
      devise: formule.devise,
      finPrevue: new Date(depart.getTime() + palier.duree_minutes * 60_000),
    })))
    .sort((a, b) => a.dureeMinutes - b.dureeMinutes)
}
