/**
 * **La file, branchée au démarrage.** *Point d'amorçage n° 4.*
 *
 * # Ce que ce fichier ferme
 *
 * `brancherFile` a vécu deux cycles dans `core/sync/attente.ts`, exportée, documentée « branchée
 * par SYN-01 », et **appelée nulle part**. `ecrituresEnAttente()` rendait donc `0` — non parce
 * qu'une file était vide, mais parce qu'aucune file n'existait. La garde de « passer la main »
 * gardait le vide.
 *
 * C'est le défaut d'`initialiserTheme()`, à ceci près qu'il était **déclaré** : `amorcage.spec.ts`
 * portait la ligne « dû par SYN-01 », et ce fichier fait échouer ce test tant que la ligne n'a pas
 * basculé à « branché ». C'est exactement ce qu'on attend de lui, et c'est ce qui rend l'oubli
 * impossible.
 *
 * # Pourquoi un plugin, et pourquoi `02`
 *
 * `entry.js` de Nuxt résout **tous** les plugins avant `vueApp.mount()` : c'est le dernier endroit
 * qui s'exécute avant le premier rendu. Un `onMounted` dans une page n'amorcerait que cette page —
 * le défaut que le cycle 004 a réparé, et le reproduire ici serait le déplacer.
 *
 * Le numéro fixe l'ordre : le **thème** d'abord (`01`), pour qu'aucun pixel ne soit peint dans la
 * mauvaise couleur ; la file ensuite. L'inverse ferait apparaître le témoin avant que `.dark` ne
 * soit posée, donc un clignotement sur le composant le plus important du produit.
 *
 * Le suffixe `.client` est **obligatoire** : ce module touche le stockage et `crypto.subtle`.
 *
 * # L'ordre des trois gestes, et ce que l'inverser coûterait
 *
 * 1. **Ouvrir** la file — lit la clé au coffre, déchiffre ce qui attendait. Asynchrone.
 * 2. **Brancher** — `ecrituresEnAttente()` cesse de rendre `0` par défaut.
 * 3. **Abonner** les déclencheurs d'envoi.
 *
 * Brancher avant d'ouvrir exposerait une file vide pendant que le déchiffrement tourne, et le
 * témoin afficherait zéro **au moment précis** où l'utilisateur ouvre l'application pour vérifier
 * que son travail est parti. Abonner avant de brancher armerait un envoi sur une file que
 * `fileCourante()` ne connaît pas encore : le premier retour au premier plan ne ferait rien.
 *
 * # Ce plugin n'échoue jamais bruyamment
 *
 * Un coffre indisponible, un `crypto.subtle` absent, un cryptogramme illisible : dans les trois
 * cas la file repart **vide et en mémoire**, et l'application démarre. Refuser de démarrer parce
 * que la persistance de la file ne fonctionne pas transformerait une dégradation en panne — sur un
 * terminal de comptoir, un soir de service.
 */

import { adaptateurCourant } from '~/core/platform/courant'
import { poserSeuilLatence } from '~/core/platform/observateur-appels'
import { brancherEnvoi, brancherFile, FileLocale, signalerChangement } from '~/core/sync'
import { envoyerNote } from '~/modules/etablissements/notes'

export default defineNuxtPlugin({
  name: 'kaya:sync',
  // Après `kaya:theme`, dont `enforce: 'pre'` garantit qu'il passe en premier.
  async setup() {
    const config = useRuntimeConfig()
    const baseUrl = String(config.public.apiBaseUrl)

    // Le seuil de « connexion faible » vient de la configuration d'établissement
    // (`sync.latence_degradee_seuil_ms`, migration `0028`). Tant qu'aucune session n'est ouverte,
    // la valeur d'attente du module tient — un terminal qui démarre n'a encore rien mesuré.
    const seuil = Number(config.public.latenceDegradeeSeuilMs)
    if (Number.isFinite(seuil)) {
      poserSeuilLatence(seuil)
    }

    const file = await FileLocale.ouvrir(adaptateurCourant())
    brancherFile(file)

    // **Un seul envoyeur aujourd'hui**, et le dire vaut mieux que de préparer un aiguillage vide :
    // `TYPES_CLASSE_A` ne déclare que `note_etablissement.creee`. Le cycle qui ajoutera un type de
    // classe A ajoutera son envoyeur ici, dans le même changement que sa ligne au registre.
    brancherEnvoi(envoyerNote, baseUrl)

    // Le témoin lit son état à la demande ; ce signal le fait relire une fois, tout de suite, pour
    // que le nombre affiché au premier rendu soit celui de la file rouverte et non zéro.
    signalerChangement()
  },
})
