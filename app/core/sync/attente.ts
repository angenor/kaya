/**
 * **Ce qui n'est pas encore parti** — la question qu'il faut pouvoir poser avant de quitter.
 *
 * # Pourquoi ce fichier existe MAINTENANT, alors que la file est débranchée
 *
 * `FileLocale` n'est instanciée nulle part : `TYPES_CLASSE_A` ne porte que
 * `note_etablissement.creee`, dont l'écran n'existe pas. La file est donc **structurellement
 * vide**, et le restera jusqu'à SYN-01.
 *
 * Ce n'est pas une raison de reporter la garde, c'est la raison de la poser tout de suite. Le jour
 * où SYN branche la file, le bouton « passer la main » existera **déjà**, sur toutes les pages, et
 * personne n'ira relire son code pour y ajouter une question qu'il ne se posait pas. Une
 * déconnexion emporte le stockage de la plateforme (`oublierRafraichissement`, principe VI) : des
 * écritures en attente y passeraient avec le reste, silencieusement, et la serveuse apprendrait au
 * service suivant que ses quatre commandes n'existent pas.
 *
 * Poser la garde coûte quinze lignes aujourd'hui. La poser après coup coûte de retrouver, dans un
 * cycle qui parle d'autre chose, tous les chemins qui vident le stockage.
 *
 * # Ce que la garde vaut aujourd'hui, écrit plutôt que supposé
 *
 * {@link ecrituresEnAttente} rend **0**, parce qu'aucune file n'est branchée — pas parce qu'elle
 * aurait constaté qu'une file est vide. La distinction est réelle et `app/tests/deconnexion.spec.ts`
 * la vérifie **dans les deux sens** : elle rend 0 à l'état débranché, et elle rend le compte réel
 * dès qu'une file est branchée. Sans le second versant, la fonction pourrait rendre `0` en dur et
 * tous les tests passeraient — c'est le corollaire du versant positif de l'exigence 4.
 *
 * # Une variable de module, comme l'état de session
 *
 * Même raisonnement que `core/auth/session.ts` : un magasin réactif ajouterait une dépendance et
 * une seconde façon d'accéder à la même chose. Une variable, deux fonctions qui la touchent.
 */

import type { FileLocale } from './index'

/**
 * La file du produit — **`null` tant que SYN-01 n'en a branché aucune**.
 *
 * `null` et « file vide » sont deux états distincts, et les confondre serait l'erreur : l'un dit
 * « personne ne sait », l'autre dit « rien n'attend ». {@link ecrituresEnAttente} les ramène tous
 * deux à 0 parce que c'est la seule réponse honnête à l'usage qu'on en fait — refuser une
 * déconnexion parce qu'aucune file n'est branchée bloquerait le produit entier —, mais
 * {@link fileBranchee} permet de les distinguer là où la distinction compte : dans un test.
 */
let file: FileLocale | null = null

/**
 * Branche la file du produit.
 *
 * **Aucun appelant à ce jour**, et c'est déclaré comme tel dans `app/tests/amorcage.spec.ts` :
 * le harnais échoue le jour où un appelant apparaît sans que la ligne passe à « branché ». C'est
 * ce qui garantit que le branchement ne se fera pas en silence.
 *
 * Accepte `null` pour rendre le débranchement possible — un test qui branche une file doit pouvoir
 * remettre l'état initial, sinon il contamine les suivants.
 */
export function brancherFile(nouvelle: FileLocale | null): void {
  file = nouvelle
}

/** Une file est-elle branchée ? **Distinct de « la file est vide »** — voir {@link file}. */
export function fileBranchee(): boolean {
  return file !== null
}

/**
 * La file branchée, ou `null`.
 *
 * # Pourquoi ce point d'accès existe, et ce qu'il n'ouvre pas
 *
 * L'état de synchronisation et l'envoi ont besoin de la file elle-même — pour lire la quarantaine,
 * pour envoyer. Ils ne peuvent pas la recevoir en paramètre : ils sont appelés depuis un composant
 * ou un plugin, qui n'a aucune raison de la tenir.
 *
 * **Ce n'est pas une seconde sortie de la file.** {@link viderFile} reste le seul chemin par
 * lequel une écriture en sort, et c'est lui qui porte l'ordre rafraîchir-avant-vider. Ce que cette
 * fonction rend est la file, pas une écriture ; ce qu'on en fait reste soumis à la même règle.
 */
export function fileCourante(): FileLocale | null {
  return file
}

/**
 * Combien d'écritures attendent d'être envoyées.
 *
 * `0` aujourd'hui, toujours, parce qu'aucune file n'est branchée. C'est la valeur qu'il faut : le
 * geste qu'elle garde — passer la main — ne doit pas devenir impossible sous prétexte que la
 * synchronisation n'est pas écrite.
 */
export function ecrituresEnAttente(): number {
  return file?.enAttente ?? 0
}
