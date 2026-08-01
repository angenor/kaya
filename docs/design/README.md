# kaya-design

Tout le travail de design de Kaya, en Tailwind 4, prêt à être lu par le développement.
Direction retenue : **Banco** — terre chaude, indigo réservé à l'action, Archivo + Chivo Mono.

---

## 1. Qu'est-ce que je copie dans mon projet ?

**`theme.css`. Et rien d'autre.**

Copiez-le dans votre projet Nuxt (par exemple `assets/css/theme.css`) et importez-le à la
place de votre `@import "tailwindcss"` — il le contient déjà. Il est autosuffisant : aucune
valeur de la maquette ne vit ailleurs.

```css
/* assets/css/main.css → remplacé par, ou important, theme.css */
@import "./theme.css";
```

Deux choses à savoir :

- **Le mode sombre passe par la classe `.dark`** (`@custom-variant dark`). Posez-la sur
  `<html>`. Les tokens de couleur portent les mêmes noms dans les deux modes et changent de
  valeur sous `.dark` : `bg-surf text-ink` bascule tout seul. La variante `dark:` ne sert
  qu'à ce qu'une couleur ne peut pas porter (une ombre remplacée par une bordure, une
  opacité, une épaisseur de trait). Si vous voulez aussi suivre la préférence système,
  la ligne à modifier est commentée en haut du fichier.
- **Les polices ne sont pas dans `theme.css`.** Les maquettes les chargent depuis Google
  Fonts ; en production, servez Archivo et Chivo Mono en local (woff2, `font-display: swap`)
  — le produit tourne sur des liaisons lentes et doit s'afficher hors ligne. Les noms de
  familles ne changent pas.

## 2. Qu'est-ce que je lis sans jamais le copier ?

**Tout le HTML.** `styleguide.html`, `fondation/`, `html/`, `documents/`, `proto/`.

Ce sont des maquettes : un seul fichier, chargé par le compilateur Tailwind du navigateur,
sans build. Elles montrent l'état exact à atteindre, avec les classes exactes à employer.
Elles ne sont pas structurées comme des composants Vue et ne doivent pas être importées :
on y lit les classes, on les remonte dans ses propres composants.

Ordre de lecture conseillé :

1. `tokens.md` — les valeurs et leur usage.
2. `composants.md` + `styleguide.html` côte à côte — les 16 pièces et tous leurs états.
3. `mouvement.md` — durées, courbes, les sept patrons.
4. `html/{code}-*.html` — l'écran que vous développez, un fichier par état.

Les **codes d'écran** (`R1`, `R4`, `P2`, `R7`, `C4`, `V1`, `M4`, `F2`, `S2`, `Q1`, `G2`,
`D1`–`D7`) sont stables : ce sont les identifiants à citer dans les tickets.

## 3. Quelles décisions restent à prendre ?

Onze, toutes ouvertes, listées par nombre d'occurrences dans l'archive complète (44 fichiers
HTML). Chacune est une valeur arbitraire employée dans les maquettes : soit elle entre dans
`@theme`, soit on s'aligne sur l'échelle existante. Rien n'est masqué.

Les six premières sont de vraies décisions. Les cinq dernières sont des constats : elles ne
demandent rien, elles sont là pour que vous n'ayez pas à vous demander pourquoi.

| Nº | Valeur arbitraire | Où | Occurrences | Décision |
| --- | --- | --- | --- | --- |
| 1 | `border-[1.5px]` `border-l-[3px]` `border-t-[2px]` | 21 fichiers — bouton secondaire, tuiles, pastilles de durée, contreforts de carte, pied de document | 116 | Tailwind 4 n'a pas d'espace de nom pour les épaisseurs de bordure. Deux épaisseurs sont concernées : **1,5 px** (le contour d'un bouton secondaire) et **3 px** (le contrefort d'une carte de vérification, plus fin que le contrefort de 4 px d'un bandeau). Trois issues : les arrondir à 1 px et 4 px et supprimer l'arbitraire ; les garder et assumer deux épaisseurs hors échelle ; ou ouvrir un espace de nom `--border-*` maison. **À trancher en premier** — c'est la valeur la plus répandue de l'archive. |
| 2 | `h-[3px]` `h-[2px]` | 9 fichiers — trait ocre de l'état vide, tiret de marque des lignes de document (10 × 3 px), filet de total | 88 | Un filet de 3 px n'est ni `h-0.5` (2 px) ni `h-1` (4 px). Soit 3 px entre dans le thème comme **épaisseur de marque**, soit on passe à 4 px partout — le tiret des documents s'épaissit alors visiblement. |
| 3 | `active:translate-y-[3px]` | 27 fichiers — tout bouton portant `shadow-bouton-grand` | 47 | Le grand bouton a 3 px de relief, donc 3 px de descente ; le bouton canonique `h-11` en a 2 (`active:translate-y-0.5`). Soit le grand bouton s'aligne sur 2 px et l'arbitraire disparaît, soit on assume **deux reliefs, un par taille**. |
| 4 | `transition-[transform,box-shadow]` et ses cinq variantes | 29 fichiers | 161 | Tailwind ne nomme que `transition-all`, `-colors`, `-transform`, `-shadow`. Animer exactement deux propriétés demande la liste explicite. Alternative : `transition-all` — plus large, donc plus coûteux sur un téléphone à 2 Go. Sans impact visuel : **décision de coût**. |
| 5 | `border-x-[5px]` `border-b-[7px]` `border-t-[7px]` `[clip-path:polygon(…)]` | 4 fichiers — triangle de delta (carte chiffre), pastille « impayé » | 17 | Deux façons de dessiner le même triangle. Alternative : l'icône Phosphor `ph-caret-up` en `text-mini`, ce qui supprime l'arbitraire mais change légèrement la forme. **Une seule décision pour les deux.** |
| 6 | `bg-[#f7f5f0]` … 20 hexadécimaux | `fondation/fondation-directions.html` | 20 | Les échantillons de **Trame**, la direction visuelle écartée. Ils sont en dur exprès : Trame n'a pas de tokens, et ne doit pas en avoir. Si la planche comparative ne part pas au développement, la question ne se pose plus. |
| 7 | `left-[123.4px]` `w-[87.5px]` | `V1-planning` (97) et `V1-planning-dense` (197) | 308 | **Ce ne sont pas des décisions mais des données.** Le ruban est élastique, son échelle se calcule par heure : 1,4 px pour une heure de nuit morte, 3 px pour une heure creuse, 9,5 px pour une heure occupée, 3,2 px pour les heures intérieures d'une nuit, × 0,13 pour les jours lointains, puis normalisé à 1040 px. En production le composant les calcule. **Rien à trancher.** |
| 8 | `w-[210mm]` `min-h-[297mm]` `px-[18mm]` `text-[2.85mm]` `w-[80mm]`… | `documents/` — les 3 fichiers | 78 | Des **millimètres physiques** : A4 pour la note et la facture, 80 mm pour le rouleau thermique. Aucune échelle de thème ne peut les porter — un ticket fait 80 mm sur du papier, pas 20 unités d'espacement. **Rien à trancher.** |
| 9 | `grid-cols-[220px_84px_minmax(0,1fr)]` et 18 autres gabarits | 14 fichiers | 96 | Les colonnes d'un tableau se règlent au contenu, pas à une échelle. `grid-cols-[minmax(0,1fr)_auto]` (23 emplois) est le seul assez répandu pour mériter un nom, si le développement en veut un. |
| 10 | `max-w-[62ch]` `max-w-[80ch]` `max-w-[86ch]` … | 22 fichiers | 105 | Mesures de texte, réglées au contenu. **Rien à trancher.** |
| 11 | `rounded-[26px]` `border-6` | `P2` et `M4` — cadre du téléphone (4 fichiers) | 52 | Le bezel de l'appareil : décor de maquette, pas un token de produit. **Disparaît au développement.** |

Deux points connexes, plus légers :

- **Opacités locales sur token** (`bg-alerte/10`, `border-bg/30`, `text-bg/60`,
  `border-info/30`, `bg-ocre/40`). Standard Tailwind, pas des valeurs arbitraires — mais si
  l'une se répète dans les écrans, elle mérite son token `-soft`.
- **Composant nº 15, barre de proportion.** Apparu après la série des 14. Il entre dans le
  canon avec ses états, ou il reste une composition locale de la carte chiffre. Voir
  `composants.md`.
- **Composant nº 16, champ de saisie — ajouté, pas en attente.** Aucun des 29 fichiers de `html/`
  ne contient d'`<input>`, de `<select>` ni de `<textarea>` : les onze écrans maquettés sont des
  écrans de lecture et de geste. La pièce a donc été **composée depuis les tokens**, qui la
  décrivaient déjà — `--color-line-2` dit littéralement « bordure de champ au repos », et
  `--radius-md` dit « champ ». Sa vignette est au styleguide, section `#c16`.

---

## Contenu de l'archive

```
kaya-design/
├── README.md            ce fichier
├── theme.css            LE bloc @theme complet — le seul fichier à copier
├── tokens.md            les mêmes valeurs en tableau lisible
├── mouvement.md         durées, courbes, les sept patrons
├── composants.md        les 16 composants : rôle, états, classes, règles
├── styleguide.html      les 16 composants dans tous leurs états, clair + sombre
├── fondation/           les cinq planches de fondation
├── html/                un fichier par écran ET par état — {code}-{nom}[-{etat}].html
├── documents/           tickets thermiques, note provisoire, facture fiscale
└── proto/               le sommaire + six prototypes animés, jouables
```

Chaque HTML est autonome : compilateur Tailwind du navigateur + le bloc `@theme` inliné,
**copie exacte de `theme.css`**. N'éditez pas ce bloc dans une maquette : il est régénéré
depuis `theme.css`.

**Les maquettes demandent le réseau à l'ouverture.** Trois ressources sont chargées en
ligne : le compilateur Tailwind (jsdelivr), Archivo + Chivo Mono (Google Fonts), les icônes
Phosphor (unpkg). Hors ligne, un fichier s'ouvre sans style du tout. **La revue se fait donc avec du réseau.**
Si un atelier sans connexion devient nécessaire, demandez une version autonome : chaque
écran embarquerait son CSS compilé et ses polices, et resterait inspectable — c'est
préférable à des captures, qui figent les états.

**Deux règles d'édition, si vous retouchez une maquette :** le `<style
type="text/tailwindcss">` reste dans `<head>`, et le `<script src="…/@tailwindcss/browser@4">`
reste **en dernière ligne du `<body>`**. Le compilateur ne compile que ce qu'il trouve dans
le document au moment où il s'exécute : placé dans le `<head>`, il ne voit que la balise
`<html>` et la page s'ouvre sans style. Le CSS explicite, quand il y en a, est regroupé en fin de fichier,
commenté, et limité à ce que Tailwind n'exprime pas — `@keyframes` et styles d'impression
thermique.

## État de la livraison

**L'archive est complète.** 44 fichiers HTML, 5 fichiers de documentation.

| Fichier | Contenu |
| --- | --- |
| `README.md` `theme.css` `tokens.md` `mouvement.md` `composants.md` | le socle |
| `styleguide.html` | les 14 composants + le nº 15 et le nº 16, tous états, clair + sombre |
| `fondation/` — 5 planches | directions visuelles (Banco vs Trame), système de mouvement, moments de plaisir, moments difficiles, illustrations |
| `html/R1-accueil*` — 4 fichiers | nominal, serveuse, propriétaire, maquis |
| `html/R4-passage*` — 5 fichiers | nominal, client connu, enregistré, hors ligne, tout est pris |
| `html/P2-saisie-commande*` — 3 fichiers | nominal mobile, hors ligne, poste du restaurant |
| `html/R7-note-depart*` — 3 fichiers | note en cours, envoi aux impôts, envoi refusé |
| `html/C4-cloture*` — 3 fichiers | possible, bloquée, réussie (M1) |
| `html/V1-planning*` — 2 fichiers | semaine calme, semaine dense |
| `html/M4-mes-etablissements*` — 2 fichiers | nominal, un établissement en alerte |
| `html/F2-registre-grave` · `html/S2-registre-grave` — 2 fichiers | document fiscal indéterminé · réconciliation d'une écriture orpheline |
| `html/Q1-page-client*` — 3 fichiers | nominal, panier, attente |
| `html/G2-offre-hebergement*` — 2 fichiers | hôtel, résidence |
| `documents/` — 3 fichiers | D1–D5 tickets thermiques 80 mm, D6 note provisoire, D7 facture fiscale A4 |
| `proto/` — 7 fichiers | sommaire + les six prototypes de mouvement |

Soit **29 fichiers d'écran-état** sur les 27 demandés : deux s'ajoutent à la liste initiale,
et il faut le dire.

- **`R4-passage-enregistre.html`** — l'état après le geste unique. Sans lui, le moment de
  plaisir M2 et l'échelle du comptoir n'existaient nulle part dans l'archive.
- **`C4-cloture.html`** — la clôture possible, avant l'appui. La liste ne demandait que
  « bloquée » et « réussie » ; sans l'état de départ, on ne voit pas ce que les quatre
  vérifications valident.

Dites-le si vous les voulez hors de l'export.

### Comment lire l'archive

`theme.css` et `tokens.md` sortent de `styleguide.html` ; `mouvement.md` sort de `proto/` ;
les écrans sont écrits sur ces tokens. C'est l'ordre dans lequel l'archive a été construite,
et celui dans lequel elle se lit.

**R1 sert de gabarit aux 28 écrans qui suivent** : même coquille (bandeau de repérage hors
produit, barre supérieure invariable, colonne de travail à gauche, rail à droite), même
ordre de classes, même `data-zone` sur le `<body>`. Regardez-le d'abord : si la coquille
vous convient, les suivants sont identiques.

**Trancher les décisions nº 1, 2 et 3 avant d'écrire le premier composant Vue** : elles
touchent 21, 9 et 27 fichiers. Après, c'est 250 corrections au lieu de trois.
