# Phase 0 — Recherche : file hors-ligne, classification et horodatage d'autorité

**Cycle** : 005 (SYN) · **Date** : 2026-08-02 · **Spec** : [spec.md](./spec.md)

Quatorze décisions. Chacune donne ce qui est retenu, pourquoi, et ce qui a été écarté. Les
versions ne sont **jamais** proposées ici : `docs/versions-gelees.md` fait foi et ce cycle
n'introduit **aucune dépendance nouvelle** (voir R-06 et R-14).

---

## R-01 — Le module d'énumération partagé : où vit-il, et que lit-il ?

**Décision.** Un module `backend/tests/commun/perimetre.rs`, exposé par `commun/mod.rs`, avec
deux fonctions et une seule règle : **rien n'y est écrit à la main**.

| Fonction | Source lue | Pourquoi cette source |
|---|---|---|
| `schemas_applicatifs()` | `pg_namespace`, moins une liste d'exclusion **nommée et justifiée** | La base est l'autorité sur ce qui existe ; un schéma créé par migration y apparaît sans que personne y pense |
| `crates_du_socle()`, `crates_des_capacites()`, `crates_des_verticales()` | `[workspace] members` de `backend/Cargo.toml` | Le manifeste est la source de vérité du principe I·b ; parcourir le système de fichiers verrait un répertoire abandonné non déclaré |

**Rationale.** Dix fichiers de tests énumèrent aujourd'hui leur propre périmètre, dont six
portent **21 occurrences** d'un chemin de crate écrit en dur. Le motif a produit deux trous
réels — schéma `comptes` invisible au cycle 003, schéma `hebergement` invisible au cycle 004 —
et un troisième était certain. Le précédent qui marche existe déjà dans le dépôt :
`tests-e2e/routes.ts` lit ses routes de `app/pages/` et **refuse bruyamment** ce qu'il ne sait
pas traduire. Ce module transpose exactement ce comportement côté serveur.

**L'exclusion est le point délicat, et elle est écrite plutôt que devinée.** Sont exclus
`pg_catalog`, `information_schema`, `pg_toast*`, `public` et le schéma des migrations sqlx —
chacun avec son motif en commentaire. La liste est **d'exclusion**, jamais d'inclusion : un
schéma nouveau est inspecté par défaut, et c'est l'inverse qui demande une justification.

**Alternatives écartées.**

- *Corriger seulement `classes_offline.rs`.* Traite le symptôme le plus visible et laisse neuf
  fichiers dans le même état. Le prochain cycle rouvrira la même plaie ailleurs.
- *Parcourir `backend/crates/*/*/Cargo.toml`.* Verrait un crate non déclaré au workspace — donc
  non compilé — et le compterait comme couvert. Un faux positif silencieux.
- *`cargo metadata` en sous-processus.* Correct mais lourd : plusieurs secondes par test, et une
  dépendance à l'outillage là où la lecture d'un fichier TOML suffit.

---

## R-02 — Comment distinguer un schéma *applicatif* d'un schéma système ?

**Décision.** Filtrage par **exclusion nommée** (R-01), complété par un **contrôle de
non-régression** : le module compte les schémas trouvés et échoue si le total **baisse**. Un
schéma qui disparaît est soit une migration destructrice, soit un filtre devenu trop large — les
deux méritent un échec.

**Rationale.** Le filtrage par propriétaire (`nspowner = kaya_owner`) a été envisagé et écarté :
il lie le périmètre d'une porte à la configuration d'un rôle, donc à `preparer-base.sh`, et un
changement de rôle applicatif viderait silencieusement la cible. L'exclusion nommée ne dépend que
de PostgreSQL lui-même.

---

## R-03 — L'horodatage d'autorité : nouvelle colonne, ou convention existante ?

**Décision.** **Aucune colonne nouvelle.** L'horodatage d'autorité est déjà en place sous le nom
`cree_le TIMESTAMPTZ NOT NULL DEFAULT now()` sur les tables métier, et `survenu_le` sur l'outbox.
Ce cycle ne renomme rien : il **rend la règle opposable** par une porte, et documente la
convention au module doré.

**Rationale.** Le module doré pose déjà les deux horodatages côte à côte, avec le commentaire qui
dit lequel fait autorité, et l'ordre de tri (`ORDER BY cree_le DESC, id DESC`) qui l'applique.
Renommer `cree_le` en `horodatage_autorite` sur quatre tables coûterait quatre migrations, une
régénération de contrat et une réécriture de client — pour un gain nul : ce n'est pas le nom qui
manquait, c'est l'**interdiction vérifiée** d'employer l'autre.

**`now()` est conservé, et le choix est nommé.** Dans une transaction, `now()` rend l'instant du
**début de transaction** — deux notes créées dans la même transaction partagent donc leur
horodatage. C'est voulu et déjà documenté au module doré : le départage se fait par l'UUID v7,
ordonné dans le temps. `clock_timestamp()` donnerait des instants distincts mais rendrait
l'horodatage dépendant du moment d'exécution de l'instruction dans la transaction, ce qui est une
propriété plus difficile à raisonner pour un gain de départage que l'identifiant assure déjà.

---

## R-04 — La dérive : calculée où, stockée où, signalée comment ?

**Décision en trois temps, chacun à l'endroit qui a l'information.**

1. **Calcul — côté serveur, à l'ingestion.** Seul endroit où les deux horodatages coexistent et
   où l'horloge est fiable. Fonction pure `constater_derive(client, autorite, seuil) -> Option<Derive>`,
   sur la **valeur absolue** de l'écart : une horloge en retard est aussi fausse qu'une horloge en
   avance.
2. **Stockage — aucun, par ligne.** La dérive n'est pas une colonne. Elle est recalculable à tout
   instant depuis les deux horodatages déjà persistés, et une colonne de plus sur chaque table
   métier serait une donnée dérivée dupliquée — exactement ce que le principe I interdit.
3. **Signalement — deux canaux, deux publics.** Au **registre des actions** (nouvelle famille
   d'audit), pour qu'Adjoua et le propriétaire puissent constater après coup quel terminal
   déviait. Et **au terminal**, en langue utilisateur, calculé par le client depuis l'horodatage
   d'autorité que la réponse porte déjà.

**Le point qu'on écrirait mal : l'inondation du registre.** Deux cents écritures pendant un
service produiraient deux cents entrées d'audit identiques, et le registre deviendrait
illisible — donc inutilisé. Le signalement est **débrayé par une clé Redis à durée de vie**,
portant `(tenant, compte, appareil)` : une entrée par épisode de dérive, pas une par écriture.
Redis est légitime ici au sens du principe II — la clé est **éphémère reconstructible** : la
perdre produit une entrée d'audit de plus, jamais une donnée manquante.

**Le client, lui, n'a besoin d'aucun canal supplémentaire.** Il compare sa propre horloge à
l'horodatage d'autorité de la réponse — et c'est précisément ce qu'il faut : il apprend que
**son** horloge est fausse, ce qui est l'information utile à la personne qui tient le terminal.

---

## R-05 — Où vit la détection dans la hiérarchie des crates ?

**Décision.** La fonction pure dans `socle/synchronisation` ; l'écriture d'audit **chez
l'appelant**, via un trait exposé.

**Rationale — c'est une contrainte de hiérarchie, pas une préférence.** L'ordre réel des
dépendances est :

```text
socle/synchronisation  ←  socle/etablissements  ←  socle/comptes
   (outbox, dérive)         (tenant_context)        (JournalAudit)
```

`JournalAudit` vit dans `comptes`, qui dépend de `synchronisation`. Faire écrire l'audit **par**
`synchronisation` créerait un cycle de dépendances — refusé par le compilateur, et par la porte
P-03 avant lui. `synchronisation` expose donc `constater_derive()` (sans aucune dépendance) et le
trait `SignalDerive` ; la couche API, qui connaît tout le monde, câble l'un sur l'autre. C'est le
même montage que `OutboxWriter` et `EstablishmentDirectory`, déjà éprouvé.

---

## R-06 — La persistance de la file : quel support, et quelle garantie réelle ?

**Décision.** **Chiffrement de la charge utile par WebCrypto (AES-GCM), clé détenue dans
`PlatformAdapter.stockageSecurise`, cryptogramme rangé dans le stockage persistant ordinaire de
la plateforme.**

**Rationale — le coffre système n'est pas un magasin.** Keystore et Keychain sont conçus pour des
secrets courts et peu nombreux ; y ranger une file de plusieurs centaines d'entrées, réécrite à
chaque saisie, est un usage qu'ils ne servent pas bien. Le montage retenu met dans le coffre ce
pour quoi il est fait — **une clé** — et laisse le volume au stockage ordinaire, où il est
illisible sans elle.

**Aucune dépendance nouvelle.** WebCrypto est une API du moteur, présente sur les quatre cibles.
`docs/versions-gelees.md` reste inchangé.

**⚠️ Deux limites, écrites plutôt que découvertes.**

- **Sur le web, la garantie reste `aucune`**, et le type le dit déjà : `stockage-web.ts` déclare
  `garantie: 'aucune'`. Une clé rangée là est accessible à tout script de la même origine. Le
  produit ne prétendra pas le contraire — l'appelant lit `NiveauGarantieStockage` avant de
  décider, et c'est précisément pourquoi ce niveau est porté **dans le type**.
- **`crypto.subtle` exige un contexte sécurisé.** Tauri sert l'application depuis un protocole
  personnalisé qui en est un ; c'est vérifié par la porte de parcours sur les deux moteurs, mais
  **pas sur WKWebView**, qui n'est pas le WebKit de Playwright. À revérifier avec la coquille
  Tauri, comme tout le reste du parcours.

**Alternatives écartées.**

- *File en clair dans IndexedDB.* Interdit par FR-013, et le motif est le cycle suivant :
  l'extraction OCR d'une pièce d'identité est de classe A, donc éligible à la file, et produit des
  données d'identité.
- *File entière dans le coffre système.* Ne passe pas à l'échelle, et échouerait d'abord sur
  Android d'entrée de gamme — la cible d'Aminata.
- *Chiffrement applicatif avec une bibliothèque tierce.* Ajouterait une dépendance native à
  vérifier sur deux architectures, pour remplacer une API du moteur qui fait la même chose.

---

## R-07 — Le retour au premier plan : quel signal, et par où passe-t-il ?

**Décision.** Une capacité nouvelle de `PlatformAdapter` : `surRetourPremierPlan(rappel)`,
implémentée par les quatre adaptateurs, rendant une fonction de désabonnement.

**Rationale.** Le principe VII range le réseau et le cycle de vie parmi les capacités qui passent
par l'adaptateur. Un composant qui poserait lui-même un écouteur `visibilitychange` marcherait
dans un navigateur et devrait être rouvert le jour où Tauri fournit un signal de fenêtre plus
fin — ce qu'il fait déjà sur desktop. Le montage est identique à `useEtatReseau()`, qui existe et
qui a raison.

**Sur web et Tauri, le signal n'est pas le même, et c'est l'objet de l'adaptateur** :
`visibilitychange` + `focus` côté navigateur, événements de fenêtre côté Tauri desktop, reprise
d'activité côté mobile. `BGTaskScheduler` et `WorkManager` **ne sont pas ici** : ils sont MOB-06,
hors périmètre, et le produit doit être complet sans eux.

---

## R-08 — Qui alimente l'état « dégradé » ?

**Décision.** Un observateur d'appels dans la couche client typée : chaque aller-retour dépose
son issue et sa durée. `etatReseau()` consulte cet observateur **en plus** de `navigator.onLine`.

```text
plateforme dit « hors ligne »                      → hors_ligne
plateforme dit « en ligne » ET dernier appel KO    → degrade
plateforme dit « en ligne » ET dernier appel > S   → degrade      (S paramétrable, défaut 3 s)
sinon                                              → connecte
```

**Rationale.** `app/core/platform/reseau.ts` porte déjà, en commentaire de tête, la phrase
« le cycle SYN alimentera [`degrade`] depuis les échecs réels de requête — il n'est produit par
personne aujourd'hui, et c'est écrit plutôt que supposé ». Ce cycle honore cette ligne. À
Abengourou, une 3G qui affiche « en ligne » sans porter la moindre requête est le cas courant :
sans ce troisième état, le témoin mentirait exactement au moment où il compte.

**Le seuil est un paramètre, pas une constante** — cohérent avec le principe I·c, et inscrit au
catalogue avec le seuil de dérive (R-13).

---

## R-09 — L'envoi opportuniste : quels déclencheurs, sans scrutation ?

**Décision.** Quatre déclencheurs, **aucune minuterie de scrutation**.

1. Retour au premier plan (R-07) — le déclencheur **par défaut**, celui qui doit suffire seul.
2. Passage de l'état réseau à `connecte`.
3. Après une écriture réussie — la file profite d'un réseau qu'on vient de constater bon.
4. Après un échec, **réessai à intervalle croissant plafonné**, jusqu'au prochain déclencheur
   naturel.

**Rationale.** Une minuterie qui réveille la radio toutes les trente secondes sur un Android
d'entrée de gamme coûte de la batterie pendant tout un service, pour un gain que le retour au
premier plan couvre déjà. Le cadrage est explicite : la file est **conçue** pour se vider au
premier plan, sur toutes les plateformes, et le reste est optimisation.

---

## R-10 — Quarantaine : quelle frontière entre « à réessayer » et « définitif » ?

**Décision.** La frontière est le **code de réponse**, et elle est écrite une fois.

| Issue | Traitement | Motif |
|---|---|---|
| Réseau injoignable, délai dépassé | **réessai** | Rien n'a été décidé côté serveur |
| `408`, `429`, `5xx` | **réessai** | Le serveur dit lui-même « plus tard » |
| `401` | **rafraîchissement de session**, file **intacte** | Traité par le point de sortie unique, jamais par un vidage |
| `400`, `403`, `404`, `409`, `422` | **quarantaine** | Le serveur a décidé, et rejouer ne changera pas sa décision |
| `200`, `201` | **retrait de la file** | Y compris `200` — c'est un rejeu réussi, pas une erreur |

**Le cas qu'on écrirait mal est `200`.** Une file qui traiterait « déjà présente » comme un
conflit remettrait l'écriture en tête et boucllerait indéfiniment. Le patron du cycle 001 rend
`200` avec la ligne telle qu'elle est en base précisément pour que ce cas soit **le chemin
normal**.

**La quarantaine n'est pas un cimetière.** Elle est consultable (écran `S1`), porte son motif en
langue utilisateur branché sur le `code` d'erreur — jamais sur le `message` de diagnostic —, et
**cesse de bloquer** les écritures suivantes comme le geste de passer la main.

---

## R-11 — Le balayage en direct de FR-005b : comment découvrir les écrans d'écriture ?

**Décision.** Le périmètre est **croisé entre deux sources déjà existantes**, aucune écrite à la
main :

1. le **contrat OpenAPI** donne toutes les opérations non-`GET` ;
2. le **registre des classes hors-ligne** donne la classe de chacune ;
3. `tests-e2e/routes.ts` donne les écrans.

La porte exige que **toute opération non-`GET` de classe B, C ou D soit couverte par au moins un
cas en direct, réseau coupé**, et rapporte le nombre d'opérations couvertes face au total. Une
opération non couverte fait échouer la porte en se nommant.

**Rationale.** C'est la transposition littérale de l'exigence 2 du § « Couverture des portes ».
Une liste d'écrans écrite à la main aurait laissé passer le septième, et deux précédents dans ce
dépôt disent que ce n'est pas une crainte théorique.

**Limite assumée et écrite dans la porte** : elle vérifie qu'une **annonce d'indisponibilité
apparaît**, pas que sa formulation est la bonne. La justesse du libellé relève du lexique et de
P-16 ; les confondre donnerait une porte qui ment sur ce qu'elle garantit.

---

## R-12 — La table de réconciliation : nom, états, et jusqu'où va la provision

**Décision.** `synchronisation.reconciliation_orpheline`, **table et contraintes seulement**.
Aucun endpoint, aucun service, aucun écran. Inscrite à `provisions_sans_logique.rs`, dont le
décompte passe de cinq à six.

**États du cycle de vie** — deux, pas davantage :

```text
constatee ──► resolue
```

**Issues de résolution** — trois, celles du cadrage §11.4 et de SYN-03, en majuscules françaises :
`AVOIR_REFACTURATION`, `PRISE_EN_CHARGE`, `RATTACHEMENT_SEJOUR_SUIVANT`. Nullable tant que l'état
est `constatee` ; la contrainte d'égalité de conditions le porte en base, comme le classement
d'établissement le fait pour ses étoiles.

**Privilèges — c'est là que la provision se prouve.** `kaya_app` reçoit `SELECT` **et rien
d'autre** : ni `INSERT`, ni `UPDATE`. Le registre déclare pourtant la création en **A** et la
résolution en **B** — les deux classes restent justes et attendent leur cycle. Accorder l'`INSERT`
dès maintenant serait l'« ajout d'un petit endpoint » que `provisions_sans_logique.rs` existe pour
rendre bruyant.

---

## R-13 — Les deux paramètres, et où ils sont déclarés

**Décision.** Deux clés au catalogue `etablissements.parametre_catalogue`, sur le patron de la
migration `0023` :

| Clé | Type | Portée la plus basse | Défaut | Story |
|---|---|---|---|---|
| `sync.derive_horloge_seuil_secondes` | entier | établissement | **300** (5 min, cadrage §11.4) | SYN-04 |
| `sync.latence_degradee_seuil_ms` | entier | établissement | **3000** | SYN-02 |

Toutes deux ajoutées au **récapitulatif des paramètres d'établissement** de
`docs/user-stories-v1.md` §708 **dans le même changement**, comme le principe I·c l'exige.

**Rationale du premier.** Le cadrage écrit « 5 minutes ». Le projet n'inscrit aucune valeur métier
en dur (principe I·c) : la valeur du cadrage devient le **défaut** de la clé, pas une constante.
Un établissement dont le parc de terminaux est mauvais pourra le resserrer sans livraison.

**Rationale du second.** Sans seuil, l'état « dégradé » n'est pas testable, et une porte ne peut
pas le distinguer de « connecté ». Le rendre paramétrable coûte une ligne de plus dans la même
migration.

---

## R-14 — L'outillage de test du §0.7 : macros Rust, et utilitaires TypeScript

**Décision — deux formes, parce que les deux versants ne se testent pas au même endroit.**

**Côté serveur : des macros déclaratives**, dans `backend/tests/commun/classes.rs`.

```rust
tester_classe_a!(note_etablissement, schema = "etablissements", creer = …);
tester_classe_c!(etablissement_module, schema = "etablissements", operation = …);
tester_classe_d!(certification_fne, …);          // installée à vide, non-régression comprise
```

Chaque macro engendre les tests que le §0.7 impose pour sa classe : rejeu triple et désordre sur
les **six** ordres pour A ; inatteignabilité hors ligne pour B, C et D ; double soumission au
retour du réseau pour D. **Aucune logique de test n'est réécrite par le cycle appelant.**

**Côté application : des utilitaires**, dans `app/tests/commun/classes.ts` — la marque de classe,
le refus d'enfilement et l'annonce avant saisie se vérifient là où ils vivent.

**Le contrôle qui empêche l'oubli.** Un test parcourt le registre, en extrait toute entité ayant
une table réelle, et échoue si elle n'a **aucune** instanciation correspondant à sa classe. C'est
le pendant exact de `classes_offline.rs` : celui-là vérifie qu'une classe a été **déclarée**,
celui-ci qu'elle a été **exercée**.

**Les trois instanciations manuelles existantes sont portées, avec un garde-fou.** Le décompte
d'assertions de `note_etablissement_classe_a.rs`, `audit_classe_a.rs` et
`hebergement_hors_ligne.rs` est relevé **avant** portage et comparé **après** : une macro qui
couvrirait moins que le code qu'elle remplace transformerait une réécriture en régression
silencieuse. C'est la faute la plus probable de tout ce cycle.

**Alternative écartée.** *Une fonction générique paramétrée plutôt qu'une macro.* Les tests
d'intégration de ce dépôt sont des `#[tokio::test]` nommés, et le nom du test est ce que la CI
affiche quand il tombe. Une fonction générique produirait un seul test dont le message d'échec
dirait « un des six ordres a échoué » sans dire lequel. La macro engendre **six tests nommés**.

---

## Ce que cette recherche N'A PAS tranché, et à qui la question revient

- **O-01 — `client` / `personne` en classe C.** L'extraction OCR d'une pièce d'identité est de
  classe **A**, donc éligible à la file livrée ici ; la fiche `client` qu'elle alimente est de
  classe **C**. Une extraction faite hors ligne ne pourra donc pas créer sa fiche. La décision
  appartient au cycle **SEJ** et son échéance est SEJ-02. **Rien dans ce cycle ne la préjuge** —
  la file ne connaît que des types d'opération déclarés, et en ajouter un est une décision
  explicite.
- **La divergence de `docs/design/derivation.md` sur l'écran `S1`.** La matrice le fait dériver du
  « Composant 8 », or le composant 8 de `docs/design/composants.md` est la **ligne de liste** et
  le **témoin de synchronisation** y porte le n° **10**. C'est vraisemblablement une référence
  restée sur une numérotation antérieure. La corriger relève de la documentation de design, pas
  d'une décision technique : elle est **signalée au plan**, à trancher avant de coder `S1`.
