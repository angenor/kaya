/**
 * Mode sombre — **par la variante `dark:`, jamais par une seconde palette** (principe XII).
 *
 * # Comment il fonctionne, et pourquoi c'est presque vide
 *
 * `app/assets/css/theme.css` — copie exacte de `docs/design/theme.css` — déclare :
 *
 * ```css
 * @custom-variant dark (&:where(.dark, .dark *));
 * ```
 *
 * et redéfinit les **valeurs** des mêmes tokens sous `.dark`. Les noms ne changent pas : `bg-surf
 * text-ink` bascule donc tout seul, sans qu'aucun composant n'ait à connaître le mode. La
 * variante `dark:` ne sert qu'à ce qu'une couleur ne peut pas porter — une ombre remplacée par
 * une bordure, une opacité, une épaisseur de trait.
 *
 * Ce module ne fait donc qu'une chose : poser ou retirer la classe `.dark` sur `<html>`. Tout le
 * reste est dans le thème.
 *
 * **Le mode sombre est câblé dès le premier écran, jamais rétrofitté** (principe VIII) — le
 * rétrofit coûte plusieurs fois son prix initial. Ce cycle ne produit aucun écran ; il livre le
 * mécanisme pour que le premier, au cycle ETB, l'ait d'emblée.
 */

export type ModeTheme = 'clair' | 'sombre' | 'systeme'

/**
 * Clé de stockage du mode retenu.
 *
 * **Exportée parce qu'un second lecteur existe, et qu'il ne peut pas importer ce module** : le
 * script en ligne de `nuxt.config.ts` s'exécute dans le `<head>`, avant le moindre module
 * JavaScript. Sans lui, la page se peint en clair puis bascule — un scintillement pire que pas de
 * mode sombre du tout. Ce script est le seul autre endroit qui connaisse cette clé, et il la
 * connaît **par cette constante**, pas par une chaîne recopiée : deux littéraux `'kaya.theme'`
 * finiraient par en désigner deux.
 */
export const CLE_STOCKAGE = 'kaya.theme'

/** Mode choisi par l'utilisateur, ou `systeme` par défaut. */
export function modeChoisi(): ModeTheme {
  if (typeof localStorage === 'undefined') {
    return 'systeme'
  }
  const valeur = localStorage.getItem(CLE_STOCKAGE)
  return valeur === 'clair' || valeur === 'sombre' ? valeur : 'systeme'
}

/** Le système préfère-t-il le sombre ? */
export function systemePrefereSombre(): boolean {
  if (typeof matchMedia === 'undefined') {
    return false
  }
  return matchMedia('(prefers-color-scheme: dark)').matches
}

/** Applique un mode et le retient. */
export function appliquerMode(mode: ModeTheme): void {
  if (typeof document === 'undefined') {
    return
  }

  const sombre = mode === 'sombre' || (mode === 'systeme' && systemePrefereSombre())
  document.documentElement.classList.toggle('dark', sombre)

  if (typeof localStorage !== 'undefined') {
    if (mode === 'systeme') {
      localStorage.removeItem(CLE_STOCKAGE)
    } else {
      localStorage.setItem(CLE_STOCKAGE, mode)
    }
  }
}

/** Applique le mode retenu — à appeler au démarrage de l'application. */
export function initialiserTheme(): void {
  appliquerMode(modeChoisi())
}
