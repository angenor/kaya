/**
 * **La fonction unique de formatage des montants.**
 *
 * `docs/design/tokens.md` §2, règle des montants **sans exception** : « Un montant s'écrit avec une
 * espace fine insécable **U+202F** entre les groupes de milliers *et* avant le F : `12 500 F`,
 * jamais `12 500 F`. C'est ce qui empêche un montant de se couper en fin de ligne et ce qui aligne
 * les colonnes en Chivo Mono tabulaire. **Côté code, une seule fonction la porte.** »
 *
 * Ce fichier est cette fonction, et il n'y en a pas d'autre. Le test `tests/montant.spec.ts` le
 * vérifie en parcourant `core/`, `modules/` et `pages/` : toute seconde implémentation le fait
 * échouer. Sans ce contrôle, le premier cycle qui affiche un tarif (HEB) ou un encaissement (CAI)
 * écrirait la sienne dans son coin, avec une espace ordinaire — et **tout le travail sur U+202F ne
 * servirait à rien** : le montant se couperait en fin de ligne et les colonnes ne s'aligneraient
 * plus.
 *
 * # Ce que la signature de `tokens.md` ne pouvait pas porter
 *
 * `tokens.md` §2 donne un extrait de maquette, `money(n)`. Il est juste pour une maquette et faux
 * pour la production, qui a deux contraintes que cette signature ignore :
 *
 * - **Principe V** — un montant est un **entier en unités mineures** plus un **code ISO 4217 porté
 *   par l'établissement**. `money(12500)` suppose que 12 500 est déjà le nombre à écrire ; ici,
 *   12 500 est le nombre d'unités mineures, et c'est la devise qui dit combien de décimales en
 *   sortir. XOF en a zéro, donc les deux coïncident — et c'est précisément ce qui rend la faute
 *   invisible tant qu'une seule devise existe.
 * - **Principe X** — les devises actives sont une **provision** du cadrage §14 : le modèle doit
 *   être prêt sans que la logique soit construite. Une fonction `money(n)` obligerait à reprendre
 *   **chaque appel** le jour de la deuxième devise, c'est-à-dire exactement la migration que le
 *   principe X existe pour éviter.
 *
 * D'où la signature : le montant **et** la devise. Le nombre de décimales et le symbole viennent de
 * la devise, jamais d'une constante.
 *
 * # Pourquoi pas `Intl.NumberFormat`, alors qu'il sait grouper les milliers
 *
 * Parce qu'il ne garantit pas le caractère. `Intl.NumberFormat('fr-FR')` produit U+202F sur les ICU
 * récents et U+00A0 sur les plus anciens — le passage de l'un à l'autre est une modification d'ICU,
 * pas de notre code, et elle ne se verrait sur aucun test. Or `tokens.md` §2 ne demande pas « une
 * espace insécable » : il demande **U+202F**, et en fait la condition de l'alignement des colonnes
 * en Chivo Mono. Le groupement est écrit à la main pour que le caractère soit **le nôtre**, et le
 * test le vérifie par code de caractère, pas à l'œil.
 *
 * # Les heures ne passent PAS par ici
 *
 * `tokens.md` §2, juste après la règle des montants : « Les heures gardent une espace ordinaire
 * (`17 h 30`) : elles ne se coupent pas, la lettre `h` tient les deux nombres ensemble. » Ce module
 * ne formate donc **que** des montants, et n'expose aucun formateur d'heure — le test le vérifie
 * aussi, pour qu'on n'en ajoute pas un ici par mimétisme, avec la fine.
 */

/**
 * **U+202F — NARROW NO-BREAK SPACE.** Le caractère de la règle, écrit en échappement plutôt qu'en
 * clair : à l'œil, dans un éditeur, il est indistinguable de U+00A0 et de U+2009.
 *
 * Il n'est dessiné **ni par Archivo ni par Chivo Mono** à la source. `app/scripts/generer-polices.ts`
 * l'ajoute à leur table `cmap` (associé au dessin de U+2009) et la porte P-21b relit la table pour
 * le vérifier. Sans cela, chaque montant ferait tomber son séparateur sur une police de repli, de
 * chasse inconnue.
 */
const FINE_INSECABLE = '\u202F'

/**
 * **Le séparateur décimal, et la limite assumée qui va avec.**
 *
 * La virgule est la convention française, et `fr` est la langue par défaut du produit (principe
 * VIII). Elle n'est **jamais affichée au MVP** : XOF a zéro décimale, donc aucun montant du produit
 * ne porte de partie décimale aujourd'hui.
 *
 * Ce qui n'est pas fait, et qui est une décision plutôt qu'un oubli : le séparateur ne suit pas la
 * langue de l'interface. Un anglophone verrait `12,50 €` là où il attend `12.50 €`. Le trancher
 * demanderait de faire dépendre cette fonction de la locale courante, donc du pipeline i18n, pour
 * un cas que le MVP ne produit pas. À reprendre avec la **première devise à décimales réellement
 * activée** — pas avant.
 */
const SEPARATEUR_DECIMAL = ','

/**
 * Ce qu'il faut savoir d'une devise pour écrire un montant : rien de plus.
 *
 * Ni son nom, ni son taux, ni sa place dans un catalogue. Les deux seules propriétés qui changent
 * la chaîne produite sont le nombre de décimales et le symbole.
 */
export interface Devise {
  /** Code **ISO 4217** — celui que l'établissement porte en base (`backend/crates/domain/monnaie.rs`). */
  readonly code: string
  /** Nombre de décimales de la devise. **XOF : 0.** C'est lui qui convertit les unités mineures. */
  readonly decimales: number
  /** Ce qui s'écrit après le montant, séparé par la fine insécable. */
  readonly symbole: string
}

/**
 * **Le référentiel des devises que le produit sait écrire** — un modèle, pas une fonctionnalité.
 *
 * `XOF` est la seule devise **active** au MVP : c'est le serveur qui la fige à la création de
 * l'établissement (`devise_figee`), pas ce fichier. Sa présence ici ne l'active pas davantage que
 * son absence ne la refuserait.
 *
 * **`EUR` est là au titre du principe X, « prêt ≠ construit »** : le cadrage §14 provisionne les
 * devises actives, et une provision est « un choix de modèle de données et d'interfaces uniquement,
 * aucune UI, aucune logique ». Deux entrées valent mieux qu'une pour une raison vérifiable : avec
 * une seule devise à zéro décimale, **rien ne distingue une fonction qui lit la devise d'une
 * fonction qui suppose XOF**. Le test s'appuie sur `EUR` pour faire exactement cette distinction.
 *
 * Le symbole du XOF est **`F`**, celui des maquettes et de `tokens.md` (`12 500 F`) — ni `FCFA`,
 * ni `XOF`, ni `₣`.
 */
const DEVISES: Readonly<Record<string, Devise>> = {
  XOF: { code: 'XOF', decimales: 0, symbole: 'F' },
  EUR: { code: 'EUR', decimales: 2, symbole: '€' },
}

/**
 * Une devise hors référentiel, ou un montant qui n'est pas un entier d'unité mineure.
 *
 * **Refusé explicitement, jamais ignoré.** Se rabattre sur deux décimales et le code ISO en guise
 * de symbole afficherait un montant faux sans que rien ne le dise — la faute la plus coûteuse du
 * principe V, puisqu'elle se réplique chez tous les clients avant d'être vue.
 */
export class MontantNonFormatable extends Error {
  constructor(message: string) {
    super(message)
    this.name = 'MontantNonFormatable'
  }
}

/**
 * Résout un code ISO 4217 en devise, ou refuse.
 *
 * Exportée parce qu'un écran qui affiche une colonne de montants a besoin de savoir combien de
 * décimales attendre **avant** de mettre en forme — pour dimensionner la colonne, par exemple.
 */
export function deviseDe(code: string): Devise {
  const devise = DEVISES[code]
  if (!devise) {
    throw new MontantNonFormatable(
      `Devise « ${code} » hors référentiel. Les devises connues sont ${Object.keys(DEVISES).join(', ')}. `
      + 'Une devise inconnue n\'est pas formatée au hasard : le nombre de décimales est une propriété '
      + 'de la devise (principe V), et se tromper produit un montant faux que rien ne signale.',
    )
  }
  return devise
}

/**
 * Écrit un montant selon la règle de `tokens.md` §2.
 *
 * @param montantMineur Montant en **unités mineures** de la devise — un entier, jamais un flottant
 *   (principe V, porte P-10). `12500` en XOF vaut douze mille cinq cents francs ; `1250` en EUR
 *   vaut douze euros cinquante.
 * @param codeDevise Code **ISO 4217** porté par l'établissement.
 *
 * @returns La chaîne à afficher, par exemple `12 500 F` — où les deux espaces sont U+202F.
 *
 * @throws {MontantNonFormatable} si la devise est hors référentiel ou si le montant n'est pas un
 *   entier fini.
 *
 * Tout élément qui porte le résultat reçoit en plus `whitespace-nowrap` (`tokens.md` §2) : la fine
 * insécable empêche la coupure **à l'intérieur** du montant, elle ne dit rien du reste de la ligne.
 */
export function formaterMontant(montantMineur: number, codeDevise: string): string {
  const devise = deviseDe(codeDevise)

  // Un montant à virgule n'existe pas : c'est le sens même de « entier d'unité mineure ». Le
  // laisser passer produirait `12,5 500 F` ou un arrondi silencieux, selon le chemin — deux façons
  // d'afficher un chiffre faux. La porte P-10 tient la même règle côté base et côté JSONB.
  if (!Number.isInteger(montantMineur)) {
    throw new MontantNonFormatable(
      `Montant « ${montantMineur} » : un montant est un ENTIER en unités mineures (principe V). `
      + 'Un décimal ici vient presque toujours d\'un calcul fait en unités majeures — c\'est le calcul '
      + 'qu\'il faut reprendre, pas l\'affichage.',
    )
  }

  const negatif = montantMineur < 0
  const absolu = Math.abs(montantMineur)

  // Séparation des unités majeures et mineures. En XOF, `facteur` vaut 1 et `reste` est toujours
  // nul : la branche décimale ne s'exécute simplement jamais.
  const facteur = 10 ** devise.decimales
  const entier = Math.trunc(absolu / facteur)
  const reste = absolu % facteur

  let ecrit = grouperMilliers(entier)
  if (devise.decimales > 0) {
    ecrit += SEPARATEUR_DECIMAL + String(reste).padStart(devise.decimales, '0')
  }

  // Le signe est posé ici et pas dans le groupement : appliquer une expression de groupement à
  // « -12500 » marche par accident aujourd'hui et casse le jour où l'on change de séparateur.
  return `${negatif ? '-' : ''}${ecrit}${FINE_INSECABLE}${devise.symbole}`
}

/**
 * Insère la fine insécable tous les trois chiffres, en partant de la droite.
 *
 * Écrit sur la chaîne plutôt qu'avec l'expression régulière de `tokens.md` (`\B(?=(\d{3})+(?!\d))`)
 * pour une raison de lisibilité, pas de correction : les deux donnent le même résultat sur une
 * suite de chiffres, mais celle-ci ne peut pas se tromper de cible si un signe ou un séparateur
 * décimal se retrouvait un jour dans la chaîne d'entrée.
 */
function grouperMilliers(entier: number): string {
  const chiffres = String(entier)
  let groupe = ''
  for (let i = 0; i < chiffres.length; i += 1) {
    // Position depuis la droite : on coupe avant chaque multiple de trois, jamais en tête.
    const restants = chiffres.length - i
    if (i > 0 && restants % 3 === 0) groupe += FINE_INSECABLE
    groupe += chiffres[i]
  }
  return groupe
}
