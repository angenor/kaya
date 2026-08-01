# Système de design

**Un seul composant CANONIQUE est écrit à ce jour : `ChampSaisie.vue`, le n° 16.**

Deux autres fichiers vivent ici sans être des composants du canon, et il vaut mieux le dire que
laisser croire à un inventaire :

| Fichier | Ce que c'est |
|---|---|
| `VitrineTheme.vue` | Le cadre à deux volets du styleguide — même contenu rendu en clair et en sombre. Outil de vérification, employé par aucun écran du produit. |
| `montage.ts` | La décision « le styleguide est-il monté ? », consommée par le hook `pages:extend` de `nuxt.config.ts`. Même mécanisme que `swagger_ui_activee()` au cycle 001. |

**Le styleguide servi par l'application est `app/pages/styleguide.vue`** — les seize composants dans
tous leurs états, avec les polices **réellement embarquées**. C'est ce que
`docs/design/styleguide.html` ne peut pas montrer : il charge les siennes depuis Google Fonts, donc
il affichera toujours les vraies, y compris le jour où l'application tombe en repli.

```sh
KAYA_STYLEGUIDE=1 pnpm --filter @kaya/app dev    # puis /styleguide
```

Ce n'est pas un début d'inventaire, c'est l'application de la règle : « du code concret se
refactore, une abstraction prématurée se subit » (constitution, § Flux de développement). Les
quinze autres composants canoniques **existent en classes Tailwind**, lues dans
`docs/design/composants.md` et posées dans les écrans qui les emploient. Ils ne deviendront des
composants Vue que le jour où **un second écran** en aura besoin — pas avant.

Le n° 16 fait exception pour une raison précise : c'est le seul qui porte de l'**état** (valeur,
erreur, lecture seule, désactivé) et des **règles d'accessibilité** (liaison `label`/`for`,
`aria-invalid`, `aria-describedby`) que recopier à la main ferait diverger dès le deuxième
formulaire. Un bandeau d'alerte se recopie sans risque ; un champ, non.

---

## `ChampSaisie.vue` — le n° 16

| Ce qu'il faut savoir | Où c'est écrit |
|---|---|
| Rôle, états, classes, règles | `docs/design/composants.md` § « 16 · Champ de saisie » |
| Tous ses états, clair et sombre | `docs/design/styleguide.html` § `#c16` |
| Son emploi réel dans un formulaire | `app/modules/etablissements/SectionServices.vue` |
| Le patron d'écriture qui l'entoure | `docs/module-dore.md` § « La septième couche » |

**Il reçoit des clés i18n, jamais du texte.** Une chaîne en dur passée en prop afficherait la clé
brute au premier rendu — la faute se voit tout de suite, au lieu d'attendre qu'un anglophone ouvre
l'application.

---

## Ce qui reste dû à TRX-08

`TRX-08` est une story **P1**. Son inventaire fait foi dans `docs/design/composants.md`, qui compte
désormais **16 composants** — les 14 de la série d'origine, le n° 15 en attente de validation, et
le n° 16 arrivé avec la première écriture depuis un écran.

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

### Trois règles qui valent pour chacun

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
