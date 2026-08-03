# Kaya — Matrice de dérivation des écrans

*Source de vérité de l'héritage visuel des écrans non maquettés. Extrait de `docs/Kaya_Design.md`
PARTIE V §25 le 2026-07-30 — ce fichier fait foi, `Kaya_Design.md` y renvoie.*

**Version 1.4.0** — l'écran **Notes internes** ajouté le 2026-08-03 (cycle SYN), **deuxième écran
composé** du produit. Le total passe de 44 à **45**.

C'est le premier écran du produit dont la raison d'être est de **donner un passager à un
mécanisme** : la file hors-ligne existait depuis deux cycles sans qu'aucun écran n'écrive en
classe A, et un mécanisme sans passager réel est du code exporté et appelé nulle part. `S1`, lui,
figurait déjà parmi les 32 dérivés — il n'est pas ajouté, il est **livré**.

**Version 1.3.0** — `G5` Chambres et types de chambre ajouté le 2026-08-02 (cycle HEB), et avec lui
**une catégorie qui manquait à cette matrice** : les écrans **composés**.

`docs/Kaya_Design.md` §2 porte depuis l'origine une doctrine à **trois** cas — on maquette, on
dérive, **ou on code directement** —, et son tableau « on code directement si… » en énumère les
quatre conditions. Cette matrice n'en reflétait que deux : tout écran non maquetté y était
« dérivé », c'est-à-dire rattaché à un motif dont il hérite. Un écran assemblé **uniquement** à
partir des seize composants canoniques n'hérite d'aucun motif en particulier — il n'avait donc
littéralement pas de ligne où s'inscrire, et la porte P-19 l'aurait refusé pour la mauvaise raison.

Le troisième cas est **fermé à la zone de charme** : un écran de comptoir se maquette toujours.

**Version 1.2.1** — `S1` référençait le composant **8** (ligne de liste) au lieu du **10**
(témoin de synchronisation) : reste d'une numérotation antérieure, corrigé le 2026-08-02 sur
signalement du plan du cycle 005. —  `R0` Connexion ajouté le 2026-08-01 (cycle CPT). `A1` À propos ajouté le
2026-08-01.

---

## Les 45 écrans du produit

| Catégorie | Nombre | Référence |
|---|---|---|
| **Écrans maquettés** | **11** codes, **29 fichiers d'états** | `docs/design/html/{code}-{nom}[-{etat}].html` |
| **Écrans dérivés** | **32** | la matrice ci-dessous |
| **Écrans composés** | **2** | le tableau « Les écrans composés », ci-dessous |

Codes maquettés : `C4` `F2` `G2` `M4` `P2` `Q1` `R1` `R4` `R7` `S2` `V1`.

---

## Les écrans COMPOSÉS

*Troisième cas de `docs/Kaya_Design.md` §2. Un écran composé n'hérite d'aucun motif : il est
assemblé **uniquement** à partir des seize composants canoniques de `docs/design/composants.md`.*

**Les quatre conditions doivent être remplies, toutes**, et la vérification s'écrit dans la ligne :

1. liste, formulaire ou fiche suivant un motif déjà posé ;
2. conception **entièrement** issue de la bibliothèque — vérifiée composant par composant ;
3. consulté rarement, par un utilisateur formé ;
4. personne n'a de doute sur ce à quoi il ressemble.

Et une cinquième, qui n'est pas une condition mais une conséquence : **zone de charme uniquement**.
Un écran de comptoir se maquette toujours — l'utilisateur y est debout, pressé, avec un client en
face et de l'argent en jeu, et c'est là que le dessin décide de la vitesse.

| Écran | Composants employés | Mention | Vérification |
|---|---|---|---|
| `G5` Chambres et types de chambre | **08** ligne de liste · **16** champ de saisie (dont l'état « choix fermé ») · **01 · 02 · 03** actions · **11** état vide illustré · **13** squelette de chargement | **composé** · **à valider à l'atelier terrain** | Une liste et deux formulaires ; couverture par la bibliothèque vérifiée motif par motif ; Adjoua règle son parc à l'ouverture puis y revient à la marge ; zone de charme |
| **Notes internes** (`/notes`) | **08** ligne de liste (dont l'état « en attente d'envoi ») · **16** champ de saisie · **01** bouton principal · **11** état vide illustré · **13** squelette de chargement | **composé** · **à valider à l'atelier terrain** | **(1)** une liste et un formulaire, motif posé par `G5` · **(2)** conception entièrement issue de la bibliothèque, vérifiée composant par composant — aucun élément n'est hors des seize · **(3)** une note interne se consulte rarement, par un utilisateur formé : c'est ce que l'équipe se laisse d'un service à l'autre · **(4)** personne n'a de doute sur son apparence — une liste de textes horodatés et un champ. **Zone de charme** : ni client en face, ni argent en jeu |

> **Pourquoi le choix du type de chambre emploie le composant 16 et non le 12.** La règle du
> composant 12 (contrôle segmenté) est explicite : « au-delà de quatre options c'est une liste, pas
> un segment ». Deloria a **six** types de chambre, salle de réunion comprise, et un segmenté à six
> options ne tient pas sur 372 px. C'est l'état « choix fermé » du composant 16 qui sert.
>
> **La mention « à valider à l'atelier terrain » n'est pas une formalité.** Un écran composé n'a
> aucune maquette contre laquelle comparer son rendu : le contrôle mécanique — jetons, thème sombre,
> parcours réel — le couvre, le jugement d'usage non. La mention dit ce qui reste dû.
>
> **Ce que l'écran de notes ajoute au composant 08, et qui n'est pas un état nouveau.** La ligne
> « en attente d'envoi » **figure déjà** dans les états du composant 08 (`composants.md` §08 :
> « repos · survol · sélectionnée · **en attente d'envoi** · annulée · ligne de total »). Elle
> n'avait simplement jamais été rendue, aucun écran n'écrivant encore en classe A. Le cycle 005
> l'emploie pour la première fois — il ne l'invente pas.

---

## Les 32 écrans codés sans maquette

## 25. Les 32 écrans codés sans maquette

C'est le document qui rend sûr le fait de coder directement. Chaque écran déclare de quel motif il hérite. **Un écran qui n'hérite d'aucun motif ne se code pas — il se maquette d'abord.**

| Écran | Hérite de | Ce qui change |
|---|---|---|
| `R0` Connexion | `G2` | Formulaire minimal ; états d'erreur et vides de `S3` |
| `R2` Vue du jour | `R1` + composant 14 | Grille d'unités au lieu de tuiles |
| `R3` Arrivée — **terme du lexique v1.6.0, « check-in » est écarté** ; route `/arrivee` | `R4` | Parcours long : plus de champs, même grammaire. **CODÉ** — cycle 006, `app/modules/sejours/EcranArrivee.vue` |
| `R5` Fiche client et recherche — route `/clients` | `R7` | Liste + fiche, pas de total. **Toujours INSCRIT, non codé** — voir la note de fin de cycle 006 |
| `R6` Note temps réel | `R7` | Sans l'action finale |
| `P1` Plan de salle | `R2` | Tables au lieu d'unités |
| `P3` Addition et division | `R7` | Le fractionnement est le seul motif neuf — **à valider dans `R7`** |
| `P4` Bon de dépôt pressing | `R7` | Cycle d'état en plus |
| `C1` Ouverture de shift | `G2` | Formulaire simple |
| `C2` Encaissement multi-modes | `R7` + `P3` | Fractionnement entre modes |
| `C3` Comptage et écart | `R7` + `F2` | Saisie par coupure, registre sobre |
| `F1` File de certification | `R5` | Liste filtrable, badges de `F2` |
| `F3` Avoir | `F2` | Registre sobre, manipulation guidée |
| `F4` État de reversement | `R7` | Document à lignes, export |
| `G1` Établissement et modules | `G2` | Configuration |
| `G3` Utilisateurs et rôles | `G2` | Configuration |
| `G4` Journal d'audit | `R5` + `F2` | Liste filtrable, registre sobre |
| `A1` À propos | `G2` | Configuration en **lecture seule** |
| `S1` Panneau de synchronisation — **titre « Mes envois », route `/mes-envois`** (livré au cycle SYN) | **Composant 10** — témoin de synchronisation | Développement du composant : le témoin dit l'état d'un coup d'œil, le panneau détaille ce qui attend et permet d'agir. **Le nom du fichier de page décide de la route, et une URL est visible** : `/synchronisation` aurait fait entrer par cette porte un mot que le lexique proscrit du visible |
| `S3` États vides et erreurs | Famille d'illustrations | Couvert par la fondation |
| `M1` Accueil mobile | `R1` + `M4` | Composition en régime mobile |
| `M2` Commande mobile | `P2` | C'est déjà la cible mobile de `P2` |
| `M3` Commandes QR à confirmer | `M4` + `P2` | Liste d'actions à un tap |
| `M5` Enregistrement OCR | `R3` | Étape caméra + chemin dégradé obligatoire |
| `V2` Création de réservation | `R3` | Même parcours, sans arrivée immédiate |
| `Q2` `Q3` États de la surface QR | `Q1` | États de `Q1` |
| `E1` Parc de tenants | `R5` | Liste filtrable |
| `E2` Provisionnement | `G2` | Configuration guidée |
| `E3` Abonnement | `G2` + `R7` | Paramètres + calcul |
| `E4` Diagnostic à distance | `F1` | Liste technique |
| `E5` Registre des paramètres | `G2` | Lecture seule |
| `STK` Écrans de stock | `R5` + `G2` | Liste + formulaire |

**Règle de conduite** : au moment de coder un écran dérivé, ouvrir la maquette dont il hérite et la respecter. Si l'écran a besoin d'un motif absent de la matrice, **arrêter et maquetter**.

**Note sur `A1` — inscrit avant d'être demandé.** Aucune story ne l'appelle aujourd'hui, et il ne
se construit donc pas (principe X, « prêt ≠ construit ») : cette ligne le rend *codable* le jour
où une story l'appellera, elle ne l'autorise pas à être bâti maintenant. Il existera de toute
façon — **ADM-02** y logera la version déployée et **TRX-07** le bundle de diagnostic. En
attendant, les mentions de licence des polices et icônes embarquées vivent dans `G1`, faute
d'écran d'accueil : cohérent en motif, bancal sur le fond, puisque les licences du produit ne sont
pas un réglage d'établissement. **Elles migreront vers `A1`.**

**Note sur `R0` — l'écran par lequel tout le monde entre, et que personne n'avait inscrit.** Il
n'apparaissait ni parmi les onze codes maquettés ni dans cette matrice : le cycle CPT l'a constaté
avant d'écrire une ligne de Vue, la règle opposable ci-dessous ne laissant pas d'autre issue. Il
hérite de **`G2`** pour la structure — en-tête, carte centrée, formulaire, action unique — et de
**`S3`** pour ses états d'erreur et ses états vides, qui sont la moitié de cet écran : hors ligne,
identifiants refusés, serveur injoignable. Deux contraintes propres, qui viennent de CPT-01 et non
du motif : **les deux échecs d'authentification rendent la même phrase** (FR-012), et le refus
hors ligne est annoncé **avant** toute tentative.

---

## Règle opposable

Un écran se code dans **deux cas exactement** :

1. **il est maquetté** — la référence est le fichier d'état exact de `docs/design/html/` ;
2. **il est dérivé** — la référence est sa ligne de la matrice ci-dessus, et on ouvre la
   maquette dont il hérite pour la respecter.

**Il n'y a pas de troisième cas.** Un écran absent des deux NE SE CODE PAS : la tâche s'arrête
et l'écran part en maquettage. Ni invention, ni déduction — porte **P-19** de la constitution.

Rappel : **le HTML de maquette n'est jamais copié vers `app/`.** C'est une cible, pas une
source — autonome, non sémantique, sans i18n, sans mode sombre câblé, sans RBAC. On lit ses
valeurs, on réimplémente. Seule exception : `docs/design/theme.css`, copié tel quel dans
`app/assets/css/`.

## Voir aussi

- `docs/design/lexique.md` — le vocabulaire utilisateur, opposable au même titre
- `docs/design/composants.md` + `styleguide.html` — les composants canoniques dans tous leurs états
- `docs/design/tokens.md` — les valeurs curées, qui priment sur tout export


---

## Note de fin de cycle 006 (SEJ) — ce qui a été codé, et ce qui ne l'a pas été

**Deux écrans du cycle sont livrés : `R4` Le passage et `R3` Arrivée.** `R4` est maquetté, dans
ses cinq états (`docs/design/html/R4-passage.html` et ses quatre variantes). Il est en **zone de vitesse** et ne
se compose jamais : `docs/Kaya_Design.md` §1 est formel, et `R4` porte une intention dessinée
qu'un assemblage ne retrouverait pas — les tailles de la durée et de l'heure de fin, la place du
prix sur le bouton.

`R3` est **dérivé** de `R4` — *« parcours long : plus de champs, même grammaire »* — et sa ligne
ci-dessus est passée à **« CODÉ »** dans le même changement que le fichier, jamais avant.

**Deux écrans du périmètre restent à coder**, et leur ligne ci-dessus reste **« inscrit »** :

| Écran | État | Ce qui manque |
|---|---|---|
| `R7` La note et le départ | **maquetté**, non codé | L'écran de départ ; son API est livrée et testée |
| `R5` Fiche client et recherche | **inscrit**, non codé | La liste et la fiche — dérivent de `R7` |

⚠️ **Ces deux lignes ne passent PAS à « codé », et c'est le point.** `derivation.md` est
**opposable** : la porte P-19 s'en sert pour autoriser un écran sans maquette. Inscrire « codé »
sur un écran qui n'existe pas ferait mentir le seul document qui dise ce qui a le droit d'être
codé — et le mensonge serait invisible, puisque rien ne relit un tableau de dérivation contre le
système de fichiers.

**Leur API est livrée, testée et au contrat** : les dix-sept opérations du cycle sont servies, et
`R7` a même la particularité d'avoir son moteur complet — départ, prolongation, changement
d'unité, constat de taxe figé — sans son écran. C'est un état inhabituel et il vaut d'être nommé :
le cycle a livré **le fond avant la forme**, ce qui est l'inverse de l'ordre habituel du produit.
