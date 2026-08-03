# Revue de la Definition of Done — Cycle 005 · SYN

**T070** · 2026-08-03 · `docs/user-stories-v1.md` §0.4

Les dix points, pour les **trois user stories P0** du cycle (SYN-01, SYN-02, SYN-04), **avec la
preuve de chacun**. Un point coché sans preuve est un point que personne n'a vérifié.

> **Le point 10 est SANS OBJET**, et c'est écrit ici plutôt que coché en silence : ce cycle
> n'imprime rien — aucun document, aucune file d'impression, aucun pilote. Même règle qu'aux
> cycles 002, 003 et 004.

---

## Les chiffres du cycle, recomptés et non repris du plan

| Grandeur | Réel | Ce qu'annonçait le plan |
|---|---|---|
| Types d'événements outbox | **28** | 27 |
| Opérations HTTP servies | **56**, inchangé | 56, inchangé |
| Tables, schémas **découverts** | **35**, dont **1** créée ici | 35 |
| Migrations du cycle | **2** (`0027`, `0028`) | 2 |
| Familles d'audit | **11**, dont **4 branchées** | 11, dont 4 |
| Paramètres au catalogue | **10** (8 + 2) | 10 |
| Provisions sans logique | **6** (5 + 1) | 6 |
| Tests backend | **362**, 0 échec | — |
| Tests front | **522** (+64), 0 échec, 0 erreur de type | — |
| Balayage hors ligne (FR-005b) | **10** cas × **2** moteurs | — |
| Clés i18n | **311 fr / 311 en**, écart nul | — |
| Écrans du produit | **45** (11 maquettés / 32 dérivés / **2** composés) | 45 |

**Deux écarts au plan, tous deux en plus, et tous deux trouvés par une porte :**

1. **28 types d'événements et non 27.** Le balayage de P-05, passé d'une liste de onze chemins de
   service aux **crates métier découverts**, a signalé `note_etablissement.creee` — émis en
   production depuis le cycle 001, absent de `TYPES_EVENEMENTS`, invisible aux **deux** versants de
   la porte. C'est le troisième trou de ce genre dans ce dépôt, et **le premier trouvé par une
   porte plutôt que par une relecture**.
2. **Le contrat OpenAPI a changé**, alors que le plan annonçait un diff vide sur P-01.
   `TypeActionAudit` porte `ToSchema` : ajouter une famille d'audit élargit une valeur
   d'énumération du contrat. Le décompte d'opérations, lui, est bien resté à 56 — P-08 intacte.

---

## Les dix points

### 1 · Critères d'acceptation couverts par des tests — unitaires **et** d'intégration ✓

| Story | Critère | La preuve |
|---|---|---|
| **SYN-01** | La file survit au rechargement **et** à l'extinction | `app/tests/file-persistance.spec.ts` — réouverture complète d'une instance neuve, quatre écritures retrouvées |
| SYN-01 | La charge est illisible dans le stockage | même fichier — le texte n'apparaît nulle part en clair ; sans la clé, la file repart vide |
| SYN-01 | Rafraîchir **avant** d'envoyer, même sur une file rouverte | `file-jeton-expire.spec.ts` — l'ordre tient y compris quand les deux réussissent |
| SYN-01 | Le rejeu est inoffensif | `commun/classes.rs::tester_classe_a!` — trois envois, **une ligne et un événement** |
| **SYN-02** | Le témoin dit vrai en permanence | `temoin-sync.spec.ts` — 3 états × 2 thèmes × 2 langues, décompte assertion |
| SYN-02 | Aucune opération B/C/D en file | `file-classe-a.spec.ts` — quatre `@ts-expect-error`, refus à la compilation |
| SYN-02 | Quatre déclencheurs, aucune scrutation | `file-envoi.spec.ts` — une file vide et une coupure n'arment **aucun** minuteur |
| **SYN-04** | La dérive est constatée dans les **deux sens** | `derive_horloge.rs` — retard et avance, sur les deux tenants |
| SYN-04 | L'écriture est **acceptée** malgré la dérive | même fichier — `201` sur un horodatage à trois heures |
| SYN-04 | Une entrée par **épisode** | même fichier — dix saisies décalées, une entrée |

**Les tests unitaires existent aussi** : `constater_derive` a ses sept tests dans son crate, dont
la symétrie des deux sens et l'arrondi des minutes.

### 2 · Annotations utoipa à jour ; client TypeScript régénéré sans diff manuel ✓ — **avec un écart au plan**

`pnpm porte:p01` ✓ — client identique au contrat.

**Le plan annonçait un diff vide ; il ne l'est pas.** `TypeActionAudit` est exposée au contrat par
`ToSchema`, et la onzième famille d'audit y ajoute une valeur d'énumération. Le client a été
régénéré dans le commit qui a ajouté la famille, pas à la fin — c'est la règle du principe I(a).

Conséquence non prévue et traitée : `modules/audit/journal.ts` devait suivre, faute de quoi
`derive_horloge_constatee` se serait affichée **en brut** dans le filtre de `G4`.

### 3 · Migration sqlx versionnée ; `cargo sqlx prepare` vert ; seeds à jour ✓

Deux migrations additives — `0027` et `0028`. `pnpm porte:p02` ✓ — 28 migrations, aucune modifiée.

**La double passe a été faite, et les DEUX contrôles avec elle** :

```
git status --short backend/.sqlx      → vide, aucune suppression
touch des fichiers à sqlx::query
SQLX_OFFLINE=true cargo check …       → 20 s de recompilation réelle, pas « Finished » en 1 s
```

Le `touch` n'est pas décoratif : sans lui, le cycle 004 a vu ce contrôle passer au vert sur un
cache **réellement périmé**.

Seeds étendus de trois à cinq valeurs Deloria. Une clé au catalogue sans valeur montrerait une
ligne vide à l'écran de configuration du pilote.

### 4 · RLS activée **et forcée** sur toute nouvelle table, avec test d'isolation ✓

Une seule table nouvelle : `synchronisation.reconciliation_orpheline`. `ENABLE` + `FORCE` +
politique `isolation_tenant`, comme toutes les tables du produit sans exception.

`isolation_tenant.rs::p08_la_reconciliation_orpheline_est_isolee_par_tenant` — le tenant A ne lit
pas le constat de B, **y compris demandé par son identifiant**, et voit bien le sien (le versant
positif, sans lequel l'assertion serait vraie pour la mauvaise raison).

`rls_catalogue.rs` (P-07) l'inspecte sur un périmètre **découvert**, plancher porté à 35.

### 5 · Classe hors-ligne déclarée pour toute nouvelle entité, avec le test du §0.7 ✓

**Aucune ligne n'a été ajoutée au registre**, et c'est le premier cycle du produit dans ce cas :
les deux tables y figuraient depuis le 2026-07-30, décidées à froid. Le registre passe en **1.3.0**,
§5.6 déclaré effectif.

Le §0.7 est **le changement de fond de ce cycle** : ses tests s'instancient désormais
(`commun/classes.rs`) au lieu d'être recopiés une fois par entité — ce qui avait déjà été fait
trois fois, avec trois formulations. `outillage_classes.rs` échoue en **nommant** l'entité qui
aurait une table sans instanciation.

### 6 · Événement outbox émis pour tout changement d'état métier ✓ — **par absence**

**Aucun type d'événement nouveau, et c'est correct** : ce cycle ne crée aucune transition d'état
métier. La file transporte des écritures dont les événements sont émis par leurs services ; la
table de réconciliation est une provision sans écriture ; la dérive est un **constat
d'exploitation**, qui va au registre des actions et non au grand livre.

Ce qui est vérifié à la place, et qui est le point : **un rejeu n'émet AUCUN événement**. Trois
envois → une ligne, **un** événement. C'est désormais une garantie de toutes les entités de
classe A à venir, pas seulement de celle-là.

### 7 · Clés i18n fr **et** en externalisées ; aucune chaîne en dur ✓

`pnpm test:i18n` ✓ — **311 fr / 311 en**, écart nul, 27 templates inspectés sans littéral.

**Les trois libellés du témoin ont été corrigés, pas seulement ajoutés.** L'i18n disait
« Connecté », « Hors ligne », « {n} éléments en attente » — ce qui décrit le **réseau**. Le lexique
1.5.0 fait foi : « Enregistré », « Hors connexion », « En attente d'envoi ({n}) » — ce qui dit à
Aminata si son travail est en sécurité.

### 8 · Écran vérifié en mode clair **et** en mode sombre ✓

`pnpm porte:p22` ✓ — **10 routes**, en chargement direct **et** par navigation interne, dans les
deux thèmes, sur **chromium et webkit**, 68 cas. `porte:p22:negatif` ✓ — la coquille cassée
volontairement est bien refusée.

`theme-sombre.spec.ts` inscrit les trois fichiers nouveaux à son inventaire ; aucune classe `dark:`
de couleur, aucune seconde palette.

**P-22 a trouvé un défaut réel avant de passer au vert** : les deux clés du catalogue posées par
`0028` n'avaient pas de libellé i18n, et l'écran de configuration affichait deux avertissements
`[intlify] Not found` sur les deux moteurs. Aucun test de composant ne l'aurait vu.

### 9 · Paramètres exposés dans la configuration d'établissement ✓

Deux clés au catalogue, migration `0028`, inscrites au récapitulatif §708 **dans le même
changement** — le principe I(c) en fait la condition, et le cycle 004 a montré que différer cet
ajout le fait oublier.

`sync.derive_horloge_seuil_secondes` (300) et `sync.latence_degradee_seuil_ms` (3 000). Ce sont
**les premières clés du produit à porter un préfixe de module** : le catalogue est un référentiel
unique, et un `latence_degradee_seuil_ms` nu serait revendiqué par le premier autre module qui
mesure une latence.

Aucune valeur métier en dur : les 5 minutes du cadrage §11.4 sont devenues le **défaut** d'un
paramètre.

### 10 · Document imprimé vérifié sur imprimante thermique — **SANS OBJET** ⊘

Ce cycle n'imprime rien.

---

## Ce qui reste non conforme, ou vérifié plus faiblement qu'il n'y paraît

*La partie de la revue qui compte. Un point coché est une information ; un point pris en défaut en
est une meilleure.*

**Huit points, dont un qui aurait empêché tout déploiement** — voir le n° 7.

### 1 · Le balayage e2e est passé au vert — mais il a d'abord passé au vert POUR RIEN

`tests-e2e/hors-ligne.spec.ts` **passe, dix cas sur dix, sur chromium et webkit**. Ce qui mérite
d'être écrit est ce qui s'est passé avant.

**Sa première version était verte sur neuf cas, et elle ne gardait rien.** Le jeton d'accès vit en
mémoire et meurt avec la page ; un rechargement oblige `reprendreSession()` à rejouer le jeton de
rafraîchissement, ce qui **exige le réseau**. Hors ligne, chaque `page.goto` renvoyait donc sur
`/connexion` — et les neuf cas inspectaient neuf fois l'écran de connexion. Le `<main>` existait,
aucune erreur de console, aucune écriture en file : tout était vert.

C'est **exactement** le mode de défaillance que le § « Couverture des portes » nomme, rencontré
sur la porte écrite pour le fermer. Il a été trouvé parce que le dixième cas — le versant positif,
la saisie de classe A — échouait, lui : il cherchait un champ qui n'était pas là.

Deux corrections, et la seconde est la leçon :

1. **Navigation par le routeur, sans rechargement.** C'est le chemin réel d'un utilisateur qui perd
   le réseau *en cours de service* : son application est ouverte, sa session est vivante.
2. **Un contrôle d'anti-régression** : chaque cas vérifie désormais que l'URL n'est **pas**
   `/connexion`. Sans lui, la même défaillance reviendrait sans que rien ne le dise.

Trois autres écueils ont été rencontrés et sont documentés dans le fichier : la coupure ne doit
porter que sur l'API (`setOffline` couperait aussi le serveur de pages, que Tauri sert localement),
l'événement `offline` doit être émis (le témoin s'y abonne pour que son passage soit instantané),
et le limiteur de tentatives punit les exécutions rapprochées — une seule connexion par exécution
désormais.

**Un défaut réel du produit en est sorti**, et c'est le point 4 de la liste ci-dessous.

### 2 · Le décompte d'assertions de T055 n'a pas été *comparé* — il a été rendu sans objet

T055 demandait de relever le décompte d'assertions des trois instanciations manuelles **avant**
portage, pour le comparer après. Relevé : 6, 18 et 12 assertions.

La comparaison n'a pas eu lieu, parce que **rien n'a été retiré** : les tests manuels sont
conservés à côté des instanciations. C'est plus fort que la comparaison — on ne peut pas perdre ce
qu'on n'enlève pas — mais ce n'est pas ce que la tâche demandait, et le dire vaut mieux que de
cocher.

Le coût est réel : les trois fichiers portent désormais deux jeux de tests qui se recouvrent en
partie. Le cycle qui voudra les fondre devra faire la comparaison alors.

### 3 · `audit_classe_a.rs` n'est pas porté — et ne peut pas l'être

La macro engendre ses tests **par HTTP** ; le registre des actions n'a aucun endpoint d'écriture.
Instancier aurait demandé d'inventer une opération que personne n'a spécifiée (principe X).

Un contrôle garde cette raison — `aucun_endpoint_d_ecriture_d_audit_n_est_apparu` — et échouera le
jour où le portage deviendra possible **et** dû.

### 4 · WKWebView reste non vérifié, et `crypto.subtle` avec lui

Le `webkit` de Playwright **n'est pas** WKWebView. Le vert de P-22 dit « le produit tourne sur un
moteur WebKit », jamais « vérifié sur la cible ».

Cela vaut particulièrement pour **`crypto.subtle`**, qui exige un contexte sécurisé et sur lequel
repose tout le chiffrement de la file. Si l'API manquait sur WKWebView, le magasin serait
**inerte** — la file vivrait en mémoire seule, sans erreur visible, et ne survivrait pas à une
extinction. Le comportement est écrit et testé ; sa vérification sur la cible viendra avec la
coquille Tauri.

### 5 · Les libellés du catalogue de paramètres n'ont pas tous d'i18n

Les deux clés de ce cycle en ont — P-22 l'a imposé. **Les huit clés antérieures n'en ont
toujours pas** : `politique_impression`, `indicatif_telephonique_defaut`,
`mot_de_passe_longueur_min`, `jeton_acces_duree_min`, `jeton_rafraichissement_duree_jours` et les
trois de HEB ont un `libelle_cle` en base sans entrée correspondante côté application.

C'est une dette antérieure au cycle, révélée par lui. Elle appartient à **ADM-06**, qui livre
l'écran de configuration.

### 6 · La justesse des classes reste humaine

`classes_offline.rs` vérifie qu'une classe est **déclarée**, `outillage_classes.rs` qu'elle est
**exercée**. Aucun des deux ne vérifie qu'elle est **juste** — aucune lecture du schéma ne retrouve
qu'un encaissement est B en espèces et D en Mobile Money.

Instancier `tester_classe_a!` sur une entité qui devrait être B produirait six tests verts sur un
classement faux. La revue mensuelle demeure.

### 7 · L'image de production était FAUSSE, et le défaut est antérieur au cycle

**L'image estampillée `linux/amd64` contenait un binaire `arm64`.** Elle n'aurait pas démarré sur
le VPS Contabo.

La cause est dans `infra/Dockerfile.api` : l'étape de construction portait
`FROM --platform=$BUILDPLATFORM`, qui la fait tourner sur l'architecture de **l'hôte** — pour
éviter l'émulation et gagner du temps. `cargo build` produisait donc un binaire de l'hôte, et
l'étape d'exécution, elle bien en `TARGETPLATFORM`, le copiait tel quel.

Le symptôme rend le diagnostic difficile :

```
exec /usr/local/bin/kaya-api: no such file or directory
```

…sur un fichier de 42 Mo qui existe. C'est le loader qui ne reconnaît pas l'architecture de l'ELF,
pas le fichier qui manque. La confirmation vient de l'en-tête ELF : `e_machine = 0xB7`, soit 183,
soit **AArch64**.

**Le défaut était invisible en intégration continue** — le runner est `amd64`, donc
`BUILDPLATFORM == TARGETPLATFORM`, et l'image sortait juste. Il ne se manifestait que sur le poste
de développement, c'est-à-dire exactement là où le §4.2 du gel dit de se méfier : « le poste est
`arm64`, la cible est `amd64` ».

Corrigé en retirant la directive : l'étape hérite de `TARGETPLATFORM` et compile pour la cible
réelle. Le coût est une compilation sous émulation sur un poste arm64 — lent, et c'est le prix
d'une image qui démarre. **En CI, cela ne coûte rien.**

La compilation croisée (cible `x86_64-unknown-linux-gnu` et éditeur de liens croisé) rendrait les
deux rapides. Elle demande d'ajouter une chaîne d'outils au gel : **aucune story ne la porte
aujourd'hui**, et c'est écrit ici plutôt que fait au passage.

### 8 · La suite backend et le balayage e2e ne peuvent pas tourner ensemble

`exiger_grand_livre_sans_consommateur_concurrent` refuse de dérouler les tests d'outbox quand un
worker de publication tourne hors de `cargo test` — c'est-à-dire quand l'API est allumée, ce que le
e2e exige.

Ce n'est pas un défaut du cycle : le garde-fou est du cycle 001 et il fait son travail. Mais il
impose de **séquencer** les deux suites, et une exécution complète en une passe n'existe pas. La
levée demanderait une base dédiée aux tests, ce qu'aucune story ne porte aujourd'hui.

---

## Les portes, une par une

| Porte | État | Comment |
|---|---|---|
| P-01 | ✓ | `pnpm porte:p01` — client identique au contrat |
| P-01b | ✓ | `couverture_portes.rs` — `operationId` tous distincts |
| P-02 | ✓ | 28 migrations, aucune modifiée |
| P-03 | ✓ | `architecture.rs` sur périmètre **découvert** — `synchronisation` reste sans dépendance vers le haut |
| P-04 | ✓ | aucune requête ne nomme deux schémas de modules |
| P-05 | ✓ | **28** types déclarés et couverts, aucun émis hors liste |
| P-05b | ✓ | aucun chemin de suppression sur les deux registres |
| P-06 | ✓ | non touchée — `capacites_refusees.rs` |
| P-07 | ✓ | `rls_catalogue.rs`, plancher 35, périmètre découvert |
| P-08 | ✓ | 56 opérations, décompte **inchangé** |
| P-09 | ✓ | non touchée |
| P-10 | ✓ | 28 migrations, 122 fichiers Rust, JSONB compris |
| P-11 | ✓ | installée à vide, assertion de non-régression |
| P-12 | ✓ | `architecture.rs` — chemins composés depuis le manifeste |
| P-13 | ✓ | versant type (4 `@ts-expect-error`) **et** versant écran (10 cas e2e × 2 moteurs) |
| P-14 | ✓ | macros du §0.7 + `outillage_classes.rs` |
| P-15 | ✓ | pont natif confiné, 93 fichiers analysés sur trois arbres |
| P-16 | ✓ | 311 clés à parité |
| P-17 | ✓ | aucune couleur ni espacement littéral |
| P-18 | ✓ | double passe + les deux contrôles, `touch` compris |
| P-19 | ✓ | aucune maquette copiée |
| P-20 | ✓ | **aucune dépendance ajoutée** — WebCrypto est une API du moteur |
| P-21 | ✓ | aucune ressource d'hôte externe |
| P-21b | ✓ | **après régénération** — trois glyphes nouveaux manquaient à la police sous-réglée |
| P-22 | ✓ | 10 routes × 2 moteurs × 2 thèmes, 68 cas ; négatif concluant |
| **P-23** | ✓ | **nouvelle** — `horodatage_autorite.rs`, périmètre découvert, trois exemptions relues dans la constitution |

---

## Les défauts trouvés par le cycle, et corrigés

Six, dont quatre qu'aucune relecture n'aurait vus.

1. **`note_etablissement.creee` émis depuis quatre cycles sans être déclaré** — trouvé par
   l'élargissement du périmètre de P-05. Le produit émettait 28 types, la porte en comptait 27, et
   **les deux versants** de la porte étaient aveugles.
2. **Le contrôle de couverture de `G4` ne contrôlait rien.** `const x: FamillesNonListees[] = []` —
   un tableau vide est assignable à n'importe quel type de tableau. Le test annonçait, commentaire
   à l'appui, une garantie qu'il n'avait pas, et 458 tests verts le confirmaient.
3. **Un `401` de rafraîchissement détruisait la file persistée.** La purge totale emportait la clé
   de chiffrement : les écritures non parties devenaient illisibles **y compris après une
   reconnexion réussie** — exactement ce que la règle 2 de `vidage.ts` interdit, en pire.
4. **Le témoin ne comptait pas une saisie hors ligne.** La note entrait bien en file, et
   l'indicateur continuait d'afficher « Hors connexion » sans le nombre : l'utilisateur voyait sa
   saisie disparaître de l'indicateur censé lui dire que son travail est en sécurité — le contraire
   de ce que le composant 10 existe pour faire. Le signal est désormais émis par la file elle-même,
   à chaque mutation ; les cinq points de mutation y passent tous, et l'oubli devient impossible.
   **Trouvé par le balayage e2e**, qu'aucun test de composant n'aurait remplacé.
5. **Une course de persistance.** Deux écritures concurrentes chiffraient chacune leur instantané
   et écrivaient dans l'ordre où elles finissaient : une entrée saisie pouvait disparaître sans
   erreur ni trace.
6. **Le `CHECK` d'égalité de conditions laissait passer un corollaire isolé.** La forme écrite au
   modèle de données acceptait une issue posée sur un constat encore ouvert — trouvé par le test
   qui l'exerce, avant que la migration ne soit figée.

Et **un défaut de conception du témoin**, trouvé par son propre test : un commentaire HTML placé
avant l'élément racine compte comme un nœud, le template compilait en fragment — la famille de
défauts qui a rendu `G3` et `G4` inatteignables au cycle 003.

**Et le septième, qui n'appartient pas à ce cycle mais qu'il a révélé** : l'image de production
embarquait un binaire arm64 sous une étiquette amd64 (non-conformité 7). Il aurait empêché tout
déploiement, et il était invisible en intégration continue.
