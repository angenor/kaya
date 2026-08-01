/**
 * **La règle d'affichage la plus structurante du produit** — ETB-02, principe VII.
 *
 * > Un module inactif est **absent**, jamais grisé.
 *
 * Ni entrée désactivée, ni mention « disponible dans votre offre », ni marqueur masqué par CSS.
 * M. Koffi possède un hôtel et une résidence meublée : ce qu'il voit sur la seconde ne doit
 * **jamais** mentionner un service qu'elle n'a pas. Un service grisé pose une question — « et si
 * je l'activais ? » — que le produit n'a pas à poser à quelqu'un venu vérifier sa recette.
 *
 * # Pourquoi une fonction pure, testée à part
 *
 * L'API ne rend déjà que les services actifs : `GET .../services` n'a même pas de paramètre pour
 * demander les autres (`contracts/http-api.md` §3). Cette fonction est donc la **seconde**
 * barrière, celle qui tient le jour où un écran recevra une liste plus large — la console
 * éditeur, par exemple, qui pilotera le référentiel.
 *
 * La sortir du composant la rend testable sans DOM, et le test de rendu qui l'accompagne vérifie
 * que le HTML produit ne porte **aucun** libellé ni code des services absents.
 */

import type { components } from '@kaya/client'

/**
 * Un service actif, tel que l'API le rend.
 *
 * **Alias du contrat, pas une copie.** Ces trois types étaient redéclarés à la main ; T062 a
 * montré qu'un champ renommé côté serveur laissait alors le front compiler. Voir la note de tête
 * de `donnees.ts`.
 */
export type ServiceActif = components['schemas']['ServiceActif']

/** Une capacité déclarée par un service. */
export type CapaciteDuService = components['schemas']['CapaciteDuService']

/**
 * Une entrée de référentiel, telle que l'API la rend.
 *
 * Elle porte `implementee`, rendu délibérément par l'API — le drapeau existe pour la console
 * éditeur et pour distinguer « inconnu » de « connu mais non implémenté » dans un message
 * d'erreur. **Le filtrage est une règle d'affichage**, et c'est [`servicesVisibles`] qui la porte.
 */
export type EntreeReferentiel = components['schemas']['EntreeReferentiel']

/**
 * Les services **à afficher**, dans l'ordre du référentiel.
 *
 * Deux filtres, et le second n'est pas redondant :
 *
 * 1. **actifs seulement** — l'API ne rend déjà que ceux-là, mais la règle est ici aussi ;
 * 2. **connus du référentiel** — un code que le référentiel ne connaît pas n'a ni libellé ni
 *    ordre d'affichage. L'afficher produirait une ligne muette à une place arbitraire.
 */
export function servicesVisibles(
  actifs: ServiceActif[],
  referentiel: EntreeReferentiel[],
): ServiceActif[] {
  const connus = new Map(referentiel.map((e) => [e.code, e]))

  return actifs
    .filter((service) => connus.has(service.module_code))
    .slice()
    .sort((a, b) => {
      const ordreA = connus.get(a.module_code)?.ordre ?? Number.MAX_SAFE_INTEGER
      const ordreB = connus.get(b.module_code)?.ordre ?? Number.MAX_SAFE_INTEGER
      // Départage par code : deux services de même ordre changeraient de place d'un rendu à
      // l'autre, et l'œil le remarque avant la raison.
      return ordreA - ordreB || a.module_code.localeCompare(b.module_code)
    })
}

/**
 * Les services **proposables à l'activation** — ceux que l'établissement n'a pas encore.
 *
 * **Aucune valeur non implémentée n'est jamais proposée** (FR-036). Le référentiel porte
 * `implementee` pour que le client sache distinguer « inconnu » de « pas encore » dans un message
 * d'erreur ; le proposer à l'activation garantirait un refus `422` que l'exploitant n'a aucune
 * raison de rencontrer.
 */
export function servicesActivables(
  actifs: ServiceActif[],
  referentiel: EntreeReferentiel[],
): EntreeReferentiel[] {
  const dejaActifs = new Set(actifs.map((s) => s.module_code))

  return referentiel
    .filter((entree) => entree.implementee && !dejaActifs.has(entree.code))
    .slice()
    .sort((a, b) => a.ordre - b.ordre || a.code.localeCompare(b.code))
}

/**
 * Les capacités **à afficher sous un service**.
 *
 * # Le mot « capacité » n'apparaît nulle part à l'écran
 *
 * `docs/design/lexique.md` : le terme est un mot d'architecture, qui oppose le transverse à la
 * verticale. L'exploitant ne voit que la capacité **concrète** — « Suivi du stock » — sous le
 * service qui la consomme, et jamais une rubrique « Capacités ».
 *
 * Une liste vide est la forme normale : la résidence meublée ne consomme rien, et cela ne se
 * signale pas.
 */
export function capacitesVisibles(service: ServiceActif): CapaciteDuService[] {
  return service.capacites
    .slice()
    .sort((a, b) => a.capacite_code.localeCompare(b.capacite_code))
}

/**
 * L'origine d'une valeur, traduite en **clé i18n** du lexique.
 *
 * « Vaut pour tous vos établissements » ou « Modifié ici ». Les mots « héritage », « surcharge »,
 * « portée » et « override » n'atteignent jamais l'interface.
 */
export function cleOrigine(origine: string): string {
  return origine === 'TENANT'
    ? 'etablissement.origine.herite'
    : 'etablissement.origine.modifie_ici'
}
