/**
 * Les fichiers de licence sont importés **en texte**, pas référencés par un chemin.
 *
 * C'est ce qui les fait entrer dans le paquet distribué : un `.txt` de `assets/` que rien n'importe
 * n'est pas copié dans la sortie de construction, et l'obligation d'accompagnement de la clause 2
 * de l'OFL — « the above copyright notice and this license notice shall be included in all copies »
 * — ne serait alors satisfaite que dans le dépôt, pas chez le client.
 *
 * Vite fournit le suffixe `?raw` ; TypeScript a besoin de cette déclaration pour l'accepter.
 */
declare module '*.txt?raw' {
  const contenu: string
  export default contenu
}
