/**
 * Déclaration de `subset-font` — le paquet n'en publie aucune.
 *
 * Vérifié sur le registre le 2026-07-31 : `subset-font@2.5.0` expose `index.js` en CommonJS, sans
 * `types` ni `typings`, et aucun `@types/subset-font` n'existe. Sans cette déclaration, le
 * typecheck de `pnpm test` sort en échec permanent sur `scripts/generer-icones.ts` — exactement la
 * dette que le gel 1.0.7 vient de solder côté `@types/node`, et **un `pnpm test` rouge en
 * permanence est un `pnpm test` que personne ne lit**.
 *
 * La signature est réduite à ce que le générateur emploie. L'élargir « au cas où » donnerait
 * l'illusion d'un typage complet du paquet, qui n'existe pas.
 */
declare module 'subset-font' {
  /**
   * Produit un sous-ensemble d'une police, limité aux glyphes couvrant `texte`.
   *
   * @param police Contenu de la police source — TTF, WOFF ou WOFF2.
   * @param texte  Les caractères à conserver. Pour une police d'icônes, les points de code du
   *               plan d'usage privé lus dans le CSS amont, jamais devinés.
   */
  export default function subsetFont(
    police: Uint8Array,
    texte: string,
    options?: { targetFormat?: 'truetype' | 'woff' | 'woff2' },
  ): Promise<Uint8Array>
}
