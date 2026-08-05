/**
 * ★ **SC-004 — la part MACHINE du parcours de passage, sur deux moteurs.**
 *
 * ═══════════════════════════════════════════════════════════════════════════════════════════
 *  CE QUE CE FICHIER MESURE, ET LES DEUX CHOSES QU'IL NE MESURE PAS
 * ═══════════════════════════════════════════════════════════════════════════════════════════
 *
 * | Mesuré ici | Mesuré ailleurs |
 * |---|---|
 * | Le temps **machine** : du premier tap à l'affichage de « C'est fait » | — |
 * | — | Le **nombre de gestes** : `app/tests/budget-gestes.spec.ts`, déterministe |
 * | — | Le temps **humain** : chronométré au terrain, consigné dans `mesures-terrain.md` |
 *
 * Les trois sont distincts et aucun ne remplace les autres. Un parcours de deux gestes peut être
 * lent ; un parcours rapide peut demander cinq gestes ; et ni l'un ni l'autre ne dit combien de
 * temps Yao met réellement, avec un client qui parle et un téléphone qui sonne.
 *
 * ═══════════════════════════════════════════════════════════════════════════════════════════
 *  ⚠️ LE BUDGET EST FIXÉ TRÈS AU-DESSUS DE LA VALEUR OBSERVÉE, ET C'EST UNE DÉCISION
 * ═══════════════════════════════════════════════════════════════════════════════════════════
 *
 * C'est la leçon de SC-004 au cycle 004 : **un seuil serré rougit au hasard et se fait désactiver
 * dans le mois**. Un test désactivé ne garde rien du tout — il est pire qu'absent, puisqu'il
 * figure encore à la liste des portes.
 *
 * Ce que ce budget attrape est une **régression d'ordre de grandeur** : un appel réseau ajouté au
 * chemin chaud, une requête N+1 introduite dans la liste des chambres, un rendu qui recharge la
 * page. Pas une variation de quelques dizaines de millisecondes, que la machine d'intégration
 * produit d'elle-même selon sa charge.
 *
 * ═══════════════════════════════════════════════════════════════════════════════════════════
 *  ⚠️ CE TEST EXIGE L'API, LA BASE ET LES SEEDS — ET IL NE COEXISTE PAS AVEC LA SUITE BACKEND
 * ═══════════════════════════════════════════════════════════════════════════════════════════
 *
 * `exiger_grand_livre_sans_consommateur_concurrent` refuse de dérouler les tests d'outbox quand un
 * worker de publication tourne hors de `cargo test` — c'est-à-dire quand l'API est allumée, ce que
 * ce fichier exige. **Séquencer**, et arrêter l'API **par port** :
 *
 *     lsof -ti:8080 | xargs kill
 *
 * ⚠️ **Jamais `pkill -f`.** Au cycle 005, `pkill -f "nuxt.mjs dev"` a tué le serveur de
 * développement d'un **autre projet** de ce poste, qui tournait depuis cinq heures.
 */

import { expect, test } from '@playwright/test'

import { COMPTE_DEMONSTRATION as COMPTE } from './routes'

/**
 * ★ **Le compte vient de `routes.ts`, une seule fois pour tout le e2e.**
 *
 * ⚠️ Ce fichier en portait une **copie**, avec un identifiant **téléphonique** —
 * `+2250700000002` — que les seeds ne posent pas : ils créent les comptes avec un
 * `identifiant_email`, et rien d'autre. Le symptôme était « Identifiant ou mot de passe
 * incorrect », **indiscernable d'un mot de passe faux** (FR-012), sur une variable
 * `KAYA_SEEDS_MOT_DE_PASSE` pourtant correcte. Cherché du mauvais côté pendant une session.
 *
 * Le repli littéral de mot de passe était de la même famille : il rendait le test **vert-muet**
 * quand la variable manquait, au lieu de dire ce qui manquait. `COMPTE_DEMONSTRATION` lève.
 *
 * ⚠️ **`receptionniste`, et pas `proprietaire`.** Depuis la migration `0030`, le propriétaire ne
 * reçoit que les **lectures** du séjour : avec son compte, la grille des chambres serait
 * **absente** du HTML — ce qui est le comportement voulu, et ferait échouer ce test sur un
 * symptôme qui n'a rien à voir avec la vitesse. Adjoua porte les trois rôles, dont celui-là.
 */

/**
 * Le budget de temps **machine**, en millisecondes, du premier tap à « C'est fait ».
 *
 * ⚠️ **Six secondes pour une opération qui en prend quelques centaines de millisecondes.** Ce
 * n'est pas de la générosité : c'est ce qui rend le test **utile pendant des années**. Voir
 * l'en-tête — un seuil serré est un test qu'on désactive.
 *
 * La valeur réellement observée est **imprimée à chaque exécution**, et c'est elle qu'on lit pour
 * juger d'une dérive. Le seuil, lui, ne sert qu'à faire rougir une régression d'ordre de grandeur.
 */
const BUDGET_MACHINE_MS = 6_000

test.describe('SC-004 — la part machine du parcours de passage', () => {
  test('du premier tap à « C\'est fait », sous le budget déclaré', async ({ page }, info) => {
    const erreurs: string[] = []
    page.on('pageerror', (e) => erreurs.push(`pageerror: ${e.message}`))

    // ── Connexion par le VRAI formulaire ──────────────────────────────────────────────────
    //
    // Poser un jeton forgé dans le stockage irait plus vite et ne prouverait rien : c'est le
    // raisonnement d'`isolation_tenant.rs`, dont les requêtes obtiennent leur jeton par
    // `session_ouvrir` plutôt que par la clé de signature.
    await page.goto('/connexion')
    await page.getByLabel(/identifiant/i).fill(COMPTE.identifiant)
    await page.getByLabel(/mot de passe/i).fill(COMPTE.motDePasse)
    await page.getByRole('button', { name: /se connecter/i }).click()
    await page.waitForURL((url) => new URL(url).pathname === '/', { timeout: 20_000 })

    // ── Montage de l'écran — HORS BUDGET, et c'est écrit ──────────────────────────────────
    //
    // Le chargement des chambres et du barème se fait **avant le premier geste** : il ne compte
    // pas dans le budget de FR-031, qui court du premier geste à la confirmation. Le précharger
    // est même ce qui permet à l'attribution de n'être qu'un tap.
    await page.goto('/passage')
    const paliers = page.locator('[data-palier]')
    await expect(paliers.first()).toBeVisible({ timeout: 20_000 })

    const chambresLibres = page.locator('[data-unite][data-etat="libre"]')
    const disponibles = await chambresLibres.count()
    // Une porte dont la cible est vide passe toujours. Sans chambre libre, ce test mesurerait
    // le temps qu'il faut pour ne rien faire.
    expect(
      disponibles,
      'aucune chambre libre dans les données de démonstration : ce test n\'aurait rien à '
      + 'mesurer.\n'
      + '⚠️ RECHARGER LES SEEDS NE SUFFIT PAS — ils ajoutent, ils n\'effacent jamais, et chaque '
      + 'exécution de ce test CONSOMME une chambre. La commande qui remet le parc à neuf est :\n'
      + '    bash scripts/dev/charger-seeds.sh --remettre-a-neuf\n'
      + 'Ne conclure à une régression qu\'APRÈS l\'avoir lancée.',
    ).toBeGreaterThan(0)

    // ── ★ LE CHRONOMÈTRE — il part au PREMIER GESTE ──────────────────────────────────────
    const depart = Date.now()

    await paliers.first().click()
    await chambresLibres.first().click()

    await expect(page.locator('[data-etat="enregistre"]')).toBeVisible({
      timeout: BUDGET_MACHINE_MS + 4_000,
    })

    const ecoule = Date.now() - depart

    // La valeur observée est **imprimée**, et c'est elle qu'on lit pour juger d'une dérive. Le
    // seuil ne sert qu'à faire rougir une régression d'ordre de grandeur.
    console.log(
      `  SC-004 · ${info.project.name.padEnd(8)} — part machine du passage : ${ecoule} ms `
      + `(budget ${BUDGET_MACHINE_MS} ms)`,
    )

    expect(
      ecoule,
      `la part machine du passage a pris ${ecoule} ms pour un budget de ${BUDGET_MACHINE_MS} ms. `
      + 'Ce budget est fixé TRÈS au-dessus de la valeur observée : le dépasser signale une '
      + 'régression d\'ordre de grandeur — un appel réseau ajouté au chemin chaud, une requête '
      + 'N+1 dans la liste des chambres, un rendu qui recharge la page. Pas une variation de '
      + 'charge de la machine d\'intégration.',
    ).toBeLessThan(BUDGET_MACHINE_MS)

    expect(erreurs, 'le parcours a produit des erreurs de page').toEqual([])
  })

  /**
   * **L'heure de fin est affichée, et c'est ce que Yao redit au client.**
   *
   * Sans elle, la confirmation ne sert à rien : Yao devrait rouvrir le séjour pour savoir jusqu'à
   * quand la chambre est prise, ce qui annule le bénéfice des deux gestes.
   */
  test('la confirmation affiche l\'heure de fin à redire au client', async ({ page }) => {
    await page.goto('/connexion')
    await page.getByLabel(/identifiant/i).fill(COMPTE.identifiant)
    await page.getByLabel(/mot de passe/i).fill(COMPTE.motDePasse)
    await page.getByRole('button', { name: /se connecter/i }).click()
    await page.waitForURL((url) => new URL(url).pathname === '/', { timeout: 20_000 })

    await page.goto('/passage')
    await page.locator('[data-palier]').first().click()
    await page.locator('[data-unite][data-etat="libre"]').first().click()

    const confirmation = page.locator('[data-etat="enregistre"]')
    await expect(confirmation).toBeVisible({ timeout: 10_000 })

    // L'heure de fin, en format français `17 h 30` — **espace ORDINAIRE**, jamais la fine
    // insécable, qui est réservée aux montants (`docs/design/tokens.md` §2).
    await expect(confirmation).toContainText(/\d{2} h \d{2}/)

    // « Client suivant » remet l'écran en nominal **sans rechargement** : un rechargement
    // perdrait le jeton d'accès, qui vit en mémoire, et renverrait sur `/connexion`.
    await confirmation.getByRole('button').click()
    await expect(page.locator('[data-palier]').first()).toBeVisible()
    expect(new URL(page.url()).pathname).toBe('/passage')
  })
})
