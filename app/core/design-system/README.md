# Système de design — 12 composants canoniques

**EMPLACEMENT SEUL. Aucun composant n'est écrit au cycle 001.**

`TRX-08` est une story **P1**. Et il y a une seconde raison, plus contraignante : **ce cycle ne
produit aucun écran**. Écrire des composants sans écran qui les consomme reviendrait à concevoir
une abstraction avant son premier usage — « du code concret se refactore, une abstraction
prématurée se subit » (constitution, § Flux de développement).

---

## Déjà en place, livré par ce cycle

| Élément | Où |
|---|---|
| **Jetons de design** | `app/assets/css/theme.css` — **copie exacte** de `docs/design/theme.css`, seule exception du principe XII |
| **Mode sombre** | `app/core/theme/` — variante `dark:`, jamais une seconde palette |
| **i18n fr/en** | `app/core/i18n/` — `fr` par défaut, parité vérifiée par la porte P-16 |
| **Porte des jetons** | `pnpm lint:tokens` — P-17, échoue sur toute couleur ou espacement littéral |
| **Porte du natif** | `app/eslint.config.js` — P-15, `@tauri-apps/api` interdit hors de `core/platform/` |

Un composant écrit au cycle ETB héritera donc de tout cela sans rien câbler.

---

## Ce qui reste dû à TRX-08

Les **12 composants canoniques**. Leur inventaire fait foi dans `docs/design/composants.md` — qui
en liste **14**. L'écart entre 12 et 14 est à trancher au moment de les écrire, en confrontant les
deux documents, plutôt qu'en choisissant l'un des deux nombres maintenant.

### Trois règles qui vaudront pour chacun

1. **Tailwind d'abord, CSS en dernier recours.** Utilitaires du noyau référençant les jetons de
   `@theme`. Aucune classe personnalisée, aucun style en ligne. Le CSS explicite est réservé à ce
   que Tailwind n'exprime pas — `@keyframes`, impression thermique — et **reste regroupé** en un
   seul endroit.
2. **Le HTML de `docs/design/html/` n'est jamais copié.** C'est une cible, pas une source : il est
   autonome, non sémantique, sans i18n, sans RBAC. On lit ses valeurs, on réimplémente. La porte
   **P-19** le vérifie par empreinte.
3. **Chaque composant est vérifié en mode clair ET en mode sombre** avant d'être considéré comme
   terminé (Definition of Done, point 8). Rétrofiter le mode sombre coûte plusieurs fois son prix
   initial.

### Un écran ne se code que s'il hérite d'un motif

Soit un fichier de `docs/design/html/` — les onze codes `C4`, `F2`, `G2`, `M4`, `P2`, `Q1`, `R1`,
`R4`, `R7`, `S2`, `V1` — soit une ligne de la matrice `docs/design/derivation.md`. **Aucun écran
inventé.**

C'est cette règle qui a fait reporter la couche écran du module doré au cycle ETB : l'écran de
notes internes n'apparaît ni dans l'un ni dans l'autre. Voir `docs/module-dore.md`, section « La
septième couche, et pourquoi elle manque ».
