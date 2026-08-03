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

/**
 * Le contexte d'une écriture, **figé au moment de la saisie**.
 *
 * # Le défaut que ce champ existe pour empêcher, et il est silencieux
 *
 * Aminata saisit quatre commandes hors ligne sur l'établissement A, puis change d'établissement
 * actif — geste normal pour une gérante de deux structures. Le réseau revient.
 *
 * Sans ce champ, la file relirait le contexte **à l'envoi** et les quatre écritures partiraient
 * sur l'établissement B. Rien n'échouerait : les identifiants sont valides, le serveur accepte, et
 * la faute ne se voit qu'à la clôture — quand le chiffre d'affaires de A manque et que celui de B
 * est faux. Impossible à démêler après coup, puisque rien ne dit d'où venait chaque ligne.
 */
export interface ContexteEcriture {
  readonly tenantId: string
  readonly etablissementId: string
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
  /** Le contexte **au moment de la saisie**, jamais relu à l'envoi. Voir {@link ContexteEcriture}. */
  readonly contexte: ContexteEcriture
  /**
   * Combien de fois l'envoi a été tenté.
   *
   * Alimente l'intervalle croissant de réessai, et le diagnostic de l'écran `S1` — « cette
   * écriture a été tentée sept fois » est une information que l'exploitant peut porter au support,
   * là où « en attente » ne dit rien.
   */
  readonly tentatives: number
}

/**
 * Ce que la file NE PORTE PAS, et qui est le point : **aucun jeton**.
 *
 * L'absence de champ est ce qui l'empêche. Un jeton mis en file serait **périmé au retour** — le
 * jeton d'accès dure soixante minutes, une coupure de service en dure quatre-vingt-dix — et le
 * ranger prolongerait la durée de vie d'un secret sur un terminal qu'on peut perdre.
 *
 * C'est pourquoi le vidage rafraîchit **avant** d'envoyer, et pourquoi il est le seul chemin de
 * sortie de la file : l'ordre est porté par une fonction, pas par la discipline des appelants.
 */
export type AucunJetonEnFile = never

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
  //
  // **Le cycle 005 n'en ajoute AUCUNE, et c'est une décision.** La file devient réelle avec le
  // seul passager qu'elle avait déjà — la note interne. Y verser un type de plus « puisqu'on y
  // est » ferait entrer une opération dont personne n'a rouvert le registre pour vérifier la
  // classe, et c'est exactement le moment où la question doit se poser.
]

/** Ce type d'opération est-il déclaré de classe A ? */
export function estTypeClasseA(type: string): boolean {
  return TYPES_CLASSE_A.includes(type)
}
