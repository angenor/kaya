# Kaya — Document de cadrage MVP

*Plateforme de gestion pour établissements d'hébergement et de service — Pilote : Résidence Hôtel Deloria, Abengourou, Côte d'Ivoire*
*Version 1.0 — Document de référence produit & technique*

> **Nom de code provisoire.** `Kaya` est un placeholder jusqu'à la décision B-06 (annexe B). Il est utilisé partout dans les documents et le dépôt ; un renommage global est trivial tant qu'il n'y a pas de marque déposée.

---

## 1. Résumé exécutif

Kaya est une plateforme de gestion pour **établissements d'hébergement et de service** en Afrique. Elle remplace la main courante papier par un système unique couvrant hébergement, restauration, bar, pressing, salle de réunion, caisse et conformité fiscale.

**L'entité centrale n'est pas l'hôtel, c'est l'établissement.** Un établissement active les **modules d'activité** dont il a besoin. Un hôtel avec restaurant, bar et pressing active les quatre ; un maquis n'active que la restauration ; une résidence meublée n'active que l'hébergement. Le produit est le même, la configuration diffère.

Piliers du modèle :

- **Conformité fiscale native** — FNE (Côte d'Ivoire), taxe communale de nuitée avec état de reversement par commune, TVA, taxe pour le développement touristique. Le moteur est un système d'adaptateurs par juridiction dès le premier jour : ajouter un pays est un adaptateur, pas une réécriture.
- **Toutes les formules de location du marché africain** — nuitée, **passage horaire**, **demi-journée**, séjour long au mois. Le passage et la demi-journée sont majoritaires en volume dans une grande partie du parc et absents des PMS existants ; c'est un différenciateur immédiat et une source de traçabilité que les propriétaires n'ont pas aujourd'hui.
- **Application unique, interface selon les rôles cumulés** — un gérant qui est aussi caissier et réceptionniste installe une seule application. Desktop, Android, iOS.
- **Résilience réseau** — le service continue pendant les coupures, sans perte ni double comptage, sans jamais produire de document fiscal invalide.
- **Souveraineté des données** — SaaS mutualisé par défaut, auto-hébergement (LAN ou cloud du client) en option commerciale.
- **Monolithe modulaire, microservices-ready** — un crate par domaine, interfaces par traits, schéma Postgres par module, journal d'événements en sortie. Aucun service n'est extrait au MVP ; aucun ne sera bloqué de l'être.

**Objectif : Deloria abandonne son cahier papier au mois 4.** La rentabilité n'est pas visée avant 18 mois ; le pilotage se fait sur l'adoption réelle, la conformité et la constitution du dossier de levée.

---

## 2. Marché et établissement pilote

### 2.1 Pilote — Résidence Hôtel Deloria, Abengourou

| Élément | Valeur |
|---|---|
| Localisation | Abengourou, ~220 km d'Abidjan (3–4 h de route) |
| Unités | 17 chambres + 1 salle de réunion |
| Catégories | Standard (A1–A3) 12 500 · Classique (B1–B5) 15 500 · Classique sup. (C1–C4) 17 500 · Supérieure A (D1–D2) 20 500 · Supérieure B (E1–E3) 25 500 · Salle de réunion 50 500/jour |
| Modules actifs | Hébergement, restauration, bar, pressing, salle de réunion, caisse |
| Fonctionnement actuel | Main courante papier ; report manuel en caisse en fin de journée ; cartes à codes-barres pour l'accès aux chambres |
| Classement | Non classé (à confirmer — détermine le barème de la taxe de nuitée) |

**Conséquences de la distance, actées dans le plan** : support à distance par défaut (télémétrie, journaux remontés, mise à jour à distance obligatoires dès l'incrément 1) ; déplacements planifiés et non réactifs, 8 à 12 allers-retours budgétés sur la durée du pilote ; qualité réseau supposée inférieure à Abidjan, le mode dégradé sera exercé en conditions réelles.

**Point de conformité à traiter à l'atelier initial** : les tarifs affichés « incluent une augmentation de 500 FCFA par catégorie », ce qui correspond au montant de la taxe communale de nuitée pour un établissement non classé. Intégrée au prix au lieu d'être une ligne distincte sur la facture, elle place l'établissement en infraction. La reprise de données devra **décomposer chaque tarif** en prix HT + TVA + taxe de nuitée.

### 2.2 Ce que le cahier des charges du pilote omet — et qui est au périmètre

Le client ne sait pas encore ce dont il a besoin. Ces éléments sont dans le MVP :

Réservations et planning · arrhes et acomptes · no-shows, annulations, avoirs · prolongation et départ anticipé · **formules passage et demi-journée** · tarifs négociés et remises tracées · fiche de police · **taxe communale de nuitée** · TVA et mentions obligatoires · moyens de paiement multiples · fond de caisse, écart de caisse, shifts et passations · gestion des accès (cartes à codes-barres à cartographier).

### 2.3 Marché

Ancre de prix locale : **1 000 FCFA/unité/mois** (Chobah). Autres acteurs sur la zone : IvoirePMS, Hotelia.cloud, KiboERP. Aucun ne traite les formules horaires ni le reversement communal automatisé.

Cible : **Afrique**. Côte d'Ivoire en premier marché, puis zone UEMOA/CEMAC (SYSCOHADA commun, franc CFA commun, réformes d'e-facturation en cours dans plusieurs pays), puis anglophone.

Le facteur limitant du projet est l'acquisition client, pas la difficulté technique. C'est ce qui justifie la stratégie : un pilote de référence irréprochable, puis levée.

---

## 3. Périmètre du MVP

### 3.1 Dans le MVP

| Composant | Description |
|---|---|
| **Socle multi-tenant** | Isolation par `tenant_id` + Row Level Security forcée, RBAC cumulatif, i18n fr/en, mode sombre, branding client (logo, couleurs, en-têtes de documents) |
| **Établissements & modules** | Un tenant, N établissements ; chaque établissement active ses modules d'activité (hébergement, restauration, bar, pressing, salle de réunion). L'interface ne montre que les modules actifs |
| **Hébergement** | Unités louables, catégories, **formules nuitée / passage horaire / demi-journée / mensuel**, disponibilité en intervalles horodatés, temps de remise en état |
| **Réservations** | Création, planning visuel, disponibilité, arrhes, no-show, annulation, prolongation |
| **Séjours** | Check-in, check-out, **enregistrement accéléré avec OCR de pièce d'identité**, note temps réel, transfert de charges, fiche de police |
| **Points de vente** | Restaurant, bar, pressing, salle de réunion — catalogue, commande par table ou par unité, envoi cuisine, ticket, division d'addition |
| **Commande par QR** | Catalogue public par table, panier, **validation obligatoire par le personnel** avant envoi |
| **Clients extérieurs** | Vente directe sans hébergement, encaissement immédiat, reçu |
| **Caisse** | Fond de caisse, encaissements multi-modes, écart, shifts, clôture journalière atomique |
| **Fiscalité** | Moteur d'adaptateurs par juridiction ; FNE via partenaire agréé puis en direct ; taxe de nuitée ; TVA ; taxe dév. touristique ; avoirs ; **état de reversement par commune** |
| **Stocks** | Articles, entrées, sorties sur vente, inventaire, alertes de seuil |
| **Direction** | Tableau de bord consolidé multi-établissements, KPI, comparaison, journal d'audit |
| **Console éditeur** | Provisionnement de tenants, abonnements, télémétrie, diagnostic |
| **Impression** | Tickets thermiques ESC/POS 80 mm, factures A4 PDF, tiroir-caisse |
| **Déploiement** | SaaS mutualisé (défaut) + paquet auto-hébergé |
| **Applications** | Une application Nuxt 4 + Tauri v2 — Windows, macOS, Linux, Android, iOS ; surfaces web publiques séparées (QR, console éditeur) |

### 3.2 Hors MVP (infrastructure prête le cas échéant, §14)

Comptabilité SYSCOHADA (export normalisé au MVP) · CRM · BI libre-service · site web vitrine offert et interconnecté · channel manager et OTA · agents IA · nœud de site LAN · conventions inter-tenants (restaurant indépendant hébergé dans un hôtel tiers) · contrats de location, cautions et charges locatives · IoT et ouverture de chambre par QR · RNE et terminaux TERNE · multi-devises actif.

**Règle de gouvernance** : toute fonctionnalité qui ne contribue pas directement à (a) faire abandonner le papier au pilote, ou (b) garantir la conformité fiscale, est reportée. Sans exception jusqu'à la fin de l'incrément 1.

---

## 4. Modèle d'entité universel

### 4.1 Hiérarchie

```
TENANT (propriétaire / groupe)
  ├── configuration fiscale, branding, abonnement, utilisateurs
  └── ÉTABLISSEMENT (1..n)
        ├── juridiction, classement, commune, fuseau horaire
        ├── MODULES D'ACTIVITÉ activés (0..n)  ── la verticale
        │     ├── HEBERGEMENT   → ressources réservables, formules, réservations, séjours
        │     ├── RESTAURATION  → points de vente, catalogue, tables
        │     ├── BAR           → points de vente, catalogue, tables
        │     ├── PRESSING      → points de vente, catalogue, prestations
        │     └── SALLE_REUNION → ressources réservables (formule journée/demi-journée)
        ├── CAPACITÉS consommées par les modules ── le transverse
        │     ├── STOCK               (profil AUCUN | SIMPLE | VALORISE | DETAILLE)
        │     ├── LIVRAISON           (non implémentée)
        │     ├── PRODUCTION          (non implémentée)
        │     ├── COMMERCE_EN_LIGNE   (non implémentée)
        │     ├── FIDELITE            (non implémentée)
        │     ├── DEVIS               (non implémentée)
        │     └── COMPTES_CLIENTS     (non implémentée)
        ├── POINTS DE VENTE (0..n, rattachés à un module)
        └── CAISSES (1..n)
```

**Règles structurelles, non négociables :**

1. **Le socle ne connaît ni « chambre », ni « unité louable », ni « séjour ».** Il connaît `article_vendable` et `ressource_reservable`. `unite_louable`, `formule` et `sejour` sont des spécialisations de la verticale hébergement et vivent dans son crate.
2. **Aucun crate du socle ne suppose que l'établissement possède de l'hébergement.** Un maquis seul est un établissement valide avec le seul module `RESTAURATION`.
3. **Aucun crate du socle ne suppose que l'établissement possède un point de vente.** Une résidence meublée seule est un établissement valide avec le seul module `HEBERGEMENT`.
4. **Module d'activité et capacité sont deux référentiels distincts.** Le module est la verticale (ce que fait l'établissement) ; la capacité est le transverse (ce dont il a besoin pour le faire). Un module **déclare les capacités qu'il consomme**. Au MVP, seule `STOCK` au profil `SIMPLE` est implémentée ; toute autre valeur est **refusée explicitement**, jamais ignorée.
5. Le module `SALLE_REUNION` est une **spécialisation d'hébergement** : c'est une ressource réservable dont les formules par défaut sont la journée et la demi-journée. Il ne crée aucune entité nouvelle.
6. La **note client** est portée par l'établissement, pas par le module. Une consommation au bar s'ajoute à la note d'un séjour si l'hébergement est actif, à une addition de table sinon.
7. L'interface **ne montre jamais un module ou une capacité inactifs**. Pas de grisé, pas de « disponible dans votre offre » : absent.

### 4.2 Configurations types

| Type d'établissement | Modules actifs | Particularités |
|---|---|---|
| Hôtel complet (Deloria) | Hébergement, restauration, bar, pressing, salle de réunion | Transfert de charges vers la note de chambre |
| Résidence meublée | Hébergement | Formules demi-journée et mensuel prépondérantes |
| Maquis / restaurant seul | Restauration | Aucune note de chambre ; toute vente est une vente comptoir |
| Bar seul | Bar | Idem |
| Pressing seul | Pressing | Prestations avec délai de retrait, bon de dépôt |
| Restaurant + bar | Restauration, bar | Deux points de vente, une caisse ou deux selon configuration |
| Hôtel sans restauration | Hébergement | Note de chambre limitée à l'hébergement et aux extras |

### 4.3 Établissement indépendant hébergé dans un tiers

Cas réel et fréquent : un restaurant appartenant à un propriétaire A, physiquement situé dans l'hôtel du propriétaire B.

**Au MVP** : ce sont deux tenants distincts, deux abonnements, deux établissements sans lien. Le client de l'hôtel qui consomme au restaurant indépendant paie directement au restaurant.

**Provision (§14.9)** : le modèle prévoit une `convention_inter_etablissements` — accord bilatéral autorisant le restaurant à pousser une consommation vers la note de chambre d'un séjour du tenant voisin, avec un compte de compensation et un relevé de règlement périodique. **Tables uniquement, aucune logique au MVP.**

---

## 5. Hébergement — unités, formules et disponibilité

### 5.1 Le changement structurel : disponibilité en intervalles horodatés

Le marché africain pratique massivement des locations infra-journalières. La disponibilité **ne peut donc pas** être modélisée en dates.

> **Principe non négociable** : une occupation est un intervalle `[début, fin)` en **timestamp avec fuseau horaire de l'établissement**, jamais une paire de dates. Toute requête de disponibilité est une recherche de chevauchement d'intervalles. Ce choix est structurant et irréversible ; il est posé au premier cycle.

Implémentation : contrainte d'exclusion PostgreSQL (`EXCLUDE USING gist` sur `unite_id` + `tstzrange`) — le chevauchement devient impossible au niveau de la base, pas seulement dans le code applicatif. C'est la garantie la plus forte contre la double attribution.

### 5.2 Les quatre familles de formules

| Formule | Comptage | Usage | Contraintes typiques |
|---|---|---|---|
| **Nuitée** | Par nuit | Hôtellerie classique | Heure d'arrivée et de départ standard (ex. 14h / 12h), durée min 1 nuit |
| **Passage** | Par heure, à paliers dégressifs | Hôtellerie — majoritaire en volume dans une partie du parc | Durée min 1 h, durée max paramétrable, plages horaires autorisées |
| **Demi-journée** | Par plage fixe | Résidences meublées, salles de réunion | Plages définies (ex. 8h–12h, 13h–16h), non fractionnables |
| **Mensuel / long séjour** | Par mois ou par tranche de N nuits | Résidences meublées **et hôtels** — un client peut séjourner plusieurs mois en hôtel | Durée min paramétrable, facturation périodique |

**Aucune formule n'est réservée à un type d'établissement.** Un hôtel peut proposer du mensuel ; une résidence meublée peut proposer du passage. La formule est attachée à la catégorie d'unité, pas au type d'établissement.

### 5.3 Barème dégressif du passage

Le passage se vend par paliers cumulés avec exonération progressive. Exemple de barème, éditable par catégorie :

| Durée | Prix | Prix horaire implicite |
|---|---|---|
| 1 h | 1 500 | 1 500 |
| 2 h | 2 800 | 1 400 |
| 3 h | 4 000 | 1 333 |
| 4 h | 5 000 | 1 250 |
| Heure supplémentaire | +1 200 | — |

Règles du moteur :
- Le barème est une **table de paliers** `{durée, prix}` avec un prix d'heure supplémentaire au-delà du dernier palier.
- Un dépassement constaté au départ **rebascule automatiquement** sur le palier supérieur, avec la différence ajoutée à la note et tracée.
- Un dépassement au-delà d'un seuil paramétrable **bascule en nuitée** (règle éditable par établissement).
- Les seeds Deloria (1 h = 1 500, 4 h = 5 000) sont à confirmer à l'atelier initial.

### 5.4 Temps de remise en état

Entre deux occupations d'une même unité, un **délai de remise en état** paramétrable par catégorie et par formule bloque l'unité (ménage, aération). Il est intégré à l'intervalle d'indisponibilité, pas géré à part.

Défauts : passage 30 min · nuitée 2 h · demi-journée 1 h. Éditables.

### 5.5 ⚠️ Fiscalité des formules infra-journalières

**Question ouverte bloquante (annexe A, question 8).** La taxe communale de nuitée est due « par nuitée et par client ». Un passage d'une heure n'est pas une nuitée ; une demi-journée non plus.

**Conception imposée** : chaque formule porte un drapeau `assujettie_taxe_nuitee` et une **règle de conversion** (`aucune` / `une_nuitee_par_occupation` / `au_prorata` / `seuil_horaire`). Aucune valeur n'est codée en dur. Le paramétrage par défaut sera fixé après avis du fiscaliste (décision B-02) et pourra différer par commune.

Même traitement pour la TVA et la taxe de développement touristique : ce sont des sorties de l'adaptateur de juridiction, jamais des constantes.

### 5.6 Point de vigilance opérationnel

Le passage est aujourd'hui massivement encaissé en espèces et non tracé. Le tracer donne au propriétaire une visibilité qu'il n'a pas — c'est un argument de vente puissant, **et une source de résistance du personnel** au déploiement. À anticiper dans la conduite du changement : le module de passage doit être irréprochable en rapidité (moins de 30 secondes pour enregistrer un passage) sinon il sera contourné.

---

## 6. Points de vente — restauration, bar, pressing

### 6.1 Modèle commun

Un **point de vente** appartient à un module d'activité d'un établissement. Il porte : un catalogue, des tables (optionnel), une politique d'impression, une caisse de rattachement.

Une **commande** est rattachée à un point de vente et à une **cible de facturation** :

| Cible | Condition | Comportement |
|---|---|---|
| Table | Point de vente avec tables | Addition ouverte, réglée au départ du client |
| Séjour | Module hébergement actif et séjour en cours | Ajout à la note de chambre |
| Comptoir | Toujours | Vente immédiate, encaissement direct |
| Emporter / livraison | Paramétrable par point de vente | Vente immédiate avec référence de retrait |

### 6.2 Spécificité du pressing

Le pressing n'est pas une vente immédiate : il y a **dépôt, délai, retrait**. Modèle : un **bon de dépôt** avec liste d'articles, état constaté, date de retrait promise, et un statut (`déposé → en traitement → prêt → retiré`). Le règlement peut être à l'avance ou au retrait, paramétrable.

Les articles déposés d'un client logé sont rattachés à son séjour ; ceux d'un client extérieur à un bon autonome avec numéro de retrait.

### 6.3 Fallback sans réseau

La prise de commande est de classe A (§11) : elle fonctionne intégralement hors ligne. L'envoi en cuisine sur le LAN local fonctionne aussi. L'encaissement et la certification fiscale ne fonctionnent pas hors ligne et le disent explicitement.

---

## 7. Commande client par QR

### 7.1 Parcours

1. Le client scanne le QR posé sur sa table.
2. Une page web publique (hors application, Nuxt SSR) affiche le catalogue du point de vente.
3. Le client compose son panier et valide.
4. La commande arrive en état `À_CONFIRMER` sur le terminal du serveur du point de vente.
5. **Le serveur valide d'un tap en constatant la présence physique du client.** Rien ne part en cuisine avant.

### 7.2 Anti-fraude — décision arrêtée

**La validation par le personnel est le seul mécanisme retenu au MVP.** Coût nul, efficacité quasi totale, aucune friction pour le client légitime. Un ordre passé à distance produit au pire une notification que le serveur balaie.

Aucun géorepérage, aucun portail captif, aucun paiement préalable au MVP.

### 7.3 Spécification du QR

- **Contenu** : URL courte `kaya.app/t/{token}` où le jeton est signé (HMAC), opaque et **révocable côté serveur** — jamais un identifiant de table lisible ou devinable.
- **Révocation** : un QR arraché, photographié ou déplacé est révoqué depuis le back-office sans changer la plaque physique.
- **Limitation de débit par jeton** : N paniers en attente maximum par table (défaut 3, paramétrable), sinon un plaisantin sature l'écran du serveur.
- **Aucune donnée personnelle demandée** : pas de compte, pas de téléphone, pas d'email. C'est une page publique anonyme.
- **Scan hors contexte** (QR photographié, table fermée) : page neutre « service indisponible ».

---

## 8. Caisse, encaissements et clôture

### 8.1 Chaîne

Fond de caisse à l'ouverture de shift → encaissements multi-modes → sorties de caisse tracées → comptage → **écart constaté et motivé** → clôture de shift → clôture journalière de l'établissement.

Modes de règlement au MVP : espèces, Mobile Money (via agrégateur), carte, virement, à crédit (compte client). Un règlement peut être **fractionné entre plusieurs modes** sur une même note — contrairement à un service de livraison, c'est la norme en hôtellerie.

### 8.2 Clôture journalière — règles de blocage

La clôture est **atomique** et **refusée** tant que :
- la file de synchronisation n'est pas vide ;
- une facture est en attente ou en échec de certification ;
- un terminal de l'établissement est déconnecté depuis plus de N minutes (défaut 15) ;
- une addition de table est restée ouverte sans décision.

Le refus affiche précisément ce qui bloque. Une clôture fausse est pire qu'une clôture tardive : elle est signée, imprimée, et devient la référence.

### 8.3 Journal d'audit

Actions systématiquement tracées, immuables, consultables par le propriétaire depuis n'importe quel terminal : remise, annulation de ligne envoyée, avoir, ouverture de tiroir, modification de tarif, suppression, changement de rôle, écart de caisse, dépassement de passage rebasculé.

**C'est ce que le propriétaire achète réellement.** Le module doit être conçu comme une fonctionnalité de premier plan, pas comme un journal technique.

---

## 9. Fiscalité et conformité — Côte d'Ivoire

### 9.1 Facture Normalisée Électronique

Modèle de **clearance** : la facture est transmise, contrôlée, puis renvoyée avec numéro normalisé, visuel officiel, QR code et sceau fiscal. **Elle ne peut être remise au client qu'après autorisation.** Obligatoire pour toutes les entreprises depuis le 1er décembre 2025. Sanctions jusqu'à 10 M FCFA et perte du droit à déduction de TVA. Archivage 6 à 10 ans.

**Endpoints documentés** : facture de vente, facture d'avoir, bordereau d'achat agricole. Environnement de test : `http://54.247.95.108`, endpoints sur `/ws`. URL de production transmise après validation des spécimens (`support.fne@dgi.gouv.ci`).

> L'environnement de test est en **HTTP nu sur IP publique**. Aucune donnée réelle ne doit y transiter.

### 9.2 Montage retenu — intégrateur technique

**La clé API FNE est rattachée au NCC du contribuable**, pas à celui de l'éditeur, et n'est visible que par le gestionnaire principal de l'entreprise dans son espace FNE.

Montage acté :

| Élément | Règle |
|---|---|
| Compte FNE | **Chaque établissement client possède son propre compte FNE et sa propre clé.** Étape obligatoire du parcours d'installation |
| Transmission | Kaya transmet à la plateforme **pour le compte du client**, avec la clé du client |
| Pont de démarrage | L'API du partenaire agréé (beyima.com) sert de canal pendant l'instruction de notre agrément propre |
| Abstraction | Trait `FneGateway { certify, refund, status }` — implémentations `Partenaire` et `Direct`, bascule par configuration de tenant, sans toucher au métier |
| Stockage des clés | **Coffre chiffré par tenant**, dès le MVP |

**Agrément éditeur** : la procédure existe (communiqué DGI du 13/08/2025). Conditions administratives : immatriculation au fichier des contribuables, résidence fiscale ivoirienne, attestation de régularité fiscale, inscription sur la plateforme FNE, courrier au DGI avec formulaire. Validation en deux temps : contrôle de conformité des spécimens, **puis contrôle technique effectué par la DGI dans nos locaux**. Délai non publié — question A-01, à poser en semaine 1. Ce contrôle sur site est le point qui pèse sur le planning.

### 9.3 Idempotence — contrainte critique

> ⚠️ **L'API FNE n'expose aucune clé d'idempotence.** Sur un timeout réseau, il est impossible de savoir si la facture a été certifiée. Un rejeu naïf produit une double certification et consomme un sticker en double.

**Conception imposée** : file persistante à quatre états.

```
EN_ATTENTE → SOUMISE → CERTIFIEE
                    ↘ ECHEC        (erreur métier explicite → correction et resoumission)
                    ↘ INDETERMINEE (timeout → JAMAIS rejouée automatiquement)
```

L'état `INDETERMINEE` alimente un **écran de rapprochement manuel** où l'opérateur vérifie dans l'espace FNE du client et tranche. Aucune automatisation n'est acceptable ici.

**Surveillance des stickers** : alerte à J-7 et J-2 du seuil bas, visible du gérant et du propriétaire. Un établissement à court de stickers ne peut plus facturer ; le délai de rechargement publié est de 48 h avant blocage.

### 9.4 Avoirs — deux contraintes structurantes

`POST {url}/external/invoices/{id}/refund`, corps limité à la liste des articles retournés (`id` d'item, `quantity`). Réponse 201, référence préfixée `A`, débit d'un sticker.

1. **L'avoir se fait par quantité, pas par montant.** Aucun geste commercial partiel sur une ligne. Une remise après réclamation impose d'annuler la ligne entière et de refacturer au tarif remisé. L'interface doit guider l'opérateur dans cette manipulation, pas la subir.
2. **Les `id` d'items retournés par l'API de certification doivent être persistés**, pas seulement les identifiants internes. Sans eux, aucun avoir n'est possible. C'est une erreur de conception irrattrapable a posteriori.

Aucune annulation pure n'est documentée : l'avoir est le seul mécanisme.

### 9.5 Documents non fiscaux

Le mode dégradé ne produit **jamais** un document ressemblant à une facture normalisée. Il produit un **document explicitement non fiscal** portant la mention « Document non fiscal — ne tient pas lieu de facture », et place l'opération dans une file de régularisation avec alerte visible du gérant. La certification est automatique au retour du réseau, sous réserve du contrôle du stock de stickers.

### 9.6 Taxe communale de nuitée

**Par nuitée et par séjour** — jamais par personne : sans étoile 500 · 1★ 1 000 · 2★ 1 500 · 3★ et plus 2 000 · résidence meublée district d'Abidjan 1 000 · hors Abidjan variable selon la collectivité.

> **⚠️ Cette ligne disait « par nuitée et par client » jusqu'au 2026-08-03.** L'arbitrage terrain de
> cette date, qui **clôt la décision B-10** (annexe B), a tranché : la taxe est due **par nuitée et
> par séjour**, le nombre de personnes ne la multiplie pas. Un couple en chambre double paie une
> taxe, pas deux.
>
> **Ce que la correction change, et ce qu'elle ne change pas.** Elle ne touche que l'**axe des
> personnes**. L'axe des **nuits** — combien de nuitées un passage ou une demi-journée produit —
> reste celui du §5.5 : le drapeau `assujettie_taxe_nuitee` et la règle de conversion de la formule,
> décision **B-02**, toujours ouverte. Les deux axes se confondaient dans l'ancienne formulation,
> et c'est ce qui rendait B-10 difficile à lire.
>
> **La question de l'exonération tombe avec elle.** B-10 demandait s'il manquait une colonne de
> motif d'exonération par personne. La taxe n'étant plus assise sur les personnes, il n'y a rien à
> exonérer personne par personne : **aucune colonne n'est due**, et `hebergement.accompagnant` n'en
> porte pas. Le nombre de personnes reste enregistré au constat de taxe (`nombre_personnes`) à
> titre **indicatif** — il documente le séjour, il n'entre dans aucun calcul.

**Ligne distincte obligatoire sur la facture**, séparée du HT et de la TVA. Reversement au trésorier municipal **au plus tard le 15 du mois suivant l'encaissement**.

Livrable produit associé : **état de reversement mensuel par commune**, avec relevé des nuitées assujetties, montant dû et échéance. Aucun concurrent ne le produit.

Le montant est **figé au check-out**, jamais recalculé dynamiquement. Toute modification postérieure passe par un avoir.

### 9.7 Autres obligations

| Sujet | Contenu |
|---|---|
| TVA | 18 %, mentions obligatoires (identification complète du client, désignation détaillée) |
| Taxe dév. touristique | 2,5 % depuis l'annexe fiscale 2026, étendue au régime de l'entreprenant |
| SYSCOHADA | Export normalisé au MVP, comptabilité intégrée en phase 2 |
| Mobile Money | Reconnu comme monnaie électronique (annexe fiscale 2026) |
| ARTCI | **Les obligations suivent les rôles, pas la localisation du serveur.** Le client est responsable de traitement, Kaya est sous-traitant. L'auto-hébergement n'y change rien. Ce qu'il change : la question du transfert transfrontalier (décision B-01) |
| Fiche de police | Registre des clients — à cartographier avec le pilote |
| Classement étoiles | Champ obligatoire au paramétrage, détermine le barème de nuitée |

### 9.8 RNE — hors périmètre, provision conservée

Décision : **le RNE est écarté du MVP.** La voie API produisant une FNE est la procédure de droit commun, ouverte à toutes les catégories d'entreprises ; nos points de vente émettent des FNE B2C.

Provision conservée sans coût : le trait `EmissionChannel { FneApi, Terne }` existe, et les lignes de facture portent une colonne `rne_ref` nullable. Si la DGI qualifie un jour nos points de vente de caisses enregistreuses au sens du texte, l'implémentation `Terne` s'ajoute sans migration. Les champs `isRne` et `rne` du payload de certification prévoient déjà le chaînage RNE → FNE consolidée.

---

## 10. Topologies de déploiement et hors-ligne

### 10.1 Trois modes, un seul binaire

| Mode | Description | Disponibilité |
|---|---|---|
| **A — SaaS mutualisé** | Défaut. Base unique, isolation par `tenant_id` + RLS. Cache local sur les terminaux | MVP |
| **B — Auto-hébergé** | Même binaire déployé chez le client (son LAN ou son cloud). Argument commercial | MVP |
| **C — Nœud de site** | Serveur local dans l'établissement, autoritaire sur ses données opérationnelles, répliquant vers le cloud | Incrément 3 |

Le serveur, le nœud de site et le paquet auto-hébergé sont **le même binaire Actix avec trois configurations**. Jamais trois produits.

### 10.2 Coût de l'auto-hébergement — règles d'encadrement

- **Versions N et N-1 supportées, pas plus.**
- Migrations automatiques et idempotentes au démarrage.
- **Télémétrie minimale obligatoire** (version, santé, erreurs) — sans elle, le diagnostic à distance est impossible.
- Export d'un **bundle de diagnostic** déclenchable par le client et transmissible au support.
- Contrat distinct : sauvegarde et disponibilité incombent au client.

### 10.3 Résilience matérielle

Le déploiement en mode B ou C impose au cahier des charges d'installation : onduleur, arrêt propre, sauvegarde horaire. Une coupure secteur sur un Postgres non protégé produit une corruption.

---

## 11. Classification des données par tolérance au hors-ligne

### 11.1 Les quatre classes

| Classe | Critère | Autorité | Écriture hors ligne |
|---|---|---|---|
| **A** | Append-only, commutatif, sans contrainte d'unicité, sans effet monétaire | Aucune | Oui |
| **B** | Sérialisation requise, à l'échelle d'un établissement | Nœud de site (mode C) ou cloud | Mode C seulement |
| **C** | Référentiel partagé entre établissements, ou relation éditeur–client | Cloud | Non |
| **D** | Dépend d'un tiers (DGI, agrégateur de paiement) | Externe | Non |

> **Règle absolue** : une opération B, C ou D n'est **jamais** atteignable depuis un chemin de code exécutable hors ligne. Invariante vérifiée par test, pas convention.

> **Règle d'interface** : une action indisponible faute de réseau le dit immédiatement et explicitement. Elle ne grise pas silencieusement, elle n'échoue pas après coup, elle n'enfile pas la requête « au cas où ».

### 11.2 Arbre de décision

S'arrêter à la première réponse « oui » :

1. Dépend d'un tiers externe ? → **D**
2. Modifie du référentiel partagé entre établissements, ou la relation éditeur–client ? → **C**
3. Peut produire un conflit si deux utilisateurs du même établissement l'exécutent simultanément (ressource unique, numérotation, décrément, effet monétaire) ? → **B**
4. Sinon → **A**

En cas de doute, **classer plus strictement**. Une entité indûment classée A produit des incohérences silencieuses découvertes trois mois plus tard en pleine clôture. Une entité indûment classée B produit une frustration immédiate, visible et corrigeable.

### 11.3 Classement de référence

**Classe C — référentiel** : établissements, modules activés, **classement étoiles**, catégories, unités, formules et barèmes, catalogue et prix, paramétrage fiscal et clés FNE, branding, utilisateurs, **attribution de rôles**, enrôlement d'appareil, jetons QR de table.

**Classe B — sérialisation par établissement** : création/modification/annulation de réservation, arrhes, no-show, **attribution d'unité**, check-in, check-out, prolongation, changement d'unité, **rebascule de palier de passage**, annulation d'une ligne envoyée, remise, ouverture/fermeture/transfert/division de table, **validation d'une commande QR**, tous mouvements de stock, ouverture de shift, **encaissement espèces**, sortie de caisse, comptage, **clôture journalière**, mise hors service d'une unité, numérotation de document interne.

**Classe A — hors ligne sûr** : consultation du planning et du catalogue (lecture, fraîcheur affichée), **ajout d'une ligne de commande**, envoi cuisine, marquage servi, modification d'une ligne non envoyée, note interne, préférence client, photo, **extraction OCR d'une pièce d'identité**, ajout d'accompagnant, statut ménage, incident de maintenance, intervention, alerte de seuil de stock, ouverture de tiroir (tracée), **réception d'une commande QR en état `À_CONFIRMER`**.

**Classe D — autorité externe** : certification FNE, avoir certifié, encaissement Mobile Money et carte, encaissement d'abonnement.

**Cas particulier** : le calcul de la taxe de nuitée est de classe A (déterministe, local) ; son **inscription sur une facture** est de classe D.

### 11.4 Cas pièges traités explicitement

**Le statut d'unité n'est pas une donnée libre.** « Occupée » et « Réservée » sont **dérivés** des occupations, jamais posés à la main. Seul le sous-statut ménage (à nettoyer / propre / maintenance) est librement modifiable et de classe A. Les confondre produit des doubles attributions.

**L'écriture orpheline — le conflit le plus fréquent.** Un serveur ajoute une consommation hors ligne sur le séjour 412 ; pendant ce temps la réception fait le check-out et certifie la facture. Au retour du réseau, la consommation arrive sur un séjour clos et facturé.
→ **File de réconciliation avec résolution humaine obligatoire.** Jamais de rejet silencieux, jamais d'ajout d'office. Le gérant tranche : avoir et refacturation, prise en charge, ou rattachement au séjour suivant. Aggravé par le fait que l'avoir FNE se fait par quantité (§9.4). **Écran obligatoire, testé en priorité.**

**Les horloges des terminaux ne sont pas fiables.** Un téléphone d'entrée de gamme dérive et le personnel change l'heure. Chaque écriture porte un horodatage client (indicatif, ordre d'affichage local) **et** reçoit un horodatage d'autorité à l'arrivée. Toute logique métier, tout calcul fiscal, toute clôture et **tout calcul de durée de passage** s'appuient sur l'horodatage d'autorité. Alerte au-delà de 5 minutes de dérive.

**Le passage aggrave la sensibilité à l'horloge.** Un passage facturé à l'heure repose sur des timestamps. Le début d'occupation est posé par le serveur au check-in ; si le check-in est fait hors ligne en mode C, c'est le nœud de site qui fait autorité, jamais le terminal.

**iOS n'a pas de synchronisation en arrière-plan.** La file est conçue pour être vidée **au retour au premier plan par défaut**, sur toutes les plateformes. `BGTaskScheduler` (iOS) et `WorkManager` (Android) sont des optimisations, jamais des hypothèses.

### 11.5 Règles d'implémentation

1. **UUID v7 généré côté client** sur toute écriture, classes A et D comprises. Le serveur déduplique. C'est ce qui rend le rejeu inoffensif.
2. **Journal d'événements append-only par établissement**, à séquence monotone.
3. **Dernier-écrit-gagne autorisé uniquement** sur les entités A sans conséquence (statut ménage). Nulle part ailleurs.
4. **Aucune donnée B, C ou D en cache d'écriture** sur un terminal. Ces entités sont en lecture seule côté client.
5. **Purge du cache à la déconnexion** ; chiffrement au repos sur mobile — ce sont des données d'identité de clients.
6. **Indicateur de synchronisation permanent** : connecté / dégradé / hors ligne + nombre d'éléments en attente. Lisible d'un coup d'œil.

---

## 12. Sécurité

### 12.1 Fondamentaux

- **Row Level Security forcée** (`FORCE ROW LEVEL SECURITY`) sur toutes les tables, avec un rôle applicatif distinct du propriétaire des tables. Test d'intégration qui échoue si une table n'a pas de politique.
- `SET LOCAL app.current_tenant` posé **dans chaque transaction**, jamais à l'ouverture de connexion — avec un pool, c'est la différence entre l'isolation et la fuite.
- Coffre chiffré **par tenant** pour les clés FNE et les secrets d'agrégateur.
- Aucun secret dans le binaire Tauri (décompilable).
- Sauvegardes quotidiennes + PITR, avec **restauration réellement testée chaque trimestre**.

### 12.2 Terminaux du personnel

Le verrouillage par adresse MAC est **techniquement impossible** : iOS 14 et Android 10 randomisent la MAC par réseau, et une application Android ne peut pas lire la MAC matérielle. Mécanismes retenus à la place :

| Mécanisme | Garantie |
|---|---|
| **Enrôlement d'appareil** — le gérant approuve l'appareil une fois ; une paire de clés est générée dans le Keystore/Keychain et signe chaque requête | Lien fort compte ↔ appareil, non transférable, survit au changement de réseau |
| **Attestation d'intégrité** — Play Integrity, DeviceCheck + App Attest | Application non modifiée, appareil non émulé |
| **Liste blanche et révocation** depuis le back-office | Coupure immédiate au départ d'un employé |
| **Géorepérage souple** — rayon paramétrable, **300 m par défaut** | Signal d'audit et alerte au gérant |

> Le géorepérage n'est **jamais bloquant sur une action critique**. Le GPS en zone bâtie donne 10 à 50 m d'erreur, se dégrade sous toiture métallique et se falsifie. Une position simulée détectée (`isMock`) déclenche une alerte, pas un refus. Un caissier qui ne peut pas encaisser parce que le GPS dérive est un client perdu.

---

## 13. Stack technique

**Règle générale : dernières versions stables, vérifiées puis figées.** À l'initialisation, vérifier sur les registres officiels et figer par lockfiles chaque brique — Rust, Actix Web, sqlx, utoipa, Nuxt 4, Tailwind 4, Tauri v2, PostgreSQL, Redis, Garage. Revue mensuelle groupée, aucune montée majeure pendant un incrément.

### 13.1 Backend — Actix Web (Rust)

- **Monolithe modulaire, microservices-ready** : un crate par domaine, interfaces par traits, **un schéma PostgreSQL par module**, communication inter-modules par appels de traits et par événements — jamais par jointures SQL entre schémas de modules distincts.
- **Trois familles de crates, hiérarchie de dépendance stricte** :

  | Famille | Contenu | Peut dépendre de |
  |---|---|---|
  | `socle/` | etablissements, comptes, caisse, fiscalite, documents, synchronisation, pilotage, editeur | `socle/` uniquement |
  | `capacites/` | stocks · *(production, livraison, commerce, fidélité, devis : non implémentées)* | `socle/` |
  | `verticales/` | hebergement, restauration, bar, pressing | `socle/`, `capacites/` |

  **Règle de CI non négociable : aucun crate de `socle/` ne dépend d'un crate de `verticales/`.** Un test structurel le vérifie et fait échouer le build. C'est ce qui empêche l'hôtellerie de contaminer le noyau, et donc ce qui garde ouverte l'extension à d'autres activités.
- **utoipa + utoipa-swagger-ui** : schémas dérivés, `#[utoipa::path]`, Swagger UI protégée hors production, spec sur `/api-docs/openapi.json`.
- **Le contrat OpenAPI est la source de vérité** : le client TypeScript est généré en CI depuis ce contrat, jamais écrit à la main.
- **`sqlx`** : requêtes vérifiées à la compilation (`cargo sqlx prepare`), migrations versionnées, une migration appliquée n'est jamais modifiée.

**Crate `domain` partagé** : moteur fiscal, calculs de formules et de barèmes, validation, types métier. Consommé par l'API, par le nœud de site et par la partie Rust de Tauri. **Une seule implémentation du calcul de la taxe de nuitée**, pas trois. C'est le principal bénéfice du choix Rust de bout en bout.

**Mesures de vélocité** (obligatoires, cycle 1) : un **module doré** écrit à la main — entité, repository, service, handler, tests — servant de patron avant toute génération assistée ; linker `mold`, `sccache`, `cargo-watch` ; `debug = "line-tables-only"` en profil dev ; découpage en crates pour limiter la recompilation. Pas de généricité prématurée : du code concret se refactore, une abstraction prématurée se subit.

### 13.2 Frontière monolithe / microservices

« Microservices-ready » signifie exactement ceci, et rien de plus :

1. Un schéma Postgres par module ; aucune requête ne joint deux schémas de modules différents. Les lectures inter-modules passent par un trait exposé.
2. Toute transition d'état métier écrit un **événement outbox** dans la même transaction SQL. Le jour où un module est extrait, ses consommateurs le sont déjà par événements.
3. Aucune transaction SQL ne couvre deux modules. Les opérations inter-modules sont des sagas simples avec compensation explicite.
4. Chaque crate expose son API interne sous forme de trait ; les dépendances sont injectées.

Aucun service n'est extrait au MVP. Aucune file de messages n'est introduite au MVP — l'outbox est consommé par un worker in-process.

### 13.3 Applications — Nuxt 4 + Tauri v2

- **Une seule application** pour tous les rôles métier, en mode SPA (SSR désactivé, obligatoire sous Tauri). Cibles : Windows, macOS, Linux, Android, iOS.
- **Chargement paresseux par module** : un serveur de salle ne télécharge pas le code du back-office.
- **RBAC cumulatif** : un utilisateur porte N rôles, ses permissions sont l'union. Un **sélecteur de contexte** permanent (quel établissement, quel poste) évite d'afficher tout simultanément.
- **`PlatformAdapter`** : aucune invocation directe de `window.__TAURI__` dans un composant. Impression, scan, OCR, stockage sécurisé, notifications, géolocalisation, réseau passent par cette interface, avec implémentations `desktop`, `android`, `ios`, `web`. Une capacité absente sur une plateforme le **dit explicitement** à l'utilisateur.
- **Tailwind 4** + design system minimal (tokens, composants canoniques), **mode sombre dès le premier écran**, **i18n fr/en** avec fr par défaut et aucune chaîne en dur.

**Surfaces web séparées** (Nuxt, hors Tauri) : page publique de commande par QR (SSR), console éditeur.

### 13.4 Plugins natifs à écrire — budget acté

| Besoin | État Tauri v2 | Effort |
|---|---|---|
| Notifications push distantes (APNs / FCM) | Le plugin officiel ne couvre que les notifications locales | 2–3 sem. |
| Synchronisation en arrière-plan | Non couvert (`BGTaskScheduler` / `WorkManager`) | 1–2 sem. |
| Impression thermique Bluetooth | Non couvert | 2 sem. |
| Attestation d'intégrité | Non couvert | 1 sem. |
| OCR de pièce d'identité (Vision / ML Kit) | Non couvert | 1–2 sem. |
| Stockage sécurisé de clés | Partiel | 0,5 sem. |

> **La mise à jour hors store n'existe pas sur mobile.** Le plugin updater de Tauri exclut explicitement les cibles Android et iOS. Conséquences actées : desktop → serveur de mise à jour auto-hébergé ; Android → distribution APK directe possible, mécanisme d'installation à écrire ; **iOS → aucune installation hors App Store**, toute mise à jour du binaire passe par la revue Apple. La mise à jour des assets web dans le WebView est un **correctif d'urgence, jamais le canal de livraison normal**.

### 13.5 Données et infrastructure

- **PostgreSQL 17** — source de vérité unique et durable. Contraintes d'exclusion GiST pour la disponibilité (§5.1). RLS forcée.
- **Redis** — éphémère reconstructible uniquement : sessions, file de certification FNE, verrous distribués, limitation de débit, cache de catalogue. Jamais de donnée métier durable.
- **Garage (API S3)** — logos, PDF de factures, photos de catalogue, pièces d'identité chiffrées. Buckets créés au provisionnement, une clé d'accès par usage. Rétention alignée ARTCI.
- **Docker + Compose** en développement et pour le paquet auto-hébergé. Kubernetes hors sujet.
- Observabilité : logs structurés avec corrélation par requête, Sentry, sonde `/health`, télémétrie de version pour le parc auto-hébergé.

### 13.6 Paiements

Trait `PaymentProvider { create_checkout, verify_webhook, refund }`. Implémentation MVP : **CinetPay** (Wave CI, Orange Money CI, MTN MoMo CI, Moov Africa CI, cartes). Intégrations directes en phase 2 derrière le même trait, bascule par configuration.

Règles : session créée côté serveur, **webhook validé par signature HMAC**, jamais de confiance dans la redirection client seule, idempotence sur le webhook.

Dossier marchand : RCCM, CNI du gérant, RIB ivoirien, justificatif de domicile professionnel. Activation 3 à 7 jours ouvrables — **à déposer en semaine 1**.

---

## 14. Prêt pour la suite — sans le construire

« Prêt » = choix de modèle de données et d'interfaces quasi gratuits aujourd'hui, évitant une migration douloureuse demain. **Rien de tout cela n'est développé au MVP.**

1. **Adaptateurs de juridiction** — trait `JurisdictionAdapter { compute_taxes, required_document_fields, emission_channel, certify, remittance_reports }`. Un seul adaptateur au MVP (`CoteDIvoire`). Aucune règle fiscale ne vit ailleurs.
2. **Devises dynamiques** — montants en **entiers d'unité mineure + code ISO 4217** porté par l'établissement (XOF, 0 décimale). Prêt pour une expansion hors zone CFA.
3. **Modules d'activité extensibles** — le référentiel de modules est une table ; ajouter `SPA`, `BOULANGERIE`, `SUPERETTE` ou `QUINCAILLERIE` est de la configuration, pas une migration.
4. **Capacités extensibles** — référentiel distinct des modules (§4.1). Seule `STOCK` est implémentée ; `LIVRAISON`, `PRODUCTION`, `COMMERCE_EN_LIGNE`, `FIDELITE`, `DEVIS` et `COMPTES_CLIENTS` sont déclarées et refusées.
5. **Profils de stock** — colonne `profil_stock ∈ {AUCUN, SIMPLE, VALORISE, DETAILLE}` sur le module d'activité. Seul `SIMPLE` est implémenté. Les profils supérieurs (valorisation, commandes fournisseurs, variantes, codes-barres, lots) sont ce que réclament la boulangerie, la supérette et la quincaillerie.
6. **Quantités décimales** — toute `quantite` de ligne de vente ou de mouvement de stock est en `NUMERIC`, **jamais en entier**. Un hôtel vend 1 bière ; une quincaillerie vend 2,3 mètres de fer ; une boulangerie achète 47,5 kg de farine. Passer d'entier à décimal après mise en production imposerait de migrer toutes les lignes.
7. **Unité de mesure** — colonne `unite_mesure` obligatoire sur `article`, défaut `unite`. Table de conversion multi-unités créée, non exploitée.
8. **Coût unitaire** — colonne `cout_unitaire` nullable sur `mouvement_stock`, **jamais renseignée au MVP**. Sans elle, aucune valorisation rétroactive ne serait possible.
9. **Codes-barres et variantes** — colonnes `code_barre` et `article_parent_id` nullables sur `article`. Non utilisées.
10. **Documents commerciaux** — tables `devis` et `document_commercial`, cycle `brouillon → émis → accepté → converti | expiré`. La quincaillerie et le B2B en dépendent lourdement ; l'hôtellerie s'en passe.
11. **Formules de location extensibles** — le moteur ne connaît que des barèmes à paliers et des contraintes de plage ; une formule nouvelle est une ligne de données.
5. **Canal d'émission fiscale** — `EmissionChannel { FneApi, Terne }` et colonne `rne_ref` nullable (§9.8).
6. **Passerelle FNE** — `FneGateway` avec implémentations `Partenaire` et `Direct`.
7. **Journal d'événements outbox — grand livre permanent, pas file de messages.** C'est la provision la plus déterminante pour la comptabilité future. Trois règles indissociables :
   - **Rétention illimitée.** L'événement est publié, jamais purgé. Le réflexe « consommer et supprimer » d'une file de messages détruirait l'historique comptable.
   - **Charge utile financière complète et dénormalisée.** Un événement d'encaissement porte le montant, le mode, la contrepartie, la ventilation de taxes et la référence du document — pas seulement un identifiant. Sinon il faudra rejoindre des tables qui auront changé.
   - **Immuable.** Un événement écrit ne se modifie jamais ; une correction est un nouvel événement.
   Base des métriques, des webhooks futurs, de l'extraction en services (§13.2) **et de la génération d'écritures comptables rétroactives**.
7 bis. **Correspondance comptable** — table `mapping_comptable {tenant_id, type_evenement, compte_debit, compte_credit, journal}` et table `exercice_comptable {debut, fin, statut}`. Une période close n'accepte plus d'écriture : c'est une contrainte distincte de la clôture journalière et de la certification fiscale, et elle interagit avec la réconciliation des écritures orphelines (§11.4). **Tables seulement.**
8. **Nœud de site** — l'API locale et l'API cloud exposent le même contrat ; le mode C est une configuration, pas un produit.
9. **Partenaires externes** — table `partenaire` avec `tenant_id` **nullable** : un restaurant, une quincaillerie ou un pressing extérieur avec qui l'établissement travaille est le cas normal ; qu'il ait lui-même un compte Kaya est l'enrichissement. Plus `demande_partenaire` et le compte de compensation. **C'est la provision la plus transversale du modèle** — elle sert la sous-traitance hôtelière, les fournisseurs d'une supérette et les grossistes d'une quincaillerie, et c'est la seule dont l'utilité croît avec le nombre de clients.
10. **Contrats de location et prestations incluses** — tables `contrat`, `caution`, `charge_locative` pour les résidences meublées en incrément 3, et `prestation_incluse` sur la formule (petit-déjeuner, blanchisserie, conciergerie). Voir question ouverte P-11.
18. **Plans d'abonnement** — `plan` + `features`, calcul de palier sur une **métrique abstraite `unite_facturable` définie par la verticale** (§15.1), jamais sur « chambre ».
19. **Comptes clients entreprises** — `compte_client`, `encours`, `condition_reglement` pour la facturation à crédit.
20. **Site web vitrine** — le catalogue public du QR partage son modèle avec la future vitrine offerte aux clients.
21. **IoT et contrôle d'accès** — table `dispositif` et trait `AccessController`, pour l'ouverture d'unité en phase 3. **Contrainte de conception à respecter dès maintenant** : tout mécanisme d'ouverture devra disposer d'un canal hors ligne (code à usage unique dérivé d'un secret partagé, validable sans réseau). Une porte qui ne s'ouvre pas parce que le réseau est tombé est un incident grave.

> **Ces provisions ouvrent l'extension à d'autres activités** — boulangerie, quincaillerie, supérette, gare routière, livraison — et à la mise en réseau des établissements entre eux. L'analyse complète, les fractures réelles et le séquencement figurent dans `Kaya_Vision_Plateforme.md`. **Ce document est fermé jusqu'au jalon J1** : les provisions ci-dessus sont le seul impact autorisé sur le MVP.

---

## 15. Modèle économique

### 15.1 Grille d'abonnement

Deux modes coexistent, **le client bénéficie automatiquement du moins-disant**. Le calcul est fait sur le **nombre total d'`unite_facturable` du tenant, tous établissements confondus**. Le nombre d'établissements n'a **aucun** impact sur le prix.

> **`unite_facturable` est une métrique abstraite définie par la verticale**, jamais « chambre » en dur : la chambre pour l'hébergement, le point de vente pour la restauration et le commerce, le véhicule pour la livraison. Le moteur de tarification ne connaît qu'un nombre ; la verticale dit ce qu'on compte. Au MVP, la seule implémentation est « chambre » et le comportement est strictement identique à une facturation à la chambre.

| Mode | Règle |
|---|---|
| **Forfait par palier** | ≤ 25 unités : 20 000 FCFA/mois · 26 à 50 unités : 40 000 FCFA/mois · > 50 unités : 1 000 FCFA/unité/mois |
| **Au compteur** | 1 000 FCFA/unité/mois, disponible à tout niveau |

Grille effective résultante :

| Unités du tenant | Mode appliqué | Montant mensuel |
|---|---|---|
| 2 | Compteur | 2 000 |
| 10 | Compteur | 10 000 |
| 17 (Deloria) | Compteur | 17 000 |
| 20 | Compteur / Forfait (égalité) | 20 000 |
| 25 | Forfait palier 1 | 20 000 |
| 30 | Compteur | 30 000 |
| 40 | Compteur / Forfait (égalité) | 40 000 |
| 50 | Forfait palier 2 | 40 000 |
| 80 | Compteur | 80 000 |

Tous les seuils, montants et paliers sont **éditables dans la console éditeur**, sans déploiement.

**Frais d'installation** : montant à décider (B-04). Couvrent paramétrage fiscal, reprise de données, formation et déplacement. Facturés indépendamment de toute gratuité d'abonnement.

### 15.2 Pilote

Deloria : **abonnement gratuit 6 mois** à compter de la mise en production de l'incrément 1, avec **engagement d'abonnement payant signé dès la phase 0** pour la période suivante. Un pilote gratuit sans engagement est un signal ambigu pour un investisseur ; l'engagement signé vaut plus qu'un témoignage.

### 15.3 Trésorerie — lecture honnête

| Échéance | Événement | Encaissement |
|---|---|---|
| M0–M4 | Construction incrément 1 | 0 |
| M4 | Livraison Deloria, début de la gratuité | Frais d'installation |
| M4–M10 | Gratuité Deloria, démarchage, incrément 2 | Frais d'installation des nouveaux clients |
| M10 | Fin de gratuité Deloria | ~17 000/mois |

Six mois de trésorerie ne couvrent pas cette trajectoire. **Une source de revenus de transition doit être décidée en semaine 2** (B-03), pas au mois 5 en situation d'urgence.

### 15.4 Seuil de rentabilité

Coûts mensuels estimés : rémunération 700 000 + infrastructure 100 000 + outils 80 000 + comptable et juridique 70 000 ≈ **950 000 FCFA/mois**.

Seuil : environ **60 unités facturées × 15 clients**, soit ~950 000 FCFA de revenu récurrent. Trajectoire réaliste : M12 → 250 000–400 000 · M18 → 700 000–1 200 000 · M24 → 1 500 000–2 200 000.

---

## 16. Roadmap

Estimation du périmètre complet : **≈ 70 semaines-homme** en solo, majoration Rust incluse. La livraison est donc découpée en trois incréments mis en production, chacun apportant une valeur autonome.

### Phase 0 — Verrouillage (S1–S3)

Aucun code produit. Livrables :

- Courrier DGI (agrément éditeur + questions annexe A) · consultation fiscaliste (moteur de taxes, arbitrage taxe de nuitée sur passage et demi-journée) · démarche ARTCI · dossier CinetPay · convention écrite avec le partenaire FNE
- **Contrat pilote Deloria signé**, avec engagement post-gratuité
- **Une journée complète d'observation** à la réception et au bar de Deloria — dont un relevé réel des formules passage et demi-journée pratiquées, avec leurs barèmes
- Spikes chronométrés, 2 jours chacun, résultat écrit : certification FNE de test · impression thermique depuis Tauri desktop et Android · build Tauri v2 + Nuxt 4 sur Windows, Android et iOS · plugin natif minimal Swift et Kotlin · **module doré** Actix + sqlx + utoipa + RLS · contrainte d'exclusion GiST sur intervalles horodatés
- `constitution.md` Spec Kit · modèle de domaine et classification hors-ligne de chaque entité

**Critère de sortie** : réponses écrites sur l'agrément, six spikes verts. Si le spike iOS échoue, iOS glisse en incrément 3 sans discussion.

### Incrément 1 — « Deloria sans papier » (S4–S17)

Socle (multi-tenant, RLS, RBAC, i18n, mode sombre, design system, OpenAPI, CI) · Établissements et modules d'activité · Unités, **formules complètes dont passage et demi-journée**, disponibilité en intervalles · Séjours et note · Points de vente restaurant, bar, pressing · Clients extérieurs · Caisse et clôture · **FNE et taxe de nuitée** · Impression · **Desktop uniquement**.

**Jalon J1** : Deloria abandonne le cahier papier. Double exploitation de 3 semaines avant bascule.

### Incrément 2 — « Mobilité et clients » (S18–S31)

Réservations et planning · **Enregistrement accéléré et OCR** · **Commande par QR** · **Android** (plugins natifs, enrôlement d'appareil, push) · Stocks · Tableau de bord multi-établissements · Console éditeur et abonnements CinetPay · Métriques.

**Jalon J2** : le produit est vendable à un second client. Dossier de levée constitué.

### Incrément 3 — « Échelle » (S32–S45)

**iOS** · Contrats, cautions et charges pour les résidences meublées · **Nœud de site LAN** · Paquet auto-hébergé durci · Second adaptateur de juridiction · Comptes clients entreprises.

### Règle de dérive

Aucune marge n'est prévue — c'est volontaire, le plan est un test. Si l'incrément 1 dérape de plus de 3 semaines, la décision est de **sortir les stocks et le tableau de bord multi-sites de l'incrément 2**, jamais de repousser la livraison au pilote.

---

## 17. Risques et mitigations

| Risque | Prob. | Impact | Mitigation |
|---|---|---|---|
| Trésorerie épuisée avant le premier revenu | Élevée | Fatal | Source de transition décidée en S2 (B-03) ; frais d'installation encaissés d'avance |
| Périmètre 70 semaines contre 26 de trésorerie | Certaine | Fatal | Livraison par incréments ; jalon J1 à S17 ; règle de dérive |
| Agrément DGI long, contrôle technique sur site | Moyenne | Élevé | Courrier en S1 ; partenaire agréé comme pont ; abstraction `FneGateway` |
| Dépendance au partenaire FNE (disponibilité, priorités, tarif) | Moyenne | Élevé | Convention écrite avec SLA ; agrément propre en parallèle ; bascule par configuration |
| Double certification FNE (pas d'idempotence côté DGI) | Élevée | Élevé | État `INDETERMINEE` jamais rejoué ; écran de rapprochement manuel |
| Fiscalité des formules infra-journalières non tranchée | Élevée | Élevé | Drapeau et règle de conversion par formule ; aucune valeur en dur ; arbitrage fiscaliste en S2 |
| Plugins natifs mobiles (8–11 sem. cumulées) | Élevée | Élevé | Chiffrés et planifiés (§13.4) ; dégradation gracieuse ; iOS en incrément 3 |
| Résistance du personnel au traçage des passages | Élevée | Élevé | Saisie d'un passage en moins de 30 s ; conduite du changement avec le gérant ; journal d'audit présenté comme outil du propriétaire |
| Erreur de calcul fiscal répliquée chez plusieurs clients | Faible | Fatal | Validation fiscaliste ; tests dorés sur jeux de cas figés ; double vérification manuelle les 3 premiers mois |
| Support à distance sur Abengourou | Élevée | Moyen | Télémétrie, journaux remontés, mise à jour à distance dès l'incrément 1 |
| Le pilote demande du sur-mesure en continu | Élevée | Moyen | Registre des demandes ; périmètre contractuel ; réponse systématique « paramètre ou phase 2 » |
| Épuisement du développeur solo sur 15 mois | Élevée | Fatal | Un jour off par semaine ; incréments courts ; jalons célébrés |
| Auto-hébergement : coût de support sous-estimé | Moyenne | Moyen | Versions N et N-1 seulement ; télémétrie obligatoire ; contrat distinct |
| Coupure secteur et corruption Postgres (modes B et C) | Moyenne | Élevé | Onduleur au cahier des charges d'installation ; arrêt propre ; sauvegarde horaire |
| Sur-périmètre | Élevée | Élevé | Règle de gouvernance §3.2 ; « prêt » ≠ « construit » (§14) ; priorités P0/P1/P2 des user stories font foi |

---

## 18. KPIs et pilotage

Instrumentation dès le premier cycle : tous les indicateurs sont calculés depuis le journal d'événements, aucun ne dépend d'un pointage manuel.

**Indicateurs d'adoption du pilote** (mesurés à M5–M6, ils déclenchent des ajustements) :

| Métrique | Vert | Rouge (plan d'action) |
|---|---|---|
| Durée de clôture journalière | < 15 min | > 40 min |
| Consommations non facturées / 30 j | 0 | > 3 |
| Factures certifiées du premier coup | > 98 % | < 90 % |
| Documents en état `INDETERMINEE` / semaine | < 1 | > 5 |
| Durée d'enregistrement, client connu | < 60 s | > 3 min |
| Durée d'enregistrement d'un passage | < 30 s | > 90 s |
| Part des passages saisis dans le système | > 95 % | < 70 % |
| Sessions hors ligne se réconciliant sans intervention | > 99 % | < 95 % |
| Écarts de caisse > 1 000 F / semaine | < 1 | > 3 |
| Connexions quotidiennes du propriétaire | > 4/semaine | < 1/semaine |

**Indicateurs d'exploitation éditeur** : tenants actifs, unités facturées, MRR, taux de rétention, délai moyen de résolution d'incident, part du parc en version N.

---

## 19. Prochaines étapes (4 semaines)

1. **Courrier DGI** avec les 10 questions de l'annexe A ; demande d'agrément éditeur déposée.
2. **Fiscaliste** : moteur de taxes validé, et arbitrage explicite de la taxe de nuitée sur passage et demi-journée (B-02).
3. **Convention partenaire FNE** signée : périmètre, SLA, responsabilité, tarif, réversibilité, NCC de rattachement.
4. **Dossier CinetPay** déposé ; démarche ARTCI engagée.
5. **Contrat pilote Deloria** signé, avec engagement post-gratuité et référent nommé.
6. **Journée d'observation terrain** à Abengourou : relevé des formules passage et demi-journée réellement pratiquées et de leurs barèmes, cartographie des cartes à codes-barres, décision A-04 (classe du stock).
7. **Décision B-03** (source de revenus de transition) — la plus urgente.
8. **Six spikes techniques** de la phase 0, résultats écrits.
9. **Vérification et gel des dernières versions stables** de toute la stack.
10. Maquettes : 6 écrans réception, 4 point de vente, 3 caisse, 4 direction, 3 console éditeur.

---

## Annexe A — Questions à adresser à la DGI

1. **Agrément éditeur** : délai d'instruction ? Modalités et préavis du contrôle technique sur site ? Un éditeur peut-il commercialiser pendant l'instruction en s'appuyant sur un partenaire déjà agréé ?
2. **Idempotence** : existe-t-il un endpoint de consultation par référence externe permettant de savoir si une facture a été certifiée après un timeout ?
3. **Indisponibilité** : quel comportement attendu quand la plateforme est injoignable ? Mode différé autorisé, délai de régularisation, mention à porter sur le document remis au client entre-temps ?
4. **Avoirs** : les avoirs hors facture et le type « Facture 3R » sont-ils disponibles par API ou seulement via l'interface web ? Comment traiter une remise commerciale partielle sur une ligne, l'avoir étant par quantité ?
5. **Granularité hôtelière** : FNE par prestation ou consolidée au check-out ? Statut juridique de la note provisoire quotidienne ?
6. **Cas particuliers** : extras réglés immédiatement au bar par un tiers non-résident ; acomptes et arrhes à la réservation ; réservations OTA (qui facture qui, commission, cartes virtuelles).
7. **Taxe de nuitée** : champ FNE dédié ou ligne de taxe additionnelle ? Quel code de taxe ?
8. **Formules infra-journalières** : la taxe communale de nuitée est-elle due sur une occupation de type passage (1 à 4 h) ou demi-journée ? Si oui, selon quelle règle de conversion ?
9. **Multi-établissements** : un groupe avec plusieurs établissements sous un même NCC utilise-t-il une clé API unique ou une par établissement ?
10. **Archivage** : l'éditeur peut-il assurer l'archivage légal pour le compte du contribuable ? Format et durée exacte par régime ?

## Annexe B — Décisions ouvertes

| # | Décision | Échéance |
|---|---|---|
| B-01 | Hébergement en Côte d'Ivoire ou en Europe ? Arbitrage latence / fiabilité / transfert transfrontalier ARTCI | S3 |
| B-02 | Taxe de nuitée sur passage et demi-journée : règle de conversion par défaut | S3 (fiscaliste) |
| B-03 | **Source de revenus de transition** — la plus urgente | S2 |
| B-04 | Montant des frais d'installation | S6 |
| B-05 | Classe du stock : A ou B ? Le stock sert-il à détecter le vol ou à réapprovisionner ? | S4 (pilote) |
| B-06 | Nom définitif, marque, entité juridique, dépôt | S6 |
| B-07 | Barèmes de passage réels du pilote (seeds) | S3 (atelier terrain) |
| B-08 | Politique de support : horaires, canaux, SLA par formule | S14 |
| B-09 | Second pays cible et calendrier de l'adaptateur de juridiction | S30 |
| ~~**B-10**~~ | ✅ **CLOSE le 2026-08-03**, à l'atelier terrain, **avant** la migration `0034` du cycle SEJ. **La taxe de nuitée est due par nuitée et par SÉJOUR, jamais par personne.** La question posée était celle de l'**exonération par personne** — un enfant en bas âge, un résident, un séjour au-delà d'un seuil — et elle **tombe avec l'arbitrage** : la taxe n'étant pas assise sur les personnes, il n'y a rien à exonérer personne par personne. **Aucune colonne de motif n'est due**, et `hebergement.accompagnant` n'en porte pas. Le nombre de personnes est enregistré au constat (`nombre_personnes`) à titre **indicatif** : il documente le séjour, il n'entre dans aucun calcul. ⚠️ **À ne pas confondre avec B-02**, toujours ouverte : celle-ci porte sur l'axe des **nuits** — combien de nuitées un passage ou une demi-journée produit —, que B-10 ne touche pas. Les deux axes se confondaient dans l'ancienne rédaction du §9.6, et c'est ce qui rendait cette ligne difficile à lire. Sources amendées dans le même changement : `cadrage-v1.md` §9.6, `user-stories-v1.md` FIS-03, FIS-08 et son récapitulatif des paramètres | **CLOSE — 2026-08-03** |

## Annexe C — Glossaire

**Verticale** — un métier, servi en profondeur : hôtellerie, restauration, bar, pressing. C'est une *colonne*. Chaque verticale a ses écrans, ses règles et son vocabulaire propres, et vit dans `crates/verticales/` · **Horizontale** (ou *transverse*, ou *capacité*) — une fonction utile à plusieurs métiers : stock, livraison, partenaires. C'est une *ligne* qui traverse les colonnes. Elle est écrite une seule fois et vit dans `crates/capacites/` · **Socle** — ce dont tout établissement a besoin quel que soit son métier : comptes, fiscalité, caisse, documents, audit, hors-ligne. Il ne connaît aucune verticale. **Règle de conception** : face à un besoin nouveau, se demander si deux métiers en auraient besoin. Si oui, c'est une horizontale, jamais une verticale.

**Établissement** unité d'exploitation physique appartenant à un tenant · **Module d'activité** capacité métier activable sur un établissement (hébergement, restauration, bar, pressing, salle de réunion) · **Unité louable** chambre, appartement ou salle · **Formule** mode de location et son barème (nuitée, passage, demi-journée, mensuel) · **Passage** location horaire à paliers dégressifs · **FNE** Facture Normalisée Électronique · **RNE** Reçu Normalisé Électronique · **TERNE** terminal d'émission du RNE · **NCC** Numéro de Compte Contribuable · **Sticker** unité de certification consommée à chaque émission FNE · **Clearance model** modèle où l'administration valide la facture avant remise au client · **Nœud de site** serveur local dans un établissement, autoritaire sur ses données opérationnelles · **RLS** Row Level Security · **Outbox** journal d'événements métier écrit dans la transaction applicative
