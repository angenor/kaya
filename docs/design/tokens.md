# Kaya — tokens

Direction retenue : **Banco**. Les mêmes noms portent les valeurs claires et sombres ;
seule la valeur change, sous `.dark`. Le balisage n'a donc pas à connaître le mode.

Source de vérité : `theme.css`. Ce fichier est la même chose, lisible.
Vue rendue de tous ces tokens en situation : `styleguide.html`.

---

## 1. Couleurs

| Token | Utilitaires | Clair | Sombre | Usage |
| --- | --- | --- | --- | --- |
| `--color-bg` | `bg-bg` | `#faf4e9` | `#17120f` | Fond de page. Jamais une carte. |
| `--color-surf` | `bg-surf` | `#fffdf8` | `#241c16` | Carte, panneau, feuille posée sur le fond. |
| `--color-tile` | `bg-tile` | `#f4ece0` | `#2e241c` | Tuile, en-tête de tableau, piste de segment, squelette. |
| `--color-line` | `border-line` | `#e2d7c6` | `#3c2f26` | Filet de séparation, bordure de carte. |
| `--color-line-2` | `border-line-2` | `#c1ac91` | `#5f4a39` | Filet appuyé, bordure de champ au repos, ligne de total. |
| `--color-ink` | `text-ink` | `#1e1a16` | `#f4e9da` | Texte, chiffres. |
| `--color-ink-2` | `text-ink-2` | `#4b4239` | `#d5c6b4` | Texte secondaire, libellé de bouton discret. |
| `--color-ink-3` | `text-ink-3` | `#6b5f52` | `#ab9a8b` | Étiquette en capitales, texte atténué, méta. |
| `--color-prim` | `bg-prim` `text-prim` | `#21458c` | `#9dbcf5` | **L'indigo : l'action, et rien d'autre.** Tout ce qui est indigo se touche. |
| `--color-prim-ink` | `text-prim-ink` | `#ffffff` | `#10203f` | Texte posé sur `prim`. |
| `--color-prim-dk` | — | `#16306a` | `#6f92cc` | Ombre pleine du bouton, survol du bouton principal. |
| `--color-prim-soft` | `bg-prim-soft` | `#eaf0fb` | `#1e2839` | Survol de ligne, fond de bouton secondaire survolé, ligne sélectionnée. |
| `--color-succes` | `bg-succes` | `#1e6b4f` | `#63cf9d` | Marque et filet de succès. |
| `--color-succes-soft` | `bg-succes-soft` | `#dff0e6` | `#18342a` | Fond de pastille et de bandeau. |
| `--color-succes-fort` | `text-succes-fort` | `#14523c` | `#a6e6c9` | Texte posé sur `-soft`. |
| `--color-alerte` | `bg-alerte` | `#9a5a06` | `#e9a848` | Ce qui attend : partiel, en attente, à nettoyer. |
| `--color-alerte-soft` | `bg-alerte-soft` | `#fbe9cf` | `#3a2a12` | Fond. |
| `--color-alerte-fort` | `text-alerte-fort` | `#6f4104` | `#f0d9a8` | Texte sur `-soft`. |
| `--color-danger` | `bg-danger` | `#a32723` | `#f4837a` | Ce qui est cassé ou irréversible. |
| `--color-danger-soft` | `bg-danger-soft` | `#f8e4e2` | `#3d1c1a` | Fond. |
| `--color-danger-fort` | `text-danger-fort` | `#7d1c19` | `#f8b3ac` | Texte sur `-soft`, ombre du bouton danger. |
| `--color-info` | `bg-info` | `#245e80` | `#78bede` | Information neutre, envoi en cours. |
| `--color-info-soft` | `bg-info-soft` | `#e2eef4` | `#12293a` | Fond. |
| `--color-info-fort` | `text-info-fort` | `#123f57` | `#b5dcee` | Texte sur `-soft`. |
| `--color-occupe` | `bg-occupe` | `#5b3fa8` | `#b79cf0` | Occupation : chambre prise, table servie. |
| `--color-occupe-soft` | `bg-occupe-soft` | `#e6dcf5` | `#3a2f4d` | Fond. |
| `--color-occupe-fort` | `text-occupe-fort` | `#3d2a63` | `#d6c4f2` | Texte sur `-soft`. |
| `--color-prim-fort` | `text-prim-fort` | `#16306a` | `#dce7fb` | Texte sur `prim-soft` — la barre « nuit » du planning. |
| `--color-ocre` | `bg-ocre` `text-ocre` | `#a86f38` | `#d9b382` | **La terre.** Décor, illustration, initiale d'établissement, barre « passage » du planning. Jamais une action. |
| `--color-ocre-fort` | `text-ocre-fort` | `#7a4e22` | `#e9cfa8` | Texte sur `ocre-soft`. |
| `--color-ocre-ink` | `text-ocre-ink` | `#fffdf8` | `#2a1a0c` | Texte posé sur `ocre` plein. |
| `--color-hachure` | — | `rgb(107 95 82 / .34)` | `rgb(171 154 139 / .32)` | Hachure à 135° : la remise en état, dans le planning. |
| `--color-grille` | — | `rgb(107 95 82 / .16)` | `rgb(171 154 139 / .14)` | Graduation des heures du planning. |
| `--color-resume` | `bg-resume` | `rgb(107 95 82 / .10)` | `rgb(0 0 0 / .32)` | Jour replié du ruban élastique. |
| `--color-ocre-soft` | `bg-ocre-soft` | `#f0e2cd` | `#33261a` | Fond d'initiale, motif d'état vide. |
| `--color-voile` | `bg-voile` | `rgb(30 26 22 / .38)` | `rgb(10 11 16 / .55)` | Voile sous un panneau ou une modale. |
| `--color-brillance` | `via-brillance` | `rgb(255 255 255 / .85)` | `rgb(233 233 237 / .09)` | Bande de scintillement du squelette. |

**Trois règles de couleur.**
1. L'indigo est un signal, pas une décoration : ce qui est indigo se touche.
2. Un état n'est jamais porté par la couleur seule — il porte aussi une forme (voir composant 04).
3. Le texte sur un fond `-soft` prend toujours la variante `-fort`, jamais la couleur de base.

## 2. Typographie

Archivo pour tout le texte. **Chivo Mono pour tout ce qui se compte** : montant, quantité,
numéro de chambre, heure, référence. Jamais l'inverse.

| Token | Utilitaire | Corps | Interligne | Usage |
| --- | --- | --- | --- | --- |
| `--font-titre` | `font-titre` | Archivo 500/600 | — | Titres, libellés de bouton, noms. |
| `--font-texte` | `font-texte` | Archivo 400 | — | Corps de texte. Police du `body`. |
| `--font-mono` | `font-mono` | Chivo Mono 400/700 | — | Montants, chiffres, heures. Tabulaire par défaut. |
| `--text-etiquette` | `text-etiquette` | 11 px | 1 | Étiquette en capitales, `letter-spacing .1em`. |
| `--text-mini` | `text-mini` | 12,5 px | 1,45 | Méta, bouton discret, pastille. |
| `--text-corps` | `text-corps` | 13,5 px | 1,55 | Corps de texte. Taille du `body`. |
| `--text-action` | `text-action` | 14,5 px | 1 | Libellé de bouton. **Ne descend jamais plus bas.** |
| `--text-lead` | `text-lead` | 15 px | 1,6 | Chapeau, pastille de durée tactile. |
| `--text-titre-s` | `text-titre-s` | 17 px | 1,2 | Titre de carte, icône en ligne. |
| `--text-titre-m` | `text-titre-m` | 20 px | 1,2 | Titre d'écran. |
| `--text-chiffre` | `text-chiffre` | 24 px | 1,15 | Montant en ligne, total. |
| `--text-chiffre-l` | `text-chiffre-l` | 30 px | 1,15 | Chiffre de carte. |
| `--text-montant` | `text-montant` | 40 px | 1,05 | Le montant que le propriétaire lit sur son téléphone (M4). |
| `--text-montant-long` | `text-montant-long` | 32 px | 1,05 | Le même au-delà de 11 caractères — sinon il déborde d'un 372 px. |
| `--text-titre-l` | `text-titre-l` | 34 px | 1,1 | Grand chiffre de clôture, icône de tuile. |
| `--text-recette` | `text-recette` | 54 px | 1,05 | La recette du jour sur l'écran de clôture (C4) — un chiffre qu'on lit assis, pas qu'on crie. |
| `--text-geste` | `text-geste` | 46 px | 1 | Le chiffre d'un bouton de décision tactile (R4 : les quatre durées). |
| `--text-total` | `text-total` | 44 px | 1 | Le total d'un document à lignes (R7 note, D6, D7). |
| `--text-annonce` | `text-annonce` | 68 px | 1 | Le numéro de chambre une fois donné (R4). |
| `--text-annonce-l` | `text-annonce-l` | 88 px | 1 | L'heure de fin — le nombre que le comptoir dit à voix haute (R4). |
| `--text-affiche` | `text-affiche` | 40 px | 1,05 | Titre de page de référence. Hors produit. |

**Règle des montants, sans exception.** Un montant s'écrit avec une **espace fine
insécable U+202F** entre les groupes de milliers *et* avant le F : `12 500 F`, jamais
`12 500 F`. C'est ce qui empêche un montant de se couper en fin de ligne et ce qui aligne
les colonnes en Chivo Mono tabulaire. Côté code, une seule fonction la porte :

```js
const NB = '\u202F';
const money = n => String(n).replace(/\B(?=(\d{3})+(?!\d))/g, NB) + NB + 'F';
```

Les heures gardent une espace ordinaire (`17 h 30`) : elles ne se coupent pas, la lettre
`h` tient les deux nombres ensemble. Tout élément qui porte un montant reçoit en plus
`whitespace-nowrap`.

Les quatre derniers corps forment **l'échelle du comptoir** : ils n'existent que parce qu'un
nombre doit se lire à bout de bras, debout, en tenant une clé. Hors de R4, R7, C4 et des documents imprimés, on ne s'en sert pas.

Les demi-valeurs sont voulues : c'est le réglage qui tient le 13 px lisible sur un
1366 × 768 délavé par le soleil, sans casser l'échelle sur un téléphone.

### 2.1 Décisions consignées — deux questions qui ne se reposent pas

Ces deux observations reviennent à chaque relecture de l'échelle typographique. Elles sont
tranchées ; elles sont écrites ici pour qu'on cesse de les rouvrir.

**`--text-action` (14,5 px) et `--text-lead` (15 px) sont indistinguables à l'œil — et ils restent
séparés.** Un demi-pixel d'écart ne se voit pas ; ce n'est pas ce qui les justifie. Ce sont **deux
rôles distincts** — le libellé de bouton et le chapeau — qui portent aujourd'hui des valeurs
voisines. C'est précisément ce qui permettra de les faire diverger demain sans rouvrir le balisage.
Les fusionner échangerait une ligne de table contre la reprise de **chaque bouton du produit** le
jour où l'un des deux doit bouger : on paierait la simplification au moment le plus coûteux.
Un token n'est pas une valeur, c'est un point d'articulation.

**`--text-etiquette` (11 px, capitales, `letter-spacing .1em`) est à la limite basse de
lisibilité — et rien n'y est changé.** Le corps est petit, les capitales et l'interlettrage
compensent en partie, et l'étiquette ne porte jamais d'information critique : elle nomme la section
que le contenu dessous rend de toute façon évidente. Mais la question ne se tranche pas ici :
elle se juge **à Abengourou, à bout de bras, sur un écran délavé par le soleil**, pas sur un poste
de développement calibré. Un ajustement décidé au bureau serait une préférence, pas une mesure.

**Elle est donc portée à l'ordre du jour de la journée d'observation terrain**, avec le 13 px du
corps, dont elle est solidaire. Les retours iront dans `docs/design/notes-terrain.md` — fichier
qui **n'existe pas encore** et se créera au retour de cette journée, quand il aura quelque chose à
consigner.

## 3. Espacement et hauteurs

Base 4 px (`--spacing: 4px`). L'échelle fractionnaire de Tailwind 4 couvre toute la
maquette : `p-3.5` = 14 px, `p-4.5` = 18 px, `p-5.5` = 22 px.

| Hauteur | Utilitaire | Valeur | Règle |
| --- | --- | --- | --- |
| Touche | `h-11` | 44 px | **Plancher tactile. Jamais moins**, y compris pour une icône seule. |
| Comptoir | `h-12` | 48 px | Cible de comptoir : une main, vite, debout. Bouton pleine largeur, pastille de durée. |
| Ligne de liste | `h-14` | 56 px | Ligne de registre, ligne de total. |
| Bouton discret | `h-9` | 36 px | Hors chemin critique uniquement. |
| Segment | `h-8` | 32 px | Dans une piste `h-10`. |
| Pastille | `h-7` | 28 px | Non cliquable. Si elle devient cliquable, elle passe à `h-11`. |
| Tuile d'action | `min-h-28` | 112 px | Grille d'accueil. |

Deux variables nommées, pour le code qui a besoin de la valeur : `--touche-min: 44px`,
`--touche-comptoir: 48px`. Elles sont déclarées dans le `:root` explicite de `theme.css`,
hors du bloc `@theme` — Tailwind 4 élague toute variable de `@theme` qu'aucun utilitaire
n'emploie, et celles-ci n'en génèrent aucun.

## 4. Rayons

| Token | Utilitaire | Valeur | Usage |
| --- | --- | --- | --- |
| `--radius-xs` | `rounded-xs` | 4 px | Marque de sélection carrée, puce. |
| `--radius-sm` | `rounded-sm` | 6 px | Barre de squelette, micro-tuile. |
| `--radius-md` | `rounded-md` | 8 px | Champ, bouton discret, segment. |
| `--radius-lg` | `rounded-lg` | 10 px | Bouton, tuile d'action, bandeau. |
| `--radius-xl` | `rounded-xl` | 12 px | Carte. |
| `--radius-2xl` | `rounded-2xl` | 14 px | Panneau, feuille, menu ouvert. |
| `--radius-pleine` | `rounded-pleine` | 999 px | Pastille, barre de proportion, point de témoin. |

Le contrefort : un bloc dont un seul côté porte 4 px de couleur et dont le rayon tombe
de ce côté — `border-l-4 border-l-prim rounded-r-xl`. C'est le motif Banco, employé pour
le bandeau d'alerte, la ligne sélectionnée et le chiffre de clôture.

## 5. Ombres

Basses et rares. Sur fond sombre, l'élévation est un bord plus une obscurité ambiante.

| Token | Utilitaire | Clair | Sombre |
| --- | --- | --- | --- |
| `--shadow-basse` | `shadow-basse` | `0 1px 2px rgb(30 26 22 / .07)` | `0 1px 2px rgb(0 0 0 / .35)` |
| `--shadow-carte` | `shadow-carte` | idem | idem |
| `--shadow-panneau` | `shadow-panneau` | `0 8px 28px rgb(30 26 22 / .16)` | `0 10px 30px rgb(0 0 0 / .5)` |
| `--shadow-bouton` | `shadow-bouton` | `0 2px 0 var(--color-prim-dk)` | idem (le token suit le mode) |
| `--shadow-bouton-appui` | `shadow-bouton-appui` | `0 0 0 var(--color-prim-dk)` | idem |
| `--shadow-bouton-danger` | `shadow-bouton-danger` | `0 2px 0 var(--color-danger-fort)` | idem |

## 6. Mouvement

Détail, patrons et justifications : `mouvement.md`.

| Token | Valeur | Utilitaire |
| --- | --- | --- |
| `--duree-instantane` | 90 ms | `duration-90` |
| `--duree-rapide` | 160 ms | `duration-160` |
| `--duree-standard` | 240 ms | `duration-240` |
| `--duree-ample` | 400 ms | `duration-400` |
| `--decalage-liste` | 28 ms | — (calculé en JS ou en `nth-child`) |
| `--decalage-plafond` | 6 | — |
| `--intensite` | 1 · 0,45 en zone de vitesse · 0 si « réduire les animations » | — |
| `--ease-entree` | `cubic-bezier(0, 0, .2, 1)` | `ease-entree` |
| `--ease-sortie` | `cubic-bezier(.4, 0, 1, 1)` | `ease-sortie` |
| `--ease-deplace` | `cubic-bezier(.4, 0, .2, 1)` | `ease-deplace` |
| `--ease-elastique` | `cubic-bezier(.34, 1.56, .64, 1)` | `ease-elastique` |
| `--animate-pulse-reseau` | `kayaPulse 2.4s ease-out infinite` | `animate-pulse-reseau` |
| `--animate-souffle` | `kayaBreath 1.6s ease-in-out infinite` | `animate-souffle` |
| `--animate-scintillement` | `kayaShimmer 1.5s ease-in-out infinite` | `animate-scintillement` |
| `--animate-roue` | `kayaSpin 900ms linear infinite` | `animate-roue` |

## 7. Ce que le thème ne porte pas

Trois choses, volontairement :

- **Les largeurs de gabarit** (`max-w-[460px]`, `max-w-[340px]`…). Ce sont des décisions de
  mise en page, pas des tokens. Elles sont listées dans `README.md`.
- **Les mesures de texte** (`max-w-[80ch]`). Idem : une mesure se règle au contenu.
- **Une épaisseur de bordure de 1,5 px** (bouton secondaire, pastille de durée). Tailwind 4
  n'a pas d'espace de nom pour les épaisseurs de bordure ; c'est la décision n° 1 du README.
