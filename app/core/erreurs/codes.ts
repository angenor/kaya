/**
 * **Les codes d'erreur métier du contrat → leur clé i18n.** Table unique, explicite et fermée.
 *
 * # Pourquoi une table partagée, et pas une par module
 *
 * Chaque module avait la sienne au cycle 002, et c'était juste : ses codes lui étaient propres.
 * CPT-01 change la donne — `permission_absente` est rendu par **dix-neuf** opérations sur
 * quarante-trois, et `etablissement_inconnu` par trois modules différents. Trois copies de la
 * même correspondance divergent, et **celle qui dérive est toujours celle qu'on ne relit pas** :
 * l'écran qui garde l'ancienne clé affiche une phrase générique là où les autres expliquent.
 *
 * # Ce que la table garantit, et ce qu'elle ne garantit pas
 *
 * Elle garantit qu'un code **connu** rend la même phrase partout. Elle ne garantit pas qu'un code
 * inconnu soit impossible : `cleDeRefus` rend alors [`REFUS_INATTENDU`], une phrase honnête et
 * générique — mieux qu'une clé i18n affichée telle quelle, qui est ce que produit une table
 * ouverte.
 *
 * `app/tests/permissions.spec.ts` vérifie que **les dix codes métier du contrat** y figurent : une
 * table qui perdrait une entrée rendrait la phrase générique sans que rien n'échoue.
 *
 * # Ce que le front ne fait JAMAIS
 *
 * Il ne lit pas `message`. `CorpsErreur` en porte un, et c'est un **diagnostic** : anglais
 * technique, il nomme des tables. L'afficher mettrait `comptes.compte_role` sous les yeux de
 * l'exploitant. Seul `code` est branché, et `motif_cle` prime quand le serveur en fournit une.
 */

/** Ce qu'on affiche d'un refus qu'on ne sait pas nommer. */
export const REFUS_INATTENDU = 'erreurs.inattendue'

/**
 * Les dix codes d'erreur métier de `contracts/http-api.md`, et rien d'autre.
 *
 * L'ordre suit celui du contrat, ce qui rend la comparaison lisible à l'œil.
 */
export const CLES_ERREUR_METIER: Readonly<Record<string, string>> = {
  // 401 — le seul code d'échec d'authentification. Il ne distingue jamais compte inconnu,
  // mot de passe faux, compte désactivé ni dépassement de tentatives (FR-012).
  identifiants_invalides: 'connexion.refus.identifiants',
  // 401 — jeton de rafraîchissement inconnu, révoqué ou déjà consommé.
  session_invalide: 'connexion.refus.session_expiree',
  // 403 — **l'interface ne devrait jamais le provoquer** : une action sans permission est
  // *absente*, pas refusée (FR-026). La phrase existe pour le seul chemin qui y mène — des
  // droits qui changent pendant qu'on regarde l'écran.
  permission_absente: 'erreurs.permission_absente',
  // 422 — `OTP_SMS`. Refus **nommé**, jamais un repli silencieux sur le mot de passe (FR-008).
  methode_non_implementee: 'connexion.refus.methode_non_implementee',
  // 422 — ni téléphone ni courriel à la création d'un compte.
  identifiant_absent: 'erreurs.identifiant_absent',
  // 422 — trop court **ou** figurant parmi les mots de passe compromis. Le motif est explicite :
  // l'utilisateur doit savoir quoi corriger. C'est l'inverse exact du refus d'authentification,
  // où le silence protège.
  mot_de_passe_refuse: 'erreurs.mot_de_passe_refuse',
  // 422 — la création est impossible, **sans dire pourquoi**. Dire « ce numéro est déjà pris »
  // apprendrait, à un habilité d'un tenant, quels numéros sont clients de Kaya.
  identifiant_refuse: 'erreurs.identifiant_refuse',
  // 422 — `etablissement_id` fourni pour `admin_editeur`, ou absent pour un rôle d'établissement.
  portee_incompatible: 'comptes.refus.portee_incompatible',
  // 404 — vérifié par trait, jamais par clé étrangère.
  etablissement_inconnu: 'comptes.refus.etablissement_inconnu',
  // 409 — **le seul refus métier du cycle**, et il est irréversible sans l'éditeur (FR-023).
  derniere_habilitation: 'comptes.refus.derniere_habilitation',
}

/**
 * Les motifs de `mot_de_passe_refuse`, rendus dans `valeur`.
 *
 * Le code seul dirait « refusé » ; l'utilisateur doit savoir **quoi corriger**. Un mot de passe
 * trop court et un mot de passe compromis demandent deux gestes différents — allonger, ou changer
 * complètement.
 */
export const CLES_MOTIF_MOT_DE_PASSE: Readonly<Record<string, string>> = {
  mot_de_passe_trop_court: 'erreurs.mot_de_passe.trop_court',
  mot_de_passe_trop_long: 'erreurs.mot_de_passe.trop_long',
  mot_de_passe_compromis: 'erreurs.mot_de_passe.compromis',
}

/**
 * La clé i18n d'un refus.
 *
 * @param code     Le `code` du corps d'erreur. **Jamais son `message`.**
 * @param motifCle Le `motif_cle` fourni par le référentiel, s'il y en a un. **Il prime** : c'est
 *   ce qui permet à deux refus de même code de dire deux choses différentes — l'un enseigne, l'autre
 *   constate.
 * @param table    Correspondances propres à un module, consultées **avant** la table partagée.
 */
export function cleDeRefus(
  code: string | undefined,
  motifCle?: string | null,
  table: Readonly<Record<string, string>> = {},
): string {
  if (motifCle) {
    return motifCle
  }
  if (!code) {
    return REFUS_INATTENDU
  }
  return table[code] ?? CLES_ERREUR_METIER[code] ?? REFUS_INATTENDU
}
