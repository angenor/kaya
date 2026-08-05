/**
 * **Les adresses exercées par les portes de bout en bout** — déclarées ICI, et nulle part ailleurs.
 *
 * ═══════════════════════════════════════════════════════════════════════════════════════════
 *  UN VERT OBTENU SUR LE PRODUIT DE QUELQU'UN D'AUTRE EST LE PIRE VERT POSSIBLE
 * ═══════════════════════════════════════════════════════════════════════════════════════════
 *
 * `playwright.config.ts` honore `KAYA_PORT_E2E` depuis le cycle 004, et il dit pourquoi :
 *
 * > un serveur Nuxt d'un **autre projet** occupait déjà le 3000.
 *
 * Trois fichiers de `tests-e2e/` écrivaient pourtant `baseURL: 'http://localhost:3000'` en dur
 * dans leur `browser.newContext()`. Sur un poste où le 3000 est pris, Playwright servait
 * l'application sur 3001 **et les portes interrogeaient le serveur du voisin** — rendant un
 * verdict sur une application qui n'est pas celle du dépôt.
 *
 * C'est le pendant silencieux du `pkill -f "nuxt.mjs dev"` qui a tué le serveur d'un autre projet
 * de ce poste : l'un détruit le travail du voisin, l'autre le prend pour le nôtre. Le second ne
 * casse rien, ce qui le rend plus durable.
 *
 * ⚠️ **Le défaut a été diagnostiqué, expliqué, corrigé sur UNE occurrence — puis reproduit dans le
 * fichier neuf du même lot.** `browser.newContext({ baseURL })` est un endroit où l'on écrit une
 * adresse sans y penser, et rien ne le gardait. Trois fichiers, trois occasions de se tromper.
 *
 * `app/tests/adresses-e2e.spec.ts` referme la porte : il refuse tout `localhost:` littéral dans
 * `tests-e2e/`, **sauf ce fichier**, et renvoie ici. Il tourne dans le job `app`, sans rien
 * allumer — le niveau qui s'exécute.
 */

/**
 * Port du serveur Nuxt exercé. **Même variable que `playwright.config.ts`**, lue de la même façon.
 *
 * Changer de port ne change rien à ce que les portes vérifient : elles exercent les mêmes routes,
 * sur les mêmes moteurs, dans les mêmes thèmes.
 */
export const PORT_APP = Number(process.env.KAYA_PORT_E2E ?? 3000)

/** L'application — ce que `browser.newContext({ baseURL })` doit recevoir, toujours. */
export const BASE_APP = `http://localhost:${PORT_APP}`

/**
 * L'API — interrogée directement par le balayage hors ligne, qui vérifie l'état de la file
 * côté serveur sans passer par l'interface.
 *
 * `KAYA_API_BASE_URL` existe pour la même raison que `KAYA_PORT_E2E` : rien ne garantit que le
 * 8080 soit libre sur un poste partagé.
 */
export const BASE_API = process.env.KAYA_API_BASE_URL ?? 'http://localhost:8080'
