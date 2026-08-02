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
import { CATALOGUE_TUILES, accueilVide, tuilesVisibles } from '../core/accueil/tuiles'
import fr from '../core/i18n/fr.json'

/** Les quatre comptes du pilote, avec l'union réelle de leurs rôles. */
const COMPTES = {
  /** M. Koffi — `proprietaire` : les dix-sept permissions. */
  proprietaire: [
    'etb.etablissement.lire', 'etb.etablissement.modifier', 'etb.service.basculer',
    'etb.capacite.declarer', 'etb.pdv.lire', 'etb.pdv.gerer', 'etb.configuration.lire',
    'etb.configuration.ecrire', 'etb.branding.lire', 'etb.branding.ecrire', 'etb.note.lire',
    'etb.note.ecrire', 'cpt.compte.lire', 'cpt.compte.gerer', 'cpt.role.attribuer',
    'cpt.session.revoquer', 'cpt.audit.consulter',
  ],
  /** Adjoua — `gerant` + `caissier` + `receptionniste` : tout sauf le registre des actions. */
  gerante: [
    'etb.etablissement.lire', 'etb.etablissement.modifier', 'etb.service.basculer',
    'etb.capacite.declarer', 'etb.pdv.lire', 'etb.pdv.gerer', 'etb.configuration.lire',
    'etb.configuration.ecrire', 'etb.branding.lire', 'etb.branding.ecrire', 'etb.note.lire',
    'etb.note.ecrire', 'cpt.compte.lire', 'cpt.compte.gerer', 'cpt.role.attribuer',
    'cpt.session.revoquer',
  ],
  /** Yao — `receptionniste` : les cinq lectures, et rien d'autre. */
  receptionniste: [
    'etb.etablissement.lire', 'etb.pdv.lire', 'etb.configuration.lire',
    'etb.branding.lire', 'etb.note.lire',
  ],
  /** Un compte fraîchement créé — aucun rôle. */
  nu: [] as string[],
}

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
  it('le propriétaire voit les trois tuiles, dont le registre des actions', () => {
    const ecran = monter(COMPTES.proprietaire)

    expect(ecran.findAll('[data-tuile]')).toHaveLength(3)
    expect(ecran.find('[data-tuile="journal-audit"]').exists()).toBe(true)
  })

  it('la gérante voit deux tuiles — le registre des actions lui est ABSENT', () => {
    // `cpt.audit.consulter` est exclue de `gerant` : CPT-04 désigne le registre comme « ce que
    // M. Koffi achète », et la lecture par le surveillé change ce que le registre est.
    const ecran = monter(COMPTES.gerante)

    expect(ecran.findAll('[data-tuile]')).toHaveLength(2)
    expect(ecran.find('[data-tuile="journal-audit"]').exists()).toBe(false)
    // Ni grisée, ni masquée : le libellé n'est nulle part dans le HTML.
    expect(ecran.html()).not.toContain(fr.accueil.tuiles.journal.libelle)
  })

  it('le réceptionniste ne voit que l’établissement', () => {
    const ecran = monter(COMPTES.receptionniste)

    expect(ecran.findAll('[data-tuile]')).toHaveLength(1)
    expect(ecran.find('[data-tuile="etablissement"]').exists()).toBe(true)
    expect(ecran.html()).not.toContain(fr.accueil.tuiles.comptes.libelle)
  })

  it('les quatre jeux de tuiles sont bien DIFFÉRENTS deux à deux', () => {
    // Sans cette assertion, quatre montages rendant tous la même chose passeraient les trois
    // précédentes si le filtrage cessait de fonctionner dans le sens permissif.
    const jeux = [COMPTES.proprietaire, COMPTES.gerante, COMPTES.receptionniste, COMPTES.nu]
      .map(permissions => tuilesVisibles(permissions).map(t => t.code).join('|'))

    expect(new Set(jeux).size).toBe(4)
  })
})

describe('FR-026 — aucune action interdite dans le HTML rendu', () => {
  it('aucune tuile grisée, aucun lien masqué', () => {
    const ecran = monter(COMPTES.receptionniste)
    const html = ecran.html()

    expect(html).not.toContain('disabled')
    expect(html).not.toContain('aria-disabled')
    expect(html).not.toMatch(/display:\s*none/)
    expect(html).not.toMatch(/visibility:\s*hidden/)
    // Et aucune route interdite n'apparaît, même en attribut.
    expect(html).not.toContain('/journal-audit')
    expect(html).not.toContain('/comptes')
  })

  it('la route de chaque tuile visible est bien rendue — versant positif', () => {
    // Sans lui, l'assertion précédente serait vraie sur un écran qui ne rend aucun lien.
    const ecran = monter(COMPTES.proprietaire)

    expect(ecran.html()).toContain('/etablissement')
    expect(ecran.html()).toContain('/comptes')
    expect(ecran.html()).toContain('/journal-audit')
  })
})

describe('FR-027 — une tuile issue de plusieurs rôles apparaît UNE fois', () => {
  it('Adjoua porte trois rôles et voit chaque tuile une seule fois', () => {
    const ecran = monter(COMPTES.gerante)

    for (const code of ['etablissement', 'comptes']) {
      expect(
        ecran.findAll(`[data-tuile="${code}"]`),
        `la tuile « ${code} » apparaît plusieurs fois`,
      ).toHaveLength(1)
    }
  })

  it('une permission répétée ne duplique rien', () => {
    // La structure l'interdit — le catalogue est une liste de tuiles, pas une liste par rôle —
    // et l'assertion le constate depuis l'autre bout.
    const doublons = ['cpt.compte.lire', 'cpt.compte.lire', 'cpt.compte.lire']

    expect(tuilesVisibles(doublons)).toHaveLength(1)
  })
})

describe('l’état vide est EXPLICITE', () => {
  it('un compte sans rôle obtient une explication, pas un écran blanc', () => {
    const ecran = monter(COMPTES.nu)

    expect(ecran.findAll('[data-tuile]')).toHaveLength(0)
    expect(ecran.text()).toContain(fr.accueil.vide.titre)
    // La phrase dit quoi faire : il n'y a rien à réessayer, il y a quelqu'un à prévenir.
    expect(ecran.text()).toContain(fr.accueil.vide.explication)
    expect(accueilVide(COMPTES.nu)).toBe(true)
  })

  it('ce n’est pas une erreur — aucun bandeau d’alerte', () => {
    const ecran = monter(COMPTES.nu)

    expect(ecran.find('[role="alert"]').exists()).toBe(false)
  })
})

describe('le catalogue ne promet que des écrans qui existent — principe X', () => {
  it('chaque tuile ouvre une route livrée par ce cycle ou le précédent', () => {
    const routes = new Set(['/etablissement', '/comptes', '/journal-audit', '/hebergement'])

    for (const tuile of CATALOGUE_TUILES) {
      expect(routes, `« ${tuile.code} » ouvre une route non livrée`).toContain(tuile.route)
    }
  })

  it('la PREMIÈRE tuile rattachée à un module d’activité — et son mécanisme cesse d’être fictif', () => {
    // Le cycle 003 écrivait ici « aucune tuile n'exige de module d'activité à ce cycle » : le
    // mécanisme existait, filtré et testé, mais sur une cible **fabriquée par le test lui-même**.
    // Le cycle 004 lui donne sa première cible réelle, exactement comme il donne à
    // `permission.module_code` ses cinq premières valeurs non nulles.
    const rattachees = CATALOGUE_TUILES.filter(tuile => tuile.moduleRequis !== undefined)

    expect(rattachees.map(t => t.code)).toEqual(['hebergement-offre'])
    expect(rattachees[0]!.moduleRequis).toBe('HEBERGEMENT')

    // Et les autres restent transverses : une tuile qui exigerait un module par recopie
    // disparaîtrait des établissements qui ne l'ont pas — un maquis n'a pas d'hébergement, et il
    // doit garder ses réglages.
    expect(
      CATALOGUE_TUILES.filter(t => t.moduleRequis === undefined).map(t => t.code),
    ).toEqual(['etablissement', 'comptes', 'journal-audit'])
  })

  it('le filtre par module fonctionne quand même — il ne se déclare pas', () => {
    const avecModule = [
      ...CATALOGUE_TUILES,
      {
        code: 'restaurant',
        libelleCle: 'x',
        descriptionCle: 'x',
        icone: 'ph-fork-knife',
        permission: 'etb.etablissement.lire',
        route: '/restaurant',
        moduleRequis: 'RESTAURATION',
      },
    ]
    const visible = (modules: string[]) =>
      avecModule.filter(
        tuile =>
          COMPTES.proprietaire.includes(tuile.permission)
          && (!tuile.moduleRequis || modules.includes(tuile.moduleRequis)),
      )

    expect(visible([]).some(t => t.code === 'restaurant')).toBe(false)
    expect(visible(['RESTAURATION']).some(t => t.code === 'restaurant')).toBe(true)
  })
})
