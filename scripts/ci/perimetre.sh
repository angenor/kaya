#!/usr/bin/env bash
#
# Filtrage par chemins — quels jobs de CI ont une raison de tourner.
#
# Écrit à la main plutôt que délégué à une action tierce : la porte P-20 interdit toute
# dépendance en intervalle, et une action GitHub non épinglée en est une. Ce script n'a besoin
# que de `git`.
#
# **Ce qui n'est jamais filtré** : les portes statiques (P-02, P-04, P-05b, P-10, P-19, P-20).
# Les sauter sur une modification de documentation laisserait passer une migration réécrite ou
# une maquette copiée sous `app/`.

set -euo pipefail

racine="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$racine"

# Base de comparaison : la branche cible sur une demande de fusion, le commit précédent sinon.
if [[ -n "${GITHUB_BASE_REF:-}" ]]; then
    base="origin/${GITHUB_BASE_REF}"
    git fetch --quiet --depth=1 origin "${GITHUB_BASE_REF}" 2>/dev/null || true
elif [[ -n "${GITHUB_EVENT_BEFORE:-}" && "${GITHUB_EVENT_BEFORE}" != "0000000000000000000000000000000000000000" ]]; then
    base="${GITHUB_EVENT_BEFORE}"
else
    base="HEAD~1"
fi

if ! git rev-parse --verify --quiet "$base" >/dev/null; then
    # Premier commit d'une branche neuve : on ne sait pas comparer, donc on ne filtre rien.
    # Le doute profite à l'exécution — un job sauté par erreur est une porte manquée.
    echo "backend=true"
    echo "app=true"
    echo "gouvernance=true"
    exit 0
fi

fichiers="$(git diff --name-only "$base"...HEAD || git diff --name-only "$base")"

contient() { grep -qE "$1" <<<"$fichiers"; }

backend=false
app=false
gouvernance=false

contient '^(backend/|infra/|scripts/ci/|\.github/workflows/)' && backend=true
# `eslint.config.js` est à la racine depuis que la porte P-15 couvre `web/qr` et `web/console` en
# plus de `app/`. Sans lui dans ce motif, élargir un `ignores` — donc aveugler la porte — ne
# déclencherait aucun job : le pire cas possible pour un filtrage par chemins.
contient '^(app/|web/|clients/|package\.json|pnpm-workspace\.yaml|pnpm-lock\.yaml|eslint\.config\.js|\.nvmrc|\.npmrc|scripts/ci/|\.github/workflows/)' && app=true
contient '^(docs/|\.specify/|specs/)' && gouvernance=true

sortie="${GITHUB_OUTPUT:-/dev/stdout}"
{
    echo "backend=$backend"
    echo "app=$app"
    echo "gouvernance=$gouvernance"
} >>"$sortie"

echo "Périmètre — backend=$backend app=$app gouvernance=$gouvernance" >&2
