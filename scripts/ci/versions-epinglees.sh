#!/usr/bin/env bash
#
# Porte P-20 — aucune dépendance en intervalle ; lockfiles commités et à jour.
#
# Cette porte est écrite et exécutée **à la fin de la phase de mise en place**, pas à la fin du
# cycle. Une porte posée après le code qu'elle protège documente l'existant au lieu de le
# contraindre : si celle-ci arrivait en dernier, un `^` posé au premier jour aurait survécu
# soixante-dix tâches, et le retirer coûterait une revue complète du graphe de dépendances.
#
# Quatre vérifications, dans l'ordre du moins cher au plus cher :
#   1. Aucun intervalle dans les manifestes Rust
#   2. Aucun intervalle dans les manifestes JavaScript
#   3. Aucun tag `latest` ni tag flottant dans les fichiers d'infrastructure
#   4. Les lockfiles suffisent réellement à reconstruire — `--locked` / `--frozen-lockfile`
#
# `docs/versions-gelees.md` fait foi sur les valeurs ; ce script ne vérifie que la **forme**.
# Vérifier les valeurs demanderait d'interroger les registres à chaque construction, ce qui
# transformerait la CI en dépendance réseau — c'est le rôle de la revue mensuelle.

set -euo pipefail

racine="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$racine"

echec=0
signaler() {
    echo "  ✗ $1" >&2
    echec=1
}

echo "── P-20 · 1/4 — manifestes Rust ──────────────────────────────────────────────"

# Le manifeste est **analysé**, pas filtré par motif. Une lecture ligne à ligne confondrait la
# version du paquet lui-même (`[package] version`) et sa `rust-version` avec des contraintes de
# dépendance — trois choses distinctes qui s'écrivent pareil.
while IFS= read -r manifeste; do
    python3 - "$manifeste" <<'PY' || echec=1
import re, sys, tomllib

chemin = sys.argv[1]
with open(chemin, "rb") as f:
    manifeste = tomllib.load(f)

# Seule forme acceptée pour une dépendance publiée : `=x.y.z`.
exact = re.compile(r"^=\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.\-]+)?$")
fautes = []


def examiner(table, origine):
    for nom, contrainte in (table or {}).items():
        if isinstance(contrainte, str):
            valeur = contrainte
        elif isinstance(contrainte, dict):
            # `path` = crate du dépôt ; `workspace = true` = contrainte héritée, déjà vérifiée
            # une fois pour toutes dans `[workspace.dependencies]`. Ni l'une ni l'autre ne porte
            # de numéro à épingler.
            if "path" in contrainte or contrainte.get("workspace") is True:
                continue
            valeur = contrainte.get("version")
            if valeur is None:
                # Dépendance git ou registre alternatif : hors du gel, à refuser franchement.
                fautes.append(f"{origine}.{nom} — dépendance sans version épinglée")
                continue
        else:
            continue
        if not exact.match(valeur):
            fautes.append(f"{origine}.{nom} = « {valeur} »")


for section in ("dependencies", "dev-dependencies", "build-dependencies"):
    examiner(manifeste.get(section), section)
examiner(manifeste.get("workspace", {}).get("dependencies"), "workspace.dependencies")
for cible, table in (manifeste.get("target") or {}).items():
    for section in ("dependencies", "dev-dependencies", "build-dependencies"):
        examiner(table.get(section), f"target.{cible}.{section}")

for faute in fautes:
    print(f"  ✗ {chemin} — version non épinglée : {faute}", file=sys.stderr)
sys.exit(1 if fautes else 0)
PY
done < <(find backend app/src-tauri -name Cargo.toml -not -path '*/target/*' 2>/dev/null | sort)

echo "── P-20 · 2/4 — manifestes JavaScript ────────────────────────────────────────"

while IFS= read -r manifeste; do
    python3 - "$manifeste" <<'PY' || echec=1
import json, re, sys

chemin = sys.argv[1]
with open(chemin, encoding="utf-8") as f:
    paquet = json.load(f)

# `workspace:` désigne un paquet du dépôt, pas une version publiée : il n'y a rien à épingler.
# `catalog:` de même. Tout le reste doit être un numéro exact, sans préfixe.
exact = re.compile(r"^\d+\.\d+\.\d+(?:-[0-9A-Za-z.\-]+)?$")
fautes = []
for section in ("dependencies", "devDependencies", "peerDependencies", "optionalDependencies"):
    for nom, contrainte in (paquet.get(section) or {}).items():
        if contrainte.startswith(("workspace:", "catalog:", "link:", "file:")):
            continue
        if not exact.match(contrainte):
            fautes.append(f"{section}.{nom} = « {contrainte} »")

for faute in fautes:
    print(f"  ✗ {chemin} — version non épinglée : {faute}", file=sys.stderr)
sys.exit(1 if fautes else 0)
PY
done < <(find . -name package.json -not -path '*/node_modules/*' -not -path '*/.nuxt/*' | sort)

echo "── P-20 · 3/4 — images d'infrastructure ──────────────────────────────────────"

while IFS= read -r fichier; do
    if grep -nE '^[[:space:]]*image:[[:space:]]*\S+:latest' "$fichier"; then
        signaler "$fichier — tag « latest » (le gel §4.2 impose un tag exact)"
    fi
    # Une image sans tag du tout vaut `latest` implicitement — le cas le plus facile à manquer.
    if grep -nE '^[[:space:]]*image:[[:space:]]*[^:[:space:]]+[[:space:]]*$' "$fichier"; then
        signaler "$fichier — image sans tag (équivaut à « latest »)"
    fi
done < <(find infra .github -name '*.yml' -o -name '*.yaml' 2>/dev/null | sort)

echo "── P-20 · 4/4 — lockfiles suffisants ─────────────────────────────────────────"

for lock in backend/Cargo.lock pnpm-lock.yaml .nvmrc backend/rust-toolchain.toml; do
    if [[ ! -f "$lock" ]]; then
        signaler "$lock manquant — la reconstruction n'est pas reproductible"
    fi
done

# `rust-toolchain.toml` doit porter un canal exact : « stable » ferait de la reconstruction un
# pari, ce que le support du parc auto-hébergé interdit (cadrage §10.2).
if [[ -f backend/rust-toolchain.toml ]]; then
    canal="$(grep -E '^\s*channel\s*=' backend/rust-toolchain.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')"
    if [[ ! "$canal" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        signaler "backend/rust-toolchain.toml — canal « $canal » (attendu un numéro exact, jamais « stable »)"
    fi
fi

if [[ "${P20_SANS_BUILD:-0}" != "1" ]]; then
    echo "  · cargo build --locked"
    (cd backend && cargo build --workspace --locked --quiet) || signaler "cargo build --locked a échoué : Cargo.lock est périmé"
    echo "  · pnpm install --frozen-lockfile"
    pnpm install --frozen-lockfile --silent || signaler "pnpm install --frozen-lockfile a échoué : pnpm-lock.yaml est périmé"
fi

if [[ $echec -ne 0 ]]; then
    echo >&2
    echo "P-20 ÉCHOUE — une dépendance en intervalle transforme une reconstruction" >&2
    echo "reproductible en pari (principe XI)." >&2
    exit 1
fi

echo "P-20 ✓ — toutes les versions sont épinglées exactement, lockfiles à jour."
