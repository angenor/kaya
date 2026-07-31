/**
 * **Porte P-13** — aucune opération B, C ou D atteignable depuis un chemin exécutable hors ligne.
 *
 * # Le mécanisme : le type, pas la convention
 *
 * La file locale n'accepte que des opérations **marquées classe A au niveau du type**. Enregistrer
 * une opération non marquée ne compile pas.
 *
 * C'est ce qui distingue cette porte d'une revue de code. Le registre
 * `docs/registre-classes-offline.md` classe chaque entité ; rien n'empêcherait un développeur de
 * mettre en file une opération de classe B « en attendant le réseau », et l'erreur ne se verrait
 * qu'en production, sur un encaissement rejoué deux fois.
 *
 * # Pourquoi une marque plutôt qu'un champ ordinaire
 *
 * TypeScript est structurellement typé : une interface avec `classe: 'A'` serait satisfaite par
 * n'importe quel objet portant cette chaîne, y compris écrite à la volée pour faire passer le
 * compilateur. Le symbole unique ci-dessous ne peut pas être fabriqué par un appelant — la seule
 * façon d'obtenir une valeur marquée est de passer par {@link marquerClasseA}, qui est le point
 * où la question « cette opération est-elle vraiment de classe A ? » se pose.
 *
 * # Ce que la marque ne garantit pas
 *
 * Elle garantit qu'**une décision a été prise**, pas qu'elle est juste. Marquer une opération de
 * classe B ne produirait aucune erreur de compilation. La justesse reste humaine et revue
 * mensuellement — même limite que la porte du registre côté backend, et pour la même raison : le
 * registre classe des opérations, pas des types.
 */

declare const MARQUE_CLASSE_A: unique symbol

/**
 * Charge utile d'une opération **de classe A**, seule admise dans la file locale.
 *
 * Classe A (cadrage §11.2, branche A4) : append-only, commutative, sans contrainte d'unicité
 * métier, sans effet monétaire. Rejeu inoffensif, ordre d'arrivée indifférent.
 */
export type OperationClasseA<T = unknown> = T & {
  readonly [MARQUE_CLASSE_A]: true
}

/** Entrée de la file locale. */
export interface EntreeFile<T = unknown> {
  /** UUID v7 **généré par le client** — c'est lui qui rend le rejeu inoffensif (principe VI). */
  readonly id: string
  /** Type d'opération, par exemple `note_etablissement.creee`. */
  readonly type: string
  /** Horodatage du terminal. **Indicatif** : ordre d'affichage local, jamais une règle. */
  readonly horodatageClient: string
  readonly charge: OperationClasseA<T>
}

/**
 * Marque une opération comme étant de classe A.
 *
 * **C'est le seul point d'entrée de la file, et c'est délibéré** : chaque appel est l'endroit où
 * un relecteur peut poser la question. Le paramètre `justification` n'est pas décoratif — il
 * force à nommer la branche de l'arbre de décision du cadrage §11.2, et un appel sans
 * justification recevable se voit en revue.
 *
 * @param charge        L'opération à mettre en file.
 * @param justification Branche de l'arbre de décision, par exemple `'A4 — append-only, sans effet monétaire'`.
 */
export function marquerClasseA<T>(charge: T, justification: string): OperationClasseA<T> {
  if (!justification || justification.trim().length === 0) {
    throw new Error(
      'marquerClasseA exige une justification : la branche de l’arbre de décision du cadrage §11.2. '
        + 'Sans elle, le classement n’a été ni décidé ni relu.',
    )
  }
  return charge as OperationClasseA<T>
}

/**
 * Types d'opération reconnus comme classe A par le registre.
 *
 * Liste **explicite**, tenue à jour avec `docs/registre-classes-offline.md`. Une opération dont le
 * type n'y figure pas est refusée à l'exécution, même marquée — seconde barrière, pour le cas où
 * `marquerClasseA` aurait été appelée sur une opération qui ne le mérite pas.
 */
export const TYPES_CLASSE_A: readonly string[] = [
  'note_etablissement.creee',
  // Ajouter ici toute opération de classe A, dans le même changement que sa déclaration au
  // registre. Les cycles suivants remplissent cette liste : sélection d'établissement actif
  // (ETB-06), journal d'audit (CPT-04), relevé de position (CPT-06), ouverture de tiroir
  // (IMP-01)…
]

/** Ce type d'opération est-il déclaré de classe A ? */
export function estTypeClasseA(type: string): boolean {
  return TYPES_CLASSE_A.includes(type)
}
