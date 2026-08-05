# Implementation Plan: Fiches clients, arrivée, départ et prolongation

**Branch**: `006-clients-sejours-enregistrement` | **Date**: 2026-08-03 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/006-clients-sejours-enregistrement/spec.md`

**Artefacts de phase** : [research.md](./research.md) · [data-model.md](./data-model.md) ·
[contracts/http-api.md](./contracts/http-api.md) · [contracts/traits-exposes.md](./contracts/traits-exposes.md) ·
[quickstart.md](./quickstart.md)

---

## Summary

Ce cycle fait servir les cinq précédents. Il livre **neuf tables** réparties sur **deux schémas
existants**, **dix-sept opérations HTTP**, **neuf types d'événements**, **sept permissions** et
**quatre écrans** — dont deux **maquettés** (`R4` Le passage, `R7` La note et le départ) et deux
**dérivés** et déjà inscrits à `docs/design/derivation.md` (`R3` Arrivée hérite de `R4`, `R5` Fiche
client hérite de `R7`).

**Le cœur du cycle n'est pas une table, c'est un budget.** Le cadrage §5.6 fait de la rapidité du
passage une **condition d'existence** du produit : *« le module de passage doit être irréprochable
en rapidité (moins de 30 secondes) sinon il sera contourné »*. La maquette `R4` traduit cette
contrainte en conception — **deux gestes, la durée puis la chambre**, et la mention normative
« Pièce d'identité : après la clé, pas avant ». Le plan la traduit en architecture : **une seule
transaction, un seul appel réseau bloquant**, tout le reste préchargé.

```
POST /etablissements/{id}/sejours          ← UN appel, UNE transaction
   ├─ attribuer l'unité      (MoteurDisponibilite::attribuer — PREND la transaction, cycle 004)
   ├─ ouvrir le séjour
   ├─ ouvrir la note + sa ligne d'hébergement
   ├─ numéroter et produire la fiche de police
   └─ écrire l'événement outbox
```

Le trait `MoteurDisponibilite::attribuer` a été écrit au cycle 004 **pour ce moment** — sa
documentation le dit mot pour mot : *« c'est ce qui rendra possible au check-in de SEJ-02
d'attribuer l'unité et d'ouvrir la note dans une seule transaction »*. Ce cycle est la première
vérification que cette promesse tient.

**Trois dettes annoncées nommément arrivent à échéance, et deux portes reçoivent une cible :**

1. **Migration `0015` du cycle 003** : `comptes.personne.type_piece` et `numero_piece` sont
   « **POSÉES, NON ALIMENTÉES** — alimentation **SEJ-01**, rétention 90 jours TRX-06 ». Ce cycle
   les alimente. C'est ce qui décide où vit la fiche client (§ Constitution Check, point 1).
2. **Le scénario orphelin du §0.7** — *« toute entité rattachée à un séjour : test du scénario
   orphelin »* — n'avait **aucune cible** depuis cinq cycles, faute de séjour. Il en a une :
   `accompagnant` est de **classe A**, donc écrit hors ligne et mis en file ; il peut arriver après
   la clôture. La table `synchronisation.reconciliation_orpheline`, posée au cycle 005 avec
   `GRANT SELECT` **seul**, reçoit son `INSERT` et **cesse d'être une provision**.
3. **`docs/design/derivation.md`** inscrit `R3` et `R5` depuis sa version 1.2.0 sans qu'aucune
   story ne les appelle. Elles sont appelées.

**Et une porte doit rester à vide, ce qui est le point le plus délicat du cycle.** Le départ **fige
la taxe de séjour**, mais **n'écrit aucune règle fiscale** : il fige un **constat** — les faits et
le paramétrage lus à cet instant — que `JurisdictionAdapter` interprétera en T3. **P-11 doit rester
verte à vide** ; si elle se réveillait, c'est qu'une règle fiscale aurait été écrite ici. Voir
§ Constitution Check, point 2.

---

## Technical Context

**Aucune version n'est proposée ici.** `docs/versions-gelees.md` (gel **1.0.12**, vérifié le
2026-08-02) fait foi, et ses valeurs sont reprises telles quelles. **Ce cycle n'ajoute aucune
dépendance, aucune extension PostgreSQL, aucun plugin natif, aucun outil** — la revue mensuelle du
2026-08-31 n'a rien à trancher de son fait.

**Language/Version** : Rust **1.97.1** (toolchain gelée) · TypeScript **5.9.3** · edition et
`rust-version` héritées du workspace

**Primary Dependencies** — toutes déjà au dépôt : `actix-web 4.14.0` · `sqlx =0.9.0`
(`uuid`, `time`, `rust_decimal`, `sqlx-toml`) · `utoipa =5.5.0` · `uuid =1.24.0` (`v7`) ·
`time =0.3.54` · `rust_decimal =1.42.1` · `futures =0.3.33` (concurrence) · `thiserror =2.0.19` ·
`async-trait =0.1.91` · `serde_json`

> **Le repli des signes diacritiques est écrit à la main, et c'est une décision.** Chercher
> « kouame » et trouver « KOUAMÉ » demande de replier les accents. La bibliothèque naturelle —
> `unicode-normalization` — **n'est pas au gel** : l'ajouter imposerait une entrée nouvelle, donc
> une décision de revue mensuelle, pour un besoin qu'une table de correspondance couvre. Le repli
> est écrit dans le crate, testé sur un jeu de noms ivoiriens réels, et **le produit décide de ce
> qu'il replie** au lieu d'hériter du choix d'une bibliothèque. Voir [research.md R-04](./research.md).

**Storage** : PostgreSQL **18.4**, schémas **`comptes`** (fiche client) et **`hebergement`**
(séjour). Aucune extension nouvelle — `btree_gist`, installée au cycle 001, suffit, et rien
n'appelle `pg_trgm` ni `unaccent` sur dix mille lignes. **Redis n'est pas touché** — rien de ce
cycle n'est éphémère reconstructible. **Garage n'est pas touché** — la photo de client est hors
périmètre, donc aucun objet.

**Testing** : `cargo test --workspace` (intégration sur base réelle) · `vitest 4.1.10` (front) ·
`@playwright/test 1.62.1` — Chromium **1234** et WebKit **rev 2336** pour P-22 et le budget de
temps machine

**Target Platform** : API en Docker `linux/amd64` sur VPS Contabo (mode A du cadrage §10.1). Le
poste de développement est `arm64` : la construction de production se fait **dans Docker pour
`linux/amd64`**, jamais par copie d'un binaire local. **Aucune dépendance native n'étant ajoutée,
la question des deux architectures ne se pose pas** — et c'est vérifié par P-20, pas supposé.

**Project Type** : monolithe modulaire Rust + application Nuxt 4 (SSR désactivé) + Tauri v2

**Performance Goals** :
- **Recherche de fiche client < 300 ms au 95ᵉ centile sur 10 000 fiches**, mesuré côté serveur
  (SC-005, FR-006) ;
- **Passage : deux gestes obligatoires, zéro champ de saisie, au plus UN appel réseau bloquant**
  (SC-001, FR-022, FR-023, FR-031) — critère **déterministe**, gardé en intégration continue ;
- **Part machine du parcours de passage sous un budget déclaré**, mesurée par parcours scripté sur
  les deux moteurs (SC-004), budget fixé très au-dessus de la valeur observée.

**Constraints** : hors-ligne **interdit** sur l'essentiel du cycle — `client` en **C** ; `sejour`,
`note_sejour`, `ligne_sejour`, `fiche_police`, `numerotation_fiche_police` et `taxe_sejour_constat`
en **B** (P-13). Deux entités seulement sont en **A** : `accompagnant` et `preference_personne`.
Montants en entiers d'unité mineure, quantités en `NUMERIC` (P-10). Toute durée depuis
l'horodatage d'autorité serveur (P-23). **Aucune règle fiscale** (P-11, P-12).

**Scale/Scope** : 9 tables · 6 migrations · **17 opérations HTTP** · 9 événements outbox ·
7 permissions · 2 traits exposés, 5 consommés · **4 écrans** (2 maquettés, 2 dérivés) ·
1 provision qui cesse d'en être une

---

## Constitution Check

*GATE — évalué avant Phase 0, réévalué après Phase 1.*

### Principes engagés

| Principe | Ce que ce cycle en fait | Verdict |
|---|---|---|
| **I·a** Contrat généré | 17 opérations utoipa ; client TS régénéré et commité, aucun type redéclaré à la main | ✅ |
| **I·b** Schéma par migrations | 6 migrations versionnées ; seeds **à part**, rejouables | ✅ |
| **I·c** Paramètres métier | **Aucune clé nouvelle** — aucune story du périmètre ne dit « paramétrable ». Le point 9 de la DoD est **sans objet**, et c'est écrit plutôt que supposé | ✅ |
| **II** Hiérarchie des crates | `verticales/hebergement` s'enrichit ; la fiche client vit dans `socle/comptes`. **Aucune arête `socle/ → verticales/`** — le séjour lit le client par un trait, jamais l'inverse | ✅ |
| **II** Un schéma par module | Deux schémas touchés, **aucune jointure entre eux** : `sejour.client_id` est un UUID **sans clé étrangère**, lu par `AnnuaireClients` | ✅ |
| **II** Outbox transactionnel | 9 événements, signature qui **prend** la transaction | ✅ |
| **III** RLS | `ENABLE` + `FORCE` + `USING`/`WITH CHECK` sur les 9 tables | ✅ |
| **IV** Temps et disponibilité | Le séjour **consomme** l'intervalle du cycle 004 ; le changement d'unité produit **deux** occupations, toutes deux protégées par la contrainte d'exclusion. Durées depuis l'horodatage d'autorité | ✅ |
| **V** Argent et fiscalité | `montant_mineur` en `BIGINT`, `quantite` en `NUMERIC`, prix **verrouillé à la ligne**, ajustement = **ligne nouvelle**. **Aucune règle fiscale** — voir point 2 | ✅ |
| **VI** Hors-ligne | 7 entités sur 9 en B ou C, refusées immédiatement. `accompagnant` et `preference_personne` en A. **Écriture orpheline : file de réconciliation, jamais de rejet silencieux** | ✅ |
| **VII** Application unique | Module front chargé paresseusement ; `PlatformAdapter` seul pont natif ; l'entrée d'arrivée est **absente** sans module hébergement | ✅ |
| **VIII** i18n et mode sombre | Clés `fr` et `en` ; quatre écrans vérifiés en clair et en sombre | ✅ |
| **IX** Registres immuables | Le registre des actions reçoit les régularisations et les changements d'unité. **Le constat de taxe est immuable par privilège** : `SELECT, INSERT` seuls | ✅ |
| **X** Prêt ≠ construit | Aucune ligne de SEJ-03, SEJ-05, SEJ-06, RSV, HEB-06. Le montant de taxe est **posé, non alimenté**, avec sa garde | ✅ |
| **XI** Versions | **Aucune dépendance nouvelle** ; gel 1.0.12 repris tel quel | ✅ |
| **XII** Référence visuelle | `R4` et `R7` sont **maquettés** : lus, jamais copiés. `R3` et `R5` sont **inscrits** à `derivation.md` avant d'être codés | ✅ |

### Les quatre points qui méritaient un examen, et non une case cochée

#### 1 · Où vit la fiche client — et pourquoi ce n'est pas une table neuve

La spécification déclare `client` comme entité. Le réflexe est une table portant nom, prénoms,
téléphone, pièce d'identité. **Ce réflexe crée un second fichier d'identités.**

`comptes.personne` porte déjà `nom`, `prenoms`, `telephone`, `email`, `type_piece`, `numero_piece`.
Ces deux dernières sont accompagnées, dans la migration `0015`, d'un commentaire qui décide de ce
cycle :

> *« **POSÉES ET NON ALIMENTÉES PAR CE CYCLE.** Ce sont des données d'identité de client : leur
> alimentation relève de **SEJ-01** (fiche client) et leur rétention de 90 jours de TRX-06. Poser
> la colonne sans la politique de rétention qui va avec serait le moyen le plus simple de
> constituer un fichier d'identités sans durée de conservation — ce qui est exactement ce que
> l'ARTCI interdit. »*

Et `provisions_sans_logique.rs` garde déjà cette propriété dans l'autre sens : il **échoue** si une
colonne dont le nom contient `piece`, `passeport`, `cni` ou `identite` apparaît ailleurs, au motif
que *« recopiées ici, elles y resteraient indéfiniment »*.

**Décision** : la fiche client est **`comptes.personne` qualifiée par `comptes.client`**.
`personne` garde l'identité civile — une seule, sous une seule rétention. `client` porte la
qualification et les deux attributs que CPT n'a aucune raison de connaître (`date_naissance`,
`nationalite`). Les deux vivent dans le **même schéma**, ce qui rend la recherche des trois formes
— nom, téléphone, numéro de pièce — faisable **en une requête**, condition de la cible des 300 ms.
Une table `client` dans un schéma séparé aurait imposé soit une jointure inter-schémas, que **P-04
interdit**, soit deux requêtes, ce qui rend la cible plus difficile pour rien.

**Ce que cela n'entame pas** : CPT-00 distingue l'identité civile (`personne`), l'authentification
(`compte`) et le contrat de travail (`employe`). Un client est une **identité civile sans compte ni
contrat** — le cas est déjà nommé par l'en-tête de `0015` (« une femme de ménage a une fiche et
aucun compte »). La distinction est respectée, pas contournée.

**L'existence de `comptes.client` est ce qui rend la recherche honnête** : elle ne renvoie que des
personnes **qualifiées clientes**, jamais le personnel. Sans elle, chercher « Kouamé » à la
réception ferait apparaître la femme de ménage.

**Non-conformité : aucune.** Deux amendements documentaires sont dus, listés au § Suites.

#### 2 · Le départ fige une taxe — le principe V est-il entamé ?

La porte **P-12** refuse toute règle fiscale hors de `JurisdictionAdapter`, et **P-11** est
installée **à vide** avec une assertion de non-régression : elle échoue dès qu'un jeu de cas
fiscal apparaît dans `backend/tests/fixtures/fiscal`. Ce cycle fige une taxe. Va-t-il la réveiller ?

**Non, et la frontière est nette : ce cycle fige un CONSTAT, il ne calcule aucun montant.**

| Ce que ce cycle écrit | Nature | Qui l'interprète |
|---|---|---|
| `nuits_constatees` — nombre de nuits calendaires de la période | **arithmétique** | — |
| `nombre_personnes` — titulaire et accompagnants | **arithmétique** | — |
| `assujettie_taxe_nuitee`, `regle_conversion_taxe` — **recopiés** de la formule à cet instant | **paramétrage** | FIS-03 |
| `classement_etablissement`, `commune` — recopiés à cet instant | **paramétrage** | FIS-03 |
| `fige_le` — l'instant d'autorité du figeage | **fait** | — |
| `montant_mineur`, `nuitees_assujetties` | **posés, jamais alimentés par ce cycle** | FIS-03 |

**Compter les nuits d'un intervalle est de l'arithmétique. Décider lesquelles sont assujetties est
une règle fiscale.** `une_nuitee_par_occupation` réduit trois nuits à une ; c'est un arbitrage
fiscal, il vit dans l'adaptateur. Ce cycle enregistre **trois** et la règle lue, jamais **un**.

C'est aussi ce qui donne au mot « figé » un contenu vérifiable : tout ce qui pourrait changer après
le départ — accompagnants, barème, formule, classement, commune — est **recopié**. Le montant
calculé plus tard depuis ce constat est donc stable, quelle que soit la date du calcul.
L'immuabilité n'est pas une intention : `taxe_sejour_constat` reçoit `GRANT SELECT, INSERT`
**seuls** — pas d'`UPDATE`, pas de `DELETE`. Une relecture ne peut pas la recalculer ; le rôle
applicatif n'en a pas le droit.

> ⚠️ **Écart assumé avec la spécification, écrit plutôt que glissé.** FR-062 et FR-066 disent
> « nombre de **nuitées assujetties** ». Le plan fige `nuits_constatees` **plus** le paramétrage
> qui décide ; le nombre d'assujetties est la sortie de FIS-03. Le fait garanti par la
> spécification — *l'assiette ne bouge plus après le départ* — est tenu, et il l'est plus
> strictement, puisque même le barème et la commune sont gelés. C'est un raffinement de plan,
> consigné en [research.md R-08](./research.md).
>
> **L'écart porte aussi sur le NOM, et il faut le dire.** La spécification nomme l'entité
> `assiette_taxe_sejour_figee` dans ses « Key Entities » ; le plan et le modèle de données retiennent
> **`taxe_sejour_constat`**, parce que ce qui est écrit est un constat et non une assiette. Ce n'est
> pas cosmétique : `classes_offline.rs` compare des **noms de table** aux entités du registre, et y
> déclarer l'ancien nom ferait échouer le build sans dire pourquoi. Le nom retenu est celui de
> [data-model.md](./data-model.md) et de la tâche T006.

**Vérification** : `portes_a_vide.rs::p11_tests_dores_fiscaux` **doit rester verte**, et
`provisions_sans_logique.rs` gagne une entrée — `taxe_sejour_constat.montant_mineur` et
`nuitees_assujetties` posées, aucun chemin de code ne les écrit, aucune opération ne les expose.

#### 3 · Une entité de classe A qui arrive après la clôture — rejet ou file ?

`accompagnant` est de **classe A** : écrivable hors ligne, donc mis en file, donc susceptible
d'arriver **après** le départ. FR-019 dit qu'un accompagnant n'est pas ajoutable sur un séjour
clos. Le principe VI dit qu'une écriture orpheline va en **file de réconciliation à résolution
humaine**, *« jamais de rejet silencieux, jamais d'ajout d'office »*.

**Les deux sont vrais et ne se contredisent pas** : l'ajout est refusé **comme ajout** et inscrit
**comme orphelin**. La réponse n'est ni `201` ni `409` : c'est `202`, avec le motif, et une ligne
dans `synchronisation.reconciliation_orpheline`.

C'est le **premier cas réel** d'écriture orpheline du produit — le cadrage §11.4 le décrit avec une
consommation de bar sur un séjour facturé (T2), mais l'accompagnant hors ligne le produit dès ce
cycle, et il est plus simple à éprouver. La table existe depuis le cycle 005 avec `GRANT SELECT`
**seul** ; elle reçoit son `INSERT`, et **cesse d'être une provision** : le décompte de
`provisions_sans_logique.rs` passe de six à cinq. Sa **résolution** reste SYN-03, tranche T3 —
ce cycle alimente la file, il ne la vide pas.

#### 4 · Un séjour qui porte des lignes de note — n'est-ce pas SEJ-03 ?

`ligne_sejour` est déclarée au registre §8 avec la mention « SEJ-03 ». SEJ-03 est en T2.

**Le registre déclare des classes d'avance, c'est son usage établi** — les neuf entités de CPT et
les six de HEB y figuraient avant d'exister et ont été « honorées, pas réécrites ». Ce cycle honore
`ligne_sejour` pour son **sous-ensemble hébergement** : la ligne de la période prévue, et les
lignes d'ajustement. Sa classe **B** est reprise telle quelle.

Sans elle, le « calcul final » exigé par SEJ-04 porterait sur une note vide. SEJ-03 ajoutera les
consommations des points de vente, les transferts de charges et les remises — **aucune de ces trois
n'est ici**, et `provisions_sans_logique.rs` vérifie qu'aucun point d'entrée ne les expose.

**Verdict global : aucune violation à justifier.** La section « Complexity Tracking » reste vide.

### Réévaluation après Phase 1 — ce que la conception a fait apparaître

*La constitution demande de rejouer la porte après la conception. Trois points n'existaient pas
avant que le modèle de données ne soit écrit ; un seul demande une décision.*

**a · `accompagnant` porte `type_piece` et `numero_piece` — une seconde surface de rétention.**
C'est le seul point réellement nouveau, et il est **assumé plutôt qu'évité**. La fiche de police
couvre le titulaire **et ses accompagnants** (FR-046) ; un accompagnant n'a pas de fiche client —
lui en créer une pour porter sa pièce ferait entrer au fichier des personnes qui n'ont rien demandé.
Les colonnes vivent donc sur `accompagnant`, avec `piece_capturee_le`, **pour que la rétention de
TRX-06 s'applique là aussi sans migration**. Conséquence à écrire au § Suites : TRX-06 devra purger
**deux** tables, pas une. `provisions_sans_logique.rs` n'est pas contourné — son contrôle porte sur
les **provisions RH** (`employe`, `appareil_enrole`), et `accompagnant` n'en est pas une ; son
périmètre est confirmé, pas élargi.

**b · Aucune clé étrangère ne traverse un schéma, vérifié table par table.** `comptes.client →
comptes.personne` et `comptes.preference_personne → comptes.personne` sont **intra-schéma** ; les
sept tables de `hebergement` ne référencent que `hebergement` ; `sejour.client_id` est un `UUID` nu.
Le modèle ne porte **aucune** arête interdite.

**c · Le principe V tient jusque dans le JSONB.** Les quatre charges utiles financières
(`heb.sejour.ouvert`, `.prolonge`, `.unite_changee`, `.clos`) portent leurs montants en **entiers
d'unité mineure** sous le nommage réservé `<nom>_mineur`, avec la devise au même niveau —
`scripts/ci/types-monetaires.sh` les inspecte. Et **aucun numéro de pièce d'identité n'entre dans
l'outbox** : le grand livre est à rétention illimitée et immuable, une donnée sensible qui y entre
ne peut jamais en sortir, et la rétention de 90 jours de TRX-06 deviendrait inapplicable.

**Verdict après conception : inchangé. Aucune violation, « Complexity Tracking » reste vide.**

---

## Project Structure

### Documentation (this feature)

```text
specs/006-clients-sejours-enregistrement/
├── plan.md                      # Ce fichier
├── spec.md
├── research.md                  # Phase 0 — 17 décisions
├── data-model.md                # Phase 1 — 9 tables, 6 migrations
├── quickstart.md                # Phase 1 — validation exécutable
├── contracts/
│   ├── http-api.md              # 17 opérations
│   └── traits-exposes.md        # 2 traits exposés, 5 consommés
├── checklists/
│   └── requirements.md
└── tasks.md                     # Phase 2 — produit par /speckit-tasks
```

### Source Code (repository root)

```text
backend/
├── migrations/
│   ├── 0029_client_et_preferences.sql      # comptes.client, preference_personne,
│   │                                       #   ALTER personne (2 attributs + 3 colonnes repliées)
│   ├── 0030_permissions_sejours.sql        # 7 permissions + attribution aux rôles
│   ├── 0031_sejour.sql                     # sejour, accompagnant, ALTER occupation ADD sejour_id
│   ├── 0032_note_sejour.sql                # note_sejour, ligne_sejour
│   ├── 0033_fiche_police.sql               # fiche_police + numerotation_fiche_police
│   ├── 0034_taxe_sejour_constat.sql        # ★ SELECT + INSERT SEULS — le figeage par privilège
│   └── seeds/                              # fiches clients + 3 séjours de démonstration
│
├── crates/socle/comptes/src/
│   ├── client/{modele,repository,service,mod}.rs   # fiche client, recherche, préférences
│   ├── client/repli.rs                             # ★ repli des signes diacritiques, écrit à la main
│   └── traits.rs                                   # MODIFIÉ — AnnuaireClients
│
├── crates/verticales/hebergement/src/
│   ├── sejour/{modele,repository,service,mod}.rs   # ★ ouverture, départ, prolongation, changement
│   ├── note/{modele,repository,service,mod}.rs     # note_sejour + ligne_sejour (hébergement seul)
│   ├── police/{modele,repository,service,mod}.rs   # fiche de police + numérotation
│   ├── taxe/{modele,repository,mod}.rs             # constat figé — AUCUNE règle fiscale
│   ├── erreurs.rs                                  # MODIFIÉ — refus du séjour
│   └── traits.rs                                   # MODIFIÉ — LecteurSejour (pour SEJ-03 et FIS)
│
├── api/src/routes/
│   ├── clients.rs                          # opérations 1 à 6
│   ├── sejours.rs                          # opérations 7 à 16
│   └── hebergement_disponibilite.rs        # MODIFIÉ — opération 17, état des unités
│
└── tests/
    ├── client_recherche.rs                 # 3 formes · repli · 300 ms p95 sur 10 000
    ├── sejour_arrivee.rs                   # ★ une transaction, un appel · classe B
    ├── sejour_depart.rs                    # ★ figeage · immuabilité par privilège
    ├── sejour_prolongation.rs              # conflit NOMMÉ · bascule de formule annoncée
    ├── sejour_changement_unite.rs          # deux occupations, un séjour
    ├── sejour_orphelin.rs                  # ★ PREMIÈRE CIBLE du scénario orphelin (§0.7)
    ├── sejour_hors_ligne.rs                # P-13 sur les 17 opérations
    ├── classes_offline.rs                  # MODIFIÉ — 9 entités, plancher relevé
    ├── outillage_classes.rs                # MODIFIÉ — 2 instanciations dues
    ├── couverture_portes.rs                # MODIFIÉ — 5 décomptes
    ├── provisions_sans_logique.rs          # MODIFIÉ — 6 → 5 provisions, 2 colonnes gardées
    ├── horodatage_autorite.rs              # MODIFIÉ — le périmètre découvre les crates nouveaux
    └── isolation_tenant.rs, rls_catalogue.rs,
        outbox_transactionnel.rs, portes_a_vide.rs,
        seeds_rejouables.rs                 # MODIFIÉS

app/
├── pages/passage.vue                       # route /passage  — R4, MAQUETTÉ, 5 états
├── pages/arrivee.vue                       # route /arrivee  — R3, dérivé de R4
├── pages/clients.vue                       # route /clients  — R5, dérivé de R7
├── pages/depart.vue                        # route /depart   — R7, MAQUETTÉ, 3 états
├── modules/sejours/
│   ├── EcranPassage.vue · ChoixDuree.vue · GrilleUnites.vue
│   ├── EcranArrivee.vue · ListeAccompagnants.vue
│   ├── EcranClients.vue · FicheClient.vue
│   ├── EcranDepart.vue  · NoteSejour.vue
│   ├── donnees.ts                          # lectures typées
│   ├── ouvrir-sejour.ts                    # écriture — septième couche
│   └── clore-sejour.ts                     # écriture — septième couche
├── tests/budget-gestes.spec.ts             # ★ SC-001, déterministe
└── core/i18n/{fr,en}.json                  # MODIFIÉS

tests-e2e/
└── passage.spec.ts                         # ★ SC-004 — budget de temps machine, 2 moteurs

docs/
├── design/lexique.md                       # MODIFIÉ — v1.6.0, AVANT le code
├── design/derivation.md                    # MODIFIÉ — R3 et R5 passent d'« inscrit » à « codé »
├── registre-classes-offline.md             # MODIFIÉ — §8 honoré + 4 lignes, §14 O-01, journal v1.4.0
├── taxonomie-audit.md                      # MODIFIÉ — état de la famille 10
├── cadrage-v1.md                           # MODIFIÉ — §9.6, annexe B-10 (décision B-10)
└── user-stories-v1.md                      # MODIFIÉ — FIS-03, FIS-08, récapitulatif
```

**Structure Decision** — la structure existante est reprise sans exception : un crate par domaine
avec `{modele, repository, service}` par agrégat, `traits.rs` à la racine du crate, handlers dans
`api/src/routes/`, tests d'intégration dans `backend/tests/`, module front dans `app/modules/`.
C'est le patron des cycles 002 à 005 ; s'en écarter coûterait la lisibilité sans rien apporter.

**Le seul choix de structure propre à ce cycle** est la répartition sur deux crates : la fiche
client dans `socle/comptes`, le séjour dans `verticales/hebergement`. Il découle du point 1 du
Constitution Check et il est **opposable par la porte P-03** — le socle ne gagne aucune notion de
séjour, de chambre ni de formule.

---

## Portes de CI — comment chacune est vérifiée, et par quel test

*Exigence de la section « Couverture des portes » de la constitution : chaque porte déclare son
périmètre inspecté, vérifie sa complétude, ne modifie pas ce qu'elle inspecte et prouve que sa
cible n'est pas vide.*

### Les portes que ce cycle touche

| Porte | Ce qu'elle vérifie ici | Mécanisme | Cible non vide prouvée par |
|---|---|---|---|
| **P-01** | Client TS identique au commité après 17 opérations nouvelles | `scripts/ci/generer-client.sh`, diff commité | Le client contient les 17 `operationId` nouveaux |
| **P-01b** | **73** `operationId` distincts | `couverture_portes.rs` — décompte relu **du contrat**, jamais d'une constante | 56 → **73**, écart asserté |
| **P-02** | Aucune des 28 migrations antérieures modifiée | `scripts/ci/migrations-figees.sh` | 6 nouvelles, 28 figées. ⚠️ voir la nuance de l'`ALTER TABLE` ci-dessous |
| **P-03** | Aucun crate `socle/` ne dépend de `verticales/` — **alors que `socle/comptes` sert le séjour** | `backend/tests/architecture.rs`, graphe de dépendances | La cible reste non vide : `verticales/hebergement` porte des symboles publics et en gagne |
| **P-04** | Aucune jointure `comptes` × `hebergement` | `scripts/ci/jointures-inter-schemas.sh` — **la paire nouvelle doit entrer dans sa liste**, avec le décompte des requêtes analysées par schéma | ⚠️ **Voir ci-dessous** — la porte la plus sollicitée du cycle |
| **P-05** | 9 événements émis **dans** leur transaction | `outbox_transactionnel.rs` + `couverture_portes.rs` | 27 → **36** types, chacun avec son test ; les modules nouveaux entrent au balayage des émetteurs |
| **P-05b** | Aucune purge de l'outbox ni du registre des actions ; **`taxe_sejour_constat` rejoint la catégorie** | `scripts/ci/outbox-sans-purge.sh` | La porte porte sur la **catégorie**, pas sur une liste : un registre immuable de plus est couvert sans amendement |
| **P-07** | RLS `ENABLE` + `FORCE` + politique sur les **9** tables nouvelles | `rls_catalogue.rs` + `couverture_portes.rs` | 29 → **38** tables |
| **P-08** | Le tenant A ne lit ni n'écrit rien du tenant B, sur les 17 opérations | `isolation_tenant.rs` + `couverture_portes.rs` | 56 → **73** opérations, régime déclaré pour chacune |
| **P-09** | La contrainte d'exclusion **survit** à `sejour_id` et au changement d'unité | `hebergement_disponibilite.rs` **ré-exercé** + `sejour_arrivee.rs` + `sejour_changement_unite.rs` | ⚠️ **Voir ci-dessous** |
| **P-10** | `montant_mineur` en `BIGINT`, `quantite` en `NUMERIC`, clés JSONB monétaires entières | `scripts/ci/types-monetaires.sh` | 4 colonnes monétaires nouvelles, 4 charges utiles financières |
| **P-11** | **DOIT RESTER VERTE À VIDE** — aucun jeu de cas fiscal n'apparaît | `portes_a_vide.rs::p11_tests_dores_fiscaux` | ⚠️ **Voir ci-dessous** |
| **P-12** | Aucune règle fiscale hors `JurisdictionAdapter` | Contrôle existant + revue du module `taxe/` | Le module **recopie** un paramétrage, il n'en dérive rien |
| **P-13** | Aucune opération B ou C atteignable hors ligne — **15 des 17** | `sejour_hors_ligne.rs` (nouveau) | Les 17 opérations sont inspectées ; les **2** de classe A sont **nommées**, jamais omises |
| **P-14** | Rejeu triple et désordre commutatif sur `accompagnant` et `preference_personne` | `tester_classe_a!` × 2, dont `outillage_classes.rs` vérifie l'existence | ⚠️ **La porte gagne ses deuxième et troisième cibles** |
| **P-15** | Aucun `window.__TAURI__` hors `PlatformAdapter` dans le module `sejours` | `pnpm porte:p15`, avec son décompte de fichiers par arbre | Le module `sejours` entre au décompte de l'arbre `app/` |
| **P-16** | Aucune chaîne en dur ; parité `fr`/`en` sur quatre écrans | `pnpm test:i18n` | Clés du module comptées des deux côtés |
| **P-17** | Aucune couleur ni espacement littéral | `pnpm lint:tokens` | Les `.vue` du module sont analysés |
| **P-18** | `cargo sqlx prepare` vert | `scripts/ci/preparer-sqlx.sh` — ⚠️ **double passe obligatoire** : ce cycle écrit des requêtes dans les seeds **et** dans les tests | `git status --short backend/.sqlx` sans **aucune** suppression, puis le check hors ligne **après `touch`** |
| **P-19** | `R4` et `R7` **lus, jamais copiés** ; `R3` et `R5` **inscrits** avant d'être codés | `scripts/ci/maquettes-non-copiees.sh` | Les deux maquettes sont lues ; les deux dérivations existent **depuis la v1.2.0**, donc antérieures au code |
| **P-20** | Aucune dépendance en intervalle ; lockfiles à jour | `scripts/ci/versions-epinglees.sh` | **Aucune dépendance nouvelle** — et c'est le repli écrit à la main qui le permet |
| **P-21 / P-21b** | Aucune ressource d'hôte externe ; tout déclaré est embarqué | `aucune-ressource-externe.sh`, `ressources-embarquees.sh` | Les quatre écrans n'ajoutent ni police ni image ; **les glyphes nouveaux du module sont ajoutés au sous-réglage** avant que P-21b ne les réclame |
| **P-22** | `/passage`, `/arrivee`, `/clients` et `/depart` s'ouvrent **en direct ET par navigation**, sur Chromium **et** WebKit, dans les **deux thèmes** | `scripts/ci/parcours-reel.sh` | Les **quatre** routes sont comptées par projet — un moteur sans cas fait échouer |
| **P-23** | Aucune durée de séjour, aucun calcul de note ni de constat ne lit `horodatage_client` | `horodatage_autorite.rs` — périmètre **découvert** | ⚠️ **Voir ci-dessous** — la cible pour laquelle la porte a été posée |

### ⚠️ P-04 — la porte la plus sollicitée du cycle

Ce cycle est le premier où **deux schémas se parlent sur le chemin chaud**. Un séjour affiche
toujours le nom de son client : c'est la jointure que tout le monde écrirait.

**Elle n'existe pas, et trois mécanismes le garantissent :**

1. **Aucune clé étrangère.** `hebergement.sejour.client_id` est un `UUID` **sans `REFERENCES`** —
   même régime que `comptes.permission.module_code`, dont le commentaire de `0016` dit exactement
   pourquoi : *« `module_code` SANS clé étrangère : ce serait une clé inter-schémas (P-04) »*.
2. **Un trait, `AnnuaireClients`**, exposé par `socle/comptes` et consommé par `hebergement`. Il
   rend un `ClientResume` **par lot d'identifiants** — `resumes(&[Uuid])`, jamais `resume(Uuid)` :
   une signature unitaire produirait N+1 requêtes sur la liste des séjours, et c'est le détail qui
   décide si l'écran de départ s'ouvre en 200 ms ou en deux secondes.
3. **La porte elle-même**, dont la liste doit gagner `comptes` × `hebergement` comme paire
   interdite, avec le décompte des requêtes analysées par schéma — sans quoi une cible vide
   passerait.

**Le sens inverse est plus dangereux et n'a aucun garde-fou naturel** : l'historique des séjours
d'un client (`GET /clients/{id}/sejours`) est servi **depuis `hebergement`**, jamais depuis
`comptes`. Si `socle/comptes` lisait `hebergement.sejour`, ce serait à la fois une jointure
inter-schémas **et** une arête `socle/ → verticales/` — P-04 et P-03 en même temps. L'opération est
donc montée sur le crate `hebergement`, et son chemin HTTP le cache à l'appelant : le contrat est
une façade, pas une carte des crates.

### ⚠️ P-09 — la porte du cycle 004 doit être **ré-exercée**, pas supposée acquise

Deux changements touchent la table protégée :

| Changement | Risque |
|---|---|
| `ALTER TABLE hebergement.occupation ADD COLUMN sejour_id UUID NULL` | Aucun sur la contrainte — mais **le vérifier**, car une migration qui recréerait la table la perdrait |
| Le **changement d'unité** produit deux occupations contiguës sur **deux unités différentes** | Aucun chevauchement, donc la contrainte ne se déclenche pas — **et c'est justement pourquoi il faut un test qui prouve qu'elle se déclencherait** si les unités étaient les mêmes |

**L'exigence 5 de la section « Couverture des portes » s'applique mot pour mot** : *« la couverture
s'étend avec les fonctionnalités : elle doit être re-exercée, pas supposée acquise »*. Trois
assertions sont donc rejouées après les migrations de ce cycle :

1. le type de `periode` est toujours `tstzrange` ;
2. la contrainte `occupation_sans_chevauchement` existe toujours, avec ses deux opérateurs ;
3. deux arrivées concurrentes chevauchantes **passant par le parcours de séjour** — pas par
   l'endpoint nu du cycle 004 — aboutissent à exactement une écriture, et le refus est un
   `ExclusionViolation` sur la contrainte nommée.

L'assertion 3 est nouvelle et c'est la seule qui compte : elle prouve que **la transaction du
check-in n'a pas contourné la garantie**, par exemple par une lecture préalable « cette chambre
est-elle libre ? » qui paraîtrait prudente et rendrait la double attribution improbable au lieu
d'impossible.

### ⚠️ P-11 — la porte qui doit rester verte, et ce que sa rougeur signifierait

`portes_a_vide.rs::p11_tests_dores_fiscaux` échoue dès qu'un fichier apparaît dans
`backend/tests/fixtures/fiscal`. **Ce cycle n'en ajoute aucun**, et c'est un critère de conformité,
pas un effet de bord : si ce cycle avait besoin d'un jeu de cas fiscal, c'est qu'il aurait écrit une
règle fiscale — donc violé P-12.

Le contrôle symétrique est le vrai. Le **versant positif** exigé par la constitution (« toute
interdiction a un versant positif ») est porté par `provisions_sans_logique.rs` : les colonnes
`montant_mineur` et `nuitees_assujetties` **existent** et **restent vides**. Sans ce versant,
supprimer les colonnes suffirait à passer au vert.

### ⚠️ P-14 — la porte gagne ses deuxième et troisième cibles

P-14 n'a **qu'une** cible depuis le cycle 001 : `note_etablissement`. `occupation` est en B,
`journal_audit` est exercé à part. Deux entités de classe A arrivent : `accompagnant` et
`preference_personne`, couvertes par **instanciation**, jamais par réécriture :

```rust
tester_classe_a!(accompagnant,          schema = "hebergement", table = "accompagnant",          …);
tester_classe_a!(preference_personne,   schema = "comptes",     table = "preference_personne",   …);
```

`outillage_classes.rs` **échoue en nommant** l'entité qui aurait une table sans instanciation. Son
en-tête décrit exactement le défaut que cela évite : *« `occupation` a sa table, sa classe déclarée,
et son rejeu n'a jamais vérifié qu'un second envoi n'émet aucun événement outbox : le contrôle
existait pour `note_etablissement`, et il a été perdu à la réécriture »*.

### ⚠️ P-23 — la porte reçoit la cible pour laquelle elle a été posée

Son en-tête l'écrit : *« SEJ et FIS écriront les premières règles de durée de passage et de taxe de
nuitée — exactement les calculs que le principe IV vise. Poser la porte maintenant coûte ce
fichier ; la poser après coûte la revue de deux moteurs déjà écrits. »* **C'est ce cycle.**

Quatre calculs sont dans son viseur, et **aucun** ne lit l'horloge d'un terminal :

| Calcul | Source de l'instant |
|---|---|
| Durée réelle d'un séjour au départ | `now()` de la base, dans la transaction du départ |
| Début d'une occupation de passage | `MoteurDisponibilite`, cycle 004 — déjà garanti |
| Instant de figeage du constat de taxe | `now()` de la base |
| Instant d'un changement d'unité | `now()` de la base |

Le périmètre de la porte est **découvert** (`commun::perimetre`), donc les modules nouveaux y
entrent **sans qu'on les énumère** — la propriété que le cycle 005 a construite pour ce cycle-ci.
Il n'y a rien à ajouter à la porte ; il y a à **vérifier qu'elle voit les fichiers nouveaux**, ce
que son décompte de fichiers inspectés dit.

`horodatage_client` **est** écrit par ce cycle — sur `accompagnant` et `preference_personne`, comme
sur toute écriture de classe A — et c'est permis : **écrire la colonne n'est pas s'appuyer dessus**,
et l'exemption « rendu de l'instant tel que le terminal l'a perçu » est limitativement énumérée.

### ⚠️ P-02 — la nuance de l'`ALTER TABLE`

`hebergement.occupation` gagne `sejour_id` et `comptes.personne` gagne quatre colonnes. P-02
interdit de **modifier une migration déjà appliquée** — pas de modifier une table. Les deux
changements se font donc par migrations **nouvelles** (`0029`, `0031`), et `0015` comme `0025`
restent au bit près.

**Le piège inverse est réel** : la tentation d'« améliorer » le commentaire de `0015` sur les
colonnes de pièce d'identité, maintenant qu'elles sont alimentées. **Ne pas y toucher.** Le
commentaire décrit l'état au cycle 003 et reste vrai de ce cycle-là ; c'est `0029` qui porte la
mise à jour, par `COMMENT ON COLUMN`.

### Les portes que ce cycle ne touche pas, et pourquoi c'est dit

| Porte | Motif |
|---|---|
| **P-06** | Aucune capacité déclarée ; `STOCK`/`SIMPLE` reste le seul couple accepté. La porte garde son périmètre |

---

## Tests d'intégration — dont les tests hors-ligne obligatoires du §0.7

### Par classe, tels que `docs/registre-classes-offline.md` §11 les impose

| Classe | Entités de ce cycle | Tests exigés | Fichier |
|---|---|---|---|
| **A** | `accompagnant`, `preference_personne` | **Rejeu triple** — un enregistrement, **et aucun second événement outbox** · **Désordre** — six ordres, même état final | `tester_classe_a!` × 2, exécutées par l'outillage |
| **B** | `sejour`, `note_sejour`, `ligne_sejour`, `fiche_police`, `numerotation_fiche_police`, `taxe_sejour_constat` | Inatteignable hors ligne · **Concurrence : deux exécutions simultanées, une seule réussit** | `sejour_hors_ligne.rs` · `sejour_arrivee.rs` |
| **C** | `client` | Inatteignable hors ligne · Isolation multi-tenant sur l'endpoint | `sejour_hors_ligne.rs` · `isolation_tenant.rs` |
| **D** | — | Sans objet : aucune opération de classe D. `tester_classe_d!` **reste installée à vide**, et `outillage_classes.rs` le dit explicitement | — |

### ★ Le scénario orphelin — sa première cible en cinq cycles

Le §0.7 impose : *« toute entité rattachée à un séjour : test du **scénario orphelin** (SYN-03) »*.
Aucune entité n'était rattachée à un séjour jusqu'ici. Trois le sont, et **une seule peut réellement
produire un orphelin à ce cycle** :

| Entité | Peut arriver après la clôture ? | Pourquoi |
|---|---|---|
| `accompagnant` | **Oui** | Classe **A** — écrit hors ligne, mis en file, vidé au retour du réseau |
| `preference_personne` | Non | Rattachée au **client**, pas au séjour. Un client n'est jamais clos |
| `ligne_sejour` | Pas encore | Classe **B** — jamais écrite hors ligne au MVP. Le cas du cadrage §11.4 (consommation de bar) arrive avec **PDV, tranche T2** |

`backend/tests/sejour_orphelin.rs` couvre le cas réel, en quatre assertions :

1. un accompagnant émis hors ligne pendant un séjour ouvert, vidé **avant** la clôture, est un ajout
   normal ;
2. le même, vidé **après** la clôture, rend `202` — **ni `201`, ni `409`** ;
3. une ligne apparaît dans `synchronisation.reconciliation_orpheline`, avec le séjour, l'entité, la
   charge utile et le motif ;
4. **le séjour clos est inchangé** : ni accompagnant ajouté, ni constat modifié — ce que le
   privilège garantit déjà, et que le test asserte tout de même, parce qu'une garantie de privilège
   se perd en une ligne de migration.

### Les deux tests transverses permanents

**1. Réseau coupé puis rétabli** au milieu d'une journée d'exploitation simulée. Ce cycle y ajoute
un cas : la file contient un accompagnant, le réseau revient **après** le départ, et la journée se
solde quand même — l'écriture part en réconciliation au lieu de fausser le constat.

**2. Agnosticité du socle (ETB-02c)** — un établissement portant un module fictif minimal, **sans
aucune capacité**, va de la création à la clôture journalière. Ce cycle le met à l'épreuve d'une
manière neuve : **`socle/comptes` sert désormais la fiche client, que le séjour consomme.** Si le
test rougissait, c'est que `comptes` aurait gagné une notion de séjour. Le tenant « Résidence
Test », module hébergement seul, sert de second exercice.

### Tests propres au cycle

| Fichier | Ce qu'il couvre |
|---|---|
| `client_recherche.rs` | Trois formes de recherche · repli des signes diacritiques sur un jeu de noms ivoiriens · apostrophe droite **et** typographique · téléphone avec et sans indicatif · **300 ms au 95ᵉ centile sur 10 000 fiches**, jeu de mesure généré par le test et **jamais chargé dans les tenants de démonstration** · le personnel **n'apparaît pas** dans les résultats |
| `sejour_arrivee.rs` | ★ **Une seule transaction** : une panne simulée après l'attribution ne laisse ni séjour, ni note, ni fiche de police · classe B · **P-09 ré-exercée par le parcours** · client connu → zéro champ ressaisi · passage sans client → séjour valide et fiche de police **déclarée incomplète** |
| `sejour_depart.rs` | ★ Figeage du constat · **immuabilité par privilège** : `UPDATE` et `DELETE` refusés au rôle applicatif · modifier accompagnant, barème, formule, classement ou commune après clôture ne change **aucune** valeur du constat · note arrêtée · rebascule de palier en **ligne d'ajustement**, jamais par modification · **aucune clôture automatique** à l'expiration de la période · **dérive d'horloge de ±1 h** → durée, ajustement et constat identiques au bit près (SC-011 — P-23 analyse le *code*, ce test éprouve le *comportement*) |
| `sejour_prolongation.rs` | Conflit **nommé** — unité et instant de l'occupation suivante · propositions de la même catégorie · franchissement de `seuil_bascule_nuitee_minutes` **annoncé avant confirmation** · prolongation refusée sur séjour clos |
| `sejour_changement_unite.rs` | Deux occupations, **un** séjour · refus si l'unité cible n'est pas libre, **sans déplacement partiel** · tarif propre à chaque période · constat figé **sur l'ensemble du séjour** |
| `sejour_orphelin.rs` | ★ Les quatre assertions ci-dessus |
| `sejour_hors_ligne.rs` | P-13 sur les 17 opérations, les 2 de classe A **nommées** et non omises |
| `provisions_sans_logique.rs` *(modifié)* | `taxe_sejour_constat.montant_mineur` et `nuitees_assujetties` posées, jamais écrites, jamais exposées · le décompte des provisions passe de **6 à 5** — `reconciliation_orpheline` cesse d'en être une |
| `seeds_rejouables.rs` *(modifié)* | Rechargement en une commande, idempotent, avec fiches clients et trois séjours de démonstration — nuitée en cours, passage en cours, séjour clos |
| `app/tests/budget-gestes.spec.ts` | ★ **SC-001, déterministe** : le parcours de passage compte **exactement deux** interactions obligatoires, **zéro** champ de saisie libre obligatoire, et **au plus un** appel réseau bloquant entre le premier geste et la confirmation |
| `tests-e2e/passage.spec.ts` | ★ **SC-004** : part machine du parcours sous un budget déclaré, sur Chromium **et** WebKit. Le budget est fixé **très au-dessus** de la valeur observée — un seuil serré rougirait au hasard et serait désactivé dans le mois (leçon SC-004 du cycle 004) |

---

## Séquencement — ce qui ne peut pas être parallélisé

1. **Le lexique AVANT le code.** `docs/design/lexique.md` gagne sa v1.6.0 — client, accompagnant,
   séjour, fiche de police, arrivée, départ, prolongation, changement de chambre, note arrêtée — en
   `fr` **et** `en`. Précédent explicite du cycle HEB : « le mot est inscrit avant d'être codé ».
   **Les routes en dépendent** : `/passage`, `/arrivee`, `/clients`, `/depart` — jamais
   `/check-in`, une URL étant visible (leçon `S1` du cycle 005).
2. **Les amendements de la décision B-10 AVANT les migrations.** `0034` recopie un paramétrage
   fiscal ; l'écrire pendant que trois sources de rang supérieur disent l'inverse produirait un
   désaccord dans le même changement.
3. **`0029` → `0030` → `0031` → `0032` → `0033` → `0034`**, dans cet ordre. `sqlx` refuse une
   version antérieure à une version déjà appliquée (constaté au cycle 001). `0031` dépend de `0029`
   et de `0025`.
4. **Les permissions (`0030`) avant les handlers**, sans quoi `exiger(...)` garde un code
   inexistant et le refus devient indistinguable d'une faute de frappe.
5. **Le contrat avant les écrans**, et le client TypeScript régénéré entre les deux. La septième
   couche l'exige : aucun type redéclaré à la main, aucun `as unknown as`.
6. **Les seeds après le référentiel complet**, et **jamais par migration** : une table en
   `FORCE ROW LEVEL SECURITY` accepte un `INSERT` de migration **en n'écrivant rien**, sans erreur.
7. **`couverture_portes.rs` en dernier.** Il suppose que toutes les migrations, toutes les
   opérations et tous les événements existent ; le lancer plus tôt compterait juste et couvrirait
   faux — son propre en-tête le dit.
8. **`cargo sqlx prepare` à DEUX passes**, puis les **deux** contrôles dans l'ordre — le second
   précédé du `touch` qui force la réévaluation des macros, sans quoi il affiche `Finished` sans
   consulter `.sqlx` (constaté au cycle 004).
9. **La suite backend et le e2e ne coexistent pas.** `exiger_grand_livre_sans_consommateur_concurrent`
   refuse de dérouler les tests d'outbox quand un worker de publication tourne hors de
   `cargo test` — c'est-à-dire quand l'API est allumée, ce que `passage.spec.ts` exige. Séquencer,
   et arrêter l'API **par port** : `lsof -ti:8080 | xargs kill`.

---

## Suites documentaires dues

| Document | Ce qui change | Bloquant ? |
|---|---|---|
| `docs/design/lexique.md` | v1.6.0 — le vocabulaire du cycle, `fr` et `en`, **avant le code** | **Oui** (séquencement 1) |
| `docs/cadrage-v1.md` §9.6, annexe B-10 | Décision B-10 : la taxe est due **par séjour**, pas par personne ; B-10 passe de « ouverte » à « close » | **Oui** (séquencement 2) |
| `docs/user-stories-v1.md` FIS-03, FIS-08, récapitulatif | Même correction ; la référence à « B-02 » du récapitulatif est **erronée**, c'est B-10 | **Oui** |
| `docs/registre-classes-offline.md` §14 | O-01 tranchée, option (a) : `client` reste en **C** | **Oui** |
| `docs/registre-classes-offline.md` §8 | Entités déclarées **honorées** ; **quatre lignes ajoutées** pour les tables que le registre ne nommait pas : `preference_personne`, `note_sejour`, `numerotation_fiche_police`, `taxe_sejour_constat`. Journal v1.4.0 | Non — fin de cycle |
| `docs/taxonomie-audit.md` | **Deux mouvements en sens inverses** : la famille 10 `forcage_disponibilite` reste **« due »** — aucun forçage n'est livré, et le dire vaut mieux que la laisser ambiguë ; une **famille nouvelle** naît **« branchée »**, la *consultation d'un numéro de pièce d'identité*, imposée par FR-012. Décompte de **onze à douze** | Non — fin de cycle, sauf la famille nouvelle qui naît **avec** son chemin de code |
| `docs/design/derivation.md` | `R3` et `R5` passent d'« inscrit » à « codé » | Non — fin de cycle |
| `docs/user-stories-v1.md` **TRX-06** | La rétention de 90 jours du numéro de pièce portera sur **deux** tables — `comptes.personne` **et** `hebergement.accompagnant`. Découvert en Phase 1 (réévaluation, point a) | Non — mais **à écrire avant que TRX-06 ne soit spécifié**, sinon la purge en oubliera une |

---

## Complexity Tracking

*Aucune violation de la constitution à justifier.* La section est laissée vide intentionnellement —
les quatre points examinés en § Constitution Check se résolvent par lecture des textes, pas par
dérogation.
