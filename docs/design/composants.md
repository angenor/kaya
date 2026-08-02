# Kaya — les composants canoniques

Tout écran de Kaya est composé de ces pièces — **seize au 2026-08-02**. Le décompte est celui
des sections numérotées de ce fichier, jamais un nombre écrit ailleurs : il a déjà été faux deux
fois.

Les **quatorze premiers** ont été dessinés sur onze écrans de lecture et de geste. Le **n° 15**
(barre de proportion) est apparu après la série et attend sa validation — voir `README.md`,
décisions. Le **n° 16** (champ de saisie) a été **composé depuis les tokens**, sans maquette :
aucun des 29 fichiers de `docs/design/html/` ne contient de champ. Il est né du besoin d'un
patron d'écriture, et c'est lui qui rend possible les **écrans composés** au sens de
`Kaya_Design.md` §2.

Le **n° 16 (champ de saisie)** est arrivé après elles aussi, et pour une raison différente : les
quatorze premiers ont été dessinés sur onze écrans de **lecture et de geste**, où l'on ne saisit
rien. Le jour où le produit a eu sa première écriture depuis un écran, la pièce manquait. Elle est
**composée depuis les jetons**, qui la décrivaient déjà — `--color-line-2` dit littéralement
« bordure de champ au repos » — et non inventée.

Rendu de tous les états, clair et sombre : `styleguide.html`.
Valeurs : `tokens.md`. Durées et courbes : `mouvement.md`.

Deux règles transversales :

- **44 px de zone de touche minimum**, y compris pour une icône seule. Un composant qui
  passe de décoratif à cliquable change de hauteur.
- **Un état n'est jamais porté par la couleur seule.** Il porte aussi une forme.

---

## 01 · Bouton principal

**Rôle.** L'action qui fait avancer la journée. Un seul par écran.

**États.** repos · survol · appui · focus clavier · en cours · désactivé · pleine largeur ·
variante danger.

```html
<button class="h-11 min-w-42 px-5 rounded-lg bg-prim text-prim-ink font-titre text-action
  font-semibold shadow-bouton cursor-pointer transition-[transform,box-shadow,background-color]
  duration-90 ease-entree hover:bg-prim-dk active:translate-y-0.5 active:shadow-bouton-appui">
```

**Règles.** L'ombre pleine de 2 px tombe à l'appui — seul relief du système, et seul
mouvement jamais réduit (P5). En cours : le libellé change (« Envoi… »), le bouton devient
`disabled`, la roue remplace rien — elle s'ajoute. `h-12 w-full` sur téléphone et au
comptoir. La variante danger prend `bg-danger` + `shadow-bouton-danger` et n'apparaît que
pour une action irréversible.

## 02 · Bouton secondaire

**Rôle.** L'issue à côté du bouton principal : annuler, remettre à plus tard, revenir.

**États.** repos · survol · appui · désactivé · avec icône · neutre.

```html
<button class="h-11 min-w-32 px-4.5 rounded-lg border-[1.5px] border-prim bg-transparent
  text-prim font-titre text-action font-semibold hover:bg-prim-soft active:translate-y-0.5">
```

**Règles.** Jamais deux secondaires côte à côte : si deux sorties existent, l'une est un
bouton discret. La variante neutre (`border-line-2 text-ink-2`) sert quand l'action n'est
pas une action de produit (« Plus tard »).

## 03 · Bouton discret

**Rôle.** Les actions de bord : trier, filtrer, modifier une ligne, ouvrir un détail.

**États.** repos · survol · danger · désactivé · icône seule (repos / survol / actif) ·
lien de retour.

```html
<button class="h-9 px-3.5 rounded-md text-ink-2 font-titre text-mini font-semibold
  transition-colors duration-90 hover:bg-tile hover:text-ink active:scale-97">
```

**Règles.** Sans fond ni contour au repos ; il ne quitte jamais son bloc. En icône seule il
passe à `size-11` (44 px) même s'il paraît plus petit. L'état actif d'un filtre est
`bg-prim-soft text-prim` — pas un fond plein.

## 04 · Pastille d'état

**Rôle.** Dire l'état d'une chose en un coup d'œil, à distance, en plein soleil.

**États.** Paiement : payé · partiel · impayé · à la charge de la société.
Chambre : occupée · à nettoyer · libre · hors service.
Envoi : en cours · en attente · échec.
Variantes : fond `-soft` (par défaut) · contour (sur `bg-tile`).

```html
<span class="h-7 pl-2 pr-2.5 rounded-pleine bg-succes-soft text-succes-fort text-mini
  font-semibold inline-flex items-center gap-1.5">
  <span class="size-2 bg-succes rotate-45"></span>Payé
</span>
```

**Règles.** **Forme + couleur, jamais la couleur seule.** Le vocabulaire de formes est fixe
et vaut pour tout le produit :

| Forme | Sens | Classe de la marque |
| --- | --- | --- |
| Losange (carré à 45°) | acquis, terminé | `size-2 bg-succes rotate-45` |
| Rond plein | en cours, partiel, occupé | `size-2 rounded-pleine bg-alerte` |
| Carré | libre, disponible | `size-2 rounded-xs bg-succes` |
| Triangle | cassé, impayé | `size-2 bg-danger [clip-path:polygon(50%_0,100%_100%,0_100%)]` |
| Cercle vide | hors ligne, non applicable | `size-2 rounded-pleine border-2 border-ink-3` |
| Roue | attente réseau indéterminée | `size-3 rounded-pleine border-2 border-info/30 border-t-info animate-roue` |

La pastille n'est pas cliquable. Si elle doit l'être, elle devient un bouton discret.

## 05 · Tuile d'action

**Rôle.** La grande cible de l'écran d'accueil : un geste du métier par tuile, quatre à six
au total.

**États.** repos · survol · appui · avec compteur · désactivé (rôle) · variante compacte.

```html
<button class="min-h-28 rounded-lg bg-surf border border-line p-4 flex flex-col
  justify-between items-start text-left shadow-basse transition-[transform,border-color]
  duration-90 ease-entree hover:border-prim active:scale-98">
```

**Règles.** Surface entièrement cliquable : pas d'îlot de clic à l'intérieur. Le compteur
(`bg-alerte`) ne s'affiche que s'il y a du travail en attente, jamais à zéro. Désactivée,
elle passe sur `bg-tile` et dit **pourquoi** (« rôle serveuse »).

## 06 · Carte chiffre

**Rôle.** Un chiffre, son étiquette, sa comparaison.

**États.** avec delta positif · avec delta négatif · sans comparaison · en chargement ·
variante contrefort.

```html
<div class="rounded-xl bg-surf border border-line p-4 shadow-basse flex flex-col gap-1.5">
  <span class="text-etiquette uppercase text-ink-3">Recette du jour</span>
  <span class="font-mono text-chiffre-l font-bold text-ink whitespace-nowrap">184 000 F</span>
  <span class="inline-flex items-center gap-1.5 text-mini font-semibold text-succes-fort">…</span>
</div>
```

**Règles.** Montant toujours en `font-mono`, et toujours écrit avec l'espace fine insécable
U+202F entre les milliers et avant le F (`184 000 F`) — voir `tokens.md` §2 : deux cartes
côte à côte alignent leurs unités sans réglage, et aucun montant ne se coupe. Le delta est un triangle **plus** une couleur, et il disparaît en zone de
vitesse (P4). `whitespace-nowrap` obligatoire : un montant ne se coupe pas.

## 07 · Bandeau d'alerte

**Rôle.** Dire ce qui s'est passé et, si besoin, offrir l'action qui répare.

**États.** info · succès · alerte · danger · variante pleine largeur (en tête d'écran).

```html
<div class="rounded-r-lg border-l-4 border-l-alerte bg-alerte-soft p-3.5 flex items-start gap-3">
```

**Règles.** Structure fixe : icône, une phrase au passé qui dit ce qui s'est produit,
l'action à droite. **Jamais deux bandeaux empilés** — le plus grave gagne, l'autre attend.
Le bandeau hors ligne est le seul à traverser toute la largeur, parce qu'il concerne
l'écran entier et non une ligne.

## 08 · Ligne de liste

**Rôle.** L'unité de tous les registres : séjours, consommations, écritures, chambres.

**États.** repos · survol · sélectionnée (contrefort indigo) · en attente d'envoi ·
annulée (barrée, 60 % d'opacité) · ligne de total.

```html
<div class="h-14 px-4 border-b border-line flex items-center gap-3 cursor-pointer
  transition-colors duration-90 hover:bg-prim-soft">
  <span class="w-9 font-mono text-corps text-ink-3">101</span>
  <span class="flex-1 flex flex-col">…</span>
  <span class="font-mono text-corps font-bold text-ink w-24 text-right">12 500 F</span>
</div>
```

**Règles.** Numéro et montant en mono — montant avec l'espace fine insécable U+202F — en
colonne de largeur fixe (`w-24 text-right`) : l'œil descend une colonne, pas un texte. La ligne entière est cliquable ;
les actions de bord n'apparaissent qu'au survol. La ligne de total est toujours hors
défilement.

## 09 · Sélecteur d'établissement

**Rôle.** Savoir **où on est** avant de faire quoi que ce soit.

**États.** fermé (un seul établissement — non cliquable) · fermé (plusieurs) · fermé avec
alerte · ouvert (courant / avec alerte / autre).

```html
<button class="h-11 px-3 rounded-lg bg-surf border border-line inline-flex items-center
  gap-2.5 shadow-basse hover:border-prim">
  <span class="size-7 rounded-md bg-ocre-soft text-ocre font-titre text-corps font-bold
    inline-flex items-center justify-center">K</span>
  …
</button>
```

**Règles.** Toujours en haut à gauche, il ne bouge jamais de place. L'initiale est en ocre
(décor), pas en indigo. Avec un seul établissement il perd son chevron et cesse d'être un
bouton. Il remonte les alertes des autres établissements mais **ne change jamais de
contexte tout seul**.

## 10 · Témoin de synchronisation

**Rôle.** Dire si le travail est en sécurité. Le composant le plus important du produit.

**États.** connecté · réseau dégradé · hors ligne · envoi en cours · retour du réseau ·
variante compacte (barre d'en-tête).

```html
<span class="relative inline-block size-2.5">
  <span class="absolute inset-0 rounded-pleine bg-succes"></span>
  <span class="absolute inset-0 rounded-pleine bg-succes animate-pulse-reseau"></span>
</span>
```

**Règles.** Trois états seulement, chacun avec sa forme et sa phrase. **Jamais de
pourcentage** : un nombre d'écritures et une heure (« 12 en attente », « hors ligne depuis
14 h 05 »). Le passage hors ligne est instantané, sans transition. Le pouls est lent
(2,4 s) : il rassure, il n'alerte pas.

## 11 · État vide illustré

**Rôle.** Dire qu'un vide est normal, et comment commencer.

**États.** vide de départ (motif ocre + action) · vide de résultat (recherche, filtre).

**Règles.** Le motif de contreforts en ocre — jamais une illustration de personnage, jamais
un dessin de marque. Trois éléments, dans cet ordre : le motif, une phrase qui dit ce qui
apparaîtra ici, l'action qui démarre. Le vide de résultat n'a pas de motif : il a l'icône
de la recherche et une porte de sortie (« chercher dans toute l'année »).

## 12 · Sélecteur segmenté

**Rôle.** Deux à quatre options courtes, toutes visibles.

**États.** deux options · trois options avec compteur · variante tactile (`h-12`).

```html
<div class="h-10 p-1 rounded-lg bg-tile inline-flex gap-1">
  <button class="h-8 px-4.5 rounded-md bg-prim text-prim-ink font-titre text-mini font-semibold">Toutes</button>
  <button class="h-8 px-4.5 rounded-md text-ink-2 font-titre text-mini font-semibold hover:text-ink">Impayées</button>
</div>
```

**Règles.** Au-delà de quatre options c'est une liste, pas un segment. Les pastilles de
durée du passage sont la variante tactile du même composant : `h-12`, `text-lead`, même
marque de sélection (rond plein cerclé du fond). La bascule se fait en courbe
**déplacement**, jamais en élastique.

## 13 · Squelette de chargement

**Rôle.** Occuper la forme exacte de ce qui arrive, pour que rien ne saute.

**États.** liste · carte chiffre · roue (attente indéterminée).

```html
<span class="h-3 w-3/5 rounded-sm bg-tile relative overflow-hidden">
  <span class="absolute inset-0 bg-linear-to-r from-transparent via-brillance to-transparent
    animate-scintillement"></span>
</span>
```

**Règles.** Même hauteur de ligne et même largeur de colonne que le contenu réel. Le
scintillement est une bande translatée (`transform` seul) — jamais un dégradé animé. La roue
est réservée à une attente réseau dont on ne connaît pas la forme (P7).

## 14 · Bandeau d'annulation

**Rôle.** Répondre à « et si je me trompe ? » sans coûter un geste à chaque action.

**États.** visible avec compte à rebours chiffré · visible avec barre · confirmation de
rétablissement · cas non annulable (confirmation explicite).

```html
<div class="h-12 pl-4 pr-2 rounded-lg bg-ink inline-flex items-center gap-3 shadow-panneau">
  <span class="flex-1 text-corps font-semibold text-bg">Consommation supprimée</span>
  <span class="font-mono text-mini text-bg/60">8 s</span>
  <button class="h-9 px-3.5 rounded-md border border-bg/30 text-bg …">Annuler</button>
</div>
```

**Règles.** Toute action destructrice s'exécute **immédiatement** et laisse 8 secondes pour
revenir : pas de fenêtre de confirmation. Le compte à rebours est visible. Une seule action
dans le bandeau. Exception : ce qui est fiscalement irréversible (facture émise) demande une
confirmation explicite et se contre-passe par un avoir — voir `documents/D7-facture-fiscale.html`.

---

## 15 · Barre de proportion — hors série, à valider

Apparue avec la clôture et le taux d'occupation. Elle porte toujours son chiffre à côté
d'elle : une barre seule ne se lit pas. `h-2 rounded-pleine bg-tile` + remplissage
`bg-prim` (part principale), `bg-ocre` (part secondaire), `bg-succes` (proportion atteinte).

**Décision à prendre** : elle entre dans le canon avec ses états, ou elle reste une
composition locale de la carte chiffre.

## 16 · Champ de saisie

**Rôle.** La pièce de **toute écriture** du produit. Les quinze premiers composants disent, montrent
et déclenchent ; celui-ci est le seul par lequel l'exploitant entre quelque chose.

**États.** repos · focus · saisie · erreur · désactivé · lecture seule · variante comptoir (`h-12`) ·
choix fermé (`<select>`).

```html
<label for="…" class="text-etiquette uppercase text-ink-3">Service à ajouter</label>
<input id="…" type="text"
  class="h-11 w-full rounded-md border-[1.5px] border-line-2 bg-surf px-3 font-texte text-corps
    text-ink transition-colors duration-90 ease-entree placeholder:text-ink-3 focus:border-prim">
```

**Où sont ses valeurs.** Il n'a **aucune maquette** : aucun des 29 fichiers de `html/` ne contient
d'`<input>`, de `<select>` ni de `<textarea>` — les onze écrans maquettés sont des écrans de lecture
et de geste. Il n'est pas inventé pour autant : `tokens.md` décrit déjà le champ sans le dessiner.

| Ce qu'il pose | Le jeton, et ce qu'il en dit littéralement |
| --- | --- |
| Bordure au repos | `--color-line-2` — « filet appuyé, **bordure de champ au repos**, ligne de total » |
| Rayon | `--radius-md` (8 px) — « **champ**, bouton discret, segment » |
| Hauteur | `h-11` (44 px) — « plancher tactile. **Jamais moins** » · `h-12` (48 px) au comptoir |
| Corps | `--text-corps` (13,5 px) · étiquette `--text-etiquette` (11 px) |
| Erreur | `--color-danger` (bordure) · `--color-danger-fort` (message) |
| Fond | `--color-surf` en saisie · `--color-tile` en lecture seule et désactivé |
| Épaisseur 1,5 px | Valeur arbitraire **assumée** — `README.md`, décision n° 1, celle du bouton secondaire |

**Règles.**

- **44 px de haut, jamais moins**, y compris pour un champ qui paraît secondaire. `h-12` au
  comptoir, comme le bouton pleine largeur.
- **Le focus et le désactivé ne se déclarent pas ici.** `theme.css` les porte globalement :
  `:focus-visible` donne l'anneau indigo de 2 px — jamais le bleu du navigateur — et `[disabled]`
  l'opacité 0,45 avec le curseur interdit. Les redéclarer créerait une seconde source de vérité.
  Le champ n'ajoute que `focus:border-prim` : l'indigo est l'action, et un champ actif se touche.
- **L'erreur porte trois signaux, jamais la couleur seule** (règle 2 des couleurs, composant 04) :
  bordure `danger`, message sous le champ, et icône `ph-fill ph-warning-circle` dans ce message. Sur
  un 1366 × 768 en plein soleil, une bordure rouge seule ne se voit pas — et pas du tout pour un
  daltonien.
- **L'étiquette est toujours visible**, au-dessus, jamais remplacée par un texte d'invite. Un champ
  dont l'étiquette disparaît à la saisie est un champ dont on ne sait plus ce qu'il attend.
- **L'aide s'efface pendant l'erreur.** Deux phrases sous un champ en font lire zéro.
- **Lecture seule ≠ désactivé.** La lecture seule se sélectionne et se copie ; elle passe sur
  `bg-tile border-line`, où le filet appuyé mentirait en suggérant qu'on peut écrire. Le désactivé,
  lui, hérite en plus de l'opacité globale.
- **Le choix fermé partage l'enveloppe** — même étiquette, même aide, même erreur — et ajoute
  `appearance-none pr-9` avec un `ph-caret-down` en `pointer-events-none`. C'est l'enveloppe qui
  fait le champ, pas le contrôle.

**Dans le code.** `app/core/design-system/ChampSaisie.vue`. Il reçoit des **clés i18n**
(`etiquette-cle`, `aide-cle`, `erreur-cle`), jamais du texte : une chaîne en dur passée en prop
afficherait la clé brute au premier rendu, au lieu d'attendre qu'un anglophone ouvre l'application.
