# Phase 0 — Recherche et décisions techniques

**Cycle 004 — HEB** · Unités louables, formules de location et moteur de disponibilité
**Date** : 2026-08-02 · **Spec** : [spec.md](./spec.md)

Ce document tranche ce que la spécification laisse ouvert côté technique. Chaque décision porte
son motif et ce qui a été écarté. **Aucune version n'y est proposée** : `docs/versions-gelees.md`
(gel 1.0.12, vérifié le 2026-08-02) fait foi, et ses valeurs sont reprises telles quelles.

---

## R-01 — Le type de l'intervalle : `tstzrange`, et `PgRange<OffsetDateTime>` côté Rust

**Décision.** La période d'une occupation est une colonne `TSTZRANGE` unique, jamais deux colonnes
`debut` / `fin`. Côté Rust, elle se lit et s'écrit en `sqlx::postgres::types::PgRange<time::OffsetDateTime>`.

**Motif.** Vérifié dans les sources de `sqlx-postgres` 0.9.0 déjà présentes au poste :
`impl Type<Postgres> for PgRange<time::OffsetDateTime>` rend `PgTypeInfo::TSTZ_RANGE`
(`src/types/range.rs:213`). Le workspace active déjà les features `time` et `uuid` sur sqlx
(`backend/Cargo.toml`), et `time` 0.3.54 est l'écart au gel introduit par le cycle 001 pour
`OffsetDateTime`, déjà nommé par le contrat HTTP. Aucune dépendance nouvelle.

Une colonne `TSTZRANGE` **unique** est ce qui rend la contrainte d'exclusion possible : `EXCLUDE`
opère sur une expression indexable par GiST, et `&&` (chevauchement) n'existe que sur un type
intervalle. Deux colonnes séparées imposeraient de réécrire l'expression `tstzrange(debut, fin, '[)')`
dans la contrainte — faisable, mais alors rien n'empêcherait une requête de lire `debut`/`fin`
directement et de reconstruire un intervalle avec des bornes différentes.

**Écarté** — `daterange` : ferme la porte au passage horaire et à la demi-journée, c'est
exactement ce que le principe IV interdit. `TIMESTAMP` sans fuseau : l'établissement porte son
fuseau (ETB-01), et une heure murale sans fuseau devient ambiguë dès le premier établissement
hors `Africa/Abidjan`. Deux colonnes `timestamptz` + contrainte sur expression : voir ci-dessus.

---

## R-02 — La contrainte d'exclusion : le mécanisme est déjà éprouvé, l'extension déjà installée

**Décision.** `CONSTRAINT occupation_sans_chevauchement EXCLUDE USING gist (unite_id WITH =, periode WITH &&)`,
posée **à la création de la table**.

**Motif.** Le cycle 001 a fait ce travail par anticipation, et le module doré le consigne
(§ « Le spike `EXCLUDE USING gist` — retour, avant HEB-02 ») :

| Point | État constaté au cycle 001 |
|---|---|
| Extension `btree_gist` | Installée par `0001_roles_et_schemas.sql:93`, et **« trusted »** — `kaya_owner`, propriétaire non superutilisateur, l'installe sans intervention d'un superutilisateur |
| `EXCLUDE USING gist (uuid WITH =, range WITH &&)` | Accepté et effectif sur `fiscalite.exercice_comptable` (`daterange`) |
| Mapping sqlx 0.9 | Validé sur `daterange` ; `PgRange<T>` présent en 0.9.0 |
| Ordre de pose | **À la création.** Une contrainte d'exclusion ajoutée sur une table déjà peuplée échoue sur les données existantes |

`unite_id WITH =` exige `btree_gist` : GiST ne sait pas indexer l'égalité sur un UUID sans
l'extension. Elle est là depuis le premier cycle, précisément pour ce moment.

**Ce qui reste neuf pour ce cycle** : `tstzrange` plutôt que `daterange`, et surtout la
**concurrence réelle** — `exercice_comptable` n'a jamais reçu deux écritures simultanées.

---

## R-03 — Détecter la violation d'exclusion en sqlx 0.9 : l'apport de `#3918` est **partiel**

**Décision.** La détection se fait par `matches!(erreur_base.kind(), ErrorKind::ExclusionViolation)`,
puis par `erreur_base.constraint()` pour distinguer *quelle* contrainte a sauté. Un helper
`fn est_violation_exclusion(&sqlx::Error) -> bool` est écrit **une fois** dans le crate
`hebergement` et employé partout.

**Motif — et correction d'une attente du gel.** `docs/versions-gelees.md` §2 retient sqlx 0.9.0
notamment pour « `#3918` — type d'erreur dédié à la violation de contrainte d'exclusion », et le
module doré inscrit cet apport comme **restant à vérifier avant HEB-02**. Vérification faite dans
les sources présentes au poste :

- `sqlx_core::error::ErrorKind::ExclusionViolation` **existe** (`sqlx-core-0.9.0/src/error.rs:206`),
  et `sqlx-postgres` mappe bien le SQLSTATE `23P01` dessus (`src/error.rs:221`, constante
  `EXCLUSION_VIOLATION = "23P01"`). L'apport est réel : plus besoin de comparer une chaîne
  SQLSTATE à la main.
- **Mais le trait `DatabaseError` n'expose PAS de `is_exclusion_violation()`.** Il porte
  `is_unique_violation()`, `is_foreign_key_violation()` et `is_check_violation()` — les trois
  autres — et s'arrête là (`sqlx-core-0.9.0/src/error.rs:260-273`). L'écrire par symétrie ne
  compilerait pas.
- `ErrorKind` est `#[non_exhaustive]` : le `matches!` est la forme correcte, un `match` exhaustif
  ne compilerait pas non plus.

**Conséquence pour le plan** : une tâche dédiée écrit ce helper et son test, plutôt que de laisser
chaque appelant redécouvrir l'absence d'accesseur. **À reporter au module doré** : l'apport de
`#3918` est vérifié, avec sa limite.

**Écarté** — lire `code() == Some("23P01")` : fonctionne, mais réintroduit exactement la chaîne
magique que la montée en 0.9 devait supprimer.

---

## R-04 — Le temps de remise en état est écrit dans l'intervalle, pas appliqué à la lecture

**Décision.** La table porte **une seule** colonne d'intervalle, `periode`, qui inclut déjà le
temps de remise en état. Les bornes commerciales — celles que le client connaît — sont portées à
côté, en `debut_client` et `fin_client`, à titre d'information pour l'affichage et la facturation.
La contrainte d'exclusion porte sur `periode`, jamais sur les bornes commerciales.

**Motif.** Le principe IV l'impose (« le temps de remise en état est intégré à l'intervalle
d'indisponibilité, pas géré à part ») et le registre des classes le confirme (§7.2, ligne
« Intervalle de remise en état — B3 — intégré à l'intervalle d'indisponibilité »).

La raison profonde tient en une phrase : **une règle appliquée à la lecture ne protège rien.**
Si la remise en état était un délai ajouté par le code au moment de vérifier la disponibilité,
deux transactions concurrentes le calculeraient toutes deux avant d'écrire, et toutes deux
passeraient — la contrainte de base, elle, ne verrait aucun chevauchement. On aurait rétabli
exactement le verrou applicatif que le principe IV refuse, en croyant l'avoir évité.

Les deux bornes commerciales sont nécessaires parce que « la chambre est libre à 14 h » et « le
client part à 12 h » sont deux faits distincts, et que la note se calcule sur le second.

**Écarté** — une seconde ligne d'occupation de type « ménage » : doublerait le nombre de lignes,
imposerait une saga pour les garder cohérentes, et rendrait la libération ambiguë (laquelle
libère-t-on ?).

---

## R-05 — Prouver que c'est la contrainte, pas un verrou : deux transactions vraiment simultanées

**Décision.** Le test de concurrence ouvre **deux transactions PostgreSQL distinctes**, insère
dans chacune sans commiter, puis commite l'une et l'autre. Il asserte trois choses :

1. exactement une transaction réussit ;
2. l'échec de l'autre est un `ErrorKind::ExclusionViolation` — **pas** une erreur applicative,
   pas un timeout, pas une erreur de sérialisation ;
3. le nom de contrainte rendu par `constraint()` est bien `occupation_sans_chevauchement`.

**Motif.** Le point 2 est ce qui distingue une garantie d'une coïncidence. Un test qui se
contenterait de « une seule a réussi » passerait au vert sur une implémentation à verrou
applicatif, à `SELECT ... FOR UPDATE`, ou à `SERIALIZABLE` — trois mécanismes qui rendraient la
double attribution *improbable* au lieu d'*impossible*, et qui se dégraderaient silencieusement
sous charge ou en cas de réplication.

`futures` 0.3.33 est déjà au dépôt (écart au gel du cycle 001, motif inscrit : « tests de
concurrence »). Aucune dépendance nouvelle.

**Écarté** — lancer N tâches en parallèle et compter : non déterministe, et ne dit rien de la
*cause* du refus. Le test l'ajoute en complément (SC-001 parle de mille tentatives), mais la
preuve tient au test à deux transactions.

---

## R-06 — L'intervalle vide est le seul trou de la contrainte, et il se ferme par `CHECK`

**Décision.** `CONSTRAINT occupation_periode_non_vide CHECK (NOT isempty(periode))`, plus
`CHECK (lower_inc(periode) AND NOT upper_inc(periode))` pour verrouiller la forme `[)`.

**Motif.** `&&` rend faux dès qu'un des deux intervalles est vide : PostgreSQL considère qu'un
intervalle vide ne chevauche rien. Une occupation `[14h, 14h)` passerait donc la contrainte
d'exclusion **et** n'empêcherait aucune autre occupation — une ligne fantôme qui occupe une unité
sans la bloquer. C'est le seul contournement possible de la garantie, et il ne se voit pas.

Le second `CHECK` empêche qu'une écriture pose `[14h, 16h]` (borne haute incluse) : deux
occupations contiguës deviendraient alors chevauchantes, et le comportement du produit
changerait selon la forme de bornes employée par l'appelant.

**Écarté** — valider en Rust seulement : la règle vaut aussi pour les seeds et pour toute
correction manuelle en base.

---

## R-07 — Schéma `hebergement`, et le nom que la porte P-09 attend déjà

**Décision.** `CREATE SCHEMA hebergement`, avec les six tables du cycle. La table d'occupation
s'appelle **`hebergement.occupation`**, sans autre nom possible.

**Motif.** `backend/tests/portes_a_vide.rs` interroge littéralement
`table_existe(&pool, "hebergement", "occupation")` et **fait échouer le build** dès que cette
table apparaît sans que P-09 soit levée. Le nom n'est pas un choix de ce cycle : il a été fixé au
cycle 001 par l'assertion de non-régression.

Le schéma suit le patron du cycle 003 : une migration dédiée à la création du schéma
(`0014_schema_comptes.sql` l'a fait pour `comptes`), afin que P-04 — qui compare les schémas
déclarés aux schémas réels — ne trouve pas un `CREATE SCHEMA` glissé dans une migration ancienne.

---

## R-08 — RLS et privilèges : `ENABLE` + `FORCE`, et les privilèges disent la classe

**Décision.** Les six tables portent le patron RLS identique du module doré. Les privilèges
accordés à `kaya_app` diffèrent selon la classe :

| Table | Classe | Privilèges `kaya_app` | Pourquoi |
|---|---|---|---|
| `categorie` | C | `SELECT, INSERT, UPDATE, DELETE` | Référentiel éditable |
| `unite` | C | `SELECT, INSERT, UPDATE, DELETE` | Référentiel éditable |
| `formule` | C | `SELECT, INSERT, UPDATE, DELETE` | Référentiel éditable |
| `bareme_palier` | C | `SELECT, INSERT, UPDATE, DELETE` | Référentiel tarifaire éditable |
| `plage_demi_journee` | C | `SELECT, INSERT, UPDATE, DELETE` | Référentiel éditable |
| `occupation` | B | `SELECT, INSERT, UPDATE` — **jamais `DELETE`** | Une occupation ne se supprime pas : elle se libère, ce qui est un `UPDATE` de sa fin. Supprimer effacerait la trace d'une chambre occupée |
| `prestation_incluse` | — (provision) | **aucun** | Provision HEB-09 : la table existe, rien ne l'écrit ni ne la lit |

**Motif.** Le module doré, § « Les privilèges disent la classe hors-ligne » : un privilège accordé
en trop rend faux un classement sans que rien ne le signale. Le refus de `DELETE` sur `occupation`
est le point le moins évident et le plus important : une occupation annulée reste une occupation
annulée.

**Point de vigilance repris du module doré** : aucune migration n'écrit de données sur une table
en `FORCE ROW LEVEL SECURITY` — les `INSERT` réussissent en n'écrivant rien, sans erreur. Les
seeds Deloria passent donc par la mécanique de seeds (qui pose le tenant courant), jamais par la
migration. Seules les permissions du référentiel **global** `comptes.permission` s'insèrent en
migration, dans l'ordre `CREATE` → `INSERT` → `ENABLE`/`FORCE` → `CREATE POLICY`, ou via la
politique `administration_editeur ... TO kaya_owner` déjà posée en `0008`.

---

## R-09 — `ressource_reservable` n'existe pas, et ce cycle ne la crée pas

**Décision.** `hebergement.unite` est une table autonome. **Aucune table
`socle/…/ressource_reservable` n'est créée.**

**Motif.** Le cadrage §4.1 et la constitution (principe II) écrivent que le socle « connaît
`article_vendable` et `ressource_reservable` ». Vérification faite : **ces deux noms n'apparaissent
dans aucune migration ni aucun crate du dépôt** — uniquement dans `docs/cadrage-v1.md:122`. Les
trois cycles livrés ont construit le socle sans elles, et `etablissements.table_pdv` — qui est
conceptuellement une ressource réservable — a été livrée au cycle 002 sans passer par cette
abstraction.

L'énoncé de la constitution est une **frontière de vocabulaire**, pas un inventaire de tables : il
dit ce que le socle ne doit pas connaître (chambre, unité louable, séjour), et par quels mots il
nommerait la chose s'il avait à la nommer. Créer une table parente maintenant serait une
abstraction spéculative à **un seul implémenteur**, que le principe X interdit explicitement
(« prêt ≠ construit »).

**Ce que ce cycle doit garantir en revanche, et qui est vérifiable** : aucun crate de `socle/` ne
gagne la moindre notion d'unité, de chambre ou de formule. C'est la porte **P-03**, et le test
`backend/tests/architecture.rs` la tient.

**À porter à la revue** : le jour où un second consommateur apparaîtra — RSV (planning de
réservation, T4) réserve aussi des ressources —, la question de l'abstraction partagée se reposera
avec, cette fois, deux implémenteurs. La consigner ici évite qu'elle soit tranchée par défaut.

---

## R-10 — Le statut d'occupation est dérivé : aucune colonne, une vue de lecture

**Décision.** `hebergement.unite` porte `statut_menage` et **ne porte aucune colonne de statut
d'occupation**. Le statut « libre / occupée / réservée » se calcule à la lecture, par une requête
qui interroge `occupation` sur l'instant demandé.

**Motif.** Le cadrage §11.4 désigne la confusion des deux comme la cause des doubles attributions,
et le registre §7.2 classe le premier « dérivé » et le second en A. Une colonne matérialisée
devrait être tenue à jour par déclencheur ou par le code : dans les deux cas, elle peut diverger
de la vérité, et c'est elle que l'écran afficherait.

**Conséquence assumée** : la lecture du statut coûte une requête sur `occupation`. L'index GiST de
la contrainte d'exclusion la sert déjà — une recherche de chevauchement sur `(unite_id, periode)`
est exactement ce qu'il indexe. Aucun index supplémentaire n'est nécessaire au MVP.

`statut_menage` reçoit une valeur par défaut et **aucun endpoint de modification** : sa gestion
est HEB-06, hors périmètre. La colonne existe parce que HEB-01 la nomme dans la définition de
`unite` ; la logique n'existe pas parce qu'aucune story du cycle ne l'appelle.

---

## R-11 — Le barème : arithmétique entière, paliers ordonnés, heure entamée due

**Décision.** Le calcul vit dans le crate `hebergement`, en fonction pure, sur des entiers d'unité
mineure. Algorithme :

1. durée réelle = `fin_autorite − debut_autorite`, en secondes ;
2. si durée ≥ seuil de bascule en nuitée → la formule appliquée devient `NUITEE`, fin du calcul ;
3. chercher le **premier** palier dont la durée ≥ durée réelle → son prix est dû ;
4. si aucun palier ne convient (durée > dernier palier) → prix du dernier palier
   + `ceil((durée − durée du dernier palier) / 1 h) × prix_heure_supplementaire`.

**Motif.** Le point 4 encode « toute heure entamée est due » (FR-028) par un plafond entier, sans
flottant. Le point 3 encode « le premier palier est dû en entier » même pour vingt minutes
(FR-028, cas limite de la spec). Le point 2 précède le 3 parce que la bascule en nuitée n'est pas
un palier majoré : c'est un changement de formule.

**Aucun flottant nulle part.** Les prix sont des `i64` d'unité mineure (principe V, porte P-10) ;
les durées sont des entiers de secondes. `rust_decimal` n'intervient pas — ce sont des montants,
pas des quantités.

**L'ordre des paliers est garanti en base**, pas au chargement : `UNIQUE (formule_id, duree)` +
tri `ORDER BY duree` à la lecture. Un barème aux paliers désordonnés est donc impossible à
constituer, et FR-025 (« refusé ou normalisé ») est tenu par la contrainte plutôt que par une
validation applicative.

**Écarté** — stocker un prix horaire et multiplier : le barème est dégressif et non linéaire,
c'est tout son objet.

---

## R-12 — La rebascule de palier : un calcul, pas une écriture de note

**Décision.** Le moteur rend une **décision de tarification** — palier retenu, montant dû,
formule appliquée, indication de rebascule — et **n'écrit aucune ligne de note**. L'écriture au
registre des actions (CPT-04) a lieu quand une rebascule est constatée, via le trait d'audit déjà
exposé par `socle/comptes`.

**Motif.** La note de séjour est SEJ-03, tranche T2 : elle n'existe pas. Un moteur qui supposerait
son existence ne serait pas testable à ce cycle, et FR-026 (« la différence est portée au débit du
séjour ») deviendrait invérifiable. La spec l'a anticipé : « il calcule, il ne facture pas ».

**Ce qui est livré et testable ici** : la décision de tarification, et sa trace au registre. Ce qui
viendra avec SEJ-03 : la ligne de note qui consomme cette décision.

---

## R-13 — Les plages de demi-journée : heure murale en base, instant à l'écriture

**Décision.** `plage_demi_journee` stocke deux colonnes `TIME` (heure locale) et non des instants.
La conversion en instant se fait à l'attribution, avec le fuseau de l'établissement lu par le
trait `EstablishmentDirectory`.

**Motif.** « 8 h – 12 h » est une règle d'exploitation, pas un fait daté : elle vaut tous les jours,
y compris ceux qui n'existent pas encore. La stocker en instant imposerait une ligne par jour.

La conversion est le seul endroit du cycle où un fuseau intervient, et elle est **côté serveur** :
le terminal n'envoie jamais une heure murale interprétée localement. FR-034 (non-fractionnable) se
vérifie après conversion, en comparant l'intervalle demandé aux instants calculés — pas en
comparant des heures murales, ce qui échouerait au passage de minuit.

**Dépendance** : le fuseau vient de l'établissement (ETB-01) par trait, jamais par jointure
inter-schémas (P-04).

---

## R-14 — La règle de conversion fiscale : le paramètre est **éditable**, et l'incohérence impossible

> **Révisé le 2026-08-02 sur constat terrain.** La première rédaction posait
> `regle_conversion_taxe` à `NULL` sur le passage et la demi-journée, au motif que B-02 n'était pas
> tranchée. Deux faits d'exploitation l'ont corrigée, et ils simplifient le cycle plutôt que de le
> compliquer.

**Décision.**

```sql
assujettie_taxe_nuitee  BOOLEAN NOT NULL,
regle_conversion_taxe   TEXT NULL
    CHECK (regle_conversion_taxe IN
           ('aucune','une_nuitee_par_occupation','au_prorata','seuil_horaire')),

-- Une formule assujettie SANS règle de conversion est une incohérence, pas un état d'attente.
CONSTRAINT formule_regle_fiscale_coherente
    CHECK (NOT assujettie_taxe_nuitee OR regle_conversion_taxe IS NOT NULL)
```

Seeds Deloria :

| Formule | `assujettie` | `regle_conversion_taxe` | Ce que cela produit |
|---|---|---|---|
| `NUITEE` | `true` | `une_nuitee_par_occupation` | **500 F pour un séjour de 3 nuits**, pas 3 × 500 |
| `PASSAGE` | **`false`** | `aucune` | Pas de taxe de séjour |
| `DEMI_JOURNEE` | **`false`** | `aucune` | Pas de taxe de séjour |
| `MENSUEL` | `false` | `aucune` | Pas de taxe de séjour |

**Motif — deux constats d'exploitation, et un mécanisme qui existait déjà.**

**1. Le passage et la demi-journée ne sont pas assujettis en pratique.** Ce n'est plus une question
ouverte tenue en suspens : c'est un fait observé sur le pilote. `assujettie_taxe_nuitee = false`
n'est donc pas une décision fiscale prise par défaut, c'est le **reflet de la pratique constatée**.
Ce que B-02 tranchera, c'est **la valeur par défaut légale** — jamais l'existence du paramètre.
Le cadrage §9.6 est explicite : « hors Abidjan **variable selon la collectivité** ». Les règles
varient par collectivité, donc le paramètre doit exister quoi qu'il arrive. **Ce n'est pas une
incertitude en attente d'arbitrage, c'est une exigence produit** : aucun code, aucun test et aucun
commentaire ne doit traiter `regle_conversion_taxe` comme une constante provisoire.

**2. La taxe se plafonne par occupation, et le mécanisme était déjà dans l'énumération.** Le
constat terrain — « si la taxe est de 500 F, une personne qui réserve 3 nuits ne paie que 500 F »
— est exactement ce que `une_nuitee_par_occupation` nomme, et cette valeur figure au cadrage §5.5
depuis l'origine. Aucun champ nouveau n'est nécessaire : `au_prorata` couvre le cas inverse
(500 F × 3), qui est la lecture stricte de « par nuitée et par client ».

**3. Ce qui manquait vraiment, c'est l'édition.** La flexibilité demandée — « ce genre
d'exonération n'est pas possible partout » — ne réclame pas un mécanisme plus riche, mais que la
règle **soit modifiable par l'exploitant, formule par formule**. Le plan initial ne l'exposait pas.
Elle l'est désormais : l'opération 8 du contrat (`hebergement_modifier_formule`) porte les deux
champs, gardée par `heb.offre.gerer`.

**Deux conséquences qui allègent le cycle :**

- **Plus de troisième état d'écran.** `NULL` sur une formule assujettie aurait imposé d'afficher
  « paramétrage fiscal en attente » — une mention **absente de la maquette `G2`** et du lexique.
  La contrainte de cohérence ci-dessus rend cet état impossible à constituer, et les deux mentions
  maquettées suffisent : « Taxe de séjour comprise dans le prix » / « Pas de taxe de séjour sur
  cette formule ».
- **Plus de type de refus `RegleFiscaleNonParametree`.** Il n'a plus de cas à couvrir : la base
  garantit qu'une formule assujettie porte sa règle.

**`NULL` reste permis sur une formule non assujettie** — « pas de taxe » et « pas de règle » disent
la même chose, et forcer `'aucune'` ajouterait une valeur sans information.

**Conformité au principe V et à la porte P-12** : ce cycle ne calcule toujours aucune taxe. Il
porte le **paramètre** et le rend éditable ; la règle qui le consommera vivra dans
`JurisdictionAdapter` (T3, FIS-03).

**⛔ L'axe « par client » n'est PAS tranché, et ne doit pas l'être par défaut.**
`une_nuitee_par_occupation` réduit trois nuits à une. **Que fait-elle de trois personnes ?** Le
cadrage §9.6 et FIS-03 disent tous deux « par nuitée **et par client** (accompagnants inclus) », et
SEJ-02 précise que l'enregistrement des accompagnants « impacte le calcul de la taxe ». Une
occupation de 3 nuits à 2 personnes vaut donc 500 F ou 1 000 F — **et aucune source du dépôt ne le
dit**. Le constat terrain porte sur une personne seule : il tranche l'axe des nuits, pas celui des
personnes.

La question relève du **calcul**, donc de FIS-03 (T3) ; ce cycle ne calcule rien et n'est pas
bloqué. Elle est consignée ici pour que le moteur fiscal porte, le jour où il s'écrira, la marque
explicite « axe des personnes non résolu » — **un multiplicateur posé à l'aveugle se retrouverait
sur des factures et dans un état de reversement communal**.

---

## R-15 — Les permissions : les premières rattachées à un module d'activité

**Décision.** Cinq permissions, `module_code = 'HEBERGEMENT'` :

| Code | Garde |
|---|---|
| `heb.offre.lire` | Lecture des catégories, unités, formules, barèmes |
| `heb.offre.gerer` | Création et modification du référentiel et des tarifs |
| `heb.unite.attribuer` | Création d'une occupation |
| `heb.unite.liberer` | Libération d'une occupation |
| `heb.disponibilite.consulter` | Interrogation de disponibilité |

**Motif.** La migration `0016_roles_permissions.sql` du cycle 003 annonce nommément
`heb.unite.attribuer` et écrit que `module_code` « restera `NULL` jusqu'au cycle HEB ». C'est donc
le premier cycle qui éprouve le filtrage par module — jusqu'ici jamais exercé sur une valeur non
nulle.

**Contrainte reprise du cycle 003** : le référentiel `comptes.permission` porte `module_code`
**sans clé étrangère** vers `etablissements.module_activite`, ce serait une clé inter-schémas
interdite par P-04. La cohérence est tenue par un test qui lit le référentiel des modules **à
travers le trait `RegistreModules`** et échoue si une permission nomme un module inconnu. Ce test
existe ; il gagne cinq cibles.

**Chaque permission garde une action réellement servie** — c'est la règle du cycle 003 qui fait
échouer le build sur une permission sans contrepartie. Les cinq sont couvertes par des endpoints
de ce cycle. Aucune permission n'est posée pour HEB-06 ou HEB-07.

**Attribution aux rôles** : `proprietaire` et `gerant` reçoivent tout ; `receptionniste` reçoit
`heb.offre.lire`, `heb.disponibilite.consulter`, `heb.unite.attribuer` et `heb.unite.liberer` —
Yao attribue des chambres, il ne fixe pas les tarifs.

---

## R-16 — Les paramètres vont au catalogue de configuration héritée

**Décision.** Quatre clés entrent à `etablissements.parametre_catalogue`, portée la plus basse
`ETABLISSEMENT` :

| Clé | Type | Story |
|---|---|---|
| `heure_arrivee_standard` | `TEXTE` (heure locale) | HEB-03 |
| `heure_depart_standard` | `TEXTE` (heure locale) | HEB-03 |
| `seuil_bascule_nuitee_minutes` | `ENTIER` | HEB-04 |

Le **temps de remise en état** ne va **pas** au catalogue : il est porté par la catégorie,
par formule, comme HEB-01 le définit (`categorie {…, temps_remise_en_etat_par_formule}`). Les
**plages de demi-journée** non plus : elles sont une table du référentiel (§7.1 du registre,
« Plages de demi-journée — C »).

**Motif.** Le catalogue sert les paramètres **scalaires à héritage** ; un temps de remise en état
qui varie par catégorie *et* par formule n'est pas un scalaire d'établissement, et le loger au
catalogue imposerait une clé composée par couple. Le récapitulatif des paramètres de
`docs/user-stories-v1.md` liste bien les trois temps de remise en état — comme **valeurs par
défaut Deloria**, ce que les seeds honorent en les posant sur les catégories.

**Conformité au principe I·c** : le récapitulatif des paramètres est mis à jour **dans le même
changement** que l'implémentation, avec les trois clés nouvelles.

---

## R-17 — Les événements outbox du cycle

**Décision.** Cinq types d'événements, tous émis dans la transaction de leur écriture :

| Type | Émis quand |
|---|---|
| `heb.occupation.attribuee` | Une occupation est créée |
| `heb.occupation.liberee` | Une occupation est libérée |
| `heb.formule.creee` | Une formule est créée |
| `heb.formule.modifiee` | Une formule ou son barème change |
| `heb.categorie.tarif_modifie` | Un prix change — l'événement que le propriétaire lira |

**Motif.** Le principe II : « toute transition d'état écrit un événement outbox dans la même
transaction », garanti par la signature `OutboxWriter::ecrire(&self, tx, evenement)` qui **prend
la transaction et n'en ouvre jamais**. La porte P-05 compte les types déclarés au modèle de
données face aux types testés — le décompte passe de 22 à 27, et `backend/tests/couverture_portes.rs`
en porte la vérification.

**Le référentiel émet aussi**, contrairement à ce qu'on pourrait supposer d'entités de classe C :
un changement de tarif est une transition d'état métier que le grand livre doit porter — c'est ce
qui rendra la reconstitution des prix historiques possible sans versionner les tables.

**Pas d'événement sur rejeu** — point 5 de l'ordre des opérations du module doré : « émettre
l'événement uniquement si la ligne vient d'être créée ». Sinon le grand livre devient le journal
des tentatives réseau.

---

## R-18 — Les écrans : `G2` maquetté, `G5` composé

> **Révisé le 2026-08-02.** La première rédaction concluait « aucun écran de catégories ni
> d'unités », au motif qu'un écran doit être maquetté ou dérivé. **Cette règle à deux cas était
> incomplète** — voir R-20.

**Décision.** Deux écrans, deux routes :

| Écran | Route | Cas | Référence |
|---|---|---|---|
| `G2` — l'offre d'hébergement | `/hebergement` | **(a) maquetté** | `G2-offre-hebergement.html` et son état `-residence` |
| `G5` — chambres et catégories | `/chambres` | **(c) composé** | Les seize composants canoniques, inscrit à `derivation.md` |

**Obligations rappelées, toutes déjà éprouvées aux cycles précédents** :

- **une seule racine, et c'est un élément** — jamais un `v-if`/`v-else` de premier niveau, sous
  peine de `Cannot read properties of null (reading 'parentNode')` à la navigation suivante ;
- la **huitième couche** du module doré (cycle de vie de l'application) dit où va le thème et ce
  que rend le layout — à relire avant de créer la page ;
- la **septième couche** donne le patron d'écriture : squelette, refus métier en langue
  utilisateur, validation au champ, action **absente** sans permission, refus immédiat hors ligne,
  rafraîchissement sans rechargement ;
- tout montant par `app/core/format/montant.ts`, jamais `Intl.NumberFormat` ni un `money()` recopié ;
- tout champ par `app/core/design-system/ChampSaisie.vue`.

**Le refus hors ligne est ici de classe C ET B** — tout le cycle est indisponible hors connexion.
L'écran l'annonce immédiatement, sans grisé silencieux et sans file d'attente (P-13).

---

## R-20 — Le troisième cas de la doctrine d'écran : l'écran composé

**Décision.** `G5` — chambres et catégories — se code **sans maquette et sans motif hérité**, au
titre du troisième cas de `docs/Kaya_Design.md` §2.

**Motif.** Le tableau « on maquette si… / **on code directement si…** » existe depuis l'origine et
énumère quatre conditions. `G5` les remplit toutes :

| Condition | Vérification |
|---|---|
| Motif déjà posé — liste, formulaire ou fiche | Une liste et deux formulaires |
| Conception entièrement issue de la bibliothèque | Vérifiée composant par composant, ci-dessous |
| Consulté rarement, par un utilisateur formé | Adjoua règle son parc à l'ouverture, puis y revient à la marge |
| Aucun doute sur son aspect | Une liste de chambres et un formulaire |

**Zone de charme**, au sens de la règle qui tranche (`Kaya_Design.md` §1) : *« si l'utilisateur est
debout, pressé, avec un client en face de lui ou de l'argent en jeu — zone de vitesse. Sinon —
zone de charme. »* **Un écran de comptoir se maquette toujours.**

**Couverture par les seize composants — aucun motif ne manque :**

| Besoin | Composant | Note |
|---|---|---|
| Liste des catégories et unités | **08 · Ligne de liste** | Son rôle nomme littéralement « chambres » |
| Formulaires | **16 · Champ de saisie** | Seul composant d'écriture du produit |
| Choix de la catégorie | **16**, état « choix fermé (`<select>`) » | ⚠️ **Pas le 12** : sa règle dit « au-delà de quatre options c'est une liste, pas un segment », et Deloria a **six** catégories. Un segmenté à six options ne tient pas sur 372 px |
| Actions · vide · chargement | **01–03 · 11 · 13** | |

**Contrepartie obligatoire** : inscription à `docs/design/derivation.md` avec les mentions
**« composé »** et **« à valider à l'atelier terrain »**. Sans cette ligne, la porte P-19 refuse
l'écran.

**Si un motif manquait**, l'écran ne se coderait pas : un composant nouveau se maquette, il ne
s'improvise pas au détour d'un écran.

---

## R-19 — Rien de nouveau au gel, et deux points à reporter à la revue

**Décision.** **Aucune dépendance nouvelle.** Tout ce dont ce cycle a besoin est déjà épinglé :
`sqlx` 0.9.0 (features `uuid`, `time`, `rust_decimal` actives), `time` 0.3.54, `uuid` 1.24.0,
`futures` 0.3.33, `utoipa` 5.5.0.

**Deux constats à porter à la revue mensuelle du 2026-08-31, sans rien changer maintenant** :

1. **`docs/versions-gelees.md` §2 peut être précisé.** Il présente `#3918` comme « type d'erreur
   dédié à la violation de contrainte d'exclusion » ; c'est exact pour `ErrorKind`, mais le trait
   `DatabaseError` n'a pas reçu l'accesseur symétrique. Le gel ne devient pas faux — sqlx 0.9.0
   reste le bon choix, et pour la bonne raison —, la note gagnerait sa limite.
2. **Le point ouvert du gel se referme ici.** L'en-tête de `docs/versions-gelees.md` écrit :
   « Un seul point reste ouvert : le choix de sqlx 0.9.0 doit être confirmé par le spike
   GiST/`tstzrange` de la phase 0 ». Ce cycle **est** cette confirmation — sur `tstzrange`, en
   concurrence réelle, avec le type d'erreur exercé. La ligne pourra être retirée à la revue.

**Aucune version n'est proposée ici**, conformément à la consigne : le gel est fait, daté et
sourcé.

---

## Récapitulatif des décisions

| # | Décision | Impact si manquée |
|---|---|---|
| R-01 | `tstzrange` + `PgRange<OffsetDateTime>` | Le passage horaire devient impossible — irréversible |
| R-02 | Contrainte d'exclusion posée **à la création** | Impossible à ajouter ensuite sur une table peuplée |
| R-03 | `matches!(kind(), ExclusionViolation)`, pas d'accesseur | Ne compile pas si on suppose la symétrie |
| R-04 | Remise en état **dans** l'intervalle écrit | Rétablit un verrou applicatif sans le savoir |
| R-05 | Test à deux transactions, cause du refus assertée | Un verrou applicatif passerait au vert |
| R-06 | `CHECK` sur intervalle vide et forme `[)` | Ligne fantôme qui n'occupe ni ne bloque |
| R-07 | Schéma `hebergement`, table `occupation` | Le nom est fixé par l'assertion P-09 du cycle 001 |
| R-08 | Pas de `DELETE` sur `occupation` | Une chambre occupée pourrait être effacée |
| R-09 | Pas de `ressource_reservable` | Abstraction spéculative à un seul implémenteur |
| R-10 | Statut d'occupation dérivé, aucune colonne | Doubles attributions (cadrage §11.4) |
| R-11 | Arithmétique entière, paliers ordonnés en base | Centimes perdus, barème incohérent |
| R-12 | Le moteur calcule, il ne facture pas | Dépendance à SEJ-03, non testable ici |
| R-13 | Plages en heure murale, converties au serveur | Une ligne par jour, ou une dérive de fuseau |
| R-14 | Paramètre fiscal **éditable**, incohérence impossible par `CHECK` | Une décision fiscale prise par défaut, et un état d'écran non maquetté |
| R-15 | Cinq permissions rattachées au module | Dette du cycle 003 non honorée |
| R-16 | Trois clés au catalogue, remise en état sur la catégorie | Paramètre en dur (principe I·c) |
| R-17 | Cinq types d'événements outbox | Grand livre incomplet (P-05) |
| R-18 | Écran des formules seulement | Écran hors matrice de dérivation |
| R-19 | Aucune dépendance nouvelle | — |
