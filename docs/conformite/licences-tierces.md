# Licences tierces — ce que Kaya redistribue

*Établi le 2026-07-31, au moment où les polices ont été embarquées. Tenu à jour à chaque œuvre
tierce ajoutée au binaire.*

**Ce document couvre ce qui est redistribué dans le produit livré**, pas les dépendances du projet.
La distinction commande l'obligation : une bibliothèque de développement ne voyage pas chez le
client et n'appelle aucune attribution ; un fichier de police embarqué dans un binaire Tauri, si.

L'inventaire des versions vit dans `docs/versions-gelees.md` et dans les lockfiles. Celui-ci ne
répond qu'à une question : **qu'est-ce qui part chez le client, et à quelles conditions.**

---

## Pourquoi l'obligation existe ici et pas ailleurs

Kaya est distribué comme **binaire Tauri, vendu par abonnement**. C'est une redistribution
commerciale, ce qui referme les deux échappatoires qu'on invoque d'habitude :

- ce n'est pas un usage interne — le logiciel est livré à des tiers ;
- ce n'est pas gratuit — l'OFL autorise la vente **avec** un logiciel, à condition que la police ne
  soit pas vendue seule et que l'avis accompagne la copie.

Trois obligations en découlent, et elles sont satisfaites en trois endroits distincts :

| Obligation | Où elle est satisfaite | Vérifiée par |
|---|---|---|
| L'avis de copyright et la licence accompagnent le fichier | `app/assets/fonts/*-LICENCE.txt` | **P-21b, contrôle 5** |
| Ils entrent dans le paquet distribué | `app/core/licences/` les importe en clair (`?raw`) | La construction |
| Ils sont atteignables par un humain | Section « Mentions » de l'écran `G1` | Revue |

---

## Les trois œuvres embarquées

### Archivo — OFL 1.1

- **Auteurs** : Omnibus Type — `https://github.com/Omnibus-Type/Archivo`
- **Avis** : `Copyright 2020 The Archivo Project Authors`
- **Source** : `@fontsource-variable/archivo` 5.3.0 (gel §3.2), lui-même issu de `google/fonts`
- **Fichiers** : `archivo-latin-kaya.woff2`, `archivo-latin-ext-kaya.woff2`
- **Usage** : tout le texte de l'application (`--font-titre`, `--font-texte`)
- **Modifiée** : oui — table `cmap`, association `U+202F → dessin de U+2009`
- **Nom réservé** : **aucun**, vérifié dans le `LICENSE` amont et le `metadata.json`

### Chivo Mono — OFL 1.1

- **Auteurs** : Omnibus Type — `https://github.com/Omnibus-Type/Chivo`
- **Avis** : `Copyright 2018 The Chivo Project Authors`
- **Source** : `@fontsource-variable/chivo-mono` 5.3.0 (gel §3.2)
- **Fichiers** : `chivo-mono-latin-kaya.woff2`, `chivo-mono-latin-ext-kaya.woff2`
- **Usage** : montants, quantités, heures, numéros de chambre (`--font-mono`)
- **Modifiée** : oui — même association `cmap`
- **Nom réservé** : **aucun**

### Phosphor Icons — MIT

- **Auteurs** : Phosphor Icons — `https://phosphoricons.com`
- **Avis** : `Copyright (c) 2020-2021 Phosphor Icons`
- **Source** : `@phosphor-icons/web` 2.1.2 (gel §3.2)
- **Fichiers** : `phosphor-kaya.woff2`, `phosphor-fill-kaya.woff2`
- **Usage** : les icônes de l'interface
- **Modifiée** : oui — sous-réglée à 77 glyphes sur ~1530

---

## Le point qui aurait pu coûter cher, et pourquoi il ne coûte rien

**Modifier une police dont le nom est réservé oblige à la renommer.** C'est la clause 3 de l'OFL
1.1 : une version modifiée ne peut pas porter un *Reserved Font Name*. Kaya modifie la table `cmap`
d'Archivo et de Chivo Mono **et garde les noms de famille** dans `polices.css` — ce qui serait une
violation si ces noms étaient réservés.

Ils ne le sont pas. Les deux lignes de copyright sont nues :

```
Copyright 2020 The Archivo Project Authors (https://github.com/Omnibus-Type/Archivo)
Copyright 2018 The Chivo Project Authors (https://github.com/Omnibus-Type/Chivo)
```

Aucune ne porte le « with Reserved Font Name "…" » que la clause 3 exige pour réserver un nom. La
clause est donc sans objet, et le nom de famille peut rester.

**Ce qui changerait avec une police à nom réservé** — et c'est la seule chose à retenir de ce
paragraphe : il faudrait renommer la famille dans `polices.css`, donc dans les jetons `--font-titre`,
`--font-texte` et `--font-mono` de `theme.css`, donc dans `docs/design/theme.css` dont
`app/assets/css/theme.css` est la copie exacte. Une contrainte de licence remonterait jusqu'à la
source de vérité du design.

---

## Ce qui reste hors de ce document

- **Les dépendances de développement et de compilation** — Rust, Nuxt, Tailwind, Vitest, ESLint.
  Elles ne sont pas redistribuées. Leurs licences vivent dans les lockfiles ; le jour où une
  bibliothèque est **compilée dans le binaire** et soumise à attribution, elle entre ici.
- **Le contenu produit par les clients** — leur logo, leur couleur d'identité visuelle. Ce sont
  leurs données, pas des œuvres tierces du produit.
- **Les données personnelles et le registre ARTCI** — `README.md` de ce répertoire, story TRX-06.

---

## Voir aussi

- `app/assets/fonts/MODIFICATIONS.md` — le détail technique de ce qui a été modifié, et pourquoi
- `scripts/ci/ressources-embarquees.sh` — la porte P-21b, contrôle 5
- `docs/versions-gelees.md` §3.2 — les versions des paquets d'origine
- `.specify/memory/constitution.md` — principe XII (référence visuelle), principe IX (sécurité)
