/**
 * **La source unique du témoin** — trois nombres, et rien d'autre.
 *
 * # Pourquoi une source unique, et pas deux lectures indépendantes
 *
 * Le témoin (composant 10) vit dans la coquille, présent sur toutes les pages. L'écran `S1` en est
 * le développement : *le témoin dit l'état d'un coup d'œil, le panneau détaille ce qui attend et
 * permet d'agir* (`docs/design/derivation.md` 1.2.1).
 *
 * Deux lectures indépendantes divergeraient — le témoin dirait « 4 en attente » pendant que le
 * panneau en montrerait trois, et l'utilisateur cesserait de croire le premier. Le composant le
 * plus important du produit ne peut pas se permettre d'être contredit par l'écran qui le détaille.
 *
 * # Trois valeurs, JAMAIS un pourcentage
 *
 * La règle du composant 10 est explicite : « un nombre d'écritures et une heure, jamais une barre
 * de progression ». Un pourcentage suppose qu'on connaisse le total, ce qui est faux — la file
 * grandit pendant qu'elle se vide —, et il ne dit rien de ce qui compte : **mon travail est-il
 * parti ?**
 *
 * # Ce qui rafraîchit cet état, et pourquoi ce n'est pas une minuterie
 *
 * Le compte change à trois moments, et à ces trois moments seulement : une écriture enfilée, une
 * écriture partie, un changement d'état réseau. Une minuterie qui relirait la file toutes les
 * secondes réveillerait le processeur d'un Android d'entrée de gamme pour recopier un nombre qui
 * n'a pas bougé — et la batterie doit tenir un service.
 *
 * D'où {@link signalerChangement}, appelée par ce qui **sait** qu'un changement a eu lieu.
 */

import { onScopeDispose, readonly, ref, type Ref } from 'vue'

import { adaptateurCourant } from '~/core/platform/courant'
import type { EtatReseau } from '~/core/platform'

import { ecrituresEnAttente, fileCourante } from './attente'

/** Ce que le témoin et `S1` lisent — et la seule chose qu'ils lisent. */
export interface EtatSynchronisation {
  readonly reseau: EtatReseau
  /** Écritures qui attendent de partir. */
  readonly enAttente: number
  /** Écritures **définitivement refusées** — visibles sur `S1`, jamais sur le témoin. */
  readonly enQuarantaine: number
}

/** Les abonnés à l'état, notifiés à chaque changement réel. */
const abonnes = new Set<() => void>()

/**
 * Signale qu'un changement a eu lieu.
 *
 * Appelée par la file, par l'envoi et par le plugin d'amorçage — jamais par un composant. Un
 * écran qui la déclencherait ferait dépendre l'exactitude du témoin de la page ouverte.
 */
export function signalerChangement(): void {
  for (const abonne of abonnes) {
    abonne()
  }
}

function lire(): EtatSynchronisation {
  const file = fileCourante()
  return {
    reseau: adaptateurCourant().etatReseau(),
    enAttente: ecrituresEnAttente(),
    enQuarantaine: file?.enQuarantaine ?? 0,
  }
}

/**
 * L'état de synchronisation, **réactif tant que la portée du composant appelant vit**.
 *
 * Les écouteurs sont retirés par `onScopeDispose` : un composant démonté qui garderait le sien
 * ferait fuir la mémoire à chaque navigation — sur des terminaux qui n'en ont pas. C'est le même
 * montage que `useEtatReseau()`, et pour la même raison.
 */
export function useEtatSynchronisation(): Readonly<Ref<EtatSynchronisation>> {
  const etat = ref<EtatSynchronisation>(lire())

  const relire = (): void => {
    etat.value = lire()
  }

  abonnes.add(relire)

  if (typeof window !== 'undefined') {
    // Le passage hors ligne doit être **instantané** (composant 10). L'écouter ici plutôt que
    // d'attendre le prochain signal de la file est ce qui le rend instantané.
    window.addEventListener('online', relire)
    window.addEventListener('offline', relire)
  }

  onScopeDispose(() => {
    abonnes.delete(relire)
    if (typeof window !== 'undefined') {
      window.removeEventListener('online', relire)
      window.removeEventListener('offline', relire)
    }
  })

  return readonly(etat) as Readonly<Ref<EtatSynchronisation>>
}

/** L'état hors de tout composant — employé par les tests et par l'envoi. */
export function etatSynchronisation(): EtatSynchronisation {
  return lire()
}
