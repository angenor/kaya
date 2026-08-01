/**
 * L'adaptateur de plateforme **en vigueur** — le point d'entrée unique des composants métier.
 *
 * # Pourquoi ce fichier existe séparément
 *
 * Les cinq fichiers voisins déclarent le contrat et ses quatre implémentations. Aucun ne disait
 * **laquelle est active**, et le premier composant qui a eu besoin de l'état réseau (le patron
 * d'écriture d'ETB-02) s'est trouvé devant le choix d'importer `adaptateurWeb` directement — ce
 * qui aurait figé le web dans un composant métier, exactement ce que le principe VII interdit.
 *
 * Il vit dans `core/platform/` et nulle part ailleurs : c'est le seul répertoire où la détection
 * de plateforme est autorisée (dérogation nommée de la porte P-15, `app/eslint.config.js`).
 *
 * # Ce qu'il fait aujourd'hui, et ce qu'il fera
 *
 * **Il renvoie `adaptateurWeb`, et c'est exact** : la coquille Tauri n'est pas construite, aucune
 * cible desktop, Android ou iOS n'existe encore, et l'application tourne dans un navigateur. Le
 * jour où la coquille arrive, la sélection se fait ici — dans un fichier, pas dans quinze
 * composants.
 *
 * **Provisoire nommé**, de même nature que `CONTEXTE_PAR_EN_TETES` : la sélection réelle demande
 * de détecter Tauri, ce qui suppose que la coquille existe. L'écrire à vide maintenant produirait
 * une branche jamais exécutée, donc jamais vérifiée.
 */

import { adaptateurWeb } from './web'
import type { PlatformAdapter } from './index'

/**
 * L'adaptateur actif.
 *
 * Les composants métier passent **toujours** par ici. Importer une implémentation nommée depuis un
 * composant fonctionnerait — sur une plateforme — et casserait sur les trois autres, sans que rien
 * ne le signale avant le cycle suivant.
 */
export function adaptateurCourant(): PlatformAdapter {
  return adaptateurWeb
}
