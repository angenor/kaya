/**
 * L'état réseau, **réactif** — la source unique des composants métier.
 *
 * # Pourquoi il vit dans `core/platform/` et nulle part ailleurs
 *
 * Le principe VII range le réseau parmi les capacités qui passent par `PlatformAdapter`, au même
 * titre que l'impression et la géolocalisation. Un composant qui poserait lui-même des écouteurs
 * `online` / `offline` sur `window` fonctionnerait — dans un navigateur — et devrait être rouvert
 * le jour où Tauri fournit un signal plus fin que celui du navigateur.
 *
 * # Ce qu'il vaut, et ce qu'il ne vaut pas
 *
 * `navigator.onLine` dit qu'une **interface réseau est active**, pas que le serveur répond. À
 * Abengourou, une 3G qui affiche « en ligne » sans porter la moindre requête est le cas courant.
 * D'où le troisième état, `degrade`, que **le cycle SYN alimentera depuis les échecs réels de
 * requête** — il n'est produit par personne aujourd'hui, et c'est écrit plutôt que supposé.
 *
 * Conséquence pour les appelants : `connecte` signifie « rien ne dit que c'est coupé », pas « ça
 * marche ». Une opération de classe C peut donc encore échouer après coup, et son message
 * d'erreur doit rester lisible — ce n'est pas parce que la garde hors-ligne existe qu'elle
 * dispense du traitement d'erreur.
 */

import { onScopeDispose, readonly, ref, type Ref } from 'vue'

import { adaptateurCourant } from './courant'
import type { EtatReseau } from './index'

/**
 * État réseau courant, mis à jour tant que la portée du composant appelant vit.
 *
 * Les écouteurs sont retirés par `onScopeDispose` : un composant démonté qui garderait son
 * écouteur ferait fuir de la mémoire à chaque navigation, sur des terminaux qui n'en ont pas.
 */
export function useEtatReseau(): Readonly<Ref<EtatReseau>> {
  const etat = ref<EtatReseau>(adaptateurCourant().etatReseau())

  if (typeof window !== 'undefined') {
    const relire = () => {
      etat.value = adaptateurCourant().etatReseau()
    }
    window.addEventListener('online', relire)
    window.addEventListener('offline', relire)
    onScopeDispose(() => {
      window.removeEventListener('online', relire)
      window.removeEventListener('offline', relire)
    })
  }

  return readonly(etat)
}
