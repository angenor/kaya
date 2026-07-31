# Guide de validation — cycle 001

**Objet** : prouver, commande par commande, que le socle technique fonctionne. Chaque section
correspond à un critère de succès de [spec.md](./spec.md).

**Ce document n'est pas un tutoriel d'implémentation.** Il décrit ce qu'on exécute et ce qu'on
doit constater. Le détail des couches est dans `docs/module-dore.md` (livrable du cycle).

---

## 0. Prérequis

| Élément | Version | Source |
|---|---|---|
| Rust (toolchain) | épinglée par `rust-toolchain.toml` | `docs/versions-gelees.md` §2 |
| Node.js | épinglée par `.nvmrc` | `docs/versions-gelees.md` §3.3 |
| pnpm | épinglée par `package.json` | `docs/versions-gelees.md` §3.3 |
| Docker + Compose | — | poste de développement |

> Aucun numéro n'est reproduit ici **volontairement** : `docs/versions-gelees.md` fait foi, et une
> copie dans un guide dériverait à la première revue mensuelle.

Le poste de développement est `darwin/arm64`, la production `linux/amd64`. Les trois images
(base, cache, stockage objet) sont multi-architecture ; **le binaire Rust ne l'est pas**.

---

## 1. Amorçage — SC-001 (< 30 minutes, poste neuf)

```sh
git clone <dépôt> && cd kaya
docker compose -f infra/compose.yml up -d      # base, cache, stockage objet
scripts/dev/preparer-base.sh                   # migrations + mots de passe locaux — voir ci-dessous
cd backend && cargo build --workspace          # le workspace Rust vit dans backend/
cargo run -p kaya-api --bin kaya-api           # applique les migrations, puis écoute
cd .. && pnpm install && pnpm --filter @kaya/app dev
```

> **Pourquoi une étape de plus que prévu.** Les mots de passe des rôles ne sont **pas** dans les
> migrations : un secret écrit dans une migration est un secret dans l'historique Git, en clair,
> pour toujours — et une migration appliquée ne se modifie jamais, donc l'erreur serait
> définitive. `scripts/dev/preparer-base.sh` les pose après migration ; la CI fait de même avec
> les siens, la production les tient hors du dépôt.

**À constater** :

- `docker compose ps` → trois services sains.
- `cargo build --workspace` → vert, **tous** les crates de `socle/`, `capacites/`, `verticales/`,
  plus `domain`, `api` et `node`.
- Les migrations s'appliquent **au démarrage de l'API** (R-12), sous le rôle propriétaire — les
  journaux le disent explicitement.
- `curl localhost:<port>/health` → `200` avec les trois dépendances à l'état opérationnel.

**Piège attendu** : si `cargo build` échoue sur le linker, vérifier que la configuration `mold` est
bien **conditionnée à la cible Linux** (R-01). `mold` n'existe pas sur macOS ; l'imposer au poste
de développement est une erreur de configuration, pas un problème d'installation.

---

## 2. Contrat d'API et client généré — SC-002 (porte P-01)

```sh
curl localhost:<port>/api-docs/openapi.json | jq '.paths | keys'
```

**À constater** : `/health` y figure (FR-031), ainsi que les deux endpoints de notes.

**Test négatif — c'est lui qui compte** :

```sh
# 1. Modifier une signature de handler sans régénérer le client
# 2. Lancer la CI localement
```

**À constater** : le build **échoue** sur un diff de client non commité. Un build vert ici
signifierait que la porte P-01 n'est pas branchée.

> **R-14 est levée.** Le gel 1.0.3 a ajouté `openapi-typescript` ; le gel **1.0.4** a corrigé la
> version de TypeScript qui l'accompagne — `7.0.2` rendait la génération impossible, l'outil
> exigeant `^5.x`. Section exécutable :
>
> ```sh
> scripts/ci/generer-client.sh --verifier
> ```
>
> Le mode `--verifier` exécute d'abord les **deux exigences du gel §3.2** : déterminisme d'octet
> constaté par `cmp`, et ordre de membres stable constaté en ajoutant un endpoint.

---

## 3. Isolation multi-tenant — SC-005 (portes P-07, P-08)

```sh
cd backend
cargo test -p kaya-backend --test isolation_tenant
cargo test -p kaya-backend --test rls_catalogue
```

> Les tests transverses appartiennent au paquet racine **`kaya-backend`**, pas à `kaya-api` : ils
> traversent plusieurs crates et ne peuvent donc appartenir à aucun d'eux sans lui donner une
> dépendance vers tous les autres — ce que la hiérarchie du principe II interdit.

**À constater** :

- `rls_catalogue` liste les tables des schémas applicatifs et échoue sur toute table sans
  `ENABLE`, sans `FORCE`, ou sans politique. Le test lit **le catalogue PostgreSQL**, pas les
  fichiers de migration (R-09).
- `isolation_tenant` s'authentifie sur le tenant Deloria et vise chaque endpoint avec un
  identifiant appartenant à « Résidence Test » → **aucune ligne lue, aucune écrite**.

**Test négatif** :

```sh
# Créer une table sans politique RLS dans une migration, puis relancer
```

**À constater** : `rls_catalogue` échoue, avec le nom de la table fautive dans le message.

**Vérification manuelle du point le plus glissant** — une transaction sans contexte de tenant ne
doit rien voir :

```sql
BEGIN;
SELECT * FROM etablissements.note_etablissement;   -- attendu : 0 ligne, PAS une erreur
COMMIT;
```

---

## 4. Grand livre d'événements — SC-003, SC-004 (porte P-05)

### 4.1 Reconstitution autonome — le test central du cycle

```sh
cargo test -p kaya-backend --test reconstitution_autonome
```

**Ce que le test fait** : il se connecte avec le rôle `kaya_ledger_reader`, qui a le droit de lire
`synchronisation.evenement_outbox` **et rien d'autre**, puis reconstitue chaque opération du jeu
de cas financier figé depuis la seule charge utile.

**À constater** : le test passe. Toute lecture d'une autre table lèverait une erreur de permission
PostgreSQL — c'est la preuve, et non une convention de rédaction (R-11).

**Test négatif** :

```sh
# Ajouter un JOIN vers etablissements.etablissement dans la reconstitution
```

**À constater** : échec immédiat sur permission refusée.

### 4.2 Immuabilité

```sql
-- Sous le rôle applicatif
UPDATE synchronisation.evenement_outbox SET payload = '{}'::jsonb WHERE id = '<un id>';
DELETE FROM synchronisation.evenement_outbox WHERE id = '<un id>';
```

**À constater** : les deux échouent. Puis **répéter sous le rôle propriétaire** — les deux
échouent encore, cette fois par le déclencheur (R-05). C'est ce second essai qui compte : c'est
le scénario réel du développeur solo connecté en production.

Seule mutation acceptée : `publie_le` passant de `NULL` à une valeur, une seule fois.

### 4.3 Redémarrage brutal du worker — SC-004

```sh
cargo test -p kaya-backend --test worker_redemarrage
```

**À constater** : le nombre d'événements en base est **identique avant et après**, et les
consommateurs voient l'effet d'une seule présentation malgré la republication.

---

## 5. Module doré et tests hors-ligne — §0.7 des user stories

```sh
cargo test -p kaya-backend --test note_etablissement_classe_a
```

**À constater** :

- **Rejeu** — le même `id` envoyé trois fois produit **un seul** enregistrement, et l'API répond
  `200` (pas `409`) aux deuxième et troisième envois.
- **Désordre** — trois notes appliquées dans les **six** ordres possibles produisent le même état
  final.

```sh
cargo test -p kaya-backend --test classes_offline
```

**À constater** : toute table d'un schéma applicatif absente de `docs/registre-classes-offline.md`
fait échouer le test (R-10).

---

## 6. Fondations d'interface — SC-009

**Ce cycle ne livre aucun écran.** La couche écran du module doré est reportée au cycle ETB :
l'écran de notes n'hérite d'aucun motif de la matrice `docs/Kaya_Design.md` §25, et un écran sans
motif ne se code pas. Ce qui se valide ici, ce sont les **fondations** que le cycle ETB
consommera.

```sh
pnpm --filter @kaya/app dev
```

| Vérification | Attendu |
|---|---|
| `app/assets/css/theme.css` | Copie **exacte** de `docs/design/theme.css` — seule exception du principe XII |
| Mode sombre | Câblé par la variante `dark:`, jamais une seconde palette |
| Catalogues i18n | `fr` et `en` à parité ; « Note interne » / *Internal note* présents |
| `PlatformAdapter` | Quatre implémentations présentes, chacune renvoyant `{ disponible: false }` pour ce qu'elle ne sait pas faire |
| Couche native | Aucun import de `@tauri-apps/api` hors de `app/core/platform/` |
| Aucun écran | Aucun fichier de `docs/design/html/` copié sous `app/` |

```sh
pnpm --filter @kaya/app test:i18n      # parité fr/en — porte P-16
pnpm --filter @kaya/app lint:tokens    # littéraux hors jetons — porte P-17
pnpm --filter @kaya/app lint           # @tauri-apps/api hors PlatformAdapter — porte P-15
pnpm --filter @kaya/app test           # file de classe A — porte P-13
```

---

## 7. Seeds — SC-007

```sh
cd backend
cargo run -p kaya-api --bin seeds
cargo run -p kaya-api --bin seeds        # deuxième exécution : même état final
cargo test -p kaya-backend --test seeds_rejouables   # trois exécutions, même état
```

**À constater** : deux tenants — Deloria (établissement d'Abengourou, fuseau `Africa/Abidjan`) et
« Résidence Test ». Rechargement **en une commande**, résultat identique.

> **Portée réduite, assumée** : les 17 unités, le catalogue et les comptes de test (FR-062) ne
> peuvent pas être seedés à ce cycle — leurs tables appartiennent aux cycles HEB, PDV et CPT. Ce
> cycle livre **la mécanique de seeds et les deux tenants** ; le contenu métier s'y ajoute à
> mesure. Voir `plan.md`, section « Écarts assumés ».

---

## 8. Sauvegarde et restauration — SC-006

```sh
infra/backup/sauvegarder.sh                     # dump chiffré → stockage tiers
infra/backup/restaurer.sh <horodatage> <cible>  # restauration en environnement vierge
```

**À constater** : la restauration aboutit, sa durée est **chronométrée et consignée**, et la
procédure est suivie **par quelqu'un qui n'a pas écrit le système** (c'est la seule façon de
savoir si elle est complète).

> **⚠ Section non exécutée au cycle 001.** Les deux scripts sont écrits et leurs garde-fous ont
> été déclenchés pour vérifier qu'ils refusent bien. L'exercice complet demande un fournisseur de
> stockage tiers provisionné, une paire de clés `age` de production et un serveur vierge — aucun
> des trois n'existe. **FR-060 n'est donc pas satisfaite et SC-006 reste ouvert.** Consigné dans
> `infra/backup/README.md` §6.

**À vérifier explicitement** : la sauvegarde est déposée sur un **hôte distinct** du serveur de
production, avec verrouillage d'objet. Une sauvegarde présente uniquement dans le stockage objet
local **ne satisfait pas** FR-060 — les deux tomberaient ensemble.

---

## 9. Construction de production

```sh
docker buildx build --platform linux/amd64 -f infra/Dockerfile.api -t kaya-api:<tag> .
```

**À constater** : le binaire est construit **dans le conteneur pour `linux/amd64`**, jamais copié
depuis le poste. Un `cargo build` local produit un binaire `aarch64-apple-darwin` non déployable
(`docs/versions-gelees.md` §4.2).

**Corollaire à ne pas oublier** : les mesures de temps de compilation de SC-010 se font **dans ce
conteneur**, pas sur le poste — c'est le seul endroit où `mold` est actif.

> **⚠ Image `linux/amd64` non produite au cycle 001.** `infra/Dockerfile.api` est écrit, ses deux
> images de base sont vérifiées sur le registre, et il construit **sans base de données** grâce à
> `backend/.sqlx` et `SQLX_OFFLINE=true`. Sur un poste Apple Silicon, la construction croisée
> passe par l'émulation : la mesure obtenue ne dirait rien de la production. **SC-010 reste donc
> non mesuré** — ce que R-01 annonçait déjà.
