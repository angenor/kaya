# Kaya — mouvement

Le mouvement de Kaya répond à une seule question : **est-ce que mon travail est en
sécurité ?** Il ne décore pas, il informe. Une gérante qui encaisse à 23 h dans un bar
bruyant ne regarde pas l'écran ; elle le sent. Le mouvement est donc court, prévisible,
et toujours redondant avec un signal statique.

Trois principes, dans cet ordre :

1. **Le mouvement habille un état déjà appliqué.** L'état change d'abord, l'animation
   suit. Rien n'attend la fin d'une transition : aucune saisie n'est jamais bloquée.
2. **Aucun état n'est porté par une animation.** Si l'animation ne joue pas — préférence
   système, appareil lent, capture d'écran — l'information est encore là.
3. **400 ms de plafond absolu, 240 ms sur tout chemin fréquent.**

---

## 1. Durées

Reprises telles quelles de `theme.css`.

```css
--duree-instantane:  90ms;   /* retour tactile. Plancher : jamais réduit. */
--duree-rapide:     160ms;   /* changement d'état sur place, sortie de panneau */
--duree-standard:   240ms;   /* entrée de liste, changement d'écran, ouverture */
--duree-ample:      400ms;   /* plafond absolu : montant qui défile, réussite */
--decalage-liste:    28ms;   /* écart entre deux éléments d'une même entrée */
--decalage-plafond:      6;  /* au-delà de 6 éléments, plus de décalage */
```

En Tailwind : `duration-90`, `duration-160`, `duration-240`, `duration-400`.
En CSS calculé : `transition-duration: calc(var(--duree-standard) * var(--intensite))`.

Ces sept variables sont déclarées dans un `:root` ordinaire de `theme.css`, **hors du bloc
`@theme`** : Tailwind 4 élague toute variable de `@theme` qu'aucun utilitaire n'emploie, et
aucune d'elles n'en génère. Les quatre courbes, elles, restent dans `@theme` (marqué
`static`, pour la même raison) parce qu'elles produisent `ease-entree`, `ease-sortie`,
`ease-deplace`, `ease-elastique`.

Une liste de 40 lignes se pose donc en 168 ms de décalage maximum (6 × 28 ms), pas en
1,1 s. C'est ce plafond qui rend le produit utilisable sur un registre de fin de mois.

## 2. Courbes

```css
--ease-entree:    cubic-bezier(0, 0, .2, 1);      /* ease-entree */
--ease-sortie:    cubic-bezier(.4, 0, 1, 1);      /* ease-sortie */
--ease-deplace:   cubic-bezier(.4, 0, .2, 1);     /* ease-deplace */
--ease-elastique: cubic-bezier(.34, 1.56, .64, 1);/* ease-elastique */
```

| Courbe | Ce qu'elle décrit | Où |
| --- | --- | --- |
| **Entrée** | Un élément qui apparaît décélère : il arrive vite puis se pose. | Listes, panneaux, bandeaux, modales. |
| **Sortie** | Un élément qui part accélère et disparaît. Toujours plus courte que son entrée — on ne regarde pas partir. | Fermeture de panneau, retrait de bandeau. |
| **Déplacement** | Un élément qui reste à l'écran et change de place ou de taille. Symétrique. | Réordonnancement, ouverture d'accordéon, bascule de segment. |
| **Élastique** | Dépasse de 4 % puis revient. **Réservée au plaisir** : montant qui monte, réussite, récompense. **Interdite en zone de vitesse.** | Clôture réussie, KPI d'accueil. |

## 3. Les sept patrons

Chaque patron a un code. Les prototypes les jouent ; `proto/proto-0-sommaire.html` les
réunit.

### P1 · Entrée de liste
`opacity 0→1` + `translateY(10px)→0`, `--duree-standard`, courbe **entrée**,
décalage `--decalage-liste` plafonné à `--decalage-plafond` éléments.
Une liste qui se recharge ne rejoue pas l'entrée : seules les lignes nouvelles entrent.

### P2 · Transition d'écran
Sortant : `translateX(-28px)` + `opacity 0`, courbe **sortie**.
Entrant : `translateX(100%)→0`, courbe **entrée**, `--duree-standard`.
Le retour inverse les directions. Jamais de fondu croisé : on doit savoir d'où on vient.

### P3 · Panneau latéral
Entrée `translateX(100%)→0` en `--duree-standard` (**entrée**) avec le voile
(`--color-voile`) qui monte en opacité. Sortie en `--duree-rapide` (**sortie**).
Le voile disparaît toujours avant le panneau, jamais après.

### P4 · Montant qui change
Le nombre **défile** (interpolation cubique) sur `--duree-ample`, avec une poussée
d'échelle de 4 % en courbe **élastique** et, en zone de charme seulement, un delta
(`+ 2 500 F`) qui monte et s'efface. En zone de vitesse : 140 ms, aucun dépassement,
pas de delta — la valeur est déjà juste quand l'œil arrive.

### P5 · Retour tactile du bouton
`translateY(3px)` + l'ombre pleine qui tombe à 0, `--duree-instantane`, courbe **entrée**.
**C'est le seul mouvement qui n'est jamais réduit** : il ne raconte rien, il confirme
que le doigt a été vu. Sous « réduire les animations », il reste à 90 ms.

### P6 · Changement d'état
L'ancienne pastille sort vers le haut, la nouvelle entre par le bas, `--duree-rapide`.
La **forme** change avec la couleur — jamais la couleur seule (voir composant 04).

### P7 · État de chargement
Squelette à la forme exacte du contenu à venir. Le scintillement est une **bande
translatée** (`transform` seul) sur 1,5 s, jamais un dégradé animé. Pose du contenu en
`--duree-standard`. La roue (`animate-roue`) est réservée à une attente réseau
**indéterminée** — jamais pour un chargement dont on connaît la forme.

## 4. Les deux zones

Une seule variable sépare le plaisir de la vitesse.

```css
:root { --intensite: 1; }               /* zone de charme */
[data-zone="vitesse"] {
  --intensite: .45;                     /* durées × 0,45, plafonnées à 160 ms */
  --decalage-liste: 0ms;                /* tout arrive ensemble */
  --ease-elastique: var(--ease-deplace);/* aucun dépassement */
}
```

| | Charme (×1) | Vitesse (×0,45) |
| --- | --- | --- |
| Où | Accueil, tableau de bord, clôture réussie, page client | Comptoir, bar, saisie de commande, encaissement, caisse |
| Durée | Nominale | × 0,45, plafond 160 ms |
| Décalage | 28 ms | 0 — tout arrive ensemble |
| Courbe | Élastique autorisée | Déplacement seulement |
| Nombre de signaux simultanés | 3 | 1 — on garde le plus lisible |

`data-zone="vitesse"` se pose sur le conteneur de l'écran, pas sur un composant. Un même
composant se comporte différemment selon l'écran qui l'accueille : c'est voulu.

## 5. Accessibilité et performance

- **« Réduire les animations » respecté.** `--intensite: 0` et `--decalage-liste: 0`, plus
  une coupe globale des `animation-duration` / `transition-duration`. Ce qui bougeait
  apparaît. Seul P5 garde ses 90 ms.
- **Aucune saisie bloquée.** Le mouvement habille un état déjà appliqué ; un champ reste
  focalisable pendant toute transition.
- **`transform` et `opacity` uniquement.** Le scintillement est une bande translatée, les
  ombres du bouton sont pré-rendues. `will-change` posé sur les seuls éléments animés,
  jamais sur un conteneur de liste.
- **Budget 60 i/s sur 2 Go.** Six éléments animés simultanément au maximum. Au-delà, on
  anime le conteneur, pas ses enfants.

## 6. Ce qui ne bouge jamais

Le témoin de synchronisation ne s'anime pas pour attirer l'œil : son pouls
(`animate-pulse-reseau`, 2,4 s) est lent et constant, et son passage hors ligne est
**instantané**. Un état grave n'a pas de transition — il est déjà là.
