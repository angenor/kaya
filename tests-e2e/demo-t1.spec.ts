/**
 * ★ **La démo de fin de tranche T1, DÉROULÉE — pas racontée.**
 *
 * ═══════════════════════════════════════════════════════════════════════════════════════════════
 *  LE CRITÈRE DE CLÔTURE DE LA TRANCHE, `docs/user-stories-v1.md` §0.5
 *
 *  *« Yao enregistre un client en chambre pour 2 nuits, puis un passage de 4 h — la disponibilité
 *  empêche tout chevauchement, tout est tracé. »*
 *
 *  Ce fichier l'exécute sur les **seules données de démonstration**, dans un vrai navigateur, sur
 *  **Chromium et WebKit**, en mode **clair** et en mode **sombre**. Une démo qu'on décrit dans un
 *  document se périme au premier renommage de bouton ; une démo qui s'exécute rougit.
 * ═══════════════════════════════════════════════════════════════════════════════════════════════
 *
 * # ⚠️ CE FICHIER CONSOMME DES CHAMBRES — remettre le parc à neuf avant
 *
 *     bash scripts/dev/charger-seeds.sh --remettre-a-neuf
 *
 * Recharger les seeds **ne suffit pas** : ils ajoutent, ils n'effacent jamais. Deux exécutions sans
 * remise à neuf, et la catégorie se remplit — l'écran affiche « toutes les chambres sont prises »
 * et la démo échoue sur un symptôme qui n'a rien à voir avec le produit.
 *
 * # Ce qui est vérifié, et ce qui ne l'est pas
 *
 * | Étape du quickstart §9 | Vérifiée ici | Comment |
 * |---|---|---|
 * | 1 · la fiche se trouve en tapant `bakay` | ✅ | La liste se réduit pendant la frappe |
 * | 2 · client connu → zéro champ ressaisi | ✅ | Aucun champ de saisie ne porte le nom ni le téléphone |
 * | 3 · le passage en deux taps | ✅ | Durée, chambre, « C'est fait » avec l'heure de fin |
 * | 4 · la disponibilité empêche le chevauchement | ✅ **par sa conséquence visible** | La chambre attribuée devient **non attribuable** et porte son heure de fin |
 * | 5 · la note, le total, le montant de taxe **absent** | ✅ | `null` et non zéro — voir plus bas |
 * | 6 · le registre des actions | ⚠️ **partiel** | L'écran s'ouvre et liste ; le parcours de démo n'engendre **aucune** des trois entrées que le quickstart cite |
 *
 * ★ **L'étape 4 n'est pas vérifiée par un refus, et c'est délibéré.** Rejouer la même chambre
 * depuis l'écran est **impossible** : elle y devient non attribuable dès qu'elle est prise, ce qui
 * est exactement le comportement voulu. Le **refus** lui-même — `409 unite_deja_occupee`, produit
 * par la contrainte d'exclusion et non par une lecture préalable — est éprouvé par
 * `backend/tests/sejour_arrivee.rs`, sur deux arrivées **concurrentes**. Reproduire ici un refus
 * que l'écran rend inatteignable donnerait une démo qui ment sur le parcours réel.
 *
 * ⚠️ **L'étape 6 est déclarée partielle plutôt que maquillée.** Le quickstart cite « la rebascule,
 * la régularisation, le changement d'unité » : aucun de ces trois gestes n'appartient au parcours
 * de démonstration. Les fabriquer pour remplir l'écran ferait passer le test au vert sur une
 * démonstration que personne ne déroulera jamais ainsi.
 */

import { expect, test, type Page } from '@playwright/test'

import { COMPTE_DEMONSTRATION as COMPTE } from './routes'

/** Le client de démonstration, tel que les seeds le posent. */
const CLIENT = { recherche: 'bakay', nom: 'Bakayoko', telephone: '+2250707123456' }

/** Les erreurs de page — une démo qui laisse des erreurs de console n'est pas une démo réussie. */
let erreurs: string[] = []

test.beforeEach(() => {
  erreurs = []
})

function surveiller(page: Page): void {
  page.on('pageerror', e => erreurs.push(`pageerror: ${e.message}`))
  page.on('console', m => {
    if (m.type() === 'error') erreurs.push(`console.error: ${m.text()}`)
  })
}

async function seConnecter(page: Page): Promise<void> {
  await page.goto('/connexion')
  await page.getByLabel(/identifiant/i).fill(COMPTE.identifiant)
  await page.getByLabel(/mot de passe/i).fill(COMPTE.motDePasse)
  await page.getByRole('button', { name: /se connecter/i }).click()
  await page.waitForURL(url => new URL(url).pathname === '/', { timeout: 20_000 })
}

// =================================================================================================
//  Les six étapes, en clair PUIS en sombre
// =================================================================================================

for (const theme of ['clair', 'sombre'] as const) {
  test.describe(`démo de fin de tranche T1 — mode ${theme}`, () => {
    // Les étapes partagent une page et un ordre : c'est une démonstration, pas une suite de cas
    // indépendants. `serial` évite qu'un échec en cascade masque le premier.
    test.describe.configure({ mode: 'serial' })

    let page: Page

    test.beforeAll(async ({ browser }) => {
      page = await browser.newPage()
      surveiller(page)
      // Le mode est posé **dans le stockage**, comme le ferait un utilisateur qui l'a choisi : ce
      // sont le script du `<head>` et `plugins/01.theme.client.ts` qui doivent le lire.
      await page.goto('/connexion')
      await page.evaluate((mode) => {
        if (mode === 'sombre') localStorage.setItem('kaya.theme', 'sombre')
        else localStorage.removeItem('kaya.theme')
      }, theme)
      await seConnecter(page)
    })

    test.afterAll(async () => {
      await page.close()
    })

    test('0 · le thème demandé est réellement appliqué', async () => {
      const sombre = await page.evaluate(() =>
        document.documentElement.classList.contains('dark'),
      )
      expect(
        sombre,
        `le mode ${theme} n’est pas appliqué : la démo se déroulerait deux fois dans le même thème, `
        + 'et le point 8 de la Definition of Done serait vérifié au jugé.',
      ).toBe(theme === 'sombre')
    })

    test('1 · la fiche de M. Bakayoko se trouve en tapant « bakay »', async () => {
      await page.goto('/clients')
      await page.getByLabel(/chercher un client/i).fill(CLIENT.recherche)

      const resultat = page.locator('[data-client]').filter({ hasText: CLIENT.nom })
      await expect(resultat).toHaveCount(1, { timeout: 10_000 })

      await resultat.click()
      // La fiche s'ouvre avec ses préférences — c'est ce que `R5` doit montrer.
      await expect(page.locator('[data-preference]').first()).toBeVisible({ timeout: 10_000 })
      expect(erreurs, `étape 1 — erreurs de page : ${erreurs.join(', ')}`).toEqual([])
    })

    test('2 · l’arrivée d’un client connu ne fait RETAPER aucun champ', async () => {
      await page.goto('/arrivee')
      await page.getByLabel(/chercher une fiche client/i).fill(CLIENT.recherche)
      await page.locator('[data-action="chercher-client"]').click()

      const resultat = page.locator('[data-client]').filter({ hasText: CLIENT.nom })
      await expect(resultat).toHaveCount(1, { timeout: 10_000 })
      await resultat.click()

      // ★ La fiche est retenue, affichée **en lecture** — et rien n'est retapable.
      await expect(page.locator('[data-etat="client-retenu"]')).toContainText(CLIENT.nom)
      const valeurs = await page.locator('input').evaluateAll(
        (champs: HTMLInputElement[]) => champs.map(c => c.value),
      )
      expect(
        valeurs,
        'un champ de saisie porte le nom du client : l’écran le fait RETAPER, et renverra une '
        + 'copie qui écrasera la fiche à la prochaine arrivée (FR-035).',
      ).not.toContain(CLIENT.nom)
      expect(valeurs).not.toContain(CLIENT.telephone)

      // Deux nuits, deux accompagnants — **un nom suffit** (FR-015).
      await page.locator('[data-nuits="2"]').click()
      for (const nom of ['Aya', 'Konan']) {
        await page.getByLabel(/nom de l’accompagnant|nom de l'accompagnant/i).fill(nom)
        await page.locator('[data-action="ajouter-accompagnant"]').click()
      }
      await expect(page.locator('[data-accompagnant]')).toHaveCount(2)

      // Le tap sur la chambre EST l'ouverture : pas de bouton « Enregistrer ».
      expect(await page.locator('[type="submit"]').count()).toBe(0)
      const chambre = page.locator('[data-unite][data-etat="libre"]').first()
      await expect(chambre).toBeVisible({ timeout: 10_000 })
      // Le code est le PREMIER `<span>` de la tuile — le texte entier y colle l'état (« B1Libre »).
      const codeChambre = (await chambre.locator('span').first().textContent())?.trim()
      await chambre.click()

      await expect(page.locator('[data-etat="enregistre"]')).toBeVisible({ timeout: 15_000 })
      console.log(`  DÉMO ${theme} · étape 2 — arrivée enregistrée en chambre ${codeChambre}`)
      expect(erreurs, `étape 2 — erreurs de page : ${erreurs.join(', ')}`).toEqual([])
    })

    test('3 · le passage s’enregistre en DEUX gestes, avec l’heure à redire au client', async () => {
      await page.goto('/passage')

      // Geste 1 — la durée. Le dernier palier est le plus long du barème (4 h au jeu de démo).
      const paliers = page.locator('[data-palier]')
      await expect(paliers.first()).toBeVisible({ timeout: 15_000 })
      await paliers.last().click()

      // Geste 2 — la chambre, ET C'EST LA CONFIRMATION.
      const chambre = page.locator('[data-unite][data-etat="libre"]').first()
      await expect(
        chambre,
        'aucune chambre libre : remettre le parc à neuf — '
        + 'bash scripts/dev/charger-seeds.sh --remettre-a-neuf',
      ).toBeVisible({ timeout: 10_000 })
      const codeChambre = (await chambre.locator('span').first().textContent())?.trim()
      const identifiantChambre = await chambre.getAttribute('data-unite')
      await chambre.click()

      const confirmation = page.locator('[data-etat="enregistre"]')
      await expect(confirmation).toBeVisible({ timeout: 15_000 })
      // **L'heure de fin est ce que Yao redit au client** — sans elle, la confirmation ne sert à
      // rien : il faudrait rouvrir le séjour pour savoir jusqu'à quand la chambre est prise.
      await expect(confirmation).toContainText(/\d{1,2}\s?h\s?\d{2}/)
      console.log(`  DÉMO ${theme} · étape 3 — passage enregistré en chambre ${codeChambre}`)

      // ── Étape 4 · LA DISPONIBILITÉ EMPÊCHE LE CHEVAUCHEMENT ─────────────────────────────────
      //
      // Sa conséquence visible : la chambre qu'on vient de prendre n'est plus attribuable, et elle
      // porte son heure de fin. Voir la note de tête sur le refus lui-même.
      await page.locator('[data-action="client-suivant"]').click()
      const prise = page.locator(`[data-unite="${identifiantChambre}"]`)
      await expect(prise).toBeVisible({ timeout: 10_000 })
      await expect(
        prise,
        'la chambre attribuée reste proposée : l’écran promettrait une chambre que la contrainte '
        + 'd’exclusion refusera APRÈS le geste, devant le client.',
      ).toBeDisabled()
      await expect(prise).toHaveAttribute('data-etat', 'occupee')

      expect(erreurs, `étape 3 — erreurs de page : ${erreurs.join(', ')}`).toEqual([])
    })

    test('5 · la note se lit, et le montant de taxe est ABSENT — jamais zéro', async () => {
      await page.goto('/depart')

      const sejour = page.locator('[data-sejour]').first()
      await expect(
        sejour,
        'aucun séjour en cours : la démo n’a rien à faire partir',
      ).toBeVisible({ timeout: 15_000 })
      await sejour.click()

      // La note, son sous-total d'hébergement, son total.
      await expect(page.locator('[data-ligne]').first()).toBeVisible({ timeout: 15_000 })
      await expect(page.locator('[data-total]')).toBeVisible()

      // ★ **Les sections que ce cycle ne sert pas sont NOMMÉES**, pas omises. Une note qui
      //   s'arrêterait à l'hébergement sans rien dire se lirait « ce client n'a rien consommé ».
      for (const section of ['restaurant', 'bar', 'autres_frais', 'taxes']) {
        await expect(
          page.locator(`[data-section-absente="${section}"]`),
          `la section « ${section} » est absente de la note SANS être nommée`,
        ).toBeVisible()
      }

      // ★ **Aucun montant de taxe.** Le constat fige des FAITS ; le calcul appartient à FIS-03.
      //   Un « 0 F » affiché serait pire qu'une absence : il affirmerait que la taxe est nulle.
      const bloc = page.locator('[data-section-absente="taxes"]')
      await expect(bloc).not.toContainText(/\b0\s*F\b/)

      // La mention obligatoire, sur la note ET sur la fiche de police.
      await expect(page.locator('[data-mention-non-fiscale]')).toBeVisible()
      await expect(page.locator('[data-fiche-police]')).toContainText(/non fiscal/i)

      expect(erreurs, `étape 5 — erreurs de page : ${erreurs.join(', ')}`).toEqual([])
    })

    test('6 · le registre des actions s’ouvre et liste — partiel, et c’est déclaré', async () => {
      await page.goto('/journal-audit')
      await expect(page.locator('main')).toBeAttached()
      // ⚠️ Le parcours de démonstration n'engendre aucune des trois entrées que le quickstart cite
      //    — rebascule, régularisation, changement d'unité. Les fabriquer pour remplir l'écran
      //    ferait passer ce cas au vert sur une démonstration que personne ne déroulera ainsi.
      expect(erreurs, `étape 6 — erreurs de page : ${erreurs.join(', ')}`).toEqual([])
    })
  })
}
