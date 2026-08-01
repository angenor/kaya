# Polices embarquées — ce qui a été modifié, et pourquoi

**Ce fichier existe pour déclarer une modification.** Les six `woff2` de ce répertoire ne sont pas
les fichiers amont tels quels : ils sont produits par deux scripts du dépôt, et deux des quatre
polices de texte portent une table `cmap` réécrite. Une police modifiée reste sous sa licence
d'origine ; la modification, elle, se déclare.

Les textes de licence à côté (`*-LICENCE.txt`) sont des **copies exactes** de l'amont, jamais
retouchées : c'est ce qui permet de les comparer octet à octet à leur source. La porte **P-21b**,
contrôle 5, vérifie que chaque `woff2` de ce répertoire a sa licence et son avis de copyright.

---

## Ce qui est embarqué

| Fichier | Origine | Licence | Modifié ? |
|---|---|---|---|
| `archivo-latin-kaya.woff2` | `@fontsource-variable/archivo` 5.3.0 | OFL 1.1 | **Oui — `cmap`** |
| `archivo-latin-ext-kaya.woff2` | `@fontsource-variable/archivo` 5.3.0 | OFL 1.1 | Non — ré-encodé à l'identique |
| `chivo-mono-latin-kaya.woff2` | `@fontsource-variable/chivo-mono` 5.3.0 | OFL 1.1 | **Oui — `cmap`** |
| `chivo-mono-latin-ext-kaya.woff2` | `@fontsource-variable/chivo-mono` 5.3.0 | OFL 1.1 | Non — ré-encodé à l'identique |
| `phosphor-kaya.woff2` | `@phosphor-icons/web` 2.1.2 | MIT | **Oui — sous-réglé** |
| `phosphor-fill-kaya.woff2` | `@phosphor-icons/web` 2.1.2 | MIT | **Oui — sous-réglé** |

Régénération :

```sh
pnpm --filter @kaya/app polices:generer     # les quatre woff2 de texte + polices.css
pnpm --filter @kaya/app icones:generer      # les deux woff2 d'icônes + icones.css
```

---

## La modification, en une phrase

**Une association de plus dans la table `cmap` : `U+202F → dessin de `U+2009`.** Aucun contour n'est
créé, aucun glyphe n'est dessiné, aucun glyphe n'est retiré des deux fichiers `latin`. Le fichier
reste la police d'origine, avec un caractère de plus atteignable.

`docs/design/tokens.md` §2 impose l'espace fine insécable **U+202F** entre les groupes de milliers
et avant le symbole de devise (`12 500 F`), et en fait la condition de l'alignement des colonnes de
montants en Chivo Mono tabulaire. Or **U+202F n'est dessiné ni par Archivo ni par Chivo Mono** —
constat fait par lecture des `woff2` de Fontsource *et* des `ttf` amont de Google Fonts, alors que
la `unicode-range` déclarée annonce `U+2000-206F`. Sans cette association, chaque montant du produit
ferait tomber son séparateur sur une police système de repli, de chasse inconnue.

Le glyphe réutilisé est celui de **U+2009 THIN SPACE**, présent dans les deux familles. Le choix est
mesuré, pas supposé — chasses relevées dans les `ttf` amont, unités de 1000 :

| Police | `U+0020` | `U+00A0` | `U+2009` | Ce que ça donne pour `U+202F` |
|---|---|---|---|---|
| Archivo (proportionnelle) | 209 | 209 | **193** | une fine, plus étroite que l'espace mot |
| Chivo Mono (monospace) | 600 | 600 | **600** | la cellule pleine — l'alignement tabulaire tient |

L'insécabilité ne vient pas de la police mais du **caractère** : U+202F est de catégorie Unicode
`Zs` avec la propriété non-sécable. Elle est préservée puisque le texte reste U+202F — on ne
substitue pas le caractère, on lui donne un dessin.

Les deux fichiers `latin-ext` ne reçoivent pas l'association : leur `unicode-range` ne couvre pas
U+202F, et le navigateur ne le leur demandera donc jamais. Ils sont malgré tout ré-encodés par le
même chemin, d'où leur présence dans le tableau.

Les deux polices d'icônes, elles, sont **sous-réglées** : 77 glyphes retenus sur ~1530, 9,4 ko au
lieu de 279. Le sous-réglage retire des glyphes, il n'en modifie aucun.

---

## Sur les noms de famille

`polices.css` déclare `font-family: "Archivo"` et `font-family: "Chivo Mono"` — **les noms
d'origine**, et non des noms dérivés. C'est licite ici, et le point mérite d'être écrit parce qu'il
ne l'est pas toujours :

> **Ni Archivo ni Chivo Mono ne déclarent de Reserved Font Name.** Leur ligne de copyright est nue —
> « Copyright 2020 The Archivo Project Authors » — sans le « with Reserved Font Name » que la
> clause 3 de l'OFL 1.1 exige pour réserver un nom. Vérifié dans les deux `LICENSE` amont et dans
> leur `metadata.json`.

La clause 3 ne s'applique donc pas, et une version modifiée peut garder le nom de famille. Le jour
où l'on embarquerait une police **avec** un nom réservé, cette conclusion tomberait : il faudrait
renommer la famille, donc modifier `theme.css` et les jetons `--font-*`.

---

## Ce que l'OFL exige, et où c'est satisfait

| Obligation (OFL 1.1) | Où |
|---|---|
| Clause 2 — l'avis de copyright et la licence accompagnent toute copie | `archivo-LICENCE.txt`, `chivo-mono-LICENCE.txt`, ici même, et la section « Mentions » du back-office |
| Clause 3 — pas de nom réservé employé | Sans objet : aucune des deux polices n'en déclare (ci-dessus) |
| Clause 4 — pas d'usage du nom des auteurs pour promouvoir | Aucun |
| Clause 5 — pas de re-licenciement | Les `woff2` restent sous OFL 1.1 |
| Clause 1 — la police n'est pas vendue seule | Elle est embarquée dans un logiciel, jamais distribuée seule |

Inventaire complet, avec les autres obligations : `docs/conformite/licences-tierces.md`.
