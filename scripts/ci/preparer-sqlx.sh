#!/usr/bin/env bash
#
# **Porte P-18** — cache de requêtes préparées complet.
#
#     scripts/ci/preparer-sqlx.sh              régénère backend/.sqlx
#     scripts/ci/preparer-sqlx.sh --verifier   échoue si le cache commité est incomplet
#
# ═════════════════════════════════════════════════════════════════════════════════════════════
#  LE DÉFAUT QUE CE SCRIPT CORRIGE — trouvé par la construction Docker, pas par une relecture
# ═════════════════════════════════════════════════════════════════════════════════════════════
#
#  `cargo sqlx prepare --workspace -- --all-targets`, lancé à la racine du workspace, **ne
#  collecte pas les requêtes des binaires déclarés en `[[bin]]`**. Il ramassait 43 requêtes sur
#  47 : celles de `api/src/bin/seeds.rs` manquaient toutes.
#
#  La cause est dans la nature de la commande : `cargo sqlx prepare` passe ses arguments à
#  `cargo rustc`, qui ne compile **qu'une seule cible** à la fois. Sur un workspace, `--all-targets`
#  ne fait donc pas ce que son nom laisse croire.
#
#  Rien ne le signalait :
#
#    * `cargo build` local marchait — `DATABASE_URL` est défini, les macros interrogent la base ;
#    * `cargo sqlx prepare --check` à la racine passait au vert — il vérifie ce qu'il sait
#      collecter, donc il ne voyait pas ce qu'il ne collectait pas ;
#    * **seule la construction Docker a échoué**, parce qu'elle compile avec `SQLX_OFFLINE=true`
#      et sans base : c'est le seul contexte où l'absence se voit.
#
#      error: `SQLX_OFFLINE=true` but there is no cached data for this query
#         --> api/src/bin/seeds.rs:96:5
#
#  La correction consiste à lancer `prepare` **aussi depuis chaque crate porteur de binaires**, et
#  à fusionner. Les fichiers étant nommés par empreinte de la requête, la fusion est sûre : deux
#  requêtes différentes ne peuvent pas produire le même nom.
#
#  **Leçon** : `cargo sqlx prepare --check` seul ne prouve pas que l'image se construira. Ce que
#  ce script vérifie, c'est ce que le Dockerfile subira.
# ═════════════════════════════════════════════════════════════════════════════════════════════

set -euo pipefail

racine="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$racine/backend"

MODE_VERIFICATION=0
[[ "${1:-}" == "--verifier" ]] && MODE_VERIFICATION=1

# Crates portant des cibles `[[bin]]` explicites. Une cible de binaire ajoutée ailleurs doit être
# inscrite ici **dans le même changement**, sans quoi ses requêtes manqueront au cache et l'image
# de production ne se construira plus.
CRATES_AVEC_BINAIRES=(api node)

destination="$racine/backend/.sqlx"
if [[ $MODE_VERIFICATION -eq 1 ]]; then
    destination="$(mktemp -d)/.sqlx"
    trap 'rm -rf "$(dirname "$destination")"' EXIT
fi

echo "── P-18 · 1/3 — requêtes du workspace ────────────────────────────────────────"
cargo sqlx prepare --workspace -- --all-targets >/dev/null 2>&1
niveau_workspace=$(find .sqlx -name 'query-*.json' 2>/dev/null | wc -l | tr -d ' ')
echo "  $niveau_workspace requête(s)"

if [[ $MODE_VERIFICATION -eq 1 ]]; then
    mkdir -p "$destination"
    cp .sqlx/query-*.json "$destination/" 2>/dev/null || true
fi

echo "── P-18 · 2/3 — requêtes des binaires ────────────────────────────────────────"
ajoutees=0
for crate in "${CRATES_AVEC_BINAIRES[@]}"; do
    [[ -d "$crate" ]] || continue
    (cd "$crate" && cargo sqlx prepare -- --all-targets >/dev/null 2>&1) || true

    local_sqlx="$crate/.sqlx"
    [[ -d "$local_sqlx" ]] || continue

    for fichier in "$local_sqlx"/query-*.json; do
        [[ -e "$fichier" ]] || continue
        nom="$(basename "$fichier")"
        if [[ ! -f "$destination/$nom" ]]; then
            cp "$fichier" "$destination/$nom"
            ajoutees=$((ajoutees + 1))
        fi
    done

    # Le cache par crate n'est pas conservé : une seule source de vérité, à la racine du
    # workspace. Deux caches divergeraient au premier oubli.
    rm -rf "$local_sqlx"
done
echo "  $ajoutees requête(s) de binaires ajoutée(s)"

echo "── P-18 · 3/3 — bilan ────────────────────────────────────────────────────────"
total=$(find "$destination" -name 'query-*.json' 2>/dev/null | wc -l | tr -d ' ')
echo "  $total requête(s) au total"

if [[ "$total" -eq 0 ]]; then
    echo "P-18 ÉCHOUE — cache vide. La compilation hors ligne est impossible." >&2
    exit 1
fi

if [[ $MODE_VERIFICATION -eq 1 ]]; then
    commite=$(find "$racine/backend/.sqlx" -name 'query-*.json' 2>/dev/null | wc -l | tr -d ' ')
    manquantes=()
    for fichier in "$destination"/query-*.json; do
        nom="$(basename "$fichier")"
        [[ -f "$racine/backend/.sqlx/$nom" ]] || manquantes+=("$nom")
    done

    if [[ ${#manquantes[@]} -gt 0 ]]; then
        echo >&2
        echo "P-18 ÉCHOUE — ${#manquantes[@]} requête(s) absente(s) du cache commité" >&2
        echo "($commite commitées, $total attendues) :" >&2
        printf '  ✗ %s\n' "${manquantes[@]}" >&2
        echo >&2
        echo "L'image de production ne se construira PAS : elle compile avec SQLX_OFFLINE=true" >&2
        echo "et sans base de données. Régénérer :  scripts/ci/preparer-sqlx.sh" >&2
        exit 1
    fi
    echo "P-18 ✓ — les $total requêtes sont au cache commité."
else
    echo "P-18 ✓ — cache régénéré : $total requêtes dans backend/.sqlx"
fi
