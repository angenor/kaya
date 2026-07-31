# Seeds — données de démonstration

**Rejouables, et séparés des migrations** (principe I(b)). Une migration décrit le schéma et n'est
jamais rejouée ; un seed décrit un jeu de données et l'est constamment.

```sh
cargo run -p kaya-api --bin seeds     # deux tenants
cargo run -p kaya-api --bin seeds     # même état final
```

Le code vit dans `backend/api/src/bin/seeds.rs` — un binaire plutôt qu'un fichier SQL, pour que
les insertions passent par le rôle applicatif et posent le contexte de tenant comme le fait
l'application. Un `.sql` exécuté sous le rôle propriétaire contournerait la sécurité au niveau
ligne, et le jeu seedé serait invisible depuis l'application.

---

## Ce que ce cycle livre

| Tenant | Établissement | Pourquoi |
|---|---|---|
| **Deloria** | Résidence Hôtel Deloria — Abengourou | Le pilote (cadrage §2.1) |
| **Résidence Test** | Hébergement seul, **aucun point de vente** | Rend vérifiable que rien dans le socle ne suppose l'existence d'un point de vente |

Le second n'est pas un doublon de confort. Il est là pour la promesse la plus structurante du
produit :

> Aucun crate partagé ne suppose qu'un établissement possède de l'hébergement, ni qu'il possède un
> point de vente (constitution, préambule).

Un jeu à un seul tenant complet la laisserait invérifiable jusqu'au premier client maquis —
c'est-à-dire jusqu'au moment où la corriger coûterait une refonte.

---

## Ce que les cycles suivants doivent ajouter ici

**Portée réduite assumée** (`plan.md`, Complexity Tracking, écart 4). Les valeurs de FR-062
peuplent des tables qui n'existent pas encore. Elles sont écrites ci-dessous pour que chaque cycle
sache ce qu'il doit ajouter, plutôt que de laisser FR-062 partiellement satisfaite en silence.

### Cycle HEB — 17 unités louables

Répartition du cadrage §2.1, à seeder sur l'établissement Deloria.

### Cycle PDV — catalogue

Les **5 catégories** du cadrage §2.1, **décomposées en prix HT + TVA + taxe communale** (FR-062b),
jamais en prix TTC seul. Un catalogue seedé en TTC rendrait impossible tout test du moteur fiscal :
il faudrait retrouver la décomposition, donc supposer le taux, donc supposer ce qu'on veut tester.

| À seeder | Forme exigée |
|---|---|
| Catégorie et tarif | `prix_ht_mineur`, `tva_mineur`, `taxe_communale_mineur` — **entiers d'unité mineure** |
| Barèmes de passage | **Marqués provisoires** tant que **B-07** n'est pas tranchée |

> **Les barèmes de passage sont provisoires.** Le récapitulatif des paramètres de
> `docs/user-stories-v1.md` les porte, mais la décision B-07 n'est pas prise. Les seeder comme
> définitifs les figerait dans les tests dorés fiscaux, et la décision deviendrait un changement
> de tests plutôt qu'un paramètre. Les marquer provisoires est ce qui garde B-07 ouverte.

### Cycle CPT — 5 comptes de test

Un compte par rôle du cadrage, **avec rôles cumulés sur au moins un d'entre eux** : les rôles
cumulables sont la norme, pas l'exception (principe VII), et un jeu de test à un rôle par compte
ne montrerait jamais le cas normal.

---

## Règles pour tout seed ajouté ici

1. **Identifiants fixes**, écrits en dur. Un `Uuid::now_v7()` produirait un nouveau jeu à chaque
   exécution : la base grossirait sans fin, et « recharger la démonstration » créerait un
   troisième établissement au lieu de retrouver le premier.
2. **`ON CONFLICT DO NOTHING`**, jamais un `DELETE` préalable. Un seed qui purge détruit les
   données de travail du pilote à chaque démonstration.
3. **Sous le rôle applicatif**, contexte de tenant posé.
4. **Montants en entiers d'unité mineure**, quantités en `NUMERIC` (principe V). La porte P-10
   vérifie les migrations ; les seeds, eux, ne sont vérifiés que par la revue — d'où cette ligne.
5. Le test `backend/tests/seeds_rejouables.rs` doit rester vert : **trois exécutions, même état**.
