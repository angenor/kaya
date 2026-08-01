# Quickstart — valider le cycle 003 (CPT)

*Guide de validation, pas de mise en œuvre. Treize vérifications, dans l'ordre où elles doivent
passer. Chacune dit ce qu'elle prouve — une commande verte dont on ignore ce qu'elle affirme ne
prouve rien.*

---

## Prérequis

```sh
docker compose -f infra/compose.yml up -d        # Postgres 18.4, Redis 8.8.1, Garage 2.3.0
bash scripts/dev/preparer-base.sh                # rôles, schémas, migrations
bash scripts/dev/preparer-stockage.sh            # buckets Garage
```

**Redis n'est plus optionnel en développement** : la liste de révocation est consultée à chaque
requête authentifiée. Une API démarrée sans Redis joignable refuse les appels au lieu de les
laisser passer — c'est un choix, et il est testé.

**Deux variables d'environnement nouvelles**, sans lesquelles l'API refuse de démarrer
(research R-05) :

```sh
export KAYA_JWT_CLE=<au moins 32 octets>         # clé de signature — JAMAIS dans le dépôt
export KAYA_SEEDS_MOT_DE_PASSE=<mot de passe de démonstration>
```

**La variable `KAYA_CONTEXTE_PAR_EN_TETES` n'existe plus.** Si elle est encore posée dans un
`.env` local, la retirer : la dérogation du cycle 001 est levée par ce cycle (research R-04). Un
binaire qui démarre encore avec elle est un binaire qui n'a pas été rebâti.

---

## 1 · Le socle compile et les requêtes sont vérifiées

```sh
cd backend
cargo sqlx prepare --workspace -- --all-targets
git status --short backend/.sqlx        # AUCUNE suppression ; que des ajouts
SQLX_OFFLINE=true DATABASE_URL= cargo check --workspace --all-targets --locked
```

**Ce que ça prouve** : les requêtes du cycle sont au cache et le build reproduit celui de l'image.
**L'ordre des deux contrôles n'est pas interchangeable** — `cargo sqlx prepare` vide le cache
silencieusement s'il ne recompile rien, et un cache amputé d'une requête inutilisée par le
`check` passerait le second contrôle seul.

---

## 2 · Les dix tables existent, sont isolées, et sont déclarées

```sh
cargo test --test rls_catalogue          # P-07 : ENABLE + FORCE + au moins une politique
cargo test --test classes_offline        # table réelle → registre, sens table → document
```

**Ce que ça prouve** : aucune des dix tables n'échappe à la RLS, et aucune n'est absente de
`docs/registre-classes-offline.md`. Le décompte est comparé au total attendu — une porte dont la
cible est vide passe toujours.

**Attendu** : `rls_catalogue` inspecte désormais **26 tables** (16 existantes + 10 de ce cycle),
dont **quatre référentiels globaux** au régime nommé — `methode_authentification`, `role`,
`permission`, `role_permission` — comptés conformes et non exemptés.

---

## 3 · Trois tables distinctes, et rien ne confond compte et employé

```sh
cargo test --test personne_compte_employe
cargo test --test provisions_sans_logique
```

**Ce que ça prouve** — c'est la vérification la plus importante du cycle, et la seule dont
l'échec ne se verrait sur aucun écran :

- les trois figures de CPT-00 fonctionnent — employé sans compte, compte sans contrat, les deux ;
- **aucune colonne de contrat, de salaire, d'embauche ou de CNPS n'existe hors de `employe`** ;
- **aucun chemin de code ne lit `employe`** pour décider d'un droit ;
- `employe` et `appareil_enrole` n'ont **aucun privilège d'écriture** pour le rôle applicatif, et
  **aucun point d'entrée d'API** ne les touche.

---

## 4 · Les deux échecs de connexion sont indiscernables

```sh
cargo test --test authentification_indiscernable
```

**Ce que ça prouve** : sur 100 tentatives de chaque type — compte inexistant et mot de passe faux
— le message, le code de retour **et la médiane du temps de réponse** sont du même ordre. Le test
échoue si le rapport des médianes sort d'un facteur 2.

**Pourquoi le temps compte autant que le message** : sans le hachage factice de research R-02, un
`401` en 2 ms contre un `401` en 90 ms publie la liste des comptes d'un établissement à qui sait
chronométrer. Le message identique n'y change rien.

---

## 4b · La révocation coupe immédiatement, et la rotation détecte les copies

```sh
cargo test --test session_revocation
pnpm --filter @kaya/app test file-jeton-expire
```

**Ce que ça prouve** :

- une session révoquée **cesse d'être acceptée à la requête suivante**, sans attendre les 60
  minutes du jeton d'accès — c'est la « coupure immédiate au départ d'un employé » du cadrage
  §12.2, et le seul recours contre un téléphone volé avant CPT-05 ;
- les **autres** sessions du même compte continuent ;
- un jeton de rafraîchissement présenté **deux fois** révoque **toute la famille**, pas seulement
  celui qui est présenté ;
- **et le versant qui ne se voit qu'à Abengourou** : une coupure de 90 minutes — une fois et
  demie la durée du jeton — ne perd **aucune** écriture de classe A. Les écritures entrent en file
  **sans jeton**, et le retour du réseau **rafraîchit avant de vider**. Le test échoue si l'ordre
  s'inverse, **y compris quand les deux réussissent** : en développement, la coupure dure trente
  secondes et le défaut ne se manifeste pas.

---

## 4c · La politique de mot de passe refuse ce qu'elle doit refuser

```sh
cargo test --test politique_mot_de_passe
```

**Ce que ça prouve** : un mot de passe de 7 caractères est refusé ; `12345678` est refusé **bien
qu'il fasse huit caractères** ; `chaise-tomate-abidjan` est **accepté** sans majuscule, sans
chiffre et sans symbole. La vérification s'exécute **sans aucun appel réseau** — la liste est
embarquée — et **ne s'applique pas à la connexion**, sous peine d'enfermer dehors un utilisateur
légitime.

---

## 5 · Le cumul de rôles donne l'union, et rien d'autre

```sh
cargo test --test roles_cumules
```

**Ce que ça prouve**, sur le compte d'Adjoua — gérante, caissière **et** réceptionniste :

- ses permissions effectives sont exactement l'union des trois ensembles, sans doublon ;
- retirer `caissier` lui laisse les permissions **partagées** avec ses deux autres rôles et ne lui
  retire que les exclusives ;
- un compte sans aucun rôle se connecte et obtient un ensemble **vide**, pas une erreur ;
- `admin_editeur` refuse un `etablissement_id`, les sept autres l'exigent.

---

## 6 · Aucune élévation de privilège hors ligne

```sh
cargo test --test classes_offline           # sept opérations de classe C du module
pnpm --filter @kaya/app test file-classe-a  # la file locale refuse les types de ce cycle
```

**Ce que ça prouve** (porte **P-13**) : aucune des sept opérations de classe C — création de
personne, création de compte, changement d'état, changement de mot de passe, attribution de rôle,
retrait de rôle, révocation de session — n'est atteignable depuis un chemin de code exécutable
hors ligne. Le test **déclare le nombre d'opérations inspectées** face au total attendu.

Côté application, `TYPES_CLASSE_A` ne reçoit **aucun** type de ce cycle, et le typage refuse la
mise en file.

---

## 7 · Le journal d'audit est immuable, et il se relit

```sh
cargo test --test audit_immuabilite
cargo test --test audit_classe_a
pnpm porte:p05b                              # contrôle statique, second volet
```

**Ce que ça prouve** :

- **versant négatif** — aucun chemin de code ne contient de `DELETE` ni d'`UPDATE` sur
  `journal_audit`, et le rôle applicatif n'a que `SELECT, INSERT` ;
- **versant positif** — une entrée s'écrit, se relit, et se filtre par les quatre critères. Sans
  ce second volet, supprimer la table suffirait à passer au vert (§ Couverture des portes,
  corollaire de l'exigence 4) ;
- **rejeu** — la même entrée soumise trois fois produit **un** enregistrement ;
- **désordre** — trois entrées appliquées dans les **six** ordres produisent le même état final,
  comparé comme **ensemble trié** et sur des identifiants **figés par permutation** ;
- **montants en `JSONB`** — une entrée portant `{"ecart_mineur": -12500, "devise": "XOF"}` est
  acceptée ; la même en `-12500.5` ou en `"12 500 F"` est **refusée**, à l'écriture comme au
  contrôle statique. C'est le registre qui trace les écarts de caisse : un montant en flottant, et
  l'audit ment sur ce qu'il est censé prouver (`pnpm porte:p10`).

---

## 8 · La taxonomie est complète, et les types dus sont nommés

```sh
cargo test --test audit_taxonomie
```

**Ce que ça prouve** : les **dix** familles de CPT-04 figurent à l'énumération et à
`docs/taxonomie-audit.md`, dans le même état des deux côtés. Le test échoue si un type dû acquiert
un chemin d'écriture sans changer d'état, si un type branché n'a pas de test, ou si le document et
le code divergent.

**Attendu au terme de ce cycle** : **2 branchés** — `changement_role`, `suppression` — et **8 dus**,
chacun portant la story qui le doit.

---

## 9 · Les dix types d'événements sont émis, sur les deux tenants

```sh
cargo test --test outbox_transactionnel
cargo test --test couverture_portes
```

**Ce que ça prouve** (portes **P-05** et **P-05b**) : chaque type émet dans la transaction de son
opération — rollback provoqué, ni ligne métier ni événement — et le **décompte des types testés
est comparé au total déclaré**, désormais **21** (11 + 10).

**Et sur les deux tenants de démonstration**, sans exception : c'est l'exigence 5 du
§ « Couverture des portes », née du défaut de séquence que la migration `0012` a corrigé et
qu'aucune relecture n'avait vu.

---

## 10 · Les quarante opérations sont isolées, et le contrat est à jour

```sh
cargo test --test isolation_tenant           # P-08, 40 opérations
pnpm porte:p01                               # client TS régénéré sans diff
pnpm generer:client
```

**Ce que ça prouve** : le tenant A ne lit ni n'écrit aucune ligne du tenant B, sur **chacune** des
quarante opérations, et le décompte des chemins couverts est comparé aux chemins servis.

**Ce qui change dans ce test, et qui est le vrai coût du cycle** : les requêtes n'envoient plus
`x-kaya-tenant`. Elles **obtiennent un jeton par le vrai chemin de connexion** (research R-04).
Un test qui forgerait le jeton avec la clé de signature passerait sans jamais exercer
l'authentification.

**Les deux opérations publiques** — `session_ouvrir`, `session_rafraichir` — sont la seule liste
d'exceptions du produit ; le test la connaît nommément et échoue si une opération s'y ajoute.

---

## 11 · Les quatre écrans, en clair et en sombre

```sh
pnpm --filter @kaya/app test                 # dont ecran-r0, ecran-r1, ecran-g3, ecran-g4
pnpm --filter @kaya/app test:i18n            # parité fr / en
pnpm --filter @kaya/app lint:tokens          # P-17
pnpm lint                                    # P-15, depuis la RACINE
KAYA_STYLEGUIDE=1 pnpm --filter @kaya/app dev
```

**Ce que ça prouve** :

| Écran | Ce qui est vérifié |
|---|---|
| **`R0` Connexion** | La ligne existe dans `docs/design/derivation.md` **avant** que l'écran ne soit codé. Les deux échecs affichent la **même** phrase. Refus immédiat et explicite hors ligne |
| **`R1` Accueil** | Quatre comptes → quatre jeux de tuiles. **Aucune action interdite dans le HTML rendu** — pas grisée : absente. Une tuile issue de trois rôles n'apparaît qu'une fois |
| **`G3` Utilisateurs et rôles** | Classe C : hors ligne, l'action disparaît et un bandeau dit pourquoi. Validation au champ, jamais au bandeau |
| **`G4` Journal d'audit** | Quatre filtres combinables. L'horodatage affiché est celui d'autorité |

**Le chargement paresseux se constate**, il ne se déclare pas : un compte à rôle unique ne doit
charger aucun morceau de module dont il n'a pas la permission.

---

## Le parcours de démonstration — ce que le cycle doit rendre visible

Une fois les treize vérifications vertes, la démonstration tient en six gestes :

1. **Adjoua se connecte** — un identifiant, un mot de passe, un écran.
2. **Son accueil porte l'union de ses trois métiers** — gérance, caisse, réception. Une seule fois
   chaque tuile.
3. **Yao se connecte sur un autre poste** — son accueil est plus court, et ce qui manque est
   *absent*, pas grisé.
4. **Adjoua retire un rôle à Yao** depuis `G3`. Le réseau coupé, l'action **disparaît** et
   l'interface dit pourquoi avant qu'on ait cliqué.
5. **M. Koffi ouvre le journal d'audit depuis son téléphone** et retrouve le retrait de rôle en
   trois filtres : qui, sur qui, quand.
6. **Adjoua révoque à distance** la session du téléphone de Yao. Au rafraîchissement suivant, ce
   téléphone revient à l'écran de connexion — et les autres sessions ne bougent pas.

C'est ce que M. Koffi achète. Le reste du cycle est ce qui rend ces six gestes vrais.
