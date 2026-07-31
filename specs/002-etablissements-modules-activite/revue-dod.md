# Revue de la Definition of Done — Cycle 002 · ETB

**T054** · 2026-07-31 · `docs/user-stories-v1.md` §0.4

Les dix points, un par un, **avec la preuve de chacun**. Un point coché sans preuve est un point
que personne n'a vérifié.

> **Le point 10 est SANS OBJET, et c'est écrit ici plutôt que coché en silence.** Même règle qu'au
> cycle 001 pour le point 8.

---

## 1 · Critères d'acceptation couverts par des tests unitaires **et** d'intégration

| Story | Tests | Ce qu'ils couvrent |
|---|---|---|
| ETB-01 | `outbox_transactionnel.rs`, `isolation_tenant.rs` | Création, modification, les trois types d'événements sensibles, l'avertissement de fuseau |
| ETB-02 | `desactivation_bloquee.rs`, `agnosticite_socle.rs` | Activation, désactivation, réactivation qui **restitue** les capacités |
| ETB-02b | `capacites_refusees.rs` | **Les neuf refus, à deux niveaux** — service et base — plus le cas nominal et son rejeu |
| ETB-02c | `agnosticite_socle.rs` | Les trois parcours, 4 étapes exercées / 8, la porte vue échouer |
| ETB-03 | `isolation_tenant.rs`, `outbox_transactionnel.rs` | Points de vente par identifiant direct, comptoir, remplacement de tables |
| ETB-04 | `configuration_heritee.rs` | La **matrice complète** : valeur ET origine à chaque cas |
| ETB-05 | `branding_identite_visuelle.rs` | Surcharge partielle, aperçu sans écriture, mention non fiscale |
| Front | `services-visibles.spec.ts`, `ecran-g1.spec.ts` | Sélection **et rendu** — l'intention et le résultat |

**98 tests backend, 60 tests front, tous verts.**

## 2 · Annotations utoipa à jour, client TS régénéré sans diff

`scripts/ci/generer-client.sh` puis `git diff --exit-code clients/ts` — **aucun diff**. 17 chemins,
24 opérations, 24 `operationId` uniques.

> **Défaut trouvé et corrigé** : les `operationId` étaient déduits du nom de fonction. Deux `lister`
> ou deux `ecrire` produisaient des identifiants en collision, donc un client TypeScript **invalide**
> — `Duplicate identifier 'resoudre'`. Les 24 sont désormais nommés explicitement.

## 3 · Migrations versionnées, `cargo sqlx prepare` vert, seeds à jour

**Sept migrations** livrées (0007 à 0013), `0002` **non modifiée** (P-02 verte). Cache sqlx à
**95 requêtes** pour 109 macros — l'écart de 14 est la déduplication par empreinte du SQL.

> **Sept et non six.** Le plan en annonçait six ; la septième (`0012`) corrige un défaut du cycle
> 001 que ce cycle est le premier à rencontrer — l'unicité de séquence de l'outbox faisait partager
> **un seul espace de numérotation** à tous les événements de niveau tenant, tous tenants
> confondus.

## 4 · RLS `ENABLE` **et** `FORCE` sur les dix tables créées, isolation sur les 21 opérations

**13 tables** dans le schéma `etablissements`, **13 en `ENABLE` + `FORCE` + au moins une
politique** — décompte lu du catalogue système par `couverture_portes.rs`, jamais écrit à la main.
Dix sont créées par ce cycle, trois viennent du 001.

Les **quatre référentiels globaux** ont un **régime nommé** et non une exemption : deux politiques
(`lecture_universelle`, `administration_editeur`), `GRANT SELECT` seul, aucun `tenant_id`. Vérifié
par `rls_catalogue.rs`, avec son test négatif sur les quatre conditions.

**24 opérations servies**, toutes avec un régime d'isolation déclaré. Les deux surfaces à risque
sont testées nommément : la descente de configuration s'isole **à chaque niveau**, et les points de
vente sont visés **par identifiant direct**, hors du chemin de l'établissement.

## 5 · Classe hors-ligne déclarée pour les onze entités, tests du §0.7

Les onze sont en **classe C**. `profil_stock` et `parametre_catalogue` ajoutées au §5.1 dans le même
changement que leur migration, plus la ligne qui manquait : **la lecture en cache d'un référentiel
est de classe A**, quand son écriture reste C.

Côté application, `TYPES_CLASSE_A` n'a reçu **aucun** type du cycle — et c'est désormais opposable
par trois tests, dont une assertion de non-régression qui oblige le prochain cycle à venir écrire
pourquoi il en ajoute un.

**Les tests de rejeu et de désordre de la classe A sont sans objet** : aucune entité de classe A.
L'idempotence des écritures C est néanmoins vérifiée (`201`/`200`/`200`).

## 6 · Événement outbox pour chaque transition

**13 types**, tous couverts. Le plan en annonçait onze : le tableau de `data-model.md` compte onze
*lignes*, dont deux portent chacune deux types. `couverture_portes.rs` compare dans les **deux
sens** — un type déclaré sans test, et un type émis par le code sans être déclaré.

Trois propriétés vérifiées au-delà de la présence :

- **aucun événement sur rejeu** — trois envois du même identifiant, un seul événement ;
- **aucun événement sans transition** — réécrire une valeur identique n'émet rien ;
- **aucun événement sur refus** — une capacité refusée ne laisse ni ligne ni événement.

## 7 · Aucune chaîne en dur, clés fr **et** en, lexique

**63 clés en français, 63 en anglais** — parité vérifiée par `pnpm test:i18n` (P-16). Six entrées
ajoutées au lexique **avant** toute clé i18n, plus la note consignant que `classement` et « numéro
de compte contribuable (NCC) » gardent leur nom officiel.

Le mot « capacité » **n'apparaît nulle part** dans le rendu — vérifié par un test qui a d'ailleurs
attrapé un commentaire de gabarit, lequel part dans le HTML livré.

## 8 · `G1` vérifié en mode clair **et** en mode sombre

**Exigible ici** — le point était sans objet au cycle 001, faute d'écran. C'est la dette que ce
cycle solde.

Vérifié **dans un navigateur**, section par section, dans les deux thèmes, sur les **deux tenants** :
Deloria (cinq services, deux points de vente dont un comptoir) et Résidence Test (un service, aucun
point de vente, section « Points de vente » entièrement absente).

Doublé d'un contrôle mécanique (`theme-sombre.spec.ts`) : chaque jeton de couleur employé porte une
valeur sous `.dark`, et aucune classe `dark:` ne porte de couleur — pas de seconde palette.

## 9 · `politique_impression` exposée et inscrite au récapitulatif

Clé au catalogue, portée la plus basse `POINT_DE_VENTE`, story `ETB-03`, **sans jeu de valeurs** —
il relève du cycle IMP. Inscrite au « Récapitulatif des paramètres d'établissement » dans le même
changement, avec sa clé technique entre accents graves pour que la porte la retrouve.

`parametres_catalogue.rs` rend le principe I·c **vérifiable** : toute clé du catalogue doit figurer
au récapitulatif, comparaison asymétrique.

## 10 · Document imprimé sur imprimante thermique — **SANS OBJET**

*Consigné explicitement, jamais coché.*

L'aperçu d'ETB-05 est un **rendu à l'écran**. Il n'est envoyé à aucune imprimante, ne passe par
aucune file d'impression et ne dépend d'aucun pilote. La première impression réelle du produit
relève du cycle **IMP**, avec la politique d'impression dont ce cycle ne pose que la clé de
catalogue.

Ce qui est vérifié ici est la seule chose vérifiable : le document rendu porte **toujours** la
mention « Document non fiscal — ne tient pas lieu de facture », y compris sur une identité visuelle
vide — le cas le plus probable au premier essai d'un exploitant.

---

## FR-079 et FR-080 — le périmètre tient

`portes_a_vide.rs` le vérifie désormais plutôt que de le supposer :

- **aucune table de provision** — ni `partenaire`, ni `demande_partenaire`, ni
  `compte_compensation` (ETB-07) ;
- **aucun sélecteur de contexte** (ETB-06, P1, hors périmètre) ;
- **aucune logique tarifaire** dans le code du cycle (FR-008, ADM-03 étant une provision) ;
- **aucune entité propre à `SALLE_REUNION`** (FR-019) ;
- **aucune contrainte n'interdit** un compte rattaché à plusieurs établissements (FR-009).

ETB-08 est **déjà satisfaite sans écrire une valeur** : les référentiels étant en table, ajouter
`SPA` sera un `INSERT`, pas une migration.

---

## Ce qui reste non conforme, ou hors du périmètre livré

*Écrit ici pour que la revue de tranche l'arbitre, pas pour être découvert plus tard.*

| Point | État | Qui le doit |
|---|---|---|
| **Police d'icônes** | Les icônes de `G1` ne s'affichent pas : la maquette charge Phosphor depuis un CDN, ce que le mode hors-ligne interdit. Elles sont **décoratives** (`aria-hidden`) et l'écran reste lisible sans elles, mais il n'est pas conforme à la maquette | À embarquer avant la démonstration de tranche |
| **Écriture depuis `G1`** | L'écran **lit** ; aucun bouton n'écrit encore. Les 21 opérations existent et sont testées côté API | Le plan ne décrivait que les quatre sections d'affichage ; l'écriture est à cadrer |
| **Authentification** | Provisoire `CONTEXTE_PAR_EN_TETES` — le tenant et le compte viennent de deux en-têtes non authentifiés | **CPT-01**, condition de levée déjà écrite |
| **DoD n° 10** | Sans objet — voir ci-dessus | Cycle IMP |
| **Obstacles à la désactivation** | Point d'accrochage posé **vide**, exercé par un obstacle factice | Cycle SEJ |
| **Rattachement de caisse** | `caisse_id` accepté sans vérification — `socle/caisse` n'a pas de table | Cycle CAI |
| **Mesures de performance** | Prises sur poste `arm64`, base locale, sans latence réseau. Elles établissent qu'aucune opération n'est structurellement lente, **pas** qu'elles tiendront la cible depuis Abengourou | Mesure sur le pilote |
