# Kaya — Prompts Design (fichier unique et complet)

> ⚠️ **Décompte de composants : ce fichier dit « 14 », et c'était vrai au maquettage.**
> Il y en a **seize** au 2026-08-02 — le n° 15 (barre de proportion) et le n° 16 (champ de
> saisie, composé depuis les tokens faute de maquette). `docs/design/composants.md` **fait
> foi** sur le nombre ; les mentions ci-dessous sont des constats d'époque, laissés tels quels.
> Si vous recollez un prompt de la partie III, corrigez le décompte avant de l'envoyer.

*Compagnon du Cadrage v1 et des User Stories v1 — Maquettage avant développement*
*Version 2.0 — Remplace `Kaya_Prompts_Design.md`, `Kaya_Design_Personnalite_Mouvement.md` et `Kaya_Design_Plan_Resserre.md`, qui peuvent être archivés.*

---

# PARTIE I — PRINCIPES

## 1. Le principe des deux zones

Ce produit a deux régimes, et les confondre est la seule vraie erreur possible.

| | **Zone de charme** | **Zone de vitesse** |
|---|---|---|
| Écrans | Accueil, tableau de bord propriétaire, états vides, réussites, onboarding, planning, configuration | Check-in passage, prise de commande, encaissement, clôture bloquée, écrans fiscaux, réconciliation |
| Objectif | Créer l'attachement, donner envie d'ouvrir l'application | Ne jamais faire perdre une seconde ni une attention |
| Mouvement | Généreux, expressif, illustré | Instantané, fonctionnel, discret |
| Ton | Chaleureux, complice, vivant | Direct, clair, respectueux |

**La règle qui tranche** : si l'utilisateur est debout, pressé, avec un client en face de lui ou de l'argent en jeu — zone de vitesse. Sinon — zone de charme.

Une animation en zone de vitesse n'est pas interdite ; elle doit être **décorative et jamais bloquante**. L'état change instantanément, l'animation habille le changement déjà fait. Jamais l'inverse.

## 2. Ce qu'on maquette, et pourquoi

On ne maquette pas un écran parce qu'il est important. **On maquette un écran parce qu'il pose un motif que d'autres reprendront, ou parce que sa conception n'est pas résolue.**

| On maquette si… | On code directement si… |
|---|---|
| L'interaction n'est pas résolue — il y a un vrai problème à trancher | C'est une liste, un formulaire ou une fiche suivant un motif déjà posé |
| L'écran est copié par cinq autres | Sa conception découle entièrement de la bibliothèque de composants |
| Il décide de l'adoption (fréquence ou enjeu) | Il est consulté rarement, par un utilisateur formé |
| Il doit être confronté au terrain | Personne n'a de doute sur ce à quoi il ressemble |

**Résultat : la fondation + 10 cibles maquettées, et 30 écrans dérivés** (partie V).

Le risque de coder sans maquette n'est pas la laideur, c'est la **dérive** : trente écrans inventés un par un finissent par ne plus se ressembler. La matrice de dérivation est ce qui l'évite.

## 3. Conventions

Les prompts Spec Kit référencent `docs/design/png/{code}`. Le code est **stable et ne change jamais** — il apparaît dans les tâches, les commits et les revues.

| Préfixe | Domaine |
|---|---|
| `R` | Réception et hébergement |
| `P` | Point de vente |
| `C` | Caisse |
| `F` | Fiscalité et documents |
| `G` | Configuration et administration établissement |
| `S` | États système |
| `M` | Mobile |
| `Q` | Surface publique QR |
| `V` | Réservations et planning |
| `E` | Console éditeur |
| `D` | Documents imprimés |

Export par écran : `html/{code}-{nom-lisible}.html` (référence), plus les captures dans `png/` (revue). États suffixés : `R4-passage-hors-ligne.html`, `C4-cloture-bloquee.html`.

**Le préfixe de code est obligatoire** — c'est lui que référencent les prompts Spec Kit, la matrice de dérivation et les tâches. Le nom lisible suit, pour l'humain. Correspondance complète avec les maquettes produites : §30 bis.

**Format d'export — le HTML est la référence normative.**

L'export HTML de Claude Design est ce que lit l'agent d'implémentation : il y trouve les **valeurs exactes** (couleurs, espacements, tailles, rayons, durées d'animation) et la **hiérarchie DOM**, au lieu de les estimer depuis une image. Sur les prototypes animés, les courbes et durées se lisent directement dans le CSS.

Le PNG reste produit, pour l'humain : revue rapide, impression A3 de l'atelier terrain d'Abengourou, comparaison visuelle avant/après.

| Fichier | Pour qui | Statut |
|---|---|---|
| `html/{code}.html` | **L'agent d'implémentation** | **Référence normative** |
| `png/` (captures d'écran des HTML) | L'humain — impression A3 de l'atelier | Confort de revue, produit à la demande |

> ### ⚠️ Trois règles, sans lesquelles le HTML se retourne contre toi
>
> **0. Tailwind d'abord, CSS en dernier recours.** La maquette cible un projet Nuxt 4 + Tailwind 4 : plus elle est écrite en utilitaires Tailwind, plus la transposition est directe. Voir §3 bis.
>
> **1. Le HTML de maquette n'est JAMAIS copié dans `app/` — à une exception près : `theme.css`.** Le bloc `@theme` est précisément fait pour être transplanté ; c'est lui qui porte les tokens. Tout le reste (markup, styles d'écran) est une cible, pas une source. C'est une *cible*, pas une *source*. L'export est autonome, non sémantique, sans i18n, sans gestion de rôles, sans mode sombre câblé, avec des styles en ligne. Le copier produit des composants inmaintenables qui divergeront au premier changement. **On le lit, on ne le colle pas.**
>
> **2. `tokens.md` est extrait UNE SEULE FOIS du HTML, puis prime sur lui.** Sans cette règle, chaque composant re-dérive ses valeurs depuis un fichier différent et tu obtiens quatorze nuances de gris. En cas de divergence entre un export et `tokens.md`, `tokens.md` gagne — et l'export est corrigé.
>
> **3. Un état = un fichier.** `R4.html`, `R4-hors-ligne.html`, `R4-erreur.html`. Un HTML montre naturellement un seul état ; ce que le PNG rendait évident par la multiplication des images doit rester explicite ici.

---

## 3 bis. Tailwind d'abord — règles d'écriture de la maquette

La maquette n'est pas un exercice de style libre : c'est la **spécification d'une implémentation Nuxt 4 + Tailwind 4**. Chaque écart entre son vocabulaire et celui du projet est du travail de traduction, donc une occasion de divergence.

| Ordre de préférence | Quoi | Exemple |
|---|---|---|
| **1. Utilitaires Tailwind du noyau** | Tout ce qui est exprimable ainsi | `bg-surface p-4 rounded-lg text-sm` |
| **2. Tokens déclarés en `@theme`** | Couleurs, espacements, rayons, polices du produit | `--color-surface: #…` → `bg-surface` |
| **3. Variantes Tailwind** | Mode sombre, états, points de rupture | `dark:bg-surface-dark hover:… md:…` |
| **4. CSS explicite** | Uniquement ce que Tailwind n'exprime pas | `@keyframes`, styles d'impression thermique, grilles complexes |

**Les quatre règles :**

1. **Le mode sombre passe par la variante `dark:`**, jamais par une seconde feuille de style ni une palette dupliquée. C'est ce qui rend la transposition mécanique.
2. **Les valeurs arbitraires (`w-[347px]`, `text-[#3a3a3a]`) sont un signal, pas une solution.** Elles veulent dire qu'une valeur manque à l'échelle de tokens. Elles sont autorisées en maquette, mais doivent être **listées** pour qu'on décide : soit la valeur entre dans `@theme`, soit on s'aligne sur l'échelle existante.
3. **Aucun nom de classe personnalisé** qui exigerait une feuille de style séparée en production. Si un composant se répète, c'est un composant Nuxt, pas une classe CSS.
4. **Le CSS résiduel est regroupé et commenté**, jamais dispersé en styles en ligne. On doit pouvoir répondre en dix secondes à : « qu'est-ce qui n'a pas pu se faire en Tailwind, et pourquoi ? »

> ⚠️ **Un piège à connaître** : la maquette chargera vraisemblablement Tailwind par CDN, qui génère les utilitaires à la volée. Le build Nuxt, lui, ne compile que ce qu'il trouve dans les sources. Une classe qui fonctionne dans la maquette peut donc être absente en production si elle vient d'un plugin ou d'une version différente. **S'en tenir au noyau Tailwind 4**, et vérifier au cycle 1 que le styleguide s'affiche à l'identique dans le projet réel.

**Livrables :**

```
docs/design/                        ← ÉTAT RÉEL DU DÉPÔT
├── theme.css          # bloc @theme Tailwind 4 — SEUL fichier copié dans app/
├── tokens.md          # valeurs curées, clair ET sombre — PRIME sur tout export
├── composants.md      # les 14 composants canoniques et leurs états
├── mouvement.md       # durées, courbes, sept patrons
├── styleguide.html    # les 14 composants dans tous leurs états, clair + sombre
├── README.md          # ce qui se copie, ce qui se lit, valeurs arbitraires en attente
├── lexique.md         # ✅ vocabulaire utilisateur — NORMATIF
├── derivation.md      # ⚠️ À RÉDIGER — quel écran hérite de quel motif
├── html/              # 27 fichiers — RÉFÉRENCE NORMATIVE, un par écran ET par état
│                      #   Nommage : {code}-{nom-lisible}[-{etat}].html
│                      #   Lu par l'agent, JAMAIS copié dans app/
│   ├── R1-accueil[-maquis|-serveuse|-proprietaire].html
│   ├── R4-passage[-complet|-connu|-hors-ligne|-enregistre].html
│   ├── P2-saisie-commande[-desktop|-hors-ligne].html
│   ├── R7-note-depart[-envoi|-echec].html
│   ├── C4-cloture[-bloquee|-reussie].html
│   ├── V1-planning[-dense].html
│   ├── M4-mes-etablissements[-alerte].html
│   ├── F2-registre-grave.html   S2-registre-grave.html
│   ├── Q1-page-client[-panier|-attente].html
│   └── G2-offre-hebergement[-residence].html
├── fondation/         # fondation-directions, -mouvement, -plaisir, -difficiles,
│                      #   -illustrations
├── documents/         # D1-D5-tickets-thermiques, D6-note-provisoire,
│                      #   D7-facture-fiscale
├── proto/             # proto-0-sommaire à proto-6-indicateur-sync
└── notes-terrain.md   # ⚠️ à remplir à l'atelier d'Abengourou
```

## 4. Séquencement

| Phase | Contenu | Quand |
|---|---|---|
| Fondation | Directions visuelles, palettes, typographie, 14 composants, mouvement | Phase 0, S1 |
| Les 10 cibles | Production groupée par familles | Phase 0, S1–S2 |
| Illustrations | Famille d'états vides et d'onboarding | Phase 0, S2 |
| Prototypes animés | Les 6 de la partie VI | Phase 0, S2 |
| Validation terrain | Atelier d'Abengourou, protocole partie VIII | Phase 0, S2–S3 |
| Corrections | Reprise des cibles selon les retours | Phase 0, S3 |
| Documents imprimés | Les 7 modèles | S8, avant le cycle IMP |

Tout tient dans la phase 0, sans repousser le développement.

**Production groupée** — la cohérence d'une famille est meilleure quand elle est conçue d'un bloc :

| Groupe | Contenu |
|---|---|
| 1 | Fondation |
| 2 | Cœur opérationnel — `R1`, `R4`, `P2` |
| 3 | L'argent — `R7`, `C4` |
| 4 | Moments difficiles — `F2`, `S2`, et l'état bloqué de `C4` |
| 5 | Temps et consultation — `V1`, `M4` |
| 6 | Surfaces particulières — `Q1`, `G2` |
| 7 | Illustrations et personnage |
| 8 | Prototypes animés |
| 9 | Documents imprimés (S8) |

---

# PARTIE II — SPÉCIFICATION DE SIMPLICITÉ

> Cette partie a le même statut normatif que le cadrage. Un écran qui viole une de ces règles ne passe pas la revue, qu'il soit maquetté ou codé directement.

## 5. Les neuf règles

**1. Une action principale par écran.** Un seul bouton porte le poids visuel dominant. S'il y en a deux, l'écran fait deux choses et doit être scindé. S'il n'y en a aucun, c'est un écran de consultation et il doit l'assumer.

**2. Trois décisions maximum sur un écran fréquent.** Le check-in passage demande : quelle durée, quelle chambre, on valide. Trois. Tout ajout doit justifier son coût. Sur un écran rare — la configuration — la limite ne s'applique pas.

**3. Reconnaître plutôt que se souvenir.** L'utilisateur ne mémorise jamais un code, un numéro de chambre, un tarif, un état. Tout ce dont il a besoin est visible au moment où il en a besoin. Corollaire : aucun champ ne demande une information déjà présente dans le système.

**4. Des valeurs par défaut justes neuf fois sur dix.** Une chambre est proposée automatiquement. La formule la plus vendue est présélectionnée. La date est aujourd'hui. L'établissement est celui de la dernière session. Un défaut juste supprime une décision ; un défaut faux en crée deux.

**5. Annuler plutôt que confirmer.** Une modale de confirmation est un aveu : le produit ne sait pas revenir en arrière. Sur les actions réversibles — supprimer une ligne non envoyée, changer de chambre, modifier une quantité — l'action se fait immédiatement, avec une possibilité d'annuler pendant quelques secondes. La confirmation reste réservée à l'irréversible : certification fiscale, clôture, avoir, encaissement.

**6. Zéro jargon.** Voir `docs/design/lexique.md`. Aucun terme technique, aucun sigle non expliqué, aucun mot que la gérante d'un maquis ne dirait pas spontanément. Le vocabulaire fiscal officiel apparaît uniquement sur les documents légaux, jamais dans les boutons ni les messages.

**7. Une erreur dit toujours quoi faire ensuite.** Trois éléments obligatoires : ce qui s'est passé, pourquoi, l'action suivante. Une erreur sans porte de sortie est un défaut de conception, pas un message à réécrire. Et elle n'accuse jamais l'utilisateur.

**8. Rien d'indispensable derrière un geste non découvrable.** Pas de balayage secret, pas d'appui long comme unique accès, pas de menu contextuel obligatoire. Les raccourcis existent, ils ne sont jamais le seul chemin.

**9. Un écran vide propose toujours une porte de sortie.** Aucune réservation aujourd'hui → « créer une réservation ». Aucun document en attente → un mot rassurant. Un écran vide sans action est une impasse.

## 6. Lexique — ce que l'utilisateur lit

> 📦 **Déplacé le 2026-07-30 vers `docs/design/lexique.md`, qui fait désormais foi.**
> Le tableau n'est pas dupliqué ici, pour la même raison qu'en partie V.

Le produit manipule des concepts fiscaux et techniques réels. **L'utilisateur ne doit jamais
les rencontrer sous leur nom d'origine.** Les 15 correspondances — « certification FNE » →
« envoi aux impôts », état `INDETERMINEE` → « nous ne savons pas si les impôts ont reçu cette
facture » — sont dans `docs/design/lexique.md`, avec la procédure d'ajout d'une entrée.

## 7. Les cinq tests

À faire passer à chaque cible maquettée, et à chaque écran dérivé avant sa fusion.

| Test | Protocole | Échec si |
|---|---|---|
| **Cinq secondes** | Montrer l'écran cinq secondes, le masquer, demander : à quoi sert-il ? quelle est l'action principale ? | La personne ne sait pas répondre |
| **Première fois sans formation** | Donner une tâche à quelqu'un qui n'a jamais vu le produit, sans un mot d'explication | Il faut intervenir |
| **Chronomètre** | Mesurer la tâche sur les écrans fréquents | Passage > 45 s · commande de 6 lignes > 60 s · clôture > 20 min |
| **Le cahier** | Adjoua fait la même tâche sur le logiciel et sur son cahier | Le cahier gagne |
| **Huit heures** | Faire tourner les animations répétitives en boucle dix minutes | On veut les couper |

**Le test du cahier est le juge de paix.** Ce produit ne concurrence pas un autre logiciel, il concurrence un objet en papier que le personnel maîtrise parfaitement depuis des années. Tant qu'une tâche est plus rapide sur le cahier, elle est mal conçue — quelle que soit la beauté de l'écran.

---

# PARTIE III — PROMPTS DE FONDATION

## 8. Fondation (à coller une seule fois)

```
Je conçois le design system d'une application de gestion pour hôtels, maquis, bars
et pressings en Afrique de l'Ouest. Elle s'appelle Kaya (nom provisoire). Avant
tout écran, produis-moi la fondation : identité visuelle, palettes, typographie,
échelles, système de mouvement, et les 14 composants canoniques.

L'AMBITION : je veux un logiciel de gestion qui donne ENVIE d'être ouvert. La
quasi-totalité des logiciels vendus aux PME africaines sont gris, froids et
vaguement hostiles. Le mien doit être chaleureux, vivant et un peu joueur — c'est
un vrai différenciateur commercial, pas de la coquetterie. Une gérante qui trouve
son logiciel agréable l'utilisera ; une gérante qui le subit reviendra au cahier.

MAIS IL Y A DEUX ZONES, et tu dois concevoir pour les deux :

ZONE DE CHARME — accueil, tableau de bord du propriétaire, états vides, écrans de
réussite, onboarding, planning, configuration. Ici je veux de la générosité : des
illustrations, du mouvement expressif, des micro-récompenses, de la couleur. C'est
là que se crée l'attachement.

ZONE DE VITESSE — enregistrement d'un client pressé au comptoir, prise de commande
debout dans un bar, encaissement, clôture de caisse, écrans fiscaux. Ici tout est
instantané et rien ne distrait. Une animation peut habiller un changement d'état
DÉJÀ EFFECTUÉ, jamais le retarder ni le conditionner.

Ce n'est pas deux design systems. C'est un seul système avec deux réglages
d'intensité. Montre-moi comment tu articules ça.

CONTEXTE D'USAGE — il commande tout :
- Utilisateurs : une gérante d'hôtel de 45 ans à Abengourou, un réceptionniste,
  une serveuse de bar de 25 ans, un propriétaire qui consulte depuis son
  téléphone. Aucun n'a d'appétence technologique. Ils remplacent un cahier papier
  qu'ils maîtrisent parfaitement.
- Matériel réel : poste de réception Windows d'entrée de gamme, 1366×768, souvent
  en plein soleil. Android d'entrée de gamme pour la serveuse, 5,5", utilisé
  debout, à une main, dans un bar en lumière faible.

DIRECTION VISUELLE — je veux une piste ancrée culturellement, pas un thème SaaS
international de plus. L'Afrique de l'Ouest a une culture visuelle riche :
géométrie rythmée, couleurs franches et chaudes, contrastes assumés. Inspire-t'en
STRUCTURELLEMENT — rythme, proportions, saturation — sans jamais tomber dans le
pastiche décoratif ni le motif plaqué en fond d'écran. Le résultat doit être
raffiné et intemporel, pas folklorique.

Propose-moi DEUX directions visuelles distinctes, chacune avec une planche
d'ambiance et une justification. Je choisirai. Ne fusionne pas les deux.

CONTRAINTES NON NÉGOCIABLES :
1. MODE SOMBRE dès maintenant. Deux palettes complètes, pas une palette et un
   filtre. Le sombre sert le bar le soir, le clair sert la réception le jour. Le
   sombre est une occasion d'élégance, traite-le comme tel.
2. FRANÇAIS PAR DÉFAUT, anglais prévu. Dessine avec les libellés français, les
   plus longs. Aucun bouton dont la largeur est calée sur son texte.
3. MONTANTS EN FRANC CFA : entiers, jamais de décimale, séparateur de milliers par
   espace fine, de 500 à 500 000. Typographie tabulaire pour l'alignement des
   colonnes. Les montants méritent une attention particulière — c'est ce que les
   gens regardent le plus.
4. CONTRASTE : la réception est en plein soleil. WCAG AA minimum, AAA sur les
   montants et les statuts. Aucune information portée par la seule couleur.
5. CIBLES TACTILES : 48 px minimum sur mobile, 40 px sur desktop tactile.
6. PERFORMANCE : Android d'entrée de gamme. Tout le mouvement en transform et
   opacity uniquement, jamais en propriétés déclenchant un recalcul de mise en
   page. Budget : 60 images par seconde tenues sur un appareil à 2 Go de RAM.
7. SIMPLICITÉ : une action principale par écran, trois décisions maximum sur un
   écran fréquent, aucune information à mémoriser, zéro jargon.

LES 14 COMPOSANTS CANONIQUES, avec tous leurs états ET leurs micro-interactions :
1. Bouton — primaire, secondaire, discret, destructif ; normal, survol, pressé,
   désactivé, en cours. Le pressé doit être satisfaisant.
2. Champ de saisie — texte, nombre, montant, date, heure, recherche.
3. Sélecteur — liste déroulante et groupe de pastilles.
4. Tableau de données — en-tête collant, ligne, sélection, montants à droite,
   total, état vide illustré.
5. Modale — confirmation, formulaire, destructive. Rappel : elle est RARE dans ce
   produit, réservée à l'irréversible.
6. Tuile de KPI — avec transition chiffrée quand la valeur change.
7. SÉLECTEUR DE CONTEXTE — barre permanente, établissement et poste actifs,
   bascule en deux taps.
8. INDICATEUR DE SYNCHRONISATION — connecté, dégradé, hors ligne, avec le nombre
   d'éléments en attente. Permanent à l'écran : donne-lui de la vie, c'est un des
   rares endroits où une animation continue est justifiée.
9. Badge de statut — unités, documents, commandes. Couleur ET forme.
10. Navigation composable — dessine-la avec 3 entrées, 6 et 9. Les modules absents
    ne sont pas grisés, ils n'existent pas.
11. Panneau latéral — détail d'une note, d'un séjour, d'un document.
12. Bandeau d'alerte — information, avertissement, erreur, blocage.
13. SÉLECTEUR DE DURÉE — le composant le plus utilisé du produit : choisir 1 h,
    2 h, 3 h, 4 h ou une plage horaire en un geste. Il mérite le meilleur travail
    de la fondation. Zone de vitesse : instantané, mais il peut être beau.
14. Sélecteur d'unité — grille de chambres avec statut, sélectionnable.

FORMAT DE LIVRAISON : produis un STYLEGUIDE HTML AUTONOME (styleguide.html)
présentant les 14 composants dans tous leurs états, en mode clair ET sombre.

ÉCRIS-LE EN TAILWIND 4, PAS EN CSS. Ma cible est un projet Nuxt 4 + Tailwind 4 :
- déclare TOUS les tokens dans un bloc @theme (couleurs, espacements, rayons,
  polices, durées) — c'est le seul fichier que je copierai tel quel dans mon
  projet ;
- écris le markup avec des UTILITAIRES TAILWIND DU NOYAU référençant ces tokens
  (bg-surface, p-4, rounded-lg), jamais avec des styles en ligne ni des classes
  personnalisées ;
- le mode sombre passe par la variante dark:, jamais par une seconde palette ;
- réserve le CSS explicite à ce que Tailwind n'exprime pas — @keyframes, styles
  d'impression — et REGROUPE-LE en fin de fichier avec un commentaire expliquant
  pourquoi chaque bloc n'a pas pu être fait en Tailwind ;
- si tu emploies une valeur arbitraire (w-[347px]), LISTE-LA en fin de livraison :
  soit elle entre dans @theme, soit on s'aligne sur l'échelle existante. C'est ce fichier qui servira de
référence à l'implémentation Nuxt + Tailwind 4 : les valeurs doivent y être
exactes et lisibles, pas approximatives. Je ne le copierai pas dans mon code — je
le lirai pour en extraire les tokens une seule fois.

LIVRE-MOI : les deux directions visuelles, puis pour celle que je retiendrai les
deux palettes en valeurs hexadécimales nommées sémantiquement (surface,
surface-elevee, texte, texte-attenue, bordure, primaire, succes, alerte, danger,
info — pas de nom de couleur littéral), l'échelle typographique avec la police et
sa justification, les espacements, rayons et élévations, et les 14 composants en
clair et en sombre.
```

## 9. Système de mouvement

```
Établis le système de mouvement de Kaya. Je veux qu'il soit défini une fois et
appliqué partout, pas improvisé écran par écran.

DÉFINIS-MOI :

1. DURÉES — une échelle nommée, pas des valeurs au hasard : instantané (retour
   tactile), rapide (changement d'état), standard (transition d'écran), ample
   (moment de célébration). Donne les millisecondes et l'usage de chacune.

2. COURBES — les fonctions d'accélération avec leur rôle. Distingue au moins :
   entrée d'élément, sortie d'élément, déplacement, et une courbe élastique
   réservée aux moments de plaisir.

3. LES SEPT PATRONS DE MOUVEMENT DU PRODUIT :
   - Apparition d'une liste — décalage progressif entre éléments, PLAFONNÉ pour
     qu'une liste de 40 lignes ne prenne pas trois secondes.
   - Transition entre écrans — direction porteuse de sens (avancer, revenir).
   - Ouverture du panneau latéral.
   - Changement de valeur d'un montant — le total d'une note qui monte quand on
     ajoute une consommation. Ce moment doit être perceptible et satisfaisant :
     c'est le geste le plus répété du produit.
   - Retour tactile sur un bouton — la sensation de fermeté.
   - Changement d'état d'un badge — une chambre qui passe d'occupée à à-nettoyer.
   - État de chargement — squelettes plutôt que roues qui tournent, sauf pour les
     attentes réseau réellement indéterminées.

4. LE RÉGLAGE D'INTENSITÉ — comment le même patron s'exprime en zone de charme
   (généreux) et en zone de vitesse (réduit au minimum perceptible). Montre-moi un
   exemple du même composant dans les deux régimes.

5. ACCESSIBILITÉ ET PERFORMANCE :
   - Respect de la préférence système « réduire les animations » : tout devient
     instantané, rien ne casse, rien ne manque.
   - Aucune animation ne bloque une saisie. On peut toujours taper pendant une
     transition.
   - Aucune animation ne dépasse 400 ms sur un chemin fréquent. Au-delà, ce n'est
     plus du plaisir, c'est de l'attente.
   - Transform et opacity uniquement.

Livre-moi le système en tableau de valeurs nommées, exploitable directement en
variables CSS, plus des prototypes HTML animés des sept patrons — je veux juger la
sensation, pas la description.
```

## 10. Moments de plaisir

```
Identifie et conçois les MOMENTS DE PLAISIR de Kaya — les instants où le produit a
le droit, et le devoir, d'être expressif.

J'en vois huit. Confirme, corrige, complète, puis conçois-les.

1. LA CLÔTURE RÉUSSIE — le sommet émotionnel de la journée d'Adjoua. Elle passait
   une heure sur son cahier ; là ça lui prend quinze minutes et les chiffres
   tombent justes. Ce moment mérite une vraie récompense visuelle. Pas des
   confettis génériques : quelque chose qui exprime « c'est bouclé, c'est propre,
   tu peux rentrer chez toi ». C'est l'animation la plus importante du produit.

2. LE PASSAGE ENREGISTRÉ EN MOINS DE 30 SECONDES — une micro-célébration très
   brève qui ne retarde rien. La chambre s'allume, l'heure de fin s'inscrit. Yao
   doit ressentir la fluidité sans jamais l'attendre.

3. LA FACTURE CERTIFIÉE — l'administration a validé, le sceau et le QR code
   apparaissent. Il y a du soulagement dans ce moment : la conformité est acquise.
   Traite l'arrivée du sceau officiel comme un petit événement.

4. LE RETOUR DU RÉSEAU — les éléments en attente partent et le compteur descend
   jusqu'à zéro. Aminata a travaillé une heure hors ligne en se demandant si tout
   était perdu. La resynchronisation doit être visiblement rassurante.

5. LES ÉTATS VIDES — aucune réservation aujourd'hui, aucune table ouverte, aucun
   document en attente. Une vingtaine d'occasions gratuites de personnalité.
   Conçois-en une famille cohérente, illustrée, chaleureuse, jamais condescendante.

6. LE PREMIER LANCEMENT — la toute première ouverture chez un nouveau client, et
   le premier séjour créé. C'est le moment où l'établissement bascule du papier au
   logiciel. Il mérite d'être marqué.

7. LE TABLEAU DE BORD DU PROPRIÉTAIRE — M. Koffi le consulte plusieurs fois par
   jour, par plaisir autant que par nécessité. Chiffres qui montent à
   l'apparition, indicateurs qui prennent vie, comparaison entre établissements
   qui s'anime. L'écran le plus « produit grand public » du système.

8. L'INDICATEUR DE SYNCHRONISATION — présent partout en permanence. Une
   respiration discrète en mode connecté, un rythme différent en mode dégradé.
   C'est le pouls du produit.

Pour chacun : conçois-le, et donne-moi un prototype HTML animé que je puisse
ressentir. La description ne suffit pas pour juger un moment de plaisir.

CONTRAINTE ABSOLUE : aucun de ces moments ne rallonge une action. Ils habillent un
état déjà atteint. Si l'animation de la clôture dure deux secondes, Adjoua peut
partir au bout de zéro seconde — l'animation continue derrière elle.
```

## 11. Le registre sobre — où le plaisir est interdit

```
Certains écrans de Kaya ne doivent JAMAIS être joyeux. Établis-moi leur traitement,
aussi soigné que le reste mais dans un registre différent : sobre, respectueux,
direct.

LA RÈGLE : quand quelqu'un perd de l'argent, risque une sanction fiscale, ou est
fatigué à 22 h devant un blocage — l'enjouement se lit comme du mépris.

Les écrans concernés :

- CLÔTURE BLOQUÉE — Adjoua veut rentrer chez elle et le logiciel refuse. Le ton :
  « voici exactement ce qui bloque, voici comment le résoudre, je suis de ton
  côté ». Pas d'illustration amusante, pas de personnage désolé. De la clarté et
  de l'aide.

- DOCUMENT FISCAL INDÉTERMINÉ — on ne sait pas si la facture a été certifiée, il
  faut aller vérifier manuellement. Aucune légèreté, aucune icône expressive. Un
  texte impeccable et deux actions bien différenciées.

- RÉCONCILIATION D'UNE ÉCRITURE ORPHELINE — une consommation est arrivée sur une
  facture déjà émise. De l'argent en jeu, une décision à prendre.

- ÉCART DE CAISSE — quelqu'un va devoir s'expliquer. Le produit constate, il ne
  commente pas et il n'accuse pas.

- JOURNAL D'AUDIT — outil de contrôle du propriétaire sur son personnel, il sera
  vécu comme de la surveillance. Vocabulaire neutre, aucune connotation policière,
  aucune mise en scène.

- PERTE DE DONNÉES OU ÉCHEC DE SYNCHRONISATION IRRÉCUPÉRABLE — le pire moment
  possible. Sobriété totale, information complète, chemin de sortie clair.

Montre-moi comment ces écrans restent VISUELLEMENT COHÉRENTS avec le reste — même
typographie, même palette, mêmes composants — tout en changeant complètement de
registre. C'est un exercice de retenue, pas d'appauvrissement.
```

## 12. Illustrations et personnage

```
Conçois la famille d'illustrations de Kaya.

USAGES : une vingtaine d'états vides, six écrans d'onboarding, quatre moments de
réussite, trois écrans d'erreur non critique.

CONTRAINTES :
- Style vectoriel simple, lisible en petit, fonctionnant en clair ET en sombre.
- Poids maîtrisé : chargées sur des connexions faibles.
- REPRÉSENTATION JUSTE : si des personnes apparaissent, ce sont des Africains de
  l'Ouest dans un environnement crédible — un comptoir de réception, un bar, une
  buanderie de pressing. Pas d'illustration internationale interchangeable, et
  surtout pas de cliché exotique. Mes utilisateurs doivent se reconnaître.
- Univers cohérent : un même vocabulaire de formes, une palette restreinte.

QUESTION OUVERTE — un personnage récurrent ?
Propose-moi une piste AVEC et une piste SANS.
Avec : un personnage discret dans les états vides et l'onboarding, qui donne de la
chaleur et aide à la mémorisation. Risque : il vieillit mal, il agace à la
centième apparition, il coûte cher à décliner.
Sans : des illustrations de situation, plus sobres, plus faciles à étendre.

Dis-moi laquelle tu recommandes et pourquoi. Je trancherai après avoir montré les
deux à ma cliente pilote — c'est elle qui sait ce qui plaira à son équipe.
```

---

# PARTIE IV — LES 10 CIBLES

## 13. Rappel à insérer dans chaque prompt d'écran

```
Applique la fondation : direction visuelle retenue, palettes clair et sombre,
typographie, espacements, système de mouvement, et les 14 composants canoniques —
n'invente pas de nouveau composant sans me dire pourquoi ceux qui existent ne
suffisent pas.

Précise pour cet écran : sa ZONE (charme ou vitesse) et le réglage d'intensité de
mouvement qui en découle ; les micro-interactions présentes avec leur durée et
leur courbe ; et s'il porte un moment de plaisir identifié, sa conception complète.

CONTRAINTE DE SIMPLICITÉ :
- UNE seule action principale, portant le poids visuel dominant.
- Trois décisions au maximum si l'écran est fréquent. Dis-moi combien tu en
  demandes et lesquelles tu as retirées.
- Aucune information à mémoriser.
- Des valeurs par défaut justes neuf fois sur dix. Explique tes choix.
- Actions réversibles : exécution immédiate avec possibilité d'annuler. Pas de
  modale de confirmation, sauf sur l'irréversible.
- ZÉRO jargon : applique docs/design/lexique.md. Si un concept n'y figure pas,
  propose-moi sa formulation.
- Toute erreur dit ce qui s'est passé, pourquoi, et quoi faire ensuite. Elle
  n'accuse jamais l'utilisateur.
- Rien d'indispensable derrière un geste non découvrable.
- Un écran vide propose toujours une action.

FORMAT : livre chaque écran en HTML AUTONOME, un fichier par état, important le
même bloc @theme que styleguide.html. C'est la référence que lira l'agent
d'implémentation — les valeurs doivent être exactes.
TAILWIND 4 D'ABORD : utilitaires du noyau référençant les tokens, variante dark:
pour le mode sombre, aucune classe personnalisée, aucun style en ligne. CSS
explicite uniquement pour ce que Tailwind n'exprime pas, regroupé et commenté.
Signale toute valeur arbitraire employée.

PRODUIS SYSTÉMATIQUEMENT :
- l'écran en mode clair et en mode sombre ;
- l'état chargé nominal, l'état vide ILLUSTRÉ, l'état d'erreur ;
- l'état hors ligne quand l'écran comporte une action indisponible sans réseau —
  annoncée AVANT que l'utilisateur tente l'action, jamais après ;
- l'état en préférence « animations réduites » si le mouvement est structurant.

Le sélecteur de contexte et l'indicateur de synchronisation sont présents partout.
Libellés en français. Montants en FCFA entiers.

Puis fais passer le TEST DES CINQ SECONDES à ta propre proposition : si je regarde
cet écran cinq secondes, qu'est-ce que je comprends, et quelle action me paraît
évidente ? Si tu n'es pas sûr de la réponse, simplifie avant de me livrer.
```

## 14. Zone de chaque cible

| Cible | Zone | Traitement |
|---|---|---|
| `R1` Accueil | **Charme** | Générosité assumée, apparition en cascade, tuiles vivantes |
| `R4` Check-in passage | **Vitesse** | Instantané. Micro-célébration de 200 ms non bloquante |
| `P2` Prise de commande | **Vitesse** | Instantané. Le total qui monte est le seul mouvement |
| `R7` Note et check-out | **Vitesse** | Sobre pendant la certification, moment de plaisir n°3 à la réussite |
| `C4` Clôture | **Les deux** | Sobre en vérification et blocage, moment de plaisir n°1 à la réussite |
| `V1` Planning | **Charme** | Visualisation soignée, transitions amples |
| `M4` Tableau de bord propriétaire | **Charme** | Vitrine du produit, moment de plaisir n°7 |
| `F2` / `S2` Moments difficiles | **Registre sobre** | La partie III §11 s'applique intégralement |
| `Q1` Page publique QR | **Charme** | Seule surface vue par un client final |
| `G2` Formules et barèmes | **Charme** | Configuration soignée, mais paramètre fiscal sensible |

## 15. Cible 1 — `R1` Accueil composé

```
Écran : la page d'accueil de l'application, après connexion.

LA CONTRAINTE STRUCTURELLE : ce n'est pas un menu figé. C'est un tableau de bord
composé de tuiles filtrées par les permissions de l'utilisateur ET par les modules
d'activité actifs sur l'établissement. Un module inactif est ABSENT, jamais grisé,
jamais mentionné.

Les rôles sont CUMULABLES et c'est la norme : Adjoua est gérante, caissière ET
réceptionniste. L'écran doit servir cette réalité sans devenir un fourre-tout.

Dessine-moi la MÊME page pour quatre utilisateurs :
1. Adjoua — gérante + caissière + réceptionniste, hôtel complet (hébergement,
   restaurant, bar, pressing, salle de réunion).
2. Aminata — serveuse seule, même établissement.
3. M. Koffi — propriétaire de deux établissements, ne saisit jamais rien.
4. Le gérant d'un MAQUIS — module restauration uniquement, rien d'autre. Cet écran
   doit avoir l'air conçu POUR LUI, pas d'un hôtel amputé de ses fonctions.

LE QUATRIÈME CAS EST LE TEST DE VÉRITÉ DU PRODUIT. S'il donne l'impression d'un
logiciel d'hôtel dont on aurait masqué des morceaux, la conception est à revoir.

Contenu type : actions du moment (arrivées, départs, tables ouvertes, unités à
nettoyer), alertes (documents non certifiés, écart de caisse, stock bas), accès aux
modules actifs, indicateurs du jour.

Variantes : les quatre utilisateurs, plus l'état de début de journée où rien ne
s'est encore passé.

MOTIF POSÉ : la composition par permissions et modules. Onze autres écrans en
hériteront.
```

## 16. Cible 2 — `R4` Check-in passage

```
Écran : enregistrement d'un PASSAGE — location horaire d'une chambre, de 1 à 4 h,
la formule la plus fréquente en volume dans une grande partie du parc hôtelier
ivoirien.

Persona : Yao, réceptionniste. Un client se présente au comptoir, il est pressé, il
n'y a pas de conversation. Yao encaisse souvent en même temps.

LA CONTRAINTE QUI COMMANDE TOUT : le parcours complet doit tenir en MOINS DE
30 SECONDES, depuis l'écran d'accueil jusqu'à la chambre attribuée. Au-delà de
90 secondes, le personnel contournera le logiciel et reprendra le cahier. C'est un
critère d'acceptation, pas un souhait.

Conséquence : ce n'est PAS le formulaire de check-in classique avec des champs en
moins. C'est un parcours distinct, conçu pour la vitesse.

Contenu :
- Choix de la durée en UN GESTE : 1 h (1 500), 2 h (2 800), 3 h (4 000),
  4 h (5 000). Le prix est visible SUR le bouton, pas dans un récapitulatif.
- Attribution d'une chambre disponible : proposition automatique, changement
  possible en un tap sur la grille d'unités.
- Identité du client : réduite au strict nécessaire légal. Montre-moi ce que tu
  proposes et DIS-MOI CE QUE TU AS RETIRÉ.
- Heure de fin calculée et affichée en grand — c'est l'information que Yao devra
  redonner au client.
- Confirmation en un tap.

Montre-moi DEUX propositions de parcours différentes, avec le NOMBRE DE GESTES de
chacune, et dis-moi laquelle tu recommandes et pourquoi.

Variantes : nominal, aucune chambre disponible dans la catégorie, client déjà
connu (reconnu au téléphone), hors ligne.

MOTIF POSÉ : le parcours court — décision unique, confirmation immédiate.
```

## 17. Cible 3 — `P2` Prise de commande

```
Écran : saisie d'une commande au bar ou au restaurant.

Persona : Aminata, serveuse, 25 ans. Android d'entrée de gamme, écran 5,5", utilisé
DEBOUT et À UNE MAIN, dans un bar en lumière faible, avec du bruit. Le réseau tombe
régulièrement.

Contraintes :
- Ajout d'un article en DEUX TAPS maximum depuis l'écran de commande.
- Le catalogue de Deloria fait une trentaine d'articles en catégories.
- La quantité se modifie sans ouvrir de modale.
- CET ÉCRAN FONCTIONNE INTÉGRALEMENT HORS LIGNE — c'est le cœur du besoin. L'état
  hors ligne ne doit pas être anxiogène : Aminata doit savoir que sa saisie est
  conservée, sans que ça la ralentisse. Trouve le traitement juste entre
  « invisible » (elle ne saura pas que ses commandes ne sont pas parties) et
  « alarmant » (elle arrêtera de saisir).
- La cible de facturation — table, chambre, comptoir — se choisit à l'ouverture,
  pas à chaque ligne. Quand le module hébergement n'est pas actif (un maquis seul),
  la cible « chambre » N'EXISTE PAS DU TOUT : ne la grise pas, retire-la.
- Le total courant est visible en permanence sans masquer le catalogue.

Variantes : nominal, hors ligne avec 4 lignes en attente, catalogue filtré par
recherche, ligne envoyée qu'on veut annuler (avec motif obligatoire), commande vide.

Produis AUSSI la version desktop — le même écran sur le poste du restaurant, en
tirant parti de la largeur sans devenir un écran différent.

MOTIF POSÉ : la saisie répétitive hors ligne, à une main.
```

## 18. Cible 4 — `R7` Note de séjour et check-out

```
Écran : la note d'un client logé, et sa clôture avec émission du document fiscal.

Deux états d'un même écran : la note vivante pendant le séjour, puis le check-out.

LA NOTE — c'est l'un des cinq problèmes explicites du cahier des charges du
pilote : « le total provisoire d'une chambre n'est pas toujours visible
instantanément ». Contenu : lignes d'hébergement, consommations par point de vente,
extras, remises, et LE TOTAL PROVISOIRE mis en évidence. Sur impression, mention
obligatoire « Document non fiscal — ne tient pas lieu de facture ».

LE CHECK-OUT — récapitulatif, calcul final, TAXE DE SÉJOUR EN LIGNE DISTINCTE
(c'est une obligation légale, elle ne peut pas être fondue dans le prix), TVA, taxe
de développement touristique, puis envoi aux impôts pour validation.

L'envoi aux impôts prend quelques secondes et peut échouer. Montre-moi le
traitement de l'attente, du succès et de l'échec — sachant que LE CLIENT EST DEBOUT
DEVANT LE COMPTOIR pendant ce temps. Le succès porte le moment de plaisir n°3 :
l'arrivée du sceau officiel et du QR code.

Variantes : note vide, note chargée de 20 lignes, note avec remise, note d'un
séjour au mois, envoi en cours, envoi échoué, plateforme fiscale injoignable,
départ anticipé avec régularisation.

MOTIF POSÉ : le document à lignes — lignes, sous-totaux, taxes, total, action
finale. Six autres écrans en hériteront, dont l'addition, l'encaissement et les
documents imprimés.
```

## 19. Cible 5 — `C4` Clôture journalière

```
Écran : clôture de la journée d'exploitation.

Persona : Adjoua, gérante. Elle fait ça tous les soirs. Aujourd'hui ça lui prend
environ une heure sur le cahier papier. L'objectif mesuré est MOINS DE 15 MINUTES.

C'est l'écran qui décide si Adjoua préfère le logiciel au cahier. Traite-le comme
le plus important du produit.

Contenu : recettes par service (hébergement, restaurant, bar, pressing, salle de
réunion), encaissements par mode de règlement, taxes collectées, écarts de caisse.
Ventilation de l'hébergement PAR FORMULE — nuitées, passages, demi-journées.
Distinguer les recettes de passage est un besoin réel qu'aucun cahier ne couvre.

LE POINT DÉLICAT : la clôture est REFUSÉE tant que quatre conditions ne sont pas
réunies — plus rien en attente d'envoi, aucune facture en attente ou refusée par
les impôts, aucun terminal déconnecté depuis plus de 15 min, aucune addition de
table restée ouverte.

Le refus doit être CONSTRUCTIF, pas punitif. Adjoua doit voir immédiatement ce qui
bloque, combien d'éléments sont concernés, et pouvoir agir DEPUIS CET ÉCRAN sans
partir en chasse. Un blocage opaque à 22 h la fera revenir au cahier dès le
lendemain. Registre sobre (partie III §11).

Montre-moi comment tu traites : un blocage unique et simple ; trois blocages
simultanés ; un blocage qu'Adjoua ne peut pas résoudre elle-même (la plateforme
fiscale est injoignable).

LA RÉUSSITE porte le moment de plaisir n°1 — l'animation la plus importante du
produit. « C'est bouclé, c'est propre, tu peux rentrer chez toi. »

Variantes : clôture possible, clôture bloquée (trois cas), clôture en cours,
réussie, journée déjà clôturée.

MOTIF POSÉ : la vérification préalable et le blocage constructif.
```

## 20. Cible 6 — `V1` Planning horaire

```
Écran : le planning d'occupation des unités.

CE QUI LE DISTINGUE DE TOUS LES PLANNINGS HÔTELIERS EXISTANTS : il a une
GRANULARITÉ HORAIRE. Les passages de 1 à 4 heures et les demi-journées doivent y
être lisibles, pas écrasés dans une case de journée. Une même chambre peut avoir
une nuitée, puis deux passages, puis une demi-journée dans la même semaine.

Ajoute les temps de remise en état entre deux occupations — 30 min après un
passage, 2 h après une nuitée — qui bloquent réellement l'unité.

C'est un problème de visualisation difficile : MONTRE-MOI DEUX APPROCHES
DIFFÉRENTES et recommande-en une.

Variantes : semaine calme, semaine dense avec passages multiples, vue d'une
journée, conflit de disponibilité signalé.

MOTIF POSÉ : la visualisation temporelle à granularité fine.
```

## 21. Cible 7 — `M4` Tableau de bord propriétaire mobile

```
Écran : la vue consolidée de M. Koffi sur ses deux établissements, sur téléphone.

C'est LA DEMANDE EXPLICITE du persona propriétaire : voir en temps réel ce qui se
passe dans ses établissements sans se déplacer.

8 à 10 indicateurs, comparaison entre établissements, alertes. Il ne saisit jamais
rien — cet écran est en lecture, conçu pour être consulté en 20 secondes plusieurs
fois par jour.

ZONE DE CHARME, et vitrine du produit : c'est l'écran le plus « grand public » du
système, celui qu'il montrera à ses relations. Il porte le moment de plaisir n°7 —
chiffres qui montent à l'apparition, indicateurs qui prennent vie, comparaison qui
s'anime.

Variantes : deux établissements, cinq établissements, un établissement en alerte,
début de journée.

MOTIF POSÉ : la consultation mobile et le régime de charme.
```

## 22. Cible 8 — `F2` et `S2` Les moments difficiles

```
Deux écrans de la même famille, à concevoir ensemble pour garantir leur cohérence
de registre. Applique intégralement le registre sobre.

────────────────────────────────────────
F2 — DOCUMENT FISCAL INDÉTERMINÉ

Contexte technique à comprendre avant de dessiner : la plateforme fiscale
ivoirienne valide chaque facture avant qu'elle puisse être remise au client. Sur un
timeout réseau, il est IMPOSSIBLE de savoir si la facture a été validée. La renvoyer
produirait une double validation et consommerait un jeton payant en double.

Il n'existe donc aucune solution automatique. Un humain doit aller vérifier dans
l'espace fiscal de l'établissement et trancher.

Persona : Adjoua, qui n'a aucune notion technique.

Le défi : expliquer une situation techniquement subtile à quelqu'un qui doit agir
correctement du premier coup, sans jargon, sans l'effrayer, et sans qu'elle puisse
cliquer sur « réessayer » par réflexe.

Contenu : ce qui s'est passé en langage ordinaire ; ce qu'elle doit aller vérifier
et où ; les deux issues possibles et leur conséquence ; une action pour chacune,
visuellement bien différenciées. AUCUN bouton « réessayer » atteignable par
inadvertance.

Variantes : un document indéterminé, une liste de cinq, un document résolu.

────────────────────────────────────────
S2 — RÉCONCILIATION D'UNE ÉCRITURE ORPHELINE

Le scénario : Aminata sert une bière à 21 h 40 au client de la chambre B3, sans
réseau. À 21 h 55, Adjoua fait le check-out de B3 et la facture part aux impôts.
À 22 h 10, le réseau revient et la bière arrive sur un séjour qui n'existe plus.

C'est le conflit le plus fréquent en exploitation réelle et il n'a AUCUNE solution
automatique. Un rejet silencieux perd de l'argent ; un ajout d'office fausse une
facture déjà validée.

Adjoua doit trancher entre trois issues :
(a) émettre un avoir et refacturer — mais l'avoir fiscal se fait PAR QUANTITÉ, pas
    par montant, ce qui impose d'annuler la ligne entière ;
(b) prendre la consommation en charge sur l'établissement ;
(c) la rattacher au prochain séjour du même client.

Le défi : elle doit comprendre en quelques secondes ce qui s'est passé, voir le
montant en jeu, et choisir en connaissance de cause. AIDE-LA À DÉCIDER — la bonne
réponse dépend le plus souvent du montant et de la relation client.

Contenu : le contexte (qui, quoi, quand, quel séjour, quel montant), les trois
options avec leur conséquence concrète, et pour l'option (a) la manipulation
complète guidée pas à pas.

Variantes : un cas isolé de 1 500 F, un cas de 45 000 F, une liste de six cas
accumulés après une panne réseau d'une journée.

────────────────────────────────────────
Montre-moi comment ces deux écrans restent visuellement cohérents avec le reste du
produit tout en changeant complètement de registre.

MOTIF POSÉ : le registre sobre des moments difficiles. Cinq autres écrans en
hériteront.
```

## 23. Cible 9 — `Q1` Page publique de commande

```
Écran : la page web que voit le client après avoir scanné le QR de sa table.

Contexte : téléphone personnel du client, connexion mobile parfois faible, PAS
D'APPLICATION À INSTALLER, PAS DE COMPTE À CRÉER.

AUCUNE DONNÉE PERSONNELLE N'EST DEMANDÉE — ni téléphone, ni email, ni nom. C'est
une page publique anonyme.

Contenu : catalogue du point de vente, panier, validation. Puis un écran d'attente
expliquant que le serveur va confirmer la commande — car rien ne part en cuisine
avant qu'Aminata ait constaté la présence du client à la table.

CETTE PAGE EST LA SEULE SURFACE DU PRODUIT VUE PAR UN CLIENT FINAL. Elle doit être
agréable et rapide, sans devenir une application. Elle obéit à des règles
différentes du reste : pas de sélecteur de contexte, pas d'indicateur de
synchronisation, pas de navigation composable.

Variantes : catalogue, panier, envoyé et en attente de confirmation, confirmé,
table fermée ou QR expiré.

MOTIF POSÉ : la surface client — règles entièrement distinctes du produit interne.
```

## 24. Cible 10 — `G2` Formules et barèmes

```
Écran : configuration de l'offre d'hébergement.

C'est ici que se paramètrent les quatre familles de formules : nuitée, PASSAGE
horaire à paliers dégressifs, DEMI-JOURNÉE en plages fixes, mensuel.

Le barème de passage est une table de paliers — 1 h : 1 500, 2 h : 2 800,
3 h : 4 000, 4 h : 5 000, heure supplémentaire : +1 200 — plus une règle de bascule
en nuitée au-delà d'un seuil.

Chaque formule porte aussi son traitement fiscal : est-elle soumise à la taxe de
séjour, et selon quelle règle. CE PARAMÈTRE EST SENSIBLE — mal réglé, il met le
client en infraction. Traite-le avec un niveau d'avertissement approprié, sans
pour autant transformer l'écran en champ de mines.

Cet écran est en zone de charme : il est consulté rarement, par quelqu'un qui prend
son temps. Il peut être soigné et pédagogique. Un aperçu de l'effet de chaque
réglage serait précieux.

Variantes : configuration d'un hôtel complet, configuration d'une résidence
meublée de 4 unités, création d'une formule, paramètre fiscal non renseigné.

MOTIF POSÉ : la configuration structurée avec paramètre sensible. Sept autres
écrans en hériteront.
```

---

# PARTIE V — MATRICE DE DÉRIVATION

## 25. Les 30 écrans codés sans maquette

> 📦 **Déplacé le 2026-07-30 vers `docs/design/derivation.md`, qui fait désormais foi.**
> Le tableau n'est pas dupliqué ici : deux copies divergeraient, ce que le principe I de la
> constitution interdit. `derivation.md` est le chemin que citent les prompts Spec Kit, la
> Definition of Done et la porte P-19.

**Ce que le fichier contient** : les 30 écrans non maquettés, chacun avec le motif dont il
hérite et ce qui change ; le décompte des 41 écrans du produit (11 maquettés en 29 fichiers
d'états, 30 dérivés) ; et la règle opposable — un écran absent des deux cas ne se code pas,
il part en maquettage.

---

# PARTIE VI — PROTOTYPES ANIMÉS

## 26. Les six prototypes

Claude Design produit surtout des cibles statiques. Pour juger une sensation, il faut du HTML animé.

```
Produis-moi un prototype HTML animé, autonome et ouvrable dans un navigateur, pour
chacun de ces moments. Je veux juger la SENSATION, pas la description.

1. Les sept patrons de mouvement du système, sur une page de démonstration avec un
   curseur de vitesse pour que je puisse évaluer les durées.

2. Le sélecteur de durée du passage — 1 h, 2 h, 3 h, 4 h — avec son retour tactile
   et l'inscription de l'heure de fin. C'est le geste le plus répété du produit.

3. L'ajout d'une consommation à une note, avec le total qui monte. Enchaîne-en cinq
   à la suite pour que je vérifie que ça reste agréable et jamais lassant.

4. La clôture réussie, dans son intégralité — le moment de plaisir n°1.

5. Le retour du réseau et la vidange de la file d'attente, avec le compteur qui
   descend de 23 à 0.

6. L'indicateur de synchronisation dans ses trois états, en boucle, pour vérifier
   qu'il ne devient pas fatigant après huit heures à l'écran.

Chaque prototype doit fonctionner en clair et en sombre, et respecter la préférence
« animations réduites ».

Déclare toutes les durées et courbes en VARIABLES CSS nommées en tête de fichier
(--duree-rapide, --courbe-entree, etc.) plutôt qu'en valeurs littérales dispersées.
C'est de ces variables que je tirerai docs/design/mouvement.md, et c'est ce que
l'agent d'implémentation lira pour reproduire la sensation exacte.
```

---

# PARTIE VII — DOCUMENTS IMPRIMÉS

## 27. Les sept modèles

> Ce ne sont pas des écrans mais des spécifications de mise en page. À produire en S8, avant le cycle IMP.

```
Produis les maquettes de sept documents imprimés.

CONTRAINTE THERMIQUE (D1 à D5) : 80 mm de large, environ 42 caractères par ligne,
PAS DE COULEUR, pas de nuance de gris fiable, pas d'image autre qu'un logo
monochrome simple. Les caractères accentués français doivent être lisibles. Prévois
la coupe papier.

D1 — Ticket de commande client (point de vente)
D2 — Bon de préparation cuisine ou bar
D3 — Bon de dépôt pressing, avec numéro de retrait bien visible
D4 — Reçu d'encaissement
D5 — Rapport de fin de shift

CONTRAINTE PDF A4 (D6 et D7) :

D6 — NOTE PROVISOIRE DE SÉJOUR. MENTION OBLIGATOIRE ET VISIBLE : « Document non
fiscal — ne tient pas lieu de facture ». Cette mention protège juridiquement le
client ; elle ne doit pas être en petits caractères en bas de page.

D7 — FACTURE FISCALE. Le document le plus contraint du produit. Doivent y figurer :
identification complète de l'établissement et du client, désignation détaillée des
prestations, prix hors taxes, TVA à 18 %, TAXE COMMUNALE DE NUITÉE EN LIGNE
DISTINCTE séparée du hors-taxes et de la TVA (obligation légale), taxe de
développement touristique, total. Plus les éléments renvoyés par l'administration
après validation : numéro normalisé, visuel officiel, QR code de vérification et
sceau électronique.

Réserve un emplacement dimensionné pour ces éléments officiels — je te donnerai les
spécimens exacts après la réponse de l'administration. DIS-MOI QUELLES DIMENSIONS
TU AS RÉSERVÉES pour que je vérifie la compatibilité.

Tous les documents portent le branding de l'établissement : logo, coordonnées,
mentions légales, paramétrables par client.
```

---

# PARTIE VIII — VALIDATION ET GARDE-FOUS

## 28. Protocole de validation terrain

À faire à l'atelier d'Abengourou, en phase 0. Les maquettes ne valent que confrontées au réel.

| Étape | Méthode |
|---|---|
| **Préparation** | Imprimer les 10 cibles en A3, et les charger sur le matériel réel — poste de réception, Android d'entrée de gamme |
| **Direction visuelle** | Montrer les deux planches d'ambiance à Adjoua, Yao et Aminata. Leur préférence prime sur la mienne |
| **Test du passage (`R4`)** | Yao simule trois enregistrements. **Chronométrer.** Si le troisième dépasse 45 s, l'écran est à refaire |
| **Test de la clôture (`C4`)** | Adjoua déroule sa clôture d'hier sur la maquette. Noter chaque hésitation |
| **Test du blocage** | Lui montrer la clôture bloquée sans explication préalable. Comprend-elle quoi faire ? |
| **Test du maquis (`R1`)** | Montrer la quatrième variante à quelqu'un qui n'est pas hôtelier. A-t-il l'impression d'un produit conçu pour lui ? |
| **Test de la commande (`P2`)** | Aminata saisit six lignes debout, à une main. Chronométrer |
| **Test des écrans difficiles (`F2`, `S2`)** | Les montrer sans contexte. Sait-elle quoi faire ? Sinon c'est le TEXTE qu'il faut reprendre, pas la mise en page |
| **Test du cahier** | Sur trois tâches courantes, comparer logiciel et cahier. Le cahier ne doit jamais gagner |
| **Test des huit heures** | Prototypes 2, 3 et 6 en boucle dix minutes |

Tout retour va dans `docs/design/notes-terrain.md`. **Un retour qui contredit le cadrage modifie le cadrage**, pas l'inverse — c'est exactement pour ça qu'on maquette avant de coder.

## 29. Garde-fous

- **Le budget d'animation est celui d'un Android à 2 Go de RAM.** Toute animation qui ne tient pas en `transform` et `opacity` est refusée. À vérifier sur le matériel réel, pas sur un émulateur.
- **Aucune animation ne conditionne un état.** L'état change, l'animation suit. Si le rendu est interrompu, la donnée est déjà juste.
- **Aucune animation sur un chemin fréquent ne dépasse 400 ms.**
- **La préférence « réduire les animations » est respectée partout.** Dans ce mode, rien ne casse et rien ne manque.
- **Le test des huit heures avant validation.** L'indicateur de synchronisation, les squelettes et les micro-interactions du sélecteur de durée sont vus des centaines de fois par jour. Ce qui charme à la dixième vue peut exaspérer à la trois-centième.
- **Le jugement final revient au terrain**, pas au designer ni au fondateur.

## 30. Règles de conduite

- **Un écran sans maquette ni motif d'héritage déclaré n'est pas codé.** Les prompts Spec Kit référencent `docs/design/html/{code}.html` ou une ligne de `docs/design/derivation.md` ; sans l'un ou l'autre, le cycle ne démarre pas.
- **Le HTML de `docs/design/html/` ne migre jamais vers `app/`.** Aucun `git mv`, aucun copier-coller de bloc. On lit les valeurs, on réimplémente en composants Nuxt avec i18n, mode sombre, RBAC et chargement paresseux — toutes choses que l'export ne contient pas.
- **Une seule extraction de tokens.** `tokens.md` est produit au moment de la fondation, à partir de `styleguide.html`. Ensuite il prime : un export qui diverge est corrigé, pas suivi.
- **Le code stable prime sur le nom.** `R4` restera `R4` même refait trois fois.
- **Si une maquette révèle une contradiction avec le cadrage**, mettre à jour `docs/cadrage-v1.md` ou `docs/user-stories-v1.md` d'abord, puis la maquette.
- **`tokens.md` est la seule source des valeurs visuelles.** Aucune couleur ni espacement littéral ailleurs, ni dans les maquettes ni dans le code.
- **`lexique.md` est la seule source du vocabulaire utilisateur.** Tout nouveau terme y entre avant d'être codé.
- **Ne pas maquetter au-delà du besoin.** Un écran maquetté six mois avant d'être codé sera refait.

---

# PARTIE IX — EXPORT VERS LE PROJET

## 30 bis. Correspondance avec les maquettes produites

Claude Design a nommé les maquettes selon sa propre logique. Les codes de ce document restent la référence des prompts Spec Kit et de la matrice de dérivation. **Convention de nommage de fichier retenue : `{code}-{nom-lisible}.html`** — le code pour la référence stable, le nom pour la lecture humaine.

### Écrans du produit

| Maquette produite | Code | Ce qu'elle pose |
|---|---|---|
| Kaya — Accueil | `R1` | Composition par permissions et modules |
| Kaya — Mes établissements | `M4` | Consultation mobile, zone de charme |
| Kaya — Le planning | `V1` | Visualisation temporelle à granularité fine |
| Kaya — Passage | `R4` | Parcours court, décision unique |
| Kaya — L'offre d'hébergement | `G2` | Configuration structurée, paramètre sensible |
| Kaya — La page du client | `Q1` | Surface client, règles distinctes |
| Kaya — Saisie de commande | `P2` | Saisie répétitive hors ligne, une main |
| Kaya — La note et le départ | `R7` | Document à lignes : lignes, taxes, total, action finale |
| Kaya — Clôture de la journée | `C4` | Vérification préalable et blocage constructif |
| Kaya — Registre grave | `F2` + `S2` | Registre sobre des moments difficiles |

### Fondations, prototypes, documents

| Maquette produite | Rôle | Fichier |
|---|---|---|
| Kaya — Directions visuelles | Identité, palettes, typographie | `fondation-directions.html` |
| Kaya — Système de mouvement | Source de `mouvement.md` | `fondation-mouvement.html` |
| Kaya — Moments de plaisir | Les 8 moments expressifs | `fondation-plaisir.html` |
| Kaya — Moments difficiles | Le registre sobre | `fondation-difficiles.html` |
| Kaya — Illustrations | États vides, onboarding | `illustrations.html` |
| Prototypes animés (sommaire) | Index | `proto-0-sommaire.html` |
| Proto 1 → 6 | Sensation de mouvement | `proto-1-patrons.html` … `proto-6-indicateur-sync.html` |
| D1-D5 — Tickets thermiques | Impression 80 mm | `D1-D5-tickets-thermiques.html` |
| D6 — Note provisoire | Document non fiscal | `D6-note-provisoire.html` |
| D7 — Facture fiscale | Document légal | `D7-facture-fiscale.html` |

> ⚠️ **Point à vérifier avant l'export : où sont les 14 composants canoniques ?**
> Ils sont peut-être inclus dans « Directions visuelles », ou pas produits du tout. C'est de ce fichier que sortent `theme.css` et `tokens.md` — sans lui, l'extraction des tokens se fait par glanage dans dix écrans, et les valeurs divergeront. **Si `styleguide.html` n'existe pas, le produire avant l'export.**

---

## 31. Prompt d'export final

À exécuter dans Claude Design **une fois les 10 cibles et les prototypes terminés**. Il produit l'archive à décompresser dans `docs/design/` du dépôt.

```
Package tout le travail de design de Kaya dans une archive .zip que je vais
décompresser dans le dossier docs/design/ de mon projet.

STRUCTURE ATTENDUE — respecte-la exactement, mes prompts de développement
référencent ces chemins :

kaya-design/
├── README.md              Ce que contient l'archive, et surtout : ce qui se copie
│                          dans le projet (theme.css SEULEMENT) et ce qui ne se
│                          copie jamais (tout le reste — référence à lire)
├── theme.css              LE bloc @theme Tailwind 4 complet : couleurs clair et
│                          sombre, espacements, rayons, polices, durées, courbes.
│                          C'est le SEUL fichier que je copierai tel quel dans mon
│                          projet Nuxt. Il doit fonctionner seul, sans retouche.
├── tokens.md              Les mêmes valeurs en tableau lisible, avec le nom du
│                          token, sa valeur claire, sa valeur sombre, son usage
├── mouvement.md           Durées, courbes et les sept patrons de mouvement, avec
│                          les noms de variables CSS correspondants
├── composants.md          Les 14 composants canoniques : rôle, états, classes
│                          Tailwind employées, règles d'usage
├── styleguide.html        Les 14 composants dans tous leurs états, clair + sombre
├── fondation/
│   ├── fondation-directions.html      (« Directions visuelles »)
│   ├── fondation-mouvement.html       (« Système de mouvement »)
│   ├── fondation-plaisir.html         (« Moments de plaisir »)
│   ├── fondation-difficiles.html      (« Moments difficiles »)
│   └── illustrations.html             (« Illustrations »)
├── html/                  Un fichier par écran ET PAR ÉTAT.
│   │                      Nommage : {code}-{nom-lisible}[-{etat}].html
│   ├── R1-accueil.html            R1-accueil-maquis.html
│   │   R1-accueil-serveuse.html   R1-accueil-proprietaire.html
│   ├── R4-passage.html            R4-passage-complet.html
│   │   R4-passage-connu.html      R4-passage-hors-ligne.html
│   ├── P2-saisie-commande.html    P2-saisie-commande-hors-ligne.html
│   │   P2-saisie-commande-desktop.html
│   ├── R7-note-depart.html        R7-note-depart-envoi.html
│   │   R7-note-depart-echec.html
│   ├── C4-cloture.html            C4-cloture-bloquee.html
│   │   C4-cloture-reussie.html
│   ├── V1-planning.html           V1-planning-dense.html
│   ├── M4-mes-etablissements.html M4-mes-etablissements-alerte.html
│   ├── F2-registre-grave.html     S2-registre-grave.html
│   ├── Q1-page-client.html        Q1-page-client-panier.html
│   │   Q1-page-client-attente.html
│   └── G2-offre-hebergement.html  G2-offre-hebergement-residence.html
├── documents/
│   ├── D1-D5-tickets-thermiques.html
│   ├── D6-note-provisoire.html
│   └── D7-facture-fiscale.html
├── proto/                 proto-0-sommaire.html à proto-6-indicateur-sync.html
└── png/                   Captures de chaque écran, clair et sombre — pour ma
                           revue et pour l'impression A3 de l'atelier terrain

EXIGENCES SUR LE CONTENU :

1. TAILWIND 4, PAS CSS. Chaque fichier HTML utilise des utilitaires Tailwind du
   noyau référençant les tokens de theme.css. Mode sombre par la variante dark:.
   Aucune classe personnalisée, aucun style en ligne. Le CSS explicite est
   regroupé en fin de fichier, commenté, et limité à ce que Tailwind n'exprime
   pas — @keyframes, styles d'impression thermique.

2. theme.css DOIT ÊTRE AUTOSUFFISANT. Je le copie dans mon projet et il marche.
   Pas de dépendance à un fichier de la maquette, pas de valeur laissée en dur
   ailleurs. Si un token manque, l'écran correspondant est incohérent — vérifie.

3. VALEURS ARBITRAIRES : liste dans README.md toutes les valeurs arbitraires
   employées (w-[347px], text-[#3a3a3a]…) avec l'écran concerné. Chacune est une
   décision en attente : soit elle entre dans @theme, soit on s'aligne sur
   l'échelle. Ne les masque pas.

4. CHAQUE ÉTAT EST UN FICHIER. Un HTML ne montre qu'un état ; ce que les captures
   rendaient évident par leur nombre doit rester explicite ici. Si un écran a
   quatre états, il a quatre fichiers. Le préfixe de code (R1, R4, P2…) est
   OBLIGATOIRE dans le nom : mes prompts de développement le référencent.

4 bis. LE STYLEGUIDE. Si les 14 composants canoniques ne sont pas déjà dans un
   fichier dédié, PRODUIS-LE avant l'export : styleguide.html, les 14 composants
   dans tous leurs états, clair et sombre, écrit en Tailwind 4 avec le bloc
   @theme complet. C'est de ce fichier que je tire theme.css et tokens.md — sans
   lui je devrais glaner les valeurs dans dix écrans, et elles divergeraient.
   « Kaya — Registre grave » couvre deux écrans distincts (F2 document fiscal
   indéterminé, S2 réconciliation d'une écriture orpheline) : livre-les en deux
   fichiers séparés.

5. LES PROTOTYPES déclarent leurs durées et courbes en variables CSS nommées,
   reprises de theme.css. C'est de là que sort mouvement.md.

6. README.md doit répondre à trois questions en tête de fichier :
   - qu'est-ce que je copie dans mon projet ? (theme.css, et rien d'autre)
   - qu'est-ce que je lis sans jamais le copier ? (tout le HTML)
   - quelles décisions restent à prendre ? (les valeurs arbitraires listées)

Si tu ne peux pas produire d'archive .zip, livre-moi les fichiers un par un en
respectant strictement cette arborescence dans leurs noms, et dis-le-moi.
```

## 32. Checklist avant le cycle 1

La maquette est déposée. Il reste six actions, dont trois bloquantes.

### Bloquant — ✅ les trois actions sont faites au 2026-07-30

| # | Action | État |
|---|---|---|
| 1 | `docs/design/derivation.md` — les 30 écrans et le motif dont chacun hérite | ✅ **fait** — la partie V y a été **déplacée**, pas recopiée : une seule source |
| 2 | `docs/design/lexique.md` — la traduction des concepts techniques | ✅ **fait** — le §6 y a été **déplacé**, pas recopié |
| 3 | `docs/Kaya_Design.md` déposé au dépôt | ✅ **fait** — 1 266 lignes, présent |

> Les deux tableaux ont été **déplacés et non dupliqués** : le document annonçait « recopier »,
> mais deux copies divergeraient, ce que le principe I de la constitution interdit. Ce fichier
> renvoie désormais vers eux (§6 et partie V).

### Non bloquant, mais à faire tôt

| # | Action | Pourquoi |
|---|---|---|
| 4 | **Vérifier `styleguide.html` dans un projet Nuxt 4 réel**, avec `theme.css` importé | Si un utilitaire manque, il venait du CDN et pas du noyau Tailwind 4. Le découvrir maintenant coûte une heure, au cycle 8 une refonte |
| 5 | **Trancher les valeurs arbitraires** listées dans `README.md` | Chacune entre dans `@theme` ou s'aligne sur l'échelle. Les laisser dispersées, c'est quarante valeurs uniques au cycle 8 |
| 6 | **Produire `Q1-page-client-ferme.html`** — table fermée ou QR expiré | État réel et spécifié : c'est ce que voit quelqu'un qui scanne un QR arraché ou photographié. Sans lui, le cycle 15 devra l'inventer |

### À l'atelier d'Abengourou

Capturer les 10 écrans principaux en PNG pour l'impression A3 — les HTML ne se montrent pas à Adjoua sur un écran de portable. Créer `docs/design/notes-terrain.md` et y consigner tous les retours.

`theme.css` reste dans `docs/design/` et n'est copié vers `app/assets/css/` **qu'au cycle 1**, quand `app/` existe.
