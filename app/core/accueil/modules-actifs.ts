/**
 * **Les modules d'activité actifs de l'établissement courant** — ce dont l'accueil se sert pour
 * décider qu'une tuile existe.
 *
 * # ⚠️ CE FICHIER EXISTE PARCE QU'UNE CONSTANTE EN DUR A CACHÉ DEUX ÉCRANS LIVRÉS
 *
 * `app/pages/index.vue` portait, depuis le cycle 003 :
 *
 * ```ts
 * const modulesActifs = computed<readonly string[]>(() => [])
 * ```
 *
 * accompagné d'un commentaire disant que la liste était vide « et c'est exact », parce qu'aucune
 * tuile n'exigeait alors de module. Le cycle 004 a posé les deux premières tuiles à
 * `moduleRequis: 'HEBERGEMENT'` — sans toucher à cette ligne. Résultat : « Vos formules » et
 * « Vos chambres » étaient **filtrées pour tout le monde, dans tous les établissements**, alors
 * que les deux écrans s'ouvraient parfaitement par l'URL et affichaient onze formules et dix-huit
 * chambres.
 *
 * Le filtre par module n'a donc **jamais rien laissé passer** depuis qu'il sert à quelque chose.
 * Ce que ça enseigne dépasse l'accueil : *une valeur en dur accompagnée d'un commentaire qui la
 * justifie survit à la raison qui l'a posée* — le commentaire rassure la relecture suivante au
 * lieu de l'alerter.
 *
 * # La source est reçue, jamais importée — `core/` ne connaît pas `modules/`
 *
 * L'appel qui rend les services actifs vit dans `modules/etablissements/donnees.ts`, et c'est sa
 * place : il est déjà la source unique de cette lecture, employée par l'écran `G1` et par sa
 * bascule de service. L'importer d'ici ferait de ce fichier **le premier de tout `core/` à
 * dépendre d'un module** — la hiérarchie du front reprend celle des crates, où `socle/` ne
 * connaît pas `verticales/`, et elle n'a encore aucune entorse.
 *
 * La page câble donc les deux : elle passe le chargeur, ce fichier tient la règle. Le contrat est
 * réduit à {@link LireServices} — un `module_code`, rien d'autre —, ce qui le rend testable sans
 * réseau et sans double du client HTTP.
 *
 * # Le repli hors ligne n'est pas une commodité : sans lui, l'accueil se vide
 *
 * La liste vient d'un appel réseau. Hors ligne, cet appel échoue — et si l'échec valait « aucun
 * module », l'accueil d'Aminata perdrait **cinq tuiles sur onze** au moment précis où elle en a le
 * plus besoin : `/passage` et `/arrivee` sont les écrans conçus pour le réseau coupé, et
 * `hors-ligne.spec.ts` les balaie déjà en montrant leurs tarifs en cache.
 *
 * Le dernier état connu est donc conservé par `stockagePersistant` — stockage **ordinaire**, pas
 * le coffre : une liste de codes de modules n'est ni un secret ni un jeton, et le stockage
 * sécurisé est réservé à ce qui doit y aller. La lecture d'un référentiel en cache est de
 * **classe A** (registre, §1.0.2), ce qui est exactement ce qu'on fait ici.
 *
 * La clé porte l'identifiant d'établissement : M. Koffi passe d'un hôtel à une résidence meublée
 * sur le même appareil, et servir le cache de l'un à l'autre montrerait des tuiles d'hébergement
 * sur un établissement qui n'en fait pas — le défaut même que le filtre existe pour éviter.
 */

import type { StockagePersistant } from '~/core/platform'

/** Préfixe de la clé de cache. Le suffixe est l'identifiant d'établissement — voir l'en-tête. */
const CLE_CACHE = 'accueil.modules-actifs.'

/**
 * Ce que l'accueil attend de sa source de services — **le strict minimum**.
 *
 * Volontairement plus étroit que `ServiceActif` du contrat : l'accueil ne lit que le code du
 * module, et déclarer le type complet ferait passer par ici les capacités, l'ordre et tout ce que
 * le cycle suivant y ajoutera.
 */
export type LireServices = () => Promise<readonly { readonly module_code: string }[]>

/**
 * D'où vient la liste rendue.
 *
 * Écrit plutôt que deviné : l'appelant qui voudra afficher une fraîcheur — comme `/passage` le
 * fait pour ses tarifs — a besoin de savoir si ce qu'il montre vient du serveur ou d'un cache.
 * Aucun écran ne s'en sert encore ; le rendre dès maintenant évite d'avoir à rouvrir la signature.
 */
export type OrigineModules = 'serveur' | 'cache' | 'aucune'

export interface ModulesActifs {
  readonly codes: readonly string[]
  readonly origine: OrigineModules
}

/**
 * Charge les modules actifs, et **retombe sur le dernier état connu** si la source ne répond pas.
 *
 * Ne lève jamais. L'accueil est le premier écran après la connexion : le faire échouer sur une
 * liste de services indisponible rendrait inatteignables les tuiles qui ne dépendent d'aucun
 * module — dont `/mes-envois`, précisément l'écran qu'on ouvre quand quelque chose ne passe pas.
 */
export async function chargerModulesActifs(
  lireServices: LireServices,
  etablissementId: string,
  stockage: StockagePersistant,
): Promise<ModulesActifs> {
  const cle = CLE_CACHE + etablissementId

  try {
    const services = await lireServices()
    const codes = services.map(service => service.module_code)
    // Le cache est réécrit **même quand la liste est vide** : une résidence meublée n'a aucun
    // service, et garder l'ancienne liste lui rendrait les tuiles d'un établissement qu'elle
    // n'est plus. Une désactivation doit se voir hors ligne au prochain démarrage.
    await stockage.ecrire(cle, JSON.stringify(codes))
    return { codes, origine: 'serveur' }
  }
  catch {
    return await modulesEnCache(stockage, etablissementId)
  }
}

/**
 * Le dernier état connu, **sans toucher au réseau**.
 *
 * L'accueil l'appelle en premier et rend ses tuiles avec, puis rafraîchit par
 * {@link chargerModulesActifs}. Attendre le serveur pour afficher quoi que ce soit ferait vivre
 * l'écran d'accueil en « Chargement… » le temps d'un aller-retour — sur le pire réseau du produit,
 * au premier écran après la connexion. Et ne PAS le faire ferait apparaître les tuiles
 * d'hébergement après coup, sous le doigt de quelqu'un qui visait déjà autre chose.
 *
 * Un cache illisible vaut cache absent — jamais une exception.
 */
export async function modulesEnCache(
  stockage: StockagePersistant,
  etablissementId: string,
): Promise<ModulesActifs> {
  const cle = CLE_CACHE + etablissementId
  try {
    const brut = await stockage.lire(cle)
    if (!brut) {
      return { codes: [], origine: 'aucune' }
    }
    const lu: unknown = JSON.parse(brut)
    // Un contrôle de forme, pas une confiance : le stockage est ordinaire, donc modifiable, et
    // un tableau de n'importe quoi passerait ensuite dans `includes` sans bruit.
    if (!Array.isArray(lu) || lu.some(code => typeof code !== 'string')) {
      return { codes: [], origine: 'aucune' }
    }
    return { codes: lu as string[], origine: 'cache' }
  }
  catch {
    return { codes: [], origine: 'aucune' }
  }
}
