# Kaya — Matrice de dérivation des écrans

*Source de vérité de l'héritage visuel des écrans non maquettés. Extrait de `docs/Kaya_Design.md`
PARTIE V §25 le 2026-07-30 — ce fichier fait foi, `Kaya_Design.md` y renvoie.*

**Version 1.1.0** — `A1` À propos ajouté le 2026-08-01.

---

## Les 42 écrans du produit

| Catégorie | Nombre | Référence |
|---|---|---|
| **Écrans maquettés** | **11** codes, **29 fichiers d'états** | `docs/design/html/{code}-{nom}[-{etat}].html` |
| **Écrans dérivés** | **31** | la matrice ci-dessous |

Codes maquettés : `C4` `F2` `G2` `M4` `P2` `Q1` `R1` `R4` `R7` `S2` `V1`.

---

## Les 31 écrans codés sans maquette

## 25. Les 31 écrans codés sans maquette

C'est le document qui rend sûr le fait de coder directement. Chaque écran déclare de quel motif il hérite. **Un écran qui n'hérite d'aucun motif ne se code pas — il se maquette d'abord.**

| Écran | Hérite de | Ce qui change |
|---|---|---|
| `R2` Vue du jour | `R1` + composant 14 | Grille d'unités au lieu de tuiles |
| `R3` Check-in nuitée | `R4` | Parcours long : plus de champs, même grammaire |
| `R5` Fiche client et recherche | `R7` | Liste + fiche, pas de total |
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
| `S1` Panneau de synchronisation | Composant 8 | Développement du composant |
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
- `docs/design/composants.md` + `styleguide.html` — les 14 composants dans tous leurs états
- `docs/design/tokens.md` — les valeurs curées, qui priment sur tout export
