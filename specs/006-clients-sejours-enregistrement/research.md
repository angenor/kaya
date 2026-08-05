# Phase 0 — Recherche et décisions

**Cycle 006** — Fiches clients, arrivée, départ et prolongation · 2026-08-03
**Plan** : [plan.md](./plan.md) · **Spécification** : [spec.md](./spec.md)

Dix-sept décisions. Aucune n'introduit de dépendance, d'extension PostgreSQL ni d'outil : le gel
**1.0.12** est repris tel quel et la revue mensuelle du 2026-08-31 n'a rien à trancher du fait de
ce cycle.

---

## R-01 — La fiche client est `comptes.personne` qualifiée par `comptes.client`

**Décision.** L'identité civile du client vit dans `comptes.personne`, table existante. Une table
`comptes.client` la **qualifie** comme cliente et porte les deux attributs que CPT n'a aucune raison
de connaître : `date_naissance`, `nationalite`.

**Rationale.** Trois raisons, dans l'ordre de force :

1. **La migration `0015` le dit.** `type_piece` et `numero_piece` y sont « POSÉES, NON ALIMENTÉES —
   alimentation **SEJ-01**, rétention 90 jours TRX-06 ». La décision a été prise au cycle 003 ; ce
   cycle l'exécute.
2. **Une pièce d'identité, un seul endroit.** `provisions_sans_logique.rs` échoue déjà si une
   colonne contenant `piece`, `passeport`, `cni` ou `identite` apparaît ailleurs, au motif que
   *« recopiées ici, elles y resteraient indéfiniment »*. Une table `client` dupliquant l'identité
   créerait une seconde surface de rétention — exactement ce que ce contrôle refuse.
3. **La cible des 300 ms.** La recherche porte sur nom, téléphone **et** numéro de pièce. Ces trois
   colonnes doivent vivre dans le même schéma que le filtre « est un client », sans quoi il faudrait
   soit une jointure inter-schémas (**P-04 l'interdit**), soit deux requêtes.

**Alternatives considérées.**

| Option | Rejetée parce que |
|---|---|
| Table `client` complète dans un schéma `sejours` neuf | Duplique l'identité, crée une seconde rétention, et rend la recherche croisée impossible sans violer P-04 |
| Tout mettre sur `comptes.personne`, sans table `client` | La recherche renverrait le **personnel** : chercher « Kouamé » à la réception ferait apparaître la femme de ménage. La qualification est ce qui rend la recherche honnête |
| Table `client` dans `hebergement` | SEJ-05 (clients extérieurs, T2) vend à un client **sans hébergement** ; la fiche serait dans une verticale dont l'établissement peut ne pas avoir le module |

**Ce que la décision n'entame pas.** CPT-00 distingue identité civile / authentification / contrat.
Un client est une identité civile sans compte ni contrat — cas déjà nommé par l'en-tête de `0015`.

---

## R-02 — Le séjour vit dans le schéma `hebergement`, et le lien au client est un UUID nu

**Décision.** `sejour`, `accompagnant`, `note_sejour`, `ligne_sejour`, `fiche_police`,
`numerotation_fiche_police` et `taxe_sejour_constat` vivent dans le schéma **`hebergement`**.
`sejour.client_id` est un `UUID` **sans `REFERENCES`**, lu par le trait `AnnuaireClients`.

**Rationale.** Un séjour est l'occupation d'une unité louable : c'est la verticale hébergement. Le
mettre ailleurs rendrait le lien `sejour ↔ occupation` inter-schémas — donc une jointure interdite
sur **la requête la plus fréquente du cycle** (afficher un séjour avec sa chambre). La clé
étrangère vers le client est en revanche impossible dans les deux sens : le régime du `UUID` nu est
celui de `comptes.permission.module_code`, posé au cycle 003 pour la même raison.

**Alternatives considérées.** Un schéma `sejours` dédié : rejeté, il déplace le problème de
jointure sur le lien le plus chaud au lieu du plus froid. `occupation` porte donc `sejour_id`, pas
l'inverse — voir R-03.

---

## R-03 — `occupation` gagne `sejour_id`, nullable, par migration nouvelle

**Décision.** `ALTER TABLE hebergement.occupation ADD COLUMN sejour_id UUID NULL REFERENCES
hebergement.sejour (id)`, dans la migration `0031`.

**Rationale.** Un séjour porte **une à N** occupations (changement d'unité) ; une occupation
appartient à **au plus un** séjour. La cardinalité met donc la clé du côté de l'occupation.
`NULL` est nécessaire : l'endpoint d'attribution nu du cycle 004 existe toujours et n'ouvre aucun
séjour — le rendre obligatoire casserait une opération servie.

**Le point de vigilance est P-09.** L'`ALTER` ne touche pas la contrainte d'exclusion, mais la
constitution exige de **ré-exercer** une porte dont le périmètre s'étend. Les trois assertions du
cycle 004 sont donc rejouées après migration, et une quatrième est ajoutée : deux arrivées
concurrentes **passant par le parcours de séjour** produisent exactement une écriture, le refus
venant de la contrainte.

**Alternative considérée.** Une table de liaison `sejour_occupation` : rejetée, elle autoriserait
une occupation partagée par deux séjours, ce qu'aucune règle métier ne veut et qu'aucune contrainte
simple n'empêcherait.

---

## R-04 — Le repli des signes diacritiques est écrit à la main, sans dépendance

**Décision.** Une fonction `repli(&str) -> String` dans `crates/socle/comptes/src/client/repli.rs`,
appliquant : minuscules, suppression des signes diacritiques latins par table de correspondance,
suppression des apostrophes **droite (U+0027) et typographique (U+2019)**, réduction des espaces
et des traits d'union. Le résultat est stocké dans des colonnes `*_repli` et indexé.

**Rationale.**

- `unicode-normalization` **n'est pas au gel**. L'ajouter imposerait une entrée nouvelle à
  `docs/versions-gelees.md`, donc une décision de revue mensuelle, pour un besoin qu'une table de
  soixante correspondances couvre.
- Le repli est une **décision de produit**, pas un détail technique : `N'Guessan` doit se trouver
  en tapant `nguessan`, et c'est un choix, pas une conséquence de NFD.
- **Une seule implémentation.** Le front n'ampute rien : il envoie la saisie brute, le serveur
  replie. C'est ce qui évite d'avoir deux replis à garder d'accord — le piège exact de
  `formaterMontant` côté front.

**Alternatives considérées.**

| Option | Rejetée parce que |
|---|---|
| Extension `unaccent` | Extension PostgreSQL nouvelle → question des deux architectures, et une image de base à modifier |
| `pg_trgm` + similarité | Idem, et inutile : dix mille lignes ne demandent pas de recherche floue |
| `lower(nom) LIKE` sans repli | « kouame » ne trouverait pas « KOUAMÉ » — l'exigence FR-004 |

**Le jeu de test est nommé** : `Kouamé`, `N'Guessan`, `N’Guessan`, `Aïcha`, `Traoré`, `Koffi`,
`Yao`, `Bakayoko`, `Adjoua`, `Éboué`, `Gbagbo`, `Ouattara`.

---

## R-05 — La recherche n'a besoin d'aucun index exotique

**Décision.** Trois index B-tree sur les colonnes repliées, portées par `comptes.personne` :
`(tenant_id, nom_repli text_pattern_ops)`, `(tenant_id, telephone_repli)`,
`(tenant_id, numero_piece_repli)`. La recherche par nom est un **préfixe** (`LIKE 'kouam%'`), les
deux autres sont des **égalités ou des suffixes courts**.

**Rationale.** Dix mille lignes tiennent dans quelques mégaoctets ; même un balayage séquentiel
répondrait sous la cible. Les index sont posés parce qu'ils sont gratuits et que le pilote grandira,
pas parce que la cible l'exige. `text_pattern_ops` est ce qui rend un `LIKE 'x%'` indexable quelle
que soit la collation de la base — détail qui se paie cher quand on l'apprend en production.

**Ce qui est mesuré, et comment.** `client_recherche.rs` génère **10 000 fiches** dans un tenant de
mesure, lance cent recherches de chaque forme, et asserte le **95ᵉ centile côté serveur**. Le jeu
n'est **jamais** chargé dans les tenants de démonstration — FR-007.

---

## R-06 — La recherche par téléphone compare des numéros repliés, avec l'indicatif de l'établissement

**Décision.** `telephone_repli` = numéro réduit à ses chiffres, préfixé de l'indicatif par défaut de
l'établissement (`indicatif_telephonique_defaut`, CPT-01, défaut `+225`) quand la saisie n'en porte
pas. La recherche accepte un **suffixe** d'au moins six chiffres.

**Rationale.** Yao tape « 07123456 » ou « 0712345678 » ou lit « +225 07 12 34 56 78 » sur une pièce.
Les trois doivent trouver la même fiche (FR-005). Le suffixe couvre le cas où le client donne son
numéro sans le zéro initial, courant au comptoir.

**Alternative considérée.** Une bibliothèque de numérotation type `libphonenumber` : hors gel, et
massivement surdimensionnée pour un pays unique dont l'indicatif est déjà un paramètre.

---

## R-07 — Une arrivée, une transaction, un appel

**Décision.** `POST /etablissements/{id}/sejours` exécute **dans une seule transaction** :
attribution de l'unité, ouverture du séjour, ouverture de la note et de sa ligne d'hébergement,
numérotation et production de la fiche de police, écriture de l'événement outbox.

**Rationale.** C'est ce que le trait `MoteurDisponibilite::attribuer` a été conçu pour permettre —
sa documentation, écrite au cycle 004, le dit : *« un trait qui prendrait un pool obligerait SEJ-02
à deux transactions — donc à une saga avec compensation explicite, pour une opération qui n'en
demande pas »*. C'est aussi ce que le budget de gestes exige : **au plus un appel réseau bloquant**
entre le premier geste et la confirmation (FR-031).

**Ce que le test doit prouver, et qu'un test naïf ne prouverait pas** : une panne simulée **après**
l'attribution ne laisse **ni** séjour, **ni** note, **ni** fiche de police — et surtout **pas
d'occupation orpheline**, qui rendrait une chambre indisponible sans qu'aucun séjour ne l'explique.

---

## R-08 — Le départ fige un CONSTAT, jamais un montant

**Décision.** `taxe_sejour_constat` enregistre `nuits_constatees`, `nombre_personnes`, et une
**copie** de `assujettie_taxe_nuitee`, `regle_conversion_taxe`, `classement_etablissement` et
`commune`, plus `fige_le`. Les colonnes `nuitees_assujetties` et `montant_mineur` sont **posées et
jamais alimentées** par ce cycle.

**Rationale.** Compter les nuits d'un intervalle est de l'arithmétique ; décider lesquelles sont
assujetties est une **règle fiscale**, qui ne vit que dans `JurisdictionAdapter` (principe V, porte
P-12). `une_nuitee_par_occupation` réduit trois nuits à une : c'est un arbitrage fiscal. Ce cycle
enregistre **trois** et la règle lue, jamais **un**.

**Ce que le figeage garantit malgré tout.** Tout ce qui pourrait changer après le départ —
accompagnants, barème, formule, classement, commune — est recopié. Le montant calculé plus tard par
FIS est donc stable quelle que soit la date du calcul. La spécification exigeait que l'assiette ne
bouge plus ; elle ne bouge pas, et **plus strictement** qu'elle ne le demandait.

**L'immuabilité est un privilège, pas une intention** : `GRANT SELECT, INSERT` seuls. Le rôle
applicatif **ne peut pas** modifier un constat, quelle que soit la ligne de code écrite au-dessus.
C'est le patron « les privilèges disent la classe » du module doré, appliqué à un registre.

**Alternative considérée.** Appeler `JurisdictionAdapter` au départ dès ce cycle : rejetée, elle
tirerait FIS-03 (tranche T3) dans T1, réveillerait P-11, et produirait des règles fiscales écrites
sans le test doré du fiscaliste — le contraire du principe V.

---

## R-09 — Un ajustement est une ligne nouvelle, jamais une modification

**Décision.** Dépassement de palier, départ anticipé, prolongation et changement d'unité produisent
tous une **ligne d'ajustement distincte** portant un `motif`. La ligne initiale est immuable :
`ligne_sejour` reçoit `GRANT SELECT, INSERT` — **pas d'`UPDATE`**.

**Rationale.** Le principe V pose que « les prix sont verrouillés à la création de la ligne ». Une
correction par `UPDATE` effacerait ce qui a été facturé au client à l'arrivée, et le propriétaire
perdrait exactement l'écart que le cahier papier ne lui montrait pas. Le montant d'un ajustement
**peut être négatif** — le type `Rebascule` du cycle 004 le dit déjà : *« peut être négatif — un
départ anticipé existe »*.

**Conséquence testable** : le total de la note est **la somme des lignes**, jamais une colonne
maintenue. Une colonne totalisatrice se désynchronise en silence.

---

## R-10 — Un accompagnant tardif est un orphelin, pas un refus

**Décision.** `POST /sejours/{id}/accompagnants` sur un séjour **clos** rend `202`, écrit une ligne
dans `synchronisation.reconciliation_orpheline`, et **ne touche pas au séjour**.

**Rationale.** `accompagnant` est de classe **A** : il est écrit hors ligne, mis en file, et peut
arriver après la clôture. Le principe VI impose une file de réconciliation à résolution humaine —
*« jamais de rejet silencieux, jamais d'ajout d'office »*. Un `409` serait un rejet ; un `201`
serait un ajout d'office.

**C'est le premier cas réel d'écriture orpheline du produit.** Le cadrage §11.4 le décrit avec une
consommation de bar (T2), mais l'accompagnant le produit dès ce cycle et il est plus simple à
éprouver. `reconciliation_orpheline` gagne `INSERT` et **cesse d'être une provision** — le décompte
de `provisions_sans_logique.rs` passe de six à cinq. Sa **résolution** reste SYN-03, tranche T3.

---

## R-11 — La fiche de police se numérote par un compteur par établissement, pas par une séquence

**Décision.** Une table `hebergement.numerotation_fiche_police (tenant_id, etablissement_id,
dernier_numero)`, incrémentée par `UPDATE … RETURNING` **dans la transaction du check-in**.

**Rationale.** Une `SEQUENCE` PostgreSQL est **globale au schéma** et laisse des trous ; les deux
propriétés sont fatales à une numérotation de document opérationnel, qui doit être **continue par
établissement**. C'est exactement le défaut corrigé par la migration `0012` du cycle 002 — un
espace de numérotation d'outbox partagé entre tenants, *« trouvé ni par relecture ni par une porte,
mais par le premier événement de portée tenant appliqué à un second tenant »*.

Le verrou de ligne de l'`UPDATE` est ce qui sérialise, et c'est **la définition même de la classe
B** : sérialisation à l'échelle d'un établissement.

**Alternative considérée.** Un compteur générique dans le socle pour tous les documents
opérationnels de FIS-02 : rejetée au titre du principe X — un seul consommateur aujourd'hui. Point
de revue nommé : **FIS-02 apportera le second**, et c'est là que la généralisation se décidera.

---

## R-12 — Un endpoint d'état des unités, dérivé, pour l'écran du passage

**Décision.** `GET /etablissements/{id}/hebergement/etat-des-unites` rend toutes les unités de
l'établissement avec leur **état d'occupation dérivé** (libre / occupée jusqu'à *h* / indisponible
pour remise en état) et leur `statut_menage`.

**Rationale.** La maquette `R4` montre la grille complète des chambres avec « Occupée · 16 h 10 » et
« À nettoyer ». L'opération de disponibilité du cycle 004 ne sert pas ce besoin : elle rend les
unités **attribuables d'une catégorie sur un intervalle**, pas l'état de tout le parc à l'instant.

**Ce n'est pas HEB-06.** HEB-06 (P1, hors périmètre) livre le **sous-statut ménage modifiable** et
son écran. Ici l'état d'occupation est **dérivé des occupations**, jamais posé — principe IV — et
`statut_menage` est rendu **en lecture seule**, tel que le cycle 004 l'expose déjà dans
`UniteDisponible`.

**Pourquoi c'est nécessaire au budget** : cet appel se fait **au montage de l'écran**, avant le
premier geste. Il ne compte donc pas dans le budget d'un appel bloquant, qui court du premier geste
à la confirmation.

---

## R-13 — Sept permissions, dont deux transversales

**Décision.**

| Code | Module | Garde |
|---|---|---|
| `sej.client.lire` | *(transversal)* | Recherche, fiche, historique |
| `sej.client.gerer` | *(transversal)* | Création, modification, préférence |
| `heb.sejour.lire` | `HEBERGEMENT` | Liste des séjours, note, fiche de police |
| `heb.sejour.ouvrir` | `HEBERGEMENT` | Arrivée et passage |
| `heb.sejour.clore` | `HEBERGEMENT` | Départ |
| `heb.sejour.prolonger` | `HEBERGEMENT` | Prolongation |
| `heb.sejour.changer_unite` | `HEBERGEMENT` | Changement de chambre |

**Rationale.** Les deux permissions de client sont **transversales** (`module_code = NULL`) parce
qu'un maquis ou un bar seul en aura besoin dès SEJ-05, sans module hébergement. Les cinq autres sont
rattachées à `HEBERGEMENT` : un établissement sans hébergement n'a pas de séjour, et l'entrée doit
être **absente** de l'accueil, pas grisée (principe VII).

**La règle du cycle 003 s'applique** : une permission sans action réelle est refusée. Les sept
gardent une opération servie par ce cycle. `heb.unite.attribuer` reste la garde de l'endpoint nu
d'attribution ; `heb.sejour.ouvrir` garde le parcours — **deux gardes distinctes pour deux chemins
distincts**, et non un doublon.

**Attribution aux rôles** : le `receptionniste` reçoit les sept. Le `gerant` aussi. Le
`proprietaire` reçoit les deux lectures.

---

## R-14 — Neuf types d'événements, par transition métier et non par ligne

**Décision.**

| Type | Agrégat | Émis par |
|---|---|---|
| `sej.client.cree` | `comptes.client` | Création de fiche |
| `sej.client.modifie` | `comptes.client` | Modification de fiche |
| `sej.preference.enregistree` | `comptes.preference_personne` | Préférence (classe A) |
| `sej.accompagnant.ajoute` | `hebergement.accompagnant` | Ajout (classe A) |
| `heb.sejour.ouvert` | `hebergement.sejour` | Arrivée / passage |
| `heb.sejour.prolonge` | `hebergement.sejour` | Prolongation |
| `heb.sejour.unite_changee` | `hebergement.sejour` | Changement de chambre |
| `heb.sejour.clos` | `hebergement.sejour` | Départ |
| `heb.fiche_police.generee` | `hebergement.fiche_police` | Arrivée |

**Rationale.** Le principe II exige un événement par **transition d'état métier**, pas par ligne
écrite. Les lignes de note voyagent donc **dans la charge utile** de la transition qui les crée —
ce qui satisfait aussi l'exigence de charge utile « financière complète et dénormalisée » de
TRX-02 : `heb.sejour.clos` porte le total, les lignes, les ajustements et le constat de taxe, de
sorte que l'opération se reconstitue **sans consulter aucune autre table**.

**Ce qui n'a pas d'événement, et pourquoi c'est dit** : l'écriture orpheline. Elle n'est pas une
transition d'état du séjour — le séjour ne change pas — et son enregistrement dans la file de
réconciliation **est** sa trace. Lui donner un type d'événement obligerait à en écrire un
consommateur qu'aucune story n'appelle.

---

## R-15 — Le budget de gestes est gardé par un test déterministe, pas par un chronomètre

**Décision.** Deux critères distincts :

1. `app/tests/budget-gestes.spec.ts` — **déterministe, en CI** : le parcours de passage compte
   **exactement deux** interactions obligatoires, **zéro** champ de saisie libre obligatoire, et
   **au plus un** appel réseau bloquant entre le premier geste et la confirmation.
2. `tests-e2e/passage.spec.ts` — **part machine** du parcours scripté sous un budget déclaré, sur
   Chromium **et** WebKit, budget fixé **très au-dessus** de la valeur observée.
3. Le chronométrage humain (30 s, 60 s, seuil d'échec à 90 s) est **mesuré au terrain et consigné**,
   jamais asserté en CI.

**Rationale.** C'est la leçon SC-004 du cycle 004, reprise mot pour mot : *« une mesure de latence
en intégration continue dépend de la machine : elle rougirait au hasard, et serait désactivée dans
le mois »*. Un test désactivé ne garde rien — il est **pire** qu'un test absent, parce qu'il donne
l'illusion d'une garde.

**Ce que le budget de gestes garde vraiment.** Il attrape la seule régression qui menace vraiment la
cible : l'ajout d'un champ « juste un de plus » au parcours de passage. Un chronomètre ne
l'attraperait qu'après que la machine soit devenue lente, c'est-à-dire trop tard.

---

## R-16 — Quatre routes, et pas un mot d'anglais dans une URL

**Décision.** `/passage`, `/arrivee`, `/clients`, `/depart`.

**Rationale.** La leçon `S1` du cycle 005 est explicite : *« le nom du fichier de page décide de la
route, et une URL est visible »* — c'est ce qui a fait renommer `/synchronisation` en `/mes-envois`.
« Check-in » et « check-out » sont du jargon anglais absent du lexique ; les maquettes disent
« Arrivée », « Départ », « Le passage ». Les routes suivent, et le lexique est amendé **avant** le
code.

**`/depart` porte la liste des séjours en cours et la note du séjour choisi**, comme `R7` le
montre. Pas de segment dynamique : une route sans paramètre est plus simple à couvrir par P-22, et
le séjour choisi passe en paramètre de requête.

---

## R-17 — `ressource_reservable` n'existe toujours pas, et ce cycle ne la crée pas

**Décision.** Aucune abstraction de ressource réservable n'est introduite. Le point de revue posé au
cycle 004 ([specs/004 research R-09]) reste ouvert.

**Rationale.** Le principe II écrit que le socle « connaît `article_vendable` et
`ressource_reservable` ». Le cycle 004 a vérifié que ces deux noms n'apparaissent dans aucune
migration ni aucun crate, et a retenu la lecture suivante : **l'énoncé est une frontière de
vocabulaire, pas un inventaire de tables**. Il dit ce que le socle ne doit pas connaître — chambre,
unité louable, séjour — et par quels mots il nommerait la chose s'il avait à la nommer.

Ce cycle **renforce** cette lecture plutôt qu'il ne la conteste : il ajoute `sejour` à la liste des
notions que le socle ignore, et la porte P-03 le vérifie. Créer l'abstraction maintenant resterait
spéculatif — **un seul implémenteur**, ce que le principe X interdit. Le second viendra avec RSV
(réservations, tranche T4), et c'est là que la question se rouvrira.

**Ce qui est vérifié, et non supposé** : `socle/comptes` sert la fiche client **sans gagner aucune
notion de séjour**. Le trait va dans l'autre sens — `hebergement` consomme `AnnuaireClients` —, et
`backend/tests/architecture.rs` fait échouer le build sur l'arête inverse.

---

## Points de revue ouverts à la sortie de ce cycle

| # | Point | Quand il se rouvre |
|---|---|---|
| 1 | Généraliser `numerotation_fiche_police` en compteur de documents opérationnels du socle | **FIS-02**, tranche T3 — le second consommateur |
| 2 | `ressource_reservable` — abstraction à un seul implémenteur | **RSV**, tranche T4 |
| 3 | La note devra-t-elle remonter au socle pour servir SEJ-05 (client sans hébergement) ? | **SEJ-05**, tranche T2 |
| 4 | Fusion de deux fiches clients en doublon | Aucune story ne l'appelle ; à ouvrir si le pilote le demande |
| 5 | Rétention 90 jours du numéro de pièce, export et suppression d'une personne | **TRX-06**, P1 — la donnée naît ici, sa politique arrive après, et c'est une **dette nommée** |
