# Kaya — Vision plateforme multi-activités

*Document de réflexion — à creuser après la mise en production du pilote hôtelier*
*Version 0.1 — Ne modifie pas le périmètre du MVP. Contient une liste d'amendements chirurgicaux à appliquer maintenant.*

---

## 1. Verdict

**Oui, l'architecture peut porter ces verticales. À une condition précise :**

> Le socle ne doit jamais connaître le mot « chambre ». Aujourd'hui il le connaît encore par endroits.

L'extension est possible parce que la part réellement partagée est grande — de l'ordre de **60 à 70 % du code** — et parce qu'elle contient l'actif le plus coûteux à reconstruire : la conformité fiscale.

Elle est dangereuse parce que trois de ces verticales exigent une **profondeur de gestion de stock** qui n'a rien à voir avec celle d'un bar d'hôtel, et que le confondre produirait un produit médiocre partout.

Ce document sépare ce qui est vrai de ce qui est séduisant.

---

## 2. Le modèle à trois couches

C'est la structure qui rend la chose gouvernable. La confusion entre ces trois couches est ce qui tue les plateformes de ce type.

```
┌─────────────────────────────────────────────────────────────┐
│  VERTICALES — minces, spécifiques, remplaçables             │
│  hôtellerie · restauration · bar · pressing · boulangerie   │
│  quincaillerie · supérette · boutique · livraison           │
│  → écrans propres, règles propres, vocabulaire propre       │
├─────────────────────────────────────────────────────────────┤
│  CAPACITÉS TRANSVERSES — consommées par plusieurs verticales│
│  stock (4 profondeurs) · production · livraison · commerce  │
│  en ligne · fidélité · comptes clients · devis              │
├─────────────────────────────────────────────────────────────┤
│  SOCLE — universel, jamais spécialisé                       │
│  tenant · établissement · comptes & RBAC · appareils        │
│  audit · configuration héritée · JURIDICTION & FISCALITÉ    │
│  caisse & encaissements · catalogue · vente & document      │
│  synchronisation hors-ligne · pilotage consolidé · abonnement│
└─────────────────────────────────────────────────────────────┘
```

**L'erreur à ne jamais commettre** : faire de « boulangerie » une verticale qui réimplémente le stock. Une boulangerie, c'est le socle + la capacité stock au profil 2 + la capacité production + une verticale mince.

**Le test de la couche** : si deux verticales en auraient besoin, ce n'est pas une verticale — c'est une capacité. Si toutes en auraient besoin, c'est le socle.

---

## 3. Ce qui est réellement partagé

| Brique | Universelle ? | Commentaire |
|---|---|---|
| Tenant, établissement, modules | ✅ Totalement | Déjà conçu ainsi |
| Comptes, rôles cumulables, appareils | ✅ Totalement | Un caissier de supérette cumule les mêmes rôles |
| **Fiscalité et juridiction** | ✅ **Totalement** | **L'actif le plus précieux.** FNE, TVA, avoirs, file de certification, coffre à clés, écran de rapprochement — identiques pour une quincaillerie |
| Caisse, shifts, encaissements, clôture | ✅ Totalement | Le fractionnement multi-modes est encore plus fréquent en commerce |
| Catalogue et prix | ✅ Structure | La profondeur diffère (variantes, codes-barres) |
| Vente, lignes, documents | ✅ Totalement | Une addition de bar et un ticket de supérette sont le même agrégat |
| Synchronisation hors-ligne | ✅ Totalement | Une supérette sans réseau doit vendre. Même contrainte, enjeu supérieur |
| Journal d'audit | ✅ Totalement | **Argument de vente n°1 en commerce de détail** — le vol au comptoir est la première douleur d'un propriétaire de quincaillerie |
| Pilotage consolidé multi-établissements | ✅ Totalement | C'est précisément ta proposition de valeur |
| Abonnements | ⚠️ À abstraire | Voir amendement A11 |
| Stocks | ⚠️ Structure oui, profondeur non | Voir §5 |
| Unités louables, formules, disponibilité | ❌ Hôtellerie seule | Doit rester dans la verticale |

**Conclusion** : le socle et les capacités représentent l'essentiel de l'effort déjà engagé. Chaque verticale supplémentaire est mince — *à condition* que la capacité qu'elle exige existe déjà.

---

## 4. Les quatre fractures réelles

### 4.1 La profondeur de stock — la fracture principale

Le bar de Deloria suit une trentaine de références. Une quincaillerie en a **5 000 à 20 000**, avec des variantes (vis M6×40, M6×50, M8×40…), plusieurs unités de mesure sur le même article (à l'unité, au mètre, au kilo, au carton), de la découpe, des commandes fournisseurs, de la valorisation, des codes-barres.

**Ce n'est pas « le même stock avec plus de lignes ».** C'est un autre produit. Le MVP exclut explicitement la valorisation, les commandes fournisseurs et la réception partielle — exclusion parfaitement saine pour un hôtel, rédhibitoire pour du commerce.

### 4.2 La nature de l'article

| Verticale | Ce que l'article exige en plus |
|---|---|
| Hôtellerie, bar | Rien — c'est le cas simple |
| Boulangerie | Production : de la farine devient 200 baguettes. Invendus en fin de journée |
| Quincaillerie | Variantes, unités multiples, découpe, conversions d'emballage |
| Supérette, supermarché | Codes-barres, pesée, dates limites, lots, promotions |
| Boutique en ligne | Variantes taille/couleur, photos multiples, expédition |

Ce sont des **modèles de domaine différents autour de l'article**, pas des catalogues différents.

### 4.3 La vitesse de passage en caisse

Un check-out d'hôtel dure deux minutes et c'est acceptable. Un passage en caisse de supérette doit durer **quinze secondes avec une file derrière**. Ça impose lecteur code-barres, balance connectée, raccourcis clavier, et une tolérance zéro à la latence. C'est une contrainte d'ingénierie, pas d'interface.

### 4.4 La boutique en ligne n'est pas la même affaire

C'est le point où je te contredis franchement.

Les vendeuses et vendeurs qui écoulent sur TikTok et Instagram forment un marché **entièrement différent** du tien : informels, très jeunes, revenu irrégulier, commandes arrivant par messages privés, paiement à la livraison, aucune adresse formelle, aucun établissement physique au sens de ton modèle. Ils ne peuvent pas payer 20 000 FCFA par mois — et ils changent d'outil tous les six mois.

Ce qu'ils partagent avec Kaya : le moteur fiscal (précisément l'aide à la formalisation, qui est une vraie idée), le catalogue, le stock simple, la livraison.
Ce qu'ils ne partagent pas : établissement, caisse, shifts, clôture, tout le pilotage.

> ### ✅ DÉCISION ACTÉE — 28 juillet 2026
>
> **La boutique en ligne n'est pas une verticale de Kaya. C'est un produit distinct, bâti sur le même socle.**
>
> Elle aura sa propre marque, sa propre mise sur le marché, son propre modèle de prix (gratuit ou commission — jamais 20 000 FCFA/mois), et sa propre équipe le jour venu.
>
> **Ce qu'elle réutilise** : le socle complet (tenant, comptes, fiscalité, documents, synchronisation), la capacité stock au profil `SIMPLE`, la capacité livraison, la capacité commerce en ligne.
> **Ce qu'elle ne réutilise pas** : établissement au sens physique, caisse, shifts, clôture, pilotage multi-établissements — c'est-à-dire l'essentiel de ce qui fait Kaya.
>
> **Elle n'est pas évoquée dans les documents du MVP, ni dans un argumentaire commercial Kaya.** Décision de lancement à prendre après la levée, sur données de marché, jamais avant.
>
> *Raison d'être de cette décision écrite* : c'est la verticale la plus séduisante à construire et la moins rentable à vendre. Elle présente le plus grand risque de détournement d'attention du projet. La consigner ici évite d'avoir à la retrancher plus tard.

---

## 5. Les quatre profils de stock

C'est la décision de conception la plus importante de ce document.

| Profil | Contenu | Verticales | Effort |
|---|---|---|---|
| **0 — Aucun** | Pas de suivi | Pressing, salle de réunion, services | — |
| **1 — Simple** | Articles, entrées, sorties sur vente, inventaire, alertes de seuil | Bar d'hôtel, maquis, restaurant | **MVP actuel** |
| **2 — Valorisé** | + coût moyen pondéré, commandes fournisseurs, réceptions, unités de mesure multiples, marge | Boulangerie, petite supérette | +8 sem. |
| **3 — Détaillé** | + variantes, codes-barres, lots et dates limites, promotions, découpe, transferts inter-sites | Quincaillerie, supermarché | +10 sem. après le profil 2 |

**Le profil est une propriété du module d'activité, pas du produit.** Un même tenant peut avoir un hôtel en profil 1 et une quincaillerie en profil 3.

**Ce qui doit être fait maintenant** : le modèle de données du profil 1 doit déjà porter les colonnes que les profils 2 et 3 rempliront. Trois colonnes nullables aujourd'hui évitent une migration de toutes les données de stock demain. Voir amendements A2, A3, A4.

---

## 6. Analyse verticale par verticale

| Verticale | Adhérence au socle | Capacités requises | Effort après socle | Verdict |
|---|---|---|---|---|
| **Boulangerie** | Élevée | Stock profil 2 + **production** | 10–12 sem. | ✅ Bon candidat. La production est une capacité neuve mais bien délimitée : recette, fournée, invendus |
| **Quincaillerie** | Moyenne | Stock profil 3 + **devis/proforma** + comptes clients | 14–16 sem. | ✅ Fort potentiel commercial. Le devis est central : un client de quincaillerie demande un devis avant d'acheter. L'audit anti-vol est l'argument de vente |
| **Supérette** | Élevée | Stock profil 2 ou 3 + **caisse rapide** (code-barres, balance) | 10–12 sem. | ✅ Bon candidat. La contrainte est la vitesse de passage, pas la fonctionnalité |
| **Supermarché** | Élevée | Supérette + multi-caisse + fidélité | +6–8 sem. sur supérette | ⚠️ **Pas une verticale distincte** — c'est une supérette avec un curseur de taille. Ne la traite jamais comme un produit à part |
| **Boutique en ligne / réseaux sociaux** | Faible | Commerce en ligne + livraison + accompagnement à la formalisation | 12–14 sem. | ⚠️ **Produit distinct**, même socle, autre marché, autre prix. Voir §4.4 |
| **Livraison** | — | — | 8 sem. (capacité) + 4 (verticale autonome) | ⚠️ **C'est une capacité, pas une verticale.** Boulangerie, supérette et boutique la consomment. Une société de livraison pure est une verticale mince par-dessus |
| **Gérant de parc** *(massa, car, taxi, VTC)* | **Élevée** | Documents réglementaires · suivi de flotte en option | **8–10 sem.** | ✅ **Meilleur candidat après la supérette.** Même persona que le propriétaire d'hôtel. Couvre transport interurbain **et** taxi/VTC : deux modèles de recette, une seule verticale |
| **Compagnie de transport** | Élevée | **Billetterie** + suivi de flotte + comptes clients | 14–18 sem. | ✅ ARPU supérieur. S'ajuste mieux au moteur de disponibilité que la gare |
| **Gare routière** *(car, massa, badjan)* | Moyenne | Billetterie + livraison (colis) + comptes clients | 12–14 sem. | ⚠️ Excellent ajustement technique, ajustement de marché difficile. Jamais en premier. Voir §14.1 |

**Ordre recommandé, si la levée réussit et qu'une équipe existe** :
`supérette` (proche du socle, marché large) → `gérant de parc` (effort faible, même persona que l'hôtelier, couvre taxi/VTC et interurbain) → `boulangerie` (production, marché nombreux) → `quincaillerie` (stock profil 3, ARPU supérieur) → `compagnie de transport` (billetterie, gros ARPU) → `livraison` (capacité à demi-construite par la conciergerie) → `gare routière` (selon l'accès terrain) → `boutique en ligne` (produit distinct, à réévaluer entièrement).

---

## 7. La proposition de valeur multi-business

**Elle est réelle et elle est différenciante.** En Afrique de l'Ouest, une personne qui possède un hôtel possède souvent aussi une supérette, une quincaillerie ou une société de transport. Ce n'est pas un cas d'usage inventé, c'est un profil d'investisseur courant.

Ce qu'aucun concurrent ne propose : **une vue consolidée sur des activités hétérogènes**. Un tableau de bord unique qui dit « ton hôtel a fait 340 000 hier, ta supérette 180 000, ta quincaillerie 95 000 », avec les écarts de caisse des trois et le journal d'audit des trois.

C'est un produit qui n'existe pas, et le besoin est authentique : ce propriétaire délègue à trois gérants qu'il ne peut pas surveiller simultanément.

**Trois conditions pour que ça marche :**

1. **Le tableau de bord consolidé doit comparer des grandeurs comparables.** Un hôtel se mesure en taux d'occupation, une supérette en panier moyen. La couche de pilotage doit exposer des indicateurs universels — chiffre d'affaires, marge, écart de caisse, alertes — et laisser les indicateurs spécifiques dans chaque verticale.

2. **La tarification doit changer de métrique.** Voir amendement A11 : facturer « à la chambre » n'a aucun sens pour une quincaillerie.

3. **La cohérence de vocabulaire.** Le propriétaire qui passe de son hôtel à sa quincaillerie dans la même application doit retrouver la même grammaire. C'est ce que garantit la discipline des couches.

---

## 8. Amendements — ✅ APPLIQUÉS le 28 juillet 2026

> Onze changements. Coût aujourd'hui : quelques heures. Coût si repoussés : une migration de données ou une refonte de crate.
> **Aucun n'ajoute de fonctionnalité au MVP.**
>
> **Ces onze amendements sont intégrés dans `cadrage-v1.md`, `user-stories-v1.md` et `Kaya_Prompts_SpecKit.md`.** La section ci-dessous est conservée comme trace de la décision et de sa justification.
>
> Les deux plus critiques, à ne jamais laisser régresser : **A2** (quantités décimales — sinon migration de toutes les lignes de vente et de stock) et **A8** (règle de CI « aucun crate du socle ne dépend d'une verticale » — sans elle, tout ce document devient théorique en trois cycles).

### A1 — Vocabulaire du socle

Le socle ne connaît ni « chambre », ni « unité louable », ni « séjour ». Il connaît `article_vendable` et `ressource_reservable`. `unite_louable`, `formule` et `sejour` deviennent des spécialisations de la verticale hébergement.

*Fichiers : cadrage §4, §6.3 · user stories ETB, HEB · constitution principe 2.*

### A2 — Quantités décimales

`quantite` en `NUMERIC`, jamais en entier. Un hôtel vend 1 bière ; une quincaillerie vend 2,3 mètres de fer à béton ; une boulangerie achète 47,5 kg de farine. Passer d'entier à décimal après mise en production, c'est migrer toutes les lignes de vente et de stock.

*Fichiers : user stories PDV-01, STK-02 · constitution principe 5.*

### A3 — Unité de mesure sur l'article

Colonne `unite_mesure` obligatoire sur `article`, valeur par défaut `unite`. Prévoir dès le modèle qu'un article puisse avoir plusieurs unités avec facteurs de conversion — **table créée, non exploitée au MVP**.

*Fichiers : user stories PDV-01, STK-01.*

### A4 — Coût unitaire sur le mouvement de stock

Colonne `cout_unitaire` nullable sur `mouvement_stock`, **jamais renseignée au MVP**. Sans elle, aucune valorisation rétroactive n'est possible : le profil 2 exigerait de recréer l'historique.

*Fichiers : user stories STK-02.*

### A5 — Code-barres et variantes

Colonnes `code_barre` et `article_parent_id`, toutes deux nullables, sur `article`. Non utilisées au MVP. Elles rendent les profils 2 et 3 additifs.

*Fichiers : user stories PDV-01.*

### A6 — Séparer module d'activité et capacité

Aujourd'hui `ETB-02` confond les deux. Créer deux référentiels distincts :
- `module_activite` — la verticale : hébergement, restauration, bar, pressing, salle de réunion.
- `capacite` — le transverse : `STOCK`, `LIVRAISON`, `PRODUCTION`, `COMMERCE_EN_LIGNE`, `FIDELITE`, `DEVIS`, `COMPTES_CLIENTS`.

Un module d'activité **déclare les capacités qu'il consomme**. Seule `STOCK` est implémentée au MVP.

*Fichiers : cadrage §4.1 · user stories ETB-02, ETB-08 · prompts cycle 2.*

### A7 — Profil de stock

Colonne `profil_stock ∈ {AUCUN, SIMPLE, VALORISE, DETAILLE}` sur le module d'activité. Seul `SIMPLE` est implémenté ; le code doit refuser explicitement les autres valeurs plutôt que les ignorer.

*Fichiers : user stories STK-01 · cadrage §14.*

### A8 — Répertoire des verticales et règle de dépendance

Le monorepo distingue :

```
backend/crates/
├── socle/          etablissements, comptes, caisse, fiscalite,
│                   documents, synchronisation, pilotage, editeur
├── capacites/      stocks, (production), (livraison), (commerce)
└── verticales/     hebergement, restauration, bar, pressing
```

**Règle de CI, non négociable** : aucun crate de `socle/` ne dépend d'un crate de `verticales/`. Un test structurel le vérifie. C'est ce qui empêche l'hôtellerie de contaminer le noyau — sans lui, tout ce document devient théorique en trois cycles.

*Fichiers : prompts §0.1, constitution principe 2, cycle 1.*

### A9 — Test structurel étendu

Le test « un maquis seul fonctionne de bout en bout » devient : **un établissement avec un module d'activité fictif minimal, ne consommant aucune capacité, fonctionne de bout en bout** — création, vente comptoir, encaissement, document fiscal, clôture.

C'est la preuve formelle que le socle est agnostique. Il tourne en CI pour toujours.

*Fichiers : user stories ETB-02 · prompts cycle 2.*

### A10 — Documents commerciaux en provision

Tables `devis` et `document_commercial` avec cycle `brouillon → émis → accepté → converti | expiré`. **Tables seulement.** La quincaillerie et tout le B2B en dépendent lourdement ; l'hôtellerie s'en passe.

*Fichiers : cadrage §14 · user stories FIS.*

### A11 — Métrique d'abonnement abstraite

`ADM-03` calcule aujourd'hui sur le nombre d'unités louables. Une quincaillerie n'en a aucune.

Introduire `unite_facturable`, **définie par la verticale** : la chambre pour l'hébergement, le point de vente pour la restauration et le commerce, le véhicule pour la livraison. Le moteur de tarification ne connaît qu'un nombre ; la verticale dit ce qu'on compte.

Au MVP, la seule implémentation est « chambre », et le comportement est strictement identique à aujourd'hui.

*Fichiers : cadrage §15.1 · user stories ADM-03 · prompts cycle 17.*

---

## 9. Ce qu'il ne faut surtout pas faire maintenant

| Tentation | Pourquoi non |
|---|---|
| Construire les profils de stock 2 et 3 | 18 semaines pour zéro client demandeur |
| Construire les capacités production, livraison ou commerce en ligne | Aucune ne sert le pilote |
| Créer un registre de verticales enfichables, un moteur de plugins | Sur-ingénierie classique. Deux verticales suffisent pour découvrir la bonne abstraction ; six sont nécessaires pour la valider. Tu en as deux |
| Généraliser l'interface avant d'avoir un second cas réel | La bonne généralisation se découvre au deuxième client, jamais au premier |
| Parler de la plateforme aux prospects hôteliers | Un hôtelier veut un logiciel d'hôtel. « Plateforme multi-activités » l'inquiétera |
| Modifier le périmètre du MVP | Les onze amendements n'ajoutent **aucune** fonctionnalité |

---

## 10. Effort réel et séquencement

| Étape | Effort | Prérequis |
|---|---|---|
| Amendements A1–A11 | **2 à 3 jours** | Maintenant |
| Hôtellerie en production, pilote validé | Plan actuel | — |
| Capacité stock profil 2 | 8 sem. | Un client demandeur identifié |
| Première verticale commerce (supérette) | 10–12 sem. | Profil 2 + **un pilote supérette signé** |
| Capacité production + boulangerie | 10–12 sem. | Un pilote boulangerie signé |
| Stock profil 3 + quincaillerie | 14–16 sem. | Une équipe |
| Capacité livraison | 8 sem. | Deux verticales la demandant |
| Produit commerce en ligne | 12–14 sem. | Réévaluation complète du marché |

**Le point d'honnêteté** : chaque verticale exige son propre pilote, ses propres cas fiscaux limites, sa propre charge de support. **Six verticales en solo est impossible** — ce n'est pas une question de rythme, c'est une question de nombre d'interlocuteurs à accompagner simultanément.

La séquence réaliste : hôtellerie → validation → levée → recrutement → **une verticale à la fois, chacune avec son pilote**. Une verticale sans pilote engagé ne se construit pas.

---

## 11. Risques propres à la plateformisation

| Risque | Prob. | Impact | Mitigation |
|---|---|---|---|
| **La vision plateforme contamine le MVP hôtelier** | **Élevée** | Fatal | Les onze amendements sont le SEUL impact autorisé. Ce document est fermé jusqu'à la mise en production du pilote |
| Abstraction prématurée du socle | Élevée | Élevé | Aucune généralisation avant un second cas réel. La règle A8 suffit ; le reste attend |
| Le socle se spécialise en hôtellerie sans qu'on le voie | Moyenne | Élevé | Test A9 en CI dès le cycle 2. C'est le garde-fou permanent |
| Chaque verticale exige son pilote et sa fiscalité | Certaine | Moyen | Une seule verticale ouverte à la fois. Aucune verticale sans pilote signé |
| Le tableau de bord consolidé devient illisible sur des activités hétérogènes | Moyenne | Moyen | Indicateurs universels au socle, spécifiques dans les verticales |
| La verticale commerce en ligne dilue la stratégie | **Élevée** | Élevé | Traitée comme un produit distinct, jamais comme un module. Décidée après la levée, sur données de marché |
| Le message commercial devient confus | Moyenne | Élevé | Une marque, une promesse par verticale. Jamais « plateforme » dans un argumentaire de vente |

---

## 14. Trois additions — juillet 2026

Trois idées ajoutées après la rédaction initiale. **Elles ne sont pas de même nature**, et c'est ce qui détermine où elles vont : une verticale, une capacité, et une chose qui n'est pas ce qu'elle paraît.

---

### 14.1 Transport et flotte — trois clients, pas un

**L'erreur à ne pas commettre : traiter « le transport » comme une verticale.** Il y a trois clients distincts, avec des douleurs, des capacités de paiement et des produits différents. Les confondre garantit un produit médiocre pour les trois.

| Profil | Qui | Douleur principale | Capacité à payer |
|---|---|---|---|
| **Gérant de parc** | 2 à 15 véhicules : massas, cars, **taxis, VTC** | *« Je ne sais pas ce que mes chauffeurs encaissent »* | ✅ Bonne — c'est le profil investisseur du §7 |
| **Compagnie de transport** | Flotte formelle, lignes et horaires réguliers | Billetterie, réservation, gestion de flotte, conformité | ✅ Bonne, mais exigences de type ERP |
| **Gare routière** | Syndicat ou gestionnaire de gare | Perceptions, tour de rôle, colis | ⚠️ Difficile — espèces, traçabilité indésirable pour certains acteurs |

#### Le gérant de parc — transport interurbain et taxi/VTC sont le même produit

**Observation qui simplifie beaucoup les choses** : celui qui possède trois massas et celui qui possède huit taxis à Abidjan ont **le même problème et le même logiciel**. Ce sont deux modèles de recette d'une seule verticale, pas deux verticales.

| | Transporteur interurbain | Parc taxi / VTC |
|---|---|---|
| Recette | Ventes de billets, le chauffeur remet la collecte | **Versement journalier fixe** — le chauffeur paie un montant convenu et garde le reste |
| Ce que le patron veut savoir | Combien a rapporté le véhicule, combien on m'a remis | Qui a versé, qui doit, depuis quand |

**Tout le reste est identique** : registre des véhicules, affectation des chauffeurs, attendu contre réalisé, entretien, documents réglementaires, compte de résultat par véhicule.

**Le versement journalier est la douleur dominante du marché ivoirien.** Un gérant de dix taxis attend 10 × 15 000 F par jour. Ce qu'il obtient réellement : des chauffeurs qui ne versent pas, qui invoquent une panne, qui disparaissent quelques jours. Il suit ça dans un cahier. **C'est exactement la forme du problème de la main courante hôtelière** — un attendu récurrent, un réalisé chaotique, aucune traçabilité.

Le socle y répond déjà presque entièrement : caisse, écarts, journal d'audit, tableau de bord consolidé. **Le véhicule devient un centre de recette**, structurellement proche d'une chambre : une ressource qui produit un revenu attendu et qu'on affecte à quelqu'un.

**Ce qu'il faut ajouter** : registre de véhicules, affectation de chauffeur, échéancier de versements, entretien, et surtout **documents réglementaires à échéance** (assurance, visite technique, vignette, patente) avec alerte avant expiration — petit, peu coûteux, et très apprécié.

> ⚠️ **Limite à énoncer honnêtement au client VTC** : si le chauffeur travaille pour une plateforme (Yango, Heetch), la donnée de recette réelle appartient à la plateforme, pas à nous. Nous ne pouvons suivre que **l'attendu contre le versé**, pas auditer le chiffre d'affaires réel. C'est déjà l'essentiel de la valeur, mais il ne faut pas promettre plus.

Effort estimé : **8 à 10 semaines**, la plus courte de toutes les verticales envisagées.

#### La compagnie de transport — le plus gros ARPU

Ligne régulière, horaire annoncé, places numérotées, vente à l'avance, parfois en ligne.

**Observation contre-intuitive** : ce cas s'ajuste **mieux** au modèle existant que la gare routière. Une place sur un départ programmé est une `ressource_reservable` avec un intervalle déterminé — c'est exactement le moteur de disponibilité de l'hôtellerie, appliqué à des sièges au lieu de chambres. Le départ au remplissage de la gare, lui, produit un intervalle dont la fin est **inconnue jusqu'au départ effectif**, ce qui est le cas difficile.

Ce que la compagnie exige en plus : gestion de flotte, maintenance, roulement des chauffeurs, manifeste passagers, vente en ligne. C'est un vrai produit, à ne pas sous-estimer.

Effort : **14 à 18 semaines**. Capacité nouvelle : **billetterie**.

#### La gare routière — le plus difficile

Six métiers empilés : billetterie, **tour de rôle**, départ au remplissage, **colis et fret** (souvent aussi rentable que les passagers), perceptions de gare et de syndicat, reversement aux propriétaires de véhicules, manifeste.

**Ce qui s'ajuste bien** : la gare est un `etablissement` sans torsion ; caisse et clôture sont le cœur du métier ; le **journal d'audit** est l'argument de vente n°1, car la fraude au billet est endémique ; l'hors-ligne est essentiel avec une connectivité médiocre et un volume élevé.

**Les fractures** :
1. **Le départ au remplissage casse la notion d'horaire.** Un départ a un seuil de remplissage, pas une heure. Le modèle doit accepter une ressource dont la fin d'intervalle est indéterminée.
2. **Le tour de rôle est une file politique.** C'est là que se logent les arrangements et les disputes. Un tour de rôle transparent et auditable serait une fonctionnalité extraordinaire — **ou la raison pour laquelle on nous met dehors**.
3. **Le partage de recette** avec les propriétaires est un module de règlement complet.
4. **Les colis** sont un métier logistique à part : dépôt, étiquetage, manifeste, remise contre signature, non-réclamés.

**Verdict** : à n'aborder qu'avec un pilote engagé et un accès terrain réel. Jamais en premier.

#### Séquencement du transport

`gérant de parc` (8–10 sem., meilleur ajustement, meilleur payeur, couvre massa/car **et** taxi/VTC) → `compagnie` (14–18 sem., billetterie + flotte) → `gare routière` (12–14 sem. de plus, selon l'accès terrain).

**Chaque étape réutilise la précédente.** La billetterie construite pour la compagnie sert la gare ; le suivi de véhicule construit pour le parc sert les deux autres.

---

### 14.1 bis — IoT transport : billetterie autonome et suivi de flotte

Deux idées, dont une est bien meilleure que l'autre.

#### Le billet auto-généré par QR — l'idée forte

Le passager scanne un QR, paie par Mobile Money, reçoit un billet à QR code ; le chauffeur le scanne à l'embarquement.

**Pourquoi c'est puissant, et ce n'est pas la raison qu'on croit** : la valeur n'est pas le confort du passager, c'est que **l'argent ne transite plus par un employé**. Au guichet d'une gare ou à bord d'un massa, la fraude classique est l'agent qui encaisse sans délivrer de billet. Un paiement direct par Wave ou Orange Money vers le compte du transporteur **supprime la main humaine du chemin de l'argent**. C'est exactement la douleur n°1 du persona transporteur.

Ça branche directement sur l'intégration CinetPay déjà prévue.

**La limite à ne jamais oublier** : ça suppose un smartphone et un compte mobile money. La pénétration est forte en Côte d'Ivoire mais **pas universelle**, en particulier sur le segment massa et badjan — clientèle plus modeste, lignes rurales. Le billet QR est donc **un complément à la vente en espèces, jamais un remplacement**. Le mode mixte est obligatoire, et la caisse doit rester irréprochable.

#### Le suivi de position en temps réel — utile, mais pour l'autre raison

Deux usages, de valeur très inégale :
- *Pour le passager* : savoir quand le car arrive. Agréable, peu monétisable.
- *Pour le transporteur* : savoir **où est son véhicule et s'il fait des courses non déclarées**. C'est un problème classique et coûteux, et c'est ce qui se vend.

Matériel : traceur GPS d'entrée de gamme, ou simplement le téléphone du chauffeur avec l'application.

**Trois prudences** : coût de données, batterie, et chauffeurs qui désactivent le suivi. Le produit doit **se dégrader proprement** — afficher « dernière position connue à telle heure », jamais promettre une continuité qu'on ne tient pas.

> ⚠️ **Fracture technique à consigner maintenant, pour que personne ne la conçoive mal plus tard.**
> Un flux de positions est une écriture **à haute fréquence et à faible valeur unitaire**. Il n'a rien à voir avec le reste des données de Kaya. Il ne doit **jamais** être écrit en une ligne Postgres par relevé : flux Redis ou base temporelle, avec instantanés périodiques et agrégats persistés. Une position brute ne survit pas plus de quelques jours.
> C'est le seul profil de donnée de tout le produit qui ne relève pas du schéma « Postgres source de vérité unique ».

Capacité : `SUIVI_FLOTTE`. Effort 6 à 8 semaines, dont l'essentiel en fiabilité de terrain et non en développement.

---

### 14.2 Partenaires externes — ce n'est pas de l'interopérabilité

**C'est l'addition la plus importante des trois, et elle est plus simple que tu ne le penses.**

Le besoin décrit : un hôtel sous-traite à un restaurant, une quincaillerie, un pressing extérieurs. Si ces entités ont Kaya, tant mieux ; sinon, SMS, appel ou WhatsApp.

Formulé ainsi, ça ressemble à deux fonctionnalités : une interopérabilité et un pis-aller. **C'est en réalité une seule fonctionnalité avec deux niveaux de qualité.**

> **Modélise le partenaire une seule fois.**
> `partenaire { etablissement_id, nom, type, telephone, canal_prefere, tenant_id? }`
> Le `tenant_id` est **nullable**. C'est tout.

À partir de là :

| Le partenaire… | Ce qui se passe |
|---|---|
| n'a pas Kaya | La demande est créée dans le système, transmise par WhatsApp ou SMS, et les statuts sont mis à jour **à la main** par le réceptionniste |
| a Kaya | La **même demande** devient une transaction réelle chez lui, et les statuts se synchronisent **automatiquement** |

**On ne construit donc pas d'interopérabilité. On construit une demande de service externe.** L'interopérabilité devient un chemin de mise à niveau — et un argument commercial redoutable : *« votre quincaillier utilise Kaya ? vos commandes se suivent toutes seules »*. C'est le mécanisme de croissance virale le plus naturel qu'on puisse avoir sur ce marché.

**Sur le canal de communication, deux vérités à connaître :**

1. **L'API WhatsApp Business n'est ni gratuite ni triviale** : vérification d'entreprise, modèles de messages soumis à approbation, tarification par conversation. Ce n'est pas une case à cocher.
2. **On n'en a pas besoin.** Un lien `wa.me` pré-rempli, ouvert depuis l'application, ne demande aucune API, aucun compte, aucun coût — et couvre 90 % du besoin. L'humain appuie sur « envoyer ». Le SMS via agrégateur est l'autre voie, souvent plus simple en Côte d'Ivoire.

**La règle** : générer le message, ouvrir le canal, laisser l'humain envoyer. L'API vient plus tard, quand le volume la justifie — jamais avant.

**Ce que ça ouvre au-delà de l'hôtellerie** : une quincaillerie qui commande à son grossiste, une boulangerie qui livre ses dépôts, une supérette qui gère ses fournisseurs. Le même modèle sert partout. **C'est probablement la capacité la plus transversale de toute la plateforme.**

Capacité : `PARTENAIRES`. Effort 4 à 6 semaines. Amendement A12 ci-dessous.

---

### 14.3 Conciergerie et coursier

**Commercialement, c'est la meilleure des trois** — et de loin la moins chère.

Pourquoi elle est bonne :
- Elle répond à un besoin observé, pas supposé.
- Elle **génère du revenu** pour l'hôtel au lieu de réduire un coût. C'est beaucoup plus facile à vendre.
- Elle est petite : quelques écrans, pas un module.
- Elle différencie **le produit hôtelier lui-même**, pas seulement le catalogue de verticales.
- Elle construit la moitié de la capacité `LIVRAISON`, qui servira ensuite à la boulangerie, à la supérette et aux colis de gare routière.

**Le modèle :**

```
coursier { etablissement_id, type ∈ {INTERNE, INDEPENDANT, ENTREPRISE},
           partenaire_id?, tarif_base, disponible }

course   { coursier_id, sejour_id?, description, destination,
           cout_course, avance_remise, depense_justifiee, retour_monnaie,
           qui_paie ∈ {CLIENT, ETABLISSEMENT, INCLUSE},
           statut: demandee → acceptee → en_cours → terminee | annulee }
```

Trois modes de facturation, tous nécessaires :
- **Client** — la course s'ajoute à la note de chambre, comme une consommation.
- **Établissement** — l'hôtel l'offre, elle entre en charge.
- **Incluse** — N courses comprises dans la formule de séjour. Voir A13.

**Le piège que personne n'anticipe : l'avance d'argent.**

Un client donne 10 000 F au coursier pour acheter un médicament à 7 500 F. Le coursier revient avec 2 500 F et un ticket. **C'est une mini-caisse**, et c'est exactement là que naissent les disputes et les disparitions d'argent.

Le modèle doit donc porter `avance_remise`, `depense_justifiee`, `retour_monnaie` **dès la conception**, avec justificatif photographié et trace au journal d'audit. Une conciergerie sans ce triplet créera des conflits dès la deuxième semaine d'exploitation.

**Deux points de prudence :**
- Un coursier interne salarié, un indépendant et une entreprise n'ont pas le même statut juridique ni la même responsabilité en cas d'accident. Ce n'est pas notre affaire, mais le produit ne doit pas laisser croire que c'est équivalent.
- Un coursier qui achète pour un client fait un acte d'achat au nom d'un tiers. Pour les catégories sensibles — médicaments notamment — c'est à cadrer.

Capacité : `CONCIERGERIE`, qui préfigure `LIVRAISON`. Effort 5 à 7 semaines. **Candidate sérieuse pour l'incrément 3**, contrairement au reste de ce document.

---

### 14.4 Ce que ça change au modèle en trois couches

Deux capacités nouvelles, aucune n'étant une verticale :

| Capacité | Consommée par | Effort |
|---|---|---|
| `PARTENAIRES` | **Toutes les verticales.** Sous-traitance, fournisseurs, grossistes | 4–6 sem. |
| `DOCUMENTS_REGLEMENTAIRES` | **Toutes les verticales.** Assurance et visite technique d'un véhicule, licence d'exploitation et classement d'un hôtel, patente, agrément fiscal | **1–2 sem.** |
| `CONCIERGERIE` | Hôtellerie d'abord, puis fusionne avec `LIVRAISON` | 5–7 sem. |
| `SUIVI_FLOTTE` | Gérant de parc, compagnie, gare, livraison | 6–8 sem. |
| `BILLETTERIE` | Compagnie de transport puis gare routière | 8–10 sem. |

> **`DOCUMENTS_REGLEMENTAIRES` est la meilleure affaire de tout ce document** : une à deux semaines d'effort, utile à absolument toutes les verticales, et immédiatement compréhensible par le client. Un document à date d'expiration, une alerte avant échéance, un tableau des documents périmés. Un hôtelier oublie son classement, un gérant de parc oublie une visite technique et perd le véhicule à un contrôle. À considérer même pour l'incrément 3 de l'hôtellerie.

**L'observation qui compte** : `PARTENAIRES` est probablement **la capacité la plus rentable de toute la plateforme**, parce qu'elle est la seule dont l'utilité augmente avec le nombre de clients. Chaque nouveau client rend le réseau plus utile aux précédents. Aucune autre capacité n'a cette propriété.

---

### 14.5 Amendements complémentaires

#### A12 — Généraliser la convention inter-établissements en partenaire ✅ appliqué

`ETB-07` prévoyait `convention_inter_etablissements`. Trop étroit : il ne couvre que le cas où les deux parties ont Kaya, c'est-à-dire le cas rare.

Remplacé par `partenaire { …, tenant_id? }` avec `tenant_id` **nullable**, plus `demande_partenaire` et le compte de compensation existant. Le partenaire sans compte Kaya est le cas normal ; celui avec compte est l'enrichissement.

**Tables seulement, aucune logique au MVP.** Coût : identique à l'ancienne provision.

#### A13 — Prestations incluses dans une formule ✅ appliqué

Table `prestation_incluse { formule_id, type, quantite, valeur_unitaire_plafond }`.

Elle sert la conciergerie (« 2 courses offertes »), mais surtout un besoin hôtelier bien plus courant.

> ✅ **Tranché le 28/07/2026.** Le **petit-déjeuner inclus** est une pratique répandue dans l'hôtellerie ivoirienne et n'apparaît nulle part dans le périmètre actuel. Que Deloria le pratique ou non, **le modèle doit le gérer** : d'autres hôtels le proposeront.
>
> - **Table `prestation_incluse` créée maintenant**, coût nul.
> - **Fonctionnalité candidate P1 de l'incrément 2**, indépendamment de la réponse de Deloria : afficher la prestation incluse sur la note, la décompter à la consommation, ne pas la facturer, et signaler le dépassement du quota.
> - Ce n'est **pas** de l'ambition plateforme : c'est une lacune du produit hôtelier, à combler comme telle.

---

| # | Question | Quand |
|---|---|---|
| P-01 | La supérette est-elle la première verticale, ou la boulangerie ? Dépend du premier pilote qu'on obtient, pas de nos préférences | Après la levée |
| P-02 | Le commerce en ligne est-il un produit distinct ou abandonné ? Décision de marché, pas de technique | Après la levée |
| P-03 | Une marque unique ou une marque par verticale ? | Avec la seconde verticale |
| P-04 | La livraison est-elle vendue séparément ? | Avec la troisième verticale |
| P-05 | Le stock profil 3 justifie-t-il un crate séparé du profil 2 ? | À la conception du profil 2 |
| P-06 | Les indicateurs consolidés hétérogènes : quel jeu universel ? | Avec la seconde verticale |
| P-07 | La conciergerie entre-t-elle dans l'incrément 3 ? C'est la seule addition assez petite pour y prétendre | À l'arbitrage de l'incrément 3 |
| P-08 | ✅ **Tranché** : les trois profils transport sont servis, dans l'ordre transporteur → compagnie → gare. Aucune porte fermée aux compagnies | 28/07/2026 |
| P-09 | `PARTENAIRES` en capacité autonome, ou fondue dans un module achats ? | Avec la première verticale commerce |
| P-10 | Canal de communication partenaire : lien `wa.me` seul, ou agrégateur SMS ? | À la conception de `PARTENAIRES` |
| P-11 | ✅ **Tranché** : le modèle doit gérer les prestations incluses, que Deloria les utilise ou non. Table en provision, fonctionnalité candidate P1 incrément 2 | 28/07/2026 |
| P-12 | Le billet QR autonome est-il vendu comme anti-fraude au gérant de parc, ou comme confort au passager ? Détermine tout l'argumentaire | Avec la verticale flotte |
| P-14 | `DOCUMENTS_REGLEMENTAIRES` remonte-t-elle en incrément 3 de l'hôtellerie ? 1 à 2 semaines pour une valeur immédiate | Arbitrage incrément 3 |
| P-13 | Stockage des positions de flotte : flux Redis, base temporelle dédiée, ou service tiers ? | Avant tout travail sur `SUIVI_FLOTTE` |

---

## 13. La règle qui protège tout

> **Ce document est fermé jusqu'à ce que Deloria ait abandonné son cahier papier.**

Les onze amendements s'appliquent maintenant parce qu'ils coûtent trois jours et évitent des migrations. Tout le reste attend le jalon J1.

Un projet qui construit une plateforme avant d'avoir un client qui paie ne construit rien du tout.
