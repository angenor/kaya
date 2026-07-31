# Guide de validation — Cycle 002 · Établissements, modules d'activité et configuration héritée

**Phase 1 du plan** · 2026-07-31 · [plan.md](plan.md) · [spec.md](spec.md)

Ce que quelqu'un exécute pour constater que le cycle est livré. **Ni code d'implémentation, ni
migrations, ni suites de tests** — celles-ci vivent dans `tasks.md` et dans le dépôt.

Ordre de lecture : les prérequis, puis les huit vérifications, dans l'ordre. La **vérification 1**
est celle qui compte : si elle tombe, aucune des sept autres ne rattrape le cycle.

---

## Prérequis

```sh
docker compose -f infra/compose.yml up -d
scripts/dev/preparer-base.sh          # rôles, schémas, migrations
scripts/dev/preparer-stockage.sh      # cluster S3, clé, compartiment — requis par ETB-05
cd backend/api && cargo sqlx migrate run --source ../migrations && cd ../..
cargo run -p kaya-api --bin seeds
```

`sqlx.toml` se résout depuis `backend/api/`, jamais depuis la racine du workspace — sans quoi le
CLI et la macro tiennent deux tables de suivi différentes et rejouent tout au démarrage
(`docs/module-dore.md`, § Pièges de l'outillage).

---

## 1 · Les trois parcours structurels — la vérification qui décide du cycle

```sh
cargo test --test agnosticite_socle -- --nocapture
```

**Attendu** : trois parcours verts, et un décompte affiché pour chacun.

```
maquis            RESTAURATION seul     4 étapes exercées / 8 déclarées   4 dues (PDV, CAI, FIS)
residence         HEBERGEMENT seul      4 / 8                             4 dues
agnosticite       MODULE_FICTIF_TEST    4 / 8                             4 dues — 0 capacité
```

Le décompte n'est pas décoratif : c'est l'exigence 2 du § « Couverture des portes » de la
constitution. Une porte qui inspecte un sous-ensemble sans le dire donne une fausse assurance, ce
qui est pire que pas de porte.

**Le test négatif, à exécuter au moins une fois à la main** — créer une table portant le nom d'une
sentinelle d'étape due, relancer, constater l'échec :

```
ÉCHEC — étape « vente_comptoir » : sentinelle documents.commande PRÉSENTE,
        branchement ABSENT sur les 3 parcours.
        Le cycle PDV a livré l'étape sans la brancher.
```

Puis supprimer la table. **Sans avoir vu cet échec une fois, on ne sait pas si la porte regarde.**

**Vérifier que le service fictif n'a pas fui** :

```sh
psql "$DATABASE_URL" -c "SELECT count(*) FROM etablissements.module_activite
                         WHERE code = 'MODULE_FICTIF_TEST'"   # attendu : 0
```

---

## 2 · Un service inactif est absent, pas grisé

```sh
cd app && pnpm test && cd ..
```

Puis à l'œil, sur les deux établissements seedés :

```sh
cd app && pnpm dev
```

| Établissement | Attendu sur `G1` |
|---|---|
| Deloria, Abengourou | Cinq services présents |
| Résidence Test | **Un seul** service. Les mots « restauration », « bar », « pressing », « salle de réunion » n'apparaissent nulle part — ni grisés, ni annoncés, ni dans le HTML |

Le contrôle par le HTML, plutôt que par l'œil, est celui qui tient : un libellé masqué par du CSS
passerait l'inspection visuelle.

```sh
curl -s localhost:3000/… | grep -ci "pressing"     # attendu : 0 sur Résidence Test
```

**Vérifier les deux thèmes** — bascule clair/sombre sur `G1`, chaque section lue dans les deux. Le
point 8 de la Definition of Done, sans objet au cycle 001, est **exigible ici** : c'est le premier
écran du produit.

---

## 3 · Les neuf refus de capacité et de profil

```sh
cargo test --test capacites_refusees
```

**Attendu** : neuf refus, chacun nommant sa valeur, **et zéro ligne écrite**.

| Tentative | Attendu |
|---|---|
| `LIVRAISON`, `PRODUCTION`, `COMMERCE_EN_LIGNE`, `FIDELITE`, `DEVIS`, `COMPTES_CLIENTS` | `422 capacite_non_implementee`, valeur nommée |
| profil `VALORISE`, `DETAILLE` | `422 profil_non_implemente`, profil nommé |
| profil `AUCUN` | `422 profil_non_implemente`, **message distinct** : une capacité non consommée ne se déclare pas |
| `STOCK` / `SIMPLE` | `201`, puis `200` au rejeu |

**Le refus doit tenir sans passer par le service.** Tenter l'écriture directement en base, sous le
rôle applicatif :

```sh
psql "$DATABASE_URL_APP" -c "INSERT INTO etablissements.module_capacite (…) VALUES (…, 'LIVRAISON', …)"
# attendu : violation de contrainte CHECK — la base refuse, pas seulement l'API
```

Si cette insertion passe, le refus n'est qu'applicatif et un import le contournera.

---

## 4 · La résolution de configuration, sur toute la matrice

```sh
cargo test --test configuration_heritee
```

**Attendu** : la matrice complète des combinaisons — quatre niveaux, chacun défini ou absent,
chaînes écourtées comprises — avec **la valeur et son origine** vérifiées à chaque cas.

Cinq cas à lire dans la sortie, parce que ce sont ceux qu'on écrit mal :

| Cas | Attendu |
|---|---|
| Défini au tenant seul | Valeur du tenant, `origine = TENANT` |
| Défini au tenant **et** au point de vente | Valeur du point de vente ; l'autre point de vente du même établissement rend celle du tenant |
| Surcharge partielle — tenant et point de vente, ni établissement ni service | Valeur du point de vente. **Aucune erreur, aucune valeur intermédiaire inventée** |
| Défini nulle part | **Absent de la réponse.** Ni `null`, ni valeur par défaut |
| Surcharge sur un service désactivé | La résolution remonte au niveau supérieur ; la surcharge est **toujours en base** et revient à la réactivation |

**Vérification manuelle du dernier cas** — désactiver un service, résoudre, réactiver, résoudre à
nouveau. La valeur doit revenir. Si elle ne revient pas, quelque chose supprime au lieu de rendre
inerte.

---

## 5 · Isolation entre tenants, à chaque niveau de la chaîne

```sh
cargo test --test isolation_tenant
```

**Attendu** : les vingt et une opérations couvertes, aucune ligne du tenant B atteignable depuis le
tenant A — y compris par identifiant direct.

Deux assertions à retrouver explicitement dans la sortie :

- **La descente de configuration s'isole à chaque niveau.** Résoudre depuis le tenant A avec le
  `point_de_vente_id` du tenant B ne rend **rien** — pas même la valeur héritée du tenant A.
- **Les trois référentiels rendent la même chose aux deux tenants**, et c'est *asserté*. Sans cette
  assertion, un relecteur futur prendra le comportement pour une fuite et « corrigera » un
  référentiel global en référentiel par tenant.

---

## 6 · Migrations, registre et catalogue

```sh
cargo test --test classes_offline        # deux tables ajoutées au registre
cargo test --test parametres_catalogue   # toute clé du catalogue figure au récapitulatif
scripts/ci/migrations-figees.sh          # P-02 — aucune migration appliquée modifiée
cd backend/api && cargo sqlx prepare --check --workspace -- --all-targets && cd ../..
```

**Attendu** : quatre vertes. Deux pièges méritent une vérification à la main, parce qu'ils
réussissent en silence :

```sh
# Les colonnes ajoutées sont REMPLIES sur les lignes préexistantes.
psql "$DATABASE_URL" -c "SELECT nom, juridiction, classement, commune FROM etablissements.etablissement"
```

Si `juridiction` est vide sur les deux établissements seedés, la migration a écrit par `UPDATE`
sous sécurité au niveau ligne forcée : **aucune ligne touchée, aucune erreur levée**
([research.md R-08](research.md)). C'est le défaut le plus discret du cycle.

```sh
# Le cache de requêtes couvre TOUT le dépôt, pas un sous-ensemble.
ls backend/api/.sqlx | wc -l    # comparer au nombre de requêtes du cycle
```

Le cycle 001 a livré une porte P-18 qui validait 43 requêtes sur 47. Le décompte se lit, il ne se
suppose pas.

---

## 7 · Contrat, client et portes de structure

```sh
scripts/ci/generer-client.sh            # P-01 — aucun diff après génération
git diff --exit-code clients/ts
cargo test --test architecture          # P-03, P-12
scripts/ci/jointures-inter-schemas.sh   # P-04
cargo test --test outbox_transactionnel # P-05
scripts/ci/outbox-sans-purge.sh         # P-05b
cargo test --test rls_catalogue         # P-07 — les 4 référentiels ont leurs 2 politiques
scripts/ci/maquettes-non-copiees.sh     # P-19
scripts/ci/versions-epinglees.sh        # P-20
cd app && pnpm test:i18n && pnpm lint:tokens && cd ..   # P-16, P-17
```

**Attendu** : toutes vertes. `rls_catalogue` mérite un regard : les quatre référentiels globaux
n'ont **pas** de politique d'isolation par tenant, mais bien deux politiques —
`lecture_universelle` et `administration_editeur`. La porte doit les compter comme conformes **et
le dire**, pas les ignorer.

Puis le recollement des trois portes qui s'étendent sur plusieurs phases :

```sh
cargo test --test couverture_portes
```

**Attendu** : trois décomptes affichés et concordants — **11/11** types d'événements couverts
(P-05), **10/10** tables inspectées (P-07), **21/21** chemins isolés (P-08). Un écart fait échouer
la porte en nommant ce qui manque. C'est ce test, et non la relecture, qui ferme le trou qu'une
porte étendue phase par phase laisse par construction.

---

## 8 · Les seeds, rechargés trois fois

```sh
cargo run -p kaya-api --bin seeds
cargo run -p kaya-api --bin seeds
cargo run -p kaya-api --bin seeds
cargo test --test seeds_rejouables
```

**Attendu** : état final identique aux trois exécutions.

```sh
psql "$DATABASE_URL_APP" -c "…"   # avec app.current_tenant posé
```

| Établissement | Attendu |
|---|---|
| Abengourou | 5 services actifs · `STOCK`/`SIMPLE` déclarée par `RESTAURATION` et `BAR` · non classé · `Africa/Abidjan` · `XOF` |
| Résidence Test | 1 service actif · **0 capacité** · 0 point de vente |
| Les deux | **0 occurrence** de `MODULE_FICTIF_TEST` |

---

## Déroulé réel — 2026-07-31, T053

*Consigné plutôt que corrigé en silence : le quickstart décrit l'attendu, cette section dit
l'observé.*

Les huit vérifications ont été déroulées dans l'ordre, sur la base des seeds rechargés. **Toutes
vertes.** Quatre écarts entre l'attendu et l'observé, tous en faveur du produit sauf le dernier.

| § | Écart entre l'attendu et l'observé |
|---|---|
| **1** | Décompte à **3 exercées / 8** au moment de la rédaction du guide, **4 / 8** à la clôture — l'étape `resolution_configuration` a été branchée par la Phase 6, postérieure au guide. Le message d'échec observé au test négatif diffère aussi de celui écrit ici : il nomme `fiscalite.document_fiscal` et **les trois parcours séparément**, alors que le guide n'en montrait qu'un. Trois manquements, pas un — brancher l'étape sur un seul parcours ne suffirait pas |
| **6** | `ls backend/api/.sqlx` est le **mauvais chemin** : le cache vit à `backend/.sqlx`, à la racine du workspace. Le décompte réel est **95 fichiers pour 109 macros `query!`** — l'écart de 14 est la déduplication par empreinte du SQL, deux requêtes littéralement identiques ne produisant qu'un fichier. Le guide laissait attendre l'égalité |
| **7** | Le recollement annonce **11/11 types d'événements** ; le décompte réel est **13**. Le tableau de `data-model.md` compte onze *lignes*, mais deux d'entre elles portent chacune deux types (`point_de_vente.cree` / `.modifie`, `table_pdv.creee` / `.desactivee`). C'est le décompte du recollement qui fait foi — un total tiré du nombre de lignes d'un tableau compte des lignes, pas des types |
| **2** | **Le guide ne pouvait pas être suivi tel quel** : `pnpm dev` seul ne suffit pas, l'écran a besoin de quatre variables d'environnement (`NUXT_PUBLIC_API_BASE_URL`, `TENANT_ID`, `COMPTE_ID`, `ETABLISSEMENT_ID`) — le contexte d'appel étant encore le provisoire `CONTEXTE_PAR_EN_TETES`. Corrigé ci-dessous |

**Commande réelle pour la vérification 2** :

```sh
cd app && NUXT_PUBLIC_API_BASE_URL=http://localhost:8080 \
  NUXT_PUBLIC_TENANT_ID=0198c4a0-0000-7000-8000-000000000001 \
  NUXT_PUBLIC_COMPTE_ID=<un uuid quelconque> \
  NUXT_PUBLIC_ETABLISSEMENT_ID=0198c4a0-0000-7000-8000-000000000002 \
  pnpm dev
```

Pour la **Résidence Test**, remplacer le tenant par `…011` et l'établissement par `…012`. Les deux
identifiants vont **ensemble** : viser l'établissement d'un autre tenant rend `404`, ce qui est
l'isolation qui fonctionne, pas une erreur de manipulation.

### Les deux critères chiffrés, mesurés

*Un critère chiffré que personne ne mesure n'est pas un critère.*

Mesures d'**aller-retour d'API**, binaire en `--release`, base locale, trois exécutions chacune :

| Critère | Cible | Mesuré | Marge |
|---|---|---|---|
| **SC-008** — activation d'un service | 30 s à l'écran | **10 à 42 ms** | l'aller-retour est négligeable devant les 30 s, qui couvrent la saisie humaine |
| **SC-009** — aperçu d'identité visuelle | 2 s | **0,5 à 0,7 ms** | l'aperçu ne touche aucune table et n'écrit rien |
| Résolution de configuration | une descente | **4 à 7 ms** | une seule requête, conforme à la conception |

**Ce que ces chiffres ne disent pas.** Ils sont pris sur un poste de développement `arm64`, base
locale, sans latence réseau. La production est un VPS Contabo `linux/amd64` atteint depuis
Abengourou : les mesures ne se transposent pas. Ce qu'elles établissent est qu'**aucune des trois
opérations n'est structurellement lente** — pas qu'elles tiendront la cible sur le terrain, ce que
seule une mesure sur le pilote dira.

---

## Ce qui n'est pas vérifiable à ce cycle

Consigné explicitement plutôt que coché en silence — même règle qu'au cycle 001.

| Point | Raison |
|---|---|
| Definition of Done n° 10 — document imprimé sur imprimante thermique réelle | L'aperçu d'ETB-05 est un rendu à l'écran. La première impression réelle est au cycle IMP (tranche T2) |
| Comportement métier des étapes dues des trois parcours | Vente comptoir, encaissement, document fiscal et clôture appartiennent aux cycles PDV, CAI et FIS. Le harnais vérifie leur **branchement**, pas leur justesse |
| Refus de désactivation d'un service occupé | Aucune verticale ne crée encore d'opération. Le chemin est exercé par un obstacle factice, jamais par un vrai séjour |
| Vérification du rattachement de caisse | `socle/caisse` n'a pas de table avant le cycle CAI |

---

## Voir aussi

- [`plan.md`](plan.md) — les vingt portes et leur mécanisme de vérification
- [`data-model.md`](data-model.md) — les onze tables, leurs contraintes et leurs privilèges
- [`contracts/http-api.md`](contracts/http-api.md) — les vingt et une opérations
- [`contracts/traits-exposes.md`](contracts/traits-exposes.md) — les six traits
- `docs/module-dore.md` — le patron des six couches, à suivre sans le réinventer
