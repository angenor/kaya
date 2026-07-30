# Kaya — Versions gelées

*Application du principe XI de `.specify/memory/constitution.md` : dernières versions stables,
**vérifiées sur les registres officiels avec l'URL citée**, puis **épinglées exactement** et
figées par lockfiles.*

**Version du gel : 1.0.0 — vérifié le 2026-07-30**
**Prochaine revue : 2026-08-31** (revue mensuelle groupée)

---

## 1. Règles d'usage de ce document

1. **Aucun numéro de version de ce tableau n'a été écrit de mémoire.** Chaque ligne porte
   l'URL du registre officiel interrogé et la date de vérification.
2. **Épinglage exact obligatoire** : `= 4.14.0` ou `4.14.0`, jamais `^4.14`, `~4.14` ni `4.*`.
   La porte de CI **P-20** échoue sur tout intervalle et sur tout lockfile absent ou périmé.
3. **Aucune montée majeure pendant un incrément.** Une faille de sécurité est la seule
   exception, et elle est consignée au §6.
4. **La revue est mensuelle et groupée**, jamais au fil de l'eau. Elle met à jour ce fichier,
   les manifestes et les lockfiles dans un seul changement.
5. Reproduire la vérification : les commandes exactes sont au §5.

---

## 2. Les dix briques du principe XI

| # | Brique | Version gelée | Publiée le | Registre officiel interrogé |
|---|---|---|---|---|
| 1 | **Rust** (toolchain stable) | **1.97.1** | 2026-07-14 | `https://static.rust-lang.org/dist/channel-rust-stable.toml` — section `[pkg.rust]` |
| 2 | **Actix Web** | **4.14.0** | 2026-06-21 | `https://crates.io/api/v1/crates/actix-web` |
| 3 | **sqlx** | **0.9.0** ⚠️ | 2026-05-21 | `https://crates.io/api/v1/crates/sqlx` |
| 4 | **utoipa** | **5.5.0** | 2026-05-04 | `https://crates.io/api/v1/crates/utoipa` |
| 5 | **Nuxt** | **4.5.1** | — | `https://registry.npmjs.org/nuxt/latest` |
| 6 | **Tailwind CSS** | **4.3.3** | — | `https://registry.npmjs.org/tailwindcss/latest` |
| 7 | **Tauri** (crate) | **2.11.5** | 2026-07-01 | `https://crates.io/api/v1/crates/tauri` |
| 8 | **PostgreSQL** | **18.4** | 2026-05-14 | `https://www.postgresql.org/versions.json` |
| 9 | **Redis** | **8.10.0** ⚠️ | 2026-07-29 | `https://api.github.com/repos/redis/redis/releases` |
| 10 | **Garage** | **2.3.0** | 2026-04-16 | `https://git.deuxfleurs.fr/api/v1/repos/Deuxfleurs/garage/releases` |

### ⚠️ Deux points d'attention avant d'épingler

**sqlx 0.9.0 — changement de version mineure avec ruptures d'API.** La stable précédente était
`0.8.6` (2025-05-19), soit un an d'écart. La quasi-totalité de la documentation, des exemples et
des réponses en ligne visent encore `0.8.x`. Conséquence pratique : le **module doré** (cadrage
§13.1) doit être écrit contre `0.9.0` et servir de patron, sinon chaque génération assistée
réintroduira des appels `0.8`. Décision : on prend `0.9.0` — commencer sur une version d'un an
d'âge coûterait une migration en cours de projet.

**Redis 8.10.0 — publiée le 2026-07-29, soit la veille de ce gel.** Aucun recul d'exploitation.
Redis ne portant que de l'éphémère reconstructible (principe II), le risque est contenu : une
régression se corrige par un redémarrage, sans perte durable. Deux options légitimes :

| Option | Version | Argument |
|---|---|---|
| **Retenue** | **8.10.0** | Principe XI à la lettre ; le rôle éphémère borne le risque |
| Alternative | 8.8.1 (2026-07-23) | Une semaine de recul, même branche majeure |

Le passage de l'une à l'autre est un changement de tag dans `compose.yml`, sans impact sur le
code. À rouvrir à la revue du 2026-08-31 si un incident survient.

---

## 3. Dépendances directes du socle

Épinglées au même titre. Ce ne sont pas des « briques » au sens du principe XI, mais elles
entrent dans le lockfile et la porte P-20 les couvre.

### 3.1 Écosystème Rust

| Crate | Version | Rôle | Contrainte vérifiée |
|---|---|---|---|
| `utoipa-swagger-ui` | **9.0.2** | Swagger UI, protégée hors production | dépend de `utoipa ^5` et `actix-web ^4` → **compatible** |
| `utoipa-actix-web` | **0.1.2** | Intégration utoipa ↔ Actix | dépend de `utoipa ^5`, `actix-web ^4` → **compatible** |
| `tauri-build` | **2.6.3** | Build de la coquille Tauri | aligné sur `tauri` 2.11.x |
| `tokio` | **1.53.1** | Runtime asynchrone | — |
| `serde` | **1.0.229** | Sérialisation | — |
| `uuid` | **1.24.0** | **UUID v7 côté client** (principe VI) | feature `v7` **présente et vérifiée** |
| `rust_decimal` | **1.42.1** | Quantités `NUMERIC` (principe V) | jamais de flottant sur une quantité |
| `redis` | **1.5.0** | Client Redis | — |
| `aws-sdk-s3` | **1.140.0** | Accès Garage **via API S3** (principe II) | — |
| `sentry` | **0.49.0** | Rapport d'erreurs (principe VIII) | — |
| `tracing` | **0.1.44** | Logs structurés corrélés (principe VIII) | — |
| `tracing-subscriber` | **0.3.23** | Souscripteur de logs | — |
| `jsonwebtoken` | **11.0.0** | JWT court + refresh révocable (CPT-01) | — |
| `argon2` | **0.5.3** | Hachage de mot de passe (CPT-01) | — |

> **`uuid` avec la feature `v7` est un prérequis du principe VI**, pas un détail : toute
> écriture porte un UUID v7 généré côté client. La présence de la feature dans `1.24.0` a été
> vérifiée sur l'API crates.io, pas supposée.

### 3.2 Écosystème JavaScript

| Paquet | Version | Rôle |
|---|---|---|
| `@tauri-apps/cli` | **2.11.4** | CLI Tauri (build Android/iOS/desktop) |
| `@tauri-apps/api` | **2.11.1** | Pont JS ↔ Rust — **consommé uniquement par `PlatformAdapter`** (principe VII) |
| `@nuxtjs/i18n` | **10.6.0** | i18n fr/en, fr par défaut (principe VIII) |

> Le décalage `tauri` 2.11.5 (crate) / `@tauri-apps/cli` 2.11.4 est normal : les deux
> versionnements sont indépendants dans la même branche 2.11.x.

### 3.3 Environnement d'exécution

Absent de la liste du principe XI, mais Nuxt ne s'exécute pas sans. Ajouté au gel pour que la
reconstruction soit reproductible.

| Brique | Version | Publiée le | Registre |
|---|---|---|---|
| **Node.js** | **24.18.1** (LTS « Krypton ») | 2026-07-28 | `https://nodejs.org/dist/index.json` |
| **pnpm** | **11.18.0** | — | `https://registry.npmjs.org/pnpm/latest` |

> **LTS, pas la dernière stable.** Node 26.5.1 existe (2026-07-28) mais n'est pas LTS. Pour une
> chaîne de build qui doit tenir 15 mois sans surprise, la LTS est le bon choix ; c'est une
> dérogation raisonnée au « dernière stable » du principe XI, consignée ici comme telle.
> À figer par `.nvmrc` **et** par le champ `engines` du `package.json`.

---

## 4. Où l'épinglage est matérialisé

| Fichier | Porte | Contenu attendu |
|---|---|---|
| `rust-toolchain.toml` | P-20 | `channel = "1.97.1"` — jamais `stable` |
| `Cargo.toml` (workspace) | P-20 | `[workspace.dependencies]` en versions exactes, héritées par tous les crates |
| `Cargo.lock` | P-20 | **Commité**, y compris pour les binaires |
| `package.json` | P-20 | Versions exactes, sans `^` ni `~` ; `engines.node` |
| `pnpm-lock.yaml` | P-20 | **Commité** |
| `.nvmrc` | P-20 | `24.18.1` |
| `compose.yml` | P-20 | Tags d'image exacts : `postgres:18.4`, `redis:8.10.0`, `dxflrs/garage:v2.3.0` — **jamais `latest`** |

> **`Cargo.lock` est commité même pour un binaire** : c'est ce qui rend la reconstruction
> identique à six mois d'écart, condition du support à distance du parc auto-hébergé
> (cadrage §10.2).

---

## 5. Reproduire la vérification

À rejouer à chaque revue mensuelle. Aucune de ces commandes ne dépend d'un cache.

```sh
# Rust — canal stable officiel
curl -sS https://static.rust-lang.org/dist/channel-rust-stable.toml \
  | grep -A2 '^\[pkg\.rust\]'

# Crates Rust — crates.io exige un User-Agent
UA="kaya-version-check (angenor99@gmail.com)"
for c in actix-web sqlx utoipa utoipa-swagger-ui utoipa-actix-web tauri tauri-build \
         tokio serde uuid redis aws-sdk-s3 sentry tracing tracing-subscriber \
         jsonwebtoken argon2 rust_decimal; do
  printf "%-22s " "$c"
  curl -sS -H "User-Agent: $UA" "https://crates.io/api/v1/crates/$c" \
    | python3 -c "import sys,json;print(json.load(sys.stdin)['crate']['max_stable_version'])"
done

# Paquets npm
for p in nuxt tailwindcss @tauri-apps/cli @tauri-apps/api @nuxtjs/i18n pnpm; do
  printf "%-22s " "$p"
  curl -sS "https://registry.npmjs.org/$(echo $p | sed 's|/|%2F|')/latest" \
    | python3 -c "import sys,json;print(json.load(sys.stdin)['version'])"
done

# PostgreSQL — la version « current » est celle à retenir
curl -sS https://www.postgresql.org/versions.json \
  | python3 -c "import sys,json;[print(v['major']+'.'+v['latestMinor'],'EOL',v['eolDate']) for v in json.load(sys.stdin) if v['current']]"

# Redis
curl -sS https://api.github.com/repos/redis/redis/releases \
  | python3 -c "import sys,json;[print(r['tag_name'],r['published_at'][:10]) for r in json.load(sys.stdin)[:3] if not r['prerelease']]"

# Garage
curl -sS "https://git.deuxfleurs.fr/api/v1/repos/Deuxfleurs/garage/releases?limit=3" \
  | python3 -c "import sys,json;[print(r['tag_name'],r['published_at'][:10]) for r in json.load(sys.stdin) if not r['prerelease']]"

# Node.js — LTS, pas la dernière stable
curl -sS https://nodejs.org/dist/index.json \
  | python3 -c "import sys,json;v=[x for x in json.load(sys.stdin) if x['lts']][0];print(v['version'],v['lts'],v['date'])"
```

**Compatibilité inter-crates** — à rejouer si `utoipa` monte de version majeure :

```sh
UA="kaya-version-check (angenor99@gmail.com)"
curl -sS -H "User-Agent: $UA" \
  "https://crates.io/api/v1/crates/utoipa-actix-web/0.1.2/dependencies" \
  | python3 -c "import sys,json;[print(d['crate_id'],d['req']) for d in json.load(sys.stdin)['dependencies'] if d['kind']=='normal']"
```

---

## 6. Journal des gels

| Version | Date | Modification |
|---|---|---|
| 1.0.0 | 2026-07-30 | Gel initial. 10 briques du principe XI + 14 crates + 3 paquets npm + Node LTS et pnpm. Compatibilité `utoipa-swagger-ui` / `utoipa-actix-web` avec `utoipa 5.5.0` vérifiée sur crates.io. Présence de la feature `uuid/v7` vérifiée. Deux points d'attention consignés : rupture d'API sqlx 0.8 → 0.9, et fraîcheur d'un jour de Redis 8.10.0. Dérogation raisonnée sur Node : LTS 24.18.1 retenue plutôt que la stable 26.5.1. |
