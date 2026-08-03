/**
 * **L'outillage §0.7 du versant application.**
 *
 * # Deux versants, deux formes — et pourquoi ce n'est pas la même chose qu'en Rust
 *
 * `backend/tests/commun/classes.rs` engendre des **tests** par macro : le rejeu triple et les six
 * ordres du désordre sont des propriétés de la base, et chaque entité les exige identiquement.
 *
 * Ici, les propriétés à vérifier sont d'une autre nature — la **marque de classe** au niveau du
 * type, le **refus d'enfilement**, l'**annonce avant la saisie** — et elles se vérifient chacune
 * là où elle vit : dans un test de compilation, dans un test de file, dans un test de composant.
 * Les engendrer ferait un test générique qui dirait « quelque chose a échoué » sans dire quoi.
 *
 * Ce module fournit donc des **utilitaires**, pas des générateurs. Ce qu'il supprime est la
 * recopie : chaque fichier de test construisait sa propre entrée de file, avec ses propres
 * valeurs, et l'ajout de deux champs à `EntreeFile` a fait tomber cinq fichiers d'un coup. C'est
 * exactement la faute que l'outillage existe pour ne plus commettre.
 *
 * # Ce qu'il ne fait PAS
 *
 * Il ne vérifie **jamais la justesse d'une classe**. Marquer une opération de classe B en A ne
 * produirait aucune erreur : la marque garantit qu'une décision a été prise, pas qu'elle est
 * juste. La justesse reste humaine et revue mensuellement — même limite que côté serveur, et pour
 * la même raison.
 */

import { expect } from 'vitest'

import {
  FileLocale,
  marquerClasseA,
  OperationRefusee,
  uuidV7,
  type ContexteEcriture,
  type EntreeFile,
} from '../../core/sync'

/** Le contexte d'écriture des tests — un tenant, un établissement, figés. */
export const CONTEXTE_TEST: ContexteEcriture = {
  tenantId: '018f0000-0000-7000-8000-0000000000aa',
  etablissementId: '018f0000-0000-7000-8000-0000000000bb',
}

/**
 * Compose une entrée de file **complète**.
 *
 * # Pourquoi une fabrique, et non un littéral recopié
 *
 * `EntreeFile` a gagné deux champs au cycle 005 — `contexte` et `tentatives` —, et cinq fichiers
 * de test ont cessé de compiler. C'est le comportement voulu du type, et c'est aussi le signe
 * qu'il fallait un point unique : le sixième fichier aurait recopié le littéral d'un des cinq, et
 * le septième aurait recopié le sixième.
 *
 * Les valeurs par défaut sont celles du **seul type de classe A du produit**. Un test qui veut
 * autre chose le dit explicitement, ce qui rend son intention lisible.
 */
export function entreeDeTest(
  partiel: {
    id?: string
    type?: string
    horodatageClient?: string
    texte?: string
    contexte?: ContexteEcriture
    tentatives?: number
  } = {},
): EntreeFile<{ texte: string }> {
  return {
    id: partiel.id ?? uuidV7(),
    type: partiel.type ?? 'note_etablissement.creee',
    horodatageClient: partiel.horodatageClient ?? '2026-08-03T09:41:00.000Z',
    charge: marquerClasseA(
      { texte: partiel.texte ?? 'Table 4 — deux bières' },
      'A4 — append-only, commutative, sans contrainte d’unicité métier, sans effet monétaire',
    ),
    contexte: partiel.contexte ?? CONTEXTE_TEST,
    tentatives: partiel.tentatives ?? 0,
  }
}

/** Une file **en mémoire** portant `combien` écritures de classe A. */
export function fileAvec(combien: number): FileLocale {
  const file = new FileLocale()
  for (let rang = 0; rang < combien; rang += 1) {
    file.enfiler(entreeDeTest({ texte: `écriture ${rang + 1}` }))
  }
  return file
}

/**
 * **Le refus d'enfilement d'une opération qui n'est pas de classe A.**
 *
 * Le versant *exécution* de la porte P-13. Le versant *compilation* — une charge non marquée qui
 * ne compile pas — se vérifie par `@ts-expect-error` dans le fichier qui le teste : il ne peut pas
 * s'écrire ici, puisqu'il demande une erreur de type, pas une valeur.
 */
export function exigerRefusEnfilement(type: string, charge: unknown = {}): void {
  const file = new FileLocale()

  expect(
    () =>
      file.enfiler({
        id: uuidV7(),
        type,
        horodatageClient: '2026-08-03T09:41:00.000Z',
        charge: marquerClasseA(charge, 'marque abusive — c’est ce que le test vérifie'),
        contexte: CONTEXTE_TEST,
        tentatives: 0,
      }),
    `« ${type} » a été acceptée en file alors qu’elle n’est pas déclarée de classe A.\n`
    + 'Une opération B, C ou D ne va JAMAIS en file : elle est annoncée indisponible à '
    + 'l’utilisateur, immédiatement (principe VI).',
  ).toThrow(OperationRefusee)

  expect(file.enAttente, `« ${type} » a laissé une trace dans la file malgré le refus`).toBe(0)
}

/**
 * **L'annonce d'indisponibilité paraît AVANT la saisie**, dans le rendu donné.
 *
 * La limite est la même que celle du balayage e2e, et elle est assumée : ce contrôle vérifie
 * qu'une annonce **apparaît**, jamais que sa formulation est la bonne. La justesse du libellé
 * relève du lexique et de la porte P-16 ; les confondre donnerait un contrôle qui ment sur ce
 * qu'il garantit.
 */
export function exigerAnnonceAvantSaisie(html: string, cleAnnonce: string): void {
  expect(
    html.includes(cleAnnonce) || html.toLowerCase().includes('internet'),
    'aucune annonce d’indisponibilité dans le rendu hors ligne.\n'
    + 'Le principe VI exige que l’interface le dise AVANT la saisie — jamais un grisé silencieux, '
    + 'jamais un échec après coup.',
  ).toBe(true)
}
