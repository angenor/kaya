# Kaya — User Stories MVP par module (v1)

*Complément d'exécution du Document de cadrage v1 — Pilote : Résidence Hôtel Deloria, Abengourou — Développement : solo + Claude Code + Spec Kit*

---

## 0. Mode d'emploi

### 0.1 Avec Claude Code + Spec Kit

- **Un module = un epic = un cycle Spec Kit** : coller la section du module dans `/speckit-specify`, dérouler `/speckit-plan` en pointant le cadrage v1 (§13 stack, §14 provisions) comme contrainte, puis `/speckit-tasks` et implémentation.
- **Implémenter par tranches verticales (§0.5)**, pas module par module de bout en bout.
- Le **contrat OpenAPI est la source de vérité** : toute story qui touche l'API met d'abord à jour les annotations utoipa, puis régénère le client TypeScript (TRX-01).
- **Dernières versions stables** : à l'initialisation, le gel est fait et sourcé dans `docs/versions-gelees.md` — le consommer, ne pas revérifier ; revue mensuelle groupée.

### 0.2 Priorités

| Priorité | Signification |
|---|---|
| **P0** | Sans cette story, on ne livre pas l'incrément. L'ensemble des P0 d'un incrément constitue un produit exploitable. |
| **P1** | Dans le périmètre de l'incrément ; avant la bascule si le calendrier le permet, sinon dans les 2 semaines qui suivent (un fallback P0 existe toujours). |
| **P2** | Incrément suivant. Documentée ici uniquement quand elle contraint le modèle de données dès maintenant. |
| **PROVISION** | Modèle de données ou interface uniquement. Aucune UI, aucune logique au MVP. |

### 0.3 Personas

- **M. Koffi (propriétaire)** — possède Deloria et une résidence meublée. Consulte depuis son téléphone, veut détecter les écarts sans se déplacer. Ne saisit jamais rien.
- **Adjoua (gérante de site)** — à Deloria tous les jours. Cumule gérante, caissière et réceptionniste. Clôture la journée. C'est elle qui décide si le logiciel remplace le cahier.
- **Yao (réceptionniste)** — enregistre les arrivées, gère les passages, encaisse. Rapidité avant tout : un passage doit se saisir en moins de 30 secondes ou il sera contourné.
- **Aminata (serveuse bar/restaurant)** — Android d'entrée de gamme, réseau intermittent, saisit debout. Doit pouvoir prendre une commande sans réseau.
- **M. Diarra (comptable externe)** — vient une fois par mois, veut un export exploitable, ne veut pas apprendre le logiciel.
- **Admin éditeur (toi)** — console web, provisionne les tenants, diagnostique à distance depuis Abidjan.

### 0.4 Definition of Done (commune)

1. Critères d'acceptation couverts par des tests (unitaires + intégration sur les transitions d'état).
2. Annotations utoipa à jour ; client TypeScript régénéré sans diff manuel.
3. Migration sqlx versionnée ; `cargo sqlx prepare` vert ; seeds à jour.
4. **RLS activée et forcée** sur toute nouvelle table, avec test d'isolation multi-tenant.
5. **Classe hors-ligne déclarée** (A/B/C/D) pour toute nouvelle entité, avec le test correspondant (§0.7).
6. Événement outbox émis pour tout changement d'état métier (TRX-02).
7. **Clés i18n fr et en** externalisées ; aucune chaîne en dur.
8. **Écran vérifié en mode clair et en mode sombre.**
9. Paramètres exposés dans la configuration d'établissement quand la story dit « paramétrable ».
10. Tout document imprimé vérifié sur imprimante thermique réelle.

### 0.5 Ordre d'implémentation — 5 tranches verticales

**Incrément 1 — « Deloria sans papier » (S4–S17)**

| Tranche | Semaines | Contenu | Démo de fin de tranche |
|---|---|---|---|
| **T1 — Colonne vertébrale** | S4–S8 | TRX-01→05a (TRX-05b en fin de tranche), ETB-01→05 **+ ETB-02b/02c**, CPT-01→04, HEB-01→05, SEJ-01/02/04, SYN-01/02 | Yao enregistre un client en chambre B3 pour 2 nuits, puis un passage de 4 h en A1 — la disponibilité empêche tout chevauchement, tout est tracé. |
| **T2 — Services et note** | S9–S12 | PDV-01→06, SEJ-03/05, CAI-01→04, IMP-01/02 | Aminata prend une commande au bar hors réseau, elle s'ajoute à la note de la chambre B3 ; Adjoua encaisse, imprime un ticket, boucle son shift. |
| **T3 — Fiscalité et clôture** | S13–S17 | FIS-01→07, CAI-05/06, IMP-03, DIR-01, SYN-03/04 | Une facture Deloria est certifiée FNE avec la taxe de nuitée en ligne distincte ; la clôture journalière tombe au franc près ; l'état de reversement communal est généré. |

**Incrément 2 — « Mobilité et clients » (S18–S31)**

| Tranche | Semaines | Contenu | Démo de fin de tranche |
|---|---|---|---|
| **T4 — Mobile et QR** | S18–S25 | CPT-05/06, MOB-01→05, QRC-01→04, SEJ-06, RSV-01→05, **HEB-09** | Aminata prend les commandes sur son Android enrôlé ; un client scanne le QR de sa table, Aminata valide d'un tap ; une réservation est posée sur le planning ; un petit-déjeuner inclus se décompte sans être facturé. |
| **T5 — Pilotage** | S26–S31 | STK-01→04, DIR-02→05, ADM-01→06, MET-01→03 | M. Koffi voit ses deux établissements en temps réel depuis son téléphone ; le stock du bar se décrémente sur vente ; un tenant est provisionné et facturé depuis la console. |

**Incrément 3 — « Échelle » (S32–S45)** : iOS, contrats et cautions des résidences meublées, nœud de site LAN, paquet auto-hébergé durci, second adaptateur de juridiction, comptes clients entreprises.

### 0.6 Vue d'ensemble

| Module | Préfixe | P0 | P1 | P2/Prov. | Tranche principale |
|---|---|---|---|---|---|
| Transverse & infrastructure | TRX | 6 | 3 | 1 | T1 |
| Établissements & modules d'activité | ETB | 7 | 1 | 2 | T1 |
| Comptes, rôles & appareils | CPT | 5 | 2 | — | T1/T4 |
| Hébergement : unités & formules | HEB | 5 | 2 | 2 | T1 |
| Séjours & enregistrement | SEJ | 5 | 1 | — | T1/T2/T4 |
| Réservations | RSV | 4 | 1 | — | T4 |
| Points de vente | PDV | 6 | 2 | — | T2 |
| Commande par QR | QRC | 4 | — | — | T4 |
| Caisse & encaissements | CAI | 6 | 1 | — | T2/T3 |
| Fiscalité & documents | FIS | 7 | 1 | 3 | T3 |
| Synchronisation & hors-ligne | SYN | 4 | — | — | T1/T3 |
| Impression & documents | IMP | 3 | 1 | — | T2/T3 |
| Stocks | STK | 4 | — | — | T5 |
| Mobile (Tauri Android/iOS) | MOB | 5 | 1 | — | T4 |
| Direction & pilotage | DIR | 5 | — | — | T3/T5 |
| Console éditeur & abonnements | ADM | 6 | — | — | T5 |
| Métriques | MET | 2 | 1 | — | T5 |

### 0.7 Tests hors-ligne obligatoires

Toute entité déclare sa classe (cadrage §11) et embarque les tests suivants :

- **Classes B, C, D** : un test qui **échoue si l'opération est atteignable depuis un chemin de code exécutable hors ligne**.
- **Classe A** : test de rejeu — la même écriture envoyée trois fois produit un seul enregistrement. Test de désordre — trois écritures appliquées dans les six ordres possibles produisent le même état final.
- **Classe D** : test de double soumission au retour du réseau.
- Toute entité rattachée à un séjour : test du **scénario orphelin** (SYN-03).

---

## Module TRX — Transverse & infrastructure

**TRX-01 — Contrat OpenAPI et génération du client (P0)**
- Handlers annotés `#[utoipa::path]` ; schémas `ToSchema`/`IntoParams` ; spec exposée sur `/api-docs/openapi.json` ; Swagger UI protégée hors production.
- CI : génération du client TypeScript depuis la spec ; **diff non commité = build en échec**.
- Le client n'est jamais édité à la main.

**TRX-02 — Journal d'événements métier (outbox) (P0)**
- Toute transition d'état insère `{type, agrégat, tenant_id, etablissement_id, payload, horodatage}` dans **la même transaction SQL**.
- Worker de publication in-process ; consommateurs idempotents (notifications, métriques).
- Aucune file de messages externe au MVP — l'outbox est la frontière de découplage (cadrage §13.2).
- ⚠️ **L'outbox est un GRAND LIVRE PERMANENT, pas une file de messages.** Trois règles indissociables, à poser dès la première migration :
  1. **Rétention illimitée** — un événement publié est marqué publié, jamais supprimé. Une purge après publication détruirait toute possibilité de comptabilité rétroactive.
  2. **Charge utile financière complète et dénormalisée** — un encaissement porte montant, mode, contrepartie, ventilation de taxes et référence de document, pas un simple identifiant. Sinon il faudra rejoindre des tables qui auront changé.
  3. **Immuable** — un événement ne se modifie jamais ; une correction est un nouvel événement.
- Test obligatoire : après publication, l'événement est toujours lisible et son payload permet de reconstituer l'opération sans consulter aucune autre table.

**TRX-02b — Provisions comptables (PROVISION)**
- Tables `mapping_comptable {tenant_id, type_evenement, compte_debit, compte_credit, journal}` et `exercice_comptable {debut, fin, statut}`.
- Une période close n'accepte plus d'écriture — contrainte **distincte** de la clôture journalière et de la certification fiscale, et qui interagira avec la réconciliation des écritures orphelines (SYN-03).
- **Tables seulement. Aucune UI, aucune logique.** Elles préparent SYSCOHADA en phase 2 sans imposer de migration.

**TRX-03 — Multi-tenant et RLS forcée (P0)**
- Chaque table porte `tenant_id` ; `ALTER TABLE ... ENABLE ROW LEVEL SECURITY` **et** `FORCE ROW LEVEL SECURITY` ; rôle applicatif distinct du propriétaire des tables.
- `SET LOCAL app.current_tenant` posé **dans chaque transaction**, jamais à l'ouverture de connexion.
- Test qui **échoue si une table du schéma n'a aucune politique RLS** — exécuté en CI.
- Test d'isolation : un utilisateur du tenant A ne lit ni n'écrit aucune ligne du tenant B, sur chaque endpoint.

**TRX-04 — Observabilité et sauvegardes (P0)**
- Logs structurés avec corrélation par requête ; Sentry ; sonde `/health` ; alerte si indisponibilité > 2 min.
- `pg_dump` quotidien chiffré externalisé + synchronisation Garage ; **restauration complète testée et documentée avant la bascule du pilote**, puis chaque trimestre.
- Immutabilité des sauvegardes portée par le stockage externe (object lock), jamais par Garage.

**TRX-05a — Mécanique de seeds et tenants (P0, cycle 1)**
- **Mécanique de seeds rejouable**, à part des migrations (principe I·b) : rechargeables en une commande, idempotents, exécutables autant de fois que voulu.
- Tenant Deloria : établissement Abengourou (non classé, commune d'Abengourou, fuseau Africa/Abidjan), modules hébergement + restauration + bar + pressing + salle de réunion **activés** — l'activation seule, sans le contenu des modules.
- Second tenant « Résidence Test » (module hébergement seul) pour valider l'universalité.
- **Périmètre strict** : seules les tables du cycle 1 sont peuplées. Le reste relève de TRX-05b.

**TRX-05b — Jeu de données Deloria complet (P0, fin de tranche T1)**
- ⚠️ **Story à cheval, volontairement.** Les données décrites ici peuplent des tables qui **n'existent pas au cycle 1** : `unite`, `categorie`, `formule` et `bareme` appartiennent à HEB (cycle 4), `article` à PDV (cycle 7), les comptes à CPT (cycle 3). La livrer au cycle 1 est structurellement impossible.
- **Chaque cycle de T1 ajoute ses propres seeds** à la mécanique de TRX-05a, dans la même tâche que ses migrations. TRX-05b est la **tâche de recollement** en fin de T1 qui vérifie l'ensemble.
- Contenu cible : 17 unités dans 5 catégories aux tarifs réels, salle de réunion, **barèmes de passage et de demi-journée**, 30 articles de catalogue répartis sur les points de vente, 5 comptes de test aux rôles cumulés ; « Résidence Test » à 4 unités.
- **Critère de clôture** : rechargement complet en une commande, les deux tenants peuplés, et la démo de fin de T1 (§0.5) exécutable sur ces seules données.

**TRX-06 — Conformité ARTCI (P1)**
- Export et suppression des données d'une personne (endpoint admin) ; registre des traitements versionné dans le dépôt.
- Rétention paramétrable par catégorie de données : pièces d'identité 90 jours par défaut, photos 365 jours, documents fiscaux 10 ans.
- Consentement recueilli et tracé à l'enregistrement d'un client.

**TRX-07 — Mise à jour et télémétrie du parc (P1)**
- Serveur de mise à jour auto-hébergé pour les cibles desktop (plugin updater Tauri).
- Télémétrie minimale du parc auto-hébergé : version, santé, erreurs. Sans elle, aucun diagnostic à distance.
- Export d'un **bundle de diagnostic** déclenchable par le client.

**TRX-08 — Design system et thème (P1)**
- Tokens Tailwind 4 (couleurs, typographie, espacements, rayons) dans un fichier unique consommé par toute l'interface.
- **Mode sombre dès le premier écran**, jamais rétrofitté.
- 12 composants canoniques (champ, bouton, table, modale, tuile KPI, sélecteur de contexte, badge de statut, indicateur de synchronisation…).
- Règle d'analyse : aucune couleur ni espacement littéral hors des tokens.

---

## Module ETB — Établissements & modules d'activité

**ETB-01 — Tenant et établissements (P0)**
- Modèle `tenant → etablissement (1..n)`. L'établissement porte : juridiction, classement (étoiles / non classé / résidence meublée), commune, fuseau horaire, devise, adresse, NCC.
- **Le nombre d'établissements n'a aucun impact sur la tarification** (ADM-03).
- Un utilisateur peut être rattaché à plusieurs établissements avec des rôles différents sur chacun.

**ETB-02 — Modules d'activité activables (P0)**
- Référentiel `module_activite ∈ {HEBERGEMENT, RESTAURATION, BAR, PRESSING, SALLE_REUNION}` — **table, pas énumération figée dans le code** (provision cadrage §14.3).
- Activation/désactivation par établissement depuis la configuration.
- **L'interface ne montre jamais un module inactif** : pas de grisé, pas de mention, absent.
- Test structurel obligatoire : un établissement avec le seul module `RESTAURATION` (maquis) fonctionne de bout en bout — création, commande, encaissement, clôture — sans qu'aucun code ne suppose l'existence d'un hébergement. Idem avec le seul module `HEBERGEMENT`.

**ETB-02b — Capacités transverses (P0)**
- **Référentiel distinct du précédent** : `capacite ∈ {STOCK, LIVRAISON, PRODUCTION, COMMERCE_EN_LIGNE, FIDELITE, DEVIS, COMPTES_CLIENTS}` — table également.
- Le module est la **verticale** (ce que fait l'établissement) ; la capacité est le **transverse** (ce dont il a besoin pour le faire). Un module **déclare les capacités qu'il consomme**.
- **Seule `STOCK` est implémentée au MVP.** Toute autre valeur est **refusée explicitement** avec un message clair, jamais ignorée silencieusement.
- `STOCK` porte un profil : `profil_stock ∈ {AUCUN, SIMPLE, VALORISE, DETAILLE}`. **Seul `SIMPLE` est implémenté**, même règle de refus explicite (cadrage §14.5).
- Le profil est une propriété du module d'activité, pas du produit : un même tenant pourra un jour avoir un hôtel en `SIMPLE` et une quincaillerie en `DETAILLE`.

**ETB-02c — Test d'agnosticité du socle (P0)**
- **Test structurel permanent, en CI pour toujours** : un établissement portant un module d'activité **fictif minimal**, ne consommant **aucune capacité**, fonctionne de bout en bout — création, vente comptoir, encaissement, document fiscal, clôture journalière.
- C'est la preuve formelle que le socle ne suppose ni hébergement, ni point de vente, ni stock, ni aucune spécificité de verticale.
- **Ce test est le garde-fou de toute extension future du produit.** S'il tombe, le socle s'est spécialisé sans qu'on le voie.

**ETB-03 — Points de vente (P0)**
- `point_de_vente {etablissement_id, module_activite, nom, tables?, politique_impression, caisse_rattachee}`.
- Un module peut porter plusieurs points de vente (restaurant + terrasse).
- Un point de vente sans tables est un comptoir.

**ETB-04 — Configuration héritée (P0)**
- Résolution de configuration `tenant → établissement → module → point de vente`, avec surcharge locale. Exposée comme un trait propre du crate `etablissements`, **utilisée par tous les modules suivants**, testée exhaustivement y compris les surcharges partielles.
- Tout paramètre qualifié de « paramétrable » dans ces stories vit ici, **jamais en dur dans le code**.

**ETB-05 — Branding (P0)**
- Logo, couleur primaire, en-tête et pied des documents imprimés, mentions légales, coordonnées. Par tenant avec surcharge par établissement.
- Aperçu immédiat sur un document de test.

**ETB-06 — Sélecteur de contexte (P1)**
- Barre permanente : établissement actif, poste actif, indicateur de synchronisation.
- Bascule d'établissement en 2 taps pour un utilisateur multi-sites, sans reconnexion.

**ETB-07 — Partenaires externes (PROVISION)**
- Tables `partenaire {etablissement_id, nom, type, telephone, canal_prefere, tenant_id?}`, `demande_partenaire`, `compte_compensation`, `mouvement_compensation`.
- **`tenant_id` est nullable, et c'est tout le principe** : un restaurant, une quincaillerie ou un pressing extérieur avec qui l'établissement travaille est le cas normal ; qu'il ait lui-même un compte Kaya est l'enrichissement. Sans compte, la demande se transmet par WhatsApp ou SMS et les statuts se mettent à jour à la main ; avec compte, la même demande devient une transaction et les statuts se synchronisent.
- Couvre aussi la sous-traitance de l'hôtel vers l'extérieur, les fournisseurs d'une supérette et les grossistes d'une quincaillerie — c'est la provision la plus transversale du modèle.
- **Tables seulement. Aucune UI, aucune logique.** Voir `Kaya_Vision_Plateforme.md` §14.2.

**ETB-08 — Modules et capacités additionnels (PROVISION)**
- Le référentiel `module_activite` accepte l'ajout de `SPA`, `BOULANGERIE`, `SUPERETTE`, `QUINCAILLERIE`, `EXCURSION` par configuration, sans migration. Aucun n'est implémenté.
- Le référentiel `capacite` accepte de même l'ajout de nouvelles capacités. Seule `STOCK` au profil `SIMPLE` est implémentée.
- Voir `Kaya_Vision_Plateforme.md` pour l'analyse d'extension. **Aucune ligne de code au MVP.**

---

## Module CPT — Comptes, rôles & appareils

**CPT-00 — Personne, compte et employé (P0 pour le modèle, PROVISION pour l'employé)**
- **Trois entités distinctes, jamais confondues** : `personne` (identité civile — nom, pièce, contact), `compte` (identité d'authentification, porteuse des rôles), `employe` (contrat, salaire, date d'embauche, numéro CNPS).
- Une femme de ménage ou un gardien de nuit est un **employé sans compte**. Un comptable externe est un **compte sans contrat**. Un propriétaire est souvent les deux sans être salarié.
- **Au MVP, seules `personne` et `compte` portent de la logique.** `employe` est une table provisionnée, vide.
- ⚠️ **C'est la précaution qui conditionne toute la faisabilité du module RH en phase 2.** Écrire « le salaire de l'utilisateur » quelque part rendrait la paie inaccessible sans refonte de l'authentification. Aucun code ne doit supposer que `compte` = employé.

**CPT-01 — Comptes et authentification (P0)**
- Identifiant = téléphone E.164 (+225 par défaut selon l'établissement) ou email. Mot de passe fort, ou OTP SMS selon la configuration du tenant.
- JWT court + refresh révocable, multi-appareils. Déconnexion à distance d'une session.
- Les messages d'erreur ne révèlent jamais si un compte existe. ⚠️ **Le message identique ne suffit pas** : un refus en 2 ms sur compte inexistant contre 90 ms sur mot de passe faux publie la liste des comptes. Un hachage factice est exécuté sur le chemin « compte inconnu », et le test compare les médianes.
- ⚠️ **La brièveté du jeton ne doit jamais bloquer une écriture de classe A.** Aminata prend des commandes hors ligne pendant une coupure plus longue que la durée du jeton d'accès : les écritures A partent en file locale sans jeton, et **le retour du réseau rafraîchit le jeton AVANT de vider la file**, jamais l'inverse. Une file bloquée par un jeton expiré perdrait un service entier.
- **La révocation est immédiate**, portée par une liste en Redis consultée à chaque requête — pas par la brièveté du jeton. Le cadrage §12.2 exige la « coupure immédiate au départ d'un employé » ; attendre l'expiration ne la donne pas.

**CPT-02 — Rôles cumulables et permissions (P0)**
- Rôles : `proprietaire`, `gerant`, `receptionniste`, `serveur`, `caissier`, `magasinier`, `comptable`, `admin_editeur`.
- **Un utilisateur porte N rôles ; ses permissions sont l'union.** C'est la norme, pas l'exception : Adjoua est gérante, caissière et réceptionniste.
- Les permissions sont granulaires et attachées aux modules d'activité (ex. `pdv.remise.appliquer`, `heb.unite.attribuer`).
- **Aucune élévation de privilège hors ligne** (classe C).

**CPT-03 — Interface adaptée aux rôles (P0)**
- L'écran d'accueil est un tableau de bord composé de **tuiles filtrées par permission**, jamais un menu figé.
- **Chargement paresseux par module** : un serveur de salle ne télécharge pas le code du back-office.
- Le RBAC contrôle ce qu'on a le droit de faire, jamais quelle application on lance — il n'y a qu'une application.

**CPT-04 — Journal d'audit (P0)**
- Traçage immuable de : remise, annulation de ligne envoyée, avoir, ouverture de tiroir, modification de tarif, suppression, changement de rôle, écart de caisse, **rebascule de palier de passage**, forçage de disponibilité.
- Consultable par le propriétaire depuis n'importe quel terminal, filtrable par utilisateur, établissement, type et période.
- **Module de premier plan, pas un journal technique** — c'est ce que M. Koffi achète.

**CPT-05 — Enrôlement d'appareil (P1, tranche T4)**
- Le gérant approuve un appareil une fois ; une paire de clés est générée dans le Keystore Android / Keychain iOS et **signe chaque requête**.
- Liste des appareils enrôlés dans le back-office, avec révocation immédiate.
- Remplace le verrouillage par adresse MAC, techniquement impossible (cadrage §12.2).

**CPT-06 — Attestation et géorepérage souple (P1, tranche T4)**
- Play Integrity (Android) et DeviceCheck + App Attest (iOS) vérifiés côté serveur.
- Géorepérage : rayon paramétrable, **300 m par défaut**, position simulée détectée → **alerte au gérant, jamais blocage**.
- **Jamais bloquant sur une action critique** : un caissier qui ne peut pas encaisser parce que le GPS dérive est un client perdu.

---

## Module HEB — Hébergement : unités & formules

**HEB-01 — Unités louables et catégories (P0)**
- `categorie {etablissement_id, nom, capacite, temps_remise_en_etat_par_formule}` puis `unite {categorie_id, code, etage, statut_menage}`.
- **Ces entités vivent dans le crate `verticales/hebergement`, jamais dans le socle.** `unite_louable` est une spécialisation de `ressource_reservable` ; `sejour` et `formule` sont également propres à la verticale. Le socle ne connaît que `article_vendable` et `ressource_reservable` (cadrage §4.1 règle 1).
- La salle de réunion est une **unité louable d'une catégorie dédiée**, pas une entité nouvelle.
- Seeds Deloria : A1–A3 standard, B1–B5 classique, C1–C4 classique supérieure, D1–D2 supérieure A, E1–E3 supérieure B, plus la salle de réunion.

**HEB-02 — Disponibilité en intervalles horodatés (P0)**
- **Décision structurante et irréversible.** Une occupation est un intervalle `[début, fin)` en **timestamp avec fuseau de l'établissement**, jamais une paire de dates.
- Implémentation : contrainte d'exclusion PostgreSQL `EXCLUDE USING gist (unite_id WITH =, periode WITH &&)` sur `tstzrange` — **le chevauchement devient impossible au niveau de la base**, pas seulement dans le code.
- Le **temps de remise en état** est intégré à l'intervalle d'indisponibilité, pas géré à part. Défauts : passage 30 min, nuitée 2 h, demi-journée 1 h (paramétrables par catégorie).
- Test obligatoire : deux attributions concurrentes de la même unité sur des intervalles chevauchants — une seule réussit, par contrainte de base et non par verrou applicatif.

**HEB-03 — Formules de location (P0)**
- `formule {categorie_id, type ∈ [NUITEE, PASSAGE, DEMI_JOURNEE, MENSUEL], bareme, contraintes, assujettie_taxe_nuitee, regle_conversion_taxe}`.
- **Aucune formule n'est réservée à un type d'établissement.** Un hôtel peut proposer du mensuel ; une résidence meublée peut proposer du passage. La formule est attachée à la catégorie d'unité.
- Contraintes par formule : durée minimale et maximale, plages horaires autorisées, jours autorisés, heures d'arrivée et de départ standard.
- Seeds Deloria : nuitées aux 5 tarifs réels (**décomposées** en HT + TVA + taxe de nuitée, cf. FIS-03), passage, demi-journée, salle de réunion à la journée.

**HEB-04 — Barème dégressif du passage (P0)**
- Le barème est une **table de paliers** `{duree, prix}` + un prix d'heure supplémentaire au-delà du dernier palier. Seeds à confirmer à l'atelier terrain (B-07) : 1 h = 1 500, 2 h = 2 800, 3 h = 4 000, 4 h = 5 000, heure supplémentaire +1 200.
- Un dépassement constaté au départ **rebascule automatiquement** sur le palier supérieur ; la différence est ajoutée à la note et **tracée au journal d'audit** (CPT-04).
- Au-delà d'un seuil paramétrable, le dépassement **bascule en nuitée** (règle éditable par établissement).
- Le calcul de durée s'appuie **exclusivement sur l'horodatage d'autorité**, jamais sur l'horloge du terminal (SYN-04).

**HEB-05 — Formule demi-journée (P0)**
- Plages fixes définies par catégorie (seeds : 8h–12h, 13h–16h), **non fractionnables**.
- Deux demi-journées consécutives sur la même unité respectent le temps de remise en état.
- Utilisée par défaut pour la salle de réunion et les résidences meublées.

**HEB-06 — Statut d'unité (P1)**
- **Le statut d'occupation est dérivé**, jamais posé à la main : « libre », « occupée », « réservée » se calculent depuis les occupations.
- Seul le **sous-statut ménage** (à nettoyer / propre / maintenance) est librement modifiable — classe A, dernier-écrit-gagne autorisé.
- Mise hors service d'une unité : classe B (retire une ressource de la disponibilité).

**HEB-07 — Calendrier tarifaire simple (P1)**
- Un tarif peut porter une date d'effet et une date de fin. Priorité entre règles, aperçu avant publication.
- Hors périmètre : yield management, tarification dynamique, tarifs par canal.

**HEB-08 — Contrats et cautions (PROVISION)**
- Tables `contrat_location`, `caution`, `charge_locative`, `etat_des_lieux` pour les résidences meublées en incrément 3. **Tables seulement.**

**HEB-09 — Prestations incluses dans une formule (PROVISION au MVP → P1 incrément 2)**
- Table `prestation_incluse {formule_id, type, quantite, valeur_unitaire_plafond}` créée dès le MVP. **Table seulement au MVP.**
- **Fonctionnalité en incrément 2** : une formule peut inclure des prestations — petit-déjeuner, blanchisserie, courses de conciergerie. La prestation incluse s'affiche sur la note, se décompte à la consommation, n'est pas facturée, et le **dépassement du quota bascule en facturation normale** avec mention explicite.
- **Le petit-déjeuner inclus est une pratique répandue dans l'hôtellerie ivoirienne** et n'apparaissait nulle part dans le périmètre initial. Que le pilote le pratique ou non, le modèle doit le gérer : d'autres établissements le proposeront. Ce n'est pas de l'ambition plateforme, c'est une **lacune du produit hôtelier**, comblée comme telle.

---

## Module SEJ — Séjours & enregistrement

**SEJ-01 — Fiche client et recherche (P0)**
- `client {nom, prenoms, date_naissance, nationalite, piece_identite, telephone, email, preferences}` — rattaché au tenant, partagé entre ses établissements.
- Recherche par nom, téléphone ou numéro de pièce, avec résultats en moins de 300 ms sur 10 000 fiches.
- Historique des séjours consultable depuis la fiche.

**SEJ-02 — Check-in (P0)**
- Sélection de la formule, de la période, de la catégorie ; **proposition automatique d'une unité disponible** ; attribution (classe B).
- Client connu → **pré-remplissage complet**, aucune ressaisie.
- Enregistrement des accompagnants (impacte le calcul de la taxe de nuitée).
- Génération de la fiche de police et ouverture de la note.
- **Objectif mesuré : moins de 60 s pour un client connu, moins de 30 s pour un passage.** Si le passage dépasse 90 s, il sera contourné par le personnel.

**SEJ-03 — Note de séjour temps réel (P0, tranche T2)**
- Toutes les lignes rattachées au séjour : hébergement, consommations des points de vente, extras, remises.
- **Total provisoire visible instantanément** — c'est un des cinq problèmes explicites du cahier des charges du pilote.
- Transfert de charges entre séjours (classe B, tracé).

**SEJ-04 — Check-out et prolongation (P0)**
- Check-out : calcul final, application de la taxe de nuitée **figée à cet instant**, génération du document fiscal (FIS-02).
- Prolongation : vérification de disponibilité sur l'intervalle étendu, conflit avec la réservation suivante signalé explicitement.
- Départ anticipé : recalcul et régularisation tracés.
- Changement d'unité en cours de séjour : deux intervalles, historique conservé.

**SEJ-05 — Clients extérieurs (P0, tranche T2)**
- Vente à un client sans hébergement : addition autonome, encaissement immédiat, reçu.
- **Fonctionne sans module hébergement actif** — c'est le mode normal d'un maquis ou d'un bar seul.
- Fiche client optionnelle ; identification par téléphone si le client la souhaite.

**SEJ-06 — Enregistrement accéléré par OCR (P1, tranche T4)**
- Capture de la pièce d'identité par la caméra ; extraction **sur l'appareil** (Vision iOS / ML Kit Android) de nom, prénoms, date de naissance, numéro, nationalité.
- L'agent **corrige, ne saisit pas**.
- **Entièrement dégradable** : caméra indisponible, modèle en échec ou document illisible → saisie manuelle immédiate, sans blocage. C'est un accélérateur, jamais un passage obligé.
- Pièce d'identité = donnée sensible : chiffrement au repos, rétention paramétrable (90 j par défaut), journal d'accès, consentement tracé (TRX-06).

---

## Module RSV — Réservations (tranche T4)

**RSV-01 — Création de réservation (P0)**
- Client, formule, période, catégorie ou unité précise. Vérification de disponibilité sur l'intervalle (classe B).
- Statuts : `provisoire → confirmee → honoree | annulee | no_show`.
- Une réservation provisoire expire automatiquement après un délai paramétrable.

**RSV-02 — Planning visuel (P0)**
- Vue calendrier par unité, avec **granularité horaire** — les passages et demi-journées doivent y être lisibles, pas écrasés dans une case de journée.
- Filtres par catégorie, par statut. Navigation clavier sur desktop.
- Vue « aujourd'hui » par défaut, avec arrivées, départs et unités à nettoyer.

**RSV-03 — Arrhes et acomptes (P0)**
- Encaissement à la réservation, imputé sur la note au check-in.
- Politique d'annulation paramétrable : délai franc, montant retenu.

**RSV-04 — Annulation et no-show (P0)**
- Annulation : libération de l'intervalle, application de la politique, avoir si un encaissement a eu lieu (FIS-05).
- No-show : facturation selon la politique, tracée.

**RSV-05 — Conversion en séjour (P1)**
- Check-in depuis la réservation en un tap, avec pré-remplissage total.

---

## Module PDV — Points de vente (tranche T2)

**PDV-01 — Catalogue (P0)**
- `article {point_de_vente_id, nom, categorie, prix, taux_tva, disponible, suivi_stock, unite_mesure, code_barre?, article_parent_id?}`.
- **`quantite` en `NUMERIC`, jamais en entier**, sur toute ligne de commande. Un hôtel vend 1 bière ; une quincaillerie vendra 2,3 mètres de fer. Passer d'entier à décimal après mise en production imposerait de migrer toutes les lignes de vente et de stock.
- **`unite_mesure` obligatoire**, valeur par défaut `unite`. Table de conversion multi-unités créée, **non exploitée au MVP**.
- **`code_barre` et `article_parent_id` nullables et non utilisés au MVP** — ils rendront les profils de stock `VALORISE` et `DETAILLE` additifs plutôt que migratoires.
- Prix verrouillé à la création de la ligne de commande — une modification de tarif ultérieure ne modifie aucune commande existante.
- Catégories d'affichage ordonnables, recherche instantanée.

**PDV-02 — Tables et cibles de facturation (P0)**
- Un point de vente peut porter des tables ; une table ouverte porte une addition.
- **Cible de facturation** d'une commande : `table`, `sejour`, `comptoir`, `emporter`. La cible `sejour` n'est proposée que si le module hébergement est actif **et** qu'un séjour est en cours.
- Ouverture, fermeture, transfert entre tables, fusion : classe B.

**PDV-03 — Prise de commande (P0)**
- **Cœur du besoin hors-ligne (classe A).** Ajout de lignes, quantité, commentaire de préparation. Fonctionne intégralement sans réseau.
- Modification d'une ligne **non encore envoyée** : purement locale, jamais synchronisée avant envoi.
- **Annulation d'une ligne envoyée** : classe B, motif obligatoire, journal d'audit.
- Remise : classe B, autorisation par permission, journal d'audit.

**PDV-04 — Envoi en préparation (P0)**
- Regroupement par destination (cuisine, bar, pressing) selon l'article.
- Impression du bon de préparation ou affichage sur un écran de préparation.
- État de ligne : `saisie → envoyee → servie`.

**PDV-05 — Division d'addition (P0)**
- Par ligne ou par montant ; chaque part peut avoir une cible de facturation différente (une part sur la chambre, une part en espèces).
- Classe B.

**PDV-06 — Pressing : bon de dépôt (P0)**
- Le pressing n'est pas une vente immédiate : `depose → en_traitement → pret → retire`.
- Bon de dépôt avec liste d'articles, état constaté, date de retrait promise, numéro de retrait.
- Articles d'un client logé rattachés à son séjour ; d'un client extérieur à un bon autonome.
- Règlement à l'avance ou au retrait, paramétrable.

**PDV-07 — Écran de préparation (P1)**
- Affichage des commandes en cours par destination, avec ancienneté et marquage « prêt ».

**PDV-08 — Salle de réunion (P1)**
- Réservation à la journée ou demi-journée (HEB-05), avec prestations associées (pause café, restauration) rattachées à la même note.

---

## Module QRC — Commande par QR (tranche T4)

**QRC-01 — Jeton de table (P0)**
- QR encodant `https://kaya.app/t/{jeton}` où le jeton est **signé HMAC, opaque et révocable côté serveur** — jamais un identifiant de table lisible ou devinable.
- Révocation depuis le back-office sans changer la plaque physique (QR arraché, photographié, déplacé).
- Génération d'un PDF de plaque téléchargeable par table.

**QRC-02 — Page publique de commande (P0)**
- Nuxt SSR, **hors application Tauri**, hors authentification.
- Catalogue du point de vente en lecture, panier, validation.
- **Aucune donnée personnelle demandée** : pas de compte, pas de téléphone, pas d'email.
- Table fermée ou jeton révoqué → page neutre « service indisponible ».
- Performance sur connexion limitée : SSR léger, images optimisées.

**QRC-03 — Validation obligatoire par le personnel (P0)**
- La commande arrive en état **`À_CONFIRMER`** sur le terminal du serveur du point de vente (réception = classe A).
- **Le serveur valide d'un tap en constatant la présence physique du client** (validation = classe B). Rien ne part en préparation avant.
- **C'est le seul mécanisme anti-fraude du MVP.** Aucun géorepérage, aucun portail captif, aucun paiement préalable.

**QRC-04 — Limitation de débit (P0)**
- N paniers en attente maximum par table (défaut 3, paramétrable) — sans quoi un plaisantin sature l'écran du serveur.
- Compteur Redis par jeton, fenêtre glissante.

---

## Module CAI — Caisse & encaissements

**CAI-01 — Shifts et fond de caisse (P0, T2)**
- Ouverture de shift avec fond de caisse déclaré ; un utilisateur, une caisse, une période.
- Passation entre shifts avec comptage contradictoire.

**CAI-02 — Encaissement multi-modes (P0, T2)**
- Modes : espèces, Mobile Money, carte, virement, à crédit.
- **Règlement fractionné entre plusieurs modes sur une même note** — c'est la norme en hôtellerie, contrairement à la livraison.
- Espèces : classe B (irréversible). Mobile Money et carte : classe D.
- Montants en **entiers de FCFA**, jamais en flottant.

**CAI-03 — Sorties de caisse (P0, T2)**
- Dépense, avance, prélèvement — motif obligatoire, pièce justificative optionnelle, journal d'audit.

**CAI-04 — Comptage et écart (P0, T2)**
- Comptage par coupure ; **écart calculé, motivé et tracé**. Un écart au-delà d'un seuil paramétrable notifie le propriétaire.

**CAI-05 — Clôture de shift (P0, T3)**
- Récapitulatif par mode de règlement, par point de vente, par module d'activité.
- Impression du rapport de shift.

**CAI-06 — Clôture journalière (P0, T3)**
- **Atomique.** Refusée tant que : la file de synchronisation n'est pas vide, une facture est en attente ou en échec de certification, un terminal est déconnecté depuis plus de N minutes (défaut 15), une addition de table est restée ouverte.
- **Le refus affiche précisément ce qui bloque.** Une clôture fausse est pire qu'une clôture tardive.
- Objectif mesuré : **moins de 15 minutes** (contre environ une heure aujourd'hui à Deloria).
- Récapitulatif : recettes par service, encaissements par mode, taxes collectées, écarts.

**CAI-07 — Comptes clients à crédit (P1)**
- Encours par client entreprise, condition de règlement, relevé mensuel. Provision pour la facturation entreprises (§14.12).

---

## Module FIS — Fiscalité & documents (tranche T3)

**FIS-01 — Adaptateur de juridiction (P0)**
- Trait `JurisdictionAdapter { compute_taxes, required_document_fields, emission_channel, certify, remittance_reports }`.
- **Un seul adaptateur au MVP** : `CoteDIvoire`. **Aucune règle fiscale ne vit ailleurs dans le code.**
- Test doré : jeu de cas figés validé par le fiscaliste, exécuté en CI. Toute modification du moteur qui casse un cas doré échoue le build.

**FIS-02 — Séparation documents opérationnels / fiscaux (P0)**
- Deux agrégats distincts, deux numérotations, deux cycles de vie.
- **Documents opérationnels** (bon de commande, ticket, note provisoire, bon de dépôt pressing) : numérotation interne par établissement, générables hors ligne (classe A ou B), **mention obligatoire « Document non fiscal — ne tient pas lieu de facture »**.
- **Documents fiscaux** (FNE, avoir) : numérotation attribuée par la DGI, **jamais générables hors ligne** (classe D).

**FIS-03 — Moteur de taxes Côte d'Ivoire (P0)**
- TVA 18 % · **taxe communale de nuitée** (sans étoile 500, 1★ 1 000, 2★ 1 500, 3★+ 2 000, résidence meublée district d'Abidjan 1 000) · taxe pour le développement touristique 2,5 %.
- La taxe de nuitée est **par nuitée et par client** (accompagnants inclus), **ligne distincte obligatoire** sur la facture, séparée du HT et de la TVA.
- **Montant figé au check-out**, jamais recalculé dynamiquement. Toute modification postérieure passe par un avoir.
- **Chaque formule porte `assujettie_taxe_nuitee` et une règle de conversion** (`aucune` / `une_nuitee_par_occupation` / `au_prorata` / `seuil_horaire`) — le traitement du passage et de la demi-journée est un **paramètre**, pas une constante (décision B-02).
- Reprise Deloria : **décomposition de chaque tarif affiché** (12 500, 15 500, 17 500, 20 500, 25 500) en HT + TVA + taxe de nuitée. Tâche de migration explicite.

**FIS-04 — Passerelle FNE (P0)**
- Trait `FneGateway { certify, refund, status }` avec implémentations `Partenaire` (API du partenaire agréé) et `Direct` (API DGI), **bascule par configuration de tenant**, sans toucher au métier.
- **Coffre chiffré par tenant** pour les clés API. Chaque établissement client possède son propre compte FNE et sa propre clé — étape obligatoire du parcours d'installation.
- Écran de saisie de clé avec procédure d'accompagnement : la clé n'est visible que par le gestionnaire principal du client dans son espace FNE.

**FIS-05 — File de certification et états (P0)**
- Quatre états : `EN_ATTENTE → SOUMISE → CERTIFIEE`, branches `ECHEC` (erreur métier explicite → correction et resoumission) et **`INDETERMINEE`** (timeout).
- **L'état `INDETERMINEE` n'est JAMAIS rejoué automatiquement** — l'API FNE n'expose aucune clé d'idempotence, un rejeu produirait une double certification et consommerait un sticker en double.
- **Écran de rapprochement manuel** : l'opérateur vérifie dans l'espace FNE du client et tranche. Aucune automatisation acceptable ici.
- Rejeu à backoff exponentiel pour les états `EN_ATTENTE` uniquement.
- Tableau de bord « documents non certifiés » visible du gérant et du propriétaire.

**FIS-06 — Avoirs (P0)**
- `POST /external/invoices/{id}/refund` — corps limité à `{id d'item, quantity}`.
- **L'avoir se fait par quantité, pas par montant.** L'interface **guide** l'opérateur dans la manipulation « annuler la ligne entière puis refacturer au tarif remisé » pour un geste commercial partiel.
- **Les `id` d'items retournés par l'API de certification sont persistés**, pas seulement les identifiants internes — sans eux aucun avoir n'est possible. Erreur irrattrapable a posteriori.

**FIS-07 — Surveillance des stickers (P0)**
- Compteur par établissement ; alerte à J-7 et J-2 du seuil bas, visible du gérant et du propriétaire.
- **Blocage préventif de la clôture** si le stock ne couvre pas les documents en attente.
- Un établissement à court de stickers ne peut plus facturer ; le délai de rechargement publié est de 48 h.

**FIS-08 — État de reversement communal (P0)**
- Relevé mensuel par commune : nuitées assujetties, nombre de clients, montant dû, **échéance au 15 du mois suivant**.
- Export PDF et tableur. **Aucun concurrent ne le produit** — c'est un argument de vente à part entière.

**FIS-09 — Export comptable (P1)**
- Écritures normalisées exploitables par un comptable externe (format tableur + CSV structuré), par période, par établissement, par journal.
- Provision : le format est conçu pour accueillir un plan comptable SYSCOHADA en phase 2, sans migration.

**FIS-10 — Canal d'émission RNE (PROVISION)**
- Trait `EmissionChannel { FneApi, Terne }` et colonne `rne_ref` nullable sur les lignes de facture.
- **Aucune implémentation `Terne` au MVP.** Si la DGI qualifie un jour les points de vente de caisses enregistreuses, l'implémentation s'ajoute sans migration.

**FIS-11 — Documents commerciaux (PROVISION)**
- Tables `devis` et `document_commercial`, cycle `brouillon → émis → accepté → converti | expiré`, avec référence vers le document fiscal issu de la conversion.
- **Tables seulement. Aucune UI, aucune logique au MVP.**
- L'hôtellerie s'en passe ; le B2B et le commerce de détail en dépendent lourdement — un client de quincaillerie demande un devis avant d'acheter.

---

## Module SYN — Synchronisation & hors-ligne

**SYN-01 — Classification et invariante (P0, T1)**
- **Chaque entité déclare sa classe** A/B/C/D dans un registre versionné (cadrage §11.3).
- Test structurel : une opération B, C ou D **atteignable depuis un chemin de code exécutable hors ligne fait échouer le build**.
- L'interface annonce immédiatement et explicitement toute action indisponible faute de réseau — jamais de grisé silencieux, jamais d'échec après coup, jamais de mise en file « au cas où ».

**SYN-02 — File d'actions hors-ligne (P0, T1)**
- Toute écriture porte un **UUID v7 généré côté client** + horodatage local ; le serveur déduplique.
- File locale persistante ; envoi opportuniste ; rejeu idempotent ; **le serveur fait foi en cas de conflit**.
- **Conçue pour être vidée au retour au premier plan par défaut**, sur toutes les plateformes — iOS n'a pas de synchronisation en arrière-plan. `BGTaskScheduler` et `WorkManager` sont des optimisations, jamais des hypothèses.
- **Indicateur permanent** dans l'interface : connecté / dégradé / hors ligne + nombre d'éléments en attente.

**SYN-03 — Réconciliation des écritures orphelines (P0, T3)**
- Scénario : une consommation saisie hors ligne arrive sur un séjour déjà clos et facturé.
- **Résolution humaine obligatoire.** Jamais de rejet silencieux, jamais d'ajout d'office.
- Écran de réconciliation : le gérant tranche entre avoir et refacturation, prise en charge, ou rattachement au séjour suivant.
- Aggravé par le fait que l'avoir FNE se fait par quantité (FIS-06). **Testé en priorité — c'est le conflit le plus fréquent en exploitation réelle.**

**SYN-04 — Horodatage d'autorité (P0, T1)**
- Chaque écriture porte un horodatage client (indicatif, ordre d'affichage local) **et reçoit un horodatage d'autorité à l'arrivée**.
- **Toute logique métier, tout calcul fiscal, toute clôture et tout calcul de durée de passage s'appuient exclusivement sur l'horodatage d'autorité.**
- Détection et signalement d'une dérive supérieure à 5 minutes.
- Test transverse permanent : couper le réseau au milieu d'une journée d'exploitation simulée, le rétablir, **vérifier que la clôture tombe au franc près**.

---

## Module IMP — Impression & documents

**IMP-01 — Impression thermique ESC/POS (P0, T2)**
- 80 mm, USB et réseau, depuis desktop et Android. Ouverture du tiroir-caisse.
- File d'impression avec reprise : une imprimante hors ligne ne bloque jamais l'encaissement.
- Modèles : ticket de commande, bon de préparation, bon de dépôt pressing, reçu, rapport de shift.

**IMP-02 — Note provisoire et documents opérationnels (P0, T2)**
- **Mention obligatoire « Document non fiscal — ne tient pas lieu de facture »** sur tout document opérationnel.
- Détail par service, total provisoire, mention de l'établissement et branding (ETB-05).

**IMP-03 — Facture fiscale A4 (P0, T3)**
- PDF conforme : mentions obligatoires, identification complète du client, désignation détaillée des prestations, **ligne distincte de taxe de nuitée**, visuel FNE, QR code et sceau retournés par la DGI.
- Stockage dans Garage, rétention 10 ans, consultable et réimprimable depuis la fiche de séjour.

**IMP-04 — Modèles éditables (P1)**
- En-tête, pied, mentions légales et coordonnées éditables par établissement, avec aperçu.

---

## Module STK — Stocks (tranche T5)

**STK-01 — Articles et points de stock (P0)**
- `article_stock {etablissement_id, nom, unite_mesure, seuil_alerte}` et `point_de_stock` (cave, cuisine, bar).
- Liaison article de catalogue → article de stock, avec quantité consommée par vente.
- **Le stock est une capacité, pas un module** (ETB-02b). Il porte un profil ; **seul `SIMPLE` est implémenté** au MVP, les autres sont refusés explicitement.
- Le crate vit dans `capacites/stocks`, jamais dans `socle/` ni dans une verticale.

**STK-02 — Mouvements (P0)**
- Entrée, sortie sur vente, ajustement d'inventaire, transfert, casse ou perte.
- **`quantite` en `NUMERIC`, jamais en entier** — une boulangerie achètera 47,5 kg de farine.
- **`cout_unitaire` nullable, JAMAIS renseigné au MVP.** Sans cette colonne, aucune valorisation rétroactive ne serait possible et le profil `VALORISE` exigerait de recréer tout l'historique. Le coût de l'ajouter aujourd'hui est nul.
- **Classe B par défaut** (décision B-05, à trancher avec le pilote : si le stock sert à détecter le vol, il reste B et sérialisé ; s'il sert seulement à réapprovisionner, il peut passer en A et tout devient plus simple).
- Consultation hors ligne : **toujours affichée comme indicative**.

**STK-03 — Inventaire (P0)**
- Saisie par point de stock, écart calculé et motivé, valorisation à venir en phase 2.

**STK-04 — Alertes de seuil (P0)**
- Classe A. Notification au magasinier et au gérant.
- Hors périmètre : valorisation PMP ou FIFO, commandes fournisseurs, réception partielle, variantes, codes-barres, lots et dates limites. Ce sont les profils `VALORISE` et `DETAILLE`, non implémentés.

---

## Module MOB — Mobile Tauri (tranche T4)

**MOB-01 — Build Android et iOS (P0)**
- Chaîne complète `cargo tauri android` et `cargo tauri ios`, signature, distribution.
- **Android** : APK direct hors store possible, mécanisme d'installation à écrire ; Play Store pour la distribution large.
- **iOS** : **aucune installation hors App Store**, toute mise à jour du binaire passe par la revue Apple. La mise à jour des assets web dans le WebView est un **correctif d'urgence, jamais le canal de livraison normal**.
- Le plugin updater de Tauri exclut explicitement les cibles mobiles.

**MOB-02 — PlatformAdapter (P0)**
- **Aucune invocation directe de `window.__TAURI__` dans un composant.** Impression, scan, OCR, stockage sécurisé, notifications, géolocalisation, réseau passent par cette interface.
- Implémentations `desktop`, `android`, `ios`, `web`.
- Une capacité absente sur une plateforme **le dit explicitement à l'utilisateur** et propose l'alternative.

**MOB-03 — Notifications push (P0)**
- Plugin natif Swift (APNs) + Kotlin (FCM). Le plugin officiel Tauri ne couvre que les notifications locales.
- Canal haute importance pour les alertes critiques : facture en échec de certification, stickers bas, écart de caisse, terminal déconnecté.
- Budget : 2–3 semaines.

**MOB-04 — Impression thermique Bluetooth (P0)**
- Plugin natif CoreBluetooth (iOS) + Android BT. Budget : 2 semaines.
- Un caissier mobile doit pouvoir imprimer un reçu.

**MOB-05 — Stockage sécurisé et purge (P0)**
- Keystore Android / Keychain iOS pour les clés d'appareil (CPT-05).
- **Chiffrement au repos du cache local** et **purge à la déconnexion** — ce sont des données d'identité de clients.

**MOB-06 — Synchronisation en arrière-plan (P1)**
- `BGTaskScheduler` (iOS) et `WorkManager` (Android). **Optimisation, jamais hypothèse** — la file se vide au premier plan par défaut (SYN-02). Budget : 1–2 semaines.

---

## Module DIR — Direction & pilotage

**DIR-01 — Tableau de bord établissement (P0, T3)**
- Occupation du jour, arrivées et départs, unités à nettoyer, recettes par service, encaissements par mode, note moyenne, documents en attente de certification.

**DIR-02 — Tableau de bord consolidé multi-établissements (P0, T5)**
- Vue unique sur tous les établissements du tenant, en temps réel, lisible sur téléphone.
- 8 à 10 KPI, comparaison inter-établissements, alertes.
- **C'est la demande explicite du persona propriétaire.**

**DIR-03 — Recettes par service (P0, T5)**
- Ventilation par module d'activité, par point de vente, par formule d'hébergement. Distinguer les recettes de passage des recettes de nuitée est un besoin réel non couvert par le papier.

**DIR-04 — Consultation du journal d'audit (P0, T5)**
- Filtres par utilisateur, établissement, type d'action, période. Export.
- Alertes configurables : remise au-delà d'un seuil, écart de caisse, rebascule de passage anormale.

**DIR-05 — Rapports périodiques (P0, T5)**
- Occupation, chiffre d'affaires, taxes collectées, écarts, par période. Export PDF et tableur.
- Hors périmètre : BI libre-service, requêtes ad hoc.

---

## Module ADM — Console éditeur & abonnements (tranche T5)

**ADM-01 — Provisionnement de tenant (P0)**
- Création, configuration initiale, seeds fiscaux par juridiction, comptes initiaux.
- Parcours d'installation guidé incluant **l'étape obligatoire de saisie de la clé FNE du client**.

**ADM-02 — Gestion du parc (P0)**
- Liste des tenants, établissements, unités, versions déployées, dernière synchronisation, état de santé.
- Filtre sur le parc auto-hébergé, avec version et remontée de télémétrie (TRX-07).

**ADM-03 — Moteur d'abonnement (P0)**
- Calcul sur le **nombre total d'`unite_facturable` du tenant, tous établissements confondus**. Le nombre d'établissements n'a aucun impact.
- **`unite_facturable` est une métrique abstraite définie par la verticale**, jamais « chambre » codé en dur : la chambre pour l'hébergement, le point de vente pour la restauration et le commerce, le véhicule pour la livraison. Le moteur de tarification ne connaît qu'un nombre ; **la verticale expose un trait qui dit ce qu'on compte.**
- Au MVP, la seule implémentation est « chambre » et le comportement observable est strictement identique à une facturation à la chambre. Le coût de l'abstraction aujourd'hui est d'un trait et d'une implémentation ; sans elle, aucune verticale sans chambres n'est facturable.
- Deux modes, **le client bénéficie automatiquement du moins-disant** : forfait par palier (≤ 25 unités : 20 000 ; 26–50 : 40 000 ; > 50 : 1 000/unité) et compteur (1 000/unité à tout niveau).
- **Tous les seuils, montants et paliers éditables depuis la console**, sans déploiement.
- Périodes de gratuité, frais d'installation, remises commerciales — tracés.

**ADM-04 — Facturation des abonnements (P0)**
- Trait `PaymentProvider { create_checkout, verify_webhook, refund }` ; implémentation CinetPay.
- Session créée côté serveur, **webhook validé par signature HMAC**, jamais de confiance dans la redirection client seule, idempotence sur le webhook.
- Relances, suspension pour impayé après délai de grâce paramétrable.

**ADM-05 — Diagnostic à distance (P0)**
- Consultation des journaux d'un tenant, état de la file de certification, état de la file de synchronisation, dernière sauvegarde.
- **Sans cet écran, le support d'Abengourou impose un déplacement.**

**ADM-06 — Registre des paramètres (P0)**
- Écran unique listant tous les paramètres d'établissement avec leur valeur effective et leur origine dans la chaîne d'héritage (ETB-04). Référence : récapitulatif en fin de document.

---

## Module MET — Métriques (tranche T5)

**MET-01 — Taxonomie d'événements (P0)**
- Catalogue versionné dans le dépôt : événements **produit** (ouverture, écran vu, action) et **opérations** (dérivés de l'outbox : transitions, certifications, écarts, réconciliations).
- Propriétés standard : tenant, établissement, module d'activité, rôle, version d'application, plateforme.
- Toute nouvelle fonctionnalité déclare ses événements (Definition of Done).

**MET-02 — Ingestion (P0)**
- Endpoint acceptant des lots ; idempotence par UUID d'événement ; file locale dans l'application (mêmes garanties que SYN-02) ; horodatage client conservé + horodatage serveur.

**MET-03 — Agrégats et KPI (P1)**
- Agrégats quotidiens alimentant les indicateurs du cadrage §18 : durée de clôture, consommations non facturées, taux de certification du premier coup, durée d'enregistrement, **part des passages saisis dans le système**, sessions hors ligne réconciliées sans intervention, écarts de caisse, connexions du propriétaire.
- Exploration par tableau de bord SQL, sans développement dédié.

---

## Récapitulatif des paramètres d'établissement (référence ADM-06)

| Paramètre | Défaut Deloria | Story |
|---|---|---|
| Classement de l'établissement | Non classé (à confirmer) | ETB-01 |
| Commune de rattachement | Abengourou | ETB-01 |
| Fuseau horaire | Africa/Abidjan | ETB-01 |
| Devise | XOF (0 décimale, entiers) | ETB-01 |
| Modules actifs | Hébergement, restauration, bar, pressing, salle de réunion | ETB-02 |
| Politique d'impression (`politique_impression`) | **Jeu de valeurs défini par le cycle IMP** — la clé existe au catalogue, sa portée la plus basse est le point de vente | ETB-03 |
| Temps de remise en état — passage | 30 min | HEB-02 |
| Temps de remise en état — nuitée | 2 h | HEB-02 |
| Temps de remise en état — demi-journée | 1 h | HEB-02 |
| Heure d'arrivée / départ standard (nuitée) | 14h / 12h | HEB-03 |
| Barème passage | 1 h : 1 500 · 2 h : 2 800 · 3 h : 4 000 · 4 h : 5 000 · h. suppl. : +1 200 | HEB-04 |
| Durée max de passage avant bascule en nuitée | 8 h | HEB-04 |
| Plages de demi-journée | 8h–12h et 13h–16h | HEB-05 |
| Expiration d'une réservation provisoire | 24 h | RSV-01 |
| Politique d'annulation — délai franc | 48 h | RSV-03 |
| Taux de TVA | 18 % | FIS-03 |
| Taxe communale de nuitée | 500 FCFA (non classé) | FIS-03 |
| Taxe de nuitée sur passage | **À trancher (B-02)** | FIS-03 |
| Taxe de nuitée sur demi-journée | **À trancher (B-02)** | FIS-03 |
| Taxe dév. touristique | 2,5 % | FIS-03 |
| Seuil d'alerte stickers FNE | J-7 et J-2 | FIS-07 |
| Rétention pièces d'identité | 90 jours | TRX-06 |
| Rétention photos | 365 jours | TRX-06 |
| Rétention documents fiscaux | 10 ans | TRX-06 |
| Seuil d'écart de caisse notifiant le propriétaire | 1 000 FCFA | CAI-04 |
| Terminal déconnecté bloquant la clôture | 15 min | CAI-06 |
| Paniers QR en attente max par table | 3 | QRC-04 |
| Indicatif téléphonique par défaut (`indicatif_telephonique_defaut`) | +225 (Côte d'Ivoire) | CPT-01 |
| Méthode d'authentification (`methode_authentification`) | Mot de passe (alternative : OTP SMS, **non implémentée**) | CPT-01 |
| Longueur minimale du mot de passe (`mot_de_passe_longueur_min`) | 8 caractères, **aucune règle de composition**, refus des mots de passe compromis | CPT-01 |
| Durée du jeton d'accès (`jeton_acces_duree_min`) | 60 min | CPT-01 |
| Durée du jeton de rafraîchissement (`jeton_rafraichissement_duree_jours`) | 90 jours, **avec rotation à chaque usage** | CPT-01 |
| Rayon de géorepérage | 300 m (alerte seulement) | CPT-06 |
| Dérive d'horloge signalée | 5 min | SYN-04 |
| Seuil d'alerte de stock | Par article | STK-04 |
| Classe hors-ligne du stock | B (à trancher B-05) | STK-02 |
| Palier d'abonnement 1 | ≤ 25 unités → 20 000 FCFA | ADM-03 |
| Palier d'abonnement 2 | 26–50 unités → 40 000 FCFA | ADM-03 |
| Palier d'abonnement 3 | > 50 unités → 1 000 FCFA/unité | ADM-03 |
| Tarif au compteur | 1 000 FCFA/unité | ADM-03 |
| Délai de grâce avant suspension pour impayé | 15 jours | ADM-04 |

---

## Prochaine étape

Trancher **B-03** (source de revenus de transition), **B-02** (taxe de nuitée sur les formules infra-journalières, avec le fiscaliste) et **B-07** (barèmes de passage réels, à l'atelier terrain d'Abengourou), puis démarrer la tranche T1 : cycle TRX, cycle ETB, cycle CPT dans cet ordre, après vérification et gel des dernières versions stables de toute la stack.
