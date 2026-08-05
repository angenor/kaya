// @vitest-environment happy-dom
/**
 * **`R1` — quatre comptes, quatre accueils, sur la même application.**
 *
 * C'est le test que le cycle 002 ne pouvait pas écrire : sans permissions, il n'y avait rien à
 * filtrer. La dette est soldée, et ce fichier est ce qui l'atteste.
 *
 * # Ce qui est vérifié sur le HTML RENDU, jamais sur un booléen
 *
 * 1. **Quatre jeux de permissions → quatre jeux de tuiles.**
 * 2. **Aucune action interdite dans le HTML** — pas de tuile grisée, pas de lien masqué par CSS.
 * 3. **Une tuile issue de plusieurs rôles apparaît UNE fois** (FR-027).
 * 4. **Un compte sans aucun rôle obtient un état vide EXPLICITE**, pas un écran blanc.
 *
 * Un composant peut perdre la propriété 2 par un `v-show` au lieu d'un `v-if`, par un attribut
 * `title`, par une liste de secours dans un commentaire de gabarit — sans que la fonction de
 * filtrage change d'une ligne. D'où la vérification sur le rendu.
 */

import { mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it } from 'vitest'

import EcranAccueil from '../modules/accueil/EcranAccueil.vue'
import { CATALOGUE_TUILES, aucunAccesAccorde, tuilesVisibles } from '../core/accueil/tuiles'
import fr from '../core/i18n/fr.json'

/**
 * Les quatre comptes du pilote, avec l'union réelle de leurs rôles.
 *
 * ⚠️ **CES TROIS JEUX ÉTAIENT FIGÉS AU CYCLE 003, ET LE TEST DÉCRIVAIT UN PRODUIT DISPARU.**
 * Yao y « ne voyait que l'établissement » avec cinq permissions ; le référentiel réel lui en donne
 * seize depuis les cycles HEB et SEJ, dont l'ouverture et la clôture d'un séjour. Un test qui
 * décrit un état ancien ne se contente pas d'être inutile : il **rassure** sur ce qu'il ne vérifie
 * plus.
 *
 * Les trois unions viennent de `comptes.role_permission`, relevé sur la base de démonstration :
 *
 * ```sql
 * SELECT permission_code, string_agg(role_code, ', ' ORDER BY role_code)
 * FROM comptes.role_permission GROUP BY permission_code ORDER BY permission_code;
 * ```
 *
 * Elles sont recopiées ici parce que les tests du front tournent **sans base** ; ce que les
 * migrations permettent de vérifier sans base — l'existence de chaque code — l'est par
 * `permissions.spec.ts` et `catalogue-accueil.spec.ts`, qui lisent les `INSERT`. Les attributions
 * par rôle, elles, sont posées par des `INSERT … SELECT … WHERE` qu'aucune lecture statique
 * raisonnable ne rejoue : c'est la limite assumée de ce fichier, et le décompte ci-dessous la
 * borne — il échoue si l'union recopiée cesse de correspondre au référentiel des migrations.
 */
const COMPTES = {
  /** M. Koffi — `proprietaire` : tout, SAUF les cinq gestes du comptoir. */
  proprietaire: [
    'etb.etablissement.lire', 'etb.etablissement.modifier', 'etb.service.basculer',
    'etb.capacite.declarer', 'etb.pdv.lire', 'etb.pdv.gerer', 'etb.configuration.lire',
    'etb.configuration.ecrire', 'etb.branding.lire', 'etb.branding.ecrire', 'etb.note.lire',
    'etb.note.ecrire', 'cpt.compte.lire', 'cpt.compte.gerer', 'cpt.role.attribuer',
    'cpt.session.revoquer', 'cpt.audit.consulter',
    'heb.offre.lire', 'heb.offre.gerer', 'heb.disponibilite.consulter',
    'heb.unite.attribuer', 'heb.unite.liberer', 'heb.sejour.lire',
    'sej.client.lire',
  ],
  /**
   * Adjoua — `gerant` + `caissier` + `receptionniste` : tout SAUF le registre des actions.
   *
   * C'est le compte le plus large du jeu de démonstration, et le seul à cumuler trois rôles.
   */
  gerante: [
    'etb.etablissement.lire', 'etb.etablissement.modifier', 'etb.service.basculer',
    'etb.capacite.declarer', 'etb.pdv.lire', 'etb.pdv.gerer', 'etb.configuration.lire',
    'etb.configuration.ecrire', 'etb.branding.lire', 'etb.branding.ecrire', 'etb.note.lire',
    'etb.note.ecrire', 'cpt.compte.lire', 'cpt.compte.gerer', 'cpt.role.attribuer',
    'cpt.session.revoquer',
    'heb.offre.lire', 'heb.offre.gerer', 'heb.disponibilite.consulter',
    'heb.unite.attribuer', 'heb.unite.liberer',
    'heb.sejour.lire', 'heb.sejour.ouvrir', 'heb.sejour.clore', 'heb.sejour.prolonger',
    'heb.sejour.changer_unite',
    'sej.client.lire', 'sej.client.gerer',
  ],
  /**
   * Yao — `receptionniste` : cinq lectures transverses, et **tout le comptoir**.
   *
   * Il ne règle rien — ni tarifs, ni comptes, ni configuration — et il fait tourner
   * l'établissement : il reçoit, il attribue, il fait partir. C'est exactement ce que l'accueil
   * doit lui montrer, et il ne le montrait pas.
   */
  receptionniste: [
    'etb.etablissement.lire', 'etb.pdv.lire', 'etb.configuration.lire',
    'etb.branding.lire', 'etb.note.lire',
    'heb.offre.lire', 'heb.disponibilite.consulter',
    'heb.unite.attribuer', 'heb.unite.liberer',
    'heb.sejour.lire', 'heb.sejour.ouvrir', 'heb.sejour.clore', 'heb.sejour.prolonger',
    'heb.sejour.changer_unite',
    'sej.client.lire', 'sej.client.gerer',
  ],
  /** Un compte fraîchement créé — aucun rôle. */
  nu: [] as string[],
}

/**
 * Les modules d'activité de Deloria — un hôtel, donc `HEBERGEMENT`.
 *
 * ⚠️ **Le passer explicitement est le point de ce cycle.** Les montages de ce fichier
 * l'omettaient, donc testaient un établissement sans aucun service : les tuiles de verticale y
 * étaient absentes, et le test le constatait comme une propriété normale. Pendant ce temps
 * `pages/index.vue` codait la même liste vide en dur, et « Vos formules » et « Vos chambres »
 * étaient invisibles **en production** sans qu'aucun test ne puisse le voir. Le double du test
 * reproduisait le défaut du code — la leçon `/passage` du cycle 006, à un cycle d'intervalle.
 */
const DELORIA = ['HEBERGEMENT']

function traduire(cle: string, valeurs?: Record<string, unknown>): string {
  const brut = cle
    .split('.')
    .reduce<unknown>((noeud, part) => (noeud as Record<string, unknown>)?.[part], fr)
  if (typeof brut !== 'string') return cle
  return brut.replace(/\{(\w+)\}/g, (_, nom: string) => String(valeurs?.[nom] ?? ''))
}

function monter(permissions: string[], modulesActifs: string[] = []) {
  return mount(EcranAccueil, {
    props: { nomAffichage: 'Adjoua Kouassi', permissions, modulesActifs },
    global: {
      mocks: { useI18n: () => ({ t: traduire }) },
      config: { globalProperties: { useI18n: () => ({ t: traduire }) } },
      // `NuxtLink` n'existe pas hors du pipeline Nuxt : un `<a>` en tient lieu, et c'est ce que
      // Nuxt rend de toute façon. Le remplacer par un `<div>` ferait passer l'assertion « aucun
      // lien interdit » sans rien vérifier.
      stubs: { NuxtLink: { template: '<a :href="to"><slot /></a>', props: ['to'] } },
    },
  })
}

beforeEach(() => {
  ;(globalThis as Record<string, unknown>).useI18n = () => ({ t: traduire })
})

describe('quatre comptes, quatre accueils', () => {
  it('le propriétaire voit TOUT le catalogue, registre des actions compris', () => {
    const ecran = monter(COMPTES.proprietaire, DELORIA)

    // Il est le seul à porter `cpt.audit.consulter`, et il porte toutes les lectures. Le décompte
    // se lit du catalogue plutôt que d'être recopié : y ajouter une tuile de lecture ne doit pas
    // demander de revenir corriger un nombre ici.
    expect(ecran.findAll('[data-tuile]')).toHaveLength(CATALOGUE_TUILES.length)
    expect(ecran.find('[data-tuile="journal-audit"]').exists()).toBe(true)
  })

  it('la gérante voit tout SAUF le registre des actions', () => {
    // `cpt.audit.consulter` est exclue de `gerant` : CPT-04 désigne le registre comme « ce que
    // M. Koffi achète », et la lecture par le surveillé change ce que le registre est.
    const ecran = monter(COMPTES.gerante, DELORIA)

    expect(ecran.findAll('[data-tuile]')).toHaveLength(CATALOGUE_TUILES.length - 1)
    expect(ecran.find('[data-tuile="journal-audit"]').exists()).toBe(false)
    // Ni grisée, ni masquée : le libellé n'est nulle part dans le HTML.
    expect(ecran.html()).not.toContain(fr.accueil.tuiles.journal.libelle)
  })

  it('le réceptionniste a TOUT LE COMPTOIR, et aucun réglage', () => {
    // ⚠️ Cette assertion disait « ne voit que l'établissement », avec une tuile. C'était vrai du
    // produit du cycle 003 et faux depuis : Yao reçoit, attribue et fait partir. Un accueil qui ne
    // lui proposait que ses réglages en lecture ne lui servait à rien.
    const ecran = monter(COMPTES.receptionniste, DELORIA)
    const visibles = ecran.findAll('[data-tuile]').map(n => n.attributes('data-tuile'))

    expect(visibles).toEqual([
      'passage', 'arrivee', 'depart', 'clients', 'notes', 'mes-envois',
      'hebergement-offre', 'hebergement-chambres', 'etablissement',
    ])
    // Il ne règle ni les comptes ni le registre : les deux sont ABSENTS, pas grisés.
    expect(ecran.html()).not.toContain(fr.accueil.tuiles.comptes.libelle)
    expect(ecran.html()).not.toContain(fr.accueil.tuiles.journal.libelle)
  })

  it('les quatre jeux de tuiles sont bien DIFFÉRENTS deux à deux', () => {
    // Sans cette assertion, quatre montages rendant tous la même chose passeraient les trois
    // précédentes si le filtrage cessait de fonctionner dans le sens permissif.
    const jeux = [COMPTES.proprietaire, COMPTES.gerante, COMPTES.receptionniste, COMPTES.nu]
      .map(permissions => tuilesVisibles(permissions, DELORIA).map(t => t.code).join('|'))

    expect(new Set(jeux).size).toBe(4)
  })

  it('⚠️ le MÊME compte, dans un établissement SANS hébergement, perd les tuiles de verticale', () => {
    // C'est le défaut du cycle 004, pris par l'autre bout. `pages/index.vue` passait `[]` en dur :
    // tous les accueils du produit étaient, de fait, celui d'un maquis. Monter les deux états côte
    // à côte est ce qui rend le filtre observable — un seul état ne prouve rien, quel qu'il soit.
    const hotel = monter(COMPTES.gerante, DELORIA)
    const maquis = monter(COMPTES.gerante, [])

    const codes = (e: ReturnType<typeof monter>) =>
      e.findAll('[data-tuile]').map(n => n.attributes('data-tuile'))

    expect(codes(maquis)).toEqual(['clients', 'notes', 'mes-envois', 'etablissement', 'comptes'])
    expect(codes(hotel).length).toBeGreaterThan(codes(maquis).length)
    // Les cinq tuiles d'hébergement sont absentes du HTML, pas désactivées.
    expect(maquis.html()).not.toContain(fr.accueil.tuiles.hebergement_offre.libelle)
    expect(maquis.html()).not.toContain('/passage')
  })
})

describe('FR-026 — aucune action interdite dans le HTML rendu', () => {
  it('aucune tuile grisée, aucun lien masqué', () => {
    const ecran = monter(COMPTES.receptionniste, DELORIA)
    const html = ecran.html()

    expect(html).not.toContain('disabled')
    expect(html).not.toContain('aria-disabled')
    expect(html).not.toMatch(/display:\s*none/)
    expect(html).not.toMatch(/visibility:\s*hidden/)
    // Et aucune route interdite n'apparaît, même en attribut.
    expect(html).not.toContain('/journal-audit')
    expect(html).not.toContain('/comptes')
  })

  it('la route de CHAQUE tuile visible est bien rendue — versant positif', () => {
    // Sans lui, l'assertion précédente serait vraie sur un écran qui ne rend aucun lien.
    // Le balayage porte sur le catalogue entier, pas sur trois routes recopiées : c'est ce qui
    // fait qu'une tuile ajoutée sans lien dans le gabarit échouerait ici.
    const ecran = monter(COMPTES.proprietaire, DELORIA)
    const html = ecran.html()

    for (const tuile of CATALOGUE_TUILES) {
      expect(html, `la tuile « ${tuile.code} » ne rend pas son lien`).toContain(`"${tuile.route}"`)
    }
  })
})

describe('FR-027 — une tuile issue de plusieurs rôles apparaît UNE fois', () => {
  it('Adjoua porte trois rôles et voit chaque tuile une seule fois', () => {
    const ecran = monter(COMPTES.gerante, DELORIA)

    // Le balayage porte sur toutes les tuiles qu'elle voit, pas sur deux codes recopiés : c'est
    // avec onze tuiles issues de trois rôles qu'un doublon deviendrait probable.
    for (const tuile of tuilesVisibles(COMPTES.gerante, DELORIA)) {
      expect(
        ecran.findAll(`[data-tuile="${tuile.code}"]`),
        `la tuile « ${tuile.code} » apparaît plusieurs fois`,
      ).toHaveLength(1)
    }
  })

  it('une permission répétée ne duplique rien', () => {
    // La structure l'interdit — le catalogue est une liste de tuiles, pas une liste par rôle —
    // et l'assertion le constate depuis l'autre bout.
    const doublons = ['cpt.compte.lire', 'cpt.compte.lire', 'cpt.compte.lire']

    expect(tuilesVisibles(doublons).map(t => t.code)).toEqual(['mes-envois', 'comptes'])
  })
})

describe('l’absence d’accès est EXPLICITE', () => {
  it('un compte sans rôle obtient une explication, pas un écran blanc', () => {
    const ecran = monter(COMPTES.nu, DELORIA)

    expect(ecran.text()).toContain(fr.accueil.vide.titre)
    // La phrase dit quoi faire : il n'y a rien à réessayer, il y a quelqu'un à prévenir.
    expect(ecran.text()).toContain(fr.accueil.vide.explication)
    expect(aucunAccesAccorde(COMPTES.nu, DELORIA)).toBe(true)
  })

  it('⚠️ l’explication COEXISTE avec « Mes envois », elle ne la remplace pas', () => {
    // Le contrôle vivait sur `tuiles.length === 0`, et « Mes envois » — la seule tuile sans
    // permission du produit — l'aurait rendu inatteignable : un compte tout neuf aurait vu une
    // tuile solitaire et AUCUNE explication de ce qui lui manque.
    const ecran = monter(COMPTES.nu, DELORIA)
    const visibles = ecran.findAll('[data-tuile]').map(n => n.attributes('data-tuile'))

    expect(visibles).toEqual(['mes-envois'])
    expect(ecran.text()).toContain(fr.accueil.vide.titre)
  })

  it('et elle DISPARAÎT dès qu’un accès est accordé — versant négatif', () => {
    // Sans lui, une explication affichée en permanence passerait les deux assertions ci-dessus.
    const ecran = monter(COMPTES.receptionniste, DELORIA)

    expect(ecran.text()).not.toContain(fr.accueil.vide.titre)
    expect(aucunAccesAccorde(COMPTES.receptionniste, DELORIA)).toBe(false)
  })

  it('ce n’est pas une erreur — aucun bandeau d’alerte', () => {
    const ecran = monter(COMPTES.nu, DELORIA)

    expect(ecran.find('[role="alert"]').exists()).toBe(false)
  })
})

describe('le catalogue ne promet que des écrans qui existent — principe X', () => {
  /**
   * ⚠️ **La liste des routes livrées était écrite ICI, à la main, et elle a été fausse deux
   * cycles.** Elle nommait cinq routes quand le produit en avait treize, et rien ne l'a dit : elle
   * ne vérifiait qu'un sens — « une tuile ne pointe pas ailleurs » —, jamais l'autre — « toute
   * route a une tuile ». Le contrôle des deux sens vit désormais dans
   * `catalogue-accueil.spec.ts`, qui **découvre** `app/pages/` au lieu de l'énumérer.
   */
  it('les tuiles rattachées à un module le sont toutes à HEBERGEMENT — seul module implémenté', () => {
    const rattachees = CATALOGUE_TUILES.filter(tuile => tuile.moduleRequis !== undefined)

    expect(rattachees.map(t => t.code)).toEqual([
      'passage', 'arrivee', 'depart', 'hebergement-offre', 'hebergement-chambres',
    ])
    expect(rattachees.every(t => t.moduleRequis === 'HEBERGEMENT')).toBe(true)

    // Et les autres restent transverses : une tuile qui exigerait un module par recopie
    // disparaîtrait des établissements qui ne l'ont pas — un maquis n'a pas d'hébergement, et il
    // doit garder ses réglages, ses notes, ses envois **et ses fiches clients**.
    expect(
      CATALOGUE_TUILES.filter(t => t.moduleRequis === undefined).map(t => t.code),
    ).toEqual(['clients', 'notes', 'mes-envois', 'etablissement', 'comptes', 'journal-audit'])
  })

  it('le filtre par module tient sur un module que le produit ne connaît pas encore', () => {
    // ⚠️ Ce test fabriquait une tuile factice **avec ses permissions** et rejouait le filtre à la
    // main. Il ne peut plus : une tuile ne déclare que sa route, et ce qui l'ouvre vit dans
    // `core/acces/ecrans.ts`. C'est une amélioration — rejouer le filtre dans le test, c'était
    // vérifier la copie plutôt que l'original.
    //
    // Ce qui reste à vérifier est le seul versant que les tuiles réelles n'exercent pas :
    // `HEBERGEMENT` est le seul module que le produit sache activer, et le filtre doit tenir sur
    // un code qu'il ne connaît pas encore — celui que le cycle 007 apportera.
    const filtreModule = (moduleRequis: string | undefined, actifs: string[]): boolean =>
      !moduleRequis || actifs.includes(moduleRequis)

    expect(filtreModule('RESTAURATION', [])).toBe(false)
    expect(filtreModule('RESTAURATION', ['HEBERGEMENT'])).toBe(false)
    expect(filtreModule('RESTAURATION', ['HEBERGEMENT', 'RESTAURATION'])).toBe(true)
    expect(filtreModule(undefined, [])).toBe(true)

    // Et le vrai filtre, sur la vraie donnée : Deloria fait de l'hébergement, pas un maquis.
    expect(tuilesVisibles(COMPTES.proprietaire, ['RESTAURATION']).map(t => t.code))
      .not.toContain('passage')
    expect(tuilesVisibles(COMPTES.proprietaire, DELORIA).map(t => t.code)).toContain('passage')
  })
})
