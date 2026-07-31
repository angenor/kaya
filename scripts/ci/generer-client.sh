#!/usr/bin/env bash
#
# Génère le client TypeScript depuis le contrat OpenAPI — **porte P-01**.
#
#   scripts/ci/generer-client.sh              régénère `clients/ts/types.gen.ts`
#   scripts/ci/generer-client.sh --verifier   régénère et ÉCHOUE sur tout diff (mode CI)
#
# ## Ce qui est généré et ce qui ne l'est pas
#
# Un seul fichier est généré : `clients/ts/types.gen.ts`, qui ne contient **que des types**.
# `clients/ts/index.ts` est écrit à la main et ne se régénère jamais ; `openapi-fetch` est une
# bibliothèque installée, pas un artefact.
#
# C'est le critère qui a fait retenir `openapi-typescript` au gel 1.0.3 plutôt qu'un générateur de
# SDK complet : la surface soumise à P-01 se limite au strict dérivé du contrat. Un générateur qui
# produit des fichiers de client entiers multiplie les occasions de faux positif, et une porte qui
# émet des faux positifs est désactivée sous trois semaines.
#
# ## Les deux vérifications exigées par le gel avant de clore US5
#
#   1. **Déterminisme d'octet** — deux exécutions successives sur le même contrat produisent deux
#      fichiers identiques. Vérifié par `cmp`, pas par lecture.
#   2. **Ordre stable des membres**, indépendant de l'ordre de découverte des routes par utoipa.
#
# `--verifier` exécute les deux avant de comparer au fichier commité.

set -euo pipefail

racine="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$racine"

MODE_VERIFICATION=0
[[ "${1:-}" == "--verifier" ]] && MODE_VERIFICATION=1

SORTIE="clients/ts/types.gen.ts"
TEMPORAIRE="$(mktemp -d)"
trap 'rm -rf "$TEMPORAIRE"' EXIT

echo "── Extraction du contrat OpenAPI ─────────────────────────────────────────────"
# Aucune base de données requise : le contrat est un produit du code, pas de l'exécution.
( cd backend && cargo run --quiet --locked -p kaya-api --bin contrat ) > "$TEMPORAIRE/openapi.json"

octets=$(wc -c < "$TEMPORAIRE/openapi.json" | tr -d ' ')
chemins=$(python3 -c "
import json,sys
print(len(json.load(open('$TEMPORAIRE/openapi.json'))['paths']))
")
echo "  contrat : ${octets} octets, ${chemins} chemin(s)"

if [[ "$chemins" -eq 0 ]]; then
    echo "  ✗ le contrat ne déclare aucun chemin — la porte P-01 ne vérifierait rien" >&2
    exit 1
fi

echo "── Génération des types ──────────────────────────────────────────────────────"
generer() {
    # `--filter @kaya/client` : le générateur est une dépendance de développement du paquet
    # `clients/ts`, pas de la racine. L'invoquer depuis la racine échoue sur « command not
    # found », message qui ne dit pas où l'outil se trouve réellement.
    pnpm --silent --filter @kaya/client exec openapi-typescript "$1" --output "$2"
}
generer "$TEMPORAIRE/openapi.json" "$TEMPORAIRE/types-1.gen.ts"

if [[ $MODE_VERIFICATION -eq 1 ]]; then
    echo "── Exigence 1 du gel — déterminisme d'octet ──────────────────────────────────"
    generer "$TEMPORAIRE/openapi.json" "$TEMPORAIRE/types-2.gen.ts"
    if ! cmp -s "$TEMPORAIRE/types-1.gen.ts" "$TEMPORAIRE/types-2.gen.ts"; then
        echo "  ✗ deux exécutions sur le MÊME contrat produisent deux fichiers différents." >&2
        echo "    P-01 échouerait au hasard, et serait désactivée sous trois semaines." >&2
        diff "$TEMPORAIRE/types-1.gen.ts" "$TEMPORAIRE/types-2.gen.ts" | head -20 >&2
        exit 1
    fi
    echo "  ✓ deux exécutions, deux fichiers identiques (cmp)"

    echo "── Exigence 2 du gel — ordre stable des membres ──────────────────────────────"
    # Un chemin est ajouté au contrat **en dernière position** du document, comme le ferait un
    # endpoint déclaré en fin de fichier Rust. Le diff doit rester LOCAL : si l'ordre de sortie
    # dépendait de l'ordre de découverte, tout le fichier serait remanié et chaque ajout
    # d'endpoint produirait un diff illisible.
    python3 - "$TEMPORAIRE/openapi.json" "$TEMPORAIRE/openapi-augmente.json" <<'PY'
import json, sys
contrat = json.load(open(sys.argv[1]))
contrat["paths"]["/zzz-sonde-ordre-stable"] = {
    "get": {
        "tags": ["systeme"],
        "responses": {"200": {"description": "sonde d'ordre, jamais servie"}},
    }
}
json.dump(contrat, open(sys.argv[2], "w"), ensure_ascii=False, indent=2)
PY
    generer "$TEMPORAIRE/openapi-augmente.json" "$TEMPORAIRE/types-augmente.gen.ts"

    lignes_changees=$(diff "$TEMPORAIRE/types-1.gen.ts" "$TEMPORAIRE/types-augmente.gen.ts" \
        | grep -cE '^[<>]' || true)
    lignes_totales=$(wc -l < "$TEMPORAIRE/types-1.gen.ts" | tr -d ' ')
    echo "  diff local : ${lignes_changees} ligne(s) changée(s) sur ${lignes_totales}"
    if [[ "$lignes_totales" -gt 0 ]] && [[ "$lignes_changees" -gt $((lignes_totales / 2)) ]]; then
        echo "  ✗ l'ajout d'un seul endpoint remanie plus de la moitié du fichier." >&2
        echo "    L'ordre de sortie dépend de l'ordre de découverte — voir les features" >&2
        echo "    preserve_order / preserve_path_order d'utoipa dans backend/Cargo.toml." >&2
        exit 1
    fi
    echo "  ✓ un endpoint ajouté ne remanie pas le fichier"
fi

cp "$TEMPORAIRE/types-1.gen.ts" "$SORTIE"

if [[ $MODE_VERIFICATION -eq 1 ]]; then
    echo "── Porte P-01 — le client commité est-il à jour ? ────────────────────────────"
    if ! git diff --exit-code -- "$SORTIE"; then
        echo >&2
        echo "P-01 ÉCHOUE — le client TypeScript commité diffère de ce que produit le contrat." >&2
        echo "Le contrat est généré depuis le code, le client depuis le contrat (principe I(a))." >&2
        echo "Régénérer et commiter :  scripts/ci/generer-client.sh" >&2
        exit 1
    fi
    echo "P-01 ✓ — client identique au contrat."
else
    echo "Client régénéré → $SORTIE"
fi
